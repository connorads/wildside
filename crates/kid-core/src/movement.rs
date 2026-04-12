use super::*;
use crate::level::fade_palette_span_sub_1C85E;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence3Descriptor {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) raw_flags: u16,
}

const INTRO_SEQUENCE_3_DESCRIPTORS: [IntroSequence3Descriptor; 14] = [
    IntroSequence3Descriptor {
        x_pos: 0x00F8,
        y_pos: 0x00D0,
        raw_flags: 0x1209,
    },
    IntroSequence3Descriptor {
        x_pos: 0x00E3,
        y_pos: 0x00CC,
        raw_flags: 0x1229,
    },
    IntroSequence3Descriptor {
        x_pos: 0x0040,
        y_pos: 0x00CC,
        raw_flags: 0x1231,
    },
    IntroSequence3Descriptor {
        x_pos: 0x0060,
        y_pos: 0x00D0,
        raw_flags: 0x120D,
    },
    IntroSequence3Descriptor {
        x_pos: 0x00D0,
        y_pos: 0x00C8,
        raw_flags: 0x1211,
    },
    IntroSequence3Descriptor {
        x_pos: 0x00C4,
        y_pos: 0x00CC,
        raw_flags: 0x1215,
    },
    IntroSequence3Descriptor {
        x_pos: 0x0070,
        y_pos: 0x00CC,
        raw_flags: 0x1219,
    },
    IntroSequence3Descriptor {
        x_pos: 0x0098,
        y_pos: 0x00C4,
        raw_flags: 0x121C,
    },
    IntroSequence3Descriptor {
        x_pos: 0x00E4,
        y_pos: 0x00C8,
        raw_flags: 0x1221,
    },
    IntroSequence3Descriptor {
        x_pos: 0x0050,
        y_pos: 0x00C8,
        raw_flags: 0x122D,
    },
    IntroSequence3Descriptor {
        x_pos: 0x0024,
        y_pos: 0x00CC,
        raw_flags: 0x1235,
    },
    IntroSequence3Descriptor {
        x_pos: 0x003C,
        y_pos: 0x00C8,
        raw_flags: 0x1239,
    },
    IntroSequence3Descriptor {
        x_pos: 0x0114,
        y_pos: 0x00CA,
        raw_flags: 0x123D,
    },
    IntroSequence3Descriptor {
        x_pos: 0x002C,
        y_pos: 0x00C8,
        raw_flags: 0x1225,
    },
];
const INTRO_SEQUENCE_3_ROSTERS: [[u8; 8]; 4] = [
    [0, 2, 3, 4, 6, 7, 9, 10],
    [0, 5, 6, 7, 8, 12, 0xFF, 0xFF],
    [1, 7, 8, 10, 11, 13, 0xFF, 0xFF],
    [1, 2, 3, 5, 7, 8, 9, 0xFF],
];
const INTRO_SEQUENCE_3_TRACE_DESCRIPTOR_SLOT: usize = 2;
const LEVEL_ENTRY_TITLE_SLOT_2_INDEX: usize = 2;
const LEVEL_ENTRY_TITLE_ANIMATION_FRAMES: usize = 64;
pub(crate) const LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME: usize = 43;
pub(crate) const LEVEL_ENTRY_TITLE_DEFERRED_NUMBER_VISIBLE_FRAME: usize =
    LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME + 1;
