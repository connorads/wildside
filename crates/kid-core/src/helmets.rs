use super::*;

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
    // ASM sub_3F5E8 walks the active linked list from the sentinel at
    // `$FFFFF5A0`, unlinks `a0`, then prepends it to the free list.
    let remove_index = lists
        .active_slots
        .iter()
        .position(|&slot| slot == remove_slot)
        .expect("ASM sub_3F5E8 expects a live diamond-power slot");
    lists.active_slots.remove(remove_index);
    lists.free_slots.insert(0, remove_slot);

    if lists.last_active_slot == Some(remove_slot) {
        lists.last_active_slot = lists.active_slots.last().copied();
    }

    // ASM `tst.l (Addr_FirstDPObjectSlot).w; sne (Diamond_power_active).w`.
    // If no active slots remain, it also clears the three shared flags.
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
                self.helmet.eyeclops_beam_active = self
                    .projectiles
                    .iter()
                    .any(|projectile| projectile.friendly && projectile.lifetime > 0);
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
            return;
        }

        // Check if beam expired
        if self.helmet.eyeclops_beam_active && !self.projectiles.iter().any(|p| p.lifetime > 0) {
            self.helmet.eyeclops_beam_active = false;
        }

        // C rising edge fires beam (one at a time)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !self.helmet.eyeclops_beam_active {
            self.helmet.eyeclops_beam_active = true;
            let dir = if slice.player.facing == Direction::Right {
                1
            } else {
                -1
            };
            self.projectiles.push(EnemyProjectile {
                x_pos: slice.player.x_pos,
                y_pos: (slice.player.y_pos as i64 - (8 << 16)) as u32,
                x_velocity: EYECLOPS_BEAM_SPEED * dir,
                y_velocity: 0,
                lifetime: EYECLOPS_BEAM_LIFETIME,
                friendly: true,
                gravity: 0,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
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
