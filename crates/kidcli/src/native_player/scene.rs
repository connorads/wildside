use super::*;

const HEADYMETAL_WOBBLE_STAGE_1: u16 = 0x015E;
const HEADYMETAL_WOBBLE_STAGE_2: u16 = 0x00FA;
const HEADYMETAL_WOBBLE_STAGE_3: u16 = 0x0096;
const HEADYMETAL_WOBBLE_STAGE_4: u16 = 0x0064;
const FRIENDLY_PROJECTILE_COLOUR: [u8; 3] = [100, 255, 100];
const ENEMY_PROJECTILE_COLOUR: [u8; 3] = [255, 80, 40];
const FIRE_TRAIL_PLACEHOLDER_COLOUR: [u8; 3] = [255, 120, 32];
const FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME: u16 = 45;
const FIRE_DEMON_OBJECT_TYPE: u8 = 0x00;
const DRIPS_OBJECT_TYPE: u8 = 0x16;
const PRIZE_GENERIC_LIFETIME_LOW_BYTE: u8 = 0x20;
const BLOCK_DEBRIS_TILE_ATTR_SUB_2A4C: u16 = 0x8228;
const BLOCK_DEBRIS_BASE_X_SUB_292E: [i16; 5] = [0, 0, 8, 8, 4];
const BLOCK_DEBRIS_BASE_Y_SUB_292E: [i16; 5] = [0, 8, 8, 0, 4];
const BLOCK_DEBRIS_VEL_D6_2_SUB_292E: [(i16, i16); 5] =
    [(-1, -4), (-1, -3), (1, -3), (1, -4), (1, -5)];
const BLOCK_DEBRIS_VEL_D6_0_SUB_292E: [(i16, i16); 5] =
    [(-1, -2), (-2, 0), (1, 0), (1, -1), (-1, -2)];
const BLOCK_DEBRIS_VEL_D6_1_SUB_292E: [(i16, i16); 5] =
    [(-5, -3), (-4, -3), (-3, -3), (-2, -3), (-1, -3)];
const BLOCK_DEBRIS_VEL_D6_DEFAULT_SUB_292E: [(i16, i16); 5] =
    [(5, -3), (4, -3), (3, -3), (2, -3), (1, -3)];

const FIRE_DEMON_DEATH_TRAIL_FRAMES: [kid_render::SpriteFrame; 6] = [
    fire_demon_trail_frame(0x0B0, 0x07, 0x00, 0x0E, 0x08),
    fire_demon_trail_frame(0x0B2, 0x07, 0x00, 0x0D, 0x07),
    fire_demon_trail_frame(0x0B4, 0x06, 0x00, 0x0C, 0x06),
    fire_demon_trail_frame(0x0B6, 0x07, 0x00, 0x0E, 0x08),
    fire_demon_trail_frame(0x0B8, 0x07, 0x00, 0x0D, 0x08),
    fire_demon_trail_frame(0x0BA, 0x06, 0x00, 0x0C, 0x06),
];

const fn fire_demon_trail_frame(
    tile_base_index: u16,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
) -> kid_render::SpriteFrame {
    kid_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet: kid_render::TileSheet { tiles: Vec::new() },
    }
}

fn drips_drop_child_frame(
    visuals: &kid_render::AnimatedSpriteVisual,
) -> Option<kid_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    Some(kid_render::SpriteFrame {
        // ASM Drips.asm loc_33424 writes `LnkTo_unk_C7B26` to the falling
        // child. `unk_C7B26` is `sprite_frame_vram $082,$02,$09,$05,$0A`.
        origin_x: 0x02,
        origin_y: 0x09,
        width_px: 0x05,
        height_px: 0x0A,
        tile_base_index: 0x082,
        tile_sheet,
    })
}

fn block_debris_velocity_sub_292e(direction: u16) -> &'static [(i16, i16); 5] {
    match direction {
        // ASM sub_292E loc_2988: d6=2 uses `unk_29FC`/`unk_2A06`.
        2 => &BLOCK_DEBRIS_VEL_D6_2_SUB_292E,
        // ASM sub_292E loc_297A: d6=0 uses `unk_2A10`/`unk_2A1A`.
        0 => &BLOCK_DEBRIS_VEL_D6_0_SUB_292E,
        // ASM sub_292E loc_296C: d6=1 uses `unk_2A38`/`unk_2A42`.
        1 => &BLOCK_DEBRIS_VEL_D6_1_SUB_292E,
        // ASM sub_292E fallthrough loc_2994: d6=3/right/default uses
        // `unk_2A24`/`unk_2A2E`.
        _ => &BLOCK_DEBRIS_VEL_D6_DEFAULT_SUB_292E,
    }
}

fn push_block_destroy_debris_sub_292e(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x_px: u16,
    y_px: u16,
    direction: u16,
    camera_x_px: i32,
    camera_y_px: i32,
) -> bool {
    let Some(tile) = resolve_ingame_vram_tile(
        level_visuals,
        TileRef::from_attr_word(BLOCK_DEBRIS_TILE_ATTR_SUB_2A4C),
    ) else {
        return false;
    };

    let velocities = block_debris_velocity_sub_292e(direction);
    for index in 0..5 {
        // ASM `sub_292E` seeds five sprite-worker rows from `unk_29E8`/
        // `unk_29F2`, then `sub_2A4C` immediately advances by the row's
        // direction-selected velocity before writing sprite tile `$8228`.
        let x = i32::from(x_px)
            + i32::from(BLOCK_DEBRIS_BASE_X_SUB_292E[index])
            + i32::from(velocities[index].0)
            - camera_x_px;
        let y = i32::from(y_px)
            + i32::from(BLOCK_DEBRIS_BASE_Y_SUB_292E[index])
            + i32::from(velocities[index].1)
            - camera_y_px;
        push_resolved_tile(commands, x, y, tile.clone());
    }

    true
}

/// Controls which layers are rendered. Used for layer-isolated diagnosis.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LayerFilter {
    pub background: bool,
    pub terrain: bool,
    pub block_overrides: bool,
    pub platforms: bool,
    pub enemies: bool,
    pub player: bool,
    pub effects: bool,
    pub hud: bool,
}

impl Default for LayerFilter {
    fn default() -> Self {
        Self {
            background: true,
            terrain: true,
            block_overrides: true,
            platforms: true,
            enemies: true,
            player: true,
            effects: true,
            hud: true,
        }
    }
}

impl LayerFilter {
    /// A filter with only one layer enabled.
    pub fn solo(layer_name: &str) -> Self {
        let mut f = Self {
            background: false,
            terrain: false,
            block_overrides: false,
            platforms: false,
            enemies: false,
            player: false,
            effects: false,
            hud: false,
        };
        match layer_name {
            "background" => f.background = true,
            "terrain" => f.terrain = true,
            "block_overrides" => f.block_overrides = true,
            "platforms" => f.platforms = true,
            "enemies" => f.enemies = true,
            "player" => f.player = true,
            "effects" => f.effects = true,
            "hud" => f.hud = true,
            _ => {}
        }
        f
    }
}

fn push_fire_demon_trail(
    commands: &mut Vec<DrawCommand>,
    sx: i32,
    sy: i32,
    _remaining_frames: u16,
) {
    // Degraded-asset fallback. ASM Fire_Demon loc_3A63C cycles
    // `stru_3A622`'s six frames `unk_C7776..unk_C779E`
    // (`sprite_frame_vram $0B0..$0BA`); when the Fire Demon visual bundle
    // is present the renderer uses those tiles directly through
    // `push_fire_demon_trail_sprite`. The previous six-phase rect
    // choreography assigned hand-invented outer/core rect compositions to
    // each ROM phase even though the ASM only dictates which sprite tile
    // to swap to. Collapse to a single warm-orange placeholder rect so
    // missing-asset states stay visible without inventing per-phase
    // geometry the ROM does not specify.
    commands.push(DrawCommand::Rect {
        x: sx - 2,
        y: sy - 3,
        width: 4,
        height: 5,
        colour: FIRE_TRAIL_PLACEHOLDER_COLOUR,
    });
}

fn push_fire_demon_trail_sprite(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    sx: i32,
    sy: i32,
    remaining_frames: u16,
) -> bool {
    let Some(fire_demon_visuals) = level_visuals.enemies.families.get(&FIRE_DEMON_OBJECT_TYPE)
    else {
        return false;
    };
    let Some(source_frame) = fire_demon_visuals.frames.first() else {
        return false;
    };
    let elapsed_frames =
        FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME.saturating_sub(remaining_frames) as usize;
    let descriptor =
        &FIRE_DEMON_DEATH_TRAIL_FRAMES[elapsed_frames % FIRE_DEMON_DEATH_TRAIL_FRAMES.len()];
    let frame = kid_render::SpriteFrame {
        tile_sheet: source_frame.tile_sheet.clone(),
        ..descriptor.clone()
    };
    // ASM Fire_Demon loc_3A63C copies the parent Fire Demon tile base/palette
    // and initialises `stru_3A622`, whose frames are
    // `unk_C7776..unk_C779E` (`sprite_frame_vram $0B0..$0BA`).
    push_sprite_frame(commands, &frame, &fire_demon_visuals.palette, sx, sy);
    true
}

pub(crate) fn build_scene(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: Option<&NativeLevelVisuals>,
    block_grid: Option<&BlockGrid>,
) -> Scene {
    build_scene_filtered(
        trace,
        debug_overlay_visible,
        level_visuals,
        block_grid,
        LayerFilter::default(),
    )
}

pub(crate) fn build_scene_filtered(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: Option<&NativeLevelVisuals>,
    block_grid: Option<&BlockGrid>,
    filter: LayerFilter,
) -> Scene {
    match trace.game_mode {
        GameMode::InGame | GameMode::DemoPlay => build_game_scene_filtered(
            trace,
            debug_overlay_visible,
            level_visuals.expect("Level 1 visuals are required for gameplay scenes"),
            block_grid,
            filter,
        ),
        GameMode::LevelTransition => build_transition_scene(trace, level_visuals),
        GameMode::OptionsMenu => build_options_scene(trace),
        GameMode::EndingSequence => build_ending_scene(trace),
        _ => build_status_scene(trace, debug_overlay_visible),
    }
}

