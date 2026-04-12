use super::*;
use crate::constants::SINE_TABLE_WORD_402B0;

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn red_stealth_swing_anchor_sub_868E(player: &PlayerState) -> (u32, u32) {
    // ASM sub_868E (kid.asm:9522): the spawned sword worker copies Kid's
    // position, subtracts 10px from Y, then offsets X by 28px toward the
    // active facing. Native `Direction::Right` mirrors the clear x-direction
    // player pose used by the existing Red Stealth combat path.
    let x_offset_px = if player.facing == Direction::Right {
        RED_STEALTH_REACH_X
    } else {
        -RED_STEALTH_REACH_X
    };
    let x_px = ((player.x_pos >> 16) as i32 + x_offset_px).max(0) as u32;
    let y_px = ((player.y_pos >> 16) as i32 + RED_STEALTH_OFFSET_Y).max(0) as u32;
    (x_px << 16, y_px << 16)
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RedStealthFloorImpactKindSubB000 {
    None,
    Smash,
    Prize,
    Ice,
    BounceOnly,
    MetalBounce,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RedStealthFloorImpactSubB000 {
    pub(crate) kind: RedStealthFloorImpactKindSubB000,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) bounce_y_velocity: i32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn red_stealth_floor_impact_sub_B000(
    current_form: PlayerForm,
    sword_swing_active: bool,
    grid: &terrain::BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
) -> RedStealthFloorImpactSubB000 {
    // ASM sub_B000 (kid.asm:13864): only Red Stealth with a live sword swing
    // samples `sub_B55C` at `(x_pos(a3), y_pos(a3)+1)`. Solid block types
    // `$0..$5` all force `y_vel(a3)=-$38000`; Smash/Prize/Ice and Metal route
    // to their block helpers, while SolidGhost and Teleporter only bounce.
    let none = RedStealthFloorImpactSubB000 {
        kind: RedStealthFloorImpactKindSubB000::None,
        block_col: 0,
        block_row: 0,
        bounce_y_velocity: 0,
    };

    if current_form != PlayerForm::RedStealth || !sword_swing_active {
        return none;
    }

    if terrain::probe_relative_collision_sub_B55C(grid, center_x_px, foot_y_px, 0, 1)
        != terrain::CollisionKind::Solid
    {
        return none;
    }

    let probe = terrain::probe_at(grid, center_x_px, foot_y_px + 1);
    if !probe.cell.has_block {
        return none;
    }

    let kind = match probe.cell.block_type {
        terrain::BlockType::Smash => RedStealthFloorImpactKindSubB000::Smash,
        terrain::BlockType::Prize => RedStealthFloorImpactKindSubB000::Prize,
        terrain::BlockType::Ice => RedStealthFloorImpactKindSubB000::Ice,
        terrain::BlockType::SolidGhost | terrain::BlockType::Teleporter => {
            RedStealthFloorImpactKindSubB000::BounceOnly
        }
        terrain::BlockType::Metal => RedStealthFloorImpactKindSubB000::MetalBounce,
        _ => RedStealthFloorImpactKindSubB000::None,
    };

    if kind == RedStealthFloorImpactKindSubB000::None {
        return none;
    }

    RedStealthFloorImpactSubB000 {
        kind,
        block_col: probe.cell_x,
        block_row: probe.cell_y,
        bounce_y_velocity: -0x0003_8000,
    }
}

fn is_juggernaut_skull_projectile(projectile: &EnemyProjectile) -> bool {
    projectile.friendly
        && projectile.gravity == JUGGERNAUT_SKULL_GRAVITY
        && projectile.homing_accel == 0
        && projectile.speed_cap == 0
        && !projectile.terrain_kill
}

const MOVE_ID_STANDINGSTILL: u16 = 0;
const MOVE_ID_WALKING: u16 = 2;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JuggernautCompanionStateSub975C {
    pub(crate) byte_15_active: bool,
    pub(crate) byte_18_active: bool,
    pub(crate) blocked_latch: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JuggernautCompanionVisualSub975C {
    PreserveCurrent,
    InitAnimation(&'static str),
    SetMapping(&'static str),
    None,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JuggernautCompanionStepSub975C {
    pub(crate) byte_15_active: bool,
    pub(crate) blocked_latch: bool,
    pub(crate) visual: JuggernautCompanionVisualSub975C,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn juggernaut_companion_step_sub_975C(
    state: JuggernautCompanionStateSub975C,
    wall_scan_blocked: bool,
    object_overlap: bool,
    down_held: bool,
    character_movement: u16,
    walking_table_byte_offset: u16,
) -> JuggernautCompanionStepSub975C {
    // ASM sub_975C (kid.asm:11286): byte `$15(a4)` keeps the current support
    // animation locked until byte `$18(a4)` becomes non-zero. Once unlocked,
    // the shared `$FFFFFA6C` latch flips between `stru_9812` and `stru_981C`
    // based on the tile/object/down predicates, otherwise the routine picks a
    // standing or walking mapping for the support sprite.
    if state.byte_15_active && !state.byte_18_active {
        return JuggernautCompanionStepSub975C {
            byte_15_active: state.byte_15_active,
            blocked_latch: state.blocked_latch,
            visual: JuggernautCompanionVisualSub975C::PreserveCurrent,
        };
    }

    let blocked = wall_scan_blocked || object_overlap || down_held;
    let visual = if state.blocked_latch {
        if blocked {
            JuggernautCompanionVisualSub975C::None
        } else {
            JuggernautCompanionVisualSub975C::InitAnimation("stru_9812")
        }
    } else if blocked {
        JuggernautCompanionVisualSub975C::InitAnimation("stru_981C")
    } else if character_movement == MOVE_ID_STANDINGSTILL {
        JuggernautCompanionVisualSub975C::SetMapping("LnkTo_unk_BEDF0-Data_Index")
    } else if character_movement == MOVE_ID_WALKING {
        const WALK_TABLE: [&str; 6] = [
            "LnkTo_unk_BEDF0-Data_Index",
            "LnkTo_unk_BEF76-Data_Index",
            "LnkTo_unk_BF0FC-Data_Index",
            "LnkTo_unk_BF282-Data_Index",
            "LnkTo_unk_BF408-Data_Index",
            "LnkTo_unk_BF58E-Data_Index",
        ];
        let table_index = usize::from(walking_table_byte_offset >> 1);
        WALK_TABLE
            .get(table_index)
            .copied()
            .map(JuggernautCompanionVisualSub975C::SetMapping)
            .unwrap_or(JuggernautCompanionVisualSub975C::None)
    } else {
        JuggernautCompanionVisualSub975C::None
    };

    JuggernautCompanionStepSub975C {
        byte_15_active: false,
        blocked_latch: blocked,
        visual,
    }
}

#[allow(non_snake_case)]
pub(crate) fn juggernaut_skull_spawn_anchor_sub_87B0(player: &PlayerState) -> (u32, u32, i32) {
    // ASM sub_87B0 (kid.asm:9625): the spawned skull starts 8px above Kid and
    // 26px toward the current facing, with `x_vel=±4px/frame`.
    let x_offset_px = if player.facing == Direction::Right {
        JUGGERNAUT_SKULL_OFFSET_X
    } else {
        -JUGGERNAUT_SKULL_OFFSET_X
    };
    let x_vel = if player.facing == Direction::Right {
        JUGGERNAUT_SKULL_X_SPEED
    } else {
        -JUGGERNAUT_SKULL_X_SPEED
    };
    let x_px = ((player.x_pos >> 16) as i32 + x_offset_px).max(0) as u32;
    let y_px = ((player.y_pos >> 16) as i32 + JUGGERNAUT_SKULL_OFFSET_Y).max(0) as u32;
    (x_px << 16, y_px << 16, x_vel)
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SkycutterVisualStateSub98F2 {
    pub(crate) flipboard_counter: u8,
    pub(crate) on_board: bool,
    pub(crate) facing: Direction,
    pub(crate) motion_phase: u8,
    pub(crate) motion_timer: u8,
    pub(crate) motion_script_index: Option<u8>,
    pub(crate) roll_accum: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SkycutterVisualStepSub98F2 {
    pub(crate) state: SkycutterVisualStateSub98F2,
    pub(crate) player_sprite_mapping: Option<&'static str>,
    pub(crate) board_sprite_mapping: Option<&'static str>,
    pub(crate) board_priority_word: u16,
    pub(crate) board_y_offset_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KidAirborneVisualKindSubB270 {
    PreserveCurrent,
    DelegateToSkycutter,
    UseMapping(&'static str),
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidAirborneVisualInputSubB270 {
    pub(crate) helmet_id: u16,
    pub(crate) is_animated: bool,
    pub(crate) on_ground: bool,
    pub(crate) global_lock_active: bool,
    pub(crate) airborne_latch_active: bool,
    pub(crate) cyclone_flying: bool,
    pub(crate) cyclone_y_accel: i16,
    pub(crate) cyclone_frame_accum: u16,
    pub(crate) juggernaut_companion_animation_active: bool,
    pub(crate) maniaxe_throwing_axe: bool,
    pub(crate) y_velocity: i32,
    pub(crate) jump_threshold: i32,
    pub(crate) c_held: bool,
    pub(crate) red_stealth_sword_swing: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidAirborneVisualStepSubB270 {
    pub(crate) visual: KidAirborneVisualKindSubB270,
    pub(crate) next_is_animated: bool,
    pub(crate) next_airborne_latch_active: bool,
    pub(crate) next_cyclone_frame_accum: u16,
    pub(crate) next_maniaxe_throwing_axe: bool,
    pub(crate) next_red_stealth_sword_swing: u8,
    pub(crate) queued_sound: Option<QueuedSoundEffect>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_airborne_visual_sub_B270(
    input: KidAirborneVisualInputSubB270,
) -> KidAirborneVisualStepSubB270 {
    // ASM sub_B270 (kid.asm:14132): early-out for Eyeclops animation, the
    // shared `$FFFFFA66/$FFFFFA67` locks, Cyclone flight, and Juggernaut's
    // helper cloud before the common four-state airborne mapping table.
    if input.helmet_id == PlayerForm::Eyeclops.helmet_id() && input.is_animated {
        return KidAirborneVisualStepSubB270 {
            visual: KidAirborneVisualKindSubB270::PreserveCurrent,
            next_is_animated: input.is_animated,
            next_airborne_latch_active: input.airborne_latch_active,
            next_cyclone_frame_accum: input.cyclone_frame_accum,
            next_maniaxe_throwing_axe: input.maniaxe_throwing_axe,
            next_red_stealth_sword_swing: input.red_stealth_sword_swing,
            queued_sound: None,
        };
    }

    if input.global_lock_active || (input.airborne_latch_active && !input.on_ground) {
        return KidAirborneVisualStepSubB270 {
            visual: KidAirborneVisualKindSubB270::PreserveCurrent,
            next_is_animated: input.is_animated,
            next_airborne_latch_active: input.airborne_latch_active,
            next_cyclone_frame_accum: input.cyclone_frame_accum,
            next_maniaxe_throwing_axe: input.maniaxe_throwing_axe,
            next_red_stealth_sword_swing: input.red_stealth_sword_swing,
            queued_sound: None,
        };
    }

    if input.cyclone_flying {
        let mut phase_word = (0i16)
            .wrapping_sub(input.cyclone_y_accel)
            .wrapping_add(0x2000) as u16;
        phase_word = phase_word.wrapping_shl(2);
        phase_word >>= 8;
        phase_word = phase_word.wrapping_add(input.cyclone_frame_accum);
        if phase_word >= 0x0300 {
            phase_word = phase_word.wrapping_sub(0x0300);
        }
        let mapping = kid_data::cyclone_flight_sprite_mapping_sub_B270(phase_word >> 8)
            .expect("ASM sub_B270 Cyclone phase should stay within 0..=2");
        return KidAirborneVisualStepSubB270 {
            visual: KidAirborneVisualKindSubB270::UseMapping(mapping),
            next_is_animated: input.is_animated,
            next_airborne_latch_active: false,
            next_cyclone_frame_accum: phase_word,
            next_maniaxe_throwing_axe: input.maniaxe_throwing_axe,
            next_red_stealth_sword_swing: input.red_stealth_sword_swing,
            queued_sound: None,
        };
    }

    let mut next_is_animated = input.is_animated;
    if input.helmet_id == PlayerForm::Juggernaut.helmet_id()
        && input.juggernaut_companion_animation_active
    {
        if !input.on_ground {
            return KidAirborneVisualStepSubB270 {
                visual: KidAirborneVisualKindSubB270::PreserveCurrent,
                next_is_animated,
                next_airborne_latch_active: false,
                next_cyclone_frame_accum: input.cyclone_frame_accum,
                next_maniaxe_throwing_axe: input.maniaxe_throwing_axe,
                next_red_stealth_sword_swing: input.red_stealth_sword_swing,
                queued_sound: None,
            };
        }
        next_is_animated = false;
    }

    if input.helmet_id == PlayerForm::Skycutter.helmet_id() {
        return KidAirborneVisualStepSubB270 {
            visual: KidAirborneVisualKindSubB270::DelegateToSkycutter,
            next_is_animated,
            next_airborne_latch_active: false,
            next_cyclone_frame_accum: input.cyclone_frame_accum,
            next_maniaxe_throwing_axe: input.maniaxe_throwing_axe,
            next_red_stealth_sword_swing: input.red_stealth_sword_swing,
            queued_sound: None,
        };
    }

    if input.helmet_id == PlayerForm::Maniaxe.helmet_id()
        && input.maniaxe_throwing_axe
        && !input.on_ground
    {
        return KidAirborneVisualStepSubB270 {
            visual: KidAirborneVisualKindSubB270::PreserveCurrent,
            next_is_animated,
            next_airborne_latch_active: false,
            next_cyclone_frame_accum: input.cyclone_frame_accum,
            next_maniaxe_throwing_axe: true,
            next_red_stealth_sword_swing: input.red_stealth_sword_swing,
            queued_sound: None,
        };
    }

    let scaled_y_velocity = input.y_velocity.wrapping_shl(2);
    let state_index = if scaled_y_velocity < 0 {
        if scaled_y_velocity > input.jump_threshold {
            1
        } else {
            0
        }
    } else if scaled_y_velocity > input.jump_threshold {
        3
    } else {
        2
    };

    if input.helmet_id == PlayerForm::RedStealth.helmet_id() && state_index == 3 && input.c_held {
        let next_sword = input.red_stealth_sword_swing.saturating_add(0x10).min(0x7F);
        let mapping = kid_data::red_stealth_downward_slash_sprite_mapping_sub_B270(next_sword)
            .expect("ASM sub_B270 Red Stealth slash should stay within 0..=3");
        return KidAirborneVisualStepSubB270 {
            visual: KidAirborneVisualKindSubB270::UseMapping(mapping),
            next_is_animated,
            next_airborne_latch_active: false,
            next_cyclone_frame_accum: input.cyclone_frame_accum,
            next_maniaxe_throwing_axe: false,
            next_red_stealth_sword_swing: next_sword,
            queued_sound: (next_sword == 0x10).then_some(QueuedSoundEffect::RedStealthAttackDown),
        };
    }

    let mapping = kid_data::airborne_helmet_sprite_mapping_sub_B270(input.helmet_id, state_index)
        .expect("ASM sub_B270 off_B3B8 should cover all live helmet ids");
    KidAirborneVisualStepSubB270 {
        visual: KidAirborneVisualKindSubB270::UseMapping(mapping),
        next_is_animated,
        next_airborne_latch_active: false,
        next_cyclone_frame_accum: input.cyclone_frame_accum,
        next_maniaxe_throwing_axe: false,
        next_red_stealth_sword_swing: 0,
        queued_sound: None,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IronKnightClimbVisualSubA3BC {
    pub(crate) pose_accum: u16,
    pub(crate) sprite_mapping: &'static str,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn skycutter_visual_step_sub_98F2(
    mut state: SkycutterVisualStateSub98F2,
    a_held: bool,
    crawling: bool,
) -> SkycutterVisualStepSub98F2 {
    // ASM sub_98F2 (kid.asm:11454): `$FFFFFA69` runs the transient flipboard
    // deploy/retract pose, toggling `$FFFFFA6A` and `x_direction(a3)` at
    // counter `$11`. With no transient pose active, the routine advances the
    // helper-board animation script `unk_96AC`, rolls `$FFFFFA0C`, and picks
    // the `off_99FE` mapping unless crawling forces `LnkTo_unk_A94AC`.
    const SCRIPT: [(u8, u8); 18] = [
        (0, 6),
        (1, 0x0A),
        (2, 0x1E),
        (3, 0x14),
        (4, 6),
        (5, 0x0A),
        (0, 0x0A),
        (1, 8),
        (2, 0x0C),
        (3, 8),
        (4, 0x0A),
        (5, 0x0C),
        (0, 0x0F),
        (1, 0x0F),
        (2, 0x19),
        (3, 0x0F),
        (4, 0x19),
        (5, 0x0A),
    ];

    if state.flipboard_counter != 0 {
        let next_counter = state
            .flipboard_counter
            .saturating_add(SKYCUTTER_COUNTER_STEP);
        if next_counter >= SKYCUTTER_FLIPBOARD_TIMEOUT {
            state.flipboard_counter = 0;
        } else {
            if next_counter == SKYCUTTER_MOUNT_FRAME {
                state.on_board = !state.on_board;
                state.facing = match state.facing {
                    Direction::Right => Direction::Left,
                    Direction::Left => Direction::Right,
                };
            }
            state.flipboard_counter = next_counter;
            return SkycutterVisualStepSub98F2 {
                player_sprite_mapping: kid_data::skycutter_flipboard_sprite_mapping_sub_98F2(
                    next_counter,
                ),
                board_sprite_mapping: None,
                board_priority_word: 0,
                board_y_offset_px: 0,
                state,
            };
        }
    }

    if state.motion_timer <= 1 {
        let index = usize::from(state.motion_script_index.unwrap_or(u8::MAX));
        let (phase, timer, next_index) = if let Some(&(phase, timer)) = SCRIPT.get(index) {
            (phase, timer, Some((index + 1) as u8))
        } else {
            let (phase, timer) = SCRIPT[0];
            (phase, timer, Some(1))
        };
        state.motion_phase = phase;
        state.motion_timer = timer;
        state.motion_script_index = next_index;
    } else {
        state.motion_timer -= 1;
    }

    state.roll_accum = state
        .roll_accum
        .wrapping_add(if a_held { 0x20 } else { 0x10 });
    let board_priority_word = match state.roll_accum.wrapping_mul(2) {
        0x00..=0x54 => 0x012C,
        0x55..=0xA9 => 0x0130,
        _ => 0x0134,
    };
    let board_mapping = kid_data::skycutter_board_sprite_mapping_sub_98F2(
        u16::from(state.motion_phase.min(5)),
        crawling,
    );

    SkycutterVisualStepSub98F2 {
        player_sprite_mapping: board_mapping,
        board_sprite_mapping: board_mapping,
        board_priority_word,
        board_y_offset_px: if state.on_board { -0x1A } else { 0 },
        state,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn iron_knight_climb_visual_sub_A3BC(
    pose_accum: u16,
    y_velocity: i32,
) -> IronKnightClimbVisualSubA3BC {
    // ASM sub_A3BC (kid.asm:12694): add `(y_vel << 3) >> 16` into the running
    // climb pose accumulator `d3`, wrap the upper/lower limits across the
    // 0..$BF cycle, then bucket `d3 >> 2` through `unk_A3F0/off_A420`.
    let next = i32::from(pose_accum) + (y_velocity >> 13);
    let pose_accum = if next < 0 {
        0x00BF
    } else if next >= 0x00BF {
        0
    } else {
        next as u16
    };

    let sprite_mapping = match pose_accum >> 2 {
        0..=15 => "LnkTo_unk_C209E-Data_Index",
        16..=31 => "LnkTo_unk_C1E18-Data_Index",
        _ => "LnkTo_unk_C1B92-Data_Index",
    };

    IronKnightClimbVisualSubA3BC {
        pose_accum,
        sprite_mapping,
    }
}

/// Diamond Power type active for the current activation.
/// ASM: DiamondPower_Init_Index (kid.asm:44375) maps form+tier to power type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DiamondPowerType {
    #[default]
    None,
    /// Kid/IronKnight(T1)/Maniaxe(T1): orbiting diamonds damage enemies on contact.
    CircleOfDoom,
    /// Skycutter(T1)/Berzerker(T1): pure invulnerability, 480f.
    Invulnerability,
    /// RedStealth(T1): invulnerability variant, 480f.
    SamuraiHaze,
    /// Cyclone(T1): projectiles rain from top of screen, 720f.
    SlashingRain,
    /// Cyclone(T2): longer rain with tracking, 1800f.
    TrackingRain,
    /// Juggernaut: 5 projectiles in spread, costs 5 diamonds, 80f.
    FiveWayShot,
    /// Micromax(T1): 6 forward-firing diamond chain, 480f.
    MiniSnake,
    /// Micromax(T2)/Kid(T2)/Skycutter(T2)/RedStealth(T2): faster snake, 1200f.
    DeathSnake,
    /// IronKnight(T2): instant +1 HP.
    ExtraHitPoint,
    /// Maniaxe(T2): instant +1 life.
    ExtraLife,
    /// Berzerker(T2): vertical diamond wall slides across screen, 1200f.
    WallOfDeath,
}

/// Map form + tier to Diamond Power type. ASM: DiamondPower_Init_Index.
pub(crate) fn diamond_power_for_form(form: PlayerForm, tier2: bool) -> DiamondPowerType {
    match (form, tier2) {
        (PlayerForm::TheKid, false) => DiamondPowerType::CircleOfDoom,
        (PlayerForm::TheKid, true) => DiamondPowerType::DeathSnake,
        (PlayerForm::Skycutter, false) => DiamondPowerType::Invulnerability,
        (PlayerForm::Skycutter, true) => DiamondPowerType::DeathSnake,
        (PlayerForm::Cyclone, false) => DiamondPowerType::SlashingRain,
        (PlayerForm::Cyclone, true) => DiamondPowerType::TrackingRain,
        (PlayerForm::RedStealth, false) => DiamondPowerType::SamuraiHaze,
        (PlayerForm::RedStealth, true) => DiamondPowerType::DeathSnake,
        (PlayerForm::Eyeclops, _) => DiamondPowerType::None, // blocked
        (PlayerForm::Juggernaut, _) => DiamondPowerType::FiveWayShot,
        (PlayerForm::IronKnight, false) => DiamondPowerType::CircleOfDoom,
        (PlayerForm::IronKnight, true) => DiamondPowerType::ExtraHitPoint,
        (PlayerForm::Berzerker, false) => DiamondPowerType::Invulnerability,
        (PlayerForm::Berzerker, true) => DiamondPowerType::WallOfDeath,
        (PlayerForm::Maniaxe, false) => DiamondPowerType::CircleOfDoom,
        (PlayerForm::Maniaxe, true) => DiamondPowerType::ExtraLife,
        (PlayerForm::Micromax, false) => DiamondPowerType::MiniSnake,
        (PlayerForm::Micromax, true) => DiamondPowerType::DeathSnake,
    }
}

/// Duration in frames for each Diamond Power type.
pub(crate) fn diamond_power_duration(power: DiamondPowerType) -> u16 {
    match power {
        DiamondPowerType::None => 0,
        DiamondPowerType::CircleOfDoom => 480, // indefinite in ASM, use 480 as practical limit
        DiamondPowerType::Invulnerability => 480, // $1E0
        DiamondPowerType::SamuraiHaze => 480,  // $1E0
        DiamondPowerType::SlashingRain => 720, // $2D0
        DiamondPowerType::TrackingRain => 1800, // $708
        DiamondPowerType::FiveWayShot => 80,   // $50
        DiamondPowerType::MiniSnake => 480,    // $1E0
        DiamondPowerType::DeathSnake => 1200,  // $4B0
        DiamondPowerType::ExtraHitPoint => 0,  // instant
        DiamondPowerType::ExtraLife => 0,      // instant
        DiamondPowerType::WallOfDeath => 1200, // $4B0
    }
}

#[allow(non_snake_case)]
pub(crate) fn juggernaut_five_way_shot_ready_sub_7428(
    current_form: PlayerForm,
    input: FrameInput,
    diamond_count: u16,
) -> bool {
    // ASM sub_7428 (kid.asm:7569): only Juggernaut with Start+A held and at
    // least 5 diamonds may arm `FiveWayShotReady`.
    current_form == PlayerForm::Juggernaut && input.start && input.a && diamond_count >= 5
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DiamondPowerObjectListsSub3F5E8 {
    pub(crate) active_slots: Vec<u16>,
    pub(crate) free_slots: Vec<u16>,
    pub(crate) last_active_slot: Option<u16>,
    pub(crate) diamond_power_active: bool,
    pub(crate) five_way_shot_ready: bool,
    pub(crate) samurai_haze_active: bool,
    pub(crate) kid_is_invulnerable: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_object_remove_sub_3F5E8(
    mut lists: DiamondPowerObjectListsSub3F5E8,
    remove_slot: u16,
) -> DiamondPowerObjectListsSub3F5E8 {
    // ASM loc_3F5EC: traversal loop — `move.l 4(a1),d0; cmp.l d0,a0;
    // beq.s loc_3F5F8; move.l d0,a1; bra.s loc_3F5EC` walks the singly-
    // linked active list from the sentinel ($FFFFF5A0) to find the
    // predecessor of `a0`.
    let remove_index = lists
        .active_slots
        .iter()
        .position(|&slot| slot == remove_slot)
        .expect("ASM sub_3F5E8 expects a live diamond-power slot");
    lists.active_slots.remove(remove_index);
    lists.free_slots.insert(0, remove_slot);

    // ASM loc_3F5F8: `cmp.l (Addr_LastDPObjectSlot).w,a0; bne.s loc_3F610`
    // checks whether the removed slot was the tail. If the predecessor is
    // the sentinel, `clr.l (Addr_LastDPObjectSlot).w`; otherwise
    // ASM loc_3F60C: `move.l a1,(Addr_LastDPObjectSlot).w` updates the
    // tail pointer to the predecessor.
    if lists.last_active_slot == Some(remove_slot) {
        lists.last_active_slot = lists.active_slots.last().copied();
    }

    // ASM loc_3F610: `tst.l (Addr_FirstDPObjectSlot).w; sne (Diamond_power_active).w`.
    // If no active slots remain, it also clears the three shared flags (loc_3F62E).
    lists.diamond_power_active = !lists.active_slots.is_empty();
    if !lists.diamond_power_active {
        lists.five_way_shot_ready = false;
        lists.samurai_haze_active = false;
        lists.kid_is_invulnerable = false;
    }

    lists
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_object_reset_sub_3F63C(
    mut lists: DiamondPowerObjectListsSub3F5E8,
) -> DiamondPowerObjectListsSub3F5E8 {
    // ASM sub_3F63C clears `Addr_FirstDPObjectSlot` and
    // `Addr_NextFreeDPObjectSlot`, then drops the shared diamond-power flags.
    // It does not touch `Addr_LastDPObjectSlot`.
    lists.active_slots.clear();
    lists.free_slots.clear();
    lists.diamond_power_active = false;
    lists.five_way_shot_ready = false;
    lists.samurai_haze_active = false;
    lists.kid_is_invulnerable = false;
    lists
}

/// A single VDP sprite entry as compiled by DiamondPower_CompileSprites.
/// ASM sprite table format: Y (word), size+link (word), pattern (word), X (word).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CompiledDpSprite {
    pub(crate) y: u16,
    pub(crate) size_link: u16,
    pub(crate) pattern: u16,
    pub(crate) x: u16,
}

/// ASM DiamondPower_CompileSprites (kid.asm:44441, loc_3F798).
/// Copies diamond-power sprite entries from the DP sprite buffer into the VDP
/// sprite table, adjusting each entry's link byte by adding the running sprite
/// count and masking the pattern's high bit.
///
/// ASM loc_3F7AE: inner loop — for each 8-byte entry:
///   `move.w (a1)+,(a2)+` copies Y;
///   `addq.w #1,d2; add.w d2,d0; move.w d0,(a2)+` adjusts size+link;
///   `andi.w #$7FFF,d0; move.w d0,(a2)+` masks pattern bit 15;
///   `move.w (a1)+,(a2)+` copies X;
///   loops until a1 reaches a0 (buffer end).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_compile_sprites_sub_3F798(
    buffer: &[[u16; 4]],
    initial_sprite_count: u8,
) -> (Vec<CompiledDpSprite>, u8) {
    // ASM: if buffer empty (a0 == a1), return immediately.
    if buffer.is_empty() {
        return (Vec::new(), initial_sprite_count);
    }
    let mut d2 = initial_sprite_count as u16;
    let mut out = Vec::with_capacity(buffer.len());
    for entry in buffer {
        // ASM loc_3F7AE: copy Y word
        let y = entry[0];
        // addq.w #1,d2; add.w d2,d0 — adjust link by running count
        d2 = d2.wrapping_add(1);
        let size_link = entry[1].wrapping_add(d2);
        // andi.w #$7FFF,d0 — clear pattern high bit
        let pattern = entry[2] & 0x7FFF;
        // copy X word
        let x = entry[3];
        out.push(CompiledDpSprite {
            y,
            size_link,
            pattern,
            x,
        });
    }
    (out, d2 as u8)
}

/// State of a single Circle-of-Doom diamond-power object slot.
/// ASM fields: $3E(a0)=radius (longword), $40(a0)=shrink flag (word),
/// $42(a0)=angle (longword), $38(a0)=status (word).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CircleOfDoomSlot {
    /// ASM $3E(a0): radius in 8.8 fixed point (longword). Upper word is the
    /// integer radius used for position calculation; lower 16 bits are fractional.
    pub(crate) radius: u32,
    /// ASM $40(a0): cleared to 0 on init (word).
    pub(crate) shrink_flag: u16,
    /// ASM $42(a0): angle in 16.16 fixed point (longword). Upper word is the
    /// 8-bit angle fed to sub_40236; lower 16 bits are fractional.
    pub(crate) angle: u32,
    /// ASM $38(a0): status word — nonzero triggers sub_3F5E8 (removal).
    pub(crate) status: u16,
}

/// ASM DiamondPower_Init_CircleOfDoom (kid.asm:44469, loc_3F7D0).
/// Allocates 8 diamond-power object slots with evenly-spaced angles.
///
/// d1=8 (count), d2 = ($10000 / 8) << 8 = $2000 << 8 = $200000 (angle step),
/// d3 starts at 0 (first angle), d0 = $00210000 (gfx ID, unused in native).
///
/// ASM loc_3F7E6: allocation loop — for each slot:
///   `bsr.w Allocate_DiamondPowerObjectSlot` allocates;
///   `move.l d0,$3E(a0)` stores gfx ID as initial radius (=$00210000);
///   `clr.w $40(a0)` clears shrink flag;
///   `move.l d3,$42(a0)` stores angle;
///   `add.l d2,d3` advances angle by step;
///   `dbf d1,loc_3F7E6` loops 8 times (d1=7 after subq).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_init_circle_of_doom_sub_3F7D0() -> Vec<CircleOfDoomSlot> {
    let count: u16 = 8;
    // ASM: divu.w d1,d2 → $10000 / 8 = $2000; lsl.l #8,d2 → $200000
    let angle_step: u32 = (0x10000u32 / count as u32) << 8;
    // ASM: moveq #$21,d0; swap d0; clr.w d0 → d0 = $00210000
    let initial_radius: u32 = 0x0021_0000;
    let mut d3: u32 = 0;
    let mut slots = Vec::with_capacity(count as usize);
    for _ in 0..count {
        slots.push(CircleOfDoomSlot {
            radius: initial_radius,
            shrink_flag: 0,
            angle: d3,
            status: 0,
        });
        d3 = d3.wrapping_add(angle_step);
    }
    slots
}

/// Radius-to-sprite-size lookup table.
/// ASM byte_3F8B4 (kid.asm:44560): 28 entries indexed by `$3E(a0).w >> 1`.
/// Each entry is the pixel half-size for the diamond sprite at that radius.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const CIRCLE_OF_DOOM_SIZE_TABLE: [u8; 28] = [
    0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
];

/// Per-slot position and size result from DiamondPower_Main_CircleOfDoom.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CircleOfDoomSpriteResult {
    /// Screen X after camera offset (d0 + d3 in ASM).
    pub(crate) screen_x: i16,
    /// Screen Y after camera offset (d1 + d4 in ASM).
    pub(crate) screen_y: i16,
    /// Sprite half-size from byte_3F8B4 table.
    pub(crate) half_size: u8,
    /// Whether the slot was removed (status nonzero or radius depleted).
    pub(crate) removed: bool,
}

/// ASM DiamondPower_Main_CircleOfDoom (kid.asm:44490) per-slot processing.
///
/// For each slot in the linked list starting at Addr_FirstDPObjectSlot:
///
/// ASM loc_3F81A: per-slot body —
///   d5 = $3E(a0) >> 8 (radius integer part);
///   angle = $42(a0).w (upper word of angle longword);
///   (cos, sin) = sub_40236(angle);
///   x = (cos * d5) >> 16 >> 6 + kid_x;
///   y = -(sin * d5) >> 16 >> 6 + kid_y - $10;
///   half_size = byte_3F8B4[$3E(a0).w >> 1];
///   if $38(a0) != 0 → call sub_3F5E8 (remove slot);
///   else shrink radius: $3E -= $400; if $3E.w >> 1 < $C → $3E -= $19000;
///     if $3E < 0 → remove slot;
///   angle -= 4 ($42 subq.w #4);
///   next slot via 4(a0).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_main_circle_of_doom_slot(
    slot: &mut CircleOfDoomSlot,
    kid_x: i16,
    kid_y: i16,
    camera_x: i16,
    camera_y: i16,
) -> CircleOfDoomSpriteResult {
    use crate::boot::sin_cos_sub_40236;

    // ASM: d3 = $7F - Camera_X_pos; d4 = $7F - Camera_Y_pos
    let d3 = 0x7Fi16.wrapping_sub(camera_x);
    let d4 = 0x7Fi16.wrapping_sub(camera_y);

    // ASM loc_3F81A: d5 = $3E(a0) >> 8 (lsr.l #8,d5)
    let d5 = (slot.radius >> 8) as u16;

    // ASM: angle = upper word of $42(a0) — move.w $42(a0),d0
    let angle = (slot.angle >> 16) as u8;
    let (cos, sin) = sin_cos_sub_40236(angle);

    // ASM: muls.w d5,d0; swap d0; asr.w #6,d0; add.w x_pos(a3),d0
    let x_offset = ((cos as i32 * d5 as i32) >> 16) as i16 >> 6;
    let world_x = kid_x.wrapping_add(x_offset);
    // ASM: move.w d0,$1A(a0); add.w d3,d0 — screen X
    let screen_x = world_x.wrapping_add(d3);

    // ASM: muls.w d5,d1; swap d1; asr.w #6,d1; neg.w d1; add.w y_pos(a3),d1; subi.w #$10,d1
    let y_offset = -(((sin as i32 * d5 as i32) >> 16) as i16 >> 6);
    let world_y = kid_y.wrapping_add(y_offset).wrapping_sub(0x10);
    // ASM: move.w d1,$1E(a0); add.w d4,d1 — screen Y
    let screen_y = world_y.wrapping_add(d4);

    // ASM: d5 = $3E(a0).w (upper word); d6 = byte_3F8B4[d5 >> 1]
    let radius_word = (slot.radius >> 16) as u16;
    let size_index = (radius_word >> 1) as usize;
    let half_size = if size_index < CIRCLE_OF_DOOM_SIZE_TABLE.len() {
        CIRCLE_OF_DOOM_SIZE_TABLE[size_index]
    } else {
        *CIRCLE_OF_DOOM_SIZE_TABLE.last().unwrap()
    };

    // ASM: tst.w $38(a0); bne.s loc_3F87A (remove)
    if slot.status != 0 {
        return CircleOfDoomSpriteResult {
            screen_x,
            screen_y,
            half_size,
            removed: true,
        };
    }

    // ASM: subi.l #$400,$3E(a0) — shrink radius
    slot.radius = slot.radius.wrapping_sub(0x400);
    // ASM: cmpi.w #$C,d5 — compare upper word >> 1 to $C
    if size_index < 0xC {
        // ASM: subi.l #$19000,$3E(a0); bge.s loc_3F880
        slot.radius = slot.radius.wrapping_sub(0x19000);
        if (slot.radius as i32) < 0 {
            return CircleOfDoomSpriteResult {
                screen_x,
                screen_y,
                half_size,
                removed: true,
            };
        }
    }

    // ASM: subq.w #4,$42(a0) — decrement angle
    let angle_upper = (slot.angle >> 16) as u16;
    let angle_lower = slot.angle as u16;
    let new_angle_upper = angle_upper.wrapping_sub(4);
    slot.angle = (new_angle_upper as u32) << 16 | angle_lower as u32;

    // ASM loc_3F8AA: move.l 4(a0),d0; bne.w loc_3F81A; rts
    // The linked-list traversal tail: loads the next-slot pointer from offset 4
    // of the current slot. Nonzero continues the loop (back to loc_3F81A);
    // zero terminates. Both the removal path (loc_3F87A → sub_3F5E8 → bra.s
    // loc_3F8AA) and the compile-sprite path (loc_3F880 fallthrough) converge
    // here. In native code the caller iterates the slot Vec, so this
    // linked-list plumbing is implicit.
    CircleOfDoomSpriteResult {
        screen_x,
        screen_y,
        half_size,
        removed: false,
    }
}

/// A single snake diamond-power slot.
/// ASM: each slot occupies the standard DP-object layout, with:
///   $1A(a0): x_pos (word)
///   $1E(a0): y_pos (word)
///   $24(a0): diamond size (word, from init-data table)
///   $2E(a0): wander speed d4 (word, first slot only)
///   $30(a0): movement speed d5 (word, first slot only)
///   $3E(a0): timer $40 (word, first slot only)
///   $40(a0)-$4A(a0): 3 trail positions (x,y,x,y,x,y)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SnakeSlot {
    pub(crate) x_pos: i16,
    pub(crate) y_pos: i16,
    /// Diamond size from the variant's init-data table.
    pub(crate) diamond_size: u16,
    /// Trail buffer: 3 previous positions (x,y pairs).
    pub(crate) trail: [(i16, i16); 3],
}

/// Result of `diamond_power_init_snake`: the full snake state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SnakeInitResult {
    pub(crate) slots: Vec<SnakeSlot>,
    /// Timer stored at $3E(a0) of the head slot ($40).
    pub(crate) head_timer: u16,
    /// Wander speed stored at $2E(a0) of the head slot (d4).
    pub(crate) wander_speed: u16,
    /// Movement speed stored at $30(a0) of the head slot (d5).
    pub(crate) movement_speed: u16,
    /// Diamond power timer value written to (Diamond_power_timer).w.
    pub(crate) power_timer: u16,
}

/// Init-data tables for each snake variant.
/// ASM: DiamondPower_InitData_SwiftMiniSnake (kid.asm:44598),
///      DiamondPower_InitData_MiniSnake (kid.asm:44610),
///      DiamondPower_InitData_DeathSnake (kid.asm:44651).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SNAKE_INIT_DATA_SWIFT_MINI: [u16; 6] = [5, 4, 3, 2, 1, 1];
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SNAKE_INIT_DATA_MINI: [u16; 6] = [4, 3, 2, 1, 1, 1];
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SNAKE_INIT_DATA_DEATH: [u16; 6] = [6, 6, 5, 4, 3, 2];

/// ASM DiamondPower_Init_Snake (kid.asm:44621, loc_3F91C).
///
/// Shared init for all three snake variants (SwiftMiniSnake, MiniSnake,
/// DeathSnake). Allocates 6 slots (d1 counts 5..0), each at Kid's position
/// with diamond sizes from the variant's init-data table and a 3-position
/// trail buffer filled with Kid's coordinates.
///
/// Parameters mirror the ASM registers at entry:
///   d4 = wander speed ($10 for swift, 6 for mini/death)
///   d5 = movement speed ($480/$300/$400)
///   init_data = 6-entry size table
///   power_timer = value for (Diamond_power_timer).w
///   kid_x, kid_y = Kid's current position
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_init_snake(
    wander_speed: u16,
    movement_speed: u16,
    init_data: &[u16; 6],
    power_timer: u16,
    kid_x: i16,
    kid_y: i16,
) -> SnakeInitResult {
    // ASM loc_3F91C: move.l (Addr_GfxObject_Kid).w,a3; moveq #5,d1
    let mut slots = Vec::with_capacity(6);

    for i in 0..6u16 {
        // ASM loc_3F92A: bsr.w Allocate_DiamondPowerObjectSlot
        // (allocation is implicit in native — we just create the slot)

        // ASM loc_3F946: store Kid x/y, read size from table, fill trail
        // move.w d6,$1A(a0) — x_pos = kid_x
        // move.w d7,$1E(a0) — y_pos = kid_y
        // move.w (a2)+,$24(a0) — diamond size from init table
        // lea $40(a0),a1 — trail buffer at offset $40
        // 3x move.w d6,(a1)+; move.w d7,(a1)+
        let slot = SnakeSlot {
            x_pos: kid_x,
            y_pos: kid_y,
            diamond_size: init_data[i as usize],
            trail: [(kid_x, kid_y); 3],
        };
        slots.push(slot);
    }

    // ASM loc_3F92A: first slot (d1==5) stores head pointer at $FFFFF5B4,
    // timer $40 at $3E(a0), d4 at $2E(a0), d5 at $30(a0)
    SnakeInitResult {
        slots,
        head_timer: 0x40,
        wander_speed,
        movement_speed,
        power_timer,
    }
}

/// ASM DiamondPower_Main_Snake trail update loop (kid.asm:44658, loc_3F978).
///
/// Iterates the snake's slot list from head to tail. For each slot except
/// the last, the visible position is shifted from trail[0], the trail buffer
/// is shifted down by one entry, and the next slot's current position fills
/// trail[2]. The last slot (4(a0)==0 → no successor) is left unchanged.
///
/// ASM loc_3F978:
///   move.l a1,a0; move.l 4(a0),d1; beq.s loc_3F9A2
///   — follow linked list; exit when next pointer is null
///   lea $40(a0),a3
///   move.w (a3),$1A(a0); move.w 2(a3),$1E(a0)
///   — copy trail[0] → x_pos, y_pos
///   move.l 4(a3),(a3)+; move.l 4(a3),(a3)+
///   — shift trail[1]→[0], trail[2]→[1]
///   move.w $1A(a1),(a3)+; move.w $1E(a1),(a3)+
///   — copy next slot's current position → trail[2]
///   bra.s loc_3F978
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_snake_update_trails(slots: &mut [SnakeSlot]) {
    // ASM loc_3F978: iterate pairs (current, next) in order
    for i in 0..slots.len().saturating_sub(1) {
        // ASM: move.w (a3),$1A(a0); move.w 2(a3),$1E(a0)
        // — trail[0] becomes the slot's visible position
        let (tx, ty) = slots[i].trail[0];
        slots[i].x_pos = tx;
        slots[i].y_pos = ty;

        // ASM: move.l 4(a3),(a3)+; move.l 4(a3),(a3)+
        // — shift trail entries down: [1]→[0], [2]→[1]
        slots[i].trail[0] = slots[i].trail[1];
        slots[i].trail[1] = slots[i].trail[2];

        // ASM: move.w $1A(a1),(a3)+; move.w $1E(a1),(a3)+
        // — next slot's current position fills trail[2]
        let next_pos = (slots[i + 1].x_pos, slots[i + 1].y_pos);
        slots[i].trail[2] = next_pos;
    }
}

/// An on-screen enemy position for the snake target scan.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SnakeTargetCandidate {
    pub(crate) x_pos: i16,
    pub(crate) y_pos: i16,
}

/// Result of `diamond_power_snake_find_target`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SnakeTarget {
    /// Target x_pos (from nearest enemy, or Kid if none on-screen).
    pub(crate) x: i16,
    /// Target y_pos (from nearest enemy, or Kid if none on-screen).
    pub(crate) y: i16,
}

/// ASM loc_3F9A2 + loc_3F9C2: snake target selection after trail update.
///
/// loc_3F9A2 (kid.asm:44673): decrements Diamond_power_timer, loads the
/// head slot's screen-relative position, then scans the enemy list for
/// the nearest on-screen enemy by Manhattan distance.
///
/// loc_3F9C2 (kid.asm:44684): per-enemy proximity scan:
///   — load enemy x/y, subtract 16 from y
///   — cull off-screen: x in [-8, $148), y in [$10, $F0)
///   — compute |dx| + |dy| Manhattan distance to head
///   — track nearest (smallest distance in d3)
///
/// If no on-screen enemy found (d3 remains $FFFF), defaults to Kid.
///
/// Returns the target position (enemy or Kid).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_snake_find_target(
    head_x: i16,
    head_y: i16,
    camera_x: i16,
    camera_y: i16,
    enemies: &[SnakeTargetCandidate],
    kid_x: i16,
    kid_y: i16,
) -> SnakeTarget {
    // ASM loc_3F9A2: d0 = head_x - Camera_X_pos, d1 = head_y - Camera_Y_pos
    let screen_head_x = head_x.wrapping_sub(camera_x);
    let screen_head_y = head_y.wrapping_sub(camera_y);

    // ASM: moveq #-1,d3 — initial "no target" sentinel
    let mut best_dist: i16 = -1;
    let mut best_target: Option<(i16, i16)> = None;

    // ASM loc_3F9C2: iterate enemy linked list ($FFFFF86A)
    for enemy in enemies {
        // ASM: move.w $1A(a1),d4; move.w $1E(a1),d5; subi.w #$10,d5
        let ex = enemy.x_pos;
        let ey = enemy.y_pos.wrapping_sub(0x10);

        // ASM: sub.w (Camera_X_pos).w,d4; screen-relative X
        let screen_ex = ex.wrapping_sub(camera_x);
        // ASM: cmpi.w #$FFF8,d4; blt.s loc_3FA06
        if screen_ex < -8 {
            continue;
        }
        // ASM: cmpi.w #$148,d4; bge.s loc_3FA06
        if screen_ex >= 0x148 {
            continue;
        }
        // ASM: sub.w (Camera_Y_pos).w,d5
        let screen_ey = ey.wrapping_sub(camera_y);
        // ASM: cmpi.w #$10,d5; blt.s loc_3FA06
        if screen_ey < 0x10 {
            continue;
        }
        // ASM: cmpi.w #$F0,d5; bge.s loc_3FA06
        if screen_ey >= 0xF0 {
            continue;
        }

        // ASM: sub.w d0,d4; bge.s loc_3F9F6; neg.w d4
        let dx = (screen_ex.wrapping_sub(screen_head_x)).abs();
        // ASM: sub.w d1,d5; bge.s loc_3F9FC; neg.w d5
        let dy = (screen_ey.wrapping_sub(screen_head_y)).abs();

        // ASM: add.w d4,d5; cmp.w d5,d3; bcs.s loc_3FA06
        let dist = dx.wrapping_add(dy);
        // ASM: cmp.w d5,d3 — unsigned compare (bcs = below/carry set)
        if (best_dist as u16) < (dist as u16) {
            continue;
        }
        best_dist = dist;
        best_target = Some((enemy.x_pos, enemy.y_pos));
    }

    // ASM loc_3FA0C: cmpi.w #$FFFF,d3; bne.s loc_3FA16
    // — if no enemy found, use Kid
    match best_target {
        Some((x, y)) => SnakeTarget { x, y },
        None => SnakeTarget { x: kid_x, y: kid_y },
    }
}

/// Result of `diamond_power_rain_find_nearest_enemy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct RainNearestEnemy {
    /// Manhattan distance to nearest on-screen enemy.
    pub(crate) distance: u16,
    /// Target enemy x_pos (world coordinates).
    pub(crate) x: i16,
    /// Target enemy y_pos (world coordinates).
    pub(crate) y: i16,
}

/// ASM loc_3FEE2 (kid.asm:45253): rain tracking enemy scan.
///
/// Called from loc_3FEBC when a non-tracking rain object's timer expires
/// and tracking is enabled ($FFFFF5B9 != 0, i.e. TrackingRain).
///
/// Iterates the enemy linked list ($FFFFF86A), for each enemy:
///   — load x/y, subtract 8 from y (vs 16 for snake)
///   — cull off-screen: x in [-8, $148), y in [$10, $F0) after camera
///   — loc_3FF16 (kid.asm:45272): abs(dx) after sub.w d0,d5; bge.s; neg.w d5
///   — loc_3FF1C (kid.asm:45277): abs(dy) after sub.w d1,d6; bge.s; neg.w d6,
///     then add.w d5,d6 for Manhattan distance, cmp.w d6,d7 tracks nearest
///
/// Returns the nearest on-screen enemy, or None if no enemies visible.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_rain_find_nearest_enemy(
    rain_x: i16,
    rain_y: i16,
    camera_x: i16,
    camera_y: i16,
    enemies: &[SnakeTargetCandidate],
) -> Option<RainNearestEnemy> {
    // ASM loc_3FEBC: d0 = rain $1A(a0) - Camera_X_pos, d1 = rain $1E(a0) - Camera_Y_pos
    let screen_rain_x = rain_x.wrapping_sub(camera_x);
    let screen_rain_y = rain_y.wrapping_sub(camera_y);

    // ASM: moveq #-1,d7 — initial "no target" sentinel
    let mut best_dist: u16 = 0xFFFF;
    let mut best_target: Option<(i16, i16)> = None;

    // ASM loc_3FEE2: iterate enemy linked list ($FFFFF86A)
    for enemy in enemies {
        // ASM loc_3FEE2: move.w $1A(a1),d5; move.w $1E(a1),d6; subi.w #8,d6
        let ex = enemy.x_pos;
        let ey = enemy.y_pos.wrapping_sub(8);

        // ASM: sub.w (Camera_X_pos).w,d5
        let screen_ex = ex.wrapping_sub(camera_x);
        // ASM: cmpi.w #$FFF8,d5; blt.s loc_3FF26
        if screen_ex < -8 {
            continue;
        }
        // ASM: cmpi.w #$148,d5; bge.s loc_3FF26
        if screen_ex >= 0x148 {
            continue;
        }
        // ASM: sub.w (Camera_Y_pos).w,d6
        let screen_ey = ey.wrapping_sub(camera_y);
        // ASM: cmpi.w #$10,d6; blt.s loc_3FF26
        if screen_ey < 0x10 {
            continue;
        }
        // ASM: cmpi.w #$F0,d6; bge.s loc_3FF26
        if screen_ey >= 0xF0 {
            continue;
        }

        // ASM loc_3FF16: sub.w d0,d5; bge.s loc_3FF16; neg.w d5
        let dx = (screen_ex.wrapping_sub(screen_rain_x)).unsigned_abs();
        // ASM loc_3FF1C: sub.w d1,d6; bge.s loc_3FF1C; neg.w d6
        let dy = (screen_ey.wrapping_sub(screen_rain_y)).unsigned_abs();

        // ASM loc_3FF1C: add.w d5,d6; cmp.w d6,d7; bcs.s loc_3FF26
        let dist = dx.wrapping_add(dy);
        if best_dist < dist {
            continue;
        }
        best_dist = dist;
        best_target = Some((enemy.x_pos, enemy.y_pos));
    }

    best_target.map(|(x, y)| RainNearestEnemy {
        distance: best_dist,
        x,
        y,
    })
}

/// Result of `diamond_power_rain_compute_tracking_velocity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct RainTrackingVelocity {
    /// 32-bit x velocity toward the enemy.
    pub(crate) x_vel: i32,
    /// 32-bit y velocity toward the enemy (aimed 8px above).
    pub(crate) y_vel: i32,
    /// Shift amount used (d6): floor(log2(distance >> 3)) + 1.
    pub(crate) shift: u32,
}

/// ASM loc_3FF3C (kid.asm:45296): tracking velocity computation loop.
///
/// After `diamond_power_rain_find_nearest_enemy` finds a target and the
/// gate at loc_3FF2C validates distance in [8, $40):
///
/// 1. `lsr.w #3,d7` — shift distance right by 3
/// 2. loc_3FF3C loop: `addq.w #1,d6; lsr.w #1,d7; bne.s loc_3FF3C` —
///    count bits remaining, giving d6 = floor(log2(distance >> 3)) + 1
/// 3. `move.l x_pos(a3),d0; sub.l $1A(a0),d0; asr.l d6,d0; move.l d0,$26(a0)`
///    — x_vel = (enemy_x - rain_x) >> d6
/// 4. `move.l y_pos(a3),d0; subi.l #$80000,d0; sub.l $1E(a0),d0; asr.l d6,d0`
///    — y_vel = (enemy_y - $80000 - rain_y) >> d6
///    The $80000 (8 pixels in 16.16 fixed-point) aims slightly above the enemy.
/// 5. `addq.w #1,$3E(a0)` marks the rain object as tracking.
///
/// Returns `None` if distance is outside [8, $40) (loc_3FF2C branches
/// to loc_3FF6E, resetting the frame counter without engaging tracking).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_rain_compute_tracking_velocity(
    distance: u16,
    enemy_x_pos: u32,
    enemy_y_pos: u32,
    rain_x_pos: u32,
    rain_y_pos: u32,
) -> Option<RainTrackingVelocity> {
    // ASM loc_3FF2C: cmpi.w #8,d7; bcs.s loc_3FF6E
    if distance < 8 {
        return None;
    }
    // ASM loc_3FF2C: cmpi.w #$40,d7; bcc.s loc_3FF6E
    if distance >= 0x40 {
        return None;
    }
    // ASM: moveq #0,d6; lsr.w #3,d7
    let mut d7 = distance >> 3;
    let mut d6: u32 = 0;
    // ASM loc_3FF3C: addq.w #1,d6; lsr.w #1,d7; bne.s loc_3FF3C
    loop {
        d6 += 1;
        d7 >>= 1;
        if d7 == 0 {
            break;
        }
    }
    // ASM: move.l x_pos(a3),d0; sub.l $1A(a0),d0; asr.l d6,d0
    let x_vel = (enemy_x_pos as i32).wrapping_sub(rain_x_pos as i32) >> d6;
    // ASM: move.l y_pos(a3),d0; subi.l #$80000,d0; sub.l $1E(a0),d0; asr.l d6,d0
    let y_vel = (enemy_y_pos as i32)
        .wrapping_sub(0x8_0000)
        .wrapping_sub(rain_y_pos as i32)
        >> d6;
    Some(RainTrackingVelocity {
        x_vel,
        y_vel,
        shift: d6,
    })
}

/// ASM loc_3FF70 (kid.asm:45317): non-tracking rain object movement.
///
/// loc_3FF6E (kid.asm:45314): entry when no valid tracking target found —
/// resets d7 to $14 (20 frames until next scan), then falls through to
/// loc_3FF70.
///
/// loc_3FF70: applies movement for non-tracking rain:
/// - X: 32-bit update: `move.l $1A(a0),d0; add.l $26(a0),d0; add.l a5,d0`
///   where a5 = Kid x_vel * 3/4 (drift computed at loc_3FE62).
/// - Y: word-only update: `move.w $1E(a0),d1; add.w $2A(a0),d1`
///   (no subpixel precision, unlike the tracking path at loc_3FE98 which
///   uses 32-bit Y).
///
/// Returns (new_x_pos_32bit, new_y_pos_word).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_rain_non_tracking_move(
    rain_x: u32,
    rain_y: u16,
    rain_x_vel: u32,
    rain_y_vel: u16,
    kid_drift: u32,
) -> (u32, u16) {
    // ASM loc_3FF70: move.l $1A(a0),d0; add.l $26(a0),d0; add.l a5,d0
    let new_x = rain_x.wrapping_add(rain_x_vel).wrapping_add(kid_drift);
    // ASM loc_3FF70: move.w $1E(a0),d1; add.w $2A(a0),d1
    let new_y = rain_y.wrapping_add(rain_y_vel);
    (new_x, new_y)
}

/// ASM loc_3FA16 through loc_3FA46: compute 8-way octant direction from
/// the snake head to its target, used by the snake steering loop.
///
/// loc_3FA16 (kid.asm:44724): compute relative dx/dy from head to target,
/// initialise octant base d3=0. If dy (head_y - (target_y-$10)) < 0: d3=6,
/// negate dy.
///
/// loc_3FA36 (kid.asm:44737): tst.w d0; bpl.s loc_3FA40; eori.w #2,d3;
/// neg.w d0 — if dx (target_x - head_x) < 0, flip bit 1 of octant and
/// take |dx|.
///
/// loc_3FA40 (kid.asm:44743): cmp.w d0,d1; bge.s loc_3FA46; addq.w #1,d3
/// — if |dy| < |dx| (more horizontal), add 1 to octant.
///
/// loc_3FA46 (kid.asm:44748): btst #1,d3; bne.s loc_3FA50; eori.w #1,d3
/// — when bit 1 is clear, flip bit 0 (corrects axis alignment encoding).
///
/// Returns a 3-bit octant index (0–7).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_snake_octant(
    target_x: i16,
    target_y: i16,
    head_x: i16,
    head_y: i16,
) -> u16 {
    // ASM loc_3FA16: d0 = x_pos(a3) - $1A(a0) = target_x - head_x
    let dx_raw = target_x.wrapping_sub(head_x);
    // ASM: d1 = y_pos(a3) - $10; sub.w $1E(a0),d1; neg.w d1
    //    = head_y - (target_y - $10)
    let dy_raw = head_y.wrapping_sub(target_y.wrapping_sub(0x10));

    // ASM: moveq #0,d3; tst.w d1; bpl.s loc_3FA36
    let mut d3: u16 = 0;
    let abs_dy = if dy_raw < 0 {
        // ASM: moveq #6,d3; neg.w d1
        d3 = 6;
        (dy_raw as u16).wrapping_neg() as i16
    } else {
        dy_raw
    };

    // ASM loc_3FA36: tst.w d0; bpl.s loc_3FA40; eori.w #2,d3; neg.w d0
    let abs_dx = if dx_raw < 0 {
        d3 ^= 2;
        (dx_raw as u16).wrapping_neg() as i16
    } else {
        dx_raw
    };

    // ASM loc_3FA40: cmp.w d0,d1; bge.s loc_3FA46; addq.w #1,d3
    if abs_dy < abs_dx {
        d3 = d3.wrapping_add(1);
    }

    // ASM loc_3FA46: btst #1,d3; bne.s loc_3FA50; eori.w #1,d3
    if d3 & 2 == 0 {
        d3 ^= 1;
    }

    d3
}

/// Angle-steering result from `diamond_power_snake_steer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum SnakeSteerResult {
    /// Angle was adjusted (turn toward target or speed-based correction).
    Adjusted(u16),
    /// Angle unchanged (same octant, target at same position as head).
    Unchanged,
}

/// ASM loc_3FA50 through loc_3FAD4 (kid.asm:44753–44822): steer the snake
/// head's angle toward the target octant, with speed control when already
/// aimed correctly.
///
/// loc_3FA50: extract current octant from angle (`($3E >> 5) & 7`), compare
/// to target octant d3. If equal, branch to loc_3FA72 speed control.
///
/// loc_3FA6A (kid.asm:44766): negative octant difference path —
/// `cmpi.w #$FFFC,d3; bge.s loc_3FAC2` — when diff in [-4, -1], subtract
/// turn_rate (shortest counter-clockwise); when diff < -4, add turn_rate
/// (shorter clockwise wrap).
///
/// loc_3FA72 (kid.asm:44772): same-octant speed control — `move.w d0,d5;
/// or.w d1,d5; beq.s loc_3FAD4` skips adjustment when target==head. Otherwise
/// loads angle into d5, negates if octant bit 0 set, then falls through to
/// loc_3FA84 for trig-based speed comparison.
///
/// loc_3FA84 (kid.asm:44781): diagonal octant axis swap — `moveq #$66,d6;
/// btst d3,d6; beq.s loc_3FA8C; exg d0,d1`. For diagonal octants (bits
/// 1,2,5,6 of $66), swaps abs_dx/abs_dy before computing the ratio
/// `(abs_dy/2 << 16) / abs_dx` vs trig-table tangent to decide whether to
/// add or subtract turn_rate.
///
/// Returns the updated angle word, or `Unchanged` when loc_3FAD4 is reached
/// without modification (same octant, zero displacement).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_snake_steer(
    target_octant: u16,
    abs_dx: u16,
    abs_dy: u16,
    current_angle: u16,
    turn_rate: u16,
) -> SnakeSteerResult {
    // ASM loc_3FA50: d4 = ($3E(a0) >> 5) & 7 — current octant from angle
    let current_octant = (current_angle >> 5) & 7;

    // ASM: cmp.w d3,d4; beq.s loc_3FA72
    if target_octant == current_octant {
        // ASM loc_3FA72: move.w d0,d5; or.w d1,d5; beq.s loc_3FAD4
        if abs_dx == 0 && abs_dy == 0 {
            return SnakeSteerResult::Unchanged;
        }

        // ASM: move.w $3E(a0),d5
        let mut d5 = current_angle;
        // ASM: btst #0,d3; beq.s loc_3FA84; neg.w d5
        if target_octant & 1 != 0 {
            d5 = d5.wrapping_neg();
        }

        // ASM loc_3FA84: moveq #$66,d6; btst d3,d6; beq.s loc_3FA8C; exg d0,d1
        let diagonal_mask: u32 = 0x66; // bits 1,2,5,6
        let (d0, d1) = if (diagonal_mask >> target_octant) & 1 != 0 {
            (abs_dy, abs_dx)
        } else {
            (abs_dx, abs_dy)
        };

        // ASM loc_3FA8C: d4 = ((d1/2) << 16) / d0 — displacement ratio
        // M68K DIVU: if quotient >= $10000, overflow leaves register unchanged
        // (low word was 0 before DIVU since value was swap'd from high word).
        if d0 == 0 {
            // Avoid division by zero — ASM would trap, but in practice d0|d1!=0
            // and the diagonal swap ensures d0 is the larger component.
            return SnakeSteerResult::Adjusted(current_angle.wrapping_add(turn_rate));
        }
        let d4_full = (((d1 as u32) / 2) << 16) / (d0 as u32);
        let d4_ratio: u16 = if d4_full >= 0x10000 {
            0
        } else {
            d4_full as u16
        };

        // ASM: andi.w #$1F,d5; add.w d5,d5
        let sub_octant = (d5 & 0x1F) as usize;
        let sin_idx = sub_octant * 2; // word offset

        // ASM: move.w (a1,d5.w),d0; lsr.w #1,d0; swap d0
        let sin_val = SINE_TABLE_WORD_402B0[sub_octant];
        let d0_trig = ((sin_val as u32) / 2) << 16;

        // ASM: neg.w d5; addi.w #$80,d5; move.w (a1,d5.w),d1
        let cos_idx = (0x80u16.wrapping_sub(sin_idx as u16)) / 2;
        let cos_val = SINE_TABLE_WORD_402B0[cos_idx as usize];

        // ASM: divu.w d1,d0 — tangent approximation
        // Same M68K DIVU overflow semantics: unchanged register → low word 0.
        let d0_full = if cos_val == 0 {
            0xFFFF_FFFF_u32 // overflow guaranteed
        } else {
            d0_trig / (cos_val as u32)
        };
        let d0_tan: u16 = if d0_full >= 0x10000 {
            0
        } else {
            d0_full as u16
        };

        // ASM: btst #0,d3; beq.s loc_3FABE; exg d0,d4
        let (cmp_a, cmp_b) = if target_octant & 1 != 0 {
            (d4_ratio, d0_tan) // swapped
        } else {
            (d0_tan, d4_ratio)
        };

        // ASM loc_3FABE: cmp.w d4,d0; ble.s loc_3FACC
        if cmp_a <= cmp_b {
            // loc_3FACC: add turn_rate
            SnakeSteerResult::Adjusted(current_angle.wrapping_add(turn_rate))
        } else {
            // loc_3FAC2: subtract turn_rate
            SnakeSteerResult::Adjusted(current_angle.wrapping_sub(turn_rate))
        }
    } else {
        // ASM: sub.w d4,d3 — octant difference
        let diff = (target_octant as i16).wrapping_sub(current_octant as i16);

        if diff < 0 {
            // ASM loc_3FA6A: cmpi.w #$FFFC,d3; bge.s loc_3FAC2; bra.s loc_3FACC
            if diff >= -4 {
                // Shortest path: subtract turn_rate
                SnakeSteerResult::Adjusted(current_angle.wrapping_sub(turn_rate))
            } else {
                // Wrap around: add turn_rate
                SnakeSteerResult::Adjusted(current_angle.wrapping_add(turn_rate))
            }
        } else {
            // ASM: cmpi.w #4,d3; bge.s loc_3FAC2; bra.s loc_3FACC
            if diff >= 4 {
                // Wrap around: subtract turn_rate
                SnakeSteerResult::Adjusted(current_angle.wrapping_sub(turn_rate))
            } else {
                // Shortest path: add turn_rate
                SnakeSteerResult::Adjusted(current_angle.wrapping_add(turn_rate))
            }
        }
    }
}

