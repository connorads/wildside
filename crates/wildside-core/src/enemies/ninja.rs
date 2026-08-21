use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaInitState {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) priority: u8,
    pub(crate) current_hp: u8,
    pub(crate) walk_speed_override: i32,
}

/// ASM Enemy11_Ninja_Init `loc_3D554` computes `current_hp` from layout level
/// and seeds shared ninja state used by the stealth pacer speed path.
pub(crate) fn ninja_init_state_loc_3d554(enemy_level: u8) -> NinjaInitState {
    let current_hp = spawn_hit_points(EnemyFamily::Ninja, enemy_level);
    NinjaInitState {
        status_flag_13: true,
        object_meta: 0x0011,
        priority: 0x00,
        current_hp,
        walk_speed_override: ninja_walk_speed_override_loc_3d59a_and_3d5aa(current_hp),
    }
}

fn ninja_saved_walk_speed_loc_3d7ca(enemy_level: u8) -> i32 {
    // ASM Ninja.asm loc_3D670/loc_3D7CA: threat and hurt recovery copy the
    // level-selected speed retained in `$5E(a5)` back into `$50(a5)`.
    ninja_init_state_loc_3d554(enemy_level).walk_speed_override
}

/// Ninja jump-threat cone: player must be 0x20–0x50 pixels above and in the
/// half-open X range [-0x10, +0x10) (Ninja.asm loc_3D706 -> loc_3D84E).
pub(crate) const NINJA_DETECT_Y_MIN: i32 = 0x20;
pub(crate) const NINJA_DETECT_Y_MAX: i32 = 0x50;
pub(crate) const NINJA_DETECT_X_RANGE: i32 = 0x10;

/// ASM Ninja.asm `stru_3D504`: non-terminal hurt holds its sole frame for `$14` ticks.
pub(crate) const NINJA_HURT_ANIMATION_FRAMES_STRU_3D504: u8 = 0x14;

fn ninja_detects_player_while_stealthed(dx_px: i64, dy_px: i64) -> bool {
    // ASM loc_3D706: attack only when kid_y - ninja_y is in [-$50, -$20] and
    // kid_x - ninja_x is in [-$10, +$10). The right edge is exclusive because
    // `cmpi.w #$10,d4` falls through only on `d4 < $10`.
    dy_px >= NINJA_DETECT_Y_MIN as i64
        && dy_px <= NINJA_DETECT_Y_MAX as i64
        && dx_px >= -(NINJA_DETECT_X_RANGE as i64)
        && dx_px < NINJA_DETECT_X_RANGE as i64
}

fn ninja_position_word_delta_px_loc_3d706(lhs_pos: u32, rhs_pos: u32) -> i16 {
    // ASM Ninja.asm loc_3D706/loc_3D734/loc_3D7FA: positions are compared as
    // integer words (`move.w x_pos(a4),d4; sub.w x_pos(a3),d4`), so ignore
    // fractional halves before subtracting.
    let lhs_bytes = lhs_pos.to_be_bytes();
    let rhs_bytes = rhs_pos.to_be_bytes();
    let lhs_word = u16::from_be_bytes([lhs_bytes[0], lhs_bytes[1]]);
    let rhs_word = u16::from_be_bytes([rhs_bytes[0], rhs_bytes[1]]);
    lhs_word.wrapping_sub(rhs_word) as i16
}