pub(super) fn build_transition_scene(
    trace: &NativeTraceFrame,
    level_visuals: Option<&NativeLevelVisuals>,
) -> Scene {
    let mut commands = Vec::new();
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [0, 0, 0],
    });

    // ASM Load_TitleCard `loc_1AB26` / `loc_1AC70` (kid.asm:28750-28986)
    // builds a Title_Letter object per glyph in the level's title-card script.
    // Each worker decompresses its sprite from `ArtComp_19C68_TtlCardLetters`
    // into VRAM at tile `$8120`, picks one of four random off-screen entry
    // poses via `loc_1ACB0`..`loc_1ACFE`, then glides toward the script-side
    // target through `loc_1AD2A`/`loc_1AD3A` (axis-independent step add).
    // The engine simulates this in `starting_level_title_workers`; each worker
    // is surfaced through `active_objects` with `Enemy{object_type:0}` and
    // debug fields `gfx_slot_index` + `glyph`. Render each visible glyph at
    // its current interpolated (x_pos, y_pos).
    let title_card_art = level_visuals.and_then(|lv| lv.title_card_letters.as_ref());
    let mut drew_any_glyph = false;
    for object in &trace.active_objects {
        let NativeObjectKind::Enemy { object_type: 0 } = object.kind else {
            continue;
        };
        let Some(glyph_str) = object.debug_fields.get("glyph") else {
            continue;
        };
        let glyph = parse_title_glyph(glyph_str);
        // ASM loc_1AC70 stores worker positions as 16.16 fixed-point signed
        // ints; entry poses include `$FFE0`/`$FFD0` (left of screen) and
        // `$160` (right of screen). Interpret as i32 to keep off-screen
        // start positions correct.
        let sx = (object.x_pos as i32) >> 16;
        let sy = (object.y_pos as i32) >> 16;
        if let Some(art) = title_card_art
            && push_title_card_glyph_tiles(&mut commands, art, glyph, sx, sy)
        {
            drew_any_glyph = true;
            continue;
        }
        // Fallback: when the title-card letter art isn't loaded, render the
        // glyph as ASCII via the placeholder font. The per-glyph motion is
        // still ASM-faithful via the worker pose; only the letter art is
        // approximate.
        if let Some(ch) = title_glyph_to_ascii(glyph) {
            commands.push(text(sx, sy, &ch.to_string(), [255, 240, 200], 2));
            drew_any_glyph = true;
        }
    }

    // Fallback: death/respawn transitions never seed title workers, and the
    // pre-cold-boot frames briefly trace LevelTransition with no workers
    // either. Keep the lightweight name/ID/score readout for those windows.
    if !drew_any_glyph {
        let level_name = trace.current_level_name.as_deref().unwrap_or("Unknown");
        let level_text = level_name.to_string();
        let text_w = level_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 20;
        commands.push(text(text_x, text_y, &level_text, [255, 255, 255], 2));

        let id_text = format!("Level {:02X}", trace.current_level.0);
        let id_w = id_text.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - id_w) / 2,
            text_y + 22,
            &id_text,
            [120, 120, 120],
            1,
        ));

        let info_text = format!("Lives: {}  Score: {}", trace.number_lives, trace.score);
        let info_w = info_text.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - info_w) / 2,
            text_y + 40,
            &info_text,
            [100, 140, 180],
            1,
        ));
    }

    Scene {
        background: [0, 0, 0],
        commands,
    }
}

fn parse_title_glyph(value: &str) -> u8 {
    let trimmed = value.strip_prefix("0x").unwrap_or(value);
    u8::from_str_radix(trimmed, 16).unwrap_or(0)
}

fn title_glyph_to_ascii(glyph: u8) -> Option<char> {
    // ASM `loc_1AC70` -> `off_1AC2A` (kid.asm:28859-28893) maps `glyph - $61`
    // into a 35-entry sprite table covering letters `a..z` (indices 0..$19)
    // and five wide glyphs at $7B..$7F (indices $1A..$1E), plus four marker
    // glyphs at $80..$83 (indices $1F..$22). The control bytes $00 (newline),
    // $84 (space), and $FF (end) never allocate a worker.
    match glyph {
        b'a'..=b'z' => Some((glyph - b'a' + b'A') as char),
        0x7B..=0x7F => Some('*'),
        0x80..=0x83 => Some('!'),
        _ => None,
    }
}

/// Per-glyph sprite descriptor from ASM `unk_C86E8`..`unk_C87F8`
/// (kid.asm:49238-49272). Each `off_1AC2A` entry points to a
/// `sprite_frame_vram tile_offset, ox, oy, width, height` macro; the
/// title-card letter sheet uses `tile_offset` as a byte offset into the
/// uploaded VRAM region. For native rendering the offset is the tile
/// index within the loaded letter `TileSheet`, and dimensions are the
/// sprite extent in pixels.
struct TitleCardGlyphFrame {
    tile_offset: u16,
    width_px: u16,
    height_px: u16,
}

fn title_card_glyph_frame(glyph: u8) -> Option<TitleCardGlyphFrame> {
    // ASM `unk_C86E8` onwards: most letters are 2×2 tiles (16×16 px);
    // `i` (`$69`) is 1×2 (8×16), `$83` is 1×1 (8×8), and the wide
    // glyphs `$7B`..`$80` use non-square tile counts. Hard-code the
    // table verbatim from kid.asm:49238-49272 so the tile offsets and
    // extents match the ROM exactly.
    let (tile_offset, width_px, height_px) = match glyph {
        b'a' => (0x000, 0x10, 0x10),
        b'b' => (0x004, 0x10, 0x10),
        b'c' => (0x008, 0x10, 0x10),
        b'd' => (0x00C, 0x10, 0x10),
        b'e' => (0x010, 0x10, 0x10),
        b'f' => (0x014, 0x10, 0x10),
        b'g' => (0x018, 0x10, 0x10),
        b'h' => (0x01C, 0x10, 0x10),
        b'i' => (0x020, 0x08, 0x10),
        b'j' => (0x022, 0x10, 0x10),
        b'k' => (0x026, 0x10, 0x10),
        b'l' => (0x02A, 0x10, 0x10),
        b'm' => (0x02E, 0x10, 0x10),
        b'n' => (0x032, 0x10, 0x10),
        b'o' => (0x036, 0x10, 0x10),
        b'p' => (0x03A, 0x10, 0x10),
        b'q' => (0x03E, 0x10, 0x10),
        b'r' => (0x042, 0x10, 0x10),
        b's' => (0x046, 0x10, 0x10),
        b't' => (0x04A, 0x10, 0x10),
        b'u' => (0x04E, 0x10, 0x10),
        b'v' => (0x052, 0x10, 0x10),
        b'w' => (0x056, 0x10, 0x10),
        b'x' => (0x05A, 0x10, 0x10),
        b'y' => (0x05E, 0x10, 0x10),
        b'z' => (0x062, 0x10, 0x10),
        0x7B => (0x066, 0x10, 0x20),
        0x7C => (0x06E, 0x28, 0x10),
        0x7D => (0x078, 0x20, 0x10),
        0x7E => (0x080, 0x18, 0x10),
        0x7F => (0x086, 0x18, 0x10),
        0x80 => (0x08C, 0x60, 0x10),
        0x81 => (0x0A4, 0x08, 0x18),
        0x82 => (0x0A7, 0x10, 0x18),
        0x83 => (0x0AD, 0x08, 0x08),
        _ => return None,
    };
    Some(TitleCardGlyphFrame {
        tile_offset,
        width_px,
        height_px,
    })
}

