/// Real input-driven physics translated from kid.asm.
///
/// All velocities and positions use the same fixed-point 16.16 format as the
/// original 68k code: the upper 16 bits are the pixel value, the lower 16 are
/// the fractional part.
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Direction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// Speed tables — direct transcription of unk_9C0A / unk_9C7A / unk_9CCE
// ---------------------------------------------------------------------------

/// Seven longwords per variant, matching the 68k struct layout at byte offsets
/// $00–$18.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeedTable {
    /// $00 — maximum walking speed (positive magnitude)
    pub max_walk_speed: i32,
    /// $04 — acceleration while walking (per frame)
    pub walk_accel: i32,
    /// $08 — deceleration when no input held, and when above walk cap while
    ///        walking (per frame)
    pub walk_run_decel: i32,
    /// $0C — brake rate when turning around without run held (facing right,
    ///        moving left)
    pub turn_brake: i32,
    /// $10 — maximum running speed (positive magnitude)
    pub max_run_speed: i32,
    /// $14 — acceleration while running (per frame)
    pub run_accel: i32,
    /// $18 — brake rate for the opposite-direction case while running, or when
    ///        facing left and moving right without run
    pub strong_brake: i32,
}

// --- Normal terrain (unk_9C0A) ---
// ORIGINAL_QUIRK: ROM has a 4th entry at offset +$54 ($20000/$4000/$2800/$4000/$30000/$3800/$6000)
// reachable only if both Micromax AND Juggernaut checks match simultaneously — impossible since
// only one helmet is worn at a time. Dead data; likely a cut form or dev remnant. (backlog #039)

pub const NORMAL_KID: SpeedTable = SpeedTable {
    max_walk_speed: 0x2_0000,
    walk_accel: 0x1000,
    walk_run_decel: 0x2800,
    turn_brake: 0x2000,
    max_run_speed: 0x3_8000,
    run_accel: 0x1C00,
    strong_brake: 0x3000,
};

pub const NORMAL_MICROMAX: SpeedTable = SpeedTable {
    max_walk_speed: 0x1_0000,
    walk_accel: 0x800,
    walk_run_decel: 0x1400,
    turn_brake: 0x1000,
    max_run_speed: 0x2_0000,
    run_accel: 0x1000,
    strong_brake: 0x1800,
};

pub const NORMAL_JUGGERNAUT: SpeedTable = SpeedTable {
    max_walk_speed: 0x1_8000,
    walk_accel: 0x2000,
    walk_run_decel: 0x5000,
    turn_brake: 0x4000,
    max_run_speed: 0x3_0000,
    run_accel: 0x3800,
    strong_brake: 0x6000,
};

// --- Ice terrain (unk_9C7A) ---

pub const ICE_KID: SpeedTable = SpeedTable {
    max_walk_speed: 0x1_8000,
    walk_accel: 0x2000,
    walk_run_decel: 0x5000,
    turn_brake: 0x4000,
    max_run_speed: 0x2_8000,
    run_accel: 0x3800,
    strong_brake: 0x6000,
};

pub const ICE_MICROMAX: SpeedTable = SpeedTable {
    max_walk_speed: 0xC000,
    walk_accel: 0x1000,
    walk_run_decel: 0x2800,
    turn_brake: 0x2000,
    max_run_speed: 0x1_8000,
    run_accel: 0x2000,
    strong_brake: 0x3000,
};

pub const ICE_JUGGERNAUT: SpeedTable = SpeedTable {
    max_walk_speed: 0x1_4000,
    walk_accel: 0x4000,
    walk_run_decel: 0xA000,
    turn_brake: 0x8000,
    max_run_speed: 0x2_0000,
    run_accel: 0x7000,
    strong_brake: 0xC000,
};

// --- Rubber terrain (unk_9CCE) ---

pub const RUBBER_KID: SpeedTable = SpeedTable {
    max_walk_speed: 0x4_0000,
    walk_accel: 0x200,
    walk_run_decel: 0x100,
    turn_brake: 0x400,
    max_run_speed: 0x7_0000,
    run_accel: 0x380,
    strong_brake: 0x680,
};

pub const RUBBER_MICROMAX: SpeedTable = SpeedTable {
    max_walk_speed: 0x2_0000,
    walk_accel: 0x100,
    walk_run_decel: 0x80,
    turn_brake: 0x200,
    max_run_speed: 0x4_0000,
    run_accel: 0x200,
    strong_brake: 0x380,
};

pub const RUBBER_JUGGERNAUT: SpeedTable = SpeedTable {
    max_walk_speed: 0x3_0000,
    walk_accel: 0x400,
    walk_run_decel: 0x200,
    turn_brake: 0x800,
    max_run_speed: 0x5_8000,
    run_accel: 0x700,
    strong_brake: 0xD00,
};

// ---------------------------------------------------------------------------
// Air speed tables — kid.asm:9301 (sub_8506) / unk_85C4..unk_85EC
// ---------------------------------------------------------------------------

/// Five longwords per terrain variant, matching the 68k struct layout at
/// byte offsets $00–$10 used by sub_8506 (airborne horizontal movement).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AirSpeedTable {
    /// $00 — maximum air speed when pressing a direction (positive magnitude)
    pub max_air_speed: i32,
    /// $04 — acceleration when pressing direction and below max_air_speed
    pub air_accel: i32,
    /// $08 — deceleration when above max_air_speed (towards max_air_speed)
    pub air_above_max_decel: i32,
    /// $0C — deceleration when no direction is held (towards zero)
    pub air_no_input_decel: i32,
    /// $10 — acceleration when turning (pressing opposite direction to velocity)
    pub air_turn_accel: i32,
}

/// Hard cap on airborne horizontal speed — inline constant in sub_8506.
pub const AIR_HARD_CAP: i32 = 0x3_8000;

// --- Normal terrain (unk_85C4) ---

pub const AIR_NORMAL: AirSpeedTable = AirSpeedTable {
    max_air_speed: 0x1_0000,
    air_accel: 0x1000,
    air_above_max_decel: 0x1000,
    air_no_input_decel: 0x1800,
    air_turn_accel: 0x4000,
};

// --- Ice terrain (unk_85D8) ---

pub const AIR_ICE: AirSpeedTable = AirSpeedTable {
    max_air_speed: 0x8000,
    air_accel: 0x2000,
    air_above_max_decel: 0x1000,
    air_no_input_decel: 0x3000,
    air_turn_accel: 0x8000,
};

// --- Rubber terrain (unk_85EC) ---

pub const AIR_RUBBER: AirSpeedTable = AirSpeedTable {
    max_air_speed: 0x1_8000,
    air_accel: 0x200,
    air_above_max_decel: 0x200,
    air_no_input_decel: 0x100,
    air_turn_accel: 0x800,
};

/// Per-helmet airborne control used while jumping (sub_B084 / unk_A4B2 / unk_A4C6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpAirProfile {
    /// Base horizontal acceleration while airborne.
    pub accel: i32,
    /// Maximum horizontal jump speed.
    pub max_speed: i32,
}

pub const JUMP_AIR_PROFILES: [JumpAirProfile; 10] = [
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Kid
    JumpAirProfile {
        accel: 0x3400,
        max_speed: 0x32000,
    }, // Skycutter
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Cyclone
    JumpAirProfile {
        accel: 0x4000,
        max_speed: 0x2A000,
    }, // Red Stealth
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Eyeclops
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Juggernaut
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Iron Knight
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Berzerker
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Maniaxe
    JumpAirProfile {
        accel: 0x2B00,
        max_speed: 0x2A000,
    }, // Micromax
];

/// Update horizontal velocity for one frame while airborne.
///
/// Direct translation of sub_8506 (kid.asm:9301). Unlike ground movement,
/// air control ignores walk/run distinction — there is a single max speed,
/// a single acceleration, and the only state that matters is direction.
///
/// ASM loc_8518 (kid.asm:9309): facing-right branch — checks Ctrl_Left_Held
/// first (turn to face left → loc_8572), then Ctrl_Right_Held (forward
/// acceleration → loc_852C), then no-input decel → loc_85AC. The mirrored
/// loc_8560 path (facing left) checks Ctrl_Right_Held first. Native collapses
/// both facing paths: `input.right` → right accel, `input.left` → left accel,
/// neither → decel. Direction flip is implicit in the returned facing.
///
/// ASM loc_8524 (kid.asm:9316): facing-right, left not held — checks
/// Ctrl_Right_Held. If held → fall through to loc_852C (accelerate right).
/// If not held → `beq.w loc_85AC` (decelerate, no input).
///
/// Returns `(new_x_vel, new_facing)`.
pub fn update_air_horizontal_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    table: &AirSpeedTable,
) -> (i32, Direction) {
    // ASM loc_8518/loc_8560: facing-dependent input priority collapses to
    // direction-based dispatch since both L+R cannot be held simultaneously.
    if input.right {
        (apply_air_holding_right(x_vel, table), Direction::Right)
    } else if input.left {
        (apply_air_holding_left(x_vel, table), Direction::Left)
    } else {
        // ASM loc_8524/loc_856C: neither direction held → loc_85AC decel.
        (air_decelerate(x_vel, table), facing)
    }
}

/// Player airborne control while jumping (sub_B084).
///
/// ASM loc_B09E (kid.asm:13930): non-Skycutter entry — loads per-helmet accel
/// from unk_A4B2, applies 1.5x multiplier when run (Ctrl_A) is held.
pub fn update_jump_horizontal_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    profile: JumpAirProfile,
) -> (i32, Direction) {
    // ASM loc_B09E: `lea unk_A4B2(pc),a0; move.w (a0,d0.w),d1; ext.l d1`
    // then `tst.b (Ctrl_A_Held).w; beq.s loc_B0B6; asr.l #1,d2; add.l d2,d1`
    let accel = if input.run {
        profile.accel + (profile.accel >> 1)
    } else {
        profile.accel
    };

    // ASM loc_B0B6 (kid.asm:13941): load max speed from unk_A4C6, then
    // dispatch on Ctrl_Left_Held / Ctrl_Right_Held / neither.
    let new_vel = if input.left {
        // ASM loc_B0B6 left path: `st x_direction(a3); sub.l d1,d0; bpl.s loc_B118;
        // neg.l d2; cmp.l d0,d2; ble.s loc_B118; add.l d1,d0; addi.l #$100,d0`
        let next = x_vel - accel;
        if next >= 0 {
            next
        } else {
            let min_speed = -profile.max_speed;
            if min_speed <= next {
                next
            } else {
                x_vel + 0x100
            }
        }
    } else if input.right {
        // ASM loc_B0E2 (kid.asm:13959): `sf x_direction(a3); add.l d1,d0;
        // bmi.s loc_B118; cmp.l d0,d2; bge.s loc_B118; sub.l d1,d0;
        // subi.l #$100,d0`
        let next = x_vel + accel;
        if next <= profile.max_speed {
            next
        } else {
            x_vel - 0x100
        }
    } else {
        match x_vel.cmp(&0) {
            std::cmp::Ordering::Greater => {
                // ASM loc_B0FE (kid.asm:13972): no L/R input, positive velocity —
                // `subi.l #$400,d0; bpl.s loc_B118; moveq #0,d0`
                (x_vel - 0x400).max(0)
            }
            std::cmp::Ordering::Less => {
                // ASM loc_B10E (kid.asm:13981): no L/R input, negative velocity —
                // `addi.l #$400,d0; bmi.s loc_B118; moveq #0,d0`
                (x_vel + 0x400).min(0)
            }
            std::cmp::Ordering::Equal => 0,
        }
    };

    let new_facing = if input.left {
        Direction::Left
    } else if input.right {
        Direction::Right
    } else {
        facing
    };
    (new_vel, new_facing)
}

/// Airborne, holding right — loc_852C..loc_855E.
fn apply_air_holding_right(vel: i32, table: &AirSpeedTable) -> i32 {
    if vel < 0 {
        // Moving left while holding right: turn with air_turn_accel ($10)
        vel + table.air_turn_accel
    } else {
        // ASM loc_8536 (kid.asm:9327): cmpi.l #$38000,d0; ble.s loc_8544;
        // move.l #$38000,d0 — hard cap positive velocity at AIR_HARD_CAP.
        let vel = vel.min(AIR_HARD_CAP);
        if vel < table.max_air_speed {
            // ASM loc_8544 (kid.asm:9332): cmp.l (a0),d0; bge.s loc_8554;
            // add.l 4(a0),d0 — below max, accelerate by air_accel, clamp to max.
            (vel + table.air_accel).min(table.max_air_speed)
        } else {
            // ASM loc_8554 (kid.asm:9344): sub.l 8(a0),d0; cmp.l (a0),d0;
            // bge.s return_855E; move.l (a0),d0 — at/above max, decelerate
            // by air_above_max_decel, clamp floor to max.
            (vel - table.air_above_max_decel).max(table.max_air_speed)
        }
    }
}

/// Airborne, holding left — loc_8572..loc_85A8. Mirror of right.
fn apply_air_holding_left(vel: i32, table: &AirSpeedTable) -> i32 {
    if vel > 0 {
        // Moving right while holding left: turn with air_turn_accel ($10)
        vel - table.air_turn_accel
    } else {
        // ASM loc_857C (kid.asm:9372): neg.l d0; cmpi.l #$38000,d0;
        // ble.s loc_858C; move.l #$38000,d0 — negate to positive,
        // hard cap at AIR_HARD_CAP. Mirror of loc_8536.
        let abs_vel = vel.saturating_neg().min(AIR_HARD_CAP);
        if abs_vel < table.max_air_speed {
            // ASM loc_858C (kid.asm:9378): cmp.l (a0),d0; bge.s loc_859E;
            // fall through to loc_8590 — below max, accelerate.
            // ASM loc_8590 (kid.asm:9382): add.l 4(a0),d0; cmp.l (a0),d0;
            // ble.s loc_859A; move.l (a0),d0 — accelerate by air_accel,
            // clamp to max. Then neg.l d0 restores negative sign. Mirror
            // of loc_8544.
            let next_abs = (abs_vel + table.air_accel).min(table.max_air_speed);
            -next_abs
        } else {
            // ASM loc_859E (kid.asm:9393): sub.l 8(a0),d0 — at/above max,
            // decelerate by air_above_max_decel, clamp floor to max,
            // then neg.l d0. Mirror of loc_8554.
            let next_abs = (abs_vel - table.air_above_max_decel).max(table.max_air_speed);
            -next_abs
        }
    }
}

/// Airborne deceleration towards zero when no direction is held (loc_85AC).
/// Snaps to zero on sign change.
///
/// ASM loc_85AC (kid.asm:9404): `tst.l d0; bpl.s loc_85B8` — branch on sign.
/// Negative vel: `add.l $C(a0),d0; bpl.s loc_85C0; rts` — add decel, if
/// result crosses zero → snap. Positive vel:
/// ASM loc_85B8 (kid.asm:9412): `sub.l $C(a0),d0; bmi.s loc_85C0; rts` —
/// subtract decel, if result crosses zero → snap.
/// ASM loc_85C0 (kid.asm:9418): `moveq #0,d0; rts` — zero-snap target.
fn air_decelerate(vel: i32, table: &AirSpeedTable) -> i32 {
    match vel.cmp(&0) {
        std::cmp::Ordering::Greater => {
            // ASM loc_85B8: sub.l $C(a0),d0; bmi.s loc_85C0; rts
            let next = vel - table.air_no_input_decel;
            if next < 0 { 0 } else { next } // loc_85C0: moveq #0,d0
        }
        std::cmp::Ordering::Less => {
            // ASM loc_85AC fall-through: add.l $C(a0),d0; bpl.s loc_85C0; rts
            let next = vel + table.air_no_input_decel;
            if next > 0 { 0 } else { next } // loc_85C0: moveq #0,d0
        }
        std::cmp::Ordering::Equal => 0,
    }
}

// ---------------------------------------------------------------------------
// Jump height per form — from unk_A49E
// ---------------------------------------------------------------------------

/// Jump-height factor per helmet, indexed by `PlayerForm::helmet_id()`.
pub const JUMP_HEIGHTS: [u16; 10] = [
    0x42, // Kid
    0x42, // Skycutter
    0x42, // Cyclone
    0x4A, // RedStealth
    0x42, // Eyeclops
    0x42, // Juggernaut
    0x42, // IronKnight
    0x42, // Berzerker
    0x42, // Maniaxe
    0x42, // Micromax
];

// ---------------------------------------------------------------------------
// Gravity constants — kid.asm:14060-14088 (sub_B168 normal gravity path)
// ---------------------------------------------------------------------------

/// Gravity applied per frame when descending or when ascending without B held.
/// ASM loc_B1E0 (kid.asm:14066): `addi.l #$4000,d7`.
pub const GRAVITY_NORMAL: i32 = 0x4000;

/// Gravity applied per frame when ascending with B (jump) held.
/// ASM loc_B1E8 (kid.asm:14071): `addi.l #$2000,d7`.
pub const GRAVITY_JUMP_HELD: i32 = 0x2000;

