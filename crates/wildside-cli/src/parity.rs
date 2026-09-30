//! Native-vs-ROM parity for input-driven routes: oracle replay, the first
//! blocking divergence, and a level-agnostic signature to cluster it by.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use wildside_core::NativeTraceFrame;

use crate::oracle::{
    OracleEnvironment, OracleManifest, OracleRunOptions, OracleState, ensure_oracles,
    jgenesis_dynamic_command,
};
use crate::scenarios::{DynamicCheckpoint, DynamicScenario};
use crate::sweep::{
    FrontierKind, StateDiff, SweepStateComparisonOptions,
    compare_sweep_states_for_frontier_with_options, extract_checkpoint_states,
    mask_unreachable_checkpoints, oracle_checkpoint_is_unreliable,
};

/// Logical levels `0x00-0x68` minus `$38`/`$47`, which ASM `constants.asm`
/// labels unplayable. `$69` is the Crypt leftover (`doc/level_list.txt`).
/// Mirrors the `candidate` rows of `docs/source-port/level-inventory.md`.
pub(crate) const PLAYABLE_LEVELS: [u8; 103] = playable_levels();

const fn playable_levels() -> [u8; 103] {
    let mut levels = [0; 103];
    let mut id = 0u8;
    let mut next = 0;
    while id <= 0x68 {
        if id != 0x38 && id != 0x47 {
            levels[next] = id;
            next += 1;
        }
        id += 1;
    }
    levels
}

/// Parses hex level ids: `all`, `0x00-0x05`, `00,03,0a` or a mix.
pub(crate) fn parse_levels(spec: &str) -> Result<Vec<u8>> {
    if spec.trim() == "all" {
        return Ok(PLAYABLE_LEVELS.to_vec());
    }
    let mut levels = BTreeSet::new();
    for part in spec
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let (first, last) = part.split_once('-').unwrap_or((part, part));
        let (first, last) = (
            crate::inspect::parse_level_id(first)?,
            crate::inspect::parse_level_id(last)?,
        );
        ensure!(first <= last, "level range {part} runs backwards");
        for id in first..=last {
            ensure!(
                PLAYABLE_LEVELS.contains(&id),
                "level {id:#04x} is not a playable candidate"
            );
            levels.insert(id);
        }
    }
    ensure!(!levels.is_empty(), "no levels in {spec:?}");
    Ok(levels.into_iter().collect())
}

/// A settled-warp route: script byte `i` is consumed on scenario frame
/// `i + 1`, and every frame is a checkpoint.
pub(crate) fn route_scenario(slug: String, level: u8, script: Vec<u8>) -> DynamicScenario {
    let frame_count = script.len() + 1;
    DynamicScenario {
        slug,
        script,
        frame_count,
        checkpoints: (0..frame_count)
            .map(|frame_index| DynamicCheckpoint {
                label: format!("frame_{frame_index}"),
                frame_index,
            })
            .collect(),
        start_position: None,
        start_velocity: None,
        camera_position: None,
        playtest_form: None,
        initial_diamond_count: None,
        initial_lives: None,
        warp_to_level: Some(level),
        settle_oracle_warp: true,
    }
}

/// Runs scenarios on the jgenesis helper, one scratch directory per job.
pub(crate) struct OracleRunner {
    env: OracleEnvironment,
    rom: PathBuf,
    run_root: PathBuf,
}

impl OracleRunner {
    pub(crate) fn new(repo_root: &Path, rom: &Path, run_root: &Path) -> Result<Self> {
        fs::create_dir_all(run_root)
            .with_context(|| format!("failed to create {}", run_root.display()))?;
        Ok(Self {
            env: ensure_oracles(repo_root)?,
            rom: rom.to_path_buf(),
            run_root: run_root.to_path_buf(),
        })
    }