/// Emit `IndexedTile` commands for every 8×8 tile in the glyph's sprite,
/// laid out column-major as the MD sprite system does. Returns true when
/// the glyph mapping resolves AND the tile sheet contains the
/// referenced tiles. ASM `loc_1AC70` writes `move.w #$409,vram_tile`,
/// which puts the letters on palette line 1.
fn push_title_card_glyph_tiles(
    commands: &mut Vec<DrawCommand>,
    art: &kid_render::TitleCardLettersVisuals,
    glyph: u8,
    sx: i32,
    sy: i32,
) -> bool {
    let Some(frame) = title_card_glyph_frame(glyph) else {
        return false;
    };
    let cols = frame.width_px / 8;
    let rows = frame.height_px / 8;
    let palette = Arc::new(art.palette.clone());
    let tile_sheet = Arc::new(art.tile_sheet.clone());
    let mut emitted = 0u32;
    for col in 0..cols {
        for row in 0..rows {
            // ASM sprite VRAM layout is column-major: tile_offset advances by
            // (rows) when moving to the next column. Mirroring the ROM's
            // column-first tile order keeps the rendered letter pixel-for-
            // pixel aligned with `sprite_frame_vram`.
            let tile_index = frame.tile_offset + (col * rows) + row;
            if usize::from(tile_index) >= tile_sheet.tiles.len() {
                continue;
            }
            commands.push(DrawCommand::IndexedTile {
                x: sx + i32::from(col) * 8,
                y: sy + i32::from(row) * 8,
                tile_sheet: tile_sheet.clone(),
                tile: kid_render::TileRef {
                    tile_index,
                    // The loaded `Title_card_letters.bin` palette holds only
                    // the 16 colours destined for VRAM palette line 1. Native
                    // stores those colours starting at slot 0 of the Palette
                    // struct, so resolve via `palette_line: 0` — using line 1
                    // would offset past the end of the loaded data.
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
                palette: palette.clone(),
            });
            emitted += 1;
        }
    }
    emitted > 0
}

pub(super) fn build_status_scene(trace: &NativeTraceFrame, debug_overlay_visible: bool) -> Scene {
    let mut commands = Vec::new();

    if trace.game_complete {
        return build_victory_scene(trace);
    }

    // Sega screen: ASM Load_SegaScreen (kid.asm:25444) decompresses
    // ArtComp_12F30_Sega into VRAM at $1780, loads Pal_12EEC (background
    // word $0000 = black) into CRAM, and writes the 4×12-tile logo at
    // nametable rows $061C/$069C/$071C/$079C. The interactive player
    // renders this through `kid_md::render_vdp_into` (native_player::run);
    // this fallback path only fires in sweep / recording / TAS-viewer
    // contexts where the VDP snapshot is unavailable. The previous blue
    // background and 4×-scaled "SEGA" text were invented placeholders —
    // the ROM never renders a typed string, and palette index 0 is black,
    // not blue.
    if trace.game_mode == GameMode::SegaScreen {
        return Scene {
            background: [0, 0, 0],
            commands,
        };
    }

    // Intro sequences: ASM `Load_IntroSequence1..5` / `Load_IntroSequence6`
    // (kid.asm:29207-29464) build tile-based VRAM compositions for each
    // segment — none of the slogans ("The Wild Side", "Heady Metal's
    // Challenge", "Enter the Game") exist as ROM-renderable ASCII text;
    // they're 4bpp tile art uploaded via `sub_1B850` / its per-segment
    // siblings. The interactive player routes through `kid_md::render_vdp_into`
    // when `runtime.vdp_state()` is populated; this fallback only fires in
    // sweep/recording/TAS-viewer contexts where the VDP snapshot is absent.
    // The previous English slogan + "Press Start to skip" text was invented
    // to fill the screen — collapse to honest black until the tile art is
    // wired through the asset bundle, matching the Sega-screen treatment.
    if matches!(
        trace.game_mode,
        GameMode::IntroSequence1
            | GameMode::IntroSequence2
            | GameMode::IntroSequence3
            | GameMode::IntroSequence4
            | GameMode::IntroSequence5
            | GameMode::IntroSequence6
    ) {
        return Scene {
            background: [0, 0, 0],
            commands,
        };
    }

    // Title screen
    // Background
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [8, 12, 20],
    });

    // Title. ASM `Obj_TitleMenu` (kid.asm:30310) renders the title screen
    // through tile art uploaded by `Load_TitleScreen` — the ROM never
    // displays typed ASCII. The interactive player routes the title through
    // `kid_md::render_vdp_into` when `runtime.vdp_state()` is populated;
    // this fallback only fires in sweep/recording/TAS-viewer contexts.
    // Keep the typed "KID CHAMELEON" as a degraded-asset readability cue —
    // the previous "Native Playtest" subtitle was a pure dev invention and
    // is dropped so the fallback doesn't claim a sub-heading the ROM never
    // shows.
    let title = "KID CHAMELEON";
    let title_w = title.len() as i32 * 8 * 3;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        3,
    ));

    // Blinking "PRESS START" (blink every 30 frames)
    if (trace.frame_index / 30).is_multiple_of(2) {
        let start_text = "PRESS START";
        let start_w = start_text.len() as i32 * 8 * 2;
        commands.push(text(
            (FRAME_WIDTH as i32 - start_w) / 2,
            120,
            start_text,
            [255, 255, 255],
            2,
        ));
    }

    // Controls help
    commands.push(text(
        20,
        160,
        "Arrows/WASD move  Z/Space jump  X special  C/Shift run",
        [140, 140, 140],
        1,
    ));
    commands.push(text(
        20,
        174,
        "R restart level  F10 dump record  Tab debug  F11 fullscreen  Esc quit",
        [140, 140, 140],
        1,
    ));
    commands.push(text(
        20,
        188,
        "B options menu  N/P warp level",
        [140, 140, 140],
        1,
    ));
    if debug_overlay_visible {
        commands.push(text(
            30,
            160,
            &format!(
                "Camera {:08X},{:08X}",
                trace.camera_x_pos, trace.camera_y_pos
            ),
            [160, 214, 255],
            1,
        ));
    }

    Scene {
        background: [18, 24, 32],
        commands,
    }
}

pub(super) fn build_options_scene(trace: &NativeTraceFrame) -> Scene {
    if let Some(track) = trace.sound_test_track {
        return build_sound_test_scene(track);
    }

    let mut commands = Vec::new();

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [8, 12, 20],
    });

    let title = "OPTIONS";
    let title_w = title.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        2,
    ));

    let cursor = trace.options_cursor.unwrap_or(0) as i32;
    let two_p_label = if trace.options_two_player_two_controllers {
        "2 PLAYERS: TWO CONTROLLERS"
    } else {
        "2 PLAYERS: ONE CONTROLLER"
    };
    let controls_label = match trace.options_controls_scheme {
        0 => "CONTROLS: A SPD  B JMP  C SPC",
        1 => "CONTROLS: A JMP  B SPD  C SPC",
        2 => "CONTROLS: A SPD  B SPC  C JMP",
        3 => "CONTROLS: A SPC  B SPD  C JMP",
        4 => "CONTROLS: A JMP  B SPC  C SPD",
        _ => "CONTROLS: A SPC  B JMP  C SPD",
    };
    let speed_label = if trace.options_speed_button_fast {
        "SPEED BUTTON: FAST ACTION"
    } else {
        "SPEED BUTTON: NORMAL ACTION"
    };
    let items = [two_p_label, controls_label, speed_label, "EXIT"];
    for (i, item) in items.iter().enumerate() {
        let y = 72 + i as i32 * 24;
        let colour = if i as i32 == cursor {
            [255, 255, 255]
        } else {
            [120, 120, 120]
        };
        commands.push(text(44, y, item, colour, 1));
        if i as i32 == cursor {
            commands.push(text(28, y, ">", [255, 220, 80], 1));
        }
    }

    commands.push(text(
        24,
        184,
        "Up/Down select  Left/Right change  Start/B back",
        [100, 100, 100],
        1,
    ));

    Scene {
        background: [8, 12, 20],
        commands,
    }
}

fn build_sound_test_scene(track: u8) -> Scene {
    let mut commands = Vec::new();

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [8, 12, 20],
    });

    let title = "SOUND TEST";
    let title_w = title.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        2,
    ));

    let track_label = format!("TRACK: {track:02X}");
    let label_w = track_label.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - label_w) / 2,
        100,
        &track_label,
        [255, 255, 255],
        2,
    ));

    commands.push(text(
        44,
        172,
        "Left/Right +1  Up/Down +16  A/C play  B back",
        [100, 100, 100],
        1,
    ));

    Scene {
        background: [8, 12, 20],
        commands,
    }
}

pub(super) fn build_victory_scene(trace: &NativeTraceFrame) -> Scene {
    let mut commands = Vec::new();

    // Dark green background
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [4, 20, 8],
    });

    let title = "CONGRATULATIONS";
    let title_w = title.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - title_w) / 2,
        40,
        title,
        [255, 220, 80],
        2,
    ));

    let sub = "YOU HAVE COMPLETED";
    let sub_w = sub.len() as i32 * 8;
    commands.push(text(
        (FRAME_WIDTH as i32 - sub_w) / 2,
        72,
        sub,
        [180, 220, 180],
        1,
    ));

    let game = "KID CHAMELEON";
    let game_w = game.len() as i32 * 8 * 2;
    commands.push(text(
        (FRAME_WIDTH as i32 - game_w) / 2,
        92,
        game,
        [120, 255, 120],
        2,
    ));

    let score_line = format!("FINAL SCORE: {}", trace.score);
    let score_w = score_line.len() as i32 * 8;
    commands.push(text(
        (FRAME_WIDTH as i32 - score_w) / 2,
        130,
        &score_line,
        [255, 255, 255],
        1,
    ));

    // Blinking "PRESS START" to return to title
    if (trace.frame_index / 30).is_multiple_of(2) {
        let start_text = "PRESS START";
        let start_w = start_text.len() as i32 * 8 * 2;
        commands.push(text(
            (FRAME_WIDTH as i32 - start_w) / 2,
            170,
            start_text,
            [255, 255, 255],
            2,
        ));
    }

    Scene {
        background: [8, 32, 16],
        commands,
    }
}

/// Credits text screens matching ASM EndText1 (kid.asm:32804-32932).
const CREDITS_SCREENS: &[&[&str]] = &[
    &[
        "CONGRATULATIONS",
        "",
        "YOU HAVE FREED THEM",
        "FROM THE EVIL BOSS",
        "",
        "YOU WILL BE REMEMBERED",
        "AS THE FIRST HOLOGRAPHIC",
        "GAME CHAMPION ...",
        "THE KID CHAMELEON.",
    ],
    &[
        "AND NOW ...",
        "",
        "THE PEOPLE WHO GAVE",
        "UP THEIR LIVES, WIVES",
        "AND SANITY TO MAKE THIS",
        "GAME A REALITY ...",
        "",
        "THE SEGA TECHNICAL",
        "INSTITUTE TEAM.",
    ],
    &[
        "... GAME DESIGN ..",
        "",
        "HOYT NG",
        "BILL DUNN",
        "RICK MACARAEG",
        "GRAEME BAYLESS",
    ],
    &[
        "... SOFTWARE ...",
        "",
        "BC TCHIU LE",
        "MARK CERNY",
        "STEVE WOITA",
        "BILL WILLIS",
    ],
    &[
        "... ART ...",
        "",
        "ALAN ACKERMAN",
        "BRENDA ROSS",
        "CRAIG STITT",
        "PAUL MICA",
        "JUDY TOTOYA",
    ],
    &[
        "SOUND BY",
        "NU ROMANTIC PRODUCTIONS",
        "",
        "SPECIAL THANKS TO ...",
        "SCOTT CHANDLER",
        "HUGH BOWEN",
        "HAVEN CARTER",
        "AND THE TEST GROUP",
    ],
    &["THE END", "", "THANK YOU FOR PLAYING!"],
];

/// ASM timing constants for the ending sequence phases.
const ENDING_PHASE1_END: usize = 40; // pause
const ENDING_PHASE2_END: usize = ENDING_PHASE1_END + 200; // music delay
const ENDING_PHASE3_END: usize = ENDING_PHASE2_END + 256; // fade to black
/// Credits screens start after the fade.
const CREDITS_START: usize = ENDING_PHASE3_END;
/// Each credits screen: 112 frames scroll + 200 frames display = 312 frames.
const CREDITS_PER_SCREEN: usize = 312;

