use crate::background::{
    BackgroundChunk, BackgroundVisuals, ChunkedBackgroundVisuals, LayeredBackgroundPlane,
    compose_layered_background, parse_background_chunk, parse_chunked_background_layout,
    parse_layered_background_layout, parse_layered_background_plane,
};
use crate::hud::{HudIcon, HudVisuals};
use crate::level::{BlockVisualMapping, ForegroundLayout, LevelVisuals};
use crate::palette::{MdColour, Palette, PaletteSet};
use crate::player::{
    AnimatedSpriteVisual, EnemyVisuals, EolHologramVisuals, EolScoreVisuals, FlagVisuals,
    FloatingPrizeVisuals, LavaVisuals, MurderWallVisuals, PlatformVisuals, PlayerFormVisuals,
    PlayerTransformStep, PlayerVisuals, SpriteFrame,
};
use crate::tiles::{Tile, TileRef, TileSheet, parse_tile_quads};
use anyhow::{Context, Result, anyhow, bail};
use kid_data::{
    AssetFile, AssetManifest, DecodedAssetFile, EnemyLayout, decode_enigma, decode_saxman,
    ensure_generated_bundle_contract, load_level_asset_bundle,
};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

const HUD_DIGIT_BASE_TILE_INDEX: u16 = 0x06BA;
const HUD_DIAMOND_BASE_TILE_INDEX: u16 = 0x06EA;
const BLOCKS_ART_PATH: &str = "artcomp/Blocks.bin";
const HUD_NUMBERS_ART_PATH: &str = "artcomp/HUD_numbers.bin";
const HUD_DIAMOND_ART_PATH: &str = "artunc/diamond_1.bin";
const HUD_HITPOINT_ART_PATH: &str = "artunc/Hitpoint_display.bin";
const FLOATING_DIAMOND_ART_PATHS: [&str; 4] = [
    "artunc/diamond_1.bin",
    "artunc/diamond_2.bin",
    "artunc/diamond_3.bin",
    "artunc/diamond_4.bin",
];
const FOREGROUND_TILE_VRAM_BASE: u16 = 0x00BC;
const BLOCK_TILE_VRAM_BASE: u16 = 0x0220;
const FLAG_ART_PATH: &str = "artunc/End_of_level_Flag_(6_frames).bin";
const FLAG_BOTTOM_ART_PATH: &str = "artunc/Flag_bottom.bin";
const MURDER_WALL_ART_PATH: &str = "artcomp/Murder_wall.bin";
const MURDER_WALL_PALETTE_PATH: &str = "palette/Murder_wall.bin";
const MURDER_WALL_VRAM_BASE_TILE_INDEX: u16 = 0x02FB;
const LAVA_ART_PATH: &str = "artcomp/Lava.bin";
const LAVA_PALETTE_PATH: &str = "palette/lava.bin";
const LAVA_VRAM_BASE_TILE_INDEX: u16 = 0x02FB;
const EOL_HOLOGRAM_BACKGROUND_ART_PATH: &str = "artcomp/Hologram_background.bin";
const EOL_HOLOGRAM_BLOCKS_ART_PATH: &str = "artcomp/Hologram_blocks.bin";
const EOL_HOLOGRAM_TILEMAP_PATH: &str = "mapeni/hologram_background.bin";
const EOL_HOLOGRAM_PALETTE_PATH: &str = "palette/Score_screen.bin";
const EOL_HOLOGRAM_BACKGROUND_VRAM_BASE_TILE_INDEX: u16 = 0x02FB;
const EOL_HOLOGRAM_BLOCKS_VRAM_BASE_TILE_INDEX: u16 = 0x0409;
// Enemy art/palette paths removed — now defined inline in ENEMY_FAMILIES table.
const HORIZONTAL_PLATFORM_ART_PATH: &str = "artcomp/Horizontal_platform.bin";
const VERTICAL_PLATFORM_ART_PATH: &str = "artcomp/Vertical_platform.bin";
const KID_PALETTE_PATH: &str = "palette_kid/Kid.bin";
const SKYCUTTER_PALETTE_PATH: &str = "palette_kid/Skycutter.bin";
const SKYCUTTER_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Skycutter_Transformation.bin";
const CYCLONE_PALETTE_PATH: &str = "palette_kid/Cyclone.bin";
const CYCLONE_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Cyclone_Transformation.bin";
const RED_STEALTH_PALETTE_PATH: &str = "palette_kid/Red_Stealth.bin";
const RED_STEALTH_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Red_Stealth_Transformation.bin";
const EYECLOPS_PALETTE_PATH: &str = "palette_kid/Eyeclops.bin";
const EYECLOPS_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Eyeclops_Transformation.bin";
const JUGGERNAUT_PALETTE_PATH: &str = "palette_kid/Juggernaut.bin";
const JUGGERNAUT_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Juggernaut_Transformation.bin";
const IRON_KNIGHT_PALETTE_PATH: &str = "palette_kid/Iron_Knight.bin";
const IRON_KNIGHT_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Iron_Knight_Transformation.bin";
const BERZERKER_PALETTE_PATH: &str = "palette_kid/Berzerker.bin";
const BERZERKER_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Berzerker_Transformation.bin";
const MANIAXE_PALETTE_PATH: &str = "palette_kid/Maniaxe.bin";
const MANIAXE_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Maniaxe_Transformation.bin";
const MICROMAX_PALETTE_PATH: &str = "palette_kid/Micromax.bin";
const MICROMAX_TRANSFORM_PALETTE_PATH: &str = "palette_kid/Micromax_Transformation.bin";
const SKYCUTTER_HELMET_PALETTE_PATH: &str = "palette_kid/Skycutter_Helmet.bin";
const CYCLONE_HELMET_PALETTE_PATH: &str = "palette_kid/Cyclone_Helmet.bin";
const RED_STEALTH_HELMET_PALETTE_PATH: &str = "palette_kid/Red_Stealth_Helmet.bin";
const EYECLOPS_HELMET_PALETTE_PATH: &str = "palette_kid/Eyeclops_Helmet.bin";
const JUGGERNAUT_HELMET_PALETTE_PATH: &str = "palette_kid/Juggernaut_Helmet.bin";
const IRON_KNIGHT_HELMET_PALETTE_PATH: &str = "palette_kid/Iron_Knight_Helmet.bin";
const BERZERKER_HELMET_PALETTE_PATH: &str = "palette_kid/Berzerker_Helmet.bin";
const MANIAXE_HELMET_PALETTE_PATH: &str = "palette_kid/Maniaxe_Helmet.bin";
const MICROMAX_HELMET_PALETTE_PATH: &str = "palette_kid/Micromax_Helmet.bin";
const BLOCK_MAPPING_PATH: &str = "block_mappings.bin";
const PLAYER_FORM_THE_KID: u16 = 0;
const PLAYER_FORM_SKYCUTTER: u16 = 1;
const PLAYER_FORM_CYCLONE: u16 = 2;
const PLAYER_FORM_RED_STEALTH: u16 = 3;
const PLAYER_FORM_EYECLOPS: u16 = 4;
const PLAYER_FORM_JUGGERNAUT: u16 = 5;
const PLAYER_FORM_IRON_KNIGHT: u16 = 6;
const PLAYER_FORM_BERZERKER: u16 = 7;
const PLAYER_FORM_MANIAXE: u16 = 8;
const PLAYER_FORM_MICROMAX: u16 = 9;
const THEME_NAMES: [&str; 11] = [
    "theme0", "sky", "ice", "hill", "island", "desert", "swamp", "mountain", "cave", "forest",
    "city",
];
const FOREST_THEME_ID: u8 = 9;
const FOREST_ENEMY_ART_PERMUTATION: [u8; 16] =
    [0, 1, 2, 3, 4, 5, 6, 7, 0xA, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF];
const FOREST_TAR_MONSTER_ENEMY_ART_PERMUTATION: [u8; 16] =
    [0, 1, 2, 3, 4, 5, 6, 7, 7, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLevelVisuals {
    pub foreground_tiles: TileSheet,
    pub background_tiles: TileSheet,
    pub block_tiles: TileSheet,
    pub palettes: PaletteSet,
    pub background: BackgroundVisuals,
    pub level: LevelVisuals,
    pub hud: HudVisuals,
    pub player: PlayerVisuals,
    pub enemies: EnemyVisuals,
    pub floating_prizes: FloatingPrizeVisuals,
    pub flag: FlagVisuals,
    /// ASM Murderwall loc_140D2/loc_140EE loads `Murder_wall.bin` art to
    /// VRAM `$5F60` and copies its eight-colour palette to CRAM line 1.
    pub murder_wall: Option<MurderWallVisuals>,
    /// ASM Init_SpecialEffect_Lava loc_14C6A loads `Lava.bin` art to VRAM
    /// `$5F60` and copies `ingame/palette/lava.bin` to CRAM line 1.
    pub lava: Option<LavaVisuals>,
    pub platforms: PlatformVisuals,
    /// ASM `loc_D55E` / `Load_Object_EOL_Text` / `Load_Object_EOL_Number`
    /// draw score-screen labels and digits from `End_level_stats_text`.
    pub eol_score: Option<EolScoreVisuals>,
    /// ASM `CharacterCollision_TouchFlagpole` `loc_C186` decodes
    /// `MapEni_CC0E` with base `$42FB` and uploads 40 words per row to Scroll
    /// B. The paired art is `Hologram_background` at VRAM tile `$2FB` and
    /// `Hologram_blocks` at tile `$409`; native keeps the absolute tile
    /// indices so the renderer follows the ROM nametable directly.
    pub eol_hologram: Option<EolHologramVisuals>,
    /// ASM `Load_TitleCard` `loc_1AB26` (kid.asm:28750) decompresses
    /// `ArtComp_19C68_TtlCardLetters` to VRAM tile `$8120` and copies
    /// `Pal_19C48` (16 colours, palette line 1) before the per-glyph
    /// worker shell at `loc_1AC70` emits sprites. Native exposes this as
    /// an optional tile sheet + palette so the `build_transition_scene`
    /// renderer can draw real letter art instead of the ASCII fallback
    /// font when the asset is available.
    pub title_card_letters: Option<TitleCardLettersVisuals>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleCardLettersVisuals {
    pub tile_sheet: TileSheet,
    pub palette: Palette,
}

pub fn load_level_visuals(manifest: &AssetManifest, level_id: u8) -> Result<NativeLevelVisuals> {
    ensure_generated_bundle_contract(manifest)?;
    let level_bundle = load_level_asset_bundle(manifest, level_id)?;
    let header = level_bundle
        .parsed_header
        .as_ref()
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} is missing header metadata"))?;
    let decoded = manifest
        .decoded_levels
        .iter()
        .find(|d| d.level_id == level_id);

    let foreground_theme = resolve_theme_name(header.level_theme)
        .ok_or_else(|| anyhow!("unsupported foreground theme id {}", header.level_theme))?;
    let background_theme = resolve_theme_name(header.background_theme).ok_or_else(|| {
        anyhow!(
            "unsupported background theme id {}",
            header.background_theme
        )
    })?;

    let foreground_layout = match level_bundle.foreground.as_ref() {
        Some(asset) => decode_level_saxman_asset(asset, "foreground layout")?,
        None => match decoded {
            Some(d) => read_decoded_file(manifest, &d.foreground, "foreground layout")?,
            None => bail!("level 0x{level_id:02X} has no foreground layout data"),
        },
    };
    let block_layout = match level_bundle.block.as_ref() {
        Some(asset) => asset.bytes.clone(),
        None => match decoded {
            Some(d) => read_decoded_file(manifest, &d.block, "block layout")?,
            None => bail!("level 0x{level_id:02X} has no block layout data"),
        },
    };
    let background_layout = match level_bundle.background.as_ref() {
        Some(asset) => asset.bytes.clone(),
        None => match decoded {
            Some(d) => read_decoded_file(manifest, &d.background, "background layout")?,
            None => bail!("level 0x{level_id:02X} has no background layout data"),
        },
    };
    let parsed_foreground_layout = ForegroundLayout::parse(
        &foreground_layout,
        header.width_screens,
        header.height_screens,
    )
    .context(format!(
        "failed to parse level 0x{level_id:02X} foreground layout"
    ))?;

    let mut foreground_tiles = load_tile_sheet(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("artcomp_fg/{foreground_theme}.bin")).ok_or_else(
            || anyhow!("missing foreground art asset theme/artcomp_fg/{foreground_theme}.bin"),
        )?,
        "foreground tile sheet",
    )?;
    let mut background_tiles = load_tile_sheet(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("artcomp_bg/{background_theme}.bin")).ok_or_else(
            || anyhow!("missing background art asset theme/artcomp_bg/{background_theme}.bin"),
        )?,
        "background tile sheet",
    )?;
    let mut block_tiles = load_tile_sheet(
        manifest,
        "ingame",
        find_ingame_asset(manifest, BLOCKS_ART_PATH)
            .ok_or_else(|| anyhow!("missing block art asset ingame/{BLOCKS_ART_PATH}"))?,
        "block tile sheet",
    )?;
    let terrain_quads = parse_tile_quads(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("mappings/{foreground_theme}.bin")).ok_or_else(
            || anyhow!("missing terrain mapping asset theme/mappings/{foreground_theme}.bin"),
        )?,
        "terrain mapping",
    )?)?;
    append_indoor_rubber_block_tiles_sub_144da(
        &mut block_tiles,
        &foreground_tiles,
        &terrain_quads,
        header.level_theme,
    );
    let hud_digits = load_tile_sheet(
        manifest,
        "ingame",
        find_ingame_asset(manifest, HUD_NUMBERS_ART_PATH)
            .ok_or_else(|| anyhow!("missing HUD digit art asset ingame/{HUD_NUMBERS_ART_PATH}"))?,
        "HUD digit tile sheet",
    )?;
    let hud_diamond_tiles = load_uncompressed_tile_sheet(
        manifest,
        "ingame",
        find_ingame_asset(manifest, HUD_DIAMOND_ART_PATH).ok_or_else(|| {
            anyhow!("missing HUD diamond art asset ingame/{HUD_DIAMOND_ART_PATH}")
        })?,
        "HUD diamond tile sheet",
    )?;
    let hud_hitpoint_sheet = load_uncompressed_tile_sheet(
        manifest,
        "ingame",
        find_ingame_asset(manifest, HUD_HITPOINT_ART_PATH).ok_or_else(|| {
            anyhow!("missing HUD hitpoint art asset ingame/{HUD_HITPOINT_ART_PATH}")
        })?,
        "HUD hitpoint tile sheet",
    )?;
    let fg_palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("palette_fg/{foreground_theme}.bin")).ok_or_else(
            || anyhow!("missing foreground palette asset theme/palette_fg/{foreground_theme}.bin"),
        )?,
        "foreground palette",
    )?)?;
    let bg_palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("palette_bg/{background_theme}.bin")).ok_or_else(
            || anyhow!("missing background palette asset theme/palette_bg/{background_theme}.bin"),
        )?,
        "background palette",
    )?)?;

    if let Some(perm) = load_palette_permutation(manifest, fg_permutation_file(header.level_theme))
    {
        foreground_tiles.apply_pixel_permutation(&perm);
    }
    if let Some(perm) =
        load_palette_permutation(manifest, bg_permutation_file(header.background_theme))
    {
        background_tiles.apply_pixel_permutation(&perm);
    }

    let player = load_player_visuals(manifest)?;
    let enemies = load_enemy_visuals(
        manifest,
        header.level_theme,
        level_bundle.parsed_enemy_layout.as_ref(),
    );
    let floating_prizes = load_floating_prize_visuals(manifest, &fg_palette, &hud_diamond_tiles)?;
    let flag = load_flag_visuals(manifest, &fg_palette)?;
    let murder_wall = load_murder_wall_visuals(manifest).unwrap_or_else(|err| {
        if !cfg!(test) {
            eprintln!("warning: murder wall art unavailable: {err}");
        }
        None
    });
    let lava = load_lava_visuals(manifest).unwrap_or_else(|err| {
        if !cfg!(test) {
            eprintln!("warning: lava art unavailable: {err}");
        }
        None
    });
    let platforms = load_platform_visuals(manifest, foreground_theme)?;

    let palettes = PaletteSet {
        foreground: fg_palette,
        background: bg_palette,
    };

    let block_quads = parse_tile_quads(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, BLOCK_MAPPING_PATH)
            .ok_or_else(|| anyhow!("missing block mapping asset theme/{BLOCK_MAPPING_PATH}"))?,
        "block mapping",
    )?)?;
    let background = load_background_visuals(manifest, background_theme, &background_layout)
        .context(format!(
            "failed to load level 0x{level_id:02X} background visuals"
        ))?;

    Ok(NativeLevelVisuals {
        foreground_tiles,
        background_tiles,
        block_tiles,
        palettes,
        background,
        level: LevelVisuals {
            level_id,
            foreground_theme_id: header.level_theme,
            background_theme_id: header.background_theme,
            murder_wall_flags: header.murder_wall_flags,
            background_scroll: level_bundle.parsed_bgscroll.clone(),
            foreground_layout: parsed_foreground_layout,
            foreground_layout_bytes: foreground_layout,
            block_layout_bytes: block_layout,
            background_layout_bytes: background_layout,
            block_mapping: BlockVisualMapping::new(terrain_quads, block_quads)
                .context("failed to build Level 1 block visual mapping")?,
        },
        hud: HudVisuals {
            digits: hud_digits,
            digit_base_tile_index: HUD_DIGIT_BASE_TILE_INDEX,
            diamond_tiles: hud_diamond_tiles,
            diamond_base_tile_index: HUD_DIAMOND_BASE_TILE_INDEX,
            hitpoint_sheet: hud_hitpoint_sheet,
            icons: vec![
                HudIcon::new("ankh", (0x01A1, 0x008D), TileRef::from_attr_word(0x8231)),
                HudIcon::new("colon", (0x0090, 0x0090), TileRef::from_attr_word(0x86C4)),
                HudIcon::new("diamond", (0x01A0, 0x00A2), TileRef::from_attr_word(0x86F2)),
            ],
            hitpoint_tiles: [
                TileRef::from_attr_word(0x822D),
                TileRef::from_attr_word(0x822E),
                TileRef::from_attr_word(0x822F),
            ],
        },
        player,
        enemies,
        floating_prizes,
        flag,
        murder_wall,
        lava,
        platforms,
        eol_score: load_eol_score_visuals(manifest).unwrap_or_else(|err| {
            if !cfg!(test) {
                eprintln!("warning: end-of-level score art unavailable: {err}");
            }
            None
        }),
        eol_hologram: load_eol_hologram_visuals(manifest).unwrap_or_else(|err| {
            if !cfg!(test) {
                eprintln!("warning: end-of-level hologram art unavailable: {err}");
            }
            None
        }),
        title_card_letters: load_title_card_letters_visuals(manifest).unwrap_or_else(|err| {
            if !cfg!(test) {
                eprintln!("warning: title card letter art unavailable: {err}");
            }
            None
        }),
    })
}