/// Diamond-power sprite attribute table.
/// ASM word_4020E (kid.asm:45605): 10 entries of (size_byte, pattern_word),
/// stored as 4-byte pairs. Indexed by `diamond_size * 4`.
///
/// Each entry's first byte is the VDP sprite size field and the word at
/// offset +2 is the base pattern index.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const DIAMOND_SPRITE_ATTR_TABLE: [(u8, u16); 10] = [
    (0x00, 0x86EA), // d6=0x00
    (0x00, 0x86EB), // d6=0x04
    (0x00, 0x86EC), // d6=0x08
    (0x00, 0x86ED), // d6=0x0C
    (0x05, 0x86EE), // d6=0x10
    (0x05, 0x86F2), // d6=0x14
    (0x05, 0x86F6), // d6=0x18
    (0x05, 0x8231), // d6=0x1C
    (0x00, 0x822D), // d6=0x20
    (0x05, 0xE6FA), // d6=0x24
];

/// Result of processing a single snake slot in the sprite compilation loop.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum SnakeSlotSpriteAction {
    /// Slot produced a visible sprite entry (Y, size_byte, pattern, X).
    Visible {
        y: u16,
        size_byte: u8,
        pattern: u16,
        x: u16,
    },
    /// Slot off-screen or not yet ready — no sprite emitted.
    Skipped,
    /// Slot size reached zero during fade-out — slot should be removed.
    Removed,
}