/// Terminal downward velocity.
/// ASM loc_B1EE (kid.asm:14074): `cmpi.l #$80000,d7; ble.s loc_B200`.
pub const MAX_FALL_SPEED: i32 = 0x8_0000;

/// Terminal upward velocity (negative = upward).
/// ASM loc_B200 (kid.asm:14081): `cmpi.l #$FFF80000,d7; bge.s loc_B20E`.
pub const MAX_RISE_SPEED: i32 = -0x8_0000; // 0xFFF8_0000 as i32

// ---------------------------------------------------------------------------
// Camera constants — kid.asm:7306-7521 (sub_71E4)
// ---------------------------------------------------------------------------

/// Viewport width in pixels.
pub const VIEWPORT_WIDTH: i32 = 0x140; // 320

/// Viewport height in pixels.
pub const VIEWPORT_HEIGHT: i32 = 0xE0; // 224

/// Player-to-camera centre offset in pixels.
pub const CAMERA_PLAYER_OFFSET: i32 = 0xA0; // 160

/// Maximum camera velocity in pixels per frame.
pub const CAMERA_MAX_STEP: i32 = 8;

// ---------------------------------------------------------------------------
// Level bounds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelBounds {
    pub width_pixels: u16,
    pub height_pixels: u16,
}

// ---------------------------------------------------------------------------
// Horizontal movement — kid.asm:11696-11851
// ---------------------------------------------------------------------------

/// Input state for horizontal movement.
#[derive(Debug, Clone, Copy)]
pub struct HorizontalInput {
    pub left: bool,
    pub right: bool,
    pub run: bool,
}

/// Update horizontal velocity for one frame.
///
/// Direct translation of the branching at loc_9B14..loc_9C06. Velocity is
/// signed i32 in fixed-point 16.16: positive = rightward.
///
/// Returns `(new_x_vel, new_facing)`.
pub fn update_horizontal_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    table: &SpeedTable,
) -> (i32, Direction) {
    // ASM loc_9B14 (kid.asm:11695): `tst.b x_direction(a3)` selects the
    // facing branch, and opposite held input flips `x_direction` before the
    // shared right/left/no-input velocity paths run.
    // ASM loc_9B26 (kid.asm:11706): the right-facing branch tests
    // `Ctrl_Right_Held`; held falls through to loc_9B2E, otherwise it jumps
    // to the shared loc_9BEE deceleration path.
    // ASM loc_9B8A (kid.asm:11770): the left-facing branch mirrors that gate:
    // `Ctrl_Left_Held` falls through to loc_9B90, otherwise loc_9BEE decelerates.
    if input.right {
        (
            apply_holding_right(x_vel, input.run, table),
            Direction::Right,
        )
    } else if input.left {
        (apply_holding_left(x_vel, input.run, table), Direction::Left)
    } else {
        (decelerate(x_vel, table), facing)
    }
}

/// Update Skycutter ground velocity on flat terrain.
///
/// ASM sub_9A20 (kid.asm:11588): unlike the shared loc_9B2E walker tables,
/// Skycutter uses fixed walk/run caps of `$20000/$40000` with `$2000/$4000`
/// steps and keeps the current facing as the primary branch selector.
#[allow(non_snake_case)]
pub fn update_skycutter_horizontal_velocity_sub_9A20(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
) -> (i32, Direction) {
    // ASM loc_9A84 (kid.asm:11631): left-facing Skycutter tests
    // Ctrl_Right_Held; right held clears x_direction and jumps to loc_9A32,
    // otherwise the reverse-deceleration path remains left-facing.
    let facing_right = match facing {
        Direction::Right => !input.left,
        Direction::Left => input.right,
    };
    let next_facing = if input.right {
        Direction::Right
    } else if input.left {
        Direction::Left
    } else {
        facing
    };

    let next_vel = if facing_right {
        // ASM loc_9A32 (kid.asm:11597): forward-facing/no opposite input
        // enters the A-held run branch or the `$2000` walk accel/cruise clamp.
        if input.run {
            // ASM loc_9A6E (kid.asm:11623): `addi.l #$4000,d0`, clamp at `$40000`.
            (x_vel + 0x4000).min(0x4_0000)
        } else if x_vel < 0x2_0000 {
            (x_vel + 0x2000).min(0x2_0000)
        } else if x_vel <= 0x4_0000 {
            // ASM loc_9A56 (kid.asm:11611): `subi.l #$2000,d0`.
            x_vel - 0x2000
        } else {
            // ASM loc_9A66 (kid.asm:11618): `move.l #$40000,d0`.
            0x4_0000
        }
    } else if input.run {
        // ASM loc_9ACA (kid.asm:11660): reverse path with Ctrl_A_Held set
        // subtracts `$4000`, then clamps at the leftward run cap.
        (x_vel - 0x4000).max(-0x4_0000)
    } else if x_vel > -0x2_0000 {
        // ASM loc_9A90 (kid.asm:11638): reverse path with Ctrl_A_Held clear
        // subtracts `$2000`, then clamps at the leftward walk cap `$FFFE0000`.
        (x_vel - 0x2000).max(-0x2_0000)
    } else if x_vel >= -0x4_0000 {
        // ASM loc_9AB2 (kid.asm:11648): velocities from `$FFFC0000` up to
        // cruise speed add `$2000`, decelerating back toward zero.
        x_vel + 0x2000
    } else {
        // ASM loc_9AC2 (kid.asm:11655): values below `$FFFC0000` clamp to
        // the leftward run cap before returning.
        -0x4_0000
    };

    (next_vel, next_facing)
}

/// Facing right, holding right — loc_9B2E..loc_9B7E.
fn apply_holding_right(vel: i32, run: bool, table: &SpeedTable) -> i32 {
    if run {
        // loc_9B62: running right
        if vel >= 0 {
            (vel + table.run_accel).min(table.max_run_speed)
        } else {
            // ASM loc_9B78 (kid.asm:11757): running right while moving left
            // adds `$18(a0)`, the strong-brake value.
            vel + table.strong_brake
        }
    } else if vel >= 0 {
        // Walking right, moving right
        // ASM loc_9B44 (kid.asm:11723): once under the run cap, compare
        // against the walk cap; above it decelerates by `8(a0)`, otherwise
        // the fall-through path applies walk acceleration and clamps to `(a0)`.
        if vel > table.max_run_speed {
            table.max_run_speed
        } else if vel > table.max_walk_speed {
            // Above walk cap: decel with walk_run_decel ($08)
            vel - table.walk_run_decel
        } else {
            // ASM loc_9B4E (kid.asm:11729): below walk cap, add `4(a0)`
            // then clamp to `(a0)`.
            (vel + table.walk_accel).min(table.max_walk_speed)
        }
    } else {
        // ASM loc_9B5C (kid.asm:11739): walking right while moving left adds
        // `$C(a0)`, the turn-brake value.
        vel + table.turn_brake
    }
}

/// Facing left, holding left — loc_9B90..loc_9BEC. Mirror of right-facing
/// logic, but uses strong_brake ($18) for opposite-direction walk brake
/// instead of turn_brake ($0C).
fn apply_holding_left(vel: i32, run: bool, table: &SpeedTable) -> i32 {
    if run {
        // loc_9BCE: running left
        // ASM: `tst.l d0; bpl.s loc_9BE8` — vel=0 is "not moving left" (bpl
        // tests N flag; 0 clears N), so stopped uses the strong_brake path.
        if vel < 0 {
            let abs_vel = vel.saturating_neg();
            let next_abs = (abs_vel + table.run_accel).min(table.max_run_speed);
            -next_abs
        } else {
            // Moving right OR stopped while holding left+run: strong_brake ($18)
            vel - table.strong_brake
        }
    } else if vel < 0 {
        // Walking left, already moving left
        // ASM: `tst.l d0; bpl.s loc_9BC8` — vel=0 uses strong_brake path.
        let abs_vel = vel.saturating_neg();
        if abs_vel > table.max_run_speed {
            -table.max_run_speed
        } else if abs_vel > table.max_walk_speed {
            // ASM loc_9BAC (kid.asm:11789): after the run-cap guard, speeds
            // above `(a0)` decelerate with `add.l 8(a0),d0`.
            // Above walk cap: decel with walk_run_decel ($08)
            vel + table.walk_run_decel
        } else {
            // ASM loc_9BB6 (kid.asm:11795): below the walk cap, subtract
            // `4(a0)`, then clamp back to `-(a0)` if acceleration crosses it.
            // Below walk cap: accelerate
            let next = vel - table.walk_accel;
            if next.saturating_neg() > table.max_walk_speed {
                -table.max_walk_speed
            } else {
                next
            }
        }
    } else {
        // Walking left, but moving right OR stopped: strong_brake ($18)
        vel - table.strong_brake
    }
}

/// Decelerate towards zero when no direction is held (loc_9BEE). Snap to zero
/// on sign change.
fn decelerate(vel: i32, table: &SpeedTable) -> i32 {
    match vel.cmp(&0) {
        std::cmp::Ordering::Greater => {
            // ASM loc_9BFC (kid.asm:11843): moving right with no direction held
            // subtracts `8(a0)` and snaps only if the result goes negative.
            let next = vel - table.walk_run_decel;
            if next < 0 { 0 } else { next }
        }
        std::cmp::Ordering::Less => {
            let next = vel + table.walk_run_decel;
            if next > 0 { 0 } else { next }
        }
        std::cmp::Ordering::Equal => 0,
    }
}

// ---------------------------------------------------------------------------
// Jump — kid.asm:12783 (sub_A432)
// ---------------------------------------------------------------------------

/// Initiate a jump. Returns `(new_x_vel, initial_y_vel)`.
///
/// Translates sub_A432:
/// 1. `factor = (|x_vel| << 4) >> 16 + $100`  (swap after lsl #4)
/// 2. `y_vel = -(jump_height * factor) << 4`
/// 3. `x_vel = x_vel * 3/4`  (asr #1, copy, asr #1, add)
pub fn initiate_jump(x_vel: i32, jump_height: u16) -> (i32, i32) {
    // ASM loc_A44A (kid.asm:12792): entry after `bpl.s` / `neg.l d1` —
    // d1 = abs(x_vel). The remaining arithmetic computes jump height from
    // horizontal speed: `lsl.l #4,d1; swap d1; addi.w #$100,d1;
    // mulu.w d1,d0; lsl.l #4,d0; neg.l d0` → y_vel = -(height * factor) << 4.
    let abs_vel = if x_vel >= 0 { x_vel } else { -x_vel } as u32;

    // lsl.l #4, d1 — multiply by 16
    let shifted = abs_vel << 4;
    // swap d1 — extract upper 16 bits (equivalent to >> 16)
    let upper = (shifted >> 16) as u16;
    // addi.w #$100, d1
    let factor = u32::from(upper) + 0x100;

    // mulu.w d1, d0 — unsigned 16×16→32 multiply
    let product = u32::from(jump_height) * factor;
    // lsl.l #4, d0 — multiply by 16
    let scaled = product << 4;
    // neg.l d0 — negate for upward
    let y_vel = -(scaled as i32);

    // ASM loc_A466 (kid.asm:12803): `move.l d0,y_vel(a3); move.l d0,($FFFFFAAA)`
    // — stores computed y_vel. Also `tst.l y_pos(a3); bpl.s loc_A480` wraps
    // negative y_pos to Level_height_pixels. KidGrabbedByHand check at 12799
    // weakens the jump by `addi.l #$20000,d0` (less negative → lower jump).
    // Native: y_vel store and y_pos clamp happen at call sites (movement.rs);
    // $FFFFFAAA is not modelled; KidGrabbedByHand not yet ported.

    // ASM loc_A480 (kid.asm:12813): `move.l x_vel(a3),d0; asr.l #1,d0;
    // move.l d0,d1; asr.l #1,d1; add.l d1,d0; move.l d0,x_vel(a3)` — x_vel
    // *= 3/4 via arithmetic shift. Then `moveq #sfx_Jump,d0; jsr PlaySound`
    // plays the jump sound. Native: sound is triggered at call sites.
    let half = x_vel >> 1; // asr.l #1
    let quarter = half >> 1; // asr.l #1
    let new_x_vel = half + quarter;

    (new_x_vel, y_vel)
}

// ---------------------------------------------------------------------------
// Gravity — kid.asm:14060-14088
// ---------------------------------------------------------------------------

/// Apply one frame of gravity to vertical velocity.
///
/// ASM loc_B1D4 (kid.asm:14060): normal gravity entry — loads y_vel, branches
/// on sign (`bpl.s loc_B1E0`) and B-held (`tst.b Ctrl_B_Held; bne.s loc_B1E8`).
/// loc_B1E0: default gravity `addi.l #$4000,d7`.
/// loc_B1E8: B-held gravity `addi.l #$2000,d7`.
/// loc_B1EE: falling cap `cmpi.l #$80000,d7; ble.s loc_B200; move.l #$80000,d7`.
/// loc_B200: rising cap `cmpi.l #$FFF80000,d7; bge.s loc_B20E; move.l #$FFF80000,d7`.
/// loc_B20E: store `move.l d7,y_vel(a3); rts`.
///
/// `ascending_and_b_held`: true when y_vel < 0 AND the jump button is held,
/// giving the variable-height jump feel.
pub fn apply_gravity(y_vel: i32, ascending_and_b_held: bool) -> i32 {
    // ASM loc_B1D4: `bpl.s loc_B1E0` — when y_vel >= 0 (descending), always
    // use normal gravity. When ascending (y_vel < 0), check B held.
    let gravity = if ascending_and_b_held {
        GRAVITY_JUMP_HELD // ASM loc_B1E8: addi.l #$2000,d7
    } else {
        GRAVITY_NORMAL // ASM loc_B1E0: addi.l #$4000,d7
    };
    let next = y_vel.saturating_add(gravity);
    // ASM loc_B1EE/loc_B200: clamp to [MAX_RISE_SPEED, MAX_FALL_SPEED]
    // ASM loc_B20E: move.l d7,y_vel(a3); rts
    next.clamp(MAX_RISE_SPEED, MAX_FALL_SPEED)
}

// ---------------------------------------------------------------------------
// Camera follow — kid.asm:7306-7521 (sub_71E4)
// ---------------------------------------------------------------------------

/// Update camera position to follow the player. All coordinates are in pixels
/// (the 68k camera logic operates on the upper 16 bits of positions).
///
/// ASM loc_71EC (kid.asm:7310): `beq.w loc_73C2` — if `Addr_GfxObject_Kid` is
/// null (d7=0), branch straight to the routine exit which writes back the
/// original scroll values unchanged (camera doesn't move).  In the native port
/// this null-bail is implicit: `update_camera_follow` is only called when a
/// player exists, so the camera naturally stays frozen when Kid is absent.
///
/// Returns `(new_camera_x, new_camera_y)` in pixels.
pub fn update_camera(
    camera_x: i32,
    camera_y: i32,
    player_x: i32,
    player_y: i32,
    bounds: &LevelBounds,
) -> (i32, i32) {
    let new_x = update_camera_axis(
        camera_x,
        player_x,
        i32::from(bounds.width_pixels) - VIEWPORT_WIDTH,
    );
    let new_y = update_camera_axis(
        camera_y,
        player_y,
        i32::from(bounds.height_pixels) - VIEWPORT_HEIGHT,
    );
    (new_x, new_y)
}