    /// Replays `scenario`, returning its manifest. The job directory holds
    /// ~50 MB of PNGs per 600 frames, so it is deleted once the manifest
    /// parses; a failed job keeps it for `stderr.log`.
    pub(crate) fn run(&self, scenario: &DynamicScenario, job: &str) -> Result<OracleManifest> {
        let directory = self.run_root.join(job);
        if directory.exists() {
            fs::remove_dir_all(&directory)?;
        }
        let frames = directory.join("frames");
        fs::create_dir_all(&frames)?;
        let scenario_path = directory.join("scenario.json");
        fs::write(&scenario_path, serde_json::to_vec(scenario)?)?;
        let manifest_path = directory.join("manifest.json");
        let options = OracleRunOptions {
            capture_rng_stream: true,
            ..OracleRunOptions::default()
        };
        // File-backed output cannot fill a pipe while the parent polls the deadline.
        let mut child =
            jgenesis_dynamic_command(&self.env, &self.rom, &scenario_path, &frames, &options)
                .stdin(Stdio::null())
                .stdout(Stdio::from(File::create(&manifest_path)?))
                .stderr(Stdio::from(File::create(directory.join("stderr.log"))?))
                .spawn()
                .context("start oracle helper")?;
        // ~450 fps per helper; the base covers boot and the settled warp.
        let budget = Duration::from_secs(60)
            + Duration::from_millis(20 * u64::try_from(scenario.frame_count)?);
        wait_for_child(&mut child, Instant::now() + budget)
            .with_context(|| format!("oracle job {}", directory.display()))?;
        let manifest: OracleManifest = serde_json::from_slice(&fs::read(&manifest_path)?)
            .with_context(|| format!("decode {}", manifest_path.display()))?;
        fs::remove_dir_all(&directory)?;
        Ok(manifest)
    }
}