pub(super) fn build_ending_scene(trace: &NativeTraceFrame) -> Scene {
    let mut commands = Vec::new();
    let elapsed = trace.ending_elapsed.unwrap_or(0);

    // Phase 1-3: fade to black (black screen, the level has already been cleared)
    if elapsed < CREDITS_START {
        // Calculate fade intensity: fully bright at start, fading to black
        let fade_progress = if elapsed >= ENDING_PHASE2_END {
            let fade_elapsed = elapsed - ENDING_PHASE2_END;
            (fade_elapsed * 255 / 256).min(255) as u8
        } else {
            0
        };
        let brightness = 255u8.saturating_sub(fade_progress);
        commands.push(DrawCommand::Rect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: FRAME_HEIGHT as i32,
            colour: [0, 0, brightness / 10], // dark blue fading to black
        });
        return Scene {
            background: [0, 0, 0],
            commands,
        };
    }

    // Credits phase: determine which screen to show
    let credits_elapsed = elapsed - CREDITS_START;
    let screen_index = credits_elapsed / CREDITS_PER_SCREEN;

    // Black background
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [4, 8, 20],
    });

    if screen_index < CREDITS_SCREENS.len() {
        let screen = CREDITS_SCREENS[screen_index];
        let within_screen = credits_elapsed % CREDITS_PER_SCREEN;

        // Fade in during scroll phase, full brightness during display
        let alpha = if within_screen < 112 {
            ((within_screen as f32 / 112.0) * 255.0) as u8
        } else {
            255
        };

        let start_y = 40i32;
        let line_height = 18i32;
        for (i, line) in screen.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let y = start_y + i as i32 * line_height;
            let line_w = line.len() as i32 * 8;
            let colour = if i == 0 && screen_index < 2 {
                // Title lines in gold
                [
                    alpha,
                    (alpha as u16 * 220 / 255) as u8,
                    (alpha as u16 * 80 / 255) as u8,
                ]
            } else {
                [alpha, alpha, alpha]
            };
            commands.push(text((FRAME_WIDTH as i32 - line_w) / 2, y, line, colour, 1));
        }
    }

    Scene {
        background: [4, 8, 20],
        commands,
    }
}

pub(super) fn resolve_ingame_vram_tile(
    level_visuals: &NativeLevelVisuals,
    tile_ref: TileRef,
) -> Option<ResolvedSceneTile> {
    let foreground_range_end =
        FOREGROUND_TILE_VRAM_BASE + level_visuals.foreground_tiles.tiles.len() as u16;
    if (FOREGROUND_TILE_VRAM_BASE..foreground_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.foreground_tiles.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - FOREGROUND_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    let block_range_end = BLOCK_TILE_VRAM_BASE + level_visuals.block_tiles.tiles.len() as u16;
    if (BLOCK_TILE_VRAM_BASE..block_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.block_tiles.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - BLOCK_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    let hud_range_end = HUD_TILE_VRAM_BASE + level_visuals.hud.digits.tiles.len() as u16;
    if (HUD_TILE_VRAM_BASE..hud_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.hud.digits.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - HUD_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    let hud_diamond_range_end =
        HUD_DIAMOND_TILE_VRAM_BASE + level_visuals.hud.diamond_tiles.tiles.len() as u16;
    if (HUD_DIAMOND_TILE_VRAM_BASE..hud_diamond_range_end).contains(&tile_ref.tile_index) {
        return Some(ResolvedSceneTile {
            tile_sheet: Arc::new(level_visuals.hud.diamond_tiles.clone()),
            tile: TileRef {
                tile_index: tile_ref.tile_index - HUD_DIAMOND_TILE_VRAM_BASE,
                ..tile_ref
            },
            palette: Arc::new(level_visuals.palettes.foreground.clone()),
        });
    }

    None
}

pub(super) fn push_resolved_tile(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    tile: ResolvedSceneTile,
) {
    commands.push(DrawCommand::IndexedTile {
        x,
        y,
        tile_sheet: tile.tile_sheet,
        tile: tile.tile,
        palette: tile.palette,
    });
}

pub(super) fn push_background_tiles(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let tile_sheet = Arc::new(level_visuals.background_tiles.clone());
    let palette = Arc::new(background_palette_for_scene(level_visuals));

    match &level_visuals.background {
        kid_render::BackgroundVisuals::Chunked(background) => {
            for placement in &background.placements {
                let Some(chunk) = background.chunks.get(usize::from(placement.chunk_id)) else {
                    continue;
                };
                for row in 0..chunk.height_tiles {
                    for column in 0..chunk.width_tiles {
                        let tile_index =
                            usize::from(row) * usize::from(chunk.width_tiles) + usize::from(column);
                        let Some(background_tile) = chunk.tiles.get(tile_index) else {
                            continue;
                        };
                        let (screen_x, screen_y) = background_tile_screen_position(
                            trace,
                            level_visuals,
                            placement.x_tiles + column,
                            placement.y_tiles + row,
                            camera_x_px,
                            camera_y_px,
                        );
                        if screen_x <= -8
                            || screen_y <= -8
                            || screen_x >= FRAME_WIDTH as i32
                            || screen_y >= FRAME_HEIGHT as i32
                        {
                            continue;
                        }
                        commands.push(DrawCommand::IndexedTile {
                            x: screen_x,
                            y: screen_y,
                            tile_sheet: tile_sheet.clone(),
                            tile: TileRef {
                                tile_index: background_tile.tile_index,
                                palette_line: 0,
                                priority: false,
                                flip_x: background_tile.flip_x,
                                flip_y: false,
                            },
                            palette: palette.clone(),
                        });
                    }
                }
            }
        }
        kid_render::BackgroundVisuals::Layered(background) => {
            for (row_index, row) in background.rows.iter().enumerate() {
                let shimmer_offset = layered_background_shimmer_offset(
                    trace.frame_index,
                    row_index,
                    row.selector,
                    background.effect_start_row_selector,
                );
                for (column, tile) in row.tiles.iter().enumerate() {
                    let (screen_x, screen_y) = background_tile_screen_position(
                        trace,
                        level_visuals,
                        column as u16,
                        row_index as u16,
                        camera_x_px,
                        camera_y_px,
                    );
                    let screen_x = screen_x + shimmer_offset;
                    if screen_x <= -8
                        || screen_y <= -8
                        || screen_x >= FRAME_WIDTH as i32
                        || screen_y >= FRAME_HEIGHT as i32
                    {
                        continue;
                    }
                    commands.push(DrawCommand::IndexedTile {
                        x: screen_x,
                        y: screen_y,
                        tile_sheet: tile_sheet.clone(),
                        tile: *tile,
                        palette: palette.clone(),
                    });
                }
            }
        }
    }
}

fn background_palette_for_scene(level_visuals: &NativeLevelVisuals) -> Palette {
    let mut palette = level_visuals.palettes.background.clone();
    if level_visuals.level.background_theme_id == 9 && palette.colours.len() >= 8 {
        // ASM sub_129AE swaps CRAM slot 8 with slot 10 for forest backgrounds
        // after loading the 8-colour background palette into line 1.
        let original = palette.colours.clone();
        palette.colours[0] = original[2];
        palette.colours[1] = original[1];
        palette.colours[2] = kid_render::MdColour::from_word(0);
        palette.colours[3..8].copy_from_slice(&original[3..8]);
    }
    palette
}

fn push_sprite_frame(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    palette: &Palette,
    screen_x: i32,
    screen_y: i32,
) {
    let facing = Direction::Right;
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette.clone()),
        tiles: build_sprite_tiles(frame, facing, 0),
    });
}

fn push_sprite_frame_top_left(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    palette: &Palette,
    x: i32,
    y: i32,
) {
    let screen_x = x + i32::from(frame.origin_x);
    let screen_y = y + i32::from(frame.height_px) - i32::from(frame.origin_y);
    push_sprite_frame(commands, frame, palette, screen_x, screen_y);
}

fn push_eol_number_sprites(
    commands: &mut Vec<DrawCommand>,
    eol: &kid_render::EolScoreVisuals,
    x: i32,
    y: i32,
    value: u32,
) {
    let mut remaining = value.min(9_999_999);
    let mut started = false;
    for (slot, divisor) in [1_000_000, 100_000, 10_000, 1_000, 100, 10, 1]
        .into_iter()
        .enumerate()
    {
        let digit = remaining / divisor;
        remaining %= divisor;
        if digit == 0 && !started && slot < 6 {
            continue;
        }
        started = true;
        if let Some(frame) = eol.digits.get(digit as usize) {
            // ASM Object_EOL_Number loc_C4E2 spaces seven digit workers 12px apart.
            push_sprite_frame_top_left(commands, frame, &eol.palette, x + slot as i32 * 12, y);
        }
    }
}

fn push_eol_bonus_row(
    commands: &mut Vec<DrawCommand>,
    eol: &kid_render::EolScoreVisuals,
    label: &kid_render::SpriteFrame,
    y: i32,
    value: u32,
) {
    push_sprite_frame_top_left(commands, label, &eol.palette, 56, y);
    push_sprite_frame_top_left(commands, &eol.bonus, &eol.palette, 120, y);
    push_eol_number_sprites(commands, eol, 200, y - 2, value);
}

fn push_eol_score_overlay(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    eol: &kid_render::EolScoreVisuals,
) {
    // ASM loc_D55E emits these rows with Load_Object_EOL_Text and
    // Load_Object_EOL_Number. Native currently renders the settled row
    // positions only; the off-screen fly-in worker timer is not in trace state.
    let mut y = (FRAME_HEIGHT as i32) / 2 - 18;
    push_eol_bonus_row(
        commands,
        eol,
        &eol.time,
        y,
        kid_core::time_bonus_loc_d56e(trace.level_timer_seconds),
    );
    y += 14;
    if !trace.level_damage_taken {
        push_eol_bonus_row(commands, eol, &eol.no_hit, y, 5_000);
        y += 14;
    }
    if !trace.level_prize_collected {
        push_eol_bonus_row(commands, eol, &eol.no_prize, y, 5_000);
        y += 14;
    }
    if trace.level_path_bonus_amount > 0 {
        push_eol_bonus_row(commands, eol, &eol.path, y, trace.level_path_bonus_amount);
        y += 14;
    }
    if trace.level_speed_bonus_awarded {
        push_eol_bonus_row(commands, eol, &eol.speed, y, 10_000);
        y += 14;
        push_sprite_frame_top_left(commands, &eol.under, &eol.palette, 56, y);
        push_eol_number_sprites(
            commands,
            eol,
            128,
            y - 2,
            u32::from(trace.level_timer_seconds),
        );
        push_sprite_frame_top_left(commands, &eol.seconds, &eol.palette, 184, y);
        y += 14;
    }
    push_sprite_frame_top_left(commands, &eol.score, &eol.palette, 56, y);
    push_eol_number_sprites(commands, eol, 200, y - 2, trace.score);
}

// Forest water ripple table from ASM: BackgroundScroll_ForestWaterRippleData
// (kid.asm line 34310). 64 signed byte entries, max amplitude 8px.
const FOREST_RIPPLE: [i8; 64] = [
    0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5, 6, 7, 7, 7, 8, 8, 8, 8, 7, 7, 7, 6, 5, 4, 3, 2, 2, 2,
    1, 1, 1, 0, 0, 0, 0, 1, 2, 3, 4, 4, 4, 4, 5, 6, 7, 8, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 1,
];