/// ASM loc_3FB20 through loc_3FB88 (kid.asm:44848–44893): per-slot sprite
/// compilation for DiamondPower_Main_Snake.
///
/// loc_3FB20: fade-out check — when `Diamond_power_timer < 0`, every 16 frames
/// (`Time_Frames & $F == 0`) decrements `$24(a0)` (diamond_size). If size
/// reaches zero, calls `sub_3F5E8` (deallocate) and skips to `loc_3FB88`.
///
/// loc_3FB3E: sprite compilation — computes screen position from slot world
/// coords + camera offsets (d3=$7F-Camera_X, d4=$7F-Camera_Y), centres by
/// subtracting diamond_size, bounds-checks x,y in `(0, $1E0)`, and writes
/// the sprite entry (Y, size+link, pattern, X) to the buffer.
///
/// loc_3FB88: linked-list traversal tail — `move.l 4(a0),d0; bne.s loc_3FB20;
/// rts`. In native code the caller iterates the slot Vec, so this is implicit.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_snake_slot_sprite(
    slot: &mut SnakeSlot,
    camera_x: i16,
    camera_y: i16,
    power_timer: i16,
    time_frames: u16,
) -> SnakeSlotSpriteAction {
    // ASM: d3 = $7F - Camera_X_pos; d4 = $7F - Camera_Y_pos
    let d3 = 0x7Fi16.wrapping_sub(camera_x);
    let d4 = 0x7Fi16.wrapping_sub(camera_y);

    // ASM loc_3FB20: tst.w (Diamond_power_timer).w; bge.s loc_3FB3E
    if power_timer < 0 {
        // ASM: move.w (Time_Frames).w,d1; andi.w #$F,d1; bne.s loc_3FB3E
        if time_frames & 0xF == 0 {
            // ASM: subq.w #1,$24(a0); bne.s loc_3FB3E
            slot.diamond_size = slot.diamond_size.wrapping_sub(1);
            if slot.diamond_size == 0 {
                // ASM: bsr.w sub_3F5E8; bra.s loc_3FB88
                return SnakeSlotSpriteAction::Removed;
            }
        }
    }

    // ASM loc_3FB3E: screen position = world pos + camera offset
    let d0 = slot.x_pos.wrapping_add(d3);
    let d1 = slot.y_pos.wrapping_add(d4);

    // ASM: sub.w d6,d0; sub.w d6,d1 — centre by diamond_size
    let d6 = slot.diamond_size as i16;
    let x = d0.wrapping_sub(d6);
    let y = d1.wrapping_sub(d6);

    // ASM: add.w d6,d6; add.w d6,d6 — d6 *= 4 (table index)
    let table_index = (slot.diamond_size as usize).wrapping_mul(4);

    // ASM: d5 = d6 + $1144; move.w d5,$22(a0) — pattern stored in slot
    // (VDP attribute, not used in native rendering pipeline)

    // ASM: bounds check — x in (0, $1E0), y in (0, $1E0)
    // tst.w d0; ble.s loc_3FB88
    if x <= 0 {
        return SnakeSlotSpriteAction::Skipped;
    }
    // cmpi.w #$1E0,d0; bge.s loc_3FB88
    if x >= 0x1E0 {
        return SnakeSlotSpriteAction::Skipped;
    }
    // tst.w d1; ble.s loc_3FB88
    if y <= 0 {
        return SnakeSlotSpriteAction::Skipped;
    }
    // cmpi.w #$1E0,d1; bge.s loc_3FB88
    if y >= 0x1E0 {
        return SnakeSlotSpriteAction::Skipped;
    }

    // ASM: look up sprite attributes from word_4020E
    let entry_index = table_index / 4;
    let (size_byte, pattern) = if entry_index < DIAMOND_SPRITE_ATTR_TABLE.len() {
        DIAMOND_SPRITE_ATTR_TABLE[entry_index]
    } else {
        // Out-of-range sizes use last entry (defensive; ASM doesn't bounds-check)
        *DIAMOND_SPRITE_ATTR_TABLE.last().unwrap()
    };

    SnakeSlotSpriteAction::Visible {
        y: y as u16,
        size_byte,
        pattern,
        x: x as u16,
    }
}

