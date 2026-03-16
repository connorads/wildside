mod background;
mod bundle;
mod hud;
mod level;
mod palette;
mod player;
mod tiles;

pub use background::{
    BackgroundChunk, BackgroundPlacement, BackgroundTile, BackgroundVisuals,
    ChunkedBackgroundVisuals, LayeredBackgroundLayout, LayeredBackgroundPlane,
    LayeredBackgroundRow, LayeredBackgroundVisuals, compose_layered_background,
    parse_background_chunk, parse_chunked_background_layout, parse_layered_background_layout,
    parse_layered_background_plane,
};
pub use bundle::{NativeLevelVisuals, load_level_visuals};
pub use hud::{HudIcon, HudVisuals};
pub use level::{
    BlockVisualKind, BlockVisualMapping, ForegroundLayout, LevelVisuals, ResolvedBlockVisual,
};
pub use palette::{MdColour, Palette, PaletteSet};
pub use player::{AnimatedSpriteVisual, EnemyVisuals, PlatformVisuals, PlayerVisuals, SpriteFrame};
pub use tiles::{Tile, TileQuad, TileRef, TileSheet};

#[cfg(test)]
mod tests {
    use super::{BlockVisualKind, load_level_visuals};
    use kid_data::{
        AssetFile, AssetManifest, DecodedAssetFile, DecodedLevelAssets, GENERATED_BUNDLE_VERSION,
        LEVEL1_DECODED_ASSET_VERSION, LevelHeader, LevelRecord, LevelResources,
    };
    use sha1::{Digest, Sha1};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const LEVEL_ID: u8 = 0x00;
    const CAVE_THEME_ID: u8 = 0x08;