pub(super) fn layered_background_shimmer_offset(
    frame_index: usize,
    row_index: usize,
    row_selector: u8,
    effect_start_row_selector: Option<u8>,
) -> i32 {
    if effect_start_row_selector.is_none_or(|start| row_selector < start) {
        return 0;
    }
    let time_phase = (frame_index / 4) & 0x3F;
    let table_idx = (time_phase + row_index) & 0x3F;
    i32::from(FOREST_RIPPLE[table_idx])
}

pub(super) fn background_tile_screen_position(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    x_tiles: u16,
    y_tiles: u16,
    camera_x_px: i32,
    camera_y_px: i32,
) -> (i32, i32) {
    let world_x_px = i32::from(x_tiles) * 8;
    let world_y_px = i32::from(y_tiles) * 8;
    // Plane B vertical scroll uses Camera_Y_pos >> 2 in the original game.
    let background_scroll_y_px = camera_y_px >> 2;
    let Some(profile) = level_visuals.level.background_scroll.as_ref() else {
        return (
            world_x_px.saturating_sub(camera_x_px),
            world_y_px.saturating_sub(background_scroll_y_px),
        );
    };

    let layer_speed = profile
        .layer_speeds
        .get(usize::from(y_tiles))
        .copied()
        .unwrap_or(0x10);
    (
        world_x_px.saturating_sub(background_scroll_x_px(trace, camera_x_px, layer_speed)),
        world_y_px.saturating_sub(background_scroll_y_px),
    )
}

pub(super) fn background_scroll_x_px(
    trace: &NativeTraceFrame,
    camera_x_px: i32,
    layer_speed: u8,
) -> i32 {
    match layer_speed {
        0x00 => (trace.frame_index as i32 / 2).rem_euclid(512),
        0x08 => (trace.frame_index as i32 / 4).rem_euclid(512),
        0x10 => 0,
        0x18..=0x88 if (layer_speed - 0x10).is_multiple_of(8) => {
            let step = i32::from((layer_speed - 0x10) / 8);
            camera_x_px.saturating_mul(step) / 16
        }
        _ => camera_x_px,
    }
}

pub(super) fn push_tile_quad(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    quad: &kid_render::TileQuad,
) {
    for (index, tile_ref) in quad.tiles.iter().copied().enumerate() {
        let tile_x = x + if index % 2 == 0 { 0 } else { 8 };
        let tile_y = y + if index < 2 { 0 } else { 8 };
        if let Some(tile) = resolve_ingame_vram_tile(level_visuals, tile_ref) {
            push_resolved_tile(commands, tile_x, tile_y, tile);
        }
    }
}

pub(super) fn push_level_layout_terrain(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let layout = &level_visuals.level.foreground_layout;

    let col_start = (camera_x_px / 16).max(0) as u16;
    let col_end =
        ((camera_x_px + FRAME_WIDTH as i32 + 15) / 16).min(i32::from(layout.width_cells)) as u16;
    let row_start = (camera_y_px / 16).max(0) as u16;
    let row_end =
        ((camera_y_px + FRAME_HEIGHT as i32 + 15) / 16).min(i32::from(layout.height_cells)) as u16;

    for row in row_start..row_end {
        for col in col_start..col_end {
            let skin = layout.skin_at(col, row);
            let Some(resolved) = level_visuals.level.block_mapping.resolve(skin, None) else {
                continue;
            };
            push_tile_quad(
                commands,
                level_visuals,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px,
                resolved.quad,
            );
        }
    }
}

pub(super) fn push_block_grid_overrides(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) {
    let Some(grid) = block_grid else { return };

    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;

    // Determine visible cell range (with 1-cell margin for partially visible cells)
    let col_start = (camera_x_px / 16).max(0) as u16;
    let col_end =
        ((camera_x_px + FRAME_WIDTH as i32 + 15) / 16).min(i32::from(grid.width_cells)) as u16;
    let row_start = (camera_y_px / 16).max(0) as u16;
    let row_end =
        ((camera_y_px + FRAME_HEIGHT as i32 + 15) / 16).min(i32::from(grid.height_cells)) as u16;

    for row in row_start..row_end {
        for col in col_start..col_end {
            let cell = grid.cell_at(col, row);
            if !cell.has_block {
                continue;
            }

            let Some(resolved) = level_visuals
                .level
                .block_mapping
                .resolve(cell.skin, Some(cell.block_type as u8))
            else {
                continue;
            };
            // ASM sub_1007A wobble: if the cell is in `block_bounces`,
            // offset the draw y_pos by the engine-supplied value so the
            // block visibly shakes up/down before settling.
            let y_offset = trace
                .block_bounces
                .iter()
                .find(|(cx, cy, _, _)| *cx == col && *cy == row)
                .map_or(0, |(_, _, _, y)| i32::from(*y));
            push_tile_quad(
                commands,
                level_visuals,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px + y_offset,
                resolved.quad,
            );
        }
    }
}

pub(super) fn background_colour(level_visuals: &NativeLevelVisuals) -> [u8; 3] {
    level_visuals
        .palettes
        .background
        .colours
        .first()
        .map_or(SKY_COLOUR, |colour| [colour.red, colour.green, colour.blue])
}

pub(super) fn object_rect(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    width: i32,
    height: i32,
    colour: [u8; 3],
) -> DrawCommand {
    DrawCommand::Rect {
        x: screen_coord(object.x_pos, trace.camera_x_pos),
        y: screen_coord(object.y_pos, trace.camera_y_pos),
        width,
        height,
        colour,
    }
}

pub(super) fn screen_coord(world: u32, camera: u32) -> i32 {
    let world = i64::from(world);
    let camera = i64::from(camera);
    ((world - camera) >> 16) as i32
}

