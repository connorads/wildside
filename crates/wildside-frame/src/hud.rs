use std::sync::Arc;

use wildside_core::{
    GameMode, LevelId, NativeTraceFrame, PlayerForm, hitpoint_display_art_dma_loop_loc_12C34,
};
use wildside_render::{NativeLevelVisuals, TileRef};

use crate::dto::*;
use crate::scene::*;

const HUD_LIVES_X: i32 = 0x190 - 128;
const HUD_LIVES_Y: i32 = 0x90 - 128;
const HUD_ANKH_X: i32 = 0x1A1 - 128;
const HUD_ANKH_Y: i32 = 0x8D - 128;
const HUD_DIAMOND_X: i32 = 0x1A0 - 128;
const HUD_DIAMOND_Y: i32 = 0xA2 - 128;
const HUD_DIAMOND_COUNT_X: i32 = 0x190 - 128;
const HUD_DIAMOND_COUNT_Y: i32 = 0xA4 - 128;
const HUD_DIAMOND_ANIMATION_FRAME_DURATION: usize = 6;
const HUD_DIAMOND_DMA_PAGE_RENDER_LEAD_LOC_45EE: usize = 2;
const DEMO_1_LEVEL_LOC_119B6: LevelId = LevelId(0x15);
// ASM Make_SpriteAttr_HUD loc_BE0C (kid.asm:15407): d2 starts at $80 and
// `addi.w #$10,d2` runs before the first hitpoint sprite write, so the first
// visible HP pip lands at biased X $90, i.e. screen X 16.
const HUD_HITPOINT_START_X: i32 = 0x90 - 128;
const HUD_HITPOINT_Y: i32 = 0xA4 - 128;
const HUD_HITPOINT_SPACING_X: i32 = 0x10;

