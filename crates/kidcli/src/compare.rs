use std::fmt::Write;

use super::*;
use crate::scenarios::DynamicScenario;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GameplayDriftReport {
    pub(crate) frame_index: usize,
    pub(crate) window: String,
    pub(crate) differing_fields: Vec<&'static str>,
    pub(crate) native: GameplayState,
    pub(crate) oracle: GameplayState,
}

pub(crate) fn compare_native_oracle(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: Scenario,
) -> Result<()> {
    let run_dir = repo_root
        .join(".cache/native-oracle-runs")
        .join(scenario.slug());
    let native_dir = run_dir.join("native");
    let oracle_dir = run_dir.join("oracle");
    if native_dir.exists() {
        fs::remove_dir_all(&native_dir)
            .with_context(|| format!("failed to remove {}", native_dir.display()))?;
    }
    if oracle_dir.exists() {
        fs::remove_dir_all(&oracle_dir)
            .with_context(|| format!("failed to remove {}", oracle_dir.display()))?;
    }
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    let script = scenario.script();
    let native_trace = run_native_script_for_scenario(manifest, &script, scenario);
    let native_states = capture_native_checkpoint_states(&native_trace, scenario)?;
    fs::write(
        native_dir.join("state.json"),
        serde_json::to_vec_pretty(&native_states)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            native_dir.join("state.json").display()
        )
    })?;

    let env = ensure_oracles(repo_root)?;
    let oracle = run_jgenesis_capture(&env, rom_path, scenario, &oracle_dir)?;
    fs::write(
        oracle_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&oracle)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            oracle_dir.join("manifest.json").display()
        )
    })?;

    compare_native_states_against_oracle(&run_dir, &native_states, &oracle.states)?;
    println!(
        "native-oracle checkpoint comparison passed for {}",
        scenario.slug()
    );
    Ok(())
}

pub(crate) fn compare_native_gameplay(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    compare_native_gameplay_with(
        repo_root,
        scenario,
        || capture_gameplay_scenario(&rom_bytes, scenario),
        || capture_native_gameplay_scenario(manifest, scenario),
        || capture_native_gameplay_frame_evidence(manifest, scenario),
    )?;
    println!("native gameplay comparison passed for {}", scenario.slug());
    Ok(())
}

pub(crate) fn compare_native_gameplay_window(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    frame_start: usize,
    frame_end: usize,
    checkpoint_interval: usize,
) -> Result<()> {
    let window = scenario.to_dynamic_window(frame_start, frame_end, checkpoint_interval)?;
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    compare_dynamic_gameplay_with(
        repo_root,
        &window.slug,
        scenario,
        || capture_dynamic_gameplay_scenario(&rom_bytes, &window, scenario.uses_level_skip_cheat()),
        || Ok(capture_native_dynamic_gameplay_scenario(manifest, &window)),
        || capture_native_dynamic_gameplay_frame_evidence(manifest, &window),
    )?;
    println!(
        "native gameplay window comparison passed for {} frames {}..={} step {}",
        scenario.slug(),
        frame_start,
        frame_end,
        checkpoint_interval
    );
    Ok(())
}

pub(crate) fn diagnose_native_gameplay_drift(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    from_frame: usize,
    limit: usize,
    context_window: usize,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    let (run_dir, native_capture, oracle_capture) = capture_native_gameplay_run(
        repo_root,
        scenario,
        || capture_gameplay_scenario(&rom_bytes, scenario),
        || capture_native_gameplay_scenario(manifest, scenario),
    )?;
    let reports = find_gameplay_live_drifts(&native_capture, &oracle_capture, scenario, from_frame);
    let reports = reports.into_iter().take(limit.max(1)).collect::<Vec<_>>();

    println!("scenario: {}", scenario.slug());
    println!("run dir: {}", run_dir.display());
    if reports.is_empty() {
        println!("no live drift from frame {from_frame}");
        return Ok(());
    }

    for report in reports {
        println!(
            "frame {} [{}] fields: {}",
            report.frame_index,
            report.window,
            report.differing_fields.join(", ")
        );
        for field in &report.differing_fields {
            println!(
                "  {field}: native {} oracle {}",
                gameplay_field_value(&report.native, field),
                gameplay_field_value(&report.oracle, field)
            );
        }
        if let Some(summary) = gameplay_state_debug_summary(&report.native) {
            println!("  native context: {summary}");
        }
        if let Some(summary) = gameplay_state_debug_summary(&report.oracle) {
            println!("  oracle context: {summary}");
        }
        if context_window > 0 {
            println!("  context window:");
            for line in gameplay_drift_context_lines(
                &native_capture,
                &oracle_capture,
                report.frame_index,
                context_window,
            ) {
                println!("    {line}");
            }
        }
    }
    Ok(())
}

pub(crate) fn capture_native_gameplay_run<FO, FN>(
    repo_root: &Path,
    scenario: GameplayScenario,
    oracle_capture_fn: FO,
    native_capture_fn: FN,
) -> Result<(PathBuf, GameplayCapture, GameplayCapture)>
where
    FO: FnOnce() -> Result<GameplayCapture>,
    FN: FnOnce() -> Result<GameplayCapture>,
{
    capture_dynamic_gameplay_run(
        repo_root,
        scenario.slug(),
        oracle_capture_fn,
        native_capture_fn,
    )
}

pub(crate) fn capture_dynamic_gameplay_run<FO, FN>(
    repo_root: &Path,
    run_slug: &str,
    oracle_capture_fn: FO,
    native_capture_fn: FN,
) -> Result<(PathBuf, GameplayCapture, GameplayCapture)>
where
    FO: FnOnce() -> Result<GameplayCapture>,
    FN: FnOnce() -> Result<GameplayCapture>,
{
    let run_dir = repo_root
        .join(".cache/native-gameplay-oracle-runs")
        .join(run_slug);
    let native_dir = run_dir.join("native");
    let oracle_dir = run_dir.join("oracle");
    let review_dir = run_dir.join("review");
    if native_dir.exists() {
        fs::remove_dir_all(&native_dir)
            .with_context(|| format!("failed to remove {}", native_dir.display()))?;
    }
    if oracle_dir.exists() {
        fs::remove_dir_all(&oracle_dir)
            .with_context(|| format!("failed to remove {}", oracle_dir.display()))?;
    }
    if review_dir.exists() {
        fs::remove_dir_all(&review_dir)
            .with_context(|| format!("failed to remove {}", review_dir.display()))?;
    }
    fs::create_dir_all(&native_dir)
        .with_context(|| format!("failed to create {}", native_dir.display()))?;
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;

    let oracle_capture = oracle_capture_fn()?;
    write_gameplay_capture(&oracle_dir, &oracle_capture)?;

    let native_capture = native_capture_fn()?;
    write_gameplay_capture(&native_dir, &native_capture)?;

    Ok((run_dir, native_capture, oracle_capture))
}

pub(crate) fn compare_native_gameplay_with<FO, FN, FE>(
    repo_root: &Path,
    scenario: GameplayScenario,
    oracle_capture_fn: FO,
    native_capture_fn: FN,
    frame_evidence_fn: FE,
) -> Result<()>
where
    FO: FnOnce() -> Result<GameplayCapture>,
    FN: FnOnce() -> Result<GameplayCapture>,
    FE: FnOnce() -> Result<Vec<GameplayFrameEvidence>>,
{
    compare_dynamic_gameplay_with(
        repo_root,
        scenario.slug(),
        scenario,
        oracle_capture_fn,
        native_capture_fn,
        frame_evidence_fn,
    )
}