/// State of a single ExtraHitPoint/ExtraLife diamond-power object slot.
/// ASM fields: $3E(a0)=radius (longword), $42(a0)=angle (longword),
/// $24(a0)=power_type (word: 8=ExtraHP, 7=ExtraLife), $22(a0)=unused (word).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct ExtraHPLifeSlot {
    /// ASM $3E(a0): radius in 16.16 fixed-point (longword). Initial $400000.
    pub(crate) radius: u32,
    /// ASM $42(a0): angle (longword). Upper word is the 8-bit angle for sub_40236.
    pub(crate) angle: u32,
    /// ASM $24(a0): power type — 8 = ExtraHitPoint, 7 = ExtraLife.
    pub(crate) power_type: u16,
}

/// Result of `diamond_power_init_extra_hp_life`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct ExtraHPLifeInitResult {
    pub(crate) slots: Vec<ExtraHPLifeSlot>,
}

/// ASM DiamondPower_Init_ExtraHitPoint (loc_3FB90) / DiamondPower_Init_ExtraLife (loc_3FB94).
///
/// loc_3FB90: `moveq #8,d2; bra.s loc_3FB96` — ExtraHitPoint sets type=8.
/// loc_3FB94: `moveq #7,d2` — ExtraLife sets type=7, falls through.
///
/// loc_3FB96 (kid.asm:44904): common entry — `moveq #0,d3; moveq #9,d1`.
///   d3=0 (initial angle), d1=9 (loop counter → 10 slots).
///
/// loc_3FB9A (kid.asm:44908): allocation loop —
///   `bsr.w Allocate_DiamondPowerObjectSlot`
///   `move.l #$400000,$3E(a0)` — radius = $400000
///   `move.l d3,$42(a0)` — angle = d3
///   `addi.l #$199999,d3` — advance angle by $199999 (~1/10 of $1000000)
///   `move.w d2,$24(a0)` — power type
///   `move.w #0,$22(a0)` — clear field $22
///   `dbf d1,loc_3FB9A` — loop 10 times
///   `clr.w (Diamond_power_timer).w` — reset timer to 0
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_init_extra_hp_life(is_extra_life: bool) -> ExtraHPLifeInitResult {
    // ASM loc_3FB90/loc_3FB94: d2 = 8 (ExtraHP) or 7 (ExtraLife)
    let d2: u16 = if is_extra_life { 7 } else { 8 };

    // ASM loc_3FB96: moveq #0,d3; moveq #9,d1
    let mut d3: u32 = 0;
    let count = 10u16; // d1=9 → dbf loops 10 times

    let mut slots = Vec::with_capacity(count as usize);

    // ASM loc_3FB9A: allocation loop
    for _ in 0..count {
        slots.push(ExtraHPLifeSlot {
            radius: 0x0040_0000, // move.l #$400000,$3E(a0)
            angle: d3,           // move.l d3,$42(a0)
            power_type: d2,      // move.w d2,$24(a0)
        });
        d3 = d3.wrapping_add(0x0019_9999); // addi.l #$199999,d3
    }
    // ASM: clr.w (Diamond_power_timer).w — caller is responsible for clearing timer

    ExtraHPLifeInitResult { slots }
}

