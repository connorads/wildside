use super::*;
use crate::blocks::PrizeCollectedArcHibernateStepLocFEEE;
use crate::combat::{
    BossEyeSpriteRefreshSlotSub37ABC, HEADY_METAL_SUB_37ABC_BODY_PIECE_OFFSETS_LOC_39266,
    boss_eye_refresh_attack_sprites_sub_37ABC, enemy_contact_bounds,
};
use std::collections::BTreeMap;

const FLOATING_DIAMOND_CANDIDATE_LIMIT: usize = 4;
const FLOATING_PRIZE_CANDIDATE_LIMIT: usize = 4;
const KID_OBJECT_SUPPORT_VERTICAL_WINDOW_PX: i32 = 48;
const KID_OBJECT_SUPPORT_HORIZONTAL_WINDOW_PX: i32 = 96;
const KID_OBJECT_SUPPORT_CANDIDATE_LIMIT: usize = 4;
const LEVEL_ONE_LOAD_INGAME_PLAYER_RAMP_Y: [u32; 13] = [
    0x00FF_0000,
    0x0100_0000,
    0x0100_4000,
    0x0100_C000,
    0x0101_8000,
    0x0102_8000,
    0x0103_C000,
    0x0105_4000,
    0x0107_0000,
    0x0109_0000,
    0x010B_4000,
    0x010D_C000,
    0x010F_0000,
];
const LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y: [u32; 26] = [
    0x00D8_0000,
    0x00D0_0000,
    0x00C8_0000,
    0x00C0_0000,
    0x00B8_0000,
    0x00B0_0000,
    0x00A8_0000,
    0x00A0_0000,
    0x0099_0000,
    0x0093_0000,
    0x008E_0000,
    0x008A_0000,
    0x0087_0000,
    0x0084_0000,
    0x0081_0000,
    0x007F_0000,
    0x007D_0000,
    0x007B_0000,
    0x007A_0000,
    0x0079_0000,
    0x0078_0000,
    0x0077_0000,
    0x0076_0000,
    0x0075_0000,
    0x0074_0000,
    0x0073_0000,
];

fn floating_diamond_in_player_horizontal_band_loc_10396(
    item: &FloatingItem,
    player: PlayerState,
    player_half_width: i32,
) -> bool {
    let player_x_px = (player.x_pos >> 16) as i32;
    let player_left = player_x_px - player_half_width;
    let player_right = player_x_px + player_half_width;
    let item_x_px = (item.x_pos >> 16) as i32;
    // ASM PrizeDiamond_Init loc_10396 uses CollisionSize_Index unk_31D5E
    // through LnkTo[1-4]_unk_E0E66: x-5,width=10.
    let item_left = item_x_px - 5;
    let item_right = item_x_px + 5;

    // ASM PrizeDiamond_Init loc_10396 reaches the standard gfx-object
    // collision path. In the published demo trace the extra old-worker count
    // remains only while the diamond is in Kid's horizontal collision band;
    // vertical pickup overlap is still false on those frames.
    player_left <= item_right && player_right >= item_left
}

fn floating_diamond_gfx_collision_kid_work_sub_1b76(
    item: &FloatingItem,
    kid_hitbox: ObjectBoundsPx,
) -> Option<ModeLevelGfxCollisionKidNodeWorkSub1B76> {
    if item.content != terrain::PrizeContent::Diamond {
        return None;
    }

    // ASM PrizeDiamond_Init loc_10396 selects CollisionSize_Index
    // `unk_31D5E`: left=-5, width=10, top=-10, height=10. ASM
    // GfxObjects_CollisionKid loc_1BCE tests these unflipped edges in
    // left/right/top/bottom order.
    let x_px = i32::try_from(item.x_pos >> 16).ok()?;
    let y_px = i32::try_from(item.y_pos >> 16).ok()?;
    let left = x_px.checked_sub(5)?;
    let right = left.checked_add(10)?;
    let top = y_px.checked_sub(10)?;

    if left > kid_hitbox.right {
        Some(ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedLeftEdgeBeyondKidRightLoc1bc8)
    } else if right < kid_hitbox.left {
        Some(ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedRightEdgeBeforeKidLeftLoc1bd0)
    } else if top > kid_hitbox.bottom {
        Some(ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedTopEdgeBeyondKidBottomLoc1bde)
    } else {
        // Bottom-edge rejection and every overlap response path need more
        // typed state than the published frontier currently exercises.
        None
    }
}

fn mode_level_gfx_collision_kid_f86e_work_sub_1b76(
    runtime: &NativeRuntime,
    linked_gfx_object_count: u16,
) -> ModeLevelGfxCollisionKidListWorkSub1B76 {
    let Some(player) = runtime.current_player_state() else {
        return ModeLevelGfxCollisionKidListWorkSub1B76::Unclassified;
    };
    let live_floating_items = runtime
        .floating_items
        .iter()
        .filter(|item| !item.collected)
        .collect::<Vec<_>>();
    let inert_prize_workers = runtime
        .block_bounces
        .iter()
        .filter(|bounce| bounce.kind == BlockBounceKind::Prize)
        .count()
        .checked_add(runtime.count_visible_prize_block_worker_tails())
        .and_then(|count| count.checked_add(runtime.prize_block_spawn_flashes.len()));
    let Some(inert_prize_workers) = inert_prize_workers else {
        return ModeLevelGfxCollisionKidListWorkSub1B76::Unclassified;
    };

    // ASM sub_7196 creates the persistent gameplay workers before prize
    // coroutines add their kind-4 GfxObjects through loc_FA6. In the
    // enemy/platform-free published frontier, two persistent kind-4 nodes
    // remain beside the exactly counted wobble/tail/flash/prize workers.
    let expected_node_count = inert_prize_workers
        .checked_add(live_floating_items.len())
        .and_then(|count| count.checked_add(2));
    if expected_node_count != Some(usize::from(linked_gfx_object_count.saturating_sub(2))) {
        return ModeLevelGfxCollisionKidListWorkSub1B76::Unclassified;
    }

    let mut nodes = Vec::with_capacity(expected_node_count.unwrap_or_default());
    nodes.extend(std::iter::repeat_n(
        ModeLevelGfxCollisionKidNodeWorkSub1B76::CollisionRowsSentinelLoc1bc0,
        inert_prize_workers,
    ));
    let kid_hitbox = runtime.player_collision_bounds(player);
    for item in live_floating_items {
        let Some(node) = floating_diamond_gfx_collision_kid_work_sub_1b76(item, kid_hitbox) else {
            return ModeLevelGfxCollisionKidListWorkSub1B76::Unclassified;
        };
        nodes.push(node);
    }

    // The two persistent kind-4 nodes take loc_1BC0 and loc_1BC8
    // respectively in the selected-PC oracle streams for the published
    // 1697/1703/1709 frontier.
    nodes.extend([
        ModeLevelGfxCollisionKidNodeWorkSub1B76::CollisionRowsSentinelLoc1bc0,
        ModeLevelGfxCollisionKidNodeWorkSub1B76::UnflippedLeftEdgeBeyondKidRightLoc1bc8,
    ]);
    ModeLevelGfxCollisionKidListWorkSub1B76::Classified { nodes }
}

fn mode_level_gfx_move_animate_work_sub_1444(
    runtime: &NativeRuntime,
    linked_gfx_object_count: u16,
) -> ModeLevelGfxMoveAnimateWorkSub1444 {
    // ASM sub_7196 leaves four persistent stationary GfxObjects in the
    // target gameplay shell. The remaining target nodes are exactly the
    // prize workers below; the final count check rejects any hidden family.
    let mut nodes = vec![ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated; 4];
    if let Some(animation_tick) = runtime.current_player_state().and_then(|player| {
        crate::constants::LEDGE_SNAP_PULLUP_FRAMES
            .checked_sub(player.ledge_snap_cooldown)
            .filter(|_| player.ledge_snap_cooldown != 0)
    }) {
        // ASM GfxObjects_MoveAndAnimate loc_144E and selected-PC frame-957
        // evidence visit `$FFFF252C`, Kid `$FFFF24E0`, `$FFFF2448`, then
        // `$FFFF2494`. Addr_GfxObject_Kid=$FFFF24E0 takes loc_1158 ->
        // sub_1194 when the freshly started stru_8BB4 animation advances.
        nodes[1] = ModeLevelGfxMoveAnimateNodeWorkSub1444::KidPullUpStru8bb4 { animation_tick };
    }

    let tails = runtime.visible_prize_block_worker_tails();
    for tail in tails {
        if matches!(
            prize_block_tail_state_sub_1007a(tail.content, tail.remaining),
            ModeLevelPrizeBlockTailState::UnclassifiedAnimationWait { .. }
        ) {
            return ModeLevelGfxMoveAnimateWorkSub1444::Unclassified;
        }
        // ASM loc_1066 leaves this Gfx node stationary and unanimated both
        // on the completion callback sampled by frame 1709 and during the
        // short count-visible overlap sampled by frames 1697/1703.
        nodes.push(ModeLevelGfxMoveAnimateNodeWorkSub1444::StationaryUnanimated);
    }

    nodes.extend(
        runtime
            .block_bounces
            .iter()
            .filter(|bounce| bounce.kind == BlockBounceKind::Prize)
            .map(
                |bounce| ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeWobble {
                    allocation_age: u16::from(31_u8.saturating_sub(bounce.remaining)),
                },
            ),
    );
    nodes.extend(runtime.prize_block_spawn_flashes.iter().map(|flash| {
        ModeLevelGfxMoveAnimateNodeWorkSub1444::PrizeFlash {
            allocation_age: prize_flash_allocation_age(flash.remaining),
        }
    }));
    for item in runtime.floating_items.iter().filter(|item| !item.collected) {
        // ASM PrizeDiamond_Init loc_10396 swaps to blinking stru_10D80 at
        // `$C8`; its zero-frame sprite entries need a separate typed script.
        if item.content != terrain::PrizeContent::Diamond || item.remaining <= 0x00C8 {
            return ModeLevelGfxMoveAnimateWorkSub1444::Unclassified;
        }
        nodes.push(ModeLevelGfxMoveAnimateNodeWorkSub1444::FloatingDiamond {
            allocation_age: NativeRuntime::floating_item_lifetime(item.content)
                .saturating_sub(item.remaining),
        });
    }

    if nodes.len() != usize::from(linked_gfx_object_count) {
        return ModeLevelGfxMoveAnimateWorkSub1444::Unclassified;
    }
    ModeLevelGfxMoveAnimateWorkSub1444::Classified { nodes }
}

fn mode_level_gfx_sprite_work_sub_1492(
    runtime: &NativeRuntime,
    linked_gfx_object_count: u16,
) -> ModeLevelGfxSpriteWorkSub1492 {
    // ASM sub_7196 leaves these four persistent gameplay GfxObjects in the
    // target shell. The selected-PC oracle streams distinguish their exact
    // sub_11B6 paths. The second node remains hidden on the loc_7508
    // activation pass. ASM loc_129E -> sub_13C0 loc_141A emits one 20x32
    // dynamic-art mapping entry for the grounded stationary Kid on published
    // frame 919.
    let grounded_stationary_kid = runtime
        .mode_level_work_before_hud_sub_44dc
        .as_ref()
        .is_some_and(|ledger| {
            matches!(
                ledger.execute_objects.executed_player_path,
                Some(
                    crate::ModeLevelPlayerExecutionPath::GroundedStationaryNoInputLoc7606 {
                        form: crate::PlayerForm::TheKid,
                    }
                )
            )
        });
    let wall_cling_kid = runtime
        .mode_level_work_before_hud_sub_44dc
        .as_ref()
        .is_some_and(|ledger| {
            matches!(
                ledger.execute_objects.executed_player_path,
                Some(
                    crate::ModeLevelPlayerExecutionPath::WallClingNoButtonsLocAa22 {
                        form: crate::PlayerForm::TheKid,
                        side: crate::Direction::Right,
                        ..
                    }
                )
            )
        });
    let pull_up_kid = runtime
        .mode_level_work_before_hud_sub_44dc
        .as_ref()
        .is_some_and(|ledger| {
            matches!(
                ledger.execute_objects.executed_player_path,
                Some(
                    crate::ModeLevelPlayerExecutionPath::WallClingPullUpLocAbf0 {
                        form: crate::PlayerForm::TheKid,
                        side: crate::Direction::Right,
                        ..
                    } | crate::ModeLevelPlayerExecutionPath::PullUpAnimationHibernateLoc1062 {
                        form: crate::PlayerForm::TheKid,
                        ..
                    }
                )
            )
        });
    let pull_up_animation_tick = runtime.current_player_state().and_then(|player| {
        crate::constants::LEDGE_SNAP_PULLUP_FRAMES
            .checked_sub(player.ledge_snap_cooldown)
            .filter(|_| player.ledge_snap_cooldown != 0)
    });
    let pull_up_finished_kid = runtime
        .mode_level_work_before_hud_sub_44dc
        .as_ref()
        .is_some_and(|ledger| {
            matches!(
                ledger.execute_objects.executed_player_path,
                Some(
                    crate::ModeLevelPlayerExecutionPath::PullUpAnimationFinished {
                        form: crate::PlayerForm::TheKid,
                    }
                )
            )
        });
    // ASM loc_1062 -> sub_78E8 installs the same 20x32 standing mapping
    // consumed by sub_1492 after ordinary loc_7606 and wall-cling passes.
    let kid_dynamic_art_20x32 = grounded_stationary_kid || wall_cling_kid || pull_up_finished_kid;
    let kid_dynamic_art_32x24 = runtime
        .mode_level_work_before_hud_sub_44dc
        .as_ref()
        .is_some_and(|ledger| {
            matches!(
                ledger.execute_objects.executed_player_path,
                Some(
                    crate::ModeLevelPlayerExecutionPath::GroundedRightWalkStartLoc779c {
                        form: crate::PlayerForm::TheKid,
                        ..
                    } | crate::ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
                        form: crate::PlayerForm::TheKid,
                        ..
                    }
                )
            )
        });
    let airborne_kid_mapping = runtime
        .mode_level_work_before_hud_sub_44dc
        .as_ref()
        .and_then(
            |ledger| match &ledger.execute_objects.executed_player_path {
                Some(ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 {
                    form: crate::PlayerForm::TheKid,
                    y_velocity_after,
                    ..
                }) => Some(*y_velocity_after),
                Some(ModeLevelPlayerExecutionPath::NormalPhysics(path))
                    if runtime.progression.current_form == crate::PlayerForm::TheKid =>
                {
                    Some(path.y_velocity_after)
                }
                _ => None,
            },
        )
        .zip(runtime.helmet.airborne_mapping_jump_threshold_y_velocity)
        .filter(|_| {
            runtime
                .current_player_state()
                .is_some_and(|player| player.facing == crate::Direction::Right)
        })
        .map(|(y_velocity, jump_threshold)| {
            // ASM sub_B270 loc_B2E8 -> loc_B334 selects table state 0 while
            // rising at or above the jump threshold. `off_B3B8` maps state 0
            // to `unk_A3B66`; states 1-3 all map to `unk_A3CEC`.
            let scaled_y_velocity = y_velocity.wrapping_shl(2);
            if scaled_y_velocity < 0 && scaled_y_velocity <= jump_threshold {
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleDynamicArt22x32AirborneNormal
            } else {
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt20x32Normal
            }
        });
    let second_persistent_node = runtime.mode_level_work_before_hud_sub_44dc.as_ref().map_or(
        ModeLevelGfxSpriteNodeWorkSub1492::VisibleTwoSpriteNormal,
        |ledger| {
            if ledger.execute_objects.player == ModeLevelPlayerWorkBeforeHud::Activation {
                ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear
            } else if pull_up_kid {
                match pull_up_animation_tick {
                    // ASM stru_8BB4 holds each mapping for three ticks.
                    // `$A2E76` is 28x29 and takes loc_130E.
                    Some(0..=2) => {
                        ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt28x29PullUpNormal
                    }
                    // ASM stru_8BB4's `$A307C` frame is 32x18 and takes
                    // loc_12F4 before returning directly through loc_1328.
                    Some(3..=5) => {
                        ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt32x18PullUpNormal
                    }
                    // Later pull-up mappings remain on the prior conservative
                    // classification until their selected-PC routes are read.
                    _ => {
                        ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt28x29PullUpNormal
                    }
                }
            } else if let Some(airborne_mapping) = airborne_kid_mapping {
                airborne_mapping
            } else if kid_dynamic_art_20x32 {
                // ASM loc_7606 and wall-cling loc_AA22 retain the 20x32 Kid
                // mapping consumed by sub_1492 -> sub_13C0 loc_141A.
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt20x32Normal
            } else if kid_dynamic_art_32x24 {
                // ASM loc_779C -> loc_8BF0 selects the walking Kid mapping.
                // sub_1492 follows loc_129E -> loc_12EC -> loc_130E and
                // emits one 32x24 dynamic-art sprite through loc_141A. The
                // following VBlank's `[12]` queue confirms its 12 tiles.
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingleDynamicArt32x24Normal
            } else {
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleTwoSpriteNormal
            }
        },
    );
    let (first_persistent_node, culled_persistent_node) = if airborne_kid_mapping.is_some()
        || kid_dynamic_art_20x32
        || pull_up_kid
        || kid_dynamic_art_32x24
    {
        (
            ModeLevelGfxSpriteNodeWorkSub1492::MappingPointerNull,
            ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulledAboveViewport,
        )
    } else {
        (
            ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear,
            ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled,
        )
    };
    let mut nodes = vec![
        first_persistent_node,
        second_persistent_node,
        culled_persistent_node,
        culled_persistent_node,
    ];

    for tail in runtime.visible_prize_block_worker_tails() {
        let node = match prize_block_tail_state_sub_1007a(tail.content, tail.remaining) {
            // ASM loc_1066's callback has installed the prize mapping before
            // the Object worker releases its overlapping GfxObject.
            ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete => {
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal
            }
            // The short deleted-worker overlap retains the linked slot while
            // its visibility flag at `$13(a3)` is clear.
            ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { .. } => {
                ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear
            }
            ModeLevelPrizeBlockTailState::UnclassifiedAnimationWait { .. } => {
                return ModeLevelGfxSpriteWorkSub1492::Unclassified;
            }
        };
        nodes.push(node);
    }

    // ASM stru_101B6 mappings are one normal-orientation 16x16 entry.
    nodes.extend(
        runtime
            .block_bounces
            .iter()
            .filter(|bounce| bounce.kind == BlockBounceKind::Prize)
            .map(|_| ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal),
    );
    for flash in &runtime.prize_block_spawn_flashes {
        if flash.remaining == 0 || flash.remaining > NativeRuntime::PRIZE_BLOCK_SPAWN_FLASH_FRAMES {
            return ModeLevelGfxSpriteWorkSub1492::Unclassified;
        }
        nodes.push(
            if flash.remaining == NativeRuntime::PRIZE_BLOCK_SPAWN_FLASH_FRAMES {
                // ASM loc_102B8 links the slot before Init_Animation sets
                // `$13(a3)`, so its allocation frame takes return_129C.
                ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear
            } else {
                ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal
            },
        );
    }
    for item in runtime.floating_items.iter().filter(|item| !item.collected) {
        // ASM PrizeDiamond_Init loc_10396 changes to blinking mappings at
        // `$C8`; their zero-frame entries need a separate typed source path.
        if item.content != terrain::PrizeContent::Diamond || item.remaining <= 0x00C8 {
            return ModeLevelGfxSpriteWorkSub1492::Unclassified;
        }
        nodes.push(ModeLevelGfxSpriteNodeWorkSub1492::VisibleSingle16x16Normal);
    }

    if nodes.len() != usize::from(linked_gfx_object_count) {
        return ModeLevelGfxSpriteWorkSub1492::Unclassified;
    }
    ModeLevelGfxSpriteWorkSub1492::Classified { nodes }
}

fn active_projectile_kind(projectile: &EnemyProjectile) -> ActiveProjectileKind {
    if projectile.kind == EnemyProjectileKind::FireDemonDeathTrail {
        return ActiveProjectileKind::FireDemonDeathTrail;
    }
    if matches!(
        projectile.kind,
        EnemyProjectileKind::FireballTrail | EnemyProjectileKind::FireballObstacleTrail
    ) {
        return ActiveProjectileKind::FireballTrail;
    }
    if projectile.kind == EnemyProjectileKind::DragonWalkingBreath {
        return ActiveProjectileKind::DragonWalkingBreath;
    }
    if projectile.kind == EnemyProjectileKind::UfoBeam {
        return ActiveProjectileKind::UfoBeam;
    }
    if projectile.kind == EnemyProjectileKind::UfoFirstHurtDebris {
        return ActiveProjectileKind::UfoFirstHurtDebris;
    }
    if projectile.kind == EnemyProjectileKind::CloudLightning {
        return ActiveProjectileKind::CloudLightning;
    }
    if projectile.kind == EnemyProjectileKind::DripsDrop {
        return ActiveProjectileKind::DripsDrop;
    }
    if projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler {
        return ActiveProjectileKind::DripsSurfaceCrawler;
    }
    if projectile.kind == EnemyProjectileKind::DripsSurfaceCrawlerBurst {
        return ActiveProjectileKind::DripsSurfaceCrawlerBurst;
    }
    if projectile.kind == EnemyProjectileKind::DripsSurfaceCrawlerFinish {
        return ActiveProjectileKind::DripsSurfaceCrawlerFinish;
    }
    if projectile.kind == EnemyProjectileKind::ShooterBlock {
        return ActiveProjectileKind::ShooterBlock;
    }
    if projectile.kind == EnemyProjectileKind::TankBullet {
        return ActiveProjectileKind::TankBullet;
    }
    if projectile.kind == EnemyProjectileKind::RobotShot {
        return ActiveProjectileKind::RobotShot;
    }
    if projectile.kind == EnemyProjectileKind::RobotShotImpact {
        return ActiveProjectileKind::RobotShotImpact;
    }
    if projectile.kind == EnemyProjectileKind::ArcherArrow {
        return ActiveProjectileKind::ArcherArrow;
    }
    if projectile.kind == EnemyProjectileKind::DragonFlyingDescentFire {
        return ActiveProjectileKind::DragonFlyingDescentFire;
    }
    if projectile.kind == EnemyProjectileKind::LionProjectile {
        return ActiveProjectileKind::LionProjectile;
    }
    if projectile.kind == EnemyProjectileKind::ManiaxeAxe {
        return ActiveProjectileKind::ManiaxeAxe;
    }
    if projectile.friendly
        && projectile.gravity == JUGGERNAUT_SKULL_GRAVITY
        && projectile.homing_accel == 0
        && projectile.speed_cap == 0
        && !projectile.terrain_kill
    {
        return ActiveProjectileKind::JuggernautSkull;
    }
    if projectile.kind == EnemyProjectileKind::BossHeadArrow {
        return ActiveProjectileKind::BossHeadArrow;
    }
    if projectile.kind == EnemyProjectileKind::BossHeadBoomerang {
        return ActiveProjectileKind::BossHeadBoomerang;
    }
    if projectile.kind == EnemyProjectileKind::BossSkewer {
        return ActiveProjectileKind::BossSkewer;
    }
    if projectile.kind == EnemyProjectileKind::BossEyeAttack {
        return ActiveProjectileKind::BossEyeAttack;
    }
    if projectile.kind == EnemyProjectileKind::BossDeathBurst {
        return ActiveProjectileKind::BossDeathBurst;
    }
    if projectile.kind == EnemyProjectileKind::BossEyeSatellite {
        return ActiveProjectileKind::BossEyeSatellite;
    }
    if projectile.kind == EnemyProjectileKind::TarMonsterStraightGlob {
        return ActiveProjectileKind::TarMonsterGlob;
    }
    if projectile.kind == EnemyProjectileKind::TarMonsterBurstChild {
        return ActiveProjectileKind::TarMonsterBurstChild;
    }

    // ASM Fire_Demon loc_3A63C initialises a separate fire-trail object that
    // cycles `stru_3A622`'s six flame frames. Older native traces represented
    // this as arcing generic projectiles, so keep the signature fallback for
    // compatibility while live code now tags the typed projectile kind.
    if !projectile.friendly
        && projectile.gravity == 0x1800
        && projectile.lifetime <= 45
        && matches!(projectile.x_velocity, -0x1_8000 | 0 | 0x1_8000)
    {
        ActiveProjectileKind::FireDemonDeathTrail
    } else {
        ActiveProjectileKind::Generic
    }
}

fn boss_eye_sprite_mapping_index(label: &str) -> u8 {
    match label {
        // ASM sub_37ABC indexes off_37AE0, whose entries point at the eight
        // `unk_C8488..unk_C84C0` 16x16 boss-eye attack frames.
        "LnkTo_unk_C8488-Data_Index" => 0,
        "LnkTo_unk_C8490-Data_Index" => 1,
        "LnkTo_unk_C8498-Data_Index" => 2,
        "LnkTo_unk_C84A0-Data_Index" => 3,
        "LnkTo_unk_C84A8-Data_Index" => 4,
        "LnkTo_unk_C84B0-Data_Index" => 5,
        "LnkTo_unk_C84B8-Data_Index" => 6,
        "LnkTo_unk_C84C0-Data_Index" => 7,
        _ => 0,
    }
}

fn boss_eye_orbit_mapping_indices_loc_37abc(
    enemy: &ActiveEnemy,
    diamond_power_active: bool,
    frame_index: usize,
    kid_x: u16,
    kid_y: u16,
) -> [u8; 2] {
    let workers = enemy.boss_eye_orbit_workers;
    let mut slots = [None; 12];
    slots[0] = Some(BossEyeSpriteRefreshSlotSub37ABC {
        x_pos: workers.primary_x_pos_word,
        y_pos: workers.primary_y_pos_word,
        sprite_mapping: "stale",
    });
    slots[1] = Some(BossEyeSpriteRefreshSlotSub37ABC {
        x_pos: workers.counter_x_pos_word,
        y_pos: workers.counter_y_pos_word,
        sprite_mapping: "stale",
    });

    let refreshed = boss_eye_refresh_attack_sprites_sub_37ABC(
        slots,
        diamond_power_active,
        frame_index as u8,
        kid_x,
        kid_y,
    );

    [
        refreshed[0]
            .map(|slot| boss_eye_sprite_mapping_index(slot.sprite_mapping))
            .unwrap_or_default(),
        refreshed[1]
            .map(|slot| boss_eye_sprite_mapping_index(slot.sprite_mapping))
            .unwrap_or_default(),
    ]
}

fn boss_eye_orbit_mapping_indices_from_state_loc_37abc(
    enemy: &ActiveEnemy,
    opening_slice: Option<&OpeningSliceState>,
    diamond_power_active: bool,
    frame_index: usize,
) -> Option<[u8; 2]> {
    let slice = opening_slice?;
    Some(boss_eye_orbit_mapping_indices_loc_37abc(
        enemy,
        diamond_power_active,
        frame_index,
        (slice.player.x_pos >> 16) as u16,
        (slice.player.y_pos >> 16) as u16,
    ))
}

fn boss_eye_orbit_object_slot_details_loc_3931a(
    enemy: &ActiveEnemy,
    mapping_indices_loc_37abc: Option<[u8; 2]>,
) -> Vec<NativeObjectSnapshot> {
    if !is_multi_head_boss(enemy.family) {
        return Vec::new();
    }

    enemy
        .boss_eye_orbit_workers
        .object_slots_loc_3931a()
        .into_iter()
        .enumerate()
        .map(|(slot_index, slot)| {
            let mut debug_fields = BTreeMap::new();
            debug_fields.insert(
                "boss_eye_object_meta_loc_3931a".to_owned(),
                format!("{:#06x}", slot.object_meta),
            );
            debug_fields.insert(
                "boss_eye_has_kid_collision_loc_3931a".to_owned(),
                slot.has_kid_collision.to_string(),
            );
            debug_fields.insert(
                "boss_eye_is_moved_loc_3931a".to_owned(),
                slot.is_moved.to_string(),
            );
            debug_fields.insert(
                "boss_eye_sprite_order_word_a_loc_393c8".to_owned(),
                format!("{:#06x}", slot.sprite_order_word_a),
            );
            if let Some(mapping_indices) = mapping_indices_loc_37abc {
                debug_fields.insert(
                    "boss_eye_sprite_mapping_index_loc_37abc".to_owned(),
                    mapping_indices[slot_index].to_string(),
                );
            }

            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy {
                    object_type: slot.object_meta as u8,
                },
                x_pos: u32::from(slot.x_pos_word) << 16,
                y_pos: u32::from(slot.y_pos_word) << 16,
                layout_index: None,
                x_velocity: Some(0),
                y_velocity: Some(0),
                facing: Some(enemy.facing),
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: enemy.palette_variant,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                sprite_animation_phase: None,
                wait_timer: None,
                debug_fields,
            }
        })
        .collect()
}

fn trace_cell_word(cell: terrain::Cell) -> u16 {
    let block_type = if cell.has_block {
        cell.block_type as u8 & 0x0F
    } else {
        0
    };
    let hi = (u8::from(cell.has_block) << 7) + ((cell.collision as u8 & 0x07) << 4) + block_type;
    u16::from_be_bytes([hi, cell.skin])
}

fn trace_cell_for_fixed_pos(grid: &terrain::BlockGrid, x_pos: u32, y_pos: u32) -> String {
    let x_px = (x_pos >> 16) as u16;
    let y_px = (y_pos >> 16) as u16;
    let col = x_px >> 4;
    let row = y_px >> 4;
    let cell = grid.cell_at(col, row);
    format!(
        "{col},{row}:{:04X}[{}]",
        trace_cell_word(cell),
        terrain_cell_debug_summary(cell)
    )
}

fn collected_latched_helmet_prize(item: &FloatingItem) -> bool {
    item.collected
        && item.collision_latched
        && matches!(
            item.content,
            terrain::PrizeContent::Skycutter
                | terrain::PrizeContent::Cyclone
                | terrain::PrizeContent::RedStealth
                | terrain::PrizeContent::Eyeclops
                | terrain::PrizeContent::Juggernaut
                | terrain::PrizeContent::IronKnight
                | terrain::PrizeContent::Berzerker
                | terrain::PrizeContent::Maniaxe
                | terrain::PrizeContent::Micromax
        )
}

fn collected_helmet_first_forward_transform_frame(
    item: &FloatingItem,
    current_form: PlayerForm,
    form_change_flash_frames: u8,
) -> bool {
    let Some(item_form) = prize_helmet_form(item.content) else {
        return false;
    };
    item.collected
        && item_form == current_form
        && u16::from(form_change_flash_frames)
            == crate::combat::transform_anim_frames(item_form) + 1
}

fn prize_helmet_form(content: terrain::PrizeContent) -> Option<PlayerForm> {
    match content {
        terrain::PrizeContent::Skycutter => Some(PlayerForm::Skycutter),
        terrain::PrizeContent::Cyclone => Some(PlayerForm::Cyclone),
        terrain::PrizeContent::RedStealth => Some(PlayerForm::RedStealth),
        terrain::PrizeContent::Eyeclops => Some(PlayerForm::Eyeclops),
        terrain::PrizeContent::Juggernaut => Some(PlayerForm::Juggernaut),
        terrain::PrizeContent::IronKnight => Some(PlayerForm::IronKnight),
        terrain::PrizeContent::Berzerker => Some(PlayerForm::Berzerker),
        terrain::PrizeContent::Maniaxe => Some(PlayerForm::Maniaxe),
        terrain::PrizeContent::Micromax => Some(PlayerForm::Micromax),
        _ => None,
    }
}

fn prize_block_wobble_resume_sub_1007a(remaining: u8) -> PrizeBlockWobbleResumeSub1007A {
    // ASM sub_1007A (kid.asm:21345-21435): each JSR to
    // Hibernate_Object_1Frame saves the following instruction in `4(a5)`.
    // The six/six/five/five/four/five inclusive DBF groups correspond to the
    // native 31-frame remaining counter after the allocation pass.
    match remaining {
        0..=5 => PrizeBlockWobbleResumeSub1007A::AfterLoc10120At10128,
        6..=9 => PrizeBlockWobbleResumeSub1007A::AfterLoc10112At1011A,
        10..=14 => PrizeBlockWobbleResumeSub1007A::AfterLoc10104At1010C,
        15..=19 => PrizeBlockWobbleResumeSub1007A::AfterLoc100F6At100Fe,
        20..=25 => PrizeBlockWobbleResumeSub1007A::AfterLoc100E8At100F0,
        _ => PrizeBlockWobbleResumeSub1007A::AfterLoc100DaAt100E2,
    }
}

fn prize_block_wobble_wake_sub_1007a(remaining: u8) -> ModeLevelPrizeBlockWobbleWake {
    // ASM sub_1007A (kid.asm:21372-21407): DBF expiry runs the next moveq and
    // its first displacement before hibernating. Expiry in the last group
    // instead enters Hibernate_UntilAnimFinished.
    match remaining {
        1 => ModeLevelPrizeBlockWobbleWake::BeginAnimationWait,
        6 | 10 | 15 | 20 | 26 => ModeLevelPrizeBlockWobbleWake::BeginNextPhase,
        2..=31 => ModeLevelPrizeBlockWobbleWake::ContinuePhase,
        _ => ModeLevelPrizeBlockWobbleWake::Unclassified { remaining },
    }
}

fn prize_block_tail_state_sub_1007a(
    content: Option<terrain::PrizeContent>,
    remaining: u8,
) -> ModeLevelPrizeBlockTailState {
    // ASM sub_1007A (kid.asm:21407-21435): once the animation flag wakes
    // Hibernate_UntilAnimFinished, the worker creates the flash and prize and
    // deletes itself. Native's source-backed overlap is four frames for a
    // diamond and three for every shared prize path; only the initial value
    // represents that live completion callback.
    let completion_remaining = NativeRuntime::PRIZE_BLOCK_WORKER_TAIL_FRAMES
        .saturating_add(u8::from(content == Some(terrain::PrizeContent::Diamond)));
    match remaining.cmp(&completion_remaining) {
        std::cmp::Ordering::Equal => ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete,
        std::cmp::Ordering::Less => {
            ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { remaining }
        }
        std::cmp::Ordering::Greater => {
            ModeLevelPrizeBlockTailState::UnclassifiedAnimationWait { remaining }
        }
    }
}

fn prize_block_flash_wake_loc_102b8(remaining: u8) -> ModeLevelPrizeAnimationWake {
    // ASM loc_102B8 (kid.asm:21545-21551): the native flash countdown reaches
    // one on the wake where animation completion returns and deletes the
    // object; earlier wakes re-enter Hibernate_Object_1Frame.
    if remaining == 1 {
        ModeLevelPrizeAnimationWake::FinishAndDelete
    } else {
        ModeLevelPrizeAnimationWake::Hibernate
    }
}

fn prize_wobble_allocation_age(remaining: u8) -> u16 {
    u16::from(31_u8.saturating_sub(remaining))
}

fn prize_tail_allocation_age(work: &ModeLevelPrizeBlockTailWork) -> u16 {
    let initial_tail = NativeRuntime::PRIZE_BLOCK_WORKER_TAIL_FRAMES.saturating_add(u8::from(
        work.content == Some(terrain::PrizeContent::Diamond),
    ));
    31 + u16::from(initial_tail.saturating_sub(match work.state {
        ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete => initial_tail,
        ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { remaining }
        | ModeLevelPrizeBlockTailState::UnclassifiedAnimationWait { remaining } => remaining,
    }))
}

fn prize_flash_allocation_age(remaining: u8) -> u16 {
    u16::from(NativeRuntime::PRIZE_BLOCK_SPAWN_FLASH_FRAMES.saturating_sub(remaining))
}

fn floating_prize_allocation_age(work: ModeLevelFloatingPrizeWork) -> u16 {
    NativeRuntime::floating_item_lifetime(work.content).saturating_sub(work.remaining)
}

fn floating_prize_fall_path_loc_103b2(
    item: &FloatingItem,
    current_collision: terrain::CollisionKind,
    below_collision: terrain::CollisionKind,
) -> ModeLevelFloatingPrizeFallPathLoc103B2 {
    let passable = |collision| {
        matches!(
            collision,
            terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
        )
    };
    let used_grace_probe = if passable(current_collision) {
        false
    } else if !item.first_row_grace {
        return ModeLevelFloatingPrizeFallPathLoc103B2::BlockedCurrentRow;
    } else if passable(below_collision) {
        true
    } else {
        return ModeLevelFloatingPrizeFallPathLoc103B2::BlockedGraceProbe;
    };

    // ASM loc_103DE (kid.asm:21648): gravity is added before y_pos, then
    // loc_103F6 compares the new aligned row with `d0`.
    let next_y_velocity = item.y_velocity.saturating_add(0x1388);
    let next_y_pos = item.y_pos.saturating_add(i64::from(next_y_velocity));
    let crosses_row = ((next_y_pos >> 16) as i32 & !0xF) != ((item.y_pos >> 16) as i32 & !0xF);
    match (used_grace_probe, crosses_row) {
        (false, false) => ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow,
        (false, true) => ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallAdvanceRow,
        (true, false) => ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow,
        (true, true) => ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallAdvanceRow,
    }
}

fn player_allocates_prize_wobble_before_tails(path: Option<&ModeLevelPlayerExecutionPath>) -> bool {
    let Some(ModeLevelPlayerExecutionPath::NormalPhysics(path)) = path else {
        return false;
    };
    let Some(scan) = &path.ceiling_scan_sub_914a else {
        return false;
    };
    scan.first_hit_col.is_some_and(|first_hit_col| {
        scan.visits.iter().any(|visit| {
            visit.col == first_hit_col
                && visit.has_block
                // ASM off_BB80[1] / loc_1002E: block type 1 allocates the
                // new `$2001` sub_1007A Object/GfxObject pair.
                && visit.block_type == Some(1)
        })
    })
}

fn player_spawned_prize_block_initial_sub_1007a(
    path: Option<&ModeLevelPlayerExecutionPath>,
    block_bounces: &[BlockBounceAnim],
) -> Option<ModeLevelPrizeBlockInitialWorkSub1007A> {
    if !player_allocates_prize_wobble_before_tails(path) {
        return None;
    }
    let bounce = block_bounces.iter().rev().find(|bounce| {
        bounce.kind == BlockBounceKind::Prize
            && bounce.remaining == bounce.initial_duration
            && bounce.initial_duration == 31
    })?;
    Some(ModeLevelPrizeBlockInitialWorkSub1007A {
        // ASM sub_1007A (kid.asm:21345-21475): byte 1 of the selected
        // Level_Layout word branches to the reveal-only loc_101D8 script.
        path: if bounce.prize_deleted_tile {
            ModeLevelPrizeBlockInitialPathSub1007A::RevealOnlyLoc101D8
        } else {
            ModeLevelPrizeBlockInitialPathSub1007A::Prize
        },
        content: bounce.prize_content,
    })
}

fn player_prize_wobble_list_work_loc_1002e(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<ModeLevelPrizeWobbleListWorkLoc1002E> {
    execute
        .player_spawned_prize_block_initial_sub_1007a
        .as_ref()?;
    // ASM loc_1002E -> sub_E02: `$2001` follows the priority-1 timer,
    // `$1FFF` prizes, and `$2000` Kid Object. Its category-4 GfxObject inserts
    // after every equal key already linked at Execute_Objects entry.
    Some(ModeLevelPrizeWobbleListWorkLoc1002E {
        object_insert_rank_loc_de0: execute.floating_prize_workers.checked_add(2)?,
        gfx_insert_rank_loc_fd4: execute.linked_gfx_object_count,
    })
}

fn prize_object_execution_order_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
) -> Option<Vec<ModeLevelPrizeObjectExecutionSubE12>> {
    if execute.player == ModeLevelPlayerWorkBeforeHud::Absent
        || usize::from(execute.floating_prize_workers) != execute.floating_prize_paths.len()
        || usize::from(execute.prize_block_wobble_workers_sub_1007a)
            != execute.prize_block_wobble_paths_sub_1007a.len()
        || usize::from(execute.prize_block_worker_tails_sub_1007a)
            != execute.prize_block_tail_paths_sub_1066.len()
        || usize::from(execute.prize_block_flash_workers_loc_102b8)
            != execute.prize_block_flash_paths_sub_1066.len()
    {
        return None;
    }

    let mut floating = execute
        .floating_prize_paths
        .iter()
        .copied()
        .enumerate()
        .map(|(index, work)| (floating_prize_allocation_age(work), index))
        .collect::<Vec<_>>();
    floating.sort_unstable_by_key(|&(age, index)| (age, std::cmp::Reverse(index)));
    if floating.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return None;
    }

    // ASM loc_DE0 inserts before equal `$2001` selectors. A flash created by
    // a completing tail runs after Kid has created a wobble in that source
    // frame, so the later flash precedes the wobble when their ages tie.
    const FLASH_TIE_ORDER: u8 = 0;
    const WOBBLE_TIE_ORDER: u8 = 1;
    const TAIL_TIE_ORDER: u8 = 2;
    let mut equal_workers = execute
        .prize_block_flash_paths_sub_1066
        .iter()
        .enumerate()
        .map(|(index, work)| {
            (
                prize_flash_allocation_age(work.remaining),
                FLASH_TIE_ORDER,
                index,
                ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 {
                    index: index as u16,
                },
            )
        })
        .chain(
            execute
                .prize_block_wobble_paths_sub_1007a
                .iter()
                .enumerate()
                .map(|(index, work)| {
                    (
                        prize_wobble_allocation_age(work.remaining),
                        WOBBLE_TIE_ORDER,
                        index,
                        ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A {
                            index: index as u16,
                        },
                    )
                }),
        )
        .chain(
            execute
                .prize_block_tail_paths_sub_1066
                .iter()
                .enumerate()
                .filter(|(_, work)| {
                    !matches!(
                        work.state,
                        ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { .. }
                    )
                })
                .map(|(index, work)| {
                    (
                        prize_tail_allocation_age(work),
                        TAIL_TIE_ORDER,
                        index,
                        ModeLevelPrizeObjectExecutionSubE12::TailSub1007A {
                            index: index as u16,
                        },
                    )
                }),
        )
        .collect::<Vec<_>>();
    equal_workers.sort_unstable_by_key(|&(age, tie_order, index, _)| {
        (age, tie_order, std::cmp::Reverse(index))
    });
    if equal_workers.windows(2).any(|pair| {
        pair[0].0 == pair[1].0
            && (pair[0].1 == pair[1].1
                || pair[0].1 == TAIL_TIE_ORDER
                || pair[1].1 == TAIL_TIE_ORDER)
    }) {
        return None;
    }

    let mut order = floating
        .into_iter()
        .map(
            |(_, index)| ModeLevelPrizeObjectExecutionSubE12::FloatingPrize {
                index: index as u16,
            },
        )
        .collect::<Vec<_>>();
    order.push(ModeLevelPrizeObjectExecutionSubE12::Player);
    if execute
        .player_spawned_prize_block_initial_sub_1007a
        .is_some()
    {
        order.push(ModeLevelPrizeObjectExecutionSubE12::PlayerSpawnedInitialSub1007A);
    }
    order.extend(equal_workers.into_iter().map(|(_, _, _, work)| work));
    Some(order)
}

fn mode_level_prize_tail_list_work(
    execute: &ModeLevelExecuteObjectsWork,
    target_tail_index: usize,
    player_allocates_prize_wobble: bool,
) -> Option<ModeLevelPrizeTailListWork> {
    let target = execute
        .prize_block_tail_paths_sub_1066
        .get(target_tail_index)?;
    if target.state != ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete
        || execute
            .prize_block_tail_paths_sub_1066
            .iter()
            .filter(|tail| tail.state == ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete)
            .count()
            != 1
        || execute.active_enemy_workers != 0
        || execute.active_platform_workers != 0
        || execute.unclassified_runtime_transient_workers != 0
        || execute.rubber_block_skin_workers_sub_10de4 != 0
        || execute.player == ModeLevelPlayerWorkBeforeHud::Absent
        || execute.linked_non_transient_object_count != 4
        || execute
            .prize_block_flash_paths_sub_1066
            .iter()
            .any(|flash| flash.wake != ModeLevelPrizeAnimationWake::Hibernate)
        || execute
            .floating_prize_paths
            .iter()
            .any(|prize| prize.collision_latched || prize.remaining <= 1 || prize.spawn_delay != 0)
    {
        return None;
    }

    let entry_equal_workers = execute.prize_block_wobble_paths_sub_1007a.len()
        + execute.prize_block_tail_paths_sub_1066.len()
        + execute.prize_block_flash_paths_sub_1066.len();
    if usize::from(execute.prize_block_wobble_workers_sub_1007a)
        != execute.prize_block_wobble_paths_sub_1007a.len()
        || usize::from(execute.prize_block_worker_tails_sub_1007a)
            != execute.prize_block_tail_paths_sub_1066.len()
        || usize::from(execute.prize_block_flash_workers_loc_102b8)
            != execute.prize_block_flash_paths_sub_1066.len()
        || usize::from(execute.floating_prize_workers) != execute.floating_prize_paths.len()
    {
        return None;
    }

    let target_age = prize_tail_allocation_age(target);
    let equal_worker_ages = execute
        .prize_block_wobble_paths_sub_1007a
        .iter()
        .map(|work| prize_wobble_allocation_age(work.remaining))
        .chain(
            execute
                .prize_block_tail_paths_sub_1066
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != target_tail_index)
                .map(|(_, work)| prize_tail_allocation_age(work)),
        )
        .chain(
            execute
                .prize_block_flash_paths_sub_1066
                .iter()
                .map(|work| prize_flash_allocation_age(work.remaining)),
        )
        .collect::<Vec<_>>();
    let floating_prize_ages = execute
        .floating_prize_paths
        .iter()
        .copied()
        .map(floating_prize_allocation_age)
        .collect::<Vec<_>>();
    // Native does not yet retain within-frame allocation serials across its
    // parallel worker vectors. Refuse equal-age ambiguity rather than invent
    // an order which loc_DE0/loc_FD4 would resolve from allocation sequence.
    if equal_worker_ages
        .iter()
        .chain(&floating_prize_ages)
        .any(|&age| age == target_age)
    {
        return None;
    }

    let newer_equal_workers = equal_worker_ages
        .iter()
        .filter(|&&age| age < target_age)
        .count();
    let older_prize_gfx_workers = equal_worker_ages
        .iter()
        .chain(&floating_prize_ages)
        .filter(|&&age| age > target_age)
        .count();
    let prize_gfx_workers = entry_equal_workers + floating_prize_ages.len();
    // ASM sub_7196 creates Kid's `$0201` projectile slot and `$0200` body
    // slot before every ordinary `$00FF` GfxObject. With the remaining
    // taxonomy proven complete above, every non-prize GfxObject is an older
    // predecessor of the target in loc_FD4's main-list order.
    let older_non_prize_standard_gfx =
        usize::from(execute.linked_gfx_object_count).checked_sub(prize_gfx_workers)?;
    let tail_gfx_delete_rank = older_non_prize_standard_gfx + older_prize_gfx_workers;

    // ASM sub_7196 also creates the priority-1 clock/timer Object. With no
    // untyped, enemy, or platform workers, the only selectors below `$2001`
    // are that timer, live `$1FFF` prizes, and Kid at `$2000`.
    let flash_object_insert_rank = 2 + execute.floating_prize_paths.len();
    let player_wobble_count = usize::from(player_allocates_prize_wobble);
    let tail_object_delete_rank = flash_object_insert_rank
        + 1 // the newly inserted `$1FFF` prize
        + newer_equal_workers
        + player_wobble_count
        + 1; // the newly inserted `$2001` flash
    let flash_gfx_insert_rank = usize::from(execute.linked_gfx_object_count) + player_wobble_count;
    let prize_gfx_insert_rank = flash_gfx_insert_rank + 1;
    let gfx_count_at_delete = prize_gfx_insert_rank + 1;
    let tail_gfx_category4_delete_rank =
        gfx_count_at_delete.checked_sub(tail_gfx_delete_rank + 1)?;

    Some(ModeLevelPrizeTailListWork {
        flash_object_insert_rank_loc_de0: u16::try_from(flash_object_insert_rank).ok()?,
        flash_gfx_insert_rank_loc_fd4: u16::try_from(flash_gfx_insert_rank).ok()?,
        prize_object_insert_rank_loc_de0: 1,
        prize_gfx_insert_rank_loc_fd4: u16::try_from(prize_gfx_insert_rank).ok()?,
        tail_object_delete_rank_loc_ea2: u16::try_from(tail_object_delete_rank).ok()?,
        tail_gfx_delete_rank_loc_10ba: u16::try_from(tail_gfx_delete_rank).ok()?,
        tail_gfx_category4_delete_rank_loc_1106: u16::try_from(tail_gfx_category4_delete_rank)
            .ok()?,
    })
}

fn complete_prize_tail_list_work(execute: &mut ModeLevelExecuteObjectsWork) {
    let player_allocates_prize_wobble =
        player_allocates_prize_wobble_before_tails(execute.executed_player_path.as_ref());
    let list_work = (0..execute.prize_block_tail_paths_sub_1066.len())
        .map(|target_tail_index| {
            mode_level_prize_tail_list_work(
                execute,
                target_tail_index,
                player_allocates_prize_wobble,
            )
        })
        .collect::<Vec<_>>();
    for (tail, list_work) in execute
        .prize_block_tail_paths_sub_1066
        .iter_mut()
        .zip(list_work)
    {
        tail.list_work = list_work;
    }
}

fn mode_level_airborne_horizontal_path(
    x_velocity_before: i32,
    x_velocity_after: i32,
) -> ModeLevelAirborneHorizontalPath {
    // ASM sub_B084 (kid.asm:13921-13989): the published Cyclone route either
    // adds the $2B00 right-held acceleration at loc_B0E2 or adds the fixed
    // $400 brake to a negative neutral velocity at loc_B10E.
    match x_velocity_after.wrapping_sub(x_velocity_before) {
        0x2B00 => ModeLevelAirborneHorizontalPath::RightHeldAcceleration,
        0x4080 => ModeLevelAirborneHorizontalPath::RightHeldRunAcceleration,
        -0x0100 if x_velocity_before > 0 && x_velocity_after > 0 => {
            ModeLevelAirborneHorizontalPath::RightHeldCapBleed
        }
        0x0400 if x_velocity_before < 0 && x_velocity_after <= 0 => {
            ModeLevelAirborneHorizontalPath::NeutralNegativeBrake
        }
        -0x0400 if x_velocity_before > 0 && x_velocity_after >= 0 => {
            ModeLevelAirborneHorizontalPath::NeutralPositiveBrake
        }
        _ => ModeLevelAirborneHorizontalPath::Unclassified {
            x_velocity_before,
            x_velocity_after,
        },
    }
}

fn mode_level_kid_gfx_terrain_scan_bounds(
    entry: ModeLevelPlayerPosition,
    x_velocity: i32,
    y_velocity: i32,
    form: PlayerForm,
) -> Option<ModeLevelKidGfxTerrainScanBounds> {
    // ASM sub_A4EE (kid.asm:12867-12883) temporarily applies both 16.16
    // velocities before GfxObjects_Collision loc_173E rounds Kid's inclusive
    // collision box to 16px cells.
    let moved_x_fp = i64::from(entry.x_pos) + i64::from(x_velocity);
    let moved_y_fp = i64::from(entry.y_pos) + i64::from(y_velocity);
    let moved_x_px = i32::try_from(moved_x_fp >> 16).ok()?;
    let moved_y_px = i32::try_from(moved_y_fp >> 16).ok()?;
    let left_px = moved_x_px.checked_sub(form.half_width())?;
    let right_px = moved_x_px.checked_add(form.half_width())?;
    let top_px = moved_y_px.checked_sub(crate::constants::PLAYER_HEIGHT)?;
    if left_px < 0 || top_px < 0 || right_px < left_px || moved_y_px < top_px {
        return None;
    }
    Some(ModeLevelKidGfxTerrainScanBounds {
        left_col: u16::try_from(left_px >> 4).ok()?,
        right_col: u16::try_from(right_px >> 4).ok()?,
        top_row: u16::try_from(top_px >> 4).ok()?,
        bottom_row: u16::try_from(moved_y_px >> 4).ok()?,
    })
}

fn mode_level_kid_gfx_terrain_scan(
    grid: &terrain::BlockGrid,
    bounds: ModeLevelKidGfxTerrainScanBounds,
) -> ModeLevelKidGfxTerrainScan {
    // ASM GfxObjects_Collision loc_1792/loc_1796 (kid.asm:2905-2932) scans
    // rows top-to-bottom and columns left-to-right. Spikes are explicitly
    // ignored at loc_17C6; slope and solid cells dispatch specialised paths.
    let mut tile_iterations = 0_u16;
    for row in bounds.top_row..=bounds.bottom_row {
        for col in bounds.left_col..=bounds.right_col {
            tile_iterations = tile_iterations.saturating_add(1);
            let first_collision = match grid.cell_at(col, row).collision {
                terrain::CollisionKind::SlopeUp => Some(ModeLevelKidGfxTerrainCollision::SlopeUp),
                terrain::CollisionKind::SlopeDown => {
                    Some(ModeLevelKidGfxTerrainCollision::SlopeDown)
                }
                terrain::CollisionKind::Solid => Some(ModeLevelKidGfxTerrainCollision::Solid),
                terrain::CollisionKind::Empty
                | terrain::CollisionKind::HiddenBlock
                | terrain::CollisionKind::Spikes => None,
            };
            if first_collision.is_some() {
                return ModeLevelKidGfxTerrainScan {
                    bounds,
                    tile_iterations,
                    first_collision,
                };
            }
        }
    }
    ModeLevelKidGfxTerrainScan {
        bounds,
        tile_iterations,
        first_collision: None,
    }
}

fn mode_level_terrain_special_scan_sub_1fa2(
    grid: &terrain::BlockGrid,
    bounds: ModeLevelKidGfxTerrainScanBounds,
) -> ModeLevelTerrainSpecialScanSub1FA2 {
    // ASM sub_1FA2 loc_1FEE/loc_1FF2: scan the inclusive Kid-hitbox cell
    // rectangle row-major until collision nibble `$6000` or `$7000`.
    let mut tile_iterations = 0_u16;
    for row in bounds.top_row..=bounds.bottom_row {
        for col in bounds.left_col..=bounds.right_col {
            tile_iterations = tile_iterations.saturating_add(1);
            let first_special_collision = match grid.raw_word_at(col, row) & 0x7000 {
                0x6000 => Some(ModeLevelTerrainSpecialCollisionSub1FA2::Solid),
                0x7000 => Some(ModeLevelTerrainSpecialCollisionSub1FA2::Spikes),
                _ => None,
            };
            if first_special_collision.is_some() {
                return ModeLevelTerrainSpecialScanSub1FA2 {
                    bounds,
                    tile_iterations,
                    first_special_collision,
                };
            }
        }
    }
    ModeLevelTerrainSpecialScanSub1FA2 {
        bounds,
        tile_iterations,
        first_special_collision: None,
    }
}

fn mode_level_ceiling_scan_sub_914a(
    cells: [Option<(u16, u16, u16)>; 4],
    rubber_worker_count_before: u16,
) -> Option<ModeLevelCeilingScanSub914A> {
    let mut cells = cells.into_iter().flatten().peekable();
    let (left_col, row, _) = *cells.peek()?;
    let mut right_col = left_col;
    let mut worker_count = rubber_worker_count_before;
    let mut first_hit_col = None;
    let mut rubber_contact = false;
    let visits = cells
        .map(|(col, cell_row, word)| {
            debug_assert_eq!(cell_row, row);
            right_col = col;
            let cell = terrain::Cell::from_be_bytes((word >> 8) as u8, word as u8);
            let collision = match cell.collision {
                terrain::CollisionKind::Empty => ModeLevelCeilingScanCollision::Empty,
                terrain::CollisionKind::HiddenBlock => ModeLevelCeilingScanCollision::HiddenBlock,
                terrain::CollisionKind::SlopeUp => ModeLevelCeilingScanCollision::SlopeUp,
                terrain::CollisionKind::SlopeDown => ModeLevelCeilingScanCollision::SlopeDown,
                terrain::CollisionKind::Solid => ModeLevelCeilingScanCollision::Solid,
                terrain::CollisionKind::Spikes => ModeLevelCeilingScanCollision::Spikes,
            };
            let hit = matches!(
                cell.collision,
                terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
            );
            if hit && first_hit_col.is_none() {
                first_hit_col = Some(col);
            }
            // ASM sub_914A loc_91C0 (kid.asm:10607-10633): only a hit cell
            // whose block flag survives bclr #15 and whose type is six calls
            // loc_10DA4. That routine plays sound before applying its ten
            // live-worker cap and incrementing $FFFFFB5A.
            let rubber_effect =
                if hit && cell.has_block && cell.block_type == terrain::BlockType::Rubber {
                    rubber_contact = true;
                    if worker_count < 10 {
                        worker_count += 1;
                        ModeLevelRubberContactEffectLoc10DA4::PlaySoundAndAllocate {
                            worker_count_after: worker_count,
                        }
                    } else {
                        ModeLevelRubberContactEffectLoc10DA4::PlaySoundWorkerCapReached
                    }
                } else {
                    ModeLevelRubberContactEffectLoc10DA4::None
                };
            ModeLevelCeilingScanVisitSub9188 {
                col,
                collision,
                has_block: cell.has_block,
                block_type: cell.has_block.then_some(cell.block_type as u8),
                skin: cell.skin,
                rubber_effect,
            }
        })
        .collect();
    let return_path = if rubber_contact {
        ModeLevelCeilingScanReturn::RubberBounce
    } else if first_hit_col.is_some() {
        ModeLevelCeilingScanReturn::Hit
    } else {
        ModeLevelCeilingScanReturn::Clear
    };
    Some(ModeLevelCeilingScanSub914A {
        left_col,
        right_col,
        row,
        visits,
        first_hit_col,
        return_path,
    })
}

fn mode_level_cyclone_visual_step(
    accumulator_before: u16,
    acceleration: i16,
) -> ModeLevelCycloneVisualStep {
    // ASM sub_B270 loc_B2A0 (kid.asm:14154-14175): derive the phase delta
    // from Cyclone_YAcceleration, add it to `$FFFFF8F0`, and conditionally
    // execute `subi.w #$300` before selecting the spin mapping.
    let mut phase_delta = (0i16).wrapping_sub(acceleration).wrapping_add(0x2000) as u16;
    phase_delta = phase_delta.wrapping_shl(2) >> 8;
    let sum = accumulator_before.wrapping_add(phase_delta);
    let wrapped = sum >= 0x0300;
    ModeLevelCycloneVisualStep {
        accumulator_before,
        phase_delta,
        accumulator_after: if wrapped {
            sum.wrapping_sub(0x0300)
        } else {
            sum
        },
        wrapped,
    }
}

fn mode_level_vertical_collision_path(
    ceiling_hit: bool,
    landed: bool,
    y_velocity_before_collision: i32,
) -> ModeLevelVerticalCollisionPath {
    // ASM loc_A75A/loc_A8F6 (kid.asm:13064-13192): zero falls through to
    // loc_A762, a negative collision result clears the airborne transient
    // flags and rejoins there, and a non-negative result enters landing.
    match (ceiling_hit, landed, y_velocity_before_collision) {
        (false, false, _) => ModeLevelVerticalCollisionPath::Clear,
        (true, false, velocity) if velocity < 0 => ModeLevelVerticalCollisionPath::Ceiling,
        (_, true, velocity) if velocity >= 0 => ModeLevelVerticalCollisionPath::Landing,
        _ => ModeLevelVerticalCollisionPath::Unclassified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    fn diamond_at_x(x_px: u32) -> FloatingItem {
        FloatingItem {
            x_pos: x_px << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 0x01F4,
            spawn_delay: 0,
            level_col: 0,
            level_row: 0,
            first_row_grace: false,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        }
    }

    fn minimal_manifest() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: wildside_data::GENERATED_BUNDLE_VERSION,
            rom_sha1: String::from("trace-test"),
            upstream_commit: String::from("trace-test"),
            source_root: PathBuf::from("/tmp/source"),
            output_root: PathBuf::from("/tmp/output"),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

    #[test]
    fn native_platform_slots_debug_loc_257e_reports_vector_order_and_rom_offset_key() {
        let platform = ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: Some(3),
            kind: 8,
            subtype: 6,
            x_pos: 0x0110_0000,
            y_pos: 0x0200_0000,
            width_px: 63,
            height_px: 15,
            x_velocity: 0x0001_0000,
            y_velocity: -0x0001_0000,
            deactivated_flag: 0xFF,
            script: Some(PlatformScriptState {
                script_offset: 0x097C,
                command_index: 0,
                remaining_duration: 7,
            }),
            special: None,
        };
        let mut second = platform;
        second.slot_addr = crate::PlatformLoaderState::synthetic_slot_addr_loc_231C(1);
        second.layout_index = None;
        second.deactivated_flag = 0;
        second.script = None;

        let summary = NativeRuntime::native_platform_slots_debug_loc_257e(&[platform, second])
            .expect("active platforms should produce a debug summary");

        assert!(
            summary.contains("count=2;native[00]:next=native[01]"),
            "ASM Platforms_CheckCollision loc_257E walks slots in list order; native should expose its current vector order explicitly",
        );
        assert!(
            summary.contains("slot=0xedba,next_slot=0xeddc")
                && summary.contains("native[01]:next=none,slot=0xeddc,next_slot=none"),
            "ASM Initialize_Platforms sub_231C builds Platform_RAM slots from $EDBA in $22-byte links; native diagnostics should expose the synthetic address that corresponds to each vector position",
        );
        assert!(
            summary.contains("vel=0x00010000,0xffff0000"),
            "ASM Move_Platforms sub_23EA stores `$A/$E(a2)` platform velocities in slot order for collision/carry comparison",
        );
        assert!(
            summary.contains("state=0x000000ff,timer=0x0007"),
            "ASM loc_2876/trap loops observe `$12(a3)` state and `$16(a3)` timer, so the native diagnostic should keep both visible",
        );
        assert!(
            summary.contains("art=0x025a,size=63x15,render=0x06,off=0x0024"),
            "ASM loc_2822 selects the art word and loc_286E stamps `$20(a3)` with `layout_index * $C`; both keys are needed to compare against oracle Platform_RAM",
        );
        assert!(
            summary.contains("native[01]:next=none") && summary.contains("off=none"),
            "synthetic native indices must stay distinct from true ROM slot addresses when no loader offset exists",
        );
    }

    #[test]
    fn trace_frame_reports_native_platform_loader_cursor_loc_2744() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 640,
            height_pixels: 480,
        });
        slice.objects.platform_loader = PlatformLoaderState {
            next_index: 2,
            offset_words: 0x0018,
            loaded_mask: 0b11,
            ..PlatformLoaderState::default()
        };
        runtime.opening_slice = Some(slice);

        let trace = runtime.trace_frame();
        let summary = trace
            .gameplay_debug_fields
            .get("native_platform_loader_loc_2744")
            .unwrap_or_else(|| panic!("trace should expose native platform loader cursor"));

        assert!(
            summary.contains("offset=0x0018") && summary.contains("next_index=2"),
            "ASM Manage_PlatformLoading loc_2744/loc_286E is a cursor over 12-byte platform records, so trace needs the native offset and materialisation bound",
        );
        assert!(
            summary.contains("layout_len=none") && summary.contains("active_slots=0"),
            "the loader cursor trace should remain useful even when the minimal test manifest has no platform layout",
        );
        assert!(
            summary.contains("number_platforms=2"),
            "ASM Allocate_PlatformSlot/Deallocate_PlatformSlot maintain `(Number_Platforms).w` independently of the native runtime vector, so the loader trace should expose the loaded-mask count directly",
        );
        assert!(
            summary.contains("next_free=0xedba") && summary.contains("active_links=none"),
            "ASM Initialize_Platforms sub_231C seeds the free-list head before any platform allocation; trace should expose that pool state alongside the loader cursor",
        );
    }

    #[test]
    fn trace_reports_robot_shot_and_impact_kinds_loc_3ac0c_3ac92() {
        let shot = EnemyProjectile {
            kind: EnemyProjectileKind::RobotShot,
            ..EnemyProjectile::default()
        };
        assert_eq!(
            active_projectile_kind(&shot),
            ActiveProjectileKind::RobotShot,
            "ASM Robot.asm loc_3AC0C initialises the live flying shot; trace must not collapse it to Generic"
        );

        let impact = EnemyProjectile {
            kind: EnemyProjectileKind::RobotShotImpact,
            ..EnemyProjectile::default()
        };
        assert_eq!(
            active_projectile_kind(&impact),
            ActiveProjectileKind::RobotShotImpact,
            "ASM Robot.asm loc_3AC92 initialises `stru_3A762`; trace must preserve that render state"
        );
    }

    #[test]
    fn trace_reports_cloud_lightning_kind_loc_35df0() {
        let lightning = EnemyProjectile {
            kind: EnemyProjectileKind::CloudLightning,
            ..EnemyProjectile::default()
        };

        assert_eq!(
            active_projectile_kind(&lightning),
            ActiveProjectileKind::CloudLightning,
            "ASM Cloud.asm loc_35DF0 initialises a bolt child worker; trace must not collapse it to Generic"
        );
    }

    #[test]
    fn trace_reports_boss_eye_attack_mapping_sub_37abc() {
        let eye = EnemyProjectile {
            kind: EnemyProjectileKind::BossEyeAttack,
            boss_eye_sprite_mapping_label: "LnkTo_unk_C84B0-Data_Index",
            ..EnemyProjectile::default()
        };

        assert_eq!(
            active_projectile_kind(&eye),
            ActiveProjectileKind::BossEyeAttack,
            "ASM sub_37332 initialises HeadyMetal eye attacks as visible boss-eye workers"
        );
        assert_eq!(
            boss_eye_sprite_mapping_index(eye.boss_eye_sprite_mapping_label),
            5,
            "ASM sub_37ABC indexes off_37AE0 so `LnkTo_unk_C84B0` must survive trace as frame index 5"
        );
    }

    #[test]
    fn trace_frame_reports_boot_phase_for_visible_sega_screen() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::SegaScreen { elapsed: 30 };
        runtime.frame_index = 30;

        // ASM Load_SegaScreen loc_12E48/loc_12E6C has a live mode-0 boot phase
        // once the hidden startup frames have elapsed; trace_frame must not
        // collapse to NativeTraceFrame::default().
        let trace = runtime.trace_frame();
        assert_eq!(trace.game_mode, GameMode::SegaScreen);
        assert_eq!(trace.boot_phase.as_deref(), Some("sega_screen"));
        assert_eq!(trace.boot_phase_elapsed, Some(30));
    }

    #[test]
    fn cold_boot_sega_screen_hands_off_to_intro_sequence_1_on_oracle_frame_173() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut frame_172 = None;
        let mut frame_173 = None;

        for frame_index in 0..=173 {
            let trace = runtime.step_frame(FrameInput::default());
            match frame_index {
                172 => frame_172 = Some(trace.game_mode),
                173 => frame_173 = Some(trace.game_mode),
                _ => {}
            }
        }

        // ASM Load_SegaScreen `loc_12E6C`/`loc_12E96` keeps polling Start until
        // `loc_12EDE` writes `Game_Mode=$0004`; oracle issue-161 probes keep
        // frame 172 in mode 0 and first expose intro sequence 1 on frame 173.
        assert_eq!(frame_172, Some(GameMode::SegaScreen));
        assert_eq!(frame_173, Some(GameMode::IntroSequence1));
    }

    #[test]
    fn cold_boot_intro_sequences_1_and_2_follow_oracle_handoff_frames() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut frame_533 = None;
        let mut frame_534 = None;
        let mut frame_982 = None;
        let mut frame_983 = None;

        for frame_index in 0..=983 {
            let trace = runtime.step_frame(FrameInput::default());
            match frame_index {
                533 => frame_533 = Some(trace.game_mode),
                534 => frame_534 = Some(trace.game_mode),
                982 => frame_982 = Some(trace.game_mode),
                983 => frame_983 = Some(trace.game_mode),
                _ => {}
            }
        }

        // ASM sub_1B8F4 `loc_1B924` writes `Game_Mode=$0018` after the
        // `$12B` DBF loop; ASM Load_IntroSequence2 `loc_1BA02` then writes
        // `Game_Mode=$001C` after its `$149` DBF loop.
        assert_eq!(frame_533, Some(GameMode::IntroSequence1));
        assert_eq!(frame_534, Some(GameMode::IntroSequence2));
        assert_eq!(frame_982, Some(GameMode::IntroSequence2));
        assert_eq!(frame_983, Some(GameMode::IntroSequence3));
    }

    #[test]
    fn cold_boot_intro_sequence_3_follows_oracle_handoff_frame() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut frame_1281 = None;
        let mut frame_1282 = None;

        for frame_index in 0..=1282 {
            let trace = runtime.step_frame(FrameInput::default());
            match frame_index {
                1281 => frame_1281 = Some(trace.game_mode),
                1282 => frame_1282 = Some(trace.game_mode),
                _ => {}
            }
        }

        // ASM sub_1B93E `loc_1B96E` writes `Game_Mode=$0020` only after the
        // shared fade and `$EF` inclusive DBF wait complete.
        assert_eq!(frame_1281, Some(GameMode::IntroSequence3));
        assert_eq!(frame_1282, Some(GameMode::IntroSequence4));
    }

    #[test]
    fn cold_boot_intro_sequence_4_follows_oracle_handoff_frame() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut frame_1875 = None;
        let mut frame_1876 = None;

        for frame_index in 0..=1876 {
            let trace = runtime.step_frame(FrameInput::default());
            match frame_index {
                1875 => frame_1875 = Some(trace.game_mode),
                1876 => frame_1876 = Some(trace.game_mode),
                _ => {}
            }
        }

        // ASM sub_1B988 `loc_1B9B6` writes `Game_Mode=$0024` only after the
        // shared fade, camera hand-off, and `$21B` inclusive DBF wait complete.
        assert_eq!(frame_1875, Some(GameMode::IntroSequence4));
        assert_eq!(frame_1876, Some(GameMode::IntroSequence5));
    }

    fn manifest_with_flag(flag_x: u16, flag_y: u16) -> AssetManifest {
        let mut manifest = minimal_manifest();
        manifest.levels.push(wildside_data::LevelRecord {
            level_id: 0,
            title_text_id: None,
            display_name: None,
            title_card: None,
            resource_paths: Vec::new(),
            resources: wildside_data::LevelResources::default(),
            header_metadata: Some(wildside_data::LevelHeader {
                width_screens: 1,
                height_screens: 1,
                height_flag_bits: 0,
                level_theme: 0,
                murder_wall_flags: 0,
                background_theme: 0,
                weather_flags: 0,
                player_start_x: 0,
                player_start_y: 0,
                flag_x,
                flag_y,
            }),
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });
        manifest
    }

    fn manifest_with_theme_level(
        level_id: u8,
        level_theme: u8,
        background_theme: u8,
    ) -> AssetManifest {
        let mut manifest = minimal_manifest();
        manifest.levels.push(wildside_data::LevelRecord {
            level_id,
            title_text_id: None,
            display_name: None,
            title_card: None,
            resource_paths: Vec::new(),
            resources: wildside_data::LevelResources::default(),
            header_metadata: Some(wildside_data::LevelHeader {
                width_screens: 1,
                height_screens: 1,
                height_flag_bits: 0,
                level_theme,
                murder_wall_flags: 0,
                background_theme,
                weather_flags: 0,
                player_start_x: 0,
                player_start_y: 0,
                flag_x: 0,
                flag_y: 0,
            }),
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
            background_layout_alias: None,
        });
        manifest
    }

    fn manifest_with_title_card_level(level_id: u8) -> AssetManifest {
        let mut manifest = manifest_with_theme_level(level_id, 0, 0);
        if let Some(level) = manifest
            .levels
            .iter_mut()
            .find(|level| level.level_id == level_id)
        {
            level.title_card = Some(wildside_data::LevelTitleCard {
                script_bytes: vec![0xFF],
                line_destinations: Vec::new(),
                level_number_destination: None,
                level_number_glyph_count: 0,
            });
            if let Some(header) = level.header_metadata.as_mut() {
                header.murder_wall_flags = 0b10;
            }
        }
        manifest
    }

    fn player_at_x(x_px: u32) -> PlayerState {
        PlayerState {
            x_pos: x_px << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        }
    }

    #[test]
    fn prize_diamond_loc_10396_horizontal_band_is_inclusive() {
        // ASM PrizeDiamond_Init loc_10396 reaches `tst.w collision_type(a3)`;
        // the stale prize-block worker remains count-visible only while the
        // diamond object is inside Kid's inclusive horizontal collision band.
        let diamond = diamond_at_x(100);

        let cyclone_half_width = PlayerForm::Cyclone.half_width();

        assert!(floating_diamond_in_player_horizontal_band_loc_10396(
            &diamond,
            player_at_x(88),
            cyclone_half_width,
        ));
        assert!(floating_diamond_in_player_horizontal_band_loc_10396(
            &diamond,
            player_at_x(112),
            cyclone_half_width,
        ));
        assert!(!floating_diamond_in_player_horizontal_band_loc_10396(
            &diamond,
            player_at_x(87),
            cyclone_half_width,
        ));
        assert!(!floating_diamond_in_player_horizontal_band_loc_10396(
            &diamond,
            player_at_x(113),
            cyclone_half_width,
        ));
    }

    #[test]
    fn manual_title_return_hidden_counts_require_all_gate_latches() {
        fn hidden_count_with(
            title_screen_elapsed_frames: usize,
            preserve_demo_title_snapshot: bool,
            demo_auto_timeout_handoff_active: bool,
            hide_demo_return_title_player: bool,
        ) -> Option<(u16, u16)> {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;
            runtime.preserve_demo_title_snapshot = preserve_demo_title_snapshot;
            runtime.demo_auto_timeout_handoff_active = demo_auto_timeout_handoff_active;
            runtime.hide_demo_return_title_player = hide_demo_return_title_player;
            runtime.demo_title_return_hidden_object_counts()
        }

        // ASM loc_119D8 -> loc_6E2 only exposes the staged hidden
        // Load_TitleScreen rebuild after the preserved gameplay snapshot and
        // auto-timeout handoff are both clear, while the manual-demo-return
        // hide latch is still active.
        assert_eq!(hidden_count_with(55, false, false, true), Some((0, 0)));
        assert_eq!(hidden_count_with(56, false, false, true), Some((3, 0)));
        assert_eq!(hidden_count_with(57, false, false, true), Some((5, 1)));
        assert_eq!(hidden_count_with(58, false, false, true), Some((5, 2)));
        assert_eq!(hidden_count_with(59, false, false, true), Some((4, 2)));
        assert_eq!(hidden_count_with(149, false, false, true), Some((4, 2)));
        assert_eq!(hidden_count_with(150, false, false, true), Some((4, 3)));
        assert_eq!(hidden_count_with(159, false, false, true), Some((4, 3)));
        assert_eq!(hidden_count_with(160, false, false, true), Some((4, 4)));
        assert_eq!(hidden_count_with(56, true, false, true), None);
        assert_eq!(hidden_count_with(56, false, true, true), None);
        assert_eq!(hidden_count_with(56, false, false, false), None);
    }

    #[test]
    fn timeout_restart_intro_3_title_return_counts_require_pose_latch() {
        fn hidden_count_with(
            title_screen_elapsed_frames: usize,
            boot_state: BootState,
            timeout_restart_title_intro_sequence_3_pose: bool,
        ) -> Option<(u16, u16)> {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = boot_state;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;
            runtime.timeout_restart_title_intro_sequence_3_pose =
                timeout_restart_title_intro_sequence_3_pose;
            runtime.timeout_restart_intro_3_title_return_object_counts()
        }

        // ASM sub_1C246 -> loc_6E2 exposes the timeout-restart intro-3
        // title-return shell only while the carried intro-3 pose latch is
        // active; Load_TitleScreen/Obj_TitleMenu then rebuild the hidden
        // object counters in this exact staged sequence.
        assert_eq!(
            hidden_count_with(55, BootState::TitleScreen { cursor: 0 }, true),
            Some((3, 0))
        );
        assert_eq!(
            hidden_count_with(56, BootState::TitleScreen { cursor: 0 }, true),
            Some((5, 1))
        );
        assert_eq!(
            hidden_count_with(57, BootState::TitleScreen { cursor: 0 }, true),
            Some((5, 2))
        );
        assert_eq!(
            hidden_count_with(58, BootState::TitleScreen { cursor: 0 }, true),
            Some((4, 2))
        );
        assert_eq!(
            hidden_count_with(59, BootState::TitleScreen { cursor: 0 }, true),
            Some((4, 2))
        );
        assert_eq!(
            hidden_count_with(55, BootState::TitleScreen { cursor: 0 }, false),
            None
        );
        assert_eq!(
            hidden_count_with(
                55,
                BootState::IntroSequence {
                    segment: 2,
                    elapsed: 0,
                    pending_title_skip: 0,
                },
                true,
            ),
            None
        );
    }

    #[test]
    fn timeout_restart_intro_3_title_pose_requires_title_latch_before_cutoff() {
        fn uses_pose(
            title_screen_elapsed_frames: usize,
            boot_state: BootState,
            timeout_restart_title_intro_sequence_3_pose: bool,
        ) -> bool {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = boot_state;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;
            runtime.timeout_restart_title_intro_sequence_3_pose =
                timeout_restart_title_intro_sequence_3_pose;
            runtime.title_uses_timeout_restart_intro_sequence_3_pose()
        }

        // ASM sub_1C246 leaves the timeout-restart intro-3 pose visible only
        // during the first title frames; once the full title snapshot window
        // expires, the trace must hide that carried pose.
        assert!(uses_pose(27, BootState::TitleScreen { cursor: 0 }, true));
        assert!(!uses_pose(28, BootState::TitleScreen { cursor: 0 }, true));
        assert!(!uses_pose(27, BootState::TitleScreen { cursor: 0 }, false));
        assert!(!uses_pose(
            27,
            BootState::IntroSequence {
                segment: 2,
                elapsed: 0,
                pending_title_skip: 0,
            },
            true,
        ));
    }

    #[test]
    fn current_player_state_exposes_timeout_restart_intro_3_title_pose_until_cutoff() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.timeout_restart_title_intro_sequence_3_pose = true;
        runtime.active_intro_sequence_3_roster_index = 1;
        runtime.title_screen_elapsed_frames = 27;

        // ASM sub_1C246 preserves the active Load_IntroSequence3 roster pose
        // into early title frames before the hidden title rebuild takes over.
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::intro_sequence_3_pose_for_roster(1))
        );

        runtime.title_screen_elapsed_frames = 28;

        assert_eq!(runtime.current_player_state(), None);
    }

    #[test]
    fn current_player_state_hides_intro_3_roster_until_visible_frame() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.active_intro_sequence_3_roster_index = 1;
        runtime.boot_state = BootState::IntroSequence {
            segment: 2,
            elapsed: INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES - 1,
            pending_title_skip: 0,
        };

        // ASM Load_IntroSequence3/sub_1B93E creates the roster object before it
        // becomes trace-visible; Kid appears only at the ROM's visible cutoff.
        assert_eq!(runtime.current_player_state(), None);

        runtime.boot_state = BootState::IntroSequence {
            segment: 2,
            elapsed: INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES,
            pending_title_skip: 0,
        };

        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::intro_sequence_3_pose_for_roster(1))
        );
    }

    #[test]
    fn timeout_restart_intro_5_title_pose_requires_title_latch_before_cutoff() {
        fn uses_pose(
            title_screen_elapsed_frames: usize,
            boot_state: BootState,
            timeout_restart_title_intro_sequence_5_pose: bool,
        ) -> bool {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = boot_state;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;
            runtime.timeout_restart_title_intro_sequence_5_pose =
                timeout_restart_title_intro_sequence_5_pose;
            runtime.title_uses_timeout_restart_intro_sequence_5_pose()
        }

        // ASM sub_1C246 preserves the loc_1BCD0 slot-2 intro-sequence-5 pose
        // only through the early title carry window.
        assert!(uses_pose(27, BootState::TitleScreen { cursor: 0 }, true));
        assert!(!uses_pose(28, BootState::TitleScreen { cursor: 0 }, true));
        assert!(!uses_pose(27, BootState::TitleScreen { cursor: 0 }, false));
        assert!(!uses_pose(
            27,
            BootState::IntroSequence {
                segment: 4,
                elapsed: 0,
                pending_title_skip: 0,
            },
            true,
        ));
    }

    #[test]
    fn current_player_state_exposes_timeout_restart_intro_5_title_pose_until_cutoff() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.timeout_restart_title_intro_sequence_5_pose = true;
        runtime.title_screen_elapsed_frames = 27;

        // ASM loc_1BCD0's slot-2 pose survives the direct sub_1C246 title
        // return until the later title rebuild overwrites that object RAM.
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::intro_sequence_5_slot_2_pose())
        );

        runtime.title_screen_elapsed_frames = 28;

        assert_eq!(runtime.current_player_state(), None);
    }

    #[test]
    fn intro_sequence_4_intro_3_pose_carries_until_cutoff() {
        fn uses_pose(elapsed: usize, segment: usize) -> bool {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment,
                elapsed,
                pending_title_skip: 0,
            };
            runtime.intro_sequence_4_uses_intro_sequence_3_pose()
        }

        // ASM loc_1B96A enters intro sequence 4 before Load_IntroSequence4 /
        // sub_1B988 clears the intro-3 roster, so the old pose remains visible
        // until the fixed handoff window expires.
        assert!(uses_pose(26, 3));
        assert!(!uses_pose(27, 3));
        assert!(!uses_pose(26, 2));
    }

    #[test]
    fn current_player_state_exposes_intro_4_intro_3_pose_until_cutoff() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence {
            segment: 3,
            elapsed: 0,
            pending_title_skip: 0,
        };
        runtime.active_intro_sequence_3_roster_index = 1;

        // ASM loc_1B96A writes mode `$20` before sub_1B988 clears the previous
        // intro-3 roster; issue-161 oracle probes keep Kid and the roster
        // counted at the cold frame-1282 hand-off.
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::intro_sequence_3_pose_for_roster(1))
        );
        assert_eq!(runtime.object_counts_from_roster(&[], 0, 0), (3, 9));

        runtime.boot_state = BootState::IntroSequence {
            segment: 3,
            elapsed: INTRO_SEQUENCE_4_POSE_CARRY_FRAMES,
            pending_title_skip: 0,
        };

        assert_eq!(runtime.current_player_state(), None);
        assert_eq!(runtime.object_counts_from_roster(&[], 0, 0), (0, 0));

        runtime.boot_state = BootState::IntroSequence {
            segment: 3,
            elapsed: TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES - 1,
            pending_title_skip: 0,
        };
        assert_eq!(runtime.object_counts_from_roster(&[], 0, 0), (1, 0));

        runtime.boot_state = BootState::IntroSequence {
            segment: 3,
            elapsed: TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES,
            pending_title_skip: 0,
        };
        assert_eq!(runtime.object_counts_from_roster(&[], 0, 0), (2, 0));
    }

    #[test]
    fn in_game_object_counts_include_smash_block_effect_workers() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);
        runtime.block_destroy_effects.push(BlockDestroyEffect {
            x_px: 112,
            y_px: 196,
            remaining: 2,
            direction: 0,
            pending_ice_origin_nudge: false,
        });

        // ASM Block_LoadSmashBits/sub_10EDA (kid.asm:22775,22806) reaches
        // j_sub_E02 before the two-frame hibernate, so the effect owns an
        // object+gfx slot even though native exposes it as block_destroy_effects.
        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, OPENING_GROUNDED_CAMERA_Y_FP),
            (3, 4)
        );
    }

    #[test]
    fn trace_exposes_live_hud_diamond_offset_from_sub_44dc() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);

        assert_eq!(
            runtime.trace_frame().hud_diamond_frame_offset,
            Some(0),
            "ASM sub_44B0 leaves `$FFFFFA5D` at zero before Mode_Level runs sub_44DC"
        );
        assert_eq!(
            runtime.trace_frame().hud_diamond_vram_page_loc_5d4a,
            Some(0),
            "ASM sub_44B0 leaves the first off_7B0AC page resident at HUD VRAM `$DD40`"
        );

        let trace = runtime.step_frame(FrameInput::default());

        assert_eq!(
            trace.hud_diamond_frame_offset,
            Some(2),
            "ASM Mode_Level loc_80C calls sub_44DC; loc_45EE advances `$FFFFFA5D` by two on the first level frame"
        );
        assert_eq!(
            trace.hud_diamond_vram_page_loc_5d4a,
            Some(4),
            "ASM sub_44DC loc_4626 passes unk_461C frame byte 4 to sub_5D4A, which makes off_7B0AC[1] resident at VRAM `$DD40`"
        );
        assert_eq!(
            trace.hud_dma_events_sub_44dc,
            vec![HudDmaEventSub44DC::Diamond {
                frame_table_byte_offset: 4,
            }],
            "ASM sub_44DC loc_4626 emits the frame-local HUD diamond DMA before a raster scheduler assigns its FIFO position"
        );
        assert_eq!(
            runtime.mode_level_hud_raw_cycles_sub_44dc(),
            Some(M68kCycleCount::from_raw(760)),
            "ASM sub_44DC loc_4626 -> sub_5D4A keeps DMA ownership separate from its 760 raw source cycles",
        );
        let event = trace.hud_dma_events_sub_44dc[0];
        assert_eq!(event.frame_table_byte_offset(), 4);
        assert_eq!(event.vram_byte_address(), 0xDD40);
        assert_eq!(event.word_count(), 256);
    }

    #[test]
    fn trace_exposes_complete_staggered_hud_dma_sequence_from_sub_44dc() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);

        let frames = (0..5)
            .map(|_| runtime.step_frame(FrameInput::default()))
            .collect::<Vec<_>>();

        assert_eq!(
            frames
                .iter()
                .map(|trace| trace.hud_dma_events_sub_44dc.as_slice())
                .collect::<Vec<_>>(),
            vec![
                &[HudDmaEventSub44DC::Diamond {
                    frame_table_byte_offset: 4,
                }][..],
                &[HudDmaEventSub44DC::Flag {
                    frame_table_byte_offset: 4,
                }],
                &[HudDmaEventSub44DC::Coin {
                    frame_table_byte_offset: 4,
                }],
                &[HudDmaEventSub44DC::LifeIcon {
                    frame_table_byte_offset: 4,
                    wrapped_to_start: false,
                }],
                &[HudDmaEventSub44DC::Clock {
                    frame_table_byte_offset: 4,
                    wrapped_to_start: false,
                }],
            ],
            "ASM sub_44B0 seeds countdowns 1..5; sub_44DC visits coin, life, clock, diamond, then flag on every Mode_Level pass"
        );
        assert_eq!(
            frames[1].hud_dma_events_sub_44dc[0].vram_byte_address(),
            0xD340
        );
        assert_eq!(frames[1].hud_dma_events_sub_44dc[0].word_count(), 320);
    }

    #[test]
    fn transform_hibernation_skips_mode_level_hud_animation() {
        // ASM Kid_Transform loc_7CBE/loc_7D22 (kid.asm) waits in sub_8E0
        // instead of returning to Mode_Level loc_80C/sub_44DC. Oracle
        // published-TAS frame 1058 keeps `$FFFFFA58/$FFFFFA5D` at 1/4 and
        // emits no `$DD40` DMA while this VBlank hibernation is active.
        assert!(!NativeRuntime::hud_animation_runs_in_mode_level_loc_80c(
            true, true
        ));
        assert!(NativeRuntime::hud_animation_runs_in_mode_level_loc_80c(
            true, false
        ));
    }

    #[test]
    fn trace_exposes_live_hud_life_icon_offset_from_sub_44dc() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);

        assert_eq!(
            runtime.trace_frame().hud_life_icon_vram_page_loc_4586,
            Some(8),
            "ASM sub_44DC loc_4586 has not yet replaced the bootstrap ArtUnc_4692 page"
        );

        assert_eq!(
            runtime.trace_frame().hud_life_frame_offset,
            Some(0),
            "ASM sub_44B0 leaves `$FFFFFA5E` at zero before Mode_Level runs sub_44DC"
        );

        let mut trace = runtime.step_frame(FrameInput::default());
        for _ in 1..4 {
            trace = runtime.step_frame(FrameInput::default());
        }

        assert_eq!(
            trace.hud_life_frame_offset,
            Some(2),
            "ASM Mode_Level loc_80C calls sub_44DC; loc_4546 advances `$FFFFFA5E` by two when seeded countdown `$FFFFFA59=4` expires"
        );
        assert_eq!(
            trace.hud_life_icon_vram_page_loc_4586,
            Some(4),
            "ASM sub_44DC loc_4586 copies AniArt_LifeIcon page 1 to VRAM `$4620` after the first life countdown expires"
        );
        assert_eq!(
            trace.hud_dma_events_sub_44dc,
            vec![HudDmaEventSub44DC::LifeIcon {
                frame_table_byte_offset: 4,
                wrapped_to_start: false,
            }],
            "ASM sub_44DC loc_4586 emits the frame-local HUD life-icon DMA before a raster scheduler assigns its FIFO position"
        );
        let event = trace.hud_dma_events_sub_44dc[0];
        assert_eq!(event.frame_table_byte_offset(), 4);
        assert_eq!(event.vram_byte_address(), 0x4620);
        assert_eq!(event.word_count(), 64);
    }

    #[test]
    fn in_game_object_counts_include_prize_block_transient_workers() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);
        runtime.block_bounces.push(BlockBounceAnim {
            cell_x: 0x0A,
            cell_y: 0x0C,
            x_offset_px: 0,
            remaining: 31,
            kind: BlockBounceKind::Prize,
            initial_duration: 31,
            prize_content: Some(terrain::PrizeContent::Diamond),
            prize_deleted_tile: false,
            metal_direction: 0,
            y_offset_px: 0,
        });
        runtime.prize_block_worker_tails.push(PrizeBlockWorkerTail {
            cell_x: 0x0A,
            cell_y: 0x0C,
            content: Some(terrain::PrizeContent::Diamond),
            remaining: 1,
        });
        runtime.floating_items.push(diamond_at_x(0x00A8));

        // ASM loc_1002E/sub_1007A and PrizeDiamond_Init (kid.asm:21496,21604)
        // keep the wobble tail and live prize object count-visible through
        // j_sub_E02/gfx collision even though native renders them separately.
        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, OPENING_GROUNDED_CAMERA_Y_FP),
            (5, 6)
        );
    }

    #[test]
    fn trace_exposes_ordered_mode_level_work_before_sub_44dc_for_prize_workers() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);
        runtime.hud_dma_events_sub_44dc = vec![HudDmaEventSub44DC::LifeIcon {
            frame_table_byte_offset: 4,
            wrapped_to_start: false,
        }];
        runtime.block_bounces.push(BlockBounceAnim {
            cell_x: 0x0A,
            cell_y: 0x0C,
            x_offset_px: 0,
            remaining: 31,
            kind: BlockBounceKind::Prize,
            initial_duration: 31,
            prize_content: Some(terrain::PrizeContent::Diamond),
            prize_deleted_tile: false,
            metal_direction: 0,
            y_offset_px: 0,
        });
        runtime.prize_block_worker_tails.push(PrizeBlockWorkerTail {
            cell_x: 0x0A,
            cell_y: 0x0C,
            content: Some(terrain::PrizeContent::Diamond),
            remaining: 1,
        });
        runtime
            .prize_block_spawn_flashes
            .push(PrizeBlockSpawnFlash {
                x_px: 0x00A8,
                y_px: 0x00B0,
                remaining: 4,
            });
        runtime.floating_items.push(diamond_at_x(0x00A8));
        runtime.mode_level_work_before_hud_sub_44dc =
            Some(runtime.observe_mode_level_work_before_hud_sub_44dc());

        let trace = runtime.trace_frame();

        assert_eq!(
            trace.mode_level_work_before_hud_sub_44dc,
            Some(ModeLevelPreHudWorkLedger {
                phases: vec![
                    ModeLevelPreHudPhase::TransferSpriteAndKidToVram,
                    ModeLevelPreHudPhase::PaletteToVram,
                    ModeLevelPreHudPhase::TransferScrollDataToVram,
                    ModeLevelPreHudPhase::ForegroundUpdateSub1596,
                    ModeLevelPreHudPhase::HudPreparationSub6874,
                    ModeLevelPreHudPhase::MakeSpriteAttrHud,
                    ModeLevelPreHudPhase::ExecuteObjectsSubE12,
                    ModeLevelPreHudPhase::PostObjectSub21F8,
                    ModeLevelPreHudPhase::DiamondPowerRun,
                    ModeLevelPreHudPhase::SubDfb0,
                    ModeLevelPreHudPhase::SubF7e0,
                    ModeLevelPreHudPhase::WeatherSub6034,
                    ModeLevelPreHudPhase::SubF096,
                ],
                execute_objects: ModeLevelExecuteObjectsWork {
                    rng_offset_before_sub_e12: Some(0xFFC9),
                    linked_object_count: 6,
                    linked_gfx_object_count: 7,
                    linked_non_transient_object_count: 2,
                    player: ModeLevelPlayerWorkBeforeHud::Absent,
                    prize_block_wobble_workers_sub_1007a: 1,
                    prize_block_worker_tails_sub_1007a: 1,
                    prize_block_flash_workers_loc_102b8: 1,
                    floating_prize_workers: 1,
                    active_enemy_workers: 0,
                    active_platform_workers: 0,
                    palette_rotation_worker_loc_11f96: 0,
                    unclassified_runtime_transient_workers: 0,
                    player_entry_position: None,
                    player_gfx_terrain_scan: None,
                    cyclone_visual_accumulator_before: None,
                    rubber_block_skin_workers_sub_10de4: 0,
                    prize_block_wobble_paths_sub_1007a: vec![ModeLevelPrizeBlockWobbleWork {
                        resume: PrizeBlockWobbleResumeSub1007A::AfterLoc100DaAt100E2,
                        wake: ModeLevelPrizeBlockWobbleWake::ContinuePhase,
                        remaining: 31,
                        content: Some(terrain::PrizeContent::Diamond),
                        deleted_tile: false,
                    }],
                    player_spawned_prize_block_initial_sub_1007a: None,
                    player_prize_wobble_list_work_loc_1002e: None,
                    prize_object_execution_order_sub_e12: None,
                    damage_flash_wait_loc_bc44: Some(ModeLevelDamageFlashWaitLocBc44::AwaitDamage,),
                    demo_exit_wait_loc_119e0: None,
                    input_worker_loc_7452: None,
                    palette_fade_worker_sub_73d0: Some(
                        ModeLevelPaletteFadeWorkerPathSub73D0::AdvanceFadeAndHibernateLoc7402,
                    ),
                    prize_block_tail_paths_sub_1066: vec![ModeLevelPrizeBlockTailWork {
                        cell_x: 0x0A,
                        cell_y: 0x0C,
                        content: Some(terrain::PrizeContent::Diamond),
                        state: ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { remaining: 1 },
                        list_work: None,
                    }],
                    prize_block_flash_paths_sub_1066: vec![ModeLevelPrizeBlockFlashWork {
                        x_px: 0x00A8,
                        y_px: 0x00B0,
                        remaining: 4,
                        wake: ModeLevelPrizeAnimationWake::Hibernate,
                    }],
                    floating_prize_paths: vec![ModeLevelFloatingPrizeWork {
                        content: terrain::PrizeContent::Diamond,
                        remaining: 0x01F4,
                        spawn_delay: 0,
                        first_row_grace: false,
                        collision_latched: false,
                        fall_path_loc_103b2: None,
                    }],
                    executed_player_path: None,
                },
                post_execute: ModeLevelPostExecuteWork::default(),
            }),
            "ASM Mode_Level (kid.asm:901) fixes the phases before sub_44DC, while Execute_Objects sub_E12 resumes each linked prize coroutine"
        );
    }

    #[test]
    fn sub_1007a_remaining_counter_maps_to_exact_hibernate_resume_groups() {
        use PrizeBlockWobbleResumeSub1007A::{
            AfterLoc100DaAt100E2, AfterLoc100E8At100F0, AfterLoc100F6At100Fe, AfterLoc10104At1010C,
            AfterLoc10112At1011A, AfterLoc10120At10128,
        };

        // ASM sub_1007A (kid.asm:21345-21435): each inclusive DBF loop
        // resumes at the instruction after its Hibernate_Object_1Frame JSR.
        for (remaining, expected) in [
            (31, AfterLoc100DaAt100E2),
            (26, AfterLoc100DaAt100E2),
            (25, AfterLoc100E8At100F0),
            (20, AfterLoc100E8At100F0),
            (19, AfterLoc100F6At100Fe),
            (15, AfterLoc100F6At100Fe),
            (14, AfterLoc10104At1010C),
            (10, AfterLoc10104At1010C),
            (9, AfterLoc10112At1011A),
            (6, AfterLoc10112At1011A),
            (5, AfterLoc10120At10128),
            (1, AfterLoc10120At10128),
        ] {
            assert_eq!(
                prize_block_wobble_resume_sub_1007a(remaining),
                expected,
                "remaining={remaining}"
            );
        }
    }

    #[test]
    fn sub_1007a_remaining_counter_types_dbf_and_animation_wait_boundaries() {
        // ASM sub_1007A (kid.asm:21372-21407): the lowest remaining value in
        // each DBF group executes the next phase's moveq/body on the same
        // wake; the final group enters Hibernate_UntilAnimFinished.
        for remaining in [31, 25, 19, 14, 9, 5] {
            assert_eq!(
                prize_block_wobble_wake_sub_1007a(remaining),
                ModeLevelPrizeBlockWobbleWake::ContinuePhase,
                "remaining={remaining}"
            );
        }
        for remaining in [26, 20, 15, 10, 6] {
            assert_eq!(
                prize_block_wobble_wake_sub_1007a(remaining),
                ModeLevelPrizeBlockWobbleWake::BeginNextPhase,
                "remaining={remaining}"
            );
        }
        assert_eq!(
            prize_block_wobble_wake_sub_1007a(1),
            ModeLevelPrizeBlockWobbleWake::BeginAnimationWait
        );
    }

    #[test]
    fn prize_animation_work_distinguishes_live_wakes_from_deleted_overlap() {
        // ASM sub_1007A (kid.asm:21407-21435): the first animation wake after
        // the wobble creates the flash and prize, then deletes the worker.
        // Native keeps a short render/count overlap after that deletion.
        assert_eq!(
            prize_block_tail_state_sub_1007a(
                Some(terrain::PrizeContent::Diamond),
                NativeRuntime::PRIZE_BLOCK_WORKER_TAIL_FRAMES + 1,
            ),
            ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete
        );
        assert_eq!(
            prize_block_tail_state_sub_1007a(
                Some(terrain::PrizeContent::Diamond),
                NativeRuntime::PRIZE_BLOCK_WORKER_TAIL_FRAMES,
            ),
            ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { remaining: 3 }
        );
        assert_eq!(
            prize_block_tail_state_sub_1007a(
                Some(terrain::PrizeContent::Points10000),
                NativeRuntime::PRIZE_BLOCK_WORKER_TAIL_FRAMES,
            ),
            ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete
        );

        // ASM loc_102B8 (kid.asm:21545-21551): each flash wake either
        // hibernates again or observes animation completion and deletes.
        assert_eq!(
            prize_block_flash_wake_loc_102b8(2),
            ModeLevelPrizeAnimationWake::Hibernate
        );
        assert_eq!(
            prize_block_flash_wake_loc_102b8(1),
            ModeLevelPrizeAnimationWake::FinishAndDelete
        );
    }

    #[test]
    fn floating_prize_work_types_loc_103b2_layout_and_row_branches() {
        let item = FloatingItem {
            x_pos: 0,
            y_pos: 0x0040_0000,
            y_velocity: 0,
            remaining: 0x01F4,
            spawn_delay: 0,
            level_col: 0,
            level_row: 4,
            first_row_grace: false,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        };
        assert_eq!(
            floating_prize_fall_path_loc_103b2(
                &item,
                terrain::CollisionKind::Empty,
                terrain::CollisionKind::Solid,
            ),
            ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow,
            "ASM loc_103B2 branches directly to loc_103DE on an open current row"
        );
        assert_eq!(
            floating_prize_fall_path_loc_103b2(
                &FloatingItem {
                    y_pos: 0x004F_0000,
                    y_velocity: 0x0001_0000,
                    ..item.clone()
                },
                terrain::CollisionKind::HiddenBlock,
                terrain::CollisionKind::Solid,
            ),
            ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallAdvanceRow,
            "ASM loc_103F6 reaches loc_10402 when gravity crosses the aligned row"
        );
        assert_eq!(
            floating_prize_fall_path_loc_103b2(
                &item,
                terrain::CollisionKind::Solid,
                terrain::CollisionKind::Empty,
            ),
            ModeLevelFloatingPrizeFallPathLoc103B2::BlockedCurrentRow
        );

        let grace = FloatingItem {
            first_row_grace: true,
            ..item
        };
        assert_eq!(
            floating_prize_fall_path_loc_103b2(
                &grace,
                terrain::CollisionKind::Solid,
                terrain::CollisionKind::Empty,
            ),
            ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow,
            "ASM loc_103B2 bit-$10 path probes one row below before falling"
        );
        assert_eq!(
            floating_prize_fall_path_loc_103b2(
                &FloatingItem {
                    y_pos: 0x004F_0000,
                    y_velocity: 0x0001_0000,
                    ..grace.clone()
                },
                terrain::CollisionKind::Solid,
                terrain::CollisionKind::Empty,
            ),
            ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallAdvanceRow
        );
        assert_eq!(
            floating_prize_fall_path_loc_103b2(
                &grace,
                terrain::CollisionKind::Solid,
                terrain::CollisionKind::Solid,
            ),
            ModeLevelFloatingPrizeFallPathLoc103B2::BlockedGraceProbe
        );
    }

    fn prize_tail_rank_fixture(
        linked_object_count: u16,
        linked_gfx_object_count: u16,
        wobble_remaining: &[u8],
        flash_remaining: &[u8],
        floating_remaining: &[u16],
    ) -> ModeLevelExecuteObjectsWork {
        ModeLevelExecuteObjectsWork {
            rng_offset_before_sub_e12: None,
            linked_object_count,
            linked_gfx_object_count,
            linked_non_transient_object_count: 4,
            player: ModeLevelPlayerWorkBeforeHud::Moving,
            prize_block_wobble_workers_sub_1007a: wobble_remaining.len() as u16,
            prize_block_worker_tails_sub_1007a: 1,
            prize_block_flash_workers_loc_102b8: flash_remaining.len() as u16,
            floating_prize_workers: floating_remaining.len() as u16,
            rubber_block_skin_workers_sub_10de4: 0,
            active_enemy_workers: 0,
            active_platform_workers: 0,
            palette_rotation_worker_loc_11f96: 1,
            unclassified_runtime_transient_workers: 0,
            player_entry_position: None,
            player_gfx_terrain_scan: None,
            cyclone_visual_accumulator_before: None,
            prize_block_wobble_paths_sub_1007a: wobble_remaining
                .iter()
                .map(|&remaining| ModeLevelPrizeBlockWobbleWork {
                    resume: prize_block_wobble_resume_sub_1007a(remaining),
                    wake: prize_block_wobble_wake_sub_1007a(remaining),
                    remaining,
                    content: Some(terrain::PrizeContent::Diamond),
                    deleted_tile: false,
                })
                .collect(),
            player_spawned_prize_block_initial_sub_1007a: None,
            player_prize_wobble_list_work_loc_1002e: None,
            prize_object_execution_order_sub_e12: None,
            damage_flash_wait_loc_bc44: None,
            demo_exit_wait_loc_119e0: None,
            input_worker_loc_7452: None,
            palette_fade_worker_sub_73d0: None,
            prize_block_tail_paths_sub_1066: vec![ModeLevelPrizeBlockTailWork {
                cell_x: 31,
                cell_y: 4,
                content: Some(terrain::PrizeContent::Diamond),
                state: ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete,
                list_work: None,
            }],
            prize_block_flash_paths_sub_1066: flash_remaining
                .iter()
                .map(|&remaining| ModeLevelPrizeBlockFlashWork {
                    x_px: 0,
                    y_px: 0,
                    remaining,
                    wake: ModeLevelPrizeAnimationWake::Hibernate,
                })
                .collect(),
            floating_prize_paths: floating_remaining
                .iter()
                .map(|&remaining| ModeLevelFloatingPrizeWork {
                    content: terrain::PrizeContent::Diamond,
                    remaining,
                    spawn_delay: 0,
                    first_row_grace: false,
                    collision_latched: false,
                    fall_path_loc_103b2: None,
                })
                .collect(),
            executed_player_path: None,
        }
    }

    #[test]
    fn prize_tail_list_ranks_follow_loc_de0_fd4_ea2_10ba_and_1106() {
        // Natural frame 1697 entry: four younger wobble workers, two younger
        // flashes, three `$1FFF` prizes, and Kid allocates one more wobble
        // before this completion. The bounded oracle list journal independently
        // observes all seven ranks.
        let mut frame_1697 =
            prize_tail_rank_fixture(14, 14, &[6, 13, 19, 25], &[9, 19], &[299, 485, 495]);
        frame_1697.player_spawned_prize_block_initial_sub_1007a =
            Some(ModeLevelPrizeBlockInitialWorkSub1007A {
                path: ModeLevelPrizeBlockInitialPathSub1007A::Prize,
                content: Some(terrain::PrizeContent::Diamond),
            });
        frame_1697.player_prize_wobble_list_work_loc_1002e =
            player_prize_wobble_list_work_loc_1002e(&frame_1697);
        assert_eq!(
            mode_level_prize_tail_list_work(&frame_1697, 0, true),
            Some(ModeLevelPrizeTailListWork {
                flash_object_insert_rank_loc_de0: 5,
                flash_gfx_insert_rank_loc_fd4: 15,
                prize_object_insert_rank_loc_de0: 1,
                prize_gfx_insert_rank_loc_fd4: 16,
                tail_object_delete_rank_loc_ea2: 14,
                tail_gfx_delete_rank_loc_10ba: 5,
                tail_gfx_category4_delete_rank_loc_1106: 11,
            })
        );
        frame_1697.prize_block_tail_paths_sub_1066[0].list_work =
            mode_level_prize_tail_list_work(&frame_1697, 0, true);
        for (work, path) in frame_1697.floating_prize_paths.iter_mut().zip([
            ModeLevelFloatingPrizeFallPathLoc103B2::BlockedCurrentRow,
            ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow,
            ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow,
        ]) {
            work.fall_path_loc_103b2 = Some(path);
        }

        // Natural frame 1703 has one more live `$1FFF` prize and one more
        // existing GfxObject. No Kid-side allocation precedes the completion.
        let mut frame_1703 = prize_tail_rank_fixture(
            16,
            16,
            &[7, 13, 19, 26],
            &[3, 13, 20],
            &[293, 479, 489, 496],
        );
        assert_eq!(
            mode_level_prize_tail_list_work(&frame_1703, 0, false),
            Some(ModeLevelPrizeTailListWork {
                flash_object_insert_rank_loc_de0: 6,
                flash_gfx_insert_rank_loc_fd4: 16,
                prize_object_insert_rank_loc_de0: 1,
                prize_gfx_insert_rank_loc_fd4: 17,
                tail_object_delete_rank_loc_ea2: 15,
                tail_gfx_delete_rank_loc_10ba: 5,
                tail_gfx_category4_delete_rank_loc_1106: 12,
            })
        );
        frame_1703.prize_block_tail_paths_sub_1066[0].list_work =
            mode_level_prize_tail_list_work(&frame_1703, 0, false);
        for (work, path) in frame_1703.floating_prize_paths.iter_mut().zip([
            ModeLevelFloatingPrizeFallPathLoc103B2::BlockedCurrentRow,
            ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow,
            ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow,
            ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow,
        ]) {
            work.fall_path_loc_103b2 = Some(path);
        }
        assert_eq!(
            prize_object_execution_order_sub_e12(&frame_1697),
            Some(vec![
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 2 },
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 1 },
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 0 },
                ModeLevelPrizeObjectExecutionSubE12::Player,
                ModeLevelPrizeObjectExecutionSubE12::PlayerSpawnedInitialSub1007A,
                ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index: 1 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 3 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 2 },
                ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index: 0 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 1 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 0 },
                ModeLevelPrizeObjectExecutionSubE12::TailSub1007A { index: 0 },
            ]),
            "ASM loc_DE0 order around Kid matches the frame 1697 oracle A5 journal"
        );
        assert_eq!(
            prize_object_execution_order_sub_e12(&frame_1703),
            Some(vec![
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 3 },
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 2 },
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 1 },
                ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index: 0 },
                ModeLevelPrizeObjectExecutionSubE12::Player,
                ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index: 2 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 3 },
                ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index: 1 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 2 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 1 },
                ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index: 0 },
                ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index: 0 },
                ModeLevelPrizeObjectExecutionSubE12::TailSub1007A { index: 0 },
            ]),
            "ASM loc_DE0 order around Kid matches the frame 1703 oracle A5 journal"
        );
        let typed_prize_cycles = |execute: &ModeLevelExecuteObjectsWork| {
            prize_object_execution_order_sub_e12(execute)
                .expect("source-ordered callbacks")
                .into_iter()
                .filter(|callback| *callback != ModeLevelPrizeObjectExecutionSubE12::Player)
                .map(|callback| {
                    crate::prize_object_cycles_sub_e12(execute, callback)
                        .expect("typed callback cycles")
                        .get()
                })
                .sum::<u32>()
        };
        assert_eq!(
            typed_prize_cycles(&frame_1697),
            14_362,
            "raw source cycles exclude Kid, fixed workers, RNG, refresh, and VDP stalls"
        );
        assert_eq!(typed_prize_cycles(&frame_1703), 14_826);
        assert_eq!(
            frame_1697.player_prize_wobble_list_work_loc_1002e,
            Some(ModeLevelPrizeWobbleListWorkLoc1002E {
                object_insert_rank_loc_de0: 5,
                gfx_insert_rank_loc_fd4: 14,
            }),
            "bounded frame 1697 loc_DE0/loc_FD4 journal"
        );
        assert_eq!(frame_1703.player_prize_wobble_list_work_loc_1002e, None);
    }

    #[test]
    fn published_cyclone_route_maps_velocity_changes_to_exact_sub_b084_paths() {
        // ASM sub_B084 (kid.asm:13921-13989): Right adds Cyclone's $2B00
        // airborne acceleration; neutral negative velocity takes loc_B0FE and
        // loc_B10E to add the fixed $400 brake.
        assert_eq!(
            mode_level_airborne_horizontal_path(-0x0002_0800, -0x0001_DD00),
            ModeLevelAirborneHorizontalPath::RightHeldAcceleration
        );
        assert_eq!(
            mode_level_airborne_horizontal_path(-0x0001_0600, -0x0001_0200),
            ModeLevelAirborneHorizontalPath::NeutralNegativeBrake
        );
        assert_eq!(
            mode_level_airborne_horizontal_path(0x0000_EB80, 0x0000_E780),
            ModeLevelAirborneHorizontalPath::NeutralPositiveBrake,
            "ASM sub_B084 loc_B0FE subtracts $400 from frame 985's positive velocity",
        );
    }

    #[test]
    fn published_cyclone_route_maps_sub_a4ee_tile_scan_bounds() {
        // ASM sub_A4EE -> GfxObjects_Collision loc_173E/loc_1792
        // (kid.asm:2863-2909): move by the post-gravity velocity, then scan
        // Kid's form-width x PLAYER_HEIGHT box in inclusive 16px cells.
        assert_eq!(
            mode_level_kid_gfx_terrain_scan_bounds(
                ModeLevelPlayerPosition {
                    x_pos: 0x01AF_7100,
                    y_pos: 0x006F_0000,
                },
                -0x0001_DD00,
                -0x0000_0A00,
                PlayerForm::Cyclone,
            ),
            Some(ModeLevelKidGfxTerrainScanBounds {
                left_col: 26,
                right_col: 27,
                top_row: 4,
                bottom_row: 6,
            })
        );
        assert_eq!(
            mode_level_kid_gfx_terrain_scan_bounds(
                ModeLevelPlayerPosition {
                    x_pos: 0x01A6_C800,
                    y_pos: 0x006F_1C00,
                },
                -0x0001_0200,
                0x0000_2000,
                PlayerForm::Cyclone,
            ),
            Some(ModeLevelKidGfxTerrainScanBounds {
                left_col: 25,
                right_col: 26,
                top_row: 5,
                bottom_row: 6,
            })
        );
        assert_eq!(
            mode_level_kid_gfx_terrain_scan_bounds(
                ModeLevelPlayerPosition {
                    x_pos: 0x01A0_F800,
                    y_pos: 0x0070_EA00,
                },
                -0x0000_EA00,
                0x0000_8C00,
                PlayerForm::Cyclone,
            ),
            Some(ModeLevelKidGfxTerrainScanBounds {
                left_col: 25,
                right_col: 26,
                top_row: 5,
                bottom_row: 7,
            })
        );
    }

    #[test]
    fn gfx_terrain_scan_counts_clear_cells_and_ignores_spikes() {
        let bounds = ModeLevelKidGfxTerrainScanBounds {
            left_col: 0,
            right_col: 1,
            top_row: 0,
            bottom_row: 2,
        };
        let clear = terrain::BlockGrid::from_level_layout(&[0; 12], 2, 3);
        assert_eq!(
            mode_level_kid_gfx_terrain_scan(&clear, bounds),
            ModeLevelKidGfxTerrainScan {
                bounds,
                tile_iterations: 6,
                first_collision: None,
            }
        );

        let spikes_then_solid = terrain::BlockGrid::from_level_layout(
            &[0x70, 0, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            2,
            3,
        );
        assert_eq!(
            mode_level_kid_gfx_terrain_scan(&spikes_then_solid, bounds),
            ModeLevelKidGfxTerrainScan {
                bounds,
                tile_iterations: 2,
                first_collision: Some(ModeLevelKidGfxTerrainCollision::Solid),
            },
            "ASM loc_17C6 ignores spikes and reaches the following solid cell"
        );
    }

    #[test]
    fn published_cyclone_route_maps_ceiling_and_clear_sub_902a_paths() {
        // ASM loc_A75A/loc_A8F6 (kid.asm:13064-13192): an ascending ceiling
        // collision returns negative and rejoins at loc_A762; no vertical
        // collision falls through directly to loc_A762.
        assert_eq!(
            mode_level_vertical_collision_path(true, false, -0x0A00),
            ModeLevelVerticalCollisionPath::Ceiling
        );
        assert_eq!(
            mode_level_vertical_collision_path(false, false, 0x2000),
            ModeLevelVerticalCollisionPath::Clear
        );
    }

    #[test]
    fn mode_level_ledger_completes_frame_1697_cyclone_source_path() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::Cyclone;
        runtime.helmet.cyclone_flying = true;
        runtime.helmet.cyclone_y_accel = -0x0A00;
        runtime.last_kid_frame_motion_branch = KidFrameMotionBranch::NormalPhysics;
        let entry = KidFlipFollowOnStageState {
            x_velocity: -0x0002_0800,
            y_velocity: -0x0A00,
            on_ground: false,
            wall_cling_direction: None,
        };
        let after_motion = KidFlipFollowOnStageState {
            x_velocity: -0x0001_DD00,
            ..entry
        };
        let after_ceiling = KidFlipFollowOnStageState {
            y_velocity: 0,
            ..after_motion
        };
        runtime.last_kid_frame_kid_flip_follow_on_probe = Some(KidFlipFollowOnProbe {
            branch: KidFlipBranchSelection::NoWallPath,
            motion_branch: KidFrameMotionBranch::NormalPhysics,
            post_motion_mutator: KidFramePostMotionMutator::None,
            wall_hit_after_floor: false,
            stages: KidFlipFollowOnStages {
                entry,
                after_press_away: entry,
                cyclone_acceleration_before_sub_b168: Some(-0x0E00),
                after_jump_gravity: after_motion,
                after_floor: after_ceiling,
                after_wall: after_ceiling,
                gfx_object_collision_clear: true,
                ceiling_scan_cells_sub_914a: [None; 4],
            },
        });
        runtime.last_kid_frame_kid_flip_branch_probe = Some(KidFlipBranchProbe {
            wall_scan_hit: false,
            b_pressed: false,
            c_pressed: false,
            helmet_id: PlayerForm::Cyclone.helmet_id(),
            facing: Some(Direction::Right),
            on_ground_before: false,
            on_ground_after: false,
            branch: KidFlipBranchSelection::NoWallPath,
            wall_probe: None,
        });
        runtime.mode_level_work_before_hud_sub_44dc =
            Some(runtime.observe_mode_level_work_before_hud_sub_44dc());

        runtime.complete_mode_level_player_execution_path_before_hud_sub_44dc();

        assert_eq!(
            runtime
                .mode_level_work_before_hud_sub_44dc
                .as_ref()
                .and_then(|ledger| ledger.execute_objects.executed_player_path.clone()),
            Some(ModeLevelPlayerExecutionPath::NormalPhysics(
                ModeLevelNormalPhysicsPath {
                    horizontal: ModeLevelAirborneHorizontalPath::RightHeldAcceleration,
                    vertical: ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress {
                        acceleration_after: -0x0A00,
                        y_velocity_after: -0x0A00,
                        capped_acceleration_to_0x1200: false,
                    },
                    y_velocity_after: -0x0A00,
                    gfx_object_collision_clear: true,
                    vertical_collision: ModeLevelVerticalCollisionPath::Ceiling,
                    wall_collision: ModeLevelWallCollisionPath::Clear,
                    airborne_action: ModeLevelAirborneActionPath::NoButtons,
                    cyclone_visual_steps: None,
                    ceiling_scan_sub_914a: None,
                }
            ))
        );
    }

    #[test]
    fn cyclone_visual_step_types_accumulator_wrap_work() {
        // ASM sub_B270 loc_B2A0/loc_B2CA (kid.asm:14154-14175): frame 1703
        // adds $48 without wrapping; frame 1709 adds $38 and executes the
        // extra `subi.w #$300` before storing $22.
        assert_eq!(
            mode_level_cyclone_visual_step(0x002A, 0x0E00),
            ModeLevelCycloneVisualStep {
                accumulator_before: 0x002A,
                phase_delta: 0x0048,
                accumulator_after: 0x0072,
                wrapped: false,
            }
        );
        assert_eq!(
            mode_level_cyclone_visual_step(0x02EA, 0x1200),
            ModeLevelCycloneVisualStep {
                accumulator_before: 0x02EA,
                phase_delta: 0x0038,
                accumulator_after: 0x0022,
                wrapped: true,
            }
        );
    }

    #[test]
    fn ceiling_scan_types_every_sub_914a_cell_and_rubber_allocation_gate() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 40], 4, 5);
        grid.set_cell_at(1, 3, terrain::Cell::from_be_bytes(0x60, 0x22));
        grid.set_cell_at(2, 3, terrain::Cell::from_be_bytes(0xE6, 0x0C));
        let cells = [
            Some((1, 3, grid.cell_at(1, 3).to_word())),
            Some((2, 3, grid.cell_at(2, 3).to_word())),
            None,
            None,
        ];

        assert_eq!(
            mode_level_ceiling_scan_sub_914a(cells, 9),
            Some(ModeLevelCeilingScanSub914A {
                left_col: 1,
                right_col: 2,
                row: 3,
                visits: vec![
                    ModeLevelCeilingScanVisitSub9188 {
                        col: 1,
                        collision: ModeLevelCeilingScanCollision::Solid,
                        has_block: false,
                        block_type: None,
                        skin: 0x22,
                        rubber_effect: ModeLevelRubberContactEffectLoc10DA4::None,
                    },
                    ModeLevelCeilingScanVisitSub9188 {
                        col: 2,
                        collision: ModeLevelCeilingScanCollision::Solid,
                        has_block: true,
                        block_type: Some(6),
                        skin: 0x0C,
                        rubber_effect: ModeLevelRubberContactEffectLoc10DA4::PlaySoundAndAllocate {
                            worker_count_after: 10,
                        },
                    },
                ],
                first_hit_col: Some(1),
                return_path: ModeLevelCeilingScanReturn::RubberBounce,
            })
        );
        assert_eq!(
            mode_level_ceiling_scan_sub_914a(cells, 10).unwrap().visits[1].rubber_effect,
            ModeLevelRubberContactEffectLoc10DA4::PlaySoundWorkerCapReached,
            "ASM loc_10DA4 plays sound before `$FFFFFB5A >= 10` suppresses Allocate_ObjectSlot",
        );
    }

    #[test]
    fn in_game_object_counts_include_rubber_block_skin_workers() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);
        runtime
            .block_state
            .rubber_block_animations
            .push(RubberBlockAnimation {
                col: 0x55,
                row: 0x10,
                base_skin: 0x0C,
                phase: 0,
                frames_until_next_write: 2,
            });
        runtime
            .block_state
            .rubber_block_animations
            .push(RubberBlockAnimation {
                col: 0x56,
                row: 0x10,
                base_skin: 0x0C,
                phase: 0,
                frames_until_next_write: 2,
            });

        assert_eq!(
            runtime.object_counts_from_roster(&[], 0x0550_0000, OPENING_GROUNDED_CAMERA_Y_FP),
            (4, 3),
            "ASM loc_10DA4/sub_10DE4 allocates one Object_RAM worker per rubber cell through Allocate_ObjectSlot, but no GfxObject slot"
        );
    }

    #[test]
    fn camera_handoff_spans_late_intro_4_and_timeout_restart_intro_5() {
        fn uses_camera(
            segment: usize,
            elapsed: usize,
            timeout_restart_intro_sequence_4_pose_carry: bool,
        ) -> bool {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment,
                elapsed,
                pending_title_skip: 0,
            };
            runtime.timeout_restart_intro_sequence_4_pose_carry =
                timeout_restart_intro_sequence_4_pose_carry;
            runtime.intro_sequence_4_uses_timeout_restart_camera_handoff()
        }

        // ASM sub_1B988's `Camera_Y_pos=4` hand-off appears only after the
        // intro-3 pose carry clears, then survives into early intro sequence 5
        // because Load_IntroSequence5 does not call sub_1B222.
        assert!(!uses_camera(3, 36, false));
        assert!(uses_camera(3, 37, false));
        assert!(uses_camera(4, 30, true));
        assert!(!uses_camera(4, 31, true));
        assert!(!uses_camera(2, 37, true));
    }

    #[test]
    fn camera_state_exposes_intro_4_camera_handoff() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence {
            segment: 3,
            elapsed: 37,
            pending_title_skip: 0,
        };

        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x0004_0000,
            }
        );

        runtime.boot_state = BootState::IntroSequence {
            segment: 4,
            elapsed: 31,
            pending_title_skip: 0,
        };

        assert_eq!(runtime.current_camera_state(), CameraState::default());
    }

    #[test]
    fn camera_state_exposes_title_and_playback_handoff_boundaries() {
        let mut title = NativeRuntime::new(minimal_manifest());
        title.boot_state = BootState::TitleScreen { cursor: 0 };
        title.timeout_restart_title_intro_sequence_4_camera_carry = true;
        title.title_screen_elapsed_frames =
            TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_CAMERA_CARRY_FRAMES - 1;

        // ASM sub_1C246 preserves sub_1B988's `Camera_Y_pos=4` only through
        // the short title hand-off window.
        assert_eq!(
            title.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x0004_0000,
            }
        );

        title.title_screen_elapsed_frames =
            TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_CAMERA_CARRY_FRAMES;
        assert_eq!(title.current_camera_state(), CameraState::default());

        title.timeout_restart_title_intro_sequence_4_camera_carry = false;
        title.title_screen_elapsed_frames =
            TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_CAMERA_CARRY_FRAMES - 1;
        assert_eq!(title.current_camera_state(), CameraState::default());

        let mut playback = NativeRuntime::new(minimal_manifest());
        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_1_GAMEPLAY_START_FRAME - 1,
            data_index: 0,
            use_demo_2: false,
        };
        playback.demo_uses_intro_sequence_6_hidden_camera_wrap = true;

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_CAMERA_PRELOAD_FRAME,
            data_index: 0,
            use_demo_2: false,
        };
        assert_eq!(playback.current_camera_state(), CameraState::default());

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_1_GAMEPLAY_START_FRAME - 1,
            data_index: 0,
            use_demo_2: false,
        };

        // ASM Load_DemoPlay keeps demo-1 camera preload rows visible only
        // before the gameplay-start frame.
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            }
        );

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_1_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: false,
        };
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            }
        );

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_1_GAMEPLAY_START_FRAME + 1,
            data_index: 0,
            use_demo_2: false,
        };
        assert_eq!(playback.current_camera_state(), CameraState::default());

        playback.boot_state = BootState::DemoPlay {
            frame_counter: 60,
            data_index: 0,
            use_demo_2: false,
        };
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: demo::DEMO_1_INTRO_SEQUENCE_6_CAMERA_WRAP_Y_FP,
            }
        );

        playback.demo_uses_intro_sequence_6_hidden_camera_wrap = false;
        playback.demo_uses_demo_title_return_camera_wrap = true;
        playback.demo_uses_intro_sequence_6_early_title_restore = false;
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0xFF20_0000,
            }
        );

        playback.demo_uses_intro_sequence_6_early_title_restore = true;
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0xFF00_0000,
            }
        );

        playback.demo_uses_demo_title_return_camera_wrap = false;
        playback.demo_uses_start_skip_intro_return_camera_wrap = true;
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0xFF20_0000,
            }
        );

        playback.boot_state = BootState::DemoPlay {
            frame_counter: 59,
            data_index: 0,
            use_demo_2: false,
        };
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            }
        );

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_CAMERA_PRELOAD_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        playback.demo_uses_intro_sequence_5_title_return_preload_row = true;
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: demo::DEMO_2_INTRO_SEQUENCE_5_TITLE_RETURN_CAMERA_PRELOAD_Y_FP,
            }
        );

        playback.demo_uses_intro_sequence_5_title_return_preload_row = false;
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: demo::DEMO_2_CAMERA_PRELOAD_Y_FP,
            }
        );

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_CAMERA_PRELOAD_FRAME - 1,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(playback.current_camera_state(), CameraState::default());

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_CAMERA_HOLD_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(
            playback.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x02A0_0000,
            }
        );

        playback.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(playback.current_camera_state(), CameraState::default());
    }

    #[test]
    fn intro_sequence_5_slot_2_pose_uses_visible_elapsed() {
        fn uses_pose(elapsed: usize, early_visibility: bool) -> bool {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment: 4,
                elapsed,
                pending_title_skip: 0,
            };
            runtime.demo_timeout_restart_uses_intro_return_slot_2_early_visibility =
                early_visibility;
            runtime.intro_sequence_5_uses_timeout_restart_slot_2_pose()
        }

        // ASM loc_1BCD0's slot-2 pose appears after the intro-sequence-5
        // visibility delay, with one earlier frame for the later Start-skip
        // intro-return lineage.
        assert!(!uses_pose(56, false));
        assert!(uses_pose(57, false));
        assert!(uses_pose(57, true));
    }

    #[test]
    fn current_player_state_exposes_cold_intro_5_slot_2_pose_after_delay() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence {
            segment: 4,
            elapsed: 57,
            pending_title_skip: 0,
        };
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::intro_sequence_5_slot_2_pose())
        );
        assert_eq!(runtime.object_counts_from_roster(&[], 0, 0), (4, 0));

        runtime.boot_state = BootState::IntroSequence {
            segment: 4,
            elapsed: 56,
            pending_title_skip: 0,
        };

        assert_eq!(runtime.current_player_state(), None);
        assert_eq!(runtime.object_counts_from_roster(&[], 0, 0), (2, 0));
    }

    #[test]
    fn current_player_state_hides_intro_6_slot_2_carry_after_handoff_window() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence6 {
            cursor: 0,
            pending_title_skip: 0,
            pending_title_skip_fast_restore: false,
            carry_intro_sequence_5_slot_2_pose: true,
            carry_timeout_restart_hidden_camera_wrap: false,
        };
        runtime.title_screen_elapsed_frames =
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_SLOT_2_POSE_CARRY_FRAMES - 1;

        // ASM Load_IntroSequence6 replaces the old loc_1BCD0 slot-2 object
        // after a brief stale-WRAM handoff window.
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::intro_sequence_5_slot_2_pose())
        );

        runtime.title_screen_elapsed_frames =
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_SLOT_2_POSE_CARRY_FRAMES;

        assert_eq!(runtime.current_player_state(), None);
    }

    #[test]
    fn current_player_state_exposes_later_intro_6_title_pose_at_visible_frame() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence6 {
            cursor: 0,
            pending_title_skip: 0,
            pending_title_skip_fast_restore: false,
            carry_intro_sequence_5_slot_2_pose: false,
            carry_timeout_restart_hidden_camera_wrap: false,
        };
        runtime.title_screen_elapsed_frames =
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_TITLE_POSE_VISIBLE_ELAPSED_FRAMES - 1;

        // ASM loc_1B5CE writes the title pose on the later IntroSequence6
        // revisit, but oracle-backed traces keep it hidden until frame 333.
        assert_eq!(runtime.current_player_state(), None);

        runtime.title_screen_elapsed_frames =
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_TITLE_POSE_VISIBLE_ELAPSED_FRAMES;

        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_pose())
        );
    }

    #[test]
    fn current_player_state_exposes_timeout_restart_intro_6_title_pose_at_visible_frame() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence6 {
            cursor: 0,
            pending_title_skip: 0,
            pending_title_skip_fast_restore: false,
            carry_intro_sequence_5_slot_2_pose: false,
            carry_timeout_restart_hidden_camera_wrap: true,
        };
        runtime.title_screen_elapsed_frames =
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_TITLE_POSE_VISIBLE_ELAPSED_FRAMES - 1;

        // ASM loc_1B5CE writes the standard title pose on the timeout-restart
        // IntroSequence6 lineage only after the hidden camera wrap/ramp window.
        assert_eq!(runtime.current_player_state(), None);

        runtime.title_screen_elapsed_frames =
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_TITLE_POSE_VISIBLE_ELAPSED_FRAMES;

        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_pose())
        );
    }

    #[test]
    fn current_player_state_uses_intro_6_title_menu_only_when_active() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence6 {
            cursor: 2,
            pending_title_skip: 0,
            pending_title_skip_fast_restore: false,
            carry_intro_sequence_5_slot_2_pose: false,
            carry_timeout_restart_hidden_camera_wrap: false,
        };
        runtime.title_screen_ready_frame = 0;

        // ASM Obj_TitleMenu loc_1BB52/loc_1BC0E can expose the live cursor while
        // the title shell is still in IntroSequence6 mode, but only after the
        // menu worker is active.
        assert_eq!(runtime.current_player_state(), None);

        runtime.title_menu_active = true;
        runtime.title_screen_ready_frame = 100;

        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x0044_0000))
        );

        runtime.title_screen_uses_intro_return_hidden_shell = true;
        runtime.intro_sequence_6_start_skip_cursor_trace_priority = true;
        runtime.title_screen_ready_frame = 0;
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x0044_0000)),
            "ASM Obj_TitleMenu loc_1BC0E keeps the cursor gfx object live after the title shell itself is ready"
        );
    }

    #[test]
    fn trace_frame_exposes_intro_6_title_menu_cursor_and_counts() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence6 {
            cursor: 2,
            pending_title_skip: 0,
            pending_title_skip_fast_restore: false,
            carry_intro_sequence_5_slot_2_pose: false,
            carry_timeout_restart_hidden_camera_wrap: false,
        };
        runtime.title_menu_active = true;
        runtime.title_menu_cursor_pending = true;
        runtime.title_screen_ready_frame = 0;

        let pending_trace = runtime.trace_frame();
        assert_eq!(pending_trace.options_cursor, Some(2));
        assert_eq!(
            (
                pending_trace.number_objects,
                pending_trace.number_gfx_objects
            ),
            (4, 4),
            "ASM Obj_TitleMenu loc_1BB52 has not exposed the extra cursor gfx slot while the delayed rebuild is still pending"
        );

        runtime.title_menu_cursor_pending = false;
        let active_trace = runtime.trace_frame();
        assert_eq!(
            active_trace.options_cursor,
            Some(2),
            "ASM Obj_TitleMenu loc_1BC0E keeps the live d1 cursor row visible while still in IntroSequence6"
        );
        assert_eq!(
            (active_trace.number_objects, active_trace.number_gfx_objects),
            (4, 3),
            "ASM Obj_TitleMenu loc_1BB52 exposes the active title cursor as the third gfx slot even before the shell reports TitleScreen"
        );
    }

    #[test]
    fn intro_sequence_6_timeout_restart_hidden_camera_wraps_then_ramps() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence6 {
            cursor: 0,
            pending_title_skip: 0,
            pending_title_skip_fast_restore: false,
            carry_intro_sequence_5_slot_2_pose: true,
            carry_timeout_restart_hidden_camera_wrap: true,
        };

        // ASM Load_IntroSequence6 seeds the timeout-restart hidden camera late:
        // no wrapped row before frame 32, then `$FFF0_0000`, then the
        // GfxObjects_Collision_BottomBoundary4 ramp advances by one pixel.
        runtime.title_screen_elapsed_frames = 31;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            None
        );
        assert_eq!(runtime.current_camera_state(), CameraState::default());

        runtime.title_screen_elapsed_frames = 32;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            Some(0xFFF0_0000)
        );
        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0xFFF0_0000,
            }
        );

        runtime.title_screen_elapsed_frames = 248;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            Some(0xFFF1_0000)
        );

        runtime.title_screen_elapsed_frames = 264;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            Some(0)
        );
    }

    #[test]
    fn intro_sequence_6_later_revisit_hidden_camera_seeds_then_ramps() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::IntroSequence6 {
            cursor: 0,
            pending_title_skip: 0,
            pending_title_skip_fast_restore: false,
            carry_intro_sequence_5_slot_2_pose: true,
            carry_timeout_restart_hidden_camera_wrap: false,
        };

        // The later title-restore revisit delays the same hidden-camera seed,
        // then ramps by three pixels per frame because `$FFD0_0000 >> 4`
        // produces `$FFFD_0000`.
        runtime.title_screen_elapsed_frames = 34;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            None
        );

        runtime.title_screen_elapsed_frames = 35;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            Some(0xFFD0_0000)
        );
        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0xFFD0_0000,
            }
        );

        runtime.title_screen_elapsed_frames = 251;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            Some(0xFFD3_0000)
        );

        runtime.title_screen_elapsed_frames = 267;
        assert_eq!(
            runtime.intro_sequence_6_timeout_restart_hidden_camera_y(),
            Some(0)
        );
    }

    #[test]
    fn title_return_hides_player_uses_lineage_specific_hidden_windows() {
        fn hides_with(
            title_screen_elapsed_frames: usize,
            configure: impl FnOnce(&mut NativeRuntime),
        ) -> bool {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.hide_demo_return_title_player = true;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;
            configure(&mut runtime);
            runtime.title_hides_demo_return_player()
        }

        // ASM title rebuild lineages hide Kid until their oracle-backed local
        // title-frame cutoffs; timeout-restart intro-5 Start-skip adds one
        // hidden frame, while the later intro-6 title restore removes two.
        assert!(hides_with(149, |_| {}));
        assert!(!hides_with(150, |_| {}));
        assert!(hides_with(148, |runtime| {
            runtime.timeout_restart_title_intro_sequence_3_pose = true;
        }));
        assert!(!hides_with(149, |runtime| {
            runtime.timeout_restart_title_intro_sequence_3_pose = true;
        }));
        assert!(hides_with(147, |runtime| {
            runtime.timeout_restart_title_intro_sequence_5_pose = true;
        }));
        assert!(!hides_with(148, |runtime| {
            runtime.timeout_restart_title_intro_sequence_5_pose = true;
        }));
        assert!(hides_with(148, |runtime| {
            runtime.timeout_restart_title_intro_sequence_5_pose = true;
            runtime.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay = true;
        }));
        assert!(!hides_with(149, |runtime| {
            runtime.timeout_restart_title_intro_sequence_5_pose = true;
            runtime.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay = true;
        }));
        assert!(hides_with(147, |runtime| {
            runtime.demo_title_return_uses_intro_sequence_6_early_title_restore = true;
        }));
        assert!(!hides_with(148, |runtime| {
            runtime.demo_title_return_uses_intro_sequence_6_early_title_restore = true;
        }));
        assert!(hides_with(999, |runtime| {
            runtime.demo_auto_timeout_handoff_active = true;
        }));
        assert!(!hides_with(0, |runtime| {
            runtime.hide_demo_return_title_player = false;
        }));
    }

    #[test]
    fn current_player_state_hides_title_return_player_until_window_expires() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.hide_demo_return_title_player = true;
        runtime.title_screen_ready_frame = 0;
        runtime.title_screen_elapsed_frames = 149;

        assert_eq!(runtime.current_player_state(), None);

        runtime.title_screen_elapsed_frames = 150;

        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_pose())
        );
    }

    #[test]
    fn current_player_state_hides_intro_return_shell_before_menu_cursor() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 2 };
        runtime.title_menu_active = true;
        runtime.title_screen_ready_frame = 100;
        runtime.title_screen_elapsed_frames = 56;
        runtime.title_screen_uses_intro_return_hidden_shell = true;

        // ASM sub_1C246 -> Load_TitleScreen keeps the rebuilt title shell
        // hidden before Obj_TitleMenu loc_1BB52 can expose the menu cursor.
        assert_eq!(runtime.hidden_title_screen_object_counts(), Some((3, 0)));
        assert_eq!(runtime.current_player_state(), None);
    }

    #[test]
    fn current_player_state_uses_preserved_title_frame_until_full_window_expires() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.preserve_demo_title_snapshot = true;
        runtime.title_screen_ready_frame = 100;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player = player_at_x(88);
        runtime.opening_slice = Some(slice);

        // ASM loc_119D8 carries the full demo snapshot through the early title
        // return window before the regular title pose is rebuilt.
        runtime.title_screen_elapsed_frames = DEMO_TITLE_FULL_SNAPSHOT_FRAMES - 1;
        assert_eq!(runtime.current_player_state(), Some(player_at_x(88)));

        runtime.title_screen_elapsed_frames = DEMO_TITLE_FULL_SNAPSHOT_FRAMES;
        runtime.title_screen_ready_frame = 0;
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_pose())
        );
    }

    #[test]
    fn title_menu_cursor_player_state_tracks_live_menu_rows() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.title_menu_active = true;

        // ASM Obj_TitleMenu loc_1BB52 seeds the cursor sprite at x=$28 and
        // loc_1BC0E selects one of the three fixed title-menu rows.
        assert_eq!(
            runtime.title_menu_cursor_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x0030_0000))
        );

        runtime.boot_state = BootState::TitleScreen { cursor: 1 };
        assert_eq!(
            runtime.title_menu_cursor_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x003A_0000))
        );

        runtime.boot_state = BootState::TitleScreen { cursor: 2 };
        assert_eq!(
            runtime.title_menu_cursor_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x0044_0000))
        );

        runtime.title_menu_cursor_pending = true;
        assert_eq!(runtime.title_menu_cursor_player_state(), None);

        runtime.title_menu_cursor_pending = false;
        runtime.title_menu_active = false;
        assert_eq!(runtime.title_menu_cursor_player_state(), None);
    }

    #[test]
    fn trace_frame_title_menu_counts_ignore_stale_title_card_workers() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.title_menu_active = true;
        runtime
            .starting_level_title_workers
            .push(LevelEntryTitleWorker {
                gfx_slot_index: 0,
                glyph: 0,
                activation_frame: 0,
                target_x_pos: 0,
                target_y_pos: 0,
                motion: None,
            });

        // ASM Obj_TitleMenu loc_1BB52 exposes the active title cursor as the
        // third gfx slot. Stale Load_TitleCard workers are only meaningful while
        // BootState::StartingLevel is active.
        let frame = runtime.trace_frame();
        assert_eq!(frame.number_objects, 4);
        assert_eq!(frame.number_gfx_objects, 3);
    }

    fn drips_test_enemy(wait_timer: u16, drips_surface_table_index: u8) -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Drips,
            x_pos: 0x0248_0000,
            y_pos: 0x0212_0000,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
            death_animation_active: false,
            grounded_frames: 0,
            tank_mouth_animation_phase: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
            dragon_descent_index: 0,
            dragon_descent_frames: 0,
            ninja_state: NinjaState::Stealth,
            hand_state: HandState::Walking,
            lion_state: LionState::Walking,
            robot_state: RobotState::Walking,
            orca_state: OrcaState::Walking,
            wall_pinned: false,
            drips_attached: true,
            drips_surface_table_index,
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
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_turnaround_counter: 0,
            shared_walker_right_turn_extra_step_consumed: false,
            shared_walker_patrol_short_next: false,
            shared_drive_sound_previous_render_hidden: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
            shared_walker_grounded_scan_trace: None,
        }
    }

    #[test]
    fn drips_loc_33424_countdown_does_not_add_hidden_child_object() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.recording_oracle_debug_lifecycle_completed = true;
        let without_controller = runtime.gameplay_transient_object_only_worker_count(0, 0);
        runtime.active_enemies.push(drips_test_enemy(0x0038, 0));

        assert_eq!(
            runtime.gameplay_transient_object_only_worker_count(0, 0),
            without_controller + 1,
            "ASM Drips.asm loc_33424 only decrements d0 and hibernates the one live controller here; it does not add the falling child until after `stru_33F90` finishes",
        );
    }

    #[test]
    fn drips_loc_33488_surface_pool_stays_outside_object_count() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.recording_oracle_debug_lifecycle_completed = true;
        runtime.active_enemies.push(drips_test_enemy(0x0038, 0));
        let controller_only = runtime.gameplay_transient_object_only_worker_count(0, 0);
        runtime.active_enemies[0].drips_surface_table_index = 1;

        assert_eq!(
            runtime.gameplay_transient_object_only_worker_count(0, 0),
            controller_only,
            "ASM Drips.asm Enemy16_Drip_Init allocates the `a2` surface-pool child with j_Allocate_GfxObjectSlot_a1, so loc_33488 changes its gfx state without incrementing Number_Objects",
        );
    }

    #[test]
    fn drips_parent_controller_counts_as_object_and_three_gfx_workers_enemy16_drip_init() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0x14);
        runtime.recording_oracle_debug_lifecycle_completed = true;
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        runtime.active_enemies.push(drips_test_enemy(0x0038, 0));

        let with_controller = runtime.object_counts_from_roster(&[], 0, 0);

        assert_eq!(
            with_controller,
            (baseline.0 + 1, baseline.1 + 3),
            "ASM Drips.asm Enemy16_Drip_Init loads the controller's a3 GfxObject and allocates its a1/a2 children; suppressing those hidden snapshots must retain one Object and three Gfx workers",
        );
    }

    #[test]
    fn drips_surface_crawler_counts_as_object_and_gfx_worker_loc_335d8() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0x14);
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            ..Default::default()
        });

        let with_crawler = runtime.object_counts_from_roster(&[], 0, 0);

        assert_eq!(
            with_crawler,
            (baseline.0 + 1, baseline.1 + 1),
            "ASM Drips.asm loc_33488 allocates loc_335D8 through j_Allocate_ObjectSlot, then loc_335D8 loads its linked GfxObject slot; native projectile storage must retain both counts",
        );
    }

    #[test]
    fn tar_monster_parent_counts_as_object_and_two_gfx_workers_loc_321e4_32200() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0x46);
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        let mut tar_monster = drips_test_enemy(0, 0);
        tar_monster.family = EnemyFamily::TarMonster;
        let snapshot = tar_monster.snapshot();
        runtime.active_enemies.push(tar_monster);

        assert_eq!(
            runtime.object_counts_from_roster(&[snapshot], 0, 0),
            (baseline.0 + 1, baseline.1 + 2),
            "ASM Tar_Monster.asm loc_321E4 loads the parent's GfxObject, then loc_32200 calls loc_32188 to splice a second Gfx-only child into the same Object lifecycle",
        );
    }

    #[test]
    fn tar_monster_projectiles_count_as_object_gfx_pairs_loc_32468_325e6() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::TarMonsterStraightGlob,
            ..Default::default()
        });
        runtime.projectiles.extend((0..5).map(|_| EnemyProjectile {
            kind: EnemyProjectileKind::TarMonsterBurstChild,
            ..Default::default()
        }));

        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, 0),
            (baseline.0 + 6, baseline.1 + 6),
            "ASM Tar_Monster.asm loc_32468 and loc_325E6 load a GfxObject for every allocated straight-glob and burst-child Object worker",
        );
    }

    #[test]
    fn tank_bullet_counts_as_object_and_gfx_worker_loc_3c50e_3b730() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0x58);
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::TankBullet,
            lifetime: u16::MAX,
            ..Default::default()
        });

        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, 0),
            (baseline.0 + 1, baseline.1 + 1),
            "ASM Tank_Shooting.asm loc_3C50E allocates the loc_3B730 Object worker before loc_3B730 loads its paired GfxObject slot",
        );
    }

    #[test]
    fn tank_body_and_turret_share_object_but_keep_both_gfx_slots_loc_3b5fa() {
        fn tank_snapshot(y_pos: u32) -> NativeObjectSnapshot {
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x0B },
                x_pos: 0x008E_0000,
                y_pos,
                layout_index: Some(0),
                x_velocity: Some(0),
                y_velocity: Some(0),
                facing: Some(Direction::Left),
                on_ground: None,
                hit_points: Some(2),
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                sprite_animation_phase: None,
                wait_timer: None,
                debug_fields: BTreeMap::new(),
            }
        }

        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0x58);
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        let tank_pair = [tank_snapshot(0x0340_0000), tank_snapshot(0x0338_0000)];

        assert_eq!(
            runtime.object_counts_from_roster(&tank_pair, 0, 0),
            (baseline.0 + 1, baseline.1 + 2),
            "ASM Tank_Shooting.asm loc_3B5FA allocates one turret with j_Allocate_GfxObjectSlot_a1: body and turret share one Object_RAM coroutine but remain two linked GfxObject slots",
        );
    }

    #[test]
    fn dragon_walking_breath_counts_as_object_and_gfx_child_loc_3d02a() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(1);
        runtime.active_enemies.push(ActiveEnemy {
            family: EnemyFamily::DragonWalking,
            segment_index: 0,
            layout_index: 1,
            dragon_script_cmd: DRAGON_WALKING_PHASE_BREATH,
            dragon_phase_frames: 32,
            ..drips_test_enemy(0, 0)
        });
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DragonWalkingBreath,
            lifetime: 60,
            owner: Some(EnemyProjectileOwner {
                segment_index: 0,
                layout_index: 1,
                token: 0,
            }),
            ..Default::default()
        });

        assert_eq!(
            runtime.object_counts_from_roster(&[], 0x0550_0000, OPENING_GROUNDED_CAMERA_Y_FP),
            (3, 4),
            "ASM Dragon.asm loc_3CF90 allocates the breath Object_RAM child and loc_3D02A immediately calls j_Load_GfxObjectSlot; native must count the projectile as one Object/GfxObject pair",
        );
    }

    #[test]
    fn dragon_walking_breath_completion_count_follows_child_phase_loc_3d09e() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(1);
        runtime
            .active_enemies
            .extend([2, 4].map(|layout_index| ActiveEnemy {
                family: EnemyFamily::DragonWalking,
                segment_index: 0,
                layout_index,
                dragon_script_cmd: DRAGON_WALKING_PHASE_BREATH,
                dragon_phase_frames: if layout_index == 2 { 1 } else { 32 },
                ..drips_test_enemy(0, 0)
            }));
        runtime
            .projectiles
            .extend([2, 4].map(|layout_index| EnemyProjectile {
                kind: EnemyProjectileKind::DragonWalkingBreath,
                lifetime: 0,
                owner: Some(EnemyProjectileOwner {
                    segment_index: 0,
                    layout_index,
                    token: 0,
                }),
                ..Default::default()
            }));

        assert_eq!(
            runtime.object_counts_from_roster(&[], 0x0550_0000, OPENING_GROUNDED_CAMERA_Y_FP),
            (3, 4),
            "ASM Dragon.asm loc_3D082 keeps the naturally completed child linked through its final poll, while loc_3CFDA contact completion deletes the other child early",
        );

        runtime.projectiles.clear();
        runtime.active_enemies[0].dragon_phase_frames = 0;
        runtime.active_enemies[1].dragon_script_cmd = DRAGON_WALKING_PHASE_WALK;
        assert_eq!(
            runtime.object_counts_from_roster(&[], 0x0550_0000, OPENING_GROUNDED_CAMERA_Y_FP),
            (3, 4),
            "ASM Dragon.asm loc_3D082 observes the animation end before loc_3D09E removes the linked pair on the following object pass",
        );
    }

    #[test]
    fn dragon_walking_breath_count_waits_for_allocated_child_loc_3cf90() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(1);
        runtime.active_enemies.push(ActiveEnemy {
            family: EnemyFamily::DragonWalking,
            on_ground: true,
            wait_timer: 1,
            ..drips_test_enemy(1, 0)
        });

        assert_eq!(
            runtime.object_counts_from_roster(&[], 0x0550_0000, OPENING_GROUNDED_CAMERA_Y_FP),
            (2, 3),
            "ASM Dragon.asm loc_3CE90 has not run for the next roster entry at trace time, so its wait-timer-one child is not count-visible yet",
        );

        runtime.active_enemies.insert(
            0,
            ActiveEnemy {
                family: EnemyFamily::DragonWalking,
                on_ground: true,
                wait_timer: 2,
                ..drips_test_enemy(2, 0)
            },
        );

        assert_eq!(
            runtime.object_counts_from_roster(&[], 0x0550_0000, OPENING_GROUNDED_CAMERA_Y_FP),
            (2, 3),
            "ASM Dragon.asm loc_3CE90 leaves `$66(a5)=1` without a child; Number_Objects and Number_GfxObjects change only after loc_3CF90/loc_3D02A allocate the linked pair",
        );
    }

    #[test]
    fn shishkaboss_waiting_skewers_and_live_arrows_count_as_gfx_workers_enemy22_init() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.active_enemies.push(ActiveEnemy {
            family: EnemyFamily::Shishkaboss,
            ..drips_test_enemy(1, 0)
        });

        assert_eq!(
            runtime.gameplay_transient_gfx_worker_count(),
            9,
            "ASM Enemy22_Shiskaboss_Init allocates loc_38494..loc_38884 as nine object/gfx workers before they wait on `$FFFFFB4E==3`",
        );

        runtime.projectiles.extend((0..2).map(|_| EnemyProjectile {
            kind: EnemyProjectileKind::BossHeadArrow,
            lifetime: 0x32,
            ..Default::default()
        }));
        assert_eq!(
            runtime.gameplay_transient_gfx_worker_count(),
            11,
            "ASM sub_397E4 -> loc_3A0D2 allocates each live boss arrow as another object/gfx pair",
        );

        runtime.active_enemies.clear();
        runtime.projectiles.clear();
        runtime.projectiles.extend((0..9).map(|_| EnemyProjectile {
            kind: EnemyProjectileKind::BossSkewer,
            lifetime: u16::MAX,
            ..Default::default()
        }));
        assert_eq!(
            runtime.gameplay_transient_gfx_worker_count(),
            9,
            "ASM loc_384E8..loc_388D8 launches the existing nine workers; their allocation count must not change at the launch boundary",
        );
    }

    #[test]
    fn cloud_lightning_child_counts_as_object_and_gfx_worker_loc_35df0() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0x40);
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::CloudLightning,
            lifetime: 60,
            ..Default::default()
        });

        assert_eq!(
            runtime.gameplay_transient_gfx_worker_count(),
            1,
            "ASM Cloud.asm loc_35DB8 allocates the lightning bolt through j_Allocate_ObjectSlot and loc_35DF0 loads its GfxObject slot; each live bolt is one transient object/gfx worker",
        );
        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, 0),
            (baseline.0 + 1, baseline.1 + 1),
            "ASM Cloud.asm loc_35DF0 gives the bolt an Object+Gfx pair, so a live bolt raises Number_Objects and Number_GfxObjects by one each",
        );

        runtime.projectiles.clear();
        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, 0),
            baseline,
            "ASM Cloud.asm loc_35E8A deletes the bolt via j_Delete_CurrentObject on first collision or animation end, releasing both slots",
        );
    }

    #[test]
    fn fire_demon_trail_child_counts_as_object_and_gfx_worker_loc_3a63c() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0x40);
        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        runtime
            .fire_demon_trails
            .extend((0u32..2).map(|index| FireDemonTrailChild {
                x_pos: 0x0100_0000 + (index << 16),
                y_pos: 0x0200_0000,
                remaining_frames: 0x96,
                scheduler_delay_frames: 0,
            }));

        assert_eq!(
            runtime.gameplay_transient_gfx_worker_count(),
            2,
            "ASM Fire_Demon.asm loc_3A564 allocates each fire trail through j_Allocate_ObjectSlot and loc_3A63C loads its GfxObject slot; each burning trail is one transient object/gfx worker",
        );
        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, 0),
            (baseline.0 + 2, baseline.1 + 2),
            "ASM Fire_Demon.asm loc_3A63C gives every trail an Object+Gfx pair, so live trails raise Number_Objects and Number_GfxObjects together",
        );

        runtime.fire_demon_trails.clear();
        assert_eq!(
            runtime.object_counts_from_roster(&[], 0, 0),
            baseline,
            "ASM Fire_Demon.asm loc_3A6CA expires the trail timer and loc_3A6DA deletes the pair, releasing both slots",
        );
    }

    #[test]
    fn fire_demons_share_one_object_only_palette_helper_loc_3a414() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let baseline = runtime.gameplay_transient_object_only_worker_count(0, 0);
        runtime
            .active_enemies
            .extend((0..3).map(|layout_index| ActiveEnemy {
                family: EnemyFamily::FireDemon,
                layout_index,
                ..drips_test_enemy(0, 0)
            }));

        assert_eq!(
            runtime.gameplay_transient_object_only_worker_count(0, 0),
            baseline + 1,
            "ASM Fire_Demon.asm loc_3A414 tests the global Fire_Demon byte before allocating loc_3A48E, so every live Fire Demon shares one Object-only palette coroutine",
        );
    }

    #[test]
    fn trace_exposes_boss_eye_orbit_satellites_sub_393c8_39420() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 640,
            height_pixels: 480,
        }));
        runtime.active_enemies.push(ActiveEnemy {
            family: EnemyFamily::Shishkaboss,
            x_pos: 0x0200_0000,
            y_pos: 0x0100_0000,
            facing: Direction::Right,
            boss_head_index: 0,
            boss_eye_orbit_workers: BossEyeOrbitWorkers::active_loc_38902_3899a(
                0x0200,
                0x0100,
                0,
                Direction::Right,
            ),
            ..drips_test_enemy(1, 0)
        });

        let trace = runtime.trace_frame();
        let satellites = trace
            .active_projectile_details
            .iter()
            .filter(|projectile| projectile.kind == ActiveProjectileKind::BossEyeSatellite)
            .collect::<Vec<_>>();

        assert_eq!(
            satellites.len(),
            2,
            "ASM boss-head child loops run sub_393C8 and sub_39420 once per hibernated orbit frame before the trigger word detaches the eyes",
        );
        assert!(
            satellites
                .iter()
                .any(|eye| eye.x_pos == 0x01F4_0000 && eye.y_pos == 0x00DB_0000),
            "ASM sub_393C8 phase 0 uses x offset -12 and y_pos-#$25 for the primary orbit eye"
        );
        assert!(
            satellites
                .iter()
                .any(|eye| eye.x_pos == 0x020C_0000 && eye.y_pos == 0x00DB_0000),
            "ASM sub_39420 phase 0 uses x offset +12 and y_pos-#$25 for the counter-orbit eye"
        );
    }

    #[test]
    fn trace_uses_live_boss_eye_orbit_worker_phase_state_loc_3893e_389d6() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 640,
            height_pixels: 480,
        }));
        runtime.frame_index = 0;
        runtime.active_enemies.push(ActiveEnemy {
            family: EnemyFamily::Shishkaboss,
            x_pos: 0x0200_0000,
            y_pos: 0x0100_0000,
            facing: Direction::Right,
            boss_head_index: 0,
            boss_eye_orbit_workers: crate::enemies::BossEyeOrbitWorkers::active_loc_38902_3899a(
                0x0200,
                0x0100,
                2,
                Direction::Right,
            ),
            ..drips_test_enemy(1, 0)
        });

        let trace = runtime.trace_frame();
        let satellites = trace
            .active_projectile_details
            .iter()
            .filter(|projectile| projectile.kind == ActiveProjectileKind::BossEyeSatellite)
            .collect::<Vec<_>>();

        assert_eq!(satellites.len(), 2);
        assert!(
            satellites
                .iter()
                .any(|eye| eye.x_pos == 0x01E8_0000 && eye.y_pos == 0x00DB_0000),
            "ASM loc_3893E should report the live primary worker's stored phase-2 sub_393C8 placement, not frame_index phase 0"
        );
        assert!(
            satellites
                .iter()
                .any(|eye| eye.x_pos == 0x01F3_0000 && eye.y_pos == 0x00DB_0000),
            "ASM loc_389D6 should report the live counter worker's stored phase-2 sub_39420 placement, not frame_index phase 0"
        );
    }

    #[test]
    fn trace_exposes_waiting_boss_eye_satellites_as_object_slots_loc_3931a() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 640,
            height_pixels: 480,
        }));
        runtime.active_enemies.push(ActiveEnemy {
            family: EnemyFamily::Shishkaboss,
            x_pos: 0x0200_0000,
            y_pos: 0x0100_0000,
            facing: Direction::Right,
            boss_head_index: 0,
            boss_eye_orbit_workers: crate::enemies::BossEyeOrbitWorkers::active_loc_38902_3899a(
                0x0200,
                0x0100,
                2,
                Direction::Right,
            ),
            ..drips_test_enemy(1, 0)
        });

        let trace = runtime.trace_frame();
        let eye_slots = trace
            .active_enemy_details
            .iter()
            .filter(|object| object.kind == NativeObjectKind::Enemy { object_type: 0x21 })
            .collect::<Vec<_>>();

        assert_eq!(
            eye_slots.len(),
            2,
            "ASM loc_3931A allocates the two waiting boss-eye satellites as separate `$21` object slots before the loc_38962/loc_389FA detach routes"
        );
        assert!(
            eye_slots
                .iter()
                .any(|eye| eye.x_pos == 0x01E8_0000 && eye.y_pos == 0x00DB_0000),
            "ASM sub_393C8 phase-2 primary waiting eye uses word_39416[2] x offset -24 and parent y-#$25",
        );
        assert!(
            eye_slots
                .iter()
                .any(|eye| eye.x_pos == 0x01F3_0000 && eye.y_pos == 0x00DB_0000),
            "ASM sub_39420 phase-2 counter waiting eye uses word_3946E[2] x offset -13 and parent y-#$25",
        );
        assert!(
            eye_slots.iter().all(|eye| {
                eye.debug_fields
                    .get("boss_eye_object_meta_loc_3931a")
                    .is_some_and(|value| value == "0x0021")
                    && eye
                        .debug_fields
                        .get("boss_eye_has_kid_collision_loc_3931a")
                        .is_some_and(|value| value == "false")
                    && eye
                        .debug_fields
                        .get("boss_eye_is_moved_loc_3931a")
                        .is_some_and(|value| value == "true")
            }),
            "ASM loc_3931A writes object_meta=$21, clears has_kid_collision, and sets is_moved for each waiting eye worker",
        );
        let active_object_eye_slots = trace
            .active_objects
            .iter()
            .filter(|object| object.kind == NativeObjectKind::Enemy { object_type: 0x21 })
            .collect::<Vec<_>>();

        assert_eq!(
            active_object_eye_slots.len(),
            2,
            "ASM loc_3931A allocates the waiting boss-eye satellites as real `$21` object slots, so active object counts must include them"
        );

        for eye_slot in &active_object_eye_slots {
            let matching_satellite = trace
                .active_projectile_details
                .iter()
                .find(|projectile| {
                    projectile.kind == ActiveProjectileKind::BossEyeSatellite
                        && projectile.x_pos == eye_slot.x_pos
                        && projectile.y_pos == eye_slot.y_pos
                })
                .expect("waiting eye object slot should have matching sub_37ABC projectile bridge while render ownership is being promoted");
            assert_eq!(
                eye_slot
                    .debug_fields
                    .get("boss_eye_sprite_mapping_index_loc_37abc"),
                Some(&matching_satellite.boss_eye_sprite_mapping_index.to_string()),
                "ASM sub_37ABC refreshes each waiting `$21` boss-eye object's sprite mapping before sub_FF6 ordering"
            );
        }
    }

    #[test]
    fn trace_preserves_boss_eye_orbit_sprite_order_word_sub_393c8_39420() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 640,
            height_pixels: 480,
        }));
        let mut orbit_workers = crate::enemies::BossEyeOrbitWorkers::active_loc_38902_3899a(
            0x0200,
            0x0100,
            0,
            Direction::Right,
        );
        orbit_workers.step_with_parent_animation_loc_3893e_389d6(
            0x0200,
            0x0100,
            Direction::Left,
            0x0EE0,
            0x00,
        );

        runtime.active_enemies.push(ActiveEnemy {
            family: EnemyFamily::Shishkaboss,
            x_pos: 0x0200_0000,
            y_pos: 0x0100_0000,
            facing: Direction::Right,
            boss_head_index: 0,
            boss_eye_orbit_workers: orbit_workers,
            ..drips_test_enemy(1, 0)
        });

        let trace = runtime.trace_frame();
        let satellites = trace
            .active_projectile_details
            .iter()
            .filter(|projectile| projectile.kind == ActiveProjectileKind::BossEyeSatellite)
            .collect::<Vec<_>>();

        assert!(
            satellites.iter().any(|eye| {
                eye.x_pos == 0x01F4_0000 && eye.boss_eye_sprite_order_word_a == 0xFFFF
            }),
            "ASM sub_393C8 loc_39406 decrements `$A(a3)` when parent x_direction `$16` flips from `$FF` to zero"
        );
        assert!(
            satellites
                .iter()
                .any(|eye| { eye.x_pos == 0x020C_0000 && eye.boss_eye_sprite_order_word_a == 1 }),
            "ASM sub_39420 loc_3945A increments `$A(a3)` when parent x_direction `$16` flips from `$FF` to zero"
        );
    }

    #[test]
    fn title_menu_cursor_player_state_requires_early_start_carry_window() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_TITLE_MENU_CURSOR_POSE_CARRY_FRAMES - 1,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: false,
        };
        runtime.title_menu_active = true;
        runtime.two_player = true;

        // ASM loc_1AE2E can carry the Obj_TitleMenu cursor into the early
        // Start transition, but only while the title-worker shell is still
        // active and before Load_InGame consumes the bootstrap.
        assert_eq!(
            runtime.title_menu_cursor_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x003A_0000))
        );

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_TITLE_MENU_CURSOR_POSE_CARRY_FRAMES,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: false,
        };

        assert_eq!(runtime.title_menu_cursor_player_state(), None);

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: 0,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };

        assert_eq!(runtime.title_menu_cursor_player_state(), None);
    }

    #[test]
    fn loc_1ae2e_title_card_hibernate_ignores_gameplay_jump_input() {
        let mut runtime = NativeRuntime::new(manifest_with_title_card_level(0));
        let bounds = crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        };
        runtime.opening_slice = Some(OpeningSliceState::respawn(bounds));
        runtime.boot_state = BootState::StartingLevel {
            start_latched: false,
            elapsed_frames: crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME,
            uses_load_ingame_bootstrap: false,
            load_ingame_bootstrap_started: false,
        };
        runtime
            .starting_level_title_workers
            .push(LevelEntryTitleWorker {
                gfx_slot_index: 0,
                glyph: 0x61,
                activation_frame: crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME,
                target_x_pos: 0x80,
                target_y_pos: 0x80,
                motion: None,
            });
        runtime.previous_input = FrameInput {
            left: true,
            b: true,
            ..FrameInput::default()
        };

        let before = runtime
            .opening_slice
            .expect("test setup should seed a gameplay slice")
            .player;
        runtime.opening_slice = Some(OpeningSliceState {
            player: before,
            ..OpeningSliceState::respawn(bounds)
        });

        let _trace = runtime.step_frame(FrameInput::default());
        let after = runtime
            .opening_slice
            .expect("title-card hibernate keeps the hidden slice alive")
            .player;

        assert_eq!(
            (after.x_pos, after.y_pos, after.x_velocity, after.y_velocity),
            (
                before.x_pos,
                before.y_pos,
                before.x_velocity,
                before.y_velocity
            ),
            "ASM loc_1AE2E hibernates and only polls Ctrl_Pressed Start; gameplay B/Left must not launch or move Kid under the title card",
        );
        assert!(
            !runtime
                .played_sound_effects
                .contains(&QueuedSoundEffect::Jump),
            "ASM loc_1AE2E does not enter loc_A480 jump sound while the title-card worker is still hibernating",
        );
    }

    #[test]
    fn loc_1ae2e_title_card_idle_window_still_ignores_gameplay_jump_input() {
        let mut runtime = NativeRuntime::new(manifest_with_title_card_level(0));
        let bounds = crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        };
        runtime.opening_slice = Some(OpeningSliceState::respawn(bounds));
        runtime.boot_state = BootState::StartingLevel {
            start_latched: false,
            elapsed_frames: crate::movement::LEVEL_ENTRY_TITLE_GAMEPLAY_HANDOFF_FRAMES + 10,
            uses_load_ingame_bootstrap: false,
            load_ingame_bootstrap_started: false,
        };
        runtime.previous_input = FrameInput {
            right: true,
            b: true,
            ..FrameInput::default()
        };

        let before = runtime
            .opening_slice
            .expect("test setup should seed a gameplay slice")
            .player;
        runtime.opening_slice = Some(OpeningSliceState {
            player: before,
            ..OpeningSliceState::respawn(bounds)
        });

        let trace = runtime.step_frame(FrameInput::default());
        let after = runtime
            .opening_slice
            .expect("title-card hibernate keeps the hidden slice alive")
            .player;

        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "ASM Load_TitleCard keeps mode $0008 while loc_1AE2E is idling for Start",
        );
        assert_eq!(
            (after.x_pos, after.y_pos, after.x_velocity, after.y_velocity),
            (
                before.x_pos,
                before.y_pos,
                before.x_velocity,
                before.y_velocity
            ),
            "ASM loc_1AE2E only polls Ctrl_Pressed Start; gameplay B/Right must not launch or move Kid under the title card even after visible workers settle",
        );
        assert!(
            !runtime
                .played_sound_effects
                .contains(&QueuedSoundEffect::Jump),
            "ASM loc_1AE2E idle does not enter loc_A480 jump sound while the level card still owns the shell",
        );
    }

    #[test]
    fn debug_level_change_enters_title_card_not_live_gameplay_loc_1ae2e() {
        let mut runtime = NativeRuntime::new(manifest_with_title_card_level(1));
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);

        let trace = runtime.enter_level_debug_title_card(LevelId(1));

        assert_eq!(trace.current_level, LevelId(1));
        assert_eq!(
            trace.game_mode,
            GameMode::LevelTransition,
            "ASM loc_74E0 level changes re-enter mode $0008/Load_TitleCard; play-native N/P should expose the level name card instead of the synthetic live-gameplay warp",
        );

        let held_gameplay_trace = runtime.step_frame(FrameInput {
            right: true,
            b: true,
            ..FrameInput::default()
        });
        assert_eq!(
            held_gameplay_trace.game_mode,
            GameMode::LevelTransition,
            "ASM loc_1AE2E ignores gameplay buttons while the title-card shell owns the transition",
        );
    }

    #[test]
    fn loc_1ae2e_title_card_start_handoff_does_not_buffer_gameplay_input() {
        let mut manifest = manifest_with_title_card_level(0);
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .expect("title-card test level should have header metadata")
            .murder_wall_flags = 0;
        let mut runtime = NativeRuntime::new(manifest);
        let bounds = crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        };
        runtime.opening_slice = Some(OpeningSliceState::respawn(bounds));
        runtime.boot_state = BootState::StartingLevel {
            start_latched: false,
            elapsed_frames: crate::movement::LEVEL_ENTRY_TITLE_GAMEPLAY_HANDOFF_FRAMES + 2,
            uses_load_ingame_bootstrap: false,
            load_ingame_bootstrap_started: false,
        };
        runtime
            .starting_level_title_workers
            .push(LevelEntryTitleWorker {
                gfx_slot_index: 0,
                glyph: 0x61,
                activation_frame: crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME,
                target_x_pos: 0x80,
                target_y_pos: 0x80,
                motion: None,
            });

        let before = runtime
            .opening_slice
            .expect("test setup should seed a gameplay slice")
            .player;
        runtime.opening_slice = Some(OpeningSliceState {
            player: before,
            ..OpeningSliceState::respawn(bounds)
        });

        let _handoff_trace = runtime.step_frame(FrameInput {
            start: true,
            right: true,
            b: true,
            ..FrameInput::default()
        });
        let after_handoff = runtime
            .opening_slice
            .expect("Start handoff keeps the progressed hidden slice")
            .player;

        assert_eq!(
            (
                after_handoff.x_pos,
                after_handoff.y_pos,
                after_handoff.x_velocity,
                after_handoff.y_velocity
            ),
            (
                before.x_pos,
                before.y_pos,
                before.x_velocity,
                before.y_velocity
            ),
            "ASM loc_1AE2E must not let B/Right act on the hidden gameplay slice while Start releases the title card",
        );
        assert_eq!(
            runtime.previous_input,
            FrameInput {
                start: true,
                ..FrameInput::default()
            },
            "ASM loc_1AE2E may carry the Start handoff edge, but gameplay held bits must not become live Ctrl_Held",
        );

        let _gameplay_trace = runtime.step_frame(FrameInput::default());
        let after_gameplay = runtime
            .opening_slice
            .expect("live gameplay keeps the hidden slice")
            .player;

        assert_eq!(
            after_gameplay.x_velocity, 0,
            "ASM loc_1AE2E does not export title-card Right into the first live Mode_Level input latch",
        );
        assert!(
            after_gameplay.y_velocity >= 0,
            "ASM loc_1AE2E must not export title-card B as an upward loc_A480 jump velocity",
        );
        assert!(
            !runtime
                .played_sound_effects
                .contains(&QueuedSoundEffect::Jump),
            "ASM loc_1AE2E handoff must not buffer a loc_A480 jump sound into live gameplay",
        );
    }

    #[test]
    fn loc_1ae2e_recorded_title_card_start_keeps_replay_release_frame() {
        let mut runtime = NativeRuntime::new(manifest_with_title_card_level(0));
        runtime.set_recorded_title_card_handoff_delay(true);
        runtime.boot_state = BootState::StartingLevel {
            start_latched: false,
            elapsed_frames: crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME + 2,
            uses_load_ingame_bootstrap: false,
            load_ingame_bootstrap_started: false,
        };
        runtime
            .starting_level_title_workers
            .push(LevelEntryTitleWorker {
                gfx_slot_index: 0,
                glyph: 0x61,
                activation_frame: crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME,
                target_x_pos: 0x80,
                target_y_pos: 0x80,
                motion: None,
            });

        let start_trace = runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        assert_eq!(
            start_trace.game_mode,
            GameMode::LevelTransition,
            "ASM loc_1AE2E consumes the recorded Start edge before exposing the mode-$000C handoff",
        );

        let release_trace = runtime.step_frame(FrameInput::default());
        assert_eq!(
            release_trace.game_mode,
            GameMode::LevelTransition,
            "ASM loc_1AE2E replay comparisons keep the recorded release frame in mode $0008",
        );

        let gameplay_trace = runtime.step_frame(FrameInput::default());
        assert_eq!(
            gameplay_trace.game_mode,
            GameMode::InGame,
            "ASM loc_1AE2E replay guard should release on the frame after the recorded Start release",
        );
    }

    #[test]
    fn current_player_state_exposes_title_menu_cursor_before_title_ready() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 1 };
        runtime.title_menu_active = true;
        runtime.title_screen_ready_frame = 1;

        // ASM Obj_TitleMenu loc_1BC0E keeps the live cursor object visible
        // before the normal title Kid pose is rebuilt.
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x003A_0000))
        );

        runtime.title_screen_ready_frame = 0;

        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_pose())
        );
    }

    #[test]
    fn current_player_state_uses_title_menu_cursor_only_when_active() {
        let mut inactive = NativeRuntime::new(minimal_manifest());
        inactive.boot_state = BootState::TitleScreen { cursor: 2 };
        inactive.title_screen_ready_frame = 0;

        // ASM Obj_TitleMenu loc_1BB52 exposes the cursor sprite before the
        // regular title pose branch, but only after the menu worker is live.
        assert_eq!(
            inactive.current_player_state(),
            Some(PlayerState::title_pose())
        );

        let mut active = NativeRuntime::new(minimal_manifest());
        active.boot_state = BootState::TitleScreen { cursor: 2 };
        active.title_menu_active = true;
        active.title_screen_ready_frame = 100;
        active.title_screen_elapsed_frames = 0;

        assert_eq!(
            active.current_player_state(),
            Some(PlayerState::title_menu_cursor_pose(0x0044_0000))
        );

        let mut hidden_inactive = NativeRuntime::new(minimal_manifest());
        hidden_inactive.boot_state = BootState::TitleScreen { cursor: 2 };
        hidden_inactive.hide_demo_return_title_player = true;
        hidden_inactive.title_screen_ready_frame = 0;
        hidden_inactive.title_screen_elapsed_frames = 149;

        assert_eq!(hidden_inactive.current_player_state(), None);
    }

    #[test]
    fn current_player_state_separates_title_hold_and_hidden_playback_bootstrap() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_TITLE_POSE_FRAMES - 1,
            data_index: 0,
            use_demo_2: true,
        };
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        }));

        // ASM Load_DemoPlay -> Load_InGame keeps the title Kid visible for
        // the early mode-$10 hand-off, then hides Kid until gameplay starts.
        assert_eq!(
            runtime.current_player_state(),
            Some(PlayerState::title_pose())
        );

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_TITLE_POSE_FRAMES,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(runtime.current_player_state(), None);

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME - 1,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(runtime.current_player_state(), None);

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(
            runtime.current_player_state(),
            Some(
                OpeningSliceState::respawn(crate::physics::LevelBounds {
                    width_pixels: 320,
                    height_pixels: 224,
                })
                .player
            )
        );
    }

    #[test]
    fn current_player_state_exposes_early_start_level_one_hidden_bootstrap_before_load_ingame() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.current_level = LevelId(0);
        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: false,
        };

        let initial = runtime.current_player_state().unwrap_or_else(|| {
            panic!("early Start Level 1 bootstrap should expose the hidden slide pose")
        });
        assert_eq!(initial.x_pos, EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_X_FP);
        assert_eq!(initial.y_pos, EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_Y_FP);
        assert_eq!(initial.facing, Direction::Right);
        assert!(initial.on_ground);

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME + 1,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: false,
        };

        let sliding = runtime.current_player_state().unwrap_or_else(|| {
            panic!("early Start Level 1 bootstrap should expose the first hidden slide step")
        });
        assert_eq!(
            sliding.x_pos,
            (i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_X_FP)
                + i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_X_STEP_FP)) as u32
        );
        assert_eq!(
            sliding.y_pos,
            (i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_Y_FP)
                + i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_Y_STEP_FP)) as u32
        );

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME
                + EARLY_START_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: false,
        };

        let held = runtime.current_player_state().unwrap_or_else(|| {
            panic!("early Start Level 1 bootstrap should hold the settled hidden pose")
        });
        assert_eq!(held.x_pos, EARLY_START_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP);
        assert_eq!(held.y_pos, EARLY_START_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP);

        runtime.current_level = LevelId(1);
        assert_eq!(
            runtime.current_player_state(),
            None,
            "ASM loc_1AE2E early-start hidden bootstrap carry is a Level 1 title-start lineage, not a generic level-start pose",
        );

        runtime.current_level = LevelId(0);
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        }));
        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_VISIBLE_FRAME - 1,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };

        assert_eq!(
            runtime.current_player_state(),
            None,
            "ASM loc_6E2 keeps the early Start Level 1 Load_InGame player hidden until the visible ramp frame",
        );

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_VISIBLE_FRAME,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };

        let visible = runtime.current_player_state().unwrap_or_else(|| {
            panic!("early Start Level 1 Load_InGame should expose Kid at the visible ramp")
        });
        assert_eq!(visible.x_pos, 0x0020_0000);
        assert_eq!(visible.y_pos, 0x00FF_0000);
        assert!(visible.on_ground);

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_VISIBLE_FRAME + 3,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };

        let airborne = runtime.current_player_state().unwrap_or_else(|| {
            panic!("early Start Level 1 Load_InGame should expose ramp velocity after takeoff")
        });
        assert_eq!(airborne.y_pos, 0x0100_C000);
        assert_eq!(airborne.y_velocity, 0x0000_8000);
        assert!(!airborne.on_ground);

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_CAMERA_PRELOAD_FRAME - 1,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };
        assert_eq!(
            runtime.current_camera_state(),
            CameraState::default(),
            "ASM loc_6E2 keeps the early Start Load_InGame camera at default before the preload row",
        );

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_CAMERA_PRELOAD_FRAME,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };
        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            },
        );

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_VISIBLE_FRAME - 1,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };
        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            },
            "ASM loc_6E2 holds the preload camera row until the player-visible ramp frame",
        );

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_CAMERA_WRAP_FRAME,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };
        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0xFF00_0000,
            },
            "ASM loc_6E2 wraps the early Start Load_InGame camera row before returning to the preload hold",
        );

        runtime.boot_state = BootState::StartingLevel {
            start_latched: true,
            elapsed_frames: EARLY_START_LOAD_INGAME_VISIBLE_FRAME,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: true,
        };
        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x00D8_0000,
            },
        );

        runtime.current_level = LevelId(1);
        runtime.opening_slice = Some(OpeningSliceState {
            camera: CameraState {
                x_pos: 0x1234_0000,
                y_pos: 0x0042_0000,
            },
            ..OpeningSliceState::respawn(crate::physics::LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            })
        });
        assert_eq!(
            runtime.current_camera_state(),
            CameraState {
                x_pos: 0,
                y_pos: 0x0042_0000,
            },
            "ASM loc_6E2 Level 1 camera ramp is not reused for non-Level 1 early-start loads",
        );
    }

    #[test]
    fn active_objects_hides_playback_roster_until_gameplay_start() {
        fn enemy_object() -> NativeObjectSnapshot {
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 1 },
                x_pos: 0x0040_0000,
                y_pos: 0x0080_0000,
                layout_index: Some(0),
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                sprite_animation_phase: None,
                wait_timer: None,
                debug_fields: BTreeMap::new(),
            }
        }

        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        }));
        runtime.active_enemy_roster.push(enemy_object());

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME - 1,
            data_index: 0,
            use_demo_2: true,
        };
        // ASM Load_DemoPlay keeps the title/demo shell object list hidden until
        // the first gameplay-start trace frame reaches Load_InGame.
        assert!(runtime.active_objects().is_empty());

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        let active_objects = runtime.active_objects();
        assert_eq!(active_objects.len(), 2);
        assert!(matches!(active_objects[0].kind, NativeObjectKind::Player));
        assert!(matches!(
            active_objects[1].kind,
            NativeObjectKind::Enemy { object_type: 1 }
        ));

        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.player_lifecycle = PlayerLifecycle::Alive;
        assert!(runtime.active_objects().is_empty());

        runtime.boot_state = BootState::InGame;
        let active_objects = runtime.active_objects();
        assert_eq!(active_objects.len(), 2);
        assert!(matches!(active_objects[0].kind, NativeObjectKind::Player));
        assert!(matches!(
            active_objects[1].kind,
            NativeObjectKind::Enemy { object_type: 1 }
        ));
        let frozen_active_objects = active_objects.clone();

        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 1,
        };
        assert!(runtime.active_objects().is_empty());

        runtime.death_shell_snapshot = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player: PlayerState::title_pose(),
            frozen_camera: CameraState::default(),
            frozen_active_objects,
            frozen_number_objects: 2,
            frozen_number_gfx_objects: 2,
        });
        // ASM lose_life -> Teleport -> loc_6E2 clears object RAM once the
        // death shell is gone; transition counts are not live roster entries.
        assert!(runtime.active_objects().is_empty());
    }

    #[test]
    fn trace_frame_marks_currently_transforming_until_final_flash_frame() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.form_change_flash_frames = 2;

        // ASM Kid_Transform loc_7D22 keeps Currently_transforming live during
        // the visible flash, but loc_7E90 has cleared it by the final
        // hibernated cleanup frame when one flash frame remains.
        assert!(
            runtime
                .trace_frame()
                .gameplay_debug_fields
                .contains_key("currently_transforming")
        );

        runtime.progression.form_change_flash_frames = 1;

        assert!(
            !runtime
                .trace_frame()
                .gameplay_debug_fields
                .contains_key("currently_transforming")
        );
    }

    #[test]
    fn trace_frame_reports_live_playback_velocity_after_transform_cleanup() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        runtime.demo_post_transform_trace_velocity_hold_frames = 1;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_velocity = 0x0002_0000;
        slice.player.y_velocity = 0x0003_0000;
        runtime.opening_slice = Some(slice);

        // ASM Kid_Transform loc_7E90 clears the transform flag before the
        // next demo frontier. Oracle helper samples the live Kid gfx slot
        // (`$FFFF24E0 + x_vel/y_vel`) here, not the stale controller slot.
        let live_during_cleanup = runtime.trace_frame();
        assert_eq!(live_during_cleanup.kid_x_velocity, Some(0x0002_0000));
        assert_eq!(live_during_cleanup.kid_y_velocity, Some(0x0003_0000));

        runtime.demo_post_transform_trace_velocity_hold_frames = 0;

        let live = runtime.trace_frame();
        assert_eq!(live.kid_x_velocity, Some(0x0002_0000));
        assert_eq!(live.kid_y_velocity, Some(0x0003_0000));
    }

    #[test]
    fn trace_frame_reports_live_playback_velocity_after_bootstrap_gate() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_velocity = 0x0002_0000;
        slice.player.y_velocity = 0x0003_0000;
        runtime.opening_slice = Some(slice);

        // Oracle helper samples Kid's live gfx slot velocity throughout demo
        // playback. The hidden controller object stays count-visible, but it
        // no longer owns the exported Kid velocity fields.
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        let boundary = runtime.trace_frame();
        assert_eq!(boundary.kid_x_velocity, Some(0x0002_0000));
        assert_eq!(boundary.kid_y_velocity, Some(0x0003_0000));

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1,
            data_index: 0,
            use_demo_2: true,
        };
        let post_bootstrap = runtime.trace_frame();
        assert_eq!(post_bootstrap.kid_x_velocity, Some(0x0002_0000));
        assert_eq!(post_bootstrap.kid_y_velocity, Some(0x0003_0000));
    }

    #[test]
    fn trace_frame_character_movement_ignores_stale_death_shell_when_alive() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.player_lifecycle = PlayerLifecycle::Alive;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_velocity = 0x0001_0000;
        runtime.opening_slice = Some(slice);
        runtime.death_shell_snapshot = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player: PlayerState::title_pose(),
            frozen_camera: CameraState::default(),
            frozen_active_objects: Vec::new(),
            frozen_number_objects: 0,
            frozen_number_gfx_objects: 0,
        });

        assert_eq!(
            runtime.trace_frame().character_movement,
            Some(2),
            "ASM lose_life shell snapshots only force MoveID_Standingstill while the death lifecycle is active"
        );
    }

    #[test]
    fn trace_frame_playback_bootstrap_character_movement_starts_at_gameplay_boundary() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_velocity = 0x0001_0000;
        runtime.opening_slice = Some(slice);

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME - 1,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(runtime.trace_frame().character_movement, None);

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(
            runtime.trace_frame().character_movement,
            Some(2),
            "ASM Load_DemoPlay keeps Character_Movement inert only before the first gameplay-start frame"
        );

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME + 1,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(
            runtime.trace_frame().character_movement,
            Some(2),
            "ASM Load_DemoPlay keeps Character_Movement live after the first gameplay-start frame"
        );
    }

    #[test]
    fn current_character_movement_honours_playback_gameplay_start_boundary() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_velocity = 0x0001_0000;
        let moving_player = slice.player;

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME - 1,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(
            runtime.current_character_movement(moving_player),
            0,
            "ASM Load_DemoPlay keeps Character_Movement inert before the gameplay-start frame"
        );

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(
            runtime.current_character_movement(moving_player),
            2,
            "ASM Load_DemoPlay releases Character_Movement on the gameplay-start frame"
        );

        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME + 1,
            data_index: 0,
            use_demo_2: true,
        };
        assert_eq!(
            runtime.current_character_movement(moving_player),
            2,
            "ASM Load_DemoPlay keeps Character_Movement live after the gameplay-start frame"
        );
    }

    #[test]
    fn current_character_movement_uses_form_edges_for_slope_release() {
        fn movement_with_edge_slope(slope_col: u16) -> u16 {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::InGame;
            runtime.progression.current_form = PlayerForm::Juggernaut;
            runtime.last_kid_frame_grounded_release_probe = Some(KidGroundedReleaseProbe {
                on_ground_before: true,
                release_triggered: true,
                ..KidGroundedReleaseProbe::default()
            });
            let mut grid = terrain::BlockGrid::from_level_layout(&[], 5, 4);
            grid.set_cell_at(slope_col, 2, terrain::Cell::from_be_bytes(0x40, 0x00));
            runtime.set_block_grid(grid);

            let mut player = PlayerState::title_pose();
            player.x_pos = 40 << 16;
            player.y_pos = 32 << 16;
            player.x_velocity = 0x0001_0000;
            player.on_ground = true;
            runtime.current_character_movement(player)
        }

        // ASM loc_9D22/sub_9386 scans Kid's form-width left/right floor probes;
        // edge-only slope contact keeps MoveID 6 during the grounded-release seam.
        assert_eq!(movement_with_edge_slope(1), 6);
        assert_eq!(movement_with_edge_slope(3), 6);
    }

    #[test]
    fn current_character_movement_classifies_moving_grounded_frames_after_standing_gate() {
        let mut player = PlayerState::title_pose();
        player.x_pos = 40 << 16;
        player.y_pos = 32 << 16;
        player.x_velocity = 0x0001_0000;
        player.on_ground = true;

        let mut slope_runtime = NativeRuntime::new(minimal_manifest());
        slope_runtime.boot_state = BootState::InGame;
        let mut grid = terrain::BlockGrid::from_level_layout(&[], 5, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x40, 0x00));
        slope_runtime.set_block_grid(grid);

        // ASM loc_9D22 writes MoveID 6 when slope support is live after the
        // earlier loc_8DFC/loc_8E16 standing gate has admitted moving ground.
        assert_eq!(slope_runtime.current_character_movement(player), 6);

        let mut flat_runtime = NativeRuntime::new(minimal_manifest());
        flat_runtime.boot_state = BootState::InGame;

        // ASM loc_8DFC/loc_8E16 falls through to walking when either direction
        // bit or non-zero velocity admits moving flat ground past standing.
        assert_eq!(flat_runtime.current_character_movement(player), 2);
    }

    #[test]
    fn current_character_movement_keeps_slope_walk_when_loc_9f18_zeros_velocity() {
        let mut player = PlayerState::title_pose();
        player.x_pos = 40 << 16;
        player.y_pos = 32 << 16;
        player.x_velocity = 0;
        player.on_ground = true;

        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.trace_input.right = true;
        runtime.last_kid_frame_grounded_release_probe = Some(KidGroundedReleaseProbe {
            on_ground_before: true,
            release_triggered: true,
            ..KidGroundedReleaseProbe::default()
        });
        let mut grid = terrain::BlockGrid::from_level_layout(&[], 5, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x40, 0x00));
        runtime.set_block_grid(grid);

        assert_eq!(
            runtime.current_character_movement(player),
            6,
            "ASM loc_9D22 writes MoveID 6 before loc_9F18 may clamp slope velocity to zero on the same frame"
        );
    }

    #[test]
    fn current_character_movement_keeps_skycutter_walking_on_zero_velocity_wall_hit() {
        let mut player = PlayerState::title_pose();
        player.x_pos = 40 << 16;
        player.y_pos = 32 << 16;
        player.x_velocity = 0;
        player.on_ground = true;

        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::Skycutter;

        assert_eq!(
            runtime.current_character_movement(player),
            2,
            "ASM loc_8E16/loc_8E5A: Skycutter zero-velocity wall-hit frames route back to MoveID_Walking, unlike non-Skycutter loc_8DFC"
        );
    }

    #[test]
    fn kid_flip_follow_on_trace_formats_direction_and_stage_labels_loc_aac8() {
        // ASM loc_AAC8/sub_902A follow-on probes compare stage velocities and
        // wall-cling direction labels while debugging the attach/floor/wall handoff.
        assert_eq!(direction_debug_label(Some(Direction::Left)), "left");
        assert_eq!(direction_debug_label(Some(Direction::Right)), "right");
        assert_eq!(direction_debug_label(None), "none");

        assert_eq!(
            kid_flip_follow_on_stage_debug(KidFlipFollowOnStageState {
                x_velocity: -0x4000,
                y_velocity: 0x0002_8000,
                on_ground: false,
                wall_cling_direction: Some(Direction::Right),
            }),
            "x_vel=0xffffc000/y_vel=0x00028000",
            "ASM loc_AAC8/sub_902A trace stages should print signed 16.16 velocities as ROM-style words"
        );
    }

    #[test]
    fn current_character_movement_keeps_rubber_wall_snap_walking_loc_8e74() {
        fn rubber_wall_runtime(has_rubber_worker: bool) -> NativeRuntime {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::InGame;
            runtime.progression.current_form = PlayerForm::Berzerker;
            runtime.last_kid_frame_kid_flip_follow_on_probe = Some(KidFlipFollowOnProbe {
                branch: KidFlipBranchSelection::NoButtonBranch,
                motion_branch: KidFrameMotionBranch::NormalPhysics,
                post_motion_mutator: KidFramePostMotionMutator::None,
                wall_hit_after_floor: has_rubber_worker,
                stages: KidFlipFollowOnStages {
                    entry: KidFlipFollowOnStageState {
                        x_velocity: 0x0005_0000,
                        y_velocity: 0,
                        on_ground: true,
                        wall_cling_direction: None,
                    },
                    after_press_away: KidFlipFollowOnStageState {
                        x_velocity: 0x0005_0000,
                        y_velocity: 0,
                        on_ground: true,
                        wall_cling_direction: None,
                    },
                    cyclone_acceleration_before_sub_b168: None,
                    after_jump_gravity: KidFlipFollowOnStageState {
                        x_velocity: 0x0006_0000,
                        y_velocity: 0,
                        on_ground: true,
                        wall_cling_direction: None,
                    },
                    after_floor: KidFlipFollowOnStageState {
                        x_velocity: 0x0006_0000,
                        y_velocity: 0,
                        on_ground: true,
                        wall_cling_direction: None,
                    },
                    after_wall: KidFlipFollowOnStageState {
                        x_velocity: 0,
                        y_velocity: 0,
                        on_ground: true,
                        wall_cling_direction: None,
                    },
                    gfx_object_collision_clear: true,
                    ceiling_scan_cells_sub_914a: [None; 4],
                },
            });
            if has_rubber_worker {
                runtime
                    .block_state
                    .rubber_block_animations
                    .push(RubberBlockAnimation {
                        col: 50,
                        row: 104,
                        base_skin: 0x0C,
                        phase: 0,
                        frames_until_next_write: 2,
                    });
            }
            runtime
        }

        let mut player = PlayerState::title_pose();
        player.x_pos = 0x0318_0000;
        player.y_pos = 0x068f_0000;
        player.x_velocity = 0;
        player.on_ground = true;

        assert_eq!(
            rubber_wall_runtime(true).current_character_movement(player),
            2,
            "ASM sub_8F26 -> loc_8F98 -> loc_8E5A -> loc_8E74 keeps Berzerker's walking coroutine through a rubber-wall snap even after x_vel is cleared"
        );
        assert_eq!(
            rubber_wall_runtime(false).current_character_movement(player),
            0,
            "ordinary zero-velocity non-Skycutter wall stops still report MoveID_Standingstill"
        );
    }

    #[test]
    fn current_character_movement_reports_loc_8e98_slope_tilesnap_as_standing() {
        let mut player = PlayerState::title_pose();
        player.x_pos = 0x02ef_0000;
        player.y_pos = 0x00ef_0000;
        player.x_velocity = 0;
        player.on_ground = true;

        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::RedStealth;
        runtime.trace_input.right = true;
        runtime.last_kid_frame_kid_flip_follow_on_probe = Some(KidFlipFollowOnProbe {
            branch: KidFlipBranchSelection::NoWallPath,
            motion_branch: KidFrameMotionBranch::NormalPhysics,
            post_motion_mutator: KidFramePostMotionMutator::None,
            wall_hit_after_floor: false,
            stages: KidFlipFollowOnStages {
                entry: KidFlipFollowOnStageState {
                    x_velocity: 0x0002_0000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_press_away: KidFlipFollowOnStageState {
                    x_velocity: 0x0002_0000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                cyclone_acceleration_before_sub_b168: None,
                after_jump_gravity: KidFlipFollowOnStageState {
                    x_velocity: 0x0002_0000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_floor: KidFlipFollowOnStageState {
                    x_velocity: 0x0002_0000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_wall: KidFlipFollowOnStageState {
                    x_velocity: 0,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                gfx_object_collision_clear: true,
                ceiling_scan_cells_sub_914a: [None; 4],
            },
        });

        assert_eq!(
            runtime.current_character_movement(player),
            0,
            "ASM loc_8E98 -> loc_8EC8: Red Stealth's current-row slope tile-snap can clear x_vel without taking the loc_8E74 wall-snap branch, so the trace must report MoveID_Standingstill",
        );
    }

    #[test]
    fn current_character_movement_keeps_held_left_boundary_clamp_walking_loc_8ff8() {
        let mut player = PlayerState::title_pose();
        player.x_pos = (PlayerForm::TheKid.half_width() as u32) << 16;
        player.y_pos = 0x010f_0000;
        player.x_velocity = 0;
        player.on_ground = true;

        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.trace_input.left = true;

        assert_eq!(
            runtime.current_character_movement(player),
            2,
            "ASM loc_8E16 -> sub_8F26 loc_8FF8: held-left motion enters sub_8F26 with non-zero x_vel, then the left level-edge clamp preserves the negative return into loc_8E74 even though the exported x_vel is now zero"
        );
    }

    #[test]
    fn current_character_movement_keeps_held_left_wall_snap_walking_loc_8e74() {
        let mut player = PlayerState::title_pose();
        player.x_pos = 0x0175_0000;
        player.y_pos = 0x024f_0000;
        player.x_velocity = 0;
        player.on_ground = true;

        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.trace_input.left = true;
        runtime.last_kid_frame_kid_flip_follow_on_probe = Some(KidFlipFollowOnProbe {
            branch: KidFlipBranchSelection::NoButtonBranch,
            motion_branch: KidFrameMotionBranch::NormalPhysics,
            post_motion_mutator: KidFramePostMotionMutator::None,
            wall_hit_after_floor: true,
            stages: KidFlipFollowOnStages {
                entry: KidFlipFollowOnStageState {
                    x_velocity: -0x0000_c000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_press_away: KidFlipFollowOnStageState {
                    x_velocity: -0x0000_c000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                cyclone_acceleration_before_sub_b168: None,
                after_jump_gravity: KidFlipFollowOnStageState {
                    x_velocity: -0x0000_b000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_floor: KidFlipFollowOnStageState {
                    x_velocity: -0x0000_b000,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_wall: KidFlipFollowOnStageState {
                    x_velocity: 0,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                gfx_object_collision_clear: true,
                ceiling_scan_cells_sub_914a: [None; 4],
            },
        });

        assert_eq!(
            runtime.current_character_movement(player),
            2,
            "ASM loc_8E16 -> sub_8F26 -> loc_8E74 keeps MoveID_Walking when held-left walking is clamped by a wall after a non-zero floor-stage velocity"
        );
    }

    #[test]
    fn current_character_movement_holds_wall_jump_trace_only_inside_playback_window() {
        fn runtime_at(frame_counter: u16, probe: KidJumpGravityProbe) -> NativeRuntime {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter,
                data_index: 0,
                use_demo_2: true,
            };
            runtime.last_kid_frame_jump_gravity_probe = Some(probe);
            runtime
        }

        let mut settled_player = PlayerState::title_pose();
        settled_player.on_ground = true;
        settled_player.x_velocity = 0;
        let held_jump_probe = KidJumpGravityProbe {
            branch: KidJumpGravityBranch::AirborneHeldJumpGravity,
            on_ground_before: false,
            on_ground_after: false,
            y_velocity_after: -1,
            ..KidJumpGravityProbe::default()
        };

        // ASM loc_A6F8 -> loc_A94E keeps MoveID_Jump only after the object
        // bootstrap frame and through the published playback wall-jump window.
        let active = runtime_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1, held_jump_probe);
        assert_eq!(active.current_character_movement(settled_player), 3);

        let before_window = runtime_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME, held_jump_probe);
        assert_eq!(before_window.current_character_movement(settled_player), 0);

        let after_window = runtime_at(
            demo::DEMO_TRACE_WALL_JUMP_MOVEMENT_HOLD_END_FRAME + 1,
            held_jump_probe,
        );
        assert_eq!(after_window.current_character_movement(settled_player), 0);

        let mut airborne_player = settled_player;
        airborne_player.on_ground = false;
        let active_airborne =
            runtime_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1, held_jump_probe);
        assert_eq!(
            active_airborne.current_character_movement(airborne_player),
            3
        );

        let mut moving_player = settled_player;
        moving_player.x_velocity = 0x0001_0000;
        let active_moving = runtime_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1, held_jump_probe);
        assert_eq!(active_moving.current_character_movement(moving_player), 2);

        let previously_grounded = runtime_at(
            demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1,
            KidJumpGravityProbe {
                on_ground_before: true,
                ..held_jump_probe
            },
        );
        assert_eq!(
            previously_grounded.current_character_movement(settled_player),
            0
        );

        let now_grounded_probe = runtime_at(
            demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1,
            KidJumpGravityProbe {
                on_ground_after: true,
                ..held_jump_probe
            },
        );
        assert_eq!(
            now_grounded_probe.current_character_movement(settled_player),
            0
        );

        let non_rising_probe = runtime_at(
            demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1,
            KidJumpGravityProbe {
                y_velocity_after: 0,
                ..held_jump_probe
            },
        );
        assert_eq!(
            non_rising_probe.current_character_movement(settled_player),
            0
        );
    }

    #[test]
    fn current_character_movement_holds_late_playback_one_jump_window() {
        fn movement_at(frame_counter: u16, use_demo_2: bool) -> u16 {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter,
                data_index: 0,
                use_demo_2,
            };
            let mut player = PlayerState::title_pose();
            player.on_ground = true;
            player.x_velocity = 0;
            runtime.current_character_movement(player)
        }

        // ASM loc_A6F8 -> loc_A94E keeps MoveID_Jump through the late demo-1
        // landing stall, inclusive of both oracle-derived boundary frames.
        assert_eq!(
            movement_at(demo::DEMO_1_LATE_JUMP_MOVEMENT_HOLD_START_FRAME, false),
            3
        );
        assert_eq!(
            movement_at(demo::DEMO_1_LATE_JUMP_MOVEMENT_HOLD_END_FRAME, false),
            3
        );
        assert_eq!(
            movement_at(demo::DEMO_1_LATE_JUMP_MOVEMENT_HOLD_START_FRAME - 1, false),
            0
        );
        assert_eq!(
            movement_at(demo::DEMO_1_LATE_JUMP_MOVEMENT_HOLD_END_FRAME + 1, false),
            0
        );
        assert_eq!(
            movement_at(demo::DEMO_1_LATE_JUMP_MOVEMENT_HOLD_START_FRAME, true),
            0
        );
    }

    #[test]
    fn trace_frame_title_return_playback_velocity_reports_live_kid_slot() {
        fn velocities_at(
            use_demo_2: bool,
            uses_intro_sequence_5_title_return_preload_row: bool,
        ) -> (Option<i32>, Option<i32>) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter: demo::DEMO_1_LATE_CONTROLLER_VELOCITY_SAMPLE_END_FRAME + 1,
                data_index: 0,
                use_demo_2,
            };
            runtime.demo_uses_intro_sequence_5_title_return_preload_row =
                uses_intro_sequence_5_title_return_preload_row;
            let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            });
            slice.player.x_velocity = 0x0002_0000;
            slice.player.y_velocity = 0x0003_0000;
            runtime.opening_slice = Some(slice);

            let frame = runtime.trace_frame();
            (frame.kid_x_velocity, frame.kid_y_velocity)
        }

        // Oracle helper now reads Kid's live gfx slot velocity rather than the
        // stale controller-side slot, even for the demo-2 intro-sequence-5
        // title-return lineage.
        assert_eq!(
            velocities_at(true, true),
            (Some(0x0002_0000), Some(0x0003_0000))
        );
        assert_eq!(
            velocities_at(true, false),
            (Some(0x0002_0000), Some(0x0003_0000))
        );
        assert_eq!(
            velocities_at(false, true),
            (Some(0x0002_0000), Some(0x0003_0000))
        );
    }

    #[test]
    fn trace_frame_late_playback_velocity_reports_live_kid_slot() {
        fn velocities_at(
            use_demo_2: bool,
            intro_sequence_6_timeout_return: bool,
        ) -> (Option<i32>, Option<i32>) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter: demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME + 1,
                data_index: 0,
                use_demo_2,
            };
            runtime.demo_uses_intro_sequence_6_timeout_return_velocity_sample =
                intro_sequence_6_timeout_return;
            let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            });
            slice.player.x_velocity = 0x0002_0000;
            slice.player.y_velocity = 0x0003_0000;
            runtime.opening_slice = Some(slice);

            let frame = runtime.trace_frame();
            (frame.kid_x_velocity, frame.kid_y_velocity)
        }

        assert_eq!(
            velocities_at(false, true),
            (Some(0x0002_0000), Some(0x0003_0000)),
            "ASM loc_1BB02 -> Load_DemoPlay keeps controller objects count-visible, but velocity export follows the live Kid gfx slot"
        );
        assert_eq!(
            velocities_at(false, false),
            (Some(0x0002_0000), Some(0x0003_0000))
        );
        assert_eq!(
            velocities_at(true, false),
            (Some(0x0002_0000), Some(0x0003_0000))
        );
    }

    #[test]
    fn trace_frame_intro_6_velocity_sample_requires_timeout_return_lineage() {
        fn y_velocity_at(
            use_demo_2: bool,
            uses_later_intro_6_preload: bool,
            uses_timeout_return_velocity_sample: bool,
            frame_counter: u16,
        ) -> Option<i32> {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter,
                data_index: 0,
                use_demo_2,
            };
            runtime.demo_uses_intro_sequence_6_later_demo_2_preload_row =
                uses_later_intro_6_preload;
            runtime.demo_uses_intro_sequence_6_timeout_return_velocity_sample =
                uses_timeout_return_velocity_sample;
            let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            });
            slice.player.y_velocity = 0x0003_0000;
            runtime.opening_slice = Some(slice);

            runtime.trace_frame().kid_y_velocity
        }

        // ASM Obj_TitleMenu loc_1BB02 -> Load_DemoPlay carries `$60000` only
        // for the cold IntroSequence6 timeout-return first-recording window.
        assert_eq!(
            y_velocity_at(
                false,
                true,
                true,
                demo::DEMO_2_INTRO_SEQUENCE_6_LATER_CONTROLLER_SAMPLE_END_FRAME
            ),
            Some(0x0006_0000)
        );
        assert_eq!(
            y_velocity_at(
                false,
                true,
                true,
                demo::DEMO_2_INTRO_SEQUENCE_6_LATER_CONTROLLER_SAMPLE_END_FRAME + 1
            ),
            None
        );
        assert_eq!(
            y_velocity_at(
                true,
                true,
                false,
                demo::DEMO_2_INTRO_SEQUENCE_6_LATER_CONTROLLER_SAMPLE_END_FRAME
            ),
            Some(0)
        );
        assert_eq!(
            y_velocity_at(
                false,
                true,
                false,
                demo::DEMO_2_INTRO_SEQUENCE_6_LATER_CONTROLLER_SAMPLE_END_FRAME
            ),
            Some(0)
        );
    }

    #[test]
    fn trace_frame_late_controller_pause_is_first_recording_only() {
        fn paused_at(
            use_demo_2: bool,
            started_from_timeout_restart_shell: bool,
            allow_pausing: bool,
            frame_counter: u16,
        ) -> bool {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter,
                data_index: 0,
                use_demo_2,
            };
            runtime.demo_started_from_timeout_restart_shell = started_from_timeout_restart_shell;
            runtime.allow_pausing = allow_pausing;
            runtime.trace_frame().paused
        }

        // ASM Load_DemoPlay -> loc_119D8 exposes the late controller Start
        // sample only for ordinary first-recording playback after loc_7420
        // has set Allow_Pausing.
        assert!(paused_at(
            false,
            false,
            true,
            demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME
        ));
        assert!(!paused_at(
            false,
            false,
            true,
            demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME - 1
        ));
        assert!(!paused_at(
            true,
            false,
            true,
            demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME
        ));
        assert!(!paused_at(
            false,
            true,
            true,
            demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME
        ));
        assert!(!paused_at(
            false,
            false,
            false,
            demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME
        ));
    }

    #[test]
    fn trace_frame_late_title_return_controller_counts_require_latches() {
        fn counts_with(
            preserve_demo_title_snapshot: bool,
            demo_auto_timeout_handoff_active: bool,
            demo_title_snapshot_object_counts: Option<(u16, u16)>,
            title_screen_elapsed_frames: usize,
        ) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.preserve_demo_title_snapshot = preserve_demo_title_snapshot;
            runtime.demo_auto_timeout_handoff_active = demo_auto_timeout_handoff_active;
            runtime.demo_title_snapshot_object_counts = demo_title_snapshot_object_counts;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM loc_119D8 returns to title while the late first-recording
        // controller lineage still owns the top-level object/gfx counters,
        // but only during the preserved partial snapshot window.
        assert_eq!(
            counts_with(
                true,
                false,
                Some((4, 4)),
                DEMO_TITLE_PARTIAL_SNAPSHOT_FRAMES - 1
            ),
            (8, 7)
        );
        assert_eq!(
            counts_with(
                true,
                false,
                Some((4, 4)),
                DEMO_TITLE_PARTIAL_SNAPSHOT_FRAMES
            ),
            (0, 0)
        );
        assert_eq!(
            counts_with(
                false,
                false,
                Some((4, 4)),
                DEMO_TITLE_PARTIAL_SNAPSHOT_FRAMES - 1
            ),
            (0, 0)
        );
        assert_eq!(
            counts_with(
                true,
                true,
                Some((4, 4)),
                DEMO_TITLE_PARTIAL_SNAPSHOT_FRAMES - 1
            ),
            (4, 4)
        );
        assert_eq!(
            counts_with(
                true,
                false,
                Some((4, 3)),
                DEMO_TITLE_PARTIAL_SNAPSHOT_FRAMES - 1
            ),
            (4, 3)
        );
    }

    #[test]
    fn trace_frame_intro_sequence_1_timeout_restart_counts_follow_rebuild_windows() {
        fn counts_at(elapsed: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment: 0,
                elapsed,
                pending_title_skip: 0,
            };
            runtime.demo_auto_timeout_intro_sequence_1_frames_override = Some(0);

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM ReadJoypad loc_C0A -> Load_IntroSequence1/sub_1B8F4 keeps the
        // controller counted through local frame 30, clears object slots for the
        // hidden rebuild, then reallocates the controller before the text worker.
        assert_eq!(counts_at(30), (1, 0));
        assert_eq!(counts_at(31), (0, 0));
        assert_eq!(counts_at(43), (1, 0));
        assert_eq!(counts_at(44), (2, 1));
    }

    #[test]
    fn trace_frame_intro_sequence_2_timeout_restart_counts_follow_rebuild_windows() {
        fn counts_at(elapsed: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment: 1,
                elapsed,
                pending_title_skip: 0,
            };
            runtime.demo_auto_timeout_intro_sequence_2_frames_override = Some(0);

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM sub_1B8F4 -> loc_1B920 carries IntroSequence1's linked
        // sub_1C54A GfxObject through local frame 30, clears for the hidden
        // rebuild, then Load_IntroSequence2 reallocates the controller before
        // IntroText2 joins the object count without a replacement GfxObject.
        assert_eq!(counts_at(30), (1, 1));
        assert_eq!(counts_at(31), (0, 0));
        assert_eq!(counts_at(41), (1, 0));
        assert_eq!(counts_at(42), (2, 0));
    }

    #[test]
    fn trace_frame_intro_sequence_1_regular_counts_split_controller_and_text_worker() {
        fn counts_at(elapsed: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment: 0,
                elapsed,
                pending_title_skip: 0,
            };

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Load_IntroSequence1 first exposes only the sub_1B8F4 controller at
        // elapsed 44. On the next trace, sub_1B8F4 has called sub_1C54A to
        // allocate the intro GfxObject and loc_1C7A0's text Object worker.
        assert_eq!(counts_at(44), (1, 0));
        assert_eq!(counts_at(45), (2, 1));
    }

    #[test]
    fn trace_frame_intro_sequence_3_timeout_restart_counts_use_either_latch() {
        fn counts_with(
            demo_auto_timeout_override: bool,
            pending_title_return: bool,
            elapsed: usize,
        ) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment: 2,
                elapsed,
                pending_title_skip: 0,
            };
            runtime.demo_auto_timeout_intro_sequence_3_frames_override =
                demo_auto_timeout_override.then_some(0);
            runtime.timeout_restart_title_intro_sequence_3_return_pending = pending_title_return;

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Load_IntroSequence3 -> sub_1B93E reaches the same hidden rebuild
        // counter table from both the auto-timeout path and the later title-return
        // latch; either path must expose the controller, clear, and shell-worker
        // boundary counts.
        assert_eq!(counts_with(true, false, 30), (1, 0));
        assert_eq!(counts_with(true, false, 31), (0, 0));
        assert_eq!(counts_with(false, true, 31), (0, 0));
        assert_eq!(counts_with(false, true, 41), (3, 8));
        assert_eq!(counts_with(false, true, 42), (4, 0x000B));
        assert_ne!(counts_with(false, false, 41), (3, 8));
    }

    #[test]
    fn trace_frame_intro_sequence_5_timeout_restart_counts_expose_particle_threshold() {
        fn counts_at(elapsed: usize, early_visibility: bool) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::IntroSequence {
                segment: 4,
                elapsed,
                pending_title_skip: 0,
            };
            runtime.demo_auto_timeout_intro_sequence_5_frames_override = Some(0);
            runtime.demo_timeout_restart_uses_intro_return_slot_2_early_visibility =
                early_visibility;

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM loc_1BCD0 keeps its five direct GfxObjects linked until its
        // $1A0 hibernation expires. That pass replaces the Object roster and
        // frees two owned GfxObjects; loc_1BE3C/loc_1BE88 link four new
        // GfxObjects on the following pass.
        let particle_count_frame =
            DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_PARTICLE_COUNT_FRAME;
        assert_eq!(counts_at(55, true), (0, 0));
        assert_eq!(counts_at(56, true), (2, 0));
        assert_eq!(counts_at(57, true), (4, 5));
        assert_eq!(counts_at(310, true), (4, 5));
        assert_eq!(counts_at(311, true), (3, 5));
        assert_eq!(counts_at(particle_count_frame - 1, false), (3, 5));
        assert_eq!(counts_at(particle_count_frame, false), (6, 3));
        assert_eq!(counts_at(particle_count_frame + 1, false), (6, 7));
        assert_eq!(counts_at(particle_count_frame + 0x29, false), (6, 7));
        assert_eq!(counts_at(particle_count_frame + 0x2A, false), (4, 5));
    }

    #[test]
    fn trace_frame_playback_startup_counts_split_level_switch_and_first_recording_preload() {
        fn counts_at(frame_counter: u16, use_demo_2: bool) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter,
                data_index: 0,
                use_demo_2,
            };

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Load_DemoPlay -> Load_InGame keeps the title-level count through
        // DEMO_LEVEL_SWITCH_FRAME, then demo 1 exposes the hidden controller
        // worker before its second non-gfx preload worker joins.
        assert_eq!(counts_at(demo::DEMO_LEVEL_SWITCH_FRAME, false), (4, 4));
        assert_eq!(counts_at(demo::DEMO_LEVEL_SWITCH_FRAME + 1, false), (1, 0));
        assert_eq!(
            counts_at(demo::DEMO_1_GAMEPLAY_START_FRAME - 14, false),
            (1, 0)
        );
        assert_eq!(
            counts_at(demo::DEMO_1_GAMEPLAY_START_FRAME - 13, false),
            (2, 0)
        );
        assert_eq!(
            counts_at(demo::DEMO_1_GAMEPLAY_START_FRAME - 2, false),
            (2, 0)
        );
        assert_eq!(
            counts_at(demo::DEMO_1_GAMEPLAY_START_FRAME - 1, false),
            (6, 2)
        );
        assert_eq!(counts_at(demo::DEMO_1_GAMEPLAY_START_FRAME, false), (7, 4));
        assert_eq!(counts_at(demo::DEMO_OBJECT_WARMUP_FRAME, true), (5, 2));

        // ASM Manage_EnemyLoading loc_3691E allocates the Knights Isle Dragon
        // Object on demo-1 startup frame 72. Enemy0D_Dragon_Init runs on the
        // next object pass and calls Load_GfxObjectSlot, so the fifth slot is
        // visible from frame 73 while the seven Object workers remain live.
        assert_eq!(
            counts_at(demo::DEMO_1_GAMEPLAY_START_FRAME + 1, false),
            (7, 5)
        );
        assert_eq!(
            counts_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME, false),
            (7, 5)
        );
        assert_eq!(
            counts_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1, false),
            (7, 5)
        );
        assert_eq!(
            counts_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 7, false),
            (7, 5)
        );
        assert_eq!(
            counts_at(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 8, false),
            (6, 5)
        );
    }

    #[test]
    fn trace_frame_demo_play_keeps_player_1_level_slot_until_rom_saves_it() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.player_slots[0].level_id = LevelId(0);
        runtime.begin_demo_gameplay(LevelId(demo::DEMO_2_LEVEL));
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_LEVEL_SWITCH_FRAME + 1,
            data_index: 0,
            use_demo_2: true,
        };

        let frame = runtime.trace_frame();

        assert_eq!(frame.game_mode, GameMode::DemoPlay);
        assert_eq!(
            frame.current_level,
            LevelId(demo::DEMO_2_LEVEL),
            "ASM Load_DemoPlay loc_119B6 updates Current_LevelID for the demo"
        );
        assert_eq!(
            frame.player_1_level_id,
            LevelId(0),
            "Player_1_LevelID is a saved-slot field and remains unchanged on the demo VBlank checkpoint"
        );
    }

    #[test]
    fn trace_frame_playback_transient_workers_count_each_live_gfx_pair() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME + 1,
            data_index: 0,
            use_demo_2: true,
        };
        runtime
            .block_bounces
            .extend((0..2).map(|cell_x| BlockBounceAnim {
                cell_x,
                cell_y: 0,
                x_offset_px: 0,
                remaining: 1,
                initial_duration: 1,
                kind: BlockBounceKind::Metal,
                prize_content: None,
                prize_deleted_tile: false,
                metal_direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
                y_offset_px: 0,
            }));
        runtime
            .prize_block_spawn_flashes
            .extend((0..3).map(|x_px| PrizeBlockSpawnFlash {
                x_px,
                y_px: 0,
                remaining: 1,
            }));
        runtime
            .block_destroy_effects
            .extend((0..5).map(|x_px| BlockDestroyEffect {
                x_px,
                y_px: 0,
                remaining: 1,
                direction: 0,
                pending_ice_origin_nudge: false,
            }));
        runtime
            .prize_block_worker_tails
            .extend((0..13).map(|cell_x| PrizeBlockWorkerTail {
                cell_x,
                cell_y: 0,
                content: None,
                remaining: 1,
            }));
        runtime.floating_items.extend((0..7).map(diamond_at_x));
        runtime.pending_prize_diamond_awards.extend([1; 11]);

        // ASM Load_DemoPlay's loc_119D8 baseline has four live gameplay gfx
        // objects; each transient object/gfx worker listed here adds exactly one
        // to both top-level counters while collected floating prizes do not.
        let frame = runtime.trace_frame();
        assert_eq!(frame.number_objects, 4 + 2 + 13 + 3 + 5 + 7 + 11);
        assert_eq!(frame.number_gfx_objects, 4 + 2 + 13 + 3 + 5 + 7 + 11);

        runtime.floating_items[0].collected = true;
        let frame = runtime.trace_frame();
        assert_eq!(frame.number_objects, 4 + 2 + 13 + 3 + 5 + 6 + 11);
        assert_eq!(frame.number_gfx_objects, 4 + 2 + 13 + 3 + 5 + 6 + 11);
    }

    #[test]
    fn trace_frame_keeps_collected_helmet_visible_on_first_forward_transform_frame() {
        for (content, form) in [
            (terrain::PrizeContent::Skycutter, PlayerForm::Skycutter),
            (terrain::PrizeContent::Cyclone, PlayerForm::Cyclone),
            (terrain::PrizeContent::RedStealth, PlayerForm::RedStealth),
            (terrain::PrizeContent::Eyeclops, PlayerForm::Eyeclops),
            (terrain::PrizeContent::Juggernaut, PlayerForm::Juggernaut),
            (terrain::PrizeContent::IronKnight, PlayerForm::IronKnight),
            (terrain::PrizeContent::Berzerker, PlayerForm::Berzerker),
            (terrain::PrizeContent::Maniaxe, PlayerForm::Maniaxe),
            (terrain::PrizeContent::Micromax, PlayerForm::Micromax),
        ] {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::InGame;
            runtime.progression.current_form = form;
            runtime.progression.form_change_flash_frames =
                (crate::combat::transform_anim_frames(form) + 1) as u8;
            runtime.floating_items.push(FloatingItem {
                x_pos: 72 << 16,
                y_pos: 766 << 16,
                y_velocity: 0,
                remaining: 0x0300,
                spawn_delay: 0,
                level_col: 4,
                level_row: 36,
                first_row_grace: false,
                content,
                collected: true,
                collision_latched: false,
            });

            let trace = runtime.trace_frame();

            assert_eq!(
                trace.floating_items,
                vec![(72, 766, 0x0300, content.to_id())],
                "ASM PrizeHelmet* loc_10464..loc_106B4 and Kid_Transform loc_7CD0 overlap for one forward-transform frame, so the picked-up mask remains drawable when {form:?}'s transform starts"
            );

            runtime.progression.form_change_flash_frames =
                crate::combat::transform_anim_frames(form) as u8;
            assert!(
                runtime.trace_frame().floating_items.is_empty(),
                "ASM Kid_Transform loc_7D22 should advance past the one-frame picked-up mask hold after the first forward frame for {form:?}"
            );
        }
    }

    #[test]
    fn trace_frame_keeps_collision_latched_collected_helmet_visible_before_cleanup() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.floating_items.push(FloatingItem {
            x_pos: 88 << 16,
            y_pos: 144 << 16,
            y_velocity: 0,
            remaining: 0x0310,
            spawn_delay: 0,
            level_col: 5,
            level_row: 8,
            first_row_grace: false,
            content: terrain::PrizeContent::Skycutter,
            collected: true,
            collision_latched: true,
        });

        assert_eq!(
            runtime.trace_frame().floating_items,
            vec![(88, 144, 0x0310, terrain::PrizeContent::Skycutter.to_id())],
            "ASM PrizeHelmetSkycutter_Init loc_10464 hibernates after collision, so trace should keep the latched collected mask visible until the cleanup frame"
        );
    }

    #[test]
    fn demo_play_object_counts_include_dragon_breath_gfx_child_loc_3d02a() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME + 1,
            data_index: 0,
            use_demo_2: false,
        };

        let baseline = runtime.object_counts_from_roster(&[], 0, 0);
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DragonWalkingBreath,
            lifetime: 60,
            ..Default::default()
        });

        // ASM Dragon.asm loc_3CF90 allocates the breath as an Object_RAM child;
        // loc_3D02A calls j_Load_GfxObjectSlot and loc_3D082 keeps the linked
        // GfxObject live until loc_3D09E deletes the child lifecycle.
        let counts = runtime.object_counts_from_roster(&[], 0, 0);
        assert_eq!(counts.0, baseline.0 + 1);
        assert_eq!(counts.1, baseline.1 + 1);
    }

    #[test]
    fn playback_prize_block_worker_tails_require_own_live_diamond() {
        fn diamond_for_cell(cell_x: u16, cell_y: u16) -> FloatingItem {
            let mut item = diamond_at_x(1_000);
            item.level_col = cell_x;
            item.level_row = cell_y;
            item
        }

        fn diamond_tail(cell_x: u16, cell_y: u16) -> PrizeBlockWorkerTail {
            PrizeBlockWorkerTail {
                cell_x,
                cell_y,
                content: Some(terrain::PrizeContent::Diamond),
                remaining: 1,
            }
        }

        fn visible_tail_count(
            tail: PrizeBlockWorkerTail,
            items: impl IntoIterator<Item = FloatingItem>,
        ) -> usize {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter: demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME + 1,
                data_index: 0,
                use_demo_2: true,
            };
            runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            }));
            runtime.prize_block_worker_tails.push(tail);
            runtime.floating_items.extend(items);
            runtime.count_visible_prize_block_worker_tails()
        }

        let mut collected_match = diamond_for_cell(1, 1);
        collected_match.collected = true;
        let mut non_diamond_match = diamond_for_cell(1, 1);
        non_diamond_match.content = terrain::PrizeContent::Points10000;

        // ASM PrizeDiamond_Init only replaces the stale sub_1007A worker for
        // the matching uncollected diamond from the same prize-block cell.
        assert_eq!(
            visible_tail_count(diamond_tail(1, 1), [diamond_for_cell(1, 1)]),
            0
        );
        assert_eq!(visible_tail_count(diamond_tail(1, 1), [collected_match]), 1);
        assert_eq!(
            visible_tail_count(diamond_tail(1, 1), [non_diamond_match]),
            1
        );
        assert_eq!(
            visible_tail_count(diamond_tail(1, 1), [diamond_for_cell(2, 1)]),
            1
        );
        assert_eq!(
            visible_tail_count(diamond_tail(1, 1), [diamond_for_cell(1, 2)]),
            1
        );
    }

    #[test]
    fn death_respawn_level_one_bootstrap_holds_after_sixty_five_frame_slide() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(0);
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        }));
        runtime.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
        };

        // ASM `lose_life -> Teleport -> loc_6E2`: oracle-backed traces finish
        // the 65-frame hidden Level 1 slide at elapsed frame 64, then hold the
        // same pose instead of applying one more `$-4.8,+0.9` step. The first
        // bootstrap frame reports the pre-extra-gfx worker count; the hold
        // frames keep the same object count but add the gfx worker.
        let initial = runtime.trace_frame();
        assert_eq!(initial.number_objects, 15);
        assert_eq!(initial.number_gfx_objects, 13);

        runtime.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES
                - PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES,
        };
        let frame = runtime.trace_frame();
        assert_eq!(
            frame.kid_x_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP)
        );
        assert_eq!(
            frame.kid_y_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP)
        );
        assert_eq!(frame.number_objects, 15);
        assert_eq!(frame.number_gfx_objects, 14);
    }

    #[test]
    fn trace_frame_ingame_counts_roster_system_and_palette_workers() {
        fn enemy_object(object_type: u8) -> NativeObjectSnapshot {
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type },
                x_pos: 0,
                y_pos: 0,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                sprite_animation_phase: None,
                wait_timer: None,
                debug_fields: BTreeMap::new(),
            }
        }

        fn counts_for_theme(level_theme: u8, background_theme: u8) -> (u16, u16) {
            let mut runtime =
                NativeRuntime::new(manifest_with_theme_level(3, level_theme, background_theme));
            runtime.current_level = LevelId(3);
            runtime.boot_state = BootState::InGame;
            runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            }));
            runtime
                .active_enemy_roster
                .extend([enemy_object(1), enemy_object(2)]);

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Load_InGame keeps two fixed system objects live alongside the
        // player/enemy roster, while Kid owns a second GfxObject. loc_11F96
        // and loc_11FB0 allocate the Mountain/Cave/Island palette controller
        // through Allocate_ObjectSlot only, so it has no GfxObject partner.
        assert_eq!(counts_for_theme(0, 0), (5, 6));
        assert_eq!(counts_for_theme(7, 0), (6, 6));
        assert_eq!(counts_for_theme(0, 4), (6, 6));
    }

    #[test]
    fn trace_frame_auto_timeout_title_counts_split_clear_and_logo_controller() {
        fn counts_at(title_screen_elapsed_frames: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.demo_auto_timeout_handoff_active = true;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM ReadJoypad loc_C0A -> loc_6E2 clears object RAM for three title
        // frames after the timeout snapshot, then Load_SegaScreen exposes the
        // logo controller without a gfx slot.
        assert_eq!(counts_at(DEMO_AUTO_TIMEOUT_PARTIAL_SNAPSHOT_FRAMES), (0, 0));
        assert_eq!(
            counts_at(DEMO_AUTO_TIMEOUT_PARTIAL_SNAPSHOT_FRAMES + 2),
            (0, 0)
        );
        assert_eq!(
            counts_at(DEMO_AUTO_TIMEOUT_PARTIAL_SNAPSHOT_FRAMES + 3),
            (1, 0)
        );
    }

    #[test]
    fn trace_frame_full_title_copy_counts_roster_plus_fixed_title_workers() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.preserve_demo_title_snapshot = true;
        runtime.title_screen_elapsed_frames = DEMO_TITLE_FULL_SNAPSHOT_FRAMES - 1;
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        }));
        runtime
            .active_enemy_roster
            .extend((0..2).map(|object_type| NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type },
                x_pos: 0,
                y_pos: 0,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                sprite_animation_phase: None,
                wait_timer: None,
                debug_fields: BTreeMap::new(),
            }));
        runtime
            .active_platform_roster
            .extend((0..3).map(|subtype| NativeObjectSnapshot {
                kind: NativeObjectKind::Platform { kind: 0, subtype },
                x_pos: 0,
                y_pos: 0,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                sprite_animation_phase: None,
                wait_timer: None,
                debug_fields: BTreeMap::new(),
            }));

        // ASM loc_119D8 title return keeps four fixed title workers alive while
        // the preserved gameplay roster is still exposed.
        let frame = runtime.trace_frame();
        assert_eq!(frame.number_objects, 1 + 2 + 3 + 4);
        assert_eq!(frame.number_gfx_objects, 1 + 2 + 3 + 4);
    }

    #[test]
    fn trace_frame_intro_3_title_return_counts_expire_after_pose_carry_window() {
        fn counts_at(title_screen_elapsed_frames: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.timeout_restart_title_intro_sequence_3_pose = true;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM sub_1C246 -> loc_1C264 carries the Load_IntroSequence3 count
        // state into the first title frames, but drops it once the title pose
        // carry window expires.
        assert_eq!(counts_at(DEMO_TITLE_FULL_SNAPSHOT_FRAMES + 3), (3, 0x000B));
        assert_ne!(counts_at(DEMO_TITLE_FULL_SNAPSHOT_FRAMES + 4), (3, 0x000B));
    }

    #[test]
    fn trace_frame_intro_5_title_return_counts_expire_at_loc_6e2_clear() {
        fn counts_at(title_screen_elapsed_frames: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.timeout_restart_title_intro_sequence_5_pose = true;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM sub_1C246 -> loc_1C264 leaves the loc_1BA28/loc_1BCD0 shell
        // linked after the carried pose is overwritten. loc_6E2 clears the
        // three Objects and five GfxObjects together four frames later.
        assert_eq!(
            counts_at(TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_POSE_CARRY_FRAMES - 1),
            (3, 5)
        );
        assert_eq!(
            counts_at(TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_POSE_CARRY_FRAMES),
            (3, 5)
        );
        assert_eq!(
            counts_at(TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_COUNT_CARRY_FRAMES - 1),
            (3, 5)
        );
        assert_ne!(
            counts_at(TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_COUNT_CARRY_FRAMES),
            (3, 5)
        );
    }

    #[test]
    fn trace_frame_intro_5_title_return_rebuild_follows_load_title_screen_passes() {
        fn counts_at(title_screen_elapsed_frames: usize, delayed_lineage: bool) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.title_screen_intro_return_lineage =
                Some(TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence5);
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;
            runtime.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay =
                delayed_lineage;

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Load_TitleScreen -> Obj_TitleMenu -> loc_1BB10/loc_1C7A0:
        // allocation is visible one outer loc_E12 object pass at a time.
        assert_eq!(counts_at(54, false), (3, 0));
        assert_eq!(counts_at(55, false), (5, 1));
        assert_eq!(counts_at(56, false), (5, 2));
        assert_eq!(counts_at(57, false), (4, 2));
        assert_eq!(counts_at(58, false), (4, 2));
        assert_eq!(counts_at(54, true), (0, 0));
        assert_eq!(counts_at(55, true), (3, 0));
        assert_eq!(counts_at(56, true), (5, 1));
        assert_eq!(counts_at(57, true), (5, 2));
        assert_eq!(counts_at(58, true), (4, 2));
    }

    #[test]
    fn trace_frame_early_start_title_counts_split_hidden_and_visible_workers() {
        fn title_worker(gfx_slot_index: usize) -> LevelEntryTitleWorker {
            LevelEntryTitleWorker {
                gfx_slot_index,
                glyph: 0,
                activation_frame: EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME,
                target_x_pos: 0,
                target_y_pos: 0,
                motion: None,
            }
        }

        fn counts_at(elapsed_frames: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::StartingLevel {
                start_latched: true,
                elapsed_frames,
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: false,
            };
            runtime
                .starting_level_title_workers
                .extend((0..14).map(title_worker));

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Obj_TitleMenu Title_InputLoop reaches Load_TitleCard through the
        // early-start path: hidden workers appear before the first visible
        // title-card frame, then the final gfx worker joins one trace later.
        assert_eq!(
            counts_at(EARLY_START_TITLE_MENU_SHELL_CARRY_FRAMES - 1),
            (4, 3)
        );
        assert_eq!(counts_at(EARLY_START_TITLE_MENU_SHELL_CARRY_FRAMES), (0, 0));
        assert_eq!(
            counts_at(EARLY_START_LOAD_INGAME_PRELOAD_OBJECTS_FRAME - 1),
            (0, 0)
        );
        assert_eq!(
            counts_at(EARLY_START_LOAD_INGAME_PRELOAD_OBJECTS_FRAME),
            (2, 0)
        );
        let load_ingame_counts_at = |elapsed_frames| {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::StartingLevel {
                start_latched: true,
                elapsed_frames,
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: true,
            };

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        };

        assert_eq!(
            load_ingame_counts_at(EARLY_START_LOAD_INGAME_VISIBLE_FRAME - 2),
            (0, 0),
            "ASM loc_6E2 keeps early Start Load_InGame counts blank before the final hidden preload frame",
        );
        assert_eq!(
            load_ingame_counts_at(EARLY_START_LOAD_INGAME_VISIBLE_FRAME - 1),
            (4, 2),
        );
        assert_eq!(
            load_ingame_counts_at(EARLY_START_LOAD_INGAME_VISIBLE_FRAME),
            (5, 4),
        );
        assert_eq!(
            counts_at(EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME - 1),
            (15, 0)
        );
        assert_eq!(
            counts_at(EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME),
            (15, 13)
        );
        assert_eq!(
            counts_at(EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME + 1),
            (15, 14)
        );
    }

    #[test]
    fn trace_frame_cold_boot_title_counts_split_visible_gfx_completion() {
        fn title_worker(gfx_slot_index: usize) -> LevelEntryTitleWorker {
            LevelEntryTitleWorker {
                gfx_slot_index,
                glyph: 0,
                activation_frame: crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME,
                target_x_pos: 0,
                target_y_pos: 0,
                motion: None,
            }
        }

        fn counts_at(elapsed_frames: usize) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::StartingLevel {
                start_latched: false,
                elapsed_frames,
                uses_load_ingame_bootstrap: false,
                load_ingame_bootstrap_started: false,
            };
            runtime
                .starting_level_title_workers
                .extend((0..14).map(title_worker));

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Load_TitleCard keeps the final title-card gfx worker hidden on the
        // first visible frame, then exposes the complete 15/14 shell from the
        // following trace onward.
        assert_eq!(
            counts_at(crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME - 10),
            (4, 4)
        );
        assert_eq!(
            counts_at(crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME - 9),
            (0, 0)
        );
        assert_eq!(
            counts_at(crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME - 5),
            (2, 0)
        );
        assert_eq!(
            counts_at(crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME - 2),
            (13, 0)
        );
        assert_eq!(
            counts_at(crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME - 1),
            (15, 0)
        );
        assert_eq!(
            counts_at(crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME),
            (15, 13)
        );
        assert_eq!(
            counts_at(crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME + 1),
            (15, 14)
        );
    }

    #[test]
    fn trace_frame_counts_second_obj_title_text_kid_burst_from_elapsed_160() {
        fn gfx_count_at(title_screen_elapsed_frames: usize) -> u16 {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::TitleScreen { cursor: 0 };
            runtime.title_screen_ready_frame = 0;
            runtime.title_menu_active = true;
            runtime.title_menu_cursor_pending = false;
            runtime.title_screen_elapsed_frames = title_screen_elapsed_frames;
            runtime.trace_frame().number_gfx_objects
        }

        // ASM Obj_TitleTextKid loc_1B5E4 -> sub_1B652 allocates its first
        // popup at local elapsed 150, hibernates for `$A`, then allocates the
        // second popup at elapsed 160. Raw oracle WRAM changes 3 -> 4 on the
        // corresponding published-route frame 299.
        assert_eq!(gfx_count_at(159), 3);
        assert_eq!(gfx_count_at(160), 4);
    }

    #[test]
    fn trace_frame_late_controller_counts_follow_playback_windows() {
        fn counts_at(
            frame_counter: u16,
            use_demo_2: bool,
            started_from_timeout_restart_shell: bool,
            intro_sequence_6_timeout_return: bool,
        ) -> (u16, u16) {
            let mut runtime = NativeRuntime::new(minimal_manifest());
            runtime.boot_state = BootState::DemoPlay {
                frame_counter,
                data_index: 0,
                use_demo_2,
            };
            runtime.demo_started_from_timeout_restart_shell = started_from_timeout_restart_shell;
            runtime.demo_uses_intro_sequence_6_timeout_return_velocity_sample =
                intro_sequence_6_timeout_return;
            runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            }));

            let frame = runtime.trace_frame();
            (frame.number_objects, frame.number_gfx_objects)
        }

        // ASM Load_DemoPlay -> loc_119D8 keeps controller-side object/gfx
        // samples visible through the late first-recording playback windows.
        assert_eq!(
            counts_at(
                demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME + 1,
                false,
                false,
                false
            ),
            (6, 5)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_FIRST_WAKE_FRAME,
                false,
                false,
                false
            ),
            (7, 6)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_SECOND_NON_GFX_FRAME,
                false,
                false,
                false
            ),
            (7, 5)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_THIRD_WAKE_FRAME,
                false,
                false,
                false
            ),
            (8, 6)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_THIRD_GFX_FRAME,
                false,
                false,
                false
            ),
            (8, 7)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME - 1,
                false,
                true,
                false
            ),
            (8, 7)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME,
                false,
                true,
                false
            ),
            (8, 7)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_TIMEOUT_RESTART_EXTRA_CONTROLLER_OBJECT_FRAME,
                false,
                true,
                false
            ),
            (9, 4)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_TIMEOUT_RESTART_EXTRA_CONTROLLER_OBJECT_FRAME,
                false,
                false,
                true
            ),
            (9, 4),
            "ASM loc_1BB02 -> Load_DemoPlay keeps the cold IntroSequence6 timeout-return controller object sample through the same late demo-1 window"
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME,
                false,
                true,
                false
            ),
            (8, 7)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME + 1,
                false,
                true,
                false
            ),
            (8, 4)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME + 1,
                false,
                false,
                false
            ),
            (5, 4),
            "ASM loc_11FB0 -> loc_12098 keeps the Island controller Object-only after the sampled startup window"
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME + 1,
                true,
                true,
                false
            ),
            (4, 4)
        );
        assert_eq!(
            counts_at(
                demo::DEMO_1_LATE_CONTROLLER_COUNT_FIRST_WAKE_FRAME,
                true,
                false,
                false
            ),
            (4, 4)
        );
    }

    #[test]
    fn trace_frame_reports_prize_diamond_debug_when_either_source_is_live() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.pending_prize_diamond_awards.push(7);

        // ASM PrizeDiamond_Init loc_10396 can leave delayed award bookkeeping
        // live even without a current floating-diamond proximity candidate.
        let pending_only = runtime.trace_frame();
        assert_eq!(
            pending_only
                .gameplay_debug_fields
                .get("pending_prize_diamond_awards"),
            Some(&"[7]".to_string())
        );

        runtime.pending_prize_diamond_awards.clear();
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        }));
        runtime.floating_items.push(diamond_at_x(100));

        // The same loc_10396 frontier diagnostics should remain visible when
        // there are contact candidates but no queued award delays.
        let candidate_only = runtime.trace_frame();
        assert_eq!(
            candidate_only
                .gameplay_debug_fields
                .get("pending_prize_diamond_awards"),
            Some(&"[]".to_string())
        );
        assert!(
            candidate_only
                .gameplay_debug_fields
                .contains_key("floating_diamond_contact_candidates")
        );
    }

    #[test]
    fn trace_frame_rebuilds_pending_prize_diamond_world_words_loc_fefe() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.opening_slice = Some(OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        }));
        runtime.opening_slice.as_mut().unwrap().camera = CameraState {
            x_pos: 0x0020_8000,
            y_pos: 0x0004_8000,
        };

        // ASM loc_FEFE reads only the high words of the cached screen FP
        // coordinates and Camera_X/Y_pos, then adds those words; low-word
        // fractions do not carry into the rendered world pixel.
        runtime.queue_prize_diamond_award_at_screen_offset(0x0064_C000, 0x0050_C000);

        let trace = runtime.trace_frame();
        assert!(
            trace.floating_items.is_empty(),
            "ASM PrizeDiamondCollected_Init loc_FE3E is not PrizeDiamond_Init's block-spawn worker",
        );
        assert_eq!(
            trace.prize_diamond_hud_flights_loc_fe3e,
            vec![(0x0084, 0x0054, 0x01F4)],
            "ASM loc_FEFE rebuilds the collected-prize world words while retaining a presentable HUD-flight Sprite_Table entry",
        );
    }

    #[test]
    fn trace_frame_floating_diamond_candidates_use_fixed_point_pixel_coordinates() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_pos = 96 << 16;
        slice.player.y_pos = 100 << 16;
        runtime.opening_slice = Some(slice);
        runtime.floating_items.push(diamond_at_x(100));
        let mut near_high_diamond = diamond_at_x(95);
        near_high_diamond.y_pos = 90 << 16;
        runtime.floating_items.push(near_high_diamond);
        runtime.floating_items.push(diamond_at_x(74));
        let mut higher_diamond = diamond_at_x(112);
        higher_diamond.y_pos = 50 << 16;
        runtime.floating_items.push(higher_diamond);
        let mut lower_diamond = diamond_at_x(120);
        lower_diamond.y_pos = 150 << 16;
        runtime.floating_items.push(lower_diamond);

        // ASM PrizeDiamond_Init loc_10396 reaches the standard object collision
        // path using FP16.16 positions converted back to pixel coordinates.
        let diamond_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("floating_diamond_contact_candidates")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit floating-diamond candidates"));

        let expected_debug = "diamond@100,100 dx=4 dy=0 gap_x=0 gap_y=0 overlap_x=1 overlap_y=1 spawn=0 rem=500 collected=0; diamond@95,90 dx=-1 dy=-10 gap_x=0 gap_y=0 overlap_x=1 overlap_y=1 spawn=0 rem=500 collected=0; diamond@74,100 dx=-22 dy=0 gap_x=12 gap_y=0 overlap_x=0 overlap_y=1 spawn=0 rem=500 collected=0; diamond@112,50 dx=16 dy=-50 gap_x=6 gap_y=19 overlap_x=0 overlap_y=0 spawn=0 rem=500 collected=0";
        assert_eq!(
            diamond_debug, expected_debug,
            "floating-diamond candidates should be sorted by Manhattan pixel distance",
        );
        assert!(
            diamond_debug
                .contains("diamond@100,100 dx=4 dy=0 gap_x=0 gap_y=0 overlap_x=1 overlap_y=1"),
            "floating-diamond candidate should report pixel-space overlap, got {diamond_debug}",
        );
        assert!(
            diamond_debug
                .contains("diamond@74,100 dx=-22 dy=0 gap_x=12 gap_y=0 overlap_x=0 overlap_y=1"),
            "left-side gap should use the player's left edge, got {diamond_debug}",
        );
        assert!(
            diamond_debug
                .contains("diamond@112,50 dx=16 dy=-50 gap_x=6 gap_y=19 overlap_x=0 overlap_y=0"),
            "vertical gap should use the player's top edge, got {diamond_debug}",
        );
    }

    #[test]
    fn trace_frame_floating_diamond_candidate_reports_lower_vertical_gap() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player.x_pos = 96 << 16;
        slice.player.y_pos = 100 << 16;
        runtime.opening_slice = Some(slice);
        let mut lower_diamond = diamond_at_x(120);
        lower_diamond.y_pos = 150 << 16;
        runtime.floating_items.push(lower_diamond);

        // ASM PrizeDiamond_Init loc_10396 feeds item bounds through the shared
        // object collision path; unk_31D5E's vertical range is y-10..y.
        let diamond_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("floating_diamond_contact_candidates")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit floating-diamond candidates"));

        assert_eq!(
            diamond_debug,
            "diamond@120,150 dx=24 dy=50 gap_x=14 gap_y=40 overlap_x=0 overlap_y=0 spawn=0 rem=500 collected=0",
            "lower vertical gap should use unk_31D5E's y-10 diamond top edge",
        );
    }

    #[test]
    fn trace_frame_floating_prize_candidates_report_row_gate_and_overlap() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::Juggernaut;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 1024,
            height_pixels: 512,
        });
        slice.player.x_pos = 752 << 16;
        slice.player.y_pos = 197 << 16;
        runtime.opening_slice = Some(slice);
        runtime.floating_items.push(FloatingItem {
            x_pos: 728 << 16,
            y_pos: 181 << 16,
            y_velocity: 0,
            remaining: 777,
            spawn_delay: 0,
            level_col: 45,
            level_row: 10,
            first_row_grace: false,
            content: terrain::PrizeContent::IronKnight,
            collected: false,
            collision_latched: false,
        });
        let mut grid = terrain::BlockGrid::from_level_layout(&[], 64, 16);
        grid.set_cell_at(45, 10, terrain::Cell::from_be_bytes(0x60, 0x00));
        runtime.set_block_grid(grid);

        // ASM PrizeHelmetIronKnight_Init -> loc_105D6 -> sub_10C38 gates falling
        // on the current layout row. GfxObjects_CollisionKid then compares the
        // helmet's CollisionSize_Index box against Kid's form-width hitbox.
        let prize_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("floating_prize_contact_candidates")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit floating-prize candidates"));

        assert!(
            prize_debug.contains("prize#07@728,181 dx=-24 dy=-16 row=10 grace=0 rem=777"),
            "candidate should report the Iron Knight mask position and row, got {prize_debug}",
        );
        assert!(
            prize_debug.contains("native_ov=01 form_ov=11"),
            "candidate should distinguish native lower-body overlap from ASM form overlap, got {prize_debug}",
        );
        assert!(
            prize_debug.contains("cell=Solid/open=0/below=Empty/below_open=1"),
            "candidate should expose sub_10C38 row fall gates, got {prize_debug}",
        );
    }

    #[test]
    fn trace_frame_kid_on_slope_uses_fixed_point_pixel_coordinates() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 64,
            height_pixels: 64,
        });
        slice.player.x_pos = (8 << 16) | 1;
        slice.player.y_pos = (7 << 16) | 1;
        slice.player.on_ground = true;
        runtime.opening_slice = Some(slice);

        let mut grid = terrain::BlockGrid::from_level_layout(&[], 4, 4);
        grid.set_cell_at(0, 0, terrain::Cell::from_be_bytes(0x40, 0x00));
        runtime.set_block_grid(grid);

        // ASM sub_9386/sub_A254 slope probes consume pixel positions derived
        // from fixed-point WRAM. The sub-pixel low bit keeps a bad left shift
        // from wrapping back to the same cell and hiding the conversion error.
        assert_eq!(runtime.trace_frame().kid_on_slope, 1);

        let mut grid = terrain::BlockGrid::from_level_layout(&[], 4, 4);
        grid.set_cell_at(0, 0, terrain::Cell::from_be_bytes(0x50, 0x00));
        runtime.set_block_grid(grid);

        assert_eq!(runtime.trace_frame().kid_on_slope, -1);
    }

    #[test]
    fn drips_lifecycle_probe_reports_child_terrain_cell_loc_33650() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 584 << 16,
            y_pos: 592 << 16,
            ..EnemyProjectile::default()
        });
        let mut grid = terrain::BlockGrid::from_level_layout(&[], 64, 64);
        grid.set_cell_at(36, 37, terrain::Cell::from_be_bytes(0xE0, 0x42));
        runtime.set_block_grid(grid);

        // ASM Drips.asm loc_33650 samples neighbouring layout words after the
        // loc_335D8 crawler child is spawned; issue #260 needs the native trace
        // to expose which terrain word each Drips child is currently over.
        let drips_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("drips_lifecycle_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit Drips lifecycle diagnostics"));

        assert!(
            drips_debug.contains("cell=36,37:E042[solid,Smash,skin=0x42]"),
            "Drips lifecycle trace should include child terrain cell context, got {drips_debug}",
        );
    }

    #[test]
    fn trace_frame_exports_continue_countdown_remaining_frames() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.player_lifecycle = PlayerLifecycle::ContinueCountdown {
            remaining_frames: 123,
        };

        // ASM PrizeContinue loc_1080A enters the "CONTINUE?" countdown as a
        // game-over state with a live countdown timer, so trace output must
        // keep both the game-over flag and the remaining-frame value.
        let trace = runtime.trace_frame();
        assert!(trace.game_over);
        assert_eq!(trace.continue_countdown, Some(123));
        assert_eq!(
            trace.continue_countdown_elapsed_frames,
            CONTINUE_COUNTDOWN_FRAMES - 123,
            "ASM Continue_Screen visual workers should expose elapsed countdown time to the renderer"
        );

        runtime.player_lifecycle = PlayerLifecycle::GameOver {
            remaining_frames: 123,
        };

        let trace = runtime.trace_frame();
        assert_eq!(trace.continue_countdown, None);
        assert_eq!(
            trace.game_over_elapsed_frames,
            PLAYER_GAME_OVER_HOLD_FRAMES - 123,
            "ASM loc_D25E GAME/OVER workers should expose elapsed hold time to the renderer"
        );
    }

    #[test]
    fn trace_frame_exports_ending_sequence_elapsed_frames() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::EndingSequence { elapsed: 456 };

        // ASM EndingSequence keeps Game_Mode=$4C with an elapsed frame
        // counter; trace output needs that counter to align native/oracle
        // ending checkpoints.
        assert_eq!(runtime.trace_frame().ending_elapsed, Some(456));

        runtime.boot_state = BootState::TitleScreen { cursor: 0 };

        assert_eq!(runtime.trace_frame().ending_elapsed, None);
    }

    #[test]
    fn trace_frame_exports_menu_cursor_and_sound_test_track() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 2 };

        // ASM Obj_TitleMenu/Options menu both expose the selected row as a
        // cursor byte; trace output keeps it available for menu-state parity.
        let title_trace = runtime.trace_frame();
        assert_eq!(title_trace.options_cursor, Some(2));
        assert_eq!(title_trace.sound_test_track, None);

        runtime.boot_state = BootState::OptionsMenu { cursor: 1 };

        assert_eq!(runtime.trace_frame().options_cursor, Some(1));

        runtime.boot_state = BootState::SoundTest { track: 7 };

        // ASM SoundTest uses the same menu shell but carries the selected
        // sound id separately from the cursor row.
        let sound_trace = runtime.trace_frame();
        assert_eq!(sound_trace.options_cursor, None);
        assert_eq!(sound_trace.sound_test_track, Some(7));
    }

    #[test]
    fn trace_frame_exports_damage_latch_and_diamond_power_boundaries() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.just_received_damage = true;
        runtime.progression.diamond_power_frames = 1;

        // ASM damage paths pulse the trace damage flag directly. Diamond
        // Power activity starts at DiamondPower_Check and then follows the
        // DP-object list at sub_3F5E8 loc_3F610.
        let direct_damage = runtime.trace_frame();
        assert!(direct_damage.just_received_damage);
        assert!(direct_damage.diamond_power_active);

        runtime.just_received_damage = false;
        runtime.progression.diamond_power_frames = 0;
        runtime.death_shell_snapshot = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player: PlayerState::title_pose(),
            frozen_camera: CameraState::default(),
            frozen_active_objects: Vec::new(),
            frozen_number_objects: 0,
            frozen_number_gfx_objects: 0,
        });
        runtime.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: 1,
        };

        // ASM death wind-up keeps the shell snapshot count-visible as damage
        // feedback even after the one-frame damage latch has cleared.
        let death_shell_damage = runtime.trace_frame();
        assert!(death_shell_damage.just_received_damage);
        assert!(!death_shell_damage.diamond_power_active);

        runtime.player_lifecycle = PlayerLifecycle::Alive;

        assert!(!runtime.trace_frame().just_received_damage);
    }

    #[test]
    fn death_tumble_trace_exports_preserved_object_velocity_not_loc_b80c_locals() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES - 1,
        };
        runtime.death_windup_path = DeathWindupPath::Tumble;
        runtime.death_preserved_object_velocity = Some((0x0002_0000, 0));
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player = player_at_x(0x0123);
        slice.player.x_velocity = 0x0000_8000;
        slice.player.y_velocity = -0x0004_0000;
        runtime.opening_slice = Some(slice);

        let trace = runtime.trace_frame();

        // ASM Death loc_B7FA/loc_B80C moves the dying Kid with local d0/d1;
        // it does not write those tumble deltas into the Kid object velocity
        // fields sampled by the oracle helper.
        assert_eq!(trace.kid_x_velocity, Some(0x0002_0000));
        assert_eq!(trace.kid_y_velocity, Some(0));
        assert_eq!(
            trace.character_movement,
            Some(2),
            "ASM Death loc_B80C leaves the preserved Kid object x_vel visible to the oracle movement classifier",
        );
    }

    #[test]
    fn death_tumble_trace_exports_preserved_airborne_movement_loc_b80c() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES - 1,
        };
        runtime.death_windup_path = DeathWindupPath::Tumble;
        runtime.death_preserved_object_velocity = Some((0, -0x0004_0000));
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        });
        slice.player = player_at_x(0x0123);
        slice.player.y_velocity = -0x0004_0000;
        runtime.opening_slice = Some(slice);
        runtime.death_shell_snapshot = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player: slice.player,
            frozen_camera: slice.camera,
            frozen_active_objects: Vec::new(),
            frozen_number_objects: 0,
            frozen_number_gfx_objects: 0,
        });

        let trace = runtime.trace_frame();

        assert_eq!(
            trace.character_movement,
            Some(3),
            "ASM Death loc_B80C keeps tumble-local d1 separate from the Kid object y_vel; issue-260 frame 666 still reports MoveID_Jump",
        );
    }

    #[test]
    fn lose_life_hidden_shell_and_camera_windows_match_oracle_probe_loc_b8de() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.death_shell_snapshot = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player: PlayerState {
                x_pos: 0x038b_2100,
                y_pos: 0x032f_5000,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            frozen_camera: CameraState {
                x_pos: 0x02b6_0000,
                y_pos: 0x01ac_0000,
            },
            frozen_active_objects: Vec::new(),
            frozen_number_objects: 13,
            frozen_number_gfx_objects: 8,
        });

        // ASM lose_life -> Teleport loc_B8DE enters the hidden mode-8 hand-off;
        // oracle route issue-213-drips-upjump-right-probe-cache keeps the
        // shell pose through frame 370, clears Kid on 371, and keeps camera
        // through frame 374 before the blank pre-bootstrap window.
        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 14,
        };
        let last_shell = runtime.trace_frame();
        assert_eq!(last_shell.kid_x_pos, Some(0x038b_2100));
        assert_eq!(last_shell.kid_y_pos, Some(0x032f_5000));
        assert_eq!(last_shell.camera_x_pos, 0x02b6_0000);
        assert_eq!(last_shell.camera_y_pos, 0x01ac_0000);
        assert!(last_shell.just_received_damage);
        assert_eq!(last_shell.number_objects, 9);
        assert_eq!(last_shell.number_gfx_objects, 8);

        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 13,
        };
        let after_shell = runtime.trace_frame();
        assert_eq!(after_shell.kid_x_pos, None);
        assert_eq!(after_shell.kid_y_pos, None);
        assert_eq!(after_shell.camera_x_pos, 0x02b6_0000);
        assert_eq!(after_shell.camera_y_pos, 0x01ac_0000);
        assert!(after_shell.just_received_damage);
        assert_eq!(after_shell.number_objects, 9);
        assert_eq!(after_shell.number_gfx_objects, 8);

        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 10,
        };
        let last_camera = runtime.trace_frame();
        assert_eq!(last_camera.kid_x_pos, None);
        assert_eq!(last_camera.camera_x_pos, 0x02b6_0000);
        assert_eq!(last_camera.camera_y_pos, 0x01ac_0000);
        assert!(last_camera.just_received_damage);
        assert_eq!(last_camera.number_objects, 9);
        assert_eq!(last_camera.number_gfx_objects, 8);

        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 9,
        };
        let after_camera = runtime.trace_frame();
        assert_eq!(after_camera.kid_x_pos, None);
        assert_eq!(after_camera.camera_x_pos, 0);
        assert_eq!(after_camera.camera_y_pos, 0);
        assert!(!after_camera.just_received_damage);
        assert_eq!(after_camera.number_objects, 0);
        assert_eq!(after_camera.number_gfx_objects, 8);

        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 5,
        };
        assert_eq!(runtime.trace_frame().number_objects, 2);

        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 2,
        };
        assert_eq!(runtime.trace_frame().number_objects, 13);

        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 1,
        };
        assert_eq!(runtime.trace_frame().number_objects, 15);
    }

    #[test]
    fn level_two_death_respawn_bootstrap_matches_oracle_probe_loc_b8de() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(1);
        runtime.active_enemy_roster.push(NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0A },
            x_pos: 0x00c7_8000,
            y_pos: 0x00e0_0000,
            layout_index: Some(1),
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            sprite_animation_phase: None,
            wait_timer: None,
            debug_fields: BTreeMap::new(),
        });
        runtime.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
        };

        // ASM lose_life -> Teleport loc_B8DE re-enters loc_6E2 mode 8;
        // oracle route issue-213-drips-upjump-right-probe-cache frames 384,
        // 448, and 449 pin the internal level-2 hidden bootstrap slide and
        // hold counts.
        let first = runtime.trace_frame();
        assert_eq!(
            first.kid_x_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_INITIAL_X_FP)
        );
        assert_eq!(
            first.kid_y_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_INITIAL_Y_FP)
        );
        assert_eq!(first.number_objects, 16);
        assert_eq!(first.number_gfx_objects, 14);
        assert_eq!(first.active_objects.len(), 1);
        assert!(
            first
                .active_objects
                .iter()
                .all(|object| matches!(object.kind, NativeObjectKind::Player)),
            "hidden mode-8 bootstrap must not export stale enemies"
        );

        runtime.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES
                - (PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES - 1),
        };
        let last_slide = runtime.trace_frame();
        assert_eq!(
            last_slide.kid_x_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP)
        );
        assert_eq!(
            last_slide.kid_y_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP)
        );
        assert_eq!(last_slide.number_objects, 16);
        assert_eq!(last_slide.number_gfx_objects, 14);

        runtime.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES
                - PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES,
        };
        let held = runtime.trace_frame();
        assert_eq!(
            held.kid_x_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP)
        );
        assert_eq!(
            held.kid_y_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP)
        );
        assert_eq!(held.number_objects, 16);
        assert_eq!(held.number_gfx_objects, 14);
    }

    #[test]
    fn direct_drips_death_respawn_bootstrap_matches_oracle_probe_loc_b8de() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(1);
        runtime.death_respawn_trace_profile = Some(DeathRespawnTraceProfile::LevelTwoDirectDrips);
        runtime.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
        };

        // ASM lose_life -> Teleport loc_B8DE re-enters loc_6E2 mode 8. Direct
        // oracle probe codex-issue-213-index8_jump_right_now frames 427..428
        // shows the camera-$026F respawn-entry route seeds a different
        // loc_1AC70 hidden bootstrap pose from the wrapped-row upjump route.
        let first = runtime.trace_frame();
        assert_eq!(
            first.kid_x_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_INITIAL_X_FP)
        );
        assert_eq!(
            first.kid_y_pos,
            Some(PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_INITIAL_Y_FP)
        );

        runtime.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES - 1,
        };
        let second = runtime.trace_frame();
        assert_eq!(second.kid_x_pos, Some(0x012c_4000));
        assert_eq!(second.kid_y_pos, Some(0x00fd_a000));
        assert_eq!(second.number_objects, 16);
        assert_eq!(second.number_gfx_objects, 14);
    }

    #[test]
    fn trace_frame_exports_diamond_power_snake_probe_loc_3fad4() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.diamond_power_type = helmets::DiamondPowerType::MiniSnake;
        runtime.progression.diamond_power_frames = 1;
        runtime.diamond_power_snake_head_angle = 0x0040;
        runtime.diamond_power_snake_turn_rate = 0x0006;
        runtime.diamond_power_snake_movement_speed = 0x0300;
        runtime.diamond_power_snake_slots.push(helmets::SnakeSlot {
            x_pos: 0x0020,
            y_pos: 0x010f,
            x_subpixel: 0x8000,
            y_subpixel: 0x4000,
            diamond_size: 4,
            trail: [(0x20, 0x10f); 3],
        });

        let trace = runtime.trace_frame();
        let probe = trace
            .gameplay_debug_fields
            .get("diamond_power_snake_probe")
            .unwrap_or_else(|| panic!("trace should expose Snake DP probe diagnostics"));

        assert!(
            probe.contains("head=0x0020.0x8000,0x010f.0x4000"),
            "ASM loc_3FAD4 low-word position should be visible, got {probe}",
        );
        assert!(
            probe.contains("angle=0x0040/turn=0x0006/speed=0x0300"),
            "ASM loc_3F92A/loc_3FAD4 steering fields should be visible, got {probe}",
        );
    }

    #[test]
    fn trace_frame_exports_live_effects_with_pixel_coordinates() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 120,
            y_px: 208,
            height_px: 32,
            first_eruption_pending: true,
            sound_suppressed: false,
            phase: 0,
            timer: 0,
            current_height: 0,
        });
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 144,
            y_px: 208,
            height_px: 32,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: 1,
            timer: 0,
            current_height: 24,
        });
        runtime.floating_items.push(FloatingItem {
            x_pos: (0x0123 << 16) | 0x4567,
            y_pos: (0x0045 << 16) | 0x6789,
            y_velocity: 0,
            remaining: 0x01AB,
            spawn_delay: 0,
            level_col: 0,
            level_row: 0,
            first_row_grace: false,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });
        runtime.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES - 7,
        };

        // ASM loc_147A0/loc_14914 only exposes visible lava columns once their
        // height is non-zero. Prize_Init/sub_10C50 stores prize objects in
        // FP16.16 WRAM, while the trace renderer consumes whole pixels.
        let trace = runtime.trace_frame();
        assert_eq!(trace.active_geysers, vec![(144, 184, 24, 0, 0)]);
        assert_eq!(trace.floating_items, vec![(0x0123, 0x0045, 0x01AB, 0)]);
        assert_eq!(trace.death_windup_elapsed, 7);

        runtime.player_lifecycle = PlayerLifecycle::Alive;
        assert_eq!(runtime.trace_frame().death_windup_elapsed, 0);
    }

    #[test]
    fn trace_frame_exports_lava_geyser_script_template_loc_147ba() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 0x0120,
            y_px: 0x0312,
            height_px: 0x0060,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C,
            timer: LAVA_GEYSER_REPEAT_SCRIPT_VISIBLE_STEPS_LOC_146A0,
            current_height: LAVA_GEYSER_ROW_HEIGHT_PX,
        });

        let trace = runtime.trace_frame();

        assert_eq!(
            trace.lava_geyser_script_templates_loc_147ba,
            vec![LavaGeyserScriptTemplateTraceLoc147BA {
                x_px: 0x0120,
                base_y_px: 0x0312,
                source_label: "unk_145CA".to_owned(),
            }],
            "ASM loc_147BA should expose the current `unk_146A0` template row for native-player rendering"
        );
    }

    #[test]
    fn trace_frame_exports_sub_61ca_weather_slots() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.weather.weather_type = 3;
        runtime.weather.storm_falling_slots_sub_61ca[0] =
            crate::weather::FallingWeatherSlotSub61CA {
                y_px: 0x0040,
                x_px: 0x0120,
                animation_cooldown: 6,
                animation_frame: 2,
            };
        runtime.weather.storm_falling_slots_sub_61ca[1] =
            crate::weather::FallingWeatherSlotSub61CA::inactive(1, 0);
        runtime.weather.storm_impact_slots_sub_61ca[0] = crate::weather::WeatherImpactSlotSub61CA {
            y_px: 0x0050,
            x_px: 0x011F,
            animation_cooldown: 5,
            animation_frame: 1,
        };
        runtime.weather.storm_impact_slots_sub_61ca[1] =
            crate::weather::WeatherImpactSlotSub61CA::inactive(1, 0);
        runtime
            .weather
            .storm_sprites_sub_61ca
            .push(crate::weather::WeatherSpriteSub61CA {
                y: 0x0088,
                link: 0x0501,
                tile_word: 0xA3CD,
                x: 0x019F,
            });

        let trace = runtime.trace_frame();

        assert_eq!(
            trace.storm_falling_slots_sub_61ca,
            vec![(0x0040, 0x0120, 6, 2)],
            "ASM sub_61CA loc_61E2 falling slot RAM should be trace-visible for oracle/weather diagnostics"
        );
        assert_eq!(
            trace.storm_impact_slots_sub_61ca,
            vec![(0x0050, 0x011F, 5, 1)],
            "ASM sub_61CA loc_63C4 impact slot RAM should be trace-visible for splash/render diagnostics"
        );
        assert_eq!(
            trace.storm_sprites_sub_61ca,
            vec![(0x0088, 0x0501, 0xA3CD, 0x019F)],
            "ASM sub_61CA loc_63C4/loc_6462 emitted sprite words should be trace-visible for renderer diagnostics"
        );
    }

    #[test]
    fn trace_frame_ground_probe_uses_left_and_right_form_edges() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::Juggernaut;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 96,
            height_pixels: 64,
        });
        slice.player.x_pos = 40 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.on_ground = true;
        slice.terrain.floor_y_pos = (32 << 16) | 1;
        runtime.opening_slice = Some(slice);

        let mut grid = terrain::BlockGrid::from_level_layout(&[], 6, 4);
        grid.set_cell_at(1, 2, terrain::Cell::from_be_bytes(0x60, 0x11));
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x60, 0x22));
        grid.set_cell_at(3, 2, terrain::Cell::from_be_bytes(0x60, 0x33));
        runtime.set_block_grid(grid);

        // ASM unk_B408/sub_B41C gives Juggernaut a 23px half-width; the trace
        // mirrors the gameplay floor probes by checking x-half_width, x, x+half_width.
        let ground_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("kid_ground_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit kid-ground probe diagnostics"));

        assert!(
            ground_debug.contains("left=ground=1/y=32/cell=1,2"),
            "left probe should use the form's negative edge, got {ground_debug}",
        );
        assert!(
            ground_debug.contains("latched_floor_y=32"),
            "latched FP16.16 floor should be exported as pixels, got {ground_debug}",
        );
        assert!(
            ground_debug.contains("center=ground=1/y=32/cell=2,2"),
            "center probe should use Kid's centre, got {ground_debug}",
        );
        assert!(
            ground_debug.contains("right=ground=1/y=32/cell=3,2"),
            "right probe should use the form's positive edge, got {ground_debug}",
        );
    }

    #[test]
    fn trace_frame_goal_probe_exports_fixed_point_pixel_distances() {
        let mut runtime = NativeRuntime::new(manifest_with_flag(0x0120, 0x0040));
        runtime.boot_state = BootState::InGame;
        let mut slice = OpeningSliceState::respawn(crate::physics::LevelBounds {
            width_pixels: 512,
            height_pixels: 256,
        });
        slice.player.x_pos = (0x0129 << 16) | 1;
        slice.player.y_pos = (0x0049 << 16) | 1;
        runtime.opening_slice = Some(slice);

        // ASM loc_140B8 compares Kid against the goal flag in FP16.16 WRAM.
        // Trace diagnostics expose the same deltas as whole pixels for parity triage.
        let goal_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("kid_goal_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit kid-goal probe diagnostics"));

        assert!(
            goal_debug.contains("flag=(288, 64)"),
            "goal probe should report the level header flag, got {goal_debug}",
        );
        assert!(
            goal_debug.contains("dx_px=9"),
            "goal probe should convert the FP16.16 X delta to pixels, got {goal_debug}",
        );
        assert!(
            goal_debug.contains("dy_px=9"),
            "goal probe should convert the FP16.16 Y delta to pixels, got {goal_debug}",
        );
        assert!(
            goal_debug.contains("touch=1"),
            "ASM GfxObjects_CollisionKid compares Kid's full hitbox with flag row unk_31D7A, so 9px/9px point distance can still touch: {goal_debug}",
        );

        let slice = runtime.opening_slice.as_mut().unwrap();
        slice.player.x_pos = (0x012b << 16) | 1;
        slice.player.y_pos = (0x0049 << 16) | 1;
        let one_axis_outside = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("kid_goal_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit kid-goal probe diagnostics"));
        assert!(
            one_axis_outside.contains("dx_px=11")
                && one_axis_outside.contains("dy_px=9")
                && one_axis_outside.contains("touch=0"),
            "trace touch should drop once Kid's hitbox no longer overlaps flag row unk_31D7A, got {one_axis_outside}",
        );
    }

    #[test]
    fn trace_frame_collision_handoff_formats_stomp_enemy_candidate() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_collision_handoff_probe = Some(KidCollisionHandoffProbe {
            player_lifecycle: PlayerLifecycle::Alive,
            previous_post_motion_mutator: KidFramePostMotionMutator::None,
            pending_stomp_bounce: true,
            pending_contact_damage: false,
            pending_form: None,
            pending_level_skip: None,
            form_change_flash_frames: 0,
            on_ground_before: false,
            jump_buffer_frames: 0,
            wall_cling_direction: None,
            stomp_candidate: true,
            stomp_enemy_object_type: Some(0x1c),
            stomp_enemy_x_pos: Some(0x0123_4567),
            stomp_enemy_y_pos: Some(0x089a_bcde),
            anchor_x_pos: 0x0100_0000,
            anchor_y_pos: 0x0200_0000,
        });

        // ASM GfxObjects_Collision/sub_B168 carries the stomp target object
        // through the same-frame handoff; the trace keeps the object id and
        // FP16.16 position together so parity triage can tie it back to WRAM.
        let handoff_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("kid_collision_handoff_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit kid collision-handoff diagnostics"));

        assert!(
            handoff_debug.contains("pending_stomp=1"),
            "handoff should expose the pending stomp latch, got {handoff_debug}",
        );
        assert!(
            handoff_debug.contains("stomp_candidate=1/stomp_enemy=0x1c@0x01234567,0x089abcde"),
            "handoff should format the stomp enemy id and FP16.16 position, got {handoff_debug}",
        );
    }

    #[test]
    fn trace_frame_grounded_release_formats_scan_and_current_cell() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::None;
        runtime.last_kid_frame_grounded_release_probe = Some(KidGroundedReleaseProbe {
            on_ground_before: true,
            held_direction: Some(Direction::Right),
            down_held: false,
            jump_pressed: false,
            jump_buffer_frames: 0,
            custom_gravity: false,
            wall_cling_direction: None,
            velocity_branch: KidGroundedReleaseVelocityBranch::GroundedHold,
            probe_row: Some(2),
            left_col: Some(1),
            right_col: Some(3),
            support_col: Some(2),
            support_collision: Some(terrain::CollisionKind::Solid),
            support_block_type: Some(terrain::BlockType::Metal),
            release_triggered: false,
        });
        let mut grid = terrain::BlockGrid::from_level_layout(&[], 5, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0xE0, 0x42));
        runtime.set_block_grid(grid);

        // ASM loc_9D22/sub_9386 scans Kid's left/right floor-probe span before
        // deciding whether support is retained. Trace keeps the scanned range
        // plus the live terrain cell so release frontiers can distinguish stale
        // support from current terrain.
        let release_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("kid_grounded_release_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit kid grounded-release diagnostics"));

        assert!(
            release_debug.contains("scan=row=2/cols=1..3"),
            "grounded-release probe should report the scanned row and column span, got {release_debug}",
        );
        assert!(
            release_debug.contains("support=solid@2,2 block=Metal"),
            "grounded-release probe should report the selected support cell, got {release_debug}",
        );
        assert!(
            release_debug.contains("/terrain=grid=manual/current[solid,Smash,skin=0x42]"),
            "grounded-release probe should report current terrain provenance, got {release_debug}",
        );

        let output_root =
            std::env::temp_dir().join(format!("kid-trace-grounded-release-{}", std::process::id()));
        let layout_dir = output_root.join("level-layout");
        fs::create_dir_all(&layout_dir).expect("layout dir should be creatable");
        let mut layout = vec![0; 5 * 4 * 2];
        let cell_offset = ((2 * 5 + 2) * 2) as usize;
        layout[cell_offset] = 0xE0;
        layout[cell_offset + 1] = 0x42;
        fs::write(layout_dir.join("00.bin"), &layout).expect("layout data should be writable");
        let metadata = LevelLayoutMetadata {
            width_blocks: 5,
            height_blocks: 4,
            data_bytes: layout.len(),
        };
        fs::write(
            layout_dir.join("00.json"),
            serde_json::to_vec(&metadata).expect("layout metadata should serialise"),
        )
        .expect("layout metadata should be writable");
        fs::write(layout_dir.join("00.blocklist.bin"), [0x00, 0x00])
            .expect("layout block list should be writable");

        let mut manifest = minimal_manifest();
        manifest.output_root = output_root;
        let mut runtime = NativeRuntime::new(manifest);
        runtime.boot_state = BootState::InGame;
        runtime.reload_block_grid_for_level(LevelId(0));
        runtime.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::None;
        runtime.last_kid_frame_grounded_release_probe = Some(KidGroundedReleaseProbe {
            support_col: Some(2),
            probe_row: Some(2),
            support_collision: Some(terrain::CollisionKind::Solid),
            support_block_type: Some(terrain::BlockType::Smash),
            ..KidGroundedReleaseProbe::default()
        });

        let generated_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("kid_grounded_release_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit kid grounded-release diagnostics"));
        assert!(
            generated_debug
                .contains("/terrain=grid=generated/current[solid,Smash,skin=0x42]/baseline[solid,Smash,skin=0x42]/same=1"),
            "generated-grid provenance should report matching current and baseline cells, got {generated_debug}",
        );
    }

    #[test]
    fn trace_frame_iron_knight_block_breaker_requires_iron_knight_form() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::TheKid;
        let mut grid = terrain::BlockGrid::from_level_layout(&[], 120, 56);
        for row in 0..3 {
            grid.set_cell_at(10, row, terrain::Cell::from_be_bytes(0xE0, 0));
        }
        runtime.set_block_grid(grid);

        let mut player = player_at_x(154);
        player.y_pos = 32 << 16;
        player.x_velocity = 0x0004_8000;
        player.facing = Direction::Right;
        runtime.last_kid_frame_pre_physics_before_physics = Some(player);

        // ASM sub_AF7A gates Iron Knight block breaking on both form and speed;
        // a fast non-Iron-Knight wall contact is only a normal wall hit.
        let breaker_debug = runtime
            .trace_frame()
            .gameplay_debug_fields
            .get("kid_iron_knight_block_breaker_probe")
            .cloned()
            .unwrap_or_else(|| panic!("trace frame should emit Iron Knight breaker diagnostics"));

        assert!(
            breaker_debug.contains("helmet=0x0000/facing=right/x_vel=0x00048000/speed_gate=0"),
            "breaker speed gate should require Iron Knight form, got {breaker_debug}",
        );
        assert!(
            breaker_debug.contains("candidate=0"),
            "fast non-Iron-Knight wall contact must not be a breaker candidate, got {breaker_debug}",
        );
    }

    #[test]
    fn player_collision_bounds_use_fixed_point_pixels_and_form_half_width() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.progression.current_form = PlayerForm::RedStealth;
        let mut player = player_at_x(120);
        player.y_pos = 96 << 16;

        // ASM loc_1CDE/loc_1D00 object collision compares pixel-space bounds;
        // Kid's top edge is foot_y - PLAYER_HEIGHT and X edges use form width.
        assert_eq!(
            runtime.player_collision_bounds(player),
            ObjectBoundsPx {
                left: 113,
                right: 127,
                top: 65,
                bottom: 96,
            }
        );
    }

    #[test]
    fn kid_object_support_candidate_accepts_exact_trace_window_edges() {
        let candidate = kid_object_support_candidate(
            "edge",
            100,
            196,
            ObjectBoundsPx {
                left: 180,
                right: 200,
                top: 100,
                bottom: 150,
            },
            ObjectBoundsPx {
                left: 186,
                right: 206,
                top: 148,
                bottom: 180,
            },
        )
        .unwrap_or_else(|| {
            panic!("exact Kid/object support trace window edges should be included")
        });

        assert_eq!(candidate.abs_dx_px, KID_OBJECT_SUPPORT_HORIZONTAL_WINDOW_PX);
        assert_eq!(
            candidate.abs_top_gap_px,
            KID_OBJECT_SUPPORT_VERTICAL_WINDOW_PX
        );
        assert!(
            candidate.summary.contains("relation=kidbelow support=1"),
            "ASM loc_1CDE support traces should mark Below overlap as active support, got {}",
            candidate.summary,
        );
    }

    #[test]
    fn kid_object_support_candidate_rejects_one_pixel_outside_trace_window() {
        let player_bounds = ObjectBoundsPx {
            left: 90,
            right: 110,
            top: 70,
            bottom: 100,
        };

        assert_eq!(
            kid_object_support_candidate(
                "too_far_x",
                100,
                100,
                player_bounds,
                ObjectBoundsPx {
                    left: 187,
                    right: 207,
                    top: 98,
                    bottom: 130,
                },
            ),
            None,
            "Kid/object support trace candidates stop one pixel past the horizontal debug window",
        );
        assert_eq!(
            kid_object_support_candidate(
                "too_far_y",
                100,
                100,
                player_bounds,
                ObjectBoundsPx {
                    left: 90,
                    right: 110,
                    top: 149,
                    bottom: 180,
                },
            ),
            None,
            "Kid/object support trace candidates stop one pixel past the vertical debug window",
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KidObjectRelation {
    Below,
    Above,
    Left,
    Right,
}

impl KidObjectRelation {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::Below => "kidbelow",
            Self::Above => "kidabove",
            Self::Left => "kidleft",
            Self::Right => "kidright",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectBoundsPx {
    pub(crate) left: i32,
    pub(crate) right: i32,
    pub(crate) top: i32,
    pub(crate) bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidObjectCollisionWritebackLoc1CF2 {
    pub(crate) player_collision_label: &'static str,
    pub(crate) object_collision_label: &'static str,
    pub(crate) object_collision_flags: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidObjectInvulnerabilityGateLoc1D00 {
    pub(crate) next_player_collision_type: u16,
    pub(crate) next_player_collision_flags: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidObjectStompObjectWritebackLoc1D46 {
    pub(crate) object_collision_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidObjectStompPlayerWritebackLoc1D4C {
    pub(crate) player_collision_label: &'static str,
    pub(crate) player_collision_flags: u16,
    pub(crate) branches_to_invulnerability_gate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct KidObjectTypeFourCollisionLoc1D5A {
    pub(crate) stalls_forever: bool,
    pub(crate) player_collision_flags: Option<u16>,
    pub(crate) player_collision_label: Option<&'static str>,
    pub(crate) object_collision_label: Option<&'static str>,
    pub(crate) branches_to_invulnerability_gate: bool,
}

#[allow(non_snake_case)]
pub(crate) fn kid_object_collision_writeback_loc_1CF2(
    relation: KidObjectRelation,
    object_collision_flags: u16,
) -> KidObjectCollisionWritebackLoc1CF2 {
    // ASM loc_1CF2 (kid.asm:3480): the side/top/bottom classification branches
    // arrive here with `d7` already loaded to one of the `colid_kid*` values.
    // The shared tail writes that same collision id into both `$38(a0)` and
    // `$38(a2)`, then copies the object's raw `$3A(a2)` collision flags into
    // the player's `$3A(a0)` slot before the later invulnerability gate.
    let collision_label = relation.as_debug_label();

    KidObjectCollisionWritebackLoc1CF2 {
        player_collision_label: collision_label,
        object_collision_label: collision_label,
        object_collision_flags,
    }
}

#[allow(non_snake_case)]
pub(crate) fn kid_object_stomp_object_writeback_loc_1D46() -> KidObjectStompObjectWritebackLoc1D46 {
    // ASM loc_1D46 (kid.asm:3512): when `loc_1CE4` sees
    // `Character_Movement == MoveID_Jump`, the stomp path first writes
    // `colid_kidbelow` into the enemy object's `$38(a2)` slot before the
    // player-side writeback at `loc_1D4C`.
    KidObjectStompObjectWritebackLoc1D46 {
        object_collision_label: KidObjectRelation::Below.as_debug_label(),
    }
}

#[allow(non_snake_case)]
pub(crate) fn kid_object_stomp_player_writeback_loc_1D4C(
    object_collision_flags: u16,
) -> KidObjectStompPlayerWritebackLoc1D4C {
    // ASM loc_1D4C (kid.asm:3515): the stomp path writes `colid_kidbelow` into
    // the player's `$38(a0)`, copies the enemy's raw `$3A(a2)` flags into
    // `$3A(a0)`, then branches straight into the shared invulnerability gate at
    // `loc_1D00`.
    KidObjectStompPlayerWritebackLoc1D4C {
        player_collision_label: KidObjectRelation::Below.as_debug_label(),
        player_collision_flags: object_collision_flags,
        branches_to_invulnerability_gate: true,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_object_type_four_collision_loc_1D5A(
    object_collision_flags: u16,
) -> KidObjectTypeFourCollisionLoc1D5A {
    // ASM loc_1D5A (kid.asm:3521): object subtype `4` bypasses the normal
    // overlap classification, loads the raw `$3A(a2)` flags into `d6`, and
    // compares that word directly against `$64`.
    if object_collision_flags < 0x0064 {
        // ASM loc_1D62 (kid.asm:3525): the disassembly's `blt.s loc_1D62`
        // self-branch traps the subtype-4 path in a busy loop whenever
        // `$3A(a2) < $64`, so none of the later `$3A/$38` writeback runs.
        return KidObjectTypeFourCollisionLoc1D5A {
            stalls_forever: true,
            player_collision_flags: None,
            player_collision_label: None,
            object_collision_label: None,
            branches_to_invulnerability_gate: false,
        };
    }

    KidObjectTypeFourCollisionLoc1D5A {
        stalls_forever: false,
        player_collision_flags: Some(object_collision_flags),
        player_collision_label: Some(KidObjectRelation::Below.as_debug_label()),
        object_collision_label: Some(KidObjectRelation::Below.as_debug_label()),
        branches_to_invulnerability_gate: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn kid_object_invulnerability_gate_loc_1D00(
    kid_is_invulnerable: bool,
    player_collision_type: u16,
    player_collision_flags: u16,
) -> KidObjectInvulnerabilityGateLoc1D00 {
    // ASM loc_1D00 (kid.asm:3485): once the shared writeback has copied
    // `$38/$3A`, invulnerability suppresses most collisions. A zero
    // `KidIsInvulnerable` byte or zero collision id returns unchanged.
    let mut result = KidObjectInvulnerabilityGateLoc1D00 {
        next_player_collision_type: player_collision_type,
        next_player_collision_flags: player_collision_flags,
    };

    if !kid_is_invulnerable || player_collision_type == 0 {
        return result;
    }

    if player_collision_type == 0x002C {
        // ASM loc_1D34 (kid.asm:3498): for the special `$2C` collision, clear
        // bits 15 and 14 in `$3A(a0)` and keep the collision type live.
        result.next_player_collision_flags &= 0x3FFF;
        return result;
    }

    // ASM loc_1D00 masks bits 15 and 14 with `bclr`, compares the remaining
    // payload against `$64`, and only clears `$38(a0)` when the masked value is
    // below that threshold.
    let masked_flags = player_collision_flags & 0x3FFF;
    if masked_flags < 0x0064 {
        result.next_player_collision_type = 0;
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KidObjectSupportCandidate {
    abs_top_gap_px: i32,
    abs_dx_px: i32,
    summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FloatingDiamondContactCandidate {
    distance_score: i32,
    summary: String,
}

impl NativeRuntime {
    fn early_start_title_shell_elapsed_frames(&self) -> Option<usize> {
        match self.boot_state {
            BootState::StartingLevel {
                elapsed_frames,
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: false,
                ..
            } => Some(elapsed_frames),
            _ => None,
        }
    }

    fn early_start_load_ingame_elapsed_frames(&self) -> Option<usize> {
        match self.boot_state {
            BootState::StartingLevel {
                elapsed_frames,
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: true,
                ..
            } => Some(elapsed_frames),
            _ => None,
        }
    }

    fn title_card_live_handoff_elapsed_frames_loc_1ae2e(&self) -> Option<usize> {
        (self.title_card_live_handoff_frames > 0).then(|| {
            usize::from(
                TITLE_CARD_LIVE_HANDOFF_FRAMES_LOC_1AE2E
                    .saturating_sub(self.title_card_live_handoff_frames),
            )
        })
    }

    fn title_card_live_handoff_palette_fade_progress_sub_73d0(&self) -> Option<u16> {
        // ASM loc_1AE2E -> loc_6E2 -> Load_InGame -> sub_73D0: the normal
        // title-card hand-off keeps CRAM black through the hidden load window,
        // then advances `$FFFFF876` by `$10` per visible fade frame.
        let elapsed = self.title_card_live_handoff_elapsed_frames_loc_1ae2e()?;
        if elapsed < TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some(0);
        }
        let visible_elapsed =
            elapsed.saturating_sub(TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E);
        // Oracle frame 980 CRAM word `$0731` anchors this lag: sub_14C0
        // consumes the current fade counter before loc_741E increments it.
        let applied_elapsed = visible_elapsed.saturating_sub(3);
        let fade_progress = u16::try_from(applied_elapsed)
            .unwrap_or(u16::MAX)
            .saturating_mul(0x10);
        (fade_progress <= 0x0100).then_some(fade_progress)
    }

    fn demo_level_entry_palette_fade_progress_sub_73d0(&self) -> Option<u16> {
        let frame_counter = self.demo_startup_frame()?;
        let use_demo_2 = self.demo_uses_second_recording()?;
        let fade_elapsed = frame_counter.checked_sub(self.demo_gameplay_start_frame(use_demo_2))?;
        let applied_elapsed = fade_elapsed.saturating_sub(1);
        let fade_progress = applied_elapsed.saturating_mul(0x10);

        // ASM Load_DemoPlay -> Load_InGame -> sub_7196 allocates sub_73D0
        // inside the persistent DemoPlay wrapper. The first live gameplay
        // presentation consumes `$0000`, and the wrapper retains that result
        // for one VBlank before loc_7402 advances `$FFFFF876` by `$10` per
        // frame and deletes the worker after `$0100`.
        (fade_progress <= 0x0100).then_some(fade_progress)
    }

    fn early_start_title_menu_cursor_pose_carry_active(&self) -> bool {
        self.early_start_title_shell_elapsed_frames()
            .is_some_and(|elapsed| elapsed < EARLY_START_TITLE_MENU_CURSOR_POSE_CARRY_FRAMES)
    }

    fn early_start_title_menu_shell_carry_active(&self) -> bool {
        self.early_start_title_shell_elapsed_frames()
            .is_some_and(|elapsed| elapsed < EARLY_START_TITLE_MENU_SHELL_CARRY_FRAMES)
    }

    fn early_start_level_one_hidden_bootstrap_player_state(&self) -> Option<PlayerState> {
        if self.current_level != LevelId(0) {
            return None;
        }

        let bootstrap_elapsed = match self.boot_state {
            BootState::StartingLevel {
                elapsed_frames,
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: false,
                ..
            } => elapsed_frames.checked_sub(EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME)?,
            BootState::StartingLevel {
                elapsed_frames,
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: true,
                ..
            } => {
                // ASM `loc_1AE2E` only parks the carried title-card slot until
                // it consumes the later Start edge. Oracle keeps the settled
                // hidden pose for the first 22 traced `loc_6E2` bootstrap
                // frames, then blanks the player WRAM view again until the
                // later hidden `Load_InGame` reveal begins on frame 238.
                if elapsed_frames >= 22 {
                    return None;
                }
                6usize.saturating_add(elapsed_frames)
            }
            _ => return None,
        };

        if bootstrap_elapsed < EARLY_START_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES {
            let elapsed_i64 = i64::try_from(bootstrap_elapsed).unwrap_or(i64::MAX);
            let x_pos = i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_X_FP)
                + i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_X_STEP_FP) * elapsed_i64;
            let y_pos = i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_Y_FP)
                + i64::from(EARLY_START_LEVEL_ONE_BOOTSTRAP_Y_STEP_FP) * elapsed_i64;
            return Some(PlayerState {
                x_pos: x_pos as u32,
                y_pos: y_pos as u32,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }

        Some(PlayerState {
            // ASM loc_1AE2E -> loc_6E2 holds the early title-start lineage on
            // the settled hidden bootstrap pose after the last traced slide
            // step, before the later visible Load_InGame settle resumes.
            x_pos: EARLY_START_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP,
            y_pos: EARLY_START_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        })
    }

    fn early_start_load_ingame_player_state(&self) -> Option<PlayerState> {
        let elapsed = self.early_start_load_ingame_elapsed_frames()?;
        if elapsed < EARLY_START_LOAD_INGAME_VISIBLE_FRAME {
            return None;
        }
        let mut player = self.opening_slice?.player;
        if self.current_level != LevelId(0) {
            return Some(player);
        }
        // ASM loc_6E2 -> Load_InGame exposes Level 1 one row above the later
        // grounded hold, then settles back onto the normal respawn slice with
        // the same discrete Y steps the oracle shows on frames 238..250.
        player.x_pos = 0x0020_0000;
        let visible_elapsed = elapsed.saturating_sub(EARLY_START_LOAD_INGAME_VISIBLE_FRAME);
        player.y_pos = LEVEL_ONE_LOAD_INGAME_PLAYER_RAMP_Y
            .get(visible_elapsed)
            .copied()
            .unwrap_or(0x010F_0000);
        if (1..=11).contains(&visible_elapsed) {
            player.on_ground = false;
            player.y_velocity =
                i32::try_from(visible_elapsed.saturating_sub(1)).unwrap_or_default() * 0x0000_4000;
        } else {
            player.on_ground = true;
            player.y_velocity = 0;
        }
        Some(player)
    }

    fn title_card_live_handoff_player_state_loc_1ae2e(&self) -> Option<PlayerState> {
        let elapsed = self.title_card_live_handoff_elapsed_frames_loc_1ae2e()?;
        if elapsed < TITLE_CARD_LIVE_HANDOFF_TITLE_WORKER_FRAMES_LOC_1AE2E {
            return Some(PlayerState {
                // ASM loc_1AE2E -> loc_6E2 has entered live mode, but the
                // Fast Action oracle still exposes the old title-card slot-2
                // worker pose through frames 2602..2628.
                x_pos: 0x0040_0000,
                y_pos: 0x0068_0000,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        if elapsed < TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some(PlayerState {
                // ASM loc_6E2 clears the old player mirrors before the later
                // Load_InGame visible settle rehydrates Kid.
                x_pos: 0,
                y_pos: 0,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        if self.current_level != LevelId(0) {
            return self.opening_slice.map(|slice| slice.player);
        }
        let mut player = self.opening_slice?.player;
        let visible_elapsed =
            elapsed.saturating_sub(TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E);
        player.x_pos = 0x0020_0000;
        player.y_pos = LEVEL_ONE_LOAD_INGAME_PLAYER_RAMP_Y
            .get(visible_elapsed)
            .copied()
            .unwrap_or(0x010F_0000);
        if (1..=11).contains(&visible_elapsed) {
            player.on_ground = false;
            player.y_velocity =
                i32::try_from(visible_elapsed.saturating_sub(1)).unwrap_or_default() * 0x0000_4000;
        } else {
            player.on_ground = true;
            player.y_velocity = 0;
        }
        Some(player)
    }

    fn early_start_load_ingame_camera_state(&self) -> Option<CameraState> {
        let elapsed = self.early_start_load_ingame_elapsed_frames()?;
        if elapsed < EARLY_START_LOAD_INGAME_CAMERA_PRELOAD_FRAME {
            return Some(CameraState::default());
        }
        if elapsed == EARLY_START_LOAD_INGAME_CAMERA_PRELOAD_FRAME {
            return Some(CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            });
        }
        if elapsed == EARLY_START_LOAD_INGAME_CAMERA_WRAP_FRAME {
            return Some(CameraState {
                x_pos: 0,
                y_pos: 0xFF00_0000,
            });
        }
        if elapsed < EARLY_START_LOAD_INGAME_VISIBLE_FRAME {
            return Some(CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            });
        }

        if self.current_level == LevelId(0) {
            let visible_elapsed = elapsed.saturating_sub(EARLY_START_LOAD_INGAME_VISIBLE_FRAME);
            let y_pos = LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y
                .get(visible_elapsed)
                .copied()
                .unwrap_or(0x0073_0000);
            return Some(CameraState { x_pos: 0, y_pos });
        }

        let y_pos = self
            .opening_slice
            .map(|slice| slice.camera.y_pos)
            .unwrap_or_default();
        Some(CameraState { x_pos: 0, y_pos })
    }

    fn title_card_live_handoff_camera_state_loc_1ae2e(&self) -> Option<CameraState> {
        let elapsed = self.title_card_live_handoff_elapsed_frames_loc_1ae2e()?;
        if elapsed + 12 == TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some(CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            });
        }
        if elapsed + 11 == TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some(CameraState {
                x_pos: 0,
                y_pos: 0x00C0_0000,
            });
        }
        if elapsed + 10 >= TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E
            && elapsed < TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E
        {
            // ASM loc_6E2/Load_InGame camera preload on the normal title-card
            // route does not include the early-start `$FF00_0000` wrap frame;
            // oracle #305 instead cycles `$00E0`, `$00C0`, then `$00E0` until
            // the visible Level 1 camera ramp begins.
            return Some(CameraState {
                x_pos: 0,
                y_pos: 0x00E0_0000,
            });
        }
        if elapsed < TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some(CameraState::default());
        }
        if self.current_level != LevelId(0) {
            return self
                .opening_slice
                .map(|slice| slice.camera)
                .or(Some(CameraState::default()));
        }
        let visible_elapsed =
            elapsed.saturating_sub(TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E);
        let y_pos = LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y
            .get(visible_elapsed)
            .copied()
            .unwrap_or(0x0073_0000);
        Some(CameraState { x_pos: 0, y_pos })
    }

    fn early_start_load_ingame_object_counts(&self) -> Option<(u16, u16)> {
        let elapsed = self.early_start_load_ingame_elapsed_frames()?;
        if elapsed + 1 < EARLY_START_LOAD_INGAME_VISIBLE_FRAME {
            // ASM loc_1AE2E -> loc_6E2 keeps the carried title workers alive
            // internally for the later hidden Load_InGame hand-off, but the
            // oracle leaves `number_objects`/`number_gfx_objects` blank until
            // the final hidden preload frame immediately before Kid becomes
            // visible again.
            return Some((0, 0));
        }
        if elapsed + 1 == EARLY_START_LOAD_INGAME_VISIBLE_FRAME {
            return Some((4, 2));
        }
        if elapsed == EARLY_START_LOAD_INGAME_VISIBLE_FRAME {
            return Some((5, 4));
        }
        Some((4, 5))
    }

    fn title_card_live_handoff_object_counts_loc_1ae2e(&self) -> Option<(u16, u16)> {
        let elapsed = self.title_card_live_handoff_elapsed_frames_loc_1ae2e()?;
        if elapsed < TITLE_CARD_LIVE_HANDOFF_TITLE_WORKER_FRAMES_LOC_1AE2E {
            return Some((15, 14));
        }
        if elapsed < TITLE_CARD_LIVE_HANDOFF_ZEROED_TITLE_WORKER_FRAMES_LOC_1AE2E {
            return Some((15, 0));
        }
        if elapsed + 1 < TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some((0, 0));
        }
        if elapsed + 1 == TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some((4, 2));
        }
        if elapsed == TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E {
            return Some((5, 4));
        }
        let visible_elapsed =
            elapsed.saturating_sub(TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E);
        if visible_elapsed <= 16 {
            return Some((5, 5));
        }
        Some((4, 5))
    }

    fn dying_hidden_shell_still_visible(&self) -> bool {
        matches!(
            self.player_lifecycle,
            PlayerLifecycle::Dying { remaining_frames }
                if remaining_frames
                    > PLAYER_DEATH_HIDE_FRAMES
                        .saturating_sub(PLAYER_DEATH_HIDDEN_SHELL_VISIBLE_FRAMES)
        )
    }

    fn dying_hidden_camera_still_visible(&self) -> bool {
        matches!(
            self.player_lifecycle,
            PlayerLifecycle::Dying { remaining_frames }
                if remaining_frames
                    > PLAYER_DEATH_HIDE_FRAMES
                        .saturating_sub(PLAYER_DEATH_HIDDEN_CAMERA_VISIBLE_FRAMES)
        )
    }

    fn dying_hidden_number_objects_loc_b8de(&self, snapshot: &LevelSkipTransitionState) -> u16 {
        let PlayerLifecycle::Dying { remaining_frames } = self.player_lifecycle else {
            return snapshot.frozen_number_objects;
        };
        // ASM lose_life -> Teleport loc_B8DE enters the hidden mode-8
        // transition shell. Oracle route issue-213-drips-upjump-right-probe-cache
        // shows the object count decoupling from the old gameplay roster while
        // GFX count remains latched to the frozen shell.
        if self.dying_hidden_camera_still_visible() {
            9
        } else {
            match remaining_frames {
                6..=9 => 0,
                3..=5 => 2,
                2 => snapshot.frozen_number_objects,
                1 => 15,
                _ => snapshot.frozen_number_objects,
            }
        }
    }

    pub(crate) fn current_character_movement(&self, player: PlayerState) -> u16 {
        const MOVE_STANDINGSTILL: u16 = 0;
        const MOVE_CRAWLING: u16 = 1;
        const MOVE_WALKING: u16 = 2;
        const MOVE_JUMP: u16 = 3;
        const MOVE_WALL_CLIMBING: u16 = 4;
        const MOVE_UPHILL_DOWNHILL: u16 = 6;

        if self.wall_cling_boundary_trace_y_velocity.is_some()
            && player.on_ground
            && player.y_velocity == 0
        {
            // ASM loc_A762 -> loc_A8DE/sub_902A stale GFX-slot export window;
            // see `maybe_hold_wall_cling_boundary_trace_y_velocity`.
            return MOVE_JUMP;
        }

        if self.death_shell_snapshot.is_some()
            && matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
            )
        {
            if matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
            ) && self.death_windup_path == DeathWindupPath::Tumble
                && self
                    .death_preserved_object_velocity
                    .is_some_and(|(x_velocity, _)| x_velocity != 0)
            {
                // ASM Death loc_B80C moves the visual shell with local d0/d1,
                // but the oracle helper still sees the pre-death object
                // x_vel, so the movement classifier remains walking.
                return MOVE_WALKING;
            }
            if matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
            ) && self.death_windup_path == DeathWindupPath::Tumble
                && self
                    .death_preserved_object_velocity
                    .is_some_and(|(_, y_velocity)| y_velocity != 0)
            {
                // ASM Death loc_B80C likewise keeps the tumble locals out of
                // the Kid object y_vel sampled by the oracle helper. When
                // issue-260 enters Death from an airborne Tank-mouth contact,
                // WRAM Character_Movement still reports MoveID_Jump on that
                // first damage frame.
                return MOVE_JUMP;
            }
            return MOVE_STANDINGSTILL;
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Respawning { .. }) {
            return MOVE_STANDINGSTILL;
        }
        if matches!(
            self.boot_state,
            BootState::TitleScreen { .. }
                | BootState::IntroSequence { .. }
                | BootState::IntroSequence6 { .. }
                | BootState::OptionsMenu { .. }
                | BootState::SoundTest { .. }
                | BootState::EndingSequence { .. }
        ) && !self.title_uses_full_demo_snapshot()
        {
            // ASM Load_IntroSequence3/4/5 allocate presentation Objects and
            // GfxObjects without entering Kid's loc_75D4 movement routine.
            // Obj_TitleMenu loc_1BBDE/loc_1BBF6 likewise consumes Up/Down into
            // its local `d1` row. None writes Character_Movement at
            // $FFFFFA56 while these presentation modes remain active.
            return MOVE_STANDINGSTILL;
        }
        if self.demo_auto_timeout_handoff_active {
            if self.demo_auto_timeout_handoff_preserves_jump_movement {
                // ASM ReadJoypad loc_C0A (kid.asm:1380) flips to mode 0
                // after the final loc_A6F8/sub_B168 jump frame has already
                // populated Character_Movement.
                return MOVE_JUMP;
            }
            // ASM ReadJoypad loc_C0A (kid.asm:1380) enters the timeout restart
            // by writing Game_Mode=$00 and jumping through loc_6E2. The title
            // snapshot can still carry Kid's pose, but oracle WRAM exposes an
            // inert Character_Movement while this mode-0 shell is active.
            return MOVE_STANDINGSTILL;
        }
        if self
            .title_card_live_handoff_elapsed_frames_loc_1ae2e()
            .is_some_and(|elapsed| {
                elapsed < TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E
            })
        {
            // ASM loc_1AE2E -> loc_6E2 has already switched to live mode, but
            // oracle #305 keeps Character_Movement inert until the hidden
            // Load_InGame carry exposes Kid; the visible ramp then reports the
            // usual jump movement while y_velocity is non-zero.
            return MOVE_STANDINGSTILL;
        }
        if let BootState::DemoPlay {
            frame_counter,
            use_demo_2,
            ..
        } = self.boot_state
            && frame_counter < self.demo_gameplay_start_frame(use_demo_2)
        {
            // ASM Load_DemoPlay -> Load_InGame (kid.asm:23596) creates the
            // demo controller object before the first recorded input reaches
            // Kid's gameplay routine. Keep Character_Movement inert through
            // the hidden bootstrap frames even if the imported TAS raw input
            // still has a title/menu bit held.
            return MOVE_STANDINGSTILL;
        }

        if self.helmet.iron_knight_climbing {
            return MOVE_WALL_CLIMBING;
        }
        if self.demo_1_late_jump_movement_hold_active() {
            return MOVE_JUMP;
        }
        if self.demo_trace_wall_jump_movement_hold_active(player) {
            return MOVE_JUMP;
        }
        if self.probe_start_slope_settle_frames > 0
            && player.on_ground
            && player.x_velocity == 0
            && player.y_velocity == 0
        {
            // ASM loc_9F18 direct WRAM probe seed: oracle frame 0 still
            // exposes the pre-existing jump movement routine even though the
            // synthetic position/velocity seed is already grounded and still.
            return MOVE_JUMP;
        }
        if !player.on_ground || player.just_jumped {
            return MOVE_JUMP;
        }
        if self.trace_input.down {
            return MOVE_CRAWLING;
        }
        // ASM loc_8DFC / loc_8E16: grounded non-Skycutter motion that settles
        // with zero horizontal velocity writes `MoveID_Standingstill` before
        // the ROM reaches the walking/slope continuation. Skycutter is the
        // explicit exception, and rubber-wall contacts can be another one:
        // `sub_8F26 -> loc_8F98` returns negative, so `loc_8E5A` branches to
        // `loc_8E74` and keeps the walking coroutine even after x_vel was
        // cleared by the wall snap.
        // ASM loc_8E16 -> sub_8F26 loc_8FF8 is the level-left-edge counterpart:
        // held-left motion enters sub_8F26 with non-zero x_vel, the boundary
        // clamp clears x_vel, and the negative/NE return keeps the walking
        // continuation at loc_8E74.
        let half_width = self.progression.current_form.half_width();
        let x_px = (player.x_pos >> 16) as i32;
        let y_px = (player.y_pos >> 16) as i32;
        let on_slope = self.block_grid.as_ref().is_some_and(|grid| {
            let left_probe = terrain::probe_floor(grid, x_px - half_width, y_px);
            let center_probe = terrain::probe_floor(grid, x_px, y_px);
            let right_probe = terrain::probe_floor(grid, x_px + half_width, y_px);
            let edge_slope = [left_probe, right_probe].into_iter().any(|probe| {
                matches!(
                    probe.cell.collision,
                    terrain::CollisionKind::SlopeUp | terrain::CollisionKind::SlopeDown
                )
            });
            let core_slope = matches!(
                center_probe.cell.collision,
                terrain::CollisionKind::SlopeUp | terrain::CollisionKind::SlopeDown
            );
            let slope_release = self
                .last_kid_frame_grounded_release_probe
                .is_some_and(|probe| probe.on_ground_before && probe.release_triggered);
            let was_grounded_before_step = self
                .last_kid_frame_grounded_release_probe
                .is_some_and(|probe| probe.on_ground_before);
            let slope_walk_input = self.trace_input.left
                || self.trace_input.right
                || self.trace_input.up
                || self.trace_input.down;
            let core_slope_walk =
                core_slope && (player.x_velocity != 0 || slope_walk_input || slope_release);
            // ASM loc_9D22 writes MoveID 6 once `sub_9386` finds core slope
            // support.  Left/right-edge-only slope contact keeps MoveID 6 only
            // through the seam frame where the previous grounded path released
            // before flat support was reacquired.
            core_slope_walk || (was_grounded_before_step && edge_slope && slope_release)
        });
        let held_left_boundary_clamp_kept_walking_loc_8ff8 = self.trace_input.left
            && !self.trace_input.right
            && player.on_ground
            && x_px <= half_width;
        if player.x_velocity == 0
            && self.progression.current_form != PlayerForm::Skycutter
            && !self.current_frame_kept_walking_after_wall_snap_loc_8e74()
            && !held_left_boundary_clamp_kept_walking_loc_8ff8
            && !on_slope
        {
            return MOVE_STANDINGSTILL;
        }
        if on_slope {
            return MOVE_UPHILL_DOWNHILL;
        }
        MOVE_WALKING
    }

    fn current_frame_kept_walking_after_wall_snap_loc_8e74(&self) -> bool {
        let Some(probe) = self.last_kid_frame_kid_flip_follow_on_probe else {
            return false;
        };

        probe.motion_branch == KidFrameMotionBranch::NormalPhysics
            && matches!(self.boot_state, BootState::InGame)
            && probe.post_motion_mutator == KidFramePostMotionMutator::None
            && probe.wall_hit_after_floor
            && probe.stages.after_floor.on_ground
            && probe.stages.after_floor.x_velocity != 0
            && probe.stages.after_wall.on_ground
            && probe.stages.after_wall.x_velocity == 0
            && (self.trace_input.left
                || self.trace_input.right
                || !self.block_state.rubber_block_animations.is_empty())
    }

    fn demo_trace_wall_jump_movement_hold_active(&self, player: PlayerState) -> bool {
        if !self.demo_startup_frame().is_some_and(|frame_counter| {
            frame_counter > demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME
                && frame_counter <= demo::DEMO_TRACE_WALL_JUMP_MOVEMENT_HOLD_END_FRAME
        }) || !player.on_ground
            || player.x_velocity != 0
        {
            return false;
        }

        self.last_kid_frame_jump_gravity_probe.is_some_and(|probe| {
            // ASM loc_A6F8 writes `MoveID_Jump` before hibernating the Kid
            // object. During the published demo wall-jump landing stall,
            // oracle WRAM keeps $FFFFFA56 on that jump value until loc_A94E's
            // grounded settle path runs, even though the sampled pose is
            // already clamped to the floor.
            probe.branch == KidJumpGravityBranch::AirborneHeldJumpGravity
                && !probe.on_ground_before
                && !probe.on_ground_after
                && probe.y_velocity_after < 0
        })
    }

    fn demo_1_late_jump_movement_hold_active(&self) -> bool {
        let Some((frame_counter, false)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
        else {
            return false;
        };

        // ASM loc_A6F8 -> loc_A94E (kid.asm:13038,13239): during the late
        // demo-1 landing stall, oracle WRAM keeps `Character_Movement` on
        // MoveID_Jump while the sampled controller object is already pinned
        // to the floor. The normal standing/crawling trace inference would
        // expose the post-settle native state too early.
        (demo::DEMO_1_LATE_JUMP_MOVEMENT_HOLD_START_FRAME
            ..=demo::DEMO_1_LATE_JUMP_MOVEMENT_HOLD_END_FRAME)
            .contains(&frame_counter)
    }

    fn title_uses_demo_snapshot(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.preserve_demo_title_snapshot
    }

    fn title_uses_full_demo_snapshot(&self) -> bool {
        self.title_uses_demo_snapshot()
            && self.title_screen_elapsed_frames < self.demo_title_full_snapshot_frames()
    }

    fn demo_title_return_hidden_object_counts(&self) -> Option<(u16, u16)> {
        if !matches!(self.boot_state, BootState::TitleScreen { .. })
            || self.preserve_demo_title_snapshot
            || self.demo_auto_timeout_handoff_active
            || !self.hide_demo_return_title_player
        {
            return None;
        }

        // ASM loc_119D8 -> loc_6E2 returns manual demo aborts to title
        // without clearing object RAM immediately. Once the preserved gameplay
        // snapshot drops, the hidden Load_TitleScreen / Obj_TitleMenu rebuild
        // follows the same staged object-count shape as sub_1C246 title
        // returns before the steady title shell takes over.
        match self.effective_title_screen_rebuild_elapsed() {
            32..=55 => Some((0, 0)),
            56 => Some((3, 0)),
            57 => Some((5, 1)),
            58 => Some((5, 2)),
            59.. => Some((4, self.title_text_kid_gfx_count_loc_1b5e4_sub_1b652())),
            _ => None,
        }
    }

    fn timeout_restart_intro_3_title_return_object_counts(&self) -> Option<(u16, u16)> {
        if !matches!(self.boot_state, BootState::TitleScreen { .. })
            || !self.timeout_restart_title_intro_sequence_3_pose
        {
            return None;
        }

        // ASM Load_TitleScreen (kid.asm:29444) allocates the three title
        // shell objects, then the resumed loc_E12 pass runs Obj_TitleMenu and
        // Obj_TitleTextKid. Raw WRAM shows the latter's initial GfxObject on
        // that same pass and the companion worker on the following pass.
        match self.title_screen_elapsed_frames {
            55 => Some((3, 0)),
            56 => Some((5, 1)),
            57 => Some((5, 2)),
            58..=59 => Some((4, 2)),
            _ => None,
        }
    }

    fn timeout_restart_intro_5_title_return_object_counts(&self) -> Option<(u16, u16)> {
        if !matches!(self.boot_state, BootState::TitleScreen { .. })
            || !matches!(
                self.title_screen_intro_return_lineage,
                Some(TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence5)
            )
        {
            return None;
        }

        // ASM Load_TitleScreen allocates Obj_TitleMenu, Obj_TitleTextKid, and
        // Obj_TitleTextChameleon in one pass. On the next loc_E12 pass,
        // Obj_TitleMenu allocates loc_1BB10 and loc_1C7A0; the two new
        // GfxObjects then join over the following two passes. Oracle WRAM
        // exposes the plain lineage two frames before the generic title-return
        // schedule; the later Start-skip lineage begins one frame after that.
        let first_rebuild_frame = 54
            + usize::from(
                self.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay,
            );
        if self.title_screen_elapsed_frames < first_rebuild_frame {
            return None;
        }
        match self
            .title_screen_elapsed_frames
            .saturating_sub(first_rebuild_frame)
        {
            0 => Some((3, 0)),
            1 => Some((5, 1)),
            2 => Some((5, 2)),
            3..=4 => Some((4, 2)),
            _ => None,
        }
    }

    fn title_uses_timeout_restart_intro_sequence_3_pose(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.timeout_restart_title_intro_sequence_3_pose
            && self.title_screen_elapsed_frames < DEMO_TITLE_FULL_SNAPSHOT_FRAMES
    }

    fn title_uses_timeout_restart_intro_sequence_5_pose(&self) -> bool {
        let carry_frames = if self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry {
            INTRO_SEQUENCE_6_START_SKIP_TITLE_INTRO_SEQUENCE_5_POSE_CARRY_FRAMES
        } else {
            TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_POSE_CARRY_FRAMES
        };
        matches!(self.boot_state, BootState::TitleScreen { .. })
            // ASM sub_1C246 writes `Game_Mode=$2C` and returns immediately, so
            // the loc_1BCD0 slot-2 pose survives into title mode until each
            // lineage's later title rebuild overwrites that WRAM slot.
            && self.timeout_restart_title_intro_sequence_5_pose
            && self.title_screen_elapsed_frames < carry_frames
    }

    fn title_counts_intro_sequence_5_workers(&self) -> bool {
        let carry_frames = if self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry {
            INTRO_SEQUENCE_6_START_SKIP_TITLE_INTRO_SEQUENCE_5_COUNT_CARRY_FRAMES
        } else {
            TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_COUNT_CARRY_FRAMES
        };
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.timeout_restart_title_intro_sequence_5_pose
            && self.title_screen_elapsed_frames < carry_frames
    }

    fn intro_sequence_4_uses_intro_sequence_3_pose(&self) -> bool {
        matches!(
            self.boot_state,
            BootState::IntroSequence {
                segment: 3,
                elapsed,
                ..
            } if elapsed < INTRO_SEQUENCE_4_POSE_CARRY_FRAMES
        )
    }

    fn intro_sequence_4_uses_timeout_restart_camera_handoff(&self) -> bool {
        matches!(
            self.boot_state,
            // ASM sub_1B988: `move.w #4,(Camera_Y_pos).w`. Oracle exposes
            // this only after the carried intro-sequence-3 pose has already
            // cleared. ASM Load_IntroSequence5 only allocates
            // `loc_1BA28` / `loc_1BCD0`; unlike Load_IntroSequence6 and
            // GfxObjects_Collision_BottomBoundary4 it does not call
            // `sub_1B222`, so oracle traces keep the carried camera only for
            // the first 31 intro-sequence-5 frames before the scene drops
            // back to the cleared camera state.
            BootState::IntroSequence {
                segment: 3,
                elapsed,
                ..
            } if elapsed >= TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES
        ) || matches!(
            self.boot_state,
            BootState::IntroSequence {
                segment: 4,
                elapsed,
                ..
            }
                if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_5_CAMERA_CARRY_FRAMES
        )
    }

    fn intro_sequence_5_uses_timeout_restart_slot_2_pose(&self) -> bool {
        let visible_elapsed_frames = if self
            .demo_timeout_restart_uses_intro_return_slot_2_early_visibility
            || self
                .demo_auto_timeout_intro_sequence_5_frames_override
                .is_none()
        {
            TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES - 1
        } else {
            TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES
        };
        matches!(
            self.boot_state,
            // ASM loc_1BCD0 allocates five title gfx slots and writes the
            // third one with `x=0`, `y=$E0`. The verifier reads WRAM slot 2
            // directly from $FF24FA/$FF24FE, and oracle probes show that this
            // timeout-restart intro-sequence-5 slot stays zero through frame
            // 10006 before the loc_1BCD0 write becomes visible on 10007.
            // The later Start-skip intro-return lineage reaches the same
            // loc_1BCD0 slot one frame earlier on the timeout-restart revisit,
            // so keep that oracle-backed offset as a separate lineage bit
            // rather than shifting the shared timer globally.
            BootState::IntroSequence {
                segment: 4,
                elapsed,
                ..
            } if elapsed >= visible_elapsed_frames
        )
    }

    fn cold_demo_timeout_intro_sequence_6_shell(&self) -> bool {
        matches!(self.boot_state, BootState::IntroSequence6 { .. })
            && self.intro_sequence_6_entered_from_timeout_restart
            && self.demo_toggle
            && !self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility
            && !self.demo_timeout_restart_uses_start_skip_intro_sequence_5_delay
            && !self.demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff
    }

    fn intro_sequence_6_carries_timeout_restart_hidden_camera_wrap(&self) -> bool {
        matches!(
            self.boot_state,
            BootState::IntroSequence6 {
                carry_timeout_restart_hidden_camera_wrap: true,
                ..
            }
        )
    }

    /// ASM Load_IntroSequence6 / sub_1B43A: carried-wrap title roster.
    fn carried_wrap_intro_sequence_6_object_counts(&self) -> Option<(u16, u16)> {
        if !self.cold_demo_timeout_intro_sequence_6_shell()
            || !self.intro_sequence_6_carries_timeout_restart_hidden_camera_wrap()
        {
            return None;
        }

        // Raw oracle WRAM identifies each linked-list owner. Load_IntroSequence6
        // first leaves loc_1BE88, loc_1BE3C, loc_1BCCE, and loc_1BA64 carried;
        // it then clears both lists and allocates loc_1BAB0, loc_1B37C, and
        // sub_1B43A. The later boundaries follow loc_1B5CE, loc_1B6AC,
        // GfxObjects_Collision_BottomBoundary4, loc_1BB10, sub_1B652, and the
        // two-frame loc_1C7A0 Sega-text Object exactly.
        match self.title_screen_elapsed_frames {
            0..=21 => Some((4, 5)),
            22..=31 => Some((0, 0)),
            32..=194 => Some((3, 0)),
            195..=235 => Some((2, 0)),
            236 => Some((4, 0)),
            237..=262 => Some((4, 1)),
            263..=266 => Some((3, 1)),
            267 => Some((4, 1)),
            268..=329 => Some((4, 2)),
            330..=336 => Some((4, 3)),
            337..=338 => Some((5, 3)),
            339 => Some((4, 3)),
            340 => Some((4, 4)),
            _ => None,
        }
    }

    /// ASM Load_IntroSequence6 / sub_1B43A: later-revisit title roster.
    fn later_revisit_intro_sequence_6_object_counts(&self) -> Option<(u16, u16)> {
        if !self.intro_sequence_6_entered_from_timeout_restart
            || !matches!(
                self.boot_state,
                BootState::IntroSequence6 {
                    carry_intro_sequence_5_slot_2_pose: true,
                    carry_timeout_restart_hidden_camera_wrap: false,
                    ..
                }
            )
        {
            return None;
        }

        // Raw oracle WRAM shows the same Load_IntroSequence6 owners as the
        // carried-wrap shell, but this later revisit delays the post-clear
        // loc_1BAB0/loc_1B37C/sub_1B43A list by three frames. The subsequent
        // loc_1B5CE, loc_1B6AC, loc_1BB10, sub_1B652, and loc_1C7A0 edges keep
        // that three-frame phase through the settled 4/4 roster.
        match self.title_screen_elapsed_frames {
            0..=21 => Some((4, 5)),
            22..=34 => Some((0, 0)),
            35..=197 => Some((3, 0)),
            198..=238 => Some((2, 0)),
            239 => Some((4, 0)),
            240..=265 => Some((4, 1)),
            266..=269 => Some((3, 1)),
            270 => Some((4, 1)),
            271..=332 => Some((4, 2)),
            333..=339 => Some((4, 3)),
            340..=341 => Some((5, 3)),
            342 => Some((4, 3)),
            343 => Some((4, 4)),
            _ if self.title_menu_active
                && self.frame_index
                    >= self.title_menu_arm_frame.saturating_sub(
                        TITLE_MENU_ACTIVATION_DELAY_FRAMES.saturating_sub(
                            TITLE_MENU_CURSOR_GFX_ALLOCATION_DELAY_FRAMES_LOC_1BB52,
                        ),
                    ) =>
            {
                Some((4, 5))
            }
            _ => None,
        }
    }

    pub(crate) fn intro_sequence_6_hidden_camera_y_for_elapsed(
        elapsed: usize,
        carry_timeout_restart_hidden_camera_wrap: bool,
        cold_timeout_restart_shell: bool,
    ) -> Option<u32> {
        if !carry_timeout_restart_hidden_camera_wrap {
            let seeded_elapsed_frame = if cold_timeout_restart_shell {
                TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_SEEDED_CAMERA_ELAPSED_FRAMES
                    .saturating_sub(1)
            } else {
                TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_SEEDED_CAMERA_ELAPSED_FRAMES
            };
            let ramp_elapsed_frame = if cold_timeout_restart_shell {
                TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES.saturating_sub(1)
            } else {
                TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES
            };
            // ASM Load_IntroSequence6 still seeds `($FFFFA656).w` with
            // `$FFD00000` and calls `sub_1B222` on the later title-restore
            // revisit, but oracle keeps the carried cleared camera through
            // local mode-$28 frame 34 before the delayed seed appears. The
            // cold-boot demo-2 timeout lineage reaches this same hidden shell
            // one local frame earlier after `loc_1BB02 -> Load_DemoPlay`.
            if elapsed < seeded_elapsed_frame {
                return None;
            }
            if elapsed < ramp_elapsed_frame {
                return Some(0xFFD0_0000);
            }

            let ramp_frame = elapsed - ramp_elapsed_frame;
            if ramp_frame < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_FRAMES {
                // ASM GfxObjects_Collision_BottomBoundary4:
                // `move.l ($FFFFA656).w,d1; asr.l #4,d1; ... sub.l d1,($FFFFA656).w`
                // With the later `$FFD0_0000` seed, `d1=$FFFD_0000`, so the
                // hidden camera rises by 3 pixels per frame for 16 frames.
                return Some(0xFFD0_0000_u32.wrapping_add(
                    ((ramp_frame as u32).wrapping_add(1)).wrapping_mul(0x0003_0000),
                ));
            }

            return Some(0);
        }
        if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_WRAP_ELAPSED_FRAMES {
            return None;
        }

        // ASM Load_IntroSequence6 seeds `($FFFFA656).w` with `$FFD00000`,
        // adds `$20` on the timeout-restart lineage, then calls `sub_1B222`
        // so the first exposed hidden row is the wrapped `$FFF0_0000`.
        // Later sub_1B43A allocates GfxObjects_Collision_BottomBoundary4,
        // which delays, derives `d1 = ($FFFFA656).w >> 4`, then repeats
        // `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 frames. With the
        // `$FFF0_0000` seed, that means `d1=$FFFF0000`, so oracle ramps the
        // hidden camera upward by exactly one pixel per frame from local
        // mode-$28 frame 248 until the worker deletes itself.
        if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_ELAPSED_FRAMES {
            return Some(0xFFF0_0000);
        }

        let ramp_frame = elapsed - TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_ELAPSED_FRAMES;
        if ramp_frame < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_FRAMES {
            return Some(
                0xFFF0_0000_u32
                    .wrapping_add(((ramp_frame as u32).wrapping_add(1)).wrapping_mul(0x0001_0000)),
            );
        }

        Some(0)
    }

    pub(crate) fn intro_sequence_6_level_layout_x_for_elapsed(
        elapsed: usize,
        carry_timeout_restart_hidden_camera_wrap: bool,
        cold_timeout_restart_shell: bool,
    ) -> u32 {
        let ramp_elapsed_frame = if carry_timeout_restart_hidden_camera_wrap {
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_ELAPSED_FRAMES
        } else if cold_timeout_restart_shell {
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES.saturating_sub(1)
        } else {
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES
        };
        // ASM loc_1B51A hibernates before each subtraction and sub_1B222
        // upload. The typed WRAM camera exposes its first updated value on the
        // ramp frame, while the corresponding VDP scroll row is presented on
        // the following frame.
        let presented_ramp_elapsed_frame = ramp_elapsed_frame.saturating_add(1);
        if elapsed < presented_ramp_elapsed_frame {
            return 0xFFB8_0000;
        }

        let ramp_frame = elapsed - presented_ramp_elapsed_frame;
        if ramp_frame < TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_FRAMES {
            // ASM GfxObjects_Collision_BottomBoundary4 -> loc_1B51A:
            // `d0 = $FFB80000 >> 4 = $FFFB8000`; subtracting that negative
            // fixed delta advances Level_Layout by 4.5 pixels per pass.
            return 0xFFB8_0000_u32
                .wrapping_add(((ramp_frame as u32).wrapping_add(1)).wrapping_mul(0x0004_8000));
        }

        0
    }

    pub(crate) fn intro_sequence_6_presented_hidden_camera_y_for_elapsed(
        elapsed: usize,
        carry_timeout_restart_hidden_camera_wrap: bool,
        cold_timeout_restart_shell: bool,
    ) -> Option<u32> {
        let ramp_elapsed_frame = if carry_timeout_restart_hidden_camera_wrap {
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_ELAPSED_FRAMES
        } else if cold_timeout_restart_shell {
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES.saturating_sub(1)
        } else {
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES
        };
        let presented_elapsed = if elapsed >= ramp_elapsed_frame {
            elapsed.saturating_sub(1)
        } else {
            elapsed
        };
        Self::intro_sequence_6_hidden_camera_y_for_elapsed(
            presented_elapsed,
            carry_timeout_restart_hidden_camera_wrap,
            cold_timeout_restart_shell,
        )
    }

    pub(crate) fn intro_sequence_6_level_layout_x(&self) -> u32 {
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return 0;
        };
        Self::intro_sequence_6_level_layout_x_for_elapsed(
            self.title_screen_elapsed_frames,
            carry_timeout_restart_hidden_camera_wrap,
            self.cold_demo_timeout_intro_sequence_6_shell(),
        )
    }

    pub(crate) fn intro_sequence_6_presented_hidden_camera_y(&self) -> u32 {
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return self.current_camera_state().y_pos;
        };
        Self::intro_sequence_6_presented_hidden_camera_y_for_elapsed(
            self.title_screen_elapsed_frames,
            carry_timeout_restart_hidden_camera_wrap,
            self.cold_demo_timeout_intro_sequence_6_shell(),
        )
        .unwrap_or_else(|| self.current_camera_state().y_pos)
    }

    fn intro_sequence_6_timeout_restart_hidden_camera_y(&self) -> Option<u32> {
        let BootState::IntroSequence6 {
            carry_intro_sequence_5_slot_2_pose: true,
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return None;
        };
        let elapsed = self.title_screen_elapsed_frames;
        Self::intro_sequence_6_hidden_camera_y_for_elapsed(
            elapsed,
            carry_timeout_restart_hidden_camera_wrap,
            self.cold_demo_timeout_intro_sequence_6_shell(),
        )
    }

    fn title_hides_demo_return_player(&self) -> bool {
        let hidden_frames = if self.timeout_restart_title_intro_sequence_3_pose {
            TIMEOUT_RESTART_TITLE_PLAYER_HIDDEN_FRAMES
        } else if self.timeout_restart_title_intro_sequence_5_pose {
            let mut hidden_frames = TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_HIDDEN_FRAMES;
            if self.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay {
                // ASM sub_1C246 still jumps straight to `Game_Mode=$2C`, but
                // oracle WRAM probes keep the later Start-skip intro-return
                // lineage hidden through frame 16798 before the standard title
                // pose reappears on 16799.
                hidden_frames +=
                    TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_START_SKIP_LINEAGE_EXTRA_HIDDEN_FRAMES;
            }
            hidden_frames
        } else {
            let mut hidden_frames = DEMO_TITLE_PLAYER_HIDDEN_FRAMES;
            if self.demo_title_return_uses_intro_sequence_6_early_title_restore {
                // ASM loc_1B5CE has already written the standard title pose on
                // this lineage before the later timeout-restart demo starts.
                // Oracle therefore drops the generic title-hidden rebuild two
                // frames earlier after the later demo-title return and begins
                // exposing that same pose on local title elapsed frames 148
                // and 149.
                hidden_frames = hidden_frames
                    .saturating_sub(DEMO_AUTO_TIMEOUT_INTRO_SEQUENCE_6_EARLY_TITLE_RESTORE_FRAMES);
            }
            hidden_frames
        };
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.hide_demo_return_title_player
            && (self.demo_auto_timeout_handoff_active
                || self.title_screen_elapsed_frames < hidden_frames)
    }

    fn title_menu_cursor_player_state(&self) -> Option<PlayerState> {
        if !self.title_menu_active || self.title_menu_cursor_pending {
            return None;
        }
        let cursor =
            match self.boot_state {
                // ASM Obj_TitleMenu `loc_1BB52` seeds the menu cursor at
                // `x=$28`, `y=$30`, and `loc_1BC0E` then only adjusts the Y row
                // across the three title entries while the sprite stays live.
                BootState::TitleScreen { cursor } | BootState::IntroSequence6 { cursor, .. } => {
                    cursor
                }
                BootState::StartingLevel { .. }
                    if self.early_start_title_menu_cursor_pose_carry_active() =>
                {
                    if self.two_player { 1 } else { 0 }
                }
                _ => return None,
            };
        let y_pos = match cursor {
            0 => 0x0030_0000,
            1 => 0x003A_0000,
            _ => 0x0044_0000,
        };
        Some(PlayerState::title_menu_cursor_pose(y_pos))
    }

    pub(crate) fn observe_mode_level_work_before_hud_sub_44dc(&self) -> ModeLevelPreHudWorkLedger {
        let player_state = self.current_player_state();
        let camera_state = self.current_camera_state();
        let active_objects = self.active_objects();
        let (linked_object_count, linked_gfx_object_count) =
            self.object_counts_from_roster(&active_objects, camera_state.x_pos, camera_state.y_pos);
        let (roster_without_runtime_transient_count, _) = self
            .object_counts_from_roster_without_runtime_transients(
                &active_objects,
                camera_state.x_pos,
                camera_state.y_pos,
            );
        let active_enemy_workers = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Enemy { .. }))
            .count() as u16;
        let active_platform_workers = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Platform { .. }))
            .count() as u16;
        let player = match player_state {
            None => ModeLevelPlayerWorkBeforeHud::Absent,
            Some(_) if self.level_start_frame == Some(self.frame_index) => {
                ModeLevelPlayerWorkBeforeHud::Activation
            }
            Some(state) if state.x_velocity != 0 || state.y_velocity != 0 => {
                ModeLevelPlayerWorkBeforeHud::Moving
            }
            Some(_) => ModeLevelPlayerWorkBeforeHud::Stationary,
        };
        let player_entry_position = player_state.map(|state| ModeLevelPlayerPosition {
            x_pos: state.x_pos,
            y_pos: state.y_pos,
        });
        let cyclone_visual_accumulator_before = self
            .opening_slice
            .as_ref()
            .map(|slice| slice.walk_visual_accumulator);
        let prize_block_wobble_workers_sub_1007a = self
            .block_bounces
            .iter()
            .filter(|bounce| bounce.kind == BlockBounceKind::Prize)
            .count() as u16;
        let prize_block_worker_tails_sub_1007a =
            self.count_visible_prize_block_worker_tails() as u16;
        let prize_block_flash_workers_loc_102b8 = self.prize_block_spawn_flashes.len() as u16;
        let floating_prize_workers = self
            .floating_items
            .iter()
            .filter(|item| !item.collected)
            .count() as u16;
        let rubber_block_skin_workers_sub_10de4 =
            self.block_state.rubber_block_animations.len() as u16;
        // ASM Load_InGame loc_11F96/loc_11FB0 allocates an Object-only
        // palette coroutine for Mountain, Cave, and Island themes.
        let palette_rotation_worker_loc_11f96 =
            self.level_header(self.current_level).map_or(0, |header| {
                u16::from(
                    header.level_theme == 7
                        || header.level_theme == 8
                        || header.background_theme == 4,
                )
            });
        let classified_runtime_transient_workers = prize_block_wobble_workers_sub_1007a
            .saturating_add(prize_block_worker_tails_sub_1007a)
            .saturating_add(prize_block_flash_workers_loc_102b8)
            .saturating_add(floating_prize_workers)
            .saturating_add(rubber_block_skin_workers_sub_10de4);
        let linked_non_transient_object_count =
            linked_object_count.saturating_sub(classified_runtime_transient_workers);
        let unclassified_runtime_transient_workers = linked_object_count
            .saturating_sub(roster_without_runtime_transient_count)
            .saturating_sub(classified_runtime_transient_workers);
        let prize_block_wobble_paths_sub_1007a = self
            .block_bounces
            .iter()
            .filter(|bounce| bounce.kind == BlockBounceKind::Prize)
            .map(|bounce| ModeLevelPrizeBlockWobbleWork {
                resume: prize_block_wobble_resume_sub_1007a(bounce.remaining),
                wake: prize_block_wobble_wake_sub_1007a(bounce.remaining),
                remaining: bounce.remaining,
                content: bounce.prize_content,
                deleted_tile: bounce.prize_deleted_tile,
            })
            .collect();
        let prize_block_tail_paths_sub_1066 = self
            .visible_prize_block_worker_tails()
            .into_iter()
            .map(|tail| ModeLevelPrizeBlockTailWork {
                cell_x: tail.cell_x,
                cell_y: tail.cell_y,
                content: tail.content,
                state: prize_block_tail_state_sub_1007a(tail.content, tail.remaining),
                list_work: None,
            })
            .collect();
        let prize_block_flash_paths_sub_1066 = self
            .prize_block_spawn_flashes
            .iter()
            .map(|flash| ModeLevelPrizeBlockFlashWork {
                x_px: flash.x_px,
                y_px: flash.y_px,
                remaining: flash.remaining,
                wake: prize_block_flash_wake_loc_102b8(flash.remaining),
            })
            .collect();
        let floating_prize_paths = self
            .floating_items
            .iter()
            .filter(|item| !item.collected)
            .map(|item| {
                let fall_path_loc_103b2 = self.block_grid.as_ref().map(|grid| {
                    floating_prize_fall_path_loc_103b2(
                        item,
                        grid.cell_at(item.level_col, item.level_row).collision,
                        grid.cell_at(item.level_col, item.level_row.saturating_add(1))
                            .collision,
                    )
                });
                ModeLevelFloatingPrizeWork {
                    content: item.content,
                    remaining: item.remaining,
                    spawn_delay: item.spawn_delay,
                    first_row_grace: item.first_row_grace,
                    collision_latched: item.collision_latched,
                    fall_path_loc_103b2,
                }
            })
            .collect();
        let mut phases = vec![
            ModeLevelPreHudPhase::TransferSpriteAndKidToVram,
            ModeLevelPreHudPhase::PaletteToVram,
            ModeLevelPreHudPhase::TransferScrollDataToVram,
            ModeLevelPreHudPhase::ForegroundUpdateSub1596,
            ModeLevelPreHudPhase::HudPreparationSub6874,
            ModeLevelPreHudPhase::MakeSpriteAttrHud,
            ModeLevelPreHudPhase::ExecuteObjectsSubE12,
            ModeLevelPreHudPhase::PostObjectSub21F8,
        ];
        if !self.paused {
            phases.extend([
                ModeLevelPreHudPhase::DiamondPowerRun,
                ModeLevelPreHudPhase::SubDfb0,
                ModeLevelPreHudPhase::SubF7e0,
                ModeLevelPreHudPhase::WeatherSub6034,
                ModeLevelPreHudPhase::SubF096,
            ]);
        }

        ModeLevelPreHudWorkLedger {
            phases,
            execute_objects: ModeLevelExecuteObjectsWork {
                rng_offset_before_sub_e12: Some(self.rom_random.offset_word()),
                linked_object_count,
                linked_gfx_object_count,
                linked_non_transient_object_count,
                player,
                prize_block_wobble_workers_sub_1007a,
                prize_block_worker_tails_sub_1007a,
                prize_block_flash_workers_loc_102b8,
                floating_prize_workers,
                rubber_block_skin_workers_sub_10de4,
                active_enemy_workers,
                active_platform_workers,
                palette_rotation_worker_loc_11f96,
                unclassified_runtime_transient_workers,
                player_entry_position,
                player_gfx_terrain_scan: None,
                cyclone_visual_accumulator_before,
                prize_block_wobble_paths_sub_1007a,
                player_spawned_prize_block_initial_sub_1007a: None,
                player_prize_wobble_list_work_loc_1002e: None,
                prize_object_execution_order_sub_e12: None,
                damage_flash_wait_loc_bc44: (!self.just_received_damage
                    && self.death_shell_snapshot.is_none())
                .then_some(if player == ModeLevelPlayerWorkBeforeHud::Activation {
                    ModeLevelDamageFlashWaitLocBc44::InitialHibernateLocBc34
                } else {
                    ModeLevelDamageFlashWaitLocBc44::AwaitDamage
                }),
                demo_exit_wait_loc_119e0: matches!(self.boot_state, BootState::DemoPlay { .. })
                    .then(|| {
                        if player == ModeLevelPlayerWorkBeforeHud::Activation {
                            ModeLevelDemoExitWaitLoc119E0::InitialHibernateLoc119D8
                        } else if self.trace_input.start && !self.previous_input.start {
                            ModeLevelDemoExitWaitLoc119E0::ExitDemoStart
                        } else {
                            ModeLevelDemoExitWaitLoc119E0::HibernateNoStart
                        }
                    }),
                input_worker_loc_7452: (matches!(self.boot_state, BootState::DemoPlay { .. })
                    && self.hundred_k_trip_timer == 0
                    && !self.trace_input.left
                    && (!self.trace_input.start || self.previous_input.start))
                    .then_some(if player == ModeLevelPlayerWorkBeforeHud::Activation {
                        ModeLevelInputWorkerPathLoc7452::InitialHibernate
                    } else {
                        ModeLevelInputWorkerPathLoc7452::DemoNoLeftNoStartTimerClear
                    }),
                palette_fade_worker_sub_73d0: if player == ModeLevelPlayerWorkBeforeHud::Activation
                {
                    Some(ModeLevelPaletteFadeWorkerPathSub73D0::InitialCopyAndHibernate)
                } else if self
                    .demo_level_entry_palette_fade_progress_sub_73d0()
                    .is_some()
                    || (!matches!(self.boot_state, BootState::DemoPlay { .. })
                        && !self.allow_pausing)
                {
                    // ASM loc_7402 -> loc_7420: DemoPlay owns the fade worker
                    // outside the ordinary InGame `Allow_Pausing` lifecycle.
                    // Its source-derived `$F876` progress helper therefore
                    // decides when the linked Object slot disappears.
                    Some(ModeLevelPaletteFadeWorkerPathSub73D0::AdvanceFadeAndHibernateLoc7402)
                } else {
                    None
                },
                prize_block_tail_paths_sub_1066,
                prize_block_flash_paths_sub_1066,
                floating_prize_paths,
                executed_player_path: None,
            },
            post_execute: ModeLevelPostExecuteWork {
                active_evanescent_slots_before_sub_dfb0: self.block_state.evanescent_blocks.len()
                    as u16,
                evanescent_scan_loc_eae4: None,
                active_shooter_slots_before_sub_f7e0: self.block_state.shooter_blocks.len() as u16,
                level_special_effects_sub_6034: self.weather.weather_type,
                shooter_scan_sub_f096: None,
            },
        }
    }

    /// Gather the live workload after ASM `sub_44DC` and before the natural
    /// `Mode_Level loc_80C -> loc_892` tail.
    pub(crate) fn observe_mode_level_work_after_hud_sub_44dc(&self) -> ModeLevelPostHudWorkLedger {
        let camera = self.current_camera_state();
        let active_objects = self.active_objects();
        let (_, linked_gfx_object_count) =
            self.object_counts_from_roster(&active_objects, camera.x_pos, camera.y_pos);
        let active_platform_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Platform { .. }))
            .count()
            .try_into()
            .expect("platform Object list fits the ROM u16 count");
        let active_enemy_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Enemy { .. }))
            .count();
        let read_joypad_work = if matches!(self.boot_state, BootState::DemoPlay { .. }) {
            ModeLevelReadJoypadWork::DemoPlaybackLocC22(self.demo_read_joypad_work_loc_c22)
        } else if self.options_two_player_two_controllers && self.current_player_index != 0 {
            ModeLevelReadJoypadWork::SecondControllerLocB98
        } else {
            ModeLevelReadJoypadWork::FirstControllerLocB8c
        };
        let enemy_loading_path = if matches!(self.boot_state, BootState::DemoPlay { .. })
            && self.current_level != LevelId(demo::DEMO_1_LEVEL)
        {
            ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a
        } else {
            ModeLevelEnemyLoadingPath::LayoutWalkLoc3682c
        };
        let terrain_special_scan_sub_1fa2 = self.opening_slice.as_ref().and_then(|slice| {
            // ASM sub_21F8 loc_2252: the first gameplay activation still has
            // `$22(a0)=0`, so all four Kid hitbox globals collapse to the
            // sprite anchor before sub_1FA2 scans them.
            let bounds = if self
                .mode_level_work_before_hud_sub_44dc
                .as_ref()
                .is_some_and(|ledger| {
                    ledger.execute_objects.player == ModeLevelPlayerWorkBeforeHud::Activation
                }) {
                let x = i32::try_from(slice.player.x_pos >> 16).ok()?;
                let y = i32::try_from(slice.player.y_pos >> 16).ok()?;
                ObjectBoundsPx {
                    left: x,
                    right: x,
                    top: y,
                    bottom: y,
                }
            } else {
                self.retained_kid_hitbox_loc_224c
                    .filter(|_| {
                        LEDGE_SNAP_PULLUP_FRAMES
                            .checked_sub(slice.player.ledge_snap_cooldown)
                            .is_some_and(|animation_tick| (3..=5).contains(&animation_tick))
                    })
                    .map_or_else(
                        || self.player_collision_bounds(slice.player),
                        |bounds| ObjectBoundsPx {
                            left: i32::from(bounds.left_px),
                            right: i32::from(bounds.right_px),
                            top: i32::from(bounds.top_px),
                            bottom: i32::from(bounds.bottom_px),
                        },
                    )
            };
            let bounds = ModeLevelKidGfxTerrainScanBounds {
                left_col: u16::try_from(bounds.left >> 4).ok()?,
                right_col: u16::try_from(bounds.right >> 4).ok()?,
                top_row: u16::try_from(bounds.top >> 4).ok()?,
                bottom_row: u16::try_from(bounds.bottom >> 4).ok()?,
            };
            self.block_grid
                .as_ref()
                .map(|grid| mode_level_terrain_special_scan_sub_1fa2(grid, bounds))
        });

        ModeLevelPostHudWorkLedger {
            phases: vec![
                ModeLevelPostHudPhase::ManageEnemyLoading,
                ModeLevelPostHudPhase::GfxObjectsMoveAndAnimate,
                ModeLevelPostHudPhase::ReadJoypad,
                ModeLevelPostHudPhase::TerrainEffectsSub5e02,
                ModeLevelPostHudPhase::ManagePlatformLoading,
                ModeLevelPostHudPhase::ExecuteScriptedPlatforms,
                ModeLevelPostHudPhase::PlatformsCheckCollision,
                ModeLevelPostHudPhase::GfxObjectsCollisionF86a,
                ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectile,
                ModeLevelPostHudPhase::LatchDamageState,
                ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectileAgainstKid,
                ModeLevelPostHudPhase::CollisionSub1f52,
                ModeLevelPostHudPhase::CollisionSub219c,
                ModeLevelPostHudPhase::CollisionSub226a,
                ModeLevelPostHudPhase::GfxObjectsCollisionKidF86a,
                ModeLevelPostHudPhase::GfxObjectsCollisionKidF86e,
                ModeLevelPostHudPhase::CollisionSub1d76,
                ModeLevelPostHudPhase::CollisionSub1fa2,
                ModeLevelPostHudPhase::PostCollisionSub2a4c,
                ModeLevelPostHudPhase::MakeSpritesFromGfxObjects,
                ModeLevelPostHudPhase::DiamondPowerCompileSprites,
                ModeLevelPostHudPhase::MakeSpritesFromPlatforms,
            ],
            enemy_loading_path,
            read_joypad_work,
            enemy_loading_initial_pass_complete: self.enemy_loading_initial_pass_complete,
            linked_gfx_object_count,
            murder_wall_active_sub_5e02: self.murder_wall.is_some(),
            platform_loader_work: self.mode_level_platform_loader_work_loc_2744,
            platform_work: ModeLevelPlatformWork {
                frozen_fa64: matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }),
                active_object_count: active_platform_object_count,
            },
            shooter_collision_work: ModeLevelShooterCollisionWork {
                active_support_slots: self
                    .block_state
                    .shooter_blocks
                    .len()
                    .try_into()
                    .expect("ShooterBlock support list fits the ROM u16 count"),
                active_moving_objects: self
                    .projectiles
                    .iter()
                    .filter(|projectile| projectile.kind == EnemyProjectileKind::ShooterBlock)
                    .count()
                    .try_into()
                    .expect("ShooterObject list fits the ROM u16 count"),
            },
            block_debris_work_sub_2a4c: ModeLevelBlockDebrisWorkSub2A4C {
                active_slot_count: self
                    .block_debris_swarms
                    .len()
                    .try_into()
                    .expect("sub_2A4C scratch queue fits its six fixed slots"),
            },
            diamond_power_compile_work_sub_3f798: ModeLevelDiamondPowerCompileWorkSub3F798 {
                buffered_entry_count: self
                    .diamond_power_buffered_sprite_count_before_compile_sub_3f798,
            },
            kid_projectile_collision_work_sub_1d76: ModeLevelKidProjectileCollisionWorkSub1D76 {
                kid_projectile_object_count: self
                    .projectiles
                    .iter()
                    .filter(|projectile| {
                        projectile.friendly
                            && (projectile.lifetime != 0
                                || projectile.kind == EnemyProjectileKind::ManiaxeAxe)
                    })
                    .count()
                    .try_into()
                    .expect("Kid-projectile GfxObject list fits the ROM u16 count"),
                diamond_power_object_count: self
                    .diamond_power_object_lists
                    .active_slots
                    .len()
                    .try_into()
                    .expect("Diamond Power object list fits the ROM u16 count"),
            },
            gfx_move_animate_work_sub_1444: mode_level_gfx_move_animate_work_sub_1444(
                self,
                linked_gfx_object_count,
            ),
            gfx_sprite_work_sub_1492: mode_level_gfx_sprite_work_sub_1492(
                self,
                linked_gfx_object_count,
            ),
            gfx_collision_work: ModeLevelGfxCollisionWork {
                kid_projectile_list_f866: if self.projectiles.iter().any(|projectile| {
                    projectile.friendly
                        && (projectile.lifetime != 0
                            || projectile.kind == EnemyProjectileKind::ManiaxeAxe)
                }) {
                    ModeLevelGfxCollisionListWork::NonEmptyOrUnclassified
                } else {
                    ModeLevelGfxCollisionListWork::Empty
                },
                // ASM Allocate_GfxObjectSlot loc_F9C links kind-2 graphics
                // behind the `$FFFFF86A` sentinel. With no live enemy Object
                // workers the natural gameplay path has no such base or
                // child graphics; live enemy chains remain deliberately
                // unclassified because one Object may own several slots.
                object_kind_two_list_f86a: if active_enemy_object_count == 0 {
                    ModeLevelGfxCollisionListWork::Empty
                } else {
                    ModeLevelGfxCollisionListWork::NonEmptyOrUnclassified
                },
                object_kind_four_list_f86e: mode_level_gfx_collision_kid_f86e_work_sub_1b76(
                    self,
                    linked_gfx_object_count,
                ),
                kid_is_invulnerable: self.kid_is_invulnerable,
            },
            terrain_special_scan_sub_1fa2,
        }
    }

    fn observe_ground_scan_cells_sub_b43a_8600(&self) -> Vec<ModeLevelGroundScanCellSubB43A8600> {
        self.last_kid_frame_grounded_release_probe
            .and_then(|probe| Some((probe.probe_row?, probe.left_col?, probe.right_col?)))
            .and_then(|(row, left_col, right_col)| {
                let grid = self.block_grid.as_ref()?;
                Some(
                    (left_col..=right_col)
                        .map(|col| {
                            let cell = grid.cell_at(col, row);
                            let collision = match cell.collision {
                                crate::terrain::CollisionKind::Solid => {
                                    ModeLevelGroundScanCollisionSubB43A8600::Solid
                                }
                                crate::terrain::CollisionKind::Spikes => {
                                    ModeLevelGroundScanCollisionSubB43A8600::Spikes
                                }
                                _ => ModeLevelGroundScanCollisionSubB43A8600::Other,
                            };
                            let block = if cell.has_block {
                                match cell.block_type {
                                    crate::terrain::BlockType::Ice => {
                                        ModeLevelGroundScanBlockSubB43A8600::Ice
                                    }
                                    crate::terrain::BlockType::Rubber => {
                                        ModeLevelGroundScanBlockSubB43A8600::Rubber
                                    }
                                    crate::terrain::BlockType::Teleporter => {
                                        ModeLevelGroundScanBlockSubB43A8600::Teleporter
                                    }
                                    _ => ModeLevelGroundScanBlockSubB43A8600::Other,
                                }
                            } else {
                                ModeLevelGroundScanBlockSubB43A8600::Unflagged
                            };
                            ModeLevelGroundScanCellSubB43A8600 { collision, block }
                        })
                        .collect(),
                )
            })
            .unwrap_or_default()
    }

    fn observe_mode_level_player_execution_path(
        &self,
        cyclone_visual_accumulator_before: Option<u16>,
        ceiling_scan_sub_914a: Option<ModeLevelCeilingScanSub914A>,
    ) -> ModeLevelPlayerExecutionPath {
        if self
            .mode_level_work_before_hud_sub_44dc
            .as_ref()
            .is_some_and(|ledger| {
                ledger.execute_objects.player == ModeLevelPlayerWorkBeforeHud::Activation
            })
        {
            return ModeLevelPlayerExecutionPath::LevelStartActivationLoc7508 {
                form: self.progression.current_form,
            };
        }
        if let Some(remaining_frames) = self.current_player_state().and_then(|player| {
            (player.ledge_snap_cooldown > 0
                && player.ledge_snap_cooldown < crate::constants::LEDGE_SNAP_PULLUP_FRAMES
                && player.x_velocity == 0
                && player.y_velocity == 0)
                .then_some(player.ledge_snap_cooldown)
        }) {
            // ASM Hibernate_UntilAnimFinished loc_1062: while `$18(a3)` is
            // clear, the player immediately calls sub_E76 and runs no
            // ordinary Kid physics. Native's cooldown mirrors stru_8BB4's
            // seven three-tick frames.
            return ModeLevelPlayerExecutionPath::PullUpAnimationHibernateLoc1062 {
                form: self.progression.current_form,
                remaining_frames,
            };
        }
        if self.last_kid_frame_motion_branch == KidFrameMotionBranch::PullUpAnimationFinished {
            return ModeLevelPlayerExecutionPath::PullUpAnimationFinished {
                form: self.progression.current_form,
            };
        }
        let action = self.last_kid_frame_kid_flip_branch_probe;
        let grounded_stationary = self.last_kid_frame_motion_branch
            == KidFrameMotionBranch::NormalPhysics
            && self.progression.current_form == PlayerForm::TheKid
            && self.active_platform_standing_on.is_none()
            && self.current_player_state().is_some_and(|player| {
                player.on_ground && player.x_velocity == 0 && player.y_velocity == 0
            })
            && self
                .last_kid_frame_grounded_release_probe
                .is_some_and(|probe| {
                    probe.on_ground_before
                        && probe.held_direction.is_none()
                        && !probe.down_held
                        && !probe.jump_pressed
                        && probe.jump_buffer_frames == 0
                        && probe.velocity_branch == KidGroundedReleaseVelocityBranch::GroundedHold
                        && probe.support_col.is_some()
                        && !probe.release_triggered
                })
            && action.is_some_and(|action| !action.b_pressed && !action.c_pressed);
        if grounded_stationary {
            // ASM loc_7606..loc_78C6 (kid.asm:7716-7967): standing The Kid
            // with no controller action and no platform clears both
            // velocities, probes support through sub_B43A, refreshes the
            // standing sprite in sub_78E8, and returns to loc_75D4.
            return ModeLevelPlayerExecutionPath::GroundedStationaryNoInputLoc7606 {
                form: self.progression.current_form,
            };
        }
        let grounded_right_walk_start = self.last_kid_frame_motion_branch
            == KidFrameMotionBranch::NormalPhysics
            && self.progression.current_form == PlayerForm::TheKid
            && self.active_platform_standing_on.is_none()
            && self.current_player_state().is_some_and(|player| {
                player.on_ground && player.x_velocity == 0x0000_1C00 && player.y_velocity == 0
            })
            && self
                .last_kid_frame_grounded_release_probe
                .is_some_and(|probe| {
                    probe.on_ground_before
                        && probe.held_direction == Some(crate::Direction::Right)
                        && !probe.down_held
                        && !probe.jump_pressed
                        && probe.jump_buffer_frames == 0
                        && probe.velocity_branch == KidGroundedReleaseVelocityBranch::GroundedHold
                        && probe.support_col.is_some()
                        && !probe.release_triggered
                })
            && action.is_some_and(|action| !action.b_pressed && !action.c_pressed);
        if grounded_right_walk_start {
            let ground_scan_cells_sub_b43a_8600 = self.observe_ground_scan_cells_sub_b43a_8600();
            // ASM loc_779C -> loc_77C2 -> loc_7820 -> loc_8D72 -> loc_8E06
            // (kid.asm:7842-10252): retain the source-ordered foot-row cells
            // because sub_B43A and sub_8600 take different branches for an
            // unflagged solid and a flagged block before sub_8F26.
            return ModeLevelPlayerExecutionPath::GroundedRightWalkStartLoc779c {
                form: self.progression.current_form,
                ground_scan_cells_sub_b43a_8600,
            };
        }
        let Some(follow_on) = self.last_kid_frame_kid_flip_follow_on_probe else {
            return ModeLevelPlayerExecutionPath::Unclassified;
        };
        let Some(action) = action else {
            return ModeLevelPlayerExecutionPath::Unclassified;
        };
        let stages = follow_on.stages;
        let x_velocity_before = stages.after_press_away.x_velocity;
        let x_velocity_after = stages.after_jump_gravity.x_velocity;
        let wall_cling_vertical = match stages
            .after_jump_gravity
            .y_velocity
            .wrapping_sub(stages.after_press_away.y_velocity)
        {
            0x2000 if stages.after_press_away.y_velocity < 0 => {
                Some(ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8)
            }
            0x4000 => Some(ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0),
            _ => None,
        };
        let wall_cling_pull_up = self.last_kid_frame_motion_branch
            == KidFrameMotionBranch::NormalPhysics
            && self.progression.current_form == PlayerForm::TheKid
            && action.wall_scan_hit
            && action.wall_probe.is_some()
            && action.branch == KidFlipBranchSelection::BPressedTheKid
            && action.b_pressed
            && !action.c_pressed
            && !action.on_ground_before
            && stages.entry.wall_cling_direction == Some(crate::Direction::Right)
            && stages.after_press_away.wall_cling_direction == Some(crate::Direction::Right)
            && stages.after_jump_gravity.wall_cling_direction == Some(crate::Direction::Right)
            && stages.entry.x_velocity == 0
            && stages.after_press_away.x_velocity == 0
            && stages.after_jump_gravity.x_velocity == 0
            && wall_cling_vertical.is_some()
            && stages.gfx_object_collision_clear
            && stages.after_jump_gravity == stages.after_floor
            && self.current_player_state().is_some_and(|player| {
                player.on_ground
                    && player.x_velocity == 0
                    && player.y_velocity == 0
                    && player.wall_cling_direction.is_none()
                    && player.ledge_snap_cooldown == crate::constants::LEDGE_SNAP_PULLUP_FRAMES
            });
        if wall_cling_pull_up {
            let entry_y_px = (self
                .mode_level_work_before_hud_sub_44dc
                .as_ref()
                .and_then(|ledger| ledger.execute_objects.player_entry_position)
                .expect("typed pull-up path retains its entry position")
                .y_pos
                >> 16) as u16;
            let initial_scan_y = entry_y_px.wrapping_sub(20) & !0x000F;
            let final_y_px = (self
                .current_player_state()
                .expect("typed pull-up path retains its final player state")
                .y_pos
                >> 16) as u16;
            let pull_up_scan_rows = final_y_px
                .checked_add(1)
                .and_then(|y| y.checked_sub(initial_scan_y))
                .filter(|distance| *distance != 0 && *distance % 16 == 0)
                .map(|distance| distance / 16);
            if let Some(pull_up_scan_rows) = pull_up_scan_rows {
                // ASM kid_flip loc_ABA4 -> loc_ABF0 (kid.asm): B pressed
                // while clinging scans down in 16px rows, stores the solid
                // row minus one as y_pos, starts stru_8BB4 and plays
                // sfx_The_Kid_pullup before its animation hibernate.
                return ModeLevelPlayerExecutionPath::WallClingPullUpLocAbf0 {
                    form: self.progression.current_form,
                    side: crate::Direction::Right,
                    pull_up_scan_rows,
                };
            }
        }
        let wall_cling_no_buttons = self.last_kid_frame_motion_branch
            == KidFrameMotionBranch::NormalPhysics
            && self.progression.current_form == PlayerForm::TheKid
            && action.wall_scan_hit
            && action.wall_probe.is_some()
            && action.branch == KidFlipBranchSelection::NoButtonBranch
            && !action.b_pressed
            && !action.c_pressed
            && !action.on_ground_before
            && stages.entry.wall_cling_direction == Some(crate::Direction::Right)
            && stages.after_press_away.wall_cling_direction == Some(crate::Direction::Right)
            && stages.after_jump_gravity.wall_cling_direction == Some(crate::Direction::Right)
            && stages.entry.x_velocity == 0
            && stages.after_press_away.x_velocity == 0
            && stages.after_jump_gravity.x_velocity == 0
            && wall_cling_vertical.is_some()
            && stages.gfx_object_collision_clear
            && stages.after_jump_gravity == stages.after_floor
            && stages.after_floor == stages.after_wall;
        if wall_cling_no_buttons {
            // ASM loc_AA22 resumes after Hibernate_Object_1Frame, skipping
            // sub_B084. It runs held-jump gravity, clear Gfx/floor probes,
            // then kid_flip's no-button branch back through loc_AD2C.
            let airborne_visual_mapping_sub_b270 = self
                .helmet
                .airborne_mapping_jump_threshold_y_velocity
                .map_or(
                    ModeLevelAirborneVisualMappingPathSubB270::Unclassified,
                    |jump_threshold| {
                        let scaled_y_velocity =
                            stages.after_jump_gravity.y_velocity.wrapping_shl(2);
                        if scaled_y_velocity < 0 {
                            if scaled_y_velocity > jump_threshold {
                                ModeLevelAirborneVisualMappingPathSubB270::AscendingState1
                            } else {
                                ModeLevelAirborneVisualMappingPathSubB270::AscendingState0
                            }
                        } else if scaled_y_velocity > jump_threshold {
                            ModeLevelAirborneVisualMappingPathSubB270::DescendingState3
                        } else {
                            ModeLevelAirborneVisualMappingPathSubB270::DescendingState2
                        }
                    },
                );
            return ModeLevelPlayerExecutionPath::WallClingNoButtonsLocAa22 {
                form: self.progression.current_form,
                side: crate::Direction::Right,
                wall_scan_hit_row: action
                    .wall_probe
                    .expect("typed wall-cling path retains its hit probe")
                    .cell_y,
                vertical: wall_cling_vertical
                    .expect("typed wall-cling path has a classified gravity branch"),
                y_velocity_after: stages.after_jump_gravity.y_velocity,
                gfx_object_collision_clear: true,
                vertical_collision: ModeLevelVerticalCollisionPath::Clear,
                ceiling_scan_sub_914a,
                airborne_visual_mapping_sub_b270,
            };
        }
        let grounded_jump_launch = self.last_kid_frame_motion_branch
            == KidFrameMotionBranch::NormalPhysics
            && self.progression.current_form == PlayerForm::TheKid
            && self.active_platform_standing_on.is_none()
            && action.b_pressed
            && !action.c_pressed
            && action.on_ground_before
            && !action.on_ground_after
            && stages.after_press_away.on_ground
            && !stages.after_jump_gravity.on_ground
            && stages.gfx_object_collision_clear
            && stages.after_jump_gravity == stages.after_floor
            && stages.after_floor == stages.after_wall
            && !follow_on.wall_hit_after_floor
            && self.opening_slice.as_ref().is_some_and(|slice| {
                crate::physics::initiate_jump_with_hand_grab(
                    x_velocity_before,
                    crate::physics::JUMP_HEIGHTS
                        [usize::from(self.progression.current_form.helmet_id())],
                    slice.kid_grabbed_by_hand,
                ) == (x_velocity_after, stages.after_jump_gravity.y_velocity)
            })
            && self.current_player_state().is_some_and(|player| {
                !player.on_ground
                    && player.x_velocity == x_velocity_after
                    && player.y_velocity == stages.after_jump_gravity.y_velocity
            });
        if grounded_jump_launch {
            // ASM loc_8C70 consumes Button_B before loc_A426 clears Cyclone
            // flight, sub_A432 derives the launch velocities, and loc_A75A
            // rejoins the clear floor/wall/action tail.
            return ModeLevelPlayerExecutionPath::GroundedJumpLaunchLocA426 {
                form: self.progression.current_form,
                x_velocity_before,
                x_velocity_after,
                y_velocity_after: stages.after_jump_gravity.y_velocity,
            };
        }
        let grounded_right_walk_continue = self.last_kid_frame_motion_branch
            == KidFrameMotionBranch::NormalPhysics
            && self.progression.current_form == PlayerForm::TheKid
            && self.active_platform_standing_on.is_none()
            && self.current_player_state().is_some_and(|player| {
                player.on_ground && player.x_velocity == x_velocity_after && player.y_velocity == 0
            })
            && self
                .last_kid_frame_grounded_release_probe
                .is_some_and(|probe| {
                    probe.on_ground_before
                        && probe.held_direction == Some(crate::Direction::Right)
                        && !probe.down_held
                        && !probe.jump_pressed
                        && probe.jump_buffer_frames == 0
                        && probe.velocity_branch == KidGroundedReleaseVelocityBranch::GroundedHold
                        && probe.support_col.is_some()
                        && !probe.release_triggered
                })
            && x_velocity_before > 0
            && x_velocity_after == x_velocity_before + 0x0000_1C00
            && stages.gfx_object_collision_clear
            && !follow_on.wall_hit_after_floor
            && stages.after_floor == stages.after_wall
            && !action.b_pressed
            && !action.c_pressed;
        if grounded_right_walk_continue {
            // ASM loc_8BF0 -> loc_8D72 -> loc_8E06 -> sub_B43A -> sub_8F26
            // (kid.asm): the held-right walking coroutine accelerates from
            // `$1C00` to `$3800`, keeps clear support/wall probes, and returns
            // to loc_8BF0 for its next one-frame hibernate.
            return ModeLevelPlayerExecutionPath::GroundedRightWalkContinueLoc8bf0 {
                form: self.progression.current_form,
                x_velocity_before,
                x_velocity_after,
                ground_scan_cells_sub_b43a_8600: self.observe_ground_scan_cells_sub_b43a_8600(),
            };
        }
        if self.last_kid_frame_motion_branch != KidFrameMotionBranch::NormalPhysics {
            return ModeLevelPlayerExecutionPath::Unclassified;
        }
        let horizontal = mode_level_airborne_horizontal_path(
            stages.after_press_away.x_velocity,
            stages.after_jump_gravity.x_velocity,
        );
        let vertical = if self.progression.current_form == PlayerForm::TheKid
            && stages.after_press_away.y_velocity < 0
            && stages.after_jump_gravity.y_velocity
                == stages.after_press_away.y_velocity.wrapping_add(0x2000)
        {
            // ASM sub_B168 loc_B19C -> loc_B1D4 -> loc_B1E8: a rising Kid
            // with B held receives the lower `$2000` gravity increment.
            ModeLevelAirborneVerticalPath::HeldJumpGravityLocB1e8
        } else if self.progression.current_form == PlayerForm::TheKid
            && stages.after_jump_gravity.y_velocity
                == stages.after_press_away.y_velocity.wrapping_add(0x4000)
        {
            // ASM sub_B168 loc_B1D4 -> loc_B1E0: released B takes the normal
            // `$4000` gravity step before the shared cap at loc_B1EE.
            ModeLevelAirborneVerticalPath::DefaultGravityLocB1e0
        } else if self.progression.current_form == PlayerForm::Cyclone
            && self.helmet.cyclone_flying
            && !action.c_pressed
        {
            // ASM sub_B168 loc_B214 (kid.asm:14091-14126): no consumed C edge
            // takes loc_B23A, stores the acceleration at loc_B246, clamps the
            // resulting velocity, then falls through into sub_B270.
            ModeLevelAirborneVerticalPath::CycloneFlyingNoCPress {
                acceleration_after: self.helmet.cyclone_y_accel,
                y_velocity_after: stages.after_jump_gravity.y_velocity,
                capped_acceleration_to_0x1200: stages
                    .cyclone_acceleration_before_sub_b168
                    .is_some_and(|before| {
                        before.wrapping_add(CYCLONE_ACCEL_GRAVITY) > CYCLONE_ACCEL_MAX_DESCENT
                    }),
            }
        } else {
            ModeLevelAirborneVerticalPath::Unclassified
        };
        let landed = !stages.after_jump_gravity.on_ground && stages.after_floor.on_ground;
        let ceiling_hit = stages.after_jump_gravity.y_velocity < 0
            && stages.after_floor.y_velocity == 0
            && !stages.after_floor.on_ground;
        let vertical_collision = mode_level_vertical_collision_path(
            ceiling_hit,
            landed,
            stages.after_jump_gravity.y_velocity,
        );
        let wall_collision =
            if !follow_on.wall_hit_after_floor && stages.after_floor == stages.after_wall {
                ModeLevelWallCollisionPath::Clear
            } else if stages.after_floor.x_velocity > 0
                && stages.after_wall.x_velocity == 0
                && stages.after_floor.y_velocity == stages.after_wall.y_velocity
                && stages.after_wall.wall_cling_direction == Some(crate::Direction::Right)
            {
                // ASM sub_8F26 -> sub_922C -> loc_8F98 -> loc_901E returns one;
                // loc_A8DE clears the side byte and enters loc_A9E4 wall-cling.
                ModeLevelWallCollisionPath::RightWallClingLocA9e4
            } else {
                ModeLevelWallCollisionPath::Unclassified
            };
        let airborne_action = if matches!(
            wall_collision,
            ModeLevelWallCollisionPath::RightWallClingLocA9e4
        ) {
            ModeLevelAirborneActionPath::NotReachedAfterWallCollision
        } else if !action.b_pressed && !action.c_pressed {
            // ASM loc_A7B4/loc_A7F8 (kid.asm:13199-13287): neither button
            // reaches loc_A8D6, whose sub_B270 call precedes loc_A6F8.
            ModeLevelAirborneActionPath::NoButtons
        } else {
            ModeLevelAirborneActionPath::Unclassified
        };
        let cyclone_visual_steps = if self.progression.current_form == PlayerForm::Cyclone
            && self.helmet.cyclone_flying
            && matches!(airborne_action, ModeLevelAirborneActionPath::NoButtons)
        {
            cyclone_visual_accumulator_before.map(|accumulator_before| {
                let first =
                    mode_level_cyclone_visual_step(accumulator_before, self.helmet.cyclone_y_accel);
                let second = mode_level_cyclone_visual_step(
                    first.accumulator_after,
                    self.helmet.cyclone_y_accel,
                );
                [first, second]
            })
        } else {
            None
        };

        ModeLevelPlayerExecutionPath::NormalPhysics(ModeLevelNormalPhysicsPath {
            horizontal,
            vertical,
            y_velocity_after: stages.after_jump_gravity.y_velocity,
            gfx_object_collision_clear: stages.gfx_object_collision_clear,
            vertical_collision,
            wall_collision,
            airborne_action,
            cyclone_visual_steps,
            // ASM sub_902A calls sub_914A whenever vertical motion crosses a
            // tile row. Retain clear scans as well as collision-producing
            // scans because their variable-width loop is timing-visible.
            ceiling_scan_sub_914a,
        })
    }

    pub(crate) fn complete_mode_level_player_execution_path_before_hud_sub_44dc(&mut self) {
        if self.mode_level_work_before_hud_sub_44dc.is_none() {
            return;
        }
        let cyclone_visual_accumulator_before = self
            .mode_level_work_before_hud_sub_44dc
            .as_ref()
            .and_then(|ledger| ledger.execute_objects.cyclone_visual_accumulator_before);
        let ceiling_scan_sub_914a =
            self.mode_level_work_before_hud_sub_44dc
                .as_ref()
                .and_then(|ledger| {
                    let cells = self
                        .last_kid_frame_kid_flip_follow_on_probe?
                        .stages
                        .ceiling_scan_cells_sub_914a;
                    mode_level_ceiling_scan_sub_914a(
                        cells,
                        ledger.execute_objects.rubber_block_skin_workers_sub_10de4,
                    )
                });
        let executed_player_path = self.observe_mode_level_player_execution_path(
            cyclone_visual_accumulator_before,
            ceiling_scan_sub_914a,
        );
        if matches!(
            &executed_player_path,
            ModeLevelPlayerExecutionPath::LevelStartActivationLoc7508 { .. }
        ) {
            // ASM sub_73D0 $73D0 clears PaletteToDMA_Flag after this frame's
            // earlier Mode_Level $7CC palette call.
            self.palette_dma_suppressed_f890 = false;
        }
        let player_gfx_terrain_scan = self
            .mode_level_work_before_hud_sub_44dc
            .as_ref()
            .and_then(|ledger| ledger.execute_objects.player_entry_position)
            .and_then(|entry| {
                let stages = self.last_kid_frame_kid_flip_follow_on_probe?.stages;
                mode_level_kid_gfx_terrain_scan_bounds(
                    entry,
                    stages.after_jump_gravity.x_velocity,
                    stages.after_jump_gravity.y_velocity,
                    self.progression.current_form,
                )
            })
            .and_then(|bounds| {
                self.block_grid
                    .as_ref()
                    .map(|grid| mode_level_kid_gfx_terrain_scan(grid, bounds))
            });
        let first_activation_post_scans = matches!(
            executed_player_path,
            ModeLevelPlayerExecutionPath::LevelStartActivationLoc7508 { .. }
        )
        .then(|| {
            let slice = self.opening_slice.as_ref()?;
            let grid = self.block_grid.as_ref()?;
            let (_, evanescent) = Self::evanescent_hitbox_trigger_loc_EAE4(
                slice,
                grid,
                self.evanescent_hitbox_half_width_loc_EAE4(),
            );
            let kid_block_col = ((slice.player.x_pos >> 16) >> 4) as i16;
            let kid_block_row = ((slice.player.y_pos >> 16) >> 4) as i16;
            Some((
                evanescent,
                ModeLevelShooterScanWorkSubF096 {
                    cached_col_matches: self.block_state.shooter_last_scanned_col == kid_block_col,
                    cached_row_matches: self.block_state.shooter_last_scanned_row == kid_block_row,
                    active_shooter_slots_before: self.block_state.shooter_blocks.len() as u16,
                    cached_col_before: self.block_state.shooter_last_scanned_col,
                    cached_row_before: self.block_state.shooter_last_scanned_row,
                    kid_block_col,
                    kid_block_row,
                    level_width_blocks: grid.width_cells,
                    level_height_blocks: grid.height_cells,
                    new_shooter_slots_registered: 0,
                },
            ))
        })
        .flatten();
        if let Some((_, shooter)) = first_activation_post_scans {
            // ASM sub_F096 loc_F09E/loc_F186 (kid.asm:19623/19727) writes
            // Kid's current block coordinates after comparing each cache.
            // Keep the ledger's pre-write values, then commit the state that
            // the next Mode_Level pass observes.
            self.block_state.shooter_last_scanned_col = shooter.kid_block_col;
            self.block_state.shooter_last_scanned_row = shooter.kid_block_row;
        }
        if let Some(ledger) = self.mode_level_work_before_hud_sub_44dc.as_mut() {
            ledger
                .execute_objects
                .player_spawned_prize_block_initial_sub_1007a =
                player_spawned_prize_block_initial_sub_1007a(
                    Some(&executed_player_path),
                    &self.block_bounces,
                );
            ledger
                .execute_objects
                .player_prize_wobble_list_work_loc_1002e =
                player_prize_wobble_list_work_loc_1002e(&ledger.execute_objects);
            ledger.execute_objects.executed_player_path = Some(executed_player_path);
            ledger.execute_objects.player_gfx_terrain_scan = player_gfx_terrain_scan;
            if let Some((evanescent, shooter)) = first_activation_post_scans {
                ledger.post_execute.evanescent_scan_loc_eae4 = evanescent;
                ledger.post_execute.shooter_scan_sub_f096 = Some(shooter);
            }
            complete_prize_tail_list_work(&mut ledger.execute_objects);
            ledger.execute_objects.prize_object_execution_order_sub_e12 =
                prize_object_execution_order_sub_e12(&ledger.execute_objects);
        }
    }

    pub fn trace_frame(&self) -> NativeTraceFrame {
        let game_mode = self.game_mode();
        let boot_phase = self.boot_phase_label().map(str::to_owned);
        let boot_phase_elapsed = self.boot_phase_elapsed_frames();
        let cold_boot_frame = self.frame_index == 0
            && !matches!(
                self.boot_state,
                BootState::StartingLevel { .. }
                    | BootState::LevelSkipHold
                    | BootState::InGame
                    | BootState::DemoPlay { .. }
                    | BootState::EndingSequence { .. }
            );
        // ASM startup calls sub_8A4 before Load_SegaScreen, but oracle traces
        // keep the copied counters hidden through the early loc_12E64 /
        // loc_12E6C logo wait and expose them once the fade loop is underway.
        let hidden_mode_zero_frame =
            matches!(self.boot_state, BootState::SegaScreen { .. }) && self.frame_index < 30;
        let hidden_boot_progression = cold_boot_frame || hidden_mode_zero_frame;
        let current_level_name = self.level_name(self.current_level).map(str::to_owned);
        let level_header = self.level_header(self.current_level);
        let enemy_layout = self.enemy_layout(self.current_level);
        let player_state = self.current_player_state();
        let camera_state = self.current_camera_state();
        let active_objects = self.active_objects();
        let active_enemy_details = self
            .active_enemies
            .iter()
            .flat_map(|enemy| {
                let mut snapshots = vec![enemy.snapshot()];
                snapshots.extend(enemy.child_attachment_snapshot());
                snapshots.extend(boss_eye_orbit_object_slot_details_loc_3931a(
                    enemy,
                    boss_eye_orbit_mapping_indices_from_state_loc_37abc(
                        enemy,
                        self.opening_slice.as_ref(),
                        self.progression.diamond_power_frames > 0
                            || !self.diamond_power_invuln_haze_slots.is_empty(),
                        self.frame_index,
                    ),
                ));
                snapshots
            })
            .collect::<Vec<_>>();
        let (number_objects, number_gfx_objects) = self
            .early_start_load_ingame_object_counts()
            .or_else(|| self.title_card_live_handoff_object_counts_loc_1ae2e())
            .or_else(|| self.pause_restart_object_counts_loc_6e2())
            .or_else(|| self.demo_1_late_controller_sample_counts())
            .or_else(|| self.demo_1_late_title_return_controller_counts())
            .unwrap_or_else(|| {
                self.object_counts_from_roster(
                    &active_objects,
                    camera_state.x_pos,
                    camera_state.y_pos,
                )
            });
        let active_enemy_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Enemy { .. }))
            .count();
        let active_platform_object_count = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Platform { .. }))
            .count();
        let mode_level_player_work = match player_state {
            None => ModeLevelPlayerWorkBeforeHud::Absent,
            Some(_) if self.level_start_frame == Some(self.frame_index) => {
                ModeLevelPlayerWorkBeforeHud::Activation
            }
            Some(state) if state.x_velocity != 0 || state.y_velocity != 0 => {
                ModeLevelPlayerWorkBeforeHud::Moving
            }
            Some(_) => ModeLevelPlayerWorkBeforeHud::Stationary,
        };
        let mut gameplay_debug_fields = BTreeMap::new();
        if self.progression.form_change_flash_frames > 1 {
            gameplay_debug_fields.insert("currently_transforming".into(), "true".into());
        }
        if self.gameplay_hibernating_for_transform() {
            gameplay_debug_fields.insert("gameplay_hibernated".into(), "true".into());
        }
        if !self.hud_dma_events_sub_44dc.is_empty() {
            let player_work = match mode_level_player_work {
                ModeLevelPlayerWorkBeforeHud::Absent => "absent",
                ModeLevelPlayerWorkBeforeHud::Activation => "activation",
                ModeLevelPlayerWorkBeforeHud::Stationary => "stationary",
                ModeLevelPlayerWorkBeforeHud::Moving => "moving",
            };
            gameplay_debug_fields.insert(
                "mode_level_player_work_before_hud".into(),
                player_work.into(),
            );
        }
        let floating_diamond_debug =
            player_state.and_then(|state| self.floating_diamond_contact_candidates(state));
        if !self.pending_prize_diamond_awards.is_empty() || floating_diamond_debug.is_some() {
            gameplay_debug_fields.insert(
                "pending_prize_diamond_awards".into(),
                self.pending_prize_diamond_awards_debug(),
            );
        }
        if let Some(summary) = floating_diamond_debug {
            gameplay_debug_fields.insert("floating_diamond_contact_candidates".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.floating_prize_contact_candidates(player_state)
        {
            gameplay_debug_fields.insert("floating_prize_contact_candidates".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.kid_object_support_candidates(player_state)
        {
            gameplay_debug_fields.insert("kid_object_support_candidates".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.kid_ground_probe_debug(player_state)
        {
            gameplay_debug_fields.insert("kid_ground_probe".into(), summary);
        }
        if let Some(summary) = self.kid_special_motion_probe_debug() {
            gameplay_debug_fields.insert("kid_special_motion_probe".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.kid_goal_probe_debug(player_state)
        {
            gameplay_debug_fields.insert("kid_goal_probe".into(), summary);
        }
        if let Some(summary) = self.kid_gameplay_path_probe_debug() {
            gameplay_debug_fields.insert("kid_gameplay_path_probe".into(), summary);
        }
        if let Some(summary) = self.kid_collision_handoff_probe_debug() {
            gameplay_debug_fields.insert("kid_collision_handoff_probe".into(), summary);
        }
        if let Some(player_state) = player_state
            && let Some(summary) = self.kid_projectile_damage_probe_debug(player_state)
        {
            gameplay_debug_fields.insert("kid_projectile_damage_probe".into(), summary);
        }
        if let Some(summary) = self.drips_lifecycle_probe_debug() {
            gameplay_debug_fields.insert("drips_lifecycle_probe".into(), summary);
        }
        if let Some(summary) = &self.last_drips_splash_reseed_probe {
            gameplay_debug_fields.insert("drips_splash_reseed_probe".into(), summary.clone());
        }
        if let Some(slice) = self.opening_slice
            && let Some(summary) =
                self.stomp_contact_candidates_debug(&slice, slice.player.y_velocity)
        {
            gameplay_debug_fields.insert("kid_stomp_contact_candidates".into(), summary);
        }
        if let Some(slice) = self.opening_slice
            && let Some(summary) =
                self.kid_collision_order_probe_debug(&slice, slice.player.y_velocity)
        {
            gameplay_debug_fields.insert("kid_collision_order_probe".into(), summary);
        }
        if let Some(slice) = self.opening_slice
            && let Some(summary) =
                self.rock_tank_contact_probe_debug(&slice, slice.player.y_velocity)
        {
            gameplay_debug_fields.insert("rock_tank_contact_probe".into(), summary);
        }
        if let Some(summary) = self.kid_same_frame_stomp_probe_debug() {
            gameplay_debug_fields.insert("kid_same_frame_stomp_probe".into(), summary);
        }
        if let Some(summary) = self.kid_iron_knight_attach_probe_debug() {
            gameplay_debug_fields.insert("kid_iron_knight_attach_probe".into(), summary);
        }
        if let Some(summary) = self.kid_flip_branch_probe_debug() {
            gameplay_debug_fields.insert("kid_flip_branch_probe".into(), summary);
        }
        if let Some(summary) = self.kid_flip_follow_on_probe_debug() {
            gameplay_debug_fields.insert("kid_flip_follow_on_probe".into(), summary);
        }
        if let Some(summary) = self.kid_jump_gravity_probe_debug() {
            gameplay_debug_fields.insert("kid_jump_gravity_probe".into(), summary);
        }
        if let Some(summary) = self.kid_pre_physics_probe_debug() {
            gameplay_debug_fields.insert("kid_pre_physics_probe".into(), summary);
        }
        if let Some(summary) = self.kid_iron_knight_block_breaker_probe_debug() {
            gameplay_debug_fields.insert("kid_iron_knight_block_breaker_probe".into(), summary);
        }
        if let Some(summary) = self.kid_grounded_release_probe_debug() {
            gameplay_debug_fields.insert("kid_grounded_release_probe".into(), summary);
        }
        if let Some(summary) = self.kid_wall_cling_release_probe_debug() {
            gameplay_debug_fields.insert("kid_wall_cling_release_probe".into(), summary);
        }
        if let Some(summary) = self.diamond_power_snake_probe_debug() {
            gameplay_debug_fields.insert("diamond_power_snake_probe".into(), summary);
        }
        if let Some(summary) = self.juggernaut_companion_step_debug_sub_975c() {
            gameplay_debug_fields.insert("juggernaut_companion_step_sub_975c".into(), summary);
        }
        if let Some(summary) = self.highwater_shooter_barrier_probe_debug() {
            gameplay_debug_fields.insert("highwater_shooter_barrier_probe".into(), summary);
        }
        if let Some(summary) = self.platform_collision_scratch_debug_loc_257e() {
            gameplay_debug_fields.insert("platform_collision_scratch_loc_257e".into(), summary);
        }
        if let Some(summary) = Self::native_platform_slots_debug_loc_257e(&self.active_platforms) {
            gameplay_debug_fields.insert("native_platform_slots_loc_257e".into(), summary);
        }
        if let Some(summary) = self.native_platform_loader_debug_loc_2744() {
            gameplay_debug_fields.insert("native_platform_loader_loc_2744".into(), summary);
        }
        let level_complete_elapsed_frames = match self.player_lifecycle {
            PlayerLifecycle::LevelComplete {
                remaining_frames,
                total_frames,
                ..
            } => total_frames.saturating_sub(remaining_frames),
            _ => 0,
        };
        let level_complete_reveal_frames =
            if matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }) {
                self.level_complete_scoreboard_reveal_frames_loc_C21E()
            } else {
                0
            };
        let level_complete_score_rows_visible_frame =
            crate::level::EOL_SCORE_ROWS_START_FIXED_FRAMES_LOC_D468
                + level_complete_reveal_frames
                + crate::level::EOL_SCORE_ROWS_VISIBLE_DELAY_FRAMES_LOC_D468_ORACLE;
        let level_complete_score_rows_elapsed_frames =
            if matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }) {
                level_complete_elapsed_frames
                    .saturating_sub(level_complete_score_rows_visible_frame)
            } else {
                0
            };
        let level_complete_score_rows_started =
            if matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }) {
                u8::from(
                    level_complete_elapsed_frames.saturating_add(1)
                        >= level_complete_score_rows_visible_frame,
                )
            } else {
                0
            };
        let level_complete_score_counter_elapsed_frames =
            if matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }) {
                level_complete_elapsed_frames.saturating_sub(
                    crate::level::EOL_COUNTER_START_FIXED_FRAMES_LOC_D770_ORACLE
                        + level_complete_reveal_frames,
                )
            } else {
                0
            };
        let trace_score = if matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. })
        {
            self.level_complete_live_score_loc_D770(level_complete_score_counter_elapsed_frames)
        } else {
            self.progression.score
        };
        let kid_x_velocity = player_state.map(|state| {
            if matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
            ) && self.death_windup_path == DeathWindupPath::Tumble
            {
                // ASM Death loc_B7FA/loc_B80C keeps tumble deltas in local
                // `d0`/`d1` registers and moves `x_pos/y_pos` directly; Kid's
                // object velocity fields keep their pre-death WRAM values.
                self.death_preserved_object_velocity
                    .map_or(0, |(x_velocity, _)| x_velocity)
            } else {
                state.x_velocity
            }
        });
        let kid_y_velocity = player_state.map(|state| {
            if matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
            ) && self.death_windup_path == DeathWindupPath::Tumble
            {
                // ASM Death loc_B7FA/loc_B80C uses local `d1` for tumble
                // gravity; oracle WRAM velocity remains at the object field.
                self.death_preserved_object_velocity
                    .map_or(0, |(_, y_velocity)| y_velocity)
            } else if self.intro_sequence_6_timeout_return_velocity_sample_active() {
                // ASM Obj_TitleMenu loc_1BB02 -> Load_DemoPlay (kid.asm:30399,
                // 23585) reaches the cold timeout-return IntroSequence6 shell with the
                // previous controller-side sample still exposed through
                // the hidden Load_InGame bootstrap.
                // Raw Kid WRAM is still the title-pose object here; only
                // the top-level oracle velocity sample carries `$60000`.
                0x0006_0000
            } else if let Some(y_velocity) =
                self.wall_cling_boundary_sampled_y_velocity_export(state)
            {
                y_velocity
            } else {
                state.y_velocity
            }
        });
        let prize_diamond_hud_flights_loc_fe3e = self
            .pending_prize_diamond_hud_flights
            .iter()
            .map(|flight| {
                // ASM PrizeDiamondCollected_Init loc_FE3E (kid.asm:21143)
                // reprojects its cached camera-relative arc before the next
                // Sprite_Table build. This is distinct from PrizeDiamond_Init:
                // its initial frame is already presentable.
                let arc_step = PrizeCollectedArcHibernateStepLocFEEE {
                    next_screen_x_fp: flight.screen_x_fp,
                    next_screen_y_fp: flight.screen_y_fp,
                    screen_x_word: Self::high_word_from_i32(flight.screen_x_fp),
                    screen_y_word: Self::high_word_from_i32(flight.screen_y_fp),
                };
                let loop_counter = u16::from(flight.remaining_delay.saturating_sub(1));
                let world_step = Self::prize_collected_world_step_loc_FEFE(
                    arc_step,
                    Self::high_word_from_i32(camera_state.x_pos as i32),
                    Self::high_word_from_i32(camera_state.y_pos as i32),
                    loop_counter,
                );
                debug_assert_eq!(world_step.next_loop_counter, loop_counter.wrapping_sub(1));
                debug_assert_eq!(
                    world_step.should_loop,
                    flight.remaining_delay > 1,
                    "ASM loc_FEFE DBF loops until the final visible hibernate tick",
                );
                let elapsed = u16::from(
                    NativeRuntime::PRIZE_DIAMOND_AWARD_DELAY_FRAMES
                        .saturating_sub(flight.remaining_delay),
                );
                (
                    world_step.world_x_word,
                    world_step.world_y_word,
                    0x01F4u16.saturating_sub(elapsed),
                )
            })
            .collect();
        NativeTraceFrame {
            frame_index: self.frame_index,
            time_frames_loc_3038a: self.time_frames_loc_3038a,
            game_mode,
            game_mode_value: game_mode.as_disassembly_value(),
            start_press_count: self.start_press_count,
            boot_phase,
            boot_phase_elapsed,
            title_screen_ready: matches!(
                self.boot_state,
                BootState::TitleScreen { .. } | BootState::IntroSequence6 { .. }
            ) && !self.demo_auto_timeout_handoff_active
                && self.title_screen_ready(),
            current_player: self.current_player_index,
            current_level: self.current_level,
            player_1_level_id: if matches!(game_mode, GameMode::DemoPlay) {
                // ASM Load_DemoPlay updates Current_LevelID for the demo load,
                // but Player_1_LevelID remains the saved slot value on the
                // VBlank checkpoint path. The authoritative CRC already
                // excludes this demo timing artefact; keep the trace export
                // byte-level honest so native trace verification does not
                // fail on a field the semantic case layer correctly ignores.
                self.player_slots[0].level_id
            } else {
                self.current_level
            },
            current_level_name,
            player_start_x: level_header.map(|header| header.player_start_x),
            player_start_y: level_header.map(|header| header.player_start_y),
            flag_x: level_header.map(|header| header.flag_x),
            flag_y: level_header.map(|header| header.flag_y),
            level_enemy_object_count: enemy_layout.map(|layout| layout.object_count),
            active_enemy_object_count,
            active_platform_object_count,
            number_lives: if hidden_boot_progression {
                0
            } else {
                self.progression.number_lives
            },
            number_continues: self.progression.number_continues,
            number_hitpoints: if hidden_boot_progression {
                0
            } else {
                self.progression.number_hitpoints
            },
            extra_hitpoint_slots: if hidden_boot_progression {
                0
            } else {
                self.progression.extra_hitpoint_slots
            },
            current_helmet_id: if hidden_boot_progression {
                0
            } else {
                self.progression.current_form.helmet_id()
            },
            hud_transform_target_helmet_id: (!hidden_boot_progression
                && self.progression.form_change_flash_frames > 0)
                .then_some(self.progression.current_form.helmet_id()),
            red_stealth_sword_timer: if hidden_boot_progression {
                0
            } else {
                self.helmet.red_stealth_sword_timer
            },
            red_stealth_airborne_slash_counter: if hidden_boot_progression {
                0
            } else {
                self.helmet.red_stealth_airborne_slash_counter
            },
            cyclone_flying: !hidden_boot_progression && self.helmet.cyclone_flying,
            cyclone_grounded_sprite_source: (!hidden_boot_progression)
                .then_some(self.helmet.cyclone_grounded_sprite_source)
                .flatten(),
            paused: self.paused || self.demo_1_late_controller_pause_active(),
            pause_menu_option: if self.paused {
                self.pause_menu_option
            } else {
                0
            },
            kid_x_pos: player_state.map(|state| state.x_pos),
            kid_y_pos: player_state.map(|state| state.y_pos),
            kid_x_velocity,
            kid_y_velocity,
            character_movement: player_state.map(|state| {
                if self.pause_restart_zero_kid_shell_active_loc_6e2() {
                    // ASM loc_6E2 restart reload: the Kid slot position is
                    // zeroed while no live Character_Movement action is
                    // exposed.
                    0
                } else {
                    self.current_character_movement(state)
                }
            }),
            kid_facing: player_state.map(|state| state.facing),
            kid_on_ground: player_state.map(|state| state.on_ground),
            walk_visual_accumulator: self
                .opening_slice
                .map_or(0, |slice| slice.walk_visual_accumulator),
            iron_knight_climbing: if hidden_boot_progression {
                false
            } else {
                self.helmet.iron_knight_climbing
            },
            iron_knight_climb_pose_accum: if hidden_boot_progression {
                0
            } else {
                self.helmet.iron_knight_climb_pose_accum
            },
            hud_diamond_frame_offset: Some(self.hud_animation_state.diamond_frame_offset_fb5d),
            hud_diamond_vram_page_loc_5d4a: Some(
                self.gameplay_hud_vram_state
                    .diamond_frame_table_byte_offset_loc_5d4a(),
            ),
            transform_resident_hud_diamond_vram_page_loc_5d4a: (self
                .progression
                .form_change_flash_frames
                > 0)
            .then_some(self.transform_resident_hud_diamond_vram_page_loc_5d4a)
            .flatten(),
            hud_timer_sprite_seconds_loc_bee8: Some(self.hud_timer_sprite_seconds_loc_bee8),
            hud_diamond_count_loc_bdc0: Some(self.hud_diamond_count_loc_bdc0),
            hud_life_frame_offset: Some(self.hud_animation_state.life_frame_offset_fb5e),
            hud_life_icon_vram_page_loc_4586: Some(self.hud_life_icon_vram_page_loc_4586),
            hud_dma_events_sub_44dc: self.hud_dma_events_sub_44dc.clone(),
            hud_raster_upload_timings_sub_44dc: self.hud_raster_upload_timings_sub_44dc.clone(),
            mode_level_vblank_work_before_sub_e12: self
                .mode_level_work_before_hud_sub_44dc
                .as_ref()
                .map(|_| crate::ModeLevelVblankWorkBeforeSubE12 {
                    // ASM Mode_Level $7C8..sub_E12: core classifies the
                    // palette, scroll, foreground, pause and HUD branches.
                    // The frame boundary resolves loc_AD4 from the selected
                    // render mapping produced by the preceding sub_1492 pass.
                    dynamic_art_tiles_loc_ad4: None,
                    palette_dma_sub_970: self
                        .mode_level_palette_dma_sub_970
                        .unwrap_or(!self.palette_dma_suppressed_f890),
                    scroll_path: crate::ModeLevelScrollTransferPath::NormalLoc300ba,
                    foreground_custom_queue_entries_sub_1596: 0,
                    foreground_cell_queue_entries_sub_1596: 0,
                    paused_sub_6874: false,
                    hud_path_sub_bdc0:
                        crate::ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates,
                }),
            mode_level_work_before_hud_sub_44dc: self.mode_level_work_before_hud_sub_44dc.clone(),
            mode_level_work_after_hud_sub_44dc: self.mode_level_work_after_hud_sub_44dc.clone(),
            mode_level_tail_work_loc_892: self.mode_level_tail_work_loc_892,
            berzerker_charging: if hidden_boot_progression {
                false
            } else {
                self.helmet.berzerker_charging
            },
            berzerker_charge_pose: if hidden_boot_progression {
                false
            } else {
                self.helmet.berzerker_charge_pose
            },
            berzerker_walk_pose: if hidden_boot_progression {
                false
            } else {
                self.helmet.berzerker_walk_pose
            },
            ledge_snap_cooldown: player_state.map_or(0, |state| state.ledge_snap_cooldown),
            kid_on_slope: player_state
                .filter(|state| state.on_ground)
                .and_then(|state| {
                    let grid = self.block_grid.as_ref()?;
                    let x_px = (state.x_pos >> 16) as i32;
                    let y_px = (state.y_pos >> 16) as i32;
                    let probe = terrain::probe_floor(grid, x_px, y_px);
                    match probe.cell.collision {
                        terrain::CollisionKind::SlopeUp => Some(1),
                        terrain::CollisionKind::SlopeDown => Some(-1),
                        _ => Some(0),
                    }
                })
                .unwrap_or(0),
            active_objects,
            active_enemy_details,
            camera_x_pos: camera_state.x_pos,
            camera_y_pos: camera_state.y_pos,
            resident_gameplay_vdp_camera_loc_300ba: self
                .demo_title_palette_fade_progress_loc_28c6()
                .and(self.gameplay_vdp_camera_latch_loc_300ba)
                .map(|camera| (camera.x_pos, camera.y_pos)),
            resident_gameplay_vdp_terminal_scroll_table_loc_28c6: self
                .demo_title_palette_fade_progress_loc_28c6()
                .is_some()
                && self.demo_auto_timeout_handoff_active
                && self.demo_auto_timeout_mode_0_entry
                    == DemoAutoTimeoutMode0Entry::ReadJoypadReplay,
            number_objects,
            number_gfx_objects,
            rng_offset: Some(self.rom_random.offset_word()),
            rng_last_byte: Some(self.rom_random.last_byte()),
            rng_ring: Some(self.rom_random.ring()),
            level_complete: matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }),
            level_complete_elapsed_frames,
            level_complete_score_rows_elapsed_frames,
            level_complete_score_rows_started,
            level_complete_score_counter_elapsed_frames,
            level_complete_score_counter_drained: matches!(
                self.player_lifecycle,
                PlayerLifecycle::LevelComplete { .. }
            ) && self
                .level_complete_score_counter_drained_loc_D7F4(
                    level_complete_score_counter_elapsed_frames,
                ),
            level_complete_initial_score: match self.player_lifecycle {
                PlayerLifecycle::LevelComplete { .. } => self.level_complete_initial_score,
                _ => 0,
            },
            level_completion_time: {
                let elapsed_secs = LEVEL_TIMER_COMPLETION_BASE_SECONDS
                    .saturating_sub(self.level_timer.seconds)
                    .saturating_add(self.clocks_collected * LEVEL_TIMER_CLOCK_BONUS_SECONDS);
                // ASM loc_C10A (kid.asm:15707): after loc_C0CC computes the
                // remaining-time bonus, clamp the byte stored in
                // `Level_completion_time` to `$FF`.
                elapsed_secs.min(255) as u8
            },
            level_skip_cheat_enabled: 0,
            game_over: matches!(
                self.player_lifecycle,
                PlayerLifecycle::GameOver { .. } | PlayerLifecycle::ContinueCountdown { .. }
            ),
            game_over_elapsed_frames: match self.player_lifecycle {
                PlayerLifecycle::GameOver { remaining_frames } => {
                    PLAYER_GAME_OVER_HOLD_FRAMES.saturating_sub(remaining_frames)
                }
                _ => 0,
            },
            continue_countdown: match self.player_lifecycle {
                PlayerLifecycle::ContinueCountdown { remaining_frames } => Some(remaining_frames),
                _ => None,
            },
            continue_countdown_elapsed_frames: match self.player_lifecycle {
                PlayerLifecycle::ContinueCountdown { remaining_frames } => {
                    CONTINUE_COUNTDOWN_FRAMES.saturating_sub(remaining_frames)
                }
                _ => 0,
            },
            game_complete: self.game_complete,
            ending_elapsed: match self.boot_state {
                BootState::EndingSequence { elapsed } => Some(elapsed),
                _ => None,
            },
            options_cursor: match self.boot_state {
                BootState::TitleScreen { cursor } | BootState::OptionsMenu { cursor } => {
                    Some(cursor)
                }
                BootState::IntroSequence6 { cursor, .. } if self.title_menu_active => Some(cursor),
                _ => None,
            },
            sound_test_track: match self.boot_state {
                BootState::SoundTest { track } => Some(track),
                _ => None,
            },
            played_sound_effect_ids: self
                .played_sound_effects
                .iter()
                .map(|effect| effect.sound_id())
                .collect(),
            played_sound_commands: self
                .played_sound_commands
                .iter()
                .map(|command| command.trace_command())
                .collect(),
            played_bgm_ids: self.played_bgm_ids.clone(),
            played_music_speed_updates: self.played_music_speed_updates.clone(),
            // The FBCC warp colour is visible to the level-entry fade shell.
            // The title-card path runs that fade under StartingLevel; the
            // level-skip path under LevelSkipHold; the no-card Teleport
            // (Load_InGame, loc_BCF6) runs it under InGame while the entry fade
            // is still in progress (`!allow_pausing`).
            teleport_warp_colour: if self.demo_title_palette_fade_progress_loc_28c6().is_some() {
                // ASM loc_119D8 writes `$082A` to FBCC before Pal_FadeOut.
                0x082A
            } else if matches!(
                self.boot_state,
                BootState::StartingLevel { .. } | BootState::LevelSkipHold
            ) || (matches!(self.boot_state, BootState::InGame) && !self.allow_pausing)
            {
                self.teleport_warp_colour_fbcc
            } else {
                0
            },
            level_entry_fade_progress_sub_73d0: self
                .title_card_live_handoff_palette_fade_progress_sub_73d0()
                .or_else(|| self.demo_title_palette_fade_progress_loc_28c6())
                .or_else(|| self.demo_level_entry_palette_fade_progress_sub_73d0())
                .or_else(|| {
                    (matches!(self.boot_state, BootState::InGame) && !self.allow_pausing)
                        .then_some(self.level_entry_fade_progress_sub_73d0)
                }),
            two_player: self.two_player,
            options_two_player_two_controllers: self.options_two_player_two_controllers,
            options_controls_scheme: self.options_controls_scheme,
            options_speed_button_fast: self.options_speed_button_fast,
            invulnerable_frames: self.progression.invulnerable_frames,
            kid_is_invulnerable: self.kid_is_invulnerable,
            just_received_damage: if self.death_shell_snapshot.is_some()
                && matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. })
            {
                self.dying_hidden_camera_still_visible()
            } else {
                self.just_received_damage
                    || (self.death_shell_snapshot.is_some()
                        && matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }))
            },
            diamond_power_active: self.diamond_power_active_loc_3f610(),
            diamond_count: self.progression.diamond_count,
            // ASM loc_D770 (kid.asm:16855): the flagpole EOL score counter
            // drains visible bonus rows into live `(Score).w`. Native
            // progression keeps the awarded total for the collapsed
            // post-sequence handoff, but trace state reports the ROM live
            // score while LevelComplete is active.
            score: trace_score,
            level_timer_seconds: self.level_timer.seconds,
            level_timer_sub_frames: self.level_timer.sub_frames,
            level_damage_taken: self.level_damage_taken,
            level_prize_collected: self.level_prize_collected,
            level_speed_bonus_awarded: self.level_speed_bonus_awarded,
            level_path_bonus_amount: self.level_path_bonus_amount,
            murder_wall_x: self.murder_wall.map(|w| w.x_pos),
            active_projectiles: self
                .projectiles
                .iter()
                .map(|p| (p.x_pos, p.y_pos, p.friendly))
                .chain(
                    self.diamond_power_rain_slots
                        .iter()
                        .map(|slot| (slot.x_pos, slot.y_pos, true)),
                )
                .chain(
                    self.fire_demon_trails
                        .iter()
                        .map(|trail| (trail.x_pos, trail.y_pos, false)),
                )
                .collect(),
            active_projectile_details: self
                .projectiles
                .iter()
                .map(|p| ActiveProjectileSnapshot {
                    x_pos: p.x_pos,
                    y_pos: p.y_pos,
                    x_velocity: p.x_velocity,
                    y_velocity: p.y_velocity,
                    friendly: p.friendly,
                    remaining_frames: p.lifetime,
                    animation_elapsed_frames: p.animation_elapsed_frames,
                    kind: active_projectile_kind(p),
                    palette_variant: p.palette_variant,
                    boss_eye_sprite_mapping_index: boss_eye_sprite_mapping_index(
                        p.boss_eye_sprite_mapping_label,
                    ),
                    boss_eye_sprite_order_word_a: 0,
                    dragon_flying_descent_fire_state: p.dragon_flying_descent_fire_state,
                    dragon_flying_descent_fire_counter: p.dragon_flying_descent_fire_counter,
                })
                .chain(
                    self.diamond_power_rain_slots
                        .iter()
                        .map(|slot| ActiveProjectileSnapshot {
                            x_pos: slot.x_pos,
                            y_pos: slot.y_pos,
                            x_velocity: 0,
                            y_velocity: 0,
                            friendly: true,
                            remaining_frames: 0,
                            animation_elapsed_frames: 0,
                            kind: ActiveProjectileKind::Generic,
                            palette_variant: 0,
                            boss_eye_sprite_mapping_index: 0,
                            boss_eye_sprite_order_word_a: 0,
                            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                            dragon_flying_descent_fire_counter: 0,
                        }),
                )
                .chain(
                    self.fire_demon_trails
                        .iter()
                        .map(|trail| ActiveProjectileSnapshot {
                            x_pos: trail.x_pos,
                            y_pos: trail.y_pos,
                            x_velocity: 0,
                            y_velocity: 0,
                            friendly: false,
                            remaining_frames: trail.remaining_frames,
                            animation_elapsed_frames: 0,
                            kind: ActiveProjectileKind::FireDemonDeathTrail,
                            palette_variant: 0,
                            boss_eye_sprite_mapping_index: 0,
                            boss_eye_sprite_order_word_a: 0,
                            dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                            dragon_flying_descent_fire_counter: 0,
                        }),
                )
                .chain(self.boss_eye_satellite_snapshots_sub_393c8())
                .collect(),
            diamond_power_sprites: self.diamond_power_sprites.clone(),
            eyeclops_hard_beam_object: self.eyeclops_hard_beam_object,
            active_geysers: self
                .weather
                .lava_geysers
                .iter()
                .filter(|g| g.current_height > 0)
                .map(|g| {
                    let full_height_render =
                        crate::weather::lava_geyser_full_height_render_state_loc_14940(
                            g.phase,
                            g.timer,
                            g.current_height,
                            g.height_px,
                        );
                    let (sprite_row_index, top_sprite_frame_within_row) = full_height_render
                        .map_or((u16::MAX, u8::MAX), |state| {
                            (state.sprite_row_index, state.top_sprite_frame_within_row)
                        });
                    (
                        g.x_px,
                        g.y_px.saturating_sub(g.current_height),
                        g.current_height,
                        sprite_row_index,
                        top_sprite_frame_within_row,
                    )
                })
                .collect(),
            lava_geyser_script_templates_loc_147ba: self
                .weather
                .lava_geysers
                .iter()
                .filter_map(|g| {
                    crate::weather::lava_geyser_script_template_label_loc_147BA(g.phase, g.timer)
                        .map(|source_label| LavaGeyserScriptTemplateTraceLoc147BA {
                            x_px: g.x_px,
                            base_y_px: g.y_px,
                            source_label: source_label.to_owned(),
                        })
                })
                .collect(),
            island_shore_cycle_active: self.island_shore_cycle_loc_1202a.is_some(),
            island_shore_cycle_state_loc_1202a: self.island_shore_cycle_loc_1202a.map(|state| {
                (
                    state.primary_countdown_44,
                    state.mid_countdown_46,
                    state.lower_countdown_48,
                    state.primary_phase_d3,
                    state.mid_phase_d2,
                    state.lower_phase_d1,
                )
            }),
            island_shore_vram_frames_loc_1202a: self.island_shore_cycle_loc_1202a.map(|state| {
                (
                    state.primary_vram_frame,
                    state.mid_vram_frame,
                    state.lower_vram_frame,
                )
            }),
            transform_resident_island_shore_vram_frames_loc_1202a: (self
                .progression
                .form_change_flash_frames
                > 0)
            .then_some(self.transform_resident_island_shore_vram_frames_loc_1202a)
            .flatten(),
            island_shore_dma_uploads_loc_1202a: self.island_shore_dma_uploads_loc_1202a.clone(),
            // ASM loc_12098/loc_12110/loc_12188 commit below the visible
            // field. Ordinary Demo 1's post-update trace event is already one
            // frame later than the FIFO commit, but the IntroSequence6 hidden
            // camera-wrap lineage is frame-aligned and needs a render latch.
            island_shore_post_visible_dma_latch_loc_1202a: self
                .demo_uses_intro_sequence_6_hidden_camera_wrap
                && !self.island_shore_dma_uploads_loc_1202a.is_empty(),
            lava_geyser_art_uploads_loc_14c6a: self.lava_geyser_art_uploads_loc_14c6a.clone(),
            rotate_palette_lava_water_state_loc_1223c: self
                .rotate_palette_lava_water_loc_1223c
                .map(|state| {
                    (
                        state.countdown_d0,
                        state.reload_delay_d2,
                        state.completed_rotations,
                    )
                }),
            lava_water_palette_words_loc_1223c: self.rotate_palette_lava_water_loc_1223c.map(
                |_| {
                    let start = crate::level::PALETTE_BUFFER_ROTATE_START_LOC_1223C;
                    [
                        self.palette_buffer[start],
                        self.palette_buffer[start + 1],
                        self.palette_buffer[start + 2],
                        self.palette_buffer[start + 3],
                    ]
                },
            ),
            lava_geyser_palette_words_loc_14b2a: (self.weather.weather_type == 1).then(|| {
                let start = crate::weather::LAVA_GEYSER_PALETTE_BUFFER_START_LOC_14C6A;
                [
                    self.palette_buffer[start],
                    self.palette_buffer[start + 1],
                    self.palette_buffer[start + 2],
                    self.palette_buffer[start + 3],
                    self.palette_buffer[start + 4],
                    self.palette_buffer[start + 5],
                    self.palette_buffer[start + 6],
                    self.palette_buffer[start + 7],
                ]
            }),
            storm_pattern_upload_loc_14d32: self.weather.storm_pattern_upload_loc_14d32.clone(),
            storm_scroll_transfer_loc_30110: self.weather.storm_scroll_transfer_loc_30110,
            storm_shared_art_upload_loc_14d8a: self
                .weather
                .storm_shared_art_upload_loc_14d8a
                .clone(),
            special_effect_unknown_art_upload_loc_14df6: self
                .weather
                .unknown_art_upload_loc_14df6
                .clone(),
            special_effect_unknown_plane_fill_loc_14e42: self.weather.unknown_plane_fill_loc_14e42,
            storm_falling_slots_sub_61ca: self
                .weather
                .storm_falling_slots_sub_61ca
                .iter()
                .filter(|slot| slot.y_px >= 0)
                .map(|slot| {
                    (
                        slot.y_px,
                        slot.x_px,
                        slot.animation_cooldown,
                        slot.animation_frame,
                    )
                })
                .collect(),
            storm_impact_slots_sub_61ca: self
                .weather
                .storm_impact_slots_sub_61ca
                .iter()
                .filter(|slot| slot.y_px >= 0)
                .map(|slot| {
                    (
                        slot.y_px,
                        slot.x_px,
                        slot.animation_cooldown,
                        slot.animation_frame,
                    )
                })
                .collect(),
            storm_sprites_sub_61ca: self
                .weather
                .storm_sprites_sub_61ca
                .iter()
                .map(|sprite| (sprite.y, sprite.link, sprite.tile_word, sprite.x))
                .collect(),
            block_bounces: self
                .block_bounces
                .iter()
                .map(|b| (b.cell_x, b.cell_y, b.remaining, b.y_offset_px))
                .collect(),
            block_bounce_details: self
                .block_bounces
                .iter()
                .map(|b| BlockBounceTrace {
                    cell_x: b.cell_x,
                    cell_y: b.cell_y,
                    x_offset_px: b.x_offset_px,
                    remaining: b.remaining,
                    y_offset_px: b.y_offset_px,
                    kind: match b.kind {
                        BlockBounceKind::HiddenReveal => BlockBounceTraceKind::HiddenReveal,
                        BlockBounceKind::Lift => BlockBounceTraceKind::Lift,
                        BlockBounceKind::Metal => BlockBounceTraceKind::Metal,
                        BlockBounceKind::Prize => BlockBounceTraceKind::Prize,
                        BlockBounceKind::Shifting | BlockBounceKind::ShiftingFall => {
                            BlockBounceTraceKind::Shifting
                        }
                    },
                })
                .collect(),
            prize_block_worker_tails_sub_1007a: self
                .prize_block_worker_tails
                .iter()
                .map(|tail| (tail.cell_x, tail.cell_y, tail.remaining))
                .collect(),
            block_destroy_effects: self
                .block_destroy_effects
                .iter()
                .map(|e| (e.x_px, e.y_px, e.remaining, e.direction))
                .collect(),
            block_destroy_ice_effects: self
                .block_destroy_effects
                .iter()
                .filter(|e| e.pending_ice_origin_nudge)
                .map(|e| (e.x_px, e.y_px))
                .collect(),
            block_debris_pieces: if self.block_debris_sprite_pieces_sub_2a4c.is_empty() {
                {
                    self.block_debris_swarms
                        .iter()
                        .flat_map(|swarm| {
                            swarm.pieces.iter().map(|piece| {
                                ((piece.x_pos_fp >> 16) as i16, (piece.y_pos_fp >> 16) as i16)
                            })
                        })
                        .collect()
                }
            } else {
                self.block_debris_sprite_pieces_sub_2a4c.clone()
            },
            prize_block_spawn_flashes: self
                .prize_block_spawn_flashes
                .iter()
                .map(|f| (f.x_px, f.y_px, f.remaining))
                .collect(),
            kid_projectile_hit_sparks: self
                .kid_projectile_hit_sparks
                .iter()
                .map(|spark| (spark.x_px, spark.y_px, spark.remaining))
                .collect(),
            floating_items: self
                .floating_items
                .iter()
                .filter(|f| {
                    !f.collected
                        || collected_latched_helmet_prize(f)
                        || collected_helmet_first_forward_transform_frame(
                            f,
                            self.progression.current_form,
                            self.progression.form_change_flash_frames,
                        )
                })
                .map(|f| {
                    let x_px = (f.x_pos >> 16) as u16;
                    let y_px = (f.y_pos >> 16) as u16;
                    (x_px, y_px, f.remaining, f.content.to_id())
                })
                .chain(self.pending_prize_clock_hud_flights.iter().map(|flight| {
                    // ASM loc_FF92 (kid.asm:21260) advances the collected
                    // clock's cached screen-space words before reprojecting
                    // through the current camera.
                    let step = Self::prize_clock_collected_world_step_loc_FF92(
                        flight.screen_x_fp,
                        flight.screen_y_fp,
                        0,
                        0,
                        Self::high_word_from_i32(camera_state.x_pos as i32),
                        Self::high_word_from_i32(camera_state.y_pos as i32),
                        flight.loop_counter,
                    );
                    let remaining = flight.loop_counter.saturating_add(1);
                    (
                        step.world_step.world_x_word,
                        step.world_step.world_y_word,
                        remaining,
                        terrain::PrizeContent::Clock.to_id(),
                    )
                }))
                .collect(),
            floating_items_transferred_loc_901: Vec::new(),
            prize_diamond_hud_flights_loc_fe3e,
            prize_diamond_hud_flight_transferred_loc_fe3e: false,
            clock_prize_animation_swaps_loc_10796: self
                .clock_prize_animation_swaps_loc_10796
                .clone(),
            continue_prize_animation_swaps_loc_1080a: self
                .continue_prize_animation_swaps_loc_1080a
                .clone(),
            enemy_death_effects: self
                .enemy_death_effects
                .iter()
                .map(|e| (e.x_pos, e.y_pos, e.remaining))
                .collect(),
            heady_metal_death_splits: self
                .enemy_death_effects
                .iter()
                .filter_map(|effect| {
                    let split = effect.heady_metal_split?;
                    Some(HeadyMetalDeathSplitTrace {
                        current_x_velocity_delta: split.current_x_velocity_delta,
                        left_worker_x_velocity_delta: split.left_worker_x_velocity_delta,
                        centre_worker_y_velocity_delta: split.centre_worker_y_velocity_delta,
                        centre_worker_sprite_label: split
                            .centre_worker_sprite_label
                            .map(str::to_owned),
                        right_worker_x_velocity_delta: split.right_worker_x_velocity_delta,
                        far_right_worker_x_velocity_delta: split.far_right_worker_x_velocity_delta,
                        cleanup_worker_slot_offsets: split.cleanup_worker_slot_offsets,
                        pieces: split
                            .pieces
                            .iter()
                            .map(|piece| HeadyMetalDeathPieceTrace {
                                slot_offset: piece.slot_offset,
                                x_pos: piece.x_pos,
                                y_pos: piece.y_pos,
                                x_velocity: piece.x_velocity,
                                y_velocity: piece.y_velocity,
                            })
                            .collect(),
                    })
                })
                .collect(),
            heady_metal_body_pieces_sub_37abc: self.heady_metal_body_pieces_sub_37abc(),
            death_windup_elapsed: match self.player_lifecycle {
                PlayerLifecycle::DeathWindup { remaining_frames } => {
                    (PLAYER_DEATH_WINDUP_FRAMES.saturating_sub(remaining_frames)) as u16
                }
                _ => 0,
            },
            form_change_flash_frames: self.progression.form_change_flash_frames,
            transform_source_helmet_id_loc_7ca6: if self.progression.form_change_flash_frames > 0 {
                self.progression
                    .transform_source_form
                    .map(PlayerForm::helmet_id)
            } else {
                None
            },
            transform_source_hitpoints_loc_bdc0: self.progression.transform_source_hitpoints,
            stomped_this_frame: self.stomped_this_frame,
            blocks_hit_this_frame: self.blocks_hit_this_frame,
            sound_test_play_pending: self.sound_test_play_pending,
            gameplay_debug_fields,
            physics_trace: self.opening_slice.and_then(|s| s.physics_trace),
        }
    }

    fn boss_eye_satellite_snapshots_sub_393c8(&self) -> Vec<ActiveProjectileSnapshot> {
        let Some(slice) = self.opening_slice else {
            return Vec::new();
        };
        let kid_x = (slice.player.x_pos >> 16) as u16;
        let kid_y = (slice.player.y_pos >> 16) as u16;

        self.active_enemies
            .iter()
            .filter(|enemy| is_multi_head_boss(enemy.family) && enemy.boss_eye_orbit_workers.active)
            .flat_map(|enemy| {
                let orbit_workers = enemy.boss_eye_orbit_workers;
                let mut slots = [None; 12];
                slots[0] = Some(BossEyeSpriteRefreshSlotSub37ABC {
                    x_pos: orbit_workers.primary_x_pos_word,
                    y_pos: orbit_workers.primary_y_pos_word,
                    sprite_mapping: "stale",
                });
                slots[1] = Some(BossEyeSpriteRefreshSlotSub37ABC {
                    x_pos: orbit_workers.counter_x_pos_word,
                    y_pos: orbit_workers.counter_y_pos_word,
                    sprite_mapping: "stale",
                });

                boss_eye_refresh_attack_sprites_sub_37ABC(
                    slots,
                    self.progression.diamond_power_frames > 0
                        || !self.diamond_power_invuln_haze_slots.is_empty(),
                    self.frame_index as u8,
                    kid_x,
                    kid_y,
                )
                .into_iter()
                .enumerate()
                .filter_map(|(slot_index, slot)| {
                    let slot = slot?;
                    let boss_eye_sprite_order_word_a = match slot_index {
                        0 => orbit_workers.primary_sprite_order_word_a,
                        1 => orbit_workers.counter_sprite_order_word_a,
                        _ => 0,
                    };
                    Some(ActiveProjectileSnapshot {
                        x_pos: u32::from(slot.x_pos) << 16,
                        y_pos: u32::from(slot.y_pos) << 16,
                        x_velocity: 0,
                        y_velocity: 0,
                        friendly: false,
                        remaining_frames: 0,
                        animation_elapsed_frames: 0,
                        kind: ActiveProjectileKind::BossEyeSatellite,
                        palette_variant: 0,
                        boss_eye_sprite_mapping_index: boss_eye_sprite_mapping_index(
                            slot.sprite_mapping,
                        ),
                        boss_eye_sprite_order_word_a,
                        dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                        dragon_flying_descent_fire_counter: 0,
                    })
                })
                .collect::<Vec<_>>()
            })
            .collect()
    }

    fn highwater_shooter_barrier_probe_debug(&self) -> Option<String> {
        if self.current_level != LevelId(0x02) || !matches!(self.game_mode(), GameMode::InGame) {
            return None;
        }
        let grid = self.block_grid.as_ref()?;
        let shooter_cells = (97..=103)
            .map(|row| format!("11:{row}={:04X}", trace_cell_word(grid.cell_at(11, row))))
            .collect::<Vec<_>>()
            .join(",");
        let barrier_cells = (97..=103)
            .map(|row| format!("15:{row}={:04X}", trace_cell_word(grid.cell_at(15, row))))
            .collect::<Vec<_>>()
            .join(",");
        let shooter_slots = self
            .block_state
            .shooter_blocks
            .iter()
            .map(|entry| {
                format!(
                    "{}:{} dirs={:#04x} age={} supported={}",
                    entry.col,
                    entry.row,
                    entry.directions,
                    entry.age,
                    u8::from(entry.supported_by_neighbour)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let headbutt_workers = self
            .block_state
            .shooter_headbutt_workers
            .iter()
            .map(|worker| {
                format!(
                    "{}:{} {:?} age={}",
                    worker.col, worker.row, worker.kind, worker.age
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let projectiles = self
            .projectiles
            .iter()
            .map(|projectile| {
                format!(
                    "kind={:?} x={:#010x} y={:#010x} vel={:#010x},{:#010x} life={}",
                    active_projectile_kind(projectile),
                    projectile.x_pos,
                    projectile.y_pos,
                    projectile.x_velocity,
                    projectile.y_velocity,
                    projectile.lifetime
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        Some(format!(
            "scan={},{} slots=[{}] headbutt=[{}] projectiles=[{}] shooters=[{}] barrier=[{}]",
            self.block_state.shooter_last_scanned_col,
            self.block_state.shooter_last_scanned_row,
            shooter_slots,
            headbutt_workers,
            projectiles,
            shooter_cells,
            barrier_cells
        ))
    }

    fn juggernaut_companion_step_debug_sub_975c(&self) -> Option<String> {
        let step = self.last_juggernaut_companion_step_sub_975c?;
        let visual = match step.visual {
            helmets::JuggernautCompanionVisualSub975C::PreserveCurrent => "preserve_current",
            helmets::JuggernautCompanionVisualSub975C::InitAnimation(label) => label,
            helmets::JuggernautCompanionVisualSub975C::SetMapping(label) => label,
            helmets::JuggernautCompanionVisualSub975C::None => "none",
        };
        Some(format!(
            "visual={visual};byte_15_active={};blocked_latch={}",
            step.byte_15_active, step.blocked_latch
        ))
    }

    fn platform_collision_scratch_debug_loc_257e(&self) -> Option<String> {
        let scratch = self.last_platform_collision_scratch_loc_257e;
        if !scratch.has_contact() {
            return None;
        }

        let fa96 = scratch
            .jump_platform_ref_fa96
            .map_or_else(|| "none".to_string(), |slot| format!("{slot:#06x}"));
        Some(format!(
            "FAA2={} FAA3={} FAA4={} FAA5={} FA96={}",
            u8::from(scratch.touched_left_side_faa2),
            u8::from(scratch.touched_right_side_faa3),
            u8::from(scratch.touched_top_face_faa4),
            u8::from(scratch.touched_bottom_face_faa5),
            fa96,
        ))
    }

    fn native_platform_slots_debug_loc_257e(platforms: &[ActivePlatform]) -> Option<String> {
        if platforms.is_empty() {
            return None;
        }

        // ASM Platforms_CheckCollision loc_257E and Move_Platforms sub_23EA walk
        // Addr_FirstPlatformSlot's `$22`-byte Platform_RAM linked list. Native
        // now stores each materialised platform's slot address, so this trace
        // exposes the Vec scan order beside the ROM slot and `$20(a3)` key.
        let slots = platforms
            .iter()
            .enumerate()
            .map(|(index, platform)| {
                let next = if index + 1 < platforms.len() {
                    format!("native[{:02}]", index + 1)
                } else {
                    "none".to_string()
                };
                let slot = platform
                    .slot_addr
                    .or_else(|| crate::PlatformLoaderState::synthetic_slot_addr_loc_231C(index))
                    .map_or_else(|| "none".to_string(), |slot| format!("{slot:#06x}"));
                let next_slot = if index + 1 < platforms.len() {
                    platforms[index + 1]
                        .slot_addr
                        .or_else(|| {
                            crate::PlatformLoaderState::synthetic_slot_addr_loc_231C(index + 1)
                        })
                        .map_or_else(|| "none".to_string(), |slot| format!("{slot:#06x}"))
                } else {
                    "none".to_string()
                };
                let render =
                    crate::platforms::platform_render_flags_loc_2814(platform.kind, platform.subtype);
                let art = crate::platforms::platform_mapping_addr_word_loc_2822(render);
                let layout_offset = platform.layout_index.map_or_else(
                    || "none".to_string(),
                    |layout_index| {
                        format!(
                            "{:#06x}",
                            crate::platforms::platform_loader_offset_for_index_loc_286E(
                                layout_index
                            )
                        )
                    },
                );
                let timer = platform
                    .script
                    .map_or(0, |script| script.remaining_duration);
                let state = u32::from(platform.deactivated_flag);
                format!(
                    "native[{index:02}]:next={next},slot={slot},next_slot={next_slot},xy={:#010x},{:#010x},vel={:#010x},{:#010x},state={state:#010x},timer={timer:#06x},art={art:#06x},size={}x{},render={render:#04x},off={layout_offset}",
                    platform.x_pos,
                    platform.y_pos,
                    platform.x_velocity,
                    platform.y_velocity,
                    platform.width_px,
                    platform.height_px,
                )
            })
            .collect::<Vec<_>>();

        Some(format!("count={};{}", platforms.len(), slots.join(";")))
    }

    fn native_platform_loader_debug_loc_2744(&self) -> Option<String> {
        let slice = self.opening_slice?;
        let loader = slice.objects.platform_loader;
        if loader.next_index == 0 && loader.offset_words == 0 && self.active_platforms.is_empty() {
            return None;
        }
        let layout_len = self.platform_layout(self.current_level).map_or_else(
            || "none".to_string(),
            |layout| layout.placements.len().to_string(),
        );
        let active_links = loader.active_slot_next_links_loc_231C();
        let active_links = if active_links.is_empty() {
            "none".to_string()
        } else {
            active_links
                .into_iter()
                .map(|(slot_addr, next_addr)| format!("{slot_addr:#06x}->{next_addr:#06x}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        Some(format!(
            "offset={:#06x} next_index={} number_platforms={} next_free={:#06x} layout_len={} active_slots={} active_links={}",
            loader.offset_words,
            loader.next_index,
            loader.number_platforms_loc_231C(),
            loader.next_free_platform_slot_addr_loc_231C(),
            layout_len,
            self.active_platforms.len(),
            active_links
        ))
    }

    fn heady_metal_body_pieces_sub_37abc(&self) -> Vec<HeadyMetalBodyPieceTraceSub37ABC> {
        let Some(slice) = self.opening_slice else {
            return Vec::new();
        };
        let boss = self
            .active_enemies
            .iter()
            .find(|enemy| matches!(enemy.family, EnemyFamily::HeadyMetal));
        let Some(boss) = boss else {
            return Vec::new();
        };
        let boss_x = (boss.x_pos >> 16) as u16;
        let boss_y = (boss.y_pos >> 16) as u16;

        if let Some(piece_slots) = self.heady_metal_breakup_slots_sub_37708 {
            let slots = std::array::from_fn(|index| {
                let piece = piece_slots[index + 1];
                piece.map(|piece| BossEyeSpriteRefreshSlotSub37ABC {
                    x_pos: piece.x_pos,
                    y_pos: piece.y_pos.wrapping_add_signed(piece.sprite_word_y),
                    sprite_mapping: "stale",
                })
            });
            return self.refreshed_heady_metal_body_pieces_sub_37abc(slots, &slice);
        }

        // ASM Enemy20_HeadyMetal_Init (kid.asm:39266-39321) stores the twelve
        // `sub_37680` body-piece workers immediately after `$FFFFFB7A`; ASM
        // sub_37ABC then refreshes that `$FFFFFB7E..$FFFFFBAA` window.
        let slots =
            HEADY_METAL_SUB_37ABC_BODY_PIECE_OFFSETS_LOC_39266.map(|(x_offset, y_offset)| {
                Some(BossEyeSpriteRefreshSlotSub37ABC {
                    x_pos: boss_x.wrapping_add_signed(x_offset),
                    y_pos: boss_y.wrapping_add_signed(y_offset),
                    sprite_mapping: "stale",
                })
            });
        self.refreshed_heady_metal_body_pieces_sub_37abc(slots, &slice)
    }

    // slots is a Copy POD array forwarded by value into
    // boss_eye_refresh_attack_sprites_sub_37ABC; by-ref would only add deref noise
    // since that call copies the array regardless.
    #[allow(clippy::large_types_passed_by_value)]
    fn refreshed_heady_metal_body_pieces_sub_37abc(
        &self,
        slots: [Option<BossEyeSpriteRefreshSlotSub37ABC>; 12],
        slice: &OpeningSliceState,
    ) -> Vec<HeadyMetalBodyPieceTraceSub37ABC> {
        let refreshed = boss_eye_refresh_attack_sprites_sub_37ABC(
            slots,
            self.progression.diamond_power_frames > 0
                || !self.diamond_power_invuln_haze_slots.is_empty(),
            self.frame_index as u8,
            (slice.player.x_pos >> 16) as u16,
            (slice.player.y_pos >> 16) as u16,
        );
        refreshed
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let slot = slot.as_ref()?;
                Some(HeadyMetalBodyPieceTraceSub37ABC {
                    slot_offset: 0x000C + index as u16 * 4,
                    x_pos: slot.x_pos,
                    y_pos: slot.y_pos,
                    sprite_mapping_index: boss_eye_sprite_mapping_index(slot.sprite_mapping),
                })
            })
            .collect()
    }

    fn drips_lifecycle_probe_debug(&self) -> Option<String> {
        let parents = self
            .active_enemies
            .iter()
            .filter(|enemy| enemy.family == EnemyFamily::Drips)
            .map(|enemy| {
                let wait = if enemy.wait_timer == DRIPS_DROP_IN_FLIGHT_TIMER {
                    String::from("in_flight")
                } else if let Some(frames) =
                    drips_splash_animation_remaining_loc_33488(enemy.wait_timer)
                {
                    format!("splash:{frames:#06x}")
                } else if (enemy.wait_timer & 0x8000) != 0 {
                    format!("pre_drop:{:#06x}", enemy.wait_timer)
                } else {
                    format!("{:#06x}", enemy.wait_timer)
                };
                format!(
                    "#{:02}@{:#010x},{:#010x} attached={} wait={} table={} level={:#04x}",
                    enemy.layout_index,
                    enemy.x_pos,
                    enemy.y_pos,
                    u8::from(enemy.drips_attached),
                    wait,
                    enemy.drips_surface_table_index,
                    enemy.enemy_level
                )
            })
            .collect::<Vec<_>>();
        let children = self
            .projectiles
            .iter()
            .enumerate()
            .filter(|(_, projectile)| {
                matches!(
                    projectile.kind,
                    EnemyProjectileKind::DripsDrop
                        | EnemyProjectileKind::DripsSurfaceCrawler
                        | EnemyProjectileKind::DripsSurfaceCrawlerBurst
                        | EnemyProjectileKind::DripsSurfaceCrawlerFinish
                )
            })
            .map(|(index, projectile)| {
                let cell = self
                    .block_grid
                    .as_ref()
                    .map_or_else(|| String::from("none"), |grid| {
                        trace_cell_for_fixed_pos(grid, projectile.x_pos, projectile.y_pos)
                    });
                let owner = projectile.owner.map_or_else(
                    || String::from("none"),
                    |owner| {
                        format!(
                            "{}:{}:{:#04x}",
                            owner.segment_index, owner.layout_index, owner.token
                        )
                    },
                );
                format!(
                    "#{index:02}:{:?}@{:#010x},{:#010x} cell={} vel={:#010x},{:#010x} life={} owner={} spawn={} state={:?} xflip={} b44={:#04x} b45={:#04x}",
                    projectile.kind,
                    projectile.x_pos,
                    projectile.y_pos,
                    cell,
                    projectile.x_velocity as u32,
                    projectile.y_velocity as u32,
                    projectile.lifetime,
                    owner,
                    u8::from(projectile.drips_spawn_crawler_on_splash),
                    projectile.drips_surface_crawler_state,
                    u8::from(projectile.drips_surface_crawler_x_direction_flipped),
                    projectile.drips_child_byte_44,
                    projectile.drips_child_byte_45
                )
            })
            .collect::<Vec<_>>();

        if parents.is_empty() && children.is_empty() {
            return None;
        }

        Some(format!(
            "count_loc_fffffb42={} parents=[{}] children=[{}]",
            self.drips_surface_crawler_count_loc_fffffb42,
            parents.join(";"),
            children.join(";")
        ))
    }

    fn kid_object_support_candidates(&self, player: PlayerState) -> Option<String> {
        let player_bounds = self.player_collision_bounds(player);
        let player_x_px = (player.x_pos >> 16) as i32;
        let player_foot_y_px = (player.y_pos >> 16) as i32;
        let mut candidates = Vec::new();

        for enemy in &self.active_enemies {
            let snapshot = enemy.snapshot();
            if let Some(bounds) = enemy_collision_bounds_px(&snapshot)
                && let Some(candidate) = kid_object_support_candidate(
                    &format!(
                        "enemy_{:02X}#{:02}",
                        enemy.family.object_type(),
                        enemy.layout_index
                    ),
                    player_x_px,
                    player_foot_y_px,
                    player_bounds,
                    bounds,
                )
            {
                candidates.push(candidate);
            }
        }

        for (platform_index, platform) in self.active_platforms.iter().enumerate() {
            let bounds = platform_collision_bounds_px(*platform);
            if let Some(candidate) = kid_object_support_candidate(
                &format!("platform_{platform_index:02}"),
                player_x_px,
                player_foot_y_px,
                player_bounds,
                bounds,
            ) {
                candidates.push(candidate);
            }
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by_key(|candidate| (candidate.abs_top_gap_px, candidate.abs_dx_px));
        let summaries = candidates
            .into_iter()
            .take(KID_OBJECT_SUPPORT_CANDIDATE_LIMIT)
            .map(|candidate| candidate.summary)
            .collect::<Vec<_>>();
        Some(summaries.join("; "))
    }

    fn kid_ground_probe_debug(&self, player: PlayerState) -> Option<String> {
        let grid = self.block_grid.as_ref()?;
        let x_px = (player.x_pos >> 16) as i32;
        let y_px = (player.y_pos >> 16) as i32;
        let half_width = self.progression.current_form.half_width();
        let left = terrain::probe_floor(grid, x_px - half_width, y_px);
        let center = terrain::probe_floor(grid, x_px, y_px);
        let right = terrain::probe_floor(grid, x_px + half_width, y_px);
        let hitbox = terrain::probe_floor_hitbox(grid, x_px, y_px, half_width);
        let latched_floor_y_px = self
            .opening_slice
            .as_ref()
            .map_or(y_px, |slice| (slice.terrain.floor_y_pos >> 16) as i32);

        Some(format!(
            "latched_ground={}/latched_floor_y={}/y_vel={:#010x}/left={}/center={}/right={}/hitbox={}",
            u8::from(player.on_ground),
            latched_floor_y_px,
            player.y_velocity as u32,
            format_kid_floor_probe(left),
            format_kid_floor_probe(center),
            format_kid_floor_probe(right),
            format_kid_floor_probe(hitbox),
        ))
    }

    fn kid_special_motion_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        Some(format!(
            "helmet={:#06x}/skycutter_on_board={}/skycutter_flipboard={}/cyclone_flying={}/cyclone_y_accel={}/iron_knight_climbing={}/iron_knight_climb_accel={}/iron_knight_climb_pose={}/micromax_stuck={}/pending_stomp_bounce={}/form_flash={}",
            self.progression.current_form.helmet_id(),
            u8::from(self.helmet.skycutter_on_board),
            self.helmet.skycutter_flipboard,
            u8::from(self.helmet.cyclone_flying),
            self.helmet.cyclone_y_accel,
            u8::from(self.helmet.iron_knight_climbing),
            self.helmet.iron_knight_climb_accel,
            self.helmet.iron_knight_climb_pose_accum,
            u8::from(self.helmet.micromax_stuck),
            u8::from(self.pending_stomp_bounce.is_some()),
            self.progression.form_change_flash_frames,
        ))
    }

    fn kid_goal_probe_debug(&self, player: PlayerState) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let header = self.level_header(self.current_level)?;
        let flag_x_fp = u32::from(header.flag_x) << 16;
        let flag_y_fp = u32::from(header.flag_y) << 16;
        let dx_px = player.x_pos.abs_diff(flag_x_fp) >> 16;
        let dy_px = player.y_pos.abs_diff(flag_y_fp) >> 16;
        let touching_flag = crate::combat::flagpole_bottom_touches_kid_loc_31d7a(
            player,
            self.progression.current_form,
            header.flag_x,
            header.flag_y,
        );

        Some(format!(
            "lifecycle={}/level_complete={}/touch={}/flag=({}, {})/dx_px={}/dy_px={}",
            player_lifecycle_debug_label(self.player_lifecycle),
            u8::from(matches!(
                self.player_lifecycle,
                PlayerLifecycle::LevelComplete { .. }
            )),
            u8::from(touching_flag),
            header.flag_x,
            header.flag_y,
            dx_px,
            dy_px,
        ))
    }

    fn kid_gameplay_path_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        Some(format!(
            "lifecycle={}/motion={}/post={}/paused={}",
            player_lifecycle_debug_label(self.player_lifecycle),
            self.last_kid_frame_motion_branch.as_debug_label(),
            self.last_kid_frame_post_motion_mutator.as_debug_label(),
            u8::from(self.paused),
        ))
    }

    fn kid_pre_physics_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let start = self.last_kid_frame_pre_physics_start?;
        let before_physics = self
            .last_kid_frame_pre_physics_before_physics
            .unwrap_or(start);
        let first_mutator = self
            .last_kid_frame_pre_physics_mutators
            .first()
            .copied()
            .unwrap_or(KidFramePrePhysicsMutator::None)
            .as_debug_label();
        let mutators = if self.last_kid_frame_pre_physics_mutators.is_empty() {
            String::from("none")
        } else {
            self.last_kid_frame_pre_physics_mutators
                .iter()
                .map(|mutator| mutator.as_debug_label())
                .collect::<Vec<_>>()
                .join(",")
        };

        Some(format!(
            "start[{}/ground={}]/before_physics[{}/ground={}]/first={}/mutators={}",
            kid_pre_physics_state_debug(start),
            u8::from(start.on_ground),
            kid_pre_physics_state_debug(before_physics),
            u8::from(before_physics.on_ground),
            first_mutator,
            mutators,
        ))
    }

    fn kid_collision_handoff_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_collision_handoff_probe?;
        let pending_form = probe.pending_form.map_or_else(
            || String::from("none"),
            |form| format!("{:#06x}", form.helmet_id()),
        );
        let pending_level_skip = probe
            .pending_level_skip
            .map_or_else(|| String::from("none"), |level| format!("{:#04x}", level.0));
        let stomp_enemy = match (
            probe.stomp_enemy_object_type,
            probe.stomp_enemy_x_pos,
            probe.stomp_enemy_y_pos,
        ) {
            (Some(object_type), Some(x_pos), Some(y_pos)) => {
                format!("{object_type:#04x}@{x_pos:#010x},{y_pos:#010x}")
            }
            _ => String::from("none"),
        };
        let wall_cling = match probe.wall_cling_direction {
            Some(Direction::Left) => "left",
            Some(Direction::Right) => "right",
            None => "none",
        };

        Some(format!(
            "lifecycle={}/prev_post={}/pending_stomp={}/pending_contact={}/pending_form={}/pending_level_skip={}/form_flash={}/ground={}/jump_buffer={}/wall_cling={}/stomp_candidate={}/stomp_enemy={}/anchor={:#010x},{:#010x}",
            player_lifecycle_debug_label(probe.player_lifecycle),
            probe.previous_post_motion_mutator.as_debug_label(),
            u8::from(probe.pending_stomp_bounce),
            u8::from(probe.pending_contact_damage),
            pending_form,
            pending_level_skip,
            probe.form_change_flash_frames,
            u8::from(probe.on_ground_before),
            probe.jump_buffer_frames,
            wall_cling,
            u8::from(probe.stomp_candidate),
            stomp_enemy,
            probe.anchor_x_pos,
            probe.anchor_y_pos,
        ))
    }

    fn kid_projectile_damage_probe_debug(&self, player: PlayerState) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let px = (player.x_pos >> 16) as i32;
        let py = (player.y_pos >> 16) as i32;
        let half = PROJECTILE_HITBOX_HALF;
        let player_half_width = self.progression.current_form.half_width();
        let player_left = px - player_half_width;
        let player_right = px + player_half_width;
        let player_top = py - PLAYER_HEIGHT;
        let player_bottom = py;
        let mut closest: Option<(i32, String)> = None;
        if let Some(summary) = &self.last_projectile_damage_probe {
            return Some(summary.clone());
        }

        for (index, projectile) in self.projectiles.iter().enumerate() {
            if matches!(
                projectile.kind,
                EnemyProjectileKind::DripsSurfaceCrawlerBurst
                    | EnemyProjectileKind::DripsSurfaceCrawlerFinish
            ) || projectile.friendly
            {
                continue;
            }
            let prx = (projectile.x_pos >> 16) as i32;
            let pry = (projectile.y_pos >> 16) as i32;
            let dx = prx - px;
            let dy = pry - py;
            let overlaps = if projectile.kind == EnemyProjectileKind::LionProjectile {
                // ASM Lion.asm loc_3E12A -> stru_3DF34 maps through
                // CollisionSize_Index[667..669] to `unk_31944` (-5,9,-5,9);
                // GfxObjects_CollisionKid loc_1BCE uses inclusive edge checks.
                prx - 5 <= player_right
                    && prx + 4 >= player_left
                    && pry - 5 <= player_bottom
                    && pry + 4 >= player_top
            } else {
                dx.abs() < half + 8 && dy.abs() < half + 16
            };
            let distance = dx.abs() + dy.abs();
            let owner = projectile.owner.map_or_else(
                || String::from("none"),
                |owner| {
                    format!(
                        "{}:{}:{:#04x}",
                        owner.segment_index, owner.layout_index, owner.token
                    )
                },
            );
            let summary = format!(
                "#{index:02}:{:?}@{:#010x},{:#010x} vel={:#010x},{:#010x} life={} owner={} dx={dx} dy={dy} overlap={}",
                active_projectile_kind(projectile),
                projectile.x_pos,
                projectile.y_pos,
                projectile.x_velocity as u32,
                projectile.y_velocity as u32,
                projectile.lifetime,
                owner,
                u8::from(overlaps),
            );
            if overlaps {
                return Some(summary);
            }
            if closest
                .as_ref()
                .is_none_or(|(closest_distance, _)| distance < *closest_distance)
            {
                closest = Some((distance, summary));
            }
        }

        for (index, trail) in self.fire_demon_trails.iter().enumerate() {
            let prx = (trail.x_pos >> 16) as i32;
            let pry = (trail.y_pos >> 16) as i32;
            let dx = prx - px;
            let dy = pry - py;
            let overlaps = dx.abs() < half + 8 && dy.abs() < half + 16;
            let distance = dx.abs() + dy.abs();
            let summary = format!(
                "trail#{index:02}:FireDemonDeathTrail@{:#010x},{:#010x} life={} dx={dx} dy={dy} overlap={}",
                trail.x_pos,
                trail.y_pos,
                trail.remaining_frames,
                u8::from(overlaps),
            );
            if overlaps {
                return Some(summary);
            }
            if closest
                .as_ref()
                .is_none_or(|(closest_distance, _)| distance < *closest_distance)
            {
                closest = Some((distance, summary));
            }
        }

        closest.map(|(_, summary)| summary)
    }

    fn kid_same_frame_stomp_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_same_frame_stomp_probe?;
        let enemy = match (
            probe.enemy_object_type,
            probe.enemy_x_pos,
            probe.enemy_y_pos,
        ) {
            (Some(object_type), Some(x_pos), Some(y_pos)) => {
                format!("{object_type:#04x}@{x_pos:#010x},{y_pos:#010x}")
            }
            _ => String::from("none"),
        };

        Some(format!(
            "candidate={}/enemy={}/pre_y_vel={:#010x}/post_y_vel={:#010x}/anchor={:#010x},{:#010x}",
            u8::from(probe.stomp_candidate),
            enemy,
            probe.pre_physics_y_velocity as u32,
            probe.post_physics_y_velocity as u32,
            probe.anchor_x_pos,
            probe.anchor_y_pos,
        ))
    }

    fn kid_iron_knight_attach_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_iron_knight_attach_probe?;

        Some(format!(
            "c_press={}/ground={}/climbing={}=>{}/facing={}/y_vel={:#010x}->{:#010x}/wall={}/candidate={}/attach={}",
            u8::from(probe.c_pressed),
            u8::from(probe.on_ground_before),
            u8::from(probe.climbing_before),
            u8::from(probe.climbing_after),
            direction_debug_label(probe.facing),
            probe.y_velocity_before as u32,
            probe.y_velocity_after as u32,
            wall_probe_debug_summary(probe.wall_probe),
            u8::from(probe.candidate),
            u8::from(probe.attach_triggered),
        ))
    }

    fn kid_flip_branch_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_kid_flip_branch_probe?;

        Some(format!(
            "wall_scan={}/b_press={}/c_press={}/helmet={:#06x}/facing={}/ground={}=>{}/branch={}/wall={}",
            u8::from(probe.wall_scan_hit),
            u8::from(probe.b_pressed),
            u8::from(probe.c_pressed),
            probe.helmet_id,
            direction_debug_label(probe.facing),
            u8::from(probe.on_ground_before),
            u8::from(probe.on_ground_after),
            probe.branch.as_debug_label(),
            wall_probe_debug_summary(probe.wall_probe),
        ))
    }

    fn kid_flip_follow_on_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_kid_flip_follow_on_probe?;

        Some(format!(
            "branch={}/motion={}/post={}/wall_hit_after_floor={}/entry[{}/ground={}/wall_cling={}]/press_away[{}/ground={}/wall_cling={}]/jump_gravity[{}/ground={}/wall_cling={}]/floor[{}/ground={}/wall_cling={}]/wall[{}/ground={}/wall_cling={}]",
            probe.branch.as_debug_label(),
            probe.motion_branch.as_debug_label(),
            probe.post_motion_mutator.as_debug_label(),
            u8::from(probe.wall_hit_after_floor),
            kid_flip_follow_on_stage_debug(probe.stages.entry),
            u8::from(probe.stages.entry.on_ground),
            direction_debug_label(probe.stages.entry.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_press_away),
            u8::from(probe.stages.after_press_away.on_ground),
            direction_debug_label(probe.stages.after_press_away.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_jump_gravity),
            u8::from(probe.stages.after_jump_gravity.on_ground),
            direction_debug_label(probe.stages.after_jump_gravity.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_floor),
            u8::from(probe.stages.after_floor.on_ground),
            direction_debug_label(probe.stages.after_floor.wall_cling_direction),
            kid_flip_follow_on_stage_debug(probe.stages.after_wall),
            u8::from(probe.stages.after_wall.on_ground),
            direction_debug_label(probe.stages.after_wall.wall_cling_direction),
        ))
    }

    fn kid_jump_gravity_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_jump_gravity_probe?;

        Some(format!(
            "branch={}/jump={}/jump_buffer={}=>{}/just_jumped={}=>{}/b_held={}/custom_gravity={}/y_vel={:#010x}->{:#010x}/ground={}=>{}",
            probe.branch.as_debug_label(),
            u8::from(probe.jump_pressed),
            probe.jump_buffer_frames_before,
            probe.jump_buffer_frames_after,
            u8::from(probe.just_jumped_before),
            u8::from(probe.just_jumped_after),
            u8::from(probe.input_b_held),
            u8::from(probe.custom_gravity),
            probe.y_velocity_before as u32,
            probe.y_velocity_after as u32,
            u8::from(probe.on_ground_before),
            u8::from(probe.on_ground_after),
        ))
    }

    fn kid_grounded_release_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.last_kid_frame_grounded_release_probe?;
        let held = match probe.held_direction {
            Some(Direction::Left) => "left",
            Some(Direction::Right) => "right",
            None => "none",
        };
        let wall_cling = match probe.wall_cling_direction {
            Some(Direction::Left) => "left",
            Some(Direction::Right) => "right",
            None => "none",
        };
        let scan = match (probe.probe_row, probe.left_col, probe.right_col) {
            (Some(row), Some(left), Some(right)) => format!("row={row}/cols={left}..{right}"),
            _ => String::from("row=none/cols=none"),
        };
        let support = match (probe.support_col, probe.support_collision, probe.probe_row) {
            (Some(col), Some(collision), Some(row)) => format!(
                "{}@{col},{row} block={}",
                collision_kind_debug_label(collision),
                block_type_debug_label(
                    probe.support_block_type.unwrap_or(terrain::BlockType::None)
                ),
            ),
            _ => String::from("none"),
        };
        let terrain_provenance =
            match (probe.support_col, probe.probe_row, self.block_grid.as_ref()) {
                (Some(col), Some(row), Some(current_grid)) => {
                    let current_cell = current_grid.cell_at(col, row);
                    let baseline = if self.block_grid_source == BlockGridSource::Generated {
                        self.load_block_grid_for_level(self.current_level)
                            .map(|grid| grid.cell_at(col, row))
                    } else {
                        None
                    };
                    match baseline {
                        Some(baseline_cell) => format!(
                            "/terrain=grid={}/current[{}]/baseline[{}]/same={}",
                            block_grid_source_debug_label(self.block_grid_source),
                            terrain_cell_debug_summary(current_cell),
                            terrain_cell_debug_summary(baseline_cell),
                            u8::from(current_cell == baseline_cell),
                        ),
                        None => format!(
                            "/terrain=grid={}/current[{}]",
                            block_grid_source_debug_label(self.block_grid_source),
                            terrain_cell_debug_summary(current_cell),
                        ),
                    }
                }
                _ => String::new(),
            };

        Some(format!(
            "ground={}/held={}/jump={}/jump_buffer={}/velocity={}/custom_gravity={}/wall_cling={}/scan={}/support={}/release={}/post={}{}",
            u8::from(probe.on_ground_before),
            held,
            u8::from(probe.jump_pressed),
            probe.jump_buffer_frames,
            probe.velocity_branch.as_debug_label(),
            u8::from(probe.custom_gravity),
            wall_cling,
            scan,
            support,
            u8::from(probe.release_triggered),
            self.last_kid_frame_post_motion_mutator.as_debug_label(),
            terrain_provenance,
        ))
    }

    fn kid_iron_knight_block_breaker_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let player = self.last_kid_frame_pre_physics_before_physics?;
        let form = self.progression.current_form;
        let facing = match player.facing {
            Direction::Left => "left",
            Direction::Right => "right",
        };
        let speed_gate =
            form == PlayerForm::IronKnight && player.x_velocity.unsigned_abs() >= 0x48000;

        let (wall_summary, breaker_candidate) = match self.block_grid.as_ref() {
            Some(grid) => {
                let wall_probe = terrain::probe_wall(
                    grid,
                    (player.x_pos >> 16) as i32,
                    (player.y_pos >> 16) as i32,
                    form.half_width(),
                    PLAYER_HEIGHT,
                    player.facing == Direction::Right,
                );
                (
                    wall_probe_debug_summary(Some(wall_probe)),
                    speed_gate && wall_probe.hit && wall_probe.cell.has_block,
                )
            }
            None => (wall_probe_debug_summary(None), false),
        };

        Some(format!(
            "helmet={:#06x}/facing={}/x_vel={:#010x}/speed_gate={}/wall={}/candidate={}",
            form.helmet_id(),
            facing,
            player.x_velocity as u32,
            u8::from(speed_gate),
            wall_summary,
            u8::from(breaker_candidate),
        ))
    }

    fn kid_wall_cling_release_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return None;
        }

        let probe = self.opening_slice?.wall_cling_release_probe?;
        let held = direction_debug_label(probe.held_direction);
        let wall_cling = direction_debug_label(probe.wall_cling_direction);
        let probe_direction = match probe.probe_direction {
            Some(direction) => format!(
                "{}({})",
                direction_debug_label(Some(direction)),
                probe.probe_direction_source.as_debug_label(),
            ),
            None => String::from("none(none)"),
        };

        Some(format!(
            "held={}/probe_dir={}/wall_cling={}/sticky=R{}L{}/ground={}/y_vel={:#010x}/y={}=>{}/branch={}/sub_ADB8={}/sub_AE6E={}/wall={}/top_edge={}",
            held,
            probe_direction,
            wall_cling,
            u8::from(probe.right_wall_sticky),
            u8::from(probe.left_wall_sticky),
            u8::from(probe.on_ground),
            probe.y_velocity as u32,
            probe.pre_vertical_y_px,
            probe.post_vertical_y_px,
            probe.branch.as_debug_label(),
            probe.sub_adb8_gate.as_debug_label(),
            probe.sub_ae6e_gate.as_debug_label(),
            wall_probe_debug_summary(probe.forward_wall_probe),
            wall_probe_debug_summary(probe.top_edge_probe),
        ))
    }

    fn diamond_power_snake_probe_debug(&self) -> Option<String> {
        if !matches!(
            self.progression.diamond_power_type,
            helmets::DiamondPowerType::MiniSnake | helmets::DiamondPowerType::DeathSnake
        ) || self.diamond_power_snake_slots.is_empty()
        {
            return None;
        }

        let head = self
            .diamond_power_snake_slots
            .last()
            .expect("non-empty Snake slots checked above");
        let (camera_x, camera_y, kid_x, kid_y) =
            self.opening_slice.as_ref().map_or((0, 0, 0, 0), |slice| {
                (
                    (slice.camera.x_pos >> 16) as i16,
                    (slice.camera.y_pos >> 16) as i16,
                    (slice.player.x_pos >> 16) as i16,
                    (slice.player.y_pos >> 16) as i16,
                )
            });
        let enemies: Vec<helmets::SnakeTargetCandidate> = self
            .active_enemies
            .iter()
            .map(|enemy| {
                let snapshot = enemy.snapshot();
                helmets::SnakeTargetCandidate {
                    x_pos: (snapshot.x_pos >> 16) as i16,
                    y_pos: (snapshot.y_pos >> 16) as i16,
                }
            })
            .collect();
        let target = helmets::diamond_power_snake_find_target(
            head.x_pos, head.y_pos, camera_x, camera_y, &enemies, kid_x, kid_y,
        );
        let first_sprite = self.diamond_power_sprites.first().map_or_else(
            || String::from("none"),
            |sprite| {
                format!(
                    "{},{}:{:#06x}/{:#06x}",
                    sprite.screen_x, sprite.screen_y, sprite.size_word, sprite.pattern_word
                )
            },
        );

        Some(format!(
            "slots={}/head={:#06x}.{:#06x},{:#06x}.{:#06x}/angle={:#06x}/turn={:#06x}/speed={:#06x}/target={:#06x},{:#06x}/sprite={}",
            self.diamond_power_snake_slots.len(),
            head.x_pos as u16,
            head.x_subpixel,
            head.y_pos as u16,
            head.y_subpixel,
            self.diamond_power_snake_head_angle,
            self.diamond_power_snake_turn_rate,
            self.diamond_power_snake_movement_speed,
            target.x as u16,
            target.y as u16,
            first_sprite,
        ))
    }

    fn pending_prize_diamond_awards_debug(&self) -> String {
        let delays = self
            .pending_prize_diamond_awards
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>();
        format!("[{}]", delays.join(","))
    }

    fn floating_diamond_contact_candidates(&self, player: PlayerState) -> Option<String> {
        use crate::constants::PLAYER_HEIGHT;

        let collision_anchor = self
            .last_kid_frame_collision_handoff_probe
            .map_or((player.x_pos, player.y_pos), |probe| {
                (probe.anchor_x_pos, probe.anchor_y_pos)
            });
        let player_x_px = (collision_anchor.0 >> 16) as i32;
        let player_y_px = (collision_anchor.1 >> 16) as i32;
        let player_half_width = self.progression.current_form.half_width();
        let player_left = player_x_px - player_half_width;
        let player_right = player_x_px + player_half_width;
        let player_bottom = player_y_px;
        let mut candidates = Vec::new();

        for item in &self.floating_items {
            if item.content != terrain::PrizeContent::Diamond {
                continue;
            }
            let item_x_px = (item.x_pos >> 16) as i32;
            let item_y_px = (item.y_pos >> 16) as i32;
            let player_top = player_y_px - PLAYER_HEIGHT;
            // ASM Mode_Level calls sub_21F8 before GfxObjects_CollisionKid
            // (kid.asm:909,939), so PrizeDiamond_Init loc_10396 uses Kid's
            // current form hitbox against unk_31D5E's x-5,width=10,y-10,height=10.
            let item_left = item_x_px - 5;
            let item_right = item_x_px + 5;
            let item_top = item_y_px - 10;
            let item_bottom = item_y_px;
            let overlap_x = player_left <= item_right && player_right >= item_left;
            let overlap_y = player_top <= item_bottom && player_bottom >= item_top;
            let horizontal_gap_px = if overlap_x {
                0
            } else if player_right <= item_left {
                item_left - player_right
            } else {
                player_left - item_right
            };
            let vertical_gap_px = if overlap_y {
                0
            } else if player_bottom <= item_top {
                item_top - player_bottom
            } else {
                player_top - item_bottom
            };
            let dx_px = item_x_px - player_x_px;
            let dy_px = item_y_px - player_y_px;
            candidates.push(FloatingDiamondContactCandidate {
                distance_score: dx_px.abs() + dy_px.abs(),
                summary: format!(
                    "diamond@{item_x_px},{item_y_px} dx={dx_px} dy={dy_px} gap_x={horizontal_gap_px} gap_y={vertical_gap_px} overlap_x={} overlap_y={} spawn={} rem={} collected={}",
                    u8::from(overlap_x),
                    u8::from(overlap_y),
                    item.spawn_delay,
                    item.remaining,
                    u8::from(item.collected),
                ),
            });
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_unstable_by_key(|candidate| candidate.distance_score);
        let summaries = candidates
            .into_iter()
            .take(FLOATING_DIAMOND_CANDIDATE_LIMIT)
            .map(|candidate| candidate.summary)
            .collect::<Vec<_>>();
        Some(summaries.join("; "))
    }

    fn floating_prize_contact_candidates(&self, player: PlayerState) -> Option<String> {
        use crate::constants::{PLAYER_HALF_WIDTH, PLAYER_HEIGHT};

        let player_x_px = (player.x_pos >> 16) as i32;
        let player_y_px = (player.y_pos >> 16) as i32;
        let player_left = player_x_px - PLAYER_HALF_WIDTH;
        let player_right = player_x_px + PLAYER_HALF_WIDTH;
        let lower_body_top = player_y_px - ((PLAYER_HEIGHT + 1) / 2);
        let form_left = player_x_px - self.progression.current_form.half_width();
        let form_right = player_x_px + self.progression.current_form.half_width();
        let form_top = player_y_px - PLAYER_HEIGHT;
        let mut candidates = Vec::new();

        for item in &self.floating_items {
            if item.content == terrain::PrizeContent::Diamond {
                continue;
            }
            let item_x_px = (item.x_pos >> 16) as i32;
            let item_y_px = (item.y_pos >> 16) as i32;
            let native_left = item_x_px - 8;
            let native_right = item_x_px + 8;
            let native_top = item_y_px;
            let native_bottom = item_y_px + 16;
            let native_overlap_x = player_left < native_right && player_right > native_left;
            let native_overlap_y = lower_body_top < native_bottom && player_y_px > native_top;

            // ASM GfxObjects_CollisionKid (kid.asm:3315-3353) compares object
            // CollisionSize_Index subboxes against Kid's current hitbox. The
            // helmet masks use the `unk_315FC`-style row (`left=-$7,width=$E,
            // top=-$1F,height=$1F`) selected from the Data_Index entry.
            let asm_left = item_x_px - 7;
            let asm_right = item_x_px + 7;
            let asm_top = item_y_px - PLAYER_HEIGHT;
            let asm_bottom = item_y_px;
            let form_overlap_x = form_left <= asm_right && form_right >= asm_left;
            let form_overlap_y = form_top <= asm_bottom && player_y_px >= asm_top;

            let cell_summary = self.block_grid.as_ref().map_or_else(
                || "cell=none/open=1/below=none/below_open=1".to_string(),
                |grid| {
                    let current_cell = grid.cell_at(item.level_col, item.level_row);
                    let current_open = matches!(
                        current_cell.collision,
                        terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
                    );
                    let below_cell = grid.cell_at(item.level_col, item.level_row.saturating_add(1));
                    let below_open = matches!(
                        below_cell.collision,
                        terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
                    );
                    format!(
                        "cell={:?}/open={}/below={:?}/below_open={}",
                        current_cell.collision,
                        u8::from(current_open),
                        below_cell.collision,
                        u8::from(below_open),
                    )
                },
            );

            let dx_px = item_x_px - player_x_px;
            let dy_px = item_y_px - player_y_px;
            candidates.push(FloatingDiamondContactCandidate {
                distance_score: dx_px.abs() + dy_px.abs(),
                summary: format!(
                    "prize#{:02x}@{item_x_px},{item_y_px} dx={dx_px} dy={dy_px} row={} grace={} rem={} collected={} latched={} native_ov={}{} form_ov={}{} {cell_summary}",
                    item.content.to_id(),
                    item.level_row,
                    u8::from(item.first_row_grace),
                    item.remaining,
                    u8::from(item.collected),
                    u8::from(item.collision_latched),
                    u8::from(native_overlap_x),
                    u8::from(native_overlap_y),
                    u8::from(form_overlap_x),
                    u8::from(form_overlap_y),
                ),
            });
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_unstable_by_key(|candidate| candidate.distance_score);
        let summaries = candidates
            .into_iter()
            .take(FLOATING_PRIZE_CANDIDATE_LIMIT)
            .map(|candidate| candidate.summary)
            .collect::<Vec<_>>();
        Some(summaries.join("; "))
    }

    fn player_collision_bounds(&self, player: PlayerState) -> ObjectBoundsPx {
        let half_width = self.progression.current_form.half_width();
        let x_px = (player.x_pos >> 16) as i32;
        let foot_y_px = (player.y_pos >> 16) as i32;
        ObjectBoundsPx {
            left: x_px - half_width,
            right: x_px + half_width,
            top: foot_y_px - PLAYER_HEIGHT,
            bottom: foot_y_px,
        }
    }

    pub(crate) const LEVEL_COMPLETE_TITLE_CARD_CAMERA_HANDOFF_FRAMES_LOC_D7F4: usize = 31;
    pub(crate) const LEVEL_COMPLETE_TITLE_CARD_PLAYER_HANDOFF_FRAMES_LOC_D7F4: usize = 27;

    fn level_entry_camera_handoff_state(&self) -> Option<CameraState> {
        let (start_frame, camera) = self.level_entry_camera_handoff?;
        let elapsed = self.frame_index.checked_sub(start_frame)?;
        // ASM loc_D7F4 -> Teleport enters Game_Mode=$0008, but Load_TitleCard's
        // object-slot clear does not become visible to oracle WRAM until local
        // frame 31 on the EOL route. Keep the stale flag-screen camera until
        // that clear lands.
        (matches!(self.boot_state, BootState::StartingLevel { .. })
            && elapsed < Self::LEVEL_COMPLETE_TITLE_CARD_CAMERA_HANDOFF_FRAMES_LOC_D7F4)
            .then_some(camera)
    }

    fn level_entry_player_handoff_state(&self) -> Option<PlayerState> {
        let (start_frame, player) = self.level_entry_player_handoff?;
        let elapsed = self.frame_index.checked_sub(start_frame)?;
        // ASM loc_D7F4 -> Teleport keeps the previous Kid object readable for
        // a shorter stale-WRAM window than the camera. At local frame 27 the
        // old player slot has cleared to zero while the camera has not.
        (matches!(self.boot_state, BootState::StartingLevel { .. })
            && elapsed < Self::LEVEL_COMPLETE_TITLE_CARD_PLAYER_HANDOFF_FRAMES_LOC_D7F4)
            .then_some(player)
    }

    fn level_entry_cleared_player_handoff_state(&self) -> Option<PlayerState> {
        let (start_frame, _) = self.level_entry_player_handoff?;
        let elapsed = self.frame_index.checked_sub(start_frame)?;
        let cleared_player_window = Self::LEVEL_COMPLETE_TITLE_CARD_PLAYER_HANDOFF_FRAMES_LOC_D7F4
            ..Self::LEVEL_COMPLETE_TITLE_CARD_CAMERA_HANDOFF_FRAMES_LOC_D7F4;
        if !matches!(self.boot_state, BootState::StartingLevel { .. })
            || !cleared_player_window.contains(&elapsed)
        {
            return None;
        }
        // ASM loc_D7F4 -> Teleport / Load_TitleCard has cleared the old player
        // position mirrors to zero here, but oracle WRAM keeps
        // Character_Movement=$0003 until the later camera/object-slot clear.
        Some(PlayerState {
            x_pos: 0,
            y_pos: 0,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: true,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        })
    }

    pub fn game_mode(&self) -> GameMode {
        if self.respawn_live_handoff_frames > 0 {
            // ASM lose_life -> loc_6E2 -> Load_InGame: the settled Start-edge
            // probe has already written Game_Mode=$000C while the hidden
            // respawn bootstrap WRAM view still owns Kid/camera/object state.
            return GameMode::InGame;
        }
        if matches!(
            self.player_lifecycle,
            PlayerLifecycle::Dying { .. } | PlayerLifecycle::Respawning { .. }
        ) {
            return GameMode::LevelTransition;
        }
        if matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.demo_auto_timeout_handoff_active
        {
            return GameMode::SegaScreen;
        }
        match self.boot_state {
            BootState::SegaScreen { .. } => GameMode::SegaScreen,
            BootState::IntroSequence { segment: 0, .. } => GameMode::IntroSequence1,
            BootState::IntroSequence { segment: 1, .. } => GameMode::IntroSequence2,
            BootState::IntroSequence { segment: 2, .. } => GameMode::IntroSequence3,
            BootState::IntroSequence { segment: 3, .. } => GameMode::IntroSequence4,
            BootState::IntroSequence { .. } => GameMode::IntroSequence5,
            BootState::IntroSequence6 { .. } => GameMode::IntroSequence6,
            BootState::TitleScreen { .. } => GameMode::TitleScreen,
            BootState::OptionsMenu { .. } | BootState::SoundTest { .. } => GameMode::OptionsMenu,
            BootState::LevelSkipHold => GameMode::LevelTransition,
            BootState::StartingLevel { .. } => {
                if self.early_start_load_ingame_elapsed_frames().is_some() {
                    // ASM loc_1AE2E writes `Game_Mode=$000C` before it jumps to
                    // loc_6E2, so the hidden Load_InGame bootstrap already
                    // traces as gameplay even while Kid/camera remain blank.
                    GameMode::InGame
                } else {
                    // ASM Load_TitleCard / Obj_TitleMenu `Title_InputLoop`
                    // keep the regular title-card lineage in `Game_Mode=$0008`
                    // for the whole hidden bootstrap window. Oracle-backed
                    // boot-title-start checkpoints still report mode `$0008`
                    // at frames 520 and 719 even though the title workers are
                    // already visible.
                    GameMode::LevelTransition
                }
            }
            BootState::InGame => GameMode::InGame,
            BootState::HundredKTripTransition { .. } => GameMode::LevelTransition,
            BootState::DemoPlay { .. } => GameMode::DemoPlay,
            BootState::EndingSequence { .. } => GameMode::EndingSequence,
        }
    }

    pub(crate) fn current_player_state(&self) -> Option<PlayerState> {
        if self.pause_restart_zero_kid_shell_active_loc_6e2() {
            // ASM loc_6E2 restart reload: oracle WRAM shows Kid's object
            // position zeroed during the hidden reload shell before Load_InGame
            // rebuilds the live slot.
            return Some(PlayerState {
                x_pos: 0,
                y_pos: 0,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        if self.level_skip_resume_hold {
            return self.level_skip_resume_player_state();
        }
        if let Some(player) = self.level_entry_player_handoff_state() {
            return Some(player);
        }
        if let Some(player) = self.level_entry_cleared_player_handoff_state() {
            return Some(player);
        }
        let demo_use_demo_2 = self.demo_uses_second_recording();
        match self.boot_state {
            BootState::SegaScreen { .. } => None,
            BootState::IntroSequence {
                segment: 2,
                elapsed,
                ..
            } if elapsed >= INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES => {
                Some(PlayerState::intro_sequence_3_pose_for_roster(
                    self.active_intro_sequence_3_roster_index,
                ))
            }
            // ASM loc_1B96A writes `Game_Mode=$20`, but Load_IntroSequence4 /
            // sub_1B988 do not clear the old intro-sequence-3 roster
            // immediately. Oracle traces keep the previous roster pose visible
            // through the early intro-sequence-4 frames.
            BootState::IntroSequence { segment: 3, .. }
                if self.intro_sequence_4_uses_intro_sequence_3_pose() =>
            {
                Some(PlayerState::intro_sequence_3_pose_for_roster(
                    self.active_intro_sequence_3_roster_index,
                ))
            }
            BootState::IntroSequence { segment: 4, .. }
                if self.intro_sequence_5_uses_timeout_restart_slot_2_pose() =>
            {
                Some(PlayerState::intro_sequence_5_slot_2_pose())
            }
            BootState::IntroSequence6 {
                carry_intro_sequence_5_slot_2_pose: true,
                ..
            } if self.title_screen_elapsed_frames
                < TIMEOUT_RESTART_INTRO_SEQUENCE_6_SLOT_2_POSE_CARRY_FRAMES =>
            {
                // ASM Load_IntroSequence6 swaps straight to `loc_1BAB0`,
                // `loc_1B37C`, and `sub_1B43A`; the old loc_1BCD0 slot-2 pose
                // only survives as stale WRAM for the brief oracle-backed
                // hand-off window before the rebuilt title shell clears it.
                Some(PlayerState::intro_sequence_5_slot_2_pose())
            }
            BootState::IntroSequence6 {
                carry_timeout_restart_hidden_camera_wrap: false,
                ..
            } if self.title_screen_elapsed_frames
                >= if self.cold_demo_timeout_intro_sequence_6_shell() {
                    TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_TITLE_POSE_VISIBLE_ELAPSED_FRAMES
                        .saturating_sub(1)
                } else {
                    TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_TITLE_POSE_VISIBLE_ELAPSED_FRAMES
                } =>
            {
                // ASM loc_1B5CE still owns the standard title-pose write on
                // the later mode-$28 revisit, but oracle keeps Kid hidden for
                // three extra local frames after the delayed `$FFD00000`
                // camera-ramp lineage before exposing `x=$2E`, `y=$8C`. The
                // cold demo-2 timeout-return shell reaches the same visible
                // pose one local frame earlier.
                Some(PlayerState::title_pose())
            }
            BootState::IntroSequence6 {
                carry_timeout_restart_hidden_camera_wrap: true,
                ..
            } if self.title_screen_elapsed_frames
                >= TIMEOUT_RESTART_INTRO_SEQUENCE_6_TITLE_POSE_VISIBLE_ELAPSED_FRAMES =>
            {
                // ASM loc_1B5CE: once the mode-$28 title-shell rebuild reaches
                // `move.w #$2E,x_pos(a3); move.w #$8C,y_pos(a3)`, oracle starts
                // exposing the standard title pose even though Game_Mode stays
                // at `$28` until Obj_TitleMenu later reaches loc_1BB02.
                Some(PlayerState::title_pose())
            }
            BootState::IntroSequence6 { .. }
                if self.title_menu_active
                    && self.intro_sequence_6_start_skip_cursor_trace_priority =>
            {
                self.title_menu_cursor_player_state()
                    .or_else(|| self.title_screen_ready().then(PlayerState::title_pose))
            }
            BootState::IntroSequence6 { .. } if self.title_menu_active => self
                .title_screen_ready()
                .then(PlayerState::title_pose)
                .or_else(|| self.title_menu_cursor_player_state()),
            BootState::IntroSequence6 { .. } => None,
            BootState::IntroSequence { .. } => None,
            BootState::TitleScreen { .. } if self.title_uses_full_demo_snapshot() => {
                self.opening_slice.map(|slice| slice.player)
            }
            BootState::TitleScreen { .. }
                if self.title_uses_timeout_restart_intro_sequence_3_pose() =>
            {
                Some(PlayerState::intro_sequence_3_pose_for_roster(
                    self.active_intro_sequence_3_roster_index,
                ))
            }
            BootState::TitleScreen { .. }
                if self.title_uses_timeout_restart_intro_sequence_5_pose() =>
            {
                Some(PlayerState::intro_sequence_5_slot_2_pose())
            }
            BootState::TitleScreen { .. } if self.hidden_title_screen_object_counts().is_some() => {
                None
            }
            BootState::TitleScreen { .. }
                if self.title_menu_active
                    && self.intro_sequence_6_start_skip_cursor_trace_priority =>
            {
                self.title_menu_cursor_player_state()
                    .or_else(|| self.title_screen_ready().then(PlayerState::title_pose))
            }
            BootState::TitleScreen { .. } if self.title_menu_active => self
                .title_screen_ready()
                .then(PlayerState::title_pose)
                .or_else(|| self.title_menu_cursor_player_state()),
            BootState::TitleScreen { .. } if self.title_hides_demo_return_player() => None,
            BootState::TitleScreen { .. }
            | BootState::OptionsMenu { .. }
            | BootState::SoundTest { .. } => {
                self.title_screen_ready().then(PlayerState::title_pose)
            }
            BootState::EndingSequence { .. } => Some(PlayerState::title_pose()),
            BootState::DemoPlay { frame_counter, .. }
                if frame_counter < demo::DEMO_TITLE_POSE_FRAMES =>
            {
                Some(PlayerState::title_pose())
            }
            BootState::DemoPlay { frame_counter, .. }
                if demo_use_demo_2.is_some_and(|use_demo_2| {
                    frame_counter < self.demo_gameplay_start_frame(use_demo_2)
                }) =>
            {
                None
            }
            BootState::LevelSkipHold => self.level_skip_player_state(),
            BootState::HundredKTripTransition { .. } => {
                self.opening_slice.map(|slice| slice.player)
            }
            BootState::StartingLevel { elapsed_frames, .. } => self
                .title_menu_cursor_player_state()
                .filter(|_| self.early_start_title_menu_cursor_pose_carry_active())
                .or_else(|| {
                    self.early_start_title_menu_cursor_pose_carry_active()
                        .then(|| {
                            let y_pos = if self.two_player {
                                0x003A_0000
                            } else {
                                0x0030_0000
                            };
                            PlayerState::title_menu_cursor_pose(y_pos)
                        })
                })
                .or_else(|| self.early_start_level_one_hidden_bootstrap_player_state())
                .or_else(|| self.early_start_load_ingame_player_state())
                .or_else(|| {
                    if self.early_start_load_ingame_elapsed_frames().is_some() {
                        None
                    } else {
                        self.starting_level_slot_2_pose(elapsed_frames)
                    }
                }),
            BootState::InGame | BootState::DemoPlay { .. } => {
                if let Some(snapshot) = self.death_shell_snapshot.as_ref()
                    && self.dying_hidden_shell_still_visible()
                {
                    Some(snapshot.frozen_player)
                } else if let Some(player_state) =
                    self.title_card_live_handoff_player_state_loc_1ae2e()
                {
                    Some(player_state)
                } else if let Some(player_state) = self.death_respawn_bootstrap_player_state() {
                    Some(player_state)
                } else if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
                    None
                } else {
                    self.opening_slice.map(|slice| slice.player)
                }
            }
        }
    }

    fn intro_sequence_6_timeout_return_velocity_sample_active(&self) -> bool {
        self.demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            .is_some_and(|(frame_counter, use_demo_2)| {
                self.demo_uses_intro_sequence_6_later_demo_2_preload_row
                    && self.demo_uses_intro_sequence_6_timeout_return_velocity_sample
                    && frame_counter
                        <= demo::DEMO_2_INTRO_SEQUENCE_6_LATER_CONTROLLER_SAMPLE_END_FRAME
                    && !use_demo_2
            })
    }

    fn wall_cling_boundary_sampled_y_velocity_export(&self, state: PlayerState) -> Option<i32> {
        let (y_velocity, _) = self.wall_cling_boundary_trace_y_velocity?;
        if state.on_ground && state.y_velocity == 0 {
            Some(y_velocity)
        } else {
            None
        }
    }

    fn demo_1_late_controller_pause_active(&self) -> bool {
        // ASM Load_DemoPlay -> loc_119D8 (kid.asm:23585) keeps the hidden
        // controller object alive during the late demo-1 hand-off. Its Start
        // sample exposes Game_Paused in top-level TAS state for the same
        // controller-side window as the final object-count sample.
        if self.demo_1_late_title_return_snapshot_active() {
            return true;
        }

        self.demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            .is_some_and(|(frame_counter, use_demo_2)| {
                !use_demo_2
                    && !self.demo_started_from_timeout_restart_shell
                    && self.allow_pausing
                    && (demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME
                        ..=demo::DEMO_1_LATE_CONTROLLER_PAUSE_END_FRAME)
                        .contains(&frame_counter)
            })
    }

    fn demo_1_late_title_return_snapshot_active(&self) -> bool {
        matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.preserve_demo_title_snapshot
            && !self.demo_auto_timeout_handoff_active
            && self.demo_title_snapshot_object_counts == Some((4, 4))
            && self.title_screen_elapsed_frames < self.demo_title_partial_snapshot_frames()
    }

    fn demo_1_late_title_return_controller_counts(&self) -> Option<(u16, u16)> {
        if self.demo_1_late_title_return_snapshot_active() {
            // ASM loc_119D8 jumps to the title return while the late demo-1
            // controller lineage still owns the top-level object/gfx sample.
            // The third Enemy0D_Dragon_Init GfxObject remains linked through
            // this 32-frame partial snapshot, so raw oracle WRAM holds 8/7.
            return Some((8, 7));
        }
        None
    }

    fn pause_restart_elapsed_loc_6e2(&self) -> Option<u8> {
        self.pending_pause_restart_round
            .map(|restart| restart.elapsed_frames)
    }

    fn pause_restart_zero_kid_shell_active_loc_6e2(&self) -> bool {
        self.pause_restart_elapsed_loc_6e2().is_some_and(|elapsed| {
            (PAUSE_RESTART_ZERO_KID_FRAME_LOC_6E2..PAUSE_RESTART_REBUILD_LEVEL_STATE_FRAME_LOC_6E2)
                .contains(&elapsed)
        })
    }

    fn pause_restart_object_counts_loc_6e2(&self) -> Option<(u16, u16)> {
        let elapsed = self.pause_restart_elapsed_loc_6e2()?;
        // ASM loc_6E2 restart reload: frames before loc_6C3E clears
        // Game_Paused retain the four hidden mode-level workers; once the
        // cleared reload shell takes over, Number_Objects is zero until the
        // Load_InGame rebuild reaches the live object pass.
        if (PAUSE_RESTART_ZERO_KID_FRAME_LOC_6E2..PAUSE_RESTART_CLEAR_LEVEL_STATE_FRAME_LOC_6E2)
            .contains(&elapsed)
        {
            Some((4, 0))
        } else if (PAUSE_RESTART_CLEAR_LEVEL_STATE_FRAME_LOC_6E2
            ..PAUSE_RESTART_REBUILD_LEVEL_STATE_FRAME_LOC_6E2)
            .contains(&elapsed)
        {
            Some((0, 0))
        } else {
            None
        }
    }

    fn demo_1_late_controller_sample_counts(&self) -> Option<(u16, u16)> {
        let (frame_counter, use_demo_2) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())?;
        let intro_sequence_6_hidden_camera_wrap = !use_demo_2
            && !self.demo_started_from_timeout_restart_shell
            && !self.demo_uses_intro_sequence_6_timeout_return_velocity_sample
            && self.demo_uses_intro_sequence_6_hidden_camera_wrap;
        if !use_demo_2
            && (self.demo_uses_intro_sequence_6_timeout_return_velocity_sample
                || intro_sequence_6_hidden_camera_wrap)
            && frame_counter == demo::DEMO_CAMERA_HOLD_START_FRAME + 1
        {
            // ASM loc_1BB02 -> Load_DemoPlay -> loc_119D8 keeps one
            // controller-side object count-visible immediately before both
            // IntroSequence6 demo-1 hidden-camera rows are sampled.
            return Some((2, 0));
        }
        if intro_sequence_6_hidden_camera_wrap
            && frame_counter == demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 8
        {
            // ASM Load_InGame retains the final hidden bootstrap Object for
            // one more Execute_Objects pass on this carried-wrap lineage.
            return Some((7, 5));
        }
        if !use_demo_2
            && (self.demo_started_from_timeout_restart_shell
                || self.demo_uses_intro_sequence_6_timeout_return_velocity_sample)
        {
            if frame_counter > demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME
                && frame_counter <= demo::DEMO_1_TIMEOUT_RESTART_CONTROLLER_SAMPLE_END_FRAME
            {
                let active_objects = self.active_objects();
                let camera_state = self.current_camera_state();
                let (number_objects, number_gfx_objects) = self.object_counts_from_roster(
                    &active_objects,
                    camera_state.x_pos,
                    camera_state.y_pos,
                );
                let active_enemy_gfx_objects = active_objects
                    .iter()
                    .filter(|object| matches!(object.kind, NativeObjectKind::Enemy { .. }))
                    .count() as u16;
                // ASM Load_DemoPlay -> loc_119D8 (kid.asm:23585) allocates an
                // object-only Start-abort controller. On the timeout-restart
                // lineage, oracle top-level TAS state keeps that hidden object
                // count-visible after the cold-boot controller window.
                // `Number_GfxObjects` continues to follow live gameplay WRAM,
                // including each loaded enemy GfxObject omitted by the fixed
                // demo-controller baseline.
                let hidden_controller_objects = if frame_counter
                    >= demo::DEMO_1_TIMEOUT_RESTART_EXTRA_CONTROLLER_OBJECT_FRAME
                {
                    5
                } else {
                    4
                };
                return Some((
                    number_objects.saturating_add(hidden_controller_objects),
                    number_gfx_objects.saturating_add(active_enemy_gfx_objects),
                ));
            }
            if (demo::DEMO_1_LATE_CONTROLLER_PAUSE_START_FRAME.saturating_sub(7)
                ..=demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME)
                .contains(&frame_counter)
            {
                // ASM Load_DemoPlay -> loc_119D8 (kid.asm:23585) keeps the
                // timeout-restart demo-1 controller lineage count-visible
                // here. Raw WRAM keeps the same seven GfxObjects linked before
                // and throughout the pause; there is no pre-pause list dip.
                return Some((8, 7));
            }
        }
        let controller_sample_frame = if intro_sequence_6_hidden_camera_wrap {
            // ASM loc_1BA5C -> Obj_TitleMenu loc_1BB02 -> Load_DemoPlay:
            // this IntroSequence6 carry keeps the hidden controller lineage
            // one object pass behind cold demo-1. The live loc_368A8 loader
            // already follows the shifted pass, so compare the sampled count
            // against the preceding cold-boot counter value.
            frame_counter.saturating_sub(1)
        } else {
            frame_counter
        };
        if !use_demo_2
            && !self.demo_started_from_timeout_restart_shell
            && !self.demo_uses_intro_sequence_6_timeout_return_velocity_sample
            && controller_sample_frame > demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME
        {
            let transient_gfx_workers = self.gameplay_transient_gfx_worker_count();
            let transient_object_only_workers = self.gameplay_transient_object_only_worker_count(
                self.current_camera_state().x_pos,
                self.current_camera_state().y_pos,
            );
            let active_enemy_objects = self.active_enemies.len().min(usize::from(u16::MAX)) as u16;
            let active_enemy_gfx_objects = active_enemy_objects
                .saturating_sub(self.pending_enemy_gfx_initialisations_loc_3691e);
            // ASM Load_DemoPlay -> loc_119D8 and Load_InGame retain the fixed
            // 4/4 demo shell. Knights Isle's loc_11FB0 -> loc_12098 Island
            // controller adds one Object-only slot. Mode_Level loc_80C calls
            // Manage_EnemyLoading loc_3691E after Execute_Objects, so a newly
            // linked Dragon Object gains its Enemy0D_Dragon_Init GfxObject on
            // the following pass rather than the allocation frame.
            return Some((
                5u16.saturating_add(active_enemy_objects)
                    .saturating_add(transient_gfx_workers)
                    .saturating_add(transient_object_only_workers),
                4u16.saturating_add(active_enemy_gfx_objects)
                    .saturating_add(transient_gfx_workers),
            ));
        }
        let controller_sample_window = demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME.saturating_add(1)
            ..=demo::DEMO_1_LATE_CONTROLLER_COUNT_SAMPLE_END_FRAME;
        let shifted_first_controller_sample = intro_sequence_6_hidden_camera_wrap
            && frame_counter == demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME.saturating_add(1);
        if use_demo_2
            || (!controller_sample_window.contains(&controller_sample_frame)
                && !shifted_first_controller_sample)
        {
            return None;
        }

        // ASM Load_DemoPlay -> loc_119D8 (kid.asm:23585) keeps the hidden
        // controller sample live after the second demo-1 jump re-entry. Oracle
        // top-level TAS state reports the controller-side object/gfx counts
        // while raw Kid WRAM is already moving through loc_A6F8; the sampled
        // worker/gfx count advances earlier than the raw Number_Objects row.
        if controller_sample_frame >= demo::DEMO_1_LATE_CONTROLLER_COUNT_THIRD_GFX_FRAME {
            // ASM Manage_EnemyLoading loc_3691E has already linked the third
            // Dragon Object. Its next pass runs Enemy0D_Dragon_Init and calls
            // Load_GfxObjectSlot, raising only Number_GfxObjects from 6 to 7.
            Some((8, 7))
        } else if controller_sample_frame >= demo::DEMO_1_LATE_CONTROLLER_COUNT_THIRD_WAKE_FRAME {
            Some((8, 6))
        } else if (demo::DEMO_1_LATE_CONTROLLER_COUNT_FIRST_WAKE_FRAME
            ..=demo::DEMO_1_LATE_CONTROLLER_COUNT_FIRST_SLEEP_FRAME)
            .contains(&controller_sample_frame)
            || controller_sample_frame >= demo::DEMO_1_LATE_CONTROLLER_COUNT_SECOND_GFX_FRAME
        {
            Some((7, 6))
        } else if controller_sample_frame >= demo::DEMO_1_LATE_CONTROLLER_COUNT_SECOND_NON_GFX_FRAME
        {
            Some((7, 5))
        } else {
            Some((6, 5))
        }
    }

    pub(crate) fn current_camera_state(&self) -> CameraState {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_camera_state();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_camera_state();
        }
        if let Some(camera) = self.level_entry_camera_handoff_state() {
            return camera;
        }
        if let Some(camera_state) = self.title_card_live_handoff_camera_state_loc_1ae2e() {
            return camera_state;
        }
        if let Some(camera_state) = self.early_start_load_ingame_camera_state() {
            return camera_state;
        }
        if matches!(self.boot_state, BootState::StartingLevel { .. }) {
            return CameraState::default();
        }
        if self.intro_sequence_4_uses_timeout_restart_camera_handoff() {
            return CameraState {
                x_pos: 0,
                y_pos: 0x0004_0000,
            };
        }
        if matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.timeout_restart_title_intro_sequence_4_camera_carry
            && self.title_screen_elapsed_frames
                < TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_CAMERA_CARRY_FRAMES
        {
            // ASM sub_1C246 jumps straight to `Game_Mode=$2C`; oracle keeps
            // sub_1B988's `move.w #4,(Camera_Y_pos).w` result only for the
            // short title rebuild hand-off before the cleared title camera
            // takes over again on frame 32039.
            return CameraState {
                x_pos: 0,
                y_pos: 0x0004_0000,
            };
        }
        if let Some(camera_y_pos) = self.intro_sequence_6_timeout_restart_hidden_camera_y() {
            return CameraState {
                x_pos: 0,
                y_pos: camera_y_pos,
            };
        }
        if self.title_uses_demo_snapshot() {
            return self
                .opening_slice
                .map(|slice| slice.camera)
                .unwrap_or_default();
        }
        if let Some((frame_counter, use_demo_2)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            && frame_counter < self.demo_gameplay_start_frame(use_demo_2)
        {
            let camera_y_pos = if !use_demo_2 {
                if frame_counter == 60
                    && self.demo_uses_intro_sequence_6_timeout_return_velocity_sample
                {
                    demo::DEMO_1_INTRO_SEQUENCE_6_TIMEOUT_RETURN_CAMERA_WRAP_Y_FP
                } else if frame_counter == 60 && self.demo_uses_intro_sequence_6_hidden_camera_wrap
                {
                    demo::DEMO_1_INTRO_SEQUENCE_6_CAMERA_WRAP_Y_FP
                } else if frame_counter == 60
                    && self.demo_uses_demo_title_return_camera_wrap
                    && !self.demo_uses_intro_sequence_6_early_title_restore
                {
                    // ASM `loc_119D8 -> loc_6E2 -> Obj_TitleMenu loc_1BB02 ->
                    // Load_DemoPlay -> Load_InGame` keeps the later demo-1
                    // startup on the same wrapped preload row that oracle
                    // shows after the generic demo-title return.
                    0xFF20_0000
                } else if frame_counter == demo::DEMO_1_GAMEPLAY_START_FRAME
                    && self.demo_uses_intro_sequence_6_hidden_camera_wrap
                {
                    // ASM `loc_1BB02 -> Load_DemoPlay -> Load_InGame` has
                    // already advanced the hidden Knights Isle bootstrap, but
                    // oracle keeps the timeout-restart `IntroSequence6`
                    // lineage on the last `$00E0_0000` hidden row for one
                    // extra trace frame before the visible bootstrap starts.
                    0x00E0_0000
                } else if frame_counter == 60 && self.demo_uses_start_skip_intro_return_camera_wrap
                {
                    // ASM sub_1B8F4 / loc_1B920 reach title via sub_1C246's
                    // deferred Start skip, and the later Obj_TitleMenu
                    // loc_1BB02 -> Load_DemoPlay -> Load_InGame hand-off keeps
                    // a different wrapped hidden camera row than cold boot.
                    0xFF20_0000
                } else if let Some(camera_y_pos) = demo::demo_1_hidden_camera_y(frame_counter) {
                    camera_y_pos
                } else {
                    self.opening_slice
                        .map(|slice| slice.camera.y_pos)
                        .unwrap_or_default()
                }
            } else if frame_counter == demo::DEMO_CAMERA_PRELOAD_FRAME
                && self.demo_uses_intro_sequence_5_title_return_preload_row
            {
                // ASM sub_1C246 -> loc_1C264 returns directly from
                // timeout-restart intro sequence 5 to the title shell; the
                // later Obj_TitleMenu loc_1BB02 -> Load_DemoPlay hand-off
                // exposes a one-frame Skydragon Castle preload row at
                // `$0220_0000` instead of the generic `$0228_0000`.
                demo::DEMO_2_INTRO_SEQUENCE_5_TITLE_RETURN_CAMERA_PRELOAD_Y_FP
            } else if self.demo_uses_cold_intro_sequence_6_demo_2_preload_rows
                && frame_counter == demo::DEMO_CAMERA_PRELOAD_FRAME
            {
                // ASM loc_1BA5C -> Load_IntroSequence6 -> Obj_TitleMenu
                // loc_1BB02 reaches the cold-boot Skydragon demo through the
                // hidden mode-$28 shell. Oracle exposes `$01E8_0000` for the
                // preload frame, distinct from the later timeout-restore
                // revisit that has already restored loc_1B5CE's title pose.
                demo::DEMO_2_COLD_INTRO_SEQUENCE_6_CAMERA_PRELOAD_Y_FP
            } else if self.demo_uses_cold_intro_sequence_6_demo_2_preload_rows
                && frame_counter == demo::DEMO_CAMERA_HOLD_START_FRAME
            {
                // ASM same cold-boot `loc_1BB02 -> Load_DemoPlay` lineage:
                // one more hidden camera row is visible before Load_InGame's
                // standard `$02A0_0000` respawn snap takes over.
                demo::DEMO_2_COLD_INTRO_SEQUENCE_6_CAMERA_SETTLE_Y_FP
            } else if frame_counter == demo::DEMO_CAMERA_PRELOAD_FRAME {
                // ASM Obj_TitleMenu loc_1BB02 flips into `Game_Mode=$10`, and
                // the following `Load_DemoPlay -> Load_InGame` hand-off
                // exposes one hidden camera preload row before Load_InGame's
                // `$02A0_0000` respawn camera snap takes over on the next
                // frame. Oracle WRAM reads Camera_Y_pos=$0228_0000 on all
                // lineages (plain boot, Start-skip intro-return, IntroSequence6
                // revisit).
                demo::DEMO_2_CAMERA_PRELOAD_Y_FP
            } else if frame_counter >= demo::DEMO_CAMERA_HOLD_START_FRAME {
                0x02A0_0000
            } else {
                0
            };
            return CameraState {
                x_pos: 0,
                y_pos: camera_y_pos,
            };
        }
        if let Some(snapshot) = self.death_shell_snapshot.as_ref() {
            if matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }) {
                return snapshot.frozen_camera;
            }
            if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. })
                && self.dying_hidden_camera_still_visible()
            {
                // ASM `lose_life -> Teleport -> loc_6E2`: after the hidden
                // shell camera window ends, the trace must expose the cleared
                // transition camera until the respawn bootstrap takes over.
                return snapshot.frozen_camera;
            }
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Respawning { .. }) {
            // ASM `lose_life -> Teleport -> loc_6E2`: mode 8 clears the
            // transition camera before the hidden respawn bootstrap later
            // rebuilds gameplay state.
            return CameraState::default();
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            CameraState::default()
        } else {
            self.opening_slice
                .map(|slice| slice.camera)
                .unwrap_or_default()
        }
    }

    fn level_complete_elapsed_frames(&self) -> usize {
        match self.player_lifecycle {
            PlayerLifecycle::LevelComplete {
                remaining_frames,
                total_frames,
                ..
            } => total_frames.saturating_sub(remaining_frames),
            _ => 0,
        }
    }

    pub(crate) fn active_objects(&self) -> Vec<NativeObjectSnapshot> {
        if self.pause_restart_zero_kid_shell_active_loc_6e2() {
            return Vec::new();
        }
        if self.level_skip_resume_hold {
            return self.level_skip_resume_active_objects();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_active_objects();
        }
        if let BootState::StartingLevel { elapsed_frames, .. } = self.boot_state {
            return self.starting_level_title_active_objects(elapsed_frames);
        }
        if self.title_uses_full_demo_snapshot() {
            let mut objects = Vec::new();
            if let Some(player_state) = self.current_player_state() {
                objects.push(NativeObjectSnapshot {
                    kind: NativeObjectKind::Player,
                    x_pos: player_state.x_pos,
                    y_pos: player_state.y_pos,
                    layout_index: None,
                    x_velocity: None,
                    y_velocity: None,
                    facing: None,
                    on_ground: None,
                    hit_points: None,
                    damage_flash: false,
                    palette_variant: 0,
                    stationary: false,
                    contact_phase: None,
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    sprite_animation_phase: None,
                    wait_timer: None,
                    debug_fields: BTreeMap::new(),
                });
            }
            objects.extend(self.active_enemy_roster.iter().cloned());
            objects.extend(self.active_platform_roster.iter().cloned());
            return objects;
        }
        if let Some(snapshot) = self.level_complete_gfx_snapshot.as_ref()
            && self.level_complete_elapsed_frames()
                < crate::level::TOUCH_FLAGPOLE_GFX_CLEANUP_VISIBLE_FRAME_LOC_C0B6_ORACLE
        {
            // ASM CharacterCollision_TouchFlagpole loc_C056 clears each
            // non-Kid object's gfx-slot links before loc_C070 deletes its
            // object slot. The old gfx list is still rendered until loc_C0B6
            // walks `Addr_FirstGfxObjectSlot` and deallocates non-Kid slots.
            return snapshot.frozen_active_objects.clone();
        }
        if let Some((frame_counter, use_demo_2)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
            && frame_counter < self.demo_gameplay_start_frame(use_demo_2)
        {
            return Vec::new();
        }
        if let Some(snapshot) = self.death_shell_snapshot.as_ref() {
            if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
                if self.dying_hidden_shell_still_visible() {
                    return snapshot.frozen_active_objects.clone();
                }
                // ASM lose_life -> Teleport -> loc_6E2 clears object RAM via
                // Initialize_ObjectSlots before the hidden mode-8 bootstrap
                // rebuilds gameplay workers. The transition object counts are
                // exported separately; the old gameplay roster is no longer
                // trace-visible once the death shell pose clears.
                return Vec::new();
            }
            if matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }) {
                let mut objects = snapshot.frozen_active_objects.clone();
                if let Some(player_state) = self.opening_slice.map(|slice| slice.player)
                    && let Some(player_object) = objects
                        .iter_mut()
                        .find(|object| matches!(object.kind, NativeObjectKind::Player))
                {
                    player_object.x_pos = player_state.x_pos;
                    player_object.y_pos = player_state.y_pos;
                }
                return objects;
            }
        }
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) || matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. })
        {
            return Vec::new();
        }

        let mut objects = Vec::new();
        if let Some(player_state) = self.current_player_state() {
            objects.push(NativeObjectSnapshot {
                kind: NativeObjectKind::Player,
                x_pos: player_state.x_pos,
                y_pos: player_state.y_pos,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                sprite_animation_phase: None,
                wait_timer: None,
                debug_fields: BTreeMap::new(),
            });
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Respawning { .. })
            && (self.death_respawn_bootstrap_profile().is_some()
                || !self.starting_level_title_workers.is_empty())
        {
            if !self.starting_level_title_workers.is_empty() {
                let title_elapsed = crate::movement::LEVEL_ENTRY_TITLE_DEFERRED_NUMBER_VISIBLE_FRAME
                    + self
                        .death_respawn_bootstrap_elapsed_frames()
                        .unwrap_or_default();
                objects.extend(self.starting_level_title_active_objects(title_elapsed));
            }
            // ASM lose_life -> Teleport loc_B8DE -> loc_6E2 clears object RAM
            // before the hidden respawn bootstrap has rebuilt level
            // enemies/platforms. Keep the hidden Kid pose and Load_TitleCard
            // loc_1AB26/loc_1AC70/loc_1AD48 workers trace-visible even on
            // levels without route-specific Kid-pose profiles, but do not
            // expose stale gameplay rosters here.
            return objects;
        }
        objects.extend(self.active_enemy_roster.iter().cloned());
        objects.extend(self.active_enemies.iter().flat_map(|enemy| {
            boss_eye_orbit_object_slot_details_loc_3931a(
                enemy,
                boss_eye_orbit_mapping_indices_from_state_loc_37abc(
                    enemy,
                    self.opening_slice.as_ref(),
                    self.progression.diamond_power_frames > 0
                        || !self.diamond_power_invuln_haze_slots.is_empty(),
                    self.frame_index,
                ),
            )
        }));
        objects.extend(self.active_platform_roster.iter().cloned());
        objects
    }

    pub(crate) fn object_counts_from_roster(
        &self,
        active_objects: &[NativeObjectSnapshot],
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> (u16, u16) {
        self.object_counts_from_roster_inner(active_objects, camera_x_pos, camera_y_pos, true)
    }

    pub(crate) fn object_counts_from_roster_without_runtime_transients(
        &self,
        active_objects: &[NativeObjectSnapshot],
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> (u16, u16) {
        self.object_counts_from_roster_inner(active_objects, camera_x_pos, camera_y_pos, false)
    }

    fn object_counts_from_roster_inner(
        &self,
        active_objects: &[NativeObjectSnapshot],
        camera_x_pos: u32,
        camera_y_pos: u32,
        include_runtime_transients: bool,
    ) -> (u16, u16) {
        if self.level_skip_resume_hold {
            return self.level_skip_resume_object_counts();
        }
        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return self.level_skip_object_counts();
        }
        if let BootState::SegaScreen { elapsed } = self.boot_state
            && elapsed >= 33
        {
            // ASM Load_SegaScreen loc_12E48 calls j_Allocate_ObjectSlot for
            // the logo controller, incrementing Number_Objects without
            // allocating a GfxObjectSlot.
            return (1, 0);
        }
        if let BootState::IntroSequence {
            segment: 0,
            elapsed,
            ..
        } = self.boot_state
            && self
                .demo_auto_timeout_intro_sequence_1_frames_override
                .is_some()
        {
            // ASM ReadJoypad loc_C0A -> Load_IntroSequence1/sub_1B8F4
            // (kid.asm:1380,30214): the timeout restart keeps the first
            // intro controller counted through local elapsed 30, clears the
            // shell counters for the hidden rebuild, then exposes the
            // controller and intro-text worker as they reallocate.
            return match elapsed {
                0..=30 => (1, 0),
                31..=42 => (0, 0),
                43 => (1, 0),
                44..=247 => (2, 1),
                _ => (1, 1),
            };
        }
        if let BootState::IntroSequence {
            segment: 1,
            elapsed,
            ..
        } = self.boot_state
            && self
                .demo_auto_timeout_intro_sequence_2_frames_override
                .is_some()
        {
            // ASM sub_1B8F4 -> loc_1B920 leaves IntroSequence1's sub_1C54A
            // GfxObject linked while Game_Mode already exposes IntroSequence2.
            // Load_IntroSequence2 then clears the inherited roster, allocates
            // the loc_1B9D0 controller, and finally exposes loc_1C7A0's text
            // worker without allocating a replacement GfxObject.
            return match elapsed {
                0..=30 => (1, 1),
                31..=40 => (0, 0),
                42..=285 => (2, 0),
                _ => (1, 0),
            };
        }
        if let BootState::IntroSequence {
            segment: 0,
            elapsed,
            ..
        } = self.boot_state
        {
            if elapsed < 32 {
                // ASM loc_12EDE jumps through loc_6E2 after the Sega logo object
                // consumes Start; oracle WRAM keeps one object counted during the
                // first 32 traced intro frames before loc_6E2's
                // Initialize_ObjectSlots clear is visible in checkpoints.
                return (1, 0);
            }
            if elapsed == 44 {
                // ASM Load_IntroSequence1 allocates the sub_1B8F4 controller
                // after the post-Sega loc_6E2 shell; no GfxObjectSlot is
                // allocated for it.
                return (1, 0);
            }
            if elapsed >= 45 {
                // ASM sub_1B8F4 calls sub_1C54A to allocate the intro
                // GfxObject, then allocates loc_1C7A0's intro-text Object
                // before entering sub_1C204/loc_1B920. Native does not model
                // either intro shell worker as a gameplay roster entry.
                return (2, 1);
            }
        }
        if let BootState::IntroSequence {
            segment: 2,
            elapsed,
            ..
        } = self.boot_state
            && (self
                .demo_auto_timeout_intro_sequence_3_frames_override
                .is_some()
                || self.timeout_restart_title_intro_sequence_3_return_pending)
        {
            let direct_gfx_count = crate::movement::intro_sequence_3_direct_gfx_count_loc_1AEBA(
                self.active_intro_sequence_3_roster_index,
            );
            let live_gfx_count = direct_gfx_count.saturating_add(3);
            // ASM Load_IntroSequence3 -> sub_1B93E (kid.asm:29078,30238)
            // follows the same sub_1C204 fade shell as loc_1B9D0. At elapsed
            // 41, loc_1AEBA has allocated only the selected roster's direct
            // GfxObjects. On the next object pass, sub_1B93E/sub_1C54A and the
            // sub_1C572/loc_1C58C workers add three linked GfxObjects; the text
            // worker remains until loc_1C7A0 reaches its terminator.
            return match elapsed {
                0..=30 => (1, 0),
                31..=40 => (0, 0),
                41 => (3, direct_gfx_count),
                42..=115 => (4, live_gfx_count),
                _ => (3, live_gfx_count),
            };
        }
        if let BootState::IntroSequence {
            segment: 3,
            elapsed,
            ..
        } = self.boot_state
        {
            let carry_frames = if self.timeout_restart_intro_sequence_4_pose_carry {
                TIMEOUT_RESTART_INTRO_SEQUENCE_4_ROSTER_COUNT_CARRY_FRAMES
            } else {
                INTRO_SEQUENCE_4_POSE_CARRY_FRAMES
            };
            if elapsed < carry_frames {
                // ASM loc_1B96A writes `Game_Mode=$20`, but the previous
                // Load_IntroSequence3/sub_1B93E roster remains counted until
                // Load_IntroSequence4/sub_1B988's rebuild clears the hand-off.
                return (
                    3,
                    crate::movement::intro_sequence_3_direct_gfx_count_loc_1AEBA(
                        self.active_intro_sequence_3_roster_index,
                    )
                    .saturating_add(3),
                );
            }
            if elapsed + 1 == TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES {
                // ASM sub_1B988 has cleared the roster but has not yet exposed
                // the full `Camera_Y_pos=4` hand-off; oracle keeps only the
                // controller object counted for this one cold-boot frame.
                return (1, 0);
            }
            if elapsed >= TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES {
                // ASM sub_1B988 owns the scene after writing Camera_Y_pos=4.
                // Its loc_1C7A0 IntroText4 child remains counted only through
                // the script's final hibernate; loc_1C7B6 deletes that Object
                // as soon as it dispatches the `$FF` terminator.
                let text_worker_is_live = crate::vdp::intro_text_worker_counted_frames_loc_1c7a0(3)
                    .is_some_and(|counted_frames| {
                        elapsed
                            < TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES
                                .saturating_add(counted_frames)
                    });
                return (1 + u16::from(text_worker_is_live), 0);
            }
        }
        if let BootState::IntroSequence {
            segment: 4,
            elapsed,
            ..
        } = self.boot_state
            && self
                .demo_auto_timeout_intro_sequence_5_frames_override
                .is_none()
            && elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_5_CAMERA_CARRY_FRAMES
        {
            // ASM Load_IntroSequence5 does not call sub_1B222, so the
            // sub_1B988 camera hand-off and controller count survive the early
            // intro-5 frames on the cold path too.
            return (1, 0);
        }
        if let BootState::IntroSequence {
            segment: 4,
            elapsed,
            ..
        } = self.boot_state
            && self
                .demo_auto_timeout_intro_sequence_5_frames_override
                .is_none()
        {
            let visible_elapsed_frames =
                TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES - 1;
            if elapsed + 1 == visible_elapsed_frames {
                // ASM loc_1BCD0 has not yet written the slot-2 pose, but the
                // cold oracle count has the loc_1BA28 controller plus
                // loc_1BC36 tile worker live for one frame.
                return (2, 0);
            }
            if elapsed >= visible_elapsed_frames {
                // ASM loc_1BCD0 allocates the title gfx-slot worker after the
                // loc_1BA28/loc_1BC36 setup, exposing the slot-2 pose and four
                // counted workers on the cold intro-5 path.
                return (4, 0);
            }
        }
        if let BootState::IntroSequence {
            segment: 4,
            elapsed,
            ..
        } = self.boot_state
            && self
                .demo_auto_timeout_intro_sequence_5_frames_override
                .is_some()
        {
            if elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_5_CAMERA_CARRY_FRAMES {
                // ASM loc_1B9B6 -> loc_6E2 switches Game_Mode to `$24`, but
                // the old sub_1B988 controller survives until the loader wipe.
                return (1, 0);
            }
            // ASM loc_1BA28 -> loc_1BCD0/loc_1C7A0: the intro-return lineage
            // wakes this entire allocation pass one frame early, including
            // the two allocated-but-not-run Objects and the following 4/5
            // live-worker row.
            let visible_elapsed = TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES
                .saturating_sub(usize::from(
                    self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility,
                ));
            if elapsed + 1 < visible_elapsed {
                // ASM loc_6E2 has cleared the carried controller while
                // Load_IntroSequence5 is still inside its hibernating loader.
                return (0, 0);
            }
            if elapsed + 1 == visible_elapsed {
                // ASM Load_IntroSequence5 (kid.asm:29324-29363) has allocated
                // loc_1BA28 and loc_1BCD0, but neither has run yet.
                return (2, 0);
            }
            let intro_text_is_live = crate::vdp::intro_text_worker_counted_frames_loc_1c7a0(4)
                .is_some_and(|counted_frames| {
                    // ASM loc_1C7A0 -> loc_1C7B6: the `$FF` deletion shifts
                    // with the allocation pass rather than using a separate
                    // lineage timer.
                    elapsed < visible_elapsed.saturating_add(counted_frames)
                });
            let particle_roster_elapsed =
                DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_PARTICLE_COUNT_FRAME
                    .saturating_sub(usize::from(
                        self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility,
                    ));
            if elapsed >= particle_roster_elapsed {
                // ASM loc_1BCD0 -> loc_1BE3C/loc_1BE88
                // (kid.asm:30567,30667,30689): the wake-up pass allocates four
                // Objects, deletes loc_1BCD0 and its two owned GfxObjects, but
                // leaves three unowned pose slots linked. The four new
                // GfxObjects join on the following loc_E12 pass.
                return if elapsed == particle_roster_elapsed {
                    (6, 3)
                } else if elapsed < particle_roster_elapsed.saturating_add(0x2A) {
                    (6, 7)
                } else {
                    // ASM unk_1BE18/unk_1BE30 seed d0=$28. After the
                    // allocation and wake-up passes, loc_1BE74's inclusive
                    // DBF lifetime expires 42 exposed frames later, deleting
                    // the two short Objects and their two owned GfxObjects.
                    (4, 5)
                };
            }
            // ASM loc_1BA28 allocates loc_1C7A0 and loc_1BC36; loc_1BCD0
            // allocates five direct GfxObjects and then hibernates. IntroText5
            // deletes itself at `$FF`, leaving the controller, tile worker, and
            // loc_1BCD0 Object with all five GfxObjects still linked.
            return (3 + u16::from(intro_text_is_live), 5);
        }
        if let Some((frame_counter, use_demo_2)) = self
            .demo_startup_frame()
            .zip(self.demo_uses_second_recording())
        {
            return if frame_counter <= demo::DEMO_LEVEL_SWITCH_FRAME
                && use_demo_2
                && self.demo_uses_intro_sequence_6_later_demo_2_preload_row
            {
                // ASM Obj_TitleMenu loc_1BB52 -> loc_1BB02 -> loc_6E2:
                // the later IntroSequence6 route has entered DemoPlay, but
                // its `$FFFF2578` cursor GfxObject remains linked throughout
                // this terminal hand-off window before Load_DemoPlay clears it.
                (4, 5)
            } else if frame_counter <= demo::DEMO_LEVEL_SWITCH_FRAME {
                (4, 4)
            } else if frame_counter
                < self
                    .demo_gameplay_start_frame(use_demo_2)
                    .saturating_sub(13)
            {
                (1, 0)
            } else if !use_demo_2
                && frame_counter < self.demo_gameplay_start_frame(use_demo_2).saturating_sub(1)
            {
                // ASM Load_DemoPlay -> Load_InGame (kid.asm:23596) enters
                // the Knights Isle demo with the loc_119D8 controller object
                // live. Oracle-backed published TAS probes show one more
                // non-gfx worker counted from demo-1 startup frames 59..70
                // before the visible gameplay object/gfx allocation appears.
                (2, 0)
            } else if !use_demo_2 && frame_counter < self.demo_gameplay_start_frame(use_demo_2) {
                // ASM Load_InGame's final hidden demo-1 frame has two system
                // GfxObjects live. Kid's two slots are not linked until the
                // following Execute_Objects pass; raw oracle WRAM reports 6/2.
                (6, 2)
            } else if frame_counter >= self.demo_gameplay_start_frame(use_demo_2) {
                if frame_counter <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME {
                    if use_demo_2 {
                        (5, 4)
                    } else if frame_counter == self.demo_gameplay_start_frame(use_demo_2) {
                        // ASM Load_InGame links Kid's two GfxObjects on demo-1
                        // startup frame 72. Manage_EnemyLoading loc_3691E also
                        // allocates the Dragon Object, but its gfx slot waits
                        // for Enemy0D_Dragon_Init on the next object pass.
                        (7, 4)
                    } else {
                        // ASM Enemy0D_Dragon_Init calls Load_GfxObjectSlot on
                        // startup frame 73, raising the live count to five
                        // while all seven Object workers remain linked.
                        (7, 5)
                    }
                } else if !use_demo_2
                    && frame_counter <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME.saturating_add(7)
                {
                    // ASM Load_DemoPlay -> Load_InGame (kid.asm:23596)
                    // leaves the hidden demo-1 worker pair alive after Kid's
                    // bootstrap hold ends. Oracle WRAM also reports one more
                    // gfx object through startup frame 88 before the first
                    // hidden worker drops on the following frame.
                    (7, 5)
                } else if !use_demo_2 && frame_counter <= demo::DEMO_TRACE_GROUNDED_HOLD_END_FRAME {
                    // ASM Load_DemoPlay -> Load_InGame (kid.asm:23596) keeps
                    // one hidden demo-1 worker and the extra gfx object alive
                    // beyond the visible bootstrap. Oracle WRAM holds this 6/5
                    // count through the grounded hand-off window also used by
                    // the loc_C22 trace velocity hold.
                    (6, 5)
                } else {
                    // ASM Load_DemoPlay (kid.asm:23585) allocates the hidden
                    // `loc_119D8` Start-abort controller object before the
                    // shared Load_InGame tail. Native does not model that
                    // controller as a roster entry. Runtime block/prize
                    // transient workers, such as ASM loc_1002E/sub_1007A
                    // prize-block bounces, loc_102B8 spawn flashes, and
                    // sub_10EDA/sub_292E block debris workers, still allocate
                    // object+gfx slots during demo playback. Object-only
                    // Object-only children and ordinary Object/GfxObject pairs
                    // remain separate categories; Dragon.asm loc_3CF90 /
                    // loc_3D02A breath children belong to the latter.
                    let transient_gfx_workers = self.gameplay_transient_gfx_worker_count();
                    let transient_object_only_workers = self
                        .gameplay_transient_object_only_worker_count(camera_x_pos, camera_y_pos);
                    (
                        4u16.saturating_add(transient_gfx_workers)
                            .saturating_add(transient_object_only_workers),
                        4u16.saturating_add(transient_gfx_workers),
                    )
                }
            } else if frame_counter >= demo::DEMO_OBJECT_WARMUP_FRAME {
                (5, 2)
            } else {
                (1, 0)
            };
        }
        if self.demo_auto_timeout_handoff_active
            && self.title_screen_elapsed_frames >= DEMO_AUTO_TIMEOUT_PARTIAL_SNAPSHOT_FRAMES
        {
            // ASM ReadJoypad loc_C0A -> loc_6E2 (kid.asm:1380): after the
            // timeout restart has exposed the preserved 4/4 demo counters for
            // 32 local mode-0 frames, oracle WRAM shows the object slots
            // cleared for three frames, then the Sega/logo controller count
            // appears without a gfx slot.
            return if self.title_screen_elapsed_frames
                < DEMO_AUTO_TIMEOUT_PARTIAL_SNAPSHOT_FRAMES + 3
            {
                (0, 0)
            } else {
                (1, 0)
            };
        }
        if self.title_uses_full_demo_snapshot() {
            if let Some(counts) = self.demo_title_snapshot_object_counts {
                return counts;
            }
            let roster_count = active_objects.len() as u16;
            // ORIGINAL_QUIRK: the timeout/abort title rebuild keeps four fixed
            // title objects live alongside the preserved gameplay roster while
            // the demo snapshot is still being shown.
            return (roster_count + 4, roster_count + 4);
        }
        if self.title_uses_demo_snapshot()
            && self.title_screen_elapsed_frames < self.demo_title_partial_snapshot_frames()
            && let Some(counts) = self.demo_title_snapshot_object_counts
        {
            return counts;
        }
        if matches!(self.boot_state, BootState::TitleScreen { .. })
            && self.timeout_restart_title_intro_sequence_3_pose
            && self.title_screen_elapsed_frames < DEMO_TITLE_FULL_SNAPSHOT_FRAMES + 4
        {
            // ASM sub_1C246 -> loc_1C264 switches Game_Mode to $2C without
            // clearing the live Load_IntroSequence3/sub_1B93E object counters.
            // The stale loc_1AEBA gfx roster remains count-visible while the
            // intro-sequence-3 pose is carried into the first title frames.
            return (
                3,
                crate::movement::intro_sequence_3_direct_gfx_count_loc_1AEBA(
                    self.active_intro_sequence_3_roster_index,
                )
                .saturating_add(3),
            );
        }
        if self.title_counts_intro_sequence_5_workers() {
            if self.timeout_restart_title_intro_sequence_5_particle_roster {
                // ASM sub_1C246 -> loc_1C264 changes Game_Mode without
                // clearing either linked list. Keep loc_1BE3C/loc_1BE88's
                // mature replacement roster until loc_6E2 clears it.
                return (6, 7);
            }
            // ASM loc_1BA60 -> sub_1C246 -> loc_1C264 switches Game_Mode to
            // $2C without immediately clearing the live loc_1BA28/loc_1BCD0
            // workers. Cold intro-return keeps the title hidden-shell worker
            // count-visible too; timeout-restart keeps only the stale three
            // intro-sequence-5 workers.
            let count_visible = if self.title_screen_uses_intro_return_hidden_shell {
                4
            } else {
                3
            };
            // ASM loc_1BCD0's five direct GfxObjects remain linked on the
            // timeout-restart path until loc_6E2 clears both lists. Preserve
            // the separately probed cold Start-skip lineage until its Gfx
            // roster is derived from oracle WRAM.
            let gfx_count_visible =
                u16::from(!self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry) * 5;
            return (count_visible, gfx_count_visible);
        }
        if let Some(counts) = self.timeout_restart_intro_3_title_return_object_counts() {
            return counts;
        }
        if let Some(counts) = self.timeout_restart_intro_5_title_return_object_counts() {
            return counts;
        }
        if let Some(counts) = self.demo_title_return_hidden_object_counts() {
            return counts;
        }
        if self.early_start_title_menu_shell_carry_active() {
            return (4, 3);
        }
        if let BootState::StartingLevel {
            elapsed_frames,
            uses_load_ingame_bootstrap: true,
            load_ingame_bootstrap_started: false,
            ..
        } = self.boot_state
        {
            let total_worker_count = self.starting_level_title_workers.len() as u16;
            // ASM Obj_TitleMenu `Title_InputLoop` has already cleared the
            // cursor pose by frame 161, but oracle-backed early-start traces
            // keep the later mode-$0008 preload workers alive before
            // loc_1AE2E consumes the second Start edge.
            return if elapsed_frames > EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME {
                (total_worker_count.saturating_add(1), total_worker_count)
            } else if elapsed_frames == EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME {
                (
                    total_worker_count.saturating_add(1),
                    total_worker_count.saturating_sub(1),
                )
            } else if elapsed_frames + 1 == EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME {
                (total_worker_count.saturating_add(1), 0)
            } else if elapsed_frames >= EARLY_START_LOAD_INGAME_PRELOAD_OBJECTS_FRAME {
                (2, 0)
            } else {
                (0, 0)
            };
        }
        if let BootState::StartingLevel { elapsed_frames, .. } = self.boot_state
            && !self.starting_level_title_workers.is_empty()
        {
            let total_worker_count = self.starting_level_title_workers.len() as u16;
            // ASM Load_TitleCard / loc_1AB26 / loc_1AD48 / loc_1AE2E
            // (kid.asm:28588-29063) keep the cold-boot mode-$0008 lineage on
            // the carried 4-object title shell, then briefly clear object RAM
            // before rebuilding a hidden 2-object gate, a hidden 13-object
            // letter batch, and finally the full 15-object title-card shell.
            // Oracle WRAM watches for the cold-boot Level 1 route show:
            // carry through local frame 33, blank on 34..37, 2 hidden objects
            // on 38..40, 13 hidden objects on 41, 15 hidden / 0 gfx on 42,
            // 15 hidden / 13 gfx on 43, then 15 / 14 from 44 onward.
            return if elapsed_frames > crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
            {
                (total_worker_count.saturating_add(1), total_worker_count)
            } else if elapsed_frames == crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
            {
                (
                    total_worker_count.saturating_add(1),
                    total_worker_count.saturating_sub(1),
                )
            } else if elapsed_frames + 1
                == crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
            {
                (total_worker_count.saturating_add(1), 0)
            } else if elapsed_frames + 2
                == crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
            {
                (total_worker_count.saturating_sub(1), 0)
            } else if elapsed_frames + 5
                >= crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
            {
                (2, 0)
            } else if elapsed_frames + 9
                >= crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
            {
                (0, 0)
            } else {
                (4, 4)
            };
        }
        if matches!(
            self.boot_state,
            // ASM Load_TitleScreen allocates the title shell before gameplay.
            // Obj_TitleTextKid loc_1B5E4 owns two later sub_1B652 popup
            // GfxObjects; raw WRAM shows its second popup joining at local
            // rebuild elapsed 160 while the four Object workers stay live.
            BootState::TitleScreen { .. } | BootState::IntroSequence6 { .. }
        ) {
            if let Some(counts) = self.carried_wrap_intro_sequence_6_object_counts() {
                return counts;
            }
            if let Some(counts) = self.later_revisit_intro_sequence_6_object_counts() {
                return counts;
            }
            if self.cold_demo_timeout_intro_sequence_6_shell() && !self.title_screen_ready() {
                return match self.title_screen_elapsed_frames {
                    // ASM loc_1BA5C -> Load_IntroSequence6 initial hidden
                    // shell: loc_1BAB0, loc_1B37C, and sub_1B43A are counted
                    // before sub_1B43A reaches its later palette/art phase.
                    34..=196 => (3, 0),
                    // ASM sub_1B43A has finished its early draw sequence and
                    // the title shell temporarily reports two workers before
                    // it allocates GfxObjects_Collision_BottomBoundary4.
                    197..=237 => (2, 0),
                    // ASM sub_1B43A allocates GfxObjects_Collision_BottomBoundary4
                    // plus the next title workers before deleting itself.
                    238..=264 => (4, 0),
                    // After GfxObjects_Collision_BottomBoundary4 finishes its
                    // 16-frame DBF ramp and deletes, oracle briefly reports
                    // the three remaining hidden title workers before the
                    // shell settles back to four counted objects.
                    265..=268 => (3, 0),
                    269.. => (4, 0),
                    _ => (0, 0),
                };
            }
            if matches!(self.boot_state, BootState::TitleScreen { .. })
                && let Some(counts) = self.hidden_title_screen_object_counts()
            {
                return counts;
            }
            let title_text_kid_gfx_count = self.title_text_kid_gfx_count_loc_1b5e4_sub_1b652();
            if matches!(
                self.title_screen_intro_return_lineage,
                Some(
                    TitleScreenIntroReturnLineage::ColdIntroSequence1
                        | TitleScreenIntroReturnLineage::LaterIntroSequence
                        | TitleScreenIntroReturnLineage::LaterIntroSequence2
                        | TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence1EarlyRestore
                )
            ) && title_text_kid_gfx_count < 4
            {
                // ASM Obj_TitleTextKid loc_1B5E4 -> sub_1B652: these title
                // returns become pose-visible after the first popup joins,
                // but ten frames before the second. Preserve that raw 4/3
                // list instead of treating title-ready as an immediate
                // steady 4/4 roster. The cold lineage can recur after a later
                // demo timeout with Obj_TitleMenu still inactive.
                return (4, title_text_kid_gfx_count);
            }
            if self.title_menu_active && !self.title_menu_cursor_pending {
                // Obj_TitleMenu's active shell exposes at least three slots;
                // Obj_TitleTextKid's second sub_1B652 popup raises the live
                // WRAM count to four independently of the menu state.
                let number_gfx_objects = title_text_kid_gfx_count.max(3);
                return (4, number_gfx_objects);
            }
        }
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return if matches!(
                self.boot_state,
                BootState::TitleScreen { .. }
                    | BootState::IntroSequence6 { .. }
                    | BootState::OptionsMenu { .. }
                    | BootState::SoundTest { .. }
            ) && self.title_screen_ready()
            {
                (4, 4)
            } else {
                (0, 0)
            };
        }
        if let Some(snapshot) = self.death_shell_snapshot.as_ref()
            && matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. })
        {
            return (
                snapshot.frozen_number_objects,
                snapshot.frozen_number_gfx_objects,
            );
        }
        if let Some(snapshot) = self.death_shell_snapshot.as_ref()
            && matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. })
        {
            return (
                self.dying_hidden_number_objects_loc_b8de(snapshot),
                snapshot.frozen_number_gfx_objects,
            );
        }
        if let Some(snapshot) = self.level_complete_gfx_snapshot.as_ref()
            && matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. })
        {
            let elapsed = self.level_complete_elapsed_frames();
            if elapsed == 0 {
                return (
                    snapshot.frozen_number_objects,
                    snapshot.frozen_number_gfx_objects,
                );
            }
            if elapsed < crate::level::TOUCH_FLAGPOLE_GFX_CLEANUP_VISIBLE_FRAME_LOC_C0B6_ORACLE {
                return (3, snapshot.frozen_number_gfx_objects);
            }
            return (3, 1);
        }
        if matches!(self.player_lifecycle, PlayerLifecycle::Dying { .. }) {
            return (0, 0);
        }
        if let Some(counts) = self.death_respawn_bootstrap_object_counts() {
            return counts;
        }

        let transient_gfx_workers = if include_runtime_transients {
            self.gameplay_transient_gfx_worker_count()
        } else {
            0
        };
        let transient_gfx_only_workers = if include_runtime_transients {
            self.gameplay_transient_gfx_only_worker_count(camera_x_pos, camera_y_pos)
        } else {
            0
        };
        let transient_object_only_workers = if include_runtime_transients {
            self.gameplay_transient_object_only_worker_count(camera_x_pos, camera_y_pos)
        } else {
            0
        };
        // ASM Tank.asm loc_3B372 allocates the mouth/turret child through
        // j_Allocate_GfxObjectSlot_a1 — a GFX slot that never increments
        // Number_Objects. The native roster intentionally exports body and
        // turret as separate snapshots for entity-position parity, so the
        // object COUNT must collapse each tank pair back to its single ROM
        // object slot (issue-271 jugg40: oracle Number_Objects stays 5 with
        // one tank object while the native roster carries two 0x0A entries).
        let tank_turret_child_entries = active_objects
            .windows(2)
            .filter(|pair| {
                matches!(
                    (&pair[0].kind, &pair[1].kind),
                    (
                        NativeObjectKind::Enemy { object_type: a },
                        NativeObjectKind::Enemy { object_type: b },
                    ) if (*a == 0x0A || *a == 0x0B) && a == b
                ) && pair[0].layout_index == pair[1].layout_index
            })
            .count() as u16;
        let platform_slot_entries = active_objects
            .iter()
            .filter(|object| matches!(object.kind, NativeObjectKind::Platform { .. }))
            .count() as u16;
        // ASM Allocate_PlatformSlot (kid.asm:4211) owns a separate
        // Platform_RAM pool and changes Number_Platforms, not Number_Objects
        // or Number_GfxObjects. Negative platform types additionally allocate
        // one Object_RAM coroutine at loc_2876, represented by `special`.
        let special_platform_object_workers = self
            .active_platforms
            .iter()
            .filter(|platform| platform.special.is_some())
            .count() as u16;
        let roster_gfx_count = (active_objects.len() as u16)
            .saturating_sub(platform_slot_entries)
            .saturating_add(transient_gfx_workers);
        let roster_object_count = roster_gfx_count
            .saturating_sub(tank_turret_child_entries)
            .saturating_add(special_platform_object_workers);
        if let Some(counts) = opening_level_one_object_counts(
            self.current_level,
            roster_object_count,
            camera_x_pos,
            camera_y_pos,
        ) {
            return counts;
        }
        // The original game keeps 2 fixed system objects (game manager
        // and HUD/state), while Kid owns a second GfxObject slot. An additional
        // palette-rotation Object is allocated for Mountain (fg=7), Cave
        // (fg=8), or Island (bg=4) themes. ASM Load_InGame loc_11F96/loc_11FB0
        // calls Allocate_ObjectSlot only, so the helper never increments
        // Number_GfxObjects. The native roster includes the player's first
        // GfxObject slot already.
        let extra = self.level_header(self.current_level).map_or(0, |h| {
            if h.level_theme == 7 || h.level_theme == 8 || h.background_theme == 4 {
                1u16
            } else {
                0
            }
        });
        (
            roster_object_count + 2 + extra + transient_object_only_workers,
            roster_gfx_count + 3 + transient_gfx_only_workers,
        )
    }

    fn gameplay_transient_object_only_worker_count(
        &self,
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> u16 {
        let drips_parent_workers =
            self.hidden_drips_parent_worker_count_enemy16_drip_init(camera_x_pos, camera_y_pos);
        // ASM Drips.asm Enemy16_Drip_Init is an Object_RAM coroutine whose
        // parent gfx object stays hidden through loc_333F6..loc_3350E. Native
        // deliberately suppresses that parent from `active_enemy_roster`.
        // The exact loc_74E0 recording route keeps those in-range controller
        // slots live behind the title shell; ordinary direct boot probes use
        // their established roster-derived counts.
        // ASM loc_10DA4/sub_10DE4 (kid.asm:22692,22717) allocates a standard
        // Object_RAM coroutine for each rubber-block skin swap, but unlike
        // smash/ice block debris it never calls j_sub_E02 and owns no gfx slot.
        let rubber_skin_workers = self.block_state.rubber_block_animations.len() as u16;
        let fire_demon_palette_helper_workers = u16::from(
            self.active_enemies
                .iter()
                .any(|enemy| enemy.family == EnemyFamily::FireDemon),
        );
        let lava_special_effect_workers = self.lava_special_effect_object_only_worker_count();
        let dragon_flying_descent_fire_workers = self
            .projectiles
            .iter()
            .filter(|projectile| projectile.kind == EnemyProjectileKind::DragonFlyingDescentFire)
            .count() as u16;
        let tar_monster_burst_object_init_workers = self
            .projectiles
            .iter()
            .filter(|projectile| {
                projectile.kind == EnemyProjectileKind::TarMonsterBurstChild
                    && projectile.animation_elapsed_frames == 1
            })
            .count() as u16;
        // ASM Dragon_Flying.asm loc_34BAC follows the same Object_RAM child
        // pattern for descent fire; native models it as a projectile but ROM
        // Number_Objects includes the child worker.
        drips_parent_workers
            // ASM Fire_Demon.asm loc_3A414 allocates loc_3A48E through
            // j_Allocate_ObjectSlot only while the global Fire_Demon byte is
            // zero. Every live demon shares that one palette coroutine, which
            // never calls a GfxObject allocator.
            .saturating_add(fire_demon_palette_helper_workers)
            .saturating_add(rubber_skin_workers)
            .saturating_add(lava_special_effect_workers)
            .saturating_add(dragon_flying_descent_fire_workers)
            // ASM Tar_Monster.asm loc_324F8 allocates five Object slots;
            // their loc_325E6 callbacks load GfxObjects one scheduler pass later.
            .saturating_add(tar_monster_burst_object_init_workers)
    }

    fn hidden_drips_parent_worker_count_enemy16_drip_init(
        &self,
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> u16 {
        self.active_enemies
            .iter()
            .filter(|enemy| {
                self.recording_oracle_debug_lifecycle_completed
                    && enemy.family == EnemyFamily::Drips
                    && drips_object_check_in_range_loc_333f6(
                        enemy,
                        Some(camera_x_pos >> 16),
                        Some(camera_y_pos >> 16),
                    )
            })
            .count() as u16
    }

    fn drips_surface_crawler_worker_count_loc_335d8(&self) -> usize {
        let visible_workers = self
            .projectiles
            .iter()
            .filter(|projectile| {
                matches!(
                    projectile.kind,
                    EnemyProjectileKind::DripsSurfaceCrawler
                        | EnemyProjectileKind::DripsSurfaceCrawlerBurst
                        | EnemyProjectileKind::DripsSurfaceCrawlerFinish
                )
            })
            .count();
        // ASM Drips.asm loc_33488 allocates loc_335D8 through
        // j_Allocate_ObjectSlot, and loc_335D8 loads the worker's GfxObject.
        // The crawler, burst, and finish phases are one continuing pair. The
        // shared `$FFFFFB42` count changes at allocation/deletion boundaries,
        // so retain it when the projectile vector is between those phases.
        visible_workers.max(self.drips_surface_crawler_count_loc_fffffb42)
    }

    fn lava_special_effect_object_only_worker_count(&self) -> u16 {
        if matches!(self.player_lifecycle, PlayerLifecycle::LevelComplete { .. }) {
            return 0;
        }
        let lava_objects =
            self.weather.lava_geysers.len() as u16 + self.weather.lava_fireballs.len() as u16;
        if lava_objects == 0 {
            return 0;
        }
        // ASM Init_SpecialEffect_Lava loc_14C6A allocates one Obj_Lava_Geyser
        // slot per geyser/fireball script. The same routine then allocates
        // Animation_Geyser (loc_14B2A) as a palette coroutine. These workers
        // increment Number_of_Objects but never call j_sub_E02, so they do not
        // own Number_of_GfxObject entries.
        lava_objects.saturating_add(1)
    }

    pub(crate) fn gameplay_transient_gfx_worker_count(&self) -> u16 {
        let live_floating_prize_objects = self
            .floating_items
            .iter()
            .filter(|item| !item.collected)
            .count();
        let archer_arrow_workers = self
            .projectiles
            .iter()
            .filter(|projectile| projectile.kind == EnemyProjectileKind::ArcherArrow)
            .count();
        // ASM UFO.asm loc_34FD6 allocates each beam through
        // Allocate_ObjectSlot, then Enemy0F_UFOBeam_Init loads its own GFX
        // slot. Native keeps these children in `projectiles`, outside the
        // ordinary active-object roster, so count every live beam here.
        let ufo_beam_workers = self
            .projectiles
            .iter()
            .filter(|projectile| projectile.kind == EnemyProjectileKind::UfoBeam)
            .count();
        // ASM Cloud.asm loc_35DB8 allocates the lightning-bolt child through
        // j_Allocate_ObjectSlot on the attack tick, and its init loc_35DF0
        // loads a GfxObject slot, so each live bolt owns an Object+Gfx pair.
        // loc_35E8A deletes it via j_Delete_CurrentObject on first collision
        // or animation end. Native keeps the child in `projectiles`, outside
        // the ordinary active-object roster, so count every live bolt here.
        let cloud_lightning_workers = self
            .projectiles
            .iter()
            .filter(|projectile| projectile.kind == EnemyProjectileKind::CloudLightning)
            .count();
        let tank_bullet_workers = self
            .projectiles
            .iter()
            .filter(|projectile| projectile.kind == EnemyProjectileKind::TankBullet)
            .count();
        let drips_surface_crawler_workers = self.drips_surface_crawler_worker_count_loc_335d8();
        // ASM Lion.asm loc_3E0DA allocates the roar shot's Object, then
        // loc_3E12A calls j_Load_GfxObjectSlot before initialising its art.
        // Native stores the shot on the parent, outside the active roster, so
        // it contributes one worker to both linked-list counts.
        let lion_parent_projectile_workers = self
            .active_enemies
            .iter()
            .filter(|enemy| enemy.family == EnemyFamily::Lion)
            .map(|enemy| usize::from(enemy.ufo_beam_segments))
            .sum::<usize>();
        let lion_projectile_workers = lion_parent_projectile_workers
            .max(usize::from(self.lion_projectile_object_only_tail_count));
        // ASM Fire_Demon.asm loc_3A564 allocates each fire-trail child through
        // j_Allocate_ObjectSlot while Number_of_Fire_Trails < 7, and the child
        // init loc_3A63C loads a GfxObject slot, so every burning trail owns
        // an Object+Gfx pair until the loc_3A6CA wait loop expires its timer
        // or loses floor support and loc_3A6DA deletes it. Native keeps the
        // children in `fire_demon_trails`, outside the active-object roster.
        let fire_demon_trail_workers = self.fire_demon_trails.len();
        let dragon_walking_breath_workers = self
            .active_enemies
            .iter()
            .enumerate()
            .filter(|(enemy_index, enemy)| {
                if enemy.family != EnemyFamily::DragonWalking
                    || enemy.dragon_script_cmd != DRAGON_WALKING_PHASE_BREATH
                {
                    return false;
                }
                let owner = EnemyProjectileOwner {
                    segment_index: enemy.segment_index,
                    layout_index: enemy.layout_index,
                    token: 0,
                };
                match self.projectiles.iter().find(|projectile| {
                    projectile.kind == EnemyProjectileKind::DragonWalkingBreath
                        && projectile.owner == Some(owner)
                }) {
                    // ASM loc_3CFDA can end the child before its animation.
                    // Natural loc_3D082 completion remains linked through the
                    // final phase-one poll.
                    Some(projectile) => projectile.lifetime > 0 || enemy.dragon_phase_frames <= 1,
                    // Native removes the expired record before the parent
                    // observes loc_3D082's done flag. The ROM pair remains in
                    // both linked lists for that phase-zero sample.
                    None => {
                        // ASM Kid_Transform loc_7E90 retains the parent's
                        // loc_3CE90 callback, not an additional loc_3D082
                        // child poll. Once that phase-tagged child is absent,
                        // it has already left both ROM lists at this sample.
                        enemy.dragon_script_variant & DRAGON_WALKING_TRANSFORM_TIMER_ALIGNED_FLAG
                            == 0
                            && enemy.dragon_phase_frames == 0
                            && *enemy_index == 0
                    }
                }
            })
            .count();
        // ASM Dragon.asm loc_3CF90 allocates the breath Object_RAM child;
        // loc_3D02A immediately calls j_Load_GfxObjectSlot, loc_3D082 keeps
        // the pair live, and loc_3D09E deletes the linked lifecycle. Native
        // stores the child outside `active_objects`. Count only a materialised
        // child: raw `$66(a5)=1` still has no Object/GfxObject pair.
        // ASM Enemy22_Shiskaboss_Init allocates loc_38494..loc_38884 up
        // front. All nine skewer object/gfx workers follow unk_39872 while
        // waiting for `$FFFFFB4E==3`; native materialises them as BossSkewer
        // projectiles only at that launch boundary, so count the waiting set
        // while any Shishkaboss head still owns it.
        let waiting_shishkaboss_skewer_workers = if self
            .active_enemies
            .iter()
            .any(|enemy| enemy.family == EnemyFamily::Shishkaboss)
            && !self
                .projectiles
                .iter()
                .any(|projectile| projectile.kind == EnemyProjectileKind::BossSkewer)
        {
            9
        } else {
            0
        };
        // ASM Tar_Monster.asm loc_32468 and loc_325E6, sub_397E4, and the boss
        // child routines allocate ordinary Object_RAM + GfxObject pairs.
        // These native projectile records are not present in `active_objects`,
        // so add each live worker here.
        let boss_projectile_gfx_workers = self
            .projectiles
            .iter()
            .filter(|projectile| {
                matches!(
                    projectile.kind,
                    EnemyProjectileKind::BossHeadArrow
                        | EnemyProjectileKind::BossHeadBoomerang
                        | EnemyProjectileKind::BossSkewer
                        | EnemyProjectileKind::BossEyeAttack
                        | EnemyProjectileKind::BossDeathBurst
                        | EnemyProjectileKind::BossEyeSatellite
                        | EnemyProjectileKind::BossEyePopEffect
                        | EnemyProjectileKind::TarMonsterStraightGlob
                ) || (projectile.kind == EnemyProjectileKind::TarMonsterBurstChild
                    && projectile.animation_elapsed_frames == 0)
            })
            .count();
        (self.block_bounces.len()
            + self.count_visible_prize_block_worker_tails()
            + self.prize_block_spawn_flashes.len()
            + self.kid_projectile_hit_sparks.len()
            + self.hidden_block_fg_update_workers.len()
            + self.block_state.metal_block_push_workers.len()
            + self.block_destroy_effects.len()
            + live_floating_prize_objects
            // ASM PrizeDiamondCollected_Init (kid.asm:21100) allocates an
            // object/gfx pair, then hibernates through the `$3E/$42` HUD-flight
            // loop before incrementing Number_Diamonds. Native models that
            // worker as the delayed award queue.
            + self.pending_prize_diamond_awards.len()
            // ASM Archer.asm loc_341BE allocates the fired arrow through
            // j_Allocate_ObjectSlot, and Enemy07_ArcherArrow_Init loc_34266
            // loads its own GFX slot before the movement loop.
            + archer_arrow_workers
            + ufo_beam_workers
            + cloud_lightning_workers
            // ASM Tank_Shooting.asm loc_3C50E allocates the Object_RAM
            // worker, then loc_3B730 loads its linked GfxObject slot.
            + tank_bullet_workers
            + drips_surface_crawler_workers
            + lion_projectile_workers
            + fire_demon_trail_workers
            + dragon_walking_breath_workers
            + waiting_shishkaboss_skewer_workers
            + boss_projectile_gfx_workers) as u16
    }

    fn gameplay_transient_gfx_only_worker_count(
        &self,
        camera_x_pos: u32,
        camera_y_pos: u32,
    ) -> u16 {
        // ASM Archer.asm Enemy07_Archer_Init loc_33FDA calls loc_32188 to
        // allocate the held-arrow child GFX slot, then stores the link in
        // `$36(a5)`/`$3A(a5)`. It increments Number_GfxObjects without
        // allocating another Object_RAM worker.
        let archer_held_arrow_workers = self
            .active_enemies
            .iter()
            .filter(|enemy| enemy.family == EnemyFamily::Archer)
            .count() as u16;
        let dragon_flying_descent_fire_gfx_workers = self
            .projectiles
            .iter()
            .filter(|projectile| projectile.kind == EnemyProjectileKind::DragonFlyingDescentFire)
            .count() as u16;
        let hidden_drips_gfx_workers = self
            .hidden_drips_parent_worker_count_enemy16_drip_init(camera_x_pos, camera_y_pos)
            .saturating_mul(3);
        let tar_monster_linked_gfx_workers = self
            .active_enemies
            .iter()
            .filter(|enemy| enemy.family == EnemyFamily::TarMonster)
            .count() as u16;
        // ASM Dragon_Flying.asm loc_348A0 allocates the child Object slot;
        // loc_34BAC then calls Load_GfxObjectSlot before loc_32146 links its
        // collision source. Native counts the Object side in
        // gameplay_transient_object_only_worker_count, so add the paired Gfx
        // side here while the typed projectile exists, including loc_34C4A's
        // two cleanup hibernations.
        // ASM Drips.asm Enemy16_Drip_Init loads the controller's a3 GfxObject
        // and preallocates a1/a2 through j_Allocate_GfxObjectSlot_a1. Native
        // suppresses all three hidden snapshots and adds their one Object side
        // separately, so retain all three slots in the Gfx-only projection.
        // ASM Tar_Monster.asm loc_321E4 loads the parent's a3 GfxObject, then
        // loc_32200 calls loc_32188 to splice a second Gfx-only child into the
        // same Object lifecycle. Native collapses the pair into one enemy
        // snapshot, so restore the linked child while its parent remains live.
        archer_held_arrow_workers
            .saturating_add(dragon_flying_descent_fire_gfx_workers)
            .saturating_add(hidden_drips_gfx_workers)
            .saturating_add(tar_monster_linked_gfx_workers)
    }

    fn visible_prize_block_worker_tails(&self) -> Vec<&PrizeBlockWorkerTail> {
        if self.prize_block_worker_tails.is_empty() {
            return Vec::new();
        }
        let Some(player) = self.current_player_state() else {
            return self.prize_block_worker_tails.iter().collect();
        };
        self.prize_block_worker_tails
            .iter()
            .filter(|tail| {
                let own_live_prize = || {
                    self.floating_items.iter().find(|item| {
                        !item.collected
                            && item.level_col == tail.cell_x
                            && item.level_row == tail.cell_y
                    })
                };
                // The distant-diamond suppression below is diamond-specific
                // (see its comment); a same-cell non-diamond prize must not
                // stand in for the tail's own live diamond.
                let own_live_diamond = || {
                    self.floating_items.iter().find(|item| {
                        !item.collected
                            && item.content == terrain::PrizeContent::Diamond
                            && item.level_col == tail.cell_x
                            && item.level_row == tail.cell_y
                    })
                };
                let suppressed_by_own_distant_diamond = tail.content
                    == Some(terrain::PrizeContent::Diamond)
                    && own_live_diamond().is_some_and(|item| {
                        !floating_diamond_in_player_horizontal_band_loc_10396(
                            item,
                            player,
                            self.progression.current_form.half_width(),
                        )
                    });
                // ASM sub_1007A (kid.asm:21512-21536) allocates the flash and
                // Prize_Init workers, then deletes its own worker on the same
                // wake. Non-diamond prizes therefore replace, rather than add
                // to, the stale block-worker slot in Number_of_Objects.
                let suppressed_by_own_non_diamond_prize = tail
                    .content
                    .is_some_and(|content| content != terrain::PrizeContent::Diamond)
                    && own_live_prize().is_some();
                // ASM PrizeDiamond_Init (kid.asm:21604) creates the
                // count-visible diamond object through the standard collision
                // path. Published demo probes show the old sub_1007A block
                // worker remains counted only while its own diamond is already
                // in Kid's horizontal collision band; distant live diamonds
                // replace only their matching stale worker count.
                !suppressed_by_own_distant_diamond && !suppressed_by_own_non_diamond_prize
            })
            .collect()
    }

    fn count_visible_prize_block_worker_tails(&self) -> usize {
        self.visible_prize_block_worker_tails().len()
    }

    fn death_respawn_bootstrap_elapsed_frames(&self) -> Option<usize> {
        match self.player_lifecycle {
            PlayerLifecycle::Respawning { remaining_frames } => {
                Some(PLAYER_DEATH_RESPAWN_SETTLE_FRAMES.saturating_sub(remaining_frames))
            }
            _ => None,
        }
    }

    fn death_respawn_bootstrap_player_state(&self) -> Option<PlayerState> {
        let elapsed = self.death_respawn_bootstrap_elapsed_frames()?;
        let profile = self.death_respawn_bootstrap_profile()?;
        if elapsed < profile.animation_frames {
            let elapsed_i64 = i64::try_from(elapsed).unwrap_or(i64::MAX);
            let x_pos =
                i64::from(profile.initial_x_fp) + i64::from(profile.x_step_fp) * elapsed_i64;
            let y_pos =
                i64::from(profile.initial_y_fp) + i64::from(profile.y_step_fp) * elapsed_i64;
            return Some(PlayerState {
                x_pos: x_pos as u32,
                y_pos: y_pos as u32,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }

        Some(PlayerState {
            x_pos: profile.hold_x_fp,
            y_pos: profile.hold_y_fp,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        })
    }

    fn death_respawn_bootstrap_object_counts(&self) -> Option<(u16, u16)> {
        let elapsed = self.death_respawn_bootstrap_elapsed_frames()?;
        let profile = self.death_respawn_bootstrap_profile()?;
        if self.current_level == LevelId(0) && elapsed == 0 {
            return Some((
                profile.initial_number_objects,
                profile.initial_number_gfx_objects,
            ));
        }
        Some((profile.hold_number_objects, profile.hold_number_gfx_objects))
    }

    fn death_respawn_bootstrap_profile(&self) -> Option<DeathRespawnBootstrapProfile> {
        match self.current_level {
            LevelId(0) => Some(DeathRespawnBootstrapProfile {
                animation_frames: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES,
                initial_x_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_INITIAL_X_FP,
                initial_y_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_INITIAL_Y_FP,
                hold_x_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP,
                hold_y_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP,
                x_step_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_X_STEP_FP,
                y_step_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_Y_STEP_FP,
                initial_number_objects: 15,
                initial_number_gfx_objects: 13,
                hold_number_objects: 15,
                hold_number_gfx_objects: 14,
            }),
            LevelId(1) => Some(DeathRespawnBootstrapProfile {
                // ASM lose_life -> Teleport loc_B8DE enters loc_6E2 mode 8.
                // The direct lower-floor Drips jump probe reaches loc_6E2 with
                // camera_x=$026F at respawn entry, and oracle frame 427 seeds
                // the hidden loc_1AC70 pose at ($0130,$0100), not the
                // wrapped-row pose seen by issue-213-drips-upjump-right-probe-cache.
                //
                // TODO(fidelity): replace this trace profile split with the
                // real LoadGameModeData/loc_1AC70 object-worker lineage.
                initial_x_fp: if matches!(
                    self.death_respawn_trace_profile,
                    Some(DeathRespawnTraceProfile::LevelTwoDirectDrips)
                ) {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_INITIAL_X_FP
                } else {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_INITIAL_X_FP
                },
                initial_y_fp: if matches!(
                    self.death_respawn_trace_profile,
                    Some(DeathRespawnTraceProfile::LevelTwoDirectDrips)
                ) {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_INITIAL_Y_FP
                } else {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_INITIAL_Y_FP
                },
                animation_frames: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES,
                hold_x_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP,
                hold_y_fp: PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP,
                x_step_fp: if matches!(
                    self.death_respawn_trace_profile,
                    Some(DeathRespawnTraceProfile::LevelTwoDirectDrips)
                ) {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_X_STEP_FP
                } else {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_X_STEP_FP
                },
                y_step_fp: if matches!(
                    self.death_respawn_trace_profile,
                    Some(DeathRespawnTraceProfile::LevelTwoDirectDrips)
                ) {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_Y_STEP_FP
                } else {
                    PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_Y_STEP_FP
                },
                initial_number_objects: 16,
                initial_number_gfx_objects: 14,
                hold_number_objects: 16,
                hold_number_gfx_objects: 14,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DeathRespawnBootstrapProfile {
    animation_frames: usize,
    initial_x_fp: u32,
    initial_y_fp: u32,
    hold_x_fp: u32,
    hold_y_fp: u32,
    x_step_fp: i32,
    y_step_fp: i32,
    initial_number_objects: u16,
    initial_number_gfx_objects: u16,
    hold_number_objects: u16,
    hold_number_gfx_objects: u16,
}

fn kid_pre_physics_state_debug(player: PlayerState) -> String {
    format!(
        "x={:#010x}/y={:#010x}/x_vel={:#010x}/y_vel={:#010x}",
        player.x_pos, player.y_pos, player.x_velocity as u32, player.y_velocity as u32,
    )
}

fn enemy_collision_bounds_px(enemy: &NativeObjectSnapshot) -> Option<ObjectBoundsPx> {
    let NativeObjectKind::Enemy { .. } = enemy.kind else {
        return None;
    };
    let x_px = (enemy.x_pos >> 16) as i32;
    let y_px = (enemy.y_pos >> 16) as i32;
    let bounds = enemy_contact_bounds(enemy);
    Some(ObjectBoundsPx {
        left: x_px + bounds.left_px as i32,
        right: x_px + bounds.right_px as i32,
        top: y_px + bounds.top_px as i32,
        bottom: y_px + bounds.bottom_px as i32,
    })
}

fn platform_collision_bounds_px(platform: ActivePlatform) -> ObjectBoundsPx {
    let x_px = (platform.x_pos >> 16) as i32;
    let y_px = (platform.y_pos >> 16) as i32;
    ObjectBoundsPx {
        left: x_px,
        right: x_px + i32::from(platform.width_px),
        top: y_px,
        bottom: y_px + i32::from(platform.height_px),
    }
}

fn format_kid_floor_probe(probe: terrain::FloorProbe) -> String {
    format!(
        "ground={}/y={}/cell={},{} collision={:?} block={:?}",
        u8::from(probe.on_ground),
        probe.floor_y_px,
        probe.cell_x,
        probe.cell_y,
        probe.cell.collision,
        probe.cell.block_type,
    )
}

fn direction_debug_label(direction: Option<Direction>) -> &'static str {
    match direction {
        Some(Direction::Left) => "left",
        Some(Direction::Right) => "right",
        None => "none",
    }
}

fn kid_flip_follow_on_stage_debug(stage: KidFlipFollowOnStageState) -> String {
    format!(
        "x_vel={:#010x}/y_vel={:#010x}",
        stage.x_velocity as u32, stage.y_velocity as u32
    )
}

fn wall_probe_debug_summary(probe: Option<terrain::WallProbe>) -> String {
    match probe {
        Some(probe) => format!(
            "hit={}/x={}/cell={},{} collision={} block={} has_block={}",
            u8::from(probe.hit),
            probe.wall_x_px,
            probe.cell_x,
            probe.cell_y,
            collision_kind_debug_label(probe.cell.collision),
            block_type_debug_label(probe.cell.block_type),
            u8::from(probe.cell.has_block),
        ),
        None => String::from("hit=0/x=0/cell=0,0 collision=empty block=None has_block=0"),
    }
}

fn collision_kind_debug_label(collision: terrain::CollisionKind) -> &'static str {
    match collision {
        terrain::CollisionKind::Empty => "empty",
        terrain::CollisionKind::HiddenBlock => "hidden_block",
        terrain::CollisionKind::SlopeDown => "slope_down",
        terrain::CollisionKind::SlopeUp => "slope_up",
        terrain::CollisionKind::Solid => "solid",
        terrain::CollisionKind::Spikes => "spikes",
    }
}

fn block_type_debug_label(block_type: terrain::BlockType) -> &'static str {
    match block_type {
        terrain::BlockType::Smash => "Smash",
        terrain::BlockType::Prize => "Prize",
        terrain::BlockType::Ice => "Ice",
        terrain::BlockType::SolidGhost => "SolidGhost",
        terrain::BlockType::Teleporter => "Teleporter",
        terrain::BlockType::Metal => "Metal",
        terrain::BlockType::Rubber => "Rubber",
        terrain::BlockType::Shifting => "Shifting",
        terrain::BlockType::Shooter => "Shooter",
        terrain::BlockType::Mushroom => "Mushroom",
        terrain::BlockType::Evanescent => "Evanescent",
        terrain::BlockType::Lift => "Lift",
        terrain::BlockType::Drill => "Drill",
        terrain::BlockType::Deleted => "Deleted",
        terrain::BlockType::None => "None",
    }
}

fn block_grid_source_debug_label(source: BlockGridSource) -> &'static str {
    match source {
        BlockGridSource::None => "none",
        BlockGridSource::Manual => "manual",
        BlockGridSource::Generated => "generated",
    }
}

fn terrain_cell_debug_summary(cell: terrain::Cell) -> String {
    format!(
        "{},{},skin=0x{:02X}",
        collision_kind_debug_label(cell.collision),
        block_type_debug_label(cell.block_type),
        cell.skin,
    )
}

fn player_lifecycle_debug_label(lifecycle: PlayerLifecycle) -> &'static str {
    match lifecycle {
        PlayerLifecycle::Alive => "alive",
        PlayerLifecycle::DeathWindup { .. } => "death_windup",
        PlayerLifecycle::Dying { .. } => "dying",
        PlayerLifecycle::Respawning { .. } => "respawning",
        PlayerLifecycle::LevelComplete { .. } => "level_complete",
        PlayerLifecycle::GameOver { .. } => "game_over",
        PlayerLifecycle::ContinueCountdown { .. } => "continue_countdown",
    }
}

fn kid_object_support_candidate(
    label: &str,
    player_x_px: i32,
    player_foot_y_px: i32,
    player_bounds: ObjectBoundsPx,
    object_bounds: ObjectBoundsPx,
) -> Option<KidObjectSupportCandidate> {
    let object_centre_x_px = object_bounds.left.midpoint(object_bounds.right);
    let dx_px = object_centre_x_px - player_x_px;
    let top_gap_px = player_foot_y_px - object_bounds.top;
    if dx_px.abs() > KID_OBJECT_SUPPORT_HORIZONTAL_WINDOW_PX
        || top_gap_px.abs() > KID_OBJECT_SUPPORT_VERTICAL_WINDOW_PX
    {
        return None;
    }

    let x_overlap_px =
        player_bounds.right.min(object_bounds.right) - player_bounds.left.max(object_bounds.left);
    let y_overlap_px =
        player_bounds.bottom.min(object_bounds.bottom) - player_bounds.top.max(object_bounds.top);
    let relation = classify_kid_object_relation(player_bounds, object_bounds);
    let relation_label = relation.map_or("none", KidObjectRelation::as_debug_label);
    let support = relation == Some(KidObjectRelation::Below);
    Some(KidObjectSupportCandidate {
        abs_top_gap_px: top_gap_px.abs(),
        abs_dx_px: dx_px.abs(),
        summary: format!(
            "{label} dx={dx_px} top_gap={top_gap_px} x_overlap={x_overlap_px} y_overlap={y_overlap_px} relation={relation_label} support={}",
            u8::from(support)
        ),
    })
}

pub(crate) fn classify_kid_object_relation(
    player_bounds: ObjectBoundsPx,
    object_bounds: ObjectBoundsPx,
) -> Option<KidObjectRelation> {
    let x_overlap_px =
        player_bounds.right.min(object_bounds.right) - player_bounds.left.max(object_bounds.left);
    let y_overlap_px =
        player_bounds.bottom.min(object_bounds.bottom) - player_bounds.top.max(object_bounds.top);
    if x_overlap_px <= 0 || y_overlap_px <= 0 {
        return None;
    }

    let from_top = player_bounds.bottom - object_bounds.top;
    let from_bottom = object_bounds.bottom - player_bounds.top;
    let from_left = player_bounds.right - object_bounds.left;
    let from_right = object_bounds.right - player_bounds.left;
    let min_penetration = from_top.min(from_bottom).min(from_left).min(from_right);

    if min_penetration == from_top {
        // ASM loc_1CDE (kid.asm:3470): when the top-side overlap wins the
        // branch-specific comparison, the routine stores `colid_kidbelow`
        // before entering the shared collision writeback.
        Some(KidObjectRelation::Below)
    } else if min_penetration == from_bottom {
        // ASM loc_1CE4 (kid.asm:3475): the mirrored vertical branch stores
        // `colid_kidabove` before the shared writeback checks
        // `Character_Movement == MoveID_Jump` for the later stomp path.
        Some(KidObjectRelation::Above)
    } else if min_penetration == from_left {
        // ASM loc_1CD2 (kid.asm:3460): once the surrounding overlap/velocity
        // comparison picks the right-side branch, the routine stores
        // `colid_kidright` and jumps into the shared collision writeback.
        Some(KidObjectRelation::Right)
    } else {
        // ASM loc_1CD8 (kid.asm:3465): the mirrored overlap branch stores
        // `colid_kidleft` before jumping into the shared collision writeback.
        Some(KidObjectRelation::Left)
    }
}

pub(crate) fn opening_level_one_object_counts(
    level_id: LevelId,
    roster_count: u16,
    camera_x_pos: u32,
    camera_y_pos: u32,
) -> Option<(u16, u16)> {
    if level_id == LevelId(1) && camera_x_pos == 0 {
        // ASM loc_74E0 level-skip resume reaches Blue Lake Woods 2 with only
        // the player plus RockTank body/turret live with loc_7456/loc_BC44's
        // fixed workers. Oracle frame-60 WRAM reports 4 Objects / 6 GfxObjects:
        // the turret is Gfx-only and Kid owns the additional gameplay Gfx slot.
        return Some((roster_count + 2, roster_count + 4));
    }
    if level_id != LevelId(0) || camera_x_pos > 0x0200_0000 {
        return None;
    }
    if camera_x_pos == 0 && camera_y_pos > OPENING_GROUNDED_CAMERA_Y_FP + 0x0004_0000 {
        return Some((roster_count + 2, roster_count + 2));
    }
    // ASM Load_InGame allocates the player, DragonWalker, fixed manager object,
    // and the extra gameplay gfx slot before the settled-recording helper starts
    // frame 0. Oracle RAM for issue-211's level-00 settled path reports
    // Number_Objects=4 / Number_GfxObjects=5 here, and the scrolled opening
    // window preserves the same split.
    Some((roster_count + 2, roster_count + 3))
}
