use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PendingDragonDeathChild {
    pub(crate) remaining_frames: u16,
    pub(crate) child: ActiveEnemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PendingDragonDeathAnimation {
    pub(crate) remaining_frames: u16,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) facing: Direction,
    pub(crate) palette_variant: u8,
    pub(crate) layout_index: usize,
    pub(crate) status_route: DragonFlyingDeathStatusRouteLoc3492C,
}

impl PendingDragonDeathAnimation {
    pub(crate) fn snapshot(&self) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: EnemyFamily::Dragon.object_type(),
            },
            x_pos: self.x_pos,
            y_pos: self.y_pos,
            layout_index: Some(self.layout_index),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(self.facing),
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: self.palette_variant,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            sprite_animation_phase: None,
            wait_timer: Some(self.remaining_frames),
            debug_fields: std::collections::BTreeMap::new(),
        }
    }
}

/// Drift acceleration per frame: $200. ASM loc_3478C.
const DRAGON_DRIFT_ACCEL: i32 = 0x200;
/// Drift speed cap: $18000. ASM loc_3478C.
const DRAGON_DRIFT_SPEED_CAP: i32 = 0x1_8000;
/// Return-to-anchor deceleration per frame: $400. ASM loc_34B4C.
const DRAGON_RETURN_DECEL: i32 = 0x400;
/// Return-to-anchor intermediate Y velocity cap: $FFFF1000. ASM loc_34B4C.
const DRAGON_RETURN_VEL_CAP_SOFT: i32 = -0xF000_i32; // $FFFF1000 as signed
/// Return-to-anchor hard Y velocity cap: $FFFF8000. ASM loc_34B70.
const DRAGON_RETURN_VEL_CAP_HARD: i32 = -0x8000_i32; // $FFFF8000 as signed
/// Face-player initial x velocity: $8000. ASM loc_3473A.
const DRAGON_FACE_PLAYER_SPEED: i32 = 0x8000;
/// Right terrain slide horizontal deceleration per frame: $1000. ASM loc_34A0A.
const DRAGON_TERRAIN_SLIDE_X_DECEL: i32 = 0x1000;
/// Upward terrain slide vertical acceleration per frame: $5000. ASM loc_34A4E.
const DRAGON_TERRAIN_UP_SLIDE_Y_ACCEL: i32 = 0x5000;
/// Downward terrain slide vertical deceleration per frame: $2000. ASM loc_34A6E.
const DRAGON_TERRAIN_DOWN_SLIDE_Y_DECEL: i32 = 0x2000;
/// Post-snap rebound vertical acceleration per frame: $2000. ASM loc_34AC2.
const DRAGON_TERRAIN_REBOUND_Y_ACCEL: i32 = 0x2000;

const DRAGON_TERRAIN_REBOUND_START_WAIT_FRAMES: u16 = 2;

pub(crate) fn dragon_drift_positive_velocity_loc_3478c(x_velocity: i32) -> i32 {
    // ASM Dragon_Flying.asm loc_3478C: positive drift adds `$200` to d7 and
    // clamps at `$18000`; the long-word addition wraps before that comparison.
    x_velocity
        .wrapping_add(DRAGON_DRIFT_ACCEL)
        .min(DRAGON_DRIFT_SPEED_CAP)
}