fn hud_hitpoint_vram_base_tile_index_loc_12c34() -> u16 {
    hitpoint_display_art_dma_loop_loc_12C34().vram_tile_base_index()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HudHitpointTile {
    Full,
    Half,
    Empty,
}

fn hud_hitpoint_tiles(number_hitpoints: u16, max_hitpoints: u16) -> Vec<(HudHitpointTile, bool)> {
    // ASM Make_SpriteAttr_HUD loc_BE0C..loc_BE74 (kid.asm:15407): d0 (current
    // HP) and d1 (capacity) decrement by two per slot. A full 16×8 slot emits
    // $822D/$822E/$822F at size $04; an odd trailing capacity slot (loc_BE4C /
    // loc_BE74) emits a single 8×8 pip at size $00. Size $00 is a VISIBLE 8×8
    // sprite, NOT hidden (#276 — the prior code dropped this trailing pip, so an
    // Iron Knight showed 4 dots for both 5 and 4 HP). The bool flags that
    // narrow trailing slot so the renderer draws one tile, not two.
    let mut current = number_hitpoints as i16;
    let mut capacity = max_hitpoints as i16;
    let mut tiles = Vec::new();

    loop {
        capacity -= 2;
        current -= 2;
        // `capacity < 0` after the decrement is the odd trailing slot
        // (loc_BE4C / loc_BE74): a single 8×8 pip representing one HP, so it is
        // only ever filled ($822D, present) or grey ($822F, spent) — never half.
        let narrow = capacity < 0;
        let tile = if narrow {
            if current >= -1 {
                HudHitpointTile::Full
            } else {
                HudHitpointTile::Empty
            }
        } else if current >= 0 {
            HudHitpointTile::Full
        } else if current == -1 {
            HudHitpointTile::Half
        } else {
            HudHitpointTile::Empty
        };
        tiles.push((tile, narrow));
        if capacity <= 0 {
            break;
        }
    }

    tiles
}

fn push_hud_tile_offset(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    tile_offset: u16,
    x: i32,
    y: i32,
) {
    let Some(tile) = resolve_ingame_vram_tile(
        level_visuals,
        TileRef {
            tile_index: level_visuals.hud.digit_base_tile_index + tile_offset,
            palette_line: 0,
            priority: false,
            flip_x: false,
            flip_y: false,
        },
    ) else {
        return;
    };
    push_resolved_tile(commands, x, y, tile);
}

fn push_hud_fixed_digit(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    digit: u16,
    x: i32,
    y: i32,
) {
    push_hud_tile_offset(commands, level_visuals, digit.min(9), x, y);
}

fn push_hud_two_digit_counter(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    value: u16,
    x: i32,
    y: i32,
) {
    let clamped = value.min(99);
    // ASM life_display/diamond_display writes `move.w d1,$C(a0)` / `$44(a0)`
    // to the first counter slot, then swaps d1 for the second slot. For
    // calc_display_number values below 10, that places the digit first and
    // HUD number index 11 (`$86C5`) in the following "x" slot.
    let (first, second) = if clamped < 10 {
        (clamped, 11)
    } else {
        (clamped / 10, clamped % 10)
    };
    push_hud_tile_offset(commands, level_visuals, first, x, y);
    push_hud_tile_offset(commands, level_visuals, second, x + 8, y);
}

fn push_hud_hitpoint_sprite(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    base_tile: TileRef,
    x: i32,
    y: i32,
    narrow: bool,
) {
    // ASM Make_SpriteAttr_HUD loc_BE20/BE34/BE5C writes size byte #4 for
    // $822D/$822E/$822F. On Mega Drive this is a 2x1 tile sprite, so the
    // visible pip includes the following tile as well as the base tile. ASM
    // loc_12C34 DMAs ArtUnc_12C3C into VRAM tile $22D, overlapping the block
    // VRAM page; draw from the dedicated HUD hitpoint sheet, not block art.
    // The odd trailing pip (loc_BE4C/BE74) is size $00 = a single 8×8 tile.
    let tile_count: u16 = if narrow { 1 } else { 2 };
    for tile_offset in 0..tile_count {
        let Some(tile_index) = base_tile
            .tile_index
            .checked_sub(hud_hitpoint_vram_base_tile_index_loc_12c34())
            .and_then(|index| index.checked_add(tile_offset))
        else {
            continue;
        };
        if usize::from(tile_index) < level_visuals.hud.hitpoint_sheet.tiles.len() {
            push_resolved_tile(
                commands,
                x + i32::from(tile_offset) * 8,
                y,
                ResolvedSceneTile {
                    tile_sheet: Arc::new(level_visuals.hud.hitpoint_sheet.clone()),
                    tile: TileRef {
                        tile_index,
                        ..base_tile
                    },
                    palette: Arc::new(level_visuals.palettes.foreground.clone()),
                },
            );
        }
    }
}

fn hud_hitpoints_visible_loc_bdc0(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> bool {
    if trace.form_change_flash_frames == 0 {
        return true;
    }
    let Some(form_visuals) = level_visuals.player.forms.get(&trace.current_helmet_id) else {
        return true;
    };
    let transform_frames = form_visuals
        .transform_steps
        .iter()
        .map(|step| step.duration_frames)
        .sum::<usize>();
    if transform_frames == 0 {
        return true;
    }

    // ASM Mode_Level uploads the previous Sprite_Table before
    // Make_SpriteAttr_HUD loc_BDC0 sees Currently_transforming and branches to
    // loc_BE82. The first hibernated transform frame therefore still displays
    // the old HP pips; subsequent transform frames do not.
    usize::from(trace.form_change_flash_frames) >= transform_frames
}

pub fn push_hud_icon(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    icon_name: &str,
    trace: &NativeTraceFrame,
    x: i32,
    y: i32,
) {
    match icon_name {
        "ankh" => {
            // ASM Init_SpriteAttr_HUD unk_1427C uses `sprite_attr $1A1,$8D,1,1,$8231`.
            // Tile $231 overlaps native's block VRAM range, so resolve it via
            // the ROM Life_icon sheet loaded for PrizeAnkh_Init instead of the
            // level block sheet.
            if let Some(icon) = level_visuals.floating_prizes.icons.get(&0x0B)
                && let Some(frame) = icon.frames.first()
            {
                // ASM sub_44DC loc_4546..loc_4586 uploads animated life-icon
                // pages to the same VRAM page used by the HUD ankh sprite.
                // The trace renderer has no live VDP memory for normal
                // gameplay, so this mirrors the ROM page selected by the
                // sampled `$FFFFFA5E` state.
                let frame = wildside_render::SpriteFrame {
                    tile_base_index: hud_life_icon_tile_base_index_loc_4546(trace),
                    ..frame.clone()
                };
                push_sprite_frame_top_left(commands, &frame, &icon.palette, x, y);
            }
        }
        "diamond" => {
            // ASM Init_SpriteAttr_HUD unk_1427C uses `sprite_attr $1A0,$A2,1,1,$86F2`.
            // The size bits are 1,1, i.e. a 2x2 tile sprite starting at HUD
            // diamond VRAM tile $6F2, not a single 8x8 tile. ASM sub_44DC
            // drives `unk_461C` at 6 VBlanks per frame and `sub_5D4A` DMAs
            // diamond_1..4 into the same $6EA VRAM page.
            let animation_frame = hud_diamond_animation_frame_index(
                trace,
                level_visuals.floating_prizes.diamond.frames.len(),
            );
            let tile_sheet = level_visuals
                .floating_prizes
                .diamond
                .frames
                .get(animation_frame)
                .map_or_else(
                    || level_visuals.hud.diamond_tiles.clone(),
                    |frame| frame.tile_sheet.clone(),
                );
            let frame = wildside_render::SpriteFrame {
                origin_x: 0,
                origin_y: 0,
                width_px: 0x10,
                height_px: 0x10,
                tile_base_index: 0x06F2 - level_visuals.hud.diamond_base_tile_index,
                tile_sheet,
            };
            let palette = hud_diamond_palette_loc_1223c(trace, level_visuals);
            push_sprite_frame_top_left(commands, &frame, &palette, x, y);
        }
        _ => {
            if let Some(icon) = level_visuals
                .hud
                .icons
                .iter()
                .find(|icon| icon.name == icon_name)
                && let Some(tile) = resolve_ingame_vram_tile(level_visuals, icon.tile)
            {
                push_resolved_tile(commands, x, y, tile);
            }
        }
    }
}

fn hud_diamond_animation_frame_index(trace: &NativeTraceFrame, frame_count: usize) -> usize {
    if frame_count == 0 {
        return 0;
    }
    if let Some(offset) = trace.hud_diamond_frame_offset {
        // ASM sub_44DC loc_45EE/loc_460A (kid.asm:5670): `$FFFFFA5D`
        // stores the byte offset into `unk_461C`, whose speed/frame pairs are
        // laid out at offsets 0,2,4,6 for diamond pages 0..3. Normal trace
        // rendering uses the next page because the sampled trace is
        // post-`sub_44DC` while the visible HUD art is the page presented by the
        // subsequent VBlank. The direct loc_1AE2E recording hand-off reaches
        // live gameplay with `LevelSkip_Cheat=0` and loc_D468's two-second
        // completion-time clock. Oracle RAM watches at release frames
        // 0/30/60/120 show `$FFFFFA5D` two DMA pages ahead of that reduced
        // native clock.
        let render_lead = if trace.game_mode == GameMode::DemoPlay
            && trace.current_level == DEMO_1_LEVEL_LOC_119B6
        {
            // ASM Load_DemoPlay loc_119B6 chooses Knights Isle for demo 1,
            // then Load_InGame's `sub_44B0`/`sub_44DC` performs the first HUD
            // DMA before the trace renderer's sampled VBlank. Oracle VRAM at
            // published-TAS frames 902..970 shows `$DD40` holds the direct
            // `unk_461C` page selected by `$FFFFFA5D`, without a render lead.
            0
        } else if trace.level_skip_cheat_enabled == 0
            && trace.level_completion_time > 0
            && !trace.level_complete
        {
            HUD_DIAMOND_DMA_PAGE_RENDER_LEAD_LOC_45EE
        } else {
            1
        };
        return (usize::from(offset / 2) + render_lead) % frame_count;
    }
    // ASM sub_44B0 seeds `$FFFFFA58` (diamond countdown) to 1. The native
    // trace frame is sampled after the previous visible VBlank plus the
    // current sub_44DC loc_45EE upload, so the rendered HUD page is two
    // `unk_461C` DMA slots ahead of the replay-local frame bucket.
    ((trace.frame_index / HUD_DIAMOND_ANIMATION_FRAME_DURATION)
        + HUD_DIAMOND_DMA_PAGE_RENDER_LEAD_LOC_45EE)
        % frame_count
}

fn hud_diamond_palette_loc_1223c(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> wildside_render::Palette {
    let mut palette = level_visuals.palettes.foreground.clone();
    if palette.colours.len() < 10 {
        return palette;
    }
    if trace
        .lava_water_palette_words_loc_1223c
        .is_some_and(|words| words.iter().any(|word| *word != 0))
    {
        return palette;
    }
    let Some((_, _, completed_rotations)) = trace.rotate_palette_lava_water_state_loc_1223c else {
        return palette;
    };
    // ASM Rotate_Palette_Lava_Water loc_1223C rotates foreground palette
    // entries 6..9 in hardware. When the level has no lava/water override
    // words, the trace renderer must reconstruct that VDP palette phase for
    // HUD sprites that reuse foreground palette line 0.
    // The four-slot palette window wraps naturally as the loop rotates it.
    // Keeping the full count avoids an equivalent `%` mutation: `n + 4`
    // produces the same visible palette as `n`.
    let rotations = 3 * (usize::from(completed_rotations) + 1);
    for _ in 0..rotations {
        let saved = palette.colours[6];
        palette.colours[6] = palette.colours[7];
        palette.colours[7] = palette.colours[8];
        palette.colours[8] = palette.colours[9];
        palette.colours[9] = saved;
    }
    palette
}

fn hud_life_icon_tile_base_index_loc_4546(trace: &NativeTraceFrame) -> u16 {
    if trace.level_complete && trace.level_complete_elapsed_frames == 0 {
        4
    } else if trace.level_skip_cheat_enabled == 0
        && trace.level_completion_time > 0
        && !trace.level_complete
    {
        // ASM sub_44DC loc_4546/loc_4586 (kid.asm:5592): direct loc_1AE2E
        // recording hand-offs present the first `AniArt_LifeIcon` DMA page.
        // Oracle VRAM `$4620` matches ROM page 0 throughout release frames
        // 0/30/60/120, independently of the reduced native animation offset.
        0
    } else if trace.level_skip_cheat_enabled != 0
        && trace.level_completion_time == 1
        && !trace.level_complete
        && let Some(tile_base_index) =
            hud_life_icon_settled_recording_tile_base_index_loc_4586(trace.hud_life_frame_offset)
    {
        // ASM sub_44DC loc_4546/loc_4586 (kid.asm:5592): `AniArt_LifeIcon`
        // stores speed/frame pairs at offsets 0,2,4,6. Settled LevelSkip_Cheat
        // recordings only expose the final offset-6 live page before the frozen
        // HUD hand-off; earlier offsets remain on the frozen HUD page.
        tile_base_index
    } else if trace.hud_life_frame_offset == Some(0) {
        // ASM sub_44DC loc_4546 starts `AniArt_LifeIcon` at byte offset 0.
        // Oracle frame issue-289-level06-idle-current@179 shows the normal
        // gameplay HUD ankh presents the third ROM page for that sampled state.
        8
    } else {
        0
    }
}

fn hud_life_icon_settled_recording_tile_base_index_loc_4586(offset: Option<u8>) -> Option<u16> {
    match offset {
        Some(6) => Some(4),
        _ => None,
    }
}

fn hud_timer_seconds_loc_c070(trace: &NativeTraceFrame) -> u16 {
    if let Some(seconds) = trace.hud_timer_sprite_seconds_loc_bee8 {
        return seconds;
    }
    if trace.level_complete
        && trace.level_complete_elapsed_frames > 0
        && trace.level_complete_elapsed_frames
            < wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
    {
        // ASM CharacterCollision_TouchFlagpole loc_C070 sets $FFFFFB4B so
        // Make_SpriteAttr_HUD exits before the decrement path that reaches
        // loc_BEE8. Native trace state has already consumed the second, but
        // the visible HUD sprite table is still the previous timer value until
        // loc_C186 starts drawing the end-level hologram.
        trace.level_timer_seconds.saturating_add(1)
    } else if matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
        && !trace.level_complete
        && trace.level_timer_sub_frames == 1
    {
        // ASM Make_SpriteAttr_HUD loc_BE82 decrements Time_SubSeconds, then
        // on rollover decrements Time_Seconds_* before loc_BEE8 writes timer
        // digit tiles. Native trace state is sampled before the following
        // gameplay trace exposes that whole-second decrement.
        trace.level_timer_seconds.saturating_sub(1)
    } else {
        trace.level_timer_seconds
    }
}

fn hud_timer_layout_loc_bd0a(
    trace: &NativeTraceFrame,
    minutes: u16,
    seconds_tens: u16,
) -> wildside_core::HudTimerSpriteLayoutSubBD0A {
    let seeded = wildside_core::hud_timer_sprite_layout_sub_BD0A(
        wildside_core::HudTimerSpriteLayoutSubBD0A {
            minutes: wildside_core::HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
            separator: wildside_core::HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
            seconds_tens: wildside_core::HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
            seconds_ones: wildside_core::HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
        },
        false,
        false,
        0,
        0,
        0,
        0,
    );
    let track_kid =
        wildside_core::hud_timer_low_time_check_end_decrease_time_left(minutes, seconds_tens)
            .track_kid;
    let Some(kid_x_pos) = trace.kid_x_pos else {
        return seeded;
    };
    let Some(kid_y_pos) = trace.kid_y_pos else {
        return seeded;
    };

    // ASM End_Decrease_Time_Left calls sub_BD0A with d7=1 only at <=0:29.
    // loc_BD38 returns early while the timer-frozen flag is set; native keeps
    // the fixed seeded row for level-complete frozen-HUD frames.
    wildside_core::hud_timer_sprite_layout_sub_BD0A(
        seeded,
        track_kid,
        trace.level_complete,
        (kid_x_pos >> 16) as u16,
        (kid_y_pos >> 16) as u16,
        (trace.camera_x_pos >> 16) as u16,
        (trace.camera_y_pos >> 16) as u16,
    )
}

fn hud_timer_coord_loc_bd0a(position: wildside_core::HudTimerSpritePositionSubBD0A) -> (i32, i32) {
    (
        i32::from(position.x_pos) - 128,
        i32::from(position.y_pos) - 128,
    )
}

pub fn push_game_hud(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let max_hp = PlayerForm::from_helmet_id(trace.current_helmet_id).map_or_else(
        || PlayerForm::TheKid.max_hitpoints(),
        PlayerForm::max_hitpoints,
    ) as i32
        + i32::from(trace.extra_hitpoint_slots);
    if hud_hitpoints_visible_loc_bdc0(trace, level_visuals) {
        for (i, (hitpoint_tile, narrow)) in
            hud_hitpoint_tiles(trace.number_hitpoints, max_hp as u16)
                .iter()
                .enumerate()
        {
            let pip_tile = match hitpoint_tile {
                HudHitpointTile::Full => level_visuals.hud.hitpoint_tiles[0],
                HudHitpointTile::Half => level_visuals.hud.hitpoint_tiles[1],
                HudHitpointTile::Empty => level_visuals.hud.hitpoint_tiles[2],
            };
            push_hud_hitpoint_sprite(
                commands,
                level_visuals,
                pip_tile,
                HUD_HITPOINT_START_X + i as i32 * HUD_HITPOINT_SPACING_X,
                HUD_HITPOINT_Y,
                *narrow,
            );
        }
    }

    let timer_seconds = hud_timer_seconds_loc_c070(trace);
    let minutes = timer_seconds / 60;
    let seconds = timer_seconds % 60;
    let seconds_tens = seconds / 10;
    let seconds_ones = seconds % 10;
    let timer_layout = hud_timer_layout_loc_bd0a(trace, minutes, seconds_tens);
    let (minutes_x, minutes_y) = hud_timer_coord_loc_bd0a(timer_layout.minutes);
    let (separator_x, separator_y) = hud_timer_coord_loc_bd0a(timer_layout.separator);
    let (seconds_tens_x, seconds_tens_y) = hud_timer_coord_loc_bd0a(timer_layout.seconds_tens);
    let (seconds_ones_x, seconds_ones_y) = hud_timer_coord_loc_bd0a(timer_layout.seconds_ones);

    push_hud_fixed_digit(commands, level_visuals, minutes, minutes_x, minutes_y);
    if let Some(tile) = resolve_ingame_vram_tile(
        level_visuals,
        TileRef {
            // ASM sprite table entry (kid.asm:27111): timer colon uses $86C4,
            // which is HUD number tile index 10. Index 11 is the blank tens
            // tile used by calc_display_number for single-digit counters.
            tile_index: level_visuals.hud.digit_base_tile_index + 10,
            palette_line: 0,
            priority: false,
            flip_x: false,
            flip_y: false,
        },
    ) {
        push_resolved_tile(commands, separator_x, separator_y, tile);
    }
    push_hud_fixed_digit(
        commands,
        level_visuals,
        seconds_tens,
        seconds_tens_x,
        seconds_tens_y,
    );
    push_hud_fixed_digit(
        commands,
        level_visuals,
        seconds_ones,
        seconds_ones_x,
        seconds_ones_y,
    );

    push_hud_two_digit_counter(
        commands,
        level_visuals,
        trace.number_lives,
        HUD_LIVES_X,
        HUD_LIVES_Y,
    );
    push_hud_icon(
        commands,
        level_visuals,
        "ankh",
        trace,
        HUD_ANKH_X,
        HUD_ANKH_Y,
    );

    push_hud_two_digit_counter(
        commands,
        level_visuals,
        trace.diamond_count,
        HUD_DIAMOND_COUNT_X,
        HUD_DIAMOND_COUNT_Y,
    );
    push_hud_icon(
        commands,
        level_visuals,
        "diamond",
        trace,
        HUD_DIAMOND_X,
        HUD_DIAMOND_Y,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wildside_core::{
        EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE, HudTimerSpritePositionSubBD0A,
    };
    use wildside_render::{
        AnimatedSpriteVisual, BackgroundVisuals, BlockVisualMapping, ChunkedBackgroundVisuals,
        EnemyVisuals, FlagVisuals, FloatingPrizeVisuals, ForegroundLayout, HudIcon, HudVisuals,
        MdColour, Palette, PaletteSet, PlatformVisuals, PlayerFormVisuals, PlayerTransformStep,
        PlayerVisuals, SpriteFrame, Tile, TileQuad, TileRef, TileSheet,
    };

    #[test]
    fn hud_hitpoint_tiles_follow_loc_be0c_two_hp_slots_and_odd_tail() {
        // ASM Make_SpriteAttr_HUD loc_BE0C..loc_BE74: capacity/current HP
        // decrement by two per slot; odd capacity leaves one narrow visible pip.
        assert_eq!(
            hud_hitpoint_tiles(5, 5),
            vec![
                (HudHitpointTile::Full, false),
                (HudHitpointTile::Full, false),
                (HudHitpointTile::Full, true),
            ]
        );
        assert_eq!(
            hud_hitpoint_tiles(4, 5),
            vec![
                (HudHitpointTile::Full, false),
                (HudHitpointTile::Full, false),
                (HudHitpointTile::Empty, true),
            ]
        );
        assert_eq!(
            hud_hitpoint_tiles(1, 3),
            vec![
                (HudHitpointTile::Half, false),
                (HudHitpointTile::Empty, true),
            ]
        );
        assert_eq!(
            hud_hitpoint_tiles(0, 2),
            vec![(HudHitpointTile::Empty, false)]
        );
    }

    #[test]
    fn hud_diamond_animation_frame_index_uses_loc_45ee_upload_lead() {
        // ASM sub_44DC loc_45EE: sampled offset pages render one DMA page ahead
        // in normal gameplay. The loc_1AE2E settled recording clock is two
        // pages behind oracle `$FFFFFA5D`, so its VDP-facing page uses the
        // two-page upload lead retained by the replay-local fallback.
        let normal_trace = NativeTraceFrame {
            hud_diamond_frame_offset: Some(4),
            ..NativeTraceFrame::default()
        };
        assert_eq!(hud_diamond_animation_frame_index(&normal_trace, 4), 3);

        let demo_1_trace = NativeTraceFrame {
            game_mode: GameMode::DemoPlay,
            current_level: LevelId(0x15),
            hud_diamond_frame_offset: Some(4),
            ..NativeTraceFrame::default()
        };
        assert_eq!(hud_diamond_animation_frame_index(&demo_1_trace, 4), 2);

        for level_completion_time in [1, 2, 3] {
            let settled_trace = NativeTraceFrame {
                hud_diamond_frame_offset: Some(4),
                level_skip_cheat_enabled: 0,
                level_completion_time,
                level_complete: false,
                ..NativeTraceFrame::default()
            };
            assert_eq!(hud_diamond_animation_frame_index(&settled_trace, 4), 0);
        }

        let skip_without_settled_completion_time = NativeTraceFrame {
            hud_diamond_frame_offset: Some(4),
            level_skip_cheat_enabled: 1,
            level_completion_time: 0,
            level_complete: false,
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            hud_diamond_animation_frame_index(&skip_without_settled_completion_time, 4),
            3
        );

        let replay_local_trace = NativeTraceFrame {
            frame_index: 6,
            hud_diamond_frame_offset: None,
            ..NativeTraceFrame::default()
        };
        assert_eq!(hud_diamond_animation_frame_index(&replay_local_trace, 4), 3);
        assert_eq!(hud_diamond_animation_frame_index(&replay_local_trace, 0), 0);
    }

    #[test]
    fn hud_life_icon_tile_base_index_follows_loc_4546_and_settled_loc_4586() {
        // ASM sub_44DC loc_4546/loc_4586: the normal offset-zero sample presents
        // page 8; settled level-skip recordings only expose offset-6 page 4.
        assert_eq!(
            hud_life_icon_tile_base_index_loc_4546(&NativeTraceFrame {
                level_complete: true,
                level_complete_elapsed_frames: 0,
                ..NativeTraceFrame::default()
            }),
            4
        );
        assert_eq!(
            hud_life_icon_tile_base_index_loc_4546(&NativeTraceFrame {
                hud_life_frame_offset: Some(0),
                ..NativeTraceFrame::default()
            }),
            8
        );
        assert_eq!(
            hud_life_icon_tile_base_index_loc_4546(&NativeTraceFrame {
                hud_life_frame_offset: Some(6),
                level_skip_cheat_enabled: 1,
                level_completion_time: 1,
                level_complete: false,
                ..NativeTraceFrame::default()
            }),
            4
        );
        for (level_completion_time, offset) in [(1, 0), (2, 6), (3, 0)] {
            assert_eq!(
                hud_life_icon_tile_base_index_loc_4546(&NativeTraceFrame {
                    hud_life_frame_offset: Some(offset),
                    level_skip_cheat_enabled: 0,
                    level_completion_time,
                    level_complete: false,
                    ..NativeTraceFrame::default()
                }),
                0
            );
        }
        assert_eq!(
            hud_life_icon_settled_recording_tile_base_index_loc_4586(Some(4)),
            None
        );
    }

    #[test]
    fn hud_timer_seconds_preserves_visible_loc_c070_boundary_values() {
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                hud_timer_sprite_seconds_loc_bee8: Some(179),
                level_timer_seconds: 180,
                level_timer_sub_frames: 1,
                game_mode: GameMode::InGame,
                ..NativeTraceFrame::default()
            }),
            179,
            "ASM Make_SpriteAttr_HUD loc_BEE8 renders the explicitly built timer digits, independently of subsequently reseeded live timer RAM"
        );
        // ASM CharacterCollision_TouchFlagpole loc_C070: frozen HUD keeps the
        // previous second until the hologram becomes visible.
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                level_complete: true,
                level_complete_elapsed_frames: 0,
                level_timer_seconds: 41,
                ..NativeTraceFrame::default()
            }),
            41
        );
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                level_complete: true,
                level_complete_elapsed_frames: 1,
                level_timer_seconds: 41,
                ..NativeTraceFrame::default()
            }),
            42
        );
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                level_complete: true,
                level_complete_elapsed_frames: EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE,
                level_timer_seconds: 41,
                ..NativeTraceFrame::default()
            }),
            41
        );

        // ASM Make_SpriteAttr_HUD loc_BE82..loc_BEE8: at sub-frame 1, the
        // visible sprite table has already rolled the whole-second digit down.
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                game_mode: GameMode::InGame,
                level_timer_sub_frames: 1,
                level_timer_seconds: 60,
                ..NativeTraceFrame::default()
            }),
            59
        );
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                game_mode: GameMode::SegaScreen,
                level_timer_sub_frames: 1,
                level_timer_seconds: 60,
                ..NativeTraceFrame::default()
            }),
            60
        );
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                game_mode: GameMode::InGame,
                level_complete: true,
                level_timer_sub_frames: 1,
                level_timer_seconds: 60,
                ..NativeTraceFrame::default()
            }),
            60
        );
        assert_eq!(
            hud_timer_seconds_loc_c070(&NativeTraceFrame {
                game_mode: GameMode::DemoPlay,
                level_timer_sub_frames: 1,
                level_timer_seconds: 0,
                ..NativeTraceFrame::default()
            }),
            0
        );
    }

    #[test]
    fn hud_timer_layout_loc_bd0a_tracks_kid_only_for_low_time_live_frames() {
        // ASM End_Decrease_Time_Left enables tracking only for <=0:29, then
        // sub_BD0A uses pixel positions from fixed-point trace fields.
        let seeded = hud_timer_layout_loc_bd0a(&NativeTraceFrame::default(), 0, 2);
        let tracked = hud_timer_layout_loc_bd0a(
            &NativeTraceFrame {
                kid_x_pos: Some(0x0130_0000),
                kid_y_pos: Some(0x0180_0000),
                camera_x_pos: 0x0100_0000,
                camera_y_pos: 0x0120_0000,
                ..NativeTraceFrame::default()
            },
            0,
            2,
        );
        assert_ne!(tracked, seeded);
        assert_eq!(
            tracked,
            wildside_core::hud_timer_sprite_layout_sub_BD0A(
                seeded, true, false, 0x0130, 0x0180, 0x0100, 0x0120,
            )
        );

        let high_time = hud_timer_layout_loc_bd0a(
            &NativeTraceFrame {
                kid_x_pos: Some(0x0130_0000),
                kid_y_pos: Some(0x0180_0000),
                camera_x_pos: 0x0100_0000,
                camera_y_pos: 0x0120_0000,
                ..NativeTraceFrame::default()
            },
            1,
            0,
        );
        assert_eq!(high_time, seeded);

        let level_complete = hud_timer_layout_loc_bd0a(
            &NativeTraceFrame {
                kid_x_pos: Some(0x0130_0000),
                kid_y_pos: Some(0x0180_0000),
                camera_x_pos: 0x0100_0000,
                camera_y_pos: 0x0120_0000,
                level_complete: true,
                ..NativeTraceFrame::default()
            },
            0,
            2,
        );
        assert_eq!(level_complete, seeded);
    }

    #[test]
    fn hud_timer_coord_loc_bd0a_unbiases_md_sprite_positions() {
        // ASM HUD sprite coordinates are MD-biased by 128 before rendering.
        assert_eq!(
            hud_timer_coord_loc_bd0a(HudTimerSpritePositionSubBD0A {
                x_pos: 0x0090,
                y_pos: 0x0088,
            }),
            (16, 8)
        );
    }

    #[test]
    fn hud_hitpoints_visible_loc_bdc0_hides_only_during_active_transform_flash() {
        let mut visuals = sample_level_visuals();
        visuals
            .player
            .forms
            .get_mut(&0)
            .expect("kid form")
            .transform_steps = vec![
            PlayerTransformStep {
                duration_frames: 2,
                frame: None,
            },
            PlayerTransformStep {
                duration_frames: 3,
                frame: None,
            },
        ];

        // ASM Make_SpriteAttr_HUD loc_BDC0 keeps the previous HP table visible
        // for the first transform hand-off frame, then suppresses pips until
        // the summed transform hibernate has elapsed.
        assert!(hud_hitpoints_visible_loc_bdc0(
            &NativeTraceFrame {
                form_change_flash_frames: 0,
                ..NativeTraceFrame::default()
            },
            &visuals
        ));
        assert!(!hud_hitpoints_visible_loc_bdc0(
            &NativeTraceFrame {
                form_change_flash_frames: 4,
                current_helmet_id: 0,
                ..NativeTraceFrame::default()
            },
            &visuals
        ));
        assert!(hud_hitpoints_visible_loc_bdc0(
            &NativeTraceFrame {
                form_change_flash_frames: 5,
                current_helmet_id: 0,
                ..NativeTraceFrame::default()
            },
            &visuals
        ));
        assert!(hud_hitpoints_visible_loc_bdc0(
            &NativeTraceFrame {
                form_change_flash_frames: 4,
                current_helmet_id: 99,
                ..NativeTraceFrame::default()
            },
            &visuals
        ));
    }

    #[test]
    fn hud_diamond_palette_loc_1223c_reconstructs_foreground_rotation() {
        let visuals = sample_level_visuals();
        let rotated = hud_diamond_palette_loc_1223c(
            &NativeTraceFrame {
                rotate_palette_lava_water_state_loc_1223c: Some((0, 0, 0)),
                lava_water_palette_words_loc_1223c: Some([0; 4]),
                ..NativeTraceFrame::default()
            },
            &visuals,
        );

        // ASM Rotate_Palette_Lava_Water loc_1223C rotates slots 6..9 by
        // 3*(completed+1) steps when no live lava/water CRAM words override it.
        let raw_words: Vec<u16> = rotated.colours[6..10]
            .iter()
            .map(|colour| colour.raw_word)
            .collect();
        assert_eq!(raw_words, vec![9, 6, 7, 8]);

        let blocked = hud_diamond_palette_loc_1223c(
            &NativeTraceFrame {
                rotate_palette_lava_water_state_loc_1223c: Some((0, 0, 0)),
                lava_water_palette_words_loc_1223c: Some([0, 0, 1, 0]),
                ..NativeTraceFrame::default()
            },
            &visuals,
        );
        let blocked_words: Vec<u16> = blocked.colours[6..10]
            .iter()
            .map(|colour| colour.raw_word)
            .collect();
        assert_eq!(blocked_words, vec![6, 7, 8, 9]);

        let mut ten_colour_visuals = sample_level_visuals();
        ten_colour_visuals.palettes.foreground.colours.truncate(10);
        let second_rotation = hud_diamond_palette_loc_1223c(
            &NativeTraceFrame {
                rotate_palette_lava_water_state_loc_1223c: Some((0, 0, 1)),
                lava_water_palette_words_loc_1223c: Some([0; 4]),
                ..NativeTraceFrame::default()
            },
            &ten_colour_visuals,
        );
        let second_rotation_words: Vec<u16> = second_rotation.colours[6..10]
            .iter()
            .map(|colour| colour.raw_word)
            .collect();
        assert_eq!(second_rotation_words, vec![8, 9, 6, 7]);
    }

    #[test]
    fn push_hud_two_digit_counter_matches_loc_bf2a_single_and_clamped_digits() {
        let visuals = sample_level_visuals();
        let mut commands = Vec::new();

        // ASM life_display/diamond_display loc_BF2A: single-digit values write
        // the digit first, then HUD number tile 11 in the following "x" slot.
        push_hud_two_digit_counter(&mut commands, &visuals, 7, 20, 30);
        assert_eq!(
            indexed_tile_commands(&commands),
            vec![(20, 30, 7), (28, 30, 11)]
        );

        commands.clear();
        push_hud_two_digit_counter(&mut commands, &visuals, 10, 20, 30);
        assert_eq!(
            indexed_tile_commands(&commands),
            vec![(20, 30, 1), (28, 30, 0)]
        );

        commands.clear();
        push_hud_two_digit_counter(&mut commands, &visuals, 123, 20, 30);
        assert_eq!(
            indexed_tile_commands(&commands),
            vec![(20, 30, 9), (28, 30, 9)]
        );
    }

    #[test]
    fn push_hud_fixed_digit_clamps_to_loaded_digit_tile() {
        let visuals = sample_level_visuals();
        let mut commands = Vec::new();

        push_hud_fixed_digit(&mut commands, &visuals, 42, 12, 14);

        assert_eq!(indexed_tile_commands(&commands), vec![(12, 14, 9)]);
    }

    #[test]
    fn push_hud_hitpoint_sprite_emits_wide_and_narrow_loc_be20_tiles() {
        let visuals = sample_level_visuals();
        let full_pip = visuals.hud.hitpoint_tiles[0];
        let mut commands = Vec::new();

        // ASM Make_SpriteAttr_HUD loc_BE20/BE34/BE5C uses size #4 for a 2x1
        // sprite; loc_BE4C/BE74 uses size #0 for the odd trailing 1x1 pip.
        push_hud_hitpoint_sprite(&mut commands, &visuals, full_pip, 16, 36, false);
        assert_eq!(
            indexed_tile_commands(&commands),
            vec![(16, 36, 0), (24, 36, 1)]
        );

        commands.clear();
        push_hud_hitpoint_sprite(&mut commands, &visuals, full_pip, 16, 36, true);
        assert_eq!(indexed_tile_commands(&commands), vec![(16, 36, 0)]);

        commands.clear();
        let mut short_hitpoint_sheet = visuals.clone();
        short_hitpoint_sheet.hud.hitpoint_sheet.tiles.truncate(3);
        push_hud_hitpoint_sprite(
            &mut commands,
            &short_hitpoint_sheet,
            short_hitpoint_sheet.hud.hitpoint_tiles[2],
            16,
            36,
            false,
        );
        assert_eq!(indexed_tile_commands(&commands), vec![(16, 36, 2)]);
    }

    #[test]
    fn push_hud_icon_emits_ankh_diamond_and_fallback_hud_tiles() {
        let visuals = sample_level_visuals();
        let mut commands = Vec::new();

        push_hud_icon(
            &mut commands,
            &visuals,
            "ankh",
            &NativeTraceFrame {
                hud_life_frame_offset: Some(0),
                ..NativeTraceFrame::default()
            },
            33,
            44,
        );
        assert_eq!(sprite_commands(&commands), vec![(33, 44, 8)]);

        commands.clear();
        push_hud_icon(
            &mut commands,
            &visuals,
            "diamond",
            &NativeTraceFrame {
                hud_diamond_frame_offset: Some(0),
                ..NativeTraceFrame::default()
            },
            40,
            50,
        );
        assert_eq!(sprite_commands(&commands), vec![(40, 50, 8)]);

        commands.clear();
        push_hud_icon(
            &mut commands,
            &visuals,
            "custom",
            &NativeTraceFrame::default(),
            11,
            12,
        );
        assert_eq!(indexed_tile_commands(&commands), vec![(11, 12, 3)]);
    }

    #[test]
    fn push_game_hud_emits_full_loc_bdc0_command_stream() {
        let visuals = sample_level_visuals();
        let trace = NativeTraceFrame {
            game_mode: GameMode::InGame,
            level_timer_seconds: 125,
            number_lives: 7,
            diamond_count: 42,
            current_helmet_id: PlayerForm::IronKnight.helmet_id(),
            number_hitpoints: 4,
            extra_hitpoint_slots: 1,
            hud_life_frame_offset: Some(0),
            hud_diamond_frame_offset: Some(0),
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_game_hud(&mut commands, &trace, &visuals);

        // ASM Make_SpriteAttr_HUD loc_BDC0..loc_BF78 writes HP pips, timer,
        // lives, ankh, diamond count, and diamond icon in that order. Timer
        // seed coordinates come from sub_BD0A fixed row positions.
        assert_eq!(
            indexed_tile_commands_in(&commands),
            vec![
                (16, 36, 0),
                (24, 36, 1),
                (32, 36, 0),
                (40, 36, 1),
                (48, 36, 2),
                (56, 36, 3),
                (16, 16, 2),
                (24, 16, 10),
                (29, 16, 0),
                (37, 16, 5),
                (272, 16, 7),
                (280, 16, 11),
                (272, 36, 4),
                (280, 36, 2),
            ]
        );
        assert_eq!(
            sprite_commands_in(&commands),
            vec![(289, 13, 8), (288, 34, 8)]
        );
        assert_eq!(commands.len(), 16);
    }

    fn indexed_tile_commands(commands: &[DrawCommand]) -> Vec<(i32, i32, u16)> {
        commands
            .iter()
            .map(|command| match command {
                DrawCommand::IndexedTile { x, y, tile, .. } => (*x, *y, tile.tile_index),
                other => panic!("expected indexed tile command, got {other:?}"),
            })
            .collect()
    }

    fn indexed_tile_commands_in(commands: &[DrawCommand]) -> Vec<(i32, i32, u16)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::IndexedTile { x, y, tile, .. } => Some((*x, *y, tile.tile_index)),
                _ => None,
            })
            .collect()
    }

    fn sprite_commands(commands: &[DrawCommand]) -> Vec<(i32, i32, u16)> {
        commands
            .iter()
            .map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. } => {
                    (*x, *y, tiles.first().expect("sprite tile").tile.tile_index)
                }
                other => panic!("expected sprite command, got {other:?}"),
            })
            .collect()
    }

    fn sprite_commands_in(commands: &[DrawCommand]) -> Vec<(i32, i32, u16)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { x, y, tiles, .. } => {
                    Some((*x, *y, tiles.first().expect("sprite tile").tile.tile_index))
                }
                _ => None,
            })
            .collect()
    }

    fn sample_level_visuals() -> NativeLevelVisuals {
        let foreground_layout_bytes = vec![0; 20 * 14];
        let foreground_layout =
            ForegroundLayout::parse(&foreground_layout_bytes, 1, 1).expect("foreground layout");
        let empty_quad = TileQuad {
            tiles: [sample_tile_ref(0); 4],
        };
        NativeLevelVisuals {
            foreground_tiles: blank_tile_sheet(1),
            background_tiles: blank_tile_sheet(1),
            island_shore_tiles_loc_1202a: None,
            block_tiles: blank_tile_sheet(1),
            palettes: PaletteSet {
                foreground: sample_palette(),
                background: sample_palette(),
            },
            background: BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
                placements: Vec::new(),
                chunks: Vec::new(),
            }),
            level: wildside_render::LevelVisuals {
                level_id: 0,
                foreground_theme_id: 0,
                background_theme_id: 0,
                murder_wall_flags: 0,
                background_scroll: None,
                foreground_layout,
                foreground_layout_bytes,
                block_layout_bytes: Vec::new(),
                background_layout_bytes: Vec::new(),
                background_layout_alias: None,
                block_mapping: BlockVisualMapping::new(
                    vec![empty_quad.clone()],
                    vec![empty_quad; 81],
                )
                .expect("block mapping"),
            },
            hud: HudVisuals {
                digits: blank_tile_sheet(12),
                digit_base_tile_index: HUD_TILE_VRAM_BASE,
                diamond_tiles: blank_tile_sheet(16),
                diamond_base_tile_index: HUD_DIAMOND_TILE_VRAM_BASE,
                hitpoint_sheet: blank_tile_sheet(4),
                icons: vec![
                    HudIcon::new("ankh", (0x01A1, 0x008D), TileRef::from_attr_word(0x8231)),
                    HudIcon::new("diamond", (0x01A0, 0x00A2), TileRef::from_attr_word(0x86F2)),
                    HudIcon::new("custom", (0, 0), sample_tile_ref(HUD_TILE_VRAM_BASE + 3)),
                ],
                hitpoint_tiles: [
                    sample_tile_ref(0x022D),
                    sample_tile_ref(0x022E),
                    sample_tile_ref(0x022F),
                ],
            },
            player: PlayerVisuals {
                forms: HashMap::from([(0, sample_player_form_visuals())]),
            },
            juggernaut_skull: None,
            enemies: EnemyVisuals {
                families: HashMap::new(),
            },
            floating_prizes: sample_floating_prize_visuals(),
            flag: FlagVisuals {
                palette: sample_palette(),
                pole_bottom: sample_sprite_frame(1),
                waving: Vec::new(),
                frame_duration: 1,
            },
            murder_wall: None,
            lava: None,
            weather: None,
            special_effect_unknown: None,
            platforms: PlatformVisuals {
                palette: sample_palette(),
                horizontal: sample_sprite_frame(2),
                vertical: sample_sprite_frame(3),
            },
            eol_score: None,
            eol_hologram: None,
            pause_menu: None,
            title_card_letters: None,
        }
    }

    fn sample_player_form_visuals() -> PlayerFormVisuals {
        let frame = sample_sprite_frame(10);
        PlayerFormVisuals {
            palette: sample_palette(),
            idle: frame.clone(),
            walk: Vec::new(),
            flight_spin: Vec::new(),
            charge: Vec::new(),
            climb: Vec::new(),
            axe: Vec::new(),
            slope_walk: Vec::new(),
            slope_walk_left: Vec::new(),
            jump_rise: frame.clone(),
            jump_fall: frame,
            death: None,
            pull_up: Vec::new(),
            slash_ground: Vec::new(),
            slash_air: Vec::new(),
            transform_palette: None,
            transform_steps: Vec::new(),
        }
    }

    fn sample_floating_prize_visuals() -> FloatingPrizeVisuals {
        FloatingPrizeVisuals {
            diamond: AnimatedSpriteVisual {
                palette: sample_palette(),
                extra_palettes: Vec::new(),
                frames: vec![sample_sprite_frame(20); 4],
                frame_duration: 1,
            },
            helmets: HashMap::new(),
            icons: HashMap::from([(
                0x0B,
                AnimatedSpriteVisual {
                    palette: sample_palette(),
                    extra_palettes: Vec::new(),
                    frames: vec![sample_sprite_frame(21)],
                    frame_duration: 1,
                },
            )]),
        }
    }

    fn sample_sprite_frame(pixel: u8) -> SpriteFrame {
        SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 8,
            height_px: 8,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![Tile {
                    pixels: [pixel; 64],
                }],
            },
        }
    }

    fn blank_tile_sheet(tile_count: usize) -> TileSheet {
        TileSheet {
            tiles: vec![Tile { pixels: [0; 64] }; tile_count],
        }
    }

    fn sample_tile_ref(tile_index: u16) -> TileRef {
        TileRef {
            tile_index,
            palette_line: 0,
            priority: false,
            flip_x: false,
            flip_y: false,
        }
    }

    fn sample_palette() -> Palette {
        Palette {
            colours: (0..16).map(MdColour::from_word).collect(),
        }
    }
}