fn build_game_scene_filtered(
    trace: &NativeTraceFrame,
    debug_overlay_visible: bool,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
    filter: LayerFilter,
) -> Scene {
    let background = background_colour(level_visuals);
    let mut commands = Vec::new();

    if filter.background {
        push_background_tiles(&mut commands, trace, level_visuals);
    }
    if filter.terrain {
        push_level_layout_terrain(&mut commands, trace, level_visuals);
    }
    if filter.block_overrides {
        push_block_grid_overrides(&mut commands, trace, level_visuals, block_grid);
    }

    if filter.platforms {
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Platform { .. }))
        {
            if let NativeObjectKind::Platform { kind, .. } = object.kind {
                push_platform_tiles(&mut commands, object, trace, level_visuals, kind);
            }
        }
    }
    if filter.enemies {
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Enemy { .. }))
        {
            if let NativeObjectKind::Enemy { object_type } = object.kind {
                push_enemy_sprite(&mut commands, object, trace, level_visuals, object_type);
                // Debug: show enemy HP above each enemy
                if debug_overlay_visible && let Some(hp) = object.hit_points {
                    let ex = (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
                    let ey = (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32 - 12;
                    commands.push(text(ex - 4, ey, &format!("{hp}"), [255, 255, 100], 1));
                }
            }
        }
    }
    if filter.player {
        // ASM `loc_BC34` damage-flash thread (kid.asm:15181-15215): during
        // the 66-frame `Just_received_damage` window, the ROM alternates
        // between the white palette (sub_80BC) for 2 frames and the normal
        // palette (sub_80D0) for 4 frames, repeating 11 times. Kid stays
        // visible the whole time. Native previously blinked Kid invisible
        // every 4 frames as a placeholder, which was visually distracting
        // and contradicted the ROM. Until the renderer wires the full
        // palette swap, draw a white overlay rect during the 2-frame white
        // phase so the player still sees a flash cue without disappearing.
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, kid_core::NativeObjectKind::Player))
        {
            // ASM loc_B7FA/loc_B80C: the death tumble is driven by the
            // engine's `advance_death_windup_motion` (combat.rs:8801).
            // The renderer must NOT add its own offset on top — that was
            // user-reported playtest bug #148 (incorrect dying animation).
            push_player_sprite(&mut commands, object, trace, level_visuals);

            if trace.invulnerable_frames > 0 {
                // PLAYER_DAMAGE_INVULNERABLE_FRAMES = 66; elapsed counts up
                // from 0 to 65 across the cycle. Every six-frame slot is
                // 2 frames of white + 4 frames of normal, matching ASM
                // `loc_BC56` (d2=1 → 2 hibernate frames white) and
                // `loc_BC6E` (d2=3 → 4 hibernate frames normal).
                let elapsed = 66usize.saturating_sub(trace.invulnerable_frames);
                let white_phase = (elapsed % 6) < 2;
                if white_phase {
                    let (sx, sy) = player_screen_position(object, trace);
                    commands.push(DrawCommand::Rect {
                        x: sx - 8,
                        y: sy - 24,
                        width: 16,
                        height: 24,
                        colour: [240, 240, 240],
                    });
                }
            }
        }
    }

    if filter.effects {
        if !trace.level_complete
            && let (Some(flag_x), Some(flag_y)) = (trace.flag_x, trace.flag_y)
        {
            // ASM Flagpole_Boss (kid.asm:15592) allocates exactly two
            // GfxObjects at the same anchor: `LnkTo_unk_E0FDE` (pole_bottom,
            // `sprite_frame_vram $696,$08,$10,$10,$10`) and `LnkTo_unk_E0FFE`
            // (waving flag, `sprite_frame_vram $69A,$06,$00,$28,$20`).
            // The ROM ships no third "pole shaft" sprite asset — the only
            // flag-related extracted art is `End_of_level_Flag_(6_frames).bin`
            // and `Flag_bottom.bin`. The previous 2×32 grey rect above the
            // anchor invented a tall pole extension that the ROM never draws.
            let sx = i32::from(flag_x) - (trace.camera_x_pos >> 16) as i32;
            let pole_base_y = i32::from(flag_y) - (trace.camera_y_pos >> 16) as i32;
            let frame_index = (trace.frame_index / level_visuals.flag.frame_duration)
                % level_visuals.flag.waving.len().max(1);
            if let Some(frame) = level_visuals.flag.waving.get(frame_index) {
                push_sprite_frame(
                    &mut commands,
                    frame,
                    &level_visuals.flag.palette,
                    sx,
                    pole_base_y,
                );
            }
            push_sprite_frame(
                &mut commands,
                &level_visuals.flag.pole_bottom,
                &level_visuals.flag.palette,
                sx,
                pole_base_y,
            );
        }

        // Murder wall placeholder. ASM `sub_5E02` (kid.asm:5840-5965) emits
        // the wall as 24 hardware sprites in a 4×6 grid using compressed
        // tile art at `$13AA4..$1408A` (`Murder_wall.bin`) with the dedicated
        // `$1408A..$1409A` palette (`Murder_wall.bin` palette entry), animated
        // through a 4-frame cycle on timer `$FFFFFAC0` / `$FFFFFABF`. The
        // tile/palette wiring is not yet wired into `level_visuals` — until
        // it is, render a single solid rect at the wall's leading edge so
        // the wall remains visible without inventing a multi-band red
        // gradient that the ROM does not draw.
        if let Some(wall_x_fp) = trace.murder_wall_x {
            let wall_x_screen = (wall_x_fp >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            commands.push(DrawCommand::Rect {
                x: wall_x_screen - 64,
                y: 0,
                width: 64,
                height: FRAME_HEIGHT as i32,
                colour: [200, 30, 10],
            });
        }

        // Lava geyser placeholder. ASM `sub_14BD8` -> `loc_14BE8`
        // (kid.asm:28255-28278) emits per-row sprite slots driven by 8-byte
        // template entries at `unk_14624`, `dword_14636`, `unk_14650`,
        // `unk_1465A`, `unk_14664`, `unk_1466E`. The companion helper
        // `lava_geyser_sprite_chunks_loc_14BE8` in kid-core already implements
        // the row walk but is marked `#[allow(dead_code)]` because the
        // template tables are not yet extracted from ROM. Until the
        // templates are wired, render a single warm-orange column at the
        // geyser's position so the eruption is visible without inventing
        // the previous "core + hot centre" two-band gradient that the ROM
        // does not draw.
        for &(gx, gy_top, height) in &trace.active_geysers {
            let sx = i32::from(gx) - (trace.camera_x_pos >> 16) as i32;
            let sy = i32::from(gy_top) - (trace.camera_y_pos >> 16) as i32;
            commands.push(DrawCommand::Rect {
                x: sx - 4,
                y: sy,
                width: 8,
                height: i32::from(height),
                colour: [255, 120, 20],
            });
        }

        // Projectiles: generic shots stay coloured dots; the Fire Demon
        // death trail renders ROM sprite frames via
        // `push_fire_demon_trail_sprite` (ASM Fire_Demon `loc_3A63C` cycling
        // `stru_3A622` -> `unk_C7776..unk_C779E`). Missing-asset states fall
        // back to a single placeholder rect in `push_fire_demon_trail`.
        let projectile_details: Vec<ActiveProjectileSnapshot> =
            if trace.active_projectile_details.is_empty() {
                trace
                    .active_projectiles
                    .iter()
                    .map(|&(x_pos, y_pos, friendly)| ActiveProjectileSnapshot {
                        x_pos,
                        y_pos,
                        friendly,
                        remaining_frames: 0,
                        kind: ActiveProjectileKind::Generic,
                    })
                    .collect()
            } else {
                trace.active_projectile_details.clone()
            };
        for projectile in projectile_details {
            let sx = (projectile.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let sy = (projectile.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            match projectile.kind {
                ActiveProjectileKind::Generic => commands.push(DrawCommand::Rect {
                    x: sx - 2,
                    y: sy - 2,
                    width: 4,
                    height: 4,
                    colour: if projectile.friendly {
                        FRIENDLY_PROJECTILE_COLOUR
                    } else {
                        ENEMY_PROJECTILE_COLOUR
                    },
                }),
                ActiveProjectileKind::FireDemonDeathTrail => {
                    if !push_fire_demon_trail_sprite(
                        &mut commands,
                        level_visuals,
                        sx,
                        sy,
                        projectile.remaining_frames,
                    ) {
                        push_fire_demon_trail(&mut commands, sx, sy, projectile.remaining_frames);
                    }
                }
                ActiveProjectileKind::DripsDrop => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&DRIPS_OBJECT_TYPE)
                        .and_then(|drips_visuals| {
                            let frame = drips_drop_child_frame(drips_visuals)?;
                            push_sprite_frame(
                                &mut commands,
                                &frame,
                                &drips_visuals.palette,
                                sx,
                                sy,
                            );
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 2,
                            y: sy - 2,
                            width: 4,
                            height: 5,
                            colour: [120, 200, 90],
                        });
                    }
                }
            }
        }

        // ASM sub_1007A bounce coroutine: the block itself moves with the
        // engine-supplied y_offset (replaces the previous grey-rect
        // overlay). The level-pass `push_block_grid_overrides` is
        // responsible for drawing the actual block art at the offset
        // position; this loop is intentionally empty so the original cell
        // is not double-drawn at the un-offset position.
        for &(_cx, _cy, _remaining, _y_offset) in &trace.block_bounces {
            // Block-pass handles the offset draw; nothing to do here.
        }

        // ASM `sub_292E` (kid.asm:4881) emits five direction-keyed debris
        // workers, and `sub_2A4C` (kid.asm:5025) draws each with tile `$8228`.
        // Keep the old centre marker only as an asset-missing fallback.
        for &(x_px, y_px, _remaining, direction) in &trace.block_destroy_effects {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            if push_block_destroy_debris_sub_292e(
                &mut commands,
                level_visuals,
                x_px,
                y_px,
                direction,
                camera_x_px,
                camera_y_px,
            ) {
                continue;
            }

            let cx = i32::from(x_px) + 8 - camera_x_px;
            let cy = i32::from(y_px) + 8 - camera_y_px;
            commands.push(DrawCommand::Rect {
                x: cx - 3,
                y: cy - 3,
                width: 6,
                height: 6,
                colour: [180, 150, 110],
            });
        }

        // Floating prize items: small coloured shape rising upward
        for &(x_px, y_px, remaining, content_id) in &trace.floating_items {
            let fx = i32::from(x_px) - (trace.camera_x_pos >> 16) as i32;
            let fy = i32::from(y_px) - (trace.camera_y_pos >> 16) as i32;
            let content = kid_core::terrain::PrizeContent::from_id(content_id);
            if render_floating_prize_worker_effect(
                &mut commands,
                level_visuals,
                content,
                fx,
                fy,
                remaining,
            ) {
                continue;
            }
            let graphics_spec = floating_item_graphics_spec(content_id);
            if let Some(prize_visuals) = floating_prize_visuals(level_visuals, content_id) {
                let frame_index = (trace.frame_index / prize_visuals.frame_duration.max(1))
                    % prize_visuals.frames.len().max(1);
                if let Some(frame) = prize_visuals.frames.get(frame_index) {
                    let facing = Direction::Right;
                    let (draw_x, draw_y) = sprite_draw_position(frame, fx, fy, facing);
                    commands.push(DrawCommand::Sprite {
                        x: draw_x,
                        y: draw_y,
                        tile_sheet: Arc::new(frame.tile_sheet.clone()),
                        palette: Arc::new(prize_visuals.palette.clone()),
                        tiles: build_sprite_tiles(frame, facing, 0),
                    });
                    continue;
                }
            }

            let alpha = u32::from(remaining).saturating_mul(8).min(255);
            let base_colour = floating_item_colour(content, graphics_spec);
            let colour = [
                (u32::from(base_colour[0]) * alpha / 255) as u8,
                (u32::from(base_colour[1]) * alpha / 255) as u8,
                (u32::from(base_colour[2]) * alpha / 255) as u8,
            ];
            // Degraded-asset fallback only: in production, Diamond / Ankh /
            // Clock / Continue / helmet floating prizes resolve through
            // `floating_prize_visuals` to real ROM sprite frames (ASM
            // PrizeDiamond_Init `stru_10D6E`, sub_10CB0 helmet selectors,
            // `stru_10C08/10C18/10C28` icon tiles), and Points10000 /
            // TenDiamonds render through `render_floating_prize_worker_effect`
            // (ASM Prize10000Points_Init / Prize10Diamonds_Init). The
            // previous Diamond / Ankh / Clock branches drew hand-invented
            // pixel-art "diamond", "cross", and "clock face" approximations
            // with no ROM citation — those shapes never appeared in the
            // original game. Collapse the fallback to one consistent
            // filled-square indicator so missing-asset states are visible
            // without inventing per-prize artwork.
            commands.push(DrawCommand::Rect {
                x: fx - 3,
                y: fy - 3,
                width: 6,
                height: 6,
                colour,
            });
            commands.push(DrawCommand::Rect {
                x: fx - 2,
                y: fy - 2,
                width: 2,
                height: 1,
                colour: [255, 255, 255],
            });
        }

        // Enemy death effect: only HeadyMetal spawns one. ASM
        // Enemy20_HeadyMetal_Init seeds the $01C2-frame wobble worker
        // (sub_37708/sub_3787C/sub_378A2); `Jump_On_Enemy` (kid.asm:14607)
        // plays sfx_Jump_on_enemy and deletes the GfxObject without
        // emitting any per-stomp death sprite.
        for &(ex, ey, remaining) in &trace.enemy_death_effects {
            push_heady_metal_death_wobble(&mut commands, trace, level_visuals, ex, ey, remaining);
        }
    }

    if filter.hud {
        push_game_hud(&mut commands, trace, level_visuals);
    }

    // Level Complete overlay. ASM `loc_D55E` onwards (kid.asm:16670-16800)
    // emits sprite-art labels — `LnkTo_Spr_EOL_Score`,
    // `LnkTo_Spr_EOL_Time`, `LnkTo_Spr_EOL_Bonus`, `LnkTo_Spr_EOL_NoHit`,
    // `LnkTo_Spr_EOL_NoPrize`, `LnkTo_Spr_EOL_Path`, `LnkTo_Spr_EOL_Speed`
    // — plus seven digit workers from `Load_Object_EOL_Number`. Native draws
    // the settled row positions when the EOL score sheet is available; the
    // off-screen fly-in timing is not in `NativeTraceFrame` yet.
    if trace.level_complete {
        if let Some(eol_score) = &level_visuals.eol_score {
            push_eol_score_overlay(&mut commands, trace, eol_score);
            return Scene {
                background,
                commands,
            };
        }
        let text_y = (FRAME_HEIGHT as i32) / 2 - 40;
        let mut line_y = text_y + 22;
        let time_bonus = kid_core::time_bonus_loc_d56e(trace.level_timer_seconds);
        let bonus_line = format!("TIME BONUS: {time_bonus}");
        let bonus_x = (FRAME_WIDTH as i32 - bonus_line.len() as i32 * 8) / 2;
        commands.push(text(bonus_x, line_y, &bonus_line, [200, 255, 200], 1));
        line_y += 12;
        // ASM loc_D5BC (kid.asm:16715): NO-HIT bonus awarded when
        // NoHit_Bonus_Flag clear, i.e. no damage taken during the level.
        if !trace.level_damage_taken {
            let nohit = "NO-HIT BONUS: 5000";
            let nohit_x = (FRAME_WIDTH as i32 - nohit.len() as i32 * 8) / 2;
            commands.push(text(nohit_x, line_y, nohit, [255, 255, 100], 1));
            line_y += 12;
        }
        // ASM loc_D5D2 (kid.asm:16732): NO-PRIZE bonus awarded when
        // NoPrize_Bonus_Flag clear, i.e. no prize-block was struck during
        // the level. The previous overlay omitted this 5000-point bonus
        // even though the engine awards it through `add_score(5000)`, so
        // displayed totals lagged the score by 5000 on no-prize runs.
        if !trace.level_prize_collected {
            let noprize = "NO-PRIZE BONUS: 5000";
            let noprize_x = (FRAME_WIDTH as i32 - noprize.len() as i32 * 8) / 2;
            commands.push(text(noprize_x, line_y, noprize, [255, 200, 255], 1));
            line_y += 12;
        }
        // ASM loc_D60E (kid.asm:16750): PATH BONUS = `PathBonus_Values[level]
        // × 1000`. Only awarded on non-Elsewhere levels with a non-zero
        // table entry; the renderer reflects the engine's awarded amount
        // directly rather than recomputing the per-level table.
        if trace.level_path_bonus_amount > 0 {
            let path_line = format!("PATH BONUS: {}", trace.level_path_bonus_amount);
            let path_x = (FRAME_WIDTH as i32 - path_line.len() as i32 * 8) / 2;
            commands.push(text(path_x, line_y, &path_line, [200, 200, 255], 1));
            line_y += 12;
        }
        // ASM loc_D660 (kid.asm:16775): SPEED BONUS = 10,000 points,
        // awarded only when `Level_completion_time <= SpeedBonus_Values[level]`
        // (per-level threshold table, accounting for clocks collected).
        // The previous overlay used a hand-tuned `elapsed < 60` gate that
        // did not match the engine's per-level condition; surface the
        // engine-driven `level_speed_bonus_awarded` flag instead.
        if trace.level_speed_bonus_awarded {
            let speed = "SPEED BONUS: 10000";
            let speed_x = (FRAME_WIDTH as i32 - speed.len() as i32 * 8) / 2;
            commands.push(text(speed_x, line_y, speed, [100, 200, 255], 1));
            line_y += 12;
        }
        let score_line = format!("SCORE: {}", trace.score);
        let score_x = (FRAME_WIDTH as i32 - score_line.len() as i32 * 8) / 2;
        commands.push(text(score_x, line_y, &score_line, [255, 255, 255], 1));
    }

    // Game Over / Continue overlay
    if trace.game_over {
        if let Some(countdown) = trace.continue_countdown {
            // "CONTINUE?" countdown screen — shows remaining time and continues
            let title = "CONTINUE?";
            let title_w = title.len() as i32 * 8 * 2;
            let title_x = (FRAME_WIDTH as i32 - title_w) / 2;
            let title_y = (FRAME_HEIGHT as i32) / 2 - 24;
            let seconds_left = (countdown / 60).min(9);
            let timer_line = format!("{seconds_left}");
            let timer_w = timer_line.len() as i32 * 8 * 3;
            let timer_x = (FRAME_WIDTH as i32 - timer_w) / 2;
            let count_line = format!("{} CONTINUES LEFT", trace.number_continues);
            let count_w = count_line.len() as i32 * 8;
            let count_x = (FRAME_WIDTH as i32 - count_w) / 2;
            let panel_w = title_w.max(count_w + 16) + 16;
            let panel_x = (FRAME_WIDTH as i32 - panel_w) / 2;
            push_panel(
                &mut commands,
                panel_x,
                title_y - 4,
                panel_w,
                56,
                [0, 40, 80],
                [80, 180, 255],
            );
            commands.push(text(title_x, title_y, title, [80, 180, 255], 2));
            // Blinking countdown number
            if (countdown / 15) % 2 == 0 {
                commands.push(text(timer_x, title_y + 20, &timer_line, [255, 255, 80], 3));
            }
            commands.push(text(count_x, title_y + 44, &count_line, [200, 220, 255], 1));
        } else if trace.number_continues > 0 {
            // "GAME OVER" display before continue screen appears
            let go_text = "GAME OVER";
            let text_w = go_text.len() as i32 * 8 * 2;
            let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
            let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
            push_panel(
                &mut commands,
                text_x - 8,
                text_y - 4,
                text_w + 16,
                24,
                [80, 0, 0],
                [255, 80, 80],
            );
            commands.push(text(text_x, text_y, go_text, [255, 80, 80], 2));
        } else {
            // No continues — final "GAME OVER"
            let go_text = "GAME OVER";
            let text_w = go_text.len() as i32 * 8 * 2;
            let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
            let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
            push_panel(
                &mut commands,
                text_x - 8,
                text_y - 4,
                text_w + 16,
                24,
                [80, 0, 0],
                [255, 80, 80],
            );
            commands.push(text(text_x, text_y, go_text, [255, 80, 80], 2));
        }
    }

    // Pause overlay
    if trace.paused {
        let pause_text = "PAUSED";
        let text_w = pause_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 16;
        push_panel(
            &mut commands,
            text_x - 12,
            text_y - 6,
            text_w + 24,
            44,
            [0, 0, 0],
            [255, 255, 255],
        );
        commands.push(text(text_x, text_y, pause_text, [255, 255, 255], 2));
        let hint = "Press Start to resume";
        let hint_w = hint.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - hint_w) / 2,
            text_y + 24,
            hint,
            [140, 140, 140],
            1,
        ));
    }

    // Level complete overlay
    if trace.level_complete {
        let complete_text = "LEVEL CLEAR!";
        let text_w = complete_text.len() as i32 * 8 * 2;
        let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
        let text_y = (FRAME_HEIGHT as i32) / 2 - 32;
        push_panel(
            &mut commands,
            text_x - 16,
            text_y - 8,
            text_w + 32,
            80,
            [0, 20, 40],
            [80, 200, 255],
        );
        commands.push(text(text_x, text_y, complete_text, [80, 255, 120], 2));
        let time_text = format!(
            "TIME BONUS: {}s x 100 = {}",
            trace.level_timer_seconds,
            u32::from(trace.level_timer_seconds) * 100
        );
        commands.push(text(
            text_x - 8,
            text_y + 22,
            &time_text,
            [200, 200, 160],
            1,
        ));
        let score_text = format!("TOTAL SCORE: {}", trace.score);
        let score_w = score_text.len() as i32 * 8;
        commands.push(text(
            (FRAME_WIDTH as i32 - score_w) / 2,
            text_y + 38,
            &score_text,
            [255, 255, 200],
            1,
        ));
    }

    if debug_overlay_visible {
        push_panel(&mut commands, 8, 8, 304, 56, [12, 18, 24], [160, 214, 255]);
        commands.push(text(
            16,
            16,
            &format!(
                "{}  Frame {}  Pause {}",
                game_mode_label(trace.game_mode),
                trace.frame_index,
                on_off(trace.paused)
            ),
            [255, 240, 196],
            1,
        ));
        commands.push(text(
            16,
            28,
            &format!(
                "Level {:02X} {}  Lives {}  HP {}  {}",
                trace.current_level.0,
                trace.current_level_name.as_deref().unwrap_or(""),
                trace.number_lives,
                trace.number_hitpoints,
                form_label(trace.current_helmet_id)
            ),
            [220, 220, 220],
            1,
        ));
        commands.push(text(
            16,
            40,
            &format!(
                "Score {}  Diamonds {}  Timer {}s  Projectiles {}",
                trace.score,
                trace.diamond_count,
                trace.level_timer_seconds,
                trace.active_projectiles.len()
            ),
            [220, 220, 190],
            1,
        ));
        commands.push(text(
            16,
            52,
            &format!(
                "Camera {:08X},{:08X}  Objects {}/{}",
                trace.camera_x_pos,
                trace.camera_y_pos,
                trace.number_objects,
                trace.number_gfx_objects
            ),
            [190, 190, 190],
            1,
        ));

        // Collision boxes: player (green), enemies (red), platforms (blue)
        let cam_x = (trace.camera_x_pos >> 16) as i32;
        let cam_y = (trace.camera_y_pos >> 16) as i32;
        for object in &trace.active_objects {
            let ox = (object.x_pos >> 16) as i32 - cam_x;
            let oy = (object.y_pos >> 16) as i32 - cam_y;
            match object.kind {
                NativeObjectKind::Player => {
                    // Player hitbox: PLAYER_HALF_WIDTH=5, PLAYER_HEIGHT=31
                    push_collision_box(&mut commands, ox - 5, oy - 31, 10, 31, [0, 255, 0]);
                }
                NativeObjectKind::Enemy { .. } => {
                    // Enemy contact box: 8x16 centred on position
                    push_collision_box(&mut commands, ox - 4, oy - 8, 8, 16, [255, 60, 60]);
                }
                NativeObjectKind::Platform { .. } => {
                    // Platform: 32x8 centred on position
                    push_collision_box(&mut commands, ox - 16, oy - 4, 32, 8, [60, 120, 255]);
                }
            }
        }
    }

    Scene {
        background,
        commands,
    }
}

