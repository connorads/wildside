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
const HUD_HITPOINT_START_X: i32 = 0x80 - 128;
const HUD_HITPOINT_Y: i32 = 0xA4 - 128;
const HUD_HITPOINT_SPACING_X: i32 = 0x10;

fn push_hud_fixed_digit(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    digit: u16,
    x: i32,
    y: i32,
) {
    let Some(tile) = resolve_ingame_vram_tile(
        level_visuals,
        TileRef {
            tile_index: level_visuals.hud.digit_base_tile_index + digit.min(9),
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

fn push_hud_two_digit_counter(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    value: u16,
    x: i32,
    y: i32,
) {
    let clamped = value.min(99);
    push_hud_fixed_digit(commands, level_visuals, clamped / 10, x, y);
    push_hud_fixed_digit(commands, level_visuals, clamped % 10, x + 8, y);
}

pub(super) fn push_hud_icon(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    icon_name: &str,
    x: i32,
    y: i32,
) {
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

pub(super) fn push_game_hud(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let max_hp = PlayerForm::from_helmet_id(trace.current_helmet_id)
        .map(PlayerForm::max_hitpoints)
        .unwrap_or_else(|| PlayerForm::TheKid.max_hitpoints()) as i32;
    for i in 0..max_hp {
        let pip_tile = if (i as u16) < trace.number_hitpoints {
            level_visuals.hud.hitpoint_tiles[0]
        } else {
            level_visuals.hud.hitpoint_tiles[2]
        };
        if let Some(tile) = resolve_ingame_vram_tile(level_visuals, pip_tile) {
            push_resolved_tile(
                commands,
                HUD_HITPOINT_START_X + i * HUD_HITPOINT_SPACING_X,
                HUD_HITPOINT_Y,
                tile,
            );
        }
    }

    let minutes = trace.level_timer_seconds / 60;
    let seconds = trace.level_timer_seconds % 60;
    push_hud_fixed_digit(commands, level_visuals, minutes, HUD_MINUTES_X, HUD_TIMER_Y);
    if let Some(tile) = resolve_ingame_vram_tile(
        level_visuals,
        TileRef {
            tile_index: level_visuals.hud.digit_base_tile_index + 11,
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
    push_hud_icon(commands, level_visuals, "ankh", HUD_ANKH_X, HUD_ANKH_Y);

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
    );
}