/// Single slot from Invulnerability / SamuraiHaze init.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct InvulnHazeSlot {
    pub(crate) radius: u16,         // $3E(a0) — byte from init data
    pub(crate) radius_frac: u16,    // $40(a0) — always 0
    pub(crate) y_offset: i16,       // $46(a0) — sign-extended byte
    pub(crate) phase: u16,          // $42(a0) — byte from init data
    pub(crate) rotation_speed: u16, // $48(a0) — byte from init data
}

/// Result of `diamond_power_init_invuln_haze`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct InvulnHazeInitResult {
    pub(crate) slots: Vec<InvulnHazeSlot>,
    pub(crate) diamond_power_timer: u16,
}

/// ASM DiamondPower_Init_Invulnerability (loc_3FC78) /
///     DiamondPower_Init_SamuraiHaze (loc_3FC82).
///
/// loc_3FC78: `st (KidIsInvulnerable).w; lea DiamondPower_InitData_Invulnerability(pc),a1; bra.s loc_3FC8A`
/// loc_3FC82: `st (SamuraiHazeActive).w; lea DiamondPower_InitData_SamuraiHaze(pc),a1`
///
/// loc_3FC8A (kid.asm:45006): common entry — `moveq #9,d1` (10 slots via dbf).
///
/// loc_3FC8C (kid.asm:45010): allocation loop —
///   `bsr.w Allocate_DiamondPowerObjectSlot`
///   `moveq #0,d3; move.b (a1)+,d3; move.w d3,$3E(a0)` — radius from table
///   `clr.w $40(a0)` — radius fraction = 0
///   `move.b (a1)+,d3; ext.w d3; move.w d3,$46(a0)` — y_offset (sign-extended byte)
///   `moveq #0,d3; move.b (a1)+,d3; move.w d3,$42(a0)` — phase from table
///   `move.b (a1)+,d3; move.w d3,$48(a0)` — rotation speed from table
///   `move.w #0,$22(a0)` — clear field $22
///   `dbf d1,loc_3FC8C` — loop 10 times
///   `move.w #$1E0,(Diamond_power_timer).w` — timer = 480
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_init_invuln_haze(is_invulnerability: bool) -> InvulnHazeInitResult {
    // ASM DiamondPower_InitData_Invulnerability (kid.asm:45043) /
    //     DiamondPower_InitData_SamuraiHaze (kid.asm:45030):
    // 10 entries × 4 bytes: (radius, y_offset, phase, rotation_speed)
    #[rustfmt::skip]
    const SAMURAI_HAZE_DATA: [[i8; 4]; 10] = [
        [0x0C, -0x24, 0x00, 0x02],
        [0x0C, -0x24, -0x80i8, 0x02], // $80 as signed = -128
        [0x0C, 0x04,  -0x80i8, 0x02],
        [0x0C, 0x04,  0x00, 0x02],
        [0x15, -0x1A, 0x20, 0x02],
        [0x15, -0x1A, -0x60i8, 0x02], // $A0 = -96
        [0x15, -0x06, 0x60, 0x02],
        [0x15, -0x06, -0x20i8, 0x02], // $E0 = -32
        [0x18, -0x10, 0x40, 0x02],
        [0x18, -0x10, -0x40i8, 0x02], // $C0 = -64
    ];
    #[rustfmt::skip]
    const INVULNERABILITY_DATA: [[i8; 4]; 10] = [
        [0x0C, -0x24, 0x00, 0x06],
        [0x0C, -0x24, -0x80i8, 0x06],
        [0x0C, 0x04,  0x40, 0x06],
        [0x0C, 0x04,  -0x40i8, 0x06], // $C0 = -64
        [0x15, -0x1A, 0x10, 0x06],
        [0x15, -0x1A, -0x70i8, 0x06], // $90 = -112
        [0x15, -0x06, 0x30, 0x06],
        [0x15, -0x06, -0x50i8, 0x06], // $B0 = -80
        [0x18, -0x10, 0x20, 0x06],
        [0x18, -0x10, -0x60i8, 0x06], // $A0 = -96
    ];

    let data = if is_invulnerability {
        &INVULNERABILITY_DATA
    } else {
        &SAMURAI_HAZE_DATA
    };

    // ASM loc_3FC8A: moveq #9,d1 → 10 slots
    let mut slots = Vec::with_capacity(10);

    // ASM loc_3FC8C: allocation loop
    for entry in data {
        // moveq #0,d3; move.b (a1)+,d3 → unsigned byte for radius
        let radius = entry[0] as u8 as u16;
        // move.b (a1)+,d3; ext.w d3 → sign-extended byte for y_offset
        let y_offset = entry[1] as i16;
        // moveq #0,d3; move.b (a1)+,d3 → unsigned byte for phase
        let phase = entry[2] as u8 as u16;
        // move.b (a1)+,d3 → note: d3 still has prior upper byte from ext.w,
        // but move.w d3,$48(a0) writes only the low word. The byte is loaded
        // with moveq #0,d3 clearing first... actually: the ASM does NOT do
        // moveq #0 before the 4th byte — it does `move.b (a1)+,d3` then
        // `move.w d3,$48(a0)`. But d3 was cleared by the prior `moveq #0,d3`
        // at the phase step and only the low byte is overwritten by move.b,
        // so the upper byte of d3.w is 0. Result: unsigned byte.
        let rotation_speed = entry[3] as u8 as u16;

        slots.push(InvulnHazeSlot {
            radius,
            radius_frac: 0, // clr.w $40(a0)
            y_offset,
            phase,
            rotation_speed,
        });
    }

    // ASM: move.w #$1E0,(Diamond_power_timer).w
    InvulnHazeInitResult {
        slots,
        diamond_power_timer: 0x1E0, // 480
    }
}

/// Per-slot update action from `diamond_power_main_extra_hp_life_slot`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum ExtraHPLifeSlotAction {
    /// Slot produced a visible sprite. ASM loc_3FC56: sprite compiled into buffer.
    Visible {
        y: u16,
        size_byte: u8,
        pattern: u16,
        x: u16,
    },
    /// Slot converged (radius < 0): award HP or life. ASM bra.w sub_3F63C.
    Converged {
        /// True if ExtraHitPoint was awarded (type=8, cap not reached).
        awarded_hp: bool,
        /// True if ExtraLife was awarded (type=7).
        awarded_life: bool,
    },
}

/// ASM DiamondPower_Main_ExtraHPLife (kid.asm:44920).
///
/// Caller sets up:
///   `addq.w #1,(Diamond_power_timer).w` — caller increments timer
///   d3 = $7F - Camera_X_pos, d4 = $7F - Camera_Y_pos
///   a3 = Addr_GfxObject_Kid
///
/// loc_3FBE4 (kid.asm:44932): per-object loop body —
///   `move.l d0,a0` — a0 = slot
///   `move.l $3E(a0),d5; lsr.l #8,d5; lsr.l #1,d5` — d5 = radius >> 9
///   `move.w $42(a0),d0; bsr.w sub_40236` — (cos, sin) from angle
///   `muls.w d5,d0; swap d0; asr.w #5,d0` — x_offset = cos * d5 >> 21
///   `add.w x_pos(a3),d0; add.w d3,d0` — screen x
///   `muls.w d5,d1; swap d1; asr.w #5,d1; neg.w d1` — y_offset
///   `add.w y_pos(a3),d1; subi.w #$10,d1; add.w d4,d1` — screen y
///   Timer < $40: `addq.w #2,$3E(a0); subq.w #2,$42(a0)` (expanding)
///   Timer >= $40 (loc_3FC2A): `addq.w #1,$42(a0); subq.w #6,$3E(a0)`
///     if radius < 0: award HP/life, bra.w sub_3F63C (remove)
///   loc_3FC56: compile sprite from word_4020E
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_main_extra_hp_life_slot(
    slot: &mut ExtraHPLifeSlot,
    kid_x: i16,
    kid_y: i16,
    camera_x: i16,
    camera_y: i16,
    diamond_power_timer: u16,
    extra_hitpoint_slots: u16,
) -> ExtraHPLifeSlotAction {
    use crate::boot::sin_cos_sub_40236;

    // ASM: d3 = $7F - Camera_X_pos; d4 = $7F - Camera_Y_pos
    let d3 = 0x7Fi16.wrapping_sub(camera_x);
    let d4 = 0x7Fi16.wrapping_sub(camera_y);

    // ASM loc_3FBE4: move.l $3E(a0),d5; lsr.l #8,d5; lsr.l #1,d5
    let d5 = (slot.radius >> 9) as u16;

    // ASM: move.w $42(a0),d0 — upper word of angle longword
    let angle = (slot.angle >> 16) as u8;
    let (cos, sin) = sin_cos_sub_40236(angle);

    // ASM: muls.w d5,d0; swap d0; asr.w #5,d0; add.w x_pos(a3),d0; add.w d3,d0
    let x_offset = ((cos as i32 * d5 as i32) >> 16) as i16 >> 5;
    let screen_x = kid_x.wrapping_add(x_offset).wrapping_add(d3);

    // ASM: muls.w d5,d1; swap d1; asr.w #5,d1; neg.w d1;
    //      add.w y_pos(a3),d1; subi.w #$10,d1; add.w d4,d1
    let y_offset = -(((sin as i32 * d5 as i32) >> 16) as i16 >> 5);
    let screen_y = kid_y
        .wrapping_add(y_offset)
        .wrapping_sub(0x10)
        .wrapping_add(d4);

    // ASM: move.w $24(a0),d6 — sprite index
    let d6 = slot.power_type;

    // ASM: cmpi.w #$40,(Diamond_power_timer).w; bge.s loc_3FC2A
    if diamond_power_timer < 0x40 {
        // Phase 1: expanding — addq.w #2,$3E(a0); subq.w #2,$42(a0)
        let radius_hi = (slot.radius >> 16) as u16;
        slot.radius = ((radius_hi.wrapping_add(2) as u32) << 16) | (slot.radius & 0xFFFF);
        let angle_hi = (slot.angle >> 16) as u16;
        slot.angle = ((angle_hi.wrapping_sub(2) as u32) << 16) | (slot.angle & 0xFFFF);
    } else {
        // Phase 2: contracting (loc_3FC2A) — addq.w #1,$42(a0); subq.w #6,$3E(a0)
        let angle_hi = (slot.angle >> 16) as u16;
        slot.angle = ((angle_hi.wrapping_add(1) as u32) << 16) | (slot.angle & 0xFFFF);
        let radius_hi = (slot.radius >> 16) as u16;
        let new_radius_hi = radius_hi.wrapping_sub(6);
        slot.radius = ((new_radius_hi as u32) << 16) | (slot.radius & 0xFFFF);
        // ASM: bge.s loc_3FC56 — if radius_hi >= 0 (unsigned sub didn't go negative)
        if (new_radius_hi as i16) < 0 {
            // Slot converged: award HP or life, then cleanup.
            // ASM loc_3FC4E: type=7 → addq.w #1,(Number_Lives).w
            // ASM loc_3FC4C/loc_3FC52: bra.w sub_3F63C (dealloc slot).
            // ASM: cmpi.w #7,$24(a0); beq.s loc_3FC4E
            if slot.power_type == 7 {
                // ExtraLife: addq.w #1,(Number_Lives).w
                return ExtraHPLifeSlotAction::Converged {
                    awarded_hp: false,
                    awarded_life: true,
                };
            }
            // ExtraHitPoint: cmpi.w #4,(Extra_hitpoint_slots).w; bge.s loc_3FC4C
            let awarded_hp = extra_hitpoint_slots < 4;
            return ExtraHPLifeSlotAction::Converged {
                awarded_hp,
                awarded_life: false,
            };
        }
    }

    // ASM loc_3FC56: compile sprite — add.w d6,d6; add.w d6,d6; lea word_4020E(pc),a1
    let entry_index = d6 as usize;
    let (size_byte, pattern) = if entry_index < DIAMOND_SPRITE_ATTR_TABLE.len() {
        DIAMOND_SPRITE_ATTR_TABLE[entry_index]
    } else {
        *DIAMOND_SPRITE_ATTR_TABLE.last().unwrap()
    };

    ExtraHPLifeSlotAction::Visible {
        y: screen_y as u16,
        size_byte,
        pattern,
        x: screen_x as u16,
    }
}