/// Load the Title Card letter sprite sheet + palette extracted from the
/// `scenes/` asset tree. ASM `Load_TitleCard` `loc_1AB26` decompresses
/// `ArtComp_19C68_TtlCardLetters` to VRAM tile `$8120` and copies
/// `Pal_19C48` (16 colours, palette line 1). The 35 letter sprites
/// (a..z plus 5 wide `$7B`..`$7F` plus 4 markers `$80`..`$83`) are
/// mapped through `off_1AC2A` (kid.asm:28859-28893), each `glyph - $61`
/// indexes into the tile sheet.
fn load_title_card_letters_visuals(
    manifest: &AssetManifest,
) -> Result<Option<TitleCardLettersVisuals>> {
    let scenes_root = manifest.output_root.join("scenes");
    let art_path = scenes_root.join("artcomp/Title_card_letters.bin");
    if !art_path.is_file() {
        return Ok(None);
    }
    let palette_path = scenes_root.join("palette/Title_card_letters.bin");
    if !palette_path.is_file() {
        return Ok(None);
    }
    let art_compressed = fs::read(&art_path).with_context(|| {
        format!(
            "failed to read title card letters art: {}",
            art_path.display()
        )
    })?;
    let art_bytes =
        decode_saxman(&art_compressed).context("failed to decompress title card letters art")?;
    let tile_sheet = TileSheet::from_4bpp_bytes(&art_bytes)
        .context("failed to parse title card letters tile sheet")?;
    let palette_raw = fs::read(&palette_path).with_context(|| {
        format!(
            "failed to read title card letters palette: {}",
            palette_path.display()
        )
    })?;
    let palette =
        Palette::from_bytes(&palette_raw).context("failed to parse title card letters palette")?;
    Ok(Some(TitleCardLettersVisuals {
        tile_sheet,
        palette,
    }))
}

fn load_murder_wall_visuals(manifest: &AssetManifest) -> Result<Option<MurderWallVisuals>> {
    let Some(art) = find_ingame_asset(manifest, MURDER_WALL_ART_PATH) else {
        return Ok(None);
    };
    let Some(palette_asset) = find_ingame_asset(manifest, MURDER_WALL_PALETTE_PATH) else {
        return Ok(None);
    };
    let tile_sheet = load_tile_sheet(manifest, "ingame", art, "murder wall tile sheet")?;
    let palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "ingame",
        palette_asset,
        "murder wall palette",
    )?)
    .context("failed to parse murder wall palette")?;
    Ok(Some(MurderWallVisuals {
        palette,
        tile_sheet,
        vram_base_tile_index: MURDER_WALL_VRAM_BASE_TILE_INDEX,
    }))
}

fn load_lava_visuals(manifest: &AssetManifest) -> Result<Option<LavaVisuals>> {
    let Some(art) = find_ingame_asset(manifest, LAVA_ART_PATH) else {
        return Ok(None);
    };
    let palette_path = manifest.source_root.join("ingame").join(LAVA_PALETTE_PATH);
    if !palette_path.is_file() {
        return Ok(None);
    }
    let tile_sheet = load_tile_sheet(manifest, "ingame", art, "lava tile sheet")?;
    let palette_raw = fs::read(&palette_path)
        .with_context(|| format!("failed to read lava palette: {}", palette_path.display()))?;
    let palette = Palette::from_bytes(&palette_raw).context("failed to parse lava palette")?;
    Ok(Some(LavaVisuals {
        palette,
        tile_sheet,
        vram_base_tile_index: LAVA_VRAM_BASE_TILE_INDEX,
    }))
}

fn load_eol_score_visuals(manifest: &AssetManifest) -> Result<Option<EolScoreVisuals>> {
    let scenes_root = manifest.output_root.join("scenes");
    let art_path = scenes_root.join("artcomp/End_level_stats_text.bin");
    if !art_path.is_file() {
        return Ok(None);
    }
    let palette_path = scenes_root.join("palette/Score_screen.bin");
    if !palette_path.is_file() {
        return Ok(None);
    }
    let art_compressed = fs::read(&art_path)
        .with_context(|| format!("failed to read EOL score art: {}", art_path.display()))?;
    let art_bytes = decode_saxman(&art_compressed).context("failed to decompress EOL score art")?;
    let tile_sheet =
        TileSheet::from_4bpp_bytes(&art_bytes).context("failed to parse EOL score tile sheet")?;
    let palette_raw = fs::read(&palette_path).with_context(|| {
        format!(
            "failed to read EOL score palette: {}",
            palette_path.display()
        )
    })?;
    let palette = Palette::from_bytes(&palette_raw).context("failed to parse EOL score palette")?;
    let frame = |descriptor: SpriteFrameDescriptor| {
        crop_eol_score_frame_to_declared_bounds(&tile_sheet, descriptor)
    };
    Ok(Some(EolScoreVisuals {
        palette,
        bonus: frame(fd(0x000, 0x00, 0x00, 0x40, 0x08)),
        continues: frame(fd(0x008, 0x00, 0x00, 0x6C, 0x08)),
        no_hit: frame(fd(0x065, 0x00, 0x00, 0x57, 0x08)),
        no_prize: frame(fd(0x070, 0x00, 0x00, 0x6F, 0x08)),
        path: frame(fd(0x080, 0x00, 0x00, 0x35, 0x08)),
        score: frame(fd(0x091, 0x00, 0x00, 0x3D, 0x08)),
        seconds: frame(fd(0x099, 0x00, 0x00, 0x5F, 0x08)),
        speed: frame(fd(0x0AD, 0x00, 0x00, 0x3F, 0x08)),
        time: frame(fd(0x0B9, 0x00, 0x00, 0x2D, 0x08)),
        under: frame(fd(0x0C3, 0x00, 0x00, 0x40, 0x08)),
        digits: vec![
            frame(fd(0x0CB, 0x00, 0x00, 0x0C, 0x0B)),
            frame(fd(0x07E, -0x03, 0x00, 0x05, 0x0B)),
            frame(fd(0x0BF, 0x00, 0x00, 0x0B, 0x0B)),
            frame(fd(0x0B5, 0x00, 0x00, 0x0B, 0x0B)),
            frame(fd(0x02D, 0x00, 0x00, 0x0C, 0x0B)),
            frame(fd(0x029, 0x00, 0x00, 0x0B, 0x0B)),
            frame(fd(0x0A9, 0x00, 0x00, 0x0A, 0x0B)),
            frame(fd(0x0A5, 0x00, 0x00, 0x0B, 0x0B)),
            frame(fd(0x016, 0x00, 0x00, 0x0B, 0x0B)),
            frame(fd(0x061, 0x00, 0x00, 0x0B, 0x0B)),
        ],
    }))
}

fn crop_eol_score_frame_to_declared_bounds(
    source_sheet: &TileSheet,
    descriptor: SpriteFrameDescriptor,
) -> SpriteFrame {
    let aligned_width = align_sprite_dimension(descriptor.width_px);
    let aligned_height = align_sprite_dimension(descriptor.height_px);
    let columns = aligned_width / 8;
    let rows = aligned_height / 8;
    let tile_count = columns.saturating_mul(rows);
    let mut tiles = vec![
        crate::Tile { pixels: [0; 64] };
        usize::from(descriptor.tile_base_index.saturating_add(tile_count))
    ];

    for column in 0..columns {
        for row in 0..rows {
            let sprite_slot = column * rows + row;
            let source_index = descriptor.tile_base_index.saturating_add(sprite_slot);
            let Some(source) = source_sheet.tiles.get(usize::from(source_index)) else {
                continue;
            };
            let Some(destination) = tiles.get_mut(usize::from(source_index)) else {
                continue;
            };
            for pixel_y in 0..8 {
                let frame_y = row * 8 + pixel_y;
                if frame_y >= descriptor.height_px {
                    continue;
                }
                for pixel_x in 0..8 {
                    let frame_x = column * 8 + pixel_x;
                    if frame_x >= descriptor.width_px {
                        continue;
                    }
                    let pixel_index = usize::from(pixel_y * 8 + pixel_x);
                    destination.pixels[pixel_index] = source.pixels[pixel_index];
                }
            }
        }
    }

    SpriteFrame {
        origin_x: descriptor.origin_x,
        origin_y: descriptor.origin_y,
        width_px: descriptor.width_px,
        height_px: descriptor.height_px,
        tile_base_index: descriptor.tile_base_index,
        tile_sheet: TileSheet { tiles },
    }
}

fn load_eol_hologram_visuals(manifest: &AssetManifest) -> Result<Option<EolHologramVisuals>> {
    let scenes_root = manifest.output_root.join("scenes");
    let background_path = scenes_root.join(EOL_HOLOGRAM_BACKGROUND_ART_PATH);
    let blocks_path = scenes_root.join(EOL_HOLOGRAM_BLOCKS_ART_PATH);
    let tilemap_path = scenes_root.join(EOL_HOLOGRAM_TILEMAP_PATH);
    let palette_path = scenes_root.join(EOL_HOLOGRAM_PALETTE_PATH);
    if !background_path.is_file()
        || !blocks_path.is_file()
        || !tilemap_path.is_file()
        || !palette_path.is_file()
    {
        return Ok(None);
    }

    let background_compressed = fs::read(&background_path).with_context(|| {
        format!(
            "failed to read EOL hologram background art: {}",
            background_path.display()
        )
    })?;
    let background_bytes = decode_saxman(&background_compressed)
        .context("failed to decompress EOL hologram background art")?;
    let background_tiles = TileSheet::from_4bpp_bytes(&background_bytes)
        .context("failed to parse EOL hologram background tiles")?;

    let blocks_compressed = fs::read(&blocks_path).with_context(|| {
        format!(
            "failed to read EOL hologram block art: {}",
            blocks_path.display()
        )
    })?;
    let blocks_bytes =
        decode_saxman(&blocks_compressed).context("failed to decompress EOL hologram block art")?;
    let block_tiles = TileSheet::from_4bpp_bytes(&blocks_bytes)
        .context("failed to parse EOL hologram block tiles")?;

    let tiles = combine_eol_hologram_tile_sheet(background_tiles, block_tiles);

    let tilemap_raw = fs::read(&tilemap_path).with_context(|| {
        format!(
            "failed to read EOL hologram tilemap: {}",
            tilemap_path.display()
        )
    })?;
    // ASM loc_C140: `move.w #$42FB,d0; lea MapEni_CC0E(pc),a0; jsr EniDec`.
    let decoded_words =
        decode_enigma(&tilemap_raw, 0x42FB).context("failed to decode EOL hologram tilemap")?;
    let rows = decoded_words
        .chunks(40)
        .take(28)
        .map(|row| {
            row.iter()
                .copied()
                .map(|word| TileRef {
                    palette_line: 0,
                    ..TileRef::from_attr_word(word)
                })
                .collect()
        })
        .collect();

    let palette_raw = fs::read(&palette_path).with_context(|| {
        format!(
            "failed to read EOL hologram palette: {}",
            palette_path.display()
        )
    })?;
    let palette =
        Palette::from_bytes(&palette_raw).context("failed to parse EOL hologram palette")?;

    Ok(Some(EolHologramVisuals {
        palette,
        tile_sheet: tiles,
        rows,
    }))
}

fn combine_eol_hologram_tile_sheet(
    background_tiles: TileSheet,
    block_tiles: TileSheet,
) -> TileSheet {
    let tile_count = (usize::from(EOL_HOLOGRAM_BACKGROUND_VRAM_BASE_TILE_INDEX)
        + background_tiles.tiles.len())
    .max(usize::from(EOL_HOLOGRAM_BLOCKS_VRAM_BASE_TILE_INDEX) + block_tiles.tiles.len());
    let mut tiles = vec![Tile { pixels: [0; 64] }; tile_count];
    for (index, tile) in background_tiles.tiles.into_iter().enumerate() {
        let dest = usize::from(EOL_HOLOGRAM_BACKGROUND_VRAM_BASE_TILE_INDEX) + index;
        if dest < tiles.len() {
            tiles[dest] = tile;
        }
    }
    for (index, tile) in block_tiles.tiles.into_iter().enumerate() {
        let dest = usize::from(EOL_HOLOGRAM_BLOCKS_VRAM_BASE_TILE_INDEX) + index;
        tiles[dest] = tile;
    }
    TileSheet { tiles }
}

fn resolve_theme_name(theme_id: u8) -> Option<&'static str> {
    THEME_NAMES.get(usize::from(theme_id)).copied()
}

/// Foreground palette permutation file for a given theme ID.
/// From ASM: City=city_fg, Forest=forest_fg, Mountain=mountain_fg, else identity.
fn fg_permutation_file(theme_id: u8) -> &'static str {
    match theme_id {
        9 => "palette_permutations/forest_fg.bin",
        7 => "palette_permutations/mountain_fg.bin",
        10 => "palette_permutations/city_fg.bin",
        _ => "palette_permutations/identity.bin",
    }
}

/// Background palette permutation file for a given theme ID.
/// From ASM: Forest=forest_bg, Mountain=mountain_bg, Hills=hill_bg, else identity.
fn bg_permutation_file(theme_id: u8) -> &'static str {
    match theme_id {
        9 => "palette_permutations/forest_bg.bin",
        7 => "palette_permutations/mountain_bg.bin",
        3 => "palette_permutations/hill_bg.bin",
        _ => "palette_permutations/identity.bin",
    }
}

/// Load a 16-byte palette permutation table from the theme assets.
fn load_palette_permutation(manifest: &AssetManifest, relative_path: &str) -> Option<[u8; 16]> {
    let asset = find_theme_asset(manifest, relative_path)?;
    let bytes = read_root_asset(manifest, "theme", asset, "palette permutation").ok()?;
    if bytes.len() < 16 {
        return None;
    }
    let mut table = [0u8; 16];
    table.copy_from_slice(&bytes[..16]);
    Some(table)
}

fn find_theme_asset<'a>(manifest: &'a AssetManifest, relative_path: &str) -> Option<&'a AssetFile> {
    manifest
        .theme_assets
        .iter()
        .find(|asset| asset.relative_path == relative_path)
}

fn find_ingame_asset<'a>(
    manifest: &'a AssetManifest,
    relative_path: &str,
) -> Option<&'a AssetFile> {
    manifest
        .ingame_assets
        .iter()
        .find(|asset| asset.relative_path == relative_path)
}

fn load_background_visuals(
    manifest: &AssetManifest,
    theme_name: &str,
    layout_bytes: &[u8],
) -> Result<BackgroundVisuals> {
    if uses_layered_backgrounds(theme_name) {
        return load_layered_background_visuals(manifest, theme_name, layout_bytes);
    }

    let placements = parse_chunked_background_layout(layout_bytes)
        .context("failed to parse chunked background layout")?;
    let prefix = format!("bg_chunks/{theme_name}_");
    let mut chunk_assets = manifest
        .theme_assets
        .iter()
        .filter_map(|asset| {
            parse_background_chunk_id(&asset.relative_path, &prefix)
                .map(|chunk_id| (chunk_id, asset))
        })
        .collect::<Vec<_>>();
    chunk_assets.sort_by_key(|(chunk_id, _)| *chunk_id);

    let max_chunk_id = chunk_assets
        .iter()
        .map(|(chunk_id, _)| *chunk_id)
        .max()
        .unwrap_or(0);
    let mut chunks = vec![
        BackgroundChunk {
            width_tiles: 0,
            height_tiles: 0,
            tiles: Vec::new(),
        };
        usize::from(max_chunk_id) + 1
    ];
    for (chunk_id, asset) in chunk_assets {
        let bytes =
            read_root_asset(manifest, "theme", asset, "background chunk").with_context(|| {
                format!(
                    "failed to read background chunk {} for theme {theme_name}",
                    asset.relative_path
                )
            })?;
        chunks[usize::from(chunk_id)] = parse_background_chunk(&bytes).with_context(|| {
            format!(
                "failed to parse background chunk {} for theme {theme_name}",
                asset.relative_path
            )
        })?;
    }

    for placement in &placements {
        let chunk = chunks.get(usize::from(placement.chunk_id)).ok_or_else(|| {
            anyhow!(
                "background layout references missing chunk id 0x{:04X} for theme {theme_name}",
                placement.chunk_id
            )
        })?;
        if chunk.tiles.is_empty() {
            bail!(
                "background layout references unloaded chunk id 0x{:04X} for theme {theme_name}",
                placement.chunk_id
            );
        }
    }

    Ok(BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
        placements,
        chunks,
    }))
}

fn load_layered_background_visuals(
    manifest: &AssetManifest,
    theme_name: &str,
    layout_bytes: &[u8],
) -> Result<BackgroundVisuals> {
    let layout = parse_layered_background_layout(layout_bytes, theme_name).with_context(|| {
        format!("failed to parse layered background layout for theme {theme_name}")
    })?;
    let prefix = format!("bg_chunks/{theme_name}_eni_");
    let mut plane_assets = manifest
        .theme_assets
        .iter()
        .filter_map(|asset| {
            parse_layered_background_chunk_id(&asset.relative_path, &prefix)
                .map(|plane_id| (plane_id, asset))
        })
        .collect::<Vec<_>>();
    plane_assets.sort_by_key(|(plane_id, _)| *plane_id);

    let mut planes = Vec::with_capacity(plane_assets.len());
    for (plane_id, asset) in plane_assets {
        let encoded = read_root_asset(manifest, "theme", asset, "layered background plane")
            .with_context(|| {
                format!(
                    "failed to read layered background plane {} for theme {theme_name}",
                    asset.relative_path
                )
            })?;
        let decoded = decode_enigma(&encoded, 0).with_context(|| {
            format!(
                "failed to decode layered background plane {} for theme {theme_name}",
                asset.relative_path
            )
        })?;
        let mut plane = parse_layered_background_plane(&decoded).with_context(|| {
            format!(
                "failed to parse layered background plane {plane_id:02X} for theme {theme_name}"
            )
        })?;
        apply_layered_background_palette_fallback(theme_name, plane_id, &mut plane);
        planes.push(plane);
    }

    Ok(BackgroundVisuals::Layered(
        compose_layered_background(&layout, &planes).with_context(|| {
            format!("failed to compose layered background rows for theme {theme_name}")
        })?,
    ))
}

fn uses_layered_backgrounds(theme_name: &str) -> bool {
    matches!(theme_name, "hill" | "desert" | "mountain" | "forest")
}

