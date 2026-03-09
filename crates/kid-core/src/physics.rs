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
}