pub(crate) fn dragon_drift_negative_velocity_loc_347b2(x_velocity: i32) -> i32 {
    // ASM Dragon_Flying.asm loc_347B2: negative drift subtracts `$200`
    // from d7 and clamps at `$FFFE8000` (-$18000); the long-word subtraction
    // wraps before that comparison.
    x_velocity
        .wrapping_sub(DRAGON_DRIFT_ACCEL)
        .max(-DRAGON_DRIFT_SPEED_CAP)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonDriftStoreLoc347C8 {
    pub(crate) x_velocity: i32,
    pub(crate) checks_fire_breath_cone: bool,
}

pub(crate) fn dragon_drift_store_loc_347c8(
    d7_velocity: i32,
    d1_fire_gate: u16,
) -> DragonDriftStoreLoc347C8 {
    // ASM Dragon_Flying.asm loc_347C8: store prepared d7 into x_vel, then
    // call loc_34836 only when d1 is non-zero.
    DragonDriftStoreLoc347C8 {
        x_velocity: d7_velocity,
        checks_fire_breath_cone: d1_fire_gate != 0,
    }
}

pub(crate) fn dragon_return_y_velocity_loc_34b4c(y_velocity: i32) -> i32 {
    // ASM Dragon_Flying.asm loc_34B4C: subtract `$400`, keep values at or above
    // `$FFFF1000`, otherwise store the hard `$FFFF8000` upward cap at loc_34B70.
    // The long-word subtraction wraps before the signed comparison.
    let y_velocity = y_velocity.wrapping_sub(DRAGON_RETURN_DECEL);
    if y_velocity < DRAGON_RETURN_VEL_CAP_SOFT {
        DRAGON_RETURN_VEL_CAP_HARD
    } else {
        y_velocity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonDriftDbfTailLoc347FC {
    pub(crate) next_d0: u16,
    pub(crate) loops_to_loc_3478c: bool,
    pub(crate) advances_script: bool,
}

pub(crate) fn dragon_drift_dbf_tail_loc_347fc(d0_before_dbf: u16) -> DragonDriftDbfTailLoc347FC {
    // ASM Dragon_Flying.asm loc_347FC: `dbf d0,loc_3478C` decrements the
    // low word, loops until it becomes `$FFFF`, then falls through to loc_346DA.
    let next_d0 = d0_before_dbf.wrapping_sub(1);
    let loops_to_loc_3478c = next_d0 != 0xFFFF;
    DragonDriftDbfTailLoc347FC {
        next_d0,
        loops_to_loc_3478c,
        advances_script: !loops_to_loc_3478c,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonHoverDbfTailLoc3482E {
    pub(crate) next_d0: u16,
    pub(crate) loops_to_loc_3480a: bool,
    pub(crate) advances_script: bool,
}

pub(crate) fn dragon_hover_dbf_tail_loc_3482e(d0_before_dbf: u16) -> DragonHoverDbfTailLoc3482E {
    // ASM Dragon_Flying.asm loc_3482E: `dbf d0,loc_3480A` uses the same
    // low-word decrement/fall-through rule as the drift tail, but targets the
    // hover hibernation loop.
    let next_d0 = d0_before_dbf.wrapping_sub(1);
    let loops_to_loc_3480a = next_d0 != 0xFFFF;
    DragonHoverDbfTailLoc3482E {
        next_d0,
        loops_to_loc_3480a,
        advances_script: !loops_to_loc_3480a,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonDescentMoveSetupLoc348E4 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) dbf_counter_after_subq: u16,
}

pub(crate) fn dragon_descent_move_setup_loc_348e4(
    facing: Direction,
    left_facing_x_vel_px: i8,
    y_vel_px: i8,
    frame_count: u8,
) -> DragonDescentMoveSetupLoc348E4 {
    // ASM Dragon_Flying.asm loc_348E4: after loc_348CE decodes signed
    // byte-sized x/y velocities, the x word is written to `x_vel`; the next
    // script byte supplies d2 and `subq.w #1,d2` seeds the DBF loop counter.
    let left_facing_x_velocity = i32::from(left_facing_x_vel_px) << 16;
    let x_velocity = match facing {
        Direction::Right => -left_facing_x_velocity,
        Direction::Left => left_facing_x_velocity,
    };
    DragonDescentMoveSetupLoc348E4 {
        x_velocity,
        y_velocity: i32::from(y_vel_px) << 16,
        dbf_counter_after_subq: u16::from(frame_count).wrapping_sub(1),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonDescentCollisionOutcomeLoc348FC {
    ContinueLoop,
    Explosion,
    TerrainResponse(u16),
}

pub(crate) fn dragon_descent_collision_outcome_loc_348fc(
    collision_type: u16,
) -> DragonDescentCollisionOutcomeLoc348FC {
    // ASM Dragon_Flying.asm loc_348FC: after one hibernated descent frame,
    // zero collision falls into the DBF tail, negative/$1C/$2C collisions use
    // the death/explosion path, and other positive collisions use terrain response.
    if collision_type == 0 {
        DragonDescentCollisionOutcomeLoc348FC::ContinueLoop
    } else if (collision_type & 0x8000) != 0 || matches!(collision_type, 0x001C | 0x002C) {
        DragonDescentCollisionOutcomeLoc348FC::Explosion
    } else {
        DragonDescentCollisionOutcomeLoc348FC::TerrainResponse(collision_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonDescentDbfTailLoc34924 {
    pub(crate) next_d2: u16,
    pub(crate) loops_to_loc_348fc: bool,
    pub(crate) advances_descent_script: bool,
}

pub(crate) fn dragon_descent_dbf_tail_loc_34924(
    d2_before_dbf: u16,
) -> DragonDescentDbfTailLoc34924 {
    // ASM Dragon_Flying.asm loc_34924: `dbf d2,loc_348FC` decrements the
    // movement-frame counter, loops while it is not `$FFFF`, then falls through
    // to the next descent script word.
    let next_d2 = d2_before_dbf.wrapping_sub(1);
    let loops_to_loc_348fc = next_d2 != 0xFFFF;
    DragonDescentDbfTailLoc34924 {
        next_d2,
        loops_to_loc_348fc,
        advances_descent_script: !loops_to_loc_348fc,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonTerrainDispatchLoc349DE {
    DeathExplosion {
        collision_cleared: bool,
    },
    RestartDescentScript {
        collision_cleared: bool,
    },
    ResponseTable {
        collision_cleared: bool,
        byte_offset: u16,
    },
}

#[allow(non_snake_case)]
pub(crate) fn dragon_terrain_dispatch_loc_349DE(
    collision_type: u16,
) -> DragonTerrainDispatchLoc349DE {
    // ASM Dragon_Flying.asm loc_349DE: negative collision branches to the
    // death path before clearing collision_type; non-negative collision clears
    // it, `$1C+` restarts the descent script, and lower values dispatch by
    // subtracting four from the collision word for the response-table offset.
    if (collision_type & 0x8000) != 0 {
        DragonTerrainDispatchLoc349DE::DeathExplosion {
            collision_cleared: false,
        }
    } else if collision_type >= 0x001C {
        DragonTerrainDispatchLoc349DE::RestartDescentScript {
            collision_cleared: true,
        }
    } else {
        DragonTerrainDispatchLoc349DE::ResponseTable {
            collision_cleared: true,
            byte_offset: collision_type.wrapping_sub(4),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonRightSlideStartLoc34A00 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

pub(crate) fn dragon_right_slide_start_loc_34a00(
    previous_x_velocity: i32,
) -> DragonRightSlideStartLoc34A00 {
    // ASM Dragon_Flying.asm loc_34A00: clear.l y_vel, then move.w #2,x_vel.
    // The word write targets the high word of the FP16.16 long, preserving the
    // previous low fractional word before the following deceleration loop.
    DragonRightSlideStartLoc34A00 {
        x_velocity: dragon_velocity_with_high_word(previous_x_velocity, 2),
        y_velocity: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonRightSlideFrameLoc34A0A {
    CollisionRoute {
        x_velocity: i32,
        collision_type: u16,
    },
    Continue {
        x_velocity: i32,
    },
    BeginReturnToAnchor {
        x_velocity: i32,
    },
}

pub(crate) fn dragon_right_slide_frame_loc_34a0a(
    x_velocity: i32,
    collision_type: u16,
) -> DragonRightSlideFrameLoc34A0A {
    // ASM Dragon_Flying.asm loc_34A0A: after one hibernated frame, non-zero
    // collision_type branches to the collision handler before velocity changes;
    // otherwise x_vel subtracts `$1000` and signed `ble` enters return-to-anchor.
    if collision_type != 0 {
        return DragonRightSlideFrameLoc34A0A::CollisionRoute {
            x_velocity,
            collision_type,
        };
    }

    let next_x_velocity = x_velocity.wrapping_sub(DRAGON_TERRAIN_SLIDE_X_DECEL);
    if next_x_velocity <= 0 {
        DragonRightSlideFrameLoc34A0A::BeginReturnToAnchor {
            x_velocity: next_x_velocity,
        }
    } else {
        DragonRightSlideFrameLoc34A0A::Continue {
            x_velocity: next_x_velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonLeftSlideStartLoc34A24 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

pub(crate) fn dragon_left_slide_start_loc_34a24(
    previous_x_velocity: i32,
) -> DragonLeftSlideStartLoc34A24 {
    // ASM Dragon_Flying.asm loc_34A24: clear.l y_vel, then move.w #$FFFE,x_vel.
    // Like the mirrored right-slide setup, the word write changes only the
    // high word of the FP16.16 long and preserves the previous fractional word.
    DragonLeftSlideStartLoc34A24 {
        x_velocity: dragon_velocity_with_high_word(previous_x_velocity, -2),
        y_velocity: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonLeftSlideFrameLoc34A2E {
    CollisionRoute {
        x_velocity: i32,
        collision_type: u16,
    },
    Continue {
        x_velocity: i32,
    },
    BeginReturnToAnchor {
        x_velocity: i32,
    },
}

pub(crate) fn dragon_left_slide_frame_loc_34a2e(
    x_velocity: i32,
    collision_type: u16,
) -> DragonLeftSlideFrameLoc34A2E {
    // ASM Dragon_Flying.asm loc_34A2E: after one hibernated frame, non-zero
    // collision_type branches to the collision handler before velocity changes;
    // otherwise x_vel adds `$1000` and signed `bge` enters return-to-anchor.
    if collision_type != 0 {
        return DragonLeftSlideFrameLoc34A2E::CollisionRoute {
            x_velocity,
            collision_type,
        };
    }

    let next_x_velocity = x_velocity.wrapping_add(DRAGON_TERRAIN_SLIDE_X_DECEL);
    if next_x_velocity >= 0 {
        DragonLeftSlideFrameLoc34A2E::BeginReturnToAnchor {
            x_velocity: next_x_velocity,
        }
    } else {
        DragonLeftSlideFrameLoc34A2E::Continue {
            x_velocity: next_x_velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonUpSlideStartLoc34A48 {
    pub(crate) y_velocity: i32,
}

pub(crate) fn dragon_up_slide_start_loc_34a48(
    previous_y_velocity: i32,
) -> DragonUpSlideStartLoc34A48 {
    // ASM Dragon_Flying.asm loc_34A48: move.w #$FFFD,y_vel writes only the
    // high word of the FP16.16 long, preserving the previous fractional word.
    DragonUpSlideStartLoc34A48 {
        y_velocity: dragon_velocity_with_high_word(previous_y_velocity, -3),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonUpSlideFrameLoc34A4E {
    CollisionRoute {
        y_velocity: i32,
        collision_type: u16,
    },
    Continue {
        y_velocity: i32,
    },
    BeginReturnToAnchor {
        y_velocity: i32,
    },
}

pub(crate) fn dragon_up_slide_frame_loc_34a4e(
    y_velocity: i32,
    collision_type: u16,
) -> DragonUpSlideFrameLoc34A4E {
    // ASM Dragon_Flying.asm loc_34A4E: after one hibernated frame, non-zero
    // collision_type branches to the collision handler before velocity changes;
    // otherwise y_vel adds `$5000` and signed `bge` enters return-to-anchor.
    if collision_type != 0 {
        return DragonUpSlideFrameLoc34A4E::CollisionRoute {
            y_velocity,
            collision_type,
        };
    }

    let next_y_velocity = y_velocity.wrapping_add(DRAGON_TERRAIN_UP_SLIDE_Y_ACCEL);
    if next_y_velocity >= 0 {
        DragonUpSlideFrameLoc34A4E::BeginReturnToAnchor {
            y_velocity: next_y_velocity,
        }
    } else {
        DragonUpSlideFrameLoc34A4E::Continue {
            y_velocity: next_y_velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonDownSlideStartLoc34A68 {
    pub(crate) y_velocity: i32,
}

pub(crate) fn dragon_down_slide_start_loc_34a68(
    previous_y_velocity: i32,
) -> DragonDownSlideStartLoc34A68 {
    // ASM Dragon_Flying.asm loc_34A68: move.w #3,y_vel writes only the
    // high word of the FP16.16 long, preserving the previous fractional word.
    DragonDownSlideStartLoc34A68 {
        y_velocity: dragon_velocity_with_high_word(previous_y_velocity, 3),
    }
}

fn dragon_velocity_with_high_word(previous_velocity: i32, high_word: i16) -> i32 {
    (i32::from(high_word) << 16).wrapping_add(previous_velocity & 0x0000_FFFF)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonDownSlideFrameLoc34A6E {
    CollisionRoute {
        y_velocity: i32,
        collision_type: u16,
    },
    Continue {
        y_velocity: i32,
    },
    BeginReturnToAnchor {
        y_velocity: i32,
    },
}

#[allow(non_snake_case)]
pub(crate) fn dragon_down_slide_frame_loc_34A6E(
    y_velocity: i32,
    collision_type: u16,
) -> DragonDownSlideFrameLoc34A6E {
    // ASM Dragon_Flying.asm loc_34A6E: after one hibernated frame, non-zero
    // collision_type branches to the collision handler before velocity changes;
    // otherwise y_vel subtracts `$2000` and signed `ble` enters return-to-anchor.
    if collision_type != 0 {
        return DragonDownSlideFrameLoc34A6E::CollisionRoute {
            y_velocity,
            collision_type,
        };
    }

    let next_y_velocity = y_velocity.wrapping_sub(DRAGON_TERRAIN_DOWN_SLIDE_Y_DECEL);
    if next_y_velocity <= 0 {
        DragonDownSlideFrameLoc34A6E::BeginReturnToAnchor {
            y_velocity: next_y_velocity,
        }
    } else {
        DragonDownSlideFrameLoc34A6E::Continue {
            y_velocity: next_y_velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonFlyingReboundStartLoc34A88 {
    pub(crate) snap_x_pos: u32,
    pub(crate) snap_y_pos: u32,
    pub(crate) x_velocity_during_snap: i32,
    pub(crate) y_velocity_during_snap: i32,
    pub(crate) level_collision_during_first_hibernate: bool,
    pub(crate) level_collision_after_snap: bool,
    pub(crate) hibernate_frames_before_rebound_velocity: u16,
    pub(crate) animation_label: &'static str,
    pub(crate) rebound_x_velocity: i32,
    pub(crate) rebound_y_velocity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonFlyingSnapBackCoordsLoc34B96 {
    pub(crate) snap_x_pos: u32,
    pub(crate) snap_y_pos: u32,
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_snap_back_coords_loc_34B96(
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    y_velocity: i32,
) -> DragonFlyingSnapBackCoordsLoc34B96 {
    // ASM Dragon_Flying.asm loc_34B96: negate x_vel/y_vel into d0/d1,
    // add the live x_pos/y_pos, and return those prepared snap coordinates.
    DragonFlyingSnapBackCoordsLoc34B96 {
        snap_x_pos: x_pos.wrapping_add(u32::from_ne_bytes(x_velocity.wrapping_neg().to_ne_bytes())),
        snap_y_pos: y_pos.wrapping_add(u32::from_ne_bytes(y_velocity.wrapping_neg().to_ne_bytes())),
    }
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_rebound_start_loc_34A88(
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    y_velocity: i32,
) -> DragonFlyingReboundStartLoc34A88 {
    let snap = dragon_flying_snap_back_coords_loc_34B96(x_pos, y_pos, x_velocity, y_velocity);
    // ASM Dragon_Flying.asm loc_34A88: a shared helper prepares `pos - velocity`,
    // then the dragon hibernates once with level collision off, snaps to that
    // prepared position, hibernates once with collision back on, and starts
    // `stru_34652` with x_vel/y_vel word writes of `$FFFE`.
    DragonFlyingReboundStartLoc34A88 {
        snap_x_pos: snap.snap_x_pos,
        snap_y_pos: snap.snap_y_pos,
        x_velocity_during_snap: 0,
        y_velocity_during_snap: 0,
        level_collision_during_first_hibernate: false,
        level_collision_after_snap: true,
        hibernate_frames_before_rebound_velocity: 2,
        animation_label: "stru_34652",
        rebound_x_velocity: -0x0002_0000,
        rebound_y_velocity: -0x0002_0000,
    }
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_right_rebound_start_loc_34ADC(
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    y_velocity: i32,
) -> DragonFlyingReboundStartLoc34A88 {
    let snap = dragon_flying_snap_back_coords_loc_34B96(x_pos, y_pos, x_velocity, y_velocity);
    // ASM Dragon_Flying.asm loc_34ADC: same snap/hibernate sequence as
    // loc_34A88, but the rebound starts with `move.w #2,x_vel(a3)`.
    DragonFlyingReboundStartLoc34A88 {
        snap_x_pos: snap.snap_x_pos,
        snap_y_pos: snap.snap_y_pos,
        x_velocity_during_snap: 0,
        y_velocity_during_snap: 0,
        level_collision_during_first_hibernate: false,
        level_collision_after_snap: true,
        hibernate_frames_before_rebound_velocity: 2,
        animation_label: "stru_34652",
        rebound_x_velocity: 0x0002_0000,
        rebound_y_velocity: -0x0002_0000,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonFlyingReboundFrameLoc34AC2 {
    BeginReturnToAnchor {
        y_velocity: i32,
    },
    CollisionRoute {
        y_velocity: i32,
        collision_type: u16,
    },
    Continue {
        y_velocity: i32,
    },
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_rebound_frame_loc_34AC2(
    y_velocity: i32,
    collision_type: u16,
) -> DragonFlyingReboundFrameLoc34AC2 {
    // ASM Dragon_Flying.asm loc_34AC2: add `$2000` to y_vel first. Exact zero
    // branches to return-to-anchor before hibernating or polling collision_type;
    // otherwise a non-zero collision after the hibernated frame takes the
    // shared collision route.
    let next_y_velocity = y_velocity.wrapping_add(DRAGON_TERRAIN_REBOUND_Y_ACCEL);
    if next_y_velocity == 0 {
        return DragonFlyingReboundFrameLoc34AC2::BeginReturnToAnchor { y_velocity: 0 };
    }

    if collision_type != 0 {
        DragonFlyingReboundFrameLoc34AC2::CollisionRoute {
            y_velocity: next_y_velocity,
            collision_type,
        }
    } else {
        DragonFlyingReboundFrameLoc34AC2::Continue {
            y_velocity: next_y_velocity,
        }
    }
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_right_rebound_frame_loc_34B16(
    y_velocity: i32,
    collision_type: u16,
) -> DragonFlyingReboundFrameLoc34AC2 {
    // ASM Dragon_Flying.asm loc_34B16: the right-rebound loop applies the
    // same `$2000` y_vel acceleration and exact-zero return exit as loc_34AC2.
    dragon_flying_rebound_frame_loc_34AC2(y_velocity, collision_type)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonFlyingReboundCollisionRouteLoc34B30 {
    DeathExplosion,
    RestartScript,
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_rebound_collision_route_loc_34B30(
    collision_type: u16,
) -> DragonFlyingReboundCollisionRouteLoc34B30 {
    // ASM Dragon_Flying.asm loc_34B30: only `$2C` and `$1C` branch to
    // the death/explosion path; all other non-zero collision words restart.
    match collision_type {
        0x002C | 0x001C => DragonFlyingReboundCollisionRouteLoc34B30::DeathExplosion,
        _ => DragonFlyingReboundCollisionRouteLoc34B30::RestartScript,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonFlyingDescentFireSetupLoc34BE6 {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) initial_x_velocity: i32,
    pub(crate) final_x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) offscreen_flag_cleared: bool,
    pub(crate) animation_label: &'static str,
    pub(crate) first_animation_counter: u16,
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_descent_fire_setup_loc_34BE6(
    dragon_x_pos: u32,
    dragon_y_pos: u32,
    dragon_x_velocity: i32,
    fire_y_velocity_word: i16,
    facing: Direction,
) -> DragonFlyingDescentFireSetupLoc34BE6 {
    let (x_offset_px, initial_x_velocity_word): (i16, i16) = match facing {
        Direction::Left => (-10, -1),
        Direction::Right => (10, 1),
    };
    let final_x_velocity_word = fp16_high_signed_word(dragon_x_velocity);
    let x_pos_word = ((dragon_x_pos >> 16) as u16).wrapping_add_signed(x_offset_px);
    let y_pos_word = ((dragon_y_pos >> 16) as u16).wrapping_add(10);

    // ASM Dragon_Flying.asm loc_34BE6: apply the signed mouth offset, seed
    // d1 into x_vel for animation setup, then overwrite x_vel/y_vel from the
    // parent-supplied words `$44/$46(a5)` before the first DBF loop.
    DragonFlyingDescentFireSetupLoc34BE6 {
        x_pos: u32::from(x_pos_word) << 16,
        y_pos: u32::from(y_pos_word) << 16,
        initial_x_velocity: fp16_from_signed_word(initial_x_velocity_word),
        final_x_velocity: fp16_from_signed_word(final_x_velocity_word),
        y_velocity: fp16_from_signed_word(fire_y_velocity_word),
        is_moved: true,
        has_level_collision: true,
        priority: 0,
        object_meta: 0x000C,
        offscreen_flag_cleared: true,
        animation_label: "LnkTo_unk_C75CE",
        first_animation_counter: 6,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonFlyingDescentFireFirstLoopLoc34C26 {
    CollisionCleanup {
        dbf_counter: u16,
        has_level_collision: bool,
        x_velocity: i32,
        y_velocity: i32,
        animation_label: &'static str,
    },
    ContinueFirstAnimation {
        next_d0: u16,
    },
    EnterTravelAnimation {
        next_d0: u16,
        animation_label: &'static str,
    },
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_descent_fire_first_loop_loc_34C26(
    d0_before_dbf: u16,
    collision_type: u16,
) -> DragonFlyingDescentFireFirstLoopLoc34C26 {
    // ASM Dragon_Flying.asm loc_34C26: hibernate, branch to the cleanup tail on
    // any collision before DBF, otherwise decrement d0 and keep the first sprite
    // until the DBF result becomes `$FFFF`.
    if collision_type != 0 {
        return DragonFlyingDescentFireFirstLoopLoc34C26::CollisionCleanup {
            dbf_counter: d0_before_dbf,
            has_level_collision: false,
            x_velocity: 0,
            y_velocity: 0,
            animation_label: "LnkTo_unk_C75DE",
        };
    }

    let next_d0 = d0_before_dbf.wrapping_sub(1);
    if next_d0 == 0xFFFF {
        DragonFlyingDescentFireFirstLoopLoc34C26::EnterTravelAnimation {
            next_d0,
            animation_label: "LnkTo_unk_C75D6",
        }
    } else {
        DragonFlyingDescentFireFirstLoopLoc34C26::ContinueFirstAnimation { next_d0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonFlyingDescentFireTravelLoopLoc34C3A {
    CollisionCleanup {
        has_level_collision: bool,
        x_velocity: i32,
        y_velocity: i32,
        animation_label: &'static str,
    },
    ContinueTravelAnimation,
    AnimationFinishedCleanup {
        has_level_collision: bool,
        x_velocity: i32,
        y_velocity: i32,
        animation_label: &'static str,
    },
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_descent_fire_travel_loop_loc_34C3A(
    collision_type: u16,
    animation_done_flag_19: bool,
) -> DragonFlyingDescentFireTravelLoopLoc34C3A {
    // ASM Dragon_Flying.asm loc_34C3A: after a hibernated frame, any
    // collision exits before testing byte `$19`; otherwise `$19 == 0` loops
    // on the travel sprite and `$19 != 0` enters the shared cleanup tail.
    if collision_type != 0 {
        return DragonFlyingDescentFireTravelLoopLoc34C3A::CollisionCleanup {
            has_level_collision: false,
            x_velocity: 0,
            y_velocity: 0,
            animation_label: "LnkTo_unk_C75DE",
        };
    }

    if !animation_done_flag_19 {
        return DragonFlyingDescentFireTravelLoopLoc34C3A::ContinueTravelAnimation;
    }

    DragonFlyingDescentFireTravelLoopLoc34C3A::AnimationFinishedCleanup {
        has_level_collision: false,
        x_velocity: 0,
        y_velocity: 0,
        animation_label: "LnkTo_unk_C75DE",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonFlyingDescentFireCleanupLoc34C4A {
    pub(crate) has_level_collision: bool,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) first_animation_label: &'static str,
    pub(crate) first_animation_hibernate_frames: u16,
    pub(crate) second_animation_label: &'static str,
    pub(crate) second_animation_hibernate_frames: u16,
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_descent_fire_cleanup_loc_34C4A()
-> DragonFlyingDescentFireCleanupLoc34C4A {
    // ASM Dragon_Flying.asm loc_34C4A: clear level collision and both velocity
    // longs, hibernate three frames on `LnkTo_unk_C75DE`, hibernate three more
    // on `LnkTo_unk_C75E6`, then tail-jump to Delete_CurrentObject.
    DragonFlyingDescentFireCleanupLoc34C4A {
        has_level_collision: false,
        x_velocity: 0,
        y_velocity: 0,
        first_animation_label: "LnkTo_unk_C75DE",
        first_animation_hibernate_frames: 3,
        second_animation_label: "LnkTo_unk_C75E6",
        second_animation_hibernate_frames: 3,
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonFlyingRespawnMarkerLoc34998 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

pub(crate) fn dragon_flying_respawn_marker_loc_34998(
    object_status_slot: u8,
) -> DragonFlyingRespawnMarkerLoc34998 {
    // ASM Dragon_Flying.asm loc_34998: the positive `$42(a5)` byte is doubled
    // as a word offset into EnemyStatus_Table, then `$2168` is written as the
    // respawn marker before falling through to loc_349A6.
    DragonFlyingRespawnMarkerLoc34998 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonFlyingDeleteTailLoc349A6 {
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_delete_tail_loc_349A6() -> DragonFlyingDeleteTailLoc349A6 {
    // ASM Dragon_Flying.asm loc_349A6: all death-status routes converge on the
    // `j_Delete_CurrentObject` tail after the explosion animation finishes.
    // The following disassembly fragment, labelled loc_349C0, is orphaned
    // after this unconditional jump; return-to-anchor uses loc_34B4C instead.
    DragonFlyingDeleteTailLoc349A6 {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonFlyingDeathStatusRouteLoc3492C {
    WriteRespawnMarker(DragonFlyingRespawnMarkerLoc34998),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonFlyingDeathSequenceLoc3492C {
    pub(crate) animation_label: &'static str,
    pub(crate) animation_hibernate_frames: u16,
    pub(crate) is_moved: bool,
    pub(crate) vram_tile: u16,
    pub(crate) pre_child_hibernate_frames: u16,
    pub(crate) child_x_word: u16,
    pub(crate) child_y_word: u16,
    pub(crate) child_facing: Direction,
    pub(crate) child_state_word_4a: u16,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) status_route: DragonFlyingDeathStatusRouteLoc3492C,
    pub(crate) delete_tail: DragonFlyingDeleteTailLoc349A6,
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_death_sequence_loc_3492C(
    parent_x_pos: u32,
    parent_y_pos: u32,
    parent_facing: Direction,
    parent_word_46: u16,
    object_status_slot: u8,
) -> DragonFlyingDeathSequenceLoc3492C {
    // ASM Dragon_Flying.asm loc_3492C: play `stru_345E4`, clear `is_moved`
    // and `vram_tile`, wait two frames, then allocate the loc_3CD16 walking
    // dragon child from integer X/Y words before waiting for the animation.
    let status_route = if object_status_slot & 0x80 == 0 {
        DragonFlyingDeathStatusRouteLoc3492C::WriteRespawnMarker(
            dragon_flying_respawn_marker_loc_34998(object_status_slot),
        )
    } else if object_status_slot & 0x40 != 0 {
        DragonFlyingDeathStatusRouteLoc3492C::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        DragonFlyingDeathStatusRouteLoc3492C::DeleteOnly
    };

    DragonFlyingDeathSequenceLoc3492C {
        animation_label: "stru_345E4",
        animation_hibernate_frames: DRAGON_FLYING_DEATH_ANIMATION_FRAMES,
        is_moved: false,
        vram_tile: 0,
        pre_child_hibernate_frames: DRAGON_FLYING_DEATH_PRE_CHILD_HIBERNATE_FRAMES,
        child_x_word: (parent_x_pos >> 16) as u16,
        child_y_word: ((parent_y_pos >> 16) as u16).wrapping_add(0x10),
        child_facing: parent_facing,
        child_state_word_4a: parent_word_46,
        hibernate_until_animation_finished: true,
        status_route,
        delete_tail: dragon_flying_delete_tail_loc_349A6(),
    }
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_death_animation_loc_3492C(
    parent: &ActiveEnemy,
) -> PendingDragonDeathAnimation {
    let sequence = dragon_flying_death_sequence_loc_3492C(
        parent.x_pos,
        parent.y_pos,
        parent.facing,
        parent.wait_timer,
        parent.layout_index as u8,
    );
    PendingDragonDeathAnimation {
        remaining_frames: DRAGON_FLYING_DEATH_ANIMATION_FRAMES,
        x_pos: parent.x_pos,
        y_pos: parent.y_pos,
        facing: parent.facing,
        palette_variant: parent.palette_variant,
        layout_index: parent.layout_index,
        status_route: sequence.status_route,
    }
}

#[allow(non_snake_case)]
pub(crate) fn dragon_flying_death_child_loc_3492C(parent: &ActiveEnemy) -> ActiveEnemy {
    let sequence = dragon_flying_death_sequence_loc_3492C(
        parent.x_pos,
        parent.y_pos,
        parent.facing,
        parent.wait_timer,
        0x80,
    );
    let child_init = dragon_walking_child_init_loc_3cd16(
        u32::from(sequence.child_x_word) << 16,
        u32::from(sequence.child_y_word) << 16,
        sequence.child_facing,
        sequence.child_state_word_4a,
    );
    debug_assert_eq!(
        child_init.inherited_state_word,
        sequence.child_state_word_4a
    );
    let family = EnemyFamily::DragonWalking;
    ActiveEnemy {
        segment_index: parent.segment_index,
        layout_index: parent.layout_index,
        enemy_level: 0,
        family,
        x_pos: child_init.x_pos,
        y_pos: child_init.y_pos,
        x_velocity: 0,
        y_velocity: 0,
        target_x_velocity: 0,
        target_y_velocity: 0,
        x_acceleration: 0,
        y_acceleration: 0,
        facing: child_init.facing,
        on_ground: true,
        hit_points: spawn_hit_points(family, 0),
        death_animation_active: false,
        grounded_frames: 1,
        tank_mouth_animation_phase: 0,
        damage_flash_frames: 0,
        palette_variant: parent.palette_variant,
        wait_timer: DRAGON_WALKING_BREATH_INITIAL,
        dragon_phase: DragonFlyingState::None,
        dragon_phase_frames: 0,
        dragon_script_variant: if child_init.parent_spawned_child {
            DRAGON_WALKING_PARENT_SPAWNED_CHILD_FLAG
        } else {
            0
        },
        dragon_script_cmd: 0,
        dragon_anchor_y: 0,
        dragon_descent_index: 0,
        dragon_descent_frames: 0,
        ninja_state: NinjaState::Stealth,
        hand_state: HandState::Walking,
        lion_state: spawn_lion_state_loc_3e030(family),
        robot_state: RobotState::Walking,
        orca_state: OrcaState::Walking,
        wall_pinned: false,
        drips_attached: false,
        drips_surface_table_index: 0,
        drips_pending_splash_x_pos: 0,
        drips_pending_splash_y_pos: 0,
        tar_monster_state: TarMonsterState::default(),
        spinning_twins_phase: SpinningTwinsPhase::default(),
        spinning_twins_orbit_index: 0,
        ufo_beam_segments: 0,
        ufo_beam_interval: 0,
        ufo_on_screen_latch: false,
        boss_head_index: BOSS_HEAD_NONE,
        boss_eye_orbit_workers: BossEyeOrbitWorkers::default(),
        boss_movement_delay_counter: 0,
        boss_movement_script_index: 0,
        heady_metal_palette_phase_counter: 0,
        heady_metal_palette_word_5e: 0x0200,
        scorpion_tracking_lock: false,
        scorpion_sting_frames: 0,
        scorpion_cached_kid_x_word: 0,
        flight_double_acceleration: false,
        flight_double_terminal_speed: false,
        walk_speed_override: 0,
        crab_frozen: false,
        crab_berzerker_knockback: false,
        shared_walker_state: spawn_shared_walker_state(family),
        shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(family),
        shared_walker_turnaround_counter: 0,
        shared_walker_right_turn_extra_step_consumed:
            spawn_shared_walker_right_turn_extra_step_consumed(family, child_init.facing),
        shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(family),
        shared_drive_sound_previous_render_hidden: false,
        driller_children: None,
        shared_walker_debug_branch: None,
        shared_walker_dx_to_kid_px: None,
        shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        shared_walker_grounded_scan_trace: None,
    }
}

/// Script command for Dragon Flying flight scripts.
#[derive(Debug, Clone, Copy)]
pub(crate) enum DragonFlyingCmd {
    /// Hover in place for N frames (ASM cmd 0).
    Hover(u16),
    /// Accelerating drift for N frames (ASM cmd 1).
    Drift(u16),
    /// Drift + fire-breath cone check for N frames (ASM cmd 5).
    DriftWithFire(u16),
    /// Face player and set initial drift speed (ASM cmd 3). Immediate.
    FacePlayer,
    /// Descent attack sub-script (ASM cmd 4).
    Descent,
    /// Return to anchor Y position (ASM cmd 6).
    ReturnToAnchor,
}

/// Variant 0 script (unk_34710): hover → descent → drift → hover → drift → return.
const DRAGON_SCRIPT_0: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::Hover(0x70), // 112 frames
    DragonFlyingCmd::Descent,
    DragonFlyingCmd::Drift(0x50), // 80 frames
    DragonFlyingCmd::Hover(0x32), // 50 frames
    DragonFlyingCmd::Drift(0x78), // 120 frames
    DragonFlyingCmd::ReturnToAnchor,
];

/// Variant 1 script (unk_34706): aggressive fire-heavy patrol.
const DRAGON_SCRIPT_1: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::DriftWithFire(0x28), // 40 frames
    DragonFlyingCmd::DriftWithFire(0x1E), // 30 frames
    DragonFlyingCmd::DriftWithFire(0x32), // 50 frames
    DragonFlyingCmd::ReturnToAnchor,
];

/// Variant 2 script (unk_346FA): long fire patrols with retargeting.
const DRAGON_SCRIPT_2: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::DriftWithFire(0x70), // 112 frames
    DragonFlyingCmd::DriftWithFire(0x70), // 112 frames
    DragonFlyingCmd::FacePlayer,
    DragonFlyingCmd::DriftWithFire(0x50), // 80 frames
    DragonFlyingCmd::ReturnToAnchor,
];

pub(crate) fn dragon_script(variant: u8) -> &'static [DragonFlyingCmd] {
    match variant {
        0 => DRAGON_SCRIPT_0,
        1 => DRAGON_SCRIPT_1,
        _ => DRAGON_SCRIPT_2,
    }
}

/// Descent sub-script entry: fixed velocity for N frames (unk_34554).
#[derive(Debug, Clone, Copy)]
enum DragonDiveAction {
    /// Move at (x_vel, y_vel) pixels/frame for N frames. x_vel is for facing-left;
    /// negated when facing right (ASM loc_348CE).
    Move { x_vel: i8, y_vel: i8, frames: u8 },
    /// Spawn fire-breath projectile (ASM $E19F sentinel).
    SpawnFire,
    /// Flip facing direction (ASM $DD48 sentinel).
    FlipDirection,
    /// Restart main flight script (ASM $D8F1 sentinel).
    RestartScript,
}

/// Descent attack animation script (unk_34554, kid.asm:37771).
/// U-shaped dive: descend → level out → ascend → flip → continue ascending → restart.
const DRAGON_DIVE_SCRIPT: &[DragonDiveAction] = &[
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 1,
        frames: 12,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 2,
        frames: 9,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 3,
        frames: 6,
    },
    DragonDiveAction::SpawnFire,
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 4,
        frames: 6,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 3,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 2,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 1,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 0,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -1,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -2,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -3,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -4,
        frames: 6,
    },
    DragonDiveAction::FlipDirection,
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -3,
        frames: 6,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -2,
        frames: 9,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -1,
        frames: 12,
    },
    DragonDiveAction::RestartScript,
];

const DRAGON_FLYING_MAIN_SPRITE_PHASES_STRU_34600: [u8; 20] =
    [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3];
const DRAGON_FLYING_MAIN_ANIMATION_TICKS: u16 = 7 * 20;

// ASM unk_34554 stores an off_345F6 mapping offset in each movement entry's
// second word. Atlas slots 0..=4 are shared with the main animation; 5..=8 are
// dive-only mappings. Sentinel entries do not install a mapping.
const DRAGON_FLYING_DIVE_SPRITE_PHASES_UNK_34554: [Option<u8>; 18] = [
    Some(6),
    Some(8),
    Some(8),
    None,
    Some(8),
    Some(6),
    Some(6),
    Some(6),
    Some(2),
    Some(7),
    Some(7),
    Some(7),
    Some(8),
    None,
    Some(8),
    Some(8),
    Some(7),
    None,
];

pub(crate) fn dragon_flying_sprite_animation_phase(enemy: &ActiveEnemy) -> Option<u8> {
    debug_assert!(matches!(enemy.family, EnemyFamily::Dragon));
    if enemy.death_animation_active {
        return None;
    }
    match enemy.dragon_phase {
        DragonFlyingState::None => None,
        DragonFlyingState::Descending => DRAGON_FLYING_DIVE_SPRITE_PHASES_UNK_34554
            .get(usize::from(enemy.dragon_descent_index))
            .copied()
            .flatten(),
        DragonFlyingState::TerrainSlideRight
        | DragonFlyingState::TerrainSlideLeft
        | DragonFlyingState::TerrainSlideUp
        | DragonFlyingState::TerrainSlideDown
        | DragonFlyingState::TerrainReboundLeft
        | DragonFlyingState::TerrainReboundRight => {
            // ASM loc_349DE installs `stru_34652`: four four-tick mappings.
            Some((enemy.wait_timer / 4) as u8)
        }
        DragonFlyingState::Hovering
        | DragonFlyingState::Drifting
        | DragonFlyingState::DriftingWithFire
        | DragonFlyingState::ReturningToAnchor => {
            // ASM loc_346C4 installs `stru_34600`. `wait_timer` is the
            // established object-local seven-tick phase counter used by
            // contact parity.
            let script_phase = usize::from(enemy.wait_timer / 7)
                % DRAGON_FLYING_MAIN_SPRITE_PHASES_STRU_34600.len();
            Some(DRAGON_FLYING_MAIN_SPRITE_PHASES_STRU_34600[script_phase])
        }
    }
}

/// Dragon Flying script interpreter (Dragon_Flying.asm).
/// Reads commands from one of 3 variant scripts, executing each per-frame.
/// Commands: Hover (idle), Drift (accelerate), DriftWithFire (drift + cone check),
/// FacePlayer (retarget), Descent (dive sub-script), ReturnToAnchor (rise to spawn Y).
pub(crate) fn step_dragon(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    if !matches!(
        enemy.dragon_phase,
        DragonFlyingState::None | DragonFlyingState::Descending
    ) && !enemy.death_animation_active
    {
        // ASM Dragon_Flying.asm loc_346C4 initialises `stru_34600`; each
        // hibernated script frame lets the GfxObject animation advance. Native
        // stores the current GfxObject animation tick in `wait_timer` for
        // contact row selection through GfxObjects_CollisionKid loc_1BCE.
        let animation_ticks = if matches!(
            enemy.dragon_phase,
            DragonFlyingState::TerrainSlideRight
                | DragonFlyingState::TerrainSlideLeft
                | DragonFlyingState::TerrainSlideUp
                | DragonFlyingState::TerrainSlideDown
                | DragonFlyingState::TerrainReboundLeft
                | DragonFlyingState::TerrainReboundRight
        ) {
            4 * 4
        } else {
            DRAGON_FLYING_MAIN_ANIMATION_TICKS
        };
        enemy.wait_timer = (enemy.wait_timer + 1) % animation_ticks;
    }
    match enemy.dragon_phase {
        DragonFlyingState::None => {}
        DragonFlyingState::Hovering => {
            // Hover: zero velocity, count down frames. ASM loc_34804 clears
            // x_vel; Dragon_Flying.asm loc_3480A then hibernates one frame
            // before checking collision_type and continuing the hover loop.
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            let hover_tail =
                dragon_hover_dbf_tail_loc_3482e(enemy.dragon_phase_frames.saturating_sub(1));
            enemy.dragon_phase_frames = if hover_tail.loops_to_loc_3480a {
                hover_tail.next_d0.wrapping_add(1)
            } else {
                0
            };
            if hover_tail.advances_script {
                dragon_advance_script(enemy, player_x_pos, projectiles);
            }
            dragon_apply_velocity(enemy);
        }
        DragonFlyingState::Drifting | DragonFlyingState::DriftingWithFire => {
            // Drift: accelerate x_velocity toward cap, count down frames.
            // ASM loc_3478C: add/sub $200 per frame, cap at ±$18000.
            if enemy.x_velocity > 0 {
                enemy.x_velocity = dragon_drift_positive_velocity_loc_3478c(enemy.x_velocity);
            } else if enemy.x_velocity < 0 {
                enemy.x_velocity = dragon_drift_negative_velocity_loc_347b2(enemy.x_velocity);
            }
            let d1_fire_gate = if enemy.dragon_phase == DragonFlyingState::DriftingWithFire {
                1
            } else {
                0
            };
            let drift_tail = dragon_drift_store_loc_347c8(enemy.x_velocity, d1_fire_gate);
            enemy.x_velocity = drift_tail.x_velocity;
            // Fire-breath cone check during DriftWithFire (ASM loc_34836).
            if drift_tail.checks_fire_breath_cone
                && dragon_fire_breath_cone(enemy, player_x_pos, player_y_pos)
            {
                // Cone hit: enter descent attack.
                dragon_enter_descent(enemy);
                dragon_apply_velocity(enemy);
                return;
            }
            let drift_tail =
                dragon_drift_dbf_tail_loc_347fc(enemy.dragon_phase_frames.saturating_sub(1));
            enemy.dragon_phase_frames = if drift_tail.loops_to_loc_3478c {
                drift_tail.next_d0.wrapping_add(1)
            } else {
                0
            };
            dragon_apply_velocity(enemy);
            // Wall collision during Drift: reverse direction on terrain hit.
            // ASM Dragon_Flying.asm loc_347D6 checks collision before the
            // loc_347FC DBF tail advances to the next command, so the final
            // drift frame still moves and can flip facing before hover/descent.
            if let Some(grid) = block_grid {
                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;
                let wall = terrain::probe_wall(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                    moving_right,
                );
                if wall.hit {
                    let (facing, x_velocity) =
                        dragon_reverse_x_motion(enemy.facing, enemy.x_velocity);
                    enemy.facing = facing;
                    enemy.x_velocity = x_velocity;
                }
            }
            if drift_tail.advances_script {
                // ASM Dragon_Flying.asm loc_347FC falls through to loc_346DA
                // after the hibernated drift frame has already run.
                dragon_advance_script(enemy, player_x_pos, projectiles);
            }
        }
        DragonFlyingState::Descending => {
            step_dragon_descent(enemy, player_x_pos, projectiles);
            dragon_apply_velocity(enemy);
            if let Some(grid) = block_grid
                && let Some(collision_type) = dragon_flying_terrain_collision_type(enemy, grid)
            {
                match dragon_descent_collision_outcome_loc_348fc(collision_type) {
                    DragonDescentCollisionOutcomeLoc348FC::ContinueLoop => {}
                    DragonDescentCollisionOutcomeLoc348FC::Explosion => {
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                    }
                    DragonDescentCollisionOutcomeLoc348FC::TerrainResponse(collision_type) => {
                        dragon_enter_terrain_response(enemy, collision_type);
                    }
                }
            }
        }
        DragonFlyingState::TerrainSlideRight
        | DragonFlyingState::TerrainSlideLeft
        | DragonFlyingState::TerrainSlideUp
        | DragonFlyingState::TerrainSlideDown
        | DragonFlyingState::TerrainReboundLeft
        | DragonFlyingState::TerrainReboundRight => {
            step_dragon_terrain_response(enemy, block_grid);
        }
        DragonFlyingState::ReturningToAnchor => {
            // Return to anchor: decelerate upward. ASM loc_34B4C.
            enemy.x_velocity = 0;
            let anchor_y = enemy.dragon_anchor_y as i64;
            let current_y = enemy.y_pos as i64;
            if current_y <= anchor_y {
                // Reached anchor: restart script. ASM bra loc_346C4.
                dragon_restart_script(enemy);
            } else {
                enemy.y_velocity = dragon_return_y_velocity_loc_34b4c(enemy.y_velocity);
            }
            dragon_apply_velocity(enemy);
        }
    }
}

/// Apply velocity to position (FP16.16 arithmetic).
fn dragon_apply_velocity(enemy: &mut ActiveEnemy) {
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
}

fn dragon_flying_terrain_collision_type(
    enemy: &ActiveEnemy,
    grid: &terrain::BlockGrid,
) -> Option<u16> {
    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;

    if enemy.x_velocity > 0 {
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            true,
        );
        if wall.hit {
            return Some(DRAGON_FLYING_COLID_RIGHTWALL);
        }
    } else if enemy.x_velocity < 0 {
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            false,
        );
        if wall.hit {
            return Some(DRAGON_FLYING_COLID_LEFTWALL);
        }
    }

    if enemy.y_velocity > 0 {
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground {
            return Some(match floor.cell.collision {
                terrain::CollisionKind::SlopeUp => DRAGON_FLYING_COLID_SLOPEUP,
                terrain::CollisionKind::SlopeDown => DRAGON_FLYING_COLID_SLOPEDOWN,
                _ => DRAGON_FLYING_COLID_FLOOR,
            });
        }
    }

    if enemy.y_velocity < 0 {
        let ceiling =
            terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceiling.hit {
            return Some(DRAGON_FLYING_COLID_CEILING);
        }
    }

    None
}

fn dragon_enter_terrain_response(enemy: &mut ActiveEnemy, collision_type: u16) {
    match dragon_terrain_dispatch_loc_349DE(collision_type) {
        DragonTerrainDispatchLoc349DE::DeathExplosion { .. } => {
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
        }
        DragonTerrainDispatchLoc349DE::RestartDescentScript { .. } => {
            dragon_restart_script(enemy);
        }
        DragonTerrainDispatchLoc349DE::ResponseTable { byte_offset, .. } => {
            // ASM loc_349DE installs `stru_34652` before dispatching through
            // off_349C6, resetting this GfxObject's private animation timer.
            enemy.wait_timer = 0;
            match byte_offset {
                // ASM Dragon_Flying.asm off_349C6: collision `$04` dispatches
                // to loc_34A24 and `$08` to loc_34A00.
                0x0000 => {
                    let start = dragon_left_slide_start_loc_34a24(enemy.x_velocity);
                    enemy.x_velocity = start.x_velocity;
                    enemy.y_velocity = start.y_velocity;
                    enemy.dragon_phase = DragonFlyingState::TerrainSlideLeft;
                }
                0x0004 => {
                    let start = dragon_right_slide_start_loc_34a00(enemy.x_velocity);
                    enemy.x_velocity = start.x_velocity;
                    enemy.y_velocity = start.y_velocity;
                    enemy.dragon_phase = DragonFlyingState::TerrainSlideRight;
                }
                0x0008 => {
                    let start = dragon_up_slide_start_loc_34a48(enemy.y_velocity);
                    enemy.y_velocity = start.y_velocity;
                    enemy.dragon_phase = DragonFlyingState::TerrainSlideUp;
                }
                0x000C => {
                    let start = dragon_down_slide_start_loc_34a68(enemy.y_velocity);
                    enemy.y_velocity = start.y_velocity;
                    enemy.dragon_phase = DragonFlyingState::TerrainSlideDown;
                }
                0x0010 => {
                    let start = dragon_flying_rebound_start_loc_34A88(
                        enemy.x_pos,
                        enemy.y_pos,
                        enemy.x_velocity,
                        enemy.y_velocity,
                    );
                    enemy.x_pos = start.snap_x_pos;
                    enemy.y_pos = start.snap_y_pos;
                    enemy.x_velocity = start.x_velocity_during_snap;
                    enemy.y_velocity = start.y_velocity_during_snap;
                    enemy.dragon_phase = DragonFlyingState::TerrainReboundLeft;
                    enemy.dragon_phase_frames = DRAGON_TERRAIN_REBOUND_START_WAIT_FRAMES;
                }
                0x0014 => {
                    let start = dragon_flying_right_rebound_start_loc_34ADC(
                        enemy.x_pos,
                        enemy.y_pos,
                        enemy.x_velocity,
                        enemy.y_velocity,
                    );
                    enemy.x_pos = start.snap_x_pos;
                    enemy.y_pos = start.snap_y_pos;
                    enemy.x_velocity = start.x_velocity_during_snap;
                    enemy.y_velocity = start.y_velocity_during_snap;
                    enemy.dragon_phase = DragonFlyingState::TerrainReboundRight;
                    enemy.dragon_phase_frames = DRAGON_TERRAIN_REBOUND_START_WAIT_FRAMES;
                }
                _ => dragon_restart_script(enemy),
            }
        }
    }
}

fn step_dragon_terrain_response(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    if matches!(
        enemy.dragon_phase,
        DragonFlyingState::TerrainReboundLeft | DragonFlyingState::TerrainReboundRight
    ) && enemy.dragon_phase_frames > 0
    {
        enemy.dragon_phase_frames -= 1;
        if enemy.dragon_phase_frames == 0 {
            if enemy.dragon_phase == DragonFlyingState::TerrainReboundRight {
                let start = dragon_flying_right_rebound_start_loc_34ADC(
                    enemy.x_pos,
                    enemy.y_pos,
                    enemy.x_velocity,
                    enemy.y_velocity,
                );
                enemy.x_velocity = start.rebound_x_velocity;
                enemy.y_velocity = start.rebound_y_velocity;
            } else {
                let start = dragon_flying_rebound_start_loc_34A88(
                    enemy.x_pos,
                    enemy.y_pos,
                    enemy.x_velocity,
                    enemy.y_velocity,
                );
                enemy.x_velocity = start.rebound_x_velocity;
                enemy.y_velocity = start.rebound_y_velocity;
            }
        }
        return;
    }

    dragon_apply_velocity(enemy);

    if let Some(grid) = block_grid
        && let Some(collision_type) = dragon_flying_terrain_collision_type(enemy, grid)
    {
        match dragon_flying_rebound_collision_route_loc_34B30(collision_type) {
            DragonFlyingReboundCollisionRouteLoc34B30::DeathExplosion => {
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
            }
            DragonFlyingReboundCollisionRouteLoc34B30::RestartScript => {
                dragon_restart_script(enemy);
            }
        }
        return;
    }

    match enemy.dragon_phase {
        DragonFlyingState::TerrainSlideRight => {
            match dragon_right_slide_frame_loc_34a0a(enemy.x_velocity, 0) {
                DragonRightSlideFrameLoc34A0A::BeginReturnToAnchor { x_velocity } => {
                    enemy.x_velocity = x_velocity;
                    enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                }
                DragonRightSlideFrameLoc34A0A::Continue { x_velocity } => {
                    enemy.x_velocity = x_velocity;
                }
                DragonRightSlideFrameLoc34A0A::CollisionRoute { .. } => {}
            }
        }
        DragonFlyingState::TerrainSlideLeft => {
            match dragon_left_slide_frame_loc_34a2e(enemy.x_velocity, 0) {
                DragonLeftSlideFrameLoc34A2E::BeginReturnToAnchor { x_velocity } => {
                    enemy.x_velocity = x_velocity;
                    enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                }
                DragonLeftSlideFrameLoc34A2E::Continue { x_velocity } => {
                    enemy.x_velocity = x_velocity;
                }
                DragonLeftSlideFrameLoc34A2E::CollisionRoute { .. } => {}
            }
        }
        DragonFlyingState::TerrainSlideUp => {
            match dragon_up_slide_frame_loc_34a4e(enemy.y_velocity, 0) {
                DragonUpSlideFrameLoc34A4E::BeginReturnToAnchor { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                    enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                }
                DragonUpSlideFrameLoc34A4E::Continue { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                }
                DragonUpSlideFrameLoc34A4E::CollisionRoute { .. } => {}
            }
        }
        DragonFlyingState::TerrainSlideDown => {
            match dragon_down_slide_frame_loc_34A6E(enemy.y_velocity, 0) {
                DragonDownSlideFrameLoc34A6E::BeginReturnToAnchor { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                    enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                }
                DragonDownSlideFrameLoc34A6E::Continue { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                }
                DragonDownSlideFrameLoc34A6E::CollisionRoute { .. } => {}
            }
        }
        DragonFlyingState::TerrainReboundLeft => {
            match dragon_flying_rebound_frame_loc_34AC2(enemy.y_velocity, 0) {
                DragonFlyingReboundFrameLoc34AC2::BeginReturnToAnchor { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                    enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                }
                DragonFlyingReboundFrameLoc34AC2::Continue { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                }
                DragonFlyingReboundFrameLoc34AC2::CollisionRoute { .. } => {}
            }
        }
        DragonFlyingState::TerrainReboundRight => {
            match dragon_flying_right_rebound_frame_loc_34B16(enemy.y_velocity, 0) {
                DragonFlyingReboundFrameLoc34AC2::BeginReturnToAnchor { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                    enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                }
                DragonFlyingReboundFrameLoc34AC2::Continue { y_velocity } => {
                    enemy.y_velocity = y_velocity;
                }
                DragonFlyingReboundFrameLoc34AC2::CollisionRoute { .. } => {}
            }
        }
        _ => {}
    }
}

/// Advance to the next command in the dragon's flight script.
/// Immediate commands (FacePlayer, Reverse) execute inline and continue advancing.
fn dragon_advance_script(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    _projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let script = dragon_script(enemy.dragon_script_variant);
    let previous_phase = enemy.dragon_phase;
    loop {
        // ASM Dragon_Flying.asm loc_346DA: fetch one script word, restart on
        // $FFFF, then use the low byte as a byte offset into off_3471E.
        enemy.dragon_script_cmd += 1;
        if enemy.dragon_script_cmd as usize >= script.len() {
            // Script end: restart. ASM $FFFF sentinel → loc_346C0.
            dragon_restart_script(enemy);
            return;
        }
        match script[enemy.dragon_script_cmd as usize] {
            DragonFlyingCmd::Hover(frames) => {
                enemy.dragon_phase = DragonFlyingState::Hovering;
                // ASM Dragon_Flying.asm loc_347FC falls through to loc_346DA,
                // then loc_34804/loc_3480A immediately enters the hover
                // hibernation loop after a drift command. Native resumes at
                // frame boundaries, so preserve those suspended hover frames
                // when a drift tail enters a hover command.
                enemy.dragon_phase_frames = if matches!(
                    previous_phase,
                    DragonFlyingState::Drifting | DragonFlyingState::DriftingWithFire
                ) {
                    frames.saturating_add(2)
                } else {
                    frames
                };
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                return;
            }
            DragonFlyingCmd::Drift(frames) => {
                dragon_begin_drift(enemy, frames, false);
                return;
            }
            DragonFlyingCmd::DriftWithFire(frames) => {
                dragon_begin_drift(enemy, frames, true);
                return;
            }
            DragonFlyingCmd::FacePlayer => {
                // Immediate: face player and set initial drift speed. ASM loc_3473A.
                if let Some(px) = player_x_pos {
                    let (facing, x_velocity) = dragon_face_player_motion(enemy.x_pos, px);
                    enemy.facing = facing;
                    enemy.x_velocity = x_velocity;
                }
                // Continue to next command immediately (no frame consumed).
            }
            DragonFlyingCmd::Descent => {
                dragon_enter_descent(enemy);
                return;
            }
            DragonFlyingCmd::ReturnToAnchor => {
                enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                return;
            }
        }
    }
}

pub(crate) fn dragon_face_player_motion(dragon_x_pos: u32, player_x_pos: u32) -> (Direction, i32) {
    let dragon_x_px = dragon_x_pos >> 16;
    let player_x_px = player_x_pos >> 16;
    if dragon_x_px < player_x_px {
        // ASM Dragon_Flying.asm loc_34758: x_direction=$FF and x_vel=$8000
        // only when the dragon's integer X word is strictly left of Kid.
        (Direction::Right, DRAGON_FACE_PLAYER_SPEED)
    } else {
        (Direction::Left, -DRAGON_FACE_PLAYER_SPEED)
    }
}

pub(crate) fn dragon_reverse_x_motion(facing: Direction, x_velocity: i32) -> (Direction, i32) {
    let facing = match facing {
        Direction::Right => Direction::Left,
        Direction::Left => Direction::Right,
    };
    // ASM Dragon_Flying.asm loc_34768: not.b x_direction(a3); neg.l x_vel(a3).
    (facing, x_velocity.wrapping_neg())
}

/// Enter the descent attack sub-script. ASM loc_3487E.
fn dragon_enter_descent(enemy: &mut ActiveEnemy) {
    enemy.dragon_phase = DragonFlyingState::Descending;
    enemy.dragon_descent_index = 0;
    enemy.dragon_descent_frames = 0;
    // First entry will be processed immediately in step_dragon_descent.
}

/// Step through the descent attack sub-script (unk_34554).
/// Each entry sets fixed x/y velocities for N frames. Special entries
/// spawn fire, flip direction, or restart the main script.
fn step_dragon_descent(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let mut immediate_steps = 0usize;
    loop {
        debug_assert!(
            immediate_steps <= DRAGON_DIVE_SCRIPT.len(),
            "ASM Dragon_Flying.asm loc_3488A immediate descent sentinels must advance through unk_34554"
        );
        immediate_steps = immediate_steps.checked_add(1).expect(
            "ASM Dragon_Flying.asm loc_3488A descent sentinel loop bound stays within script length",
        );
        // ASM Dragon_Flying.asm loc_3488A reads the next `unk_34554` word:
        // restart, flip, and fire sentinels are immediate; movement entries
        // consume frames after velocity setup.
        if enemy.dragon_descent_index as usize >= DRAGON_DIVE_SCRIPT.len() {
            // Shouldn't happen — script ends with RestartScript.
            dragon_restart_script(enemy);
            return;
        }
        match DRAGON_DIVE_SCRIPT[enemy.dragon_descent_index as usize] {
            DragonDiveAction::Move {
                x_vel,
                y_vel,
                frames,
            } => {
                if enemy.dragon_descent_frames == 0 {
                    // Start this entry: set velocities (pixel/frame → FP16.16).
                    let setup =
                        dragon_descent_move_setup_loc_348e4(enemy.facing, x_vel, y_vel, frames);
                    enemy.x_velocity = setup.x_velocity;
                    enemy.y_velocity = setup.y_velocity;
                    // Native keeps a hibernated frame count here; ASM loc_348E4
                    // seeds the equivalent DBF counter with `frame_count - 1`.
                    enemy.dragon_descent_frames = frames;
                }
                // ASM Dragon_Flying.asm loc_34924 advances the descent script
                // only after the DBF counter underflows to `$FFFF`; native uses
                // a direct remaining-frame count for the same hibernated loop.
                let descent_tail =
                    dragon_descent_dbf_tail_loc_34924(u16::from(enemy.dragon_descent_frames - 1));
                enemy.dragon_descent_frames = if descent_tail.loops_to_loc_348fc {
                    u8::try_from(descent_tail.next_d2.wrapping_add(1))
                        .expect("ASM Dragon_Flying.asm loc_34924 descent counter stays byte-sized")
                } else {
                    0
                };
                if descent_tail.advances_descent_script {
                    enemy.dragon_descent_index += 1;
                    // ASM loc_34924 branches straight to loc_3488A after the
                    // DBF underflow. Process sentinels and install the next
                    // movement mapping before this frame's GfxObject move.
                    continue;
                }
                return; // Consume one frame.
            }
            DragonDiveAction::SpawnFire => {
                // Spawn fire-breath projectile during descent.
                // ASM loc_348A0 prepares the parent words; loc_34BAC/loc_34BE6
                // apply x_pos += $FFF6 (-10px, negated if facing right),
                // y_pos += $A (+10px), x_vel from the dragon's signed high
                // word, y_vel = 3<<16.
                // No frame-based lifetime — dies off-screen ($19) or on terrain.
                if let Some(projs) = projectiles {
                    let setup = dragon_flying_descent_fire_setup_loc_34BE6(
                        enemy.x_pos,
                        enemy.y_pos,
                        enemy.x_velocity,
                        3,
                        enemy.facing,
                    );
                    projs.push(EnemyProjectile {
                        x_pos: setup.x_pos,
                        y_pos: setup.y_pos,
                        x_velocity: setup.final_x_velocity,
                        y_velocity: setup.y_velocity,
                        // ASM Dragon_Flying.asm loc_34C26/loc_34C3A has no
                        // generic DBF lifetime; the child advances through its
                        // own startup, travel, and cleanup coroutine.
                        terrain_kill: true,
                        kind: EnemyProjectileKind::DragonFlyingDescentFire,
                        dragon_flying_descent_fire_state:
                            DragonFlyingDescentFireState::FirstAnimationLoc34C26,
                        dragon_flying_descent_fire_counter: setup.first_animation_counter,
                        ..Default::default()
                    });
                }
                enemy.dragon_descent_index += 1;
                // Immediate — continue to next entry without consuming a frame.
            }
            DragonDiveAction::FlipDirection => {
                // Flip facing direction. ASM $DD48.
                enemy.facing = match enemy.facing {
                    Direction::Left => Direction::Right,
                    Direction::Right => Direction::Left,
                };
                enemy.dragon_descent_index += 1;
                // Immediate — continue.
            }
            DragonDiveAction::RestartScript => {
                // ASM Dragon_Flying.asm loc_3488A `$D8F1` branches to
                // loc_346C4, which reinitialises animation and falls through
                // to loc_346DA with the current main-script `a0`. It does not
                // reload `$48(a5)`, so native must continue after the descent
                // command rather than restart at script index 0.
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                dragon_advance_script(enemy, player_x_pos, projectiles);
                return;
            }
        }
    }
}

/// Fire-breath cone check (ASM loc_34836).
/// Returns true if the player is within the dragon's breath cone.
/// Cone geometry: player must be below dragon (Y >= dragon Y), and within
/// a triangular region defined by the relative position and facing direction.
/// The cone is 3:1 aspect (3× lateral distance = vertical threshold).
fn dragon_fire_breath_cone(
    enemy: &ActiveEnemy,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) -> bool {
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return false;
    };
    let dragon_x = (enemy.x_pos >> 16) as i16;
    let dragon_y = (enemy.y_pos >> 16) as i16;
    let player_x = (px >> 16) as i16;
    let player_y = (py >> 16) as i16;

    // Player must be at or below dragon. ASM: cmp.w y_pos(a4),y_pos(a3) / blt return.
    if player_y < dragon_y {
        return false;
    }

    // ASM Dragon_Flying.asm loc_34836 keeps every lateral operation in d7.w:
    // subtract once, add the result twice, then subtract/negate for the
    // right-facing branch. Each instruction wraps independently.
    let dx = dragon_x.wrapping_sub(player_x);
    let three_dx = dx.wrapping_add(dx).wrapping_add(dx);
    let cone_delta = if enemy.facing == Direction::Right {
        three_dx.wrapping_sub(dx).wrapping_neg()
    } else {
        three_dx
    };

    // ASM Dragon_Flying.asm loc_34860 adds and subtracts the Y bounds as words
    // before applying signed BGT/BLT comparisons.
    let cone_y = cone_delta.wrapping_add(dragon_y);
    player_y >= cone_y.wrapping_sub(8) && player_y <= cone_y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::mini_hopping_skull_test_enemy;

    fn dragon_flying_test_enemy(
        x_px: u32,
        y_px: u32,
        x_velocity: i32,
        y_velocity: i32,
    ) -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Dragon,
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            x_velocity,
            y_velocity,
            dragon_phase: DragonFlyingState::Descending,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn dragon_test_grid(
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
    fn dragon_negative_drift_acceleration_matches_loc_347b2() {
        assert_eq!(
            dragon_drift_negative_velocity_loc_347b2(-0x800),
            -0xA00,
            "ASM Dragon_Flying.asm loc_347B2 subtracts `$200` from a negative drift velocity"
        );
        assert_eq!(
            dragon_drift_negative_velocity_loc_347b2(-0x17F00),
            -0x18000,
            "ASM Dragon_Flying.asm loc_347B2 clamps when subtracting `$200` would pass `$FFFE8000`"
        );
        assert_eq!(
            dragon_drift_negative_velocity_loc_347b2(-0x18000),
            -0x18000,
            "ASM Dragon_Flying.asm loc_347B2 preserves the `$FFFE8000` negative speed cap"
        );
        assert_eq!(
            dragon_drift_negative_velocity_loc_347b2(i32::MIN),
            i32::MAX - DRAGON_DRIFT_ACCEL + 1,
            "ASM Dragon_Flying.asm loc_347B2 `subi.l #$200,d7` wraps before the signed cap comparison"
        );
    }

    #[test]
    fn dragon_positive_drift_acceleration_matches_loc_3478c() {
        assert_eq!(
            dragon_drift_positive_velocity_loc_3478c(0x800),
            0xA00,
            "ASM Dragon_Flying.asm loc_3478C adds `$200` to a positive drift velocity"
        );
        assert_eq!(
            dragon_drift_positive_velocity_loc_3478c(0x17F00),
            0x18000,
            "ASM Dragon_Flying.asm loc_3478C clamps when adding `$200` would pass `$18000`"
        );
        assert_eq!(
            dragon_drift_positive_velocity_loc_3478c(i32::MAX),
            i32::MIN + DRAGON_DRIFT_ACCEL - 1,
            "ASM Dragon_Flying.asm loc_3478C `addi.l #$200,d7` wraps before the signed cap comparison"
        );
    }

    #[test]
    fn dragon_drift_tail_store_matches_loc_347c8() {
        assert_eq!(
            dragon_drift_store_loc_347c8(0x18000, 0),
            DragonDriftStoreLoc347C8 {
                x_velocity: 0x18000,
                checks_fire_breath_cone: false,
            },
            "ASM Dragon_Flying.asm loc_347C8 stores d7 into x_vel and skips loc_34836 when d1 is zero"
        );
        assert_eq!(
            dragon_drift_store_loc_347c8(-0x18000, 1),
            DragonDriftStoreLoc347C8 {
                x_velocity: -0x18000,
                checks_fire_breath_cone: true,
            },
            "ASM Dragon_Flying.asm loc_347C8 stores the signed d7 velocity before the non-zero d1 fire-cone branch"
        );
    }

    #[test]
    fn dragon_return_y_velocity_caps_match_loc_34b4c_34b70() {
        assert_eq!(
            dragon_return_y_velocity_loc_34b4c(-0xEC00),
            -0xF000,
            "ASM Dragon_Flying.asm loc_34B4C keeps the signed `$FFFF1000` threshold when subtracting `$400` lands exactly on it"
        );
        assert_eq!(
            dragon_return_y_velocity_loc_34b4c(-0xED00),
            -0x8000,
            "ASM Dragon_Flying.asm loc_34B4C/loc_34B70 replaces values below `$FFFF1000` with the signed hard cap `$FFFF8000`"
        );
        assert_eq!(
            dragon_return_y_velocity_loc_34b4c(i32::MIN),
            i32::MAX - DRAGON_RETURN_DECEL + 1,
            "ASM Dragon_Flying.asm loc_34B4C `subi.l #$400,d7` wraps before the signed `$FFFF1000` comparison"
        );
    }

    #[test]
    fn dragon_flying_terrain_collision_type_floor_requires_positive_y_velocity_and_slope_ids() {
        let slope_up_grid = dragon_test_grid(20, 15, &[(6, 6, 0x40, 0x00)]);
        let slope_down_grid = dragon_test_grid(20, 15, &[(6, 6, 0x50, 0x00)]);
        let stationary = dragon_flying_test_enemy(100, 107, 0, 0);

        assert_eq!(
            dragon_flying_terrain_collision_type(
                &dragon_flying_test_enemy(100, 107, 0, 1 << 16),
                &slope_up_grid,
            ),
            Some(DRAGON_FLYING_COLID_SLOPEUP),
            "ASM Dragon_Flying.asm loc_349DE routes the positive downward floor collision `$14` through off_349C6"
        );
        assert_eq!(
            dragon_flying_terrain_collision_type(
                &dragon_flying_test_enemy(100, 107, 0, 1 << 16),
                &slope_down_grid,
            ),
            Some(DRAGON_FLYING_COLID_SLOPEDOWN),
            "ASM Dragon_Flying.asm loc_349DE routes the positive downward floor collision `$18` through off_349C6"
        );
        assert_eq!(
            dragon_flying_terrain_collision_type(&stationary, &slope_up_grid),
            None,
            "ASM Dragon_Flying.asm loc_349DE only consumes the floor/slope collision that descending movement produced"
        );
    }

    #[test]
    fn dragon_flying_terrain_collision_type_ceiling_requires_negative_y_velocity() {
        let grid = dragon_test_grid(8, 8, &[(5, 3, 0x60, 0x01)]);

        assert_eq!(
            dragon_flying_terrain_collision_type(
                &dragon_flying_test_enemy(82, 64, 0, -(1 << 16)),
                &grid,
            ),
            Some(DRAGON_FLYING_COLID_CEILING),
            "ASM Dragon_Flying.asm loc_349DE routes the upward ceiling collision `$10` through off_349C6"
        );
        assert_eq!(
            dragon_flying_terrain_collision_type(&dragon_flying_test_enemy(82, 64, 0, 0), &grid),
            None,
            "ASM Dragon_Flying.asm loc_349DE should not see a ceiling collision without upward movement"
        );
        assert_eq!(
            dragon_flying_terrain_collision_type(
                &dragon_flying_test_enemy(82, 64, 0, 1 << 16),
                &grid,
            ),
            None,
            "ASM Dragon_Flying.asm loc_349DE should not treat downward movement as a ceiling response"
        );
    }

    #[test]
    fn dragon_flying_terrain_collision_type_walls_follow_signed_x_velocity() {
        let grid = dragon_test_grid(8, 8, &[(5, 1, 0x60, 0x01), (5, 2, 0x60, 0x01)]);

        assert_eq!(
            dragon_flying_terrain_collision_type(
                &dragon_flying_test_enemy(88, 31, -(1 << 16), 0),
                &grid,
            ),
            Some(DRAGON_FLYING_COLID_LEFTWALL),
            "ASM Dragon_Flying.asm loc_349DE routes the moving-left wall collision `$08` through off_349C6"
        );
        assert_eq!(
            dragon_flying_terrain_collision_type(
                &dragon_flying_test_enemy(74, 31, 1 << 16, 0),
                &grid,
            ),
            Some(DRAGON_FLYING_COLID_RIGHTWALL),
            "ASM Dragon_Flying.asm loc_349DE routes the moving-right wall collision `$04` through off_349C6"
        );
        assert_eq!(
            dragon_flying_terrain_collision_type(&dragon_flying_test_enemy(88, 31, 0, 0), &grid),
            None,
            "ASM Dragon_Flying.asm loc_349DE should not see a wall collision without horizontal movement"
        );
    }

    #[test]
    fn dragon_enter_terrain_response_collision_10_enters_loc_34a68_down_slide() {
        let mut enemy = dragon_flying_test_enemy(82, 64, 0, 0x0000_1234);

        dragon_enter_terrain_response(&mut enemy, DRAGON_FLYING_COLID_CEILING);

        assert_eq!(
            enemy.dragon_phase,
            DragonFlyingState::TerrainSlideDown,
            "ASM Dragon_Flying.asm off_349C6 dispatches collision `$10` byte offset `$0C` to loc_34A68"
        );
        assert_eq!(
            enemy.y_velocity, 0x0003_1234,
            "ASM Dragon_Flying.asm loc_34A68 writes word `$0003` into y_vel and preserves its low word"
        );
    }

    #[test]
    fn dragon_enter_terrain_response_routes_remaining_off_349c6_entries() {
        let mut right_slide = dragon_flying_test_enemy(82, 64, 0x0000_1234, 0x0000_5678);
        dragon_enter_terrain_response(&mut right_slide, DRAGON_FLYING_COLID_LEFTWALL);
        assert_eq!(
            right_slide.dragon_phase,
            DragonFlyingState::TerrainSlideRight,
            "ASM Dragon_Flying.asm off_349C6 dispatches collision `$08` byte offset `$04` to loc_34A00"
        );
        assert_eq!(right_slide.x_velocity, 0x0002_1234);
        assert_eq!(right_slide.y_velocity, 0);

        let mut rebound_left = dragon_flying_test_enemy(100, 80, 0x0001_0000, 0x0002_0000);
        dragon_enter_terrain_response(&mut rebound_left, DRAGON_FLYING_COLID_SLOPEUP);
        assert_eq!(
            rebound_left.dragon_phase,
            DragonFlyingState::TerrainReboundLeft,
            "ASM Dragon_Flying.asm off_349C6 dispatches collision `$14` byte offset `$10` to loc_34A88"
        );
        assert_eq!(rebound_left.x_pos, 99 << 16);
        assert_eq!(rebound_left.y_pos, 78 << 16);
        assert_eq!(rebound_left.x_velocity, 0);
        assert_eq!(rebound_left.y_velocity, 0);
        assert_eq!(
            rebound_left.dragon_phase_frames,
            DRAGON_TERRAIN_REBOUND_START_WAIT_FRAMES
        );

        let mut rebound_right = dragon_flying_test_enemy(100, 80, -0x0001_0000, 0x0002_0000);
        dragon_enter_terrain_response(&mut rebound_right, DRAGON_FLYING_COLID_SLOPEDOWN);
        assert_eq!(
            rebound_right.dragon_phase,
            DragonFlyingState::TerrainReboundRight,
            "ASM Dragon_Flying.asm off_349C6 dispatches collision `$18` byte offset `$14` to loc_34ADC"
        );
        assert_eq!(rebound_right.x_pos, 101 << 16);
        assert_eq!(rebound_right.y_pos, 78 << 16);
        assert_eq!(rebound_right.x_velocity, 0);
        assert_eq!(rebound_right.y_velocity, 0);
        assert_eq!(
            rebound_right.dragon_phase_frames,
            DRAGON_TERRAIN_REBOUND_START_WAIT_FRAMES
        );
    }

    #[test]
    fn dragon_terrain_rebound_wait_frames_gate_velocity_until_counter_reaches_zero() {
        let mut left = dragon_flying_test_enemy(100, 80, 0, 0);
        left.dragon_phase = DragonFlyingState::TerrainReboundLeft;
        left.dragon_phase_frames = 2;

        step_dragon_terrain_response(&mut left, None);
        assert_eq!(left.dragon_phase_frames, 1);
        assert_eq!(left.x_velocity, 0);
        assert_eq!(left.y_velocity, 0);

        step_dragon_terrain_response(&mut left, None);
        assert_eq!(left.dragon_phase_frames, 0);
        assert_eq!(
            (left.x_velocity, left.y_velocity),
            (-0x0002_0000, -0x0002_0000),
            "ASM Dragon_Flying.asm loc_34A88 waits two hibernated frames before writing rebound velocity"
        );
    }

    #[test]
    fn dragon_terrain_rebound_right_wait_tail_uses_loc_34adc_velocity() {
        let mut right = dragon_flying_test_enemy(100, 80, 0, 0);
        right.dragon_phase = DragonFlyingState::TerrainReboundRight;
        right.dragon_phase_frames = 1;

        step_dragon_terrain_response(&mut right, None);

        assert_eq!(right.dragon_phase_frames, 0);
        assert_eq!(
            (right.x_velocity, right.y_velocity),
            (0x0002_0000, -0x0002_0000),
            "ASM Dragon_Flying.asm loc_34ADC mirrors loc_34A88 but starts with positive x_vel"
        );
    }

    #[test]
    fn dragon_terrain_response_steps_each_recovery_phase_once() {
        let cases = [
            (
                DragonFlyingState::TerrainSlideRight,
                0x0002_0000,
                0,
                DragonFlyingState::TerrainSlideRight,
                0x0001_F000,
                0,
                "ASM Dragon_Flying.asm loc_34A0A subtracts `$1000` from right-slide x_vel",
            ),
            (
                DragonFlyingState::TerrainSlideLeft,
                -0x0002_0000,
                0,
                DragonFlyingState::TerrainSlideLeft,
                -0x0001_F000,
                0,
                "ASM Dragon_Flying.asm loc_34A2E adds `$1000` to left-slide x_vel",
            ),
            (
                DragonFlyingState::TerrainSlideUp,
                0,
                -0x0003_0000,
                DragonFlyingState::TerrainSlideUp,
                0,
                -0x0002_B000,
                "ASM Dragon_Flying.asm loc_34A4E adds `$5000` to upward-slide y_vel",
            ),
            (
                DragonFlyingState::TerrainSlideDown,
                0,
                0x0003_0000,
                DragonFlyingState::TerrainSlideDown,
                0,
                0x0002_E000,
                "ASM Dragon_Flying.asm loc_34A6E subtracts `$2000` from downward-slide y_vel",
            ),
            (
                DragonFlyingState::TerrainReboundLeft,
                -0x0002_0000,
                -0x0002_0000,
                DragonFlyingState::TerrainReboundLeft,
                -0x0002_0000,
                -0x0001_E000,
                "ASM Dragon_Flying.asm loc_34AC2 adds `$2000` to rebound y_vel",
            ),
            (
                DragonFlyingState::TerrainReboundRight,
                0x0002_0000,
                -0x0002_0000,
                DragonFlyingState::TerrainReboundRight,
                0x0002_0000,
                -0x0001_E000,
                "ASM Dragon_Flying.asm loc_34B16 shares the loc_34AC2 rebound y_vel update",
            ),
        ];

        for (phase, x_velocity, y_velocity, expected_phase, expected_x, expected_y, message) in
            cases
        {
            let mut enemy = dragon_flying_test_enemy(100, 80, x_velocity, y_velocity);
            enemy.dragon_phase = phase;
            enemy.dragon_phase_frames = 0;

            step_dragon_terrain_response(&mut enemy, None);

            assert_eq!(enemy.dragon_phase, expected_phase, "{message}");
            assert_eq!(enemy.x_velocity, expected_x, "{message}");
            assert_eq!(enemy.y_velocity, expected_y, "{message}");
        }
    }

    #[test]
    fn dragon_descent_move_tail_installs_next_script_entry_loc_34924_348ce() {
        let mut enemy = dragon_flying_test_enemy(100, 80, 0, 0);
        enemy.facing = Direction::Left;
        enemy.dragon_descent_index = 0;
        enemy.dragon_descent_frames = 1;

        step_dragon_descent(&mut enemy, None, &mut None);

        assert_eq!(
            enemy.dragon_descent_index, 1,
            "ASM Dragon_Flying.asm loc_34924 advances the descent script after the DBF tail underflows"
        );
        assert_eq!(
            enemy.dragon_descent_frames, 8,
            "ASM loc_3488A/loc_348CE immediately installs and consumes the next nine-frame entry"
        );
        assert_eq!(
            (enemy.x_velocity, enemy.y_velocity),
            (-0x0001_0000, 0x0002_0000)
        );
    }

    #[test]
    fn dragon_flying_exports_main_and_dive_script_mappings() {
        let mut enemy = dragon_flying_test_enemy(100, 80, 0, 0);
        enemy.dragon_phase = DragonFlyingState::Hovering;

        for (elapsed, expected) in [(0, 0), (1, 0), (6, 0), (7, 1), (21, 3), (84, 4), (139, 3)] {
            enemy.wait_timer = elapsed;
            assert_eq!(
                dragon_flying_sprite_animation_phase(&enemy),
                Some(expected),
                "ASM Init_Animation loc_1040 and Dragon_Flying.asm stru_34600 tick {elapsed}"
            );
        }

        enemy.dragon_phase = DragonFlyingState::Descending;
        for (script_index, expected) in [
            (0, Some(6)),
            (1, Some(8)),
            (4, Some(8)),
            (5, Some(6)),
            (8, Some(2)),
            (9, Some(7)),
            (12, Some(8)),
            (14, Some(8)),
            (16, Some(7)),
        ] {
            enemy.dragon_descent_index = script_index;
            assert_eq!(
                dragon_flying_sprite_animation_phase(&enemy),
                expected,
                "ASM Dragon_Flying.asm unk_34554 entry {script_index} indexes off_345F6"
            );
        }
    }

    #[test]
    fn dragon_descent_spawn_fire_initialises_projectile_then_consumes_next_move() {
        let mut enemy = dragon_flying_test_enemy(100, 80, 0x0001_8000, 0);
        enemy.facing = Direction::Right;
        enemy.dragon_descent_index = 3;
        enemy.dragon_descent_frames = 0;
        let mut projectiles = Vec::new();
        let mut projectiles_ref = Some(&mut projectiles);

        step_dragon_descent(&mut enemy, None, &mut projectiles_ref);

        assert_eq!(
            enemy.dragon_descent_index, 4,
            "ASM Dragon_Flying.asm loc_348A0 spawn-fire sentinel advances to the following movement entry"
        );
        assert_eq!(enemy.dragon_descent_frames, 5);
        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0],
            EnemyProjectile {
                x_pos: 110 << 16,
                y_pos: 90 << 16,
                x_velocity: 0x0001_0000,
                y_velocity: 0x0003_0000,
                terrain_kill: true,
                kind: EnemyProjectileKind::DragonFlyingDescentFire,
                dragon_flying_descent_fire_state:
                    DragonFlyingDescentFireState::FirstAnimationLoc34C26,
                dragon_flying_descent_fire_counter: 6,
                ..Default::default()
            },
            "ASM Dragon_Flying.asm loc_348A0/loc_34BE6 seeds the descent fire projectile without a generic DBF lifetime"
        );
    }

    #[test]
    fn dragon_flying_descent_fire_loc_34bac_34be6_copies_and_wraps_position_words() {
        let setup = dragon_flying_descent_fire_setup_loc_34BE6(
            (0xFFF8 << 16) | 0xBEEF,
            (0xFFFA << 16) | 0xCAFE,
            0,
            3,
            Direction::Right,
        );

        assert_eq!(
            setup.x_pos, 0x0002_0000,
            "ASM Dragon_Flying.asm loc_34BAC/loc_34BE6 copies X as a word and wraps ADD.W #$A"
        );
        assert_eq!(
            setup.y_pos, 0x0004_0000,
            "ASM Dragon_Flying.asm loc_34BAC copies Y as a word and wraps ADDI.W #$A"
        );
    }

    #[test]
    fn dragon_fire_breath_cone_boundaries_match_loc_34836_34860() {
        let mut dragon = dragon_flying_test_enemy(100, 80, 0, 0);
        dragon.facing = Direction::Left;

        assert!(
            dragon_fire_breath_cone(&dragon, Some(100 << 16), Some(80 << 16)),
            "ASM Dragon_Flying.asm loc_34836 accepts a player exactly level with the dragon before loc_34860 cone bounds"
        );
        assert!(
            !dragon_fire_breath_cone(&dragon, Some(100 << 16), Some(79 << 16)),
            "ASM Dragon_Flying.asm loc_34836 rejects player y below dragon y with signed `blt`"
        );
        assert!(
            !dragon_fire_breath_cone(&dragon, Some(96 << 16), Some(83 << 16)),
            "ASM Dragon_Flying.asm loc_34860 rejects positions above the cone's `d7 + y - 8` lower bound"
        );
        assert!(
            dragon_fire_breath_cone(&dragon, Some(98 << 16), Some(86 << 16)),
            "ASM Dragon_Flying.asm loc_34836 uses `3 * dx` for left-facing cone depth"
        );

        dragon.facing = Direction::Right;
        assert!(
            dragon_fire_breath_cone(&dragon, Some(102 << 16), Some(84 << 16)),
            "ASM Dragon_Flying.asm loc_34836 negates `2 * dx` for the right-facing cone"
        );
    }

    #[test]
    fn dragon_fire_breath_cone_wraps_word_arithmetic_loc_34836() {
        let mut dragon = dragon_flying_test_enemy(0x5556, 100, 0, 0);
        dragon.facing = Direction::Left;

        assert!(
            dragon_fire_breath_cone(&dragon, Some(0), Some(100 << 16)),
            "ASM Dragon_Flying.asm loc_34836 wraps three word-width copies of $5556 to +2 before adding dragon Y",
        );

        dragon.x_pos = 0x7FFF_0000;
        dragon.facing = Direction::Right;
        assert!(
            dragon_fire_breath_cone(&dragon, Some(0), Some(100 << 16)),
            "ASM Dragon_Flying.asm loc_34836 wraps and negates 2 * $7FFF to +2 for the right-facing cone",
        );
    }

    #[test]
    fn dragon_dive_script_signed_moves_match_unk_34554() {
        let mut moves = Vec::new();
        let mut sentinels = Vec::new();
        for (index, action) in DRAGON_DIVE_SCRIPT.iter().copied().enumerate() {
            match action {
                DragonDiveAction::Move {
                    x_vel,
                    y_vel,
                    frames,
                } => moves.push((index, x_vel, y_vel, frames)),
                DragonDiveAction::SpawnFire => sentinels.push((index, "E19F")),
                DragonDiveAction::FlipDirection => sentinels.push((index, "DD48")),
                DragonDiveAction::RestartScript => sentinels.push((index, "D8F1")),
            }
        }

        assert_eq!(
            moves,
            vec![
                (0, -1, 1, 12),
                (1, -1, 2, 9),
                (2, -1, 3, 6),
                (4, -1, 4, 6),
                (5, -2, 3, 3),
                (6, -2, 2, 3),
                (7, -2, 1, 3),
                (8, -2, 0, 3),
                (9, -2, -1, 3),
                (10, -2, -2, 3),
                (11, -2, -3, 3),
                (12, -1, -4, 6),
                (14, -1, -3, 6),
                (15, -1, -2, 9),
                (16, -1, -1, 12),
            ],
            "ASM kid.asm unk_34554 movement words use signed x/y bytes; loc_348CE sign-extends them before storing x_vel/y_vel"
        );
        assert_eq!(
            sentinels,
            vec![(3, "E19F"), (13, "DD48"), (17, "D8F1")],
            "ASM kid.asm unk_34554 sentinels spawn fire, flip direction, then restart the main Dragon Flying script"
        );
    }

    #[test]
    fn dragon_drift_dbf_tail_matches_loc_347fc() {
        assert_eq!(
            dragon_drift_dbf_tail_loc_347fc(0x0028),
            DragonDriftDbfTailLoc347FC {
                next_d0: 0x0027,
                loops_to_loc_3478c: true,
                advances_script: false,
            },
            "ASM Dragon_Flying.asm loc_347FC decrements d0 and branches back while the result is not `$FFFF`"
        );
        assert_eq!(
            dragon_drift_dbf_tail_loc_347fc(0x0000),
            DragonDriftDbfTailLoc347FC {
                next_d0: 0xFFFF,
                loops_to_loc_3478c: false,
                advances_script: true,
            },
            "ASM Dragon_Flying.asm loc_347FC falls through to loc_346DA only after dbf decrements zero to `$FFFF`"
        );
    }
}