fn wait_for_child(child: &mut Child, deadline: Instant) -> Result<()> {
    loop {
        if let Some(status) = child.try_wait()? {
            ensure!(status.success(), "oracle exited {status}; see stderr.log");
            return Ok(());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("oracle deadline reached");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DivergenceKind {
    /// A blocking field diff between native and oracle checkpoint state.
    State,
    /// Native RNG differs from the oracle's per-frame RNG stream.
    RngStream,
    /// Authoritative CRC mismatch, or one side's trace ends early.
    Checksum,
}

/// The first frame where native and the ROM disagree on a comparable frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Divergence {
    pub(crate) frame: usize,
    pub(crate) kind: DivergenceKind,
    /// Blocking diffs at `frame`; empty unless `kind` is `State`.
    pub(crate) diffs: Vec<StateDiff>,
}

/// Finds the first blocking divergence of a settled-warp route to `warp_level`.
///
/// Oracle checkpoints that never reached the requested gameplay context are
/// masked (`oracle_checkpoint_is_unreliable`), which also hides everything
/// after the route leaves the level. Native death gaps are not masked: an
/// input-driven native death is exactly the divergence being searched for.
pub(crate) fn first_divergence(
    native_trace: &[NativeTraceFrame],
    oracle: &OracleManifest,
    warp_level: u8,
) -> Option<Divergence> {
    let checkpoints = oracle
        .states
        .iter()
        .map(|state| DynamicCheckpoint {
            label: state.label.clone(),
            frame_index: state.frame_index,
        })
        .collect::<Vec<_>>();
    let native_states = extract_checkpoint_states(native_trace, &checkpoints);
    let mut diffs = Vec::new();
    let mut seen = BTreeSet::new();
    for kind in [FrontierKind::Gameplay, FrontierKind::Enemy] {
        for diff in compare_sweep_states_for_frontier_with_options(
            &native_states,
            &oracle.states,
            kind,
            SweepStateComparisonOptions {
                include_rng: true,
                ..SweepStateComparisonOptions::default()
            },
        ) {
            if seen.insert((diff.checkpoint.clone(), diff.field.clone())) {
                diffs.push(diff);
            }
        }
    }
    mask_unreachable_checkpoints(
        &mut diffs,
        &native_states,
        &oracle.states,
        Some(warp_level),
        false,
    );
    let frames = oracle
        .states
        .iter()
        .map(|state| (state.label.as_str(), state.frame_index))
        .collect::<BTreeMap<_, _>>();
    let unreliable = oracle
        .states
        .iter()
        .filter(|state| oracle_checkpoint_is_unreliable(state, Some(warp_level)))
        .map(|state| state.frame_index)
        .collect::<BTreeSet<_>>();

    let state_frame = diffs
        .iter()
        .filter(|diff| diff.category.is_blocking())
        .filter_map(|diff| frames.get(diff.checkpoint.as_str()).copied())
        .min();
    let rng_frame = native_states
        .iter()
        .filter(|state| !unreliable.contains(&state.frame_index))
        .find(|state| {
            oracle
                .frame_rng_states
                .get(state.frame_index)
                .and_then(Option::as_ref)
                .is_some_and(|stream| {
                    state.rng_offset != Some(stream.offset())
                        || state.rng_ring.as_ref() != Some(stream.ring())
                })
        })
        .map(|state| state.frame_index);
    let checksum_frame = first_unmasked_crc_mismatch(native_trace, &oracle.frame_crcs, &unreliable);

    let (frame, kind) = [
        (state_frame, DivergenceKind::State),
        (rng_frame, DivergenceKind::RngStream),
        (checksum_frame, DivergenceKind::Checksum),
    ]
    .into_iter()
    .filter_map(|(frame, kind)| Some((frame?, kind)))
    .min_by_key(|(frame, _)| *frame)?;
    let diffs = if kind == DivergenceKind::State {
        diffs
            .into_iter()
            .filter(|diff| {
                diff.category.is_blocking() && frames.get(diff.checkpoint.as_str()) == Some(&frame)
            })
            .collect()
    } else {
        Vec::new()
    };
    Some(Divergence { frame, kind, diffs })
}

/// First CRC mismatch on a comparable frame. A length difference is always
/// reported: it means one side stopped early.
fn first_unmasked_crc_mismatch(
    native_trace: &[NativeTraceFrame],
    oracle_crcs: &[u32],
    unreliable: &BTreeSet<usize>,
) -> Option<usize> {
    (0..native_trace.len().max(oracle_crcs.len())).find(|&frame| {
        match (native_trace.get(frame), oracle_crcs.get(frame)) {
            (Some(native), Some(oracle)) => {
                !unreliable.contains(&frame) && native.authoritative_crc32() != *oracle
            }
            _ => true,
        }
    })
}

/// Level-agnostic cluster key for a divergence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct Signature {
    /// Sorted blocking field names, or the non-state check that fired.
    pub(crate) fields: Vec<String>,
    pub(crate) native_movement: Option<u16>,
    pub(crate) oracle_movement: Option<u16>,
    pub(crate) native_on_ground: Option<bool>,
    /// Script masks consumed on frames `f-2` and `f-1`. A settled-warp route
    /// feeds script byte `i` on frame `i + 1`, and ASM Mode_Level runs player
    /// logic before ReadJoypad, so gameplay on frame `f` acts on the first.
    pub(crate) inputs: [Option<u8>; 2],
}

impl Signature {
    pub(crate) fn new(
        divergence: &Divergence,
        native_trace: &[NativeTraceFrame],
        oracle: &OracleManifest,
        script: &[u8],
    ) -> Self {
        let frame = divergence.frame;
        let mut fields = match divergence.kind {
            DivergenceKind::State => divergence
                .diffs
                .iter()
                .map(|diff| diff.field.clone())
                .collect(),
            DivergenceKind::RngStream => vec!["rng_stream".to_owned()],
            DivergenceKind::Checksum => vec!["authoritative_crc".to_owned()],
        };
        fields.sort();
        fields.dedup();
        let native = native_trace.get(frame);
        let consumed = |before: usize| {
            frame
                .checked_sub(before + 1)
                .and_then(|index| script.get(index).copied())
        };
        Self {
            fields,
            native_movement: native.and_then(|trace| trace.character_movement),
            oracle_movement: oracle_state_at(oracle, frame)
                .and_then(|state| state.character_movement),
            native_on_ground: native.and_then(|trace| trace.kid_on_ground),
            inputs: [consumed(2), consumed(1)],
        }
    }

    /// Stable short id, used for repro file names and backlog references.
    pub(crate) fn id(&self) -> String {
        let digest = Sha1::digest(self.to_string().as_bytes());
        let prefix: [u8; 4] = digest[..4].try_into().expect("sha1 is 20 bytes");
        format!("{:08x}", u32::from_be_bytes(prefix))
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let show = |value: Option<u16>| value.map_or_else(|| "-".to_owned(), |v| v.to_string());
        let mask =
            |value: Option<u8>| value.map_or_else(|| "--".to_owned(), |v| format!("{v:02x}"));
        write!(
            formatter,
            "{} movement={}/{} ground={} inputs={},{}",
            self.fields.join(","),
            show(self.native_movement),
            show(self.oracle_movement),
            self.native_on_ground
                .map_or_else(|| "-".to_owned(), |ground| ground.to_string()),
            mask(self.inputs[0]),
            mask(self.inputs[1]),
        )
    }
}

fn oracle_state_at(oracle: &OracleManifest, frame: usize) -> Option<&OracleState> {
    oracle
        .states
        .iter()
        .find(|state| state.frame_index == frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wildside_core::{GameMode, LevelId, RomRandomFrameState, RomRandomRing};

    const LEVEL: u8 = 3;

    fn native_trace(frames: usize) -> Vec<NativeTraceFrame> {
        (0..frames)
            .map(|frame_index| NativeTraceFrame {
                frame_index,
                game_mode: GameMode::InGame,
                game_mode_value: 0x000c,
                current_level: LevelId(LEVEL),
                player_1_level_id: LevelId(LEVEL),
                kid_x_pos: Some(0x0040_0000),
                kid_y_pos: Some(0x0080_0000),
                kid_x_velocity: Some(0),
                kid_y_velocity: Some(0),
                character_movement: Some(0),
                kid_on_ground: Some(true),
                number_hitpoints: 2,
                number_lives: 3,
                rng_offset: Some(0xffff),
                rng_ring: Some(RomRandomRing::from_bytes([7; RomRandomRing::LENGTH])),
                ..NativeTraceFrame::default()
            })
            .collect()
    }

    fn oracle_for(trace: &[NativeTraceFrame]) -> OracleManifest {
        let checkpoints = (0..trace.len())
            .map(|frame_index| DynamicCheckpoint {
                label: format!("frame_{frame_index}"),
                frame_index,
            })
            .collect::<Vec<_>>();
        let mut manifest: OracleManifest = serde_json::from_value(serde_json::json!({
            "sample_rate": 0, "audio_path": "", "frames": [], "states": [],
            "data_registers": [0, 0, 0, 0, 0, 0, 0, 0],
            "address_registers": [0, 0, 0, 0, 0, 0, 0], "status_register": 0,
        }))
        .expect("manifest fixture");
        manifest.states = extract_checkpoint_states(trace, &checkpoints);
        manifest.frame_crcs = trace
            .iter()
            .map(NativeTraceFrame::authoritative_crc32)
            .collect();
        manifest.frame_rng_states = trace
            .iter()
            .map(|frame| {
                RomRandomFrameState::from_initialised_parts(
                    frame.rng_offset?,
                    frame.rng_ring.clone()?,
                )
            })
            .collect();
        manifest
    }

    #[test]
    fn identical_routes_do_not_diverge() {
        let trace = native_trace(20);
        assert!(first_divergence(&trace, &oracle_for(&trace), LEVEL).is_none());
    }

    #[test]
    fn a_velocity_diff_is_a_state_divergence_at_its_frame() {
        let trace = native_trace(20);
        let mut oracle = oracle_for(&trace);
        oracle.states[9].kid_x_vel = Some(0x100);
        oracle.states[12].kid_y_pos = 0;
        let divergence = first_divergence(&trace, &oracle, LEVEL).expect("diverges");
        assert_eq!(
            (divergence.frame, divergence.kind),
            (9, DivergenceKind::State)
        );
        let fields = divergence
            .diffs
            .iter()
            .map(|diff| diff.field.as_str())
            .collect::<Vec<_>>();
        assert_eq!(fields, ["kid_x_vel"]);
    }

    #[test]
    fn a_crc_only_mismatch_and_a_truncated_oracle_are_checksum_divergences() {
        let trace = native_trace(20);
        let mut oracle = oracle_for(&trace);
        oracle.frame_crcs[5] ^= 1;
        let divergence = first_divergence(&trace, &oracle, LEVEL).expect("diverges");
        assert_eq!(
            (divergence.frame, divergence.kind),
            (5, DivergenceKind::Checksum)
        );

        let mut oracle = oracle_for(&trace);
        oracle.frame_crcs.truncate(15);
        let divergence = first_divergence(&trace, &oracle, LEVEL).expect("diverges");
        assert_eq!(
            (divergence.frame, divergence.kind),
            (15, DivergenceKind::Checksum)
        );
    }

    #[test]
    fn an_rng_stream_mismatch_diverges_even_when_states_agree() {
        let trace = native_trace(20);
        let mut oracle = oracle_for(&trace);
        oracle.frame_rng_states[7] = RomRandomFrameState::from_initialised_parts(
            0xfffe,
            RomRandomRing::from_bytes([7; RomRandomRing::LENGTH]),
        );
        let divergence = first_divergence(&trace, &oracle, LEVEL).expect("diverges");
        assert_eq!(
            (divergence.frame, divergence.kind),
            (7, DivergenceKind::RngStream)
        );
    }

    #[test]
    fn unreliable_oracle_frames_are_masked_but_native_deaths_are_not() {
        let trace = native_trace(20);
        let mut oracle = oracle_for(&trace);
        // The oracle left the level: everything from frame 10 is masked.
        for state in &mut oracle.states[10..] {
            state.current_level_id = 0x40;
            state.kid_x_pos = 0;
        }
        for crc in &mut oracle.frame_crcs[10..] {
            *crc = 0;
        }
        assert!(first_divergence(&trace, &oracle, LEVEL).is_none());

        let mut dying = native_trace(20);
        dying[4].number_hitpoints = 0;
        dying[4].kid_x_pos = Some(0);
        let divergence =
            first_divergence(&dying, &oracle_for(&trace), LEVEL).expect("death diverges");
        assert_eq!(divergence.frame, 4);
        assert!(
            divergence
                .diffs
                .iter()
                .any(|diff| diff.field == "kid_x_pos")
        );
    }

    #[test]
    fn signature_keys_on_fields_movement_ground_and_the_consumed_inputs() {
        let trace = native_trace(20);
        let mut oracle = oracle_for(&trace);
        oracle.states[9].kid_x_pos = 0;
        oracle.states[9].character_movement = Some(2);
        let divergence = first_divergence(&trace, &oracle, LEVEL).expect("diverges");
        let script = (0..19).collect::<Vec<u8>>();
        let signature = Signature::new(&divergence, &trace, &oracle, &script);
        assert_eq!(signature.fields, ["character_movement", "kid_x_pos"]);
        assert_eq!(
            (signature.native_movement, signature.oracle_movement),
            (Some(0), Some(2))
        );
        assert_eq!(signature.native_on_ground, Some(true));
        // Frame f consumes script[f - 1]: frames 7 and 8 took script[6] and [7].
        assert_eq!(signature.inputs, [Some(6), Some(7)]);
        assert_eq!(signature.id().len(), 8);
        assert_eq!(signature.id(), signature.clone().id());
        let mut other = signature.clone();
        other.inputs[1] = Some(0);
        assert_ne!(signature.id(), other.id());
    }

    #[test]
    fn level_specs_are_hex_and_limited_to_playable_candidates() {
        assert_eq!(parse_levels("0x00-0x03").unwrap(), [0, 1, 2, 3]);
        assert_eq!(parse_levels("0a, 03,0x03").unwrap(), [3, 10]);
        assert_eq!(parse_levels("all").unwrap().len(), 103);
        assert!(parse_levels("0x37-0x39").is_err());
        assert!(parse_levels("0x69").is_err());
        assert!(parse_levels("05-01").is_err());
    }

    #[test]
    fn playable_levels_match_the_inventory_candidates() {
        let inventory = include_str!("../../../docs/source-port/level-inventory.md");
        let candidates = inventory
            .lines()
            .filter(|line| line.ends_with("| candidate |"))
            .map(|line| {
                let id = line
                    .split('`')
                    .nth(1)
                    .and_then(|cell| cell.strip_prefix("0x"))
                    .expect("logical id cell");
                u8::from_str_radix(id, 16).expect("hex logical id")
            })
            .collect::<Vec<_>>();
        assert_eq!(candidates, PLAYABLE_LEVELS);
    }
}
