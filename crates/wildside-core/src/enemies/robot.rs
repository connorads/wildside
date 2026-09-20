use super::*;

/// Robot walk speed selected once from immutable enemy_level at init.
/// ASM Robot.asm loc_3A7AC/loc_3A7EE derives current_hp as enemy_level + 2,
/// then stores `$8000` for level 0 or `$10000` for every higher level.
pub(crate) fn robot_walk_speed(enemy_level: u8) -> i32 {
    if enemy_level >= 1 { 0x1_0000 } else { 0x8000 }
}

pub(crate) fn robot_charge_flip_reload_with_rng(rng: &mut RomRandomState) -> u16 {
    // ASM loc_3A9F0: `Get_RandomNumber_byte`, `lsr.w #1,d7`, `addi.w #$F,d7`.
    let random_byte = rng.next_byte();
    u16::from(random_byte >> 1) + ROBOT_CHARGE_FLIP_BASE
}

/// ASM Init_Animation / GfxObject_Animate: selected mapping and next script entry
/// are independent; loc_3AA9E deliberately replays the sixth body entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RobotAnimationCursor {
    pub(crate) selected: u8,
    pub(crate) next_entry: u8,
    pub(crate) timer: u8,
}

impl RobotAnimationCursor {
    fn init(first: u8) -> Self {
        Self {
            selected: first,
            next_entry: 0,
            timer: 1,
        }
    }

    fn advance(&mut self, first: u8, entries: u8, duration: u8) {
        self.timer -= 1;
        if self.timer == 0 {
            self.selected = first + self.next_entry;
            self.next_entry = (self.next_entry + 1) % entries;
            self.timer = duration;
        }
    }
}

/// ASM Enemy03_Robot_Init: body and linked head own separate GfxObject state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RobotGfxState {
    pub(crate) body: RobotAnimationCursor,
    pub(crate) head: RobotAnimationCursor,
    pub(crate) head_facing: Option<Direction>,
}

impl Default for RobotGfxState {
    fn default() -> Self {
        Self {
            body: RobotAnimationCursor::init(0),
            head: RobotAnimationCursor::init(10),
            head_facing: None,
        }
    }
}

pub(crate) fn advance_robot_gfx(enemy: &mut ActiveEnemy) {
    // ASM loc_142E runs after Object callbacks, including hibernated walkers.
    // TODO(fidelity): #433 loc_3A8FC hurt replaces only the body script;
    // the retained native hurt path freezes both cursors and needs collision probes.
    if enemy.death_animation_active
        || enemy.damage_flash_frames > 0
        || matches!(enemy.robot_state, RobotState::Shooting)
    {
        return;
    }
    let charging = matches!(enemy.robot_state, RobotState::Charging);
    enemy
        .robot_gfx
        .body
        .advance(0, 6, if charging { 4 } else { 9 });
    if !matches!(enemy.robot_state, RobotState::Aiming) {
        enemy.robot_gfx.head.advance(
            if charging { 16 } else { 10 },
            if charging { 3 } else { 6 },
            if charging { 4 } else { 9 },
        );
    }
}

pub(crate) fn robot_head_facing(enemy: &ActiveEnemy) -> Direction {
    enemy.robot_gfx.head_facing.unwrap_or(enemy.facing)
}

pub(crate) fn reset_robot_walk_animation(enemy: &mut ActiveEnemy) {
    // ASM loc_3A936/loc_3AB14 reinitialises both scripts, preserving direction.
    enemy.robot_gfx.body = RobotAnimationCursor::init(0);
    enemy.robot_gfx.head = RobotAnimationCursor::init(10);
}

/// Robot projectile speed: ~5 px/frame horizontal (Robot.asm).
pub(crate) const ROBOT_PROJECTILE_SPEED: i32 = 0x5_0000;
pub(crate) const ROBOT_SHOT_IMPACT_ANIMATION_FRAMES_LOC_3A762: u16 = 3 + 4 + 8;

