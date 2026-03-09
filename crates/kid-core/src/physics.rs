/// Real input-driven physics translated from kid.asm.
///
/// All velocities and positions use the same fixed-point 16.16 format as the
/// original 68k code: the upper 16 bits are the pixel value, the lower 16 are
/// the fractional part.

// ---------------------------------------------------------------------------
// Direction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        (apply_holding_right(x_vel, input.run, table), Direction::Right)
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
        assert_eq!(NORMAL_MICROMAX.max_walk_speed, NORMAL_KID.max_walk_speed / 2);
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
        HorizontalInput { left: false, right: true, run: false }
    }

    fn right_run() -> HorizontalInput {
        HorizontalInput { left: false, right: true, run: true }
    }

    fn left_walk() -> HorizontalInput {
        HorizontalInput { left: true, right: false, run: false }
    }

    fn no_input() -> HorizontalInput {
        HorizontalInput { left: false, right: false, run: false }
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
            let (v, _) = update_horizontal_velocity(vel, Direction::Right, right_walk(), &NORMAL_KID);
            vel = v;
        }
        assert_eq!(vel, NORMAL_KID.max_walk_speed);
    }

    #[test]
    fn run_right_caps_at_max_run_speed() {
        let mut vel = 0i32;
        for _ in 0..1000 {
            let (v, _) = update_horizontal_velocity(vel, Direction::Right, right_run(), &NORMAL_KID);
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
        let (result, _) = update_horizontal_velocity(vel, Direction::Right, no_input(), &NORMAL_KID);
        assert_eq!(result, 0);
    }

    #[test]
    fn brake_direction_change_right_to_left() {
        // Moving right, now holding left → should brake with strong_brake
        let vel = NORMAL_KID.max_walk_speed;
        let (result, dir) = update_horizontal_velocity(vel, Direction::Right, left_walk(), &NORMAL_KID);
        assert_eq!(result, vel - NORMAL_KID.strong_brake);
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn brake_direction_change_left_to_right() {
        // Moving left, now holding right → should brake with turn_brake
        let vel = -NORMAL_KID.max_walk_speed;
        let (result, dir) = update_horizontal_velocity(vel, Direction::Right, right_walk(), &NORMAL_KID);
        assert_eq!(result, vel + NORMAL_KID.turn_brake);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn brake_asymmetry_turn_vs_strong() {
        // The key disassembly asymmetry: right-moving-left uses turn_brake ($0C),
        // left-moving-right uses strong_brake ($18)
        let vel_r = -NORMAL_KID.max_walk_speed;
        let (result_r, _) = update_horizontal_velocity(vel_r, Direction::Right, right_walk(), &NORMAL_KID);
        let vel_l = NORMAL_KID.max_walk_speed;
        let (result_l, _) = update_horizontal_velocity(vel_l, Direction::Left, left_walk(), &NORMAL_KID);

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
        let (result, _) = update_horizontal_velocity(vel, Direction::Right, right_walk(), &NORMAL_KID);
        assert_eq!(result, vel - NORMAL_KID.walk_run_decel);
        assert!(result > NORMAL_KID.max_walk_speed);
    }
}
