use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wildside_core::{
    ActiveProjectileKind, ActiveProjectileSnapshot, BlockBounceTraceKind,
    DiamondPowerSpriteSnapshot, DragonFlyingDescentFireState, GameMode, NativeObjectKind,
    NativeObjectSnapshot, NativeTraceFrame, PlayerForm,
    TOUCH_FLAGPOLE_GFX_CLEANUP_VISIBLE_FRAME_LOC_C0B6_ORACLE, physics::Direction,
    terrain::BlockGrid,
};
use wildside_render::{MdColour, NativeLevelVisuals, Palette, TileRef, TileSheet};

use crate::dto::*;
use crate::hud::*;
use crate::render::*;
use crate::sprites::{self, *};

fn game_mode_label(mode: GameMode) -> &'static str {
    match mode {
        GameMode::SegaScreen => "SegaScreen",
        GameMode::IntroSequence1 => "IntroSequence1",
        GameMode::IntroSequence2 => "IntroSequence2",
        GameMode::IntroSequence3 => "IntroSequence3",
        GameMode::IntroSequence4 => "IntroSequence4",
        GameMode::IntroSequence5 => "IntroSequence5",
        GameMode::IntroSequence6 => "IntroSequence6",
        GameMode::TitleScreen => "TitleScreen",
        GameMode::LevelTransition => "LevelTransition",
        GameMode::InGame => "InGame",
        GameMode::DemoPlay => "DemoPlay",
        GameMode::OptionsMenu => "OptionsMenu",
        GameMode::EndingSequence => "EndingSequence",
    }
}

fn form_label(helmet_id: u16) -> &'static str {
    match PlayerForm::from_helmet_id(helmet_id) {
        Some(PlayerForm::TheKid) => "TheKid",
        Some(PlayerForm::Skycutter) => "Skycutter",
        Some(PlayerForm::Cyclone) => "Cyclone",
        Some(PlayerForm::RedStealth) => "RedStealth",
        Some(PlayerForm::Eyeclops) => "Eyeclops",
        Some(PlayerForm::Juggernaut) => "Juggernaut",
        Some(PlayerForm::IronKnight) => "IronKnight",
        Some(PlayerForm::Berzerker) => "Berzerker",
        Some(PlayerForm::Maniaxe) => "Maniaxe",
        Some(PlayerForm::Micromax) => "Micromax",
        None => "Unknown",
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

const HEADYMETAL_WOBBLE_STAGE_1: u16 = 0x015E;
const HEADYMETAL_WOBBLE_STAGE_2: u16 = 0x00FA;
const HEADYMETAL_WOBBLE_STAGE_3: u16 = 0x0096;
const HEADYMETAL_WOBBLE_STAGE_4: u16 = 0x0064;
const FRIENDLY_PROJECTILE_COLOUR: [u8; 3] = [100, 255, 100];
const ENEMY_PROJECTILE_COLOUR: [u8; 3] = [255, 80, 40];
const SHOOTER_BLOCK_PROJECTILE_COLOUR: [u8; 3] = [255, 216, 72];
const FIRE_TRAIL_PLACEHOLDER_COLOUR: [u8; 3] = [255, 120, 32];
const FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME: u16 = 45;
const DRAGON_WALKING_BREATH_LIFETIME: u16 = 68;
const ROBOT_SHOT_IMPACT_LIFETIME_LOC_3A762: u16 = 3 + 4 + 8;
const VDP_SPRITE_SCREEN_BIAS_LOC_13C0: i32 = 0x80;
const MURDER_WALL_FALLBACK_BASE_COLOUR: [u8; 3] = [190, 20, 8];
const MURDER_WALL_FALLBACK_SHADOW_COLOUR: [u8; 3] = [65, 10, 10];
const MURDER_WALL_FALLBACK_HIGHLIGHT_COLOUR: [u8; 3] = [255, 160, 70];
const FIRE_DEMON_OBJECT_TYPE: u8 = 0x00;
const ROBOT_OBJECT_TYPE: u8 = 0x03;
const FLYING_DRAGON_OBJECT_TYPE: u8 = 0x0C;
const WALKING_DRAGON_OBJECT_TYPE: u8 = 0x0D;
const CLOUD_OBJECT_TYPE: u8 = 0x0E;
const LION_OBJECT_TYPE: u8 = 0x12;
const DRIPS_OBJECT_TYPE: u8 = 0x16;
const BOSS_EYE_OBJECT_TYPE: u8 = 0x21;
const TAR_MONSTER_OBJECT_TYPE: u8 = 0x05;
const PRIZE_GENERIC_LIFETIME_LOW_BYTE: u8 = 0x20;
const PRIZE_BLOCK_SPAWN_FLASH_TOTAL_FRAMES_LOC_102B8: u8 = 25;
// ASM loc_D734: after loc_D702 allocates the EOL row workers, the ROM runs
// eight one-frame palette fade steps before the score rows are visible in the
// reduced oracle bundle. The trace clock keeps loc_D468 object timing; the
// renderer applies this final visual fade phase.
pub const EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE: usize = 8;
// ASM loc_D734 falls through after the fade, then loc_C5A2/loc_C48C workers
// update before sub_13C0 latches the sprite table for the *next* displayed
// frame. The reduced `issue-175-eol-flag-touch-level12` oracle bundle exposes
// the visible frame through `eol_pre_frame_vdp_pixel_samples_loc_c070`, which
// is 15 worker ticks ahead of the trace's post-frame row counter.
const EOL_SCORE_ROWS_RENDER_LATCH_LEAD_FRAMES_LOC_C070_ORACLE: usize = 15;
// ASM loc_C0B6 cleanup is visible in trace object counts at elapsed 60, but
// pixels are composed from the pre-frame sprite table, so the flag sprites have
// already vanished from the rendered frame at elapsed 59.
const FLAGPOLE_SPRITES_RENDER_CLEANUP_FRAME_LOC_C0B6_ORACLE: usize =
    TOUCH_FLAGPOLE_GFX_CLEANUP_VISIBLE_FRAME_LOC_C0B6_ORACLE - 1;
const CONTINUE_SCREEN_INITIAL_HOLD_FRAMES_LOC_D31E: usize = 0x1C;
const EOL_TEXT_WORKER_SLIDE_FRAMES_LOC_C5A2: usize = 0x1C;
const PAUSE_MENU_TILE_X_LOC_69D2: i32 = 0x60;
const PAUSE_MENU_TILE_Y_RESTART_TEXT: i32 = 0x40;
const PAUSE_MENU_TILE_COLUMNS_LOC_69D2: usize = 16;
const PAUSE_MENU_TILE_ROWS_LOC_69D2: usize = 5;
const PAUSE_MENU_TEXT_X_LOC_69D2: i32 = 0x78;
const PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT: i32 = 0x48;
const PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT: i32 = 0x58;
const PAUSE_MENU_CURSOR_X_LOC_6A84: i32 = 0x68;
const PAUSE_MENU_CURSOR_TILE_LOC_6B0E: u16 = 0x0006;
const PAUSE_MENU_BLANK_TILE_LOC_6B0E: u16 = 0x0002;
const PRIZE_BLOCK_SPAWN_FLASH_FRAMES_STRU_102DE: [(u8, u16); 4] =
    [(4, 0x02CF), (6, 0x02D8), (10, 0x02E1), (4, 0x02EA)];
const PRIZE_BLOCK_VISIBLE_SKIN_LOC_12408: u8 = 0x02;
const SHIFTING_BLOCK_RISE_TILE_LOC_113F4: u16 = 0x0276;
const BACKGROUND_TILE_VRAM_BASE_LOC_128BA: u16 = 0x0780;
const ISLAND_SHORE_PRIMARY_VRAM_TILE_LOC_12098: u16 = 0xF600 / 32;
const ISLAND_SHORE_MID_VRAM_TILE_LOC_12110: u16 = 0xF6C0 / 32;
const ISLAND_SHORE_LOWER_VRAM_TILE_LOC_12188: u16 = 0xF760 / 32;
const SPECIAL_EFFECT_UNKNOWN_FOREGROUND_PLANE_REGISTER_LOC_14E42: u16 = 0x8218;
const KID_PROJECTILE_HIT_SPARK_FRAMES_STRU_1F40: [(u8, u16); 4] =
    [(1, 0x02CF), (1, 0x02D8), (1, 0x02E1), (1, 0x02EA)];
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
const PLANE_B_ROW_CELLS_LOC_11B7A: usize = 0x40;
const CHUNKED_BACKGROUND_VISIBLE_ROW_CELLS_LOC_307D8_30744: usize = 0x2A;
const LAVA_GEYSER_TOP_TEMPLATE_UNK_14624: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 2] = [
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0008,
        tile_base_word: 0xA33F,
        size_word: 0x0C00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0000,
        y_step_px: 0x0020,
        tile_base_word: 0xA399,
        size_word: 0x0F00,
    },
];
const LAVA_GEYSER_TOP_ALT_TEMPLATE_DWORD_14636: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8;
    3] = [
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0000,
        y_step_px: 0x0010,
        tile_base_word: 0xA3A9,
        size_word: 0x0D00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0000,
        y_step_px: 0x0008,
        tile_base_word: 0xA33B,
        size_word: 0x0C00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0020,
        tile_base_word: 0xA389,
        size_word: 0x0F00,
    },
];
const LAVA_GEYSER_MIDDLE_TEMPLATE_UNK_1465A: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 1] =
    [wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0010,
        tile_base_word: 0xA36E,
        size_word: 0x0D00,
    }];
const LAVA_GEYSER_BOTTOM_TEMPLATE_UNK_14650: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 1] =
    [wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0002,
        y_step_px: 0x0010,
        tile_base_word: 0xA3B1,
        size_word: 0x0D00,
    }];
const LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145CA: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 1] =
    [wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0006,
        y_step_px: 0x0008,
        tile_base_word: 0xA376,
        size_word: 0x0800,
    }];
const LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145D4: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 1] =
    [wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0018,
        tile_base_word: 0xA307,
        size_word: 0x0E00,
    }];
const LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145DE: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 1] =
    [wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0020,
        tile_base_word: 0xA379,
        size_word: 0x0F00,
    }];
const LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145E8: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 3] = [
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0000,
        y_step_px: 0x0010,
        tile_base_word: 0xA3A9,
        size_word: 0x0D00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0000,
        y_step_px: 0x0008,
        tile_base_word: 0xA33B,
        size_word: 0x0C00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0020,
        tile_base_word: 0xA389,
        size_word: 0x0F00,
    },
];
const LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_14602: [wildside_core::LavaGeyserSpriteTemplateLoc14BE8; 4] = [
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0002,
        y_step_px: 0x0010,
        tile_base_word: 0xA3B1,
        size_word: 0x0D00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0010,
        tile_base_word: 0xA36E,
        size_word: 0x0D00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0001,
        y_step_px: 0x0008,
        tile_base_word: 0xA33F,
        size_word: 0x0C00,
    },
    wildside_core::LavaGeyserSpriteTemplateLoc14BE8 {
        x_offset_px: 0x0000,
        y_step_px: 0x0020,
        tile_base_word: 0xA399,
        size_word: 0x0F00,
    },
];

const FIRE_DEMON_DEATH_TRAIL_FRAMES: [wildside_render::SpriteFrame; 6] = [
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
) -> wildside_render::SpriteFrame {
    wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet: wildside_render::TileSheet { tiles: Vec::new() },
    }
}

fn drips_drop_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    Some(wildside_render::SpriteFrame {
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

fn tar_monster_glob_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    Some(wildside_render::SpriteFrame {
        // ASM Tar_Monster.asm loc_3247E writes `LnkTo_unk_C781E` to the straight
        // glob. `unk_C781E` is `sprite_frame_vram $0BC,$04,$04,$1E,$08` (decoded
        // from kid.bin @ $C781E). This is the dark tar comet, not a red dot.
        origin_x: 0x04,
        origin_y: 0x04,
        width_px: 0x1E,
        height_px: 0x08,
        tile_base_index: 0x0BC,
        tile_sheet,
    })
}

fn tar_monster_burst_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    Some(wildside_render::SpriteFrame {
        // ASM Tar_Monster.asm loc_325E6 writes `LnkTo_unk_C782E` to each
        // retaliation fragment. `unk_C782E` is `sprite_frame_vram
        // $0C1,$03,$03,$06,$06` (decoded from kid.bin @ $C782E) — a small dark
        // tar blob.
        origin_x: 0x03,
        origin_y: 0x03,
        width_px: 0x06,
        height_px: 0x06,
        tile_base_index: 0x0C1,
        tile_sheet,
    })
}

fn tank_bullet_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed_frames = u16::MAX.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, width_px) = if elapsed_frames % 10 < 6 {
        // ASM Tank_Shooting.asm `stru_3B526`: first frame `unk_C7EE8`.
        (0x038, 0x09, 0x10)
    } else {
        // ASM Tank_Shooting.asm `stru_3B526`: second frame `unk_C7EF0`.
        (0x03A, 0x09, 0x0F)
    };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y: 0x00,
        width_px,
        height_px: 0x05,
        tile_base_index,
        tile_sheet,
    })
}

fn robot_shot_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
    remaining_frames: u16,
    animation_elapsed_frames: u16,
    impact: bool,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = if impact {
        let elapsed_frames = ROBOT_SHOT_IMPACT_LIFETIME_LOC_3A762.saturating_sub(remaining_frames);
        if elapsed_frames < 3 {
            // ASM Robot.asm `stru_3A762`: first impact frame `LnkTo_unk_C8260`.
            (0x000, 0x08, 0x00, 0x10, 0x10)
        } else if elapsed_frames < 7 {
            // ASM Robot.asm `stru_3A762`: second impact frame `LnkTo_unk_C8268`.
            (0x004, 0x0A, 0x00, 0x14, 0x14)
        } else {
            // ASM Robot.asm `stru_3A762`: final impact frame `LnkTo_unk_C8270`.
            (0x00D, 0x0B, 0x00, 0x16, 0x16)
        }
    } else {
        match (animation_elapsed_frames / 3) % 3 {
            // ASM Robot.asm `stru_3A754`: first flying frame `LnkTo_unk_C8278`.
            0 => (0x016, 0x03, 0x00, 0x1F, 0x06),
            // ASM Robot.asm `stru_3A754`: second flying frame `LnkTo_unk_C8280`.
            1 => (0x01A, 0x03, 0x00, 0x1F, 0x06),
            // ASM Robot.asm `stru_3A754`: third flying frame `LnkTo_unk_C8288`.
            _ => (0x01E, 0x03, 0x00, 0x17, 0x06),
        }
    };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn cloud_lightning_child_frame_loc_35df0(
    visuals: &wildside_render::AnimatedSpriteVisual,
    projectile: &ActiveProjectileSnapshot,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let (tile_base_index, origin_x, origin_y, width_px, height_px) =
        if projectile.x_velocity != 0 && projectile.y_velocity != 0 {
            // ASM Cloud.asm loc_35DF0 selects `LnkTo_unk_C7876` for diagonal
            // bolts: `sprite_frame_vram $00C,$00,$23,$23,$23`.
            (0x00C, 0x00, 0x23, 0x23, 0x23)
        } else if projectile.x_velocity != 0 {
            // ASM Cloud.asm loc_35DF0 selects `LnkTo_unk_C7866` for horizontal
            // bolts: `sprite_frame_vram $000,$00,$04,$30,$07`.
            (0x000, 0x00, 0x04, 0x30, 0x07)
        } else {
            // ASM Cloud.asm loc_35DF0 defaults to `LnkTo_unk_C786E` for vertical
            // bolts: `sprite_frame_vram $006,$04,$2F,$07,$30`.
            (0x006, 0x04, 0x2F, 0x07, 0x30)
        };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn dragon_walking_breath_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed_frames = DRAGON_WALKING_BREATH_LIFETIME.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = if elapsed_frames < 8 {
        // ASM Dragon.asm `stru_3CCC8`: first frame `LnkTo_unk_C7576`.
        (0x028, 0x0B, 0x00, 0x1B, 0x13)
    } else if ((elapsed_frames - 8) / 10).is_multiple_of(2) {
        // ASM Dragon.asm `stru_3CCC8`: alternating frame `LnkTo_unk_C757E`.
        (0x034, 0x0A, 0x00, 0x20, 0x15)
    } else {
        // ASM Dragon.asm `stru_3CCC8`: alternating frame `LnkTo_unk_C7586`.
        (0x040, 0x0A, 0x00, 0x20, 0x16)
    };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn dragon_flying_descent_fire_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
    projectile: &ActiveProjectileSnapshot,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let (tile_base_index, origin_x, origin_y, width_px, height_px) =
        match projectile.dragon_flying_descent_fire_state {
            // ASM Dragon_Flying.asm loc_34BAC/loc_34C26 installs
            // LnkTo_unk_C75CE for the first DBF loop.
            DragonFlyingDescentFireState::FirstAnimationLoc34C26 => (0x0EC, 0x05, 0x05, 0x13, 0x18),
            // ASM Dragon_Flying.asm loc_34C3A switches to LnkTo_unk_C75D6 for
            // the travelling flame after the startup counter underflows.
            DragonFlyingDescentFireState::TravelAnimationLoc34C3A
            | DragonFlyingDescentFireState::None => (0x0F5, 0x06, 0x07, 0x1B, 0x1F),
            // ASM Dragon_Flying.asm loc_34C4A uses LnkTo_unk_C75DE for the
            // first cleanup hibernation loop before switching to LnkTo_unk_C75E6.
            DragonFlyingDescentFireState::CleanupFirstLoc34C4A => (0x105, 0x0D, 0x05, 0x1F, 0x1B),
            DragonFlyingDescentFireState::CleanupSecondLoc34C4A => (0x115, 0x0F, 0x05, 0x20, 0x10),
        };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn dragon_walking_breath_child_facing_loc_3d02a(
    trace: &NativeTraceFrame,
    projectile: &ActiveProjectileSnapshot,
) -> Direction {
    trace
        .active_objects
        .iter()
        .find_map(|object| {
            if !matches!(
                object.kind,
                NativeObjectKind::Enemy {
                    object_type: WALKING_DRAGON_OBJECT_TYPE,
                }
            ) || object.hit_points == Some(0)
            {
                return None;
            }

            let dragon_y_px = object.y_pos >> 16;
            let breath_y_px = projectile.y_pos >> 16;
            if dragon_y_px.abs_diff(breath_y_px) > 1 {
                return None;
            }

            let dragon_x_px = object.x_pos >> 16;
            let breath_x_px = projectile.x_pos >> 16;
            match breath_x_px as i32 - dragon_x_px as i32 {
                // ASM Dragon.asm loc_3D02A copies parent x_direction into the
                // breath child, then loc_3D070 adds -$1C or +$1C to x_pos.
                // Dragon art is stored facing left, so native renders the
                // default left child unflipped and the right child flipped.
                -28 => Some(Direction::Right),
                28 => Some(Direction::Left),
                _ => None,
            }
        })
        .unwrap_or(Direction::Right)
}

fn fireball_trail_child_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed_frames = 0x0096_u16.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = match (elapsed_frames / 6) % 3
    {
        // ASM Fireball.asm `stru_330C2`: `LnkTo_unk_C7996`.
        0 => (0x070, 0x04, 0x05, 0x0C, 0x0D),
        // ASM Fireball.asm `stru_330C2`: `LnkTo_unk_C799E`.
        1 => (0x074, 0x04, 0x05, 0x0B, 0x0C),
        // ASM Fireball.asm `stru_330C2`: `LnkTo_unk_C79A6`.
        _ => (0x078, 0x04, 0x05, 0x0B, 0x0B),
    };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn drips_surface_crawler_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
    remaining_frames: u16,
    animation_elapsed_frames: u16,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = if animation_elapsed_frames > 0
    {
        let elapsed = 31_u16.saturating_sub(animation_elapsed_frames);
        match elapsed / 10 {
            // ASM Drips.asm loc_335D8 initialises `stru_33FA6` before the child
            // becomes mobile/hostile.
            0 => (0x0BD, 0x08, 0x00, 0x0F, 0x04), // LnkTo_unk_C7BAE
            1 => (0x0BF, 0x05, 0x00, 0x0F, 0x08), // LnkTo_unk_C7BB6
            _ => (0x0C1, 0x07, 0x00, 0x0E, 0x0C), // LnkTo_unk_C7BBE
        }
    } else {
        let elapsed = u16::MAX.wrapping_sub(remaining_frames);
        // ASM Drips.asm loc_33630 installs `stru_33F14` for the ordinary
        // horizontal crawler, not the tiny `off_33520` pool-surface table.
        drips_surface_crawler_stru_33f14_frame(elapsed)
    };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet,
    })
}

fn drips_surface_crawler_stru_33f14_frame(elapsed_frames: u16) -> (u16, i16, i16, u16, u16) {
    const C7B46: (u16, i16, i16, u16, u16) = (0x08D, 0x07, 0x00, 0x0F, 0x0F);
    const C7B3E: (u16, i16, i16, u16, u16) = (0x089, 0x07, 0x00, 0x0D, 0x0F);
    const C7B36: (u16, i16, i16, u16, u16) = (0x085, 0x07, 0x00, 0x0F, 0x10);
    const C7BE6: (u16, i16, i16, u16, u16) = (0x0DC, 0x07, 0x00, 0x0D, 0x0E);
    const STRU_33F14: [((u16, i16, i16, u16, u16), u16); 15] = [
        (C7B46, 0x0C),
        (C7B3E, 0x0C),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7B3E, 0x0C),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7B3E, 0x0C),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7BE6, 0x08),
        (C7B36, 0x0C),
        (C7B46, 0x0C),
        (C7BE6, 0x08),
        (C7B36, 0x0C),
    ];

    let total_frames = STRU_33F14
        .iter()
        .map(|(_, duration)| *duration)
        .sum::<u16>();
    let mut frame_offset = elapsed_frames % total_frames;
    for (frame, duration) in STRU_33F14 {
        if frame_offset < duration {
            return frame;
        }
        frame_offset -= duration;
    }
    C7B46
}

fn drips_surface_crawler_finish_frame(
    visuals: &wildside_render::AnimatedSpriteVisual,
    remaining_frames: u16,
) -> Option<wildside_render::SpriteFrame> {
    let tile_sheet = visuals.frames.first()?.tile_sheet.clone();
    let elapsed = 60_u16.saturating_sub(remaining_frames);
    let (tile_base_index, origin_x, origin_y, width_px, height_px) = match elapsed / 4 {
        // ASM Drips.asm loc_338F8 installs `stru_33F06` on the original
        // crawler before loc_3392C hibernates until the animation finishes.
        0 => (0x0C5, 0x07, 0x07, 0x10, 0x10), // LnkTo_unk_C7BC6
        1 => (0x0C9, 0x0A, 0x09, 0x17, 0x16), // LnkTo_unk_C7BCE
        _ => (0x0D2, 0x0B, 0x0C, 0x18, 0x18), // LnkTo_unk_C7BD6
    };
    Some(wildside_render::SpriteFrame {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
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
pub struct LayerFilter {
    pub background: bool,
    pub terrain: bool,
    pub block_overrides: bool,
    pub platforms: bool,
    pub enemies: bool,
    pub player: bool,
    pub effects: bool,
    pub eol_score: bool,
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
            eol_score: true,
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
            eol_score: false,
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
            "eol_score" => f.eol_score = true,
            "hud" => f.hud = true,
            _ => {}
        }
        f
    }
}

fn player_damage_flash_white_phase_loc_bc56(trace: &NativeTraceFrame) -> bool {
    if !trace.just_received_damage || trace.invulnerable_frames == 0 {
        return false;
    }
    // ASM loc_BC56 starts each damage-flash cycle by calling sub_80BC, then
    // loc_BC5C holds the white palette for two hibernated frames before
    // loc_BC6E restores the normal palette for four frames. Native traces
    // sample the countdown after the live damage shell has entered that
    // visible body, so the rendered phase is keyed from 64 remaining frames
    // rather than the full 66-frame latch duration.
    (64usize.saturating_sub(trace.invulnerable_frames) % 6) < 2
}

pub fn eol_score_sprite_diagnostics(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&wildside_core::terrain::BlockGrid>,
) -> Vec<NativeEolScoreSpriteDiagnostic> {
    // ASM Object_EOL_Number loc_C466 and Object_EOL_Text loc_C592 allocate
    // priority gfx-object slots for the score rows. This diagnostic exposes
    // native's rendered EOL score sprite commands so verifier reports can be
    // compared directly with oracle `eol_visible_gfx_slots_loc_d468`.
    let scene = build_scene_filtered(
        trace,
        false,
        Some(level_visuals),
        block_grid,
        LayerFilter::solo("eol_score"),
    );
    scene
        .commands
        .iter()
        .enumerate()
        .filter_map(|(command_index, command)| {
            let DrawCommand::Sprite { x, y, tiles, .. } = command else {
                return None;
            };
            if tiles.is_empty() {
                return None;
            }

            let min_offset_x = tiles.iter().map(|tile| tile.offset_x).min().unwrap_or(0);
            let min_offset_y = tiles.iter().map(|tile| tile.offset_y).min().unwrap_or(0);
            let max_offset_x = tiles
                .iter()
                .map(|tile| tile.offset_x.saturating_add(7))
                .max()
                .unwrap_or(0);
            let max_offset_y = tiles
                .iter()
                .map(|tile| tile.offset_y.saturating_add(7))
                .max()
                .unwrap_or(0);
            let hardware_sprites =
                eol_score_hardware_sprite_diagnostics(command_index, *x, *y, tiles);
            Some(NativeEolScoreSpriteDiagnostic {
                command_index,
                x: *x,
                y: *y,
                min_x: x.saturating_add(min_offset_x),
                min_y: y.saturating_add(min_offset_y),
                max_x: x.saturating_add(max_offset_x),
                max_y: y.saturating_add(max_offset_y),
                tile_count: tiles.len(),
                tiles: tiles
                    .iter()
                    .map(|tile| NativeSpriteTileDiagnostic {
                        offset_x: tile.offset_x,
                        offset_y: tile.offset_y,
                        tile_index: tile.tile.tile_index,
                        palette_line: tile.tile.palette_line,
                        priority: tile.tile.priority,
                        flip_x: tile.tile.flip_x,
                        flip_y: tile.tile.flip_y,
                    })
                    .collect(),
                hardware_sprites,
            })
        })
        .collect()
}

fn eol_score_hardware_sprite_diagnostics(
    command_index: usize,
    x: i32,
    y: i32,
    tiles: &[SpriteTile],
) -> Vec<NativeEolScoreHardwareSpriteDiagnostic> {
    // ASM Make_SpriteFromGfxObject loc_12B8/loc_12C0 splits each gfx frame
    // into max-32px hardware entries before sub_13C0 writes the sprite table.
    // `build_sprite_tiles` already preserves that chunk order, so regroup the
    // logical draw command to make native reports comparable with oracle
    // loc_C070 hardware sprite samples.
    let mut chunk_tiles: Vec<Vec<&SpriteTile>> = Vec::new();
    for tile in tiles {
        let chunk_x = tile.offset_x.div_euclid(32);
        let chunk_y = tile.offset_y.div_euclid(32);
        if let Some(existing) = chunk_tiles.iter_mut().find(|chunk| {
            chunk.first().is_some_and(|first| {
                first.offset_x.div_euclid(32) == chunk_x && first.offset_y.div_euclid(32) == chunk_y
            })
        }) {
            existing.push(tile);
        } else {
            chunk_tiles.push(vec![tile]);
        }
    }

    chunk_tiles
        .into_iter()
        .enumerate()
        .filter_map(|(chunk_index, chunk)| {
            let first = chunk.first()?;
            let min_offset_x = chunk.iter().map(|tile| tile.offset_x).min().unwrap_or(0);
            let min_offset_y = chunk.iter().map(|tile| tile.offset_y).min().unwrap_or(0);
            let max_offset_x = chunk
                .iter()
                .map(|tile| tile.offset_x.saturating_add(7))
                .max()
                .unwrap_or(0);
            let max_offset_y = chunk
                .iter()
                .map(|tile| tile.offset_y.saturating_add(7))
                .max()
                .unwrap_or(0);
            Some(NativeEolScoreHardwareSpriteDiagnostic {
                command_index,
                chunk_index,
                x: x.saturating_add(min_offset_x),
                y: y.saturating_add(min_offset_y),
                width: u16::try_from(max_offset_x.saturating_sub(min_offset_x) + 1).unwrap_or(0),
                height: u16::try_from(max_offset_y.saturating_sub(min_offset_y) + 1).unwrap_or(0),
                first_tile_index: first.tile.tile_index,
                min_x: x.saturating_add(min_offset_x),
                min_y: y.saturating_add(min_offset_y),
                max_x: x.saturating_add(max_offset_x),
                max_y: y.saturating_add(max_offset_y),
            })
        })
        .collect()
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
    let frame = wildside_render::SpriteFrame {
        tile_sheet: source_frame.tile_sheet.clone(),
        ..descriptor.clone()
    };
    // ASM Fire_Demon loc_3A63C copies the parent Fire Demon tile base/palette
    // and initialises `stru_3A622`, whose frames are
    // `unk_C7776..unk_C779E` (`sprite_frame_vram $0B0..$0BA`).
    push_sprite_frame(commands, &frame, &fire_demon_visuals.palette, sx, sy);
    true
}

fn shooter_block_projectile_sprite_tail_loc_ef62(
    projectile: &ActiveProjectileSnapshot,
) -> (i32, i32, u16) {
    if projectile.y_velocity < 0 {
        (-4, -8, 0x8295)
    } else if projectile.x_velocity > 0 {
        (0, -4, 0x8A96)
    } else if projectile.y_velocity > 0 {
        (-4, 0, 0x9295)
    } else {
        (-8, -4, 0x8296)
    }
}

fn push_shooter_block_projectile_sprite_loc_ef62(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    projectile: &ActiveProjectileSnapshot,
    sx: i32,
    sy: i32,
) -> bool {
    let (x_offset, y_offset, tile_word) = shooter_block_projectile_sprite_tail_loc_ef62(projectile);
    let tile_ref = TileRef::from_attr_word(tile_word);
    let Some(resolved) = resolve_ingame_vram_tile(level_visuals, tile_ref) else {
        return false;
    };

    commands.push(DrawCommand::Sprite {
        x: sx + x_offset,
        y: sy + y_offset,
        tile_sheet: resolved.tile_sheet,
        palette: resolved.palette,
        tiles: vec![SpriteTile {
            offset_x: 0,
            offset_y: 0,
            tile: resolved.tile,
        }],
    });
    true
}

fn push_lion_projectile_sprite_loc_3e12a(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    projectile: &ActiveProjectileSnapshot,
    sx: i32,
    sy: i32,
) -> bool {
    let Some(lion_visuals) = level_visuals.enemies.families.get(&LION_OBJECT_TYPE) else {
        return false;
    };
    // ASM Lion.asm loc_3E12A sets hardware `palette_line(a3)=2` before
    // initialising `stru_3DF34`: C7E20, C7E28, C7E30 for six frames each.
    // Native stores Lion's packed enemy palette as local slots, so use line 0
    // when indexing the extracted palette colours.
    let frame_index = 9 + usize::from(projectile.animation_elapsed_frames / 6) % 3;
    let Some(frame) = lion_visuals.frames.get(frame_index) else {
        return false;
    };
    let facing = if projectile.x_velocity < 0 {
        Direction::Left
    } else {
        Direction::Right
    };
    let (draw_x, draw_y) = sprite_draw_position(frame, sx, sy, facing);
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(lion_visuals.palette.clone()),
        tiles: build_sprite_tiles(frame, facing, 0),
    });
    true
}

pub fn build_scene(
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

pub fn render_trace_with_vdp_latched_state(
    trace: &NativeTraceFrame,
    previous_trace: Option<&NativeTraceFrame>,
) -> NativeTraceFrame {
    let mut render_trace = trace.clone();
    if matches!(
        trace.game_mode,
        GameMode::SegaScreen | GameMode::TitleScreen
    ) && trace.teleport_warp_colour == 0x082A
        && trace.level_entry_fade_progress_sub_73d0.is_some()
    {
        // ASM manual return loc_119D8 and timeout return loc_C0A change mode
        // before Pal_FadeOut loc_28C6 has finished presenting the resident
        // gameplay planes. Select the gameplay scene while retaining current
        // frozen snapshot state and the ROM palette-fade render command.
        render_trace.game_mode = GameMode::DemoPlay;
        render_trace.game_mode_value = GameMode::DemoPlay.as_disassembly_value();
        if let Some((camera_x_pos, camera_y_pos)) = trace.resident_gameplay_vdp_camera_loc_300ba {
            render_trace.camera_x_pos = camera_x_pos;
            render_trace.camera_y_pos = camera_y_pos;
        }
        // ASM loc_119D8 can leave Game_Paused set on the Start edge, but
        // loc_6E2 is blocked inside synchronous Pal_FadeOut rather than
        // returning to Mode_Level's loc_67E4 pause-menu renderer.
        render_trace.paused = false;
    }
    if matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
        && trace.level_complete
        && trace.level_complete_elapsed_frames == 0
        && let Some(previous_trace) = previous_trace
    {
        // ASM loc_C070 enters the flagpole hand-off after the frame's scroll
        // transfer has already latched Camera_X_pos/Camera_Y_pos into VDP
        // h/v-scroll state, so the first visible hand-off frame uses the
        // previous scroll/sprite latch while gameplay state has already
        // switched to LevelComplete and loc_C070 is about to clear object
        // links.
        render_trace.camera_x_pos = previous_trace.camera_x_pos;
        render_trace.camera_y_pos = previous_trace.camera_y_pos;
        render_trace
            .active_objects
            .clone_from(&previous_trace.active_objects);
        render_trace.frame_index = previous_trace.frame_index;
        render_trace.current_helmet_id = previous_trace.current_helmet_id;
        render_trace.cyclone_flying = previous_trace.cyclone_flying;
        render_trace.kid_facing = previous_trace.kid_facing;
        render_trace.kid_on_ground = previous_trace.kid_on_ground;
        render_trace.kid_on_slope = previous_trace.kid_on_slope;
        render_trace.kid_x_velocity = previous_trace.kid_x_velocity;
        render_trace.kid_y_velocity = previous_trace.kid_y_velocity;
        render_trace.form_change_flash_frames = previous_trace.form_change_flash_frames;
        render_trace.invulnerable_frames = previous_trace.invulnerable_frames;
        render_trace.death_windup_elapsed = previous_trace.death_windup_elapsed;
        render_trace.number_hitpoints = previous_trace.number_hitpoints;
        render_trace.extra_hitpoint_slots = previous_trace.extra_hitpoint_slots;
        render_trace.number_lives = previous_trace.number_lives;
        render_trace.diamond_count = previous_trace.diamond_count;
        render_trace.hud_diamond_frame_offset = previous_trace.hud_diamond_frame_offset;
        render_trace.hud_timer_sprite_seconds_loc_bee8 =
            previous_trace.hud_timer_sprite_seconds_loc_bee8;
        render_trace.level_timer_seconds = previous_trace.level_timer_seconds;
    }
    if matches!(trace.game_mode, GameMode::InGame | GameMode::DemoPlay)
        && !trace.level_complete
        && let Some(previous_trace) = previous_trace
        && previous_trace.game_mode == trace.game_mode
        && previous_trace.current_level == trace.current_level
    {
        // ASM Transfer_ScrollDataToVRAM loc_300BA writes Camera_X_pos and
        // Camera_Y_pos to VDP scroll before the next gameplay update becomes
        // visible. Trace state is post-update, so render with the previous
        // scroll latch. ASM Mode_Level also calls Transfer_SpriteAndKidToVRAM
        // before Execute_Objects/GfxObjects_MoveAndAnimate, then rebuilds the
        // next Sprite_Table at Make_SpritesFromGfxObjects near the end of the
        // frame. Non-player GfxObject sprites therefore present the previous
        // trace while current gameplay state remains authoritative.
        render_trace.camera_x_pos = previous_trace.camera_x_pos;
        render_trace.camera_y_pos = previous_trace.camera_y_pos;
        latch_previous_sprite_sources_loc_901(&mut render_trace, previous_trace);
        render_trace.kid_x_pos = previous_trace.kid_x_pos;
        render_trace.kid_y_pos = previous_trace.kid_y_pos;
        render_trace.frame_index = previous_trace.frame_index;
        render_trace.current_helmet_id = previous_trace.current_helmet_id;
        render_trace.cyclone_flying = previous_trace.cyclone_flying;
        render_trace.kid_facing = previous_trace.kid_facing;
        render_trace.kid_on_ground = previous_trace.kid_on_ground;
        render_trace.kid_on_slope = previous_trace.kid_on_slope;
        render_trace.kid_x_velocity = previous_trace.kid_x_velocity;
        render_trace.kid_y_velocity = previous_trace.kid_y_velocity;
        render_trace.character_movement = previous_trace.character_movement;
        render_trace.walk_visual_accumulator = previous_trace.walk_visual_accumulator;
        render_trace.form_change_flash_frames = previous_trace.form_change_flash_frames;
        // ASM Mode_Level calls Make_SpriteAttr_HUD/sub_BDC0 before the later
        // GfxObjects_CollisionKid path can reach Check_for_recent_damge
        // loc_B700, so the visible HP pips are also a pre-update latch.
        render_trace.number_hitpoints = previous_trace.number_hitpoints;
        render_trace.extra_hitpoint_slots = previous_trace.extra_hitpoint_slots;
        // ASM sub_44DC loc_45EE updates the diamond DMA script offset before
        // the next frame's Make_SpriteAttr_HUD/sub_BDC0-built symbol becomes
        // visible, so the HUD diamond page is latched from the previous trace.
        render_trace.hud_diamond_frame_offset = previous_trace.hud_diamond_frame_offset;
        render_trace.hud_timer_sprite_seconds_loc_bee8 =
            previous_trace.hud_timer_sprite_seconds_loc_bee8;
        if trace.form_change_flash_frames > 0 && previous_trace.form_change_flash_frames > 0 {
            // ASM Kid_Transform loc_7CD0 hibernates the player coroutine while
            // Mode_Level still reaches loc_80C/sub_44DC. Oracle frames show
            // the visible HUD diamond page remains one `unk_461C` art slot
            // behind that post-update trace value during those repeated
            // transform hibernates.
            render_trace.hud_diamond_frame_offset = previous_trace
                .hud_diamond_frame_offset
                .map(previous_hud_diamond_frame_offset_loc_45ee);
        }
        if trace.form_change_flash_frames > 0 && previous_trace.form_change_flash_frames == 0 {
            // ASM Kid_Transform loc_7C40/loc_7CD0 starts after the frame's
            // sprite attributes have already been built. The first visible
            // transform frame therefore keeps Kid's previous visual state
            // even though the post-update trace has already entered the
            // transform coroutine and adjusted the new form's collision width.
            render_trace.current_helmet_id = previous_trace.current_helmet_id;
            render_trace.kid_facing = previous_trace.kid_facing;
            render_trace.kid_on_ground = previous_trace.kid_on_ground;
            render_trace.kid_on_slope = previous_trace.kid_on_slope;
            render_trace.kid_x_velocity = previous_trace.kid_x_velocity;
            render_trace.kid_y_velocity = previous_trace.kid_y_velocity;
            render_trace.form_change_flash_frames = previous_trace.form_change_flash_frames;
            if let Some(previous_player) = previous_trace
                .active_objects
                .iter()
                .find(|object| object.kind == NativeObjectKind::Player)
                .cloned()
                && let Some(render_player) = render_trace
                    .active_objects
                    .iter_mut()
                    .find(|object| object.kind == NativeObjectKind::Player)
            {
                *render_player = previous_player;
            }
        }
    }
    render_trace
}

fn latch_previous_sprite_sources_loc_901(
    render_trace: &mut NativeTraceFrame,
    previous_trace: &NativeTraceFrame,
) {
    render_trace
        .active_objects
        .clone_from(&previous_trace.active_objects);
    render_trace
        .active_projectiles
        .clone_from(&previous_trace.active_projectiles);
    render_trace
        .active_projectile_details
        .clone_from(&previous_trace.active_projectile_details);
}

fn previous_hud_diamond_frame_offset_loc_45ee(offset: u8) -> u8 {
    match offset {
        // ASM sub_44DC loc_45EE cycles the four `unk_461C` HUD diamond pages;
        // only the first page wraps, all other offsets step back by one page.
        0 => 6,
        _ => offset.saturating_sub(2),
    }
}

pub fn build_scene_filtered(
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

pub fn build_transition_scene(
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

    commands.extend(build_title_card_worker_overlay_scene(trace, level_visuals).commands);
    push_teleport_warp_tint_loc_bcf6(&mut commands, trace);
    push_level_entry_palette_fade_sub_73d0(&mut commands, trace);

    Scene {
        background: [0, 0, 0],
        commands,
        pause_palette_darken: false,
    }
}

pub fn build_title_card_worker_overlay_scene(
    trace: &NativeTraceFrame,
    level_visuals: Option<&NativeLevelVisuals>,
) -> Scene {
    let mut commands = Vec::new();

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
            continue;
        }
        // Fallback: when the title-card letter art isn't loaded, render the
        // glyph as ASCII via the placeholder font. The per-glyph motion is
        // still ASM-faithful via the worker pose; only the letter art is
        // approximate.
        if let Some(ch) = title_glyph_to_ascii(glyph) {
            commands.push(text(sx, sy, &ch.to_string(), [255, 240, 200], 2));
        }
    }

    Scene {
        background: [0, 0, 0],
        commands,
        pause_palette_darken: false,
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

/// Composite the ROM-derived title-card letter workers over a VDP-rendered
/// frame. This is the one overlay that belongs on shell screens; both the
/// native player/capture pipeline and the web shell call it after
/// `wildside_md::render_vdp_into`, so the gating logic lives here once.
pub fn render_vdp_trace_overlay(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    frame: &mut [u8],
) {
    // ASM Load_IntroSequence4 (`sub_1B988`) sets `Camera_Y_pos=4` and allocates
    // a single object running `loc_1C7A0` with `IntroText4` — the holographic
    // background plus the story-text writer. It never allocates a player/Kid
    // GfxObjectSlot: the per-glyph kid montage poses belong to
    // `Load_IntroSequence3` (`unk_1AF24`/`loc_1AEEC`) and `Load_IntroSequence5`
    // (`loc_1BCD0`), not IntroSequence4. The previous overlay composited a
    // normal Kid sprite here from a stale carried `kid_gfx` pose
    // (kid_x/kid_y ~ $60/$D0), which drew a spurious Kid at the bottom-left of
    // the story screen (#210). No player overlay belongs on IntroSequence4.
    if trace.boot_phase.as_deref() != Some("starting_level")
        || trace.game_mode != GameMode::LevelTransition
    {
        return;
    }

    // ASM Load_TitleCard has two independent pieces: `Load_TitleCard` builds
    // the static VDP shell, then `loc_1AB26` / `loc_1AC70` animate
    // Title_Letter gfx workers over it. The VDP path owns the shell; this
    // overlay keeps the ROM-derived letter workers visible without restoring
    // the old invented static level-card text.
    let overlay = build_title_card_worker_overlay_scene(trace, Some(level_visuals));
    if !overlay.commands.is_empty() {
        render_scene_overlay(&overlay, frame);
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
    art: &wildside_render::TitleCardLettersVisuals,
    glyph: u8,
    sx: i32,
    sy: i32,
) -> bool {
    let Some(frame) = title_card_glyph_frame(glyph) else {
        return false;
    };
    let cols = frame.width_px / 8;
    let rows = frame.height_px / 8;
    let Some(last_tile_offset) = cols
        .checked_mul(rows)
        .and_then(|count| count.checked_sub(1))
    else {
        return false;
    };
    let Some(last_tile_index) = frame.tile_offset.checked_add(last_tile_offset) else {
        return false;
    };
    if usize::from(last_tile_index) >= art.tile_sheet.tiles.len() {
        return false;
    }
    let palette = Arc::new(art.palette.clone());
    let tile_sheet = Arc::new(art.tile_sheet.clone());
    for col in 0..cols {
        for row in 0..rows {
            // ASM sprite VRAM layout is column-major: tile_offset advances by
            // (rows) when moving to the next column. Mirroring the ROM's
            // column-first tile order keeps the rendered letter pixel-for-
            // pixel aligned with `sprite_frame_vram`.
            let tile_index = frame.tile_offset + (col * rows) + row;
            commands.push(DrawCommand::IndexedTile {
                x: sx + i32::from(col) * 8,
                y: sy + i32::from(row) * 8,
                tile_sheet: tile_sheet.clone(),
                tile: wildside_render::TileRef {
                    tile_index,
                    // The loaded `Title_card_letters.bin` palette is the
                    // complete 16-colour line that ASM loc_1AB40 copies to
                    // CRAM line 1. Native stores that standalone line at
                    // Palette slot 0; `palette_line: 0` lets the 16-colour
                    // resolver keep the ROM slot numbers while avoiding a
                    // second-line offset past the loaded data.
                    palette_line: 0,
                    priority: false,
                    flip_x: false,
                    flip_y: false,
                },
                palette: palette.clone(),
            });
        }
    }
    true
}

pub fn build_status_scene(trace: &NativeTraceFrame, debug_overlay_visible: bool) -> Scene {
    let mut commands = Vec::new();

    if trace.game_complete {
        return build_victory_scene(trace);
    }

    // Sega screen: ASM Load_SegaScreen (kid.asm:25444) decompresses
    // ArtComp_12F30_Sega into VRAM at $1780, loads Pal_12EEC (background
    // word $0000 = black) into CRAM, and writes the 4×12-tile logo at
    // nametable rows $061C/$069C/$071C/$079C. The interactive player
    // renders this through `wildside_md::render_vdp_into` (native_player::run);
    // this fallback path only fires in sweep / recording / TAS-viewer
    // contexts where the VDP snapshot is unavailable. The previous blue
    // background and 4×-scaled "SEGA" text were invented placeholders —
    // the ROM never renders a typed string, and palette index 0 is black,
    // not blue.
    if trace.game_mode == GameMode::SegaScreen {
        return Scene {
            background: [0, 0, 0],
            commands,
            pause_palette_darken: false,
        };
    }

    // Intro sequences: ASM `Load_IntroSequence1..5` / `Load_IntroSequence6`
    // (kid.asm:29207-29464) build tile-based VRAM compositions for each
    // segment — none of the slogans ("The Wild Side", "Heady Metal's
    // Challenge", "Enter the Game") exist as ROM-renderable ASCII text;
    // they're 4bpp tile art uploaded via `sub_1B850` / its per-segment
    // siblings. The interactive player routes through `wildside_md::render_vdp_into`
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
            pause_palette_darken: false,
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
    // `wildside_md::render_vdp_into` when `runtime.vdp_state()` is populated;
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

    if let Some(cursor) = trace.options_cursor {
        // ASM Obj_TitleMenu loc_1BB52 (kid.asm:30428): the ROM swaps the
        // pre-menu title prompt for the three-choice tilemap decoded from
        // `unk_2EAAE`; loc_1BC0E then moves the cursor through rows 0..2.
        let rows = ["1 PLAYER", "2 PLAYERS", "OPTIONS"];
        for (row, label) in rows.iter().enumerate() {
            let y = 100 + row as i32 * 20;
            let selected = row == usize::from(cursor.min(2));
            commands.push(text(
                116,
                y,
                label,
                if selected {
                    [255, 255, 255]
                } else {
                    [150, 150, 150]
                },
                1,
            ));
            if selected {
                commands.push(text(100, y, ">", [255, 220, 80], 1));
            }
        }
    } else if (trace.frame_index / 30).is_multiple_of(2) {
        // Blinking "PRESS START" (blink every 30 frames)
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
        pause_palette_darken: false,
    }
}

pub fn build_options_scene(trace: &NativeTraceFrame) -> Scene {
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

    let cursor = usize::from(trace.options_cursor.unwrap_or(0).min(3));
    let controller_label = if trace.options_two_player_two_controllers {
        "TWO CONTROLLERS"
    } else {
        "ONE CONTROLLER"
    };
    let controls = match trace.options_controls_scheme.min(5) {
        0 => ["SPEED", "JUMP", "SPECIAL"],
        1 => ["SPEED", "SPECIAL", "JUMP"],
        2 => ["JUMP", "SPEED", "SPECIAL"],
        3 => ["JUMP", "SPECIAL", "SPEED"],
        4 => ["SPECIAL", "SPEED", "JUMP"],
        _ => ["SPECIAL", "JUMP", "SPEED"],
    };
    let speed_lines = if trace.options_speed_button_fast {
        ["FAST ACTION", "PUSH FOR SLOW"]
    } else {
        ["NORMAL ACTION", "PUSH FOR FAST"]
    };

    let row_colour = |row: usize| {
        if row == cursor {
            [255, 255, 255]
        } else {
            [120, 120, 120]
        }
    };
    let option_text = |x_cell: i32, y_cell: i32, value: &str, colour: [u8; 3]| {
        text(x_cell * 8, y_cell * 8, value, colour, 1)
    };

    // ASM DrawOptText1..4 loc_1CE3A/loc_1CEB2/loc_1CF78/loc_1CFFE
    // (kid.asm:32034-32373): DrawTextLine_Offset adds the ROM base cells
    // `(7, 10)` to each packed text offset. The fallback still uses typed
    // glyphs, but it must preserve those cell positions so the Options screen
    // is readable instead of joining whole groups into over-wide rows.
    commands.push(option_text(7, 10, "2 PLAYERS", row_colour(0)));
    commands.push(option_text(19, 10, controller_label, row_colour(0)));

    commands.push(option_text(7, 13, "CONTROLS", row_colour(1)));
    commands.push(option_text(18, 13, "MODE", row_colour(1)));
    commands.push(option_text(12, 15, "A", row_colour(1)));
    commands.push(option_text(16, 15, controls[0], row_colour(1)));
    commands.push(option_text(12, 17, "B", row_colour(1)));
    commands.push(option_text(16, 17, controls[1], row_colour(1)));
    commands.push(option_text(12, 19, "C", row_colour(1)));
    commands.push(option_text(16, 19, controls[2], row_colour(1)));

    commands.push(option_text(7, 22, "SPEED BUTTON", row_colour(2)));
    commands.push(option_text(21, 22, speed_lines[0], row_colour(2)));
    commands.push(option_text(21, 24, speed_lines[1], row_colour(2)));

    commands.push(option_text(7, 27, "EXIT", row_colour(3)));

    Scene {
        background: [8, 12, 20],
        commands,
        pause_palette_darken: false,
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
        pause_palette_darken: false,
    }
}

pub fn build_victory_scene(trace: &NativeTraceFrame) -> Scene {
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
        pause_palette_darken: false,
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

pub fn build_ending_scene(trace: &NativeTraceFrame) -> Scene {
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
            pause_palette_darken: false,
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
        pause_palette_darken: false,
    }
}

pub fn resolve_ingame_vram_tile(
    level_visuals: &NativeLevelVisuals,
    tile_ref: TileRef,
) -> Option<ResolvedSceneTile> {
    resolve_ingame_vram_tile_with_foreground_palette(
        level_visuals,
        &level_visuals.palettes.foreground,
        tile_ref,
    )
}

fn resolve_ingame_vram_tile_with_foreground_palette(
    level_visuals: &NativeLevelVisuals,
    foreground_palette: &Palette,
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
            palette: Arc::new(foreground_palette.clone()),
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
            palette: Arc::new(foreground_palette.clone()),
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
            palette: Arc::new(foreground_palette.clone()),
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
            palette: Arc::new(foreground_palette.clone()),
        });
    }

    None
}

pub fn push_resolved_tile(
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

pub fn push_background_tiles(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let tile_sheet = Arc::new(background_tiles_for_trace_loc_1202a(trace, level_visuals));
    let palette = Arc::new(background_palette_for_scene(trace, level_visuals));

    match &level_visuals.background {
        wildside_render::BackgroundVisuals::Chunked(background) => {
            let chunked_palette_line =
                chunked_background_palette_line_loc_128ba(level_visuals.level.background_theme_id);
            for placement in &background.placements {
                let Some(chunk) = background.chunks.get(usize::from(placement.chunk_id)) else {
                    continue;
                };
                for row in 0..chunk.height_tiles {
                    let background_row = usize::from(placement.y_tiles + row);
                    if chunked_background_row_window_applies_loc_30744(
                        level_visuals,
                        background_row,
                    ) {
                        continue;
                    }
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
                                palette_line: chunked_palette_line,
                                priority: false,
                                flip_x: background_tile.flip_x,
                                flip_y: false,
                            },
                            palette: palette.clone(),
                        });
                    }
                }
            }

            let Some(plane_b) =
                ChunkedPlaneBRowsLoc307D8::from_background(level_visuals, background)
            else {
                return;
            };
            let background_scroll_y_px = camera_y_px >> 2;
            let first_row = (background_scroll_y_px.div_euclid(8)).max(0) as usize;
            let last_row =
                ((background_scroll_y_px + FRAME_HEIGHT as i32 + 7).div_euclid(8)).max(0) as usize;
            for row in first_row..=last_row {
                if !chunked_background_row_window_applies_loc_30744(level_visuals, row) {
                    continue;
                }
                let Some(cells) = plane_b.row_for_trace(trace, level_visuals, camera_x_px, row)
                else {
                    continue;
                };
                let screen_y = i32::try_from(row)
                    .unwrap_or(i32::MAX)
                    .saturating_mul(8)
                    .saturating_sub(background_scroll_y_px);
                if screen_y <= -8 || screen_y >= FRAME_HEIGHT as i32 {
                    continue;
                }
                let layer_speed = background_layer_speed_loc_3050a(level_visuals, row);
                let scroll_x_px = background_display_scroll_x_px_for_chunked_row_loc_3050a(
                    trace,
                    camera_x_px,
                    camera_y_px,
                    row,
                    first_row,
                    last_row,
                    layer_speed,
                );
                for (column, cell) in cells.iter().enumerate() {
                    let Some(cell) = cell else {
                        continue;
                    };
                    for screen_x in chunked_plane_b_cell_screen_xs_loc_3050a(column, scroll_x_px) {
                        if screen_x <= -8 || screen_x >= FRAME_WIDTH as i32 {
                            continue;
                        }
                        commands.push(DrawCommand::IndexedTile {
                            x: screen_x,
                            y: screen_y,
                            tile_sheet: tile_sheet.clone(),
                            tile: cell.tile_ref,
                            palette: palette.clone(),
                        });
                    }
                }
            }
        }
        wildside_render::BackgroundVisuals::Layered(background) => {
            let row_height_px = i32::try_from(background.rows.len())
                .ok()
                .and_then(|rows| rows.checked_mul(8));
            for (row_index, row) in background.rows.iter().enumerate() {
                let row_width_px = i32::try_from(row.tiles.len())
                    .ok()
                    .and_then(|tiles| tiles.checked_mul(8));
                let wrap_row = layered_background_row_wraps_for_trace_loc_3050a(
                    trace,
                    level_visuals,
                    row_index,
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
                    let screen_y = row_height_px.map_or(screen_y, |row_height_px| {
                        wrap_layered_background_y_loc_11b7a(screen_y, row_height_px)
                    });
                    let (screen_xs, screen_x_count) = if wrap_row {
                        row_width_px.map_or(([screen_x, 0, 0], 1), |row_width_px| {
                            (
                                layered_background_tile_screen_xs_loc_11b7a(screen_x, row_width_px),
                                3,
                            )
                        })
                    } else {
                        ([screen_x, 0, 0], 1)
                    };
                    for screen_x in screen_xs.into_iter().take(screen_x_count) {
                        if !layered_background_tile_horizontal_visible_sub_30598(
                            screen_x,
                            background.effect_start_row_selector,
                        ) || screen_y <= -8
                            || screen_y >= FRAME_HEIGHT as i32
                        {
                            continue;
                        }
                        push_layered_background_tile(
                            commands,
                            trace,
                            background.effect_start_row_selector,
                            level_visuals.level.background_theme_id,
                            camera_y_px,
                            screen_x,
                            screen_y,
                            tile_sheet.clone(),
                            palette.clone(),
                            *tile,
                        );
                    }
                }
            }
        }
    }
}

fn background_tiles_for_trace_loc_1202a(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> TileSheet {
    let Some((primary_frame, mid_frame, lower_frame)) = trace.island_shore_vram_frames_loc_1202a
    else {
        return level_visuals.background_tiles.clone();
    };
    let Some(shore) = &level_visuals.island_shore_tiles_loc_1202a else {
        return level_visuals.background_tiles.clone();
    };

    let mut tile_sheet = level_visuals.background_tiles.clone();
    // ASM loc_1202A/loc_12098/loc_12110/loc_12188 stream three art bands
    // from `off_1220E` into VRAM `$F600/$F6C0/$F760`. Scroll B art starts at
    // `$780`, so `$F600 / 32 = $7B0` appears at relative background tile `$30`.
    patch_island_shore_band_loc_1202a(
        &mut tile_sheet,
        shore,
        primary_frame,
        0,
        usize::from(ISLAND_SHORE_PRIMARY_VRAM_TILE_LOC_12098 - BACKGROUND_TILE_VRAM_BASE_LOC_128BA),
        6,
    );
    patch_island_shore_band_loc_1202a(
        &mut tile_sheet,
        shore,
        mid_frame,
        6,
        usize::from(ISLAND_SHORE_MID_VRAM_TILE_LOC_12110 - BACKGROUND_TILE_VRAM_BASE_LOC_128BA),
        5,
    );
    patch_island_shore_band_loc_1202a(
        &mut tile_sheet,
        shore,
        lower_frame,
        11,
        usize::from(ISLAND_SHORE_LOWER_VRAM_TILE_LOC_12188 - BACKGROUND_TILE_VRAM_BASE_LOC_128BA),
        12,
    );
    tile_sheet
}

fn patch_island_shore_band_loc_1202a(
    tile_sheet: &mut TileSheet,
    shore: &wildside_render::IslandShoreVisualsLoc1202A,
    frame_index: u16,
    source_tile_index: usize,
    destination_tile_index: usize,
    tile_count: usize,
) {
    let Some(frame) = island_shore_frame_loc_1220e(shore, frame_index) else {
        return;
    };
    if tile_sheet.tiles.len() < destination_tile_index + tile_count {
        tile_sheet.tiles.resize(
            destination_tile_index + tile_count,
            wildside_render::Tile { pixels: [0; 64] },
        );
    }
    for offset in 0..tile_count {
        let Some(tile) = frame.tiles.get(source_tile_index + offset) else {
            break;
        };
        tile_sheet.tiles[destination_tile_index + offset] = tile.clone();
    }
}

fn island_shore_frame_loc_1220e(
    shore: &wildside_render::IslandShoreVisualsLoc1202A,
    frame_index: u16,
) -> Option<&TileSheet> {
    // ASM off_1220E: 1, 2, 3, 4, 3, 2 (`unk_9784A`, `unk_97B2C`,
    // `unk_97E0E`, `unk_980F0`, `unk_97E0E`, `unk_97B2C`).
    const FRAME_TABLE_LOC_1220E: [usize; 6] = [0, 1, 2, 3, 2, 1];
    let asset_index = FRAME_TABLE_LOC_1220E.get(usize::from(frame_index))?;
    shore.frames.get(*asset_index)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundPixelSample {
    pub screen_x: i32,
    pub screen_y: i32,
    pub tile_origin_x: i32,
    pub tile_origin_y: i32,
    pub background_row_index: Option<usize>,
    pub background_row_selector: Option<u8>,
    pub background_column: Option<usize>,
    pub tile_index: u16,
    pub palette_line: u8,
    pub palette_index: u8,
    pub source_x: u8,
    pub source_y: u8,
    pub source_row_pixels: [u8; 8],
    pub row_offset: Option<i32>,
    pub rgb: [u8; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundScrollPixelDiagnostic {
    pub camera_x_px: i32,
    pub camera_y_px: i32,
    pub time_frames_loc_3038a: usize,
    pub background_scroll_y_px: i32,
    pub sample_row: usize,
    pub plane_b_row: usize,
    pub sample_column: i32,
    pub layer_speed_loc_3050a: u8,
    pub scroll_x_px_loc_3050a: i32,
    pub destination_column_loc_30744: usize,
    pub source_column_bias_loc_30744: usize,
    pub source_column_start_loc_30744: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fresh_window_offset_loc_30744: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_source_column_loc_30744: Option<usize>,
}

pub fn background_scroll_pixel_diagnostic_loc_3050a(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    sample_x: i32,
    sample_y: i32,
) -> Option<BackgroundScrollPixelDiagnostic> {
    let wildside_render::BackgroundVisuals::Chunked(_) = &level_visuals.background else {
        return None;
    };
    level_visuals.level.background_scroll.as_ref()?;

    let (background_width, _) = chunked_background_dimensions_loc_12848(level_visuals)?;
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let background_scroll_y_px = camera_y_px >> 2;
    let sample_row = (sample_y + background_scroll_y_px).div_euclid(8);
    if sample_row < 0 {
        return None;
    }
    let sample_row = usize::try_from(sample_row).ok()?;
    let layer_speed = background_layer_speed_loc_3050a(level_visuals, sample_row);
    let first_row = background_scroll_y_px.div_euclid(8).max(0) as usize;
    let last_row =
        ((background_scroll_y_px + FRAME_HEIGHT as i32 + 7).div_euclid(8)).max(0) as usize;
    let scroll_x_px = background_display_scroll_x_px_for_chunked_row_loc_3050a(
        trace,
        camera_x_px,
        camera_y_px,
        sample_row,
        first_row,
        last_row,
        layer_speed,
    );
    // ASM BackgroundScroll_ComputeShiftLayers loc_3050A supplies the scroll
    // column, while sub_30744/sub_307D8 refresh the exposed Plane-B window.
    let destination_column = scroll_x_px
        .div_euclid(8)
        .rem_euclid(i32::try_from(PLANE_B_ROW_CELLS_LOC_11B7A).ok()?)
        as usize;
    let source_column_bias = 1_i32;
    let source_column_start = (scroll_x_px.div_euclid(8) + source_column_bias)
        .rem_euclid(i32::try_from(background_width).ok()?) as usize;
    let (sample_column, _) = chunked_plane_b_cell_at_sample_x_loc_3050a(sample_x, scroll_x_px)?;
    let fresh_window_offset = Some(
        (sample_column + PLANE_B_ROW_CELLS_LOC_11B7A - destination_column)
            % PLANE_B_ROW_CELLS_LOC_11B7A,
    )
    .filter(|offset| *offset < CHUNKED_BACKGROUND_VISIBLE_ROW_CELLS_LOC_307D8_30744);
    let sample_source_column =
        fresh_window_offset.map(|offset| (source_column_start + offset) % background_width);

    Some(BackgroundScrollPixelDiagnostic {
        camera_x_px,
        camera_y_px,
        time_frames_loc_3038a: background_time_frames_for_trace_loc_3050a(trace),
        background_scroll_y_px,
        sample_row,
        plane_b_row: sample_row & 0x1F,
        sample_column: i32::try_from(sample_column).ok()?,
        layer_speed_loc_3050a: layer_speed,
        scroll_x_px_loc_3050a: scroll_x_px,
        destination_column_loc_30744: destination_column,
        source_column_bias_loc_30744: source_column_bias as usize,
        source_column_start_loc_30744: source_column_start,
        fresh_window_offset_loc_30744: fresh_window_offset,
        sample_source_column_loc_30744: sample_source_column,
    })
}

pub fn sample_background_pixel_loc_c070(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    sample_x: i32,
    sample_y: i32,
) -> Option<BackgroundPixelSample> {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let palette = background_palette_for_scene(trace, level_visuals);
    let tile_sheet = background_tiles_for_trace_loc_1202a(trace, level_visuals);
    let mut sample = None;

    match &level_visuals.background {
        wildside_render::BackgroundVisuals::Chunked(background) => {
            let chunked_palette_line =
                chunked_background_palette_line_loc_128ba(level_visuals.level.background_theme_id);
            for placement in &background.placements {
                let Some(chunk) = background.chunks.get(usize::from(placement.chunk_id)) else {
                    continue;
                };
                for row in 0..chunk.height_tiles {
                    let background_row = usize::from(placement.y_tiles + row);
                    if chunked_background_row_window_applies_loc_30744(
                        level_visuals,
                        background_row,
                    ) {
                        continue;
                    }
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
                        let tile_ref = TileRef {
                            tile_index: background_tile.tile_index,
                            palette_line: chunked_palette_line,
                            priority: false,
                            flip_x: background_tile.flip_x,
                            flip_y: false,
                        };
                        if let Some(hit) = sample_background_tile_pixel(
                            &tile_sheet,
                            &palette,
                            tile_ref,
                            screen_x,
                            screen_y,
                            sample_x,
                            sample_y,
                            None,
                            None,
                            screen_x,
                            screen_y,
                            Some(background_row),
                            None,
                            Some(usize::from(placement.x_tiles + column)),
                        ) {
                            sample = Some(hit);
                        }
                    }
                }
            }

            if let Some(plane_b) =
                ChunkedPlaneBRowsLoc307D8::from_background(level_visuals, background)
            {
                let background_scroll_y_px = camera_y_px >> 2;
                let row = (sample_y + background_scroll_y_px).div_euclid(8);
                if row >= 0
                    && chunked_background_row_window_applies_loc_30744(level_visuals, row as usize)
                    && let Some(cells) =
                        plane_b.row_for_trace(trace, level_visuals, camera_x_px, row as usize)
                {
                    let layer_speed = background_layer_speed_loc_3050a(level_visuals, row as usize);
                    let first_row = background_scroll_y_px.div_euclid(8).max(0) as usize;
                    let last_row = ((background_scroll_y_px + FRAME_HEIGHT as i32 + 7)
                        .div_euclid(8))
                    .max(0) as usize;
                    let scroll_x_px = background_display_scroll_x_px_for_chunked_row_loc_3050a(
                        trace,
                        camera_x_px,
                        camera_y_px,
                        row as usize,
                        first_row,
                        last_row,
                        layer_speed,
                    );
                    if let Some((column, screen_x)) =
                        chunked_plane_b_cell_at_sample_x_loc_3050a(sample_x, scroll_x_px)
                        && let Some(cell) = cells[column]
                    {
                        let screen_y = row * 8 - background_scroll_y_px;
                        if let Some(hit) = sample_background_tile_pixel(
                            &tile_sheet,
                            &palette,
                            cell.tile_ref,
                            screen_x,
                            screen_y,
                            sample_x,
                            sample_y,
                            None,
                            None,
                            screen_x,
                            screen_y,
                            Some(cell.background_row_index),
                            None,
                            Some(cell.background_column),
                        ) {
                            sample = Some(hit);
                        }
                    }
                }
            }
        }
        wildside_render::BackgroundVisuals::Layered(background) => {
            let row_height_px = i32::try_from(background.rows.len())
                .ok()
                .and_then(|rows| rows.checked_mul(8));
            for (row_index, row) in background.rows.iter().enumerate() {
                let row_width_px = i32::try_from(row.tiles.len())
                    .ok()
                    .and_then(|tiles| tiles.checked_mul(8));
                let wrap_row = layered_background_row_wraps_for_trace_loc_3050a(
                    trace,
                    level_visuals,
                    row_index,
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
                    let screen_y = row_height_px.map_or(screen_y, |row_height_px| {
                        wrap_layered_background_y_loc_11b7a(screen_y, row_height_px)
                    });
                    let (screen_xs, screen_x_count) = if wrap_row {
                        row_width_px.map_or(([screen_x, 0, 0], 1), |row_width_px| {
                            (
                                layered_background_tile_screen_xs_loc_11b7a(screen_x, row_width_px),
                                3,
                            )
                        })
                    } else {
                        ([screen_x, 0, 0], 1)
                    };
                    for screen_x in screen_xs.into_iter().take(screen_x_count) {
                        if let Some(hit) = sample_layered_background_tile_pixel(
                            trace,
                            level_visuals,
                            &tile_sheet,
                            &palette,
                            background.effect_start_row_selector,
                            camera_y_px,
                            screen_x,
                            screen_y,
                            row_index,
                            row.selector,
                            column,
                            *tile,
                            sample_x,
                            sample_y,
                        ) {
                            sample = Some(hit);
                        }
                    }
                }
            }
        }
    }

    sample
}

fn chunked_background_palette_line_loc_128ba(background_theme_id: u8) -> u8 {
    // ASM Load_BackgroundMappings_Layered loc_128BA decodes Forest with
    // base `$0780`, so zero palette bits read foreground CRAM line 0. Other
    // chunked backgrounds rendered by sub_307D8 use `$2780`-based words and
    // therefore read background CRAM line 1.
    if background_theme_id == 9 { 0 } else { 1 }
}

fn chunked_plane_b_row_width_px_loc_11b7a() -> i32 {
    i32::try_from(PLANE_B_ROW_CELLS_LOC_11B7A)
        .unwrap_or(0x40)
        .saturating_mul(8)
}

// ASM BackgroundScroll_ComputeShiftLayers loc_3050A writes row H-scroll
// separately from sub_30744/sub_307D8's Plane-B nametable cells. Native keeps
// those cells in VDP-column order, then applies the hardware scroll here.
fn chunked_plane_b_cell_screen_xs_loc_3050a(column: usize, scroll_x_px: i32) -> [i32; 3] {
    let row_width_px = chunked_plane_b_row_width_px_loc_11b7a();
    let column_x = i32::try_from(column).unwrap_or(i32::MAX).saturating_mul(8);
    let screen_x = column_x.saturating_sub(scroll_x_px.rem_euclid(row_width_px));
    [screen_x - row_width_px, screen_x, screen_x + row_width_px]
}

fn chunked_plane_b_cell_at_sample_x_loc_3050a(
    sample_x: i32,
    scroll_x_px: i32,
) -> Option<(usize, i32)> {
    let row_width_px = chunked_plane_b_row_width_px_loc_11b7a();
    let scroll_x_mod = scroll_x_px.rem_euclid(row_width_px);
    let plane_x = sample_x
        .saturating_add(scroll_x_mod)
        .rem_euclid(row_width_px);
    let column = usize::try_from(plane_x.div_euclid(8)).ok()?;
    let mut screen_x = plane_x.saturating_sub(plane_x.rem_euclid(8)) - scroll_x_mod;
    while sample_x < screen_x {
        screen_x -= row_width_px;
    }
    while sample_x >= screen_x + 8 {
        screen_x += row_width_px;
    }
    Some((column, screen_x))
}

fn chunked_background_row_window_applies_loc_30744(
    level_visuals: &NativeLevelVisuals,
    y_tiles: usize,
) -> bool {
    let layer_speed = background_layer_speed_loc_3050a(level_visuals, y_tiles);
    matches!(layer_speed, 0x00 | 0x08)
        || matches!(layer_speed, 0x18..=0x88 if (layer_speed - 0x10).is_multiple_of(8))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChunkedPlaneBCellLoc307D8 {
    tile_ref: TileRef,
    background_row_index: usize,
    background_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkedBackgroundSourceLoc12848 {
    width: usize,
    height: usize,
    cells: Vec<Option<ChunkedPlaneBCellLoc307D8>>,
}

impl ChunkedBackgroundSourceLoc12848 {
    pub fn from_background_direct_loc_12830(
        level_visuals: &NativeLevelVisuals,
        background: &wildside_render::ChunkedBackgroundVisuals,
    ) -> Option<Self> {
        Self::from_background_with_offsets_loc_12824_12830(level_visuals, background, 0, 0)
    }

    pub fn from_background_alias_loc_12824(
        level_visuals: &NativeLevelVisuals,
        background: &wildside_render::ChunkedBackgroundVisuals,
    ) -> Option<Self> {
        let alias = level_visuals.level.background_layout_alias.as_ref()?;
        Self::from_background_with_offsets_loc_12824_12830(
            level_visuals,
            background,
            i32::from(alias.x_offset_tiles),
            i32::from(alias.y_offset_tiles),
        )
    }

    fn from_background_with_offsets_loc_12824_12830(
        level_visuals: &NativeLevelVisuals,
        background: &wildside_render::ChunkedBackgroundVisuals,
        x_offset_tiles: i32,
        y_offset_tiles: i32,
    ) -> Option<Self> {
        let (width, height) = chunked_background_dimensions_loc_12848(level_visuals)?;
        let palette_line =
            chunked_background_palette_line_loc_128ba(level_visuals.level.background_theme_id);
        let mut cells = Vec::with_capacity(width.checked_mul(height)?);
        for row in 0..height {
            for column in 0..width {
                // ASM Load_BackgroundChunk loc_12848 writes chunk bytes into
                // `$FFFF87B2`, whose cleared cells remain chunk id `0`.
                // sub_30744/sub_307D8 then map that byte through `unk_309B8`,
                // so sparse Plane-B rows still render `$2780`, not
                // transparency.
                cells.push(Some(ChunkedPlaneBCellLoc307D8 {
                    tile_ref: TileRef {
                        tile_index: 0,
                        palette_line,
                        priority: false,
                        flip_x: false,
                        flip_y: false,
                    },
                    background_row_index: row,
                    background_column: column,
                }));
            }
        }

        for placement in &background.placements {
            let Some(chunk) = background.chunks.get(usize::from(placement.chunk_id)) else {
                continue;
            };
            // ASM Load_BackgroundMappings loc_12824 subtracts the shared
            // layout offsets from the source placement before loc_12848
            // clips each copied byte against `Background_width/height`.
            let placement_x = i32::from(placement.x_tiles).saturating_sub(x_offset_tiles);
            let placement_y = i32::from(placement.y_tiles).saturating_sub(y_offset_tiles);
            let Ok(placement_x) = i16::try_from(placement_x) else {
                continue;
            };
            let Ok(placement_y) = i16::try_from(placement_y) else {
                continue;
            };
            let Ok(background_width) = u16::try_from(width) else {
                continue;
            };
            let Ok(background_height) = u16::try_from(height) else {
                continue;
            };
            let chunk_bytes = chunk
                .tiles
                .iter()
                .map(|tile| {
                    u8::try_from(tile.tile_index & 0x007F).unwrap_or(0)
                        | if tile.flip_x { 0x80 } else { 0 }
                })
                .collect::<Vec<_>>();
            let copy = wildside_core::background_chunk_row_loop_loc_12884(
                chunk.width_tiles,
                chunk.height_tiles,
                placement_x,
                placement_y,
                background_width,
                background_height,
                &chunk_bytes,
            );
            for copied_tile in copy.copied_tiles {
                let Some(background_tile) = chunk.tiles.get(copied_tile.source_index) else {
                    continue;
                };
                // ASM Load_BackgroundChunk loc_12848 copies chunk bytes into
                // `$FFFF87B2` after clipping against `Background_width` and
                // `Background_height`. Bit 7 of the byte becomes the VDP
                // horizontal-flip bit when sub_30744/sub_307D8 add `$2780`.
                cells[copied_tile.target_row * width + copied_tile.target_column] =
                    Some(ChunkedPlaneBCellLoc307D8 {
                        tile_ref: TileRef {
                            tile_index: background_tile.tile_index & 0x007F,
                            palette_line,
                            priority: false,
                            flip_x: background_tile.flip_x,
                            flip_y: false,
                        },
                        background_row_index: copied_tile.target_row,
                        background_column: copied_tile.target_column,
                    });
            }
        }

        Some(Self {
            width,
            height,
            cells,
        })
    }

    // Used by the native player's cross-crate tests, so it cannot be
    // `#[cfg(test)]`-gated (that compiles only for this crate's own tests).
    pub fn tile_index_at(&self, row: usize, column: usize) -> Option<u16> {
        if row >= self.height || column >= self.width {
            return None;
        }
        self.cells[row * self.width + column].map(|cell| cell.tile_ref.tile_index)
    }

    fn palette_line_loc_307d8(&self) -> u8 {
        self.cells
            .iter()
            .flatten()
            .next()
            .map_or(1, |cell| cell.tile_ref.palette_line)
    }

    fn chunk_buffer_bytes_loc_12848(&self) -> Vec<u8> {
        self.cells
            .iter()
            .map(|cell| {
                cell.map_or(0, |cell| {
                    u8::try_from(cell.tile_ref.tile_index & 0x007F).unwrap_or(0)
                        | if cell.tile_ref.flip_x { 0x80 } else { 0 }
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChunkedPlaneBRowHistoryLoc30744 {
    cells: [Option<ChunkedPlaneBCellLoc307D8>; PLANE_B_ROW_CELLS_LOC_11B7A],
}

impl ChunkedPlaneBRowHistoryLoc30744 {
    fn from_old_plane_b_row_loc_307d8(row: usize) -> Self {
        Self {
            cells: std::array::from_fn(|column| {
                // ASM sub_307D8 only DMA-writes the newly exposed `$2780` span;
                // cells outside that span keep the old Plane-B `$0780` tile.
                Some(ChunkedPlaneBCellLoc307D8 {
                    tile_ref: TileRef {
                        tile_index: 0,
                        palette_line: 0,
                        priority: false,
                        flip_x: false,
                        flip_y: false,
                    },
                    background_row_index: row,
                    background_column: column,
                })
            }),
        }
    }

    fn apply_row_upload_loc_307d8(
        &mut self,
        source_buffer: &ChunkedBackgroundSourceLoc12848,
        row: usize,
        destination_column: usize,
        source_column: usize,
    ) {
        // ASM sub_307D8 sets `d4 = #$28` and DBF streams the newly exposed
        // Plane-B row. The previous sub_30744 pass writes the single
        // horizontally exposed column first, so the displayed frame contains
        // 42 fresh cells in the issue #289 oracle row dump.
        for offset in 0..CHUNKED_BACKGROUND_VISIBLE_ROW_CELLS_LOC_307D8_30744 {
            let source = (source_column + offset) % source_buffer.width;
            let destination = (destination_column + offset) % PLANE_B_ROW_CELLS_LOC_11B7A;
            self.cells[destination] = source_buffer.cells[row * source_buffer.width + source];
        }
    }

    fn apply_preload_segment_loc_307d8(
        &mut self,
        row: usize,
        vram_word_address: u16,
        background_chunk_ids: &[u8],
        palette_line: u8,
    ) {
        let plane_row = usize::from((vram_word_address & 0x0F80) >> 7);
        if plane_row != (row & 0x1F) {
            return;
        }
        let start_column = usize::from((vram_word_address & 0x007E) >> 1);
        for (offset, chunk_id) in background_chunk_ids.iter().copied().enumerate() {
            let destination = (start_column + offset) % PLANE_B_ROW_CELLS_LOC_11B7A;
            self.cells[destination] = Some(ChunkedPlaneBCellLoc307D8 {
                tile_ref: TileRef {
                    tile_index: u16::from(chunk_id & 0x7F),
                    palette_line,
                    priority: false,
                    flip_x: chunk_id & 0x80 != 0,
                    flip_y: false,
                },
                background_row_index: row,
                background_column: destination,
            });
        }
    }

    fn apply_changed_column_loc_30744(
        &mut self,
        source_buffer: &ChunkedBackgroundSourceLoc12848,
        previous_scroll_word: u16,
        current_scroll_word: u16,
        row: usize,
    ) {
        let previous_tile_column = previous_scroll_word >> 3;
        let current_tile_column = current_scroll_word >> 3;
        if previous_tile_column == current_tile_column {
            return;
        }

        // ASM loc_30452/sub_304B4 stores one changed source column into
        // `Horiz_Scroll_Data`, then sub_30744/loc_307B2 streams that single
        // mapping word into Plane B before the conditional sub_307D8 row span.
        let source_column_seed = if previous_tile_column > current_tile_column {
            i32::from(current_tile_column)
        } else {
            i32::from(current_tile_column) + 0x28
        };
        let source =
            source_column_seed.rem_euclid(i32::try_from(source_buffer.width).unwrap_or(1)) as usize;
        let destination = source_column_seed
            .rem_euclid(i32::try_from(PLANE_B_ROW_CELLS_LOC_11B7A).unwrap_or(0x40))
            as usize;
        self.cells[destination] = source_buffer.cells[row * source_buffer.width + source];
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChunkedPlaneBRowsLoc307D8 {
    source: ChunkedBackgroundSourceLoc12848,
    alias_source_loc_12824: Option<ChunkedBackgroundSourceLoc12848>,
}

impl ChunkedPlaneBRowsLoc307D8 {
    fn from_background(
        level_visuals: &NativeLevelVisuals,
        background: &wildside_render::ChunkedBackgroundVisuals,
    ) -> Option<Self> {
        let source = ChunkedBackgroundSourceLoc12848::from_background_direct_loc_12830(
            level_visuals,
            background,
        )?;
        let alias_source_loc_12824 =
            ChunkedBackgroundSourceLoc12848::from_background_alias_loc_12824(
                level_visuals,
                background,
            );
        Some(Self {
            source,
            alias_source_loc_12824,
        })
    }

    fn row_for_trace(
        &self,
        trace: &NativeTraceFrame,
        level_visuals: &NativeLevelVisuals,
        camera_x_px: i32,
        row: usize,
    ) -> Option<[Option<ChunkedPlaneBCellLoc307D8>; PLANE_B_ROW_CELLS_LOC_11B7A]> {
        if row >= self.source.height {
            return None;
        }
        let layer_speed = background_layer_speed_loc_3050a(level_visuals, row);
        let scroll_x_px =
            chunked_background_upload_scroll_x_px_loc_2ffe8(trace, camera_x_px, layer_speed);
        let destination_column = scroll_x_px
            .div_euclid(8)
            .rem_euclid(i32::try_from(PLANE_B_ROW_CELLS_LOC_11B7A).unwrap_or(0x40))
            as usize;
        let source_column_bias = if level_visuals.level.background_scroll.is_some() {
            1
        } else {
            0
        };
        let source_column = (scroll_x_px.div_euclid(8) + source_column_bias)
            .rem_euclid(i32::try_from(self.source.width).ok()?)
            as usize;
        let mut history = ChunkedPlaneBRowHistoryLoc30744::from_old_plane_b_row_loc_307d8(row);
        if trace.level_skip_cheat_enabled != 0 {
            if !self.apply_preload_rows_loc_2ffe8(
                &mut history,
                trace,
                level_visuals,
                camera_x_px,
                row,
            ) {
                history.apply_row_upload_loc_307d8(
                    &self.source,
                    row,
                    destination_column,
                    source_column,
                );
            }
        } else {
            history.apply_row_upload_loc_307d8(
                &self.source,
                row,
                destination_column,
                source_column,
            );
        }

        let changed_column_source = self.source_buffer_for_trace_loc_12824(trace);
        let changed_column_row = source_row_for_plane_b_changed_column_loc_30744(
            trace,
            row,
            changed_column_source.height,
        )
        .unwrap_or(row);
        self.apply_elapsed_column_writes_loc_30744(
            &mut history,
            trace,
            camera_x_px,
            layer_speed,
            changed_column_row,
            changed_column_source,
        );

        Some(history.cells)
    }

    fn row_debug_for_trace_loc_30744(
        &self,
        trace: &NativeTraceFrame,
        level_visuals: &NativeLevelVisuals,
        camera_x_px: i32,
        row: usize,
    ) -> Option<String> {
        if row >= self.source.height {
            return None;
        }
        let layer_speed = background_layer_speed_loc_3050a(level_visuals, row);
        let current_time_frames = background_time_frames_for_trace_loc_3050a(trace);
        let preload_time_frames = current_time_frames.saturating_sub(trace.frame_index);
        let first_time_frames = if trace.level_skip_cheat_enabled != 0 {
            0
        } else {
            preload_time_frames
        };
        let upload_scroll_x_px =
            chunked_background_upload_scroll_x_px_loc_2ffe8(trace, camera_x_px, layer_speed);
        let upload_destination = upload_scroll_x_px
            .div_euclid(8)
            .rem_euclid(i32::try_from(PLANE_B_ROW_CELLS_LOC_11B7A).unwrap_or(0x40))
            as usize;
        let source_column_bias = if level_visuals.level.background_scroll.is_some() {
            1
        } else {
            0
        };
        let upload_source = (upload_scroll_x_px.div_euclid(8) + source_column_bias)
            .rem_euclid(i32::try_from(self.source.width).ok()?)
            as usize;
        // ASM sub_307D8 (kid.asm:34646): the row stream does not consume
        // BackgroundScroll_ComputeShiftLayers loc_3050A display pixels. It
        // reads the packed Horiz_Scroll_Data word written by loc_30452 and
        // shifts it by three before building the VDP/source row window.
        let upload_data_word =
            background_scroll_data_word_loc_30452(preload_time_frames, camera_x_px, layer_speed);
        let upload_data_column = usize::from((upload_data_word >> 3) & 0x003F);
        let upload_data_source =
            (usize::from(upload_data_word >> 3) + source_column_bias as usize) % self.source.width;
        let changed_column_source = self.source_buffer_for_trace_loc_12824(trace);
        let changed_column_row = source_row_for_plane_b_changed_column_loc_30744(
            trace,
            row,
            changed_column_source.height,
        )
        .unwrap_or(row);
        let changed_source_kind =
            if trace.level_skip_cheat_enabled != 0 && self.alias_source_loc_12824.is_some() {
                "alias"
            } else {
                "direct"
            };
        let mut changed_writes = Vec::new();
        if preload_time_frames != current_time_frames {
            let mut previous_scroll =
                background_scroll_data_word_loc_30452(first_time_frames, camera_x_px, layer_speed);
            for time_frames in first_time_frames.saturating_add(1)..=current_time_frames {
                let current_scroll =
                    background_scroll_data_word_loc_30452(time_frames, camera_x_px, layer_speed);
                if let Some(write) = changed_column_write_debug_loc_30744(
                    changed_column_source,
                    previous_scroll,
                    current_scroll,
                    changed_column_row,
                    time_frames,
                ) {
                    changed_writes.push(write);
                }
                previous_scroll = current_scroll;
            }
        }
        let changed_writes = if changed_writes.is_empty() {
            "none".to_owned()
        } else {
            changed_writes.join("|")
        };
        let words = self.row_for_trace(trace, level_visuals, camera_x_px, row)?;
        let words = words
            .into_iter()
            .map(|cell| {
                cell.map_or_else(
                    || "....".to_string(),
                    |cell| format!("{:04X}", chunked_plane_b_cell_vdp_word_loc_307d8(cell)),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        Some(format!(
            "B{row:02} sel={layer_speed:02X} t={current_time_frames} preload={preload_time_frames} first={first_time_frames} upload_px={upload_scroll_x_px} upload=d{upload_destination:02}<-s{upload_source:02} upload_data=d{upload_data_column:02}<-s{upload_data_source:02}w{upload_data_word:04X} changed_src={changed_source_kind} writes={changed_writes} words={words}"
        ))
    }

    fn source_buffer_for_trace_loc_12824(
        &self,
        trace: &NativeTraceFrame,
    ) -> &ChunkedBackgroundSourceLoc12848 {
        if trace.level_skip_cheat_enabled != 0
            && let Some(source) = self.alias_source_loc_12824.as_ref()
        {
            return source;
        }
        &self.source
    }

    fn apply_preload_rows_loc_2ffe8(
        &self,
        history: &mut ChunkedPlaneBRowHistoryLoc30744,
        trace: &NativeTraceFrame,
        level_visuals: &NativeLevelVisuals,
        camera_x_px: i32,
        row: usize,
    ) -> bool {
        let source_buffer = self.source_buffer_for_trace_loc_12824(trace);
        let Some(background_scroll) = level_visuals.level.background_scroll.as_ref() else {
            return false;
        };
        // ASM Load_InGame loc_11F48 seeds Camera_X from PlayerStart before
        // later load-stage camera adjustment. In the #289 oracle preload rows,
        // Time_Frames-driven `$00/$08` slots retain that grid phase, while
        // camera-parallax slots match the settled trace Camera_X.
        let load_grid_camera_x_px =
            load_ingame_camera_grid_x_px_loc_11f48(trace.player_start_x, camera_x_px);
        let load_grid_camera_y_px = load_ingame_camera_grid_y_px_loc_11f48(trace.player_start_y);
        let preload_camera_y_px = i32::try_from(trace.camera_y_pos >> 16).unwrap_or(0);
        let level_height_pixels =
            i32::from(level_visuals.level.foreground_layout.height_cells).saturating_mul(16);
        let Ok(level_height_pixels) = i16::try_from(level_height_pixels) else {
            return false;
        };
        let Ok(background_width) = u16::try_from(source_buffer.width) else {
            return false;
        };

        let Ok(background_height) = u16::try_from(source_buffer.height) else {
            return false;
        };
        let row_pointers =
            wildside_core::background_row_pointers_loc_11B7A(background_height, background_width);
        let Some(&first_row_pointer) = row_pointers.first() else {
            return false;
        };
        let row_offsets = row_pointers
            .iter()
            .map(|row_pointer| row_pointer.wrapping_sub(first_row_pointer))
            .collect::<Vec<_>>();

        let mut bgscroll_profile = vec![
            0xFF;
            source_buffer
                .height
                .max(background_scroll.layer_speeds.len())
        ];
        for (index, speed) in background_scroll.layer_speeds.iter().copied().enumerate() {
            bgscroll_profile[index] = speed;
        }

        let palette_line = source_buffer.palette_line_loc_307d8();
        if let Some(load_grid_camera_y_px) = load_grid_camera_y_px
            && load_grid_camera_y_px != preload_camera_y_px
        {
            // ASM Load_InGame loc_11F48 rounds PlayerStart_X/Y to screen-grid
            // camera values before the first loc_2FFE8 Plane-B preload. Later
            // level-skip settle camera motion leaves cells outside subsequent
            // sub_307D8 spans intact in the Plane-B nametable.
            if !self.apply_preload_rows_for_camera_loc_2ffe8(
                history,
                load_grid_camera_y_px,
                load_grid_camera_x_px,
                load_grid_camera_x_px,
                level_height_pixels,
                background_width,
                &row_offsets,
                &bgscroll_profile,
                source_buffer,
                palette_line,
                row,
            ) {
                return false;
            }
        }

        self.apply_preload_rows_for_camera_loc_2ffe8(
            history,
            preload_camera_y_px,
            load_grid_camera_x_px,
            camera_x_px,
            level_height_pixels,
            background_width,
            &row_offsets,
            &bgscroll_profile,
            source_buffer,
            palette_line,
            row,
        )
    }

    #[allow(
        clippy::unused_self,
        reason = "semantically a method on the plane-B row renderer"
    )]
    fn apply_preload_rows_for_camera_loc_2ffe8(
        &self,
        history: &mut ChunkedPlaneBRowHistoryLoc30744,
        camera_y_px: i32,
        timed_camera_x_px: i32,
        parallax_camera_x_px: i32,
        level_height_pixels: i16,
        background_width: u16,
        row_offsets: &[u16],
        bgscroll_profile: &[u8],
        source_buffer: &ChunkedBackgroundSourceLoc12848,
        palette_line: u8,
        row: usize,
    ) -> bool {
        let Ok(camera_y_px) = i16::try_from(camera_y_px) else {
            return false;
        };
        let max_scroll_slot = max_horiz_scroll_slot_loc_2ffe8(bgscroll_profile);
        let mut horiz_scroll_entries = Vec::with_capacity(max_scroll_slot + 1);
        for slot in 0..=max_scroll_slot {
            let layer_speed = u8::try_from(slot.saturating_mul(8)).unwrap_or(0x88);
            let preload_camera_x_px = match layer_speed {
                0x00 | 0x08 => timed_camera_x_px,
                _ => parallax_camera_x_px,
            };
            let scroll_word =
                background_scroll_data_word_loc_30452(0, preload_camera_x_px, layer_speed);
            let source_column = usize::from(scroll_word >> 3) % source_buffer.width;
            let vram_word_offset = ((scroll_word >> 3) & 0x003F) << 1;
            horiz_scroll_entries.push(wildside_data::BackgroundScrollStateSub30744 {
                background_buffer_column: Some(u16::try_from(source_column).unwrap_or(0)),
                vram_word_offset,
            });
        }

        let Ok(uploads) = wildside_data::background_preload_row_uploads_loc_2FFE8(
            camera_y_px,
            level_height_pixels,
            background_width,
            row_offsets,
            bgscroll_profile,
            &horiz_scroll_entries,
            &source_buffer.chunk_buffer_bytes_loc_12848(),
        ) else {
            return false;
        };
        for upload in uploads {
            history.apply_preload_segment_loc_307d8(
                row,
                upload.vram_word_address,
                &upload.background_chunk_ids,
                palette_line,
            );
        }
        true
    }

    #[allow(
        clippy::unused_self,
        reason = "semantically a method on the plane-B row renderer"
    )]
    fn apply_elapsed_column_writes_loc_30744(
        &self,
        history: &mut ChunkedPlaneBRowHistoryLoc30744,
        trace: &NativeTraceFrame,
        camera_x_px: i32,
        layer_speed: u8,
        row: usize,
        source_buffer: &ChunkedBackgroundSourceLoc12848,
    ) {
        let current_time_frames = background_time_frames_for_trace_loc_3050a(trace);
        let preload_time_frames = current_time_frames.saturating_sub(trace.frame_index);
        if preload_time_frames == current_time_frames {
            return;
        }

        let first_time_frames = if trace.level_skip_cheat_enabled != 0 {
            0
        } else {
            preload_time_frames
        };
        let mut previous_scroll =
            background_scroll_data_word_loc_30452(first_time_frames, camera_x_px, layer_speed);
        for time_frames in first_time_frames.saturating_add(1)..=current_time_frames {
            let current_scroll =
                background_scroll_data_word_loc_30452(time_frames, camera_x_px, layer_speed);
            history.apply_changed_column_loc_30744(
                source_buffer,
                previous_scroll,
                current_scroll,
                row,
            );
            previous_scroll = current_scroll;
        }
    }
}

fn load_ingame_camera_grid_x_px_loc_11f48(player_start_x: Option<u16>, camera_x_px: i32) -> i32 {
    // ASM Load_InGame loc_11F48 rounds PlayerStart_X to the 320px screen grid
    // before the first loc_2FFE8 Plane-B preload. If no player start was
    // captured, keep the trace camera used by ordinary live rendering.
    player_start_x.map_or(camera_x_px, |start_x| i32::from(start_x / 0x0140) * 0x0140)
}

fn load_ingame_camera_grid_y_px_loc_11f48(player_start_y: Option<u16>) -> Option<i32> {
    // ASM Load_InGame loc_11F48 uses the 224px vertical screen grid for the
    // same preload seed.
    player_start_y.map(|start_y| i32::from(start_y / 0x00E0) * 0x00E0)
}

fn max_horiz_scroll_slot_loc_2ffe8(bgscroll_profile: &[u8]) -> usize {
    // ASM sub_307D8 treats `$FF` bgscroll bytes as row terminators/no-op rows;
    // only real Horiz_Scroll_Data entry offsets determine how many preload
    // slots loc_2FFE8 must build.
    bgscroll_profile
        .iter()
        .copied()
        .filter(|speed| *speed != 0xFF)
        .map(|speed| usize::from(speed / 8))
        .max()
        .unwrap_or(0)
}

fn source_row_for_plane_b_changed_column_loc_30744(
    trace: &NativeTraceFrame,
    plane_row: usize,
    source_height: usize,
) -> Option<usize> {
    let camera_y_px = i32::try_from(trace.camera_y_pos >> 16).ok()?;
    let camera_row = camera_y_px >> 5;
    if camera_row < 0 {
        return None;
    }
    let camera_row = usize::try_from(camera_row).ok()?;
    // ASM sub_30744 (kid.asm:34563): live column writes walk 29 rows from
    // `Camera_Y_pos>>5`, while the VDP row address wraps with `& $1F`.
    (0..=0x1C)
        .map(|offset| camera_row + offset)
        .find(|source_row| source_row < &source_height && (source_row & 0x1F) == (plane_row & 0x1F))
}

fn changed_column_write_debug_loc_30744(
    source_buffer: &ChunkedBackgroundSourceLoc12848,
    previous_scroll_word: u16,
    current_scroll_word: u16,
    row: usize,
    time_frames: usize,
) -> Option<String> {
    let previous_tile_column = previous_scroll_word >> 3;
    let current_tile_column = current_scroll_word >> 3;
    if previous_tile_column == current_tile_column {
        return None;
    }

    let source_column_seed = if previous_tile_column > current_tile_column {
        i32::from(current_tile_column)
    } else {
        i32::from(current_tile_column) + 0x28
    };
    let source =
        source_column_seed.rem_euclid(i32::try_from(source_buffer.width).unwrap_or(1)) as usize;
    let destination = source_column_seed
        .rem_euclid(i32::try_from(PLANE_B_ROW_CELLS_LOC_11B7A).unwrap_or(0x40))
        as usize;
    let word = source_buffer.cells[row * source_buffer.width + source]
        .map(chunked_plane_b_cell_vdp_word_loc_307d8)
        .map_or_else(|| "....".to_owned(), |word| format!("{word:04X}"));
    Some(format!(
        "t{time_frames}:c{previous_tile_column:04X}->{current_tile_column:04X}:d{destination:02}<-s{source:02}:{word}"
    ))
}

// Cross-crate test helper for the native player's background-scroll tests, so
// it cannot be `#[cfg(test)]`-gated (that compiles only for this crate's tests).
pub fn chunked_plane_b_row_words_for_trace_loc_30744(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    camera_x_px: i32,
    row: usize,
) -> Option<Vec<Option<(u16, u8)>>> {
    let wildside_render::BackgroundVisuals::Chunked(background) = &level_visuals.background else {
        return None;
    };
    let plane_b = ChunkedPlaneBRowsLoc307D8::from_background(level_visuals, background)?;
    plane_b
        .row_for_trace(trace, level_visuals, camera_x_px, row)
        .map(|cells| {
            cells
                .into_iter()
                .map(|cell| cell.map(|cell| (cell.tile_ref.tile_index, cell.tile_ref.palette_line)))
                .collect()
        })
}

// Cross-crate test helper (see `chunked_plane_b_row_words_for_trace_loc_30744`).
pub fn chunked_plane_b_row_vdp_words_for_trace_loc_30744(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    camera_x_px: i32,
    row: usize,
) -> Option<Vec<Option<u16>>> {
    let wildside_render::BackgroundVisuals::Chunked(background) = &level_visuals.background else {
        return None;
    };
    let plane_b = ChunkedPlaneBRowsLoc307D8::from_background(level_visuals, background)?;
    plane_b
        .row_for_trace(trace, level_visuals, camera_x_px, row)
        .map(|cells| {
            cells
                .into_iter()
                .map(|cell| cell.map(chunked_plane_b_cell_vdp_word_loc_307d8))
                .collect()
        })
}

pub fn chunked_plane_b_row_debug_for_trace_loc_30744(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    camera_x_px: i32,
    row: usize,
) -> Option<String> {
    let wildside_render::BackgroundVisuals::Chunked(background) = &level_visuals.background else {
        return None;
    };
    let plane_b = ChunkedPlaneBRowsLoc307D8::from_background(level_visuals, background)?;
    plane_b.row_debug_for_trace_loc_30744(trace, level_visuals, camera_x_px, row)
}

fn chunked_plane_b_cell_vdp_word_loc_307d8(cell: ChunkedPlaneBCellLoc307D8) -> u16 {
    // ASM sub_307D8 adds `$2780` to the `$FFFF87B2` chunk byte before
    // streaming the word into Plane B via `unk_309B8`: `$2780..$27FF`,
    // then `$2F80..$2FFF` for high-bit chunk bytes. Bit 7 therefore becomes
    // the VDP H-flip bit, not tile-index bit 7.
    let palette_bits = u16::from(cell.tile_ref.palette_line) << 13;
    let hflip_bit = if cell.tile_ref.flip_x { 0x0800 } else { 0 };
    palette_bits | hflip_bit | 0x0780 | (cell.tile_ref.tile_index & 0x007F)
}

pub fn background_scroll_data_word_loc_30452(
    time_frames: usize,
    camera_x_px: i32,
    layer_speed: u8,
) -> u16 {
    let camera_column = (camera_x_px >> 3) as i16;
    let scroll_word = match layer_speed {
        0x00 => (time_frames as i32).wrapping_add(i32::from(camera_column)),
        0x08 => ((time_frames as i32) >> 1).wrapping_add(i32::from(camera_column)),
        0x10 => 0,
        0x18..=0x88 if (layer_speed - 0x10).is_multiple_of(8) => {
            let step = i32::from((layer_speed - 0x10) / 8);
            camera_x_px.saturating_mul(step) / 16
        }
        _ => camera_x_px,
    };
    scroll_word as u16
}

fn chunked_background_upload_scroll_x_px_loc_2ffe8(
    trace: &NativeTraceFrame,
    camera_x_px: i32,
    layer_speed: u8,
) -> i32 {
    let time_frames = background_time_frames_for_trace_loc_3050a(trace);
    let preload_time_frames = time_frames.saturating_sub(trace.frame_index);
    match layer_speed {
        // ASM loc_2FFE8 preloads chunked Plane-B rows before the visible
        // gameplay recording window. `$00`/`$08` rows are Time_Frames-driven,
        // so their uploaded VDP cell arrangement can be older than the
        // current loc_3050A display H-scroll.
        0x00 if preload_time_frames != time_frames => {
            background_scroll_x_px_for_time_loc_3050a(preload_time_frames, camera_x_px, layer_speed)
        }
        0x08 if preload_time_frames != time_frames => {
            background_scroll_x_px_for_time_loc_3050a(preload_time_frames, camera_x_px, layer_speed)
        }
        _ => background_scroll_x_px(trace, camera_x_px, layer_speed),
    }
}

fn chunked_background_dimensions_loc_12848(
    level_visuals: &NativeLevelVisuals,
) -> Option<(usize, usize)> {
    let level_width_tiles = level_visuals
        .level
        .foreground_layout
        .width_cells
        .checked_mul(2)?;
    let level_height_tiles = level_visuals
        .level
        .foreground_layout
        .height_cells
        .checked_mul(2)?;
    let (width, height) =
        wildside_core::background_dimensions_loc_11B40(level_width_tiles, level_height_tiles);
    let width = usize::from(width);
    let height = usize::from(height);
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

fn sample_layered_background_tile_pixel(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    tile_sheet: &TileSheet,
    palette: &Palette,
    effect_start_row_selector: Option<u8>,
    camera_y_px: i32,
    screen_x: i32,
    screen_y: i32,
    background_row_index: usize,
    background_row_selector: u8,
    background_column: usize,
    tile: TileRef,
    sample_x: i32,
    sample_y: i32,
) -> Option<BackgroundPixelSample> {
    let Some(effect_start_row_selector) = effect_start_row_selector else {
        return sample_background_tile_pixel(
            tile_sheet,
            palette,
            tile,
            screen_x,
            screen_y,
            sample_x,
            sample_y,
            None,
            None,
            screen_x,
            screen_y,
            Some(background_row_index),
            Some(background_row_selector),
            Some(background_column),
        );
    };
    if matches!(
        wildside_core::background_scroll_theme_dispatch_loc_30598(u16::from(
            level_visuals.level.background_theme_id
        )),
        wildside_core::BackgroundScrollThemeDispatchLoc30598::Return
    ) {
        return sample_background_tile_pixel(
            tile_sheet,
            palette,
            tile,
            screen_x,
            screen_y,
            sample_x,
            sample_y,
            None,
            None,
            screen_x,
            screen_y,
            Some(background_row_index),
            Some(background_row_selector),
            Some(background_column),
        );
    }

    for row in 0..8 {
        let offset = background_ripple_offset_for_screen_y_sub_30598(
            trace,
            level_visuals.level.background_theme_id,
            screen_y + row,
            effect_start_row_selector,
            camera_y_px,
        )
        .unwrap_or(0);
        if let Some(sample) = sample_background_tile_pixel(
            tile_sheet,
            palette,
            tile,
            screen_x + offset,
            screen_y + row,
            sample_x,
            sample_y,
            Some(row as u8),
            Some(offset),
            screen_x,
            screen_y,
            Some(background_row_index),
            Some(background_row_selector),
            Some(background_column),
        ) {
            return Some(sample);
        }
    }
    None
}

fn sample_background_tile_pixel(
    tile_sheet: &TileSheet,
    palette: &Palette,
    tile_ref: TileRef,
    screen_x: i32,
    screen_y: i32,
    sample_x: i32,
    sample_y: i32,
    row_override: Option<u8>,
    row_offset: Option<i32>,
    tile_origin_x: i32,
    tile_origin_y: i32,
    background_row_index: Option<usize>,
    background_row_selector: Option<u8>,
    background_column: Option<usize>,
) -> Option<BackgroundPixelSample> {
    if sample_x < screen_x || sample_x >= screen_x + 8 {
        return None;
    }
    let dest_x = sample_x - screen_x;
    let dest_y = match row_override {
        Some(row) if sample_y == screen_y => i32::from(row),
        Some(_) => return None,
        None if sample_y >= screen_y && sample_y < screen_y + 8 => sample_y - screen_y,
        None => return None,
    };
    let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
    let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
    let tile = tile_sheet.tiles.get(usize::from(tile_ref.tile_index))?;
    let palette_index = tile.pixels[(source_y * 8 + source_x) as usize];
    let row_start = (source_y * 8) as usize;
    let source_row_pixels = tile.pixels[row_start..row_start + 8]
        .try_into()
        .expect("tile rows are always 8 pixels wide");
    let rgb = resolve_palette_colour(palette, tile_ref.palette_line, palette_index)?;
    Some(BackgroundPixelSample {
        screen_x,
        screen_y: if row_override.is_some() {
            sample_y
        } else {
            screen_y
        },
        tile_origin_x,
        tile_origin_y,
        background_row_index,
        background_row_selector,
        background_column,
        tile_index: tile_ref.tile_index,
        palette_line: tile_ref.palette_line,
        palette_index,
        source_x: source_x as u8,
        source_y: source_y as u8,
        source_row_pixels,
        row_offset,
        rgb,
    })
}

fn push_layered_background_tile(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    effect_start_row_selector: Option<u8>,
    background_theme_id: u8,
    camera_y_px: i32,
    screen_x: i32,
    screen_y: i32,
    tile_sheet: Arc<TileSheet>,
    palette: Arc<Palette>,
    tile: TileRef,
) {
    let Some(effect_start_row_selector) = effect_start_row_selector else {
        commands.push(DrawCommand::IndexedTile {
            x: screen_x,
            y: screen_y,
            tile_sheet,
            tile,
            palette,
        });
        return;
    };
    if matches!(
        wildside_core::background_scroll_theme_dispatch_loc_30598(u16::from(background_theme_id)),
        wildside_core::BackgroundScrollThemeDispatchLoc30598::Return
    ) {
        commands.push(DrawCommand::IndexedTile {
            x: screen_x,
            y: screen_y,
            tile_sheet,
            tile,
            palette,
        });
        return;
    }

    let row_offsets = (0..8)
        .map(|row| {
            background_ripple_offset_for_screen_y_sub_30598(
                trace,
                background_theme_id,
                screen_y + row,
                effect_start_row_selector,
                camera_y_px,
            )
        })
        .collect::<Vec<_>>();
    if row_offsets.iter().all(Option::is_none) {
        commands.push(DrawCommand::IndexedTile {
            x: screen_x,
            y: screen_y,
            tile_sheet,
            tile,
            palette,
        });
        return;
    }

    for (row, offset) in row_offsets.into_iter().enumerate() {
        commands.push(DrawCommand::IndexedTileRow {
            x: screen_x + offset.unwrap_or(0),
            y: screen_y + row as i32,
            tile_sheet: tile_sheet.clone(),
            tile,
            palette: palette.clone(),
            row: row as u8,
        });
    }
}

pub fn foreground_palette_for_scene(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> Palette {
    let mut palette = level_visuals.palettes.foreground.clone();
    let delay = match level_visuals.level.foreground_theme_id {
        7 => 5usize,
        8 => 8usize,
        _ => return palette,
    };
    if palette.colours.len() < 10 {
        return palette;
    }

    if let Some(words) = trace.lava_water_palette_words_loc_1223c
        && words.iter().any(|word| *word != 0)
    {
        // ASM RotatePalette_Lava_Water loc_1223C writes the live colours to
        // Palette_Buffer+$E/$10/$12/$14. Use those typed trace words when the
        // core has seeded them; until loc_11DB0/loc_11E16 live palette loading
        // is fully wired, zero-filled buffers fall back to the extracted asset
        // palette path below.
        for (colour, word) in palette.colours[6..10].iter_mut().zip(words) {
            *colour = wildside_render::MdColour::from_word(word);
        }
        return palette;
    }

    // ASM RotatePalette_Lava_Water loc_1223A/loc_1223C: Mountain levels
    // hibernate 5 frames (Cave 8), then rotate CRAM words
    // Palette_Buffer+$E/$10/$12/$14. Native foreground palettes omit CRAM
    // word 0, so those ROM words map to visible colour indices 6..=9. The
    // extracted level palette is two ROM rotations behind the oracle
    // settled-playtest frame 0 for Highwater's direct level-skip lineage, so
    // pre-advance by two rotations before applying the live frame cadence.
    for _ in 0..((trace.frame_index / delay + 2) % 4) {
        let saved = palette.colours[9];
        palette.colours[9] = palette.colours[8];
        palette.colours[8] = palette.colours[7];
        palette.colours[7] = palette.colours[6];
        palette.colours[6] = saved;
    }

    palette
}

fn lava_geyser_palette_for_scene_loc_14b2a(
    trace: &NativeTraceFrame,
    visuals: &wildside_render::LavaVisuals,
) -> Palette {
    let mut palette = visuals.palette.clone();
    if let Some(words) = trace.lava_geyser_palette_words_loc_14b2a
        && words.iter().any(|word| *word != 0)
    {
        // ASM Init_SpecialEffect_Lava loc_14C6A copies Pal_7B8AC into
        // Palette_Buffer+$20; Animation_Geyser loc_14B2A then rotates the
        // live `$28..$2E` entries before sub_14BD8/loc_14BE8 draws Lava.bin.
        for (colour, word) in palette.colours.iter_mut().take(8).zip(words) {
            *colour = wildside_render::MdColour::from_word(word);
        }
    }
    palette
}

fn background_palette_for_scene(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> Palette {
    let mut palette = level_visuals.palettes.background.clone();
    if matches!(level_visuals.level.background_theme_id, 7 | 3)
        && level_visuals.palettes.foreground.colours.len() >= PALETTE_LINE_COLOUR_COUNT - 1
        && palette.colours.len() >= 8
    {
        // ASM Load_BackgroundMappings_Layered loc_12910 decodes Mountain's
        // main planes with palette-line bits from base `$2780`, while the
        // clear-weather follow-up plane uses `$07E5` and therefore reads
        // foreground CRAM line 0. loc_11E16 copies background colours 1..7
        // into line-1 slots 9..15 for the main planes without replacing the
        // foreground line used by the follow-up art.
        //
        // #350: Hills (theme 3) shares this two-line layout. loc_1295E decodes
        // Hill planes 0/1 with base `$2780` (CRAM line 1) and the lower
        // foliage plane 2 with base `$07C0` (CRAM line 0), so the foliage plane
        // reads the foreground line exactly like Mountain's `$07E5` follow-up.
        // Without this branch the line-0 foliage indexed the background sky
        // colours and rendered blue/purple below the tree canopy.
        let original = palette.colours.clone();
        let mut combined = foreground_palette_for_scene(trace, level_visuals).colours;
        combined.truncate(PALETTE_LINE_COLOUR_COUNT - 1);
        combined.resize(
            PALETTE_LINE_COLOUR_COUNT * 2,
            wildside_render::MdColour::from_word(0),
        );
        combined[PALETTE_LINE_COLOUR_COUNT + 8..PALETTE_LINE_COLOUR_COUNT + 15]
            .copy_from_slice(&original[1..8]);
        palette.colours = combined;
        return palette;
    }
    if level_visuals.level.background_theme_id == 9 && palette.colours.len() >= 8 {
        // ASM Load_BackgroundMappings_Layered loc_128BA decodes Forest tile
        // words with base `$0780`, so attr palette 0 really reads from CRAM
        // line 0.  The background palette is only copied into line 1 slots
        // 8..15, then ASM sub_129AE swaps line-1 slots 8 and 10.
        let original = palette.colours.clone();
        let mut combined = foreground_palette_for_scene(trace, level_visuals).colours;
        if combined.len() < PALETTE_LINE_COLOUR_COUNT {
            return palette;
        }
        combined.truncate(PALETTE_LINE_COLOUR_COUNT);
        combined.extend(std::iter::repeat_n(
            wildside_render::MdColour::from_word(0),
            PALETTE_LINE_COLOUR_COUNT,
        ));
        combined[PALETTE_LINE_COLOUR_COUNT + 7] = original[2];
        combined[PALETTE_LINE_COLOUR_COUNT + 8] = original[1];
        combined[PALETTE_LINE_COLOUR_COUNT + 9] = wildside_render::MdColour::from_word(0);
        combined[PALETTE_LINE_COLOUR_COUNT + 10..PALETTE_LINE_COLOUR_COUNT + 15]
            .copy_from_slice(&original[3..8]);
        palette.colours = combined;
    }
    palette
}

fn push_sprite_frame(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
    palette: &Palette,
    screen_x: i32,
    screen_y: i32,
) {
    push_sprite_frame_with_priority(commands, frame, palette, screen_x, screen_y, None);
}

fn push_sprite_frame_with_priority(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
    palette: &Palette,
    screen_x: i32,
    screen_y: i32,
    priority_override: Option<bool>,
) {
    let facing = Direction::Right;
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, facing);
    let mut tiles = build_sprite_tiles(frame, facing, 0);
    if let Some(priority) = priority_override {
        for tile in &mut tiles {
            tile.tile.priority = priority;
        }
    }
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette.clone()),
        tiles,
    });
}

pub fn push_sprite_frame_top_left(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
    palette: &Palette,
    x: i32,
    y: i32,
) {
    let screen_x = x + i32::from(frame.origin_x);
    let screen_y = y + i32::from(frame.height_px) - i32::from(frame.origin_y);
    push_sprite_frame(commands, frame, palette, screen_x, screen_y);
}

fn push_diamond_power_sprite_loc_3f880(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    sprite: DiamondPowerSpriteSnapshot,
) -> bool {
    let size_code = sprite.size_word >> 8;
    let width_px = ((size_code & 0x03) + 1) * 8;
    let height_px = (((size_code >> 2) & 0x03) + 1) * 8;
    let vram_tile = sprite.pattern_word & 0x07FF;
    let Some(tile_base_index) = vram_tile.checked_sub(level_visuals.hud.diamond_base_tile_index)
    else {
        return false;
    };
    if usize::from(tile_base_index) >= level_visuals.hud.diamond_tiles.tiles.len() {
        return false;
    }
    let frame = wildside_render::SpriteFrame {
        origin_x: 0,
        origin_y: 0,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet: level_visuals.hud.diamond_tiles.clone(),
    };
    // ASM DiamondPower_Main_CircleOfDoom loc_3F880 writes VDP sprite
    // coordinates already biased by `$7F-Camera`; native drawing uses
    // top-left screen pixels, so strip the VDP sprite bias here.
    push_sprite_frame_top_left(
        commands,
        &frame,
        &level_visuals.palettes.foreground,
        i32::from(sprite.screen_x) - VDP_SPRITE_SCREEN_BIAS_LOC_13C0,
        i32::from(sprite.screen_y) - VDP_SPRITE_SCREEN_BIAS_LOC_13C0,
    );
    true
}

fn push_murder_wall_warning_sprite_sub_5e02(
    commands: &mut Vec<DrawCommand>,
    visuals: &wildside_render::MurderWallVisuals,
    sprite: wildside_core::MurderWallWarningSpriteSub5E02,
) -> bool {
    let tile_ref = TileRef::from_attr_word(sprite.tile_word);
    let Some(tile_base_index) = tile_ref
        .tile_index
        .checked_sub(visuals.vram_base_tile_index)
    else {
        return false;
    };
    let frame = wildside_render::SpriteFrame {
        origin_x: 0,
        origin_y: 0,
        width_px: 24,
        height_px: 24,
        tile_base_index,
        tile_sheet: visuals.tile_sheet.clone(),
    };
    let facing = if tile_ref.flip_x {
        Direction::Left
    } else {
        Direction::Right
    };
    let mut tiles = build_sprite_tiles(&frame, facing, tile_ref.palette_line);
    for tile in &mut tiles {
        tile.tile.priority = tile_ref.priority;
        tile.tile.flip_y = tile_ref.flip_y;
    }
    commands.push(DrawCommand::Sprite {
        x: i32::from(sprite.x) - 0x80,
        y: i32::from(sprite.y) - 0x80,
        tile_sheet: Arc::new(visuals.tile_sheet.clone()),
        palette: Arc::new(visuals.palette.clone()),
        tiles,
    });
    true
}

fn push_murder_wall_warning_fallback_sub_5e02(
    commands: &mut Vec<DrawCommand>,
    min_x: i32,
    max_x: i32,
) {
    let width = (max_x - min_x).max(8);
    // TODO(fidelity): decode/render exact `Murder_wall.bin` drill art from
    // ASM Murderwall loc_140D2. This fallback is intentionally conspicuous so
    // live playtests can see the active `sub_5E02` wall instead of a flat red
    // placeholder while the ArtComp/Saxman path is still unresolved.
    commands.push(DrawCommand::Rect {
        x: min_x,
        y: 0,
        width,
        height: FRAME_HEIGHT as i32,
        colour: MURDER_WALL_FALLBACK_BASE_COLOUR,
    });
    let mut y = -8;
    while y < FRAME_HEIGHT as i32 {
        commands.push(DrawCommand::Rect {
            x: min_x,
            y,
            width,
            height: 6,
            colour: MURDER_WALL_FALLBACK_SHADOW_COLOUR,
        });
        commands.push(DrawCommand::Rect {
            x: min_x + width / 3,
            y: y + 6,
            width: (width * 2 / 3).max(4),
            height: 4,
            colour: MURDER_WALL_FALLBACK_HIGHLIGHT_COLOUR,
        });
        commands.push(DrawCommand::Rect {
            x: min_x,
            y: y + 12,
            width: (width * 2 / 3).max(4),
            height: 4,
            colour: MURDER_WALL_FALLBACK_SHADOW_COLOUR,
        });
        y += 24;
    }
}

fn push_lava_geyser_chunk_loc_14be8(
    commands: &mut Vec<DrawCommand>,
    visuals: &wildside_render::LavaVisuals,
    palette: &Palette,
    chunk: wildside_core::LavaGeyserSpriteChunkLoc14BE8,
) -> bool {
    let tile_ref = TileRef::from_attr_word(chunk.tile_word);
    let Some(tile_base_index) = tile_ref
        .tile_index
        .checked_sub(visuals.vram_base_tile_index)
    else {
        return false;
    };
    let size_code = (chunk.size_word >> 8) & 0x0F;
    let width_px = ((size_code & 0x03) + 1) * 8;
    let height_px = (((size_code >> 2) & 0x03) + 1) * 8;
    let frame = wildside_render::SpriteFrame {
        origin_x: 0,
        origin_y: 0,
        width_px,
        height_px,
        tile_base_index,
        tile_sheet: visuals.tile_sheet.clone(),
    };
    let mut tiles = build_sprite_tiles(&frame, Direction::Right, tile_ref.palette_line);
    for tile in &mut tiles {
        tile.tile.priority = tile_ref.priority;
        tile.tile.flip_x = tile_ref.flip_x;
        tile.tile.flip_y = tile_ref.flip_y;
    }
    commands.push(DrawCommand::Sprite {
        x: i32::from(chunk.screen_x_px) - 0x80,
        y: i32::from(chunk.screen_y_with_bias) - 0x80,
        tile_sheet: Arc::new(visuals.tile_sheet.clone()),
        palette: Arc::new(palette.clone()),
        tiles,
    });
    true
}

// ASM Init_SpecialEffect_Storm loc_14D1E/loc_14D32 skips the first longword of
// `unk_9B65C`/`unk_9B6E0`, then uploads these 64 source tile words to Scroll B
// as a 64x32 plane while adding `$0380_0380` to each longword.
const STORM_PATTERN_RAIN_WORDS_LOC_14D32: [u16; 64] = [
    0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A, 0x000B,
    0x000C, 0x000D, 0x000E, 0x000F, 0x000A, 0x000B, 0x0007, 0x000C, 0x000D, 0x000E, 0x0007, 0x000F,
    0x0007, 0x0007, 0x0010, 0x0011, 0x0007, 0x0012, 0x0013, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0014, 0x0006, 0x0015, 0x0016, 0x0017, 0x0018, 0x0019, 0x0007, 0x0007, 0x001A, 0x001B,
    0x001C, 0x001D, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0007, 0x0002, 0x001E, 0x001F, 0x0020,
    0x0007, 0x0007, 0x0021, 0x0022,
];

const STORM_PATTERN_HAIL_WORDS_LOC_14D32: [u16; 64] = [
    0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0004, 0x0007, 0x0008, 0x0004, 0x0009,
    0x0004, 0x000A, 0x000B, 0x000C, 0x0004, 0x000D, 0x0004, 0x000E, 0x0004, 0x000F, 0x000A, 0x000B,
    0x000C, 0x0009, 0x000D, 0x0004, 0x000E, 0x000F, 0x0010, 0x0004, 0x0011, 0x0012, 0x0004, 0x0004,
    0x0004, 0x0013, 0x0001, 0x0004, 0x0004, 0x0004, 0x0001, 0x0004, 0x0004, 0x0001, 0x0001, 0x0001,
    0x0001, 0x1813, 0x0014, 0x0015, 0x0004, 0x0004, 0x0016, 0x0017, 0x0018, 0x0004, 0x0004, 0x0019,
    0x0004, 0x001A, 0x0004, 0x001B,
];

fn storm_pattern_words_loc_14d32(source_asset_label: &str) -> Option<&'static [u16; 64]> {
    match source_asset_label {
        "unk_9B65C" => Some(&STORM_PATTERN_RAIN_WORDS_LOC_14D32),
        "unk_9B6E0" => Some(&STORM_PATTERN_HAIL_WORDS_LOC_14D32),
        _ => None,
    }
}

fn push_storm_pattern_plane_loc_14d32(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> bool {
    let Some(upload) = &trace.storm_pattern_upload_loc_14d32 else {
        return false;
    };
    let Some(transfer) = trace.storm_scroll_transfer_loc_30110 else {
        return false;
    };
    let Some(visuals) = &level_visuals.weather else {
        return false;
    };
    let Some(source_words) = storm_pattern_words_loc_14d32(&upload.source_asset_label) else {
        return false;
    };
    if upload.vram_tile_addend != 0x0380_0380
        || upload.skipped_header_longwords != 1
        || upload.vdp_command != 0x4000_0003
        || upload.written_longword_count != 4 * 8 * 8 * 4
    {
        return false;
    }
    if transfer.plane_address_word != 0x8403 || transfer.h_scroll_word_pairs != 0x00E0 {
        return false;
    }

    // ASM Transfer_ScrollDataToVRAM loc_30110: the Scroll-B storm pattern uses
    // the second HScroll/VSRAM words, both sourced from `$FFFFFAD8`.
    let scroll_x_px = i32::from(transfer.second_h_scroll_word);
    let scroll_y_px = i32::from(transfer.second_vsram_word);
    let first_plane_col = scroll_x_px.div_euclid(8);
    let first_plane_row = scroll_y_px.div_euclid(8);
    let tile_addend = (upload.vram_tile_addend & 0xFFFF) as u16;
    let visible_cols = FRAME_WIDTH.div_ceil(8) as usize + 1;
    let visible_rows = FRAME_HEIGHT.div_ceil(8) as usize + 1;
    let tile_sheet = Arc::new(visuals.tile_sheet.clone());
    let palette = Arc::new(background_palette_for_scene(trace, level_visuals));

    for visible_row in 0..visible_rows {
        let plane_row = first_plane_row
            + i32::try_from(visible_row).expect("visible row fits i32 for storm plane");
        let pattern_row = plane_row.rem_euclid(8) as usize;
        let y = plane_row * 8 - scroll_y_px;
        for visible_col in 0..visible_cols {
            let plane_col = first_plane_col
                + i32::try_from(visible_col).expect("visible col fits i32 for storm plane");
            let pattern_col = plane_col.rem_euclid(8) as usize;
            let source_word = source_words[pattern_row * 8 + pattern_col];
            let tile = TileRef::from_attr_word(source_word.wrapping_add(tile_addend));
            if tile.tile_index < visuals.vram_base_tile_index {
                continue;
            }
            let local_tile_index = tile.tile_index - visuals.vram_base_tile_index;
            if usize::from(local_tile_index) >= visuals.tile_sheet.tiles.len() {
                continue;
            }
            commands.push(DrawCommand::IndexedTile {
                x: plane_col * 8 - scroll_x_px,
                y,
                tile_sheet: tile_sheet.clone(),
                tile: TileRef {
                    tile_index: local_tile_index,
                    ..tile
                },
                palette: palette.clone(),
            });
        }
    }
    true
}

fn push_storm_weather_sprite_sub_61ca(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    visuals: &wildside_render::WeatherVisuals,
    sprite: (u16, u16, u16, u16),
) -> bool {
    let (screen_y, _link_word, tile_word, screen_x) = sprite;
    let tile_ref = TileRef::from_attr_word(tile_word);
    let (tile_sheet, vram_base_tile_index) =
        storm_weather_tile_sheet_loc_14d8a(trace, visuals, tile_ref.tile_index);
    let Some(tile_base_index) = tile_ref.tile_index.checked_sub(vram_base_tile_index) else {
        return false;
    };
    if usize::from(tile_base_index) >= tile_sheet.tiles.len() {
        return false;
    }
    let frame = wildside_render::SpriteFrame {
        origin_x: 0,
        origin_y: 0,
        width_px: 8,
        height_px: 8,
        tile_base_index,
        tile_sheet: tile_sheet.clone(),
    };
    let mut tiles = build_sprite_tiles(&frame, Direction::Right, tile_ref.palette_line);
    for tile in &mut tiles {
        tile.tile.priority = tile_ref.priority;
        tile.tile.flip_x = tile_ref.flip_x;
        tile.tile.flip_y = tile_ref.flip_y;
    }
    commands.push(DrawCommand::Sprite {
        x: i32::from(screen_x) - VDP_SPRITE_SCREEN_BIAS_LOC_13C0,
        y: i32::from(screen_y) - VDP_SPRITE_SCREEN_BIAS_LOC_13C0,
        tile_sheet: Arc::new(tile_sheet.clone()),
        palette: Arc::new(visuals.palette.clone()),
        tiles,
    });
    true
}

fn storm_weather_tile_sheet_loc_14d8a<'a>(
    trace: &NativeTraceFrame,
    visuals: &'a wildside_render::WeatherVisuals,
    tile_index: u16,
) -> (&'a wildside_render::TileSheet, u16) {
    let Some(upload) = &trace.storm_shared_art_upload_loc_14d8a else {
        return (&visuals.tile_sheet, visuals.vram_base_tile_index);
    };
    let Some(shared_art) = &visuals.shared_art else {
        return (&visuals.tile_sheet, visuals.vram_base_tile_index);
    };
    // ASM Init_SpecialEffect_Storm loc_14D8A decompresses `ArtComp_9A7D2` to
    // VRAM byte offset `$76A0` after the rain/hail `$7000` upload. Later
    // `sub_61CA` sprite tile words can reference that shared tail.
    let vram_base_tile_index = upload.vram_word / 32;
    if vram_base_tile_index == shared_art.vram_base_tile_index
        && tile_index >= shared_art.vram_base_tile_index
        && usize::from(tile_index - shared_art.vram_base_tile_index)
            < shared_art.tile_sheet.tiles.len()
    {
        (&shared_art.tile_sheet, shared_art.vram_base_tile_index)
    } else {
        (&visuals.tile_sheet, visuals.vram_base_tile_index)
    }
}

fn push_lava_geyser_template_loc_14be8(
    commands: &mut Vec<DrawCommand>,
    visuals: &wildside_render::LavaVisuals,
    palette: &Palette,
    geyser_x_px: u16,
    camera_x_px: u16,
    screen_y_px: i32,
    templates: &[wildside_core::LavaGeyserSpriteTemplateLoc14BE8],
) {
    let initial_screen_y_with_bias = (screen_y_px + 0x90).clamp(0, i32::from(u16::MAX)) as u16;
    for chunk in wildside_core::lava_geyser_sprite_chunks_loc_14BE8(
        geyser_x_px,
        camera_x_px,
        initial_screen_y_with_bias,
        0,
        templates,
    ) {
        push_lava_geyser_chunk_loc_14be8(commands, visuals, palette, chunk);
    }
}

fn push_lava_geyser_rom_sprites_loc_14be8(
    commands: &mut Vec<DrawCommand>,
    visuals: &wildside_render::LavaVisuals,
    palette: &Palette,
    geyser_x_px: u16,
    geyser_top_y_px: u16,
    height_px: u16,
    sprite_row_index: u16,
    top_sprite_frame_within_row: u8,
    camera_x_px: u16,
    camera_y_px: u16,
) {
    let base_y_px = geyser_top_y_px.saturating_add(height_px);
    let rows = if sprite_row_index == u16::MAX || top_sprite_frame_within_row == u8::MAX {
        wildside_core::lava_geyser_full_height_rows_loc_14874(
            geyser_x_px,
            base_y_px,
            height_px,
            camera_x_px,
            camera_y_px,
            0,
        )
    } else {
        wildside_core::lava_geyser_full_height_rows_for_sprite_row_loc_14874(
            geyser_x_px,
            base_y_px,
            sprite_row_index,
            camera_x_px,
            camera_y_px,
            top_sprite_frame_within_row,
        )
    };
    for row in rows {
        let Some(templates) = lava_geyser_full_height_template_loc_14874(row.source_label) else {
            continue;
        };
        // ASM loc_14874/loc_148F8 passes row `d1` directly to sub_14BD8.
        // The shared push helper adds the `$10` top-loop compensation before
        // converting back into the sub_14BD8 biased screen coordinate.
        push_lava_geyser_template_loc_14be8(
            commands,
            visuals,
            palette,
            geyser_x_px,
            camera_x_px,
            i32::from(row.row_y_px) - i32::from(camera_y_px) - 0x10,
            templates,
        );
    }
}

fn lava_geyser_full_height_template_loc_14874(
    source_label: &str,
) -> Option<&'static [wildside_core::LavaGeyserSpriteTemplateLoc14BE8]> {
    match source_label {
        "unk_14624" => Some(&LAVA_GEYSER_TOP_TEMPLATE_UNK_14624),
        "dword_14636" => Some(&LAVA_GEYSER_TOP_ALT_TEMPLATE_DWORD_14636),
        "unk_1465A" => Some(&LAVA_GEYSER_MIDDLE_TEMPLATE_UNK_1465A),
        "unk_14650" => Some(&LAVA_GEYSER_BOTTOM_TEMPLATE_UNK_14650),
        _ => None,
    }
}

fn lava_geyser_script_template_loc_147ba(
    source_label: &str,
) -> Option<&'static [wildside_core::LavaGeyserSpriteTemplateLoc14BE8]> {
    match source_label {
        "unk_145CA" => Some(&LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145CA),
        "unk_145D4" => Some(&LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145D4),
        "unk_145DE" => Some(&LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145DE),
        "unk_145E8" => Some(&LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_145E8),
        "unk_14602" => Some(&LAVA_GEYSER_SCRIPT_TEMPLATE_UNK_14602),
        _ => None,
    }
}

fn push_lava_geyser_script_template_loc_147ba(
    commands: &mut Vec<DrawCommand>,
    visuals: &wildside_render::LavaVisuals,
    palette: &Palette,
    template: &wildside_core::LavaGeyserScriptTemplateTraceLoc147BA,
    camera_x_px: u16,
    camera_y_px: u16,
) -> bool {
    let Some(templates) = lava_geyser_script_template_loc_147ba(&template.source_label) else {
        return false;
    };
    if !wildside_core::lava_geyser_is_inside_camera_window_loc_147D6(
        wildside_core::LavaGeyserCameraWindowInputLoc147D6 {
            x_px: template.x_px,
            y_px: template.base_y_px,
            camera_x_px,
            camera_y_px,
        },
    ) {
        return true;
    }
    // ASM loc_147BA/loc_1480C hands `sub_14BD8` the geyser baseline `d1`.
    // `push_lava_geyser_template_loc_14be8` adds the top-loop `$10` bias, so
    // subtract it here to leave the script rows at their ROM baseline.
    let base_screen_y = i32::from(template.base_y_px) - i32::from(camera_y_px) - 0x10;
    push_lava_geyser_template_loc_14be8(
        commands,
        visuals,
        palette,
        template.x_px,
        camera_x_px,
        base_screen_y,
        templates,
    );
    true
}

fn push_eol_number_sprites_with_palette(
    commands: &mut Vec<DrawCommand>,
    eol: &wildside_render::EolScoreVisuals,
    palette: &Palette,
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
            // ASM Object_EOL_Number loc_C466 sets `priority(a1)=1`, and
            // loc_C4E2 spaces seven digit workers 12px apart.
            push_sprite_frame_with_priority(
                commands,
                frame,
                palette,
                x + slot as i32 * 12,
                y,
                Some(true),
            );
        }
    }
}

fn push_eol_level_skip_time_digits_with_palette_loc_d47e(
    commands: &mut Vec<DrawCommand>,
    eol: &wildside_render::EolScoreVisuals,
    palette: &Palette,
    trace: &NativeTraceFrame,
) {
    if trace.level_skip_cheat_enabled == 0 || trace.level_complete_score_rows_started == 0 {
        return;
    }

    // ASM loc_D468/loc_D47E: when `LevelSkip_Cheat` is set, emit three
    // completion-time digit gfx objects at camera-relative x `$11C`, `$110`,
    // `$104`, y `$D4`, consuming ones/tens/hundreds by repeated `divu #$A`.
    // These are direct gfx-object coordinates, not pre-resolved top-lefts.
    let mut remaining = trace.level_completion_time;
    for x in [0x11C, 0x110, 0x104] {
        let digit = remaining % 10;
        remaining /= 10;
        if let Some(frame) = eol.digits.get(usize::from(digit)) {
            // ASM loc_D47E allocates the level-skip time digits through the
            // same priority=$01 EOL digit gfx-object setup as loc_C466.
            push_sprite_frame_with_priority(commands, frame, palette, x, 0xD4, Some(true));
        }
    }
}

fn eol_position_loc_c5a2(initial: i32, velocity: i32, timer: usize, elapsed: usize) -> i32 {
    // ASM Object_EOL_Text/Object_EOL_Number loc_C5A2 updates `$64/$68` by
    // `$6C/$70` until `$46` or `$A` reaches zero, then clears velocity and
    // leaves the worker anchored at its settled coordinate.
    let moving_frames = elapsed.min(timer);
    initial + velocity * i32::try_from(moving_frames).unwrap_or(i32::MAX)
}

fn eol_number_position_loc_c48c(initial: i32, velocity: i32, timer: usize, elapsed: usize) -> i32 {
    // ASM Object_EOL_Number loc_C48C draws digits from `$64/$68`, then loc_C4E2
    // advances `$64/$68` and decrements `$A`. Text workers in loc_C5A2 move
    // before drawing, so EOL numbers are visibly one worker tick behind text.
    let moving_frames = elapsed.saturating_sub(1).min(timer);
    initial + velocity * i32::try_from(moving_frames).unwrap_or(i32::MAX)
}

fn push_eol_text_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
    eol: &wildside_render::EolScoreVisuals,
    initial_x: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    push_eol_text_with_velocity_loc_c5a2(commands, frame, eol, initial_x, 8, y, timer, elapsed);
}

fn push_eol_text_with_palette_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
    eol: &wildside_render::EolScoreVisuals,
    palette: &Palette,
    initial_x: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    push_eol_text_with_velocity_and_palette_loc_c5a2(
        commands, frame, eol, palette, initial_x, 8, y, timer, elapsed,
    );
}

fn push_eol_text_with_velocity_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
    eol: &wildside_render::EolScoreVisuals,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    push_eol_text_with_velocity_and_palette_loc_c5a2(
        commands,
        frame,
        eol,
        &eol.palette,
        initial_x,
        velocity,
        y,
        timer,
        elapsed,
    );
}

fn push_eol_text_with_velocity_and_palette_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
    _eol: &wildside_render::EolScoreVisuals,
    palette: &Palette,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
) {
    let x = eol_position_loc_c5a2(initial_x, velocity, timer, elapsed);
    // ASM Object_EOL_Text loc_C5A2 writes directly to the gfx object's
    // x_pos/y_pos fields after adding the camera, so render from sprite
    // coordinates rather than treating `$64/$68` as top-lefts. loc_C592 sets
    // `priority(a3)=1`, so these rows draw above high-priority foreground.
    push_sprite_frame_with_priority(commands, frame, palette, x, y, Some(true));
}

fn push_eol_number_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    eol: &wildside_render::EolScoreVisuals,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
    value: u32,
) {
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        &eol.palette,
        initial_x,
        velocity,
        y,
        timer,
        elapsed,
        value,
    );
}

fn push_eol_number_with_palette_loc_c5a2(
    commands: &mut Vec<DrawCommand>,
    eol: &wildside_render::EolScoreVisuals,
    palette: &Palette,
    initial_x: i32,
    velocity: i32,
    y: i32,
    timer: usize,
    elapsed: usize,
    value: u32,
) {
    let x = eol_number_position_loc_c48c(initial_x, velocity, timer, elapsed);
    // ASM Object_EOL_Number loc_C4E2 writes each digit worker's gfx-object
    // x_pos/y_pos directly from `$64/$68`.
    push_eol_number_sprites_with_palette(commands, eol, palette, x, y, value);
}

fn push_eol_bonus_row_with_palette_loc_d468(
    commands: &mut Vec<DrawCommand>,
    eol: &wildside_render::EolScoreVisuals,
    palette: &Palette,
    label: &wildside_render::SpriteFrame,
    label_initial_x: i32,
    bonus_initial_x: i32,
    y: i32,
    number_initial_x: i32,
    timer: usize,
    elapsed: usize,
    value: u32,
) {
    push_eol_text_with_palette_loc_c5a2(
        commands,
        label,
        eol,
        palette,
        label_initial_x,
        y,
        timer,
        elapsed,
    );
    push_eol_text_with_palette_loc_c5a2(
        commands,
        &eol.bonus,
        eol,
        palette,
        bonus_initial_x,
        y,
        timer,
        elapsed,
    );
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        palette,
        number_initial_x,
        -8,
        y,
        timer,
        elapsed,
        value,
    );
}

fn eol_bonus_amounts_for_trace(trace: &NativeTraceFrame) -> ([u32; 5], usize) {
    let mut amounts = [0; 5];
    let mut count = 0;
    let mut push = |amount: u32| {
        amounts[count] = amount;
        count += 1;
    };

    push(wildside_core::time_bonus_loc_d56e(
        trace.level_timer_seconds,
    ));
    if !trace.level_damage_taken {
        push(5_000);
    }
    if !trace.level_prize_collected {
        push(5_000);
    }
    if trace.level_path_bonus_amount > 0 {
        push(trace.level_path_bonus_amount);
    }
    if trace.level_speed_bonus_awarded {
        push(10_000);
    }

    (amounts, count)
}

fn eol_score_counter_snapshot_for_trace(
    trace: &NativeTraceFrame,
) -> wildside_core::EolScoreCounterSnapshotLocD770 {
    let (amounts, count) = eol_bonus_amounts_for_trace(trace);
    let total_bonus = amounts[..count].iter().copied().sum::<u32>();
    let initial_score = if trace.level_complete && trace.level_complete_initial_score > 0 {
        trace.level_complete_initial_score
    } else {
        trace.score.saturating_sub(total_bonus)
    };
    let transfer_frames = wildside_core::eol_score_counter_transfer_frames_from_trace_loc_D770(
        trace.level_complete_score_counter_elapsed_frames,
    );
    wildside_core::eol_score_counter_snapshot_loc_D770(
        initial_score,
        &amounts[..count],
        transfer_frames,
    )
}

fn eol_score_room_palette_loc_d734(trace: &NativeTraceFrame, palette: &Palette) -> Palette {
    if trace.level_complete_score_rows_elapsed_frames
        < EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE
    {
        return palette.clone();
    }

    // ASM loc_D734: after the score-row setup, the ROM backs up
    // `Palette_Buffer`, clears `Palette_Buffer_3`, then runs eight
    // `sub_14C0` fade steps to `$0080`. The native asset stores the extracted
    // score-room colours as one flattened Pal_D00C palette, but oracle frames
    // show the grid greens/white stay on the bright CRAM line while the room
    // wall/stone colours are the dimmed loc_D734 result.
    Palette {
        colours: palette
            .colours
            .iter()
            .map(|colour| {
                let raw_word = match colour.raw_word {
                    0x0000 | 0x0040 | 0x00E0 | 0x0EEE => colour.raw_word,
                    raw_word => {
                        wildside_core::eol_title_return_faded_palette_word_loc_D734(raw_word)
                    }
                };
                wildside_render::MdColour::from_word(raw_word)
            })
            .collect(),
    }
}

fn push_eol_score_overlay(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    eol: &wildside_render::EolScoreVisuals,
) {
    let palette = eol_score_room_palette_loc_d734(trace, &eol.palette);
    push_eol_level_skip_time_digits_with_palette_loc_d47e(commands, eol, &palette, trace);

    if trace.level_complete_score_rows_elapsed_frames
        < EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE
    {
        return;
    }

    // ASM loc_D4CE emits CONTINUES and SCORE first, then the bonus rows with
    // Load_Object_EOL_Text and Load_Object_EOL_Number. Native follows the ROM
    // `$64/$68/$6C` seeds directly and advances them through loc_C5A2.
    // ASM loc_D770 then drains each number object's `$60` into the score
    // display; native derives that visual state from the level-complete
    // elapsed frame while keeping progression score awarded at flag touch.
    let counter = eol_score_counter_snapshot_for_trace(trace);
    let elapsed = trace.level_complete_score_rows_elapsed_frames
        - EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE
        + EOL_SCORE_ROWS_RENDER_LATCH_LEAD_FRAMES_LOC_C070_ORACLE;
    push_eol_text_with_palette_loc_c5a2(
        commands,
        &eol.continues,
        eol,
        &palette,
        -0xB8,
        0xCC,
        0x1C,
        elapsed,
    );
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        &palette,
        -0x7C,
        8,
        0xCC,
        0x1C,
        elapsed,
        u32::from(trace.number_continues),
    );
    push_eol_text_with_palette_loc_c5a2(
        commands, &eol.score, eol, &palette, -0x68, 0xB4, 0x1C, elapsed,
    );
    push_eol_number_with_palette_loc_c5a2(
        commands,
        eol,
        &palette,
        0x1B0,
        -8,
        0xB4,
        0x1C,
        elapsed,
        counter.score,
    );

    let mut amount_index = 0usize;
    let mut y = 0x9C;
    let mut timer = 0x24usize;
    // ASM loc_D4CE subtracts `$18` from d7 and adds `$40` to d6 after the
    // SCORE row, before loading the first TIME/BONUS row.
    let mut d6 = 0x40;
    push_eol_bonus_row_with_palette_loc_d468(
        commands,
        eol,
        &palette,
        &eol.time,
        -0xA0 - d6,
        -0x68 - d6,
        y,
        0x1B0 + d6,
        timer,
        elapsed,
        counter.remaining_amounts[amount_index],
    );
    amount_index += 1;
    y -= 0x18;
    timer += 8;
    d6 += 0x40;
    if !trace.level_damage_taken {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.no_hit,
            -0xC8 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        amount_index += 1;
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
    }
    if !trace.level_prize_collected {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.no_prize,
            -0xE0 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        amount_index += 1;
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
    }
    if trace.level_path_bonus_amount > 0 {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.path,
            -0xA8 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        amount_index += 1;
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
    }
    if trace.level_speed_bonus_awarded {
        push_eol_bonus_row_with_palette_loc_d468(
            commands,
            eol,
            &palette,
            &eol.speed,
            -0xB0 - d6,
            -0x68 - d6,
            y,
            0x1B0 + d6,
            timer,
            elapsed,
            counter.remaining_amounts[amount_index],
        );
        y -= 0x18;
        timer += 8;
        d6 += 0x40;
        push_eol_text_with_palette_loc_c5a2(
            commands,
            &eol.under,
            eol,
            &palette,
            -0xB0 - d6,
            y,
            timer,
            elapsed,
        );
        // ASM loc_D660: the "UNDER ... SECONDS" number is the threshold byte
        // loaded from SpeedBonus_Values[Current_LevelID], not remaining time.
        let speed_threshold = wildside_core::speed_bonus_threshold_loc_d660(trace.current_level)
            .map_or(u32::from(trace.level_timer_seconds), u32::from);
        push_eol_number_with_palette_loc_c5a2(
            commands,
            eol,
            &palette,
            -0xA4 - d6,
            8,
            y,
            timer,
            elapsed,
            speed_threshold,
        );
        push_eol_text_with_palette_loc_c5a2(
            commands,
            &eol.seconds,
            eol,
            &palette,
            -0x4C - d6,
            y,
            timer,
            elapsed,
        );
    }
}

fn push_game_over_continue_overlay(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    eol: &wildside_render::EolScoreVisuals,
) {
    if let Some(_countdown) = trace.continue_countdown {
        // ASM Continue_Screen (kid.asm:16505): allocate Spr_EOL_Continues at
        // `$64=$FF88,$68=$60,$6C=8`, then Object_EOL_Number at
        // `$64=$134,$68=$80,$6C=$FFF8` using Number_Continues. The ROM then
        // hibernates for `$1C` frames at loc_D31E before loc_D350 starts the
        // interactive countdown; keep those objects at their spawn coordinates
        // until the same hold has elapsed.
        let elapsed = trace
            .continue_countdown_elapsed_frames
            .saturating_sub(CONTINUE_SCREEN_INITIAL_HOLD_FRAMES_LOC_D31E);
        push_eol_text_loc_c5a2(
            commands,
            &eol.continues,
            eol,
            -0x78,
            0x60,
            EOL_TEXT_WORKER_SLIDE_FRAMES_LOC_C5A2,
            elapsed,
        );
        push_eol_number_loc_c5a2(
            commands,
            eol,
            0x134,
            -8,
            0x80,
            EOL_TEXT_WORKER_SLIDE_FRAMES_LOC_C5A2,
            elapsed,
            u32::from(trace.number_continues),
        );
    } else if trace.number_continues == 0 {
        let elapsed = trace.game_over_elapsed_frames;
        // ASM loc_D25E (kid.asm:16472): with no continues, GAME enters from
        // x=$FF98 with +8 velocity and OVER enters from x=$158 with -8.
        // If continues remain, the ROM branches to Continue_Screen instead of
        // showing GAME OVER, so native must not render a pre-continue panel.
        push_eol_text_loc_c5a2(
            commands,
            &eol.game,
            eol,
            -0x68,
            0x60,
            EOL_TEXT_WORKER_SLIDE_FRAMES_LOC_C5A2,
            elapsed,
        );
        push_eol_text_with_velocity_loc_c5a2(
            commands,
            &eol.over,
            eol,
            0x158,
            -8,
            0x80,
            EOL_TEXT_WORKER_SLIDE_FRAMES_LOC_C5A2,
            elapsed,
        );
    }
}

fn push_pause_menu_text_loc_69d2(commands: &mut Vec<DrawCommand>, trace: &NativeTraceFrame) {
    // ASM loc_69D2/restart_text writes a 16x5-tile pause menu block at
    // screen-space Camera_X+$60, Camera_Y+$40. The ROM chooses the lower row
    // text from Number_Lives: >1 means Restart Round, otherwise Give Up.
    let lower_row = match wildside_core::pause_menu_text_selection_restart_text(trace.number_lives)
    {
        wildside_core::PauseMenuTextSelectionRestartText::RestartRound => "RESTART ROUND",
        wildside_core::PauseMenuTextSelectionRestartText::GiveUp => "GIVE UP",
    };

    // ASM loc_6A84 seeds Pause_Option=0 and writes the upper cursor command at
    // Camera_X+$68, Camera_Y+$48; Game_Paused_Loop loc_6B0E moves it to the
    // lower row at Camera_Y+$58 for Restart Round / Give Up.
    let cursor_y = if trace.pause_menu_option == 0 {
        PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT
    } else {
        PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT
    };
    commands.push(text(
        PAUSE_MENU_CURSOR_X_LOC_6A84,
        cursor_y,
        ">",
        [255, 224, 96],
        1,
    ));
    commands.push(text(
        PAUSE_MENU_TEXT_X_LOC_69D2,
        PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT,
        "CONTINUE",
        [255, 255, 255],
        1,
    ));
    commands.push(text(
        PAUSE_MENU_TEXT_X_LOC_69D2,
        PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT,
        lower_row,
        [255, 255, 255],
        1,
    ));
}

fn push_pause_menu_tilemap_loc_69d2(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) -> bool {
    let Some(pause_menu) = level_visuals.pause_menu.as_ref() else {
        return false;
    };
    let rows = match wildside_core::pause_menu_text_selection_restart_text(trace.number_lives) {
        wildside_core::PauseMenuTextSelectionRestartText::RestartRound => &pause_menu.restart_rows,
        wildside_core::PauseMenuTextSelectionRestartText::GiveUp => &pause_menu.give_up_rows,
    };
    let tile_sheet = Arc::new(pause_menu.tile_sheet.clone());
    let palette = Arc::new(foreground_palette_for_scene(trace, level_visuals));
    for (row_index, row) in rows.iter().take(PAUSE_MENU_TILE_ROWS_LOC_69D2).enumerate() {
        for (column_index, &tile) in row
            .iter()
            .take(PAUSE_MENU_TILE_COLUMNS_LOC_69D2)
            .enumerate()
        {
            let tile =
                pause_menu_tile_with_live_cursor_loc_6b0e(trace, row_index, column_index, tile);
            commands.push(DrawCommand::IndexedTile {
                x: PAUSE_MENU_TILE_X_LOC_69D2 + column_index as i32 * 8,
                y: PAUSE_MENU_TILE_Y_RESTART_TEXT + row_index as i32 * 8,
                tile_sheet: Arc::clone(&tile_sheet),
                tile,
                palette: Arc::clone(&palette),
            });
        }
    }
    true
}

fn pause_menu_tile_with_live_cursor_loc_6b0e(
    trace: &NativeTraceFrame,
    row_index: usize,
    column_index: usize,
    mut tile: TileRef,
) -> TileRef {
    // ASM Game_Paused_Loop loc_6B0E rewrites the two cursor slots only:
    // `$867C` is the arrow tile and `$8678` is blank filler.
    if column_index == 1 && row_index == 1 {
        tile.tile_index = if trace.pause_menu_option == 0 {
            PAUSE_MENU_CURSOR_TILE_LOC_6B0E
        } else {
            PAUSE_MENU_BLANK_TILE_LOC_6B0E
        };
    } else if column_index == 1 && row_index == 3 {
        tile.tile_index = if trace.pause_menu_option == 0 {
            PAUSE_MENU_BLANK_TILE_LOC_6B0E
        } else {
            PAUSE_MENU_CURSOR_TILE_LOC_6B0E
        };
    }
    tile
}

pub fn hide_pause_menu_overwritten_sprite_commands_sub_6874(commands: &mut Vec<DrawCommand>) {
    // ASM loc_68AC-loc_68C0: sub_6874 scans the hardware sprite table and
    // clears the Y word for sprites whose masked first pattern tile overlaps
    // the pause-menu art upload range `$676..=$68E`.
    commands.retain(|command| {
        let DrawCommand::Sprite { tiles, .. } = command else {
            return true;
        };
        let Some(first_tile) = tiles.first() else {
            return true;
        };
        let masked_tile = first_tile.tile.tile_index & 0x07FF;
        !(0x0676..=0x068E).contains(&masked_tile)
    });
}

// Forest water ripple table from ASM: BackgroundScroll_ForestWaterRippleData
// (kid.asm line 34310). Duplicated so loc_3060A can walk through phase wrap.
const FOREST_RIPPLE_LOC_3060A: [u8; 128] = [
    0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5, 6, 7, 7, 7, 8, 8, 8, 8, 7, 7, 7, 6, 5, 4, 3, 2, 2, 2,
    1, 1, 1, 0, 0, 0, 0, 1, 2, 3, 4, 4, 4, 4, 5, 6, 7, 8, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 1,
    0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5, 6, 7, 7, 7, 8, 8, 8, 8, 7, 7, 7, 6, 5, 4, 3, 2, 2, 2,
    1, 1, 1, 0, 0, 0, 0, 1, 2, 3, 4, 4, 4, 4, 5, 6, 7, 8, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 1,
];
// Desert heat ripple table from ASM: BackgroundScroll_DesertHeatRippleData
// (kid.asm line 34494). loc_306D4 masks Time_Frames with `$30`, and loc_306F4
// consumes at most 16 bytes, so the 64-byte ROM table covers every phase.
const DESERT_HEAT_RIPPLE_LOC_306F4: [u8; 64] = [
    0, 0xFF, 0, 1, 0, 0, 0xFF, 0, 1, 0, 0, 0xFF, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0xFF, 0, 1, 0, 0, 0,
    0xFF, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0xFF, 0, 1, 0, 0, 0xFF, 0, 1, 0, 0, 0xFF, 0, 0, 0, 0, 0xFF, 0,
    1, 0, 0, 0xFF, 0, 1, 0, 0, 0xFF, 0,
];
const ZERO_HORIZ_SCROLL_BUFFER_LOC_305E6: [u32; FRAME_HEIGHT as usize] = [0; FRAME_HEIGHT as usize];
const SETTLED_WARP_FOREST_RIPPLE_TIME_FRAMES_OFFSET_LOC_305E6: usize = 52;

fn background_time_frames_for_trace_loc_3050a(trace: &NativeTraceFrame) -> usize {
    // ASM BackgroundScroll_ComputeShiftData / loc_305E6 use the global
    // `Time_Frames` counter, not the recording-local frame index.
    if trace.time_frames_loc_3038a != 0 {
        return usize::from(trace.time_frames_loc_3038a);
    }
    // Legacy traces predate the typed Time_Frames latch. Settled warp
    // recordings replay through the ROM level-skip path and keep
    // `LevelSkip_Cheat` armed; their visible frame-based Scroll B effects are
    // 52 VBlanks ahead of the rebased native recording clock in the #175
    // forest ripple route.
    if trace.level_skip_cheat_enabled != 0 {
        trace
            .frame_index
            .saturating_add(SETTLED_WARP_FOREST_RIPPLE_TIME_FRAMES_OFFSET_LOC_305E6)
    } else {
        trace.frame_index
    }
}

pub fn forest_water_ripple_time_frames_for_trace_loc_305e6(trace: &NativeTraceFrame) -> usize {
    background_time_frames_for_trace_loc_3050a(trace)
}

fn background_ripple_offset_for_screen_y_sub_30598(
    trace: &NativeTraceFrame,
    background_theme_id: u8,
    screen_y: i32,
    effect_start_row_selector: u8,
    camera_y_px: i32,
) -> Option<i32> {
    match wildside_core::background_scroll_theme_dispatch_loc_30598(u16::from(background_theme_id))
    {
        wildside_core::BackgroundScrollThemeDispatchLoc30598::ApplyForestWaterRipple => {
            forest_water_ripple_offset_for_screen_y_sub_30598(
                forest_water_ripple_time_frames_for_trace_loc_305e6(trace),
                screen_y,
                effect_start_row_selector,
                camera_y_px,
            )
        }
        wildside_core::BackgroundScrollThemeDispatchLoc30598::ApplyDesertHeatRipple => {
            desert_heat_ripple_offset_for_screen_y_sub_30598(
                background_time_frames_for_trace_loc_3050a(trace),
                screen_y,
                effect_start_row_selector,
                camera_y_px,
            )
        }
        wildside_core::BackgroundScrollThemeDispatchLoc30598::Return => None,
    }
}

pub fn forest_water_ripple_offset_for_screen_y_sub_30598(
    frame_index: usize,
    screen_y: i32,
    effect_start_row_selector: u8,
    camera_y_px: i32,
) -> Option<i32> {
    // ASM BackgroundScroll_ApplyForestWaterRipple loc_30598-loc_3060A:
    // start at `(map_header+$14)[0] * 8 - (Camera_Y_pos >> 2)`, affect at
    // most 64 visible scanlines, then add one table byte per screen line.
    let start_y = i32::from(effect_start_row_selector) * 8 - (camera_y_px >> 2);
    if !(0..FRAME_HEIGHT as i32).contains(&screen_y) {
        return None;
    }
    let (visible_start, table_start, line_count) = if start_y < 0 {
        (0, -start_y, (64 + start_y).clamp(0, 64))
    } else {
        let visible_band =
            wildside_core::background_scroll_forest_water_ripple_visible_band_loc_305D4(
                start_y as i16,
            )?;
        (
            i32::from(visible_band.start_scanline),
            i32::from(visible_band.ripple_data_byte_offset),
            i32::from(visible_band.affected_scanline_count),
        )
    };
    let setup = wildside_core::background_scroll_forest_water_ripple_setup_loc_305E6(
        &ZERO_HORIZ_SCROLL_BUFFER_LOC_305E6,
        visible_start as u16,
        line_count as u16,
        table_start as u16,
        frame_index as u16,
    )?;
    let visible_end = visible_start + line_count;
    if screen_y < visible_start || screen_y >= visible_end {
        return None;
    }

    let ripple_buffer = wildside_core::background_scroll_forest_water_ripple_loop_loc_3060A(
        &ZERO_HORIZ_SCROLL_BUFFER_LOC_305E6,
        setup.start_scanline,
        setup.ripple_dbf_counter,
        setup.base_scroll_word,
        &FOREST_RIPPLE_LOC_3060A,
        setup.ripple_data_byte_offset,
    )?;
    let scroll_word = ripple_buffer.get(screen_y as usize).copied()? as u16;
    Some(i32::from(scroll_word as i16))
}

pub fn desert_heat_ripple_offset_for_screen_y_sub_30598(
    frame_index: usize,
    screen_y: i32,
    effect_start_row_selector: u8,
    camera_y_px: i32,
) -> Option<i32> {
    // ASM BackgroundScroll_ApplyDesertHeatRipple loc_306C2-loc_306F4:
    // start at `(map_header+$14)[0] * 8 - (Camera_Y_pos >> 2)`, affect at
    // most 16 visible scanlines, then add signed heat-ripple table bytes.
    if !(0..FRAME_HEIGHT as i32).contains(&screen_y) {
        return None;
    }
    let visible_band = wildside_core::background_scroll_desert_heat_ripple_visible_band_loc_306C2(
        effect_start_row_selector,
        camera_y_px as i16,
    )?;
    let visible_start = i32::from(visible_band.start_scanline);
    let line_count = i32::from(visible_band.affected_scanline_count);
    let visible_end = visible_start + line_count;
    if screen_y < visible_start || screen_y >= visible_end {
        return None;
    }

    let setup = wildside_core::background_scroll_desert_heat_ripple_setup_loc_306D4(
        &ZERO_HORIZ_SCROLL_BUFFER_LOC_305E6,
        visible_band.start_scanline,
        visible_band.affected_scanline_count,
        visible_band.ripple_data_byte_offset,
        frame_index as u16,
    )?;
    let ripple_buffer = wildside_core::background_scroll_desert_heat_ripple_loop_loc_306F4(
        &ZERO_HORIZ_SCROLL_BUFFER_LOC_305E6,
        setup.start_scanline,
        setup.ripple_dbf_counter,
        setup.base_scroll_word,
        &DESERT_HEAT_RIPPLE_LOC_306F4,
        setup.ripple_data_byte_offset,
    )?;
    let scroll_word = ripple_buffer.get(screen_y as usize).copied()? as u16;
    Some(i32::from(scroll_word as i16))
}

pub fn background_tile_screen_position(
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
    if level_visuals.level.background_scroll.is_none() {
        return (
            world_x_px.saturating_sub(camera_x_px),
            world_y_px.saturating_sub(background_scroll_y_px),
        );
    }

    let layer_speed = background_layer_speed_loc_3050a(level_visuals, usize::from(y_tiles));
    (
        world_x_px.saturating_sub(background_display_scroll_x_px_loc_3050a(
            trace,
            camera_x_px,
            layer_speed,
        )),
        world_y_px.saturating_sub(background_scroll_y_px),
    )
}

fn background_layer_speed_loc_3050a(level_visuals: &NativeLevelVisuals, y_tiles: usize) -> u8 {
    level_visuals
        .level
        .background_scroll
        .as_ref()
        .and_then(|profile| profile.layer_speeds.get(y_tiles).copied())
        .unwrap_or(0x10)
}

pub fn layered_background_row_wraps_loc_3050a(
    level_visuals: &NativeLevelVisuals,
    y_tiles: usize,
) -> bool {
    // ASM BackgroundScroll_ComputeShiftLayers loc_3050A resolves bgscroll rows
    // from the per-level profile and writes 224 scanline scroll values into
    // the hardware-wrapping Plane B nametable. Static `$10` rows can remain
    // finite, but moving rows need native repetition when parallax exposes the
    // right seam; the flagpole hand-off guard below handles the one known
    // forest no-scroll transition case.
    let layer_speed = background_layer_speed_loc_3050a(level_visuals, y_tiles);
    layer_speed != 0x00
        && layer_speed != 0x10
        && matches!(layer_speed, 0x08..=0x88 if layer_speed.is_multiple_of(8))
}

pub fn layered_background_row_wraps_for_trace_loc_3050a(
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    y_tiles: usize,
) -> bool {
    // ASM CharacterCollision_TouchFlagpole loc_C070 starts a frozen flagpole
    // hand-off. Native reports `level_complete` on the same flag-touch trace
    // tick, but oracle pixels still contain the previous Scroll B frame only
    // for elapsed=0; later hand-off frames stop repeating the forest rows.
    (!trace.level_complete || trace.level_complete_elapsed_frames == 0)
        && layered_background_row_wraps_loc_3050a(level_visuals, y_tiles)
}

pub fn wrap_layered_background_x_loc_11b7a(screen_x_px: i32, row_width_px: i32) -> i32 {
    if row_width_px <= 0 {
        return screen_x_px;
    }
    // ASM loc_11B7A builds Plane B row pointers and the VDP scroll plane wraps
    // horizontally. Forest rows are composed as two 32-tile planes (64 tiles);
    // without this wrap, parallax-scrolled rows that move more than one full
    // plane disappear to the left. Do not fold ordinary right-edge tiles back
    // into the viewport until they cross the whole Plane B row; the ROM still
    // leaves clear sky between x=320 and the 512px nametable wrap point.
    let mut wrapped = screen_x_px;
    while wrapped < -8 {
        wrapped += row_width_px;
    }
    while wrapped >= row_width_px {
        wrapped -= row_width_px;
    }
    wrapped
}

pub fn layered_background_tile_screen_xs_loc_11b7a(
    screen_x_px: i32,
    row_width_px: i32,
) -> [i32; 3] {
    // ASM loc_11B7A composes the layered background in a wrapping Plane B
    // nametable. A row whose H-scroll phase is 1..7 pixels therefore presents
    // the final tile again at the left edge, rather than exposing backdrop.
    let wrapped = wrap_layered_background_x_loc_11b7a(screen_x_px, row_width_px);
    [
        wrapped.saturating_sub(row_width_px),
        wrapped,
        wrapped.saturating_add(row_width_px),
    ]
}

pub fn layered_background_tile_horizontal_visible_sub_30598(
    screen_x_px: i32,
    effect_start_row_selector: Option<u8>,
) -> bool {
    // ASM BackgroundScroll_ApplyForestWaterRipple loc_30598-loc_3060A can
    // shift a scanline right by 8 pixels. Keep the wrapping tile at x=-8
    // alive until the per-scanline effect moves it into the viewport.
    let effect_margin = if effect_start_row_selector.is_some() {
        8
    } else {
        0
    };
    screen_x_px > -8 - effect_margin && screen_x_px < FRAME_WIDTH as i32 + effect_margin
}

pub fn wrap_layered_background_y_loc_11b7a(screen_y_px: i32, row_height_px: i32) -> i32 {
    if row_height_px <= 0 {
        return screen_y_px;
    }
    // ASM loc_11B7A backs Plane B with a row-pointer table and
    // BackgroundScroll_ComputeShiftLayers biases the visible rows from
    // Camera_Y_pos. Native must wrap the composed Plane B rows vertically so
    // tall Mountain levels keep background pixels behind transparent terrain.
    let mut wrapped = screen_y_px;
    while wrapped < -8 {
        wrapped += row_height_px;
    }
    while wrapped >= row_height_px {
        wrapped -= row_height_px;
    }
    wrapped
}

pub fn background_scroll_x_px(trace: &NativeTraceFrame, camera_x_px: i32, layer_speed: u8) -> i32 {
    background_scroll_x_px_for_time_loc_3050a(
        background_time_frames_for_trace_loc_3050a(trace),
        camera_x_px,
        layer_speed,
    )
}

pub fn background_display_scroll_x_px_loc_3050a(
    trace: &NativeTraceFrame,
    camera_x_px: i32,
    layer_speed: u8,
) -> i32 {
    i32::from(background_display_scroll_data_word_loc_3050a(trace, camera_x_px, layer_speed) as i16)
        .rem_euclid(512)
}

fn background_display_scroll_x_px_for_chunked_row_loc_3050a(
    trace: &NativeTraceFrame,
    camera_x_px: i32,
    camera_y_px: i32,
    row: usize,
    first_row: usize,
    last_row: usize,
    layer_speed: u8,
) -> i32 {
    let scroll_word =
        background_display_scroll_data_word_loc_3050a(trace, camera_x_px, layer_speed);
    let quarter_row_phase = ((camera_y_px as u16) >> 2) & 0x0007;
    let negated_camera_x_longword = u32::from((camera_x_px as i16).wrapping_neg() as u16) << 16;

    let packed_scroll_longword = if quarter_row_phase != 0 && row == first_row {
        wildside_core::background_scroll_shift_layers_leading_partial_fill_loc_30562(
            negated_camera_x_longword,
            scroll_word as i16,
            quarter_row_phase,
            0x001B,
        )
        .map(|fill| fill.repeated_scroll_longword)
    } else if quarter_row_phase != 0 && row == last_row {
        wildside_core::background_scroll_shift_layers_trailing_partial_fill_loc_30592(
            negated_camera_x_longword,
            scroll_word as i16,
            quarter_row_phase,
        )
        .map(|fill| fill.repeated_scroll_longword)
    } else {
        None
    };

    let hscroll_word = packed_scroll_longword.map_or_else(
        || (scroll_word as i16).wrapping_neg() as u16,
        |longword| longword as u16,
    );
    i32::from((hscroll_word as i16).wrapping_neg()).rem_euclid(512)
}

fn background_display_scroll_data_word_loc_3050a(
    trace: &NativeTraceFrame,
    camera_x_px: i32,
    layer_speed: u8,
) -> u16 {
    // ASM loc_3050A (kid.asm:34220-34239) copies the packed
    // `Horiz_Scroll_Data+4` word into the per-scanline H-scroll table after
    // negating it. The framebuffer samples show auto-scroll rows render from
    // the previous VDP table phase while sub_30744's row-upload window can use
    // a different preload-era phase.
    let time_frames = background_time_frames_for_trace_loc_3050a(trace);
    let display_time_frames = match layer_speed {
        0x00 | 0x08 => time_frames.saturating_sub(1),
        _ => time_frames,
    };
    background_scroll_data_word_loc_30452(display_time_frames, camera_x_px, layer_speed)
}

fn background_scroll_x_px_for_time_loc_3050a(
    time_frames: usize,
    camera_x_px: i32,
    layer_speed: u8,
) -> i32 {
    background_scroll_x_px_unwrapped_for_time_loc_3050a(time_frames, camera_x_px, layer_speed)
        .rem_euclid(512)
}

fn background_scroll_x_px_unwrapped_for_time_loc_3050a(
    time_frames: usize,
    camera_x_px: i32,
    layer_speed: u8,
) -> i32 {
    match layer_speed {
        0x00 => time_frames as i32 / 2,
        0x08 => time_frames as i32 / 4,
        0x10 => 0,
        0x18..=0x88 if (layer_speed - 0x10).is_multiple_of(8) => {
            let step = i32::from((layer_speed - 0x10) / 8);
            camera_x_px.saturating_mul(step) / 16
        }
        _ => camera_x_px,
    }
}

pub fn push_tile_quad(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    foreground_palette: &Palette,
    x: i32,
    y: i32,
    quad: &wildside_render::TileQuad,
) {
    for (index, tile_ref) in quad.tiles.iter().copied().enumerate() {
        let tile_x = x + if index % 2 == 0 { 0 } else { 8 };
        let tile_y = y + if index < 2 { 0 } else { 8 };
        if let Some(tile) = resolve_ingame_vram_tile_with_foreground_palette(
            level_visuals,
            foreground_palette,
            tile_ref,
        ) {
            push_resolved_tile(commands, tile_x, tile_y, tile);
        }
    }
}

fn push_tile_quad_with_priority_override(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    foreground_palette: &Palette,
    x: i32,
    y: i32,
    quad: &wildside_render::TileQuad,
    priority_override: Option<bool>,
) {
    match priority_override {
        Some(priority) => {
            let mut quad = quad.clone();
            for tile in &mut quad.tiles {
                tile.priority = priority;
            }
            push_tile_quad(commands, level_visuals, foreground_palette, x, y, &quad);
        }
        None => push_tile_quad(commands, level_visuals, foreground_palette, x, y, quad),
    }
}

fn prize_block_wobble_frame_sub_1007a(
    level_visuals: &NativeLevelVisuals,
    remaining: u8,
) -> wildside_render::SpriteFrame {
    let elapsed = 31_u8.saturating_sub(remaining);
    // ASM stru_101B6/stru_10296 (kid.asm:21447,21529): the prize-block
    // wobble worker alternates 16x16 sprite frames at VRAM tiles $220 and
    // $229 for three hibernated frames each while sub_1007A moves y_pos.
    let tile_base_index = if (elapsed / 3).is_multiple_of(2) {
        0x000
    } else {
        0x009
    };
    wildside_render::SpriteFrame {
        origin_x: 0x00,
        origin_y: 0x10,
        width_px: 0x10,
        height_px: 0x10,
        tile_base_index,
        tile_sheet: level_visuals.block_tiles.clone(),
    }
}

fn push_prize_block_wobble_sub_1007a(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    remaining: u8,
) {
    let _oracle_frame = prize_block_wobble_frame_sub_1007a(level_visuals, remaining);
    let Some(resolved) = level_visuals.level.block_mapping.resolve(
        PRIZE_BLOCK_VISIBLE_SKIN_LOC_12408,
        Some(wildside_core::terrain::BlockType::Prize as u8),
    ) else {
        return;
    };
    let tiles = resolved
        .quad
        .tiles
        .iter()
        .copied()
        .enumerate()
        .map(|(index, mut tile)| {
            let local_tile_index = tile
                .tile_index
                .checked_sub(BLOCK_TILE_VRAM_BASE)
                .unwrap_or(tile.tile_index);
            tile.tile_index = local_tile_index;
            tile.priority = false;
            SpriteTile {
                offset_x: if index % 2 == 0 { 0 } else { 8 },
                offset_y: if index < 2 { 0 } else { 8 },
                tile,
            }
        })
        .collect();
    // ASM sub_1007A/loc_10200 draws low-priority `unk_E0E4E`/`unk_E0E5E`
    // mappings from live VRAM copy slots `$220`/`$229` while the Level_Layout
    // cell is `$6F00`. Native does not model those scratch copies, so it draws
    // the `$E102` prize source quad that the scratch slot contains.
    commands.push(DrawCommand::Sprite {
        x,
        y,
        tile_sheet: Arc::new(level_visuals.block_tiles.clone()),
        palette: Arc::new(level_visuals.palettes.foreground.clone()),
        tiles,
    });
}

fn push_shifting_block_rise_loc_113f4(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
) -> bool {
    let Some(tile_base_index) =
        SHIFTING_BLOCK_RISE_TILE_LOC_113F4.checked_sub(BLOCK_TILE_VRAM_BASE)
    else {
        return false;
    };
    if usize::from(tile_base_index.saturating_add(3)) >= level_visuals.block_tiles.tiles.len() {
        return false;
    }
    // ASM sub_113C0/loc_113F4 (kid.asm:23332-23339) draws
    // `LnkTo_unk_E0EBE` while the worker rises by 2px for eight frames, then
    // commits `$E710` to the cell above. The temporary `$6F00` reservation
    // does not describe the sprite art.
    let tiles = (0..4)
        .map(|index| SpriteTile {
            offset_x: if index % 2 == 0 { 0 } else { 8 },
            offset_y: if index < 2 { 0 } else { 8 },
            tile: TileRef {
                tile_index: tile_base_index + u16::try_from(index).expect("2x2 tile index fits"),
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        })
        .collect();
    commands.push(DrawCommand::Sprite {
        x,
        y,
        tile_sheet: Arc::new(level_visuals.block_tiles.clone()),
        palette: Arc::new(level_visuals.palettes.foreground.clone()),
        tiles,
    });
    true
}

fn prize_block_spawn_flash_frame_loc_102b8(
    level_visuals: &NativeLevelVisuals,
    remaining: u8,
) -> Option<wildside_render::SpriteFrame> {
    let flash = wildside_core::NativeRuntime::prize_block_spawn_flash_loc_102B8();
    debug_assert!(!flash.display_enabled);
    debug_assert_eq!(flash.palette_line, 0);
    debug_assert_eq!(flash.addroffset_sprite, 0x0F2E);
    debug_assert_eq!(flash.animation_script_addr, 0x0001_02DE);

    let elapsed = PRIZE_BLOCK_SPAWN_FLASH_TOTAL_FRAMES_LOC_102B8.saturating_sub(remaining);
    let mut frame_start = 0_u8;
    // ASM loc_102B8/stru_102DE (kid.asm:21578) initialises four 24x24 sprite
    // frames at block-art VRAM tiles $2CF, $2D8, $2E1, and $2EA.
    for (duration, vram_tile) in PRIZE_BLOCK_SPAWN_FLASH_FRAMES_STRU_102DE {
        let frame_end = frame_start.saturating_add(duration);
        if elapsed < frame_end || frame_end == 24 {
            let tile_base_index = vram_tile.checked_sub(BLOCK_TILE_VRAM_BASE)?;
            if usize::from(tile_base_index.saturating_add(8))
                >= level_visuals.block_tiles.tiles.len()
            {
                return None;
            }
            return Some(wildside_render::SpriteFrame {
                origin_x: 0x0C,
                origin_y: 0x0C,
                width_px: 0x18,
                height_px: 0x18,
                tile_base_index,
                tile_sheet: level_visuals.block_tiles.clone(),
            });
        }
        frame_start = frame_end;
    }
    None
}

fn push_prize_block_spawn_flash_loc_102b8(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    remaining: u8,
) -> bool {
    let Some(frame) = prize_block_spawn_flash_frame_loc_102b8(level_visuals, remaining) else {
        return false;
    };
    push_sprite_frame(commands, &frame, &level_visuals.palettes.foreground, x, y);
    true
}

fn push_kid_projectile_hit_spark_loc_1f08(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    x: i32,
    y: i32,
    remaining: u8,
) {
    let elapsed = 4_u8.saturating_sub(remaining);
    let mut frame_start = 0_u8;
    // ASM loc_1F08/stru_1F40 (kid.asm:3716,3730): four one-frame 24x24
    // mappings from the same `$2CF/$2D8/$2E1/$2EA` block-art VRAM range used
    // by the prize-block flash family, but with palette line 3.
    for (duration, vram_tile) in KID_PROJECTILE_HIT_SPARK_FRAMES_STRU_1F40 {
        let frame_end = frame_start.saturating_add(duration);
        if elapsed < frame_end || frame_end == 4 {
            let Some(tile_base_index) = vram_tile.checked_sub(BLOCK_TILE_VRAM_BASE) else {
                break;
            };
            if usize::from(tile_base_index.saturating_add(8))
                < level_visuals.block_tiles.tiles.len()
            {
                let frame = wildside_render::SpriteFrame {
                    origin_x: 0x0C,
                    origin_y: 0x0C,
                    width_px: 0x18,
                    height_px: 0x18,
                    tile_base_index,
                    tile_sheet: level_visuals.block_tiles.clone(),
                };
                push_sprite_frame(commands, &frame, &level_visuals.palettes.foreground, x, y);
                return;
            }
            break;
        }
        frame_start = frame_end;
    }

    commands.push(DrawCommand::Rect {
        x: x - 3,
        y: y - 3,
        width: 6,
        height: 6,
        colour: [255, 240, 96],
    });
}

pub fn push_level_layout_terrain(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
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
    let foreground_palette = foreground_palette_for_scene(trace, level_visuals);

    for row in row_start..row_end {
        for col in col_start..col_end {
            let live_cell = block_grid.map(|grid| grid.cell_at(col, row));
            let raw_word = block_grid.map(|grid| grid.raw_word_at(col, row));
            let active_prize_bounce =
                block_bounce_kind_for_trace(trace, col, row) == Some(BlockBounceTraceKind::Prize);
            let priority_override = raw_word
                .filter(|word| word & 0x8100 == 0x0100)
                .map(|_| true);
            // ASM sub_302F6/sub_3034A route negative Level_Layout words
            // through Block_Mappings, so static live block cells do not also
            // draw the immutable foreground layout underneath. ASM sub_1007A
            // owns active `$6Fxx` prize-wobble cells with `unk_E0E4E`/
            // `unk_E0E5E`; drawing the foreground skin under that live worker
            // leaks tree/wood pixels through the transparent block-art slots.
            // Non-active erased cells still reveal the separate foreground
            // skin after ASM loc_11542/sub_1007A has finished with them.
            let skin = match live_cell {
                Some(cell) if cell.has_block => continue,
                Some(_cell)
                    if active_prize_bounce
                        && raw_word.is_some_and(|word| word & 0xFF00 == 0x6F00) =>
                {
                    continue;
                }
                Some(cell)
                    if cell.collision != wildside_core::terrain::CollisionKind::Empty
                        || cell.skin != 0 =>
                {
                    cell.skin
                }
                _ => layout.skin_at(col, row),
            };
            let Some(resolved) = level_visuals.level.block_mapping.resolve(skin, None) else {
                continue;
            };
            // ASM sub_302F6/sub_3034A ORs `$8000` into Scroll A tile
            // words when the raw Level_Layout terrain cell carries bit
            // `$0100`; Cell remains the collision-normalised view.
            push_tile_quad_with_priority_override(
                commands,
                level_visuals,
                &foreground_palette,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px,
                resolved.quad,
                priority_override,
            );
        }
    }
}

fn push_special_effect_unknown_foreground_plane_loc_14e42(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    let Some(fill) = &trace.special_effect_unknown_plane_fill_loc_14e42 else {
        return;
    };
    let Some(visuals) = &level_visuals.special_effect_unknown else {
        return;
    };
    if fill.foreground_plane_register != SPECIAL_EFFECT_UNKNOWN_FOREGROUND_PLANE_REGISTER_LOC_14E42
        || fill.group_count_per_pass == 0
        || fill.inner_repeat_count == 0
        || fill.outer_pass_count == 0
    {
        return;
    }

    let plane_width_tiles = fill.inner_repeat_count * 4;
    let plane_height_tiles = fill.outer_pass_count * fill.group_count_per_pass;
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let first_plane_col = camera_x_px.div_euclid(8);
    let first_plane_row = camera_y_px.div_euclid(8);
    let x_offset = camera_x_px.rem_euclid(8);
    let y_offset = camera_y_px.rem_euclid(8);
    let visible_cols = FRAME_WIDTH.div_ceil(8) as usize + 1;
    let visible_rows = FRAME_HEIGHT.div_ceil(8) as usize + 1;
    let tile_sheet = Arc::new(visuals.tile_sheet.clone());
    let palette = Arc::new(foreground_palette_for_scene(trace, level_visuals));

    for screen_row in 0..visible_rows {
        let plane_row = (first_plane_row + screen_row as i32)
            .rem_euclid(i32::try_from(plane_height_tiles).expect("plane height fits i32"))
            as usize;
        let pattern_row = plane_row % fill.group_count_per_pass;
        for screen_col in 0..visible_cols {
            let plane_col = (first_plane_col + screen_col as i32)
                .rem_euclid(i32::try_from(plane_width_tiles).expect("plane width fits i32"))
                as usize;
            // ASM Init_SpecialEffect_Unknown loc_14E42/loc_14E44/loc_14E46:
            // the inner loop writes `d6=$8380_8381` then `d7=$8382_8383`
            // sixteen times per row; the middle loop adds `$0004_0004`
            // for four rows, and the outer loop repeats those four rows.
            let attr_word = 0x8380
                + u16::try_from(pattern_row * 4 + (plane_col % 4))
                    .expect("loc_14E42 tile word offset fits u16");
            let tile = TileRef::from_attr_word(attr_word);
            if tile.tile_index < visuals.vram_base_tile_index {
                continue;
            }
            let local_tile_index = tile.tile_index - visuals.vram_base_tile_index;
            if usize::from(local_tile_index) >= visuals.tile_sheet.tiles.len() {
                continue;
            }
            commands.push(DrawCommand::IndexedTile {
                x: i32::try_from(screen_col).expect("visible col fits i32") * 8 - x_offset,
                y: i32::try_from(screen_row).expect("visible row fits i32") * 8 - y_offset,
                tile_sheet: tile_sheet.clone(),
                tile: TileRef {
                    tile_index: local_tile_index,
                    ..tile
                },
                palette: palette.clone(),
            });
        }
    }
}

fn block_bounce_kind_for_trace(
    trace: &NativeTraceFrame,
    col: u16,
    row: u16,
) -> Option<BlockBounceTraceKind> {
    trace
        .block_bounce_details
        .iter()
        .find(|bounce| bounce.cell_x == col && bounce.cell_y == row && bounce.remaining > 0)
        .map(|bounce| bounce.kind)
        .or_else(|| {
            if !trace.block_bounce_details.is_empty() {
                return None;
            }
            // Legacy recordings only serialised `(col,row,remaining,y)`.
            // They predate typed bounce workers, so keep their historical
            // renderer behaviour and treat active entries as prize wobble.
            trace
                .block_bounces
                .iter()
                .any(|(cx, cy, remaining, _)| *cx == col && *cy == row && *remaining > 0)
                .then_some(BlockBounceTraceKind::Prize)
        })
}

pub fn push_prize_block_wobble_overrides_sub_1007a(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    _block_grid: Option<&BlockGrid>,
) {
    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    for &(col, row, remaining, y_offset) in &trace.block_bounces {
        if remaining == 0 {
            continue;
        }
        if block_bounce_kind_for_trace(trace, col, row) != Some(BlockBounceTraceKind::Prize) {
            continue;
        }
        if remaining == 1 && y_offset == 0 {
            // ASM sub_1007A loc_10120/loc_1026E has reached the spawn/commit
            // hand-off here; oracle frames no longer paint the copied wobble
            // sprite over the settled row cell.
            continue;
        }
        push_prize_block_wobble_sub_1007a(
            commands,
            level_visuals,
            i32::from(col) * 16 - camera_x_px,
            i32::from(row) * 16 - camera_y_px + i32::from(y_offset),
            remaining,
        );
    }
}

pub fn push_block_grid_overrides(
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
    let foreground_palette = foreground_palette_for_scene(trace, level_visuals);

    for row in row_start..row_end {
        for col in col_start..col_end {
            let cell = grid.cell_at(col, row);
            let bounce_y_offset = trace
                .block_bounces
                .iter()
                .find(|(cx, cy, _, _)| *cx == col && *cy == row)
                .map(|(_, _, remaining, y)| (*remaining, i32::from(*y)));
            let bounce_kind = block_bounce_kind_for_trace(trace, col, row);
            let is_prize_bounce = bounce_kind == Some(BlockBounceTraceKind::Prize);
            let is_hidden_reveal_bounce = bounce_kind == Some(BlockBounceTraceKind::HiddenReveal);
            let is_shifting_bounce = bounce_kind == Some(BlockBounceTraceKind::Shifting);
            if !cell.has_block
                && !is_prize_bounce
                && !is_hidden_reveal_bounce
                && !is_shifting_bounce
            {
                continue;
            }
            if !cell.has_block && bounce_y_offset.is_none() {
                continue;
            }
            if is_prize_bounce {
                continue;
            }
            if is_shifting_bounce {
                if let Some((_, y_offset)) = bounce_y_offset {
                    push_shifting_block_rise_loc_113f4(
                        commands,
                        level_visuals,
                        i32::from(col) * 16 - camera_x_px,
                        i32::from(row) * 16 - camera_y_px + y_offset,
                    );
                }
                continue;
            }

            let skin = if cell.has_block {
                cell.skin
            } else {
                // ASM sub_1007A draws the prize wobble object using the
                // original Scroll A block art after the live Level_Layout
                // cell has been rewritten to `$6Fxx`/cleared. Preserve the
                // immutable foreground skin so the shaking block does not
                // flash as skin zero.
                level_visuals.level.foreground_layout.skin_at(col, row)
            };
            let Some(resolved) = level_visuals.level.block_mapping.resolve(
                skin,
                Some(if cell.has_block {
                    cell.block_type as u8
                } else if is_hidden_reveal_bounce {
                    // ASM sub_DFB0 loc_E096 (kid.asm:17413) writes only the
                    // `$60` solid-placeholder byte while loc_E2E2/loc_E2F0
                    // bob the reveal object; block type is not represented
                    // in the temporary cell and native block quads are keyed
                    // by the preserved skin byte.
                    wildside_core::terrain::BlockType::Smash as u8
                } else {
                    // ASM sub_1007A writes `$6Fxx` while the prize-block
                    // wobble object remains live. The block flag is clear,
                    // but the worker still draws the block sprite frames
                    // (`unk_E0E4E`/`unk_E0E5E`) at the bounced position.
                    wildside_core::terrain::BlockType::Prize as u8
                }),
            ) else {
                continue;
            };
            if is_hidden_reveal_bounce && !trace.block_bounce_details.is_empty() {
                // ASM loc_E0C2 (kid.asm:17414): type-$0A hidden blocks draw
                // four quarter-tile reveal workers. Typed trace details carry
                // the worker offsets; legacy compact traces fall through to the
                // single whole-tile draw below.
                for bounce in trace.block_bounce_details.iter().filter(|bounce| {
                    bounce.cell_x == col
                        && bounce.cell_y == row
                        && bounce.remaining > 0
                        && bounce.kind == BlockBounceTraceKind::HiddenReveal
                }) {
                    push_tile_quad(
                        commands,
                        level_visuals,
                        &foreground_palette,
                        i32::from(col) * 16 - camera_x_px + i32::from(bounce.x_offset_px),
                        i32::from(row) * 16 - camera_y_px + i32::from(bounce.y_offset_px),
                        resolved.quad,
                    );
                }
                continue;
            }

            // ASM sub_1007A wobble: if the cell is in `block_bounces`, offset
            // the draw y_pos by the engine-supplied value so the block visibly
            // shakes up/down before settling.
            let y_offset = bounce_y_offset.map_or(0, |(_, y)| y);
            push_tile_quad(
                commands,
                level_visuals,
                &foreground_palette,
                i32::from(col) * 16 - camera_x_px,
                i32::from(row) * 16 - camera_y_px + y_offset,
                resolved.quad,
            );
        }
    }
}

pub fn background_colour(level_visuals: &NativeLevelVisuals) -> [u8; 3] {
    level_visuals
        .palettes
        .background
        .colours
        .first()
        .map_or(SKY_COLOUR, |colour| [colour.red, colour.green, colour.blue])
}

fn push_eol_hologram_room(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    // ASM loc_C186 uploads MapEni_CC0E to Scroll B after the flag-touch
    // hibernate. Scroll A stays live until the loc_C21E/sub_C254 reveal
    // writes replace its foreground cells.
    if let Some(hologram) = &level_visuals.eol_hologram {
        let tile_sheet = Arc::new(hologram.tile_sheet.clone());
        let palette = Arc::new(eol_score_room_palette_loc_d734(trace, &hologram.palette));
        for (row_index, row) in hologram.rows.iter().enumerate() {
            for (col_index, tile) in row.iter().copied().enumerate() {
                commands.push(DrawCommand::IndexedTile {
                    x: (col_index * 8) as i32,
                    y: (row_index * 8) as i32,
                    tile_sheet: Arc::clone(&tile_sheet),
                    tile,
                    palette: Arc::clone(&palette),
                });
            }
        }
        return;
    }

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 0,
        width: FRAME_WIDTH as i32,
        height: FRAME_HEIGHT as i32,
        colour: [0, 0, 0],
    });

    let grid_colour = [255, 112, 0];
    for y in (32..190).step_by(16) {
        commands.push(DrawCommand::Rect {
            x: 0,
            y,
            width: FRAME_WIDTH as i32,
            height: 2,
            colour: grid_colour,
        });
    }
    for x in (0..FRAME_WIDTH as i32).step_by(16) {
        commands.push(DrawCommand::Rect {
            x,
            y: 32,
            width: 2,
            height: 158,
            colour: grid_colour,
        });
    }

    let stone_dark = [64, 72, 96];
    let stone_light = [144, 152, 184];
    for x in [176, 224] {
        commands.push(DrawCommand::Rect {
            x,
            y: 0,
            width: 16,
            height: 152,
            colour: stone_dark,
        });
        for y in (0..152).step_by(16) {
            commands.push(DrawCommand::Rect {
                x,
                y,
                width: 16,
                height: 2,
                colour: stone_light,
            });
            commands.push(DrawCommand::Rect {
                x,
                y,
                width: 2,
                height: 16,
                colour: stone_light,
            });
        }
    }

    commands.push(DrawCommand::Rect {
        x: 0,
        y: 152,
        width: FRAME_WIDTH as i32,
        height: 72,
        colour: [111, 70, 40],
    });
    commands.push(DrawCommand::Rect {
        x: 0,
        y: 144,
        width: FRAME_WIDTH as i32,
        height: 12,
        colour: [30, 168, 72],
    });
}

#[allow(clippy::too_many_arguments)]
fn push_eol_hologram_tile_clipped(
    commands: &mut Vec<DrawCommand>,
    hologram: &wildside_render::EolHologramVisuals,
    tile_x: i32,
    tile_y: i32,
    tile_col: usize,
    tile_row: usize,
    clip_x: i32,
    clip_y: i32,
    clip_width: i32,
    clip_height: i32,
    tile_sheet: &Arc<TileSheet>,
    palette: &Arc<Palette>,
) {
    let Some(row) = hologram.rows.get(tile_row) else {
        return;
    };
    let Some(tile) = row.get(tile_col).copied() else {
        return;
    };
    let clipped_left = tile_x.max(clip_x);
    let clipped_top = tile_y.max(clip_y);
    let clipped_right = (tile_x + 8).min(clip_x + clip_width);
    let clipped_bottom = (tile_y + 8).min(clip_y + clip_height);
    if clipped_left >= clipped_right || clipped_top >= clipped_bottom {
        return;
    }

    let start_column = u8::try_from(clipped_left - tile_x).expect("clipped column fits");
    let width = u8::try_from(clipped_right - clipped_left).expect("clipped width fits");
    for screen_y in clipped_top..clipped_bottom {
        commands.push(DrawCommand::IndexedTileSlice {
            x: tile_x,
            y: tile_y,
            tile_sheet: Arc::clone(tile_sheet),
            tile,
            palette: Arc::clone(palette),
            row: u8::try_from(screen_y - tile_y).expect("clipped row fits"),
            start_column,
            width,
        });
    }
}

fn push_eol_hologram_tiles_for_screen_rect(
    commands: &mut Vec<DrawCommand>,
    hologram: &wildside_render::EolHologramVisuals,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    tile_sheet: &Arc<TileSheet>,
    palette: &Arc<Palette>,
) {
    let start_col = x.div_euclid(8).max(0);
    let end_col = (x + width - 1)
        .div_euclid(8)
        .min((FRAME_WIDTH as i32 - 1) / 8);
    let start_row = y.div_euclid(8).max(0);
    let end_row = (y + height - 1)
        .div_euclid(8)
        .min((FRAME_HEIGHT as i32 - 1) / 8);
    if start_col > end_col || start_row > end_row {
        return;
    }

    for tile_row in start_row..=end_row {
        for tile_col in start_col..=end_col {
            // ASM loc_C21E/sub_C254 updates a clipped 16x16 Scroll A cell. The
            // Scroll B hologram stays on whole screen tiles underneath, but the
            // redraw must be clipped back to that foreground cell; otherwise
            // sub-tile-aligned cells leak a row/column into unrevealed Plane A.
            push_eol_hologram_tile_clipped(
                commands,
                hologram,
                tile_col * 8,
                tile_row * 8,
                usize::try_from(tile_col).expect("screen tile column fits"),
                usize::try_from(tile_row).expect("screen tile row fits"),
                x,
                y,
                width,
                height,
                tile_sheet,
                palette,
            );
        }
    }
}

fn push_eol_foreground_reveal_tiles(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) {
    let (Some(hologram), Some(grid)) = (&level_visuals.eol_hologram, block_grid) else {
        return;
    };
    let reveal_elapsed = trace.level_complete_elapsed_frames.saturating_sub(
        wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
            + wildside_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE,
    );
    if reveal_elapsed == 0 {
        return;
    }

    let camera_x_px = (trace.camera_x_pos >> 16) as i32;
    let camera_y_px = (trace.camera_y_pos >> 16) as i32;
    let tile_sheet = Arc::new(hologram.tile_sheet.clone());
    let palette = Arc::new(eol_score_room_palette_loc_d734(trace, &hologram.palette));
    let writes = wildside_core::flagpole_scoreboard_reveal_tiles_loc_C21E(
        grid,
        (trace.camera_x_pos >> 16) as u16,
        (trace.camera_y_pos >> 16) as u16,
        reveal_elapsed,
    );
    for write in &writes {
        let x = i32::from(write.col) * 16 - camera_x_px;
        let y = i32::from(write.row) * 16 - camera_y_px;
        commands.push(DrawCommand::Rect {
            x,
            y,
            width: 16,
            height: 16,
            colour: [0, 0, 0],
        });
        // ASM loc_C186 uploads the hologram as a Scroll B screen map. Redraw the
        // visible screen-space tiles under the loc_C21E/sub_C254 foreground write.
        // The foreground write is camera sub-tile aligned, but Scroll B is fixed
        // to screen tile coordinates after Background_NoScrollFlag is set at
        // loc_C0CC, so the replacement must stay on 8px screen boundaries.
        push_eol_hologram_tiles_for_screen_rect(
            commands,
            hologram,
            x,
            y,
            16,
            16,
            &tile_sheet,
            &palette,
        );
    }

    for write in &writes {
        let x = i32::from(write.col) * 16 - camera_x_px;
        let y = i32::from(write.row) * 16 - camera_y_px;
        // ASM loc_C21E/sub_C254 writes replacement Scroll A cells after the
        // loc_C186 Scroll B hologram upload. Native redraws Scroll B in a first
        // pass, then clears the local priority mask before drawing Scroll A
        // replacements so later EOL sprites can still appear above them.
        commands.push(DrawCommand::ClearPriorityRect {
            x,
            y,
            width: 16,
            height: 16,
        });
        for (index, word) in write.tile_words.iter().copied().enumerate() {
            commands.push(DrawCommand::IndexedTile {
                x: x + i32::try_from(index % 2).expect("tile column fits") * 8,
                y: y + i32::try_from(index / 2).expect("tile row fits") * 8,
                tile_sheet: Arc::clone(&tile_sheet),
                tile: TileRef::from_attr_word(word),
                palette: Arc::clone(&palette),
            });
        }
    }
}

pub fn object_rect(
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

pub fn screen_coord(world: u32, camera: u32) -> i32 {
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
    let eol_hologram_visible = trace.level_complete
        && trace.level_complete_elapsed_frames
            >= wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE;
    let background = if eol_hologram_visible {
        [0, 0, 0]
    } else {
        background_colour(level_visuals)
    };
    let mut commands = Vec::new();

    if eol_hologram_visible && filter.background {
        push_eol_hologram_room(&mut commands, trace, level_visuals);
    } else if filter.background
        && (!trace.level_complete || trace.level_complete_elapsed_frames == 0)
    {
        // ASM CharacterCollision_TouchFlagpole loc_C070 starts the flagpole
        // hand-off, but the oracle frame at native `level_complete_elapsed=0`
        // still contains the previous Scroll B frame. Suppress ordinary Scroll
        // B during the later pre-hologram hibernate frames.
        if !push_storm_pattern_plane_loc_14d32(&mut commands, trace, level_visuals) {
            push_background_tiles(&mut commands, trace, level_visuals);
        }
    }
    if filter.terrain {
        push_level_layout_terrain(&mut commands, trace, level_visuals, block_grid);
    }
    if filter.block_overrides {
        push_block_grid_overrides(&mut commands, trace, level_visuals, block_grid);
    }
    if filter.terrain {
        push_special_effect_unknown_foreground_plane_loc_14e42(&mut commands, trace, level_visuals);
    }
    if eol_hologram_visible && filter.terrain {
        push_eol_foreground_reveal_tiles(&mut commands, trace, level_visuals, block_grid);
    }
    if filter.block_overrides {
        push_prize_block_wobble_overrides_sub_1007a(
            &mut commands,
            trace,
            level_visuals,
            block_grid,
        );
    }

    if !eol_hologram_visible && filter.platforms {
        for object in trace.active_objects.iter().filter(|object| {
            matches!(
                object.kind,
                wildside_core::NativeObjectKind::Platform { .. }
            )
        }) {
            if let NativeObjectKind::Platform { kind, .. } = object.kind {
                push_platform_tiles(&mut commands, object, trace, level_visuals, kind);
            }
        }
    }
    if !eol_hologram_visible && filter.enemies {
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, wildside_core::NativeObjectKind::Enemy { .. }))
        {
            if let NativeObjectKind::Enemy { object_type } = object.kind {
                if object.debug_fields.contains_key("glyph") {
                    // #349: ASM Load_TitleCard Title_Letter workers ride
                    // active_objects as Enemy{object_type:0} during the
                    // (death-)respawn title card; they render through the
                    // title-card glyph overlay below, not as an enemy family.
                    continue;
                }
                if object_type == DRIPS_OBJECT_TYPE {
                    // ASM Drips.asm Enemy16_Drip_Init/loc_33488 keeps the
                    // parent/controller gfx object hidden via `$13`; native
                    // renders only the falling child and hostile surface
                    // crawler through typed projectile snapshots.
                    continue;
                }
                if object_type == BOSS_EYE_OBJECT_TYPE
                    && boss_eye_object_mapping_index_loc_37abc(object).is_some()
                {
                    continue;
                }
                push_enemy_sprite(&mut commands, object, trace, level_visuals, object_type);
                // Debug: show enemy HP above each enemy
                if debug_overlay_visible && let Some(hp) = object.hit_points {
                    let ex = (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
                    let ey = (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32 - 12;
                    commands.push(text(ex - 4, ey, &format!("{hp}"), [255, 255, 100], 1));
                }
            }
        }
        let mut waiting_boss_eye_objects = trace
            .active_objects
            .iter()
            .filter(|object| {
                matches!(
                    object.kind,
                    NativeObjectKind::Enemy {
                        object_type: BOSS_EYE_OBJECT_TYPE
                    }
                ) && boss_eye_object_mapping_index_loc_37abc(object).is_some()
            })
            .collect::<Vec<_>>();
        waiting_boss_eye_objects.sort_by_key(|object| boss_eye_object_order_word_loc_393c8(object));
        for object in waiting_boss_eye_objects {
            let sx = (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let sy = (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            // ASM loc_3931A allocates the waiting eyes as `$21` objects;
            // sub_37ABC refreshes their sprite mapping before sub_FF6 sorts
            // them by `$A(a3)`.
            sprites::push_boss_eye_attack_marker_sub_37abc(
                &mut commands,
                sx,
                sy,
                boss_eye_object_mapping_index_loc_37abc(object).unwrap_or_default(),
            );
        }
    }
    if !eol_hologram_visible && filter.player {
        // ASM `loc_BC34` damage-flash thread (kid.asm:15181-15215): during
        // the 66-frame `Just_received_damage` window, the ROM alternates
        // between the white palette (sub_80BC) for 2 frames and the normal
        // palette (sub_80D0) for 4 frames, repeating 11 times. Kid stays
        // visible the whole time.
        for object in trace
            .active_objects
            .iter()
            .filter(|object| matches!(object.kind, wildside_core::NativeObjectKind::Player))
        {
            // ASM loc_B7FA/loc_B80C: the death tumble is driven by the
            // engine's `advance_death_windup_motion` (combat.rs:8801).
            // The renderer must NOT add its own offset on top — that was
            // user-reported playtest bug #148 (incorrect dying animation).
            let damage_flash_white_phase = player_damage_flash_white_phase_loc_bc56(trace);
            let priority_override =
                if trace.level_complete && trace.level_complete_elapsed_frames == 0 {
                    // ASM CharacterCollision_TouchFlagpole loc_C070 keeps the
                    // current Kid gfx object while the first hand-off frame still
                    // presents the old level/flag sprites. Native terrain priority
                    // can otherwise mask Kid on that preserved frame.
                    Some(true)
                } else {
                    None
                };
            push_player_sprite_with_priority_override(
                &mut commands,
                object,
                trace,
                level_visuals,
                damage_flash_white_phase,
                priority_override,
            );
        }
    }

    if !eol_hologram_visible && filter.effects {
        if (!trace.level_complete
            || trace.level_complete_elapsed_frames
                < FLAGPOLE_SPRITES_RENDER_CLEANUP_FRAME_LOC_C0B6_ORACLE)
            && let (Some(flag_x), Some(flag_y)) = (trace.flag_x, trace.flag_y)
        {
            // ASM Flagpole_Boss (kid.asm:15592) allocates exactly two
            // GfxObjects at the same anchor: `LnkTo_unk_E0FDE` (pole_bottom,
            // `sprite_frame_vram $696,$08,$10,$10,$10`) and `LnkTo_unk_E0FFE`
            // (waving flag, `sprite_frame_vram $69A,$06,$00,$28,$20`).
            // Both allocations set `priority(a3)=1`, so the sprites draw over
            // high-priority foreground cells at the hand-off.
            // The ROM ships no third "pole shaft" sprite asset — the only
            // flag-related extracted art is `End_of_level_Flag_(6_frames).bin`
            // and `Flag_bottom.bin`. CharacterCollision_TouchFlagpole loc_C070
            // hibernates `$24` frames before loc_C0B6 deletes non-Kid gfx
            // slots; the renderer uses the oracle-visible pre-frame sprite
            // table boundary, one frame before the trace count drops.
            // The previous 2×32 grey rect above the anchor invented a tall pole
            // extension that the ROM never draws.
            let sx = i32::from(flag_x) - (trace.camera_x_pos >> 16) as i32;
            let pole_base_y = i32::from(flag_y) - (trace.camera_y_pos >> 16) as i32;
            let frame_index = (trace.frame_index / level_visuals.flag.frame_duration)
                % level_visuals.flag.waving.len().max(1);
            if let Some(frame) = level_visuals.flag.waving.get(frame_index) {
                push_sprite_frame_with_priority(
                    &mut commands,
                    frame,
                    &level_visuals.flag.palette,
                    sx,
                    pole_base_y,
                    Some(true),
                );
            }
            push_sprite_frame_with_priority(
                &mut commands,
                &level_visuals.flag.pole_bottom,
                &level_visuals.flag.palette,
                sx,
                pole_base_y,
                Some(true),
            );
        }

        // ASM `sub_5E02` (kid.asm:5840-5965) emits the visible murder-wall
        // warning strip as 24 hardware sprites using `Murder_wall.bin`, which
        // Murderwall loc_140D2 uploads to VRAM tile `$2FB`.
        if let Some(wall_x_fp) = trace.murder_wall_x {
            // NativeTraceFrame does not yet expose `$FFFFFAC0/$FFFFFABF`;
            // this keeps the ASM sprite geometry and four ROM tile pages,
            // with the phase derived from frame time for native-player
            // presentation only.
            let animation_frame = ((trace.frame_index / 5) & 3) as u8;
            let render = wildside_core::compile_murder_wall_warning_sprites_sub_5E02(
                wildside_core::MurderWallWarningInputSub5E02 {
                    murder_wall_enabled: true,
                    murder_wall_x_px: (wall_x_fp >> 16) as i16,
                    murder_wall_reversed: level_visuals.level.murder_wall_flags & 1 != 0,
                    kid_hitbox_left_px: 0,
                    kid_hitbox_right_px: 0,
                    camera_x_px: (trace.camera_x_pos >> 16) as i16,
                    camera_y_px: (trace.camera_y_pos >> 16) as u16,
                    animation_timer: 2,
                    animation_frame,
                    starting_number_sprites: 0,
                },
            );
            if let Some(murder_wall_visuals) = &level_visuals.murder_wall {
                for sprite in render.sprites {
                    push_murder_wall_warning_sprite_sub_5e02(
                        &mut commands,
                        murder_wall_visuals,
                        sprite,
                    );
                }
            } else if let (Some(min_x), Some(max_x)) = (
                render
                    .sprites
                    .iter()
                    .map(|sprite| i32::from(sprite.x) - 0x80)
                    .min(),
                render
                    .sprites
                    .iter()
                    .map(|sprite| i32::from(sprite.x) - 0x80 + 24)
                    .max(),
            ) {
                push_murder_wall_warning_fallback_sub_5e02(&mut commands, min_x, max_x);
            }
        }

        // ASM `sub_14BD8 -> loc_14BE8` (kid.asm:28255-28278) emits lava
        // geyser sprite chunks from 8-byte template entries. Traced
        // `loc_147BA` script rows use their exact template labels; the
        // full-height loop consumes traced `loc_14874` `d3`/`$4C` state when
        // present, falling back to the legacy height-derived row plan for old
        // recordings.
        if let Some(lava_visuals) = &level_visuals.lava {
            let lava_palette = lava_geyser_palette_for_scene_loc_14b2a(trace, lava_visuals);
            for template in &trace.lava_geyser_script_templates_loc_147ba {
                push_lava_geyser_script_template_loc_147ba(
                    &mut commands,
                    lava_visuals,
                    &lava_palette,
                    template,
                    (trace.camera_x_pos >> 16) as u16,
                    (trace.camera_y_pos >> 16) as u16,
                );
            }
            for &(gx, gy_top, height, sprite_row_index, top_sprite_frame_within_row) in
                &trace.active_geysers
            {
                let base_y = gy_top.saturating_add(height);
                if trace
                    .lava_geyser_script_templates_loc_147ba
                    .iter()
                    .any(|template| template.x_px == gx && template.base_y_px == base_y)
                {
                    continue;
                }
                push_lava_geyser_rom_sprites_loc_14be8(
                    &mut commands,
                    lava_visuals,
                    &lava_palette,
                    gx,
                    gy_top,
                    height,
                    sprite_row_index,
                    top_sprite_frame_within_row,
                    (trace.camera_x_pos >> 16) as u16,
                    (trace.camera_y_pos >> 16) as u16,
                );
            }
        } else {
            for &(gx, gy_top, height, _, _) in &trace.active_geysers {
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
        }

        // ASM sub_61CA loc_63C4/loc_6462 emits storm weather sprites with
        // VDP-biased screen coordinates and absolute tile words rooted at the
        // Init_SpecialEffect_Storm loc_14CCE/loc_14CD8 `$7000` art upload
        // plus loc_14D8A shared `$76A0` storm art when the trace exposes it.
        if let Some(weather_visuals) = &level_visuals.weather {
            for &sprite in &trace.storm_sprites_sub_61ca {
                let _ = push_storm_weather_sprite_sub_61ca(
                    &mut commands,
                    trace,
                    weather_visuals,
                    sprite,
                );
            }
        }

        // Projectiles: generic shots stay coloured dots; shooter-block
        // bullets keep the ROM's 1x1-sprite footprint visibly distinct; the
        // Fire Demon death trail renders ROM sprite frames via
        // `push_fire_demon_trail_sprite` (ASM Fire_Demon `loc_3A63C` cycling
        // `stru_3A622` -> `unk_C7776..unk_C779E`). Missing-asset states fall
        // back to a single placeholder rect in `push_fire_demon_trail`.
        let mut projectile_details: Vec<ActiveProjectileSnapshot> =
            if trace.active_projectile_details.is_empty() {
                trace
                    .active_projectiles
                    .iter()
                    .map(|&(x_pos, y_pos, friendly)| ActiveProjectileSnapshot {
                        x_pos,
                        y_pos,
                        x_velocity: 0,
                        y_velocity: 0,
                        friendly,
                        remaining_frames: 0,
                        animation_elapsed_frames: 0,
                        kind: ActiveProjectileKind::Generic,
                        palette_variant: 0,
                        boss_eye_sprite_mapping_index: 0,
                        boss_eye_sprite_order_word_a: 0,
                        dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
                        dragon_flying_descent_fire_counter: 0,
                    })
                    .collect()
            } else {
                trace.active_projectile_details.clone()
            };
        order_boss_eye_satellites_sub_ff6(&mut projectile_details);
        for projectile in projectile_details {
            if projectile.kind == ActiveProjectileKind::BossEyeSatellite
                && trace_has_boss_eye_object_slot_loc_3931a(
                    trace,
                    projectile.x_pos,
                    projectile.y_pos,
                )
            {
                continue;
            }
            let sx = (projectile.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let sy = (projectile.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            match projectile.kind {
                ActiveProjectileKind::Generic | ActiveProjectileKind::ArcherArrow => {
                    commands.push(DrawCommand::Rect {
                        x: sx - 2,
                        y: sy - 2,
                        width: 4,
                        height: 4,
                        colour: if projectile.friendly {
                            FRIENDLY_PROJECTILE_COLOUR
                        } else {
                            ENEMY_PROJECTILE_COLOUR
                        },
                    });
                }
                ActiveProjectileKind::LionProjectile => {
                    if !push_lion_projectile_sprite_loc_3e12a(
                        &mut commands,
                        level_visuals,
                        &projectile,
                        sx,
                        sy,
                    ) {
                        commands.push(DrawCommand::Rect {
                            x: sx - 3,
                            y: sy - 3,
                            width: 6,
                            height: 6,
                            colour: ENEMY_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::ManiaxeAxe => {
                    let pushed_sprite = level_visuals
                        .player
                        .forms
                        .get(&PlayerForm::Maniaxe.helmet_id())
                        .and_then(|maniaxe_visuals| {
                            if maniaxe_visuals.axe.is_empty() {
                                return None;
                            }
                            let frame_index = usize::from(projectile.animation_elapsed_frames / 2)
                                % maniaxe_visuals.axe.len();
                            let frame = &maniaxe_visuals.axe[frame_index];
                            let facing = if projectile.x_velocity < 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            };
                            let (draw_x, draw_y) = sprite_draw_position(frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(maniaxe_visuals.palette.clone()),
                                tiles: build_sprite_tiles(frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 2,
                            y: sy - 2,
                            width: 4,
                            height: 4,
                            colour: FRIENDLY_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::JuggernautSkull => {
                    let pushed_sprite = level_visuals
                        .juggernaut_skull
                        .as_ref()
                        .and_then(|visuals| {
                            let frame_index = usize::from(projectile.animation_elapsed_frames / 7)
                                % visuals.frames.len().max(1);
                            let frame = visuals.frames.get(frame_index)?;
                            let facing = if projectile.x_velocity < 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            };
                            let (draw_x, draw_y) = sprite_draw_position(frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(visuals.palette.clone()),
                                tiles: build_sprite_tiles(frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 2,
                            y: sy - 2,
                            width: 4,
                            height: 4,
                            colour: FRIENDLY_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::BossHeadArrow => {
                    // ASM loc_3A0D2 uses boss projectile sprite `stru_3A1CC`;
                    // until the exact C8510/C8518 child frame is compiled here,
                    // keep a thin horizontal footprint matching the live arrow
                    // path instead of a generic dot.
                    commands.push(DrawCommand::Rect {
                        x: sx - 5,
                        y: sy - 1,
                        width: 10,
                        height: 2,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::BossHeadBoomerang => {
                    // ASM loc_3A1EA uses boss projectile sprite `stru_3A308`;
                    // render a compact wide marker until the exact C8530 frame
                    // is compiled into native play.
                    commands.push(DrawCommand::Rect {
                        x: sx - 4,
                        y: sy - 2,
                        width: 8,
                        height: 4,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::BossSkewer => {
                    // ASM loc_384E8..loc_388D8 uses the Shishkaboss skewer
                    // child sprites; render a tall marker until those exact
                    // C85E8/C85F0/C85F8 frames are compiled into native play.
                    commands.push(DrawCommand::Rect {
                        x: sx - 2,
                        y: sy - 8,
                        width: 4,
                        height: 16,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::BossDeathBurst => {
                    // ASM sub_370D2 uses `LnkTo_unk_C8460` and `stru_37BD8`;
                    // retain a distinct 8x8 footprint until that exact boss
                    // burst frame is compiled into native play.
                    commands.push(DrawCommand::Rect {
                        x: sx - 4,
                        y: sy - 4,
                        width: 8,
                        height: 8,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::UfoFirstHurtDebris => {
                    // ASM UFO.asm loc_354DA links debris children to sprite mapping LnkTo_unk_C8408.
                    commands.push(DrawCommand::Rect {
                        x: sx - 2,
                        y: sy - 2,
                        width: 4,
                        height: 4,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::UfoBeam => {
                    // ASM UFO.asm Enemy0F_UFOBeam_Init uses LnkTo_unk_C83A0.
                    // TODO(fidelity): compile that exact mapping; meanwhile
                    // keep its narrow laser footprint distinct from the
                    // stationary walking-Dragon breath rendering.
                    commands.push(DrawCommand::Rect {
                        x: sx - 2,
                        y: sy - 4,
                        width: 4,
                        height: 8,
                        colour: ENEMY_PROJECTILE_COLOUR,
                    });
                }
                ActiveProjectileKind::BossEyeAttack | ActiveProjectileKind::BossEyeSatellite => {
                    sprites::push_boss_eye_attack_marker_sub_37abc(
                        &mut commands,
                        sx,
                        sy,
                        projectile.boss_eye_sprite_mapping_index,
                    );
                }
                ActiveProjectileKind::ShooterBlock => {
                    if !push_shooter_block_projectile_sprite_loc_ef62(
                        &mut commands,
                        level_visuals,
                        &projectile,
                        sx,
                        sy,
                    ) {
                        // ASM Process_ShooterObjects_MakeSprite (kid.asm:19475):
                        // shooter slots emit a hostile 1x1 sprite. If the ROM
                        // block-art tile is unavailable, retain an explicit
                        // 8x8 footprint instead of the tiny generic dot.
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 4,
                            width: 8,
                            height: 8,
                            colour: SHOOTER_BLOCK_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::TankBullet => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&0x0B)
                        .or_else(|| level_visuals.enemies.families.get(&0x0A))
                        .and_then(|tank_visuals| {
                            let frame =
                                tank_bullet_child_frame(tank_visuals, projectile.remaining_frames)?;
                            let facing = if projectile.x_velocity < 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            };
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(tank_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 3,
                            width: 8,
                            height: 5,
                            colour: ENEMY_PROJECTILE_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::RobotShot | ActiveProjectileKind::RobotShotImpact => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&ROBOT_OBJECT_TYPE)
                        .and_then(|robot_visuals| {
                            let impact = projectile.kind == ActiveProjectileKind::RobotShotImpact;
                            let frame = robot_shot_child_frame(
                                robot_visuals,
                                projectile.remaining_frames,
                                projectile.animation_elapsed_frames,
                                impact,
                            )?;
                            let facing = if !impact && projectile.x_velocity > 0 {
                                Direction::Right
                            } else {
                                Direction::Left
                            };
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(robot_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 3,
                            width: 8,
                            height: 6,
                            colour: ENEMY_PROJECTILE_COLOUR,
                        });
                    }
                }
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
                ActiveProjectileKind::FireballTrail => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&0x19)
                        .and_then(|fireball_visuals| {
                            let frame = fireball_trail_child_frame(
                                fireball_visuals,
                                projectile.remaining_frames,
                            )?;
                            push_sprite_frame(
                                &mut commands,
                                &frame,
                                &fireball_visuals.palette,
                                sx,
                                sy,
                            );
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 5,
                            width: 8,
                            height: 8,
                            colour: FIRE_TRAIL_PLACEHOLDER_COLOUR,
                        });
                    }
                }
                ActiveProjectileKind::CloudLightning => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&CLOUD_OBJECT_TYPE)
                        .and_then(|cloud_visuals| {
                            let frame =
                                cloud_lightning_child_frame_loc_35df0(cloud_visuals, &projectile)?;
                            let facing = if projectile.x_velocity < 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            };
                            let sx = if projectile.x_velocity != 0 && projectile.y_velocity != 0 {
                                // ASM Cloud.asm loc_35DF0 nudges the diagonal
                                // bolt by +4/-4 px after selecting C7876.
                                sx + if matches!(facing, Direction::Left) {
                                    -4
                                } else {
                                    4
                                }
                            } else {
                                sx
                            };
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(cloud_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 4,
                            y: sy - 4,
                            width: 8,
                            height: 8,
                            colour: [190, 220, 250],
                        });
                    }
                }
                ActiveProjectileKind::DragonWalkingBreath => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&WALKING_DRAGON_OBJECT_TYPE)
                        .and_then(|dragon_visuals| {
                            let frame = dragon_walking_breath_child_frame(
                                dragon_visuals,
                                projectile.remaining_frames,
                            )?;
                            let facing =
                                dragon_walking_breath_child_facing_loc_3d02a(trace, &projectile);
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(dragon_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 2,
                            y: sy - 3,
                            width: 4,
                            height: 5,
                            colour: [255, 90, 20],
                        });
                    }
                }
                ActiveProjectileKind::DragonFlyingDescentFire => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&FLYING_DRAGON_OBJECT_TYPE)
                        .and_then(|dragon_visuals| {
                            let frame = dragon_flying_descent_fire_child_frame(
                                dragon_visuals,
                                &projectile,
                            )?;
                            push_sprite_frame(
                                &mut commands,
                                &frame,
                                &dragon_visuals.palette,
                                sx,
                                sy,
                            );
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        commands.push(DrawCommand::Rect {
                            x: sx - 3,
                            y: sy - 3,
                            width: 6,
                            height: 6,
                            colour: [255, 110, 30],
                        });
                    }
                }
                ActiveProjectileKind::DripsDrop
                | ActiveProjectileKind::DripsSurfaceCrawler
                | ActiveProjectileKind::DripsSurfaceCrawlerFinish => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&DRIPS_OBJECT_TYPE)
                        .and_then(|drips_visuals| {
                            let frame = match projectile.kind {
                                ActiveProjectileKind::DripsDrop => {
                                    drips_drop_child_frame(drips_visuals)?
                                }
                                ActiveProjectileKind::DripsSurfaceCrawler => {
                                    drips_surface_crawler_frame(
                                        drips_visuals,
                                        projectile.remaining_frames,
                                        projectile.animation_elapsed_frames,
                                    )?
                                }
                                ActiveProjectileKind::DripsSurfaceCrawlerFinish => {
                                    drips_surface_crawler_finish_frame(
                                        drips_visuals,
                                        projectile.remaining_frames,
                                    )?
                                }
                                _ => return None,
                            };
                            push_sprite_frame(
                                &mut commands,
                                &frame,
                                sprites::select_enemy_palette(
                                    drips_visuals,
                                    projectile.palette_variant,
                                ),
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
                ActiveProjectileKind::TarMonsterGlob
                | ActiveProjectileKind::TarMonsterBurstChild
                | ActiveProjectileKind::DripsSurfaceCrawlerBurst => {
                    let pushed_sprite = level_visuals
                        .enemies
                        .families
                        .get(&TAR_MONSTER_OBJECT_TYPE)
                        .and_then(|tar_visuals| {
                            let frame = match projectile.kind {
                                ActiveProjectileKind::TarMonsterGlob => {
                                    tar_monster_glob_child_frame(tar_visuals)?
                                }
                                ActiveProjectileKind::TarMonsterBurstChild => {
                                    tar_monster_burst_child_frame(tar_visuals)?
                                }
                                ActiveProjectileKind::DripsSurfaceCrawlerBurst => {
                                    tar_monster_burst_child_frame(tar_visuals)?
                                }
                                _ => return None,
                            };
                            // ASM Tar_Monster.asm loc_324D0/loc_324DC mirrors the
                            // glob when `x_direction != 0`, so face the sprite by
                            // travel direction. Burst fragments scatter both ways.
                            let facing = if projectile.x_velocity < 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            };
                            let (draw_x, draw_y) = sprite_draw_position(&frame, sx, sy, facing);
                            commands.push(DrawCommand::Sprite {
                                x: draw_x,
                                y: draw_y,
                                tile_sheet: Arc::new(frame.tile_sheet.clone()),
                                palette: Arc::new(tar_visuals.palette.clone()),
                                tiles: build_sprite_tiles(&frame, facing, 0),
                            });
                            Some(())
                        })
                        .is_some();
                    if !pushed_sprite {
                        // Asset-missing fallback: keep a dark tar footprint rather
                        // than reverting to the generic red enemy-projectile dot.
                        commands.push(DrawCommand::Rect {
                            x: sx - 3,
                            y: sy - 3,
                            width: 6,
                            height: 6,
                            colour: [40, 50, 80],
                        });
                    }
                }
            }
        }

        for sprite in &trace.diamond_power_sprites {
            let _ = push_diamond_power_sprite_loc_3f880(&mut commands, level_visuals, *sprite);
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

        if let Some(tile) = resolve_ingame_vram_tile(
            level_visuals,
            TileRef::from_attr_word(BLOCK_DEBRIS_TILE_ATTR_SUB_2A4C),
        ) {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            for &(x_px, y_px) in &trace.block_debris_pieces {
                // ASM sub_2A4C (kid.asm:5053) emits persistent falling debris
                // sprite entries with tile `$8228` after sub_292E seeds the
                // scratch slot.
                push_resolved_tile(
                    &mut commands,
                    i32::from(x_px) - camera_x_px,
                    i32::from(y_px) - camera_y_px,
                    tile.clone(),
                );
            }
        }

        for &(x_px, y_px, remaining) in &trace.prize_block_spawn_flashes {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            let screen_x = i32::from(x_px) - camera_x_px;
            let screen_y = i32::from(y_px) - camera_y_px;
            push_prize_block_spawn_flash_loc_102b8(
                &mut commands,
                level_visuals,
                screen_x,
                screen_y,
                remaining,
            );
        }

        for &(x_px, y_px, remaining) in &trace.kid_projectile_hit_sparks {
            let camera_x_px = (trace.camera_x_pos >> 16) as i32;
            let camera_y_px = (trace.camera_y_pos >> 16) as i32;
            let screen_x = i32::from(x_px) - camera_x_px;
            let screen_y = i32::from(y_px) - camera_y_px;
            push_kid_projectile_hit_spark_loc_1f08(
                &mut commands,
                level_visuals,
                screen_x,
                screen_y,
                remaining,
            );
        }

        // Floating prize items: small coloured shape rising upward
        for &(x_px, y_px, remaining, content_id) in &trace.floating_items {
            let fx = i32::from(x_px) - (trace.camera_x_pos >> 16) as i32;
            let fy = i32::from(y_px) - (trace.camera_y_pos >> 16) as i32;
            let content = wildside_core::terrain::PrizeContent::from_id(content_id);
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
                if !floating_prize_sprite_visible_loc_103b2(content, remaining) {
                    continue;
                }
                let frame_index = floating_prize_frame_index_loc_10396(
                    content,
                    remaining,
                    trace.frame_index,
                    prize_visuals.frame_duration.max(1),
                    prize_visuals.frames.len().max(1),
                );
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

        if trace.heady_metal_death_splits.is_empty() {
            // Enemy death effect: only HeadyMetal spawns one. ASM
            // Enemy20_HeadyMetal_Init seeds the $01C2-frame wobble worker
            // (sub_37708/sub_3787C/sub_378A2); `Jump_On_Enemy` (kid.asm:14607)
            // plays sfx_Jump_on_enemy and deletes the GfxObject without
            // emitting any per-stomp death sprite.
            for &(ex, ey, remaining) in &trace.enemy_death_effects {
                push_heady_metal_death_wobble(
                    &mut commands,
                    trace,
                    level_visuals,
                    ex,
                    ey,
                    remaining,
                );
            }
        } else {
            push_heady_metal_death_split_pieces(&mut commands, trace, level_visuals);
        }
    }

    if !eol_hologram_visible && filter.hud {
        // ASM Mode_Level calls Make_SpriteAttr_HUD before sub_5E02 but the
        // HUD occupies the fixed top overlay in native-player. Clear native's
        // local high-priority mask so later murder-wall/weather sprites cannot
        // occlude the timer/lives/diamond HUD row.
        commands.push(DrawCommand::ClearPriorityRect {
            x: 0,
            y: 0,
            width: FRAME_WIDTH as i32,
            height: 48,
        });
        push_game_hud(&mut commands, trace, level_visuals);
    }

    // Level Complete overlay. ASM `loc_D55E` onwards (kid.asm:16670-16800)
    // emits sprite-art labels — `LnkTo_Spr_EOL_Score`,
    // `LnkTo_Spr_EOL_Time`, `LnkTo_Spr_EOL_Bonus`, `LnkTo_Spr_EOL_NoHit`,
    // `LnkTo_Spr_EOL_NoPrize`, `LnkTo_Spr_EOL_Path`, `LnkTo_Spr_EOL_Speed`
    // — plus seven digit workers from `Load_Object_EOL_Number`. Native uses
    // the trace's EOL worker elapsed fields to replay the ROM fly-in timing.
    if trace.level_complete && filter.eol_score {
        if let Some(eol_score) = &level_visuals.eol_score {
            push_eol_score_overlay(&mut commands, trace, eol_score);
            return Scene {
                background,
                commands,
                pause_palette_darken: trace.paused,
            };
        }
        let text_y = (FRAME_HEIGHT as i32) / 2 - 40;
        let mut line_y = text_y + 22;
        let time_bonus = wildside_core::time_bonus_loc_d56e(trace.level_timer_seconds);
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
    if trace.game_over && filter.eol_score {
        if let Some(eol_score) = &level_visuals.eol_score {
            push_game_over_continue_overlay(&mut commands, trace, eol_score);
        } else if trace.number_continues == 0 && filter.hud {
            // Degraded-asset fallback only. The normal asset path above uses
            // ASM loc_D25E / Continue_Screen EOL sprite workers.
            let go_text = "GAME OVER";
            let text_w = go_text.len() as i32 * 8 * 2;
            let text_x = (FRAME_WIDTH as i32 - text_w) / 2;
            let text_y = (FRAME_HEIGHT as i32) / 2 - 8;
            commands.push(text(text_x, text_y, go_text, [255, 80, 80], 2));
        }
    }

    // Pause overlay
    if trace.paused && filter.hud {
        hide_pause_menu_overwritten_sprite_commands_sub_6874(&mut commands);
        if !push_pause_menu_tilemap_loc_69d2(&mut commands, trace, level_visuals) {
            push_pause_menu_text_loc_69d2(&mut commands, trace);
        }
    }

    // Level complete overlay
    if trace.level_complete && filter.hud {
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

    if debug_overlay_visible && filter.hud {
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

    // #349: ASM lose_life (kid.asm:14763) -> Game_Mode 8 -> Load_TitleCard
    // allocates the same Title_Letter glyph workers as a normal entry. The
    // death-respawn card is traced as InGame during the respawn_live_handoff
    // hand-off (trace.rs:9177), so the gameplay scene must draw the level-name
    // glyph workers too — otherwise the post-death card shows the scene but
    // drops the name. No-op outside the title-card window (no glyph workers).
    commands.extend(build_title_card_worker_overlay_scene(trace, Some(level_visuals)).commands);

    push_teleport_warp_tint_loc_bcf6(&mut commands, trace);
    push_level_entry_palette_fade_sub_73d0(&mut commands, trace);

    Scene {
        background,
        commands,
        pause_palette_darken: trace.paused,
    }
}

fn boss_eye_object_mapping_index_loc_37abc(object: &NativeObjectSnapshot) -> Option<u8> {
    object
        .debug_fields
        .get("boss_eye_sprite_mapping_index_loc_37abc")
        .and_then(|value| value.parse::<u8>().ok())
}

fn boss_eye_object_order_word_loc_393c8(object: &NativeObjectSnapshot) -> u16 {
    object
        .debug_fields
        .get("boss_eye_sprite_order_word_a_loc_393c8")
        .and_then(|value| {
            value
                .strip_prefix("0x")
                .and_then(|hex| u16::from_str_radix(hex, 16).ok())
                .or_else(|| value.parse::<u16>().ok())
        })
        .unwrap_or_default()
}

fn trace_has_boss_eye_object_slot_loc_3931a(
    trace: &NativeTraceFrame,
    x_pos: u32,
    y_pos: u32,
) -> bool {
    trace.active_objects.iter().any(|object| {
        matches!(
            object.kind,
            NativeObjectKind::Enemy {
                object_type: BOSS_EYE_OBJECT_TYPE
            }
        ) && object.x_pos == x_pos
            && object.y_pos == y_pos
            && boss_eye_object_mapping_index_loc_37abc(object).is_some()
    })
}

fn order_boss_eye_satellites_sub_ff6(projectile_details: &mut [ActiveProjectileSnapshot]) {
    let mut ordered_satellites = projectile_details
        .iter()
        .copied()
        .filter(|projectile| projectile.kind == ActiveProjectileKind::BossEyeSatellite)
        .collect::<Vec<_>>();
    ordered_satellites.sort_by_key(|projectile| projectile.boss_eye_sprite_order_word_a);

    let mut next_satellite = ordered_satellites.into_iter();
    for projectile in projectile_details {
        if projectile.kind == ActiveProjectileKind::BossEyeSatellite
            && let Some(ordered_projectile) = next_satellite.next()
        {
            *projectile = ordered_projectile;
        }
    }
}

fn push_teleport_warp_tint_loc_bcf6(commands: &mut Vec<DrawCommand>, trace: &NativeTraceFrame) {
    if trace.teleport_warp_colour == 0 || trace.level_entry_fade_progress_sub_73d0.is_some() {
        return;
    }
    let colour = MdColour::from_word(trace.teleport_warp_colour);
    commands.push(DrawCommand::Tint {
        colour: [colour.red, colour.green, colour.blue],
        alpha: 96,
    });
}

fn push_level_entry_palette_fade_sub_73d0(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
) {
    if let Some(fade_progress) = trace.level_entry_fade_progress_sub_73d0 {
        commands.push(DrawCommand::PaletteFadeSub73D0 {
            target_word: trace.teleport_warp_colour,
            fade_progress,
        });
    }
}

fn floating_prize_sprite_visible_loc_103b2(
    content: wildside_core::terrain::PrizeContent,
    remaining_frames: u16,
) -> bool {
    match content {
        wildside_core::terrain::PrizeContent::Diamond if remaining_frames <= 0x00C8 => {
            // ASM PrizeDiamond_Init loc_103B2 swaps to `stru_10D80` exactly
            // when d3 reaches $00C8. That script alternates `unk_E0E66` with
            // null frame pointers every three frames, so Make_SpriteFromGfxObject
            // returns without emitting a VDP sprite on the odd slots.
            let blink_elapsed = 0x00C8u16.saturating_sub(remaining_frames);
            (blink_elapsed / 3).is_multiple_of(2)
        }
        _ => true,
    }
}

fn floating_prize_frame_index_loc_10396(
    content: wildside_core::terrain::PrizeContent,
    remaining_frames: u16,
    trace_frame_index: usize,
    frame_duration: usize,
    frame_count: usize,
) -> usize {
    if frame_count == 0 {
        return 0;
    }

    match content {
        wildside_core::terrain::PrizeContent::Diamond if remaining_frames > 0x00C8 => {
            // ASM PrizeDiamond_Init loc_10396 decrements d3 from $01F4 after
            // j_Init_Animation starts `stru_10D6E`; global Time_Frames does
            // not choose the local diamond frame.
            let elapsed = 0x01F4u16.saturating_sub(remaining_frames);
            usize::from(elapsed) / frame_duration % frame_count
        }
        wildside_core::terrain::PrizeContent::Diamond => {
            // ASM loc_103B2 switches to `stru_10D80` at d3=$00C8:
            // visible frame, null frame, next visible frame, null frame...
            let blink_elapsed = 0x00C8u16.saturating_sub(remaining_frames);
            usize::from(blink_elapsed) / (frame_duration * 2) % frame_count
        }
        _ => trace_frame_index / frame_duration % frame_count,
    }
}

fn render_floating_prize_worker_effect(
    commands: &mut Vec<DrawCommand>,
    level_visuals: &NativeLevelVisuals,
    content: wildside_core::terrain::PrizeContent,
    screen_x: i32,
    screen_y: i32,
    remaining_frames: u16,
) -> bool {
    let remaining_low_byte = remaining_frames as u8;
    if remaining_low_byte > PRIZE_GENERIC_LIFETIME_LOW_BYTE {
        return false;
    }
    let effect_age = PRIZE_GENERIC_LIFETIME_LOW_BYTE.wrapping_sub(remaining_low_byte);
    match content {
        wildside_core::terrain::PrizeContent::TenDiamonds => render_ten_diamonds_worker_effect(
            commands,
            level_visuals,
            screen_x,
            screen_y,
            effect_age,
        ),
        wildside_core::terrain::PrizeContent::Points10000 => render_10000_points_worker_effect(
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
    let frame = wildside_render::SpriteFrame {
        origin_x: 0x02,
        origin_y: -2,
        width_px: 0x08,
        height_px: 0x08,
        tile_base_index: 0x06ED - level_visuals.hud.diamond_base_tile_index,
        tile_sheet: level_visuals.hud.diamond_tiles.clone(),
    };
    let worker_scan = wildside_core::NativeRuntime::prize_10_diamonds_worker_scan_start_loc_10960();
    let worker_slots = wildside_core::NativeRuntime::prize_10_diamonds_init_loc_108E8();
    let max_worker_count = worker_slots.cleared_worker_slots.len() as u16;
    debug_assert_eq!(worker_scan.first_worker_slot_byte_offset, 0);
    debug_assert_eq!(
        max_worker_count,
        worker_scan.worker_table_len_bytes / worker_scan.worker_slot_stride_bytes
    );
    let mut drew_worker = false;
    for spawn_frame in 0..=u16::from(effect_age).min(0x001D) {
        let spawn = wildside_core::NativeRuntime::prize_10_diamonds_spawn_loc_108EE(
            spawn_frame,
            screen_x as u16,
            screen_y as u16,
        );
        if !spawn.should_spawn_worker {
            continue;
        }
        debug_assert!(spawn.should_hibernate_one_frame);
        debug_assert!(spawn.should_scan_workers);
        debug_assert_eq!(spawn.allocation_spec, Some(0x00FE_0000));
        debug_assert_eq!(spawn.priority, Some(1));
        debug_assert_eq!(spawn.display_enabled, Some(true));
        debug_assert_eq!(
            spawn.animation_script_label,
            Some("LnkTo_unk_E1046-Data_Index")
        );
        debug_assert_eq!(spawn.x_pos, Some(screen_x as u16));
        debug_assert_eq!(spawn.y_pos, Some(screen_y as u16));
        debug_assert_eq!(spawn.is_moved, Some(true));
        debug_assert_eq!(spawn.lifetime_frames, Some(0x0078));
        debug_assert!(u16::from(spawn.burst_table_index.unwrap_or_default()) < max_worker_count);

        let worker_age_u16 = u16::from(effect_age).wrapping_sub(spawn_frame);
        let worker_lifetime = spawn
            .lifetime_frames
            .unwrap_or_default()
            .saturating_sub(worker_age_u16);
        let countdown_gate =
            wildside_core::NativeRuntime::prize_10_diamonds_worker_countdown_gate_loc_10984(
                worker_lifetime,
            );
        if countdown_gate.should_clear_display_enabled || worker_lifetime < 0x0070 {
            continue;
        }

        let worker_age = i64::from(worker_age_u16);
        let worker_x = screen_x
            + ((i64::from(spawn.x_velocity.unwrap_or_default()) * worker_age) >> 16) as i32;
        let worker_y = screen_y
            + ((i64::from(spawn.y_velocity.unwrap_or_default()) * worker_age) >> 16) as i32;
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
    let mut drew_worker = false;
    for spawn_frame in 0..=u16::from(effect_age).min(0x0014) {
        let Some(spawn) = wildside_core::prize_10000_points_star_spawn_loc_10B3A(
            spawn_frame,
            screen_x as u16,
            screen_y as u16,
        ) else {
            continue;
        };
        debug_assert_eq!(spawn.palette_line, 1);
        debug_assert!(spawn.display_enabled);
        debug_assert_eq!(spawn.x_pos, screen_x as u16);
        debug_assert_eq!(spawn.y_pos, screen_y as u16);
        debug_assert!(spawn.is_moved);
        debug_assert_eq!(spawn.timer, 0x0078);

        let worker_age = u16::from(effect_age).wrapping_sub(spawn_frame);
        let tile_base_index = if spawn.sprite_mapping_label == "LnkTo_unk_E1056-Data_Index" {
            1
        } else {
            debug_assert_eq!(spawn.sprite_mapping_label, "LnkTo_unk_E104E-Data_Index");
            0
        };
        let frame = wildside_render::SpriteFrame {
            origin_x: 0x04,
            origin_y: -4,
            width_px: 0x08,
            height_px: 0x08,
            tile_base_index,
            tile_sheet: level_visuals.hud.digits.clone(),
        };
        let slot_offset = (spawn_frame / 5) * 4;
        let Some(star_step) = prize_10000_points_star_visible_step_loc_10b74(
            slot_offset,
            spawn.timer,
            screen_x as i16,
            screen_y as i16,
            spawn.is_moved,
            worker_age,
        ) else {
            continue;
        };
        push_worker_sprite(
            commands,
            &frame,
            &level_visuals.palettes.foreground,
            i32::from(star_step.x_pos),
            screen_y,
        );
        drew_worker = true;
    }
    drew_worker
}

fn prize_10000_points_star_visible_step_loc_10b74(
    slot_offset: u16,
    initial_timer: u16,
    initial_screen_x: i16,
    initial_screen_y: i16,
    initial_is_moved: bool,
    worker_age: u16,
) -> Option<wildside_core::Prize10000PointsStarStepLoc10B74> {
    // ASM loc_10B74/loc_10BB2: the source-port renderer has screen-relative
    // prize positions, so pass zero camera words and replay the star's age
    // through the ROM step model before drawing this frame.
    let mut timer = initial_timer;
    let mut x_pos = initial_screen_x;
    let mut is_moved = initial_is_moved;
    let mut step = wildside_core::Prize10000PointsStarStepLoc10B74 {
        latched_screen_pos: None,
        x_pos,
        is_moved,
        timer,
        delete_prize_worker: false,
    };

    for _ in 0..worker_age {
        step = wildside_core::prize_10000_points_star_step_loc_10B74(
            slot_offset,
            timer,
            x_pos,
            initial_screen_y,
            0,
            0,
            is_moved,
        );
        if step.delete_prize_worker {
            return None;
        }
        timer = step.timer;
        x_pos = step.x_pos;
        is_moved = step.is_moved;
    }

    Some(step)
}

fn push_worker_sprite(
    commands: &mut Vec<DrawCommand>,
    frame: &wildside_render::SpriteFrame,
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
    let (horizontal, vertical) = if (trace.frame_index & 0x2) == 0 {
        (-amplitude, amplitude)
    } else {
        (amplitude, -amplitude)
    };

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

fn push_heady_metal_death_split_pieces(
    commands: &mut Vec<DrawCommand>,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
) {
    for split in &trace.heady_metal_death_splits {
        for piece in &split.pieces {
            // ASM sub_378C8 loc_378D6 mutates the five cached HeadyMetal
            // workers at $FB72/$FB76/$FB7A/$FBAE/$FBB2; loc_37922 later
            // deletes the same slots. Core trace positions are already the
            // post-impulse worker positions, so render them directly.
            let screen_x = (piece.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32;
            let screen_y = (piece.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32;
            push_effect_enemy_sprite(
                commands,
                level_visuals,
                0x21,
                screen_x,
                screen_y,
                Direction::Right,
                false,
            );
        }
    }
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
    if enemy_visuals.frames.is_empty() {
        return;
    }
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

pub fn floating_item_graphics_spec(
    content_id: u8,
) -> Option<wildside_core::FloatingPrizeGraphicsSpec> {
    wildside_core::floating_prize_graphics_spec(wildside_core::terrain::PrizeContent::from_id(
        content_id,
    ))
}

fn floating_prize_visuals(
    level_visuals: &NativeLevelVisuals,
    content_id: u8,
) -> Option<&wildside_render::AnimatedSpriteVisual> {
    match wildside_core::terrain::PrizeContent::from_id(content_id) {
        // ASM PrizeDiamond_Init: `stru_10D6E` selects `unk_E0E66`, a VRAM
        // sprite frame over the HUD diamond tiles.
        wildside_core::terrain::PrizeContent::Diamond => {
            Some(&level_visuals.floating_prizes.diamond)
        }
        // ASM sub_10CB0 followed by PrizeHelmet*_Init selects the mask frame
        // and helmet palette for content ids 2..A.
        wildside_core::terrain::PrizeContent::Skycutter
        | wildside_core::terrain::PrizeContent::Cyclone
        | wildside_core::terrain::PrizeContent::RedStealth
        | wildside_core::terrain::PrizeContent::Eyeclops
        | wildside_core::terrain::PrizeContent::Juggernaut
        | wildside_core::terrain::PrizeContent::IronKnight
        | wildside_core::terrain::PrizeContent::Berzerker
        | wildside_core::terrain::PrizeContent::Maniaxe
        | wildside_core::terrain::PrizeContent::Micromax => {
            level_visuals.floating_prizes.helmets.get(&content_id)
        }
        // ASM PrizeAnkh_Init/PrizeClock_Init/PrizeContinue_Init select
        // `stru_10C08`, `stru_10C18`, and `stru_10C28` over ROM icon tiles.
        wildside_core::terrain::PrizeContent::Ankh
        | wildside_core::terrain::PrizeContent::Clock
        | wildside_core::terrain::PrizeContent::Continue => {
            level_visuals.floating_prizes.icons.get(&content_id)
        }
        _ => None,
    }
}

/// Returns a distinct RGB colour for each prize content type.
fn floating_item_colour(
    content: wildside_core::terrain::PrizeContent,
    graphics_spec: Option<wildside_core::FloatingPrizeGraphicsSpec>,
) -> [u8; 3] {
    match (content, graphics_spec) {
        // ASM PrizeDiamond_Init: palette line 0, `stru_10D6E`.
        (wildside_core::terrain::PrizeContent::Diamond, Some(_)) => [100, 200, 255],
        // ASM sub_10CB0: all helmet prizes use palette line 3; use the
        // object metadata to keep placeholder helmets visually distinct until
        // the real sprite art is wired.
        (_, Some(spec)) if spec.palette_line == 3 => helmet_placeholder_colour(spec.object_meta),
        (wildside_core::terrain::PrizeContent::Points10000, _) => [255, 255, 100],
        (wildside_core::terrain::PrizeContent::Ankh, _) => [255, 255, 200],
        (wildside_core::terrain::PrizeContent::Clock, _) => [200, 255, 200],
        (wildside_core::terrain::PrizeContent::Continue, _) => [255, 255, 255],
        (wildside_core::terrain::PrizeContent::TenDiamonds, _) => [100, 200, 255],
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap};

    use super::*;
    use wildside_core::{BlockBounceTrace, HeadyMetalDeathPieceTrace, HeadyMetalDeathSplitTrace};
    use wildside_data::BackgroundLayoutAlias;
    use wildside_render::{
        AnimatedSpriteVisual, BackgroundChunk, BackgroundPlacement, BackgroundTile,
        BackgroundVisuals, BlockVisualMapping, ChunkedBackgroundVisuals, EnemyVisuals, FlagVisuals,
        FloatingPrizeVisuals, ForegroundLayout, HudVisuals, LevelVisuals, PaletteSet,
        PlatformVisuals, PlayerFormVisuals, PlayerVisuals, SpecialEffectUnknownVisuals,
        SpriteFrame, Tile, TileQuad, WeatherVisuals,
    };

    #[test]
    fn manual_demo_return_renders_resident_gameplay_during_pal_fade_loc_119d8_28c6() {
        let trace = NativeTraceFrame {
            game_mode: GameMode::TitleScreen,
            game_mode_value: GameMode::TitleScreen.as_disassembly_value(),
            paused: true,
            camera_x_pos: 0x01E7_0000,
            camera_y_pos: 0x011C_0000,
            resident_gameplay_vdp_camera_loc_300ba: Some((0x01E8_0000, 0x011C_0000)),
            teleport_warp_colour: 0x082A,
            level_entry_fade_progress_sub_73d0: Some(0x00E0),
            ..NativeTraceFrame::default()
        };

        let render_trace = render_trace_with_vdp_latched_state(&trace, None);

        assert_eq!(render_trace.game_mode, GameMode::DemoPlay);
        assert!(!render_trace.paused);
        assert_eq!(
            (render_trace.camera_x_pos, render_trace.camera_y_pos),
            (0x01E8_0000, 0x011C_0000)
        );
        assert_eq!(render_trace.teleport_warp_colour, 0x082A);
        assert_eq!(
            render_trace.level_entry_fade_progress_sub_73d0,
            Some(0x00E0)
        );
    }

    #[test]
    fn title_card_glyph_tiles_follow_loc_1ac70_sprite_table_and_bounds() {
        let art = sample_title_card_letters_visuals(0x70);
        let mut commands = Vec::new();

        assert!(
            push_title_card_glyph_tiles(&mut commands, &art, 0x7B, 0x23, -0x10),
            "ASM loc_1AC70 should resolve glyph $7B through off_1AC2A and loaded title-card letter art"
        );

        let snapshots = commands
            .indexed_tiles()
            .into_iter()
            .map(indexed_tile_snapshot)
            .collect::<Vec<_>>();
        assert_eq!(
            snapshots,
            vec![
                (0x23, -0x10, 0x066, false),
                (0x23, -0x08, 0x067, false),
                (0x23, 0x00, 0x068, false),
                (0x23, 0x08, 0x069, false),
                (0x2B, -0x10, 0x06A, false),
                (0x2B, -0x08, 0x06B, false),
                (0x2B, 0x00, 0x06C, false),
                (0x2B, 0x08, 0x06D, false),
            ],
            "ASM loc_1AC70/off_1AC2A renders glyph $7B as a 2x4 tile sprite in column-major VRAM order"
        );

        let mut invalid_commands = Vec::new();
        assert!(
            !push_title_card_glyph_tiles(&mut invalid_commands, &art, 0x84, 0, 0),
            "ASM loc_1ABEA treats glyph $84 as spacing, not a loc_1AC70 sprite worker"
        );
        assert!(invalid_commands.is_empty());

        let short_art = sample_title_card_letters_visuals(0x06D);
        let mut short_commands = Vec::new();
        assert!(
            !push_title_card_glyph_tiles(&mut short_commands, &short_art, 0x7B, 0, 0),
            "ASM loc_1AC70 must fall back unless the loaded sheet contains every sprite_frame_vram tile"
        );
        assert!(short_commands.is_empty());
    }

    #[test]
    fn fire_demon_trail_sprite_uses_loaded_family_tile_sheet_loc_3a63c() {
        let mut visuals = sample_level_visuals(1);
        visuals.enemies.families.insert(
            FIRE_DEMON_OBJECT_TYPE,
            sample_animated_visual(sample_sprite_frame(0x77)),
        );
        let mut commands = Vec::new();

        assert!(push_fire_demon_trail_sprite(
            &mut commands,
            &visuals,
            32,
            40,
            FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME
        ));

        let DrawCommand::Sprite {
            tile_sheet, tiles, ..
        } = commands.single()
        else {
            panic!("ASM Fire_Demon loc_3A63C should emit a sprite trail frame");
        };
        assert_eq!(
            tile_sheet.tiles[0].pixels[0], 0x77,
            "ASM Fire_Demon loc_3A63C reuses the loaded Fire Demon art sheet for `stru_3A622` trail descriptors"
        );
        assert_eq!(tiles[0].tile.tile_index, 0x0B0);

        let mut wrapped_commands = Vec::new();
        assert!(push_fire_demon_trail_sprite(
            &mut wrapped_commands,
            &visuals,
            32,
            40,
            FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME - 11
        ));

        let DrawCommand::Sprite { tiles, .. } = wrapped_commands.single() else {
            panic!("ASM Fire_Demon loc_3A63C should wrap the six `stru_3A622` trail frames");
        };
        assert_eq!(
            tiles[0].tile.tile_index, 0x0BA,
            "ASM Fire_Demon loc_3A63C indexes `stru_3A622` modulo its six sprite frames"
        );
    }

    #[test]
    fn fire_demon_trail_fallback_keeps_single_rom_unspecified_marker_loc_3a63c() {
        let mut commands = Vec::new();

        push_fire_demon_trail(
            &mut commands,
            32,
            40,
            FIRE_DEMON_DEATH_TRAIL_NATIVE_LIFETIME,
        );

        assert_eq!(
            scene_rects(&commands),
            vec![(30, 37, 4, 5, FIRE_TRAIL_PLACEHOLDER_COLOUR)],
            "ASM Fire_Demon loc_3A63C specifies `stru_3A622` sprite descriptors; missing native art should stay a single placeholder rect, not invent per-frame geometry"
        );
    }

    #[test]
    fn block_debris_velocity_tables_match_asm_sub_292e_all_directions() {
        for (direction, expected, label) in [
            (
                2,
                [(-1, -4), (-1, -3), (1, -3), (1, -4), (1, -5)],
                "ASM sub_292E loc_2988 uses `unk_29FC`/`unk_2A06`",
            ),
            (
                0,
                [(-1, -2), (-2, 0), (1, 0), (1, -1), (-1, -2)],
                "ASM sub_292E loc_297A uses `unk_2A10`/`unk_2A1A`",
            ),
            (
                1,
                [(-5, -3), (-4, -3), (-3, -3), (-2, -3), (-1, -3)],
                "ASM sub_292E loc_296C uses `unk_2A38`/`unk_2A42`",
            ),
            (
                3,
                [(5, -3), (4, -3), (3, -3), (2, -3), (1, -3)],
                "ASM sub_292E fallthrough uses `unk_2A24`/`unk_2A2E`",
            ),
            (
                7,
                [(5, -3), (4, -3), (3, -3), (2, -3), (1, -3)],
                "ASM sub_292E non-0/1/2 directions share the fallthrough table",
            ),
        ] {
            assert_eq!(
                block_debris_velocity_sub_292e(direction),
                &expected,
                "{label}"
            );
        }
    }

    #[test]
    fn block_destroy_debris_sub_292e_emits_camera_relative_rom_tiles() {
        let mut visuals = sample_level_visuals(9);
        visuals.block_tiles = sample_tile_sheet_with_pixel(9, 0x44);
        let mut commands = Vec::new();

        assert!(
            push_block_destroy_debris_sub_292e(&mut commands, &visuals, 80, 96, 2, 11, 13),
            "ASM sub_292E/sub_2A4C should resolve debris tile `$8228` from block-art VRAM"
        );
        assert_eq!(
            commands
                .iter()
                .map(indexed_tile_snapshot)
                .collect::<Vec<_>>(),
            vec![
                (68, 79, 0x08, true),
                (68, 88, 0x08, true),
                (78, 88, 0x08, true),
                (78, 79, 0x08, true),
                (74, 82, 0x08, true),
            ],
            "ASM sub_292E seeds `unk_29E8`/`unk_29F2`, applies loc_2988 velocity, then sub_2A4C writes tile `$8228` after camera subtraction"
        );
        let DrawCommand::IndexedTile { tile_sheet, .. } = commands[0].clone() else {
            panic!("ASM sub_2A4C debris should emit indexed block-art tiles");
        };
        assert_eq!(tile_sheet.tiles[8].pixels[0], 0x44);

        let short_visuals = sample_level_visuals(8);
        commands.clear();
        assert!(
            !push_block_destroy_debris_sub_292e(&mut commands, &short_visuals, 80, 96, 2, 11, 13),
            "ASM sub_2A4C tile `$8228` requires block-art local tile `$08` to be loaded"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn previous_hud_diamond_frame_offset_loc_45ee_rewinds_dma_page_ring() {
        for (offset, expected, label) in [
            (
                0,
                6,
                "ASM sub_44DC loc_45EE wraps the first `unk_461C` page to the previous final page",
            ),
            (
                2,
                0,
                "ASM sub_44DC loc_45EE rewinds the second `unk_461C` page",
            ),
            (
                4,
                2,
                "ASM sub_44DC loc_45EE rewinds the third `unk_461C` page",
            ),
            (
                6,
                4,
                "ASM sub_44DC loc_45EE rewinds the fourth `unk_461C` page",
            ),
            (
                1,
                0,
                "ASM sub_44DC loc_45EE non-page offset saturates at zero",
            ),
            (
                8,
                6,
                "ASM sub_44DC loc_45EE fallback still rewinds one two-byte page",
            ),
        ] {
            assert_eq!(
                previous_hud_diamond_frame_offset_loc_45ee(offset),
                expected,
                "{label}"
            );
        }
    }

    #[test]
    fn mode_level_loc_901_latches_previous_sprite_sources() {
        let mut render_trace = NativeTraceFrame {
            active_objects: vec![NativeObjectSnapshot {
                x_pos: 0x0021_0000,
                y_pos: 0x0084_0000,
                ..sample_object_snapshot()
            }],
            active_projectiles: vec![(0x004B_0000, 0x00A4_0000, true)],
            active_projectile_details: vec![ActiveProjectileSnapshot {
                kind: ActiveProjectileKind::Generic,
                x_pos: 0x004B_0000,
                y_pos: 0x00A4_0000,
                friendly: true,
                ..ActiveProjectileSnapshot::default()
            }],
            ..NativeTraceFrame::default()
        };
        let previous_trace = NativeTraceFrame {
            active_objects: vec![
                NativeObjectSnapshot {
                    x_pos: 0x0020_0000,
                    y_pos: 0x0080_0000,
                    ..sample_object_snapshot()
                },
                NativeObjectSnapshot {
                    kind: NativeObjectKind::Enemy { object_type: 0x0C },
                    x_pos: 0x0044_0000,
                    y_pos: 0x006D_0000,
                    x_velocity: Some(-0x0002_0000),
                    y_velocity: Some(-0x0003_0000),
                    facing: Some(Direction::Left),
                    ..sample_object_snapshot()
                },
            ],
            active_projectiles: vec![
                (0x004C_0000, 0x00A1_0000, false),
                (0x0050_0000, 0x00A8_0000, true),
            ],
            active_projectile_details: vec![ActiveProjectileSnapshot {
                kind: ActiveProjectileKind::LionProjectile,
                x_pos: 0x004C_0000,
                y_pos: 0x00A1_0000,
                x_velocity: -0x0006_0000,
                animation_elapsed_frames: 6,
                ..ActiveProjectileSnapshot::default()
            }],
            ..NativeTraceFrame::default()
        };

        latch_previous_sprite_sources_loc_901(&mut render_trace, &previous_trace);

        assert_eq!(
            render_trace.active_objects, previous_trace.active_objects,
            "ASM Mode_Level loc_901 transfers the previous Sprite_Table before Execute_Objects advances object sprites"
        );
        assert_eq!(
            render_trace.active_projectiles, previous_trace.active_projectiles,
            "ASM Mode_Level loc_901 presents previous projectile sprite workers"
        );
        assert_eq!(
            render_trace.active_projectile_details, previous_trace.active_projectile_details,
            "ASM Mode_Level loc_901 presents previous typed projectile sprite details"
        );
    }

    #[test]
    fn eol_score_hardware_sprite_diagnostics_groups_32px_chunks_loc_12b8() {
        let tiles = vec![
            diagnostic_sprite_tile(0, 0, 0x10),
            diagnostic_sprite_tile(24, 24, 0x11),
            diagnostic_sprite_tile(32, 0, 0x20),
            diagnostic_sprite_tile(40, 8, 0x21),
            diagnostic_sprite_tile(0, 32, 0x30),
            diagnostic_sprite_tile(8, 40, 0x31),
        ];

        let diagnostics = eol_score_hardware_sprite_diagnostics(7, 100, 40, &tiles);

        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        diagnostic.command_index,
                        diagnostic.chunk_index,
                        diagnostic.x,
                        diagnostic.y,
                        diagnostic.width,
                        diagnostic.height,
                        diagnostic.first_tile_index,
                        diagnostic.min_x,
                        diagnostic.min_y,
                        diagnostic.max_x,
                        diagnostic.max_y,
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (7, 0, 100, 40, 32, 32, 0x10, 100, 40, 131, 71),
                (7, 1, 132, 40, 16, 16, 0x20, 132, 40, 147, 55),
                (7, 2, 100, 72, 16, 16, 0x30, 100, 72, 115, 87),
            ],
            "ASM Make_SpriteFromGfxObject loc_12B8/loc_12C0 splits EOL gfx frames into 32px hardware chunks before sub_13C0 sprite-table emission"
        );
    }

    #[test]
    fn eol_score_hardware_sprite_diagnostics_ignores_empty_tile_lists_loc_12b8() {
        assert!(
            eol_score_hardware_sprite_diagnostics(7, 100, 40, &[]).is_empty(),
            "ASM Make_SpriteFromGfxObject loc_12B8 has no hardware sprite when a logical frame has no tiles"
        );
    }

    #[test]
    fn eol_text_position_moves_before_draw_and_settles_loc_c5a2() {
        for (initial, velocity, timer, elapsed, expected, label) in [
            (0x40, 8, 0x1C, 0, 0x40, "initial frame"),
            (0x40, 8, 0x1C, 1, 0x48, "first hibernate tick"),
            (0x160, -8, 0x1C, 4, 0x140, "negative velocity"),
            (0x40, 8, 3, 10, 0x58, "settled after timer"),
        ] {
            assert_eq!(
                eol_position_loc_c5a2(initial, velocity, timer, elapsed),
                expected,
                "ASM Object_EOL_Text loc_C5A2 applies `$6C/$70` before drawing for {label}"
            );
        }
    }

    #[test]
    fn eol_number_position_draws_one_tick_before_advance_loc_c48c() {
        for (initial, velocity, timer, elapsed, expected, label) in [
            (0x40, 8, 0x1C, 0, 0x40, "initial frame"),
            (0x40, 8, 0x1C, 1, 0x40, "first visible digit draw"),
            (0x40, 8, 0x1C, 2, 0x48, "second visible digit draw"),
            (0x160, -8, 0x1C, 5, 0x140, "negative velocity"),
            (0x40, 8, 3, 10, 0x58, "settled after timer"),
        ] {
            assert_eq!(
                eol_number_position_loc_c48c(initial, velocity, timer, elapsed),
                expected,
                "ASM Object_EOL_Number loc_C48C draws from `$64/$68` before loc_C4E2 advances for {label}"
            );
        }
    }

    #[test]
    fn eol_number_sprites_follow_loc_c48c_digit_slots_and_zero_rules() {
        let eol = sample_eol_score_visuals();
        let palette = sample_palette();
        let mut commands = Vec::new();

        push_eol_number_sprites_with_palette(&mut commands, &eol, &palette, 0x80, 0x90, 1_203);

        assert_eq!(
            sprite_snapshots(&commands),
            vec![
                (0xA4, 0x88, 1, false),
                (0xB0, 0x88, 2, false),
                (0xBC, 0x88, 0, false),
                (0xC8, 0x88, 3, false),
            ],
            "ASM Object_EOL_Number loc_C48C suppresses leading zero slots, keeps interior zeroes, and spaces digits by $C from `$64/$68`"
        );
        for command in &commands {
            let DrawCommand::Sprite { tiles, .. } = command else {
                panic!("ASM Object_EOL_Number loc_C466 should emit sprite commands only");
            };
            assert!(
                tiles.iter().all(|tile| tile.tile.priority),
                "ASM Object_EOL_Number loc_C466 sets priority=$01 for every digit slot"
            );
        }

        commands.clear();
        push_eol_number_sprites_with_palette(&mut commands, &eol, &palette, 0x80, 0x90, 0);
        assert_eq!(
            sprite_snapshots(&commands),
            vec![(0xC8, 0x88, 0, false)],
            "ASM Object_EOL_Number loc_C4C0 keeps the final ones-place zero even when all leading slots are blank"
        );

        commands.clear();
        push_eol_number_sprites_with_palette(&mut commands, &eol, &palette, 0x80, 0x90, 12_345_678);
        assert_eq!(
            sprite_snapshots(&commands),
            vec![
                (0x80, 0x88, 9, false),
                (0x8C, 0x88, 9, false),
                (0x98, 0x88, 9, false),
                (0xA4, 0x88, 9, false),
                (0xB0, 0x88, 9, false),
                (0xBC, 0x88, 9, false),
                (0xC8, 0x88, 9, false),
            ],
            "ASM Object_EOL_Number loc_C4AE clamps displayed score-row numbers to 9999999"
        );
    }

    #[test]
    fn eol_bonus_row_bridge_follows_loc_d468_text_and_number_velocities() {
        let eol = sample_eol_score_visuals();
        let palette = sample_palette();
        let mut commands = Vec::new();

        push_eol_bonus_row_with_palette_loc_d468(
            &mut commands,
            &eol,
            &palette,
            &eol.path,
            0x10,
            0x30,
            0x90,
            0x100,
            0x24,
            3,
            1_203,
        );

        assert_eq!(
            sprite_snapshots(&commands),
            vec![
                (0x28, 0x88, 0xA3, false),
                (0x48, 0x88, 0xB0, false),
                (0x114, 0x88, 1, false),
                (0x120, 0x88, 2, false),
                (0x12C, 0x88, 0, false),
                (0x138, 0x88, 3, false),
            ],
            "ASM loc_D468 uses Load_Object_EOL_Text +8 slide for label/BONUS and Load_Object_EOL_Number -8 slide before loc_C48C digit slots"
        );
        for command in &commands {
            let DrawCommand::Sprite { tiles, .. } = command else {
                panic!("ASM loc_D468 EOL bonus rows should emit sprite commands only");
            };
            assert!(
                tiles.iter().all(|tile| tile.tile.priority),
                "ASM Object_EOL_Text loc_C592 and Object_EOL_Number loc_C466 set priority=$01"
            );
        }
    }

    #[test]
    fn game_over_continue_overlay_follows_loc_d25e_and_continue_screen_branches() {
        let eol = sample_eol_score_visuals();
        let mut commands = Vec::new();

        push_game_over_continue_overlay(
            &mut commands,
            &NativeTraceFrame {
                continue_countdown: Some(291),
                continue_countdown_elapsed_frames: CONTINUE_SCREEN_INITIAL_HOLD_FRAMES_LOC_D31E + 4,
                number_continues: 7,
                ..NativeTraceFrame::default()
            },
            &eol,
        );
        assert_eq!(
            sprite_snapshots(&commands),
            vec![(-0x58, 0x58, 0xC0, false), (0x164, 0x78, 7, false)],
            "ASM Continue_Screen loc_D31E/loc_D350 slides CONTINUES from $FF88 with +8 and the continue number from $134 with -8"
        );

        commands.clear();
        push_game_over_continue_overlay(
            &mut commands,
            &NativeTraceFrame {
                number_continues: 2,
                ..NativeTraceFrame::default()
            },
            &eol,
        );
        assert!(
            commands.is_empty(),
            "ASM loc_D25E branches to Continue_Screen when Number_Continues is non-zero; without countdown state native must not render GAME OVER"
        );

        commands.clear();
        push_game_over_continue_overlay(
            &mut commands,
            &NativeTraceFrame {
                number_continues: 0,
                game_over_elapsed_frames: 3,
                ..NativeTraceFrame::default()
            },
            &eol,
        );
        assert_eq!(
            sprite_snapshots(&commands),
            vec![(-0x50, 0x58, 0x60, false), (0x140, 0x78, 0x61, false)],
            "ASM loc_D25E no-continues path slides GAME from $FF98 with +8 and OVER from $158 with -8"
        );
    }

    #[test]
    fn eol_bonus_amounts_follow_loc_d468_row_order_and_flags() {
        let all_rows_trace = NativeTraceFrame {
            level_timer_seconds: 125,
            level_damage_taken: false,
            level_prize_collected: false,
            level_path_bonus_amount: 7_000,
            level_speed_bonus_awarded: true,
            ..NativeTraceFrame::default()
        };
        let (all_amounts, all_count) = eol_bonus_amounts_for_trace(&all_rows_trace);
        assert_eq!(all_count, 5);
        assert_eq!(
            &all_amounts[..all_count],
            &[2_050, 5_000, 5_000, 7_000, 10_000],
            "ASM loc_D468-loc_D660 loads EOL amounts as time, no-hit, no-prize, path, then speed"
        );

        let time_only_trace = NativeTraceFrame {
            level_timer_seconds: 1,
            level_damage_taken: true,
            level_prize_collected: true,
            level_path_bonus_amount: 0,
            level_speed_bonus_awarded: false,
            ..NativeTraceFrame::default()
        };
        let (time_only_amounts, time_only_count) = eol_bonus_amounts_for_trace(&time_only_trace);
        assert_eq!(time_only_count, 1);
        assert_eq!(
            &time_only_amounts[..time_only_count],
            &[10],
            "ASM loc_D5A0/loc_D5D2/loc_D60E/loc_D660 skip bonus rows when their trace flags say the ROM did not allocate them"
        );
    }

    #[test]
    fn eol_score_counter_snapshot_uses_loc_d17e_initial_score_gate() {
        let captured_initial_score_trace = NativeTraceFrame {
            level_complete: true,
            level_complete_initial_score: 1_000,
            score: 9_999,
            level_timer_seconds: 1,
            level_damage_taken: true,
            level_prize_collected: true,
            level_complete_score_counter_elapsed_frames: 1,
            ..NativeTraceFrame::default()
        };

        let captured = eol_score_counter_snapshot_for_trace(&captured_initial_score_trace);
        assert_eq!(
            (
                captured.score,
                captured.amount_count,
                captured.remaining_amounts[0]
            ),
            (1_010, 1, 0),
            "ASM loc_D17E preserves the copied starting score for loc_D770 rendering while the first frame drains the only 10-point row"
        );

        let fallback_initial_score_trace = NativeTraceFrame {
            level_complete: true,
            level_complete_initial_score: 0,
            score: 9_999,
            level_timer_seconds: 1,
            level_damage_taken: true,
            level_prize_collected: true,
            level_complete_score_counter_elapsed_frames: 1,
            ..NativeTraceFrame::default()
        };
        let fallback = eol_score_counter_snapshot_for_trace(&fallback_initial_score_trace);
        assert_eq!(
            fallback.score, 9_999,
            "without a non-zero loc_D17E copy, native reconstructs the pre-bonus score from the trace score minus the active loc_D468 rows"
        );

        let stalled_trace = NativeTraceFrame {
            level_complete: true,
            level_complete_initial_score: 1_000,
            level_timer_seconds: 125,
            level_damage_taken: true,
            level_prize_collected: true,
            level_complete_score_counter_elapsed_frames: 5,
            ..NativeTraceFrame::default()
        };
        let stalled = eol_score_counter_snapshot_for_trace(&stalled_trace);
        assert_eq!(
            (stalled.score, stalled.remaining_amounts[0]),
            (1_160, 1_890),
            "ASM loc_D774 oracle stall mapping exposes four loc_D770 transfers by trace frame 5, not five"
        );
    }

    #[test]
    fn eol_level_skip_time_digits_follow_loc_d47e_gates_and_order() {
        let eol = sample_eol_score_visuals();
        let palette = sample_palette();

        for (trace, label) in [
            (
                NativeTraceFrame {
                    level_skip_cheat_enabled: 0,
                    level_complete_score_rows_started: 1,
                    level_completion_time: 123,
                    ..NativeTraceFrame::default()
                },
                "disabled level-skip cheat",
            ),
            (
                NativeTraceFrame {
                    level_skip_cheat_enabled: 1,
                    level_complete_score_rows_started: 0,
                    level_completion_time: 123,
                    ..NativeTraceFrame::default()
                },
                "score rows not started",
            ),
        ] {
            let mut commands = Vec::new();

            push_eol_level_skip_time_digits_with_palette_loc_d47e(
                &mut commands,
                &eol,
                &palette,
                &trace,
            );

            assert!(
                commands.is_empty(),
                "ASM loc_D468 should skip loc_D47E time digits for {label}"
            );
        }

        let mut commands = Vec::new();
        let active_trace = NativeTraceFrame {
            level_skip_cheat_enabled: 1,
            level_complete_score_rows_started: 1,
            level_completion_time: 123,
            ..NativeTraceFrame::default()
        };

        push_eol_level_skip_time_digits_with_palette_loc_d47e(
            &mut commands,
            &eol,
            &palette,
            &active_trace,
        );

        assert_eq!(
            sprite_snapshots(&commands),
            vec![
                (0x11C, 0xCC, 3, false),
                (0x110, 0xCC, 2, false),
                (0x104, 0xCC, 1, false)
            ],
            "ASM loc_D47E emits ones/tens/hundreds completion-time digits at $11C/$110/$104 with priority set"
        );
        for command in &commands {
            let DrawCommand::Sprite { tiles, .. } = command else {
                panic!("ASM loc_D47E should emit sprite commands only");
            };
            assert!(
                tiles.iter().all(|tile| tile.tile.priority),
                "ASM loc_D47E sets priority=$01 on each level-skip time digit"
            );
        }
    }

    #[test]
    fn eol_score_room_palette_fades_on_loc_d734_boundary() {
        let palette = Palette {
            colours: [0x0866, 0x00CE, 0x0000, 0x0040, 0x00E0, 0x0EEE]
                .into_iter()
                .map(MdColour::from_word)
                .collect(),
        };
        let before_fade_trace = NativeTraceFrame {
            level_complete_score_rows_elapsed_frames:
                EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE - 1,
            ..NativeTraceFrame::default()
        };
        let fade_boundary_trace = NativeTraceFrame {
            level_complete_score_rows_elapsed_frames:
                EOL_SCORE_ROWS_RENDER_FADE_IN_FRAMES_LOC_D734_ORACLE,
            ..NativeTraceFrame::default()
        };

        assert_eq!(
            palette_raw_words(&eol_score_room_palette_loc_d734(
                &before_fade_trace,
                &palette
            )),
            vec![0x0866, 0x00CE, 0x0000, 0x0040, 0x00E0, 0x0EEE],
            "ASM loc_D734 waits eight Hibernate_Object_1Frame iterations before the final score-room palette is visible"
        );
        assert_eq!(
            palette_raw_words(&eol_score_room_palette_loc_d734(
                &fade_boundary_trace,
                &palette
            )),
            vec![0x0433, 0x0067, 0x0000, 0x0040, 0x00E0, 0x0EEE],
            "ASM loc_D734/sub_14C0 fades room colours at the eighth tick while native keeps the oracle-bright grid colours"
        );
    }

    #[test]
    fn pause_menu_text_follows_loc_69d2_life_branch_and_loc_6b0e_cursor() {
        for (trace, expected, label) in [
            (
                NativeTraceFrame {
                    number_lives: 2,
                    pause_menu_option: 0,
                    ..NativeTraceFrame::default()
                },
                vec![
                    (
                        PAUSE_MENU_CURSOR_X_LOC_6A84,
                        PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT,
                        ">",
                    ),
                    (
                        PAUSE_MENU_TEXT_X_LOC_69D2,
                        PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT,
                        "CONTINUE",
                    ),
                    (
                        PAUSE_MENU_TEXT_X_LOC_69D2,
                        PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT,
                        "RESTART ROUND",
                    ),
                ],
                "restart row",
            ),
            (
                NativeTraceFrame {
                    number_lives: 1,
                    pause_menu_option: 1,
                    ..NativeTraceFrame::default()
                },
                vec![
                    (
                        PAUSE_MENU_CURSOR_X_LOC_6A84,
                        PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT,
                        ">",
                    ),
                    (
                        PAUSE_MENU_TEXT_X_LOC_69D2,
                        PAUSE_MENU_TEXT_TOP_Y_RESTART_TEXT,
                        "CONTINUE",
                    ),
                    (
                        PAUSE_MENU_TEXT_X_LOC_69D2,
                        PAUSE_MENU_TEXT_BOTTOM_Y_RESTART_TEXT,
                        "GIVE UP",
                    ),
                ],
                "give-up row",
            ),
        ] {
            let mut commands = Vec::new();

            push_pause_menu_text_loc_69d2(&mut commands, &trace);

            assert_eq!(
                text_snapshots(&commands),
                expected
                    .into_iter()
                    .map(|(x, y, text)| (x, y, text.to_owned()))
                    .collect::<Vec<_>>(),
                "ASM loc_69D2 chooses {label} from Number_Lives and loc_6B0E places the live cursor from Pause_Option"
            );
        }
    }

    #[test]
    fn pause_menu_tilemap_follows_loc_69d2_block_and_restart_text_rows() {
        let trace = NativeTraceFrame::default();
        let mut visuals = sample_level_visuals(1);
        let mut commands = Vec::new();
        assert!(
            !push_pause_menu_tilemap_loc_69d2(&mut commands, &trace, &visuals),
            "ASM loc_69D2 tilemap path should fall back when pause menu art is absent"
        );
        assert!(commands.is_empty());

        visuals.pause_menu = Some(sample_pause_menu_visuals());
        for (trace, expected_base, expected_upper_cursor, expected_lower_cursor, label) in [
            (
                NativeTraceFrame {
                    number_lives: 2,
                    pause_menu_option: 0,
                    ..NativeTraceFrame::default()
                },
                0x0100,
                PAUSE_MENU_CURSOR_TILE_LOC_6B0E,
                PAUSE_MENU_BLANK_TILE_LOC_6B0E,
                "restart",
            ),
            (
                NativeTraceFrame {
                    number_lives: 1,
                    pause_menu_option: 1,
                    ..NativeTraceFrame::default()
                },
                0x0200,
                PAUSE_MENU_BLANK_TILE_LOC_6B0E,
                PAUSE_MENU_CURSOR_TILE_LOC_6B0E,
                "give-up",
            ),
        ] {
            let mut commands = Vec::new();

            assert!(
                push_pause_menu_tilemap_loc_69d2(&mut commands, &trace, &visuals),
                "ASM loc_69D2 should use loaded pause menu tile art for the {label} row"
            );

            let snapshots = commands
                .indexed_tiles()
                .into_iter()
                .map(indexed_tile_snapshot)
                .collect::<Vec<_>>();
            assert_eq!(
                snapshots.len(),
                PAUSE_MENU_TILE_ROWS_LOC_69D2 * PAUSE_MENU_TILE_COLUMNS_LOC_69D2,
                "ASM loc_69D2/restart_text writes exactly five rows of 16 pause-menu tile words"
            );
            assert_eq!(
                snapshots[0],
                (
                    PAUSE_MENU_TILE_X_LOC_69D2,
                    PAUSE_MENU_TILE_Y_RESTART_TEXT,
                    expected_base,
                    true
                )
            );
            assert_eq!(
                snapshots[1],
                (
                    PAUSE_MENU_TILE_X_LOC_69D2 + 8,
                    PAUSE_MENU_TILE_Y_RESTART_TEXT,
                    expected_base + 1,
                    true
                )
            );
            assert_eq!(
                snapshots[16],
                (
                    PAUSE_MENU_TILE_X_LOC_69D2,
                    PAUSE_MENU_TILE_Y_RESTART_TEXT + 8,
                    expected_base + 0x10,
                    true
                )
            );
            assert_eq!(
                snapshots[17],
                (
                    PAUSE_MENU_TILE_X_LOC_69D2 + 8,
                    PAUSE_MENU_TILE_Y_RESTART_TEXT + 8,
                    expected_upper_cursor,
                    true
                ),
                "ASM loc_6B0E rewrites the upper cursor slot for the {label} option"
            );
            assert_eq!(
                snapshots[49],
                (
                    PAUSE_MENU_TILE_X_LOC_69D2 + 8,
                    PAUSE_MENU_TILE_Y_RESTART_TEXT + 0x18,
                    expected_lower_cursor,
                    true
                ),
                "ASM loc_6B0E rewrites the lower cursor slot for the {label} option"
            );
            assert_eq!(
                snapshots[79],
                (
                    PAUSE_MENU_TILE_X_LOC_69D2 + 0x78,
                    PAUSE_MENU_TILE_Y_RESTART_TEXT + 0x20,
                    expected_base + 0x4F,
                    true
                ),
                "ASM loc_69D2 keeps the pause menu tile block on an 8-pixel grid"
            );
        }
    }

    #[test]
    fn pause_menu_tile_cursor_rewrites_only_live_slots_loc_6b0e() {
        for (pause_menu_option, expected_top, expected_bottom, label) in [
            (
                0,
                PAUSE_MENU_CURSOR_TILE_LOC_6B0E,
                PAUSE_MENU_BLANK_TILE_LOC_6B0E,
                "upper",
            ),
            (
                1,
                PAUSE_MENU_BLANK_TILE_LOC_6B0E,
                PAUSE_MENU_CURSOR_TILE_LOC_6B0E,
                "lower",
            ),
        ] {
            let trace = NativeTraceFrame {
                pause_menu_option,
                ..NativeTraceFrame::default()
            };

            assert_eq!(
                pause_menu_tile_with_live_cursor_loc_6b0e(&trace, 1, 1, sample_tile_ref(0x0123))
                    .tile_index,
                expected_top,
                "ASM loc_6B0E writes the {label} cursor state to the top arrow slot only"
            );
            assert_eq!(
                pause_menu_tile_with_live_cursor_loc_6b0e(&trace, 3, 1, sample_tile_ref(0x0123))
                    .tile_index,
                expected_bottom,
                "ASM loc_6B0E writes the {label} cursor state to the bottom arrow slot only"
            );

            for (row, column) in [(1, 2), (2, 1), (3, 2), (0, 1)] {
                assert_eq!(
                    pause_menu_tile_with_live_cursor_loc_6b0e(
                        &trace,
                        row,
                        column,
                        sample_tile_ref(0x0123),
                    )
                    .tile_index,
                    0x0123,
                    "ASM loc_6B0E leaves non-cursor pause tile row {row} column {column} unchanged"
                );
            }
        }
    }

    #[test]
    fn pause_menu_sprite_hide_masks_first_tile_range_sub_6874() {
        let mut commands = vec![
            DrawCommand::Rect {
                x: 1,
                y: 2,
                width: 3,
                height: 4,
                colour: [5, 6, 7],
            },
            sample_sprite_command(&[]),
            sample_sprite_command(&[0x0675]),
            sample_sprite_command(&[0x0676]),
            sample_sprite_command(&[0x8676]),
            sample_sprite_command(&[0x068E]),
            sample_sprite_command(&[0x068F]),
            sample_sprite_command(&[0x0675, 0x0676]),
        ];

        hide_pause_menu_overwritten_sprite_commands_sub_6874(&mut commands);

        let retained_first_tiles = commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite { tiles, .. } => {
                    Some(tiles.first().map(|tile| tile.tile.tile_index))
                }
                DrawCommand::Rect { .. } => None,
                other => panic!("unexpected command in pause sprite-hide fixture: {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            retained_first_tiles,
            vec![None, Some(0x0675), Some(0x068F), Some(0x0675)],
            "ASM sub_6874 loc_68AC masks first pattern tile with `$7FF` and clears only `$676..=$68E` pause-menu sprites"
        );
        assert!(
            matches!(commands.first(), Some(DrawCommand::Rect { .. })),
            "ASM sub_6874 scans hardware sprites only and leaves other draw commands intact"
        );
    }

    #[test]
    fn teleport_warp_tint_emits_until_palette_fade_starts_loc_bcf6_sub_73d0() {
        let mut commands = Vec::new();
        let trace = NativeTraceFrame {
            teleport_warp_colour: 0x00EE,
            level_entry_fade_progress_sub_73d0: None,
            ..NativeTraceFrame::default()
        };

        push_teleport_warp_tint_loc_bcf6(&mut commands, &trace);

        let colour = MdColour::from_word(0x00EE);
        assert_eq!(
            commands.single(),
            &DrawCommand::Tint {
                colour: [colour.red, colour.green, colour.blue],
                alpha: 96,
            },
            "ASM loc_BCF6 seeds `$FBCC=$00EE`; native renders it as the pre-sub_73D0 teleport tint"
        );

        let mut commands = Vec::new();
        push_teleport_warp_tint_loc_bcf6(
            &mut commands,
            &NativeTraceFrame {
                teleport_warp_colour: 0,
                level_entry_fade_progress_sub_73d0: None,
                ..NativeTraceFrame::default()
            },
        );
        assert!(
            commands.is_empty(),
            "without ASM loc_BCF6 `$FBCC` colour there is no teleport tint command"
        );

        push_teleport_warp_tint_loc_bcf6(
            &mut commands,
            &NativeTraceFrame {
                teleport_warp_colour: 0x00EE,
                level_entry_fade_progress_sub_73d0: Some(0),
                ..NativeTraceFrame::default()
            },
        );
        assert!(
            commands.is_empty(),
            "once ASM sub_73D0 owns the fullscreen fade, native must not also draw the pre-fade tint"
        );
    }

    #[test]
    fn level_entry_palette_fade_carries_target_word_and_progress_sub_73d0() {
        let mut commands = Vec::new();

        push_level_entry_palette_fade_sub_73d0(
            &mut commands,
            &NativeTraceFrame {
                teleport_warp_colour: 0x00EE,
                level_entry_fade_progress_sub_73d0: Some(0x0040),
                ..NativeTraceFrame::default()
            },
        );

        assert_eq!(
            commands.single(),
            &DrawCommand::PaletteFadeSub73D0 {
                target_word: 0x00EE,
                fade_progress: 0x0040,
            },
            "ASM sub_73D0 fades from Palette_Buffer_2 toward the `$FBCC` target word using `$F876` progress"
        );

        let mut commands = Vec::new();
        push_level_entry_palette_fade_sub_73d0(
            &mut commands,
            &NativeTraceFrame {
                teleport_warp_colour: 0x00EE,
                level_entry_fade_progress_sub_73d0: None,
                ..NativeTraceFrame::default()
            },
        );
        assert!(
            commands.is_empty(),
            "without the traced ASM sub_73D0 `$F876` progress, native emits no fullscreen fade command"
        );
    }

    #[test]
    fn shooter_block_projectile_tail_loc_ef62_matches_direction_tile_words() {
        for (x_velocity, y_velocity, expected, label) in [
            (0, -1, (-4, -8, 0x8295), "ASM loc_EF62 up tail"),
            (1, 0, (0, -4, 0x8A96), "ASM loc_EF62 right tail"),
            (0, 1, (-4, 0, 0x9295), "ASM loc_EF62 down tail"),
            (-1, 0, (-8, -4, 0x8296), "ASM loc_EF62 left tail"),
            (0, 0, (-8, -4, 0x8296), "ASM loc_EF62 stopped tail"),
            (
                1,
                -1,
                (-4, -8, 0x8295),
                "ASM loc_EF62 checks vertical-up before horizontal travel",
            ),
            (
                1,
                1,
                (0, -4, 0x8A96),
                "ASM loc_EF62 checks horizontal-right before vertical-down",
            ),
        ] {
            let projectile = ActiveProjectileSnapshot {
                x_velocity,
                y_velocity,
                ..ActiveProjectileSnapshot::default()
            };

            assert_eq!(
                shooter_block_projectile_sprite_tail_loc_ef62(&projectile),
                expected,
                "{label}"
            );
        }
    }

    #[test]
    fn shooter_block_projectile_sprite_loc_ef62_resolves_offsets_and_vram_tiles() {
        let mut visuals = sample_level_visuals(0x77);
        visuals.block_tiles = sample_tile_sheet_with_pixel(0x77, 0x66);

        for (
            x_velocity,
            y_velocity,
            expected_x,
            expected_y,
            expected_local_tile,
            expected_flip_x,
            expected_flip_y,
            label,
        ) in [
            (0, -1, 36, 42, 0x75, false, false, "up"),
            (1, 0, 40, 46, 0x76, true, false, "right"),
            (0, 1, 36, 50, 0x75, false, true, "down"),
            (-1, 0, 32, 46, 0x76, false, false, "left"),
        ] {
            let projectile = ActiveProjectileSnapshot {
                x_velocity,
                y_velocity,
                ..ActiveProjectileSnapshot::default()
            };
            let mut commands = Vec::new();

            assert!(
                push_shooter_block_projectile_sprite_loc_ef62(
                    &mut commands,
                    &visuals,
                    &projectile,
                    40,
                    50
                ),
                "ASM loc_EF62 {label} shooter-block tail should resolve to loaded block art"
            );

            assert_eq!(
                sprite_tile_snapshot(commands.single()),
                (
                    expected_x,
                    expected_y,
                    expected_local_tile,
                    0,
                    true,
                    expected_flip_x,
                    expected_flip_y,
                    0,
                    0,
                ),
                "ASM loc_EF62 {label} shooter-block tail tile word/offset"
            );
            let DrawCommand::Sprite {
                tile_sheet, tiles, ..
            } = commands.single()
            else {
                panic!("ASM loc_EF62 should emit a one-tile sprite");
            };
            assert_eq!(tiles.len(), 1);
            assert_eq!(
                tile_sheet.tiles[usize::from(expected_local_tile)].pixels[0],
                0x66,
                "ASM loc_EF62 {label} tile should be resolved from block-art VRAM"
            );
        }

        let short_visuals = sample_level_visuals(0x76);
        let projectile = ActiveProjectileSnapshot {
            x_velocity: 1,
            ..ActiveProjectileSnapshot::default()
        };
        let mut commands = Vec::new();

        assert!(
            !push_shooter_block_projectile_sprite_loc_ef62(
                &mut commands,
                &short_visuals,
                &projectile,
                40,
                50
            ),
            "ASM loc_EF62 tile `$296` needs block-art local tile `$76` to be loaded"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn lion_projectile_sprite_loc_3e12a_uses_stru_3df34_frames_and_facing() {
        let mut visuals = sample_level_visuals(1);
        visuals
            .enemies
            .families
            .insert(LION_OBJECT_TYPE, sample_lion_projectile_visuals());

        for (
            animation_elapsed_frames,
            x_velocity,
            expected_x,
            expected_y,
            expected_pixel,
            expected_tile_index,
            expected_flip_x,
            label,
        ) in [
            (0, 1, 44, 48, 0x90, 0x000, false, "first right"),
            (6, 1, 44, 48, 0x91, 0x004, false, "second right"),
            (12, 1, 44, 49, 0x92, 0x008, false, "third right"),
            (18, 1, 44, 48, 0x90, 0x000, false, "wrapped right"),
            (6, -1, 40, 48, 0x91, 0x004, true, "second left"),
            (6, 0, 44, 48, 0x91, 0x004, false, "zero velocity right"),
        ] {
            let projectile = ActiveProjectileSnapshot {
                animation_elapsed_frames,
                x_velocity,
                ..ActiveProjectileSnapshot::default()
            };
            let mut commands = Vec::new();

            assert!(
                push_lion_projectile_sprite_loc_3e12a(&mut commands, &visuals, &projectile, 50, 60),
                "ASM Lion.asm loc_3E12A {label} projectile frame should resolve from `stru_3DF34`"
            );

            assert_eq!(
                sprite_tile_snapshot(commands.single()),
                (
                    expected_x,
                    expected_y,
                    expected_tile_index,
                    0,
                    false,
                    expected_flip_x,
                    false,
                    if expected_flip_x { 8 } else { 0 },
                    0,
                ),
                "ASM Lion.asm loc_3E12A {label} projectile draw command"
            );
            let DrawCommand::Sprite {
                tile_sheet, tiles, ..
            } = commands.single()
            else {
                panic!("ASM Lion.asm loc_3E12A should emit a projectile sprite");
            };
            assert_eq!(
                tiles.len(),
                4,
                "ASM Lion.asm loc_3E12A `stru_3DF34` projectile frames cover a 2x2 aligned tile footprint"
            );
            assert_eq!(
                tile_sheet.tiles[0].pixels[0], expected_pixel,
                "ASM Lion.asm loc_3E12A {label} should select the expected `stru_3DF34` frame"
            );
        }
    }

    #[test]
    fn lion_projectile_sprite_loc_3e12a_rejects_missing_or_short_visuals() {
        let visuals = sample_level_visuals(1);
        let projectile = ActiveProjectileSnapshot::default();
        let mut commands = Vec::new();

        assert!(
            !push_lion_projectile_sprite_loc_3e12a(&mut commands, &visuals, &projectile, 50, 60),
            "ASM Lion.asm loc_3E12A projectile art requires the Lion enemy family"
        );
        assert!(commands.is_empty());

        let mut visuals = sample_level_visuals(1);
        let mut short_visual = sample_lion_projectile_visuals();
        short_visual.frames.truncate(9);
        visuals
            .enemies
            .families
            .insert(LION_OBJECT_TYPE, short_visual);

        assert!(
            !push_lion_projectile_sprite_loc_3e12a(&mut commands, &visuals, &projectile, 50, 60),
            "ASM Lion.asm loc_3E12A needs frame index 9 from `stru_3DF34`"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn projectile_child_static_frames_use_rom_sprite_descriptors() {
        let visuals = sample_animated_visual(sample_sprite_frame(0x5A));

        assert_child_frame(
            drips_drop_child_frame(&visuals),
            (0x082, 0x02, 0x09, 0x05, 0x0A),
            "ASM Drips.asm loc_33424 falling drop `unk_C7B26`",
        );
        assert_child_frame(
            tar_monster_glob_child_frame(&visuals),
            (0x0BC, 0x04, 0x04, 0x1E, 0x08),
            "ASM Tar_Monster.asm loc_3247E straight glob `unk_C781E`",
        );
        assert_child_frame(
            tar_monster_burst_child_frame(&visuals),
            (0x0C1, 0x03, 0x03, 0x06, 0x06),
            "ASM Tar_Monster.asm loc_325E6 burst fragment `unk_C782E`",
        );
        assert_child_frame(
            dragon_flying_descent_fire_child_frame(
                &visuals,
                &ActiveProjectileSnapshot {
                    dragon_flying_descent_fire_state:
                        DragonFlyingDescentFireState::FirstAnimationLoc34C26,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            (0x0EC, 0x05, 0x05, 0x13, 0x18),
            "ASM Dragon_Flying.asm loc_34BAC first descent fire frame",
        );
        assert_child_frame(
            dragon_flying_descent_fire_child_frame(
                &visuals,
                &ActiveProjectileSnapshot {
                    dragon_flying_descent_fire_state:
                        DragonFlyingDescentFireState::TravelAnimationLoc34C3A,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            (0x0F5, 0x06, 0x07, 0x1B, 0x1F),
            "ASM Dragon_Flying.asm loc_34C3A travel descent fire frame",
        );
        assert_child_frame(
            dragon_flying_descent_fire_child_frame(
                &visuals,
                &ActiveProjectileSnapshot {
                    dragon_flying_descent_fire_state:
                        DragonFlyingDescentFireState::CleanupFirstLoc34C4A,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            (0x105, 0x0D, 0x05, 0x1F, 0x1B),
            "ASM Dragon_Flying.asm loc_34C4A first cleanup descent fire frame",
        );
        assert_child_frame(
            dragon_flying_descent_fire_child_frame(
                &visuals,
                &ActiveProjectileSnapshot {
                    dragon_flying_descent_fire_state:
                        DragonFlyingDescentFireState::CleanupSecondLoc34C4A,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            (0x115, 0x0F, 0x05, 0x20, 0x10),
            "ASM Dragon_Flying.asm loc_34C4A second cleanup descent fire frame",
        );
    }

    #[test]
    fn projectile_child_animation_frames_pin_rom_script_boundaries() {
        let visuals = sample_animated_visual(sample_sprite_frame(0x5A));

        for (remaining, expected, label) in [
            (
                u16::MAX,
                (0x038, 0x09, 0x00, 0x10, 0x05),
                "ASM Tank_Shooting.asm `stru_3B526` elapsed 0",
            ),
            (
                u16::MAX - 5,
                (0x038, 0x09, 0x00, 0x10, 0x05),
                "ASM Tank_Shooting.asm `stru_3B526` elapsed 5",
            ),
            (
                u16::MAX - 6,
                (0x03A, 0x09, 0x00, 0x0F, 0x05),
                "ASM Tank_Shooting.asm `stru_3B526` elapsed 6",
            ),
            (
                u16::MAX - 15,
                (0x038, 0x09, 0x00, 0x10, 0x05),
                "ASM Tank_Shooting.asm `stru_3B526` elapsed 15",
            ),
            (
                u16::MAX - 60,
                (0x038, 0x09, 0x00, 0x10, 0x05),
                "ASM Tank_Shooting.asm `stru_3B526` elapsed 60",
            ),
        ] {
            assert_child_frame(
                tank_bullet_child_frame(&visuals, remaining),
                expected,
                label,
            );
        }

        for (remaining, expected, label) in [
            (
                15,
                (0x000, 0x08, 0x00, 0x10, 0x10),
                "ASM Robot.asm `stru_3A762` impact elapsed 0",
            ),
            (
                12,
                (0x004, 0x0A, 0x00, 0x14, 0x14),
                "ASM Robot.asm `stru_3A762` impact elapsed 3",
            ),
            (
                8,
                (0x00D, 0x0B, 0x00, 0x16, 0x16),
                "ASM Robot.asm `stru_3A762` impact elapsed 7",
            ),
        ] {
            assert_child_frame(
                robot_shot_child_frame(&visuals, remaining, 0, true),
                expected,
                label,
            );
        }

        for (elapsed, expected, label) in [
            (
                0,
                (0x016, 0x03, 0x00, 0x1F, 0x06),
                "ASM Robot.asm `stru_3A754` flying elapsed 0",
            ),
            (
                3,
                (0x01A, 0x03, 0x00, 0x1F, 0x06),
                "ASM Robot.asm `stru_3A754` flying elapsed 3",
            ),
            (
                6,
                (0x01E, 0x03, 0x00, 0x17, 0x06),
                "ASM Robot.asm `stru_3A754` flying elapsed 6",
            ),
            (
                9,
                (0x016, 0x03, 0x00, 0x1F, 0x06),
                "ASM Robot.asm `stru_3A754` flying elapsed 9",
            ),
        ] {
            assert_child_frame(
                robot_shot_child_frame(&visuals, 0, elapsed, false),
                expected,
                label,
            );
        }

        for (projectile, expected, label) in [
            (
                ActiveProjectileSnapshot {
                    x_velocity: 0x1000,
                    y_velocity: -0x1000,
                    ..ActiveProjectileSnapshot::default()
                },
                (0x00C, 0x00, 0x23, 0x23, 0x23),
                "ASM Cloud.asm loc_35DF0 diagonal bolt `unk_C7876`",
            ),
            (
                ActiveProjectileSnapshot {
                    x_velocity: 0x1000,
                    y_velocity: 0,
                    ..ActiveProjectileSnapshot::default()
                },
                (0x000, 0x00, 0x04, 0x30, 0x07),
                "ASM Cloud.asm loc_35DF0 horizontal bolt `unk_C7866`",
            ),
            (
                ActiveProjectileSnapshot {
                    x_velocity: 0,
                    y_velocity: 0x1000,
                    ..ActiveProjectileSnapshot::default()
                },
                (0x006, 0x04, 0x2F, 0x07, 0x30),
                "ASM Cloud.asm loc_35DF0 vertical bolt `unk_C786E`",
            ),
        ] {
            assert_child_frame(
                cloud_lightning_child_frame_loc_35df0(&visuals, &projectile),
                expected,
                label,
            );
        }

        for (remaining, expected, label) in [
            (
                68,
                (0x028, 0x0B, 0x00, 0x1B, 0x13),
                "ASM Dragon.asm `stru_3CCC8` elapsed 0",
            ),
            (
                61,
                (0x028, 0x0B, 0x00, 0x1B, 0x13),
                "ASM Dragon.asm `stru_3CCC8` elapsed 7",
            ),
            (
                60,
                (0x034, 0x0A, 0x00, 0x20, 0x15),
                "ASM Dragon.asm `stru_3CCC8` elapsed 8",
            ),
            (
                50,
                (0x040, 0x0A, 0x00, 0x20, 0x16),
                "ASM Dragon.asm `stru_3CCC8` elapsed 18",
            ),
            (
                40,
                (0x034, 0x0A, 0x00, 0x20, 0x15),
                "ASM Dragon.asm `stru_3CCC8` elapsed 28",
            ),
        ] {
            assert_child_frame(
                dragon_walking_breath_child_frame(&visuals, remaining),
                expected,
                label,
            );
        }

        for (remaining, expected, label) in [
            (
                0x0096,
                (0x070, 0x04, 0x05, 0x0C, 0x0D),
                "ASM Fireball.asm `stru_330C2` elapsed 0",
            ),
            (
                0x0090,
                (0x074, 0x04, 0x05, 0x0B, 0x0C),
                "ASM Fireball.asm `stru_330C2` elapsed 6",
            ),
            (
                0x008A,
                (0x078, 0x04, 0x05, 0x0B, 0x0B),
                "ASM Fireball.asm `stru_330C2` elapsed 12",
            ),
            (
                0x0084,
                (0x070, 0x04, 0x05, 0x0C, 0x0D),
                "ASM Fireball.asm `stru_330C2` elapsed 18",
            ),
        ] {
            assert_child_frame(
                fireball_trail_child_frame(&visuals, remaining),
                expected,
                label,
            );
        }
    }

    #[test]
    fn drips_surface_crawler_rom_frames_pin_start_run_and_finish_scripts() {
        let visuals = sample_animated_visual(sample_sprite_frame(0x5A));

        for (remaining, animation_elapsed, expected, label) in [
            (
                u16::MAX,
                31,
                (0x0BD, 0x08, 0x00, 0x0F, 0x04),
                "ASM Drips.asm loc_335D8 startup `stru_33FA6` elapsed 0",
            ),
            (
                u16::MAX,
                21,
                (0x0BF, 0x05, 0x00, 0x0F, 0x08),
                "ASM Drips.asm loc_335D8 startup `stru_33FA6` elapsed 10",
            ),
            (
                u16::MAX,
                11,
                (0x0C1, 0x07, 0x00, 0x0E, 0x0C),
                "ASM Drips.asm loc_335D8 startup `stru_33FA6` elapsed 20",
            ),
            (
                u16::MAX,
                0,
                (0x08D, 0x07, 0x00, 0x0F, 0x0F),
                "ASM Drips.asm loc_33630 mobile `stru_33F14` elapsed 0",
            ),
        ] {
            assert_child_frame(
                drips_surface_crawler_frame(&visuals, remaining, animation_elapsed),
                expected,
                label,
            );
        }

        for (elapsed, expected, label) in [
            (
                0,
                (0x08D, 0x07, 0x00, 0x0F, 0x0F),
                "ASM Drips.asm loc_33630 `stru_33F14` first C7B46 frame",
            ),
            (
                11,
                (0x08D, 0x07, 0x00, 0x0F, 0x0F),
                "ASM Drips.asm loc_33630 `stru_33F14` C7B46 duration end",
            ),
            (
                12,
                (0x089, 0x07, 0x00, 0x0D, 0x0F),
                "ASM Drips.asm loc_33630 `stru_33F14` switches to C7B3E",
            ),
            (
                24,
                (0x085, 0x07, 0x00, 0x0F, 0x10),
                "ASM Drips.asm loc_33630 `stru_33F14` switches to C7B36",
            ),
            (
                120,
                (0x0DC, 0x07, 0x00, 0x0D, 0x0E),
                "ASM Drips.asm loc_33630 `stru_33F14` short C7BE6 frame",
            ),
            (
                127,
                (0x0DC, 0x07, 0x00, 0x0D, 0x0E),
                "ASM Drips.asm loc_33630 `stru_33F14` C7BE6 duration end",
            ),
            (
                128,
                (0x085, 0x07, 0x00, 0x0F, 0x10),
                "ASM Drips.asm loc_33630 `stru_33F14` resumes C7B36",
            ),
            (
                152,
                (0x0DC, 0x07, 0x00, 0x0D, 0x0E),
                "ASM Drips.asm loc_33630 `stru_33F14` second C7BE6 frame",
            ),
            (
                160,
                (0x085, 0x07, 0x00, 0x0F, 0x10),
                "ASM Drips.asm loc_33630 `stru_33F14` final C7B36 frame",
            ),
            (
                171,
                (0x085, 0x07, 0x00, 0x0F, 0x10),
                "ASM Drips.asm loc_33630 `stru_33F14` cycle end",
            ),
            (
                184,
                (0x089, 0x07, 0x00, 0x0D, 0x0F),
                "ASM Drips.asm loc_33630 `stru_33F14` wraps modulo total duration",
            ),
        ] {
            assert_eq!(
                drips_surface_crawler_stru_33f14_frame(elapsed),
                expected,
                "{label}"
            );
        }

        for (remaining, expected, label) in [
            (
                60,
                (0x0C5, 0x07, 0x07, 0x10, 0x10),
                "ASM Drips.asm loc_338F8 finish `stru_33F06` elapsed 0",
            ),
            (
                56,
                (0x0C9, 0x0A, 0x09, 0x17, 0x16),
                "ASM Drips.asm loc_338F8 finish `stru_33F06` elapsed 4",
            ),
            (
                52,
                (0x0D2, 0x0B, 0x0C, 0x18, 0x18),
                "ASM Drips.asm loc_338F8 finish `stru_33F06` elapsed 8",
            ),
        ] {
            assert_child_frame(
                drips_surface_crawler_finish_frame(&visuals, remaining),
                expected,
                label,
            );
        }
    }

    #[test]
    fn dragon_walking_breath_facing_follows_parent_offset_loc_3d02a() {
        let dragon = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: WALKING_DRAGON_OBJECT_TYPE,
            },
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            hit_points: Some(1),
            ..sample_object_snapshot()
        };
        let trace = NativeTraceFrame {
            active_objects: vec![dragon.clone()],
            ..NativeTraceFrame::default()
        };

        assert_eq!(
            dragon_walking_breath_child_facing_loc_3d02a(
                &trace,
                &ActiveProjectileSnapshot {
                    x_pos: 72 << 16,
                    y_pos: 50 << 16,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            Direction::Right,
            "ASM Dragon.asm loc_3D02A/loc_3D070 places the left-mouth breath at parent x-$1C"
        );
        let multi_parent_trace = NativeTraceFrame {
            active_objects: vec![
                dragon.clone(),
                NativeObjectSnapshot {
                    x_pos: 44 << 16,
                    ..dragon.clone()
                },
            ],
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            dragon_walking_breath_child_facing_loc_3d02a(
                &multi_parent_trace,
                &ActiveProjectileSnapshot {
                    x_pos: 72 << 16,
                    y_pos: 50 << 16,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            Direction::Right,
            "ASM Dragon.asm loc_3D02A binds the breath to the first live Walking Dragon parent at x-$1C instead of falling through to a later x+$1C candidate"
        );
        assert_eq!(
            dragon_walking_breath_child_facing_loc_3d02a(
                &trace,
                &ActiveProjectileSnapshot {
                    x_pos: 128 << 16,
                    y_pos: 51 << 16,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            Direction::Left,
            "ASM Dragon.asm loc_3D02A/loc_3D070 accepts the one-pixel Y capture skew and maps parent x+$1C to left-facing native art"
        );
        assert_eq!(
            dragon_walking_breath_child_facing_loc_3d02a(
                &trace,
                &ActiveProjectileSnapshot {
                    x_pos: 128 << 16,
                    y_pos: 52 << 16,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            Direction::Right,
            "ASM Dragon.asm loc_3D02A parent lookup ignores breath children more than one pixel off the parent Y"
        );

        let dead_trace = NativeTraceFrame {
            active_objects: vec![NativeObjectSnapshot {
                hit_points: Some(0),
                ..dragon.clone()
            }],
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            dragon_walking_breath_child_facing_loc_3d02a(
                &dead_trace,
                &ActiveProjectileSnapshot {
                    x_pos: 128 << 16,
                    y_pos: 50 << 16,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            Direction::Right,
            "ASM Dragon.asm loc_3D02A only derives breath facing from a live Walking Dragon parent"
        );

        let wrong_kind_trace = NativeTraceFrame {
            active_objects: vec![NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x0C },
                ..dragon
            }],
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            dragon_walking_breath_child_facing_loc_3d02a(
                &wrong_kind_trace,
                &ActiveProjectileSnapshot {
                    x_pos: 128 << 16,
                    y_pos: 50 << 16,
                    ..ActiveProjectileSnapshot::default()
                },
            ),
            Direction::Right,
            "ASM Dragon.asm loc_3D02A should not borrow facing from a Flying Dragon object slot"
        );
    }

    #[test]
    fn boss_eye_scene_helpers_parse_debug_fields_and_exact_object_slots_loc_3931a() {
        let object = sample_boss_eye_object(40 << 16, 48 << 16, Some("7"), Some("0x002A"));
        assert_eq!(
            boss_eye_object_mapping_index_loc_37abc(&object),
            Some(7),
            "ASM sub_37ABC stores the selected `unk_C8488..unk_C84C0` mapping index in the waiting eye debug field"
        );
        assert_eq!(
            boss_eye_object_order_word_loc_393c8(&object),
            0x002A,
            "ASM sub_393C8/sub_FF6 sorts waiting eyes by `$A(a3)`, exported as a hex debug word"
        );

        let decimal_order = sample_boss_eye_object(40 << 16, 48 << 16, Some("3"), Some("42"));
        assert_eq!(
            boss_eye_object_order_word_loc_393c8(&decimal_order),
            42,
            "debug traces may serialise the ASM `$A(a3)` order word in decimal"
        );

        let invalid_mapping =
            sample_boss_eye_object(40 << 16, 48 << 16, Some("not-a-byte"), Some("bad-order"));
        assert_eq!(
            boss_eye_object_mapping_index_loc_37abc(&invalid_mapping),
            None
        );
        assert_eq!(
            boss_eye_object_order_word_loc_393c8(&invalid_mapping),
            0,
            "invalid or absent order words fall back to the ROM sub_FF6 default order"
        );

        let trace = NativeTraceFrame {
            active_objects: vec![object],
            ..NativeTraceFrame::default()
        };
        assert!(trace_has_boss_eye_object_slot_loc_3931a(
            &trace,
            40 << 16,
            48 << 16
        ));
        assert!(
            !trace_has_boss_eye_object_slot_loc_3931a(&trace, 41 << 16, 48 << 16),
            "ASM loc_3931A suppression requires the exact waiting-eye X slot"
        );
        assert!(
            !trace_has_boss_eye_object_slot_loc_3931a(&trace, 40 << 16, 49 << 16),
            "ASM loc_3931A suppression requires the exact waiting-eye Y slot"
        );

        let no_mapping_trace = NativeTraceFrame {
            active_objects: vec![sample_boss_eye_object(40 << 16, 48 << 16, None, Some("0"))],
            ..NativeTraceFrame::default()
        };
        assert!(
            !trace_has_boss_eye_object_slot_loc_3931a(&no_mapping_trace, 40 << 16, 48 << 16),
            "only ASM sub_37ABC-mapped `$21` waiting-eye slots suppress duplicate satellite projectiles"
        );

        let mut wrong_kind = sample_boss_eye_object(40 << 16, 48 << 16, Some("2"), Some("0"));
        wrong_kind.kind = NativeObjectKind::Enemy { object_type: 0x20 };
        let wrong_kind_trace = NativeTraceFrame {
            active_objects: vec![wrong_kind],
            ..NativeTraceFrame::default()
        };
        assert!(
            !trace_has_boss_eye_object_slot_loc_3931a(&wrong_kind_trace, 40 << 16, 48 << 16),
            "only ASM loc_3931A `$21` Boss Eye objects participate in duplicate suppression"
        );
    }

    #[test]
    fn boss_eye_waiting_objects_render_sorted_scene_markers_sub_393c8() {
        let visuals = sample_level_visuals(1);
        let high_order = sample_boss_eye_object(100 << 16, 72 << 16, Some("2"), Some("0x0020"));
        let low_order = sample_boss_eye_object(80 << 16, 64 << 16, Some("5"), Some("0x0010"));
        let trace = NativeTraceFrame {
            camera_x_pos: 8 << 16,
            camera_y_pos: 4 << 16,
            active_objects: vec![high_order, low_order],
            ..NativeTraceFrame::default()
        };

        let scene =
            build_game_scene_filtered(&trace, false, &visuals, None, LayerFilter::solo("enemies"));
        let rects = scene_rects(&scene.commands);

        assert_eq!(
            rects,
            vec![
                (64, 52, 16, 16, [120, 200, 255]),
                (66, 62, 4, 4, [255, 255, 255]),
                (84, 60, 16, 16, [120, 200, 255]),
                (94, 62, 4, 4, [255, 255, 255]),
            ],
            "ASM loc_3931A waiting Boss Eye objects render after sub_393C8/sub_FF6 `$A(a3)` ordering, using sub_37ABC mapping highlights"
        );
    }

    #[test]
    fn boss_eye_satellites_sort_without_moving_other_projectiles_sub_ff6() {
        let mut projectiles = vec![
            sample_boss_eye_projectile(ActiveProjectileKind::BossEyeSatellite, 40, 48, 2, 0x0020),
            sample_boss_eye_projectile(ActiveProjectileKind::Generic, 96, 48, 0, 0),
            sample_boss_eye_projectile(ActiveProjectileKind::BossEyeSatellite, 56, 48, 6, 0x0010),
            sample_boss_eye_projectile(ActiveProjectileKind::BossEyeAttack, 72, 48, 1, 0),
        ];

        order_boss_eye_satellites_sub_ff6(&mut projectiles);

        assert_eq!(
            (
                projectiles[0].kind,
                projectiles[0].x_pos,
                projectiles[0].boss_eye_sprite_mapping_index,
                projectiles[0].boss_eye_sprite_order_word_a,
            ),
            (ActiveProjectileKind::BossEyeSatellite, 56 << 16, 6, 0x0010),
            "ASM sub_FF6 reorders only Boss Eye satellite GfxObjects by `$A(a3)`"
        );
        assert_eq!(
            (projectiles[1].kind, projectiles[1].x_pos),
            (ActiveProjectileKind::Generic, 96 << 16),
            "ASM sub_FF6 satellite ordering must not consume unrelated projectile slots"
        );
        assert_eq!(
            (
                projectiles[2].kind,
                projectiles[2].x_pos,
                projectiles[2].boss_eye_sprite_mapping_index,
                projectiles[2].boss_eye_sprite_order_word_a,
            ),
            (ActiveProjectileKind::BossEyeSatellite, 40 << 16, 2, 0x0020)
        );
        assert_eq!(
            (projectiles[3].kind, projectiles[3].x_pos),
            (ActiveProjectileKind::BossEyeAttack, 72 << 16),
            "Boss Eye attack markers are rendered in trace order; only satellites mirror sub_FF6 sorting here"
        );
    }

    #[test]
    fn boss_eye_satellite_render_suppresses_matching_waiting_object_loc_3931a() {
        let visuals = sample_level_visuals(1);
        let trace = NativeTraceFrame {
            camera_x_pos: 8 << 16,
            camera_y_pos: 4 << 16,
            active_objects: vec![sample_boss_eye_object(
                40 << 16,
                48 << 16,
                Some("3"),
                Some("0x0010"),
            )],
            active_projectile_details: vec![
                sample_boss_eye_projectile(
                    ActiveProjectileKind::BossEyeSatellite,
                    40,
                    48,
                    3,
                    0x0010,
                ),
                sample_boss_eye_projectile(
                    ActiveProjectileKind::BossEyeSatellite,
                    56,
                    48,
                    6,
                    0x0020,
                ),
                sample_boss_eye_projectile(ActiveProjectileKind::BossEyeAttack, 72, 48, 1, 0),
            ],
            ..NativeTraceFrame::default()
        };

        let scene =
            build_game_scene_filtered(&trace, false, &visuals, None, LayerFilter::solo("effects"));
        let rects = scene_rects(&scene.commands);

        assert_eq!(
            rects,
            vec![
                (40, 36, 16, 16, [120, 200, 255]),
                (46, 47, 4, 4, [255, 255, 255]),
                (56, 36, 16, 16, [120, 200, 255]),
                (62, 37, 4, 4, [255, 255, 255]),
            ],
            "ASM loc_3931A waiting-eye object slots replace matching Boss Eye satellite projectile markers, while unmatched satellites and attacks still render"
        );
    }

    #[test]
    fn floating_prize_diamond_visibility_and_frame_timing_follow_loc_10396_103b2() {
        use wildside_core::terrain::PrizeContent;

        assert!(
            floating_prize_sprite_visible_loc_103b2(PrizeContent::Diamond, 0x00C9),
            "ASM PrizeDiamond_Init loc_10396 keeps the normal diamond script visible before the $00C8 blink switch"
        );
        assert!(
            floating_prize_sprite_visible_loc_103b2(PrizeContent::Diamond, 0x00C8),
            "ASM PrizeDiamond_Init loc_103B2 starts the blink script with a visible frame at d3=$00C8"
        );
        assert!(
            !floating_prize_sprite_visible_loc_103b2(PrizeContent::Diamond, 0x00C5),
            "ASM PrizeDiamond_Init loc_103B2 alternates to a null frame after three blink ticks"
        );
        assert!(
            floating_prize_sprite_visible_loc_103b2(PrizeContent::Diamond, 0x00C2),
            "ASM PrizeDiamond_Init loc_103B2 returns to the next visible frame after six blink ticks"
        );
        assert!(
            floating_prize_sprite_visible_loc_103b2(PrizeContent::Clock, 0),
            "non-diamond prize scripts do not use the loc_103B2 null-frame blink table"
        );
        assert!(
            floating_prize_sprite_visible_loc_103b2(PrizeContent::Clock, 0x00C5),
            "non-diamond prize scripts must not enter the diamond-only loc_103B2 blink guard"
        );
        assert!(
            !floating_prize_sprite_visible_loc_103b2(PrizeContent::Diamond, 0x00C4),
            "ASM PrizeDiamond_Init loc_103B2 blink parity comes from elapsed/3 script slots, not raw elapsed ticks"
        );

        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Diamond, 0x01F4, 99, 3, 4),
            0,
            "ASM PrizeDiamond_Init loc_10396 uses local remaining-frame time, not global frame index"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Diamond, 0x01EB, 0, 3, 4),
            3,
            "ASM PrizeDiamond_Init loc_10396 advances the normal diamond frame every frame_duration ticks"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Diamond, 0x01EB, 0, 4, 5),
            2,
            "ASM PrizeDiamond_Init loc_10396 divides elapsed ticks by frame_duration before wrapping normal diamond frames"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Diamond, 0x00C8, 99, 3, 4),
            0,
            "ASM PrizeDiamond_Init loc_103B2 restarts visible/null blink frame selection at d3=$00C8"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Diamond, 0x00C8, 99, 7, 5),
            0,
            "ASM PrizeDiamond_Init loc_103B2 owns the exact d3=$00C8 boundary instead of the normal loc_10396 script"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Diamond, 0x00BC, 0, 3, 4),
            2,
            "ASM PrizeDiamond_Init loc_103B2 advances visible diamond frames every two script slots"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Diamond, 0x00BE, 0, 3, 4),
            1,
            "ASM PrizeDiamond_Init loc_103B2 divides by frame_duration*2 for visible/null pairs"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Clock, 0, 12, 3, 4),
            0,
            "ordinary prize visuals use global trace frame timing"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Clock, 0, 14, 5, 6),
            2,
            "ordinary prize visuals divide the global trace frame by frame_duration before wrapping"
        );
        assert_eq!(
            floating_prize_frame_index_loc_10396(PrizeContent::Clock, 0, 12, 3, 0),
            0,
            "empty visual families keep the renderer in bounds"
        );
    }

    #[test]
    fn floating_prize_worker_effect_dispatch_and_sprites_follow_prize_scripts() {
        use wildside_core::terrain::PrizeContent;

        let mut visuals = sample_level_visuals(1);
        visuals.hud.diamond_base_tile_index = 0x06E0;
        visuals.hud.diamond_tiles = sample_tile_sheet_with_pixel(32, 0x44);
        visuals.hud.digit_base_tile_index = 0x06BA;
        visuals.hud.digits = sample_tile_sheet_with_pixel(4, 0x55);

        let mut commands = Vec::new();
        assert!(
            !render_floating_prize_worker_effect(
                &mut commands,
                &visuals,
                PrizeContent::TenDiamonds,
                80,
                96,
                u16::from(PRIZE_GENERIC_LIFETIME_LOW_BYTE + 1),
            ),
            "ASM prize worker effects only take over during the low-byte worker lifetime window"
        );
        assert!(commands.is_empty());

        assert!(render_floating_prize_worker_effect(
            &mut commands,
            &visuals,
            PrizeContent::TenDiamonds,
            80,
            96,
            u16::from(PRIZE_GENERIC_LIFETIME_LOW_BYTE),
        ));
        let DrawCommand::Sprite {
            x,
            y,
            tile_sheet,
            tiles,
            ..
        } = commands.single()
        else {
            panic!("ASM Prize10Diamonds_Init loc_108EE should draw the first HUD diamond worker");
        };
        assert_eq!((*x, *y), (78, 86));
        assert_eq!(tile_sheet.tiles[0].pixels[0], 0x44);
        assert_eq!(tiles[0].tile.tile_index, 0x000D);

        commands.clear();
        assert!(render_floating_prize_worker_effect(
            &mut commands,
            &visuals,
            PrizeContent::TenDiamonds,
            80,
            96,
            u16::from(PRIZE_GENERIC_LIFETIME_LOW_BYTE - 1),
        ));
        assert_eq!(
            commands.len(),
            2,
            "ASM Prize10Diamonds_Init loc_108EE scans the previous moving worker and the new frame-1 worker"
        );
        let DrawCommand::Sprite { x, y, .. } = &commands[0] else {
            panic!(
                "ASM Prize10Diamonds_Init loc_108EE should keep drawing the moving frame-0 worker"
            );
        };
        assert_eq!(
            (*x, *y),
            (75, 81),
            "ASM Prize10Diamonds_Init loc_108EE applies the worker FP velocity before sprite-origin placement"
        );

        commands.clear();
        assert!(render_ten_diamonds_worker_effect(
            &mut commands,
            &visuals,
            80,
            96,
            8,
        ));
        assert_eq!(
            commands.len(),
            3,
            "ASM loc_10984 clears the oldest worker at countdown $70 while younger workers remain visible"
        );

        commands.clear();
        assert!(
            !render_ten_diamonds_worker_effect(&mut commands, &visuals, 80, 96, 40),
            "ASM loc_10984 clears ten-diamond workers after the second burst's visible countdown window"
        );
        assert!(commands.is_empty());

        assert!(render_floating_prize_worker_effect(
            &mut commands,
            &visuals,
            PrizeContent::Points10000,
            80,
            96,
            u16::from(PRIZE_GENERIC_LIFETIME_LOW_BYTE),
        ));
        let DrawCommand::Sprite {
            x,
            y,
            tile_sheet,
            tiles,
            ..
        } = commands.single()
        else {
            panic!("ASM Prize10000Points_Init loc_10B00 should draw the first star worker");
        };
        assert_eq!((*x, *y), (76, 84));
        assert_eq!(tile_sheet.tiles[0].pixels[0], 0x55);
        assert_eq!(tiles[0].tile.tile_index, 1);

        commands.clear();
        assert!(render_10000_points_worker_effect(
            &mut commands,
            &visuals,
            80,
            96,
            5,
        ));
        assert_eq!(
            commands.len(),
            2,
            "ASM Prize10000Points_Init loc_10B00 spawns the second star worker at effect age 5"
        );

        commands.clear();
        assert!(render_10000_points_worker_effect(
            &mut commands,
            &visuals,
            80,
            96,
            150,
        ));
        assert_eq!(
            commands.len(),
            4,
            "ASM Prize10000Points_Init loc_10B00 maps spawn frame 20 to slot offset $10 so the fifth star deletes after exiting left"
        );

        assert!(
            prize_10000_points_star_visible_step_loc_10b74(0, 0x0078, 80, 96, true, 2).is_some(),
            "ASM loc_10B74 keeps a fresh 10000-point star worker visible"
        );
        assert!(
            prize_10000_points_star_visible_step_loc_10b74(0x0010, 0x0010, -0x001B, 96, true, 1,)
                .is_none(),
            "ASM loc_10B74 deletes the fifth star worker once its replayed screen X exits left"
        );

        commands.clear();
        assert!(
            !render_floating_prize_worker_effect(
                &mut commands,
                &visuals,
                PrizeContent::Clock,
                80,
                96,
                u16::from(PRIZE_GENERIC_LIFETIME_LOW_BYTE),
            ),
            "ordinary floating prizes stay on their sprite/fallback render path"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn heady_metal_death_wobble_uses_loc_37708_tiers_and_phase() {
        let mut visuals = sample_level_visuals(1);
        add_heady_metal_death_effect_visuals(&mut visuals);

        for (remaining, frame_index, amplitude, label) in [
            (
                HEADYMETAL_WOBBLE_STAGE_1 + 1,
                0,
                0x30D40 >> 16,
                "first tier",
            ),
            (
                HEADYMETAL_WOBBLE_STAGE_1,
                2,
                0x493E0 >> 16,
                "$015E boundary",
            ),
            (
                HEADYMETAL_WOBBLE_STAGE_2,
                0,
                0x61A80 >> 16,
                "$00FA boundary",
            ),
            (
                HEADYMETAL_WOBBLE_STAGE_3,
                2,
                0x7A120 >> 16,
                "$0096 boundary",
            ),
            (
                HEADYMETAL_WOBBLE_STAGE_4,
                0,
                0x927C0 >> 16,
                "$0064 boundary",
            ),
        ] {
            let trace = NativeTraceFrame {
                frame_index,
                camera_x_pos: 8 << 16,
                camera_y_pos: 4 << 16,
                ..NativeTraceFrame::default()
            };
            let mut commands = Vec::new();

            push_heady_metal_death_wobble(
                &mut commands,
                &trace,
                &visuals,
                100 << 16,
                80 << 16,
                remaining,
            );

            let phase = if frame_index & 0x2 == 0 { -1 } else { 1 };
            assert_eq!(
                sprite_snapshots(&commands),
                expected_heady_metal_wobble_snapshots(92, 76, amplitude, phase),
                "ASM sub_37708/sub_3787C/sub_378A2 {label} should render from the ROM countdown tier and Frame_Counter bit-1 phase"
            );
        }
    }

    #[test]
    fn heady_metal_death_split_pieces_render_loc_378d6_worker_positions() {
        let mut visuals = sample_level_visuals(1);
        add_heady_metal_death_effect_visuals(&mut visuals);
        let trace = NativeTraceFrame {
            camera_x_pos: 8 << 16,
            camera_y_pos: 4 << 16,
            heady_metal_death_splits: vec![heady_metal_split_trace(vec![
                heady_metal_piece(0xFB72, 100, 120),
                heady_metal_piece(0xFB76, 144, 136),
            ])],
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_heady_metal_death_split_pieces(&mut commands, &trace, &visuals);

        assert_eq!(
            sprite_snapshots(&commands),
            vec![(92, 108, 0x21, false), (136, 124, 0x21, false)],
            "ASM sub_378C8 loc_378D6 renders the cached split workers from traced post-impulse positions"
        );
    }

    #[test]
    fn effects_scene_prefers_heady_metal_split_pieces_over_wobble_loc_378c8() {
        let mut visuals = sample_level_visuals(1);
        add_heady_metal_death_effect_visuals(&mut visuals);
        let trace = NativeTraceFrame {
            game_mode: GameMode::InGame,
            game_mode_value: GameMode::InGame.as_disassembly_value(),
            camera_x_pos: 8 << 16,
            camera_y_pos: 4 << 16,
            enemy_death_effects: vec![(100 << 16, 80 << 16, HEADYMETAL_WOBBLE_STAGE_1 + 1)],
            heady_metal_death_splits: vec![heady_metal_split_trace(vec![
                heady_metal_piece(0xFB72, 100, 120),
                heady_metal_piece(0xFB76, 144, 136),
            ])],
            ..NativeTraceFrame::default()
        };

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            None,
            LayerFilter::solo("effects"),
        );

        assert_eq!(
            sprite_snapshots(&scene.commands),
            vec![(92, 108, 0x21, false), (136, 124, 0x21, false)],
            "ASM sub_378C8 loc_378D6 takes over once split-worker traces exist; the earlier sub_37708 wobble worker must not render alongside them"
        );
    }

    #[test]
    fn effect_enemy_sprite_skips_missing_or_empty_visual_family() {
        let mut visuals = sample_level_visuals(1);
        let mut commands = Vec::new();

        push_effect_enemy_sprite(
            &mut commands,
            &visuals,
            0x21,
            92,
            116,
            Direction::Right,
            false,
        );
        assert!(
            commands.is_empty(),
            "ASM sub_378C8 loc_378D6 split pieces require loaded enemy art before native can emit a sprite"
        );

        visuals.enemies.families.insert(
            0x21,
            AnimatedSpriteVisual {
                palette: sample_palette(),
                extra_palettes: Vec::new(),
                frames: Vec::new(),
                frame_duration: 1,
            },
        );
        push_effect_enemy_sprite(
            &mut commands,
            &visuals,
            0x21,
            92,
            116,
            Direction::Right,
            false,
        );
        assert!(
            commands.is_empty(),
            "empty native enemy visual families stay render-no-op instead of inventing degraded Heady split art"
        );
    }

    #[test]
    fn floating_prize_visual_lookup_and_fallback_colours_follow_content_specs() {
        use wildside_core::terrain::PrizeContent;

        let mut visuals = sample_level_visuals(1);
        visuals.floating_prizes.diamond = sample_animated_visual(sample_sprite_frame(0xD1));
        visuals.floating_prizes.helmets.insert(
            PrizeContent::Berzerker.to_id(),
            sample_animated_visual(sample_sprite_frame(0xB8)),
        );
        visuals.floating_prizes.icons.insert(
            PrizeContent::Clock.to_id(),
            sample_animated_visual(sample_sprite_frame(0xC1)),
        );

        let diamond_spec = floating_item_graphics_spec(PrizeContent::Diamond.to_id())
            .expect("ASM PrizeDiamond_Init graphics spec");
        assert_eq!(diamond_spec.object_meta, 0x0064);
        assert_eq!(diamond_spec.palette_line, 0);
        assert_eq!(diamond_spec.animation_script_addr, 0x0001_0D6E);
        assert!(floating_item_graphics_spec(PrizeContent::Points10000.to_id()).is_none());

        assert_eq!(
            floating_prize_visuals(&visuals, PrizeContent::Diamond.to_id())
                .expect("diamond visuals")
                .frames[0]
                .tile_sheet
                .tiles[0]
                .pixels[0],
            0xD1,
            "ASM PrizeDiamond_Init selects the dedicated diamond visual family"
        );
        assert_eq!(
            floating_prize_visuals(&visuals, PrizeContent::Berzerker.to_id())
                .expect("helmet visuals")
                .frames[0]
                .tile_sheet
                .tiles[0]
                .pixels[0],
            0xB8,
            "ASM sub_10CB0 helmet prizes route through helmet visuals"
        );
        assert_eq!(
            floating_prize_visuals(&visuals, PrizeContent::Clock.to_id())
                .expect("clock visuals")
                .frames[0]
                .tile_sheet
                .tiles[0]
                .pixels[0],
            0xC1,
            "ASM PrizeClock_Init selects the icon visual family"
        );
        assert!(floating_prize_visuals(&visuals, PrizeContent::TenDiamonds.to_id()).is_none());

        assert_eq!(
            floating_item_colour(PrizeContent::Diamond, Some(diamond_spec)),
            [100, 200, 255]
        );
        let helmet_spec = floating_item_graphics_spec(PrizeContent::Berzerker.to_id())
            .expect("ASM helmet graphics spec");
        assert_eq!(helmet_spec.palette_line, 3);
        assert_eq!(
            floating_item_colour(PrizeContent::Berzerker, Some(helmet_spec)),
            [220, 220, 255],
            "ASM sub_10CB0 helmet fallback colour is a single neutral placeholder until ROM palettes are wired"
        );
        assert_eq!(
            floating_item_colour(
                PrizeContent::Berzerker,
                Some(wildside_core::FloatingPrizeGraphicsSpec {
                    palette_line: 0,
                    ..helmet_spec
                }),
            ),
            [255, 255, 100],
            "only ASM sub_10CB0 palette-line-3 helmet specs use the helmet placeholder"
        );
        assert_eq!(
            floating_item_colour(PrizeContent::Points10000, None),
            [255, 255, 100]
        );
        assert_eq!(
            floating_item_colour(PrizeContent::Ankh, None),
            [255, 255, 200]
        );
        assert_eq!(
            floating_item_colour(PrizeContent::Clock, None),
            [200, 255, 200]
        );
        assert_eq!(
            floating_item_colour(PrizeContent::Continue, None),
            [255, 255, 255]
        );
        assert_eq!(
            floating_item_colour(PrizeContent::TenDiamonds, None),
            [100, 200, 255]
        );
        assert_eq!(helmet_placeholder_colour(0x007C), [220, 220, 255]);
    }

    #[test]
    fn chunked_plane_b_row_debug_reports_loc_30744_provenance() {
        let trace = NativeTraceFrame {
            frame_index: 60,
            time_frames_loc_3038a: 174,
            level_skip_cheat_enabled: 1,
            camera_x_pos: 292 << 16,
            camera_y_pos: 0,
            ..NativeTraceFrame::default()
        };

        let mut visuals = sample_level_visuals(1);
        visuals.level.background_theme_id = 8;
        visuals.level.foreground_layout_bytes = vec![0; 80 * 14];
        visuals.level.foreground_layout =
            ForegroundLayout::parse(&visuals.level.foreground_layout_bytes, 4, 1)
                .expect("wide foreground layout");
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x08; 32],
            has_terminator: true,
        });
        visuals.level.background_layout_alias = Some(BackgroundLayoutAlias {
            alias_label: "BackgroundLayout_0E".to_string(),
            source_label: "BackgroundLayout_0C".to_string(),
            source_background_path: Some("background/0C.bin".to_string()),
            x_offset_tiles: 0x13,
            y_offset_tiles: 0,
        });
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0x13 + 49,
                y_tiles: 18,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 8,
                height_tiles: 1,
                tiles: (1..=8)
                    .map(|tile_index| BackgroundTile {
                        tile_index,
                        flip_x: false,
                    })
                    .collect(),
            }],
        });

        let debug = chunked_plane_b_row_debug_for_trace_loc_30744(&trace, &visuals, 292, 18)
            .expect("settled chunked Plane-B row debug");

        for expected in [
            "B18 sel=08 t=174 preload=114 first=0",
            "upload_px=28",
            "upload=d03<-s04",
            "upload_data=d11<-s12w005D",
            "changed_src=alias",
            "t88:c0009->000A:d50<-s50:2782",
            "words=",
            "2781,2782,2783,2784,2785,2786,2787",
        ] {
            assert!(
                debug.contains(expected),
                "ASM loc_30452/sub_30744/sub_307D8 diagnostics should expose `{expected}` for the #289 alias-row provenance: {debug}"
            );
        }

        let direct_trace = NativeTraceFrame {
            level_skip_cheat_enabled: 0,
            ..trace
        };
        let direct_debug =
            chunked_plane_b_row_debug_for_trace_loc_30744(&direct_trace, &visuals, 292, 18)
                .expect("direct chunked Plane-B row debug");
        assert!(
            direct_debug.contains("changed_src=direct"),
            "ASM loc_30744 only reads the loc_12824 alias source during level-skip settle; normal live row diagnostics keep the direct source even when an alias definition exists: {direct_debug}"
        );
    }

    #[test]
    fn load_ingame_camera_grid_rounds_player_start_loc_11f48() {
        assert_eq!(
            load_ingame_camera_grid_x_px_loc_11f48(Some(0x013F), 0x0280),
            0,
            "ASM Load_InGame loc_11F48 rounds PlayerStart_X down to a 320px screen grid before loc_2FFE8"
        );
        assert_eq!(
            load_ingame_camera_grid_x_px_loc_11f48(Some(0x0140), 0),
            0x0140,
            "ASM Load_InGame loc_11F48 keeps exact 320px X grid boundaries"
        );
        assert_eq!(
            load_ingame_camera_grid_x_px_loc_11f48(None, 0x03C0),
            0x03C0,
            "without a traced PlayerStart_X, native keeps the current camera X for the loc_2FFE8 preload"
        );
        assert_eq!(
            load_ingame_camera_grid_y_px_loc_11f48(Some(0x00DF)),
            Some(0),
            "ASM Load_InGame loc_11F48 rounds PlayerStart_Y down to a 224px screen grid before loc_2FFE8"
        );
        assert_eq!(
            load_ingame_camera_grid_y_px_loc_11f48(Some(0x02BF)),
            Some(0x02A0),
            "ASM Load_InGame loc_11F48 preserves the Y screen-grid quotient, not the raw player-start pixel"
        );
    }

    #[test]
    fn preload_scroll_slot_filter_ignores_ff_rows_loc_2ffe8() {
        assert_eq!(
            max_horiz_scroll_slot_loc_2ffe8(&[0x00, 0x08, 0x20]),
            4,
            "ASM loc_2FFE8 must build Horiz_Scroll_Data slots up to the largest real sub_307D8 bgscroll entry even when no `$FF` terminator row is present"
        );
        assert_eq!(
            max_horiz_scroll_slot_loc_2ffe8(&[0xFF, 0x08, 0xFF, 0x18]),
            3,
            "ASM sub_307D8 skips `$FF` bgscroll rows instead of treating them as Horiz_Scroll_Data slot `$1F`"
        );
        assert_eq!(
            max_horiz_scroll_slot_loc_2ffe8(&[0xFF, 0xFF]),
            0,
            "all-terminator preload profiles fall back to the base scroll slot"
        );
    }

    #[test]
    fn chunked_plane_b_direct_row_upload_uses_source_bias_loc_307d8() {
        let trace = NativeTraceFrame {
            frame_index: 0,
            time_frames_loc_3038a: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            ..NativeTraceFrame::default()
        };

        let mut visuals = sample_level_visuals(1);
        visuals.level.background_theme_id = 8;
        visuals.level.foreground_layout_bytes = vec![0; 80 * 14];
        visuals.level.foreground_layout =
            ForegroundLayout::parse(&visuals.level.foreground_layout_bytes, 4, 1)
                .expect("wide foreground layout");
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds: vec![0x10; 32],
            has_terminator: true,
        });
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![BackgroundPlacement {
                chunk_id: 0,
                x_tiles: 0,
                y_tiles: 0,
            }],
            chunks: vec![BackgroundChunk {
                width_tiles: 4,
                height_tiles: 1,
                tiles: [0x20, 0x21, 0x22, 0x23]
                    .into_iter()
                    .map(|tile_index| BackgroundTile {
                        tile_index,
                        flip_x: false,
                    })
                    .collect(),
            }],
        });

        let row = chunked_plane_b_row_words_for_trace_loc_30744(&trace, &visuals, 0, 0)
            .expect("direct chunked Plane-B row upload");
        assert_eq!(
            &row[0..4],
            &[
                Some((0x21, 1)),
                Some((0x22, 1)),
                Some((0x23, 1)),
                Some((0, 1)),
            ],
            "ASM sub_307D8 consumes the scroll-column source with the native +1 chunked background bias before streaming the Plane-B row"
        );
    }

    #[test]
    fn chunked_plane_b_settled_preload_replays_loc_2ffe8_rows() {
        let trace = NativeTraceFrame {
            frame_index: 174,
            time_frames_loc_3038a: 174,
            level_skip_cheat_enabled: 1,
            camera_x_pos: 292 << 16,
            camera_y_pos: 547 << 16,
            player_start_x: Some(448),
            player_start_y: Some(703),
            ..NativeTraceFrame::default()
        };

        let mut visuals = sample_level_visuals(1);
        visuals.level.background_theme_id = 8;
        visuals.level.foreground_layout_bytes = vec![0; 80 * 56];
        visuals.level.foreground_layout =
            ForegroundLayout::parse(&visuals.level.foreground_layout_bytes, 4, 4)
                .expect("issue #289-shaped foreground layout");
        let mut layer_speeds = vec![0xFF; 64];
        layer_speeds[8] = 0x08;
        layer_speeds[18] = 0x08;
        layer_speeds[23] = 0x00;
        layer_speeds[29] = 0x20;
        layer_speeds[40] = 0x08;
        visuals.level.background_scroll = Some(wildside_data::BgscrollProfile {
            layer_speeds,
            has_terminator: true,
        });
        visuals.level.background_layout_alias = Some(BackgroundLayoutAlias {
            alias_label: "BackgroundLayout_0E".to_string(),
            source_label: "BackgroundLayout_0C".to_string(),
            source_background_path: Some("background/0C.bin".to_string()),
            x_offset_tiles: 0x13,
            y_offset_tiles: 0,
        });
        visuals.background = BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
            placements: vec![
                BackgroundPlacement {
                    chunk_id: 0,
                    x_tiles: 0x13 + 49,
                    y_tiles: 18,
                },
                BackgroundPlacement {
                    chunk_id: 1,
                    x_tiles: 0x13 + 4,
                    y_tiles: 29,
                },
                BackgroundPlacement {
                    chunk_id: 1,
                    x_tiles: 0x13 + 40,
                    y_tiles: 29,
                },
                BackgroundPlacement {
                    chunk_id: 2,
                    x_tiles: 0x13 + 62,
                    y_tiles: 23,
                },
                BackgroundPlacement {
                    chunk_id: 3,
                    x_tiles: 0x13 + 45,
                    y_tiles: 40,
                },
                BackgroundPlacement {
                    chunk_id: 3,
                    x_tiles: 0x13 + 51,
                    y_tiles: 40,
                },
            ],
            chunks: vec![
                BackgroundChunk {
                    width_tiles: 8,
                    height_tiles: 1,
                    tiles: (1..=8)
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: false,
                        })
                        .collect(),
                },
                BackgroundChunk {
                    width_tiles: 6,
                    height_tiles: 1,
                    tiles: [0x19, 0x15, 0x16, 0x17, 0x18, 0x39]
                        .into_iter()
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: false,
                        })
                        .collect(),
                },
                BackgroundChunk {
                    width_tiles: 2,
                    height_tiles: 1,
                    tiles: [0x12, 0x11]
                        .into_iter()
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: true,
                        })
                        .collect(),
                },
                BackgroundChunk {
                    width_tiles: 6,
                    height_tiles: 1,
                    tiles: [0x45, 0x46, 0x41, 0x42, 0x43, 0x44]
                        .into_iter()
                        .map(|tile_index| BackgroundTile {
                            tile_index,
                            flip_x: false,
                        })
                        .collect(),
                },
            ],
        });

        let row = chunked_plane_b_row_words_for_trace_loc_30744(&trace, &visuals, 292, 18)
            .expect("settled chunked Plane-B row history");
        assert!(
            row[0..5].iter().all(|cell| *cell == Some((0, 0))),
            "ASM loc_2FFE8/sub_307D8 leaves B18 columns 0..4 as the old Plane-B `$0780` fill"
        );
        assert_eq!(
            row[5..49],
            [Some((0, 1)); 44],
            "ASM loc_2FFE8/sub_307D8 seeds B18 columns 5..48 with `$2780` chunk-zero cells"
        );
        let cloud_tail: Vec<u16> = (49..=55)
            .map(|column| row[column].expect("cloud tail should be populated").0)
            .collect();
        assert_eq!(
            cloud_tail,
            vec![1, 2, 3, 4, 5, 6, 7],
            "ASM sub_30744 then accumulates the loc_12824 alias-source cloud tail after the persisted loc_2FFE8 row seed"
        );

        let ocean_row = chunked_plane_b_row_words_for_trace_loc_30744(&trace, &visuals, 292, 29)
            .expect("settled chunked Plane-B ocean row history");
        assert_eq!(
            &ocean_row[4..10],
            &[
                Some((0x19, 1)),
                Some((0x15, 1)),
                Some((0x16, 1)),
                Some((0x17, 1)),
                Some((0x18, 1)),
                Some((0x39, 1)),
            ],
            "ASM loc_2FFE8/sub_307D8 seeds B29 from the settled Camera_X column, not the pre-adjusted player-start grid column"
        );
        assert_eq!(
            ocean_row[45],
            Some((0x39, 1)),
            "ASM Load_InGame loc_11F48 first preloads Plane B at the player-start camera grid; the later settled B29 upload leaves the old far-edge cell from that grid pass intact"
        );

        let flipped_row =
            chunked_plane_b_row_vdp_words_for_trace_loc_30744(&trace, &visuals, 292, 23)
                .expect("settled chunked Plane-B flipped row history");
        assert_eq!(
            &flipped_row[62..64],
            &[Some(0x2F92), Some(0x2F91)],
            "ASM unk_309B8 maps high-bit chunk bytes from loc_12848 into the VDP H-flip bit for B23"
        );

        let wrapped_row =
            chunked_plane_b_row_vdp_words_for_trace_loc_30744(&trace, &visuals, 292, 8)
                .expect("settled chunked Plane-B wrapped row history");
        assert_eq!(
            &wrapped_row[45..56],
            &[
                Some(0x27C5),
                Some(0x27C6),
                Some(0x27C1),
                Some(0x27C2),
                Some(0x27C3),
                Some(0x27C4),
                Some(0x27C5),
                Some(0x27C6),
                Some(0x27C1),
                Some(0x27C2),
                Some(0x27C3),
            ],
            "ASM sub_30744 starts live column writes at Camera_Y_pos>>5; with camera chunk 17, Plane-B row 8 is fed by source row 40, not source row 8"
        );
    }

    #[test]
    fn prize_block_wobble_frame_alternates_every_three_elapsed_frames_sub_1007a() {
        let visuals = sample_level_visuals(0x300);

        let cases = [
            (31, 0x000),
            (30, 0x000),
            (29, 0x000),
            (28, 0x009),
            (26, 0x009),
            (25, 0x000),
        ];

        for (remaining, expected_tile_base_index) in cases {
            let frame = prize_block_wobble_frame_sub_1007a(&visuals, remaining);
            assert_eq!(frame.origin_x, 0x00);
            assert_eq!(frame.origin_y, 0x10);
            assert_eq!(frame.width_px, 0x10);
            assert_eq!(frame.height_px, 0x10);
            assert_eq!(
                frame.tile_base_index, expected_tile_base_index,
                "ASM stru_101B6/stru_10296 remaining={remaining}"
            );
        }
    }

    #[test]
    fn prize_block_wobble_draw_clears_priority_and_uses_fixed_vram_frames_sub_1007a() {
        let visuals = sample_level_visuals(0x300);
        let mut commands = Vec::new();

        push_prize_block_wobble_sub_1007a(&mut commands, &visuals, 32, 48, 28);

        let DrawCommand::Sprite { x, y, tiles, .. } = commands.single() else {
            panic!("ASM sub_1007A should emit one wobble sprite");
        };
        assert_eq!((*x, *y), (32, 48));
        assert_eq!(tiles.len(), 4);
        assert_eq!(
            tiles
                .iter()
                .map(|tile| tile.tile.tile_index)
                .collect::<Vec<_>>(),
            vec![0x008, 0x009, 0x00A, 0x00B],
            "ASM stru_101B6 remaining=28 selects live VRAM copy slot `$229`; native renders the `$E102` source quad"
        );
        assert_eq!(
            tiles
                .iter()
                .map(|tile| (tile.offset_x, tile.offset_y))
                .collect::<Vec<_>>(),
            vec![(0, 0), (8, 0), (0, 8), (8, 8)]
        );
        assert!(
            tiles.iter().all(|tile| !tile.tile.priority),
            "ASM sub_1007A/loc_10200 low-priority block copy stays behind Scroll A"
        );

        let mut commands = Vec::new();
        push_prize_block_wobble_sub_1007a(&mut commands, &visuals, 32, 48, 31);

        let DrawCommand::Sprite { tiles, .. } = commands.single() else {
            panic!("ASM sub_1007A should emit one wobble sprite");
        };
        assert_eq!(
            tiles
                .iter()
                .map(|tile| tile.tile.tile_index)
                .collect::<Vec<_>>(),
            vec![0x008, 0x009, 0x00A, 0x00B],
            "ASM stru_101B6 remaining=31 selects live VRAM copy slot `$220`; native renders the `$E102` source quad"
        );
    }

    #[test]
    fn prize_block_spawn_flash_frame_pins_script_thresholds_and_sheet_guard_loc_102b8() {
        let visuals = sample_level_visuals(0x300);
        let cases = [
            (25, 0x02CF),
            (22, 0x02CF),
            (21, 0x02D8),
            (16, 0x02D8),
            (15, 0x02E1),
            (6, 0x02E1),
            (5, 0x02EA),
            (1, 0x02EA),
        ];

        for (remaining, expected_vram_tile) in cases {
            let frame = prize_block_spawn_flash_frame_loc_102b8(&visuals, remaining)
                .expect("flash frame available");
            assert_eq!(
                frame.tile_base_index,
                expected_vram_tile - BLOCK_TILE_VRAM_BASE,
                "ASM stru_102DE remaining={remaining}"
            );
            assert_eq!((frame.origin_x, frame.origin_y), (0x0C, 0x0C));
            assert_eq!((frame.width_px, frame.height_px), (0x18, 0x18));
        }

        let short_sheet = sample_level_visuals(usize::from(0x02EA - BLOCK_TILE_VRAM_BASE + 8));
        assert!(
            prize_block_spawn_flash_frame_loc_102b8(&short_sheet, 5).is_none(),
            "24x24 frame needs all nine tiles from the final `$2EA` base"
        );
    }

    #[test]
    fn push_prize_block_spawn_flash_reports_whether_rom_frame_was_available_loc_102b8() {
        let visuals = sample_level_visuals(0x300);
        let mut commands = Vec::new();

        assert!(push_prize_block_spawn_flash_loc_102b8(
            &mut commands,
            &visuals,
            64,
            72,
            25
        ));
        let DrawCommand::Sprite { x, y, tiles, .. } = commands.single() else {
            panic!("ASM loc_102B8 should emit one flash sprite");
        };
        assert_eq!((*x, *y), (64 - 0x0C, 72 - 0x0C));
        assert_eq!(tiles.len(), 9);

        let short_sheet = sample_level_visuals(usize::from(0x02CF - BLOCK_TILE_VRAM_BASE + 8));
        commands.clear();
        assert!(!push_prize_block_spawn_flash_loc_102b8(
            &mut commands,
            &short_sheet,
            64,
            72,
            25
        ));
        assert!(commands.is_empty());
    }

    #[test]
    fn kid_projectile_hit_spark_frames_follow_stru_1f40_and_fallback_loc_1f08() {
        let mut visuals = sample_level_visuals(0x300);
        visuals.block_tiles = sample_tile_sheet_with_pixel(0x300, 0x66);

        for (remaining, expected_vram_tile) in [
            (4, 0x02CF),
            (3, 0x02D8),
            (2, 0x02E1),
            (1, 0x02EA),
            (0, 0x02EA),
        ] {
            let mut commands = Vec::new();
            push_kid_projectile_hit_spark_loc_1f08(&mut commands, &visuals, 80, 96, remaining);

            let DrawCommand::Sprite {
                x,
                y,
                tile_sheet,
                palette,
                tiles,
            } = commands.single()
            else {
                panic!("ASM loc_1F08/stru_1F40 should emit the hit-spark sprite");
            };
            let expected_tile_base = expected_vram_tile - BLOCK_TILE_VRAM_BASE;
            assert_eq!(
                (*x, *y),
                (68, 84),
                "ASM loc_1F08 places the 24x24 mapping around `$44/$46`; remaining={remaining}"
            );
            assert_eq!(tiles.len(), 9);
            assert_eq!(tiles[0].offset_x, 0);
            assert_eq!(tiles[0].offset_y, 0);
            assert_eq!(tiles[8].offset_x, 16);
            assert_eq!(tiles[8].offset_y, 16);
            assert_eq!(
                tiles
                    .iter()
                    .map(|tile| tile.tile.tile_index)
                    .collect::<Vec<_>>(),
                (expected_tile_base..=expected_tile_base + 8).collect::<Vec<_>>(),
                "ASM stru_1F40 remaining={remaining} selects the `$2CF/$2D8/$2E1/$2EA` 3x3 block-art mapping"
            );
            assert!(
                tiles.iter().all(|tile| !tile.tile.priority
                    && tile.tile.palette_line == 0
                    && !tile.tile.flip_x
                    && !tile.tile.flip_y),
                "ASM loc_1F08 palette_line=3 is already represented by the foreground block palette in native frame assets"
            );
            assert_eq!(
                tile_sheet.tiles[usize::from(expected_tile_base)].pixels[0],
                0x66
            );
            assert_eq!(palette.colours, visuals.palettes.foreground.colours);
        }

        let final_tile_base = 0x02EA - BLOCK_TILE_VRAM_BASE;
        let short_sheet = sample_level_visuals(usize::from(final_tile_base + 8));
        let mut commands = Vec::new();
        push_kid_projectile_hit_spark_loc_1f08(&mut commands, &short_sheet, 80, 96, 1);

        let DrawCommand::Rect {
            x,
            y,
            width,
            height,
            colour,
        } = commands.single()
        else {
            panic!("ASM loc_1F08 missing block-art fallback should be a diagnostic rect");
        };
        assert_eq!(
            (*x, *y, *width, *height, *colour),
            (77, 93, 6, 6, [255, 240, 96])
        );
    }

    #[test]
    fn kid_projectile_hit_spark_effect_layer_applies_camera_and_uses_block_art_loc_1f08() {
        let mut visuals = sample_level_visuals(0x300);
        visuals.block_tiles = sample_tile_sheet_with_pixel(0x300, 0x77);
        let trace = NativeTraceFrame {
            camera_x_pos: 8 << 16,
            camera_y_pos: 4 << 16,
            kid_projectile_hit_sparks: vec![(100, 72, 3)],
            ..NativeTraceFrame::default()
        };

        let scene = build_game_scene_filtered(
            &trace,
            false,
            &visuals,
            None,
            LayerFilter {
                background: false,
                terrain: false,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: true,
                eol_score: false,
                hud: false,
            },
        );

        let DrawCommand::Sprite {
            x,
            y,
            tile_sheet,
            tiles,
            ..
        } = scene.commands.single()
        else {
            panic!("ASM loc_1F08 trace helper should render through the effects layer");
        };
        let expected_tile_base = 0x02D8 - BLOCK_TILE_VRAM_BASE;
        assert_eq!(
            (*x, *y),
            (80, 56),
            "ASM loc_1F08 copies `$44/$46`; frame rendering subtracts Camera_X/Y before sprite-origin placement"
        );
        assert_eq!(tiles[0].tile.tile_index, expected_tile_base);
        assert_eq!(tiles[8].tile.tile_index, expected_tile_base + 8);
        assert_eq!(
            tile_sheet.tiles[usize::from(expected_tile_base)].pixels[0],
            0x77
        );
    }

    #[test]
    fn block_bounce_kind_prefers_typed_trace_and_legacy_defaults_to_prize() {
        let mut trace = NativeTraceFrame {
            block_bounces: vec![(3, 4, 12, 0)],
            ..NativeTraceFrame::default()
        };

        assert_eq!(
            block_bounce_kind_for_trace(&trace, 3, 4),
            Some(BlockBounceTraceKind::Prize)
        );

        trace.block_bounce_details = vec![BlockBounceTrace {
            cell_x: 3,
            cell_y: 4,
            x_offset_px: 0,
            remaining: 12,
            y_offset_px: 0,
            kind: BlockBounceTraceKind::HiddenReveal,
        }];

        assert_eq!(
            block_bounce_kind_for_trace(&trace, 3, 4),
            Some(BlockBounceTraceKind::HiddenReveal)
        );
        assert_eq!(
            block_bounce_kind_for_trace(&trace, 2, 4),
            None,
            "typed traces must not fall back to unrelated legacy tuples"
        );

        trace.block_bounce_details[0].remaining = 0;
        assert_eq!(
            block_bounce_kind_for_trace(&trace, 3, 4),
            None,
            "typed traces keep expired bounce workers out of the render path"
        );
    }

    #[test]
    fn block_bounce_kind_legacy_requires_exact_live_cell_match() {
        let trace = NativeTraceFrame {
            block_bounces: vec![(3, 4, 12, 0), (3, 5, 0, 0)],
            ..NativeTraceFrame::default()
        };

        assert_eq!(
            block_bounce_kind_for_trace(&trace, 3, 4),
            Some(BlockBounceTraceKind::Prize)
        );
        assert_eq!(block_bounce_kind_for_trace(&trace, 4, 4), None);
        assert_eq!(block_bounce_kind_for_trace(&trace, 3, 5), None);
        assert_eq!(block_bounce_kind_for_trace(&trace, 3, 6), None);
    }

    #[test]
    fn prize_block_wobble_overrides_apply_camera_position_and_skip_non_prize_workers_sub_1007a() {
        let visuals = sample_level_visuals(0x300);
        let grid = wildside_core::terrain::BlockGrid::from_level_layout(
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0xE1, 0x02, 0x00, 0x00,
            ],
            3,
            2,
        );
        let trace = NativeTraceFrame {
            camera_x_pos: 8 << 16,
            camera_y_pos: 4 << 16,
            block_bounces: vec![(1, 1, 28, 0)],
            block_bounce_details: vec![BlockBounceTrace {
                cell_x: 1,
                cell_y: 1,
                x_offset_px: 0,
                remaining: 28,
                y_offset_px: 0,
                kind: BlockBounceTraceKind::Prize,
            }],
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_prize_block_wobble_overrides_sub_1007a(&mut commands, &trace, &visuals, Some(&grid));

        let DrawCommand::Sprite { x, y, .. } = commands.single() else {
            panic!("ASM sub_1007A should emit one source-position wobble sprite");
        };
        assert_eq!((*x, *y), (8, 12));

        let DrawCommand::Sprite { tiles, .. } = commands.single() else {
            panic!("ASM sub_1007A should emit one source-position wobble sprite");
        };
        assert_eq!(
            tiles
                .iter()
                .map(|tile| tile.tile.tile_index)
                .collect::<Vec<_>>(),
            vec![0x008, 0x009, 0x00A, 0x00B],
            "ASM sub_1007A/stru_101B6 uses copied `$220/$229` VRAM slots; native draws the copied `$E102` source quad"
        );

        let rewritten_grid = wildside_core::terrain::BlockGrid::from_level_layout(
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x6F, 0x00, 0x00, 0x00,
            ],
            3,
            2,
        );
        let mut commands = Vec::new();
        push_prize_block_wobble_overrides_sub_1007a(
            &mut commands,
            &trace,
            &visuals,
            Some(&rewritten_grid),
        );
        let DrawCommand::Sprite { tiles, .. } = commands.single() else {
            panic!("ASM sub_1007A should still draw the fixed sprite over rewritten `$6F00` cells");
        };
        assert_eq!(
            tiles
                .iter()
                .map(|tile| tile.tile.tile_index)
                .collect::<Vec<_>>(),
            vec![0x008, 0x009, 0x00A, 0x00B],
            "ASM sub_1007A/stru_101B6 ignores the `$6F00` low byte for wobble sprite art"
        );

        let shifted_trace = NativeTraceFrame {
            camera_x_pos: 8 << 16,
            camera_y_pos: 4 << 16,
            block_bounces: vec![(1, 1, 28, 1)],
            block_bounce_details: vec![BlockBounceTrace {
                cell_x: 1,
                cell_y: 1,
                x_offset_px: 0,
                remaining: 28,
                y_offset_px: 1,
                kind: BlockBounceTraceKind::Prize,
            }],
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();
        push_prize_block_wobble_overrides_sub_1007a(
            &mut commands,
            &shifted_trace,
            &visuals,
            Some(&grid),
        );
        let DrawCommand::Sprite { x, y, .. } = commands.single() else {
            panic!("ASM sub_1007A should emit shifted wobble sprites");
        };
        assert_eq!(
            (*x, *y),
            (8, 13),
            "ASM sub_1007A applies y_pos movement to the copied block GfxObject"
        );

        for bounce in [
            (1, 0, 0, 0, BlockBounceTraceKind::Prize),
            (1, 0, 28, 0, BlockBounceTraceKind::HiddenReveal),
        ] {
            let trace = NativeTraceFrame {
                block_bounces: vec![(bounce.0, bounce.1, bounce.2, bounce.3)],
                block_bounce_details: vec![BlockBounceTrace {
                    cell_x: bounce.0,
                    cell_y: bounce.1,
                    x_offset_px: 0,
                    remaining: bounce.2,
                    y_offset_px: bounce.3,
                    kind: bounce.4,
                }],
                ..NativeTraceFrame::default()
            };
            let mut commands = Vec::new();
            push_prize_block_wobble_overrides_sub_1007a(
                &mut commands,
                &trace,
                &visuals,
                Some(&grid),
            );
            assert!(commands.is_empty(), "skipped bounce case: {bounce:?}");
        }
    }

    #[test]
    fn prize_block_wobble_suppresses_foreground_skin_while_shifted_sub_1007a_issue324() {
        let mut visuals = sample_level_visuals(0x300);
        let mut foreground_layout_bytes = vec![0; 20 * 14];
        foreground_layout_bytes[1] = 7;
        visuals.level.foreground_layout =
            wildside_render::ForegroundLayout::parse(&foreground_layout_bytes, 1, 1)
                .expect("foreground layout");
        visuals.level.foreground_layout_bytes = foreground_layout_bytes;
        visuals.level.block_mapping = wildside_render::BlockVisualMapping::new(
            (0..8).map(|skin| sample_quad(0x040 + skin * 4)).collect(),
            (0..81)
                .map(|skin| sample_quad(BLOCK_TILE_VRAM_BASE + skin * 4))
                .collect(),
        )
        .expect("block mapping");

        let grid = wildside_core::terrain::BlockGrid::from_level_layout(
            &[0x00, 0x00, 0x6F, 0x00, 0x00, 0x00],
            3,
            1,
        );
        let trace = NativeTraceFrame {
            camera_x_pos: 0,
            camera_y_pos: 0,
            block_bounces: vec![(1, 0, 28, -6)],
            block_bounce_details: vec![BlockBounceTrace {
                cell_x: 1,
                cell_y: 0,
                x_offset_px: 0,
                remaining: 28,
                y_offset_px: -6,
                kind: BlockBounceTraceKind::Prize,
            }],
            ..NativeTraceFrame::default()
        };

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            Some(&grid),
            LayerFilter {
                background: false,
                terrain: true,
                block_overrides: false,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );

        assert!(
            !scene.commands.iter().any(|command| matches!(
                command,
                DrawCommand::IndexedTile {
                    x: 16,
                    y: 0,
                    tile,
                    ..
                } if (0x05C..=0x05F).contains(&tile.tile_index)
            )),
            "ASM sub_1007A owns active `$6Fxx` prize cells for the whole bob; native must not leak the immutable foreground skin, such as the row-12 diagonal forest tile, under a shifted copied P block"
        );
    }

    #[test]
    fn prize_block_wobble_sprite_stays_over_low_priority_terrain_sub_1007a_issue372() {
        let mut visuals = sample_level_visuals(0x300);
        visuals.foreground_tiles = sample_tile_sheet_with_pixel(8, 0x05);
        visuals.block_tiles = sample_tile_sheet_with_pixel(0x300, 0x0A);
        let mut foreground_layout_bytes = vec![0; 20 * 14];
        foreground_layout_bytes[1] = 1;
        visuals.level.foreground_layout =
            wildside_render::ForegroundLayout::parse(&foreground_layout_bytes, 1, 1)
                .expect("foreground layout");
        visuals.level.foreground_layout_bytes = foreground_layout_bytes;
        let low_priority_foreground_quad = TileQuad {
            tiles: [
                TileRef {
                    tile_index: FOREGROUND_TILE_VRAM_BASE,
                    priority: false,
                    ..sample_tile_ref(FOREGROUND_TILE_VRAM_BASE)
                },
                TileRef {
                    tile_index: FOREGROUND_TILE_VRAM_BASE + 1,
                    priority: false,
                    ..sample_tile_ref(FOREGROUND_TILE_VRAM_BASE + 1)
                },
                TileRef {
                    tile_index: FOREGROUND_TILE_VRAM_BASE + 2,
                    priority: false,
                    ..sample_tile_ref(FOREGROUND_TILE_VRAM_BASE + 2)
                },
                TileRef {
                    tile_index: FOREGROUND_TILE_VRAM_BASE + 3,
                    priority: false,
                    ..sample_tile_ref(FOREGROUND_TILE_VRAM_BASE + 3)
                },
            ],
        };
        visuals.level.block_mapping = wildside_render::BlockVisualMapping::new(
            vec![
                sample_quad(FOREGROUND_TILE_VRAM_BASE + 4),
                low_priority_foreground_quad,
            ],
            (0..81)
                .map(|skin| sample_quad(BLOCK_TILE_VRAM_BASE + skin * 4))
                .collect(),
        )
        .expect("block mapping");

        let grid = wildside_core::terrain::BlockGrid::from_level_layout(
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x6F, 0x00, 0x00, 0x00,
            ],
            3,
            2,
        );
        let trace = NativeTraceFrame {
            game_mode: GameMode::InGame,
            block_bounces: vec![(1, 1, 28, -6)],
            block_bounce_details: vec![BlockBounceTrace {
                cell_x: 1,
                cell_y: 1,
                x_offset_px: 0,
                remaining: 28,
                y_offset_px: -6,
                kind: BlockBounceTraceKind::Prize,
            }],
            ..NativeTraceFrame::default()
        };

        let scene = build_scene_filtered(
            &trace,
            false,
            Some(&visuals),
            Some(&grid),
            LayerFilter {
                background: false,
                terrain: true,
                block_overrides: true,
                platforms: false,
                enemies: false,
                player: false,
                effects: false,
                eol_score: false,
                hud: false,
            },
        );

        let mut pixels = BTreeSet::new();
        pixels.insert((16, 10));
        let diagnostic = sample_scene_pixel_provenance(&scene, &pixels)
            .into_iter()
            .next()
            .expect("pixel diagnostic");
        let sprite_colour = visuals.palettes.foreground.colours[0x0A];
        assert_eq!(
            diagnostic.final_rgb,
            [sprite_colour.red, sprite_colour.green, sprite_colour.blue],
            "ASM sub_1007A/stru_101B6 draws the copied prize block as a low-priority GfxObject; ordinary low-priority Scroll A terrain behind the shifted wobble must not erase it after priority has already been resolved"
        );
    }

    #[test]
    fn resolve_ingame_vram_tile_maps_absolute_vdp_ranges_to_local_sheets() {
        let mut visuals = sample_level_visuals(0x20);
        visuals.foreground_tiles = sample_tile_sheet_with_pixel(3, 0x11);
        visuals.block_tiles = sample_tile_sheet_with_pixel(5, 0x22);
        visuals.hud.digits = sample_tile_sheet_with_pixel(7, 0x33);
        visuals.hud.diamond_tiles = sample_tile_sheet_with_pixel(11, 0x44);

        let public_resolved =
            resolve_ingame_vram_tile(&visuals, sample_tile_ref(FOREGROUND_TILE_VRAM_BASE + 1))
                .expect("public wrapper should use the scene foreground palette");
        assert_eq!(public_resolved.tile.tile_index, 1);
        assert_eq!(
            public_resolved.palette.colours,
            visuals.palettes.foreground.colours
        );

        let cases = [
            (FOREGROUND_TILE_VRAM_BASE + 2, 2, 0x11),
            (BLOCK_TILE_VRAM_BASE + 4, 4, 0x22),
            (HUD_TILE_VRAM_BASE + 6, 6, 0x33),
            (HUD_DIAMOND_TILE_VRAM_BASE + 10, 10, 0x44),
        ];

        for (absolute_tile_index, expected_local_index, expected_pixel) in cases {
            let resolved = resolve_ingame_vram_tile_with_foreground_palette(
                &visuals,
                &sample_palette_with_first_word(0x0EEE),
                TileRef {
                    tile_index: absolute_tile_index,
                    palette_line: 3,
                    priority: true,
                    flip_x: true,
                    flip_y: true,
                },
            )
            .expect("absolute VDP tile should resolve inside native ingame ranges");

            assert_eq!(resolved.tile.tile_index, expected_local_index);
            assert_eq!(resolved.tile.palette_line, 3);
            assert!(resolved.tile.priority);
            assert!(resolved.tile.flip_x);
            assert!(resolved.tile.flip_y);
            assert_eq!(
                resolved.tile_sheet.tiles[usize::from(expected_local_index)].pixels[0],
                expected_pixel
            );
            assert_eq!(resolved.palette.colours[0], MdColour::from_word(0x0EEE));
        }
    }

    #[test]
    fn resolve_ingame_vram_tile_rejects_range_end_and_unknown_tiles() {
        let mut visuals = sample_level_visuals(0x20);
        visuals.foreground_tiles = sample_tile_sheet(1);
        visuals.block_tiles = sample_tile_sheet(1);
        visuals.hud.digits = sample_tile_sheet(1);
        visuals.hud.diamond_tiles = sample_tile_sheet(2);

        for tile_index in [
            FOREGROUND_TILE_VRAM_BASE - 1,
            FOREGROUND_TILE_VRAM_BASE + 1,
            BLOCK_TILE_VRAM_BASE - 1,
            BLOCK_TILE_VRAM_BASE + 1,
            HUD_TILE_VRAM_BASE - 1,
            HUD_TILE_VRAM_BASE + 1,
            HUD_DIAMOND_TILE_VRAM_BASE - 1,
            HUD_DIAMOND_TILE_VRAM_BASE + 2,
            HUD_DIAMOND_TILE_VRAM_BASE * 2 - 1,
        ] {
            assert!(
                resolve_ingame_vram_tile(&visuals, sample_tile_ref(tile_index)).is_none(),
                "tile {tile_index:#06X} must stay outside the one-tile test ranges"
            );
        }
    }

    #[test]
    fn storm_pattern_plane_converts_absolute_vdp_words_to_local_tile_indices_loc_14d32() {
        let mut visuals = sample_level_visuals(0x20);
        visuals.weather = Some(WeatherVisuals {
            palette: sample_palette(),
            tile_sheet: sample_tile_sheet_with_pixel(0x30, 0x55),
            vram_base_tile_index: 0x0380,
            shared_art: None,
        });
        let trace = NativeTraceFrame {
            storm_pattern_upload_loc_14d32: Some(
                wildside_core::StormPatternRuntimeUploadLoc14D32 {
                    source_asset_label: "unk_9B65C".to_owned(),
                    vram_tile_addend: 0x0380_0380,
                    skipped_header_longwords: 1,
                    vdp_command: 0x4000_0003,
                    written_longword_count: 4 * 8 * 8 * 4,
                },
            ),
            storm_scroll_transfer_loc_30110: Some(wildside_core::TransferScrollDataStormLoc30110 {
                plane_address_word: 0x8403,
                h_scroll_word_pairs: 0x00E0,
                first_h_scroll_word: 0,
                second_h_scroll_word: 11,
                first_vsram_word: 0,
                second_vsram_word: 13,
                weather_mode_byte: 0,
                next_storm_scroll_offset: 0,
            }),
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        assert!(push_storm_pattern_plane_loc_14d32(
            &mut commands,
            &trace,
            &visuals
        ));

        let DrawCommand::IndexedTile {
            x,
            y,
            tile,
            tile_sheet,
            ..
        } = commands.first_indexed_tile()
        else {
            panic!("ASM loc_14D32 storm plane should emit indexed tiles");
        };
        assert_eq!((*x, *y), (-3, -5));
        assert_eq!(tile.tile_index, 9);
        assert_eq!(
            tile_sheet.tiles[usize::from(tile.tile_index)].pixels[0],
            0x55
        );
        let indexed_tiles = commands.indexed_tiles();
        let visible_cols = FRAME_WIDTH.div_ceil(8) as usize + 1;
        assert_eq!(
            indexed_tiles.len(),
            visible_cols * (FRAME_HEIGHT.div_ceil(8) as usize + 1)
        );
        assert_eq!(
            indexed_tile_snapshot(indexed_tiles[1]),
            (5, -5, 0x000A, false)
        );
        assert_eq!(
            indexed_tile_snapshot(indexed_tiles[visible_cols]),
            (-3, 3, 0x000B, false)
        );
    }

    #[test]
    fn storm_pattern_plane_rejects_non_matching_loc_14d32_metadata() {
        fn valid_upload() -> wildside_core::StormPatternRuntimeUploadLoc14D32 {
            wildside_core::StormPatternRuntimeUploadLoc14D32 {
                source_asset_label: "unk_9B65C".to_owned(),
                vram_tile_addend: 0x0380_0380,
                skipped_header_longwords: 1,
                vdp_command: 0x4000_0003,
                written_longword_count: 4 * 8 * 8 * 4,
            }
        }

        fn valid_transfer() -> wildside_core::TransferScrollDataStormLoc30110 {
            wildside_core::TransferScrollDataStormLoc30110 {
                plane_address_word: 0x8403,
                h_scroll_word_pairs: 0x00E0,
                first_h_scroll_word: 0,
                second_h_scroll_word: 0,
                first_vsram_word: 0,
                second_vsram_word: 0,
                weather_mode_byte: 0,
                next_storm_scroll_offset: 0,
            }
        }

        let mut visuals = sample_level_visuals(0x20);
        visuals.weather = Some(WeatherVisuals {
            palette: sample_palette(),
            tile_sheet: sample_tile_sheet_with_pixel(0x30, 0x55),
            vram_base_tile_index: 0x0380,
            shared_art: None,
        });

        for (upload, transfer, label) in [
            (
                wildside_core::StormPatternRuntimeUploadLoc14D32 {
                    source_asset_label: "unknown".to_owned(),
                    ..valid_upload()
                },
                valid_transfer(),
                "source asset",
            ),
            (
                wildside_core::StormPatternRuntimeUploadLoc14D32 {
                    vram_tile_addend: 0x0380_037F,
                    ..valid_upload()
                },
                valid_transfer(),
                "tile addend",
            ),
            (
                wildside_core::StormPatternRuntimeUploadLoc14D32 {
                    skipped_header_longwords: 0,
                    ..valid_upload()
                },
                valid_transfer(),
                "skipped header",
            ),
            (
                wildside_core::StormPatternRuntimeUploadLoc14D32 {
                    vdp_command: 0x4000_0002,
                    ..valid_upload()
                },
                valid_transfer(),
                "VDP command",
            ),
            (
                wildside_core::StormPatternRuntimeUploadLoc14D32 {
                    written_longword_count: 4 * 8 * 8 * 4 - 1,
                    ..valid_upload()
                },
                valid_transfer(),
                "written longwords",
            ),
            (
                valid_upload(),
                wildside_core::TransferScrollDataStormLoc30110 {
                    plane_address_word: 0x8402,
                    ..valid_transfer()
                },
                "plane address",
            ),
            (
                valid_upload(),
                wildside_core::TransferScrollDataStormLoc30110 {
                    h_scroll_word_pairs: 0x00DF,
                    ..valid_transfer()
                },
                "HScroll count",
            ),
        ] {
            let trace = NativeTraceFrame {
                storm_pattern_upload_loc_14d32: Some(upload),
                storm_scroll_transfer_loc_30110: Some(transfer),
                ..NativeTraceFrame::default()
            };
            let mut commands = Vec::new();

            assert!(
                !push_storm_pattern_plane_loc_14d32(&mut commands, &trace, &visuals),
                "ASM loc_14D32 should reject mismatched {label} metadata"
            );
            assert!(
                commands.is_empty(),
                "ASM loc_14D32 rejected {label} metadata must not emit tiles"
            );
        }
    }

    #[test]
    fn special_effect_unknown_plane_converts_absolute_vdp_words_to_local_tile_indices_loc_14e42() {
        let mut visuals = sample_level_visuals(0x20);
        visuals.special_effect_unknown = Some(SpecialEffectUnknownVisuals {
            tile_sheet: sample_tile_sheet_with_pixel(0x20, 0x66),
            vram_base_tile_index: 0x0380,
        });
        let trace = NativeTraceFrame {
            camera_x_pos: 3 << 16,
            camera_y_pos: 5 << 16,
            special_effect_unknown_plane_fill_loc_14e42: Some(
                wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                    vdp_command: 0x4000_0003,
                    outer_pass_count: 8,
                    group_count_per_pass: 4,
                    inner_repeat_count: 16,
                    written_longword_count: 8 * 4 * 16 * 2,
                    foreground_plane_register:
                        SPECIAL_EFFECT_UNKNOWN_FOREGROUND_PLANE_REGISTER_LOC_14E42,
                    background_plane_register: 0x8400,
                },
            ),
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_special_effect_unknown_foreground_plane_loc_14e42(&mut commands, &trace, &visuals);

        let DrawCommand::IndexedTile {
            x,
            y,
            tile,
            tile_sheet,
            ..
        } = commands.first_indexed_tile()
        else {
            panic!("ASM loc_14E42 unknown foreground plane should emit indexed tiles");
        };
        assert_eq!((*x, *y), (-3, -5));
        assert_eq!(tile.tile_index, 0);
        assert!(tile.priority);
        assert_eq!(
            tile_sheet.tiles[usize::from(tile.tile_index)].pixels[0],
            0x66
        );
        let indexed_tiles = commands.indexed_tiles();
        let visible_cols = FRAME_WIDTH.div_ceil(8) as usize + 1;
        assert_eq!(
            indexed_tiles.len(),
            visible_cols * (FRAME_HEIGHT.div_ceil(8) as usize + 1)
        );
        assert_eq!(indexed_tile_snapshot(indexed_tiles[1]), (5, -5, 1, true));
        assert_eq!(
            indexed_tile_snapshot(indexed_tiles[visible_cols]),
            (-3, 3, 4, true)
        );
    }

    #[test]
    fn special_effect_unknown_plane_rejects_invalid_loc_14e42_fill_metadata() {
        fn valid_fill() -> wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
            wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                vdp_command: 0x4000_0003,
                outer_pass_count: 8,
                group_count_per_pass: 4,
                inner_repeat_count: 16,
                written_longword_count: 8 * 4 * 16 * 2,
                foreground_plane_register:
                    SPECIAL_EFFECT_UNKNOWN_FOREGROUND_PLANE_REGISTER_LOC_14E42,
                background_plane_register: 0x8400,
            }
        }

        let mut visuals = sample_level_visuals(0x20);
        visuals.special_effect_unknown = Some(SpecialEffectUnknownVisuals {
            tile_sheet: sample_tile_sheet_with_pixel(0x20, 0x66),
            vram_base_tile_index: 0x0380,
        });

        for (fill, label) in [
            (
                wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                    foreground_plane_register: 0x8403,
                    ..valid_fill()
                },
                "foreground plane register",
            ),
            (
                wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                    group_count_per_pass: 0,
                    ..valid_fill()
                },
                "group count",
            ),
            (
                wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                    inner_repeat_count: 0,
                    ..valid_fill()
                },
                "inner repeat count",
            ),
            (
                wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                    outer_pass_count: 0,
                    ..valid_fill()
                },
                "outer pass count",
            ),
        ] {
            let trace = NativeTraceFrame {
                special_effect_unknown_plane_fill_loc_14e42: Some(fill),
                ..NativeTraceFrame::default()
            };
            let mut commands = Vec::new();

            push_special_effect_unknown_foreground_plane_loc_14e42(&mut commands, &trace, &visuals);
            assert!(
                commands.is_empty(),
                "ASM loc_14E42 rejected {label} metadata must not emit tiles"
            );
        }
    }

    #[test]
    fn special_effect_unknown_plane_wraps_by_computed_loc_14e42_dimensions() {
        let mut visuals = sample_level_visuals(0x20);
        visuals.special_effect_unknown = Some(SpecialEffectUnknownVisuals {
            tile_sheet: sample_tile_sheet_with_pixel(0x20, 0x66),
            vram_base_tile_index: 0x0380,
        });
        let trace = NativeTraceFrame {
            special_effect_unknown_plane_fill_loc_14e42: Some(
                wildside_core::SpecialEffectUnknownPlaneFillRuntimeLoc14E42 {
                    vdp_command: 0x4000_0003,
                    outer_pass_count: 2,
                    group_count_per_pass: 3,
                    inner_repeat_count: 3,
                    written_longword_count: 2 * 3 * 3 * 2,
                    foreground_plane_register:
                        SPECIAL_EFFECT_UNKNOWN_FOREGROUND_PLANE_REGISTER_LOC_14E42,
                    background_plane_register: 0x8400,
                },
            ),
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_special_effect_unknown_foreground_plane_loc_14e42(&mut commands, &trace, &visuals);

        let indexed_tiles = commands.indexed_tiles();
        let visible_cols = FRAME_WIDTH.div_ceil(8) as usize + 1;
        assert_eq!(
            indexed_tile_snapshot(indexed_tiles[7]),
            (56, 0, 3, true),
            "ASM loc_14E42 inner repeat count controls the wrapped plane width"
        );
        assert_eq!(
            indexed_tile_snapshot(indexed_tiles[5 * visible_cols]),
            (0, 40, 8, true),
            "ASM loc_14E42 outer pass and group counts control the wrapped plane height"
        );
    }

    #[test]
    fn eol_foreground_reveal_preserves_rom_attr_palette_line_loc_c21e_sub_c254() {
        let mut visuals = sample_level_visuals(1);
        visuals.eol_hologram = Some(sample_eol_hologram_visuals());
        let mut layout = vec![0; 3 * 4 * 2];
        layout[(3 * 3 + 2) * 2] = 0x80;
        let grid = wildside_core::terrain::BlockGrid::from_level_layout(&layout, 3, 4);
        let trace = NativeTraceFrame {
            level_complete: true,
            camera_x_pos: 32 << 16,
            camera_y_pos: 48 << 16,
            level_complete_elapsed_frames: wildside_core::EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE
                + wildside_core::EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE
                + 1,
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_eol_foreground_reveal_tiles(&mut commands, &trace, &visuals, Some(&grid));

        let DrawCommand::Rect { x, y, .. } = commands[0] else {
            panic!("ASM loc_C21E/sub_C254 first pass should clear the foreground cell");
        };
        assert_eq!((x, y), (0, 0));

        let indexed_tiles = commands.indexed_tiles();
        assert_eq!(
            indexed_tiles.len(),
            20,
            "ASM loc_C21E/sub_C254 consumes exactly one reveal-frame budget from the camera-derived cursor"
        );
        let DrawCommand::IndexedTile { x, y, tile, .. } = indexed_tiles[0] else {
            panic!("ASM loc_C21E/sub_C254 should emit Scroll A replacement tiles");
        };
        assert_eq!((*x, *y), (0, 0));
        assert_eq!(tile.tile_index, 0x0417);
        assert_eq!(
            tile.palette_line, 2,
            "ASM loc_C21E/sub_C254 emits `$4417` for block cells; native must preserve its VDP palette bits"
        );
        let positions = indexed_tiles
            .iter()
            .take(4)
            .map(|command| match command {
                DrawCommand::IndexedTile { x, y, .. } => (*x, *y),
                _ => unreachable!("filtered indexed tiles"),
            })
            .collect::<Vec<_>>();
        assert_eq!(positions, vec![(0, 0), (8, 0), (0, 8), (8, 8)]);
    }

    trait SingleCommand {
        fn single(&self) -> &DrawCommand;
    }

    impl SingleCommand for Vec<DrawCommand> {
        fn single(&self) -> &DrawCommand {
            assert_eq!(self.len(), 1);
            &self[0]
        }
    }

    trait FirstIndexedTile {
        fn first_indexed_tile(&self) -> &DrawCommand;
    }

    impl FirstIndexedTile for Vec<DrawCommand> {
        fn first_indexed_tile(&self) -> &DrawCommand {
            self.iter()
                .find(|command| matches!(command, DrawCommand::IndexedTile { .. }))
                .expect("indexed tile command")
        }
    }

    trait IndexedTiles {
        fn indexed_tiles(&self) -> Vec<&DrawCommand>;
    }

    impl IndexedTiles for Vec<DrawCommand> {
        fn indexed_tiles(&self) -> Vec<&DrawCommand> {
            self.iter()
                .filter(|command| matches!(command, DrawCommand::IndexedTile { .. }))
                .collect()
        }
    }

    fn indexed_tile_snapshot(command: &DrawCommand) -> (i32, i32, u16, bool) {
        match command {
            DrawCommand::IndexedTile { x, y, tile, .. } => (*x, *y, tile.tile_index, tile.priority),
            other => panic!("expected indexed tile command, got {other:?}"),
        }
    }

    fn diagnostic_sprite_tile(offset_x: i32, offset_y: i32, tile_index: u16) -> SpriteTile {
        SpriteTile {
            offset_x,
            offset_y,
            tile: sample_tile_ref(tile_index),
        }
    }

    fn sample_sprite_command(tile_indices: &[u16]) -> DrawCommand {
        DrawCommand::Sprite {
            x: 0,
            y: 0,
            tile_sheet: Arc::new(sample_tile_sheet(tile_indices.len().max(1))),
            palette: Arc::new(sample_palette()),
            tiles: tile_indices
                .iter()
                .map(|&tile_index| SpriteTile {
                    offset_x: 0,
                    offset_y: 0,
                    tile: sample_tile_ref(tile_index),
                })
                .collect(),
        }
    }

    fn sprite_tile_snapshot(
        command: &DrawCommand,
    ) -> (i32, i32, u16, u8, bool, bool, bool, i32, i32) {
        match command {
            DrawCommand::Sprite { x, y, tiles, .. } => {
                let sprite_tile = *tiles.first().expect("sprite command should contain tiles");
                (
                    *x,
                    *y,
                    sprite_tile.tile.tile_index,
                    sprite_tile.tile.palette_line,
                    sprite_tile.tile.priority,
                    sprite_tile.tile.flip_x,
                    sprite_tile.tile.flip_y,
                    sprite_tile.offset_x,
                    sprite_tile.offset_y,
                )
            }
            other => panic!("expected sprite command, got {other:?}"),
        }
    }

    fn scene_rects(commands: &[DrawCommand]) -> Vec<(i32, i32, i32, i32, [u8; 3])> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Rect {
                    x,
                    y,
                    width,
                    height,
                    colour,
                } => Some((*x, *y, *width, *height, *colour)),
                _ => None,
            })
            .collect()
    }

    fn text_snapshots(commands: &[DrawCommand]) -> Vec<(i32, i32, String)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { x, y, text, .. } => Some((*x, *y, text.clone())),
                _ => None,
            })
            .collect()
    }

    fn sprite_snapshots(commands: &[DrawCommand]) -> Vec<(i32, i32, u8, bool)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Sprite {
                    x,
                    y,
                    tile_sheet,
                    tiles,
                    ..
                } => {
                    let tile = tiles
                        .first()
                        .expect("test sprite commands should contain one tile");
                    Some((*x, *y, tile_sheet.tiles[0].pixels[0], tile.tile.flip_x))
                }
                _ => None,
            })
            .collect()
    }

    fn expected_heady_metal_wobble_snapshots(
        screen_x: i32,
        screen_y: i32,
        amplitude: i32,
        phase: i32,
    ) -> Vec<(i32, i32, u8, bool)> {
        let horizontal = amplitude * phase;
        let vertical = amplitude * -phase;
        vec![
            (screen_x, screen_y + vertical - 8, 0x20, false),
            (screen_x - 12 + horizontal, screen_y - 26, 0x21, false),
            (screen_x + 4 - horizontal, screen_y - 26, 0x21, true),
        ]
    }

    fn add_heady_metal_death_effect_visuals(visuals: &mut NativeLevelVisuals) {
        visuals
            .enemies
            .families
            .insert(0x20, sample_animated_visual(sample_sprite_frame(0x20)));
        visuals
            .enemies
            .families
            .insert(0x21, sample_animated_visual(sample_sprite_frame(0x21)));
    }

    fn heady_metal_split_trace(
        pieces: Vec<HeadyMetalDeathPieceTrace>,
    ) -> HeadyMetalDeathSplitTrace {
        HeadyMetalDeathSplitTrace {
            current_x_velocity_delta: -0x0002_6D18,
            left_worker_x_velocity_delta: 0x0002_6D18,
            centre_worker_y_velocity_delta: 0x0001_3880,
            centre_worker_sprite_label: Some("LnkTo_unk_C8458-Data_Index".to_owned()),
            right_worker_x_velocity_delta: -0x0002_6D18,
            far_right_worker_x_velocity_delta: 0x0002_6D18,
            cleanup_worker_slot_offsets: [0xFB72, 0xFB76, 0xFB7A, 0xFBAE, 0xFBB2],
            pieces,
        }
    }

    fn heady_metal_piece(slot_offset: u16, x_px: u32, y_px: u32) -> HeadyMetalDeathPieceTrace {
        HeadyMetalDeathPieceTrace {
            slot_offset,
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            x_velocity: 0,
            y_velocity: 0,
        }
    }

    fn assert_child_frame(
        frame: Option<SpriteFrame>,
        expected: (u16, i16, i16, u16, u16),
        context: &str,
    ) {
        let frame = frame.unwrap_or_else(|| panic!("{context} should resolve a sprite frame"));
        assert_eq!(
            (
                frame.tile_base_index,
                frame.origin_x,
                frame.origin_y,
                frame.width_px,
                frame.height_px,
            ),
            expected,
            "{context}"
        );
        assert_eq!(
            frame.tile_sheet.tiles[0].pixels[0], 0x5A,
            "{context} should reuse the loaded enemy tile sheet"
        );
    }

    fn sample_level_visuals(block_tile_count: usize) -> NativeLevelVisuals {
        let foreground_layout_bytes = vec![0; 20 * 14];
        let foreground_layout =
            ForegroundLayout::parse(&foreground_layout_bytes, 1, 1).expect("foreground layout");
        let empty_quad = sample_quad(0);
        let empty_frame = sample_sprite_frame(0);
        let empty_prize = sample_animated_visual(empty_frame.clone());

        NativeLevelVisuals {
            foreground_tiles: sample_tile_sheet(1),
            background_tiles: sample_tile_sheet(1),
            island_shore_tiles_loc_1202a: None,
            block_tiles: sample_tile_sheet(block_tile_count),
            palettes: PaletteSet {
                foreground: sample_palette(),
                background: sample_palette(),
            },
            background: BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
                placements: Vec::new(),
                chunks: Vec::new(),
            }),
            level: LevelVisuals {
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
                    vec![empty_quad],
                    (0..81)
                        .map(|skin| sample_quad(BLOCK_TILE_VRAM_BASE + skin * 4))
                        .collect(),
                )
                .expect("block mapping"),
            },
            hud: HudVisuals {
                digits: sample_tile_sheet(1),
                digit_base_tile_index: 0,
                diamond_tiles: sample_tile_sheet(1),
                diamond_base_tile_index: 0,
                hitpoint_sheet: sample_tile_sheet(1),
                icons: Vec::new(),
                hitpoint_tiles: [sample_tile_ref(0), sample_tile_ref(1), sample_tile_ref(2)],
            },
            player: PlayerVisuals {
                forms: HashMap::from([(0, sample_player_form_visuals(empty_frame.clone()))]),
            },
            juggernaut_skull: None,
            enemies: EnemyVisuals {
                families: HashMap::new(),
            },
            floating_prizes: FloatingPrizeVisuals {
                diamond: empty_prize,
                helmets: HashMap::new(),
                icons: HashMap::new(),
            },
            flag: FlagVisuals {
                palette: sample_palette(),
                pole_bottom: empty_frame.clone(),
                waving: Vec::new(),
                frame_duration: 1,
            },
            murder_wall: None,
            lava: None,
            weather: None,
            special_effect_unknown: None,
            platforms: PlatformVisuals {
                palette: sample_palette(),
                horizontal: empty_frame.clone(),
                vertical: empty_frame,
            },
            eol_score: None,
            eol_hologram: None,
            pause_menu: None,
            title_card_letters: None,
        }
    }

    fn sample_player_form_visuals(frame: SpriteFrame) -> PlayerFormVisuals {
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

    fn sample_animated_visual(frame: SpriteFrame) -> AnimatedSpriteVisual {
        AnimatedSpriteVisual {
            palette: sample_palette(),
            extra_palettes: Vec::new(),
            frames: vec![frame],
            frame_duration: 1,
        }
    }

    fn sample_lion_projectile_visuals() -> AnimatedSpriteVisual {
        let mut frames = vec![sample_sprite_frame(0); 9];
        frames.push(sample_lion_projectile_frame(0x000, 0x0C, 0x0C, 0x90));
        frames.push(sample_lion_projectile_frame(0x004, 0x0C, 0x0C, 0x91));
        frames.push(sample_lion_projectile_frame(0x008, 0x0A, 0x0B, 0x92));
        AnimatedSpriteVisual {
            palette: sample_palette(),
            extra_palettes: Vec::new(),
            frames,
            frame_duration: 1,
        }
    }

    fn sample_lion_projectile_frame(
        tile_base_index: u16,
        width_px: u16,
        height_px: u16,
        pixel: u8,
    ) -> SpriteFrame {
        SpriteFrame {
            origin_x: 0x06,
            origin_y: 0x00,
            width_px,
            height_px,
            tile_base_index,
            tile_sheet: sample_tile_sheet_with_pixel(0x10, pixel),
        }
    }

    fn sample_sprite_frame(pixel: u8) -> SpriteFrame {
        SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 8,
            height_px: 8,
            tile_base_index: 0,
            tile_sheet: sample_tile_sheet_with_pixel(1, pixel),
        }
    }

    fn sample_object_snapshot() -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: 0,
            y_pos: 0,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            sprite_animation_phase: None,
            wait_timer: None,
            debug_fields: BTreeMap::default(),
        }
    }

    fn sample_boss_eye_object(
        x_pos: u32,
        y_pos: u32,
        mapping_index: Option<&str>,
        order_word: Option<&str>,
    ) -> NativeObjectSnapshot {
        let mut debug_fields = BTreeMap::default();
        if let Some(mapping_index) = mapping_index {
            debug_fields.insert(
                "boss_eye_sprite_mapping_index_loc_37abc".to_owned(),
                mapping_index.to_owned(),
            );
        }
        if let Some(order_word) = order_word {
            debug_fields.insert(
                "boss_eye_sprite_order_word_a_loc_393c8".to_owned(),
                order_word.to_owned(),
            );
        }

        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: BOSS_EYE_OBJECT_TYPE,
            },
            x_pos,
            y_pos,
            debug_fields,
            ..sample_object_snapshot()
        }
    }

    fn sample_boss_eye_projectile(
        kind: ActiveProjectileKind,
        x_px: u32,
        y_px: u32,
        mapping_index: u8,
        order_word: u16,
    ) -> ActiveProjectileSnapshot {
        ActiveProjectileSnapshot {
            kind,
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            boss_eye_sprite_mapping_index: mapping_index,
            boss_eye_sprite_order_word_a: order_word,
            ..ActiveProjectileSnapshot::default()
        }
    }

    fn sample_quad(tile_index: u16) -> TileQuad {
        TileQuad {
            tiles: [
                sample_tile_ref(tile_index),
                sample_tile_ref(tile_index + 1),
                sample_tile_ref(tile_index + 2),
                sample_tile_ref(tile_index + 3),
            ],
        }
    }

    fn sample_tile_ref(tile_index: u16) -> TileRef {
        TileRef {
            tile_index,
            palette_line: 0,
            priority: true,
            flip_x: false,
            flip_y: false,
        }
    }

    fn sample_tile_sheet(tile_count: usize) -> TileSheet {
        sample_tile_sheet_with_pixel(tile_count, 0)
    }

    fn sample_tile_sheet_with_pixel(tile_count: usize, pixel: u8) -> TileSheet {
        TileSheet {
            tiles: vec![
                Tile {
                    pixels: [pixel; 64]
                };
                tile_count
            ],
        }
    }

    fn sample_eol_hologram_visuals() -> wildside_render::EolHologramVisuals {
        wildside_render::EolHologramVisuals {
            palette: sample_palette(),
            tile_sheet: sample_tile_sheet(1),
            rows: vec![vec![sample_tile_ref(0); 40]; 28],
        }
    }

    fn sample_pause_menu_visuals() -> wildside_render::PauseMenuVisuals {
        wildside_render::PauseMenuVisuals {
            tile_sheet: sample_tile_sheet(0x300),
            vram_base_tile_index: 0x0676,
            give_up_rows: sample_pause_menu_rows(0x0200),
            restart_rows: sample_pause_menu_rows(0x0100),
        }
    }

    fn sample_pause_menu_rows(base_tile: u16) -> Vec<Vec<TileRef>> {
        (0..PAUSE_MENU_TILE_ROWS_LOC_69D2)
            .map(|row| {
                (0..PAUSE_MENU_TILE_COLUMNS_LOC_69D2)
                    .map(|column| {
                        sample_tile_ref(
                            base_tile + (row * PAUSE_MENU_TILE_COLUMNS_LOC_69D2 + column) as u16,
                        )
                    })
                    .collect()
            })
            .collect()
    }

    fn sample_title_card_letters_visuals(
        tile_count: usize,
    ) -> wildside_render::TitleCardLettersVisuals {
        wildside_render::TitleCardLettersVisuals {
            tile_sheet: sample_tile_sheet(tile_count),
            palette: sample_palette(),
        }
    }

    fn sample_eol_score_visuals() -> wildside_render::EolScoreVisuals {
        wildside_render::EolScoreVisuals {
            palette: sample_palette(),
            bonus: sample_sprite_frame(0xB0),
            continues: sample_sprite_frame(0xC0),
            game: sample_sprite_frame(0x60),
            no_hit: sample_sprite_frame(0xA1),
            no_prize: sample_sprite_frame(0xA2),
            over: sample_sprite_frame(0x61),
            path: sample_sprite_frame(0xA3),
            score: sample_sprite_frame(0xC1),
            seconds: sample_sprite_frame(0x5E),
            speed: sample_sprite_frame(0xA4),
            time: sample_sprite_frame(0xA5),
            under: sample_sprite_frame(0xA6),
            digits: (0..10).map(sample_sprite_frame).collect(),
        }
    }

    fn sample_palette() -> Palette {
        Palette {
            colours: (0..16).map(MdColour::from_word).collect(),
        }
    }

    fn sample_palette_with_first_word(word: u16) -> Palette {
        let mut palette = sample_palette();
        palette.colours[0] = MdColour::from_word(word);
        palette
    }

    fn palette_raw_words(palette: &Palette) -> Vec<u16> {
        palette
            .colours
            .iter()
            .map(|colour| colour.raw_word)
            .collect()
    }
}
