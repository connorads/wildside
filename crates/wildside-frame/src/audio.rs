//! Trace→GEMS audio direction shared by the native player and the browser
//! build.
//!
//! Turns per-frame [`NativeTraceFrame`]s into GEMS driver commands (BGM
//! start/stop cues, SFX start/stop sequences, music-speed updates) exactly as
//! the ROM does, so every frontend hears the same thing the parity gates
//! verify. The logic is sink-agnostic: [`NativeAudioSink`] abstracts the GEMS
//! engine, and only the `audio` cargo feature pulls in `wildside-md` for the
//! real [`GemsAudioEngine`](wildside_md::GemsAudioEngine) implementation.

use wildside_core::{GameMode, LevelId, NativeSoundCommand, NativeTraceFrame};
#[cfg(feature = "audio")]
use wildside_data::AssetSource;
use wildside_data::{AssetManifest, LevelHeader, resource_level_record};
#[cfg(feature = "audio")]
use wildside_md::GemsAudioEngine;

pub const AUDIO_SAMPLE_RATE: u32 = 44_100;
pub const AUDIO_FRAME_RATE_HZ: u32 = 60;
pub const AUDIO_SAMPLES_PER_FRAME: usize = 735;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSampleClock {
    sample_rate: u32,
    remainder: u32,
}

impl AudioSampleClock {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            remainder: 0,
        }
    }

    pub fn next_frame_samples(&mut self) -> usize {
        let total = self.remainder + self.sample_rate;
        let samples = total / AUDIO_FRAME_RATE_HZ;
        self.remainder = total % AUDIO_FRAME_RATE_HZ;
        usize::try_from(samples).unwrap_or(usize::MAX)
    }
}

/// GEMS sound IDs for Kid Chameleon (from constants.asm).
pub mod gems_sfx {
    // General
    pub const SCORE_COUNTER: u8 = 0x0C;
    pub const BLOCK_HIT: u8 = 0x0D;
    pub const CLOCK_PRIZE: u8 = 0x0E;
    pub const DESTROY_BLOCK: u8 = 0x11;
    pub const METAL_BLOCK: u8 = 0x12;
    pub const REVEAL_HIDDEN: u8 = 0x12;
    pub const EVANESCENT: u8 = 0x14;
    pub const SHIFTING: u8 = 0x15;
    pub const JUMP: u8 = 0x17;
    pub const LAVA_GEYSER: u8 = 0x1E;
    pub const TANK_DRIVING: u8 = 0x1F;
    pub const ELEVATOR: u8 = 0x27;
    pub const DIAMOND_POWER_AVAILABLE: u8 = 0x28;
    pub const ANKH_PRIZE: u8 = 0x29;
    pub const REPLENISH_HEALTH: u8 = 0x2A;
    pub const COIN_PRIZE: u8 = 0x2C;
    pub const DIAMOND_PRIZE: u8 = 0x2D;
    pub const NAVIGATE_JINGLE: u8 = 0x2D;
    pub const TANK_MOUTH_OPEN: u8 = 0x3C;
    pub const MINI_HOPPING_SKULL_SCREAM: u8 = 0x3F;
    pub const JUMP_ON_ENEMY: u8 = 0x4C;
    pub const VOICE_NO_TIME: u8 = 0x51;
    pub const TAR_MONSTER_DISAPPEARS: u8 = 0x53;
    pub const TAR_MONSTER_APPEARS: u8 = 0x54;
    pub const RUBBER_BLOCK: u8 = 0x56;
    pub const DRIPS_DRIPPING: u8 = 0x58;
    pub const TAR_MONSTER_SHOOTS: u8 = 0x5E;
    pub const VOICE_DIE: u8 = 0x60;
    pub const VOICE_OUCH: u8 = 0x61;
    pub const VOICE_DIAMOND: u8 = 0x62;
    pub const KID_PULLUP: u8 = 0x64;
    pub const VOICE_BUMMER: u8 = 0x68;
    pub const VOICE_OUCH_1: u8 = 0x6B;
    pub const VOICE_OUCH_2: u8 = 0x6C;
    pub const VOICE_OUCH_3: u8 = 0x6D;
    pub const TELEPORT: u8 = 0x76;
    pub const THUNDERSTORM: u8 = 0x77;
    // Helmet transforms
    pub const IRON_KNIGHT_TRANSFORM: u8 = 0x1A;
    pub const MANIAXE_TRANSFORM: u8 = 0x1B;
    pub const RED_STEALTH_TRANSFORM: u8 = 0x2E;
    pub const BERZERKER_TRANSFORM: u8 = 0x2F;
    pub const SKYCUTTER_TRANSFORM: u8 = 0x30;
    pub const EYECLOPS_TRANSFORM: u8 = 0x31;
    pub const CYCLONE_TRANSFORM: u8 = 0x32;
    pub const MICROMAX_TRANSFORM: u8 = 0x33;
    pub const JUGGERNAUT_TRANSFORM: u8 = 0x34;
    // Helmet abilities
    pub const MANIAXE_THROW: u8 = 0x20;
    pub const RED_STEALTH_ATTACK_DOWN: u8 = 0x13;
    pub const RED_STEALTH_SWORD: u8 = 0x23;
    pub const RED_STEALTH_HITTING_TARGET: u8 = 0x5A;
    pub const EYECLOPS_BEAM: u8 = 0x35;
    pub const SKYCUTTER_FLIPBOARD: u8 = 0x17;
    pub const MICROMAX_STICK: u8 = 0x22;
}

