//! `wildside ledger`: search each level for an exit on native, then confirm
//! the route on the ROM. The headline is levels confirmed out of 103.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, ensure};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use wildside_core::{FrameInput, GameMode, LevelId, NativeRuntime, NativeTraceFrame};
use wildside_data::AssetManifest;

use crate::input_gen::Choices;
use crate::parity::{
    OracleRunner, PLAYABLE_LEVELS, Signature, exit_matches, first_divergence,
    native_transition_frame, route_scenario,
};
use crate::scenarios::{
    DynamicScenario, NativeDynamicRunOptions, run_native_dynamic_scenario,
    try_for_each_native_dynamic_frame,
};

const LEDGER_PATH: &str = "docs/source-port/level-ledger.yaml";
const ROUTE_DIR: &str = "tests/data/ledger-routes";

/// Held masks the explorer bursts: Right/Left with B (jump) and C (run),
/// Down, Up and A. Start is excluded so routes never pause.
const MACROS: [u8; 18] = [
    0x08, 0x28, 0x18, 0x38, 0x04, 0x24, 0x14, 0x34, 0x10, 0x20, 0x02, 0x01, 0x40, 0x00, 0x48, 0x44,
    0x12, 0x11,
];
const BURSTS_PER_VISIT: usize = 12;
/// Archive cells are 24px squares per helmet form.
const CELL_PIXELS: u32 = 24;
/// Measured resident cost of one cell's runtime snapshot.
const CELL_MEGABYTES: f64 = 0.35;
/// Explorer threads. Stepping and snapshotting are allocation-bound: 4
/// threads measured ~21k frames/s in total, 8 threads ~18k with most time in
/// the kernel.
const EXPLORER_THREADS: usize = 4;
/// Native frames replayed after the exit to reach the mode/level transition.
const TRANSITION_SEARCH: usize = 600;
/// Frames kept after the transition so the oracle shows it.
const TRANSITION_TAIL: usize = 30;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExitKind {
    /// ASM `level_complete`: the flag at the end of the level.
    Flag,
    GameComplete,
    /// `current_level` changed, e.g. a teleporter to another level.
    ToLevel(#[serde(with = "crate::fuzz::hex_level")] u8),
}

/// How a native frame leaves the level it started in, if it does.
fn classify_exit(start_level: LevelId, frame: &NativeTraceFrame) -> Option<ExitKind> {
    if frame.game_complete {
        Some(ExitKind::GameComplete)
    } else if frame.level_complete {
        Some(ExitKind::Flag)
    } else if frame.current_level != start_level {
        Some(ExitKind::ToLevel(frame.current_level.0))
    } else {
        None
    }
}

/// A lost life ends the visit: death routes are never exits.
fn is_death(start_lives: u16, frame: &NativeTraceFrame) -> bool {
    frame.number_lives < start_lives || frame.game_over || frame.game_mode != GameMode::InGame
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CellKey {
    x: u32,
    y: u32,
    helmet: u16,
}

impl CellKey {
    fn of(frame: &NativeTraceFrame) -> Option<Self> {
        Some(Self {
            x: (frame.kid_x_pos? >> 16) / CELL_PIXELS,
            y: (frame.kid_y_pos? >> 16) / CELL_PIXELS,
            helmet: frame.current_helmet_id,
        })
    }
}

struct Cell<T> {
    runtime: Arc<T>,
    inputs: Vec<u8>,
    hit_points: u16,
    chosen: u32,
}

/// Go-Explore archive: the best known way to stand in each cell, holding a
/// runtime snapshot `T` per cell.
struct Archive<T> {
    cells: BTreeMap<CellKey, Cell<T>>,
    cap: usize,
}

impl<T> Archive<T> {
    /// Offers a state for `key`. It replaces the cell's state when it has
    /// more hit points, or as many in fewer frames. At the cap a new cell
    /// evicts the most-visited cell, if any cell has been visited, so the
    /// frontier keeps moving in bounded memory. The snapshot is taken only
    /// when the state is kept.
    fn offer(
        &mut self,
        key: CellKey,
        hit_points: u16,
        inputs: &[u8],
        snapshot: impl FnOnce() -> T,
    ) -> bool {
        if self.cells.len() >= self.cap && !self.cells.contains_key(&key) {
            let most_visited = self
                .cells
                .iter()
                .filter(|(_, cell)| cell.chosen > 0)
                .max_by_key(|(cell_key, cell)| (cell.chosen, std::cmp::Reverse(**cell_key)))
                .map(|(cell_key, _)| *cell_key);
            let Some(evicted) = most_visited else {
                return false;
            };
            self.cells.remove(&evicted);
        }
        if let Some(cell) = self.cells.get_mut(&key) {
            let better = hit_points > cell.hit_points
                || (hit_points == cell.hit_points && inputs.len() < cell.inputs.len());
            if better {
                cell.runtime = Arc::new(snapshot());
                cell.inputs = inputs.to_vec();
                cell.hit_points = hit_points;
            }
            return better;
        }
        self.cells.insert(
            key,
            Cell {
                runtime: Arc::new(snapshot()),
                inputs: inputs.to_vec(),
                hit_points,
                chosen: 0,
            },
        );
        true
    }

    /// Picks uniformly among the least-chosen cells.
    fn select(&mut self, choices: &mut Choices) -> (Arc<T>, Vec<u8>) {
        let least = self
            .cells
            .values()
            .map(|cell| cell.chosen)
            .min()
            .expect("seeded archive");
        let count = self
            .cells
            .values()
            .filter(|cell| cell.chosen == least)
            .count();
        let cell = self
            .cells
            .values_mut()
            .filter(|cell| cell.chosen == least)
            .nth(choices.index(count))
            .expect("least-chosen cell");
        cell.chosen += 1;
        (Arc::clone(&cell.runtime), cell.inputs.clone())
    }
}

/// The settled-warp runtime after scenario frame 0: the state a route script
/// starts from, since script byte `i` is consumed on frame `i + 1`.
fn settled_start(manifest: &AssetManifest, level: u8) -> Result<(NativeRuntime, NativeTraceFrame)> {
    let scenario = route_scenario(format!("ledger-{level:02x}"), level, Vec::new());
    let mut start = None;
    try_for_each_native_dynamic_frame(
        manifest.clone(),
        &scenario,
        NativeDynamicRunOptions::default(),
        |_, runtime, _, trace| {
            start = Some((runtime.clone(), trace.clone()));
            Ok(())
        },
    )?;
    let (runtime, frame) = start.context("settled warp produced no frame")?;
    ensure!(
        frame.game_mode == GameMode::InGame && frame.kid_x_pos.is_some(),
        "level {level:#04x} settled warp is not live gameplay"
    );
    Ok((runtime, frame))
}

struct Search {
    found: Option<(Vec<u8>, ExitKind)>,
    cells: usize,
    frames: u64,
}

/// One deterministic explorer: a fixed seed, frame budget and cell cap.
/// `abandon` is polled between visits so a job can stop once its result can
/// no longer be chosen; it never changes which route a job finds.
fn explore(
    start: &NativeRuntime,
    start_frame: &NativeTraceFrame,
    seed: u64,
    budget: u64,
    cap: usize,
    abandon: impl Fn() -> bool,
) -> Search {
    let mut archive = Archive {
        cells: BTreeMap::new(),
        cap,
    };
    let key = CellKey::of(start_frame).expect("live start has a position");
    archive.offer(key, start_frame.number_hitpoints, &[], || start.clone());
    let mut choices = Choices(seed);
    let mut frames = 0;
    while frames < budget && !abandon() {
        let (runtime, mut inputs) = archive.select(&mut choices);
        let mut runtime = (*runtime).clone();
        let mut mask = MACROS[choices.index(MACROS.len())];
        'visit: for _ in 0..BURSTS_PER_VISIT {
            if choices.index(10) < 3 {
                mask = MACROS[choices.index(MACROS.len())];
            }
            let mut last = None;
            for _ in 0..4 + choices.index(21) {
                inputs.push(mask);
                let frame = runtime.step_frame(FrameInput::from_mask(mask));
                frames += 1;
                if let Some(exit) = classify_exit(start_frame.current_level, &frame) {
                    return Search {
                        found: Some((inputs, exit)),
                        cells: archive.cells.len(),
                        frames,
                    };
                }
                if is_death(start_frame.number_lives, &frame) {
                    break 'visit;
                }
                last = Some(frame);
            }
            let Some(frame) = last else { break };
            let Some(key) = CellKey::of(&frame) else {
                break;
            };
            archive.offer(key, frame.number_hitpoints, &inputs, || runtime.clone());
        }
    }
    Search {
        found: None,
        cells: archive.cells.len(),
        frames,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LevelStatus {
    /// Exited on native and the ROM replay matched through the exit.
    Confirmed,
    /// Exited on native; the ROM replay diverged first (`blocked_by`).
    NativeExit,
    /// No exit within the search budget.
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Blocker {
    pub(crate) id: String,
    pub(crate) frame: usize,
    pub(crate) signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelEntry {
    #[serde(with = "crate::fuzz::hex_level")]
    pub(crate) level: u8,
    pub(crate) status: LevelStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) exit: Option<ExitKind>,
    /// Frame on which native reports the exit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) exit_frame: Option<usize>,
    /// Frame on which the game mode or level changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) transition_frame: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) blocked_by: Option<Blocker>,
    pub(crate) run: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Ledger {
    pub(crate) revision: String,
    pub(crate) confirmed: usize,
    pub(crate) native_exits: usize,
    pub(crate) playable: usize,
    pub(crate) levels: Vec<LevelEntry>,
}

impl Ledger {
    fn merge(&mut self, entries: Vec<LevelEntry>, revision: String) {
        let mut levels = self
            .levels
            .drain(..)
            .map(|entry| (entry.level, entry))
            .collect::<BTreeMap<_, _>>();
        levels.extend(entries.into_iter().map(|entry| (entry.level, entry)));
        self.levels = levels.into_values().collect();
        self.revision = revision;
        self.playable = PLAYABLE_LEVELS.len();
        self.confirmed = self
            .levels
            .iter()
            .filter(|entry| entry.status == LevelStatus::Confirmed)
            .count();
        self.native_exits = self
            .levels
            .iter()
            .filter(|entry| entry.status != LevelStatus::NotFound)
            .count();
    }
}

pub(crate) struct LedgerArgs {
    pub(crate) levels: Vec<u8>,
    pub(crate) budget: u64,
    pub(crate) jobs: u64,
    pub(crate) cap: usize,
}

pub(crate) fn run_ledger(
    repo_root: &Path,
    rom: &Path,
    manifest: &AssetManifest,
    args: &LedgerArgs,
    write: bool,
) -> Result<()> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string();
    let runs = repo_root.join(".cache/ledger-runs");
    let run_dir = runs.join(&stamp);
    let oracle = OracleRunner::new(repo_root, rom, &run_dir.join("jobs"))?;
    fs::create_dir_all(run_dir.join("routes"))?;

    let starts = args
        .levels
        .iter()
        .map(|&level| Ok((level, settled_start(manifest, level)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    // Per level, the lowest seed that has found an exit so far. A job whose
    // seed is above it can stop: the lowest successful seed is the result.
    let lowest_found = args
        .levels
        .iter()
        .map(|&level| (level, AtomicU64::new(u64::MAX)))
        .collect::<BTreeMap<_, _>>();
    let jobs = args
        .levels
        .iter()
        .flat_map(|&level| (0..args.jobs).map(move |seed| (level, seed)))
        .collect::<Vec<_>>();
    println!(
        "ledger: {} levels x {} seeds, {} frames each, <= {} cells (~{:.0} MB) per job, {EXPLORER_THREADS} at a time",
        args.levels.len(),
        args.jobs,
        args.budget,
        args.cap,
        f64::from(u32::try_from(args.cap).unwrap_or(u32::MAX)) * CELL_MEGABYTES
    );
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(EXPLORER_THREADS)
        .build()?;
    let searches = pool.install(|| {
        jobs.par_iter()
            .map(|&(level, seed)| {
                let (runtime, frame) = &starts[&level];
                let lowest = &lowest_found[&level];
                let search = explore(
                    runtime,
                    frame,
                    (u64::from(level) << 32) | seed,
                    args.budget,
                    args.cap,
                    || lowest.load(Ordering::Relaxed) < seed,
                );
                if search.found.is_some() {
                    lowest.fetch_min(seed, Ordering::Relaxed);
                }
                println!(
                    "ledger-{level:02x}-{seed}: {} after {} frames, {} cells",
                    match &search.found {
                        Some((_, exit)) => format!("{exit:?}"),
                        None if search.frames < args.budget => {
                            "abandoned for a lower seed's exit".to_owned()
                        }
                        None => "no exit".to_owned(),
                    },
                    search.frames,
                    search.cells
                );
                ((level, seed), search)
            })
            .collect::<BTreeMap<_, _>>()
    });

    let entries = args
        .levels
        .iter()
        .map(|&level| {
            let found = searches
                .range((level, 0)..=(level, u64::MAX))
                .find_map(|(_, search)| search.found.clone());
            let entry = match found {
                Some((inputs, exit)) => {
                    confirm_route(manifest, &oracle, level, &inputs, exit, &stamp, &run_dir)?
                }
                None => LevelEntry {
                    level,
                    status: LevelStatus::NotFound,
                    exit: None,
                    exit_frame: None,
                    transition_frame: None,
                    blocked_by: None,
                    run: stamp.clone(),
                },
            };
            println!("{}", entry_line(&entry));
            Ok(entry)
        })
        .collect::<Result<Vec<_>>>()?;
    fs::remove_dir_all(run_dir.join("jobs")).ok();
    fs::write(
        run_dir.join("report.json"),
        serde_json::to_vec_pretty(&entries)?,
    )?;
    let latest = runs.join("latest");
    let _ = fs::remove_file(&latest);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&run_dir, &latest).ok();

    if write {
        write_run_results(repo_root, manifest, &oracle, &run_dir, entries)?;
    }
    println!("ledger report: {}", run_dir.join("report.json").display());
    Ok(())
}

fn write_run_results(
    repo_root: &Path,
    manifest: &AssetManifest,
    oracle: &OracleRunner,
    run_dir: &Path,
    entries: Vec<LevelEntry>,
) -> Result<()> {
    let mut ledger = load_ledger(repo_root).unwrap_or_default();
    let mut updates = Vec::with_capacity(entries.len());
    for entry in entries {
        if entry.status != LevelStatus::Confirmed
            && let Some(previous) = ledger
                .levels
                .iter()
                .find(|previous| previous.level == entry.level)
                .filter(|previous| previous.status == LevelStatus::Confirmed)
        {
            verify_route(repo_root, manifest, oracle, previous)
                .with_context(|| format!("revalidate saved exit for {:#04x}", entry.level))?;
            println!("{:#04x}: retained ROM-revalidated exit", entry.level);
        } else {
            updates.push(entry);
        }
    }
    for entry in &updates {
        if entry.status == LevelStatus::Confirmed {
            fs::create_dir_all(repo_root.join(ROUTE_DIR))?;
            fs::copy(
                run_dir.join("routes").join(route_file(entry.level)),
                repo_root.join(ROUTE_DIR).join(route_file(entry.level)),
            )?;
        }
    }
    ledger.merge(updates, crate::fuzz::git_revision(repo_root));
    write_ledger(repo_root, &ledger)?;
    println!(
        "ledger: {}/{} confirmed, {} native exits",
        ledger.confirmed, ledger.playable, ledger.native_exits
    );
    Ok(())
}

fn route_file(level: u8) -> String {
    format!("{level:02x}.json")
}

/// Replays a found route on native to find its transition frame, trims the
/// script to just past it, then replays it on the ROM.
fn confirm_route(
    manifest: &AssetManifest,
    oracle: &OracleRunner,
    level: u8,
    inputs: &[u8],
    exit: ExitKind,
    run: &str,
    run_dir: &Path,
) -> Result<LevelEntry> {
    let slug = format!("ledger-{level:02x}");
    let mut script = inputs.to_vec();
    script.resize(inputs.len() + TRANSITION_SEARCH, 0);
    let native = run_native_dynamic_scenario(
        manifest.clone(),
        &route_scenario(slug.clone(), level, script.clone()),
        NativeDynamicRunOptions::default(),
    )?;
    let exit_frame = inputs.len();
    ensure!(
        classify_exit(LevelId(level), &native[exit_frame]).as_ref() == Some(&exit),
        "level {level:#04x} route does not replay its exit on frame {exit_frame}"
    );
    let transition = native_transition_frame(&native);
    script.truncate(transition.map_or(script.len(), |frame| frame + TRANSITION_TAIL));
    let scenario = route_scenario(slug.clone(), level, script);
    fs::write(
        run_dir.join("routes").join(route_file(level)),
        serde_json::to_vec_pretty(&scenario)?,
    )?;
    let (status, blocked_by) = check_on_rom(manifest, oracle, &scenario, level)?;
    Ok(LevelEntry {
        level,
        status,
        exit: Some(exit),
        exit_frame: Some(exit_frame),
        transition_frame: transition,
        blocked_by,
        run: run.to_owned(),
    })
}

/// Confirmed when no blocking divergence precedes the transition and native
/// and the ROM transition on the same frame.
fn check_on_rom(
    manifest: &AssetManifest,
    oracle: &OracleRunner,
    scenario: &DynamicScenario,
    level: u8,
) -> Result<(LevelStatus, Option<Blocker>)> {
    let native = run_native_dynamic_scenario(
        manifest.clone(),
        scenario,
        NativeDynamicRunOptions::default(),
    )?;
    let rom = oracle.run(scenario, &scenario.slug)?;
    let transition = native_transition_frame(&native);
    let divergence = first_divergence(&native, &rom, level)
        .filter(|divergence| transition.is_none_or(|frame| divergence.frame < frame));
    Ok(match divergence {
        Some(divergence) => {
            let signature = Signature::new(&divergence, &native, &rom, &scenario.script);
            (
                LevelStatus::NativeExit,
                Some(Blocker {
                    id: signature.id(),
                    frame: divergence.frame,
                    signature,
                }),
            )
        }
        None if exit_matches(&native, &rom) => (LevelStatus::Confirmed, None),
        // The states agree but the transition frames differ or were not seen.
        None => (LevelStatus::NativeExit, None),
    })
}

/// Replays every checked-in confirmed route: native must reach the recorded
/// exit on the recorded frame and the ROM must still confirm it.
pub(crate) fn verify_ledger(repo_root: &Path, rom: &Path, manifest: &AssetManifest) -> Result<()> {
    let ledger = load_ledger(repo_root)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let oracle = OracleRunner::new(
        repo_root,
        rom,
        &repo_root
            .join(".cache/ledger-runs/verify")
            .join(stamp.to_string()),
    )?;
    let confirmed = ledger
        .levels
        .iter()
        .filter(|entry| entry.status == LevelStatus::Confirmed)
        .collect::<Vec<_>>();
    let failures = confirmed
        .par_iter()
        .filter_map(|entry| {
            let result = verify_route(repo_root, manifest, &oracle, entry);
            let line = match &result {
                Ok(()) => format!("{:#04x}: confirmed", entry.level),
                Err(error) => format!("{:#04x}: FAILED {error:#}", entry.level),
            };
            println!("{line}");
            result.err().map(|_| entry.level)
        })
        .collect::<Vec<_>>();
    ensure!(
        failures.is_empty(),
        "{} of {} ledger routes no longer confirm",
        failures.len(),
        confirmed.len()
    );
    println!("verify-ledger: {} routes confirmed", confirmed.len());
    Ok(())
}

fn verify_route(
    repo_root: &Path,
    manifest: &AssetManifest,
    oracle: &OracleRunner,
    entry: &LevelEntry,
) -> Result<()> {
    let path = repo_root.join(ROUTE_DIR).join(route_file(entry.level));
    let scenario: DynamicScenario = serde_json::from_slice(&fs::read(&path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    let native = run_native_dynamic_scenario(
        manifest.clone(),
        &scenario,
        NativeDynamicRunOptions::default(),
    )?;
    let first_exit = native.iter().enumerate().find_map(|(frame, trace)| {
        classify_exit(LevelId(entry.level), trace).map(|exit| (frame, exit))
    });
    ensure!(
        first_exit == entry.exit_frame.zip(entry.exit.clone()),
        "native exit {first_exit:?}, ledger records {:?} at {:?}",
        entry.exit,
        entry.exit_frame
    );
    let (status, blocked_by) = check_on_rom(manifest, oracle, &scenario, entry.level)?;
    ensure!(
        status == LevelStatus::Confirmed,
        "ROM replay now {status:?}{}",
        blocked_by.map_or_else(String::new, |blocker| format!(
            " at frame {} ({})",
            blocker.frame, blocker.signature
        ))
    );
    Ok(())
}

fn entry_line(entry: &LevelEntry) -> String {
    let mut line = format!("{:#04x}: {:?}", entry.level, entry.status);
    if let (Some(exit), Some(frame)) = (&entry.exit, entry.exit_frame) {
        let _ = write!(line, " {exit:?} at frame {frame}");
    }
    if let Some(blocker) = &entry.blocked_by {
        let _ = write!(
            line,
            ", blocked by {} at frame {}: {}",
            blocker.id, blocker.frame, blocker.signature
        );
    }
    line
}

pub(crate) fn load_ledger(repo_root: &Path) -> Result<Ledger> {
    let path = repo_root.join(LEDGER_PATH);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_yaml::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn write_ledger(repo_root: &Path, ledger: &Ledger) -> Result<()> {
    let path = repo_root.join(LEDGER_PATH);
    let body = format!(
        "# Generated by `wildside ledger run --write`; do not edit by hand.\n{}",
        serde_yaml::to_string(ledger)?
    );
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> NativeTraceFrame {
        NativeTraceFrame {
            game_mode: GameMode::InGame,
            current_level: LevelId(3),
            number_lives: 3,
            number_hitpoints: 2,
            kid_x_pos: Some(100 << 16),
            kid_y_pos: Some(50 << 16),
            ..NativeTraceFrame::default()
        }
    }

    #[test]
    fn exits_are_flag_game_complete_or_a_level_change_and_deaths_are_not() {
        let start = LevelId(3);
        assert_eq!(classify_exit(start, &frame()), None);
        let mut flag = frame();
        flag.level_complete = true;
        assert_eq!(classify_exit(start, &flag), Some(ExitKind::Flag));
        let mut teleport = frame();
        teleport.current_level = LevelId(0x4a);
        assert_eq!(
            classify_exit(start, &teleport),
            Some(ExitKind::ToLevel(0x4a))
        );
        let mut end = flag.clone();
        end.game_complete = true;
        assert_eq!(classify_exit(start, &end), Some(ExitKind::GameComplete));

        assert!(!is_death(3, &frame()));
        let mut lost = frame();
        lost.number_lives = 2;
        assert!(is_death(3, &lost));
        let mut over = frame();
        over.game_over = true;
        assert!(is_death(3, &over));
    }

    #[test]
    fn cells_prefer_more_hit_points_then_shorter_routes_under_a_cap() {
        let mut archive = Archive::<u8> {
            cells: BTreeMap::new(),
            cap: 2,
        };
        let key = |x| CellKey { x, y: 0, helmet: 0 };
        assert!(archive.offer(key(0), 2, &[1; 10], || 0));
        assert!(!archive.offer(key(0), 2, &[1; 12], || unreachable!()));
        assert!(archive.offer(key(0), 2, &[2; 8], || 0));
        assert!(archive.offer(key(0), 3, &[3; 20], || 0));
        assert!(!archive.offer(key(0), 2, &[4; 1], || unreachable!()));
        assert_eq!(archive.cells[&key(0)].inputs, vec![3; 20]);
        assert!(archive.offer(key(1), 1, &[5], || 0));
        assert!(
            !archive.offer(key(2), 9, &[6], || unreachable!()),
            "at the cap, unvisited cells are never evicted"
        );

        let mut choices = Choices(4);
        let first = archive.select(&mut choices).1;
        let second = archive.select(&mut choices).1;
        assert_ne!(first, second, "least-chosen cells are visited first");
        archive.select(&mut choices);
        let busiest = *archive
            .cells
            .iter()
            .max_by_key(|(_, cell)| cell.chosen)
            .unwrap()
            .0;
        assert!(archive.offer(key(2), 9, &[6], || 0));
        assert!(
            !archive.cells.contains_key(&busiest),
            "most-visited cell evicted"
        );
        assert_eq!(archive.cells.len(), 2);
    }

    #[test]
    fn ledger_merge_keeps_other_levels_and_counts_the_headline() {
        let entry = |level, status| LevelEntry {
            level,
            status,
            exit: None,
            exit_frame: None,
            transition_frame: None,
            blocked_by: None,
            run: "1".to_owned(),
        };
        let mut ledger = Ledger::default();
        ledger.merge(
            vec![
                entry(0, LevelStatus::Confirmed),
                entry(1, LevelStatus::NotFound),
            ],
            "a".to_owned(),
        );
        ledger.merge(
            vec![
                entry(1, LevelStatus::NativeExit),
                entry(2, LevelStatus::Confirmed),
            ],
            "b".to_owned(),
        );
        assert_eq!(ledger.levels.len(), 3);
        assert_eq!((ledger.confirmed, ledger.native_exits), (2, 3));
        assert_eq!(ledger.playable, 103);
        let yaml = serde_yaml::to_string(&ledger).unwrap();
        let parsed: Ledger = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.levels[1].status, LevelStatus::NativeExit);
    }

    #[test]
    fn ledger_writer_revalidates_confirmed_witnesses_before_replacing_results() {
        let Some(manifest) = crate::tests::cached_generated_manifest() else {
            return;
        };
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let oracle = OracleRunner::new(
            repo,
            &repo.join("roms/kid.bin"),
            &scratch.path().join("oracle"),
        )
        .unwrap();
        // Saved route proves the native/ROM exit and transition at frame340.
        let existing = LevelEntry {
            level: 5,
            status: LevelStatus::Confirmed,
            exit: Some(ExitKind::ToLevel(74)),
            exit_frame: Some(340),
            transition_frame: Some(340),
            blocked_by: None,
            run: "previous".into(),
        };
        let route = include_bytes!("../../../tests/data/ledger-routes/05.json");
        let seed = |name: &str, entry: &LevelEntry, with_route: bool| {
            let root = scratch.path().join(name);
            fs::create_dir_all(root.join("docs/source-port")).unwrap();
            let mut ledger = Ledger::default();
            ledger.merge(vec![entry.clone()], "previous".into());
            write_ledger(&root, &ledger).unwrap();
            if with_route {
                fs::create_dir_all(root.join(ROUTE_DIR)).unwrap();
                fs::write(root.join(ROUTE_DIR).join("05.json"), route).unwrap();
            }
            root
        };
        let incoming = LevelEntry {
            status: LevelStatus::NativeExit,
            exit_frame: Some(278),
            transition_frame: Some(278),
            run: "new".into(),
            ..existing.clone()
        };
        let run = scratch.path().join("run");
        fs::create_dir_all(run.join("routes")).unwrap();

        let retained = seed("retained", &existing, true);
        write_run_results(&retained, &manifest, &oracle, &run, vec![incoming.clone()]).unwrap();
        let ledger = load_ledger(&retained).unwrap();
        assert_eq!(
            ledger.confirmed, 1,
            "blocked new route must not erase a verified exit"
        );
        assert_eq!(ledger.levels[0].exit_frame, Some(340));
        assert_eq!(ledger.levels[0].run, "previous");
        assert_eq!(
            fs::read(retained.join(ROUTE_DIR).join("05.json")).unwrap(),
            route
        );

        let stale = LevelEntry {
            exit_frame: Some(339),
            ..existing.clone()
        };
        for (name, entry, with_route) in [("stale", &stale, true), ("missing", &existing, false)] {
            let root = seed(name, entry, with_route);
            let before = fs::read(root.join(LEDGER_PATH)).unwrap();
            assert!(
                write_run_results(&root, &manifest, &oracle, &run, vec![incoming.clone()]).is_err()
            );
            assert_eq!(
                fs::read(root.join(LEDGER_PATH)).unwrap(),
                before,
                "failed validation must block writing"
            );
        }

        let replaced = seed("replaced", &stale, true);
        let confirmed = LevelEntry {
            run: "new".into(),
            ..existing
        };
        let mut new_route: DynamicScenario = serde_json::from_slice(route).unwrap();
        new_route.slug = "new-confirmed-route".into();
        let new_bytes = serde_json::to_vec(&new_route).unwrap();
        fs::write(run.join("routes/05.json"), &new_bytes).unwrap();
        write_run_results(&replaced, &manifest, &oracle, &run, vec![confirmed]).unwrap();
        let ledger = load_ledger(&replaced).unwrap();
        assert_eq!(ledger.confirmed, 1);
        assert_eq!(ledger.levels[0].run, "new");
        assert_eq!(ledger.levels[0].exit_frame, Some(340));
        assert_eq!(
            fs::read(replaced.join(ROUTE_DIR).join("05.json")).unwrap(),
            new_bytes
        );
    }

    #[test]
    fn exploration_is_deterministic_for_a_fixed_frame_budget() {
        let Some(manifest) = crate::tests::cached_generated_manifest() else {
            return;
        };
        let (runtime, frame) = settled_start(&manifest, 0).unwrap();
        let run = || explore(&runtime, &frame, 7, 3000, 64, || false);
        let (first, second) = (run(), run());
        assert_eq!(first.frames, second.frames);
        assert_eq!(first.cells, second.cells);
        assert_eq!(first.found, second.found);
        assert!(first.cells > 1, "explorer left the start cell");
    }
}