pub(crate) fn compare_dynamic_gameplay_with<FO, FN, FE>(
    repo_root: &Path,
    run_slug: &str,
    scenario: GameplayScenario,
    oracle_capture_fn: FO,
    native_capture_fn: FN,
    frame_evidence_fn: FE,
) -> Result<()>
where
    FO: FnOnce() -> Result<GameplayCapture>,
    FN: FnOnce() -> Result<GameplayCapture>,
    FE: FnOnce() -> Result<Vec<GameplayFrameEvidence>>,
{
    let (run_dir, native_capture, oracle_capture) =
        capture_dynamic_gameplay_run(repo_root, run_slug, oracle_capture_fn, native_capture_fn)?;
    compare_gameplay_checkpoints_against_oracle(
        &run_dir,
        scenario,
        &native_capture.checkpoints,
        &oracle_capture.checkpoints,
    )
    .map_err(|err| {
        annotate_gameplay_checkpoint_failure(err, &native_capture, &oracle_capture, scenario)
    })?;
    let frame_evidence = frame_evidence_fn()?;
    write_gameplay_frame_evidence(&run_dir.join("review"), &frame_evidence)?;
    Ok(())
}

fn annotate_gameplay_checkpoint_failure(
    err: anyhow::Error,
    native_capture: &GameplayCapture,
    oracle_capture: &GameplayCapture,
    scenario: GameplayScenario,
) -> anyhow::Error {
    let from_frame = native_capture
        .checkpoints
        .first()
        .map_or(0, |state| state.frame_index);
    let Some(first_drift) =
        find_gameplay_live_drifts(native_capture, oracle_capture, scenario, from_frame)
            .into_iter()
            .next()
    else {
        return err;
    };

    let mut message = err.to_string();
    let mut context = format!(
        "; first live drift frame {} [{}] fields: {}",
        first_drift.frame_index,
        first_drift.window,
        first_drift.differing_fields.join(", ")
    );
    if let Some(summary) = gameplay_state_debug_summary(&first_drift.native) {
        let _ = write!(context, "; native context: {summary}");
    }
    if let Some(phase_shift) =
        gameplay_phase_shift_hint(native_capture, oracle_capture, first_drift.frame_index)
    {
        let _ = write!(context, "; {phase_shift}");
    }
    message.push_str(&context);
    anyhow!(message)
}