/// GEMS BGM IDs (from constants.asm).
pub mod gems_bgm {
    pub const MOUNTAIN: u8 = 0;
    pub const DESERT: u8 = 1;
    pub const SKY: u8 = 2;
    pub const ICE: u8 = 3;
    pub const HILLS: u8 = 4;
    pub const ENDING: u8 = 5;
    pub const FOREST: u8 = 6;
    pub const CAVE: u8 = 7;
    pub const ISLAND: u8 = 8;
    pub const MURDERWALL: u8 = 9;
    pub const SWAMP: u8 = 0x0A;
    pub const CITY: u8 = 0x65;
    pub const SCORE_BOARD: u8 = 0x74;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBgmCue {
    TitleShell,
    Options,
    Level(LevelId),
    CoreQueued(u8),
    ScoreBoard,
    Ending,
    StoppedForTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBgmPlaybackCommand {
    Start(u8),
    StopMusic,
    StopMusicThenStart(u8),
}

pub fn native_bgm_cue_for_trace(trace: &NativeTraceFrame) -> Option<NativeBgmCue> {
    match (trace.game_mode, trace.played_bgm_ids.last().copied()) {
        // ASM Score_Board loc_C238 (kid.asm:15812): core queues the exact
        // `bgm_Score_Board` command once the EOL path reaches the BGM frame.
        (GameMode::InGame | GameMode::DemoPlay, Some(gems_bgm::SCORE_BOARD))
            if trace.level_complete =>
        {
            return Some(NativeBgmCue::ScoreBoard);
        }
        // ASM End_Credits loc_D824 (kid.asm:16924): core queues
        // `bgm_Ending` after the `$28` StopMusic pause.
        (GameMode::EndingSequence, Some(gems_bgm::ENDING)) => {
            return Some(NativeBgmCue::Ending);
        }
        // ASM Load_InGame loc_11FD2/loc_12004 is now emitted by wildside-core into
        // the trace. Prefer that exact command over re-deriving level music
        // from manifest headers, while keeping the scoreboard override below.
        (GameMode::InGame | GameMode::DemoPlay, Some(bgm)) if !trace.level_complete => {
            return Some(NativeBgmCue::CoreQueued(bgm));
        }
        _ => {}
    }

    match trace.game_mode {
        // ASM Load_IntroSequence1 / Load_TitleScreen both queue `bgm_Ice` via
        // `j_PlaySound`; later intro/title-shell modes inherit that cue. Native
        // may enter those modes directly through manual Start/demo paths, so
        // keep them recoverable instead of requiring the cold first sequence.
        GameMode::IntroSequence1
        | GameMode::IntroSequence2
        | GameMode::IntroSequence3
        | GameMode::IntroSequence4
        | GameMode::IntroSequence5
        | GameMode::IntroSequence6
        | GameMode::TitleScreen => Some(NativeBgmCue::TitleShell),
        // ASM Load_OptionMenu (kid.asm:31570-31572) stops the active shell
        // music, then queues bgm_City before drawing the options screen.
        GameMode::OptionsMenu => Some(NativeBgmCue::Options),
        // ASM Score_Board (kid.asm:15838): after the flagpole transition
        // sleeps, the EOL sequence queues bgm_Score_Board before loc_D55E.
        GameMode::InGame | GameMode::DemoPlay if trace.level_complete => {
            Some(NativeBgmCue::ScoreBoard)
        }
        GameMode::InGame | GameMode::DemoPlay => Some(NativeBgmCue::Level(trace.current_level)),
        _ => None,
    }
}

pub fn native_bgm_state_after_trace(
    manifest: &AssetManifest,
    trace: &NativeTraceFrame,
    active_bgm_cue: Option<NativeBgmCue>,
) -> (Option<NativeBgmCue>, Option<NativeBgmPlaybackCommand>) {
    let Some(cue) = native_bgm_cue_for_trace(trace) else {
        if active_bgm_cue == Some(NativeBgmCue::StoppedForTransition) {
            return (active_bgm_cue, None);
        }
        if trace.game_mode == GameMode::SegaScreen && active_bgm_cue.is_some() {
            // ASM Load_SegaScreen (kid.asm:25444-25445) runs StopMusic before
            // loading the Sega logo, so the following Load_IntroSequence1
            // must be able to queue bgm_Ice again.
            return (None, Some(NativeBgmPlaybackCommand::StopMusic));
        }
        if matches!(
            active_bgm_cue,
            Some(
                NativeBgmCue::TitleShell
                    | NativeBgmCue::Level(_)
                    | NativeBgmCue::CoreQueued(_)
                    | NativeBgmCue::ScoreBoard
            )
        ) && trace.game_mode == GameMode::LevelTransition
        {
            // ASM loc_6E2 dispatches mode 8 through LoadGameModeData; the
            // subsequent Load_InGame path (kid.asm:23623-23629) resets GEMS
            // with StopMusic before loc_12004 queues the level BGM again.
            return (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic),
            );
        }
        if active_bgm_cue.is_none()
            && trace.game_mode == GameMode::LevelTransition
            && (trace.level_complete
                || trace.level_complete_elapsed_frames > 0
                || trace.level_complete_score_counter_drained)
        {
            // If the active cue tracker has already been cleared by an earlier
            // shell frame, the GEMS driver can still be releasing the final
            // loc_D7F4 score-counter SFX. StopMusic is harmless with no BGM and
            // mirrors the ROM hard reset before Load_InGame.
            return (
                Some(NativeBgmCue::StoppedForTransition),
                Some(NativeBgmPlaybackCommand::StopMusic),
            );
        }
        return (active_bgm_cue, None);
    };
    if Some(cue) == active_bgm_cue {
        return (active_bgm_cue, None);
    }
    let Some(bgm) = bgm_for_native_cue(manifest, cue) else {
        return (active_bgm_cue, None);
    };
    let command = if active_bgm_cue.is_none()
        && matches!(cue, NativeBgmCue::Level(_) | NativeBgmCue::CoreQueued(_))
    {
        // ASM Load_InGame (kid.asm:23623) runs StopMusic even when
        // native's cue tracker has no previous BGM. This hard reset is the
        // incoming-level off switch for stale looping GEMS sequences (#339).
        NativeBgmPlaybackCommand::StopMusicThenStart(bgm)
    } else if active_bgm_cue.is_some_and(|active| active != NativeBgmCue::StoppedForTransition)
        || (cue == NativeBgmCue::TitleShell && trace.game_mode == GameMode::IntroSequence1)
    {
        NativeBgmPlaybackCommand::StopMusicThenStart(bgm)
    } else {
        NativeBgmPlaybackCommand::Start(bgm)
    };
    (Some(cue), Some(command))
}

pub fn native_audio_events_for_trace(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
) -> Vec<String> {
    native_audio_events_for_trace_with_initial_bgm(manifest, trace_frames, None)
}

pub fn native_audio_events_for_trace_with_initial_bgm(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
    initial_active_bgm_cue: Option<NativeBgmCue>,
) -> Vec<String> {
    let mut active_bgm_cue = initial_active_bgm_cue;
    let mut events = Vec::new();
    let mut prev_trace: Option<&NativeTraceFrame> = None;
    for (frame_index, trace) in trace_frames.iter().enumerate() {
        let active_before = active_bgm_cue;
        let (next_bgm_cue, bgm_command) =
            native_bgm_state_after_trace(manifest, trace, active_bgm_cue);
        if let Some(command) = bgm_command {
            events.push(format_native_bgm_event(
                frame_index,
                trace,
                active_before,
                next_bgm_cue,
                command,
            ));
        }
        active_bgm_cue = next_bgm_cue;
        if let Some(prev) = prev_trace {
            let suppress_sfx_this_frame = bgm_command.is_some_and(|command| {
                native_bgm_command_suppresses_same_frame_sfx(command, trace, active_before)
            });
            if !suppress_sfx_this_frame {
                for command in gems_event_audio_commands(prev, trace) {
                    match command {
                        NativeAudioTraceCommand::Sound(command) => {
                            events.push(format_native_sound_event(frame_index, trace, command));
                        }
                        NativeAudioTraceCommand::MusicSpeed(speed) => {
                            events.push(format_native_music_speed_event(frame_index, trace, speed));
                        }
                    }
                }
            }
        }
        prev_trace = Some(trace);
    }
    events
}

pub fn render_native_audio_samples_for_trace(
    manifest: &AssetManifest,
    trace_frames: &[NativeTraceFrame],
    initial_active_bgm_cue: Option<NativeBgmCue>,
    audio: &mut impl NativeAudioSink,
) -> Vec<f32> {
    let mut director =
        NativeAudioDirector::with_initial_bgm(manifest, initial_active_bgm_cue, audio);
    let mut audio_samples = Vec::with_capacity(trace_frames.len() * AUDIO_SAMPLES_PER_FRAME * 2);
    for trace in trace_frames {
        director.step(manifest, trace, audio);
        audio_samples.extend(audio.generate_samples(AUDIO_SAMPLES_PER_FRAME));
    }

    audio_samples
}

/// Incremental per-frame driver of the trace→GEMS command logic: the loop body
/// of [`render_native_audio_samples_for_trace`] held as reusable state
/// (`active_bgm_cue`, the one-frame `pending_bgm_start` delay after a
/// StopMusic reset, and the previous trace for command deltas). The batch
/// renderer above and the browser session both drive this single body, so the
/// web audio path cannot drift from the parity-verified native one (#20).
#[derive(Debug, Clone, Default)]
pub struct NativeAudioDirector {
    active_bgm_cue: Option<NativeBgmCue>,
    pending_bgm_start: Option<u8>,
    prev_trace: Option<NativeTraceFrame>,
}

impl NativeAudioDirector {
    /// Cold start: no active BGM, first `step` derives everything from its trace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start with `initial_active_bgm_cue` already active. Settled dynamic
    /// audio probes start after the ROM has already loaded level music:
    /// suppress duplicate event-log commands, but prime the sink so
    /// block/pickup SFX are mixed over the same active BGM instead of silence
    /// until the first SFX.
    pub fn with_initial_bgm(
        manifest: &AssetManifest,
        initial_active_bgm_cue: Option<NativeBgmCue>,
        audio: &mut impl NativeAudioSink,
    ) -> Self {
        if let Some(bgm) = settled_audio_initial_bgm_id(manifest, initial_active_bgm_cue) {
            audio.start_sequence(bgm);
        }
        Self {
            active_bgm_cue: initial_active_bgm_cue,
            ..Self::default()
        }
    }

