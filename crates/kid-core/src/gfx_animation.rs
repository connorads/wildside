#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GfxAnimationState {
    pub(crate) timer: u16,
    pub(crate) sprite_offset: u16,
    pub(crate) script_offset: usize,
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