fn apply_layered_background_palette_fallback(
    theme_name: &str,
    plane_id: u16,
    plane: &mut LayeredBackgroundPlane,
) {
    let fallback_palette_line = match (theme_name, plane_id) {
        // PlaneEd palette destination offsets are byte offsets into CRAM:
        // 16 => upper half of line 0, 48 => upper half of line 1.
        ("desert", 0..=2) => Some(2),
        ("forest", 0 | 1) => Some(1),
        ("forest", 2) => Some(2),
        ("hill", 0 | 1) => Some(2),
        _ => None,
    };
    let Some(palette_line) = fallback_palette_line else {
        return;
    };

    for row in &mut plane.rows {
        for tile in row {
            if tile.palette_line == 0 {
                tile.palette_line = palette_line;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlayerFrameDescriptor {
    relative_path: &'static str,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
}

#[derive(Clone, Copy)]
struct PlayerTransformStepDescriptor {
    duration_frames: usize,
    frame: Option<PlayerFrameDescriptor>,
}

struct PlayerFormSpec {
    form_id: u16,
    palette_path: &'static str,
    palette_family: &'static str,
    idle: PlayerFrameDescriptor,
    walk: &'static [PlayerFrameDescriptor],
    /// ASM stru_8B6A: pre-drawn tilted slope walk frames (right-facing).
    slope_walk: &'static [PlayerFrameDescriptor],
    /// Left-facing slope walk frames.
    slope_walk_left: &'static [PlayerFrameDescriptor],
    jump_rise: PlayerFrameDescriptor,
    jump_fall: PlayerFrameDescriptor,
    pull_up: &'static [PlayerFrameDescriptor],
    slash_ground: &'static [PlayerFrameDescriptor],
    slash_air: &'static [PlayerFrameDescriptor],
    transform_palette: Option<(&'static str, &'static str)>,
    transform_steps: &'static [PlayerTransformStepDescriptor],
}

const fn pfd(
    relative_path: &'static str,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
) -> PlayerFrameDescriptor {
    PlayerFrameDescriptor {
        relative_path,
        origin_x,
        origin_y,
        width_px,
        height_px,
    }
}

const fn tfd(
    duration_frames: usize,
    frame: PlayerFrameDescriptor,
) -> PlayerTransformStepDescriptor {
    PlayerTransformStepDescriptor {
        duration_frames,
        frame: Some(frame),
    }
}

const fn blank_tfd(duration_frames: usize) -> PlayerTransformStepDescriptor {
    PlayerTransformStepDescriptor {
        duration_frames,
        frame: None,
    }
}

#[derive(Clone, Copy)]
struct SpriteFrameDescriptor {
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
    tile_base_index: u16,
}

/// Enemy family specification: object_type, art path, palette path, frame duration, frames.
/// Frame data extracted from kid-chameleon-disasm sprite_frame_vram definitions.
struct EnemyFamilySpec {
    object_type: u8,
    art_path: &'static str,
    palette_path: &'static str,
    /// Extra palette variant paths (variants 1 and 2).
    extra_palette_paths: &'static [&'static str],
    frame_duration: usize,
    frames: &'static [SpriteFrameDescriptor],
}

struct FloatingPrizeHelmetSpec {
    content_id: u8,
    art: PlayerFrameDescriptor,
    base_palette_path: &'static str,
    palette_path: &'static str,
    palette_family: &'static str,
}

struct FloatingPrizeIconSpec {
    content_id: u8,
    art: PlayerFrameDescriptor,
}

const fn fd(
    tile_base_index: u16,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
) -> SpriteFrameDescriptor {
    SpriteFrameDescriptor {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
    }
}

static FLOATING_PRIZE_HELMETS: &[FloatingPrizeHelmetSpec] = &[
    FloatingPrizeHelmetSpec {
        content_id: 0x02,
        art: pfd("artunc_kid/skycutter_mask.bin", 0x08, -2, 0x11, 0x0E),
        base_palette_path: SKYCUTTER_PALETTE_PATH,
        palette_path: SKYCUTTER_HELMET_PALETTE_PATH,
        palette_family: "Skycutter helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x03,
        art: pfd("artunc_kid/cyclone_mask.bin", 0x0B, -2, 0x17, 0x0F),
        base_palette_path: CYCLONE_PALETTE_PATH,
        palette_path: CYCLONE_HELMET_PALETTE_PATH,
        palette_family: "Cyclone helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x04,
        art: pfd("artunc_kid/red_stealth_mask.bin", 0x0A, -2, 0x15, 0x11),
        base_palette_path: RED_STEALTH_PALETTE_PATH,
        palette_path: RED_STEALTH_HELMET_PALETTE_PATH,
        palette_family: "Red Stealth helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x05,
        art: pfd("artunc_kid/eyeclops_mask.bin", 0x0A, -2, 0x15, 0x10),
        base_palette_path: EYECLOPS_PALETTE_PATH,
        palette_path: EYECLOPS_HELMET_PALETTE_PATH,
        palette_family: "Eyeclops helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x06,
        art: pfd("artunc_kid/juggernaut_mask.bin", 0x08, -2, 0x12, 0x14),
        base_palette_path: JUGGERNAUT_PALETTE_PATH,
        palette_path: JUGGERNAUT_HELMET_PALETTE_PATH,
        palette_family: "Juggernaut helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x07,
        art: pfd("artunc_kid/iron_knight_mask.bin", 0x09, -2, 0x16, 0x15),
        base_palette_path: IRON_KNIGHT_PALETTE_PATH,
        palette_path: IRON_KNIGHT_HELMET_PALETTE_PATH,
        palette_family: "Iron Knight helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x08,
        art: pfd("artunc_kid/berzerker_mask.bin", 0x09, -2, 0x13, 0x14),
        base_palette_path: BERZERKER_PALETTE_PATH,
        palette_path: BERZERKER_HELMET_PALETTE_PATH,
        palette_family: "Berzerker helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x09,
        art: pfd("artunc_kid/maniaxe_mask.bin", 0x08, -2, 0x10, 0x12),
        base_palette_path: MANIAXE_PALETTE_PATH,
        palette_path: MANIAXE_HELMET_PALETTE_PATH,
        palette_family: "Maniaxe helmet palette",
    },
    FloatingPrizeHelmetSpec {
        content_id: 0x0A,
        art: pfd("artunc_kid/micromax_mask.bin", 0x09, -2, 0x13, 0x12),
        base_palette_path: MICROMAX_PALETTE_PATH,
        palette_path: MICROMAX_HELMET_PALETTE_PATH,
        palette_family: "Micromax helmet palette",
    },
];

static FLOATING_PRIZE_ICONS: &[FloatingPrizeIconSpec] = &[
    FloatingPrizeIconSpec {
        // ASM PrizeAnkh_Init -> stru_10C08 -> unk_E0FE6:
        // sprite_frame_vram $231,$06,$FE,$10,$10.
        content_id: 0x0B,
        art: pfd("artunc/Life_icon_(3_frames).bin", 0x06, -2, 0x10, 0x10),
    },
    FloatingPrizeIconSpec {
        // ASM PrizeClock_Init -> stru_10C18 -> unk_E0FCE:
        // sprite_frame_vram $235,$07,$FE,$10,$10.
        content_id: 0x0C,
        art: pfd("artunc/Clock_icon_(3_frames).bin", 0x07, -2, 0x10, 0x10),
    },
    FloatingPrizeIconSpec {
        // ASM PrizeContinue_Init -> stru_10C28 -> unk_E0FD6:
        // sprite_frame_vram $239,$07,$FE,$10,$10.
        content_id: 0x0D,
        art: pfd(
            "artunc/Coin_continue_icon_(6_frames).bin",
            0x07,
            -2,
            0x10,
            0x10,
        ),
    },
];

static ENEMY_FAMILIES: &[EnemyFamilySpec] = &[
    EnemyFamilySpec {
        object_type: 0x00, // Fire Demon
        art_path: "artcomp/Enemy00_Fire_demon.bin",
        palette_path: "palette_enemy/Fire_Demon_1_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Fire_Demon_1_2.bin",
            "palette_enemy/Fire_Demon_1_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x000, 0x0E, 0x00, 0x1A, 0x20),
            fd(0x010, 0x0D, 0x00, 0x1A, 0x20),
            fd(0x020, 0x0C, 0x00, 0x1A, 0x20),
            fd(0x030, 0x0C, 0x00, 0x1E, 0x20),
            fd(0x020, 0x0C, 0x00, 0x1A, 0x20),
            fd(0x010, 0x0D, 0x00, 0x1A, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x01, // Diamond (collectible)
        art_path: "artcomp/Enemy01_Diamond.bin",
        palette_path: "palette_enemy/Crystal_1.bin",
        extra_palette_paths: &["palette_enemy/Crystal_2.bin", "palette_enemy/Crystal_3.bin"],
        frame_duration: 6,
        frames: &[
            fd(0x02F, 0x0A, 0x00, 0x14, 0x20),
            fd(0x03B, 0x0A, 0x00, 0x14, 0x20),
            fd(0x047, 0x0A, 0x00, 0x14, 0x20),
            fd(0x053, 0x0A, 0x00, 0x14, 0x20),
            fd(0x05F, 0x0A, 0x00, 0x14, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x03, // Robot
        art_path: "artcomp/Enemy03_Robot.bin",
        palette_path: "palette_enemy/Plethora.bin",
        extra_palette_paths: &[],
        frame_duration: 9,
        frames: &[
            fd(0x039, 0x12, 0x00, 0x1D, 0x14),
            fd(0x045, 0x0A, 0x00, 0x12, 0x15),
            fd(0x04E, 0x10, 0x00, 0x1C, 0x15),
            fd(0x05A, 0x12, 0x00, 0x1D, 0x14),
            fd(0x066, 0x0A, 0x00, 0x15, 0x15),
            fd(0x06F, 0x10, 0x00, 0x1C, 0x15),
            fd(0x0BB, 0x12, 0x00, 0x1E, 0x20),
            fd(0x0CB, 0x12, 0x00, 0x20, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x04, // Armadillo
        art_path: "artcomp/Enemy04_Armadillo.bin",
        palette_path: "palette_enemy/Armadillo_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Armadillo_2.bin",
            "palette_enemy/Armadillo_3.bin",
        ],
        frame_duration: 11,
        frames: &[
            fd(0x0A5, 0x15, 0x00, 0x27, 0x17),
            fd(0x0B4, 0x15, 0x00, 0x28, 0x18),
            fd(0x0C3, 0x15, 0x00, 0x28, 0x18),
            fd(0x0D2, 0x15, 0x00, 0x28, 0x17),
            fd(0x0E1, 0x15, 0x00, 0x28, 0x18),
            fd(0x0F0, 0x15, 0x00, 0x28, 0x18),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x05, // Tar Monster
        art_path: "artcomp/Enemy05_Tar_monster.bin",
        palette_path: "palette_enemy/Tar_Monster_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Tar_Monster_2.bin",
            "palette_enemy/Tar_Monster_3.bin",
        ],
        frame_duration: 3,
        frames: &[
            fd(0x0AC, 0x11, 0x00, 0x1B, 0x20),
            fd(0x044, 0x10, 0x00, 0x1B, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x06, // Sphere
        art_path: "artcomp/Enemy06_Sphere.bin",
        palette_path: "palette_enemy/Sphere_1.bin",
        extra_palette_paths: &["palette_enemy/Sphere_2.bin", "palette_enemy/Sphere_3.bin"],
        frame_duration: 9,
        frames: &[
            fd(0x012, 0x08, 0x09, 0x10, 0x10),
            fd(0x016, 0x08, 0x09, 0x10, 0x10),
            fd(0x01A, 0x08, 0x09, 0x10, 0x10),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x07, // Archer (skeleton)
        art_path: "artcomp/Enemy07_Archer.bin",
        palette_path: "palette_enemy/Archer_1.bin",
        extra_palette_paths: &["palette_enemy/Archer_2.bin", "palette_enemy/Archer_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x030, 0x0F, 0x00, 0x1D, 0x1F),
            fd(0x040, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x050, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x060, 0x0E, 0x00, 0x1B, 0x1F),
            fd(0x070, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x080, 0x0E, 0x00, 0x1B, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x08, // Orca
        art_path: "artcomp/Enemy08_Orca.bin",
        palette_path: "palette_enemy/Orca_1.bin",
        extra_palette_paths: &[],
        frame_duration: 9,
        frames: &[
            fd(0x05C, 0x10, 0x00, 0x1C, 0x25),
            fd(0x070, 0x10, 0x00, 0x1C, 0x26),
            fd(0x084, 0x10, 0x00, 0x1D, 0x26),
            fd(0x098, 0x10, 0x00, 0x1E, 0x25),
            fd(0x0AC, 0x10, 0x00, 0x1C, 0x26),
            fd(0x0C0, 0x10, 0x00, 0x1C, 0x26),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x09, // Crab
        art_path: "artcomp/Enemy09_Crab.bin",
        palette_path: "palette_enemy/Crab_1.bin",
        extra_palette_paths: &["palette_enemy/Crab_2.bin", "palette_enemy/Crab_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x030, 0x0F, 0x00, 0x1D, 0x1F),
            fd(0x040, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x050, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x060, 0x0E, 0x00, 0x1B, 0x1F),
            fd(0x070, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x080, 0x0E, 0x00, 0x1B, 0x20),
            // ASM Crab.asm loc_3B094 / `stru_3AF82`: C7C16, C7C06,
            // C7C0E are the three waited death frames before deletion.
            fd(0x020, 0x00, 0x00, 0x1D, 0x1B),
            fd(0x000, 0x00, 0x00, 0x1E, 0x1D),
            fd(0x010, 0x00, 0x00, 0x20, 0x1C),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0A, // Rock Tank
        art_path: "artcomp/Enemy0A_Rock_tank.bin",
        palette_path: "palette_enemy/Rock_tank_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Rock_tank_2.bin",
            "palette_enemy/Rock_tank_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x02C, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x030, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x034, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x000, 0x0F, 0x00, 0x1F, 0x1A),
            fd(0x010, 0x0F, 0x00, 0x1F, 0x19),
            fd(0x06C, 0x10, 0x00, 0x20, 0x14),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0B, // Rock Tank (shooting)
        art_path: "artcomp/Enemy0A_Rock_tank.bin",
        palette_path: "palette_enemy/Rock_tank_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Rock_tank_2.bin",
            "palette_enemy/Rock_tank_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x02C, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x030, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x034, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x000, 0x0F, 0x00, 0x1F, 0x1A),
            fd(0x010, 0x0F, 0x00, 0x1F, 0x19),
            fd(0x03C, 0x10, 0x00, 0x20, 0x14),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0C, // Dragon (flying)
        art_path: "artcomp/Enemy0C_Dragon.bin",
        palette_path: "palette_enemy/Dragon_1.bin",
        extra_palette_paths: &["palette_enemy/Dragon_2.bin", "palette_enemy/Dragon_3.bin"],
        frame_duration: 7,
        frames: &[
            fd(0x11D, 0x0D, 0x10, 0x2E, 0x1E),
            fd(0x135, 0x0D, 0x10, 0x30, 0x19),
            fd(0x14D, 0x0E, 0x10, 0x2F, 0x1A),
            fd(0x165, 0x0D, 0x10, 0x2E, 0x19),
            fd(0x17D, 0x0D, 0x10, 0x2E, 0x1E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0D, // Dragon (walking)
        art_path: "artcomp/Enemy0C_Dragon.bin",
        palette_path: "palette_enemy/Dragon_1.bin",
        extra_palette_paths: &["palette_enemy/Dragon_2.bin", "palette_enemy/Dragon_3.bin"],
        frame_duration: 9,
        frames: &[
            fd(0x060, 0x10, 0x00, 0x1F, 0x26),
            fd(0x074, 0x10, 0x00, 0x1F, 0x27),
            fd(0x088, 0x10, 0x00, 0x20, 0x28),
            fd(0x09C, 0x10, 0x00, 0x20, 0x27),
            fd(0x0B0, 0x10, 0x00, 0x20, 0x28),
            fd(0x0C4, 0x10, 0x00, 0x1F, 0x27),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0E, // Cloud
        art_path: "artcomp/Enemy0E_Cloud.bin",
        palette_path: "palette_enemy/Cloud_1.bin",
        extra_palette_paths: &["palette_enemy/Cloud_2.bin", "palette_enemy/Cloud_3.bin"],
        frame_duration: 11,
        frames: &[
            fd(0x095, 0x10, 0x00, 0x1F, 0x1F),
            fd(0x0A5, 0x10, -1, 0x20, 0x1E),
            fd(0x0B5, 0x10, -1, 0x20, 0x1E),
            fd(0x0C5, 0x10, -2, 0x20, 0x1D),
            fd(0x0B5, 0x10, -1, 0x20, 0x1E),
            fd(0x0A5, 0x10, -1, 0x20, 0x1E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0F, // UFO
        art_path: "artcomp/Enemy0F_UFO.bin",
        palette_path: "palette_enemy/UFO_1.bin",
        extra_palette_paths: &["palette_enemy/UFO_2.bin", "palette_enemy/UFO_3.bin"],
        frame_duration: 5,
        frames: &[
            fd(0x15A, 0x10, 0x00, 0x20, 0x1D),
            fd(0x16A, 0x10, 0x01, 0x20, 0x1E),
            fd(0x17A, 0x10, 0x02, 0x20, 0x1F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x10, // Goat
        art_path: "artcomp/Enemy10_Goat.bin",
        palette_path: "palette_enemy/Goat_1.bin",
        extra_palette_paths: &["palette_enemy/Goat_2.bin", "palette_enemy/Goat_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x0A8, 0x0F, 0x00, 0x1E, 0x1F),
            fd(0x0B8, 0x0F, 0x00, 0x1E, 0x20),
            fd(0x0C8, 0x10, 0x00, 0x1F, 0x1F),
            fd(0x0D8, 0x11, 0x00, 0x20, 0x1F),
            fd(0x0E8, 0x10, 0x00, 0x1F, 0x20),
            fd(0x0F8, 0x0F, 0x00, 0x1E, 0x1F),
            // ASM Goat.asm loc_3CB64 / `stru_3C9EA`: C7C56, C7C8E,
            // C7C96 are selected while the dead body hibernates.
            fd(0x000, 0x10, 0x00, 0x1F, 0x20),
            fd(0x088, 0x0F, 0x00, 0x1E, 0x1E),
            fd(0x098, 0x10, 0x00, 0x1F, 0x1D),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x11, // Ninja
        art_path: "artcomp/Enemy11_Ninja.bin",
        palette_path: "palette_enemy/Ninja_1.bin",
        extra_palette_paths: &["palette_enemy/Ninja_2.bin", "palette_enemy/Ninja_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x078, 0x0C, 0x00, 0x1A, 0x1F),
            fd(0x088, 0x0D, 0x00, 0x1A, 0x20),
            fd(0x098, 0x09, 0x00, 0x16, 0x20),
            fd(0x0A4, 0x0A, 0x00, 0x17, 0x1F),
            fd(0x0B0, 0x0A, 0x00, 0x18, 0x20),
            fd(0x0BC, 0x0D, 0x00, 0x1C, 0x20),
            // ASM Ninja.asm loc_3DB4E / `stru_3D50A`: C7D56, C7D5E,
            // C7D66 are the three waited death frames before deletion.
            fd(0x000, 0x11, 0x00, 0x1F, 0x1C),
            fd(0x010, 0x11, 0x00, 0x1D, 0x19),
            fd(0x020, 0x10, 0x00, 0x1D, 0x0F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x12, // Lion
        art_path: "artcomp/Enemy12_Lion.bin",
        palette_path: "palette_enemy/Lion_1.bin",
        extra_palette_paths: &["palette_enemy/Lion_2.bin", "palette_enemy/Lion_3.bin"],
        frame_duration: 7,
        frames: &[
            fd(0x16C, 0x15, 0x00, 0x2C, 0x2D),
            fd(0x190, 0x16, 0x00, 0x2C, 0x2E),
            fd(0x1B4, 0x15, 0x00, 0x2B, 0x2E),
            fd(0x1D8, 0x14, 0x00, 0x2D, 0x2D),
            fd(0x1FC, 0x15, 0x00, 0x28, 0x2E),
            fd(0x21A, 0x15, 0x00, 0x2B, 0x2E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x13, // Scorpion
        art_path: "artcomp/Enemy13_Scorpion.bin",
        palette_path: "palette_enemy/Scorpion_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Scorpion_2.bin",
            "palette_enemy/Scorpion_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x04C, 0x18, 0x00, 0x30, 0x1F),
            fd(0x064, 0x16, 0x00, 0x2E, 0x20),
            fd(0x07C, 0x16, 0x00, 0x30, 0x20),
            fd(0x094, 0x16, 0x00, 0x30, 0x1F),
            fd(0x0AC, 0x16, 0x00, 0x2F, 0x20),
            fd(0x0C4, 0x16, 0x00, 0x2F, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x14, // Spinning Twins
        art_path: "artcomp/Enemy14_Spinning_twins.bin",
        palette_path: "palette_enemy/Spinning_Twins_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Spinning_Twins_2.bin",
            "palette_enemy/Spinning_Twins_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x0A2, 0x20, 0x00, 0x40, 0x1F),
            fd(0x0C2, 0x19, 0x00, 0x34, 0x1F),
            fd(0x0DE, 0x0F, 0x00, 0x1E, 0x1F),
            fd(0x0EE, 0x1E, 0x00, 0x3D, 0x1F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x16, // Drip
        art_path: "artcomp/Enemy16_Drip.bin",
        palette_path: "palette_enemy/Drips_1.bin",
        extra_palette_paths: &["palette_enemy/Drips_2.bin", "palette_enemy/Drips_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x000, 0x0C, 0x00, 0x0F, 0x0E),
            fd(0x004, 0x0A, 0x00, 0x0E, 0x0F),
            fd(0x008, 0x08, 0x00, 0x0F, 0x0F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x17, // Hand
        art_path: "artcomp/Enemy17_Hand.bin",
        palette_path: "palette_enemy/Hand_1.bin",
        extra_palette_paths: &["palette_enemy/Hand_2.bin", "palette_enemy/Hand_3.bin"],
        frame_duration: 1,
        frames: &[fd(0x03F, 0x10, 0x01, 0x20, 0x12)],
    },
    EnemyFamilySpec {
        object_type: 0x18, // Tornado
        art_path: "artcomp/Enemy18_Tornado.bin",
        palette_path: "palette_enemy/Tornado_1.bin",
        extra_palette_paths: &["palette_enemy/Tornado_2.bin", "palette_enemy/Tornado_3.bin"],
        frame_duration: 5,
        frames: &[
            fd(0x028, 0x0E, 0x00, 0x1D, 0x28),
            fd(0x03C, 0x0D, 0x00, 0x1C, 0x27),
            fd(0x050, 0x0D, 0x00, 0x1C, 0x28),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x19, // Fireball
        art_path: "artcomp/Enemy19_Fireball.bin",
        palette_path: "palette_enemy/Fireball_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Fireball_2.bin",
            "palette_enemy/Fireball_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x064, 0x07, 0x00, 0x0E, 0x08),
            fd(0x066, 0x07, 0x00, 0x0D, 0x07),
            fd(0x068, 0x06, 0x00, 0x0C, 0x06),
            fd(0x06A, 0x07, 0x00, 0x0E, 0x08),
            fd(0x06C, 0x07, 0x00, 0x0D, 0x08),
            fd(0x06E, 0x06, 0x00, 0x0C, 0x06),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1A, // Driller
        art_path: "artcomp/Enemy1A_Driller.bin",
        palette_path: "palette_enemy/Drill_1.bin",
        extra_palette_paths: &["palette_enemy/Drill_2.bin", "palette_enemy/Drill_3.bin"],
        frame_duration: 9,
        frames: &[
            fd(0x028, 0x08, 0x00, 0x10, 0x0F),
            fd(0x02C, 0x08, 0x00, 0x10, 0x0E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1B, // Emo Rock
        art_path: "artcomp/Enemy1B_Emo_rock.bin",
        palette_path: "palette_enemy/Emo_Rock_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Emo_Rock_2.bin",
            "palette_enemy/Emo_Rock_3.bin",
        ],
        frame_duration: 20,
        frames: &[
            fd(0x010, 0x10, 0x00, 0x1D, 0x20),
            fd(0x040, 0x10, 0x00, 0x1F, 0x1C),
            fd(0x050, 0x10, 0x00, 0x1F, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1C, // Mini Hopping Skull
        art_path: "artcomp/Enemy1C_Mini_hopping_skull.bin",
        palette_path: "palette_enemy/Mini_Hopping_Skull_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Mini_Hopping_Skull_2.bin",
            "palette_enemy/Mini_Hopping_Skull_3.bin",
        ],
        frame_duration: 9,
        frames: &[
            fd(0x02C, 0x07, 0x00, 0x0F, 0x11),
            fd(0x032, 0x08, 0x00, 0x0F, 0x12),
            // ASM ingame/anim/enemy/Mini_Hopping_Skull.asm `stru_3E9A2`
            // death animation frames: LnkTo_unk_C8000/C8008.
            fd(0x000, 0x05, 0x00, 0x0D, 0x0F),
            fd(0x004, 0x05, 0x00, 0x0B, 0x0E),
            // ASM `stru_3E998` death animation frames:
            // LnkTo_unk_C8020/C8028.
            fd(0x010, 0x06, 0x00, 0x0D, 0x0E),
            fd(0x014, 0x06, 0x00, 0x0E, 0x0B),
            // ASM `stru_3E9AC` death animation frames:
            // LnkTo_unk_C8040/C8048.
            fd(0x020, 0x06, 0x00, 0x0C, 0x0F),
            fd(0x024, 0x08, 0x00, 0x10, 0x0B),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1D, // Big Hopping Skull
        art_path: "artcomp/Enemy1D_Big_hopping_skull.bin",
        palette_path: "palette_enemy/Big_Hopping_Skull_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Big_Hopping_Skull_2.bin",
            "palette_enemy/Big_Hopping_Skull_3.bin",
        ],
        frame_duration: 80,
        frames: &[fd(0x040, 0x0D, 0x00, 0x19, 0x20)],
    },
    EnemyFamilySpec {
        object_type: 0x20, // Heady Metal (boss)
        art_path: "artcomp/Enemy20_Heady_Metal.bin",
        palette_path: "palette_enemy/Boss.bin",
        extra_palette_paths: &[],
        frame_duration: 10,
        frames: &[fd(0x040, 0x0D, 0x00, 0x19, 0x20)],
    },
    EnemyFamilySpec {
        object_type: 0x21, // Boss Eyes / Attacks
        art_path: "artcomp/Enemy21_Boss_eyes_or_attacks.bin",
        palette_path: "palette_enemy/Boss_Eyes.bin",
        extra_palette_paths: &[],
        frame_duration: 6,
        frames: &[
            fd(0x000, 0x08, 0x08, 0x10, 0x10),
            fd(0x004, 0x08, 0x08, 0x10, 0x10),
            fd(0x008, 0x08, 0x08, 0x10, 0x10),
            fd(0x00C, 0x08, 0x08, 0x10, 0x10),
            fd(0x010, 0x08, 0x08, 0x10, 0x10),
            fd(0x014, 0x08, 0x08, 0x10, 0x10),
            fd(0x018, 0x08, 0x08, 0x10, 0x10),
            fd(0x01C, 0x08, 0x08, 0x10, 0x10),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x22, // Shiskaboss (head)
        art_path: "artcomp/Enemy22_Boss_head_stick_boomerangs_arrows.bin",
        palette_path: "palette_enemy/Boss.bin",
        extra_palette_paths: &[],
        frame_duration: 12,
        frames: &[
            fd(0x0A4, 0x20, 0x00, 0x40, 0x40),
            fd(0x0E4, 0x20, 0x00, 0x3B, 0x40),
            fd(0x124, 0x20, 0x00, 0x3C, 0x40),
            fd(0x164, 0x23, 0x00, 0x3F, 0x40),
            fd(0x1A4, 0x20, 0x0F, 0x3C, 0x4F),
        ],
    },
];

const HORIZONTAL_PLATFORM_FRAME: SpriteFrameDescriptor = SpriteFrameDescriptor {
    origin_x: 0,
    origin_y: 0,
    width_px: 0x48,
    height_px: 0x10,
    tile_base_index: 0,
};

const VERTICAL_PLATFORM_FRAME: SpriteFrameDescriptor = SpriteFrameDescriptor {
    origin_x: 0,
    origin_y: 0,
    width_px: 0x20,
    height_px: 0x20,
    tile_base_index: 0,
};

const FLAG_WAVING_FRAMES: [SpriteFrameDescriptor; 6] = [
    fd(0, 0x06, 0x00, 0x28, 0x20),
    fd(20, 0x06, 0x00, 0x28, 0x20),
    fd(40, 0x06, 0x00, 0x28, 0x20),
    fd(60, 0x06, 0x00, 0x28, 0x20),
    fd(80, 0x06, 0x00, 0x28, 0x20),
    fd(100, 0x06, 0x00, 0x28, 0x20),
];

const FLAG_POLE_BOTTOM_FRAME: SpriteFrameDescriptor = SpriteFrameDescriptor {
    origin_x: 0x08,
    origin_y: 0x10,
    width_px: 0x10,
    height_px: 0x10,
    tile_base_index: 0,
};

const KID_IDLE_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/kid.bin",
    origin_x: 0x0B,
    origin_y: 0x01,
    width_px: 0x14,
    height_px: 0x20,
};

const KID_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_1.bin",
        origin_x: 0x0A,
        origin_y: 0x01,
        width_px: 0x13,
        height_px: 0x1F,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_2.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_3.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_4.bin",
        origin_x: 0x0A,
        origin_y: 0x01,
        width_px: 0x14,
        height_px: 0x1F,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_5.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_6.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
];

const KID_WALK_1_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/kid_walk_1.bin", 0x0A, 0x01, 0x13, 0x1F);
const KID_WALK_2_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/kid_walk_2.bin", 0x08, 0x01, 0x11, 0x20);
const KID_WALK_4_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/kid_walk_4.bin", 0x0A, 0x01, 0x14, 0x1F);

const KID_JUMP_RISE_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/kid_jumping_1.bin",
    origin_x: 0x0A,
    origin_y: 0x00,
    width_px: 0x16,
    height_px: 0x20,
};

const KID_JUMP_FALL_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/kid_jumping_2.bin",
    origin_x: 0x09,
    origin_y: 0x00,
    width_px: 0x14,
    height_px: 0x20,
};

// ASM stru_8BB4 (kid.asm:10014): Kid wall-grab pull-up / rollover animation,
// seven frames, each held for three VBlanks before `loc_ABF0` returns to
// MoveID_Standingstill.
const KID_PULL_UP_FRAMES: [PlayerFrameDescriptor; 7] = [
    pfd("artunc_kid/kid_flip_1.bin", 0x19, 0x0D, 0x1C, 0x1D),
    pfd("artunc_kid/kid_flip_2.bin", 0x1A, 0x02, 0x20, 0x12),
    pfd("artunc_kid/kid_flip_3.bin", 0x14, 0x01, 0x1E, 0x19),
    pfd("artunc_kid/kid_flip_4.bin", 0x08, 0x01, 0x12, 0x1E),
    pfd("artunc_kid/kid_flip_5.bin", 0x12, 0x01, 0x1F, 0x19),
    pfd("artunc_kid/kid_flip_6.bin", 0x10, 0x01, 0x1F, 0x18),
    pfd("artunc_kid/kid_flip_7.bin", 0x07, 0x01, 0x11, 0x1C),
];

// ASM stru_8B6A / stru_8B74: pre-drawn tilted slope walk frames.
// Right-facing set (kid_slope_7..12) — dimensions from ROM sprite mappings.
const KID_SLOPE_WALK_RIGHT_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/kid_slope_7.bin", 0x0B, 0x00, 0x17, 0x1F),
    pfd("artunc_kid/kid_slope_8.bin", 0x0C, -0x02, 0x16, 0x1E),
    pfd("artunc_kid/kid_slope_9.bin", 0x0A, -0x01, 0x15, 0x20),
    pfd("artunc_kid/kid_slope_10.bin", 0x0A, 0x01, 0x16, 0x20),
    pfd("artunc_kid/kid_slope_11.bin", 0x0A, -0x02, 0x16, 0x1E),
    pfd("artunc_kid/kid_slope_12.bin", 0x0A, -0x02, 0x16, 0x1F),
];

// Left-facing set (kid_slope_1..6).
const KID_SLOPE_WALK_LEFT_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/kid_slope_1.bin", 0x0B, 0x06, 0x13, 0x21),
    pfd("artunc_kid/kid_slope_2.bin", 0x08, 0x04, 0x10, 0x20),
    pfd("artunc_kid/kid_slope_3.bin", 0x08, 0x06, 0x10, 0x21),
    pfd("artunc_kid/kid_slope_4.bin", 0x09, 0x07, 0x12, 0x22),
    pfd("artunc_kid/kid_slope_5.bin", 0x08, 0x04, 0x10, 0x20),
    pfd("artunc_kid/kid_slope_6.bin", 0x08, 0x04, 0x10, 0x20),
];

const IRON_KNIGHT_IDLE_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/iron_knight.bin",
    origin_x: 0x13,
    origin_y: 0x01,
    width_px: 0x1E,
    height_px: 0x26,
};

const IRON_KNIGHT_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/iron_knight_walk_1.bin",
        origin_x: 0x10,
        origin_y: 0x01,
        width_px: 0x1B,
        height_px: 0x25,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/iron_knight_walk_2.bin",
        origin_x: 0x11,
        origin_y: 0x01,
        width_px: 0x1C,
        height_px: 0x26,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/iron_knight_walk_3.bin",
        origin_x: 0x11,
        origin_y: 0x01,
        width_px: 0x1C,
        height_px: 0x25,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/iron_knight_walk_4.bin",
        origin_x: 0x11,
        origin_y: 0x01,
        width_px: 0x1D,
        height_px: 0x24,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/iron_knight_walk_5.bin",
        origin_x: 0x11,
        origin_y: 0x01,
        width_px: 0x1C,
        height_px: 0x25,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/iron_knight_walk_6.bin",
        origin_x: 0x12,
        origin_y: 0x01,
        width_px: 0x1D,
        height_px: 0x25,
    },
];

const IRON_KNIGHT_JUMP_RISE_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/iron_knight_jumping_1.bin",
    origin_x: 0x14,
    origin_y: 0x01,
    width_px: 0x1F,
    height_px: 0x24,
};

const IRON_KNIGHT_JUMP_FALL_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/iron_knight_jumping_2.bin",
    origin_x: 0x0D,
    origin_y: -0x02,
    width_px: 0x1A,
    height_px: 0x21,
};

const IRON_KNIGHT_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 16] = [
    PlayerTransformStepDescriptor {
        duration_frames: 8,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_1.bin",
            origin_x: 0x11,
            origin_y: 0x07,
            width_px: 0x23,
            height_px: 0x32,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 7,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_2.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_1.bin",
            origin_x: 0x11,
            origin_y: 0x07,
            width_px: 0x23,
            height_px: 0x32,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_2.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 5,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_3.bin",
            origin_x: 0x17,
            origin_y: 0x11,
            width_px: 0x2F,
            height_px: 0x3F,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 5,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_2.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 5,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_3.bin",
            origin_x: 0x17,
            origin_y: 0x11,
            width_px: 0x2F,
            height_px: 0x3F,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 5,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_4.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_3.bin",
            origin_x: 0x17,
            origin_y: 0x11,
            width_px: 0x2F,
            height_px: 0x3F,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_4.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_3.bin",
            origin_x: 0x17,
            origin_y: 0x11,
            width_px: 0x2F,
            height_px: 0x3F,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_4.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: None,
    },
    PlayerTransformStepDescriptor {
        duration_frames: 6,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_4.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
    PlayerTransformStepDescriptor {
        duration_frames: 10,
        frame: None,
    },
    PlayerTransformStepDescriptor {
        duration_frames: 4,
        frame: Some(PlayerFrameDescriptor {
            relative_path: "artunc_kid/iron_knight_transformation_4.bin",
            origin_x: 0x14,
            origin_y: 0x0B,
            width_px: 0x26,
            height_px: 0x36,
        }),
    },
];

const SKYCUTTER_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/skycutter.bin", 0x11, 0x00, 0x1F, 0x1F);
const SKYCUTTER_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    SKYCUTTER_IDLE_FRAME,
    pfd("artunc_kid/skycutter_walk_2.bin", 0x11, 0x00, 0x1F, 0x1E),
    pfd("artunc_kid/skycutter_walk_3.bin", 0x11, 0x00, 0x1F, 0x1E),
    pfd("artunc_kid/skycutter_walk_4.bin", 0x11, 0x00, 0x1F, 0x1D),
    pfd("artunc_kid/skycutter_walk_5.bin", 0x11, 0x00, 0x1F, 0x1E),
    pfd("artunc_kid/skycutter_walk_6.bin", 0x11, 0x00, 0x1F, 0x1F),
];
const SKYCUTTER_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 13] = [
    tfd(8, KID_WALK_2_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/skycutter_transformation_1.bin",
            0x0E,
            0x01,
            0x19,
            0x20,
        ),
    ),
    tfd(4, KID_WALK_2_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/skycutter_transformation_1.bin",
            0x0E,
            0x01,
            0x19,
            0x20,
        ),
    ),
    tfd(4, KID_WALK_2_FRAME),
    tfd(
        2,
        pfd(
            "artunc_kid/skycutter_transformation_1.bin",
            0x0E,
            0x01,
            0x19,
            0x20,
        ),
    ),
    tfd(
        2,
        pfd(
            "artunc_kid/skycutter_transformation_2.bin",
            0x13,
            0x01,
            0x23,
            0x26,
        ),
    ),
    tfd(
        2,
        pfd(
            "artunc_kid/skycutter_transformation_3.bin",
            0x16,
            0x01,
            0x2B,
            0x3F,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/skycutter_transformation_2.bin",
            0x13,
            0x01,
            0x23,
            0x26,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/skycutter_transformation_3.bin",
            0x16,
            0x01,
            0x2B,
            0x3F,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/skycutter_transformation_2.bin",
            0x13,
            0x01,
            0x23,
            0x26,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/skycutter_transformation_1.bin",
            0x0E,
            0x01,
            0x19,
            0x20,
        ),
    ),
    tfd(
        5,
        pfd(
            "artunc_kid/skycutter_transformation_4.bin",
            0x13,
            0x01,
            0x20,
            0x20,
        ),
    ),
];

const CYCLONE_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/cyclone.bin", 0x0C, 0x01, 0x16, 0x20);
const CYCLONE_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/cyclone_walk_1.bin", 0x0C, 0x01, 0x17, 0x20),
    pfd("artunc_kid/cyclone_walk_2.bin", 0x0C, 0x01, 0x17, 0x20),
    pfd("artunc_kid/cyclone_walk_3.bin", 0x0C, 0x01, 0x17, 0x1F),
    pfd("artunc_kid/cyclone_walk_4.bin", 0x0D, 0x01, 0x17, 0x20),
    pfd("artunc_kid/cyclone_walk_5.bin", 0x0D, 0x01, 0x17, 0x20),
    pfd("artunc_kid/cyclone_walk_6.bin", 0x0D, 0x01, 0x17, 0x1F),
];
const CYCLONE_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 10] = [
    tfd(16, KID_WALK_1_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/cyclone_transformation_1.bin",
            0x0F,
            0x01,
            0x1D,
            0x26,
        ),
    ),
    tfd(8, KID_WALK_1_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/cyclone_transformation_1.bin",
            0x0F,
            0x01,
            0x1D,
            0x26,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/cyclone_transformation_2.bin",
            0x0F,
            0x01,
            0x1F,
            0x26,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/cyclone_transformation_1.bin",
            0x0F,
            0x01,
            0x1D,
            0x26,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/cyclone_transformation_2.bin",
            0x0F,
            0x01,
            0x1F,
            0x26,
        ),
    ),
    blank_tfd(8),
    tfd(
        4,
        pfd(
            "artunc_kid/cyclone_transformation_2.bin",
            0x0F,
            0x01,
            0x1F,
            0x26,
        ),
    ),
    blank_tfd(16),
];

const RED_STEALTH_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/red_stealth.bin", 0x10, 0x01, 0x1E, 0x20);
const RED_STEALTH_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/red_stealth_walk_1.bin", 0x0E, 0x01, 0x17, 0x21),
    pfd("artunc_kid/red_stealth_walk_2.bin", 0x0E, 0x01, 0x17, 0x22),
    pfd("artunc_kid/red_stealth_walk_3.bin", 0x0E, 0x01, 0x18, 0x23),
    pfd("artunc_kid/red_stealth_walk_4.bin", 0x0E, 0x01, 0x17, 0x26),
    pfd("artunc_kid/red_stealth_walk_5.bin", 0x0F, 0x01, 0x18, 0x24),
    pfd("artunc_kid/red_stealth_walk_6.bin", 0x0F, 0x01, 0x18, 0x22),
];
// ASM off_A23C: Red Stealth slope animation repeats two pre-drawn tilted
// frames in a six-step cycle. The ROM only has one Red Stealth slope art set;
// native uses the same set for both facing buckets and applies sprite H-flip.
const RED_STEALTH_SLOPE_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/red_stealth_slope_1.bin", 0x0B, 0x05, 0x1D, 0x21),
    pfd("artunc_kid/red_stealth_slope_2.bin", 0x0C, 0x04, 0x1C, 0x22),
    pfd("artunc_kid/red_stealth_slope_2.bin", 0x0C, 0x04, 0x1C, 0x22),
    pfd("artunc_kid/red_stealth_slope_1.bin", 0x0B, 0x05, 0x1D, 0x21),
    pfd("artunc_kid/red_stealth_slope_2.bin", 0x0C, 0x04, 0x1C, 0x22),
    pfd("artunc_kid/red_stealth_slope_2.bin", 0x0C, 0x04, 0x1C, 0x22),
];
const RED_STEALTH_SLASH_GROUND_FRAMES: [PlayerFrameDescriptor; 4] = [
    pfd("artunc_kid/red_stealth_slash_1.bin", 0x0E, 0x01, 0x18, 0x26),
    pfd("artunc_kid/red_stealth_slash_2.bin", 0x1A, 0x01, 0x25, 0x21),
    pfd("artunc_kid/red_stealth_slash_3.bin", 0x19, 0x01, 0x23, 0x20),
    pfd("artunc_kid/red_stealth_slash_4.bin", 0x0E, 0x01, 0x2E, 0x1F),
];
const RED_STEALTH_SLASH_AIR_FRAMES: [PlayerFrameDescriptor; 4] = [
    pfd(
        "artunc_kid/red_stealth_slash_air_1.bin",
        0x0C,
        0x01,
        0x17,
        0x28,
    ),
    pfd(
        "artunc_kid/red_stealth_slash_air_2.bin",
        0x0B,
        0x01,
        0x23,
        0x27,
    ),
    pfd(
        "artunc_kid/red_stealth_slash_air_3.bin",
        0x0C,
        0x04,
        0x19,
        0x28,
    ),
    pfd(
        "artunc_kid/red_stealth_slash_air_4.bin",
        0x0D,
        0x06,
        0x17,
        0x2A,
    ),
];
const RED_STEALTH_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 13] = [
    tfd(
        4,
        pfd(
            "artunc_kid/red_stealth_transformation_1.bin",
            0x18,
            0x13,
            0x2F,
            0x4E,
        ),
    ),
    tfd(8, KID_WALK_1_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/red_stealth_transformation_1.bin",
            0x18,
            0x13,
            0x2F,
            0x4E,
        ),
    ),
    tfd(4, KID_WALK_1_FRAME),
    tfd(
        2,
        pfd(
            "artunc_kid/red_stealth_transformation_1.bin",
            0x18,
            0x13,
            0x2F,
            0x4E,
        ),
    ),
    tfd(
        2,
        pfd(
            "artunc_kid/red_stealth_transformation_2.bin",
            0x18,
            0x11,
            0x2E,
            0x49,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/red_stealth_transformation_1.bin",
            0x18,
            0x13,
            0x2F,
            0x4E,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/red_stealth_transformation_2.bin",
            0x18,
            0x11,
            0x2E,
            0x49,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/red_stealth_transformation_1.bin",
            0x18,
            0x13,
            0x2F,
            0x4E,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/red_stealth_transformation_2.bin",
            0x18,
            0x11,
            0x2E,
            0x49,
        ),
    ),
    blank_tfd(3),
    tfd(
        5,
        pfd(
            "artunc_kid/red_stealth_transformation_2.bin",
            0x18,
            0x11,
            0x2E,
            0x49,
        ),
    ),
    blank_tfd(8),
];

const EYECLOPS_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/eyeclops.bin", 0x12, 0x01, 0x20, 0x21);
const EYECLOPS_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/eyeclops_walk_1.bin", 0x0C, 0x01, 0x1F, 0x21),
    pfd("artunc_kid/eyeclops_walk_2.bin", 0x0C, 0x01, 0x20, 0x20),
    pfd("artunc_kid/eyeclops_walk_3.bin", 0x0C, 0x01, 0x1F, 0x21),
    pfd("artunc_kid/eyeclops_walk_4.bin", 0x0C, 0x01, 0x1F, 0x21),
    pfd("artunc_kid/eyeclops_walk_5.bin", 0x0C, 0x01, 0x1F, 0x20),
    pfd("artunc_kid/eyeclops_walk_6.bin", 0x0C, 0x01, 0x1F, 0x21),
];
const EYECLOPS_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 14] = [
    tfd(24, KID_WALK_1_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_1.bin",
            0x17,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    tfd(12, KID_WALK_1_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_1.bin",
            0x17,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    tfd(6, KID_WALK_1_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_1.bin",
            0x17,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_2.bin",
            0x18,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_3.bin",
            0x18,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_2.bin",
            0x18,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_3.bin",
            0x18,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    blank_tfd(8),
    tfd(
        4,
        pfd(
            "artunc_kid/eyeclops_transformation_3.bin",
            0x18,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
    blank_tfd(8),
    tfd(
        3,
        pfd(
            "artunc_kid/eyeclops_transformation_3.bin",
            0x18,
            0x02,
            0x2F,
            0x2F,
        ),
    ),
];

const JUGGERNAUT_HEAD_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/juggernaut_head.bin", 0x13, -0x0A, 0x1A, 0x15);
const JUGGERNAUT_HEAD_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    JUGGERNAUT_HEAD_IDLE_FRAME,
    pfd(
        "artunc_kid/juggernaut_head_walk_2.bin",
        0x13,
        -0x0A,
        0x1A,
        0x15,
    ),
    pfd(
        "artunc_kid/juggernaut_head_walk_3.bin",
        0x13,
        -0x0A,
        0x1A,
        0x15,
    ),
    pfd(
        "artunc_kid/juggernaut_head_walk_4.bin",
        0x13,
        -0x0A,
        0x1A,
        0x15,
    ),
    pfd(
        "artunc_kid/juggernaut_head_walk_5.bin",
        0x13,
        -0x0A,
        0x1A,
        0x15,
    ),
    pfd(
        "artunc_kid/juggernaut_head_walk_6.bin",
        0x13,
        -0x0A,
        0x1A,
        0x15,
    ),
];
const JUGGERNAUT_BOTTOM_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/juggernaut_bottom.bin", 0x17, 0x01, 0x2F, 0x0D);
const JUGGERNAUT_BOTTOM_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    JUGGERNAUT_BOTTOM_IDLE_FRAME,
    pfd(
        "artunc_kid/juggernaut_bottom_move_1.bin",
        0x17,
        0x01,
        0x30,
        0x0F,
    ),
    pfd(
        "artunc_kid/juggernaut_bottom_move_2.bin",
        0x17,
        0x01,
        0x30,
        0x0F,
    ),
    pfd(
        "artunc_kid/juggernaut_bottom_move_3.bin",
        0x17,
        0x01,
        0x30,
        0x0F,
    ),
    pfd(
        "artunc_kid/juggernaut_bottom_move_4.bin",
        0x17,
        0x01,
        0x30,
        0x0F,
    ),
    pfd(
        "artunc_kid/juggernaut_bottom_move_1.bin",
        0x17,
        0x01,
        0x30,
        0x0F,
    ),
];
const JUGGERNAUT_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 11] = [
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_1.bin",
            0x10,
            0x05,
            0x22,
            0x28,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_2.bin",
            0x11,
            0x05,
            0x23,
            0x28,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_3.bin",
            0x11,
            0x05,
            0x23,
            0x2A,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_4.bin",
            0x17,
            0x11,
            0x2E,
            0x3E,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_3.bin",
            0x11,
            0x05,
            0x23,
            0x2A,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_2.bin",
            0x11,
            0x05,
            0x23,
            0x28,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_1.bin",
            0x10,
            0x05,
            0x22,
            0x28,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_2.bin",
            0x11,
            0x05,
            0x23,
            0x28,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_3.bin",
            0x11,
            0x05,
            0x23,
            0x2A,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_4.bin",
            0x17,
            0x11,
            0x2E,
            0x3E,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/juggernaut_transformation_3.bin",
            0x11,
            0x05,
            0x23,
            0x2A,
        ),
    ),
];

const BERZERKER_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/berzerker.bin", 0x15, 0x01, 0x1F, 0x25);
const BERZERKER_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/berzerker_walk_1.bin", 0x0F, 0x01, 0x1B, 0x22),
    pfd("artunc_kid/berzerker_walk_2.bin", 0x0F, 0x01, 0x1B, 0x23),
    pfd("artunc_kid/berzerker_walk_3.bin", 0x0E, 0x01, 0x1A, 0x23),
    pfd("artunc_kid/berzerker_walk_4.bin", 0x0D, 0x01, 0x19, 0x22),
    pfd("artunc_kid/berzerker_walk_5.bin", 0x0F, 0x01, 0x1B, 0x23),
    pfd("artunc_kid/berzerker_walk_6.bin", 0x0F, 0x01, 0x1B, 0x23),
];
const BERZERKER_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 10] = [
    tfd(15, KID_WALK_4_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/berzerker_transformation_1.bin",
            0x0C,
            0x01,
            0x1B,
            0x23,
        ),
    ),
    tfd(4, KID_WALK_4_FRAME),
    tfd(
        2,
        pfd(
            "artunc_kid/berzerker_transformation_1.bin",
            0x0C,
            0x01,
            0x1B,
            0x23,
        ),
    ),
    tfd(
        2,
        pfd(
            "artunc_kid/berzerker_transformation_2.bin",
            0x0D,
            0x01,
            0x1C,
            0x22,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/berzerker_transformation_1.bin",
            0x0C,
            0x01,
            0x1B,
            0x23,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/berzerker_transformation_2.bin",
            0x0D,
            0x01,
            0x1C,
            0x22,
        ),
    ),
    blank_tfd(3),
    tfd(
        5,
        pfd(
            "artunc_kid/berzerker_transformation_2.bin",
            0x0D,
            0x01,
            0x1C,
            0x22,
        ),
    ),
    blank_tfd(8),
];

const MANIAXE_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/maniaxe.bin", 0x0E, 0x01, 0x1D, 0x20);
const MANIAXE_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/maniaxe_walk_1.bin", 0x11, 0x01, 0x1C, 0x20),
    pfd("artunc_kid/maniaxe_walk_2.bin", 0x0F, 0x01, 0x19, 0x21),
    pfd("artunc_kid/maniaxe_walk_3.bin", 0x0B, 0x01, 0x15, 0x21),
    pfd("artunc_kid/maniaxe_walk_4.bin", 0x0B, 0x01, 0x18, 0x20),
    pfd("artunc_kid/maniaxe_walk_5.bin", 0x0B, 0x01, 0x16, 0x21),
    pfd("artunc_kid/maniaxe_walk_6.bin", 0x0D, 0x01, 0x16, 0x20),
];
const MANIAXE_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 9] = [
    tfd(
        8,
        pfd(
            "artunc_kid/maniaxe_transformation_1.bin",
            0x0B,
            0x01,
            0x19,
            0x21,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/maniaxe_transformation_2.bin",
            0x0E,
            0x01,
            0x20,
            0x22,
        ),
    ),
    tfd(
        6,
        pfd(
            "artunc_kid/maniaxe_transformation_1.bin",
            0x0B,
            0x01,
            0x19,
            0x21,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/maniaxe_transformation_2.bin",
            0x0E,
            0x01,
            0x20,
            0x22,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/maniaxe_transformation_3.bin",
            0x11,
            0x01,
            0x22,
            0x23,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/maniaxe_transformation_2.bin",
            0x0E,
            0x01,
            0x20,
            0x22,
        ),
    ),
    tfd(
        6,
        pfd(
            "artunc_kid/maniaxe_transformation_3.bin",
            0x11,
            0x01,
            0x22,
            0x23,
        ),
    ),
    tfd(
        3,
        pfd(
            "artunc_kid/maniaxe_transformation_2.bin",
            0x0E,
            0x01,
            0x20,
            0x22,
        ),
    ),
    tfd(
        8,
        pfd(
            "artunc_kid/maniaxe_transformation_3.bin",
            0x11,
            0x01,
            0x22,
            0x23,
        ),
    ),
];

const MICROMAX_IDLE_FRAME: PlayerFrameDescriptor =
    pfd("artunc_kid/micromax.bin", 0x07, 0x01, 0x0E, 0x12);
const MICROMAX_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    pfd("artunc_kid/micromax_walk_1.bin", 0x0A, 0x01, 0x10, 0x11),
    pfd("artunc_kid/micromax_walk_2.bin", 0x09, 0x01, 0x0F, 0x12),
    pfd("artunc_kid/micromax_walk_3.bin", 0x09, 0x01, 0x0F, 0x12),
    pfd("artunc_kid/micromax_walk_4.bin", 0x09, 0x01, 0x0F, 0x11),
    pfd("artunc_kid/micromax_walk_5.bin", 0x08, 0x01, 0x0E, 0x12),
    pfd("artunc_kid/micromax_walk_6.bin", 0x09, 0x01, 0x0F, 0x12),
];
const MICROMAX_TRANSFORM_STEPS: [PlayerTransformStepDescriptor; 12] = [
    tfd(15, KID_WALK_1_FRAME),
    tfd(
        4,
        pfd(
            "artunc_kid/micromax_transformation_1.bin",
            0x0F,
            0x01,
            0x20,
            0x22,
        ),
    ),
    tfd(4, KID_WALK_4_FRAME),
    tfd(
        2,
        pfd(
            "artunc_kid/micromax_transformation_1.bin",
            0x0F,
            0x01,
            0x20,
            0x22,
        ),
    ),
    tfd(
        2,
        pfd(
            "artunc_kid/micromax_transformation_2.bin",
            0x0F,
            0x01,
            0x1F,
            0x1E,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/micromax_transformation_3.bin",
            0x0D,
            0x01,
            0x1A,
            0x18,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/micromax_transformation_2.bin",
            0x0F,
            0x01,
            0x1F,
            0x1E,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/micromax_transformation_3.bin",
            0x0D,
            0x01,
            0x1A,
            0x18,
        ),
    ),
    tfd(
        4,
        pfd(
            "artunc_kid/micromax_transformation_2.bin",
            0x0F,
            0x01,
            0x1F,
            0x1E,
        ),
    ),
    blank_tfd(3),
    tfd(
        5,
        pfd(
            "artunc_kid/micromax_transformation_3.bin",
            0x0D,
            0x01,
            0x1A,
            0x18,
        ),
    ),
    blank_tfd(8),
];

static PLAYER_FORM_SPECS: &[PlayerFormSpec] = &[
    PlayerFormSpec {
        form_id: PLAYER_FORM_THE_KID,
        palette_path: KID_PALETTE_PATH,
        palette_family: "Kid palette",
        idle: KID_IDLE_FRAME,
        walk: &KID_WALK_FRAMES,
        slope_walk: &KID_SLOPE_WALK_RIGHT_FRAMES,
        slope_walk_left: &KID_SLOPE_WALK_LEFT_FRAMES,
        jump_rise: KID_JUMP_RISE_FRAME,
        jump_fall: KID_JUMP_FALL_FRAME,
        pull_up: &KID_PULL_UP_FRAMES,
        slash_ground: &[],
        slash_air: &[],
        transform_palette: None,
        transform_steps: &[],
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_SKYCUTTER,
        palette_path: SKYCUTTER_PALETTE_PATH,
        palette_family: "Skycutter palette",
        idle: SKYCUTTER_IDLE_FRAME,
        walk: &SKYCUTTER_WALK_FRAMES,
        slope_walk: &[],
        slope_walk_left: &[],
        jump_rise: SKYCUTTER_IDLE_FRAME,
        jump_fall: SKYCUTTER_IDLE_FRAME,
        pull_up: &[],
        slash_ground: &[],
        slash_air: &[],
        transform_palette: Some((
            SKYCUTTER_TRANSFORM_PALETTE_PATH,
            "Skycutter transform palette",
        )),
        transform_steps: &SKYCUTTER_TRANSFORM_STEPS,
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_CYCLONE,
        palette_path: CYCLONE_PALETTE_PATH,
        palette_family: "Cyclone palette",
        idle: CYCLONE_IDLE_FRAME,
        walk: &CYCLONE_WALK_FRAMES,
        slope_walk: &[],
        slope_walk_left: &[],
        jump_rise: pfd("artunc_kid/cyclone_jumping_1.bin", 0x0D, 0x00, 0x17, 0x22),
        jump_fall: pfd("artunc_kid/cyclone_jumping_3.bin", 0x0C, 0x00, 0x16, 0x23),
        pull_up: &[],
        slash_ground: &[],
        slash_air: &[],
        transform_palette: Some((CYCLONE_TRANSFORM_PALETTE_PATH, "Cyclone transform palette")),
        transform_steps: &CYCLONE_TRANSFORM_STEPS,
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_RED_STEALTH,
        palette_path: RED_STEALTH_PALETTE_PATH,
        palette_family: "Red Stealth palette",
        idle: RED_STEALTH_IDLE_FRAME,
        walk: &RED_STEALTH_WALK_FRAMES,
        slope_walk: &RED_STEALTH_SLOPE_WALK_FRAMES,
        slope_walk_left: &RED_STEALTH_SLOPE_WALK_FRAMES,
        jump_rise: pfd(
            "artunc_kid/red_stealth_jumping_1.bin",
            0x0B,
            0x02,
            0x16,
            0x28,
        ),
        jump_fall: pfd(
            "artunc_kid/red_stealth_jumping_2.bin",
            0x0A,
            0x00,
            0x15,
            0x25,
        ),
        pull_up: &[],
        slash_ground: &RED_STEALTH_SLASH_GROUND_FRAMES,
        slash_air: &RED_STEALTH_SLASH_AIR_FRAMES,
        transform_palette: Some((
            RED_STEALTH_TRANSFORM_PALETTE_PATH,
            "Red Stealth transform palette",
        )),
        transform_steps: &RED_STEALTH_TRANSFORM_STEPS,
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_EYECLOPS,
        palette_path: EYECLOPS_PALETTE_PATH,
        palette_family: "Eyeclops palette",
        idle: EYECLOPS_IDLE_FRAME,
        walk: &EYECLOPS_WALK_FRAMES,
        slope_walk: &[],
        slope_walk_left: &[],
        jump_rise: pfd("artunc_kid/eyeclops_jumping_1.bin", 0x10, 0x00, 0x1C, 0x25),
        jump_fall: pfd("artunc_kid/eyeclops_jumping_2.bin", 0x0A, 0x00, 0x19, 0x21),
        pull_up: &[],
        slash_ground: &[],
        slash_air: &[],
        transform_palette: Some((
            EYECLOPS_TRANSFORM_PALETTE_PATH,
            "Eyeclops transform palette",
        )),
        transform_steps: &EYECLOPS_TRANSFORM_STEPS,
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_IRON_KNIGHT,
        palette_path: IRON_KNIGHT_PALETTE_PATH,
        palette_family: "Iron Knight palette",
        idle: IRON_KNIGHT_IDLE_FRAME,
        walk: &IRON_KNIGHT_WALK_FRAMES,
        slope_walk: &[],
        slope_walk_left: &[],
        jump_rise: IRON_KNIGHT_JUMP_RISE_FRAME,
        jump_fall: IRON_KNIGHT_JUMP_FALL_FRAME,
        pull_up: &[],
        slash_ground: &[],
        slash_air: &[],
        transform_palette: Some((
            IRON_KNIGHT_TRANSFORM_PALETTE_PATH,
            "Iron Knight transform palette",
        )),
        transform_steps: &IRON_KNIGHT_TRANSFORM_STEPS,
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_BERZERKER,
        palette_path: BERZERKER_PALETTE_PATH,
        palette_family: "Berzerker palette",
        idle: BERZERKER_IDLE_FRAME,
        walk: &BERZERKER_WALK_FRAMES,
        slope_walk: &[],
        slope_walk_left: &[],
        jump_rise: pfd("artunc_kid/berzerker_jumping_1.bin", 0x11, 0x00, 0x20, 0x1F),
        jump_fall: pfd("artunc_kid/berzerker_jumping_3.bin", 0x11, 0x00, 0x1F, 0x21),
        pull_up: &[],
        slash_ground: &[],
        slash_air: &[],
        transform_palette: Some((
            BERZERKER_TRANSFORM_PALETTE_PATH,
            "Berzerker transform palette",
        )),
        transform_steps: &BERZERKER_TRANSFORM_STEPS,
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_MANIAXE,
        palette_path: MANIAXE_PALETTE_PATH,
        palette_family: "Maniaxe palette",
        idle: MANIAXE_IDLE_FRAME,
        walk: &MANIAXE_WALK_FRAMES,
        slope_walk: &[],
        slope_walk_left: &[],
        jump_rise: pfd("artunc_kid/maniaxe_jumping_1.bin", 0x0E, 0x02, 0x1C, 0x23),
        jump_fall: pfd("artunc_kid/maniaxe_jumping_2.bin", 0x0F, -0x02, 0x1C, 0x20),
        pull_up: &[],
        slash_ground: &[],
        slash_air: &[],
        transform_palette: Some((MANIAXE_TRANSFORM_PALETTE_PATH, "Maniaxe transform palette")),
        transform_steps: &MANIAXE_TRANSFORM_STEPS,
    },
    PlayerFormSpec {
        form_id: PLAYER_FORM_MICROMAX,
        palette_path: MICROMAX_PALETTE_PATH,
        palette_family: "Micromax palette",
        idle: MICROMAX_IDLE_FRAME,
        walk: &MICROMAX_WALK_FRAMES,
        slope_walk: &[],
        slope_walk_left: &[],
        jump_rise: pfd("artunc_kid/micromax_jumping_1.bin", 0x09, 0x00, 0x10, 0x14),
        jump_fall: pfd("artunc_kid/micromax_jumping_3.bin", 0x09, 0x00, 0x10, 0x12),
        pull_up: &[],
        slash_ground: &[],
        slash_air: &[],
        transform_palette: Some((
            MICROMAX_TRANSFORM_PALETTE_PATH,
            "Micromax transform palette",
        )),
        transform_steps: &MICROMAX_TRANSFORM_STEPS,
    },
];

fn load_enemy_visuals(
    manifest: &AssetManifest,
    level_theme: u8,
    enemy_layout: Option<&EnemyLayout>,
) -> EnemyVisuals {
    let mut families = std::collections::HashMap::new();
    for spec in ENEMY_FAMILIES {
        let palette = match load_ingame_palette_asset(manifest, spec.palette_path, spec.art_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skipping enemy 0x{:02X} sprite: {e}", spec.object_type);
                continue;
            }
        };
        let mut frames = match load_compressed_sprite_frames(
            manifest,
            spec.art_path,
            &format!("enemy 0x{:02X}", spec.object_type),
            spec.frames,
        ) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("skipping enemy 0x{:02X} art: {e}", spec.object_type);
                continue;
            }
        };
        if let Some(permutation) =
            forest_enemy_art_permutation(level_theme, enemy_layout, spec.object_type)
        {
            apply_enemy_art_permutation(&mut frames, permutation);
        }
        let extra_palettes = spec
            .extra_palette_paths
            .iter()
            .filter_map(|path| load_ingame_palette_asset(manifest, path, spec.art_path).ok())
            .collect();
        families.insert(
            spec.object_type,
            AnimatedSpriteVisual {
                palette,
                extra_palettes,
                frames,
                frame_duration: spec.frame_duration,
            },
        );
    }
    EnemyVisuals { families }
}