pub(crate) fn capture_native_gameplay_frame_evidence(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<Vec<GameplayFrameEvidence>> {
    let script = scenario.script();
    let trace = run_native_script_for_gameplay(manifest, &script, scenario);
    render_gameplay_frame_evidence(manifest, &trace, scenario.checkpoints())
}

pub(crate) fn capture_native_dynamic_gameplay_frame_evidence(
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
) -> Result<Vec<GameplayFrameEvidence>> {
    let trace = run_native_script_dynamic(manifest, &scenario.script, scenario.frame_count);
    render_dynamic_gameplay_frame_evidence(manifest, &trace, &scenario.checkpoints)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GameplayCheckpointTolerance {
    pub(crate) camera_x: u32,
    pub(crate) camera_y: u32,
    pub(crate) kid_x: u32,
    pub(crate) kid_y: u32,
    pub(crate) number_objects: u16,
    pub(crate) number_gfx_objects: u16,
}

pub(crate) fn compare_gameplay_checkpoints_against_oracle(
    run_dir: &Path,
    scenario: GameplayScenario,
    native_states: &[GameplayState],
    oracle_states: &[GameplayState],
) -> Result<()> {
    if native_states.len() != oracle_states.len() {
        bail!(
            "native captured {} gameplay checkpoints but oracle captured {}; artefacts in {}",
            native_states.len(),
            oracle_states.len(),
            run_dir.display()
        );
    }

    for (native, oracle) in native_states.iter().zip(oracle_states) {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            bail!(
                "gameplay checkpoint mismatch: native {}@{} vs oracle {}@{}",
                native.label,
                native.frame_index,
                oracle.label,
                oracle.frame_index
            );
        }
        if native.game_mode != oracle.game_mode {
            bail!(
                "gameplay state {}@{} diverged in game mode (native {:04X}, oracle {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.game_mode,
                oracle.game_mode,
                run_dir.display()
            );
        }
        if native.current_player != oracle.current_player {
            bail!(
                "gameplay state {}@{} diverged in current player (native {:02X}, oracle {:02X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_player,
                oracle.current_player,
                run_dir.display()
            );
        }
        if native.number_lives != oracle.number_lives {
            bail!(
                "gameplay state {}@{} diverged in lives (native {}, oracle {}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_lives,
                oracle.number_lives,
                run_dir.display()
            );
        }
        if native.number_hitpoints != oracle.number_hitpoints
            || native.current_helmet_id != oracle.current_helmet_id
        {
            bail!(
                "gameplay state {}@{} diverged in form state (native hp {} helmet {:04X}, oracle hp {} helmet {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_hitpoints,
                native.current_helmet_id,
                oracle.number_hitpoints,
                oracle.current_helmet_id,
                run_dir.display()
            );
        }
        if native.current_level_id != oracle.current_level_id
            || native.player_1_level_id != oracle.player_1_level_id
        {
            bail!(
                "gameplay state {}@{} diverged in level ids (native current {:04X} / p1 {:04X}, oracle current {:04X} / p1 {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_level_id,
                native.player_1_level_id,
                oracle.current_level_id,
                oracle.player_1_level_id,
                run_dir.display()
            );
        }

        let tolerance = gameplay_checkpoint_tolerance(scenario, &native.label);
        compare_u32_delta(
            run_dir,
            "camera X",
            native,
            native.camera_x_pos,
            oracle.camera_x_pos,
            tolerance.camera_x,
        )?;
        compare_u32_delta(
            run_dir,
            "camera Y",
            native,
            native.camera_y_pos,
            oracle.camera_y_pos,
            tolerance.camera_y,
        )?;
        compare_u32_delta(
            run_dir,
            "kid X",
            native,
            native.kid_x_pos,
            oracle.kid_x_pos,
            tolerance.kid_x,
        )?;
        compare_u32_delta(
            run_dir,
            "kid Y",
            native,
            native.kid_y_pos,
            oracle.kid_y_pos,
            tolerance.kid_y,
        )?;
        compare_u16_delta(
            run_dir,
            "object counts",
            native,
            native.number_objects,
            oracle.number_objects,
            tolerance.number_objects,
        )?;
        compare_u16_delta(
            run_dir,
            "gfx object counts",
            native,
            native.number_gfx_objects,
            oracle.number_gfx_objects,
            tolerance.number_gfx_objects,
        )?;
        if native.character_movement != oracle.character_movement {
            bail!(
                "gameplay state {}@{} diverged in character movement (native {:?}, oracle {:?}); artefacts in {}{}",
                native.label,
                native.frame_index,
                native.character_movement,
                oracle.character_movement,
                run_dir.display(),
                gameplay_debug_context(native)
            );
        }
    }

    Ok(())
}

pub(crate) fn gameplay_checkpoint_tolerance(
    scenario: GameplayScenario,
    label: &str,
) -> GameplayCheckpointTolerance {
    match (scenario, label) {
        (GameplayScenario::PauseResumeLevel1, frame_label) if frame_label.starts_with("frame_") => {
            GameplayCheckpointTolerance {
                camera_x: 0x0020_0000,
                camera_y: CAMERA_POSITION_TOLERANCE,
                kid_x: 0x0020_0000,
                kid_y: CAMERA_POSITION_TOLERANCE,
                number_objects: 6,
                number_gfx_objects: 6,
            }
        }
        (GameplayScenario::DeathRespawnLevel1, frame_label)
            if frame_label.starts_with("frame_") =>
        {
            GameplayCheckpointTolerance {
                camera_x: CAMERA_POSITION_TOLERANCE,
                camera_y: CAMERA_POSITION_TOLERANCE,
                kid_x: CAMERA_POSITION_TOLERANCE,
                kid_y: CAMERA_POSITION_TOLERANCE,
                number_objects: 8,
                number_gfx_objects: 8,
            }
        }
        (GameplayScenario::PauseResumeLevel1, "resumed_tail") => GameplayCheckpointTolerance {
            camera_x: 0x0020_0000,
            camera_y: 0x0060_0000,
            kid_x: 0x0020_0000,
            kid_y: 0x0060_0000,
            number_objects: 6,
            number_gfx_objects: 6,
        },
        (GameplayScenario::PauseResumeLevel1, _) => GameplayCheckpointTolerance {
            camera_x: 0x0020_0000,
            camera_y: CAMERA_POSITION_TOLERANCE,
            kid_x: 0x0020_0000,
            kid_y: CAMERA_POSITION_TOLERANCE,
            number_objects: 6,
            number_gfx_objects: 6,
        },
        (GameplayScenario::DeathRespawnLevel1, "pre_death") => GameplayCheckpointTolerance {
            camera_x: 0x0030_0000,
            camera_y: 0x0060_0000,
            kid_x: 0x0030_0000,
            kid_y: 0x0060_0000,
            number_objects: 8,
            number_gfx_objects: 8,
        },
        _ => GameplayCheckpointTolerance {
            camera_x: CAMERA_POSITION_TOLERANCE,
            camera_y: CAMERA_POSITION_TOLERANCE,
            kid_x: CAMERA_POSITION_TOLERANCE,
            kid_y: CAMERA_POSITION_TOLERANCE,
            number_objects: 1,
            number_gfx_objects: 1,
        },
    }
}

pub(crate) fn compare_u32_delta(
    run_dir: &Path,
    field: &str,
    native: &GameplayState,
    native_value: u32,
    oracle_value: u32,
    tolerance: u32,
) -> Result<()> {
    if native_value.abs_diff(oracle_value) > tolerance {
        let debug_context = gameplay_debug_context(native);
        bail!(
            "gameplay state {}@{} diverged in {} (native {:08X}, oracle {:08X}); artefacts in {}{}",
            native.label,
            native.frame_index,
            field,
            native_value,
            oracle_value,
            run_dir.display(),
            debug_context
        );
    }
    Ok(())
}

pub(crate) fn compare_u16_delta(
    run_dir: &Path,
    field: &str,
    native: &GameplayState,
    native_value: u16,
    oracle_value: u16,
    tolerance: u16,
) -> Result<()> {
    if native_value.abs_diff(oracle_value) > tolerance {
        let debug_context = gameplay_debug_context(native);
        bail!(
            "gameplay state {}@{} diverged in {} (native {}, oracle {}); artefacts in {}{}",
            native.label,
            native.frame_index,
            field,
            native_value,
            oracle_value,
            run_dir.display(),
            debug_context
        );
    }
    Ok(())
}

pub(crate) fn find_gameplay_live_drifts(
    native_capture: &GameplayCapture,
    oracle_capture: &GameplayCapture,
    scenario: GameplayScenario,
    from_frame: usize,
) -> Vec<GameplayDriftReport> {
    native_capture
        .states
        .iter()
        .zip(&oracle_capture.states)
        .filter(|(native, oracle)| {
            native.frame_index >= from_frame && native.frame_index == oracle.frame_index
        })
        .filter_map(|(native, oracle)| {
            let differing_fields = gameplay_differing_fields(native, oracle);
            if differing_fields.is_empty() {
                None
            } else {
                Some(GameplayDriftReport {
                    frame_index: native.frame_index,
                    window: gameplay_checkpoint_window(scenario, native.frame_index),
                    differing_fields,
                    native: native.clone(),
                    oracle: oracle.clone(),
                })
            }
        })
        .collect()
}

pub(crate) fn gameplay_differing_fields(
    native: &GameplayState,
    oracle: &GameplayState,
) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if native.game_mode != oracle.game_mode {
        fields.push("game_mode");
    }
    if native.current_player != oracle.current_player {
        fields.push("current_player");
    }
    if native.number_lives != oracle.number_lives {
        fields.push("number_lives");
    }
    if native.number_hitpoints != oracle.number_hitpoints {
        fields.push("number_hitpoints");
    }
    if native.current_level_id != oracle.current_level_id {
        fields.push("current_level_id");
    }
    if native.player_1_level_id != oracle.player_1_level_id {
        fields.push("player_1_level_id");
    }
    if native.current_helmet_id != oracle.current_helmet_id {
        fields.push("current_helmet_id");
    }
    if native.camera_x_pos.abs_diff(oracle.camera_x_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("camera_x_pos");
    }
    if native.camera_y_pos.abs_diff(oracle.camera_y_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("camera_y_pos");
    }
    if native.kid_x_pos.abs_diff(oracle.kid_x_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("kid_x_pos");
    }
    if native.kid_y_pos.abs_diff(oracle.kid_y_pos) > CAMERA_POSITION_TOLERANCE {
        fields.push("kid_y_pos");
    }
    if native.character_movement != oracle.character_movement {
        fields.push("character_movement");
    }
    fields
}

pub(crate) fn gameplay_checkpoint_window(scenario: GameplayScenario, frame_index: usize) -> String {
    let checkpoints = scenario.checkpoints();
    let previous = checkpoints
        .iter()
        .rev()
        .find(|checkpoint| checkpoint.frame_index <= frame_index);
    let next = checkpoints
        .iter()
        .find(|checkpoint| checkpoint.frame_index >= frame_index);
    match (previous, next) {
        (Some(previous), Some(next)) if previous.frame_index == next.frame_index => {
            format!("{}@{}", previous.label, previous.frame_index)
        }
        (Some(previous), Some(next)) => {
            format!(
                "{}@{} -> {}@{}",
                previous.label, previous.frame_index, next.label, next.frame_index
            )
        }
        (Some(previous), None) => format!("after {}@{}", previous.label, previous.frame_index),
        (None, Some(next)) => format!("before {}@{}", next.label, next.frame_index),
        (None, None) => String::from("no checkpoints"),
    }
}

