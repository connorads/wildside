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
    fn boss_eye_pop_effect_face_kid_loc_37300_uses_signed_word_compare() {
        assert_eq!(
            boss_eye_pop_effect_face_kid_sub_3729E(0x8001, 0x7FFF),
            0xFF,
            "ASM loc_37300 uses signed `cmp.w d5,d3; bgt.s loc_37316`, so $8001 is left of $7FFF"
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
            left.initial_y_velocity, -0x0004_0000,
            "ASM sub_37332 writes `$FFFC` to the high word of `y_vel(a3)`, seeding a 16.16 -4px/frame launch",
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
    fn boss_eye_attack_init_loc_37394_uses_signed_word_compare() {
        let attack = boss_eye_attack_init_sub_37332(0x8001, 0x0046, 0x7FFF);

        assert_eq!(
            attack.initial_x_velocity, -0x0000_7FFF,
            "ASM sub_37332 uses signed `cmp.w d5,d3; bge.s loc_37394`, so $8001 launches left when Kid is at $7FFF"
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
    fn weather_spawn_loc_626c_does_not_check_next_tile_when_footprint_stays_in_column() {
        // ASM sub_61CA loc_626C..loc_62AA: `addi.w #$F,d6; lsr.w #4,d6`
        // computes the ceil column, then `cmp.w d7,d6; beq.s loc_62AA`
        // skips the second collision-word test when spawn_x..spawn_x+$F
        // remains inside the same 16px column. A left shift here would make
        // every nonzero spawn_x look like it crossed into the next column.
        let inactive = FallingWeatherSlotSub61CA::inactive(6, 3);
        let input = WeatherWorkerInputSub61CA {
            camera_x_px: -0x18,
            camera_y_px: 0,
            spawn_cooldown: 1,
            free_falling_slots: 1,
            falling_slots: [inactive; 5],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 0x100,
            kid_hitbox_bottom_px: 0x120,
            kid_hitbox_left_px: 0x100,
            kid_hitbox_right_px: 0x120,
            random_byte: 0,
            spawn_random_byte: 0,
            collision_rows: WeatherCollisionRowsSub61CA {
                rows: vec![vec![0, 0, 0x00FF]],
            },
            starting_number_sprites: 0,
        };

        let result = step_weather_worker_sub_61CA(input);

        assert_eq!(
            result.falling_slots[0].x_px, 0x0010,
            "ASM loc_62AA should spawn at x=$10 without consulting the blocked next tile because (($10+$0F)>>4) == ($10>>4)"
        );
        assert_eq!(result.falling_slots[0].y_px, 0);
        assert_eq!(result.free_falling_slots, 0);
    }

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

    #[test]
    fn weather_impact_slot_loc_63c4_deactivates_both_position_words_after_final_frame() {
        // ASM sub_61CA loc_63C4 (kid.asm:6405): once an impact slot advances
        // past frame 1, `move.l #$FFFFFFFF,(a0)` clears both the y and x
        // words before `move.w #$100,4(a0)` parks the inactive cooldown.
        let inactive_falling = FallingWeatherSlotSub61CA::inactive(6, 0);
        let inactive_impact = WeatherImpactSlotSub61CA::inactive(1, 0);
        let mut impact_slots = [inactive_impact; 10];
        impact_slots[0] = WeatherImpactSlotSub61CA {
            y_px: 0x0040,
            x_px: 0x0050,
            animation_cooldown: 1,
            animation_frame: 1,
        };
        let input = WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 0,
            free_falling_slots: 0,
            falling_slots: [inactive_falling; 5],
            impact_slots,
            kid_hitbox_top_px: 0x0100,
            kid_hitbox_bottom_px: 0x0120,
            kid_hitbox_left_px: 0x0100,
            kid_hitbox_right_px: 0x0120,
            random_byte: 0,
            spawn_random_byte: 0,
            collision_rows: WeatherCollisionRowsSub61CA { rows: vec![] },
            starting_number_sprites: 0,
        };

        let result = step_weather_worker_sub_61CA(input);

        assert_eq!(
            (result.impact_slots[0].y_px, result.impact_slots[0].x_px),
            (-1, -1),
            "ASM loc_63C4 should clear both contiguous position words with one longword write"
        );
        assert_eq!(result.impact_slots[0].animation_cooldown, 1);
        assert_eq!(result.impact_slots[0].animation_frame, 0);
        assert!(
            result.sprites.is_empty(),
            "ASM loc_63F6 branches to loc_6456 immediately after deactivation, so the expired impact slot emits no sprite"
        );
    }
}

mod chunk_09_weather_mutation_coverage {
    use crate::weather::{
        FALLING_WEATHER_SLOT_COUNT_SUB_61CA, LAVA_GEYSER_ART_SOURCE_LABEL_LOC_14C6A,
        LAVA_GEYSER_ART_VRAM_DESTINATION_LOC_14C6A, SPECIAL_EFFECT_UNKNOWN_ART_LABEL_LOC_14DE4,
        SPECIAL_EFFECT_UNKNOWN_BACKGROUND_PLANE_REGISTER_LOC_14E42,
        SPECIAL_EFFECT_UNKNOWN_FOREGROUND_PLANE_REGISTER_LOC_14E42,
        SPECIAL_EFFECT_UNKNOWN_PLANE_VDP_COMMAND_LOC_14E42, STORM_ART_HAIL_SOURCE_LABEL_LOC_14CCE,
        STORM_PALETTE_BUFFER_START_LOC_14CF2, STORM_PALETTE_HAIL_LABEL_LOC_14CEC,
        STORM_PALETTE_HAIL_PATH_LOC_14CEC, STORM_PATTERN_HAIL_TABLE_LABEL_LOC_14D1E,
        STORM_PATTERN_VDP_COMMAND_LOC_14D32, SpecialEffectUnknownRuntimeArtUploadLoc14DF6,
        StormArtAssetLoc14CCE, StormArtUploadLoc14CD8, StormPaletteAssetLoc14CEC,
        StormPaletteCopyLoc14CF2, copy_storm_palette_loc_14CF2,
        record_special_effect_unknown_art_upload_loc_14DF6, special_effect_unknown_group_loc_14E44,
        special_effect_unknown_inner_repeat_count_loc_14E44,
        special_effect_unknown_pair_write_loc_14E46, storm_pattern_emit_group_loc_14D40,
        upload_special_effect_unknown_art_loc_14DF6, upload_storm_art_loc_14CD8,
    };

    use super::*;

    fn weather_worker_input_sub_61ca() -> WeatherWorkerInputSub61CA {
        WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 3,
            free_falling_slots: 0,
            falling_slots: [FallingWeatherSlotSub61CA::inactive(6, 0); 5],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 0x0200,
            kid_hitbox_bottom_px: 0x0220,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0220,
            random_byte: 0,
            spawn_random_byte: 0,
            collision_rows: WeatherCollisionRowsSub61CA { rows: vec![] },
            starting_number_sprites: 0,
        }
    }

    fn write_be_words(path: &std::path::Path, words: &[u16]) {
        let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_be_bytes()).collect();
        std::fs::write(path, bytes).expect("write palette fixture");
    }

    #[test]
    fn storm_pending_palette_0_mask_sub_6048_preserves_negative_high_word_and_speed() {
        assert_eq!(
            crate::weather::storm_pending_palette_0_mask_sub_6048(-1, 0x03),
            0xFFFF_0003,
            "ASM loc_60C0 writes #-1 to the high word and loc_6058 leaves the palette speed visible in the low byte",
        );
        assert_eq!(
            crate::weather::storm_pending_palette_0_mask_sub_6048(0x0080, 0x05),
            0x8000_0005,
            "ASM loc_605E writes only the pending-mask top byte for non-negative masks, then ORs in the palette speed byte",
        );
        assert_eq!(
            crate::weather::storm_pending_palette_0_mask_sub_6048(0, 0x03),
            0x0000_0003,
            "ASM loc_605E treats a zero pending mask as non-negative and leaves the high word clear",
        );
    }

    #[test]
    fn copy_storm_palette_loc_14cf2_replaces_only_the_eight_storm_words() {
        let source_palette = [
            0x1001, 0x1002, 0x1003, 0x1004, 0x1005, 0x1006, 0x1007, 0x1008,
        ];
        let mut palette_buffer = [0xDEAD; 0x40];

        copy_storm_palette_loc_14CF2(&source_palette, &mut palette_buffer);

        assert_eq!(
            &palette_buffer[0x10..0x18],
            &source_palette,
            "ASM loc_14CF2 copies four longwords from the selected storm palette into Palette_Buffer+$20",
        );
        assert!(
            palette_buffer[..0x10].iter().all(|&word| word == 0xDEAD)
                && palette_buffer[0x18..].iter().all(|&word| word == 0xDEAD),
            "ASM loc_14CF2 leaves palette entries outside the eight-word `$20..$2E` destination span untouched",
        );
    }

    #[test]
    fn storm_pattern_emit_group_loc_14d40_adds_the_tile_offset_to_each_longword() {
        let source_group = [0x1000_0000, 0x1000_0001, 0x1000_0002, 0x1000_0003];

        assert_eq!(
            storm_pattern_emit_group_loc_14D40(&source_group, 0x0380_0380),
            [0x1380_0380, 0x1380_0381, 0x1380_0382, 0x1380_0383],
            "ASM loc_14D40 loads `(a0)`, `4(a0)`, `8(a0)`, and `$C(a0)`, adds `$0380_0380`, and emits the adjusted longwords in order",
        );
    }

    #[test]
    fn upload_special_effect_unknown_art_loc_14df6_streams_inclusive_eight_longword_bursts() {
        let source_longwords: Vec<u32> = (0..24).map(|index| 0x9000_0000u32 + index).collect();

        assert_eq!(
            upload_special_effect_unknown_art_loc_14DF6(&source_longwords, 0).written_longwords,
            source_longwords[..8],
            "ASM loc_14DF6 streams one eight-longword burst when `d0` starts at zero because the stores run before `dbf` reaches -1",
        );
        assert_eq!(
            upload_special_effect_unknown_art_loc_14DF6(&source_longwords, 2).written_longwords,
            source_longwords[..24],
            "ASM loc_14DF6 streams `(d0+1)*8` longwords because each inclusive DBF iteration copies eight source longwords",
        );
    }

    #[test]
    fn special_effect_unknown_pair_write_loc_14e46_writes_d6_then_d7() {
        assert_eq!(
            special_effect_unknown_pair_write_loc_14E46(),
            [0x8380_8381, 0x8382_8383],
            "ASM loc_14E46 writes the current `d6` longword before the current `d7` longword",
        );
    }

    #[test]
    fn special_effect_unknown_group_loc_14e44_repeats_the_seed_pair_sixteen_times() {
        assert_eq!(
            special_effect_unknown_inner_repeat_count_loc_14E44(),
            16,
            "ASM loc_14E44 loads `d0=#$F`, so `dbf d0,loc_14E46` runs sixteen pair writes",
        );

        let group = special_effect_unknown_group_loc_14E44();

        assert_eq!(
            group.len(),
            16 * 2,
            "ASM loc_14E44 emits sixteen `d6,d7` pairs before the later `$00040004` group advance",
        );
        assert_eq!(
            &group[..4],
            &[0x8380_8381, 0x8382_8383, 0x8380_8381, 0x8382_8383],
            "ASM loc_14E44 starts with the seed pair derived between loc_14DF6 and loc_14E42",
        );
        assert_eq!(
            &group[28..32],
            &[0x8380_8381, 0x8382_8383, 0x8380_8381, 0x8382_8383],
            "ASM loc_14E44 still writes the same pair on the final two iterations; the group bump happens after the inner loop",
        );
    }

    #[test]
    fn kid_hitbox_bounds_px_at_screen_anchors_at_foot_pixel_for_sub_14a58_14ab0() {
        assert_eq!(
            crate::weather::kid_hitbox_bounds_px_at_screen(100, 200),
            (95, 105, 169, 200),
            "ASM sub_14A58/sub_14AB0 read Kid_hitbox_left/right/top/bottom; native rebuilds them as x +/- 5 and foot_y - 31/foot_y",
        );
    }

    #[test]
    fn lava_geyser_hitbox_window_loc_14aaa_matches_sub_14a58_edges() {
        assert_eq!(
            crate::weather::lava_geyser_hitbox_window_loc_14AAA(0x0100, 0x0120, 0x0040),
            Some(crate::weather::LavaGeyserHitboxWindowLoc14AAA {
                left_px: 0x0103,
                right_px: 0x011D,
                top_px: 0x00F0,
                bottom_px: 0x0120,
            }),
            "ASM sub_14A58 compares Kid against X [$44-$80+3, +$1A] and Y [top sprite Y+$10, geyser base]",
        );

        assert_eq!(
            crate::weather::lava_geyser_hitbox_window_loc_14AAA(0x0100, 0x0120, 0),
            None,
            "ASM loc_1484E starts at zero visible height; native exposes no sub_14A58 window until a geyser row is active",
        );
    }

    #[test]
    fn hail_projectiles_from_falling_slots_sub_61ca_exports_active_slot_fields() {
        let projectiles = crate::weather::hail_projectiles_from_falling_slots_sub_61ca(&[
            FallingWeatherSlotSub61CA {
                y_px: 0x0040,
                x_px: 0x0120,
                animation_cooldown: 6,
                animation_frame: 0,
            },
            FallingWeatherSlotSub61CA::inactive(6, 0),
            FallingWeatherSlotSub61CA::inactive(6, 0),
            FallingWeatherSlotSub61CA::inactive(6, 0),
            FallingWeatherSlotSub61CA::inactive(6, 0),
        ]);

        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0].kind,
            EnemyProjectileKind::Hail,
            "ASM sub_61CA loc_61E2/loc_6462 should export active falling-weather slots as hail particles"
        );
        assert_eq!(
            (projectiles[0].x_pos, projectiles[0].y_pos),
            (0x0120_0000, 0x0040_0000),
            "ASM loc_6462 should project the live falling-weather X/Y words into fixed-point projectile positions"
        );
        assert_eq!(
            (projectiles[0].x_velocity, projectiles[0].y_velocity),
            (-0x0001_0000, 0x0004_0000),
            "ASM loc_61E2 should keep falling weather drifting left 1px/frame and down 4px/frame in the legacy projectile bridge"
        );
    }

    #[test]
    fn step_weather_worker_sub_61ca_advances_falling_slots_and_deletes_only_past_edges() {
        let mut input = weather_worker_input_sub_61ca();
        input.falling_slots[0] = FallingWeatherSlotSub61CA {
            y_px: 0x00DC,
            x_px: 0x0040,
            animation_cooldown: 1,
            animation_frame: 5,
        };
        input.falling_slots[1] = FallingWeatherSlotSub61CA {
            y_px: 0x0020,
            x_px: -0x000F,
            animation_cooldown: 2,
            animation_frame: 2,
        };
        input.falling_slots[2] = FallingWeatherSlotSub61CA {
            y_px: 0x00DD,
            x_px: 0x0050,
            animation_cooldown: 2,
            animation_frame: 0,
        };
        input.falling_slots[3] = FallingWeatherSlotSub61CA {
            y_px: 0x0020,
            x_px: -0x0010,
            animation_cooldown: 2,
            animation_frame: 0,
        };

        let result = step_weather_worker_sub_61CA(input);

        assert_eq!(
            (
                result.falling_slots[0].y_px,
                result.falling_slots[0].x_px,
                result.falling_slots[0].animation_cooldown,
                result.falling_slots[0].animation_frame,
            ),
            (0x00E0, 0x003F, 6, 0),
            "ASM loc_61E2/loc_620C should move an active slot, reload cooldown 6, and wrap frame 5 back to 0",
        );
        assert_eq!(
            (result.falling_slots[1].y_px, result.falling_slots[1].x_px),
            (0x0024, -0x0010),
            "ASM loc_6212 should keep slots exactly on the left delete boundary active",
        );
        assert_eq!(
            (result.falling_slots[2].y_px, result.falling_slots[2].x_px),
            (-1, -1),
            "ASM loc_6212/loc_6226 should clear slots below Camera_Y_pos+$E0",
        );
        assert_eq!(
            (result.falling_slots[3].y_px, result.falling_slots[3].x_px),
            (-1, -1),
            "ASM loc_6212/loc_6226 should clear slots left of Camera_X_pos-$10",
        );
        assert_eq!(
            result.free_falling_slots, 2,
            "ASM loc_6226 should increment `$FFFFFB3D` once per cleared falling slot",
        );
        assert_eq!(
            result.sprites.len(),
            2,
            "ASM loc_6462 should render the two falling slots that remain active after the off-screen pass",
        );
        assert_eq!(
            result.sprites[0],
            WeatherSpriteSub61CA {
                y: 0x0160,
                link: 0x0501,
                tile_word: 0xA3B5,
                x: 0x00BF,
            },
            "ASM loc_6462 should render frame 0 with screen-space `$80` bias after the movement step",
        );
    }

    #[test]
    fn step_weather_worker_sub_61ca_spawn_uses_low_nibble_and_second_tile_only_when_crossing() {
        let mut input = weather_worker_input_sub_61ca();
        input.camera_x_px = -0x0018;
        input.spawn_cooldown = 1;
        input.free_falling_slots = 1;
        input.collision_rows = WeatherCollisionRowsSub61CA {
            rows: vec![vec![0, 0, 0x00FF]],
        };

        let same_column = step_weather_worker_sub_61CA(input.clone());

        assert_eq!(
            (
                same_column.falling_slots[0].y_px,
                same_column.falling_slots[0].x_px
            ),
            (0, 0x0010),
            "ASM loc_62AA should not read the second tile when `(spawn_x+$F)>>4 == spawn_x>>4`",
        );
        assert_eq!(same_column.free_falling_slots, 0);

        input.camera_x_px = -0x0017;
        let crossing_blocked = step_weather_worker_sub_61CA(input.clone());

        assert_eq!(
            crossing_blocked.free_falling_slots, 1,
            "ASM loc_62AA should block spawn footprints that cross into a second tile whose low byte is nonzero",
        );
        assert!(
            crossing_blocked.falling_slots[0].y_px < 0,
            "ASM loc_62CE should leave the inactive slot untouched when the second tile blocks the spawn",
        );

        input.camera_x_px = -0x0018;
        input.collision_rows.rows[0][1] = 0x000F;
        let first_tile_blocked = step_weather_worker_sub_61CA(input);

        assert_eq!(
            first_tile_blocked.free_falling_slots, 1,
            "ASM loc_62AA should block spawn footprints when the first tile's low nibble is nonzero",
        );
        assert!(
            first_tile_blocked.falling_slots[0].y_px < 0,
            "ASM loc_62CE should skip loc_62C2 storage when the first tile blocks the spawn",
        );
    }

    #[test]
    fn step_weather_worker_sub_61ca_kid_hitbox_edges_are_inclusive() {
        let mut input = weather_worker_input_sub_61ca();
        input.camera_y_px = 0x0040;
        input.falling_slots[0] = FallingWeatherSlotSub61CA {
            y_px: 0x00FA,
            x_px: 0x011F,
            animation_cooldown: 2,
            animation_frame: 0,
        };
        input.kid_hitbox_top_px = 0x0100;
        input.kid_hitbox_bottom_px = 0x010C;
        input.kid_hitbox_right_px = 0x0120;
        input.kid_hitbox_left_px = 0x012C;

        let result = step_weather_worker_sub_61CA(input);

        assert_eq!(
            result.kid_collision_timer,
            Some(0x0028),
            "ASM loc_62D6 should count top/bottom/right/left equality as hail overlap before writing Kid collision `$28`",
        );
    }

    #[test]
    fn step_weather_worker_sub_61ca_kid_hitbox_rejects_each_non_overlap_side() {
        let mut input = weather_worker_input_sub_61ca();
        input.camera_y_px = 0x0040;
        input.kid_hitbox_top_px = 0x0100;
        input.kid_hitbox_bottom_px = 0x0110;
        input.kid_hitbox_left_px = 0x0100;
        input.kid_hitbox_right_px = 0x0110;
        input.falling_slots[0] = FallingWeatherSlotSub61CA {
            // After loc_61E2 movement: y=$00FD, x=$0100. Top edge y+2 is just
            // above Kid_hitbox_top.
            y_px: 0x00F9,
            x_px: 0x0101,
            animation_cooldown: 2,
            animation_frame: 0,
        };
        input.falling_slots[1] = FallingWeatherSlotSub61CA {
            // After movement: y=$0105. Bottom edge y+$E is below Kid_hitbox_bottom.
            y_px: 0x0101,
            x_px: 0x0101,
            animation_cooldown: 2,
            animation_frame: 0,
        };
        input.falling_slots[2] = FallingWeatherSlotSub61CA {
            // After movement: x=$010F. Left edge x+2 is right of Kid_hitbox_right.
            y_px: 0x00FE,
            x_px: 0x0110,
            animation_cooldown: 2,
            animation_frame: 0,
        };
        input.falling_slots[3] = FallingWeatherSlotSub61CA {
            // After movement: x=$00F1. Right edge x+$E is left of Kid_hitbox_left.
            y_px: 0x00FE,
            x_px: 0x00F2,
            animation_cooldown: 2,
            animation_frame: 0,
        };

        let result = step_weather_worker_sub_61CA(input);

        assert_eq!(
            result.kid_collision_timer, None,
            "ASM loc_62D6 should reject hail particles above, below, right, or left of Kid before writing collision `$28`",
        );
    }

    fn step_weather_worker_terrain_hit_sub_61ca(
        terrain_word: u16,
    ) -> crate::weather::WeatherWorkerOutputSub61CA {
        let mut input = weather_worker_input_sub_61ca();
        input.falling_slots[0] = FallingWeatherSlotSub61CA {
            y_px: 0x001C,
            x_px: 0x0031,
            animation_cooldown: 2,
            animation_frame: 0,
        };
        input.collision_rows = WeatherCollisionRowsSub61CA {
            rows: vec![vec![0; 4], vec![0; 4], vec![0, 0, 0, terrain_word]],
        };

        step_weather_worker_sub_61CA(input)
    }

    #[test]
    fn step_weather_worker_sub_61ca_terrain_splash_offsets_only_slope_kinds() {
        let normal = step_weather_worker_terrain_hit_sub_61ca(0x7000);
        let slope = step_weather_worker_terrain_hit_sub_61ca(0x4000);

        assert_eq!(
            (normal.falling_slots[0].y_px, normal.falling_slots[0].x_px),
            (-1, -1),
            "ASM loc_636A should clear the falling slot after a nonzero terrain collision",
        );
        assert_eq!(
            (
                normal.impact_slots[0].y_px,
                normal.impact_slots[0].x_px,
                normal.impact_slots[0].animation_cooldown,
                normal.impact_slots[0].animation_frame,
            ),
            (0x0020, 0x0030, 5, 0),
            "ASM loc_636A should copy normal terrain splash Y/X directly before the same-frame loc_63C4 cooldown tick",
        );
        assert_eq!(
            (
                slope.impact_slots[0].y_px,
                slope.impact_slots[0].x_px,
                slope.impact_slots[0].animation_cooldown,
                slope.impact_slots[0].animation_frame,
            ),
            (0x002C, 0x0030, 5, 0),
            "ASM loc_638E/loc_6398 should add `$C` to splash Y only for `$4000/$5000` slope terrain",
        );
        assert_eq!(
            normal.free_falling_slots, 1,
            "ASM loc_636A should increment `$FFFFFB3D` after moving a falling slot into an impact slot",
        );
        assert_eq!(
            normal.sprites[0],
            WeatherSpriteSub61CA {
                y: 0x00A0,
                link: 0x0501,
                tile_word: 0xA3CD,
                x: 0x00B0,
            },
            "ASM loc_6402 should render the newly created frame-0 impact splash with `$500+Number_Sprites` link",
        );
    }

    #[test]
    fn step_weather_worker_sub_61ca_impact_slots_move_then_deactivate_after_frame_one() {
        let mut input = weather_worker_input_sub_61ca();
        input.starting_number_sprites = 2;
        input.impact_slots[0] = WeatherImpactSlotSub61CA {
            y_px: 0x0040,
            x_px: 0x0050,
            animation_cooldown: 1,
            animation_frame: 0,
        };
        input.impact_slots[1] = WeatherImpactSlotSub61CA {
            y_px: 0x0060,
            x_px: 0x0070,
            animation_cooldown: 1,
            animation_frame: 1,
        };

        let result = step_weather_worker_sub_61CA(input);

        assert_eq!(
            (
                result.impact_slots[0].y_px,
                result.impact_slots[0].x_px,
                result.impact_slots[0].animation_cooldown,
                result.impact_slots[0].animation_frame,
            ),
            (0x003C, 0x004C, 6, 1),
            "ASM loc_63C4 should move an impact splash up-left by four pixels and advance frame 0 to frame 1",
        );
        assert_eq!(
            (
                result.impact_slots[1].y_px,
                result.impact_slots[1].x_px,
                result.impact_slots[1].animation_cooldown,
                result.impact_slots[1].animation_frame,
            ),
            (-1, -1, 1, 0),
            "ASM loc_63C4 should clear both position words and park cooldown `$100` after frame 1 expires",
        );
        assert_eq!(
            result.sprites,
            vec![WeatherSpriteSub61CA {
                y: 0x00BC,
                link: 0x0A03,
                tile_word: 0xA3D1,
                x: 0x00CC,
            }],
            "ASM loc_6432 should render only the still-live frame-1 impact splash with `$A00+Number_Sprites` link",
        );
    }

    #[test]
    fn lava_geyser_script_template_label_loc_147ba_rejects_out_of_range_timer() {
        assert_eq!(
            crate::weather::lava_geyser_script_template_label_loc_147BA(
                LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C,
                LAVA_GEYSER_REPEAT_SCRIPT_VISIBLE_STEPS_LOC_146A0 + 1,
            ),
            None,
            "ASM loc_147BA has no live template once the repeat timer is past the visible script window",
        );
    }

    #[test]
    fn lava_geyser_script_template_label_loc_147ba_maps_startup_and_repeat_cadence() {
        assert_eq!(
            crate::weather::lava_geyser_script_template_label_loc_147BA(
                LAVA_GEYSER_PHASE_STARTUP_SCRIPT_LOC_14788,
                LAVA_GEYSER_STARTUP_SCRIPT_VISIBLE_STEPS_LOC_146B6,
            ),
            Some("unk_14602"),
            "ASM loc_147BA over unk_146B6 should start with the first startup template",
        );
        assert_eq!(
            crate::weather::lava_geyser_script_template_label_loc_147BA(
                LAVA_GEYSER_PHASE_STARTUP_SCRIPT_LOC_14788,
                1,
            ),
            Some("unk_145CA"),
            "ASM loc_147BA over unk_146B6 should end on the fifth startup template before `$FFFE`",
        );
        assert_eq!(
            crate::weather::lava_geyser_script_template_label_loc_147BA(
                LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C,
                LAVA_GEYSER_REPEAT_SCRIPT_VISIBLE_STEPS_LOC_146A0,
            ),
            Some("unk_145CA"),
            "ASM loc_147BA over unk_146A0 should hold the first repeat template for the first three-frame group",
        );
        assert_eq!(
            crate::weather::lava_geyser_script_template_label_loc_147BA(
                LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C,
                LAVA_GEYSER_REPEAT_SCRIPT_VISIBLE_STEPS_LOC_146A0
                    - LAVA_GEYSER_REPEAT_SCRIPT_FRAMES_PER_TEMPLATE_LOC_146A0,
            ),
            Some("unk_145D4"),
            "ASM loc_14804 should advance to the second repeat template only after the three-frame group completes",
        );
        assert_eq!(
            crate::weather::lava_geyser_script_template_label_loc_147BA(
                LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C,
                1,
            ),
            Some("unk_14602"),
            "ASM loc_147BA over unk_146A0 should finish on the fifth repeat template before `$FFFF`",
        );
    }

    #[test]
    fn lava_geyser_full_height_render_state_loc_14940_distinguishes_hold_and_collapse_rows() {
        let peak_rows = 5;
        assert_eq!(
            crate::weather::lava_geyser_full_height_render_state_loc_14940(
                LAVA_GEYSER_PHASE_RISING_LOC_1484E,
                0,
                3 * LAVA_GEYSER_ROW_HEIGHT_PX,
                peak_rows * LAVA_GEYSER_ROW_HEIGHT_PX,
            ),
            Some(crate::weather::LavaGeyserFullHeightRenderStateLoc14940 {
                sprite_row_index: 2,
                top_sprite_frame_within_row: 0,
            }),
            "ASM loc_1484E/loc_14940 should render rising geysers from the current visible row minus one",
        );
        assert_eq!(
            crate::weather::lava_geyser_full_height_render_state_loc_14940(
                LAVA_GEYSER_PHASE_HOLDING_LOC_1496C,
                LAVA_GEYSER_HOLD_CYCLES * LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE,
                peak_rows * LAVA_GEYSER_ROW_HEIGHT_PX,
                peak_rows * LAVA_GEYSER_ROW_HEIGHT_PX,
            ),
            Some(crate::weather::LavaGeyserFullHeightRenderStateLoc14940 {
                sprite_row_index: peak_rows - 2,
                top_sprite_frame_within_row: 1,
            }),
            "ASM loc_14940 should contract the full-height top by two rows during the first `$4C(a5)=1` hold frame",
        );
        assert_eq!(
            crate::weather::lava_geyser_full_height_render_state_loc_14940(
                LAVA_GEYSER_PHASE_HOLDING_LOC_1496C,
                LAVA_GEYSER_HOLD_CYCLES * LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE
                    - LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE,
                peak_rows * LAVA_GEYSER_ROW_HEIGHT_PX,
                peak_rows * LAVA_GEYSER_ROW_HEIGHT_PX,
            ),
            Some(crate::weather::LavaGeyserFullHeightRenderStateLoc14940 {
                sprite_row_index: peak_rows,
                top_sprite_frame_within_row: 0,
            }),
            "ASM loc_14966 should expand the top row target by two when `$4C(a5)` toggles back to zero",
        );
        assert_eq!(
            crate::weather::lava_geyser_full_height_render_state_loc_14940(
                LAVA_GEYSER_PHASE_COLLAPSING_LOC_1498C,
                0,
                3 * LAVA_GEYSER_ROW_HEIGHT_PX,
                peak_rows * LAVA_GEYSER_ROW_HEIGHT_PX,
            ),
            Some(crate::weather::LavaGeyserFullHeightRenderStateLoc14940 {
                sprite_row_index: 2,
                top_sprite_frame_within_row: 0,
            }),
            "ASM loc_1498C collapse rendering still uses the current visible row minus one, not the held peak row",
        );
        assert_eq!(
            crate::weather::lava_geyser_full_height_render_state_loc_14940(
                LAVA_GEYSER_PHASE_HOLDING_LOC_1496C,
                LAVA_GEYSER_HOLD_CYCLES * LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE
                    - LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE,
                LAVA_GEYSER_ROW_HEIGHT_PX,
                LAVA_GEYSER_ROW_HEIGHT_PX,
            ),
            Some(crate::weather::LavaGeyserFullHeightRenderStateLoc14940 {
                sprite_row_index: 1,
                top_sprite_frame_within_row: 0,
            }),
            "ASM loc_14966 must not expand a one-row peak by two rows on the second hold sprite frame",
        );
    }

    #[test]
    fn lava_geyser_full_height_rows_for_sprite_row_loc_14874_builds_visible_rows() {
        let rows = crate::weather::lava_geyser_full_height_rows_for_sprite_row_loc_14874(
            0x0180, 0x0150, 2, 0x0100, 0x0100, 0,
        );

        assert!(
            !rows.is_empty(),
            "ASM loc_14874 should build rows when the full-height geyser column is inside the camera window",
        );
        assert_eq!(
            rows[0],
            crate::weather::LavaGeyserFullHeightRowLoc14874 {
                row_y_px: 0x0110,
                source_label: "unk_14624",
            },
            "ASM loc_148D4 stores the top row Y for sprite_row_index=2 and selects the live top-template script",
        );
    }

    #[test]
    fn start_lava_geyser_rise_loc_1484e_seeds_zero_height_and_two_frame_timer() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 3 * LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C,
            timer: 1,
            current_height: LAVA_GEYSER_ROW_HEIGHT_PX,
        });

        runtime.step_lava_geysers(&airborne_slice());

        let geyser = &runtime.weather.lava_geysers[0];
        assert_eq!(geyser.phase, LAVA_GEYSER_PHASE_RISING_LOC_1484E);
        assert_eq!(
            geyser.timer, 2,
            "ASM loc_1484E seeds `$48(a5)=2` for the full-height rise loop",
        );
        assert_eq!(
            geyser.current_height, 0,
            "ASM loc_1484E clears the row cursor before the first shared loop iteration",
        );
    }

    #[test]
    fn step_lava_geysers_first_eruption_enters_startup_script_loc_14788() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 3 * LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: true,
            sound_suppressed: true,
            phase: LAVA_GEYSER_PHASE_DORMANT_LOC_14772,
            timer: 0,
            current_height: 0,
        });

        runtime.step_lava_geysers(&airborne_slice());

        let geyser = &runtime.weather.lava_geysers[0];
        assert_eq!(geyser.phase, LAVA_GEYSER_PHASE_STARTUP_SCRIPT_LOC_14788);
        assert_eq!(
            geyser.timer, LAVA_GEYSER_STARTUP_SCRIPT_VISIBLE_STEPS_LOC_146B6,
            "ASM loc_147BA over unk_146B6 should seed one frame for each startup template",
        );
        assert_eq!(
            geyser.current_height, LAVA_GEYSER_ROW_HEIGHT_PX,
            "ASM loc_147BA should expose one visible sprite row while the startup script walks its templates",
        );
    }

    #[test]
    fn step_murder_wall_forward_clamps_integer_word_and_preserves_fraction() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.current_level = LevelId(0);
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0x02FF_00AA,
            speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });
        let mut camera_x = 0x0200_0000;

        runtime.step_murder_wall(&mut camera_x);

        let wall = runtime.murder_wall.as_ref().unwrap();
        assert_eq!(
            wall.x_pos & 0xFFFF,
            0x00AA,
            "ASM loc_7334 only replaces the high word when clamping to Camera_max_X_pos, preserving the fractional low word",
        );
        assert!(
            wall.x_pos >> 16 <= 0x0640,
            "ASM loc_7334 must clamp the forward wall integer word to the level camera max",
        );
    }

    #[test]
    fn step_lava_geysers_palette_delay_counts_down_before_rotation_loc_14b2a() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: true,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_DORMANT_LOC_14772,
            timer: 10,
            current_height: 0,
        });
        runtime.lava_geyser_palette_delay_loc_14b2a = 2;
        runtime.palette_buffer[0x14] = 0x1111;
        runtime.palette_buffer[0x15] = 0x2222;
        runtime.palette_buffer[0x16] = 0x3333;
        runtime.palette_buffer[0x17] = 0x4444;

        runtime.step_lava_geysers(&airborne_slice());

        assert_eq!(
            runtime.lava_geyser_palette_delay_loc_14b2a, 1,
            "ASM Animation_Geyser loc_14B2A should decrement the five-frame counter until it reaches the reload frame",
        );
        assert_eq!(
            &runtime.palette_buffer[0x14..0x18],
            &[0x1111, 0x2222, 0x3333, 0x4444],
            "ASM loc_14B2A should not rotate palette words on countdown-only frames",
        );
    }

    #[test]
    fn init_lava_geysers_reads_big_endian_palette_words_loc_14c86() {
        let output_root = unique_temp_path("wildside-weather-lava-palette");
        let palette_dir = output_root.join("ingame/palette");
        std::fs::create_dir_all(&palette_dir).expect("create lava palette fixture dir");
        let palette_words = [
            0x0C01, 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B, 0x0C0D, 0x0E0F,
        ];
        let palette_bytes: Vec<u8> = palette_words
            .iter()
            .flat_map(|word: &u16| word.to_be_bytes())
            .collect();
        std::fs::write(palette_dir.join("lava.bin"), &palette_bytes)
            .expect("write lava palette fixture");

        let mut assets = manifest();
        assets.output_root = output_root;
        assets.ingame_assets = vec![wildside_data::AssetFile {
            relative_path: String::from("palette/lava.bin"),
            size_bytes: palette_bytes.len() as u64,
            sha1: String::from("fixture-weather-lava-palette"),
        }];
        let mut runtime = NativeRuntime::new(assets);

        runtime.init_lava_geysers(LevelId(0));

        assert_eq!(
            &runtime.palette_buffer[0x10..0x18],
            &palette_words,
            "ASM loc_14C86 copies eight big-endian Pal_7B8AC words from consecutive byte pairs into Palette_Buffer+$20",
        );
    }

    #[test]
    fn init_lava_geysers_ignores_short_palette_asset_loc_14c86() {
        let output_root = unique_temp_path("wildside-weather-short-lava-palette");
        let palette_dir = output_root.join("ingame/palette");
        std::fs::create_dir_all(&palette_dir).expect("create short lava palette fixture dir");
        let palette_bytes = [
            0x0C, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
        ];
        std::fs::write(palette_dir.join("lava.bin"), palette_bytes)
            .expect("write short lava palette fixture");

        let mut assets = manifest();
        assets.output_root = output_root;
        assets.ingame_assets = vec![wildside_data::AssetFile {
            relative_path: String::from("palette/lava.bin"),
            size_bytes: palette_bytes.len() as u64,
            sha1: String::from("fixture-weather-short-lava-palette"),
        }];
        let mut runtime = NativeRuntime::new(assets);
        runtime.palette_buffer[0x10..0x18].fill(0x0BAD);

        runtime.init_lava_geysers(LevelId(0));

        assert_eq!(
            &runtime.palette_buffer[0x10..0x18],
            &[0x0BAD; 8],
            "ASM loc_14C86 requires all eight Pal_7B8AC words; native must not copy a truncated decoded asset",
        );
    }

    #[test]
    fn step_lava_geysers_rise_adds_one_row_then_enters_hold_at_peak() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 2 * LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_RISING_LOC_1484E,
            timer: 1,
            current_height: LAVA_GEYSER_ROW_HEIGHT_PX,
        });

        runtime.step_lava_geysers(&airborne_slice());

        let geyser = &runtime.weather.lava_geysers[0];
        assert_eq!(geyser.phase, LAVA_GEYSER_PHASE_HOLDING_LOC_1496C);
        assert_eq!(
            geyser.current_height,
            2 * LAVA_GEYSER_ROW_HEIGHT_PX,
            "ASM loc_1484E should add exactly one 16px row before clamping at `$4A(a5)`",
        );
        assert_eq!(
            geyser.timer,
            LAVA_GEYSER_HOLD_CYCLES * LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE,
            "ASM loc_1496C should seed the full peak hold countdown in 6-frame sprite cycles",
        );
    }

    #[test]
    fn step_lava_geysers_startup_and_disabled_scripts_count_down_by_one() {
        let mut startup = NativeRuntime::new(manifest());
        startup.weather.weather_type = 1;
        startup.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_STARTUP_SCRIPT_LOC_14788,
            timer: 3,
            current_height: 0,
        });

        startup.step_lava_geysers(&airborne_slice());
        assert_eq!(
            startup.weather.lava_geysers[0].phase,
            LAVA_GEYSER_PHASE_STARTUP_SCRIPT_LOC_14788
        );
        assert_eq!(
            startup.weather.lava_geysers[0].timer, 2,
            "ASM loc_147BA startup script hibernates one frame per visible row entry",
        );
        assert_eq!(
            startup.weather.lava_geysers[0].current_height,
            LAVA_GEYSER_ROW_HEIGHT_PX
        );

        let mut disabled = NativeRuntime::new(manifest());
        disabled.weather.weather_type = 1;
        disabled.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_DISABLED_LOC_1483A,
            timer: 3,
            current_height: LAVA_GEYSER_ROW_HEIGHT_PX,
        });

        disabled.step_lava_geysers(&airborne_slice());
        assert_eq!(
            disabled.weather.lava_geysers[0].phase,
            LAVA_GEYSER_PHASE_DISABLED_LOC_1483A
        );
        assert_eq!(
            disabled.weather.lava_geysers[0].timer, 2,
            "ASM loc_1483A should subtract one hibernation frame while the disabled timer remains positive",
        );
        assert_eq!(disabled.weather.lava_geysers[0].current_height, 0);
    }

    #[test]
    fn step_lava_geysers_repeat_script_counts_down_before_rise() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C,
            timer: 3,
            current_height: 0,
        });

        runtime.step_lava_geysers(&airborne_slice());

        assert_eq!(
            runtime.weather.lava_geysers[0].phase,
            LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C
        );
        assert_eq!(
            runtime.weather.lava_geysers[0].timer, 2,
            "ASM loc_147BA repeat script should decrement by one and stay in-script until the `$FFFF` sentinel",
        );
        assert_eq!(
            runtime.weather.lava_geysers[0].current_height,
            LAVA_GEYSER_ROW_HEIGHT_PX
        );
    }

    #[test]
    fn transfer_scroll_data_storm_loc_30110_uses_integer_camera_words() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 2;
        runtime.weather.storm_weather_mode_byte = 1;
        runtime.weather.storm_scroll_offset_loc_30110 = 0x0100;
        let mut slice = airborne_slice();
        slice.camera = CameraState {
            x_pos: 0x0123_0000,
            y_pos: 0x0045_0000,
        };
        runtime.opening_slice = Some(slice);

        runtime.transfer_scroll_data_storm_loc_30110();

        assert_eq!(
            runtime.weather.storm_scroll_transfer_loc_30110,
            Some(crate::level::TransferScrollDataStormLoc30110 {
                plane_address_word: 0x8403,
                h_scroll_word_pairs: 0x00E0,
                first_h_scroll_word: -0x0123,
                second_h_scroll_word: 0x0100,
                first_vsram_word: 0x0045,
                second_vsram_word: 0x0100,
                weather_mode_byte: 4,
                next_storm_scroll_offset: 0x00F8,
            }),
            "ASM loc_30110 consumes Camera_X_pos/Camera_Y_pos integer words before building the storm HScroll/VSRAM transfer",
        );
    }

    #[test]
    fn step_lava_geysers_dormant_repeat_script_seeds_all_template_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: LAVA_GEYSER_ROW_HEIGHT_PX,
            first_eruption_pending: false,
            sound_suppressed: false,
            phase: LAVA_GEYSER_PHASE_DORMANT_LOC_14772,
            timer: 0,
            current_height: 0,
        });

        runtime.step_lava_geysers(&airborne_slice());

        let geyser = &runtime.weather.lava_geysers[0];
        assert_eq!(geyser.phase, LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C);
        assert_eq!(
            geyser.timer, 15,
            "ASM loc_147BA repeat path walks five `d2=2` template entries, so loc_1479C seeds 15 visible script frames",
        );
        assert_eq!(geyser.current_height, LAVA_GEYSER_ROW_HEIGHT_PX);
    }

    #[test]
    fn step_weather_hail_shell_spawns_after_frame_gate_loc_6172_61ca() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.frame_index = 0x0896;
        runtime.weather.hail_active = true;
        runtime.weather.hail_period_timer = 100;
        runtime.weather.storm_spawn_cooldown = 1;
        runtime.weather.hail_spawn_timer = 1;
        runtime.weather.storm_free_falling_slots = 5;
        let mut slice = airborne_slice();
        slice.camera.x_pos = 0x0040_0000;
        slice.camera.y_pos = 0x0030_0000;

        let mut expected_rng = runtime.rom_random.clone();
        let cooldown_random = expected_rng.next_byte();
        let spawn_random = expected_rng.next_byte();
        let expected_cooldown = 0x25u8.wrapping_sub(cooldown_random & 0x0F);

        runtime.step_weather(&slice);

        assert_eq!(runtime.projectiles.len(), 1);
        let projectile = &runtime.projectiles[0];
        assert_eq!(
            projectile.kind,
            EnemyProjectileKind::Hail,
            "ASM sub_61CA loc_62C2 should expose one hail projectile from the spawned falling-weather slot",
        );
        assert_eq!(
            projectile.x_pos,
            (0x0040u32 + u32::from(spawn_random) + 0x0028u32) << 16,
        );
        assert_eq!(projectile.y_pos, 0x0030_0000);
        assert_eq!(projectile.x_velocity, -0x0001_0000);
        assert_eq!(projectile.y_velocity, 0x0004_0000);
        assert_eq!(runtime.weather.storm_spawn_cooldown, expected_cooldown);
        assert_eq!(
            runtime.weather.storm_free_falling_slots, 4,
            "ASM loc_62C2 decrements `$FFFFFB3D` after consuming one inactive falling-weather slot",
        );
    }

    #[test]
    fn step_weather_hail_shell_returns_at_frame_threshold_loc_6172() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.frame_index = 0x0873;
        runtime.weather.hail_active = true;
        runtime.weather.hail_period_timer = 100;
        runtime.weather.storm_spawn_cooldown = 1;
        runtime.weather.storm_free_falling_slots = 5;

        runtime.step_weather(&airborne_slice());

        assert!(
            runtime.projectiles.is_empty(),
            "ASM loc_6172 takes the `bls` return branch while Frame_Counter <= $873",
        );
        assert_eq!(
            runtime.weather.storm_spawn_cooldown, 1,
            "ASM loc_6172 should return before sub_61CA decrements the spawn cooldown",
        );
    }

    #[test]
    fn step_weather_hail_shell_skips_inactive_full_slot_period_loc_61a8() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.frame_index = 0x0896;
        runtime.weather.hail_active = false;
        runtime.weather.hail_period_timer = 100;
        runtime.weather.storm_spawn_cooldown = 1;
        runtime.weather.storm_free_falling_slots = 0;
        runtime.weather.storm_falling_slots_sub_61ca =
            [FallingWeatherSlotSub61CA::inactive(6, 0); 5];

        runtime.step_weather(&airborne_slice());

        assert!(
            runtime.projectiles.is_empty(),
            "ASM loc_61A8 returns when inactive hail has all five falling-weather slots free",
        );
        assert_eq!(
            runtime.weather.storm_spawn_cooldown, 1,
            "ASM loc_61A8 should not call sub_61CA in the inactive full-slot branch",
        );
    }

    #[test]
    fn step_weather_replaces_previous_hail_projectiles_only_loc_61ca() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.frame_index = 0x0896;
        runtime.weather.hail_active = true;
        runtime.weather.hail_period_timer = 100;
        runtime.weather.storm_spawn_cooldown = 3;
        runtime.weather.hail_spawn_timer = 3;
        runtime.weather.storm_free_falling_slots = 0;
        runtime.weather.storm_falling_slots_sub_61ca[0] = FallingWeatherSlotSub61CA {
            y_px: 0x0030,
            x_px: 0x0040,
            animation_cooldown: 6,
            animation_frame: 0,
        };
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::Hail,
            x_pos: 0xAAAA_0000,
            ..EnemyProjectile::default()
        });
        runtime.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::Generic,
            x_pos: 0xBBBB_0000,
            ..EnemyProjectile::default()
        });

        runtime.step_weather(&airborne_slice());

        assert!(
            runtime
                .projectiles
                .iter()
                .all(|projectile| projectile.x_pos != 0xAAAA_0000),
            "ASM sub_61CA exports hail from falling-weather RAM each frame; stale hail projectiles must be replaced",
        );
        assert!(
            runtime
                .projectiles
                .iter()
                .any(|projectile| projectile.kind == EnemyProjectileKind::Generic),
            "non-hail projectiles are outside the ASM sub_61CA export path",
        );
        assert!(
            runtime
                .projectiles
                .iter()
                .any(|projectile| projectile.kind == EnemyProjectileKind::Hail),
            "the live falling-weather slot should be re-exported as a fresh hail projectile",
        );
    }

    #[test]
    fn step_weather_hail_spawn_rng_waits_for_inactive_slot_loc_625e() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.frame_index = 0x0896;
        runtime.weather.hail_active = true;
        runtime.weather.hail_period_timer = 100;
        runtime.weather.storm_spawn_cooldown = 1;
        runtime.weather.hail_spawn_timer = 1;
        runtime.weather.storm_free_falling_slots = 5;
        for index in 0..5 {
            runtime.weather.storm_falling_slots_sub_61ca[index] = FallingWeatherSlotSub61CA {
                y_px: 0x0030,
                x_px: 0x0040 + index as i16 * 0x10,
                animation_cooldown: 6,
                animation_frame: 0,
            };
        }
        let mut expected_rng = runtime.rom_random.clone();
        let _cooldown_random = expected_rng.next_byte();
        let expected_next_random = expected_rng.next_byte();

        runtime.step_weather(&airborne_slice());

        assert_eq!(
            runtime.rom_random.next_byte(),
            expected_next_random,
            "ASM loc_626C must not call Get_RandomNumber_byte when loc_625E finds no inactive falling-weather slot",
        );
    }

    #[test]
    fn step_weather_hail_spawn_rng_waits_for_cooldown_zero_loc_623a() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.frame_index = 0x0896;
        runtime.weather.hail_active = true;
        runtime.weather.hail_period_timer = 100;
        runtime.weather.storm_spawn_cooldown = 3;
        runtime.weather.hail_spawn_timer = 3;
        runtime.weather.storm_free_falling_slots = 5;
        runtime.weather.storm_falling_slots_sub_61ca =
            [FallingWeatherSlotSub61CA::inactive(6, 0); 5];
        let mut expected_rng = runtime.rom_random.clone();
        let expected_next_random = expected_rng.next_byte();

        runtime.step_weather(&airborne_slice());

        assert_eq!(
            runtime.weather.storm_spawn_cooldown, 2,
            "ASM loc_623A should only decrement the cooldown while it remains non-zero",
        );
        assert_eq!(
            runtime.rom_random.next_byte(),
            expected_next_random,
            "ASM loc_623A must not call Get_RandomNumber_byte until the spawn cooldown reaches zero",
        );
    }

    #[test]
    fn step_weather_mountain_thunder_reseed_uses_low_frame_byte_loc_613e() {
        let mut assets = manifest_with_levels(1);
        let header = assets.levels[0].header_metadata.as_mut().unwrap();
        header.weather_flags = 2;
        header.background_theme = 7; // ASM constant `Mountain`.
        let mut runtime = NativeRuntime::new(assets);
        runtime.current_level = LevelId(0);
        runtime.weather.weather_type = 2;
        runtime.weather.storm_fade_countdown = 0;
        runtime.weather.storm_thunder_timer = -26;
        runtime.frame_index = 0x08A5;

        runtime.step_weather(&airborne_slice());

        assert_eq!(
            runtime.weather.storm_thunder_timer,
            0x00F7 - 0x00A5,
            "ASM loc_613E subtracts the low frame byte masked by `$F5` when reseeding thunder",
        );
    }

    #[test]
    fn step_weather_foreground_mountain_fade_uses_overlay_palette_loc_60c0_60e0() {
        let output_root = unique_temp_path("weather-mountain-fade");
        let palette_bg_dir = output_root.join("theme/palette_bg");
        let palette_fg_dir = output_root.join("theme/palette_fg");
        std::fs::create_dir_all(&palette_bg_dir).expect("create storm bg palette fixture dir");
        std::fs::create_dir_all(&palette_fg_dir).expect("create storm fg palette fixture dir");
        let saved_storm_words = [
            0x0111, 0x0222, 0x0333, 0x0444, 0x0555, 0x0666, 0x0777, 0x0888,
        ];
        let mountain_overlay_words = [
            0x1001, 0x1002, 0x1003, 0x1004, 0x1005, 0x1006, 0x1007, 0x1008, 0x1009, 0x100A, 0x100B,
            0x100C, 0x100D, 0x100E, 0x100F,
        ];
        write_be_words(
            &palette_bg_dir.join("mountain_storm.bin"),
            &saved_storm_words,
        );
        write_be_words(
            &palette_fg_dir.join("mountain_storm.bin"),
            &mountain_overlay_words,
        );

        let mut assets = manifest_with_levels(1);
        assets.output_root = output_root;
        assets.theme_assets = vec![
            wildside_data::AssetFile {
                relative_path: String::from("palette_bg/mountain_storm.bin"),
                size_bytes: saved_storm_words.len() as u64 * 2,
                sha1: String::from("fixture-weather-storm-bg"),
            },
            wildside_data::AssetFile {
                relative_path: String::from("palette_fg/mountain_storm.bin"),
                size_bytes: mountain_overlay_words.len() as u64 * 2,
                sha1: String::from("fixture-weather-storm-fg"),
            },
        ];
        let header = assets.levels[0].header_metadata.as_mut().unwrap();
        header.weather_flags = 2;
        header.level_theme = 7; // ASM constant `Mountain`.
        header.background_theme = 0;
        let mut runtime = NativeRuntime::new(assets);
        runtime.current_level = LevelId(0);
        runtime.weather.weather_type = 2;
        runtime.weather.storm_fade_countdown = 0x0100;
        runtime.frame_index = 0x04B1;
        runtime.palette_buffer = core::array::from_fn(|index| 0x2000 + index as u16);

        runtime.step_weather(&airborne_slice());

        assert_eq!(
            runtime.palette_buffer_3[0], saved_storm_words[0],
            "ASM loc_60C0 writes the first saved storm palette word to Palette_Buffer_3[0] only when Foreground_theme is Mountain",
        );
        assert_eq!(
            &runtime.palette_buffer_3[0x19..0x20],
            &saved_storm_words[1..],
            "ASM loc_60C6 copies the remaining seven saved storm words to Palette_Buffer_3+$32 on the Mountain foreground path",
        );
        assert_eq!(
            &runtime.palette_buffer_3[1..0x10],
            &mountain_overlay_words,
            "ASM loc_60E0 overlays all fifteen Pal_7B774 foreground words at Palette_Buffer_3+2",
        );
        assert_eq!(
            runtime.weather.storm_fade_countdown, 0x00FC,
            "ASM loc_60E0 tail subtracts four from `$FFFFFADA` after staging the Mountain storm fade",
        );
    }

    #[test]
    fn init_weather_resets_previous_weather_state_when_level_has_no_effect_loc_14dac() {
        let mut assets = manifest_with_levels(1);
        assets.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .weather_flags = 0;
        let mut runtime = NativeRuntime::new(assets);
        runtime.weather.weather_type = 3;
        runtime.weather.storm_spawn_cooldown = 7;
        runtime.weather.storm_free_falling_slots = 4;
        runtime.weather.storm_falling_slots_sub_61ca[0] = FallingWeatherSlotSub61CA {
            y_px: 0x20,
            x_px: 0x40,
            animation_cooldown: 3,
            animation_frame: 2,
        };
        runtime.weather.storm_art_upload_loc_14cd8 =
            Some(upload_storm_art_loc_14CD8(StormArtAssetLoc14CCE::Rain));
        runtime.weather.unknown_art_upload_loc_14df6 =
            Some(record_special_effect_unknown_art_upload_loc_14DF6());
        runtime.weather.hail_active = true;
        runtime.weather.hail_period_timer = 99;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 0,
            y_px: 0,
            height_px: 0,
            first_eruption_pending: true,
            sound_suppressed: false,
            phase: 0,
            timer: 0,
            current_height: 0,
        });

        runtime.init_weather(LevelId(0));

        assert_eq!(runtime.weather.weather_type, 0);
        assert_eq!(runtime.weather.storm_spawn_cooldown, 0);
        assert_eq!(
            runtime.weather.storm_falling_slots_sub_61ca,
            [FallingWeatherSlotSub61CA::default(); FALLING_WEATHER_SLOT_COUNT_SUB_61CA],
            "ASM Init_SpecialEffects skips loc_14DAC for weather 0, so native clears stale storm slot RAM before leaving weather disabled",
        );
        assert_eq!(
            runtime.weather.storm_art_upload_loc_14cd8, None,
            "ASM Init_SpecialEffects weather 0 performs no storm loc_14CD8 upload",
        );
        assert_eq!(
            runtime.weather.unknown_art_upload_loc_14df6, None,
            "ASM Init_SpecialEffects weather 0 performs no loc_14DF6 unknown upload",
        );
        assert!(
            !runtime.weather.hail_active,
            "ASM weather 0 has no `$FFFFFB54` hail-active latch to carry from the previous level",
        );
        assert!(
            runtime.weather.lava_geysers.is_empty(),
            "ASM Init_SpecialEffects weather 0 performs no Init_SpecialEffect_Lava loc_14C6A object allocation",
        );
    }

    #[test]
    fn init_weather_ice_storm_selects_hail_art_palette_and_pattern_loc_14cce_14d1e() {
        let mut assets = manifest_with_levels(1);
        let header = assets.levels[0].header_metadata.as_mut().unwrap();
        header.weather_flags = 2;
        header.background_theme = 2; // ASM constant `Ice`.
        let mut runtime = NativeRuntime::new(assets);

        runtime.init_weather(LevelId(0));

        assert_eq!(
            runtime.weather.storm_art_upload_loc_14cd8,
            Some(StormArtUploadLoc14CD8 {
                asset: StormArtAssetLoc14CCE::Hail,
                asset_label: STORM_ART_HAIL_SOURCE_LABEL_LOC_14CCE,
                vram_word: 0x7000,
            }),
            "ASM loc_14CCE selects the hail art pointer before the shared loc_14CD8 upload when Background_theme is Ice",
        );
        assert_eq!(
            runtime.weather.storm_palette_copy_loc_14cf2,
            Some(StormPaletteCopyLoc14CF2 {
                asset: StormPaletteAssetLoc14CEC::Hail,
                asset_label: STORM_PALETTE_HAIL_LABEL_LOC_14CEC,
                relative_path: STORM_PALETTE_HAIL_PATH_LOC_14CEC,
                palette_buffer_start: STORM_PALETTE_BUFFER_START_LOC_14CF2,
                copied_words: 8,
            }),
            "ASM loc_14CEC selects Pal_7B8CC for Ice storm palettes before loc_14CF2 copies eight words",
        );
        assert_eq!(
            runtime.weather.storm_pattern_upload_loc_14d32,
            Some(StormPatternRuntimeUploadLoc14D32 {
                source_asset_label: STORM_PATTERN_HAIL_TABLE_LABEL_LOC_14D1E.to_owned(),
                vram_tile_addend: 0x0380_0380,
                skipped_header_longwords: 1,
                vdp_command: STORM_PATTERN_VDP_COMMAND_LOC_14D32,
                written_longword_count: 4 * 8 * 8 * 4,
            }),
            "ASM loc_14D1E selects the hail pattern table for Ice backgrounds even when weather is plain storm",
        );
    }

    #[test]
    fn init_weather_type1_runs_lava_initialiser_loc_14c6a() {
        let mut assets = manifest_with_levels(1);
        assets.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .weather_flags = 1;
        let mut runtime = NativeRuntime::new(assets);

        runtime.init_weather(LevelId(0));

        assert_eq!(runtime.weather.weather_type, 1);
        assert!(
            !runtime.weather.lava_geysers.is_empty(),
            "ASM Init_SpecialEffect_Lava loc_14C6A allocates lava geyser objects for lava weather",
        );
        assert_eq!(
            runtime.trace_frame().lava_geyser_art_uploads_loc_14c6a,
            vec![LavaGeyserArtUploadLoc14C6A {
                art_source_label: LAVA_GEYSER_ART_SOURCE_LABEL_LOC_14C6A.to_owned(),
                vram_destination: LAVA_GEYSER_ART_VRAM_DESTINATION_LOC_14C6A,
            }],
            "ASM loc_14C6A uploads ArtComp_983D2_Lava to VRAM `$5F60` during lava init",
        );
        assert_eq!(
            runtime.weather.storm_art_upload_loc_14cd8, None,
            "ASM lava init must not fall through to storm loc_14CD8 uploads",
        );
    }

    #[test]
    fn init_weather_storm_hail_starts_with_inactive_hail_latch_loc_14dac() {
        let mut assets = manifest_with_levels(1);
        assets.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .weather_flags = 3;
        let mut runtime = NativeRuntime::new(assets);
        runtime.weather.hail_active = true;

        runtime.init_weather(LevelId(0));

        assert_eq!(
            runtime.weather.hail_period_timer, 1,
            "ASM loc_14DAC writes `$FFFFFB52=1` before the later sub_6048 hail cadence starts",
        );
        assert!(
            !runtime.weather.hail_active,
            "ASM loc_14DAC uses `sf ($FFFFFB54).w`, so storm+hail init starts with the hail-active latch clear",
        );
    }

    #[test]
    fn init_weather_type4_records_unknown_uploads_and_mode_byte_loc_14df6_14e42() {
        let mut assets = manifest_with_levels(1);
        assets.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .weather_flags = 4;
        let mut runtime = NativeRuntime::new(assets);

        runtime.init_weather(LevelId(0));

        assert_eq!(runtime.weather.weather_type, 4);
        assert_eq!(
            runtime.weather.unknown_art_upload_loc_14df6,
            Some(SpecialEffectUnknownRuntimeArtUploadLoc14DF6 {
                source_asset_label: SPECIAL_EFFECT_UNKNOWN_ART_LABEL_LOC_14DE4.to_owned(),
                vram_word: 0x7000,
                burst_count: 16,
                written_longword_count: 16 * 8,
            }),
            "ASM Init_SpecialEffect_Unknown loc_14DF6 streams unk_9AA50 art to VRAM `$7000`",
        );
        assert_eq!(
            runtime.weather.unknown_plane_fill_loc_14e42,
            Some(SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                vdp_command: SPECIAL_EFFECT_UNKNOWN_PLANE_VDP_COMMAND_LOC_14E42,
                outer_pass_count: 8,
                group_count_per_pass: 4,
                inner_repeat_count: 16,
                written_longword_count: 8 * 4 * 16 * 2,
                foreground_plane_register:
                    SPECIAL_EFFECT_UNKNOWN_FOREGROUND_PLANE_REGISTER_LOC_14E42,
                background_plane_register:
                    SPECIAL_EFFECT_UNKNOWN_BACKGROUND_PLANE_REGISTER_LOC_14E42,
            }),
            "ASM loc_14E42 fills the foreground plane and restores plane registers for weather type 4",
        );
        assert_eq!(
            runtime.weather.storm_weather_mode_byte, 1,
            "ASM loc_14DF6 writes `$FFFFFAD6=1`, distinct from storm loc_14DAC's mode byte 4",
        );
    }

    #[test]
    fn init_murder_wall_preserves_previous_fraction_on_new_wall_loc_140d2() {
        let mut assets = manifest();
        assets.levels[0]
            .header_metadata
            .as_mut()
            .expect("fixture level header")
            .murder_wall_flags = 0b10;
        let mut runtime = NativeRuntime::new(assets);
        runtime.current_level = LevelId(0);
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0x0000_00A5,
            speed: 0x1234,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });

        runtime.init_murder_wall(LevelId(0), 0);

        let wall = runtime
            .murder_wall
            .expect("fixture level has murder wall flags enabled");
        assert_eq!(
            wall.x_pos & 0xFFFF,
            0x00A5,
            "ASM loc_140D2 writes only the integer word of MurderWall_X_pos; the low word is left as existing RAM",
        );
    }
}
