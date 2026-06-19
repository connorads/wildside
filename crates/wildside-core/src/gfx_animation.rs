#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GfxAnimationState {
    pub(crate) timer: u16,
    pub(crate) sprite_offset: u16,
    pub(crate) script_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GfxAnimationDispatchLoc1158 {
    StopAnimation,
    AdvanceNormally,
    RewindScriptPointer,
}

#[allow(dead_code)]
pub(crate) fn decode_gfx_animation_dispatch_loc_1158(
    script: &[u8],
    script_offset: usize,
) -> Option<(GfxAnimationDispatchLoc1158, usize)> {
    let opcode = *script.get(script_offset)?;
    let dispatch = match opcode {
        // ASM loc_1158 (kid.asm:2124) reads one byte from `(a2)+`, multiplies
        // it by four, and indexes `off_1176`.
        0 => GfxAnimationDispatchLoc1158::StopAnimation,
        1 => GfxAnimationDispatchLoc1158::AdvanceNormally,
        2 => GfxAnimationDispatchLoc1158::RewindScriptPointer,
        _ => return None,
    };

    Some((dispatch, script_offset.checked_add(1)?))
}

#[allow(dead_code)]
pub(crate) fn should_stop_gfx_animation_loc_1166(
    timer: u16,
    script: &[u8],
    script_offset: usize,
) -> Option<bool> {
    let next_opcode = *script.get(script_offset)?;

    // ASM loc_1166 (kid.asm:2133) only reaches loc_1182 when the decremented
    // timer equals 1 and the next script byte is 0. Any other state restores
    // a2 and returns with animation still running.
    Some(timer == 1 && next_opcode == 0)
}

#[allow(dead_code)]
pub(crate) fn advance_gfx_animation_sub_1194(
    script: &[u8],
    script_offset: usize,
    state: &mut GfxAnimationState,
) -> Option<()> {
    let timer = u16::from(*script.get(script_offset)?);
    let sprite_offset = u16::from_be_bytes([
        *script.get(script_offset.checked_add(1)?)?,
        *script.get(script_offset.checked_add(2)?)?,
    ]);

    // ASM sub_1194 (kid.asm:2161): read timer byte + sprite word, store the
    // advanced script pointer, then tail-call back into GfxObject_Animate.
    state.timer = timer;
    state.sprite_offset = sprite_offset;
    state.script_offset = script_offset.checked_add(3)?;
    Some(())
}

#[allow(dead_code)]
pub(crate) fn rewind_gfx_animation_pointer_sub_11ac(
    current_byte: u8,
    script_offset: usize,
) -> Option<usize> {
    let rewind = i8::from_be_bytes([current_byte]);

    // ASM sub_11AC (kid.asm:2176): sign-extend the current byte and subtract it
    // from a2, so positive values rewind while negative values move forward.
    script_offset.checked_add_signed(-(isize::from(rewind)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_1194_loads_timer_sprite_and_advanced_pointer() {
        let script = [0x05, 0x12, 0x34, 0xFF];
        let mut state = GfxAnimationState {
            timer: 0,
            sprite_offset: 0,
            script_offset: 0,
        };

        advance_gfx_animation_sub_1194(&script, 0, &mut state)
            .expect("ASM sub_1194 should accept a complete timer+sprite record");

        assert_eq!(state.timer, 5);
        assert_eq!(state.sprite_offset, 0x1234);
        assert_eq!(
            state.script_offset, 3,
            "ASM sub_1194 stores a2 after consuming the timer byte and sprite word"
        );
    }

    #[test]
    fn sub_1194_rejects_truncated_timer_sprite_record() {
        let script = [0x05, 0x12];
        let mut state = GfxAnimationState {
            timer: 7,
            sprite_offset: 0xAAAA,
            script_offset: 9,
        };

        assert!(
            advance_gfx_animation_sub_1194(&script, 0, &mut state).is_none(),
            "ASM sub_1194 needs one byte of timer and one word of sprite data"
        );
        assert_eq!(state.timer, 7);
        assert_eq!(state.sprite_offset, 0xAAAA);
        assert_eq!(state.script_offset, 9);
    }

    #[test]
    fn loc_1158_decodes_stop_animation_dispatch_and_advances_script_pointer() {
        let (dispatch, next_offset) = decode_gfx_animation_dispatch_loc_1158(&[0x00, 0xFF], 0)
            .expect("ASM loc_1158 should accept `off_1176` selector 0 for loc_1182");

        assert_eq!(dispatch, GfxAnimationDispatchLoc1158::StopAnimation);
        assert_eq!(
            next_offset, 1,
            "ASM loc_1158 post-increments a2 after reading the dispatch byte"
        );
    }

    #[test]
    fn loc_1158_decodes_normal_animation_and_rewind_dispatch_entries() {
        let (normal, normal_offset) = decode_gfx_animation_dispatch_loc_1158(&[0x01], 0)
            .expect("ASM loc_1158 should map selector 1 to sub_1194");
        let (rewind, rewind_offset) = decode_gfx_animation_dispatch_loc_1158(&[0x02], 0)
            .expect("ASM loc_1158 should map selector 2 to sub_11AC");

        assert_eq!(normal, GfxAnimationDispatchLoc1158::AdvanceNormally);
        assert_eq!(normal_offset, 1);
        assert_eq!(rewind, GfxAnimationDispatchLoc1158::RewindScriptPointer);
        assert_eq!(rewind_offset, 1);
    }

    #[test]
    fn loc_1158_rejects_dispatch_selectors_outside_off_1176() {
        assert!(
            decode_gfx_animation_dispatch_loc_1158(&[0x03], 0).is_none(),
            "ASM loc_1158 only has three handlers in off_1176: loc_1182, sub_1194, and sub_11AC"
        );
    }

    #[test]
    fn loc_1166_only_stops_animation_on_timer_one_with_stop_opcode() {
        assert_eq!(
            should_stop_gfx_animation_loc_1166(1, &[0x00], 0),
            Some(true),
            "ASM loc_1166 only branches to loc_1182 when `animation_timer(a3)` is 1 and `(a2)` is 0"
        );
    }

    #[test]
    fn loc_1166_keeps_animation_running_for_non_terminal_timer_or_opcode() {
        assert_eq!(
            should_stop_gfx_animation_loc_1166(2, &[0x00], 0),
            Some(false),
            "ASM loc_1166 returns through loc_1172 while the decremented timer is not yet 1"
        );
        assert_eq!(
            should_stop_gfx_animation_loc_1166(1, &[0x01], 0),
            Some(false),
            "ASM loc_1166 also returns through loc_1172 when the next opcode is non-zero"
        );
    }

    #[test]
    fn loc_1166_requires_a_script_byte_to_probe_the_stop_opcode() {
        assert!(
            should_stop_gfx_animation_loc_1166(1, &[], 0).is_none(),
            "ASM loc_1166 tests `(a2)`, so the native helper should reject a missing script byte"
        );
    }

    #[test]
    fn sub_11ac_rewinds_by_unsigned_distance_byte() {
        let next_offset = rewind_gfx_animation_pointer_sub_11ac(0x04, 12)
            .expect("ASM sub_11AC should rewind a2 by the current byte");

        assert_eq!(next_offset, 8);
    }

    #[test]
    fn sub_11ac_treats_high_bytes_as_signed_forward_jumps() {
        let next_offset = rewind_gfx_animation_pointer_sub_11ac(0xFC, 12)
            .expect("ASM sub_11AC sign-extends the rewind byte before subtracting it");

        assert_eq!(
            next_offset, 16,
            "0xFC extends to -4, so subtracting it moves the script pointer forward by 4"
        );
    }
}