pub(crate) fn gameplay_field_value(state: &GameplayState, field: &str) -> String {
    match field {
        "game_mode" => format!("{:04X}", state.game_mode),
        "current_player" => state.current_player.to_string(),
        "number_lives" => state.number_lives.to_string(),
        "number_hitpoints" => state.number_hitpoints.to_string(),
        "current_level_id" => format!("{:04X}", state.current_level_id),
        "player_1_level_id" => format!("{:04X}", state.player_1_level_id),
        "current_helmet_id" => format!("{:04X}", state.current_helmet_id),
        "camera_x_pos" => format!("{:08X}", state.camera_x_pos),
        "camera_y_pos" => format!("{:08X}", state.camera_y_pos),
        "kid_x_pos" => format!("{:08X}", state.kid_x_pos),
        "kid_y_pos" => format!("{:08X}", state.kid_y_pos),
        "kid_x_vel" => state
            .kid_x_vel
            .map_or_else(|| String::from("none"), |value| format!("{value:08X}")),
        "kid_y_vel" => state
            .kid_y_vel
            .map_or_else(|| String::from("none"), |value| format!("{value:08X}")),
        "character_movement" => state
            .character_movement
            .map_or_else(|| String::from("none"), |value| format!("{value:04X}")),
        "kid_collision_type" => state
            .kid_collision_type
            .map_or_else(|| String::from("none"), |value| format!("{value:04X}")),
        "invulnerable_frames" => state
            .invulnerable_frames
            .map_or_else(|| String::from("none"), |value| value.to_string()),
        "just_received_damage" => state
            .just_received_damage
            .map_or_else(|| String::from("none"), |value| value.to_string()),
        "number_objects" => state.number_objects.to_string(),
        "number_gfx_objects" => state.number_gfx_objects.to_string(),
        _ => String::from("<unknown>"),
    }
}

fn gameplay_drift_context_lines(
    native_capture: &GameplayCapture,
    oracle_capture: &GameplayCapture,
    frame_index: usize,
    context_window: usize,
) -> Vec<String> {
    let start = frame_index.saturating_sub(context_window);
    let end = frame_index.saturating_add(context_window);
    native_capture
        .states
        .iter()
        .zip(&oracle_capture.states)
        .filter(|(native, oracle)| {
            native.frame_index == oracle.frame_index
                && (start..=end).contains(&native.frame_index)
        })
        .map(|(native, oracle)| {
            let differing_fields = gameplay_differing_fields(native, oracle);
            let marker = if native.frame_index == frame_index {
                '*'
            } else if differing_fields.is_empty() {
                '='
            } else {
                '!'
            };
            format!(
                "{marker} frame {} native[x={} y={} xv={} yv={} move={}] oracle[x={} y={} xv={} yv={} move={}] fields: {}",
                native.frame_index,
                gameplay_field_value(native, "kid_x_pos"),
                gameplay_field_value(native, "kid_y_pos"),
                gameplay_field_value(native, "kid_x_vel"),
                gameplay_field_value(native, "kid_y_vel"),
                gameplay_field_value(native, "character_movement"),
                gameplay_field_value(oracle, "kid_x_pos"),
                gameplay_field_value(oracle, "kid_y_pos"),
                gameplay_field_value(oracle, "kid_x_vel"),
                gameplay_field_value(oracle, "kid_y_vel"),
                gameplay_field_value(oracle, "character_movement"),
                if differing_fields.is_empty() {
                    String::from("-")
                } else {
                    differing_fields.join(", ")
                }
            )
        })
        .collect()
}

fn gameplay_debug_context(state: &GameplayState) -> String {
    gameplay_state_debug_summary(state)
        .map(|summary| format!("\n  context: {summary}"))
        .unwrap_or_default()
}

fn gameplay_state_debug_summary(state: &GameplayState) -> Option<String> {
    let mut fields = Vec::new();
    if let Some(value) = state.kid_x_vel {
        fields.push(format!("kid_x_vel={value:#010x}"));
    }
    if let Some(value) = state.kid_y_vel {
        fields.push(format!("kid_y_vel={value:#010x}"));
    }
    if let Some(value) = state.character_movement {
        fields.push(format!("character_movement={value:#06x}"));
    }
    if let Some(value) = state.kid_collision_type {
        fields.push(format!("kid_collision_type={value:#06x}"));
    }
    if let Some(value) = state.invulnerable_frames {
        fields.push(format!("invulnerable_frames={value}"));
    }
    if let Some(value) = state.just_received_damage {
        fields.push(format!("just_received_damage={}", u8::from(value)));
    }
    if let Some(value) = &state.kid_jump_gravity_probe {
        fields.push(format!("kid_jump_gravity_probe={value}"));
    }
    if let Some(value) = &state.kid_ground_probe {
        fields.push(format!("kid_ground_probe={value}"));
    }
    if let Some(value) = &state.kid_grounded_release_probe {
        fields.push(format!("kid_grounded_release_probe={value}"));
    }
    (!fields.is_empty()).then(|| fields.join(", "))
}

fn gameplay_phase_shift_hint(
    native_capture: &GameplayCapture,
    oracle_capture: &GameplayCapture,
    frame_index: usize,
) -> Option<String> {
    const LOOKBACK_FRAMES: usize = 64;
    const MIN_MATCHES: usize = 3;
    const POSITION_HINT_TOLERANCE_FP: u32 = 0x0001_0000;
    const VELOCITY_HINT_TOLERANCE_FP: u32 = 0x0000_4000;

    let start = frame_index.saturating_sub(LOOKBACK_FRAMES);
    let mut matches = Vec::new();

    for native in native_capture
        .states
        .iter()
        .filter(|state| (start..=frame_index).contains(&state.frame_index))
    {
        let Some(same_oracle) = oracle_capture
            .states
            .iter()
            .find(|state| state.frame_index == native.frame_index)
        else {
            continue;
        };
        let Some(prev_oracle) = oracle_capture
            .states
            .iter()
            .find(|state| state.frame_index + 1 == native.frame_index)
        else {
            continue;
        };

        let same_distance = gameplay_phase_distance(native, same_oracle);
        let prev_distance = gameplay_phase_distance(native, prev_oracle);

        if prev_distance.score < same_distance.score
            && prev_distance.position_max <= POSITION_HINT_TOLERANCE_FP
            && prev_distance.velocity_max <= VELOCITY_HINT_TOLERANCE_FP
            && prev_distance.character_movement_matches
        {
            matches.push(native.frame_index);
        }
    }

    if matches.len() < MIN_MATCHES {
        return None;
    }

    let start_frame = *matches.first()?;
    let end_frame = *matches.last()?;
    Some(format!(
        "phase hint: native frames {start_frame}..{end_frame} more closely match oracle one frame earlier; likely phase-slip onset at frame {start_frame}"
    ))
}

#[derive(Debug, Clone, Copy, Default)]
struct GameplayPhaseDistance {
    score: u128,
    position_max: u32,
    velocity_max: u32,
    character_movement_matches: bool,
}

