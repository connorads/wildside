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
use wildside_core::{
    GemsQueueCommandAdvance, GemsQueueCommandPath, GemsStopMusicBusAckPolls,
    GemsStopMusicElapsedTiming, HudDmaEventSub44DC, LoadInGameTimerBonusPath, M68kCycleCount,
    M68kRefreshPhase,
    vdp::{
        DecompressToVramTransferAdvance, LoadInGameCpuRasterCursor,
        LoadInGamePlatformInitialisationAdvance, LoadInGamePlatformRefreshLedger,
        LoadInGamePlatformVintEvent, ModeLevelCpuRasterCursor, TitleIntroSourceAdvance,
        TitleWaitForVintAdvance,
    },
};
#[cfg(feature = "audio")]
use wildside_data::AssetSource;
use wildside_data::{AssetManifest, LevelHeader, resource_level_record};
#[cfg(feature = "audio")]
use wildside_md::{
    GemsAudioEngine, GemsMasterClockAdvance, GemsMasterClockTimingEngine, GemsNtscTimelineOrigin,
    GemsQueueCommand, GemsQueueCommandControl, GemsStopMusicControl, GemsStopMusicQueueTransaction,
};

pub const AUDIO_SAMPLE_RATE: u32 = 44_100;
pub const AUDIO_FRAME_RATE_HZ: u32 = 60;
pub const AUDIO_SAMPLES_PER_FRAME: usize = 735;

// ASM sub_1C246 -> loc_1C264 -> loc_6E2 blocks inside the title rebuild.
// Published-route selected PCs place StopMusic $E14C8 at title elapsed 0 and
// Load_TitleScreen's PlaySound/sub_E151C at elapsed 32.
const TITLE_SCREEN_PLAY_SOUND_ELAPSED_SUB_E151C: usize = 32;

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
    native_bgm_cue_for_trace_with_event(trace, trace.played_bgm_ids.last().copied())
}