    /// Apply one traced frame's BGM/SFX commands to `audio`, then advance the
    /// GEMS driver one 60Hz tick (`step_frame`). The caller pulls however many
    /// samples its output rate needs afterwards ([`AUDIO_SAMPLES_PER_FRAME`]
    /// at 44.1kHz, or an [`AudioSampleClock`] share at the device rate).
    pub fn step(
        &mut self,
        manifest: &AssetManifest,
        trace: &NativeTraceFrame,
        audio: &mut impl NativeAudioSink,
    ) {
        let (next_bgm_cue, bgm_command) =
            native_bgm_state_after_trace(manifest, trace, self.active_bgm_cue);
        let mut queued_bgm_start_this_frame = false;
        let mut suppress_sfx_this_frame = false;
        if let Some(command) = bgm_command {
            self.pending_bgm_start = apply_native_bgm_command(audio, command);
            queued_bgm_start_this_frame = self.pending_bgm_start.is_some();
            suppress_sfx_this_frame =
                native_bgm_command_suppresses_same_frame_sfx(command, trace, self.active_bgm_cue);
        }
        self.active_bgm_cue = next_bgm_cue;

        if !queued_bgm_start_this_frame && let Some(bgm) = self.pending_bgm_start.take() {
            audio.start_sequence(bgm);
        }
        if !suppress_sfx_this_frame && let Some(prev) = self.prev_trace.as_ref() {
            for command in gems_event_audio_commands(prev, trace) {
                match command {
                    NativeAudioTraceCommand::Sound(command) => {
                        apply_native_sound_command(audio, command);
                    }
                    NativeAudioTraceCommand::MusicSpeed(speed) => {
                        audio.set_music_speed(speed);
                    }
                }
            }
        }
        audio.step_frame();
        self.prev_trace = Some(trace.clone());
    }
}

pub fn settled_audio_initial_bgm_id(
    manifest: &AssetManifest,
    initial_active_bgm_cue: Option<NativeBgmCue>,
) -> Option<u8> {
    initial_active_bgm_cue.and_then(|cue| bgm_for_native_cue(manifest, cue))
}

pub trait NativeAudioSink {
    fn start_sequence(&mut self, sound_id: u8);
    fn stop_sequence(&mut self, sound_id: u8);
    fn stop_music(&mut self);
    fn set_music_speed(&mut self, speed: u8);
    fn step_frame(&mut self);
    fn generate_samples(&mut self, sample_count: usize) -> Vec<f32>;
}

#[cfg(feature = "audio")]
impl NativeAudioSink for GemsAudioEngine {
    fn start_sequence(&mut self, sound_id: u8) {
        GemsAudioEngine::start_sequence(self, sound_id);
    }