fn gameplay_phase_distance(
    native: &GameplayState,
    oracle: &GameplayState,
) -> GameplayPhaseDistance {
    let position_diffs = [
        native.kid_x_pos.abs_diff(oracle.kid_x_pos),
        native.kid_y_pos.abs_diff(oracle.kid_y_pos),
    ];
    let velocity_diffs = [
        optional_i32_abs_diff(native.kid_x_vel, oracle.kid_x_vel),
        optional_i32_abs_diff(native.kid_y_vel, oracle.kid_y_vel),
    ];

    let score = position_diffs
        .into_iter()
        .chain(velocity_diffs)
        .map(u128::from)
        .sum::<u128>();

    let position_max = position_diffs.into_iter().max().unwrap_or(0);
    let velocity_max = velocity_diffs.into_iter().max().unwrap_or(0);
    let character_movement_matches = native.character_movement == oracle.character_movement
        && native.character_movement.is_some();

    GameplayPhaseDistance {
        score,
        position_max,
        velocity_max,
        character_movement_matches,
    }
}

fn optional_i32_abs_diff(left: Option<i32>, right: Option<i32>) -> u32 {
    left.zip(right)
        .map_or(u32::MAX, |(left, right)| left.abs_diff(right))
}

pub(crate) fn run_port_loop(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: Scenario,
    iterations: u32,
) -> Result<()> {
    let total_iterations = iterations.max(1);
    for iteration in 1..=total_iterations {
        println!(
            "port loop iteration {iteration}/{total_iterations} for {}",
            scenario.slug()
        );
        verify_native_bootstrap(manifest)?;
        compare_native_oracle(repo_root, rom_path, manifest, scenario)?;
    }
    Ok(())
}

pub(crate) fn capture_native_checkpoint_states(
    trace: &[NativeTraceFrame],
    scenario: Scenario,
) -> Result<Vec<OracleState>> {
    scenario
        .checkpoints()
        .iter()
        .map(|checkpoint| {
            let frame = native_trace_at_checkpoint(trace, checkpoint.label, checkpoint.frame_index)
                .ok_or_else(|| {
                    anyhow!(
                        "missing native checkpoint {}@{}",
                        checkpoint.label,
                        checkpoint.frame_index
                    )
                })?;
            Ok(native_trace_to_oracle_state(
                checkpoint.label,
                checkpoint.frame_index,
                frame,
            ))
        })
        .collect()
}

pub(crate) fn native_trace_at_checkpoint<'a>(
    trace: &'a [NativeTraceFrame],
    checkpoint_label: &str,
    checkpoint_frame_index: usize,
) -> Option<&'a NativeTraceFrame> {
    if checkpoint_label == "title" {
        return checkpoint_frame_index
            .checked_sub(1)
            .and_then(|index| trace.get(index))
            .or_else(|| trace.get(checkpoint_frame_index));
    }
    trace.get(checkpoint_frame_index).or_else(|| {
        checkpoint_frame_index
            .checked_sub(1)
            .and_then(|index| trace.get(index))
    })
}

pub(crate) fn native_trace_to_oracle_state(
    label: &str,
    frame_index: usize,
    frame: &NativeTraceFrame,
) -> OracleState {
    let hidden_player = frame.kid_x_pos.is_none() || frame.kid_y_pos.is_none();
    let main_menu_option = match frame.game_mode {
        kid_core::GameMode::TitleScreen => frame.options_cursor,
        kid_core::GameMode::OptionsMenu => frame.options_cursor,
        _ => None,
    };

    OracleState {
        label: label.to_owned(),
        frame_index,
        game_mode: frame.game_mode_value,
        boot_phase: frame.boot_phase.clone(),
        boot_phase_elapsed: frame.boot_phase_elapsed,
        title_screen_ready: frame.title_screen_ready,
        main_menu_option,
        current_player: frame.current_player,
        number_hitpoints: frame.number_hitpoints,
        current_helmet_id: frame.current_helmet_id,
        current_level_id: u16::from(frame.current_level.0),
        player_1_level_id: u16::from(frame.player_1_level_id.0),
        camera_x_pos: frame.camera_x_pos,
        camera_y_pos: frame.camera_y_pos,
        kid_x_pos: frame.kid_x_pos.unwrap_or(0),
        kid_y_pos: frame.kid_y_pos.unwrap_or(0),
        number_objects: frame.number_objects,
        number_gfx_objects: frame.number_gfx_objects,
        active_enemies: active_enemy_states_from_native_frame(frame),
        game_paused: frame.paused,
        score: frame.score,
        number_lives: frame.number_lives,
        number_diamonds: frame.diamond_count,
        diamond_power_active: frame.diamond_power_active,
        murder_wall_x: frame.murder_wall_x.map(|x| x as i32),
        // ASM WRAM keeps the player position/motion mirrors allocated during
        // hidden boot/title shells. Native trace hides the player pose as
        // `None`, so normalise the raw motion mirrors to the same zeroes used
        // by the oracle instead of mixing zero position with absent movement.
        kid_x_vel: frame.kid_x_velocity.or(hidden_player.then_some(0)),
        kid_y_vel: frame.kid_y_velocity.or(hidden_player.then_some(0)),
        character_movement: frame.character_movement.or(hidden_player.then_some(0)),
        just_received_damage: Some(frame.just_received_damage),
        kid_collision_type: None,
        kid_collision_source: None,
        // ASM `x_direction = $16` (GfxObject struct): $00 = right (default
        // sprite orientation), $FF = left. Native carries facing on
        // `kid_facing`; emit it here only during gameplay so boot/title
        // sequences (where the byte is unstable) don't drive false diffs.
        kid_x_direction: matches!(
            frame.game_mode,
            kid_core::GameMode::InGame | kid_core::GameMode::DemoPlay
        )
        .then(|| match frame.kid_facing {
            Some(kid_core::physics::Direction::Right) => 0u8,
            Some(kid_core::physics::Direction::Left) => 0xFFu8,
            None => 0u8,
        }),
        invulnerable_frames: Some(frame.invulnerable_frames as u8),
        // ASM Kid_Transform loc_7E90 clears `Currently_transforming` during
        // the final cleanup frame, while the gameplay loop is still hibernated
        // until the coroutine returns. Native keeps that cleanup frame in
        // `form_change_flash_frames`, so only frames above 1 map to the ROM
        // RAM flag.
        currently_transforming: Some(frame.form_change_flash_frames > 1),
        rng_offset: frame.rng_offset,
        level_special_effects: None,
        gameplay_debug_fields: frame.gameplay_debug_fields.clone(),
        cpu_registers: None,
    }
}