#[cfg(test)]
pub(crate) fn robot_jump_sound_edge_loc_3a890(
    body_sprite_is_landing_frame: bool,
    animation_timer: u16,
    motion_suppressed: bool,
) -> bool {
    // ASM Robot.asm loc_3A890: C82A8 sprite, animation_timer == 4, and clear
    // `$19(a3)` replay sfx_Robot_jumping_on before collision dispatch.
    body_sprite_is_landing_frame && animation_timer == 4 && !motion_suppressed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotCollisionRouteLoc3A8Ae {
    ClearAndWalk,
    HitReaction,
    EnterCharge,
    ContinueCharge,
    Destroy,
}

pub(crate) fn robot_collision_dispatch_loc_3a8ae(
    parent_collision_type: i16,
    child_collision_type: i16,
    berzerker_charging: bool,
) -> RobotCollisionRouteLoc3A8Ae {
    // ASM Robot.asm loc_3A8AE: parent collision_type is tested before `$38(a2)`.
    if parent_collision_type.is_negative() {
        return robot_destroy_prep_loc_3a9d2().route;
    }
    match parent_collision_type {
        0x0800 => return RobotCollisionRouteLoc3A8Ae::ContinueCharge,
        0x001C => return RobotCollisionRouteLoc3A8Ae::HitReaction,
        _ => {}
    }

    if child_collision_type.is_negative() {
        return RobotCollisionRouteLoc3A8Ae::EnterCharge;
    }
    match child_collision_type {
        0x001C | 0x002C => RobotCollisionRouteLoc3A8Ae::HitReaction,
        0x0020 | 0x0024 => robot_berzerker_side_gate_loc_3a9cc(berzerker_charging),
        0x0800 => RobotCollisionRouteLoc3A8Ae::ContinueCharge,
        _ => RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
    }
}

pub(crate) fn robot_berzerker_side_gate_loc_3a9cc(
    berzerker_charging: bool,
) -> RobotCollisionRouteLoc3A8Ae {
    // ASM Robot.asm loc_3A9CC: `$20/$24` child side contacts branch to
    // loc_3A958 when `Berzerker_charging` is clear; otherwise they fall
    // through to the destruction prep path at loc_3A9D2.
    if berzerker_charging {
        robot_destroy_prep_loc_3a9d2().route
    } else {
        RobotCollisionRouteLoc3A8Ae::ClearAndWalk
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotDestroyPrepLoc3A9D2 {
    pub(crate) child_status_13: bool,
    pub(crate) child_has_kid_collision: bool,
    pub(crate) parent_has_kid_collision: bool,
    pub(crate) plays_destroy_animation: bool,
    pub(crate) hibernates_until_animation_finished: bool,
    pub(crate) route: RobotCollisionRouteLoc3A8Ae,
}

pub(crate) fn robot_destroy_prep_loc_3a9d2() -> RobotDestroyPrepLoc3A9D2 {
    // ASM Robot.asm loc_3A9D2: hide the child (`sf $13(a2)`), mark child and
    // parent Kid collision flags, run `stru_3A74A` to completion, then branch
    // to the shared destroy tail.
    RobotDestroyPrepLoc3A9D2 {
        child_status_13: false,
        child_has_kid_collision: true,
        parent_has_kid_collision: true,
        plays_destroy_animation: true,
        hibernates_until_animation_finished: true,
        route: RobotCollisionRouteLoc3A8Ae::Destroy,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotCollisionClearLoc3A958 {
    pub(crate) next_parent_collision_type: u16,
    pub(crate) next_child_collision_type: u16,
    pub(crate) resume_walk_direction: Direction,
}

#[cfg(test)]
pub(crate) fn robot_collision_clear_loc_3a958(facing: Direction) -> RobotCollisionClearLoc3A958 {
    // ASM Robot.asm loc_3A958: clear child `$38(a2)` and parent
    // `collision_type(a3)`, then dispatch to the shared left/right walker
    // from x_direction. Native `facing` is the decoded branch target.
    RobotCollisionClearLoc3A958 {
        next_parent_collision_type: 0,
        next_child_collision_type: 0,
        resume_walk_direction: facing,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotChargeEntryLoc3A96C {
    pub(crate) tracking_latch_cleared: bool,
    pub(crate) child_status_13_during_windup: bool,
    pub(crate) child_status_13_after_windup: bool,
    pub(crate) parent_has_kid_collision: bool,
    pub(crate) child_has_kid_collision: bool,
    pub(crate) charge_speed: i32,
    pub(crate) next_parent_collision_type: u16,
    pub(crate) next_child_collision_type: u16,
    pub(crate) flip_timer: u16,
    pub(crate) resume_walk_direction: Direction,
}

pub(crate) fn robot_charge_entry_loc_3a96c(facing: Direction) -> RobotChargeEntryLoc3A96C {
    // ASM Robot.asm loc_3A96C: clear `$4E(a5)`, hide the child during the
    // windup, mark both body/head as Kid-collidable, then seed charge speed,
    // `$800` collision words, and `$54(a5)=$F` before branching by x_direction.
    RobotChargeEntryLoc3A96C {
        tracking_latch_cleared: true,
        child_status_13_during_windup: false,
        child_status_13_after_windup: true,
        parent_has_kid_collision: true,
        child_has_kid_collision: true,
        charge_speed: ROBOT_CHARGE_SPEED,
        next_parent_collision_type: 0x0800,
        next_child_collision_type: 0x0800,
        flip_timer: ROBOT_CHARGE_FLIP_BASE,
        resume_walk_direction: facing,
    }
}

pub(crate) fn robot_direction_branch_loc_3aa0c(x_direction: u8) -> Direction {
    // ASM Robot.asm loc_3AA0C: `tst.b x_direction(a3); beq.w loc_3C0FA;
    // bra.w loc_3C04E`. Native stores the decoded shared-walker branch.
    if x_direction == 0 {
        Direction::Left
    } else {
        Direction::Right
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotShootWindupRouteLoc3AA4C {
    ContinueShoot,
    CollisionHandler,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootWindupLoc3AA4C {
    pub(crate) hibernated_frames: u8,
    pub(crate) route: RobotShootWindupRouteLoc3AA4C,
}

#[cfg(test)]
pub(crate) fn robot_shoot_windup_loc_3aa4c(
    first_child_collision_frame: Option<u8>,
) -> RobotShootWindupLoc3AA4C {
    // ASM Robot.asm loc_3AA4C: with `d0=#8`, hibernate one frame, test
    // `$38(a2)`, and DBF back. That gives 9 post-hibernate collision checks.
    match first_child_collision_frame {
        Some(frame @ 1..=9) => RobotShootWindupLoc3AA4C {
            hibernated_frames: frame,
            route: RobotShootWindupRouteLoc3AA4C::CollisionHandler,
        },
        _ => RobotShootWindupLoc3AA4C {
            hibernated_frames: 9,
            route: RobotShootWindupRouteLoc3AA4C::ContinueShoot,
        },
    }
}

#[cfg(test)]
pub(crate) fn robot_shoot_head_x_delta_loc_3aa68(child_x_direction: u8) -> i16 {
    // ASM Robot.asm loc_3AA68: zero child `$16` falls through here and loads
    // `d1=#4`; non-zero child direction uses the preceding `d1=#-$4` path.
    if child_x_direction == 0 { 4 } else { -4 }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootHeadNudgeLoc3AA6C {
    pub(crate) child_x_delta_px: i16,
    pub(crate) next_loop_counter: u8,
}

#[cfg(test)]
pub(crate) fn robot_shoot_head_nudge_loc_3aa6c(
    child_x_direction: u8,
) -> RobotShootHeadNudgeLoc3AA6C {
    let child_x_delta_px = robot_shoot_head_x_delta_loc_3aa68(child_x_direction);
    // ASM Robot.asm loc_3AA6C: `add.w d1,$1A(a2)` nudges the child head X
    // position, then `moveq #4,d0` seeds the following checked hold loop.
    RobotShootHeadNudgeLoc3AA6C {
        child_x_delta_px,
        next_loop_counter: 4,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotShootHoldRouteLoc3AA72 {
    ContinueShoot,
    CollisionHandler,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootHeadHoldLoc3AA72 {
    pub(crate) hibernated_frames: u8,
    pub(crate) route: RobotShootHoldRouteLoc3AA72,
    pub(crate) child_x_delta_after_loop_px: i16,
}

#[cfg(test)]
pub(crate) fn robot_shoot_head_hold_loc_3aa72(
    child_x_delta_px: i16,
    first_child_collision_frame: Option<u8>,
) -> RobotShootHeadHoldLoc3AA72 {
    // ASM Robot.asm loc_3AA72: `d0=#4` hibernates/checks child `$38(a2)` five
    // times. A collision branches before `sub.w d1,$1A(a2)` can restore X.
    match first_child_collision_frame {
        Some(frame @ 1..=5) => RobotShootHeadHoldLoc3AA72 {
            hibernated_frames: frame,
            route: RobotShootHoldRouteLoc3AA72::CollisionHandler,
            child_x_delta_after_loop_px: child_x_delta_px,
        },
        _ => RobotShootHeadHoldLoc3AA72 {
            hibernated_frames: 5,
            route: RobotShootHoldRouteLoc3AA72::ContinueShoot,
            child_x_delta_after_loop_px: 0,
        },
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotShootRecoveryRouteLoc3AA88 {
    ContinueShoot,
    CollisionHandler,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootRecoveryPauseLoc3AA88 {
    pub(crate) hibernated_frames: u8,
    pub(crate) route: RobotShootRecoveryRouteLoc3AA88,
    pub(crate) child_y_delta_after_loop_px: i16,
}

#[cfg(test)]
pub(crate) fn robot_shoot_recovery_pause_loc_3aa88(
    first_child_collision_frame: Option<u8>,
) -> RobotShootRecoveryPauseLoc3AA88 {
    // ASM Robot.asm loc_3AA88: `d0=#$A` hibernates/checks child `$38(a2)`
    // eleven times. Only the uninterrupted path then `subi.w #3,$1E(a2)`.
    match first_child_collision_frame {
        Some(frame @ 1..=11) => RobotShootRecoveryPauseLoc3AA88 {
            hibernated_frames: frame,
            route: RobotShootRecoveryRouteLoc3AA88::CollisionHandler,
            child_y_delta_after_loop_px: 0,
        },
        _ => RobotShootRecoveryPauseLoc3AA88 {
            hibernated_frames: 11,
            route: RobotShootRecoveryRouteLoc3AA88::ContinueShoot,
            child_y_delta_after_loop_px: -3,
        },
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootRecoveryVisualLoc3AA9E {
    pub(crate) child_is_animated: bool,
    pub(crate) body_animation_source_offset_bytes: u16,
    pub(crate) body_sprite_word_offset_from_source: u16,
    pub(crate) body_is_animated: bool,
    pub(crate) body_phase_byte_18: u8,
    pub(crate) body_animation_timer: u16,
}

#[cfg(test)]
pub(crate) fn robot_shoot_recovery_visual_loc_3aa9e() -> RobotShootRecoveryVisualLoc3AA9E {
    // ASM Robot.asm loc_3AA9E: restore the shooting head (`st $15(a2)`), point
    // body animation state at `stru_3A6E2+$14`, copy word 2 to
    // `addroffset_sprite(a3)`, then restart body animation with timer 9.
    RobotShootRecoveryVisualLoc3AA9E {
        child_is_animated: true,
        body_animation_source_offset_bytes: 0x14,
        body_sprite_word_offset_from_source: 2,
        body_is_animated: true,
        body_phase_byte_18: 0,
        body_animation_timer: 9,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootResumeLoc3AAC4 {
    pub(crate) next_child_collision_type: u16,
    pub(crate) resume_walk_direction: Direction,
}

#[cfg(test)]
pub(crate) fn robot_shoot_resume_loc_3aac4(x_direction: u8) -> RobotShootResumeLoc3AAC4 {
    // ASM Robot.asm loc_3AAC4: clear child `$38(a2)`, then use the same
    // `x_direction(a3)` branch shape as loc_3AA0C (`loc_3C0FA`/`loc_3C04E`).
    RobotShootResumeLoc3AAC4 {
        next_child_collision_type: 0,
        resume_walk_direction: robot_direction_branch_loc_3aa0c(x_direction),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotShootCollisionRouteLoc3AAD4 {
    RestoreShootWalk,
    HitReaction,
    Destroy,
}

pub(crate) fn robot_shoot_berzerker_side_gate_loc_3aafa(
    berzerker_charging: bool,
) -> RobotShootCollisionRouteLoc3AAD4 {
    // ASM Robot.asm loc_3AAFA: child side contacts `$20/$24` branch back to
    // loc_3AA9E while `Berzerker_charging` is clear; a charging Berzerker
    // tail-jumps to the shared Robot destroy prep at loc_3A9D2.
    if berzerker_charging {
        RobotShootCollisionRouteLoc3AAD4::Destroy
    } else {
        RobotShootCollisionRouteLoc3AAD4::RestoreShootWalk
    }
}

pub(crate) fn robot_shoot_collision_dispatch_loc_3aad4(
    child_collision_type_d0: u16,
    latched_child_collision_type_d4: u16,
    parent_collision_type: u16,
    berzerker_charging: bool,
) -> RobotShootCollisionRouteLoc3AAD4 {
    // ASM Robot.asm loc_3AAD4/loc_3AB04: `$38(a2)==$2C`, stale/latched
    // `d4==$1C`, or parent `collision_type(a3)==colid_hurt` route to the hit
    // reaction. Child side contacts `$20/$24` only destroy the Robot while
    // Berzerker is charging; every other checked value restores shoot/walk.
    if child_collision_type_d0 == 0x002C
        || latched_child_collision_type_d4 == 0x001C
        || parent_collision_type == 0x001C
    {
        return RobotShootCollisionRouteLoc3AAD4::HitReaction;
    }

    if matches!(latched_child_collision_type_d4, 0x0020 | 0x0024) {
        return robot_shoot_berzerker_side_gate_loc_3aafa(berzerker_charging);
    }

    RobotShootCollisionRouteLoc3AAD4::RestoreShootWalk
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootHitAnimationStartLoc3AB14 {
    pub(crate) child_y_offset_delta_px: i16,
    pub(crate) animation_label: &'static str,
    pub(crate) hibernates_until_animation_finished: bool,
}

#[cfg(test)]
pub(crate) fn robot_shoot_hit_animation_start_loc_3ab14() -> RobotShootHitAnimationStartLoc3AB14 {
    // ASM Robot.asm loc_3AB14: the shoot hit reaction always drops the child
    // head by 4px, starts `stru_3A744`, and waits for that animation before
    // the later `subi.w #1,$44(a3)` HP decrement.
    RobotShootHitAnimationStartLoc3AB14 {
        child_y_offset_delta_px: 4,
        animation_label: "stru_3A744",
        hibernates_until_animation_finished: true,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShootLethalHitLoc3AB56 {
    pub(crate) tracking_latch_cleared: bool,
    pub(crate) child_status_13_during_windup: bool,
    pub(crate) child_status_13_after_windup: bool,
    pub(crate) parent_has_kid_collision: bool,
    pub(crate) child_has_kid_collision: bool,
    pub(crate) body_windup_animation_label: &'static str,
    pub(crate) body_charge_animation_label: &'static str,
    pub(crate) child_charge_animation_label: &'static str,
    pub(crate) hibernates_until_windup_finished: bool,
    pub(crate) charge_speed: i32,
    pub(crate) next_parent_collision_type: u16,
    pub(crate) next_child_collision_type: u16,
    pub(crate) flip_timer: u16,
    pub(crate) resume_walk_direction: Direction,
}

#[cfg(test)]
pub(crate) fn robot_shoot_lethal_hit_loc_3ab56(facing: Direction) -> RobotShootLethalHitLoc3AB56 {
    // ASM Robot.asm loc_3AB56: lethal shoot-hit clears `$4E(a5)`, hides the
    // child head for `stru_3A74A`, then restores it and seeds the shared charge
    // state before branching by x_direction.
    RobotShootLethalHitLoc3AB56 {
        tracking_latch_cleared: true,
        child_status_13_during_windup: false,
        child_status_13_after_windup: true,
        parent_has_kid_collision: true,
        child_has_kid_collision: true,
        body_windup_animation_label: "stru_3A74A",
        body_charge_animation_label: "stru_3A6FC",
        child_charge_animation_label: "stru_3A730",
        hibernates_until_windup_finished: true,
        charge_speed: ROBOT_CHARGE_SPEED,
        next_parent_collision_type: 0x0800,
        next_child_collision_type: 0x0800,
        flip_timer: ROBOT_CHARGE_FLIP_BASE,
        resume_walk_direction: facing,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShotSpawnLoc3AC0C {
    pub(crate) projectile_x_direction: u8,
    pub(crate) projectile_x_velocity: i32,
    pub(crate) projectile_x_delta_px: i16,
    pub(crate) projectile_y_delta_px: i16,
    pub(crate) animation_label: &'static str,
    pub(crate) hibernates_one_frame_before_sound_gate: bool,
}

pub(crate) fn robot_shot_spawn_loc_3ac0c(
    child_x_direction: u8,
    child_y_offset_46_px: i16,
) -> RobotShotSpawnLoc3AC0C {
    // ASM Robot.asm loc_3AC0C: after copying child x_direction, zero direction
    // keeps d0=$FFFB and d1=$FFE6; non-zero direction negates both before
    // storing shot velocity and X spawn offset.
    let faces_left = child_x_direction == 0;
    RobotShotSpawnLoc3AC0C {
        projectile_x_direction: child_x_direction,
        projectile_x_velocity: if faces_left {
            -ROBOT_PROJECTILE_SPEED
        } else {
            ROBOT_PROJECTILE_SPEED
        },
        projectile_x_delta_px: if faces_left { -26 } else { 26 },
        projectile_y_delta_px: 2 - child_y_offset_46_px,
        animation_label: "stru_3A754",
        hibernates_one_frame_before_sound_gate: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotShotFlightRouteLoc3AC48 {
    ContinueFlight,
    StartImpact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShotFlightLoc3AC48 {
    pub(crate) collision_type_cleared_before_frame: bool,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: RobotShotFlightRouteLoc3AC48,
}

pub(crate) fn robot_shot_flight_loop_loc_3ac48(
    screen_x_px: i16,
    screen_y_px: i16,
    post_frame_collision_type: u16,
) -> RobotShotFlightLoc3AC48 {
    // ASM Robot.asm loc_3AC48: clear collision_type, hibernate one frame, then
    // loop only while the shot is inside the inclusive camera window and no
    // new collision was written during the frame.
    let inside_camera_window =
        (-0x80..=0x01C0).contains(&screen_x_px) && (-0x80..=0x01C0).contains(&screen_y_px);
    let route = if inside_camera_window && post_frame_collision_type == 0 {
        RobotShotFlightRouteLoc3AC48::ContinueFlight
    } else {
        RobotShotFlightRouteLoc3AC48::StartImpact
    };

    RobotShotFlightLoc3AC48 {
        collision_type_cleared_before_frame: true,
        hibernates_one_frame: true,
        route,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShotImpactPrepLoc3AC7E {
    pub(crate) projectile_x_velocity: i32,
    pub(crate) impact_x_delta_px: i16,
}

pub(crate) fn robot_shot_impact_prep_loc_3ac7e(x_direction: u8) -> RobotShotImpactPrepLoc3AC7E {
    // ASM Robot.asm loc_3AC7E: zero x_vel, seed d0=$FFF8, then negate d0 when
    // `x_direction(a3)` is non-zero before the impact position tail applies it.
    RobotShotImpactPrepLoc3AC7E {
        projectile_x_velocity: 0,
        impact_x_delta_px: if x_direction == 0 { -8 } else { 8 },
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotShotImpactTailLoc3AC92 {
    pub(crate) x_pos_px: i16,
    pub(crate) animation_label: &'static str,
    pub(crate) hibernates_until_animation_finished: bool,
    pub(crate) deletes_current_object: bool,
}

#[cfg(test)]
pub(crate) fn robot_shot_impact_tail_loc_3ac92(
    x_pos_px: i16,
    impact_x_delta_px: i16,
) -> RobotShotImpactTailLoc3AC92 {
    // ASM Robot.asm loc_3AC92: add d0 to the integer x_pos word, initialise
    // impact animation stru_3A762, wait for it to finish, then delete the shot.
    RobotShotImpactTailLoc3AC92 {
        x_pos_px: x_pos_px.wrapping_add(impact_x_delta_px),
        animation_label: "stru_3A762",
        hibernates_until_animation_finished: true,
        deletes_current_object: true,
    }
}

const ROBOT_AIM_TURN_FRAMES: u16 = 26;
const ROBOT_AIM_DIRECTION_FLIP_REMAINING: u16 = 9;

fn robot_x_direction_byte(facing: Direction) -> u8 {
    match facing {
        Direction::Left => 0x00,
        Direction::Right => 0xFF,
    }
}

fn push_robot_shot(enemy: &ActiveEnemy, projectiles: &mut Option<&mut Vec<EnemyProjectile>>) {
    if let Some(projs) = projectiles {
        // ASM Enemy03_Robot_Shoot allocates an Object; Enemy03_Robot_Shot_Init
        // hibernates ten frames before reading the parent and allocating Gfx.
        let mut shot = EnemyProjectile {
            lifetime: 60,
            kind: EnemyProjectileKind::RobotShot,
            robot_shot_phase: crate::types::RobotShotPhase::Allocated,
            owner: Some(EnemyProjectileOwner {
                segment_index: enemy.segment_index,
                layout_index: enemy.layout_index,
                token: 0,
            }),
            ..Default::default()
        };
        refresh_robot_shot_parent(&mut shot, enemy);
        projs.push(shot);
    }
}

pub(crate) fn refresh_robot_shot_parent(shot: &mut EnemyProjectile, enemy: &ActiveEnemy) {
    // ASM Enemy03_Robot_Shot_Init / loc_3AC0C: copy parent coordinate words,
    // linked head direction and `$46=$10`, then apply word-width muzzle offsets.
    let facing = robot_head_facing(enemy);
    let spawn = robot_shot_spawn_loc_3ac0c(robot_x_direction_byte(facing), 0x10);
    shot.x_pos =
        u32::from(((enemy.x_pos >> 16) as u16).wrapping_add_signed(spawn.projectile_x_delta_px))
            << 16;
    shot.y_pos =
        u32::from(((enemy.y_pos >> 16) as u16).wrapping_add_signed(spawn.projectile_y_delta_px))
            << 16;
    shot.x_velocity = spawn.projectile_x_velocity;
    shot.palette_variant = enemy.palette_variant;
}

fn begin_robot_shoot(enemy: &mut ActiveEnemy, projectiles: &mut Option<&mut Vec<EnemyProjectile>>) {
    // ASM Enemy03_Robot_Shoot allocates the worker, then freezes both GfxObjects.
    push_robot_shot(enemy, projectiles);
    enemy.robot_state = RobotState::Shooting;
    enemy.wait_timer = 25;
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
    enemy.robot_gfx.body.selected = 9;
    enemy.robot_gfx.head.selected = 20;
}

/// ASM loc_3A884 (Robot.asm): enter the shared walker in the saved body direction.
fn step_robot_walk(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    if let Some(grid) = block_grid {
        if enemy.on_ground {
            // ASM loc_3C04E/loc_3C0FA scans the current pose, then compares
            // the proposed integer displacement with the next tile boundary.
            let x = (enemy.x_pos >> 16) as i32;
            let y = (enemy.y_pos >> 16) as i32;
            let width = pacer_probe_half_width(enemy.family);
            let height = pacer_probe_height(enemy.family);
            let moving_right = enemy.facing == Direction::Right;
            let step = horizontal_velocity(enemy.facing, robot_walk_speed(enemy.enemy_level));
            let scan = if moving_right {
                terrain::grounded_walker_scan_right_sub_36972(grid, x, y, width, height)
            } else {
                terrain::grounded_walker_scan_left_sub_36A58(grid, x, y, width, height)
            };
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            if scan.flags_d7 & 4 != 0 {
                // ASM loc_3C464 writes parent collision $FFFF and dispatches
                // Robot.asm loc_3A890 -> loc_3A9D2's destruction animation.
                register_enemy_terrain_kill(enemy);
                return;
            }
            let next_x = enemy.x_pos.wrapping_add_signed(step);
            let delta = ((next_x >> 16) as i32 - x).abs();
            let remainder = if moving_right {
                (-(x + width)) & 15
            } else {
                (x - width) & 15
            };
            let crosses_boundary = remainder < delta;
            let snap_into_fall = crosses_boundary
                && scan.flags_d7 & 2 != 0
                && scan.floor_row_mask_d5 == 0
                && i32::from(scan.column_span_d3) * 8 >= width;
            if scan.floor_class_d6 < 0 || snap_into_fall {
                if scan.floor_class_d6 >= 0 {
                    // ASM loc_3C04E/loc_3C0FA discards the fraction only on
                    // the boundary-snap arm before loc_3C17A/loc_3C266.
                    let snapped_x = if moving_right {
                        x + remainder
                    } else {
                        x - remainder
                    };
                    enemy.x_pos = (snapped_x as u32) << 16;
                }
                enemy.on_ground = false;
                enemy.y_pos = enemy.y_pos.wrapping_add(1 << 16);
                enemy.x_velocity = step;
                step_shared_walker_falling_loc_3c288(enemy, grid);
            } else if crosses_boundary && (scan.flags_d7 & 1 != 0 || scan.flags_d7 & 2 == 0) {
                // ASM loc_3C0A6/loc_3C152 -> loc_3C0D2/loc_3C026 turns
                // without committing the proposed step. Subpixels survive.
                reverse_enemy_direction(enemy);
                register_shared_walker_turnaround(enemy);
            } else {
                // ASM loc_3C0C6/loc_3C172 writes X directly, leaving Gfx
                // velocity zero. Only the rightward arm clears `$5A(a5)`.
                enemy.x_pos = next_x;
                if moving_right {
                    enemy.shared_walker_turnaround_counter = 0;
                }
            }
        } else {
            // ASM loc_3C19C/loc_3C288 owns the airborne continuation.
            step_shared_walker_falling_loc_3c288(enemy, grid);
        }
    }

    // ASM sub_3BF72 preserves an independent head only above enemy level zero.
    if enemy.enemy_level == 0 {
        enemy.robot_gfx.head_facing = Some(enemy.facing);
    }
}

/// ASM sub_3C4F8 / Robot.asm: patrol, independent head turn and shooting waits.
#[cfg(test)]
pub(crate) fn step_robot(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let mut rng = RomRandomState::default();
    step_robot_with_rng(enemy, block_grid, player_x_pos, projectiles, &mut rng);
}

pub(crate) fn step_robot_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
    rng: &mut RomRandomState,
) {
    enemy.robot_gfx.head_facing.get_or_insert(enemy.facing);
    match enemy.robot_state {
        RobotState::Walking => {
            // ASM loc_3C538/loc_3C554 consumes a random long only at C82F0/9.
            if enemy.robot_gfx.head.selected == 13
                && enemy.robot_gfx.head.timer == 9
                && let Some(px) = player_x_pos
            {
                let head = enemy.child_attachment_snapshot().expect("live Robot head");
                let result =
                    shared_enemy_robot_helper_sub_3C4F8(SharedEnemyRobotHelperInputSub3C4F8 {
                        helper_kind: SharedEnemyRobotHelperKindSub3C4F8::Robot,
                        parent_mapping: ROBOT_IDLE_MAPPING_SUB_3C4F8,
                        parent_state_word_32: u16::from(enemy.robot_gfx.head.timer),
                        child_state_word_40: u16::from(enemy.enemy_level),
                        child_counter_word_4e: 0,
                        child_timer_word_54: 0,
                        parent_x_pos: (head.x_pos >> 16) as u16,
                        walker_x_pos: (enemy.x_pos >> 16) as u16,
                        kid_x_pos: (px >> 16) as u16,
                        parent_x_direction: robot_x_direction_byte(robot_head_facing(enemy)),
                        parent_flag_15: true,
                        random_word: if enemy.enemy_level == 2 {
                            0
                        } else {
                            rng.next_long() as u16
                        },
                    });
                if result.branch_to_robot_shoot {
                    begin_robot_shoot(enemy, projectiles);
                    return;
                }
                if result.next_child_counter_word_4e != 0 {
                    // ASM loc_3C58E freezes only the head and burns the first turn tick.
                    enemy.robot_state = RobotState::Aiming;
                    enemy.wait_timer = ROBOT_AIM_TURN_FRAMES;
                    enemy.robot_gfx.head.selected = 21;
                }
            }
            step_robot_walk(enemy, block_grid);
        }
        RobotState::Aiming => {
            enemy.wait_timer = enemy.wait_timer.saturating_sub(1);
            if enemy.wait_timer == ROBOT_AIM_DIRECTION_FLIP_REMAINING {
                // ASM loc_3C5C4 flips `$16(a4)`, the head, independently of the body.
                enemy.robot_gfx.head_facing = Some(match robot_head_facing(enemy) {
                    Direction::Left => Direction::Right,
                    Direction::Right => Direction::Left,
                });
            }
            enemy.robot_gfx.head.selected = if (10..=18).contains(&enemy.wait_timer) {
                22
            } else {
                21
            };
            if enemy.wait_timer == 0 {
                // ASM loc_3C5D8 resets the head cursor before tail-jumping to Shoot.
                enemy.robot_gfx.head = RobotAnimationCursor::init(10);
                begin_robot_shoot(enemy, projectiles);
            } else {
                step_robot_walk(enemy, block_grid);
            }
        }
        RobotState::Shooting => {
            // ASM loc_3AA4C/loc_3AA72/loc_3AA88 wait 9, 5 and 11 frames.
            enemy.wait_timer = enemy.wait_timer.saturating_sub(1);
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            if enemy.wait_timer == 0 {
                finish_robot_shoot_sequence(enemy);
                step_robot_walk(enemy, block_grid);
            }
        }
        RobotState::Charging => {
            // ASM Robot.asm loc_3A9F0: charge persists across shared-walker
            // terrain turns. The random timer also flips direction in place.
            enemy.x_velocity = horizontal_velocity(enemy.facing, ROBOT_CHARGE_SPEED);
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

            if let Some(grid) = block_grid {
                let probe_half_width = pacer_probe_half_width(enemy.family);
                let probe_height = pacer_probe_height(enemy.family);
                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;
                let wall =
                    terrain::probe_wall(grid, cx, cy, probe_half_width, probe_height, moving_right);
                if wall.hit {
                    // ASM Robot.asm loc_3C04E/loc_3C0FA: the shared walker
                    // turns at the wall, hibernates, then loc_3A890 observes
                    // the retained `$800` collision word and resumes the
                    // charge timer through loc_3A9F0.
                    reverse_enemy_direction(enemy);
                } else {
                    let look_ahead_x = if moving_right {
                        cx + probe_half_width
                    } else {
                        cx - probe_half_width
                    };
                    if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                        // ASM loc_3C04E/loc_3C0FA: ledges use the same shared
                        // walker turn without clearing the charge collision.
                        reverse_enemy_direction(enemy);
                    }
                }
            }

            // Direction-flip timer: flip direction, stay charging
            // ASM reload: (random_byte >> 1) + $F (range 15-142)
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            } else {
                reverse_enemy_direction(enemy);
                enemy.wait_timer = robot_charge_flip_reload_with_rng(rng);
            }
            // ASM sub_3BF72: the `$800` charge collision word copies body direction.
            enemy.robot_gfx.head_facing = Some(enemy.facing);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn robot_live_animation_gate_runs_after_objects_and_obeys_haze() {
        // ASM loc_3C538 reads prior Gfx state; loc_147E throttles category 2.
        let mut enemies = [robot_test_enemy(
            RobotState::Walking,
            Direction::Right,
            100,
            80,
            0,
        )];
        let mut rng = RomRandomState::default();
        let mut shots = Vec::new();
        for time in 1..=116 {
            step_active_enemies_with_rng(
                &mut enemies,
                None,
                Some(140 << 16),
                Some(80 << 16),
                None,
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                time,
                true,
                Some(&mut shots),
                &mut rng,
                &mut Vec::new(),
                None,
                &mut std::collections::HashMap::new(),
            );
            // 28 eligible Gfx passes select C82F0 at frame 112; the next
            // eligible Object pass (116) consumes the default seed and shoots.
            assert_eq!(shots.len(), usize::from(time == 116), "frame {time}");
        }
        assert_eq!(enemies[0].snapshot().sprite_animation_phase, Some(9));
    }

    #[test]
    fn robot_signed_random_gate_draws_exactly_one_long() {
        // ASM loc_3C554 BLE accepts negative D7.W as well as 0..10.
        // Get_RandomNumber_long returns C092 in the default seed's low word.
        let mut robot = robot_test_enemy(RobotState::Walking, Direction::Right, 100, 80, 0);
        for _ in 0..28 {
            advance_robot_gfx(&mut robot);
        }
        let mut rng = RomRandomState::default();
        let mut expected = rng.clone();
        assert_eq!(expected.next_long() as u16, 0xC092);
        let mut shots = Vec::new();
        step_robot_with_rng(
            &mut robot,
            None,
            Some(140 << 16),
            &mut Some(&mut shots),
            &mut rng,
        );
        assert_eq!(shots.len(), 1);
        assert_eq!(rng.next_long(), expected.next_long());
    }

    #[test]
    fn robot_walk_does_not_draw_random_between_animation_gates() {
        // ASM sub_3C4F8 loc_3C538/loc_3C554: C82F0 and timer 9 gate RNG.
        let mut robot = robot_test_enemy(RobotState::Walking, Direction::Right, 100, 80, 0);
        let mut rng = RomRandomState::default();
        let expected = rng.clone().next_long();
        step_robot_with_rng(&mut robot, None, Some(140 << 16), &mut None, &mut rng);
        assert_eq!(rng.next_long(), expected);
    }

    #[test]
    fn robot_shoot_holds_body_for_twenty_five_frames() {
        // ASM Robot.asm loc_3AA4C/loc_3AA72/loc_3AA88: DBF waits 9 + 5 + 11.
        let mut robot = robot_test_enemy(RobotState::Aiming, Direction::Right, 100, 80, 1);
        let mut shots = Vec::new();
        step_robot(&mut robot, None, Some(140 << 16), &mut Some(&mut shots));
        for _ in 0..24 {
            step_robot(&mut robot, None, Some(140 << 16), &mut Some(&mut shots));
            assert_eq!(robot.snapshot().sprite_animation_phase, Some(9));
            assert_eq!(robot.x_pos, 100 << 16);
        }
        step_robot(&mut robot, None, Some(140 << 16), &mut Some(&mut shots));
        assert_eq!(robot.robot_state, RobotState::Walking);
        assert_eq!(shots.len(), 1);
    }

    #[test]
    fn robot_recovery_resumes_saved_head_cursor_and_replays_body_entry() {
        // ASM loc_3AA9E; Crypt oracle frames 322..339: body C82D0 for 17
        // recovery frames, head C8328 for eight, then C82F8 for nine.
        let mut robot = robot_test_enemy(RobotState::Walking, Direction::Right, 100, 80, 0);
        for _ in 0..28 {
            advance_robot_gfx(&mut robot);
        }
        assert_eq!(
            (robot.robot_gfx.head.selected, robot.robot_gfx.head.timer),
            (13, 9)
        );
        begin_robot_shoot(&mut robot, &mut None);
        finish_robot_shoot_sequence(&mut robot);
        for frame in 0..18 {
            advance_robot_gfx(&mut robot);
            assert_eq!(
                robot.snapshot().sprite_animation_phase,
                Some(if frame < 17 { 5 } else { 0 })
            );
            assert_eq!(
                robot
                    .child_attachment_snapshot()
                    .unwrap()
                    .sprite_animation_phase,
                Some(if frame < 8 {
                    20
                } else if frame < 17 {
                    14
                } else {
                    15
                })
            );
        }
    }

    fn robot_test_enemy(
        robot_state: RobotState,
        facing: Direction,
        x_px: u32,
        y_px: u32,
        wait_timer: u16,
    ) -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Robot,
            robot_state,
            facing,
            hit_points: 2,
            wait_timer,
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn grid_with_solid_cells(cells: &[(u16, u16)]) -> terrain::BlockGrid {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 4 * 2], 8, 4);
        for &(col, row) in cells {
            grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0x60, 0x00));
        }
        grid
    }

    fn rng_at_robot_random_gate() -> RomRandomState {
        let mut rng = RomRandomState::default();
        while (rng.clone().next_long() as u16) > 0x000A {
            rng.next_long();
        }
        rng
    }

    #[test]
    fn robot_shot_impact_animation_frame_count_matches_stru_3a762() {
        assert_eq!(
            ROBOT_SHOT_IMPACT_ANIMATION_FRAMES_LOC_3A762, 15,
            "ASM Robot.asm loc_3AC92 starts `stru_3A762`, whose impact animation lasts 3 + 4 + 8 frames",
        );
    }

    #[test]
    fn robot_collision_dispatch_child_zero_and_default_route_clear_loc_3a958() {
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0, 0, false),
            RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
            "ASM Robot.asm loc_3A8AE: `beq.w loc_3A958` clears the child/parent collision words",
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0, 0x0010, false),
            RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
            "ASM Robot.asm loc_3A8AE: unhandled positive child collision types also fall through to loc_3A958",
        );
    }

    #[test]
    fn robot_live_step_allocates_hidden_shot_without_sound() {
        let robot = ActiveEnemy {
            family: EnemyFamily::Robot,
            robot_state: RobotState::Aiming,
            wait_timer: 1,
            facing: Direction::Right,
            hit_points: 2,
            x_pos: 100 << 16,
            y_pos: 80 << 16,
            ..mini_hopping_skull_test_enemy()
        };
        let mut enemies = [robot];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            Some(140 << 16),
            Some(80 << 16),
            None,
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            0,
            false,
            Some(&mut projectiles),
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
            &mut std::collections::HashMap::new(),
        );

        assert_eq!(
            projectiles.len(),
            1,
            "ASM Robot.asm loc_3AC0C should allocate the shot object when the aiming sequence reaches its shoot tail",
        );
        let projectile = projectiles[0];
        assert_eq!(projectile.x_pos, 126 << 16);
        assert_eq!(projectile.y_pos, 66 << 16);
        assert_eq!(projectile.x_velocity, ROBOT_PROJECTILE_SPEED);
        assert_eq!(projectile.y_velocity, 0);
        assert_eq!(projectile.lifetime, 60);
        assert_eq!(projectile.gravity, 0);
        assert!(!projectile.friendly);
        assert_eq!(projectile.kind, EnemyProjectileKind::RobotShot);
        assert_eq!(projectile.homing_accel, 0);
        assert_eq!(projectile.speed_cap, 0);
        assert!(!projectile.wall_bounce);
        assert!(
            sound_effects.is_empty(),
            "ASM Enemy03_Robot_Shot_Init hibernates before Gfx and sound"
        );
        assert!(!projectile.robot_shot_has_gfx_object());
    }

    #[test]
    fn robot_shot_spawn_loc_3ac0c_copies_and_wraps_position_words() {
        let robot = ActiveEnemy {
            family: EnemyFamily::Robot,
            facing: Direction::Right,
            x_pos: (0xFFF0 << 16) | 0xBEEF,
            y_pos: (5 << 16) | 0xCAFE,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        push_robot_shot(&robot, &mut Some(&mut projectiles));

        assert_eq!(
            projectiles[0].x_pos, 0x000A_0000,
            "ASM Robot.asm loc_3AC0C copies the X word and wraps ADD.W #$1A"
        );
        assert_eq!(
            projectiles[0].y_pos, 0xFFF7_0000,
            "ASM Robot.asm loc_3AC0C copies the Y word and wraps the muzzle-height SUB.W"
        );
    }

    #[test]
    fn robot_aiming_zero_wait_shoots_without_underflowing_loc_3aa9e_3aac4() {
        let mut robot = robot_test_enemy(RobotState::Aiming, Direction::Right, 100, 80, 0);
        let mut rng = RomRandomState::default();
        let mut projectiles = Vec::new();

        step_robot_with_rng(
            &mut robot,
            None,
            Some(140 << 16),
            &mut Some(&mut projectiles),
            &mut rng,
        );

        assert_eq!(
            projectiles.len(),
            1,
            "ASM Robot.asm loc_3AAC4 should shoot immediately once the aiming wait reaches zero",
        );
        assert_eq!(robot.robot_state, RobotState::Shooting);
        assert_eq!(robot.wait_timer, 25);
    }

    #[test]
    fn robot_walk_turns_at_crossed_tile_boundary_without_committing_step() {
        // ASM loc_3C04E/loc_3C0FA checks the predicted integer displacement
        // against the leading-edge remainder before loc_3C0D2/loc_3C026.
        // Crypt oracle turns at X 149.5 right and X 139 left on a 32px ledge.
        let grid = grid_with_solid_cells(&[(2, 2), (3, 2)]);
        for (facing, start, expected_positions) in [
            (
                Direction::Right,
                (52 << 16) | 0x8000,
                [53 << 16, (53 << 16) | 0x8000, (53 << 16) | 0x8000],
            ),
            (
                Direction::Left,
                44 << 16,
                [(43 << 16) | 0x8000, 43 << 16, 43 << 16],
            ),
        ] {
            let mut robot = robot_test_enemy(RobotState::Walking, facing, 0, 32, 0);
            robot.x_pos = start;
            robot.on_ground = true;
            for (i, expected) in expected_positions.into_iter().enumerate() {
                step_robot(&mut robot, Some(&grid), None, &mut None);
                assert_eq!(robot.x_pos, expected, "{facing:?} tick {i}");
                assert_eq!(robot.x_velocity, 0, "ASM grounded walker writes X directly");
                let expected_facing = if i < 2 {
                    facing
                } else if facing == Direction::Left {
                    Direction::Right
                } else {
                    Direction::Left
                };
                assert_eq!(robot.facing, expected_facing, "tick {i}");
            }
        }
    }

    #[test]
    fn robot_walk_wall_turn_preserves_pre_crossing_position() {
        // ASM loc_3C0A6/loc_3C152 rejects the crossing step when d7 bit 0
        // reports a leading wall. The body has not entered that wall yet.
        for (facing, x, wall, expected_facing) in [
            (
                Direction::Right,
                (53 << 16) | 0x8000,
                (4, 1),
                Direction::Left,
            ),
            (Direction::Left, 75 << 16, (3, 1), Direction::Right),
        ] {
            let mut grid = grid_with_solid_cells(&[wall]);
            for col in 0..8 {
                grid.set_cell_at(col, 2, terrain::Cell::from_be_bytes(0x60, 0));
            }
            let mut robot = robot_test_enemy(RobotState::Walking, facing, 0, 32, 10);
            robot.x_pos = x;
            step_robot(&mut robot, Some(&grid), None, &mut None);
            assert_eq!(robot.x_pos, x);
            assert_eq!(robot.facing, expected_facing);
            assert!(!robot.death_animation_active);
        }
    }

    #[test]
    fn robot_walk_body_overlap_routes_to_destruction() {
        // ASM loc_3C04E tests d7 bit 2 before movement; loc_3C464 sends
        // collision $FFFF through Robot.asm loc_3A890 -> loc_3A9D2.
        let grid = grid_with_solid_cells(&[(4, 1)]);
        let mut robot = robot_test_enemy(RobotState::Walking, Direction::Right, 56, 32, 10);
        step_robot(&mut robot, Some(&grid), None, &mut None);
        assert_eq!(robot.x_pos, 56 << 16);
        assert!(robot.death_animation_active);
        assert_eq!(robot.wait_timer, 18);
    }

    #[test]
    fn robot_walk_preserves_level_speed_after_damage_loc_3a7ee() {
        let floor = &[
            (0, 2),
            (1, 2),
            (2, 2),
            (3, 2),
            (4, 2),
            (5, 2),
            (6, 2),
            (7, 2),
        ];
        let grid = grid_with_solid_cells(floor);
        let mut robot = robot_test_enemy(RobotState::Walking, Direction::Right, 48, 32, 10);
        robot.enemy_level = 1;
        robot.hit_points = 2;

        step_robot(&mut robot, Some(&grid), None, &mut None);

        assert_eq!(
            robot.x_pos,
            49 << 16,
            "ASM Robot.asm loc_3A7AC/loc_3A7EE stores the level-1 `$10000` speed at init, so damage must not reduce later walking steps",
        );
    }

    #[test]
    fn robot_walk_wall_scan_uses_loc_3a7ac_probe_height() {
        // ASM loc_3A7AC seeds `$4A(a5)=$20`; sub_36972 must see the
        // upper wall row before loc_3C0A6 turns at the tile boundary.
        let mut grid = grid_with_solid_cells(&[(4, 1)]);
        for col in 0..8 {
            grid.set_cell_at(col, 3, terrain::Cell::from_be_bytes(0x60, 0));
        }
        let mut robot = robot_test_enemy(RobotState::Walking, Direction::Right, 53, 48, 10);
        robot.x_pos |= 0x8000;
        step_robot(&mut robot, Some(&grid), None, &mut None);
        assert_eq!(robot.x_pos, (53 << 16) | 0x8000);
        assert_eq!(robot.facing, Direction::Left);
        assert!(!robot.death_animation_active);
    }

    #[test]
    fn robot_walk_floor_lookahead_uses_leading_edge_for_both_directions() {
        let cases = [
            (Direction::Right, 80, &[(5, 2)][..]),
            (Direction::Left, 80, &[(4, 2)][..]),
        ];

        for (facing, x_px, cells) in cases {
            let grid = grid_with_solid_cells(cells);
            let mut robot = robot_test_enemy(RobotState::Walking, facing, x_px, 32, 10);

            step_robot(&mut robot, Some(&grid), None, &mut None);

            assert_eq!(
                robot.facing, facing,
                "ASM Robot.asm loc_3A884 shared walker should keep direction when the leading-edge floor probe is grounded",
            );
        }
    }

    #[test]
    fn robot_walk_shoot_helper_uses_robot_integer_x_for_loc_3c56a() {
        let mut robot = robot_test_enemy(RobotState::Walking, Direction::Left, 0x0200, 80, 0);
        robot.enemy_level = 1;
        for _ in 0..28 {
            advance_robot_gfx(&mut robot);
        }
        let mut rng = rng_at_robot_random_gate();
        let mut projectiles = Vec::new();

        step_robot_with_rng(
            &mut robot,
            None,
            Some(0x0140 << 16),
            &mut Some(&mut projectiles),
            &mut rng,
        );

        assert_eq!(
            projectiles.len(),
            1,
            "ASM sub_3C4F8 loc_3C56A should see Robot right of Kid and facing left, then branch to Enemy03_Robot_Shoot",
        );
        assert_eq!(projectiles[0].x_velocity, -ROBOT_PROJECTILE_SPEED);
        assert_eq!(robot.robot_state, RobotState::Shooting);
    }

    #[test]
    fn robot_charging_advances_at_charge_speed_loc_3a96c() {
        let mut robot = robot_test_enemy(RobotState::Charging, Direction::Right, 80, 32, 1);
        let start_x = robot.x_pos;

        step_robot(&mut robot, None, None, &mut None);

        assert_eq!(
            robot.x_pos,
            start_x + ROBOT_CHARGE_SPEED as u32,
            "ASM Robot.asm loc_3A96C seeds `$50(a5)=$28000`, and the charge loop advances by that speed each frame",
        );
        assert_eq!(robot.robot_state, RobotState::Charging);
    }

    #[test]
    fn robot_charge_wall_turn_retains_collision_state_loc_3a9f0() {
        let grid = grid_with_solid_cells(&[(4, 1)]);
        let mut robot = robot_test_enemy(RobotState::Charging, Direction::Right, 56, 32, 10);

        step_robot(&mut robot, Some(&grid), None, &mut None);

        assert_eq!(robot.facing, Direction::Left);
        assert_eq!(
            robot.robot_state,
            RobotState::Charging,
            "ASM Robot.asm loc_3C04E turns at the wall, then loc_3A890 retains `$800` and resumes the charge through loc_3A9F0",
        );
        assert_eq!(
            robot.wait_timer, 9,
            "ASM Robot.asm loc_3A9F0 should continue decrementing the charge flip timer after the shared-walker turn",
        );
    }

    #[test]
    fn robot_charge_ledge_turn_retains_collision_state_loc_3a9f0() {
        let grid = grid_with_solid_cells(&[(4, 2)]);
        let mut robot = robot_test_enemy(RobotState::Charging, Direction::Right, 80, 32, 10);

        step_robot(&mut robot, Some(&grid), None, &mut None);

        assert_eq!(robot.facing, Direction::Left);
        assert_eq!(
            robot.robot_state,
            RobotState::Charging,
            "ASM Robot.asm loc_3C04E turns at a ledge, then loc_3A890 retains `$800` and resumes the charge through loc_3A9F0",
        );
        assert_eq!(robot.wait_timer, 9);
    }

    #[test]
    fn robot_charging_floor_lookahead_uses_leading_edge_for_both_directions() {
        let cases = [
            (Direction::Right, 80, &[(5, 2)][..]),
            (Direction::Left, 80, &[(4, 2)][..]),
        ];

        for (facing, x_px, cells) in cases {
            let grid = grid_with_solid_cells(cells);
            let mut robot = robot_test_enemy(RobotState::Charging, facing, x_px, 32, 10);

            step_robot(&mut robot, Some(&grid), None, &mut None);

            assert_eq!(
                robot.robot_state,
                RobotState::Charging,
                "ASM Robot.asm charge should remain active while the leading-edge floor probe is grounded",
            );
            assert_eq!(robot.facing, facing);
        }
    }
}