/// Result of the per-frame timer check in `DiamondPower_Main_InvulnAndHaze`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum InvulnHazeFrameAction {
    /// Timer < -$1B → call sub_3F63C (dealloc all slots). ASM `bsr.w sub_3F63C; rts`.
    Expired,
    /// Timer >= -$1B → continue with per-slot processing. `fade` is the shrink
    /// parameter passed to each slot (d7 after neg/clamp/lsr).
    Active { fade: u16 },
}

/// ASM DiamondPower_Main_InvulnAndHaze (kid.asm:45056) frame-level timer logic.
///
/// Decrements `Diamond_power_timer`, then:
///
/// - If timer < -$1B: all slots expired → return `Expired` (caller deallocs).
///
/// - loc_3FD28 (kid.asm:45065): `neg.w d7` — negate the timer value.
///   `bpl.s loc_3FD2E` — if result >= 0, keep it; otherwise clamp to 0.
///
/// - loc_3FD2E (kid.asm:45070): `lsr.w #2,d7` — divide by 4 to get fade amount.
///   The fade is 0 while timer is positive (diamonds are fully visible and
///   rotating); once timer goes negative the fade grows, shrinking sprites
///   until the power expires.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_invuln_haze_frame(timer: &mut i16) -> InvulnHazeFrameAction {
    // ASM: subq.w #1,(Diamond_power_timer).w
    *timer = timer.wrapping_sub(1);
    let d7 = *timer;

    // ASM: cmpi.w #-$1B,d7; bge.s loc_3FD28
    if d7 < -0x1B {
        return InvulnHazeFrameAction::Expired;
    }

    // ASM loc_3FD28: neg.w d7; bpl.s loc_3FD2E; moveq #0,d7
    let d7 = (-(d7 as i32)) as i16;
    let d7 = if d7 < 0 { 0u16 } else { d7 as u16 };

    // ASM loc_3FD2E: lsr.w #2,d7
    let fade = d7 >> 2;

    InvulnHazeFrameAction::Active { fade }
}

/// Per-slot sprite result from `diamond_power_invuln_haze_slot_sprite`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum InvulnHazeSpriteAction {
    /// Small sprite (table index < $C / entry < 3): write to a4 buffer.
    /// 4 words: Y, size_link word, pattern word, X.
    SmallSprite {
        y: u16,
        size_link: u16,
        pattern: u16,
        x: u16,
    },
    /// Large sprite (table index >= $C / entry >= 3): write to a2 buffer.
    /// Format: Y word, size byte, pattern word, X word (link managed by caller).
    LargeSprite {
        y: u16,
        size_byte: u8,
        pattern: u16,
        x: u16,
    },
    /// Sprite invisible (size - fade < 0). ASM: `sub.w d7,d6; bmi.s loc_3FDD2`.
    Skipped,
}

/// Invuln/Haze size lookup table.
/// ASM byte_3FDDC (kid.asm:45148): 12 entries indexed by the clamped
/// `(sin_offset + $18) >> 2` value (0–11). Maps depth-based sin offset
/// to sprite half-size.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const INVULN_HAZE_SIZE_TABLE: [u8; 12] = [1, 1, 2, 2, 3, 3, 3, 3, 4, 4, 5, 5];

/// ASM loc_3FD4C through loc_3FDD2 (kid.asm:45082–45146): per-slot sprite
/// compilation for DiamondPower_Main_InvulnAndHaze.
///
/// loc_3FD4C (kid.asm:45082): per-slot entry —
///   `move.l d0,a0` — a0 = slot pointer.
///   `tst.w d7; bne.s loc_3FD5A` — when fade == 0 (timer still positive),
///   update angle: `$42(a0) += $48(a0)` (phase += rotation_speed).
///
///   Polar-to-screen via sub_40236:
///   - d5 = `move.l $3E(a0)` >> 9 (radius with fraction, shifted).
///   - cos from sub_40236 → x_offset = `(cos * d5) >> 21 + kid_x + ($7F - cam_x)`.
///   - sin from sub_40236 → size index = `((sin * d5) >> 21 + $18)`, clamped [0, $B].
///   - d6 = byte_3FDDC[size_index] - fade; if < 0 → Skipped.
///   - Screen Y from slot y_offset ($46) + kid_y + ($7F - cam_y).
///   - Centre by subtracting d6 from both x and y.
///   - d6 * 4 → index into word_4020E:
///     < $C → SmallSprite (a4 buffer); >= $C → LargeSprite (a2 buffer).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diamond_power_invuln_haze_slot_sprite(
    slot: &mut InvulnHazeSlot,
    fade: u16,
    kid_x: i16,
    kid_y: i16,
    camera_x: i16,
    camera_y: i16,
) -> InvulnHazeSpriteAction {
    use crate::boot::sin_cos_sub_40236;

    // ASM loc_3FD2E: d3 = $7F - Camera_X_pos; d4 = $7F - Camera_Y_pos
    let d3 = 0x7Fi16.wrapping_sub(camera_x);
    let d4 = 0x7Fi16.wrapping_sub(camera_y);

    // ASM loc_3FD4C: tst.w d7; bne.s loc_3FD5A — only update angle when fade == 0
    if fade == 0 {
        // ASM: move.w $48(a0),d1; add.w d1,$42(a0)
        slot.phase = slot.phase.wrapping_add(slot.rotation_speed);
    }

    // ASM loc_3FD5A: move.l $3E(a0),d5; lsr.l #8,d5; lsr.l #1,d5
    // $3E is radius (word), $40 is radius_frac (word) — longword read combines them.
    let radius_long = ((slot.radius as u32) << 16) | (slot.radius_frac as u32);
    let d5 = (radius_long >> 9) as u16;

    // ASM: move.w $42(a0),d0; bsr.w sub_40236
    let angle = slot.phase as u8;
    let (cos, sin) = sin_cos_sub_40236(angle);

    // ASM: muls.w d5,d0; swap d0; asr.w #5,d0; add.w x_pos(a3),d0; add.w d3,d0
    let x_offset = ((cos as i32 * d5 as i32) >> 16) as i16 >> 5;
    let mut screen_x = kid_x.wrapping_add(x_offset).wrapping_add(d3);

    // ASM: muls.w d5,d1; swap d1; asr.w #5,d1; addi.w #$18,d1
    let sin_offset = ((sin as i32 * d5 as i32) >> 16) as i16 >> 5;
    let mut size_index = sin_offset.wrapping_add(0x18);

    // ASM: bpl.s loc_3FD84; moveq #0,d1
    if size_index < 0 {
        size_index = 0;
    }

    // ASM loc_3FD84: lsr.w #2,d1; cmpi.w #$B,d1; ble.s loc_3FD8E; moveq #$B,d1
    let mut size_index = (size_index as u16) >> 2;
    if size_index > 0xB {
        size_index = 0xB;
    }

    // ASM loc_3FD8E: moveq #0,d6; move.b byte_3FDDC(pc,d1.w),d6; sub.w d7,d6
    let raw_size = INVULN_HAZE_SIZE_TABLE[size_index as usize] as i16;
    let d6 = raw_size - fade as i16;

    // ASM: bmi.s loc_3FDD2 — invisible if size - fade < 0
    if d6 < 0 {
        return InvulnHazeSpriteAction::Skipped;
    }
    let d6 = d6 as u16;

    // ASM: move.w $46(a0),d1; add.w y_pos(a3),d1; add.w d4,d1
    let mut screen_y = slot.y_offset.wrapping_add(kid_y).wrapping_add(d4);

    // ASM: sub.w d6,d0; sub.w d6,d1 — centre sprite by size
    screen_x = screen_x.wrapping_sub(d6 as i16);
    screen_y = screen_y.wrapping_sub(d6 as i16);

    // ASM: add.w d6,d6; add.w d6,d6 — d6 *= 4 (table byte index)
    let table_index = (d6 as usize) * 4;

    // ASM: lea word_4020E(pc),a1; cmpi.w #$C,d6
    let entry_index = table_index / 4;
    let (size_byte, pattern) = if entry_index < DIAMOND_SPRITE_ATTR_TABLE.len() {
        DIAMOND_SPRITE_ATTR_TABLE[entry_index]
    } else {
        *DIAMOND_SPRITE_ATTR_TABLE.last().unwrap()
    };

    if table_index < 0xC {
        // ASM: small sprite path → a4 buffer
        // move.w (a1,d6.w),(a4)+ — full size_link word
        let size_link = (size_byte as u16) << 8; // low byte is link, filled by caller
        InvulnHazeSpriteAction::SmallSprite {
            y: screen_y as u16,
            size_link,
            pattern,
            x: screen_x as u16,
        }
    } else {
        // ASM loc_3FDC2: large sprite path → a2 buffer
        InvulnHazeSpriteAction::LargeSprite {
            y: screen_y as u16,
            size_byte,
            pattern,
            x: screen_x as u16,
        }
    }
}

impl NativeRuntime {
    #[allow(non_snake_case)]
    pub(crate) fn clear_special_movement_state_sub_DB22(&mut self) {
        // ASM sub_DB22 (kid.asm:17170): grounded/default-pose recovery clears
        // the per-form transient action flags: Cyclone flight, Skycutter's
        // flipboard/on-board latch, Red Stealth's downward slash, Maniaxe's
        // thrown-axe state, and Berzerker / Iron Knight special motion.
        self.helmet.cyclone_flying = false;
        self.helmet.skycutter_on_board = false;
        self.helmet.skycutter_flipboard = 0;
        self.helmet.red_stealth_sword_timer = 0;
        self.helmet.maniaxe_axe_active = false;
        self.helmet.berzerker_charging = false;
        self.helmet.berzerker_charge_accum = 0;
        self.helmet.iron_knight_climbing = false;
        self.helmet.iron_knight_climb_accel = 0;
        self.helmet.iron_knight_climb_pose_accum = 0;
    }

