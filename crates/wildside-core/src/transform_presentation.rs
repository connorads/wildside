//! Presented Kid_Transform pass enumeration.
//!
//! ASM Kid_Transform (kid.asm): the transform coroutine hibernates gameplay and
//! walks the old helmet's `off_7EE2` animation table backward (loc_7CA6), then
//! the new helmet's table forward (loc_7D0A). Each `anim_frame` descriptor holds
//! a pass count (first byte), an `off_7D34` handler selector (second byte), and
//! an art-link word. The inner loops (`loc_7CBE`/`loc_7D22`) run
//! `jsr j_sub_8E0; dbf d0,...` with d0 seeded pass_count-1, so each descriptor
//! is displayed for exactly pass_count VBlank frames. This module reproduces
//! that displayed-frame sequence as a pure function of
//! (old helmet, new helmet, countdown) so the renderer can resolve the art and
//! palette the ROM committed on each hibernated frame without emulating the VDP.

/// ASM off_7D34 (kid.asm): per-descriptor handler dispatch. Selector 0 →
/// loc_7D80 (Kid art on Kid palette, no spark), 4 → loc_7D44 (helmet transform
/// art with the sub_8106 transform palette), 8 → loc_7D5A (helmet default art
/// with the sub_80D0 form palette), $C → loc_7D7C (Kid art plus a spark
/// overlay object, then falls into loc_7D80).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformHandler {
    HelmetArt,
    HelmetDefault,
    KidWithSpark { spark: bool },
}

/// One `anim_frame` descriptor from an `off_7EE2` transform table
/// (kid.asm, macros.asm `anim_frame mode, duration, spriteframe`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformDescriptor {
    pub pass_count: u8,
    pub handler: TransformHandler,
    pub art_link: u32,
}

/// Which Kid_Transform table walk the displayed pass belongs to:
/// loc_7CA6 (old helmet, reverse) or loc_7D0A (new helmet, forward).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformTableSource {
    OldReverse,
    NewForward,
}

/// The art/palette commit for one displayed `sub_8E0` pass of a live transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentedTransformPhase {
    pub table: TransformTableSource,
    pub descriptor_index: usize,
    pub pass_within_descriptor: u8,
    pub handler: TransformHandler,
    pub art_link: u32,
    pub palette_helmet_id: u16,
    /// ASM loc_7E90 (kid.asm): the countdown's final frame is the cleanup
    /// frame after both table walks; the new helmet's default art/palette are
    /// already committed via sub_80D0.
    pub finished: bool,
}

/// Control-flow edge after one synchronous `sub_8E0` transform pass.
///
/// ASM `loc_7CBE/loc_7D22` restores the saved registers and executes `dbf`.
/// A non-final pass branches directly back to `sub_8E0`; the last pass of a
/// descriptor runs the next `off_7D34` handler; the final descriptor reaches
/// `loc_7E14` and `loc_7E90`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformRasterResume {
    RepeatDescriptor,
    AdvanceDescriptor {
        descriptor_index: usize,
        handler: TransformHandler,
    },
    FinishLoc7e14,
}

const HELMET_THE_KID: u16 = 0;
const HELMET_CYCLONE: u16 = 2;
const HELMET_IRON_KNIGHT: u16 = 6;

const fn helmet_art(pass_count: u8, art_link: u32) -> TransformDescriptor {
    TransformDescriptor {
        pass_count,
        handler: TransformHandler::HelmetArt,
        art_link,
    }
}

const fn helmet_default(pass_count: u8) -> TransformDescriptor {
    TransformDescriptor {
        pass_count,
        handler: TransformHandler::HelmetDefault,
        art_link: 0,
    }
}

const fn kid_art(pass_count: u8, art_link: u32) -> TransformDescriptor {
    TransformDescriptor {
        pass_count,
        handler: TransformHandler::KidWithSpark { spark: false },
        art_link,
    }
}

/// ASM stru_7F30 (kid.asm): Cyclone forward transform table. Its selector-0
/// entries use Kid walk-1 art on Kid's palette; selector 4 uses the Cyclone
/// transform palette and selector 8 restores Cyclone's default art/palette.
pub const CYCLONE_TRANSFORM_TABLE_STRU_7F30: [TransformDescriptor; 10] = [
    kid_art(16, 0xA4A22),
    helmet_art(4, 0xBB48A),
    kid_art(8, 0xA4A22),
    helmet_art(4, 0xBB48A),
    helmet_art(4, 0xBB710),
    helmet_art(4, 0xBB48A),
    helmet_art(4, 0xBB710),
    helmet_default(8),
    helmet_art(4, 0xBB710),
    helmet_default(16),
];