fn render_floating_prize_worker_effect(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    content: kid_core::terrain::PrizeContent,
    screen_x: i32,
    screen_y: i32,
    remaining_low_byte: u8,
) -> bool {
    if remaining_low_byte > PRIZE_GENERIC_LIFETIME_LOW_BYTE {
        return false;
    }
    let effect_age = PRIZE_GENERIC_LIFETIME_LOW_BYTE.wrapping_sub(remaining_low_byte);
    match content {
        kid_core::terrain::PrizeContent::TenDiamonds => render_ten_diamonds_worker_effect(
            commands,
            level_visuals,
            screen_x,
            screen_y,
            effect_age,
        ),
        kid_core::terrain::PrizeContent::Points10000 => render_10000_points_worker_effect(
            commands,
            level_visuals,
            screen_x,
            screen_y,
            effect_age,
        ),
        _ => false,
    }
}

fn render_ten_diamonds_worker_effect(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    screen_x: i32,
    screen_y: i32,
    effect_age: u8,
) -> bool {
    // ASM Prize10Diamonds_Init loc_108EE uses byte_108AC to spawn ten
    // workers on frames 0..3 and 24..29. Each worker uses unk_E1046:
    // sprite_frame_vram $6ED,$02,$FE,$08,$08 over the HUD diamond DMA page.
    const SPAWNS: &[(u8, i32, i32)] = &[
        (0, -20, -36),
        (1, -20, -27),
        (2, -20, -18),
        (3, -20, -9),
        (24, 6, -36),
        (25, -4, -28),
        (26, 16, -28),
        (27, -4, -16),
        (28, 16, -16),
        (29, 6, -8),
    ];

    let frame = kid_render::SpriteFrame {
        origin_x: 0x02,
        origin_y: -2,
        width_px: 0x08,
        height_px: 0x08,
        tile_base_index: 0x06ED - level_visuals.hud.diamond_base_tile_index,
        tile_sheet: level_visuals.hud.diamond_tiles.clone(),
    };
    let mut drew_worker = false;
    for &(spawn_frame, x_velocity_word, y_velocity_word) in SPAWNS {
        if effect_age < spawn_frame {
            continue;
        }
        let worker_age = i32::from(effect_age - spawn_frame);
        let worker_x = screen_x + ((x_velocity_word * worker_age) >> 3);
        let worker_y = screen_y + ((y_velocity_word * worker_age) >> 3);
        push_worker_sprite(
            commands,
            &frame,
            &level_visuals.palettes.foreground,
            worker_x,
            worker_y,
        );
        drew_worker = true;
    }
    drew_worker
}

