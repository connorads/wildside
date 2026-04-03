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
}