fn forest_enemy_art_permutation(
    level_theme: u8,
    enemy_layout: Option<&EnemyLayout>,
    object_type: u8,
) -> Option<&'static [u8; 16]> {
    if level_theme != FOREST_THEME_ID {
        return None;
    }
    let first_slot = *enemy_layout?.enemy_type_slots.first()?;
    if first_slot == 0xFFFF || first_slot & 0x0FFF != u16::from(object_type) {
        return None;
    }
    if object_type == 0x05 {
        Some(&FOREST_TAR_MONSTER_ENEMY_ART_PERMUTATION)
    } else {
        Some(&FOREST_ENEMY_ART_PERMUTATION)
    }
}

fn apply_enemy_art_permutation(frames: &mut [SpriteFrame], permutation: &[u8; 16]) {
    for frame in frames {
        frame.tile_sheet.apply_pixel_permutation(permutation);
    }
}

fn load_floating_prize_visuals(
    manifest: &AssetManifest,
    foreground_palette: &Palette,
    hud_diamond_tiles: &TileSheet,
) -> Result<FloatingPrizeVisuals> {
    let diamond_frames = if FLOATING_DIAMOND_ART_PATHS
        .iter()
        .all(|path| find_ingame_asset(manifest, path).is_some())
    {
        FLOATING_DIAMOND_ART_PATHS
            .iter()
            .map(|path| {
                Ok(SpriteFrame {
                    // ASM PrizeDiamond_Init selects `stru_10D6E`, whose frames
                    // point at `unk_E0E66: sprite_frame_vram $6F2,$06,$FE,$10,$0C`.
                    // `sub_5D4A` DMAs diamond_1..4 to VRAM $DD40 (tile $6EA),
                    // so $6F2 is tile offset 8 inside each animation art page.
                    origin_x: 0x06,
                    origin_y: -2,
                    width_px: 0x10,
                    height_px: 0x0C,
                    tile_base_index: 0x06F2 - HUD_DIAMOND_BASE_TILE_INDEX,
                    tile_sheet: load_uncompressed_tile_sheet(
                        manifest,
                        "ingame",
                        find_ingame_asset(manifest, path)
                            .ok_or_else(|| anyhow!("missing floating diamond art ingame/{path}"))?,
                        &format!("floating diamond art {path}"),
                    )?,
                })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![SpriteFrame {
            // Test fixtures may only include the first HUD diamond page. Keep
            // the same ASM frame geometry while using that available page.
            origin_x: 0x06,
            origin_y: -2,
            width_px: 0x10,
            height_px: 0x0C,
            tile_base_index: 0x06F2 - HUD_DIAMOND_BASE_TILE_INDEX,
            tile_sheet: hud_diamond_tiles.clone(),
        }]
    };
    let diamond = AnimatedSpriteVisual {
        palette: foreground_palette.clone(),
        extra_palettes: Vec::new(),
        frames: diamond_frames,
        frame_duration: 3,
    };

    let mut helmets = std::collections::HashMap::new();
    for spec in FLOATING_PRIZE_HELMETS {
        if find_ingame_asset(manifest, spec.art.relative_path).is_none()
            || find_ingame_asset(manifest, spec.palette_path).is_none()
        {
            continue;
        }
        helmets.insert(
            spec.content_id,
            AnimatedSpriteVisual {
                palette: load_floating_prize_helmet_palette_asset(
                    manifest,
                    spec.base_palette_path,
                    spec.palette_path,
                    spec.palette_family,
                )?,
                extra_palettes: Vec::new(),
                frames: vec![load_player_frame(manifest, spec.art)?],
                frame_duration: 1,
            },
        );
    }

    let mut icons = std::collections::HashMap::new();
    for spec in FLOATING_PRIZE_ICONS {
        if find_ingame_asset(manifest, spec.art.relative_path).is_none() {
            continue;
        }
        icons.insert(
            spec.content_id,
            AnimatedSpriteVisual {
                palette: foreground_palette.clone(),
                extra_palettes: Vec::new(),
                frames: vec![load_player_frame(manifest, spec.art)?],
                frame_duration: 1,
            },
        );
    }

    Ok(FloatingPrizeVisuals {
        diamond,
        helmets,
        icons,
    })
}

fn load_platform_visuals(
    manifest: &AssetManifest,
    foreground_theme: &str,
) -> Result<PlatformVisuals> {
    let palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("palette_fg/{foreground_theme}.bin")).ok_or_else(
            || anyhow!("missing platform palette asset theme/palette_fg/{foreground_theme}.bin"),
        )?,
        "platform palette",
    )?)?;

    Ok(PlatformVisuals {
        palette,
        horizontal: load_compressed_sprite_frame(
            manifest,
            HORIZONTAL_PLATFORM_ART_PATH,
            "horizontal platform art",
            HORIZONTAL_PLATFORM_FRAME,
        )?,
        vertical: load_compressed_sprite_frame(
            manifest,
            VERTICAL_PLATFORM_ART_PATH,
            "vertical platform art",
            VERTICAL_PLATFORM_FRAME,
        )?,
    })
}