    fn stop_sequence(&mut self, sound_id: u8) {
        GemsAudioEngine::stop_sequence(self, sound_id);
    }

    fn stop_music(&mut self) {
        GemsAudioEngine::stop_music(self);
    }

    fn set_music_speed(&mut self, speed: u8) {
        GemsAudioEngine::set_music_speed(self, speed);
    }

    fn step_frame(&mut self) {
        GemsAudioEngine::step_frame(self);
    }

    fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
        GemsAudioEngine::generate_samples(self, sample_count)
    }
}

/// Load and initialise a [`GemsAudioEngine`] from a generated bundle's
/// `sound/` directory. `pack_bundle_blob` packs the whole bundle recursively,
/// so both the native fs bundle and the fetched web blob resolve the same
/// five extracted GEMS banks.
#[cfg(feature = "audio")]
pub fn load_gems_audio_engine(source: &dyn AssetSource) -> anyhow::Result<GemsAudioEngine> {
    let read = |name: &str| source.read(&std::path::Path::new("sound").join(name));
    let driver = read("GEMS_Sounddriver.bin")?;
    let patches = read("instruments.bin")?;
    let envelopes = read("envelopes.bin")?;
    let sequences = read("sequences.bin")?;
    let samples = read("samples.bin")?;

    let mut engine = GemsAudioEngine::new(&driver, &patches, &envelopes, &sequences, &samples);
    engine.init();
    Ok(engine)
}

