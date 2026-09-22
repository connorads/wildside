//! Private case adapter. Ordinary inputs and recording replay remain authoritative.

use super::*;
use anyhow::ensure;
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseInput {
    issue: u16,
    output: PathBuf,
    start_level: Option<u8>,
    goal: Option<String>,
    #[serde(default)]
    progress: ProgressPolicy,
    #[serde(default)]
    actions: Vec<String>,
    target: Option<DiscoveryTarget>,
    #[serde(default = "default_tail_frames")]
    tail_frames: usize,
    #[serde(default)]
    stop_on_death: bool,
    script: Vec<u8>,
    checkpoints: Vec<usize>,
    #[serde(default)]
    live: bool,
    #[serde(default = "default_max_decisions")]
    max_decisions: usize,
}

fn default_max_decisions() -> usize {
    600
}

fn default_tail_frames() -> usize {
    30
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProgressPolicy {
    #[default]
    Cells,
    Rightward,
}

struct ProgressGuard {
    policy: ProgressPolicy,
    cells: BTreeSet<(u8, i32, i32)>,
    best_x: i32,
    level: u8,
    advanced: bool,
    stagnant_batches: usize,
}

impl ProgressGuard {
    fn new(policy: ProgressPolicy, level: u8, x: i32, y: i32) -> Self {
        Self {
            policy,
            cells: BTreeSet::from([(level, x.div_euclid(16), y.div_euclid(16))]),
            best_x: x,
            level,
            advanced: false,
            stagnant_batches: 0,
        }
    }

    fn observe(&mut self, level: u8, x: i32, y: i32) {
        let advanced = match self.policy {
            ProgressPolicy::Cells => self
                .cells
                .insert((level, x.div_euclid(16), y.div_euclid(16))),
            ProgressPolicy::Rightward => {
                let advanced = level != self.level || x > self.best_x;
                if advanced {
                    self.best_x = x;
                }
                advanced
            }
        };
        self.level = level;
        self.advanced |= advanced;
    }

    fn end_batch(&mut self, suspended: bool) -> bool {
        if self.advanced {
            self.stagnant_batches = 0;
        } else if !suspended {
            self.stagnant_batches += 1;
        }
        self.advanced = false;
        self.stagnant_batches >= 24
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum DiscoveryTarget {
    Position {
        level: u8,
        x: i32,
        y: i32,
        radius: u16,
        dwell_frames: usize,
    },
    Blw2Teleport,
    DragonBreathDeath,
    ObjectPressureCandidate,
}

impl DiscoveryTarget {
    fn matches(&self, trace: &NativeTraceFrame) -> bool {
        match self {
            Self::Position {
                level,
                x,
                y,
                radius,
                ..
            } => {
                let (px, py) = kid_pixels(trace);
                trace.current_level.0 == *level
                    && px.abs_diff(*x) <= u32::from(*radius)
                    && py.abs_diff(*y) <= u32::from(*radius)
            }
            // Discovery candidates are not proof assertions. The case evaluator
            // separately establishes the complete route and ASM/oracle evidence.
            Self::Blw2Teleport => {
                trace.current_level.0 == 1
                    && trace.teleport_warp_colour == 238
                    && kid_pixels(trace).0.abs_diff(448) < 24
            }
            Self::DragonBreathDeath => {
                trace.number_hitpoints == 0
                    && trace
                        .gameplay_debug_fields
                        .get("kid_gameplay_path_probe")
                        .is_some_and(|s| s.contains("post=projectile_damage/"))
                    && trace
                        .active_projectile_details
                        .iter()
                        .any(|p| p.kind == wildside_core::ActiveProjectileKind::DragonWalkingBreath)
            }
            Self::ObjectPressureCandidate => trace.number_objects >= 21,
        }
    }

    fn dwell_frames(&self) -> usize {
        match self {
            Self::Position { dwell_frames, .. } => *dwell_frames,
            _ => 1,
        }
    }
}

fn kid_pixels(trace: &NativeTraceFrame) -> (i32, i32) {
    (
        trace.kid_x_pos.unwrap_or_default() as i32 >> 16,
        trace.kid_y_pos.unwrap_or_default() as i32 >> 16,
    )
}

#[derive(Default)]
struct TargetProgress {
    dwell: usize,
    witness: Option<usize>,
}

impl TargetProgress {
    fn observe(
        &mut self,
        target: Option<&DiscoveryTarget>,
        trace: &NativeTraceFrame,
        index: usize,
    ) {
        if self.witness.is_some() {
            return;
        }
        if let Some(target) = target {
            if target.matches(trace) {
                self.dwell += 1;
                if self.dwell >= target.dwell_frames() {
                    self.witness = Some(index);
                }
            } else {
                self.dwell = 0;
            }
        }
    }

    fn complete(&self, index: usize, tail_frames: usize) -> bool {
        self.witness
            .is_some_and(|frame| index >= frame + tail_frames)
    }
}

fn discovery_life_lost(initial_lives: u16, lives: u16, hitpoints: u16) -> bool {
    hitpoints == 0 || lives < initial_lives
}

fn request_decision(state: &serde_json::Value) -> Result<u8> {
    let mut child = Command::new("python3")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/jev_case_service.py"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let write_result = (|| -> Result<()> {
        let mut input = child.stdin.take().context("helper stdin")?;
        serde_json::to_writer(&mut input, state)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = child.kill();
    }
    let response = child.wait_with_output()?;
    write_result?;
    ensure!(
        response.status.success(),
        "request failed or collection limit reached"
    );
    Ok(serde_json::from_slice(&response.stdout)?)
}

fn collect_with_controller(
    config: &CaseInput,
    manifest: &AssetManifest,
    start: RecordingStart,
    mut decide: impl FnMut(&serde_json::Value) -> Result<u8>,
) -> Result<Vec<u8>> {
    let mut recording =
        build_recording(manifest, "case-live", start, config.script.clone(), vec![])?;
    let mut runtime =
        initialise_recording_runtime(manifest, start, &recording.manifest.scenario)?.runtime;
    let mut observed = Vec::new();
    // ASM Mode_Level / ReadJoypad: boot and settled starts share the recorder's feed.
    for index in 0..config.script.len() {
        let applied =
            recording_script_buttons_at_frame(&start, &recording.manifest.scenario, index);
        observed.push(runtime.step_frame(FrameInput::from_mask(applied)));
    }
    let suffix_limit = config.script.len() + config.max_decisions * 6;
    let initial = runtime.trace_frame();
    let (initial_x, initial_y) = kid_pixels(&initial);
    let initial_lives = initial.number_lives;
    let mut was_alive = initial.number_hitpoints > 0;
    let mut progress = ProgressGuard::new(
        config.progress,
        initial.current_level.0,
        initial_x,
        initial_y,
    );
    let mut target = TargetProgress::default();
    let mut stop_reason = "frame_limit";
    let mut decisions = 0;
    let parent = config.output.parent().context("case output parent")?;
    let mut journal = fs::File::create(parent.join("inputs.bin"))?;
    journal.write_all(&recording.manifest.scenario.script)?;
    'collection: while recording.manifest.scenario.script.len() < suffix_limit {
        let trace = runtime.trace_frame();
        let x = trace.kid_x_pos.unwrap_or_default() as i32 >> 16;
        let y = trace.kid_y_pos.unwrap_or_default() as i32 >> 16;
        let mut terrain = vec![];
        if let Some(grid) = runtime.block_grid() {
            for row in (y / 16 - 8).max(0)..=(y / 16 + 3).max(0) {
                for col in (x / 16 - 6).max(0)..=(x / 16 + 12).max(0) {
                    let cell = grid.cell_at(col as u16, row as u16);
                    if cell.is_solid_floor() || cell.has_block {
                        terrain.push(json!({"x": col * 16, "y": row * 16,
                            "collision": format!("{:?}", cell.collision), "block": format!("{:?}", cell.block_type)}));
                    }
                }
            }
        }
        let state = json!({"goal": config.goal.as_deref().context("live case needs a goal")?,
            "allow_wait": target.dwell > 0,
            "actions": config.actions,
            "progress": if config.progress == ProgressPolicy::Rightward {"rightward"} else {"cells"},
            "stalled_forward_decisions": if config.progress == ProgressPolicy::Rightward {progress.stagnant_batches} else {0},
            "frame": recording.manifest.scenario.script.len(), "mode": trace.game_mode, "paused": trace.paused, "level": trace.current_level,
            "number_objects": trace.number_objects, "number_gfx_objects": trace.number_gfx_objects,
            "kid": {"x": x, "y": y, "vx": trace.kid_x_velocity.map(|v| f64::from(v)/65536.0),
                "vy": trace.kid_y_velocity.map(|v| f64::from(v)/65536.0), "grounded": trace.kid_on_ground,
                "hp": trace.number_hitpoints, "lives": trace.number_lives, "helmet_id": trace.current_helmet_id},
            "held_buttons": FrameInput::from_mask(recording.manifest.scenario.script.last().copied().unwrap_or_default()), "terrain_cells_16px": terrain,
            "enemies": trace.active_enemy_details, "projectiles": trace.active_projectile_details});
        let mask = if target.witness.is_some() {
            0
        } else {
            decisions += 1;
            let Ok(mask) = decide(&state) else {
                stop_reason = "request_failed_or_limit";
                break;
            };
            mask
        };
        let mut dwelled = false;
        for _ in 0..6 {
            let mask = if target.witness.is_some() { 0 } else { mask };
            recording.manifest.scenario.script.push(mask);
            let index = recording.manifest.scenario.script.len() - 1;
            let applied =
                recording_script_buttons_at_frame(&start, &recording.manifest.scenario, index);
            let frame = runtime.step_frame(FrameInput::from_mask(applied));
            let (x, y) = kid_pixels(&frame);
            progress.observe(frame.current_level.0, x, y);
            target.observe(config.target.as_ref(), &frame, index);
            dwelled |= target.dwell > 0;
            let dead = was_alive
                && discovery_life_lost(initial_lives, frame.number_lives, frame.number_hitpoints);
            was_alive |= frame.number_hitpoints > 0;
            observed.push(frame);
            journal.write_all(&[mask])?;
            if config.stop_on_death && dead {
                stop_reason = "life_lost";
                break 'collection;
            }
            if target.complete(index, config.tail_frames) {
                stop_reason = "target_reached";
                break 'collection;
            }
        }
        journal.flush()?;
        println!(
            "Jev case {} frame {}",
            config.issue,
            recording.manifest.scenario.script.len()
        );
        if progress.end_batch(dwelled || target.witness.is_some()) {
            stop_reason = "stalled";
            break;
        }
    }
    journal.flush()?;
    let length = recording.manifest.scenario.script.len();
    fs::write(
        parent.join("collection.json"),
        serde_json::to_vec_pretty(&json!({
            "stop_reason": stop_reason, "prefix_frames": config.script.len(),
            "suffix_frames": length - config.script.len(), "decisions": decisions,
            "target_witness_frame": target.witness,
            "target_tail_complete": length.checked_sub(1).is_some_and(|i| target.complete(i, config.tail_frames)),
            "tail_controls": "release", "claim": "discovery candidate only"
        }))?,
    )?;
    if start.settled {
        rebase_trace_frame_indices(&mut observed);
        if start.warp_level.is_some() {
            for frame in &mut observed {
                frame.level_skip_cheat_enabled = 1;
            }
        }
    }
    let replay = run_recorded_trace(manifest, start, &recording.manifest.scenario)?;
    ensure!(
        serde_json::to_value(replay)? == serde_json::to_value(observed)?,
        "live input journal does not exactly reconstruct collected state"
    );
    Ok(recording.manifest.scenario.script)
}

fn case_start(level: Option<u8>) -> RecordingStart {
    RecordingStart {
        warp_level: level,
        settled: level.is_some(),
        record_frames: None,
    }
}

#[test]
#[ignore = "private jev-case runner supplies inputs; requires ROM and generated assets"]
fn case_bundle() -> Result<()> {
    let config: CaseInput = serde_json::from_slice(&fs::read(std::env::var("JEV_CASE_INPUT")?)?)?;
    ensure!(!config.output.exists(), "case output must be fresh");
    ensure!(
        config.live || !config.script.is_empty(),
        "invalid frame count"
    );
    ensure!(!config.checkpoints.is_empty(), "case needs checkpoints");
    ensure!(
        config.live
            || config
                .checkpoints
                .iter()
                .all(|frame| *frame < config.script.len()),
        "checkpoint outside route"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repo")?;
    let rom = crate::resolve_rom_path(root, None)?;
    let verified = wildside_rom::verify_rom(&rom)?;
    if config.live {
        ensure!(
            std::env::var("JEV_EXPECTED_ROM_SHA1")? == verified.sha1,
            "ROM changed after preflight"
        );
    }
    let bundle = root.join(".cache/generated").join(verified.sha1);
    let manifest = wildside_data::load_manifest(&wildside_data::manifest_path(&bundle))?;
    let start = case_start(config.start_level);
    ensure!(
        !config.live
            || (config
                .goal
                .as_ref()
                .is_some_and(|goal| !goal.trim().is_empty())),
        "live discovery needs a goal"
    );
    ensure!(
        (1..=600).contains(&config.max_decisions),
        "decision limit must be 1..600"
    );
    let script = if config.live {
        collect_with_controller(&config, &manifest, start, request_decision)?
    } else {
        config.script
    };
    ensure!(!script.is_empty(), "no controller frames collected");
    let slug = format!("jev-case-{}", config.issue);
    let mut recording = build_recording(&manifest, &slug, start, script, vec![])?;
    recording.manifest.scenario.checkpoints = config
        .checkpoints
        .into_iter()
        .map(|frame_index| DynamicCheckpoint {
            label: format!("case-{frame_index}"),
            frame_index,
        })
        .collect();
    let trace = run_recorded_trace(&manifest, start, &recording.manifest.scenario)?;
    let checkpoints = recording.manifest.scenario.checkpoints;
    recording = build_recording(
        &manifest,
        &slug,
        start,
        recording.manifest.scenario.script,
        trace,
    )?;
    recording.manifest.scenario.checkpoints = checkpoints;
    save_recording_with_artifacts(
        &config.output,
        &recording,
        &RecordingBundleArtifacts::default(),
    )?;
    Ok(())
}

#[test]
fn gameplay_regression_jev_case_starts_do_not_seed_gameplay_state() {
    assert_eq!(case_start(None).warp_level, None);
    assert!(!case_start(None).settled);
    assert_eq!(case_start(Some(1)).warp_level, Some(1));
    assert_eq!(case_start(Some(0)).warp_level, Some(0));
}

#[test]
fn discovery_progress_covers_climbs_backtracking_and_stalls() {
    let mut guard = ProgressGuard::new(ProgressPolicy::Cells, 0, 32, 64);
    for (level, x, y) in [(0, 16, 64), (0, 16, 48), (0, 32, 48), (1, 32, 48)] {
        guard.observe(level, x, y);
        assert!(!guard.end_batch(false));
        assert_eq!(guard.stagnant_batches, 0);
    }
    for _ in 0..90 {
        assert!(!guard.end_batch(true));
    }
    for _ in 0..23 {
        guard.observe(1, 32, 48);
        assert!(!guard.end_batch(false));
    }
    assert!(guard.end_batch(false));
    let mut right = ProgressGuard::new(ProgressPolicy::Rightward, 0, 32, 64);
    right.observe(0, 33, 64);
    assert!(!right.end_batch(false));
    right.observe(0, 16, 48);
    assert!(!right.end_batch(false));
    assert_eq!(right.stagnant_batches, 1);
}

#[test]
fn paid_discovery_stops_at_fatal_hit_or_life_spend() {
    assert!(!discovery_life_lost(3, 3, 2));
    assert!(discovery_life_lost(3, 3, 0));
    assert!(discovery_life_lost(3, 2, 2));
}

fn case_test_manifest() -> Result<AssetManifest> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let rom = crate::resolve_rom_path(root, None)?;
    let verified = wildside_rom::verify_rom(&rom)?;
    let manifest = wildside_data::load_manifest(&wildside_data::manifest_path(
        &root.join(".cache/generated").join(verified.sha1),
    ))?;
    Ok(manifest)
}

#[test]
fn gameplay_regression_jev_prefix_handoff_preserves_mode_level_readjoypad() -> Result<()> {
    let manifest = case_test_manifest()?;
    // ASM Mode_Level / ReadJoypad: preserve the pending input at each handoff.
    for (level, prefix) in [
        (Some(0), vec![16]),
        (Some(0), vec![128, 128]),
        (None, vec![128]),
        (
            None,
            (0..3800)
                .map(|i| if i == 2400 || i == 2700 { 128 } else { 0 })
                .collect(),
        ),
    ] {
        let output = tempfile::tempdir()?;
        let config: CaseInput = serde_json::from_value(json!({
            "issue": 344, "output": output.path().join("bundle"), "start_level": level,
            "goal": "Test input handoff", "script": prefix, "checkpoints": [0],
            "max_decisions": 2, "live": true
        }))?;
        let mut calls = 0;
        let script = collect_with_controller(&config, &manifest, case_start(level), |state| {
            assert_eq!(state["frame"], config.script.len() + calls * 6);
            if calls == 0 {
                assert_eq!(
                    state["held_buttons"],
                    serde_json::to_value(FrameInput::from_mask(*config.script.last().unwrap()))?
                );
            }
            calls += 1;
            Ok(if calls == 1 { 24 } else { 0 })
        })?;
        assert_eq!(calls, 2);
        assert_eq!(&script[..config.script.len()], config.script);
        assert_eq!(
            &script[config.script.len()..],
            &[24, 24, 24, 24, 24, 24, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(fs::read(output.path().join("inputs.bin"))?, script);
    }
    Ok(())
}

#[test]
fn gameplay_regression_jev_discovery_tail_and_death_mode_level() -> Result<()> {
    let manifest = case_test_manifest()?;
    let output = tempfile::tempdir()?;
    let config: CaseInput = serde_json::from_value(json!({
        "issue": 344, "output": output.path().join("bundle"), "start_level": 0,
        "goal": "Observe a bounded dwell", "script": [], "checkpoints": [0],
        "max_decisions": 30, "actions": ["release"], "live": true,
        "target": {"kind":"position", "level":0, "x":32, "y":191, "radius":65535, "dwell_frames":150}
    }))?;
    let mut calls = 0;
    let script = collect_with_controller(&config, &manifest, case_start(Some(0)), |state| {
        assert_eq!(state["actions"], json!(["release"]));
        assert!(state["kid"]["helmet_id"].is_number());
        calls += 1;
        Ok(0)
    })?;
    assert_eq!(script.len(), 180);
    assert_eq!(
        calls, 25,
        "tail uses ordinary release controls without paid calls"
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(output.path().join("collection.json"))?)?;
    assert_eq!(report["stop_reason"], "target_reached");
    assert_eq!(report["target_witness_frame"], 149);
    assert_eq!(report["target_tail_complete"], true);

    // ASM Mode_Level / ReadJoypad: the collector must stop at the same fatal
    // frame as the complete ordinary replay, including inside a controller batch.
    let baseline = build_recording(
        &manifest,
        "death-stop",
        case_start(Some(0)),
        vec![0; 400],
        vec![],
    )?;
    let trace = run_recorded_trace(&manifest, case_start(Some(0)), &baseline.manifest.scenario)?;
    let fatal = trace
        .windows(2)
        .position(|pair| pair[0].number_hitpoints > 0 && pair[1].number_hitpoints == 0)
        .context("ordinary Dragon route must encounter death")?
        + 1;
    let output = tempfile::tempdir()?;
    let config: CaseInput = serde_json::from_value(json!({
        "issue":370, "output":output.path().join("bundle"), "start_level":0,
        "goal":"Capture a fatal hit", "script":vec![0;fatal-1], "checkpoints":[0],
        "max_decisions":2, "stop_on_death":true, "live":true
    }))?;
    let script = collect_with_controller(&config, &manifest, case_start(Some(0)), |_| Ok(0))?;
    assert_eq!(script.len(), fatal + 1);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(output.path().join("collection.json"))?)?;
    assert_eq!(report["stop_reason"], "life_lost");
    assert_eq!(report["suffix_frames"], 2);
    assert_eq!(report["decisions"], 1);
    Ok(())
}

#[test]
fn gameplay_regression_jev_failed_request_retains_replayable_prefix() -> Result<()> {
    let manifest = case_test_manifest()?;
    let output = tempfile::tempdir()?;
    let config: CaseInput = serde_json::from_value(json!({
        "issue":344, "output":output.path().join("bundle"), "start_level":0,
        "goal":"Retain a failed discovery", "script":[16], "checkpoints":[0], "live":true
    }))?;
    let script = collect_with_controller(&config, &manifest, case_start(Some(0)), |_| {
        anyhow::bail!("offline failure")
    })?;
    assert_eq!(script, [16]);
    assert_eq!(fs::read(output.path().join("inputs.bin"))?, script);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(output.path().join("collection.json"))?)?;
    assert_eq!(report["stop_reason"], "request_failed_or_limit");
    assert_eq!(report["suffix_frames"], 0);
    Ok(())
}
