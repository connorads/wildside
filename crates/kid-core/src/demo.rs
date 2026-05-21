/// Demo playback data extracted from ASM (kid.asm:33373-33436).
///
/// Each byte encodes one Genesis controller state for 2 VBlanks.
/// Bit layout: 7=Start(ignored), 6=A, 5=C, 4=B, 3=Right, 2=Left, 1=Down, 0=Up.
/// Bit 7 is cleared during playback (real Start button overrides it).
///
/// Level: Knights Isle ($15). 498 bytes = 996 frames.
#[rustfmt::skip]
pub(crate) const DEMO_INPUT_DATA_1: [u8; 498] = [
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88,
    0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x48, 0x48,
    0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48,
    0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48,
    0x48, 0x48, 0x48, 0x48, 0x48, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x48, 0x48, 0x48,
    0x48, 0x48, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x04, 0x04,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x1A,
    0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x08, 0x08, 0x08, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x10, 0x10,
    0x00, 0x00, 0x00, 0x05, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x00, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00,
    0x00, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
    0x05, 0x09, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x09, 0x09, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x08, 0x18, 0x18, 0x18, 0x19, 0x18, 0x19, 0x19, 0x19, 0x19, 0x19, 0x10, 0x10, 0x10, 0x10,
    0x10, 0x14, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
    0x11, 0x11, 0x11, 0x19, 0x11, 0x11, 0x11, 0x11, 0x11, 0x19, 0x19, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x08, 0x08, 0x08, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x19, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x05, 0x09,
    0x08, 0x08, 0x08, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x09,
    0x01, 0x09, 0x09, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x48,
    0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x58,
    0x58, 0x59, 0x19, 0x19, 0x09, 0x09, 0x09, 0x29, 0x09, 0x09, 0x09, 0x28, 0x28, 0x29, 0x29, 0x09,
    0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09,
    0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x29, 0x29, 0x29, 0x29, 0x29, 0x29, 0x29, 0x09, 0x09, 0x29,
    0x29, 0x29, 0x09, 0x09, 0x29, 0x29, 0x29, 0x09, 0x09, 0x29, 0x29, 0x29, 0x09, 0x09, 0x29, 0x29,
    0x29, 0x09, 0x09, 0x29, 0x29, 0x09, 0x09, 0x29, 0x29, 0x29, 0x09, 0x09, 0x29, 0x29, 0x29, 0x09,
    0x09, 0x29,
];

/// ORIGINAL_QUIRK: `ReadJoypad` (`loc_C22`) advances the raw demo pointer with
/// no table-end guard. Demo 1 times out after `Demo_InputData1`, so the ROM
/// reads six extra bytes from the immediately following `byte_1E6E3` Dark Alley
/// art blob before the timeout handoff returns to title.
pub(crate) const DEMO_INPUT_DATA_1_OVERFLOW: [u8; 6] = [0x06, 0xC4, 0xC7, 0xFF, 0x9A, 0x14];

/// Level: Skydragon Castle 1 ($1B). 499 bytes = 998 frames.
#[rustfmt::skip]
pub(crate) const DEMO_INPUT_DATA_2: [u8; 499] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x40, 0x40, 0x48, 0x48, 0x48, 0x48, 0x48, 0x58,
    0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x48, 0x48, 0x48, 0x58, 0x58, 0x58, 0x48, 0x48,
    0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x58, 0x58, 0x50, 0x50, 0x50, 0x50, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x44,
    0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x41, 0x40, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48,
    0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x58, 0x58,
    0x58, 0x58, 0x58, 0x58, 0x50, 0x10, 0x20, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x20, 0x00, 0x00,
    0x20, 0x20, 0x00, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20,
    0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x00,
    0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20,
    0x20, 0x00, 0x00, 0x20, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x00,
    0x00, 0x20, 0x20, 0x00, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x20,
    0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00, 0x20, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10, 0x00, 0x00, 0x00, 0x00, 0x20,
    0x00, 0x00, 0x00, 0x20, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x00, 0x00, 0x00, 0x20, 0x20, 0x04,
    0x04, 0x24, 0x24, 0x04, 0x04, 0x24, 0x24, 0x04, 0x04, 0x24, 0x24, 0x04, 0x04, 0x04, 0x24, 0x24,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x48, 0x48, 0x48, 0x48,
    0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x40, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
];