pub fn apply_native_bgm_command(
    audio: &mut impl NativeAudioSink,
    command: NativeBgmPlaybackCommand,
) -> Option<u8> {
    match command {
        NativeBgmPlaybackCommand::Start(bgm) => {
            audio.start_sequence(bgm);
            None
        }
        NativeBgmPlaybackCommand::StopMusic => {
            audio.stop_music();
            None
        }
        NativeBgmPlaybackCommand::StopMusicThenStart(bgm) => {
            // ASM StopMusic (kid.asm:49727) reloads/resets the GEMS driver
            // and queues command `$0B` with the sound data pointers. Native
            // playback must use that ROM path before changing music contexts;
            // `$16` stop-all leaves the standalone driver with stale title
            // sequence state and can starve gameplay SFX channels.
            audio.stop_music();
            Some(bgm)
        }
    }
}

pub fn native_bgm_command_flushes_queued_audio(command: NativeBgmPlaybackCommand) -> bool {
    // ASM StopMusic (kid.asm:49727) resets the Z80/GEMS driver immediately.
    // play-native queues audio ahead of playback through SDL, so mirror that
    // hard reset by dropping already-generated samples from the host queue.
    matches!(
        command,
        NativeBgmPlaybackCommand::StopMusic | NativeBgmPlaybackCommand::StopMusicThenStart(_)
    )
}

pub fn native_bgm_command_suppresses_same_frame_sfx(
    command: NativeBgmPlaybackCommand,
    trace: &NativeTraceFrame,
    active_before: Option<NativeBgmCue>,
) -> bool {
    if !native_bgm_command_flushes_queued_audio(command) {
        return false;
    }
    // ASM loc_D7F4 (kid.asm:16906) replays sfx_Score_counter before Teleport,
    // then Load_InGame (kid.asm:23623-23629) runs StopMusic before the next
    // level's BGM. Native condenses that handoff into one LevelTransition trace
    // frame, so any same-frame SFX must be treated as pre-reset driver state.
    if trace.game_mode == GameMode::LevelTransition {
        return true;
    }
    // ASM Load_InGame runs j_StopMusic before loc_11FD2/loc_12004
    // queues level music. A cold native cue means this frame is also the first
    // visible level-load boundary, so stale enemy SFX deltas from the previous
    // route must not be replayed after the reset.
    matches!(command, NativeBgmPlaybackCommand::StopMusicThenStart(_))
        && active_before.is_none()
        && matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
}

pub fn apply_native_sound_command(audio: &mut impl NativeAudioSink, command: NativeSoundCommand) {
    match command.command_byte {
        NativeSoundCommand::STOP_SEQUENCE_COMMAND_BYTE => audio.stop_sequence(command.sound_id),
        _ => audio.start_sequence(command.sound_id),
    }
}

pub fn format_native_bgm_event(
    frame_index: usize,
    trace: &NativeTraceFrame,
    active_before: Option<NativeBgmCue>,
    active_after: Option<NativeBgmCue>,
    command: NativeBgmPlaybackCommand,
) -> String {
    format!(
        "frame={frame_index} mode={:?} level={:02X} active_before={active_before:?} active_after={active_after:?} command={command:?}",
        trace.game_mode, trace.current_level.0
    )
}

pub fn format_native_sound_event(
    frame_index: usize,
    trace: &NativeTraceFrame,
    command: NativeSoundCommand,
) -> String {
    let command_name = command.command_name();
    let sound_id = command.sound_id;
    format!(
        "frame={frame_index} mode={:?} level={:02X} command={command_name}({sound_id})",
        trace.game_mode, trace.current_level.0
    )
}