fn ninja_position_integer_px_loc_3d878(pos: u32) -> i32 {
    let bytes = pos.to_be_bytes();
    i32::from(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn ninja_floor_y_fp_loc_3d8e6(floor_y_px: i32) -> u32 {
    let floor_word = u16::try_from(floor_y_px).unwrap_or(0).to_be_bytes();
    u32::from_be_bytes([floor_word[0], floor_word[1], 0, 0])
}

fn ninja_floor_collision_word_loc_3d8e6(floor: terrain::FloorProbe, enemy_y_pos: u32) -> u16 {
    if floor.on_ground && enemy_y_pos >= ninja_floor_y_fp_loc_3d8e6(floor.floor_y_px) {
        NINJA_COLID_FLOOR
    } else {
        0
    }
}

fn ninja_jump_attack_moving_right_loc_3da46(x_velocity: i32) -> bool {
    x_velocity >= 0
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaKidThreatRouteLoc3D734 {
    /// No close Kid-driven guard/contact response.
    ProjectileThreatCheck,
    /// Enter the blocking/guard pose facing d4.
    GuardPose,
    /// Berzerker charging takes the kill path.
    BerzerkerKillPath,
}

pub(crate) fn ninja_kid_threat_route_loc_3d734(
    kid_minus_ninja_x_px: i16,
    kid_minus_ninja_y_px: i16,
    player_form: PlayerForm,
    berzerker_charging: bool,
) -> NinjaKidThreatRouteLoc3D734 {
    // ASM Ninja.asm loc_3D734: d4 is `kid_x - ninja_x`. The broad +/-$50
    // band gates Berzerker's charge kill path; Red Stealth blocking then
    // narrows to +/-$30 and requires Kid to be no more than 4 px above.
    if !(-0x50..=0x50).contains(&kid_minus_ninja_x_px) {
        return NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck;
    }

    if berzerker_charging {
        return NinjaKidThreatRouteLoc3D734::BerzerkerKillPath;
    }

    if !(-0x30..=0x30).contains(&kid_minus_ninja_x_px) || kid_minus_ninja_y_px < -4 {
        return NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck;
    }

    if player_form == PlayerForm::RedStealth {
        NinjaKidThreatRouteLoc3D734::GuardPose
    } else {
        NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaProjectileThreatRouteLoc3D776 {
    /// No active Maniaxe projectile threat; continue normal post-threat handling.
    ContinuePostThreat,
    /// Enter the blocking/guard pose facing d4.
    GuardPose,
}

pub(crate) fn ninja_projectile_threat_route_loc_3d776(
    projectile_minus_ninja_x_px: Option<i16>,
    player_form: PlayerForm,
) -> NinjaProjectileThreatRouteLoc3D776 {
    // ASM Ninja.asm loc_3D776: require a live Kid projectile, Current_Helmet
    // Maniaxe, and projectile_x - ninja_x in the half-open [-$5A, +$5A) band.
    let Some(projectile_dx) = projectile_minus_ninja_x_px else {
        return NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat;
    };

    if player_form == PlayerForm::Maniaxe && (-0x5A..0x5A).contains(&projectile_dx) {
        NinjaProjectileThreatRouteLoc3D776::GuardPose
    } else {
        NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaPostThreatRouteLoc3D79C {
    /// Keep the current guard/pacing branch.
    ResumePacing,
    /// Clear the guard state, restore walk animation/speed, and resume moving.
    ResetStealthPacing,
    /// Positive `$4C(a5)` state branches to the airborne proximity check.
    PositiveStateCheck,
}

pub(crate) fn ninja_post_threat_route_loc_3d79c(
    threat_state_4c: i8,
    kid_projectile_present: bool,
    kid_minus_ninja_x_px: i16,
    player_form: PlayerForm,
) -> NinjaPostThreatRouteLoc3D79C {
    // ASM Ninja.asm loc_3D79C: `$4C(a5)` is a signed byte. Zero keeps the
    // normal branch, positive delegates to the airborne proximity check, and
    // negative guard state persists while a Kid projectile exists or Red
    // Stealth remains inside the inclusive +/-$30 Kid X band.
    match threat_state_4c.cmp(&0) {
        std::cmp::Ordering::Equal => return NinjaPostThreatRouteLoc3D79C::ResumePacing,
        std::cmp::Ordering::Greater => {
            return NinjaPostThreatRouteLoc3D79C::PositiveStateCheck;
        }
        std::cmp::Ordering::Less => {}
    }

    if kid_projectile_present {
        return NinjaPostThreatRouteLoc3D79C::ResumePacing;
    }

    if (-0x30..=0x30).contains(&kid_minus_ninja_x_px) && player_form == PlayerForm::RedStealth {
        NinjaPostThreatRouteLoc3D79C::ResumePacing
    } else {
        NinjaPostThreatRouteLoc3D79C::ResetStealthPacing
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaResetStealthPacingRouteLoc3D7CA {
    Loc3E546,
    Loc3E392,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaResetStealthPacingStateLoc3D7CA {
    pub(crate) threat_state_4c: i8,
    pub(crate) animation_script: &'static str,
    pub(crate) is_animated: bool,
    pub(crate) walk_speed_override: i32,
    pub(crate) next: NinjaResetStealthPacingRouteLoc3D7CA,
}

pub(crate) fn ninja_reset_stealth_pacing_loc_3d7ca(
    saved_walk_speed_override_5e: i32,
    x_direction_right: bool,
) -> NinjaResetStealthPacingStateLoc3D7CA {
    // ASM Ninja.asm loc_3D7CA: `sf $4C(a5)`, initialise `stru_3D4EA`,
    // `st is_animated(a3)`, restore `$5E(a5)` into `$50(a5)`, then dispatch
    // clear/set x_direction to loc_3E546/loc_3E392.
    NinjaResetStealthPacingStateLoc3D7CA {
        threat_state_4c: 0,
        animation_script: "stru_3D4EA",
        is_animated: true,
        walk_speed_override: saved_walk_speed_override_5e,
        next: if x_direction_right {
            NinjaResetStealthPacingRouteLoc3D7CA::Loc3E392
        } else {
            NinjaResetStealthPacingRouteLoc3D7CA::Loc3E546
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaResumePacingRouteLoc3D7EE {
    Loc3E560,
    Loc3E3AC,
}

pub(crate) fn ninja_resume_pacing_route_loc_3d7ee(
    x_direction_right: bool,
) -> NinjaResumePacingRouteLoc3D7EE {
    // ASM Ninja.asm loc_3D7EE: `tst.b x_direction(a3); beq.w loc_3E560;
    // bra.w loc_3E3AC`. Clear x_direction keeps the left-facing loop, set
    // x_direction resumes the right-facing loop.
    if x_direction_right {
        NinjaResumePacingRouteLoc3D7EE::Loc3E3AC
    } else {
        NinjaResumePacingRouteLoc3D7EE::Loc3E560
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaPositiveStateRouteLoc3D7FA {
    ResetStealthPacing {
        next: NinjaResetStealthPacingRouteLoc3D7CA,
    },
    ResumePacing {
        next: NinjaResumePacingRouteLoc3D7EE,
    },
}

pub(crate) fn ninja_positive_state_route_loc_3d7fa(
    kid_minus_ninja_x_px: i16,
    kid_minus_ninja_y_px: i16,
    x_direction_right: bool,
) -> NinjaPositiveStateRouteLoc3D7FA {
    // ASM Ninja.asm loc_3D7FA: require Kid to be above the ninja with
    // `kid_y - ninja_y` in [-$60, 0) and `kid_x - ninja_x` in inclusive
    // [-$10, +$10]. Outside that box branches to loc_3D7CA; inside it branches
    // to loc_3D7EE.
    if !(-0x60..0).contains(&kid_minus_ninja_y_px)
        || !(-0x10..=0x10).contains(&kid_minus_ninja_x_px)
    {
        let reset = ninja_reset_stealth_pacing_loc_3d7ca(0, x_direction_right);
        return NinjaPositiveStateRouteLoc3D7FA::ResetStealthPacing { next: reset.next };
    }

    NinjaPositiveStateRouteLoc3D7FA::ResumePacing {
        next: ninja_resume_pacing_route_loc_3d7ee(x_direction_right),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaThreatPoseRoute {
    Loc3D79C,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaThreatPoseState {
    pub(crate) threat_state_4c: i8,
    pub(crate) is_animated: bool,
    pub(crate) sprite_link: &'static str,
    pub(crate) walk_speed_override: i32,
    pub(crate) x_direction_right: bool,
    pub(crate) next: NinjaThreatPoseRoute,
}

pub(crate) fn ninja_guard_pose_loc_3d826(
    d4_kid_or_projectile_minus_ninja_x_px: i16,
) -> NinjaThreatPoseState {
    // ASM Ninja.asm loc_3D826: `st $4C(a5)`, clear animation, select
    // `LnkTo_unk_C7D8E`, zero `$50(a5)`, face by signed d4, then branch to
    // loc_3D79C. Negative d4 keeps `x_direction` clear; zero/positive sets it.
    NinjaThreatPoseState {
        threat_state_4c: -1,
        is_animated: false,
        sprite_link: "LnkTo_unk_C7D8E",
        walk_speed_override: 0,
        x_direction_right: d4_kid_or_projectile_minus_ninja_x_px >= 0,
        next: NinjaThreatPoseRoute::Loc3D79C,
    }
}

pub(crate) fn ninja_jump_threat_pose_loc_3d84e(
    d4_kid_minus_ninja_x_px: i16,
) -> NinjaThreatPoseState {
    // ASM Ninja.asm loc_3D84E: clear animation, set `$4C(a5)` to positive 1,
    // select `LnkTo_unk_C7D86`, zero `$50(a5)`, face by signed d4, then branch
    // to loc_3D79C.
    NinjaThreatPoseState {
        threat_state_4c: 1,
        is_animated: false,
        sprite_link: "LnkTo_unk_C7D86",
        walk_speed_override: 0,
        x_direction_right: d4_kid_minus_ninja_x_px >= 0,
        next: NinjaThreatPoseRoute::Loc3D79C,
    }
}

/// Ninja jump-attack velocities (Ninja.asm loc_3D9A4).
pub(crate) const NINJA_JUMP_X_SPEED_LEFT: i32 = 0x1_8000;
pub(crate) const NINJA_JUMP_X_SPEED_RIGHT: i32 = 0x1_0000;
pub(crate) const NINJA_JUMP_Y_SPEED: i32 = -0x4_8000;
pub(crate) const NINJA_GRAVITY: i32 = 0x2800;
pub(crate) const NINJA_KNOCKDOWN_Y_SPEED_LOC_3D8D6: i32 = 0x0001_8000;
pub(crate) const NINJA_KNOCKDOWN_GRAVITY_LOC_3D8E6: i32 = 0x0000_1800;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaJumpAttackStartStateLoc3D9A4 {
    pub(crate) next_collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) has_level_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_link: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaJumpAttackAirborneStateLoc3D9DA {
    pub(crate) y_velocity: i32,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checked_sub_3e8ba: bool,
    pub(crate) sprite_link: Option<&'static str>,
}

pub(crate) fn ninja_jump_attack_start_loc_3d9a4(
    x_direction_right: bool,
) -> NinjaJumpAttackStartStateLoc3D9A4 {
    // ASM Ninja.asm loc_3D9A4: clear `collision_type(a3)`, choose X velocity
    // by `x_direction(a3)`, set jump Y velocity and level-collision/motion
    // flags, clear animation, then select `LnkTo_unk_C7D6E`.
    NinjaJumpAttackStartStateLoc3D9A4 {
        next_collision_type: 0,
        x_velocity: if x_direction_right {
            // ASM Ninja.asm loc_3D9B8: set `x_vel(a3)` to +$10000 for set `x_direction`.
            NINJA_JUMP_X_SPEED_RIGHT
        } else {
            -NINJA_JUMP_X_SPEED_LEFT
        },
        // ASM Ninja.asm loc_3D9C0: seed `y_vel`, enable level collision/movement,
        // clear animation, and select the jump-attack sprite.
        y_velocity: NINJA_JUMP_Y_SPEED,
        has_level_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_link: "LnkTo_unk_C7D6E",
    }
}

pub(crate) fn ninja_jump_attack_airborne_tick_loc_3d9da(
    y_velocity: i32,
) -> NinjaJumpAttackAirborneStateLoc3D9DA {
    // ASM Ninja.asm loc_3D9DA: hibernate, run `sub_3E8BA`, add `$2800` to
    // `y_vel(a3)`, then select the midair sprite from signed velocity bands.
    let accelerated_y_velocity = y_velocity.wrapping_add(NINJA_GRAVITY);
    let sprite_link = if accelerated_y_velocity < -0x0000_8000 {
        None
    } else if accelerated_y_velocity > 0x0000_8000 {
        // ASM Ninja.asm loc_3DA06: falling faster than +$8000 selects C7D7E.
        Some("LnkTo_unk_C7D7E")
    } else {
        Some("LnkTo_unk_C7D76")
    };

    NinjaJumpAttackAirborneStateLoc3D9DA {
        y_velocity: accelerated_y_velocity,
        hibernates_one_frame: true,
        checked_sub_3e8ba: true,
        sprite_link,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaJumpAttackCollisionRouteLoc3DA0C {
    ContinueAirborneLoop,
    Land,
    WallBounce,
    CeilingContact,
    KidAboveContact,
    KidSideContact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaJumpAttackCollisionStateLoc3DA0C {
    pub(crate) next_collision_type: u16,
    pub(crate) route: NinjaJumpAttackCollisionRouteLoc3DA0C,
}

pub(crate) fn ninja_jump_attack_collision_dispatch_loc_3da0c(
    collision_type: u16,
) -> NinjaJumpAttackCollisionStateLoc3DA0C {
    // ASM Ninja.asm loc_3DA0C: copy `collision_type(a3)` into d4, clear it,
    // then compare d4 against floor, side walls, ceiling, and Kid contacts.
    let route = match collision_type {
        NINJA_COLID_FLOOR => NinjaJumpAttackCollisionRouteLoc3DA0C::Land,
        NINJA_COLID_RIGHTWALL | NINJA_COLID_LEFTWALL => {
            NinjaJumpAttackCollisionRouteLoc3DA0C::WallBounce
        }
        NINJA_COLID_CEILING => NinjaJumpAttackCollisionRouteLoc3DA0C::CeilingContact,
        NINJA_COLID_KIDABOVE => NinjaJumpAttackCollisionRouteLoc3DA0C::KidAboveContact,
        NINJA_COLID_KIDRIGHT | NINJA_COLID_KIDLEFT => {
            NinjaJumpAttackCollisionRouteLoc3DA0C::KidSideContact
        }
        _ => NinjaJumpAttackCollisionRouteLoc3DA0C::ContinueAirborneLoop,
    };

    NinjaJumpAttackCollisionStateLoc3DA0C {
        next_collision_type: 0,
        route,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaJumpAttackWallBounceStateLoc3DA46 {
    pub(crate) next_collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) x_direction_right: bool,
    pub(crate) loops_airborne: bool,
}

pub(crate) fn ninja_jump_attack_wall_bounce_loc_3da46(
    x_velocity: i32,
    x_direction_right: bool,
) -> NinjaJumpAttackWallBounceStateLoc3DA46 {
    // ASM Ninja.asm loc_3DA46: clear `collision_type(a3)`, negate x_vel,
    // toggle `x_direction(a3)` with `eori.b #$FF`, then branch to loc_3D9DA.
    NinjaJumpAttackWallBounceStateLoc3DA46 {
        next_collision_type: 0,
        x_velocity: x_velocity.wrapping_neg(),
        x_direction_right: !x_direction_right,
        loops_airborne: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaJumpAttackLandingResumeLoc3DA56 {
    Loc3E566,
    Loc3E3B2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaJumpAttackLandingStateLoc3DA56 {
    pub(crate) next_collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) animation_script: &'static str,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) is_animated: bool,
    pub(crate) next: NinjaJumpAttackLandingResumeLoc3DA56,
}

pub(crate) fn ninja_jump_attack_landing_loc_3da56(
    x_direction_right: bool,
) -> NinjaJumpAttackLandingStateLoc3DA56 {
    // ASM Ninja.asm loc_3DA56: clear collision, zero x/y velocity, initialise
    // `stru_3D4EA`, stop movement and level collision, enable animation, then
    // branch on raw `x_direction(a3)` to loc_3E566/loc_3E3B2.
    NinjaJumpAttackLandingStateLoc3DA56 {
        next_collision_type: 0,
        x_velocity: 0,
        y_velocity: 0,
        animation_script: "stru_3D4EA",
        is_moved: false,
        has_level_collision: false,
        is_animated: true,
        next: if x_direction_right {
            NinjaJumpAttackLandingResumeLoc3DA56::Loc3E3B2
        } else {
            NinjaJumpAttackLandingResumeLoc3DA56::Loc3E566
        },
    }
}

pub(crate) const NINJA_KID_ABOVE_BOUNCE_Y_SPEED_LOC_3DA98: i32 = 0x0003_8000;
pub(crate) const NINJA_KID_ABOVE_LANDING_GRAVITY_LOC_3DABC: i32 = 0x0000_1800;
pub(crate) const NINJA_BERZERKER_SIDE_LAUNCH_Y_SPEED_LOC_3DB0A: i32 = -0x0004_8000;
pub(crate) const NINJA_BERZERKER_SIDE_LAUNCH_GRAVITY_LOC_3DB3C: i32 = 0x0000_4000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaKidAboveContactRouteLoc3DA98 {
    ContinueAirborneLoop,
    FallthroughLandingLoop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaKidAboveContactStateLoc3DA98 {
    pub(crate) next_collision_type: u16,
    pub(crate) y_velocity: i32,
    pub(crate) block_timer_44: u16,
    pub(crate) route: NinjaKidAboveContactRouteLoc3DA98,
}

pub(crate) fn ninja_kid_above_contact_loc_3da98(
    block_timer_44: u16,
) -> NinjaKidAboveContactStateLoc3DA98 {
    // ASM Ninja.asm loc_3DA98: set `y_vel(a3)` to `$38000`, decrement `$44(a3)`
    // as a word, and enter the landing loop only when the post-decrement value is zero.
    let next_block_timer_44 = block_timer_44.wrapping_sub(1);
    NinjaKidAboveContactStateLoc3DA98 {
        next_collision_type: 0,
        y_velocity: NINJA_KID_ABOVE_BOUNCE_Y_SPEED_LOC_3DA98,
        block_timer_44: next_block_timer_44,
        route: if next_block_timer_44 == 0 {
            NinjaKidAboveContactRouteLoc3DA98::FallthroughLandingLoop
        } else {
            NinjaKidAboveContactRouteLoc3DA98::ContinueAirborneLoop
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaKidAboveLandingLoopRouteLoc3DABC {
    Loop,
    To3DB4E,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaKidAboveLandingLoopStateLoc3DABC {
    pub(crate) next_collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) y_position: u32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checked_sub_3e8ba: bool,
    pub(crate) route: NinjaKidAboveLandingLoopRouteLoc3DABC,
}

pub(crate) fn ninja_kid_above_landing_loop_loc_3dabc(
    collision_type: u16,
    x_velocity: i32,
    y_velocity: i32,
    y_position: u32,
) -> NinjaKidAboveLandingLoopStateLoc3DABC {
    // ASM Ninja.asm loc_3DABC: after the Kid-above HP/timer reaches zero,
    // clear collision, hibernate, run `sub_3E8BA`, apply `$1800` gravity, and
    // loop until `colid_floor`. Floor contact clears motion/terrain flags and
    // snaps integer Y to `((y + $F) & $FFF0)` before loc_3DB4E.
    let next_y_velocity = y_velocity.wrapping_add(NINJA_KID_ABOVE_LANDING_GRAVITY_LOC_3DABC);
    if collision_type == NINJA_COLID_FLOOR {
        let snapped_y_position = (((y_position >> 16).wrapping_add(0x0F)) & 0x0000_FFF0) << 16;
        NinjaKidAboveLandingLoopStateLoc3DABC {
            next_collision_type: 0,
            x_velocity: 0,
            y_velocity: 0,
            y_position: snapped_y_position,
            is_moved: false,
            has_level_collision: false,
            hibernates_one_frame: true,
            checked_sub_3e8ba: true,
            route: NinjaKidAboveLandingLoopRouteLoc3DABC::To3DB4E,
        }
    } else {
        NinjaKidAboveLandingLoopStateLoc3DABC {
            next_collision_type: 0,
            x_velocity,
            y_velocity: next_y_velocity,
            y_position,
            is_moved: true,
            has_level_collision: true,
            hibernates_one_frame: true,
            checked_sub_3e8ba: true,
            route: NinjaKidAboveLandingLoopRouteLoc3DABC::Loop,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaDeathSequenceLoc3DB4E {
    pub(crate) death_animation: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) status_action: DeactivationStatusActionLoc3C4C0,
    pub(crate) exits_enemy: bool,
}

pub(crate) fn ninja_death_sequence_loc_3db4e(object_status_slot: u8) -> NinjaDeathSequenceLoc3DB4E {
    // ASM Ninja.asm loc_3DB4E: initialise `stru_3D50A`, hibernate until that
    // animation finishes, then branch to loc_3E956's shared delete tail.
    NinjaDeathSequenceLoc3DB4E {
        death_animation: "stru_3D50A",
        waits_until_animation_finished: true,
        status_action: deactivation_status_update_loc_3C4C0(object_status_slot),
        exits_enemy: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaBerzerkerSideLaunchStateLoc3DB0A {
    pub(crate) has_kid_collision: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_link: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaBerzerkerSideContactRouteLoc3DB0A {
    ContinueAirborneLoop,
    Launch(NinjaBerzerkerSideLaunchStateLoc3DB0A),
}

pub(crate) fn ninja_berzerker_side_contact_loc_3db0a(
    berzerker_charging: bool,
) -> NinjaBerzerkerSideContactRouteLoc3DB0A {
    // ASM Ninja.asm loc_3DB0A: clear `Berzerker_charging` resumes the jump
    // attack airborne loop. While charging, the duplicated `st has_kid_collision`
    // leaves Kid collision enabled, disables level collision and animation,
    // keeps movement enabled, selects C7D7E, zeroes X velocity, and launches
    // upward at `-$48000`.
    if !berzerker_charging {
        return NinjaBerzerkerSideContactRouteLoc3DB0A::ContinueAirborneLoop;
    }

    NinjaBerzerkerSideContactRouteLoc3DB0A::Launch(NinjaBerzerkerSideLaunchStateLoc3DB0A {
        has_kid_collision: true,
        has_level_collision: false,
        is_moved: true,
        is_animated: false,
        sprite_link: "LnkTo_unk_C7D7E",
        x_velocity: 0,
        y_velocity: NINJA_BERZERKER_SIDE_LAUNCH_Y_SPEED_LOC_3DB0A,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaBerzerkerSideLaunchLoopTickLoc3DB3C {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checked_sub_3e8ba: bool,
    pub(crate) y_velocity: i32,
    pub(crate) loops: bool,
}

pub(crate) fn ninja_berzerker_side_launch_loop_tick_loc_3db3c(
    y_velocity: i32,
) -> NinjaBerzerkerSideLaunchLoopTickLoc3DB3C {
    // ASM Ninja.asm loc_3DB3C: each launch tick hibernates one frame, runs
    // `sub_3E8BA`, adds `$4000` to `y_vel(a3)`, then branches back to repeat.
    NinjaBerzerkerSideLaunchLoopTickLoc3DB3C {
        hibernates_one_frame: true,
        checked_sub_3e8ba: true,
        y_velocity: y_velocity.wrapping_add(NINJA_BERZERKER_SIDE_LAUNCH_GRAVITY_LOC_3DB3C),
        loops: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaCeilingContactStateLoc3DAB0 {
    pub(crate) next_collision_type: u16,
    pub(crate) y_velocity: i32,
    pub(crate) loops_airborne: bool,
}

pub(crate) fn ninja_ceiling_contact_loc_3dab0(y_velocity: i32) -> NinjaCeilingContactStateLoc3DAB0 {
    // ASM Ninja.asm loc_3DAB0: clear `collision_type(a3)`, negate `y_vel(a3)`,
    // then branch back to loc_3D9DA.
    NinjaCeilingContactStateLoc3DAB0 {
        next_collision_type: 0,
        y_velocity: y_velocity.wrapping_neg(),
        loops_airborne: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaCollisionResumeLoc3D638 {
    /// ASM Ninja.asm loc_3E7CA: `beq.w loc_3E7CA` path (left-facing).
    Loc3E7CA,
    /// ASM Ninja.asm loc_3E6AE: `bra.w loc_3E6AE` path (right-facing).
    Loc3E6AE,
    /// ASM Ninja.asm loc_3E560: `beq.w loc_3E560` path (left-facing when `$5C(a5)` is set).
    Loc3E560,
    /// ASM Ninja.asm loc_3E3AC: `bra.w loc_3E3AC` path (right-facing when `$5C(a5)` is set).
    Loc3E3AC,
}

pub(crate) fn ninja_collision_resume_loc_3d638(
    wall_pinned: bool,
    x_direction_right: bool,
) -> NinjaCollisionResumeLoc3D638 {
    // ASM Ninja.asm loc_3D638: clear collision, branch by `$5C(a5)` and
    // x_direction to one of `loc_3E7CA/3E6AE/3E560/3E3AC`.
    if wall_pinned {
        ninja_collision_resume_loc_3d64e(x_direction_right)
    } else if x_direction_right {
        NinjaCollisionResumeLoc3D638::Loc3E6AE
    } else {
        NinjaCollisionResumeLoc3D638::Loc3E7CA
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaAirborneCollisionRouteLoc3D878 {
    Resume { next: NinjaCollisionResumeLoc3D638 },
    To3DB4E,
    To3D8BC,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaAirborneCollisionStateLoc3D878 {
    pub(crate) next_collision_type: u16,
    pub(crate) route: NinjaAirborneCollisionRouteLoc3D878,
}

pub(crate) fn ninja_airborne_collision_dispatch_loc_3d878(
    collision_type: i16,
    x_direction_right: bool,
    player_form: PlayerForm,
    kid_sprite_allows_berzerker_knockdown: bool,
) -> NinjaAirborneCollisionStateLoc3D878 {
    // ASM Ninja.asm loc_3D878: capture and clear `collision_type(a3)`. Side
    // contact routes through loc_3D8A8 and only enters loc_3D8BC when
    // Current_Helmet is Berzerker and Kid's `addroffset_sprite` is not after
    // `LnkTo_unk_ADFC8`; otherwise loc_3D89C resumes by x_direction.
    let resume = NinjaAirborneCollisionRouteLoc3D878::Resume {
        next: ninja_collision_resume_loc_3d638(false, x_direction_right),
    };
    let route = match collision_type {
        v if v.is_negative() => NinjaAirborneCollisionRouteLoc3D878::To3DB4E,
        v if v == NINJA_COLID_KIDABOVE as i16 => NinjaAirborneCollisionRouteLoc3D878::To3D8BC,
        v if v == NINJA_COLID_KIDRIGHT as i16 || v == NINJA_COLID_KIDLEFT as i16 => {
            if player_form == PlayerForm::Berzerker && kid_sprite_allows_berzerker_knockdown {
                NinjaAirborneCollisionRouteLoc3D878::To3D8BC
            } else {
                resume
            }
        }
        _ => resume,
    };

    NinjaAirborneCollisionStateLoc3D878 {
        next_collision_type: 0,
        route,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaKnockdownStartRouteLoc3D8D6 {
    FallthroughLandingLoop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaKnockdownStartStateLoc3D8D6 {
    pub(crate) y_velocity: i32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) next: NinjaKnockdownStartRouteLoc3D8D6,
}

pub(crate) fn ninja_knockdown_start_loc_3d8d6() -> NinjaKnockdownStartStateLoc3D8D6 {
    // ASM Ninja.asm loc_3D8D6: set `y_vel(a3)` to `$18000`, set
    // `is_moved(a3)` and `has_level_collision(a3)`, then fall through to
    // the floor-wait loop.
    NinjaKnockdownStartStateLoc3D8D6 {
        y_velocity: NINJA_KNOCKDOWN_Y_SPEED_LOC_3D8D6,
        is_moved: true,
        has_level_collision: true,
        next: NinjaKnockdownStartRouteLoc3D8D6::FallthroughLandingLoop,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaKnockdownLandingRouteLoc3D8E6 {
    Loop,
    KillPath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaKnockdownLandingStateLoc3D8E6 {
    pub(crate) next_collision_type: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) y_position: u32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checked_sub_3e8ba: bool,
    pub(crate) route: NinjaKnockdownLandingRouteLoc3D8E6,
}

pub(crate) fn ninja_knockdown_landing_loop_loc_3d8e6(
    collision_type: u16,
    x_velocity: i32,
    y_velocity: i32,
    y_position: u32,
) -> NinjaKnockdownLandingStateLoc3D8E6 {
    // ASM Ninja.asm loc_3D8E6: clear collision, hibernate one frame, run
    // `sub_3E8BA`, add `$1800` to y velocity, and loop until `colid_floor`.
    // On floor contact, clear both velocities/flags and snap the integer Y word
    // to `((y + $F) & $FFF0)` while clearing sub-pixels.
    let next_y_velocity = y_velocity.wrapping_add(NINJA_KNOCKDOWN_GRAVITY_LOC_3D8E6);
    if collision_type == NINJA_COLID_FLOOR {
        let snapped_y_position = (((y_position >> 16).wrapping_add(0x0F)) & 0x0000_FFF0) << 16;
        NinjaKnockdownLandingStateLoc3D8E6 {
            next_collision_type: 0,
            x_velocity: 0,
            y_velocity: 0,
            y_position: snapped_y_position,
            is_moved: false,
            has_level_collision: false,
            hibernates_one_frame: true,
            checked_sub_3e8ba: true,
            route: NinjaKnockdownLandingRouteLoc3D8E6::KillPath,
        }
    } else {
        NinjaKnockdownLandingStateLoc3D8E6 {
            next_collision_type: 0,
            x_velocity,
            y_velocity: next_y_velocity,
            y_position,
            is_moved: true,
            has_level_collision: true,
            hibernates_one_frame: true,
            checked_sub_3e8ba: true,
            route: NinjaKnockdownLandingRouteLoc3D8E6::Loop,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaFarKidFacingDecisionLoc3D92A {
    WithinBand,
    AlreadyFacingFarKid,
    TurnTowardFarKid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaFarKidFacingStateLoc3D92A {
    pub(crate) x_direction_right: bool,
    pub(crate) decision: NinjaFarKidFacingDecisionLoc3D92A,
}

pub(crate) fn ninja_far_kid_facing_loc_3d92a(
    kid_minus_ninja_x_px: i16,
    x_direction_right: bool,
) -> NinjaFarKidFacingStateLoc3D92A {
    // ASM Ninja.asm loc_3D92A: compute `d4 = kid_x - ninja_x`; only signed
    // values outside the inclusive [-$80, +$80] band reach the facing checks.
    let target_right = if kid_minus_ninja_x_px > 0x80 {
        // ASM Ninja.asm loc_3D978: positive far `d4` keeps set
        // `x_direction`, or toggles clear `x_direction` with `eori.b #$FF`.
        Some(true)
    } else if kid_minus_ninja_x_px < -0x80 {
        // ASM Ninja.asm loc_3D98A: negative far `d4` keeps clear
        // `x_direction`, or toggles set `x_direction` with `eori.b #$FF`.
        Some(false)
    } else {
        None
    };

    let Some(target_right) = target_right else {
        return NinjaFarKidFacingStateLoc3D92A {
            x_direction_right,
            decision: NinjaFarKidFacingDecisionLoc3D92A::WithinBand,
        };
    };

    if target_right == x_direction_right {
        NinjaFarKidFacingStateLoc3D92A {
            x_direction_right,
            decision: NinjaFarKidFacingDecisionLoc3D92A::AlreadyFacingFarKid,
        }
    } else {
        NinjaFarKidFacingStateLoc3D92A {
            x_direction_right: target_right,
            decision: NinjaFarKidFacingDecisionLoc3D92A::TurnTowardFarKid,
        }
    }
}

pub(crate) const NINJA_ATTACK_PERIOD_SHORT_LOC_3D960: u16 = 0x0032;
pub(crate) const NINJA_ATTACK_PERIOD_LONG_LOC_3D960: u16 = 0x012C;
pub(crate) const NINJA_WALK_ANIMATION_CYCLE_FRAMES_LOC_3D4EA: u16 = 40;
pub(crate) const NINJA_THROW_FRAME_PHASE_LOC_3D958: u16 = 37;

pub(crate) fn ninja_walk_throw_frame_phase_loc_3d958(walk_phase: u16) -> bool {
    // ASM ingame/anim/enemy/Ninja.asm stru_3D4EA: frame C7DBE starts after
    // 34 ticks; loc_3D958 wants its third remaining animation tick.
    walk_phase % NINJA_WALK_ANIMATION_CYCLE_FRAMES_LOC_3D4EA == NINJA_THROW_FRAME_PHASE_LOC_3D958
}

pub(crate) fn ninja_advance_walk_phase_loc_3d4ea(walk_phase: u16) -> u16 {
    (walk_phase + 1) % NINJA_WALK_ANIMATION_CYCLE_FRAMES_LOC_3D4EA
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaAttackCooldownRouteLoc3D942 {
    ContinuePacing,
    CheckThrowFrame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaAttackCooldownStateLoc3D942 {
    pub(crate) attack_timer: u16,
    pub(crate) route: NinjaAttackCooldownRouteLoc3D942,
}

pub(crate) fn ninja_attack_cooldown_tick_loc_3d942(
    attack_timer: u16,
) -> NinjaAttackCooldownStateLoc3D942 {
    // ASM Ninja.asm loc_3D942: `subq.w #1,$54(a5); bne.s loc_3D998`.
    // Only a post-decrement zero reaches the throw-frame gate.
    let next_attack_timer = attack_timer.wrapping_sub(1);
    let route = if next_attack_timer == 0 {
        NinjaAttackCooldownRouteLoc3D942::CheckThrowFrame
    } else {
        NinjaAttackCooldownRouteLoc3D942::ContinuePacing
    };

    NinjaAttackCooldownStateLoc3D942 {
        attack_timer: next_attack_timer,
        route,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaThrowFrameGateRouteLoc3D958 {
    ContinuePacing { attack_timer: u16 },
    RotateAttackPeriod,
}

pub(crate) fn ninja_throw_frame_gate_loc_3d958(
    sprite_is_throw_frame: bool,
    animation_timer: u16,
) -> NinjaThrowFrameGateRouteLoc3D958 {
    // ASM Ninja.asm loc_3D958 is reached unless `addroffset_sprite(a3)` is
    // `LnkTo_unk_C7DBE-Data_Index` and `animation_timer(a3)` is exactly 3.
    // The mismatch path stores one frame into `$54(a5)` before loc_3D998.
    if sprite_is_throw_frame && animation_timer == 3 {
        NinjaThrowFrameGateRouteLoc3D958::RotateAttackPeriod
    } else {
        NinjaThrowFrameGateRouteLoc3D958::ContinuePacing { attack_timer: 1 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaAttackPeriodRotationRouteLoc3D960 {
    ContinuePacing,
    StartJumpAttack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaAttackPeriodRotationStateLoc3D960 {
    pub(crate) attack_period_pair: u32,
    pub(crate) attack_timer: u16,
    pub(crate) route: NinjaAttackPeriodRotationRouteLoc3D960,
}

pub(crate) fn ninja_attack_period_rotate_loc_3d960(
    attack_period_pair: u32,
    guard_state: i8,
) -> NinjaAttackPeriodRotationStateLoc3D960 {
    // ASM Ninja.asm loc_3D960: load `$6E(a5)`, `swap d0`, store the rotated
    // long back to `$6E(a5)`, copy d0.w into `$54(a5)`, then only jump to
    // the jump-attack path when `$4C(a5)` is zero.
    let next_attack_period_pair = attack_period_pair.rotate_left(16);
    let next_attack_period_pair_bytes = next_attack_period_pair.to_be_bytes();
    let next_attack_timer = u16::from_be_bytes([
        next_attack_period_pair_bytes[2],
        next_attack_period_pair_bytes[3],
    ]);
    let route = if guard_state == 0 {
        NinjaAttackPeriodRotationRouteLoc3D960::StartJumpAttack
    } else {
        NinjaAttackPeriodRotationRouteLoc3D960::ContinuePacing
    };

    NinjaAttackPeriodRotationStateLoc3D960 {
        attack_period_pair: next_attack_period_pair,
        attack_timer: next_attack_timer,
        route,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaCollisionLoc3D606 {
    /// ASM Ninja.asm loc_3D6DA: no collision, so jump to stealth/attack gate.
    To3D6DA,
    /// ASM Ninja.asm loc_3DB4E: negative collision IDs send to generic hit-kill logic.
    To3DB4E,
    /// ASM Ninja.asm loc_3D65A: hurt or kid-above collisions.
    To3D65A,
    /// ASM Ninja.asm loc_3D698: side collisions against Kid or left/right.
    To3D698,
    /// ASM Ninja.asm loc_3D638: clear collision and route by wall-pinned/direction.
    To3D638 { next: NinjaCollisionResumeLoc3D638 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaCollisionStateLoc3D606 {
    pub(crate) next_collision_type: u16,
    pub(crate) route: NinjaCollisionLoc3D606,
}

pub(crate) fn ninja_collision_dispatch_loc_3d606(
    collision_type: i16,
    wall_pinned: bool,
    x_direction_right: bool,
) -> NinjaCollisionStateLoc3D606 {
    // ASM Ninja.asm loc_3D606:
    // - capture `$collision_type(a3)` in d4 and immediately clear it
    // - branch on zero/negative/hurt/side/other collision IDs
    let route = match collision_type {
        0 => NinjaCollisionLoc3D606::To3D6DA,
        v if v.is_negative() => NinjaCollisionLoc3D606::To3DB4E,
        v if v == NINJA_COLID_HURT as i16 || v == NINJA_COLID_KIDABOVE as i16 => {
            NinjaCollisionLoc3D606::To3D65A
        }
        v if v == NINJA_COLID_KIDRIGHT as i16 || v == NINJA_COLID_KIDLEFT as i16 => {
            NinjaCollisionLoc3D606::To3D698
        }
        _ => NinjaCollisionLoc3D606::To3D638 {
            next: ninja_collision_resume_loc_3d638(wall_pinned, x_direction_right),
        },
    };

    NinjaCollisionStateLoc3D606 {
        next_collision_type: 0,
        route,
    }
}

pub(crate) fn ninja_collision_resume_loc_3d64e(
    x_direction_right: bool,
) -> NinjaCollisionResumeLoc3D638 {
    // ASM Ninja.asm loc_3D64E: with no wall pin, dispatch only by facing:
    // x_direction=0 → loc_3E560 (left-facing), x_direction=$FF → loc_3E3AC.
    if x_direction_right {
        NinjaCollisionResumeLoc3D638::Loc3E3AC
    } else {
        NinjaCollisionResumeLoc3D638::Loc3E560
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaCollisionStateLoc3D66E {
    pub(crate) next: NinjaCollisionResumeLoc3D638,
}

pub(crate) fn ninja_collision_resume_loc_3d66e(
    next: NinjaCollisionResumeLoc3D638,
) -> NinjaCollisionStateLoc3D66E {
    // ASM Ninja.asm loc_3D66E is a plain branch to loc_3D638
    // with no side effects.
    NinjaCollisionStateLoc3D66E { next }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NinjaCollisionLoc3D670Route {
    /// ASM Ninja.asm loc_3D6DA branch after `$44(a5)` underflow.
    To3DB4E,
    /// ASM Ninja.asm loc_3D638 route after blocking animation completes.
    To3D638 { next: NinjaCollisionResumeLoc3D638 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaCollisionStateLoc3D670 {
    pub(crate) next_collision_type: u16,
    pub(crate) remaining_block_timer: u16,
    pub(crate) route: NinjaCollisionLoc3D670Route,
    pub(crate) uses_blocking_animation: bool,
    pub(crate) blocking_animation: Option<&'static str>,
    pub(crate) restored_animation: Option<&'static str>,
    pub(crate) hibernates_until_animation_finished: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn ninja_collision_block_animation_loc_3d670(
    block_timer: u16,
    wall_pinned: bool,
    x_direction_right: bool,
    restored_animation: &'static str,
) -> NinjaCollisionStateLoc3D670 {
    // ASM Ninja.asm loc_3D670:
    // - subtracts 1 from `$44(a5)`
    // - if it reached zero, jumps to loc_3DB4E
    // - otherwise plays `stru_3D504`, waits for it, restores `$62(a5)` into
    //   `$50(a5)`, and reinitialises animation before jumping to loc_3D638.
    let remaining_block_timer = block_timer.wrapping_sub(1);
    if remaining_block_timer == 0 {
        NinjaCollisionStateLoc3D670 {
            next_collision_type: 0,
            remaining_block_timer,
            route: NinjaCollisionLoc3D670Route::To3DB4E,
            uses_blocking_animation: false,
            blocking_animation: None,
            restored_animation: None,
            hibernates_until_animation_finished: false,
        }
    } else {
        NinjaCollisionStateLoc3D670 {
            next_collision_type: 0,
            remaining_block_timer,
            route: NinjaCollisionLoc3D670Route::To3D638 {
                next: ninja_collision_resume_loc_3d638(wall_pinned, x_direction_right),
            },
            uses_blocking_animation: true,
            blocking_animation: Some("stru_3D504"),
            restored_animation: Some(restored_animation),
            hibernates_until_animation_finished: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NinjaCollisionStateLoc3D6C8 {
    pub(crate) y_velocity: i32,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checked_sub_3e8ba: bool,
    pub(crate) loops: bool,
}

pub(crate) fn ninja_collision_stun_loop_loc_3d6c8(y_velocity: i32) -> NinjaCollisionStateLoc3D6C8 {
    // ASM Ninja.asm loc_3D6C8:
    // - hibernate one frame
    // - call sub_3E8BA (shared deactivation gate)
    // - add `$4000` to y_vel(a3)
    // - branch back and repeat.
    NinjaCollisionStateLoc3D6C8 {
        y_velocity: y_velocity.wrapping_add(0x4000),
        hibernates_one_frame: true,
        checked_sub_3e8ba: true,
        loops: true,
    }
}

fn step_ninja_stealth_cadence_loc_3d942(enemy: &mut ActiveEnemy) -> bool {
    let cooldown = ninja_attack_cooldown_tick_loc_3d942(enemy.shared_walker_patrol_timer);
    enemy.shared_walker_patrol_timer = cooldown.attack_timer;
    if cooldown.route == NinjaAttackCooldownRouteLoc3D942::ContinuePacing {
        return false;
    }

    let throw_frame = ninja_walk_throw_frame_phase_loc_3d958(enemy.wait_timer);
    let animation_timer = if throw_frame { 3 } else { 0 };
    match ninja_throw_frame_gate_loc_3d958(throw_frame, animation_timer) {
        NinjaThrowFrameGateRouteLoc3D958::ContinuePacing { attack_timer } => {
            enemy.shared_walker_patrol_timer = attack_timer;
            false
        }
        NinjaThrowFrameGateRouteLoc3D958::RotateAttackPeriod => {
            let short_bytes = NINJA_ATTACK_PERIOD_SHORT_LOC_3D960.to_be_bytes();
            let long_bytes = NINJA_ATTACK_PERIOD_LONG_LOC_3D960.to_be_bytes();
            let attack_period_pair = if enemy.shared_walker_patrol_short_next {
                u32::from_be_bytes([short_bytes[0], short_bytes[1], long_bytes[0], long_bytes[1]])
            } else {
                u32::from_be_bytes([long_bytes[0], long_bytes[1], short_bytes[0], short_bytes[1]])
            };
            let rotation = ninja_attack_period_rotate_loc_3d960(
                attack_period_pair,
                enemy.shared_walker_state as i8,
            );
            enemy.shared_walker_patrol_timer = rotation.attack_timer;
            enemy.shared_walker_patrol_short_next = !enemy.shared_walker_patrol_short_next;

            if rotation.route == NinjaAttackPeriodRotationRouteLoc3D960::ContinuePacing {
                return false;
            }

            let jump = ninja_jump_attack_start_loc_3d9a4(matches!(enemy.facing, Direction::Right));
            enemy.ninja_state = NinjaState::JumpAttack;
            enemy.x_velocity = jump.x_velocity;
            enemy.y_velocity = jump.y_velocity;
            enemy.on_ground = false;
            true
        }
    }
}

/// Step a Ninja enemy: stealth-paces until player enters detection cone,
/// then faces player and jump-attacks. Returns to stealth on landing.
pub(crate) fn step_ninja(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_form: PlayerForm,
    maniaxe_projectile_minus_ninja_x_px: Option<i16>,
) {
    match enemy.ninja_state {
        NinjaState::Stealth => {
            let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
                // Pace like normal
                step_pacer(enemy, block_grid, None);
                return;
            };
            let kid_minus_ninja_x = ninja_position_word_delta_px_loc_3d706(px, enemy.x_pos);
            let kid_minus_ninja_y = ninja_position_word_delta_px_loc_3d706(py, enemy.y_pos);
            let dx = i64::from(kid_minus_ninja_x);
            let dy = -i64::from(kid_minus_ninja_y);

            // ASM Ninja.asm loc_3D706 checks the overhead threat window before
            // loc_3D734's later Red Stealth guard-pose branch, then enters the
            // positive `$4C(a5)` loc_3D84E pose rather than launching directly.
            if ninja_detects_player_while_stealthed(dx, dy) {
                let threat = ninja_jump_threat_pose_loc_3d84e(kid_minus_ninja_x);
                enemy.ninja_state = NinjaState::JumpThreat;
                enemy.shared_walker_state = i16::from(threat.threat_state_4c);
                enemy.walk_speed_override = threat.walk_speed_override;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                enemy.facing = if threat.x_direction_right {
                    Direction::Right
                } else {
                    Direction::Left
                };
                return;
            }

            if matches!(
                ninja_kid_threat_route_loc_3d734(
                    kid_minus_ninja_x,
                    kid_minus_ninja_y,
                    player_form,
                    false,
                ),
                NinjaKidThreatRouteLoc3D734::GuardPose
            ) {
                let guard = ninja_guard_pose_loc_3d826(kid_minus_ninja_x);
                enemy.ninja_state = NinjaState::Guard;
                enemy.shared_walker_state = i16::from(guard.threat_state_4c);
                enemy.walk_speed_override = guard.walk_speed_override;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                enemy.facing = if guard.x_direction_right {
                    Direction::Right
                } else {
                    Direction::Left
                };
                return;
            }

            if matches!(
                ninja_projectile_threat_route_loc_3d776(
                    maniaxe_projectile_minus_ninja_x_px,
                    player_form,
                ),
                NinjaProjectileThreatRouteLoc3D776::GuardPose
            ) {
                let guard =
                    ninja_guard_pose_loc_3d826(maniaxe_projectile_minus_ninja_x_px.unwrap_or(0));
                enemy.ninja_state = NinjaState::Guard;
                enemy.shared_walker_state = i16::from(guard.threat_state_4c);
                enemy.walk_speed_override = guard.walk_speed_override;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                enemy.facing = if guard.x_direction_right {
                    Direction::Right
                } else {
                    Direction::Left
                };
                return;
            }

            // ASM Ninja.asm loc_3D92A / loc_3D978 / loc_3D98A: while pacing,
            // turn toward Kid when he is outside the inclusive +/-$80 px band.
            let far_kid_facing = ninja_far_kid_facing_loc_3d92a(
                kid_minus_ninja_x,
                matches!(enemy.facing, Direction::Right),
            );
            enemy.facing = if far_kid_facing.x_direction_right {
                Direction::Right
            } else {
                Direction::Left
            };

            if step_ninja_stealth_cadence_loc_3d942(enemy) {
                return;
            }
            enemy.wait_timer = ninja_advance_walk_phase_loc_3d4ea(enemy.wait_timer);

            // Pace like normal
            step_pacer(enemy, block_grid, None);
        }
        NinjaState::Guard => {
            let Some(px) = player_x_pos else {
                enemy.ninja_state = NinjaState::Stealth;
                enemy.shared_walker_state = 0;
                return;
            };
            let kid_minus_ninja_x = ninja_position_word_delta_px_loc_3d706(px, enemy.x_pos);
            match ninja_post_threat_route_loc_3d79c(-1, false, kid_minus_ninja_x, player_form) {
                NinjaPostThreatRouteLoc3D79C::ResumePacing => {
                    enemy.x_velocity = 0;
                    enemy.y_velocity = 0;
                }
                NinjaPostThreatRouteLoc3D79C::ResetStealthPacing => {
                    let reset = ninja_reset_stealth_pacing_loc_3d7ca(
                        ninja_saved_walk_speed_loc_3d7ca(enemy.enemy_level),
                        matches!(enemy.facing, Direction::Right),
                    );
                    enemy.ninja_state = NinjaState::Stealth;
                    enemy.shared_walker_state = i16::from(reset.threat_state_4c);
                    enemy.walk_speed_override = reset.walk_speed_override;
                }
                NinjaPostThreatRouteLoc3D79C::PositiveStateCheck => {}
            }
        }
        NinjaState::JumpThreat => {
            let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
                let reset = ninja_reset_stealth_pacing_loc_3d7ca(
                    ninja_saved_walk_speed_loc_3d7ca(enemy.enemy_level),
                    matches!(enemy.facing, Direction::Right),
                );
                enemy.ninja_state = NinjaState::Stealth;
                enemy.shared_walker_state = i16::from(reset.threat_state_4c);
                enemy.walk_speed_override = reset.walk_speed_override;
                return;
            };

            let kid_minus_ninja_x = ninja_position_word_delta_px_loc_3d706(px, enemy.x_pos);
            let kid_minus_ninja_y = ninja_position_word_delta_px_loc_3d706(py, enemy.y_pos);
            match ninja_positive_state_route_loc_3d7fa(
                kid_minus_ninja_x,
                kid_minus_ninja_y,
                matches!(enemy.facing, Direction::Right),
            ) {
                NinjaPositiveStateRouteLoc3D7FA::ResumePacing { .. } => {
                    // ASM Ninja.asm loc_3D7FA -> loc_3D7EE keeps `$4C(a5)`
                    // positive and `$50(a5)` zero while Kid remains above.
                    enemy.x_velocity = 0;
                    enemy.y_velocity = 0;
                }
                NinjaPositiveStateRouteLoc3D7FA::ResetStealthPacing { .. } => {
                    let reset = ninja_reset_stealth_pacing_loc_3d7ca(
                        ninja_saved_walk_speed_loc_3d7ca(enemy.enemy_level),
                        matches!(enemy.facing, Direction::Right),
                    );
                    enemy.ninja_state = NinjaState::Stealth;
                    enemy.shared_walker_state = i16::from(reset.threat_state_4c);
                    enemy.walk_speed_override = reset.walk_speed_override;
                }
            }
        }
        NinjaState::AirborneKnockdown => {
            let Some(grid) = block_grid else {
                enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
                let fall = ninja_knockdown_landing_loop_loc_3d8e6(
                    0,
                    enemy.x_velocity,
                    enemy.y_velocity,
                    enemy.y_pos,
                );
                enemy.x_velocity = fall.x_velocity;
                enemy.y_velocity = fall.y_velocity;
                return;
            };

            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
            let cx = ninja_position_integer_px_loc_3d878(enemy.x_pos);
            let cy = ninja_position_integer_px_loc_3d878(enemy.y_pos);
            let floor = terrain::probe_floor(grid, cx, cy);
            let collision_type = ninja_floor_collision_word_loc_3d8e6(floor, enemy.y_pos);
            let fall = ninja_knockdown_landing_loop_loc_3d8e6(
                collision_type,
                enemy.x_velocity,
                enemy.y_velocity,
                enemy.y_pos,
            );
            enemy.x_velocity = fall.x_velocity;
            enemy.y_velocity = fall.y_velocity;
            enemy.y_pos = fall.y_position;
            match fall.route {
                NinjaKnockdownLandingRouteLoc3D8E6::Loop => {
                    enemy.on_ground = false;
                }
                NinjaKnockdownLandingRouteLoc3D8E6::KillPath => {
                    // ASM Ninja.asm loc_3D8E6 reaches loc_3DB4E only after
                    // floor contact, then hibernates through `stru_3D50A`.
                    enemy.on_ground = true;
                    enemy.hit_points = 0;
                    enemy.death_animation_active = true;
                    enemy.wait_timer = NINJA_DEATH_ANIMATION_FRAMES;
                }
            }
        }
        NinjaState::KidAboveKnockdown => {
            let Some(grid) = block_grid else {
                enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
                let fall = ninja_kid_above_landing_loop_loc_3dabc(
                    0,
                    enemy.x_velocity,
                    enemy.y_velocity,
                    enemy.y_pos,
                );
                enemy.x_velocity = fall.x_velocity;
                enemy.y_velocity = fall.y_velocity;
                return;
            };

            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
            let cx = ninja_position_integer_px_loc_3d878(enemy.x_pos);
            let cy = ninja_position_integer_px_loc_3d878(enemy.y_pos);
            let floor = terrain::probe_floor(grid, cx, cy);
            let collision_type = ninja_floor_collision_word_loc_3d8e6(floor, enemy.y_pos);
            let fall = ninja_kid_above_landing_loop_loc_3dabc(
                collision_type,
                enemy.x_velocity,
                enemy.y_velocity,
                enemy.y_pos,
            );
            enemy.x_velocity = fall.x_velocity;
            enemy.y_velocity = fall.y_velocity;
            enemy.y_pos = fall.y_position;
            match fall.route {
                NinjaKidAboveLandingLoopRouteLoc3DABC::Loop => {
                    enemy.on_ground = false;
                }
                NinjaKidAboveLandingLoopRouteLoc3DABC::To3DB4E => {
                    // ASM Ninja.asm loc_3DABC reaches loc_3DB4E only after
                    // floor contact, then hibernates through `stru_3D50A`.
                    enemy.on_ground = true;
                    enemy.hit_points = 0;
                    enemy.death_animation_active = true;
                    enemy.wait_timer = NINJA_DEATH_ANIMATION_FRAMES;
                }
            }
        }
        NinjaState::JumpAttack => {
            let airborne_tick = ninja_jump_attack_airborne_tick_loc_3d9da(enemy.y_velocity);
            // ASM Ninja.asm loc_3D9DA: `addi.l #$2800,y_vel(a3)` is uncapped;
            // the shared native max-fall clamp belongs to other approximated paths.
            enemy.y_velocity = airborne_tick.y_velocity;

            // Move
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let Some(grid) = block_grid else {
                return;
            };
            let cx = ninja_position_integer_px_loc_3d878(enemy.x_pos);
            let cy = ninja_position_integer_px_loc_3d878(enemy.y_pos);

            // Wall bounce: reverse x on wall hit
            let moving_right = ninja_jump_attack_moving_right_loc_3da46(enemy.x_velocity);
            let wall = terrain::probe_wall(
                grid,
                cx,
                cy,
                ENEMY_PROBE_HALF_WIDTH,
                ENEMY_PROBE_HEIGHT,
                moving_right,
            );
            if wall.hit {
                let bounce = ninja_jump_attack_wall_bounce_loc_3da46(
                    enemy.x_velocity,
                    matches!(enemy.facing, Direction::Right),
                );
                enemy.x_velocity = bounce.x_velocity;
                enemy.facing = if bounce.x_direction_right {
                    Direction::Right
                } else {
                    Direction::Left
                };
            }

            let ceiling =
                terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
            if ceiling.hit && enemy.y_velocity.is_negative() {
                // ASM Ninja.asm loc_3DA0C dispatches `colid_ceiling` to
                // loc_3DAB0, which clears the collision word, negates y_vel,
                // and returns to the airborne loop.
                enemy.y_velocity = ninja_ceiling_contact_loc_3dab0(enemy.y_velocity).y_velocity;
            }

            // Floor landing: return to stealth
            if enemy.y_velocity > 0 {
                let floor = terrain::probe_floor(grid, cx, cy);
                if floor.on_ground {
                    let floor_y_fp = ninja_floor_y_fp_loc_3d8e6(floor.floor_y_px);
                    if enemy.y_pos >= floor_y_fp {
                        let landing = ninja_jump_attack_landing_loc_3da56(matches!(
                            enemy.facing,
                            Direction::Right
                        ));
                        enemy.y_pos = floor_y_fp;
                        enemy.x_velocity = landing.x_velocity;
                        enemy.y_velocity = landing.y_velocity;
                        enemy.on_ground = !landing.is_moved;
                        enemy.ninja_state = NinjaState::Stealth;
                        debug_assert_eq!(landing.next_collision_type, 0);
                        debug_assert_eq!(landing.animation_script, "stru_3D4EA");
                        debug_assert!(!landing.has_level_collision);
                        debug_assert!(landing.is_animated);
                        match landing.next {
                            NinjaJumpAttackLandingResumeLoc3DA56::Loc3E566 => {
                                debug_assert_eq!(enemy.facing, Direction::Left);
                            }
                            NinjaJumpAttackLandingResumeLoc3DA56::Loc3E3B2 => {
                                debug_assert_eq!(enemy.facing, Direction::Right);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    fn ninja_test_grid(
        width: u16,
        height: u16,
        cells: &[(u16, u16, u8, u8)],
    ) -> terrain::BlockGrid {
        let mut grid = terrain::BlockGrid::from_level_layout(
            &vec![0; width as usize * height as usize * 2],
            width,
            height,
        );
        for &(col, row, high, low) in cells {
            grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(high, low));
        }
        grid
    }

    #[test]
    fn ninja_init_state_matches_loc_3d554() {
        // ASM Enemy11_Ninja_Init `loc_3D554` sets object_meta and current_hp from
        // layout level; loc_3D59A/3D5AA then shape the movement speed tier.
        let low_hp = ninja_init_state_loc_3d554(0);
        assert_eq!(low_hp.current_hp, 2);
        assert!(low_hp.status_flag_13);
        assert_eq!(low_hp.object_meta, 0x0011);
        assert_eq!(low_hp.priority, 0);
        // ASM `cmpi.w #3,d7; blt.s loc_3D5AA`: HP 2 skips both upper adds and
        // takes only the final `+$8000`. Oracle witness: 0x56 settle probe,
        // HP-2 Ninja x_pos steps `-$8000`/frame.
        assert_eq!(low_hp.walk_speed_override, 0x0000_8000);

        let mid_hp = ninja_init_state_loc_3d554(1);
        assert_eq!(mid_hp.current_hp, 3);
        // ASM `beq.s loc_3D59A`: HP 3 takes the middle and final adds.
        assert_eq!(mid_hp.walk_speed_override, 0x0001_0000);
    }

    #[test]
    fn ninja_init_speed_follows_loc_3d59a_branch() {
        // ASM: HP > 3 falls through the `move.l #$8000,$50(a5)` seed, then
        // loc_3D59A and loc_3D5AA each add `$8000` — three adds total.
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(4),
            0x0001_8000
        );
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(5),
            0x0001_8000
        );
    }

    #[test]
    fn ninja_init_speed_follows_loc_3d5aa_fallthrough() {
        // ASM `cmpi.w #3,d7`: `blt.s loc_3D5AA` (HP 2, one add) versus
        // `beq.s loc_3D59A` (HP 3, two adds) — the same accumulation shape
        // Driller.asm loc_3BD6C/loc_3BD74 uses.
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(2),
            0x0000_8000
        );
        assert_eq!(
            ninja_walk_speed_override_loc_3d59a_and_3d5aa(3),
            0x0001_0000
        );
    }

    #[test]
    fn ninja_position_word_delta_ignores_fractional_halves_loc_3d706() {
        assert_eq!(
            ninja_position_word_delta_px_loc_3d706(0x006F_F000, 0x0060_1000),
            0x000F,
            "ASM Ninja.asm loc_3D706 compares the high position words, so fractional halves do not round the delta up"
        );
        assert_eq!(
            ninja_position_word_delta_px_loc_3d706(0x0050_1000, 0x0060_F000),
            -0x0010,
            "ASM Ninja.asm loc_3D706 subtracts ninja_x from Kid/projectile x after integer-word extraction"
        );
    }

    #[test]
    fn ninja_position_word_delta_wraps_sub_w_loc_3d706() {
        assert_eq!(
            ninja_position_word_delta_px_loc_3d706(0x0000_0000, 0xFFFF_0000),
            1,
            "ASM Ninja.asm loc_3D706 uses SUB.W, so 0 - $FFFF wraps to +1 rather than saturating at the signed boundary",
        );
    }

    #[test]
    fn ninja_position_integer_px_uses_high_word_loc_3d878() {
        assert_eq!(
            ninja_position_integer_px_loc_3d878(0x0030_F000),
            0x0030,
            "ASM Ninja.asm terrain probes use the integer position word, so fractional halves do not affect probe coordinates"
        );
    }

    #[test]
    fn ninja_floor_collision_word_requires_ground_and_reached_floor_loc_3d8e6() {
        let floor = terrain::FloorProbe {
            on_ground: true,
            floor_y_px: 0x30,
            cell: terrain::Cell::from_be_bytes(0x60, 0x00),
            cell_x: 2,
            cell_y: 3,
        };
        assert_eq!(
            ninja_floor_y_fp_loc_3d8e6(0x30),
            0x0030_0000,
            "ASM Ninja.asm loc_3D8E6/loc_3DABC floor checks compare against the floor integer word in FP16.16 form"
        );
        assert_eq!(
            ninja_floor_collision_word_loc_3d8e6(floor, 0x0030_0000),
            NINJA_COLID_FLOOR,
            "ASM Ninja.asm loc_3D8E6 should emit colid_floor once the moved y_pos reaches the solid floor"
        );
        assert_eq!(
            ninja_floor_collision_word_loc_3d8e6(
                terrain::FloorProbe {
                    on_ground: false,
                    ..floor
                },
                0x0030_0000,
            ),
            0,
            "ASM Ninja.asm loc_3D8E6 needs an actual floor probe hit; y_pos alone cannot synthesize colid_floor"
        );
        assert_eq!(
            ninja_floor_collision_word_loc_3d8e6(floor, 0x002F_FFFF),
            0,
            "ASM Ninja.asm loc_3D8E6 should keep looping until y_pos reaches the floor word"
        );
    }

    #[test]
    fn ninja_jump_attack_moving_right_includes_zero_loc_3da46() {
        assert!(ninja_jump_attack_moving_right_loc_3da46(0));
        assert!(ninja_jump_attack_moving_right_loc_3da46(1));
        assert!(
            !ninja_jump_attack_moving_right_loc_3da46(-1),
            "ASM Ninja.asm loc_3DA46 wall dispatch treats zero x velocity as non-negative for the right-side probe"
        );
    }

    #[test]
    fn ninja_stealth_detection_excludes_positive_0x10_edge_per_loc_3d706() {
        let mut enemy = ninja_test_enemy();
        let player_x = (enemy.x_pos as i64 + ((NINJA_DETECT_X_RANGE as i64) << 16)) as u32;
        let player_y = (enemy.y_pos as i64 - ((NINJA_DETECT_Y_MIN as i64) << 16)) as u32;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(enemy.ninja_state, NinjaState::Stealth);
        assert_eq!(
            enemy.x_velocity, 0,
            "ASM Ninja.asm loc_3D706 only enters the jump branch when kid_x - ninja_x < $10; the +$10 edge must stay in stealth"
        );
    }

    #[test]
    fn ninja_live_stealth_detection_wraps_position_words_loc_3d706() {
        let mut enemy = ninja_test_enemy();
        enemy.x_pos = 0x0100_0000;
        enemy.y_pos = 0xFFFF_0000;
        let player_x = enemy.x_pos;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(0x0000_0000),
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "ASM Ninja.asm loc_3D706 wraps Kid Y - Ninja Y to +1 before the signed threat-window comparisons",
        );
    }

    #[test]
    fn ninja_live_stealth_detection_includes_negative_edge_per_loc_3d706() {
        let mut enemy = ninja_test_enemy();
        let player_x = (enemy.x_pos as i64 - ((NINJA_DETECT_X_RANGE as i64) << 16)) as u32;
        let player_y = (enemy.y_pos as i64 - ((NINJA_DETECT_Y_MIN as i64) << 16)) as u32;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (enemy.ninja_state, enemy.shared_walker_state, enemy.facing),
            (NinjaState::JumpThreat, 1, Direction::Left),
            "ASM Ninja.asm loc_3D706 includes kid_x - ninja_x == -$10 and loc_3D84E keeps x_direction clear for negative d4"
        );
    }

    #[test]
    fn ninja_live_red_stealth_guard_uses_word_deltas_loc_3d734_3d826() {
        let mut enemy = ninja_test_enemy();
        let player_x = enemy.x_pos + (0x30 << 16);
        let player_y = enemy.y_pos - (4 << 16);

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::RedStealth,
            None,
        );

        assert_eq!(
            (enemy.ninja_state, enemy.shared_walker_state, enemy.facing),
            (NinjaState::Guard, -1, Direction::Right),
            "ASM Ninja.asm loc_3D734 uses kid_x - ninja_x and kid_y - ninja_y words; the +$30/-4 Red Stealth edge enters loc_3D826 facing right"
        );
    }

    #[test]
    fn ninja_live_maniaxe_projectile_guard_uses_projectile_dx_loc_3d776_3d826() {
        let mut enemy = ninja_test_enemy();
        let player_x = enemy.x_pos;
        let player_y = enemy.y_pos;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::Maniaxe,
            Some(0x59),
        );

        assert_eq!(
            (enemy.ninja_state, enemy.shared_walker_state, enemy.facing),
            (NinjaState::Guard, -1, Direction::Right),
            "ASM Ninja.asm loc_3D776 uses projectile_x - ninja_x; +$59 is inside the half-open Maniaxe guard band and loc_3D826 faces right"
        );
    }

    #[test]
    fn ninja_live_guard_state_resets_when_red_stealth_leaves_loc_3d79c() {
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::Guard;
        enemy.shared_walker_state = -1;
        enemy.facing = Direction::Right;
        let player_x = enemy.x_pos + (0x31 << 16);
        let player_y = enemy.y_pos;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::RedStealth,
            None,
        );

        assert_eq!(
            (
                enemy.ninja_state,
                enemy.shared_walker_state,
                enemy.walk_speed_override
            ),
            (
                NinjaState::Stealth,
                0,
                pacer_walk_speed(enemy.family, enemy.hit_points)
            ),
            "ASM Ninja.asm loc_3D79C branches to loc_3D7CA once Red Stealth moves beyond the inclusive +$30 guard persistence band"
        );
    }

    #[test]
    fn damaged_level_three_ninja_restores_saved_speed_loc_3d7ca() {
        let mut enemy = ninja_test_enemy();
        enemy.enemy_level = 2;
        enemy.hit_points = 1;
        enemy.ninja_state = NinjaState::Guard;
        enemy.shared_walker_state = -1;
        enemy.walk_speed_override = 0;
        enemy.facing = Direction::Right;
        let player_x = enemy.x_pos + (0x31 << 16);
        let player_y = enemy.y_pos;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::RedStealth,
            None,
        );
        assert_eq!(
            enemy.walk_speed_override,
            ninja_init_state_loc_3d554(2).walk_speed_override,
            "ASM Ninja.asm loc_3D670/loc_3D7CA restores the level-three `$18000` speed saved in `$5E(a5)`, so damage does not reduce resumed pacing"
        );
    }

    #[test]
    fn ninja_live_jump_threat_state_resets_when_kid_drops_below_loc_3d7fa() {
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::JumpThreat;
        enemy.shared_walker_state = 1;
        enemy.facing = Direction::Left;
        let player_x = enemy.x_pos;
        let player_y = enemy.y_pos;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (
                enemy.ninja_state,
                enemy.shared_walker_state,
                enemy.walk_speed_override
            ),
            (
                NinjaState::Stealth,
                0,
                pacer_walk_speed(enemy.family, enemy.hit_points)
            ),
            "ASM Ninja.asm loc_3D7FA branches to loc_3D7CA when kid_y - ninja_y is zero instead of negative"
        );
    }

    #[test]
    fn ninja_live_airborne_knockdown_no_grid_moves_before_loc_3d8e6_loop() {
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::AirborneKnockdown;
        enemy.y_pos = 0x0020_0000;
        enemy.y_velocity = 0x0001_0000;

        step_ninja(&mut enemy, None, None, None, PlayerForm::TheKid, None);

        assert_eq!(
            (enemy.y_pos, enemy.y_velocity, enemy.hit_points),
            (0x0021_0000, 0x0001_1800, 2),
            "ASM Ninja.asm loc_3D8E6 moves by current y_vel before the non-floor loop adds `$1800`; without a floor collision it must not enter loc_3DB4E"
        );
    }

    #[test]
    fn ninja_live_kid_above_knockdown_no_grid_moves_before_loc_3dabc_loop() {
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::KidAboveKnockdown;
        enemy.y_pos = 0x0028_0000;
        enemy.y_velocity = 0x0001_0000;

        step_ninja(&mut enemy, None, None, None, PlayerForm::TheKid, None);

        assert_eq!(
            (enemy.y_pos, enemy.y_velocity, enemy.hit_points),
            (0x0029_0000, 0x0001_1800, 2),
            "ASM Ninja.asm loc_3DABC moves by current y_vel before the non-floor loop adds `$1800`; without a floor collision it must not enter loc_3DB4E"
        );
    }

    #[test]
    fn ninja_live_airborne_knockdown_floor_contact_enters_death_loc_3d8e6() {
        let grid = ninja_test_grid(8, 8, &[(2, 3, 0x60, 0x00)]);
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::AirborneKnockdown;
        enemy.x_pos = 0x0020_0000;
        enemy.y_pos = 0x002F_0000;
        enemy.y_velocity = 0x0001_0000;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (
                enemy.on_ground,
                enemy.hit_points,
                enemy.death_animation_active,
                enemy.wait_timer
            ),
            (true, 0, true, NINJA_DEATH_ANIMATION_FRAMES),
            "ASM Ninja.asm loc_3D8E6 reaches loc_3DB4E only when the moved position has an actual floor collision"
        );
    }

    #[test]
    fn ninja_live_airborne_knockdown_empty_grid_does_not_fake_floor_loc_3d8e6() {
        let grid = ninja_test_grid(8, 8, &[]);
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::AirborneKnockdown;
        enemy.x_pos = 0x0020_0000;
        enemy.y_pos = 0x002F_0000;
        enemy.y_velocity = 0x0001_0000;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (enemy.on_ground, enemy.hit_points, enemy.y_velocity),
            (false, 2, 0x0001_1800),
            "ASM Ninja.asm loc_3D8E6 needs the floor collision word; being at or past a would-be floor Y without a solid cell must keep looping"
        );
    }

    #[test]
    fn ninja_live_kid_above_knockdown_floor_contact_enters_death_loc_3dabc() {
        let grid = ninja_test_grid(8, 8, &[(2, 3, 0x60, 0x00)]);
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::KidAboveKnockdown;
        enemy.x_pos = 0x0020_0000;
        enemy.y_pos = 0x002F_0000;
        enemy.y_velocity = 0x0001_0000;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (
                enemy.on_ground,
                enemy.hit_points,
                enemy.death_animation_active,
                enemy.wait_timer
            ),
            (true, 0, true, NINJA_DEATH_ANIMATION_FRAMES),
            "ASM Ninja.asm loc_3DABC reaches loc_3DB4E only when the moved position has an actual floor collision"
        );
    }

    #[test]
    fn ninja_live_kid_above_knockdown_empty_grid_does_not_fake_floor_loc_3dabc() {
        let grid = ninja_test_grid(8, 8, &[]);
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::KidAboveKnockdown;
        enemy.x_pos = 0x0020_0000;
        enemy.y_pos = 0x002F_0000;
        enemy.y_velocity = 0x0001_0000;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (enemy.on_ground, enemy.hit_points, enemy.y_velocity),
            (false, 2, 0x0001_1800),
            "ASM Ninja.asm loc_3DABC needs the floor collision word; being at or past a would-be floor Y without a solid cell must keep looping"
        );
    }

    #[test]
    fn ninja_live_jump_attack_no_grid_moves_after_gravity_loc_3d9da() {
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::JumpAttack;
        enemy.x_pos = 0x0020_0000;
        enemy.y_pos = 0x0020_0000;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;

        step_ninja(&mut enemy, None, None, None, PlayerForm::TheKid, None);

        assert_eq!(
            (
                enemy.x_pos,
                enemy.y_pos,
                enemy.y_velocity,
                enemy.ninja_state
            ),
            (
                0x0021_0000,
                0x0020_2800,
                0x0000_2800,
                NinjaState::JumpAttack
            ),
            "ASM Ninja.asm loc_3D9DA applies `$2800` gravity before the live jump-attack movement step and loops without terrain"
        );
    }

    #[test]
    fn ninja_live_jump_attack_empty_grid_does_not_reflect_ceiling_loc_3dab0() {
        let grid = ninja_test_grid(8, 8, &[]);
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::JumpAttack;
        enemy.x_pos = 0x0020_0000;
        enemy.y_pos = 0x0030_0000;
        enemy.x_velocity = 0;
        enemy.y_velocity = -0x0001_0000;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (enemy.y_velocity, enemy.ninja_state),
            (-0x0000_D800, NinjaState::JumpAttack),
            "ASM Ninja.asm loc_3DAB0 only reflects y_vel when the ceiling collision word is present; upward motion alone keeps the airborne loop"
        );
    }

    #[test]
    fn ninja_live_jump_attack_zero_y_velocity_does_not_land_loc_3da56() {
        let grid = ninja_test_grid(8, 8, &[(2, 3, 0x60, 0x00)]);
        let mut enemy = ninja_test_enemy();
        enemy.ninja_state = NinjaState::JumpAttack;
        enemy.x_pos = 0x0020_0000;
        enemy.y_pos = 0x0030_0000;
        enemy.x_velocity = 0;
        enemy.y_velocity = -0x0000_2800;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            (enemy.y_velocity, enemy.ninja_state),
            (0, NinjaState::JumpAttack),
            "ASM Ninja.asm loc_3DA0C should not enter the floor landing path unless y_vel is positive after gravity"
        );
    }

    #[test]
    fn ninja_kid_threat_route_loc_3d734_matches_charge_and_red_stealth_gates() {
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x50, -5, PlayerForm::TheKid, true),
            NinjaKidThreatRouteLoc3D734::BerzerkerKillPath,
            "ASM Ninja.asm loc_3D734 checks Berzerker_charging inside the inclusive +/-$50 Kid X band before the tighter Red Stealth guard gates"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x51, 0, PlayerForm::TheKid, true),
            NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck,
            "ASM Ninja.asm loc_3D734 should skip the Berzerker kill path when kid_x - ninja_x is greater than +$50"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x30, -4, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::GuardPose,
            "ASM Ninja.asm loc_3D734 should enter the guard pose at the inclusive +$30 Red Stealth edge when Kid is not more than 4 px above"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0x31, -4, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck,
            "ASM Ninja.asm loc_3D734 should continue to the projectile-threat check outside the tighter +/-$30 Red Stealth guard band"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(-0x30, -4, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::GuardPose,
            "ASM Ninja.asm loc_3D734 keeps the negative -$30 Red Stealth edge inside the guard band"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(-0x31, -4, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck,
            "ASM Ninja.asm loc_3D734 branches to loc_3D776 when kid_x - ninja_x is less than -$30"
        );
        assert_eq!(
            ninja_kid_threat_route_loc_3d734(0, -5, PlayerForm::RedStealth, false),
            NinjaKidThreatRouteLoc3D734::ProjectileThreatCheck,
            "ASM Ninja.asm loc_3D734 should require kid_y - ninja_y >= -4 before entering the guard pose"
        );
    }

    #[test]
    fn ninja_projectile_threat_route_loc_3d776_matches_maniaxe_half_open_band() {
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(None, PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat,
            "ASM Ninja.asm loc_3D776 should ignore the projectile threat gate when Addr_GfxObject_KidProjectile is zero"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(0), PlayerForm::RedStealth),
            NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat,
            "ASM Ninja.asm loc_3D776 should require Current_Helmet == Maniaxe before checking projectile X range"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(-0x5A), PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::GuardPose,
            "ASM Ninja.asm loc_3D776 should include the left projectile edge because d4 < -$5A is the only left-side reject"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(0x59), PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::GuardPose,
            "ASM Ninja.asm loc_3D776 should include +$59 because the right compare branches while d4 < +$5A"
        );
        assert_eq!(
            ninja_projectile_threat_route_loc_3d776(Some(0x5A), PlayerForm::Maniaxe),
            NinjaProjectileThreatRouteLoc3D776::ContinuePostThreat,
            "ASM Ninja.asm loc_3D776 should reject +$5A exactly, making the right edge exclusive"
        );
    }

    #[test]
    fn ninja_post_threat_route_loc_3d79c_matches_signed_state_and_guard_persistence() {
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(0, false, 0, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::ResumePacing,
            "ASM Ninja.asm loc_3D79C should keep normal pacing when `$4C(a5)` is zero"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(1, false, 0, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::PositiveStateCheck,
            "ASM Ninja.asm loc_3D79C should treat positive `$4C(a5)` as the airborne proximity branch"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, true, 0x7F, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::ResumePacing,
            "ASM Ninja.asm loc_3D79C should keep negative guard state while Addr_GfxObject_KidProjectile is non-zero"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, 0x30, PlayerForm::RedStealth),
            NinjaPostThreatRouteLoc3D79C::ResumePacing,
            "ASM Ninja.asm loc_3D79C should keep Red Stealth guard state at the inclusive +$30 Kid X edge"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, -0x30, PlayerForm::RedStealth),
            NinjaPostThreatRouteLoc3D79C::ResumePacing,
            "ASM Ninja.asm loc_3D79C should keep Red Stealth guard state at the inclusive -$30 Kid X edge"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, 0x31, PlayerForm::RedStealth),
            NinjaPostThreatRouteLoc3D79C::ResetStealthPacing,
            "ASM Ninja.asm loc_3D79C should reset guard state when Red Stealth moves beyond +$30"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, -0x31, PlayerForm::RedStealth),
            NinjaPostThreatRouteLoc3D79C::ResetStealthPacing,
            "ASM Ninja.asm loc_3D79C should reset guard state when Red Stealth moves beyond -$30"
        );
        assert_eq!(
            ninja_post_threat_route_loc_3d79c(-1, false, 0, PlayerForm::TheKid),
            NinjaPostThreatRouteLoc3D79C::ResetStealthPacing,
            "ASM Ninja.asm loc_3D79C should reset negative guard state without a projectile unless Current_Helmet is Red Stealth"
        );
    }

    #[test]
    fn ninja_reset_stealth_pacing_loc_3d7ca_restores_walk_state_and_facing_route() {
        let left = ninja_reset_stealth_pacing_loc_3d7ca(0x0000_8000, false);
        assert_eq!(
            left,
            NinjaResetStealthPacingStateLoc3D7CA {
                threat_state_4c: 0,
                animation_script: "stru_3D4EA",
                is_animated: true,
                walk_speed_override: 0x0000_8000,
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E546,
            },
            "ASM Ninja.asm loc_3D7CA should clear `$4C(a5)`, restore `$5E(a5)` into `$50(a5)`, and send clear x_direction to loc_3E546"
        );

        let right = ninja_reset_stealth_pacing_loc_3d7ca(0x0001_0000, true);
        assert_eq!(
            right,
            NinjaResetStealthPacingStateLoc3D7CA {
                threat_state_4c: 0,
                animation_script: "stru_3D4EA",
                is_animated: true,
                walk_speed_override: 0x0001_0000,
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E392,
            },
            "ASM Ninja.asm loc_3D7CA should send set x_direction to loc_3E392 after restoring the walk state"
        );
    }

    #[test]
    fn ninja_resume_pacing_route_loc_3d7ee_dispatches_by_x_direction() {
        assert_eq!(
            ninja_resume_pacing_route_loc_3d7ee(false),
            NinjaResumePacingRouteLoc3D7EE::Loc3E560,
            "ASM Ninja.asm loc_3D7EE should send clear x_direction to loc_3E560"
        );
        assert_eq!(
            ninja_resume_pacing_route_loc_3d7ee(true),
            NinjaResumePacingRouteLoc3D7EE::Loc3E3AC,
            "ASM Ninja.asm loc_3D7EE should send set x_direction to loc_3E3AC"
        );
    }

    #[test]
    fn ninja_positive_state_route_loc_3d7fa_matches_proximity_box() {
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(-0x10, -0x60, false),
            NinjaPositiveStateRouteLoc3D7FA::ResumePacing {
                next: NinjaResumePacingRouteLoc3D7EE::Loc3E560,
            },
            "ASM Ninja.asm loc_3D7FA should include the left X edge and top Y edge before branching to loc_3D7EE"
        );
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(0x10, -1, true),
            NinjaPositiveStateRouteLoc3D7FA::ResumePacing {
                next: NinjaResumePacingRouteLoc3D7EE::Loc3E3AC,
            },
            "ASM Ninja.asm loc_3D7FA should include the +$10 X edge and require kid_y - ninja_y to stay negative"
        );
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(0, 0, false),
            NinjaPositiveStateRouteLoc3D7FA::ResetStealthPacing {
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E546,
            },
            "ASM Ninja.asm loc_3D7FA should branch to loc_3D7CA once Kid is level with or below the ninja"
        );
        assert_eq!(
            ninja_positive_state_route_loc_3d7fa(0x11, -1, true),
            NinjaPositiveStateRouteLoc3D7FA::ResetStealthPacing {
                next: NinjaResetStealthPacingRouteLoc3D7CA::Loc3E392,
            },
            "ASM Ninja.asm loc_3D7FA should branch to loc_3D7CA outside the inclusive +/-$10 X band"
        );
    }

    #[test]
    fn ninja_guard_pose_loc_3d826_sets_negative_guard_state_and_faces_by_d4() {
        assert_eq!(
            ninja_guard_pose_loc_3d826(-1),
            NinjaThreatPoseState {
                threat_state_4c: -1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D8E",
                walk_speed_override: 0,
                x_direction_right: false,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D826 should leave x_direction clear for negative d4 before branching to loc_3D79C"
        );

        assert_eq!(
            ninja_guard_pose_loc_3d826(0),
            NinjaThreatPoseState {
                threat_state_4c: -1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D8E",
                walk_speed_override: 0,
                x_direction_right: true,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D826 should set x_direction when d4 is zero or positive"
        );
    }

    #[test]
    fn ninja_jump_threat_pose_loc_3d84e_sets_positive_state_and_jump_sprite() {
        assert_eq!(
            ninja_jump_threat_pose_loc_3d84e(-1),
            NinjaThreatPoseState {
                threat_state_4c: 1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D86",
                walk_speed_override: 0,
                x_direction_right: false,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D84E should set `$4C(a5)` to positive 1 and keep x_direction clear for negative d4"
        );

        assert_eq!(
            ninja_jump_threat_pose_loc_3d84e(1),
            NinjaThreatPoseState {
                threat_state_4c: 1,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D86",
                walk_speed_override: 0,
                x_direction_right: true,
                next: NinjaThreatPoseRoute::Loc3D79C,
            },
            "ASM Ninja.asm loc_3D84E should set x_direction for non-negative d4 before loc_3D79C"
        );
    }

    #[test]
    fn ninja_airborne_collision_dispatch_loc_3d878_clears_and_routes_collision_types() {
        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(0, false, PlayerForm::TheKid, true),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::Resume {
                    next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
                },
            },
            "ASM Ninja.asm loc_3D878 should clear collision_type and route zero through loc_3D89C by x_direction"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(-1, true, PlayerForm::TheKid, true),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::To3DB4E,
            },
            "ASM Ninja.asm loc_3D878 should send negative collision IDs to loc_3DB4E"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDABOVE as i16,
                true,
                PlayerForm::TheKid,
                false
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::To3D8BC,
            },
            "ASM Ninja.asm loc_3D878 should send colid_kidabove directly to loc_3D8BC"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDRIGHT as i16,
                true,
                PlayerForm::TheKid,
                true
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::Resume {
                    next: NinjaCollisionResumeLoc3D638::Loc3E6AE,
                },
            },
            "ASM Ninja.asm loc_3D8A8 should require Current_Helmet == Berzerker before side contact can reach loc_3D8BC"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDLEFT as i16,
                false,
                PlayerForm::Berzerker,
                false
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::Resume {
                    next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
                },
            },
            "ASM Ninja.asm loc_3D8A8 should fall back to loc_3D89C when Kid's sprite offset is after LnkTo_unk_ADFC8"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(
                NINJA_COLID_KIDLEFT as i16,
                false,
                PlayerForm::Berzerker,
                true
            ),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::To3D8BC,
            },
            "ASM Ninja.asm loc_3D878 should allow Berzerker side contact through to loc_3D8BC when the Kid sprite threshold passes"
        );

        assert_eq!(
            ninja_airborne_collision_dispatch_loc_3d878(0x00FF, true, PlayerForm::Berzerker, true),
            NinjaAirborneCollisionStateLoc3D878 {
                next_collision_type: 0,
                route: NinjaAirborneCollisionRouteLoc3D878::Resume {
                    next: NinjaCollisionResumeLoc3D638::Loc3E6AE,
                },
            },
            "ASM Ninja.asm loc_3D878 should send unhandled positive collision IDs through loc_3D89C, not the Kid-side loc_3D8A8 gate"
        );
    }

    #[test]
    fn ninja_knockdown_start_loc_3d8d6_seeds_landing_loop() {
        assert_eq!(
            ninja_knockdown_start_loc_3d8d6(),
            NinjaKnockdownStartStateLoc3D8D6 {
                y_velocity: 0x0001_8000,
                is_moved: true,
                has_level_collision: true,
                next: NinjaKnockdownStartRouteLoc3D8D6::FallthroughLandingLoop,
            },
            "ASM Ninja.asm loc_3D8D6 should start the HP-depleted knockdown fall before the floor-contact wait"
        );
    }

    #[test]
    fn ninja_knockdown_landing_loop_loc_3d8e6_waits_for_floor_then_snaps_y() {
        assert_eq!(
            ninja_knockdown_landing_loop_loc_3d8e6(0, 0x0000_4000, 0x0001_8000, 0x0031_8000),
            NinjaKnockdownLandingStateLoc3D8E6 {
                next_collision_type: 0,
                x_velocity: 0x0000_4000,
                y_velocity: 0x0001_9800,
                y_position: 0x0031_8000,
                is_moved: true,
                has_level_collision: true,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                route: NinjaKnockdownLandingRouteLoc3D8E6::Loop,
            },
            "ASM Ninja.asm loc_3D8E6 should clear collision_type, add $1800 to y_vel, and loop while collision is not floor"
        );

        assert_eq!(
            ninja_knockdown_landing_loop_loc_3d8e6(
                NINJA_COLID_FLOOR,
                0x0000_4000,
                0x0001_8000,
                0x0031_8000,
            ),
            NinjaKnockdownLandingStateLoc3D8E6 {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                y_position: 0x0040_0000,
                is_moved: false,
                has_level_collision: false,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                route: NinjaKnockdownLandingRouteLoc3D8E6::KillPath,
            },
            "ASM Ninja.asm loc_3D8E6 should clear motion/flags and snap y_pos to ((integer_y + $F) & $FFF0) on floor contact"
        );
    }

    #[test]
    fn ninja_far_kid_facing_loc_3d92a_only_turns_outside_inclusive_band() {
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(0x80, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: false,
                decision: NinjaFarKidFacingDecisionLoc3D92A::WithinBand,
            },
            "ASM Ninja.asm loc_3D92A should keep current facing at the inclusive +$80 Kid X edge"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(-0x80, true),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: true,
                decision: NinjaFarKidFacingDecisionLoc3D92A::WithinBand,
            },
            "ASM Ninja.asm loc_3D92A should keep current facing at the inclusive -$80 Kid X edge"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(0x81, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: true,
                decision: NinjaFarKidFacingDecisionLoc3D92A::TurnTowardFarKid,
            },
            "ASM Ninja.asm loc_3D92A should set x_direction when Kid is more than $80 px to the right"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(-0x81, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: false,
                decision: NinjaFarKidFacingDecisionLoc3D92A::AlreadyFacingFarKid,
            },
            "ASM Ninja.asm loc_3D92A should leave clear x_direction unchanged when Kid is already far left"
        );
    }

    #[test]
    fn ninja_far_kid_facing_loc_3d978_turns_right_for_positive_far_kid() {
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(0x81, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: true,
                decision: NinjaFarKidFacingDecisionLoc3D92A::TurnTowardFarKid,
            },
            "ASM Ninja.asm loc_3D978 should toggle clear x_direction when Kid is more than $80 px to the right"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(0x81, true),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: true,
                decision: NinjaFarKidFacingDecisionLoc3D92A::AlreadyFacingFarKid,
            },
            "ASM Ninja.asm loc_3D978 should leave set x_direction unchanged when already facing a far-right Kid"
        );
    }

    #[test]
    fn ninja_far_kid_facing_loc_3d98a_turns_left_for_negative_far_kid() {
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(-0x81, true),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: false,
                decision: NinjaFarKidFacingDecisionLoc3D92A::TurnTowardFarKid,
            },
            "ASM Ninja.asm loc_3D98A should toggle set x_direction when Kid is more than $80 px to the left"
        );
        assert_eq!(
            ninja_far_kid_facing_loc_3d92a(-0x81, false),
            NinjaFarKidFacingStateLoc3D92A {
                x_direction_right: false,
                decision: NinjaFarKidFacingDecisionLoc3D92A::AlreadyFacingFarKid,
            },
            "ASM Ninja.asm loc_3D98A should leave clear x_direction unchanged when already facing a far-left Kid"
        );
    }

    #[test]
    fn ninja_attack_cooldown_tick_loc_3d942_branches_on_post_decrement_zero() {
        assert_eq!(
            ninja_attack_cooldown_tick_loc_3d942(2),
            NinjaAttackCooldownStateLoc3D942 {
                attack_timer: 1,
                route: NinjaAttackCooldownRouteLoc3D942::ContinuePacing,
            },
            "ASM Ninja.asm loc_3D942 should branch to loc_3D998 while `$54(a5)` is non-zero after decrement"
        );
        assert_eq!(
            ninja_attack_cooldown_tick_loc_3d942(1),
            NinjaAttackCooldownStateLoc3D942 {
                attack_timer: 0,
                route: NinjaAttackCooldownRouteLoc3D942::CheckThrowFrame,
            },
            "ASM Ninja.asm loc_3D942 should only fall through to the sprite gate when `$54(a5)` becomes zero"
        );
        assert_eq!(
            ninja_attack_cooldown_tick_loc_3d942(0),
            NinjaAttackCooldownStateLoc3D942 {
                attack_timer: 0xFFFF,
                route: NinjaAttackCooldownRouteLoc3D942::ContinuePacing,
            },
            "ASM Ninja.asm loc_3D942 uses word wrapping arithmetic, so zero underflows and branches to loc_3D998"
        );
    }

    #[test]
    fn ninja_spawn_attack_cadence_matches_init_loc_3d5aa() {
        assert_eq!(
            spawn_shared_walker_patrol_timer(EnemyFamily::Ninja),
            NINJA_ATTACK_TIMER_INIT_LOC_3D5AA,
            "ASM Ninja.asm loc_3D5AA should seed `$54(a5)` with $12C"
        );
        assert!(
            spawn_shared_walker_patrol_short_next(EnemyFamily::Ninja),
            "ASM Ninja.asm loc_3D5AA should seed `$6E(a5)` as $0032/$012C, making the first successful reload short"
        );
    }

    #[test]
    fn ninja_walk_throw_frame_phase_loc_3d958_matches_c7dbe_timer_three() {
        assert!(!ninja_walk_throw_frame_phase_loc_3d958(36));
        assert!(
            ninja_walk_throw_frame_phase_loc_3d958(37),
            "ASM ingame/anim/enemy/Ninja.asm stru_3D4EA should expose C7DBE with animation_timer(a3) == 3 on phase 37"
        );
        assert_eq!(
            ninja_advance_walk_phase_loc_3d4ea(39),
            0,
            "ASM ingame/anim/enemy/Ninja.asm stru_3D4EA has a 40-frame walk cycle"
        );
        assert_eq!(
            ninja_advance_walk_phase_loc_3d4ea(0),
            1,
            "ASM ingame/anim/enemy/Ninja.asm stru_3D4EA advances ordinary walk phases, not only the wraparound edge"
        );
    }

    #[test]
    fn ninja_stealth_cadence_rotates_rom_attack_period_pair_loc_3d942_3d960() {
        let mut enemy = ninja_test_enemy();
        enemy.shared_walker_patrol_timer = 1;
        enemy.wait_timer = NINJA_THROW_FRAME_PHASE_LOC_3D958;
        enemy.shared_walker_patrol_short_next = true;
        enemy.shared_walker_state = 0;

        assert!(
            step_ninja_stealth_cadence_loc_3d942(&mut enemy),
            "ASM Ninja.asm loc_3D942/loc_3D960 should start the jump attack when the timer expires on the exact throw frame with `$4C(a5)` clear"
        );
        assert_eq!(
            (
                enemy.shared_walker_patrol_timer,
                enemy.shared_walker_patrol_short_next,
                enemy.ninja_state,
                enemy.x_velocity,
                enemy.y_velocity,
            ),
            (
                NINJA_ATTACK_PERIOD_SHORT_LOC_3D960,
                false,
                NinjaState::JumpAttack,
                -0x0001_8000,
                -0x0004_8000,
            ),
            "ASM Ninja.asm loc_3D960 stores `$0032/$012C` as a word pair, swaps it, copies the new low word into `$54(a5)`, and then launches left when x_direction is clear"
        );
    }

    #[test]
    fn ninja_throw_frame_gate_loc_3d958_requires_exact_sprite_and_timer() {
        assert_eq!(
            ninja_throw_frame_gate_loc_3d958(false, 3),
            NinjaThrowFrameGateRouteLoc3D958::ContinuePacing { attack_timer: 1 },
            "ASM Ninja.asm loc_3D958 should reset `$54(a5)` to one frame when the sprite frame is not LnkTo_unk_C7DBE"
        );
        assert_eq!(
            ninja_throw_frame_gate_loc_3d958(true, 2),
            NinjaThrowFrameGateRouteLoc3D958::ContinuePacing { attack_timer: 1 },
            "ASM Ninja.asm loc_3D958 should reset `$54(a5)` to one frame until animation_timer(a3) reaches 3"
        );
        assert_eq!(
            ninja_throw_frame_gate_loc_3d958(true, 3),
            NinjaThrowFrameGateRouteLoc3D958::RotateAttackPeriod,
            "ASM Ninja.asm loc_3D958 should be skipped only for the exact C7DBE sprite frame with animation_timer(a3) == 3"
        );
    }

    #[test]
    fn ninja_attack_period_rotate_loc_3d960_swaps_period_words_and_checks_guard_state() {
        assert_eq!(
            ninja_attack_period_rotate_loc_3d960(0x0010_0030, 1),
            NinjaAttackPeriodRotationStateLoc3D960 {
                attack_period_pair: 0x0030_0010,
                attack_timer: 0x0010,
                route: NinjaAttackPeriodRotationRouteLoc3D960::ContinuePacing,
            },
            "ASM Ninja.asm loc_3D960 should rotate `$6E(a5)`, copy the new low word into `$54(a5)`, and branch to loc_3D998 when `$4C(a5)` is non-zero"
        );
        assert_eq!(
            ninja_attack_period_rotate_loc_3d960(0x0010_0030, -1),
            NinjaAttackPeriodRotationStateLoc3D960 {
                attack_period_pair: 0x0030_0010,
                attack_timer: 0x0010,
                route: NinjaAttackPeriodRotationRouteLoc3D960::ContinuePacing,
            },
            "ASM Ninja.asm loc_3D960 uses tst.b on `$4C(a5)`, so negative guard state is still non-zero and keeps pacing"
        );
        assert_eq!(
            ninja_attack_period_rotate_loc_3d960(0x0010_0030, 0),
            NinjaAttackPeriodRotationStateLoc3D960 {
                attack_period_pair: 0x0030_0010,
                attack_timer: 0x0010,
                route: NinjaAttackPeriodRotationRouteLoc3D960::StartJumpAttack,
            },
            "ASM Ninja.asm loc_3D960 should fall through to the jump-attack path only when `$4C(a5)` is zero"
        );
    }

    #[test]
    fn ninja_jump_attack_start_loc_3d9a4_seeds_directional_lunge() {
        assert_eq!(
            ninja_jump_attack_start_loc_3d9a4(false),
            NinjaJumpAttackStartStateLoc3D9A4 {
                next_collision_type: 0,
                x_velocity: -0x0001_8000,
                y_velocity: -0x0004_8000,
                has_level_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D6E",
            },
            "ASM Ninja.asm loc_3D9A4 should clear collision, lunge left at -$18000, jump at -$48000, and select C7D6E"
        );
        assert_eq!(
            ninja_jump_attack_start_loc_3d9a4(true),
            NinjaJumpAttackStartStateLoc3D9A4 {
                next_collision_type: 0,
                x_velocity: 0x0001_0000,
                y_velocity: -0x0004_8000,
                has_level_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D6E",
            },
            "ASM Ninja.asm loc_3D9B8 should branch by set x_direction and use +$10000 for the right-facing lunge"
        );
    }

    #[test]
    fn ninja_jump_attack_start_loc_3d9c0_sets_shared_airborne_state() {
        let state = ninja_jump_attack_start_loc_3d9a4(false);

        assert_eq!(state.next_collision_type, 0);
        assert_eq!(state.y_velocity, -0x0004_8000);
        assert!(state.has_level_collision);
        assert!(state.is_moved);
        assert!(!state.is_animated);
        assert_eq!(
            state.sprite_link, "LnkTo_unk_C7D6E",
            "ASM Ninja.asm loc_3D9C0 should set y_vel=-$48000, enable collision/motion, clear animation, and select C7D6E"
        );
    }

    #[test]
    fn ninja_jump_attack_airborne_tick_loc_3d9da_applies_gravity_and_sprite_bands() {
        assert_eq!(
            ninja_jump_attack_airborne_tick_loc_3d9da(-0x0004_8000),
            NinjaJumpAttackAirborneStateLoc3D9DA {
                y_velocity: -0x0004_5800,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                sprite_link: None,
            },
            "ASM Ninja.asm loc_3D9DA should add $2800 after sub_3E8BA and keep the rising sprite while y_vel < -$8000"
        );

        assert_eq!(
            ninja_jump_attack_airborne_tick_loc_3d9da(-0x0000_A800),
            NinjaJumpAttackAirborneStateLoc3D9DA {
                y_velocity: -0x0000_8000,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                sprite_link: Some("LnkTo_unk_C7D76"),
            },
            "ASM Ninja.asm loc_3D9DA should include -$8000 in the middle sprite band"
        );

        assert_eq!(
            ninja_jump_attack_airborne_tick_loc_3d9da(0x0000_5800),
            NinjaJumpAttackAirborneStateLoc3D9DA {
                y_velocity: 0x0000_8000,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                sprite_link: Some("LnkTo_unk_C7D76"),
            },
            "ASM Ninja.asm loc_3D9DA should include +$8000 in the middle sprite band"
        );

        assert_eq!(
            ninja_jump_attack_airborne_tick_loc_3d9da(0x0000_5801),
            NinjaJumpAttackAirborneStateLoc3D9DA {
                y_velocity: 0x0000_8001,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                sprite_link: Some("LnkTo_unk_C7D7E"),
            },
            "ASM Ninja.asm loc_3DA06 should select C7D7E only when y_vel > +$8000"
        );
    }

    #[test]
    fn ninja_jump_attack_collision_dispatch_loc_3da0c_clears_and_routes_contact() {
        assert_eq!(
            ninja_jump_attack_collision_dispatch_loc_3da0c(0),
            NinjaJumpAttackCollisionStateLoc3DA0C {
                next_collision_type: 0,
                route: NinjaJumpAttackCollisionRouteLoc3DA0C::ContinueAirborneLoop,
            },
            "ASM Ninja.asm loc_3DA0C should clear zero collision_type and loop back to loc_3D9DA"
        );

        assert_eq!(
            ninja_jump_attack_collision_dispatch_loc_3da0c(NINJA_COLID_FLOOR),
            NinjaJumpAttackCollisionStateLoc3DA0C {
                next_collision_type: 0,
                route: NinjaJumpAttackCollisionRouteLoc3DA0C::Land,
            },
            "ASM Ninja.asm loc_3DA0C should route colid_floor to the landing path"
        );

        assert_eq!(
            ninja_jump_attack_collision_dispatch_loc_3da0c(NINJA_COLID_RIGHTWALL),
            NinjaJumpAttackCollisionStateLoc3DA0C {
                next_collision_type: 0,
                route: NinjaJumpAttackCollisionRouteLoc3DA0C::WallBounce,
            },
            "ASM Ninja.asm loc_3DA0C should send right-wall contact to the wall bounce path"
        );

        assert_eq!(
            ninja_jump_attack_collision_dispatch_loc_3da0c(NINJA_COLID_LEFTWALL),
            NinjaJumpAttackCollisionStateLoc3DA0C {
                next_collision_type: 0,
                route: NinjaJumpAttackCollisionRouteLoc3DA0C::WallBounce,
            },
            "ASM Ninja.asm loc_3DA0C should send left-wall contact to the same wall bounce path"
        );

        assert_eq!(
            ninja_jump_attack_collision_dispatch_loc_3da0c(NINJA_COLID_CEILING),
            NinjaJumpAttackCollisionStateLoc3DA0C {
                next_collision_type: 0,
                route: NinjaJumpAttackCollisionRouteLoc3DA0C::CeilingContact,
            },
            "ASM Ninja.asm loc_3DA0C should route colid_ceiling to the ceiling-contact path"
        );

        assert_eq!(
            ninja_jump_attack_collision_dispatch_loc_3da0c(NINJA_COLID_KIDABOVE),
            NinjaJumpAttackCollisionStateLoc3DA0C {
                next_collision_type: 0,
                route: NinjaJumpAttackCollisionRouteLoc3DA0C::KidAboveContact,
            },
            "ASM Ninja.asm loc_3DA0C should route colid_kidabove to the Kid-above contact path"
        );

        assert_eq!(
            ninja_jump_attack_collision_dispatch_loc_3da0c(NINJA_COLID_KIDLEFT),
            NinjaJumpAttackCollisionStateLoc3DA0C {
                next_collision_type: 0,
                route: NinjaJumpAttackCollisionRouteLoc3DA0C::KidSideContact,
            },
            "ASM Ninja.asm loc_3DA0C should route either Kid-side contact to the Kid-side contact path"
        );
    }

    #[test]
    fn ninja_jump_attack_wall_bounce_loc_3da46_clears_reverses_and_toggles_facing() {
        assert_eq!(
            ninja_jump_attack_wall_bounce_loc_3da46(0x0001_0000, true),
            NinjaJumpAttackWallBounceStateLoc3DA46 {
                next_collision_type: 0,
                x_velocity: -0x0001_0000,
                x_direction_right: false,
                loops_airborne: true,
            },
            "ASM Ninja.asm loc_3DA46 should clear collision_type, negate positive x_vel, toggle set x_direction, and loop"
        );

        assert_eq!(
            ninja_jump_attack_wall_bounce_loc_3da46(-0x0001_8000, false),
            NinjaJumpAttackWallBounceStateLoc3DA46 {
                next_collision_type: 0,
                x_velocity: 0x0001_8000,
                x_direction_right: true,
                loops_airborne: true,
            },
            "ASM Ninja.asm loc_3DA46 should negate negative x_vel and toggle clear x_direction with eori.b #$FF"
        );
    }

    #[test]
    fn ninja_jump_attack_landing_loc_3da56_stops_motion_and_resumes_by_facing() {
        assert_eq!(
            ninja_jump_attack_landing_loc_3da56(false),
            NinjaJumpAttackLandingStateLoc3DA56 {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                animation_script: "stru_3D4EA",
                is_moved: false,
                has_level_collision: false,
                is_animated: true,
                next: NinjaJumpAttackLandingResumeLoc3DA56::Loc3E566,
            },
            "ASM Ninja.asm loc_3DA56 should stop the jump attack, restart stru_3D4EA, and branch clear x_direction to loc_3E566"
        );

        assert_eq!(
            ninja_jump_attack_landing_loc_3da56(true),
            NinjaJumpAttackLandingStateLoc3DA56 {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                animation_script: "stru_3D4EA",
                is_moved: false,
                has_level_collision: false,
                is_animated: true,
                next: NinjaJumpAttackLandingResumeLoc3DA56::Loc3E3B2,
            },
            "ASM Ninja.asm loc_3DA56 should branch set x_direction to loc_3E3B2 after clearing jump state"
        );
    }

    #[test]
    fn ninja_kid_above_contact_loc_3da98_bounces_and_branches_on_timer_zero() {
        assert_eq!(
            ninja_kid_above_contact_loc_3da98(2),
            NinjaKidAboveContactStateLoc3DA98 {
                next_collision_type: 0,
                y_velocity: 0x0003_8000,
                block_timer_44: 1,
                route: NinjaKidAboveContactRouteLoc3DA98::ContinueAirborneLoop,
            },
            "ASM Ninja.asm loc_3DA98 should set y_vel=$38000, decrement `$44`, clear collision, and branch back to loc_3D9DA while non-zero"
        );

        assert_eq!(
            ninja_kid_above_contact_loc_3da98(1),
            NinjaKidAboveContactStateLoc3DA98 {
                next_collision_type: 0,
                y_velocity: 0x0003_8000,
                block_timer_44: 0,
                route: NinjaKidAboveContactRouteLoc3DA98::FallthroughLandingLoop,
            },
            "ASM Ninja.asm loc_3DA98 should fall through to the landing loop when `$44` becomes zero after decrement"
        );

        assert_eq!(
            ninja_kid_above_contact_loc_3da98(0),
            NinjaKidAboveContactStateLoc3DA98 {
                next_collision_type: 0,
                y_velocity: 0x0003_8000,
                block_timer_44: 0xFFFF,
                route: NinjaKidAboveContactRouteLoc3DA98::ContinueAirborneLoop,
            },
            "ASM Ninja.asm loc_3DA98 uses word wrapping arithmetic, so zero underflows and keeps the airborne loop"
        );
    }

    #[test]
    fn ninja_kid_above_landing_loop_loc_3dabc_waits_for_floor_then_snaps_y() {
        assert_eq!(
            ninja_kid_above_landing_loop_loc_3dabc(0, 0x0001_0000, 0x0000_4000, 0x0028_8000),
            NinjaKidAboveLandingLoopStateLoc3DABC {
                next_collision_type: 0,
                x_velocity: 0x0001_0000,
                y_velocity: 0x0000_5800,
                y_position: 0x0028_8000,
                is_moved: true,
                has_level_collision: true,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                route: NinjaKidAboveLandingLoopRouteLoc3DABC::Loop,
            },
            "ASM Ninja.asm loc_3DABC should clear collision_type, add $1800 to y_vel, and loop until floor contact"
        );

        assert_eq!(
            ninja_kid_above_landing_loop_loc_3dabc(
                NINJA_COLID_FLOOR,
                -0x0001_8000,
                0x0000_5800,
                0x0028_8000
            ),
            NinjaKidAboveLandingLoopStateLoc3DABC {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                y_position: 0x0030_0000,
                is_moved: false,
                has_level_collision: false,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                route: NinjaKidAboveLandingLoopRouteLoc3DABC::To3DB4E,
            },
            "ASM Ninja.asm loc_3DABC should clear motion/flags and snap y_pos to ((integer_y + $F) & $FFF0) on floor contact"
        );
    }

    #[test]
    fn ninja_berzerker_side_contact_loc_3db0a_launches_only_while_charging() {
        assert_eq!(
            ninja_berzerker_side_contact_loc_3db0a(false),
            NinjaBerzerkerSideContactRouteLoc3DB0A::ContinueAirborneLoop,
            "ASM Ninja.asm loc_3DB0A should branch back to the airborne loop when Berzerker_charging is clear"
        );

        assert_eq!(
            ninja_berzerker_side_contact_loc_3db0a(true),
            NinjaBerzerkerSideContactRouteLoc3DB0A::Launch(NinjaBerzerkerSideLaunchStateLoc3DB0A {
                has_kid_collision: true,
                has_level_collision: false,
                is_moved: true,
                is_animated: false,
                sprite_link: "LnkTo_unk_C7D7E",
                x_velocity: 0,
                y_velocity: -0x0004_8000,
            },),
            "ASM Ninja.asm loc_3DB0A should freeze animation, keep Kid collision/movement enabled, and launch upward while charging"
        );
    }

    #[test]
    fn ninja_berzerker_side_launch_loop_loc_3db3c_applies_gravity_and_loops() {
        assert_eq!(
            ninja_berzerker_side_launch_loop_tick_loc_3db3c(-0x0004_8000),
            NinjaBerzerkerSideLaunchLoopTickLoc3DB3C {
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                y_velocity: -0x0004_4000,
                loops: true,
            },
            "ASM Ninja.asm loc_3DB3C should hibernate, run sub_3E8BA, add $4000 to y_vel, and loop"
        );

        assert_eq!(
            ninja_berzerker_side_launch_loop_tick_loc_3db3c(0x0000_7FFF),
            NinjaBerzerkerSideLaunchLoopTickLoc3DB3C {
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                y_velocity: 0x0000_BFFF,
                loops: true,
            },
            "ASM Ninja.asm loc_3DB3C applies the same $4000 gravity increment regardless of velocity sign"
        );
    }

    #[test]
    fn ninja_death_sequence_matches_loc_3db4e() {
        assert_eq!(
            ninja_death_sequence_loc_3db4e(0x05),
            NinjaDeathSequenceLoc3DB4E {
                death_animation: "stru_3D50A",
                waits_until_animation_finished: true,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                    index: 5,
                    word_value: 0x2168
                },
                exits_enemy: true,
            },
            "ASM Ninja.asm loc_3DB4E should play the death animation, wait for it, then exit through loc_3E956"
        );
    }

    #[test]
    fn ninja_ceiling_contact_loc_3dab0_clears_collision_and_reflects_y_velocity() {
        assert_eq!(
            ninja_ceiling_contact_loc_3dab0(-0x0002_4000),
            NinjaCeilingContactStateLoc3DAB0 {
                next_collision_type: 0,
                y_velocity: 0x0002_4000,
                loops_airborne: true,
            },
            "ASM Ninja.asm loc_3DAB0 should clear collision_type, negate upward y_vel, and branch back to loc_3D9DA"
        );

        assert_eq!(
            ninja_ceiling_contact_loc_3dab0(0x0000_8000),
            NinjaCeilingContactStateLoc3DAB0 {
                next_collision_type: 0,
                y_velocity: -0x0000_8000,
                loops_airborne: true,
            },
            "ASM Ninja.asm loc_3DAB0 should use signed neg.l behaviour for positive y_vel too"
        );
    }

    #[test]
    fn ninja_collision_dispatch_loc_3d606_clears_and_routes_collision_types() {
        assert_eq!(
            ninja_collision_dispatch_loc_3d606(0, false, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D6DA,
            },
            "ASM Ninja.asm loc_3D606 should clear collision_type and jump to loc_3D6DA when d4 is zero"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(-1, false, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3DB4E,
            },
            "ASM Ninja.asm loc_3D606 should route negative collision IDs to loc_3DB4E via bmi"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_HURT as i16, true, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D65A,
            },
            "ASM Ninja.asm loc_3D606 should route colid_hurt to loc_3D65A after clearing collision_type"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_KIDABOVE as i16, false, true),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D65A,
            },
            "ASM Ninja.asm loc_3D606 should route colid_kidabove to loc_3D65A after clearing collision_type"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_KIDRIGHT as i16, false, true),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D698,
            },
            "ASM Ninja.asm loc_3D606 should route colid_kidright to loc_3D698"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(NINJA_COLID_KIDLEFT as i16, true, false),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D698,
            },
            "ASM Ninja.asm loc_3D606 should route colid_kidleft to loc_3D698"
        );

        assert_eq!(
            ninja_collision_dispatch_loc_3d606(0x00FF, true, true),
            NinjaCollisionStateLoc3D606 {
                next_collision_type: 0,
                route: NinjaCollisionLoc3D606::To3D638 {
                    next: NinjaCollisionResumeLoc3D638::Loc3E3AC
                },
            },
            "ASM Ninja.asm loc_3D606 should fall through to loc_3D638 when collision_id is unhandled"
        );
    }

    #[test]
    fn ninja_collision_resume_loc_3d638_dispatches_wallpin_and_facing() {
        assert_eq!(
            ninja_collision_resume_loc_3d638(false, false),
            NinjaCollisionResumeLoc3D638::Loc3E7CA,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E7CA for left-facing when `$5C(a5)` is clear"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d638(false, true),
            NinjaCollisionResumeLoc3D638::Loc3E6AE,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E6AE for right-facing when `$5C(a5)` is clear"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d638(true, false),
            NinjaCollisionResumeLoc3D638::Loc3E560,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E560 for left-facing when `$5C(a5)` is set"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d638(true, true),
            NinjaCollisionResumeLoc3D638::Loc3E3AC,
            "ASM Ninja.asm loc_3D638 should jump to loc_3E3AC for right-facing when `$5C(a5)` is set"
        );
    }

    #[test]
    fn ninja_collision_resume_loc_3d64e_removes_wallpin_gate_from_loc_3d638() {
        assert_eq!(
            ninja_collision_resume_loc_3d64e(false),
            NinjaCollisionResumeLoc3D638::Loc3E560,
            "ASM Ninja.asm loc_3D64E should branch to loc_3E560 with x_direction cleared"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d64e(true),
            NinjaCollisionResumeLoc3D638::Loc3E3AC,
            "ASM Ninja.asm loc_3D64E should branch to loc_3E3AC when x_direction indicates right-facing"
        );
    }

    #[test]
    fn ninja_collision_loc_3d66e_is_a_fallthrough_gate_to_loc_3d638() {
        assert_eq!(
            ninja_collision_resume_loc_3d66e(NinjaCollisionResumeLoc3D638::Loc3E7CA),
            NinjaCollisionStateLoc3D66E {
                next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
            },
            "ASM Ninja.asm loc_3D66E should pass the collision-resume route through unchanged"
        );
        assert_eq!(
            ninja_collision_resume_loc_3d66e(NinjaCollisionResumeLoc3D638::Loc3E6AE),
            NinjaCollisionStateLoc3D66E {
                next: NinjaCollisionResumeLoc3D638::Loc3E6AE,
            },
            "ASM Ninja.asm loc_3D66E should always behave as an unconditional branch to loc_3D638"
        );
    }

    #[test]
    fn ninja_collision_loc_3d670_handles_block_timer_and_animation_swap() {
        assert_eq!(
            ninja_collision_block_animation_loc_3d670(1, false, false, "test_restore_animation"),
            NinjaCollisionStateLoc3D670 {
                next_collision_type: 0,
                remaining_block_timer: 0,
                route: NinjaCollisionLoc3D670Route::To3DB4E,
                uses_blocking_animation: false,
                blocking_animation: None,
                restored_animation: None,
                hibernates_until_animation_finished: false,
            },
            "ASM Ninja.asm loc_3D670 should jump to loc_3DB4E when `$44(a5)` becomes zero after decrement"
        );
        assert_eq!(
            ninja_collision_block_animation_loc_3d670(5, false, false, "saved_animation"),
            NinjaCollisionStateLoc3D670 {
                next_collision_type: 0,
                remaining_block_timer: 4,
                route: NinjaCollisionLoc3D670Route::To3D638 {
                    next: NinjaCollisionResumeLoc3D638::Loc3E7CA,
                },
                uses_blocking_animation: true,
                blocking_animation: Some("stru_3D504"),
                restored_animation: Some("saved_animation"),
                hibernates_until_animation_finished: true,
            },
            "ASM Ninja.asm loc_3D670 should decrement `$44(a5)`, run blocking animation + Hibernate_UntilAnimFinished, then restore `$62(a5)` into `$50(a5)` before loc_3D638"
        );
        assert_eq!(
            ninja_collision_block_animation_loc_3d670(2, true, true, "other_animation"),
            NinjaCollisionStateLoc3D670 {
                next_collision_type: 0,
                remaining_block_timer: 1,
                route: NinjaCollisionLoc3D670Route::To3D638 {
                    next: NinjaCollisionResumeLoc3D638::Loc3E3AC,
                },
                uses_blocking_animation: true,
                blocking_animation: Some("stru_3D504"),
                restored_animation: Some("other_animation"),
                hibernates_until_animation_finished: true,
            },
            "ASM Ninja.asm loc_3D670 should branch to loc_3D638 with wallpin/facing resume preserved after the blocking animation gate"
        );
    }

    #[test]
    fn ninja_collision_loc_3d6c8_loops_after_sub_3e8ba_and_y_vel_bump() {
        assert_eq!(
            ninja_collision_stun_loop_loc_3d6c8(-0x0004_0000),
            NinjaCollisionStateLoc3D6C8 {
                y_velocity: -0x0003_C000,
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                loops: true,
            },
            "ASM Ninja.asm loc_3D6C8 should hibernate, run `sub_3E8BA`, add `$4000` to y_vel(a3), then loop"
        );
        assert_eq!(
            ninja_collision_stun_loop_loc_3d6c8(-0x3FFF_0000),
            NinjaCollisionStateLoc3D6C8 {
                y_velocity: (-0x3FFF_0000_i32).wrapping_add(0x4000),
                hibernates_one_frame: true,
                checked_sub_3e8ba: true,
                loops: true,
            },
            "ASM Ninja.asm loc_3D6C8 increments y_vel(a3) each frame by `$4000` regardless of starting sign"
        );
    }
}
