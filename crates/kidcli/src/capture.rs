use super::*;
use crate::scenarios::DynamicScenario;
use kid_core::run_script_with_trace;

pub(crate) const AUDIO_SAMPLE_RATE: u32 = 44_100;
pub(crate) const AUDIO_FRAME_SAMPLES: usize = 735;
pub(crate) const AUDIO_ANALYSIS_EPSILON: f32 = 1e-6;
pub(crate) const CAMERA_POSITION_TOLERANCE: u32 = 0x0004_0000;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AudioAnalysis {
    pub(crate) peak_left: f32,
    pub(crate) peak_right: f32,
    pub(crate) change_count_left: usize,
    pub(crate) change_count_right: usize,
    pub(crate) first_dynamic_frame: Option<usize>,
    pub(crate) tail_change_count_left: usize,
    pub(crate) tail_change_count_right: usize,
    pub(crate) rms_windows: Vec<f32>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScenarioCapture {
    pub(crate) samples: Vec<f32>,
    pub(crate) frames: Vec<CapturedFrame>,
    pub(crate) states: Vec<CapturedState>,
    pub(crate) audio_trace: AudioTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GameplayCapture {
    pub(crate) scenario: String,
    pub(crate) frame_count: usize,
    pub(crate) missed_vblank_frames: usize,
    pub(crate) states: Vec<GameplayState>,
    pub(crate) checkpoints: Vec<GameplayState>,
}

#[derive(Debug, Clone)]
pub(crate) struct CapturedFrame {
    pub(crate) label: String,
    pub(crate) frame_index: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) rgb: Vec<u8>,
    pub(crate) sha1: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapturedState {
    pub(crate) label: String,
    pub(crate) frame_index: usize,
    pub(crate) game_mode: u16,
    pub(crate) current_player: u8,
    pub(crate) number_hitpoints: u16,
    pub(crate) current_helmet_id: u16,
    pub(crate) current_level_id: u16,
    pub(crate) player_1_level_id: u16,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
    pub(crate) kid_x_pos: u32,
    pub(crate) kid_y_pos: u32,
    pub(crate) number_objects: u16,
    pub(crate) number_gfx_objects: u16,
    pub(crate) active_enemies: Vec<OracleEnemyState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GameplayState {
    pub(crate) label: String,
    pub(crate) frame_index: usize,
    pub(crate) game_mode: u16,
    pub(crate) current_player: u8,
    pub(crate) number_lives: u16,
    pub(crate) number_hitpoints: u16,
    pub(crate) current_level_id: u16,
    pub(crate) player_1_level_id: u16,
    pub(crate) current_helmet_id: u16,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
    pub(crate) kid_x_pos: u32,
    pub(crate) kid_y_pos: u32,
    pub(crate) kid_x_vel: Option<i32>,
    pub(crate) kid_y_vel: Option<i32>,
    pub(crate) character_movement: Option<u16>,
    pub(crate) kid_collision_type: Option<u16>,
    pub(crate) invulnerable_frames: Option<u8>,
    pub(crate) just_received_damage: Option<bool>,
    pub(crate) number_objects: u16,
    pub(crate) number_gfx_objects: u16,
}

pub(crate) type GameplayFrameEvidence = (String, usize, usize, usize, Vec<u8>);

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AudioTrace {
    pub(crate) scenario: String,
    pub(crate) sample_rate: u32,
    pub(crate) frame_samples: usize,
    pub(crate) z80_reset_count: usize,
    pub(crate) first_ym2612_write_frame: Option<usize>,
    pub(crate) first_ym2612_write_sample: Option<usize>,
    pub(crate) first_ym2612_key_on_frame: Option<usize>,
    pub(crate) first_ym2612_key_on_sample: Option<usize>,
    pub(crate) first_psg_write_frame: Option<usize>,
    pub(crate) first_psg_write_sample: Option<usize>,
    pub(crate) first_non_silent_sample: Option<usize>,
    pub(crate) first_significant_audio_sample: Option<usize>,
    pub(crate) peak_left: f32,
    pub(crate) peak_right: f32,
    pub(crate) frames: Vec<AudioTraceFrame>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AudioTraceFrame {
    pub(crate) frame_index: usize,
    pub(crate) z80_reset_count: usize,
    pub(crate) ym2612_write_count: usize,
    pub(crate) ym2612_key_on_count: usize,
    pub(crate) psg_write_count: usize,
    pub(crate) peak_left: f32,
    pub(crate) peak_right: f32,
}

pub(crate) fn capture_scenario(rom_bytes: &[u8], scenario: Scenario) -> Result<ScenarioCapture> {
    let mut runtime = MdRuntime::new(rom_bytes)?;
    let script = scenario.script()?;
    let checkpoints = scenario.checkpoints();
    let mut checkpoint_index = 0usize;
    let mut samples = Vec::with_capacity(scenario.frame_count() * AUDIO_FRAME_SAMPLES * 2);
    let mut frames = Vec::with_capacity(checkpoints.len());
    let mut states = Vec::with_capacity(checkpoints.len());
    let mut audio_frames = Vec::with_capacity(scenario.frame_count());
    let mut first_ym2612_write_frame = None;
    let mut first_ym2612_key_on_frame = None;
    let mut first_psg_write_frame = None;
    let mut previous_z80_reset_count = 0usize;
    let mut previous_ym2612_write_count = 0usize;
    let mut previous_ym2612_key_on_count = 0usize;
    let mut previous_psg_write_count = 0usize;

    for frame_index in 0..scenario.frame_count() {
        let buttons = frame_index
            .checked_sub(1)
            .and_then(|previous_frame| script.get(previous_frame))
            .copied()
            .unwrap_or(0);
        runtime.set_controller_1(buttons);
        runtime.run_until_frame_rendered(500_000);
        let snapshot = runtime.snapshot();
        while checkpoint_index < checkpoints.len()
            && checkpoints[checkpoint_index].frame_index == frame_index
        {
            let checkpoint = checkpoints[checkpoint_index];
            let rendered = render_frame(&snapshot);
            frames.push(CapturedFrame {
                label: checkpoint.label.to_string(),
                frame_index,
                sha1: sha1_hex(&rendered.rgb),
                width: rendered.width,
                height: rendered.height,
                rgb: rendered.rgb,
            });
            states.push(CapturedState {
                label: checkpoint.label.to_string(),
                frame_index,
                game_mode: snapshot.game_mode,
                current_player: snapshot.current_player,
                number_hitpoints: snapshot.number_hitpoints,
                current_helmet_id: snapshot.current_helmet_id,
                current_level_id: snapshot.current_level_id,
                player_1_level_id: snapshot.player_1_level_id,
                camera_x_pos: snapshot.camera_x_pos,
                camera_y_pos: snapshot.camera_y_pos,
                kid_x_pos: snapshot.kid_x_pos,
                kid_y_pos: snapshot.kid_y_pos,
                number_objects: snapshot.number_objects,
                number_gfx_objects: snapshot.number_gfx_objects,
                active_enemies: Vec::new(),
            });
            checkpoint_index += 1;
        }
        let z80_reset_count = snapshot.z80_reset_count;
        let ym2612_write_count = snapshot.ym2612_write_count;
        let ym2612_key_on_count = snapshot.ym2612_key_on_count;
        let psg_write_count = snapshot.psg_write_count;
        if first_ym2612_write_frame.is_none() && ym2612_write_count > 0 {
            first_ym2612_write_frame = Some(frame_index);
        }
        if first_ym2612_key_on_frame.is_none() && ym2612_key_on_count > 0 {
            first_ym2612_key_on_frame = Some(frame_index);
        }
        if first_psg_write_frame.is_none() && psg_write_count > 0 {
            first_psg_write_frame = Some(frame_index);
        }

        let z80_reset_delta = z80_reset_count.saturating_sub(previous_z80_reset_count);
        let ym2612_write_delta = ym2612_write_count.saturating_sub(previous_ym2612_write_count);
        let ym2612_key_on_delta = ym2612_key_on_count.saturating_sub(previous_ym2612_key_on_count);
        let psg_write_delta = psg_write_count.saturating_sub(previous_psg_write_count);
        previous_z80_reset_count = z80_reset_count;
        previous_ym2612_write_count = ym2612_write_count;
        previous_ym2612_key_on_count = ym2612_key_on_count;
        previous_psg_write_count = psg_write_count;

        let frame_samples = runtime.generate_audio_samples(AUDIO_SAMPLE_RATE, AUDIO_FRAME_SAMPLES);
        let (peak_left, peak_right) = stereo_peak(&frame_samples);
        audio_frames.push(AudioTraceFrame {
            frame_index,
            z80_reset_count: z80_reset_delta,
            ym2612_write_count: ym2612_write_delta,
            ym2612_key_on_count: ym2612_key_on_delta,
            psg_write_count: psg_write_delta,
            peak_left,
            peak_right,
        });
        samples.extend(frame_samples);
    }

    let audio_analysis = analyse_audio_samples(&samples, AUDIO_SAMPLE_RATE);
    let first_non_silent_sample = first_non_silent_sample(&samples);
    let audio_trace = AudioTrace {
        scenario: scenario.slug().to_string(),
        sample_rate: AUDIO_SAMPLE_RATE,
        frame_samples: AUDIO_FRAME_SAMPLES,
        z80_reset_count: previous_z80_reset_count,
        first_ym2612_write_sample: first_ym2612_write_frame
            .map(|frame| frame * AUDIO_FRAME_SAMPLES),
        first_ym2612_write_frame,
        first_ym2612_key_on_sample: first_ym2612_key_on_frame
            .map(|frame| frame * AUDIO_FRAME_SAMPLES),
        first_ym2612_key_on_frame,
        first_psg_write_sample: first_psg_write_frame.map(|frame| frame * AUDIO_FRAME_SAMPLES),
        first_psg_write_frame,
        first_non_silent_sample,
        first_significant_audio_sample: audio_analysis.first_dynamic_frame,
        peak_left: audio_analysis.peak_left,
        peak_right: audio_analysis.peak_right,
        frames: audio_frames,
    };

    Ok(ScenarioCapture {
        samples,
        frames,
        states,
        audio_trace,
    })
}

pub(crate) fn capture_gameplay_scenario(
    rom_bytes: &[u8],
    scenario: GameplayScenario,
) -> Result<GameplayCapture> {
    capture_dynamic_gameplay_scenario(
        rom_bytes,
        &scenario.to_dynamic()?,
        scenario.uses_level_skip_cheat(),
    )
}

pub(crate) fn capture_dynamic_gameplay_scenario(
    rom_bytes: &[u8],
    scenario: &DynamicScenario,
    uses_level_skip_cheat: bool,
) -> Result<GameplayCapture> {
    let mut runtime = MdRuntime::new(rom_bytes)?;
    let checkpoints = &scenario.checkpoints;
    let mut checkpoint_index = 0usize;
    let mut states = Vec::with_capacity(scenario.frame_count);
    let mut captured_checkpoints = Vec::with_capacity(checkpoints.len());
    let mut missed_vblank_frames = 0usize;

    for frame_index in 0..scenario.frame_count {
        if uses_level_skip_cheat && frame_index >= 720 {
            runtime.set_level_skip_cheat_enabled(true);
        }
        let buttons = scenario.script.get(frame_index).copied().unwrap_or(0);
        runtime.set_controller_1(buttons);
        if !runtime.run_until_vblank(500_000) {
            missed_vblank_frames += 1;
        }
        let snapshot = runtime.snapshot();
        let state = gameplay_state_from_probe(frame_index, &snapshot);
        while checkpoint_index < checkpoints.len()
            && checkpoints[checkpoint_index].frame_index == frame_index
        {
            let checkpoint = &checkpoints[checkpoint_index];
            captured_checkpoints.push(GameplayState {
                label: checkpoint.label.clone(),
                ..state.clone()
            });
            checkpoint_index += 1;
        }
        states.push(state);
    }

    Ok(GameplayCapture {
        scenario: scenario.slug.clone(),
        frame_count: scenario.frame_count,
        missed_vblank_frames,
        states,
        checkpoints: captured_checkpoints,
    })
}

pub(crate) fn gameplay_state_from_probe(
    frame_index: usize,
    snapshot: &kid_md::CpuProbe,
) -> GameplayState {
    GameplayState {
        label: format!("frame_{frame_index:05}"),
        frame_index,
        game_mode: snapshot.game_mode,
        current_player: snapshot.current_player,
        number_lives: snapshot.number_lives,
        number_hitpoints: snapshot.number_hitpoints,
        current_level_id: snapshot.current_level_id,
        player_1_level_id: snapshot.player_1_level_id,
        current_helmet_id: snapshot.current_helmet_id,
        camera_x_pos: snapshot.camera_x_pos,
        camera_y_pos: snapshot.camera_y_pos,
        kid_x_pos: snapshot.kid_x_pos,
        kid_y_pos: snapshot.kid_y_pos,
        kid_x_vel: Some(snapshot.kid_x_vel),
        kid_y_vel: Some(snapshot.kid_y_vel),
        character_movement: Some(snapshot.character_movement),
        kid_collision_type: Some(snapshot.kid_collision_type),
        invulnerable_frames: Some(snapshot.invulnerable_frames),
        just_received_damage: Some(snapshot.just_received_damage),
        number_objects: snapshot.number_objects,
        number_gfx_objects: snapshot.number_gfx_objects,
    }
}

pub(crate) fn capture_native_gameplay_scenario(
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<GameplayCapture> {
    if !matches!(
        scenario,
        GameplayScenario::PauseResumeLevel1
            | GameplayScenario::DeathRespawnLevel1
            | GameplayScenario::ProgressTwoTransitions
            | GameplayScenario::SoakLevel10m
    ) {
        bail!(
            "verify-native-gameplay does not support scenario {}",
            scenario.slug()
        );
    }

    capture_native_dynamic_gameplay_scenario(manifest, &scenario.to_dynamic()?)
}

pub(crate) fn capture_native_dynamic_gameplay_scenario(
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
) -> Result<GameplayCapture> {
    let checkpoints = &scenario.checkpoints;
    let trace = run_native_script_dynamic(manifest, &scenario.script, scenario.frame_count);
    let mut checkpoint_index = 0usize;
    let mut states = Vec::with_capacity(trace.len());
    let mut captured_checkpoints = Vec::with_capacity(checkpoints.len());

    for snapshot in trace {
        let state = gameplay_state_from_native_trace(&snapshot);
        while checkpoint_index < checkpoints.len()
            && checkpoints[checkpoint_index].frame_index == snapshot.frame_index
        {
            let checkpoint = &checkpoints[checkpoint_index];
            captured_checkpoints.push(GameplayState {
                label: checkpoint.label.clone(),
                ..state.clone()
            });
            checkpoint_index += 1;
        }
        states.push(state);
    }

    Ok(GameplayCapture {
        scenario: format!("native_{}", scenario.slug),
        frame_count: scenario.frame_count,
        missed_vblank_frames: 0,
        states,
        checkpoints: captured_checkpoints,
    })
}

pub(crate) fn gameplay_state_from_native_trace(snapshot: &NativeTraceFrame) -> GameplayState {
    GameplayState {
        label: format!("frame_{:05}", snapshot.frame_index),
        frame_index: snapshot.frame_index,
        game_mode: snapshot.game_mode_value,
        current_player: snapshot.current_player,
        number_lives: snapshot.number_lives,
        number_hitpoints: snapshot.number_hitpoints,
        current_level_id: u16::from(snapshot.current_level.0),
        player_1_level_id: u16::from(snapshot.player_1_level_id.0),
        current_helmet_id: snapshot.current_helmet_id,
        camera_x_pos: snapshot.camera_x_pos,
        camera_y_pos: snapshot.camera_y_pos,
        kid_x_pos: snapshot.kid_x_pos.unwrap_or(0),
        kid_y_pos: snapshot.kid_y_pos.unwrap_or(0),
        kid_x_vel: snapshot.kid_x_velocity,
        kid_y_vel: snapshot.kid_y_velocity,
        character_movement: snapshot.character_movement,
        kid_collision_type: None,
        invulnerable_frames: Some(snapshot.invulnerable_frames.try_into().unwrap_or(u8::MAX)),
        just_received_damage: Some(snapshot.just_received_damage),
        number_objects: snapshot.number_objects,
        number_gfx_objects: snapshot.number_gfx_objects,
    }
}

pub(crate) fn analyse_audio_samples(samples: &[f32], sample_rate: u32) -> AudioAnalysis {
    let mut peak_left = 0.0f32;
    let mut peak_right = 0.0f32;
    let mut change_count_left = 0usize;
    let mut change_count_right = 0usize;
    let mut first_dynamic_frame = None;
    let mut previous_left: Option<f32> = None;
    let mut previous_right: Option<f32> = None;
    let total_frames = samples.len() / 2;
    let tail_window_frames = (sample_rate as usize) * 3;
    let tail_window_start = total_frames.saturating_sub(tail_window_frames);
    let mut tail_change_count_left = 0usize;
    let mut tail_change_count_right = 0usize;
    let rms_window_frames = (sample_rate / 4).max(1) as usize;
    let onset_window_frames = (sample_rate / 20).max(1) as usize;
    let mut rms_windows = Vec::new();
    let mut rms_accumulator = 0.0f64;
    let mut rms_frames = 0usize;
    let mut onset_activity_windows = Vec::new();

    for (frame_index, frame) in samples.chunks_exact(2).enumerate() {
        let left = frame[0];
        let right = frame[1];
        peak_left = peak_left.max(left.abs());
        peak_right = peak_right.max(right.abs());
        if let Some(previous) = previous_left
            && (left - previous).abs() > AUDIO_ANALYSIS_EPSILON
        {
            change_count_left += 1;
            if frame_index >= tail_window_start {
                tail_change_count_left += 1;
            }
            if first_dynamic_frame.is_none() {
                first_dynamic_frame = Some(frame_index);
            }
        }
        if let Some(previous) = previous_right
            && (right - previous).abs() > AUDIO_ANALYSIS_EPSILON
        {
            change_count_right += 1;
            if frame_index >= tail_window_start {
                tail_change_count_right += 1;
            }
            if first_dynamic_frame.is_none() {
                first_dynamic_frame = Some(frame_index);
            }
        }
        previous_left = Some(left);
        previous_right = Some(right);
        rms_accumulator += f64::from((left * left + right * right) * 0.5);
        rms_frames += 1;
        if rms_frames == rms_window_frames {
            rms_windows.push((rms_accumulator / rms_frames as f64).sqrt() as f32);
            rms_accumulator = 0.0;
            rms_frames = 0;
        }
    }

    if rms_frames != 0 {
        rms_windows.push((rms_accumulator / rms_frames as f64).sqrt() as f32);
    }

    for window in samples.chunks_exact(onset_window_frames * 2) {
        let mut left_mean = 0.0f64;
        let mut right_mean = 0.0f64;
        let mut channel_frames = 0usize;
        for frame in window.chunks_exact(2) {
            left_mean += f64::from(frame[0]);
            right_mean += f64::from(frame[1]);
            channel_frames += 1;
        }
        if channel_frames == 0 {
            continue;
        }
        left_mean /= channel_frames as f64;
        right_mean /= channel_frames as f64;
        let mut variance = 0.0f64;
        for frame in window.chunks_exact(2) {
            let left_delta = f64::from(frame[0]) - left_mean;
            let right_delta = f64::from(frame[1]) - right_mean;
            variance += (left_delta * left_delta + right_delta * right_delta) * 0.5;
        }
        onset_activity_windows.push((variance / channel_frames as f64).sqrt() as f32);
    }

    if first_dynamic_frame.is_some() {
        let peak_activity = onset_activity_windows
            .iter()
            .copied()
            .fold(0.0f32, f32::max);
        let activity_threshold = (peak_activity * 0.15).max(5e-4);
        first_dynamic_frame = onset_activity_windows
            .windows(2)
            .position(|window| window[0] >= activity_threshold && window[1] >= activity_threshold)
            .map(|index| index * onset_window_frames);
    }

    AudioAnalysis {
        peak_left,
        peak_right,
        change_count_left,
        change_count_right,
        first_dynamic_frame,
        tail_change_count_left,
        tail_change_count_right,
        rms_windows,
    }
}

pub(crate) fn stereo_peak(samples: &[f32]) -> (f32, f32) {
    let mut peak_left = 0.0f32;
    let mut peak_right = 0.0f32;
    for frame in samples.chunks_exact(2) {
        peak_left = peak_left.max(frame[0].abs());
        peak_right = peak_right.max(frame[1].abs());
    }
    (peak_left, peak_right)
}

pub(crate) fn first_non_silent_sample(samples: &[f32]) -> Option<usize> {
    samples.chunks_exact(2).position(|frame| {
        frame[0].abs() > AUDIO_ANALYSIS_EPSILON || frame[1].abs() > AUDIO_ANALYSIS_EPSILON
    })
}

pub(crate) fn dump_audio(rom_bytes: &[u8], scenario: Scenario, out: &Path) -> Result<()> {
    let capture = capture_scenario(rom_bytes, scenario)?;
    write_wav(out, AUDIO_SAMPLE_RATE, &capture.samples)?;
    println!("wrote audio to {}", out.display());
    Ok(())
}

pub(crate) fn write_wav(out: &Path, sample_rate: u32, samples: &[f32]) -> Result<()> {
    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(out, spec)
        .with_context(|| format!("failed to create {}", out.display()))?;
    for sample in samples {
        writer
            .write_sample(sample.clamp(-1.0, 1.0))
            .with_context(|| format!("failed to write {}", out.display()))?;
    }
    writer
        .finalize()
        .with_context(|| format!("failed to finalise {}", out.display()))?;
    Ok(())
}

pub(crate) fn read_wav_f32(path: &Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let spec = reader.spec();
    if spec.channels != 2 {
        bail!(
            "expected stereo WAV from {}, got {}",
            path.display(),
            spec.channels
        );
    }
    match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("failed to read {}", path.display())),
        hound::SampleFormat::Int => {
            let scale = if spec.bits_per_sample <= 16 {
                f32::from(i16::MAX)
            } else {
                i32::MAX as f32
            };
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| format!("failed to read {}", path.display()))
        }
    }
}

pub(crate) fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::Digest;

    let mut hasher = sha1::Sha1::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

pub(crate) fn run_native_script_for_scenario(
    manifest: &AssetManifest,
    script: &[u8],
    scenario: Scenario,
) -> Vec<NativeTraceFrame> {
    run_script(manifest.clone(), script, scenario.frame_count())
}

pub(crate) fn run_native_script_for_gameplay(
    manifest: &AssetManifest,
    script: &[u8],
    scenario: GameplayScenario,
) -> Vec<NativeTraceFrame> {
    run_native_script_dynamic(manifest, script, scenario.frame_count())
}

pub(crate) fn run_native_script_for_gameplay_with_trace(
    manifest: &AssetManifest,
    script: &[u8],
    scenario: GameplayScenario,
) -> Vec<NativeTraceFrame> {
    run_native_script_dynamic_with_trace(manifest, script, scenario.frame_count())
}

pub(crate) fn run_native_script_dynamic(
    manifest: &AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    run_script(manifest.clone(), script, frame_count)
}

pub(crate) fn run_native_script_dynamic_with_trace(
    manifest: &AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    run_script_with_trace(manifest.clone(), script, frame_count, None)
}