fn load_player_visuals(manifest: &AssetManifest) -> Result<PlayerVisuals> {
    let mut forms = std::collections::HashMap::new();
    for spec in PLAYER_FORM_SPECS {
        let Some(visuals) = load_player_form_visuals_if_present(manifest, spec)? else {
            if spec.form_id == PLAYER_FORM_THE_KID {
                bail!("missing required Kid visuals");
            }
            if !cfg!(test) {
                eprintln!(
                    "skipping player form 0x{:02X}: missing {}",
                    spec.form_id, spec.palette_family
                );
            }
            continue;
        };
        forms.insert(spec.form_id, visuals);
    }
    if let Some(visuals) = load_juggernaut_form_visuals_if_present(manifest)? {
        forms.insert(PLAYER_FORM_JUGGERNAUT, visuals);
    }

    Ok(PlayerVisuals { forms })
}

fn load_player_form_visuals_if_present(
    manifest: &AssetManifest,
    spec: &PlayerFormSpec,
) -> Result<Option<PlayerFormVisuals>> {
    if find_ingame_asset(manifest, spec.palette_path).is_none() {
        return Ok(None);
    }
    load_player_form_visuals(manifest, spec).map(Some)
}

fn load_juggernaut_form_visuals_if_present(
    manifest: &AssetManifest,
) -> Result<Option<PlayerFormVisuals>> {
    if find_ingame_asset(manifest, JUGGERNAUT_PALETTE_PATH).is_none() {
        return Ok(None);
    }

    let palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "ingame",
        find_ingame_asset(manifest, JUGGERNAUT_PALETTE_PATH).ok_or_else(|| {
            anyhow!("missing Juggernaut palette asset ingame/{JUGGERNAUT_PALETTE_PATH}")
        })?,
        "Juggernaut palette",
    )?)?;
    let transform_palette = load_ingame_palette_asset(
        manifest,
        JUGGERNAUT_TRANSFORM_PALETTE_PATH,
        "Juggernaut transform palette",
    )?;

    Ok(Some(PlayerFormVisuals {
        palette,
        idle: load_composite_player_frame(
            manifest,
            &[JUGGERNAUT_HEAD_IDLE_FRAME, JUGGERNAUT_BOTTOM_IDLE_FRAME],
        )?,
        walk: JUGGERNAUT_HEAD_WALK_FRAMES
            .iter()
            .zip(JUGGERNAUT_BOTTOM_WALK_FRAMES.iter())
            .map(|(&head, &bottom)| load_composite_player_frame(manifest, &[head, bottom]))
            .collect::<Result<Vec<_>>>()?,
        slope_walk: Vec::new(),
        slope_walk_left: Vec::new(),
        jump_rise: load_composite_player_frame(
            manifest,
            &[
                JUGGERNAUT_HEAD_IDLE_FRAME,
                pfd(
                    "artunc_kid/juggernaut_bottom_jumping_1.bin",
                    0x17,
                    0x06,
                    0x30,
                    0x14,
                ),
            ],
        )?,
        jump_fall: load_composite_player_frame(
            manifest,
            &[
                JUGGERNAUT_HEAD_IDLE_FRAME,
                pfd(
                    "artunc_kid/juggernaut_bottom_jumping_3.bin",
                    0x17,
                    0x04,
                    0x2F,
                    0x12,
                ),
            ],
        )?,
        pull_up: Vec::new(),
        slash_ground: Vec::new(),
        slash_air: Vec::new(),
        transform_palette: Some(transform_palette),
        transform_steps: JUGGERNAUT_TRANSFORM_STEPS
            .iter()
            .copied()
            .map(|step| {
                Ok(PlayerTransformStep {
                    duration_frames: step.duration_frames,
                    frame: step
                        .frame
                        .map(|descriptor| load_player_frame(manifest, descriptor))
                        .transpose()?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
    }))
}

fn load_player_form_visuals(
    manifest: &AssetManifest,
    spec: &PlayerFormSpec,
) -> Result<PlayerFormVisuals> {
    let palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "ingame",
        find_ingame_asset(manifest, spec.palette_path).ok_or_else(|| {
            anyhow!(
                "missing {} asset ingame/{}",
                spec.palette_family,
                spec.palette_path
            )
        })?,
        spec.palette_family,
    )?)?;

    let transform_palette = spec
        .transform_palette
        .map(|(relative_path, family)| load_ingame_palette_asset(manifest, relative_path, family))
        .transpose()?;

    Ok(PlayerFormVisuals {
        palette,
        idle: load_player_frame(manifest, spec.idle)?,
        walk: spec
            .walk
            .iter()
            .copied()
            .map(|descriptor| load_player_frame(manifest, descriptor))
            .collect::<Result<Vec<_>>>()?,
        slope_walk: spec
            .slope_walk
            .iter()
            .copied()
            .filter_map(|descriptor| load_player_frame(manifest, descriptor).ok())
            .collect(),
        slope_walk_left: spec
            .slope_walk_left
            .iter()
            .copied()
            .filter_map(|descriptor| load_player_frame(manifest, descriptor).ok())
            .collect(),
        jump_rise: load_player_frame(manifest, spec.jump_rise)?,
        jump_fall: load_player_frame(manifest, spec.jump_fall)?,
        pull_up: load_player_frames(manifest, spec.pull_up)?,
        slash_ground: load_player_frames(manifest, spec.slash_ground)?,
        slash_air: load_player_frames(manifest, spec.slash_air)?,
        transform_palette,
        transform_steps: spec
            .transform_steps
            .iter()
            .copied()
            .map(|step| {
                Ok(PlayerTransformStep {
                    duration_frames: step.duration_frames,
                    frame: step
                        .frame
                        .map(|descriptor| load_player_frame(manifest, descriptor))
                        .transpose()?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn load_player_frame(
    manifest: &AssetManifest,
    descriptor: PlayerFrameDescriptor,
) -> Result<SpriteFrame> {
    let asset = find_ingame_asset(manifest, descriptor.relative_path).ok_or_else(|| {
        anyhow!(
            "missing Kid sprite asset ingame/{}",
            descriptor.relative_path
        )
    })?;
    let tile_sheet = load_uncompressed_tile_sheet(
        manifest,
        "ingame",
        asset,
        &format!("Kid sprite frame {}", descriptor.relative_path),
    )?;
    Ok(SpriteFrame {
        origin_x: descriptor.origin_x,
        origin_y: descriptor.origin_y,
        width_px: descriptor.width_px,
        height_px: descriptor.height_px,
        tile_base_index: 0,
        tile_sheet,
    })
}

fn load_player_frames(
    manifest: &AssetManifest,
    descriptors: &[PlayerFrameDescriptor],
) -> Result<Vec<SpriteFrame>> {
    descriptors
        .iter()
        .copied()
        .map(|descriptor| load_player_frame(manifest, descriptor))
        .collect()
}

fn load_composite_player_frame(
    manifest: &AssetManifest,
    descriptors: &[PlayerFrameDescriptor],
) -> Result<SpriteFrame> {
    let parts = descriptors
        .iter()
        .copied()
        .map(|descriptor| load_player_frame(manifest, descriptor))
        .collect::<Result<Vec<_>>>()?;
    compose_player_frame(&parts)
}

fn compose_player_frame(parts: &[SpriteFrame]) -> Result<SpriteFrame> {
    if parts.is_empty() {
        bail!("composite player frame requires at least one part");
    }

    let mut min_left = i32::MAX;
    let mut min_top = i32::MAX;
    let mut max_right = i32::MIN;
    let mut max_bottom = i32::MIN;
    for part in parts {
        let left = -i32::from(part.origin_x);
        let top = -i32::from(part.height_px) + i32::from(part.origin_y);
        let right = left + i32::from(part.width_px);
        let bottom = top + i32::from(part.height_px);
        min_left = min_left.min(left);
        min_top = min_top.min(top);
        max_right = max_right.max(right);
        max_bottom = max_bottom.max(bottom);
    }

    let width_px =
        u16::try_from(max_right.saturating_sub(min_left)).context("composite player width")?;
    let height_px =
        u16::try_from(max_bottom.saturating_sub(min_top)).context("composite player height")?;
    let canvas_width = usize::from(align_sprite_dimension(width_px));
    let canvas_height = usize::from(align_sprite_dimension(height_px));
    let mut canvas = vec![0_u8; canvas_width * canvas_height];

    for part in parts {
        let left = -i32::from(part.origin_x) - min_left;
        let top = -i32::from(part.height_px) + i32::from(part.origin_y) - min_top;
        blit_player_frame_pixels(
            part,
            &mut canvas,
            canvas_width,
            canvas_height,
            usize::try_from(left).context("composite frame x offset")?,
            usize::try_from(top).context("composite frame y offset")?,
        );
    }

    Ok(SpriteFrame {
        origin_x: i16::try_from(-min_left).context("composite origin x")?,
        origin_y: i16::try_from(max_bottom).context("composite origin y")?,
        width_px,
        height_px,
        tile_base_index: 0,
        tile_sheet: TileSheet {
            tiles: canvas_to_tiles(&canvas, canvas_width, canvas_height),
        },
    })
}

fn blit_player_frame_pixels(
    frame: &SpriteFrame,
    canvas: &mut [u8],
    canvas_width: usize,
    canvas_height: usize,
    offset_x: usize,
    offset_y: usize,
) {
    let aligned_width = usize::from(align_sprite_dimension(frame.width_px));
    let aligned_height = usize::from(align_sprite_dimension(frame.height_px));
    let frame_pixels = frame_to_pixels(frame);
    let copy_width = aligned_width.min(canvas_width.saturating_sub(offset_x));
    let copy_height = aligned_height.min(canvas_height.saturating_sub(offset_y));
    for y in 0..copy_height {
        let dst_row = offset_y + y;
        let src_row = y * aligned_width;
        let dst_index = dst_row * canvas_width + offset_x;
        let dst_slice = &mut canvas[dst_index..dst_index + copy_width];
        let src_slice = &frame_pixels[src_row..src_row + copy_width];
        for (dst, &src) in dst_slice.iter_mut().zip(src_slice.iter()) {
            if src != 0 {
                *dst = src;
            }
        }
    }
}

fn frame_to_pixels(frame: &SpriteFrame) -> Vec<u8> {
    let aligned_width = usize::from(align_sprite_dimension(frame.width_px));
    let aligned_height = usize::from(align_sprite_dimension(frame.height_px));
    let tiles_per_row = aligned_width / 8;
    let mut pixels = vec![0_u8; aligned_width * aligned_height];
    for (tile_index, tile) in frame.tile_sheet.tiles.iter().enumerate() {
        let tile_x = (tile_index % tiles_per_row) * 8;
        let tile_y = (tile_index / tiles_per_row) * 8;
        for row in 0..8 {
            let dst_row = (tile_y + row) * aligned_width + tile_x;
            let src_row = row * 8;
            pixels[dst_row..dst_row + 8].copy_from_slice(&tile.pixels[src_row..src_row + 8]);
        }
    }
    pixels
}

fn canvas_to_tiles(canvas: &[u8], width: usize, height: usize) -> Vec<crate::Tile> {
    let mut tiles = Vec::with_capacity((width / 8) * (height / 8));
    for tile_y in (0..height).step_by(8) {
        for tile_x in (0..width).step_by(8) {
            let mut pixels = [0_u8; 64];
            for row in 0..8 {
                let src_index = (tile_y + row) * width + tile_x;
                let dst_index = row * 8;
                pixels[dst_index..dst_index + 8].copy_from_slice(&canvas[src_index..src_index + 8]);
            }
            tiles.push(crate::Tile { pixels });
        }
    }
    tiles
}

const fn align_sprite_dimension(value: u16) -> u16 {
    value.div_ceil(8) * 8
}

fn load_ingame_palette_asset(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
) -> Result<Palette> {
    Palette::from_bytes(&read_root_asset(
        manifest,
        "ingame",
        find_ingame_asset(manifest, relative_path)
            .ok_or_else(|| anyhow!("missing {family} asset ingame/{relative_path}"))?,
        family,
    )?)
}

fn load_floating_prize_helmet_palette_asset(
    manifest: &AssetManifest,
    base_relative_path: &str,
    relative_path: &str,
    family: &str,
) -> Result<Palette> {
    let base_path = if find_ingame_asset(manifest, base_relative_path).is_some() {
        base_relative_path
    } else {
        KID_PALETTE_PATH
    };
    let base = load_ingame_palette_asset(manifest, base_path, family)?;
    let helmet = load_ingame_palette_asset(manifest, relative_path, family)?;
    if helmet.colours.len() != 3 {
        return Ok(helmet);
    }

    let mut colours = base.colours;
    colours.resize(15, MdColour::from_word(0));
    // ASM sub_10CB0: `lea Palette_Buffer+$7A,a1; move.w (a5)+,(a1)+`
    // copies the three helmet colours into palette-line 3 slots $D..$F.
    // The mask art also references low player-form colours, so compose the
    // standalone palette from the form palette plus the three-word helmet patch.
    colours[12..15].copy_from_slice(&helmet.colours);
    Ok(Palette { colours })
}

fn load_compressed_sprite_frames(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
    descriptors: &[SpriteFrameDescriptor],
) -> Result<Vec<SpriteFrame>> {
    let tile_sheet = load_compressed_sprite_sheet(manifest, relative_path, family)?;
    Ok(descriptors
        .iter()
        .copied()
        .map(|descriptor| SpriteFrame {
            origin_x: descriptor.origin_x,
            origin_y: descriptor.origin_y,
            width_px: descriptor.width_px,
            height_px: descriptor.height_px,
            tile_base_index: descriptor.tile_base_index,
            tile_sheet: tile_sheet.clone(),
        })
        .collect())
}

fn load_compressed_sprite_frame(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
    descriptor: SpriteFrameDescriptor,
) -> Result<SpriteFrame> {
    Ok(SpriteFrame {
        origin_x: descriptor.origin_x,
        origin_y: descriptor.origin_y,
        width_px: descriptor.width_px,
        height_px: descriptor.height_px,
        tile_base_index: descriptor.tile_base_index,
        tile_sheet: load_compressed_sprite_sheet(manifest, relative_path, family)?,
    })
}

fn load_compressed_sprite_sheet(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
) -> Result<TileSheet> {
    let asset = find_ingame_asset(manifest, relative_path)
        .ok_or_else(|| anyhow!("missing {family} asset ingame/{relative_path}"))?;
    load_tile_sheet(manifest, "ingame", asset, family)
}

fn load_flag_visuals(
    manifest: &AssetManifest,
    foreground_palette: &Palette,
) -> Result<FlagVisuals> {
    let flag_art = find_ingame_asset(manifest, FLAG_ART_PATH)
        .ok_or_else(|| anyhow!("missing end-of-level flag art asset ingame/{FLAG_ART_PATH}"))?;
    let pole_art = find_ingame_asset(manifest, FLAG_BOTTOM_ART_PATH).ok_or_else(|| {
        anyhow!("missing flag pole bottom art asset ingame/{FLAG_BOTTOM_ART_PATH}")
    })?;
    let flag_tile_sheet =
        load_uncompressed_tile_sheet(manifest, "ingame", flag_art, "end-of-level flag art")?;
    let pole_tile_sheet =
        load_uncompressed_tile_sheet(manifest, "ingame", pole_art, "flag pole bottom art")?;

    Ok(FlagVisuals {
        palette: foreground_palette.clone(),
        pole_bottom: SpriteFrame {
            origin_x: FLAG_POLE_BOTTOM_FRAME.origin_x,
            origin_y: FLAG_POLE_BOTTOM_FRAME.origin_y,
            width_px: FLAG_POLE_BOTTOM_FRAME.width_px,
            height_px: FLAG_POLE_BOTTOM_FRAME.height_px,
            tile_base_index: FLAG_POLE_BOTTOM_FRAME.tile_base_index,
            tile_sheet: pole_tile_sheet,
        },
        waving: FLAG_WAVING_FRAMES
            .iter()
            .copied()
            .map(|descriptor| SpriteFrame {
                origin_x: descriptor.origin_x,
                origin_y: descriptor.origin_y,
                width_px: descriptor.width_px,
                height_px: descriptor.height_px,
                tile_base_index: descriptor.tile_base_index,
                tile_sheet: flag_tile_sheet.clone(),
            })
            .collect(),
        frame_duration: 6,
    })
}

fn parse_background_chunk_id(relative_path: &str, prefix: &str) -> Option<u16> {
    let suffix = relative_path.strip_prefix(prefix)?.strip_suffix(".bin")?;
    if suffix.contains("_eni_") {
        return None;
    }
    u16::from_str_radix(suffix, 16).ok()
}

fn parse_layered_background_chunk_id(relative_path: &str, prefix: &str) -> Option<u16> {
    let suffix = relative_path.strip_prefix(prefix)?.strip_suffix(".bin")?;
    u16::from_str_radix(suffix, 16).ok()
}

const BASE_RUBBER_BLOCK_VRAM_START: u16 = 0x026E;
const BASE_RUBBER_BLOCK_RELATIVE_START: usize =
    (BASE_RUBBER_BLOCK_VRAM_START - BLOCK_TILE_VRAM_BASE) as usize;
const BASE_RUBBER_BLOCK_RELATIVE_END: usize =
    BASE_RUBBER_BLOCK_RELATIVE_START + RUBBER_BLOCK_TILE_COUNT;

/// Number of generated rubber-block tiles (`$2F3..$2FA`, eight tiles).
const RUBBER_BLOCK_TILE_COUNT: usize = 8;

/// ASM `unk_14576` (kid.asm:27425) selects the terrain-mapping quad whose
/// four foreground tiles are duplicated and merged into indoor rubber blocks.
const INDOOR_RUBBER_BLOCK_THEME_MAPPING_INDICES: [usize; THEME_NAMES.len()] = [
    0x10, 0x0A, 0x7D, 0x60, 0xE5, 0x67, 0x73, 0x5F, 0x01, 0xC5, 0x9A,
];

/// Append the generated indoor rubber-block tiles so VRAM indices
/// `$2F3..$2FA` resolve. ASM `sub_144DA` (kid.asm:27355-27420) reads four
/// theme-specific foreground tiles from `unk_14576[theme]*8`, duplicates that
/// four-tile buffer, then merges it with the eight base block tiles at
/// `$26E..$275` through `loc_1453E`.
fn append_indoor_rubber_block_tiles_sub_144da(
    block_tiles: &mut TileSheet,
    foreground_tiles: &TileSheet,
    terrain_quads: &[crate::tiles::TileQuad],
    foreground_theme_id: u8,
) {
    let Some(base_slice) = block_tiles
        .tiles
        .get(BASE_RUBBER_BLOCK_RELATIVE_START..BASE_RUBBER_BLOCK_RELATIVE_END)
    else {
        return;
    };
    if base_slice.len() != RUBBER_BLOCK_TILE_COUNT {
        return;
    }
    let base_tiles = base_slice.to_vec();
    let Some(mapping_quad_index) =
        INDOOR_RUBBER_BLOCK_THEME_MAPPING_INDICES.get(usize::from(foreground_theme_id))
    else {
        block_tiles.tiles.extend(base_tiles);
        return;
    };
    let Some(source_quad) = terrain_quads.get(*mapping_quad_index) else {
        block_tiles.tiles.extend(base_tiles);
        return;
    };

    let mut generated_tiles = Vec::with_capacity(RUBBER_BLOCK_TILE_COUNT);
    for (rubber_tile_index, base_tile) in base_tiles.iter().enumerate() {
        let source_ref = source_quad.tiles[rubber_tile_index % source_quad.tiles.len()];
        let Some(relative_foreground_index) = source_ref
            .tile_index
            .checked_sub(FOREGROUND_TILE_VRAM_BASE)
            .map(usize::from)
        else {
            block_tiles.tiles.extend(base_tiles.clone());
            return;
        };
        let Some(overlay_tile) = foreground_tiles.tiles.get(relative_foreground_index) else {
            block_tiles.tiles.extend(base_tiles.clone());
            return;
        };
        generated_tiles.push(compose_indoor_rubber_block_tile_sub_144da(
            base_tile,
            overlay_tile,
        ));
    }

    block_tiles.tiles.extend(generated_tiles);
}

fn compose_indoor_rubber_block_tile_sub_144da(
    base_tile: &crate::tiles::Tile,
    overlay_tile: &crate::tiles::Tile,
) -> crate::tiles::Tile {
    // ASM loc_1453E (kid.asm:27397-27405): keep each non-zero nibble from the
    // existing base rubber-block VRAM tile; only zero holes receive the
    // duplicated foreground tile nibble from the decompression buffer.
    let mut pixels = [0_u8; 64];
    for (pixel_index, (&base_pixel, &overlay_pixel)) in base_tile
        .pixels
        .iter()
        .zip(overlay_tile.pixels.iter())
        .enumerate()
    {
        pixels[pixel_index] = if base_pixel != 0 {
            base_pixel
        } else {
            overlay_pixel
        };
    }
    crate::tiles::Tile { pixels }
}

fn load_tile_sheet(
    manifest: &AssetManifest,
    root_name: &str,
    asset: &AssetFile,
    family: &str,
) -> Result<TileSheet> {
    let compressed = read_root_asset(manifest, root_name, asset, family)?;
    let decoded = decode_saxman(&compressed).with_context(|| {
        format!(
            "failed to decode {family} from {root_name}/{}",
            asset.relative_path
        )
    })?;
    TileSheet::from_4bpp_bytes(&decoded).with_context(|| {
        format!(
            "failed to parse {family} from {root_name}/{}",
            asset.relative_path
        )
    })
}

fn load_uncompressed_tile_sheet(
    manifest: &AssetManifest,
    root_name: &str,
    asset: &AssetFile,
    family: &str,
) -> Result<TileSheet> {
    let bytes = read_root_asset(manifest, root_name, asset, family)?;
    TileSheet::from_4bpp_bytes(&bytes).with_context(|| {
        format!(
            "failed to parse {family} from {root_name}/{}",
            asset.relative_path
        )
    })
}

fn decode_level_saxman_asset(asset: &kid_data::LoadedAsset, family: &str) -> Result<Vec<u8>> {
    decode_saxman(&asset.bytes)
        .with_context(|| format!("failed to decode {family} from {}", asset.relative_path))
}

fn read_decoded_file(
    manifest: &AssetManifest,
    asset: &DecodedAssetFile,
    family: &str,
) -> Result<Vec<u8>> {
    let path = manifest.output_root.join(&asset.relative_path);
    read_file_with_contract(
        &path,
        &asset.relative_path,
        asset.size_bytes,
        &asset.sha1,
        family,
    )
}

fn read_root_asset(
    manifest: &AssetManifest,
    root_name: &str,
    asset: &AssetFile,
    family: &str,
) -> Result<Vec<u8>> {
    let root = match root_name {
        "theme" => manifest.output_root.join("theme"),
        "ingame" => manifest.output_root.join("ingame"),
        other => bail!("unsupported generated bundle root {other}"),
    };
    let path = root.join(&asset.relative_path);
    read_file_with_contract(
        &path,
        &format!("{root_name}/{}", asset.relative_path),
        asset.size_bytes,
        &asset.sha1,
        family,
    )
}

fn read_file_with_contract(
    path: &Path,
    display_path: &str,
    expected_size: u64,
    expected_sha1: &str,
    family: &str,
) -> Result<Vec<u8>> {
    if !path.is_file() {
        bail!("missing {family} asset at {display_path}");
    }
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read {family} asset from {}", path.display()))?;
    if bytes.len() as u64 != expected_size {
        bail!(
            "{family} asset size mismatch for {display_path}: expected {expected_size} bytes, got {}",
            bytes.len()
        );
    }
    let actual_sha1 = sha1_hex(&bytes);
    if actual_sha1 != expected_sha1 {
        bail!(
            "{family} asset hash mismatch for {display_path}: expected {expected_sha1}, got {actual_sha1}"
        );
    }
    Ok(bytes)
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{
        FOREST_ENEMY_ART_PERMUTATION, FOREST_TAR_MONSTER_ENEMY_ART_PERMUTATION, FOREST_THEME_ID,
        append_indoor_rubber_block_tiles_sub_144da, apply_enemy_art_permutation,
        apply_layered_background_palette_fallback, bg_permutation_file, canvas_to_tiles,
        combine_eol_hologram_tile_sheet, compose_player_frame, fg_permutation_file,
        forest_enemy_art_permutation, frame_to_pixels, load_palette_permutation,
    };
    use crate::{LayeredBackgroundPlane, SpriteFrame, Tile, TileQuad, TileRef, TileSheet};
    use kid_data::{AssetFile, AssetManifest, EnemyLayout};
    use sha1::{Digest, Sha1};
    use std::fs;
    use tempfile::TempDir;

    fn tile_ref(tile_index: u16) -> TileRef {
        TileRef {
            tile_index,
            palette_line: 0,
            priority: false,
            flip_x: false,
            flip_y: false,
        }
    }

    fn solid_tile(pixel: u8) -> Tile {
        Tile {
            pixels: [pixel; 64],
        }
    }

    #[test]
    fn eol_hologram_sheet_keeps_holobg_tiles_beyond_holoblocks_upload() {
        let background_tiles = TileSheet {
            tiles: (0..0x0120)
                .map(|index| solid_tile((index & 0xFF) as u8))
                .collect(),
        };
        let block_tiles = TileSheet {
            tiles: vec![solid_tile(0xA0), solid_tile(0xA1)],
        };

        let sheet = combine_eol_hologram_tile_sheet(background_tiles, block_tiles);

        assert_eq!(
            sheet.tiles[0x0409].pixels[0], 0xA0,
            "ASM loc_C10A uploads ArtComp_CAB2_HoloBlocks at VRAM tile $409, overlaying HoloBG there"
        );
        assert_eq!(
            sheet.tiles[0x0417].pixels[0], 0x1C,
            "ASM sub_C254/unk_C2EA can reveal tile word $4417, so native must keep the longer HoloBG art range instead of truncating at the two HoloBlocks tiles"
        );
        assert_eq!(sheet.tiles[0x041A].pixels[0], 0x1F);
    }

    fn enemy_layout_with_slots(enemy_type_slots: Vec<u16>) -> EnemyLayout {
        EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots,
            unknown_word: 0,
            header_byte: 0,
            object_count: 0,
            objects: Vec::new(),
            segments: Vec::new(),
        }
    }

    fn asset_file(relative_path: &str, bytes: &[u8]) -> AssetFile {
        AssetFile {
            relative_path: relative_path.to_owned(),
            size_bytes: u64::try_from(bytes.len()).expect("asset size fits in u64"),
            sha1: format!("{:x}", Sha1::digest(bytes)),
        }
    }

    #[test]
    fn forest_enemy_art_permutation_matches_asm_first_slot_rules() {
        let dragon_first = enemy_layout_with_slots(vec![0x000C, 0x401C, 0xFFFF]);
        assert_eq!(
            forest_enemy_art_permutation(FOREST_THEME_ID, Some(&dragon_first), 0x0C),
            Some(&FOREST_ENEMY_ART_PERMUTATION),
            "ASM Load_EnemyArtToVRAM loc_36E24 remaps the first forest enemy slot"
        );
        assert_eq!(
            forest_enemy_art_permutation(FOREST_THEME_ID, Some(&dragon_first), 0x1C),
            None,
            "ASM only routes d7=0, the first loaded enemy art slot, through the forest RAM remap"
        );

        let tar_first = enemy_layout_with_slots(vec![0x0005, 0xFFFF, 0xFFFF]);
        assert_eq!(
            forest_enemy_art_permutation(FOREST_THEME_ID, Some(&tar_first), 0x05),
            Some(&FOREST_TAR_MONSTER_ENEMY_ART_PERMUTATION),
            "ASM loc_36E24 selects the Tar Monster-specific remap table"
        );
        assert_eq!(
            forest_enemy_art_permutation(8, Some(&dragon_first), 0x0C),
            None
        );
    }

    #[test]
    fn apply_enemy_art_permutation_rewrites_loaded_sprite_pixels() {
        let mut frames = vec![SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 8,
            height_px: 8,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [8; 64] }],
            },
        }];

        apply_enemy_art_permutation(&mut frames, &FOREST_ENEMY_ART_PERMUTATION);

        assert_eq!(
            frames[0].tile_sheet.tiles[0].pixels, [0x0A; 64],
            "ASM unk_36E54 maps pixel index 8 to $A before forest enemy art is rendered"
        );
    }

    fn palette_manifest(relative_path: &str, bytes: &[u8]) -> (TempDir, AssetManifest) {
        let temp_dir = TempDir::new().expect("temp generated bundle root");
        let theme_root = temp_dir.path().join("theme");
        let asset_path = theme_root.join(relative_path);
        fs::create_dir_all(asset_path.parent().expect("asset has parent"))
            .expect("create theme asset parent");
        fs::write(&asset_path, bytes).expect("write theme asset");

        (
            temp_dir,
            AssetManifest {
                generated_bundle_version: 0,
                rom_sha1: "test-rom".to_owned(),
                upstream_commit: "test-upstream".to_owned(),
                source_root: ".".into(),
                output_root: theme_root
                    .parent()
                    .expect("theme root has parent")
                    .to_owned(),
                map_order: Vec::new(),
                levels: Vec::new(),
                constants: Vec::new(),
                ram_variables: Vec::new(),
                enemy_sources: Vec::new(),
                theme_assets: vec![asset_file(relative_path, bytes)],
                scene_assets: Vec::new(),
                animation_assets: Vec::new(),
                sound_assets: Vec::new(),
                ingame_assets: Vec::new(),
                decoded_levels: Vec::new(),
            },
        )
    }

    #[test]
    fn palette_permutation_paths_match_rom_theme_switches() {
        assert_eq!(
            fg_permutation_file(10),
            "palette_permutations/city_fg.bin",
            "ASM loc_12A22 selects Palette_Permutation_FGCity for foreground theme $0A"
        );
        assert_eq!(
            fg_permutation_file(9),
            "palette_permutations/forest_fg.bin",
            "ASM loc_12A27 selects Palette_Permutation_FGForest for foreground theme $09"
        );
        assert_eq!(
            fg_permutation_file(7),
            "palette_permutations/mountain_fg.bin",
            "ASM loc_12A2C selects Palette_Permutation_FGMountain for foreground theme $07"
        );
        assert_eq!(
            fg_permutation_file(0),
            "palette_permutations/identity.bin",
            "ASM loc_12A19 defaults foreground themes to Palette_Permutation_Identity"
        );

        assert_eq!(
            bg_permutation_file(9),
            "palette_permutations/forest_bg.bin",
            "ASM loc_12A41 selects Palette_Permutation_BGForest for background theme $09"
        );
        assert_eq!(
            bg_permutation_file(7),
            "palette_permutations/mountain_bg.bin",
            "ASM loc_12A46 selects Palette_Permutation_BGMountain for background theme $07"
        );
        assert_eq!(
            bg_permutation_file(3),
            "palette_permutations/hill_bg.bin",
            "ASM loc_12A4B selects Palette_Permutation_BGHill for background theme $03"
        );
        assert_eq!(
            bg_permutation_file(0),
            "palette_permutations/identity.bin",
            "ASM loc_12A3C defaults background themes to Palette_Permutation_Identity"
        );
    }

    #[test]
    fn palette_permutation_loader_requires_complete_sixteen_byte_table() {
        let exact_bytes = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        let (_temp_dir, manifest) =
            palette_manifest("palette_permutations/exact.bin", &exact_bytes);

        assert_eq!(
            load_palette_permutation(&manifest, "palette_permutations/exact.bin"),
            Some(exact_bytes),
            "ASM Palette_Permutation_* tables are valid at exactly sixteen entries"
        );

        let bytes = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 99];
        let (_temp_dir, manifest) = palette_manifest("palette_permutations/test.bin", &bytes);

        assert_eq!(
            load_palette_permutation(&manifest, "palette_permutations/test.bin"),
            Some([15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]),
            "ASM Palette_Permutation_* tables are 16 byte nibble lookup tables"
        );

        let short_bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        let (_temp_dir, manifest) =
            palette_manifest("palette_permutations/short.bin", &short_bytes);

        assert_eq!(
            load_palette_permutation(&manifest, "palette_permutations/short.bin"),
            None,
            "ASM Palette_Permutation_* tables must contain all sixteen entries"
        );
    }

    #[test]
    fn forest_plane_fallback_promotes_zero_palette_line_to_plane_line() {
        let mut plane = LayeredBackgroundPlane {
            rows: vec![vec![TileRef {
                tile_index: 1,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            }]],
        };

        apply_layered_background_palette_fallback("forest", 0, &mut plane);

        assert_eq!(plane.rows[0][0].palette_line, 1);
    }

    #[test]
    fn layered_background_palette_fallbacks_match_rom_plane_lines() {
        for (theme_name, plane_id, expected_palette_line) in [
            ("desert", 0, 2),
            ("desert", 1, 2),
            ("desert", 2, 2),
            ("forest", 0, 1),
            ("forest", 1, 1),
            ("forest", 2, 2),
            ("hill", 0, 2),
            ("hill", 1, 2),
        ] {
            let mut plane = LayeredBackgroundPlane {
                rows: vec![vec![tile_ref(1)]],
            };

            apply_layered_background_palette_fallback(theme_name, plane_id, &mut plane);

            assert_eq!(
                plane.rows[0][0].palette_line, expected_palette_line,
                "ASM PlaneEd CRAM offsets for {theme_name} plane {plane_id} set the fallback palette line"
            );
        }

        let mut mountain_plane = LayeredBackgroundPlane {
            rows: vec![vec![tile_ref(1)]],
        };
        apply_layered_background_palette_fallback("mountain", 0, &mut mountain_plane);

        assert_eq!(
            mountain_plane.rows[0][0].palette_line, 0,
            "ASM layered mountain planes already carry explicit palette lines"
        );
    }

    #[test]
    fn fallback_preserves_explicit_palette_lines() {
        let mut plane = LayeredBackgroundPlane {
            rows: vec![vec![TileRef {
                tile_index: 1,
                palette_line: 2,
                priority: false,
                flip_x: false,
                flip_y: false,
            }]],
        };

        apply_layered_background_palette_fallback("forest", 0, &mut plane);

        assert_eq!(plane.rows[0][0].palette_line, 2);
    }

    #[test]
    fn composite_player_frame_clips_padded_tiles_to_declared_bounds() {
        let head = SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 9,
            height_px: 9,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![Tile { pixels: [1; 64] }; 4],
            },
        };
        let mut body_tile = Tile { pixels: [2; 64] };
        body_tile.pixels[0] = 0;
        body_tile.pixels[8] = 3;
        let body = SpriteFrame {
            origin_x: 0,
            origin_y: 4,
            width_px: 8,
            height_px: 8,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![body_tile],
            },
        };

        let composite = compose_player_frame(&[head, body]).expect("composite frame");

        assert_eq!(composite.width_px, 9);
        assert_eq!(composite.height_px, 13);
        assert!(!composite.tile_sheet.tiles.is_empty());
        let pixels = frame_to_pixels(&composite);
        assert_eq!(
            pixels[5 * 16],
            1,
            "ASM sprite_frame_unc treats palette index 0 as transparent during composite blits"
        );
        assert_eq!(
            pixels[5 * 16 + 1],
            2,
            "ASM sprite_frame_unc non-zero pixels from later composite parts overwrite earlier art"
        );
        assert_eq!(
            pixels[6 * 16],
            3,
            "ASM sprite_frame_unc blits each source tile row from its own 8-pixel row"
        );
    }

    #[test]
    fn composite_player_frame_preserves_rom_origin_geometry() {
        let left_part = SpriteFrame {
            origin_x: 3,
            origin_y: 0,
            width_px: 8,
            height_px: 8,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![solid_tile(3)],
            },
        };
        let right_part = SpriteFrame {
            origin_x: -5,
            origin_y: 2,
            width_px: 8,
            height_px: 8,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![solid_tile(4)],
            },
        };

        let composite = compose_player_frame(&[left_part, right_part]).expect("composite frame");

        assert_eq!(
            composite.origin_x, 3,
            "ASM sprite_frame_unc origin_x is the negated left extent of all composite parts"
        );
        assert_eq!(
            composite.origin_y, 2,
            "ASM sprite_frame_unc origin_y is the lower extent of all composite parts"
        );
        assert_eq!(composite.width_px, 16);
        assert_eq!(composite.height_px, 10);
    }

    #[test]
    fn frame_to_pixels_and_canvas_to_tiles_preserve_tile_grid_positions() {
        let top_left = Tile {
            pixels: core::array::from_fn(|pixel_index| {
                u8::try_from(pixel_index).expect("pixel index fits in u8")
            }),
        };
        let top_right = solid_tile(64);
        let bottom_left = solid_tile(128);
        let bottom_right = solid_tile(192);
        let frame = SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 16,
            height_px: 16,
            tile_base_index: 0,
            tile_sheet: TileSheet {
                tiles: vec![
                    top_left.clone(),
                    top_right.clone(),
                    bottom_left.clone(),
                    bottom_right.clone(),
                ],
            },
        };

        let pixels = frame_to_pixels(&frame);

        assert_eq!(
            pixels[0], 0,
            "ASM sprite_frame_unc stores sprite art as row-major 8x8 tiles"
        );
        assert_eq!(
            pixels[7], 7,
            "ASM sprite_frame_unc stores sprite art as row-major 8x8 tiles"
        );
        assert_eq!(
            pixels[16], 8,
            "ASM sprite_frame_unc advances to the next tile row after eight pixels"
        );
        assert_eq!(
            pixels[8], 64,
            "ASM sprite_frame_unc places the second tile at x + 8"
        );
        assert_eq!(
            pixels[16 * 8],
            128,
            "ASM sprite_frame_unc places the third tile at y + 8"
        );
        assert_eq!(
            pixels[16 * 8 + 8],
            192,
            "ASM sprite_frame_unc places the fourth tile at x + 8, y + 8"
        );

        assert_eq!(
            canvas_to_tiles(&pixels, 16, 16),
            vec![top_left, top_right, bottom_left, bottom_right],
            "ASM sprite_frame_unc tile order round-trips through the native canvas"
        );
    }

    #[test]
    fn sub_144da_appends_theme_composed_indoor_rubber_block_tiles() {
        assert_eq!(
            super::BASE_RUBBER_BLOCK_RELATIVE_START,
            0x4E,
            "ASM sub_144DA starts generated rubber-block composition at VRAM tile $026E - block base $0220"
        );
        assert_eq!(
            super::BASE_RUBBER_BLOCK_RELATIVE_END,
            0x56,
            "ASM sub_144DA consumes the eight source rubber-block tiles $026E..$0275"
        );

        let mut block_tiles = TileSheet {
            tiles: vec![solid_tile(0); super::BASE_RUBBER_BLOCK_RELATIVE_END],
        };
        for tile in &mut block_tiles.tiles
            [super::BASE_RUBBER_BLOCK_RELATIVE_START..super::BASE_RUBBER_BLOCK_RELATIVE_END]
        {
            for (pixel_index, pixel) in tile.pixels.iter_mut().enumerate() {
                *pixel = if pixel_index % 2 == 0 { 2 } else { 0 };
            }
        }
        let foreground_tiles = TileSheet {
            tiles: (0..8).map(solid_tile).collect(),
        };
        let terrain_quads = vec![
            TileQuad {
                tiles: [tile_ref(0); 4],
            },
            TileQuad {
                tiles: [
                    tile_ref(super::FOREGROUND_TILE_VRAM_BASE + 4),
                    tile_ref(super::FOREGROUND_TILE_VRAM_BASE + 5),
                    tile_ref(super::FOREGROUND_TILE_VRAM_BASE + 6),
                    tile_ref(super::FOREGROUND_TILE_VRAM_BASE + 7),
                ],
            },
        ];

        append_indoor_rubber_block_tiles_sub_144da(
            &mut block_tiles,
            &foreground_tiles,
            &terrain_quads,
            0x08,
        );

        assert_eq!(
            block_tiles.tiles.len(),
            super::BASE_RUBBER_BLOCK_RELATIVE_END + super::RUBBER_BLOCK_TILE_COUNT,
            "ASM sub_144DA should append the generated $2F3..$2FA tiles after the static block art"
        );
        let generated = &block_tiles.tiles[super::BASE_RUBBER_BLOCK_RELATIVE_END..];
        assert_eq!(generated[0].pixels[0], 2);
        assert_eq!(generated[0].pixels[1], 4);
        assert_eq!(generated[1].pixels[1], 5);
        assert_eq!(
            generated[4].pixels[1], 4,
            "ASM loc_14512 duplicates the four source foreground tiles before the eight-tile merge"
        );
    }

    #[test]
    fn red_stealth_visual_spec_includes_rom_slope_cycle_off_a23c() {
        let red_stealth = super::PLAYER_FORM_SPECS
            .iter()
            .find(|spec| spec.form_id == super::PLAYER_FORM_RED_STEALTH)
            .expect("red stealth player form spec");

        assert_eq!(
            red_stealth.slope_walk,
            &super::RED_STEALTH_SLOPE_WALK_FRAMES,
            "ASM off_A23C uses Red Stealth slope frames `unk_B6ECA, unk_B7150, unk_B7150` twice"
        );
        assert_eq!(
            red_stealth.slope_walk_left,
            &super::RED_STEALTH_SLOPE_WALK_FRAMES,
            "native mirrors the one ROM Red Stealth slope art set by H-flipping the sprite"
        );
        assert_eq!(
            red_stealth.slope_walk[0].relative_path,
            "artunc_kid/red_stealth_slope_1.bin"
        );
        assert_eq!(
            red_stealth.slope_walk[1].relative_path,
            "artunc_kid/red_stealth_slope_2.bin"
        );
    }

    #[test]
    fn kid_right_slope_visual_spec_preserves_signed_rom_origins() {
        let kid = super::PLAYER_FORM_SPECS
            .iter()
            .find(|spec| spec.form_id == super::PLAYER_FORM_THE_KID)
            .expect("kid player form spec");

        assert_eq!(kid.slope_walk.len(), 6);
        assert_eq!(
            kid.slope_walk[1].origin_y, -0x02,
            "ASM unk_A2552 / kid_slope_8 stores signed origin_y=-$02 for the SlopeDown/right-facing bank"
        );
        assert_eq!(
            kid.slope_walk[2].origin_y, -0x01,
            "ASM unk_A26D8 / kid_slope_9 stores signed origin_y=-$01 for the SlopeDown/right-facing bank"
        );
        assert_eq!(
            kid.slope_walk[4].origin_y, -0x02,
            "ASM unk_A29E4 / kid_slope_11 keeps the right-facing slope bank two pixels higher"
        );
        assert_eq!(
            kid.slope_walk[5].origin_y, -0x02,
            "ASM unk_A2B6A / kid_slope_12 keeps the right-facing slope bank two pixels higher"
        );
    }

    #[test]
    fn enemy_visual_specs_include_rom_waited_death_frames() {
        let death_tiles = |object_type, range: std::ops::Range<usize>| {
            super::ENEMY_FAMILIES
                .iter()
                .find(|spec| spec.object_type == object_type)
                .expect("enemy family spec")
                .frames[range]
                .iter()
                .map(|frame| frame.tile_base_index)
                .collect::<Vec<_>>()
        };

        assert_eq!(
            death_tiles(0x03, 6..8),
            vec![0x00BB, 0x00CB],
            "ASM Robot.asm loc_3A9D2 / stru_3A74A death frames C8318/C8320 must be selectable"
        );
        assert_eq!(
            death_tiles(0x09, 6..9),
            vec![0x0020, 0x0000, 0x0010],
            "ASM Crab.asm loc_3B094 / stru_3AF82 death frames C7C16/C7C06/C7C0E must be selectable"
        );
        assert_eq!(
            death_tiles(0x0A, 3..6),
            vec![0x0000, 0x0010, 0x006C],
            "ASM Tank.asm loc_3B480 / stru_3B29A death frames C7EB8/C7EC0/C7F18 must be selectable"
        );
        assert_eq!(
            death_tiles(0x0B, 3..6),
            vec![0x0000, 0x0010, 0x003C],
            "ASM Tank_Shooting.asm loc_3B6E6 / stru_3B518 death frames C7EB8/C7EC0/C7EF8 must be selectable"
        );
        assert_eq!(
            death_tiles(0x10, 6..9),
            vec![0x0000, 0x0088, 0x0098],
            "ASM Goat.asm loc_3CB64 / stru_3C9EA death frames C7C56/C7C8E/C7C96 must be selectable"
        );
        assert_eq!(
            death_tiles(0x11, 6..9),
            vec![0x0000, 0x0010, 0x0020],
            "ASM Ninja.asm loc_3DB4E / stru_3D50A death frames C7D56/C7D5E/C7D66 must be selectable"
        );
    }

    #[test]
    fn player_visual_specs_include_rom_negative_y_offsets() {
        assert_eq!(
            super::IRON_KNIGHT_JUMP_FALL_FRAME.origin_y,
            -0x02,
            "ASM sprite frame `unk_C7850` stores Iron Knight jump-fall origin_y=-$02"
        );
        assert!(
            super::JUGGERNAUT_HEAD_WALK_FRAMES
                .iter()
                .all(|frame| frame.origin_y == -0x0A),
            "ASM Juggernaut split-head walk descriptors `unk_BEDF0`/`unk_BF976`/`unk_BFBDD`/`unk_BFE64`/`unk_C00EB` store origin_y=-$0A"
        );

        let maniaxe = super::PLAYER_FORM_SPECS
            .iter()
            .find(|spec| spec.form_id == super::PLAYER_FORM_MANIAXE)
            .expect("Maniaxe player form spec");

        assert_eq!(
            maniaxe.jump_fall.origin_y, -0x02,
            "ASM Maniaxe jump-fall sprite descriptor stores origin_y=-$02"
        );
    }
}