fn native_bgm_cue_for_trace_with_event(
    trace: &NativeTraceFrame,
    queued_bgm: Option<u8>,
) -> Option<NativeBgmCue> {
    match (trace.game_mode, queued_bgm) {
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
    native_bgm_state_after_trace_delta(manifest, None, trace, active_bgm_cue)
}

pub fn native_bgm_state_after_trace_delta(
    manifest: &AssetManifest,
    previous_trace: Option<&NativeTraceFrame>,
    trace: &NativeTraceFrame,
    active_bgm_cue: Option<NativeBgmCue>,
) -> (Option<NativeBgmCue>, Option<NativeBgmPlaybackCommand>) {
    let queued_bgm = previous_trace.map_or_else(
        || trace.played_bgm_ids.last().copied(),
        |previous| {
            if trace.played_bgm_ids.len() >= previous.played_bgm_ids.len() {
                trace.played_bgm_ids[previous.played_bgm_ids.len()..]
                    .last()
                    .copied()
            } else {
                trace.played_bgm_ids.last().copied()
            }
        },
    );
    let level_changed_during_loader = previous_trace.is_some_and(|previous| {
        matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
            && previous.current_level != trace.current_level
    });
    let title_screen_elapsed = (trace.game_mode == GameMode::TitleScreen
        && trace.boot_phase.as_deref() == Some("title_screen"))
    .then_some(trace.boot_phase_elapsed)
    .flatten();
    if title_screen_elapsed == Some(0)
        && previous_trace.is_some_and(|previous| previous.game_mode != GameMode::TitleScreen)
    {
        // ASM sub_1C246 -> loc_1C264 -> loc_6E2 reaches StopMusic before
        // Load_TitleScreen spends 32 outer frames rebuilding title VDP state.
        return (
            Some(NativeBgmCue::StoppedForTransition),
            Some(NativeBgmPlaybackCommand::StopMusic),
        );
    }
    if active_bgm_cue == Some(NativeBgmCue::StoppedForTransition)
        && title_screen_elapsed
            .is_some_and(|elapsed| elapsed < TITLE_SCREEN_PLAY_SOUND_ELAPSED_SUB_E151C)
    {
        return (active_bgm_cue, None);
    }
    if queued_bgm.is_none() && level_changed_during_loader {
        if active_bgm_cue == Some(NativeBgmCue::StoppedForTransition) {
            return (active_bgm_cue, None);
        }
        // ASM Load_DemoPlay -> Load_InGame (kid.asm:23623-23629) changes
        // Current_LevelID at the loader boundary which reaches StopMusic,
        // before the later loc_11FD2/loc_12004 PlaySound call.
        return (
            Some(NativeBgmCue::StoppedForTransition),
            Some(NativeBgmPlaybackCommand::StopMusic),
        );
    }
    if queued_bgm.is_none()
        && active_bgm_cue == Some(NativeBgmCue::StoppedForTransition)
        && matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
    {
        return (active_bgm_cue, None);
    }
    if queued_bgm.is_none()
        && previous_trace.is_some()
        && matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
    {
        // Keep the cue selected by the most recent explicit PlaySound event.
        // Falling back to `Level(level_id)` here would make the next frame
        // appear to change cue type and reset GEMS a second time.
        return (active_bgm_cue, None);
    }
    let Some(cue) = native_bgm_cue_for_trace_with_event(trace, queued_bgm) else {
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
    if queued_bgm.is_none() {
        match (active_bgm_cue, cue) {
            (Some(NativeBgmCue::Level(active_level)), NativeBgmCue::Level(next_level))
                if active_level != next_level =>
            {
                // ASM Load_InGame (kid.asm:23623-23629) runs StopMusic before
                // level loading, while loc_12004 queues the new BGM only after
                // the loading-card wait. A level-ID change is therefore a
                // reset boundary, never evidence that PlaySound already ran.
                return (
                    Some(NativeBgmCue::StoppedForTransition),
                    Some(NativeBgmPlaybackCommand::StopMusic),
                );
            }
            (Some(NativeBgmCue::StoppedForTransition), NativeBgmCue::Level(_)) => {
                // Wait for the explicit `played_bgm_ids` event emitted by
                // loc_12004 instead of replaying the inferred level theme.
                return (active_bgm_cue, None);
            }
            _ => {}
        }
    }
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
            native_bgm_state_after_trace_delta(manifest, prev_trace, trace, active_bgm_cue);
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
        if director.step(manifest, trace, audio) {
            audio_samples.extend(audio.generate_samples(AUDIO_SAMPLES_PER_FRAME));
        } else {
            audio_samples.extend(std::iter::repeat_n(0.0, AUDIO_SAMPLES_PER_FRAME * 2));
        }
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
    ) -> bool {
        let (next_bgm_cue, bgm_command) = native_bgm_state_after_trace_delta(
            manifest,
            self.prev_trace.as_ref(),
            trace,
            self.active_bgm_cue,
        );
        let mut queued_bgm_start_this_frame = false;
        let mut suppress_sfx_this_frame = false;
        if let Some(command) = bgm_command {
            self.pending_bgm_start = apply_native_bgm_command(audio, command);
            queued_bgm_start_this_frame = self.pending_bgm_start.is_some();
            suppress_sfx_this_frame =
                native_bgm_command_suppresses_same_frame_sfx(command, trace, self.active_bgm_cue);
        }
        self.active_bgm_cue = next_bgm_cue;

        let trace_audio_commands = if suppress_sfx_this_frame {
            Vec::new()
        } else {
            self.prev_trace
                .as_ref()
                .map(|prev| gems_event_audio_commands(prev, trace))
                .unwrap_or_default()
        };
        if !native_audio_frame_needs_driver_step(
            self.active_bgm_cue,
            self.pending_bgm_start,
            bgm_command,
            trace_audio_commands.len(),
        ) {
            if audio.requires_continuous_clock() {
                audio.step_frame();
                self.prev_trace = Some(trace.clone());
                return true;
            }
            self.prev_trace = Some(trace.clone());
            return false;
        }

        if !queued_bgm_start_this_frame && let Some(bgm) = self.pending_bgm_start.take() {
            audio.start_sequence(bgm);
        }
        for command in trace_audio_commands {
            match command {
                NativeAudioTraceCommand::Sound(command) => {
                    apply_native_sound_command(audio, command);
                }
                NativeAudioTraceCommand::MusicSpeed(speed) => {
                    audio.set_music_speed(speed);
                }
            }
        }
        audio.step_frame();
        self.prev_trace = Some(trace.clone());
        true
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

    fn requires_continuous_clock(&self) -> bool {
        false
    }
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

#[cfg(feature = "audio")]
impl NativeAudioSink for GemsMasterClockTimingEngine {
    fn start_sequence(&mut self, sound_id: u8) {
        GemsMasterClockTimingEngine::start_sequence(self, sound_id);
    }

    fn stop_sequence(&mut self, sound_id: u8) {
        GemsMasterClockTimingEngine::stop_sequence(self, sound_id);
    }

    fn stop_music(&mut self) {
        GemsMasterClockTimingEngine::stop_music(self);
    }

    fn set_music_speed(&mut self, speed: u8) {
        GemsMasterClockTimingEngine::set_music_speed(self, speed);
    }

    fn step_frame(&mut self) {
        GemsMasterClockTimingEngine::advance_ntsc_frame(self);
    }

    fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
        vec![0.0; sample_count * 2]
    }

    fn requires_continuous_clock(&self) -> bool {
        true
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

/// Advance the final pre-`StopMusic` `Load_InGame` source instructions.
///
/// The timing engine must already be positioned at `Load_InGame $11A12`.
/// Each Z80 bank access is drained as a 68000 wait before the next source
/// instruction, preserving the hardware ordering through `StopMusic $E14C8`.
#[cfg(feature = "audio")]
pub fn apply_gems_load_in_game_stop_music_tail(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    timer_path: LoadInGameTimerBonusPath,
) -> Option<LoadInGameCpuRasterCursor> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    let advance = wildside_core::load_in_game_stop_music_tail_advance_with_scheduler(
        entry.refresh_phase,
        timer_path,
        |cycles| {
            audio.advance_ntsc_master_clocks(u64::from(cycles.get()) * 7);
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )?;
    Some(LoadInGameCpuRasterCursor {
        beam: entry.beam.advance_m68k_cycles(advance.elapsed),
        refresh_phase: advance.phase_after,
    })
}

/// Clock persistent GEMS through `Initialize_Platforms sub_231C`.
///
/// Decoder/source spans and the exact `loc_14314` FIFO timeline share the
/// CPU/raster schedule. The adapter rejects an unowned interval which
/// generates a 68000 bank wait; those paths require a decoder instruction
/// visitor before their next source instruction can be positioned safely.
#[cfg(feature = "audio")]
pub fn apply_gems_load_in_game_platform_initialisation(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    vint: Option<LoadInGamePlatformVintEvent>,
    refresh: LoadInGamePlatformRefreshLedger,
) -> Option<LoadInGamePlatformInitialisationAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::load_in_game_platform_initialisation_cursor_from_entry_with_hardware(
        entry,
        vint,
        refresh,
        |cycles, vdp_owns_bus| {
            let advance = audio.advance_ntsc_master_clocks_with_vdp_ownership(
                u64::from(cycles.get()) * 7,
                vdp_owns_bus,
            );
            (advance.pending_m68k_wait_cycles == 0).then_some(())
        },
    )
}

/// Clock one generic ASM `DecompressToVRAM loc_14314` FIFO transfer.
///
/// Core owns the source instructions, refresh, optional instruction-boundary
/// VINT and VDP FIFO. Persistent GEMS returns each newly generated external
/// 68000 wait to the scheduler while banked reads that collide with VDP
/// ownership halt the Z80 instead.
#[cfg(feature = "audio")]
pub fn apply_gems_decompress_to_vram_transfer_loc_14314(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    output_words: u16,
    vint_count: u8,
    exit: wildside_core::vdp::DecompressToVramTransferExit,
) -> Option<DecompressToVramTransferAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::decompress_to_vram_transfer_cursor_loc_14314_with_scheduler(
        entry,
        output_words,
        vint_count,
        exit,
        |cycles, vdp_owns_bus| {
            audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                u64::from(cycles.get()) * 7,
                vdp_owns_bus,
            );
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock ASM `DecompressToVRAM loc_14300` through its first transfer entry,
/// servicing the requested recurring VINT count.
#[cfg(feature = "audio")]
pub fn apply_gems_decompress_to_vram_decoder_loc_14300(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    compressed: &[u8],
    resume: Option<wildside_core::saxman_timing::DecompressChunkResumeSub13684>,
    vint_count: u8,
) -> Option<wildside_core::vdp::DecompressToVramDecoderAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::decompress_to_vram_decoder_cursor_loc_14300_with_scheduler(
        entry,
        compressed,
        resume,
        vint_count,
        |cycles, vdp_owns_bus, source_instruction| {
            if source_instruction {
                audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            } else {
                audio.advance_ntsc_master_clocks_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            }
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock one source-ordered ASM `EniDec` call on the persistent GEMS timeline.
#[cfg(feature = "audio")]
pub fn apply_gems_enigma_decoder(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    caller_instruction_cycles: &[u32],
    compressed: &[u8],
    base_tile_word: u16,
    source_address: u32,
    vint_count: u8,
) -> Option<wildside_core::vdp::EnigmaDecoderAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::enigma_decoder_cursor_with_scheduler(
        entry,
        caller_instruction_cycles,
        compressed,
        base_tile_word,
        source_address,
        vint_count,
        |cycles, vdp_owns_bus, source_instruction| {
            if source_instruction {
                audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            } else {
                audio.advance_ntsc_master_clocks_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            }
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock sub_1B850's 40x28 `sub_1C5D0` plane-A blit.
#[cfg(feature = "audio")]
pub fn apply_gems_title_intro_plane_a_blit(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    vint_count: u8,
) -> Option<wildside_core::vdp::DecompressToVramTransferAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::title_intro_plane_a_blit_with_scheduler(
        entry,
        vint_count,
        |cycles, vdp_owns_bus| {
            audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                u64::from(cycles.get()) * 7,
                vdp_owns_bus,
            );
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock sub_1B850's 28 full-width repeated plane-B rows.
#[cfg(feature = "audio")]
pub fn apply_gems_title_intro_repeated_rows(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    vint_count: u8,
) -> Option<wildside_core::vdp::DecompressToVramTransferAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::title_intro_repeated_rows_with_scheduler(
        entry,
        vint_count,
        |cycles, vdp_owns_bus| {
            audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                u64::from(cycles.get()) * 7,
                vdp_owns_bus,
            );
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock `Load_IntroSequence1`'s controller allocation after `sub_1B850`.
#[cfg(feature = "audio")]
pub fn apply_gems_title_intro_controller_allocation(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
) -> Option<TitleIntroSourceAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::title_intro_controller_allocation_cursor_with_scheduler(
        entry,
        |cycles, vdp_owns_bus, source_instruction| {
            if source_instruction {
                audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            } else {
                audio.advance_ntsc_master_clocks_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            }
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock the intro loader return through the first `WaitForVint` flag poll.
#[cfg(feature = "audio")]
pub fn apply_gems_title_intro_wait_for_vint_poll(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
) -> Option<TitleIntroSourceAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::title_intro_wait_for_vint_poll_cursor_with_scheduler(
        entry,
        |cycles, vdp_owns_bus, source_instruction| {
            if source_instruction {
                audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            } else {
                audio.advance_ntsc_master_clocks_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            }
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock the blocking title frame wait through `Mode_Standard` entry.
#[cfg(feature = "audio")]
pub fn apply_gems_title_wait_for_vint_to_mode_standard(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
) -> Option<TitleWaitForVintAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::title_wait_for_vint_to_mode_standard_cursor_with_scheduler(
        entry,
        |cycles, vdp_owns_bus, source_instruction| {
            if source_instruction {
                audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            } else {
                audio.advance_ntsc_master_clocks_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            }
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock the first title `Mode_Standard -> Do_Nothing` delay.
#[cfg(feature = "audio")]
pub fn apply_gems_title_mode_standard_do_nothing(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
) -> Option<TitleIntroSourceAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::title_mode_standard_do_nothing_cursor_with_scheduler(
        entry,
        |cycles, vdp_owns_bus, source_instruction| {
            if source_instruction {
                audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            } else {
                audio.advance_ntsc_master_clocks_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            }
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock a caller and ASM `sub_142CC` up to `DecompressToVRAM loc_14300`.
#[cfg(feature = "audio")]
pub fn apply_gems_decompress_to_vram_setup_sub_142cc(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    caller_instruction_cycles: &[u32],
) -> Option<wildside_core::vdp::DecompressToVramSetupAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::decompress_to_vram_setup_cursor_sub_142cc_with_scheduler(
        entry,
        caller_instruction_cycles,
        |cycles, vdp_owns_bus, source_instruction| {
            if source_instruction {
                audio.advance_ntsc_m68k_instruction_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            } else {
                audio.advance_ntsc_master_clocks_with_vdp_ownership(
                    u64::from(cycles.get()) * 7,
                    vdp_owns_bus,
                );
            }
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Clock `Load_InGame $11A0E..$11A24` up to `StopMusic $E14C8`.
#[cfg(feature = "audio")]
pub fn apply_gems_load_in_game_to_stop_music_entry(
    audio: &mut GemsMasterClockTimingEngine,
    entry: LoadInGameCpuRasterCursor,
    vint: Option<LoadInGamePlatformVintEvent>,
    refresh: LoadInGamePlatformRefreshLedger,
    timer_path: LoadInGameTimerBonusPath,
) -> Option<LoadInGameCpuRasterCursor> {
    let platforms = apply_gems_load_in_game_platform_initialisation(audio, entry, vint, refresh)?;
    apply_gems_load_in_game_stop_music_tail(audio, platforms.cursor, timer_path)
}

/// Execute an already-positioned ASM `StopMusic` schedule on the persistent
/// master-clocked GEMS machine.
#[cfg(feature = "audio")]
pub fn apply_timed_gems_stop_music(
    audio: &mut GemsMasterClockTimingEngine,
    timing: GemsStopMusicElapsedTiming,
) -> GemsMasterClockAdvance {
    const MCLK_PER_M68K_CYCLE: u64 = 7;

    let mut cursor = M68kCycleCount::from_raw(0);
    let mut reads = Vec::new();
    let mut advance_to = |target: M68kCycleCount, audio: &mut GemsMasterClockTimingEngine| {
        let cycles = target
            .get()
            .checked_sub(cursor.get())
            .expect("StopMusic control events stay ordered");
        reads.extend(
            audio
                .advance_ntsc_master_clocks(u64::from(cycles) * MCLK_PER_M68K_CYCLE)
                .banked_68k_reads,
        );
        cursor = target;
    };

    let driver = timing.bus_windows[0];
    advance_to(timing.driver_reset_release_complete_elapsed_offset, audio);
    audio.apply_stop_music_control(GemsStopMusicControl::ReleaseDriverReset);
    advance_to(driver.request_complete_elapsed_offset, audio);
    audio.apply_stop_music_control(GemsStopMusicControl::BeginDriverBus);
    advance_to(driver.ack_complete_elapsed_offset, audio);
    audio.apply_stop_music_control(GemsStopMusicControl::CopyDriver);
    advance_to(timing.reset_assert_complete_elapsed_offset, audio);
    audio.apply_stop_music_control(GemsStopMusicControl::AssertReset);
    advance_to(driver.release_complete_elapsed_offset, audio);
    audio.apply_stop_music_control(GemsStopMusicControl::ReleaseDriverBus);
    advance_to(timing.reset_release_complete_elapsed_offset, audio);
    audio.apply_stop_music_control(GemsStopMusicControl::ReleaseReset);

    let transactions = [
        GemsStopMusicQueueTransaction::CommandMarker,
        GemsStopMusicQueueTransaction::DataPointersCommand,
        GemsStopMusicQueueTransaction::PatchPointer,
        GemsStopMusicQueueTransaction::EnvelopePointer,
        GemsStopMusicQueueTransaction::SequencePointer,
        GemsStopMusicQueueTransaction::SamplePointer,
    ];
    for (window, transaction) in timing.bus_windows[1..].iter().zip(transactions) {
        advance_to(window.request_complete_elapsed_offset, audio);
        audio.apply_stop_music_control(GemsStopMusicControl::BeginQueueTransaction);
        advance_to(window.release_complete_elapsed_offset, audio);
        audio.apply_stop_music_control(GemsStopMusicControl::WriteQueue(transaction));
        audio.apply_stop_music_control(GemsStopMusicControl::ReleaseQueueBus);
    }
    advance_to(timing.elapsed_cycles, audio);

    GemsMasterClockAdvance {
        banked_68k_reads: reads,
        pending_m68k_wait_cycles: audio.pending_m68k_wait_cycles(),
    }
}

/// Derive and execute a source-positioned `StopMusic $E14C8` call.
///
/// The timing engine must already be on the same NTSC raster position as the
/// absolute 68000 entry counter. This keeps VINT derivation engine-level while
/// rejecting callers that try to apply the schedule at a different beam.
#[cfg(feature = "audio")]
pub fn apply_timed_gems_stop_music_from_absolute_entry(
    audio: &mut GemsMasterClockTimingEngine,
    polls: GemsStopMusicBusAckPolls,
    phase_before: M68kRefreshPhase,
    absolute_entry_cycles: M68kCycleCount,
) -> Option<(GemsStopMusicElapsedTiming, GemsMasterClockAdvance)> {
    const MCLK_PER_M68K_CYCLE: u64 = 7;
    const NTSC_FRAME_MASTER_CLOCKS: u64 = 3_420 * 262;

    if audio.ntsc_timeline_origin() != GemsNtscTimelineOrigin::PowerOnScanlineZero {
        return None;
    }
    let absolute_entry_master_clock =
        u64::from(absolute_entry_cycles.get()).checked_mul(MCLK_PER_M68K_CYCLE)?;
    if audio.master_clock() % NTSC_FRAME_MASTER_CLOCKS
        != absolute_entry_master_clock % NTSC_FRAME_MASTER_CLOCKS
    {
        return None;
    }
    let timing = wildside_core::gems_stop_music_elapsed_timing_from_absolute_entry(
        polls,
        phase_before,
        absolute_entry_cycles,
    )?;
    let advance = apply_timed_gems_stop_music(audio, timing);
    Some((timing, advance))
}

/// Co-schedule GEMS banked reads with the life-icon-only ASM `sub_44DC` path.
///
/// The timing engine must already be positioned at the supplied CPU cursor.
/// Core owns the source instructions, refresh recurrence, and VDP FIFO
/// ownership; this adapter advances the persistent Z80 for each interval and
/// returns its newly generated 68000 wait to the next scheduler tick.
#[cfg(feature = "audio")]
pub fn apply_gems_life_icon_schedule_sub_44dc(
    audio: &mut GemsMasterClockTimingEngine,
    entry: ModeLevelCpuRasterCursor,
    commands: &[HudDmaEventSub44DC],
) -> Option<ModeLevelCpuRasterCursor> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    wildside_core::vdp::life_icon_cpu_raster_cursor_after_sub_44dc_with_scheduler(
        entry,
        commands,
        |cycles, vdp_owns_bus| {
            audio.advance_master_clocks(u64::from(cycles.get()) * 7, false, vdp_owns_bus);
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
    )
}

/// Execute one ASM-positioned GEMS queue command on the persistent machine.
#[cfg(feature = "audio")]
pub fn apply_timed_gems_queue_command(
    audio: &mut GemsMasterClockTimingEngine,
    path: GemsQueueCommandPath,
    phase_before: M68kRefreshPhase,
    command: Option<GemsQueueCommand>,
) -> Option<GemsQueueCommandAdvance> {
    if audio.pending_m68k_wait_cycles() != 0 {
        return None;
    }
    if (path == GemsQueueCommandPath::OpenCloseSubE1576) != command.is_none() {
        return None;
    }
    let audio = std::cell::RefCell::new(audio);
    wildside_core::gems_queue_command_advance_with_scheduler(
        path,
        phase_before,
        |cycles, vdp_owns_bus| {
            let mut audio = audio.borrow_mut();
            audio.advance_master_clocks(u64::from(cycles.get()) * 7, false, vdp_owns_bus);
            Some(M68kCycleCount::from_raw(
                audio.take_pending_m68k_wait_cycles(),
            ))
        },
        |control| {
            let control = match control {
                wildside_core::GemsQueueCommandControl::BeginQueueBus => {
                    GemsQueueCommandControl::BeginQueueBus
                }
                wildside_core::GemsQueueCommandControl::WriteQueueBytes => {
                    GemsQueueCommandControl::WriteQueueBytes(
                        command.expect("validated queue-writing command"),
                    )
                }
                wildside_core::GemsQueueCommandControl::ReleaseQueueBus => {
                    GemsQueueCommandControl::ReleaseQueueBus
                }
            };
            audio.borrow_mut().apply_queue_command_control(control);
        },
    )
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

pub fn native_bgm_cue_has_audible_output(active_bgm_cue: Option<NativeBgmCue>) -> bool {
    matches!(
        active_bgm_cue,
        Some(
            NativeBgmCue::TitleShell
                | NativeBgmCue::Options
                | NativeBgmCue::Level(_)
                | NativeBgmCue::CoreQueued(_)
                | NativeBgmCue::ScoreBoard
                | NativeBgmCue::Ending
        )
    )
}

pub fn native_audio_frame_needs_driver_step(
    active_bgm_cue: Option<NativeBgmCue>,
    pending_bgm_start: Option<u8>,
    bgm_command: Option<NativeBgmPlaybackCommand>,
    trace_audio_command_count: usize,
) -> bool {
    native_bgm_cue_has_audible_output(active_bgm_cue)
        || pending_bgm_start.is_some()
        || bgm_command.is_some()
        || trace_audio_command_count != 0
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use wildside_data::{LevelRecord, LevelResources};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RecordingAudioAction {
        StartSequence(u8),
        StopSequence(u8),
        StopMusic,
        SetMusicSpeed(u8),
        StepFrame,
        GenerateSamples(usize),
    }

    #[derive(Default)]
    struct RecordingAudioSink {
        actions: Vec<RecordingAudioAction>,
        continuous_clock: bool,
    }

    impl NativeAudioSink for RecordingAudioSink {
        fn start_sequence(&mut self, sound_id: u8) {
            self.actions
                .push(RecordingAudioAction::StartSequence(sound_id));
        }

        fn stop_sequence(&mut self, sound_id: u8) {
            self.actions
                .push(RecordingAudioAction::StopSequence(sound_id));
        }

        fn stop_music(&mut self) {
            self.actions.push(RecordingAudioAction::StopMusic);
        }

        fn set_music_speed(&mut self, speed: u8) {
            self.actions
                .push(RecordingAudioAction::SetMusicSpeed(speed));
        }

        fn step_frame(&mut self) {
            self.actions.push(RecordingAudioAction::StepFrame);
        }

        fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
            self.actions
                .push(RecordingAudioAction::GenerateSamples(sample_count));
            vec![sample_count as f32; sample_count * 2]
        }

        fn requires_continuous_clock(&self) -> bool {
            self.continuous_clock
        }
    }

    fn test_manifest() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: 0,
            rom_sha1: String::from("test"),
            upstream_commit: String::from("test"),
            source_root: PathBuf::new(),
            output_root: PathBuf::new(),
            map_order: Vec::new(),
            levels: vec![LevelRecord {
                level_id: 0x00,
                title_text_id: None,
                display_name: Some(String::from("Blue Lake Woods 1")),
                title_card: None,
                resource_paths: Vec::new(),
                resources: LevelResources::default(),
                header_metadata: Some(LevelHeader {
                    width_screens: 1,
                    height_screens: 1,
                    height_flag_bits: 0,
                    level_theme: 0x08,
                    murder_wall_flags: 0,
                    background_theme: 0x08,
                    weather_flags: 0,
                    player_start_x: 0x0750,
                    player_start_y: 0x00E0,
                    flag_x: 0x0750,
                    flag_y: 0x0300,
                }),
                enemy_layout_metadata: None,
                bgscroll_metadata: None,
                platform_metadata: None,
                background_layout_alias: None,
            }],
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

    fn trace(mode: GameMode) -> NativeTraceFrame {
        NativeTraceFrame {
            game_mode: mode,
            game_mode_value: mode.as_disassembly_value(),
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            ..NativeTraceFrame::default()
        }
    }

    #[test]
    fn audio_sample_clock_distributes_fractional_frame_remainder() {
        let mut default_clock = AudioSampleClock::new(AUDIO_SAMPLE_RATE);
        assert_eq!(default_clock.next_frame_samples(), AUDIO_SAMPLES_PER_FRAME);

        let mut half_rate_clock = AudioSampleClock::new(22_050);
        assert_eq!(
            (0..6)
                .map(|_| half_rate_clock.next_frame_samples())
                .collect::<Vec<_>>(),
            vec![367, 368, 367, 368, 367, 368],
            "the 60Hz audio clock must carry the integer remainder between frames"
        );

        let mut fractional_clock = AudioSampleClock::new(44_117);
        assert_eq!(
            (0..60)
                .map(|_| fractional_clock.next_frame_samples())
                .sum::<usize>(),
            44_117,
            "fractional output rates must not drift over a one-second 60Hz window"
        );
    }

    #[test]
    fn settled_audio_initial_bgm_id_resolves_manifest_level_cue() {
        let manifest = test_manifest();

        assert_eq!(
            settled_audio_initial_bgm_id(&manifest, Some(NativeBgmCue::Level(LevelId(0)))),
            Some(gems_bgm::CAVE),
            "ASM loc_12004 maps theme 8 through the foreground-theme BGM table"
        );
        assert_eq!(
            settled_audio_initial_bgm_id(&manifest, Some(NativeBgmCue::TitleShell)),
            Some(gems_bgm::ICE),
            "ASM Load_TitleScreen queues the title-shell bgm_Ice cue"
        );
        assert_eq!(settled_audio_initial_bgm_id(&manifest, None), None);
        assert_eq!(
            settled_audio_initial_bgm_id(&manifest, Some(NativeBgmCue::StoppedForTransition)),
            None
        );
    }

    #[test]
    fn level_change_waits_for_explicit_loc_12004_bgm_after_stop_music() {
        let manifest = test_manifest();
        let mut next_level = trace(GameMode::DemoPlay);
        next_level.current_level = LevelId(27);

        let (stopped, reset) = native_bgm_state_after_trace(
            &manifest,
            &next_level,
            Some(NativeBgmCue::Level(LevelId(0))),
        );
        assert_eq!(stopped, Some(NativeBgmCue::StoppedForTransition));
        assert_eq!(reset, Some(NativeBgmPlaybackCommand::StopMusic));

        assert_eq!(
            native_bgm_state_after_trace(&manifest, &next_level, stopped),
            (Some(NativeBgmCue::StoppedForTransition), None),
            "ASM Load_InGame waits between StopMusic and loc_12004"
        );

        next_level.played_bgm_ids.push(gems_bgm::SKY);
        assert_eq!(
            native_bgm_state_after_trace(&manifest, &next_level, stopped),
            (
                Some(NativeBgmCue::CoreQueued(gems_bgm::SKY)),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::SKY)),
            ),
            "ASM loc_12004's j_PlaySound call is the authoritative start event"
        );
    }

    #[test]
    fn intro_return_holds_title_bgm_until_load_title_screen_play_sound() {
        // ASM sub_1C246 -> loc_1C264 enters the blocking title rebuild:
        // StopMusic $E14C8 runs on published frame 139, while
        // Load_TitleScreen -> PlaySound/sub_E151C runs at title elapsed 32.
        let manifest = test_manifest();
        let mut intro = trace(GameMode::IntroSequence1);
        intro.boot_phase = Some(String::from("intro_sequence_1"));
        intro.boot_phase_elapsed = Some(102);
        let mut title_entry = trace(GameMode::TitleScreen);
        title_entry.boot_phase = Some(String::from("title_screen"));
        title_entry.boot_phase_elapsed = Some(0);

        let (stopped, reset) = native_bgm_state_after_trace_delta(
            &manifest,
            Some(&intro),
            &title_entry,
            Some(NativeBgmCue::TitleShell),
        );
        assert_eq!(stopped, Some(NativeBgmCue::StoppedForTransition));
        assert_eq!(reset, Some(NativeBgmPlaybackCommand::StopMusic));

        let mut title_loading = title_entry.clone();
        title_loading.boot_phase_elapsed = Some(31);
        assert_eq!(
            native_bgm_state_after_trace_delta(
                &manifest,
                Some(&title_entry),
                &title_loading,
                stopped,
            ),
            (stopped, None),
            "the blocking Load_TitleScreen workload must remain silent before PlaySound",
        );

        let mut title_play_sound = title_loading.clone();
        title_play_sound.boot_phase_elapsed = Some(32);
        assert_eq!(
            native_bgm_state_after_trace_delta(
                &manifest,
                Some(&title_loading),
                &title_play_sound,
                stopped,
            ),
            (
                Some(NativeBgmCue::TitleShell),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::ICE)),
            ),
        );
    }

    #[test]
    fn demo_entry_treats_only_new_loc_12004_bgm_ids_as_queue_events() {
        let manifest = test_manifest();
        let mut title = trace(GameMode::TitleScreen);
        title.played_bgm_ids.push(gems_bgm::SKY);
        let mut demo_loader = trace(GameMode::DemoPlay);
        demo_loader.played_bgm_ids.push(gems_bgm::SKY);

        let (stopped, reset) = native_bgm_state_after_trace_delta(
            &manifest,
            Some(&title),
            &demo_loader,
            Some(NativeBgmCue::TitleShell),
        );
        assert_eq!(stopped, Some(NativeBgmCue::TitleShell));
        assert_eq!(reset, None);

        let mut level_switched = demo_loader.clone();
        level_switched.current_level = LevelId(27);
        let (stopped, reset) = native_bgm_state_after_trace_delta(
            &manifest,
            Some(&demo_loader),
            &level_switched,
            stopped,
        );
        assert_eq!(stopped, Some(NativeBgmCue::StoppedForTransition));
        assert_eq!(
            reset,
            Some(NativeBgmPlaybackCommand::StopMusic),
            "ASM Load_DemoPlay -> Load_InGame reaches StopMusic at the Current_LevelID switch before loc_12004"
        );

        let mut queued = level_switched.clone();
        queued.played_bgm_ids.push(gems_bgm::SKY);
        assert_eq!(
            native_bgm_state_after_trace_delta(&manifest, Some(&level_switched), &queued, stopped,),
            (
                Some(NativeBgmCue::CoreQueued(gems_bgm::SKY)),
                Some(NativeBgmPlaybackCommand::Start(gems_bgm::SKY)),
            ),
            "ASM loc_12004's new trace event starts the demo BGM after the reset gap"
        );

        let active = Some(NativeBgmCue::CoreQueued(gems_bgm::SKY));
        assert_eq!(
            native_bgm_state_after_trace_delta(&manifest, Some(&queued), &queued, active),
            (active, None),
            "the cumulative loc_12004 event must not reset GEMS again on the following frame"
        );
    }

    #[test]
    fn apply_native_bgm_command_maps_stopmusic_delay() {
        let mut sink = RecordingAudioSink::default();

        assert_eq!(
            apply_native_bgm_command(&mut sink, NativeBgmPlaybackCommand::Start(gems_bgm::ICE)),
            None
        );
        assert_eq!(
            apply_native_bgm_command(&mut sink, NativeBgmPlaybackCommand::StopMusic),
            None
        );
        assert_eq!(
            apply_native_bgm_command(
                &mut sink,
                NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE),
            ),
            Some(gems_bgm::CAVE),
            "ASM StopMusic (kid.asm:49727) resets GEMS before the next BGM start"
        );
        assert_eq!(
            sink.actions,
            vec![
                RecordingAudioAction::StartSequence(gems_bgm::ICE),
                RecordingAudioAction::StopMusic,
                RecordingAudioAction::StopMusic,
            ]
        );
    }

    #[test]
    fn native_bgm_command_flush_and_sfx_suppression_rules_match_reset_boundaries() {
        let in_game = trace(GameMode::InGame);
        let transition = trace(GameMode::LevelTransition);
        let title = trace(GameMode::TitleScreen);

        assert!(!native_bgm_command_flushes_queued_audio(
            NativeBgmPlaybackCommand::Start(gems_bgm::ICE),
        ));
        assert!(native_bgm_command_flushes_queued_audio(
            NativeBgmPlaybackCommand::StopMusic,
        ));
        assert!(native_bgm_command_flushes_queued_audio(
            NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE),
        ));

        assert!(!native_bgm_command_suppresses_same_frame_sfx(
            NativeBgmPlaybackCommand::Start(gems_bgm::ICE),
            &transition,
            Some(NativeBgmCue::ScoreBoard),
        ));
        assert!(
            native_bgm_command_suppresses_same_frame_sfx(
                NativeBgmPlaybackCommand::StopMusic,
                &transition,
                Some(NativeBgmCue::ScoreBoard),
            ),
            "ASM Load_InGame (kid.asm:23623-23629) resets GEMS during the level-transition shell"
        );
        assert!(!native_bgm_command_suppresses_same_frame_sfx(
            NativeBgmPlaybackCommand::StopMusic,
            &in_game,
            Some(NativeBgmCue::ScoreBoard),
        ));
        assert!(native_bgm_command_suppresses_same_frame_sfx(
            NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE),
            &in_game,
            None,
        ));
        assert!(!native_bgm_command_suppresses_same_frame_sfx(
            NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE),
            &in_game,
            Some(NativeBgmCue::Options),
        ));
        assert!(!native_bgm_command_suppresses_same_frame_sfx(
            NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE),
            &title,
            None,
        ));
    }

    #[test]
    fn native_audio_frame_driver_step_rule_skips_silent_initial_title_card_loc_1ab26() {
        assert!(
            !native_audio_frame_needs_driver_step(None, None, None, 0),
            "ASM Load_TitleCard loc_1AB26/loc_1AE2E hibernates without PlaySound, so a cold loading card must not clock idle GEMS noise"
        );
        assert!(!native_audio_frame_needs_driver_step(
            Some(NativeBgmCue::StoppedForTransition),
            None,
            None,
            0
        ));
        assert!(native_audio_frame_needs_driver_step(
            Some(NativeBgmCue::Level(LevelId(0))),
            None,
            None,
            0
        ));
        assert!(native_audio_frame_needs_driver_step(
            None,
            Some(gems_bgm::CAVE),
            None,
            0
        ));
        assert!(native_audio_frame_needs_driver_step(
            None,
            None,
            Some(NativeBgmPlaybackCommand::StopMusic),
            0
        ));
        assert!(native_audio_frame_needs_driver_step(None, None, None, 1));
    }

    #[test]
    fn timing_sink_clocks_through_silent_title_card_loc_1ab26() {
        let manifest = test_manifest();
        let transition = trace(GameMode::LevelTransition);
        let mut sink = RecordingAudioSink {
            continuous_clock: true,
            ..RecordingAudioSink::default()
        };
        let mut director = NativeAudioDirector::new();

        assert!(director.step(&manifest, &transition, &mut sink));
        assert_eq!(
            sink.actions,
            vec![RecordingAudioAction::StepFrame],
            "persistent GEMS timing state must advance while the audible mixer emits silence",
        );
    }

    #[cfg(feature = "audio")]
    #[test]
    fn timed_stop_music_drives_each_e14c8_control_window_on_master_clock() {
        let timing = wildside_core::gems_stop_music_elapsed_timing(
            wildside_core::GemsStopMusicBusAckPolls {
                driver_copy_sub_e13da: 1,
                queue_transactions_sub_e143a: [1; 6],
            },
            wildside_core::M68kRefreshPhase::new(14).expect("trace entry phase"),
            &[M68kCycleCount::from_raw(108_848)],
        )
        .expect("frame-867 StopMusic schedule");
        let mut engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);
        let load_in_game_cursor = wildside_core::vdp::LoadInGameCpuRasterCursor {
            beam: wildside_core::vdp::H40BeamPosition::new(151, 432)
                .expect("published-route Load_InGame absolute-cycle beam"),
            refresh_phase: wildside_core::M68kRefreshPhase::new(116)
                .expect("Load_InGame refresh phase"),
        };
        let load_in_game_frame_cursor =
            wildside_core::vdp::ntsc_vblank_cursor_from_h40_beam(load_in_game_cursor.beam);
        engine.advance_ntsc_master_clocks(u64::from(load_in_game_frame_cursor.get()));
        let entry_cursor = apply_gems_load_in_game_to_stop_music_entry(
            &mut engine,
            load_in_game_cursor,
            Some(wildside_core::vdp::LoadInGamePlatformVintEvent {
                vertical_decoder_raw_offset: M68kCycleCount::from_raw(9_424),
            }),
            wildside_core::vdp::LoadInGamePlatformRefreshLedger {
                prelude: 7,
                horizontal_decoder: 115,
                between_uploads: 0,
                vertical_decoder_before_vint: 75,
                vint_exception: 0,
                vint_handler: 1,
                vertical_decoder_after_vint: 40,
                epilogue: 0,
            },
            wildside_core::LoadInGameTimerBonusPath::Initialise,
        )
        .expect("published-route Load_InGame to StopMusic schedule");
        assert_eq!(
            entry_cursor.beam,
            wildside_core::vdp::H40BeamPosition::new(250, 1_219)
                .expect("oracle StopMusic absolute-cycle beam"),
        );
        assert_eq!(
            engine.master_clock() % (3_420 * 262),
            u64::from(
                wildside_core::vdp::ntsc_vblank_cursor_from_h40_beam(entry_cursor.beam).get()
            )
        );

        let entry_master_clock = engine.master_clock();
        let advance = apply_timed_gems_stop_music(&mut engine, timing);

        assert!(advance.banked_68k_reads.is_empty());
        assert_eq!(engine.master_clock(), entry_master_clock + 229_570 * 7);
        engine.start_sequence(3);
    }

    #[cfg(feature = "audio")]
    #[test]
    fn timed_title_stop_music_matches_frame_139_absolute_schedule_e14c8() {
        // ASM sub_1C246 -> loc_1C264 -> StopMusic $E14C8. Selected-PC
        // tracing enters at absolute cycle 17,789,000 / phase 32 and reaches
        // the final RTS at +229,558 before its 16-cycle completion.
        let absolute_entry = M68kCycleCount::from_raw(17_789_000);
        let entry_master_clock = u64::from(absolute_entry.get()) * 7;
        let mut frame_relative_engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);
        assert!(
            apply_timed_gems_stop_music_from_absolute_entry(
                &mut frame_relative_engine,
                wildside_core::GemsStopMusicBusAckPolls {
                    driver_copy_sub_e13da: 1,
                    queue_transactions_sub_e143a: [1; 6],
                },
                M68kRefreshPhase::new(32).expect("frame-139 selected-PC phase"),
                absolute_entry,
            )
            .is_none(),
            "absolute schedules require a power-on raster origin"
        );
        let mut engine = GemsMasterClockTimingEngine::new_with_ntsc_timeline_origin(
            &[0x76],
            vec![],
            GemsNtscTimelineOrigin::PowerOnScanlineZero,
        );
        engine.advance_ntsc_master_clocks(entry_master_clock);
        assert!(
            engine.checkpoint().reset,
            "absolute power-on timeline starts with the Z80 reset asserted"
        );

        let (timing, advance) = apply_timed_gems_stop_music_from_absolute_entry(
            &mut engine,
            wildside_core::GemsStopMusicBusAckPolls {
                driver_copy_sub_e13da: 1,
                queue_transactions_sub_e143a: [1; 6],
            },
            M68kRefreshPhase::new(32).expect("frame-139 selected-PC phase"),
            absolute_entry,
        )
        .expect("frame-139 title StopMusic schedule");

        assert_eq!(timing.elapsed_cycles, M68kCycleCount::from_raw(229_574));
        assert_eq!(
            timing.driver_reset_release_complete_elapsed_offset,
            M68kCycleCount::from_raw(66),
        );
        assert_eq!(timing.refresh_wait_events, 1_509);
        assert_eq!(timing.phase_after, M68kRefreshPhase::new(102).unwrap());
        assert!(advance.banked_68k_reads.is_empty());
        assert!(!engine.checkpoint().reset);
        assert_eq!(
            engine.master_clock() % (3_420 * 262),
            (entry_master_clock + 229_574 * 7) % (3_420 * 262),
        );
    }

    #[cfg(feature = "audio")]
    #[test]
    fn load_in_game_tail_adapter_reaches_stop_music_e14c8_with_halted_z80() {
        let entry = wildside_core::vdp::LoadInGameCpuRasterCursor {
            beam: wildside_core::vdp::H40BeamPosition::new(243, 2_563)
                .expect("published-route Load_InGame tail entry beam"),
            refresh_phase: wildside_core::M68kRefreshPhase::new(114)
                .expect("Load_InGame tail entry phase"),
        };
        let expected = wildside_core::vdp::load_in_game_stop_music_cursor_from_sub_28fc(
            entry,
            wildside_core::LoadInGameTimerBonusPath::Initialise,
        );
        let mut engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);
        let master_clock_before = engine.master_clock();

        let actual = apply_gems_load_in_game_stop_music_tail(
            &mut engine,
            entry,
            wildside_core::LoadInGameTimerBonusPath::Initialise,
        )
        .expect("halted Z80 generates no bank waits");

        assert_eq!(actual, expected);
        assert_eq!(
            engine.master_clock() - master_clock_before,
            u64::from(
                wildside_core::load_in_game_stop_music_tail_advance(
                    entry.refresh_phase,
                    wildside_core::LoadInGameTimerBonusPath::Initialise,
                )
                .elapsed
                .get()
            ) * 7,
            "ASM Load_InGame $11A12..$11A24 clocks GEMS through every source and refresh interval",
        );
    }

    #[cfg(feature = "audio")]
    #[test]
    fn platform_adapter_rejects_unpositioned_decoder_gems_waits() {
        // ASM Initialize_Platforms sub_231C -> Decompress_Chunk and GEMS
        // XFER68K $0285: fixed decoder spans cannot place a generated 68000
        // wait until their instruction visitor exists.
        let driver = [0x3A, 0x00, 0x80, 0xC3, 0x00, 0x00];
        let mut engine = GemsMasterClockTimingEngine::new(&driver, vec![0; 0x8001]);
        let advance = apply_gems_load_in_game_platform_initialisation(
            &mut engine,
            wildside_core::vdp::LoadInGameCpuRasterCursor {
                beam: wildside_core::vdp::H40BeamPosition::new(151, 432)
                    .expect("published-route entry beam"),
                refresh_phase: wildside_core::M68kRefreshPhase::new(116)
                    .expect("published-route entry phase"),
            },
            Some(wildside_core::vdp::LoadInGamePlatformVintEvent {
                vertical_decoder_raw_offset: M68kCycleCount::from_raw(9_424),
            }),
            wildside_core::vdp::LoadInGamePlatformRefreshLedger {
                prelude: 7,
                horizontal_decoder: 115,
                between_uploads: 0,
                vertical_decoder_before_vint: 75,
                vint_exception: 0,
                vint_handler: 1,
                vertical_decoder_after_vint: 40,
                epilogue: 0,
            },
        );

        assert!(advance.is_none());
        assert_ne!(engine.pending_m68k_wait_cycles(), 0);
    }

    #[cfg(feature = "audio")]
    #[test]
    fn gems_life_icon_adapter_matches_no_read_fifo_schedule_sub_44dc() {
        let entry = wildside_core::vdp::ModeLevelCpuRasterCursor {
            beam: wildside_core::vdp::H40BeamPosition::new(32, 765)
                .expect("frame-1703 sub_44DC beam"),
            refresh_phase: wildside_core::M68kRefreshPhase::new(116)
                .expect("frame-1703 refresh phase"),
        };
        let commands = [HudDmaEventSub44DC::LifeIcon {
            frame_table_byte_offset: 8,
        }];
        let expected =
            wildside_core::vdp::life_icon_cpu_raster_cursor_after_sub_44dc(entry, &commands, &[])
                .expect("no-GEMS reference schedule");
        let mut engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);

        let actual = apply_gems_life_icon_schedule_sub_44dc(&mut engine, entry, &commands)
            .expect("HALT-only GEMS engine adds no bank waits");

        assert_eq!(actual, expected);
        assert!(engine.take_banked_68k_reads().is_empty());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn intro_decompress_to_vram_reaches_all_sub_1b850_transfers() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.cache/upstream/kid-chameleon-disasm/kid.bin"),
        )
        .expect("bootstrapped Kid ROM");
        let mut engine = GemsMasterClockTimingEngine::from_kid_rom_with_ntsc_timeline_origin(
            &rom,
            GemsNtscTimelineOrigin::PowerOnScanlineZero,
        )
        .expect("Kid GEMS driver");
        let polls = GemsStopMusicBusAckPolls {
            driver_copy_sub_e13da: 1,
            queue_transactions_sub_e143a: [1; 6],
        };

        // ASM StopMusic $E14C8 on the three pre-title shell resets.
        for (entry, phase) in [(1_886_313_u32, 46_u8), (3_868_666, 72), (8_776_957, 124)] {
            engine.advance_ntsc_master_clocks(u64::from(entry) * 7 - engine.master_clock());
            let _ = engine.take_pending_m68k_wait_cycles();
            apply_timed_gems_stop_music_from_absolute_entry(
                &mut engine,
                polls,
                M68kRefreshPhase::new(phase).unwrap(),
                M68kCycleCount::from_raw(entry),
            )
            .expect("source-positioned StopMusic");
        }

        // ASM Load_TitleScreen -> PlaySound -> sub_E151C queues bgm_Ice.
        engine.advance_ntsc_master_clocks(9_006_731 * 7 - engine.master_clock());
        let _ = engine.take_pending_m68k_wait_cycles();
        apply_timed_gems_queue_command(
            &mut engine,
            GemsQueueCommandPath::StartOrStopSequenceSubE151c,
            M68kRefreshPhase::new(10).unwrap(),
            Some(GemsQueueCommand::StartSequence(gems_bgm::ICE)),
        )
        .expect("title Ice queue command");

        let transfer_entry_cycles = M68kCycleCount::from_raw(9_161_190);
        engine.advance_ntsc_master_clocks(
            u64::from(transfer_entry_cycles.get()) * 7 - engine.master_clock(),
        );
        let _ = engine.take_pending_m68k_wait_cycles();
        let entry_checkpoint = engine.checkpoint();
        assert_eq!(entry_checkpoint.z80_pc, 0x0793);
        assert_eq!(entry_checkpoint.z80_mclk_counter, 8);
        assert_eq!(entry_checkpoint.z80_t_cycles_wait, 4);
        assert_eq!(
            entry_checkpoint.z80_index_registers,
            [0x1B80, 0x0009, 0x1A20]
        );
        assert_eq!(entry_checkpoint.bank_address, 0x000E_0000);
        assert!(entry_checkpoint.z80_odd_access);
        assert!(entry_checkpoint.z80_iff1);
        assert!(entry_checkpoint.z80_iff2);
        let mut scheduler_elapsed = 0_u32;
        let mut first_fifo_halt_cycle = None;
        let mut first_oracle_collision_is_fifo_halted = false;
        let mut vint_autovector_cycle = None;
        let mut external_wait_cycles = 0_u32;
        let transfer =
            wildside_core::vdp::decompress_to_vram_transfer_cursor_loc_14314_with_scheduler(
                LoadInGameCpuRasterCursor {
                    // ASM sub_1B2A4 -> DecompressToVRAM loc_14314, selected-PC
                    // oracle frame 71: d3=$07DF means 2,016 inclusive writes.
                    beam: wildside_core::vdp::h40_beam_from_absolute_m68k_cycles(
                        transfer_entry_cycles,
                    ),
                    refresh_phase: M68kRefreshPhase::new(101).unwrap(),
                },
                2_016,
                1,
                wildside_core::vdp::DecompressToVramTransferExit::Return,
                |cycles, vdp_owns_bus| {
                    if vdp_owns_bus && first_fifo_halt_cycle.is_none() {
                        first_fifo_halt_cycle =
                            Some(transfer_entry_cycles.get() + scheduler_elapsed);
                    }
                    if vdp_owns_bus && transfer_entry_cycles.get() + scheduler_elapsed == 9_216_149
                    {
                        first_oracle_collision_is_fifo_halted = true;
                    }
                    if cycles.get() == 44 {
                        vint_autovector_cycle =
                            Some(transfer_entry_cycles.get() + scheduler_elapsed);
                    }
                    engine.advance_ntsc_m68k_instruction_with_vdp_ownership(
                        u64::from(cycles.get()) * 7,
                        vdp_owns_bus,
                    );
                    scheduler_elapsed += cycles.get();
                    let pending = engine.take_pending_m68k_wait_cycles();
                    external_wait_cycles += pending;
                    Some(M68kCycleCount::from_raw(pending))
                },
            )
            .expect("title loc_14314 transfer");
        assert!(transfer.vint_serviced);
        assert_eq!(first_fifo_halt_cycle, Some(9_161_360));
        assert!(first_oracle_collision_is_fifo_halted);
        assert_eq!(vint_autovector_cycle, Some(9_197_868));
        assert_eq!(
            engine.master_clock(),
            9_245_822 * 7,
            "ASM return_14334 master-clock boundary",
        );
        assert_eq!(external_wait_cycles, 1_320);
        assert_eq!(transfer.fifo_halted_ticks.get(), 38_532);
        let return_checkpoint = engine.checkpoint();
        assert_eq!(return_checkpoint.z80_pc, 0x0273);
        assert_eq!(return_checkpoint.z80_mclk_counter, 4);
        assert_eq!(return_checkpoint.z80_t_cycles_wait, 3);
        assert_eq!(return_checkpoint.z80_wait_mclk_cycles, 0);
        assert!(return_checkpoint.z80_odd_access);
        assert!(return_checkpoint.z80_iff1);
        assert!(return_checkpoint.z80_iff2);
        assert!(!return_checkpoint.z80_halt);

        // ASM sub_1B850 continues through four row uploads, then passes
        // byte_2D71A (the Wild Side intro background) to DecompressToVRAM.
        let setup = apply_gems_decompress_to_vram_setup_sub_142cc(
            &mut engine,
            transfer.cursor,
            &[
                // ASM sub_1B850 $1B860: move.w/lea/jsr; jump stub.
                8, 12, 20, 10,
            ],
        )
        .expect("source-positioned second DecompressToVRAM setup");
        assert_eq!(setup.refresh_events, 2);
        assert_eq!(setup.external_wait_cycles.get(), 55);
        assert_eq!(engine.master_clock(), 9_246_105 * 7);
        let decoder_entry = engine.checkpoint();
        assert_eq!(decoder_entry.z80_pc, 0x02A5);
        assert_eq!(decoder_entry.z80_mclk_counter, 12);
        assert_eq!(decoder_entry.z80_t_cycles_wait, 5);
        assert_eq!(decoder_entry.z80_wait_mclk_cycles, 0);
        assert_eq!(decoder_entry.z80_index_registers, [0x1A20, 0x15A8, 0x1A00]);
        assert!(!decoder_entry.z80_odd_access);

        let compressed = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../.cache/upstream/kid-chameleon-disasm/scenes/artcomp/Background_for_intro_(Wild_Side).bin",
            ),
        )
        .expect("bootstrapped byte_2D71A");
        let decoder = apply_gems_decompress_to_vram_decoder_loc_14300(
            &mut engine,
            setup.cursor,
            &compressed,
            None,
            0,
        )
        .expect("source-positioned Wild Side decoder");
        assert_eq!(decoder.timing.output_bytes, 192);
        assert_eq!(decoder.timing.instruction_count, 576);
        assert_eq!(decoder.timing.raw_cycles, 5_390);
        assert_eq!(decoder.refresh_events, 43);
        assert_eq!(decoder.external_wait_cycles.get(), 154);
        assert_eq!(engine.master_clock(), 9_251_793 * 7);

        let transfer_entry = engine.checkpoint();
        // ORACLE LIMITATION: jgenesis preserves the previous undocumented X/Y
        // flag bits for GEMS `SUB 1` at $07B3. Real Z80 hardware, and the MIT
        // core used here, copy bits 5/3 from the $FF result. GEMS later
        // serialises AF through its stack, leaving the oracle at PC $02AA
        // while hardware-faithful execution reaches $02AE at this boundary.
        assert_eq!(transfer_entry.z80_pc, 0x02AE);
        assert_eq!(transfer_entry.z80_mclk_counter, 0);
        assert_eq!(transfer_entry.z80_t_cycles_wait, 1);
        assert_eq!(transfer_entry.z80_wait_mclk_cycles, 0);
        assert_eq!(transfer_entry.z80_index_registers[2], 0x1A08);
        assert!(!transfer_entry.z80_odd_access);

        let second_transfer = apply_gems_decompress_to_vram_transfer_loc_14314(
            &mut engine,
            decoder.cursor,
            96,
            0,
            wildside_core::vdp::DecompressToVramTransferExit::Return,
        )
        .expect("byte_2D71A 96-word transfer");
        assert!(!second_transfer.vint_serviced);
        assert_eq!(engine.master_clock(), 9_256_826 * 7);

        // ASM sub_1B850 $1B86E passes byte_2DEBF (the Wild Side door and
        // inside art) to the same DecompressToVRAM source path.
        let setup = apply_gems_decompress_to_vram_setup_sub_142cc(
            &mut engine,
            second_transfer.cursor,
            &[8, 12, 20, 10],
        )
        .expect("source-positioned third DecompressToVRAM setup");
        assert_eq!(engine.master_clock(), 9_257_076 * 7);

        let compressed = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../.cache/upstream/kid-chameleon-disasm/scenes/artcomp/Wild_Side_door_and_inside.bin",
            ),
        )
        .expect("bootstrapped byte_2DEBF");
        let decoder = apply_gems_decompress_to_vram_decoder_loc_14300(
            &mut engine,
            setup.cursor,
            &compressed,
            None,
            1,
        )
        .expect("source-positioned Wild Side door decoder");
        assert_eq!(decoder.timing.output_bytes, 3_456);
        assert_eq!(decoder.timing.instruction_count, 10_831);
        assert_eq!(decoder.timing.raw_cycles, 98_874);
        assert!(decoder.vint_serviced);
        assert_eq!(decoder.refresh_events, 789);
        assert_eq!(decoder.external_wait_cycles.get(), 2_057);
        // ORACLE LIMITATION: the jgenesis undocumented-X/Y SUB defect
        // established above makes GEMS request 32 more 68000 wait cycles
        // over this decoder. Source instructions, refresh, and VINT agree.
        assert_eq!(engine.master_clock(), 9_359_747 * 7);

        let third_transfer = apply_gems_decompress_to_vram_transfer_loc_14314(
            &mut engine,
            decoder.cursor,
            1_728,
            0,
            wildside_core::vdp::DecompressToVramTransferExit::Return,
        )
        .expect("byte_2DEBF 1,728-word transfer");
        assert!(!third_transfer.vint_serviced);
        // The 32-cycle decoder shift changes FIFO collision phase and the
        // hardware-faithful transfer recovers 30 cycles by its return.
        assert_eq!(engine.master_clock(), 9_453_428 * 7);

        // ASM sub_1B850 $1B87C passes byte_261D7 (the kids hanging around
        // Wild Side) to DecompressToVRAM.
        let setup = apply_gems_decompress_to_vram_setup_sub_142cc(
            &mut engine,
            third_transfer.cursor,
            &[8, 12, 20, 10],
        )
        .expect("source-positioned fourth DecompressToVRAM setup");
        assert_eq!(engine.master_clock(), 9_453_656 * 7);

        let compressed = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../.cache/upstream/kid-chameleon-disasm/scenes/artcomp/Kids_hanging_around_Wild_Side.bin",
            ),
        )
        .expect("bootstrapped byte_261D7");
        let decoder = apply_gems_decompress_to_vram_decoder_loc_14300(
            &mut engine,
            setup.cursor,
            &compressed,
            None,
            2,
        )
        .expect("source-positioned kids-hanging decoder chunk 1");
        assert_eq!(
            decoder.timing,
            wildside_core::saxman_timing::DecompressChunkTimingSub13684 {
                instruction_count: 16_810,
                raw_cycles: 140_228,
                output_bytes: 4_096,
            },
        );
        assert_eq!(
            decoder.resume,
            Some(
                wildside_core::saxman_timing::DecompressChunkResumeSub13684 {
                    token_index: 1_240,
                    next_lane: 4,
                },
            ),
        );
        assert!(decoder.vint_serviced);
        assert_eq!(decoder.refresh_events, 1_126);
        assert_eq!(decoder.external_wait_cycles.get(), 3_080);
        assert_eq!(engine.master_clock(), 9_599_482 * 7);

        let transfer = apply_gems_decompress_to_vram_transfer_loc_14314(
            &mut engine,
            decoder.cursor,
            2_048,
            1,
            wildside_core::vdp::DecompressToVramTransferExit::CompactAndResume,
        )
        .expect("byte_261D7 chunk 1 transfer and loc_1432C compaction");
        assert!(transfer.vint_serviced);
        assert_eq!(engine.master_clock(), 9_726_517 * 7);

        let decoder = apply_gems_decompress_to_vram_decoder_loc_14300(
            &mut engine,
            transfer.cursor,
            &compressed,
            decoder.resume,
            0,
        )
        .expect("source-positioned kids-hanging decoder chunk 2");
        assert_eq!(
            decoder.timing,
            wildside_core::saxman_timing::DecompressChunkTimingSub13684 {
                instruction_count: 8_812,
                raw_cycles: 72_476,
                output_bytes: 2_048,
            },
        );
        assert_eq!(
            decoder.resume,
            Some(
                wildside_core::saxman_timing::DecompressChunkResumeSub13684 {
                    token_index: 1_886,
                    next_lane: 4,
                },
            ),
        );
        assert_eq!(decoder.refresh_events, 584);
        assert_eq!(decoder.external_wait_cycles.get(), 1_408);
        assert_eq!(engine.master_clock(), 9_801_627 * 7);

        let transfer = apply_gems_decompress_to_vram_transfer_loc_14314(
            &mut engine,
            decoder.cursor,
            1_024,
            1,
            wildside_core::vdp::DecompressToVramTransferExit::CompactAndResume,
        )
        .expect("byte_261D7 chunk 2 transfer and loc_1432C compaction");
        assert!(transfer.vint_serviced);
        assert_eq!(engine.master_clock(), 9_861_599 * 7);

        let decoder = apply_gems_decompress_to_vram_decoder_loc_14300(
            &mut engine,
            transfer.cursor,
            &compressed,
            decoder.resume,
            0,
        )
        .expect("source-positioned kids-hanging decoder chunk 3");
        assert_eq!(
            decoder.timing,
            wildside_core::saxman_timing::DecompressChunkTimingSub13684 {
                instruction_count: 3_965,
                raw_cycles: 33_704,
                output_bytes: 1_024,
            },
        );
        assert_eq!(decoder.resume, None);
        assert_eq!(decoder.refresh_events, 259);
        assert_eq!(decoder.external_wait_cycles.get(), 1_276);
        assert_eq!(engine.master_clock(), 9_897_155 * 7);

        let transfer = apply_gems_decompress_to_vram_transfer_loc_14314(
            &mut engine,
            decoder.cursor,
            512,
            0,
            wildside_core::vdp::DecompressToVramTransferExit::Return,
        )
        .expect("byte_261D7 final 512-word transfer");
        assert!(!transfer.vint_serviced);
        assert_eq!(engine.master_clock(), 9_924_774 * 7);

        // ASM sub_1B850 $1B88A passes ArtComp_1DA86_LettersNumbers to the
        // final DecompressToVRAM call in the title-art batch.
        let setup = apply_gems_decompress_to_vram_setup_sub_142cc(
            &mut engine,
            transfer.cursor,
            &[8, 12, 20, 10],
        )
        .expect("source-positioned fifth DecompressToVRAM setup");
        assert_eq!(engine.master_clock(), 9_925_002 * 7);

        let compressed = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../.cache/upstream/kid-chameleon-disasm/scenes/artcomp/Intro_text_letters.bin",
        ))
        .expect("bootstrapped ArtComp_1DA86_LettersNumbers");
        let decoder = apply_gems_decompress_to_vram_decoder_loc_14300(
            &mut engine,
            setup.cursor,
            &compressed,
            None,
            1,
        )
        .expect("source-positioned intro-letters decoder");
        assert_eq!(
            decoder.timing,
            wildside_core::saxman_timing::DecompressChunkTimingSub13684 {
                instruction_count: 6_230,
                raw_cycles: 51_386,
                output_bytes: 1_440,
            },
        );
        assert_eq!(decoder.resume, None);
        assert!(decoder.vint_serviced);
        assert_eq!(decoder.refresh_events, 406);
        assert_eq!(decoder.external_wait_cycles.get(), 1_408);
        assert_eq!(engine.master_clock(), 9_978_770 * 7);

        let transfer = apply_gems_decompress_to_vram_transfer_loc_14314(
            &mut engine,
            decoder.cursor,
            720,
            0,
            wildside_core::vdp::DecompressToVramTransferExit::Return,
        )
        .expect("ArtComp_1DA86_LettersNumbers 720-word transfer");
        assert!(!transfer.vint_serviced);
        // The established oracle Z80 flag defect leaves the hardware-faithful
        // GEMS/FIFO schedule 39 cycles earlier at the batch return.
        assert_eq!(engine.master_clock(), 10_010_491 * 7);

        // ASM sub_1B850 $1B89E decodes unk_2FDE0 with base tile $80BC.
        let compressed =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../.cache/upstream/kid-chameleon-disasm/scenes/mapeni/intro1_wildside.bin",
            ))
            .expect("bootstrapped unk_2FDE0");
        let enigma = apply_gems_enigma_decoder(
            &mut engine,
            transfer.cursor,
            &[8, 12, 12, 4, 20, 10],
            &compressed,
            0x80BC,
            0x2FDE0,
            1,
        )
        .expect("source-positioned intro plane-A EniDec");
        assert_eq!(
            enigma.timing,
            wildside_core::enigma_timing::EnigmaTiming {
                instruction_count: 13_848,
                raw_cycles: 119_798,
                output_words: 1_120,
            },
        );
        assert!(enigma.vint_serviced);
        assert_eq!(enigma.refresh_events, 948);
        assert_eq!(enigma.external_wait_cycles.get(), 1_408);
        // Oracle loc_13586 returns at 10,136,554. Its established
        // undocumented-X/Y SUB defect requests 2,752 more GEMS bank-wait
        // cycles in this long VDP-idle decode; the 39-cycle prior offset
        // therefore grows to 2,791 cycles here.
        assert_eq!(engine.master_clock(), 10_133_763 * 7);

        let plane_a = apply_gems_title_intro_plane_a_blit(&mut engine, enigma.cursor, 0)
            .expect("source-positioned sub_1C5D0 plane-A blit");
        assert!(!plane_a.vint_serviced);
        assert_eq!(plane_a.refresh_events, 4);
        assert_eq!(plane_a.fifo_halted_ticks.get(), 33_894);
        assert_eq!(engine.master_clock(), 10_194_570 * 7);

        // ASM sub_1B850 $1B8C2 decodes unk_2FDCE with base tile $413A.
        let compressed = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.cache/upstream/kid-chameleon-disasm/scenes/mapeni/2FDCE.bin"),
        )
        .expect("bootstrapped unk_2FDCE");
        let enigma = apply_gems_enigma_decoder(
            &mut engine,
            plane_a.cursor,
            &[8, 12, 12, 4, 20, 10],
            &compressed,
            0x413A,
            0x2FDCE,
            0,
        )
        .expect("source-positioned repeated-row EniDec");
        assert_eq!(
            enigma.timing,
            wildside_core::enigma_timing::EnigmaTiming {
                instruction_count: 504,
                raw_cycles: 4_672,
                output_words: 28,
            },
        );
        assert!(!enigma.vint_serviced);
        assert_eq!(enigma.refresh_events, 38);
        assert_eq!(enigma.external_wait_cycles.get(), 0);
        assert_eq!(engine.master_clock(), 10_199_384 * 7);

        let repeated_rows = apply_gems_title_intro_repeated_rows(&mut engine, enigma.cursor, 1)
            .expect("source-positioned repeated plane-B rows");
        assert!(repeated_rows.vint_serviced);
        assert_eq!(repeated_rows.refresh_events, 140);
        assert_eq!(repeated_rows.fifo_halted_ticks.get(), 31_916);
        // Oracle sub_1B850 RTS completes at 10,276,031; the same documented
        // GEMS defect leaves the hardware-faithful path 3,151 cycles earlier.
        assert_eq!(engine.master_clock(), 10_272_880 * 7);

        let controller =
            apply_gems_title_intro_controller_allocation(&mut engine, repeated_rows.cursor)
                .expect("source-positioned Load_IntroSequence1 controller allocation");
        assert_eq!(
            controller.timing,
            wildside_core::intro_timing::IntroSourceTiming {
                instruction_count: 81,
                raw_cycles: M68kCycleCount::from_raw(1_020),
            },
        );
        assert_eq!(controller.refresh_events, 8);
        assert_eq!(controller.external_wait_cycles.get(), 0);
        // Oracle Load_IntroSequence1 RTS completes at 10,277,067. The
        // established GEMS defect leaves the hardware-faithful path at the
        // same 3,151-cycle offset inherited from sub_1B850.
        assert_eq!(engine.master_clock(), 10_273_916 * 7);

        let poll = apply_gems_title_intro_wait_for_vint_poll(&mut engine, controller.cursor)
            .expect("source-positioned first WaitForVint poll");
        assert_eq!(
            poll.timing,
            wildside_core::intro_timing::IntroSourceTiming {
                instruction_count: 4,
                raw_cycles: M68kCycleCount::from_raw(70),
            },
        );
        assert_eq!(poll.refresh_events, 1);
        assert_eq!(poll.external_wait_cycles.get(), 0);
        // Oracle loc_93E begins at 10,277,139. The exact 72-cycle elapsed
        // path retains the inherited 3,151-cycle GEMS offset.
        assert_eq!(engine.master_clock(), 10_273_988 * 7);

        let mode_standard =
            apply_gems_title_wait_for_vint_to_mode_standard(&mut engine, poll.cursor)
                .expect("source-positioned WaitForVint/V_Int/Mode_Standard entry");
        assert_eq!(
            mode_standard.resume,
            wildside_core::WaitForVintResumePath::StaleBranchLoc942,
            "ASM WaitForVint $942 is the saved PC when title-frame VINT asserts",
        );
        assert_eq!(mode_standard.refresh_events, 585);
        assert_eq!(mode_standard.external_wait_cycles.get(), 1_408);
        // Oracle V_Int $488 begins at 10,349,961 and Mode_Standard $77A
        // begins at 10,350,137.
        //
        // ORACLE LIMITATION: the established jgenesis undocumented-X/Y SUB
        // defect shifts GEMS' 68000 instruction phase. The blocking poll
        // absorbs 3,144 of the inherited 3,151-cycle loader deficit, but
        // hardware-faithful execution samples VINT at H40 mclk 41 rather
        // than oracle mclk 69 and reaches Mode_Standard seven cycles early.
        assert_eq!(
            engine.master_clock(),
            10_350_130 * 7,
            "title WaitForVint ledger: {mode_standard:?}",
        );
        assert_eq!(mode_standard.post_assertion_refresh_events, 1);

        let do_nothing =
            apply_gems_title_mode_standard_do_nothing(&mut engine, mode_standard.cursor)
                .expect("source-positioned Mode_Standard Do_Nothing");
        assert_eq!(
            do_nothing.timing,
            wildside_core::intro_timing::IntroSourceTiming {
                instruction_count: 618,
                raw_cycles: M68kCycleCount::from_raw(6_192),
            },
        );
        // Oracle resumes Mode_Standard at $77E on cycle 10,356,427.
        // ORACLE LIMITATION: the carried hardware-faithful refresh phase
        // adds one event, reducing the documented residue from seven to
        // five cycles at this boundary.
        assert_eq!(engine.master_clock(), 10_356_422 * 7);
        assert_eq!(do_nothing.refresh_events, 50);
        assert_eq!(do_nothing.external_wait_cycles.get(), 0);
    }

    #[cfg(feature = "audio")]
    #[test]
    fn timed_gems_start_sequence_matches_sub_e151c_source_schedule() {
        let mut engine = GemsMasterClockTimingEngine::new(&[0x76], vec![]);
        engine.stop_music();
        let entry_master_clock = engine.master_clock();

        let advance = apply_timed_gems_queue_command(
            &mut engine,
            GemsQueueCommandPath::StartOrStopSequenceSubE151c,
            M68kRefreshPhase::new(48).expect("frame-171 selected-PC phase"),
            Some(GemsQueueCommand::StartSequence(gems_bgm::ICE)),
        )
        .expect("ASM Load_TitleScreen -> sub_E151C command schedule");

        assert_eq!(advance.queue_bus_request_complete.get(), 162);
        assert_eq!(advance.queue_write_complete.get(), 424);
        assert_eq!(advance.queue_bus_release_complete.get(), 444);
        assert_eq!(advance.elapsed.get(), 518);
        assert_eq!(advance.phase_after, M68kRefreshPhase::new(54).unwrap());
        assert_eq!(engine.master_clock(), entry_master_clock + 518 * 7);
        assert_eq!(engine.pending_m68k_wait_cycles(), 0);
    }

    #[test]
    fn apply_native_sound_command_uses_raw_gems_command_byte() {
        let mut sink = RecordingAudioSink::default();

        apply_native_sound_command(
            &mut sink,
            NativeSoundCommand::start_sequence(gems_sfx::NAVIGATE_JINGLE),
        );
        apply_native_sound_command(
            &mut sink,
            NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER),
        );

        assert_eq!(
            sink.actions,
            vec![
                RecordingAudioAction::StartSequence(gems_sfx::NAVIGATE_JINGLE),
                RecordingAudioAction::StopSequence(gems_sfx::SCORE_COUNTER),
            ],
            "ASM j_PlaySound uses GEMS $10 and j_PlaySound2 uses GEMS $12"
        );
    }

    #[test]
    fn native_audio_event_formatters_include_frame_context() {
        let trace = trace(GameMode::InGame);

        assert_eq!(
            format_native_bgm_event(
                42,
                &trace,
                Some(NativeBgmCue::TitleShell),
                Some(NativeBgmCue::Level(LevelId(0))),
                NativeBgmPlaybackCommand::StopMusicThenStart(gems_bgm::CAVE),
            ),
            "frame=42 mode=InGame level=00 active_before=Some(TitleShell) active_after=Some(Level(LevelId(0))) command=StopMusicThenStart(7)",
            "BGM trace diagnostics must keep the before/after cue context that explains reset boundaries"
        );
        assert_eq!(
            format_native_sound_event(
                43,
                &trace,
                NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER),
            ),
            "frame=43 mode=InGame level=00 command=StopSequence(12)",
            "sound trace diagnostics must include the raw GEMS command path, not only the sound id"
        );
        assert_eq!(
            format_native_music_speed_event(44, &trace, 0x7A),
            "frame=44 mode=InGame level=00 command=SetMusicSpeed(122)"
        );
    }

    #[test]
    fn queued_gems_helpers_return_deltas_and_reset_spans() {
        let mut prev = trace(GameMode::InGame);
        let mut curr = trace(GameMode::InGame);
        prev.played_sound_effect_ids = vec![gems_sfx::JUMP, gems_sfx::BLOCK_HIT];
        curr.played_sound_effect_ids = vec![
            gems_sfx::JUMP,
            gems_sfx::BLOCK_HIT,
            gems_sfx::NAVIGATE_JINGLE,
        ];

        assert_eq!(
            queued_gems_sound_ids_since(&prev, &curr),
            &[gems_sfx::NAVIGATE_JINGLE],
            "cumulative sound-id traces emit only the appended frame delta"
        );

        curr.played_sound_effect_ids = vec![gems_sfx::SCORE_COUNTER];
        assert_eq!(
            queued_gems_sound_ids_since(&prev, &curr),
            &[gems_sfx::SCORE_COUNTER],
            "shorter current sound-id traces are post-reset spans and replay in full"
        );

        prev.played_sound_commands = vec![
            NativeSoundCommand::start_sequence(gems_sfx::JUMP),
            NativeSoundCommand::start_sequence(gems_sfx::BLOCK_HIT),
        ];
        curr.played_sound_commands = vec![
            NativeSoundCommand::start_sequence(gems_sfx::JUMP),
            NativeSoundCommand::start_sequence(gems_sfx::BLOCK_HIT),
            NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER),
        ];
        assert_eq!(
            queued_gems_sound_commands_since(&prev, &curr),
            vec![NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER)],
            "typed sound-command traces preserve the command-byte delta"
        );

        curr.played_sound_commands =
            vec![NativeSoundCommand::start_sequence(gems_sfx::METAL_BLOCK)];
        assert_eq!(
            queued_gems_sound_commands_since(&prev, &curr),
            vec![NativeSoundCommand::start_sequence(gems_sfx::METAL_BLOCK)],
            "shorter current command traces are post-reset spans and replay in full"
        );

        curr.played_sound_commands.clear();
        assert_eq!(
            queued_gems_sound_commands_since(&prev, &curr),
            vec![NativeSoundCommand::start_sequence(gems_sfx::SCORE_COUNTER)],
            "legacy traces without command bytes fall back to start-sequence commands from sound ids"
        );
    }

    #[test]
    fn queued_gems_music_speed_helpers_return_deltas_and_reset_spans() {
        let mut prev = trace(GameMode::InGame);
        let mut curr = trace(GameMode::InGame);
        prev.played_music_speed_updates = vec![0x7E, 0x7A];
        curr.played_music_speed_updates = vec![0x7E, 0x7A, 0x76];

        assert_eq!(queued_gems_music_speed_updates_since(&prev, &curr), &[0x76]);
        assert_eq!(
            gems_event_music_speed_updates(&prev, &curr),
            vec![0x76],
            "ASM-backed music-speed trace updates are cumulative, so only the new tail should replay"
        );

        curr.played_music_speed_updates = vec![0x74];
        assert_eq!(
            queued_gems_music_speed_updates_since(&prev, &curr),
            &[0x74],
            "shorter current speed traces are post-reset spans and replay in full"
        );
    }

    #[test]
    fn native_audio_sound_classifiers_match_gems_ids() {
        for sound_id in [
            gems_sfx::VOICE_OUCH,
            gems_sfx::VOICE_OUCH_1,
            gems_sfx::VOICE_OUCH_2,
            gems_sfx::VOICE_OUCH_3,
        ] {
            assert!(is_ouch_sound_id(sound_id));
            assert!(is_damage_resolution_voice_sound_id(sound_id));
        }

        assert!(is_damage_resolution_voice_sound_id(gems_sfx::VOICE_DIE));
        for sound_id in [gems_sfx::JUMP, gems_sfx::BLOCK_HIT] {
            assert!(!is_ouch_sound_id(sound_id));
            assert!(!is_damage_resolution_voice_sound_id(sound_id));
        }

        for sound_id in [
            gems_sfx::DESTROY_BLOCK,
            gems_sfx::METAL_BLOCK,
            gems_sfx::RUBBER_BLOCK,
        ] {
            assert!(is_explicit_block_sound_id(sound_id));
        }
        assert!(!is_explicit_block_sound_id(gems_sfx::BLOCK_HIT));

        for sound_id in [
            gems_sfx::IRON_KNIGHT_TRANSFORM,
            gems_sfx::MANIAXE_TRANSFORM,
            gems_sfx::RED_STEALTH_TRANSFORM,
            gems_sfx::BERZERKER_TRANSFORM,
            gems_sfx::SKYCUTTER_TRANSFORM,
            gems_sfx::EYECLOPS_TRANSFORM,
            gems_sfx::CYCLONE_TRANSFORM,
            gems_sfx::MICROMAX_TRANSFORM,
            gems_sfx::JUGGERNAUT_TRANSFORM,
        ] {
            assert!(is_transform_sound_id(sound_id));
        }
        assert!(!is_transform_sound_id(gems_sfx::MANIAXE_THROW));
    }

    #[test]
    fn transform_sound_for_helmet_id_matches_loc_7cd0_table() {
        let expected = [
            (1, gems_sfx::SKYCUTTER_TRANSFORM),
            (2, gems_sfx::CYCLONE_TRANSFORM),
            (3, gems_sfx::RED_STEALTH_TRANSFORM),
            (4, gems_sfx::EYECLOPS_TRANSFORM),
            (5, gems_sfx::JUGGERNAUT_TRANSFORM),
            (6, gems_sfx::IRON_KNIGHT_TRANSFORM),
            (7, gems_sfx::BERZERKER_TRANSFORM),
            (8, gems_sfx::MANIAXE_TRANSFORM),
            (9, gems_sfx::MICROMAX_TRANSFORM),
        ];

        for (helmet_id, sound_id) in expected {
            assert_eq!(
                transform_sound_for_helmet_id_loc_7cd0(helmet_id),
                Some(sound_id),
                "ASM loc_7CD0 maps Current_Helmet through unk_7ED8"
            );
        }
        assert_eq!(transform_sound_for_helmet_id_loc_7cd0(0), None);
        assert_eq!(transform_sound_for_helmet_id_loc_7cd0(10), None);
    }

    #[test]
    fn native_audio_director_initial_bgm_primes_sink_without_duplicate_start() {
        let manifest = test_manifest();
        let mut sink = RecordingAudioSink::default();
        let mut director = NativeAudioDirector::with_initial_bgm(
            &manifest,
            Some(NativeBgmCue::Level(LevelId(0))),
            &mut sink,
        );

        director.step(&manifest, &trace(GameMode::InGame), &mut sink);

        assert_eq!(
            sink.actions,
            vec![
                RecordingAudioAction::StartSequence(gems_bgm::CAVE),
                RecordingAudioAction::StepFrame,
            ],
            "settled audio starts with the already-active level BGM and must not re-run Load_InGame's StopMusic path"
        );
    }

    #[test]
    fn render_native_audio_samples_for_trace_delays_stopmusic_then_start_one_frame() {
        let manifest = test_manifest();
        let intro = trace(GameMode::IntroSequence1);
        let title = trace(GameMode::TitleScreen);
        let mut sink = RecordingAudioSink::default();

        let samples =
            render_native_audio_samples_for_trace(&manifest, &[intro, title], None, &mut sink);

        assert_eq!(samples.len(), 2 * AUDIO_SAMPLES_PER_FRAME * 2);
        assert_eq!(
            sink.actions,
            vec![
                RecordingAudioAction::StopMusic,
                RecordingAudioAction::StepFrame,
                RecordingAudioAction::GenerateSamples(AUDIO_SAMPLES_PER_FRAME),
                RecordingAudioAction::StartSequence(gems_bgm::ICE),
                RecordingAudioAction::StepFrame,
                RecordingAudioAction::GenerateSamples(AUDIO_SAMPLES_PER_FRAME),
            ],
            "ASM Load_IntroSequence1 (kid.asm:29068-29070) runs StopMusic before queuing bgm_Ice"
        );
    }

    #[test]
    fn render_native_audio_samples_for_trace_zero_fills_silent_title_card_loc_1ab26() {
        let manifest = test_manifest();
        let transition = trace(GameMode::LevelTransition);
        let mut sink = RecordingAudioSink::default();

        let samples = render_native_audio_samples_for_trace(
            &manifest,
            &[transition.clone(), transition],
            None,
            &mut sink,
        );

        assert_eq!(samples.len(), 2 * AUDIO_SAMPLES_PER_FRAME * 2);
        assert!(
            samples.iter().all(|sample| *sample == 0.0),
            "ASM Load_TitleCard loc_1AB26/loc_1AE2E has no GEMS command; recorded/native audio should be digital silence, not idle driver output"
        );
        assert!(
            sink.actions.is_empty(),
            "the sink must not step or render samples for a silent cold title-card frame"
        );
    }

    #[test]
    fn native_audio_director_applies_trace_sfx_and_music_speed_deltas() {
        let manifest = test_manifest();
        let prev = trace(GameMode::InGame);
        let mut curr = trace(GameMode::InGame);
        curr.played_sound_effect_ids = vec![gems_sfx::NAVIGATE_JINGLE, gems_sfx::SCORE_COUNTER];
        curr.played_sound_commands = vec![
            NativeSoundCommand::start_sequence(gems_sfx::NAVIGATE_JINGLE),
            NativeSoundCommand::stop_sequence(gems_sfx::SCORE_COUNTER),
        ];
        curr.played_music_speed_updates = vec![0x34];
        let mut sink = RecordingAudioSink::default();
        let mut director = NativeAudioDirector::with_initial_bgm(
            &manifest,
            Some(NativeBgmCue::Level(LevelId(0))),
            &mut sink,
        );
        sink.actions.clear();

        director.step(&manifest, &prev, &mut sink);
        director.step(&manifest, &curr, &mut sink);

        assert_eq!(
            sink.actions,
            vec![
                RecordingAudioAction::StepFrame,
                RecordingAudioAction::StartSequence(gems_sfx::NAVIGATE_JINGLE),
                RecordingAudioAction::StopSequence(gems_sfx::SCORE_COUNTER),
                RecordingAudioAction::SetMusicSpeed(0x34),
                RecordingAudioAction::StepFrame,
            ],
            "ASM-backed GEMS trace commands must be replayed between 60Hz driver steps"
        );
    }

    #[test]
    fn native_audio_director_suppresses_same_frame_sfx_after_transition_stopmusic() {
        let manifest = test_manifest();
        let mut draining = trace(GameMode::InGame);
        draining.level_complete = true;
        draining.level_complete_score_counter_elapsed_frames = 1;
        let mut transition = trace(GameMode::LevelTransition);
        transition.level_complete = true;
        transition.level_complete_elapsed_frames = 1;
        transition.level_complete_score_counter_elapsed_frames = 1;
        transition.level_complete_score_counter_drained = true;
        transition.played_sound_effect_ids = vec![gems_sfx::SCORE_COUNTER];
        transition.played_sound_commands =
            vec![NativeSoundCommand::start_sequence(gems_sfx::SCORE_COUNTER)];
        let mut sink = RecordingAudioSink::default();
        let mut director = NativeAudioDirector::with_initial_bgm(
            &manifest,
            Some(NativeBgmCue::ScoreBoard),
            &mut sink,
        );
        sink.actions.clear();

        director.step(&manifest, &draining, &mut sink);
        director.step(&manifest, &transition, &mut sink);

        assert_eq!(
            sink.actions,
            vec![
                RecordingAudioAction::StepFrame,
                RecordingAudioAction::StopMusic,
                RecordingAudioAction::StepFrame,
            ],
            "ASM loc_D7F4's score-counter replay is pre-reset state when the same condensed LevelTransition frame runs StopMusic"
        );
    }
}