    #[allow(non_snake_case)]
    pub(crate) fn refresh_grounded_helmet_state_sub_78E8(&mut self) {
        // ASM sub_78E8 (kid.asm:7973): grounded frames reset the default
        // helmet pose unless the form-specific grounded action is still live.
        let previous_maniaxe_axe_active = self.helmet.maniaxe_axe_active;
        let previous_red_stealth_sword_timer = self.helmet.red_stealth_sword_timer;
        self.clear_special_movement_state_sub_DB22();
        match self.progression.current_form {
            PlayerForm::Maniaxe => {
                if previous_maniaxe_axe_active
                    && self.projectiles.iter().any(|projectile| {
                        projectile.friendly && projectile.lifetime > 0 && projectile.gravity == 0
                    })
                {
                    self.helmet.maniaxe_axe_active = true;
                }
            }
            PlayerForm::RedStealth => {
                if previous_red_stealth_sword_timer > 0 {
                    self.helmet.red_stealth_sword_timer = previous_red_stealth_sword_timer;
                }
            }
            PlayerForm::Eyeclops => {
                self.helmet.eyeclops_beam_active = self.projectiles.iter().any(|projectile| {
                    projectile.friendly && projectile.lifetime > 0 && projectile.gravity == 0
                });
                if !self.helmet.eyeclops_beam_active {
                    self.helmet.eyeclops_beam_counter = 0;
                    self.helmet.eyeclops_beam_hard = false;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn step_skycutter_flipboard(
        &mut self,
        slice: &mut OpeningSliceState,
        input: FrameInput,
    ) {
        if self.progression.current_form != PlayerForm::Skycutter {
            self.helmet.skycutter_flipboard = 0;
            self.helmet.skycutter_on_board = false;
            return;
        }

        // C button rising edge → deploy/retract flipboard
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && self.helmet.skycutter_flipboard == 0 {
            self.helmet.skycutter_flipboard = 1;
        }

        // Step flipboard counter
        if self.helmet.skycutter_flipboard > 0 {
            let next = self
                .helmet
                .skycutter_flipboard
                .saturating_add(SKYCUTTER_COUNTER_STEP);
            if next >= SKYCUTTER_FLIPBOARD_TIMEOUT {
                // Animation complete: retract
                self.helmet.skycutter_flipboard = 0;
            } else {
                // Mount toggle at midpoint
                if self.helmet.skycutter_flipboard < SKYCUTTER_MOUNT_FRAME
                    && next >= SKYCUTTER_MOUNT_FRAME
                {
                    // ASM sub_98F2 / loc_991A: midpoint `$11` toggles both
                    // `$FFFFFA6A` and `x_direction(a3)`, so the mounted board
                    // reverses Kid's facing on the same frame it flips the
                    // board latch.
                    self.helmet.skycutter_on_board = !self.helmet.skycutter_on_board;
                    slice.player.facing = match slice.player.facing {
                        Direction::Right => Direction::Left,
                        Direction::Left => Direction::Right,
                    };
                }
                self.helmet.skycutter_flipboard = next;
            }
        }

        // While on board: override gravity with Skycutter flight physics
        if self.helmet.skycutter_on_board && !slice.player.on_ground {
            let gravity = if input.b {
                SKYCUTTER_GRAVITY_B_HELD
            } else {
                SKYCUTTER_GRAVITY_NO_B
            };
            // In the original, the gravity is subtracted (y_vel is positive=down,
            // and the board makes you fall more slowly). But looking at the ASM:
            // sub.l d6,d7 where d7=y_vel and d6=gravity. Since positive y_vel=down
            // in the original game and we use negative y_vel=up, this adds gravity
            // (descent) rather than subtracting.
            slice.player.y_velocity += gravity;
            // Clamp max upward speed
            if slice.player.y_velocity < SKYCUTTER_MAX_UPWARD_VEL {
                slice.player.y_velocity = SKYCUTTER_MAX_UPWARD_VEL;
            }

            // B button press while on board: upward kick
            let b_pressed = input.b && !self.prior_input.b;
            if b_pressed {
                let kick = -(slice.player.x_velocity.unsigned_abs() as i32 / 2 + 0x3_0000);
                slice.player.y_velocity = kick;
            }
        }

        // Landing while on board: bounce upward
        if self.helmet.skycutter_on_board && slice.player.on_ground {
            slice.player.y_velocity = SKYCUTTER_LANDING_BOUNCE;
            slice.player.on_ground = false;
            self.helmet.skycutter_flipboard = 1; // restart deploy animation
        }
    }

    /// Step Cyclone vertical flight: acceleration-based with C-button thrust.
    ///
    /// When flying, an acceleration accumulator drifts downward by $400/frame
    /// (gravity). Holding C subtracts $3000 (thrust). The accumulator is added
    /// to y_velocity each frame, with velocity clamped to [-$30000, $28000].
    /// Landing clears the flying flag.
    /// ASM sub_B168 (loc_B214): when Cyclone_flying is already set, replace
    /// normal gravity with the flight accumulator. The thrust check uses
    /// `bclr #Button_C,(Ctrl_Pressed)` — a rising-edge test that consumes the
    /// C press bit so kid_flip won't see it. Native equivalent: check
    /// `previous_input` for 1-frame edge detection (not `prior_input` which
    /// has a 2-frame gap).
    pub(crate) fn step_cyclone_flight(&mut self, slice: &mut OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Cyclone {
            self.helmet.cyclone_flying = false;
            self.helmet.cyclone_y_accel = 0;
            return;
        }

        // Landing clears flight
        if self.helmet.cyclone_flying && slice.player.on_ground {
            self.helmet.cyclone_flying = false;
            return;
        }

        if !self.helmet.cyclone_flying {
            return;
        }

        // ASM uses `addi.w` which wraps on M68K 16-bit word arithmetic.
        // Must use wrapping_add, not saturating_add — the wrap behaviour is
        // load-bearing when the accumulator gets very negative.
        self.helmet.cyclone_y_accel = self
            .helmet
            .cyclone_y_accel
            .wrapping_add(CYCLONE_ACCEL_GRAVITY);

        // ASM: `bclr #Button_C,(Ctrl_Pressed)` — thrust on C rising edge only,
        // not while held. Uses previous_input for proper 1-frame edge detection
        // (prior_input is 2 frames back due to gameplay_input asymmetry).
        let c_newly_pressed = input.c && !self.previous_input.c;
        if c_newly_pressed {
            // ASM `subi.w` wraps: e.g. 0xAA00 - 0x3000 = 0x7A00 (negative →
            // positive wrap). This is intentional — the wrap produces a sudden
            // downward push that's part of the flight feel.
            self.helmet.cyclone_y_accel = self
                .helmet
                .cyclone_y_accel
                .wrapping_sub(CYCLONE_ACCEL_THRUST);
            if self.helmet.cyclone_y_accel < CYCLONE_ACCEL_MIN {
                self.helmet.cyclone_y_accel = CYCLONE_ACCEL_MIN;
            }
        } else {
            // Not thrusting: cap descent acceleration
            if self.helmet.cyclone_y_accel > CYCLONE_ACCEL_MAX_DESCENT {
                self.helmet.cyclone_y_accel = CYCLONE_ACCEL_MAX_DESCENT;
            }
        }

        // Apply acceleration to velocity
        slice.player.y_velocity += i32::from(self.helmet.cyclone_y_accel);

        // Clamp velocity to rise/fall limits.
        slice.player.y_velocity = slice
            .player
            .y_velocity
            .clamp(CYCLONE_MAX_RISE_VEL, CYCLONE_MAX_FALL_VEL);
    }

    /// ASM kid_flip (loc_ACB0): C rising edge activates flight when airborne.
    /// Runs AFTER physics, so flight takes effect on the next frame. Uses
    /// `bclr #Button_C,(Ctrl_Pressed)` which is consumed by sub_B168 when
    /// already flying — so only activates when cyclone_flying is false.
    pub(crate) fn step_cyclone_activation(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Cyclone {
            return;
        }
        // Only activate when not already flying (ASM: sub_B168 consumes C
        // pressed bit via bclr when flying, so kid_flip never sees it).
        if self.helmet.cyclone_flying {
            return;
        }
        let c_pressed = input.c && !self.previous_input.c;
        if c_pressed && !slice.player.on_ground {
            self.helmet.cyclone_flying = true;
            self.helmet.cyclone_y_accel = CYCLONE_ACCEL_INITIAL;
        }
    }

    /// IronKnight wall climb: the ROM accepts the attach from the grounded
    /// walking path as well as from airborne wall contact. C presses while
    /// climbing boost upward, natural drift slides down.
    pub(crate) fn step_iron_knight_climb(
        &mut self,
        slice: &mut OpeningSliceState,
        input: FrameInput,
    ) -> IronKnightClimbStep {
        if self.progression.current_form != PlayerForm::IronKnight {
            self.helmet.iron_knight_climbing = false;
            self.helmet.iron_knight_climb_accel = 0;
            self.helmet.iron_knight_climb_pose_accum = 0;
            return IronKnightClimbStep::None;
        }

        // ASM `Get_RandomNumber_wordC` consumes `Ctrl_Pressed`, so climb boost
        // only triggers on the immediate rising edge, not two frames later.
        let c_pressed = input.c && !self.previous_input.c;

        // ROM walking state `loc_7664` and jump state `kid_flip` both allow a
        // raw C edge to enter wall climbing when Iron Knight is pressed against
        // a wall in the facing direction.
        if c_pressed
            && !self.helmet.iron_knight_climbing
            && let Some(ref grid) = self.block_grid
        {
            let attach_from_grounded_walk = slice.player.on_ground;
            let facing_right = slice.player.facing == Direction::Right;
            let px = (slice.player.x_pos >> 16) as i32;
            let py = (slice.player.y_pos >> 16) as i32;
            let probe = if attach_from_grounded_walk {
                // ROM walking-state `loc_7664` uses `sub_922C`: a single edge
                // column at half_width + 1 through the form's full height.
                terrain::probe_wall_column(
                    grid,
                    px,
                    py,
                    self.progression.current_form.half_width(),
                    PLAYER_HEIGHT,
                    facing_right,
                )
            } else {
                terrain::probe_wall(grid, px, py, 5, 31, facing_right)
            };
            if probe.hit {
                self.helmet.iron_knight_climbing = true;
                self.helmet.iron_knight_climb_accel = 0;
                self.helmet.iron_knight_climb_pose_accum = 0;
                slice.player.y_velocity = IRON_KNIGHT_ATTACH_Y_VEL;
                slice.player.x_velocity = 0;
                slice.player.on_ground = false;
                return if attach_from_grounded_walk {
                    IronKnightClimbStep::AttachHibernate
                } else {
                    IronKnightClimbStep::None
                };
            }
        }

        if !self.helmet.iron_knight_climbing {
            return IronKnightClimbStep::None;
        }

        // Landing stops climb
        if slice.player.on_ground {
            self.helmet.iron_knight_climbing = false;
            self.helmet.iron_knight_climb_accel = 0;
            self.helmet.iron_knight_climb_pose_accum = 0;
            return IronKnightClimbStep::None;
        }

        // B press = jump off wall
        let b_pressed = input.b && !self.previous_input.b;
        if b_pressed {
            self.helmet.iron_knight_climbing = false;
            self.helmet.iron_knight_climb_accel = 0;
            self.helmet.iron_knight_climb_pose_accum = 0;
            // Jump away from wall
            let jump_height =
                physics::JUMP_HEIGHTS[usize::from(PlayerForm::IronKnight.helmet_id())];
            let (_, y_vel) = physics::initiate_jump(0, jump_height);
            slice.player.y_velocity = y_vel;
            return IronKnightClimbStep::None;
        }

        let release_direction = match slice.player.facing {
            Direction::Right if input.left => Some(Direction::Left),
            Direction::Left if input.right => Some(Direction::Right),
            _ => None,
        };
        if let Some(direction) = release_direction {
            self.helmet.iron_knight_climbing = false;
            self.helmet.iron_knight_climb_accel = 0;
            self.helmet.iron_knight_climb_pose_accum = 0;
            slice.player.facing = direction;
            return IronKnightClimbStep::ExitHibernate;
        }

        // Check wall still exists
        if let Some(ref grid) = self.block_grid {
            let facing_right = slice.player.facing == Direction::Right;
            let px = (slice.player.x_pos >> 16) as i32;
            let py = (slice.player.y_pos >> 16) as i32;
            let probe = terrain::probe_iron_knight_climb_resume_wall(
                grid,
                px,
                py,
                self.progression.current_form.half_width(),
                facing_right,
            );
            if !probe.hit {
                // Wall gone, fall off
                self.helmet.iron_knight_climbing = false;
                self.helmet.iron_knight_climb_accel = 0;
                self.helmet.iron_knight_climb_pose_accum = 0;
                return IronKnightClimbStep::None;
            }
        }

        // Accumulate drift (sliding down)
        self.helmet.iron_knight_climb_accel = self
            .helmet
            .iron_knight_climb_accel
            .saturating_add(IRON_KNIGHT_SLIDE_DRIFT);

        // C press: boost upward
        if c_pressed {
            self.helmet.iron_knight_climb_accel = self
                .helmet
                .iron_knight_climb_accel
                .saturating_sub(IRON_KNIGHT_CLIMB_BOOST);
            if self.helmet.iron_knight_climb_accel < IRON_KNIGHT_MIN_ACCEL {
                self.helmet.iron_knight_climb_accel = IRON_KNIGHT_MIN_ACCEL;
            }
        } else {
            // Cap slide-down acceleration
            if self.helmet.iron_knight_climb_accel > IRON_KNIGHT_MAX_SLIDE_ACCEL {
                self.helmet.iron_knight_climb_accel = IRON_KNIGHT_MAX_SLIDE_ACCEL;
            }
        }

        // Apply acceleration to velocity
        slice.player.y_velocity += i32::from(self.helmet.iron_knight_climb_accel);

        // Clamp velocity to climb/fall limits.
        slice.player.y_velocity = slice
            .player
            .y_velocity
            .clamp(IRON_KNIGHT_MAX_CLIMB_VEL, IRON_KNIGHT_MAX_FALL_VEL);
        self.helmet.iron_knight_climb_pose_accum = iron_knight_climb_visual_sub_A3BC(
            self.helmet.iron_knight_climb_pose_accum,
            slice.player.y_velocity,
        )
        .pose_accum;

        // Zero horizontal velocity while climbing
        slice.player.x_velocity = 0;
        IronKnightClimbStep::ResumeClimb
    }

    /// Micromax wall/ceiling stick: auto-attach on contact while airborne,
    /// reduced gravity while stuck, B press detaches with kick velocity.
    pub(crate) fn step_micromax_stick(&mut self, slice: &mut OpeningSliceState, input: FrameInput) {
        // Only active for Micromax form
        if self.progression.current_form != PlayerForm::Micromax {
            self.helmet.micromax_stuck = false;
            return;
        }

        // Already stuck: handle slide and detach
        if self.helmet.micromax_stuck {
            let b_pressed = input.b && !self.prior_input.b;
            if b_pressed {
                // Detach: kick upward and away
                self.helmet.micromax_stuck = false;
                slice.player.y_velocity = MICROMAX_DETACH_Y_VEL;
                // Reverse facing direction
                slice.player.facing = match slice.player.facing {
                    Direction::Right => Direction::Left,
                    Direction::Left => Direction::Right,
                };
                let kick_x = match slice.player.facing {
                    Direction::Right => MICROMAX_DETACH_X_SPEED,
                    Direction::Left => -MICROMAX_DETACH_X_SPEED,
                };
                slice.player.x_velocity = kick_x;
                slice.player.on_ground = false;
                return;
            }

            // While stuck: apply reduced gravity (slowly peels off)
            slice.player.y_velocity += MICROMAX_STUCK_GRAVITY;
            if slice.player.y_velocity > MICROMAX_STUCK_MAX_FALL {
                // Peeled off
                self.helmet.micromax_stuck = false;
                return;
            }

            // Zero horizontal velocity while stuck (player can't walk)
            slice.player.x_velocity = 0;

            // Check surface still exists
            if let Some(ref grid) = self.block_grid {
                let px = (slice.player.x_pos >> 16) as i32;
                let py = (slice.player.y_pos >> 16) as i32;
                let ceiling = terrain::probe_ceiling(grid, px, py, MICROMAX_HALF_W, MICROMAX_H);
                let wall_r = terrain::probe_wall(grid, px, py, MICROMAX_HALF_W, MICROMAX_H, true);
                let wall_l = terrain::probe_wall(grid, px, py, MICROMAX_HALF_W, MICROMAX_H, false);
                if !ceiling.hit && !wall_r.hit && !wall_l.hit {
                    self.helmet.micromax_stuck = false;
                }
            }
            return;
        }

        // Not stuck: try to attach while airborne
        if slice.player.on_ground {
            return;
        }
        // Only stick when falling or neutral (y_vel >= 0) per ASM
        if slice.player.y_velocity < 0 {
            return;
        }

        if let Some(ref grid) = self.block_grid {
            let px = (slice.player.x_pos >> 16) as i32;
            let py = (slice.player.y_pos >> 16) as i32;

            // Check ceiling contact
            let ceiling = terrain::probe_ceiling(grid, px, py, MICROMAX_HALF_W, MICROMAX_H);
            if ceiling.hit {
                self.helmet.micromax_stuck = true;
                slice.player.y_velocity = 0;
                slice.player.x_velocity = 0;
                return;
            }

            // Check wall contact in facing direction
            let facing_right = slice.player.facing == Direction::Right;
            let wall = terrain::probe_wall(grid, px, py, MICROMAX_HALF_W, MICROMAX_H, facing_right);
            if wall.hit {
                self.helmet.micromax_stuck = true;
                slice.player.y_velocity = 0;
                slice.player.x_velocity = 0;
            }
        }
    }

    /// Step RedStealth sword swing: C press starts swing, hitbox active after startup.
    pub(crate) fn step_red_stealth_sword(&mut self, input: FrameInput) {
        if self.progression.current_form != PlayerForm::RedStealth {
            self.helmet.red_stealth_sword_timer = 0;
            return;
        }

        // C rising edge starts a new swing (only if not already swinging)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && self.helmet.red_stealth_sword_timer == 0 {
            self.helmet.red_stealth_sword_timer =
                RED_STEALTH_STARTUP_FRAMES + RED_STEALTH_ACTIVE_FRAMES;
        }

        // Tick down active timer
        if self.helmet.red_stealth_sword_timer > 0 {
            self.helmet.red_stealth_sword_timer -= 1;
        }
    }

    /// Returns true if the sword hitbox is active (past startup, within active frames).
    pub(crate) fn sword_is_active(&self) -> bool {
        self.helmet.red_stealth_sword_timer > 0
            && self.helmet.red_stealth_sword_timer <= RED_STEALTH_ACTIVE_FRAMES
    }

    /// Check sword hitbox against active enemies during active frames.
    /// Damages hit enemies and removes dead ones.
    pub(crate) fn process_sword_hits(&mut self, slice: &OpeningSliceState) {
        if !self.sword_is_active() {
            return;
        }

        let (sword_x_fp, sword_y_fp) = red_stealth_swing_anchor_sub_868E(&slice.player);
        let sword_x = (sword_x_fp >> 16) as i32;
        let sword_y = (sword_y_fp >> 16) as i32;

        let mut kill_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter_mut().enumerate() {
            if matches!(enemy.family, EnemyFamily::Diamond) {
                continue;
            }
            let ex = (enemy.x_pos >> 16) as i32;
            let ey = (enemy.y_pos >> 16) as i32;
            let dx = (sword_x - ex).abs();
            let dy = (sword_y - ey).abs();
            if dx <= RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_WIDTH as i32
                && dy <= RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32
            {
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(i);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                }
            }
        }

        for &idx in kill_indices.iter().rev() {
            self.remove_dead_enemy(idx);
        }
        if !kill_indices.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// Check friendly projectiles (Eyeclops beam, Maniaxe axe) against enemies.
    pub(crate) fn process_friendly_projectile_hits(&mut self) {
        let mut kill_indices = Vec::new();
        let mut projectile_consumed = Vec::new();

        for (pi, proj) in self.projectiles.iter().enumerate() {
            if !proj.friendly || proj.lifetime == 0 {
                continue;
            }
            let prx = (proj.x_pos >> 16) as i32;
            let pry = (proj.y_pos >> 16) as i32;

            for (ei, enemy) in self.active_enemies.iter_mut().enumerate() {
                if matches!(enemy.family, EnemyFamily::Diamond) {
                    continue;
                }
                let ex = (enemy.x_pos >> 16) as i32;
                let ey = (enemy.y_pos >> 16) as i32;
                let dx = (prx - ex).abs();
                let dy = (pry - ey).abs();
                if dx <= PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_WIDTH as i32
                    && dy <= PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32
                {
                    enemy.hit_points = enemy.hit_points.saturating_sub(1);
                    if enemy.hit_points == 0 {
                        kill_indices.push(ei);
                    } else {
                        enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    }
                    projectile_consumed.push(pi);
                    break; // one enemy per projectile per frame
                }
            }
        }

        // Remove consumed projectiles
        for &pi in projectile_consumed.iter().rev() {
            self.projectiles.remove(pi);
        }

        // Remove dead enemies, spawn death effects, and award points
        for &idx in kill_indices.iter().rev() {
            self.remove_dead_enemy(idx);
        }
        if !kill_indices.is_empty() || !projectile_consumed.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// When Berzerker is charging, side contact with enemies damages them.
    pub(crate) fn process_berzerker_charge_hits(&mut self, slice: &OpeningSliceState) {
        if !self.helmet.berzerker_charging {
            return;
        }

        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;

        let mut kill_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter_mut().enumerate() {
            if matches!(enemy.family, EnemyFamily::Diamond) {
                continue;
            }
            let ex = (enemy.x_pos >> 16) as i32;
            let ey = (enemy.y_pos >> 16) as i32;
            let dx = (px - ex).abs();
            let dy = (py - ey).abs();
            if dx <= PLAYER_HALF_WIDTH + ENEMY_CONTACT_HALF_WIDTH as i32
                && dy <= ENEMY_CONTACT_HALF_HEIGHT as i32
            {
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(i);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                }
            }
        }

        for &idx in kill_indices.iter().rev() {
            self.remove_dead_enemy(idx);
        }
        if !kill_indices.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// Step Eyeclops beam: C press fires a horizontal beam projectile.
    pub(crate) fn step_eyeclops_beam(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Eyeclops {
            self.helmet.eyeclops_beam_active = false;
            self.helmet.eyeclops_beam_counter = 0;
            self.helmet.eyeclops_beam_hard = false;
            return;
        }

        let beam_still_live = self
            .projectiles
            .iter()
            .any(|p| p.friendly && p.lifetime > 0 && p.gravity == 0);
        if self.helmet.eyeclops_beam_active && !beam_still_live {
            self.helmet.eyeclops_beam_active = false;
            self.helmet.eyeclops_beam_counter = 0;
            self.helmet.eyeclops_beam_hard = false;
        }

        let fire_beam = |runtime: &mut NativeRuntime, direction: i32| {
            runtime.projectiles.push(EnemyProjectile {
                // ASM sub_E49A latches the current x_direction in `$FFFFFABE`
                // and grows the beam from Kid's live coordinates on each wake.
                // Native still models the gameplay effect as one horizontal
                // friendly projectile, but it now mirrors the same beam-state
                // counter in `eyeclops_beam_counter`.
                x_pos: slice.player.x_pos,
                y_pos: (slice.player.y_pos as i64 - (8 << 16)) as u32,
                x_velocity: EYECLOPS_BEAM_SPEED * direction,
                y_velocity: 0,
                lifetime: EYECLOPS_BEAM_LIFETIME,
                friendly: true,
                gravity: 0,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        };
        let direction = if slice.player.facing == Direction::Right {
            1
        } else {
            -1
        };
        let hard_beam_requested = input.b && input.c;
        if hard_beam_requested
            && !self.helmet.eyeclops_beam_active
            && self.progression.diamond_count >= 2
        {
            // ASM sub_E49A seeds `$FFFFFAB8` with `$8001` for the hard beam and
            // immediately spends two diamonds before the beam object renders.
            self.progression.diamond_count -= 2;
            self.helmet.eyeclops_beam_active = true;
            self.helmet.eyeclops_beam_counter = 0x8001;
            self.helmet.eyeclops_beam_hard = true;
            fire_beam(self, direction);
            return;
        }

        // C rising edge fires beam (one at a time)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !self.helmet.eyeclops_beam_active {
            self.helmet.eyeclops_beam_active = true;
            self.helmet.eyeclops_beam_counter = 1;
            self.helmet.eyeclops_beam_hard = false;
            fire_beam(self, direction);
        } else if self.helmet.eyeclops_beam_active {
            self.helmet.eyeclops_beam_counter = self.helmet.eyeclops_beam_counter.wrapping_add(1);
        }
    }

    /// Step Maniaxe thrown axe: boomerang projectile that reverses after 12 frames.
    pub(crate) fn step_maniaxe_axe(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Maniaxe {
            self.helmet.maniaxe_axe_active = false;
            return;
        }

        // C rising edge throws axe (only one at a time)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !self.helmet.maniaxe_axe_active {
            self.helmet.maniaxe_axe_active = true;
            self.helmet.maniaxe_turnaround = MANIAXE_TRAVEL_FRAMES;
            let dir = if slice.player.facing == Direction::Right {
                1
            } else {
                -1
            };
            let speed = (MANIAXE_BASE_SPEED * dir) << 16;
            self.projectiles.push(EnemyProjectile {
                x_pos: slice.player.x_pos,
                y_pos: (slice.player.y_pos as i64 - (16 << 16)) as u32,
                x_velocity: speed,
                y_velocity: 0,
                lifetime: (MANIAXE_TRAVEL_FRAMES as u16) * 2 + MANIAXE_STARTUP_FRAMES as u16,
                friendly: true,
                gravity: 0,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }

        // Turnaround: reverse axe projectile direction after travel frames
        if self.helmet.maniaxe_axe_active {
            if self.helmet.maniaxe_turnaround > 0 {
                self.helmet.maniaxe_turnaround -= 1;
            }
            if self.helmet.maniaxe_turnaround == 0 {
                // Reverse the axe projectile direction
                if let Some(p) = self.projectiles.last_mut() {
                    p.x_velocity = -p.x_velocity;
                    self.helmet.maniaxe_turnaround = MANIAXE_TRAVEL_FRAMES;
                }
            }
            // Check if axe expired
            if !self.projectiles.iter().any(|p| p.lifetime > 0) {
                self.helmet.maniaxe_axe_active = false;
            }
        }
    }

    /// Update Berzerker charge state: builds accumulator when moving fast, activates on overflow.
    pub(crate) fn step_berzerker_charge(&mut self, slice: &OpeningSliceState) {
        if self.progression.current_form != PlayerForm::Berzerker {
            self.helmet.berzerker_charging = false;
            self.helmet.berzerker_charge_accum = 0;
            return;
        }

        let vel_abs = slice.player.x_velocity.unsigned_abs() as i32;
        if vel_abs < BERZERKER_CHARGE_VEL_THRESHOLD
            || !(slice.player.on_ground || self.previous_input.left || self.previous_input.right)
        {
            self.helmet.berzerker_charging = false;
            self.helmet.berzerker_charge_accum = 0;
            return;
        }

        let (new_accum, overflow) = self
            .helmet
            .berzerker_charge_accum
            .overflowing_add(BERZERKER_CHARGE_ACCUM_STEP);
        if overflow {
            self.helmet.berzerker_charge_accum = 0xFF;
            self.helmet.berzerker_charging = true;
        } else {
            self.helmet.berzerker_charge_accum = new_accum;
        }
    }

    /// Berzerker charge wall break: when charging into a Smash block wall, break it.
    pub(crate) fn berzerker_break_wall_block(&mut self, slice: &OpeningSliceState) {
        let Some(grid) = self.block_grid.as_mut() else {
            return;
        };
        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let moving_right = slice.player.x_velocity > 0
            || (slice.player.x_velocity == 0 && slice.player.facing == Direction::Right);
        let probe = terrain::probe_wall(
            grid,
            x_px,
            y_px,
            PLAYER_HALF_WIDTH,
            PLAYER_HEIGHT,
            moving_right,
        );
        if probe.hit && probe.cell.has_block && probe.cell.block_type == terrain::BlockType::Smash {
            grid.set_cell_at(
                probe.cell_x,
                probe.cell_y,
                terrain::Cell::from_be_bytes(0, 0),
            );
        }
    }

    /// Juggernaut skull shot: C fires a skull projectile with gravity arc.
    pub(crate) fn step_juggernaut_skull(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Juggernaut {
            self.helmet.juggernaut_skull_delay_frames = 0;
            return;
        }

        if self.helmet.juggernaut_skull_delay_frames > 0 {
            self.helmet.juggernaut_skull_delay_frames -= 1;
            if self.helmet.juggernaut_skull_delay_frames == 0 {
                let (x_pos, y_pos, x_velocity) =
                    juggernaut_skull_spawn_anchor_sub_87B0(&slice.player);
                self.projectiles.push(EnemyProjectile {
                    x_pos,
                    y_pos,
                    x_velocity,
                    y_velocity: 0,
                    lifetime: JUGGERNAUT_SKULL_LIFETIME,
                    friendly: true,
                    gravity: JUGGERNAUT_SKULL_GRAVITY,
                    homing_accel: 0,
                    speed_cap: 0,
                    wall_bounce: false,
                    reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
                    ..Default::default()
                });
            }
        }

        let c_pressed = input.c && !self.prior_input.c;
        if !c_pressed || self.helmet.juggernaut_skull_delay_frames > 0 {
            return;
        }

        // ASM loc_8D2C (kid.asm:10136) checks the shared live-skull count
        // before it allocates the deferred `sub_86FA` worker.
        let active_skulls = self
            .projectiles
            .iter()
            .filter(|projectile| is_juggernaut_skull_projectile(projectile))
            .count();
        if active_skulls >= JUGGERNAUT_MAX_SKULLS {
            return;
        }

        // ASM sub_86FA (kid.asm:9568) hibernates for three frames while the
        // firing worker tracks Kid, then spawns `sub_87B0`.
        self.helmet.juggernaut_skull_delay_frames = JUGGERNAUT_SKULL_STARTUP_FRAMES;
    }

    /// Diamond Power: Start+A with 20+ diamonds activates a per-form special.
    /// ASM: DiamondPower_Run (kid.asm:44331). Eyeclops excluded.
    ///
    /// Two tiers: 20 diamonds = basic power, 50 diamonds (20+30) = enhanced power.
    /// Each form has a unique effect per tier (ASM: DiamondPower_Init_Index).
    pub(crate) fn step_diamond_power(&mut self, input: FrameInput) {
        // Count down active Diamond Power
        if self.progression.diamond_power_frames > 0 {
            self.progression.diamond_power_frames -= 1;

            match self.progression.diamond_power_type {
                DiamondPowerType::CircleOfDoom | DiamondPowerType::WallOfDeath => {
                    // AoE damage: kill enemies each frame (orbiting diamonds / wall)
                    self.progression.invulnerable_frames =
                        self.progression.invulnerable_frames.max(1);
                    self.diamond_power_kill_enemies();
                }
                DiamondPowerType::Invulnerability | DiamondPowerType::SamuraiHaze => {
                    // Pure invulnerability, no AoE damage
                    self.progression.invulnerable_frames =
                        self.progression.invulnerable_frames.max(1);
                }
                DiamondPowerType::SlashingRain | DiamondPowerType::TrackingRain => {
                    // ASM loc_3FE62 (kid.asm:45204): rain main loop preamble —
                    // d3=$7F-Camera_X, d4=$7F-Camera_Y screen offsets,
                    // a5=Kid x_vel*3/4 (drift for non-tracking rain),
                    // d7=Time_Frames&7 (tracking check cadence), then
                    // iterates diamond power object linked list via loc_3FE90.
                    //
                    // ASM loc_3FE90 (kid.asm:45221): per-object loop entry —
                    // tst.w $3E(a0): nonzero → loc_3FE98 (tracking: apply own
                    // velocities), zero → loc_3FEBC (not yet tracking: check
                    // frame timing, search enemies, possibly start tracking).
                    //
                    // TODO(fidelity): native uses simplified projectile spawning;
                    // ASM uses linked-list DP object pool with tracking state.
                    // Rain projectiles from top of screen every 4 frames
                    if self.progression.diamond_power_frames.is_multiple_of(4)
                        && self.projectiles.len() < MAX_PROJECTILES
                    {
                        let slice = self.opening_slice.as_ref();
                        let cam_x = slice.map(|s| s.camera.x_pos).unwrap_or(0);
                        // Random-ish X across screen width (320px)
                        let offset = ((self.progression.diamond_power_frames as u32)
                            .wrapping_mul(137))
                            % 320;
                        let rain_x = (cam_x as i64 + ((offset as i64) << 16)) as u32;
                        let rain_y = slice.map(|s| s.camera.y_pos).unwrap_or(0);
                        self.projectiles.push(EnemyProjectile {
                            x_pos: rain_x,
                            y_pos: rain_y,
                            x_velocity: 0,
                            y_velocity: 0x3_0000, // fast downward
                            lifetime: 60,
                            gravity: 0,
                            friendly: true, // damages enemies, not player
                            homing_accel: 0,
                            speed_cap: 0,
                            wall_bounce: false,
                            ..Default::default()
                        });
                    }
                }
                DiamondPowerType::FiveWayShot => {
                    // 5-way spread on first frame only
                    if self.progression.diamond_power_frames
                        == diamond_power_duration(DiamondPowerType::FiveWayShot) - 1
                    {
                        let slice = self.opening_slice.as_ref();
                        let (px, py) = slice
                            .map(|s| (s.player.x_pos, s.player.y_pos))
                            .unwrap_or((0, 0));
                        let spreads: [(i32, i32); 5] = [
                            (0x4_0000, -0x2_0000),  // far right, up
                            (0x2_0000, -0x3_0000),  // right, steep up
                            (0, -0x4_0000),         // straight up
                            (-0x2_0000, -0x3_0000), // left, steep up
                            (-0x4_0000, -0x2_0000), // far left, up
                        ];
                        for (xv, yv) in spreads {
                            if self.projectiles.len() < MAX_PROJECTILES {
                                self.projectiles.push(EnemyProjectile {
                                    x_pos: px,
                                    y_pos: py,
                                    x_velocity: xv,
                                    y_velocity: yv,
                                    lifetime: 60,
                                    gravity: 0x2000,
                                    friendly: true,
                                    homing_accel: 0,
                                    speed_cap: 0,
                                    wall_bounce: false,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
                DiamondPowerType::MiniSnake | DiamondPowerType::DeathSnake => {
                    // Forward-firing diamond chain: spawn projectile every 8 frames
                    self.progression.invulnerable_frames =
                        self.progression.invulnerable_frames.max(1);
                    if self.progression.diamond_power_frames.is_multiple_of(8)
                        && self.projectiles.len() < MAX_PROJECTILES
                    {
                        let slice = self.opening_slice.as_ref();
                        let (px, py, facing) = slice
                            .map(|s| (s.player.x_pos, s.player.y_pos, s.player.facing))
                            .unwrap_or((0, 0, Direction::Right));
                        let speed =
                            if self.progression.diamond_power_type == DiamondPowerType::MiniSnake {
                                0x3_0000 // 0x300 scaled
                            } else {
                                0x4_0000 // 0x400 scaled
                            };
                        let xv = if facing == Direction::Right {
                            speed
                        } else {
                            -speed
                        };
                        self.projectiles.push(EnemyProjectile {
                            x_pos: px,
                            y_pos: py,
                            x_velocity: xv,
                            y_velocity: 0,
                            lifetime: 90,
                            gravity: 0,
                            friendly: true,
                            homing_accel: 0,
                            speed_cap: 0,
                            wall_bounce: false,
                            ..Default::default()
                        });
                    }
                }
                DiamondPowerType::None
                | DiamondPowerType::ExtraHitPoint
                | DiamondPowerType::ExtraLife => {}
            }

            if self.progression.diamond_power_frames == 0 {
                self.progression.diamond_power_type = DiamondPowerType::None;
            }
            return;
        }

        // Activation: Start+A held, not Eyeclops, 20+ diamonds
        // ASM: DiamondPower_Check (kid.asm:44357)
        if !(input.start && input.a) {
            return;
        }
        if self.progression.current_form == PlayerForm::Eyeclops {
            return;
        }

        // ASM sub_7428 (kid.asm:7569) arms Juggernaut's 5-way shot once
        // Start+A is held with 5+ diamonds; the shared DiamondPower_Run path
        // then spends that reduced cost instead of the normal 20.
        let cost = if juggernaut_five_way_shot_ready_sub_7428(
            self.progression.current_form,
            input,
            self.progression.diamond_count,
        ) {
            5
        } else {
            20
        };
        if self.progression.diamond_count < cost {
            return;
        }

        // Two-tier diamond cost (ASM: DiamondPower_Check):
        // Always costs base amount. If remaining >= 30 after first deduction, costs 30 more
        // (50 total for non-Juggernaut) for enhanced tier-2 power.
        self.progression.diamond_count -= cost;
        let tier2 = if self.progression.current_form != PlayerForm::Juggernaut
            && self.progression.diamond_count >= 30
        {
            self.progression.diamond_count -= 30;
            true
        } else {
            false
        };

        let power = diamond_power_for_form(self.progression.current_form, tier2);
        self.progression.diamond_power_type = power;

        // Instant effects (no timer)
        match power {
            DiamondPowerType::ExtraHitPoint => {
                self.progression.number_hitpoints =
                    self.progression.number_hitpoints.saturating_add(1);
                self.progression.diamond_power_frames = 1; // single frame for visual feedback
                return;
            }
            DiamondPowerType::ExtraLife => {
                self.progression.number_lives = self.progression.number_lives.saturating_add(1);
                self.progression.diamond_power_frames = 1;
                return;
            }
            _ => {}
        }

        // ASM loc_3FDFE (kid.asm:45172): shared init epilogue for
        // DiamondPower_Init_TrackingRain/SlashingRain — clr.w ($FFFFF5C0).w
        // resets active rain object count to 0. Native has no explicit count
        // (projectile list length serves as the equivalent).
        self.progression.diamond_power_frames = diamond_power_duration(power);
    }

    /// AoE damage helper: kill all non-diamond enemies (used by CircleOfDoom, WallOfDeath).
    fn diamond_power_kill_enemies(&mut self) {
        let mut kill_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter_mut().enumerate() {
            if matches!(enemy.family, EnemyFamily::Diamond) {
                continue;
            }
            enemy.hit_points = enemy.hit_points.saturating_sub(1);
            if enemy.hit_points == 0 {
                kill_indices.push(i);
            } else {
                enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
            }
        }
        for &idx in kill_indices.iter().rev() {
            self.remove_dead_enemy(idx);
        }
        if !kill_indices.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }
}
