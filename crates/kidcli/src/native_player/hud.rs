use super::*;

pub(super) const HUD_BG: [u8; 3] = [20, 20, 30];

pub(super) fn push_hud_number(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    value: u16,
    x: i32,
    y: i32,
) {
    for (index, digit) in value.to_string().chars().enumerate() {
        let Some(digit_value) = digit.to_digit(10) else {
            continue;
        };
        let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef {
                tile_index: level_visuals.hud.digit_base_tile_index + digit_value as u16,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        ) else {
            continue;
        };
        push_resolved_tile(commands, x + (index as i32 * 8), y, tile);
    }
}

pub(super) fn hud_counter_width(value: u16) -> i32 {
    let digit_count = value.to_string().len() as i32;
    if digit_count == 1 {
        16
    } else {
        digit_count * 8
    }
}

pub(super) fn push_hud_counter(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    value: u16,
    x: i32,
    y: i32,
) {
    push_hud_number(commands, level_visuals, value, x, y);
    if value < 10
        && let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef {
                tile_index: level_visuals.hud.digit_base_tile_index + 11,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        )
    {
        push_resolved_tile(commands, x + 8, y, tile);
    }
}

pub(super) fn push_hud_score(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    score: u32,
    x: i32,
    y: i32,
) {
    for (index, digit) in score.to_string().chars().enumerate() {
        let Some(digit_value) = digit.to_digit(10) else {
            continue;
        };
        let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef {
                tile_index: level_visuals.hud.digit_base_tile_index + digit_value as u16,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        ) else {
            continue;
        };
        push_resolved_tile(commands, x + (index as i32 * 8), y, tile);
    }
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
    let hud_y = 0i32;
    let hud_h = 18;

    commands.push(DrawCommand::Rect {
        x: 0,
        y: hud_y,
        width: FRAME_WIDTH as i32,
        height: hud_h,
        colour: HUD_BG,
    });

    let max_hp = PlayerForm::from_helmet_id(trace.current_helmet_id)
        .map(PlayerForm::max_hitpoints)
        .unwrap_or_else(|| PlayerForm::TheKid.max_hitpoints()) as i32;
    let pip_gap = 2;
    let pip_y = hud_y + 5;
    for i in 0..max_hp {
        let pip_tile = if (i as u16) < trace.number_hitpoints {
            level_visuals.hud.hitpoint_tiles[0]
        } else {
            level_visuals.hud.hitpoint_tiles[2]
        };
        if let Some(tile) = resolve_ingame_vram_tile(level_visuals, pip_tile) {
            push_resolved_tile(commands, 4 + i * (8 + pip_gap), pip_y, tile);
        }
    }

    let lives_x = 4 + max_hp * (8 + pip_gap) + 8;
    push_hud_counter(commands, level_visuals, trace.number_lives, lives_x, pip_y);
    push_hud_icon(commands, level_visuals, "ankh", lives_x + 18, pip_y);

    // Level timer: M:SS in centre of HUD (flash red background when under 30s)
    let minutes = trace.level_timer_seconds / 60;
    let seconds = trace.level_timer_seconds % 60;
    let timer_x = (FRAME_WIDTH as i32) / 2 - 16;
    if trace.level_timer_seconds <= 30 && (trace.frame_index / 15).is_multiple_of(2) {
        commands.push(DrawCommand::Rect {
            x: timer_x - 2,
            y: pip_y - 1,
            width: 40,
            height: 10,
            colour: [140, 20, 20],
        });
    }
    push_hud_number(commands, level_visuals, minutes, timer_x, pip_y);
    // Colon character: use digit tile index + 11 (separator glyph)
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
        push_resolved_tile(commands, timer_x + 8, pip_y, tile);
    }
    // Two-digit seconds: zero-pad by rendering tens then ones
    push_hud_number(commands, level_visuals, seconds / 10, timer_x + 16, pip_y);
    push_hud_number(commands, level_visuals, seconds % 10, timer_x + 24, pip_y);

    // Score display right of timer
    let score_x = timer_x + 40;
    push_hud_score(commands, level_visuals, trace.score, score_x, pip_y);

    // Helmet name if wearing one
    let form = PlayerForm::from_helmet_id(trace.current_helmet_id);
    if let Some(f) = form
        && f != PlayerForm::TheKid {
            let name = helmet_display_name(f);
            let name_x = lives_x + 30;
            commands.push(text(name_x, pip_y, name, [180, 220, 255], 1));
        }

    let diamond_icon_x = FRAME_WIDTH as i32 - 12;
    let diamond_counter_x = diamond_icon_x - 2 - hud_counter_width(trace.diamond_count);
    push_hud_counter(
        commands,
        level_visuals,
        trace.diamond_count,
        diamond_counter_x,
        pip_y,
    );
    push_hud_icon(commands, level_visuals, "diamond", diamond_icon_x, pip_y);

    // Diamond Power indicator
    if trace.diamond_power_active {
        let dp_colour = if (trace.frame_index / 4).is_multiple_of(2) {
            [255, 220, 50]
        } else {
            [255, 180, 0]
        };
        commands.push(text(diamond_counter_x - 24, pip_y, "DP", dp_colour, 1));
    }

    // 2P player turn indicator
    if trace.two_player {
        let label = if trace.current_player == 0 {
            "P1"
        } else {
            "P2"
        };
        let colour = if trace.current_player == 0 {
            [100, 180, 255]
        } else {
            [255, 140, 100]
        };
        commands.push(text(diamond_counter_x - 48, pip_y, label, colour, 1));
    }
}

pub(super) fn helmet_display_name(form: PlayerForm) -> &'static str {
    match form {
        PlayerForm::TheKid => "KID",
        PlayerForm::Skycutter => "SKY",
        PlayerForm::Cyclone => "CYC",
        PlayerForm::RedStealth => "RED",
        PlayerForm::Eyeclops => "EYE",
        PlayerForm::Juggernaut => "JUG",
        PlayerForm::IronKnight => "IRN",
        PlayerForm::Berzerker => "BRZ",
        PlayerForm::Maniaxe => "MNX",
        PlayerForm::Micromax => "MCR",
    }
}