/// Single-axis camera follow: target = player - $A0, clamped to [0, max_pos],
/// delta = (target - camera + 4) >> 3, clamped to [-8, 8]. Special edge
/// behaviour when camera is near boundary.
///
/// The X-axis instance maps to kid.asm:7314-7364 inside sub_71E4:
///
/// **Target clamp (loc_720E, kid.asm:7322)**:
/// ```asm
///   move.w (Level_width_pixels).w,d1   ; d1 = level width
///   subi.w #$140,d1                     ; d1 = max camera X
///   move.w $1A(a0),d0                   ; d0 = Kid screen X
///   subi.w #$A0,d0                      ; d0 = target
///   bcc.s  loc_720E                     ; if >= 0, skip floor
///   moveq  #0,d0                        ; floor at 0
/// loc_720E:
///   cmp.w  d1,d0                        ; target vs max?
///   bcs.s  loc_7214                     ; if < max, keep
///   move.w d1,d0                        ; cap at max
/// ```
/// Native: `(player - CAMERA_PLAYER_OFFSET).max(0).min(max_pos)`
///
/// **Velocity logic (loc_7214, kid.asm:7327)**:
/// ```asm
/// loc_7214:
///   move.w (Camera_X_pos).w,d4
///   cmpi.w #4,d4          ; near left edge?
///   bgt.s  loc_7226
///   cmp.w  d4,d0          ; target < camera?
///   bge.s  loc_7226
///   moveq  #-1,d2         ; nudge left
///   bra.s  loc_723E
/// loc_7226:
///   subq.w #4,d1          ; max_pos - 4
///   cmp.w  d1,d4          ; near right edge?
///   blt.s  loc_7234
///   cmp.w  d4,d0          ; target > camera?
///   ble.s  loc_7234
///   moveq  #1,d2          ; nudge right
///   bra.s  loc_723E
/// loc_7234:
///   sub.w  (Camera_X_pos).w,d0  ; delta = target - camera
///   addq.w #4,d0                ; + 4 bias
///   asr.w  #3,d0                ; >> 3
///   move.w d0,d2
/// loc_723E:
///   cmpi.w #8,d2           ; clamp to [-8, 8]
///   blt.s  loc_7246
///   moveq  #8,d2
/// loc_7246:
///   cmpi.w #$FFF8,d2
///   bgt.s  loc_724E
///   moveq  #-8,d2
/// loc_724E:
///   add.w  d2,(Camera_X_pos).w
/// ```
/// Native: edge-snap branches then `(target - camera + 4) >> 3` clamped to ±8.
pub(crate) fn update_camera_axis(camera: i32, player: i32, max_pos: i32) -> i32 {
    // ASM loc_720E (kid.asm:7322) / loc_7266 (kid.asm:7372): target clamp.
    // X-axis: loc_720E clamps to [0, Level_width - $140].
    // Y-axis: loc_7266 clamps to [0, Level_height - $E0].
    // Both use the same pattern: `subi.w #offset,d0; bcc.s skip; moveq #0,d0`
    // then `cmp.w d1,d0; bcs.s skip; move.w d1,d0`.
    let target = (player - CAMERA_PLAYER_OFFSET).max(0).min(max_pos);

    // ASM loc_7214 / loc_72A8 (kid.asm:7327 / 7401): velocity with edge-snap
    // special cases.  X-axis uses d2/d4, Y-axis uses d3/d4; logic is identical.
    let velocity = if camera <= 4 && target < camera {
        // ASM loc_7214 / loc_72A8: near left/top edge, target further left/up
        // → nudge at -1.  `cmpi.w #4,d4; bgt.s ...; cmp.w d4,d0; bge.s ...;
        //   moveq #-1,d2` (X) / `moveq #-1,d3` (Y)
        -1
    } else if camera >= max_pos - 4 && target > camera {
        // ASM loc_7226 / loc_72BA (kid.asm:7337 / 7411): near right/bottom
        // edge, target further right/down → nudge at +1.
        // `subq.w #4,d1; cmp.w d1,d4; blt.s ...; cmp.w d4,d0; ble.s ...;
        //  moveq #1,d2` (X) / `moveq #1,d3` (Y)
        1
    } else {
        // ASM loc_7234 / loc_72C8 (kid.asm:7347 / 7421): normal smooth scroll.
        // `sub.w Camera_X/Y_pos,d0; addq.w #4,d0; asr.w #3,d0; move.w d0,d2/d3`
        let delta = (target - camera + 4) >> 3;
        delta.clamp(-CAMERA_MAX_STEP, CAMERA_MAX_STEP)
    };

    // ASM loc_724E (kid.asm:7363) / loc_72E2 (kid.asm:7437): apply velocity.
    // X-axis: `add.w d2,(Camera_X_pos).w`
    // Y-axis: `add.w d3,(Camera_Y_pos).w`
    // Native clamps result to [0, max_pos]; the ROM relies on unsigned
    // saturation from the earlier target clamp to stay in bounds.
    (camera + velocity).max(0).min(max_pos)
}

/// ASM loc_72E2 (kid.asm:7437–7444): background parallax scroll.
///
/// After applying camera Y velocity, the ROM computes background-layer scroll
/// positions by dividing camera coordinates by 4:
/// ```asm
/// loc_72E2:
///   add.w   d3,(Camera_Y_pos).w
///   move.l  (Camera_X_pos).w,d0
///   lsr.l   #2,d0
///   move.l  d0,(Camera_BG_X_pos).w
///   move.l  (Camera_Y_pos).w,d0
///   lsr.l   #2,d0
///   move.l  d0,(Camera_BG_Y_pos).w
/// ```
///
/// The native port does not maintain `Camera_BG_X/Y_pos` because background
/// rendering uses a separate scroll register pipeline.  The 1:4 ratio is
/// implicit in the VDP scroll setup.
// TODO(fidelity): BG parallax ratio is currently hardcoded in rendering;
// if a level ever uses a non-1:4 ratio this function would need to supply it.
#[cfg(test)]
pub(crate) fn camera_bg_scroll_loc_72e2(camera_x: u32, camera_y: u32) -> (u32, u32) {
    (camera_x >> 2, camera_y >> 2)
}

/// ASM loc_726C (kid.asm:7377): Y-axis look-offset pull-back at level ceiling.
///
/// When the camera target Y is clamped to 0 (Kid near the top of the level)
/// and Kid's current Y is above their previous-frame Y (`$FFFFFA2E`), the ROM
/// decrements the look-down accumulator (`$FFFFFB58`) by 2. This prevents the
/// look-down offset from lingering when Kid is jumping upward near the level
/// ceiling.
///
/// ```asm
/// loc_726C:
///   move.w  $1E(a0),d7          ; d7 = kid_y
///   tst.w   d0                   ; target_y == 0?
///   bne.s   loc_727E             ; no → skip
///   cmp.w   ($FFFFFA2E).w,d7    ; kid_y vs last_kid_y
///   bge.s   loc_727E             ; kid_y >= last → skip
///   subq.w  #2,($FFFFFB58).w    ; y_look_offset -= 2
/// ```
///
/// Returns the adjustment to apply to `y_look_offset` (0 or -2).
pub fn camera_y_ceiling_look_adjust_loc_726c(target_y: i32, kid_y: i16, last_kid_y: i16) -> i16 {
    if target_y == 0 && kid_y < last_kid_y {
        -2
    } else {
        0
    }
}

/// ASM loc_727E (kid.asm:7385): Y-axis look-offset pull-back near viewport top.
///
/// When Kid is within 32 pixels of the top of the camera viewport
/// (`kid_y - Camera_Y_pos < $20`), the ROM decrements the look-down
/// accumulator (`$FFFFFB58`) by 1. This keeps the camera from getting stuck
/// looking down when Kid is high on screen.
///
/// ```asm
/// loc_727E:
///   sub.w   (Camera_Y_pos).w,d7  ; d7 = kid_y - camera_y
///   cmpi.w  #$20,d7              ; < 32?
///   bge.s   loc_728C             ; no → skip
///   subq.w  #1,($FFFFFB58).w    ; y_look_offset -= 1
/// ```
///
/// Returns the adjustment to apply to `y_look_offset` (0 or -1).
pub fn camera_y_viewport_top_look_adjust_loc_727e(kid_y: i16, camera_y: i16) -> i16 {
    if kid_y.wrapping_sub(camera_y) < 0x20 {
        -1
    } else {
        0
    }
}

/// ASM loc_7396–loc_73B8 (kid.asm:7499–7514): reverse murder wall camera push.
///
/// After the reversed wall decelerates leftward, compare camera to wall:
///
/// ```asm
/// loc_7396:
///   move.w  (Camera_X_pos).w,d0
///   sub.w   (MurderWall_X_pos).w,d0   ; d0 = camera_px - wall_px
///   bge.s   loc_73B8                   ; camera ahead → snap camera to wall
///
/// loc_73A0:                            ; camera behind wall (d0 < 0)
///   cmpi.w  #-$80,d0
///   bgt.w   loc_73C2                   ; small gap → do nothing
///   move.w  (Camera_X_pos).w,(MurderWall_X_pos).w
///   addi.w  #$80,(MurderWall_X_pos).w  ; clamp wall = camera + $80
///   bra.w   loc_73C2
///
/// loc_73B8:                            ; camera ahead of wall (d0 >= 0)
///   move.w  (MurderWall_X_pos).w,(Camera_X_pos).w  ; snap camera to wall
///   bra.w   *+4                        ; → loc_73C2
/// ```
///
/// Returns `(new_camera_x, new_wall_x)`.  Both values are fixed-point 16.16;
/// only the integer (high-word) portion is modified, matching the ROM's `.w`
/// operations.
pub(crate) fn reverse_murder_wall_camera_push_loc_7396(camera_x: u32, wall_x: i64) -> (u32, i64) {
    let camera_px = (camera_x >> 16) as i16;
    let wall_px = (wall_x >> 16) as i16;
    let d0 = camera_px.wrapping_sub(wall_px);

    if d0 >= 0 {
        // loc_73B8: camera ahead of (right of) reversed wall — snap camera
        // leftward to wall position.
        let new_camera_x = (camera_x & 0xFFFF) | ((wall_px as u16 as u32) << 16);
        (new_camera_x, wall_x)
    } else if d0 > -0x80 {
        // loc_73A0 → loc_73C2: gap within $80 — tolerable, no adjustment.
        (camera_x, wall_x)
    } else {
        // loc_73A0: camera far behind — clamp wall so it stays within $80 of
        // camera: MurderWall_X_pos.w = Camera_X_pos.w + $80.
        let new_wall_high = camera_px.wrapping_add(0x80) as u16;
        let new_wall_x = (wall_x & 0xFFFF) | (i64::from(new_wall_high) << 16);
        (camera_x, new_wall_x)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::assertions_on_constants, clippy::identity_op)]
mod tests {
    use super::*;

    #[test]
    fn normal_kid_table_matches_disasm() {
        assert_eq!(NORMAL_KID.max_walk_speed, 0x2_0000);
        assert_eq!(NORMAL_KID.walk_accel, 0x1000);
        assert_eq!(NORMAL_KID.walk_run_decel, 0x2800);
        assert_eq!(NORMAL_KID.turn_brake, 0x2000);
        assert_eq!(NORMAL_KID.max_run_speed, 0x3_8000);
        assert_eq!(NORMAL_KID.run_accel, 0x1C00);
        assert_eq!(NORMAL_KID.strong_brake, 0x3000);
    }

    #[test]
    fn normal_micromax_table_matches_disasm() {
        assert_eq!(NORMAL_MICROMAX.max_walk_speed, 0x1_0000);
        assert_eq!(NORMAL_MICROMAX.walk_accel, 0x800);
        assert_eq!(NORMAL_MICROMAX.max_run_speed, 0x2_0000);
    }

    #[test]
    fn normal_juggernaut_table_matches_disasm() {
        assert_eq!(NORMAL_JUGGERNAUT.max_walk_speed, 0x1_8000);
        assert_eq!(NORMAL_JUGGERNAUT.walk_accel, 0x2000);
        assert_eq!(NORMAL_JUGGERNAUT.max_run_speed, 0x3_0000);
    }

    #[test]
    fn ice_kid_table_matches_disasm() {
        assert_eq!(ICE_KID.max_walk_speed, 0x1_8000);
        assert_eq!(ICE_KID.walk_accel, 0x2000);
        assert_eq!(ICE_KID.max_run_speed, 0x2_8000);
    }

    #[test]
    fn rubber_kid_table_matches_disasm() {
        assert_eq!(RUBBER_KID.max_walk_speed, 0x4_0000);
        assert_eq!(RUBBER_KID.walk_accel, 0x200);
        assert_eq!(RUBBER_KID.max_run_speed, 0x7_0000);
    }

    #[test]
    fn jump_height_kid_matches_disasm() {
        assert_eq!(JUMP_HEIGHTS[0], 0x42);
    }

    #[test]
    fn jump_height_red_stealth_is_higher() {
        assert_eq!(JUMP_HEIGHTS[3], 0x4A);
        assert!(JUMP_HEIGHTS[3] > JUMP_HEIGHTS[0]);
    }

    #[test]
    fn gravity_constants_match_disasm() {
        assert_eq!(GRAVITY_NORMAL, 0x4000);
        assert_eq!(GRAVITY_JUMP_HELD, 0x2000);
        assert_eq!(MAX_FALL_SPEED, 0x8_0000);
        assert_eq!(MAX_RISE_SPEED, -0x8_0000);
        // Verify two's complement matches 68k representation
        assert_eq!(MAX_RISE_SPEED as u32, 0xFFF8_0000);
    }

    #[test]
    fn micromax_walk_speed_is_half_of_kid_on_normal_terrain() {
        assert_eq!(
            NORMAL_MICROMAX.max_walk_speed,
            NORMAL_KID.max_walk_speed / 2
        );
        assert_eq!(NORMAL_MICROMAX.walk_accel, NORMAL_KID.walk_accel / 2);
    }

    #[test]
    fn rubber_has_highest_speed_caps() {
        assert!(RUBBER_KID.max_walk_speed > NORMAL_KID.max_walk_speed);
        assert!(RUBBER_KID.max_run_speed > NORMAL_KID.max_run_speed);
        assert!(RUBBER_KID.max_walk_speed > ICE_KID.max_walk_speed);
    }

    #[test]
    fn rubber_has_lowest_acceleration() {
        assert!(RUBBER_KID.walk_accel < NORMAL_KID.walk_accel);
        assert!(RUBBER_KID.walk_accel < ICE_KID.walk_accel);
    }

    // -- Horizontal movement tests --

    fn right_walk() -> HorizontalInput {
        HorizontalInput {
            left: false,
            right: true,
            run: false,
        }
    }

    fn right_run() -> HorizontalInput {
        HorizontalInput {
            left: false,
            right: true,
            run: true,
        }
    }

    fn left_walk() -> HorizontalInput {
        HorizontalInput {
            left: true,
            right: false,
            run: false,
        }
    }

    fn no_input() -> HorizontalInput {
        HorizontalInput {
            left: false,
            right: false,
            run: false,
        }
    }

