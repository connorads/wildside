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
/// Returns `(new_x_vel, new_facing)`.
pub fn update_air_horizontal_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    table: &AirSpeedTable,
) -> (i32, Direction) {
    if input.right {
        (apply_air_holding_right(x_vel, table), Direction::Right)
    } else if input.left {
        (apply_air_holding_left(x_vel, table), Direction::Left)
    } else {
        (air_decelerate(x_vel, table), facing)
    }
}

/// Player airborne control while jumping (sub_B084).
pub fn update_jump_horizontal_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    profile: JumpAirProfile,
) -> (i32, Direction) {
    let accel = if input.run {
        profile.accel + (profile.accel >> 1)
    } else {
        profile.accel
    };

    let new_vel = if input.left {
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
        let next = x_vel + accel;
        if next < 0 {
            next
        } else if profile.max_speed >= next {
            next
        } else {
            x_vel - 0x100
        }
    } else if x_vel > 0 {
        (x_vel - 0x400).max(0)
    } else if x_vel < 0 {
        (x_vel + 0x400).min(0)
    } else {
        0
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
        // Moving right — cap at hard limit first
        let vel = vel.min(AIR_HARD_CAP);
        if vel < table.max_air_speed {
            // Below max air speed: accelerate
            (vel + table.air_accel).min(table.max_air_speed)
        } else {
            // At or above max air speed: decelerate towards it
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
        // Moving left — cap at hard limit first
        let abs_vel = vel.saturating_neg().min(AIR_HARD_CAP);
        if abs_vel < table.max_air_speed {
            // Below max air speed: accelerate
            let next_abs = (abs_vel + table.air_accel).min(table.max_air_speed);
            -next_abs
        } else {
            // At or above max air speed: decelerate towards it
            let next_abs = (abs_vel - table.air_above_max_decel).max(table.max_air_speed);
            -next_abs
        }
    }
}

/// Airborne deceleration towards zero when no direction is held (loc_85AC).
/// Snaps to zero on sign change.
fn air_decelerate(vel: i32, table: &AirSpeedTable) -> i32 {
    if vel > 0 {
        let next = vel - table.air_no_input_decel;
        if next < 0 { 0 } else { next }
    } else if vel < 0 {
        let next = vel + table.air_no_input_decel;
        if next > 0 { 0 } else { next }
    } else {
        0
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
// Gravity constants — kid.asm:14060-14088
// ---------------------------------------------------------------------------

/// Gravity applied per frame when descending or when ascending without B held.
pub const GRAVITY_NORMAL: i32 = 0x4000;

/// Gravity applied per frame when ascending with B (jump) held.
pub const GRAVITY_JUMP_HELD: i32 = 0x2000;

/// Terminal downward velocity.
pub const MAX_FALL_SPEED: i32 = 0x8_0000;

/// Terminal upward velocity (negative = upward).
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
/// Direct translation of the branching at loc_9B2E..loc_9C06. Velocity is
/// signed i32 in fixed-point 16.16: positive = rightward.
///
/// Returns `(new_x_vel, new_facing)`.
pub fn update_horizontal_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    table: &SpeedTable,
) -> (i32, Direction) {
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

/// Facing right, holding right — loc_9B2E..loc_9B7E.
fn apply_holding_right(vel: i32, run: bool, table: &SpeedTable) -> i32 {
    if run {
        // loc_9B62: running right
        if vel >= 0 {
            (vel + table.run_accel).min(table.max_run_speed)
        } else {
            // Moving left while holding right+run: strong_brake ($18)
            vel + table.strong_brake
        }
    } else if vel >= 0 {
        // Walking right, moving right
        if vel > table.max_run_speed {
            table.max_run_speed
        } else if vel > table.max_walk_speed {
            // Above walk cap: decel with walk_run_decel ($08)
            vel - table.walk_run_decel
        } else {
            // Below walk cap: accelerate
            (vel + table.walk_accel).min(table.max_walk_speed)
        }
    } else {
        // Walking right, but moving left: turn_brake ($0C)
        vel + table.turn_brake
    }
}

/// Facing left, holding left — loc_9B90..loc_9BEC. Mirror of right-facing
/// logic, but uses strong_brake ($18) for opposite-direction walk brake
/// instead of turn_brake ($0C).
fn apply_holding_left(vel: i32, run: bool, table: &SpeedTable) -> i32 {
    if run {
        // loc_9BCE: running left
        if vel <= 0 {
            let abs_vel = vel.saturating_neg();
            let next_abs = (abs_vel + table.run_accel).min(table.max_run_speed);
            -next_abs
        } else {
            // Moving right while holding left+run: strong_brake ($18)
            vel - table.strong_brake
        }
    } else if vel <= 0 {
        // Walking left, moving left
        let abs_vel = vel.saturating_neg();
        if abs_vel > table.max_run_speed {
            -table.max_run_speed
        } else if abs_vel > table.max_walk_speed {
            // Above walk cap: decel with walk_run_decel ($08)
            vel + table.walk_run_decel
        } else {
            // Below walk cap: accelerate
            let next = vel - table.walk_accel;
            if next.saturating_neg() > table.max_walk_speed {
                -table.max_walk_speed
            } else {
                next
            }
        }
    } else {
        // Walking left, but moving right: strong_brake ($18)
        vel - table.strong_brake
    }
}

/// Decelerate towards zero when no direction is held (loc_9BEE). Snap to zero
/// on sign change.
fn decelerate(vel: i32, table: &SpeedTable) -> i32 {
    if vel > 0 {
        let next = vel - table.walk_run_decel;
        if next < 0 { 0 } else { next }
    } else if vel < 0 {
        let next = vel + table.walk_run_decel;
        if next > 0 { 0 } else { next }
    } else {
        0
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

    // x_vel *= 3/4: asr #1, copy, asr #1, add
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
/// `ascending_and_b_held`: true when y_vel < 0 AND the jump button is held,
/// giving the variable-height jump feel.
pub fn apply_gravity(y_vel: i32, ascending_and_b_held: bool) -> i32 {
    let gravity = if ascending_and_b_held {
        GRAVITY_JUMP_HELD
    } else {
        GRAVITY_NORMAL
    };
    let next = y_vel + gravity;
    next.clamp(MAX_RISE_SPEED, MAX_FALL_SPEED)
}

// ---------------------------------------------------------------------------
// Camera follow — kid.asm:7306-7521 (sub_71E4)
// ---------------------------------------------------------------------------

/// Update camera position to follow the player. All coordinates are in pixels
/// (the 68k camera logic operates on the upper 16 bits of positions).
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
pub(crate) fn update_camera_axis(camera: i32, player: i32, max_pos: i32) -> i32 {
    let target = (player - CAMERA_PLAYER_OFFSET).max(0).min(max_pos);

    let velocity = if camera <= 4 && target < camera {
        // Near left/top edge, target is further left/up: nudge at -1
        -1
    } else if camera >= max_pos - 4 && target > camera {
        // Near right/bottom edge, target is further right/down: nudge at +1
        1
    } else {
        let delta = (target - camera + 4) >> 3;
        delta.clamp(-CAMERA_MAX_STEP, CAMERA_MAX_STEP)
    };

    (camera + velocity).max(0).min(max_pos)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
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
}

// ---------------------------------------------------------------------------
// Property-based tests (proptest)
// ---------------------------------------------------------------------------

#[cfg(test)]
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
        /// Jump always reduces x velocity magnitude (3/4 factor).
        #[test]
        fn jump_reduces_x_velocity(
            x_vel in -0x8_0000i32..=0x8_0000,
            height_idx in 0usize..10,
        ) {
            let (new_x, _) = initiate_jump(x_vel, JUMP_HEIGHTS[height_idx]);
            prop_assert!(
                new_x.abs() <= x_vel.abs(),
                "|new_x| ({}) > |x_vel| ({})",
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
}
