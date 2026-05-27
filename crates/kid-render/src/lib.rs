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
pub use bundle::{NativeLevelVisuals, TitleCardLettersVisuals, load_level_visuals};
pub use hud::{HudIcon, HudVisuals};
pub use level::{
    BlockVisualKind, BlockVisualMapping, ForegroundLayout, LevelVisuals, ResolvedBlockVisual,
};
pub use palette::{MdColour, Palette, PaletteSet};
pub use player::{
    AnimatedSpriteVisual, EnemyVisuals, EolHologramVisuals, EolScoreVisuals, FlagVisuals,
    FloatingPrizeVisuals, LavaVisuals, MurderWallVisuals, PlatformVisuals, PlayerFormVisuals,
    PlayerTransformStep, PlayerVisuals, SpriteFrame,
};
pub use tiles::{Tile, TileQuad, TileRef, TileSheet};

#[cfg(test)]
mod tests {
    use super::{BackgroundVisuals, BlockVisualKind, load_level_visuals};
    use kid_data::{
        AssetFile, AssetManifest, DECODED_LEVEL_ASSET_VERSION, DecodedAssetFile,
        DecodedLevelAssets, GENERATED_BUNDLE_VERSION, LevelHeader, LevelRecord, LevelResources,
    };
    use sha1::{Digest, Sha1};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const LEVEL_ID: u8 = 0x00;
    const CAVE_THEME_ID: u8 = 0x08;
    const FOREST_THEME_ID: u8 = 0x09;
    const ENIGMA_COMMON_RUN_FIXTURE: &[u8] =
        &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7D, 0xFF, 0xE0, 0x00];
    const FLOATING_PRIZE_HELMET_FIXTURE_ASSETS: [(&str, &str, &str); 9] = [
        (
            "ingame/artunc_kid/skycutter_mask.bin",
            "ingame/palette_kid/Skycutter.bin",
            "ingame/palette_kid/Skycutter_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/cyclone_mask.bin",
            "ingame/palette_kid/Cyclone.bin",
            "ingame/palette_kid/Cyclone_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/red_stealth_mask.bin",
            "ingame/palette_kid/Red_Stealth.bin",
            "ingame/palette_kid/Red_Stealth_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/eyeclops_mask.bin",
            "ingame/palette_kid/Eyeclops.bin",
            "ingame/palette_kid/Eyeclops_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/juggernaut_mask.bin",
            "ingame/palette_kid/Juggernaut.bin",
            "ingame/palette_kid/Juggernaut_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/iron_knight_mask.bin",
            "ingame/palette_kid/Iron_Knight.bin",
            "ingame/palette_kid/Iron_Knight_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/berzerker_mask.bin",
            "ingame/palette_kid/Berzerker.bin",
            "ingame/palette_kid/Berzerker_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/maniaxe_mask.bin",
            "ingame/palette_kid/Maniaxe.bin",
            "ingame/palette_kid/Maniaxe_Helmet.bin",
        ),
        (
            "ingame/artunc_kid/micromax_mask.bin",
            "ingame/palette_kid/Micromax.bin",
            "ingame/palette_kid/Micromax_Helmet.bin",
        ),
    ];

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
        let murder_wall = visuals
            .murder_wall
            .as_ref()
            .expect("ASM Murderwall loc_140D2 art should load");
        assert_eq!(murder_wall.vram_base_tile_index, 0x02FB);
        assert_eq!(murder_wall.palette.colours.len(), 8);
        let lava = visuals
            .lava
            .as_ref()
            .expect("ASM Init_SpecialEffect_Lava loc_14C6A art should load");
        assert_eq!(lava.vram_base_tile_index, 0x02FB);
        assert_eq!(lava.palette.colours.len(), 8);
    }

    #[test]
    fn load_block_visual_mapping_uses_skin_byte_for_every_block_type() {
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
            .resolve(1, Some(0))
            .expect("smash block mapping");
        let prize = visuals
            .level
            .block_mapping
            .resolve(1, Some(1))
            .expect("prize block mapping");
        let ice = visuals
            .level
            .block_mapping
            .resolve(1, Some(2))
            .expect("ice block mapping");
        let metal = visuals
            .level
            .block_mapping
            .resolve(1, Some(5))
            .expect("metal block mapping");
        let rubber = visuals
            .level
            .block_mapping
            .resolve(1, Some(6))
            .expect("rubber block mapping");
        let shifting = visuals
            .level
            .block_mapping
            .resolve(1, Some(7))
            .expect("shifting block mapping");

        assert_eq!(terrain.kind, BlockVisualKind::Terrain);
        assert_eq!(terrain.quad.tiles[0].tile_index, 10);
        assert_eq!(terrain.quad.tiles[1].tile_index, 11);

        assert_eq!(smash.kind, BlockVisualKind::Smash);
        assert_eq!(smash.quad.tiles[0].tile_index, 0x0204);
        assert_eq!(prize.kind, BlockVisualKind::Prize);
        assert_eq!(prize.quad.tiles[0].tile_index, 0x0204);
        assert_eq!(ice.kind, BlockVisualKind::Ice);
        assert_eq!(ice.quad.tiles[0].tile_index, 0x0204);
        assert_eq!(metal.kind, BlockVisualKind::Metal);
        assert_eq!(metal.quad.tiles[0].tile_index, 0x0204);
        assert_eq!(rubber.kind, BlockVisualKind::Rubber);
        assert_eq!(rubber.quad.tiles[0].tile_index, 0x0204);
        assert_eq!(shifting.kind, BlockVisualKind::GenericBlock);
        assert_eq!(
            shifting.quad.tiles[0].tile_index, 0x0204,
            "ASM sub_30194 sets d4=$FF, so sub_302F6/loc_30336 and sub_3034A/loc_3037C use the skin byte directly for all block types"
        );
    }

    #[test]
    fn load_hud_visuals_returns_digit_and_icon_metadata() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.hud.digits.tiles.len(), 12);
        assert_eq!(visuals.hud.digit_base_tile_index, 0x06BA);
        assert_eq!(visuals.hud.diamond_tiles.tiles.len(), 16);
        assert_eq!(visuals.hud.diamond_base_tile_index, 0x06EA);
        assert_eq!(
            visuals.hud.hitpoint_sheet.tiles.len(),
            4,
            "ASM loc_12C34 DMA streams 4 HUD hitpoint tiles from ArtUnc_12C3C"
        );
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
        let kid = visuals.player.forms.get(&0).expect("Kid visuals");
        let red_stealth = visuals.player.forms.get(&3).expect("Red Stealth visuals");
        let juggernaut = visuals.player.forms.get(&5).expect("Juggernaut visuals");
        let iron_knight = visuals.player.forms.get(&6).expect("Iron Knight visuals");

        assert_eq!(kid.palette.colours.len(), 15);
        assert_eq!(kid.idle.origin_x, 0x0B);
        assert_eq!(kid.idle.origin_y, 0x01);
        assert_eq!(kid.idle.width_px, 0x14);
        assert_eq!(kid.idle.height_px, 0x20);
        assert_eq!(kid.idle.tile_sheet.tiles.len(), 12);
        assert_eq!(kid.walk.len(), 6);
        assert_eq!(kid.walk[0].origin_x, 0x0A);
        assert_eq!(kid.walk[0].width_px, 0x13);
        assert_eq!(kid.jump_rise.origin_x, 0x0A);
        assert_eq!(kid.jump_rise.width_px, 0x16);
        assert_eq!(kid.jump_fall.origin_x, 0x09);
        assert_eq!(kid.jump_fall.width_px, 0x14);
        let death = kid.death.as_ref().expect("Kid death frame");
        assert_eq!(death.origin_x, 0x0F);
        assert_eq!(death.origin_y, 0x03);
        assert_eq!(death.width_px, 0x1C);
        assert_eq!(
            death.height_px, 0x1B,
            "ASM Death loc_B7FA installs unk_A3FF8 / kid_death.bin for the visible tumble"
        );
        assert_eq!(kid.pull_up.len(), 7);
        assert_eq!(kid.pull_up[0].origin_x, 0x19);
        assert_eq!(kid.pull_up[0].width_px, 0x1C);
        assert_eq!(kid.idle.tile_base_index, 0);
        assert_eq!(red_stealth.slash_ground.len(), 4);
        assert_eq!(red_stealth.slash_ground[0].origin_x, 0x0E);
        assert_eq!(red_stealth.slash_ground[0].width_px, 0x18);
        assert_eq!(red_stealth.slash_air.len(), 4);
        assert_eq!(red_stealth.slash_air[0].origin_x, 0x0C);
        assert_eq!(red_stealth.slash_air[0].height_px, 0x28);

        assert_eq!(
            juggernaut.idle.width_px, 0x2F,
            "ASM unk_BEDF0 + unk_BF714 compose Juggernaut's split head/body idle sprite"
        );
        assert_eq!(juggernaut.idle.height_px, 0x20);
        assert_eq!(juggernaut.idle.tile_sheet.tiles.len(), 24);
        assert_eq!(juggernaut.walk.len(), 6);
        assert_eq!(
            juggernaut.jump_rise.height_px, 0x25,
            "ASM unk_BFAE0 pairs the taller rising body with the shared head"
        );
        assert_eq!(
            juggernaut
                .transform_palette
                .as_ref()
                .expect("transform palette")
                .colours
                .len(),
            15
        );
        assert_eq!(juggernaut.transform_steps.len(), 11);

        assert_eq!(iron_knight.palette.colours.len(), 15);
        assert_eq!(iron_knight.idle.origin_x, 0x13);
        assert_eq!(iron_knight.walk.len(), 6);
        assert_eq!(iron_knight.jump_rise.width_px, 0x1F);
        assert_eq!(
            iron_knight.jump_fall.origin_y, -0x02,
            "ASM Iron Knight jump-fall sprite descriptor stores origin_y=-$02"
        );
        assert_eq!(
            iron_knight
                .transform_palette
                .as_ref()
                .expect("transform palette")
                .colours
                .len(),
            15
        );
        assert_eq!(iron_knight.transform_steps.len(), 16);
        assert!(
            iron_knight
                .transform_steps
                .iter()
                .any(|step| step.frame.is_none())
        );
    }

    #[test]
    fn load_floating_prize_visuals_returns_rom_icon_frames() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.floating_prizes.diamond.frame_duration, 3);
        assert_eq!(visuals.floating_prizes.diamond.frames.len(), 4);
        for diamond in &visuals.floating_prizes.diamond.frames {
            assert_eq!(
                diamond.origin_y, -2,
                "ASM PrizeDiamond_Init frame unk_E0E66 places diamond art two pixels above object origin"
            );
            assert_eq!(
                diamond.tile_base_index, 0x08,
                "ASM PrizeDiamond_Init uses tile $6F2 relative to HUD diamond DMA base $6EA"
            );
        }

        for (content_id, origin_x, origin_y, width_px, height_px, tile_count) in [
            // ASM PrizeAnkh_Init -> stru_10C08 -> unk_E0FE6.
            (0x0B, 0x06, -2, 0x10, 0x10, 12),
            // ASM PrizeClock_Init -> stru_10C18 -> unk_E0FCE.
            (0x0C, 0x07, -2, 0x10, 0x10, 12),
            // ASM PrizeContinue_Init -> stru_10C28 -> unk_E0FD6.
            (0x0D, 0x07, -2, 0x10, 0x10, 24),
        ] {
            let visual = visuals
                .floating_prizes
                .icons
                .get(&content_id)
                .expect("floating prize icon visual");
            assert_eq!(visual.frame_duration, 1);
            assert_eq!(visual.frames.len(), 1);
            let frame = &visual.frames[0];
            assert_eq!(frame.origin_x, origin_x);
            assert_eq!(frame.origin_y, origin_y);
            assert_eq!(frame.width_px, width_px);
            assert_eq!(frame.height_px, height_px);
            assert_eq!(frame.tile_base_index, 0);
            assert_eq!(frame.tile_sheet.tiles.len(), tile_count);
        }

        for (content_id, origin_x, width_px, height_px) in [
            // ASM PrizeHelmetSkycutter_Init -> artunc_kid/skycutter_mask.bin.
            (0x02, 0x08, 0x11, 0x0E),
            // ASM PrizeHelmetCyclone_Init -> artunc_kid/cyclone_mask.bin.
            (0x03, 0x0B, 0x17, 0x0F),
            // ASM PrizeHelmetRedStealth_Init -> artunc_kid/red_stealth_mask.bin.
            (0x04, 0x0A, 0x15, 0x11),
            // ASM PrizeHelmetEyeclops_Init -> artunc_kid/eyeclops_mask.bin.
            (0x05, 0x0A, 0x15, 0x10),
            // ASM PrizeHelmetJuggernaut_Init -> artunc_kid/juggernaut_mask.bin.
            (0x06, 0x08, 0x12, 0x14),
            // ASM PrizeHelmetIronKnight_Init -> artunc_kid/iron_knight_mask.bin.
            (0x07, 0x09, 0x16, 0x15),
            // ASM PrizeHelmetBerzerker_Init -> artunc_kid/berzerker_mask.bin.
            (0x08, 0x09, 0x13, 0x14),
            // ASM PrizeHelmetManiaxe_Init -> artunc_kid/maniaxe_mask.bin.
            (0x09, 0x08, 0x10, 0x12),
            // ASM PrizeHelmetMicromax_Init -> artunc_kid/micromax_mask.bin.
            (0x0A, 0x09, 0x13, 0x12),
        ] {
            let visual = visuals
                .floating_prizes
                .helmets
                .get(&content_id)
                .expect("floating prize helmet visual");
            let frame = &visual.frames[0];

            assert_eq!(visual.frame_duration, 1);
            assert_eq!(frame.origin_x, origin_x);
            assert_eq!(
                frame.origin_y, -2,
                "ASM floating helmet sprite descriptors all place mask art two pixels above the object origin"
            );
            assert_eq!(frame.width_px, width_px);
            assert_eq!(frame.height_px, height_px);
            assert_eq!(
                visual.palette.colours.len(),
                15,
                "ASM sub_10CB0 copies three helmet palette words into slots $D..$F; native expands the extracted partial palette so mask pixels $D..$F resolve"
            );
            assert!(
                visual.palette.colours[12..15]
                    .iter()
                    .all(|colour| colour.raw_word != 0)
            );
        }
    }

    #[test]
    fn load_floating_prize_helmets_skip_incomplete_art_palette_pairs() {
        let mut fixture = TestBundle::new().expect("test bundle");
        fixture
            .manifest
            .ingame_assets
            .retain(|asset| asset.relative_path != "palette_kid/Cyclone_Helmet.bin");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert!(
            !visuals.floating_prizes.helmets.contains_key(&0x03),
            "ASM floating helmet visuals need both mask art and palette assets before native can render the pickup"
        );
        assert!(visuals.floating_prizes.helmets.contains_key(&0x02));
    }

    #[test]
    fn load_flag_visuals_returns_rom_flag_and_pole_frames() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(
            visuals.flag.frame_duration, 6,
            "ASM AniArt_Flag stores six-frame timing entries before each frame index"
        );
        assert_eq!(visuals.flag.waving.len(), 6);
        assert_eq!(
            visuals.flag.waving[0].tile_sheet.tiles.len(),
            120,
            "ASM AniArt_Flag uses six $280-byte frames from ArtUnc_4C92"
        );
        assert_eq!(visuals.flag.waving[0].tile_base_index, 0);
        assert_eq!(visuals.flag.waving[5].tile_base_index, 100);
        assert_eq!(visuals.flag.waving[0].origin_x, 0x06);
        assert_eq!(visuals.flag.waving[0].width_px, 0x28);
        assert_eq!(
            visuals.flag.pole_bottom.tile_sheet.tiles.len(),
            4,
            "ASM Load_FlagBottomArt copies $20 longwords from ArtUnc_12BA4"
        );
        assert_eq!(visuals.flag.pole_bottom.origin_x, 0x08);
        assert_eq!(visuals.flag.pole_bottom.height_px, 0x10);
    }

    #[test]
    fn load_enemy_and_platform_visuals_returns_visible_level1_families() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        let diamond = visuals
            .enemies
            .families
            .get(&0x01)
            .expect("diamond visuals");
        assert_eq!(diamond.frames.len(), 10);
        assert_eq!(diamond.frames[0].tile_base_index, 0x002F);
        assert_eq!(
            diamond.frames[5].tile_base_index, 0x0023,
            "ASM Diamond.asm loc_3AE28/loc_3AEE6 death frames stay appended after the live cycle"
        );
        let dragon = visuals.enemies.families.get(&0x0C).expect("dragon visuals");
        assert_eq!(dragon.frames.len(), 5);
        assert_eq!(dragon.frames[0].tile_base_index, 0x011D);
        let skull = visuals.enemies.families.get(&0x1C).expect("skull visuals");
        assert_eq!(skull.frames.len(), 8);
        assert_eq!(skull.frames[0].tile_base_index, 0x002C);
        assert_eq!(skull.frames[6].tile_base_index, 0x0020);
        assert_eq!(skull.frames[7].tile_base_index, 0x0024);
        let cloud = visuals.enemies.families.get(&0x0E).expect("cloud visuals");
        assert_eq!(cloud.frame_duration, 11);
        assert_eq!(
            cloud
                .frames
                .iter()
                .map(|frame| frame.origin_y)
                .collect::<Vec<_>>(),
            vec![0, -1, -1, -2, -1, -1, -1, 0, -2],
            "ASM Enemy0E Cloud live and death frame tables keep the bobbing offsets in the sprite descriptors"
        );
        assert_eq!(visuals.platforms.horizontal.width_px, 0x48);
        assert_eq!(visuals.platforms.horizontal.height_px, 0x10);
    }

    #[test]
    fn load_eol_score_visuals_returns_rom_label_and_digit_frames() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");
        let eol = visuals.eol_score.expect("EOL score visuals");

        assert_eq!(eol.time.tile_base_index, 0x0B9);
        assert_eq!(eol.time.width_px, 0x2D);
        assert_eq!(eol.bonus.tile_base_index, 0x000);
        assert_eq!(eol.bonus.width_px, 0x40);
        assert_eq!(eol.continues.tile_base_index, 0x008);
        assert_eq!(eol.continues.width_px, 0x6C);
        assert_eq!(eol.game.tile_base_index, 0x031);
        assert_eq!(eol.game.width_px, 0x5D);
        assert_eq!(eol.over.tile_base_index, 0x049);
        assert_eq!(eol.over.width_px, 0x60);
        assert_eq!(eol.digits.len(), 10);
        assert_eq!(eol.digits[0].tile_base_index, 0x0CB);
        assert_eq!(eol.digits[1].origin_x, -0x03);
        assert_eq!(
            eol.digits[9].tile_base_index, 0x061,
            "ASM Spr_EOL_Digits_Index maps digit 9 to Spr_EOL_Digit9",
        );
        let digit3 = &eol.digits[3];
        let right_tile = digit3
            .tile_sheet
            .tiles
            .get(usize::from(digit3.tile_base_index + 2))
            .expect("digit 3 right tile should keep its ROM VRAM slot");
        assert!(
            (0..8).all(|row| right_tile.pixels[row * 8 + 3..row * 8 + 8]
                .iter()
                .all(|&pixel| pixel == 0)),
            "ASM Spr_EOL_Digit3 is 11px wide, so native must not render packed score art past the declared bounds"
        );
        let bottom_tile = digit3
            .tile_sheet
            .tiles
            .get(usize::from(digit3.tile_base_index + 1))
            .expect("digit 3 bottom tile should keep its ROM VRAM slot");
        assert!(
            (3..8).all(|row| bottom_tile.pixels[row * 8..row * 8 + 8]
                .iter()
                .all(|&pixel| pixel == 0)),
            "ASM Spr_EOL_Digit3 is 11px tall, so native must clear the rounded sprite tile tail"
        );
    }

    #[test]
    fn load_eol_hologram_visuals_returns_loc_c186_tilemap_and_art() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");
        let hologram = visuals
            .eol_hologram
            .expect("ASM loc_C186 EOL hologram visuals");

        assert_eq!(
            hologram.palette.colours.len(),
            15,
            "ASM loc_C1BC copies the 15 visible words from Pal_D00C"
        );
        assert!(
            hologram.tile_sheet.tiles.len() > 0x0409,
            "ASM loc_C140 uploads HoloBlocks at tile $409 after HoloBG at tile $2FB"
        );
        assert_eq!(
            hologram.rows[0][0].tile_index, 0x02FB,
            "ASM loc_C140 decodes MapEni_CC0E with base tile word $42FB"
        );
        assert_eq!(
            hologram.rows[0][0].palette_line, 0,
            "the extracted Pal_D00C slice is standalone, so native normalises the ROM line-2 tile words to palette line 0 for rendering"
        );
    }

    #[test]
    fn load_title_card_letters_returns_rom_letter_sheet_and_palette() {
        let fixture = TestBundle::new().expect("test bundle");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");
        let letters = visuals
            .title_card_letters
            .expect("ASM Load_TitleCard loc_1AB26 letter visuals");

        assert_eq!(
            letters.tile_sheet.tiles.len(),
            35,
            "ASM off_1AC2A indexes 35 title-card letter/marker sprite frames"
        );
        assert_eq!(
            letters.palette.colours.len(),
            16,
            "ASM loc_1AB26 copies Pal_19C48 as one complete 16-colour line"
        );
        assert_eq!(letters.tile_sheet.tiles[0].pixels[0], 0);
        assert_eq!(letters.tile_sheet.tiles[34].pixels[63], 15);
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

        let error =
            load_level_visuals(&fixture.manifest, LEVEL_ID).expect_err("missing asset should fail");
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

    #[test]
    fn load_level_visuals_decodes_compressed_foreground_resource() {
        let mut fixture = TestBundle::new().expect("test bundle");
        fixture
            .enable_compressed_foreground_resource()
            .expect("compressed foreground fixture");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(
            visuals.level.foreground_layout_bytes,
            foreground_layout_fixture_bytes(),
            "ASM level foreground resources are Saxman-compressed before native parses the layout"
        );
    }

    #[test]
    fn load_level_visuals_decodes_layered_forest_background_planes() {
        let mut fixture = TestBundle::new().expect("test bundle");
        fixture
            .enable_layered_forest_background()
            .expect("layered forest fixture");

        let visuals = load_level_visuals(&fixture.manifest, LEVEL_ID).expect("visuals should load");

        assert_eq!(visuals.level.background_theme_id, FOREST_THEME_ID);
        let BackgroundVisuals::Layered(background) = visuals.background else {
            panic!("forest background should load through the layered renderer path");
        };
        assert_eq!(background.rows.len(), 1);
        assert_eq!(background.rows[0].tiles.len(), 64);
        assert_eq!(background.rows[0].tiles[0].tile_index, 0);
        assert_eq!(background.rows[0].tiles[32].tile_index, 1);
    }

    struct TestBundle {
        root: TempDir,
        manifest: AssetManifest,
    }

    impl TestBundle {
        fn new() -> anyhow::Result<Self> {
            let root = TempDir::new()?;
            write_file(
                root.path(),
                "decoded/level/00/foreground.bin",
                &foreground_layout_fixture_bytes(),
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
                "ingame/artcomp/Murder_wall.bin",
                &direct_copy_saxman(
                    &(0..(32 * 0x90))
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/palette/Murder_wall.bin",
                &(0..8)
                    .map(|value| 0x0400_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/Lava.bin",
                &direct_copy_saxman(
                    &(0..(32 * 0xC0))
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/palette/lava.bin",
                &(0..8)
                    .map(|value| 0x0500_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/artcomp/HUD_numbers.bin",
                &direct_copy_saxman(&(0..384).map(|value| (value % 16) as u8).collect::<Vec<_>>()),
            )?;
            write_file(
                root.path(),
                "ingame/artunc/Hitpoint_display.bin",
                &(0..128).map(|value| (value % 16) as u8).collect::<Vec<_>>(),
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
                "ingame/artcomp/Enemy0E_Cloud.bin",
                &direct_copy_saxman(
                    &(0..4096)
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "ingame/palette_enemy/Cloud_1.bin",
                &(0..15)
                    .map(|value| 0x01A0_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
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
            for (path, fill) in [
                ("ingame/artunc/diamond_2.bin", 0x2D),
                ("ingame/artunc/diamond_3.bin", 0x3D),
                ("ingame/artunc/diamond_4.bin", 0x4D),
            ] {
                write_file(root.path(), path, &vec![fill; 512])?;
            }
            write_file(
                root.path(),
                "ingame/artunc/Life_icon_(3_frames).bin",
                &vec![0x6B; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc/Clock_icon_(3_frames).bin",
                &vec![0x6C; 32 * 12],
            )?;
            write_file(
                root.path(),
                "ingame/artunc/Coin_continue_icon_(6_frames).bin",
                &vec![0x6D; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc/End_of_level_Flag_(6_frames).bin",
                &vec![0x6E; 32 * 120],
            )?;
            write_file(
                root.path(),
                "ingame/artunc/Flag_bottom.bin",
                &[0x6F; 32 * 4],
            )?;
            write_file(
                root.path(),
                "scenes/artcomp/End_level_stats_text.bin",
                &direct_copy_saxman(
                    &(0..(32 * 0xD8))
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "scenes/artcomp/Hologram_background.bin",
                &direct_copy_saxman(&[0x11; 32 * 2]),
            )?;
            write_file(
                root.path(),
                "scenes/artcomp/Hologram_blocks.bin",
                &direct_copy_saxman(&[0x22; 32 * 2]),
            )?;
            write_file(
                root.path(),
                "scenes/mapeni/hologram_background.bin",
                ENIGMA_COMMON_RUN_FIXTURE,
            )?;
            write_file(
                root.path(),
                "scenes/palette/Score_screen.bin",
                &(0..15)
                    .map(|value| 0x0340_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "scenes/artcomp/Title_card_letters.bin",
                &direct_copy_saxman(
                    &(0..(32 * 35))
                        .map(|value| (value % 16) as u8)
                        .collect::<Vec<_>>(),
                ),
            )?;
            write_file(
                root.path(),
                "scenes/palette/Title_card_letters.bin",
                &(0..16)
                    .map(|value| 0x0380_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
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
                "ingame/palette_kid/Iron_Knight.bin",
                &(0..15)
                    .map(|value| 0x0240_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/palette_kid/Iron_Knight_Transformation.bin",
                &(0..15)
                    .map(|value| 0x0280_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/palette_kid/Juggernaut.bin",
                &(0..15)
                    .map(|value| 0x02A0_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/palette_kid/Juggernaut_Transformation.bin",
                &(0..15)
                    .map(|value| 0x02B0_u16 + value)
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
                "ingame/artunc_kid/kid_death.bin",
                &vec![0x3A; 32 * 12],
            )?;
            for (path, tile_count, fill) in [
                ("ingame/artunc_kid/kid_flip_1.bin", 16, 0x33),
                ("ingame/artunc_kid/kid_flip_2.bin", 12, 0x34),
                ("ingame/artunc_kid/kid_flip_3.bin", 16, 0x35),
                ("ingame/artunc_kid/kid_flip_4.bin", 12, 0x36),
                ("ingame/artunc_kid/kid_flip_5.bin", 16, 0x37),
                ("ingame/artunc_kid/kid_flip_6.bin", 16, 0x38),
                ("ingame/artunc_kid/kid_flip_7.bin", 12, 0x39),
            ] {
                write_file(root.path(), path, &vec![fill; 32 * tile_count])?;
            }
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight.bin",
                &vec![0x41; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_walk_1.bin",
                &vec![0x42; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_walk_2.bin",
                &vec![0x43; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_walk_3.bin",
                &vec![0x44; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_walk_4.bin",
                &vec![0x45; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_walk_5.bin",
                &vec![0x46; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_walk_6.bin",
                &vec![0x47; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_jumping_1.bin",
                &vec![0x48; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_jumping_2.bin",
                &vec![0x49; 32 * 24],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_transformation_1.bin",
                &vec![0x4A; 32 * 40],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_transformation_2.bin",
                &vec![0x4B; 32 * 40],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_transformation_3.bin",
                &vec![0x4C; 32 * 48],
            )?;
            write_file(
                root.path(),
                "ingame/artunc_kid/iron_knight_transformation_4.bin",
                &vec![0x4D; 32 * 40],
            )?;
            for (path, tile_count, fill) in [
                ("ingame/artunc_kid/juggernaut_head.bin", 12, 0x4E),
                ("ingame/artunc_kid/juggernaut_head_walk_2.bin", 12, 0x4F),
                ("ingame/artunc_kid/juggernaut_head_walk_3.bin", 12, 0x50),
                ("ingame/artunc_kid/juggernaut_head_walk_4.bin", 12, 0x51),
                ("ingame/artunc_kid/juggernaut_head_walk_5.bin", 12, 0x52),
                ("ingame/artunc_kid/juggernaut_head_walk_6.bin", 12, 0x53),
                ("ingame/artunc_kid/juggernaut_bottom.bin", 12, 0x54),
                ("ingame/artunc_kid/juggernaut_bottom_move_1.bin", 12, 0x55),
                ("ingame/artunc_kid/juggernaut_bottom_move_2.bin", 12, 0x56),
                ("ingame/artunc_kid/juggernaut_bottom_move_3.bin", 12, 0x57),
                ("ingame/artunc_kid/juggernaut_bottom_move_4.bin", 12, 0x58),
                (
                    "ingame/artunc_kid/juggernaut_bottom_jumping_1.bin",
                    18,
                    0x59,
                ),
                (
                    "ingame/artunc_kid/juggernaut_bottom_jumping_3.bin",
                    18,
                    0x5A,
                ),
                (
                    "ingame/artunc_kid/juggernaut_transformation_1.bin",
                    25,
                    0x5B,
                ),
                (
                    "ingame/artunc_kid/juggernaut_transformation_2.bin",
                    25,
                    0x5C,
                ),
                (
                    "ingame/artunc_kid/juggernaut_transformation_3.bin",
                    30,
                    0x5D,
                ),
                (
                    "ingame/artunc_kid/juggernaut_transformation_4.bin",
                    48,
                    0x5E,
                ),
            ] {
                write_file(root.path(), path, &vec![fill; 32 * tile_count])?;
            }
            write_file(
                root.path(),
                "ingame/palette_kid/Red_Stealth.bin",
                &(0..15)
                    .map(|value| 0x02C0_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                root.path(),
                "ingame/palette_kid/Red_Stealth_Transformation.bin",
                &(0..15)
                    .map(|value| 0x0300_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            for (path, fill) in [
                ("ingame/artunc_kid/red_stealth.bin", 0x51),
                ("ingame/artunc_kid/red_stealth_walk_1.bin", 0x52),
                ("ingame/artunc_kid/red_stealth_walk_2.bin", 0x53),
                ("ingame/artunc_kid/red_stealth_walk_3.bin", 0x54),
                ("ingame/artunc_kid/red_stealth_walk_4.bin", 0x55),
                ("ingame/artunc_kid/red_stealth_walk_5.bin", 0x56),
                ("ingame/artunc_kid/red_stealth_walk_6.bin", 0x57),
                ("ingame/artunc_kid/red_stealth_jumping_1.bin", 0x58),
                ("ingame/artunc_kid/red_stealth_jumping_2.bin", 0x59),
                ("ingame/artunc_kid/red_stealth_transformation_1.bin", 0x5A),
                ("ingame/artunc_kid/red_stealth_transformation_2.bin", 0x5B),
                ("ingame/artunc_kid/red_stealth_slash_1.bin", 0x5C),
                ("ingame/artunc_kid/red_stealth_slash_2.bin", 0x5D),
                ("ingame/artunc_kid/red_stealth_slash_3.bin", 0x5E),
                ("ingame/artunc_kid/red_stealth_slash_4.bin", 0x5F),
                ("ingame/artunc_kid/red_stealth_slash_air_1.bin", 0x60),
                ("ingame/artunc_kid/red_stealth_slash_air_2.bin", 0x61),
                ("ingame/artunc_kid/red_stealth_slash_air_3.bin", 0x62),
                ("ingame/artunc_kid/red_stealth_slash_air_4.bin", 0x63),
            ] {
                write_file(root.path(), path, &vec![fill; 32 * 48])?;
            }
            for (index, (art_path, base_palette_path, palette_path)) in
                FLOATING_PRIZE_HELMET_FIXTURE_ASSETS.iter().enumerate()
            {
                let fill = 0x70 + u8::try_from(index).expect("helmet fixture index fits in u8");
                write_file(root.path(), art_path, &vec![fill; 32 * 24])?;
                write_file(
                    root.path(),
                    base_palette_path,
                    &(0..15)
                        .map(|value| (0x0390_u16 + u16::from(fill) + value).to_be_bytes())
                        .collect::<Vec<_>>()
                        .concat(),
                )?;
                write_file(
                    root.path(),
                    palette_path,
                    &(0..3)
                        .map(|value| (0x0490_u16 + u16::from(fill) + value).to_be_bytes())
                        .collect::<Vec<_>>()
                        .concat(),
                )?;
            }
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

            let mut manifest = AssetManifest {
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
                    title_card: None,
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
                scene_assets: vec![
                    asset_file(root.path(), "scenes/artcomp/End_level_stats_text.bin"),
                    asset_file(root.path(), "scenes/artcomp/Hologram_background.bin"),
                    asset_file(root.path(), "scenes/artcomp/Hologram_blocks.bin"),
                    asset_file(root.path(), "scenes/mapeni/hologram_background.bin"),
                    asset_file(root.path(), "scenes/palette/Score_screen.bin"),
                    asset_file(root.path(), "scenes/artcomp/Title_card_letters.bin"),
                    asset_file(root.path(), "scenes/palette/Title_card_letters.bin"),
                ],
                animation_assets: Vec::new(),
                sound_assets: Vec::new(),
                ingame_assets: vec![
                    asset_file(root.path(), "ingame/artcomp/Blocks.bin"),
                    asset_file(root.path(), "ingame/artcomp/Murder_wall.bin"),
                    asset_file(root.path(), "ingame/artcomp/Lava.bin"),
                    asset_file(root.path(), "ingame/artcomp/HUD_numbers.bin"),
                    asset_file(root.path(), "ingame/artunc/Hitpoint_display.bin"),
                    asset_file(root.path(), "ingame/artcomp/Enemy01_Diamond.bin"),
                    asset_file(root.path(), "ingame/artcomp/Enemy0C_Dragon.bin"),
                    asset_file(root.path(), "ingame/artcomp/Enemy0E_Cloud.bin"),
                    asset_file(root.path(), "ingame/artcomp/Enemy1C_Mini_hopping_skull.bin"),
                    asset_file(root.path(), "ingame/artcomp/Horizontal_platform.bin"),
                    asset_file(root.path(), "ingame/artcomp/Vertical_platform.bin"),
                    asset_file(root.path(), "ingame/artunc/diamond_1.bin"),
                    asset_file(root.path(), "ingame/artunc/diamond_2.bin"),
                    asset_file(root.path(), "ingame/artunc/diamond_3.bin"),
                    asset_file(root.path(), "ingame/artunc/diamond_4.bin"),
                    asset_file(root.path(), "ingame/artunc/Life_icon_(3_frames).bin"),
                    asset_file(root.path(), "ingame/artunc/Clock_icon_(3_frames).bin"),
                    asset_file(
                        root.path(),
                        "ingame/artunc/Coin_continue_icon_(6_frames).bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc/End_of_level_Flag_(6_frames).bin",
                    ),
                    asset_file(root.path(), "ingame/artunc/Flag_bottom.bin"),
                    asset_file(root.path(), "ingame/palette/Murder_wall.bin"),
                    asset_file(root.path(), "ingame/palette_enemy/Crystal_1.bin"),
                    asset_file(root.path(), "ingame/palette_enemy/Dragon_1.bin"),
                    asset_file(root.path(), "ingame/palette_enemy/Cloud_1.bin"),
                    asset_file(root.path(), "ingame/palette_enemy/Mini_Hopping_Skull_1.bin"),
                    asset_file(root.path(), "ingame/palette_kid/Kid.bin"),
                    asset_file(root.path(), "ingame/palette_kid/Iron_Knight.bin"),
                    asset_file(
                        root.path(),
                        "ingame/palette_kid/Iron_Knight_Transformation.bin",
                    ),
                    asset_file(root.path(), "ingame/palette_kid/Juggernaut.bin"),
                    asset_file(
                        root.path(),
                        "ingame/palette_kid/Juggernaut_Transformation.bin",
                    ),
                    asset_file(root.path(), "ingame/palette_kid/Red_Stealth.bin"),
                    asset_file(
                        root.path(),
                        "ingame/palette_kid/Red_Stealth_Transformation.bin",
                    ),
                    asset_file(root.path(), "ingame/artunc_kid/kid.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_4.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_5.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_walk_6.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_jumping_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_jumping_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_death.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_flip_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_flip_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_flip_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_flip_4.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_flip_5.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_flip_6.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/kid_flip_7.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_walk_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_walk_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_walk_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_walk_4.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_walk_5.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_walk_6.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_jumping_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/iron_knight_jumping_2.bin"),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/iron_knight_transformation_1.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/iron_knight_transformation_2.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/iron_knight_transformation_3.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/iron_knight_transformation_4.bin",
                    ),
                    asset_file(root.path(), "ingame/artunc_kid/juggernaut_head.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/juggernaut_head_walk_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/juggernaut_head_walk_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/juggernaut_head_walk_4.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/juggernaut_head_walk_5.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/juggernaut_head_walk_6.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/juggernaut_bottom.bin"),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_bottom_move_1.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_bottom_move_2.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_bottom_move_3.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_bottom_move_4.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_bottom_jumping_1.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_bottom_jumping_3.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_transformation_1.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_transformation_2.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_transformation_3.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/juggernaut_transformation_4.bin",
                    ),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_walk_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_walk_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_walk_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_walk_4.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_walk_5.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_walk_6.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_jumping_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_jumping_2.bin"),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/red_stealth_transformation_1.bin",
                    ),
                    asset_file(
                        root.path(),
                        "ingame/artunc_kid/red_stealth_transformation_2.bin",
                    ),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_4.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_air_1.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_air_2.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_air_3.bin"),
                    asset_file(root.path(), "ingame/artunc_kid/red_stealth_slash_air_4.bin"),
                ],
                decoded_levels: vec![DecodedLevelAssets {
                    contract_version: DECODED_LEVEL_ASSET_VERSION,
                    level_id: LEVEL_ID,
                    foreground: decoded_asset(root.path(), "decoded/level/00/foreground.bin"),
                    block: decoded_asset(root.path(), "decoded/level/00/block.bin"),
                    background: decoded_asset(root.path(), "decoded/level/00/background.bin"),
                }],
            };

            for (art_path, _base_palette_path, palette_path) in FLOATING_PRIZE_HELMET_FIXTURE_ASSETS
            {
                manifest.ingame_assets.extend([
                    asset_file(root.path(), art_path),
                    asset_file(root.path(), palette_path),
                ]);
            }

            Ok(Self { root, manifest })
        }

        fn enable_compressed_foreground_resource(&mut self) -> anyhow::Result<()> {
            write_file(
                self.root.path(),
                "level/00/foreground.sax",
                &direct_copy_saxman(&foreground_layout_fixture_bytes()),
            )?;
            self.manifest.levels[0].resources.foreground = Some("00/foreground.sax".to_owned());
            Ok(())
        }

        fn enable_layered_forest_background(&mut self) -> anyhow::Result<()> {
            write_file(
                self.root.path(),
                "decoded/level/00/background.bin",
                &[0x00, 0x00],
            )?;
            write_file(
                self.root.path(),
                "theme/artcomp_bg/forest.bin",
                &direct_copy_saxman(&[0; 64]),
            )?;
            write_file(
                self.root.path(),
                "theme/palette_bg/forest.bin",
                &(0..8)
                    .map(|value| 0x0180_u16 + value)
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            )?;
            write_file(
                self.root.path(),
                "theme/bg_chunks/forest_eni_00.bin",
                &layered_background_enigma_plane(0),
            )?;
            write_file(
                self.root.path(),
                "theme/bg_chunks/forest_eni_01.bin",
                &layered_background_enigma_plane(1),
            )?;

            self.manifest.levels[0]
                .header_metadata
                .as_mut()
                .expect("level header")
                .background_theme = FOREST_THEME_ID;
            self.manifest.decoded_levels[0].background =
                decoded_asset(self.root.path(), "decoded/level/00/background.bin");
            self.manifest.theme_assets.extend([
                asset_file(self.root.path(), "theme/artcomp_bg/forest.bin"),
                asset_file(self.root.path(), "theme/palette_bg/forest.bin"),
                asset_file(self.root.path(), "theme/bg_chunks/forest_eni_00.bin"),
                asset_file(self.root.path(), "theme/bg_chunks/forest_eni_01.bin"),
            ]);

            Ok(())
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

    fn foreground_layout_fixture_bytes() -> Vec<u8> {
        (0..(6 * 4 * 20 * 14))
            .map(|value| (value % 16) as u8)
            .collect::<Vec<_>>()
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
        if !remaining.is_multiple_of(8) {
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

    fn layered_background_enigma_plane(common_value: u16) -> Vec<u8> {
        let [common_hi, common_lo] = common_value.to_be_bytes();
        vec![
            0x01, 0x00, 0x00, 0x00, common_hi, common_lo, 0x7D, 0xFF, 0xE0, 0x00,
        ]
    }

    fn block_mapping_words() -> Vec<u16> {
        let mut words = Vec::new();
        for skin in 0..81_u16 {
            let tile_base = 0x0200_u16 + skin * 4;
            words.extend([tile_base, tile_base + 2, tile_base + 1, tile_base + 3]);
        }
        words
    }

    fn sha1_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}