/// ASM stru_7FF8 (kid.asm): IronKnight forward transform table. 16 descriptors
/// whose pass counts sum to 81 = `transform_anim_frames(IronKnight)`. Art links
/// are the `LnkTo_unk_*` art addresses; 0 marks the helmet-default entries.
pub const IRON_KNIGHT_TRANSFORM_TABLE_STRU_7FF8: [TransformDescriptor; 16] = [
    helmet_art(7, 0xC30E8),
    helmet_art(6, 0xC354E),
    helmet_art(5, 0xC30E8),
    helmet_art(5, 0xC354E),
    helmet_art(4, 0xC39B4),
    helmet_art(4, 0xC354E),
    helmet_art(4, 0xC39B4),
    helmet_art(4, 0xC3FBA),
    helmet_art(5, 0xC39B4),
    helmet_art(5, 0xC3FBA),
    helmet_art(5, 0xC39B4),
    helmet_art(5, 0xC3FBA),
    helmet_default(5),
    helmet_art(5, 0xC3FBA),
    helmet_default(9),
    helmet_art(3, 0xC3FBA),
];

/// Forward (`loc_7D0A`) table for a target helmet, when ported. Unported
/// helmets return `None` so the renderer keeps its existing fallback path.
fn forward_table_off_7ee2(new_helmet_id: u16) -> Option<&'static [TransformDescriptor]> {
    match new_helmet_id {
        HELMET_CYCLONE => Some(&CYCLONE_TRANSFORM_TABLE_STRU_7F30),
        HELMET_IRON_KNIGHT => Some(&IRON_KNIGHT_TRANSFORM_TABLE_STRU_7FF8),
        _ => None,
    }
}

/// ASM Kid_Transform loc_7CA6/loc_7D0A (kid.asm): enumerate the displayed
/// `sub_8E0` pass for a live transform countdown.
///
/// `frames_remaining` is the live `form_change_flash_frames` value, seeded
/// `total_passes + 1` (combat.rs `resolve_pending_form_change`; the +1 is the
/// loc_7E90 cleanup frame). Pass index = `(total_passes + 1) - frames_remaining`
/// counts through reverse(old table) ++ forward(new table); an index at or past
/// the pass total is the finished cleanup frame.
///
/// Scope: only old-form-Kid forward transforms into ported helmets return a
/// phase. ASM loc_7C76 (kid.asm): `move.w (Current_Helmet).w,d7; beq.w
/// loc_7CD0` — old form Kid skips the backward walk entirely, so the ordered
/// pass list is the forward table alone. Helmet→helmet backward walks need the
/// old table's art stamped from core state and are not ported yet.
pub fn presented_transform_phase(
    old_helmet_id: u16,
    new_helmet_id: u16,
    frames_remaining: u8,
) -> Option<PresentedTransformPhase> {
    if frames_remaining == 0 || old_helmet_id != HELMET_THE_KID {
        return None;
    }
    let table = forward_table_off_7ee2(new_helmet_id)?;
    let total_passes: u32 = table.iter().map(|d| u32::from(d.pass_count)).sum();
    let seeded = total_passes + 1;
    let remaining = u32::from(frames_remaining);
    if remaining > seeded {
        return None;
    }
    let index = seeded - remaining;
    if index >= total_passes {
        return Some(PresentedTransformPhase {
            table: TransformTableSource::NewForward,
            descriptor_index: table.len(),
            pass_within_descriptor: 0,
            handler: TransformHandler::HelmetDefault,
            art_link: 0,
            palette_helmet_id: new_helmet_id,
            finished: true,
        });
    }
    let mut pass_index = index;
    for (descriptor_index, descriptor) in table.iter().enumerate() {
        let passes = u32::from(descriptor.pass_count);
        if pass_index < passes {
            let palette_helmet_id = match descriptor.handler {
                // ASM loc_7D44 → sub_8106 off_8128[Current_Helmet] transform
                // palette; loc_7D5A → sub_80D0 off_80F2[Current_Helmet] form
                // palette. During the forward walk Current_Helmet already
                // holds the new helmet (loc_7CD0).
                TransformHandler::HelmetArt | TransformHandler::HelmetDefault => new_helmet_id,
                // ASM loc_7DF0 → `moveq #0,d7; bsr.w sub_80D0` — Kid palette.
                TransformHandler::KidWithSpark { .. } => HELMET_THE_KID,
            };
            return Some(PresentedTransformPhase {
                table: TransformTableSource::NewForward,
                descriptor_index,
                pass_within_descriptor: pass_index as u8,
                handler: descriptor.handler,
                art_link: descriptor.art_link,
                palette_helmet_id,
                finished: false,
            });
        }
        pass_index -= passes;
    }
    // Unreachable: index < total_passes always lands inside a descriptor.
    None
}