    #[test]
    fn load_level1_tile_sheet_and_palettes() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.foreground_tiles.tiles.len(), 2);
        assert_eq!(visuals.background_tiles.tiles.len(), 1);
        assert_eq!(visuals.block_tiles.tiles.len(), 2);
        assert_eq!(visuals.palettes.foreground.colours.len(), 15);
        assert_eq!(visuals.palettes.background.colours.len(), 8);
        assert_eq!(visuals.foreground_tiles.tile_size(), (8, 8));
        assert_eq!(visuals.background_tiles.tile_size(), (8, 8));
    }

    #[test]
    fn load_block_visual_mapping_from_skin_and_block_type() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");
        let terrain = visuals
            .level
            .block_mapping
            .resolve(0, None)
            .expect("terrain mapping");
        let smash = visuals
            .level
            .block_mapping
            .resolve(0, Some(0))
            .expect("smash block mapping");
        let prize = visuals
            .level
            .block_mapping
            .resolve(0, Some(1))
            .expect("prize block mapping");
        let ice = visuals
            .level
            .block_mapping
            .resolve(0, Some(2))
            .expect("ice block mapping");
        let metal = visuals
            .level
            .block_mapping
            .resolve(0, Some(5))
            .expect("metal block mapping");
        let rubber = visuals
            .level
            .block_mapping
            .resolve(0, Some(6))
            .expect("rubber block mapping");
        let generic = visuals
            .level
            .block_mapping
            .resolve(0, Some(7))
            .expect("generic block mapping");

        assert_eq!(terrain.kind, BlockVisualKind::Terrain);
        assert_eq!(terrain.quad.tiles[0].tile_index, 10);
        assert_eq!(terrain.quad.tiles[1].tile_index, 11);

        assert_eq!(smash.kind, BlockVisualKind::Smash);
        assert_eq!(smash.quad.tiles[0].tile_index, 0x0200);
        assert_eq!(prize.kind, BlockVisualKind::Prize);
        assert_eq!(prize.quad.tiles[0].tile_index, 0x0300);
        assert_eq!(ice.kind, BlockVisualKind::Ice);
        assert_eq!(ice.quad.tiles[0].tile_index, 0x0400);
        assert_eq!(metal.kind, BlockVisualKind::Metal);
        assert_eq!(metal.quad.tiles[0].tile_index, 0x0500);
        assert_eq!(rubber.kind, BlockVisualKind::Rubber);
        assert_eq!(rubber.quad.tiles[0].tile_index, 0x0600);

        assert_eq!(generic.kind, BlockVisualKind::GenericBlock);
        assert_eq!(generic.quad.tiles[0].tile_index, 0x00BC);

        assert_ne!(smash, metal);
    }

    #[test]
    fn load_hud_visuals_returns_digit_and_icon_metadata() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.hud.digits.tiles.len(), 12);
        assert_eq!(visuals.hud.digit_base_tile_index, 0x06BA);
        assert_eq!(visuals.hud.diamond_tiles.tiles.len(), 16);
        assert_eq!(visuals.hud.diamond_base_tile_index, 0x06EA);
        assert!(visuals.hud.icons.iter().any(|icon| {
            icon.name == "ankh"
                && icon.position == (0x01A1, 0x008D)
                && icon.tile.tile_index == 0x0231
        }));
        assert!(visuals.hud.icons.iter().any(|icon| {
            icon.name == "diamond"
                && icon.position == (0x01A0, 0x00A2)
                && icon.tile.tile_index == 0x06F2
        }));
    }

    #[test]
    fn load_player_visuals_returns_required_kid_frames() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.player.palette.colours.len(), 15);
        assert_eq!(visuals.player.idle.origin_x, 0x0B);
        assert_eq!(visuals.player.idle.origin_y, 0x01);
        assert_eq!(visuals.player.idle.width_px, 0x14);
        assert_eq!(visuals.player.idle.height_px, 0x20);
        assert_eq!(visuals.player.idle.tile_sheet.tiles.len(), 12);
        assert_eq!(visuals.player.walk.len(), 6);
        assert_eq!(visuals.player.walk[0].origin_x, 0x0A);
        assert_eq!(visuals.player.walk[0].width_px, 0x13);
        assert_eq!(visuals.player.jump_rise.origin_x, 0x0A);
        assert_eq!(visuals.player.jump_rise.width_px, 0x16);
        assert_eq!(visuals.player.jump_fall.origin_x, 0x09);
        assert_eq!(visuals.player.jump_fall.width_px, 0x14);
        assert_eq!(visuals.player.idle.tile_base_index, 0);
    }

    #[test]
    fn load_enemy_and_platform_visuals_returns_visible_level1_families() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.enemies.diamond.frames.len(), 3);
        assert_eq!(visuals.enemies.diamond.frames[0].tile_base_index, 0x002F);
        assert_eq!(visuals.enemies.dragon.frames.len(), 5);
        assert_eq!(visuals.enemies.dragon.frames[0].tile_base_index, 0x011D);
        assert_eq!(visuals.enemies.mini_hopping_skull.frames.len(), 2);
        assert_eq!(
            visuals.enemies.mini_hopping_skull.frames[0].tile_base_index,
            0x002C
        );
        assert_eq!(visuals.platforms.horizontal.width_px, 0x48);
        assert_eq!(visuals.platforms.horizontal.height_px, 0x10);
    }

    #[test]
    fn missing_required_asset_reports_contextual_error() {
        let fixture = TestBundle::new().expect("test bundle");
        let missing_path = fixture
            .root
            .path()
            .join("theme")
            .join("palette_fg")
            .join("cave.bin");
        fs::remove_file(&missing_path).expect("remove palette");

        let error = load_level_visuals(&fixture.manifest, LEVEL_ID).expect_err("missing asset should fail");
        let message = format!("{error:#}");
        assert!(message.contains("theme/palette_fg/cave.bin"));
        assert!(message.contains("missing"));
    }

    #[test]
    fn kid_render_loads_complete_level1_bundle_from_generated_assets() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.level.level_id, LEVEL_ID);
        assert_eq!(visuals.level.foreground_theme_id, CAVE_THEME_ID);
        assert_eq!(visuals.level.background_theme_id, CAVE_THEME_ID);
        assert!(!visuals.level.foreground_layout_bytes.is_empty());
        assert!(!visuals.level.block_layout_bytes.is_empty());
        assert!(!visuals.level.background_layout_bytes.is_empty());
    }

    struct TestBundle {
        root: TempDir,
        manifest: AssetManifest,
    }

    impl TestBundle {
        fn new() -> anyhow::Result<Self> {
            let root = TempDir::new()?;
            let foreground_bytes = (0..(6 * 4 * 20 * 14))
                .map(|value| (value % 16) as u8)
                .collect::<Vec<_>>();
            write_file(
                root.path(),
                "decoded/level/00/foreground.bin",
                &foreground_bytes,
            )?;
            write_file(
                root.path(),
                "decoded/level/00/foreground.bin.png",
                b"preview",
            )?;
            write_file(
                root.path(),
                "decoded/level/00/block.bin",
                &[0x20, 0x21, 0x22, 0x23],
            )?;
            write_file(root.path(), "decoded/level/00/block.bin.png", b"preview")?;
            write_file(
                root.path(),
                "decoded/level/00/background.bin",
                &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF],
            )?;
            write_file(
                root.path(),
                "decoded/level/00/background.bin.png",
                b"preview",
            )?;
            write_file(
                root.path(),
                "theme/artcomp_fg/cave.bin",
                &direct_copy_saxman(&(0u8..64).collect::<Vec<_>>()),
            )?;
            write_file(
                root.path(),
                "theme/artcomp_bg/cave.bin",
                &direct_copy_saxman(&(64u8..96).collect::<Vec<_>>()),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/Blocks.bin",
                &direct_copy_saxman(&(96u8..160).collect::<Vec<_>>()),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/HUD_numbers.bin",
                &direct_copy_saxman(&(0..384).map(|value| (value % 16) as u8).collect::<Vec<_>>()),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/Enemy01_Diamond.bin",
                &direct_copy_saxman(
                    &(0..4096)
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/Enemy1C_Mini_hopping_skull.bin",
                &direct_copy_saxman(
                    &(0..2048)
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/Horizontal_platform.bin",
                &direct_copy_saxman(
                    &(0..1024)
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/Vertical_platform.bin",
                &direct_copy_saxman(
                    &(0..1024)
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/artunc/diamond_1.bin",
                &(0..512).map(|value| (value % 16) as u8).collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/palette_enemy/Crystal_1.bin",
                &(0..15)
                    .map(|value| 0x0100_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/Enemy0C_Dragon.bin",
                &direct_copy_saxman(
                    &(0..16_384)
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/palette_enemy/Dragon_1.bin",
                &(0..15)
                    .map(|value| 0x0120_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/palette_enemy/Mini_Hopping_Skull_1.bin",
                &(0..15)
                    .map(|value| 0x0180_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/palette_kid/Kid.bin",
                &(0..15)
                    .map(|value| 0x0200_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid.bin",
                &vec![0x11; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_walk_1.bin",
                &vec![0x22; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_walk_2.bin",
                &vec![0x23; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_walk_3.bin",
                &vec![0x24; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_walk_4.bin",
                &vec![0x25; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_walk_5.bin",
                &vec![0x26; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_walk_6.bin",
                &vec![0x27; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_jumping_1.bin",
                &vec![0x31; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/kid_jumping_2.bin",
                &vec![0x32; 32 * 12],
            )?;
            write_file(
                root.path(),
                "theme/palette_fg/cave.bin",
                &(0..15)
                    .flat_map(|value| (value as u16).to_be_bytes())
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "theme/palette_bg/cave.bin",
                &(0..8)
                    .map(|value| 0x0100_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "theme/mappings/cave.bin",
                &[
                    0x000A_u16, 0x000B, 0x000C, 0x000D, 0x0010, 0x0011, 0x0012, 0x0013,
                ]
                .into_iter()
                .flat_map(u16::to_be_bytes)
                .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "theme/block_mappings.bin",
                &block_mapping_words()
                    .into_iter()
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "theme/bg_chunks/cave_00.bin",
                &[0x00, 0x02, 0x00, 0x01, 0x00, 0x01],
            )?;

            let manifest = AssetManifest {
                generated_bundle_version: GENERATED_BUNDLE_VERSION,
                rom_sha1: String::from("fixture"),
                upstream_commit: String::from("fixture"),
                source_root: root.path().to_path_buf(),
                output_root: root.path().to_path_buf(),
                map_order: Vec::new(),
                levels: vec![LevelRecord {
                    level_id: LEVEL_ID,
                    title_text_id: Some(0x4A),
                    display_name: Some(String::from("Blue Lake Woods 1")),
                    resource_paths: Vec::new(),
                    resources: LevelResources::default(),
                    header_metadata: Some(LevelHeader {
                        width_screens: 6,
                        height_screens: 4,
                        height_flag_bits: 0,
                        level_theme: CAVE_THEME_ID,
                        murder_wall_flags: 0,
                        background_theme: CAVE_THEME_ID,
                        weather_flags: 0,
                        player_start_x: 0x0750,
                        player_start_y: 0x00E0,
                        flag_x: 0x0750,
                        flag_y: 0x0300,
                    }),
                    enemy_layout_metadata: None,
                    bgscroll_metadata: None,
                    platform_metadata: None,
                }],
                constants: Vec::new(),
                ram_variables: Vec::new(),
                enemy_sources: Vec::new(),
                theme_assets: vec![
                    asset_file(root.path(), "theme/artcomp_fg/cave.bin"),
                    asset_file(root.path(), "theme/artcomp_bg/cave.bin"),
                    asset_file(root.path(), "theme/palette_fg/cave.bin"),
                    asset_file(root.path(), "theme/palette_bg/cave.bin"),
                    asset_file(root.path(), "theme/mappings/cave.bin"),
                    asset_file(root.path(), "theme/block_mappings.bin"),
                    asset_file(root.path(), "theme/bg_chunks/cave_00.bin"),
                ],
                scene_assets: Vec::new(),
                animation_assets: Vec::new(),
                sound_assets: Vec::new(),
                ingame_assets: vec![
                    asset_file(root.path(), "ingame/artcomp/Blocks.bin"),
                    asset_file(root.path(), "ingame/artcomp/HUD_numbers.bin"),
                    asset_file(root.path(), "ingame/artcomp/Enemy01_Diamond.bin"),
                    asset_file(root.path(), "ingame/artcomp/Enemy0C_Dragon.bin"),
                    asset_file(root.path(), "ingame/artcomp/Enemy1C_Mini_hopping_skull.bin"),
                    asset_file(root.path(), "ingame/artcomp/Horizontal_platform.bin"),
                    asset_file(root.path(), "ingame/artcomp/Vertical_platform.bin"),
                    asset_file(root.path(), "ingame/artunc/diamond_1.bin"),
                    asset_file(root.path(), "ingame/palette_enemy/Crystal_1.bin"),
                    asset_file(root.path(), "ingame/palette_enemy/Dragon_1.bin"),
                    asset_file(root.path(), "ingame/palette_enemy/Mini_Hopping_Skull_1.bin"),
                    asset_file(root.path(), "ingame/palette_kid/Kid.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_4.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_5.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_6.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_jumping_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_jumping_2.bin"),
                ],
                decoded_level1: Some(DecodedLevelAssets {
                    contract_version: LEVEL1_DECODED_ASSET_VERSION,
                    level_id: LEVEL_ID,
                    foreground: decoded_asset(root.path(), "decoded/level/00/foreground.bin"),
                    block: decoded_asset(root.path(), "decoded/level/00/block.bin"),
                    background: decoded_asset(root.path(), "decoded/level/00/background.bin"),
                }),
            };

            Ok(Self { root, manifest })
        }
    }

    fn write_file(root: &Path, relative_path: &str, bytes: &[u8]) -> anyhow::Result<()> {
        let path = root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
        Ok(())
    }

    fn asset_file(root: &Path, relative_path: &str) -> AssetFile {
        let bytes = fs::read(root.join(relative_path)).expect("asset bytes");
        let stored_relative_path = relative_path
            .strip_prefix("theme/")
            .or_else(|| relative_path.strip_prefix("ingame/"))
            .unwrap_or(relative_path);
        AssetFile {
            relative_path: stored_relative_path.to_owned(),
            size_bytes: bytes.len() as u64,
            sha1: sha1_hex(&bytes),
        }
    }

    fn decoded_asset(root: &Path, relative_path: &str) -> DecodedAssetFile {
        let bytes = fs::read(root.join(relative_path)).expect("asset bytes");
        DecodedAssetFile {
            relative_path: relative_path.to_owned(),
            preview_png_path: format!("{relative_path}.png"),
            size_bytes: bytes.len() as u64,
            sha1: sha1_hex(&bytes),
        }
    }

    fn direct_copy_saxman(bytes: &[u8]) -> Vec<u8> {
        let mut bits = vec![1_u8; bytes.len()];
        bits.extend([0, 1, 0, 0, 0, 1, 1]);
        let mut process = Vec::new();
        let mut current = 0_u8;
        for (index, bit) in bits.into_iter().enumerate() {
            current = (current << 1) | bit;
            if index % 8 == 7 {
                process.push(current);
                current = 0;
            }
        }
        let remaining = bytes.len() + 7;
        if remaining % 8 != 0 {
            current <<= 8 - (remaining % 8);
            process.push(current);
        }

        let input_offset = process.len() as u16;
        let mut compressed = Vec::with_capacity(2 + process.len() + bytes.len() + 2);
        compressed.extend(input_offset.to_be_bytes());
        compressed.extend(process);
        compressed.extend(bytes);
        compressed.extend([0x00, 0x00]);
        compressed
    }

    fn block_mapping_words() -> Vec<u16> {
        let mut words = vec![0x00BC_u16; 4];
        for group in 0..5_u16 {
            let group_base = 0x0200_u16 + group * 0x0100;
            for skin in 0..16_u16 {
                let tile_base = group_base + skin * 4;
                words.extend([tile_base, tile_base + 2, tile_base + 1, tile_base + 3]);
            }
        }
        words
    }

    fn sha1_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}
