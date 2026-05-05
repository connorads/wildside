use super::*;

mod chunk_07 {
    use super::*;

    #[test]
    fn boss_death_burst_timeout_ground_gate_loc_3724a_waits_until_grounded_then_spawns_final_helper()
     {
        assert_eq!(
            boss_death_burst_timeout_ground_gate_loc_3724A(false, 0, 0x0044, 0x0046),
            BossDeathBurstTimeoutGroundGateLoc3724A {
                final_spawn: None,
                finish_animation_label: None,
            },
            "ASM loc_3724A should branch back to loc_3720A while `$18(a3)` is zero, allocating no helper object",
        );

        assert_eq!(
            boss_death_burst_timeout_ground_gate_loc_3724A(true, 0, 0x0044, 0x0046),
            BossDeathBurstTimeoutGroundGateLoc3724A {
                final_spawn: Some(BossDeathBurstFinalSpawnLoc3724A {
                    worker_label: "sub_37002",
                    x_pos: 0x003F,
                    y_pos: 0x004E,
                }),
                finish_animation_label: Some("stru_37BFC"),
            },
            "ASM loc_3724A should spawn `sub_37002` at `x_pos-5,y_pos+8` when `x_direction(a3)` is clear before running sub_3728C",
        );

        assert_eq!(
            boss_death_burst_timeout_ground_gate_loc_3724A(true, 0xFF, 0x0044, 0x0046)
                .final_spawn
                .expect("grounded loc_3724A should allocate a final helper")
                .x_pos,
            0x0049,
            "ASM loc_37280 should use `x_pos+5` for any non-zero `x_direction(a3)` after loc_3724A allocates the helper",
        );
    }

    #[test]
    fn boss_death_burst_finish_sub_3728c_plays_the_final_anim_then_deletes() {
        let finish = boss_death_burst_finish_sub_3728C();

        assert_eq!(
            finish.animation_label, "stru_37BFC",
            "ASM sub_3728C should switch the burst sprite to `stru_37BFC` before the final hibernate",
        );
        assert!(
            finish.hibernate_until_anim_finished,
            "ASM sub_3728C should call `Hibernate_UntilAnimFinished` before deleting the current object",
        );
        assert!(
            finish.delete_current_object,
            "ASM sub_3728C should delete the current object immediately after the finish animation ends",
        );
    }

    #[test]
    fn boss_eye_pop_effect_init_sub_3729e_seeds_the_tracking_sprite_and_counter() {
        let init = boss_eye_pop_effect_init_sub_3729E(0x0044, 0x0046);

        assert_eq!(
            init.gfx_slot_base, 0x0101_0002,
            "ASM sub_3729E should allocate gfx slot `$01010002` for the boss eye pop effect",
        );
        assert_eq!(
            init.object_meta, 0x0020,
            "ASM sub_3729E should seed `object_meta(a3)` with `$20` before `loc_32146`",
        );
        assert!(
            init.display_enabled && init.is_moved,
            "ASM sub_3729E should set `$13(a3)` and `is_moved(a3)` on the spawned eye effect",
        );
        assert_eq!(
            init.x_direction, 0,
            "ASM sub_3729E should clear `x_direction(a3)` before the entry animation starts",
        );
        assert_eq!(
            init.sprite_mapping_label, "LnkTo_unk_C8460-Data_Index",
            "ASM sub_3729E should use the shared burst sprite mapping for the eye effect",
        );
        assert_eq!(
            (init.x_pos, init.y_pos, init.sprite_word_y_adjust),
            (0x0044, 0x0046, 1),
            "ASM sub_3729E should copy `$44/$46(a5)` into the sprite and add 1 to `$A(a3)`",
        );
        assert_eq!(
            init.active_eye_delta, 1,
            "ASM sub_3729E should increment `$FFFFFB4F` before the eye effect starts tracking",
        );
        assert_eq!(
            (
                init.entry_animation_label,
                init.entry_hibernate_until_anim_finished
            ),
            ("stru_37BEA", true),
            "ASM sub_3729E should play `stru_37BEA` and hibernate until that entry animation finishes",
        );
        assert_eq!(
            init.chase_frames, 0x0064,
            "ASM loc_372F6 should run exactly `#$64` hibernated tracking frames before finishing",
        );
    }

    #[test]
    fn boss_eye_pop_effect_face_kid_sub_3729e_points_left_when_not_right_of_kid() {
        assert_eq!(
            boss_eye_pop_effect_face_kid_sub_3729E(0x0040, 0x0060),
            0xFF,
            "ASM loc_37300 should set `x_direction(a3)=$FF` when the effect is left of Kid",
        );
        assert_eq!(
            boss_eye_pop_effect_face_kid_sub_3729E(0x0060, 0x0060),
            0xFF,
            "ASM loc_37300 should keep `x_direction(a3)=$FF` when the effect shares Kid's X position",
        );
        assert_eq!(
            boss_eye_pop_effect_face_kid_sub_3729E(0x0061, 0x0060),
            0,
            "ASM loc_37316 should clear `x_direction(a3)` only when the effect is strictly right of Kid",
        );
    }

    #[test]
    fn boss_eye_pop_effect_finish_sub_3729e_plays_the_final_anim_and_decrements_the_counter() {
        let finish = boss_eye_pop_effect_finish_sub_3729E();

        assert_eq!(
            finish.animation_label, "stru_37BFC",
            "ASM loc_3731C should switch the eye effect to `stru_37BFC` before its final hibernate",
        );
        assert!(
            finish.hibernate_until_anim_finished,
            "ASM loc_3731C should call `Hibernate_UntilAnimFinished` before deleting the eye effect",
        );
        assert_eq!(
            finish.active_eye_delta, -1,
            "ASM loc_3731C should decrement `$FFFFFB4F` after the final animation completes",
        );
        assert!(
            finish.delete_current_object,
            "ASM loc_3731C should delete the current object after the final animation and counter decrement",
        );
    }