/// Pass-count sum of a ported forward table, exposed so callers/tests can pin
/// the countdown seed (`sum + 1`) against `transform_anim_frames`.
pub fn ported_forward_transform_passes(new_helmet_id: u16) -> Option<u32> {
    forward_table_off_7ee2(new_helmet_id)
        .map(|table| table.iter().map(|d| u32::from(d.pass_count)).sum())
}

/// Classify the source edge taken after the displayed transform pass.
///
/// `frames_remaining` names the pass entering `sub_8E0`, using the same
/// countdown convention as [`presented_transform_phase`]. The cleanup-only
/// value (`1`) has no `sub_8E0` call and therefore returns `None`.
pub fn transform_raster_resume_after_sub_8e0(
    old_helmet_id: u16,
    new_helmet_id: u16,
    frames_remaining: u8,
) -> Option<TransformRasterResume> {
    let current = presented_transform_phase(old_helmet_id, new_helmet_id, frames_remaining)?;
    if current.finished {
        return None;
    }
    let next = presented_transform_phase(
        old_helmet_id,
        new_helmet_id,
        frames_remaining.checked_sub(1)?,
    )?;
    if next.finished {
        return Some(TransformRasterResume::FinishLoc7e14);
    }
    if next.descriptor_index == current.descriptor_index {
        Some(TransformRasterResume::RepeatDescriptor)
    } else {
        Some(TransformRasterResume::AdvanceDescriptor {
            descriptor_index: next.descriptor_index,
            handler: next.handler,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::transform_anim_frames;
    use crate::types::PlayerForm;

    /// ASM stru_7FF8 (kid.asm): pass counts sum to 81, matching
    /// `transform_anim_frames(PlayerForm::IronKnight)`, and the countdown seed
    /// in `resolve_pending_form_change` is 81 + 1.
    #[test]
    fn iron_knight_table_matches_stru_7ff8_pass_sum() {
        let sum: u16 = IRON_KNIGHT_TRANSFORM_TABLE_STRU_7FF8
            .iter()
            .map(|d| u16::from(d.pass_count))
            .sum();
        assert_eq!(sum, 81);
        assert_eq!(sum, transform_anim_frames(PlayerForm::IronKnight));
        assert_eq!(ported_forward_transform_passes(6), Some(81));
    }

    /// ASM stru_7F30 (kid.asm): Cyclone's ten descriptors sum to 72 passes;
    /// selector 0 is Kid art, selectors 4/8 are transform/default Cyclone art.
    #[test]
    fn cyclone_table_matches_stru_7f30_descriptors() {
        let expected = [
            (16, TransformHandler::KidWithSpark { spark: false }, 0xA4A22),
            (4, TransformHandler::HelmetArt, 0xBB48A),
            (8, TransformHandler::KidWithSpark { spark: false }, 0xA4A22),
            (4, TransformHandler::HelmetArt, 0xBB48A),
            (4, TransformHandler::HelmetArt, 0xBB710),
            (4, TransformHandler::HelmetArt, 0xBB48A),
            (4, TransformHandler::HelmetArt, 0xBB710),
            (8, TransformHandler::HelmetDefault, 0),
            (4, TransformHandler::HelmetArt, 0xBB710),
            (16, TransformHandler::HelmetDefault, 0),
        ];
        for (descriptor, expected) in CYCLONE_TRANSFORM_TABLE_STRU_7F30.iter().zip(expected) {
            assert_eq!(
                (
                    descriptor.pass_count,
                    descriptor.handler,
                    descriptor.art_link
                ),
                expected
            );
        }
        assert_eq!(ported_forward_transform_passes(2), Some(72));
        assert_eq!(
            ported_forward_transform_passes(2),
            Some(u32::from(transform_anim_frames(PlayerForm::Cyclone)))
        );

        let first = presented_transform_phase(0, 2, 73).expect("Cyclone first pass");
        assert_eq!(
            first.handler,
            TransformHandler::KidWithSpark { spark: false }
        );
        assert_eq!(first.palette_helmet_id, HELMET_THE_KID);
        let default = presented_transform_phase(0, 2, 29).expect("Cyclone default pass");
        assert_eq!(default.descriptor_index, 7);
        assert_eq!(default.handler, TransformHandler::HelmetDefault);
        assert_eq!(default.palette_helmet_id, HELMET_CYCLONE);
    }

    #[test]
    fn cyclone_raster_resume_edges_cover_the_whole_loc_7d22_loop() {
        // ASM Kid_Transform stru_7F30 -> loc_7D0A/loc_7D22: 72 sub_8E0
        // calls comprise 62 DBF repeats, nine descriptor transitions, and
        // one final edge to loc_7E14. Oracle published frames 1058..1129
        // independently expose those 72 calls.
        let resumes = (2..=73)
            .rev()
            .map(|remaining| {
                transform_raster_resume_after_sub_8e0(0, HELMET_CYCLONE, remaining)
                    .expect("every displayed Cyclone pass has a resume edge")
            })
            .collect::<Vec<_>>();
        assert_eq!(resumes.len(), 72);
        assert_eq!(
            resumes
                .iter()
                .filter(|resume| matches!(resume, TransformRasterResume::RepeatDescriptor))
                .count(),
            62,
        );
        assert_eq!(
            resumes
                .iter()
                .filter(|resume| matches!(resume, TransformRasterResume::AdvanceDescriptor { .. }))
                .count(),
            9,
        );
        assert_eq!(resumes.last(), Some(&TransformRasterResume::FinishLoc7e14));
        assert_eq!(
            transform_raster_resume_after_sub_8e0(0, HELMET_CYCLONE, 1),
            None,
            "loc_7E90 cleanup is not another sub_8E0 pass",
        );
    }

    /// ASM stru_7FF8 (kid.asm) restated independently: run-length shape,
    /// handler selectors (4 = HelmetArt, 8 = HelmetDefault at descriptors 12
    /// and 14), and the C30E8/C354E/C39B4/C3FBA art-link sequence.
    #[test]
    fn iron_knight_table_matches_stru_7ff8_descriptors() {
        let expected: [(u8, TransformHandler, u32); 16] = [
            (7, TransformHandler::HelmetArt, 0xC30E8),
            (6, TransformHandler::HelmetArt, 0xC354E),
            (5, TransformHandler::HelmetArt, 0xC30E8),
            (5, TransformHandler::HelmetArt, 0xC354E),
            (4, TransformHandler::HelmetArt, 0xC39B4),
            (4, TransformHandler::HelmetArt, 0xC354E),
            (4, TransformHandler::HelmetArt, 0xC39B4),
            (4, TransformHandler::HelmetArt, 0xC3FBA),
            (5, TransformHandler::HelmetArt, 0xC39B4),
            (5, TransformHandler::HelmetArt, 0xC3FBA),
            (5, TransformHandler::HelmetArt, 0xC39B4),
            (5, TransformHandler::HelmetArt, 0xC3FBA),
            (5, TransformHandler::HelmetDefault, 0),
            (5, TransformHandler::HelmetArt, 0xC3FBA),
            (9, TransformHandler::HelmetDefault, 0),
            (3, TransformHandler::HelmetArt, 0xC3FBA),
        ];
        for (descriptor, (pass_count, handler, art_link)) in
            IRON_KNIGHT_TRANSFORM_TABLE_STRU_7FF8.iter().zip(expected)
        {
            assert_eq!(descriptor.pass_count, pass_count);
            assert_eq!(descriptor.handler, handler);
            assert_eq!(descriptor.art_link, art_link);
        }
    }

    /// ASM loc_7D0A/loc_7D22 (kid.asm): the full 82-value countdown maps to
    /// descriptor passes in table order, exactly pass_count frames each, with
    /// the final countdown value (1) being the loc_7E90 cleanup frame.
    #[test]
    fn iron_knight_kid_forward_walk_phase_table_loc_7d0a() {
        // Run-lengths restated from stru_7FF8 (kid.asm), independent of the
        // module const: (descriptor_index, pass_count).
        let run_lengths: [(usize, u8); 16] = [
            (0, 7),
            (1, 6),
            (2, 5),
            (3, 5),
            (4, 4),
            (5, 4),
            (6, 4),
            (7, 4),
            (8, 5),
            (9, 5),
            (10, 5),
            (11, 5),
            (12, 5),
            (13, 5),
            (14, 9),
            (15, 3),
        ];
        let mut expected: Vec<(usize, u8, bool)> = Vec::new();
        for (descriptor_index, pass_count) in run_lengths {
            for pass in 0..pass_count {
                expected.push((descriptor_index, pass, false));
            }
        }
        expected.push((16, 0, true));
        assert_eq!(expected.len(), 82);

        for (offset, (descriptor_index, pass, finished)) in expected.into_iter().enumerate() {
            let frames_remaining = 82 - u8::try_from(offset).unwrap();
            let phase = presented_transform_phase(0, 6, frames_remaining)
                .unwrap_or_else(|| panic!("no phase at countdown {frames_remaining}"));
            assert_eq!(
                (
                    phase.descriptor_index,
                    phase.pass_within_descriptor,
                    phase.finished
                ),
                (descriptor_index, pass, finished),
                "countdown {frames_remaining}"
            );
            assert_eq!(phase.table, TransformTableSource::NewForward);
        }
    }

    /// ASM loc_7D44/loc_7D5A (kid.asm): HelmetArt passes carry the target
    /// helmet's transform palette (sub_8106) and the descriptor's art link;
    /// HelmetDefault passes (descriptors 12/14) carry art link 0 and the form
    /// palette (sub_80D0). Current_Helmet is already the new helmet in the
    /// forward walk (loc_7CD0).
    #[test]
    fn iron_knight_handler_and_palette_decode_off_7d34() {
        // Countdown 82 → descriptor 0 pass 0: HelmetArt, C30E8.
        let first = presented_transform_phase(0, 6, 82).unwrap();
        assert_eq!(first.handler, TransformHandler::HelmetArt);
        assert_eq!(first.art_link, 0xC30E8);
        assert_eq!(first.palette_helmet_id, 6);
        assert!(!first.finished);

        // Countdown 76 → descriptor 0 pass 6 (last of 7); 75 → descriptor 1 pass 0.
        let last_of_first = presented_transform_phase(0, 6, 76).unwrap();
        assert_eq!(
            (
                last_of_first.descriptor_index,
                last_of_first.pass_within_descriptor
            ),
            (0, 6)
        );
        let second = presented_transform_phase(0, 6, 75).unwrap();
        assert_eq!(
            (second.descriptor_index, second.pass_within_descriptor),
            (1, 0)
        );
        assert_eq!(second.art_link, 0xC354E);

        // Descriptor 12 starts after 7+6+5+5+4+4+4+4+5+5+5+5 = 59 passes:
        // countdown 82-59 = 23 → HelmetDefault, art link 0, form palette.
        let default_pass = presented_transform_phase(0, 6, 23).unwrap();
        assert_eq!(default_pass.descriptor_index, 12);
        assert_eq!(default_pass.handler, TransformHandler::HelmetDefault);
        assert_eq!(default_pass.art_link, 0);
        assert_eq!(default_pass.palette_helmet_id, 6);
    }

    /// ASM loc_7E90 (kid.asm): countdown 1 is the cleanup frame after the
    /// table walk; the new helmet's default art/palette are committed.
    #[test]
    fn iron_knight_finished_frame_is_helmet_default_loc_7e90() {
        let cleanup = presented_transform_phase(0, 6, 1).unwrap();
        assert!(cleanup.finished);
        assert_eq!(cleanup.handler, TransformHandler::HelmetDefault);
        assert_eq!(cleanup.art_link, 0);
        assert_eq!(cleanup.palette_helmet_id, 6);
        // Countdown 2 is the last real pass: descriptor 15, pass 2.
        let last_pass = presented_transform_phase(0, 6, 2).unwrap();
        assert!(!last_pass.finished);
        assert_eq!(
            (last_pass.descriptor_index, last_pass.pass_within_descriptor),
            (15, 2)
        );
    }

    /// ASM loc_7C76 (kid.asm): `move.w (Current_Helmet).w,d7; beq.w loc_7CD0`
    /// — only old-form-Kid transforms skip the backward walk; helmet→helmet
    /// transforms are not ported and must fall back.
    #[test]
    fn non_kid_old_form_returns_none_loc_7c76() {
        assert!(presented_transform_phase(2, 6, 82).is_none());
        assert!(presented_transform_phase(6, 6, 82).is_none());
        assert!(presented_transform_phase(0, 6, 82).is_some());
    }

    /// Unported target helmets and out-of-range countdowns fall back.
    #[test]
    fn unported_or_out_of_range_returns_none() {
        assert!(presented_transform_phase(0, 3, 40).is_none());
        assert!(presented_transform_phase(0, 6, 0).is_none());
        assert!(presented_transform_phase(0, 6, 83).is_none());
    }
}