pub fn format_native_music_speed_event(
    frame_index: usize,
    trace: &NativeTraceFrame,
    speed: u8,
) -> String {
    format!(
        "frame={frame_index} mode={:?} level={:02X} command=SetMusicSpeed({speed})",
        trace.game_mode, trace.current_level.0
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeAudioTraceCommand {
    Sound(NativeSoundCommand),
    MusicSpeed(u8),
}

pub fn gems_event_audio_commands(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
) -> Vec<NativeAudioTraceCommand> {
    let sound_commands = gems_event_sound_commands(prev, curr);
    let music_speed_updates = gems_event_music_speed_updates(prev, curr);
    let (voice_bummer_commands, pre_speed_sound_commands): (Vec<_>, Vec<_>) = sound_commands
        .into_iter()
        .partition(|command| command.sound_id == gems_sfx::VOICE_BUMMER);

    pre_speed_sound_commands
        .into_iter()
        .map(NativeAudioTraceCommand::Sound)
        .chain(
            music_speed_updates
                .into_iter()
                .map(NativeAudioTraceCommand::MusicSpeed),
        )
        .chain(
            voice_bummer_commands
                .into_iter()
                .map(NativeAudioTraceCommand::Sound),
        )
        .collect()
}

pub fn bgm_for_native_cue(manifest: &AssetManifest, cue: NativeBgmCue) -> Option<u8> {
    match cue {
        // ASM Load_TitleScreen queues bgm_Ice through j_PlaySound before loading title art.
        NativeBgmCue::TitleShell => Some(gems_bgm::ICE),
        NativeBgmCue::Options => Some(gems_bgm::CITY),
        NativeBgmCue::Level(level) => {
            resolved_level_header(manifest, level.0).and_then(level_start_bgm)
        }
        NativeBgmCue::CoreQueued(bgm) => Some(bgm),
        NativeBgmCue::ScoreBoard => Some(gems_bgm::SCORE_BOARD),
        NativeBgmCue::Ending => Some(gems_bgm::ENDING),
        NativeBgmCue::StoppedForTransition => None,
    }
}

pub fn queued_gems_sound_ids_since<'a>(
    prev: &NativeTraceFrame,
    curr: &'a NativeTraceFrame,
) -> &'a [u8] {
    if curr.played_sound_effect_ids.len() >= prev.played_sound_effect_ids.len() {
        &curr.played_sound_effect_ids[prev.played_sound_effect_ids.len()..]
    } else {
        &curr.played_sound_effect_ids
    }
}

pub fn queued_gems_sound_commands_since(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
) -> Vec<NativeSoundCommand> {
    if curr.played_sound_commands.is_empty() {
        return queued_gems_sound_ids_since(prev, curr)
            .iter()
            .copied()
            .map(NativeSoundCommand::start_sequence)
            .collect();
    }

    if curr.played_sound_commands.len() >= prev.played_sound_commands.len() {
        curr.played_sound_commands[prev.played_sound_commands.len()..].to_vec()
    } else {
        curr.played_sound_commands.clone()
    }
}

pub fn queued_gems_music_speed_updates_since<'a>(
    prev: &NativeTraceFrame,
    curr: &'a NativeTraceFrame,
) -> &'a [u8] {
    if curr.played_music_speed_updates.len() >= prev.played_music_speed_updates.len() {
        &curr.played_music_speed_updates[prev.played_music_speed_updates.len()..]
    } else {
        &curr.played_music_speed_updates
    }
}

pub fn gems_event_music_speed_updates(prev: &NativeTraceFrame, curr: &NativeTraceFrame) -> Vec<u8> {
    queued_gems_music_speed_updates_since(prev, curr).to_vec()
}

pub fn is_ouch_sound_id(sound_id: u8) -> bool {
    matches!(
        sound_id,
        gems_sfx::VOICE_OUCH
            | gems_sfx::VOICE_OUCH_1
            | gems_sfx::VOICE_OUCH_2
            | gems_sfx::VOICE_OUCH_3
    )
}

pub fn is_damage_resolution_voice_sound_id(sound_id: u8) -> bool {
    // ASM loc_B73C/Death (kid.asm:14661,14714): once the core trace carries
    // either the selected ouch take or the direct death voice, the legacy
    // HP-drop fallback must stay silent.
    is_ouch_sound_id(sound_id) || sound_id == gems_sfx::VOICE_DIE
}

pub fn is_explicit_block_sound_id(sound_id: u8) -> bool {
    matches!(
        sound_id,
        gems_sfx::DESTROY_BLOCK | gems_sfx::METAL_BLOCK | gems_sfx::RUBBER_BLOCK
    )
}

