use super::*;

const HUD_LIVES_X: i32 = 0x190 - 128;
const HUD_LIVES_Y: i32 = 0x90 - 128;
const HUD_ANKH_X: i32 = 0x1A1 - 128;
const HUD_ANKH_Y: i32 = 0x8D - 128;
const HUD_MINUTES_X: i32 = 0x88 - 128;
const HUD_TIMER_Y: i32 = 0x90 - 128;
const HUD_COLON_X: i32 = 0x90 - 128;
const HUD_SECONDS_TENS_X: i32 = 0x95 - 128;
const HUD_SECONDS_ONES_X: i32 = 0x9D - 128;
const HUD_DIAMOND_X: i32 = 0x1A0 - 128;
const HUD_DIAMOND_Y: i32 = 0xA2 - 128;
const HUD_DIAMOND_COUNT_X: i32 = 0x190 - 128;
const HUD_DIAMOND_COUNT_Y: i32 = 0xA4 - 128;
const HUD_DIAMOND_ANIMATION_FRAME_DURATION: usize = 6;
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

fn hud_hitpoint_tiles(number_hitpoints: u16, max_hitpoints: u16) -> Vec<HudHitpointTile> {
    // ASM Make_SpriteAttr_HUD loc_BE0C..loc_BE74 (kid.asm:15407): d0
    // (current HP) and d1 (capacity) decrement by two per visible slot. A
    // remainder of one current HP emits $822E; an odd trailing capacity slot is
    // written with size 0 and is therefore skipped by native rendering.
    let mut current = number_hitpoints as i16;
    let mut capacity = max_hitpoints as i16;
    let mut tiles = Vec::new();

    loop {
        capacity -= 2;
        current -= 2;
        let tile = if current >= 0 {
            Some(HudHitpointTile::Full)
        } else if current == -1 {
            if capacity < 0 {
                None
            } else {
                Some(HudHitpointTile::Half)
            }
        } else if capacity < 0 {
            None
        } else {
            Some(HudHitpointTile::Empty)
        };
        if let Some(tile) = tile {
            tiles.push(tile);
        }
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
    // ASM calc_display_number (kid.asm:15568): values below 10 use tile
    // $86C5 (HUD number index 11) as a blank tens digit, not a leading zero.
    let tens = if clamped < 10 { 11 } else { clamped / 10 };
    push_hud_tile_offset(commands, level_visuals, tens, x, y);
    push_hud_fixed_digit(commands, level_visuals, clamped % 10, x + 8, y);
}

fn push_hud_hitpoint_sprite(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    base_tile: TileRef,
    x: i32,
    y: i32,
) {
    // ASM Make_SpriteAttr_HUD loc_BE20/BE34/BE5C writes size byte #4 for
    // $822D/$822E/$822F. On Mega Drive this is a 2x1 tile sprite, so the
    // visible pip includes the following tile as well as the base tile.
    for tile_offset in 0..2 {
        let tile_ref = TileRef {
            tile_index: base_tile.tile_index + tile_offset,
            ..base_tile
        };
        if let Some(tile) = resolve_ingame_vram_tile(level_visuals, tile_ref) {
            push_resolved_tile(commands, x + i32::from(tile_offset) * 8, y, tile);
        }
    }
}

pub(super) fn push_hud_icon(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    icon_name: &str,
    x: i32,
    y: i32,
    frame_index: usize,
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
                push_sprite_frame_top_left(commands, frame, &icon.palette, x, y);
            }
        }
        "diamond" => {
            // ASM Init_SpriteAttr_HUD unk_1427C uses `sprite_attr $1A0,$A2,1,1,$86F2`.
            // The size bits are 1,1, i.e. a 2x2 tile sprite starting at HUD
            // diamond VRAM tile $6F2, not a single 8x8 tile. ASM sub_44DC
            // drives `unk_461C` at 6 VBlanks per frame and `sub_5D4A` DMAs
            // diamond_1..4 into the same $6EA VRAM page, so the HUD icon uses
            // the currently uploaded diamond page.
            let animation_frame = hud_diamond_animation_frame_index(
                frame_index,
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
            let frame = kid_render::SpriteFrame {
                origin_x: 0,
                origin_y: 0,
                width_px: 0x10,
                height_px: 0x10,
                tile_base_index: 0x06F2 - level_visuals.hud.diamond_base_tile_index,
                tile_sheet,
            };
            push_sprite_frame_top_left(commands, &frame, &level_visuals.palettes.foreground, x, y);
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

fn hud_diamond_animation_frame_index(frame_index: usize, frame_count: usize) -> usize {
    if frame_count == 0 {
        return 0;
    }
    (frame_index / HUD_DIAMOND_ANIMATION_FRAME_DURATION) % frame_count
}

pub(super) fn push_game_hud(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let max_hp = PlayerForm::from_helmet_id(trace.current_helmet_id).map_or_else(
        || PlayerForm::TheKid.max_hitpoints(),
        PlayerForm::max_hitpoints,
    ) as i32;
    for (i, hitpoint_tile) in hud_hitpoint_tiles(trace.number_hitpoints, max_hp as u16)
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
        );
    }

    let minutes = trace.level_timer_seconds / 60;
    let seconds = trace.level_timer_seconds % 60;
    push_hud_fixed_digit(commands, level_visuals, minutes, HUD_MINUTES_X, HUD_TIMER_Y);
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
        push_resolved_tile(commands, HUD_COLON_X, HUD_TIMER_Y, tile);
    }
    push_hud_fixed_digit(
        commands,
        level_visuals,
        seconds / 10,
        HUD_SECONDS_TENS_X,
        HUD_TIMER_Y,
    );
    push_hud_fixed_digit(
        commands,
        level_visuals,
        seconds % 10,
        HUD_SECONDS_ONES_X,
        HUD_TIMER_Y,
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
        HUD_ANKH_X,
        HUD_ANKH_Y,
        trace.frame_index,
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
        HUD_DIAMOND_X,
        HUD_DIAMOND_Y,
        trace.frame_index,
    );
}