    #[test]
    fn skycutter_sub_9a20_walk_right_caps_at_walk_speed() {
        let (vel, dir) =
            update_skycutter_horizontal_velocity_sub_9A20(0x1_E000, Direction::Right, right_walk());

        assert_eq!(vel, 0x2_0000);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn skycutter_sub_9a20_run_right_from_standstill_adds_4000() {
        // ASM loc_9A6E (kid.asm:11623): `addi.l #$4000,d0`, then clamp at `$40000`.
        let (vel, dir) =
            update_skycutter_horizontal_velocity_sub_9A20(0, Direction::Right, right_run());

        assert_eq!(vel, 0x4000);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn skycutter_sub_9a20_walk_right_decelerates_back_from_run_speed() {
        let (vel, dir) =
            update_skycutter_horizontal_velocity_sub_9A20(0x2_2000, Direction::Right, right_walk());

        assert_eq!(vel, 0x2_0000);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn skycutter_sub_9a20_walk_right_at_exact_walk_speed_decelerates() {
        // ASM loc_9A32 (kid.asm:11597): `cmpi.l #$20000,d0; bge.s loc_9A56`.
        // Equality branches to loc_9A56, so exact walk speed decelerates by $2000.
        let (vel, dir) =
            update_skycutter_horizontal_velocity_sub_9A20(0x2_0000, Direction::Right, right_walk());

        assert_eq!(vel, 0x1_E000);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn skycutter_sub_9a20_pressing_away_flips_facing_and_uses_left_branch() {
        let (vel, dir) =
            update_skycutter_horizontal_velocity_sub_9A20(0, Direction::Right, left_walk());

        assert_eq!(vel, -0x2000);
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn skycutter_sub_9a20_run_left_clamps_to_negative_run_cap() {
        let (vel, dir) = update_skycutter_horizontal_velocity_sub_9A20(
            -0x3_E000,
            Direction::Left,
            HorizontalInput {
                left: true,
                right: false,
                run: true,
            },
        );

        assert_eq!(vel, -0x4_0000);
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn skycutter_sub_9a20_walk_left_at_exact_walk_speed_decelerates() {
        // ASM loc_9A90 (kid.asm:11638): `cmpi.l #$FFFE0000,d0; ble.s loc_9AB2`.
        // Equality branches to loc_9AB2, so exact left walk speed decelerates by $2000.
        let (vel, dir) =
            update_skycutter_horizontal_velocity_sub_9A20(-0x2_0000, Direction::Left, left_walk());

        assert_eq!(vel, -0x1_E000);
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn walk_right_accelerates_from_standstill() {
        let (vel, dir) = update_horizontal_velocity(0, Direction::Right, right_walk(), &NORMAL_KID);
        assert_eq!(vel, NORMAL_KID.walk_accel);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn walk_right_caps_at_max_walk_speed() {
        let mut vel = 0i32;
        for _ in 0..1000 {
            let (v, _) =
                update_horizontal_velocity(vel, Direction::Right, right_walk(), &NORMAL_KID);
            vel = v;
        }
        assert_eq!(vel, NORMAL_KID.max_walk_speed);
    }

    #[test]
    fn run_right_caps_at_max_run_speed() {
        let mut vel = 0i32;
        for _ in 0..1000 {
            let (v, _) =
                update_horizontal_velocity(vel, Direction::Right, right_run(), &NORMAL_KID);
            vel = v;
        }
        assert_eq!(vel, NORMAL_KID.max_run_speed);
    }

    #[test]
    fn decel_to_zero_from_walk_speed() {
        let mut vel = NORMAL_KID.max_walk_speed;
        let mut frames = 0;
        while vel != 0 {
            let (v, _) = update_horizontal_velocity(vel, Direction::Right, no_input(), &NORMAL_KID);
            vel = v;
            frames += 1;
            assert!(frames < 1000, "should reach zero");
        }
        assert_eq!(vel, 0);
    }

    #[test]
    fn decel_snaps_to_zero_on_sign_change() {
        // Velocity just barely positive, decel should snap to zero not go negative
        let vel = NORMAL_KID.walk_run_decel - 1;
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Right, no_input(), &NORMAL_KID);
        assert_eq!(result, 0);
    }

    #[test]
    fn brake_direction_change_right_to_left() {
        // Moving right, now holding left → should brake with strong_brake
        let vel = NORMAL_KID.max_walk_speed;
        let (result, dir) =
            update_horizontal_velocity(vel, Direction::Right, left_walk(), &NORMAL_KID);
        assert_eq!(result, vel - NORMAL_KID.strong_brake);
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn brake_direction_change_left_to_right() {
        // Moving left, now holding right → should brake with turn_brake
        let vel = -NORMAL_KID.max_walk_speed;
        let (result, dir) =
            update_horizontal_velocity(vel, Direction::Right, right_walk(), &NORMAL_KID);
        assert_eq!(result, vel + NORMAL_KID.turn_brake);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn brake_asymmetry_turn_vs_strong() {
        // The key disassembly asymmetry: right-moving-left uses turn_brake ($0C),
        // left-moving-right uses strong_brake ($18)
        let vel_r = -NORMAL_KID.max_walk_speed;
        let (result_r, _) =
            update_horizontal_velocity(vel_r, Direction::Right, right_walk(), &NORMAL_KID);
        let vel_l = NORMAL_KID.max_walk_speed;
        let (result_l, _) =
            update_horizontal_velocity(vel_l, Direction::Left, left_walk(), &NORMAL_KID);

        // Right-facing brake: turn_brake = 0x2000
        assert_eq!(result_r, vel_r + NORMAL_KID.turn_brake);
        // Left-facing brake: strong_brake = 0x3000
        assert_eq!(result_l, vel_l - NORMAL_KID.strong_brake);
        // They should differ (asymmetry)
        assert_ne!(NORMAL_KID.turn_brake, NORMAL_KID.strong_brake);
    }

    #[test]
    fn walk_left_accelerates_and_caps() {
        let mut vel = 0i32;
        for _ in 0..1000 {
            let (v, _) = update_horizontal_velocity(vel, Direction::Left, left_walk(), &NORMAL_KID);
            vel = v;
        }
        assert_eq!(vel, -NORMAL_KID.max_walk_speed);
    }

    #[test]
    fn walk_above_walk_cap_decelerates_towards_cap() {
        // Start at run speed, hold walk → should decel towards walk cap
        let vel = NORMAL_KID.max_run_speed;
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Right, right_walk(), &NORMAL_KID);
        assert_eq!(result, vel - NORMAL_KID.walk_run_decel);
        assert!(result > NORMAL_KID.max_walk_speed);
    }

    // -- Jump tests --

    #[test]
    fn standstill_jump_y_velocity() {
        // x_vel=0 → factor = 0 + $100 = $100
        // y_vel = -(0x42 * 0x100) << 4 = -(0x4200) << 4 = -0x42000
        let (new_x, y_vel) = initiate_jump(0, JUMP_HEIGHTS[0]);
        assert_eq!(new_x, 0);
        assert_eq!(y_vel, -0x4_2000);
    }

    #[test]
    fn walk_speed_jump_y_velocity() {
        let x_vel = NORMAL_KID.max_walk_speed; // 0x20000
        // |x_vel| << 4 = 0x200000, swap → upper 16 = 0x20
        // factor = 0x20 + 0x100 = 0x120
        // product = 0x42 * 0x120 = 0x4A40
        // y_vel = -(0x4A40 << 4) = -0x4A400
        let (new_x, y_vel) = initiate_jump(x_vel, JUMP_HEIGHTS[0]);
        assert_eq!(y_vel, -0x4_A400);
        // x_vel *= 3/4: 0x20000 >> 1 = 0x10000, >> 1 = 0x8000, sum = 0x18000
        assert_eq!(new_x, 0x1_8000);
    }

    #[test]
    fn red_stealth_jumps_higher_than_kid() {
        let (_, y_kid) = initiate_jump(0, JUMP_HEIGHTS[0]);
        let (_, y_rs) = initiate_jump(0, JUMP_HEIGHTS[3]);
        // RedStealth has higher jump height factor (0x4A vs 0x42)
        assert!(y_rs < y_kid); // more negative = higher
    }

    #[test]
    fn jump_reduces_x_velocity_to_three_quarters() {
        let x_vel = 0x4_0000i32;
        let (new_x, _) = initiate_jump(x_vel, JUMP_HEIGHTS[0]);
        assert_eq!(new_x, 0x3_0000); // 3/4 of 0x40000
    }

    #[test]
    fn jump_negative_x_velocity_reduces_correctly() {
        let x_vel = -0x4_0000i32;
        let (new_x, _) = initiate_jump(x_vel, JUMP_HEIGHTS[0]);
        // asr preserves sign: -0x40000 >> 1 = -0x20000, >> 1 = -0x10000
        // sum = -0x20000 + -0x10000 = -0x30000
        assert_eq!(new_x, -0x3_0000);
    }

    /// M68K `asr.l #1` on -1 yields -1 (rounds toward -∞), so the 3/4
    /// factor (half + quarter = -1 + -1 = -2) overshoots by 1 unit.
    /// This is faithful to the ROM's `sub_A432` arithmetic.
    #[test]
    fn jump_asr_rounding_overshoots_at_minus_one() {
        let (new_x, _) = initiate_jump(-1, JUMP_HEIGHTS[0]);
        // half(-1) = -1, quarter(-1) = -1, sum = -2
        assert_eq!(new_x, -2);
    }

    /// Positive x_vel = 1 reduces to 0 (no overshoot on positive side).
    #[test]
    fn jump_asr_rounding_zeroes_at_plus_one() {
        let (new_x, _) = initiate_jump(1, JUMP_HEIGHTS[0]);
        // half(1) = 0, quarter(0) = 0, sum = 0
        assert_eq!(new_x, 0);
    }

    // -- Gravity tests --

    #[test]
    fn gravity_normal_adds_4000() {
        let result = apply_gravity(0, false);
        assert_eq!(result, GRAVITY_NORMAL);
    }

    #[test]
    fn gravity_jump_held_adds_2000_when_ascending() {
        let result = apply_gravity(-0x1_0000, true);
        assert_eq!(result, -0x1_0000 + GRAVITY_JUMP_HELD);
    }

    #[test]
    fn gravity_caps_at_max_fall_speed() {
        let result = apply_gravity(MAX_FALL_SPEED, false);
        assert_eq!(result, MAX_FALL_SPEED);
    }

    #[test]
    fn gravity_caps_at_max_rise_speed() {
        let result = apply_gravity(MAX_RISE_SPEED, true);
        assert_eq!(result, MAX_RISE_SPEED + GRAVITY_JUMP_HELD);
        // Even extreme upward velocity is clamped
        let extreme = apply_gravity(i32::MIN / 2, false);
        assert_eq!(extreme, MAX_RISE_SPEED);
    }

    /// ASM loc_B1E0 (kid.asm:14066): descending uses default gravity.
    /// `bpl.s loc_B1E0` — when y_vel >= 0 the sign bit is clear (positive),
    /// so the branch is taken and `addi.l #$4000,d7` applies unconditionally.
    /// In native, the caller ensures `ascending_and_b_held` is false when
    /// y_vel >= 0, mirroring the ASM `bpl.s` guard.
    #[test]
    fn loc_b1e0_descending_uses_normal_gravity() {
        // y_vel = +$1_0000 (descending): bpl.s taken → loc_B1E0 adds $4000
        let result = apply_gravity(0x1_0000, false);
        assert_eq!(result, 0x1_0000 + 0x4000);
        // y_vel = 0 (apex): bpl.s still taken (zero is non-negative)
        let result_zero = apply_gravity(0, false);
        assert_eq!(result_zero, 0x4000);
    }

    /// ASM loc_B1E0 (kid.asm:14066): ascending without B held also uses $4000.
    /// When y_vel < 0, bpl.s is NOT taken, but `tst.b (Ctrl_B_Held)` is zero
    /// so `bne.s loc_B1E8` is NOT taken → falls through to loc_B1E0.
    #[test]
    fn loc_b1e0_ascending_b_released_uses_normal_gravity() {
        let result = apply_gravity(-0x2_0000, false);
        assert_eq!(result, -0x2_0000 + 0x4000);
    }

    /// ASM loc_B1E8 (kid.asm:14071): ascending with B held uses weaker $2000.
    /// `tst.b (Ctrl_B_Held); bne.s loc_B1E8` — B is held so branch taken,
    /// `addi.l #$2000,d7` gives the variable-height jump feel.
    #[test]
    fn loc_b1e8_ascending_b_held_uses_jump_held_gravity() {
        let result = apply_gravity(-0x2_0000, true);
        assert_eq!(result, -0x2_0000 + 0x2000);
    }

    /// ASM loc_B1EE (kid.asm:14074): falling speed cap at $80000.
    /// `cmpi.l #$80000,d7; ble.s loc_B200` — when d7 > $80000, clamp to $80000
    /// then branch to loc_B20E to store.
    #[test]
    fn loc_b1ee_falling_speed_capped_at_max() {
        // Just below cap: $7_C000 + $4000 = $8_0000 = MAX_FALL_SPEED (not clamped)
        let at_boundary = apply_gravity(0x7_C000, false);
        assert_eq!(at_boundary, 0x8_0000);
        // Above cap: $8_0000 + $4000 would be $8_4000, clamped to $8_0000
        let above = apply_gravity(0x8_0000, false);
        assert_eq!(above, MAX_FALL_SPEED);
    }

    /// ASM loc_B200 (kid.asm:14081): rising speed cap at $FFF80000.
    /// `cmpi.l #$FFF80000,d7; bge.s loc_B20E` — when d7 < $FFF80000, clamp.
    #[test]
    fn loc_b200_rising_speed_capped_at_max() {
        // Just above cap: result after gravity still above MIN
        let at_boundary = apply_gravity(MAX_RISE_SPEED, true);
        assert_eq!(at_boundary, MAX_RISE_SPEED + GRAVITY_JUMP_HELD);
        // Way below cap: clamped to MAX_RISE_SPEED ($FFF80000)
        let extreme = apply_gravity(i32::MIN / 2, true);
        assert_eq!(extreme, MAX_RISE_SPEED);
    }

    /// ASM loc_B20E (kid.asm:14086): store result — `move.l d7,y_vel(a3); rts`.
    /// All paths through loc_B1D4..loc_B20E converge here. Verify the mid-range
    /// case where neither cap triggers — the raw add is stored directly.
    #[test]
    fn loc_b20e_mid_range_stored_directly() {
        // y_vel = 0: result = $4000, well within [-$80000, $80000]
        let result = apply_gravity(0, false);
        assert_eq!(result, 0x4000);
        // y_vel = -$4_0000 with B held: -$4_0000 + $2000 = -$3_E000
        let result_asc = apply_gravity(-0x4_0000, true);
        assert_eq!(result_asc, -0x3_E000);
    }

    // -- Camera tests --

    fn level1_bounds() -> LevelBounds {
        // Level 1: 6 screens wide, 4 screens tall
        LevelBounds {
            width_pixels: 6 * 320,
            height_pixels: 4 * 224,
        }
    }

    #[test]
    fn camera_follows_player_right() {
        let bounds = level1_bounds();
        // Player at x=320 (centre-ish), camera at 0
        let (cx, _) = update_camera(0, 0, 320, 160, &bounds);
        // target = 320 - 160 = 160, delta = (160 - 0 + 4) >> 3 = 20, clamped to 8
        assert_eq!(cx, 8);
    }

    #[test]
    fn camera_delta_clamped_to_max_step() {
        let bounds = level1_bounds();
        // Large distance: delta should be clamped to ±8
        let (cx, _) = update_camera(0, 0, 1000, 160, &bounds);
        assert_eq!(cx, 8);
        // Moving left
        let (cx2, _) = update_camera(500, 0, 0, 160, &bounds);
        assert_eq!(cx2, 500 - 8);
    }

    #[test]
    fn camera_reaches_steady_state() {
        let bounds = level1_bounds();
        let player_x = 500;
        let player_y = 400;
        let mut cx = 0;
        let mut cy = 0;
        for _ in 0..2000 {
            let (nx, ny) = update_camera(cx, cy, player_x, player_y, &bounds);
            cx = nx;
            cy = ny;
        }
        // Should converge near target
        let target_x = (player_x - CAMERA_PLAYER_OFFSET).max(0);
        let target_y = (player_y - CAMERA_PLAYER_OFFSET).max(0);
        assert!((cx - target_x).abs() <= 4, "cx={cx} target_x={target_x}");
        assert!((cy - target_y).abs() <= 4, "cy={cy} target_y={target_y}");
    }

    #[test]
    fn camera_clamps_at_level_left_edge() {
        let bounds = level1_bounds();
        // Player at far left
        let (cx, _) = update_camera(0, 0, 0, 160, &bounds);
        assert!(cx >= 0);
    }

    #[test]
    fn camera_clamps_at_level_right_edge() {
        let bounds = level1_bounds();
        let max_x = i32::from(bounds.width_pixels) - VIEWPORT_WIDTH;
        // Player at far right
        let (cx, _) = update_camera(max_x, 0, i32::from(bounds.width_pixels), 160, &bounds);
        assert!(cx <= max_x);
    }

    /// ASM loc_720E (kid.asm:7322): `cmp.w d1,d0; bcs.s loc_7214; move.w d1,d0`
    /// — target is capped at max_pos (level_width - $140).  When player_x - $A0
    /// exceeds max_pos, target is clamped down.
    #[test]
    fn loc_720e_target_clamped_to_max_pos() {
        // level_width = 1920, max_pos = 1920 - 320 = 1600
        let bounds = LevelBounds {
            width_pixels: 1920,
            height_pixels: 896,
        };
        let max_pos = i32::from(bounds.width_pixels) - VIEWPORT_WIDTH; // 1600
        // Player far right: player_x = 2000, target = 2000 - 160 = 1840 > 1600
        // Camera at max_pos already, so delta = (1600 - 1600 + 4) >> 3 = 0
        let result = update_camera_axis(max_pos, 2000, max_pos);
        assert_eq!(
            result, max_pos,
            "ASM loc_720E: target exceeding max_pos should be clamped"
        );
    }

    /// ASM loc_720E (kid.asm:7319-7320): `subi.w #$A0,d0; bcc.s loc_720E;
    /// moveq #0,d0` — when player_x < $A0, the subtraction underflows (carry
    /// clear fails), so target is floored at 0.  loc_720E then checks the
    /// upper bound but it's already within range.
    #[test]
    fn loc_720e_target_floored_at_zero() {
        // Player at x=50, target = 50 - 160 = -110 → floored to 0
        // Camera at 10, delta = (0 - 10 + 4) >> 3 = -6/8 = 0 (integer)
        // Actually (0 - 10 + 4) = -6, >> 3 = -1 (arithmetic shift)
        let result = update_camera_axis(10, 50, 1600);
        assert_eq!(
            result, 9,
            "ASM loc_720E: target below 0 floored, camera moves left by 1"
        );
    }

    /// ASM loc_7214 (kid.asm:7327-7334): near-left-edge snap.
    /// When Camera_X_pos <= 4 and target < camera, velocity = -1.
    #[test]
    fn loc_7214_near_left_edge_nudge_minus_one() {
        // camera = 3 (<= 4), target = 0 (< camera) → velocity = -1
        let result = update_camera_axis(3, 0, 1600);
        assert_eq!(
            result, 2,
            "ASM loc_7214: near left edge with target<camera → nudge -1"
        );
    }

    /// ASM loc_7226 (kid.asm:7337-7344): near-right-edge snap.
    /// When Camera_X_pos >= max_pos - 4 and target > camera, velocity = +1.
    #[test]
    fn loc_7214_near_right_edge_nudge_plus_one() {
        let max_pos = 1600;
        // camera = 1597 (>= 1600 - 4 = 1596), target = 1600 (> camera)
        // Player needs to produce target = 1600 → player = 1600 + 160 = 1760
        let result = update_camera_axis(1597, 1760, max_pos);
        assert_eq!(
            result, 1598,
            "ASM loc_7226: near right edge with target>camera → nudge +1"
        );
    }

    /// ASM loc_7234 (kid.asm:7347-7351): normal smooth scroll path.
    /// `sub.w Camera_X_pos,d0; addq.w #4,d0; asr.w #3,d0` → delta clamped ±8.
    #[test]
    fn loc_7214_smooth_scroll_delta() {
        // camera = 100, player = 300, target = 300 - 160 = 140
        // delta = (140 - 100 + 4) >> 3 = 44 >> 3 = 5
        let result = update_camera_axis(100, 300, 1600);
        assert_eq!(
            result, 105,
            "ASM loc_7234: smooth scroll delta = (target - camera + 4) >> 3"
        );
    }

    /// ASM loc_723E/loc_7246 (kid.asm:7353-7361): velocity clamped to [-8, 8].
    #[test]
    fn loc_7214_velocity_clamped_to_plus_minus_8() {
        // Large positive delta: camera=0, player=1000, target=840
        // delta = (840 - 0 + 4) >> 3 = 105, clamped to 8
        let result_pos = update_camera_axis(0, 1000, 1600);
        assert_eq!(result_pos, 8, "ASM loc_723E: positive delta clamped to 8");

        // Large negative delta: camera=800, player=0, target=0
        // delta = (0 - 800 + 4) >> 3 = -99, clamped to -8
        let result_neg = update_camera_axis(800, 0, 1600);
        assert_eq!(
            result_neg, 792,
            "ASM loc_7246: negative delta clamped to -8"
        );
    }

    #[test]
    fn variable_jump_height_difference() {
        // Ascending, B held = weaker gravity
        let held = apply_gravity(-0x4_0000, true);
        // Ascending, B released = stronger gravity
        let released = apply_gravity(-0x4_0000, false);
        assert!(held < released); // held result is more negative (higher)
        assert_eq!(released - held, GRAVITY_NORMAL - GRAVITY_JUMP_HELD);
    }

    // -- Air control tests --

    #[test]
    fn air_normal_table_matches_disasm() {
        assert_eq!(AIR_NORMAL.max_air_speed, 0x1_0000);
        assert_eq!(AIR_NORMAL.air_accel, 0x1000);
        assert_eq!(AIR_NORMAL.air_above_max_decel, 0x1000);
        assert_eq!(AIR_NORMAL.air_no_input_decel, 0x1800);
        assert_eq!(AIR_NORMAL.air_turn_accel, 0x4000);
    }

    #[test]
    fn jump_air_profile_kid_matches_disasm() {
        assert_eq!(JUMP_AIR_PROFILES[0].accel, 0x2B00);
        assert_eq!(JUMP_AIR_PROFILES[0].max_speed, 0x2A000);
    }

    #[test]
    fn jump_air_profile_red_stealth_matches_disasm() {
        assert_eq!(JUMP_AIR_PROFILES[3].accel, 0x4000);
        assert_eq!(JUMP_AIR_PROFILES[3].max_speed, 0x2A000);
    }

    #[test]
    fn air_max_speed_is_lower_than_ground_walk_speed() {
        assert!(AIR_NORMAL.max_air_speed < NORMAL_KID.max_walk_speed);
    }

    #[test]
    fn air_right_accelerates_from_standstill() {
        let (vel, dir) =
            update_air_horizontal_velocity(0, Direction::Right, right_walk(), &AIR_NORMAL);
        assert_eq!(vel, AIR_NORMAL.air_accel);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn air_right_caps_at_max_air_speed() {
        let mut vel = 0i32;
        for _ in 0..1000 {
            let (v, _) =
                update_air_horizontal_velocity(vel, Direction::Right, right_walk(), &AIR_NORMAL);
            vel = v;
        }
        assert_eq!(vel, AIR_NORMAL.max_air_speed);
    }

    #[test]
    fn air_preserves_momentum_above_max_but_decelerates() {
        // Start at ground run speed — air control should decelerate towards max_air_speed
        let vel = NORMAL_KID.max_run_speed; // 0x38000
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Right, right_walk(), &AIR_NORMAL);
        assert!(result < vel, "should decelerate");
        assert!(
            result >= AIR_NORMAL.max_air_speed,
            "should not drop below max_air_speed"
        );
    }

    #[test]
    fn air_hard_cap_at_run_speed() {
        // Velocity above hard cap should be clamped
        let vel = AIR_HARD_CAP + 0x1_0000;
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Right, right_walk(), &AIR_NORMAL);
        assert!(result <= AIR_HARD_CAP);
    }

    #[test]
    fn air_turn_uses_turn_accel() {
        // Moving left, holding right → turn with air_turn_accel
        let vel = -0x2_0000;
        let (result, dir) =
            update_air_horizontal_velocity(vel, Direction::Left, right_walk(), &AIR_NORMAL);
        assert_eq!(result, vel + AIR_NORMAL.air_turn_accel);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn air_no_input_decelerates_towards_zero() {
        let vel = 0x2_0000;
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Right, no_input(), &AIR_NORMAL);
        assert_eq!(result, vel - AIR_NORMAL.air_no_input_decel);
    }

    #[test]
    fn air_no_input_snaps_to_zero() {
        let vel = AIR_NORMAL.air_no_input_decel - 1;
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Right, no_input(), &AIR_NORMAL);
        assert_eq!(result, 0);
    }

    #[test]
    fn air_left_caps_at_max_air_speed() {
        let mut vel = 0i32;
        for _ in 0..1000 {
            let (v, _) =
                update_air_horizontal_velocity(vel, Direction::Left, left_walk(), &AIR_NORMAL);
            vel = v;
        }
        assert_eq!(vel, -AIR_NORMAL.max_air_speed);
    }

    #[test]
    fn air_turn_response_slower_than_ground() {
        // Air turn uses air_turn_accel ($4000), ground turn uses turn_brake ($2000)
        // or strong_brake ($3000). Air turn is actually faster per-frame, but the
        // reduced max speed means reaching target speed takes proportionally longer.
        // The key difference is the lower max_air_speed cap.
        assert!(AIR_NORMAL.max_air_speed < NORMAL_KID.max_walk_speed);
    }

    #[test]
    fn jump_air_right_accelerates_above_generic_air_cap() {
        let mut vel = 0;
        for _ in 0..16 {
            (vel, _) = update_jump_horizontal_velocity(
                vel,
                Direction::Right,
                right_walk(),
                JUMP_AIR_PROFILES[0],
            );
        }
        assert!(vel > AIR_NORMAL.max_air_speed);
    }

    #[test]
    fn jump_air_run_held_uses_stronger_accel() {
        let (walk_vel, _) = update_jump_horizontal_velocity(
            0,
            Direction::Right,
            right_walk(),
            JUMP_AIR_PROFILES[0],
        );
        let (run_vel, _) =
            update_jump_horizontal_velocity(0, Direction::Right, right_run(), JUMP_AIR_PROFILES[0]);

        assert_eq!(walk_vel, JUMP_AIR_PROFILES[0].accel);
        assert_eq!(
            run_vel,
            JUMP_AIR_PROFILES[0].accel + (JUMP_AIR_PROFILES[0].accel >> 1)
        );
    }

    #[test]
    fn jump_air_no_input_decelerates_slowly() {
        let (result, _) = update_jump_horizontal_velocity(
            0x2_0000,
            Direction::Right,
            no_input(),
            JUMP_AIR_PROFILES[0],
        );
        assert_eq!(result, 0x1FC00);
    }

    // -----------------------------------------------------------------------
    // ASM-backed constant verification — values cross-referenced against
    // kid-chameleon-disasm to catch drift between native and original.
    // -----------------------------------------------------------------------

    #[test]
    fn asm_normal_kid_speed_table_matches_unk_9c0a() {
        // unk_9C0A: 7 longwords for Kid on normal terrain
        assert_eq!(NORMAL_KID.max_walk_speed, 0x2_0000);
        assert_eq!(NORMAL_KID.walk_accel, 0x1000);
        assert_eq!(NORMAL_KID.walk_run_decel, 0x2800);
        assert_eq!(NORMAL_KID.turn_brake, 0x2000);
        assert_eq!(NORMAL_KID.max_run_speed, 0x3_8000);
        assert_eq!(NORMAL_KID.run_accel, 0x1C00);
        assert_eq!(NORMAL_KID.strong_brake, 0x3000);
    }

    #[test]
    fn asm_air_hard_cap_matches_ground_run_cap() {
        // Air accumulated speed must not exceed ground run cap
        assert_eq!(AIR_HARD_CAP, NORMAL_KID.max_run_speed);
    }

    #[test]
    fn asm_jump_heights_all_helmets_match_unk_a49e() {
        // unk_A49E: per-helmet jump factor table (10 entries)
        let expected: [u16; 10] = [
            0x42, // Kid
            0x42, // Skycutter
            0x42, // Cyclone
            0x4A, // RedStealth
            0x42, // Eyeclops
            0x42, // Juggernaut
            0x42, // IronKnight
            0x42, // Berzerker
            0x42, // Maniaxe
            0x42, // Micromax
        ];
        assert_eq!(JUMP_HEIGHTS, expected);
    }

    #[test]
    fn asm_skycutter_jump_air_profile_is_fastest() {
        // Skycutter (index 1) has highest accel and max_speed in air
        assert_eq!(JUMP_AIR_PROFILES[1].accel, 0x3400);
        assert_eq!(JUMP_AIR_PROFILES[1].max_speed, 0x32000);
        assert!(JUMP_AIR_PROFILES[1].max_speed > JUMP_AIR_PROFILES[0].max_speed);
    }

    #[test]
    fn asm_red_stealth_jump_air_profile_has_strongest_accel() {
        assert_eq!(JUMP_AIR_PROFILES[3].accel, 0x4000);
        assert_eq!(JUMP_AIR_PROFILES[3].max_speed, 0x2A000);
    }

    // -- Left-facing ground movement (apply_holding_left) --
    // These kill mutants in the left-mirror of apply_holding_right.

    fn left_run() -> HorizontalInput {
        HorizontalInput {
            left: true,
            right: false,
            run: true,
        }
    }

    #[test]
    fn walk_left_single_frame_from_standstill() {
        // loc_9B90: walking left from stopped should apply -walk_accel
        let (vel, dir) = update_horizontal_velocity(0, Direction::Left, left_walk(), &NORMAL_KID);
        assert_eq!(vel, -NORMAL_KID.strong_brake); // stopped uses strong_brake path (ASM: vel=0 → bpl taken)
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn run_left_accelerates_from_moving_left() {
        // loc_9BCE: running left when already moving left → accelerate with run_accel
        let vel = -0x1_0000;
        let (result, _) = update_horizontal_velocity(vel, Direction::Left, left_run(), &NORMAL_KID);
        // abs(vel) + run_accel = 0x10000 + 0x1C00 = 0x11C00, negated
        assert_eq!(result, -(0x1_0000 + NORMAL_KID.run_accel));
    }

    #[test]
    fn run_left_caps_at_max_run_speed() {
        let mut vel = 0i32;
        for _ in 0..1000 {
            let (v, _) = update_horizontal_velocity(vel, Direction::Left, left_run(), &NORMAL_KID);
            vel = v;
        }
        assert_eq!(vel, -NORMAL_KID.max_run_speed);
    }

    #[test]
    fn run_left_from_moving_right_uses_strong_brake() {
        // loc_9BE8: running left but vel >= 0 → strong_brake
        let vel = NORMAL_KID.max_walk_speed;
        let (result, _) = update_horizontal_velocity(vel, Direction::Left, left_run(), &NORMAL_KID);
        assert_eq!(result, vel - NORMAL_KID.strong_brake);
    }

    #[test]
    fn walk_left_above_walk_cap_decelerates() {
        // Walking left at above-walk speed: decel with walk_run_decel
        let vel = -(NORMAL_KID.max_walk_speed + 0x8000);
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Left, left_walk(), &NORMAL_KID);
        // vel + walk_run_decel (towards zero, i.e. less negative)
        assert_eq!(result, vel + NORMAL_KID.walk_run_decel);
    }

    #[test]
    fn walk_left_above_run_cap_snaps_to_run_cap() {
        // Walking left at above-run speed: snap to -max_run_speed
        let vel = -(NORMAL_KID.max_run_speed + 0x1_0000);
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Left, left_walk(), &NORMAL_KID);
        assert_eq!(result, -NORMAL_KID.max_run_speed);
    }

    #[test]
    fn walk_left_below_walk_cap_accelerates() {
        // Walking left, speed below walk cap: accelerate with walk_accel
        let vel = -0x1_0000; // below max_walk (0x2_0000)
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Left, left_walk(), &NORMAL_KID);
        assert_eq!(result, -(0x1_0000 + NORMAL_KID.walk_accel));
    }

    #[test]
    fn walk_left_acceleration_caps_at_walk_speed() {
        // Just below max_walk: accel should not overshoot
        let vel = -(NORMAL_KID.max_walk_speed - 0x100);
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Left, left_walk(), &NORMAL_KID);
        assert_eq!(result, -NORMAL_KID.max_walk_speed);
    }

    // -- Ground decelerate (no input) --

    #[test]
    fn decel_positive_subtracts_walk_run_decel() {
        // loc_9BEE: vel > 0, no input → subtract walk_run_decel
        let vel = NORMAL_KID.max_walk_speed;
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Right, no_input(), &NORMAL_KID);
        assert_eq!(result, vel - NORMAL_KID.walk_run_decel);
    }

    #[test]
    fn decel_negative_adds_walk_run_decel() {
        // loc_9BEE: vel < 0, no input → add walk_run_decel (towards zero)
        let vel = -NORMAL_KID.max_walk_speed;
        let (result, _) = update_horizontal_velocity(vel, Direction::Left, no_input(), &NORMAL_KID);
        assert_eq!(result, vel + NORMAL_KID.walk_run_decel);
    }

    #[test]
    fn decel_negative_snaps_to_zero() {
        // Small negative velocity: should snap to 0, not go positive
        let vel = -(NORMAL_KID.walk_run_decel - 1);
        let (result, _) = update_horizontal_velocity(vel, Direction::Left, no_input(), &NORMAL_KID);
        assert_eq!(result, 0);
    }

    #[test]
    fn decel_zero_stays_zero() {
        let (result, _) = update_horizontal_velocity(0, Direction::Right, no_input(), &NORMAL_KID);
        assert_eq!(result, 0);
    }

    // -- Air decelerate (no input) --

    #[test]
    fn air_decel_negative_adds_towards_zero() {
        // Symmetric: vel < 0, no input → add air_no_input_decel
        let vel = -0x2_0000;
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Left, no_input(), &AIR_NORMAL);
        assert_eq!(result, vel + AIR_NORMAL.air_no_input_decel);
    }

    #[test]
    fn air_decel_negative_snaps_to_zero() {
        let vel = -(AIR_NORMAL.air_no_input_decel - 1);
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Left, no_input(), &AIR_NORMAL);
        assert_eq!(result, 0);
    }