/// Demo 1 plays level Knights Isle (0x15).
pub(crate) const DEMO_1_LEVEL: u8 = 0x15;
/// Demo 2 plays level Skydragon Castle 1 (0x1B).
pub(crate) const DEMO_2_LEVEL: u8 = 0x1B;
/// Title screen idle timeout before demo starts (ASM: $280 = 640). The
/// timeout-restart `IntroSequence6` lineage adds its extra DBF-backed hold in
/// `boot.rs`.
pub(crate) const TITLE_IDLE_FRAMES: u16 = 640;
/// Demo auto-ends after this many frames (ASM: $3F2 = 1010).
pub(crate) const DEMO_MAX_FRAMES: u16 = 1010;
/// Oracle-backed demo-mode traces keep the title pose visible for 27 frames
/// after the mode flip before the demo level bootstrap begins.
pub(crate) const DEMO_TITLE_POSE_FRAMES: u16 = 27;
/// Current level switches from title (0x00) to the demo level on this demo
/// startup frame.
pub(crate) const DEMO_LEVEL_SWITCH_FRAME: u16 = 30;
/// Demo bootstrap briefly exposes a camera-only preload pose before gameplay.
pub(crate) const DEMO_CAMERA_PRELOAD_FRAME: u16 = 57;
/// Oracle-backed Skydragon Castle demo preload row on the hidden frame before
/// `Load_InGame`'s camera snap becomes visible on the following hold frame.
/// Oracle WRAM reads `Camera_Y_pos=$0228_0000` on this preload frame for both
/// the plain initial boot and the Start-skip intro-return lineage.
pub(crate) const DEMO_2_CAMERA_PRELOAD_Y_FP: u32 = 0x0228_0000;
/// The direct timeout-restart intro-sequence-5 title return reaches the same
/// demo-2 preload frame through a different hidden title shell. Oracle WRAM
/// shows `Camera_Y_pos=$0220_0000` only for that `sub_1C246 -> loc_1C264 ->
/// Obj_TitleMenu loc_1BB02` lineage.
pub(crate) const DEMO_2_INTRO_SEQUENCE_5_TITLE_RETURN_CAMERA_PRELOAD_Y_FP: u32 = 0x0220_0000;
/// Oracle-backed timeout-restart `IntroSequence6` demo-1 hidden wrap. ASM
/// `loc_1BA5C -> Load_IntroSequence6 -> loc_1BB02 -> Load_DemoPlay ->
/// Load_InGame` leaves a different wrapped preload row than both the
/// cold-boot demo (`0xFF00_0000`) and the earlier Start-skip intro return
/// (`0xFF20_0000`).
pub(crate) const DEMO_1_INTRO_SEQUENCE_6_CAMERA_WRAP_Y_FP: u32 = 0xFD20_0000;
/// Demo bootstrap holds the level camera on the respawn pose for these frames.
pub(crate) const DEMO_CAMERA_HOLD_START_FRAME: u16 = 58;
/// Demo bootstrap publishes widened object counts one frame before the player
/// becomes visible.
pub(crate) const DEMO_OBJECT_WARMUP_FRAME: u16 = 64;
/// First demo startup frame where gameplay, physics, and recorded inputs apply.
pub(crate) const DEMO_GAMEPLAY_START_FRAME: u16 = 65;
/// Demo 1 (Knights Isle) keeps the player hidden and inert seven frames longer
/// than demo 2 before exposing gameplay state in RAM.
pub(crate) const DEMO_1_GAMEPLAY_START_FRAME: u16 = 72;
/// Oracle-backed published TAS traces keep the Knights Isle demo auto-timeout
/// compare sixty-eight frames behind the raw startup frame counter. `loc_C22`
/// compares the stored timeout counter before incrementing it, and oracle RAM
/// keeps `($FFFFFBC2).w` at `0x03B2` on published frame 8004 and only reaches
/// the `$3F2` timeout threshold on frame 8068.
pub(crate) const DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES: u16 = 68;
/// Oracle-backed published TAS probes show the Skydragon Castle demo timeout
/// counter stays 61 frames behind the native startup counter. Keep the raw
/// startup frame counter for visible choreography, but subtract this offset
/// when modelling the ROM's demo auto-end compare.
pub(crate) const DEMO_2_TIMEOUT_COUNTER_OFFSET_FRAMES: u16 = 61;
/// Hidden bootstrap frame where the player drops the missing spawn pixel before
/// the grounded hand-off begins.
pub(crate) const DEMO_BOOTSTRAP_INIT_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 1;
/// First grounded bootstrap frame after the hidden settle pixel and short
/// free-fall hand-off complete.
pub(crate) const DEMO_BOOTSTRAP_GROUNDED_FRAME: u16 = DEMO_BOOTSTRAP_INIT_FRAME + 4;
/// Native roster counts lag the oracle by one hidden system object during the
/// first visible demo bootstrap frames, even after regular gameplay physics
/// takes over from the hidden settle path.
pub(crate) const DEMO_OBJECT_BOOTSTRAP_END_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 16;
/// Demo 1 keeps the grounded hand-off pinned through startup frame 130 before
/// the first visible floor-release step. ASM `loc_8E16 -> sub_8F26 ->
/// sub_902A` moves horizontally first, but oracle WRAM still reports the
/// Load_InGame floor Y at published TAS frame 2665.
pub(crate) const DEMO_1_GROUNDED_RUN_HANDOFF_END_FRAME: u16 = DEMO_OBJECT_BOOTSTRAP_END_FRAME + 49;
/// The timeout-restart IntroSequence6 -> demo-1 path carries the same
/// grounded-run hand-off one frame further. Oracle frame 21896 is startup
/// frame 131 and still reports the old `sub_902A` floor result while X has
/// advanced through `sub_8F26`.
pub(crate) const DEMO_1_TIMEOUT_RESTART_GROUNDED_RUN_HANDOFF_END_FRAME: u16 =
    DEMO_1_GROUNDED_RUN_HANDOFF_END_FRAME + 1;