pub fn is_transform_sound_id(sound_id: u8) -> bool {
    matches!(
        sound_id,
        gems_sfx::IRON_KNIGHT_TRANSFORM
            | gems_sfx::MANIAXE_TRANSFORM
            | gems_sfx::RED_STEALTH_TRANSFORM
            | gems_sfx::BERZERKER_TRANSFORM
            | gems_sfx::SKYCUTTER_TRANSFORM
            | gems_sfx::EYECLOPS_TRANSFORM
            | gems_sfx::CYCLONE_TRANSFORM
            | gems_sfx::MICROMAX_TRANSFORM
            | gems_sfx::JUGGERNAUT_TRANSFORM
    )
}

pub fn transform_sound_for_helmet_id_loc_7cd0(helmet_id: u16) -> Option<u8> {
    // ASM loc_7CD0 (kid.asm:8426): `unk_7ED8[Current_Helmet]` maps helmet ids
    // to transform SFX. Keep this fallback for older traces; current core
    // frames queue the same id explicitly.
    match helmet_id {
        1 => Some(gems_sfx::SKYCUTTER_TRANSFORM),
        2 => Some(gems_sfx::CYCLONE_TRANSFORM),
        3 => Some(gems_sfx::RED_STEALTH_TRANSFORM),
        4 => Some(gems_sfx::EYECLOPS_TRANSFORM),
        5 => Some(gems_sfx::JUGGERNAUT_TRANSFORM),
        6 => Some(gems_sfx::IRON_KNIGHT_TRANSFORM),
        7 => Some(gems_sfx::BERZERKER_TRANSFORM),
        8 => Some(gems_sfx::MANIAXE_TRANSFORM),
        9 => Some(gems_sfx::MICROMAX_TRANSFORM),
        _ => None,
    }
}

pub fn gems_event_sound_commands(
    prev: &NativeTraceFrame,
    curr: &NativeTraceFrame,
) -> Vec<NativeSoundCommand> {
    let queued_sound_ids = queued_gems_sound_ids_since(prev, curr);
    let mut sound_commands = queued_gems_sound_commands_since(prev, curr);
    let gameplay_sfx_fallback_active =
        matches!(curr.game_mode, GameMode::InGame | GameMode::DemoPlay);

    // Sound test: play the selected track when A/C pressed
    if curr.sound_test_play_pending
        && let Some(track) = curr.sound_test_track
    {
        sound_commands.push(NativeSoundCommand::start_sequence(track));
    }

    // Jump
    if gameplay_sfx_fallback_active
        && !queued_sound_ids.contains(&gems_sfx::JUMP)
        && prev.kid_on_ground == Some(true)
        && curr.kid_on_ground == Some(false)
        && curr.kid_y_velocity.is_some_and(|v| v < 0)
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::JUMP));
    }

    // Stomp enemy is now driven by the core trace. ASM Jump_On_Enemy
    // (kid.asm:14607) plays `sfx_Jump_on_enemy` before any enemy HP/score
    // consequence, and many real stomps do not award score.

    // Diamond collected: the cue is now driven authoritatively by the core
    // trace, not by the visible count. ASM PrizeDiamondCollected_Init
    // (kid.asm:21103) plays `sfx_Diamond_prize` ($2D) on the *contact* frame,
    // which the core queues via `queue_prize_diamond_award`. The visible
    // `diamond_count` only rises ~33 frames later, after the fly-to-counter
    // loop (kid.asm:21149-21162), so the previous `diamond_count > prev`
    // heuristic emitted the cue that late and is what #216 heard as the diamond
    // sound lagging. The contact cue arrives through `queued_sound_ids` above,
    // so no count-based fallback is needed.

    // Damage taken (HP decreased). ASM loc_B73C/loc_B750/loc_B764/loc_B778
    // can queue enemy-family-specific ouch takes; keep the heuristic only for
    // paths that have not yet promoted their exact core SFX into the trace.
    if !queued_sound_ids
        .iter()
        .copied()
        .any(is_damage_resolution_voice_sound_id)
        && curr.number_hitpoints < prev.number_hitpoints
        && prev.number_hitpoints > 0
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::VOICE_OUCH));
    }

    // Death (lives decreased)
    if curr.number_lives < prev.number_lives {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::VOICE_DIE));
    }

    // Form change (helmet ID changed). ASM loc_7CD0 skips the unk_7ED8 cue in
    // Demo_Mode_flag, otherwise indexes this table by Current_Helmet whose
    // order differs from the extracted art order.
    if curr.game_mode != GameMode::DemoPlay
        && curr.current_helmet_id != prev.current_helmet_id
        && let Some(id) = transform_sound_for_helmet_id_loc_7cd0(curr.current_helmet_id)
        && !queued_sound_ids.contains(&id)
    {
        sound_commands.push(NativeSoundCommand::start_sequence(id));
    }

    // Diamond Power activation
    if curr.diamond_power_active && !prev.diamond_power_active {
        sound_commands.push(NativeSoundCommand::start_sequence(
            gems_sfx::DIAMOND_POWER_AVAILABLE,
        ));
    }

    // ASM loc_D752/loc_D770: Score_Board BGM and EOL rows appear before the
    // `sfx_Score_counter` cue; only start it when the ROM counter drain begins.
    if curr.level_complete
        && curr.level_complete_score_counter_elapsed_frames > 0
        && prev.level_complete_score_counter_elapsed_frames == 0
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::SCORE_COUNTER));
    }

    // ASM loc_D7F4 (kid.asm:16906-16908): when the counter finishes draining,
    // the ROM issues GEMS `$12` (cmdstopseq, the call labelled `j_PlaySound2`
    // in the disassembly) to *stop* sequence `$0C` (`sfx_Score_counter`) before
    // hibernating and routing to Teleport/End_Credits. loc_D752 starts the tick
    // with `$10` (cmdstartseq); loc_D7F4 stops it with `$12`, cleanly silencing
    // the tick once the tally lands instead of leaving it ringing (#216
    // "level-complete sound does not finish cleanly").
    if curr.level_complete
        && curr.level_complete_score_counter_drained
        && !prev.level_complete_score_counter_drained
    {
        sound_commands.push(NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER));
    }

    // Block headbutt. ASM loc_10DA4 queues rubber blocks as
    // `sfx_Rubber_block` and loc_110D0 queues metal/springy blocks as
    // `sfx_Metal_block`; when an exact block cue is present for this frame,
    // do not layer the generic `sfx_Block_hit` brick crunch over it.
    if curr.blocks_hit_this_frame > 0
        && !queued_sound_ids
            .iter()
            .copied()
            .any(is_explicit_block_sound_id)
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::BLOCK_HIT));
    }

    // Block destroyed (new destroy effects appeared). ASM ice blocks queue
    // sfx_Prize_block=$11 from the core trace on the same frame as sub_10F9C's
    // destroy effect, so do not replay the historical delta heuristic too.
    if !queued_sound_ids.contains(&gems_sfx::DESTROY_BLOCK)
        && curr.block_destroy_effects.len() > prev.block_destroy_effects.len()
    {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::DESTROY_BLOCK));
    }

    // Lava geyser erupted (new geysers appeared)
    if curr.active_geysers.len() > prev.active_geysers.len() {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_sfx::LAVA_GEYSER));
    }

    // Game complete — ending music
    if curr.game_complete && !prev.game_complete {
        sound_commands.push(NativeSoundCommand::start_sequence(gems_bgm::ENDING));
    }

    // #342: the clock-prize loop is no longer inferred from a timer jump here.
    // ASM PrizeClock_Init loc_10792 starts sfx_Clock_prize when the falling
    // clock object spawns and loc_107E0 stops it when that object is deleted;
    // the core now queues both Start/StopSequence commands, which arrive via
    // `queued_gems_sound_commands_since`. A timer-jump start fired at collection
    // would re-trigger the loop after the core's stop, re-leaking it.

    sound_commands
}