    #[test]
    fn boss_eye_attack_init_sub_37332_seeds_the_boss_projectile_and_launch_velocity() {
        let left = boss_eye_attack_init_sub_37332(0x0044, 0x0046, 0x0060);
        let right = boss_eye_attack_init_sub_37332(0x0060, 0x0046, 0x0044);

        assert_eq!(
            left.gfx_slot_base, 0x0101_0002,
            "ASM sub_37332 should allocate gfx slot `$01010002` for the boss eye projectile",
        );
        assert_eq!(
            left.object_meta, 0x0021,
            "ASM sub_37332 should seed `object_meta(a3)` with `$21` before `loc_32146`",
        );
        assert!(
            left.has_kid_collision && left.display_enabled && left.is_moved,
            "ASM sub_37332 should set `has_kid_collision(a3)`, `$13(a3)`, and `is_moved(a3)` on the projectile",
        );
        assert_eq!(
            left.sprite_mapping_label, "LnkTo_unk_C8488-Data_Index",
            "ASM sub_37332 should start from the first boss-eye projectile sprite mapping",
        );
        assert_eq!(
            (left.x_pos, left.y_pos, left.sprite_word_y_adjust),
            (0x0044, 0x0046, 2),
            "ASM sub_37332 should copy `$44/$46(a5)` into the projectile and add 2 to `$A(a3)`",
        );
        assert_eq!(
            left.initial_y_velocity, -4,
            "ASM sub_37332 should seed `y_vel(a3)` with `$FFFC` before the first frame",
        );
        assert_eq!(
            left.initial_x_velocity, -0x0000_7FFF,
            "ASM loc_37394 should launch left with `$FFFF8001` when Kid is to the right",
        );
        assert_eq!(
            right.initial_x_velocity, 0x0000_7FFF,
            "ASM sub_37332 should launch right with `$7FFF` when Kid is at or left of the projectile",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_sub_37332_matches_sub_37af0_direction_buckets_and_diamond_override() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0040, 0x0030),
            "LnkTo_unk_C8488-Data_Index",
            "ASM sub_37AF0 should choose mapping index 0 when Kid is above the projectile within the centred X bucket",
        );
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0030, 0x0040),
            "LnkTo_unk_C8498-Data_Index",
            "ASM sub_37AF0 should choose mapping index 2 when Kid is in the middle Y band and left of `x_pos-8`",
        );
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0050, 0x0040),
            "LnkTo_unk_C84B8-Data_Index",
            "ASM sub_37AF0 should choose mapping index 6 for the remaining middle-band bucket",
        );
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0050, 0x0050),
            "LnkTo_unk_C84B0-Data_Index",
            "ASM sub_37AF0 should choose mapping index 5 when Kid is below and right of the projectile",
        );
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(true, 0x00, 0x0040, 0x0040, 0x0050, 0x0050),
            "$0E30",
            "ASM loc_37B06 should override the directional sprite with `$E30` when diamond power is active and bit 3 is clear",
        );
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(true, 0x08, 0x0040, 0x0040, 0x0050, 0x0050),
            "$0E20",
            "ASM sub_37AF0 should override the directional sprite with `$E20` when diamond power is active and bit 3 is set",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b0c_uses_signed_word_vertical_compare() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x7FFE, 0x0040, 0x7FFF),
            "LnkTo_unk_C84A8-Data_Index",
            "ASM loc_37B0C should use signed `cmp.w`/`bgt` for Kid Y against `y_pos(a3)+4`, so `$7FFF > $8002` selects the lower centred bucket",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b42_uses_middle_left_bucket() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x8006, 0x0040, 0x8000, 0x0040),
            "LnkTo_unk_C8498-Data_Index",
            "ASM loc_37B42 should select index 2 after signed Kid X is less than `x_pos(a3)-8` in the middle Y band",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b46_uses_lower_centred_bucket() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0040, 0x0045),
            "LnkTo_unk_C84A8-Data_Index",
            "ASM loc_37B46 should select index 4 when Kid is below `y_pos(a3)+4` and within the signed +/-4 X window",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b5a_uses_lower_left_bucket() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x003B, 0x0045),
            "LnkTo_unk_C84A0-Data_Index",
            "ASM loc_37B5A should choose index 3 when lower-band Kid X is left of `x_pos(a3)-4`",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b5e_uses_lower_right_bucket() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0045, 0x0045),
            "LnkTo_unk_C84B0-Data_Index",
            "ASM loc_37B5E should choose index 5 when lower-band Kid X is right of `x_pos(a3)+4`",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b62_uses_upper_centred_bucket() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0040, 0x003B),
            "LnkTo_unk_C8488-Data_Index",
            "ASM loc_37B62 should choose index 0 when Kid is above `y_pos(a3)-4` and horizontally centred",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b76_uses_upper_left_bucket() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x003B, 0x003B),
            "LnkTo_unk_C8490-Data_Index",
            "ASM loc_37B76 should choose index 1 when upper-band Kid X is left of `x_pos(a3)-4`",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b7a_uses_upper_right_bucket() {
        assert_eq!(
            boss_eye_attack_sprite_sub_37332(false, 0x00, 0x0040, 0x0040, 0x0045, 0x003B),
            "LnkTo_unk_C84C0-Data_Index",
            "ASM loc_37B7A should choose index 7 when upper-band Kid X is right of `x_pos(a3)+4`",
        );
    }

    #[test]
    fn boss_eye_attack_sprite_loc_37b7c_loads_mapping_from_selected_index() {
        assert_eq!(
            boss_eye_attack_sprite_mapping_loc_37B7C(7),
            "LnkTo_unk_C84C0-Data_Index",
            "ASM loc_37B7C should double the selected index and load the matching `off_37AE0` word",
        );
    }

    #[test]
    fn boss_eye_attack_step_sub_37332_applies_gravity_refreshes_sprite_and_deletes_below_floor() {
        let airborne = boss_eye_attack_step_sub_37332(
            0,
            BossEyeAttackFrameInputsSub37332 {
                effect_x: 0x0040,
                effect_y: 0x0040,
                kid_x: 0x0050,
                kid_y: 0x0050,
                diamond_power_active: false,
                v_int_counter_fff809: 0,
                level_height_pixels: 0x0050,
            },
        );
        assert_eq!(
            airborne.next_y_velocity, 0x0000_0BB8,
            "ASM loc_3739C should add `$BB8` to `y_vel(a3)` after each hibernated frame",
        );
        assert_eq!(
            airborne.sprite_mapping, "LnkTo_unk_C84B0-Data_Index",
            "ASM loc_3739C should refresh `addroffset_sprite(a3)` from sub_37AF0 on every frame",
        );
        assert!(
            !airborne.delete_current_object,
            "ASM loc_373BE should keep the projectile alive while `y_pos(a3)` is within the level height",
        );

        let below_floor = boss_eye_attack_step_sub_37332(
            0,
            BossEyeAttackFrameInputsSub37332 {
                effect_x: 0x0040,
                effect_y: 0x0051,
                kid_x: 0x0050,
                kid_y: 0x0050,
                diamond_power_active: false,
                v_int_counter_fff809: 0,
                level_height_pixels: 0x0050,
            },
        );
        assert!(
            below_floor.delete_current_object,
            "ASM loc_373BE should delete the projectile once `y_pos(a3)` is strictly greater than `Level_height_pixels`",
        );
    }

    #[test]
    fn boss_eye_refresh_attack_sprites_sub_37abc_updates_each_live_slot_and_skips_nulls() {
        let refreshed = boss_eye_refresh_attack_sprites_sub_37ABC(
            [
                Some(BossEyeSpriteRefreshSlotSub37ABC {
                    x_pos: 0x0040,
                    y_pos: 0x0040,
                    sprite_mapping: "stale",
                }),
                None,
                Some(BossEyeSpriteRefreshSlotSub37ABC {
                    x_pos: 0x0060,
                    y_pos: 0x0030,
                    sprite_mapping: "stale",
                }),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(BossEyeSpriteRefreshSlotSub37ABC {
                    x_pos: 0x0040,
                    y_pos: 0x0060,
                    sprite_mapping: "stale",
                }),
            ],
            false,
            0,
            0x0050,
            0x0050,
        );

        assert_eq!(
            refreshed[0],
            Some(BossEyeSpriteRefreshSlotSub37ABC {
                x_pos: 0x0040,
                y_pos: 0x0040,
                sprite_mapping: "LnkTo_unk_C84B0-Data_Index",
            }),
            "ASM loc_37AC6 should refresh the first live `$FFFFFB7E` worker with sub_37AF0's directional mapping",
        );
        assert_eq!(
            refreshed[1], None,
            "ASM loc_37AD4 should skip null longwords but keep scanning the fixed `$FFFFFB7E` worker table",
        );
        assert_eq!(
            refreshed[2],
            Some(BossEyeSpriteRefreshSlotSub37ABC {
                x_pos: 0x0060,
                y_pos: 0x0030,
                sprite_mapping: "LnkTo_unk_C84A0-Data_Index",
            }),
            "ASM sub_37ABC should keep iterating through later live workers before the `dbf` terminates",
        );
        assert_eq!(
            refreshed[11],
            Some(BossEyeSpriteRefreshSlotSub37ABC {
                x_pos: 0x0040,
                y_pos: 0x0060,
                sprite_mapping: "LnkTo_unk_C84C0-Data_Index",
            }),
            "ASM sub_37ABC should still refresh the twelfth worker loaded from the final loop iteration",
        );
    }

    #[test]
    fn boss_eye_refresh_attack_sprites_loc_37ac6_refreshes_live_worker_mapping() {
        let mut slots = [None; 12];
        slots[0] = Some(BossEyeSpriteRefreshSlotSub37ABC {
            x_pos: 0x0040,
            y_pos: 0x0040,
            sprite_mapping: "stale",
        });

        let refreshed = boss_eye_refresh_attack_sprites_sub_37ABC(slots, false, 0, 0x0050, 0x0050);

        assert_eq!(
            refreshed[0].map(|slot| slot.sprite_mapping),
            Some("LnkTo_unk_C84B0-Data_Index"),
            "ASM loc_37AC6 should load a live worker pointer and store the returned sprite mapping",
        );
    }

    #[test]
    fn boss_eye_detach_step_sub_393b2_plays_the_pop_sound_and_raises_sprite_order() {
        let result = boss_eye_detach_step_sub_393B2(0x0012);

        assert_eq!(
            result.sound_effect_label, "sfx_Boss_eye_pops",
            "ASM sub_393B2 should queue `sfx_Boss_eye_pops` before detaching the worker",
        );
        assert_eq!(
            result.next_sprite_word_y_adjust, 0x0016,
            "ASM sub_393B2 should add 4 to `$A(a3)` before the worker starts falling",
        );
        assert!(
            result.rerank_gfx_object,
            "ASM sub_393B2 should call sub_FF6 after changing `$A(a3)` so the gfx-object list is resorted",
        );
    }

    #[test]
    fn boss_eye_orbit_placement_sub_393c8_uses_the_right_side_table_and_leaves_order_unchanged_when_frame_matches()
     {
        let result = boss_eye_orbit_placement_sub_393C8(0x0200, 0x0100, 0x0EE2, 0, 3, 3, 0x0010);

        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x01EC, 0x00DB),
            "ASM sub_393C8 should use `word_39416` when `d2` is clear and subtract `$25` from the parent Y position",
        );
        assert_eq!(
            result.next_sprite_word_y_adjust, 0x0010,
            "ASM sub_393C8 should leave `$A(a3)` untouched when `d0` already matches `$16(a2)`",
        );
        assert!(
            !result.rerank_gfx_object,
            "ASM sub_393C8 should skip sub_FF6 when the parent frame byte has not changed",
        );
    }

    #[test]
    fn boss_eye_orbit_placement_sub_393c8_uses_the_left_table_and_increments_order_after_zero_frame()
     {
        let result = boss_eye_orbit_placement_sub_393C8(0x0200, 0x0100, 0x0EE4, 1, 0, 2, 0x0010);

        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x020D, 0x00DB),
            "ASM sub_393C8 should use `word_3940C` when `d2` is set before adding the parent X position",
        );
        assert_eq!(
            result.next_sprite_word_y_adjust, 0x0011,
            "ASM loc_39402 should add 1 to `$A(a3)` when the previous frame byte in `d0` was zero",
        );
        assert!(
            result.rerank_gfx_object,
            "ASM loc_39406 should call sub_FF6 after the zero-to-nonzero frame transition changes `$A(a3)`",
        );
    }

    #[test]
    fn boss_eye_orbit_placement_sub_393c8_decrements_order_after_nonzero_frame_change() {
        let result = boss_eye_orbit_placement_sub_393C8(0x0200, 0x0100, 0x0EE8, 0, 1, 4, 0x0010);

        assert_eq!(
            result.next_sprite_word_y_adjust, 0x000F,
            "ASM sub_393C8 should subtract 1 from `$A(a3)` when `d0` was nonzero and `$16(a2)` changes",
        );
        assert!(
            result.rerank_gfx_object,
            "ASM sub_393C8 should rerun sub_FF6 after the nonzero frame change updates `$A(a3)`",
        );
    }

    #[test]
    fn boss_eye_counter_orbit_placement_sub_39420_uses_the_right_side_table_and_leaves_order_unchanged_when_frame_matches()
     {
        let result =
            boss_eye_counter_orbit_placement_sub_39420(0x0200, 0x0100, 0x0EE2, 0, 3, 3, 0x0010);

        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0200, 0x00DB),
            "ASM sub_39420 should use `word_3946E` when `d2` is clear and keep the shared `-$25` Y offset",
        );
        assert_eq!(
            result.next_sprite_word_y_adjust, 0x0010,
            "ASM sub_39420 should leave `$A(a3)` untouched when `d0` already matches `$16(a2)`",
        );
        assert!(
            !result.rerank_gfx_object,
            "ASM sub_39420 should skip sub_FF6 when the parent frame byte has not changed",
        );
    }

    #[test]
    fn boss_eye_counter_orbit_placement_sub_39420_uses_the_left_side_table_and_decrements_order_after_zero_frame()
     {
        let result =
            boss_eye_counter_orbit_placement_sub_39420(0x0200, 0x0100, 0x0EE4, 1, 0, 2, 0x0010);

        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0218, 0x00DB),
            "ASM sub_39420 should use `word_39464` when `d2` is set before adding the parent X position",
        );
        assert_eq!(
            result.next_sprite_word_y_adjust, 0x000F,
            "ASM loc_3945E should subtract 1 from `$A(a3)` when the previous frame byte in `d0` was zero",
        );
        assert!(
            result.rerank_gfx_object,
            "ASM loc_3945E should call sub_FF6 after the zero-to-nonzero frame transition changes `$A(a3)`",
        );
    }

    #[test]
    fn boss_eye_counter_orbit_placement_sub_39420_loc_3945a_increments_order_after_nonzero_frame_change()
     {
        // ASM loc_3945A (kid.asm:42030): addq.w #1,$A(a3) — when d0 is nonzero
        // (previous_parent_frame != 0) and the parent's $16(a2) frame byte has
        // changed, the sprite word is incremented before falling through to
        // loc_3945E → jsr (j_sub_FF6).w for reranking.
        let result =
            boss_eye_counter_orbit_placement_sub_39420(0x0200, 0x0100, 0x0EE8, 0, 1, 4, 0x0010);

        assert_eq!(
            result.next_sprite_word_y_adjust, 0x0011,
            "ASM loc_3945A: addq.w #1,$A(a3) should add 1 when `d0` was nonzero and `$16(a2)` changes",
        );
        assert!(
            result.rerank_gfx_object,
            "ASM loc_3945A: jsr (j_sub_FF6).w must fire after the addq.w frame-change path",
        );
    }

    #[test]
    fn shishka_bagel_head_1_init_loc_39478_shishkaboss_variant() {
        // ASM loc_39478 (kid.asm:42057): when d2==0 (ShishkaBoss), beq.s loc_39494
        // leaves d0=$22. loc_39494 stores $22 into object_meta, bset #7, jsr loc_32146,
        // then branches to loc_394E2 → ShishkaBoss_HitPointsPerHead=$14 and unk_39872.
        let result = shishka_bagel_head_1_init_loc_39478(0x0280, 0x0160, 0);

        assert_eq!(
            (
                result.gfx_slot_base,
                result.priority,
                result.object_meta_base
            ),
            (0x0104_0002, 0, 0x0022),
            "ASM loc_39478: gfx slot $01040002, priority 0, object_meta $22 for ShishkaBoss",
        );
        assert!(
            result.sets_object_meta_bit_f,
            "ASM loc_39494: bset #7,object_meta(a3) must set bit F of the stored word",
        );
        assert!(result.runs_common_boss_init_loc_32146);
        assert_eq!(
            (
                result.has_kid_collision,
                result.display_enabled,
                result.is_moved
            ),
            (false, true, true),
            "ASM loc_39494: sf has_kid_collision, st $13, st is_moved",
        );
        assert_eq!(
            (result.hit_points, result.movement_script_label),
            (0x0014, "unk_39872"),
            "ASM loc_394E2: ShishkaBoss_HitPointsPerHead=$14, movement script unk_39872",
        );
        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0280, 0x0160),
            "ASM loc_39494: anchor $44/$46 copied to x_pos/y_pos",
        );
        assert_eq!(
            (result.cached_worker_slot_label, result.sprite_mapping_label),
            ("$FFFFFB7A", "LnkTo_unk_C8600-Data_Index"),
            "ASM loc_39494: cache a3 at $FFFFFB7A, sprite LnkTo_unk_C8600",
        );
        assert_eq!(
            (result.delay_counter_d0, result.script_index_d1),
            (1, 0),
            "ASM loc_39478: moveq #1,d0 and move.w #0,d1 seed the movement script",
        );
    }

    #[test]
    fn shishka_bagel_head_1_init_loc_39478_bagel_variant() {
        // ASM loc_39478 (kid.asm:42057): when d2!=0 (BagelBrothers), d0=$24 is loaded
        // before falling through to loc_39494. Then branches to unk_39AB2 with
        // BagelBrothers_HitPointsPerHead=$1E.
        let result = shishka_bagel_head_1_init_loc_39478(0x0300, 0x0180, 1);

        assert_eq!(
            result.object_meta_base, 0x0024,
            "ASM loc_39478: move.w #$24,d0 when d2!=0 (BagelBrothers)",
        );
        assert_eq!(
            (result.hit_points, result.movement_script_label),
            (0x001E, "unk_39AB2"),
            "ASM loc_39478: BagelBrothers_HitPointsPerHead=$1E, movement script unk_39AB2",
        );
        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0300, 0x0180),
            "ASM loc_39494: anchor positions forwarded from parent's $44/$46",
        );
    }

    #[test]
    fn shishka_bagel_head_1_wait_loop_loc_394e8_shishka_zero_status() {
        // ASM loc_394E8 (kid.asm:42091): d2==0 (ShishkaBoss) → bsr.w sub_398BC.
        // ASM loc_394FA (kid.asm:42102): d7==0 → beq.s loc_39506 (activation window).
        let result = shishka_bagel_head_1_wait_loop_loc_394E8(0, 0);

        assert!(
            result.hibernates_one_frame,
            "ASM loc_394E8: jsr (j_Hibernate_Object_1Frame).w before movement check",
        );
        assert!(
            result.runs_shishka_movement_sub_398bc,
            "ASM loc_394E8: tst.w d2; bne.s loc_394F6 — d2==0 calls sub_398BC",
        );
        assert!(
            !result.runs_bagel_movement_sub_39bf8,
            "ASM loc_394E8: ShishkaBoss path must not call sub_39BF8",
        );
        assert_eq!(
            result.cached_collision_word_fffffb72, 0,
            "ASM loc_394FA: move.w d7,($FFFFFB72).w stores zero status",
        );
        assert_eq!(
            result.next,
            ShishkaBagelHead1WaitNextLoc394E8::CheckKidWindowLoc39506,
            "ASM loc_394FA: tst.w d7; beq.s loc_39506 when d7==0",
        );
    }

    #[test]
    fn shishka_bagel_head_1_wait_loop_loc_394f6_bagel_zero_status() {
        // ASM loc_394E8 (kid.asm:42091): d2!=0 (BagelBrothers) → bne.s loc_394F6.
        // ASM loc_394F6 (kid.asm:42099): bsr.w sub_39BF8.
        // ASM loc_394FA (kid.asm:42102): d7==0 → beq.s loc_39506.
        let result = shishka_bagel_head_1_wait_loop_loc_394E8(1, 0);

        assert!(
            !result.runs_shishka_movement_sub_398bc,
            "ASM loc_394F6: BagelBrothers path must not call sub_398BC",
        );
        assert!(
            result.runs_bagel_movement_sub_39bf8,
            "ASM loc_394F6: bsr.w sub_39BF8 for BagelBrothers variant",
        );
        assert_eq!(
            result.next,
            ShishkaBagelHead1WaitNextLoc394E8::CheckKidWindowLoc39506,
            "ASM loc_394FA: d7==0 falls through to activation window check",
        );
    }

    #[test]
    fn shishka_bagel_head_1_wait_loop_loc_394fa_nonzero_collision_exit() {
        // ASM loc_394FA (kid.asm:42102): move.w d7,($FFFFFB72).w; tst.w d7;
        // bne → bra.w loc_39EBE (shared collision exit) when d7 nonzero.
        let result = shishka_bagel_head_1_wait_loop_loc_394E8(0, 0x0003);

        assert_eq!(
            result.cached_collision_word_fffffb72, 0x0003,
            "ASM loc_394FA: move.w d7,($FFFFFB72).w must store nonzero status",
        );
        assert_eq!(
            result.next,
            ShishkaBagelHead1WaitNextLoc394E8::SharedCollisionExitLoc39EBE,
            "ASM loc_394FA: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_1_activation_loc_39506_waits_outside_signed_vertical_window() {
        // ASM loc_39506 (kid.asm:42109): Kid Y < head_y-$40 or Kid Y > head_y
        // → branch back to loc_394E8 wait loop
        let above =
            shishka_bagel_head_1_activation_window_loc_39506(0x0090, 0x003F, 0x0080, 0x0080);
        let below =
            shishka_bagel_head_1_activation_window_loc_39506(0x0090, 0x0081, 0x0080, 0x0080);

        assert_eq!(
            above.next,
            ShishkaBagelHead1ActivationNextLoc39506::WaitLoopLoc394E8,
            "ASM loc_39506: blt.s loc_394E8 when signed Kid Y < head_y-$40",
        );
        assert_eq!(
            below.next,
            ShishkaBagelHead1ActivationNextLoc39506::WaitLoopLoc394E8,
            "ASM loc_39506: bgt.s loc_394E8 when signed Kid Y > head_y",
        );
        assert!(!above.kid_within_vertical_window);
        assert!(!below.kid_within_vertical_window);
        assert_eq!((above.x_direction, below.x_direction), (None, None));
    }

    #[test]
    fn shishka_bagel_head_1_activation_loc_39506_proceeds_inside_signed_vertical_window() {
        // ASM loc_39506: inclusive bounds [head_y-$40, head_y] → direction check
        let lower_edge =
            shishka_bagel_head_1_activation_window_loc_39506(0x0090, 0x0040, 0x0080, 0x0080);
        let upper_edge =
            shishka_bagel_head_1_activation_window_loc_39506(0x0090, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (lower_edge.next, upper_edge.next),
            (
                ShishkaBagelHead1ActivationNextLoc39506::AttackAnimationLoc39548,
                ShishkaBagelHead1ActivationNextLoc39506::AttackAnimationLoc39548,
            ),
            "ASM loc_39506: both vertical compare bounds are inclusive",
        );
        assert!(lower_edge.kid_within_vertical_window);
        assert!(upper_edge.kid_within_vertical_window);
    }

    #[test]
    fn shishka_bagel_head_1_activation_loc_39534_faces_left_when_head_right_of_kid() {
        // ASM loc_39534 (kid.asm:42127): `cmp.w d5,d3; bgt.s loc_39534` →
        // `sf x_direction(a3)` clears direction to 0 when head_x > kid_x
        let result =
            shishka_bagel_head_1_activation_window_loc_39506(0x007F, 0x0040, 0x0080, 0x0080);

        assert!(result.kid_within_vertical_window);
        assert_eq!(
            (
                result.x_direction,
                result.cached_x_direction_fffffb86,
                result.animation_script_label,
                result.next,
            ),
            (
                Some(0),
                Some(0),
                Some("stru_39FC6"),
                ShishkaBagelHead1ActivationNextLoc39506::AttackAnimationLoc39548,
            ),
            "ASM loc_39534: sf x_direction(a3) when head_x > kid_x; loc_39538: mirror to $FFFFFB86, start stru_39FC6",
        );
    }

    #[test]
    fn shishka_bagel_head_1_activation_loc_39538_faces_right_on_equal_or_left_head_x() {
        // ASM loc_39538 (kid.asm:42130): fallthrough `st x_direction(a3)` → $FF
        // when head_x <= kid_x, then mirror to $FFFFFB86 and start stru_39FC6
        let equal =
            shishka_bagel_head_1_activation_window_loc_39506(0x0080, 0x0080, 0x0080, 0x0080);
        let left = shishka_bagel_head_1_activation_window_loc_39506(0x0081, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (equal.x_direction, left.x_direction),
            (Some(0xFF), Some(0xFF)),
            "ASM loc_39534 skipped: st x_direction(a3) stores $FF when head_x <= kid_x",
        );
        assert_eq!(equal.cached_x_direction_fffffb86, Some(0xFF));
        assert_eq!(left.animation_script_label, Some("stru_39FC6"));
    }

    #[test]
    fn shishka_bagel_head_1_attack_wait_loc_39548_shishka_zero_status() {
        // ASM loc_39548 (kid.asm:42135): hibernate, tst.w d2 → beq sub_398BC,
        // d7→$FFFFFB72, beq.s loc_39566 (check animation done)
        let result = shishka_bagel_head_1_attack_wait_loop_loc_39548(0, 0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_shishka_movement_sub_398bc);
        assert!(!result.runs_bagel_movement_sub_39bf8);
        assert_eq!(result.cached_collision_word_fffffb72, 0);
        assert_eq!(
            result.next,
            ShishkaBagelHead1AttackWaitNextLoc39548::CheckAttackAnimationDoneLoc39566,
            "ASM loc_39548: zero d7 → beq.s loc_39566 to check animation done",
        );
    }

    #[test]
    fn shishka_bagel_head_1_attack_wait_loc_39548_bagel_zero_status() {
        // ASM loc_39548+loc_39556: d2!=0 → bsr.w sub_39BF8 (BagelBrothers path)
        let result = shishka_bagel_head_1_attack_wait_loop_loc_39548(1, 0);

        assert!(result.hibernates_one_frame);
        assert!(!result.runs_shishka_movement_sub_398bc);
        assert!(result.runs_bagel_movement_sub_39bf8);
        assert_eq!(
            result.next,
            ShishkaBagelHead1AttackWaitNextLoc39548::CheckAttackAnimationDoneLoc39566,
        );
    }

    #[test]
    fn shishka_bagel_head_1_attack_wait_loc_39548_nonzero_collision_exit() {
        // ASM loc_3955A: move.w d7,($FFFFFB72).w; tst.w d7;
        // bra.w loc_39EBE on nonzero
        let result = shishka_bagel_head_1_attack_wait_loop_loc_39548(0, 0x0005);

        assert_eq!(result.cached_collision_word_fffffb72, 0x0005);
        assert_eq!(
            result.next,
            ShishkaBagelHead1AttackWaitNextLoc39548::SharedCollisionExitLoc39EBE,
            "ASM loc_3955A: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_1_attack_anim_gate_loc_39566_not_done() {
        // ASM loc_39566 (kid.asm:42153): tst.b $18(a3); beq.s loc_39548
        // Animation not done ($18==0) → loop back to attack wait
        let result = shishka_bagel_head_1_attack_anim_gate_loc_39566(0);

        assert!(!result.attack_animation_done);
        assert!(!result.runs_projectile_spawn_sub_397e4);
        assert_eq!(
            result.next,
            ShishkaBagelHead1AttackAnimGateNextLoc39566::AttackWaitLoopLoc39548,
            "ASM loc_39566: $18==0 → beq.s loc_39548 to continue attack wait loop",
        );
    }

    #[test]
    fn shishka_bagel_head_1_attack_anim_gate_loc_39566_done_spawns_projectile() {
        // ASM loc_39566 (kid.asm:42153): tst.b $18(a3); bne → bsr.w sub_397E4
        // Animation done ($18!=0) → spawn projectile, then post-projectile wait
        let result = shishka_bagel_head_1_attack_anim_gate_loc_39566(1);

        assert!(result.attack_animation_done);
        assert!(result.runs_projectile_spawn_sub_397e4);
        assert_eq!(
            result.next,
            ShishkaBagelHead1AttackAnimGateNextLoc39566::SpawnProjectileSub397E4,
            "ASM loc_39566: $18!=0 → bsr.w sub_397E4 to spawn head 1 projectile",
        );
    }

    #[test]
    fn shishka_bagel_head_1_post_projectile_wait_loc_39570_shishka_zero_status() {
        // ASM loc_39570 (kid.asm:42158): hibernate, tst.w d2 → beq sub_398BC,
        // d7→$FFFFFB72, beq.s loc_3958E (check animation done)
        let result = shishka_bagel_head_1_post_projectile_wait_loop_loc_39570(0, 0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_shishka_movement_sub_398bc);
        assert!(!result.runs_bagel_movement_sub_39bf8);
        assert_eq!(result.cached_collision_word_fffffb72, 0);
        assert_eq!(
            result.next,
            ShishkaBagelHead1PostProjectileWaitNextLoc39570::CheckAnimationDoneLoc3958E,
            "ASM loc_39570: zero d7 → beq.s loc_3958E to check animation done",
        );
    }

    #[test]
    fn shishka_bagel_head_1_post_projectile_wait_loc_39570_bagel_zero_status() {
        // ASM loc_39570+loc_3957E (kid.asm:42158+42166): d2!=0 → bsr.w sub_39BF8
        let result = shishka_bagel_head_1_post_projectile_wait_loop_loc_39570(1, 0);

        assert!(result.hibernates_one_frame);
        assert!(!result.runs_shishka_movement_sub_398bc);
        assert!(result.runs_bagel_movement_sub_39bf8);
        assert_eq!(
            result.next,
            ShishkaBagelHead1PostProjectileWaitNextLoc39570::CheckAnimationDoneLoc3958E,
            "ASM loc_3957E: BagelBrothers path converges at loc_39582 with zero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_1_post_projectile_wait_loc_39570_nonzero_collision_exit() {
        // ASM loc_39582: move.w d7,($FFFFFB72).w; tst.w d7;
        // bra.w loc_39EBE on nonzero
        let result = shishka_bagel_head_1_post_projectile_wait_loop_loc_39570(0, 0x0003);

        assert_eq!(result.cached_collision_word_fffffb72, 0x0003);
        assert_eq!(
            result.next,
            ShishkaBagelHead1PostProjectileWaitNextLoc39570::SharedCollisionExitLoc39EBE,
            "ASM loc_39582: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_1_post_projectile_anim_gate_loc_3958e_not_done() {
        // ASM loc_3958E (kid.asm:42176): tst.b $18(a3); beq.s loc_39570
        let result = shishka_bagel_head_1_post_projectile_anim_gate_loc_3958E(0);

        assert!(!result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            ShishkaBagelHead1PostProjectileAnimGateNextLoc3958E::AwaitPostProjectileWaitLoc39570,
            "ASM loc_3958E: $18==0 → beq.s loc_39570 to continue post-projectile wait",
        );
    }

    #[test]
    fn shishka_bagel_head_1_post_projectile_anim_gate_loc_3958e_done_returns() {
        // ASM loc_3958E (kid.asm:42176): tst.b $18(a3); bne → bra.w loc_394E8
        let result = shishka_bagel_head_1_post_projectile_anim_gate_loc_3958E(1);

        assert!(result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            ShishkaBagelHead1PostProjectileAnimGateNextLoc3958E::ReturnToActivationWaitLoc394E8,
            "ASM loc_3958E: $18!=0 → bra.w loc_394E8 to return to head 1 activation wait",
        );
    }

    #[test]
    fn shishka_bagel_head_2_init_loc_39598_shishkaboss_variant() {
        // ASM loc_39598 (kid.asm:42182): when d2==0 (ShishkaBoss), beq.s loc_395B4
        // leaves d0=$22. loc_395B4 stores into object_meta, bset #7, jsr loc_32146.
        // loc_39608: ShishkaBoss_HitPointsPerHead=$14 overwrites initial $3E=$1E.
        let result = shishka_bagel_head_2_init_loc_39598(0x0280, 0x0160, 0);

        assert_eq!(
            (
                result.gfx_slot_base,
                result.priority,
                result.object_meta_base
            ),
            (0x0104_0002, 0, 0x0022),
            "ASM loc_39598: gfx slot $01040002, priority 0, object_meta $22 for ShishkaBoss",
        );
        assert!(
            result.sets_object_meta_bit_f,
            "ASM loc_395B4: bset #7,object_meta(a3) must set bit F of the stored word",
        );
        assert!(result.runs_common_boss_init_loc_32146);
        assert_eq!(
            (
                result.has_kid_collision,
                result.display_enabled,
                result.is_moved
            ),
            (false, true, true),
            "ASM loc_395B4: sf has_kid_collision, st $13, st is_moved",
        );
        assert_eq!(
            result.initial_field_3e, 0x001E,
            "ASM loc_395B4: move.w #$1E,$3E(a3) — initial value before HP override",
        );
        assert_eq!(
            (result.hit_points, result.movement_script_label),
            (0x0014, "unk_39872"),
            "ASM loc_39608: ShishkaBoss_HitPointsPerHead=$14, movement script unk_39872",
        );
        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0280, 0x0160),
            "ASM loc_395B4: anchor $44/$46 copied to x_pos/y_pos",
        );
        assert_eq!(
            (result.cached_worker_slot_label, result.sprite_mapping_label),
            ("$FFFFFB7E", "LnkTo_unk_C8600-Data_Index"),
            "ASM loc_395B4: cache a3 at $FFFFFB7E, sprite LnkTo_unk_C8600",
        );
        assert_eq!(
            (result.delay_counter_d0, result.script_index_d1),
            (1, 0),
            "ASM loc_395B4: moveq #1,d0 and move.w #0,d1 seed the movement script",
        );
    }

    #[test]
    fn shishka_bagel_head_2_init_loc_39598_bagel_variant() {
        // ASM loc_39598 (kid.asm:42182): when d2!=0 (BagelBrothers), d0=$24 is loaded
        // before falling through to loc_395B4. Uses unk_39C2E with
        // BagelBrothers_HitPointsPerHead=$1E (same as initial $3E value).
        let result = shishka_bagel_head_2_init_loc_39598(0x0300, 0x0180, 1);

        assert_eq!(
            result.object_meta_base, 0x0024,
            "ASM loc_39598: move.w #$24,d0 when d2!=0 (BagelBrothers)",
        );
        assert_eq!(
            (result.hit_points, result.movement_script_label),
            (0x001E, "unk_39C2E"),
            "ASM loc_395B4: BagelBrothers_HitPointsPerHead=$1E, movement script unk_39C2E",
        );
        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0300, 0x0180),
            "ASM loc_395B4: anchor positions forwarded from parent's $44/$46",
        );
    }

    #[test]
    fn shishka_bagel_head_2_wait_loop_loc_3960e_shishka_zero_status() {
        // ASM loc_3960E (kid.asm:42217): d2==0 (ShishkaBoss) → bsr.w sub_398BC.
        // ASM loc_39620 (kid.asm:42228): d7==0 → beq.s loc_3962C (activation window).
        let result = shishka_bagel_head_2_wait_loop_loc_3960E(0, 0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_shishka_movement_sub_398bc);
        assert!(!result.runs_bagel_movement_sub_39d4a);
        assert_eq!(result.cached_collision_word_fffffb74, 0);
        assert_eq!(
            result.next,
            ShishkaBagelHead2WaitNextLoc3960E::CheckKidWindowLoc3962C,
            "ASM loc_39620: tst.w d7; beq.s loc_3962C when d7==0",
        );
    }

    #[test]
    fn shishka_bagel_head_2_wait_loop_loc_3960e_bagel_zero_status() {
        // ASM loc_3960E+loc_3961C: d2!=0 → bsr.w sub_39D4A (BagelBrothers path)
        let result = shishka_bagel_head_2_wait_loop_loc_3960E(1, 0);

        assert!(!result.runs_shishka_movement_sub_398bc);
        assert!(result.runs_bagel_movement_sub_39d4a);
        assert_eq!(
            result.next,
            ShishkaBagelHead2WaitNextLoc3960E::CheckKidWindowLoc3962C,
            "ASM loc_3961C: BagelBrothers path converges at loc_39620 with zero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_2_wait_loop_loc_3960e_nonzero_collision_exit() {
        // ASM loc_39620: move.w d7,($FFFFFB74).w; tst.w d7;
        // bra.w loc_39EBE on nonzero
        let result = shishka_bagel_head_2_wait_loop_loc_3960E(0, 0x0007);

        assert_eq!(result.cached_collision_word_fffffb74, 0x0007);
        assert_eq!(
            result.next,
            ShishkaBagelHead2WaitNextLoc3960E::SharedCollisionExitLoc39EBE,
            "ASM loc_39620: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_2_activation_loc_3962c_waits_outside_signed_vertical_window() {
        // ASM loc_3962C (kid.asm:42235): Kid Y < head_y-$40 or Kid Y > head_y
        // → branch back to loc_3960E wait loop
        let above =
            shishka_bagel_head_2_activation_window_loc_3962C(0x0090, 0x003F, 0x0080, 0x0080);
        let below =
            shishka_bagel_head_2_activation_window_loc_3962C(0x0090, 0x0081, 0x0080, 0x0080);

        assert_eq!(
            above.next,
            ShishkaBagelHead2ActivationNextLoc3962C::WaitLoopLoc3960E,
            "ASM loc_3962C: blt.s loc_3960E when signed Kid Y < head_y-$40",
        );
        assert_eq!(
            below.next,
            ShishkaBagelHead2ActivationNextLoc3962C::WaitLoopLoc3960E,
            "ASM loc_3962C: bgt.s loc_3960E when signed Kid Y > head_y",
        );
        assert!(!above.kid_within_vertical_window);
        assert!(!below.kid_within_vertical_window);
        assert_eq!((above.x_direction, below.x_direction), (None, None));
    }

    #[test]
    fn shishka_bagel_head_2_activation_loc_3962c_proceeds_inside_signed_vertical_window() {
        // ASM loc_3962C: inclusive bounds [head_y-$40, head_y] → direction check
        let lower_edge =
            shishka_bagel_head_2_activation_window_loc_3962C(0x0090, 0x0040, 0x0080, 0x0080);
        let upper_edge =
            shishka_bagel_head_2_activation_window_loc_3962C(0x0090, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (lower_edge.next, upper_edge.next),
            (
                ShishkaBagelHead2ActivationNextLoc3962C::AttackAnimationLoc3966E,
                ShishkaBagelHead2ActivationNextLoc3962C::AttackAnimationLoc3966E,
            ),
            "ASM loc_3962C: both vertical compare bounds are inclusive",
        );
        assert!(lower_edge.kid_within_vertical_window);
        assert!(upper_edge.kid_within_vertical_window);
    }

    #[test]
    fn shishka_bagel_head_2_activation_loc_3965a_faces_left_when_head_right_of_kid() {
        // ASM loc_3965A (kid.asm:42253): `cmp.w d5,d3; bgt.s loc_3965A` →
        // `sf x_direction(a3)` clears direction to 0 when head_x > kid_x
        let result =
            shishka_bagel_head_2_activation_window_loc_3962C(0x007F, 0x0040, 0x0080, 0x0080);

        assert!(result.kid_within_vertical_window);
        assert_eq!(
            (
                result.x_direction,
                result.cached_x_direction_fffffb87,
                result.animation_script_label,
                result.next,
            ),
            (
                Some(0),
                Some(0),
                Some("stru_39FC6"),
                ShishkaBagelHead2ActivationNextLoc3962C::AttackAnimationLoc3966E,
            ),
            "ASM loc_3965A: sf x_direction(a3) when head_x > kid_x; loc_3965E: mirror to $FFFFFB87, start stru_39FC6",
        );
    }

    #[test]
    fn shishka_bagel_head_2_activation_loc_3965e_faces_right_on_equal_or_left_head_x() {
        // ASM loc_3965E (kid.asm:42256): fallthrough `st x_direction(a3)` → $FF
        // when head_x <= kid_x, then mirror to $FFFFFB87 and start stru_39FC6
        let equal =
            shishka_bagel_head_2_activation_window_loc_3962C(0x0080, 0x0080, 0x0080, 0x0080);
        let left = shishka_bagel_head_2_activation_window_loc_3962C(0x0081, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (equal.x_direction, left.x_direction),
            (Some(0xFF), Some(0xFF)),
            "ASM loc_3965A skipped: st x_direction(a3) stores $FF when head_x <= kid_x",
        );
        assert_eq!(equal.cached_x_direction_fffffb87, Some(0xFF));
        assert_eq!(left.animation_script_label, Some("stru_39FC6"));
    }

    #[test]
    fn shishka_bagel_head_2_attack_wait_loc_3966e_shishka_zero_status() {
        // ASM loc_3966E (kid.asm:42261): hibernate, tst.w d2 → beq sub_398BC,
        // d7→$FFFFFB74, beq.s loc_3968C (check animation done)
        let result = shishka_bagel_head_2_attack_wait_loop_loc_3966E(0, 0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_shishka_movement_sub_398bc);
        assert!(!result.runs_bagel_movement_sub_39d4a);
        assert_eq!(result.cached_collision_word_fffffb74, 0);
        assert_eq!(
            result.next,
            ShishkaBagelHead2AttackWaitNextLoc3966E::CheckAttackAnimationDoneLoc3968C,
            "ASM loc_3966E: zero d7 → beq.s loc_3968C to check animation done",
        );
    }

    #[test]
    fn shishka_bagel_head_2_attack_wait_loc_3966e_bagel_zero_status() {
        // ASM loc_3966E+loc_3967C: d2!=0 → bsr.w sub_39D4A (BagelBrothers path)
        let result = shishka_bagel_head_2_attack_wait_loop_loc_3966E(1, 0);

        assert!(result.hibernates_one_frame);
        assert!(!result.runs_shishka_movement_sub_398bc);
        assert!(result.runs_bagel_movement_sub_39d4a);
        assert_eq!(
            result.next,
            ShishkaBagelHead2AttackWaitNextLoc3966E::CheckAttackAnimationDoneLoc3968C,
        );
    }

    #[test]
    fn shishka_bagel_head_2_attack_wait_loc_3966e_nonzero_collision_exit() {
        // ASM loc_39680: move.w d7,($FFFFFB74).w; tst.w d7;
        // bra.w loc_39EBE on nonzero
        let result = shishka_bagel_head_2_attack_wait_loop_loc_3966E(0, 0x0005);

        assert_eq!(result.cached_collision_word_fffffb74, 0x0005);
        assert_eq!(
            result.next,
            ShishkaBagelHead2AttackWaitNextLoc3966E::SharedCollisionExitLoc39EBE,
            "ASM loc_39680: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_2_attack_anim_gate_loc_3968c_not_done() {
        // ASM loc_3968C (kid.asm:42279): tst.b $18(a3); beq.s loc_3966E
        // Animation not done ($18==0) → loop back to attack wait
        let result = shishka_bagel_head_2_attack_anim_gate_loc_3968C(0);

        assert!(!result.attack_animation_done);
        assert!(!result.runs_projectile_spawn_sub_397e4);
        assert_eq!(
            result.next,
            ShishkaBagelHead2AttackAnimGateNextLoc3968C::AttackWaitLoopLoc3966E,
            "ASM loc_3968C: $18==0 → beq.s loc_3966E to continue attack wait loop",
        );
    }

    #[test]
    fn shishka_bagel_head_2_attack_anim_gate_loc_3968c_done_spawns_projectile() {
        // ASM loc_3968C (kid.asm:42279): tst.b $18(a3); bne → bsr.w sub_397E4
        // Animation done ($18!=0) → spawn projectile, then post-projectile wait
        let result = shishka_bagel_head_2_attack_anim_gate_loc_3968C(1);

        assert!(result.attack_animation_done);
        assert!(result.runs_projectile_spawn_sub_397e4);
        assert_eq!(
            result.next,
            ShishkaBagelHead2AttackAnimGateNextLoc3968C::SpawnProjectileSub397E4,
            "ASM loc_3968C: $18!=0 → bsr.w sub_397E4 to spawn head 2 projectile",
        );
    }

    #[test]
    fn shishka_bagel_head_2_post_projectile_wait_loc_39696_shishka_zero_status() {
        // ASM loc_39696 (kid.asm:42284): hibernate, tst.w d2 → beq sub_398BC,
        // d7→$FFFFFB74, beq.s loc_396B4 (check animation done)
        let result = shishka_bagel_head_2_post_projectile_wait_loop_loc_39696(0, 0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_shishka_movement_sub_398bc);
        assert!(!result.runs_bagel_movement_sub_39d4a);
        assert_eq!(result.cached_collision_word_fffffb74, 0);
        assert_eq!(
            result.next,
            ShishkaBagelHead2PostProjectileWaitNextLoc39696::CheckAnimationDoneLoc396B4,
            "ASM loc_39696: zero d7 → beq.s loc_396B4 to check animation done",
        );
    }

    #[test]
    fn shishka_bagel_head_2_post_projectile_wait_loc_39696_bagel_zero_status() {
        // ASM loc_39696+loc_396A4 (kid.asm:42284+42292): d2!=0 → bsr.w sub_39D4A
        let result = shishka_bagel_head_2_post_projectile_wait_loop_loc_39696(1, 0);

        assert!(result.hibernates_one_frame);
        assert!(!result.runs_shishka_movement_sub_398bc);
        assert!(result.runs_bagel_movement_sub_39d4a);
        assert_eq!(
            result.next,
            ShishkaBagelHead2PostProjectileWaitNextLoc39696::CheckAnimationDoneLoc396B4,
            "ASM loc_396A4: BagelBrothers path converges at loc_396A8 with zero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_2_post_projectile_wait_loc_39696_nonzero_collision_exit() {
        // ASM loc_396A8: move.w d7,($FFFFFB74).w; tst.w d7;
        // bra.w loc_39EBE on nonzero
        let result = shishka_bagel_head_2_post_projectile_wait_loop_loc_39696(0, 0x0003);

        assert_eq!(result.cached_collision_word_fffffb74, 0x0003);
        assert_eq!(
            result.next,
            ShishkaBagelHead2PostProjectileWaitNextLoc39696::SharedCollisionExitLoc39EBE,
            "ASM loc_396A8: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_2_post_projectile_anim_gate_loc_396b4_not_done() {
        // ASM loc_396B4 (kid.asm:42302): tst.b $18(a3); beq.s loc_39696
        let result = shishka_bagel_head_2_post_projectile_anim_gate_loc_396B4(0);

        assert!(!result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            ShishkaBagelHead2PostProjectileAnimGateNextLoc396B4::AwaitPostProjectileWaitLoc39696,
            "ASM loc_396B4: $18==0 → beq.s loc_39696 to continue post-projectile wait",
        );
    }

    #[test]
    fn shishka_bagel_head_2_post_projectile_anim_gate_loc_396b4_done_returns() {
        // ASM loc_396B4 (kid.asm:42302): tst.b $18(a3); bne → bra.w loc_3960E
        let result = shishka_bagel_head_2_post_projectile_anim_gate_loc_396B4(1);

        assert!(result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            ShishkaBagelHead2PostProjectileAnimGateNextLoc396B4::ReturnToActivationWaitLoc3960E,
            "ASM loc_396B4: $18!=0 → bra.w loc_3960E to return to head 2 activation wait",
        );
    }

    #[test]
    fn shishka_bagel_head_3_init_loc_396be_shishkaboss_variant() {
        // ASM loc_396BE (kid.asm:42308): when d2==0 (ShishkaBoss), beq.s loc_396DA
        // leaves d0=$22. loc_396DA stores into object_meta, bset #7, jsr loc_32146.
        // loc_3972E: ShishkaBoss_HitPointsPerHead=$14 overwrites initial $3E=$1E.
        let result = shishka_bagel_head_3_init_loc_396BE(0x0280, 0x0160, 0);

        assert_eq!(
            (
                result.gfx_slot_base,
                result.priority,
                result.object_meta_base
            ),
            (0x0104_0002, 0, 0x0022),
            "ASM loc_396BE: gfx slot $01040002, priority 0, object_meta $22 for ShishkaBoss",
        );
        assert!(
            result.sets_object_meta_bit_f,
            "ASM loc_396DA: bset #7,object_meta(a3) must set bit F of the stored word",
        );
        assert!(result.runs_common_boss_init_loc_32146);
        assert_eq!(
            (
                result.has_kid_collision,
                result.display_enabled,
                result.is_moved
            ),
            (false, true, true),
            "ASM loc_396DA: sf has_kid_collision, st $13, st is_moved",
        );
        assert_eq!(
            result.initial_field_3e, 0x001E,
            "ASM loc_396DA: move.w #$1E,$3E(a3) — initial value before HP override",
        );
        assert_eq!(
            (result.hit_points, result.movement_script_label),
            (0x0014, "unk_39872"),
            "ASM loc_3972E: ShishkaBoss_HitPointsPerHead=$14, movement script unk_39872",
        );
        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0280, 0x0160),
            "ASM loc_396DA: anchor $44/$46 copied to x_pos/y_pos",
        );
        assert_eq!(
            (result.cached_worker_slot_label, result.sprite_mapping_label),
            ("$FFFFFB82", "LnkTo_unk_C8600-Data_Index"),
            "ASM loc_396DA: cache a3 at $FFFFFB82, sprite LnkTo_unk_C8600",
        );
        assert_eq!(
            (result.delay_counter_d0, result.script_index_d1),
            (1, 0),
            "ASM loc_396DA: moveq #1,d0 and move.w #0,d1 seed the movement script",
        );
    }

    #[test]
    fn shishka_bagel_head_3_init_loc_396be_bagel_variant() {
        // ASM loc_396BE (kid.asm:42308): when d2!=0 (BagelBrothers), d0=$24 is loaded
        // before falling through to loc_396DA. Uses unk_39D80 with
        // BagelBrothers_HitPointsPerHead=$1E (same as initial $3E value).
        let result = shishka_bagel_head_3_init_loc_396BE(0x0300, 0x0180, 1);

        assert_eq!(
            result.object_meta_base, 0x0024,
            "ASM loc_396BE: move.w #$24,d0 when d2!=0 (BagelBrothers)",
        );
        assert_eq!(
            (result.hit_points, result.movement_script_label),
            (0x001E, "unk_39D80"),
            "ASM loc_396DA: BagelBrothers_HitPointsPerHead=$1E, movement script unk_39D80",
        );
        assert_eq!(
            (result.x_pos, result.y_pos),
            (0x0300, 0x0180),
            "ASM loc_396DA: anchor positions forwarded from parent's $44/$46",
        );
    }

    #[test]
    fn shishka_bagel_head_3_wait_loop_loc_39734_shishka_zero_status() {
        // ASM loc_39734 (kid.asm:42343): d2==0 (ShishkaBoss) → bsr.w sub_398BC.
        // ASM loc_39746 (kid.asm:42354): d7==0 → beq.s loc_39752 (activation window).
        let result = shishka_bagel_head_3_wait_loop_loc_39734(0, 0);

        assert!(
            result.hibernates_one_frame,
            "ASM loc_39734: jsr (j_Hibernate_Object_1Frame).w before movement check",
        );
        assert!(
            result.runs_shishka_movement_sub_398bc,
            "ASM loc_39734: tst.w d2; bne.s loc_39742 — d2==0 calls sub_398BC",
        );
        assert!(
            !result.runs_bagel_movement_sub_39eb4,
            "ASM loc_39734: ShishkaBoss path must not call sub_39EB4",
        );
        assert_eq!(
            result.cached_collision_word_fffffb76, 0,
            "ASM loc_39746: move.w d7,($FFFFFB76).w stores zero status",
        );
        assert_eq!(
            result.next,
            ShishkaBagelHead3WaitNextLoc39734::CheckKidWindowLoc39752,
            "ASM loc_39746: tst.w d7; beq.s loc_39752 when d7==0",
        );
    }

    #[test]
    fn shishka_bagel_head_3_wait_loop_loc_39742_bagel_zero_status() {
        // ASM loc_39734 (kid.asm:42343): d2!=0 (BagelBrothers) → bne.s loc_39742.
        // ASM loc_39742 (kid.asm:42351): bsr.w sub_39EB4.
        // ASM loc_39746 (kid.asm:42354): d7==0 → beq.s loc_39752.
        let result = shishka_bagel_head_3_wait_loop_loc_39734(1, 0);

        assert!(
            !result.runs_shishka_movement_sub_398bc,
            "ASM loc_39742: BagelBrothers path must not call sub_398BC",
        );
        assert!(
            result.runs_bagel_movement_sub_39eb4,
            "ASM loc_39742: bsr.w sub_39EB4 for BagelBrothers variant",
        );
        assert_eq!(
            result.next,
            ShishkaBagelHead3WaitNextLoc39734::CheckKidWindowLoc39752,
            "ASM loc_39746: d7==0 falls through to activation window check",
        );
    }

    #[test]
    fn shishka_bagel_head_3_wait_loop_loc_39746_nonzero_collision_exit() {
        // ASM loc_39746 (kid.asm:42354): move.w d7,($FFFFFB76).w; tst.w d7;
        // bne → bra.w loc_39EBE (shared collision exit) when d7 nonzero.
        let result = shishka_bagel_head_3_wait_loop_loc_39734(0, 0x0003);

        assert_eq!(
            result.cached_collision_word_fffffb76, 0x0003,
            "ASM loc_39746: move.w d7,($FFFFFB76).w must store nonzero status",
        );
        assert_eq!(
            result.next,
            ShishkaBagelHead3WaitNextLoc39734::SharedCollisionExitLoc39EBE,
            "ASM loc_39746: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_3_activation_loc_39752_waits_outside_signed_vertical_window() {
        // ASM loc_39752 (kid.asm:42361): Kid Y < head_y-$40 or Kid Y > head_y
        // → branch back to loc_39734 wait loop
        let above =
            shishka_bagel_head_3_activation_window_loc_39752(0x0090, 0x003F, 0x0080, 0x0080);
        let below =
            shishka_bagel_head_3_activation_window_loc_39752(0x0090, 0x0081, 0x0080, 0x0080);

        assert_eq!(
            above.next,
            ShishkaBagelHead3ActivationNextLoc39752::WaitLoopLoc39734,
            "ASM loc_39752: blt.s loc_39734 when signed Kid Y < head_y-$40",
        );
        assert_eq!(
            below.next,
            ShishkaBagelHead3ActivationNextLoc39752::WaitLoopLoc39734,
            "ASM loc_39752: bgt.s loc_39734 when signed Kid Y > head_y",
        );
        assert!(!above.kid_within_vertical_window);
        assert!(!below.kid_within_vertical_window);
        assert_eq!((above.x_direction, below.x_direction), (None, None));
    }

    #[test]
    fn shishka_bagel_head_3_activation_loc_39752_proceeds_inside_signed_vertical_window() {
        // ASM loc_39752: inclusive bounds [head_y-$40, head_y] → direction check
        let lower_edge =
            shishka_bagel_head_3_activation_window_loc_39752(0x0090, 0x0040, 0x0080, 0x0080);
        let upper_edge =
            shishka_bagel_head_3_activation_window_loc_39752(0x0090, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (lower_edge.next, upper_edge.next),
            (
                ShishkaBagelHead3ActivationNextLoc39752::AttackAnimationLoc39794,
                ShishkaBagelHead3ActivationNextLoc39752::AttackAnimationLoc39794,
            ),
            "ASM loc_39752: both vertical compare bounds are inclusive",
        );
        assert!(lower_edge.kid_within_vertical_window);
        assert!(upper_edge.kid_within_vertical_window);
    }

    #[test]
    fn shishka_bagel_head_3_activation_loc_39780_faces_left_when_head_right_of_kid() {
        // ASM loc_39780 (kid.asm:42379): `cmp.w d5,d3; bgt.s loc_39780` →
        // `sf x_direction(a3)` clears direction to 0 when head_x > kid_x
        let result =
            shishka_bagel_head_3_activation_window_loc_39752(0x007F, 0x0040, 0x0080, 0x0080);

        assert!(result.kid_within_vertical_window);
        assert_eq!(
            (
                result.x_direction,
                result.cached_x_direction_fffffb88,
                result.animation_script_label,
                result.next,
            ),
            (
                Some(0),
                Some(0),
                Some("stru_39FC6"),
                ShishkaBagelHead3ActivationNextLoc39752::AttackAnimationLoc39794,
            ),
            "ASM loc_39780: sf x_direction(a3) when head_x > kid_x; loc_39784: mirror to $FFFFFB88, start stru_39FC6",
        );
    }

    #[test]
    fn shishka_bagel_head_3_activation_loc_39784_faces_right_on_equal_or_left_head_x() {
        // ASM loc_39784 (kid.asm:42382): fallthrough `st x_direction(a3)` → $FF
        // when head_x <= kid_x, then mirror to $FFFFFB88 and start stru_39FC6
        let equal =
            shishka_bagel_head_3_activation_window_loc_39752(0x0080, 0x0080, 0x0080, 0x0080);
        let left = shishka_bagel_head_3_activation_window_loc_39752(0x0081, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (equal.x_direction, left.x_direction),
            (Some(0xFF), Some(0xFF)),
            "ASM loc_39780 skipped: st x_direction(a3) stores $FF when head_x <= kid_x",
        );
        assert_eq!(equal.cached_x_direction_fffffb88, Some(0xFF));
        assert_eq!(left.animation_script_label, Some("stru_39FC6"));
    }

    #[test]
    fn shishka_bagel_head_3_attack_wait_loc_39794_shishka_zero_status() {
        // ASM loc_39794 (kid.asm:42387): hibernate, tst.w d2 → beq sub_398BC,
        // d7→$FFFFFB76, beq.s loc_397B2 (check animation done)
        let result = shishka_bagel_head_3_attack_wait_loop_loc_39794(0, 0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_shishka_movement_sub_398bc);
        assert!(!result.runs_bagel_movement_sub_39eb4);
        assert_eq!(result.cached_collision_word_fffffb76, 0);
        assert_eq!(
            result.next,
            ShishkaBagelHead3AttackWaitNextLoc39794::CheckAttackAnimationDoneLoc397B2,
            "ASM loc_39794: zero d7 → beq.s loc_397B2 to check animation done",
        );
    }

    #[test]
    fn shishka_bagel_head_3_attack_wait_loc_39794_bagel_zero_status() {
        // ASM loc_39794+loc_397A2: d2!=0 → bsr.w sub_39EB4 (BagelBrothers path)
        let result = shishka_bagel_head_3_attack_wait_loop_loc_39794(1, 0);

        assert!(result.hibernates_one_frame);
        assert!(!result.runs_shishka_movement_sub_398bc);
        assert!(result.runs_bagel_movement_sub_39eb4);
        assert_eq!(
            result.next,
            ShishkaBagelHead3AttackWaitNextLoc39794::CheckAttackAnimationDoneLoc397B2,
        );
    }

    #[test]
    fn shishka_bagel_head_3_attack_wait_loc_39794_nonzero_collision_exit() {
        // ASM loc_397A6: move.w d7,($FFFFFB76).w; tst.w d7;
        // bra.w loc_39EBE on nonzero
        let result = shishka_bagel_head_3_attack_wait_loop_loc_39794(0, 0x0005);

        assert_eq!(result.cached_collision_word_fffffb76, 0x0005);
        assert_eq!(
            result.next,
            ShishkaBagelHead3AttackWaitNextLoc39794::SharedCollisionExitLoc39EBE,
            "ASM loc_397A6: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_3_post_projectile_wait_loc_397bc_shishka_zero_status() {
        // ASM loc_397BC (kid.asm:42410): hibernate, tst.w d2 → beq sub_398BC,
        // d7→$FFFFFB76, beq.s loc_397DA (check animation done)
        let result = shishka_bagel_head_3_post_projectile_wait_loop_loc_397bc(0, 0);

        assert!(
            result.hibernates_one_frame,
            "ASM loc_397BC: jsr (j_Hibernate_Object_1Frame).w before movement check",
        );
        assert!(
            result.runs_shishka_movement_sub_398bc,
            "ASM loc_397BC: tst.w d2; d2==0 calls sub_398BC",
        );
        assert!(
            !result.runs_bagel_movement_sub_39eb4,
            "ASM loc_397BC: ShishkaBoss path must not call sub_39EB4",
        );
        assert_eq!(result.cached_collision_word_fffffb76, 0);
        assert_eq!(
            result.next,
            ShishkaBagelHead3PostProjectileWaitNextLoc397BC::CheckAnimationDoneLoc397DA,
            "ASM loc_397CE: zero d7 → beq.s loc_397DA to check animation done",
        );
    }

    #[test]
    fn shishka_bagel_head_3_post_projectile_wait_loc_397bc_bagel_zero_status() {
        // ASM loc_397BC+loc_397CA: d2!=0 → bsr.w sub_39EB4 (BagelBrothers path)
        let result = shishka_bagel_head_3_post_projectile_wait_loop_loc_397bc(1, 0);

        assert!(result.runs_bagel_movement_sub_39eb4);
        assert!(!result.runs_shishka_movement_sub_398bc);
        assert_eq!(
            result.next,
            ShishkaBagelHead3PostProjectileWaitNextLoc397BC::CheckAnimationDoneLoc397DA,
            "ASM loc_397CE: zero d7 → beq.s loc_397DA",
        );
    }

    #[test]
    fn shishka_bagel_head_3_post_projectile_wait_loc_397bc_nonzero_collision_exit() {
        // ASM loc_397CE: move.w d7,($FFFFFB76).w; tst.w d7;
        // bra.w loc_39EBE on nonzero
        let result = shishka_bagel_head_3_post_projectile_wait_loop_loc_397bc(0, 0x0007);

        assert_eq!(result.cached_collision_word_fffffb76, 0x0007);
        assert_eq!(
            result.next,
            ShishkaBagelHead3PostProjectileWaitNextLoc397BC::SharedCollisionExitLoc39EBE,
            "ASM loc_397CE: tst.w d7; bra.w loc_39EBE on nonzero d7",
        );
    }

    #[test]
    fn shishka_bagel_head_3_post_projectile_anim_gate_loc_397da_not_done() {
        // ASM loc_397DA (kid.asm:42428): tst.b $18(a3); beq.s loc_397BC
        let result = shishka_bagel_head_3_post_projectile_anim_gate_loc_397da(0);

        assert!(!result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            ShishkaBagelHead3PostProjectileAnimGateNextLoc397DA::AwaitPostProjectileWaitLoc397BC,
            "ASM loc_397DA: zero $18(a3) loops back to loc_397BC",
        );
    }

    #[test]
    fn shishka_bagel_head_3_post_projectile_anim_gate_loc_397da_done() {
        // ASM loc_397DA: nonzero $18(a3) → bra.w loc_39734 (return to wait loop)
        let result = shishka_bagel_head_3_post_projectile_anim_gate_loc_397da(1);

        assert!(result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            ShishkaBagelHead3PostProjectileAnimGateNextLoc397DA::ReturnToWaitLoopLoc39734,
            "ASM loc_397DA: nonzero $18(a3) returns to loc_39734 wait loop",
        );
    }

    #[test]
    fn boss_eye_orbit_sub_393c8_loc_393de_applies_right_table_offset_and_y_shift() {
        // ASM loc_393DE (kid.asm:41970): common path after table selection in sub_393C8.
        // When d2==0, beq.s loc_393DE skips the word_3940C load and uses word_39416[0]=$FFF4=-12.
        // loc_393DE: add.w $1A(a2),d7 → x_pos; move.w $1E(a2) → y_pos; subi.w #$25 → y_pos.
        let result = boss_eye_orbit_placement_sub_393C8(0x0180, 0x00C0, 0x0EE0, 0, 5, 5, 0x0008);

        assert_eq!(
            result.x_pos,
            0x0174, // 0x0180 + (-12) = 0x0174
            "ASM loc_393DE: add.w $1A(a2),d7 should add word_39416[0]=$FFF4 to parent X",
        );
        assert_eq!(
            result.y_pos,
            0x009B, // 0x00C0 - 0x25 = 0x009B
            "ASM loc_393DE: subi.w #$25,y_pos(a3) should subtract $25 from parent Y",
        );
    }

    #[test]
    fn boss_eye_orbit_sub_393c8_loc_39406_calls_sub_ff6_after_nonzero_to_different_frame_change() {
        // ASM loc_39406 (kid.asm:41986): jsr (j_sub_FF6).w — convergence of both
        // the subq.w #1,$A(a3) path (d0!=0, frame changed) and the addq.w #1,$A(a3)
        // path (d0==0, frame changed) in sub_393C8. Both reach loc_39406 to rerank.
        // Exercise the subq path: d0=3 (nonzero), current_frame=7 (different).
        let result = boss_eye_orbit_placement_sub_393C8(0x0200, 0x0100, 0x0EE6, 0, 3, 7, 0x0020);

        assert_eq!(
            result.next_sprite_word_y_adjust, 0x001F,
            "ASM sub_393C8 subq.w #1,$A(a3): d0=3 nonzero and frame changed → decrement",
        );
        assert!(
            result.rerank_gfx_object,
            "ASM loc_39406: jsr (j_sub_FF6).w must fire after the frame-change adjustment path",
        );
    }

    #[test]
    fn boss_eye_counter_orbit_sub_39420_loc_39436_applies_right_table_offset_and_y_shift() {
        // ASM loc_39436 (kid.asm:42017): common path after table selection in sub_39420.
        // When d2==0, beq.s loc_39436 skips the word_39464 load and uses word_3946E[0]=$000C=12.
        // loc_39436: add.w $1A(a2),d7 → x_pos; move.w $1E(a2) → y_pos; subi.w #$25 → y_pos.
        let result =
            boss_eye_counter_orbit_placement_sub_39420(0x0180, 0x00C0, 0x0EE0, 0, 5, 5, 0x0008);

        assert_eq!(
            result.x_pos,
            0x018C, // 0x0180 + 12 = 0x018C
            "ASM loc_39436: add.w $1A(a2),d7 should add word_3946E[0]=$000C to parent X",
        );
        assert_eq!(
            result.y_pos,
            0x009B, // 0x00C0 - 0x25 = 0x009B
            "ASM loc_39436: subi.w #$25,y_pos(a3) should subtract $25 from parent Y",
        );
    }

    #[test]
    fn boss_head_projectile_spawn_sub_397e4_uses_loc_3a0d2_and_spawns_24px_left_for_clear_x_direction()
     {
        let result = boss_head_projectile_spawn_sub_397E4(0x0200, 0x0100, 0, 0);

        assert_eq!(
            result.callback_label, "loc_3A0D2",
            "ASM sub_397E4 should keep `4(a0)` on `loc_3A0D2` when `d1` is zero",
        );
        assert_eq!(
            (result.x_pos, result.y_pos, result.x_direction),
            (0x01E8, 0x0100, 0),
            "ASM loc_3980A should copy the parent's `$46/$49` and subtract `$18` from X when `x_direction(a3)` is clear",
        );
        assert_eq!(
            result.velocity_word_48, -3,
            "ASM loc_3980A should write `$FFFD` to `$48(a0)` for the left-facing spawn path",
        );
        assert_eq!(
            result.animation_script_label, "stru_39FD8",
            "ASM loc_39836 should start the child on `stru_39FD8` before returning",
        );
    }

    #[test]
    fn boss_head_projectile_spawn_sub_397e4_uses_loc_3a1ea_and_spawns_24px_right_for_set_x_direction()
     {
        let result = boss_head_projectile_spawn_sub_397E4(0x0200, 0x0100, 0xFF, 1);

        assert_eq!(
            result.callback_label, "loc_3A1EA",
            "ASM loc_397FA should switch `4(a0)` to `loc_3A1EA` when `d1` is nonzero",
        );
        assert_eq!(
            (result.x_pos, result.y_pos, result.x_direction),
            (0x0218, 0x0100, 0xFF),
            "ASM loc_3982C should add `$18` to X and preserve `$49(a0)` when `x_direction(a3)` is set",
        );
        assert_eq!(
            result.velocity_word_48, 3,
            "ASM loc_3982C should write `3` to `$48(a0)` for the right-facing spawn path",
        );
    }

    #[test]
    fn boomerang_boss_head_1_init_loc_37f30_seeds_the_anchor_head_and_movement_script() {
        let result = boomerang_boss_head_1_init_loc_37F30(0x0340, 0x0120);

        assert_eq!(
            (
                result.gfx_slot_base,
                result.priority,
                result.object_meta,
                result.has_kid_collision,
                result.display_enabled,
                result.is_moved,
            ),
            (0x0104_0002, 1, 0x0023, false, true, true),
            "ASM loc_37F30 should initialise BoomerangBoss head 1's gfx slot, metadata, collision/display flags, and movement bit",
        );
        assert!(result.runs_common_boss_init_loc_32146);
        assert_eq!(
            (
                result.hit_points,
                result.sprite_mapping_label,
                result.cached_worker_slot_label,
            ),
            (30, "LnkTo_unk_C8600-Data_Index", "$FFFFFB7A"),
            "ASM loc_37F30 should store `BoomerangBoss_HitPointsPerHead`, mapping `LnkTo_unk_C8600`, and cache `a3` at `$FFFFFB7A`",
        );
        assert_eq!(
            (
                result.x_pos,
                result.y_pos,
                result.movement_script_label,
                result.delay_counter_d0,
                result.script_index_d1,
            ),
            (0x0340, 0x0120, "unk_398F0", 1, 1),
            "ASM loc_37F30 should copy anchor `$44/$46`, load `unk_398F0`, and seed `d0/d1` to 1 before loc_37F80",
        );
    }

    #[test]
    fn boomerang_boss_head_1_wait_loop_loc_37f80_checks_kid_window_when_status_is_zero() {
        let result = boomerang_boss_head_1_wait_loop_loc_37F80(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_39952,
            "ASM loc_37F80 should run `sub_39952` after `Hibernate_Object_1Frame`",
        );
        assert_eq!(
            result.cached_collision_word_fffffb72, 0,
            "ASM loc_37F80 should copy zero `d7` into `$FFFFFB72` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHeadWaitNextLoc37F80::CheckKidWindowLoc37F94,
            "ASM loc_37F80 should fall through to loc_37F94 when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_wait_loop_loc_37f80_branches_to_shared_exit_when_status_is_nonzero() {
        let result = boomerang_boss_head_1_wait_loop_loc_37F80(0x002C);

        assert_eq!(
            result.cached_collision_word_fffffb72, 0x002C,
            "ASM loc_37F80 should preserve the nonzero `d7` word from `sub_39952` in `$FFFFFB72`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHeadWaitNextLoc37F80::SharedCollisionExitLoc39EBE,
            "ASM loc_37F80 should branch to loc_39EBE instead of loc_37F94 when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_activation_loc_37f94_waits_outside_the_signed_vertical_window() {
        let above = boomerang_boss_head_1_activation_loc_37F94(0x0080, 0x003F, 0x0090, 0x0080);
        let below = boomerang_boss_head_1_activation_loc_37F94(0x0080, 0x0081, 0x0090, 0x0080);

        assert_eq!(
            above.next,
            BoomerangBossHeadActivationNextLoc37F94::WaitLoopLoc37F80,
            "ASM loc_37F94 should branch back to loc_37F80 when signed Kid Y is less than `y_pos(a3)-$40`",
        );
        assert_eq!(
            below.next,
            BoomerangBossHeadActivationNextLoc37F94::WaitLoopLoc37F80,
            "ASM loc_37F94 should branch back to loc_37F80 when signed Kid Y is greater than `y_pos(a3)`",
        );
        assert_eq!((above.x_direction, below.x_direction), (None, None));
    }

    #[test]
    fn boomerang_boss_head_1_activation_loc_37f94_faces_left_when_head_is_strictly_right_of_kid() {
        let result = boomerang_boss_head_1_activation_loc_37F94(0x007F, 0x0040, 0x0080, 0x0080);

        assert!(result.kid_within_vertical_window);
        assert_eq!(
            (
                result.x_direction,
                result.cached_x_direction_fffffb86,
                result.animation_script_label,
                result.next,
            ),
            (
                Some(0),
                Some(0),
                Some("stru_39FC6"),
                BoomerangBossHeadActivationNextLoc37F94::AttackAnimationLoc37FC6,
            ),
            "ASM loc_37F94/loc_37FC2 should clear `x_direction(a3)` when `head_x > kid_x`; ASM loc_37FC6 should mirror it to `$FFFFFB86` and start `stru_39FC6`",
        );
    }

    #[test]
    fn boomerang_boss_head_1_activation_loc_37f94_faces_right_on_equal_or_left_head_x() {
        let equal = boomerang_boss_head_1_activation_loc_37F94(0x0080, 0x0080, 0x0080, 0x0080);
        let left = boomerang_boss_head_1_activation_loc_37F94(0x0081, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (equal.x_direction, left.x_direction),
            (Some(0xFF), Some(0xFF)),
            "ASM loc_37F94 should set `x_direction(a3)` unless the signed `cmp.w d5,d3; bgt.s loc_37FC2` branch fires",
        );
        assert_eq!(equal.animation_script_label, Some("stru_39FC6"));
        assert_eq!(left.cached_x_direction_fffffb86, Some(0xFF));
    }

    #[test]
    fn boomerang_boss_head_1_attack_wait_loop_loc_37fd6_checks_animation_when_status_is_zero() {
        let result = boomerang_boss_head_1_attack_wait_loop_loc_37FD6(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_39952,
            "ASM loc_37FD6 should run `sub_39952` after hibernating the attack animation frame",
        );
        assert_eq!(
            result.cached_collision_word_fffffb72, 0,
            "ASM loc_37FD6 should copy zero `d7` into `$FFFFFB72` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHeadAttackWaitNextLoc37FD6::CheckAttackAnimationDoneLoc37FEA,
            "ASM loc_37FD6 should fall through to loc_37FEA when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_attack_wait_loop_loc_37fd6_branches_to_shared_exit_on_status() {
        let result = boomerang_boss_head_1_attack_wait_loop_loc_37FD6(0x0001);

        assert_eq!(
            result.cached_collision_word_fffffb72, 0x0001,
            "ASM loc_37FD6 should preserve nonzero `d7` in `$FFFFFB72`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHeadAttackWaitNextLoc37FD6::SharedCollisionExitLoc39EBE,
            "ASM loc_37FD6 should branch to loc_39EBE instead of testing `$18(a3)` when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_attack_animation_gate_loc_37fea_loops_until_animation_finishes() {
        let result = boomerang_boss_head_1_attack_animation_gate_loc_37FEA(0);

        assert!(!result.attack_animation_done);
        assert!(!result.runs_projectile_spawn_sub_397e4);
        assert_eq!(
            result.next,
            BoomerangBossHeadAttackAnimationNextLoc37FEA::AwaitAnimationLoc37FD6,
            "ASM loc_37FEA should branch back to loc_37FD6 while `$18(a3)` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_attack_animation_gate_loc_37fea_spawns_projectile_when_done() {
        let result = boomerang_boss_head_1_attack_animation_gate_loc_37FEA(1);

        assert!(result.attack_animation_done);
        assert!(
            result.runs_projectile_spawn_sub_397e4,
            "ASM loc_37FEA should call `sub_397E4` as soon as `$18(a3)` is nonzero",
        );
        assert_eq!(
            result.next,
            BoomerangBossHeadAttackAnimationNextLoc37FEA::SpawnProjectileSub397E4,
        );
    }

    #[test]
    fn boomerang_boss_head_1_post_projectile_wait_loop_loc_37ff4_checks_animation_when_status_is_zero()
     {
        let result = boomerang_boss_head_1_post_projectile_wait_loop_loc_37FF4(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_39952,
            "ASM loc_37FF4 should run `sub_39952` after hibernating the post-projectile frame",
        );
        assert_eq!(
            result.cached_collision_word_fffffb72, 0,
            "ASM loc_37FF4 should copy zero `d7` into `$FFFFFB72` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHeadPostProjectileWaitNextLoc37FF4::CheckAnimationDone,
            "ASM loc_37FF4 should fall through to the animation-done gate when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_post_projectile_wait_loop_loc_37ff4_branches_to_shared_exit_on_status()
    {
        let result = boomerang_boss_head_1_post_projectile_wait_loop_loc_37FF4(0xFFFF);

        assert_eq!(
            result.cached_collision_word_fffffb72, 0xFFFF,
            "ASM loc_37FF4 should preserve nonzero `d7` in `$FFFFFB72`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHeadPostProjectileWaitNextLoc37FF4::SharedCollisionExitLoc39EBE,
            "ASM loc_37FF4 should branch to loc_39EBE instead of testing `$18(a3)` when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_post_projectile_animation_gate_loc_38008_loops_until_animation_finishes()
     {
        let result = boomerang_boss_head_1_post_projectile_animation_gate_loc_38008(0);

        assert_eq!(result.animation_done_byte_18, 0);
        assert!(!result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            BoomerangBossHeadPostProjectileAnimationNextLoc38008::AwaitPostProjectileWaitLoc37FF4,
            "ASM loc_38008 should branch back to loc_37FF4 while `$18(a3)` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_post_projectile_animation_gate_loc_38008_returns_to_activation_when_done()
     {
        let result = boomerang_boss_head_1_post_projectile_animation_gate_loc_38008(0x80);

        assert_eq!(result.animation_done_byte_18, 0x80);
        assert!(result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            BoomerangBossHeadPostProjectileAnimationNextLoc38008::ReturnToActivationWaitLoc37F80,
            "ASM loc_38008 should branch back to loc_37F80 once `$18(a3)` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_init_loc_38012_seeds_the_second_head_and_movement_script() {
        let result = boomerang_boss_head_2_init_loc_38012(0x0280, 0x0118);

        assert_eq!(
            (
                result.gfx_slot_base,
                result.priority,
                result.object_meta,
                result.has_kid_collision,
                result.display_enabled,
                result.is_moved,
            ),
            (0x0104_0002, 1, 0x0023, false, true, true),
            "ASM loc_38012 should initialise BoomerangBoss head 2's gfx slot, metadata, collision/display flags, and movement bit",
        );
        assert!(result.runs_common_boss_init_loc_32146);
        assert_eq!(
            (
                result.hit_points,
                result.sprite_mapping_label,
                result.cached_worker_slot_label,
            ),
            (30, "LnkTo_unk_C8600-Data_Index", "$FFFFFB7E"),
            "ASM loc_38012 should store `BoomerangBoss_HitPointsPerHead`, mapping `LnkTo_unk_C8600`, and cache `a3` at `$FFFFFB7E`",
        );
        assert_eq!(
            (
                result.x_pos,
                result.y_pos,
                result.movement_script_label,
                result.delay_counter_d0,
                result.script_index_d1,
            ),
            (0x0280, 0x0118, "unk_39988", 1, 1),
            "ASM loc_38012 should copy anchor `$44/$46`, load `unk_39988`, and seed `d0/d1` to 1 before its wait loop",
        );
    }

    #[test]
    fn boomerang_boss_head_3_init_loc_380f4_seeds_the_third_head_and_movement_script() {
        let result = boomerang_boss_head_3_init_loc_380F4(0x02A0, 0x0124);

        assert_eq!(
            (
                result.gfx_slot_base,
                result.priority,
                result.object_meta,
                result.has_kid_collision,
                result.display_enabled,
                result.is_moved,
            ),
            (0x0104_0002, 1, 0x0023, false, true, true),
            "ASM loc_380F4 should initialise BoomerangBoss head 3's gfx slot, metadata, collision/display flags, and movement bit",
        );
        assert!(result.runs_common_boss_init_loc_32146);
        assert_eq!(
            (
                result.hit_points,
                result.sprite_mapping_label,
                result.cached_worker_slot_label,
            ),
            (30, "LnkTo_unk_C8600-Data_Index", "$FFFFFB82"),
            "ASM loc_380F4 should store `BoomerangBoss_HitPointsPerHead`, mapping `LnkTo_unk_C8600`, and cache `a3` at `$FFFFFB82`",
        );
        assert_eq!(
            (
                result.x_pos,
                result.y_pos,
                result.movement_script_label,
                result.delay_counter_d0,
                result.script_index_d1,
            ),
            (0x02A0, 0x0124, "unk_39A1A", 1, 1),
            "ASM loc_380F4 should copy anchor `$44/$46`, load `unk_39A1A`, and seed `d0/d1` to 1 before its wait loop",
        );
    }

    #[test]
    fn boomerang_boss_head_2_wait_loop_loc_38062_checks_kid_window_when_status_is_zero() {
        let result = boomerang_boss_head_2_wait_loop_loc_38062(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_399e4,
            "ASM loc_38062 should run `sub_399E4` after `Hibernate_Object_1Frame`",
        );
        assert_eq!(
            result.cached_collision_word_fffffb74, 0,
            "ASM loc_38062 should copy zero `d7` into `$FFFFFB74` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2WaitNextLoc38062::CheckKidWindow,
            "ASM loc_38062 should fall through to the Kid-window check when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_wait_loop_loc_38062_branches_to_shared_exit_when_status_is_nonzero() {
        let result = boomerang_boss_head_2_wait_loop_loc_38062(0x0040);

        assert_eq!(
            result.cached_collision_word_fffffb74, 0x0040,
            "ASM loc_38062 should preserve nonzero `d7` in `$FFFFFB74`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2WaitNextLoc38062::SharedCollisionExitLoc39EBE,
            "ASM loc_38062 should branch to loc_39EBE instead of the Kid-window check when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_activation_window_loc_38076_waits_outside_signed_vertical_window() {
        let above =
            boomerang_boss_head_2_activation_window_loc_38076(0x0090, 0x003F, 0x0080, 0x0080);
        let below =
            boomerang_boss_head_2_activation_window_loc_38076(0x0090, 0x0081, 0x0080, 0x0080);

        assert_eq!(
            above.next,
            BoomerangBossHead2ActivationNextLoc38076::WaitLoopLoc38062,
            "ASM loc_38076 should branch back to loc_38062 when signed Kid Y is less than `y_pos(a3)-$40`",
        );
        assert_eq!(
            below.next,
            BoomerangBossHead2ActivationNextLoc38076::WaitLoopLoc38062,
            "ASM loc_38076 should branch back to loc_38062 when signed Kid Y is greater than `y_pos(a3)`",
        );
        assert_eq!(
            (
                above.kid_within_vertical_window,
                below.kid_within_vertical_window
            ),
            (false, false),
        );
        assert_eq!((above.x_direction, below.x_direction), (None, None));
    }

    #[test]
    fn boomerang_boss_head_2_activation_window_loc_38076_continues_inside_signed_vertical_window() {
        let lower_edge =
            boomerang_boss_head_2_activation_window_loc_38076(0x0090, 0x0040, 0x0080, 0x0080);
        let upper_edge =
            boomerang_boss_head_2_activation_window_loc_38076(0x0090, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (lower_edge.next, upper_edge.next),
            (
                BoomerangBossHead2ActivationNextLoc38076::AttackAnimation,
                BoomerangBossHead2ActivationNextLoc38076::AttackAnimation,
            ),
            "ASM loc_38076 should treat both vertical compare bounds as inclusive before choosing attack direction",
        );
        assert!(lower_edge.kid_within_vertical_window);
        assert!(upper_edge.kid_within_vertical_window);
    }

    #[test]
    fn boomerang_boss_head_2_activation_loc_380a4_faces_left_when_head_is_strictly_right_of_kid() {
        let result =
            boomerang_boss_head_2_activation_window_loc_38076(0x007F, 0x0040, 0x0080, 0x0080);

        assert!(result.kid_within_vertical_window);
        assert_eq!(
            (
                result.x_direction,
                result.cached_x_direction_fffffb87,
                result.animation_script_label,
                result.next,
            ),
            (
                Some(0),
                Some(0),
                Some("stru_39FC6"),
                BoomerangBossHead2ActivationNextLoc38076::AttackAnimation,
            ),
            "ASM loc_380A4 should clear `x_direction(a3)` when signed `head_x > kid_x`; ASM loc_380A8 should mirror that byte to `$FFFFFB87` and start `stru_39FC6`",
        );
    }

    #[test]
    fn boomerang_boss_head_2_activation_loc_380a4_faces_right_on_equal_or_left_head_x() {
        let equal =
            boomerang_boss_head_2_activation_window_loc_38076(0x0080, 0x0080, 0x0080, 0x0080);
        let left =
            boomerang_boss_head_2_activation_window_loc_38076(0x0081, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (equal.x_direction, left.x_direction),
            (Some(0xFF), Some(0xFF)),
            "ASM loc_380A4 should be skipped unless signed `head_x > kid_x`, leaving the fallthrough `$FF` direction",
        );
        assert_eq!(equal.cached_x_direction_fffffb87, Some(0xFF));
        assert_eq!(left.animation_script_label, Some("stru_39FC6"));
    }

    #[test]
    fn boomerang_boss_head_3_wait_loop_loc_38144_checks_kid_window_when_status_is_zero() {
        let result = boomerang_boss_head_3_wait_loop_loc_38144(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_39a7c,
            "ASM loc_38144 should run `sub_39A7C` after `Hibernate_Object_1Frame`",
        );
        assert_eq!(
            result.cached_collision_word_fffffb76, 0,
            "ASM loc_38144 should copy zero `d7` into `$FFFFFB76` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3WaitNextLoc38144::CheckKidWindow,
            "ASM loc_38144 should fall through to the Kid-window check when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_wait_loop_loc_38144_branches_to_shared_exit_when_status_is_nonzero() {
        let result = boomerang_boss_head_3_wait_loop_loc_38144(0x0040);

        assert_eq!(
            result.cached_collision_word_fffffb76, 0x0040,
            "ASM loc_38144 should preserve nonzero `d7` in `$FFFFFB76`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3WaitNextLoc38144::SharedCollisionExitLoc39EBE,
            "ASM loc_38144 should branch to loc_39EBE instead of the Kid-window check when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_activation_window_loc_38158_waits_outside_signed_vertical_window() {
        // ASM loc_38158: Kid Y < head_y-$40 or Kid Y > head_y → branch back to loc_38144
        let above =
            boomerang_boss_head_3_activation_window_loc_38158(0x0090, 0x003F, 0x0080, 0x0080);
        let below =
            boomerang_boss_head_3_activation_window_loc_38158(0x0090, 0x0081, 0x0080, 0x0080);

        assert_eq!(
            above.next,
            BoomerangBossHead3ActivationNextLoc38158::WaitLoopLoc38144,
            "ASM loc_38158 should branch back to loc_38144 when signed Kid Y is less than `y_pos(a3)-$40`",
        );
        assert_eq!(
            below.next,
            BoomerangBossHead3ActivationNextLoc38158::WaitLoopLoc38144,
            "ASM loc_38158 should branch back to loc_38144 when signed Kid Y is greater than `y_pos(a3)`",
        );
        assert_eq!(
            (
                above.kid_within_vertical_window,
                below.kid_within_vertical_window
            ),
            (false, false),
        );
        assert_eq!((above.x_direction, below.x_direction), (None, None));
    }

    #[test]
    fn boomerang_boss_head_3_activation_window_loc_38158_continues_inside_signed_vertical_window() {
        // ASM loc_38158: inclusive bounds [head_y-$40, head_y] → proceed to direction check
        let lower_edge =
            boomerang_boss_head_3_activation_window_loc_38158(0x0090, 0x0040, 0x0080, 0x0080);
        let upper_edge =
            boomerang_boss_head_3_activation_window_loc_38158(0x0090, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (lower_edge.next, upper_edge.next),
            (
                BoomerangBossHead3ActivationNextLoc38158::AttackAnimation,
                BoomerangBossHead3ActivationNextLoc38158::AttackAnimation,
            ),
            "ASM loc_38158 should treat both vertical compare bounds as inclusive before choosing attack direction",
        );
        assert!(lower_edge.kid_within_vertical_window);
        assert!(upper_edge.kid_within_vertical_window);
    }

    #[test]
    fn boomerang_boss_head_3_activation_loc_38186_faces_left_when_head_is_strictly_right_of_kid() {
        // ASM loc_38186: `cmp.w d5,d3; bgt.s` → sf x_direction(a3) when head_x > kid_x
        let result =
            boomerang_boss_head_3_activation_window_loc_38158(0x007F, 0x0040, 0x0080, 0x0080);

        assert!(result.kid_within_vertical_window);
        assert_eq!(
            (
                result.x_direction,
                result.cached_x_direction_fffffb88,
                result.animation_script_label,
                result.next,
            ),
            (
                Some(0),
                Some(0),
                Some("stru_39FC6"),
                BoomerangBossHead3ActivationNextLoc38158::AttackAnimation,
            ),
            "ASM loc_38186 should clear `x_direction(a3)` when signed `head_x > kid_x`; ASM loc_3818A should mirror that byte to `$FFFFFB88` and start `stru_39FC6`",
        );
    }

    #[test]
    fn boomerang_boss_head_3_activation_loc_38186_faces_right_on_equal_or_left_head_x() {
        // ASM loc_38158 fallthrough: `st x_direction(a3)` when head_x <= kid_x
        let equal =
            boomerang_boss_head_3_activation_window_loc_38158(0x0080, 0x0080, 0x0080, 0x0080);
        let left =
            boomerang_boss_head_3_activation_window_loc_38158(0x0081, 0x0080, 0x0080, 0x0080);

        assert_eq!(
            (equal.x_direction, left.x_direction),
            (Some(0xFF), Some(0xFF)),
            "ASM loc_38186 should be skipped unless signed `head_x > kid_x`, leaving the fallthrough `$FF` direction",
        );
        assert_eq!(equal.cached_x_direction_fffffb88, Some(0xFF));
        assert_eq!(left.animation_script_label, Some("stru_39FC6"));
    }

    #[test]
    fn boomerang_boss_head_3_attack_wait_loop_loc_3819a_checks_animation_when_status_is_zero() {
        let result = boomerang_boss_head_3_attack_wait_loop_loc_3819A(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_39a7c,
            "ASM loc_3819A should run `sub_39A7C` after hibernating the attack animation frame",
        );
        assert_eq!(
            result.cached_collision_word_fffffb76, 0,
            "ASM loc_3819A should copy zero `d7` into `$FFFFFB76` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3AttackWaitNextLoc3819A::CheckAttackAnimationDone,
            "ASM loc_3819A should continue to the attack animation gate when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_attack_wait_loop_loc_3819a_branches_to_shared_exit_on_status() {
        let result = boomerang_boss_head_3_attack_wait_loop_loc_3819A(0x8000);

        assert_eq!(
            result.cached_collision_word_fffffb76, 0x8000,
            "ASM loc_3819A should preserve nonzero `d7` in `$FFFFFB76`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3AttackWaitNextLoc3819A::SharedCollisionExitLoc39EBE,
            "ASM loc_3819A should branch to loc_39EBE instead of testing the animation byte when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_attack_animation_gate_loc_381ae_loops_until_animation_finishes() {
        let result = boomerang_boss_head_3_attack_animation_gate_loc_381AE(0);

        assert!(!result.attack_animation_done);
        assert!(!result.runs_projectile_spawn_sub_397e4);
        assert_eq!(
            result.next,
            BoomerangBossHead3AttackAnimationNextLoc381AE::AwaitAttackWaitLoopLoc3819A,
            "ASM loc_381AE should branch back to loc_3819A while `$18(a3)` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_attack_animation_gate_loc_381ae_spawns_projectile_when_done() {
        let result = boomerang_boss_head_3_attack_animation_gate_loc_381AE(0x80);

        assert!(result.attack_animation_done);
        assert!(
            result.runs_projectile_spawn_sub_397e4,
            "ASM loc_381AE should call `sub_397E4` as soon as `$18(a3)` is nonzero",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3AttackAnimationNextLoc381AE::SpawnProjectileSub397E4,
        );
    }

    #[test]
    fn boomerang_boss_head_3_post_projectile_wait_loop_loc_381b8_checks_animation_when_status_is_zero()
     {
        let result = boomerang_boss_head_3_post_projectile_wait_loop_loc_381B8(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_39a7c,
            "ASM loc_381B8 should run `sub_39A7C` after hibernating the post-projectile animation frame",
        );
        assert_eq!(
            result.cached_collision_word_fffffb76, 0,
            "ASM loc_381B8 should copy zero `d7` into `$FFFFFB76` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3PostProjectileWaitNextLoc381B8::CheckAnimationDone,
            "ASM loc_381B8 should continue to the post-projectile animation gate when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_post_projectile_wait_loop_loc_381b8_branches_to_shared_exit_on_status()
    {
        let result = boomerang_boss_head_3_post_projectile_wait_loop_loc_381B8(0x0002);

        assert_eq!(
            result.cached_collision_word_fffffb76, 0x0002,
            "ASM loc_381B8 should preserve nonzero `d7` in `$FFFFFB76`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3PostProjectileWaitNextLoc381B8::SharedCollisionExitLoc39EBE,
            "ASM loc_381B8 should branch to loc_39EBE instead of testing `$18(a3)` when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_post_projectile_animation_gate_loc_381cc_loops_until_animation_finishes()
     {
        let result = boomerang_boss_head_3_post_projectile_animation_gate_loc_381CC(0);

        assert!(!result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            BoomerangBossHead3PostProjectileAnimationNextLoc381CC::AwaitPostProjectileWaitLoc381B8,
            "ASM loc_381CC should branch back to loc_381B8 while `$18(a3)` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_post_projectile_animation_gate_loc_381cc_returns_to_activation_when_done()
     {
        let result = boomerang_boss_head_3_post_projectile_animation_gate_loc_381CC(1);

        assert!(result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            BoomerangBossHead3PostProjectileAnimationNextLoc381CC::ReturnToActivationWaitLoc38144,
            "ASM loc_381CC should branch to loc_38144 when the post-projectile animation byte is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_1_init_loc_38494_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_1_init_loc_38494(0x0100, 0x0200);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_38494 should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38494 should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_38494 should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38494 should clear `has_kid_collision(a3)` with `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38494 should set `$13(a3)` (display) with `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_38494 should set `is_moved(a3)` with `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F8-Data_Index",
            "ASM loc_38494 should assign sprite `LnkTo_unk_C85F8-Data_Index`",
        );
        assert_eq!(result.x_pos, 0x0100);
        assert_eq!(result.y_pos, 0x0200);
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_38494 should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_38494 should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_1_wait_loop_loc_384d6_loops_until_all_heads_dead() {
        let result = boomerang_boss_skewer_piece_1_wait_loop_loc_384D6(2);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_step_sub_39846,
            "ASM loc_384D6 should run `sub_39846` (boss head movement step) each frame",
        );
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece1WaitNextLoc384D6::WaitLoopLoc384D6,
            "ASM loc_384D6 should loop back when `$FFFFFB4E` is not 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_1_wait_loop_loc_384d6_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_1_wait_loop_loc_384D6(3);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece1WaitNextLoc384D6::LaunchLoc384E8,
            "ASM loc_384D6 should branch to loc_384E8 when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_1_wait_loop_loc_384d6_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_1_wait_loop_loc_384D6(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece1WaitNextLoc384D6::WaitLoopLoc384D6,
            "ASM loc_384D6 should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_1_launch_loc_384e8_sets_up_left_velocity_and_gravity() {
        let result = boomerang_boss_skewer_piece_1_launch_loc_384E8();

        assert_eq!(
            result.y_vel, -7,
            "ASM loc_384E8 should set `y_vel(a3)` to `$FFF9` (signed word = -7, upward)",
        );
        assert_eq!(
            result.x_vel, -0xE000,
            "ASM loc_384E8 should set `x_vel(a3)` to `$FFFF2000` (signed longword = -0xE000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_384F6 should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_384F6 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38510,
            "ASM loc_38510 should `bra.s loc_384F6` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_2_init_loc_38512_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_2_init_loc_38512(0x0300, 0x0400);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_38512 should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38512 should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_38512 should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38512 should clear `has_kid_collision(a3)` with `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38512 should set `$13(a3)` (display) with `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_38512 should set `is_moved(a3)` with `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F0-Data_Index",
            "ASM loc_38512 should assign sprite `LnkTo_unk_C85F0-Data_Index` (piece 2, differs from piece 1's C85F8)",
        );
        assert_eq!(
            result.x_pos, 0x0300,
            "ASM loc_38512 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0400,
            "ASM loc_38512 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_38512 should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_38512 should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_2_wait_loop_loc_38554_loops_until_all_heads_dead() {
        let result = boomerang_boss_skewer_piece_2_wait_loop_loc_38554(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece2WaitNextLoc38554::WaitLoopLoc38554,
            "ASM loc_38554 should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_2_wait_loop_loc_38554_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_2_wait_loop_loc_38554(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece2WaitNextLoc38554::LaunchLoc38566,
            "ASM loc_38554 should branch to loc_38566 when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_2_wait_loop_loc_38554_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_2_wait_loop_loc_38554(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece2WaitNextLoc38554::WaitLoopLoc38554,
            "ASM loc_38554 should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_2_launch_loc_38566_sets_up_right_velocity_and_gravity() {
        let result = boomerang_boss_skewer_piece_2_launch_loc_38566();

        assert_eq!(
            result.y_vel, -7,
            "ASM loc_38566 should set `y_vel(a3)` to `$FFF9` (signed word = -7, upward)",
        );
        assert_eq!(
            result.x_vel, 0xE000,
            "ASM loc_38566 should set `x_vel(a3)` to `$0000E000` (signed longword = +0xE000, rightward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_38574 should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38574 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_3858e,
            "ASM loc_3858E should `bra.s loc_38574` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_3_init_loc_38590_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_3_init_loc_38590(0x0300, 0x0400);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_38590 should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38590 should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_38590 should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38590 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38590 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38590 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F0-Data_Index",
            "ASM loc_38590 should assign sprite `LnkTo_unk_C85F0-Data_Index` (same as piece 2)",
        );
        assert_eq!(
            result.x_pos, 0x0300,
            "ASM loc_38590 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0400,
            "ASM loc_38590 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_38590 should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_38590 should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_3_wait_loop_loc_385d2_loops_until_all_heads_dead() {
        // ASM loc_385D2 (kid.asm:40804): piece 3 wait loop — hibernate, sub_39846,
        // cmpi.b #3,($FFFFFB4E).w; beq.s loc_385E4; bra.s loc_385D2
        let result = boomerang_boss_skewer_piece_3_wait_loop_loc_385D2(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece3WaitNextLoc385D2::WaitLoopLoc385D2,
            "ASM loc_385D2 should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_3_wait_loop_loc_385d2_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_3_wait_loop_loc_385D2(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece3WaitNextLoc385D2::LaunchLoc385E4,
            "ASM loc_385D2 should branch to loc_385E4 when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_3_wait_loop_loc_385d2_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_3_wait_loop_loc_385D2(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece3WaitNextLoc385D2::WaitLoopLoc385D2,
            "ASM loc_385D2 should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_3_launch_loc_385e4_sets_up_left_velocity_and_gravity() {
        // ASM loc_385E4 (kid.asm:40812): y_vel=$FFF9, x_vel=$FFFF4000 (leftward).
        // loc_385F2 (kid.asm:40816): gravity loop — addi.l #$1B58,y_vel; check
        // y_pos vs Level_height_pixels; delete or loop back via loc_3860C.
        let result = boomerang_boss_skewer_piece_3_launch_loc_385E4();

        assert_eq!(
            result.y_vel, -7,
            "ASM loc_385E4 should set `y_vel(a3)` to `$FFF9` (signed word = -7, upward)",
        );
        assert_eq!(
            result.x_vel, -0xC000,
            "ASM loc_385E4 should set `x_vel(a3)` to `$FFFF4000` (signed longword = -0xC000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_385F2 should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_385F2 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_3860c,
            "ASM loc_3860C should `bra.s loc_385F2` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_4_init_loc_3860e_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_4_init_loc_3860E(0x0300, 0x0400);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_3860E should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_3860E should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_3860E should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_3860E should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_3860E should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_3860E should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F0-Data_Index",
            "ASM loc_3860E should assign sprite `LnkTo_unk_C85F0-Data_Index` (same as pieces 2-3)",
        );
        assert_eq!(
            result.x_pos, 0x0300,
            "ASM loc_3860E should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0400,
            "ASM loc_3860E should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_3860E should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_3860E should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_4_wait_loop_loc_38650_loops_until_all_heads_dead() {
        // ASM loc_38650 (kid.asm:40846): piece 4 wait loop — hibernate, sub_39846,
        // cmpi.b #3,($FFFFFB4E).w; beq.s loc_38662; bra.s loc_38650
        let result = boomerang_boss_skewer_piece_4_wait_loop_loc_38650(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece4WaitNextLoc38650::WaitLoopLoc38650,
            "ASM loc_38650 should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_4_wait_loop_loc_38650_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_4_wait_loop_loc_38650(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece4WaitNextLoc38650::LaunchLoc38662,
            "ASM loc_38650 should branch to loc_38662 when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_4_wait_loop_loc_38650_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_4_wait_loop_loc_38650(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece4WaitNextLoc38650::WaitLoopLoc38650,
            "ASM loc_38650 should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_4_launch_loc_38662_sets_up_right_velocity_and_gravity() {
        // ASM loc_38662 (kid.asm:40854): y_vel=$FFF9, x_vel=$C000 (rightward).
        // loc_38670 (kid.asm:40858): gravity loop — addi.l #$1B58,y_vel; check
        // y_pos vs Level_height_pixels; delete or loop back via loc_3868A.
        let result = boomerang_boss_skewer_piece_4_launch_loc_38662();

        assert_eq!(
            result.y_vel, -7,
            "ASM loc_38662 should set `y_vel(a3)` to `$FFF9` (signed word = -7, upward)",
        );
        assert_eq!(
            result.x_vel, 0xC000,
            "ASM loc_38662 should set `x_vel(a3)` to `$C000` (signed longword = +0xC000, rightward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_38670 should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38670 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_3868a,
            "ASM loc_3868A should `bra.s loc_38670` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_5_init_loc_3868c_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_5_init_loc_3868C(0x0500, 0x0600);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_3868C should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_3868C should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_3868C should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_3868C should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_3868C should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_3868C should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F0-Data_Index",
            "ASM loc_3868C should assign sprite `LnkTo_unk_C85F0-Data_Index` (same as pieces 2-4)",
        );
        assert_eq!(
            result.x_pos, 0x0500,
            "ASM loc_3868C should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0600,
            "ASM loc_3868C should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_3868C should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_3868C should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_5_wait_loop_loc_386ce_loops_until_all_heads_dead() {
        // ASM loc_386CE (kid.asm:40888): piece 5 wait loop — hibernate, sub_39846,
        // cmpi.b #3,($FFFFFB4E).w; beq.s loc_386E0; bra.s loc_386CE
        let result = boomerang_boss_skewer_piece_5_wait_loop_loc_386CE(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece5WaitNextLoc386CE::WaitLoopLoc386CE,
            "ASM loc_386CE should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_5_wait_loop_loc_386ce_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_5_wait_loop_loc_386CE(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece5WaitNextLoc386CE::LaunchLoc386E0,
            "ASM loc_386CE should branch to loc_386E0 when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_5_wait_loop_loc_386ce_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_5_wait_loop_loc_386CE(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece5WaitNextLoc386CE::WaitLoopLoc386CE,
            "ASM loc_386CE should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_5_launch_loc_386e0_sets_up_left_velocity_and_gravity() {
        // ASM loc_386E0 (kid.asm:40896): y_vel=$FFF9, x_vel=$FFFF6000 (leftward).
        // loc_386EE (kid.asm:40900): gravity loop — addi.l #$1B58,y_vel; check
        // y_pos vs Level_height_pixels; delete or loop back via loc_38708.
        let result = boomerang_boss_skewer_piece_5_launch_loc_386E0();

        assert_eq!(
            result.y_vel, -7,
            "ASM loc_386E0 should set `y_vel(a3)` to `$FFF9` (signed word = -7, upward)",
        );
        assert_eq!(
            result.x_vel, -0xA000_i32,
            "ASM loc_386E0 should set `x_vel(a3)` to `$FFFF6000` (signed longword = -0xA000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_386EE should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_386EE should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38708,
            "ASM loc_38708 should `bra.s loc_386EE` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_6_init_loc_3870a_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_6_init_loc_3870A(0x0700, 0x0800);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_3870A should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_3870A should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_3870A should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_3870A should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_3870A should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_3870A should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F0-Data_Index",
            "ASM loc_3870A should assign sprite `LnkTo_unk_C85F0-Data_Index` (same as pieces 2-5)",
        );
        assert_eq!(
            result.x_pos, 0x0700,
            "ASM loc_3870A should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0800,
            "ASM loc_3870A should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_3870A should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_3870A should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_6_wait_loop_loc_3874c_loops_until_all_heads_dead() {
        // ASM loc_3874C (kid.asm:40930): piece 6 wait loop — hibernate, sub_39846,
        // cmpi.b #3,($FFFFFB4E).w; beq.s loc_3875E; bra.s loc_3874C
        let result = boomerang_boss_skewer_piece_6_wait_loop_loc_3874C(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece6WaitNextLoc3874C::WaitLoopLoc3874C,
            "ASM loc_3874C should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_6_wait_loop_loc_3874c_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_6_wait_loop_loc_3874C(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece6WaitNextLoc3874C::LaunchLoc3875E,
            "ASM loc_3874C should branch to loc_3875E when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_6_wait_loop_loc_3874c_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_6_wait_loop_loc_3874C(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece6WaitNextLoc3874C::WaitLoopLoc3874C,
            "ASM loc_3874C should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_6_launch_loc_3875e_sets_up_right_velocity_and_gravity() {
        // ASM loc_3875E (kid.asm:40938): y_vel=$FFF9, x_vel=$A000 (rightward).
        // loc_3876C (kid.asm:40942): gravity loop — addi.l #$1B58,y_vel; check
        // y_pos vs Level_height_pixels; delete or loop back via loc_38786.
        let result = boomerang_boss_skewer_piece_6_launch_loc_3875E();

        assert_eq!(
            result.y_vel, -7,
            "ASM loc_3875E should set `y_vel(a3)` to `$FFF9` (signed word = -7, upward)",
        );
        assert_eq!(
            result.x_vel, 0xA000_i32,
            "ASM loc_3875E should set `x_vel(a3)` to `$A000` (signed longword = +0xA000, rightward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_3876C should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_3876C should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38786,
            "ASM loc_38786 should `bra.s loc_3876C` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_7_init_loc_38788_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_7_init_loc_38788(0x0900, 0x0A00);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_38788 should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38788 should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_38788 should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38788 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38788 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38788 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F0-Data_Index",
            "ASM loc_38788 should assign sprite `LnkTo_unk_C85F0-Data_Index` (same as pieces 2-6)",
        );
        assert_eq!(
            result.x_pos, 0x0900,
            "ASM loc_38788 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0A00,
            "ASM loc_38788 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_38788 should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_38788 should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_7_wait_loop_loc_387ca_loops_until_all_heads_dead() {
        // ASM loc_387CA (kid.asm:40972): piece 7 wait loop — hibernate, sub_39846,
        // cmpi.b #3,($FFFFFB4E).w; beq.s loc_387DC; bra.s loc_387CA
        let result = boomerang_boss_skewer_piece_7_wait_loop_loc_387CA(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece7WaitNextLoc387CA::WaitLoopLoc387CA,
            "ASM loc_387CA should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_7_wait_loop_loc_387ca_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_7_wait_loop_loc_387CA(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece7WaitNextLoc387CA::LaunchLoc387DC,
            "ASM loc_387CA should branch to loc_387DC when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_7_wait_loop_loc_387ca_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_7_wait_loop_loc_387CA(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece7WaitNextLoc387CA::WaitLoopLoc387CA,
            "ASM loc_387CA should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_7_launch_loc_387dc_sets_up_left_velocity_and_gravity() {
        // ASM loc_387DC (kid.asm:40980): y_vel=$FFF9, x_vel=$FFFF8000 (leftward).
        // loc_387EA (kid.asm:40984): gravity loop — addi.l #$1B58,y_vel; check
        // y_pos vs Level_height_pixels; delete or loop back via loc_38804.
        let result = boomerang_boss_skewer_piece_7_launch_loc_387DC();

        assert_eq!(
            result.y_vel, -7,
            "ASM loc_387DC should set `y_vel(a3)` to `$FFF9` (signed word = -7, upward)",
        );
        assert_eq!(
            result.x_vel, -0x8000_i32,
            "ASM loc_387DC should set `x_vel(a3)` to `$FFFF8000` (signed longword = -0x8000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_387EA should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_387EA should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38804,
            "ASM loc_38804 should `bra.s loc_387EA` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_8_init_loc_38806_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_8_init_loc_38806(0x0B00, 0x0C00);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_38806 should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38806 should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_38806 should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38806 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38806 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38806 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85F0-Data_Index",
            "ASM loc_38806 should assign sprite `LnkTo_unk_C85F0-Data_Index` (same as pieces 2-7)",
        );
        assert_eq!(
            result.x_pos, 0x0B00,
            "ASM loc_38806 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0C00,
            "ASM loc_38806 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_38806 should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_38806 should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_8_wait_loop_loc_38848_loops_until_all_heads_dead() {
        // ASM loc_38848 (kid.asm:41014): piece 8 wait loop — hibernate, sub_39846,
        // cmpi.b #3,($FFFFFB4E).w; beq.s loc_3885A; bra.s loc_38848
        let result = boomerang_boss_skewer_piece_8_wait_loop_loc_38848(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece8WaitNextLoc38848::WaitLoopLoc38848,
            "ASM loc_38848 should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_8_wait_loop_loc_38848_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_8_wait_loop_loc_38848(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece8WaitNextLoc38848::LaunchLoc3885A,
            "ASM loc_38848 should branch to loc_3885A when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_8_wait_loop_loc_38848_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_8_wait_loop_loc_38848(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece8WaitNextLoc38848::WaitLoopLoc38848,
            "ASM loc_38848 should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_8_launch_loc_3885a_sets_up_right_velocity_and_gravity() {
        // ASM loc_3885A (kid.asm:41022): y_vel=$FFF8, x_vel=$8000 (rightward).
        // loc_38868 (kid.asm:41026): gravity loop — addi.l #$1B58,y_vel; check
        // y_pos vs Level_height_pixels; delete or loop back via loc_38882.
        let result = boomerang_boss_skewer_piece_8_launch_loc_3885A();

        assert_eq!(
            result.y_vel, -8,
            "ASM loc_3885A should set `y_vel(a3)` to `$FFF8` (signed word = -8, upward)",
        );
        assert_eq!(
            result.x_vel, 0x8000_i32,
            "ASM loc_3885A should set `x_vel(a3)` to `$00008000` (signed longword = 0x8000, rightward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_38868 should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38868 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38882,
            "ASM loc_38882 should `bra.s loc_38868` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_9_init_loc_38884_sets_gfx_slot_and_sprite() {
        let result = boomerang_boss_skewer_piece_9_init_loc_38884(0x0D00, 0x0E00);

        assert_eq!(
            result.gfx_slot, 0x0101_0002,
            "ASM loc_38884 should allocate gfx slot `$01010002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38884 should set `priority(a3)` to 1",
        );
        assert_eq!(
            result.object_meta, 0x22,
            "ASM loc_38884 should set `object_meta(a3)` to `$22`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38884 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38884 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38884 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C85E8-Data_Index",
            "ASM loc_38884 should assign sprite `LnkTo_unk_C85E8-Data_Index` (differs from pieces 2-8 C85F0)",
        );
        assert_eq!(
            result.x_pos, 0x0D00,
            "ASM loc_38884 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0E00,
            "ASM loc_38884 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.movement_script_label, "unk_39872",
            "ASM loc_38884 should load movement script `unk_39872` into `a2`",
        );
        assert_eq!(
            result.movement_script_index, 1,
            "ASM loc_38884 should start the movement script at `moveq #1,d0`",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_9_wait_loop_loc_388c6_loops_until_all_heads_dead() {
        // ASM loc_388C6 (kid.asm:41056): piece 9 wait loop — hibernate, sub_39846,
        // cmpi.b #3,($FFFFFB4E).w; beq.s loc_388D8; bra.s loc_388C6
        let result = boomerang_boss_skewer_piece_9_wait_loop_loc_388C6(2);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece9WaitNextLoc388C6::WaitLoopLoc388C6,
            "ASM loc_388C6 should loop back when `$FFFFFB4E` != 3 (only 2 heads dead)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_9_wait_loop_loc_388c6_launches_when_all_3_heads_dead() {
        let result = boomerang_boss_skewer_piece_9_wait_loop_loc_388C6(3);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_movement_step_sub_39846);
        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece9WaitNextLoc388C6::LaunchLoc388D8,
            "ASM loc_388C6 should branch to loc_388D8 when `cmpi.b #3,($FFFFFB4E).w` is equal",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_9_wait_loop_loc_388c6_does_not_launch_at_zero() {
        let result = boomerang_boss_skewer_piece_9_wait_loop_loc_388C6(0);

        assert_eq!(
            result.next,
            BoomerangBossSkewerPiece9WaitNextLoc388C6::WaitLoopLoc388C6,
            "ASM loc_388C6 should loop when no heads have died yet ($FFFFFB4E == 0)",
        );
    }

    #[test]
    fn boomerang_boss_skewer_piece_9_launch_loc_388d8_sets_up_left_velocity_and_gravity() {
        // ASM loc_388D8 (kid.asm:41064): y_vel=$FFFA, x_vel=$FFFF6000 (leftward).
        // loc_388E6 (kid.asm:41068): gravity loop — addi.l #$1B58,y_vel; check
        // y_pos vs Level_height_pixels; delete or loop back via loc_38900.
        let result = boomerang_boss_skewer_piece_9_launch_loc_388D8();

        assert_eq!(
            result.y_vel, -6,
            "ASM loc_388D8 should set `y_vel(a3)` to `$FFFA` (signed word = -6, upward)",
        );
        assert_eq!(
            result.x_vel, -0xA000_i32,
            "ASM loc_388D8 should set `x_vel(a3)` to `$FFFF6000` (signed longword = -0xA000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x1B58,
            "ASM loc_388E6 should add `$1B58` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_388E6 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38900,
            "ASM loc_38900 should `bra.s loc_388E6` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_1_init_loc_38902_sets_gfx_slot_and_sprite() {
        // ASM loc_38902 (kid.asm:41081): allocate eye piece 1 with gfx slot
        // $01020002, priority 0, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_393C8).
        let result = boomerang_boss_eye_piece_1_init_loc_38902(0x0A00, 0x0B00);

        assert_eq!(
            result.gfx_slot, 0x0102_0002,
            "ASM loc_38902 should allocate gfx slot `$01020002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_38902 should set `priority(a3)` to 0",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38902 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38902 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38902 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38902 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38902 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0A00,
            "ASM loc_38902 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0B00,
            "ASM loc_38902 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38902 should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_1_tracking_loop_loc_3893e_loops_when_breakup_is_zero() {
        // ASM loc_3893E (kid.asm:41096): hibernate, sub_393C8, sub_37A22,
        // tst.w ($FFFFFB72).w; bne.s loc_38962; move.b $16(a2),d0; bra.s loc_3893E
        let result = boomerang_boss_eye_piece_1_tracking_loop_loc_3893E(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_393c8);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossEyePiece1TrackingNextLoc3893E::TrackingLoopLoc3893E,
            "ASM loc_3893E should loop back when `$FFFFFB72` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_1_tracking_loop_loc_3893e_launches_when_breakup_nonzero() {
        let result = boomerang_boss_eye_piece_1_tracking_loop_loc_3893E(1);

        assert_eq!(
            result.next,
            BoomerangBossEyePiece1TrackingNextLoc3893E::LaunchLoc38962,
            "ASM loc_3893E should branch to `loc_38962` when `tst.w ($FFFFFB72).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_1_tracking_loop_loc_3893e_launches_at_large_breakup_value() {
        let result = boomerang_boss_eye_piece_1_tracking_loop_loc_3893E(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossEyePiece1TrackingNextLoc3893E::LaunchLoc38962,
            "ASM loc_3893E should launch for any nonzero `$FFFFFB72` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_1_launch_loc_38962_sets_leftward_velocity_and_gravity() {
        // ASM loc_38962 (kid.asm:41109): sub_393B2, y_vel=$FFFD, x_vel=$FFFFC000.
        // loc_38976 (kid.asm:41114): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38998.
        let result = boomerang_boss_eye_piece_1_launch_loc_38962();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38962 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38962 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_38962 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38976 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38976 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38976 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38998,
            "ASM loc_38998 should `bra.s loc_38976` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_2_init_loc_3899a_sets_gfx_slot_and_sprite() {
        // ASM loc_3899A (kid.asm:41129): allocate eye piece 2 with gfx slot
        // $01030002, priority 0, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_39420).
        let result = boomerang_boss_eye_piece_2_init_loc_3899A(0x0A00, 0x0B00);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_3899A should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_3899A should set `priority(a3)` to 0",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_3899A should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_3899A should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_3899A should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_3899A should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_3899A should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0A00,
            "ASM loc_3899A should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0B00,
            "ASM loc_3899A should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_3899A should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_2_tracking_loop_loc_389d6_loops_when_breakup_is_zero() {
        // ASM loc_389D6 (kid.asm:41144): hibernate, sub_39420, sub_37A22,
        // tst.w ($FFFFFB72).w; bne.s loc_389FA; move.b $16(a2),d0; bra.s loc_389D6
        let result = boomerang_boss_eye_piece_2_tracking_loop_loc_389D6(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_39420);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossEyePiece2TrackingNextLoc389D6::TrackingLoopLoc389D6,
            "ASM loc_389D6 should loop back when `$FFFFFB72` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_2_tracking_loop_loc_389d6_launches_when_breakup_nonzero() {
        let result = boomerang_boss_eye_piece_2_tracking_loop_loc_389D6(1);

        assert_eq!(
            result.next,
            BoomerangBossEyePiece2TrackingNextLoc389D6::LaunchLoc389FA,
            "ASM loc_389D6 should branch to `loc_389FA` when `tst.w ($FFFFFB72).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_2_tracking_loop_loc_389d6_launches_at_large_breakup_value() {
        let result = boomerang_boss_eye_piece_2_tracking_loop_loc_389D6(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossEyePiece2TrackingNextLoc389D6::LaunchLoc389FA,
            "ASM loc_389D6 should launch for any nonzero `$FFFFFB72` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_eye_piece_2_launch_loc_389fa_sets_rightward_velocity_and_gravity() {
        // ASM loc_389FA (kid.asm:41157): sub_393B2, y_vel=$FFFD, x_vel=$4000.
        // loc_38A0E (kid.asm:41162): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38A30.
        let result = boomerang_boss_eye_piece_2_launch_loc_389FA();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_389FA should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_389FA should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_389FA should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38A0E should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38A0E should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38A0E should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38a30,
            "ASM loc_38A30 should `bra.s loc_38A0E` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_1_init_loc_38a32_sets_gfx_slot_and_sprite() {
        // ASM loc_38A32 (kid.asm:41177): allocate head 2's eye piece 1 with gfx slot
        // $01020002, priority 0, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_393C8).
        let result = boomerang_boss_head_2_eye_piece_1_init_loc_38A32(0x0A00, 0x0B00);

        assert_eq!(
            result.gfx_slot, 0x0102_0002,
            "ASM loc_38A32 should allocate gfx slot `$01020002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_38A32 should set `priority(a3)` to 0",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38A32 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38A32 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38A32 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38A32 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38A32 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0A00,
            "ASM loc_38A32 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0B00,
            "ASM loc_38A32 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38A32 should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_1_tracking_loop_loc_38a6e_loops_when_breakup_is_zero() {
        // ASM loc_38A6E (kid.asm:41192): hibernate, sub_393C8, sub_37A22,
        // tst.w ($FFFFFB74).w; bne.s loc_38A92; move.b $16(a2),d0; bra.s loc_38A6E
        let result = boomerang_boss_head_2_eye_piece_1_tracking_loop_loc_38A6E(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_393c8);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece1TrackingNextLoc38A6E::TrackingLoopLoc38A6E,
            "ASM loc_38A6E should loop back when `$FFFFFB74` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_1_tracking_loop_loc_38a6e_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_2_eye_piece_1_tracking_loop_loc_38A6E(1);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece1TrackingNextLoc38A6E::LaunchLoc38A92,
            "ASM loc_38A6E should branch to `loc_38A92` when `tst.w ($FFFFFB74).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_1_tracking_loop_loc_38a6e_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_2_eye_piece_1_tracking_loop_loc_38A6E(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece1TrackingNextLoc38A6E::LaunchLoc38A92,
            "ASM loc_38A6E should launch for any nonzero `$FFFFFB74` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_1_launch_loc_38a92_sets_leftward_velocity_and_gravity() {
        // ASM loc_38A92 (kid.asm:41205): sub_393B2, y_vel=$FFFD, x_vel=$FFFFC000.
        // loc_38AA6 (kid.asm:41210): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38AC8.
        let result = boomerang_boss_head_2_eye_piece_1_launch_loc_38A92();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38A92 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38A92 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_38A92 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38AA6 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38AA6 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38AA6 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38ac8,
            "ASM loc_38AC8 should `bra.s loc_38AA6` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_2_init_loc_38aca_sets_gfx_slot_and_sprite() {
        // ASM loc_38ACA (kid.asm:41225): allocate head 2's eye piece 2 with gfx slot
        // $01030002, priority 0, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_39420).
        let result = boomerang_boss_head_2_eye_piece_2_init_loc_38ACA(0x0C00, 0x0D00);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_38ACA should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_38ACA should set `priority(a3)` to 0",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38ACA should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38ACA should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38ACA should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38ACA should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38ACA should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0C00,
            "ASM loc_38ACA should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0D00,
            "ASM loc_38ACA should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38ACA should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_2_tracking_loop_loc_38b06_loops_when_breakup_is_zero() {
        // ASM loc_38B06 (kid.asm:41240): hibernate, sub_39420, sub_37A22,
        // tst.w ($FFFFFB74).w; bne.s loc_38B2A; move.b $16(a2),d0; bra.s loc_38B06
        let result = boomerang_boss_head_2_eye_piece_2_tracking_loop_loc_38B06(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_39420);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece2TrackingNextLoc38B06::TrackingLoopLoc38B06,
            "ASM loc_38B06 should loop back when `$FFFFFB74` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_2_tracking_loop_loc_38b06_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_2_eye_piece_2_tracking_loop_loc_38B06(1);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece2TrackingNextLoc38B06::LaunchLoc38B2A,
            "ASM loc_38B06 should branch to `loc_38B2A` when `tst.w ($FFFFFB74).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_2_tracking_loop_loc_38b06_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_2_eye_piece_2_tracking_loop_loc_38B06(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece2TrackingNextLoc38B06::LaunchLoc38B2A,
            "ASM loc_38B06 should launch for any nonzero `$FFFFFB74` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_2_launch_loc_38b2a_sets_rightward_velocity_and_gravity() {
        // ASM loc_38B2A (kid.asm:41253): sub_393B2, y_vel=$FFFD, x_vel=$4000.
        // loc_38B3E (kid.asm:41258): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38B60.
        let result = boomerang_boss_head_2_eye_piece_2_launch_loc_38B2A();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38B2A should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38B2A should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_38B2A should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38B3E should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38B3E should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38B3E should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38b60,
            "ASM loc_38B60 should `bra.s loc_38B3E` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_1_init_loc_38b62_sets_gfx_slot_and_sprite() {
        // ASM loc_38B62 (kid.asm:41273): allocate head 3's eye piece 1 with gfx slot
        // $01020002, priority 0, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_393C8).
        let result = boomerang_boss_head_3_eye_piece_1_init_loc_38B62(0x0E00, 0x0F00);

        assert_eq!(
            result.gfx_slot, 0x0102_0002,
            "ASM loc_38B62 should allocate gfx slot `$01020002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_38B62 should set `priority(a3)` to 0",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38B62 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38B62 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38B62 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38B62 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38B62 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0E00,
            "ASM loc_38B62 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0F00,
            "ASM loc_38B62 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38B62 should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_1_tracking_loop_loc_38b9e_loops_when_breakup_is_zero() {
        // ASM loc_38B9E (kid.asm:41288): hibernate, sub_393C8, sub_37A22,
        // tst.w ($FFFFFB76).w; bne.s loc_38BC2; move.b $16(a2),d0; bra.s loc_38B9E
        let result = boomerang_boss_head_3_eye_piece_1_tracking_loop_loc_38B9E(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_393c8);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece1TrackingNextLoc38B9E::TrackingLoopLoc38B9E,
            "ASM loc_38B9E should loop back when `$FFFFFB76` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_1_tracking_loop_loc_38b9e_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_3_eye_piece_1_tracking_loop_loc_38B9E(1);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece1TrackingNextLoc38B9E::LaunchLoc38BC2,
            "ASM loc_38B9E should branch to `loc_38BC2` when `tst.w ($FFFFFB76).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_1_tracking_loop_loc_38b9e_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_3_eye_piece_1_tracking_loop_loc_38B9E(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece1TrackingNextLoc38B9E::LaunchLoc38BC2,
            "ASM loc_38B9E should launch for any nonzero `$FFFFFB76` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_1_launch_loc_38bc2_sets_leftward_velocity_and_gravity() {
        // ASM loc_38BC2 (kid.asm:41301): sub_393B2, y_vel=$FFFD, x_vel=#-$4000.
        // loc_38BD6 (kid.asm:41306): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38BF8.
        let result = boomerang_boss_head_3_eye_piece_1_launch_loc_38BC2();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38BC2 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38BC2 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_38BC2 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38BD6 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38BD6 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38BD6 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38bf8,
            "ASM loc_38BF8 should `bra.s loc_38BD6` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_2_init_loc_38bfa_sets_gfx_slot_and_sprite() {
        // ASM loc_38BFA (kid.asm:41321): allocate head 3's eye piece 2 with gfx slot
        // $01030002, priority 0, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_39420).
        let result = boomerang_boss_head_3_eye_piece_2_init_loc_38BFA(0x0E00, 0x0F00);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_38BFA should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_38BFA should set `priority(a3)` to 0",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38BFA should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38BFA should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38BFA should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38BFA should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38BFA should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0E00,
            "ASM loc_38BFA should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0F00,
            "ASM loc_38BFA should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38BFA should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_2_tracking_loop_loc_38c36_loops_when_breakup_is_zero() {
        // ASM loc_38C36 (kid.asm:41336): hibernate, sub_39420, sub_37A22,
        // tst.w ($FFFFFB76).w; bne.s loc_38C5A; move.b $16(a2),d0; bra.s loc_38C36
        let result = boomerang_boss_head_3_eye_piece_2_tracking_loop_loc_38C36(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_39420);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece2TrackingNextLoc38C36::TrackingLoopLoc38C36,
            "ASM loc_38C36 should loop back when `$FFFFFB76` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_2_tracking_loop_loc_38c36_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_3_eye_piece_2_tracking_loop_loc_38C36(1);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece2TrackingNextLoc38C36::LaunchLoc38C5A,
            "ASM loc_38C36 should branch to `loc_38C5A` when `tst.w ($FFFFFB76).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_2_tracking_loop_loc_38c36_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_3_eye_piece_2_tracking_loop_loc_38C36(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece2TrackingNextLoc38C36::LaunchLoc38C5A,
            "ASM loc_38C36 should launch for any nonzero `$FFFFFB76` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_2_launch_loc_38c5a_sets_rightward_velocity_and_gravity() {
        // ASM loc_38C5A (kid.asm:41349): sub_393B2, y_vel=$FFFD, x_vel=#$4000.
        // loc_38C6E (kid.asm:41354): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38C90.
        let result = boomerang_boss_head_3_eye_piece_2_launch_loc_38C5A();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38C5A should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38C5A should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_38C5A should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38C6E should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38C6E should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38C6E should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38c90,
            "ASM loc_38C90 should `bra.s loc_38C6E` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_3_init_loc_38c92_sets_gfx_slot_and_priority_1() {
        // ASM loc_38C92 (kid.asm:41369): allocate head 1's eye piece 3 with gfx slot
        // $01030002, priority 1, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_393C8).
        let result = boomerang_boss_head_1_eye_piece_3_init_loc_38C92(0x0A00, 0x0B00);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_38C92 should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38C92 should set `priority(a3)` to 1 (unlike priority-0 pieces at loc_38902/loc_3899A)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38C92 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38C92 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38C92 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38C92 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38C92 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0A00,
            "ASM loc_38C92 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0B00,
            "ASM loc_38C92 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38C92 should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_3_tracking_loop_loc_38cce_loops_when_breakup_is_zero() {
        // ASM loc_38CCE (kid.asm:41384): hibernate, sub_393C8, sub_37A22,
        // tst.w ($FFFFFB72).w; bne.s loc_38CF2; move.b $16(a2),d0; bra.s loc_38CCE
        let result = boomerang_boss_head_1_eye_piece_3_tracking_loop_loc_38CCE(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_393c8);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece3TrackingNextLoc38CCE::TrackingLoopLoc38CCE,
            "ASM loc_38CCE should loop back when `$FFFFFB72` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_3_tracking_loop_loc_38cce_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_1_eye_piece_3_tracking_loop_loc_38CCE(1);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece3TrackingNextLoc38CCE::LaunchLoc38CF2,
            "ASM loc_38CCE should branch to `loc_38CF2` when `tst.w ($FFFFFB72).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_3_tracking_loop_loc_38cce_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_1_eye_piece_3_tracking_loop_loc_38CCE(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece3TrackingNextLoc38CCE::LaunchLoc38CF2,
            "ASM loc_38CCE should launch for any nonzero `$FFFFFB72` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_3_launch_loc_38cf2_sets_leftward_velocity_and_gravity() {
        // ASM loc_38CF2 (kid.asm:41397): sub_393B2, y_vel=$FFFD, x_vel=#$FFFFC000.
        // loc_38D06 (kid.asm:41402): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38D28.
        let result = boomerang_boss_head_1_eye_piece_3_launch_loc_38CF2();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38CF2 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38CF2 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_38CF2 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38D06 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38D06 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38D06 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38d28,
            "ASM loc_38D28 should `bra.s loc_38D06` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_4_init_loc_38d2a_sets_gfx_slot_and_priority_1() {
        // ASM loc_38D2A (kid.asm:41417): allocate head 1's eye piece 4 with gfx slot
        // $01030002, priority 1, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_39420).
        let result = boomerang_boss_head_1_eye_piece_4_init_loc_38D2A(0x0A00, 0x0B00);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_38D2A should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38D2A should set `priority(a3)` to 1 (secondary wave, same as piece 3 at loc_38C92)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38D2A should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38D2A should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38D2A should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38D2A should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38D2A should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0A00,
            "ASM loc_38D2A should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0B00,
            "ASM loc_38D2A should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38D2A should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_4_tracking_loop_loc_38d66_loops_when_breakup_is_zero() {
        // ASM loc_38D66 (kid.asm:41432): hibernate, sub_39420, sub_37A22,
        // tst.w ($FFFFFB72).w; bne.s loc_38D8A; move.b $16(a2),d0; bra.s loc_38D66
        let result = boomerang_boss_head_1_eye_piece_4_tracking_loop_loc_38D66(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_39420);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece4TrackingNextLoc38D66::TrackingLoopLoc38D66,
            "ASM loc_38D66 should loop back when `$FFFFFB72` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_4_tracking_loop_loc_38d66_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_1_eye_piece_4_tracking_loop_loc_38D66(1);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece4TrackingNextLoc38D66::LaunchLoc38D8A,
            "ASM loc_38D66 should branch to `loc_38D8A` when `tst.w ($FFFFFB72).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_4_tracking_loop_loc_38d66_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_1_eye_piece_4_tracking_loop_loc_38D66(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece4TrackingNextLoc38D66::LaunchLoc38D8A,
            "ASM loc_38D66 should launch for any nonzero `$FFFFFB72` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_4_launch_loc_38d8a_sets_rightward_velocity_and_gravity() {
        // ASM loc_38D8A (kid.asm:41445): sub_393B2, y_vel=$FFFD, x_vel=#$4000.
        // loc_38D9E (kid.asm:41450): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38DC0.
        let result = boomerang_boss_head_1_eye_piece_4_launch_loc_38D8A();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38D8A should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38D8A should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_38D8A should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward — mirrors piece 3's leftward $FFFFC000)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38D9E should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38D9E should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38D9E should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38dc0,
            "ASM loc_38DC0 should `bra.s loc_38D9E` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_3_init_loc_38dc2_sets_gfx_slot_and_priority_1() {
        // ASM loc_38DC2 (kid.asm:41465): allocate head 2's eye piece 3 with gfx slot
        // $01030002, priority 1, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_393C8).
        let result = boomerang_boss_head_2_eye_piece_3_init_loc_38DC2(0x0C00, 0x0D00);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_38DC2 should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38DC2 should set `priority(a3)` to 1 (secondary wave, same as head 1's piece 3 at loc_38C92)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38DC2 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38DC2 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38DC2 should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38DC2 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38DC2 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0C00,
            "ASM loc_38DC2 should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0D00,
            "ASM loc_38DC2 should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38DC2 should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_3_tracking_loop_loc_38dfe_loops_when_breakup_is_zero() {
        // ASM loc_38DFE (kid.asm:41480): hibernate, sub_393C8, sub_37A22,
        // tst.w ($FFFFFB74).w; bne.s loc_38E22; move.b $16(a2),d0; bra.s loc_38DFE
        let result = boomerang_boss_head_2_eye_piece_3_tracking_loop_loc_38DFE(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_393c8);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece3TrackingNextLoc38DFE::TrackingLoopLoc38DFE,
            "ASM loc_38DFE should loop back when `$FFFFFB74` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_3_tracking_loop_loc_38dfe_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_2_eye_piece_3_tracking_loop_loc_38DFE(1);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece3TrackingNextLoc38DFE::LaunchLoc38E22,
            "ASM loc_38DFE should branch to `loc_38E22` when `tst.w ($FFFFFB74).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_3_tracking_loop_loc_38dfe_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_2_eye_piece_3_tracking_loop_loc_38DFE(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece3TrackingNextLoc38DFE::LaunchLoc38E22,
            "ASM loc_38DFE should launch for any nonzero `$FFFFFB74` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_3_launch_loc_38e22_sets_leftward_velocity_and_gravity() {
        // ASM loc_38E22 (kid.asm:41493): sub_393B2, y_vel=$FFFD, x_vel=#$FFFFC000.
        // loc_38E36 (kid.asm:41498): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38E58.
        let result = boomerang_boss_head_2_eye_piece_3_launch_loc_38E22();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38E22 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38E22 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_38E22 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward — same as head 2's piece 1 at loc_38A92)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38E36 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38E36 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38E36 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38e58,
            "ASM loc_38E58 should `bra.s loc_38E36` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_4_init_loc_38e5a_sets_gfx_slot_and_priority_1() {
        // ASM loc_38E5A (kid.asm:41513): allocate head 2's eye piece 4 with gfx slot
        // $01030002, priority 1, object_meta $21, sprite LnkTo_unk_C8488,
        // anchor from $44/$46(a5), sf d0 (initial frame byte for sub_39420).
        let result = boomerang_boss_head_2_eye_piece_4_init_loc_38E5A(0x0C00, 0x0D00);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_38E5A should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38E5A should set `priority(a3)` to 1 (secondary wave, same as piece 3 at loc_38DC2)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38E5A should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38E5A should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38E5A should set `$13(a3)` via `st` (display enabled)",
        );
        assert!(
            result.is_moved,
            "ASM loc_38E5A should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38E5A should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0C00,
            "ASM loc_38E5A should copy anchor X from `$44(a5)`",
        );
        assert_eq!(
            result.y_pos, 0x0D00,
            "ASM loc_38E5A should copy anchor Y from `$46(a5)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38E5A should clear `d0` via `sf` as the initial frame byte for orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_4_tracking_loop_loc_38e96_loops_when_breakup_is_zero() {
        // ASM loc_38E96 (kid.asm:41528): hibernate, sub_39420, sub_37A22,
        // tst.w ($FFFFFB74).w; bne.s loc_38EBA; move.b $16(a2),d0; bra.s loc_38E96
        let result = boomerang_boss_head_2_eye_piece_4_tracking_loop_loc_38E96(0);

        assert!(result.hibernates_one_frame);
        assert!(result.runs_orbit_placement_sub_39420);
        assert!(result.runs_directional_sprite_sub_37a22);
        assert!(result.updates_sprite_from_d6);
        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece4TrackingNextLoc38E96::TrackingLoopLoc38E96,
            "ASM loc_38E96 should loop back when `$FFFFFB74` is zero (no breakup trigger)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_4_tracking_loop_loc_38e96_launches_when_breakup_nonzero() {
        let result = boomerang_boss_head_2_eye_piece_4_tracking_loop_loc_38E96(1);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece4TrackingNextLoc38E96::LaunchLoc38EBA,
            "ASM loc_38E96 should branch to `loc_38EBA` when `tst.w ($FFFFFB74).w` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_4_tracking_loop_loc_38e96_launches_at_large_breakup_value() {
        let result = boomerang_boss_head_2_eye_piece_4_tracking_loop_loc_38E96(0x00FF);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece4TrackingNextLoc38E96::LaunchLoc38EBA,
            "ASM loc_38E96 should launch for any nonzero `$FFFFFB74` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_4_launch_loc_38eba_sets_rightward_velocity_and_gravity() {
        // ASM loc_38EBA (kid.asm:41541): sub_393B2, y_vel=$FFFD, x_vel=#$4000.
        // loc_38ECE (kid.asm:41546): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38EF0.
        let result = boomerang_boss_head_2_eye_piece_4_launch_loc_38EBA();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38EBA should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38EBA should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_38EBA should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward — mirrors piece 3's leftward at loc_38E22)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38ECE should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38ECE should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38ECE should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38ef0,
            "ASM loc_38EF0 should `bra.s loc_38ECE` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_3_init_loc_38ef2_allocates_priority_1_with_sub_393c8() {
        // ASM loc_38EF2 (kid.asm:41561): head 3 eye piece 3 init — $01030002,
        // priority 1 (secondary wave), sub_393C8 orbit.
        let result = boomerang_boss_head_3_eye_piece_3_init_loc_38EF2(0x0120, 0x0080);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_38EF2 should load gfx slot `$01030002` (same as other priority-1 eye pieces)",
        );
        assert_eq!(
            result.priority, 1,
            "ASM loc_38EF2 should set `priority(a3)` to 1 (secondary wave)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_38EF2 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_38EF2 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_38EF2 should set `$13(a3)` (display enabled) via `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_38EF2 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_38EF2 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0120,
            "ASM loc_38EF2 should copy `$44(a5)` into `x_pos(a3)`",
        );
        assert_eq!(
            result.y_pos, 0x0080,
            "ASM loc_38EF2 should copy `$46(a5)` into `y_pos(a3)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_38EF2 should clear `d0` via `sf` (initial frame byte for sub_393C8)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_3_tracking_loop_loc_38f2e_loops_when_breakup_zero() {
        // ASM loc_38F2E (kid.asm:41576): tracking loop — sub_393C8 + sub_37A22,
        // tst.w ($FFFFFB76) == 0 → loop back.
        let result = boomerang_boss_head_3_eye_piece_3_tracking_loop_loc_38F2E(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_393c8,
            "ASM loc_38F2E should call `sub_393C8` for orbit placement (head 3's piece 3)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_38F2E should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_38F2E should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece3TrackingNextLoc38F2E::TrackingLoopLoc38F2E,
            "ASM loc_38F2E should loop back when `$FFFFFB76` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_3_tracking_loop_loc_38f2e_launches_when_breakup_nonzero() {
        // ASM loc_38F2E: tst.w ($FFFFFB76) != 0 → bne.s loc_38F52 (launch).
        let result = boomerang_boss_head_3_eye_piece_3_tracking_loop_loc_38F2E(1);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece3TrackingNextLoc38F2E::LaunchLoc38F52,
            "ASM loc_38F2E should branch to loc_38F52 (launch) when `$FFFFFB76` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_3_tracking_loop_loc_38f2e_launches_for_any_nonzero() {
        // ASM loc_38F2E: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_3_eye_piece_3_tracking_loop_loc_38F2E(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece3TrackingNextLoc38F2E::LaunchLoc38F52,
            "ASM loc_38F2E should launch for any nonzero `$FFFFFB76` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_3_launch_loc_38f52_sets_leftward_velocity_and_gravity() {
        // ASM loc_38F52 (kid.asm:41589): sub_393B2, y_vel=$FFFD, x_vel=#$FFFFC000.
        // loc_38F66 (kid.asm:41594): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_38F88.
        let result = boomerang_boss_head_3_eye_piece_3_launch_loc_38F52();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38F52 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38F52 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_38F52 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward — mirrors piece 4's rightward at loc_38FEA)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38F66 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38F66 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38F66 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_38f88,
            "ASM loc_38F88 should `bra.s loc_38F66` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_4_tracking_loop_loc_38fc6_loops_when_breakup_zero() {
        // ASM loc_38FC6 (kid.asm:41624): tracking loop — sub_39420 + sub_37A22,
        // tst.w ($FFFFFB76) == 0 → loop back.
        let result = boomerang_boss_head_3_eye_piece_4_tracking_loop_loc_38FC6(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_39420,
            "ASM loc_38FC6 should call `sub_39420` for orbit placement (head 3's piece 4, mirrors piece 3's sub_393C8)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_38FC6 should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_38FC6 should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece4TrackingNextLoc38FC6::TrackingLoopLoc38FC6,
            "ASM loc_38FC6 should loop back when `$FFFFFB76` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_4_tracking_loop_loc_38fc6_launches_when_breakup_nonzero() {
        // ASM loc_38FC6: tst.w ($FFFFFB76) != 0 → bne.s loc_38FEA (launch).
        let result = boomerang_boss_head_3_eye_piece_4_tracking_loop_loc_38FC6(1);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece4TrackingNextLoc38FC6::LaunchLoc38FEA,
            "ASM loc_38FC6 should branch to loc_38FEA (launch) when `$FFFFFB76` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_4_tracking_loop_loc_38fc6_launches_for_any_nonzero() {
        // ASM loc_38FC6: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_3_eye_piece_4_tracking_loop_loc_38FC6(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece4TrackingNextLoc38FC6::LaunchLoc38FEA,
            "ASM loc_38FC6 should launch for any nonzero `$FFFFFB76` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_4_launch_loc_38fea_sets_rightward_velocity_and_gravity() {
        // ASM loc_38FEA (kid.asm:41637): sub_393B2, y_vel=$FFFD, x_vel=#$4000.
        // loc_38FFE (kid.asm:41642): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_39020.
        let result = boomerang_boss_head_3_eye_piece_4_launch_loc_38FEA();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_38FEA should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_38FEA should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_38FEA should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward — mirrors piece 3's leftward at loc_38F52)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_38FFE should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_38FFE should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_38FFE should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_39020,
            "ASM loc_39020 should `bra.s loc_38FFE` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_5_init_loc_39022_allocates_priority_0_with_sub_393c8() {
        // ASM loc_39022 (kid.asm:41657): head 1 eye piece 5 init — $01030002,
        // priority 0 (tertiary wave), sub_393C8 orbit.
        let result = boomerang_boss_head_1_eye_piece_5_init_loc_39022(0x0120, 0x0080);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_39022 should load gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_39022 should set `priority(a3)` to 0 (tertiary wave)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_39022 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_39022 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_39022 should set `$13(a3)` (display enabled) via `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_39022 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_39022 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0120,
            "ASM loc_39022 should copy `$44(a5)` into `x_pos(a3)`",
        );
        assert_eq!(
            result.y_pos, 0x0080,
            "ASM loc_39022 should copy `$46(a5)` into `y_pos(a3)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_39022 should clear `d0` via `sf` (initial frame byte for sub_393C8)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_5_tracking_loop_loc_3905e_loops_when_breakup_zero() {
        // ASM loc_3905E (kid.asm:41672): tracking loop — sub_393C8 + sub_37A22,
        // tst.w ($FFFFFB72) == 0 → loop back.
        let result = boomerang_boss_head_1_eye_piece_5_tracking_loop_loc_3905E(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_393c8,
            "ASM loc_3905E should call `sub_393C8` for orbit placement (head 1's piece 5)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_3905E should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_3905E should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece5TrackingNextLoc3905E::TrackingLoopLoc3905E,
            "ASM loc_3905E should loop back when `$FFFFFB72` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_5_tracking_loop_loc_3905e_launches_when_breakup_nonzero() {
        // ASM loc_3905E: tst.w ($FFFFFB72) != 0 → bne.s loc_39082 (launch).
        let result = boomerang_boss_head_1_eye_piece_5_tracking_loop_loc_3905E(1);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece5TrackingNextLoc3905E::LaunchLoc39082,
            "ASM loc_3905E should branch to loc_39082 (launch) when `$FFFFFB72` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_5_tracking_loop_loc_3905e_launches_for_any_nonzero() {
        // ASM loc_3905E: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_1_eye_piece_5_tracking_loop_loc_3905E(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece5TrackingNextLoc3905E::LaunchLoc39082,
            "ASM loc_3905E should launch for any nonzero `$FFFFFB72` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_5_launch_loc_39082_sets_leftward_velocity_and_gravity() {
        // ASM loc_39082 (kid.asm:41685): sub_393B2, y_vel=$FFFD, x_vel=#$FFFFC000.
        // loc_39096 (kid.asm:41690): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_390B8.
        let result = boomerang_boss_head_1_eye_piece_5_launch_loc_39082();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_39082 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_39082 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_39082 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward — mirrors piece 6's rightward at loc_390BA)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_39096 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_39096 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_39096 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_390b8,
            "ASM loc_390B8 should `bra.s loc_39096` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_6_init_loc_390ba_allocates_priority_0_with_sub_39420() {
        // ASM loc_390BA (kid.asm:41705): head 1 eye piece 6 init — $01030002,
        // priority 0 (tertiary wave), sub_39420 orbit.
        let result = boomerang_boss_head_1_eye_piece_6_init_loc_390ba(0x0120, 0x0080);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_390BA should load gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_390BA should set `priority(a3)` to 0 (tertiary wave)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_390BA should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_390BA should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_390BA should set `$13(a3)` (display enabled) via `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_390BA should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_390BA should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x0120,
            "ASM loc_390BA should copy `$44(a5)` into `x_pos(a3)`",
        );
        assert_eq!(
            result.y_pos, 0x0080,
            "ASM loc_390BA should copy `$46(a5)` into `y_pos(a3)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_390BA should clear `d0` via `sf` (initial frame byte for sub_39420)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_6_tracking_loop_loc_390f6_loops_when_breakup_zero() {
        // ASM loc_390F6 (kid.asm:41720): tracking loop — sub_39420 + sub_37A22,
        // tst.w ($FFFFFB72) == 0 → loop back.
        let result = boomerang_boss_head_1_eye_piece_6_tracking_loop_loc_390f6(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_39420,
            "ASM loc_390F6 should call `sub_39420` for orbit placement (head 1's piece 6)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_390F6 should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_390F6 should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece6TrackingNextLoc390F6::TrackingLoopLoc390F6,
            "ASM loc_390F6 should loop back when `$FFFFFB72` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_6_tracking_loop_loc_390f6_launches_when_breakup_nonzero() {
        // ASM loc_390F6: tst.w ($FFFFFB72) != 0 → bne.s loc_3911A (launch).
        let result = boomerang_boss_head_1_eye_piece_6_tracking_loop_loc_390f6(1);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece6TrackingNextLoc390F6::LaunchLoc3911A,
            "ASM loc_390F6 should branch to loc_3911A (launch) when `$FFFFFB72` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_6_tracking_loop_loc_390f6_launches_for_any_nonzero() {
        // ASM loc_390F6: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_1_eye_piece_6_tracking_loop_loc_390f6(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead1EyePiece6TrackingNextLoc390F6::LaunchLoc3911A,
            "ASM loc_390F6 should launch for any nonzero `$FFFFFB72` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_1_eye_piece_6_launch_loc_3911a_sets_rightward_velocity_and_gravity() {
        // ASM loc_3911A (kid.asm:41733): sub_393B2, y_vel=$FFFD, x_vel=#$4000.
        // loc_3912E (kid.asm:41738): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_39150.
        let result = boomerang_boss_head_1_eye_piece_6_launch_loc_3911a();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_3911A should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_3911A should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_3911A should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward — mirrors piece 5's leftward at loc_39082)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_3912E should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_3912E should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_3912E should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_39150,
            "ASM loc_39150 should `bra.s loc_3912E` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_5_init_loc_39152_sets_priority_0_tertiary_wave() {
        // ASM loc_39152 (kid.asm:41753): head 2 eye piece 5 init — gfx $01030002,
        // priority 0 (tertiary wave), sub_393C8 orbit, parent $FFFFFB7E.
        let result = boomerang_boss_head_2_eye_piece_5_init_loc_39152(0x100, 0x80);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_39152 should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_39152 should set priority 0 (tertiary wave, same as piece 6 at loc_391EA)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_39152 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_39152 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_39152 should set `$13(a3)` (display enabled) via `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_39152 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_39152 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x100,
            "ASM loc_39152 should copy `$44(a5)` into `x_pos(a3)`",
        );
        assert_eq!(
            result.y_pos, 0x80,
            "ASM loc_39152 should copy `$46(a5)` into `y_pos(a3)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_39152 should clear `d0` via `sf` for sub_393C8 orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_5_tracking_loop_loc_3918e_loops_when_breakup_zero() {
        // ASM loc_3918E (kid.asm:41768): tracking loop — sub_393C8 + sub_37A22,
        // tst.w ($FFFFFB74) == 0 → loop back.
        let result = boomerang_boss_head_2_eye_piece_5_tracking_loop_loc_3918E(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_393c8,
            "ASM loc_3918E should call `sub_393C8` for orbit placement (head 2's piece 5)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_3918E should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_3918E should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece5TrackingNextLoc3918E::TrackingLoopLoc3918E,
            "ASM loc_3918E should loop back when `$FFFFFB74` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_5_tracking_loop_loc_3918e_launches_when_breakup_nonzero() {
        // ASM loc_3918E: tst.w ($FFFFFB74) != 0 → bne.s loc_391B2 (launch).
        let result = boomerang_boss_head_2_eye_piece_5_tracking_loop_loc_3918E(1);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece5TrackingNextLoc3918E::LaunchLoc391B2,
            "ASM loc_3918E should branch to loc_391B2 (launch) when `$FFFFFB74` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_5_tracking_loop_loc_3918e_launches_for_any_nonzero() {
        // ASM loc_3918E: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_2_eye_piece_5_tracking_loop_loc_3918E(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece5TrackingNextLoc3918E::LaunchLoc391B2,
            "ASM loc_3918E should launch for any nonzero `$FFFFFB74` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_5_launch_loc_391b2_sets_leftward_velocity_and_gravity() {
        // ASM loc_391B2 (kid.asm:41781): sub_393B2, y_vel=$FFFD, x_vel=#$FFFFC000.
        // loc_391C6 (kid.asm:41786): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_391E8.
        let result = boomerang_boss_head_2_eye_piece_5_launch_loc_391B2();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_391B2 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_391B2 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_391B2 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward — mirrors piece 6's rightward at loc_3924A)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_391C6 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_391C6 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_391C6 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_391e8,
            "ASM loc_391E8 should `bra.s loc_391C6` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_6_init_loc_391ea_sets_priority_0_tertiary_wave() {
        // ASM loc_391EA (kid.asm:41801): head 2 eye piece 6 init — gfx $01030002,
        // priority 0 (tertiary wave), sub_39420 orbit, parent $FFFFFB7E.
        let result = boomerang_boss_head_2_eye_piece_6_init_loc_391EA(0x100, 0x80);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_391EA should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_391EA should set priority 0 (tertiary wave, same as piece 5 at loc_39152)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_391EA should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_391EA should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_391EA should set `$13(a3)` (display enabled) via `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_391EA should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_391EA should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x100,
            "ASM loc_391EA should copy `$44(a5)` into `x_pos(a3)`",
        );
        assert_eq!(
            result.y_pos, 0x80,
            "ASM loc_391EA should copy `$46(a5)` into `y_pos(a3)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_391EA should clear `d0` via `sf` for sub_39420 orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_6_tracking_loop_loc_39226_loops_when_breakup_zero() {
        // ASM loc_39226 (kid.asm:41816): tracking loop — sub_39420 + sub_37A22,
        // tst.w ($FFFFFB74) == 0 → loop back.
        let result = boomerang_boss_head_2_eye_piece_6_tracking_loop_loc_39226(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_39420,
            "ASM loc_39226 should call `sub_39420` for orbit placement (head 2's piece 6)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_39226 should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_39226 should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece6TrackingNextLoc39226::TrackingLoopLoc39226,
            "ASM loc_39226 should loop back when `$FFFFFB74` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_6_tracking_loop_loc_39226_launches_when_breakup_nonzero() {
        // ASM loc_39226: tst.w ($FFFFFB74) != 0 → bne.s loc_3924A (launch).
        let result = boomerang_boss_head_2_eye_piece_6_tracking_loop_loc_39226(1);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece6TrackingNextLoc39226::LaunchLoc3924A,
            "ASM loc_39226 should branch to loc_3924A (launch) when `$FFFFFB74` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_6_tracking_loop_loc_39226_launches_for_any_nonzero() {
        // ASM loc_39226: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_2_eye_piece_6_tracking_loop_loc_39226(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead2EyePiece6TrackingNextLoc39226::LaunchLoc3924A,
            "ASM loc_39226 should launch for any nonzero `$FFFFFB74` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_eye_piece_6_launch_loc_3924a_sets_rightward_velocity_and_gravity() {
        // ASM loc_3924A (kid.asm:41829): sub_393B2, y_vel=$FFFD, x_vel=#$4000.
        // loc_3925E (kid.asm:41834): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_39280.
        let result = boomerang_boss_head_2_eye_piece_6_launch_loc_3924A();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_3924A should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_3924A should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_3924A should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward — mirrors piece 5's leftward at loc_391B2)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_3925E should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_3925E should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_3925E should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_39280,
            "ASM loc_39280 should `bra.s loc_3925E` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_5_init_loc_39282_sets_priority_0_tertiary_wave() {
        // ASM loc_39282 (kid.asm:41849): head 3 eye piece 5 init — gfx $01030002,
        // priority 0 (tertiary wave), sub_393C8 orbit, parent $FFFFFB82.
        let result = boomerang_boss_head_3_eye_piece_5_init_loc_39282(0x100, 0x80);

        assert_eq!(
            result.gfx_slot, 0x0103_0002,
            "ASM loc_39282 should allocate gfx slot `$01030002`",
        );
        assert_eq!(
            result.priority, 0,
            "ASM loc_39282 should set priority 0 (tertiary wave, same as piece 6 at loc_3931A)",
        );
        assert_eq!(
            result.object_meta, 0x21,
            "ASM loc_39282 should set `object_meta(a3)` to `$21`",
        );
        assert!(
            !result.has_kid_collision,
            "ASM loc_39282 should clear `has_kid_collision(a3)` via `sf`",
        );
        assert!(
            result.display_enabled,
            "ASM loc_39282 should set `$13(a3)` (display enabled) via `st`",
        );
        assert!(
            result.is_moved,
            "ASM loc_39282 should set `is_moved(a3)` via `st`",
        );
        assert_eq!(
            result.sprite_label, "LnkTo_unk_C8488-Data_Index",
            "ASM loc_39282 should assign sprite `LnkTo_unk_C8488-Data_Index`",
        );
        assert_eq!(
            result.x_pos, 0x100,
            "ASM loc_39282 should copy `$44(a5)` into `x_pos(a3)`",
        );
        assert_eq!(
            result.y_pos, 0x80,
            "ASM loc_39282 should copy `$46(a5)` into `y_pos(a3)`",
        );
        assert_eq!(
            result.initial_frame_byte, 0,
            "ASM loc_39282 should clear `d0` via `sf` for sub_393C8 orbit placement",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_5_tracking_loop_loc_392be_loops_when_breakup_zero() {
        // ASM loc_392BE (kid.asm:41864): tracking loop — sub_393C8 + sub_37A22,
        // tst.w ($FFFFFB76) == 0 → loop back.
        let result = boomerang_boss_head_3_eye_piece_5_tracking_loop_loc_392BE(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_393c8,
            "ASM loc_392BE should call `sub_393C8` for orbit placement (head 3's piece 5)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_392BE should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_392BE should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece5TrackingNextLoc392BE::TrackingLoopLoc392BE,
            "ASM loc_392BE should loop back when `$FFFFFB76` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_5_tracking_loop_loc_392be_launches_when_breakup_nonzero() {
        // ASM loc_392BE: tst.w ($FFFFFB76) != 0 → bne.s loc_392E2 (launch).
        let result = boomerang_boss_head_3_eye_piece_5_tracking_loop_loc_392BE(1);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece5TrackingNextLoc392BE::LaunchLoc392E2,
            "ASM loc_392BE should branch to loc_392E2 (launch) when `$FFFFFB76` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_5_tracking_loop_loc_392be_launches_for_any_nonzero() {
        // ASM loc_392BE: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_3_eye_piece_5_tracking_loop_loc_392BE(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece5TrackingNextLoc392BE::LaunchLoc392E2,
            "ASM loc_392BE should launch for any nonzero `$FFFFFB76` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_5_launch_loc_392e2_sets_leftward_velocity_and_gravity() {
        // ASM loc_392E2 (kid.asm:41877): sub_393B2, y_vel=$FFFD, x_vel=#$FFFFC000.
        // loc_392F6 (kid.asm:41882): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_39318.
        let result = boomerang_boss_head_3_eye_piece_5_launch_loc_392E2();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_392E2 should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_392E2 should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, -0x4000_i32,
            "ASM loc_392E2 should set `x_vel(a3)` to `$FFFFC000` (signed longword = -0x4000, leftward — mirrors piece 6's rightward at loc_3937A)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_392F6 should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_392F6 should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_392F6 should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_39318,
            "ASM loc_39318 should `bra.s loc_392F6` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_6_tracking_loop_loc_39356_loops_when_breakup_zero() {
        // ASM loc_39356 (kid.asm:41912): tracking loop — sub_39420 + sub_37A22,
        // tst.w ($FFFFFB76) == 0 → loop back.
        let result = boomerang_boss_head_3_eye_piece_6_tracking_loop_loc_39356(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_orbit_placement_sub_39420,
            "ASM loc_39356 should call `sub_39420` for orbit placement (head 3's piece 6, mirrors piece 5's sub_393C8)",
        );
        assert!(
            result.runs_directional_sprite_sub_37a22,
            "ASM loc_39356 should call `sub_37A22` for directional sprite selection",
        );
        assert!(
            result.updates_sprite_from_d6,
            "ASM loc_39356 should store `d6` into `addroffset_sprite(a3)`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece6TrackingNextLoc39356::TrackingLoopLoc39356,
            "ASM loc_39356 should loop back when `$FFFFFB76` is zero (no breakup)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_6_tracking_loop_loc_39356_launches_when_breakup_nonzero() {
        // ASM loc_39356: tst.w ($FFFFFB76) != 0 → bne.s loc_3937A (launch).
        let result = boomerang_boss_head_3_eye_piece_6_tracking_loop_loc_39356(1);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece6TrackingNextLoc39356::LaunchLoc3937A,
            "ASM loc_39356 should branch to loc_3937A (launch) when `$FFFFFB76` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_6_tracking_loop_loc_39356_launches_for_any_nonzero() {
        // ASM loc_39356: `tst.w` tests the full word — any nonzero value triggers launch.
        let result = boomerang_boss_head_3_eye_piece_6_tracking_loop_loc_39356(0x8000);

        assert_eq!(
            result.next,
            BoomerangBossHead3EyePiece6TrackingNextLoc39356::LaunchLoc3937A,
            "ASM loc_39356 should launch for any nonzero `$FFFFFB76` value (word test)",
        );
    }

    #[test]
    fn boomerang_boss_head_3_eye_piece_6_launch_loc_3937a_sets_rightward_velocity_and_gravity() {
        // ASM loc_3937A (kid.asm:41925): sub_393B2, y_vel=$FFFD, x_vel=#$4000.
        // loc_3938E (kid.asm:41930): gravity loop — sub_37A22, addi.l #$7D0,y_vel;
        // check y_pos vs Level_height_pixels; delete or loop back via loc_393B0.
        let result = boomerang_boss_head_3_eye_piece_6_launch_loc_3937A();

        assert!(
            result.runs_detach_step_sub_393b2,
            "ASM loc_3937A should call `sub_393B2` (detach step) before setting velocity",
        );
        assert_eq!(
            result.y_vel, -3,
            "ASM loc_3937A should set `y_vel(a3)` to `$FFFD` (signed word = -3, upward)",
        );
        assert_eq!(
            result.x_vel, 0x4000_i32,
            "ASM loc_3937A should set `x_vel(a3)` to `$4000` (signed longword = +0x4000, rightward — mirrors piece 5's leftward at loc_392E2)",
        );
        assert_eq!(
            result.gravity_per_frame, 0x7D0,
            "ASM loc_3938E should add `$7D0` to `y_vel(a3)` each frame (gravity)",
        );
        assert!(
            result.runs_directional_sprite_during_flight,
            "ASM loc_3938E should call `sub_37A22` each frame to update directional sprite",
        );
        assert!(
            result.deletes_below_level_height,
            "ASM loc_3938E should delete the object when `y_pos > Level_height_pixels`",
        );
        assert!(
            result.loops_back_loc_393b0,
            "ASM loc_393B0 should `bra.s loc_3938E` when `y_pos <= Level_height_pixels` (gravity loop-back)",
        );
    }

    #[test]
    fn boomerang_boss_head_2_attack_wait_loop_loc_380b8_checks_animation_when_status_is_zero() {
        let result = boomerang_boss_head_2_attack_wait_loop_loc_380B8(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_399e4,
            "ASM loc_380B8 should run `sub_399E4` after hibernating the attack animation frame",
        );
        assert_eq!(
            result.cached_collision_word_fffffb74, 0,
            "ASM loc_380B8 should copy zero `d7` into `$FFFFFB74` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2AttackWaitNextLoc380B8::CheckAttackAnimationDone,
            "ASM loc_380B8 should continue to the attack animation gate when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_attack_wait_loop_loc_380b8_branches_to_shared_exit_on_status() {
        let result = boomerang_boss_head_2_attack_wait_loop_loc_380B8(0x8000);

        assert_eq!(
            result.cached_collision_word_fffffb74, 0x8000,
            "ASM loc_380B8 should preserve nonzero `d7` in `$FFFFFB74`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2AttackWaitNextLoc380B8::SharedCollisionExitLoc39EBE,
            "ASM loc_380B8 should branch to loc_39EBE instead of testing the animation byte when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_attack_animation_gate_loc_380cc_loops_until_animation_finishes() {
        let result = boomerang_boss_head_2_attack_animation_gate_loc_380CC(0);

        assert!(!result.attack_animation_done);
        assert!(!result.runs_projectile_spawn_sub_397e4);
        assert_eq!(
            result.next,
            BoomerangBossHead2AttackAnimationNextLoc380CC::AwaitAttackWaitLoopLoc380B8,
            "ASM loc_380CC should branch back to loc_380B8 while `$18(a3)` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_attack_animation_gate_loc_380cc_spawns_projectile_when_done() {
        let result = boomerang_boss_head_2_attack_animation_gate_loc_380CC(0x80);

        assert!(result.attack_animation_done);
        assert!(
            result.runs_projectile_spawn_sub_397e4,
            "ASM loc_380CC should call `sub_397E4` as soon as `$18(a3)` is nonzero",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2AttackAnimationNextLoc380CC::SpawnProjectileSub397E4,
        );
    }

    #[test]
    fn boomerang_boss_head_2_post_projectile_wait_loop_loc_380d6_checks_animation_when_status_is_zero()
     {
        let result = boomerang_boss_head_2_post_projectile_wait_loop_loc_380D6(0);

        assert!(result.hibernates_one_frame);
        assert!(
            result.runs_movement_wrapper_sub_399e4,
            "ASM loc_380D6 should run `sub_399E4` after hibernating the post-projectile animation frame",
        );
        assert_eq!(
            result.cached_collision_word_fffffb74, 0,
            "ASM loc_380D6 should copy zero `d7` into `$FFFFFB74` before falling through",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2PostProjectileWaitNextLoc380D6::CheckAnimationDone,
            "ASM loc_380D6 should continue to the post-projectile animation gate when `tst.w d7` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_post_projectile_wait_loop_loc_380d6_branches_to_shared_exit_on_status()
    {
        let result = boomerang_boss_head_2_post_projectile_wait_loop_loc_380D6(0x0002);

        assert_eq!(
            result.cached_collision_word_fffffb74, 0x0002,
            "ASM loc_380D6 should preserve nonzero `d7` in `$FFFFFB74`",
        );
        assert_eq!(
            result.next,
            BoomerangBossHead2PostProjectileWaitNextLoc380D6::SharedCollisionExitLoc39EBE,
            "ASM loc_380D6 should branch to loc_39EBE instead of testing `$18(a3)` when `d7` is nonzero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_post_projectile_animation_gate_loc_380ea_loops_until_animation_finishes()
     {
        let result = boomerang_boss_head_2_post_projectile_animation_gate_loc_380EA(0);

        assert!(!result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            BoomerangBossHead2PostProjectileAnimationNextLoc380EA::AwaitPostProjectileWaitLoc380D6,
            "ASM loc_380EA should branch back to loc_380D6 while `$18(a3)` is zero",
        );
    }

    #[test]
    fn boomerang_boss_head_2_post_projectile_animation_gate_loc_380ea_returns_to_activation_when_done()
     {
        let result = boomerang_boss_head_2_post_projectile_animation_gate_loc_380EA(1);

        assert!(result.post_projectile_animation_done);
        assert_eq!(
            result.next,
            BoomerangBossHead2PostProjectileAnimationNextLoc380EA::ReturnToActivationWaitLoc38062,
            "ASM loc_380EA should branch to loc_38062 when the post-projectile animation byte is nonzero",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_39846_only_counts_down_when_delay_has_not_elapsed() {
        let script = [(0x0168, 0, -1), (-1, 0, 0)];

        let result = boss_head_movement_step_sub_39846(2, 0, &script, &script);

        assert_eq!(
            result.next_delay_counter, 1,
            "ASM sub_39846 should return immediately after `subq.w #1,d0` when the decremented delay is still nonzero",
        );
        assert_eq!(
            result.next_script_index, 0,
            "ASM sub_39846 should not advance `(a2)+` until the delay reaches zero",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (None, None),
            "ASM sub_39846 should leave `x_vel/y_vel(a3)` untouched on countdown-only frames",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_39846_loads_the_next_velocity_pair_when_delay_elapses() {
        let script = [(0x0168, 0, -1), (-1, 0, 0)];

        let result = boss_head_movement_step_sub_39846(1, 0, &script, &script);

        assert_eq!(
            result.next_delay_counter, 0x0168,
            "ASM loc_3984C should load the next delay word from `(a2)+` once `d0` reaches zero",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM loc_39858 should advance `(a2)+` past the consumed `(delay, x, y)` triple",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0), Some(-0x0000_8000)),
            "ASM loc_39858 should convert the signed X/Y words into FP16.16 velocities via `swap; asr.l #1`",
        );
    }
}

mod chunk_08_inactive_slot_constructors {
    use super::*;

    #[test]
    fn falling_weather_slot_inactive_writes_minus_one_to_both_y_and_x() {
        // ASM sub_61CA loc_6226 (kid.asm:6227), loc_636A (kid.asm:6363),
        // loc_6398 (kid.asm:6384), and loc_63C4 (kid.asm:6405) all clear a
        // falling slot via `move.l #$FFFFFFFF,(a0)` — a single 32-bit write
        // that sets the y_px word AND the x_px word to $FFFF. The native
        // `inactive` constructor must mirror this longword contract: x_px
        // must equal -1 (not just any negative or positive sentinel),
        // because the off-screen scan at loc_625E reads the y_px word and
        // the spawn path at loc_62C2 then writes (y_px, x_px) as a contiguous
        // pair. A constructor that left x_px > 0 would silently break any
        // future code that scanned the x_px word for the inactive sentinel.
        let slot = FallingWeatherSlotSub61CA::inactive(6, 3);

        assert_eq!(
            slot.y_px, -1,
            "ASM loc_6226 `move.l #$FFFFFFFF,(a0)` writes $FFFF into the y_px word"
        );
        assert_eq!(
            slot.x_px, -1,
            "ASM loc_6226 `move.l #$FFFFFFFF,(a0)` writes $FFFF into the x_px word \
             (low half of the same longword)"
        );
        assert_eq!(slot.animation_cooldown, 6);
        assert_eq!(slot.animation_frame, 3);
    }

    #[test]
    fn weather_impact_slot_inactive_writes_minus_one_to_both_y_and_x() {
        // ASM sub_61CA loc_63C4 (kid.asm:6405) deactivates an impact slot via
        // `move.l #$FFFFFFFF,(a0); move.w #$100,4(a0)`. Same longword
        // contract as the falling slots: y_px AND x_px both go to $FFFF.
        let slot = WeatherImpactSlotSub61CA::inactive(1, 0);

        assert_eq!(
            slot.y_px, -1,
            "ASM loc_63C4 `move.l #$FFFFFFFF,(a0)` writes $FFFF into the y_px word"
        );
        assert_eq!(
            slot.x_px, -1,
            "ASM loc_63C4 `move.l #$FFFFFFFF,(a0)` writes $FFFF into the x_px word \
             (low half of the same longword)"
        );
        assert_eq!(slot.animation_cooldown, 1);
        assert_eq!(slot.animation_frame, 0);
    }
}
