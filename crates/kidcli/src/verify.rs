use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GameplayFrameEvidenceRecord {
    pub(crate) label: String,
    pub(crate) frame_index: usize,
    pub(crate) png_path: String,
    pub(crate) sha1: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FrameDiff {
    pub(crate) mean_absolute_error: f32,
    pub(crate) max_channel_delta: u8,
}

pub(crate) struct OracleComparison {
    pub(crate) run_dir: PathBuf,
    pub(crate) native: ScenarioCapture,
    pub(crate) oracle: OracleManifest,
}

pub(crate) fn verify_native_bootstrap(manifest: &AssetManifest) -> Result<()> {
    let scenario = Scenario::BootTitleStartLevel1;
    let script = scenario.script()?;
    let trace = run_native_script_for_scenario(manifest, &script, scenario);
    let title_frame = trace
        .get(359)
        .ok_or_else(|| anyhow!("native trace never reached title checkpoint"))?;
    if title_frame.game_mode != NativeGameMode::TitleScreen {
        bail!(
            "native bootstrap diverged before title: expected {:?}, got {:?}",
            NativeGameMode::TitleScreen,
            title_frame.game_mode
        );
    }

    let final_frame = trace
        .last()
        .ok_or_else(|| anyhow!("native bootstrap trace was empty"))?;
    if final_frame.game_mode != NativeGameMode::InGame {
        bail!(
            "native bootstrap did not reach gameplay: expected {:?}, got {:?}",
            NativeGameMode::InGame,
            final_frame.game_mode
        );
    }
    if final_frame.current_level.0 != 0 {
        bail!(
            "native bootstrap loaded the wrong first level: expected 0x00, got {:?}",
            final_frame.current_level.0
        );
    }

    println!("native bootstrap verification passed");
    Ok(())
}

pub(crate) fn verify_boot_probe(probe: &kid_md::CpuProbe) -> Result<()> {
    let expected_vdp_prefix = [0x8004, 0x8114, 0x8230, 0x833C];
    if probe.reset_sp != 0xFFFF_F7FE {
        bail!("unexpected reset SP: 0x{:08X}", probe.reset_sp);
    }
    if probe.reset_pc != 0x0000_04B0 {
        bail!("unexpected reset PC: 0x{:08X}", probe.reset_pc);
    }
    if probe.vdp_control_writes.len() < expected_vdp_prefix.len() {
        bail!("not enough VDP control writes observed during boot");
    }
    if probe.vdp_control_writes[..expected_vdp_prefix.len()] != expected_vdp_prefix {
        bail!(
            "unexpected VDP init prefix: {:?}",
            &probe.vdp_control_writes[..expected_vdp_prefix.len()]
        );
    }
    let saw_port_a = probe
        .accesses
        .iter()
        .any(|access| access.address == 0x00A1_0008 && access.value == 0);
    let saw_port_b = probe
        .accesses
        .iter()
        .any(|access| access.address == 0x00A1_000A && access.value == 0);
    let saw_port_c = probe
        .accesses
        .iter()
        .any(|access| access.address == 0x00A1_000C && access.value == 0);
    if !(saw_port_a && saw_port_b && saw_port_c) {
        bail!("expected cold-boot port control reads were not observed");
    }
    Ok(())
}

pub(crate) fn verify_startup_probe(probe: &kid_md::CpuProbe) -> Result<()> {
    verify_boot_probe(probe)?;

    if probe.final_pc < 0x0000_0700 {
        bail!(
            "startup probe did not reach the later startup path; final PC was 0x{:08X}",
            probe.final_pc
        );
    }

    let expected_game_program_writes = [0x8104, 0x8F02, 0xC000, 0x0000, 0x4000, 0x0010];
    if !contains_u16_subsequence(&probe.vdp_control_writes, &expected_game_program_writes) {
        bail!(
            "startup probe missed the expected GameProgram VDP setup writes: {}",
            format_u16_samples(&probe.vdp_control_writes)
        );
    }

    if probe.synthetic_vblanks == 0 || probe.v_int_counter == 0 {
        bail!(
            "startup probe never serviced a synthetic VBlank interrupt (synthetic_vblanks={}, v_int_counter={})",
            probe.synthetic_vblanks,
            probe.v_int_counter
        );
    }

    Ok(())
}

pub(crate) fn verify_level_probe(rom_bytes: &[u8], steps: u32) -> Result<()> {
    let entry_probe = probe_rom_with_script(rom_bytes, steps, &gameplay_entry_script()?)?;
    if entry_probe.game_mode != 0x000C {
        bail!(
            "expected in-level game mode 0x000C after scripted title skips, got 0x{:04X}",
            entry_probe.game_mode
        );
    }
    if entry_probe.camera_y_pos == 0 {
        bail!("expected non-zero level camera Y after gameplay entry");
    }
    if entry_probe.number_gfx_objects == 0 {
        bail!("expected active gameplay gfx objects after level entry");
    }

    let movement_probe = probe_rom_with_script(rom_bytes, steps, &gameplay_move_right_script()?)?;
    if movement_probe.game_mode != 0x000C {
        bail!(
            "expected to remain in level mode during scripted movement, got 0x{:04X}",
            movement_probe.game_mode
        );
    }
    if movement_probe.camera_x_pos <= entry_probe.camera_x_pos {
        bail!(
            "expected camera X to advance during scripted movement, got baseline 0x{:08X} and moved 0x{:08X}",
            entry_probe.camera_x_pos,
            movement_probe.camera_x_pos
        );
    }

    Ok(())
}

pub(crate) fn verify_gameplay_scenario(
    repo_root: &Path,
    rom_bytes: &[u8],
    scenario: GameplayScenario,
) -> Result<()> {
    let capture = capture_gameplay_scenario(rom_bytes, scenario)?;
    write_gameplay_artifacts(repo_root, &capture)?;
    match scenario {
        GameplayScenario::PauseResumeLevel1 => verify_pause_resume_level_1(&capture),
        GameplayScenario::DeathRespawnLevel1 => verify_death_respawn_level_1(&capture),
        GameplayScenario::ProgressTwoTransitions => verify_progress_two_transitions(&capture),
        GameplayScenario::SoakLevel10m => verify_soak_level_10m(&capture),
    }
}

pub(crate) fn verify_native_gameplay_scenario(
    repo_root: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<()> {
    let capture = capture_native_gameplay_scenario(manifest, scenario)?;
    write_native_gameplay_artifacts(repo_root, &capture)?;
    match scenario {
        GameplayScenario::PauseResumeLevel1 => verify_pause_resume_level_1(&capture),
        GameplayScenario::DeathRespawnLevel1 => verify_death_respawn_level_1(&capture),
        GameplayScenario::ProgressTwoTransitions => verify_progress_two_transitions(&capture),
        GameplayScenario::SoakLevel10m => verify_soak_level_10m(&capture),
    }
}

pub(crate) fn write_gameplay_artifacts(repo_root: &Path, capture: &GameplayCapture) -> Result<()> {
    let out_dir = repo_root
        .join(".cache/gameplay-runs")
        .join(&capture.scenario);
    write_gameplay_capture(&out_dir, capture)
}

pub(crate) fn write_native_gameplay_artifacts(repo_root: &Path, capture: &GameplayCapture) -> Result<()> {
    let out_dir = repo_root
        .join(".cache/native-gameplay-runs")
        .join(&capture.scenario);
    write_gameplay_capture(&out_dir, capture)
}

pub(crate) fn write_gameplay_capture(out_dir: &Path, capture: &GameplayCapture) -> Result<()> {
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    fs::write(
        out_dir.join("summary.json"),
        serde_json::to_vec_pretty(capture)?,
    )
    .with_context(|| format!("failed to write {}", out_dir.join("summary.json").display()))?;
    fs::write(
        out_dir.join("checkpoints.json"),
        serde_json::to_vec_pretty(&capture.checkpoints)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            out_dir.join("checkpoints.json").display()
        )
    })?;
    Ok(())
}

