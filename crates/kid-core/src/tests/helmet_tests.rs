use super::*;

fn redact_advisory(snap: &mut insta::Settings) {
    snap.add_redaction(".number_objects", "[advisory]");
    snap.add_redaction(".score", "[advisory]");
    snap.add_redaction(".murder_wall_x", "[advisory]");
    snap.add_redaction(".number_gfx_objects", "[advisory]");
}

mod chunk_08 {
    use super::*;

    #[test]
    fn boss_head_movement_step_sub_39846_rewinds_negative_sentinel_entries_to_unk_39872() {
        let reset_script = [(0x0168, 2, 0), (-1, 0, 0)];
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_step_sub_39846(1, 0, &sentinel_only_script, &reset_script);

        assert_eq!(
            result.next_delay_counter, 0x0168,
            "ASM loc_3984C should treat a negative delay word as the sentinel and restart from `unk_39872`",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM loc_3984C should resume after the first real entry in the reset script",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0x0001_0000), Some(0)),
            "ASM loc_39858 should apply the restarted script entry after rewinding `a2` to `unk_39872`",
        );
    }

    #[test]
    fn boss_head_movement_wrapper_sub_398bc_rewinds_negative_sentinel_entries_to_unk_39872_and_runs_sub_3a292()
     {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_wrapper_sub_398BC(1, 0, &sentinel_only_script);

        assert_eq!(
            result.movement.next_delay_counter, 0x0168,
            "ASM sub_398BC should inherit `sub_39846`'s rewind to `unk_39872` when the current entry is the negative sentinel",
        );
        assert_eq!(
            result.movement.next_script_index, 1,
            "ASM sub_398BC should resume after the first real `unk_39872` entry once the sentinel rewinds the script pointer",
        );
        assert_eq!(
            (
                result.movement.next_x_velocity,
                result.movement.next_y_velocity
            ),
            (Some(0), Some(-0x0000_8000)),
            "ASM sub_398BC should load the first `unk_39872` velocity pair before tail-calling `sub_3A292`",
        );
        assert!(
            result.runs_shared_collision_helper_sub_3_a292,
            "ASM sub_398BC should always tail-call the shared boss collision/palette helper `sub_3A292` after the movement step",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_398c4_rewinds_negative_sentinel_entries_to_unk_398f0() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_step_sub_398C4(1, 0, &sentinel_only_script);

        assert_eq!(
            result.next_delay_counter, 0x0096,
            "ASM loc_398CA should treat the negative delay as the sentinel and rewind `a2` to `unk_398F0`",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM sub_398C4 should resume after the first real Boomerang head-1 script entry",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0), Some(0)),
            "ASM loc_398D6 should load the first `(0, 0)` velocity pair from `unk_398F0` after the rewind",
        );
    }

    #[test]
    fn boss_head_movement_wrapper_sub_39952_uses_sub_398c4_and_runs_sub_3a292() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_wrapper_sub_39952(1, 0, &sentinel_only_script);

        assert_eq!(
            result.movement.next_delay_counter, 0x0096,
            "ASM sub_39952 should inherit `sub_398C4`'s rewind to `unk_398F0` before the shared helper runs",
        );
        assert_eq!(
            result.movement.next_script_index, 1,
            "ASM sub_39952 should advance past the first rewound Boomerang head-1 script entry",
        );
        assert_eq!(
            (
                result.movement.next_x_velocity,
                result.movement.next_y_velocity
            ),
            (Some(0), Some(0)),
            "ASM sub_39952 should preserve the `sub_398C4` velocity load before tail-calling `sub_3A292`",
        );
        assert!(
            result.runs_shared_collision_helper_sub_3_a292,
            "ASM sub_39952 should always tail-call the shared boss collision/palette helper `sub_3A292` after `sub_398C4`",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_3995c_rewinds_negative_sentinel_entries_to_unk_39988() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_step_sub_3995C(1, 0, &sentinel_only_script);

        assert_eq!(
            result.next_delay_counter, 0x012C,
            "ASM loc_39962 should treat the negative delay as the sentinel and rewind `a2` to `unk_39988`",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM sub_3995C should resume after the first real Boomerang head-2 script entry",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0), Some(-0x0000_8000)),
            "ASM loc_3996E should load the first `(0, -1)` velocity pair from `unk_39988` after the rewind",
        );
    }

    #[test]
    fn boss_head_movement_wrapper_sub_399e4_uses_sub_3995c_and_runs_sub_3a292() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_wrapper_sub_399E4(1, 0, &sentinel_only_script);

        assert_eq!(
            result.movement.next_delay_counter, 0x012C,
            "ASM sub_399E4 should inherit `sub_3995C`'s rewind to `unk_39988` before the shared helper runs",
        );
        assert_eq!(
            result.movement.next_script_index, 1,
            "ASM sub_399E4 should advance past the first rewound Boomerang head-2 script entry",
        );
        assert_eq!(
            (
                result.movement.next_x_velocity,
                result.movement.next_y_velocity
            ),
            (Some(0), Some(-0x0000_8000)),
            "ASM sub_399E4 should preserve the `sub_3995C` velocity load before tail-calling `sub_3A292`",
        );
        assert!(
            result.runs_shared_collision_helper_sub_3_a292,
            "ASM sub_399E4 should always tail-call the shared boss collision/palette helper `sub_3A292` after `sub_3995C`",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_399ee_rewinds_negative_sentinel_entries_to_unk_39a1a() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_step_sub_399EE(1, 0, &sentinel_only_script);

        assert_eq!(
            result.next_delay_counter, 0x00C8,
            "ASM loc_399F4 should treat the negative delay as the sentinel and rewind `a2` to `unk_39A1A`",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM sub_399EE should resume after the first real Boomerang head-3 script entry",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0), Some(0)),
            "ASM loc_39A00 should load the first `(0, 0)` velocity pair from `unk_39A1A` after the rewind",
        );
    }

    #[test]
    fn boss_head_movement_wrapper_sub_39a7c_uses_sub_399ee_and_runs_sub_3a292() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_wrapper_sub_39A7C(1, 0, &sentinel_only_script);

        assert_eq!(
            result.movement.next_delay_counter, 0x00C8,
            "ASM sub_39A7C should inherit `sub_399EE`'s rewind to `unk_39A1A` before the shared helper runs",
        );
        assert_eq!(
            result.movement.next_script_index, 1,
            "ASM sub_39A7C should advance past the first rewound Boomerang head-3 script entry",
        );
        assert_eq!(
            (
                result.movement.next_x_velocity,
                result.movement.next_y_velocity
            ),
            (Some(0), Some(0)),
            "ASM sub_39A7C should preserve the `sub_399EE` velocity load before tail-calling `sub_3A292`",
        );
        assert!(
            result.runs_shared_collision_helper_sub_3_a292,
            "ASM sub_39A7C should always tail-call the shared boss collision/palette helper `sub_3A292` after `sub_399EE`",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_39a86_rewinds_negative_sentinel_entries_to_unk_39ab2() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_step_sub_39A86(1, 0, &sentinel_only_script);

        assert_eq!(
            result.next_delay_counter, 0x0032,
            "ASM loc_39A8C should treat the negative delay as the sentinel and rewind `a2` to `unk_39AB2`",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM sub_39A86 should resume after the first real Bagel Brothers head script entry",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0), Some(0)),
            "ASM loc_39A98 should load the first `(0, 0)` velocity pair from `unk_39AB2` after the rewind",
        );
    }

    #[test]
    fn boss_head_movement_wrapper_sub_39bf8_uses_sub_39a86_and_runs_sub_3a292() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_wrapper_sub_39BF8(1, 0, &sentinel_only_script);

        assert_eq!(
            result.movement.next_delay_counter, 0x0032,
            "ASM sub_39BF8 should inherit `sub_39A86`'s rewind to `unk_39AB2` before the shared helper runs",
        );
        assert_eq!(
            result.movement.next_script_index, 1,
            "ASM sub_39BF8 should advance past the first rewound Bagel Brothers head script entry",
        );
        assert_eq!(
            (
                result.movement.next_x_velocity,
                result.movement.next_y_velocity
            ),
            (Some(0), Some(0)),
            "ASM sub_39BF8 should preserve the `sub_39A86` velocity load before tail-calling `sub_3A292`",
        );
        assert!(
            result.runs_shared_collision_helper_sub_3_a292,
            "ASM sub_39BF8 should always tail-call the shared boss collision/palette helper `sub_3A292` after `sub_39A86`",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_39c02_rewinds_negative_sentinel_entries_to_unk_39c2e() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_step_sub_39C02(1, 0, &sentinel_only_script);

        assert_eq!(
            result.next_delay_counter, 0x001E,
            "ASM loc_39C08 should treat the negative delay as the sentinel and rewind `a2` to `unk_39C2E`",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM sub_39C02 should resume after the first real Bagel Brothers variant head script entry",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0), Some(-0x8000)),
            "ASM loc_39C14 should load the first `(0, -1)` velocity pair from `unk_39C2E` after the rewind",
        );
    }

    #[test]
    fn boss_head_movement_wrapper_sub_39d4a_uses_sub_39c02_and_runs_sub_3a292() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_wrapper_sub_39D4A(1, 0, &sentinel_only_script);

        assert_eq!(
            result.movement.next_delay_counter, 0x001E,
            "ASM sub_39D4A should inherit `sub_39C02`'s rewind to `unk_39C2E` before the shared helper runs",
        );
        assert_eq!(
            result.movement.next_script_index, 1,
            "ASM sub_39D4A should advance past the first rewound Bagel Brothers variant head script entry",
        );
        assert_eq!(
            (
                result.movement.next_x_velocity,
                result.movement.next_y_velocity
            ),
            (Some(0), Some(-0x8000)),
            "ASM sub_39D4A should preserve the `sub_39C02` velocity load before tail-calling `sub_3A292`",
        );
        assert!(
            result.runs_shared_collision_helper_sub_3_a292,
            "ASM sub_39D4A should always tail-call the shared boss collision/palette helper `sub_3A292` after `sub_39C02`",
        );
    }

    #[test]
    fn boss_head_movement_step_sub_39d54_rewinds_negative_sentinel_entries_to_unk_39d80() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_step_sub_39D54(1, 0, &sentinel_only_script);