/// Final demo-1 walking frame where the ROM performs the `sub_8F26` horizontal
/// step while still holding the previous `sub_902A` floor result. The following
/// frame drops through the walking no-support path (`loc_B5F6`) before the next
/// horizontal movement becomes visible.
pub(crate) const DEMO_1_FINAL_HORIZONTAL_FLOOR_RELEASE_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 93;
/// The matching final horizontal hand-off on the timeout-restart
/// IntroSequence6 -> demo-1 path is one startup frame later.
pub(crate) const DEMO_1_TIMEOUT_RESTART_FINAL_HORIZONTAL_FLOOR_RELEASE_FRAME: u16 =
    DEMO_1_FINAL_HORIZONTAL_FLOOR_RELEASE_FRAME + 1;
/// Later demo-1 walking frame with the same horizontal-before-vertical release
/// ordering: `sub_8F26` advances X, then the following frame exposes the
/// `loc_B5F6` no-support Y increment.
pub(crate) const DEMO_1_LATE_HORIZONTAL_FLOOR_RELEASE_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 148;
/// The timeout-restart IntroSequence6 -> demo-1 path carries the same late
/// horizontal hand-off one startup frame later.
pub(crate) const DEMO_1_TIMEOUT_RESTART_LATE_HORIZONTAL_FLOOR_RELEASE_FRAME: u16 =
    DEMO_1_LATE_HORIZONTAL_FLOOR_RELEASE_FRAME + 1;
/// Demo-1 resumes a second live jump arc after the long controller-sampled
/// idle window. ASM `loc_A6F8 -> sub_B168 -> sub_902A -> sub_8F26` enters with
/// `y_vel=$FFFCE000`, applies held-jump gravity to `$FFFD0000`, then moves
/// upward on startup frame 252.
pub(crate) const DEMO_1_LATE_JUMP_REENTRY_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 187;
/// Last demo-1 frame where oracle top-level TAS state still samples Kid
/// velocity from the hidden `loc_119D8` controller object rather than the live
/// Kid object. Live physics continues independently.
pub(crate) const DEMO_1_LATE_CONTROLLER_VELOCITY_SAMPLE_END_FRAME: u16 =
    DEMO_1_GAMEPLAY_START_FRAME + 514;
/// Last demo-1 frame where oracle top-level TAS state still samples object
/// counts from the hidden `loc_119D8` controller-side lineage.
pub(crate) const DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME: u16 =
    DEMO_1_GAMEPLAY_START_FRAME + 546;
/// Timeout-restarted demo-1 keeps sampling the hidden `loc_119D8` controller
/// object through the auto-timeout edge. The ROM `ReadJoypad` counter compare
/// uses `$3F2`, but native's frame counter is offset to match oracle WRAM.
pub(crate) const DEMO_1_TIMEOUT_RESTART_CONTROLLER_SAMPLE_END_FRAME: u16 =
    DEMO_MAX_FRAMES + DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES + 84;
