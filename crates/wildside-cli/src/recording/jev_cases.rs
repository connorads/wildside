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
    stall_after_batches: Option<usize>,
    #[serde(default)]
    allow_wait: bool,
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

/// Stop a paid route when its best forward position has stalled for its
/// configured number of six-frame decisions.
struct ForwardProgressGuard {
    best_x: i32,
    stagnant_batches: usize,
    limit: usize,
}

impl ForwardProgressGuard {
    fn new(x: i32, limit: usize) -> Self {
        Self {
            best_x: x,
            stagnant_batches: 0,
            limit,
        }
    }

    fn observe(&mut self, x: i32) -> bool {
        if x > self.best_x {
            self.best_x = x;
            self.stagnant_batches = 0;
        } else {
            self.stagnant_batches += 1;
        }
        self.stagnant_batches >= self.limit
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
    let initial_x = runtime.trace_frame().kid_x_pos.unwrap_or_default() as i32 >> 16;
    let initial_lives = runtime.trace_frame().number_lives;
    let mut progress = config
        .stall_after_batches
        .map(|limit| ForwardProgressGuard::new(initial_x, limit));
    let parent = config.output.parent().context("case output parent")?;
    let mut journal = fs::File::create(parent.join("inputs.bin"))?;
    journal.write_all(&recording.manifest.scenario.script)?;
    while recording.manifest.scenario.script.len() < suffix_limit {
        let trace = runtime.trace_frame();
        if config.stop_on_death
            && discovery_life_lost(initial_lives, trace.number_lives, trace.number_hitpoints)
        {
            fs::write(
                parent.join("collection-stop.txt"),
                "life lost before route goal; saved input journal remains replayable",
            )?;
            break;
        }
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
            "allow_wait": config.allow_wait,
            "stalled_forward_decisions": progress.as_ref().map_or(0, |guard| guard.stagnant_batches),
            "frame": recording.manifest.scenario.script.len(), "mode": trace.game_mode, "paused": trace.paused, "level": trace.current_level,
            "number_objects": trace.number_objects, "number_gfx_objects": trace.number_gfx_objects,
            "kid": {"x": x, "y": y, "vx": trace.kid_x_velocity.map(|v| f64::from(v)/65536.0),
                "vy": trace.kid_y_velocity.map(|v| f64::from(v)/65536.0), "grounded": trace.kid_on_ground,
                "hp": trace.number_hitpoints, "lives": trace.number_lives},
            "held_buttons": FrameInput::from_mask(recording.manifest.scenario.script.last().copied().unwrap_or_default()), "terrain_cells_16px": terrain,
            "enemies": trace.active_enemy_details, "projectiles": trace.active_projectile_details});
        let Ok(mask) = decide(&state) else {
            fs::write(
                parent.join("collection-stop.txt"),
                "request failed or collection limit reached; saved inputs retained",
            )?;
            break;
        };
        for _ in 0..6 {
            recording.manifest.scenario.script.push(mask);
            let index = recording.manifest.scenario.script.len() - 1;
            let applied =
                recording_script_buttons_at_frame(&start, &recording.manifest.scenario, index);
            observed.push(runtime.step_frame(FrameInput::from_mask(applied)));
            journal.write_all(&[mask])?;
        }
        journal.flush()?;
        println!(
            "Jev case {} frame {}",
            config.issue,
            recording.manifest.scenario.script.len()
        );
        let current_x = runtime.trace_frame().kid_x_pos.unwrap_or_default() as i32 >> 16;
        if progress
            .as_mut()
            .is_some_and(|guard| guard.observe(current_x))
        {
            fs::write(
                parent.join("collection-stop.txt"),
                format!(
                    "no new forward position for {} controller decisions; saved input journal remains replayable",
                    config.stall_after_batches.unwrap_or_default()
                ),
            )?;
            break;
        }
    }
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
        config.stall_after_batches.is_none_or(|limit| limit > 0),
        "stall limit must be positive"
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
fn stalled_jev_route_stops_after_saved_blw2_release_pattern() {
    let mut guard = ForwardProgressGuard::new(32, 24);
    assert!(!guard.observe(58));
    for _ in 1..24 {
        assert!(!guard.observe(58));
    }
    assert!(guard.observe(58));

    let mut advancing = ForwardProgressGuard::new(32, 24);
    for x in 33..=60 {
        assert!(!advancing.observe(x));
    }
    for _ in 0..23 {
        assert!(!advancing.observe(57));
    }
    assert!(!advancing.observe(61));
}

#[test]
fn paid_discovery_stops_at_fatal_hit_or_life_spend() {
    assert!(!discovery_life_lost(3, 3, 2));
    assert!(discovery_life_lost(3, 3, 0));
    assert!(discovery_life_lost(3, 2, 2));
}

#[test]
fn gameplay_regression_jev_prefix_handoff_preserves_mode_level_readjoypad() -> Result<()> {
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