// ASM Load_TitleCard allocates loc_1AB26/loc_1AE2E immediately, but oracle
// boot traces do not let the queued Start edge reach visible gameplay until
// global frame 520 after the frame-178 title-card hand-off. Keep the shared
// title-card lineage on the hidden bootstrap for 342 local frames so slot-2
// stays hidden at frame 200 and only the title shell appears on frame 504.
pub(crate) const LEVEL_ENTRY_TITLE_GAMEPLAY_HANDOFF_FRAMES: usize = 342;
// ASM loc_1AE2E only owns the later `Game_Mode=$000C` write; oracle-backed
// gameplay captures show the hidden title-card bootstrap starts exposing the
// settled gameplay slice nine local frames earlier once live movement input is
// already being held.
pub(crate) const LEVEL_ENTRY_TITLE_ACTIVE_INPUT_HANDOFF_FRAMES: usize = 333;

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn intro_sequence_3_descriptor_loc_1AEBA(
    roster_index: usize,
    descriptor_slot: usize,
) -> Option<IntroSequence3Descriptor> {
    // ASM loc_1AEBA (kid.asm:29097): read the next roster byte from
    // `unk_1AF78`, stop immediately when it is negative (`$FF` sentinel), and
    // otherwise scale the descriptor index by six bytes into `unk_1AF24`.
    let descriptor_index =
        INTRO_SEQUENCE_3_ROSTERS[roster_index % INTRO_SEQUENCE_3_ROSTERS.len()][descriptor_slot];
    if descriptor_index == u8::MAX {
        None
    } else {
        Some(INTRO_SEQUENCE_3_DESCRIPTORS[usize::from(descriptor_index)])
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn intro_sequence_3_allocation_spec_loc_1AEE2(
    descriptor: IntroSequence3Descriptor,
) -> u32 {
    // ASM loc_1AEBA / loc_1AEE2 (kid.asm:29106-29115): `btst #0,5(a0)` checks
    // bit 0 of the descriptor flag byte and switches the raw
    // `Allocate_GfxObjectSlot_a1` spec between `$01FF0000` and `$02010000`.
    if descriptor.raw_flags & 1 == 0 {
        0x01FF_0000
    } else {
        0x0201_0000
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence3GfxObjectLoc1AEEC {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) display_enabled: bool,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
    pub(crate) priority: u8,
    pub(crate) x_direction: u8,
    pub(crate) addroffset_sprite: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn intro_sequence_3_x_direction_loc_1AF12(raw_flags: u16) -> u8 {
    // ASM loc_1AF12 (kid.asm:29137): `bclr #1,d0` clears descriptor bit 1 and
    // `sne $16(a1)` stores `$FF` only when that bit was previously set.
    if raw_flags & 0b10 == 0 { 0 } else { u8::MAX }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn intro_sequence_3_gfx_object_loc_1AEEC(
    descriptor: IntroSequence3Descriptor,
) -> IntroSequence3GfxObjectLoc1AEEC {
    // ASM loc_1AEEC (kid.asm:29117): copy the descriptor X/Y words into
    // `$1A(a1)`/`$1E(a1)`, set display flag `$13(a1)`, palette line 1, and
    // VRAM tile `$01D6`, then clear bits 0-1 out of the descriptor flag word
    // while routing bit 0 into priority `$10(a1)`. ASM loc_1AF12 then clears
    // bit 1 and routes the old carry into `x_direction(a1)` via `sne`.
    let priority = if descriptor.raw_flags & 1 == 0 { 0 } else { 1 };
    IntroSequence3GfxObjectLoc1AEEC {
        x_pos: descriptor.x_pos,
        y_pos: descriptor.y_pos,
        display_enabled: true,
        palette_line: 1,
        vram_tile: 0x01D6,
        priority,
        x_direction: intro_sequence_3_x_direction_loc_1AF12(descriptor.raw_flags),
        addroffset_sprite: descriptor.raw_flags & !0b11,
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence5ParticleDataLoc1BE3C {
    pub(crate) sprite_label: u16,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) x_vel: u16,
    pub(crate) y_vel: u16,
    pub(crate) loop_count: u16,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence5ParticleVelocityLoc1BE74 {
    pub(crate) gravity_add: u32,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence5CenterParticleLoc1BE88 {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) x_vel: u32,
    pub(crate) y_vel: u32,
    pub(crate) vram_tile: u16,
    pub(crate) sprite_label: &'static str,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn intro_sequence_5_particle_data_loc_1BE3C() -> [IntroSequence5ParticleDataLoc1BE3C; 3]
{
    // ASM loc_1BE3C (kid.asm:30667-30681): Load particle sprite from data
    // table referenced by $44(a5). The three unk_ tables each contain:
    //   dc.b sprite_label_word_hi, sprite_label_word_lo
    //   dc.b y_pos_hi, y_pos_lo
    //   dc.b x_pos_hi, x_pos_lo
    //   dc.b y_vel_hi, y_vel_lo
    //   dc.b x_vel_hi, x_vel_lo
    //   dc.b loop_count_hi, loop_count_lo
    // See unk_1BE18, unk_1BE24, unk_1BE30 (kid.asm:30629-30664).
    [
        IntroSequence5ParticleDataLoc1BE3C {
            sprite_label: 0x0011, // unk_1BE18 byte 0-1
            x_pos: 0x0078,        // unk_1BE18 bytes 4-5
            y_pos: 0x0000,        // unk_1BE18 byte 2
            x_vel: 0xFFFC,        // unk_1BE18 bytes 8-9 (word)
            y_vel: 0xFFFE,        // unk_1BE18 bytes 10-11 (word)
            loop_count: 0x0028,   // unk_1BE18 bytes 6-7 (word)
        },
        IntroSequence5ParticleDataLoc1BE3C {
            sprite_label: 0x0011, // unk_1BE24 byte 0-1
            x_pos: 0x0058,        // unk_1BE24 bytes 4-5
            y_pos: 0x0000,        // unk_1BE24 byte 2
            x_vel: 0x0004,        // unk_1BE24 bytes 8-9
            y_vel: 0xFFFE,        // unk_1BE24 bytes 10-11
            loop_count: 0x003C,   // unk_1BE24 bytes 6-7
        },
        IntroSequence5ParticleDataLoc1BE3C {
            sprite_label: 0x0011, // unk_1BE30 byte 0-1
            x_pos: 0x0030,        // unk_1BE30 bytes 4-5
            y_pos: 0x0000,        // unk_1BE30 byte 2
            x_vel: 0x0000,        // unk_1BE30 bytes 8-9
            y_vel: 0xFF04,        // unk_1BE30 bytes 10-11
            loop_count: 0x0028,   // unk_1BE30 bytes 6-7
        },
    ]
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn intro_sequence_5_particle_fall_loc_1BE74() -> IntroSequence5ParticleVelocityLoc1BE74 {
    // ASM loc_1BE74 (kid.asm:30682-30686): Hibernate 1 frame, add $4000 to
    // y_vel (16.16 fixed-point gravity of 0.25 px/frame), loop d0 times.
    // After DBF expires, delete the object.
    IntroSequence5ParticleVelocityLoc1BE74 {
        gravity_add: 0x0000_4000,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn intro_sequence_5_center_particle_loc_1BE88(
    fc82_variant: u8,
) -> IntroSequence5CenterParticleLoc1BE88 {
    // ASM loc_1BE88 (kid.asm:30689-30724): Allocate fixed-position sprite
    // with position and animation pointer dependent on ($FFFFFC82).w:
    //   FC82 == 0: (x,y) = ($2A,$74), mapping LnkTo_unk_E10B6
    //   FC82 == 1: (x,y) = ($30,$70), mapping LnkTo_unk_E10C6
    //   FC82 == 2: (x,y) = ($2A,$74), mapping LnkTo_unk_E10BE
    // ASM loc_1BEBE (kid.asm:30704-30709): Second position adjustment
    // branch for FC82 != 0 && FC82 != 1; subtracts 6 from x, adds 4 to y.
    let (x_pos, y_pos, sprite_label) = match fc82_variant {
        0 => (0x002A, 0x0074, "LnkTo_unk_E10B6-Data_Index"),
        1 => (0x0030, 0x0070, "LnkTo_unk_E10C6-Data_Index"),
        _ => (0x002A, 0x0074, "LnkTo_unk_E10BE-Data_Index"),
    };

    IntroSequence5CenterParticleLoc1BE88 {
        x_pos,
        y_pos,
        x_vel: 0x0002_F000,
        y_vel: 0xFFFF_7100, // -0x8F000 in 16.16 fixed-point
        vram_tile: 0x05EB,
        sprite_label,
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence5SpriteOffsetLoc1BF02 {
    pub(crate) addroffset_sprite: u16,
    pub(crate) hibernate_loop_count: u16,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn intro_sequence_5_sprite_offset_loc_1BF02(
    sprite_label: &'static str,
) -> IntroSequence5SpriteOffsetLoc1BF02 {
    // ASM loc_1BF02 (kid.asm:30723-30725): After FC82-variant-specific sprite
    // label selection, write the chosen mapping offset to addroffset_sprite(a3)
    // and seed the $1C counter for the loc_1BF0A hibernate loop.
    // d0 holds one of the LnkTo_unk_E10B6/E10BE/E10C6 mapping indices.
    let addroffset_sprite = match sprite_label {
        "LnkTo_unk_E10B6-Data_Index" => 0x0F16,
        "LnkTo_unk_E10C6-Data_Index" => 0x0F1E,
        _ => 0x0F26, // LnkTo_unk_E10BE-Data_Index
    };
    IntroSequence5SpriteOffsetLoc1BF02 {
        addroffset_sprite,
        hibernate_loop_count: 0x001C, // $1C = 28 iterations
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntroSequence5HibernateLoopLoc1BF0A {
    pub(crate) iterations: u16,
    pub(crate) y_vel_delta_per_frame: i32,
    pub(crate) is_moved_after_loop: bool,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn intro_sequence_5_hibernate_loop_loc_1BF0A() -> IntroSequence5HibernateLoopLoc1BF0A {
    // ASM loc_1BF0A (kid.asm:30727-30730): Hibernate for one frame, add $9000
    // to y_vel each iteration, repeat d0 times (d0 seeded as $1C at loc_1BF02).
    // After DBF expires: sf is_moved(a3) clears the movement flag.
    // Total y_vel change: 28 * $9000 = $F4000 = ~15.25 pixels in 16.16 fixed-point.
    IntroSequence5HibernateLoopLoc1BF0A {
        iterations: 0x001C,
        y_vel_delta_per_frame: 0x0000_9000,
        is_moved_after_loop: false,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn intro_sequence_5_hibernate_forever_loc_1BF1E() -> bool {
    // ASM loc_1BF1E (kid.asm:30733-30735): Infinite hibernate loop after the
    // velocity-accumulation phase clears is_moved. The object stays alive but
    // never moves again - `jsr (j_Hibernate_Object_1Frame).w; bra.s loc_1BF1E`.
    // This is the terminal state for intro sequence 5 center particles.
    true
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceCreditsGlyphInitSub1D786 {
    pub(crate) raw_allocation_spec: u32,
    pub(crate) display_enabled: bool,
    pub(crate) tile_word: u16,
    pub(crate) priority: u8,
    pub(crate) palette_line: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceCreditsGlyphPlacementSub1D7A8 {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) addroffset_sprite: u16,
    pub(crate) display_enabled: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceScriptDriverLoc1D1B0 {
    pub(crate) script_table_label: &'static str,
    pub(crate) first_delay_seed: Option<u16>,
    pub(crate) first_script_id: Option<u16>,
    pub(crate) first_dispatch_subroutine: Option<&'static str>,
    pub(crate) first_hibernate_iterations: Option<u16>,
    pub(crate) sentinel_branches_to: Option<&'static str>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceCreditsDecodeLoc1D97E {
    pub(crate) script_index: u16,
    pub(crate) flip_x_flag: bool,
    pub(crate) alternate_layout_flag: bool,
    pub(crate) decode_command: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EndSequenceCreditsUploadLoopLoc1D986 {
    pub(crate) dbf_seed: u16,
    pub(crate) iterations: u16,
    pub(crate) row_indices: Vec<u16>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceCreditsRowUploadLoc1D9B6 {
    pub(crate) row_index: u16,
    pub(crate) source_byte_offset: u16,
    pub(crate) source_word_count: u16,
    pub(crate) vdp_command: u32,
    pub(crate) z80_guard_subroutines: [&'static str; 2],
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn end_sequence_credits_glyph_init_sub_1D786() -> EndSequenceCreditsGlyphInitSub1D786 {
    // ASM sub_1D786 (kid.asm:32983): allocate the credits glyph worker with
    // raw gfx-object spec `$03000002`, clear display flag `$13(a1)`, then set
    // tile word `$0509`, priority 2, and palette line 1.
    EndSequenceCreditsGlyphInitSub1D786 {
        raw_allocation_spec: 0x0300_0002,
        display_enabled: false,
        tile_word: 0x0509,
        priority: 2,
        palette_line: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn end_sequence_script_driver_loc_1D1B0(
    script: &[u8],
) -> EndSequenceScriptDriverLoc1D1B0 {
    // ASM loc_1D1B0 (kid.asm:32560): seed `a2` with `unk_1D91C` before
    // falling straight into the shared script loop at `loc_1D1B4`.
    // ASM loc_1D1B4 (kid.asm:32564-32570): clear `d1/d0`, read the next delay
    // byte into `d1`, branch to `loc_1D1CE` when that byte is negative,
    // otherwise read the script id into `d0` and dispatch it through
    // `sub_1D944`.
    // ASM loc_1D1C4 (kid.asm:32572-32574): hibernate on each loop body before
    // `dbf d1,loc_1D1C4`, so a delay seed of `$3C` waits `$3C + 1 = $3D`
    // frames before the script driver fetches the next pair.
    let Some(&delay_byte) = script.first() else {
        return EndSequenceScriptDriverLoc1D1B0 {
            script_table_label: "unk_1D91C",
            first_delay_seed: None,
            first_script_id: None,
            first_dispatch_subroutine: None,
            first_hibernate_iterations: None,
            sentinel_branches_to: None,
        };
    };

    if (delay_byte as i8).is_negative() {
        return EndSequenceScriptDriverLoc1D1B0 {
            script_table_label: "unk_1D91C",
            first_delay_seed: None,
            first_script_id: None,
            first_dispatch_subroutine: None,
            first_hibernate_iterations: None,
            sentinel_branches_to: Some("loc_1D1CE"),
        };
    }

    let first_delay_seed = u16::from(delay_byte);
    let first_script_id = script.get(1).copied().map(u16::from);

    EndSequenceScriptDriverLoc1D1B0 {
        script_table_label: "unk_1D91C",
        first_delay_seed: Some(first_delay_seed),
        first_script_id,
        first_dispatch_subroutine: first_script_id.map(|_| "sub_1D944"),
        first_hibernate_iterations: first_script_id.map(|_| first_delay_seed.wrapping_add(1)),
        sentinel_branches_to: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn end_sequence_credits_decode_loc_1D97E(
    raw_script_id: u16,
) -> EndSequenceCreditsDecodeLoc1D97E {
    // ASM sub_1D944 / loc_1D97E (kid.asm:33226-33238): `bclr #$F,d0` and
    // `bclr #$E,d0` strip the two high control bits before indexing
    // `unk_2E39A`, while each old bit is latched via `sne` into
    // `($FFFFFB5C).w` / `($FFFFFB5E).w`. `loc_1D97E` keeps the default
    // `EniDec` command `$813E`, except when bit 15 was set and bit 14 was
    // clear, where the routine replaces it with `$22DC`.
    let flip_x_flag = raw_script_id & 0x8000 != 0;
    let alternate_layout_flag = raw_script_id & 0x4000 != 0;

    EndSequenceCreditsDecodeLoc1D97E {
        script_index: raw_script_id & 0x3FFF,
        flip_x_flag,
        alternate_layout_flag,
        decode_command: if flip_x_flag && !alternate_layout_flag {
            0x22DC
        } else {
            0x813E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn end_sequence_credits_upload_loop_loc_1D986() -> EndSequenceCreditsUploadLoopLoc1D986 {
    // ASM loc_1D986 (kid.asm:33240): after `moveq #$D,d7`, the routine calls
    // `sub_1D990` before `dbf d7,loc_1D986`, so the upload loop executes
    // `$D + 1 = 14` passes and visits rows in descending `d7` order.
    let dbf_seed = 0x000D;
    EndSequenceCreditsUploadLoopLoc1D986 {
        dbf_seed,
        iterations: dbf_seed.wrapping_add(1),
        row_indices: (0..=dbf_seed).rev().collect(),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn end_sequence_credits_row_upload_loc_1D9B6(
    row_index: u16,
    flip_x_flag: bool,
) -> EndSequenceCreditsRowUploadLoc1D9B6 {
    // ASM sub_1D990 / loc_1D9B6 (kid.asm:33247-33270): scale `d7` by `$1C`
    // bytes to pick one 14-word row from `Decompression_Buffer`, then derive
    // the VRAM word address from `(7 + d7) * $80 + $1A`. When
    // `($FFFFFB5C).w != 0`, the routine subtracts `$2000` before the
    // `asl.l #2` / `lsr.w #2` / `addi.w #$4000` / `swap` sequence that
    // materialises the final VDP command longword at `4(a6)`.
    let source_byte_offset = row_index.wrapping_mul(0x001C);
    let mut vram_word_address = row_index
        .wrapping_add(0x0007)
        .wrapping_mul(0x0080)
        .wrapping_add(0x001A);
    if flip_x_flag {
        vram_word_address = vram_word_address.wrapping_sub(0x2000);
    }

    let mut vdp_command = u32::from(vram_word_address) << 2;
    let low_word = ((vdp_command as u16) >> 2).wrapping_add(0x4000);
    vdp_command = (vdp_command & 0xFFFF_0000) | u32::from(low_word);
    vdp_command = vdp_command.rotate_right(16);

    EndSequenceCreditsRowUploadLoc1D9B6 {
        row_index,
        source_byte_offset,
        source_word_count: 0x000E,
        vdp_command,
        z80_guard_subroutines: ["j_sub_914", "j_sub_924"],
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn end_sequence_credits_row_copy_word_count_loc_1D9CA(d5_seed: u16) -> u16 {
    // ASM loc_1D9CA (kid.asm:33272): `moveq #$D,d5` seeds the inner row-copy
    // loop, then `move.w (a4)+,(a6)` executes before `dbf d5,loc_1D9CA`, so
    // the credits upload always copies `d5 + 1` words from
    // `Decompression_Buffer` into the VDP data port.
    d5_seed.wrapping_add(1)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_end_sequence_credits_glyphs_sub_1D7A8(
    script: &[u8],
    mut glyphs: [Option<EndSequenceCreditsGlyphPlacementSub1D7A8>; 6],
) -> [Option<EndSequenceCreditsGlyphPlacementSub1D7A8>; 6] {
    // ASM sub_1D7A8 / loc_1D7B0 (kid.asm:32997-33012): read a
    // sentinel-terminated byte stream from `a4` as `(x, y, sprite)` triples.
    // `loc_1D7B0` loads the next X byte, exits at `loc_1D7E0` when it is zero,
    // otherwise advances to the next cached credits glyph worker starting at
    // `$FFFFFB76` and keeps consuming triples.
    let mut script_offset = 0usize;
    let mut glyph_offset = 0usize;

    while script_offset < script.len() && glyph_offset < glyphs.len() {
        let x_byte = script[script_offset];
        script_offset += 1;
        if x_byte == 0 {
            break;
        }
        if script_offset + 1 >= script.len() {
            break;
        }

        let y_byte = script[script_offset];
        let sprite_byte = script[script_offset + 1];
        script_offset += 2;

        glyphs[glyph_offset] = Some(EndSequenceCreditsGlyphPlacementSub1D7A8 {
            x_pos: u16::from(x_byte).wrapping_add(0x0032),
            y_pos: u16::from(y_byte).wrapping_sub(0x00A0),
            addroffset_sprite: 0x1106u16.wrapping_add(u16::from(sprite_byte)),
            display_enabled: true,
        });
        glyph_offset += 1;
    }

    glyphs
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectGfxLinksSubE02 {
    pub(crate) object_slot_address: u32,
    pub(crate) gfx_slot_address: u32,
    pub(crate) gfx_to_object_link: u32,
    pub(crate) object_to_gfx_link: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GfxObjectSpecialSlotLoc10C6 {
    None,
    Kid,
    KidProjectile,
    UnknownF86A,
    UnknownF86E,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GfxObjectDeleteLoc10C6 {
    pub(crate) predecessor_next_index: Option<usize>,
    pub(crate) target_next_free_index: Option<usize>,
    pub(crate) next_free_head_index: usize,
    pub(crate) next_number_gfx_objects: u16,
    pub(crate) special_slot: GfxObjectSpecialSlotLoc10C6,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn clear_owner_gfx_backref_loc_1078(
    owner_addr: u32,
    mut owner_links: [u32; 3],
    target_gfx_addr: u32,
) -> [u32; 3] {
    // ASM loc_1078 (kid.asm:2002): only treat `a5` as an owner-object pointer
    // when bit 0 is clear and the masked 24-bit address stays in WRAM
    // (`>= $FF0000`). Then compare the deleted gfx pointer in `a3` against
    // `$36(a5)`.
    // ASM loc_10A0 (kid.asm:2017): when `$36(a5)` does not match, compare
    // `$3A(a5)` next, clear only that second back-reference on equality, and
    // branch straight to the shared free-list walk without touching `$3E(a5)`.
    // ASM loc_10AC (kid.asm:2024): only if both earlier slots miss does the
    // routine compare and clear `$3E(a5)`.
    if owner_addr & 1 != 0 {
        return owner_links;
    }

    let masked_owner_addr = owner_addr & 0x00FF_FFFF;
    if masked_owner_addr < 0x00FF_0000 {
        return owner_links;
    }

    if owner_links[0] == target_gfx_addr {
        owner_links[0] = 0;
    } else if owner_links[1] == target_gfx_addr {
        owner_links[1] = 0;
    } else if owner_links[2] == target_gfx_addr {
        owner_links[2] = 0;
    }

    owner_links
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_delete_predecessor_index_loc_10B6(
    next_indices: &[Option<usize>],
    target_index: usize,
) -> Option<usize> {
    // ASM loc_10B6 (kid.asm:2029): after the owner back-reference cleanup,
    // reset `a0` to `(Addr_FirstGfxObjectSlot).w` so the shared `loc_10BA`
    // walk starts from the sentinel head before unlinking `a3`.
    gfx_object_predecessor_index_sub_FF6(next_indices, target_index)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_delete_loc_10C6(
    target_next_index: Option<usize>,
    next_free_head_index: Option<usize>,
    number_gfx_objects: u16,
    object_kind_word: u16,
    target_index: usize,
) -> GfxObjectDeleteLoc10C6 {
    // ASM loc_10C6 (kid.asm:2041): splice `0(a3)` over the predecessor link,
    // push `a3` onto `(Addr_NextFreeGfxObjectSlot).w`, decrement
    // `Number_GfxObjects`, then fan out on `8(a3)` so kinds 1/2/3 clear the
    // Kid / KidProjectile / `$FFFFF86A` singleton pointer before the shared
    // `loc_1106` tail. ASM loc_1114 (kid.asm:2082) is the predecessor relink:
    // `move.l 4(a3),4(a0)` copies the target's successor over the predecessor's
    // next link before the free-list push and singleton cleanup continue.
    let special_slot = match object_kind_word {
        1 => GfxObjectSpecialSlotLoc10C6::Kid,
        // ASM loc_10EA (kid.asm:2054): after the shared `subq.w #1,d0`,
        // object kind 2 branches here, decrements once more to zero, then
        // clears `Addr_GfxObject_KidProjectile`.
        2 => GfxObjectSpecialSlotLoc10C6::KidProjectile,
        // ASM loc_10F4 (kid.asm:2061): after the shared `subq.w #1,d0`, kind
        // 3 decrements once more to zero here and clears the `$FFFFF86A`
        // singleton pointer before the shared `loc_1106` tail.
        3 => GfxObjectSpecialSlotLoc10C6::UnknownF86A,
        // ASM loc_10FE (kid.asm:2068): after the shared `subq.w #1,d0`, kind
        // 4 decrements once more to zero here and clears the `$FFFFF86E`
        // singleton pointer before the shared `loc_1106` tail.
        4 => GfxObjectSpecialSlotLoc10C6::UnknownF86E,
        _ => GfxObjectSpecialSlotLoc10C6::None,
    };

    GfxObjectDeleteLoc10C6 {
        predecessor_next_index: target_next_index,
        target_next_free_index: next_free_head_index,
        next_free_head_index: target_index,
        next_number_gfx_objects: number_gfx_objects.wrapping_sub(1),
        special_slot,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_delete_special_predecessor_index_loc_111A(
    next_indices: &[Option<usize>],
    target_index: usize,
) -> Option<usize> {
    // ASM loc_1106 (kid.asm:2072): walk the special singleton chain through
    // `4(a0)` until it finds `a3`. If `move.l 4(a0),d0` hits a null link
    // first, `beq.s loc_111A` returns immediately without the `loc_1114`
    // relink, so a miss is modelled as `None`.
    gfx_object_predecessor_index_sub_FF6(next_indices, target_index)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_head_under_samurai_haze_loc_1462(
    first_gfx_object_index: Option<usize>,
) -> Option<usize> {
    // ASM loc_1462 (kid.asm:2531): the Samurai Haze branch reloads `a3` from
    // `(Addr_FirstGfxObjectSlot).w` before falling straight into `loc_1466`, so
    // the slowed-object walk always restarts from the sentinel head pointer.
    first_gfx_object_index
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_move_and_animate_loc_144E(
    first_gfx_object_index: Option<usize>,
    next_indices: &[Option<usize>],
) -> Vec<usize> {
    // ASM loc_144E (kid.asm:2518-2523): load the current link from `0(a3)`,
    // return immediately when the sentinel head points at zero, otherwise move
    // `a3` to that worker, call `GfxObject_Move` and `GfxObject_Animate`, then
    // branch back to `loc_144E` and repeat until the next link is null.
    let mut visited_indices = Vec::new();
    let mut current = first_gfx_object_index;

    while let Some(index) = current {
        visited_indices.push(index);
        current = next_indices[index];
    }

    visited_indices
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_next_index_loc_17BE(
    current_next_index: Option<usize>,
) -> Option<usize> {
    // ASM GfxObjects_Collision loc_16FC / loc_17BE (kid.asm:2829, 2934):
    // load the current `4(a2)` link into `d0`; if it is zero, branch straight
    // to the shared epilogue at loc_17BE, otherwise move that link into `a2`
    // and continue the collision walk.
    current_next_index
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GfxObjectsCollisionTileDispatchLoc17C6 {
    Continue,
    Solid,
    UpSlope,
    DownSlope,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_tile_dispatch_loc_17C6(
    tile_word: u16,
) -> GfxObjectsCollisionTileDispatchLoc17C6 {
    // ASM loc_17C6 (kid.asm:2940): reload the current tile after the bit-$4000
    // precheck, keep only the collision nibble, ignore spikes (`$7000`), route
    // solids to the solid worker, slope-up to the upslope worker, and treat
    // the remaining collision case as slope-down.
    match tile_word & 0x7000 {
        0x7000 => GfxObjectsCollisionTileDispatchLoc17C6::Continue,
        0x6000 => GfxObjectsCollisionTileDispatchLoc17C6::Solid,
        0x4000 => GfxObjectsCollisionTileDispatchLoc17C6::UpSlope,
        _ => GfxObjectsCollisionTileDispatchLoc17C6::DownSlope,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_upslope_continue_loc_180E(
    hitbox_right_px: u16,
    hitbox_bottom_px: u16,
    tile_left_px: u16,
    tile_top_px: u16,
) -> bool {
    // ASM GfxObjects_Collision_UpSlope / loc_180E (kid.asm:2951-2970): only
    // keep the current tile when the right edge lives in that tile's column
    // and the sampled bottom-right corner has crossed below the upslope line
    // `x_within > $E - y_within`; otherwise restore `d0-d1` and continue the
    // rectangle scan via loc_180E.
    if (hitbox_right_px & 0xFFF0) != tile_left_px {
        return true;
    }

    let y_within = i32::from(hitbox_bottom_px.wrapping_sub(tile_top_px));
    let x_within = i32::from(hitbox_right_px.wrapping_sub(tile_left_px));
    x_within <= (0x0E - y_within)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_upslope_tile_addr_loc_1814(
    scanned_tile_layout_addr: u16,
) -> u16 {
    // ASM loc_1814 (kid.asm:2973): the row loop has already post-incremented
    // `a3`, so the upslope hit path rewinds by one tile before caching the
    // matched Level_Layout address in `$FFFFFB6C` for `GfxObjects_Collision_ChkUpSlope`.
    scanned_tile_layout_addr.wrapping_sub(2)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_downslope_tile_addr_loc_1838(
    scanned_tile_layout_addr: u16,
) -> u16 {
    // ASM loc_1838 (kid.asm:2991): like loc_1814, the down-slope hit path
    // arrives after the row walk has post-incremented `a3`, so it rewinds by
    // one tile before caching `$FFFFFB6C` for `GfxObjects_Collision_ChkDownSlope`.
    scanned_tile_layout_addr.wrapping_sub(2)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GfxObjectsCollisionSolidLeftWallLoc19BA {
    ContinueLoc19D2,
    RedirectToDownSlope { layout_addr: u16 },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_solid_leftwall_loc_19BA(
    lower_right_tile_word: u16,
    cached_layout_addr: u16,
) -> GfxObjectsCollisionSolidLeftWallLoc19BA {
    // ASM loc_19BA (kid.asm:3139-3144): inspect `2(a3)` after the solid-left
    // wall resolver decides the contact came from above/right. A `$5000`
    // slope-down there advances the cached `FB6C` tile pointer by one cell and
    // jumps straight to `GfxObjects_Collision_ChkDownSlope`; every other
    // collision nibble falls through to the non-Kid left-wall snap at loc_19D2.
    if lower_right_tile_word & 0x7000 == 0x5000 {
        GfxObjectsCollisionSolidLeftWallLoc19BA::RedirectToDownSlope {
            layout_addr: cached_layout_addr.wrapping_add(2),
        }
    } else {
        GfxObjectsCollisionSolidLeftWallLoc19BA::ContinueLoc19D2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GfxObjectsCollisionSolidRightWallLoc1966 {
    ContinueLoc197E,
    RedirectToUpSlope { layout_addr: u16 },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_solid_rightwall_loc_1966(
    upper_left_tile_word: u16,
    cached_layout_addr: u16,
) -> GfxObjectsCollisionSolidRightWallLoc1966 {
    // ASM loc_1966 (kid.asm:3106): after the solid-rightwall resolver decides
    // the contact came from above/left, inspect `-2(a3)`. A `$4000` upslope
    // there rewinds the cached `FB6C` tile pointer by one cell and jumps to
    // `GfxObjects_Collision_ChkUpSlope`; every other collision nibble falls
    // through to the non-Kid right-wall snap at loc_197E.
    if upper_left_tile_word & 0x7000 == 0x4000 {
        GfxObjectsCollisionSolidRightWallLoc1966::RedirectToUpSlope {
            layout_addr: cached_layout_addr.wrapping_sub(2),
        }
    } else {
        GfxObjectsCollisionSolidRightWallLoc1966::ContinueLoc197E
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GfxObjectsCollisionRightWallSnapLoc197E {
    pub(crate) collision_type: u16,
    pub(crate) snapped_x_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_rightwall_snap_loc_197E(
    is_kid_gfx_object: bool,
    direction_flag: u8,
    anchor_x_px: u16,
    first_extent_px: u16,
    second_extent_px: u16,
) -> Option<GfxObjectsCollisionRightWallSnapLoc197E> {
    // ASM loc_197E / loc_19A8 (kid.asm:3115-3128): Kid itself exits via
    // loc_1ADA, but every other gfx object stores `colid_rightwall` and snaps
    // `$1A(a2)` from the stacked anchor coordinate. Non-zero `$16(a2)` uses
    // `anchor + first_extent - 1`; zero `$16(a2)` uses
    // `anchor - first_extent - second_extent - 1`.
    if is_kid_gfx_object {
        return None;
    }

    let snapped_x_px = if direction_flag != 0 {
        anchor_x_px.wrapping_add(first_extent_px).wrapping_sub(1)
    } else {
        anchor_x_px
            .wrapping_sub(first_extent_px)
            .wrapping_sub(second_extent_px)
            .wrapping_sub(1)
    };

    Some(GfxObjectsCollisionRightWallSnapLoc197E {
        collision_type: 0x0004,
        snapped_x_px,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GfxObjectsCollisionLeftWallSnapLoc1A00 {
    pub(crate) collision_type: u16,
    pub(crate) snapped_x_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_leftwall_snap_loc_1A00(
    is_kid_gfx_object: bool,
    anchor_x_px: u16,
    first_extent_px: u16,
) -> Option<GfxObjectsCollisionLeftWallSnapLoc1A00> {
    // ASM loc_19D2 / loc_1A00 (kid.asm:3147-3169): Kid itself exits via
    // `loc_1ADA`, but every other gfx object stores `colid_leftwall` and the
    // zero-direction branch uses only one `(a1)+` extent before the shared
    // `addi.w #$10,d7`, yielding `anchor - first_extent + $10`.
    if is_kid_gfx_object {
        return None;
    }

    Some(GfxObjectsCollisionLeftWallSnapLoc1A00 {
        collision_type: 0x0008,
        snapped_x_px: anchor_x_px.wrapping_sub(first_extent_px).wrapping_add(0x10),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GfxObjectsCollisionSolidFloorAboveTileLoc1A12 {
    ContinueLoc1A44,
    RedirectToDownSlope { layout_addr: u16 },
    RedirectToUpSlope { layout_addr: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GfxObjectsCollisionCeilingSnapLoc1A6C {
    pub(crate) collision_type: u16,
    pub(crate) snapped_y_px: u16,
    pub(crate) y_subpixel: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_ceiling_snap_loc_1A6C(
    is_kid_gfx_object: bool,
    anchor_y_px: u16,
    first_extent_px: u16,
) -> Option<GfxObjectsCollisionCeilingSnapLoc1A6C> {
    // ASM loc_1A6C (kid.asm:3206-3215): Kid itself exits via `loc_1ADA`, but
    // every other gfx object stores `colid_ceiling`, snaps `$1E(a2)` from the
    // stacked anchor row using `anchor - first_extent + $10`, and clears the
    // vertical subpixel at `$20(a2)`.
    if is_kid_gfx_object {
        return None;
    }

    Some(GfxObjectsCollisionCeilingSnapLoc1A6C {
        collision_type: 0x0002,
        snapped_y_px: anchor_y_px
            .wrapping_sub(first_extent_px)
            .wrapping_add(0x0010),
        y_subpixel: 0,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_collision_solid_floor_above_tile_loc_1A12(
    upper_tile_word: u16,
    cached_layout_addr: u16,
    level_width_tiles: u16,
) -> GfxObjectsCollisionSolidFloorAboveTileLoc1A12 {
    // ASM loc_1A12 / loc_1A30 (kid.asm:3172-3188): inspect the tile one row
    // above the cached floor contact. A `$4000` upslope or `$5000` downslope
    // both rewind `$FFFFFB6C` by one row via
    // `sub.w Level_width_tiles,($FFFFFB6C).w` before branching to the
    // corresponding slope resolver. Every other collision nibble falls through
    // to the non-Kid floor snap at `loc_1A44`.
    match upper_tile_word & 0x7000 {
        0x4000 => GfxObjectsCollisionSolidFloorAboveTileLoc1A12::RedirectToUpSlope {
            layout_addr: cached_layout_addr.wrapping_sub(level_width_tiles),
        },
        0x5000 => GfxObjectsCollisionSolidFloorAboveTileLoc1A12::RedirectToDownSlope {
            layout_addr: cached_layout_addr.wrapping_sub(level_width_tiles),
        },
        _ => GfxObjectsCollisionSolidFloorAboveTileLoc1A12::ContinueLoc1A44,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_objects_move_and_animate_under_samurai_haze_loc_1466(
    time_frames: u16,
    first_gfx_object_index: Option<usize>,
    next_indices: &[Option<usize>],
    kind_words: &[u16],
) -> Vec<usize> {
    // ASM loc_1466 / loc_147E / loc_1488 (kid.asm:2534-2548): walk the same
    // linked gfx-object list as `loc_144E`, but when a worker's kind word at
    // `8(a3)` is 2 or 3, skip `GfxObject_Move`/`GfxObject_Animate` unless
    // `Time_Frames & 3 == 0`. Other kinds still run every frame.
    let mut moved_indices = Vec::new();
    let mut current = first_gfx_object_index;
    let slowed_frame = time_frames & 3 != 0;

    while let Some(index) = current {
        let kind_word = kind_words.get(index).copied().unwrap_or_default();
        if !(slowed_frame && matches!(kind_word, 2 | 3)) {
            moved_indices.push(index);
        }
        current = next_indices[index];
    }

    moved_indices
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_predecessor_index_sub_FF6(
    next_indices: &[Option<usize>],
    target_index: usize,
) -> Option<usize> {
    // ASM loc_1004 (kid.asm:1925): when the current `0(a4)` link is neither
    // null nor the target `a3`, load that next pointer into `a4` and branch
    // back to `loc_FFE`; if the walk reaches a null link first, `beq.s
    // loc_1038` returns without unlinking anything.
    let mut cursor_index = 0usize;
    loop {
        let next_index = next_indices[cursor_index]?;
        if next_index == target_index {
            return Some(cursor_index);
        }
        cursor_index = next_index;
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_unlink_loc_100E(
    next_indices: &mut [Option<usize>],
    predecessor_index: usize,
    target_index: usize,
) {
    // ASM loc_100E (kid.asm:1932): after loc_FFE finds the predecessor in
    // `a4`, copy the target's `0(a3)` next link over `0(a4)` to unlink `a3`
    // before the later sentinel restart and priority-ordered reinsertion.
    next_indices[predecessor_index] = next_indices[target_index];
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_insertion_predecessor_index_loc_101C(
    next_indices: &[Option<usize>],
    priority_words: &[u16],
    target_priority_word: u16,
) -> usize {
    // ASM loc_101C (kid.asm:1937): restart from the sentinel, keep following
    // `0(a4)` while each candidate's `$A(a4)` priority word is less than or
    // equal to `d5`, then insert after the last such node.
    let mut cursor_index = 0usize;
    loop {
        let Some(next_index) = next_indices[cursor_index] else {
            return cursor_index;
        };
        if priority_words[next_index] > target_priority_word {
            return cursor_index;
        }
        cursor_index = next_index;
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_insert_loc_102E(
    next_indices: &mut [Option<usize>],
    predecessor_index: usize,
    target_index: usize,
) {
    // ASM loc_102E (kid.asm:1946): copy the predecessor's current `0(a4)`
    // link into the target's `0(a3)` slot, then store `a3` back into `0(a4)`
    // so reinsertion splices the detached node ahead of the old successor.
    next_indices[target_index] = next_indices[predecessor_index];
    next_indices[predecessor_index] = Some(target_index);
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn allocate_object_and_gfx_links_sub_E02(
    object_slot_address: u32,
    gfx_slot_address: u32,
) -> ObjectGfxLinksSubE02 {
    // ASM sub_E02 (kid.asm:1622): `Allocate_ObjectSlot` leaves the new object
    // in `a0`, `Allocate_GfxObjectSlot_a1` leaves the gfx object in `a1`,
    // then the routine cross-links them through object `$36(a0)` and gfx
    // `$C(a1)` before returning.
    ObjectGfxLinksSubE02 {
        object_slot_address,
        gfx_slot_address,
        gfx_to_object_link: object_slot_address,
        object_to_gfx_link: gfx_slot_address,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockJumpWindowSubDFB0 {
    pub(crate) left_reach_px: u16,
    pub(crate) right_reach_px: u16,
    pub(crate) vertical_reach_px: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockRevealWorkerSubE0F8 {
    pub(crate) allocation_spec: u16,
    pub(crate) object_handler_addr: u32,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word_addr: u32,
    pub(crate) sprite_mapping: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockRevealWorkerSubE1D4 {
    pub(crate) revealed_layout_byte: u8,
    pub(crate) allocation_spec: u16,
    pub(crate) object_handler_addr: u32,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word_addr: u32,
    pub(crate) sprite_mapping: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_jump_window_sub_DFB0(
    form: PlayerForm,
    facing: Direction,
) -> HiddenBlockJumpWindowSubDFB0 {
    // ASM sub_DFB0 (kid.asm:17317): when `Character_Movement == MoveID_Jump`,
    // index `unk_E246` by `Current_Helmet`, load two horizontal reach words
    // plus one vertical reach word, then swap the horizontal pair when
    // `x_direction(a3)` is non-zero before probing hidden blocks.
    let (mut left_reach_px, mut right_reach_px, vertical_reach_px) = match form {
        PlayerForm::Juggernaut => (0x0010, 0x0010, 0x0010),
        PlayerForm::Micromax => (0x0002, 0x0002, 0x0010),
        _ => (0x0004, 0x0004, 0x0020),
    };
    if facing == Direction::Left {
        std::mem::swap(&mut left_reach_px, &mut right_reach_px);
    }
    HiddenBlockJumpWindowSubDFB0 {
        left_reach_px,
        right_reach_px,
        vertical_reach_px,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_reveal_worker_sub_E0F8(
    block_col: u16,
    block_row: u16,
    layout_word_addr: u32,
    sprite_mapping: u16,
) -> HiddenBlockRevealWorkerSubE0F8 {
    // ASM sub_E0F8 (kid.asm:17444): allocate a `$2001` object slot, install
    // `loc_E304`, copy the caller's block column/row plus layout pointer, and
    // pull the next sprite mapping word from the `unk_E11E` table into `$4E(a0)`.
    HiddenBlockRevealWorkerSubE0F8 {
        allocation_spec: 0x2001,
        object_handler_addr: 0x0000_E304,
        block_col,
        block_row,
        layout_word_addr,
        sprite_mapping,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_reveal_worker_sub_E1D4(
    block_col: u16,
    block_row: u16,
    layout_word_addr: u32,
    sprite_mapping: u16,
) -> HiddenBlockRevealWorkerSubE1D4 {
    // ASM sub_E1D4 (kid.asm:17619): write `#$60` into `(a2)`, allocate a
    // `$2001` object slot, install `loc_E282`, then copy the caller's block
    // column/row plus layout pointer and sprite mapping into `$44/$46/$48/$4C/$4E`.
    HiddenBlockRevealWorkerSubE1D4 {
        revealed_layout_byte: 0x60,
        allocation_spec: 0x2001,
        object_handler_addr: 0x0000_E282,
        block_col,
        block_row,
        layout_word_addr,
        sprite_mapping,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn hide_end_sequence_credits_glyphs_sub_1D7E6(display_enabled: &mut [bool; 6]) {
    // ASM sub_1D7E6 (kid.asm:33029): load the six credits glyph worker slots
    // from `$FFFFFB72`, then clear each worker's display-enable byte `$13(a3)`.
    for visible in display_enabled.iter_mut() {
        *visible = false;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn fade_end_sequence_palette_sub_1D800<F>(
    palette_buffer: &mut [u16],
    mut refresh_and_hibernate: F,
) where
    F: FnMut(),
{
    // ASM sub_1D800 (kid.asm:33046): spill the return address and
    // `d0-d1/d7/a4` into the ending scratch area, darken Palette_Buffer+$60
    // via sub_1C85E eight times, and after each fade pass run the
    // `sub_1D830 -> Hibernate_Object_1Frame` pair for four iterations.
    for _ in 0..8 {
        fade_palette_span_sub_1C85E(palette_buffer, 0x0060);
        for _ in 0..4 {
            refresh_and_hibernate();
        }
    }
}

impl PlayerState {
    pub(crate) fn intro_sequence_3_roster_count() -> usize {
        INTRO_SEQUENCE_3_ROSTERS.len()
    }

    pub(crate) fn intro_sequence_3_pose_for_roster(roster_index: usize) -> Self {
        // ROM Load_IntroSequence3 rotates through the four `unk_1AF78`
        // descriptor rosters via `$FFFFFC83`, and oracle traces expose the
        // third allocated roster descriptor in slot 2 as `kid_gfx`.
        let descriptor = intro_sequence_3_descriptor_loc_1AEBA(
            roster_index,
            INTRO_SEQUENCE_3_TRACE_DESCRIPTOR_SLOT,
        )
        .expect("intro sequence 3 trace descriptor slot must not hit the ROM sentinel");
        Self {
            x_pos: u32::from(descriptor.x_pos) << 16,
            y_pos: u32::from(descriptor.y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Left,
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

    pub(crate) fn title_pose() -> Self {
        Self {
            x_pos: TITLE_KID_X_POS,
            y_pos: TITLE_KID_Y_POS,
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

    pub(crate) fn title_menu_cursor_pose(y_pos: u32) -> Self {
        Self {
            x_pos: 0x0028_0000,
            y_pos,
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

    pub(crate) fn intro_sequence_5_slot_2_pose() -> Self {
        Self {
            x_pos: INTRO_SEQUENCE_5_SLOT_2_X_POS,
            y_pos: INTRO_SEQUENCE_5_SLOT_2_Y_POS,
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

    pub(crate) fn from_level_entry_title_worker(
        worker: &LevelEntryTitleWorker,
        elapsed_frames: usize,
    ) -> Self {
        let motion = worker
            .motion
            .as_ref()
            .expect("seeded level-entry title worker");
        Self {
            x_pos: level_entry_title_worker_axis_pos(
                motion.start_x_pos,
                worker.target_x_pos,
                motion.x_step_fp,
                elapsed_frames,
            ),
            y_pos: level_entry_title_worker_axis_pos(
                motion.start_y_pos,
                worker.target_y_pos,
                motion.y_step_fp,
                elapsed_frames,
            ),
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Left,
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
}

impl NativeRuntime {
    pub(crate) fn rebuild_starting_level_title_workers(&mut self) {
        self.starting_level_title_workers.clear();
        let title_text_level_id = level_entry_title_text_level_id_loc_1AB52(self.current_level);
        let Some(title_card) = self
            .level_record(title_text_level_id)
            .and_then(|level| level.title_card.as_ref())
            .cloned()
        else {
            return;
        };

        let visible_glyph_count = title_card
            .script_bytes
            .iter()
            .filter(|&&glyph| glyph != 0 && glyph != 0x84 && glyph != 0xFF)
            .count();
        let mut current_destination = title_card.line_destinations.first().cloned();
        let mut line_index = 0usize;
        let mut visible_glyph_index = 0usize;

        for &glyph in &title_card.script_bytes {
            match glyph {
                0 => {
                    // ASM loc_1ABE2 (kid.asm:28827-28830): the line-break
                    // token consumes one script byte, then reloads `d2/d3`
                    // from the next `(x,y)` pair in the title destination
                    // stream before resuming glyph parsing.
                    line_index += 1;
                    current_destination = title_card.line_destinations.get(line_index).cloned();
                }
                0x84 => {
                    if let Some(destination) = current_destination.as_mut() {
                        destination.x = level_entry_title_space_token_x_loc_1ABEA(destination.x);
                    }
                }
                0xFF => break,
                _ => {
                    let Some(destination) = current_destination.clone() else {
                        break;
                    };
                    let gfx_slot_index = level_entry_title_letter_gfx_slot_index(
                        visible_glyph_index,
                        visible_glyph_count,
                    );
                    let target_x_pos = u32::from(destination.x) << 16;
                    let target_y_pos =
                        u32::from(level_entry_title_target_y(glyph, destination.y)) << 16;
                    self.starting_level_title_workers
                        .push(LevelEntryTitleWorker {
                            // ASM loc_1AB26 allocates title objects in script
                            // order, but `Allocate_ObjectSlot` reverse-inserts
                            // equal-priority workers before the next
                            // `Execute_Objects` pass. Oracle slot probes still
                            // read fixed gfx RAM slots directly, so keep the
                            // traced slot id separate from the later first-pass
                            // execution order used for RNG seeding.
                            gfx_slot_index,
                            activation_frame: LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME,
                            glyph,
                            target_x_pos,
                            target_y_pos,
                            motion: None,
                        });
                    if let Some(destination) = current_destination.as_mut() {
                        destination.x = destination
                            .x
                            .saturating_add(level_entry_title_advance_x(glyph));
                    }
                    visible_glyph_index += 1;
                }
            }
        }

        if let Some(destination) = title_card.level_number_destination {
            for number_index in level_entry_title_deferred_number_indices_loc_1ABF4(
                title_card.level_number_glyph_count,
            ) {
                self.starting_level_title_workers
                    .push(LevelEntryTitleWorker {
                        // ASM loc_1AD48 allocates the deferred number worker
                        // after the visible title glyph shell, so the suffix
                        // occupies the next free gfx slots above the letters.
                        // ASM loc_1AD56 then writes glyph `#$7B`, stores the
                        // `(d1,d2)` destination pair into `$46/$48(a0)`, and
                        // advances `d1` by `#$10` on each inclusive DBF pass.
                        gfx_slot_index: visible_glyph_count + number_index,
                        activation_frame: LEVEL_ENTRY_TITLE_DEFERRED_NUMBER_VISIBLE_FRAME,
                        glyph: 0x7B,
                        target_x_pos: u32::from(
                            level_entry_title_deferred_number_target_x_loc_1AD56(
                                destination.x,
                                number_index,
                            ),
                        ) << 16,
                        target_y_pos: u32::from(destination.y) << 16,
                        motion: None,
                    });
            }
        }

        if level_entry_title_spawns_two_player_waiter_loc_1AC10(self.two_player) {
            // TODO(fidelity): ASM loc_1AC10 (kid.asm:28849-28856) allocates
            // loc_1AD80 when `Two_player_flag` is set, and loc_1AD80 then
            // builds the alternating-player head sprites during the level
            // entry. The current title-worker reconstruction only models the
            // loc_1AB26 / loc_1AD48 script shell, so keep the 2P waiter branch
            // explicit until we add a two-player oracle case for that lineage.
        }
    }

    pub(crate) fn maybe_seed_starting_level_title_workers(&mut self, elapsed_frames: usize) {
        let uses_early_start_title_seed_rng = matches!(
            self.boot_state,
            BootState::StartingLevel {
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: false,
                ..
            }
        );
        let workers = &mut self.starting_level_title_workers;
        let rom_random = &mut self.rom_random;
        let title_rng = &mut self.starting_level_title_rng;
        let seed_indices = (0..workers.len())
            .filter(|&index| {
                let worker = &workers[index];
                worker.motion.is_none()
                    && level_entry_title_effective_activation_frame(
                        worker,
                        uses_early_start_title_seed_rng,
                    ) == elapsed_frames
            })
            .collect::<Vec<_>>();
        for index in seed_indices.into_iter().rev() {
            let worker = &mut workers[index];
            let (start_x_pos, start_y_pos) =
                if uses_early_start_title_seed_rng || title_rng.is_some() {
                    let rng = title_rng
                        .as_mut()
                        .expect("level-entry title seed RNG should be initialised");
                    let start_selector = rng.next_byte() & 3;
                    level_entry_title_start_pose(rng, start_selector)
                } else {
                    let start_selector = rom_random.next_byte() & 3;
                    level_entry_title_start_pose(rom_random, start_selector)
                };
            worker.motion = Some(LevelEntryTitleWorkerMotion {
                start_x_pos,
                start_y_pos,
                x_step_fp: level_entry_title_axis_step(start_x_pos, worker.target_x_pos),
                y_step_fp: level_entry_title_axis_step(start_y_pos, worker.target_y_pos),
            });
        }
    }

    pub(crate) fn starting_level_slot_2_pose(&self, elapsed_frames: usize) -> Option<PlayerState> {
        self.starting_level_title_workers
            .iter()
            .find(|worker| worker.gfx_slot_index == LEVEL_ENTRY_TITLE_SLOT_2_INDEX)
            .and_then(|worker| {
                let activation_frame = if matches!(
                    self.boot_state,
                    BootState::StartingLevel {
                        uses_load_ingame_bootstrap: true,
                        load_ingame_bootstrap_started: false,
                        ..
                    }
                ) {
                    level_entry_title_effective_activation_frame(worker, true)
                } else {
                    level_entry_title_effective_activation_frame(worker, false)
                };
                if elapsed_frames < activation_frame {
                    return None;
                }
                worker.motion.as_ref().map(|_| {
                    // ASM loc_1AC70 -> loc_1AD2A seeds the start pose on first
                    // execution, then hibernates for one frame before the
                    // first glide step. Oracle frame 504 is that first visible
                    // shell for slot 2, while the early title-start lineage
                    // exposes the same seeded pose on frame 172 after the
                    // shorter pre-Start mode-$0008 setup.
                    let visible_elapsed_frames = elapsed_frames.saturating_sub(activation_frame);
                    PlayerState::from_level_entry_title_worker(worker, visible_elapsed_frames)
                })
            })
    }

    pub(crate) fn starting_level_title_active_objects(
        &self,
        elapsed_frames: usize,
    ) -> Vec<NativeObjectSnapshot> {
        if matches!(
            self.boot_state,
            BootState::StartingLevel {
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: true,
                ..
            }
        ) {
            // ASM loc_1AE2E -> loc_6E2 stops exposing the carried loc_1AB26 /
            // loc_1AD48 title workers once the hidden Load_InGame bootstrap
            // begins; the later preload-object timeline owns the trace from
            // there.
            return Vec::new();
        }
        let mut objects = self
            .starting_level_title_workers
            .iter()
            .filter(|worker| {
                let activation_frame = if matches!(
                    self.boot_state,
                    BootState::StartingLevel {
                        uses_load_ingame_bootstrap: true,
                        load_ingame_bootstrap_started: false,
                        ..
                    }
                ) {
                    level_entry_title_effective_activation_frame(worker, true)
                } else {
                    level_entry_title_effective_activation_frame(worker, false)
                };
                worker.motion.is_some() && elapsed_frames >= activation_frame
            })
            .map(|worker| {
                let activation_frame = if matches!(
                    self.boot_state,
                    BootState::StartingLevel {
                        uses_load_ingame_bootstrap: true,
                        load_ingame_bootstrap_started: false,
                        ..
                    }
                ) {
                    level_entry_title_effective_activation_frame(worker, true)
                } else {
                    level_entry_title_effective_activation_frame(worker, false)
                };
                let visible_elapsed_frames = elapsed_frames.saturating_sub(activation_frame);
                let pose =
                    PlayerState::from_level_entry_title_worker(worker, visible_elapsed_frames);
                NativeObjectSnapshot {
                    kind: NativeObjectKind::Enemy { object_type: 0 },
                    x_pos: pose.x_pos,
                    y_pos: pose.y_pos,
                    layout_index: None,
                    x_velocity: None,
                    y_velocity: None,
                    facing: Some(Direction::Left),
                    on_ground: Some(true),
                    hit_points: None,
                    damage_flash: false,
                    palette_variant: 0,
                    stationary: true,
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    wait_timer: None,
                    debug_fields: BTreeMap::from([(
                        String::from("gfx_slot_index"),
                        worker.gfx_slot_index.to_string(),
                    )]),
                }
            })
            .collect::<Vec<_>>();
        objects.sort_by_key(|object| {
            object
                .debug_fields
                .get("gfx_slot_index")
                .and_then(|slot| slot.parse::<usize>().ok())
                .unwrap_or(usize::MAX)
        });
        objects
    }
}

#[allow(non_snake_case)]
fn level_entry_title_text_level_id_loc_1AB52(level_id: LevelId) -> LevelId {
    // ASM loc_1AB40 / loc_1AB52 (kid.asm:28761-28767): after the title-letter
    // palette copy, `Current_LevelID` stays in `d7` only while it is below
    // `FirstElsewhere_LevelID`; later Elsewhere stages clamp `d7` back to the
    // first Elsewhere entry before indexing `AddrTbl_LevelNames`.
    if level_id.0 < FIRST_ELSEWHERE_LEVEL_ID {
        level_id
    } else {
        LevelId(FIRST_ELSEWHERE_LEVEL_ID)
    }
}

fn level_entry_title_target_y(glyph: u8, y: u16) -> u16 {
    if glyph == 0x83 {
        y.saturating_sub(8)
    } else {
        y
    }
}

fn level_entry_title_advance_x(glyph: u8) -> u16 {
    // ASM loc_1AB6A (kid.asm:28775-28805): after each visible title glyph
    // worker is allocated, advance `d2` by a glyph-specific width before the
    // next script byte is read.
    match glyph.wrapping_sub(b'a') {
        // ASM loc_1ABCA (kid.asm:28807-28809): the wide `$7C` title glyph
        // skips the default branch and advances the destination by `#$28`.
        0x1B => 0x28,
        // ASM loc_1ABD0 (kid.asm:28812-28814): glyph `$7D` uses its own
        // spacing branch and advances the destination by `#$20`.
        0x1C => 0x20,
        // ASM loc_1ABD6 (kid.asm:28817-28819): glyphs `$7E` and `$7F` share
        // the same spacing branch and advance the destination by `#$18`.
        0x1D | 0x1E => 0x18,
        // ASM loc_1ABDC (kid.asm:28822-28824): both lowercase `i` (`$69`)
        // and the raised `$83` glyph take the narrow spacing branch and
        // advance the destination by `#8`.
        0x08 | 0x22 => 0x08,
        _ => 0x10,
    }
}

#[allow(non_snake_case)]
fn level_entry_title_space_token_x_loc_1ABEA(x: u16) -> u16 {
    // ASM loc_1ABEA (kid.asm:28834-28837): control byte `#$84` consumes one
    // script byte with `addq.w #1,a2`, skips worker allocation, and advances
    // the current title-card destination by exactly `#$10`.
    x.saturating_add(0x10)
}

#[allow(non_snake_case)]
fn level_entry_title_deferred_number_indices_loc_1ABF4(
    level_number_glyph_count: u16,
) -> core::ops::Range<usize> {
    // ASM loc_1ABF4 (kid.asm:28840-28847): `tst.w d0; beq.s loc_1AC10`
    // skips the deferred number-worker shell when the level suffix count is
    // zero. Otherwise the fallthrough seeds loc_1AD48, whose `subq.w #1,d3`
    // plus `dbf` allocates exactly `d0` `#$7B` workers.
    0..usize::from(level_number_glyph_count)
}

#[allow(non_snake_case)]
fn level_entry_title_spawns_two_player_waiter_loc_1AC10(two_player: bool) -> bool {
    // ASM loc_1AC10 (kid.asm:28849-28856): `tst.b (Two_player_flag).w`
    // falls through to `loc_1AC26` in 1P, but allocates `loc_1AD80` before
    // the shared delete tail when alternating 2P mode is active.
    two_player
}

fn level_entry_title_letter_gfx_slot_index(
    visible_glyph_index: usize,
    visible_glyph_count: usize,
) -> usize {
    visible_glyph_count
        .saturating_sub(visible_glyph_index)
        .saturating_sub(1)
}

fn level_entry_title_effective_activation_frame(
    worker: &LevelEntryTitleWorker,
    uses_early_start_bootstrap: bool,
) -> usize {
    if uses_early_start_bootstrap {
        EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME
            + worker
                .activation_frame
                .saturating_sub(LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME)
    } else {
        worker.activation_frame
    }
}

fn level_entry_title_axis_step(start_pos: u32, target_pos: u32) -> i32 {
    // ASM loc_1ACFE (kid.asm:28955-28970): after seeding `x_pos/y_pos(a3)`
    // with `(d5,d6)`, the worker derives each 16.16 glide step from the
    // fixed-point delta over 64 frames via `muls.w #$100`, `asr.l #6`,
    // `lsl.l #8`. On native fixed-point positions that reduces to `delta >> 6`.
    (target_pos as i32).wrapping_sub(start_pos as i32) >> 6
}

#[allow(non_snake_case)]
fn level_entry_title_deferred_number_target_x_loc_1AD56(base_x: u16, number_index: usize) -> u16 {
    // ASM loc_1AD56 (kid.asm:28996-29004): each deferred number worker stores
    // the current `d1` X destination into `$46(a0)`, then advances `d1` by
    // `#$10` before the inclusive `dbf d3,loc_1AD56` back-edge.
    base_x.saturating_add(number_index as u16 * 0x10)
}

fn level_entry_title_worker_axis_pos(
    start_pos: u32,
    target_pos: u32,
    step_fp: i32,
    elapsed_frames: usize,
) -> u32 {
    // ASM loc_1AD2A -> loc_1AD3A (kid.asm:28974-28994): each wake compares X
    // first and skips the `add.l d2,x_pos(a3)` write once the destination is
    // reached, then independently applies the same compare/add loop to Y.
    let current = (start_pos as i32).wrapping_add(
        step_fp.wrapping_mul(elapsed_frames.min(LEVEL_ENTRY_TITLE_ANIMATION_FRAMES) as i32),
    );
    let target = target_pos as i32;
    if step_fp >= 0 {
        current.min(target) as u32
    } else {
        current.max(target) as u32
    }
}

fn level_entry_title_start_pose(rng: &mut RomRandomState, selector: u8) -> (u32, u32) {
    // ASM loc_1ACB0 (kid.asm:28914-28923) masks the first RNG byte to two
    // bits, then dispatches four entry poses: selectors 0/2 pin X to the left
    // and right edges while sharing the random-Y branch, selector 1 seeds the
    // top edge, and selector 3 seeds the bottom edge.
    let (x, y) = match selector {
        0 => (-0x20, level_entry_title_random_y(rng)),
        1 => (
            level_entry_title_random_x(rng),
            level_entry_title_top_edge_y_loc_1ace6(),
        ),
        2 => (
            level_entry_title_right_edge_x_loc_1ace0(),
            level_entry_title_random_y(rng),
        ),
        _ => (level_entry_title_random_x(rng), 0x100),
    };
    ((x << 16) as u32, (y << 16) as u32)
}

fn level_entry_title_right_edge_x_loc_1ace0() -> i32 {
    // ASM loc_1ACE0 (kid.asm:28937-28939) is the selector-2 branch:
    // `move.w #$160,d5` fixes X to the right title-entry edge before the
    // shared loc_1ACEA random-Y tail runs.
    0x160
}

fn level_entry_title_top_edge_y_loc_1ace6() -> i32 {
    // ASM loc_1ACE6 (kid.asm:28942-28944) is the selector-1 branch tail:
    // `move.w #$FFE0,d5` fixes Y to the top title-entry edge before the shared
    // loc_1ACEA random-Y branch resumes.
    -0x20
}

fn level_entry_title_random_y(rng: &mut RomRandomState) -> i32 {
    let mut value = i32::from(rng.next_byte()) - 0x20;
    if value <= 0 {
        value += 0x28;
    }
    level_entry_title_random_y_snap_loc_1acf8(value)
}

fn level_entry_title_random_y_snap_loc_1acf8(value: i32) -> i32 {
    // ASM loc_1ACF8 (kid.asm:28951-28952) clears the low two bits after the
    // shared loc_1ACEA bias step so the vertical-entry Y positions land on
    // 4px boundaries before moving into `d6`.
    value & !3
}

fn level_entry_title_random_x(rng: &mut RomRandomState) -> i32 {
    // ASM loc_1ACCC (kid.asm:28928-28933) is the shared selector-1/3 tail:
    // fetch a random word, keep the low byte, bias by `#$3E`, then clear the
    // low two bits so both vertical-entry branches land on 4px X increments.
    (i32::from(rng.next_word() & 0x00FF) + 0x3E) & !3
}

impl OpeningSliceState {
    pub(crate) fn new(bounds: LevelBounds) -> Self {
        Self {
            player: PlayerState {
                x_pos: 0x0020_0000,
                y_pos: OPENING_GROUNDED_KID_Y_POS,
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
                y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            },
            terrain: TerrainState {
                floor_y_pos: OPENING_GROUNDED_KID_Y_POS,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn respawn(bounds: LevelBounds) -> Self {
        Self::respawn_from_pose(
            bounds,
            LevelStartPose {
                kid_x_pos: 0x0020_0000,
                kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                camera_x_pos: 0,
                camera_y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            },
        )
    }

    pub(crate) fn respawn_from_pose(bounds: LevelBounds, pose: LevelStartPose) -> Self {
        Self {
            player: PlayerState {
                x_pos: pose.kid_x_pos,
                y_pos: pose.kid_y_pos,
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
                x_pos: pose.camera_x_pos,
                y_pos: pose.camera_y_pos,
            },
            terrain: TerrainState {
                floor_y_pos: pose.kid_y_pos,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        custom_gravity: bool,
        y_look_offset: i16,
    ) {
        self.step_with_order(
            input,
            jump_pressed,
            form,
            block_grid,
            block_list,
            custom_gravity,
            y_look_offset,
            PhysicsOrder::GravityFirst,
            true,
            &[],
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_with_order(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        custom_gravity: bool,
        y_look_offset: i16,
        physics_order: PhysicsOrder,
        allow_asm_faithful_solid_touch_snap: bool,
        deferred_prize_ceiling_cells: &[(u16, u16, DeferredPrizeBlockPhase)],
    ) {
        // ASM loc_1062 / Hibernate_UntilAnimFinished (kid.asm:1985) loops for
        // 21 frames after ledge snap, suppressing all physics.
        // Camera follow resumes on the final frame (ASM exits animation,
        // transitions to Standingstill, then main loop runs camera follow).
        if self.player.ledge_snap_cooldown > 0 {
            self.player.ledge_snap_cooldown -= 1;
            if self.player.ledge_snap_cooldown > 0 {
                return;
            }
            // Last cooldown frame: physics still suppressed but camera catches up.
            self.update_camera_follow(y_look_offset);
            return;
        }
        let mut block_grid = block_grid;
        let on_ground_before = self.player.on_ground;
        self.wall_cling_release_probe = None;
        self.kid_flip_follow_on_stages = None;
        self.kid_jump_gravity_probe = None;
        // ASM sub_914A uses FA78 (per-form half-width from unk_B408) for all
        // collision probes. Thread the form's half-width through to collision
        // functions instead of the constant PLAYER_HALF_WIDTH.
        let collision_hw = form.half_width();

        match physics_order {
            PhysicsOrder::GravityFirst => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = on_ground_before;
                }

                // Current native order: horizontal, vertical (gravity+position), collisions
                let horizontal_step_applied =
                    self.update_horizontal_motion(input, jump_pressed, form, block_grid.as_deref());

                if let Some(ref mut t) = self.physics_trace {
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }

                // Save pre-gravity y_vel for floor landing sub-pixel logic (ASM tests
                // y_vel in Character_CheckCollision, before sub_B168 applies gravity).
                let pre_gravity_y_vel = self.player.y_velocity;
                self.update_vertical_motion(input, jump_pressed, form, custom_gravity);
                self.pre_gravity_y_vel = pre_gravity_y_vel;

                if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = self.player.y_velocity;
                    t.gravity_applied =
                        self.player.y_velocity != pre_gravity_y_vel || !self.player.on_ground;
                    t.ascending_and_b_held = pre_gravity_y_vel < 0 && input.b;
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                    // Detect jump initiation: y_vel went negative from non-negative
                    if pre_gravity_y_vel >= 0 && self.player.y_velocity < 0 {
                        t.jump_initiated = true;
                        t.jump_y_vel = Some(self.player.y_velocity);
                    }
                }

                let pre_collision_x = self.player.x_pos;
                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                };
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
            }
            PhysicsOrder::PositionFirst => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = self.player.on_ground;
                }
                // ASM order: horizontal motion + position, apply y_pos from y_velocity
                // (without gravity first), collisions, then gravity
                let horizontal_step_applied =
                    self.update_horizontal_motion(input, jump_pressed, form, block_grid.as_deref());
                if let Some(ref mut t) = self.physics_trace {
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }
                // Apply vertical position from current y_velocity (before gravity)
                self.apply_vertical_position_only(input, jump_pressed, form);
                // In PositionFirst, y_vel at collision time is pre-gravity (matches ASM).
                self.pre_gravity_y_vel = self.player.y_velocity;
                if let Some(ref mut t) = self.physics_trace {
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                }
                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                };
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
                // Apply gravity after collisions (ASM sub_B168 runs after collision checks)
                let pre_post_grav_vel = self.player.y_velocity;
                if !self.player.on_ground && !custom_gravity {
                    let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                    self.player.y_velocity =
                        physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
                    if let Some(ref mut t) = self.physics_trace {
                        t.post_gravity_y_vel = self.player.y_velocity;
                        t.gravity_applied = true;
                        t.ascending_and_b_held = ascending_and_b_held;
                    }
                } else if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = pre_post_grav_vel;
                }
            }
            PhysicsOrder::AsmFaithful => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = on_ground_before;
                }

                // ASM walking coroutine (loc_8BF0): sub_B43A checks floor at
                // (y_pos + 1) / 16 across player width BEFORE any velocity/position
                // updates. If no Solid/Spikes cell found → loc_B5F6: y_pos += 1px,
                // transition to Jump, skip sub_9A0A/sub_8F26 entirely.
                let grounded_slope_before = self
                    .grounded_slope_collision(block_grid.as_deref())
                    .filter(|_| on_ground_before);
                if on_ground_before
                    && grounded_slope_before.is_none()
                    && let Some(grid) = block_grid.as_deref()
                    && !self.walking_floor_support_exists_at_current_position(grid, collision_hw)
                {
                    // loc_B5F6: addq.w #1,y_pos(a3)
                    self.player.y_pos = self.player.y_pos.wrapping_add(0x0001_0000);
                    self.player.on_ground = false;
                    self.player.on_spikes = false;
                    self.terrain.floor_terrain_type = TerrainType::Normal;
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }

                let entry_stage = self.kid_flip_follow_on_stage_state();
                let mut kid_flip_follow_on_stages = KidFlipFollowOnStages {
                    entry: entry_stage,
                    after_press_away: entry_stage,
                    after_jump_gravity: entry_stage,
                    after_floor: entry_stage,
                    after_wall: entry_stage,
                };

                // ASM loc_AA9C: pressing away from wall exits wall-cling.
                if let Some(dir) = self.player.wall_cling_direction {
                    let pressing_away = match dir {
                        Direction::Right => input.left,
                        Direction::Left => input.right,
                    };
                    if pressing_away {
                        self.player.wall_cling_direction = None;
                    }
                }
                kid_flip_follow_on_stages.after_press_away = self.kid_flip_follow_on_stage_state();

                // Step 1: sub_B084 — horizontal velocity only (no x_pos update).
                // Suppressed during wall-cling (ASM loc_AA22 never calls sub_B084).
                let horizontal_step_applied = if self.player.wall_cling_direction.is_some() {
                    false
                } else {
                    self.update_horizontal_velocity_only(
                        input,
                        jump_pressed,
                        form,
                        block_grid.as_deref(),
                    )
                };

                // ASM loc_9D22: on slopes, y_vel mirrors x_vel so the player
                // moves diagonally along the 45° surface. SlopeUp (/): going
                // right means going up → y_vel = -x_vel. SlopeDown (\): going
                // right means going down → y_vel = +x_vel.
                if self.player.on_ground
                    && let Some(grid) = block_grid.as_deref()
                {
                    let slope = self.grounded_slope_collision(Some(grid));
                    match slope {
                        Some(terrain::CollisionKind::SlopeUp) => {
                            self.player.y_velocity = -self.player.x_velocity;
                        }
                        Some(terrain::CollisionKind::SlopeDown) => {
                            self.player.y_velocity = self.player.x_velocity;
                        }
                        _ => {}
                    }
                }

                if let Some(ref mut t) = self.physics_trace {
                    // Record post-horizontal with unchanged x_pos (vel-only update)
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }

                // Step 2: sub_B168 — jump initiation + gravity (velocity only)
                let pre_gravity_y_vel = self.player.y_velocity;
                self.apply_jump_and_gravity_vel_only(input, jump_pressed, form, custom_gravity);
                // In AsmFaithful, gravity runs before position, so pre_gravity_y_vel
                // is the velocity before gravity was applied (same as GravityFirst).
                self.pre_gravity_y_vel = pre_gravity_y_vel;
                kid_flip_follow_on_stages.after_jump_gravity =
                    self.kid_flip_follow_on_stage_state();

                if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = self.player.y_velocity;
                    t.gravity_applied =
                        self.player.y_velocity != pre_gravity_y_vel || !self.player.on_ground;
                    t.ascending_and_b_held = pre_gravity_y_vel < 0 && input.b;
                    if pre_gravity_y_vel >= 0 && self.player.y_velocity < 0 {
                        t.jump_initiated = true;
                        t.jump_y_vel = Some(self.player.y_velocity);
                    }
                }

                // Step 3: sub_902A — apply y_pos += y_vel, then floor/ceiling collision.
                // ASM runs sub_902A (vertical) before sub_8F26 (horizontal), so floor
                // probes see old x_pos. This is critical at ledge edges: with old x_pos
                // the probe still finds ground, preventing a 1-frame-early fall.
                let pre_vertical_y_px = (self.player.y_pos >> 16) as i32;
                self.apply_y_velocity_to_position();

                if let Some(ref mut t) = self.physics_trace {
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                }

                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        allow_asm_faithful_solid_touch_snap,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        allow_asm_faithful_solid_touch_snap,
                    )
                };
                kid_flip_follow_on_stages.after_floor = self.kid_flip_follow_on_stage_state();
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
                if let Some(grid) = block_grid.as_deref() {
                    self.record_wall_cling_release_probe(input, form, grid, pre_vertical_y_px);
                }
                if let Some(grid) = block_grid.as_deref()
                    && self.try_release_wall_cling_overhang(form, grid, pre_vertical_y_px)
                {
                    kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                    self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }
                if let Some(grid) = block_grid.as_deref()
                    && self.try_release_descending_wall_cling_into_run(input, form, grid)
                {
                    kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                    self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }

                // Step 4: sub_8F26 — apply x_pos += x_vel, then wall collision.
                self.apply_x_velocity_to_position();

                let pre_collision_x = self.player.x_pos;
                if let Some(grid) = block_grid {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        kid_flip_follow_on_stages.after_wall =
                            self.kid_flip_follow_on_stage_state();
                        self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                        self.finalise_trace();
                        return;
                    }
                    // ASM sub_9386 + sub_A254: after horizontal movement, snap Y
                    // to the slope surface. The ROM stores a linear constant
                    // ($FFFFFA24) and computes y = constant ± x each frame.
                    // Native equivalent: compute slope_floor_y from x_px directly,
                    // checking both the current cell row AND one row below/above
                    // to handle tile boundary crossings during horizontal movement.
                    if self.player.on_ground {
                        let x_px = (self.player.x_pos >> 16) as i32;
                        let y_px = (self.player.y_pos >> 16) as i32;
                        let col = (x_px >> 4).max(0) as u16;
                        // ASM sub_9386 + sub_A254: snap Y to slope surface using
                        // the current x_pos. Search current row ± 1 to handle
                        // tile boundary crossings during horizontal movement.
                        for row_offset in [0i32, -1, 1] {
                            let row = ((y_px >> 4) + row_offset).max(0) as u16;
                            let cell = grid.cell_at(col, row);
                            if matches!(
                                cell.collision,
                                terrain::CollisionKind::SlopeUp | terrain::CollisionKind::SlopeDown
                            ) {
                                let cell_top = i32::from(row) * 16;
                                let floor_y =
                                    terrain::slope_floor_y_pub(cell.collision, cell_top, x_px);
                                if (y_px - floor_y).abs() <= 16 {
                                    self.player.y_pos = (floor_y.max(0) as u32) << 16;
                                    break;
                                }
                            }
                        }
                    }
                    if self.player.on_ground
                        && self.grounded_slope_collision(Some(grid)).is_none()
                        && !self
                            .walking_floor_support_exists_at_current_position(grid, collision_hw)
                    {
                        // ASM sub_B43A/loc_B5F6: once the stepped walking
                        // hitbox no longer overlaps any solid floor, the next
                        // state is the airborne jump path rather than another
                        // synthetic grounded hold.
                        self.player.y_pos = self.player.y_pos.wrapping_add(0x0001_0000);
                        self.player.on_ground = false;
                        self.player.on_spikes = false;
                        self.terrain.floor_terrain_type = TerrainType::Normal;
                    }
                }
                kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
            }
        }
        self.finalise_trace();
        self.update_camera_follow(y_look_offset);
    }

    /// ASM loc_7464 (kid.asm:7610–7614): when both Left and Right are held,
    /// the ROM clears `Ctrl_Right_Held` via `sf`, making left win.  Native
    /// treats simultaneous L+R as no direction (None) — the practical effect
    /// is the same for movement since the ROM's "left wins" path also
    /// produces zero net acceleration in most movement branches.
    fn held_horizontal_direction(input: FrameInput) -> Option<Direction> {
        match (input.left, input.right) {
            (true, false) => Some(Direction::Left),
            (false, true) => Some(Direction::Right),
            _ => None,
        }
    }

    fn kid_flip_follow_on_stage_state(&self) -> KidFlipFollowOnStageState {
        KidFlipFollowOnStageState {
            x_velocity: self.player.x_velocity,
            y_velocity: self.player.y_velocity,
            on_ground: self.player.on_ground,
            wall_cling_direction: self.player.wall_cling_direction,
        }
    }

    fn wall_cling_probe_direction(
        &self,
        held_direction: Option<Direction>,
    ) -> (Option<Direction>, KidWallClingProbeDirectionSource) {
        if let Some(direction) = self.player.wall_cling_direction {
            return (Some(direction), KidWallClingProbeDirectionSource::WallCling);
        }
        if let Some(direction) = held_direction {
            return (Some(direction), KidWallClingProbeDirectionSource::Held);
        }
        (
            Some(self.player.facing),
            KidWallClingProbeDirectionSource::Facing,
        )
    }

    fn wall_cling_direction_is_sticky(&self, direction: Direction) -> bool {
        match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        }
    }

    fn record_wall_cling_release_probe(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
        pre_vertical_y_px: i32,
    ) {
        let held_direction = Self::held_horizontal_direction(input);
        let (probe_direction, probe_direction_source) =
            self.wall_cling_probe_direction(held_direction);
        let (forward_wall_probe, top_edge_probe) = match probe_direction {
            Some(direction) => {
                let moving_right = direction == Direction::Right;
                (
                    Some(terrain::probe_wall(
                        grid,
                        (self.player.x_pos >> 16) as i32,
                        (self.player.y_pos >> 16) as i32,
                        form.half_width(),
                        PLAYER_HEIGHT,
                        moving_right,
                    )),
                    Some(terrain::probe_wall_top_edge(
                        grid,
                        (self.player.x_pos >> 16) as i32,
                        (self.player.y_pos >> 16) as i32,
                        form.half_width(),
                        PLAYER_HEIGHT,
                        moving_right,
                    )),
                )
            }
            None => (None, None),
        };

        let branch = match self.player.wall_cling_direction {
            Some(direction) => {
                let pressing_away = match direction {
                    Direction::Right => input.left,
                    Direction::Left => input.right,
                };
                if pressing_away {
                    KidWallClingReleaseBranch::PressAwayExit
                } else if self.player.y_velocity < 0 {
                    KidWallClingReleaseBranch::SubAe6e
                } else {
                    KidWallClingReleaseBranch::SubAdb8
                }
            }
            None => KidWallClingReleaseBranch::NoWallClingState,
        };

        let sub_adb8_gate = match self.player.wall_cling_direction {
            None => KidWallClingReleaseGate::WallClingMissing,
            Some(direction) => {
                if !self.wall_cling_direction_is_sticky(direction) {
                    KidWallClingReleaseGate::StickyInactive
                } else if self.player.on_ground {
                    KidWallClingReleaseGate::OnGround
                } else if self.player.y_velocity < 0 {
                    KidWallClingReleaseGate::Rising
                } else if held_direction != Some(direction) {
                    KidWallClingReleaseGate::HeldDirectionMismatch
                } else if forward_wall_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::ForwardWallBlocked
                } else if top_edge_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::TopEdgeBlocked
                } else {
                    KidWallClingReleaseGate::Ready
                }
            }
        };

        let post_vertical_y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = pre_vertical_y_px - post_vertical_y_px;
        let distance_to_tile = (-(post_vertical_y_px + 1)) & 0xF;
        let sub_ae6e_gate = match self.player.wall_cling_direction {
            None => KidWallClingReleaseGate::WallClingMissing,
            Some(direction) => {
                if !self.wall_cling_direction_is_sticky(direction) {
                    KidWallClingReleaseGate::StickyInactive
                } else if self.player.y_velocity >= 0 {
                    KidWallClingReleaseGate::NotAscending
                } else if upward_displacement <= distance_to_tile {
                    KidWallClingReleaseGate::NeedsTileCrossing
                } else if forward_wall_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::ForwardWallBlocked
                } else if top_edge_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::TopEdgeBlocked
                } else {
                    KidWallClingReleaseGate::Ready
                }
            }
        };

        self.wall_cling_release_probe = Some(KidWallClingReleaseProbe {
            held_direction,
            probe_direction,
            probe_direction_source,
            wall_cling_direction: self.player.wall_cling_direction,
            right_wall_sticky: self.player.right_wall_sticky,
            left_wall_sticky: self.player.left_wall_sticky,
            on_ground: self.player.on_ground,
            y_velocity: self.player.y_velocity,
            pre_vertical_y_px,
            post_vertical_y_px,
            branch,
            sub_adb8_gate,
            sub_ae6e_gate,
            forward_wall_probe,
            top_edge_probe,
        });
    }

    /// ASM loc_779C (kid.asm:7842): standing-still horizontal movement gate.
    ///
    /// The ROM checks `tst.b (Maniaxe_throwing_axe) / bne.w loc_7828` — if
    /// Maniaxe has an active axe, skip all walk checks (stay standing on
    /// platform). Then loads half-width (`$FFFFFA78`) and `x_pos(a3)`.
    ///
    /// Right held (loc_779C+8): `add.w d0,d7 / addq.w #1,d7 / bsr.w sub_922C`
    /// — probes one column at x + half_width + 1. If wall found (NE), face
    /// right (`sf x_direction`) and continue standing (loc_7828).
    ///
    /// Left held (loc_77E6, kid.asm:7871): mirror — `sub.w d0,d7 / subq.w #1,d7
    /// / bsr.w sub_922C`. If wall found, face left (`st x_direction`), loc_7828.
    ///
    /// No wall → loc_77C2 (right) / loc_77FC (left): helmet bitmask
    /// `#%1011000101` (bits 0,2,6,7,9 = Kid, Cyclone, IronKnight, Berzerker,
    /// Micromax) → loc_7820 (enter walking). For other helmets, `sub_B55C`
    /// probes (1,-1)/(−1,-1) for ledge block ($4000/$5000); if found, face
    /// direction and stay standing, otherwise loc_7820.
    ///
    /// loc_7820 (kid.asm:7896): `clr.w ($FFFFF8F0) / bra.w loc_8D72` — clears
    /// walk accumulator and enters walking state.
    ///
    /// TODO(fidelity): native uses `probe_wall` (3-point) instead of `sub_922C`
    /// (full column scan). The Maniaxe axe-throwing gate and the loc_77C2/77FC
    /// `sub_B55C` ledge probe for non-bitmask helmets are not yet modelled.
    fn should_hold_standing_still(
        &mut self,
        input: FrameInput,
        block_grid: Option<&terrain::BlockGrid>,
        collision_hw: i32,
    ) -> bool {
        if !self.player.on_ground || self.player.x_velocity != 0 {
            return false;
        }
        let Some(direction) = Self::held_horizontal_direction(input) else {
            return false;
        };
        let Some(grid) = block_grid else {
            return false;
        };

        // ASM loc_779C: sf/st x_direction(a3) — face the held direction even
        // when blocked by a wall.
        self.player.facing = direction;
        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        // ASM loc_779C/loc_77E6: sub_922C single-column wall scan at the
        // player's leading edge.  Native approximates with 3-point probe_wall.
        terrain::probe_wall(
            grid,
            x_px,
            y_px,
            collision_hw,
            PLAYER_HEIGHT,
            direction == Direction::Right,
        )
        .hit
    }

    fn try_release_wall_cling_overhang(
        &mut self,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
        pre_vertical_y_px: i32,
    ) -> bool {
        let Some(direction) = self.player.wall_cling_direction else {
            return false;
        };
        let sticky_active = match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        };
        if !sticky_active || self.player.y_velocity >= 0 {
            return false;
        }

        let y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = pre_vertical_y_px - y_px;
        let distance_to_tile = (-(y_px + 1)) & 0xF;
        if upward_displacement <= distance_to_tile {
            return false;
        }

        // ASM jump wall-cling state keeps suppressing h_accel until the first
        // upward tile-row crossing where the overhang has actually cleared.
        // On that release frame, the sticky wall hold contributes a 1px x-pos
        // shimmy and clears the cling state, which is exactly the demo-1
        // frame-2723 frontier behaviour.
        let x_px = (self.player.x_pos >> 16) as i32;
        let moving_right = direction == Direction::Right;
        if terrain::probe_wall(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }
        if terrain::probe_wall_top_edge(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }

        match direction {
            Direction::Right => self.player.x_pos = self.player.x_pos.wrapping_add(0x0001_0000),
            Direction::Left => self.player.x_pos = self.player.x_pos.wrapping_sub(0x0001_0000),
        }
        self.player.x_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        true
    }

    fn try_release_descending_wall_cling_into_run(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
    ) -> bool {
        let Some(direction) = self.player.wall_cling_direction else {
            return false;
        };
        let sticky_active = match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        };
        if !sticky_active || self.player.on_ground || self.player.y_velocity < 0 {
            return false;
        }
        if Self::held_horizontal_direction(input) != Some(direction) {
            return false;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = direction == Direction::Right;
        // ASM loc_AA9C/loc_AD34: while descending in wall-cling, holding back
        // into the wall exits cling with a 1px shimmy once the wall/top-edge
        // probes both clear, so the next frame can re-enter jump air-accel.
        if terrain::probe_wall(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }
        if terrain::probe_wall_top_edge(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }

        match direction {
            Direction::Right => self.player.x_pos = self.player.x_pos.wrapping_add(0x0001_0000),
            Direction::Left => self.player.x_pos = self.player.x_pos.wrapping_sub(0x0001_0000),
        }
        self.player.x_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        true
    }

    /// Record final state into the physics trace and detect floor landing.
    fn finalise_trace(&mut self) {
        if let Some(ref mut t) = self.physics_trace {
            t.on_ground_after = self.player.on_ground;
            t.final_pos = (self.player.x_pos, self.player.y_pos);
            t.final_vel = (self.player.x_velocity, self.player.y_velocity);
            // Detect floor landing: transitioned from airborne to grounded
            if !t.on_ground_before && t.on_ground_after {
                t.floor_hit = true;
                t.floor_y_px = Some((self.terrain.floor_y_pos >> 16) as i32);
                // sub_902A clears sub-pixel on new landings (clr.w $20(a3))
                t.landing_path = Some(LandingPath::ZeroSubPixel);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_iron_knight_climb_resume(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        y_look_offset: i16,
        allow_asm_faithful_solid_touch_snap: bool,
        deferred_prize_ceiling_cells: &[(u16, u16, DeferredPrizeBlockPhase)],
    ) {
        let stage = self.kid_flip_follow_on_stage_state();
        self.kid_flip_follow_on_stages = Some(KidFlipFollowOnStages {
            entry: stage,
            after_press_away: stage,
            after_jump_gravity: stage,
            after_floor: stage,
            after_wall: stage,
        });
        self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
            branch: KidJumpGravityBranch::AirborneCustomGravity,
            jump_pressed,
            input_b_held: input.b,
            custom_gravity: true,
            on_ground_before: self.player.on_ground,
            on_ground_after: self.player.on_ground,
            just_jumped_before: self.player.just_jumped,
            just_jumped_after: self.player.just_jumped,
            jump_buffer_frames_before: self.player.jump_buffer_frames,
            jump_buffer_frames_after: self.player.jump_buffer_frames,
            y_velocity_before: self.player.y_velocity,
            y_velocity_after: self.player.y_velocity,
        });

        if let Some(ref mut trace) = self.physics_trace {
            trace.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
            trace.post_horizontal = (self.player.x_pos, self.player.x_velocity);
            trace.horizontal_step_applied = false;
            trace.pre_vertical = (self.player.y_pos, self.player.y_velocity);
            trace.pre_gravity_y_vel = self.player.y_velocity;
            trace.post_gravity_y_vel = self.player.y_velocity;
            trace.gravity_applied = false;
            trace.ascending_and_b_held = false;
        }
        self.pre_gravity_y_vel = self.player.y_velocity;

        let pre_vertical_y_px = (self.player.y_pos >> 16) as i32;
        self.apply_y_velocity_to_position();
        if let Some(ref mut trace) = self.physics_trace {
            trace.post_vertical = (self.player.y_pos, self.player.y_velocity);
        }

        if let Some(grid) = block_grid {
            self.apply_ceiling_collision(
                Some(grid),
                block_list,
                form,
                form.half_width(),
                deferred_prize_ceiling_cells,
            );
            self.apply_floor_collision(
                Some(grid),
                false,
                !jump_pressed,
                form.half_width(),
                allow_asm_faithful_solid_touch_snap,
            );
            self.record_wall_cling_release_probe(input, form, grid, pre_vertical_y_px);
        } else {
            self.apply_floor_collision(
                None,
                false,
                !jump_pressed,
                form.half_width(),
                allow_asm_faithful_solid_touch_snap,
            );
        }

        self.finalise_trace();
        self.update_camera_follow(y_look_offset);
    }

    pub(crate) fn try_snap_wall_sticky_ledge(
        &mut self,
        jump_pressed: bool,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) -> bool {
        let jump_requested = jump_pressed || self.player.jump_buffer_frames > 0;
        if !jump_requested || self.player.on_ground {
            return false;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let (probe_x_px, direct_probe, snap_x_px) = if self.player.right_wall_sticky {
            let probe_x_px = x_px + collision_hw + 1;
            let probe = terrain::probe_floor(grid, probe_x_px, y_px);
            let support_left_px = i32::from(probe.cell_x) * 16;
            (probe_x_px, probe, support_left_px + collision_hw - 1)
        } else if self.player.left_wall_sticky {
            let probe_x_px = x_px - collision_hw - 1;
            let probe = terrain::probe_floor(grid, probe_x_px, y_px);
            let support_right_px = (i32::from(probe.cell_x) + 1) * 16 - 1;
            (probe_x_px, probe, support_right_px - collision_hw + 1)
        } else {
            return false;
        };

        let probe = if direct_probe.cell.is_solid_floor() {
            direct_probe
        } else if self.player.y_velocity >= 0 && self.player.jump_buffer_frames > 0 {
            // ASM pull-up path (loc_ABA4) searches downward from roughly 20px
            // above the current feet position and snaps onto the first solid
            // ledge it finds while the wall-cling jump request is still live.
            let search_start_y_px = y_px.saturating_sub(0x14) & !0xF;
            let Some(surface_y_px) = terrain::find_floor_below(grid, probe_x_px, search_start_y_px)
            else {
                return false;
            };
            let scanned_probe = terrain::probe_floor(grid, probe_x_px, surface_y_px);
            if !scanned_probe.cell.is_solid_floor() {
                return false;
            }
            scanned_probe
        } else {
            return false;
        };

        let (floor_y_px, contact_y_px) = match probe.cell.collision {
            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes => {
                (probe.floor_y_px.saturating_sub(1), probe.floor_y_px)
            }
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::HiddenBlock
            | terrain::CollisionKind::Empty => (probe.floor_y_px, probe.floor_y_px),
        };
        let contact_y_fp = (contact_y_px.max(0) as u32) << 16;
        let valid_contact_window = if self.player.y_velocity < 0 {
            self.player.y_pos >= contact_y_fp
                && self.player.y_pos <= contact_y_fp.saturating_add(15u32 << 16)
        } else {
            self.player.y_pos >= contact_y_fp
                && self.player.y_pos <= contact_y_fp.saturating_add(18u32 << 16)
        };
        if !valid_contact_window {
            return false;
        }

        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        self.player.x_pos = (snap_x_px.max(0) as u32) << 16;
        // Preserve sub-pixel from current position: ASM's ledge snap uses
        // move.w to set the pixel part, leaving the low word (sub-pixel) intact.
        self.player.y_pos = (floor_y_fp & 0xFFFF_0000) | (self.player.y_pos & 0x0000_FFFF);
        self.player.x_velocity = 0;
        self.player.y_velocity = 0;
        self.player.on_ground = true;
        self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        self.player.wall_cling_direction = None;
        // ASM has no jump buffer — the VBlank interrupt's Ctrl_1_Press is a
        // single-frame edge. Clear the buffer on ledge snap to prevent a
        // spurious jump on the next grounded frame.
        self.player.jump_buffer_frames = 0;
        self.player.ledge_snap_cooldown = LEDGE_SNAP_PULLUP_FRAMES;
        self.terrain.floor_y_pos = floor_y_fp;
        self.terrain.floor_terrain_type =
            grounded_surface_type_sub_8600(grid, x_px, y_px, collision_hw);
        true
    }

    /// Horizontal velocity update only (sub_B084 equivalent).
    /// Updates x_vel and facing but does NOT apply x_pos += x_vel.
    /// Returns (horizontal_step_applied, jump_suppressed_by_buffer).
    pub(crate) fn update_horizontal_velocity_only(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        if self.should_hold_standing_still(input, block_grid, form.half_width()) {
            return false;
        }
        if self.player.on_ground && self.player.jump_buffer_frames > 0 && !input.b {
            self.player.jump_buffer_frames = 0;
        }
        let grounded_slope = self
            .grounded_slope_collision(block_grid)
            .filter(|_| self.player.on_ground);
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            return false;
        }
        if self.player.on_ground && jump_pressed {
            if let Some(collision) = grounded_slope {
                let h_input = HorizontalInput {
                    left: input.left,
                    right: input.right,
                    run: input.a,
                };
                let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                );
                self.player.x_velocity = new_vel;
                self.player.facing = new_facing;
            }
            return false;
        }

        let near_floor =
            self.player.y_pos.abs_diff(self.terrain.floor_y_pos) <= GROUND_SNAP_TOLERANCE_FP;
        if !self.player.on_ground && jump_pressed && self.player.y_velocity >= 0 && near_floor {
            return false;
        }

        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = if self.player.on_ground {
            if let Some(collision) = grounded_slope {
                update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                )
            } else if form == PlayerForm::Skycutter {
                physics::update_skycutter_horizontal_velocity_sub_9A20(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                )
            } else {
                physics::update_horizontal_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    form.ground_speed_table(self.terrain.floor_terrain_type),
                )
            }
        } else if self.player.just_jumped {
            (self.player.x_velocity, self.player.facing)
        } else {
            let jump_profile = physics::JUMP_AIR_PROFILES[usize::from(form.helmet_id())];
            physics::update_jump_horizontal_velocity(
                self.player.x_velocity,
                self.player.facing,
                h_input,
                jump_profile,
            )
        };
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;
        true
    }

    /// Jump initiation + gravity on y_vel only (sub_B168 equivalent).
    /// Does NOT update x_pos or y_pos — only velocity fields.
    pub(crate) fn apply_jump_and_gravity_vel_only(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        custom_gravity: bool,
    ) {
        let before = self.player;
        // Jump initiation
        if self.player.on_ground && jump_pressed {
            // ASM loc_9F18 -> loc_B580 -> loc_A426: grounded slope jumps clear the
            // temporary slope-tracking y_vel before sub_A432 launches the jump.
            // Native mirrors x_vel into y_vel earlier in the frame, so the
            // grounded jump gate cannot require y_vel >= 0.
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
                branch: KidJumpGravityBranch::JumpPressed,
                jump_pressed,
                input_b_held: input.b,
                custom_gravity,
                on_ground_before: before.on_ground,
                on_ground_after: self.player.on_ground,
                just_jumped_before: before.just_jumped,
                just_jumped_after: self.player.just_jumped,
                jump_buffer_frames_before: before.jump_buffer_frames,
                jump_buffer_frames_after: self.player.jump_buffer_frames,
                y_velocity_before: before.y_velocity,
                y_velocity_after: self.player.y_velocity,
            });
            return;
        }

        // Jump buffer initiation
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
                branch: KidJumpGravityBranch::JumpBuffered,
                jump_pressed,
                input_b_held: input.b,
                custom_gravity,
                on_ground_before: before.on_ground,
                on_ground_after: self.player.on_ground,
                just_jumped_before: before.just_jumped,
                just_jumped_after: self.player.just_jumped,
                jump_buffer_frames_before: before.jump_buffer_frames,
                jump_buffer_frames_after: self.player.jump_buffer_frames,
                y_velocity_before: before.y_velocity,
                y_velocity_after: self.player.y_velocity,
            });
            return;
        }

        let branch = if !self.player.on_ground {
            if self.player.just_jumped {
                KidJumpGravityBranch::AirborneJustJumpedHold
            } else if custom_gravity {
                KidJumpGravityBranch::AirborneCustomGravity
            } else if self.player.y_velocity < 0 && input.b {
                KidJumpGravityBranch::AirborneHeldJumpGravity
            } else {
                KidJumpGravityBranch::AirborneDefaultGravity
            }
        } else {
            KidJumpGravityBranch::GroundedHold
        };

        // Airborne: apply gravity to y_vel only
        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            } else if !custom_gravity {
                let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                self.player.y_velocity =
                    physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
            }
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }

        self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
            branch,
            jump_pressed,
            input_b_held: input.b,
            custom_gravity,
            on_ground_before: before.on_ground,
            on_ground_after: self.player.on_ground,
            just_jumped_before: before.just_jumped,
            just_jumped_after: self.player.just_jumped,
            jump_buffer_frames_before: before.jump_buffer_frames,
            jump_buffer_frames_after: self.player.jump_buffer_frames,
            y_velocity_before: before.y_velocity,
            y_velocity_after: self.player.y_velocity,
        });
    }

    /// Apply y_vel to y_pos only. ASM sub_902A applies vertical position
    /// before sub_8F26 applies horizontal, so floor probes see old x_pos.
    fn apply_y_velocity_to_position(&mut self) {
        let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
        self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
    }

    /// Apply x_vel to x_pos only. ASM sub_8F26 runs after sub_902A.
    fn apply_x_velocity_to_position(&mut self) {
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
    }

    /// Physics-driven horizontal movement using speed tables.
    /// Uses ground tables when on_ground, air tables when airborne.
    pub(crate) fn update_horizontal_motion(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        if self.should_hold_standing_still(input, block_grid, form.half_width()) {
            return false;
        }
        if self.player.on_ground && self.player.jump_buffer_frames > 0 && !input.b {
            self.player.jump_buffer_frames = 0;
        }
        let grounded_slope = self
            .grounded_slope_collision(block_grid)
            .filter(|_| self.player.on_ground);
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            return false;
        }
        if self.player.on_ground && jump_pressed {
            if let Some(collision) = grounded_slope {
                let h_input = HorizontalInput {
                    left: input.left,
                    right: input.right,
                    run: input.a,
                };
                let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                );
                self.player.x_velocity = new_vel;
                self.player.facing = new_facing;
            }
            return false;
        }

        let near_floor =
            self.player.y_pos.abs_diff(self.terrain.floor_y_pos) <= GROUND_SNAP_TOLERANCE_FP;
        if !self.player.on_ground && jump_pressed && self.player.y_velocity >= 0 && near_floor {
            return false;
        }

        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = if self.player.on_ground {
            if let Some(collision) = grounded_slope {
                update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                )
            } else if form == PlayerForm::Skycutter {
                physics::update_skycutter_horizontal_velocity_sub_9A20(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                )
            } else {
                physics::update_horizontal_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    form.ground_speed_table(self.terrain.floor_terrain_type),
                )
            }
        } else if self.player.just_jumped {
            (self.player.x_velocity, self.player.facing)
        } else {
            let jump_profile = physics::JUMP_AIR_PROFILES[usize::from(form.helmet_id())];
            physics::update_jump_horizontal_velocity(
                self.player.x_velocity,
                self.player.facing,
                h_input,
                jump_profile,
            )
        };
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;

        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
        true
    }

    pub(crate) fn grounded_slope_collision(
        &self,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> Option<terrain::CollisionKind> {
        let grid = block_grid?;
        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        match terrain::probe_floor(grid, x_px, y_px).cell.collision {
            terrain::CollisionKind::SlopeDown => Some(terrain::CollisionKind::SlopeDown),
            terrain::CollisionKind::SlopeUp => Some(terrain::CollisionKind::SlopeUp),
            _ => None,
        }
    }

    fn walking_floor_support_exists_at_current_position(
        &self,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) -> bool {
        let x_int = (self.player.x_pos >> 16) as i16 as i32;
        let y_int = (self.player.y_pos >> 16) as i16 as i32;
        let probe_row = ((y_int + 1) >> 4).max(0) as u16;
        let left_col = ((x_int - collision_hw) >> 4).max(0) as u16;
        let right_col = ((x_int + collision_hw) >> 4)
            .max(0)
            .min(grid.width_cells as i32 - 1) as u16;
        (left_col..=right_col).any(|col| {
            matches!(
                grid.cell_at(col, probe_row).collision,
                terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
            )
        })
    }

    /// Physics-driven vertical movement: jump initiation and gravity.
    pub(crate) fn update_vertical_motion(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        custom_gravity: bool,
    ) {
        if self.player.on_ground && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            } else if !custom_gravity {
                // Normal gravity; helmet flight abilities (Skycutter/Cyclone)
                // apply their own gravity before physics runs.
                let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                self.player.y_velocity =
                    physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
            }
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }
    }

    /// PositionFirst mode: apply y_pos from y_velocity (and handle jump initiation)
    /// but defer gravity to after collisions.
    pub(crate) fn apply_vertical_position_only(
        &mut self,
        _input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
    ) {
        // Jump initiation (same as update_vertical_motion)
        if self.player.on_ground && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        // Jump buffer initiation
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        // Airborne: apply position from current velocity (no gravity yet)
        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            }
            // Position update only — gravity is deferred
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }
    }

    pub(crate) fn apply_wall_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        collision_hw: i32,
    ) {
        let Some(grid) = block_grid else {
            self.player.right_wall_sticky = false;
            self.player.left_wall_sticky = false;
            return;
        };
        if self.player.x_velocity == 0 {
            if self.player.on_ground {
                self.player.right_wall_sticky = false;
                self.player.left_wall_sticky = false;
            }
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = self.player.x_velocity > 0;
        let moving_left = self.player.x_velocity < 0;
        if !moving_right || self.player.on_ground {
            self.player.right_wall_sticky = false;
        }
        if !moving_left || self.player.on_ground {
            self.player.left_wall_sticky = false;
        }

        // ASM sub_8F26 tile-boundary guard (kid.asm:10335-10341): skip wall probe
        // if pixel displacement doesn't cross a 16px tile boundary. Without this,
        // a false probe hit zeroes sub-pixel via x_pos = clamped_center << 16.
        // Suppressed when wall_sticky is set (first frame after wall hit needs
        // re-probe even though x_vel=0; subsequent wall-cling frames with
        // x_vel=0 are handled earlier in the jump loop).
        let wall_stuck = (moving_right && self.player.right_wall_sticky)
            || (moving_left && self.player.left_wall_sticky);
        if !wall_stuck {
            let old_x_pos = self
                .player
                .x_pos
                .wrapping_sub(self.player.x_velocity as u32);
            let old_px = (old_x_pos >> 16) as i32;
            let new_px = (self.player.x_pos >> 16) as i32;
            if moving_right {
                let displacement = new_px - old_px;
                let right_edge = old_px + collision_hw;
                let distance_to_tile = (-(right_edge + 1)) & 0xF;
                // ASM: cmp.w d0,d4 / ble (skip when displacement <= distance).
                // Includes distance=0 case (player at tile boundary, no crossing).
                if displacement <= distance_to_tile {
                    return;
                }
            } else {
                let displacement = old_px - new_px;
                let left_edge = old_px - collision_hw;
                let distance_to_tile = left_edge & 0xF;
                if displacement <= distance_to_tile {
                    return;
                }
            }
        }

        // ASM sub_8F26 only latches wall-cling from the direct side-wall probe
        // (`sub_922C`). A top-edge-only overhang contact is used later by
        // loc_AAC8/sub_ADB8/sub_AE6E to retain an existing sticky cling, not to
        // create a fresh wall hit. Promoting `probe_wall_top_edge` into a new
        // collision source falsely attached Kid on the level-0x15 frame-24921
        // frontier where the oracle keeps drifting right past the overhang.
        let mut probe =
            terrain::probe_wall(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, moving_right);

        // Right wall top-edge retention (overhang sticky)
        if !probe.hit && moving_right && self.player.right_wall_sticky {
            let top_edge_probe =
                terrain::probe_wall_top_edge(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, true);
            if top_edge_probe.hit {
                probe = top_edge_probe;
            } else {
                self.player.x_pos = ((self.player.x_pos >> 16) + 1) << 16;
                self.player.x_velocity = 0;
                self.player.right_wall_sticky = false;
                return;
            }
        }

        // Left wall top-edge retention (overhang sticky)
        if !probe.hit && moving_left && self.player.left_wall_sticky {
            let top_edge_probe =
                terrain::probe_wall_top_edge(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, false);
            if top_edge_probe.hit {
                probe = top_edge_probe;
            } else {
                self.player.x_pos = ((self.player.x_pos >> 16) - 1) << 16;
                self.player.x_velocity = 0;
                self.player.left_wall_sticky = false;
                return;
            }
        }

        if probe.hit {
            if probe.cell.has_block && probe.cell.block_type == terrain::BlockType::Rubber {
                // ASM sub_8F26 -> sub_922C -> off_933E index 6 -> loc_10DA4:
                // side contact with a rubber block takes sub_8F26's special
                // `d7=-1` return, so the just-applied horizontal step is
                // clamped, the bounce effect is queued, and sufficiently fast
                // airborne motion rebounds instead of latching wall-cling.
                self.rubber_block_hit = Some((probe.cell_x, probe.cell_y));
            }
            let clamped_center = if moving_right {
                probe.wall_x_px - (collision_hw + 1)
            } else {
                // Leftward wall probes test the current left edge pixel
                // (`center_x - half_width`), so the flush non-colliding centre
                // sits exactly `half_width` pixels to the right of the wall
                // surface. The extra `+1` belongs only to the moving-right path,
                // which probes one pixel beyond the right edge.
                probe.wall_x_px + collision_hw
            };
            if moving_right && x_px < clamped_center {
                self.player.right_wall_sticky = false;
                return;
            }
            if moving_left && x_px > clamped_center {
                self.player.left_wall_sticky = false;
                return;
            }
            self.player.x_pos = (clamped_center.max(0) as u32) << 16;
            let incoming_x_velocity = self.player.x_velocity;
            self.player.x_velocity = 0;
            if probe.cell.has_block
                && probe.cell.block_type == terrain::BlockType::Rubber
                && !self.player.on_ground
            {
                self.player.wall_cling_direction = None;
                self.player.right_wall_sticky = false;
                self.player.left_wall_sticky = false;
                if incoming_x_velocity.unsigned_abs() >= 0x0000_8000 {
                    self.player.x_velocity = -incoming_x_velocity;
                }
                return;
            }
            self.player.right_wall_sticky = moving_right && !self.player.on_ground;
            self.player.left_wall_sticky = moving_left && !self.player.on_ground;
            // ASM loc_A8DE: enter wall-cling on airborne wall hit. Suppresses
            // sub_B084 (h_accel) in subsequent frames until landing or press-away.
            if !self.player.on_ground {
                self.player.wall_cling_direction = Some(if moving_right {
                    Direction::Right
                } else {
                    Direction::Left
                });
            }
        } else {
            self.player.right_wall_sticky = false;
            self.player.left_wall_sticky = false;
        }
    }

    pub(crate) fn apply_ceiling_collision(
        &mut self,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        form: PlayerForm,
        collision_hw: i32,
        deferred_prize_ceiling_cells: &[(u16, u16, DeferredPrizeBlockPhase)],
    ) {
        let Some(grid) = block_grid else { return };
        if self.player.y_velocity >= 0 {
            return;
        }

        // ASM sub_902A upward path only probes the ceiling if the current
        // movement actually crosses the next tile boundary above the head.
        // Without this guard, we keep re-hitting the same ceiling tile every
        // frame while still near it, producing a false multi-frame latch.
        let collision_height = form.ceiling_probe_height();
        let old_y_pos = self
            .player
            .y_pos
            .wrapping_sub(self.player.y_velocity as u32);
        let old_y_px = (old_y_pos >> 16) as i32;
        let new_y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = old_y_px - new_y_px;
        let top_edge_old = old_y_px - collision_height;
        let distance_to_tile = top_edge_old & 0xF;
        if upward_displacement <= distance_to_tile {
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;

        let probe = terrain::probe_ceiling(grid, x_px, y_px, collision_hw, collision_height);

        if probe.hit {
            let deferred_prize_ceiling_phase =
                deferred_prize_ceiling_cells
                    .iter()
                    .find_map(|&(col, row, phase)| {
                        (col == probe.cell_x && row == probe.cell_y).then_some(phase)
                    });
            if deferred_prize_ceiling_phase == Some(DeferredPrizeBlockPhase::FreshSmash) {
                return;
            }
            let deferred_prize_ceiling_cell = deferred_prize_ceiling_phase.is_some();
            // Hidden block reveal: headbutting a hidden block from below
            // changes it to solid, making the block visible and functional.
            if !deferred_prize_ceiling_cell
                && probe.cell.collision == terrain::CollisionKind::HiddenBlock
            {
                let layout_addr = ((u32::from(probe.cell_y) * u32::from(grid.width_cells))
                    + u32::from(probe.cell_x))
                    * 2;
                let revealed_cell = block_list
                    .and_then(|bl| bl.hidden_block_by_layout_addr_sub_E37A(layout_addr as u16))
                    .and_then(|entry| {
                        // ASM sub_E3C6 derives the revealed cell from the
                        // hidden-block metadata record rather than trusting the
                        // placeholder HiddenBlock cell still in Level_Layout.
                        block_list.and_then(|bl| bl.hidden_block_reveal_sub_E3C6(entry))
                    })
                    .map(|reveal| reveal.cell)
                    .unwrap_or_else(|| {
                        // Fallback for native-only fixtures that do not carry a
                        // full ROM Block List entry: preserve the existing block
                        // type/skin and only flip the collision to Solid.
                        let block_flag: u8 = if probe.cell.has_block { 0x80 } else { 0 };
                        let block_type_bits = probe.cell.block_type as u8 & 0x0F;
                        let new_hi = block_flag | 0x60 | block_type_bits;
                        terrain::Cell::from_be_bytes(new_hi, probe.cell.skin)
                    });
                grid.set_cell_at(probe.cell_x, probe.cell_y, revealed_cell);

                // If the hidden block contains a prize, dispatch it
                if probe.cell.has_block && probe.cell.block_type == terrain::BlockType::Prize {
                    let content = block_list
                        .and_then(|bl| bl.prize_content_at(probe.cell_x, probe.cell_y))
                        .unwrap_or(terrain::PrizeContent::Diamond);
                    let block_x_px = probe.cell_x * 16;
                    let block_y_px = probe.cell_y * 16;
                    self.prize_block_hit = Some((content, block_x_px, block_y_px));
                    // ASM sub_1007A: keep cell Solid, clear has_block
                    grid.set_cell_at(
                        probe.cell_x,
                        probe.cell_y,
                        terrain::Cell::from_be_bytes(0x6F, probe.cell.skin),
                    );
                    self.ceiling_prize_clear = Some((probe.cell_x, probe.cell_y, probe.cell));
                }
            }

            // Prize/smash/lift block headbutt: hitting from below.
            if !deferred_prize_ceiling_cell
                && probe.cell.collision == terrain::CollisionKind::Solid
                && probe.cell.has_block
            {
                match probe.cell.block_type {
                    terrain::BlockType::Prize => {
                        let content = block_list
                            .and_then(|bl| bl.prize_content_at(probe.cell_x, probe.cell_y))
                            .unwrap_or(terrain::PrizeContent::Diamond);
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.prize_block_hit = Some((content, block_x_px, block_y_px));
                        // ASM sub_1007A writes $6F to the cell: Solid collision,
                        // has_block=false, block_type=Deleted. The tile stays solid
                        // as structural ceiling for ~32 frames (bounce animation),
                        // then clears. Without this, Cyclone flight near a Prize
                        // ceiling clears the tile to Empty immediately, letting
                        // the player fly through.
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0x6F, probe.cell.skin),
                        );
                        self.ceiling_prize_clear = Some((probe.cell_x, probe.cell_y, probe.cell));
                        // ASM sub_914A scans ALL tiles across player width, so
                        // adjacent Smash blocks are cleared on the same frame.
                        // probe_ceiling returns on the first hit, missing the
                        // other edge. Clear adjacent Smash blocks now.
                        let top_y = y_px - collision_height;
                        let tile_row = (top_y / 16).max(0) as u16;
                        for &edge_x in &[x_px - collision_hw, x_px + collision_hw] {
                            let col = (edge_x / 16).max(0) as u16;
                            if col == probe.cell_x && tile_row == probe.cell_y {
                                continue;
                            }
                            let cell = grid.cell_at(col, tile_row);
                            if cell.collision == terrain::CollisionKind::Solid
                                && cell.has_block
                                && cell.block_type == terrain::BlockType::Smash
                            {
                                grid.set_cell_at(col, tile_row, terrain::Cell::from_be_bytes(0, 0));
                            }
                        }
                    }
                    terrain::BlockType::Smash => {
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.smash_block_hit = Some((block_x_px, block_y_px));
                        // ASM j_Block_LoadSmashBits clears Smash blocks immediately
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Lift => {
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.lift_block_hit = Some((block_x_px, block_y_px));
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Drill => {
                        let drill_x_fp = (u32::from(probe.cell_x) * 16 + 8) << 16;
                        let drill_y_fp = (u32::from(probe.cell_y) * 16 + 16) << 16;
                        self.drill_projectile_hit = Some((drill_x_fp, drill_y_fp));
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Metal => {
                        self.metal_block_hit = Some((probe.cell_x, probe.cell_y));
                    }
                    terrain::BlockType::Shooter => {
                        // ASM loc_BBC2 -> loc_11430 routes shooter ceiling
                        // hits into the deleted-tile worker coroutines instead
                        // of clearing the block immediately.
                        self.shooter_block_hit = Some((probe.cell_x, probe.cell_y));
                    }
                    terrain::BlockType::Rubber => {
                        // ASM sub_902A -> sub_914A -> loc_91F6/loc_10DA4:
                        // ceiling contact with rubber also spawns the jiggle
                        // effect without deleting the block.
                        self.rubber_block_hit = Some((probe.cell_x, probe.cell_y));
                    }
                    _ => {}
                }
            }

            // Push feet down so head clears the ceiling: feet = ceiling_bottom + height
            let new_y = probe.ceiling_y_px + collision_height;
            self.player.y_pos = (new_y.max(0) as u32) << 16;
            // ASM sub_902A ceiling path (loc_908C): clr.l y_vel(a3) then
            // clr.w $20(a3) (zero sub-pixel). y_pos is already pixel-aligned
            // from the << 16 above (sub-pixel zeroed). sub_BB54 also re-zeros
            // y_vel at line 15092.
            self.player.y_velocity = 0;
        }
    }

    pub(crate) fn apply_floor_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        horizontal_step_applied: bool,
        allow_upward_slope_snap: bool,
        collision_hw: i32,
        allow_solid_touch_snap: bool,
    ) -> Option<terrain::CollisionKind> {
        if let Some(grid) = block_grid {
            let was_on_ground = self.player.on_ground;
            let x_px = (self.player.x_pos >> 16) as i32;
            let y_px = (self.player.y_pos >> 16) as i32;
            let left_edge_probe = terrain::probe_floor(grid, x_px - collision_hw, y_px);
            let right_edge_probe = terrain::probe_floor(grid, x_px + collision_hw, y_px);
            let mut probe = terrain::probe_floor_hitbox(grid, x_px, y_px, collision_hw);
            if was_on_ground
                && matches!(
                    probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                )
            {
                let left_edge_support =
                    left_edge_probe.on_ground || left_edge_probe.cell.is_solid_floor();
                let right_edge_support =
                    right_edge_probe.on_ground || right_edge_probe.cell.is_solid_floor();
                let edge_probe = match (left_edge_support, right_edge_support) {
                    (true, true) => {
                        if left_edge_probe.floor_y_px >= right_edge_probe.floor_y_px {
                            Some(left_edge_probe)
                        } else {
                            Some(right_edge_probe)
                        }
                    }
                    (true, false) => Some(left_edge_probe),
                    (false, true) => Some(right_edge_probe),
                    (false, false) => None,
                };
                if let Some(edge_probe) = edge_probe
                    && edge_probe.floor_y_px > probe.floor_y_px
                {
                    // ASM sub_914A keeps the previously grounded support under
                    // either hitbox edge while a flat-to-slope seam is still
                    // carrying Kid. The death-respawn Level 1 run-up stays on
                    // the deeper flat plateau through frame 1012 instead of
                    // dropping onto the adjacent slope early.
                    probe = edge_probe;
                }
            }
            let mut allow_hitbox_snap = true;
            let mut center_slope_snap_candidate = false;
            if !was_on_ground {
                let center_probe = terrain::probe_floor(grid, x_px, y_px);
                if matches!(
                    center_probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                ) {
                    let center_floor_fp = (center_probe.floor_y_px.max(0) as u32) << 16;
                    let near_center_slope = self.player.y_pos >= center_floor_fp
                        || self.player.y_pos.abs_diff(center_floor_fp)
                            <= SLOPE_AIR_SNAP_TOLERANCE_FP;
                    if near_center_slope {
                        probe = center_probe;
                        center_slope_snap_candidate = true;
                    } else {
                        allow_hitbox_snap = matches!(
                            left_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        ) || matches!(
                            right_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    }
                }
            }
            let preserving_ground_contact = self.player.on_ground && probe.cell.is_solid_floor();
            if center_slope_snap_candidate
                || (allow_hitbox_snap && (probe.on_ground || preserving_ground_contact))
            {
                let (floor_y_px, contact_y_px) = match probe.cell.collision {
                    terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes => {
                        (probe.floor_y_px.saturating_sub(1), probe.floor_y_px)
                    }
                    terrain::CollisionKind::SlopeDown
                    | terrain::CollisionKind::SlopeUp
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Empty => (probe.floor_y_px, probe.floor_y_px),
                };
                let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
                let contact_y_fp = (contact_y_px.max(0) as u32) << 16;
                self.terrain.floor_y_pos = floor_y_fp;
                let within_snap =
                    self.player.y_pos.abs_diff(floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP;
                let can_land = match probe.cell.collision {
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp => {
                        let within_slope_touch_snap =
                            self.player.y_pos.abs_diff(floor_y_fp) <= 0x0001_0000;
                        (self.player.y_velocity >= 0
                            && (self.player.y_pos >= floor_y_fp || within_slope_touch_snap))
                            || (allow_upward_slope_snap
                                && center_slope_snap_candidate
                                && self.player.y_velocity < 0
                                && self.player.y_velocity >= -(SLOPE_AIR_SNAP_TOLERANCE_FP as i32)
                                && (self.player.y_pos >= floor_y_fp || within_snap))
                    }
                    terrain::CollisionKind::Solid
                    | terrain::CollisionKind::Spikes
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Empty => {
                        // ORIGINAL_QUIRK: ASM sub_902A only lands after the
                        // falling step reaches the solid contact row. The
                        // AsmFaithful path must not collapse a remaining
                        // sub-half-pixel gap into an early landing.
                        let within_solid_touch_snap = allow_solid_touch_snap
                            && !was_on_ground
                            && self.player.y_pos.abs_diff(floor_y_fp) < 0x0000_8000;
                        self.player.y_velocity >= 0
                            && (self.player.y_pos >= contact_y_fp
                                || within_solid_touch_snap
                                || (was_on_ground && within_snap))
                    }
                };
                if can_land {
                    let rubber_rebound = !was_on_ground
                        && probe.cell.has_block
                        && probe.cell.block_type == terrain::BlockType::Rubber;
                    let should_apply_slope_recontact = !was_on_ground
                        && self.player.y_velocity < 0
                        && matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    if !was_on_ground
                        && horizontal_step_applied
                        && !rubber_rebound
                        && !matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        )
                    {
                        let landing_x =
                            i64::from(self.player.x_pos) - i64::from(self.player.x_velocity);
                        self.player.x_pos = landing_x.clamp(0, i64::from(u32::MAX)) as u32;
                    }
                    // ASM sub_902A floor collision (kid.asm:10461-10462):
                    // When falling through a floor tile, sub_902A snaps y_pos
                    // and does `clr.w $20(a3)` (clears sub-pixel). But when
                    // already on the ground (y_vel=0, no tile crossing), sub_902A
                    // exits early via loc_913A without touching y_pos. So we
                    // only clear sub-pixel on actual landings (airborne→grounded).
                    if !was_on_ground {
                        // New landing: clear sub-pixel (sub_902A clr.w $20(a3))
                        self.player.y_pos = floor_y_fp & 0xFFFF_0000;
                    } else {
                        // Already grounded: preserve sub-pixel (sub_902A exits early)
                        self.player.y_pos =
                            (floor_y_fp & 0xFFFF_0000) | (self.player.y_pos & 0x0000_FFFF);
                    }
                    let incoming_y_velocity = self.player.y_velocity;
                    if rubber_rebound {
                        // ASM sub_902A -> sub_914A -> loc_91F6 -> loc_9082:
                        // a fresh downward contact on rubber snaps to the
                        // contact row, clears the sub-pixel, then restores the
                        // negated incoming y_vel instead of settling grounded.
                        // The rebound branch keeps the frame's horizontal carry;
                        // unlike the normal landing path, loc_9082 does not
                        // rewind x_pos after the contact-row snap.
                        self.player.on_ground = false;
                        self.player.y_velocity = -incoming_y_velocity;
                        self.player.on_spikes = false;
                        self.terrain.floor_terrain_type =
                            grounded_surface_type_sub_8600(grid, x_px, y_px, collision_hw);
                        self.rubber_block_hit = Some((probe.cell_x, probe.cell_y));
                        self.player.wall_cling_direction = None;
                        return None;
                    }
                    // ASM off_933E: Mushroom (index 9) is a NOP — no bounce.
                    // Non-rubber block types land normally here.
                    self.player.on_ground = true;
                    self.player.y_velocity = 0;
                    self.player.wall_cling_direction = None;
                    if !was_on_ground {
                        // ASM loc_9F18 consumes only the single-frame
                        // `Ctrl_Pressed` edge. A generic buffered jump request
                        // must not survive a fresh floor landing into the next
                        // grounded frame, or native auto-jumps one frame early
                        // on the Level 1 death-respawn route.
                        self.player.jump_buffer_frames = 0;
                    }
                    // Teleporter blocks warp player on contact
                    if probe.cell.has_block
                        && probe.cell.block_type == terrain::BlockType::Teleporter
                    {
                        let tp_col = (x_px / 16).max(0) as u16;
                        let tp_row = (probe.floor_y_px / 16).max(0) as u16;
                        self.teleporter_contact = Some((tp_col, tp_row));
                    }
                    self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
                    self.terrain.floor_terrain_type =
                        grounded_surface_type_sub_8600(grid, x_px, y_px, collision_hw);
                    if should_apply_slope_recontact {
                        return match probe.cell.collision {
                            terrain::CollisionKind::SlopeDown => {
                                Some(terrain::CollisionKind::SlopeDown)
                            }
                            terrain::CollisionKind::SlopeUp => {
                                Some(terrain::CollisionKind::SlopeUp)
                            }
                            _ => None,
                        };
                    }
                }
            } else if self.player.on_ground {
                // Was on ground but no longer — start falling
                self.player.on_ground = false;
                self.player.on_spikes = false;
                self.terrain.floor_terrain_type = TerrainType::Normal;
            }
        } else if self.player.on_ground {
            self.player.on_ground = false;
            self.player.on_spikes = false;
            self.terrain.floor_terrain_type = TerrainType::Normal;
        }
        None
    }

    pub(crate) fn apply_immediate_slope_recontact(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        collision: terrain::CollisionKind,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) {
        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
            self.player.x_velocity,
            self.player.facing,
            h_input,
            collision,
            form,
            self.terrain.floor_terrain_type,
        );
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let probe = terrain::probe_floor_hitbox(grid, x_px, y_px, collision_hw);
        let floor_y_px = match probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::HiddenBlock
            | terrain::CollisionKind::Spikes => probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => probe.floor_y_px,
        };
        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        self.terrain.floor_y_pos = floor_y_fp;
        self.player.y_pos = floor_y_fp;
        self.player.on_ground = true;
        self.player.y_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
    }

    /// Camera follows player using the real sub_71E4 follow algorithm.
    /// `y_look_offset` is the down-look accumulator ($FFFFFB58): when >= 30,
    /// shifts the camera target downward by that many pixels.
    ///
    /// The Y-axis path in sub_71E4 has two look-offset pull-back branches
    /// (loc_726C, loc_727E) that modify `$FFFFFB58` before the loc_728C offset
    /// application. Those adjustments are applied once per frame in the game
    /// tick (lib.rs) before this function is called, so `y_look_offset` arrives
    /// already adjusted.
    pub(crate) fn update_camera_follow(&mut self, y_look_offset: i16) {
        let player_x_px = (self.player.x_pos >> 16) as i32;
        let mut player_y_px = (self.player.y_pos >> 16) as i32;
        // ASM sub_71E4 loc_728C: when offset >= $1E, add it to camera Y target
        // and subtract the reference $1E. This effectively shifts the view down.
        if y_look_offset >= 0x1E {
            player_y_px += i32::from(y_look_offset);
        }
        let camera_x_px = (self.camera.x_pos >> 16) as i32;
        let camera_y_px = (self.camera.y_pos >> 16) as i32;

        let (new_cx, new_cy) = physics::update_camera(
            camera_x_px,
            camera_y_px,
            player_x_px,
            player_y_px,
            &self.level_bounds,
        );

        self.camera.x_pos = (new_cx.max(0) as u32) << 16;
        self.camera.y_pos = (new_cy.max(0) as u32) << 16;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DownHoldAlignmentSub7A10 {
    pub(crate) x_pos: u32,
    pub(crate) blocked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct CrawlWallEscapeSub81B4 {
    pub(crate) x_pos: u32,
    pub(crate) blocked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct CrawlSlopeSnapSub83E6 {
    pub(crate) x_pos: u32,
    pub(crate) adjusted: bool,
}

pub(crate) fn update_level_one_slope_ground_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    collision: terrain::CollisionKind,
    form: PlayerForm,
    terrain_type: TerrainType,
) -> (i32, Direction) {
    let _ = terrain_type;
    let _ = facing;
    let slope_descends_right = collision == terrain::CollisionKind::SlopeDown;
    // ASM sub_9386 stores `$FFFFFA26=0` for `$4000` SlopeUp and `$FF` for
    // `$5000` SlopeDown. loc_9D94 only negates/swallows inputs when `$FA26`
    // is set, so the canonical slope frame keeps live `x_vel` for SlopeUp and
    // flips it for SlopeDown.
    let mut canonical_vel = if slope_descends_right { -x_vel } else { x_vel };

    // ASM loc_9D22 uses three slope-velocity families: the shared `unk_9EB0`
    // table for Kid/Skycutter/Cyclone/Juggernaut/IronKnight/Berzerker, the
    // Micromax-specific `unk_9EE4` table, and the simplified loc_9E6C branch
    // for Eyeclops/Red Stealth/Maniaxe.
    if matches!(
        form,
        PlayerForm::RedStealth | PlayerForm::Eyeclops | PlayerForm::Maniaxe
    ) {
        let next_x_vel = if slope_descends_right {
            let advanced = x_vel + SPECIAL_FORM_SLOPE_STEP_FP;
            if advanced >= 0 {
                SPECIAL_FORM_SLOPE_STEP_FP
            } else {
                advanced
            }
        } else {
            let advanced = x_vel - SPECIAL_FORM_SLOPE_STEP_FP;
            if advanced < 0 {
                -SPECIAL_FORM_SLOPE_STEP_FP
            } else {
                advanced
            }
        };
        let next_facing = if next_x_vel >= 0 {
            Direction::Right
        } else {
            Direction::Left
        };
        return (next_x_vel, next_facing);
    }

    let (
        max_speed,
        brake,
        neutral_decel,
        neutral_target,
        uphill_cap,
        uphill_accel,
        downhill_cap,
        downhill_accel,
    ) = if input.run {
        match form {
            PlayerForm::Micromax => (
                MICROMAX_SLOPE_MAX_SPEED_FP,
                MICROMAX_SLOPE_BRAKE_FP,
                MICROMAX_SLOPE_NEUTRAL_DECEL_FP,
                MICROMAX_SLOPE_NEUTRAL_TARGET_FP,
                MICROMAX_SLOPE_RUN_UPHILL_CAP_FP,
                MICROMAX_SLOPE_RUN_UPHILL_ACCEL_FP,
                MICROMAX_SLOPE_RUN_DOWNHILL_CAP_FP,
                MICROMAX_SLOPE_RUN_DOWNHILL_ACCEL_FP,
            ),
            _ => (
                KID_SLOPE_MAX_SPEED_FP,
                KID_SLOPE_BRAKE_FP,
                KID_SLOPE_NEUTRAL_DECEL_FP,
                KID_SLOPE_NEUTRAL_TARGET_FP,
                KID_SLOPE_RUN_UPHILL_CAP_FP,
                KID_SLOPE_RUN_UPHILL_ACCEL_FP,
                KID_SLOPE_RUN_DOWNHILL_CAP_FP,
                KID_SLOPE_RUN_DOWNHILL_ACCEL_FP,
            ),
        }
    } else {
        match form {
            PlayerForm::Micromax => (
                MICROMAX_SLOPE_MAX_SPEED_FP,
                MICROMAX_SLOPE_BRAKE_FP,
                MICROMAX_SLOPE_NEUTRAL_DECEL_FP,
                MICROMAX_SLOPE_NEUTRAL_TARGET_FP,
                MICROMAX_SLOPE_WALK_UPHILL_CAP_FP,
                MICROMAX_SLOPE_WALK_UPHILL_ACCEL_FP,
                MICROMAX_SLOPE_WALK_DOWNHILL_CAP_FP,
                MICROMAX_SLOPE_WALK_DOWNHILL_ACCEL_FP,
            ),
            _ => (
                KID_SLOPE_MAX_SPEED_FP,
                KID_SLOPE_BRAKE_FP,
                KID_SLOPE_NEUTRAL_DECEL_FP,
                KID_SLOPE_NEUTRAL_TARGET_FP,
                KID_SLOPE_WALK_UPHILL_CAP_FP,
                KID_SLOPE_WALK_UPHILL_ACCEL_FP,
                KID_SLOPE_WALK_DOWNHILL_CAP_FP,
                KID_SLOPE_WALK_DOWNHILL_ACCEL_FP,
            ),
        }
    };

    let uphill_pressed = if slope_descends_right {
        input.left
    } else {
        input.right
    };
    let downhill_pressed = if slope_descends_right {
        input.right
    } else {
        input.left
    };

    if uphill_pressed {
        if canonical_vel < 0 {
            canonical_vel += uphill_accel * 2;
        } else if canonical_vel <= uphill_cap {
            canonical_vel += uphill_accel;
            if canonical_vel > uphill_cap {
                canonical_vel = uphill_cap;
            }
        } else {
            canonical_vel -= brake;
            if canonical_vel > max_speed {
                canonical_vel = max_speed;
            }
        }
    } else if downhill_pressed {
        if canonical_vel >= 0 {
            canonical_vel -= downhill_accel * 2;
        } else if canonical_vel >= downhill_cap {
            canonical_vel -= downhill_accel;
            if canonical_vel < downhill_cap {
                canonical_vel = downhill_cap;
            }
        } else {
            canonical_vel += brake;
            if canonical_vel < -max_speed {
                canonical_vel = -max_speed;
            }
        }
    } else if canonical_vel < neutral_target {
        canonical_vel += neutral_decel;
        if canonical_vel > neutral_target {
            canonical_vel = neutral_target;
        }
    } else {
        canonical_vel -= neutral_decel;
        if canonical_vel < neutral_target {
            canonical_vel = neutral_target;
        }
    }

    let next_facing = if uphill_pressed {
        if slope_descends_right {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if downhill_pressed {
        if slope_descends_right {
            Direction::Left
        } else {
            Direction::Right
        }
    } else if slope_descends_right == (canonical_vel >= 0) {
        Direction::Right
    } else {
        Direction::Left
    };

    let next_x_vel = if slope_descends_right {
        -canonical_vel
    } else {
        canonical_vel
    };
    (next_x_vel, next_facing)
}

#[allow(non_snake_case)]
pub(crate) fn align_player_for_down_hold_sub_7A10(
    grid: &terrain::BlockGrid,
    x_pos: u32,
    y_pos: u32,
    level_width_pixels: u16,
) -> DownHoldAlignmentSub7A10 {
    // ASM sub_7A10 (kid.asm:8099): checks whether Kid has 14px clearance on
    // both sides for the crouch/down-hold pose. Probes left then right; if
    // either side is occupied or out-of-bounds, snaps to the nearest tile
    // boundary and re-probes. Returns d7=0 (clear) or d7=1 (blocked).
    let original_x_px = (x_pos >> 16) as i32;
    let foot_y_px = (y_pos >> 16) as i32;
    let mut aligned_x_px = original_x_px;
    let half_width = 14i32;
    let level_width_px = i32::from(level_width_pixels);
    let occupied = |center_x_px: i32, x_offset_px: i32| {
        !matches!(
            terrain::probe_relative_collision_sub_B55C(
                grid,
                center_x_px,
                foot_y_px,
                x_offset_px,
                0,
            ),
            terrain::CollisionKind::Empty
        )
    };

    if original_x_px - half_width < 0 || occupied(original_x_px, -half_width) {
        // ASM loc_7A60 (kid.asm:8126): left probe blocked — snap x_pos right
        // to the next tile boundary: `andi.w #$FFF0,d7; addi.w #$10,d7;
        // add.w d4,d7`. Then probe the right side from the new position.
        aligned_x_px = ((original_x_px - half_width) & !0xF) + 0x10 + half_width;
        if aligned_x_px + half_width >= level_width_px || occupied(aligned_x_px, half_width) {
            // ASM loc_7A90 (kid.asm:8143): right side also blocked after snap
            // — `move.w (sp)+,x_pos(a3); moveq #1,d7; rts`. Restore original
            // x_pos and return blocked.
            return DownHoldAlignmentSub7A10 {
                x_pos: (original_x_px.max(0) as u32) << 16,
                blocked: true,
            };
        }
    } else if original_x_px + half_width >= level_width_px || occupied(original_x_px, half_width) {
        // ASM loc_7A98 (kid.asm:8149): right probe blocked — snap x_pos left
        // by `(x+14) & $F + 1` pixels. Then probe the left side.
        let adjust = ((original_x_px + half_width) & 0xF) + 1;
        aligned_x_px -= adjust;
        if aligned_x_px - half_width < 0 || occupied(aligned_x_px, -half_width) {
            // ASM loc_7A90 (kid.asm:8143): left side also blocked after snap
            // — restore original x_pos and return blocked.
            return DownHoldAlignmentSub7A10 {
                x_pos: (original_x_px.max(0) as u32) << 16,
                blocked: true,
            };
        }
    }

    // ASM loc_7AC6 (kid.asm:8168): both sides clear — `addq.w #2,sp;
    // moveq #0,d7; rts`. Discard saved x_pos, return unblocked.
    DownHoldAlignmentSub7A10 {
        x_pos: (aligned_x_px.max(0) as u32) << 16,
        blocked: false,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn grounded_surface_type_sub_8600(
    grid: &terrain::BlockGrid,
    player_x_px: i32,
    player_y_px: i32,
    half_width_px: i32,
) -> TerrainType {
    // ASM sub_8600 (kid.asm:9445): scan the foot row under Kid's full width,
    // count solid/spike support cells, then promote ice only on a strict
    // majority while rubber wins ties.
    let min_x_px = (player_x_px - half_width_px).max(0);
    let max_x_px = (player_x_px + half_width_px).clamp(0, grid.width_pixels().saturating_sub(1));
    let start_col = (min_x_px / 16).max(0) as u16;
    let end_col = (max_x_px / 16).max(0) as u16;
    let row = ((player_y_px + 1).max(0) / 16) as u16;

    let mut solid_count = 0u16;
    let mut ice_count = 0u16;
    let mut rubber_count = 0u16;

    for col in start_col..=end_col {
        let cell = grid.cell_at(col, row);
        if matches!(
            cell.collision,
            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
        ) {
            solid_count = solid_count.saturating_add(1);
        }
        if cell.has_block {
            match cell.block_type {
                terrain::BlockType::Ice => ice_count = ice_count.saturating_add(1),
                terrain::BlockType::Rubber => rubber_count = rubber_count.saturating_add(1),
                _ => {}
            }
        }
    }

    let mut terrain_type = TerrainType::Normal;
    if ice_count.saturating_mul(2) > solid_count {
        terrain_type = TerrainType::Ice;
    }
    if solid_count != 0 && rubber_count.saturating_mul(2) >= solid_count {
        terrain_type = TerrainType::Rubber;
    }
    terrain_type
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn crawl_wall_escape_sub_81B4(
    grid: &terrain::BlockGrid,
    x_pos: u32,
    y_pos: u32,
    half_width: i32,
) -> CrawlWallEscapeSub81B4 {
    // ASM sub_81B4 (kid.asm:8952): probe one row 16px above the feet at
    // `x_pos ± half_width`, take the first non-solid side, and snap the player
    // flush to that tile boundary. Only return blocked when both side probes
    // are solid (`$6000`).
    let center_x_px = (x_pos >> 16) as i32;
    let foot_y_px = (y_pos >> 16) as i32;
    let sample_y_offset_px = -16;
    let original_x_pos = (center_x_px.max(0) as u32) << 16;

    if terrain::probe_relative_collision_sub_B55C(
        grid,
        center_x_px,
        foot_y_px,
        -half_width,
        sample_y_offset_px,
    ) != terrain::CollisionKind::Solid
    {
        let snapped_x_px = ((center_x_px - half_width) & !0xF) + 0xF - half_width;
        return CrawlWallEscapeSub81B4 {
            x_pos: (snapped_x_px.max(0) as u32) << 16,
            blocked: false,
        };
    }

    if terrain::probe_relative_collision_sub_B55C(
        grid,
        center_x_px,
        foot_y_px,
        half_width,
        sample_y_offset_px,
    ) != terrain::CollisionKind::Solid
    {
        let snapped_x_px = ((center_x_px + half_width) & !0xF) + half_width;
        return CrawlWallEscapeSub81B4 {
            x_pos: (snapped_x_px.max(0) as u32) << 16,
            blocked: false,
        };
    }

    CrawlWallEscapeSub81B4 {
        x_pos: original_x_pos,
        blocked: true,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn crawl_slope_snap_sub_83E6(
    grid: &terrain::BlockGrid,
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    half_width: i32,
) -> CrawlSlopeSnapSub83E6 {
    // ASM sub_83E6 (kid.asm:9163): when crawling without a direct wall hit,
    // inspect the motion-side foot cell. Rightward or neutral motion accepts
    // only `$4000` slope-up and nudges left by the low nibble + 1; leftward
    // motion accepts only `$5000` slope-down and snaps to the current tile
    // column plus `half_width`. The ROM also clears `$1C(a3)` here, but that
    // y-size scratch word is not modelled in native state.
    let center_x_px = (x_pos >> 16) as i32;
    let foot_y_px = (y_pos >> 16) as i32;
    let original_x_pos = (center_x_px.max(0) as u32) << 16;

    if x_velocity >= 0 {
        if terrain::probe_relative_collision_sub_B55C(grid, center_x_px, foot_y_px, half_width, 0)
            == terrain::CollisionKind::SlopeUp
        {
            let adjust_px = ((center_x_px + half_width) & 0xF) + 1;
            let snapped_x_px = center_x_px - adjust_px;
            return CrawlSlopeSnapSub83E6 {
                x_pos: (snapped_x_px.max(0) as u32) << 16,
                adjusted: true,
            };
        }
    } else if terrain::probe_relative_collision_sub_B55C(
        grid,
        center_x_px,
        foot_y_px,
        -half_width,
        0,
    ) == terrain::CollisionKind::SlopeDown
    {
        let snapped_x_px = (center_x_px & !0xF) + half_width;
        return CrawlSlopeSnapSub83E6 {
            x_pos: (snapped_x_px.max(0) as u32) << 16,
            adjusted: true,
        };
    }

    CrawlSlopeSnapSub83E6 {
        x_pos: original_x_pos,
        adjusted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        level_entry_title_advance_x, level_entry_title_axis_step,
        level_entry_title_deferred_number_indices_loc_1ABF4,
        level_entry_title_deferred_number_target_x_loc_1AD56,
        level_entry_title_random_y_snap_loc_1acf8, level_entry_title_right_edge_x_loc_1ace0,
        level_entry_title_space_token_x_loc_1ABEA,
        level_entry_title_spawns_two_player_waiter_loc_1AC10, level_entry_title_start_pose,
        level_entry_title_top_edge_y_loc_1ace6, level_entry_title_worker_axis_pos,
    };
    use crate::types::RomRandomState;

    #[test]
    fn level_entry_title_space_token_adds_10_pixels_loc_1abea() {
        assert_eq!(
            level_entry_title_space_token_x_loc_1ABEA(0x20),
            0x30,
            "ASM loc_1ABEA should skip the `#$84` control byte and advance `d2` by `#$10` before looping back",
        );
    }

    #[test]
    fn level_entry_title_deferred_number_indices_skip_zero_count_loc_1abf4() {
        assert_eq!(
            level_entry_title_deferred_number_indices_loc_1ABF4(0).collect::<Vec<_>>(),
            Vec::<usize>::new(),
            "ASM loc_1ABF4 should branch straight to loc_1AC10 when `tst.w d0` sees a zero suffix count",
        );
    }

    #[test]
    fn level_entry_title_deferred_number_indices_emit_one_slot_per_digit_loc_1abf4() {
        assert_eq!(
            level_entry_title_deferred_number_indices_loc_1ABF4(3).collect::<Vec<_>>(),
            vec![0, 1, 2],
            "ASM loc_1ABF4 should fall through to loc_1AD48 when `d0` is non-zero, and loc_1AD48's `subq.w #1,d3` / `dbf` pair should allocate exactly `d0` deferred number workers",
        );
    }

    #[test]
    fn level_entry_title_two_player_waiter_stays_off_in_single_player_loc_1ac10() {
        assert!(
            !level_entry_title_spawns_two_player_waiter_loc_1AC10(false),
            "ASM loc_1AC10 should branch straight to the shared delete tail when `Two_player_flag` is clear",
        );
    }

    #[test]
    fn level_entry_title_two_player_waiter_turns_on_in_two_player_mode_loc_1ac10() {
        assert!(
            level_entry_title_spawns_two_player_waiter_loc_1AC10(true),
            "ASM loc_1AC10 should allocate loc_1AD80 before `Delete_CurrentObject` when `Two_player_flag` is set",
        );
    }

    #[test]
    fn level_entry_title_advance_x_defaults_to_10_pixels_for_plain_glyphs_loc_1ab6a() {
        assert_eq!(
            level_entry_title_advance_x(b'a'),
            0x10,
            "ASM loc_1AB6A falls through to `addi.w #$10,d2` for ordinary title glyphs",
        );
    }

    #[test]
    fn level_entry_title_advance_x_uses_28_pixels_for_wide_glyph_loc_1abca() {
        assert_eq!(
            level_entry_title_advance_x(0x7C),
            0x28,
            "ASM loc_1ABCA handles glyph $7C with `addi.w #$28,d2` before looping back",
        );
    }

    #[test]
    fn level_entry_title_start_pose_loc_1acb0_keeps_left_and_right_edge_x_values() {
        let mut left_rng = RomRandomState::default();
        let mut right_rng = RomRandomState::default();

        let (left_x, left_y) = level_entry_title_start_pose(&mut left_rng, 0);
        let (right_x, right_y) = level_entry_title_start_pose(&mut right_rng, 2);

        assert_eq!(left_x, 0xFFE0_0000);
        assert_eq!(right_x, 0x0160_0000);
        assert_eq!(
            left_y, right_y,
            "ASM loc_1ACB0 routes selectors 0 and 2 through the same loc_1ACEA random-Y path while keeping their fixed edge X positions",
        );
    }

    #[test]
    fn level_entry_title_start_pose_loc_1acc8_keeps_top_edge_y_fixed() {
        let mut rng = RomRandomState::default();

        let (_x, y) = level_entry_title_start_pose(&mut rng, 1);
        assert_eq!(
            y, 0xFFE0_0000,
            "ASM loc_1ACC8 should set `d6=$FFE0` for selector 1 before the shared loc_1ACCC random-X logic runs",
        );
    }

    #[test]
    fn level_entry_title_right_edge_x_loc_1ace0_returns_right_spawn_edge() {
        assert_eq!(
            level_entry_title_right_edge_x_loc_1ace0(),
            0x160,
            "ASM loc_1ACE0 should seed `d5=$160` before the shared loc_1ACEA random-Y branch resumes",
        );
    }

    #[test]
    fn level_entry_title_top_edge_y_loc_1ace6_returns_top_spawn_edge() {
        assert_eq!(
            level_entry_title_top_edge_y_loc_1ace6(),
            -0x20,
            "ASM loc_1ACE6 should seed `d5=$FFE0` before the shared loc_1ACEA random-Y branch resumes",
        );
    }

    #[test]
    fn level_entry_title_random_y_snap_loc_1acf8_clears_low_two_bits() {
        assert_eq!(
            level_entry_title_random_y_snap_loc_1acf8(0x11),
            0x10,
            "ASM loc_1ACF8 should clear the low two bits after the shared bias step so positive Y entries land on 4px boundaries",
        );
        assert_eq!(
            level_entry_title_random_y_snap_loc_1acf8(0x07),
            0x04,
            "ASM loc_1ACF8 should apply the same 4px snap after the non-positive loc_1ACEA path adds `#$28`",
        );
    }

    #[test]
    fn level_entry_title_random_x_loc_1accc_is_shared_by_top_and_bottom_entries() {
        let mut top_rng = RomRandomState::default();
        let mut bottom_rng = RomRandomState::default();

        let (top_x, top_y) = level_entry_title_start_pose(&mut top_rng, 1);
        let (bottom_x, bottom_y) = level_entry_title_start_pose(&mut bottom_rng, 3);

        assert_eq!(
            top_x, bottom_x,
            "ASM loc_1ACCC should feed the same low-byte-plus-#$3E snapped X generator to selectors 1 and 3",
        );
        assert_eq!(top_y, 0xFFE0_0000);
        assert_eq!(bottom_y, 0x0100_0000);
    }

    #[test]
    fn level_entry_title_axis_step_loc_1acfe_divides_fixed_point_delta_by_64_frames() {
        assert_eq!(
            level_entry_title_axis_step(0xFFE0_0000, 0x0040_0000),
            0x0001_8000,
            "ASM loc_1ACFE should turn a 96px delta into a 1.5px-per-frame fixed-point glide step",
        );
        assert_eq!(
            level_entry_title_axis_step(0x0160_0000, 0x0040_0000),
            -0x0004_8000,
            "ASM loc_1ACFE should preserve the sign when the worker glides back from the right edge",
        );
    }

    #[test]
    fn level_entry_title_worker_axis_pos_loc_1ad3a_pins_finished_axis_while_other_axis_keeps_gliding()
     {
        let x_step = level_entry_title_axis_step(0xFFE0_0000, 0x0040_0000);
        let y_step = level_entry_title_axis_step(0xFFE0_0000, 0x0080_0000);

        assert_eq!(
            level_entry_title_worker_axis_pos(0xFFE0_0000, 0x0040_0000, x_step, 64),
            0x0040_0000,
            "ASM loc_1AD3A should stop applying `d2` once the X compare reaches the destination",
        );
        assert_eq!(
            level_entry_title_worker_axis_pos(0xFFE0_0000, 0x0080_0000, y_step, 32),
            0x0030_0000,
            "ASM loc_1AD3A should keep the other axis moving until its own compare matches",
        );
    }

    #[test]
    fn level_entry_title_deferred_number_worker_stride_loc_1ad56_uses_glyph_7b_every_16_pixels() {
        assert_eq!(
            [
                level_entry_title_deferred_number_target_x_loc_1AD56(0x0050, 0),
                level_entry_title_deferred_number_target_x_loc_1AD56(0x0050, 1),
                level_entry_title_deferred_number_target_x_loc_1AD56(0x0050, 2),
            ],
            [0x0050, 0x0060, 0x0070],
            "ASM loc_1AD56 should keep glyph `#$7B` on the shared worker and advance `d1` by `#$10` between DBF iterations",
        );
    }

    #[test]
    fn level_entry_title_advance_x_uses_20_pixels_for_glyph_7d_loc_1abd0() {
        assert_eq!(
            level_entry_title_advance_x(0x7D),
            0x20,
            "ASM loc_1ABD0 handles glyph $7D with `addi.w #$20,d2` before looping back",
        );
    }

    #[test]
    fn level_entry_title_advance_x_uses_18_pixels_for_glyphs_7e_and_7f_loc_1abd6() {
        assert_eq!(
            level_entry_title_advance_x(0x7E),
            0x18,
            "ASM loc_1ABD6 handles glyph $7E with `addi.w #$18,d2` before looping back",
        );
        assert_eq!(
            level_entry_title_advance_x(0x7F),
            0x18,
            "ASM loc_1ABD6 reuses the same `addi.w #$18,d2` branch for glyph $7F",
        );
    }

    #[test]
    fn level_entry_title_advance_x_uses_8_pixels_for_glyphs_69_and_83_loc_1abdc() {
        assert_eq!(
            level_entry_title_advance_x(b'i'),
            0x08,
            "ASM loc_1ABDC handles lowercase `i` with `addi.w #8,d2` before looping back",
        );
        assert_eq!(
            level_entry_title_advance_x(0x83),
            0x08,
            "ASM loc_1ABDC reuses the same `addi.w #8,d2` branch for glyph $83",
        );
    }
}