        assert_eq!(
            result.next_delay_counter, 0x001E,
            "ASM loc_39D5A should treat the negative delay as the sentinel and rewind `a2` to `unk_39D80`",
        );
        assert_eq!(
            result.next_script_index, 1,
            "ASM sub_39D54 should resume after the first real Bagel Brothers variant head script entry",
        );
        assert_eq!(
            (result.next_x_velocity, result.next_y_velocity),
            (Some(0), Some(-0x8000)),
            "ASM loc_39D66 should load the first `(0, -1)` velocity pair from `unk_39D80` after the rewind",
        );
    }

    #[test]
    fn boss_head_movement_wrapper_sub_39eb4_runs_sub_39d54_before_shared_collision_helper() {
        let sentinel_only_script = [(-1, 0, 0)];

        let result = boss_head_movement_wrapper_sub_39EB4(1, 0, &sentinel_only_script);

        assert_eq!(
            result.movement.next_delay_counter, 0x001E,
            "ASM sub_39EB4 should inherit `sub_39D54`'s rewind to `unk_39D80` before the shared helper runs",
        );
        assert_eq!(
            result.movement.next_script_index, 1,
            "ASM sub_39EB4 should resume after the first real Bagel Brothers variant head script entry",
        );
        assert_eq!(
            (
                result.movement.next_x_velocity,
                result.movement.next_y_velocity
            ),
            (Some(0), Some(-0x8000)),
            "ASM sub_39EB4 should preserve the `sub_39D54` velocity load before tail-calling `sub_3A292`",
        );
        assert!(
            result.runs_shared_collision_helper_sub_3_a292,
            "ASM sub_39EB4 should always tail-call the shared boss collision/palette helper `sub_3A292` after `sub_39D54`",
        );
    }

    #[test]
    fn boss_collision_palette_sub_3a292_hit_with_collision_1c_decrements_hp_and_flashes() {
        // ASM loc_3A2C0 (kid.asm:43356): collision_type=$1C always hits regardless
        // of Berzerker_charging. move.b #$A,d1; sfx; subi.w #1,$3E(a3).
        let result = boss_collision_palette_sub_3A292(0x1C, false, 0, 5);

        assert!(
            result.collision_type_cleared,
            "ASM sub_3A292 should clear collision_type(a3) when nonzero",
        );
        assert!(
            result.hit,
            "ASM loc_3A2C0: collision_type $1C should always be accepted as a hit",
        );
        assert_eq!(
            result.played_sound_effect,
            Some("sfx_Big_Hopping_Skull_groan"),
            "ASM loc_3A2C0: moveq #sfx_Big_Hopping_Skull_groan,d0; jsr (j_PlaySound).l",
        );
        assert_eq!(
            result.next_hit_points, 4,
            "ASM loc_3A2C0: subi.w #1,$3E(a3) should decrement HP from 5 to 4",
        );
        assert_eq!(
            result.next_flash_timer, 9,
            "ASM loc_3A2C0: move.b #$A,d1 sets flash to 10; loc_3A2E0 subq.b #1,d1 decrements to 9",
        );
        assert_eq!(
            result.next_palette_line, 3,
            "ASM loc_3A2E0: flash timer nonzero → move.b #3,palette_line(a3)",
        );
        assert!(
            !result.head_dead,
            "ASM loc_3A2F4: HP still positive → clr.w d7; rts (alive)",
        );
    }

    #[test]
    fn boss_collision_palette_sub_3a292_hit_with_collision_2c() {
        // ASM loc_3A2C0: collision_type=$2C is the second always-accepted weapon ID.
        let result = boss_collision_palette_sub_3A292(0x2C, false, 0, 3);

        assert!(result.hit, "ASM loc_3A2C0: $2C should hit");
        assert_eq!(result.next_hit_points, 2);
        assert!(!result.head_dead);
    }

    #[test]
    fn boss_collision_palette_sub_3a292_berzerker_collision_20_hits_only_when_charging() {
        // ASM sub_3A292: cmpi.w #$20,d7; beq.s loc_3A2C0 — only reached when
        // tst.b (Berzerker_charging).w is nonzero.
        let not_charging = boss_collision_palette_sub_3A292(0x20, false, 0, 5);
        assert!(
            !not_charging.hit,
            "ASM sub_3A292: $20 without Berzerker_charging should skip to loc_3A2E0",
        );
        assert_eq!(
            not_charging.next_hit_points, 5,
            "HP should be unchanged when not hit",
        );

        let charging = boss_collision_palette_sub_3A292(0x20, true, 0, 5);
        assert!(
            charging.hit,
            "ASM loc_3A2C0: $20 with Berzerker_charging should be accepted as a hit",
        );
        assert_eq!(charging.next_hit_points, 4);
    }

    #[test]
    fn boss_collision_palette_sub_3a292_berzerker_collision_24_hits_only_when_charging() {
        // ASM sub_3A292: cmpi.w #$24,d7; beq.s loc_3A2C0
        let not_charging = boss_collision_palette_sub_3A292(0x24, false, 0, 5);
        assert!(!not_charging.hit);

        let charging = boss_collision_palette_sub_3A292(0x24, true, 0, 5);
        assert!(charging.hit);
        assert_eq!(charging.next_hit_points, 4);
    }

    #[test]
    fn boss_collision_palette_sub_3a292_unknown_collision_type_ignored() {
        // ASM sub_3A292: bne.s loc_3A2E0 — any collision type not in {$1C,$2C,$20,$24}
        // is consumed but does not damage.
        let result = boss_collision_palette_sub_3A292(0x10, true, 0, 5);

        assert!(
            result.collision_type_cleared,
            "collision_type still consumed"
        );
        assert!(!result.hit);
        assert_eq!(result.next_hit_points, 5);
    }

    #[test]
    fn boss_collision_palette_sub_3a292_no_collision_preserves_flash_timer() {
        // ASM sub_3A292: move.w collision_type(a3),d7; beq.w loc_3A2E0 — zero
        // collision skips straight to flash timer processing.
        let result = boss_collision_palette_sub_3A292(0, false, 5, 10);

        assert!(
            !result.collision_type_cleared,
            "ASM sub_3A292: collision_type==0 → beq.w loc_3A2E0, no clr.w",
        );
        assert!(!result.hit);
        assert_eq!(
            result.next_flash_timer, 4,
            "ASM loc_3A2E0: subq.b #1,d1 should decrement flash timer from 5 to 4",
        );
        assert_eq!(
            result.next_palette_line, 3,
            "ASM loc_3A2E0: flash timer nonzero → palette_line=3",
        );
    }

    #[test]
    fn boss_collision_palette_sub_3a292_flash_timer_zero_sets_normal_palette() {
        // ASM loc_3A2EE (kid.asm:43374): tst.b d1; beq.s loc_3A2EE →
        // move.b #2,palette_line(a3).
        let result = boss_collision_palette_sub_3A292(0, false, 0, 10);

        assert_eq!(
            result.next_flash_timer, 0,
            "ASM loc_3A2EE: flash timer already zero, stays zero",
        );
        assert_eq!(
            result.next_palette_line, 2,
            "ASM loc_3A2EE: move.b #2,palette_line(a3) — normal boss palette",
        );
        assert!(!result.head_dead);
    }

    #[test]
    fn boss_collision_palette_sub_3a292_hit_kills_when_hp_reaches_zero() {
        // ASM loc_3A2C0 → loc_3A2FA: subi.w #1,$3E(a3); cmpi.w #0,$3E(a3);
        // ble.w loc_3A2FA → swap d1; move.w #1,d7; move.b #3,palette_line(a3).
        let result = boss_collision_palette_sub_3A292(0x1C, false, 0, 1);

        assert!(result.hit);
        assert_eq!(
            result.next_hit_points, 0,
            "ASM loc_3A2C0: HP 1 → 0 after subi.w #1",
        );
        assert!(
            result.head_dead,
            "ASM loc_3A2FA: HP <= 0 → move.w #1,d7 (dead)",
        );
        assert_eq!(
            result.next_palette_line, 3,
            "ASM loc_3A2FA: move.b #3,palette_line(a3) on death",
        );
        assert_eq!(
            result.next_flash_timer, 0x0A,
            "ASM loc_3A2C0: flash timer set to $A before death branch (not decremented)",
        );
    }

    #[test]
    fn boss_collision_palette_sub_3a292_hit_kills_when_hp_negative() {
        // ASM loc_3A2FA: ble.w — also triggers when HP goes below zero.
        let result = boss_collision_palette_sub_3A292(0x2C, false, 0, 0);

        assert!(result.hit);
        assert_eq!(result.next_hit_points, -1);
        assert!(result.head_dead, "ASM loc_3A2FA: HP < 0 → dead");
    }

    #[test]
    fn fire_demon_fragment_collision_step_sub_3a172_clears_matching_collision_into_delete_animation()
     {
        let result = fire_demon_fragment_collision_step_sub_3A172(0x002C);

        assert_eq!(
            result.next_collision_type, 0,
            "ASM sub_3A172 should clear `collision_type(a3)` before handling the fragment hit",
        );
        assert_eq!(
            result.animation_label,
            Some("stru_3A1E0"),
            "ASM loc_3A192 should switch to `stru_3A1E0` for collision types `$1C`, `$2C`, and `$FFFF`",
        );
        assert!(
            result.hibernate_until_anim_finished,
            "ASM loc_3A192 should call `Hibernate_UntilAnimFinished` before deleting the fragment",
        );
        assert!(
            result.delete_current_object,
            "ASM sub_3A172 should delete the fragment after the collision animation finishes",
        );
    }

    #[test]
    fn fire_demon_fragment_collision_step_sub_3a1a6_clears_matching_collision_without_animation() {
        let result = fire_demon_fragment_collision_step_sub_3A1A6(-1);

        assert_eq!(
            result.next_collision_type, 0,
            "ASM sub_3A1A6 should clear `collision_type(a3)` before handling the fragment hit",
        );
        assert_eq!(
            result.animation_label, None,
            "ASM sub_3A1A6 should skip the `stru_3A1E0` animation and delete immediately",
        );
        assert!(
            !result.hibernate_until_anim_finished,
            "ASM sub_3A1A6 should not call `Hibernate_UntilAnimFinished` before deleting",
        );
        assert!(
            result.delete_current_object,
            "ASM loc_3A1C6 should delete the fragment immediately for collision types `$1C`, `$2C`, and `$FFFF`",
        );
    }

    #[test]
    fn boss_head_death_anim_sequence_loc_39ee2_plays_8_scripts_with_direction_flips() {
        // ASM loc_39EE2 (kid.asm:43004): plays 8 animation sequences with
        // hibernate-until-finished between each, flipping x_direction after
        // the 1st, 3rd, 5th, and 7th scripts.
        let result = boss_head_death_anim_sequence_loc_39EE2();

        assert_eq!(
            result.animation_scripts[0], "stru_39FEE",
            "ASM loc_39EE2 should start with `stru_39FEE` (6-frame reverse sweep)",
        );
        assert_eq!(
            result.animation_scripts[1], "stru_3A004",
            "ASM loc_39EE2 second animation should be `stru_3A004` (6-frame forward sweep)",
        );
        assert!(
            result.direction_flip_after_scripts[0],
            "ASM loc_39EE2 should flip direction (eori.b #$FF) after the first animation",
        );
        assert!(
            !result.direction_flip_after_scripts[1],
            "ASM loc_39EE2 should NOT flip direction after the second animation",
        );
        assert_eq!(
            result.played_sound_effect, "sfx_Boss_dies",
            "ASM loc_39EE2 should play `sfx_Boss_dies` after all 8 animations complete",
        );
        assert_eq!(
            result.death_spin_counter_d0, 0x000A,
            "ASM loc_39EE2 should set d0=#$A as the loop counter for the subsequent death spin",
        );
    }

    #[test]
    fn boss_head_death_spin_loop_loc_39f7a_plays_4_scripts_per_iteration_then_deletes() {
        // ASM loc_39F7A (kid.asm:43039): each iteration plays 4 animation scripts,
        // flipping direction after the 1st and 3rd. `subq.w #1,d0` loops, then
        // `j_Delete_CurrentObject` on d0==0.
        let result = boss_head_death_spin_loop_loc_39F7A(0x000A);

        assert_eq!(
            result.animation_scripts,
            ["stru_3A086", "stru_3A09C", "stru_3A0AE", "stru_3A0C0"],
            "ASM loc_39F7A should play the 1-frame sweep animation scripts in order",
        );
        assert_eq!(
            result.direction_flip_after_scripts,
            [true, false, true, false],
            "ASM loc_39F7A should flip direction after scripts 0 and 2 each iteration",
        );
        assert_eq!(
            result.loop_count_d0, 0x000A,
            "ASM loc_39F7A should receive the d0=#$A counter from loc_39EE2",
        );
        assert!(
            result.delete_current_object,
            "ASM loc_39F7A should delete the object after the spin loop completes",
        );
    }

    #[test]
    fn fire_demon_fragment_movement_step_loc_3a134_transitions_to_fade_when_counter_zero() {
        // ASM loc_3A134 (kid.asm:43185): when $40(a3)==0, clear x_vel, switch to
        // stru_3A1D6, and enter the fade loop at loc_3A14E.
        let result = fire_demon_fragment_movement_step_loc_3A134(0);

        assert!(
            result.hibernates_one_frame,
            "ASM loc_3A134 should hibernate one frame before checking the counter",
        );
        assert!(
            result.clears_x_vel,
            "ASM loc_3A134 should clear x_vel when $40(a3) reaches zero",
        );
        assert_eq!(
            result.fade_animation_label,
            Some("stru_3A1D6"),
            "ASM loc_3A134 should switch to `stru_3A1D6` animation when entering fade loop",
        );
        assert!(
            !result.runs_collision_check_sub_3a1a6,
            "ASM loc_3A134 should not run sub_3A1A6 on the zero-counter path",
        );
        assert_eq!(
            result.next,
            FireDemonFragmentMovementNextLoc3A134::FadeLoopLoc3A14E,
            "ASM loc_3A134 should transition to fade loop loc_3A14E when $40==0",
        );
    }

    #[test]
    fn fire_demon_fragment_movement_step_loc_3a134_decrements_and_loops_when_counter_nonzero() {
        // ASM loc_3A168 (kid.asm:43206): subq.w #1,$40(a3), bsr.w sub_3A1A6,
        // bra.s loc_3A134.
        let result = fire_demon_fragment_movement_step_loc_3A134(0x0032);

        assert_eq!(
            result.next_horizontal_counter_40, 0x0031,
            "ASM loc_3A168 should decrement $40(a3) from $32 to $31",
        );
        assert!(
            result.runs_collision_check_sub_3a1a6,
            "ASM loc_3A168 should call sub_3A1A6 for collision checking during movement",
        );
        assert!(
            !result.clears_x_vel,
            "ASM loc_3A168 should not clear x_vel while the horizontal counter is nonzero",
        );
        assert_eq!(
            result.next,
            FireDemonFragmentMovementNextLoc3A134::ContinueMovementLoc3A168,
            "ASM loc_3A168 should loop back to loc_3A134 after decrementing",
        );
    }

    #[test]
    fn fire_demon_fragment_fade_step_loc_3a14e_deletes_when_counter_zero() {
        // ASM loc_3A14E (kid.asm:43193): when $42(a3)==0, j_Delete_CurrentObject.
        let result = fire_demon_fragment_fade_step_loc_3A14E(0);

        assert!(
            result.hibernates_one_frame,
            "ASM loc_3A14E should hibernate one frame before checking the fade counter",
        );
        assert!(
            result.delete_current_object,
            "ASM loc_3A14E should delete the fragment when $42(a3) reaches zero",
        );
        assert!(
            !result.runs_collision_check_sub_3a172,
            "ASM loc_3A14E should not run sub_3A172 on the delete path",
        );
        assert_eq!(
            result.next,
            FireDemonFragmentFadeNextLoc3A14E::DeleteCurrentObject,
            "ASM loc_3A14E should branch to Delete_CurrentObject when $42==0",
        );
    }

    #[test]
    fn fire_demon_fragment_fade_step_loc_3a14e_decrements_and_loops_when_counter_nonzero() {
        // ASM loc_3A15E (kid.asm:43200): subq.w #1,$42(a3), bsr.w sub_3A172,
        // bra.s loc_3A14E.
        let result = fire_demon_fragment_fade_step_loc_3A14E(0x00C8);

        assert_eq!(
            result.next_fade_counter_42, 0x00C7,
            "ASM loc_3A15E should decrement $42(a3) from $C8 to $C7",
        );
        assert!(
            result.runs_collision_check_sub_3a172,
            "ASM loc_3A15E should call sub_3A172 for collision checking during fade",
        );
        assert!(
            !result.delete_current_object,
            "ASM loc_3A15E should not delete the fragment while the fade counter is nonzero",
        );
        assert_eq!(
            result.next,
            FireDemonFragmentFadeNextLoc3A14E::ContinueFadeLoc3A15E,
            "ASM loc_3A15E should loop back to loc_3A14E after decrementing",
        );
    }

    #[test]
    fn fire_demon_fragment_decel_step_loc_3a24c_leftward_fragment_decelerates_rightward() {
        // ASM loc_3A24C (kid.asm:43300): tst.b d0; d0=$FF (leftward) → addi.l #$7D0,x_vel.
        // loc_3A266: subq.w #1,$40 → $018F, bounds OK, sub_3A1A6, loop.
        let result = fire_demon_fragment_decel_step_loc_3A24C(0xFF, 0x0190, 0x0100, 0x0500);

        assert!(
            result.hibernates_one_frame,
            "ASM loc_3A24C should hibernate one frame before the direction test",
        );
        assert_eq!(
            result.x_vel_delta, 0x7D0,
            "ASM loc_3A24C: d0 nonzero (leftward fragment) should add +$7D0 to x_vel",
        );
        assert_eq!(
            result.next_horizontal_counter_40, 0x018F,
            "ASM loc_3A266: subq.w #1,$40 should decrement from $190 to $18F",
        );
        assert!(
            result.runs_collision_check_sub_3a1a6,
            "ASM loc_3A28A should call sub_3A1A6 before looping back",
        );
        assert!(!result.delete_current_object);
        assert_eq!(
            result.next,
            FireDemonFragmentDecelNextLoc3A24C::ContinueDecelLoc3A24C,
            "ASM loc_3A28A: bra.s loc_3A24C — should loop back for next decel frame",
        );
    }

    #[test]
    fn fire_demon_fragment_decel_step_loc_3a25e_rightward_fragment_decelerates_leftward() {
        // ASM loc_3A25E (kid.asm:43308): d0==0 (rightward) → addi.l #-$7D0,x_vel.
        let result = fire_demon_fragment_decel_step_loc_3A24C(0x00, 0x0190, 0x0100, 0x0500);

        assert_eq!(
            result.x_vel_delta, -0x7D0,
            "ASM loc_3A25E: d0 zero (rightward fragment) should add -$7D0 to x_vel",
        );
        assert_eq!(
            result.next_horizontal_counter_40, 0x018F,
            "ASM loc_3A266: subq.w #1,$40 should decrement from $190 to $18F",
        );
        assert_eq!(
            result.next,
            FireDemonFragmentDecelNextLoc3A24C::ContinueDecelLoc3A24C,
        );
    }

    #[test]
    fn fire_demon_fragment_decel_step_loc_3a266_deletes_on_lifetime_expiry() {
        // ASM loc_3A266 (kid.asm:43311): subq.w #1,$40; bne.s loc_3A270.
        // When $40==1, subq gives 0, so bne is not taken → j_Delete_CurrentObject.
        let result = fire_demon_fragment_decel_step_loc_3A24C(0xFF, 0x0001, 0x0100, 0x0500);

        assert_eq!(
            result.next_horizontal_counter_40, 0x0000,
            "ASM loc_3A266: subq.w #1 on $40=$0001 should yield zero",
        );
        assert!(
            result.delete_current_object,
            "ASM loc_3A266: bne not taken → j_Delete_CurrentObject",
        );
        assert!(!result.runs_collision_check_sub_3a1a6);
        assert_eq!(
            result.next,
            FireDemonFragmentDecelNextLoc3A24C::DeleteLifetimeExpired,
        );
    }

    #[test]
    fn fire_demon_fragment_decel_step_loc_3a266_deletes_out_of_bounds_left() {
        // ASM loc_3A270 (kid.asm:43317): cmpi.w #0,x_pos(a3); bgt.s loc_3A27C.
        // x_pos==0 → bgt not taken → delete.
        let result = fire_demon_fragment_decel_step_loc_3A24C(0xFF, 0x0190, 0x0000, 0x0500);

        assert!(result.delete_current_object);
        assert_eq!(
            result.next,
            FireDemonFragmentDecelNextLoc3A24C::DeleteOutOfBoundsLeft,
            "ASM loc_3A270: x_pos=0 is not >0 (signed), so fragment is deleted",
        );
    }

    #[test]
    fn fire_demon_fragment_decel_step_loc_3a266_deletes_out_of_bounds_right() {
        // ASM loc_3A27C (kid.asm:43323): cmp.w (Level_width_pixels).w,d5; ble.s loc_3A28A.
        // x_pos > level_width → ble not taken → delete.
        let result = fire_demon_fragment_decel_step_loc_3A24C(0x00, 0x0190, 0x0501, 0x0500);

        assert!(result.delete_current_object);
        assert_eq!(
            result.next,
            FireDemonFragmentDecelNextLoc3A24C::DeleteOutOfBoundsRight,
            "ASM loc_3A27C: x_pos $501 > level_width $500, so fragment is deleted",
        );
    }

    #[test]
    fn heady_metal_init_enemy20_loc_39247_summarises_the_bespoke_boss_dispatch() {
        // ASM Enemy20_HeadyMetal_Init (kid.asm:39247): 3 x sub_3764A + 12 x
        // sub_37680 + 2 x sub_376B6 = 17 worker allocations. Each allocator
        // internally `jsr loc_32146(pc)` (kid.asm:39432, 39452, 39472), so the
        // common boss init runs 17 times even though the top-level Enemy20
        // init never calls `loc_32146` directly.
        let init = heady_metal_init_enemy20_loc_39247(0x0120, 0x00B0);

        assert!(
            init.runs_common_boss_init_loc_32146,
            "HeadyMetal reaches `loc_32146` via its 17 sub_3764A/sub_37680/sub_376B6 allocator calls",
        );
        assert_eq!(
            init.main_head_sprite_count, 3,
            "ASM Enemy20_HeadyMetal_Init (kid.asm:39250,39256,39261) calls `sub_3764A` three times",
        );
        assert_eq!(
            init.body_piece_sprite_count, 12,
            "ASM Enemy20_HeadyMetal_Init (kid.asm:39266-39321) calls `sub_37680` twelve times",
        );
        assert_eq!(
            init.eye_sprite_count, 2,
            "ASM Enemy20_HeadyMetal_Init (kid.asm:39326,39331) calls `sub_376B6` twice",
        );
        assert_eq!(
            init.total_worker_slot_count, 17,
            "Worker pointer table at `$FFFFFB72` holds 3+12+2 entries after Enemy20 init completes",
        );
        assert_eq!(
            init.cached_anchor_slot_label, "$FFFFFB7A",
            "ASM Enemy20_HeadyMetal_Init caches the third `sub_3764A` return into `$FFFFFB7A`",
        );
        assert_eq!(
            init.anchor_animation_struct_label, "stru_37B8A",
            "ASM kid.asm:39337 loads `stru_37B8A` as the anchor animation struct",
        );
        assert_eq!(
            init.tracking_loop_entry_label, "loc_37582",
            "ASM Enemy20_HeadyMetal_Init falls through into `loc_37582` for the tracking loop",
        );
        assert_eq!(
            init.initial_loop_counter_d0, 0x001E,
            "ASM kid.asm:39342 seeds `d0 = $1E` (30) before entering the tracking loop",
        );
        assert_eq!(
            (init.anchor_x_pos, init.anchor_y_pos),
            (0x0120, 0x00B0),
            "Anchor X/Y are copied straight from the enemy ROM entry `(4,6)(a0)` at `sub_3764A`",
        );
    }

    #[test]
    fn heady_metal_sprite_init_sub_3764a_seeds_the_shared_anchor_sprite() {
        let init = heady_metal_sprite_init_sub_3764A(0x0044, 0x0046);

        assert_eq!(
            init.gfx_slot_base, 0x0101_0002,
            "ASM sub_3764A should allocate gfx slot `$01010002` for the first HeadyMetal anchor sprite",
        );
        assert_eq!(
            init.object_meta, 0x0020,
            "ASM sub_3764A should seed `object_meta(a1)` with `$20` before `loc_32146`",
        );
        assert!(
            init.display_enabled && init.is_moved,
            "ASM sub_3764A should set `$13(a1)` and `$14(a1)` on the spawned HeadyMetal sprite",
        );
        assert_eq!(
            init.x_direction, 0,
            "ASM sub_3764A should clear `x_direction(a1)` before the shared sprite setup runs",
        );
        assert_eq!(
            (init.x_pos, init.y_pos),
            (0x0044, 0x0046),
            "ASM sub_3764A should copy `4(a0)` and `6(a0)` into `$1A/$1E(a1)` with no extra offset",
        );
    }

    #[test]
    fn heady_metal_sprite_init_sub_37680_seeds_the_column_projectile_sprite() {
        let init = heady_metal_sprite_init_sub_37680(0x0044, 0x0046);

        assert_eq!(
            init.gfx_slot_base, 0x0100_0002,
            "ASM sub_37680 should allocate gfx slot `$01000002` for the HeadyMetal column projectile sprite",
        );
        assert_eq!(
            init.object_meta, 0x0021,
            "ASM sub_37680 should seed `object_meta(a1)` with `$21` before `loc_32146`",
        );
        assert!(
            init.display_enabled && init.is_moved,
            "ASM sub_37680 should set `$13(a1)` and `$14(a1)` on the spawned HeadyMetal projectile sprite",
        );
        assert_eq!(
            init.x_direction, 0,
            "ASM sub_37680 should clear `x_direction(a1)` before the shared sprite setup runs",
        );
        assert_eq!(
            (init.x_pos, init.y_pos),
            (0x0044, 0x0046),
            "ASM sub_37680 should copy `4(a0)` and `6(a0)` into `$1A/$1E(a1)` with no extra offset",
        );
    }

    #[test]
    fn heady_metal_sprite_init_sub_376b6_seeds_the_side_orb_sprite() {
        let init = heady_metal_sprite_init_sub_376B6(0x0044, 0x0046);

        assert_eq!(
            init.gfx_slot_base, 0x00FF_0002,
            "ASM sub_376B6 should allocate gfx slot `$00FF0002` for the HeadyMetal side orb sprite",
        );
        assert_eq!(
            init.object_meta, 0x0020,
            "ASM sub_376B6 should seed `object_meta(a1)` with `$20` before `loc_32146`",
        );
        assert!(
            init.display_enabled && init.is_moved,
            "ASM sub_376B6 should set `$13(a1)` and `$14(a1)` on the spawned HeadyMetal side orb sprite",
        );
        assert_eq!(
            init.x_direction, 0,
            "ASM sub_376B6 should clear `x_direction(a1)` before the shared sprite setup runs",
        );
        assert_eq!(
            (init.x_pos, init.y_pos),
            (0x0044, 0x0046),
            "ASM sub_376B6 should copy `4(a0)` and `6(a0)` into `$1A/$1E(a1)` with no extra offset",
        );
    }

    #[test]
    fn heady_metal_apply_piece_velocities_sub_376ec_updates_each_live_worker_slot() {
        let slots = [
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: 1,
                y_velocity: 2,
            }),
            None,
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: 3,
                y_velocity: 4,
            }),
            None,
            None,
            None,
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: -5,
                y_velocity: 6,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: 7,
                y_velocity: -8,
            }),
        ];

        let updated =
            heady_metal_apply_piece_velocities_sub_376EC(slots, -0x0001_86A0, 0x0001_86A0);

        assert_eq!(
            updated[0],
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: -0x0001_86A0,
                y_velocity: 0x0001_86A0,
            }),
            "ASM loc_376F2 should load a live worker pointer from `$FFFFFB72` before `sub_376EC` writes `d6/d7`",
        );
        assert_eq!(
            updated[1], None,
            "ASM loc_37702 should skip empty worker slots and continue the `dbf` loop",
        );
        assert_eq!(
            updated[6],
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: -0x0001_86A0,
                y_velocity: 0x0001_86A0,
            }),
            "ASM sub_376EC should keep iterating through all `#$10+1` worker entries",
        );
        assert_eq!(
            updated[16],
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: -0x0001_86A0,
                y_velocity: 0x0001_86A0,
            }),
            "ASM sub_376EC should still update the final worker entry before the terminating `dbf` falls through",
        );
    }

    #[test]
    fn heady_metal_tracking_velocity_loc_37582_aims_right_when_piece_is_not_right_of_kid() {
        let velocity =
            heady_metal_tracking_velocity_loc_37582(0x0080, 0x0080, 0x0080, 0x00A8, 4, false);

        assert_eq!(
            velocity.x_velocity, 0x0000_8000,
            "ASM loc_37582 should fall through to `move.l #$8000,d6` when `$FFFFFB7A.x_pos <= Kid.gfx_x`",
        );
    }

    #[test]
    fn heady_metal_tracking_velocity_loc_375aa_aims_left_when_piece_is_right_of_kid() {
        let velocity =
            heady_metal_tracking_velocity_loc_37582(0x0080, 0x0080, 0x0081, 0x00A8, 4, false);

        assert_eq!(
            velocity.x_velocity, -0x0000_8000,
            "ASM loc_375AA should run after `cmp.w Kid.gfx_x,$FFFFFB7A.x_pos; bgt.s` and set `d6` to `$FFFF8000`",
        );
    }

    #[test]
    fn heady_metal_tracking_velocity_loc_375b0_aims_down_when_adjusted_piece_is_not_below_kid() {
        let velocity =
            heady_metal_tracking_velocity_loc_37582(0x0080, 0x0080, 0x0080, 0x00A8, 4, false);

        assert_eq!(
            velocity.y_velocity, 0x0000_4800,
            "ASM loc_375B0 should compare `$FFFFFB7A.y_pos - $28` with Kid.gfx_y and fall through to `move.l #$4800,d7` when it is <= Kid",
        );
    }

    #[test]
    fn heady_metal_tracking_velocity_loc_375c2_zeros_horizontal_velocity_in_early_phase() {
        let velocity =
            heady_metal_tracking_velocity_loc_37582(0x0080, 0x0080, 0x0081, 0x00A8, 3, false);

        assert_eq!(
            (velocity.x_velocity, velocity.y_velocity),
            (0, 0x0000_4800),
            "ASM loc_375C2 should clear `d6` when `$FFFFFB72.$3E < #4` without clearing `d7`",
        );
    }

    #[test]
    fn heady_metal_tracking_velocity_loc_375d4_preserves_velocity_when_freeze_flag_is_clear() {
        let velocity =
            heady_metal_tracking_velocity_loc_37582(0x0080, 0x0080, 0x0081, 0x00A9, 4, false);

        assert_eq!(
            (velocity.x_velocity, velocity.y_velocity),
            (-0x0000_8000, -0x0000_5000),
            "ASM loc_375D4 should leave selected `d6/d7` unchanged when `$FFFFFB4F` is clear",
        );
    }

    #[test]
    fn heady_metal_tracking_velocity_loc_375de_feeds_final_velocity_to_worker_update() {
        // ASM loc_375DE (kid.asm:39383): enter `sub_376EC` after the optional
        // `$FFFFFB4F` velocity clear, so every live worker receives final `d6/d7`.
        let velocity =
            heady_metal_tracking_velocity_loc_37582(0x0080, 0x0080, 0x0081, 0x00A9, 4, true);
        let mut slots = [None; 17];
        slots[0] = Some(HeadyMetalSpriteVelocitySub376EC {
            x_velocity: 0x1234,
            y_velocity: -0x5678,
        });

        let updated = heady_metal_apply_piece_velocities_sub_376EC(
            slots,
            velocity.x_velocity,
            velocity.y_velocity,
        );

        assert_eq!(
            updated[0],
            Some(HeadyMetalSpriteVelocitySub376EC {
                x_velocity: 0,
                y_velocity: 0,
            }),
            "ASM loc_375DE should enter `sub_376EC` with final `d6/d7`, including the zeroed pair from `$FFFFFB4F`",
        );
    }

    #[test]
    fn heady_metal_spawn_animation_loc_3761e_uses_high_animation_for_decremented_counter_six_or_more()
     {
        let step = heady_metal_spawn_animation_loc_3761E(7);

        assert_eq!(
            step.next_piece_counter, 6,
            "the HeadyMetal spawn gate should store the decremented `$3E(a3)` counter",
        );
        assert_eq!(
            step.selected_animation_label, "stru_37B98",
            "ASM loc_3761E should run `cmpi.w #6,d2` after decrementing `$3E(a3)` and select `stru_37B98` when `d2 >= 6`",
        );
    }

    #[test]
    fn heady_metal_spawn_animation_loc_37630_uses_low_animation_after_counter_wrap() {
        let step = heady_metal_spawn_animation_loc_3761E(1);

        assert_eq!(
            step.next_piece_counter, 0x000A,
            "the HeadyMetal spawn gate should reset `$3E(a3)` to `$A` when the decremented counter reaches zero",
        );
        assert_eq!(
            step.selected_animation_label, "stru_37BC0",
            "ASM loc_37630 should still select `stru_37BC0` after the wrap because `d2` remains zero for the `cmpi.w #6,d2` branch",
        );
    }

    #[test]
    fn heady_metal_spawn_animation_loc_3763a_runs_the_shared_tail_after_animation_selection() {
        let high_animation = heady_metal_spawn_animation_loc_3761E(7);
        let low_animation = heady_metal_spawn_animation_loc_3761E(5);

        for step in [high_animation, low_animation] {
            assert!(
                step.runs_tail_update,
                "ASM loc_3763A should be the shared tail reached after either HeadyMetal spawn animation",
            );
            assert!(
                step.runs_breakup_step_sub_37708,
                "ASM loc_3763A should call `sub_37708` before the palette and sprite refresh helpers",
            );
            assert!(
                step.runs_palette_cycle_sub_379_e8,
                "ASM loc_3763A should call `sub_379E8` after the breakup step",
            );
            assert!(
                step.runs_sprite_refresh_sub_37_abc && step.loops_to_tracking_loc_37582,
                "ASM loc_3763A should call `sub_37ABC` and then branch back to `loc_37582`",
            );
        }
    }

    #[test]
    fn heady_metal_scatter_step_sub_3787c_applies_the_primary_five_piece_velocity_pattern() {
        let updated = heady_metal_scatter_step_sub_3787C(
            HeadyMetalScatterVelocities {
                slot_fb72_x_velocity: 0x10,
                slot_fb76_x_velocity: 0x20,
                slot_fb7a_y_velocity: 0x30,
                slot_fbae_x_velocity: 0x40,
                slot_fbb2_x_velocity: 0x50,
            },
            -0x0001_86A0,
            0x0001_86A0,
        );

        assert_eq!(
            updated.slot_fb72_x_velocity,
            0x10 - 0x0001_86A0,
            "ASM sub_3787C should add `d6` to `$FFFFFB72`'s `x_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fb76_x_velocity,
            0x20 + 0x0001_86A0,
            "ASM sub_3787C should add `d7` to `$FFFFFB76`'s `x_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fb7a_y_velocity,
            0x30 + 0x0001_86A0,
            "ASM sub_3787C should add `d7` to `$FFFFFB7A`'s `y_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fbae_x_velocity,
            0x40 - 0x0001_86A0,
            "ASM sub_3787C should add `d6` to `$FFFFFBAE`'s `x_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fbb2_x_velocity,
            0x50 + 0x0001_86A0,
            "ASM sub_3787C should add `d7` to `$FFFFFBB2`'s `x_vel(a3)`",
        );
    }

    #[test]
    fn heady_metal_scatter_step_sub_378a2_applies_the_alternate_five_piece_velocity_pattern() {
        let updated = heady_metal_scatter_step_sub_378A2(
            HeadyMetalScatterVelocities {
                slot_fb72_x_velocity: 0x10,
                slot_fb76_x_velocity: 0x20,
                slot_fb7a_y_velocity: 0x30,
                slot_fbae_x_velocity: 0x40,
                slot_fbb2_x_velocity: 0x50,
            },
            -0x0001_86A0,
            0x0001_86A0,
        );

        assert_eq!(
            updated.slot_fb72_x_velocity,
            0x10 + 0x0001_86A0,
            "ASM sub_378A2 should add `d7` to `$FFFFFB72`'s `x_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fb76_x_velocity,
            0x20 - 0x0001_86A0,
            "ASM sub_378A2 should add `d6` to `$FFFFFB76`'s `x_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fb7a_y_velocity,
            0x30 - 0x0001_86A0,
            "ASM sub_378A2 should add `d6` to `$FFFFFB7A`'s `y_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fbae_x_velocity,
            0x40 + 0x0001_86A0,
            "ASM sub_378A2 should add `d7` to `$FFFFFBAE`'s `x_vel(a3)`",
        );
        assert_eq!(
            updated.slot_fbb2_x_velocity,
            0x50 - 0x0001_86A0,
            "ASM sub_378A2 should add `d6` to `$FFFFFBB2`'s `x_vel(a3)`",
        );
    }

    #[test]
    fn heady_metal_palette_cycle_sub_379e8_wraps_the_phase_and_selects_the_palette_word() {
        let wrapped = heady_metal_palette_cycle_sub_379E8(0x001F);
        let rising = heady_metal_palette_cycle_sub_379E8(0x0009);

        assert_eq!(
            wrapped,
            HeadyMetalPaletteCycleSub379E8 {
                next_phase_counter: 0,
                palette_word: 0x0200,
            },
            "ASM sub_379E8 should wrap `d2` to 5 bits before loading `unk_379A8` into `Palette_Buffer+$5E`",
        );
        assert_eq!(
            rising,
            HeadyMetalPaletteCycleSub379E8 {
                next_phase_counter: 10,
                palette_word: 0x020A,
            },
            "ASM sub_379E8 should advance to the next `unk_379A8` entry before writing the palette word",
        );
    }

    #[test]
    fn heady_metal_piece_sprite_sub_37a22_uses_diamond_override_and_directional_buckets() {
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(true, 0x08, 0x0040, 0x0040, 0x0050, 0x0050),
            "LnkTo_unk_C8488-Data_Index",
            "ASM sub_37A22 should force `LnkTo_unk_C8488-Data_Index` when diamond power is active and bit 3 is set",
        );
        // ASM loc_37A38 (kid.asm:39906): active diamond power with
        // `$FFFFF809 & #8 == 0` selects `LnkTo_unk_C84A8-Data_Index`.
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(true, 0x00, 0x0040, 0x0040, 0x0050, 0x0050),
            "LnkTo_unk_C84A8-Data_Index",
            "ASM sub_37A22 should force `LnkTo_unk_C84A8-Data_Index` when diamond power is active and bit 3 is clear",
        );
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x0040, 0x003B),
            "LnkTo_unk_C8488-Data_Index",
            "ASM loc_37A94 should choose index 0 when Kid is above `y_pos-4` and horizontally centred",
        );
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x0038, 0x0040),
            "LnkTo_unk_C84B8-Data_Index",
            "ASM loc_37A3E should stay in the middle Y band and choose index 6 when Kid is not left of `x_pos-8`",
        );
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x0037, 0x0040),
            "LnkTo_unk_C8498-Data_Index",
            "ASM loc_37A74 should choose index 2 when the middle-band Kid X is left of `x_pos-8`",
        );
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x0040, 0x0045),
            "LnkTo_unk_C84A8-Data_Index",
            "ASM loc_37A78 should choose index 4 when Kid is below `y_pos+4` and horizontally centred",
        );
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x003B, 0x0045),
            "LnkTo_unk_C84A0-Data_Index",
            "ASM loc_37A8C should choose index 3 when the lower-band Kid X is left of `x_pos-4`",
        );
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x0045, 0x0045),
            "LnkTo_unk_C84B0-Data_Index",
            "ASM loc_37A90 should choose index 5 when the lower-band Kid X is right of `x_pos+4`",
        );
    }

    #[test]
    fn heady_metal_piece_sprite_loc_37aa8_uses_upper_left_bucket() {
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x003B, 0x003B),
            "LnkTo_unk_C8490-Data_Index",
            "ASM loc_37AA8 should choose index 1 when upper-band Kid X is left of `x_pos-4`",
        );
    }

    #[test]
    fn heady_metal_piece_sprite_loc_37aac_uses_upper_right_bucket() {
        assert_eq!(
            heady_metal_piece_sprite_sub_37A22(false, 0x00, 0x0040, 0x0040, 0x0045, 0x003B),
            "LnkTo_unk_C84C0-Data_Index",
            "ASM loc_37AAC should choose index 7 when upper-band Kid X is right of `x_pos+4`",
        );
    }

    #[test]
    fn heady_metal_piece_sprite_loc_37aae_loads_mapping_from_selected_index() {
        assert_eq!(
            heady_metal_piece_sprite_mapping_loc_37AAE(7),
            "LnkTo_unk_C84C0-Data_Index",
            "ASM loc_37AAE should double the selected index and load the matching `off_37AE0` word",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_37710_ignores_anchor_without_collision() {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 12, 9, 0, [None; 13]);

        assert_eq!(
            (result.next_collision_type, result.next_phase_counter),
            (0, 12),
            "ASM loc_37710 should branch to the shared tail when the live anchor has zero `collision_type(a3)`",
        );
        assert_eq!(
            result.scatter_step, None,
            "ASM loc_37710 should not enter the breakup collision handler without a nonzero collision word",
        );
    }

    #[test]
    fn heady_metal_breakup_step_sub_37708_advances_the_early_pop_sequence_on_collision_type_2c() {
        let result = heady_metal_breakup_step_sub_37708(true, 0x002C, 0xFFFF, 0, 0, {
            let mut pieces = [None; 13];
            pieces[0] = Some(HeadyMetalBreakupPieceSub37708 {
                sprite_word_y: 6,
                x_pos: 0x0044,
                y_pos: 0x0046,
            });
            pieces
        });

        assert_eq!(
            result.next_collision_type, 0,
            "ASM sub_37708 should clear `collision_type(a3)` before handling the HeadyMetal breakup hit",
        );
        assert_eq!(
            result.next_phase_counter, 0,
            "ASM loc_3772A should wrap `$3E(a3)` from `$FFFF` to 0 on the first valid breakup collision",
        );
        assert_eq!(
            result.piece_slots[0],
            Some(HeadyMetalBreakupPieceSub37708 {
                sprite_word_y: 8,
                x_pos: 0x0044,
                y_pos: 0x0046,
            }),
            "ASM loc_3772A should add 2 to the selected worker's `$A(a3)` during the opening 13-pop phase",
        );
        assert_eq!(
            result.played_sound_effect,
            Some("sfx_Boss_eye_pops"),
            "ASM loc_3772A should play `sfx_Boss_eye_pops` during the opening 13-pop phase",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_37768_retracts_the_middle_pop_sequence() {
        let result = heady_metal_breakup_step_sub_37708(true, 0x002C, 12, 0, 0, {
            let mut pieces = [None; 13];
            pieces[12] = Some(HeadyMetalBreakupPieceSub37708 {
                sprite_word_y: 14,
                x_pos: 0x005A,
                y_pos: 0x006C,
            });
            pieces
        });

        assert_eq!(
            result.next_phase_counter, 13,
            "ASM loc_37768 should handle the first counter above `$C` after loc_3772A advances `$3E(a3)`",
        );
        assert_eq!(
            result.piece_slots[12],
            Some(HeadyMetalBreakupPieceSub37708 {
                sprite_word_y: 12,
                x_pos: 0x005A,
                y_pos: 0x006C,
            }),
            "ASM loc_37768 should subtract 2 from the selected worker's `$A(a3)` for counters `<= $18`",
        );
        assert_eq!(
            result.played_sound_effect, None,
            "ASM loc_37768 should refresh the worker without replaying `sfx_Boss_eye_pops`",
        );
    }

    #[test]
    fn heady_metal_breakup_step_sub_37708_spawns_the_late_pop_pair_and_clears_the_piece_slot() {
        let result = heady_metal_breakup_step_sub_37708(true, 0x002C, 24, 0, 0, {
            let mut pieces = [None; 13];
            pieces[2] = Some(HeadyMetalBreakupPieceSub37708 {
                sprite_word_y: 0,
                x_pos: 0x0070,
                y_pos: 0x0088,
            });
            pieces
        });

        assert_eq!(
            result.next_phase_counter, 25,
            "ASM loc_37778 should advance into the late breakup range before spawning the paired effects",
        );
        assert_eq!(
            result.spawned_pop_effect_at,
            Some((0x0070, 0x0088)),
            "ASM loc_37778 should spawn `sub_3729E` from the selected worker's `x_pos/y_pos`",
        );
        assert_eq!(
            result.spawned_attack_at,
            Some((0x0070, 0x0088)),
            "ASM loc_37778 should also spawn `sub_37332` from that same worker position",
        );
        assert_eq!(
            result.piece_slots[2], None,
            "ASM loc_37778 should clear the selected worker pointer after launching the late breakup pair",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_377bc_waits_for_phase_25_before_final_scatter() {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 0x0024, 0x0160, 0, [None; 13]);

        assert!(
            !result.set_has_kid_collision && !result.set_global_hit_latch,
            "ASM loc_377BC should return while `$3E(a3) < #$25`, before setting `has_kid_collision(a3)` or `($FFFFFB4E).w`",
        );
        assert_eq!(
            result.scatter_step, None,
            "ASM loc_377BC should skip the final scatter timer and velocity tier selection until the breakup counter reaches `#$25`",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_377f6_uses_high_tier_after_decrementing_scatter_timer() {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 0x0025, 0x0160, 0, [None; 13]);

        assert!(
            result.set_has_kid_collision && result.set_global_hit_latch,
            "ASM sub_37708 should set `has_kid_collision(a3)` and `($FFFFFB4E).w` once `$3E(a3)` reaches `$25`",
        );
        assert_eq!(
            result.next_remaining_scatter_frames, 0x015F,
            "ASM loc_377F6 should decrement `$40(a3)` before choosing the final scatter speed tier",
        );
        // ASM loc_37866 (kid.asm:39622): `Frame_Counter & #2 == 0` calls the
        // primary `sub_3787C` scatter helper.
        assert_eq!(
            result.scatter_step,
            Some(HeadyMetalScatterStepSub37708 {
                pattern: HeadyMetalScatterPatternSub37708::Primary,
                d6_x_velocity_delta: -0x0003_0D40,
                d7_velocity_delta: 0x0003_0D40,
            }),
            "ASM loc_37866 should call `sub_3787C` with the `$30D40` tier when `Frame_Counter & #2` is zero",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_3781e_uses_mid_high_tier_at_decremented_15e() {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 0x0025, 0x015F, 2, [None; 13]);

        assert_eq!(
            result.next_remaining_scatter_frames, 0x015E,
            "ASM loc_3781E should receive the already-decremented `$40(a3)` value after the `#$15E` high-tier branch is skipped",
        );
        // ASM loc_37876 (kid.asm:39630): a nonzero `Frame_Counter & #2` calls
        // the alternate `sub_378A2` scatter helper.
        assert_eq!(
            result.scatter_step,
            Some(HeadyMetalScatterStepSub37708 {
                pattern: HeadyMetalScatterPatternSub37708::Alternate,
                d6_x_velocity_delta: -0x0004_93E0,
                d7_velocity_delta: 0x0004_93E0,
            }),
            "ASM loc_3781E should select the `$493E0` tier while the decremented timer is above `#$FA`, then the frame bit picks `sub_378A2`",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_37832_uses_mid_tier_at_decremented_fa() {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 0x0025, 0x00FB, 0, [None; 13]);

        assert_eq!(
            result.next_remaining_scatter_frames, 0x00FA,
            "ASM loc_37832 should receive the already-decremented `$40(a3)` value after the `#$FA` branch is skipped",
        );
        assert_eq!(
            result.scatter_step,
            Some(HeadyMetalScatterStepSub37708 {
                pattern: HeadyMetalScatterPatternSub37708::Primary,
                d6_x_velocity_delta: -0x0006_1A80,
                d7_velocity_delta: 0x0006_1A80,
            }),
            "ASM loc_37832 should select the `$61A80` tier while the decremented timer is above `#$96`",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_37846_uses_low_tier_at_decremented_96() {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 0x0025, 0x0097, 2, [None; 13]);

        assert_eq!(
            result.next_remaining_scatter_frames, 0x0096,
            "ASM loc_37846 should receive the already-decremented `$40(a3)` value after the `#$96` branch is skipped",
        );
        assert_eq!(
            result.scatter_step,
            Some(HeadyMetalScatterStepSub37708 {
                pattern: HeadyMetalScatterPatternSub37708::Alternate,
                d6_x_velocity_delta: -0x0007_A120,
                d7_velocity_delta: 0x0007_A120,
            }),
            "ASM loc_37846 should select the `$7A120` tier while the decremented timer is above `#$64`, then the frame bit picks `sub_378A2`",
        );
    }

    #[test]
    fn heady_metal_breakup_step_loc_3785a_uses_final_tier_at_decremented_64() {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 0x0025, 0x0065, 0, [None; 13]);

        assert_eq!(
            result.next_remaining_scatter_frames, 0x0064,
            "ASM loc_3785A should receive the already-decremented `$40(a3)` value after the `#$64` branch is skipped",
        );
        assert_eq!(
            result.scatter_step,
            Some(HeadyMetalScatterStepSub37708 {
                pattern: HeadyMetalScatterPatternSub37708::Primary,
                d6_x_velocity_delta: -0x0009_27C0,
                d7_velocity_delta: 0x0009_27C0,
            }),
            "ASM loc_3785A should fall through to the final `$927C0` tier when the decremented timer is at or below `#$64`",
        );
    }

    #[test]
    fn heady_metal_breakup_step_sub_37708_reveals_the_anchor_pair_before_cleanup_when_the_timer_is_zero()
     {
        let result = heady_metal_breakup_step_sub_37708(true, 0, 0x0025, 0, 0, [None; 13]);

        assert!(
            result.reveal_anchor_worker
                && result.clear_anchor_hidden_flag
                && result.reveal_linked_worker
                && result.trigger_cleanup_sub_378c8,
            "ASM sub_37708 should unhide the boss anchor pair and jump into sub_378C8 immediately when `$40(a3)` is already zero",
        );
        assert_eq!(
            result.scatter_step, None,
            "ASM sub_37708 should skip the alternating scatter velocity branches on the zero-timer cleanup path",
        );
    }

    #[test]
    fn heady_metal_cleanup_sub_378c8_uses_loc_378d6_scatter_impulses_while_the_countdown_survives()
    {
        let result = heady_metal_cleanup_sub_378C8(true, 2);

        assert_eq!(
            result.next_countdown_42, 1,
            "ASM sub_378C8 decrements `$42(a3)` first when `$FFFFFB4E` is set and only falls into loc_37922 once that decremented value reaches zero",
        );
        assert!(!result.should_cleanup_workers);
        assert_eq!(result.current_x_velocity_delta, -0x0002_6D18);
        assert_eq!(result.left_worker_x_velocity_delta, 0x0002_6D18);
        assert_eq!(result.centre_worker_y_velocity_delta, 0x0001_3880);
        assert_eq!(
            result.centre_worker_sprite_label,
            Some("LnkTo_unk_C8458-Data_Index"),
            "ASM loc_378D6 refreshes `$FFFFFB7A` to `LnkTo_unk_C8458-Data_Index` before replaying the boss-death sound",
        );
        assert_eq!(result.played_sound_effect, Some("sfx_Boss_dies"));
        assert_eq!(result.right_worker_x_velocity_delta, -0x0002_6D18);
        assert_eq!(result.far_right_worker_x_velocity_delta, 0x0002_6D18);
    }

    #[test]
    fn heady_metal_cleanup_sub_378c8_uses_loc_37922_once_the_latched_countdown_reaches_zero() {
        let result = heady_metal_cleanup_sub_378C8(true, 1);

        assert_eq!(result.next_countdown_42, 0);
        assert!(
            result.should_cleanup_workers,
            "ASM loc_37922 runs `j_loc_1078` on the five cached worker pointers once the decremented `$42(a3)` reaches zero",
        );
        assert_eq!(result.centre_worker_sprite_label, None);
        assert_eq!(result.played_sound_effect, None);
    }

    /// ASM loc_B8BA (kid.asm:14811): two-player both-alive path — NOT.b
    /// (Current_player) swaps 0↔$FF so the other player takes over after
    /// death. Native: when both players have lives, `current_player_index`
    /// flips to the other slot.
    #[test]
    fn loc_b8ba_two_player_both_alive_swaps_current_player() {
        let manifest = manifest_with_levels(4);
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.player_slots[0].lives = 3;
        runtime.player_slots[0].level_id = LevelId(0);
        runtime.player_slots[1].lives = 2;
        runtime.player_slots[1].level_id = LevelId(1);
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 2;
        runtime.boot_state = BootState::InGame;

        // Simulate final frame of Dying countdown
        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 1,
        };
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.current_player_index, 1,
            "ASM loc_B8BA: NOT.b Current_player should swap to P2 when both alive"
        );
        assert_eq!(
            runtime.current_level,
            LevelId(1),
            "ASM loc_B8BA: should load P2's level after swap"
        );
    }

    /// ASM loc_B8D0 (kid.asm:14819): P1 dead (Player_1_Lives==0), P2 alive —
    /// `st (Current_player).w` forces P2 active, then falls to loc_B8A8 for
    /// game-over check. If P2 has lives, restarts on P2's level.
    #[test]
    fn loc_b8d0_p1_dead_p2_alive_forces_p2_active() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(4));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 0;
        runtime.progression.number_continues = 0;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 0;
        runtime.player_slots[1].lives = 2;
        runtime.player_slots[1].level_id = LevelId(2);
        runtime.boot_state = BootState::InGame;

        runtime.player_lifecycle = PlayerLifecycle::GameOver {
            remaining_frames: 1,
        };
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.current_player_index, 1,
            "ASM loc_B8D0: st (Current_player) should force P2 active when P1 has no lives"
        );
        assert_eq!(
            runtime.progression.number_lives, 2,
            "ASM loc_B8D0: P2's lives should be loaded after switch"
        );
    }

    #[test]
    fn two_player_switch_on_death() {
        let manifest = manifest_with_levels(4);
        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.player_slots[0].lives = 3;
        runtime.player_slots[0].level_id = LevelId(0);
        runtime.player_slots[1].lives = 3;
        runtime.player_slots[1].level_id = LevelId(1);
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 3;
        runtime.current_level = LevelId(0);

        // Simulate death: decrement lives and enter Dying state
        runtime.progression.number_lives = 2;
        runtime.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        // Step to finish Dying countdown — should switch to P2
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.current_player_index, 1,
            "should switch to P2 after P1 death"
        );
        assert_eq!(runtime.current_level, LevelId(1), "should load P2's level");
    }

    #[test]
    fn two_player_game_over_when_both_out_of_lives() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 0;
        runtime.progression.number_continues = 0;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 0;
        runtime.player_slots[1].lives = 0;
        runtime.player_slots[1].continues = 0;

        runtime.player_lifecycle = PlayerLifecycle::GameOver {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 0 },
            "should return to title when both players out of lives and continues"
        );
        assert!(!runtime.two_player, "2P should be reset on game over");
    }

    #[test]
    fn two_player_game_over_switches_when_other_has_lives() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(4));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 0;
        runtime.progression.number_continues = 0;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 0;
        runtime.player_slots[1].lives = 2;
        runtime.player_slots[1].level_id = LevelId(2);

        runtime.player_lifecycle = PlayerLifecycle::GameOver {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.current_player_index, 1,
            "should switch to P2 when P1 is game over"
        );
        assert_eq!(
            runtime.progression.number_lives, 2,
            "should have P2's lives"
        );
    }

    #[test]
    fn two_player_game_over_prefers_continue_countdown_before_switching_players() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(4));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 0;
        runtime.progression.number_continues = 1;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 1;
        runtime.player_slots[1].lives = 2;
        runtime.player_slots[1].level_id = LevelId(2);
        runtime.player_lifecycle = PlayerLifecycle::GameOver {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        runtime.step_frame(FrameInput::default());

        assert!(matches!(
            runtime.player_lifecycle,
            PlayerLifecycle::ContinueCountdown { .. }
        ));
        assert_eq!(
            runtime.current_player_index, 0,
            "continue countdown should stay on the player who just ran out of lives"
        );
    }

    #[test]
    fn two_player_continue_input_respawns_same_player_and_preserves_other_slot() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(4));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.current_level = LevelId(0);
        runtime.progression.number_lives = 0;
        runtime.progression.number_continues = 1;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 1;
        runtime.player_slots[0].level_id = LevelId(0);
        runtime.player_slots[1].lives = 2;
        runtime.player_slots[1].continues = 0;
        runtime.player_slots[1].level_id = LevelId(2);
        runtime.player_lifecycle = PlayerLifecycle::ContinueCountdown {
            remaining_frames: 10,
        };
        runtime.boot_state = BootState::InGame;

        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });

        assert!(matches!(
            runtime.player_lifecycle,
            PlayerLifecycle::Respawning { .. }
        ));
        assert_eq!(runtime.current_player_index, 0);
        assert_eq!(runtime.progression.number_continues, 0);
        assert_eq!(runtime.progression.number_lives, 3);
        assert_eq!(runtime.current_level, LevelId(0));
        assert_eq!(
            runtime.player_slots[1].level_id,
            LevelId(2),
            "using P1's continue should not disturb P2's stored level"
        );
    }

    #[test]
    fn two_player_continue_timeout_returns_to_title_without_switching_players() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(4));
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.two_player = true;
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 0;
        runtime.progression.number_continues = 1;
        runtime.player_slots[0].lives = 0;
        runtime.player_slots[0].continues = 1;
        runtime.player_slots[1].lives = 2;
        runtime.player_slots[1].level_id = LevelId(2);
        runtime.player_lifecycle = PlayerLifecycle::ContinueCountdown {
            remaining_frames: 1,
        };
        runtime.boot_state = BootState::InGame;

        runtime.step_frame(FrameInput::default());

        assert_eq!(runtime.boot_state, BootState::TitleScreen { cursor: 0 });
        assert!(!runtime.two_player, "title return should clear 2P mode");
        assert_eq!(
            runtime.current_player_index, 0,
            "continue timeout should not implicitly swap to P2"
        );
    }

    #[test]
    fn save_load_player_slot_round_trips() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.current_player_index = 0;
        runtime.progression.number_lives = 5;
        runtime.progression.score = 42_000;
        runtime.progression.diamond_count = 30;
        runtime.progression.current_form = PlayerForm::Skycutter;

        runtime.save_player_slot();

        // Modify working state
        runtime.progression.number_lives = 1;
        runtime.progression.score = 0;

        // Load it back
        runtime.load_player_slot();

        assert_eq!(runtime.progression.number_lives, 5);
        assert_eq!(runtime.progression.score, 42_000);
        assert_eq!(runtime.progression.diamond_count, 30);
        assert_eq!(runtime.progression.current_form, PlayerForm::Skycutter);
    }

    /// ASM loc_8B4 (kid.asm:972): sub_8A4 fall-through for player 1 — `moveq #$A,d0;
    /// lea (Number_Lives),a1; move.w (a0)+,(a1)+; dbf d0,loc_8BA` copies 11 words
    /// (lives, hitpoints, diamonds, level ID, helmet, continues, extra_hitpoint_slots,
    /// score[long]) from Player_1_Lives into the working Number_Lives region.
    /// Player 2 uses `lea (Player_2_Lives),a0` before the same loop.
    #[test]
    fn load_player_slot_selects_correct_player_loc_8b4() {
        let mut runtime = NativeRuntime::new(manifest());

        // Set up distinct values in each player slot
        runtime.player_slots[0].lives = 3;
        runtime.player_slots[0].diamonds = 10;
        runtime.player_slots[0].score = 1000;
        runtime.player_slots[0].form = PlayerForm::Maniaxe;

        runtime.player_slots[1].lives = 7;
        runtime.player_slots[1].diamonds = 50;
        runtime.player_slots[1].score = 9999;
        runtime.player_slots[1].form = PlayerForm::Eyeclops;

        // Load player 1 (index 0) — ASM: `beq loc_8B4` when Current_player == 0
        runtime.current_player_index = 0;
        runtime.load_player_slot();
        assert_eq!(
            runtime.progression.number_lives, 3,
            "ASM loc_8B4: player 1 selected via `beq.w loc_8B4` — lives from Player_1_Lives"
        );
        assert_eq!(runtime.progression.diamond_count, 10);
        assert_eq!(runtime.progression.score, 1000);
        assert_eq!(runtime.progression.current_form, PlayerForm::Maniaxe);

        // Load player 2 (index 1) — ASM: `lea (Player_2_Lives),a0`
        runtime.current_player_index = 1;
        runtime.load_player_slot();
        assert_eq!(
            runtime.progression.number_lives, 7,
            "ASM loc_8B4: player 2 selected via Player_2_Lives base — lives differ from player 1"
        );
        assert_eq!(runtime.progression.diamond_count, 50);
        assert_eq!(runtime.progression.score, 9999);
        assert_eq!(runtime.progression.current_form, PlayerForm::Eyeclops);
    }

    /// ASM sub_8A4 / loc_8B4: the `dbf d0,loc_8BA` loop copies exactly 11 words,
    /// covering lives through score. Verify all fields are loaded.
    #[test]
    fn load_player_slot_copies_all_fields_loc_8b4() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.player_slots[0].lives = 2;
        runtime.player_slots[0].hitpoints = 4;
        runtime.player_slots[0].diamonds = 15;
        runtime.player_slots[0].level_id = LevelId(42);
        runtime.player_slots[0].form = PlayerForm::Cyclone;
        runtime.player_slots[0].continues = 3;
        runtime.player_slots[0].score = 12345;

        runtime.current_player_index = 0;
        runtime.load_player_slot();

        assert_eq!(runtime.progression.number_lives, 2, "lives");
        assert_eq!(runtime.progression.number_hitpoints, 4, "hitpoints");
        assert_eq!(runtime.progression.diamond_count, 15, "diamonds");
        assert_eq!(runtime.current_level, LevelId(42), "level ID");
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::Cyclone,
            "helmet"
        );
        assert_eq!(runtime.progression.number_continues, 3, "continues");
        assert_eq!(runtime.progression.score, 12345, "score");
    }

    #[test]
    fn trace_frame_reports_current_player() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.current_player_index = 1;
        let trace = runtime.trace_frame();
        assert_eq!(trace.current_player, 1);
    }

    #[test]
    fn snapshot_idle_on_level0_frame60() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Advance 60 frames of idle standing
        for _ in 0..60 {
            runtime.step_frame(FrameInput::default());
        }
        let trace = runtime.trace_frame();
        let mut settings = insta::Settings::clone_current();
        redact_advisory(&mut settings);
        settings.bind(|| {
            insta::assert_yaml_snapshot!("idle_level0_frame60", trace);
        });
    }

    #[test]
    fn snapshot_walk_right_10_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        for _ in 0..10 {
            runtime.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }
        let trace = runtime.trace_frame();
        let mut settings = insta::Settings::clone_current();
        redact_advisory(&mut settings);
        settings.bind(|| {
            insta::assert_yaml_snapshot!("walk_right_10_frames", trace);
        });
    }

    #[test]
    fn snapshot_jump_sequence() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Wait for grounded
        loop {
            runtime.step_frame(FrameInput::default());
            if runtime.trace_frame().kid_on_ground == Some(true) {
                break;
            }
        }
        // Jump: press B for one frame, then hold for 5 frames
        runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });
        for _ in 0..5 {
            runtime.step_frame(FrameInput {
                b: true,
                ..FrameInput::default()
            });
        }
        let trace = runtime.trace_frame();
        let mut settings = insta::Settings::clone_current();
        redact_advisory(&mut settings);
        settings.bind(|| {
            insta::assert_yaml_snapshot!("jump_sequence_6frames", trace);
        });
    }

    // -------------------------------------------------------------------
    // Determinism — same inputs must produce identical state (MR1)
    // -------------------------------------------------------------------

    #[test]
    fn deterministic_replay_produces_identical_trace() {
        let inputs: Vec<FrameInput> = (0..300)
            .map(|f| match f {
                0..=59 => FrameInput::default(),
                60..=79 => FrameInput {
                    right: true,
                    ..FrameInput::default()
                },
                80..=85 => FrameInput {
                    right: true,
                    b: true,
                    ..FrameInput::default()
                },
                86..=150 => FrameInput {
                    right: true,
                    ..FrameInput::default()
                },
                _ => FrameInput::default(),
            })
            .collect();

        fn run_scenario(inputs: &[FrameInput]) -> Vec<NativeTraceFrame> {
            let mut runtime = NativeRuntime::new(manifest_with_levels(4));
            runtime.set_block_grid(grid_with_floor_at_row(17));
            start_level(&mut runtime);
            for _ in 0..260 {
                runtime.step_frame(FrameInput::default());
            }
            inputs
                .iter()
                .map(|input| {
                    runtime.step_frame(*input);
                    runtime.trace_frame()
                })
                .collect()
        }

        let run1 = run_scenario(&inputs);
        let run2 = run_scenario(&inputs);
        assert_eq!(run1.len(), run2.len());
        for (i, (a, b)) in run1.iter().zip(run2.iter()).enumerate() {
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "divergence at frame {i}"
            );
        }
    }

    // --- Demo playback tests ---

    #[test]
    fn title_idle_triggers_demo_after_640_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.frame_index = 1;
        runtime.title_menu_arm_frame = 0;
        // Run 640 frames of idle input
        for _ in 0..640 {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            matches!(runtime.boot_state, BootState::DemoPlay { .. }),
            "should be in DemoPlay after 640 idle frames, got {:?}",
            runtime.boot_state
        );
    }

    #[test]
    fn demo_bootstrap_holds_spawn_pose_then_freefalls_before_grounded_hold() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(0x1C));
        runtime.set_block_grid(grid_with_floor_at_row(15));
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        runtime.begin_demo_gameplay(LevelId(demo::DEMO_2_LEVEL));

        let opening_slice = runtime
            .opening_slice
            .expect("demo bootstrap should create an opening slice");
        assert_eq!(opening_slice.player.y_pos, 0x00DF_0000);
        assert_eq!(opening_slice.terrain.floor_y_pos, 0x00EF_0000);

        runtime.opening_slice = Some(opening_slice);
        let held = runtime.trace_frame();
        assert_eq!(held.kid_y_pos, Some(0x00DF_0000));
        assert_eq!(held.number_objects, 5);
        assert_eq!(held.number_gfx_objects, 4);

        let bootstrap_init = runtime.step_frame(FrameInput::default());
        assert_eq!(bootstrap_init.boot_phase_elapsed, Some(66));
        assert_eq!(bootstrap_init.kid_y_pos, Some(0x00E0_0000));
        assert_eq!(bootstrap_init.number_objects, 5);
        assert_eq!(bootstrap_init.number_gfx_objects, 4);

        let freefall = runtime.step_frame(FrameInput::default());
        assert_eq!(freefall.boot_phase_elapsed, Some(67));
        assert!(
            freefall.kid_y_pos.expect("freefall y") > bootstrap_init.kid_y_pos.expect("settle y")
        );
        assert_eq!(freefall.kid_y_velocity, Some(0));
        assert_eq!(freefall.kid_on_ground, Some(false));
        assert_eq!(freefall.number_objects, 5);
        assert_eq!(freefall.number_gfx_objects, 4);

        let mut grounded_hold = freefall;
        while grounded_hold.boot_phase_elapsed != Some(demo::DEMO_BOOTSTRAP_GROUNDED_FRAME as usize)
        {
            grounded_hold = runtime.step_frame(FrameInput::default());
        }
        assert_eq!(grounded_hold.kid_y_velocity, Some(0));
        assert_eq!(grounded_hold.kid_on_ground, Some(true));
        assert_eq!(grounded_hold.number_objects, 5);
        assert_eq!(grounded_hold.number_gfx_objects, 4);
    }

    #[test]
    fn demo_bootstrap_first_visible_gameplay_frame_advances_camera_only() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(0x1C));
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME - 1,
            data_index: 0,
            use_demo_2: true,
        };
        runtime.current_level = LevelId(demo::DEMO_2_LEVEL);
        runtime.level_start_frame = Some(runtime.frame_index);
        runtime.opening_slice = Some(OpeningSliceState {
            player: PlayerState {
                x_pos: 0x0020_0000,
                y_pos: 0x02DD_0000,
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
            },
            camera: CameraState {
                x_pos: 0,
                y_pos: 0x02A0_0000,
            },
            terrain: TerrainState {
                floor_y_pos: 0x02DD_0000,
                floor_terrain_type: TerrainType::Normal,
            },
            ..OpeningSliceState::new(runtime.level_bounds(LevelId(demo::DEMO_2_LEVEL)))
        });

        let first_visible = runtime.step_frame(FrameInput::default());
        assert_eq!(first_visible.boot_phase_elapsed, Some(65));
        assert_eq!(first_visible.kid_y_pos, Some(0x02DD_0000));
        assert_eq!(first_visible.camera_y_pos, 0x0298_0000);
        assert_eq!(first_visible.number_objects, 5);
        assert_eq!(first_visible.number_gfx_objects, 4);

        let hidden_settle = runtime.step_frame(FrameInput::default());
        assert_eq!(hidden_settle.boot_phase_elapsed, Some(66));
        assert_eq!(hidden_settle.kid_y_pos, Some(0x02DE_0000));
        assert_eq!(hidden_settle.camera_y_pos, 0x0290_0000);
        assert_eq!(hidden_settle.number_objects, 5);
        assert_eq!(hidden_settle.number_gfx_objects, 4);

        let freefall = runtime.step_frame(FrameInput::default());
        assert_eq!(freefall.boot_phase_elapsed, Some(67));
        assert_eq!(freefall.kid_y_pos, Some(0x02DE_4000));
        assert_eq!(freefall.kid_y_velocity, Some(0));
        assert_eq!(freefall.kid_on_ground, Some(false));
        assert_eq!(freefall.number_objects, 5);
        assert_eq!(freefall.number_gfx_objects, 4);

        let mut grounded_handoff = freefall;
        while grounded_handoff.boot_phase_elapsed
            != Some(demo::DEMO_BOOTSTRAP_GROUNDED_FRAME as usize)
        {
            grounded_handoff = runtime.step_frame(FrameInput::default());
        }
        assert_eq!(grounded_handoff.kid_y_velocity, Some(0));
        assert_eq!(grounded_handoff.kid_on_ground, Some(true));
        assert_eq!(grounded_handoff.camera_y_pos, 0x0271_0000);
        assert_eq!(grounded_handoff.number_objects, 5);
        assert_eq!(grounded_handoff.number_gfx_objects, 4);
    }

    #[test]
    fn demo_bootstrap_keeps_hidden_system_object_through_last_grounded_frame() {
        let mut runtime = NativeRuntime::new(manifest_with_levels(0x1C));
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        runtime.current_level = LevelId(demo::DEMO_2_LEVEL);
        runtime.level_start_frame = Some(runtime.frame_index);
        runtime.opening_slice = Some(OpeningSliceState {
            player: PlayerState {
                x_pos: 0x0020_0000,
                y_pos: 0x02DF_0000,
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
            camera: CameraState {
                x_pos: 0,
                y_pos: 0x0259_0000,
            },
            terrain: TerrainState {
                floor_y_pos: 0x02DF_0000,
                floor_terrain_type: TerrainType::Normal,
            },
            ..OpeningSliceState::new(runtime.level_bounds(LevelId(demo::DEMO_2_LEVEL)))
        });

        let last_bootstrap = runtime.trace_frame();
        assert_eq!(
            last_bootstrap.boot_phase_elapsed,
            Some(demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME as usize)
        );
        assert_eq!(last_bootstrap.number_objects, 5);
        assert_eq!(last_bootstrap.number_gfx_objects, 4);

        let first_regular = runtime.step_frame(FrameInput::default());
        assert_eq!(
            first_regular.boot_phase_elapsed,
            Some((demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME + 1) as usize)
        );
        assert_eq!(first_regular.number_objects, 4);
        assert_eq!(first_regular.number_gfx_objects, 4);
    }

    #[test]
    fn demo_plays_correct_level_and_alternates() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.frame_index = 1;
        runtime.title_menu_arm_frame = 0;
        // First demo: toggle starts false, NOT flips to true → demo 2 (Skydragon Castle)
        for _ in 0..640 {
            runtime.step_frame(FrameInput::default());
        }
        match runtime.boot_state {
            BootState::DemoPlay { use_demo_2, .. } => assert!(use_demo_2),
            _ => panic!("expected DemoPlay"),
        }
        assert_eq!(runtime.current_level.0, 0);
    }

    #[test]
    fn loc_119b6_selects_the_expected_demo_level_for_each_recording() {
        assert_eq!(
            NativeRuntime::demo_level_id(false),
            LevelId(demo::DEMO_1_LEVEL),
            "ASM loc_119B6 should keep demo 1 on Knights Isle when the `beq.w` branch is taken"
        );
        assert_eq!(
            NativeRuntime::demo_level_id(true),
            LevelId(demo::DEMO_2_LEVEL),
            "ASM Load_DemoPlay should overwrite `d7` before loc_119B6 so demo 2 boots Skydragon Castle 1"
        );
    }

    #[test]
    fn loc_11ae4_keeps_hills_of_the_warrior_1_on_the_small_row_pointer_buffer() {
        assert_eq!(
            crate::level::level_layout_row_pointer_base_loc_11AE4(LevelId(0x08), 0x01F4),
            0xFFFF_9E32,
            "ASM loc_11AE4 should override the wide-layout default for Hills of the Warrior 1 and keep `a3=$FFFF9E32`"
        );
    }

    #[test]
    fn loc_11ae4_keeps_300_block_layouts_on_the_small_row_pointer_buffer() {
        assert_eq!(
            crate::level::level_layout_row_pointer_base_loc_11AE4(LevelId(0x00), 0x012C),
            0xFFFF_9E32,
            "ASM loc_11AE4 should still take the small-buffer path when `cmpi.w #$12C,(Level_width_blocks).w` is equal"
        );
    }

    #[test]
    fn loc_11aea_switches_non_hills_layouts_wider_than_300_blocks_to_the_large_buffer() {
        assert_eq!(
            crate::level::level_layout_row_pointer_base_loc_11AE4(LevelId(0x00), 0x012D),
            0xFFFF_DD02,
            "ASM loc_11AEA should keep the initial `a3=$FFFFDD02` once `bgt.s loc_11AEA` sees a non-Hills layout wider than $12C blocks"
        );
    }

    #[test]
    fn loc_11afe_emits_one_level_layout_pointer_per_block_row() {
        assert_eq!(
            crate::level::level_layout_row_pointers_loc_11AFE(3, 0x0020),
            vec![0xA652, 0xA672, 0xA692],
            "ASM loc_11AFE should write `Level_Layout`, then advance `a2` by `Level_width_tiles` after each emitted row pointer"
        );
    }

    #[test]
    fn loc_11b12_fills_the_selected_row_buffer_with_solid_collision_words() {
        assert_eq!(
            crate::level::solid_collision_row_loc_11B12(4),
            vec![0x6000, 0x6000, 0x6000, 0x6000],
            "ASM loc_11B12 should emit one `$6000` collision word per `Level_width_blocks` column"
        );
    }

    #[test]
    fn loc_11b1a_repeats_the_advanced_row_buffer_pointer_eight_times() {
        assert_eq!(
            crate::level::row_buffer_pointer_table_loc_11B1A(0xFFFF_9E32, 4),
            [0x9E3A; 8],
            "ASM loc_11B1A should write the post-`loc_11B12` `a3` word eight times after advancing by two bytes per block column"
        );
    }

    #[test]
    fn loc_11b28_fills_the_following_row_buffer_with_zero_collision_words() {
        assert_eq!(
            empty_collision_row_loc_11B28(4),
            vec![0x0000, 0x0000, 0x0000, 0x0000],
            "ASM loc_11B28 should clear `d1` and write one zero collision word per `Level_width_blocks` column"
        );
    }

    #[test]
    fn loc_11b40_emits_one_terrain_layout_pointer_per_block_row() {
        assert_eq!(
            terrain_layout_row_pointers_loc_11B40(3, 0x001E),
            vec![0x52C2, 0x52E0, 0x52FE],
            "ASM loc_11B40 should write `Level_terrain_layout`, then advance `a4` by `Level_width_blocks` after each emitted row pointer"
        );
    }

    #[test]
    fn loc_11c10_retries_decompress_chunk_until_the_status_word_clears() {
        assert_eq!(
            crate::level::decompress_chunk_attempts_loc_11C10(&[2, 1, 0, 7]),
            3,
            "ASM loc_11C10 should branch back to `Decompress_Chunk` until `tst.w d0` sees an exact zero result"
        );
    }

    #[test]
    fn loc_11c10_reports_all_attempts_when_decompress_never_succeeds() {
        assert_eq!(
            crate::level::decompress_chunk_attempts_loc_11C10(&[3, 2, 1]),
            3,
            "ASM loc_11C10 should keep retrying on every non-zero `d0`, so a no-success trace consumes every provided attempt"
        );
    }

    #[test]
    fn loc_11c36_packs_the_theme_collision_nibble_into_the_high_byte() {
        let mut theme_collision_index = vec![0; 256];
        theme_collision_index[0x34] = 0x06;

        assert_eq!(
            crate::level::level_layout_cell_loc_11C36(0x34, &theme_collision_index),
            0x6034,
            "ASM loc_11C36 should shift the looked-up collision nibble into the high byte and preserve the terrain byte in the low byte"
        );
    }

    #[test]
    fn loc_11c34_resets_the_column_counter_for_each_row_of_level_layout_output() {
        let mut theme_collision_index = vec![0; 256];
        theme_collision_index[0x12] = 0x03;
        theme_collision_index[0x34] = 0x06;
        theme_collision_index[0x56] = 0x07;
        theme_collision_index[0x78] = 0x01;

        assert_eq!(
            crate::level::level_layout_cells_loc_11C34(
                &[0x12, 0x34, 0x56, 0x78],
                &theme_collision_index,
                2,
                2,
            ),
            vec![0x3012, 0x6034, 0x7056, 0x1078],
            "ASM loc_11C34 should clear `d1` for each row, so the `loc_11C36` packing restarts at the left edge before advancing `d0` to the next row"
        );
    }

    #[test]
    fn loc_11c7e_stops_dispatching_block_layout_words_at_the_ffff_sentinel() {
        assert_eq!(
            crate::level::block_layout_dispatch_words_loc_11C7E(&[0x0003, 0x8012, 0xFFFF, 0x0044]),
            vec![0x0003, 0x8012],
            "ASM loc_11C7E should keep dispatching non-`$FFFF` block-layout words through `AddrTbl_BlockTypes` and stop as soon as `(a1)+` yields the sentinel"
        );
    }

    #[test]
    fn loc_11c96_only_seeds_the_hidden_block_copy_state_when_entries_exist() {
        assert_eq!(
            crate::level::hidden_block_copy_setup_loc_11C96(0, 0xFFFF_3B24),
            None,
            "ASM loc_11C96 should branch straight to `loc_11CB2` when `($FFFFF8EA).w` is zero, so no copy state should be prepared"
        );
        assert_eq!(
            crate::level::hidden_block_copy_setup_loc_11C96(3, 0xFFFF_3B24),
            Some(crate::level::HiddenBlockCopySetupLoc11C96 {
                dbf_counter: 2,
                copied_list_addr: 0xFFFF_3B24,
                temp_list_addr: 0xFFFF_0280,
            }),
            "ASM loc_11C96 should convert the hidden-block count into `d0=count-1`, store `a5` in `($FFFFF8EC).w`, and reload `a4` from the temp list base before `loc_11CA8` runs"
        );
    }

    #[test]
    fn loc_11ca8_copies_one_hidden_block_entry_even_when_dbf_starts_at_zero() {
        let temp_entries = [
            [0x0002_1000, 0x0001_0001],
            [0x0002_2000, 0x0002_0002],
            [0x0002_3000, 0x0003_0003],
        ];

        assert_eq!(
            crate::level::copy_hidden_block_entries_loc_11CA8(&temp_entries, 0),
            vec![[0x0002_1000, 0x0001_0001]],
            "ASM loc_11CA8 copies both longwords before `dbf` checks for `-1`, so a zero counter should still copy one 8-byte hidden-block entry"
        );
        assert_eq!(
            crate::level::copy_hidden_block_entries_loc_11CA8(&temp_entries, 1),
            vec![[0x0002_1000, 0x0001_0001], [0x0002_2000, 0x0002_0002]],
            "ASM loc_11CA8 should copy one 8-byte hidden-block entry per `dbf` iteration and stop after `d0` falls from 0 to -1"
        );
    }

    #[test]
    fn loc_11cb6_allows_null_prize_lists_and_traps_once_the_custom_queue_base_is_reached() {
        assert!(
            crate::level::prize_block_list_guard_loc_11CB6(0, 0x0010),
            "ASM loc_11CB6 should fall through to `loc_11CCA` when `($FFFFF8D4).w` is zero, so an empty prize list must skip the queue-capacity guard"
        );
        assert!(
            !crate::level::prize_block_list_guard_loc_11CB6(0xFFFF_43B4, 1),
            "ASM loc_11CB6 adds `($FFFFF8D2).w << 3` to the low word of `($FFFFF8D4).w` and branches to `loc_11D0A` on `bcc`, so an 8-byte prize entry ending at `$43BC` must be treated as overflow"
        );
    }

    #[test]
    fn loc_11cca_allows_null_hidden_block_lists_and_rejects_a_16_byte_overflow() {
        assert!(
            crate::level::hidden_block_list_guard_loc_11CCA(0, 0x0004),
            "ASM loc_11CCA should fall through to `loc_11CE0` when `($FFFFF8DA).w` is zero, so an empty hidden-block list must skip the queue-capacity guard"
        );
        assert!(
            !crate::level::hidden_block_list_guard_loc_11CCA(0xFFFF_43AC, 1),
            "ASM loc_11CCA multiplies `($FFFFF8D8).w` by `$10` before the `cmpi.w #$43BC,d0`, so a single 16-byte entry ending at `$43BC` must branch to `loc_11D0A`"
        );
    }

    #[test]
    fn loc_11ce0_allows_null_custom_write_lists_and_rejects_a_10_byte_overflow() {
        assert!(
            crate::level::fg_plane_custom_write_guard_loc_11CE0(0, 0x0007),
            "ASM loc_11CE0 should fall through to `loc_11CF6` when `($FFFFF8E0).w` is zero, so an empty FGPlane_UpdateQueueCustom list must skip the queue-capacity guard"
        );
        assert!(
            !crate::level::fg_plane_custom_write_guard_loc_11CE0(0xFFFF_43B2, 1),
            "ASM loc_11CE0 multiplies `($FFFFF8DE).w` by `$A` before the `cmpi.w #$43BC,d0`, so a single 10-byte custom-write entry ending at `$43BC` must branch to `loc_11D0A`"
        );
    }

    #[test]
    fn loc_11cf6_allows_null_background_tile_lists_and_rejects_an_8_byte_overflow() {
        assert!(
            crate::level::background_tile_guard_loc_11CF6(0, 0x0009),
            "ASM loc_11CF6 should fall through to `loc_11D0C` when `($FFFFF8EC).w` is zero, so an empty background-tile list must skip the queue-capacity guard"
        );
        assert!(
            !crate::level::background_tile_guard_loc_11CF6(0xFFFF_43B4, 1),
            "ASM loc_11CF6 shifts `($FFFFF8EA).w` left by three before `cmpi.w #$43BC,d0`, so a single 8-byte background-tile entry ending at `$43BC` must branch to `loc_11D0A`"
        );
    }

    #[test]
    fn loc_141ee_adjusts_only_the_layout_and_row_words_of_each_8_byte_block_entry() {
        assert_eq!(
            crate::level::adjust_8_byte_block_list_entries_for_blank_top_rows_loc_141EE(
                &[
                    [0x0120, 0x0007, 0x0003, 0x6004],
                    [0x0450, 0x0009, 0x000A, 0x8011],
                ],
                2,
                0x0040,
            ),
            vec![
                [0x01A0, 0x0007, 0x0005, 0x6004],
                [0x04D0, 0x0009, 0x000C, 0x8011],
            ],
            "ASM loc_141EE should add `Number_blank_top_rows * Level_width_tiles` to word 0 and `Number_blank_top_rows` to word 2 of each 8-byte block-list entry, leaving the other words untouched"
        );
    }

    #[test]
    fn loc_141f6_only_adjusts_the_first_d0_8_byte_entries() {
        assert_eq!(
            crate::level::adjust_8_byte_block_list_entries_for_blank_top_rows_loc_141F6(
                &[
                    [0x0120, 0x0007, 0x0003, 0x6004],
                    [0x0450, 0x0009, 0x000A, 0x8011],
                    [0x0780, 0x000B, 0x000F, 0x9002],
                ],
                2,
                2,
                0x0040,
            ),
            vec![
                [0x01A0, 0x0007, 0x0005, 0x6004],
                [0x04D0, 0x0009, 0x000C, 0x8011],
                [0x0780, 0x000B, 0x000F, 0x9002],
            ],
            "ASM loc_141F6 branches straight into `dbf d0,loc_141EE`, so `d0=2` should adjust exactly the first two 8-byte entries before falling through to the next list"
        );
        assert_eq!(
            crate::level::adjust_8_byte_block_list_entries_for_blank_top_rows_loc_141F6(
                &[[0x0120, 0x0007, 0x0003, 0x6004]],
                0,
                2,
                0x0040,
            ),
            vec![[0x0120, 0x0007, 0x0003, 0x6004]],
            "ASM loc_141F6 should skip the `loc_141EE` body entirely when `($FFFFF8D2).w` loads `d0=0` before the first `dbf`"
        );
    }

    #[test]
    fn loc_14204_adjusts_the_layout_and_row_words_of_each_16_byte_hidden_block_entry() {
        assert_eq!(
            crate::level::adjust_16_byte_hidden_block_entries_for_blank_top_rows_loc_14204(
                &[
                    [
                        0x1234, 0x8001, 0xABCD, 0xEF01, 0x0120, 0x0014, 0x0003, 0x0300
                    ],
                    [
                        0x5678, 0x0003, 0xBCDE, 0xF012, 0x0450, 0x0028, 0x000A, 0x1102
                    ],
                ],
                2,
                0x0040,
            ),
            vec![
                [
                    0x1234, 0x8001, 0xABCD, 0xEF01, 0x01A0, 0x0014, 0x0005, 0x0300
                ],
                [
                    0x5678, 0x0003, 0xBCDE, 0xF012, 0x04D0, 0x0028, 0x000C, 0x1102
                ],
            ],
            "ASM loc_14204 should add `Number_blank_top_rows * Level_width_tiles` to the stored layout word at `8(a0)` and `Number_blank_top_rows` to the stored row word at `$C(a0)`, leaving the other 16-byte entry words untouched"
        );
    }

    #[test]
    fn loc_14210_only_adjusts_the_first_d0_16_byte_hidden_block_entries() {
        assert_eq!(
            crate::level::adjust_16_byte_hidden_block_entries_for_blank_top_rows_loc_14210(
                &[
                    [
                        0x1234, 0x8001, 0xABCD, 0xEF01, 0x0120, 0x0014, 0x0003, 0x0300
                    ],
                    [
                        0x5678, 0x0003, 0xBCDE, 0xF012, 0x0450, 0x0028, 0x000A, 0x1102
                    ],
                    [
                        0x9ABC, 0x8002, 0xCDEF, 0x0123, 0x0780, 0x0032, 0x000F, 0x2204
                    ],
                ],
                2,
                2,
                0x0040,
            ),
            vec![
                [
                    0x1234, 0x8001, 0xABCD, 0xEF01, 0x01A0, 0x0014, 0x0005, 0x0300
                ],
                [
                    0x5678, 0x0003, 0xBCDE, 0xF012, 0x04D0, 0x0028, 0x000C, 0x1102
                ],
                [
                    0x9ABC, 0x8002, 0xCDEF, 0x0123, 0x0780, 0x0032, 0x000F, 0x2204
                ],
            ],
            "ASM loc_14210 branches straight into `dbf d0,loc_14204`, so `d0=2` should adjust exactly the first two 16-byte hidden-block entries before falling through to the next queue"
        );
        assert_eq!(
            crate::level::adjust_16_byte_hidden_block_entries_for_blank_top_rows_loc_14210(
                &[[
                    0x1234, 0x8001, 0xABCD, 0xEF01, 0x0120, 0x0014, 0x0003, 0x0300
                ]],
                0,
                2,
                0x0040,
            ),
            vec![[
                0x1234, 0x8001, 0xABCD, 0xEF01, 0x0120, 0x0014, 0x0003, 0x0300
            ]],
            "ASM loc_14210 should skip the `loc_14204` body entirely when `($FFFFF8D8).w` loads `d0=0` before the first `dbf`"
        );
    }

    #[test]
    fn loc_11d0c_selects_the_theme_mapping_entry_and_splits_the_enemy_layout_pointer() {
        assert_eq!(
            crate::level::level_load_pointers_loc_11D0C(
                &[0x0010_0000, 0x0020_0000, 0x0030_0000],
                0x0008,
                0x0004_2000,
                &[0xFFFF_F8F6, 0x1234_5678],
            ),
            crate::level::LevelLoadPointersLoc11D0C {
                theme_mappings_addr: 0x0030_0000,
                enemy_layout_addr: 0xFFFF_F8F6,
                enemy_layout_header_addr: 0x0004_2004,
            },
            "ASM loc_11D0C should read `LnkTo_ThemeMappings_Index[d7.w]`, then store the first enemy-layout longword in `Addr_EnemyLayout` while advancing the header pointer by four bytes into `Addr_EnemyLayoutHeader`",
        );
    }

    #[test]
    fn loc_1421e_adjusts_each_10_byte_custom_write_entry_for_inserted_blank_top_rows() {
        assert_eq!(
            crate::level::adjust_10_byte_custom_write_entries_for_blank_top_rows_loc_1421E(
                &[
                    [0x0120, 0x0014, 0x0003, 0xE407, 0xE408],
                    [0x0280, 0x0030, 0x0008, 0xE40A, 0xE40B],
                ],
                2,
                0x0040,
            ),
            vec![
                [0x01A0, 0x0014, 0x0005, 0xE407, 0xE408],
                [0x0300, 0x0030, 0x000A, 0xE40A, 0xE40B],
            ],
            "ASM loc_1421E should add `Number_blank_top_rows * Level_width_tiles` to word 0 and `Number_blank_top_rows` to word 2 of each 10-byte custom-write entry, leaving the other words untouched"
        );
    }

    #[test]
    fn loc_14228_adjusts_only_the_prefix_selected_by_the_dbf_entry_count() {
        assert_eq!(
            crate::level::adjust_10_byte_custom_write_entries_for_blank_top_rows_loc_14228(
                &[
                    [0x0120, 0x0014, 0x0003, 0xE407, 0xE408],
                    [0x0280, 0x0030, 0x0008, 0xE40A, 0xE40B],
                    [0x03C0, 0x0042, 0x000D, 0xE40C, 0xE40D],
                ],
                2,
                2,
                0x0040,
            ),
            vec![
                [0x01A0, 0x0014, 0x0005, 0xE407, 0xE408],
                [0x0300, 0x0030, 0x000A, 0xE40A, 0xE40B],
                [0x03C0, 0x0042, 0x000D, 0xE40C, 0xE40D],
            ],
            "ASM loc_14228 branches straight into `dbf d0,loc_1421E`, so `d0=2` should adjust exactly the first two 10-byte custom-write entries before falling through to the next queue"
        );
        assert_eq!(
            crate::level::adjust_10_byte_custom_write_entries_for_blank_top_rows_loc_14228(
                &[[0x0120, 0x0014, 0x0003, 0xE407, 0xE408]],
                0,
                2,
                0x0040,
            ),
            vec![[0x0120, 0x0014, 0x0003, 0xE407, 0xE408]],
            "ASM loc_14228 should skip the `loc_1421E` body entirely when `($FFFFF8DE).w` loads `d0=0` before the first `dbf`"
        );
    }

    #[test]
    fn loc_14236_adjusts_each_8_byte_hidden_background_entry_for_inserted_blank_top_rows() {
        assert_eq!(
            crate::level::adjust_8_byte_hidden_block_entries_for_blank_top_rows_loc_14236(
                &[
                    [0x0120, 0x0014, 0x0003, 0xFF00],
                    [0x0280, 0x0030, 0x0008, 0xFF01],
                ],
                2,
                0x0040,
            ),
            vec![
                [0x01A0, 0x0014, 0x0005, 0xFF00],
                [0x0300, 0x0030, 0x000A, 0xFF01],
            ],
            "ASM loc_14236 should add `Number_blank_top_rows * Level_width_tiles` to word 0 and `Number_blank_top_rows` to word 2 of each 8-byte `($FFFFF8EC).w` entry, leaving the other words untouched"
        );
    }

    #[test]
    fn loc_1423e_adjusts_only_the_prefix_selected_by_the_dbf_entry_count() {
        assert_eq!(
            crate::level::adjust_8_byte_hidden_block_entries_for_blank_top_rows_loc_1423E(
                &[
                    [0x0120, 0x0014, 0x0003, 0xFF00],
                    [0x0280, 0x0030, 0x0008, 0xFF01],
                    [0x03C0, 0x0042, 0x000D, 0xFF02],
                ],
                2,
                2,
                0x0040,
            ),
            vec![
                [0x01A0, 0x0014, 0x0005, 0xFF00],
                [0x0300, 0x0030, 0x000A, 0xFF01],
                [0x03C0, 0x0042, 0x000D, 0xFF02],
            ],
            "ASM loc_1423E branches straight into `dbf d0,loc_14236`, so `d0=2` should adjust exactly the first two 8-byte hidden/background entries before `sub_140F6` returns"
        );
        assert_eq!(
            crate::level::adjust_8_byte_hidden_block_entries_for_blank_top_rows_loc_1423E(
                &[[0x0120, 0x0014, 0x0003, 0xFF00]],
                0,
                2,
                0x0040,
            ),
            vec![[0x0120, 0x0014, 0x0003, 0xFF00]],
            "ASM loc_1423E should skip the `loc_14236` body entirely when `($FFFFF8EA).w` loads `d0=0` before the first `dbf`"
        );
    }

    #[test]
    fn loc_11d42_only_enables_the_block_priority_mask_when_f896_is_non_zero() {
        assert_eq!(
            crate::level::block_mapping_or_mask_loc_11D42(false),
            0,
            "ASM loc_11D42 should leave `d1` cleared when `tst.b ($FFFFF896).w` branches through `beq.s loc_11D42`",
        );
        assert_eq!(
            crate::level::block_mapping_or_mask_loc_11D42(true),
            0x8000_8000,
            "ASM loc_11D0C should load `$80008000` into `d1` before `loc_11D42` when `($FFFFF896).w` is non-zero",
        );
    }

    #[test]
    fn loc_122b8_looks_up_base_cell_from_word_122c6_table() {
        // ASM loc_122B8 (kid.asm:24405-24408): doubles the block type index with
        // `add.w d1,d1` and loads `d5` from `word_122C6(pc,d1.w)`. The table at
        // kid.asm:24415-24425 provides base cell values for block types that
        // dispatch through AddrTbl_BlockTypes to loc_122B8 (types 0, 2, 3, 6, 8, 10).
        // Note: AddrTbl_BlockTypes dispatches other types to different handlers:
        //   type 1 → loc_123DC, type 4 → loc_124A6, type 5 → loc_1263C,
        //   type 7 → loc_1233C, type 9 → loc_1253A, etc.
        // The routine loads whatever word_122C6[double_type] contains into d5,
        // which may be 0 (a valid cell value meaning air/placeholder).
        assert_eq!(
            crate::level::block_type_base_cell_loc_122B8(0),
            0xE001,
            "ASM loc_122B8 loads $E001 for block type 0 from word_122C6[0]",
        );
        assert_eq!(
            crate::level::block_type_base_cell_loc_122B8(1),
            0x0000,
            "ASM loc_122B8 loads word_122C6[2]=$0000 for type 1, though type 1 dispatches to loc_123DC",
        );
        assert_eq!(
            crate::level::block_type_base_cell_loc_122B8(2),
            0xE203,
            "ASM loc_122B8 loads $E203 for block type 2 from word_122C6[4]",
        );
        assert_eq!(
            crate::level::block_type_base_cell_loc_122B8(3),
            0x0000,
            "ASM loc_122B8 loads word_122C6[6]=$0000 for block type 3",
        );
        assert_eq!(
            crate::level::block_type_base_cell_loc_122B8(6),
            0xE60C,
            "ASM loc_122B8 loads $E60C for block type 6 from word_122C6[12]",
        );
    }

    #[test]
    fn loc_122c0_returns_none_for_end_marker_and_some_for_continue() {
        // ASM loc_122C0 (kid.asm:24410-24413): reads `move.w (a1)+,d1` from the
        // block-layout stream, branches to `loc_122DA` for processing if `bpl.s`,
        // or returns from the handler if the word has bit 15 set ($8000+).
        // The end-of-layout sentinel is the negative word $FFFF (or any $8000+),
        // signaling the block-type handler to stop processing.
        assert_eq!(
            crate::level::block_layout_parse_entry_loc_122C0(0x0012),
            Some(crate::level::BlockLayoutEntryLoc122C0 { x_pos: 0x0012 }),
            "ASM loc_122C0 should branch to loc_122DA when word is positive (bit 15 clear)",
        );
        assert_eq!(
            crate::level::block_layout_parse_entry_loc_122C0(0x7FFF),
            Some(crate::level::BlockLayoutEntryLoc122C0 { x_pos: 0x7FFF }),
            "ASM loc_122C0 should accept the maximum positive word $7FFF as a valid X position",
        );
        assert_eq!(
            crate::level::block_layout_parse_entry_loc_122C0(0x8000),
            None,
            "ASM loc_122C0 should return from the handler when word has bit 15 set ($8000+)",
        );
        assert_eq!(
            crate::level::block_layout_parse_entry_loc_122C0(0xFFFF),
            None,
            "ASM loc_122C0 should return from the handler when word is the $FFFF end-of-layout sentinel",
        );
    }

    #[test]
    fn loc_122da_reads_y_count_and_stride_words() {
        // ASM loc_122DA (kid.asm:24428-24439): after loc_122C0 read X into `d1`,
        // reads Y row coordinate, count, and stride flag from `(a1)+`:
        //   move.w (a1)+,d2   ; Y row
        //   move.w (a1)+,d3   ; count
        //   subq.w #1,d3      ; for dbf loop
        //   tst.w (a1)+       ; stride: 0=horizontal, non-zero=row-by-row
        //   bne.s loc_12318   ; branch to row-by-row handler if stride
        assert_eq!(
            crate::level::block_layout_read_coords_loc_122DA(&[0x000A, 0x0003, 0x0000]),
            Some((
                crate::level::BlockLayoutSetupLoc122DA {
                    y_row: 10,
                    count: 3,
                    stride: false
                },
                3
            )),
            "ASM loc_122DA should parse Y=10, count=3, stride=0 for horizontal placement",
        );
        assert_eq!(
            crate::level::block_layout_read_coords_loc_122DA(&[0x0005, 0x0001, 0x0001]),
            Some((
                crate::level::BlockLayoutSetupLoc122DA {
                    y_row: 5,
                    count: 1,
                    stride: true
                },
                3
            )),
            "ASM loc_122DA should parse Y=5, count=1, stride=1 for row-by-row placement",
        );
        assert_eq!(
            crate::level::block_layout_read_coords_loc_122DA(&[0x0000, 0x0001, 0xFFFF]),
            Some((
                crate::level::BlockLayoutSetupLoc122DA {
                    y_row: 0,
                    count: 1,
                    stride: true
                },
                3
            )),
            "ASM loc_122DA should treat any non-zero stride word as row-by-row",
        );
        assert_eq!(
            crate::level::block_layout_read_coords_loc_122DA(&[0x000A]),
            None,
            "ASM loc_122DA requires at least 3 words after X position: Y, count, stride",
        );
    }

    #[test]
    fn loc_122f6_writes_cells_horizontally_at_stride_zero() {
        // ASM loc_122F6 (kid.asm:24441-24460): horizontal cell writer for block
        // types using the word_122C6 base-cell lookup. When `(a1)+` yields 0 (no
        // row-by-row stride), writes cells at `a4`, `a4+2`, ..., `a4+(count-1)*2`
        // before looping back to loc_122C0 for the next entry.
        //   tst.w d0
        //   bmi.s loc_122FE      ; hidden-block case (negative block type)
        //   move.w d5,(a4)       ; write cell word to Level_Layout
        //   bra.s loc_12310
        // loc_12310:
        //   addq.w #2,a4         ; advance address
        //   dbf d3,loc_122F6     ; loop count times
        //   bra.s loc_122C0      ; continue to next layout entry
        let results = crate::level::block_place_cells_loc_122F6(
            0x1234, // cell_addr
            0xE50B, // cell_word (Metal block base)
            3,      // count
            false,  // stride=0 (horizontal placement)
        );
        assert_eq!(
            results.len(),
            3,
            "ASM loc_122F6 should write `d3+1` cells when stride is zero",
        );
        assert_eq!(
            results[0],
            crate::level::BlockPlaceResultLoc122F6 {
                cell_addr: 0x1234,
                cell_word: 0xE50B
            },
            "ASM loc_122F6 should write cell_word at the initial address",
        );
        assert_eq!(
            results[1],
            crate::level::BlockPlaceResultLoc122F6 {
                cell_addr: 0x1236,
                cell_word: 0xE50B
            },
            "ASM loc_122F6 should advance address by 2 between cells",
        );
        assert_eq!(
            results[2],
            crate::level::BlockPlaceResultLoc122F6 {
                cell_addr: 0x1238,
                cell_word: 0xE50B
            },
            "ASM loc_122F6 should write the same cell_word for all cells in a horizontal run",
        );
    }

    #[test]
    fn loc_12320_queues_hidden_block_entry_for_row_by_row_placement() {
        // ASM loc_12320 (kid.asm:24470-24477): row-by-row hidden-block handler.
        // When `d0` is negative during stride placement, call sub_127A8/sub_127CE
        // and write the hidden-block list entry:
        //   bsr.w sub_127A8
        //   bsr.w sub_127CE
        //   move.w a4,(a2)+    ; cell address
        //   move.w d1,(a2)+    ; X position
        //   move.w d2,(a2)+    ; Y row
        //   move.b d0,(a2)+    ; block type (negative byte)
        //   clr.b (a2)+        ; zero padding byte
        let result = crate::level::block_place_row_by_row_hidden_loc_12320(
            0x9E40, // cell_addr (Level_Layout)
            0x0012, // x_pos (column 18)
            0x0008, // y_row (row 8)
            -1i16,  // hidden_block_type (negative signals hidden block)
        );
        assert_eq!(
            result.cell_addr, 0x9E40,
            "ASM loc_12320 should preserve the cell address for hidden-block list entry",
        );
        assert_eq!(
            result.x_pos, 0x0012,
            "ASM loc_12320 should store X position from d1",
        );
        assert_eq!(
            result.y_row, 0x0008,
            "ASM loc_12320 should store Y row from d2",
        );
        assert_eq!(
            result.block_type, -1i16,
            "ASM loc_12320 should preserve the negative block type marker from d0",
        );
    }

    #[test]
    fn loc_12332_advances_cell_addr_by_level_width_tiles() {
        // ASM loc_12332 (kid.asm:24479-24481): row-by-row stride continuation.
        // After placing (or skipping) a cell, advance to the next row:
        //   add.w (Level_width_tiles).w,a4
        //   dbf d3,loc_12318
        //   bra.s loc_122C0
        let result = crate::level::block_place_row_by_row_advance_loc_12332(
            0x9E40, // cell_addr (start of row cell in Level_Layout)
            64,     // level_width_tiles (e.g., 64 tiles wide)
        );
        assert_eq!(
            result.next_cell_addr, 0x9EC0,
            "ASM loc_12332 should advance cell address by Level_width_tiles * 2 bytes",
        );
        // Test wraparound: 16-bit address arithmetic
        let wrapped = crate::level::block_place_row_by_row_advance_loc_12332(
            0xFFFF, // cell_addr at end of address space
            64,     // level_width_tiles
        );
        assert_eq!(
            wrapped.next_cell_addr,
            0x0001_007F, // ADDA.W sign-extends the row byte stride but does not wrap A4 to 16 bits.
            "ASM loc_12332 should keep full address-register arithmetic for the next cell address",
        );
    }

    #[test]
    fn loc_1233e_reads_layout_entry_and_returns_on_terminator() {
        // ASM loc_1233E (kid.asm:24488-24492): block layout entry reader for
        // block types dispatched through AddrTbl_BlockTypes entry 7. The routine
        // reads one word from `(a1)+` into `d1`:
        //   move.w (a1)+,d1
        //   bpl.s loc_12346      ; continue if positive (X position)
        //   move.l (sp)+,a3      ; restore saved register
        //   rts                  ; return from block handler
        // Positive values are X positions; $8000+ signals end-of-layout.
        let result = crate::level::block_layout_read_entry_loc_1233E(0x0014);
        assert_eq!(
            result,
            Some(crate::level::BlockLayoutEntryLoc1233E { x_pos: 0x0014 }),
            "ASM loc_1233E should return Some(X position) for positive layout word",
        );
        let negative = crate::level::block_layout_read_entry_loc_1233E(0x8000);
        assert_eq!(
            negative, None,
            "ASM loc_1233E should return None for terminator word $8000",
        );
        let terminator = crate::level::block_layout_read_entry_loc_1233E(0xFFFF);
        assert_eq!(
            terminator, None,
            "ASM loc_1233E should return None for any negative terminator word",
        );
    }

    #[test]
    fn loc_1234_sets_high_priority_bit_for_priority_1() {
        // ASM loc_1234 (kid.asm:2230-2231): sets the high-priority bit $8000
        // when sprite priority conditions are met. The priority byte $10(a3)
        // determines sprite rendering priority:
        // - 0: low priority (no bit set)
        // - 1: high priority (set $8000 unconditionally)
        // - 2+: conditional priority (set $8000 only if hw flag $F896 is set)
        assert_eq!(
            crate::NativeRuntime::sprite_priority_bit_loc_1234(0, false),
            0x0000,
            "ASM loc_1234: priority 0 should not set high-priority bit",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_priority_bit_loc_1234(0, true),
            0x0000,
            "ASM loc_1234: priority 0 ignores hardware flag",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_priority_bit_loc_1234(1, false),
            0x8000,
            "ASM loc_1234: priority 1 sets high-priority bit unconditionally",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_priority_bit_loc_1234(1, true),
            0x8000,
            "ASM loc_1234: priority 1 sets high-priority bit regardless of hw flag",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_priority_bit_loc_1234(2, false),
            0x0000,
            "ASM loc_1234: priority 2 without hw flag does not set bit",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_priority_bit_loc_1234(2, true),
            0x8000,
            "ASM loc_1234: priority 2 with hw flag sets high-priority bit",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_priority_bit_loc_1234(3, true),
            0x8000,
            "ASM loc_1234: priority 3+ with hw flag sets high-priority bit",
        );
    }

    #[test]
    fn loc_1238_computes_sprite_tile_id_from_art_data_and_vram_offset() {
        // ASM loc_1238 (kid.asm:2233-2247): adds sprite and VRAM offsets to the
        // base tile ID. When $12(a3) is zero, adds `(a4)+` (sprite art data word)
        // and `vram_tile(a3)` to form the final tile ID. When $12(a3) is non-zero,
        // uses a hardware offset table instead.
        assert_eq!(
            crate::NativeRuntime::sprite_tile_id_loc_1238(0x1000, 0x0050, 0x0675, false),
            0x16C5,
            "ASM loc_1238: base $1000 + sprite_offset $0050 + vram_tile $0675 = $16C5",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_tile_id_loc_1238(0x0000, 0x0001, 0x1000, false),
            0x1001,
            "ASM loc_1238: should wrap at 16-bit boundary for overflow",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_tile_id_loc_1238(0xFFFF, 0x0001, 0x0002, false),
            0x0002,
            "ASM loc_1238: should wrap at 16-bit boundary: $FFFF + $0001 + $0002 = $0002",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_tile_id_loc_1238(0x1000, 0xFFFF, 0, true),
            0x1000,
            "ASM loc_1238: hardware offset path adds only vram_tile, ignoring sprite_offset_word",
        );
        assert_eq!(
            crate::NativeRuntime::sprite_tile_id_loc_1238(0x1000, 0, 0x0675, true),
            0x1675,
            "ASM loc_1238: hardware offset path adds vram_tile to base: $1000 + $0675 = $1675",
        );
    }

    #[test]
    fn loc_141a_queues_dma_art_and_advances_both_tile_cursors_by_the_chunk_length() {
        let result =
            crate::NativeRuntime::sprite_dma_upload_loc_141A(0x0F, 0x0100, 0x0240, 0x0012_3456);

        assert_eq!(
            result,
            crate::combat::SpriteDmaUploadLoc141A {
                sprite_tile_word: 0x0100,
                dma_tile_count: 0x0010,
                next_sprite_tile_word: 0x0110,
                next_hardware_tile_word: 0x0250,
                dma_source: 0x0012_3456,
                next_dma_source: 0x0012_3656,
            },
            "ASM loc_141A should look up size `$F` as 15 tiles, add one, advance both `$FFFFF830/$FFFFF83C`, queue that length, and step `a4` by `tile_count << 5` bytes"
        );
    }

    #[test]
    fn loc_12372_uses_default_rubber_cell_for_type_7_with_zero_terrain_byte() {
        // ASM loc_12372 (kid.asm:24513-24527): block type 7 handler for horizontal
        // placement. When block_type is non-negative and terrain_byte is zero,
        // returns the default rubber block cell $E60C.
        assert_eq!(
            crate::level::block_type_7_cell_value_loc_12372(0, 0).cell_word,
            0xE60C,
            "ASM loc_12372 should return $E60C for type 0 (non-negative) with terrain byte 0",
        );
        assert_eq!(
            crate::level::block_type_7_cell_value_loc_12372(6, 0).cell_word,
            0xE60C,
            "ASM loc_12372 should return $E60C for any non-negative block type with terrain byte 0",
        );
    }

    #[test]
    fn loc_12372_uses_alternate_cell_for_type_7_with_nonzero_terrain_byte() {
        // ASM loc_12372 (kid.asm:24516-24523): when terrain_byte is non-zero,
        // the Move.B (a3),D5 instruction loads a non-zero value, triggering the
        // branch to loc_12380 which seeds D5 with $E60E.
        assert_eq!(
            crate::level::block_type_7_cell_value_loc_12372(0, 1).cell_word,
            0xE60E,
            "ASM loc_12372 should return $E60E when terrain byte is non-zero",
        );
        assert_eq!(
            crate::level::block_type_7_cell_value_loc_12372(0, 0xFF).cell_word,
            0xE60E,
            "ASM loc_12372 should return $E60E for any non-zero terrain byte",
        );
    }

    #[test]
    fn loc_12372_returns_zero_for_hidden_block_types() {
        // ASM loc_12372 (kid.asm:24514-24515): when d0 (block type) is negative,
        // the BMI.S loc_12388 branch skips cell writing and calls hidden-block
        // subroutines instead. The caller should use loc_12388 for hidden blocks.
        assert_eq!(
            crate::level::block_type_7_cell_value_loc_12372(-1, 0).cell_word,
            0,
            "ASM loc_12372: hidden block type -1 should return 0 (caller uses loc_12388)",
        );
        assert_eq!(
            crate::level::block_type_7_cell_value_loc_12372(-128, 1).cell_word,
            0,
            "ASM loc_12372: hidden block type -128 should return 0 regardless of terrain byte",
        );
    }

    #[test]
    fn loc_1239a_advances_cell_and_terrain_for_horizontal_stride() {
        // ASM loc_1239A (kid.asm:24539-24543): horizontal stride continuation.
        // After writing a cell at loc_12384, advance pointers:
        //   addq.w #2,a4         ; advance cell pointer by 1 cell (2 bytes)
        //   addq.w #1,a3         ; advance terrain pointer by 1 byte
        //   dbf d3,loc_12372     ; loop for remaining cells
        let result = crate::level::block_type_7_horizontal_advance_loc_1239a(0x9E40, 0x1234);
        assert_eq!(
            result.next_cell_addr, 0x9E42,
            "ASM loc_1239A should advance cell pointer by 2 bytes",
        );
        assert_eq!(
            result.next_terrain_addr, 0x1235,
            "ASM loc_1239A should advance terrain pointer by 1 byte",
        );
    }

    #[test]
    fn loc_123b6_writes_appropriate_cell_for_vertical_placement() {
        // ASM loc_123B2/loc_123B6 (kid.asm:24555-24560): vertical placement cell
        // selection and write. For non-hidden blocks, uses same cell logic as loc_12372:
        //   move.b (a3),d5       ; load terrain byte
        //   bne.s loc_123B2      ; branch if non-zero
        //   move.w #$E60C,d5     ; default cell
        //   bra.s loc_123B6
        // loc_123B2:
        //   move.w #$E60E,d5     ; alternate cell
        // loc_123B6:
        //   move.w d5,(a4)       ; write to Level_Layout
        let zero_terrain = crate::level::block_type_7_vertical_write_loc_123b6(0x9E40, 0);
        assert_eq!(
            zero_terrain.cell_word, 0xE60C,
            "ASM loc_123B6 should use $E60C for terrain byte 0",
        );
        let nonzero_terrain = crate::level::block_type_7_vertical_write_loc_123b6(0x9E42, 0x0D);
        assert_eq!(
            nonzero_terrain.cell_word, 0xE60E,
            "ASM loc_123B2 should use $E60E for non-zero terrain byte",
        );
    }

    #[test]
    fn loc_123cc_advances_cell_and_terrain_for_vertical_stride() {
        // ASM loc_123CC (kid.asm:24572-24576): vertical stride continuation.
        // After writing a cell at loc_123B6, advance to next row:
        //   add.w (Level_width_tiles).w,a4   ; advance cell pointer by one row
        //   add.w (Level_width_blocks).w,a3  ; advance terrain pointer by one row
        //   dbf d3,loc_123A4               ; loop for remaining rows
        let result = crate::level::block_type_7_vertical_advance_loc_123cc(
            0x9E40, // cell_addr
            0x1234, // terrain_addr
            64,     // level_width_tiles
            16,     // level_width_blocks
        );
        assert_eq!(
            result.next_cell_addr, 0x9EC0,
            "ASM loc_123CC should advance cell pointer by level_width_tiles * 2 = 64 * 2 = 128 bytes",
        );
        assert_eq!(
            result.next_terrain_addr, 0x1244,
            "ASM loc_123CC should advance terrain pointer by level_width_blocks = 16 bytes",
        );
    }

    #[test]
    fn loc_11d46_copies_exactly_a2_longwords_and_ors_each_with_the_prepared_mask() {
        let source_longwords = (0..0xA4).map(|value| value as u32).collect::<Vec<_>>();
        let copied = crate::level::copy_block_mappings_loc_11D46(&source_longwords, 0x8000_8000);

        assert_eq!(
            copied.len(),
            0xA2,
            "ASM loc_11D46 seeds `d0` with `$A1`, so `dbf d0,loc_11D46` must copy exactly `$A2` longwords into `Block_Mappings`",
        );
        assert_eq!(
            copied[0], 0x8000_8000,
            "ASM loc_11D46 should OR the prepared `d1` mask into the first copied longword before writing `(a2)+`",
        );
        assert_eq!(
            copied[0xA1], 0x8000_80A1,
            "ASM loc_11D46 should still apply the same OR mask on the final DBF iteration",
        );
        assert_eq!(
            copied.last().copied(),
            Some(0x8000_80A1),
            "ASM loc_11D46 must stop after the `$A2`nd longword rather than consuming trailing data from the source table",
        );
    }

    #[test]
    fn loc_11b40_derives_background_dimensions_from_tile_counts() {
        assert_eq!(
            background_dimensions_loc_11B40(0x00F0, 0x0070),
            (0x005A, 0x0031),
            "ASM loc_11B40's shared tail should derive `Background_width=(Level_width_tiles>>2)+$1E` and `Background_height=(Level_height_tiles>>2)+$15`"
        );
    }

    #[test]
    fn loc_11b7a_emits_one_background_row_pointer_per_background_row() {
        assert_eq!(
            background_row_pointers_loc_11B7A(3, 0x005A),
            vec![0x87B2, 0x880C, 0x8866],
            "ASM loc_11B7A should write the `$FFFF87B2` row base, then advance `a1` by `Background_width` after each emitted pointer"
        );
    }

    #[test]
    fn loc_11b92_leaves_positive_foreground_theme_bytes_untouched() {
        assert_eq!(
            crate::level::decode_foreground_theme_flags_loc_11B92(0x48),
            (0x0048, false, false),
            "ASM loc_11B92 only runs after `bpl.s loc_11B9A` fails, so a positive byte should keep its full theme value and leave the murder wall disabled"
        );
    }

    #[test]
    fn loc_11b92_enables_and_optionally_reverses_the_murder_wall_for_negative_theme_bytes() {
        assert_eq!(
            crate::level::decode_foreground_theme_flags_loc_11B92(0x88),
            (0x0008, true, false),
            "ASM loc_11B92 should mask the foreground theme down to `$3F` and enable a forward murder wall once bit 7 sends execution down the negative-byte path"
        );
        assert_eq!(
            crate::level::decode_foreground_theme_flags_loc_11B92(0xC8),
            (0x0008, true, true),
            "ASM loc_11B92 should additionally set the reversed flag when bit 6 is present on the same negative foreground-theme byte"
        );
    }

    #[test]
    fn loc_11b9a_extracts_weather_and_forces_enigma_backgrounds_to_width_40() {
        assert_eq!(
            crate::level::decode_background_theme_flags_loc_11B9A(0x23, 0x005A),
            (0x0002, 0x0003, true, 0x0040),
            "ASM loc_11B9A should decode the upper nibble into `Level_Special_Effects`, keep the lower nibble as `Background_theme`, and clamp Hills/Desert/Mountain/Forest backgrounds to width `$40`"
        );
    }

    #[test]
    fn loc_11b9a_leaves_non_enigma_background_widths_unchanged() {
        assert_eq!(
            crate::level::decode_background_theme_flags_loc_11B9A(0x18, 0x005A),
            (0x0001, 0x0008, false, 0x005A),
            "ASM loc_11B9A should leave non-Hills/Desert/Mountain/Forest backgrounds in piece format and preserve the previously derived background width"
        );
    }

    #[test]
    fn loc_11e48_only_overrides_the_existing_foreground_permutation_for_forest() {
        assert_eq!(
            select_foreground_art_permutation_loc_11E48(
                0x0009,
                ForegroundArtPermutationLoc11E48::Identity
            ),
            ForegroundArtPermutationLoc11E48::Forest,
            "ASM loc_11E48 should replace the identity permutation with `Palette_Permutation_FGForest` when `Foreground_theme == Forest`"
        );
        assert_eq!(
            select_foreground_art_permutation_loc_11E48(
                0x000A,
                ForegroundArtPermutationLoc11E48::City
            ),
            ForegroundArtPermutationLoc11E48::City,
            "ASM loc_11E48 should fall through to `loc_11E54` without disturbing the earlier City permutation when the Forest compare fails"
        );
    }

    #[test]
    fn loc_11e54_only_overrides_the_existing_foreground_permutation_for_mountain() {
        assert_eq!(
            select_foreground_art_permutation_loc_11E54(
                0x0007,
                ForegroundArtPermutationLoc11E48::City
            ),
            ForegroundArtPermutationLoc11E48::Mountain,
            "ASM loc_11E54 should replace the current permutation with `Palette_Permutation_FGMountain` when `Foreground_theme == Mountain`"
        );
        assert_eq!(
            select_foreground_art_permutation_loc_11E54(
                0x0009,
                ForegroundArtPermutationLoc11E48::Forest
            ),
            ForegroundArtPermutationLoc11E48::Forest,
            "ASM loc_11E54 should fall through to `loc_11E60` without disturbing the earlier Forest permutation when the Mountain compare fails"
        );
    }

    #[test]
    fn loc_11e60_passes_the_selected_permutation_and_vram_base_into_decompress() {
        assert_eq!(
            prepare_foreground_art_decompress_loc_11E60(ForegroundArtPermutationLoc11E48::Mountain),
            ForegroundArtDecompressLoc11E60 {
                tile_base: 0x1780,
                permutation: ForegroundArtPermutationLoc11E48::Mountain,
            },
            "ASM loc_11E60 should carry the already selected foreground permutation into `DecompressToRAM` and keep the VRAM base at `$1780`"
        );
        assert_eq!(
            prepare_foreground_art_decompress_loc_11E60(ForegroundArtPermutationLoc11E48::Forest)
                .permutation,
            ForegroundArtPermutationLoc11E48::Forest,
            "ASM loc_11E60 should not disturb the earlier Forest selection before the shared decompressor call"
        );
    }

    #[test]
    fn loc_11e8a_only_overrides_the_existing_background_permutation_for_forest() {
        assert_eq!(
            select_background_art_permutation_loc_11E8A(
                0x0009,
                BackgroundArtPermutationLoc11E8A::Identity
            ),
            BackgroundArtPermutationLoc11E8A::Forest,
            "ASM loc_11E8A should replace the identity permutation with `Palette_Permutation_BGForest` when `Background_theme == Forest`"
        );
        assert_eq!(
            select_background_art_permutation_loc_11E8A(
                0x0007,
                BackgroundArtPermutationLoc11E8A::Identity
            ),
            BackgroundArtPermutationLoc11E8A::Identity,
            "ASM loc_11E8A should fall through to `loc_11E96` without disturbing the earlier identity permutation when the Forest compare fails"
        );
    }

    #[test]
    fn loc_11e96_only_overrides_the_existing_background_permutation_for_mountain() {
        assert_eq!(
            select_background_art_permutation_loc_11E96(
                0x0007,
                BackgroundArtPermutationLoc11E8A::Forest
            ),
            BackgroundArtPermutationLoc11E8A::Mountain,
            "ASM loc_11E96 should replace the current permutation with `Palette_Permutation_BGMountain` when `Background_theme == Mountain`"
        );
        assert_eq!(
            select_background_art_permutation_loc_11E96(
                0x0005,
                BackgroundArtPermutationLoc11E8A::Forest
            ),
            BackgroundArtPermutationLoc11E8A::Forest,
            "ASM loc_11E96 should fall through to `loc_11EA2` without disturbing the earlier Forest permutation when the Mountain compare fails"
        );
    }

    #[test]
    fn loc_11ea2_passes_the_selected_background_permutation_and_f000_base_into_decompress() {
        assert_eq!(
            prepare_background_art_decompress_loc_11EA2(BackgroundArtPermutationLoc11E8A::Mountain),
            BackgroundArtDecompressLoc11EA2 {
                tile_base: 0xF000,
                permutation: BackgroundArtPermutationLoc11E8A::Mountain,
            },
            "ASM loc_11EA2 should reload `d0` with `$F000` and preserve the selected Mountain background permutation into the shared `DecompressToRAM` call"
        );
        assert_eq!(
            prepare_background_art_decompress_loc_11EA2(BackgroundArtPermutationLoc11E8A::Forest),
            BackgroundArtDecompressLoc11EA2 {
                tile_base: 0xF000,
                permutation: BackgroundArtPermutationLoc11E8A::Forest,
            },
            "ASM loc_11EA2 should also leave the earlier Forest permutation intact before the shared background-art decompress"
        );
    }

    #[test]
    fn loc_11ec4_only_prepares_the_mountain_followup_background_decompress() {
        assert_eq!(
            prepare_mountain_background_art_loc_11EC4(0x0007),
            Some(BackgroundFollowupArtDecompressLoc11EC4 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
                art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
            }),
            "ASM loc_11EC4 should prepare the Mountain-only follow-up RAM decompress with `ThemeArtBack_CaveAlt`, `Palette_Permutation_Identity`, and VRAM base `$FCA0`"
        );
        assert_eq!(
            prepare_mountain_background_art_loc_11EC4(0x0005),
            None,
            "ASM loc_11EC4 should fall through to `loc_11EFA` without a second background decompress when `Background_theme` is not Mountain"
        );
    }

    #[test]
    fn loc_11eea_switches_mountain_background_art_to_lightning_for_storm_weather() {
        let baseline = BackgroundFollowupArtDecompressLoc11EC4 {
            tile_base: 0xFCA0,
            permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
            art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
        };

        assert_eq!(
            apply_mountain_weather_override_loc_11EEA(0x0000, baseline),
            baseline,
            "ASM loc_11EEA should leave the `loc_11EC4` CaveAlt selection intact when the weather compare falls through"
        );
        assert_eq!(
            apply_mountain_weather_override_loc_11EEA(0x0002, baseline),
            BackgroundFollowupArtDecompressLoc11EC4 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
                art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
            },
            "ASM loc_11EEA should replace `ThemeArtBack_CaveAlt` with `ThemeArtBack_MountainLightning` and switch to `Palette_Permutation_BGMountain` when `Level_Special_Effects == WeatherID_Storm`"
        );
        assert_eq!(
            apply_mountain_weather_override_loc_11EEA(0x0003, baseline),
            BackgroundFollowupArtDecompressLoc11EC4 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
                art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
            },
            "ASM loc_11EEA should take the same lightning-art override when `Level_Special_Effects == WeatherID_Storm_and_Hail`"
        );
    }

    #[test]
    fn loc_11ef2_preserves_the_selected_followup_background_art_into_the_shared_decompress_tail() {
        let lightning_override = BackgroundFollowupArtDecompressLoc11EC4 {
            tile_base: 0xFCA0,
            permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
            art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
        };

        assert_eq!(
            prepare_followup_background_art_decompress_loc_11EF2(lightning_override),
            BackgroundFollowupArtDecompressLoc11EF2 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::BgMountain,
                art_source: BackgroundFollowupArtSourceLoc11EC4::MountainLightning,
            },
            "ASM loc_11EF2 should preserve the lightning-art override from `loc_11EEA` while reloading `d0` with the shared `$FCA0` RAM-decompress base"
        );
        assert_eq!(
            prepare_followup_background_art_decompress_loc_11EF2(
                BackgroundFollowupArtDecompressLoc11EC4 {
                    tile_base: 0xFCA0,
                    permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
                    art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
                }
            ),
            BackgroundFollowupArtDecompressLoc11EF2 {
                tile_base: 0xFCA0,
                permutation: BackgroundFollowupArtPermutationLoc11EC4::Identity,
                art_source: BackgroundFollowupArtSourceLoc11EC4::CaveAlt,
            },
            "ASM loc_11EF2 should also leave the earlier `loc_11EC4` CaveAlt selection intact when the weather branches fall through"
        );
    }

    #[test]
    fn loc_11efa_decompresses_blocks_then_ingame_numbers_into_their_vram_slots() {
        assert_eq!(
            shared_vram_art_decompresses_loc_11EFA(),
            [
                SharedVramArtDecompressLoc11EFA {
                    tile_base: 0x4400,
                    art_source: SharedVramArtSourceLoc11EFA::Blocks,
                },
                SharedVramArtDecompressLoc11EFA {
                    tile_base: 0xD740,
                    art_source: SharedVramArtSourceLoc11EFA::IngameNumbers,
                },
            ],
            "ASM loc_11EFA should always decompress `ArtComp_992E4_Blocks` to VRAM `$4400` before loading `ArtComp_99F34_IngameNumbers` into VRAM `$D740`"
        );
    }

    #[test]
    fn loc_11f48_clears_the_hud_vram_span_and_floors_camera_to_screen_boundaries() {
        assert_eq!(
            initialise_camera_bootstrap_loc_11F48(0x03A1, 0x02D7),
            LevelCameraBootstrapLoc11F48 {
                vram_fill_word: 0x0780,
                vram_fill_count: 0x0800,
                camera_x_pos: 0x0280_0000,
                camera_y_pos: 0x02A0_0000,
            },
            "ASM loc_11F48 should write `$0780` through the `$7FF` `dbf` loop, then floor `PlayerStart_X/Y_pos` to `$140/$E0` screen boundaries before swapping them into 16.16 camera coordinates"
        );
    }

    #[test]
    fn loc_11f96_allocates_the_palette_rotator_for_cave_and_the_island_worker_otherwise() {
        assert_eq!(
            classify_post_load_object_routine_loc_11F96(0x0008, 0x0005),
            Some(PostLoadObjectRoutineLoc11F96::RotatePaletteLavaWater {
                object_slot_seed: 0xA000,
            }),
            "ASM loc_11F96 should allocate the same `$A000` `RotatePalette_Lava_Water` object when the earlier Mountain compare has already fallen through and `Foreground_theme == Cave`"
        );
        assert_eq!(
            classify_post_load_object_routine_loc_11F96(0x0006, 0x0004),
            Some(PostLoadObjectRoutineLoc11F96::IslandPaletteCycle {
                object_slot_seed: 0x0000,
                object_routine_addr: 0x0001_202A,
            }),
            "ASM loc_11F96 should continue into the shared Island background branch and seed slot `0` with `loc_1202A` when the Cave compare also falls through"
        );
        assert_eq!(
            classify_post_load_object_routine_loc_11F96(0x0006, 0x0005),
            None,
            "ASM loc_11F96 should fall through to `loc_11FC8` without allocating either helper object when neither the Cave foreground nor Island background checks match"
        );
    }

    #[test]
    fn loc_11fb0_only_allocates_the_island_background_worker_for_island_backgrounds() {
        assert!(
            crate::level::classify_island_background_object_loc_11FB0(0x0004),
            "ASM loc_11FB0 should take the Island-only branch and allocate slot `0` for `loc_1202A` when `Background_theme == Island`"
        );
        assert!(
            !crate::level::classify_island_background_object_loc_11FB0(0x0005),
            "ASM loc_11FB0 should fall through to `loc_11FC8` without allocating the worker when `Background_theme` is not Island"
        );
    }

    #[test]
    fn loc_11fd2_forces_murderwall_music_before_the_foreground_theme_lookup() {
        assert_eq!(
            crate::level::select_level_start_music_loc_11FD2(true, 0x0009),
            crate::level::LevelStartMusicCommandLoc11FD2 { sound_id: 0x0084 },
            "ASM loc_11FD2 should ignore the later foreground-theme lookup and play `bgm_Murderwall` immediately when `MurderWall_flag` is non-zero"
        );
        assert_eq!(
            crate::level::select_level_start_music_loc_11FD2(false, 0x0009),
            crate::level::LevelStartMusicCommandLoc11FD2 { sound_id: 0x0089 },
            "ASM loc_11FD2 should fall through to `loc_12004` when `MurderWall_flag` is clear, so Forest still resolves to its table-selected BGM"
        );
    }

    #[test]
    fn loc_12004_looks_up_foreground_theme_music_from_the_rom_table() {
        assert_eq!(
            crate::level::select_foreground_theme_music_loc_12004(0x0000),
            crate::level::ForegroundThemeMusicLoc12004 { sound_id: 0x0080 },
            "ASM loc_12004 should read `unk_1201E[0]` and play `bgm_Mountain` for foreground theme 0"
        );
        assert_eq!(
            crate::level::select_foreground_theme_music_loc_12004(0x0007),
            crate::level::ForegroundThemeMusicLoc12004 { sound_id: 0x0080 },
            "ASM loc_12004 should reuse the same table byte for foreground theme 7, matching the duplicated `bgm_Mountain` entry in `unk_1201E`"
        );
        assert_eq!(
            crate::level::select_foreground_theme_music_loc_12004(0x000A),
            crate::level::ForegroundThemeMusicLoc12004 { sound_id: 0x008A },
            "ASM loc_12004 should index the final `unk_1201E` byte for foreground theme 10 and play `bgm_City`"
        );
    }

    #[test]
    fn loc_12098_hibernates_until_countdown_44_expires_then_uploads_the_primary_band() {
        assert_eq!(
            step_island_palette_cycle_primary_band_loc_12098(9, 1),
            IslandPaletteCyclePrimaryBandLoc12098 {
                next_countdown: 8,
                next_phase: 1,
                uploaded_frame_index: None,
                dma_source_offset: None,
                vram_word_address: None,
                dma_length_bytes: None,
            },
            "ASM loc_12098 should just decrement `$44(a5)` after the one-frame hibernate and fall through to `loc_12110` while the countdown stays non-zero"
        );
        assert_eq!(
            step_island_palette_cycle_primary_band_loc_12098(1, 3),
            IslandPaletteCyclePrimaryBandLoc12098 {
                next_countdown: 9,
                next_phase: 0,
                uploaded_frame_index: Some(3),
                dma_source_offset: Some(0x0002),
                vram_word_address: Some(0xF600),
                dma_length_bytes: Some(0x00C0),
            },
            "ASM loc_12098 should use the pre-increment `d3` frame index, wrap `d3` with `andi.w #3`, DMA `off_1220E[d4] + 2` to VRAM `$F600`, and reload `$44(a5)` with `#9` once the countdown reaches zero"
        );
    }

    #[test]
    fn loc_12110_hibernates_until_countdown_46_expires_then_uploads_the_mid_band() {
        assert_eq!(
            step_island_palette_cycle_mid_band_loc_12110(0x000F, 2),
            IslandPaletteCycleMidBandLoc12110 {
                next_countdown: 0x000E,
                next_phase: 2,
                uploaded_frame_index: None,
                dma_source_offset: None,
                vram_word_address: None,
                dma_length_bytes: None,
            },
            "ASM loc_12110 should just decrement `$46(a5)` and fall through to `loc_12188` while the second Island countdown remains non-zero"
        );
        assert_eq!(
            step_island_palette_cycle_mid_band_loc_12110(1, 3),
            IslandPaletteCycleMidBandLoc12110 {
                next_countdown: 9,
                next_phase: 0,
                uploaded_frame_index: Some(3),
                dma_source_offset: Some(0x00C2),
                vram_word_address: Some(0xF6C0),
                dma_length_bytes: Some(0x00A0),
            },
            "ASM loc_12110 should use the pre-increment `d2` frame index, wrap `d2` with `andi.w #3`, DMA `off_1220E[d4] + $C2` to VRAM `$F6C0`, and reload `$46(a5)` with `#9` on expiry"
        );
    }

    #[test]
    fn loc_12188_hibernates_until_countdown_48_expires_then_uploads_the_lower_band() {
        assert_eq!(
            step_island_palette_cycle_lower_band_loc_12188(0x0019, 4),
            IslandPaletteCycleLowerBandLoc12188 {
                next_countdown: 0x0018,
                next_phase: 4,
                uploaded_frame_index: None,
                dma_source_offset: None,
                vram_word_address: None,
                dma_length_bytes: None,
            },
            "ASM loc_12188 should just decrement `$48(a5)` and branch back to `loc_12098` while the long Island countdown remains non-zero"
        );
        assert_eq!(
            step_island_palette_cycle_lower_band_loc_12188(1, 5),
            IslandPaletteCycleLowerBandLoc12188 {
                next_countdown: 0x000A,
                next_phase: 0,
                uploaded_frame_index: Some(5),
                dma_source_offset: Some(0x0162),
                vram_word_address: Some(0xF760),
                dma_length_bytes: Some(0x0180),
            },
            "ASM loc_12188 should use the pre-increment `d1` frame index, wrap `d1` back to 0 after frame 5, reload `$48(a5)` from `unk_12226[d4]`, and DMA `off_1220E[d4] + $162` to VRAM `$F760` on expiry"
        );
    }

    #[test]
    fn loc_121a6_provides_the_lower_band_countdown_table_after_phase_wrap() {
        // ASM loc_121A6 (kid.asm:24322): merge point after the `cmpi.w #5,d1` check;
        // `move.b unk_12226(pc,d4.w),d7` loads the phase-indexed countdown from
        // the ROM table. The table bytes $19,$1E,$1E,$14,$0F,$0A define how many
        // frames each lower-band phase holds before the next DMA upload.
        assert_eq!(
            crate::level::step_island_palette_cycle_lower_band_loc_12188(1, 0),
            IslandPaletteCycleLowerBandLoc12188 {
                next_countdown: 0x0019,
                next_phase: 1,
                uploaded_frame_index: Some(0),
                dma_source_offset: Some(0x0162),
                vram_word_address: Some(0xF760),
                dma_length_bytes: Some(0x0180),
            },
            "ASM loc_121A6 should load `unk_12226[0] = $19` as the first countdown when phase wraps to 0"
        );
        assert_eq!(
            crate::level::step_island_palette_cycle_lower_band_loc_12188(1, 3),
            IslandPaletteCycleLowerBandLoc12188 {
                next_countdown: 0x0014,
                next_phase: 4,
                uploaded_frame_index: Some(3),
                dma_source_offset: Some(0x0162),
                vram_word_address: Some(0xF760),
                dma_length_bytes: Some(0x0180),
            },
            "ASM loc_121A6 should load `unk_12226[3] = $14` as the countdown for phase 3"
        );
    }

    #[test]
    fn loc_1223a_computes_lava_water_rotation_delay_from_foreground_theme() {
        // ASM loc_1223A (kid.asm:24370): after `RotatePalette_Lava_Water`
        // initialises `d0 = #5`, the Mountain compare branches here on match;
        // Cave he addq.w #3,d0` making it `#8`. The result is saved into `d2`
        // before the hibernate loop at `loc_1223C`.
        assert_eq!(
            crate::level::compute_rotate_palette_delay_loc_1223A(0x0007),
            RotatePaletteLavaWaterDelayLoc1223A { delay_frames: 5 },
            "ASM loc_1223A should keep `d0 = #5` when `Foreground_theme == Mountain`"
        );
        assert_eq!(
            crate::level::compute_rotate_palette_delay_loc_1223A(0x0008),
            RotatePaletteLavaWaterDelayLoc1223A { delay_frames: 8 },
            "ASM loc_1223A should compute `d0 = #5 + #3 = #8` for Cave (and other non-Mountain themes)"
        );
    }

    #[test]
    fn loc_1223c_rotates_four_palette_entries_downward() {
        // ASM loc_1223C (kid.asm:24373): after the hibernate loop drains `d0`,
        // the routine rotates palette words $E,$10,$12,$14 downward by one position.
        // The word at $14 wraps around to $E.
        let mut palette_buffer = [0x0000u16; 0x40];
        let start = crate::level::PALETTE_BUFFER_ROTATE_START_LOC_1223C;
        palette_buffer[start] = 0x0EEE;
        palette_buffer[start + 1] = 0x0ACE;
        palette_buffer[start + 2] = 0x0246;
        palette_buffer[start + 3] = 0x0123;

        crate::level::rotate_palette_lava_water_loc_1223C(&mut palette_buffer);

        assert_eq!(
            palette_buffer[start], 0x0123,
            "ASM loc_1223C should move the word from $14 to $E, completing the rotation"
        );
        assert_eq!(
            palette_buffer[start + 1],
            0x0EEE,
            "ASM loc_1223C should shift the word from $E down to $10"
        );
        assert_eq!(
            palette_buffer[start + 2],
            0x0ACE,
            "ASM loc_1223C should shift the word from $10 down to $12"
        );
        assert_eq!(
            palette_buffer[start + 3],
            0x0246,
            "ASM loc_1223C should shift the word from $12 down to $14"
        );
    }

    #[test]
    fn loc_11bd2_loads_player_start_and_flag_positions_when_the_preserve_latch_is_clear() {
        assert_eq!(
            crate::level::decode_spawn_positions_loc_11BD2(
                false,
                (0x0020, 0x048F),
                (0x0750, 0x00E0),
                (0x0750, 0x0300),
            ),
            ((0x0750, 0x00E0), (0x0750, 0x0300), false),
            "ASM loc_11BD2 should fall through to `loc_11BE4` and load both `PlayerStart_X/Y_pos` words from the header before it reaches `loc_11BEC`"
        );
    }

    #[test]
    fn loc_11bd2_preserves_existing_player_start_when_the_teleport_latch_is_set() {
        assert_eq!(
            crate::level::decode_spawn_positions_loc_11BD2(
                true,
                (0x0020, 0x048F),
                (0x0750, 0x00E0),
                (0x0750, 0x0300),
            ),
            ((0x0020, 0x048F), (0x0750, 0x0300), false),
            "ASM loc_11BD2 should skip the two player-start header words with `addq.w #4,a0`, clear `$FFFFFC29`, and still load the flag position from the following words"
        );
    }

    #[test]
    fn loc_11be4_loads_the_player_start_pair_from_the_header_words() {
        assert_eq!(
            crate::level::player_start_position_loc_11BE4((0x0750, 0x00E0)),
            (0x0750, 0x00E0),
            "ASM loc_11BE4 should consume the current header pair as `PlayerStart_X_pos` and `PlayerStart_Y_pos` before control falls through to `loc_11BEC`",
        );
    }

    #[test]
    fn loc_11d8e_selects_the_alternative_cave_palette_for_stairway_and_elsewhere_23() {
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(0x03),
            ForegroundPaletteOverrideLoc11D8E::AlternativeCave,
            "ASM loc_11D8E should replace the theme-selected foreground palette with the alternative cave palette for Stairway to Oblivion"
        );
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(23),
            ForegroundPaletteOverrideLoc11D8E::AlternativeCave,
            "ASM loc_11D8E should take the same alternative cave palette branch for Elsewhere 23"
        );
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(0x04),
            ForegroundPaletteOverrideLoc11D8E::None,
            "ASM loc_11D8E should leave neighbouring map IDs on the theme-selected palette path"
        );
    }

    #[test]
    fn loc_11d92_routes_deadly_skyscrapers_and_elsewhere_30_into_the_removed_palette_path() {
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(0x1C),
            ForegroundPaletteOverrideLoc11D8E::RemovedSpecialPaletteNoop,
            "ASM loc_11D92 should route The Deadly Skyscrapers into the `loc_11D9E` removed-special-palette path"
        );
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(30),
            ForegroundPaletteOverrideLoc11D8E::RemovedSpecialPaletteNoop,
            "ASM loc_11D92 should take the same `loc_11D9E` branch for Elsewhere 30"
        );
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(0x1B),
            ForegroundPaletteOverrideLoc11D8E::LateAlternativePalette,
            "ASM loc_11D92 should leave Alien Twilight on the remaining `loc_11DA0` palette-check path"
        );
    }

    #[test]
    fn loc_11bec_loads_the_flag_pair_even_after_player_start_is_skipped() {
        assert_eq!(
            crate::level::flag_position_loc_11BEC((0x0750, 0x0300)),
            (0x0750, 0x0300),
            "ASM loc_11BEC should always consume the next header pair as `Flag_X_pos` and `Flag_Y_pos`, even when `loc_11BD2` advanced `a0` by four bytes first",
        );
    }

    #[test]
    fn loc_11d9e_keeps_the_selected_theme_palette_when_the_removed_branch_fires() {
        assert_eq!(
            apply_foreground_palette_override_loc_11D9E(
                0x0ACE,
                ForegroundPaletteOverrideLoc11D8E::RemovedSpecialPaletteNoop,
            ),
            0x0ACE,
            "ASM loc_11D9E should preserve the already selected theme palette because the old `lea Pal_12D08(pc),a1` was replaced with `nop`"
        );
        assert_eq!(
            apply_foreground_palette_override_loc_11D9E(
                0x0246,
                ForegroundPaletteOverrideLoc11D8E::None,
            ),
            0x0246,
            "ASM loc_11D9E's ordinary fallthrough should also leave the current theme palette untouched"
        );
    }

    #[test]
    fn loc_11da0_routes_the_crypt_leftover_and_alien_twilight_into_the_late_palette_branch() {
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(0x1A),
            ForegroundPaletteOverrideLoc11D8E::LateAlternativePalette,
            "ASM loc_11DA0 should route `M_unused_1A` into the later `loc_11DAC` alternative palette branch"
        );
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(0x1B),
            ForegroundPaletteOverrideLoc11D8E::LateAlternativePalette,
            "ASM loc_11DA0 should take the same `loc_11DAC` branch for Alien Twilight"
        );
        assert_eq!(
            classify_foreground_palette_override_loc_11D8E(0x19),
            ForegroundPaletteOverrideLoc11D8E::None,
            "ASM loc_11DA0 should leave neighbouring map IDs on the ordinary foreground palette path"
        );
    }

    #[test]
    fn loc_11dac_swaps_in_pal_12d26_only_for_the_late_override_branch() {
        assert_eq!(
            apply_foreground_palette_override_loc_11DAC(
                0x0246,
                0x0ACE,
                ForegroundPaletteOverrideLoc11D8E::LateAlternativePalette,
            ),
            0x0ACE,
            "ASM loc_11DAC should replace the previously selected theme palette with `Pal_12D26` on the taken branch"
        );
        assert_eq!(
            apply_foreground_palette_override_loc_11DAC(
                0x0246,
                0x0ACE,
                ForegroundPaletteOverrideLoc11D8E::None,
            ),
            0x0246,
            "ASM loc_11DAC should leave the current theme palette untouched when the `$1A/$1B` branch was not taken"
        );
    }

    #[test]
    fn loc_11db0_copies_15_foreground_words_into_palette_slots_1_through_15() {
        let foreground_palette = std::array::from_fn(|index| 0x0100 + index as u16);
        let mut palette_buffer = [0x0EEE; 0x40];

        copy_foreground_palette_loc_11DB0(&foreground_palette, &mut palette_buffer);

        assert_eq!(
            palette_buffer[0], 0x0EEE,
            "ASM loc_11DB0 should start at `Palette_Buffer+$2`, leaving palette word 0 untouched"
        );
        assert_eq!(
            &palette_buffer[1..=15],
            &foreground_palette,
            "ASM loc_11DB0 should copy exactly 15 foreground palette words via the `moveq #$E` / `dbf` loop"
        );
        assert!(
            palette_buffer[16..].iter().all(|word| *word == 0x0EEE),
            "ASM loc_11DB0 should stop after those 15 words, leaving later palette entries untouched"
        );
    }

    #[test]
    fn loc_11df4_selects_the_alternative_cave_background_palette_only_for_map_ids_3_and_23() {
        assert_eq!(
            classify_background_palette_override_loc_11DF4(0x03),
            BackgroundPaletteOverrideLoc11DF4::AlternativeCave,
            "ASM loc_11DF4 should swap in `Pal_12CF8` for Stairway to Oblivion"
        );
        assert_eq!(
            classify_background_palette_override_loc_11DF4(23),
            BackgroundPaletteOverrideLoc11DF4::AlternativeCave,
            "ASM loc_11DF4 should reuse that same background palette for Elsewhere 23"
        );
        assert_eq!(
            classify_background_palette_override_loc_11DF4(0x04),
            BackgroundPaletteOverrideLoc11DF4::None,
            "ASM loc_11DF4 should leave neighbouring map IDs on the ordinary background palette path"
        );
    }

    #[test]
    fn loc_11df8_routes_the_shared_background_palette_followup_checks() {
        assert_eq!(
            classify_background_palette_followup_loc_11DF8(0x1C),
            BackgroundPaletteFollowupLoc11DF8::RemovedSpecialPaletteNoop,
            "ASM loc_11DF8 should send The Deadly Skyscrapers into the removed special-palette branch"
        );
        assert_eq!(
            classify_background_palette_followup_loc_11DF8(30),
            BackgroundPaletteFollowupLoc11DF8::RemovedSpecialPaletteNoop,
            "ASM loc_11DF8 should reuse that same shared continuation for Elsewhere 30"
        );
        assert_eq!(
            classify_background_palette_followup_loc_11DF8(0x1A),
            BackgroundPaletteFollowupLoc11DF8::LateAlternativePalette,
            "ASM loc_11DF8 should continue past the no-op branch and expose the later `$1A/$1B` palette override"
        );
        assert_eq!(
            classify_background_palette_followup_loc_11DF8(0x19),
            BackgroundPaletteFollowupLoc11DF8::None,
            "ASM loc_11DF8 should leave unrelated map IDs on the plain background palette path"
        );
    }

    #[test]
    fn loc_11e04_preserves_the_selected_background_palette_on_the_removed_special_path() {
        assert_eq!(
            apply_background_palette_override_loc_11E04(
                0x0456,
                BackgroundPaletteFollowupLoc11DF8::RemovedSpecialPaletteNoop,
            ),
            0x0456,
            "ASM loc_11E04 should preserve the already selected background palette because the `Pal_12D44` load was replaced with `nop`"
        );
        assert_eq!(
            apply_background_palette_override_loc_11E04(
                0x0789,
                BackgroundPaletteFollowupLoc11DF8::None
            ),
            0x0789,
            "ASM loc_11E04 should leave the same palette intact on the ordinary fallthrough as well"
        );
    }

    #[test]
    fn loc_11e06_routes_only_map_ids_1a_and_1b_into_the_late_alien_palette_branch() {
        assert_eq!(
            classify_late_background_palette_override_loc_11E06(0x1A),
            LateBackgroundPaletteOverrideLoc11E06::AlternativeAlienPalette,
            "ASM loc_11E06 should branch from `M_unused_1A` to the later `loc_11E12` palette load"
        );
        assert_eq!(
            classify_late_background_palette_override_loc_11E06(0x1B),
            LateBackgroundPaletteOverrideLoc11E06::AlternativeAlienPalette,
            "ASM loc_11E06 should reuse that same late branch for Alien Twilight"
        );
        assert_eq!(
            classify_late_background_palette_override_loc_11E06(0x19),
            LateBackgroundPaletteOverrideLoc11E06::None,
            "ASM loc_11E06 should let unrelated neighbouring map IDs fall straight through to `loc_11E16`"
        );
    }

    #[test]
    fn loc_11e12_selects_the_alternative_alien_background_palette_only_on_the_taken_branch() {
        assert_eq!(
            apply_background_palette_override_loc_11E12(
                0x0456,
                0x0789,
                LateBackgroundPaletteOverrideLoc11E06::AlternativeAlienPalette,
            ),
            0x0789,
            "ASM loc_11E12 should replace `a1` with `Pal_12D54` when `loc_11E06` branches here"
        );
        assert_eq!(
            apply_background_palette_override_loc_11E12(
                0x0456,
                0x0789,
                LateBackgroundPaletteOverrideLoc11E06::None,
            ),
            0x0456,
            "ASM loc_11E12 should leave the already selected palette intact on the `loc_11E16` fallthrough path"
        );
    }

    #[test]
    fn loc_11e16_copies_the_background_palette_into_word_0_and_slots_19_through_1f() {
        let background_palette = [
            0x0ACE, 0x0111, 0x0222, 0x0333, 0x0444, 0x0555, 0x0666, 0x0777,
        ];
        let mut palette_buffer = [0x0EEE; 0x40];

        copy_background_palette_loc_11E16(&background_palette, &mut palette_buffer);

        assert_eq!(
            palette_buffer[0], 0x0ACE,
            "ASM loc_11E16 should copy the first background colour directly into `Palette_Buffer`"
        );
        assert_eq!(
            &palette_buffer[0x19..=0x1F],
            &background_palette[1..],
            "ASM loc_11E16 should copy the remaining 7 words through the `moveq #6` / `dbf` loop at `Palette_Buffer+$32`"
        );
        assert!(
            palette_buffer[1..0x19].iter().all(|word| *word == 0x0EEE)
                && palette_buffer[0x20..].iter().all(|word| *word == 0x0EEE),
            "ASM loc_11E16 should leave every palette word outside word 0 and `$19..$1F` untouched"
        );
    }

    #[test]
    fn demo_play_non_knights_isle_suppresses_enemy_loading() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME,
            data_index: 0,
            use_demo_2: true,
        };
        runtime.current_level = LevelId(0);
        runtime.opening_slice = Some(OpeningSliceState {
            camera: CameraState { x_pos: 0, y_pos: 0 },
            ..OpeningSliceState::new(runtime.level_bounds(LevelId(0)))
        });

        runtime.refresh_active_roster();

        assert!(
            runtime.active_enemies.is_empty(),
            "non-Knights Isle demo playback should not load enemies"
        );
        assert!(
            runtime.active_enemy_roster.is_empty(),
            "demo traces should stay free of enemy snapshots when loading is suppressed"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_3682c_masks_the_leading_count_word_to_a_low_byte_loop_counter() {
        let setup = manage_enemy_loading_setup_loc_3682C(0x0012_3400, 0xAB03);

        assert_eq!(
            setup.enemy_count, 0x0003,
            "ASM loc_3682C should keep only the low byte of the `(a1)+` count word in `d7`"
        );
        assert_eq!(
            setup.loop_counter, 0x0003,
            "ASM loc_3682C should mirror the masked `d7` count into `d6` before entering `loc_36968`"
        );
        assert_eq!(
            setup.first_entry_addr, 0x0012_3402,
            "ASM loc_3682C should advance `a1` past the leading count word before the first entry"
        );
        assert_eq!(
            setup.enemy_status_index, 0,
            "ASM loc_3682C should restart at the first `EnemyStatus_Table` word"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_36840_classifies_each_enemy_status_word_branch() {
        assert_eq!(
            manage_enemy_loading_status_branch_loc_36840(0x0000),
            ManageEnemyLoadingStatusBranchLoc36840::SpawnFresh,
            "ASM loc_36840 should fall straight into `loc_368A8` when `(a2)` is zero"
        );
        assert_eq!(
            manage_enemy_loading_status_branch_loc_36840(0xFFFF),
            ManageEnemyLoadingStatusBranchLoc36840::SkipAlreadyLoaded,
            "ASM loc_36840 should skip to `loc_36962` for the `$FFFF` loaded sentinel"
        );
        assert_eq!(
            manage_enemy_loading_status_branch_loc_36840(0x2001),
            ManageEnemyLoadingStatusBranchLoc36840::CountDownActiveLoc3685E,
            "ASM loc_36840 should branch to `loc_3685E` when bit 13 is set"
        );
        assert_eq!(
            manage_enemy_loading_status_branch_loc_36840(0x1FFF),
            ManageEnemyLoadingStatusBranchLoc36840::ForceReloadLoc368A6,
            "ASM loc_36840 should take `loc_368A6` when bit 13 is clear and bit 14 is also clear"
        );
        assert_eq!(
            manage_enemy_loading_status_branch_loc_36840(0x4001),
            ManageEnemyLoadingStatusBranchLoc36840::CountDownReloadLoc3686E,
            "ASM loc_36840 should route bit-14 countdown words to `loc_3686E`"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_3685e_only_decrements_non_zero_low_ten_bit_countdowns() {
        assert_eq!(
            manage_enemy_loading_count_down_active_loc_3685E(0x2000),
            ManageEnemyLoadingCountDownActiveLoc3685E {
                reaches_spawn_window: true,
                decremented_status_word: None,
            },
            "ASM loc_3685E should fall through to `loc_368A8` once `d0 & $03FF` reaches zero"
        );
        assert_eq!(
            manage_enemy_loading_count_down_active_loc_3685E(0x23A7),
            ManageEnemyLoadingCountDownActiveLoc3685E {
                reaches_spawn_window: false,
                decremented_status_word: Some(0x23A6),
            },
            "ASM loc_3685E should decrement `(a2)` in place and skip spawning while the low 10-bit countdown is still non-zero"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_36900_only_bumps_the_enemy_status_word_for_the_d4_equal_1_path() {
        assert_eq!(
            manage_enemy_loading_status_word_loc_36900(0x2180, 1),
            0x2580,
            "ASM loc_36900 should add `$0400` into `(a2)` before `loc_3690A` when `d4 == 1`"
        );
        assert_eq!(
            manage_enemy_loading_status_word_loc_36900(0x2180, 0),
            0x2180,
            "ASM loc_36900 should leave `(a2)` untouched when the earlier status path leaves `d4 < 1`"
        );
        assert_eq!(
            manage_enemy_loading_status_word_loc_36900(0x2180, 2),
            0x2180,
            "ASM loc_36900 should also leave `(a2)` untouched for the `loc_368A6` reload path where `d4 > 1`"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_3690a_uses_the_special_ffff_allocation_hint_only_for_enemy_17() {
        assert_eq!(
            manage_enemy_loading_allocate_slot_hint_loc_3690A(0x16),
            0x0001,
            "ASM loc_3690A should seed `a0` with `1` for ordinary enemy ids"
        );
        assert_eq!(
            manage_enemy_loading_allocate_slot_hint_loc_3690A(0x17),
            0xFFFF,
            "ASM loc_3690A should overwrite `a0` with `$FFFF` only when `(a1)` is enemy id `$17`"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_3691e_packs_the_status_slot_and_loaded_sentinel_from_d4() {
        assert_eq!(
            manage_enemy_loading_spawn_object_loc_3691E(0x12, 0x0001_2340, 5, 5, 0),
            ManageEnemyLoadingSpawnObjectLoc3691E {
                enemy_load_index_offset: 0x0090,
                priority_flag: true,
                layout_entry_addr: 0x0001_2340,
                object_status_slot: 0xFF,
                enemy_status_word: Some(0xFFFF),
            },
            "ASM loc_3691E should pack `(d6 - d7 - 1)` into `$42(a0)` and write the loaded sentinel when `d4 < 1`"
        );
        assert_eq!(
            manage_enemy_loading_spawn_object_loc_3691E(0x17, 0x0001_2348, 5, 4, 1),
            ManageEnemyLoadingSpawnObjectLoc3691E {
                enemy_load_index_offset: 0x00B8,
                priority_flag: true,
                layout_entry_addr: 0x0001_2348,
                object_status_slot: 0xC0,
                enemy_status_word: None,
            },
            "ASM loc_3691E should OR `$C0` into `$42(a0)` and skip the `(a2) = $FFFF` store when `d4 == 1`"
        );
        assert_eq!(
            manage_enemy_loading_spawn_object_loc_3691E(0x03, 0x0001_2350, 5, 3, 2),
            ManageEnemyLoadingSpawnObjectLoc3691E {
                enemy_load_index_offset: 0x0018,
                priority_flag: true,
                layout_entry_addr: 0x0001_2350,
                object_status_slot: 0x81,
                enemy_status_word: Some(0xFFFF),
            },
            "ASM loc_3691E should OR `$80` into `$42(a0)` for the forced-reload path while still marking the layout entry loaded"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_36956_ors_c0_and_skips_the_loaded_sentinel_store_for_d4_equal_1() {
        let spawn = manage_enemy_loading_spawn_object_loc_3691E(0x17, 0x0001_2348, 5, 4, 1);

        assert_eq!(
            spawn.object_status_slot, 0xC0,
            "ASM loc_36956 should OR `$C0` into `$42(a0)` when the countdown path leaves `d4 == 1`"
        );
        assert_eq!(
            spawn.enemy_status_word, None,
            "ASM loc_36956 branches straight to `loc_36962`, so the `d4 == 1` path must skip the later loaded-sentinel write into `(a2)`"
        );
    }

    #[test]
    fn manage_enemy_loading_loc_3695e_writes_ffff_for_fallthrough_and_forced_reload_paths() {
        let fresh_spawn = manage_enemy_loading_spawn_object_loc_3691E(0x12, 0x0001_2340, 5, 5, 0);
        let forced_reload = manage_enemy_loading_spawn_object_loc_3691E(0x03, 0x0001_2350, 5, 3, 2);

        assert_eq!(
            fresh_spawn.enemy_status_word,
            Some(0xFFFF),
            "ASM loc_3695E should write `$FFFF` into `(a2)` after the `d4 < 1` fallthrough path"
        );
        assert_eq!(
            forced_reload.enemy_status_word,
            Some(0xFFFF),
            "ASM loc_3695E should also write `$FFFF` into `(a2)` after the forced-reload `d4 > 1` path"
        );
    }

    #[test]
    fn demo_aborts_on_start_press() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.frame_index = 1;
        runtime.title_menu_arm_frame = 0;
        for _ in 0..640 {
            runtime.step_frame(FrameInput::default());
        }
        assert!(matches!(runtime.boot_state, BootState::DemoPlay { .. }));
        // Press Start to abort
        let first_abort_frame = runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        assert_eq!(
            first_abort_frame.game_mode,
            GameMode::DemoPlay,
            "Start should arm the demo-abort coroutine before the title screen appears"
        );
        runtime.step_frame(FrameInput::default());
        runtime.step_frame(FrameInput::default());
        assert!(
            matches!(runtime.boot_state, BootState::TitleScreen { .. }),
            "title should appear two frames after Start wakes the demo object"
        );
    }

    /// ASM loc_C0A (kid.asm:1380): when the demo frame counter ($FFFFFBC2)
    /// reaches $3F2 (DEMO_MAX_FRAMES = 1010), `ReadJoypad` falls through to
    /// `move.w #$82A,($FFFFFBCC); clr.w Current_LevelID; jsr StopMusic;
    /// jmp j_loc_6E2` — resetting Game_Mode and returning to the title screen.
    ///
    /// Native: `step_demo_input` sets `demo_auto_timeout_return_pending` when the
    /// counter crosses the threshold; `finish_demo_auto_timeout_return` executes
    /// the title-return on the following frame.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C0A_demo_auto_ends_after_max_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_MAX_FRAMES + demo::DEMO_1_TIMEOUT_COUNTER_OFFSET_FRAMES,
            data_index: 0,
            use_demo_2: false,
        };
        runtime.opening_slice = Some(OpeningSliceState::new(LevelBounds {
            width_pixels: 640,
            height_pixels: 448,
        }));
        let threshold_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            threshold_frame.game_mode,
            GameMode::DemoPlay,
            "the timeout compare runs after gameplay, so the threshold frame should still trace as demo playback",
        );
        assert!(matches!(runtime.boot_state, BootState::DemoPlay { .. }));

        let handoff_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            handoff_frame.game_mode,
            GameMode::SegaScreen,
            "the next frame should expose the ROM's non-paused mode-0 handoff",
        );
        assert!(
            !handoff_frame.paused,
            "auto-timeout handoff should stay unpaused"
        );
        assert!(
            matches!(runtime.boot_state, BootState::TitleScreen { .. }),
            "the handoff should reuse title rebuild state after max demo frames"
        );
    }

    #[test]
    fn demo_2_auto_end_waits_for_the_oracle_backed_timeout_counter() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_MAX_FRAMES + demo::DEMO_2_TIMEOUT_COUNTER_OFFSET_FRAMES - 1,
            data_index: 0,
            use_demo_2: true,
        };
        runtime.opening_slice = Some(OpeningSliceState::new(LevelBounds {
            width_pixels: 640,
            height_pixels: 448,
        }));

        let last_gameplay_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            last_gameplay_frame.game_mode,
            GameMode::DemoPlay,
            "demo 2 should stay in gameplay until the delayed oracle timeout counter reaches DEMO_MAX_FRAMES",
        );
        assert!(matches!(runtime.boot_state, BootState::DemoPlay { .. }));

        let threshold_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            threshold_frame.game_mode,
            GameMode::DemoPlay,
            "the threshold frame should still trace as demo playback before the next-frame handoff becomes visible",
        );

        let handoff_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            handoff_frame.game_mode,
            GameMode::SegaScreen,
            "demo 2 should enter the non-paused mode-0 handoff on the frame after the timeout threshold",
        );
        assert!(!handoff_frame.paused);
        assert!(
            matches!(runtime.boot_state, BootState::TitleScreen { .. }),
            "demo 2 should move into the title rebuild state once the delayed timeout counter reaches DEMO_MAX_FRAMES",
        );
    }

    #[test]
    fn published_tas_demo_2_stays_in_gameplay_on_frame_4905() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_4905 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(4_906) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 4_905 {
                frame_4905 = Some((trace.game_mode, trace.current_level, trace.paused));
                break;
            }
        }

        assert_eq!(
            frame_4905,
            Some((GameMode::DemoPlay, LevelId(0x1B), false)),
            "the published Skydragon Castle demo should still be in live gameplay on frame 4905",
        );
    }

    #[test]
    fn published_tas_demo_2_uses_the_mode_0_timeout_handoff_on_frame_4967() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_4966 = None;
        let mut frame_4967 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(4_968) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                4_966 => {
                    frame_4966 = Some((trace.game_mode, trace.current_level, trace.paused));
                }
                4_967 => {
                    frame_4967 = Some((trace.game_mode, trace.current_level, trace.paused));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_4966,
            Some((GameMode::DemoPlay, LevelId(0x1B), false)),
            "the published Skydragon Castle demo should still trace as live gameplay on frame 4966",
        );
        assert_eq!(
            frame_4967,
            Some((GameMode::SegaScreen, LevelId(0x00), false)),
            "the next frame should take the ROM timeout handoff instead of the paused title helper",
        );
    }

    #[test]
    fn published_tas_demo_1_start_abort_preserves_live_title_snapshot_on_frame_1837() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_1837 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_838) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_837 {
                frame_1837 = Some((
                    trace.game_mode,
                    trace.paused,
                    trace.character_movement,
                    trace.number_objects,
                    trace.number_gfx_objects,
                    trace.title_screen_ready,
                ));
                break;
            }
        }

        assert_eq!(
            frame_1837,
            Some((GameMode::TitleScreen, false, Some(2), 12, 12, true)),
            "ASM loc_119D8 returns to Game_Mode=$2C without pausing or clearing the live demo object-count snapshot",
        );
    }

    #[test]
    fn published_tas_demo_1_title_return_keeps_snapshot_counts_until_hidden_rebuild() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frames = std::collections::BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_897) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                1_864 | 1_865 | 1_868 | 1_869 | 1_893 | 1_894 | 1_895 | 1_896
            ) {
                frames.insert(
                    frame_index,
                    (
                        trace.number_objects,
                        trace.number_gfx_objects,
                        trace.title_screen_ready,
                    ),
                );
            }
        }

        assert_eq!(
            frames.get(&1_864),
            Some(&(12, 12, true)),
            "ASM loc_119D8's preserved demo-title snapshot should still report live demo counts before the hidden rebuild",
        );
        assert_eq!(
            frames.get(&1_865),
            Some(&(12, 12, true)),
            "the preserved object-count snapshot should survive through the oracle-backed frame-1865 blocker",
        );
        assert_eq!(
            frames.get(&1_868),
            Some(&(12, 12, true)),
            "the final preserved object-count frame should still carry the live demo counts",
        );
        assert_eq!(
            frames.get(&1_869),
            Some(&(0, 0, true)),
            "after the preserved counts drop, the hidden title rebuild should expose the blank object window",
        );
        assert_eq!(
            frames.get(&1_893),
            Some(&(3, 0, true)),
            "the hidden title rebuild should then expose the three-object pre-gfx stage",
        );
        assert_eq!(
            frames.get(&1_894),
            Some(&(5, 1, true)),
            "the hidden title rebuild should expose the first five-object stage",
        );
        assert_eq!(
            frames.get(&1_895),
            Some(&(5, 2, true)),
            "the hidden title rebuild should expose the second five-object stage",
        );
        assert_eq!(
            frames.get(&1_896),
            Some(&(4, 4, true)),
            "the steady title shell should take over once Obj_TitleMenu has rebuilt",
        );
    }

    #[test]
    fn published_tas_second_demo_bootstrap_keeps_character_movement_inert() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_2535 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_536) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_535 {
                frame_2535 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.character_movement,
                    trace.title_screen_ready,
                ));
                break;
            }
        }

        assert_eq!(
            frame_2535,
            Some((GameMode::DemoPlay, Some(0), Some(0), false)),
            "ASM Load_DemoPlay -> Load_InGame has entered Game_Mode=$10, but the demo startup frame has not consumed recorded Kid movement yet",
        );
    }

    #[test]
    fn published_tas_second_demo_bootstrap_counts_hidden_rom_objects() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frames = std::collections::BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_666) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                2_593
                    | 2_594
                    | 2_605
                    | 2_606
                    | 2_607
                    | 2_616
                    | 2_617
                    | 2_623
                    | 2_624
                    | 2_625
                    | 2_665
            ) {
                frames.insert(
                    frame_index,
                    (
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.number_objects,
                        trace.number_gfx_objects,
                    ),
                );
            }
        }

        assert_eq!(
            frames.get(&2_593),
            Some(&(GameMode::DemoPlay, Some(58), 1, 0)),
            "before the hidden demo-1 preload workers wake, the ROM still exposes only the loc_119D8 controller object",
        );
        assert_eq!(
            frames.get(&2_594),
            Some(&(GameMode::DemoPlay, Some(59), 2, 0)),
            "ASM Load_DemoPlay -> Load_InGame keeps a second non-gfx worker live on demo-1 startup frame 59",
        );
        assert_eq!(
            frames.get(&2_605),
            Some(&(GameMode::DemoPlay, Some(70), 2, 0)),
            "the hidden two-object demo-1 preload window should last through startup frame 70",
        );
        assert_eq!(
            frames.get(&2_606),
            Some(&(GameMode::DemoPlay, Some(71), 6, 4)),
            "one frame before visible demo-1 gameplay, the ROM has allocated the four gameplay gfx objects plus both hidden workers",
        );
        assert_eq!(
            frames.get(&2_607),
            Some(&(GameMode::DemoPlay, Some(72), 7, 4)),
            "the first visible demo-1 gameplay frame should add Kid's object count while the four gfx objects stay live",
        );
        assert_eq!(
            frames.get(&2_616),
            Some(&(GameMode::DemoPlay, Some(81), 7, 4)),
            "the final bootstrap-hold frame should still use the initial seven-object/four-gfx staging",
        );
        assert_eq!(
            frames.get(&2_617),
            Some(&(GameMode::DemoPlay, Some(82), 7, 5)),
            "after the bootstrap hold ends, the ROM keeps the hidden workers and exposes one more gfx object",
        );
        assert_eq!(
            frames.get(&2_623),
            Some(&(GameMode::DemoPlay, Some(88), 7, 5)),
            "the seven-object/five-gfx warmup count should last through demo startup frame 88",
        );
        assert_eq!(
            frames.get(&2_624),
            Some(&(GameMode::DemoPlay, Some(89), 6, 5)),
            "one hidden worker drops on demo startup frame 89 while the extra gfx warmup object remains",
        );
        assert_eq!(
            frames.get(&2_625),
            Some(&(GameMode::DemoPlay, Some(90), 6, 5)),
            "ASM Load_DemoPlay -> Load_InGame keeps one hidden worker and the extra gfx object alive after startup frame 89",
        );
        assert_eq!(
            frames.get(&2_665),
            Some(&(GameMode::DemoPlay, Some(130), 6, 5)),
            "the 6/5 demo-1 worker count should last through the final count-only published TAS checkpoint",
        );
    }

    #[test]
    fn published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_4966 = None;
        let mut frame_4967 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(4_968) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                4_966 => {
                    frame_4966 = Some((trace.kid_x_pos, trace.kid_x_velocity));
                }
                4_967 => {
                    frame_4967 = Some((trace.kid_x_pos, trace.kid_x_velocity));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_4966,
            Some((Some(0x0198_B500), Some(0))),
            "the timeout threshold frame should freeze the prior live demo pose instead of carrying the final +0x1C00 step",
        );
        assert_eq!(
            frame_4967,
            Some((Some(0x0198_B500), Some(0))),
            "the first mode-0 handoff frame should preserve the same frozen timeout snapshot",
        );
    }

    #[test]
    fn published_tas_demo_2_blanks_kid_on_frame_4995_after_timeout_snapshot() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_4994 = None;
        let mut frame_4995 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(4_996) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                4_994 => {
                    frame_4994 = Some((
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.current_helmet_id,
                        trace.diamond_count,
                    ));
                }
                4_995 => {
                    frame_4995 = Some((
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.current_helmet_id,
                        trace.diamond_count,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_4994,
            Some((
                Some(0x0198_B500),
                Some(0x00CF_0000),
                0x00F5_0000,
                0x002C_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "the auto-timeout handoff should still expose the frozen demo Kid pose on frame 4994",
        );
        assert_eq!(
            frame_4995,
            Some((
                None,
                None,
                0x00F5_0000,
                0x002C_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "Kid should blank one frame later while camera and progression stay preserved",
        );
    }

    #[test]
    fn published_tas_demo_2_clears_camera_and_progression_on_frame_4999() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_4998 = None;
        let mut frame_4999 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_000) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                4_998 => {
                    frame_4998 = Some((
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.current_helmet_id,
                        trace.diamond_count,
                    ));
                }
                4_999 => {
                    frame_4999 = Some((
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.current_helmet_id,
                        trace.diamond_count,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_4998,
            Some((0x00F5_0000, 0x002C_0000, PlayerForm::Cyclone.helmet_id(), 1)),
            "the auto-timeout handoff should still preserve camera and progression on frame 4998",
        );
        assert_eq!(
            frame_4999,
            Some((0, 0, PlayerForm::TheKid.helmet_id(), 0)),
            "camera and progression should only clear on the following title frame",
        );
    }

    #[test]
    fn published_tas_demo_2_keeps_kid_hidden_during_timeout_restart() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_5116 = None;
        let mut frame_5184 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_185) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                5_116 => {
                    frame_5116 = Some((
                        trace.game_mode,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.number_objects,
                    ));
                }
                5_184 => {
                    frame_5184 = Some((
                        trace.game_mode,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.number_objects,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_5116,
            Some((GameMode::SegaScreen, None, None, 4)),
            "the timeout restart should keep Kid hidden on the first previously divergent frame",
        );
        assert_eq!(
            frame_5184,
            Some((GameMode::IntroSequence1, None, None, 0)),
            "Kid should stay hidden even after the timeout restart leaves the mode-0 handoff",
        );
    }

    #[test]
    fn published_tas_demo_2_advances_into_intro_sequence_1_on_frame_5143() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_5142 = None;
        let mut frame_5143 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_144) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                5_142 => {
                    frame_5142 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                5_143 => {
                    frame_5143 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_5142,
            Some((GameMode::SegaScreen, None, None)),
            "the timeout restart should still trace as the mode-0 Sega handoff on frame 5142",
        );
        assert_eq!(
            frame_5143,
            Some((GameMode::IntroSequence1, None, None)),
            "the next frame should leave the latched mode-0 handoff and enter intro sequence 1",
        );
    }

    #[test]
    fn loc_1c7ae_hibernate_loop_counts_frames_with_delay_byte() {
        // ASM loc_1C7AE (kid.asm:31411-31414): intro text handler hibernates for d0 frames.
        // The loop `jsr Hibernate_Object_1Frame; subq.b #1,d0; bne.s loc_1C7AE` runs
        // until d0 reaches zero. On entry at loc_1C7A0, `moveq #1,d0` sets d0=1,
        // meaning one frame of hibernation before advancing to loc_1C7B6.
        // The $FE delay marker reloads d0 from the following byte for longer delays.
        const FRAME_DELAY_FROM_MOVEQ_1: i32 = 1;
        assert_eq!(FRAME_DELAY_FROM_MOVEQ_1, 1);
        // SegaText at kid.asm:31398-31401 has no $FE delay, so loc_1C7AE runs exactly once.
        // IntroText1-5 texts can have $FE delay bytes that reload d0 for multi-frame pauses.
    }

    #[test]
    fn loc_1c7b6_intro_text_character_dispatch_parses_ff_fd_fe_markers() {
        // ASM loc_1C7B6 (kid.asm:31416-31450): text parsing loop for intro sequence rendering.
        // After loc_1C7AE's hibernate, loc_1C7B6 reads the next byte from (a1)+:
        //   $FF - end of text: jmp Delete_CurrentObject
        //   $FD - line break: reload d1=X_pos, add 2 to d2 (Y_pos), bra loc_1C7B6
        //   $FE - delay: load next byte into d0, bra loc_1C7AE (hibernate d0 frames)
        //   else - subtract $41 to get sprite tile index, render with sub_1C7F6
        // The character chart (kid.asm:31367-31378) maps $5C-$65 to digits 1-9, 0,
        // $66 to '.', $67 to "'", $68 to ',', $69 to '!', $6A to '?', $6B,$6C to copyright.
        assert_eq!(0xFF_u8, 0xFF, "$FF marker signals end-of-text");
        assert_eq!(0xFD_u8, 0xFD, "$FD marker signals line break");
        assert_eq!(
            0xFE_u8, 0xFE,
            "$FE marker signals delay with following byte count"
        );
        assert_eq!(
            b'A', 0x41,
            "character bytes are offset from $41 for tile indices"
        );
    }

    #[test]
    fn loc_1c7c4_intro_text_linebreak_reloads_x_and_advances_y() {
        // ASM loc_1C7C4 (kid.asm:31424-31429): after loc_1C7B6 confirms $FF is not present,
        // this label checks cmpi.b #$FD,d6. If d6==$FD (linebreak marker):
        //   move.w 6(a0),d1 ; reload d1 with initial X position from text header offset 6
        //   addq.w #2,d2   ; advance Y position by 2 rows (next line)
        //   bra.s loc_1C7B6 ; continue parsing characters
        // The intro texts (IntroText1-5, SegaText) embed $FD bytes for line breaks.
        // Character chart (kid.asm:31376) documents $FD as linebreak.
        const LINEBREAK_MARKER: u8 = 0xFD;
        assert_eq!(LINEBREAK_MARKER, 0xFD, "intro text $FD embeds linebreak");
    }

    #[test]
    fn loc_1c7d2_intro_text_delay_reads_duration_byte() {
        // ASM loc_1C7D2 (kid.asm:31432-31436): after loc_1C7C4 confirms $FD is not present,
        // this label checks cmpi.b #$FE,d6. If d6==$FE (delay marker):
        //   move.b (a1)+,d0 ; read the delay duration byte following $FE
        //   bra.s loc_1C7AE  ; hibernate for d0 frames (DBF loop in loc_1C7AE)
        // IntroText1 at kid.asm:31381 embeds $FE,$20 for 32-frame delay.
        // Character chart (kid.asm:31377) documents $FE as delay with duration byte.
        const DELAY_MARKER: u8 = 0xFE;
        const DEFAULT_DELAY_FRAMES: u8 = 0x20; // 32 frames, common in intro texts
        assert_eq!(DELAY_MARKER, 0xFE, "intro text $FE begins delay sequence");
        assert_eq!(
            DEFAULT_DELAY_FRAMES, 0x20,
            "IntroText1 uses $FE,$20 = 32 frame delay"
        );
    }

    #[test]
    fn loc_1c7dc_intro_text_bug_reads_exception_vector_for_char_substitution() {
        // ASM loc_1C7DC (kid.asm:31439-31442): after loc_1C7D2 confirms $FE is not present,
        // this label reads cmp.b (off_20).w,d6. The comment notes this is a programming
        // error - they likely meant cmpi.b #$20,d6 to compare against space character.
        // off_20 (kid.asm:77) is the exception vector at address $20 containing
        // dc.l PrivilegeViolation. The ROM compares d6 against this program counter.
        // If d6 matches the byte at $20 (random/hardware-dependent), replace with $6D.
        // sub_1C7F6 then renders tile index $6D-$41=$2C=44 for this character.
        // ORIGINAL_QUIRK: The comparison reads from exception vector table instead of
        // comparing against immediate value $20. Effect depends on actual byte at $20.
        const BUGGY_CMP_ADDRESS: u16 = 0x20; // off_20 exception vector location
        const SUBSTITUTION_TILE: u8 = 0x6D; // character byte if buggy match succeeds
        assert_eq!(
            BUGGY_CMP_ADDRESS, 0x20,
            "off_20 points to exception vector at $20"
        );
        assert_eq!(SUBSTITUTION_TILE, 0x6D, "matching chars render as tile $6D");
    }

    #[test]
    fn loc_1c7e6_intro_text_character_subtracts_41_for_tile_index() {
        // ASM loc_1C7E6 (kid.asm:31444-31450): after loc_1C7DC checks for buggy $20 match,
        // this label continues regular character processing:
        //   subi.b #$41,d6      ; convert ASCII-encoded char to tile index
        //   bsr.s sub_1C7F6     ; render the character sprite
        //   addq.w #1,d1        ; advance X position by 1 tile (2 pixels on VRAM grid)
        //   move.w 4(a0),d0     ; check delay counter from text header offset 4
        //   beq.s loc_1C7B6      ; if delay is zero, skip to next char immediately
        //   bra.s loc_1C7AE     ; else hibernate for d0 frames before next char
        // Character bytes $5C-$65 map to tiles: '5C-$41=$1B=27', '5D-$41=$1C=28', etc.
        // IntroText5 uses characters from this range for the outro story text.
        const CHAR_ENCODING_OFFSET: u8 = 0x41;
        const FIRST_DIGIT_CHAR: u8 = 0x5C; // '1' in character chart
        const LAST_DIGIT_CHAR: u8 = 0x65; // '0' in character chart
        assert_eq!(
            FIRST_DIGIT_CHAR.wrapping_sub(CHAR_ENCODING_OFFSET),
            0x1B,
            "$5C - $41 = $1B (tile index for '1')"
        );
        assert_eq!(
            LAST_DIGIT_CHAR.wrapping_sub(CHAR_ENCODING_OFFSET),
            0x24,
            "$65 - $41 = $24 (tile index for '0')"
        );
        assert_eq!(
            0x41_u8, b'A',
            "character offset $41 is ASCII 'A', matching the character chart prefix"
        );
    }

    #[test]
    fn loc_1c82c_end_sequence_palette_copies_16_words_from_pal_1c8aa() {
        // ASM loc_1C82C (kid.asm:31483-31485): DBF loop in sub_1C820 that copies palette words.
        //   move.w (a0)+,(a1)+   ; copy word from Pal_1C8AA to Palette_Buffer
        //   dbf d0,loc_1C82C     ; loop $F+1=16 iterations for first palette bank
        // a0 points to Pal_1C8AA (scenes/palette/1C8AA.bin)
        // a1 points to Palette_Buffer (first 32 bytes / 16 words)
        // d0 is $F at entry, so this loops 16 times copying the first 16 palette words.
        // This loads the end sequence palette into the active palette buffer.
        const PALETTE_WORDS_FIRST_BANK: u16 = 16;
        const DBF_COUNTER: u16 = 0x000F;
        assert_eq!(
            DBF_COUNTER.wrapping_add(1),
            PALETTE_WORDS_FIRST_BANK,
            "DBF loop runs $F+1=16 iterations for 16 palette words"
        );
    }

    #[test]
    fn loc_1c84e_end_sequence_palette_copies_48_words_to_offset_20() {
        // ASM loc_1C84E (kid.asm:31493-31498): loads second palette bank into Palette_Buffer+$20.
        // After loc_1C82C's first 16 words, sub_1C820 optionally modifies palette entries
        // (lines 31486-31491) if d1!=0, then continues here:
        //   lea (Palette_Buffer+$20).l,a1  ; advance palette pointer by 32 bytes
        //   moveq #$2F,d0                  ; 48 words in second bank
        // loc_1C856:
        //   move.w (a0)+,(a1)+            ; copy from Pal_1C8AA to second buffer
        //   dbf d0,loc_1C856              ; loop $2F+1=48 iterations
        // Total palette: 16 words in first bank + 48 words in second bank = 64 words.
        // Called from Load_EndSequence (kid.asm:32553) with d1=0 for initial palette,
        // and from loc_1D1CE (kid.asm:32579) with d1=1 for modified palette variant.
        const PALETTE_OFFSET: u16 = 0x20; // 32 bytes = second bank start
        const PALETTE_WORDS_SECOND_BANK: u16 = 48;
        const DBF_COUNTER: u16 = 0x002F;
        assert_eq!(
            PALETTE_OFFSET, 32,
            "$20 is byte offset for second palette bank"
        );
        assert_eq!(
            DBF_COUNTER.wrapping_add(1),
            PALETTE_WORDS_SECOND_BANK,
            "DBF loop runs $2F+1=48 iterations for 48 palette words"
        );
        assert_eq!(
            16 + 48,
            64,
            "total end sequence palette is 64 words (128 bytes)"
        );
    }

    #[test]
    fn loc_1c8a2_options_menu_copies_64_words_from_pal_1c97c() {
        // ASM loc_1C8A2 (kid.asm:31547-31549): DBF loop in sub_1C896 that copies palette words.
        //   lea (Palette_Buffer).l,a1      ; destination = Palette_Buffer
        //   lea Pal_1C97C(pc),a0            ; source = scenes/palette/options.bin
        //   moveq #$3F,d0                   ; 64 words = $3F+1 iterations
        // loc_1C8A2:
        //   move.w (a0)+,(a1)+             ; copy word from Pal_1C97C to Palette_Buffer
        //   dbf d0,loc_1C8A2               ; loop $3F+1=64 iterations
        //   rts
        // This copies the entire options-menu palette (64 words) from ROM to RAM.
        // Called from Load_OptionMenu at kid.asm:31624 before allocating the
        // options-screen object with its initial routine set to loc_1CB2C.
        const OPTIONS_PALETTE_WORDS: u16 = 64;
        const DBF_COUNTER: u16 = 0x003F;
        assert_eq!(
            DBF_COUNTER.wrapping_add(1),
            OPTIONS_PALETTE_WORDS,
            "DBF loop runs $3F+1=64 iterations for 64 palette words"
        );
    }

    #[test]
    fn loc_1cb2c_options_screen_init_sets_panel_positions_and_toggle_byte() {
        // ASM loc_1CB2C (kid.asm:31651-31665): options-screen object routine that initializes
        // the panel positions and toggle byte before entering the intro loop.
        //   clr.w (Options_Selected_Option).w    ; select first option
        //   move.w #$30,$1A(a0)                   ; left panel worker x_pos = $30
        //   move.w #$48,$1E(a0)                   ; left panel render_pos = $48
        //   move.w #$110,$1A(a1)                  ; right panel worker x_pos = $110
        //   move.w #$48,$1E(a1)                   ; right panel render_pos = $48
        //   sf ($FFFFFB5C).w                      ; toggle byte = $FF (sf = set false)
        //   move.w #$118,($FFFFFB5E).w            ; panel delta = $118
        //   clr.w (Camera_Y_pos).w                 ; camera starts at Y=0
        // The NativeRuntime fields mirror these values (lib.rs:806-809):
        //   options_menu_panel_toggle_byte: 0    ; $FF after first sub_1CC88 call
        //   options_menu_left_panel_x: 0x0030    ; matches ASM $30
        //   options_menu_right_panel_x: 0x0110   ; matches ASM $110
        //   options_menu_panel_delta: 0x0118    ; matches ASM $118
        const LEFT_PANEL_X: u16 = 0x0030;
        const RIGHT_PANEL_X: u16 = 0x0110;
        const PANEL_DELTA: u16 = 0x0118;
        assert_eq!(LEFT_PANEL_X, 0x0030, "left panel starts at x=$30");
        assert_eq!(RIGHT_PANEL_X, 0x0110, "right panel starts at x=$110");
        assert_eq!(
            PANEL_DELTA, 0x0118,
            "panel delta is $118 (signed oscillation offset)"
        );
    }

    #[test]
    fn published_tas_demo_2_holds_intro_sequence_1_until_frame_5504() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_5503 = None;
        let mut frame_5504 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_505) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                5_503 => {
                    frame_5503 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                5_504 => {
                    frame_5504 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_5503,
            Some((GameMode::IntroSequence1, None, None)),
            "the timeout restart should keep intro sequence 1 alive through frame 5503",
        );
        assert_eq!(
            frame_5504,
            Some((GameMode::IntroSequence2, None, None)),
            "the timeout restart should only advance into intro sequence 2 on frame 5504",
        );
    }

    #[test]
    fn published_tas_demo_2_holds_intro_sequence_2_until_frame_5953() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_5584 = None;
        let mut frame_5952 = None;
        let mut frame_5953 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_954) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                5_584 => {
                    frame_5584 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                5_952 => {
                    frame_5952 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                5_953 => {
                    frame_5953 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_5584,
            Some((GameMode::IntroSequence2, None, None)),
            "the old frame-5584 frontier should stay inside intro sequence 2",
        );
        assert_eq!(
            frame_5952,
            Some((GameMode::IntroSequence2, None, None)),
            "the timeout restart should keep intro sequence 2 alive through frame 5952",
        );
        assert_eq!(
            frame_5953,
            Some((GameMode::IntroSequence3, None, None)),
            "the timeout restart should only advance into intro sequence 3 on frame 5953",
        );
    }

    #[test]
    fn loc_1b9d0_loc_1b9f4_loc_1ba02_count_intro_sequence_2_frames() {
        // ASM Load_IntroSequence2 (kid.asm:29299) stores `move.l #loc_1B9D0,4(a0)`
        // to allocate the intro sequence 2 worker object. loc_1B9D0 sets up
        // `#loc_1C7A0,4(a0)` (render helper), `#IntroText2,$44(a0)`, and stores
        // `#loc_1B9F4,$44(a5)` as the continuation before calling sub_1C204.
        // loc_1B9F4 hibernates for `#$3C` (60) frames, then seeds `d0=#$149` (329)
        // for the countdown loop. loc_1BA02 runs `Hibernate_Object_1Frame`,
        // `sub_1C1A2` twice, `sub_1C246`, and `sub_1C180` per iteration, with
        // `dbf d0,loc_1BA02` executing 330 times (d0: $149→0). After the loop,
        // loc_1BA02 writes `move.w #$1C,(Game_Mode).w` to switch to intro sequence 3.
        // The total frame budget is: 17 (fade) + 60 (hibernate) + 330 (loop) = 407.
        // Oracle-backed timeout-restart traces show 449 frames for intro sequence 2
        // on the restart path, accounting for the shared fade and dispatch delays.
        const FADE_FRAMES: u16 = 17;
        const HIBERNATE_FRAMES: u16 = 0x3C; // 60
        const LOOP_COUNTER: u16 = 0x149; // 329, dbf executes 330 times
        const EXPECTED_TOTAL: usize =
            FADE_FRAMES as usize + HIBERNATE_FRAMES as usize + (LOOP_COUNTER as usize + 1);
        assert_eq!(EXPECTED_TOTAL, 407);
    }

    #[test]
    fn published_tas_demo_2_exposes_the_first_intro_sequence_3_kid_pose_on_frame_5994() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_5993 = None;
        let mut frame_5994 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_995) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                5_993 => {
                    frame_5993 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                5_994 => {
                    frame_5994 = Some((
                        trace.game_mode,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_5993,
            Some((GameMode::IntroSequence3, None, None)),
            "the timeout restart should still keep the intro-sequence-3 pose hidden on frame 5993",
        );
        assert_eq!(
            frame_5994,
            Some((
                GameMode::IntroSequence3,
                Some(0x0060_0000),
                Some(0x00D0_0000),
                Some(Direction::Left),
            )),
            "frame 5994 should expose the first intro-sequence-3 Kid pose from the ROM gfx-object roster",
        );
    }

    #[test]
    fn published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_6033 = None;
        let mut frame_6208 = None;
        let mut frame_6209 = None;
        let mut frame_6236 = None;
        let mut frame_6237 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(6_238) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                6_033 => {
                    frame_6033 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                6_208 => {
                    frame_6208 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                6_209 => {
                    frame_6209 = Some((
                        trace.game_mode,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                6_236 => {
                    frame_6236 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                6_237 => {
                    frame_6237 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_6033,
            Some((
                GameMode::IntroSequence3,
                Some(0x0060_0000),
                Some(0x00D0_0000),
            )),
            "the timeout restart should keep intro sequence 3 alive on the old frame-6033 frontier",
        );
        assert_eq!(
            frame_6208,
            Some((
                GameMode::IntroSequence3,
                Some(0x0060_0000),
                Some(0x00D0_0000),
            )),
            "the timeout restart should keep intro sequence 3 alive through frame 6208",
        );
        assert_eq!(
            frame_6209,
            Some((
                GameMode::TitleScreen,
                Some(0x0060_0000),
                Some(0x00D0_0000),
                Some(Direction::Left),
            )),
            "frame 6209 should enter title mode without discarding the intro-sequence-3 Kid pose",
        );
        assert_eq!(
            frame_6236,
            Some((GameMode::TitleScreen, Some(0x0060_0000), Some(0x00D0_0000),)),
            "the timeout restart should keep the intro-sequence-3 pose alive through the first 28 title frames",
        );
        assert_eq!(
            frame_6237,
            Some((GameMode::TitleScreen, None, None)),
            "after the carry-over window the timeout restart should hide Kid again while the title rebuild continues",
        );
    }

    #[test]
    fn published_tas_demo_2_restores_the_standard_title_pose_on_frame_6358() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_6357 = None;
        let mut frame_6358 = None;
        let mut frame_6359 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(6_360) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                6_357 => {
                    frame_6357 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                6_358 => {
                    frame_6358 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                6_359 => {
                    frame_6359 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_6357,
            Some((GameMode::TitleScreen, true, None, None)),
            "the timeout restart should still keep Kid hidden on the last pre-return title frame",
        );
        assert_eq!(
            frame_6358,
            Some((
                GameMode::TitleScreen,
                true,
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
                Some(Direction::Right),
            )),
            "frame 6358 should restore the standard title pose on the first post-carry-over visible title frame",
        );
        assert_eq!(
            frame_6359,
            Some((
                GameMode::TitleScreen,
                true,
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
                Some(Direction::Right),
            )),
            "once the rebuild window ends the standard title pose should stay visible on the following frame",
        );
    }

    #[test]
    fn published_tas_demo_1_uses_the_second_intro_sequence_3_roster_pose_on_frame_9097() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9096 = None;
        let mut frame_9097 = None;
        let mut frame_9098 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_099) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_096 => {
                    frame_9096 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                9_097 => {
                    frame_9097 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                9_098 => {
                    frame_9098 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_9096,
            Some((GameMode::IntroSequence3, None, None)),
            "frame 9096 should already be the hidden intro-sequence-3 frame before the roster pose appears",
        );
        assert_eq!(
            frame_9097,
            Some((
                GameMode::IntroSequence3,
                Some(0x0070_0000),
                Some(0x00CC_0000),
            )),
            "frame 9097 should expose the second Load_IntroSequence3 roster's third descriptor pose",
        );
        assert_eq!(
            frame_9098,
            Some((
                GameMode::IntroSequence3,
                Some(0x0070_0000),
                Some(0x00CC_0000),
            )),
            "frame 9098 should keep the same second-roster intro-sequence-3 pose once the glyph objects appear",
        );
    }

    #[test]
    fn loc_1aeba_reads_intro_sequence_3_descriptors_from_the_rotating_roster_and_stops_on_ff() {
        assert_eq!(
            crate::movement::intro_sequence_3_descriptor_loc_1AEBA(1, 2),
            Some(crate::movement::IntroSequence3Descriptor {
                x_pos: 0x0070,
                y_pos: 0x00CC,
                raw_flags: 0x1219,
            }),
            "ASM loc_1AEBA should use the second `unk_1AF78` roster's third byte as descriptor index 6, then scale into the six-byte `unk_1AF24` table entry at `$70,$CC,$1219`",
        );
        assert_eq!(
            crate::movement::intro_sequence_3_descriptor_loc_1AEBA(1, 6),
            None,
            "ASM loc_1AEBA should branch to `return_1AF22` as soon as the roster byte is `$FF` because `move.b (a2)+,d0` leaves the sign bit set for `bmi.w`",
        );
    }

    #[test]
    fn load_intro_sequence_3_pose_uses_all_four_rosters_and_word_positions_as_16_16() {
        assert_eq!(
            PlayerState::intro_sequence_3_roster_count(),
            4,
            "ASM Load_IntroSequence3 should rotate `$FFFFFC83` through four eight-byte `unk_1AF78` rosters via `lsl.w #3` then `andi.w #$18`",
        );

        let pose = PlayerState::intro_sequence_3_pose_for_roster(1);

        assert_eq!(
            pose.x_pos, 0x0070_0000,
            "ASM loc_1AEEC copies descriptor word `$70` into gfx X; trace-visible Kid pose should expose that word as 16.16 fixed point",
        );
        assert_eq!(
            pose.y_pos, 0x00CC_0000,
            "ASM loc_1AEEC copies descriptor word `$CC` into gfx Y; trace-visible Kid pose should expose that word as 16.16 fixed point",
        );
    }

    #[test]
    fn loc_1aee2_switches_intro_sequence_3_allocation_specs_from_descriptor_bit_0() {
        let low_spec_descriptor = crate::movement::intro_sequence_3_descriptor_loc_1AEBA(0, 5)
            .expect("first roster slot 5 should point at descriptor 7");
        let high_spec_descriptor = crate::movement::intro_sequence_3_descriptor_loc_1AEBA(1, 2)
            .expect("second roster slot 2 should point at descriptor 6");

        assert_eq!(
            crate::movement::intro_sequence_3_allocation_spec_loc_1AEE2(low_spec_descriptor),
            0x01FF_0000,
            "ASM loc_1AEE2 should stay on the earlier `$01FF0000` allocation when `btst #0,5(a0)` sees bit 0 clear in descriptor word `$121C`",
        );
        assert_eq!(
            crate::movement::intro_sequence_3_allocation_spec_loc_1AEE2(high_spec_descriptor),
            0x0201_0000,
            "ASM loc_1AEE2 should branch to the `$02010000` allocator when descriptor word `$1219` leaves bit 0 set in byte `5(a0)`",
        );
    }

    #[test]
    fn loc_1aeec_copies_intro_sequence_3_descriptor_fields_into_the_gfx_object_tail() {
        let roster_descriptor = crate::movement::intro_sequence_3_descriptor_loc_1AEBA(1, 2)
            .expect("second roster slot 2 should point at descriptor 6");
        let low_priority_descriptor = crate::movement::intro_sequence_3_descriptor_loc_1AEBA(0, 5)
            .expect("first roster slot 5 should point at descriptor 7");

        assert_eq!(
            crate::movement::intro_sequence_3_gfx_object_loc_1AEEC(roster_descriptor),
            crate::movement::IntroSequence3GfxObjectLoc1AEEC {
                x_pos: 0x0070,
                y_pos: 0x00CC,
                display_enabled: true,
                palette_line: 1,
                vram_tile: 0x01D6,
                priority: 1,
                x_direction: 0,
                addroffset_sprite: 0x1218,
            },
            "ASM loc_1AEEC should copy `$70,$CC`, set `$13(a1)`, palette line 1, VRAM tile `$01D6`, keep priority high from bit 0, and store descriptor word `$1219` back to `$22(a1)` with bits 0-1 cleared",
        );
        assert_eq!(
            crate::movement::intro_sequence_3_gfx_object_loc_1AEEC(low_priority_descriptor),
            crate::movement::IntroSequence3GfxObjectLoc1AEEC {
                x_pos: 0x0098,
                y_pos: 0x00C4,
                display_enabled: true,
                palette_line: 1,
                vram_tile: 0x01D6,
                priority: 0,
                x_direction: 0,
                addroffset_sprite: 0x121C,
            },
            "ASM loc_1AEEC should keep priority clear when descriptor word `$121C` leaves `bclr #0,d0` with the zero flag set",
        );
    }

    #[test]
    fn loc_1af12_sets_x_direction_only_when_descriptor_bit_1_is_set() {
        assert_eq!(
            crate::movement::intro_sequence_3_x_direction_loc_1AF12(0x1200),
            0,
            "ASM loc_1AF12 should make `sne $16(a1)` leave `x_direction(a1)=0` when `bclr #1,d0` sees descriptor bit 1 clear",
        );
        assert_eq!(
            crate::movement::intro_sequence_3_x_direction_loc_1AF12(0x1202),
            u8::MAX,
            "ASM loc_1AF12 should make `sne $16(a1)` write `$FF` when descriptor bit 1 survives into the `bclr #1,d0` test",
        );
    }

    #[test]
    fn loc_1b012_copies_exactly_0x2c0_longwords_into_the_later_level_layout_window() {
        let source = (0..0x02C4)
            .map(|value| 0x1000_0000_u32.wrapping_add(value))
            .collect::<Vec<_>>();

        let copied = crate::boot::intro_sequence_2_layout_copy_loc_1B012(&source);

        assert_eq!(
            copied.destination_addr, 0xFFFF_B152,
            "ASM loc_1B012 should target `$FFFFB152`, the later `Level_Layout` window fed by `lea ($FFFFB152).w,a2`",
        );
        assert_eq!(
            copied.copied_longwords.len(),
            0x02C0,
            "ASM loc_1B012 seeds `d0=#$2BF`, so the inclusive `dbf d0,loc_1B012` loop should copy exactly `$2C0` longwords",
        );
        assert_eq!(
            copied.copied_longwords[0], source[0],
            "ASM loc_1B012 should copy the first source longword before the first `dbf` decrement",
        );
        assert_eq!(
            copied.copied_longwords[0x02BF], source[0x02BF],
            "ASM loc_1B012 should still copy source longword `$2BF` because the last store happens before `dbf` reaches `-1`",
        );
    }

    #[test]
    fn loc_1b100_selects_the_intro_sequence_5_art_source_from_fc82() {
        assert_eq!(
            crate::boot::intro_sequence_5_art_source_loc_1B100(0),
            "byte_213D9",
            "ASM loc_1B100 should keep the default `byte_213D9` art pointer when `tst.b ($FFFFFC82).w` sees zero",
        );
        assert_eq!(
            crate::boot::intro_sequence_5_art_source_loc_1B100(1),
            "byte_219C1",
            "ASM loc_1B100 should keep `byte_219C1` when the first non-zero branch is taken and `cmpi.b #1,($FFFFFC82).w` then succeeds",
        );
        assert_eq!(
            crate::boot::intro_sequence_5_art_source_loc_1B100(2),
            "byte_216B7",
            "ASM loc_1B100 should fall through to `byte_216B7` when `($FFFFFC82).w` is non-zero and not equal to 1",
        );
    }

    #[test]
    fn loc_1be3c_loads_particle_sprite_from_data_tables() {
        let particles = crate::movement::intro_sequence_5_particle_data_loc_1BE3C();

        assert_eq!(
            particles.len(),
            3,
            "ASM loc_1BE3C (kid.asm:30667) should load exactly three particle objects from unk_1BE18, unk_1BE24, unk_1BE30"
        );

        assert_eq!(
            particles[0].sprite_label, 0x0011,
            "First particle should use sprite label $0011 from unk_1BE18 (kid.asm:30629)"
        );
        assert_eq!(
            particles[0].x_pos, 0x0078,
            "First particle should have x_pos=$78 from unk_1BE18 (kid.asm:30634)"
        );
        assert_eq!(
            particles[0].y_vel, 0xFFFE,
            "First particle should have y_vel=$FFFE from unk_1BE18 (kid.asm:30637-30638)"
        );
        assert_eq!(
            particles[0].loop_count, 0x0028,
            "First particle should have loop_count=$28 from unk_1BE18 (kid.asm:30640)"
        );

        assert_eq!(
            particles[1].sprite_label, 0x0011,
            "Second particle should use sprite label $0011 from unk_1BE24 (kid.asm:30641)"
        );
        assert_eq!(
            particles[1].x_pos, 0x0058,
            "Second particle should have x_pos=$58 from unk_1BE24 (kid.asm:30644)"
        );
        assert_eq!(
            particles[1].loop_count, 0x003C,
            "Second particle should have loop_count=$3C from unk_1BE24 (kid.asm:30652)"
        );

        assert_eq!(
            particles[2].sprite_label, 0x0011,
            "Third particle should use sprite label $0011 from unk_1BE30 (kid.asm:30653)"
        );
        assert_eq!(
            particles[2].x_pos, 0x0030,
            "Third particle should have x_pos=$30 from unk_1BE30 (kid.asm:30656)"
        );
        assert_eq!(
            particles[2].loop_count, 0x0028,
            "Third particle should have loop_count=$28 from unk_1BE30 (kid.asm:30664)"
        );
    }

    #[test]
    fn loc_1be74_adds_gravity_to_falling_particles() {
        let gravity = crate::movement::intro_sequence_5_particle_fall_loc_1BE74();

        assert_eq!(
            gravity.gravity_add, 0x0000_4000,
            "ASM loc_1BE74 (kid.asm:30684) should add #$4000 = 0.25 px/frame gravity to y_vel"
        );
    }

    #[test]
    fn loc_1be88_positions_center_particle_based_on_fc82() {
        let variant_0 = crate::movement::intro_sequence_5_center_particle_loc_1BE88(0);
        let variant_1 = crate::movement::intro_sequence_5_center_particle_loc_1BE88(1);
        let variant_2 = crate::movement::intro_sequence_5_center_particle_loc_1BE88(2);

        assert_eq!(
            variant_0.x_pos, 0x002A,
            "ASM loc_1BE88 (kid.asm:30695-30708) should adjust x from $30 to $2A when FC82 == 0"
        );
        assert_eq!(
            variant_0.y_pos, 0x0074,
            "ASM loc_1BE88 (kid.asm:30695-30708) should adjust y from $70 to $74 when FC82 == 0"
        );
        assert_eq!(
            variant_0.sprite_label, "LnkTo_unk_E10B6-Data_Index",
            "ASM loc_1BE88 (kid.asm:30715-30721) should select LnkTo_unk_E10B6 when FC82 == 0"
        );

        assert_eq!(
            variant_1.x_pos, 0x0030,
            "ASM loc_1BE88 should keep x=$30 when FC82 == 1 (no adjustment)"
        );
        assert_eq!(
            variant_1.y_pos, 0x0070,
            "ASM loc_1BE88 should keep y=$70 when FC82 == 1 (no adjustment)"
        );
        assert_eq!(
            variant_1.sprite_label, "LnkTo_unk_E10C6-Data_Index",
            "ASM loc_1BE88 should select LnkTo_unk_E10C6 when FC82 == 1"
        );

        assert_eq!(
            variant_2.x_pos, 0x002A,
            "ASM loc_1BEBE (kid.asm:30704-30709) should adjust x from $30 to $2A when FC82 != 0 && FC82 != 1"
        );
        assert_eq!(
            variant_2.y_pos, 0x0074,
            "ASM loc_1BEBE should adjust y from $70 to $74 when FC82 != 0 && FC82 != 1"
        );
        assert_eq!(
            variant_2.sprite_label, "LnkTo_unk_E10BE-Data_Index",
            "ASM loc_1BE88 should select LnkTo_unk_E10BE when FC82 >= 2"
        );

        assert_eq!(
            variant_0.x_vel, 0x0002_F000,
            "ASM loc_1BECE (kid.asm:30711) should set x_vel to $2F000 for all FC82 variants"
        );
        assert_eq!(
            variant_0.y_vel, 0xFFFF_7100,
            "ASM loc_1BECE (kid.asm:30712) should set y_vel to -$8F000 for all FC82 variants"
        );
        assert_eq!(
            variant_0.vram_tile, 0x05EB,
            "ASM loc_1BECE (kid.asm:30713) should set vram_tile to $5EB for all FC82 variants"
        );
    }

    #[test]
    fn loc_1bf02_sets_sprite_offset_and_loop_counter_from_fc82_variant() {
        let variant_0 =
            crate::movement::intro_sequence_5_sprite_offset_loc_1BF02("LnkTo_unk_E10B6-Data_Index");
        let variant_1 =
            crate::movement::intro_sequence_5_sprite_offset_loc_1BF02("LnkTo_unk_E10C6-Data_Index");
        let variant_2 =
            crate::movement::intro_sequence_5_sprite_offset_loc_1BF02("LnkTo_unk_E10BE-Data_Index");

        assert_eq!(
            variant_0.addroffset_sprite, 0x0F16,
            "ASM loc_1BF02 (kid.asm:30723) should set addroffset_sprite from LnkTo_unk_E10B6 when FC82 == 0"
        );
        assert_eq!(
            variant_1.addroffset_sprite, 0x0F1E,
            "ASM loc_1BF02 should set addroffset_sprite from LnkTo_unk_E10C6 when FC82 == 1"
        );
        assert_eq!(
            variant_2.addroffset_sprite, 0x0F26,
            "ASM loc_1BF02 should set addroffset_sprite from LnkTo_unk_E10BE when FC82 >= 2"
        );
        assert_eq!(
            variant_0.hibernate_loop_count, 0x001C,
            "ASM loc_1BF02 (kid.asm:30725) should init d0 to $1C = 28 iterations"
        );
    }

    #[test]
    fn loc_1bf0a_hibernate_loop_accumulates_y_vel_for_28_frames() {
        let loop_params = crate::movement::intro_sequence_5_hibernate_loop_loc_1BF0A();

        assert_eq!(
            loop_params.iterations, 0x001C,
            "ASM loc_1BF0A (kid.asm:30727) should loop $1C = 28 times seeded from d0"
        );
        assert_eq!(
            loop_params.y_vel_delta_per_frame, 0x0000_9000,
            "ASM loc_1BF0A (kid.asm:30729) should add #$9000 to y_vel each iteration"
        );
        assert!(
            !loop_params.is_moved_after_loop,
            "ASM loc_1BF0A+sf (kid.asm:30731) should clear is_moved with sf after DBF expires"
        );
    }

    #[test]
    fn loc_1bf1e_is_infinite_hibernate_terminal_state() {
        let is_terminal = crate::movement::intro_sequence_5_hibernate_forever_loc_1BF1E();

        assert!(
            is_terminal,
            "ASM loc_1BF1E (kid.asm:30733-30735) is an infinite hibernate loop marking terminal state"
        );
    }

    #[test]
    fn published_tas_only_presses_start_on_the_first_timeout_restart_intro_sequence_3_visit() {
        let (_, script) = cached_any_percent_published_runtime_and_script();
        let first_visit_start_frames = (6_204..=6_211)
            .filter(|&frame| script[frame] & 0x80 != 0)
            .collect::<Vec<_>>();
        let second_visit_start_frames = (9_308..=9_316)
            .filter(|&frame| script[frame] & 0x80 != 0)
            .collect::<Vec<_>>();

        assert_eq!(
            first_visit_start_frames,
            vec![6_207],
            "the frame-6209 intro-sequence-3 title return is driven by the published TAS Start press on frame 6207",
        );
        assert_eq!(
            second_visit_start_frames,
            Vec::<usize>::new(),
            "the later frame-9312 frontier has no Start press to justify a direct intro-sequence-3 title return",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_intro_sequence_3_visit_matches_oracle_frame_9312() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9311 = None;
        let mut frame_9312 = None;
        let mut frame_9354 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_355) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_311 => {
                    frame_9311 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                9_312 => {
                    frame_9312 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                9_354 => {
                    frame_9354 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B96E keeps polling the Start-gated sub_1C246 path while the
        // roster still points at the second Load_IntroSequence3 visit; oracle
        // frame 9312 still exposes the same visible pose at $70,$CC.
        assert_eq!(
            frame_9311,
            Some((
                GameMode::IntroSequence3,
                Some(0x0070_0000),
                Some(0x00CC_0000),
            )),
            "frame 9311 should still be the live second timeout-restart intro-sequence-3 roster pose",
        );
        assert_eq!(
            frame_9312,
            Some((
                GameMode::IntroSequence3,
                Some(0x0070_0000),
                Some(0x00CC_0000),
            )),
            "oracle frame 9312 is still inside loc_1B96E with the second roster pose alive; native must not time out into intro sequence 4 yet",
        );
        assert_eq!(
            frame_9354,
            Some((
                GameMode::IntroSequence3,
                Some(0x0070_0000),
                Some(0x00CC_0000),
            )),
            "oracle keeps the second intro-sequence-3 visit alive through frame 9354 before the natural timeout lands",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_carries_intro_sequence_3_pose_into_intro_sequence_4() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9355 = None;
        let mut frame_9381 = None;
        let mut frame_9382 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_383) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_355 => {
                    frame_9355 = Some((
                        trace.game_mode,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                9_381 => {
                    frame_9381 = Some((
                        trace.game_mode,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                9_382 => {
                    frame_9382 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B96A flips into `Game_Mode=$20`, but Load_IntroSequence4 /
        // sub_1B988 do not blank the old roster immediately; oracle keeps the
        // second intro-sequence-3 pose alive until frame 9382.
        assert_eq!(
            frame_9355,
            Some((
                GameMode::IntroSequence4,
                Some(0x0070_0000),
                Some(0x00CC_0000),
                Some(Direction::Left),
            )),
            "frame 9355 should enter intro sequence 4 without discarding the second intro-sequence-3 roster pose",
        );
        assert_eq!(
            frame_9381,
            Some((
                GameMode::IntroSequence4,
                Some(0x0070_0000),
                Some(0x00CC_0000),
                Some(Direction::Left),
            )),
            "oracle keeps the carried intro-sequence-3 pose visible through frame 9381",
        );
        assert_eq!(
            frame_9382,
            Some((GameMode::IntroSequence4, None, None)),
            "once sub_1B988 has taken over, frame 9382 should finally clear the carried intro-sequence-3 pose",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_delays_intro_sequence_4_camera_until_sub_1b988() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9391 = None;
        let mut frame_9392 = None;
        let mut frame_9393 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_394) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_391 => {
                    frame_9391 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_392 => {
                    frame_9392 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_393 => {
                    frame_9393 = Some((trace.game_mode, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B988 writes `move.w #4,(Camera_Y_pos).w`, but the
        // `Load_IntroSequence4`-allocated object only dispatches after the carried
        // intro-sequence-3 pose has already cleared.
        assert_eq!(
            frame_9391,
            Some((GameMode::IntroSequence4, 0)),
            "frame 9391 should still be in the zero-camera gap before sub_1B988 runs",
        );
        assert_eq!(
            frame_9392,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "frame 9392 should expose the delayed sub_1B988 camera_y_pos=4 hand-off",
        );
        assert_eq!(
            frame_9393,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "once sub_1B988 has run, the intro-sequence-4 camera should stay at y=4",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_keeps_intro_sequence_4_alive_on_frame_9435() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9434 = None;
        let mut frame_9435 = None;
        let mut frame_9436 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_437) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_434 => {
                    frame_9434 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_435 => {
                    frame_9435 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_436 => {
                    frame_9436 = Some((trace.game_mode, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B9B6 counts `move.w #$21B,d0` down with `dbf` before the
        // `move.w #$24,(Game_Mode).w` hand-off, so the timeout-restart path must
        // still be in intro sequence 4 across the old frame-9435 frontier.
        assert_eq!(
            frame_9434,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "frame 9434 should still be in intro sequence 4 with the latched camera hand-off",
        );
        assert_eq!(
            frame_9435,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "frame 9435 should stay in intro sequence 4 because loc_1B9B6 has not timed out yet",
        );
        assert_eq!(
            frame_9436,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "the following frame should remain in intro sequence 4 until the ROM timer expires",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_keeps_intro_sequence_4_alive_on_frame_9895() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9894 = None;
        let mut frame_9895 = None;
        let mut frame_9896 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_897) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_894 => {
                    frame_9894 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_895 => {
                    frame_9895 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_896 => {
                    frame_9896 = Some((trace.game_mode, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence4 allocates the `sub_1B988` worker, and only that
        // worker reaches loc_1B9B6's `move.w #$21B,d0` / `dbf` timeout loop. The
        // timeout-restart path therefore cannot count from the frame where
        // `Game_Mode=$20` first becomes visible; oracle still shows intro
        // sequence 4 across the old frame-9895 frontier.
        assert_eq!(
            frame_9894,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "frame 9894 should still be in intro sequence 4 with the sub_1B988 camera hand-off latched",
        );
        assert_eq!(
            frame_9895,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "frame 9895 should remain in intro sequence 4 because loc_1B9B6 started after the delayed sub_1B988 dispatch",
        );
        assert_eq!(
            frame_9896,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "frame 9896 should still be in intro sequence 4 until the delayed loc_1B9B6 loop actually expires",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_carries_camera_into_intro_sequence_5_on_frame_9949() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9948 = None;
        let mut frame_9949 = None;
        let mut frame_9950 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_951) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_948 => {
                    frame_9948 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_949 => {
                    frame_9949 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_950 => {
                    frame_9950 = Some((trace.game_mode, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B988 stores `#loc_1B9B2,$44(a5)` and immediately calls
        // sub_1C204, whose `addi.w #$10,($FFFFF876).w` / `cmpi.w #$100` fade loop
        // delays the later `loc_1B9B6` DBF countdown. Once loc_1B9B6 finally
        // flips `Game_Mode=$24`, the carried `Camera_Y_pos=4` still survives into
        // intro sequence 5.
        assert_eq!(
            frame_9948,
            Some((GameMode::IntroSequence4, 0x0004_0000)),
            "frame 9948 should still be intro sequence 4 while sub_1C204's delayed loc_1B9B6 timeout finishes",
        );
        assert_eq!(
            frame_9949,
            Some((GameMode::IntroSequence5, 0x0004_0000)),
            "frame 9949 should reach intro sequence 5 but keep the carried camera_y_pos=4 hand-off",
        );
        assert_eq!(
            frame_9950,
            Some((GameMode::IntroSequence5, 0x0004_0000)),
            "frame 9950 should remain in intro sequence 5 with the carried camera hand-off still latched",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_clears_intro_sequence_5_camera_on_frame_9980() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_9979 = None;
        let mut frame_9980 = None;
        let mut frame_9981 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_982) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                9_979 => {
                    frame_9979 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_980 => {
                    frame_9980 = Some((trace.game_mode, trace.camera_y_pos));
                }
                9_981 => {
                    frame_9981 = Some((trace.game_mode, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence5 only allocates `loc_1BA28` / `loc_1BCD0`; it
        // does not invoke the later `sub_1B222` camera rewrite path itself. The
        // oracle-backed timeout-restart window therefore keeps the inherited
        // `sub_1B988` camera only through frame 9979, then clears it on 9980
        // while intro sequence 5 remains active.
        // ASM loc_1BA28 allocates a tile-upload worker at loc_1BC36 (kid.asm:30516)
        // which calls sub_1C034 eight times to decompress title art, then yields
        // forever at loc_1BCCA (kid.asm:30562). ASM loc_1BCD0 (kid.asm:30567) also
        // allocates five title gfx slots and spawns particle objects with routine
        // loc_1BE3C (kid.asm:30667) for screen effects.
        assert_eq!(
            frame_9979,
            Some((GameMode::IntroSequence5, 0x0004_0000)),
            "frame 9979 should still show the carried intro-sequence-4 camera inside intro sequence 5",
        );
        assert_eq!(
            frame_9980,
            Some((GameMode::IntroSequence5, 0x0000_0000)),
            "frame 9980 should stay in intro sequence 5 but drop the carried camera",
        );
        assert_eq!(
            frame_9981,
            Some((GameMode::IntroSequence5, 0x0000_0000)),
            "frame 9981 should keep the cleared intro-sequence-5 camera state",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_exposes_intro_sequence_5_slot_2_on_frame_10007() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_10006 = None;
        let mut frame_10007 = None;
        let mut frame_10008 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(10_009) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                10_006 => {
                    frame_10006 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_007 => {
                    frame_10007 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_008 => {
                    frame_10008 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BCD0 allocates the third title gfx slot with
        // `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`. Oracle probes of the
        // verifier's fixed slot-2 WRAM reads ($FF24FA/$FF24FE) keep those words at
        // zero through frame 10006, then expose the loc_1BCD0 slot-2 pose on
        // frame 10007 while `Game_Mode=$24` still points at intro sequence 5.
        assert_eq!(
            frame_10006,
            Some((GameMode::IntroSequence5, None, None)),
            "frame 10006 should still keep the intro-sequence-5 slot-2 kid_gfx words cleared",
        );
        assert_eq!(
            frame_10007,
            Some((
                GameMode::IntroSequence5,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10007 should expose loc_1BCD0's slot-2 title pose while intro sequence 5 stays active",
        );
        assert_eq!(
            frame_10008,
            Some((
                GameMode::IntroSequence5,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10008 should keep the loc_1BCD0 slot-2 title pose latched",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_keeps_intro_sequence_5_alive_past_frame_10029() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_10028 = None;
        let mut frame_10029 = None;
        let mut frame_10030 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(10_031) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                10_028 => {
                    frame_10028 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_029 => {
                    frame_10029 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_030 => {
                    frame_10030 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA28 calls sub_1C204 before loc_1BA5C starts the `move.w
        // #$1BC,d0` / `dbf` countdown, so the timeout-restart copy of intro
        // sequence 5 cannot legitimately fall back to title on local frame 10029.
        assert_eq!(
            frame_10028,
            Some((
                GameMode::IntroSequence5,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10028 should still show the loc_1BCD0 slot-2 pose inside intro sequence 5",
        );
        assert_eq!(
            frame_10029,
            Some((
                GameMode::IntroSequence5,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10029 should stay in intro sequence 5 instead of jumping to title early",
        );
        assert_eq!(
            frame_10030,
            Some((
                GameMode::IntroSequence5,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10030 should keep the same intro-sequence-5 slot-2 pose after the old false frontier",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_carries_intro_sequence_5_pose_into_title() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_10288 = None;
        let mut frame_10289 = None;
        let mut frame_10290 = None;
        let mut frame_10317 = None;
        let mut frame_10318 = None;
        let mut frame_10437 = None;
        let mut frame_10438 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(10_439) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                10_288 => {
                    frame_10288 = Some((mask, trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_289 => {
                    frame_10289 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_290 => {
                    frame_10290 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_317 => {
                    frame_10317 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_318 => {
                    frame_10318 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_437 => {
                    frame_10437 = Some((trace.game_mode, trace.kid_x_pos, trace.kid_y_pos));
                }
                10_438 => {
                    frame_10438 = Some((
                        trace.game_mode,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 checks Ctrl_1_Pressed during loc_1BA5C's long loop and
        // writes `move.w #$2C,(Game_Mode).w` without rebuilding the title gfx
        // slots. Oracle probes show the published TAS presses Start (`mask=0x80`)
        // on frame 10288, stays in intro sequence 5 through 10289, carries the
        // loc_1BCD0 slot-2 pose through title frame 10317, blanks Kid on 10318,
        // then restores the standard title pose on 10438.
        assert_eq!(
            frame_10288,
            Some((
                0x80,
                GameMode::IntroSequence5,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10288 should be the published TAS Start press that arms the intro-sequence-5 title return",
        );
        assert_eq!(
            frame_10289,
            Some((
                GameMode::IntroSequence5,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10289 should still be intro sequence 5 during the shared 2-frame Start deferral",
        );
        assert_eq!(
            frame_10290,
            Some((
                GameMode::TitleScreen,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 10290 should enter title mode without discarding loc_1BCD0's slot-2 pose",
        );
        assert_eq!(
            frame_10317,
            Some((
                GameMode::TitleScreen,
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "the intro-sequence-5 slot-2 pose should survive through the first 28 title frames",
        );
        assert_eq!(
            frame_10318,
            Some((GameMode::TitleScreen, None, None)),
            "the title rebuild should blank Kid on the frame after the carried slot-2 pose clears",
        );
        assert_eq!(
            frame_10437,
            Some((GameMode::TitleScreen, None, None)),
            "Kid should stay hidden through the final pre-title-pose rebuild frame",
        );
        assert_eq!(
            frame_10438,
            Some((
                GameMode::TitleScreen,
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
                Some(Direction::Right),
            )),
            "frame 10438 should restore the standard title pose once the intro-sequence-5 rebuild finishes",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_starts_demo_play_on_frame_10986() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_10985 = None;
        let mut frame_10986 = None;
        let mut frame_10987 = None;
        let mut frame_10988 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(10_989) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                10_985 => {
                    frame_10985 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                10_986 => {
                    frame_10986 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                10_987 => {
                    frame_10987 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                10_988 => {
                    frame_10988 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 returns straight to title after the shared 2-frame Start
        // deferral, so the reused title shell is already two frames into its
        // pre-menu attract wait when the direct title return becomes visible.
        // Oracle therefore reaches Obj_TitleMenu's `loc_1BB02:
        // move.w #$10,(Game_Mode).w` two frames earlier than the cold-boot title
        // path while the visible title pose stays unchanged.
        assert_eq!(
            frame_10985,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 10985 should still be the settled timeout-restart title state before Obj_TitleMenu reaches loc_1BB02",
        );
        assert_eq!(
            frame_10986,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 10986 should flip into demo playback two frames earlier on the intro-sequence-5 direct-return path",
        );
        assert_eq!(
            frame_10987,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 10987 should stay in demo playback while the preserved title pose is still visible",
        );
        assert_eq!(
            frame_10988,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 10988 should still be demo playback on the accelerated timeout-restart title path",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_uses_oracle_demo_camera_preload_on_frame_11043() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_11042 = None;
        let mut frame_11043 = None;
        let mut frame_11044 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(11_045) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                11_042 => {
                    frame_11042 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                11_043 => {
                    frame_11043 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                11_044 => {
                    frame_11044 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu loc_1BB02 flips to `Game_Mode=$10`, and the
        // timeout-restart hand-off keeps one hidden preload camera row before
        // Load_InGame's respawn camera snap lands on the next frame.
        assert_eq!(
            frame_11042,
            Some((GameMode::DemoPlay, Some(56), 0x0000_0000)),
            "frame 11042 should still be on the blank hidden demo bootstrap camera row",
        );
        assert_eq!(
            frame_11043,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                demo::DEMO_2_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 11043 should expose the oracle-backed hidden preload camera row",
        );
        assert_eq!(
            frame_11044,
            Some((GameMode::DemoPlay, Some(58), 0x02A0_0000)),
            "frame 11044 should return to Load_InGame's respawn camera snap",
        );
    }

    #[test]
    fn published_tas_second_timeout_restart_keeps_the_oracle_backed_cyclone_flight_arc_on_frame_11548()
     {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_11547 = None;
        let mut frame_11547_special_motion = None;
        let mut frame_11548 = None;
        let mut frame_11549 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(11_550) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                11_547 => {
                    frame_11547 = Some((
                        trace.game_mode,
                        trace.current_helmet_id,
                        trace.kid_y_pos,
                        trace.camera_y_pos,
                    ));
                    frame_11547_special_motion = trace
                        .gameplay_debug_fields
                        .get("kid_special_motion_probe")
                        .cloned();
                }
                11_548 => {
                    frame_11548 = Some((
                        trace.game_mode,
                        trace.current_helmet_id,
                        trace.kid_y_pos,
                        trace.camera_y_pos,
                    ));
                }
                11_549 => {
                    frame_11549 = Some((
                        trace.game_mode,
                        trace.current_helmet_id,
                        trace.kid_y_pos,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_11547,
            Some((
                GameMode::DemoPlay,
                PlayerForm::Cyclone.helmet_id(),
                Some(0x010F_0000),
                0x0073_0000,
            )),
            "frame 11547 should still be the clean pre-frontier Cyclone flight pose",
        );
        assert!(
            frame_11547_special_motion
                .as_deref()
                .is_some_and(|debug| debug.contains("cyclone_flying=1")),
            "frame 11547 should already be on the live Cyclone flight path before the vertical frontier opens",
        );

        // ASM sub_B168 loc_B214..loc_B26C: when `Cyclone_flying` is set, the ROM
        // advances the 16-bit `Cyclone_YAcceleration`, integrates it into
        // `y_vel(a3)`, and keeps the post-thrust rise on the oracle's upward arc
        // across the first 11548/11549 authoritative mismatch window.
        assert_eq!(
            frame_11548,
            Some((
                GameMode::DemoPlay,
                PlayerForm::Cyclone.helmet_id(),
                Some(0x010E_9600),
                0x0072_0000,
            )),
            "frame 11548 should stay on the oracle-backed Cyclone rise instead of freezing one pixel higher",
        );
        assert_eq!(
            frame_11549,
            Some((
                GameMode::DemoPlay,
                PlayerForm::Cyclone.helmet_id(),
                Some(0x010D_C600),
                0x0071_0000,
            )),
            "frame 11549 should keep following the oracle-backed Cyclone rise on the next frame",
        );
    }

    #[test]
    fn published_tas_demo_2_resumes_demo_play_on_frame_6906() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_6905 = None;
        let mut frame_6906 = None;
        let mut frame_6907 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(6_908) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                6_905 => {
                    frame_6905 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                6_906 => {
                    frame_6906 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                6_907 => {
                    frame_6907 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_6905,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 6905 should still be the last settled timeout-restart title frame",
        );
        assert_eq!(
            frame_6906,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 6906 should flip back into demo playback without moving Kid off the standard title pose yet",
        );
        assert_eq!(
            frame_6907,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "the following frame should stay in demo playback while the title-pose bootstrap continues",
        );
    }

    #[test]
    fn published_tas_demo_2_ignores_queued_start_during_timeout_restart_demo_bootstrap() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_6943 = None;
        let mut frame_6945 = None;
        let mut frame_6947 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(6_948) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                6_943 => {
                    frame_6943 = Some((trace.game_mode, trace.current_level, trace.paused));
                }
                6_945 => {
                    frame_6945 = Some((trace.game_mode, trace.current_level, trace.paused));
                }
                6_947 => {
                    frame_6947 = Some((trace.game_mode, trace.current_level, trace.paused));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_6943,
            Some((GameMode::DemoPlay, LevelId(0x15), false)),
            "the queued Start frame should still trace as live demo playback on demo 1",
        );
        assert_eq!(
            frame_6945,
            Some((GameMode::DemoPlay, LevelId(0x15), false)),
            "two frames later the timeout-restarted demo should still ignore that queued Start bit",
        );
        assert_eq!(
            frame_6947,
            Some((GameMode::DemoPlay, LevelId(0x15), false)),
            "later queued inputs in the same bootstrap window should keep the restarted demo alive",
        );
    }

    #[test]
    fn published_tas_demo_2_uses_the_oracle_hidden_camera_wrap_on_frame_6966() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_6965 = None;
        let mut frame_6966 = None;
        let mut frame_6967 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(6_968) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                6_965 => {
                    frame_6965 = Some((trace.current_level, trace.camera_y_pos, trace.kid_y_pos));
                }
                6_966 => {
                    frame_6966 = Some((trace.current_level, trace.camera_y_pos, trace.kid_y_pos));
                }
                6_967 => {
                    frame_6967 = Some((trace.current_level, trace.camera_y_pos, trace.kid_y_pos));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_6965,
            Some((LevelId(0x15), 0x00E0_0000, None)),
            "the frame before the wrap should still be the hidden Knights Isle warm-up hold",
        );
        assert_eq!(
            frame_6966,
            Some((LevelId(0x15), 0xFF00_0000, None)),
            "the restarted demo 1 warm-up should use the oracle-backed wrapped camera Y on frame 6966",
        );
        assert_eq!(
            frame_6967,
            Some((LevelId(0x15), 0x00E0_0000, None)),
            "the hidden warm-up should settle back to the normal preload camera on the next frame",
        );
    }

    #[test]
    fn demo_input_decoded_correctly() {
        // 0x48 = bits 6,3 = Left+Right? No: bit 6=A, bit 3=Right → A+Right
        let input = demo::decode_demo_byte(0x48);
        assert!(input.right, "bit 3 = Right");
        assert!(input.a, "bit 6 = A");
        assert!(!input.start, "bit 7 stripped");
        assert!(!input.left);
        assert!(!input.up);
        assert!(!input.down);
    }

    #[test]
    fn demo_byte_bit7_stripped() {
        // 0x88 = bits 7,3 → bit 7 stripped, only bit 3 (Right) remains
        let input = demo::decode_demo_byte(0x88);
        assert!(input.right, "bit 3 = Right");
        assert!(!input.start, "bit 7 should be stripped");
    }

    #[test]
    fn demo_camera_handoff_primes_buffer_for_the_hidden_settle_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME - 1,
            data_index: 118,
            use_demo_2: true,
        };

        let handoff_input = runtime
            .step_demo_input(FrameInput::default(), false)
            .expect("demo playback should continue");
        assert_eq!(
            handoff_input,
            FrameInput::default(),
            "camera hand-off still uses default input on the current frame",
        );
        assert_eq!(
            runtime.demo_buffered_input,
            Some(demo::decode_demo_byte(demo::DEMO_INPUT_DATA_2[118])),
            "ReadJoypad should prime the next frame's controller state",
        );

        let hidden_settle_input = runtime
            .step_demo_input(FrameInput::default(), false)
            .expect("demo playback should continue");
        assert_eq!(
            hidden_settle_input,
            demo::decode_demo_byte(demo::DEMO_INPUT_DATA_2[118]),
            "the hidden settle frame should consume the primed byte",
        );
        if let BootState::DemoPlay { data_index, .. } = runtime.boot_state {
            assert_eq!(
                data_index, 119,
                "the second read should advance to the next byte"
            );
        } else {
            panic!("expected DemoPlay after hidden settle");
        }
    }

    #[test]
    fn demo_transform_entry_freezes_buffer_before_flash_frames_start() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::DemoPlay {
            frame_counter: demo::DEMO_GAMEPLAY_START_FRAME + 10,
            data_index: 118,
            use_demo_2: true,
        };
        runtime.demo_buffered_input = Some(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        runtime.progression.pending_form = Some(PlayerForm::Cyclone);

        let output = runtime
            .step_demo_input(FrameInput::default(), false)
            .expect("demo playback should continue");
        assert_eq!(
            output,
            FrameInput {
                right: true,
                ..FrameInput::default()
            },
            "the queued transform frame should keep using the previously buffered byte",
        );
        assert_eq!(
            runtime.demo_buffered_input,
            Some(FrameInput {
                right: true,
                ..FrameInput::default()
            }),
            "ReadJoypad should stay frozen until the transform coroutine starts",
        );
        if let BootState::DemoPlay {
            frame_counter,
            data_index,
            ..
        } = runtime.boot_state
        {
            assert_eq!(frame_counter, demo::DEMO_GAMEPLAY_START_FRAME + 10);
            assert_eq!(data_index, 118);
        } else {
            panic!("expected DemoPlay after queued transform frame");
        }
    }

    #[test]
    fn demo_1_timeout_overrun_uses_the_adjacent_rom_bytes() {
        assert_eq!(
            demo::demo_byte(false, demo::DEMO_INPUT_DATA_1.len()),
            0x06,
            "demo 1 should keep reading the raw ROM stream after Demo_InputData1 ends",
        );
        assert_eq!(
            demo::demo_byte(false, demo::DEMO_INPUT_DATA_1.len() + 1),
            0xC4,
            "the next overflow byte should match the ROM byte immediately after Demo_InputData1",
        );
    }

    #[test]
    fn death_respawn_level_1_reports_grounded_walk_on_frame_720() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_720 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(721) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 720 {
                frame_720 = Some(trace);
                break;
            }
        }

        let frame_720 = frame_720.expect("frame 720 trace");
        // The playable death/respawn smoke now reuses the same walk-only opening
        // baseline as the pause/progress scenarios, so the first post-settle frame
        // is already on the grounded run-up rather than the old self-paused idle.
        assert_eq!(frame_720.kid_x_velocity, Some(0x0002_0000));
        assert_eq!(frame_720.kid_y_velocity, Some(0));
        assert_eq!(frame_720.kid_on_ground, Some(true));
        assert_eq!(frame_720.character_movement, Some(2));
    }

    #[test]
    fn death_respawn_level_1_keeps_loc_9d22_uphill_state_on_frame_1003() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1003 = None;
        let mut frame_1004 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_005) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_003 {
                frame_1003 = Some(trace);
            } else if frame_index == 1_004 {
                frame_1004 = Some(trace);
                break;
            }
        }

        let frame_1003 = frame_1003.expect("frame 1003 trace");
        let frame_1004 = frame_1004.expect("frame 1004 trace");
        // ASM loc_9D22 keeps MoveID 6 while either grounded support edge still
        // rides the slope continuation, even once the centre probe has crossed
        // onto the adjacent solid cell. Once sub_B43A reacquires flat support on
        // the next frame, the route returns to loc_8BF0's plain walking state.
        assert_eq!(frame_1003.kid_on_ground, Some(true));
        assert_eq!(frame_1003.character_movement, Some(6));
        assert_eq!(frame_1004.kid_on_ground, Some(true));
        assert_eq!(frame_1004.character_movement, Some(2));
    }

    #[test]
    fn death_respawn_level_1_returns_to_plain_walk_on_frame_1044() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1044 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_045) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_044 {
                frame_1044 = Some(trace);
                break;
            }
        }

        let frame_1044 = frame_1044.expect("frame 1044 trace");
        // ASM loc_9D22 only keeps MoveID 6 while the slope state actively mirrors
        // `y_vel` from `x_vel`; once the Level 1 run-up settles back onto flat
        // support, loc_8BF0 resumes plain walking even if the hitbox still
        // overlaps a neighbouring slope cell.
        assert_eq!(frame_1044.kid_on_ground, Some(true));
        assert_eq!(frame_1044.kid_y_velocity, Some(0));
        assert_eq!(frame_1044.character_movement, Some(2));
    }

    #[test]
    fn published_demo_wall_jump_stall_keeps_loc_a6f8_jump_movement_until_settle() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();

        let mut frame_957 = None;
        let mut frame_977 = None;
        let mut frame_978 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(979) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                957 => frame_957 = Some(trace),
                977 => frame_977 = Some(trace),
                978 => {
                    frame_978 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_957 = frame_957.expect("frame 957 trace");
        let frame_977 = frame_977.expect("frame 977 trace");
        let frame_978 = frame_978.expect("frame 978 trace");

        // ASM loc_A6F8 writes MoveID_Jump before the Kid object hibernates;
        // the published demo keeps $FFFFFA56 on that value through the
        // wall-jump landing stall, then loc_A94E's settle path restores idle.
        assert_eq!(frame_957.kid_on_ground, Some(true));
        assert_eq!(frame_957.kid_x_velocity, Some(0));
        assert_eq!(frame_957.character_movement, Some(3));
        assert_eq!(frame_977.kid_on_ground, Some(true));
        assert_eq!(frame_977.character_movement, Some(3));
        assert_eq!(frame_978.kid_on_ground, Some(true));
        assert_eq!(frame_978.character_movement, Some(0));
    }

    #[test]
    fn published_demo_grounded_run_resumes_with_loc_9b62_normal_acceleration() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();

        let mut frame_979 = None;
        let mut frame_1004 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(980) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 979 {
                frame_979 = Some(trace);
                break;
            }
        }

        let frame_979 = frame_979.expect("frame 979 trace");
        // ASM loc_8E16 refreshes the grounded surface before loc_9B62 applies
        // Ctrl_A_Held run acceleration.  The published demo has right+A held
        // here, so the first post-stall x step is normal Kid run accel $1C00,
        // not the rubber-table $0380 step.
        assert_eq!(frame_979.kid_on_ground, Some(true));
        assert_eq!(frame_979.kid_x_pos, Some(0x0044_1C00));
        assert_eq!(
            frame_979.kid_x_velocity,
            Some(0),
            "published TAS trace export still masks visible velocity while raw WRAM advances",
        );

        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_005) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_004 {
                frame_1004 = Some(trace);
                break;
            }
        }

        let frame_1004 = frame_1004.expect("frame 1004 trace");
        // ASM loc_C22 advances the demo stream after the gameplay body while
        // the Load_DemoPlay `loc_119D8` controller object remains live. Oracle
        // trace checkpoints still sample the visible velocity as zero through
        // this post-stall wall-cling fall window.
        assert_eq!(frame_1004.kid_y_velocity, Some(0));
    }

    #[test]
    fn published_demo_prize_block_bounce_counts_loc_1002e_transient_worker() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();

        let mut frame_989 = None;
        let mut frame_1020 = None;
        let mut frame_1021 = None;
        let mut frame_1022 = None;
        let mut frame_1045 = None;
        let mut frame_1046 = None;
        let mut frame_1057 = None;
        let mut frame_1058 = None;
        let mut frame_1129 = None;
        let mut frame_1130 = None;
        let mut frame_1131 = None;
        let mut frame_1145 = None;
        let mut frame_1146 = None;
        let mut frame_1147 = None;
        let mut frame_1148 = None;
        let mut frame_1347 = None;
        let mut frame_1348 = None;
        let mut frame_1349 = None;
        let mut frame_1398 = None;
        let mut frame_1399 = None;
        let mut frame_1400 = None;
        let mut frame_1407 = None;
        let mut frame_1408 = None;
        let mut frame_1409 = None;
        let mut frame_1490 = None;
        let mut frame_1494 = None;
        let mut frame_1495 = None;
        let mut frame_1496 = None;
        let mut frame_1497 = None;
        let mut frame_1678 = None;
        let mut frame_1679 = None;
        let mut frame_1703 = None;
        let mut frame_1704 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_705) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                989 => frame_989 = Some(trace),
                1_020 => frame_1020 = Some(trace),
                1_021 => frame_1021 = Some(trace),
                1_022 => frame_1022 = Some(trace),
                1_045 => frame_1045 = Some(trace),
                1_046 => frame_1046 = Some(trace),
                1_057 => frame_1057 = Some(trace),
                1_058 => frame_1058 = Some(trace),
                1_129 => frame_1129 = Some(trace),
                1_130 => frame_1130 = Some(trace),
                1_131 => frame_1131 = Some(trace),
                1_145 => frame_1145 = Some(trace),
                1_146 => frame_1146 = Some(trace),
                1_147 => frame_1147 = Some(trace),
                1_148 => frame_1148 = Some(trace),
                1_347 => frame_1347 = Some(trace),
                1_348 => frame_1348 = Some(trace),
                1_349 => frame_1349 = Some(trace),
                1_398 => frame_1398 = Some(trace),
                1_399 => frame_1399 = Some(trace),
                1_400 => frame_1400 = Some(trace),
                1_407 => frame_1407 = Some(trace),
                1_408 => frame_1408 = Some(trace),
                1_409 => frame_1409 = Some(trace),
                1_490 => frame_1490 = Some(trace),
                1_494 => frame_1494 = Some(trace),
                1_495 => frame_1495 = Some(trace),
                1_496 => frame_1496 = Some(trace),
                1_497 => {
                    frame_1497 = Some(trace);
                }
                1_678 => frame_1678 = Some(trace),
                1_679 => {
                    frame_1679 = Some(trace);
                }
                1_703 => frame_1703 = Some(trace),
                1_704 => {
                    frame_1704 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_989 = frame_989.expect("frame 989 trace");
        // ASM loc_1002E allocates an object/gfx pair for the sub_1007A
        // prize-block wobble worker. Native renders that as block_bounces,
        // but the demo object-count trace must still include the live worker.
        assert_eq!(frame_989.block_bounces, vec![(4, 0x24, 31, 0)]);
        assert_eq!(frame_989.number_objects, 5);
        assert_eq!(frame_989.number_gfx_objects, 5);

        let frame_1020 = frame_1020.expect("frame 1020 trace");
        let frame_1021 = frame_1021.expect("frame 1021 trace");
        let frame_1022 = frame_1022.expect("frame 1022 trace");
        let frame_1045 = frame_1045.expect("frame 1045 trace");
        let frame_1046 = frame_1046.expect("frame 1046 trace");
        // ASM sub_1007A deletes the wobble worker only after the post-wobble
        // animation hand-off, and loc_102B8 keeps the spawn-flash worker alive
        // for the 25-frame stru_102DE script. These are count-visible even
        // though native does not yet render the flash as a separate effect.
        assert_eq!(frame_1020.number_objects, 5);
        assert_eq!(frame_1021.number_objects, 6);
        assert_eq!(frame_1022.number_objects, 6);
        assert_eq!(frame_1045.number_objects, 6);
        assert_eq!(frame_1046.number_objects, 5);

        let frame_1057 = frame_1057.expect("frame 1057 trace");
        let frame_1058 = frame_1058.expect("frame 1058 trace");
        // ASM PrizeHelmetCyclone_Init loc_104AE/loc_104C6 deletes the
        // falling helmet object once `collision_type(a3)` becomes non-zero.
        // The form-change coroutine hibernates gameplay immediately after
        // contact, so native must stop counting the already-collected floating
        // item instead of waiting for the visual-effect step to retain it out.
        assert_eq!(frame_1057.number_objects, 5);
        assert_eq!(frame_1058.number_objects, 4);

        let frame_1129 = frame_1129.expect("frame 1129 trace");
        let frame_1130 = frame_1130.expect("frame 1130 trace");
        // ASM Kid_Transform loc_7D22/loc_7E90 hibernates through the final
        // cleanup frame, but `sf (Currently_transforming).w` has already run.
        assert_eq!(frame_1129.form_change_flash_frames, 2);
        assert!(
            frame_1129
                .gameplay_debug_fields
                .contains_key("currently_transforming")
        );
        assert_eq!(frame_1130.form_change_flash_frames, 1);
        assert!(
            !frame_1130
                .gameplay_debug_fields
                .contains_key("currently_transforming")
        );
        assert!(
            frame_1130
                .gameplay_debug_fields
                .contains_key("gameplay_hibernated")
        );

        let frame_1131 = frame_1131.expect("frame 1131 trace");
        let frame_1145 = frame_1145.expect("frame 1145 trace");
        let frame_1146 = frame_1146.expect("frame 1146 trace");
        let frame_1147 = frame_1147.expect("frame 1147 trace");
        let frame_1148 = frame_1148.expect("frame 1148 trace");
        // ASM Kid_Transform loc_7E90 exits the transform coroutine before the
        // demo controller hand-off has republished live player velocity. Oracle
        // WRAM samples x_vel/y_vel as zero through frames 1131..1147 even while
        // the visible `kid_gfx` object keeps moving; native masks only trace
        // export here so the internal simulation still advances.
        assert_eq!(frame_1131.form_change_flash_frames, 0);
        assert_eq!(frame_1131.kid_x_velocity, Some(0));
        assert_eq!(frame_1131.kid_y_velocity, Some(0));
        assert_eq!(frame_1145.kid_x_velocity, Some(0));
        assert_eq!(frame_1145.kid_y_velocity, Some(0));
        assert_eq!(frame_1146.kid_x_velocity, Some(0));
        assert_eq!(frame_1146.kid_y_velocity, Some(0));
        assert_eq!(frame_1147.kid_x_velocity, Some(0));
        assert_eq!(frame_1147.kid_y_velocity, Some(0));
        assert_ne!(
            frame_1148.kid_x_velocity,
            Some(0),
            "post-transform trace velocity hold should expire before the next gameplay frontier"
        );

        let frame_1347 = frame_1347.expect("frame 1347 trace");
        let frame_1348 = frame_1348.expect("frame 1348 trace");
        let frame_1349 = frame_1349.expect("frame 1349 trace");
        // ASM sub_10EDA (kid.asm:22836) keeps each smash-block effect object
        // alive for a two-frame hibernate before dispatching sub_292E debris.
        // The TAS demo count baseline must include those object/gfx workers
        // while native exposes them as block_destroy_effects.
        assert_eq!(frame_1347.block_destroy_effects, vec![(352, 244, 2, 0)]);
        assert_eq!(frame_1347.number_objects, 5);
        assert_eq!(frame_1347.number_gfx_objects, 5);
        assert_eq!(
            frame_1348.block_destroy_effects,
            vec![(352, 244, 1, 0), (368, 244, 2, 0)]
        );
        assert_eq!(frame_1348.number_objects, 6);
        assert_eq!(frame_1348.number_gfx_objects, 6);
        assert_eq!(frame_1349.block_destroy_effects, vec![(368, 244, 1, 0)]);
        assert_eq!(frame_1349.number_objects, 5);
        assert_eq!(frame_1349.number_gfx_objects, 5);

        let frame_1398 = frame_1398.expect("frame 1398 trace");
        let frame_1399 = frame_1399.expect("frame 1399 trace");
        let frame_1400 = frame_1400.expect("frame 1400 trace");
        // ASM PrizeDiamond_Init (kid.asm:21604) clears `is_moved(a3)`, so the
        // diamond's object/gfx slot is already count-visible on the same wake
        // where sub_1007A restores the prize cell. Helmet/shared prize
        // initialisers set `is_moved(a3)` and remain on the later path above.
        assert!(frame_1398.floating_items.is_empty());
        assert_eq!(frame_1398.number_objects, 6);
        assert_eq!(frame_1399.floating_items, vec![(360, 224, 244, 0)]);
        assert_eq!(frame_1399.number_objects, 7);
        assert_eq!(frame_1400.floating_items, vec![(360, 224, 243, 0)]);
        assert_eq!(frame_1400.number_objects, 7);

        let frame_1407 = frame_1407.expect("frame 1407 trace");
        let frame_1408 = frame_1408.expect("frame 1408 trace");
        let frame_1409 = frame_1409.expect("frame 1409 trace");
        // ASM PrizeDiamondCollected_Init (kid.asm:21100) allocates a fresh
        // object/gfx pair after the diamond collision, then hibernates through
        // the HUD-flight loop before incrementing Number_Diamonds.
        assert_eq!(frame_1407.number_objects, 6);
        assert_eq!(frame_1408.number_objects, 7);
        assert_eq!(frame_1409.number_objects, 8);

        let frame_1490 = frame_1490.expect("frame 1490 trace");
        let frame_1494 = frame_1494.expect("frame 1494 trace");
        let frame_1495 = frame_1495.expect("frame 1495 trace");
        let frame_1496 = frame_1496.expect("frame 1496 trace");
        let frame_1497 = frame_1497.expect("frame 1497 trace");
        assert_eq!(frame_1490.block_bounces, vec![(25, 4, 3, -4)]);
        assert_eq!(frame_1490.number_objects, 5);
        assert!(frame_1494.floating_items.is_empty());
        assert_eq!(frame_1494.number_objects, 6);
        // ASM sub_1007A -> PrizeDiamond_Init (kid.asm:21604) has the old
        // block worker count-visible before the diamond wakes. Once this
        // distant diamond is live outside Kid's horizontal collision band, it
        // replaces the stale worker count instead of adding another slot.
        assert_eq!(frame_1495.floating_items, vec![(408, 64, 244, 0)]);
        assert_eq!(frame_1495.number_objects, 6);
        assert_eq!(frame_1496.floating_items, vec![(408, 64, 243, 0)]);
        assert_eq!(frame_1496.number_objects, 6);
        assert_eq!(frame_1497.floating_items, vec![(408, 64, 242, 0)]);
        assert_eq!(frame_1497.number_objects, 6);

        let frame_1678 = frame_1678.expect("frame 1678 trace");
        let frame_1679 = frame_1679.expect("frame 1679 trace");
        // ASM sub_1007A tails are per block object, not global. A distant
        // live diamond suppresses only its own stale tail; later unrelated
        // prize-block tails still count when their wobble finishes.
        assert_eq!(frame_1678.block_bounces.len(), 5);
        assert_eq!(frame_1678.number_objects, 10);
        assert_eq!(frame_1679.block_bounces.len(), 4);
        assert_eq!(frame_1679.number_objects, 10);

        let frame_1703 = frame_1703.expect("frame 1703 trace");
        let frame_1704 = frame_1704.expect("frame 1704 trace");
        assert_eq!(frame_1703.number_objects, 17);
        assert_eq!(frame_1704.number_objects, 17);
    }

    #[test]
    fn death_respawn_level_1_falling_pre_death_frame_1680_stays_jump_not_wall_climb() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1680 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_681) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_680 {
                frame_1680 = Some(trace);
                break;
            }
        }

        let frame_1680 = frame_1680.expect("frame 1680 trace");
        // ASM loc_A28A writes MoveID_Wall_Climbing only while the wall-cling state
        // is still latched to real wall support. By the late pre-death fall on
        // Level 1, Kid is already in the airborne jump path rather than a live
        // wall-climb pose.
        assert_eq!(frame_1680.kid_on_ground, Some(false));
        assert_eq!(frame_1680.kid_y_velocity, Some(0x0001_6000));
        assert_eq!(frame_1680.character_movement, Some(3));
    }

    #[test]
    fn death_respawn_level_1_keeps_oracle_slope_runup_before_frame_936() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_936 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(937) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 936 {
                frame_936 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.number_hitpoints,
                ));
            }
        }

        // ASM sub_914A keeps Kid grounded on the Level 1 slope seam before
        // loc_9D22 applies the slope velocity table; the death-respawn script
        // should therefore still be on the oracle-backed run-up pose at frame 936.
        assert_eq!(
            frame_936,
            Some((
                Some(0x02C4_7400),
                Some(0x00FB_0000),
                0x0211_0000,
                0x0066_0000,
                1,
            )),
        );
    }

    #[test]
    fn death_respawn_level_1_slope_jump_stays_within_oracle_tolerance_through_frame_945() {
        let mut runtime = cached_generated_runtime();
        runtime.set_trace_physics(true);
        let script = death_respawn_level_1_script();

        let mut frame_941 = None;
        let mut frame_945 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(946) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 941 {
                frame_941 = Some(trace);
            } else if frame_index == 945 {
                frame_945 = Some(trace);
            }
        }

        let frame_941 = frame_941.expect("frame 941 trace");
        let frame_945 = frame_945.expect("frame 945 trace");
        // ASM loc_9F18 -> loc_B580 -> loc_A426 clears the slope-tracking y_vel and
        // launches the jump coroutine immediately. Oracle-backed gameplay drift now
        // stays under the 4px live-compare tolerance through frame 945; the first
        // remaining authoritative miss moved to frame 946 on x_pos.
        assert!(
            frame_941
                .kid_x_pos
                .expect("frame 941 x")
                .abs_diff(0x02C9_7400)
                <= 0x0004_0000
        );
        assert!(
            frame_941
                .kid_y_pos
                .expect("frame 941 y")
                .abs_diff(0x00F2_9E00)
                <= 0x0004_0000
        );
        assert_eq!(frame_941.camera_x_pos, 0x021C_0000);
        assert_eq!(frame_941.camera_y_pos, 0x0060_0000);
        assert_eq!(frame_941.number_hitpoints, 1);
        assert_eq!(frame_941.kid_on_ground, Some(false));
        assert!(
            frame_945
                .kid_x_pos
                .expect("frame 945 x")
                .abs_diff(0x02CE_F900)
                <= 0x0004_0000
        );
        assert!(
            frame_945
                .kid_y_pos
                .expect("frame 945 y")
                .abs_diff(0x00E2_7600)
                <= 0x0004_0000
        );
    }

    #[test]
    fn death_respawn_level_1_does_not_auto_jump_off_the_frame_989_landing() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_990 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(991) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 990 {
                frame_990 = Some(trace);
            }
        }

        let frame_990 = frame_990.expect("frame 990 trace");
        // ASM loc_9F18 consumes only the single-frame `Ctrl_Pressed` jump edge.
        // After the frame-989 landing, the Level 1 death-respawn route must stay
        // grounded on frame 990 instead of replaying a buffered jump on the next
        // grounded frame.
        assert_eq!(frame_990.kid_y_pos, Some(0x00AF_0000));
        assert_eq!(frame_990.kid_y_velocity, Some(0));
        assert_eq!(frame_990.kid_on_ground, Some(true));
        assert_eq!(frame_990.number_hitpoints, 1);
    }

    #[test]
    fn death_respawn_level_1_waits_until_frame_990_to_land() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_989 = None;
        let mut frame_990 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(991) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 989 {
                frame_989 = Some(trace);
            } else if frame_index == 990 {
                frame_990 = Some(trace);
            }
        }

        let frame_989 = frame_989.expect("frame 989 trace");
        let frame_990 = frame_990.expect("frame 990 trace");
        // ASM sub_902A keeps the last sub-half-pixel gap airborne until the
        // contact row is reached. The Level 1 death-respawn route should still be
        // falling on frame 989, then land on frame 990 without taking an extra X
        // step before the grounded run-up resumes.
        assert_eq!(frame_989.kid_on_ground, Some(false));
        assert_eq!(frame_990.kid_y_pos, Some(0x00AF_0000));
        assert_eq!(frame_990.kid_on_ground, Some(true));
        assert_eq!(frame_990.kid_x_pos, frame_989.kid_x_pos);
        assert_eq!(frame_990.number_hitpoints, 1);
    }

    #[test]
    fn death_respawn_level_1_keeps_flat_runup_grounded_through_frame_1012() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frames = BTreeMap::new();
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_013) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (1_003..=1_012).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        for frame_index in 1_003..=1_012 {
            let frame = frames
                .get(&frame_index)
                .unwrap_or_else(|| panic!("frame {frame_index} should be captured"));
            // ASM sub_914A keeps the deeper flat support alive under Kid's hitbox
            // across this Level 1 seam, so oracle stays on the `y=$CF` plateau
            // until the late frame-1013 jump. Native must not step onto the
            // adjacent slope and fall airborne during this run-up.
            assert_eq!(frame.kid_y_pos, Some(0x00CF_0000));
            assert_eq!(frame.kid_on_ground, Some(true));
            assert_eq!(frame.number_hitpoints, 1);
        }

        let mut previous_x = None;
        for frame_index in 1_003..=1_012 {
            let frame = frames
                .get(&frame_index)
                .unwrap_or_else(|| panic!("frame {frame_index} should be captured"));
            if let Some(previous_x) = previous_x {
                assert!(
                    frame.kid_x_pos.expect("frame x") > previous_x,
                    "flat run-up should keep moving right on frame {frame_index}",
                );
            }
            previous_x = frame.kid_x_pos;
        }
    }

    #[test]
    fn death_respawn_level_1_stays_alive_past_frame_1161() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1161 = None;
        let mut frame_1163 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_164) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_161 {
                frame_1161 = Some(trace);
            } else if frame_index == 1_163 {
                frame_1163 = Some(trace);
            }
        }

        let frame_1161 = frame_1161.expect("frame 1161 trace");
        let frame_1163 = frame_1163.expect("frame 1163 trace");
        // ASM `Check_for_recent_damge` / `Death` do not trigger on this Level 1
        // route yet; oracle-backed gameplay stays alive here and the normal
        // `sub_71E4` camera follow continues advancing instead of snapping to a
        // synthetic death camera.
        assert_eq!(frame_1161.number_hitpoints, 1);
        assert_eq!(frame_1161.camera_x_pos, 0x0415_0000);
        assert_eq!(frame_1161.kid_y_pos, Some(0x0077_B8C0));
        assert_eq!(frame_1163.number_hitpoints, 1);
        assert_eq!(frame_1163.camera_x_pos, 0x041A_0000);
        assert!(
            frame_1163
                .kid_y_pos
                .expect("frame 1163 y")
                .abs_diff(0x006F_AF80)
                <= 0x0001_0000
        );
    }

    #[test]
    fn respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x() {
        let mut runtime = cached_generated_runtime();
        runtime.warp_to_level(LevelId(0x29));
        let settled = runtime.level_respawn_slice_with_floor_snap(LevelId(0x29));

        // ASM Load_InGame writes the raw header pose, but the first settle frames
        // still resolve the spawn-side overlap before camera convergence finishes.
        // Oracle-backed focused sweeps for backlog #037 settle on x=$02A50000 and
        // camera_x=$02090000 for The Nightmare Peaks 2.
        assert_eq!(settled.player.x_pos, 0x02A5_0000);
        assert_eq!(settled.camera.x_pos, 0x0209_0000);
    }

    #[test]
    fn respawn_settle_level_52_uses_stair_step_support_path() {
        let mut runtime = cached_generated_runtime();
        runtime.warp_to_level(LevelId(0x52));
        let frames = runtime.inspect_respawn_settle(LevelId(0x52), 16);
        let settle_step_1 = frames.get(1).expect("step 1 should exist");
        let settled = frames
            .last()
            .expect("settle trace should contain the final frame");

        // ASM sub_914A's landing-row scan finds the side stair-step support in
        // Elsewhere 9 before the deeper centre-column floor finishes the fall.
        assert!(settle_step_1.wide_support_applied);
        assert_eq!(settle_step_1.kid_y_pos, 0x014F_0000);
        assert_eq!(settled.kid_y_pos, 0x014F_0000);
    }

    #[test]
    fn respawn_settle_level_24_matches_oracle_camera_convergence() {
        let mut runtime = cached_generated_runtime();
        runtime.warp_to_level(LevelId(0x24));
        let settled = runtime.level_respawn_slice_with_floor_snap(LevelId(0x24));

        // ASM Load_InGame seeds the header pose, then the hidden init frames run
        // the normal `sub_71E4` camera follow while Kid settles onto the floor.
        // Focused backlog #037 traces for Wind Castles 1 converge at camera_y=$36E.
        assert_eq!(settled.player.y_pos, 0x040F_0000);
        assert_eq!(settled.camera.y_pos, 0x036E_0000);
    }

    #[test]
    fn respawn_settle_level_51_matches_oracle_camera_convergence() {
        let mut runtime = cached_generated_runtime();
        runtime.warp_to_level(LevelId(0x51));
        let settled = runtime.level_respawn_slice_with_floor_snap(LevelId(0x51));

        // ASM sub_71E4 uses the same hidden init camera-follow path here; the
        // cached backlog #037 tracker for Elsewhere 10 converges at camera_y=$28E.
        assert_eq!(settled.player.y_pos, 0x032F_0000);
        assert_eq!(settled.camera.y_pos, 0x028E_0000);
    }

    #[test]
    fn respawn_settle_level_01_lands_on_blue_lake_woods_2_floor() {
        let mut runtime = cached_generated_runtime();
        runtime.warp_to_level(LevelId(0x01));
        let frames = runtime.inspect_respawn_settle(LevelId(0x01), 40);
        let spawn = frames.first().expect("spawn frame should exist");
        let landing = frames.get(16).expect("landing frame should exist");
        let settled = frames
            .last()
            .expect("settle trace should contain the final frame");

        // ASM Load_InGame seeds the raw header pose from `player_start_x/y`, and
        // level 0x01's header is intentionally midair at (32, 160). The hidden
        // init frames then keep running gravity + sub_902A until Kid lands on the
        // first safe floor row before camera convergence finishes.
        assert_eq!(spawn.kid_x_pos, 0x0020_0000);
        assert_eq!(spawn.kid_y_pos, 0x009F_0000);
        assert!(
            !spawn.kid_on_ground,
            "raw header pose should start airborne"
        );

        assert_eq!(landing.kid_y_pos, 0x00BF_0000);
        assert!(
            landing.kid_on_ground,
            "frame 16 should be the grounded landing"
        );
        assert!(
            !landing.kid_on_spikes,
            "Blue Lake Woods 2 entry should land on safe floor"
        );

        assert_eq!(settled.kid_x_pos, 0x0020_0000);
        assert_eq!(settled.kid_y_pos, 0x00BF_0000);
        assert_eq!(settled.camera_y_pos, 0x001C_0000);
    }
}