pub(crate) fn active_enemy_states_from_native_frame(
    frame: &NativeTraceFrame,
) -> Vec<OracleEnemyState> {
    let mut enemies = frame
        .active_objects
        .iter()
        .filter_map(|object| match object.kind {
            kid_core::NativeObjectKind::Enemy { object_type } => Some(OracleEnemyState {
                object_type,
                x_pos: object.x_pos,
                y_pos: object.y_pos,
                x_velocity: object.x_velocity.unwrap_or(0),
                y_velocity: object.y_velocity.unwrap_or(0),
                facing: enemy_facing_from_native(object.facing),
                collision_type: None,
                collision_source: None,
                layout_index: object.layout_index,
                layout_ptr: None,
                parent_object_ptr: None,
                gfx_slot_index: None,
                on_ground: object.on_ground,
                // Native `hit_points` is the byte the runtime decrements; the
                // ROM stores it as the word `current_hp` at GfxObject + $44.
                // Widen u8 → u16 for the comparison so both sides represent
                // the same M68K word.
                current_hp: object.hit_points.map(u16::from),
                debug_fields: object.debug_fields.clone(),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    sort_enemy_states(&mut enemies);
    enemies
}

pub(crate) fn enemy_facing_from_native(facing: Option<NativeDirection>) -> EnemyFacing {
    match facing.unwrap_or(NativeDirection::Left) {
        NativeDirection::Left => EnemyFacing::Left,
        NativeDirection::Right => EnemyFacing::Right,
    }
}

pub(crate) fn sort_enemy_states(enemies: &mut [OracleEnemyState]) {
    enemies.sort_by_key(|enemy| {
        (
            enemy.object_type,
            enemy.layout_index.unwrap_or(usize::MAX),
            enemy.layout_ptr.unwrap_or(u32::MAX),
            enemy.x_pos,
            enemy.y_pos,
        )
    });
}

pub(crate) fn compare_native_states_against_oracle(
    run_dir: &Path,
    native_states: &[OracleState],
    oracle_states: &[OracleState],
) -> Result<()> {
    const ENEMY_POSITION_TOLERANCE: u32 = 0x0010_0000;
    const ENEMY_VELOCITY_TOLERANCE: i32 = 0x0001_0000;
    let compare_enemy_snapshots = oracle_states.iter().any(|state| {
        gameplay_object_roster_is_authoritative(state) && !state.active_enemies.is_empty()
    });
    let native_enemy_labels = native_states
        .iter()
        .filter(|state| !state.active_enemies.is_empty())
        .map(|state| format!("{}@{}", state.label, state.frame_index))
        .collect::<Vec<_>>();

    if native_states.len() != oracle_states.len() {
        bail!(
            "native captured {} states but oracle captured {}; artefacts in {}",
            native_states.len(),
            oracle_states.len(),
            run_dir.display()
        );
    }

    for (native, oracle) in native_states.iter().zip(oracle_states) {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            bail!(
                "state checkpoint mismatch: native {}@{} vs oracle {}@{}",
                native.label,
                native.frame_index,
                oracle.label,
                oracle.frame_index
            );
        }
        if native.game_mode != oracle.game_mode {
            bail!(
                "state {}@{} diverged in game mode (native {:04X}, oracle {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.game_mode,
                oracle.game_mode,
                run_dir.display()
            );
        }
        if native.current_player != oracle.current_player {
            bail!(
                "state {}@{} diverged in current player (native {:02X}, oracle {:02X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_player,
                oracle.current_player,
                run_dir.display()
            );
        }
        if native.number_hitpoints != oracle.number_hitpoints
            || native.current_helmet_id != oracle.current_helmet_id
        {
            bail!(
                "state {}@{} diverged in form state (native hp {} helmet {:04X}, oracle hp {} helmet {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_hitpoints,
                native.current_helmet_id,
                oracle.number_hitpoints,
                oracle.current_helmet_id,
                run_dir.display()
            );
        }
        if native.current_level_id != oracle.current_level_id
            || native.player_1_level_id != oracle.player_1_level_id
        {
            bail!(
                "state {}@{} diverged in level ids (native current {:04X} / p1 {:04X}, oracle current {:04X} / p1 {:04X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_level_id,
                native.player_1_level_id,
                oracle.current_level_id,
                oracle.player_1_level_id,
                run_dir.display()
            );
        }
        if native.label == "level_entry" {
            // ASM Obj_TitleMenu `loc_1BBD6` writes `Game_Mode=$0008` and
            // jumps through `loc_6E2` into `Load_InGame`, so the first
            // boot-title-start level-entry checkpoint is only authoritative
            // for the hand-off identity fields checked above. The title-card
            // worker pose can still be mid-transition on this frame before
            // both sides settle to the same Kid pose by `level_settle@719`.
            continue;
        }

        let camera_delta_x = native.camera_x_pos.abs_diff(oracle.camera_x_pos);
        let camera_delta_y = native.camera_y_pos.abs_diff(oracle.camera_y_pos);
        if camera_delta_x > CAMERA_POSITION_TOLERANCE || camera_delta_y > CAMERA_POSITION_TOLERANCE
        {
            bail!(
                "state {}@{} diverged in camera position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.camera_x_pos,
                native.camera_y_pos,
                oracle.camera_x_pos,
                oracle.camera_y_pos,
                run_dir.display()
            );
        }
        let kid_delta_x = native.kid_x_pos.abs_diff(oracle.kid_x_pos);
        let kid_delta_y = native.kid_y_pos.abs_diff(oracle.kid_y_pos);
        if kid_delta_x > CAMERA_POSITION_TOLERANCE || kid_delta_y > CAMERA_POSITION_TOLERANCE {
            bail!(
                "state {}@{} diverged in kid position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.kid_x_pos,
                native.kid_y_pos,
                oracle.kid_x_pos,
                oracle.kid_y_pos,
                run_dir.display()
            );
        }
        let level_complete_visual_checkpoint =
            is_level_complete_visual_worker_checkpoint(native.label.as_str());
        if !level_complete_visual_checkpoint
            && gameplay_object_roster_is_authoritative(oracle)
            && (native.number_objects.abs_diff(oracle.number_objects) > 1
                || native
                    .number_gfx_objects
                    .abs_diff(oracle.number_gfx_objects)
                    > 1)
        {
            bail!(
                "state {}@{} diverged in object counts (native {} / {}, oracle {} / {}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_objects,
                native.number_gfx_objects,
                oracle.number_objects,
                oracle.number_gfx_objects,
                run_dir.display()
            );
        }
        if native.character_movement != oracle.character_movement {
            bail!(
                "state {}@{} diverged in character movement (native {:?}, oracle {:?}); artefacts in {}",
                native.label,
                native.frame_index,
                native.character_movement,
                oracle.character_movement,
                run_dir.display()
            );
        }
        if native.just_received_damage != oracle.just_received_damage {
            bail!(
                "state {}@{} diverged in just_received_damage (native {:?}, oracle {:?}); artefacts in {}",
                native.label,
                native.frame_index,
                native.just_received_damage,
                oracle.just_received_damage,
                run_dir.display()
            );
        }
        if compare_enemy_snapshots
            && !level_complete_visual_checkpoint
            && gameplay_object_roster_is_authoritative(oracle)
        {
            compare_enemy_snapshots_against_oracle(
                run_dir,
                native.label.as_str(),
                native.frame_index,
                &native.active_enemies,
                &oracle.active_enemies,
                ENEMY_POSITION_TOLERANCE,
                ENEMY_VELOCITY_TOLERANCE,
            )?;
        }
    }

    if !compare_enemy_snapshots {
        if native_enemy_labels.is_empty() {
            eprintln!(
                "warning: oracle captured no enemy snapshots; skipped per-enemy comparison for {}",
                run_dir.display()
            );
        } else {
            eprintln!(
                "warning: oracle captured no Diamond/MiniHoppingSkull snapshots, but native captured enemy snapshots at {}; skipped per-enemy comparison for {}",
                native_enemy_labels.join(", "),
                run_dir.display()
            );
        }
    }

    Ok(())
}

fn is_level_complete_visual_worker_checkpoint(label: &str) -> bool {
    // ASM loc_C21E/sub_C254 and loc_D4CE/loc_D770 build the level-complete
    // score room out of transient object/gfx-object workers. Native records
    // those visuals through trace fields and renderer overlays, so the
    // recording verifier should keep checking gameplay identity fields without
    // requiring the core object roster to model every visual worker slot.
    matches!(
        label,
        "flag_touch" | "eol_reveal" | "eol_counter" | "eol_tail"
    )
}

fn gameplay_object_roster_is_authoritative(state: &OracleState) -> bool {
    let in_game = kid_core::GameMode::InGame.as_disassembly_value();
    let demo_play = kid_core::GameMode::DemoPlay.as_disassembly_value();
    matches!(state.game_mode, mode if mode == in_game || mode == demo_play)
}

pub(crate) fn compare_enemy_snapshots_against_oracle(
    run_dir: &Path,
    label: &str,
    frame_index: usize,
    native_enemies: &[OracleEnemyState],
    oracle_enemies: &[OracleEnemyState],
    position_tolerance: u32,
    velocity_tolerance: i32,
) -> Result<()> {
    if native_enemies.len() != oracle_enemies.len() {
        bail!(
            "state {}@{} diverged in enemy count (native {}, oracle {}); artefacts in {}",
            label,
            frame_index,
            native_enemies.len(),
            oracle_enemies.len(),
            run_dir.display()
        );
    }

    for (native, oracle) in native_enemies.iter().zip(oracle_enemies) {
        if native.object_type != oracle.object_type {
            bail!(
                "state {}@{} diverged in enemy type ordering (native {:02X}, oracle {:02X}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                oracle.object_type,
                run_dir.display()
            );
        }
        if native.x_pos.abs_diff(oracle.x_pos) > position_tolerance
            || native.y_pos.abs_diff(oracle.y_pos) > position_tolerance
        {
            bail!(
                "state {}@{} diverged in enemy {:02X} position (native {:08X},{:08X}; oracle {:08X},{:08X}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                native.x_pos,
                native.y_pos,
                oracle.x_pos,
                oracle.y_pos,
                run_dir.display()
            );
        }
        if (native.x_velocity - oracle.x_velocity).abs() > velocity_tolerance
            || (native.y_velocity - oracle.y_velocity).abs() > velocity_tolerance
        {
            bail!(
                "state {}@{} diverged in enemy {:02X} velocity (native {},{}; oracle {},{}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                native.x_velocity,
                native.y_velocity,
                oracle.x_velocity,
                oracle.y_velocity,
                run_dir.display()
            );
        }
        if native.facing != oracle.facing {
            bail!(
                "state {}@{} diverged in enemy {:02X} facing (native {:?}, oracle {:?}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                native.facing,
                oracle.facing,
                run_dir.display()
            );
        }
        if native.layout_index != oracle.layout_index {
            bail!(
                "state {}@{} diverged in enemy {:02X} layout index (native {:?}, oracle {:?}); artefacts in {}",
                label,
                frame_index,
                native.object_type,
                native.layout_index,
                oracle.layout_index,
                run_dir.display()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_state(label: &str, frame_index: usize, game_mode: u16) -> OracleState {
        OracleState {
            label: label.into(),
            frame_index,
            game_mode,
            boot_phase: Some(String::from("starting_level")),
            boot_phase_elapsed: Some(0),
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 2,
            current_helmet_id: 0,
            current_level_id: 0,
            player_1_level_id: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            active_enemies: Vec::new(),
            game_paused: false,
            score: 0,
            number_lives: 3,
            number_diamonds: 0,
            diamond_power_active: false,
            murder_wall_x: None,
            kid_x_vel: Some(0),
            kid_y_vel: Some(0),
            character_movement: Some(0),
            just_received_damage: Some(false),
            kid_collision_type: None,
            kid_collision_source: None,
            kid_x_direction: None,
            invulnerable_frames: Some(0),
            currently_transforming: Some(false),
            rng_offset: None,
            level_special_effects: None,
            gameplay_debug_fields: BTreeMap::new(),
            cpu_registers: None,
        }
    }

    fn gameplay_state(frame_index: usize, label: &str, kid_x_pos: u32) -> GameplayState {
        GameplayState {
            label: label.into(),
            frame_index,
            game_mode: 0x000C,
            current_player: 0,
            number_lives: 3,
            number_hitpoints: 2,
            current_level_id: 0,
            player_1_level_id: 0,
            current_helmet_id: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            kid_x_pos,
            kid_y_pos: 0,
            kid_x_vel: Some(0),
            kid_y_vel: Some(0),
            character_movement: Some(2),
            kid_collision_type: Some(0),
            invulnerable_frames: Some(0),
            just_received_damage: Some(false),
            kid_ground_probe: Some("ground".into()),
            kid_grounded_release_probe: None,
            kid_jump_gravity_probe: None,
            number_objects: 0,
            number_gfx_objects: 0,
        }
    }

    #[test]
    fn gameplay_drift_context_lines_marks_matching_and_drifting_frames() {
        let native_capture = GameplayCapture {
            scenario: "test".into(),
            frame_count: 5,
            missed_vblank_frames: 0,
            checkpoints: Vec::new(),
            states: vec![
                gameplay_state(10, "frame_00010", 0x0010_0000),
                gameplay_state(11, "frame_00011", 0x0011_0000),
                gameplay_state(12, "frame_00012", 0x0012_0000),
                gameplay_state(13, "frame_00013", 0x0013_0000),
                gameplay_state(14, "frame_00014", 0x0014_0000),
            ],
        };
        let mut oracle_capture = native_capture.clone();
        oracle_capture.states[1].kid_x_pos = 0x0016_0000;
        oracle_capture.states[2].kid_x_pos = 0x0019_0000;

        let lines = gameplay_drift_context_lines(&native_capture, &oracle_capture, 12, 1);

        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("! frame 11"));
        assert!(lines[0].contains("fields: kid_x_pos"));
        assert!(lines[1].starts_with("* frame 12"));
        assert!(lines[1].contains("fields: kid_x_pos"));
        assert!(lines[2].starts_with("= frame 13"));
        assert!(lines[2].ends_with("fields: -"));
    }

    #[test]
    fn annotate_gameplay_checkpoint_failure_appends_first_live_drift() {
        let native_capture = GameplayCapture {
            scenario: "test".into(),
            frame_count: 3,
            missed_vblank_frames: 0,
            states: vec![
                gameplay_state(0, "frame_00000", 0),
                gameplay_state(1, "frame_00001", 0x0010_0000),
                gameplay_state(2, "frame_00002", 0x0020_0000),
            ],
            checkpoints: vec![gameplay_state(2, "death_tail", 0x0020_0000)],
        };
        let oracle_capture = GameplayCapture {
            scenario: "test".into(),
            frame_count: 3,
            missed_vblank_frames: 0,
            states: vec![
                gameplay_state(0, "frame_00000", 0),
                gameplay_state(1, "frame_00001", 0x0018_0001),
                gameplay_state(2, "frame_00002", 0x0028_0001),
            ],
            checkpoints: vec![gameplay_state(2, "death_tail", 0x0028_0001)],
        };

        let annotated = annotate_gameplay_checkpoint_failure(
            anyhow!("gameplay state death_tail@2 diverged in kid X"),
            &native_capture,
            &oracle_capture,
            GameplayScenario::DeathRespawnLevel1,
        );

        let message = annotated.to_string();
        assert!(message.contains("death_tail@2 diverged in kid X"));
        assert!(message.contains("first live drift frame 2"));
        assert!(message.contains("fields: kid_x_pos"));
        assert!(message.contains("native context:"));
    }

    #[test]
    fn compare_native_states_reports_boot_level_entry_mode_8_release_gate() {
        let run_dir = Path::new("/tmp/release-gate-md-runtime-ctrl-pressed-mode8");
        let native = vec![oracle_state("level_entry", 520, 0x000C)];
        let oracle = vec![oracle_state("level_entry", 520, 0x0008)];

        let error = compare_native_states_against_oracle(run_dir, &native, &oracle)
            .expect_err("level_entry mode split should surface as a game-mode drift");

        let message = error.to_string();
        assert!(message.contains("state level_entry@520 diverged in game mode"));
        assert!(message.contains("native 000C, oracle 0008"));
    }

    #[test]
    fn compare_native_states_ignores_transient_level_entry_pose_loc_1bbd6() {
        let run_dir = Path::new("/tmp/boot-title-start-level1");
        let mut native = oracle_state("level_entry", 520, 0x0008);
        native.kid_x_pos = 0x0040_0000;
        native.kid_y_pos = 0x0068_0000;
        native.number_objects = 16;
        native.number_gfx_objects = 14;
        native.character_movement = Some(0);
        let mut oracle = oracle_state("level_entry", 520, 0x0008);
        oracle.kid_x_pos = 0x007F_0000;
        oracle.kid_y_pos = 0x0002_0000;
        oracle.number_objects = 4;
        oracle.number_gfx_objects = 3;
        oracle.character_movement = Some(3);

        compare_native_states_against_oracle(run_dir, &[native], &[oracle])
            .expect("level_entry should only assert the loc_1BBD6 hand-off identity fields");
    }

    #[test]
    fn compare_native_states_ignores_eol_visual_worker_roster_counts_loc_c21e_d770() {
        let run_dir = Path::new("/tmp/eol-visual-worker-roster");
        let mut native = oracle_state("eol_tail", 199, 0x000C);
        native.number_objects = 3;
        native.number_gfx_objects = 3;
        native.active_enemies.clear();
        let mut oracle = native.clone();
        oracle.number_objects = 20;
        oracle.number_gfx_objects = 65;
        oracle.active_enemies = vec![OracleEnemyState {
            object_type: 0,
            x_pos: 0x0610_0000,
            y_pos: 0x0200_0000,
            x_velocity: 0,
            y_velocity: 0,
            facing: EnemyFacing::Left,
            collision_type: Some(0),
            collision_source: Some(0),
            layout_index: None,
            layout_ptr: Some(0xFFFF_0000),
            parent_object_ptr: None,
            gfx_slot_index: Some(30),
            on_ground: None,
            current_hp: Some(0),
            debug_fields: BTreeMap::new(),
        }];

        compare_native_states_against_oracle(run_dir, &[native], &[oracle]).expect(
            "ASM loc_C21E/sub_C254 and loc_D4CE/loc_D770 EOL visual workers are rendered from native overlay state, not core object slots",
        );
    }

    #[test]
    fn compare_native_states_ignores_intro_object_roster_counts() {
        let run_dir = Path::new("/tmp/boot-title-idle10s");
        let mut native = oracle_state("title_tail", 599, 0x0004);
        native.number_objects = 0;
        native.number_gfx_objects = 0;
        let mut oracle = native.clone();
        oracle.number_objects = 2;
        oracle.number_gfx_objects = 0;

        compare_native_states_against_oracle(run_dir, &[native], &[oracle]).expect(
            "ASM Load_IntroSequence1 title workers are not authoritative gameplay roster counts",
        );
    }

    #[test]
    fn compare_native_states_still_rejects_gameplay_object_roster_counts() {
        let run_dir = Path::new("/tmp/gameplay-object-roster");
        let mut native = oracle_state("level_settle", 719, 0x000C);
        native.number_objects = 0;
        native.number_gfx_objects = 0;
        let mut oracle = native.clone();
        oracle.number_objects = 3;
        oracle.number_gfx_objects = 3;

        let error = compare_native_states_against_oracle(run_dir, &[native], &[oracle])
            .expect_err("gameplay object-count drift should still fail");

        assert!(format!("{error:#}").contains("object counts"));
    }

    #[test]
    fn gameplay_phase_shift_hint_reports_one_frame_native_lag() {
        let native_capture = GameplayCapture {
            scenario: "test".into(),
            frame_count: 7,
            missed_vblank_frames: 0,
            checkpoints: vec![gameplay_state(6, "death_tail", 0x0016_0000)],
            states: vec![
                gameplay_state(0, "frame_00000", 0x000F_0000),
                gameplay_state(1, "frame_00001", 0x0010_0000),
                gameplay_state(2, "frame_00002", 0x0011_0000),
                gameplay_state(3, "frame_00003", 0x0012_0000),
                gameplay_state(4, "frame_00004", 0x0013_0000),
                gameplay_state(5, "frame_00005", 0x0014_0000),
                gameplay_state(6, "frame_00006", 0x0016_0000),
            ],
        };
        let oracle_capture = GameplayCapture {
            scenario: "test".into(),
            frame_count: 7,
            missed_vblank_frames: 0,
            checkpoints: vec![gameplay_state(6, "death_tail", 0x0017_0000)],
            states: vec![
                gameplay_state(0, "frame_00000", 0x0010_0000),
                gameplay_state(1, "frame_00001", 0x0011_0000),
                gameplay_state(2, "frame_00002", 0x0012_0000),
                gameplay_state(3, "frame_00003", 0x0013_0000),
                gameplay_state(4, "frame_00004", 0x0014_0000),
                gameplay_state(5, "frame_00005", 0x0015_0000),
                gameplay_state(6, "frame_00006", 0x0017_0000),
            ],
        };

        let hint = gameplay_phase_shift_hint(&native_capture, &oracle_capture, 6)
            .expect("phase hint should be detected");
        assert!(hint.contains("native frames 1..5"));
        assert!(hint.contains("oracle one frame earlier"));
        assert!(hint.contains("likely phase-slip onset at frame 1"));
    }

    #[test]
    fn compare_gameplay_checkpoints_reports_character_movement_drift() {
        let run_dir = Path::new("/tmp/gameplay-character-movement-drift");
        let native = vec![gameplay_state(720, "frame_00720", 0x0020_0000)];
        let mut oracle = native.clone();
        oracle[0].character_movement = Some(0);

        let error = compare_gameplay_checkpoints_against_oracle(
            run_dir,
            GameplayScenario::DeathRespawnLevel1,
            &native,
            &oracle,
        )
        .expect_err("movement drift should fail checkpoint comparison");

        assert!(error.to_string().contains("character movement"));
    }

    #[test]
    fn gameplay_differing_fields_reports_character_movement_drift() {
        let native = gameplay_state(720, "frame_00720", 0x0020_0000);
        let mut oracle = native.clone();
        oracle.character_movement = Some(0);

        let fields = gameplay_differing_fields(&native, &oracle);

        assert_eq!(fields, vec!["character_movement"]);
    }
}
