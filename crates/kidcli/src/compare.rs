use super::*;

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

    let script = scenario.script()?;
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

pub(crate) fn diagnose_native_gameplay_drift(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    from_frame: usize,
    limit: usize,
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
    let run_dir = repo_root
        .join(".cache/native-gameplay-oracle-runs")
        .join(scenario.slug());
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
    let (run_dir, native_capture, oracle_capture) =
        capture_native_gameplay_run(repo_root, scenario, oracle_capture_fn, native_capture_fn)?;
    compare_gameplay_checkpoints_against_oracle(
        &run_dir,
        scenario,
        &native_capture.checkpoints,
        &oracle_capture.checkpoints,
    )?;
    let frame_evidence = frame_evidence_fn()?;
    write_gameplay_frame_evidence(&run_dir.join("review"), &frame_evidence)?;
    Ok(())
}

pub(crate) fn capture_native_gameplay_frame_evidence(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<Vec<GameplayFrameEvidence>> {
    let script = scenario.script()?;
    let trace = run_native_script_for_gameplay(manifest, &script, scenario);
    render_gameplay_frame_evidence(manifest, &trace, scenario.checkpoints())
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
    }

    Ok(())
}

pub(crate) fn gameplay_checkpoint_tolerance(
    scenario: GameplayScenario,
    label: &str,
) -> GameplayCheckpointTolerance {
    match (scenario, label) {
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
        bail!(
            "gameplay state {}@{} diverged in {} (native {:08X}, oracle {:08X}); artefacts in {}",
            native.label,
            native.frame_index,
            field,
            native_value,
            oracle_value,
            run_dir.display()
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
        bail!(
            "gameplay state {}@{} diverged in {} (native {}, oracle {}); artefacts in {}",
            native.label,
            native.frame_index,
            field,
            native_value,
            oracle_value,
            run_dir.display()
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

pub(crate) fn gameplay_differing_fields(native: &GameplayState, oracle: &GameplayState) -> Vec<&'static str> {
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
        "number_objects" => state.number_objects.to_string(),
        "number_gfx_objects" => state.number_gfx_objects.to_string(),
        _ => String::from("<unknown>"),
    }
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
    OracleState {
        label: label.to_owned(),
        frame_index,
        game_mode: frame.game_mode_value,
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
    }
}

pub(crate) fn active_enemy_states_from_native_frame(frame: &NativeTraceFrame) -> Vec<OracleEnemyState> {
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
    enemies.sort_by_key(|enemy| (enemy.object_type, enemy.x_pos, enemy.y_pos));
}

pub(crate) fn compare_native_states_against_oracle(
    run_dir: &Path,
    native_states: &[OracleState],
    oracle_states: &[OracleState],
) -> Result<()> {
    const ENEMY_POSITION_TOLERANCE: u32 = 0x0010_0000;
    const ENEMY_VELOCITY_TOLERANCE: i32 = 0x0001_0000;
    let compare_enemy_snapshots = oracle_states
        .iter()
        .any(|state| !state.active_enemies.is_empty());
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
        if native.number_objects.abs_diff(oracle.number_objects) > 1
            || native
                .number_gfx_objects
                .abs_diff(oracle.number_gfx_objects)
                > 1
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
        if compare_enemy_snapshots {
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
    }

    Ok(())
}