/// Map level theme ID to GEMS BGM ID.
pub fn level_start_bgm(header: &LevelHeader) -> Option<u8> {
    // ASM loc_11FD2 (kid.asm:24183-24186): after Load_InGame's shared setup,
    // non-zero MurderWall_flag bypasses loc_12004's foreground-theme table and
    // queues bgm_Murderwall directly.
    if header.murder_wall_flags != 0 {
        Some(gems_bgm::MURDERWALL)
    } else {
        theme_to_bgm(header.level_theme)
    }
}

pub fn theme_to_bgm(theme: u8) -> Option<u8> {
    // ASM loc_12004 indexes `unk_1201E` by Foreground_theme, then queues the
    // resulting byte through j_PlaySound. These are sound IDs, not theme indexes.
    match theme {
        0 => Some(gems_bgm::MOUNTAIN),
        1 => Some(gems_bgm::SKY),
        2 => Some(gems_bgm::ICE),
        3 => Some(gems_bgm::HILLS),
        4 => Some(gems_bgm::ISLAND),
        5 => Some(gems_bgm::DESERT),
        6 => Some(gems_bgm::SWAMP),
        7 => Some(gems_bgm::MOUNTAIN),
        8 => Some(gems_bgm::CAVE),
        9 => Some(gems_bgm::FOREST),
        10 => Some(gems_bgm::CITY),
        _ => None,
    }
}

pub fn resolved_level_header(manifest: &AssetManifest, level_id: u8) -> Option<&LevelHeader> {
    resource_level_record(manifest, level_id)
        .and_then(|level| level.header_metadata.as_ref())
        .or_else(|| {
            manifest
                .levels
                .iter()
                .find(|level| level.level_id == level_id)
                .and_then(|level| level.header_metadata.as_ref())
        })
}