    // -- Air holding left (apply_air_holding_left) --

    #[test]
    fn air_left_from_standstill_accelerates_left() {
        let (vel, dir) =
            update_air_horizontal_velocity(0, Direction::Left, left_walk(), &AIR_NORMAL);
        assert_eq!(vel, -AIR_NORMAL.air_accel);
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn air_left_turn_from_moving_right() {
        // Moving right, holding left → turn with air_turn_accel (subtract)
        let vel = 0x2_0000;
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Right, left_walk(), &AIR_NORMAL);
        assert_eq!(result, vel - AIR_NORMAL.air_turn_accel);
    }

    #[test]
    fn air_left_above_max_decelerates() {
        // Moving left above max_air_speed: decel towards max
        let vel = -(AIR_NORMAL.max_air_speed + 0x8000);
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Left, left_walk(), &AIR_NORMAL);
        // abs decreases by air_above_max_decel
        let expected_abs = vel.saturating_neg() - AIR_NORMAL.air_above_max_decel;
        assert_eq!(result, -expected_abs);
    }

    // -- Jump horizontal velocity (update_jump_horizontal_velocity) --

    #[test]
    fn jump_air_left_accelerates() {
        // Holding left from standstill: should go negative
        let (vel, dir) =
            update_jump_horizontal_velocity(0, Direction::Right, left_walk(), JUMP_AIR_PROFILES[0]);
        assert!(vel < 0, "holding left should produce negative velocity");
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn jump_air_left_exact_value() {
        // From standstill, left walk: vel = 0 - accel (via the left branch)
        // But: 0 is not < 0 (min_speed check path), so: next = 0 - accel = -accel
        // -accel >= min_speed (-max_speed) since accel < max_speed, so result = -accel
        let profile = JUMP_AIR_PROFILES[0];
        let (vel, _) = update_jump_horizontal_velocity(0, Direction::Right, left_walk(), profile);
        assert_eq!(vel, -profile.accel);
    }

    #[test]
    fn jump_air_left_at_max_speed_clamps() {
        // Already at -max_speed: result should stay at limit, not go further
        let profile = JUMP_AIR_PROFILES[0];
        let (vel, _) = update_jump_horizontal_velocity(
            -profile.max_speed,
            Direction::Left,
            left_walk(),
            profile,
        );
        // next = -max_speed - accel → below min_speed → fallback: vel + 0x100
        assert_eq!(vel, -profile.max_speed + 0x100);
    }

    #[test]
    fn jump_air_no_input_decel_positive() {
        // No input, positive velocity: should decelerate by 0x400, snapping to zero
        let profile = JUMP_AIR_PROFILES[0];
        let (result, _) =
            update_jump_horizontal_velocity(0x2_0000, Direction::Right, no_input(), profile);
        assert_eq!(result, 0x2_0000 - 0x400);
    }

    #[test]
    fn jump_air_no_input_decel_negative() {
        // No input, negative velocity: should decelerate towards zero
        let profile = JUMP_AIR_PROFILES[0];
        let (result, _) =
            update_jump_horizontal_velocity(-0x2_0000, Direction::Left, no_input(), profile);
        assert_eq!(result, -0x2_0000 + 0x400);
    }

    #[test]
    fn jump_air_no_input_snap_positive_to_zero() {
        let profile = JUMP_AIR_PROFILES[0];
        let (result, _) =
            update_jump_horizontal_velocity(0x100, Direction::Right, no_input(), profile);
        assert_eq!(result, 0, "small positive vel should snap to zero");
    }

    #[test]
    fn jump_air_no_input_snap_negative_to_zero() {
        let profile = JUMP_AIR_PROFILES[0];
        let (result, _) =
            update_jump_horizontal_velocity(-0x100, Direction::Left, no_input(), profile);
        assert_eq!(result, 0, "small negative vel should snap to zero");
    }

    /// ASM loc_B09E (kid.asm:13930): run-held multiplies accel by 1.5.
    /// `move.l d1,d2; asr.l #1,d2; add.l d2,d1` — d1 = accel + accel/2.
    #[test]
    fn loc_b09e_run_held_accel_is_1_5x() {
        let profile = JUMP_AIR_PROFILES[0]; // Kid: accel=0x2B00
        let (walk_vel, _) =
            update_jump_horizontal_velocity(0, Direction::Right, right_walk(), profile);
        let (run_vel, _) =
            update_jump_horizontal_velocity(0, Direction::Right, right_run(), profile);
        // Walk: accel = 0x2B00. Run: 0x2B00 + (0x2B00 >> 1) = 0x2B00 + 0x1580 = 0x4080
        assert_eq!(walk_vel, 0x2B00);
        assert_eq!(run_vel, 0x4080);
        assert_eq!(run_vel, profile.accel + (profile.accel >> 1));
    }

    /// ASM loc_B0B6 (kid.asm:13941): left-held path — `st x_direction(a3);
    /// sub.l d1,d0; bpl.s loc_B118` — still-positive result accepted directly.
    #[test]
    fn loc_b0b6_left_held_positive_result_accepted() {
        let profile = JUMP_AIR_PROFILES[0];
        // Start with large positive vel so subtracting accel stays positive.
        let start = 0x4_0000;
        let (vel, dir) =
            update_jump_horizontal_velocity(start, Direction::Right, left_walk(), profile);
        assert_eq!(vel, start - profile.accel);
        assert_eq!(dir, Direction::Left, "st x_direction → face left");
    }

    /// ASM loc_B0B6 left path — past max speed: `neg.l d2; cmp.l d0,d2;
    /// ble.s loc_B118` fails → revert: `add.l d1,d0; addi.l #$100,d0`.
    #[test]
    fn loc_b0b6_left_past_max_speed_reverts_plus_100() {
        let profile = JUMP_AIR_PROFILES[0]; // max_speed=0x2A000
        let start = -(profile.max_speed + 1); // just past negative cap
        let (vel, _) =
            update_jump_horizontal_velocity(start, Direction::Left, left_walk(), profile);
        // next = start - accel, which is < -max_speed → fallback = start + 0x100
        assert_eq!(vel, start + 0x100);
    }

    /// ASM loc_B0E2 (kid.asm:13959): right-held, result still negative —
    /// `add.l d1,d0; bmi.s loc_B118` accepts negative result directly.
    #[test]
    fn loc_b0e2_right_held_negative_result_accepted() {
        let profile = JUMP_AIR_PROFILES[0]; // accel=0x2B00
        let start = -0x4_0000;
        let (vel, dir) =
            update_jump_horizontal_velocity(start, Direction::Right, right_walk(), profile);
        assert_eq!(vel, start + profile.accel);
        assert!(vel < 0, "result should still be negative");
        assert_eq!(dir, Direction::Right, "sf x_direction → face right");
    }

    /// ASM loc_B0E2 right path — past max speed: `cmp.l d0,d2; bge.s loc_B118`
    /// fails → revert: `sub.l d1,d0; subi.l #$100,d0`.
    #[test]
    fn loc_b0e2_right_past_max_speed_reverts_minus_100() {
        let profile = JUMP_AIR_PROFILES[0]; // max_speed=0x2A000
        let start = profile.max_speed + 1; // just past positive cap
        let (vel, _) =
            update_jump_horizontal_velocity(start, Direction::Right, right_walk(), profile);
        assert_eq!(vel, start - 0x100);
    }

    /// ASM loc_B0FE (kid.asm:13972): no L/R, positive vel — `subi.l #$400,d0;
    /// bpl.s loc_B118; moveq #0,d0`. Decelerate by $400, clamp to 0.
    #[test]
    fn loc_b0fe_no_input_positive_decel_exact() {
        let profile = JUMP_AIR_PROFILES[0];
        // Exactly $400: decel to 0
        let (vel, _) =
            update_jump_horizontal_velocity(0x400, Direction::Right, no_input(), profile);
        assert_eq!(vel, 0);
        // $401: decel to 1 (still positive, bpl taken)
        let (vel2, _) =
            update_jump_horizontal_velocity(0x401, Direction::Right, no_input(), profile);
        assert_eq!(vel2, 1);
        // $3FF: decel overshoots negative, clamp to 0 (bpl not taken → moveq #0)
        let (vel3, _) =
            update_jump_horizontal_velocity(0x3FF, Direction::Right, no_input(), profile);
        assert_eq!(vel3, 0);
    }

    /// ASM loc_B10E (kid.asm:13981): no L/R, negative vel — `addi.l #$400,d0;
    /// bmi.s loc_B118; moveq #0,d0`. Decelerate toward 0, clamp.
    #[test]
    fn loc_b10e_no_input_negative_decel_exact() {
        let profile = JUMP_AIR_PROFILES[0];
        // Exactly -$400: decel to 0
        let (vel, _) =
            update_jump_horizontal_velocity(-0x400, Direction::Left, no_input(), profile);
        assert_eq!(vel, 0);
        // -$401: decel to -1 (still negative, bmi taken)
        let (vel2, _) =
            update_jump_horizontal_velocity(-0x401, Direction::Left, no_input(), profile);
        assert_eq!(vel2, -1);
        // -$3FF: decel overshoots positive, clamp to 0 (bmi not taken → moveq #0)
        let (vel3, _) =
            update_jump_horizontal_velocity(-0x3FF, Direction::Left, no_input(), profile);
        assert_eq!(vel3, 0);
    }

    // -- Camera edge nudge (update_camera_axis) --

    #[test]
    fn camera_near_left_edge_nudges_left() {
        // camera <= 4 && target < camera → nudge at -1
        // camera=3, player=0 → target = max(0 - 0xA0, 0) = 0, target < camera
        let max_pos = 1600;
        let result = update_camera_axis(3, 0, max_pos);
        assert_eq!(result, 2); // 3 + (-1) = 2
    }

    #[test]
    fn camera_near_right_edge_nudges_right() {
        // camera >= max_pos - 4 && target > camera → nudge at +1
        let max_pos = 1600;
        // camera = max_pos - 3, player far right so target = max_pos
        let camera = max_pos - 3;
        let result = update_camera_axis(camera, max_pos + CAMERA_PLAYER_OFFSET, max_pos);
        assert_eq!(result, camera + 1);
    }

    #[test]
    fn camera_mid_field_uses_delta_formula() {
        // Not near edge: delta = (target - camera + 4) >> 3
        let max_pos = 1600;
        let camera = 400;
        let player = 400 + CAMERA_PLAYER_OFFSET + 80; // target = 480
        let result = update_camera_axis(camera, player, max_pos);
        let expected_delta = (480 - 400 + 4) >> 3; // 84 >> 3 = 10, clamped to 8
        assert_eq!(result, camera + 8);
        assert_eq!(expected_delta, 10); // confirms clamping was needed
    }

    // -- Right-facing edge cases (apply_holding_right / apply_air_holding_right) --

    #[test]
    fn run_right_from_moving_left_uses_strong_brake() {
        // loc_9B62: running right but vel < 0 → vel + strong_brake (towards zero)
        let vel = -NORMAL_KID.max_walk_speed;
        let (result, _) =
            update_horizontal_velocity(vel, Direction::Right, right_run(), &NORMAL_KID);
        assert_eq!(result, vel + NORMAL_KID.strong_brake);
        assert!(result > vel, "should move towards zero (less negative)");
    }

    #[test]
    fn air_right_above_max_decelerates_exact() {
        // loc_852C: moving right above max_air_speed → decel by air_above_max_decel
        let vel = AIR_NORMAL.max_air_speed + 0x8000;
        let (result, _) =
            update_air_horizontal_velocity(vel, Direction::Right, right_walk(), &AIR_NORMAL);
        assert_eq!(result, vel - AIR_NORMAL.air_above_max_decel);
    }

    /// ASM loc_A44A (kid.asm:12792): after `bpl.s` / `neg.l d1`, d1 holds
    /// abs(x_vel).  The remaining chain `lsl.l #4; swap; addi.w #$100;
    /// mulu.w; lsl.l #4; neg.l` computes the upward launch velocity.
    /// This test verifies a concrete running-jump scenario end-to-end.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A44A_abs_x_vel_jump_height_scaling() {
        // x_vel = -0x2_0000 (moving left at walk speed)
        // After neg: abs = 0x2_0000
        // lsl #4: 0x20_0000, swap: 0x0020
        // +$100 = 0x0120
        // mulu 0x42 * 0x120 = 0x4A40
        // lsl #4: 0x4_A400
        // neg: -0x4_A400
        let (_, y_vel) = initiate_jump(-0x2_0000, JUMP_HEIGHTS[0]);
        assert_eq!(
            y_vel, -0x4_A400,
            "loc_A44A: negative x_vel should produce same y_vel as positive"
        );

        // Verify against positive x_vel (same abs)
        let (_, y_vel_pos) = initiate_jump(0x2_0000, JUMP_HEIGHTS[0]);
        assert_eq!(y_vel, y_vel_pos);
    }

    /// ASM loc_A466 (kid.asm:12803): `tst.b (KidGrabbedByHand); beq.w loc_A466;
    /// addi.l #$20000,d0` — when KidGrabbedByHand is set, y_vel is weakened
    /// by adding $20000 (making the negative value less negative → lower jump).
    /// This test verifies the delta: grabbed jump = ungrabbed jump + $20000.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A466_grabbed_by_hand_weakens_jump() {
        // Normal jump y_vel (KidGrabbedByHand = false)
        let (_, y_vel_normal) = initiate_jump(0x2_0000, JUMP_HEIGHTS[0]);
        // Grabbed adjustment: y_vel += $20000 (closer to zero)
        let y_vel_grabbed = y_vel_normal + 0x2_0000;
        assert!(
            y_vel_grabbed > y_vel_normal,
            "loc_A466: grabbed y_vel ({y_vel_grabbed:#x}) should be closer to zero \
             than normal ({y_vel_normal:#x})"
        );
        // Grabbed jump is still upward (negative)
        assert!(
            y_vel_grabbed < 0,
            "loc_A466: grabbed y_vel ({y_vel_grabbed:#x}) should still be negative (upward)"
        );
    }

    /// ASM loc_A480 (kid.asm:12813): `move.l x_vel(a3),d0; asr.l #1,d0;
    /// move.l d0,d1; asr.l #1,d1; add.l d1,d0; move.l d0,x_vel(a3)` — the
    /// jump reduces x_vel to 3/4 of its original value via arithmetic shift.
    /// Verify exact values for representative speeds.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A480_x_vel_three_quarter_reduction() {
        // Walking speed: 0x2_0000
        // asr #1 → 0x1_0000; asr #1 → 0x8000; sum = 0x1_8000
        let (new_x, _) = initiate_jump(0x2_0000, JUMP_HEIGHTS[0]);
        assert_eq!(new_x, 0x1_8000, "loc_A480: 0x2_0000 * 3/4 = 0x1_8000");

        // Negative walking speed: -0x2_0000
        // asr #1 → -0x1_0000; asr #1 → -0x8000; sum = -0x1_8000
        let (new_x_neg, _) = initiate_jump(-0x2_0000, JUMP_HEIGHTS[0]);
        assert_eq!(
            new_x_neg, -0x1_8000,
            "loc_A480: -0x2_0000 * 3/4 = -0x1_8000"
        );

        // Odd value: -1 (ASR rounding toward -∞)
        // asr #1 → -1; asr #1 → -1; sum = -2
        let (new_x_odd, _) = initiate_jump(-1, JUMP_HEIGHTS[0]);
        assert_eq!(
            new_x_odd, -2,
            "loc_A480: -1 * 3/4 via ASR = -2 (rounds toward -∞)"
        );

        // Zero stays zero
        let (new_x_zero, _) = initiate_jump(0, JUMP_HEIGHTS[0]);
        assert_eq!(new_x_zero, 0, "loc_A480: 0 * 3/4 = 0");
    }

    // -- initiate_jump negative x_vel sign --

    #[test]
    fn initiate_jump_negative_x_vel_produces_same_y_as_positive() {
        // abs(-x) == abs(x), so y_vel should be identical
        let (_, y_pos) = initiate_jump(0x2_0000, JUMP_HEIGHTS[0]);
        let (_, y_neg) = initiate_jump(-0x2_0000, JUMP_HEIGHTS[0]);
        assert_eq!(y_pos, y_neg);
    }

    /// ASM loc_71EC (kid.asm:7310): `beq.w loc_73C2` — when `Addr_GfxObject_Kid`
    /// is null (d7=0 from `move.l (Addr_GfxObject_Kid).w,d7`), sub_71E4 branches
    /// straight to loc_73C2.  loc_73C2 writes back the original d2/d3 scroll
    /// values (loaded at entry from $FFFFF892/$FFFFF894) and returns — the camera
    /// position is unchanged.
    ///
    /// In the native port, `update_camera` is a pure function: calling it with
    /// `player == camera + CAMERA_PLAYER_OFFSET` (i.e. player centred) produces
    /// zero delta, equivalent to the null-bail "camera doesn't move" behaviour.
    #[test]
    fn loc_71ec_null_kid_bail_camera_unchanged() {
        let camera_x = 100;
        let camera_y = 50;
        let bounds = LevelBounds {
            width_pixels: 1024,
            height_pixels: 512,
        };
        // Simulate the null-bail: if Kid doesn't exist, no update_camera call
        // happens and camera stays put.  Equivalently, calling update_camera with
        // player positioned exactly at camera + offset produces no movement.
        let player_x = camera_x + CAMERA_PLAYER_OFFSET;
        let player_y = camera_y + CAMERA_PLAYER_OFFSET;
        let (new_cx, new_cy) = update_camera(camera_x, camera_y, player_x, player_y, &bounds);
        assert_eq!(
            (new_cx, new_cy),
            (camera_x, camera_y),
            "ASM loc_71EC: when Kid is null, camera position is unchanged — \
             native equivalent: centred player produces zero delta",
        );
    }
}