/// Timeout-restart demo-1 exposes one additional object-only controller-side
/// worker after the initial extended `loc_119D8` sample settles.
pub(crate) const DEMO_1_TIMEOUT_RESTART_EXTRA_CONTROLLER_OBJECT_FRAME: u16 =
    DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME + 32;
/// Direct timeout-restart intro-sequence-5 title-return demo-2 keeps sampling
/// the hidden `Load_DemoPlay -> loc_119D8` controller object for top-level TAS
/// velocity after the live Cyclone Kid object has started its upward arc.
pub(crate) const DEMO_2_INTRO_SEQUENCE_5_TITLE_RETURN_CONTROLLER_SAMPLE_START_FRAME: u16 =
    DEMO_GAMEPLAY_START_FRAME + 424;
/// The matching controller-sampled velocity window reaches the adjusted demo-2
/// auto-timeout edge. `ReadJoypad loc_C0A` owns the top-level sample while raw
/// Kid WRAM continues moving under the controller shell.
pub(crate) const DEMO_2_INTRO_SEQUENCE_5_TITLE_RETURN_CONTROLLER_SAMPLE_END_FRAME: u16 =
    DEMO_MAX_FRAMES + DEMO_2_TIMEOUT_COUNTER_OFFSET_FRAMES;
/// After the long controller-sampled demo-1 arc, native's trace-only hold has
/// the oracle live pose but must restore the raw Kid `loc_A6F8` fall velocity
/// before the next `sub_B168` gravity step.
pub(crate) const DEMO_1_LATE_FALL_VELOCITY_RESTORE_FRAME: u16 = DEMO_1_GAMEPLAY_START_FRAME + 451;
/// First late demo-1 controller-sample count wake in the top-level TAS sample.
pub(crate) const DEMO_1_LATE_CONTROLLER_COUNT_FIRST_WAKE_FRAME: u16 =
    DEMO_1_GAMEPLAY_START_FRAME + 301;
/// Last frame of the first late demo-1 controller-sample count wake.
pub(crate) const DEMO_1_LATE_CONTROLLER_COUNT_FIRST_SLEEP_FRAME: u16 =
    DEMO_1_GAMEPLAY_START_FRAME + 368;
/// Second late demo-1 controller-sample count transition: one non-gfx worker wakes.
pub(crate) const DEMO_1_LATE_CONTROLLER_COUNT_SECOND_NON_GFX_FRAME: u16 =
    DEMO_1_GAMEPLAY_START_FRAME + 438;
/// Second late demo-1 controller-sample count transition: the matching gfx worker wakes.
pub(crate) const DEMO_1_LATE_CONTROLLER_COUNT_SECOND_GFX_FRAME: u16 =
    DEMO_1_GAMEPLAY_START_FRAME + 439;
/// Third late demo-1 controller-side worker wakes after the live Kid lands.
pub(crate) const DEMO_1_LATE_CONTROLLER_COUNT_THIRD_WAKE_FRAME: u16 =
    DEMO_1_GAMEPLAY_START_FRAME + 457;
/// Oracle top-level demo-1 trace reports the pause-menu latch after the
/// controller object has consumed the recorded Start edge. Native keeps this
/// as trace state; live gameplay has already matched the raw Kid/object WRAM.
pub(crate) const DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME: u16 = DEMO_1_GAMEPLAY_START_FRAME + 515;
pub(crate) const DEMO_1_LATE_CONTROLLER_PAUSE_END_FRAME: u16 =
    DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME;
/// Late demo-1 landing stall keeps `Character_Movement=MoveID_Jump` after
/// floor contact, until the controller-sampled object settles on the next
/// frame.
pub(crate) const DEMO_1_LATE_JUMP_MOVEMENT_HOLD_START_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 294;
pub(crate) const DEMO_1_LATE_JUMP_MOVEMENT_HOLD_END_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 314;
/// Published TAS trace sampling keeps the demo player's velocity field on the
/// held respawn value after raw WRAM has started moving. ASM `Load_DemoPlay`
/// allocates the `loc_119D8` controller object before `Load_InGame`, and
/// `loc_C22` updates the demo input stream outside the gameplay object body;
/// native keeps this as a trace-only hold until the first later live movement
/// miss so the simulation can still advance.
pub(crate) const DEMO_TRACE_GROUNDED_HOLD_END_FRAME: u16 = DEMO_1_LATE_JUMP_REENTRY_FRAME;
/// After Kid_Transform exits in demo mode, oracle WRAM keeps Kid's sampled
/// velocity fields at zero while the visible `kid_gfx` object continues moving
/// for the short controller hand-off window. Native models that as trace-only
/// export masking; physics still advances with the real player velocity.
pub(crate) const DEMO_POST_TRANSFORM_TRACE_VELOCITY_HOLD_FRAMES: u8 = 17;
/// Oracle-backed published TAS traces keep Character_Movement at
/// `MoveID_Jump` through the demo-1 wall-jump landing stall, then restore
/// `MoveID_Standingstill` one startup frame before the velocity trace hold
/// expires.
pub(crate) const DEMO_TRACE_WALL_JUMP_MOVEMENT_HOLD_END_FRAME: u16 = DEMO_GAMEPLAY_START_FRAME + 75;