fn render_10000_points_worker_effect(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    screen_x: i32,
    screen_y: i32,
    effect_age: u8,
) -> bool {
    // ASM Prize10000Points_Init loc_10B00 uses unk_10A7A to spawn five workers
    // on frames 0,5,10,15,20. Frame 0 uses unk_E1056 (tile $6BB); later
    // workers use unk_E104E (tile $6BA), both 8x8 over HUD number tiles.
    const SPAWNS: &[u8] = &[0, 5, 10, 15, 20];
    let mut drew_worker = false;
    for &spawn_frame in SPAWNS {
        if effect_age < spawn_frame {
            continue;
        }
        let worker_age = i32::from(effect_age - spawn_frame);
        let tile_base_index = if spawn_frame == 0 { 1 } else { 0 };
        let frame = kid_render::SpriteFrame {
            origin_x: 0x04,
            origin_y: -4,
            width_px: 0x08,
            height_px: 0x08,
            tile_base_index,
            tile_sheet: level_visuals.hud.digits.clone(),
        };
        let worker_x = screen_x - 6 * worker_age.saturating_sub(0x58);
        push_worker_sprite(
            commands,
            &frame,
            &level_visuals.palettes.foreground,
            worker_x,
            screen_y,
        );
        drew_worker = true;
    }
    drew_worker
}

fn push_worker_sprite(
    commands: &mut Vec<DrawCommand>,
    frame: &kid_render::SpriteFrame,
    palette: &Palette,
    screen_x: i32,
    screen_y: i32,
) {
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, Direction::Right);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette.clone()),
        tiles: build_sprite_tiles(frame, Direction::Right, 0),
    });
}

fn push_heady_metal_death_wobble(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    x_pos: u32,
    y_pos: u32,
    remaining: u16,
) {
    let screen_x = (x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
    let screen_y = (y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
    // ASM sub_37708/sub_3787C/sub_378A2 applies wobble velocities
    // -$30D40/$30D40, -$493E0/$493E0, -$61A80/$61A80, -$7A120/$7A120,
    // and -$927C0/$927C0 as the countdown crosses $015E/$00FA/$0096/$0064.
    // Use those stage magnitudes directly instead of the earlier hand-tuned
    // 2..6px approximation so the render path follows the ROM's quiver tiers.
    let amplitude = match remaining {
        r if r > HEADYMETAL_WOBBLE_STAGE_1 => 0x30D40 >> 16,
        r if r > HEADYMETAL_WOBBLE_STAGE_2 => 0x493E0 >> 16,
        r if r > HEADYMETAL_WOBBLE_STAGE_3 => 0x61A80 >> 16,
        r if r > HEADYMETAL_WOBBLE_STAGE_4 => 0x7A120 >> 16,
        _ => 0x927C0 >> 16,
    };
    let phase = if (trace.frame_index & 0x2) == 0 {
        -1
    } else {
        1
    };
    let horizontal = amplitude * phase;
    let vertical = amplitude * -phase;

    push_effect_enemy_sprite(
        commands,
        level_visuals,
        0x20,
        screen_x,
        screen_y + vertical,
        Direction::Right,
        true,
    );
    push_effect_enemy_sprite(
        commands,
        level_visuals,
        0x21,
        screen_x - 12 + horizontal,
        screen_y - 18,
        Direction::Right,
        false,
    );
    push_effect_enemy_sprite(
        commands,
        level_visuals,
        0x21,
        screen_x + 12 - horizontal,
        screen_y - 18,
        Direction::Left,
        false,
    );
}

fn push_effect_enemy_sprite(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    object_type: u8,
    screen_x: i32,
    screen_y: i32,
    facing: Direction,
    _stationary: bool,
) {
    let Some(enemy_visuals) = level_visuals.enemies.families.get(&object_type) else {
        return;
    };
    let frame = &enemy_visuals.frames[0];
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(enemy_visuals.palette.clone()),
        tiles: build_sprite_tiles(frame, facing, 0),
    });
}

/// Draw a 1px outline rectangle for collision box visualisation.
fn push_collision_box(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    colour: [u8; 3],
) {
    // Top edge
    commands.push(DrawCommand::Rect {
        x,
        y,
        width: w,
        height: 1,
        colour,
    });
    // Bottom edge
    commands.push(DrawCommand::Rect {
        x,
        y: y + h - 1,
        width: w,
        height: 1,
        colour,
    });
    // Left edge
    commands.push(DrawCommand::Rect {
        x,
        y,
        width: 1,
        height: h,
        colour,
    });
    // Right edge
    commands.push(DrawCommand::Rect {
        x: x + w - 1,
        y,
        width: 1,
        height: h,
        colour,
    });
}

pub(super) fn floating_item_graphics_spec(
    content_id: u8,
) -> Option<kid_core::FloatingPrizeGraphicsSpec> {
    kid_core::floating_prize_graphics_spec(kid_core::terrain::PrizeContent::from_id(content_id))
}

fn floating_prize_visuals(
    level_visuals: &NativeLevelVisuals,
    content_id: u8,
) -> Option<&kid_render::AnimatedSpriteVisual> {
    match kid_core::terrain::PrizeContent::from_id(content_id) {
        // ASM PrizeDiamond_Init: `stru_10D6E` selects `unk_E0E66`, a VRAM
        // sprite frame over the HUD diamond tiles.
        kid_core::terrain::PrizeContent::Diamond => Some(&level_visuals.floating_prizes.diamond),
        // ASM sub_10CB0 followed by PrizeHelmet*_Init selects the mask frame
        // and helmet palette for content ids 2..A.
        kid_core::terrain::PrizeContent::Skycutter
        | kid_core::terrain::PrizeContent::Cyclone
        | kid_core::terrain::PrizeContent::RedStealth
        | kid_core::terrain::PrizeContent::Eyeclops
        | kid_core::terrain::PrizeContent::Juggernaut
        | kid_core::terrain::PrizeContent::IronKnight
        | kid_core::terrain::PrizeContent::Berzerker
        | kid_core::terrain::PrizeContent::Maniaxe
        | kid_core::terrain::PrizeContent::Micromax => {
            level_visuals.floating_prizes.helmets.get(&content_id)
        }
        // ASM PrizeAnkh_Init/PrizeClock_Init/PrizeContinue_Init select
        // `stru_10C08`, `stru_10C18`, and `stru_10C28` over ROM icon tiles.
        kid_core::terrain::PrizeContent::Ankh
        | kid_core::terrain::PrizeContent::Clock
        | kid_core::terrain::PrizeContent::Continue => {
            level_visuals.floating_prizes.icons.get(&content_id)
        }
        _ => None,
    }
}

/// Returns a distinct RGB colour for each prize content type.
fn floating_item_colour(
    content: kid_core::terrain::PrizeContent,
    graphics_spec: Option<kid_core::FloatingPrizeGraphicsSpec>,
) -> [u8; 3] {
    match (content, graphics_spec) {
        // ASM PrizeDiamond_Init: palette line 0, `stru_10D6E`.
        (kid_core::terrain::PrizeContent::Diamond, Some(_)) => [100, 200, 255],
        // ASM sub_10CB0: all helmet prizes use palette line 3; use the
        // object metadata to keep placeholder helmets visually distinct until
        // the real sprite art is wired.
        (_, Some(spec)) if spec.palette_line == 3 => helmet_placeholder_colour(spec.object_meta),
        (kid_core::terrain::PrizeContent::Points10000, _) => [255, 255, 100],
        (kid_core::terrain::PrizeContent::Ankh, _) => [255, 255, 200],
        (kid_core::terrain::PrizeContent::Clock, _) => [200, 255, 200],
        (kid_core::terrain::PrizeContent::Continue, _) => [255, 255, 255],
        (kid_core::terrain::PrizeContent::TenDiamonds, _) => [100, 200, 255],
        _ => [255, 255, 100],
    }
}

fn helmet_placeholder_colour(_object_meta: u16) -> [u8; 3] {
    // ASM sub_10CB0 places every helmet prize on palette line 3 with the
    // helmet's own `Pal_*` block (e.g. `Pal_DD7A1` for Berzerker). The
    // previous nine hand-picked RGB triples — "Skycutter pale blue",
    // "Cyclone mint", "RedStealth pink", etc. — were guessed palette
    // colours with no ROM citation. Until each helmet's ROM palette
    // flows through `level_visuals.floating_prizes.helmets`, render a
    // single consistent placeholder so missing-asset states do not
    // claim per-helmet hues that the ROM does not specify.
    [220, 220, 255]
}