// ---------------------------------------------------------------------------
// Property-based tests (proptest)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // All 9 ground speed tables for parameterised properties.
    const ALL_SPEED_TABLES: [(&str, SpeedTable); 9] = [
        ("normal_kid", NORMAL_KID),
        ("normal_micromax", NORMAL_MICROMAX),
        ("normal_juggernaut", NORMAL_JUGGERNAUT),
        ("ice_kid", ICE_KID),
        ("ice_micromax", ICE_MICROMAX),
        ("ice_juggernaut", ICE_JUGGERNAUT),
        ("rubber_kid", RUBBER_KID),
        ("rubber_micromax", RUBBER_MICROMAX),
        ("rubber_juggernaut", RUBBER_JUGGERNAUT),
    ];

    const ALL_AIR_TABLES: [(&str, AirSpeedTable); 3] = [
        ("air_normal", AIR_NORMAL),
        ("air_ice", AIR_ICE),
        ("air_rubber", AIR_RUBBER),
    ];

    // -- Speed table structural invariants --

    #[test]
    fn all_speed_tables_well_formed() {
        for (name, t) in &ALL_SPEED_TABLES {
            assert!(
                t.max_walk_speed <= t.max_run_speed,
                "{name}: max_walk ({:#x}) > max_run ({:#x})",
                t.max_walk_speed,
                t.max_run_speed
            );
            assert!(
                t.max_walk_speed > 0,
                "{name}: max_walk_speed must be positive"
            );
            assert!(
                t.max_run_speed > 0,
                "{name}: max_run_speed must be positive"
            );
            assert!(t.walk_accel > 0, "{name}: walk_accel must be positive");
            assert!(t.run_accel > 0, "{name}: run_accel must be positive");
            assert!(
                t.walk_run_decel > 0,
                "{name}: walk_run_decel must be positive"
            );
            assert!(t.turn_brake > 0, "{name}: turn_brake must be positive");
            assert!(t.strong_brake > 0, "{name}: strong_brake must be positive");
        }
    }

    #[test]
    fn all_air_tables_well_formed() {
        for (name, t) in &ALL_AIR_TABLES {
            assert!(
                t.max_air_speed > 0,
                "{name}: max_air_speed must be positive"
            );
            assert!(t.air_accel > 0, "{name}: air_accel must be positive");
            assert!(
                t.air_above_max_decel > 0,
                "{name}: air_above_max_decel must be positive"
            );
            assert!(
                t.air_no_input_decel > 0,
                "{name}: air_no_input_decel must be positive"
            );
            assert!(
                t.air_turn_accel > 0,
                "{name}: air_turn_accel must be positive"
            );
        }
    }

    // -- update_horizontal_velocity properties --

    fn table_index() -> impl Strategy<Value = usize> {
        0..ALL_SPEED_TABLES.len()
    }

    fn horizontal_input_strategy() -> impl Strategy<Value = HorizontalInput> {
        (any::<bool>(), any::<bool>(), any::<bool>())
            .prop_map(|(left, right, run)| HorizontalInput { left, right, run })
    }

    fn facing_strategy() -> impl Strategy<Value = Direction> {
        prop_oneof![Just(Direction::Left), Just(Direction::Right)]
    }

    proptest! {
        /// Ground velocity moves towards the target range: when starting within
        /// max_run_speed, output never exceeds max_run_speed + one frame of force.
        /// When starting outside, output moves closer to zero than input.
        #[test]
        fn ground_velocity_within_range_stays_bounded(
            table_idx in table_index(),
            start_vel in -0x8_0000i32..=0x8_0000,
            input in horizontal_input_strategy(),
            facing in facing_strategy(),
        ) {
            let (_, table) = ALL_SPEED_TABLES[table_idx];
            // Clamp start_vel to max_run_speed (the game never produces higher)
            let clamped = start_vel.clamp(-table.max_run_speed, table.max_run_speed);
            let (result, _) = update_horizontal_velocity(clamped, facing, input, &table);
            // Within game range, output is bounded by max_run_speed + one frame of
            // strongest applied force (brake can overshoot zero).
            let max_force = table.strong_brake
                .max(table.run_accel)
                .max(table.turn_brake)
                .max(table.walk_run_decel);
            let bound = table.max_run_speed + max_force;
            prop_assert!(
                result.abs() <= bound,
                "result {result:#x} exceeds bound {bound:#x} for table {}",
                ALL_SPEED_TABLES[table_idx].0
            );
        }

        /// Walking converges: iterating walk-right from any sub-walk velocity
        /// reaches max_walk_speed within a bounded number of frames.
        #[test]
        fn walk_converges_to_max_walk_speed(
            table_idx in table_index(),
            start_vel in 0i32..=0x1_0000,
        ) {
            let (_, table) = ALL_SPEED_TABLES[table_idx];
            let input = HorizontalInput { left: false, right: true, run: false };
            let mut vel = start_vel.min(table.max_walk_speed);
            for _ in 0..5000 {
                let (v, _) = update_horizontal_velocity(vel, Direction::Right, input, &table);
                vel = v;
                if vel == table.max_walk_speed {
                    break;
                }
            }
            prop_assert_eq!(vel, table.max_walk_speed);
        }

        /// Deceleration from any positive velocity reaches zero.
        #[test]
        fn decel_reaches_zero(
            table_idx in table_index(),
            start_vel in 1i32..=0x8_0000,
        ) {
            let (_, table) = ALL_SPEED_TABLES[table_idx];
            let input = HorizontalInput { left: false, right: false, run: false };
            let mut vel = start_vel;
            for _ in 0..10000 {
                let (v, _) = update_horizontal_velocity(vel, Direction::Right, input, &table);
                vel = v;
                if vel == 0 { break; }
            }
            prop_assert_eq!(vel, 0);
        }
    }

    // -- apply_gravity properties --

    proptest! {
        /// Gravity result is always clamped to [MAX_RISE_SPEED, MAX_FALL_SPEED].
        #[test]
        fn gravity_result_clamped(
            y_vel in prop::num::i32::ANY,
            b_held in any::<bool>(),
        ) {
            let result = apply_gravity(y_vel, b_held);
            prop_assert!(result >= MAX_RISE_SPEED, "result {result} < MAX_RISE_SPEED");
            prop_assert!(result <= MAX_FALL_SPEED, "result {result} > MAX_FALL_SPEED");
        }

        /// Holding B while ascending always produces weaker gravity than not.
        #[test]
        fn jump_held_weaker_gravity(y_vel in MAX_RISE_SPEED..0i32) {
            let held = apply_gravity(y_vel, true);
            let released = apply_gravity(y_vel, false);
            prop_assert!(held <= released, "held={held} > released={released}");
        }
    }

    // -- initiate_jump properties --

    proptest! {
        /// Jump reduces x velocity magnitude via 3/4 ASR factor.
        /// M68K `asr.l #1` rounds toward -∞, so for x_vel = -1:
        /// half = -1, quarter = -1, new = -2 (magnitude +1 overshoot).
        /// Allow a 1-unit tolerance to account for this faithful quirk.
        #[test]
        fn jump_reduces_x_velocity(
            x_vel in -0x8_0000i32..=0x8_0000,
            height_idx in 0usize..10,
        ) {
            let (new_x, _) = initiate_jump(x_vel, JUMP_HEIGHTS[height_idx]);
            prop_assert!(
                new_x.abs() <= x_vel.abs() + 1,
                "|new_x| ({}) > |x_vel| ({}) + 1 (ASR rounding tolerance)",
                new_x.abs(),
                x_vel.abs()
            );
        }

        /// Jump always produces upward (negative) y velocity.
        #[test]
        fn jump_produces_upward_velocity(
            x_vel in -0x8_0000i32..=0x8_0000,
            height_idx in 0usize..10,
        ) {
            let (_, y_vel) = initiate_jump(x_vel, JUMP_HEIGHTS[height_idx]);
            prop_assert!(y_vel < 0, "y_vel {y_vel} should be negative (upward)");
        }

        /// Higher x speed produces higher jump (more negative y_vel).
        #[test]
        fn faster_horizontal_means_higher_jump(
            slow in 0i32..0x2_0000,
            fast in 0x2_0000i32..0x8_0000,
            height_idx in 0usize..10,
        ) {
            let (_, y_slow) = initiate_jump(slow, JUMP_HEIGHTS[height_idx]);
            let (_, y_fast) = initiate_jump(fast, JUMP_HEIGHTS[height_idx]);
            prop_assert!(
                y_fast <= y_slow,
                "faster ({fast:#x}) should jump at least as high: y_fast={y_fast}, y_slow={y_slow}"
            );
        }
    }

    // -- update_camera_axis properties --

    proptest! {
        /// Camera position always stays within [0, max_pos].
        #[test]
        fn camera_stays_in_bounds(
            camera_x in 0i32..2000,
            camera_y in 0i32..1000,
            player_x in 0i32..3000,
            player_y in 0i32..2000,
            width in 320i32..4000,
            height in 224i32..2000,
        ) {
            let bounds = LevelBounds {
                width_pixels: width as u16,
                height_pixels: height as u16,
            };
            let max_x = width - VIEWPORT_WIDTH;
            let max_y = height - VIEWPORT_HEIGHT;
            let cx = camera_x.min(max_x).max(0);
            let cy = camera_y.min(max_y).max(0);
            let (new_cx, new_cy) = update_camera(cx, cy, player_x, player_y, &bounds);
            prop_assert!(new_cx >= 0 && new_cx <= max_x, "cx={new_cx} out of [0, {max_x}]");
            prop_assert!(new_cy >= 0 && new_cy <= max_y, "cy={new_cy} out of [0, {max_y}]");
        }

        /// Camera movement per frame is bounded by CAMERA_MAX_STEP.
        #[test]
        fn camera_step_bounded(
            camera in 0i32..1500,
            player in 0i32..2500,
        ) {
            let bounds = LevelBounds {
                width_pixels: 1920,
                height_pixels: 896,
            };
            let max_pos = 1920 - VIEWPORT_WIDTH;
            let cam = camera.min(max_pos);
            let (new_cam, _) = update_camera(cam, 0, player, 0, &bounds);
            let delta = (new_cam - cam).abs();
            prop_assert!(
                delta <= CAMERA_MAX_STEP,
                "delta {delta} > CAMERA_MAX_STEP {CAMERA_MAX_STEP}"
            );
        }
    }

    // -- Air horizontal velocity properties --

    proptest! {
        /// Air velocity from within-range input is bounded by AIR_HARD_CAP + turn accel.
        #[test]
        fn air_velocity_bounded(
            air_idx in 0..ALL_AIR_TABLES.len(),
            vel in -0x4_0000i32..=0x4_0000,
            input in horizontal_input_strategy(),
            facing in facing_strategy(),
        ) {
            let (_, table) = ALL_AIR_TABLES[air_idx];
            // Clamp to hard cap (game never exceeds this in practice)
            let clamped = vel.clamp(-AIR_HARD_CAP, AIR_HARD_CAP);
            let (result, _) = update_air_horizontal_velocity(clamped, facing, input, &table);
            // Turn accel can overshoot when crossing zero
            let bound = AIR_HARD_CAP + table.air_turn_accel;
            prop_assert!(
                result.abs() <= bound,
                "|result| ({}) > bound ({bound})",
                result.abs()
            );
        }

        /// Air deceleration (no input) moves velocity towards zero.
        #[test]
        fn air_decel_moves_toward_zero(
            air_idx in 0..ALL_AIR_TABLES.len(),
            vel in -0x8_0000i32..=0x8_0000,
        ) {
            let (_, table) = ALL_AIR_TABLES[air_idx];
            let input = HorizontalInput { left: false, right: false, run: false };
            let (result, _) = update_air_horizontal_velocity(vel, Direction::Right, input, &table);
            prop_assert!(
                result.abs() <= vel.abs(),
                "|result| ({}) > |vel| ({})",
                result.abs(),
                vel.abs()
            );
        }
    }

    // -- Boundary tests (killing < vs <= / > vs >= mutants) --

    /// ASM loc_B0FE/loc_B10E (kid.asm:13972/13981): no-input jump-air
    /// decel treats vel=0 as neither positive nor negative, so it stays zero.
    #[test]
    fn jump_horizontal_no_input_at_zero_stays_zero() {
        let no_input = HorizontalInput {
            left: false,
            right: false,
            run: false,
        };
        let profile = JumpAirProfile {
            accel: 0x400,
            max_speed: 0x2_0000,
        };
        let (vel, _) = update_jump_horizontal_velocity(0, Direction::Right, no_input, profile);
        assert_eq!(vel, 0, "vel=0 with no input must stay 0");
    }

    /// ASM loc_B0FE (kid.asm:13972): positive no-input jump-air velocity
    /// subtracts `$400` and snaps to zero if the subtraction crosses sign.
    #[test]
    fn jump_horizontal_no_input_positive_one_decelerates() {
        let no_input = HorizontalInput {
            left: false,
            right: false,
            run: false,
        };
        let profile = JumpAirProfile {
            accel: 0x400,
            max_speed: 0x2_0000,
        };
        let (vel, _) = update_jump_horizontal_velocity(1, Direction::Right, no_input, profile);
        // (1 - 0x400).max(0) = 0
        assert_eq!(vel, 0, "vel=1 with no input snaps to 0");
    }

    /// ASM loc_B10E (kid.asm:13981): negative no-input jump-air velocity
    /// adds `$400` and snaps to zero if the addition crosses sign.
    #[test]
    fn jump_horizontal_no_input_negative_one_decelerates() {
        let no_input = HorizontalInput {
            left: false,
            right: false,
            run: false,
        };
        let profile = JumpAirProfile {
            accel: 0x400,
            max_speed: 0x2_0000,
        };
        let (vel, _) = update_jump_horizontal_velocity(-1, Direction::Right, no_input, profile);
        // (-1 + 0x400).min(0) = 0
        assert_eq!(vel, 0, "vel=-1 with no input snaps to 0");
    }

    /// Airborne holding right at exactly max_air_speed: already at cap, no accel.
    /// ASM: `cmp.l d1,d0; blt.s` — vel < max branches to accel; vel == max
    /// falls through to decel-towards-max (which is a no-op at exact max).
    #[test]
    fn air_holding_right_at_exact_max_does_not_accelerate() {
        let table = AIR_NORMAL;
        let result = apply_air_holding_right(table.max_air_speed, &table);
        // At max: (max - decel).max(max) = max (decel towards max is no-op)
        assert_eq!(result, table.max_air_speed);
    }

    /// Airborne holding left at exactly max_air_speed (negated): no accel.
    #[test]
    fn air_holding_left_at_exact_max_does_not_accelerate() {
        let table = AIR_NORMAL;
        let result = apply_air_holding_left(-table.max_air_speed, &table);
        assert_eq!(result, -table.max_air_speed);
    }

    /// Air decelerate at vel=0 stays at zero.
    #[test]
    fn air_decelerate_at_zero_stays_zero() {
        assert_eq!(air_decelerate(0, &AIR_NORMAL), 0);
    }

    /// Air decelerate from exactly +decel snaps to 0 (next = 0, not < 0).
    #[test]
    fn air_decelerate_from_exact_decel_positive_snaps_to_zero() {
        let table = AIR_NORMAL;
        let result = air_decelerate(table.air_no_input_decel, &table);
        // decel - decel = 0, which is >= 0, so returns 0
        assert_eq!(result, 0);
    }

    /// Air decelerate from exactly -decel snaps to 0.
    #[test]
    fn air_decelerate_from_exact_decel_negative_snaps_to_zero() {
        let table = AIR_NORMAL;
        let result = air_decelerate(-table.air_no_input_decel, &table);
        // -decel + decel = 0, which is <= 0, so returns 0
        assert_eq!(result, 0);
    }

    /// Air decelerate from +1: next < 0 snaps to 0.
    #[test]
    fn air_decelerate_positive_one_snaps_to_zero() {
        let table = AIR_NORMAL;
        let result = air_decelerate(1, &table);
        // 1 - decel < 0, snap to 0
        assert_eq!(result, 0);
    }

    /// Air decelerate from -1: next > 0 snaps to 0.
    #[test]
    fn air_decelerate_negative_one_snaps_to_zero() {
        let table = AIR_NORMAL;
        let result = air_decelerate(-1, &table);
        // -1 + decel > 0, snap to 0
        assert_eq!(result, 0);
    }

    /// Ground decelerate at vel=0 stays at zero.
    #[test]
    fn ground_decelerate_at_zero_stays_zero() {
        assert_eq!(decelerate(0, &NORMAL_KID), 0);
    }

    /// Ground decelerate from exactly +walk_run_decel snaps to 0.
    #[test]
    fn ground_decelerate_from_exact_decel_positive_snaps_to_zero() {
        let result = decelerate(NORMAL_KID.walk_run_decel, &NORMAL_KID);
        assert_eq!(result, 0);
    }

    /// Ground decelerate from exactly -walk_run_decel snaps to 0.
    #[test]
    fn ground_decelerate_from_exact_decel_negative_snaps_to_zero() {
        let result = decelerate(-NORMAL_KID.walk_run_decel, &NORMAL_KID);
        assert_eq!(result, 0);
    }

    /// Ground decelerate from +1 snaps to 0.
    #[test]
    fn ground_decelerate_positive_one_snaps_to_zero() {
        assert_eq!(decelerate(1, &NORMAL_KID), 0);
    }

    /// Ground decelerate from -1 snaps to 0.
    #[test]
    fn ground_decelerate_negative_one_snaps_to_zero() {
        assert_eq!(decelerate(-1, &NORMAL_KID), 0);
    }

    // -- Left-direction ground movement boundary tests --

    /// Holding left+run at vel=0: vel=0 is "not moving left" (ASM `bpl`),
    /// so it takes the strong_brake path, returning 0 - strong_brake.
    #[test]
    fn holding_left_run_at_zero_uses_strong_brake() {
        let result = apply_holding_left(0, true, &NORMAL_KID);
        assert_eq!(result, -NORMAL_KID.strong_brake);
    }

    /// Holding left+run at vel=-1: vel<0 so it enters the run-accel path.
    #[test]
    fn holding_left_run_at_neg_one_uses_run_accel() {
        let result = apply_holding_left(-1, true, &NORMAL_KID);
        // abs(-1) + run_accel = 1 + 0x1C00 = 0x1C01, < max_run, so -0x1C01
        assert_eq!(result, -(1 + NORMAL_KID.run_accel));
    }

    /// Holding left+walk at vel=0: vel=0 is "not moving left" (ASM `bpl`),
    /// so it takes the strong_brake path, returning 0 - strong_brake.
    #[test]
    fn holding_left_walk_at_zero_uses_strong_brake() {
        let result = apply_holding_left(0, false, &NORMAL_KID);
        assert_eq!(result, -NORMAL_KID.strong_brake);
    }

    /// Holding left+walk at vel=-1: vel<0, abs < max_walk, so walk_accel.
    #[test]
    fn holding_left_walk_at_neg_one_uses_walk_accel() {
        let result = apply_holding_left(-1, false, &NORMAL_KID);
        // abs(-1) = 1, < max_walk ($20000), accel: -1 - walk_accel = -(1+0x1000)
        assert_eq!(result, -(1 + NORMAL_KID.walk_accel));
    }

    /// Holding left+walk at exactly -max_run_speed: caps at max_run.
    /// ASM: abs_vel > max_run tests as false (equal), so falls to
    /// the abs_vel > max_walk branch.
    #[test]
    fn holding_left_walk_at_max_run_uses_walk_run_decel() {
        let vel = -(NORMAL_KID.max_run_speed);
        let result = apply_holding_left(vel, false, &NORMAL_KID);
        // abs = max_run, NOT > max_run, so doesn't cap to -max_run.
        // abs > max_walk? max_run ($38000) > max_walk ($20000) = true
        // So: vel + walk_run_decel = -max_run + 0x2800
        assert_eq!(result, vel + NORMAL_KID.walk_run_decel);
    }

    /// Holding left+walk at exactly -max_walk_speed: at walk cap, no accel.
    /// ASM: abs_vel > max_walk is false (equal), so enters walk_accel path
    /// but caps at max_walk.
    #[test]
    fn holding_left_walk_at_max_walk_stays_at_cap() {
        let vel = -(NORMAL_KID.max_walk_speed);
        let result = apply_holding_left(vel, false, &NORMAL_KID);
        // abs = max_walk, NOT > max_walk, so enters walk_accel:
        // next = vel - walk_accel, |next| = max_walk + walk_accel > max_walk → cap
        assert_eq!(result, -(NORMAL_KID.max_walk_speed));
    }

    // -- Camera boundary tests --

    /// Camera at position 4, target == camera: not < camera, so standard path.
    #[test]
    fn camera_at_4_with_equal_target_uses_standard_path() {
        // camera=4, player such that target=4: player = 4 + CAMERA_PLAYER_OFFSET
        let player = 4 + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(4, player, 1000);
        // target=4, delta=(4-4+4)>>3 = 0, so camera stays at 4
        assert_eq!(result, 4);
    }

    /// Camera at max_pos-4, target == camera: not > camera, so standard path.
    #[test]
    fn camera_at_max_minus_4_with_equal_target_uses_standard_path() {
        let max_pos = 1000;
        let camera = max_pos - 4;
        let player = camera + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(camera, player, max_pos);
        // target = min(player - offset, max_pos) = min(camera, max_pos) = camera
        // delta = (camera - camera + 4) >> 3 = 0
        assert_eq!(result, camera);
    }

    /// Camera at 4, target < camera by 1: triggers the near-edge nudge of -1.
    #[test]
    fn camera_at_4_with_lower_target_nudges_left() {
        // target = 3 → player = 3 + CAMERA_PLAYER_OFFSET
        let player = 3 + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(4, player, 1000);
        // camera<=4 && target(3) < camera(4) → nudge -1 → 3
        assert_eq!(result, 3);
    }

    /// Camera at max_pos-4, target > camera by 1: triggers the near-edge nudge of +1.
    #[test]
    fn camera_at_max_minus_4_with_higher_target_nudges_right() {
        let max_pos = 1000;
        let camera = max_pos - 4;
        // target = camera + 1 = max_pos - 3, player = target + offset
        // But target is clamped to max_pos, so we need target = camera + 1 ≤ max_pos
        // camera + 1 = max_pos - 3, which is < max_pos, so ok
        let player = (camera + 1) + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(camera, player, max_pos);
        // camera >= max_pos-4 && target > camera → nudge +1
        assert_eq!(result, camera + 1);
    }

    // --- loc_7266: Y-axis target upper clamp ---

    /// ASM loc_7266 (kid.asm:7372): target_y clamped to Level_height - $E0.
    /// Same pattern as X-axis loc_720E but for Y. Verify the Y max_pos
    /// computation uses VIEWPORT_HEIGHT ($E0 = 224).
    #[test]
    fn loc_7266_y_target_clamped_to_max_pos() {
        // Level height 512: max camera Y = 512 - 224 = 288
        let level_height = 512;
        let max_y = level_height - VIEWPORT_HEIGHT;
        assert_eq!(max_y, 288);
        // Kid far below level bottom: target would exceed max_y → clamped
        let player_y = max_y + CAMERA_PLAYER_OFFSET + 100; // well past max
        let result = update_camera_axis(0, player_y, max_y);
        // Camera starts at 0, target clamped to 288, delta = (288+4)>>3 = 36 → clamped to 8
        assert_eq!(result, 8);
    }

    /// loc_7266 fallthrough: target_y < max_pos is kept unchanged.
    #[test]
    fn loc_7266_y_target_below_max_passes_through() {
        let max_y = 288;
        // Kid at y=200: target = 200 - 160 = 40, well below max_y
        let player_y = 200;
        let result = update_camera_axis(40, player_y, max_y);
        // target == camera → delta = (0+4)>>3 = 0, no movement
        assert_eq!(result, 40);
    }

    // --- loc_72A8: Y-axis near-top-edge nudge ---

    /// ASM loc_72A8 (kid.asm:7401): camera near top of level, target below
    /// camera → nudge down at -1.
    /// ```asm
    /// loc_72A8:
    ///   move.w  (Camera_Y_pos).w,d4  ; d4 = camera_y
    ///   cmpi.w  #4,d4                ; camera_y > 4?
    ///   bgt.s   loc_72BA             ; yes → skip
    ///   cmp.w   d4,d0                ; target >= camera?
    ///   bge.s   loc_72BA             ; yes → skip
    ///   moveq   #-1,d3              ; velocity = -1
    ///   bra.s   loc_72D2
    /// ```
    #[test]
    fn loc_72a8_near_top_edge_nudges_up() {
        // camera_y=3 (<=4), target_y=0 (< camera) → velocity = -1
        let result = update_camera_axis(3, 0, 288);
        assert_eq!(result, 2, "ASM loc_72A8: near top edge → nudge -1");
    }

    /// loc_72A8: camera_y=4 is still within the ≤4 threshold.
    #[test]
    fn loc_72a8_boundary_at_exactly_4() {
        // camera_y=4 (<=4), target_y=0 (< camera) → velocity = -1
        let result = update_camera_axis(4, 0, 288);
        assert_eq!(result, 3, "ASM loc_72A8: camera_y==4 still triggers nudge");
    }

    /// loc_72A8: camera_y=5 exceeds threshold → falls through to loc_72C8.
    #[test]
    fn loc_72a8_camera_at_5_falls_through() {
        // camera_y=5 (>4), target_y=0 (< camera) → standard path
        // delta = (0 - 5 + 4) >> 3 = -1 >> 3 = -1 (arithmetic shift of negative)
        let result = update_camera_axis(5, 0, 288);
        assert_eq!(result, 4, "ASM loc_72A8: camera_y>4 → standard delta path");
    }

    /// loc_72A8: camera_y<=4 but target >= camera → no nudge, falls through.
    #[test]
    fn loc_72a8_target_ge_camera_no_nudge() {
        // camera_y=2 (<=4), target_y=2 (== camera) → standard path
        let player_y = 2 + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(2, player_y, 288);
        // delta = (2 - 2 + 4) >> 3 = 0
        assert_eq!(result, 2, "ASM loc_72A8: target==camera → no nudge");
    }

    // --- loc_72BA: Y-axis near-bottom-edge nudge ---

    /// ASM loc_72BA (kid.asm:7411): camera near bottom of level, target above
    /// camera → nudge up at +1.
    /// ```asm
    /// loc_72BA:
    ///   subq.w  #4,d1               ; d1 = max_pos - 4
    ///   cmp.w   d1,d4               ; camera_y < max_pos-4?
    ///   blt.s   loc_72C8            ; yes → skip
    ///   cmp.w   d4,d0               ; target <= camera?
    ///   ble.s   loc_72C8            ; yes → skip
    ///   moveq   #1,d3              ; velocity = +1
    ///   bra.s   loc_72D2
    /// ```
    #[test]
    fn loc_72ba_near_bottom_edge_nudges_down() {
        let max_y = 288;
        // camera_y = max_y - 3 = 285 (>= max_y-4=284), target = max_y (> camera)
        let camera = max_y - 3;
        let player_y = max_y + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(camera, player_y, max_y);
        assert_eq!(
            result,
            camera + 1,
            "ASM loc_72BA: near bottom edge → nudge +1"
        );
    }

    /// loc_72BA: camera_y exactly at max_pos-4 threshold.
    #[test]
    fn loc_72ba_boundary_at_max_minus_4() {
        let max_y = 288;
        let camera = max_y - 4; // 284, >= max_y-4=284
        let player_y = max_y + CAMERA_PLAYER_OFFSET; // target = max_y > camera
        let result = update_camera_axis(camera, player_y, max_y);
        assert_eq!(
            result,
            camera + 1,
            "ASM loc_72BA: camera==max-4 still triggers"
        );
    }

    /// loc_72BA: camera_y below threshold → falls through to loc_72C8.
    #[test]
    fn loc_72ba_camera_below_threshold_falls_through() {
        let max_y = 288;
        let camera = max_y - 5; // 283, < max_y-4=284
        let player_y = max_y + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(camera, player_y, max_y);
        // target = max_y = 288, delta = (288 - 283 + 4) >> 3 = 9 >> 3 = 1
        assert_eq!(
            result,
            camera + 1,
            "ASM loc_72BA: camera<max-4 → standard delta"
        );
    }

    /// loc_72BA: target <= camera → no nudge even when near bottom.
    #[test]
    fn loc_72ba_target_le_camera_no_nudge() {
        let max_y = 288;
        let camera = max_y - 3; // near bottom
        let player_y = camera + CAMERA_PLAYER_OFFSET; // target == camera
        let result = update_camera_axis(camera, player_y, max_y);
        // delta = (camera - camera + 4) >> 3 = 0
        assert_eq!(result, camera, "ASM loc_72BA: target==camera → no nudge");
    }

    // --- loc_72C8: Y-axis normal smooth scroll ---

    /// ASM loc_72C8 (kid.asm:7421): normal case, compute velocity as
    /// `(target - Camera_Y_pos + 4) >> 3`.
    /// ```asm
    /// loc_72C8:
    ///   sub.w   (Camera_Y_pos).w,d0  ; d0 = target - camera_y
    ///   addq.w  #4,d0                ; d0 += 4
    ///   asr.w   #3,d0                ; d0 >>= 3 (arithmetic)
    ///   move.w  d0,d3                ; d3 = velocity
    /// ```
    #[test]
    fn loc_72c8_smooth_scroll_positive_delta() {
        let max_y = 288;
        // camera_y = 50, player such that target = 90 → delta = (90-50+4)>>3 = 5
        let player_y = 90 + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(50, player_y, max_y);
        assert_eq!(
            result, 55,
            "ASM loc_72C8: positive delta = (90-50+4)>>3 = 5"
        );
    }

    /// loc_72C8: negative delta when target is below camera.
    #[test]
    fn loc_72c8_smooth_scroll_negative_delta() {
        let max_y = 288;
        // camera_y = 100, player such that target = 60
        // delta = (60-100+4)>>3 = -36>>3 = -5
        let player_y = 60 + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(100, player_y, max_y);
        assert_eq!(
            result, 95,
            "ASM loc_72C8: negative delta = (60-100+4)>>3 = -5"
        );
    }

    /// loc_72C8 → loc_72D2: large delta clamped to +8.
    #[test]
    fn loc_72c8_large_positive_clamped_to_8() {
        let max_y = 288;
        // camera_y = 10, target = 288 → delta = (288-10+4)>>3 = 35, clamped to 8
        let player_y = max_y + CAMERA_PLAYER_OFFSET;
        let result = update_camera_axis(10, player_y, max_y);
        assert_eq!(result, 18, "ASM loc_72D2: positive delta clamped to 8");
    }

    /// loc_72C8 → loc_72DA: large negative delta clamped to -8.
    #[test]
    fn loc_72c8_large_negative_clamped_to_minus_8() {
        let max_y = 288;
        // camera_y = 200, target = 0 → delta = (0-200+4)>>3 = -24, clamped to -8
        let result = update_camera_axis(200, 0, max_y);
        assert_eq!(result, 192, "ASM loc_72DA: negative delta clamped to -8");
    }

    // --- loc_726C: ceiling look-offset pull-back ---

    /// ASM loc_726C (kid.asm:7377): when target_y == 0 and kid_y < last_kid_y,
    /// return -2 (decrement look-offset by 2).
    #[test]
    fn loc_726c_ceiling_and_rising_returns_minus_2() {
        // target_y clamped to 0, Kid moved upward (100 → 80)
        let adj = camera_y_ceiling_look_adjust_loc_726c(0, 80, 100);
        assert_eq!(adj, -2);
    }

    /// loc_726C: target_y != 0 → no adjustment even if kid is rising.
    #[test]
    fn loc_726c_nonzero_target_returns_zero() {
        let adj = camera_y_ceiling_look_adjust_loc_726c(10, 80, 100);
        assert_eq!(adj, 0);
    }

    /// loc_726C: target_y == 0 but kid_y >= last_kid_y → no adjustment.
    #[test]
    fn loc_726c_not_rising_returns_zero() {
        // Kid stationary or falling
        let adj = camera_y_ceiling_look_adjust_loc_726c(0, 100, 100);
        assert_eq!(adj, 0);
        let adj = camera_y_ceiling_look_adjust_loc_726c(0, 110, 100);
        assert_eq!(adj, 0);
    }

    // --- loc_727E: viewport-top look-offset pull-back ---

    /// ASM loc_727E (kid.asm:7385): when kid_y - camera_y < $20 (32),
    /// return -1 (decrement look-offset by 1).
    #[test]
    fn loc_727e_near_viewport_top_returns_minus_1() {
        // Kid at y=50, camera at y=40: delta = 10 < 32 → -1
        let adj = camera_y_viewport_top_look_adjust_loc_727e(50, 40);
        assert_eq!(adj, -1);
    }

    /// loc_727E: kid_y - camera_y == 31 → still triggers (< 32).
    #[test]
    fn loc_727e_delta_31_returns_minus_1() {
        let adj = camera_y_viewport_top_look_adjust_loc_727e(71, 40);
        assert_eq!(adj, -1);
    }

    /// loc_727E: kid_y - camera_y == 32 → no adjustment (>= $20).
    #[test]
    fn loc_727e_delta_32_returns_zero() {
        let adj = camera_y_viewport_top_look_adjust_loc_727e(72, 40);
        assert_eq!(adj, 0);
    }

    /// loc_727E: kid_y well below camera → no adjustment.
    #[test]
    fn loc_727e_kid_below_camera_returns_zero() {
        let adj = camera_y_viewport_top_look_adjust_loc_727e(200, 40);
        assert_eq!(adj, 0);
    }

    // --- loc_72E2: camera Y velocity apply + BG parallax scroll ---

    /// ASM loc_72E2 (kid.asm:7437): `add.w d3,(Camera_Y_pos).w` —
    /// apply clamped Y velocity.  This is the Y-axis counterpart of
    /// loc_724E (`add.w d2,(Camera_X_pos).w`).  Verified via
    /// `update_camera_axis` which returns `camera + velocity`.
    #[test]
    fn loc_72e2_applies_y_velocity_positive() {
        // camera_y=50, target=90 → delta = (90-50+4)>>3 = 5 → new = 55
        let result = update_camera_axis(50, 90 + CAMERA_PLAYER_OFFSET, 288);
        assert_eq!(result, 55, "ASM loc_72E2: camera_y += positive velocity");
    }

    #[test]
    fn loc_72e2_applies_y_velocity_negative() {
        // camera_y=100, target=60 → delta = (60-100+4)>>3 = -5 → new = 95
        let result = update_camera_axis(100, 60 + CAMERA_PLAYER_OFFSET, 288);
        assert_eq!(result, 95, "ASM loc_72E2: camera_y += negative velocity");
    }

    /// ASM loc_72E2 (kid.asm:7439–7444): BG parallax = camera / 4.
    /// ```asm
    ///   move.l  (Camera_X_pos).w,d0
    ///   lsr.l   #2,d0
    ///   move.l  d0,(Camera_BG_X_pos).w
    ///   move.l  (Camera_Y_pos).w,d0
    ///   lsr.l   #2,d0
    ///   move.l  d0,(Camera_BG_Y_pos).w
    /// ```
    #[test]
    fn loc_72e2_bg_parallax_divides_by_4() {
        let (bg_x, bg_y) = camera_bg_scroll_loc_72e2(0x0100_0000, 0x0080_0000);
        assert_eq!(bg_x, 0x0040_0000, "ASM loc_72E2: BG_X = Camera_X >> 2");
        assert_eq!(bg_y, 0x0020_0000, "ASM loc_72E2: BG_Y = Camera_Y >> 2");
    }

    #[test]
    fn loc_72e2_bg_parallax_zero() {
        let (bg_x, bg_y) = camera_bg_scroll_loc_72e2(0, 0);
        assert_eq!(bg_x, 0, "ASM loc_72E2: zero camera → zero BG");
        assert_eq!(bg_y, 0);
    }
}