pub(crate) fn gameplay_start_frame(use_demo_2: bool) -> u16 {
    if use_demo_2 {
        DEMO_GAMEPLAY_START_FRAME
    } else {
        DEMO_1_GAMEPLAY_START_FRAME
    }
}

pub(crate) fn enemy_warmup_frame(use_demo_2: bool) -> u16 {
    if use_demo_2 {
        DEMO_GAMEPLAY_START_FRAME
    } else {
        DEMO_1_GAMEPLAY_START_FRAME + 1
    }
}

pub(crate) fn demo_1_hidden_camera_y(frame_counter: u16) -> Option<u32> {
    match frame_counter {
        57 | 58 => Some(0),
        59 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 70 | 71 => Some(0x00E0_0000),
        // ORIGINAL_QUIRK: demo 1 briefly exposes a wrapped negative camera Y
        // one frame before the hidden Knights Isle bootstrap settles.
        60 => Some(0xFF00_0000),
        _ => None,
    }
}

/// Decode a demo data byte into a FrameInput.
/// Bit 7 (Start) is stripped — the real Start button state is applied separately.
pub(crate) fn decode_demo_byte(byte: u8) -> super::FrameInput {
    super::FrameInput::from_mask(byte & 0x7F)
}

pub(crate) fn demo_byte(use_demo_2: bool, data_index: usize) -> u8 {
    if use_demo_2 {
        DEMO_INPUT_DATA_2.get(data_index).copied().unwrap_or(0)
    } else if let Some(byte) = DEMO_INPUT_DATA_1.get(data_index).copied() {
        byte
    } else {
        DEMO_INPUT_DATA_1_OVERFLOW
            .get(data_index - DEMO_INPUT_DATA_1.len())
            .copied()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameInput;

    #[test]
    fn loc_c22_playback_bootstrap_frame_boundaries_are_oracle_backed() {
        assert_eq!(DEMO_BOOTSTRAP_INIT_FRAME, DEMO_GAMEPLAY_START_FRAME + 1);
        assert_eq!(DEMO_BOOTSTRAP_GROUNDED_FRAME, DEMO_BOOTSTRAP_INIT_FRAME + 4);
        assert_eq!(
            DEMO_OBJECT_BOOTSTRAP_END_FRAME,
            DEMO_GAMEPLAY_START_FRAME + 16
        );
        assert_eq!(DEMO_BOOTSTRAP_INIT_FRAME, 66);
        assert_eq!(DEMO_BOOTSTRAP_GROUNDED_FRAME, 70);
        assert_eq!(DEMO_OBJECT_BOOTSTRAP_END_FRAME, 81);
        assert_eq!(DEMO_1_GROUNDED_RUN_HANDOFF_END_FRAME, 130);
        assert_eq!(DEMO_1_TIMEOUT_RESTART_GROUNDED_RUN_HANDOFF_END_FRAME, 131);
        assert_eq!(DEMO_1_FINAL_HORIZONTAL_FLOOR_RELEASE_FRAME, 158);
        assert_eq!(
            DEMO_1_TIMEOUT_RESTART_FINAL_HORIZONTAL_FLOOR_RELEASE_FRAME,
            159
        );
        assert_eq!(DEMO_1_LATE_HORIZONTAL_FLOOR_RELEASE_FRAME, 213);
        assert_eq!(
            DEMO_1_TIMEOUT_RESTART_LATE_HORIZONTAL_FLOOR_RELEASE_FRAME,
            214
        );
        assert_eq!(DEMO_1_LATE_JUMP_REENTRY_FRAME, 252);
        assert_eq!(DEMO_1_LATE_CONTROLLER_COUNT_FIRST_WAKE_FRAME, 373);
        assert_eq!(DEMO_1_LATE_CONTROLLER_COUNT_FIRST_SLEEP_FRAME, 440);
        assert_eq!(DEMO_1_LATE_CONTROLLER_COUNT_SECOND_NON_GFX_FRAME, 510);
        assert_eq!(DEMO_1_LATE_CONTROLLER_COUNT_SECOND_GFX_FRAME, 511);
        assert_eq!(DEMO_1_LATE_CONTROLLER_VELOCITY_SAMPLE_END_FRAME, 586);
        assert_eq!(DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME, 618);
        assert_eq!(DEMO_1_TIMEOUT_RESTART_CONTROLLER_SAMPLE_END_FRAME, 1162);
        assert_eq!(DEMO_1_TIMEOUT_RESTART_EXTRA_CONTROLLER_OBJECT_FRAME, 650);
        assert_eq!(DEMO_1_LATE_FALL_VELOCITY_RESTORE_FRAME, 523);
        assert_eq!(DEMO_1_LATE_CONTROLLER_COUNT_THIRD_WAKE_FRAME, 529);
        assert_eq!(DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME, 587);
        assert_eq!(DEMO_1_LATE_CONTROLLER_PAUSE_END_FRAME, 618);
        assert_eq!(DEMO_1_LATE_JUMP_MOVEMENT_HOLD_START_FRAME, 359);
        assert_eq!(DEMO_1_LATE_JUMP_MOVEMENT_HOLD_END_FRAME, 379);
        assert_eq!(DEMO_TRACE_GROUNDED_HOLD_END_FRAME, 252);
        assert_eq!(DEMO_POST_TRANSFORM_TRACE_VELOCITY_HOLD_FRAMES, 17);
    }

    #[test]
    fn loc_c22_playback_start_and_enemy_warmup_frames_split_by_recording() {
        assert_eq!(gameplay_start_frame(true), DEMO_GAMEPLAY_START_FRAME);
        assert_eq!(gameplay_start_frame(false), DEMO_1_GAMEPLAY_START_FRAME);
        assert_eq!(enemy_warmup_frame(true), DEMO_GAMEPLAY_START_FRAME);
        assert_eq!(enemy_warmup_frame(false), DEMO_1_GAMEPLAY_START_FRAME + 1);
        assert_eq!(gameplay_start_frame(true), 65);
        assert_eq!(gameplay_start_frame(false), 72);
        assert_eq!(enemy_warmup_frame(false), 73);
    }

    #[test]
    fn loc_c22_knights_isle_hidden_camera_rows_keep_the_wrap_quirk() {
        assert_eq!(demo_1_hidden_camera_y(56), None);
        assert_eq!(demo_1_hidden_camera_y(57), Some(0));
        assert_eq!(demo_1_hidden_camera_y(58), Some(0));
        assert_eq!(demo_1_hidden_camera_y(59), Some(0x00E0_0000));
        assert_eq!(demo_1_hidden_camera_y(60), Some(0xFF00_0000));
        assert_eq!(demo_1_hidden_camera_y(61), Some(0x00E0_0000));
        assert_eq!(demo_1_hidden_camera_y(71), Some(0x00E0_0000));
        assert_eq!(demo_1_hidden_camera_y(72), None);
    }

    #[test]
    fn loc_c22_input_bytes_strip_start_before_frame_input_decode() {
        assert_eq!(
            decode_demo_byte(0xFF),
            FrameInput {
                up: true,
                down: true,
                left: true,
                right: true,
                a: true,
                b: true,
                c: true,
                start: false,
            }
        );
        assert_eq!(
            decode_demo_byte(0x80),
            FrameInput::default(),
            "ASM loc_C22 clears the recorded Start bit before normal button decode"
        );
    }

    #[test]
    fn loc_c22_reads_recorded_bytes_and_the_overflow_tail() {
        assert_eq!(demo_byte(true, 7), 0x40);
        assert_eq!(demo_byte(true, DEMO_INPUT_DATA_2.len()), 0);
        assert_eq!(demo_byte(false, 0), 0x80);
        assert_eq!(demo_byte(false, DEMO_INPUT_DATA_1.len()), 0x06);
        assert_eq!(demo_byte(false, DEMO_INPUT_DATA_1.len() + 2), 0xC7);
        assert_eq!(
            demo_byte(
                false,
                DEMO_INPUT_DATA_1.len() + DEMO_INPUT_DATA_1_OVERFLOW.len()
            ),
            0,
            "ORIGINAL_QUIRK loc_C22 only has the adjacent six ROM bytes before native clamps to zero"
        );
    }
}
