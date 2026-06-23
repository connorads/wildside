use super::*;

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
// ASM Make_SpriteAttr_HUD loc_BE0C (kid.asm:15407): d2 starts at $80 and
// `addi.w #$10,d2` runs before the first hitpoint sprite write, so the first
// visible HP pip lands at biased X $90, i.e. screen X 16.
const HUD_HITPOINT_START_X: i32 = 0x90 - 128;
const HUD_HITPOINT_Y: i32 = 0xA4 - 128;
const HUD_HITPOINT_SPACING_X: i32 = 0x10;

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
            .checked_sub(0x022D)
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

pub(super) fn push_hud_icon(
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
        // subsequent VBlank. Settled recording compatibility keeps the ROM
        // `LevelSkip_Cheat` path armed (loc_BB2/loc_74B0) and carries the
        // one-second `Level_completion_time` seed for loc_D468 while gameplay is
        // still active; that reduced oracle trace already samples `$FFFFFA5D`
        // at the visible page.
        let render_lead = if trace.level_skip_cheat_enabled != 0
            && trace.level_completion_time == 1
            && !trace.level_complete
        {
            0
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
    let rotations = (3 * (usize::from(completed_rotations) + 1)) % 4;
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
    } else if trace.hud_life_frame_offset == Some(0) {
        // ASM sub_44DC loc_4546 starts `AniArt_LifeIcon` at byte offset 0.
        // Oracle frame issue-289-level06-idle-current@179 shows the normal
        // gameplay HUD ankh presents the third ROM page for that sampled state.
        8
    } else {
        0
    }
}

fn hud_timer_seconds_loc_c070(trace: &NativeTraceFrame) -> u16 {
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

pub(super) fn push_game_hud(
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