pub(crate) fn write_gameplay_frame_evidence(out_dir: &Path, frames: &[GameplayFrameEvidence]) -> Result<()> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let mut records = Vec::with_capacity(frames.len());
    for (label, frame_index, width, height, rgb) in frames {
        let png_name = format!("{frame_index:04}_{label}.png");
        let path = out_dir.join(&png_name);
        image::save_buffer_with_format(
            &path,
            rgb,
            *width as u32,
            *height as u32,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        records.push(GameplayFrameEvidenceRecord {
            label: label.clone(),
            frame_index: *frame_index,
            png_path: png_name,
            sha1: sha1_hex(rgb),
        });
    }
    fs::write(
        out_dir.join("frame-evidence.json"),
        serde_json::to_vec_pretty(&records)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            out_dir.join("frame-evidence.json").display()
        )
    })?;
    Ok(())
}

pub(crate) fn verify_pause_resume_level_1(capture: &GameplayCapture) -> Result<()> {
    const PAUSE_DRIFT_TOLERANCE: u32 = 0x0010_0000;

    if capture.missed_vblank_frames != 0 {
        bail!(
            "pause_resume_level1 missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let level_settle = gameplay_checkpoint(capture, "level_settle")?;
    let pre_pause = gameplay_checkpoint(capture, "pre_pause")?;
    let paused_tail = gameplay_checkpoint(capture, "paused_tail")?;
    let resumed_tail = gameplay_checkpoint(capture, "resumed_tail")?;

    if level_settle.game_mode != 0x000C || pre_pause.game_mode != 0x000C {
        bail!("pause_resume_level1 never stabilised in gameplay mode before pause");
    }
    if pre_pause.camera_x_pos <= level_settle.camera_x_pos
        || pre_pause.kid_x_pos <= level_settle.kid_x_pos
    {
        bail!("pause_resume_level1 did not advance before the pause point");
    }
    if paused_tail.camera_x_pos.abs_diff(pre_pause.camera_x_pos) > PAUSE_DRIFT_TOLERANCE
        || paused_tail.kid_x_pos.abs_diff(pre_pause.kid_x_pos) > PAUSE_DRIFT_TOLERANCE
    {
        bail!("pause_resume_level1 kept advancing while paused");
    }
    if resumed_tail.camera_x_pos <= paused_tail.camera_x_pos
        || resumed_tail.kid_x_pos <= paused_tail.kid_x_pos
    {
        bail!("pause_resume_level1 did not resume movement after unpausing");
    }

    Ok(())
}

pub(crate) fn verify_death_respawn_level_1(capture: &GameplayCapture) -> Result<()> {
    if capture.missed_vblank_frames != 0 {
        bail!(
            "death_respawn_level1 missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let level_settle = gameplay_checkpoint(capture, "level_settle")?;
    let pre_death = gameplay_checkpoint(capture, "pre_death")?;
    let death_tail = gameplay_checkpoint(capture, "death_tail")?;
    let respawn_tail = gameplay_checkpoint(capture, "respawn_tail")?;
    if level_settle.game_mode != 0x000C || pre_death.game_mode != 0x000C {
        bail!("death_respawn_level1 never stabilised in gameplay before the death window");
    }
    if death_tail.number_lives >= pre_death.number_lives {
        bail!(
            "death_respawn_level1 did not lose a life (before {}, after {})",
            pre_death.number_lives,
            death_tail.number_lives
        );
    }
    if respawn_tail.game_mode != 0x000C {
        bail!(
            "death_respawn_level1 did not return to gameplay mode after death: 0x{:04X}",
            respawn_tail.game_mode
        );
    }
    if respawn_tail.camera_x_pos == 0 && respawn_tail.kid_x_pos == 0 {
        bail!("death_respawn_level1 respawned into an invalid position");
    }

    Ok(())
}

pub(crate) fn verify_progress_two_transitions(capture: &GameplayCapture) -> Result<()> {
    if capture.missed_vblank_frames != 0 {
        bail!(
            "progress_two_transitions missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let level_settle = gameplay_checkpoint(capture, "level_settle")?;
    let run_end = gameplay_checkpoint(capture, "run_end")?;
    if level_settle.game_mode != 0x000C {
        bail!("progress_two_transitions never reached stable in-level gameplay");
    }
    let distinct_levels = distinct_level_sequence(&capture.states);
    if distinct_levels.len() < 3 {
        bail!(
            "progress_two_transitions saw only {:?}; expected at least three distinct level ids",
            distinct_levels
        );
    }
    if run_end.number_objects == 0 || run_end.number_gfx_objects == 0 {
        bail!("progress_two_transitions ended without active gameplay objects");
    }
    if run_end.number_lives == 0 {
        bail!("progress_two_transitions exhausted all lives before the run finished");
    }
    if run_end.current_level_id != *distinct_levels.last().unwrap_or(&run_end.current_level_id) {
        bail!(
            "progress_two_transitions ended on level {:04X} after visiting {:?}",
            run_end.current_level_id,
            distinct_levels
        );
    }

    Ok(())
}

pub(crate) fn verify_soak_level_10m(capture: &GameplayCapture) -> Result<()> {
    if capture.missed_vblank_frames != 0 {
        bail!(
            "soak_level_10m missed {} vblank frames; artefacts in .cache/gameplay-runs/{}",
            capture.missed_vblank_frames,
            capture.scenario
        );
    }
    let soak_end = gameplay_checkpoint(capture, "soak_end")?;
    if gameplay_checkpoint(capture, "level_settle")?.game_mode != 0x000C {
        bail!("soak_level_10m never reached stable in-level gameplay");
    }
    if soak_end.game_mode == 0 || soak_end.number_objects == 0 || soak_end.number_gfx_objects == 0 {
        bail!("soak_level_10m ended in an invalid gameplay state");
    }
    if soak_end.number_lives == 0 {
        bail!("soak_level_10m exhausted all lives");
    }
    let distinct_levels = distinct_level_sequence(&capture.states);
    if distinct_levels.len() < 8 {
        bail!(
            "soak_level_10m visited only {:?}; expected broad transition coverage",
            distinct_levels
        );
    }

    Ok(())
}

pub(crate) fn gameplay_checkpoint<'a>(capture: &'a GameplayCapture, label: &str) -> Result<&'a GameplayState> {
    capture
        .checkpoints
        .iter()
        .find(|state| state.label == label)
        .ok_or_else(|| anyhow!("missing gameplay checkpoint {label}"))
}

pub(crate) fn distinct_level_sequence(states: &[GameplayState]) -> Vec<u16> {
    let mut levels = Vec::new();
    for state in states {
        if levels.last().copied() != Some(state.current_level_id) {
            levels.push(state.current_level_id);
        }
    }
    levels
}

pub(crate) fn verify_audio_driver_probe(rom_bytes: &[u8], steps: u32) -> Result<()> {
    let probe = probe_rom_with_script(rom_bytes, steps, &gameplay_entry_script()?)?;
    if probe.ym2612_write_count == 0 {
        bail!("expected the Z80 sound path to issue YM2612 writes by level entry");
    }
    if probe.psg_write_count == 0 {
        bail!("expected the Z80 sound path to issue PSG writes by level entry");
    }

    Ok(())
}

pub(crate) fn verify_audio_output(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<()> {
    let comparison = capture_oracle_pair(repo_root, rom_path, rom_bytes, scenario)?;
    compare_audio_against_oracle(&comparison)
}

pub(crate) fn verify_frames(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<()> {
    let comparison = capture_oracle_pair(repo_root, rom_path, rom_bytes, scenario)?;
    compare_frames_against_oracle(&comparison)
}

pub(crate) fn verify_oracle(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<()> {
    let comparison = capture_oracle_pair(repo_root, rom_path, rom_bytes, scenario)?;
    compare_audio_against_oracle(&comparison)?;
    compare_frames_against_oracle(&comparison)
}

pub(crate) fn capture_oracle_pair(
    repo_root: &Path,
    rom_path: &Path,
    rom_bytes: &[u8],
    scenario: Scenario,
) -> Result<OracleComparison> {
    let env = ensure_oracles(repo_root)?;
    let run_dir = repo_root.join(".cache/oracle-runs").join(scenario.slug());
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

    let native = capture_scenario(rom_bytes, scenario)?;
    write_native_artifacts(&native_dir, &native)?;
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

    Ok(OracleComparison {
        run_dir,
        native,
        oracle,
    })
}

pub(crate) fn compare_audio_against_oracle(comparison: &OracleComparison) -> Result<()> {
    let native_analysis = analyse_audio_samples(&comparison.native.samples, AUDIO_SAMPLE_RATE);
    let oracle_samples = read_wav_f32(&comparison.oracle.audio_path)?;
    let oracle_analysis = analyse_audio_samples(&oracle_samples, comparison.oracle.sample_rate);
    let trace_path = comparison.run_dir.join("native/audio-trace.json");

    if native_analysis.change_count_left == 0 && native_analysis.change_count_right == 0 {
        bail!(
            "native audio remained constant across {}; trace {}",
            comparison.run_dir.display(),
            trace_path.display()
        );
    }
    if native_analysis.peak_left < 0.01 && native_analysis.peak_right < 0.01 {
        bail!(
            "native audio peak was too low (left {:.4}, right {:.4}); trace {}",
            native_analysis.peak_left,
            native_analysis.peak_right,
            trace_path.display()
        );
    }
    let native_onset = native_analysis
        .first_dynamic_frame
        .ok_or_else(|| anyhow!("native audio never became dynamic"))?;
    let oracle_onset = oracle_analysis
        .first_dynamic_frame
        .ok_or_else(|| anyhow!("oracle audio never became dynamic"))?;
    let onset_delta_seconds =
        native_onset.abs_diff(oracle_onset) as f32 / comparison.oracle.sample_rate as f32;
    if onset_delta_seconds > 0.5 {
        bail!(
            "audio onset drift was {:.3}s (native {:.3}s @ sample {}, oracle {:.3}s @ sample {}); native peaks {:.4}/{:.4}; artefacts in {}; trace {}",
            onset_delta_seconds,
            native_onset as f32 / AUDIO_SAMPLE_RATE as f32,
            native_onset,
            oracle_onset as f32 / comparison.oracle.sample_rate as f32,
            oracle_onset,
            native_analysis.peak_left,
            native_analysis.peak_right,
            comparison.run_dir.display(),
            trace_path.display()
        );
    }
    if native_analysis.tail_change_count_left == 0 && native_analysis.tail_change_count_right == 0 {
        bail!(
            "native audio tail stayed constant; artefacts in {}; trace {}",
            comparison.run_dir.display(),
            trace_path.display()
        );
    }
    let similarity = audio_envelope_similarity(
        &native_analysis.rms_windows,
        &oracle_analysis.rms_windows,
        native_onset,
        oracle_onset,
        comparison.oracle.sample_rate,
    );
    if similarity < 0.90 {
        bail!(
            "audio envelope similarity {:.3} was below 0.90; native peaks {:.4}/{:.4}; artefacts in {}; trace {}",
            similarity,
            native_analysis.peak_left,
            native_analysis.peak_right,
            comparison.run_dir.display(),
            trace_path.display()
        );
    }

    Ok(())
}

pub(crate) fn compare_frames_against_oracle(comparison: &OracleComparison) -> Result<()> {
    compare_states_against_oracle(comparison)?;

    if comparison.native.frames.len() != comparison.oracle.frames.len() {
        bail!(
            "native captured {} frames but oracle captured {}; artefacts in {}",
            comparison.native.frames.len(),
            comparison.oracle.frames.len(),
            comparison.run_dir.display()
        );
    }

    for (native, oracle) in comparison
        .native
        .frames
        .iter()
        .zip(&comparison.oracle.frames)
    {
        if native.label != oracle.label || native.frame_index != oracle.frame_index {
            bail!(
                "frame checkpoint mismatch: native {}@{} vs oracle {}@{}",
                native.label,
                native.frame_index,
                oracle.label,
                oracle.frame_index
            );
        }
        if native.label == "level_entry" {
            continue;
        }
        let oracle_rgb = load_png_rgb(&oracle.png_path)?;
        let diff = compare_rgb_frames(&native.rgb, &oracle_rgb)?;
        let mae_limit = match native.label.as_str() {
            "title" => 16.0,
            "level_entry" | "level_settle" => 28.0,
            _ => 32.0,
        };
        if diff.mean_absolute_error > mae_limit {
            bail!(
                "frame {}@{} diverged too far (MAE {:.2}, max delta {}, native {}, oracle {}); artefacts in {}",
                native.label,
                native.frame_index,
                diff.mean_absolute_error,
                diff.max_channel_delta,
                native.sha1,
                oracle.sha1,
                comparison.run_dir.display()
            );
        }
    }

    Ok(())
}

pub(crate) fn compare_states_against_oracle(comparison: &OracleComparison) -> Result<()> {
    if comparison.native.states.len() != comparison.oracle.states.len() {
        bail!(
            "native captured {} states but oracle captured {}; artefacts in {}",
            comparison.native.states.len(),
            comparison.oracle.states.len(),
            comparison.run_dir.display()
        );
    }

    for (native, oracle) in comparison
        .native
        .states
        .iter()
        .zip(&comparison.oracle.states)
    {
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
                comparison.run_dir.display()
            );
        }
        if native.current_player != oracle.current_player {
            bail!(
                "state {}@{} diverged in current player (native {:02X}, oracle {:02X}); artefacts in {}",
                native.label,
                native.frame_index,
                native.current_player,
                oracle.current_player,
                comparison.run_dir.display()
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
                comparison.run_dir.display()
            );
        }
        if native.label == "level_entry" {
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
                comparison.run_dir.display()
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
                comparison.run_dir.display()
            );
        }
        if native.number_objects.abs_diff(oracle.number_objects) > 8
            || native
                .number_gfx_objects
                .abs_diff(oracle.number_gfx_objects)
                > 8
        {
            bail!(
                "state {}@{} diverged in object counts (native {} / {}, oracle {} / {}); artefacts in {}",
                native.label,
                native.frame_index,
                native.number_objects,
                native.number_gfx_objects,
                oracle.number_objects,
                oracle.number_gfx_objects,
                comparison.run_dir.display()
            );
        }
    }

    Ok(())
}

pub(crate) fn write_native_artifacts(out_dir: &Path, capture: &ScenarioCapture) -> Result<()> {
    write_wav(
        &out_dir.join("audio.wav"),
        AUDIO_SAMPLE_RATE,
        &capture.samples,
    )?;
    fs::write(
        out_dir.join("audio-trace.json"),
        serde_json::to_vec_pretty(&capture.audio_trace)?,
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            out_dir.join("audio-trace.json").display()
        )
    })?;
    fs::write(
        out_dir.join("state.json"),
        serde_json::to_vec_pretty(
            &capture
                .states
                .iter()
                .map(native_state_to_oracle_state)
                .collect::<Vec<_>>(),
        )?,
    )
    .with_context(|| format!("failed to write {}", out_dir.join("state.json").display()))?;
    for frame in &capture.frames {
        let path = out_dir.join(format!("{:04}_{}.png", frame.frame_index, frame.label));
        image::save_buffer_with_format(
            &path,
            &frame.rgb,
            frame.width as u32,
            frame.height as u32,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

pub(crate) fn native_state_to_oracle_state(state: &CapturedState) -> OracleState {
    OracleState {
        label: state.label.clone(),
        frame_index: state.frame_index,
        game_mode: state.game_mode,
        current_player: state.current_player,
        number_hitpoints: state.number_hitpoints,
        current_helmet_id: state.current_helmet_id,
        current_level_id: state.current_level_id,
        player_1_level_id: state.player_1_level_id,
        camera_x_pos: state.camera_x_pos,
        camera_y_pos: state.camera_y_pos,
        kid_x_pos: state.kid_x_pos,
        kid_y_pos: state.kid_y_pos,
        number_objects: state.number_objects,
        number_gfx_objects: state.number_gfx_objects,
        active_enemies: state.active_enemies.clone(),
        game_paused: false,
    }
}

pub(crate) fn load_png_rgb(path: &Path) -> Result<Vec<u8>> {
    let image = image::open(path)
        .with_context(|| format!("failed to open oracle frame {}", path.display()))?;
    Ok(image.to_rgb8().into_raw())
}

pub(crate) fn compare_rgb_frames(native_rgb: &[u8], oracle_rgb: &[u8]) -> Result<FrameDiff> {
    if native_rgb.len() != oracle_rgb.len() {
        bail!(
            "frame size mismatch: native {} bytes vs oracle {} bytes",
            native_rgb.len(),
            oracle_rgb.len()
        );
    }

    let mut total_delta = 0u64;
    let mut max_channel_delta = 0u8;
    for (native, oracle) in native_rgb.iter().zip(oracle_rgb) {
        let delta = native.abs_diff(*oracle);
        total_delta += u64::from(delta);
        max_channel_delta = max_channel_delta.max(delta);
    }
    Ok(FrameDiff {
        mean_absolute_error: total_delta as f32 / native_rgb.len() as f32,
        max_channel_delta,
    })
}

pub(crate) fn audio_envelope_similarity(
    native: &[f32],
    oracle: &[f32],
    native_onset_frame: usize,
    oracle_onset_frame: usize,
    sample_rate: u32,
) -> f32 {
    let window_frames = (sample_rate / 4).max(1) as usize;
    let native_start = native_onset_frame / window_frames;
    let oracle_start = oracle_onset_frame / window_frames;
    let mut best_similarity = 0.0f32;

    for lag in -1isize..=1 {
        let (shifted_native_start, shifted_oracle_start) = if lag >= 0 {
            (native_start.saturating_add(lag as usize), oracle_start)
        } else {
            (native_start, oracle_start.saturating_add((-lag) as usize))
        };
        let overlapping = native
            .len()
            .saturating_sub(shifted_native_start)
            .min(oracle.len().saturating_sub(shifted_oracle_start));
        if overlapping < 2 {
            continue;
        }

        let native_slice = &native[shifted_native_start..shifted_native_start + overlapping];
        let oracle_slice = &oracle[shifted_oracle_start..shifted_oracle_start + overlapping];
        let mut numerator = 0.0f32;
        let mut native_energy = 0.0f32;
        let mut oracle_energy = 0.0f32;
        for (native_value, oracle_value) in native_slice.iter().zip(oracle_slice) {
            numerator += native_value * oracle_value;
            native_energy += native_value * native_value;
            oracle_energy += oracle_value * oracle_value;
        }
        if native_energy <= f32::EPSILON || oracle_energy <= f32::EPSILON {
            continue;
        }
        let similarity = numerator / (native_energy.sqrt() * oracle_energy.sqrt());
        best_similarity = best_similarity.max(similarity);
    }

    best_similarity
}

pub(crate) fn contains_u16_subsequence(haystack: &[u16], needle: &[u16]) -> bool {
    if needle.is_empty() {
        return true;
    }

    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
