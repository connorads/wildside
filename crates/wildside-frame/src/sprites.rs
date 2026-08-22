use std::sync::Arc;

use wildside_core::{
    CycloneGroundedSpriteSource, NativeContactPhase, NativeObjectKind, NativeObjectSnapshot,
    NativeTraceFrame, PlatformSpriteDispatchInputLoc244C, PlatformSpriteDispatchLoc244C,
    PlatformSpriteEntryLoc2548, PlatformSpriteVariantLoc244C, PlayerForm, TransformHandler,
    physics::Direction, presented_transform_phase,
};
use wildside_render::{MdColour, NativeLevelVisuals, Palette, SpriteFrame, TileRef};

use crate::dto::*;
use crate::scene::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TankTurretFrameSpec {
    duration_frames: usize,
    tile_base_index: u16,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
}

const ROCK_TANK_TURRET_STRU_3B27E: [TankTurretFrameSpec; 5] = [
    TankTurretFrameSpec {
        duration_frames: 0x37,
        // ASM unk_C7EC8 / `stru_3B27E`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0A,
        // ASM unk_C7F20 / `stru_3B27E`
        tile_base_index: 0x078,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F28 / `stru_3B27E`
        tile_base_index: 0x084,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x04,
        // ASM unk_C7F20 / `stru_3B27E`
        tile_base_index: 0x078,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x05,
        // ASM unk_C7EC8 / `stru_3B27E`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
];

const ROCK_TANK_SHOOTING_FAST_STRU_3B4D8: [TankTurretFrameSpec; 7] = [
    TankTurretFrameSpec {
        duration_frames: 0x37,
        // ASM unk_C7EC8 / `stru_3B4D8`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4D8`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F08 / `stru_3B4D8`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F10 / `stru_3B4D8`
        tile_base_index: 0x060,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x14,
        // ASM unk_C7F08 / `stru_3B4D8`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4D8`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x05,
        // ASM unk_C7EC8 / `stru_3B4D8`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
];

const BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE: u16 = 120 + (10 * 17);

fn boss_head_death_frame_loc_39ebe(remaining: u16) -> usize {
    // ASM loc_39EBE plays eight sweep scripts, then loc_39F7A loops four
    // one-tick spin scripts ten times before j_Delete_CurrentObject.
    const INTRO: &[(u16, usize)] = &[
        // stru_39FEE
        (6, 4),
        (6, 3),
        (6, 2),
        (6, 1),
        (6, 0),
        // stru_3A004
        (6, 1),
        (6, 2),
        (6, 3),
        (6, 4),
        // stru_3A016
        (4, 3),
        (4, 2),
        (4, 1),
        (4, 0),
        // stru_3A028
        (4, 1),
        (4, 2),
        (4, 3),
        (4, 4),
        // stru_3A03A
        (2, 4),
        (2, 3),
        (2, 2),
        (2, 1),
        (2, 0),
        // stru_3A050
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
        // stru_3A062
        (2, 3),
        (2, 2),
        (2, 1),
        (2, 0),
        // stru_3A074
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
    ];
    const SPIN: &[(u16, usize)] = &[
        // stru_3A086
        (1, 4),
        (1, 3),
        (1, 2),
        (1, 1),
        (1, 0),
        // stru_3A09C
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        // stru_3A0AE
        (1, 3),
        (1, 2),
        (1, 1),
        (1, 0),
        // stru_3A0C0
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
    ];

    let mut elapsed =
        usize::from(BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE.saturating_sub(remaining));
    for &(duration, frame) in INTRO {
        let duration = usize::from(duration);
        if elapsed < duration {
            return frame;
        }
        elapsed -= duration;
    }

    let spin_total = SPIN
        .iter()
        .map(|(duration, _)| usize::from(*duration))
        .sum::<usize>();
    let mut spin_elapsed = elapsed % spin_total;
    for &(duration, frame) in SPIN {
        let duration = usize::from(duration);
        if spin_elapsed < duration {
            return frame;
        }
        spin_elapsed -= duration;
    }
    4
}

fn boss_head_death_facing_loc_39ebe(facing: Direction, remaining: u16) -> Direction {
    let elapsed = usize::from(BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE.saturating_sub(remaining));
    // ASM loc_39EE2/loc_39F7A toggles x_direction with `eori.b #$FF,x_direction(a3)`
    // between the death animation scripts; no extra toggle happens at the loop tail.
    let toggled = match elapsed {
        0..30 => false,
        30..70 => true,
        70..96 => false,
        96..112 => true,
        112..120 => false,
        _ => {
            let spin_elapsed = (elapsed - 120) % 17;
            (5..13).contains(&spin_elapsed)
        }
    };
    if toggled {
        opposite_direction(facing)
    } else {
        facing
    }
}

fn opposite_direction(facing: Direction) -> Direction {
    match facing {
        Direction::Right => Direction::Left,
        Direction::Left => Direction::Right,
    }
}

const ROCK_TANK_SHOOTING_SLOW_STRU_3B4F6: [TankTurretFrameSpec; 8] = [
    TankTurretFrameSpec {
        duration_frames: 0x37,
        // ASM unk_C7EC8 / `stru_3B4F6`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4F6`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F08 / `stru_3B4F6`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x06,
        // ASM unk_C7F10 / `stru_3B4F6`
        tile_base_index: 0x060,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x14,
        // ASM unk_C7F08 / `stru_3B4F6`
        tile_base_index: 0x054,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1F,
        height_px: 0x18,
    },
    TankTurretFrameSpec {
        duration_frames: 0x0C,
        // ASM unk_C7F00 / `stru_3B4F6`
        tile_base_index: 0x048,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x15,
    },
    TankTurretFrameSpec {
        duration_frames: 0x05,
        // ASM unk_C7EC8 / `stru_3B4F6`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
    TankTurretFrameSpec {
        duration_frames: 0x74,
        // ASM unk_C7EC8 / `stru_3B4F6`
        tile_base_index: 0x020,
        origin_x: 0x0F,
        origin_y: 0x00,
        width_px: 0x1E,
        height_px: 0x14,
    },
];

fn build_player_sprite_tiles_with_flip(
    frame: &wildside_render::SpriteFrame,
    flip_x: bool,
    priority_override: Option<bool>,
) -> Vec<SpriteTile> {
    let mut tiles = build_sprite_tiles_with_flip(frame, flip_x, 0);
    if let Some(priority) = priority_override {
        for tile in &mut tiles {
            tile.tile.priority = priority;
        }
    }
    tiles
}

pub fn align_sprite_dimension(px: u16) -> u16 {
    px.div_ceil(8) * 8
}

/// Derive the Kid-art tile lengths written by ASM `sub_1492 -> sub_13C0
/// loc_141A` for one selected player mapping.
///
/// The ROM emits hardware sprites in 32px-high bands, splitting each band
/// into 32px-wide chunks. `Transfer_SpriteAndKidToVRAM loc_AD4` consumes the
/// resulting tile counts on the following `Mode_Level` pass. Six entries
/// cover the largest player mapping in the Kid ROM: Red Stealth's 47x78
/// transformation frame.
pub fn player_frame_dynamic_art_tiles_loc_141a(frame: &wildside_render::SpriteFrame) -> [u16; 6] {
    let mut tile_lengths = [0_u16; 6];
    let mut next = 0_usize;
    let mut remaining_height_px = frame.height_px;

    while remaining_height_px > 0 {
        let chunk_height_px = align_sprite_dimension(remaining_height_px.min(32));
        let mut remaining_width_px = frame.width_px;
        while remaining_width_px > 0 {
            let chunk_width_px = align_sprite_dimension(remaining_width_px.min(32));
            let destination = tile_lengths
                .get_mut(next)
                .expect("Kid ROM player mappings use at most six hardware sprites");
            *destination = (chunk_width_px / 8) * (chunk_height_px / 8);
            next += 1;
            remaining_width_px = remaining_width_px.saturating_sub(32);
        }
        remaining_height_px = remaining_height_px.saturating_sub(32);
    }

    tile_lengths
}

/// Player-art words resident at VRAM `$C4A0` after the independently paced
/// `Transfer_SpriteAndKidToVRAM loc_AD4` chunk DMAs.
///
/// The VDP can present a prefix of one DMA at `FrameRendered`, including a
/// partial 8x8 tile. Keeping this page separate from [`SpriteFrame`] lets the
/// current Sprite Table geometry address art retained from an earlier frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDynamicArtVramPageLocAd4 {
    pub tile_sheet: wildside_render::TileSheet,
}

impl PlayerDynamicArtVramPageLocAd4 {
    pub fn from_resident_frame(frame: &wildside_render::SpriteFrame) -> Self {
        Self {
            tile_sheet: frame.tile_sheet.clone(),
        }
    }

    /// Apply the word prefix resident from each `loc_AD4` DMA at the sampled
    /// `FrameRendered` boundary. Each Genesis VRAM word contains four 4bpp
    /// pixels, so an incomplete DMA may replace only part of a tile row.
    pub fn apply_frame_rendered_prefixes(
        &mut self,
        source: &wildside_render::SpriteFrame,
        residency: &[wildside_core::vdp::MemoryDmaFrameRenderedResidency],
    ) -> Option<()> {
        let mut destination_word = 0usize;
        for dma in residency {
            let dma_words = usize::from(dma.words);
            let committed_words = usize::from(dma.committed_words_before_frame_rendered);
            if committed_words > dma_words {
                return None;
            }
            let source_end = destination_word.checked_add(committed_words)?;
            if source_end > source.tile_sheet.tiles.len().checked_mul(16)? {
                return None;
            }
            self.ensure_word_capacity(destination_word.checked_add(dma_words)?);
            for word in destination_word..source_end {
                copy_tile_word(&mut self.tile_sheet, &source.tile_sheet, word)?;
            }
            destination_word = destination_word.checked_add(dma_words)?;
        }
        Some(())
    }

    /// Apply a completed `loc_AD4` queue. This conservative fallback keeps
    /// whole-frame presentation when persistent hardware phase is absent,
    /// while retaining destination words not touched by the current queue.
    pub fn apply_complete_frame(
        &mut self,
        source: &wildside_render::SpriteFrame,
        tile_lengths: [u16; 6],
    ) -> Option<()> {
        let residency = tile_lengths
            .into_iter()
            .filter(|&tiles| tiles != 0)
            .map(|tiles| {
                let words = tiles.checked_mul(16)?;
                Some(wildside_core::vdp::MemoryDmaFrameRenderedResidency {
                    words,
                    committed_words_before_frame_rendered: words,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        self.apply_frame_rendered_prefixes(source, &residency)
    }

    fn ensure_word_capacity(&mut self, words: usize) {
        let tiles = words.div_ceil(16);
        if self.tile_sheet.tiles.len() < tiles {
            self.tile_sheet
                .tiles
                .resize_with(tiles, || wildside_render::Tile { pixels: [0; 64] });
        }
    }
}

fn copy_tile_word(
    destination: &mut wildside_render::TileSheet,
    source: &wildside_render::TileSheet,
    word: usize,
) -> Option<()> {
    const WORDS_PER_TILE: usize = 16;
    const PIXELS_PER_WORD: usize = 4;

    let tile_index = word / WORDS_PER_TILE;
    let pixel_start = (word % WORDS_PER_TILE) * PIXELS_PER_WORD;
    let source_pixels =
        &source.tiles.get(tile_index)?.pixels[pixel_start..pixel_start + PIXELS_PER_WORD];
    destination.tiles.get_mut(tile_index)?.pixels[pixel_start..pixel_start + PIXELS_PER_WORD]
        .copy_from_slice(source_pixels);
    Some(())
}

pub fn sprite_draw_position(
    frame: &wildside_render::SpriteFrame,
    screen_x: i32,
    screen_y: i32,
    facing: Direction,
) -> (i32, i32) {
    let draw_x = match facing {
        Direction::Right => screen_x - i32::from(frame.origin_x),
        Direction::Left => {
            screen_x + i32::from(frame.origin_x) - i32::from(align_sprite_dimension(frame.width_px))
        }
    };
    (
        draw_x,
        screen_y - i32::from(frame.height_px) + i32::from(frame.origin_y),
    )
}

pub fn build_sprite_tiles(
    frame: &wildside_render::SpriteFrame,
    facing: Direction,
    palette_line: u8,
) -> Vec<SpriteTile> {
    build_sprite_tiles_with_flip(frame, matches!(facing, Direction::Left), palette_line)
}

fn build_sprite_tiles_with_flip(
    frame: &wildside_render::SpriteFrame,
    flip_x: bool,
    palette_line: u8,
) -> Vec<SpriteTile> {
    let aligned_width_px = align_sprite_dimension(frame.width_px);
    let aligned_height_px = align_sprite_dimension(frame.height_px);
    let mut tiles =
        Vec::with_capacity(usize::from(aligned_width_px / 8) * usize::from(aligned_height_px / 8));
    let mut next_tile_index = frame.tile_base_index;
    let mut band_y_px = 0u16;
    let mut remaining_height_px = frame.height_px;

    while remaining_height_px >= 32 {
        append_sprite_band_tiles(
            &mut tiles,
            &mut next_tile_index,
            band_y_px,
            32,
            frame.width_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
        band_y_px += 32;
        remaining_height_px -= 32;
    }

    let tail_height_px = align_sprite_dimension(remaining_height_px);
    if tail_height_px > 0 {
        append_sprite_band_tiles(
            &mut tiles,
            &mut next_tile_index,
            band_y_px,
            tail_height_px,
            frame.width_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
    }

    tiles
}

pub fn append_sprite_band_tiles(
    tiles: &mut Vec<SpriteTile>,
    next_tile_index: &mut u16,
    band_y_px: u16,
    band_height_px: u16,
    frame_width_px: u16,
    aligned_width_px: u16,
    flip_x: bool,
    palette_line: u8,
) {
    let mut remaining_width_px = frame_width_px;
    let mut chunk_x_px = 0u16;

    while remaining_width_px >= 32 {
        append_sprite_chunk_tiles(
            tiles,
            next_tile_index,
            chunk_x_px,
            band_y_px,
            32,
            band_height_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
        chunk_x_px += 32;
        remaining_width_px -= 32;
    }

    let tail_width_px = align_sprite_dimension(remaining_width_px);
    if tail_width_px > 0 {
        append_sprite_chunk_tiles(
            tiles,
            next_tile_index,
            chunk_x_px,
            band_y_px,
            tail_width_px,
            band_height_px,
            aligned_width_px,
            flip_x,
            palette_line,
        );
    }
}

pub fn append_sprite_chunk_tiles(
    tiles: &mut Vec<SpriteTile>,
    next_tile_index: &mut u16,
    chunk_x_px: u16,
    chunk_y_px: u16,
    chunk_width_px: u16,
    chunk_height_px: u16,
    aligned_width_px: u16,
    flip_x: bool,
    palette_line: u8,
) {
    let columns = chunk_width_px / 8;
    let rows = chunk_height_px / 8;

    for column in 0..columns {
        for row in 0..rows {
            let unflipped_x_px = chunk_x_px + column * 8;
            let offset_x = if flip_x {
                i32::from(aligned_width_px - unflipped_x_px - 8)
            } else {
                i32::from(unflipped_x_px)
            };
            tiles.push(SpriteTile {
                offset_x,
                offset_y: i32::from(chunk_y_px + row * 8),
                tile: TileRef {
                    tile_index: *next_tile_index,
                    palette_line,
                    priority: false,
                    flip_x,
                    flip_y: false,
                },
            });
            *next_tile_index += 1;
        }
    }
}

pub fn select_player_frame<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a wildside_render::PlayerFormVisuals,
) -> &'a wildside_render::SpriteFrame {
    let on_ground = trace.kid_on_ground.unwrap_or(true);
    let x_velocity = trace.kid_x_velocity.unwrap_or(0);
    let y_velocity = trace.kid_y_velocity.unwrap_or(0);

    if trace.ledge_snap_cooldown > 0 && !player_visuals.pull_up.is_empty() {
        // ASM loc_ABF0 (kid.asm:13449) starts `stru_8BB4`, then
        // `Hibernate_UntilAnimFinished` holds seven rollover frames for three
        // VBlanks each before returning to MoveID_Standingstill.
        let total = player_visuals.pull_up.len() * 3;
        let elapsed = total.saturating_sub(usize::from(trace.ledge_snap_cooldown));
        let frame_index = (elapsed / 3).min(player_visuals.pull_up.len() - 1);
        return &player_visuals.pull_up[frame_index];
    }

    if trace.current_helmet_id == PlayerForm::Cyclone.helmet_id()
        && trace.cyclone_flying
        && !player_visuals.flight_spin.is_empty()
    {
        // ASM sub_B270 loc_B2A0 (kid.asm:14154-14175) stores the flight phase
        // in `$FFFFF8F0` and selects `cyclone_spinning_1..3` via `phase >> 8`.
        let frame_index = usize::from(trace.walk_visual_accumulator >> 8)
            .min(player_visuals.flight_spin.len() - 1);
        return &player_visuals.flight_spin[frame_index];
    }

    if x_velocity != 0
        && let Some(source) = trace.cyclone_grounded_sprite_source
        && !player_visuals.walk.is_empty()
    {
        // ASM loc_AB0A/loc_AB4A (kid.asm:13390-13430) clears flight without
        // calling sub_942A, preserving the last `addroffset_sprite` through
        // the Sprite_Table handoff. The shared phase still names a flight
        // phase here, so select the typed retained mapping directly while
        // grounded motion continues. Once x velocity reaches zero, sub_942A
        // loc_95BA writes the standing mapping instead.
        let frame_index = match source {
            CycloneGroundedSpriteSource::Walk1 => 0,
            CycloneGroundedSpriteSource::Walk2 => 1,
            CycloneGroundedSpriteSource::Walk3 => 2,
            CycloneGroundedSpriteSource::Walk4 => 3,
            CycloneGroundedSpriteSource::Walk5 => 4,
            CycloneGroundedSpriteSource::Walk6 => 5,
        };
        return &player_visuals.walk[frame_index.min(player_visuals.walk.len() - 1)];
    }

    // ASM sub_B270 loc_B350/loc_B380 (kid.asm:14220-14248): the airborne
    // attack-down counter (`red_stealth_airborne_slash_counter`, 0→$7F)
    // selects one of four downward-slash phases via `counter >> 5`. Native
    // wired the counter through the runtime step in commit `bcfb47f6`; this
    // branch now consumes it so the visual sword actually points down during
    // a fast-fall hold instead of falling through to the generic jump-fall
    // pose. Takes precedence over the ground swing timer because the airborne
    // path is what the user reported broken (#153).
    if !on_ground
        && trace.red_stealth_airborne_slash_counter > 0
        && !player_visuals.slash_air.is_empty()
    {
        let phase = usize::from(trace.red_stealth_airborne_slash_counter >> 5);
        let frame_index = phase.min(player_visuals.slash_air.len() - 1);
        return &player_visuals.slash_air[frame_index];
    }

    if trace.red_stealth_sword_timer > 0 {
        let slash_frames = if on_ground {
            &player_visuals.slash_ground
        } else {
            &player_visuals.slash_air
        };
        if !slash_frames.is_empty() {
            // ASM loc_76B0 initialises `stru_8B3C` for the grounded sword and
            // sub_B270 indexes `unk_B40DE..unk_B4870` for the air slash.
            let elapsed = usize::from(12u8.saturating_sub(trace.red_stealth_sword_timer));
            let frame_index = (elapsed / 3).min(slash_frames.len() - 1);
            return &slash_frames[frame_index];
        }
    }

    let iron_knight_attach_hibernate = trace.current_helmet_id
        == PlayerForm::IronKnight.helmet_id()
        && trace.iron_knight_climbing
        && trace.iron_knight_climb_pose_accum == 0
        && y_velocity == -0x4000;
    if iron_knight_attach_hibernate {
        // ASM loc_A276: Hibernate_Object_1Frame returns before sub_A3BC can
        // replace addroffset_sprite. The grounded idle mapping that entered
        // the attach therefore remains resident even though loc_A276 has
        // already cleared on_ground and seeded upward velocity.
        return &player_visuals.idle;
    }
    if trace.current_helmet_id == PlayerForm::IronKnight.helmet_id()
        && trace.iron_knight_climbing
        && !player_visuals.climb.is_empty()
    {
        // ASM sub_A3BC (kid.asm:12710-12715) stores d3, buckets `d3 >> 2`
        // through `unk_A3F0`, then writes `off_A420` into addroffset_sprite.
        // Preserve that sprite write directly; a stationary cling is not idle.
        //
        // ASM loc_A276 seeds d3=0/y_vel=$FFFFC000 and hibernates through
        // loc_A28A before sub_A3BC has written an off_A420 climb sprite, so
        // the exact attach snapshot must fall through to the ordinary pose.
        let frame_index =
            iron_knight_climb_frame_index_sub_a3bc(trace.iron_knight_climb_pose_accum)
                .min(player_visuals.climb.len() - 1);
        return &player_visuals.climb[frame_index];
    }

    if on_ground {
        if trace.current_helmet_id == PlayerForm::Berzerker.helmet_id()
            && (trace.berzerker_charging || trace.berzerker_charge_pose)
            && !player_visuals.charge.is_empty()
        {
            // ASM loc_9564 writes off_9688 (`berzerker_run_*`) immediately
            // after the charge accumulator carries. That sprite write survives
            // loc_9326 clearing Berzerker_charging after wall contact, so the
            // latched pose must also win over idle when native velocity stops.
            let frame_index =
                usize::from(trace.walk_visual_accumulator >> 8) % player_visuals.charge.len();
            &player_visuals.charge[frame_index]
        } else if trace.current_helmet_id == PlayerForm::Berzerker.helmet_id()
            && trace.kid_on_slope != 0
        {
            // ASM loc_A13A / off_A190: slope animation table selection first
            // canonicalises x_direction through $FFFFFA26. SlopeUp keeps the
            // live facing byte; SlopeDown XORs it with $FF before loc_A16C
            // chooses the second six-frame table half.
            let facing = trace.kid_facing.unwrap_or(Direction::Right);
            let canonical_reverse = match (trace.kid_on_slope < 0, facing) {
                (false, Direction::Right) | (true, Direction::Left) => false,
                (false, Direction::Left) | (true, Direction::Right) => true,
            };
            let slope_frames = match canonical_reverse {
                false if !player_visuals.slope_walk_left.is_empty() => {
                    &player_visuals.slope_walk_left
                }
                true if !player_visuals.slope_walk.is_empty() => &player_visuals.slope_walk,
                _ => &player_visuals.walk,
            };
            // ASM loc_A13A writes the slope frame through `$FFFFF8F0 >> 8` for
            // every form. Native used to approximate non-Juggernaut slopes
            // from frame time, which made downhill/reverse motion animate out
            // of phase with the ROM accumulator.
            let frame_index = usize::from(trace.walk_visual_accumulator >> 8) % slope_frames.len();
            &slope_frames[frame_index]
        } else if trace.current_helmet_id == PlayerForm::Berzerker.helmet_id()
            && trace.berzerker_walk_pose
            && !player_visuals.walk.is_empty()
        {
            // ASM loc_9580 -> loc_95DA -> loc_9600 writes Berzerker's
            // ordinary off_967C walk sprite before later wall collision can
            // leave the sampled post-frame velocity at zero, but loc_A13A's
            // slope sprite write happens later and therefore wins above.
            let frame_index =
                usize::from(trace.walk_visual_accumulator >> 8) % player_visuals.walk.len();
            &player_visuals.walk[frame_index]
        } else if trace.kid_on_slope != 0 {
            // ASM loc_A13A / off_A190: non-Berzerker slope animation table
            // selection follows the same `$FFFFF8F0 >> 8` phase and happens
            // before the standing-still draw state, so wedged forms do not
            // flicker between idle and slope art when x_vel reaches zero.
            let facing = trace.kid_facing.unwrap_or(Direction::Right);
            let canonical_reverse = match (trace.kid_on_slope < 0, facing) {
                (false, Direction::Right) | (true, Direction::Left) => false,
                (false, Direction::Left) | (true, Direction::Right) => true,
            };
            let slope_frames = match canonical_reverse {
                false if !player_visuals.slope_walk_left.is_empty() => {
                    &player_visuals.slope_walk_left
                }
                true if !player_visuals.slope_walk.is_empty() => &player_visuals.slope_walk,
                _ => &player_visuals.walk,
            };
            let frame_index = usize::from(trace.walk_visual_accumulator >> 8) % slope_frames.len();
            &slope_frames[frame_index]
        } else if x_velocity != 0
            && trace.current_helmet_id == PlayerForm::TheKid.helmet_id()
            && trace.kid_facing.is_some_and(|facing| {
                matches!(
                    (x_velocity.is_positive(), facing),
                    (true, Direction::Left) | (false, Direction::Right)
                )
            })
            && let Some(skid) = player_visuals.skid.as_ref()
        {
            // ASM sub_942A loc_9580/loc_9596/loc_959C (kid.asm:11031-11043):
            // only The Kid bypasses off_9618 when velocity opposes facing and
            // writes fixed mapping LnkTo_unk_A5346 (`kid_skid.bin`). The scene
            // renderer supplies the previous trace's motion state because
            // Mode_Level transfers Sprite_Table before this mapping is built.
            skid
        } else if x_velocity == 0 {
            &player_visuals.idle
        } else {
            // ASM loc_95DA -> loc_9600/off_9618 indexes ordinary walking
            // frames from `$FFFFF8F0 >> 8` for Kid and helmet forms. Native
            // used to drive most forms from wall-clock frame time, which put
            // Red Stealth's held-right slope approach visibly out of phase.
            let frame_index =
                usize::from(trace.walk_visual_accumulator >> 8) % player_visuals.walk.len();
            &player_visuals.walk[frame_index]
        }
    } else {
        // ASM sub_B270 loc_B334/loc_B350 indexes the current helmet's four
        // `off_B3B8` mappings. The typed core state preserves the dynamic
        // `$FFFFFAAA` jump threshold, which cannot be recovered from the sign
        // of the post-frame Y velocity alone.
        let state_index = match trace.airborne_visual_mapping_sub_b270 {
            wildside_core::ModeLevelAirborneVisualMappingPathSubB270::AscendingState0 => 0,
            wildside_core::ModeLevelAirborneVisualMappingPathSubB270::AscendingState1 => 1,
            wildside_core::ModeLevelAirborneVisualMappingPathSubB270::DescendingState2 => 2,
            wildside_core::ModeLevelAirborneVisualMappingPathSubB270::DescendingState3 => 3,
            // Preserve the established sign-only renderer when no typed
            // sub_B270 selection is available. Its fall page is table entry
            // 3; entry 2 is the newly distinguished slow-descending state.
            wildside_core::ModeLevelAirborneVisualMappingPathSubB270::Unclassified => {
                usize::from(y_velocity >= 0) * 3
            }
        };
        &player_visuals.airborne[state_index.min(player_visuals.airborne.len() - 1)]
    }
}

fn iron_knight_climb_frame_index_sub_a3bc(pose_accum: u16) -> usize {
    match pose_accum >> 2 {
        0..=15 => 0,
        16..=31 => 1,
        _ => 2,
    }
}

pub fn select_player_form_visuals<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a wildside_render::PlayerVisuals,
) -> &'a wildside_render::PlayerFormVisuals {
    player_visuals
        .forms
        .get(&trace.current_helmet_id)
        .or_else(|| player_visuals.forms.get(&0))
        .expect("Kid visuals should always be loaded")
}

/// Select the player art mapping whose hardware-sprite chunks `sub_1492`
/// writes to the dynamic-art queue at `loc_141A`.
pub fn select_player_dynamic_art_frame_loc_141a<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a wildside_render::PlayerVisuals,
) -> &'a wildside_render::SpriteFrame {
    if trace.death_windup_elapsed > 0
        && let Some(frame) = player_visuals
            .forms
            .get(&0)
            .and_then(|visuals| visuals.death.as_ref())
    {
        // ASM Death loc_B7FA installs Kid's tumble mapping before the shared
        // `sub_1492 -> sub_13C0 loc_141A` producer runs.
        return frame;
    }

    let form_visuals = select_player_form_visuals(trace, player_visuals);
    if let Some((frame, _)) = select_player_transform_render(trace, player_visuals, form_visuals) {
        return frame;
    }
    select_player_frame(trace, form_visuals)
}

/// Resolve the transform frame and palette for one displayed frame.
///
/// Ported path: ASM Kid_Transform loc_7D0A (kid.asm) — when the trace carries
/// the transform source helmet and `presented_transform_phase` covers the
/// (old, new) pair, resolve art positionally from the phase's descriptor index
/// (the bundle's `transform_steps` mirror the `off_7EE2` descriptor order) and
/// the palette from the `off_7D34` handler: HelmetArt → sub_8106 transform
/// palette, HelmetDefault (and the loc_7E90 cleanup frame) → sub_80D0 form
/// palette on the form's default/idle art. Unported transforms fall back to
/// the legacy duration-sum walk plus the frame-identity palette heuristic.
pub fn select_player_transform_render<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a wildside_render::PlayerVisuals,
    form_visuals: &'a wildside_render::PlayerFormVisuals,
) -> Option<(&'a wildside_render::SpriteFrame, &'a Palette)> {
    if trace.form_change_flash_frames == 0 {
        return None;
    }
    if let Some(old_helmet_id) = trace.transform_source_helmet_id_loc_7ca6
        && let Some(phase) = presented_transform_phase(
            old_helmet_id,
            trace.current_helmet_id,
            trace.form_change_flash_frames,
        )
        && let Some(resolved) =
            phase_transform_render_loc_7d0a(&phase, player_visuals, form_visuals)
    {
        return Some(resolved);
    }
    let frame = select_player_transform_frame(trace, form_visuals)?;
    Some((
        frame,
        transform_frame_palette(frame, player_visuals, form_visuals),
    ))
}

/// ASM off_7D34 handler → (art, palette) for a ported transform pass.
fn phase_transform_render_loc_7d0a<'a>(
    phase: &wildside_core::PresentedTransformPhase,
    player_visuals: &'a wildside_render::PlayerVisuals,
    form_visuals: &'a wildside_render::PlayerFormVisuals,
) -> Option<(&'a wildside_render::SpriteFrame, &'a Palette)> {
    match phase.handler {
        TransformHandler::HelmetArt => {
            // ASM loc_7D44 (kid.asm): descriptor art link on the sub_8106
            // transform palette. The bundle's transform_steps are positional
            // with stru_7FF8, so a missing frame means a bundle mismatch —
            // return None and let the legacy walk handle it.
            let frame = form_visuals
                .transform_steps
                .get(phase.descriptor_index)
                .and_then(|step| step.frame.as_ref())?;
            let palette = form_visuals
                .transform_palette
                .as_ref()
                .unwrap_or(&form_visuals.palette);
            Some((frame, palette))
        }
        TransformHandler::HelmetDefault => {
            // ASM loc_7D5A (kid.asm): off_79B2[Current_Helmet] default pose on
            // the sub_80D0 form palette; also the loc_7E90 cleanup frame.
            Some((&form_visuals.idle, &form_visuals.palette))
        }
        TransformHandler::KidWithSpark { spark: false } => {
            // ASM stru_7F30 descriptor 0/2 → loc_7D80 (kid.asm): both use
            // LnkTo_unk_A4A22, Kid walk-1, and loc_7DF0's Kid palette.
            // The Cyclone table has no selector-$C spark descriptor.
            let kid = player_visuals.forms.get(&0)?;
            Some((&kid.walk[0], &kid.palette))
        }
        // Selector $C enters loc_7D7C and creates a separate sparkle GFX
        // object before the Kid-art path. Keep its legacy fallback until that
        // second sprite is represented by the trace renderer.
        TransformHandler::KidWithSpark { spark: true } => None,
    }
}

pub fn select_player_transform_frame<'a>(
    trace: &NativeTraceFrame,
    player_visuals: &'a wildside_render::PlayerFormVisuals,
) -> Option<&'a wildside_render::SpriteFrame> {
    if trace.form_change_flash_frames == 0 || player_visuals.transform_steps.is_empty() {
        return None;
    }

    let total_duration = player_visuals
        .transform_steps
        .iter()
        .map(|step| step.duration_frames)
        .sum::<usize>();
    let mut elapsed = total_duration.saturating_sub(usize::from(trace.form_change_flash_frames));
    for step in &player_visuals.transform_steps {
        if elapsed < step.duration_frames {
            return step.frame.as_ref();
        }
        elapsed -= step.duration_frames;
    }

    player_visuals
        .transform_steps
        .iter()
        .rev()
        .find_map(|step| step.frame.as_ref())
}

fn transform_frame_palette<'a>(
    frame: &wildside_render::SpriteFrame,
    player_visuals: &'a wildside_render::PlayerVisuals,
    form_visuals: &'a wildside_render::PlayerFormVisuals,
) -> &'a Palette {
    if let Some(kid_visuals) = player_visuals.forms.get(&0)
        && kid_transform_frame_uses_kid_palette_loc_7cd0(frame, kid_visuals)
    {
        // ASM Kid_Transform loc_7CD0 (kid.asm:8426) only switches
        // Current_Helmet after the old-form table. The first-helmet
        // transforms can therefore render Kid art while the trace already
        // reports the target helmet; keep that art on Kid's palette instead
        // of the target transform palette.
        return &kid_visuals.palette;
    }

    form_visuals
        .transform_palette
        .as_ref()
        .unwrap_or(&form_visuals.palette)
}

fn kid_transform_frame_uses_kid_palette_loc_7cd0(
    frame: &wildside_render::SpriteFrame,
    kid_visuals: &wildside_render::PlayerFormVisuals,
) -> bool {
    frame == &kid_visuals.idle
        || kid_visuals.walk.iter().any(|kid_frame| frame == kid_frame)
        || kid_visuals
            .airborne
            .iter()
            .any(|kid_frame| frame == kid_frame)
        || kid_visuals
            .death
            .as_ref()
            .is_some_and(|kid_frame| frame == kid_frame)
        || kid_visuals
            .pull_up
            .iter()
            .any(|kid_frame| frame == kid_frame)
}

pub fn player_screen_position(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) -> (i32, i32) {
    (
        (object.x_pos >> 16) as i32 - (trace.camera_x_pos >> 16) as i32,
        (object.y_pos >> 16) as i32 - (trace.camera_y_pos >> 16) as i32,
    )
}

pub fn boss_eye_attack_highlight_offset_sub_37abc(mapping_index: u8) -> (i32, i32) {
    match mapping_index & 0x07 {
        0 => (-4, -4),
        1 => (0, -5),
        2 => (4, -4),
        3 => (-5, 0),
        4 => (5, 0),
        5 => (-4, 4),
        6 => (0, 5),
        _ => (4, 4),
    }
}

pub fn push_boss_eye_attack_marker_sub_37abc(
    commands: &mut Vec<DrawCommand>,
    sx: i32,
    sy: i32,
    mapping_index: u8,
) {
    // ASM sub_37332/loc_3739C and sub_37ABC select one of
    // `unk_C8488..unk_C84C0`, all decoded as 16x16 HeadyMetal eye frames.
    // Until those boss-head tiles are available in native play, keep the ROM
    // footprint and expose the selected frame with a small indexed highlight.
    commands.push(DrawCommand::Rect {
        x: sx - 8,
        y: sy - 8,
        width: 16,
        height: 16,
        colour: [120, 200, 255],
    });
    let (dx, dy) = boss_eye_attack_highlight_offset_sub_37abc(mapping_index);
    commands.push(DrawCommand::Rect {
        x: sx + dx - 2,
        y: sy + dy - 2,
        width: 4,
        height: 4,
        colour: [255, 255, 255],
    });
}

pub fn push_player_sprite_with_priority_override(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    damage_flash_white_phase: bool,
    priority_override: Option<bool>,
) {
    push_player_sprite_with_priority_and_resident_art(
        commands,
        object,
        trace,
        level_visuals,
        damage_flash_white_phase,
        priority_override,
        None,
    );
}

pub fn push_player_sprite_with_priority_and_resident_art(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    damage_flash_white_phase: bool,
    priority_override: Option<bool>,
    resident_art: Option<&wildside_render::TileSheet>,
) {
    if trace.death_windup_elapsed > 0
        && let Some(kid_visuals) = level_visuals.player.forms.get(&0)
        && let Some(frame) = kid_visuals.death.as_ref()
    {
        // ASM Death loc_B7EA/loc_B7FA: helmeted deaths first run
        // Kid_Transform, then loc_B7FA installs `unk_A3FF8` (`kid_death.bin`)
        // for the tumble loop. Use Kid's death art/palette even if the trace
        // still carries the pre-death form until the hidden `lose_life` shell.
        let render_facing = trace.kid_facing.unwrap_or(Direction::Right);
        let (screen_x, screen_y) = player_screen_position(object, trace);
        let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
        commands.push(DrawCommand::SatSprite {
            x: draw_x,
            y: draw_y,
            tile_sheet: Arc::new(resident_art.unwrap_or(&frame.tile_sheet).clone()),
            palette: Arc::new(player_palette_for_damage_phase(
                &kid_visuals.palette,
                damage_flash_white_phase,
            )),
            tiles: build_player_sprite_tiles_with_flip(
                frame,
                matches!(render_facing, Direction::Left),
                priority_override,
            ),
        });
        return;
    }

    let form_visuals = select_player_form_visuals(trace, &level_visuals.player);
    let render_facing = trace.kid_facing.unwrap_or(Direction::Right);
    let (screen_x, screen_y) = player_screen_position(object, trace);
    if let Some((frame, base_palette)) =
        select_player_transform_render(trace, &level_visuals.player, form_visuals)
    {
        let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
        commands.push(DrawCommand::SatSprite {
            x: draw_x,
            y: draw_y,
            tile_sheet: Arc::new(resident_art.unwrap_or(&frame.tile_sheet).clone()),
            palette: Arc::new(player_palette_for_damage_phase(
                base_palette,
                damage_flash_white_phase,
            )),
            tiles: build_player_sprite_tiles_with_flip(
                frame,
                matches!(render_facing, Direction::Left),
                priority_override,
            ),
        });
        return;
    }

    let frame = select_player_frame(trace, form_visuals);
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
    // ASM loc_A13A/off_A190 chooses the tilted slope frame bank, but the
    // later loc_1292 draw path still flips solely from live x_direction.
    let flip_x = matches!(render_facing, Direction::Left);

    commands.push(DrawCommand::SatSprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(resident_art.unwrap_or(&frame.tile_sheet).clone()),
        palette: Arc::new(player_palette_for_damage_phase(
            &form_visuals.palette,
            damage_flash_white_phase,
        )),
        tiles: build_player_sprite_tiles_with_flip(frame, flip_x, priority_override),
    });
}

fn player_palette_for_damage_phase(palette: &Palette, white_phase: bool) -> Palette {
    if !white_phase {
        return palette.clone();
    }

    let mut flash_palette = palette.clone();
    let white = MdColour::from_word(0x0FFF);
    // ASM sub_80BC writes twelve $FFF words starting at Palette_Buffer+$62:
    // player palette line visible slots 1..12. Slot 0 is omitted from native
    // player palettes, so indices 0..11 correspond to those visible slots.
    for colour in flash_palette.colours.iter_mut().take(12) {
        *colour = white;
    }
    flash_palette
}

fn enemy_palette_for_damage_flash(palette: &Palette, damage_flash: bool) -> Palette {
    if !damage_flash {
        return palette.clone();
    }

    let mut flash_palette = palette.clone();
    let white = MdColour::from_word(0x0FFF);
    // ASM enemy hurt routes initialise a short hibernate/recoil animation;
    // native's `damage_flash` trace flag is the observable rendering cue for
    // that non-lethal hit. Palette index 0 still resolves as transparent in
    // the renderer, so whitening the stored colours keeps sprite holes clear.
    for colour in &mut flash_palette.colours {
        *colour = white;
    }
    flash_palette
}

fn enemy_palette_for_heady_metal_sub_379e8(
    palette: &Palette,
    object: &NativeObjectSnapshot,
    object_type: u8,
) -> Palette {
    let mut palette = palette.clone();
    if object_type != 0x20 {
        return palette;
    }

    let Some(raw_word) = object
        .debug_fields
        .get("heady_metal_palette_word_5e_sub_379e8")
        .and_then(|word| {
            word.strip_prefix("0x")
                .and_then(|hex| u16::from_str_radix(hex, 16).ok())
                .or_else(|| word.parse::<u16>().ok())
        })
    else {
        return palette;
    };

    if let Some(slot) = palette.colours.get_mut(15) {
        // ASM sub_379E8 (kid.asm:39865): write `unk_379A8[d2]` to
        // Palette_Buffer+$5E, i.e. palette line 2 colour slot 15.
        *slot = MdColour::from_word(raw_word);
    }
    palette
}

fn select_waited_sequence_frame(remaining: u16, sequence: &[(u16, usize)]) -> Option<usize> {
    let mut rest = sequence.iter().map(|(duration, _)| *duration).sum::<u16>();
    for &(duration, frame_index) in sequence {
        if remaining > rest.saturating_sub(duration) {
            return Some(frame_index);
        }
        rest = rest.saturating_sub(duration);
    }
    None
}

fn select_live_walking_dragon_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0D })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 8
    {
        return None;
    }

    // ASM Dragon.asm loc_3CE48/loc_3CDFC installs `stru_3CC5E` for normal
    // walking dragons and `stru_3CC90` for HP-3 dragons. The extracted native
    // sheet appends `loc_3CEFC` death/smoke frames after the live frames, so
    // live animation must follow the ROM sequence rather than cycling every
    // frame in the sheet.
    if object.damage_flash {
        // ASM Dragon.asm loc_3CEBE installs `stru_3CCE6` after a nonlethal
        // `colid_hurt`/`colid_kidabove` hit and hibernates until that one-frame
        // sequence finishes before restoring the saved walking script.
        return Some(&enemy_visuals.frames[6]);
    }

    if object.walk_phase == Some(1) && enemy_visuals.frames.len() >= 11 {
        // ASM Dragon.asm loc_3CF90 starts `stru_3CCC2` on the parent while
        // the child breath object (`stru_3CCC8`) is active.
        return Some(&enemy_visuals.frames[10]);
    }

    let live_sequence = [0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 7, 5];
    let frame_duration = if object.hit_points == Some(3) { 6 } else { 9 };
    // ASM Init_Animation/GfxObject_Animate loc_1040-loc_1194 keeps a
    // per-GfxObject pointer and timer. Prefer the typed phase captured from
    // that object; the global trace clock is retained for legacy recordings.
    let phase = object.sprite_animation_phase.map_or_else(
        || (trace.frame_index / frame_duration) % live_sequence.len(),
        |phase| usize::from(phase) % live_sequence.len(),
    );
    Some(&enemy_visuals.frames[live_sequence[phase]])
}

fn select_live_rock_tank_body_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(
        object.kind,
        NativeObjectKind::Enemy {
            object_type: 0x0A | 0x0B
        }
    ) || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 3
    {
        return None;
    }

    // ASM Tank.asm `stru_3B270` and Tank_Shooting.asm `stru_3B4CA` animate
    // the body through only C7ED0/C7ED8/C7EE0. Native appends death frames
    // after those three entries, so live cycling must not include them.
    let phase = (trace.frame_index / enemy_visuals.frame_duration.max(1)) % 3;
    Some(&enemy_visuals.frames[phase])
}

fn select_live_diamond_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x01 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 5
    {
        return None;
    }

    // ASM Diamond.asm Enemy01_Diamond_Init installs `stru_3ACA8`: five
    // six-tick live frames, then dispatch selector 2 rewinds to the start.
    // The later extracted frames are hurt/death-only (`stru_3ACBE`/`stru_3ACC4`),
    // so live Diamonds must not cycle the whole sheet or desynchronise by
    // layout index.
    let phase = (trace.frame_index / 6) % 5;
    Some(&enemy_visuals.frames[phase])
}

fn select_live_flying_dragon_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0C })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 5
    {
        return None;
    }

    if let Some(frame) = select_flying_dragon_dive_frame(enemy_visuals, object) {
        return Some(frame);
    }

    // ASM Dragon_Flying.asm loc_346DA runs `stru_34600`, whose visible
    // sequence repeats C75EE/C75F6/C75FE/C7606 before occasionally showing
    // C760E. It is not the simple five-frame loop used by the fallback path.
    const STRU_34600_FRAME_SEQUENCE: [usize; 20] =
        [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3];
    let phase =
        (trace.frame_index / enemy_visuals.frame_duration.max(1)) % STRU_34600_FRAME_SEQUENCE.len();
    Some(&enemy_visuals.frames[STRU_34600_FRAME_SEQUENCE[phase]])
}

fn select_flying_dragon_dive_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if enemy_visuals.frames.len() < 9 {
        return None;
    }
    let x_px = object.x_velocity? / 0x0001_0000;
    let y_px = object.y_velocity? / 0x0001_0000;
    let facing = object.facing?;

    // ASM Dragon_Flying.asm loc_348CE reads the dive script's sprite byte and
    // indexes `off_345F6`: C75FE, C7616, C761E, C762E, C7636. Native does not
    // yet expose the script pointer, so match the exact pixel/frame velocity
    // bands from `unk_34554`.
    let frame_index = match (x_px.abs(), y_px, facing) {
        (1, 1, _) | (2, 1..=3, _) => 5,                   // LnkTo_unk_C7616
        (1, 2..=4, _) => 6,                               // LnkTo_unk_C761E
        (2, 0, _) => 2,                                   // LnkTo_unk_C75FE
        (2, -3..=-1, _) | (1, -1, _) => 7,                // LnkTo_unk_C762E
        (1, -4, _) | (1, -3..=-2, Direction::Right) => 8, // LnkTo_unk_C7636
        _ => return None,
    };
    Some(&enemy_visuals.frames[frame_index])
}

fn select_live_tar_monster_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    let _ = trace;
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x05 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 2
    {
        return None;
    }
    // ASM Tar_Monster.asm off_32290 dispatches stru_3272E (emerge,
    // C77A6->C77D6) and stru_3274C (submerge, C77D6->C77A6). The extracted
    // sheet keeps only the hidden body (frame 0 = C77A6) and the fully emerged
    // body (frame 1 = C77D6); frames 2.. are the stru_32700 submerge DEATH
    // tail. A live tar monster holds one of the two body frames off its phase
    // instead of free-running the whole sheet into the death frames. The 7-frame
    // emerge/submerge intermediates are not extracted, so the transition pops
    // rather than tweens (atlas limit, not a regression). contact_phase
    // TarMonsterHidden is set whenever tar_monster_state is not Visible/Attacking
    // (enemies.rs:9399-9408).
    let frame = if matches!(
        object.contact_phase,
        Some(NativeContactPhase::TarMonsterHidden)
    ) {
        0
    } else {
        1
    };
    Some(&enemy_visuals.frames[frame])
}

fn select_live_cloud_attack_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0E })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 13
    {
        return None;
    }
    // ASM Cloud.asm loc_35D9E/stru_35EC8: core exports the exact parent
    // mapping while the attack animation prepares its linked lightning.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

fn select_live_spinning_twins_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x14 })
        || enemy_visuals.frames.len() < 14
    {
        return None;
    }

    // ASM Spinning_Twins.asm loc_3BA74/loc_3B9B0/loc_3BB94: core exports
    // the fixed launch/falling mappings and the settled `stru_3B81C` loop;
    // a true loc_3BC56 death has no typed phase and remains routed through
    // the waited death selector below.
    match object.sprite_animation_phase? {
        phase @ (4 | 7..=13) => enemy_visuals.frames.get(usize::from(phase)),
        _ => None,
    }
}

fn select_live_fireball_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x19 })
        || enemy_visuals.frames.len() < 24
    {
        return None;
    }

    // ASM Fireball.asm loc_32F3E/loc_3304A/loc_33072: core exports the
    // exact live `off_33026` link as atlas phase 6..23. Death snapshots have
    // no typed phase and remain routed through `stru_345E4` above.
    match object.sprite_animation_phase? {
        phase @ 6..=23 => enemy_visuals.frames.get(usize::from(phase)),
        _ => None,
    }
}

fn select_live_tornado_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x18 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 5
    {
        return None;
    }

    // ASM Tornado.asm loc_35908 / `stru_35B10`: core exports only the three
    // live mappings. Atlas frames 3..4 are reserved for loc_359B2 death.
    match object.sprite_animation_phase? {
        phase @ 0..=2 => enemy_visuals.frames.get(usize::from(phase)),
        _ => None,
    }
}

fn select_live_armadillo_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x04 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 15
    {
        return None;
    }
    // ASM Armadillo.asm stru_3C632/stru_3C64C, loc_3E4CE/loc_3E50A and
    // loc_3C848: core exports the exact walk, charge-roll, or hurt mapping.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

fn select_live_robot_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x03 })
        || (object.wait_timer.is_some() && object.hit_points == Some(0))
        || enemy_visuals.frames.len() < 23
    {
        return None;
    }
    // ASM Robot.asm stru_3A6E2/stru_3A6FC and stru_3A716/stru_3A730:
    // core distinguishes the parent legs from the linked head by typed phase.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

fn select_live_sphere_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x06 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 3
    {
        return None;
    }
    // ASM Sphere.asm installs stru_36174 once at Enemy06_Sphere_Init and
    // Enemy06_Sphere_Loop never re-inits, so the live sphere spins through
    // exactly three frames C8120/C8128/C8130 (frames 0,1,2) via the `dc.b 2,$D`
    // rewind. Frames 3.. are the stru_36182 disappear/death tail and must not
    // appear while alive.
    let phase = (trace.frame_index / enemy_visuals.frame_duration.max(1)) % 3;
    Some(&enemy_visuals.frames[phase])
}

fn select_live_emo_rock_frame<'a>(
    _trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x1B })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 13
    {
        return None;
    }
    // ASM Emo_Rock.asm stru_3647E/stru_363FE/stru_36456: core exports the
    // current windup, drop, settled, or disappear mapping from its live timers.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

fn select_live_driller_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x1A })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 5
    {
        return None;
    }
    // ASM Driller.asm stru_3BCB2/loc_3BF4E: core exports either the live
    // C80E0/C80E8 parent phase or the fixed stru_3BCCE C80D8 hurt mapping.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

fn select_live_hand_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x17 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 7
    {
        return None;
    }

    // ASM Hand.asm installs live scripts by state: `loc_327F2` walking
    // `stru_32B6A` starts on C79AE (frame 0), `loc_328E0` jump attack
    // `stru_32B56` starts on C79C6 (frame 3), and `loc_32906` grab
    // `stru_32B50` holds C79DE (frame 6). Frames 7..13 are the `loc_32AA2`
    // death tail and must not appear while the Hand is alive.
    let frame = match object.contact_phase {
        Some(NativeContactPhase::HandJumping) => 3,
        Some(NativeContactPhase::HandGrabbing) => 6,
        _ => 0,
    };
    enemy_visuals.frames.get(frame)
}

fn select_live_lion_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x12 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 6
    {
        return None;
    }

    // ASM Lion.asm loc_3DFD8/loc_3E0F4 restore the saved walk animation
    // (`stru_3DEDA`, or `stru_3DEF4` for HP 8) before dispatching through
    // loc_3E11E's charge path. The extracted sheet appends death frames after
    // the six live C7E88..C7EB0 mappings, so live Lions must stay in that
    // script instead of free-running across the whole sheet.
    let durations: &[usize] = if object.hit_points == Some(8) {
        &[5, 4, 4, 5, 4, 5]
    } else {
        &[7, 6, 6, 7, 6, 6]
    };
    let cycle = durations.iter().sum::<usize>();
    if cycle == 0 {
        return None;
    }
    let mut phase = if let Some(walk_phase) = object.walk_phase {
        usize::from(walk_phase) % cycle
    } else {
        let offset = object.layout_index.unwrap_or(0) * 3;
        (trace.frame_index + offset) % cycle
    };
    for (frame_index, duration) in durations.iter().enumerate() {
        if phase < *duration {
            return enemy_visuals.frames.get(frame_index);
        }
        phase -= *duration;
    }
    enemy_visuals.frames.get(durations.len().saturating_sub(1))
}

fn select_live_ufo_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0F })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 6
    {
        return None;
    }

    // ASM UFO.asm Enemy0F_UFO_Hurt / loc_35228 sets `$47(a5)` on the first
    // hit and permanently replaces `stru_34C7E` with `stru_34C92`. Native's
    // surviving-hit state is HP 1; the terminal hit enters the death route.
    let first_frame = if object.hit_points == Some(1) { 3 } else { 0 };
    let phase = (trace.frame_index / 5) % 3;
    enemy_visuals.frames.get(first_frame + phase)
}

fn select_live_crab_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x09 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 10
    {
        return None;
    }

    if object.stationary {
        // ASM Crab.asm loc_3B104 disables animation and selects C7C4E until
        // loc_3B11E restores the walking script.
        return enemy_visuals.frames.get(9);
    }
    if object.damage_flash {
        // ASM Crab.asm loc_3B0A6 holds the sole eight-tick `stru_3AF7C`
        // mapping C7C16 before restoring `stru_3AF62`.
        return enemy_visuals.frames.get(6);
    }

    // ASM Crab.asm `stru_3AF62`: C7C1E..C7C46 use asymmetric
    // 8/6/6/8/6/6-tick spans before the animation command rewinds.
    let mut phase = (trace.frame_index + object.layout_index.unwrap_or(0) * 3) % 40;
    for (frame_index, duration) in [8, 6, 6, 8, 6, 6].into_iter().enumerate() {
        if phase < duration {
            return enemy_visuals.frames.get(frame_index);
        }
        phase -= duration;
    }
    enemy_visuals.frames.get(5)
}

fn select_live_goat_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x10 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 15
    {
        return None;
    }

    if object.damage_flash || object.wait_timer == Some(0xCBA8) {
        // ASM Goat.asm stru_3C9E4 and loc_3CB76: hurt and airborne
        // Berzerker knockback both disable the walk script and hold C7C56.
        return enemy_visuals.frames.get(6);
    }

    let charging = object.shared_walker_state.is_some_and(|state| state > 0);
    // ASM Goat.asm loc_3CA7E selects 8/6/6/8/6/6 for low-HP patrols;
    // loc_3CACA/loc_3CAE8 selects 6/4/4/6/4/4 at HP 4+. HP 4 retains
    // the slow alternate script, while HP 5+ uses the fast charge script.
    let fast = object
        .hit_points
        .is_some_and(|hp| hp >= 4 + u8::from(charging));
    let durations = if fast {
        [6, 4, 4, 6, 4, 4]
    } else {
        [8, 6, 6, 8, 6, 6]
    };
    let cycle = durations.iter().sum::<usize>();
    let mut phase = (trace.frame_index + object.layout_index.unwrap_or(0) * 3) % cycle;
    let first_frame = if charging { 9 } else { 0 };
    for (frame_index, duration) in durations.into_iter().enumerate() {
        if phase < duration {
            return enemy_visuals.frames.get(first_frame + frame_index);
        }
        phase -= duration;
    }
    enemy_visuals.frames.get(first_frame + 5)
}

fn select_live_fire_demon_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x00 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 13
    {
        return None;
    }

    if object.wait_timer.is_some() {
        // ASM Fire_Demon.asm loc_3A5E4 disables animation and fixes C773E
        // throughout the Berzerker-launch loop at loc_3A610.
        return enemy_visuals.frames.get(6);
    }

    let phase = usize::from(object.sprite_animation_phase?);
    if object.damage_flash {
        // ASM Fire_Demon.asm loc_3A5BE / `stru_3A34E` repeats walking
        // mappings 0..3 plus C773E across its ten animation entries.
        return [0, 1, 2, 3, 6, 0, 1, 2, 3, 6]
            .get(phase)
            .and_then(|frame| enemy_visuals.frames.get(*frame));
    }

    // ASM Fire_Demon.asm `stru_3A31A`/`stru_3A334` use the same six
    // mappings; core exports their HP-selected animation phase.
    enemy_visuals.frames.get(phase % 6)
}

fn select_live_archer_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x07 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 12
    {
        return None;
    }

    // ASM Archer.asm Shooting_Vertical/Shooting_Horizontal: core exports the
    // current loc_341BE mapping as the eight-entry live atlas phase.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

fn select_live_ninja_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x11 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 14
    {
        return None;
    }

    // ASM Ninja.asm stru_3D4EA and loc_3D670/loc_3D826/loc_3D84E/
    // loc_3D9DA: core exports the current scripted or fixed mapping.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

fn select_live_orca_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x08 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 14
    {
        return None;
    }

    if let Some(phase) = object.sprite_animation_phase {
        // ASM Orca.asm loc_3D2F4/loc_3D348/loc_3D420: core exports the
        // animation-disabled windup, lunge and landing mapping.
        return enemy_visuals.frames.get(usize::from(phase));
    }

    // ASM Orca.asm stru_3D0A2: twelve 9-tick entries, including C769E at
    // entry ten. Native has no per-object GfxObject animation timer yet.
    let script = [0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 11, 5];
    let offset = object.layout_index.unwrap_or(0) * 3;
    let entry = ((trace.frame_index + offset) / 9) % script.len();
    enemy_visuals.frames.get(script[entry])
}

fn select_live_scorpion_frame<'a>(
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> Option<&'a wildside_render::SpriteFrame> {
    if !matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x13 })
        || object.hit_points == Some(0)
        || enemy_visuals.frames.len() < 11
    {
        return None;
    }

    // ASM Scorpion.asm stru_3DB60/stru_3DB7A/stru_3DB8E and loc_3DD0E:
    // core exports the current walk, hurt, sting, or fixed knockback mapping.
    enemy_visuals
        .frames
        .get(usize::from(object.sprite_animation_phase?))
}

pub fn select_enemy_frame<'a>(
    trace: &NativeTraceFrame,
    enemy_visuals: &'a wildside_render::AnimatedSpriteVisual,
    object: &NativeObjectSnapshot,
) -> &'a wildside_render::SpriteFrame {
    if let Some(frame) = select_live_spinning_twins_frame(enemy_visuals, object) {
        return frame;
    }
    if object.hit_points == Some(0)
        && let NativeObjectKind::Enemy { object_type } = object.kind
        && let Some(remaining) = object.wait_timer
    {
        let death_frame = match object_type {
            // ASM Fire_Demon.asm loc_3A5A6 runs `stru_3A378`: three
            // 3-tick frames, then three 4-tick frames before deletion.
            0x00 if enemy_visuals.frames.len() >= 13 => {
                if remaining > 0x03 + 0x03 + 0x04 + 0x04 + 0x04 {
                    Some(7)
                } else if remaining > 0x03 + 0x04 + 0x04 + 0x04 {
                    Some(8)
                } else if remaining > 0x04 + 0x04 + 0x04 {
                    Some(9)
                } else if remaining > 0x04 + 0x04 {
                    Some(10)
                } else if remaining > 0x04 {
                    Some(11)
                } else {
                    Some(12)
                }
            }
            // ASM Diamond.asm loc_3AE28/loc_3AEE6 runs `stru_3ACC4`:
            // C8158 for $08, C8138 for $06, C8140/C8148 for $03 each,
            // then C8150 for $02 before the shared delete tail.
            0x01 if enemy_visuals.frames.len() >= 10 => {
                if remaining > 0x06 + 0x03 + 0x03 + 0x02 {
                    Some(5)
                } else if remaining > 0x03 + 0x03 + 0x02 {
                    Some(6)
                } else if remaining > 0x03 + 0x02 {
                    Some(7)
                } else if remaining > 0x02 {
                    Some(8)
                } else {
                    Some(9)
                }
            }
            // ASM Robot.asm loc_3A9D2 runs `stru_3A74A`: C8318 for $09,
            // then C8320 for $09 before deletion.
            0x03 if enemy_visuals.frames.len() >= 8 => {
                if remaining > 0x09 {
                    Some(6)
                } else {
                    Some(7)
                }
            }
            // ASM Sphere.asm Enemy06_Sphere_Kill runs `stru_36182`: C8108
            // for $0A, C8110 for $0A, then C8118 for $0A before deletion.
            0x06 if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x14 {
                    Some(3)
                } else if remaining > 0x0A {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Archer.asm loc_3414A runs `stru_33FC2`: C81B8 for $05,
            // C81F0 for $05, C81C8 for $05, then C81D0 for $23 before delete.
            0x07 if enemy_visuals.frames.len() >= 12 => {
                if remaining > 0x05 + 0x05 + 0x23 {
                    Some(8)
                } else if remaining > 0x05 + 0x23 {
                    Some(9)
                } else if remaining > 0x23 {
                    Some(10)
                } else {
                    Some(11)
                }
            }
            // ASM Armadillo.asm loc_3C886 runs `stru_3C69A`: C76D6/C76DE
            // flash with shrinking delays, then C76E6 for $03 before delete.
            0x04 if enemy_visuals.frames.len() >= 15 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x16, 12),
                    (0x04, 13),
                    (0x04, 12),
                    (0x03, 13),
                    (0x03, 12),
                    (0x02, 13),
                    (0x02, 12),
                    (0x01, 13),
                    (0x01, 12),
                    (0x01, 13),
                    (0x01, 12),
                    (0x01, 13),
                    (0x03, 14),
                ],
            ),
            // ASM Tar_Monster.asm loc_323E6 runs `stru_32700`: five
            // shrinking submerge frames before the status/delete tail.
            0x05 if enemy_visuals.frames.len() >= 7 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x09, 2),
                    (0x09, 3),
                    (0x03, 4),
                    (0x03, 3),
                    (0x09, 4),
                    (0x03, 5),
                    (0x03, 4),
                    (0x09, 5),
                    (0x03, 6),
                    (0x03, 5),
                    (0x09, 6),
                ],
            ),
            // ASM Orca.asm loc_3D268/loc_3D4BC runs `stru_3D11E`: C7656
            // and C7696 lead into shrinking C763E/C7646 flashes, then C764E.
            0x08 if enemy_visuals.frames.len() >= 11 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x09, 6),
                    (0x04, 7),
                    (0x03, 6),
                    (0x11, 8),
                    (0x04, 9),
                    (0x04, 8),
                    (0x03, 9),
                    (0x03, 8),
                    (0x02, 9),
                    (0x02, 8),
                    (0x02, 9),
                    (0x02, 8),
                    (0x02, 9),
                    (0x63, 10),
                ],
            ),
            // ASM Crab.asm loc_3B094 runs `stru_3AF82`: C7C16 for $14,
            // C7C06 for $0A, then C7C0E for $1E before deletion.
            0x09 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x1E {
                    Some(6)
                } else if remaining > 0x1E {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Tank.asm loc_3B480 runs `stru_3B29A`: C7EB8 for $10,
            // C7EC0 for $08, then C7F18 for $3C before deletion.
            0x0A if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x08 + 0x3C {
                    Some(3)
                } else if remaining > 0x3C {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Tank_Shooting.asm loc_3B6E6 runs `stru_3B518`: C7EB8
            // for $10, C7EC0 for $08, then C7EF8 for $3C before deletion.
            0x0B if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x08 + 0x3C {
                    Some(3)
                } else if remaining > 0x3C {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Dragon.asm loc_3CEFC runs `stru_3CCEC`: C755E/C75C6
            // lead into the C7566/C756E smoke/body loop before deletion.
            0x0D if enemy_visuals.frames.len() >= 10 => select_waited_sequence_frame(
                remaining,
                &[
                    (0x07, 6),
                    (0x06, 7),
                    (0x03, 6),
                    (0x16, 8),
                    (0x05, 9),
                    (0x04, 8),
                    (0x04, 9),
                    (0x0C, 8),
                    (0x05, 9),
                    (0x63, 8),
                ],
            ),
            // ASM Cloud.asm loc_35C0E runs `stru_35EBA`: C789E, C78A6,
            // then C78AE for seven ticks each before deletion.
            0x0E if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0E {
                    Some(6)
                } else if remaining > 0x07 {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Goat.asm loc_3CB64 runs `stru_3C9EA`: C7C56 for $1E,
            // C7C8E for $0A, then C7C96 for $37 before deletion.
            0x10 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x37 {
                    Some(6)
                } else if remaining > 0x37 {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Spinning_Twins.asm loc_3BC56 runs `stru_3B83C`:
            // C7CDE for $19, C7CE6 for $0F, then C7CEE for $14.
            0x14 if enemy_visuals.frames.len() >= 7 => {
                if remaining > 0x0F + 0x14 {
                    Some(4)
                } else if remaining > 0x14 {
                    Some(5)
                } else {
                    Some(6)
                }
            }
            // ASM Drips.asm loc_3392C runs `stru_33F06`: C7BC6/C7BCE/C7BD6
            // for four ticks each before the object deletes itself.
            0x16 if enemy_visuals.frames.len() >= 6 => {
                if remaining > 0x08 {
                    Some(3)
                } else if remaining > 0x04 {
                    Some(4)
                } else {
                    Some(5)
                }
            }
            // ASM Hand.asm loc_32AA2 runs `stru_32B78`: C79E6/C79EE repeat
            // three times, then C79F6/C79FE/C7A06/C7A0E/C7A16.
            0x17 if enemy_visuals.frames.len() >= 14 => {
                if remaining > 0x3C {
                    Some(7)
                } else if remaining > 0x36 {
                    Some(8)
                } else if remaining > 0x30 {
                    Some(7)
                } else if remaining > 0x2A {
                    Some(8)
                } else if remaining > 0x24 {
                    Some(7)
                } else if remaining > 0x1E {
                    Some(8)
                } else if remaining > 0x18 {
                    Some(9)
                } else if remaining > 0x12 {
                    Some(10)
                } else if remaining > 0x0C {
                    Some(11)
                } else if remaining > 0x06 {
                    Some(12)
                } else {
                    Some(13)
                }
            }
            // ASM Ninja.asm loc_3DB4E runs `stru_3D50A`: C7D56 for $14,
            // C7D5E for $0A, then C7D66 for $2D before deletion.
            0x11 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x2D {
                    Some(6)
                } else if remaining > 0x2D {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Lion.asm loc_3E378 runs `stru_3DF48`: C7E38, C7E40,
            // then C7E48 for ten ticks each before deletion.
            0x12 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0A + 0x0A {
                    Some(6)
                } else if remaining > 0x0A {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Scorpion.asm loc_3DDEE runs `stru_3DB80`: C7DD6 for
            // $0A, C7DC6 for $14, then C7DCE for $2D before deletion.
            0x13 if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x14 + 0x2D {
                    Some(6)
                } else if remaining > 0x2D {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM UFO.asm loc_35174/loc_35180 runs parent `stru_34CA0`:
            // C83B0, C83B8, C83C0 for 7 ticks each before loc_351B6 delete.
            0x0F if enemy_visuals.frames.len() >= 9 => {
                if remaining > 0x0E {
                    Some(6)
                } else if remaining > 0x07 {
                    Some(7)
                } else {
                    Some(8)
                }
            }
            // ASM Dragon_Flying.asm loc_3492C runs shared `stru_345E4`:
            // four 2-tick frames while the flying parent hibernates before
            // loc_349A6 deletes it.
            0x0C if enemy_visuals.frames.len() >= 4 => {
                if remaining > 0x06 {
                    Some(0)
                } else if remaining > 0x04 {
                    Some(1)
                } else if remaining > 0x02 {
                    Some(2)
                } else {
                    Some(3)
                }
            }
            // ASM Fireball.asm Enemy19_Fireball_Kill reuses `stru_345E4`:
            // four 2-tick frames while the fireball hibernates before
            // loc_32ED8 deletes it.
            0x19 if enemy_visuals.frames.len() >= 4 => {
                if remaining > 0x06 {
                    Some(0)
                } else if remaining > 0x04 {
                    Some(1)
                } else if remaining > 0x02 {
                    Some(2)
                } else {
                    Some(3)
                }
            }
            // ASM Tornado.asm loc_359B2 runs `stru_35B1E`: C783E/C7846
            // alternate across $06/$03/$04/$04/$03/$06 tick spans.
            0x18 if enemy_visuals.frames.len() >= 5 => {
                if remaining > 0x03 + 0x04 + 0x04 + 0x03 + 0x06 {
                    Some(3)
                } else if remaining > 0x04 + 0x04 + 0x03 + 0x06 {
                    Some(4)
                } else if remaining > 0x04 + 0x03 + 0x06 {
                    Some(3)
                } else if remaining > 0x03 + 0x06 {
                    Some(4)
                } else if remaining > 0x06 {
                    Some(3)
                } else {
                    Some(4)
                }
            }
            // ASM Emo_Rock.asm loc_36346 runs `stru_36468`: five 6-tick
            // frames C7F68/C7F70/C7F78/C7F80/C7F88 before deletion.
            0x1B if enemy_visuals.frames.len() >= 8 => {
                if remaining > 0x18 {
                    Some(3)
                } else if remaining > 0x12 {
                    Some(4)
                } else if remaining > 0x0C {
                    Some(5)
                } else if remaining > 0x06 {
                    Some(6)
                } else {
                    Some(7)
                }
            }
            // ASM Driller.asm loc_3BF1C runs parent `stru_3BCD4`: C80D8 for
            // $0A, C80C8 for $14, then C80D0 for $28 before deletion.
            0x1A if enemy_visuals.frames.len() >= 5 => {
                if remaining > 0x14 + 0x28 {
                    Some(2)
                } else if remaining > 0x28 {
                    Some(3)
                } else {
                    Some(4)
                }
            }
            // ASM Big_Hopping_Skull.asm loc_3664E runs `stru_367FA`:
            // C7FA0 for $18, C7FE8 for $0C, C7FF0 for $0A, then C7FF8
            // for $64 before deletion.
            0x1D if enemy_visuals.frames.len() >= 5 => {
                if remaining > 0x0C + 0x0A + 0x64 {
                    Some(1)
                } else if remaining > 0x0A + 0x64 {
                    Some(2)
                } else if remaining > 0x64 {
                    Some(3)
                } else {
                    Some(4)
                }
            }
            // ASM loc_39EBE/loc_39F7A shared boss-head death spin uses
            // C8620/C8618/C8610/C8608/C8600 sweeps, then ten fast spin loops.
            0x22 if enemy_visuals.frames.len() >= 5 => {
                Some(boss_head_death_frame_loc_39ebe(remaining))
            }
            _ => None,
        };
        if let Some(frame_index) = death_frame {
            return &enemy_visuals.frames[frame_index];
        }
    }
    if matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x1C })
        && object.hit_points == Some(0)
        && let Some(remaining) = object.wait_timer
        && enemy_visuals.frames.len() >= 8
    {
        // ASM Mini_Hopping_Skull.asm loc_3F158 runs `stru_3E9AC`: C8040 for
        // $0A ticks, then C8048 for $1E ticks before deletion.
        let death_frame = if remaining > 0x1E { 6 } else { 7 };
        return &enemy_visuals.frames[death_frame];
    }
    if let Some(frame) = select_live_walking_dragon_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_rock_tank_body_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_diamond_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_flying_dragon_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_tar_monster_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_cloud_attack_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_fireball_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_tornado_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_armadillo_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_robot_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_sphere_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_emo_rock_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_driller_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_hand_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_lion_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_ufo_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_crab_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_goat_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_archer_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_ninja_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_orca_frame(trace, enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_scorpion_frame(enemy_visuals, object) {
        return frame;
    }
    if let Some(frame) = select_live_fire_demon_frame(enemy_visuals, object) {
        return frame;
    }
    let keeps_base_animation = matches!(
        object.kind,
        NativeObjectKind::Enemy {
            object_type: 0x01 | 0x0C
        }
    );
    if object.stationary && !keeps_base_animation {
        return &enemy_visuals.frames[0];
    }
    // Offset animation by layout_index so enemies of the same type don't
    // animate in lockstep. ASM uses per-object animation counters at
    // $36(a3)/$38(a3); this is a visual approximation.
    let offset = object.layout_index.unwrap_or(0);
    let frame_index = ((trace.frame_index + offset * 3) / enemy_visuals.frame_duration.max(1))
        % enemy_visuals.frames.len();
    &enemy_visuals.frames[frame_index]
}

pub fn push_enemy_sprite(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    object_type: u8,
) {
    let Some(enemy_visuals) = level_visuals.enemies.families.get(&object_type) else {
        let colour = if object.damage_flash {
            ENEMY_FLASH_COLOUR
        } else {
            enemy_family_colour(object_type)
        };
        commands.push(object_rect(object, trace, 16, 16, colour));
        return;
    };

    let (screen_x, screen_y) = player_screen_position(object, trace);
    let mut facing = object.facing.unwrap_or(Direction::Right);
    if object_type == 0x22
        && object.hit_points == Some(0)
        && let Some(remaining) = object.wait_timer
    {
        facing = boss_head_death_facing_loc_39ebe(facing, remaining);
    }
    // Enemy VRAM art is stored facing left (the ROM uses the H-flip bit to
    // show right-facing sprites). Invert the facing for rendering so that
    // Direction::Right flips the left-facing art to face right.
    let render_facing = opposite_direction(facing);

    if is_rock_tank_child_snapshot(object_type, object, trace) {
        if let Some(turret_frame) =
            rock_tank_turret_frame(object, trace, enemy_visuals, object_type)
        {
            // ASM Tank.asm loc_3B372 copies parent X/Y into the linked turret
            // GfxObject and subtracts 8 from child Y. ASM Make_SpriteFromGfxObject
            // loc_11EE still subtracts the child frame height, so do not add
            // the 8px back here; the child trace already carries the ROM offset.
            let turret_screen_y = rock_tank_turret_screen_y(object_type, screen_y);
            let (turret_x, turret_y) =
                sprite_draw_position(&turret_frame, screen_x, turret_screen_y, render_facing);
            let palette = enemy_palette_for_damage_flash(
                select_enemy_palette(
                    enemy_visuals,
                    enemy_palette_variant(level_visuals, object_type, object.palette_variant),
                ),
                rock_tank_damage_flash(object, trace, object_type),
            );
            commands.push(DrawCommand::Sprite {
                x: turret_x,
                y: turret_y,
                tile_sheet: Arc::new(turret_frame.tile_sheet.clone()),
                palette: Arc::new(palette),
                tiles: build_sprite_tiles(&turret_frame, render_facing, 0),
            });
        }
        return;
    }

    if enemy_visuals.frames.is_empty() {
        let colour = if object.damage_flash {
            ENEMY_FLASH_COLOUR
        } else {
            enemy_family_colour(object_type)
        };
        commands.push(object_rect(object, trace, 16, 16, colour));
        return;
    }

    let frame = select_enemy_frame(trace, enemy_visuals, object);
    let (draw_x, draw_y) = sprite_draw_position(frame, screen_x, screen_y, render_facing);
    let palette = enemy_palette_for_heady_metal_sub_379e8(
        select_enemy_palette(
            enemy_visuals,
            enemy_palette_variant(level_visuals, object_type, object.palette_variant),
        ),
        object,
        object_type,
    );
    let palette = enemy_palette_for_damage_flash(
        &palette,
        enemy_damage_palette_white_phase(object_type, object.damage_flash),
    );
    commands.push(DrawCommand::Sprite {
        x: draw_x,
        y: draw_y,
        tile_sheet: Arc::new(frame.tile_sheet.clone()),
        palette: Arc::new(palette),
        tiles: build_sprite_tiles(frame, render_facing, 0),
    });
    if object_type == 0x20 {
        push_heady_metal_sub_37abc_body_piece_markers(commands, object, trace);
    }

    if is_rock_tank_parent_without_child_snapshot(object_type, object, trace)
        && let Some(turret_frame) =
            rock_tank_turret_frame(object, trace, enemy_visuals, object_type)
    {
        // Fallback for traces that only expose the parent. ASM Tank.asm
        // loc_3B372 creates the real child at parent_y-8 before drawing.
        let turret_screen_y = rock_tank_turret_screen_y(object_type, screen_y - 8);
        let (turret_x, turret_y) =
            sprite_draw_position(&turret_frame, screen_x, turret_screen_y, render_facing);
        let palette = enemy_palette_for_damage_flash(
            select_enemy_palette(
                enemy_visuals,
                enemy_palette_variant(level_visuals, object_type, object.palette_variant),
            ),
            object.damage_flash,
        );
        commands.push(DrawCommand::Sprite {
            x: turret_x,
            y: turret_y,
            tile_sheet: Arc::new(turret_frame.tile_sheet.clone()),
            palette: Arc::new(palette),
            tiles: build_sprite_tiles(&turret_frame, render_facing, 0),
        });
    }
}

fn enemy_damage_palette_white_phase(object_type: u8, damage_flash: bool) -> bool {
    // ASM Dragon.asm loc_3CEBE swaps to stru_3CCE6 and hibernates until the
    // hurt animation completes; it does not write the enemy palette. Native's
    // shared damage_flash timer also carries that animation duration, so it
    // must not imply a white palette for walking Dragons.
    damage_flash && object_type != 0x0D
}

fn push_heady_metal_sub_37abc_body_piece_markers(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) {
    let anchor_x = (object.x_pos >> 16) as u16;
    let anchor_y = (object.y_pos >> 16) as u16;
    let kid_x = trace.kid_x_pos.map_or(anchor_x, |x| (x >> 16) as u16);
    let kid_y = trace.kid_y_pos.map_or(anchor_y, |y| (y >> 16) as u16);
    let camera_x = (trace.camera_x_pos >> 16) as i32;
    let camera_y = (trace.camera_y_pos >> 16) as i32;

    if !trace.heady_metal_body_pieces_sub_37abc.is_empty() {
        for piece in &trace.heady_metal_body_pieces_sub_37abc {
            // ASM loc_37AC6 refreshes each non-null `$FFFFFB7E..$FFFFFBAA`
            // worker through `sub_37AF0`; the trace carries the refreshed
            // mapping so rendering does not have to reconstruct it.
            push_boss_eye_attack_marker_sub_37abc(
                commands,
                i32::from(piece.x_pos) - camera_x,
                i32::from(piece.y_pos) - camera_y,
                piece.sprite_mapping_index,
            );
        }
        return;
    }

    for &(x_offset, y_offset) in &wildside_core::HEADY_METAL_SUB_37ABC_BODY_PIECE_OFFSETS_LOC_39266
    {
        // ASM Enemy20_HeadyMetal_Init (kid.asm:39266-39321) seeds the twelve
        // `$FFFFFB7E` body-piece workers with these offsets. ASM sub_37ABC
        // then refreshes each worker's mapping every `loc_3763A` loop.
        let piece_x = anchor_x.wrapping_add_signed(x_offset);
        let piece_y = anchor_y.wrapping_add_signed(y_offset);
        let mapping_index = wildside_core::heady_metal_piece_sprite_index_sub_37A22(
            false, 0, piece_x, piece_y, kid_x, kid_y,
        );
        push_boss_eye_attack_marker_sub_37abc(
            commands,
            i32::from(piece_x) - camera_x,
            i32::from(piece_y) - camera_y,
            mapping_index,
        );
    }
}

fn is_rock_tank_type(object_type: u8) -> bool {
    matches!(object_type, 0x0A | 0x0B)
}

fn is_rock_tank_child_snapshot(
    object_type: u8,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) -> bool {
    // ASM Tank.asm loc_3B372 / Tank_Shooting.asm loc_3B5FA allocate the
    // linked turret as a zeroed GfxObject. Native traces may expose that child
    // as either `None` or `Some(0)` HP depending on the source path; require a
    // live same-layout parent so true zero-HP death parents still render their
    // death animation.
    is_rock_tank_type(object_type)
        && matches!(object.hit_points, None | Some(0))
        && trace.active_objects.iter().any(|candidate| {
            matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
                && candidate.layout_index == object.layout_index
                && candidate.hit_points.is_some_and(|hit_points| hit_points > 0)
        })
}

fn rock_tank_damage_flash(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    object_type: u8,
) -> bool {
    if object.damage_flash {
        return true;
    }
    if !is_rock_tank_child_snapshot(object_type, object, trace) {
        return false;
    }

    trace.active_objects.iter().any(|candidate| {
        matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
            && candidate.layout_index == object.layout_index
            && candidate.hit_points.is_some()
            && candidate.damage_flash
    })
}

fn is_rock_tank_parent_without_child_snapshot(
    object_type: u8,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
) -> bool {
    if !is_rock_tank_type(object_type)
        || object.hit_points.is_none()
        || is_rock_tank_child_snapshot(object_type, object, trace)
    {
        return false;
    }
    !trace.active_objects.iter().any(|candidate| {
        matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
            && candidate.layout_index == object.layout_index
            && is_rock_tank_child_snapshot(object_type, candidate, trace)
    })
}

fn rock_tank_turret_screen_y(object_type: u8, child_screen_y: i32) -> i32 {
    if object_type == 0x0B {
        // ASM Tank_Shooting.asm loc_3B5FA copies the same `y_pos-8` child
        // setup as the plain tank, but the shooting turret frames are
        // top-anchored muzzle art. Compensate the child offset so those frames
        // share the tread baseline instead of stacking a second body above it.
        child_screen_y + 8
    } else {
        child_screen_y
    }
}

fn rock_tank_turret_frame(
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    enemy_visuals: &wildside_render::AnimatedSpriteVisual,
    object_type: u8,
) -> Option<wildside_render::SpriteFrame> {
    let hit_points = object.hit_points.or_else(|| {
        trace
            .active_objects
            .iter()
            .find(|candidate| {
                matches!(candidate.kind, NativeObjectKind::Enemy { object_type: candidate_type } if candidate_type == object_type)
                    && candidate.layout_index == object.layout_index
                    && candidate.hit_points.is_some()
            })
            .and_then(|candidate| candidate.hit_points)
    });
    let specs = match object_type {
        // ASM Tank.asm loc_3B372 + `stru_3B27E`: plain RockTank allocates a
        // second GfxObject at `$3A(a5)` for the turret and keeps it 8px above
        // the body while the body animation uses the first object.
        0x0A => &ROCK_TANK_TURRET_STRU_3B27E[..],
        // ASM Tank_Shooting.asm loc_3B5FA/loc_3B682: HP 4 uses `stru_3B4D8`,
        // HP 2-3 use `stru_3B4F6`, again on the child GfxObject at `$3A(a5)`.
        0x0B if hit_points == Some(4) => &ROCK_TANK_SHOOTING_FAST_STRU_3B4D8[..],
        0x0B => &ROCK_TANK_SHOOTING_SLOW_STRU_3B4F6[..],
        _ => return None,
    };
    let spec = scripted_tank_turret_frame(trace, specs)?;
    let tile_sheet = enemy_visuals.frames.first()?.tile_sheet.clone();
    Some(wildside_render::SpriteFrame {
        origin_x: spec.origin_x,
        origin_y: spec.origin_y,
        width_px: spec.width_px,
        height_px: spec.height_px,
        tile_base_index: spec.tile_base_index,
        tile_sheet,
    })
}

fn scripted_tank_turret_frame(
    trace: &NativeTraceFrame,
    specs: &[TankTurretFrameSpec],
) -> Option<TankTurretFrameSpec> {
    let cycle_frames = specs.iter().map(|spec| spec.duration_frames).sum::<usize>();
    if cycle_frames == 0 {
        return None;
    }
    let mut phase = trace.frame_index % cycle_frames;
    for spec in specs {
        if phase < spec.duration_frames {
            return Some(*spec);
        }
        phase -= spec.duration_frames;
    }
    specs.last().copied()
}

pub fn select_enemy_palette(
    visuals: &wildside_render::AnimatedSpriteVisual,
    variant: u8,
) -> &wildside_render::Palette {
    if variant > 0
        && let Some(pal) = visuals.extra_palettes.get(usize::from(variant) - 1)
    {
        return pal;
    }
    &visuals.palette
}

/// ASM `Load_EnemyArtPaletteToVRAM` `loc_36D16`: use the palette that the
/// level's physical enemy slot loaded when a live object has no explicit
/// palette override. Object-local variants still describe runtime effects.
fn enemy_palette_variant(level_visuals: &NativeLevelVisuals, object_type: u8, explicit: u8) -> u8 {
    if explicit != 0 {
        explicit
    } else {
        level_visuals
            .enemies
            .layout_palette_variants
            .get(&object_type)
            .copied()
            .unwrap_or(0)
    }
}

pub fn push_platform_tiles(
    commands: &mut Vec<DrawCommand>,
    object: &NativeObjectSnapshot,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    kind: u8,
) {
    let width_px = platform_snapshot_dimension_px(object, "platform_width_px");
    let height_px = platform_snapshot_dimension_px(object, "platform_height_px");
    let Some(dispatch) = platform_snapshot_u8(object, "platform_render_flags")
        .zip(platform_snapshot_u16(object, "platform_mapping_addr_word"))
        .and_then(|(render_flags, mapping_addr_word)| {
            wildside_core::platform_sprite_dispatch_loc_244C(PlatformSpriteDispatchInputLoc244C {
                world_x_px: (object.x_pos >> 16) as i16,
                world_y_px: (object.y_pos >> 16) as i16,
                camera_x_px: (trace.camera_x_pos >> 16) as i16,
                camera_y_px: (trace.camera_y_pos >> 16) as i16,
                width_px: width_px as i16,
                height_px: height_px as i16,
                mapping_addr_word,
                render_flags,
            })
        })
    else {
        return;
    };
    let variant = dispatch.variant;
    let art = platform_art_for_mapping_word_loc_2548(object, level_visuals).unwrap_or_else(|| {
        let frame = match variant {
            PlatformSpriteVariantLoc244C::Loc251C => level_visuals.platforms.vertical.clone(),
            _ => match kind {
                0x00 => level_visuals.platforms.horizontal.clone(),
                _ => level_visuals.platforms.vertical.clone(),
            },
        };
        PlatformArtSourceLoc2548 {
            frame,
            palette: level_visuals.platforms.palette.clone(),
        }
    });
    let frame = &art.frame;
    let (screen_x, screen_y) = player_screen_position(object, trace);
    let width_cells = dispatch.width_tiles_minus_one.saturating_add(1);
    let height_cells = dispatch.height_tiles_minus_one.saturating_add(1);
    let tile_sheet = Arc::new(frame.tile_sheet.clone());
    let palette = Arc::new(art.palette);
    for cell in platform_sprite_cells_loc_24AC(variant, dispatch, width_cells, height_cells) {
        commands.push(DrawCommand::Sprite {
            x: screen_x - i32::from(frame.origin_x) + i32::from(cell.column) * 16,
            y: screen_y - i32::from(frame.height_px)
                + i32::from(frame.origin_y)
                + cell.y_bias
                + i32::from(cell.row) * 16,
            tile_sheet: Arc::clone(&tile_sheet),
            palette: Arc::clone(&palette),
            tiles: platform_cell_tiles_loc_2548(frame, cell.mapping_tile_offset),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlatformSpriteCellLoc24AC {
    column: u16,
    row: u16,
    y_bias: i32,
    mapping_tile_offset: u16,
}

#[allow(non_snake_case)]
fn platform_sprite_cells_loc_24AC(
    variant: PlatformSpriteVariantLoc244C,
    dispatch: PlatformSpriteDispatchLoc244C,
    width_cells: u16,
    height_cells: u16,
) -> Vec<PlatformSpriteCellLoc24AC> {
    let cells = match variant {
        PlatformSpriteVariantLoc244C::Loc24CC => {
            wildside_core::platform_strip_sprites_loc_24CC(dispatch, 0)
                .sprites
                .into_iter()
                .map(|sprite| platform_sprite_entry_cell_loc_2548(dispatch, sprite))
                .collect()
        }
        PlatformSpriteVariantLoc244C::Loc24F0 => {
            wildside_core::platform_strip_sprites_loc_24F0(dispatch, 0)
                .sprites
                .into_iter()
                .map(|sprite| platform_sprite_entry_cell_loc_2548(dispatch, sprite))
                .collect()
        }
        PlatformSpriteVariantLoc244C::Loc251C => {
            wildside_core::platform_strip_sprites_loc_251C(dispatch, 0)
                .sprites
                .into_iter()
                .map(|sprite| platform_sprite_entry_cell_loc_2548(dispatch, sprite))
                .collect()
        }
        PlatformSpriteVariantLoc244C::Loc24D4 => (0..height_cells)
            .flat_map(|row| (0..width_cells).map(move |column| platform_cell(column, row, 0, 0)))
            .collect(),
        PlatformSpriteVariantLoc244C::Loc24C0 => vec![platform_sprite_entry_cell_loc_2548(
            dispatch,
            wildside_core::platform_single_sprite_loc_24C0(dispatch, 0).sprite,
        )],
    };

    // ASM loc_256E (kid.asm:4456): after Make_SpritesFromPlatforms emits all
    // platform sprites, publish the first free 8-byte sprite slot.
    let next_sprite_slot_addr = wildside_core::finish_platform_sprite_list_loc_256E(
        0xFFFF_0000u32.wrapping_add((cells.len() as u32).wrapping_mul(8)),
    );
    debug_assert_eq!(
        next_sprite_slot_addr.wrapping_sub(0xFFFF_0000) / 8,
        cells.len() as u32,
    );

    cells
}

fn platform_cell(
    column: u16,
    row: u16,
    y_bias: i32,
    mapping_tile_offset: u16,
) -> PlatformSpriteCellLoc24AC {
    PlatformSpriteCellLoc24AC {
        column,
        row,
        y_bias,
        mapping_tile_offset,
    }
}

#[allow(non_snake_case)]
fn platform_sprite_entry_cell_loc_2548(
    dispatch: PlatformSpriteDispatchLoc244C,
    sprite: PlatformSpriteEntryLoc2548,
) -> PlatformSpriteCellLoc24AC {
    // ASM loc_2548 writes sprite table coordinates with the VDP +$80 screen
    // bias already applied. The frame renderer stores cell offsets relative to
    // the unbiased platform origin, so undo the dispatch origin here.
    let dx = (sprite.x_word as i16).wrapping_sub(dispatch.screen_x_word);
    let dy = (sprite.y_word as i16).wrapping_sub(dispatch.screen_y_word);
    let column = (dx / 0x10).max(0) as u16;
    let row = (dy / 0x10).max(0) as u16;
    let mapping_addr_word = sprite
        .mapping_word
        .wrapping_sub(0x8000)
        .wrapping_sub(dispatch.mapping_addr_word);
    platform_cell(
        column,
        row,
        i32::from(dy) - i32::from(row) * 0x10,
        mapping_addr_word,
    )
}

#[allow(non_snake_case)]
fn platform_cell_tiles_loc_2548(frame: &SpriteFrame, mapping_tile_offset: u16) -> Vec<SpriteTile> {
    let first_tile = frame.tile_base_index.saturating_add(mapping_tile_offset);
    vec![
        SpriteTile {
            offset_x: 0,
            offset_y: 0,
            tile: TileRef {
                tile_index: first_tile,
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
        SpriteTile {
            offset_x: 0,
            offset_y: 8,
            tile: TileRef {
                tile_index: first_tile.saturating_add(1),
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
        SpriteTile {
            offset_x: 8,
            offset_y: 0,
            tile: TileRef {
                tile_index: first_tile.saturating_add(2),
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
        SpriteTile {
            offset_x: 8,
            offset_y: 8,
            tile: TileRef {
                tile_index: first_tile.saturating_add(3),
                palette_line: 0,
                priority: false,
                flip_x: false,
                flip_y: false,
            },
        },
    ]
}

fn platform_snapshot_dimension_px(object: &NativeObjectSnapshot, key: &str) -> u16 {
    object
        .debug_fields
        .get(key)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(15)
}

fn platform_snapshot_u8(object: &NativeObjectSnapshot, key: &str) -> Option<u8> {
    object
        .debug_fields
        .get(key)
        .and_then(|value| value.parse::<u8>().ok())
}

fn platform_snapshot_u16(object: &NativeObjectSnapshot, key: &str) -> Option<u16> {
    object
        .debug_fields
        .get(key)
        .and_then(|value| value.parse::<u16>().ok())
}

#[allow(non_snake_case)]
struct PlatformArtSourceLoc2548 {
    frame: SpriteFrame,
    palette: Palette,
}

const PLATFORM_BLOCK_ART_VRAM_BASE_LOC_E0E4E: u16 = 0x0220;

#[allow(non_snake_case)]
fn platform_art_for_mapping_word_loc_2548(
    object: &NativeObjectSnapshot,
    level_visuals: &NativeLevelVisuals,
) -> Option<PlatformArtSourceLoc2548> {
    // ASM loc_244C/loc_2548 (kid.asm:4352,4452): `$18(a2)` is copied into the
    // sprite mapping word. `PlatformArtAddr_Table` at loc_2822 stores VRAM tile
    // indices; Initialize_Platforms loads Horizontal_platform.bin at byte
    // `$1280` (tile `$94`) and Vertical_platform.bin at byte `$D5C0`
    // (tile `$6AE`). `$25A` falls in the common block-art range starting at
    // `unk_E0E4E` / VRAM tile `$220`.
    match object
        .debug_fields
        .get("platform_mapping_addr_word")
        .and_then(|value| value.parse::<u16>().ok())?
    {
        0x0094 => Some(PlatformArtSourceLoc2548 {
            frame: level_visuals.platforms.horizontal.clone(),
            palette: level_visuals.platforms.palette.clone(),
        }),
        0x06AE => Some(PlatformArtSourceLoc2548 {
            frame: level_visuals.platforms.vertical.clone(),
            palette: level_visuals.platforms.palette.clone(),
        }),
        0x025A => Some(PlatformArtSourceLoc2548 {
            frame: SpriteFrame {
                origin_x: 0,
                origin_y: 0x10,
                width_px: 0x10,
                height_px: 0x10,
                tile_base_index: 0x025A - PLATFORM_BLOCK_ART_VRAM_BASE_LOC_E0E4E,
                tile_sheet: level_visuals.block_tiles.clone(),
            },
            palette: level_visuals.palettes.foreground.clone(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked_dynamic_art_frame(marker: u8, tile_count: usize) -> SpriteFrame {
        SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 32,
            height_px: 40,
            tile_base_index: 0,
            tile_sheet: wildside_render::TileSheet {
                tiles: (0..tile_count)
                    .map(|tile| wildside_render::Tile {
                        pixels: core::array::from_fn(|pixel| {
                            marker.wrapping_add((tile * 64 + pixel) as u8)
                        }),
                    })
                    .collect(),
            },
        }
    }

    #[test]
    fn loc_ad4_page_applies_word_prefix_without_rounding_to_tiles() {
        let old = marked_dynamic_art_frame(0x10, 20);
        let new = marked_dynamic_art_frame(0x80, 20);
        let mut resident = PlayerDynamicArtVramPageLocAd4::from_resident_frame(&old);

        resident
            .apply_frame_rendered_prefixes(
                &new,
                &[
                    wildside_core::vdp::MemoryDmaFrameRenderedResidency {
                        words: 256,
                        committed_words_before_frame_rendered: 252,
                    },
                    wildside_core::vdp::MemoryDmaFrameRenderedResidency {
                        words: 64,
                        committed_words_before_frame_rendered: 0,
                    },
                ],
            )
            .expect("ASM frame-7968 loc_AD4 residency is within the 20-tile page");

        assert_eq!(
            resident.tile_sheet.tiles[..15],
            new.tile_sheet.tiles[..15],
            "ASM loc_AD4 makes the first fifteen 16-word tiles fully resident"
        );
        assert_eq!(
            resident.tile_sheet.tiles[15].pixels[..48],
            new.tile_sheet.tiles[15].pixels[..48],
            "the next 12 committed words replace three rows of tile sixteen",
        );
        assert_eq!(
            resident.tile_sheet.tiles[15].pixels[48..],
            old.tile_sheet.tiles[15].pixels[48..],
            "jgenesis FrameRendered precedes the final row's four DMA words",
        );
        assert_eq!(
            resident.tile_sheet.tiles[16..],
            old.tile_sheet.tiles[16..],
            "the second 64-word DMA stays resident from the prior page",
        );
    }

    fn synthetic_player_frame() -> wildside_render::SpriteFrame {
        wildside_render::SpriteFrame {
            origin_x: 8,
            origin_y: 0,
            width_px: 16,
            height_px: 16,
            tile_base_index: 0,
            tile_sheet: wildside_render::TileSheet {
                tiles: vec![
                    wildside_render::Tile { pixels: [0; 64] },
                    wildside_render::Tile { pixels: [0; 64] },
                    wildside_render::Tile { pixels: [0; 64] },
                    wildside_render::Tile { pixels: [0; 64] },
                ],
            },
        }
    }

    fn labelled_frame(tile_base_index: u16) -> wildside_render::SpriteFrame {
        wildside_render::SpriteFrame {
            tile_base_index,
            ..synthetic_player_frame()
        }
    }

    fn marked_tile_sheet(tile_count: usize, marker: u8) -> wildside_render::TileSheet {
        wildside_render::TileSheet {
            tiles: vec![
                wildside_render::Tile {
                    pixels: [marker; 64]
                };
                tile_count
            ],
        }
    }

    fn marked_frame(
        tile_base_index: u16,
        origin_x: i16,
        origin_y: i16,
        height_px: u16,
        marker: u8,
    ) -> wildside_render::SpriteFrame {
        wildside_render::SpriteFrame {
            origin_x,
            origin_y,
            width_px: 16,
            height_px,
            tile_base_index,
            tile_sheet: marked_tile_sheet(64, marker),
        }
    }

    fn marked_palette(marker: u16) -> wildside_render::Palette {
        wildside_render::Palette {
            colours: (0..16)
                .map(|offset| MdColour::from_word(marker + offset))
                .collect(),
        }
    }

    fn labelled_player_visuals() -> wildside_render::PlayerFormVisuals {
        wildside_render::PlayerFormVisuals {
            palette: wildside_render::Palette {
                colours: Vec::new(),
            },
            idle: labelled_frame(0x0100),
            walk: vec![
                labelled_frame(0x0200),
                labelled_frame(0x0201),
                labelled_frame(0x0202),
            ],
            skid: Some(labelled_frame(0x0250)),
            flight_spin: vec![
                labelled_frame(0x0300),
                labelled_frame(0x0301),
                labelled_frame(0x0302),
            ],
            charge: vec![labelled_frame(0x0400), labelled_frame(0x0401)],
            climb: vec![
                labelled_frame(0x0500),
                labelled_frame(0x0501),
                labelled_frame(0x0502),
            ],
            axe: Vec::new(),
            slope_walk: vec![labelled_frame(0x0600), labelled_frame(0x0601)],
            slope_walk_left: vec![labelled_frame(0x0700), labelled_frame(0x0701)],
            airborne: vec![
                labelled_frame(0x0800),
                labelled_frame(0x0801),
                labelled_frame(0x0802),
                labelled_frame(0x0803),
            ],
            death: None,
            pull_up: vec![
                labelled_frame(0x0900),
                labelled_frame(0x0901),
                labelled_frame(0x0902),
            ],
            slash_ground: vec![
                labelled_frame(0x0A00),
                labelled_frame(0x0A01),
                labelled_frame(0x0A02),
                labelled_frame(0x0A03),
            ],
            slash_air: vec![
                labelled_frame(0x0B00),
                labelled_frame(0x0B01),
                labelled_frame(0x0B02),
                labelled_frame(0x0B03),
            ],
            transform_palette: None,
            transform_steps: Vec::new(),
        }
    }

    fn minimal_level_visuals_for_platforms() -> wildside_render::NativeLevelVisuals {
        let foreground_layout_bytes = vec![0; 20 * 14];
        let foreground_layout =
            wildside_render::ForegroundLayout::parse(&foreground_layout_bytes, 1, 1)
                .expect("foreground layout");
        let empty_quad = wildside_render::TileQuad {
            tiles: [sample_tile_ref(0); 4],
        };
        let empty_frame = marked_frame(0, 0, 0, 8, 0);
        let prize_visual = wildside_render::AnimatedSpriteVisual {
            palette: marked_palette(0x0100),
            extra_palettes: Vec::new(),
            frames: vec![empty_frame.clone()],
            frame_duration: 1,
        };

        wildside_render::NativeLevelVisuals {
            foreground_tiles: marked_tile_sheet(1, 1),
            background_tiles: marked_tile_sheet(1, 2),
            island_shore_tiles_loc_1202a: None,
            block_tiles: marked_tile_sheet(128, 3),
            palettes: wildside_render::PaletteSet {
                foreground: marked_palette(0x0200),
                background: marked_palette(0x0300),
            },
            background: wildside_render::BackgroundVisuals::Chunked(
                wildside_render::ChunkedBackgroundVisuals {
                    placements: Vec::new(),
                    chunks: Vec::new(),
                },
            ),
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
                block_mapping: wildside_render::BlockVisualMapping::new(
                    vec![empty_quad.clone()],
                    vec![empty_quad; 81],
                )
                .expect("block mapping"),
            },
            hud: wildside_render::HudVisuals {
                digits: marked_tile_sheet(1, 4),
                digit_base_tile_index: 0,
                diamond_tiles: marked_tile_sheet(1, 5),
                diamond_base_tile_index: 0,
                hitpoint_sheet: marked_tile_sheet(1, 6),
                icons: Vec::new(),
                hitpoint_tiles: [sample_tile_ref(0), sample_tile_ref(1), sample_tile_ref(2)],
            },
            player: wildside_render::PlayerVisuals {
                forms: std::collections::HashMap::from([(0, labelled_player_visuals())]),
            },
            juggernaut_skull: None,
            enemies: wildside_render::EnemyVisuals {
                families: std::collections::HashMap::new(),
                layout_palette_variants: std::collections::HashMap::new(),
            },
            floating_prizes: wildside_render::FloatingPrizeVisuals {
                diamond: prize_visual,
                helmets: std::collections::HashMap::new(),
                icons: std::collections::HashMap::new(),
            },
            flag: wildside_render::FlagVisuals {
                palette: marked_palette(0x0400),
                pole_bottom: empty_frame.clone(),
                waving: Vec::new(),
                frame_duration: 1,
            },
            murder_wall: None,
            lava: None,
            weather: None,
            special_effect_unknown: None,
            platforms: wildside_render::PlatformVisuals {
                palette: marked_palette(0x0500),
                horizontal: marked_frame(0x0100, 4, 14, 20, 7),
                vertical: marked_frame(0x0200, 6, 18, 24, 8),
            },
            eol_score: None,
            eol_hologram: None,
            pause_menu: None,
            title_card_letters: None,
        }
    }

    fn labelled_enemy_visuals(
        frame_count: u16,
        frame_duration: usize,
    ) -> wildside_render::AnimatedSpriteVisual {
        wildside_render::AnimatedSpriteVisual {
            palette: wildside_render::Palette {
                colours: Vec::new(),
            },
            extra_palettes: Vec::new(),
            frames: (0..frame_count)
                .map(|index| labelled_frame(0x0100 + index))
                .collect(),
            frame_duration,
        }
    }

    fn native_object_at(x_pos: u32, y_pos: u32) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos,
            y_pos,
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
            debug_fields: std::collections::BTreeMap::new(),
        }
    }

    fn platform_object_at(
        x_px: u32,
        y_px: u32,
        render_flags: u8,
        mapping_addr_word: u16,
    ) -> NativeObjectSnapshot {
        let mut debug_fields = std::collections::BTreeMap::new();
        debug_fields.insert("platform_width_px".to_owned(), "47".to_owned());
        debug_fields.insert("platform_height_px".to_owned(), "31".to_owned());
        debug_fields.insert("platform_render_flags".to_owned(), render_flags.to_string());
        debug_fields.insert(
            "platform_mapping_addr_word".to_owned(),
            mapping_addr_word.to_string(),
        );
        NativeObjectSnapshot {
            kind: NativeObjectKind::Platform {
                kind: 0,
                subtype: 0,
            },
            x_pos: x_px << 16,
            y_pos: y_px << 16,
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
            debug_fields,
        }
    }

    fn sample_tile_ref(tile_index: u16) -> wildside_render::TileRef {
        wildside_render::TileRef {
            tile_index,
            palette_line: 0,
            priority: false,
            flip_x: false,
            flip_y: false,
        }
    }

    fn selected_tile_base(
        trace: &NativeTraceFrame,
        visuals: &wildside_render::PlayerFormVisuals,
    ) -> u16 {
        select_player_frame(trace, visuals).tile_base_index
    }

    fn sprite_tile_summary(tiles: &[SpriteTile]) -> Vec<(i32, i32, u16, bool)> {
        tiles
            .iter()
            .map(|tile| {
                (
                    tile.offset_x,
                    tile.offset_y,
                    tile.tile.tile_index,
                    tile.tile.flip_x,
                )
            })
            .collect()
    }

    fn only_sprite_command(
        commands: &[DrawCommand],
    ) -> (&i32, &i32, &Arc<Palette>, &Vec<SpriteTile>) {
        let [
            DrawCommand::Sprite {
                x,
                y,
                palette,
                tiles,
                ..
            }
            | DrawCommand::SatSprite {
                x,
                y,
                palette,
                tiles,
                ..
            },
        ] = commands
        else {
            panic!("expected exactly one sprite command");
        };
        (x, y, palette, tiles)
    }

    fn live_enemy_object(object_type: u8) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type },
            x_pos: 0,
            y_pos: 0,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(1),
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
            debug_fields: std::collections::BTreeMap::new(),
        }
    }

    fn dead_enemy_object(object_type: u8, wait_timer: u16) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            hit_points: Some(0),
            wait_timer: Some(wait_timer),
            ..live_enemy_object(object_type)
        }
    }

    fn selected_enemy_tile_base(
        trace: &NativeTraceFrame,
        visuals: &wildside_render::AnimatedSpriteVisual,
        object: &NativeObjectSnapshot,
    ) -> u16 {
        select_enemy_frame(trace, visuals, object).tile_base_index
    }

    fn selected_live_tile_base(frame: Option<&wildside_render::SpriteFrame>) -> u16 {
        frame
            .expect("live selector should return a frame")
            .tile_base_index
    }

    #[test]
    fn boss_head_death_helpers_follow_loc_39ebe_script_boundaries() {
        // ASM loc_39EBE/loc_39F7A: intro scripts consume 120 frames, then a
        // 17-frame spin script repeats while toggling facing for frames 5..12.
        let total = BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE;
        assert_eq!(total, 290);
        assert_eq!(boss_head_death_frame_loc_39ebe(total), 4);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 5), 4);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 6), 3);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 29), 0);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 30), 1);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 119), 4);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 120), 4);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 124), 0);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 125), 1);
        assert_eq!(boss_head_death_frame_loc_39ebe(total - 137), 4);

        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Right, total - 29),
            Direction::Right
        );
        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Right, total - 30),
            Direction::Left
        );
        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Right, total - 70),
            Direction::Right
        );
        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Right, total - 96),
            Direction::Left
        );
        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Right, total - 112),
            Direction::Right
        );
        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Right, total - 124),
            Direction::Right
        );
        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Right, total - 125),
            Direction::Left
        );
        assert_eq!(
            boss_head_death_facing_loc_39ebe(Direction::Left, total - 125),
            Direction::Right
        );
    }

    #[test]
    fn sprite_draw_position_uses_aligned_width_for_left_facing() {
        let frame = wildside_render::SpriteFrame {
            origin_x: 7,
            origin_y: 3,
            width_px: 17,
            height_px: 18,
            tile_base_index: 0,
            tile_sheet: wildside_render::TileSheet { tiles: Vec::new() },
        };

        assert_eq!(align_sprite_dimension(0), 0);
        assert_eq!(align_sprite_dimension(1), 8);
        assert_eq!(align_sprite_dimension(8), 8);
        assert_eq!(align_sprite_dimension(9), 16);
        assert_eq!(align_sprite_dimension(33), 40);
        assert_eq!(
            sprite_draw_position(&frame, 100, 50, Direction::Right),
            (93, 35)
        );
        assert_eq!(
            sprite_draw_position(&frame, 100, 50, Direction::Left),
            (83, 35)
        );
    }

    #[test]
    fn player_dynamic_art_queue_follows_loc_141a_hardware_sprite_chunks() {
        let frame = |width_px, height_px| wildside_render::SpriteFrame {
            width_px,
            height_px,
            ..synthetic_player_frame()
        };

        assert_eq!(
            player_frame_dynamic_art_tiles_loc_141a(&frame(24, 32)),
            [12, 0, 0, 0, 0, 0],
            "ASM sub_1492 -> sub_13C0 loc_141A emits one 3x4-tile Kid chunk",
        );
        assert_eq!(
            player_frame_dynamic_art_tiles_loc_141a(&frame(40, 32)),
            [16, 4, 0, 0, 0, 0],
            "ASM loc_141A emits the 32x32 chunk before the 8x32 tail",
        );
        assert_eq!(
            player_frame_dynamic_art_tiles_loc_141a(&frame(40, 40)),
            [16, 4, 4, 1, 0, 0],
            "ASM loc_141A preserves band-major order for four chunks",
        );
    }

    #[test]
    fn build_sprite_tiles_splits_bands_chunks_and_flips_x_offsets() {
        let frame = wildside_render::SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 33,
            height_px: 40,
            tile_base_index: 0x20,
            tile_sheet: wildside_render::TileSheet { tiles: Vec::new() },
        };

        assert_eq!(
            sprite_tile_summary(&build_sprite_tiles_with_flip(&frame, false, 2)),
            vec![
                (0, 0, 0x20, false),
                (0, 8, 0x21, false),
                (0, 16, 0x22, false),
                (0, 24, 0x23, false),
                (8, 0, 0x24, false),
                (8, 8, 0x25, false),
                (8, 16, 0x26, false),
                (8, 24, 0x27, false),
                (16, 0, 0x28, false),
                (16, 8, 0x29, false),
                (16, 16, 0x2A, false),
                (16, 24, 0x2B, false),
                (24, 0, 0x2C, false),
                (24, 8, 0x2D, false),
                (24, 16, 0x2E, false),
                (24, 24, 0x2F, false),
                (32, 0, 0x30, false),
                (32, 8, 0x31, false),
                (32, 16, 0x32, false),
                (32, 24, 0x33, false),
                (0, 32, 0x34, false),
                (8, 32, 0x35, false),
                (16, 32, 0x36, false),
                (24, 32, 0x37, false),
                (32, 32, 0x38, false),
            ]
        );

        assert_eq!(
            sprite_tile_summary(&build_sprite_tiles_with_flip(&frame, true, 3)),
            vec![
                (32, 0, 0x20, true),
                (32, 8, 0x21, true),
                (32, 16, 0x22, true),
                (32, 24, 0x23, true),
                (24, 0, 0x24, true),
                (24, 8, 0x25, true),
                (24, 16, 0x26, true),
                (24, 24, 0x27, true),
                (16, 0, 0x28, true),
                (16, 8, 0x29, true),
                (16, 16, 0x2A, true),
                (16, 24, 0x2B, true),
                (8, 0, 0x2C, true),
                (8, 8, 0x2D, true),
                (8, 16, 0x2E, true),
                (8, 24, 0x2F, true),
                (0, 0, 0x30, true),
                (0, 8, 0x31, true),
                (0, 16, 0x32, true),
                (0, 24, 0x33, true),
                (32, 32, 0x34, true),
                (24, 32, 0x35, true),
                (16, 32, 0x36, true),
                (8, 32, 0x37, true),
                (0, 32, 0x38, true),
            ]
        );

        let player_tiles = build_player_sprite_tiles_with_flip(&frame, true, Some(true));
        assert_eq!(player_tiles.len(), 25);
        assert!(
            player_tiles
                .iter()
                .all(|tile| tile.tile.priority && tile.tile.palette_line == 0)
        );

        let full_bands_frame = wildside_render::SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 96,
            height_px: 96,
            tile_base_index: 0x80,
            tile_sheet: wildside_render::TileSheet { tiles: Vec::new() },
        };
        let full_bands = build_sprite_tiles_with_flip(&full_bands_frame, false, 1);
        assert_eq!(
            full_bands.len(),
            144,
            "96x96 sprites need three 32px mapping bands and three 32px chunks per band"
        );
        assert_eq!(
            full_bands.first().map(|tile| tile.tile.tile_index),
            Some(0x80)
        );
        assert_eq!(
            full_bands.get(47).map(|tile| {
                (
                    tile.offset_x,
                    tile.offset_y,
                    tile.tile.tile_index,
                    tile.tile.palette_line,
                )
            }),
            Some((88, 24, 0xAF, 1))
        );
        assert_eq!(
            full_bands.get(48).map(|tile| {
                (
                    tile.offset_x,
                    tile.offset_y,
                    tile.tile.tile_index,
                    tile.tile.palette_line,
                )
            }),
            Some((0, 32, 0xB0, 1))
        );
        assert_eq!(
            full_bands.last().map(|tile| {
                (
                    tile.offset_x,
                    tile.offset_y,
                    tile.tile.tile_index,
                    tile.tile.palette_line,
                )
            }),
            Some((88, 88, 0x10F, 1))
        );
    }

    #[test]
    fn platform_art_mapping_words_select_asm_sources_loc_2548() {
        let visuals = minimal_level_visuals_for_platforms();

        for (mapping_word, expected_base, expected_marker, expected_palette, asm_label) in [
            (
                0x0094,
                0x0100,
                7,
                0x0500,
                "ASM PlatformArtAddr_Table loc_2822 horizontal",
            ),
            (
                0x06AE,
                0x0200,
                8,
                0x0500,
                "ASM PlatformArtAddr_Table loc_2822 vertical",
            ),
            (
                0x025A,
                0x003A,
                3,
                0x0200,
                "ASM loc_2548 block-art range `unk_E0E4E`",
            ),
        ] {
            let object = platform_object_at(0x0120, 0x0090, 8, mapping_word);
            let art = platform_art_for_mapping_word_loc_2548(&object, &visuals)
                .expect("mapping word should resolve to platform art");

            assert_eq!(
                art.frame.tile_base_index, expected_base,
                "{asm_label} tile base"
            );
            assert_eq!(
                art.frame.tile_sheet.tiles[0].pixels[0], expected_marker,
                "{asm_label} tile sheet"
            );
            assert_eq!(
                art.palette.colours[0].raw_word, expected_palette,
                "{asm_label} palette"
            );
        }

        let object = platform_object_at(0x0120, 0x0090, 8, 0x1234);
        assert!(
            platform_art_for_mapping_word_loc_2548(&object, &visuals).is_none(),
            "unknown loc_2822 mapping words fall back to the render variant/kind heuristic"
        );
    }

    #[test]
    fn platform_sprite_cells_translate_asm_entries_loc_24ac_2548() {
        let dispatch = PlatformSpriteDispatchLoc244C {
            screen_x_word: 0x00A0,
            screen_y_word: 0x0090,
            width_tiles_minus_one: 2,
            height_tiles_minus_one: 1,
            base_tile_word: 0x0500,
            mapping_addr_word: 0x2468,
            variant: PlatformSpriteVariantLoc244C::Loc24D4,
        };

        assert_eq!(
            platform_sprite_cells_loc_24AC(PlatformSpriteVariantLoc244C::Loc24D4, dispatch, 3, 2),
            vec![
                platform_cell(0, 0, 0, 0),
                platform_cell(1, 0, 0, 0),
                platform_cell(2, 0, 0, 0),
                platform_cell(0, 1, 0, 0),
                platform_cell(1, 1, 0, 0),
                platform_cell(2, 1, 0, 0),
            ],
            "ASM loc_24D4 emits a full width-by-height cell grid"
        );

        let translated = platform_sprite_entry_cell_loc_2548(
            dispatch,
            PlatformSpriteEntryLoc2548 {
                y_word: 0x00B5,
                tile_word: 0x0512,
                mapping_word: 0xA47A,
                x_word: 0x00C0,
            },
        );
        assert_eq!(
            translated,
            platform_cell(2, 2, 5, 0x0012),
            "ASM loc_2548 stores biased x/y/mapping words; frame code must recover cell coordinates and the intra-art tile offset"
        );

        let frame = marked_frame(0x0300, 0, 0, 16, 9);
        assert_eq!(
            platform_cell_tiles_loc_2548(&frame, 0x0012)
                .iter()
                .map(|tile| (tile.offset_x, tile.offset_y, tile.tile.tile_index))
                .collect::<Vec<_>>(),
            vec![
                (0, 0, 0x0312),
                (0, 8, 0x0313),
                (8, 0, 0x0314),
                (8, 8, 0x0315)
            ],
            "ASM loc_2548 maps one 16x16 platform sprite to four consecutive 8x8 tiles"
        );
    }

    #[test]
    fn push_platform_tiles_emits_loc_244c_commands_and_art_sources() {
        let visuals = minimal_level_visuals_for_platforms();
        let trace = NativeTraceFrame {
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0x0080_0000,
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        // ASM loc_244C -> loc_24D4 -> loc_2548: horizontal `$94` art, six
        // 16px grid cells, and frame-origin-adjusted screen coordinates.
        push_platform_tiles(
            &mut commands,
            &platform_object_at(0x0120, 0x0090, 6, 0x0094),
            &trace,
            &visuals,
            0x00,
        );
        assert_eq!(commands.len(), 6);
        let summary = commands
            .iter()
            .map(|command| match command {
                DrawCommand::Sprite {
                    x,
                    y,
                    tile_sheet,
                    palette,
                    tiles,
                } => (
                    *x,
                    *y,
                    tile_sheet.tiles[0].pixels[0],
                    palette.colours[0].raw_word,
                    tiles
                        .iter()
                        .map(|tile| tile.tile.tile_index)
                        .collect::<Vec<_>>(),
                ),
                _ => panic!("platforms should emit sprite commands"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            summary,
            vec![
                (28, 10, 7, 0x0500, vec![0x0100, 0x0101, 0x0102, 0x0103]),
                (44, 10, 7, 0x0500, vec![0x0100, 0x0101, 0x0102, 0x0103]),
                (60, 10, 7, 0x0500, vec![0x0100, 0x0101, 0x0102, 0x0103]),
                (28, 26, 7, 0x0500, vec![0x0100, 0x0101, 0x0102, 0x0103]),
                (44, 26, 7, 0x0500, vec![0x0100, 0x0101, 0x0102, 0x0103]),
                (60, 26, 7, 0x0500, vec![0x0100, 0x0101, 0x0102, 0x0103]),
            ],
        );

        // ASM loc_2822 `$25A`: shifting-block/common block art uses block
        // tiles and the foreground palette instead of the platform sheet.
        commands.clear();
        push_platform_tiles(
            &mut commands,
            &platform_object_at(0x0120, 0x0090, 8, 0x025A),
            &trace,
            &visuals,
            0x00,
        );
        let [
            DrawCommand::Sprite {
                x,
                y,
                tile_sheet,
                palette,
                tiles,
            },
        ] = commands.as_slice()
        else {
            panic!("loc_24C0 platform should emit one sprite command");
        };
        assert_eq!((*x, *y), (32, 16));
        assert_eq!(tile_sheet.tiles[0].pixels[0], 3);
        assert_eq!(palette.colours[0].raw_word, 0x0200);
        assert_eq!(
            tiles
                .iter()
                .map(|tile| tile.tile.tile_index)
                .collect::<Vec<_>>(),
            vec![0x003A, 0x003B, 0x003C, 0x003D]
        );

        // Unknown mapping words fall back to render-variant/kind selection.
        // Loc24CC includes ASM's `subq.w #2,d6` y-bias, which must survive the
        // frame-origin conversion.
        commands.clear();
        push_platform_tiles(
            &mut commands,
            &platform_object_at(0x0120, 0x0090, 0, 0x1234),
            &trace,
            &visuals,
            0x00,
        );
        let DrawCommand::Sprite {
            x, y, tile_sheet, ..
        } = &commands[0]
        else {
            panic!("loc_24CC fallback platform should emit sprite commands");
        };
        assert_eq!((*x, *y), (28, 8));
        assert_eq!(tile_sheet.tiles[0].pixels[0], 7);

        commands.clear();
        push_platform_tiles(
            &mut commands,
            &platform_object_at(0x0120, 0x0090, 4, 0x1234),
            &trace,
            &visuals,
            0x00,
        );
        let DrawCommand::Sprite { tile_sheet, .. } = &commands[0] else {
            panic!("loc_251C fallback platform should emit sprite commands");
        };
        assert_eq!(
            tile_sheet.tiles[0].pixels[0], 8,
            "ASM off_24AC loc_251C renders with vertical platform art even when kind is horizontal"
        );

        // Non-zero camera high words are part of loc_244C visibility culling,
        // not just final draw-command placement.
        commands.clear();
        let far_trace = NativeTraceFrame {
            camera_x_pos: 0x0200_0000,
            camera_y_pos: 0x0300_0000,
            ..NativeTraceFrame::default()
        };
        push_platform_tiles(
            &mut commands,
            &platform_object_at(0x0210, 0x0310, 8, 0x0094),
            &far_trace,
            &visuals,
            0x00,
        );
        assert_eq!(
            commands.len(),
            1,
            "ASM loc_244C should keep a platform visible when both camera-relative high words are near the viewport origin"
        );

        commands.clear();
        let mut missing_mapping = platform_object_at(0x0120, 0x0090, 8, 0x025A);
        missing_mapping
            .debug_fields
            .remove("platform_mapping_addr_word");
        push_platform_tiles(&mut commands, &missing_mapping, &trace, &visuals, 0x00);
        assert!(
            commands.is_empty(),
            "ASM loc_244C cannot dispatch without the Platform_RAM mapping word"
        );
    }

    #[test]
    fn live_enemy_selectors_reject_wrong_family_dead_and_short_sheets() {
        let trace = NativeTraceFrame::default();
        let full = labelled_enemy_visuals(23, 3);
        let short = labelled_enemy_visuals(2, 3);

        // ASM Dragon.asm loc_3CE48/loc_3CEFC: live walking-dragon selector is
        // family-specific; dead snapshots route to the death script instead.
        let dragon = live_enemy_object(0x0D);
        assert!(select_live_walking_dragon_frame(&trace, &full, &dragon).is_some());
        let mut wrong_family = dragon.clone();
        wrong_family.kind = NativeObjectKind::Enemy { object_type: 0x0E };
        assert!(select_live_walking_dragon_frame(&trace, &full, &wrong_family).is_none());
        let mut dead = dragon.clone();
        dead.hit_points = Some(0);
        assert!(select_live_walking_dragon_frame(&trace, &full, &dead).is_none());
        assert!(select_live_walking_dragon_frame(&trace, &short, &dragon).is_none());

        // ASM Hand.asm loc_327F2/loc_328E0/loc_32906: live Hand requires its
        // first seven body frames; the death tail must be unreachable here.
        let hand = live_enemy_object(0x17);
        assert!(select_live_hand_frame(&full, &hand).is_some());
        assert!(select_live_hand_frame(&short, &hand).is_none());
        let mut wrong_family = hand.clone();
        wrong_family.kind = NativeObjectKind::Enemy { object_type: 0x16 };
        assert!(select_live_hand_frame(&full, &wrong_family).is_none());
        let mut dead = hand;
        dead.hit_points = Some(0);
        assert!(select_live_hand_frame(&full, &dead).is_none());

        for (object_type, accepts) in [
            (
                0x0A,
                select_live_rock_tank_body_frame(&trace, &full, &live_enemy_object(0x0A)).is_some(),
            ),
            (
                0x01,
                select_live_diamond_frame(&trace, &full, &live_enemy_object(0x01)).is_some(),
            ),
            (
                0x0C,
                select_live_flying_dragon_frame(&trace, &full, &live_enemy_object(0x0C)).is_some(),
            ),
            (0x04, {
                let mut armadillo = live_enemy_object(0x04);
                armadillo.sprite_animation_phase = Some(0);
                select_live_armadillo_frame(&full, &armadillo).is_some()
            }),
            (0x03, {
                let mut robot_head = live_enemy_object(0x03);
                robot_head.hit_points = Some(0);
                robot_head.sprite_animation_phase = Some(10);
                select_live_robot_frame(&full, &robot_head).is_some()
            }),
            (0x0E, {
                let mut cloud = live_enemy_object(0x0E);
                cloud.sprite_animation_phase = Some(9);
                select_live_cloud_attack_frame(&full, &cloud).is_some()
            }),
            (0x14, {
                let mut twin = live_enemy_object(0x14);
                twin.sprite_animation_phase = Some(4);
                select_live_spinning_twins_frame(&full, &twin).is_some()
            }),
            (
                0x06,
                select_live_sphere_frame(&trace, &full, &live_enemy_object(0x06)).is_some(),
            ),
            (0x1B, {
                let mut emo_rock = live_enemy_object(0x1B);
                emo_rock.sprite_animation_phase = Some(10);
                select_live_emo_rock_frame(&trace, &full, &emo_rock).is_some()
            }),
            (0x1A, {
                let mut driller = live_enemy_object(0x1A);
                driller.sprite_animation_phase = Some(0);
                select_live_driller_frame(&full, &driller).is_some()
            }),
            (
                0x12,
                select_live_lion_frame(&trace, &full, &live_enemy_object(0x12)).is_some(),
            ),
        ] {
            assert!(
                accepts,
                "live selector should accept ASM enemy family {object_type:#04X}"
            );
        }

        let mut rejected = live_enemy_object(0x0A);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x0C };
        assert!(select_live_rock_tank_body_frame(&trace, &full, &rejected).is_none());
        rejected = live_enemy_object(0x0A);
        rejected.hit_points = Some(0);
        assert!(select_live_rock_tank_body_frame(&trace, &full, &rejected).is_none());
        assert!(
            select_live_rock_tank_body_frame(&trace, &short, &live_enemy_object(0x0A)).is_none()
        );

        let mut rejected = live_enemy_object(0x01);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x0D };
        assert!(select_live_diamond_frame(&trace, &full, &rejected).is_none());
        rejected = live_enemy_object(0x01);
        rejected.hit_points = Some(0);
        assert!(select_live_diamond_frame(&trace, &full, &rejected).is_none());
        assert!(select_live_diamond_frame(&trace, &short, &live_enemy_object(0x01)).is_none());

        let mut rejected = live_enemy_object(0x0C);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x0D };
        assert!(select_live_flying_dragon_frame(&trace, &full, &rejected).is_none());
        rejected = live_enemy_object(0x0C);
        rejected.hit_points = Some(0);
        assert!(select_live_flying_dragon_frame(&trace, &full, &rejected).is_none());
        assert!(
            select_live_flying_dragon_frame(&trace, &short, &live_enemy_object(0x0C)).is_none()
        );

        let mut rejected = live_enemy_object(0x05);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x06 };
        assert!(select_live_tar_monster_frame(&trace, &full, &rejected).is_none());
        rejected = live_enemy_object(0x05);
        rejected.hit_points = Some(0);
        assert!(select_live_tar_monster_frame(&trace, &full, &rejected).is_none());
        let too_short_tar = labelled_enemy_visuals(1, 3);
        assert!(
            select_live_tar_monster_frame(&trace, &too_short_tar, &live_enemy_object(0x05))
                .is_none()
        );

        let mut rejected = live_enemy_object(0x04);
        rejected.sprite_animation_phase = Some(0);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x03 };
        assert!(select_live_armadillo_frame(&full, &rejected).is_none());
        rejected = live_enemy_object(0x04);
        rejected.sprite_animation_phase = Some(0);
        rejected.hit_points = Some(0);
        assert!(select_live_armadillo_frame(&full, &rejected).is_none());
        let mut live_armadillo = live_enemy_object(0x04);
        live_armadillo.sprite_animation_phase = Some(0);
        assert!(select_live_armadillo_frame(&short, &live_armadillo).is_none());

        let mut rejected = live_enemy_object(0x06);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x05 };
        assert!(select_live_sphere_frame(&trace, &full, &rejected).is_none());
        rejected = live_enemy_object(0x06);
        rejected.hit_points = Some(0);
        assert!(select_live_sphere_frame(&trace, &full, &rejected).is_none());
        assert!(select_live_sphere_frame(&trace, &short, &live_enemy_object(0x06)).is_none());

        let mut rejected = live_enemy_object(0x1B);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x1A };
        assert!(select_live_emo_rock_frame(&trace, &full, &rejected).is_none());
        rejected = live_enemy_object(0x1B);
        rejected.hit_points = Some(0);
        assert!(select_live_emo_rock_frame(&trace, &full, &rejected).is_none());
        assert!(select_live_emo_rock_frame(&trace, &short, &live_enemy_object(0x1B)).is_none());

        let mut rejected = live_enemy_object(0x1A);
        rejected.sprite_animation_phase = Some(0);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x19 };
        assert!(select_live_driller_frame(&full, &rejected).is_none());
        rejected = live_enemy_object(0x1A);
        rejected.sprite_animation_phase = Some(0);
        rejected.hit_points = Some(0);
        assert!(select_live_driller_frame(&full, &rejected).is_none());
        let mut live_driller = live_enemy_object(0x1A);
        live_driller.sprite_animation_phase = Some(0);
        assert!(select_live_driller_frame(&short, &live_driller).is_none());

        let mut rejected = live_enemy_object(0x12);
        rejected.kind = NativeObjectKind::Enemy { object_type: 0x13 };
        assert!(select_live_lion_frame(&trace, &full, &rejected).is_none());
        rejected = live_enemy_object(0x12);
        rejected.hit_points = Some(0);
        assert!(select_live_lion_frame(&trace, &full, &rejected).is_none());
        assert!(select_live_lion_frame(&trace, &short, &live_enemy_object(0x12)).is_none());
    }

    #[test]
    fn live_enemy_selectors_accept_exact_asm_minimum_sheet_lengths() {
        let trace = NativeTraceFrame::default();

        // ASM Dragon.asm loc_3CE48 live walk uses frames 0..7; loc_3CF90 breath
        // may only read frame 10 when the extracted sheet actually has it.
        let dragon_min = labelled_enemy_visuals(8, 3);
        let dragon = live_enemy_object(0x0D);
        assert_eq!(
            selected_live_tile_base(select_live_walking_dragon_frame(
                &trace,
                &dragon_min,
                &dragon
            )),
            0x0100
        );
        let mut breathing_dragon = dragon;
        breathing_dragon.walk_phase = Some(1);
        let dragon_without_breath = labelled_enemy_visuals(10, 3);
        assert_ne!(
            selected_live_tile_base(select_live_walking_dragon_frame(
                &trace,
                &dragon_without_breath,
                &breathing_dragon
            )),
            0x010A,
            "ASM Dragon.asm loc_3CF90 breath frame 10 is unavailable on a ten-frame synthetic sheet"
        );
        let dragon_with_breath = labelled_enemy_visuals(11, 3);
        assert_eq!(
            selected_live_tile_base(select_live_walking_dragon_frame(
                &trace,
                &dragon_with_breath,
                &breathing_dragon
            )),
            0x010A
        );

        // These selectors deliberately accept the exact live-frame window and
        // reject only shorter sheets; appended frames are hurt/death tails.
        assert_eq!(
            selected_live_tile_base(select_live_diamond_frame(
                &trace,
                &labelled_enemy_visuals(5, 3),
                &live_enemy_object(0x01)
            )),
            0x0100
        );
        assert_eq!(
            selected_live_tile_base(select_live_flying_dragon_frame(
                &trace,
                &labelled_enemy_visuals(5, 3),
                &live_enemy_object(0x0C)
            )),
            0x0100
        );
        assert_eq!(
            selected_live_tile_base(select_live_tar_monster_frame(
                &trace,
                &labelled_enemy_visuals(2, 3),
                &live_enemy_object(0x05)
            )),
            0x0101
        );
        let mut armadillo = live_enemy_object(0x04);
        armadillo.sprite_animation_phase = Some(0);
        assert_eq!(
            selected_live_tile_base(select_live_armadillo_frame(
                &labelled_enemy_visuals(15, 3),
                &armadillo
            )),
            0x0100
        );
        assert_eq!(
            selected_live_tile_base(select_live_sphere_frame(
                &trace,
                &labelled_enemy_visuals(3, 3),
                &live_enemy_object(0x06)
            )),
            0x0100
        );
        let mut emo_rock = live_enemy_object(0x1B);
        emo_rock.sprite_animation_phase = Some(0);
        assert_eq!(
            selected_live_tile_base(select_live_emo_rock_frame(
                &trace,
                &labelled_enemy_visuals(13, 3),
                &emo_rock
            )),
            0x0100
        );
        let mut driller = live_enemy_object(0x1A);
        driller.sprite_animation_phase = Some(0);
        assert_eq!(
            selected_live_tile_base(select_live_driller_frame(
                &labelled_enemy_visuals(5, 3),
                &driller
            )),
            0x0100
        );
        assert_eq!(
            selected_live_tile_base(select_live_hand_frame(
                &labelled_enemy_visuals(7, 3),
                &live_enemy_object(0x17)
            )),
            0x0100
        );
        assert_eq!(
            selected_live_tile_base(select_live_lion_frame(
                &trace,
                &labelled_enemy_visuals(6, 3),
                &live_enemy_object(0x12)
            )),
            0x0100
        );
    }

    #[test]
    fn live_enemy_selectors_follow_asm_body_sequences() {
        let visuals = labelled_enemy_visuals(12, 3);
        let mut trace = NativeTraceFrame {
            frame_index: 90,
            ..NativeTraceFrame::default()
        };
        let mut dragon = live_enemy_object(0x0D);

        // ASM Dragon.asm loc_3CE48/loc_3CDFC/loc_3CEBE/loc_3CF90: normal,
        // HP-3, hurt, and breath phases each pick a specific live frame.
        assert_eq!(selected_enemy_tile_base(&trace, &visuals, &dragon), 0x0107);
        dragon.sprite_animation_phase = Some(9);
        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &dragon),
            0x0103,
            "ASM GfxObject_Animate loc_113E must use the Dragon's object-local stru_3CC5E phase instead of global trace time"
        );
        dragon.sprite_animation_phase = None;
        dragon.hit_points = Some(3);
        trace.frame_index = 60;
        assert_eq!(selected_enemy_tile_base(&trace, &visuals, &dragon), 0x0107);
        dragon.hit_points = Some(1);
        dragon.damage_flash = true;
        assert_eq!(selected_enemy_tile_base(&trace, &visuals, &dragon), 0x0106);
        dragon.damage_flash = false;
        dragon.walk_phase = Some(1);
        assert_eq!(selected_enemy_tile_base(&trace, &visuals, &dragon), 0x010A);

        // ASM Diamond.asm stru_3ACA8 and Sphere.asm stru_36174 cycle only the
        // live frame windows, not the appended hurt/death descriptors.
        let diamond_visuals = labelled_enemy_visuals(10, 99);
        trace.frame_index = 13;
        assert_eq!(
            selected_live_tile_base(select_live_diamond_frame(
                &trace,
                &diamond_visuals,
                &live_enemy_object(0x01)
            )),
            0x0102
        );
        let sphere_visuals = labelled_enemy_visuals(6, 9);
        trace.frame_index = 19;
        assert_eq!(
            selected_live_tile_base(select_live_sphere_frame(
                &trace,
                &sphere_visuals,
                &live_enemy_object(0x06)
            )),
            0x0102
        );

        // ASM Tank.asm stru_3B270 and Emo_Rock.asm loc_3621A/loc_362A4
        // similarly stay in the body triplet while alive.
        let rock_tank_visuals = labelled_enemy_visuals(6, 4);
        trace.frame_index = 5;
        assert_eq!(
            selected_live_tile_base(select_live_rock_tank_body_frame(
                &trace,
                &rock_tank_visuals,
                &live_enemy_object(0x0A)
            )),
            0x0101
        );
        let emo_visuals = labelled_enemy_visuals(13, 20);
        let mut emo_rock = live_enemy_object(0x1B);
        emo_rock.sprite_animation_phase = Some(10);
        assert_eq!(
            selected_live_tile_base(select_live_emo_rock_frame(&trace, &emo_visuals, &emo_rock)),
            0x010A,
            "ASM Emo_Rock.asm stru_3647E final C7F38 mapping must hold during loc_3625E drop"
        );

        // ASM Armadillo.asm stru_3C632/stru_3C64C use the first six walk
        // mappings; loc_3E4CE switches to C76A6..C76CE rolling mappings and
        // loc_3C848 temporarily holds C76D6.
        let armadillo_visuals = labelled_enemy_visuals(15, 9);
        let mut armadillo = live_enemy_object(0x04);
        armadillo.sprite_animation_phase = Some(0);
        trace.frame_index = 72;
        assert_eq!(
            selected_enemy_tile_base(&trace, &armadillo_visuals, &armadillo),
            0x0100,
            "live Armadillo walk must not free-run into charge or death mappings"
        );
        armadillo.sprite_animation_phase = Some(6);
        assert_eq!(
            selected_enemy_tile_base(&trace, &armadillo_visuals, &armadillo),
            0x0106,
            "ASM Lion.asm loc_3E4CE must select the Armadillo rolling bank"
        );
        armadillo.damage_flash = true;
        armadillo.sprite_animation_phase = Some(12);
        assert_eq!(
            selected_enemy_tile_base(&trace, &armadillo_visuals, &armadillo),
            0x010C,
            "ASM Armadillo.asm loc_3C848/stru_3C6D0 must hold C76D6"
        );

        // ASM Robot.asm loc_3A7EE/stru_3A716: the linked head owns a
        // separate six-frame C82D8..C8300 walk script. It must not use the
        // parent body's C82A8..C82D0 atlas phase.
        let robot_visuals = labelled_enemy_visuals(23, 9);
        let mut robot_head = live_enemy_object(0x03);
        robot_head.hit_points = Some(0);
        robot_head.sprite_animation_phase = Some(10);
        trace.frame_index = 0;
        assert_eq!(
            selected_enemy_tile_base(&trace, &robot_visuals, &robot_head),
            0x010A,
            "linked Robot head must select its typed C82D8 mapping instead of Robot legs"
        );

        // ASM Cloud.asm loc_35D9E/stru_35EC8: an attacking Cloud replaces
        // its six-frame flight loop with C787E/C7886/C788E/C7896 while the
        // linked lightning worker is prepared.
        let cloud_visuals = labelled_enemy_visuals(13, 11);
        let mut cloud = live_enemy_object(0x0E);
        cloud.sprite_animation_phase = Some(9);
        trace.frame_index = 0;
        assert_eq!(
            selected_enemy_tile_base(&trace, &cloud_visuals, &cloud),
            0x0109,
            "Cloud attack wind-up must select typed C787E instead of continuing the normal flight loop"
        );

        // ASM Spinning_Twins.asm loc_3BA74/loc_3B9B0: split walking
        // children use fixed C7CDE, while the falling visual worker uses
        // fixed C7D26 instead of the four-frame orbit parent bank.
        let spinning_twins_visuals = labelled_enemy_visuals(14, 6);
        let mut spinning_twin = live_enemy_object(0x14);
        spinning_twin.sprite_animation_phase = Some(4);
        assert_eq!(
            selected_enemy_tile_base(&trace, &spinning_twins_visuals, &spinning_twin),
            0x0104,
            "walking Spinning Twin must select fixed C7CDE instead of the orbit parent"
        );
        spinning_twin.hit_points = Some(0);
        spinning_twin.sprite_animation_phase = Some(7);
        assert_eq!(
            selected_enemy_tile_base(&trace, &spinning_twins_visuals, &spinning_twin),
            0x0107,
            "Spinning Twins falling visual must select fixed C7D26 instead of the orbit parent"
        );
        spinning_twin.hit_points = Some(1);
        spinning_twin.sprite_animation_phase = Some(8);
        assert_eq!(
            selected_enemy_tile_base(&trace, &spinning_twins_visuals, &spinning_twin),
            0x0108,
            "settled Spinning Twin must select typed stru_3B81C instead of the orbit parent"
        );

        // ASM Driller.asm stru_3BCB2 alternates only C80E0/C80E8 while
        // alive; loc_3BF4E temporarily replaces it with C80D8.
        let driller_visuals = labelled_enemy_visuals(5, 9);
        trace.frame_index = 18;
        let mut driller = live_enemy_object(0x1A);
        driller.sprite_animation_phase = Some(0);
        assert_eq!(
            selected_enemy_tile_base(&trace, &driller_visuals, &driller),
            0x0100,
            "ASM Driller.asm stru_3BCB2 must not free-run into the stru_3BCD4 death bank"
        );
        driller.damage_flash = true;
        driller.sprite_animation_phase = Some(2);
        assert_eq!(
            selected_enemy_tile_base(&trace, &driller_visuals, &driller),
            0x0102,
            "ASM Driller.asm loc_3BF4E/stru_3BCCE holds C80D8 while hurt"
        );

        // ASM Dragon_Flying.asm loc_346DA/loc_348CE: the normal wing sequence
        // and dive velocity table are both distinct from the generic loop.
        let flying_visuals = labelled_enemy_visuals(9, 7);
        trace.frame_index = 84;
        let mut flying_dragon = live_enemy_object(0x0C);
        assert_eq!(
            selected_live_tile_base(select_live_flying_dragon_frame(
                &trace,
                &flying_visuals,
                &flying_dragon
            )),
            0x0104
        );
        flying_dragon.x_velocity = Some(-0x0002_0000);
        flying_dragon.y_velocity = Some(-0x0003_0000);
        flying_dragon.facing = Some(Direction::Left);
        assert_eq!(
            selected_live_tile_base(select_live_flying_dragon_frame(
                &trace,
                &flying_visuals,
                &flying_dragon
            )),
            0x0107
        );
    }

    #[test]
    fn flying_dragon_dive_selector_covers_loc_348ce_velocity_bands() {
        let visuals = labelled_enemy_visuals(9, 7);
        let mut dragon = live_enemy_object(0x0C);

        dragon.x_velocity = Some(0x0001_0000);
        dragon.y_velocity = Some(0x0001_0000);
        dragon.facing = Some(Direction::Right);
        assert!(
            select_flying_dragon_dive_frame(&labelled_enemy_visuals(8, 7), &dragon).is_none(),
            "ASM Dragon_Flying.asm loc_348CE indexes C7636 at frame 8, so dive sheets need nine frames"
        );

        // ASM Dragon_Flying.asm loc_348CE splits `unk_34554` velocity bytes and
        // indexes `off_345F6`; native maps those visible bands back to the
        // extracted C75FE/C7616/C761E/C762E/C7636 frames.
        for (x_px, y_px, facing, expected_tile) in [
            (1, 1, Direction::Right, 0x0105),  // LnkTo_unk_C7616
            (1, 4, Direction::Left, 0x0106),   // LnkTo_unk_C761E
            (-2, 0, Direction::Left, 0x0102),  // LnkTo_unk_C75FE
            (2, -3, Direction::Left, 0x0107),  // LnkTo_unk_C762E
            (1, -4, Direction::Right, 0x0108), // LnkTo_unk_C7636
        ] {
            dragon.x_velocity = Some(x_px * 0x0001_0000);
            dragon.y_velocity = Some(y_px * 0x0001_0000);
            dragon.facing = Some(facing);
            assert_eq!(
                selected_live_tile_base(select_flying_dragon_dive_frame(&visuals, &dragon)),
                expected_tile,
                "ASM Dragon_Flying.asm loc_348CE should preserve the dive script velocity band"
            );
        }
    }

    #[test]
    fn live_enemy_phase_selectors_use_contact_and_walk_phase() {
        let visuals = labelled_enemy_visuals(8, 7);
        let trace = NativeTraceFrame::default();

        // ASM Tar_Monster.asm off_32290: hidden and visible live states hold
        // distinct body frames.
        let mut tar = live_enemy_object(0x05);
        tar.contact_phase = Some(NativeContactPhase::TarMonsterHidden);
        assert_eq!(
            selected_live_tile_base(select_live_tar_monster_frame(&trace, &visuals, &tar)),
            0x0100
        );
        tar.contact_phase = None;
        assert_eq!(
            selected_live_tile_base(select_live_tar_monster_frame(&trace, &visuals, &tar)),
            0x0101
        );

        // ASM Hand.asm live scripts: walk, jump attack, and grab start from
        // separate body descriptors, all before the death tail.
        let mut hand = live_enemy_object(0x17);
        assert_eq!(
            selected_live_tile_base(select_live_hand_frame(&visuals, &hand)),
            0x0100
        );
        hand.contact_phase = Some(NativeContactPhase::HandJumping);
        assert_eq!(
            selected_live_tile_base(select_live_hand_frame(&visuals, &hand)),
            0x0103
        );
        hand.contact_phase = Some(NativeContactPhase::HandGrabbing);
        assert_eq!(
            selected_live_tile_base(select_live_hand_frame(&visuals, &hand)),
            0x0106
        );

        // ASM Lion.asm loc_3DFD8/loc_3E0F4: walk_phase indexes the HP-specific
        // duration table instead of using the generic layout-offset loop.
        let mut lion = live_enemy_object(0x12);
        lion.hit_points = Some(6);
        lion.walk_phase = Some(13);
        assert_eq!(
            selected_live_tile_base(select_live_lion_frame(&trace, &visuals, &lion)),
            0x0102
        );
        lion.hit_points = Some(8);
        lion.walk_phase = Some(5);
        assert_eq!(
            selected_live_tile_base(select_live_lion_frame(&trace, &visuals, &lion)),
            0x0101
        );

        let lion_visuals = labelled_enemy_visuals(6, 3);
        let mut lion = live_enemy_object(0x12);
        lion.layout_index = Some(2);
        let trace = NativeTraceFrame {
            frame_index: 1,
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            selected_live_tile_base(select_live_lion_frame(&trace, &lion_visuals, &lion)),
            0x0101,
            "ASM Lion.asm loc_3DFD8 layout offset is layout_index*3 before duration dispatch"
        );
        lion.walk_phase = Some(45);
        assert_eq!(
            selected_live_tile_base(select_live_lion_frame(&trace, &lion_visuals, &lion)),
            0x0101,
            "ASM Lion.asm loc_3E0F4 walk phase wraps by the live duration cycle"
        );
    }

    #[test]
    fn goat_live_selector_follows_patrol_charge_and_fixed_pose_scripts() {
        let visuals = labelled_enemy_visuals(15, 1);
        let mut trace = NativeTraceFrame::default();
        let mut goat = live_enemy_object(0x10);
        goat.shared_walker_state = Some(0);

        // ASM Goat.asm stru_3C97C: slow patrol uses 8/6/6/8/6/6 ticks.
        for (frame, expected) in [(0, 0), (7, 0), (8, 1), (14, 2), (20, 3), (39, 5), (40, 0)] {
            trace.frame_index = frame;
            assert_eq!(
                selected_live_tile_base(select_live_goat_frame(&trace, &visuals, &goat)),
                0x0100 + expected,
                "ASM Goat.asm stru_3C97C patrol boundary {frame}"
            );
        }

        // ASM Goat.asm loc_3E4CE/loc_3E50A selects the alternate charge bank.
        goat.shared_walker_state = Some(1);
        trace.frame_index = 0;
        assert_eq!(
            selected_live_tile_base(select_live_goat_frame(&trace, &visuals, &goat)),
            0x0109
        );

        // ASM Goat.asm stru_3C9E4 and loc_3CB76 hold C7C56.
        goat.damage_flash = true;
        assert_eq!(
            selected_live_tile_base(select_live_goat_frame(&trace, &visuals, &goat)),
            0x0106
        );
        goat.damage_flash = false;
        goat.wait_timer = Some(0xCBA8);
        assert_eq!(
            selected_live_tile_base(select_live_goat_frame(&trace, &visuals, &goat)),
            0x0106
        );
    }

    #[test]
    fn rock_tank_child_helpers_classify_loc_3b372_snapshots_and_flash() {
        let mut parent = live_enemy_object(0x0B);
        parent.layout_index = Some(7);
        parent.hit_points = Some(4);
        parent.damage_flash = true;
        let child = NativeObjectSnapshot {
            hit_points: None,
            damage_flash: false,
            on_ground: None,
            ..parent.clone()
        };
        let zero_hp_child = NativeObjectSnapshot {
            hit_points: Some(0),
            ..child.clone()
        };
        let trace = NativeTraceFrame {
            active_objects: vec![parent.clone(), child.clone()],
            ..NativeTraceFrame::default()
        };

        // ASM Tank.asm loc_3B372 / Tank_Shooting.asm loc_3B5FA allocate only
        // object types $0A/$0B as linked turret children.
        assert!(is_rock_tank_type(0x0A));
        assert!(is_rock_tank_type(0x0B));
        assert!(!is_rock_tank_type(0x09));
        assert!(is_rock_tank_child_snapshot(0x0B, &child, &trace));
        assert!(is_rock_tank_child_snapshot(0x0B, &zero_hp_child, &trace));

        let live_not_child = NativeObjectSnapshot {
            hit_points: Some(1),
            ..child.clone()
        };
        assert!(!is_rock_tank_child_snapshot(0x0B, &live_not_child, &trace));
        assert!(!is_rock_tank_child_snapshot(0x09, &child, &trace));
        let wrong_layout_child = NativeObjectSnapshot {
            layout_index: Some(8),
            ..child.clone()
        };
        assert!(!is_rock_tank_child_snapshot(
            0x0B,
            &wrong_layout_child,
            &trace
        ));
        let no_parent_trace = NativeTraceFrame {
            active_objects: vec![child.clone()],
            ..NativeTraceFrame::default()
        };
        assert!(!is_rock_tank_child_snapshot(0x0B, &child, &no_parent_trace));
        let dead_parent_trace = NativeTraceFrame {
            active_objects: vec![NativeObjectSnapshot {
                hit_points: Some(0),
                ..parent.clone()
            }],
            ..NativeTraceFrame::default()
        };
        assert!(
            !is_rock_tank_child_snapshot(0x0B, &child, &dead_parent_trace),
            "a zero-HP Tank body is the ASM loc_3B480 death parent, not a live turret owner"
        );

        assert!(
            rock_tank_damage_flash(&child, &trace, 0x0B),
            "ASM Tank_Shooting.asm loc_3B700 flashes the linked turret with the parent hurt state"
        );
        let calm_parent_trace = NativeTraceFrame {
            active_objects: vec![NativeObjectSnapshot {
                damage_flash: false,
                ..parent.clone()
            }],
            ..NativeTraceFrame::default()
        };
        assert!(!rock_tank_damage_flash(&child, &calm_parent_trace, 0x0B));
        assert!(!rock_tank_damage_flash(
            &NativeObjectSnapshot {
                damage_flash: false,
                ..parent.clone()
            },
            &trace,
            0x0B
        ));

        assert!(
            !is_rock_tank_parent_without_child_snapshot(0x0B, &parent, &trace),
            "when the trace already has the ASM loc_3B5FA child, the parent must not synthesise another turret"
        );
        assert!(is_rock_tank_parent_without_child_snapshot(
            0x0B,
            &parent,
            &calm_parent_trace
        ));
        assert!(!is_rock_tank_parent_without_child_snapshot(
            0x09,
            &parent,
            &calm_parent_trace
        ));
        assert!(!is_rock_tank_parent_without_child_snapshot(
            0x0B,
            &child,
            &calm_parent_trace
        ));
        assert!(!is_rock_tank_parent_without_child_snapshot(
            0x0B,
            &child,
            &no_parent_trace
        ));

        assert_eq!(rock_tank_turret_screen_y(0x0A, 12), 12);
        assert_eq!(
            rock_tank_turret_screen_y(0x0B, 12),
            20,
            "ASM Tank_Shooting.asm loc_3B5FA stores the child at parent_y-8; native compensates the top-anchored shooting turret art"
        );
    }

    #[test]
    fn rock_tank_turret_scripts_follow_stru_3b27e_and_3b4d8_boundaries() {
        let visuals = labelled_enemy_visuals(1, 1);
        let mut trace = NativeTraceFrame::default();
        let plain = live_enemy_object(0x0A);
        let mut shooting_fast = live_enemy_object(0x0B);
        shooting_fast.hit_points = Some(4);
        let mut shooting_slow = shooting_fast.clone();
        shooting_slow.hit_points = Some(3);

        for (frame, tile) in [
            (0, 0x020),
            (0x37, 0x078),
            (0x37 + 0x0A, 0x084),
            (0x37 + 0x0A + 0x06, 0x078),
            (0x37 + 0x0A + 0x06 + 0x04, 0x020),
            (0x37 + 0x0A + 0x06 + 0x04 + 0x05, 0x020),
        ] {
            trace.frame_index = frame;
            assert_eq!(
                rock_tank_turret_frame(&plain, &trace, &visuals, 0x0A)
                    .expect("plain turret frame")
                    .tile_base_index,
                tile,
                "ASM Tank.asm stru_3B27E frame boundary {frame}"
            );
        }

        for (frame, tile) in [
            (0, 0x020),
            (0x37, 0x048),
            (0x37 + 0x0C, 0x054),
            (0x37 + 0x0C + 0x06, 0x060),
            (0x37 + 0x0C + 0x06 + 0x06, 0x054),
            (0x37 + 0x0C + 0x06 + 0x06 + 0x14, 0x048),
            (0x37 + 0x0C + 0x06 + 0x06 + 0x14 + 0x0C, 0x020),
        ] {
            trace.frame_index = frame;
            assert_eq!(
                rock_tank_turret_frame(&shooting_fast, &trace, &visuals, 0x0B)
                    .expect("shooting fast turret frame")
                    .tile_base_index,
                tile,
                "ASM Tank_Shooting.asm stru_3B4D8 frame boundary {frame}"
            );
        }

        trace.frame_index = 0x80 + 0x37;
        assert_eq!(
            rock_tank_turret_frame(&shooting_fast, &trace, &visuals, 0x0B)
                .expect("fast cycle wraps after $80 frames")
                .tile_base_index,
            0x054
        );
        assert_eq!(
            rock_tank_turret_frame(&shooting_slow, &trace, &visuals, 0x0B)
                .expect("slow cycle keeps the long idle tail")
                .tile_base_index,
            0x020
        );
        assert!(rock_tank_turret_frame(&plain, &trace, &visuals, 0x09).is_none());

        let mut child = NativeObjectSnapshot {
            hit_points: None,
            ..shooting_fast.clone()
        };
        child.layout_index = Some(3);
        let correct_parent = NativeObjectSnapshot {
            layout_index: Some(3),
            hit_points: Some(4),
            ..shooting_fast.clone()
        };
        trace.active_objects = vec![correct_parent];
        assert_eq!(
            rock_tank_turret_frame(&child, &trace, &visuals, 0x0B)
                .expect("child should inherit same-layout parent HP")
                .tile_base_index,
            0x054,
            "ASM Tank_Shooting.asm loc_3B5FA child must sample the live same-layout parent to choose stru_3B4D8"
        );
        for candidate in [
            NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x0A },
                layout_index: Some(3),
                hit_points: Some(4),
                ..shooting_fast.clone()
            },
            NativeObjectSnapshot {
                layout_index: Some(4),
                hit_points: Some(4),
                ..shooting_fast.clone()
            },
            NativeObjectSnapshot {
                layout_index: Some(3),
                hit_points: None,
                ..shooting_fast.clone()
            },
        ] {
            trace.active_objects = vec![candidate];
            assert_eq!(
                rock_tank_turret_frame(&child, &trace, &visuals, 0x0B)
                    .expect("child without matching live HP parent uses slow fallback")
                    .tile_base_index,
                0x020
            );
        }
    }

    #[test]
    fn rock_tank_body_selector_uses_three_frame_stru_3b270_cycle() {
        let mut trace = NativeTraceFrame {
            frame_index: 8,
            ..NativeTraceFrame::default()
        };
        let three_frames = labelled_enemy_visuals(3, 4);
        let object = live_enemy_object(0x0A);

        assert_eq!(
            selected_live_tile_base(select_live_rock_tank_body_frame(
                &trace,
                &three_frames,
                &object
            )),
            0x0102,
            "ASM Tank.asm stru_3B270 advances by frame_duration division before modulo over the three live body frames"
        );
        trace.frame_index = 9;
        assert_eq!(
            selected_live_tile_base(select_live_rock_tank_body_frame(
                &trace,
                &three_frames,
                &object
            )),
            0x0102
        );
    }

    #[test]
    fn crab_live_selector_keeps_normal_hurt_and_frozen_scripts_distinct() {
        let visuals = labelled_enemy_visuals(10, 8);
        let trace = NativeTraceFrame {
            frame_index: 48,
            ..NativeTraceFrame::default()
        };
        let mut crab = live_enemy_object(0x09);

        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &crab),
            0x0101,
            "ASM Crab.asm stru_3AF62 must cycle only C7C1E..C7C46 with its 8/6/6/8/6/6-tick timings"
        );

        crab.damage_flash = true;
        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &crab),
            0x0106,
            "ASM Crab.asm loc_3B0A6 / stru_3AF7C must hold C7C16 throughout the surviving-hit hibernate"
        );

        crab.damage_flash = false;
        crab.stationary = true;
        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &crab),
            0x0109,
            "ASM Crab.asm loc_3B104 must hold the unanimated C7C4E proximity pose"
        );
    }

    #[test]
    fn fire_demon_selector_keeps_normal_hurt_and_death_scripts_distinct() {
        let visuals = labelled_enemy_visuals(13, 6);
        let trace = NativeTraceFrame::default();
        let mut demon = live_enemy_object(0x00);

        demon.sprite_animation_phase = Some(3);
        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &demon),
            0x0103,
            "ASM Fire_Demon.asm stru_3A31A/stru_3A334 must select only the six walking mappings"
        );

        demon.damage_flash = true;
        demon.sprite_animation_phase = Some(4);
        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &demon),
            0x0106,
            "ASM Fire_Demon.asm loc_3A5BE / stru_3A34E frame four must select hurt mapping C773E"
        );

        demon.damage_flash = false;
        demon.sprite_animation_phase = None;
        demon.wait_timer = Some(1);
        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &demon),
            0x0106,
            "ASM Fire_Demon.asm loc_3A5E4 / loc_3A610 must hold C773E with animation disabled during Berzerker launch"
        );

        let dying = dead_enemy_object(0x00, 21);
        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &dying),
            0x0107,
            "ASM Fire_Demon.asm loc_3A5A6 / stru_3A378 must start at C7746 after the hurt mapping"
        );
    }

    #[test]
    fn fireball_live_snapshot_selects_loc_33072_angle_frame() {
        let visuals = labelled_enemy_visuals(24, 6);
        let trace = NativeTraceFrame::default();
        let mut fireball = live_enemy_object(0x19);
        fireball.sprite_animation_phase = Some(14);

        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &fireball),
            0x010E,
            "ASM Fireball.asm loc_32F3E/loc_3304A/loc_33072 must render the selected live angle mapping instead of cycling death art"
        );
    }

    #[test]
    fn tornado_live_snapshot_excludes_loc_359b2_death_frames() {
        let visuals = labelled_enemy_visuals(5, 5);
        let trace = NativeTraceFrame {
            frame_index: 15,
            ..NativeTraceFrame::default()
        };
        let mut tornado = live_enemy_object(0x18);
        tornado.sprite_animation_phase = Some(2);

        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &tornado),
            0x0102,
            "ASM Tornado.asm loc_35908 / stru_35B10 must render C785E instead of free-running into loc_359B2 death frame C783E"
        );
    }

    #[test]
    fn enemy_palette_variant_indexes_extra_palettes_after_base_line() {
        let mut visuals = labelled_enemy_visuals(1, 1);
        visuals.palette = marked_palette(0x0100);
        visuals.extra_palettes = vec![marked_palette(0x0200), marked_palette(0x0300)];

        assert_eq!(
            select_enemy_palette(&visuals, 0).colours[0].raw_word,
            0x0100
        );
        assert_eq!(
            select_enemy_palette(&visuals, 1).colours[0].raw_word,
            0x0200,
            "ASM enemy palette variant 1 selects the first extra palette after the base line"
        );
        assert_eq!(
            select_enemy_palette(&visuals, 2).colours[0].raw_word,
            0x0300,
            "variant indexing is one-based over the extra palette list"
        );
        assert_eq!(
            select_enemy_palette(&visuals, 3).colours[0].raw_word,
            0x0100,
            "out-of-range variants fall back to the base palette"
        );
    }

    #[test]
    fn enemy_palette_variant_uses_physical_layout_slot_unless_explicit() {
        let mut level_visuals = minimal_level_visuals_for_platforms();
        level_visuals
            .enemies
            .layout_palette_variants
            .insert(0x0D, 1);

        assert_eq!(
            enemy_palette_variant(&level_visuals, 0x0D, 0),
            1,
            "ASM Load_EnemyArtPaletteToVRAM loc_36D16 derives the ordinary Dragon palette from its second physical layout slot"
        );
        assert_eq!(
            enemy_palette_variant(&level_visuals, 0x0D, 2),
            2,
            "runtime object palette variants remain explicit overrides"
        );
    }

    #[test]
    fn enemy_death_threshold_scripts_select_asm_frames() {
        let trace = NativeTraceFrame::default();

        for (object_type, frame_count, cases, asm_label) in [
            (
                0x00,
                13,
                &[(21, 7), (18, 8), (15, 9), (12, 10), (8, 11), (4, 12)][..],
                "ASM Fire_Demon.asm loc_3A5A6 `stru_3A378`",
            ),
            (
                0x01,
                10,
                &[(22, 5), (14, 6), (8, 7), (5, 8), (2, 9)],
                "ASM Diamond.asm loc_3AE28/loc_3AEE6 `stru_3ACC4`",
            ),
            (
                0x03,
                8,
                &[(18, 6), (9, 7)],
                "ASM Robot.asm loc_3A9D2 `stru_3A74A`",
            ),
            (
                0x06,
                6,
                &[(30, 3), (20, 4), (10, 5)],
                "ASM Sphere.asm Enemy06_Sphere_Kill `stru_36182`",
            ),
            (
                0x07,
                12,
                &[(50, 8), (45, 9), (40, 10), (35, 11)],
                "ASM Archer.asm loc_3414A `stru_33FC2`",
            ),
            (
                0x09,
                9,
                &[(60, 6), (31, 7), (30, 8)],
                "ASM Crab.asm loc_3B094 `stru_3AF82`",
            ),
            (
                0x0A,
                6,
                &[(84, 3), (68, 4), (60, 5)],
                "ASM Tank.asm loc_3B480 `stru_3B29A`",
            ),
            (
                0x0B,
                6,
                &[(84, 3), (68, 4), (60, 5)],
                "ASM Tank_Shooting.asm loc_3B6E6 `stru_3B518`",
            ),
            (
                0x0E,
                9,
                &[(21, 6), (14, 7), (7, 8)],
                "ASM Cloud.asm loc_35C0E `stru_35EBA`",
            ),
            (
                0x10,
                9,
                &[(95, 6), (65, 7), (55, 8)],
                "ASM Goat.asm loc_3CB64 `stru_3C9EA`",
            ),
            (
                0x11,
                9,
                &[(75, 6), (55, 7), (45, 8)],
                "ASM Ninja.asm loc_3DB4E `stru_3D50A`",
            ),
            (
                0x12,
                9,
                &[(30, 6), (20, 7), (10, 8)],
                "ASM Lion.asm loc_3E378 `stru_3DF48`",
            ),
            (
                0x13,
                9,
                &[(75, 6), (65, 7), (45, 8)],
                "ASM Scorpion.asm loc_3DDEE `stru_3DB80`",
            ),
            (
                0x14,
                7,
                &[(60, 4), (35, 5), (20, 6)],
                "ASM Spinning_Twins.asm loc_3BC56 `stru_3B83C`",
            ),
            (
                0x16,
                6,
                &[(12, 3), (8, 4), (4, 5)],
                "ASM Drips.asm loc_3392C `stru_33F06`",
            ),
            (
                0x17,
                14,
                &[
                    (66, 7),
                    (60, 8),
                    (54, 7),
                    (48, 8),
                    (42, 7),
                    (36, 8),
                    (30, 9),
                    (24, 10),
                    (18, 11),
                    (12, 12),
                    (6, 13),
                ],
                "ASM Hand.asm loc_32AA2 `stru_32B78`",
            ),
            (
                0x18,
                5,
                &[(26, 3), (25, 3), (20, 4), (17, 3), (13, 4), (9, 3), (6, 4)],
                "ASM Tornado.asm loc_359B2 `stru_35B1E`",
            ),
            (
                0x1A,
                5,
                &[(70, 2), (60, 3), (40, 4)],
                "ASM Driller.asm loc_3BF1C `stru_3BCD4`",
            ),
            (
                0x1B,
                8,
                &[(30, 3), (24, 4), (18, 5), (12, 6), (6, 7)],
                "ASM Emo_Rock.asm loc_36346 `stru_36468`",
            ),
            (
                0x1D,
                5,
                &[(146, 1), (122, 2), (110, 3), (100, 4)],
                "ASM Big_Hopping_Skull.asm loc_3664E `stru_367FA`",
            ),
        ] {
            let visuals = labelled_enemy_visuals(frame_count, 3);
            for &(remaining, frame_index) in cases {
                let object = dead_enemy_object(object_type, remaining);
                assert_eq!(
                    selected_enemy_tile_base(&trace, &visuals, &object),
                    0x0100 + frame_index,
                    "{asm_label} should select frame {frame_index} when wait_timer={remaining}"
                );
            }
        }

        for object_type in [0x0C, 0x19] {
            let visuals = labelled_enemy_visuals(4, 3);
            for (remaining, frame_index) in [(8, 0), (6, 1), (4, 2), (2, 3)] {
                let object = dead_enemy_object(object_type, remaining);
                assert_eq!(
                    selected_enemy_tile_base(&trace, &visuals, &object),
                    0x0100 + frame_index,
                    "ASM shared `stru_345E4` death script should select frame {frame_index} for object {object_type:#04X}"
                );
            }
        }

        let ufo_visuals = labelled_enemy_visuals(9, 3);
        for (remaining, frame_index) in [(21, 6), (14, 7), (7, 8)] {
            let object = dead_enemy_object(0x0F, remaining);
            assert_eq!(
                selected_enemy_tile_base(&trace, &ufo_visuals, &object),
                0x0100 + frame_index,
                "ASM UFO.asm loc_35174/loc_35180 should select parent death frame {frame_index}"
            );
        }

        let mini_skull_visuals = labelled_enemy_visuals(8, 3);
        for (remaining, frame_index) in [(40, 6), (30, 7)] {
            let object = dead_enemy_object(0x1C, remaining);
            assert_eq!(
                selected_enemy_tile_base(&trace, &mini_skull_visuals, &object),
                0x0100 + frame_index,
                "ASM Mini_Hopping_Skull.asm loc_3F158 should select frame {frame_index}"
            );
        }
    }

    #[test]
    fn diamond_death_script_pins_all_cumulative_threshold_edges() {
        let trace = NativeTraceFrame::default();
        let visuals = labelled_enemy_visuals(10, 3);

        for (remaining, frame_index) in [
            (17, 5),
            (15, 5),
            (14, 6),
            (11, 6),
            (9, 6),
            (8, 7),
            (6, 7),
            (5, 8),
            (3, 8),
            (2, 9),
        ] {
            let object = dead_enemy_object(0x01, remaining);
            assert_eq!(
                selected_enemy_tile_base(&trace, &visuals, &object),
                0x0100 + frame_index,
                "ASM Diamond.asm loc_3AE28/loc_3AEE6 cumulative death threshold for wait_timer={remaining}"
            );
        }
    }

    #[test]
    fn crab_death_script_uses_strict_first_threshold() {
        let trace = NativeTraceFrame::default();
        let visuals = labelled_enemy_visuals(9, 3);

        for (remaining, frame_index) in [(41, 6), (40, 7), (31, 7), (30, 8)] {
            let object = dead_enemy_object(0x09, remaining);
            assert_eq!(
                selected_enemy_tile_base(&trace, &visuals, &object),
                0x0100 + frame_index,
                "ASM Crab.asm loc_3B094 strict death threshold for wait_timer={remaining}"
            );
        }
    }

    #[test]
    fn enemy_death_scripts_fall_back_when_sheet_is_too_short() {
        let trace = NativeTraceFrame::default();

        for (object_type, required_frames, death_frame, asm_label) in [
            (0x00, 13, 7, "ASM Fire_Demon.asm loc_3A5A6 `stru_3A378`"),
            (
                0x01,
                10,
                5,
                "ASM Diamond.asm loc_3AE28/loc_3AEE6 `stru_3ACC4`",
            ),
            (0x03, 8, 6, "ASM Robot.asm loc_3A9D2 `stru_3A74A`"),
            (
                0x06,
                6,
                3,
                "ASM Sphere.asm Enemy06_Sphere_Kill `stru_36182`",
            ),
            (0x07, 12, 8, "ASM Archer.asm loc_3414A `stru_33FC2`"),
            (0x04, 15, 12, "ASM Armadillo.asm loc_3C886 `stru_3C69A`"),
            (0x05, 7, 2, "ASM Tar_Monster.asm loc_323E6 `stru_32700`"),
            (0x08, 11, 6, "ASM Orca.asm loc_3D268/loc_3D4BC `stru_3D11E`"),
            (0x09, 9, 6, "ASM Crab.asm loc_3B094 `stru_3AF82`"),
            (0x0A, 6, 3, "ASM Tank.asm loc_3B480 `stru_3B29A`"),
            (0x0B, 6, 3, "ASM Tank_Shooting.asm loc_3B6E6 `stru_3B518`"),
            (0x0D, 10, 6, "ASM Dragon.asm loc_3CEFC `stru_3CCEC`"),
            (0x0E, 9, 6, "ASM Cloud.asm loc_35C0E `stru_35EBA`"),
            (0x10, 9, 6, "ASM Goat.asm loc_3CB64 `stru_3C9EA`"),
            (0x11, 9, 6, "ASM Ninja.asm loc_3DB4E `stru_3D50A`"),
            (0x12, 9, 6, "ASM Lion.asm loc_3E378 `stru_3DF48`"),
            (0x13, 9, 6, "ASM Scorpion.asm loc_3DDEE `stru_3DB80`"),
            (0x0F, 9, 6, "ASM UFO.asm loc_35174/loc_35180 `stru_34CA0`"),
            (0x0C, 4, 0, "ASM Dragon_Flying.asm loc_3492C `stru_345E4`"),
            (
                0x19,
                4,
                0,
                "ASM Fireball.asm Enemy19_Fireball_Kill `stru_345E4`",
            ),
            (0x18, 5, 3, "ASM Tornado.asm loc_359B2 `stru_35B1E`"),
            (0x1B, 8, 3, "ASM Emo_Rock.asm loc_36346 `stru_36468`"),
            (0x1A, 5, 2, "ASM Driller.asm loc_3BF1C `stru_3BCD4`"),
            (
                0x1D,
                5,
                1,
                "ASM Big_Hopping_Skull.asm loc_3664E `stru_367FA`",
            ),
            (0x22, 5, 4, "ASM loc_39EBE/loc_39F7A boss-head death spin"),
            (0x14, 7, 4, "ASM Spinning_Twins.asm loc_3BC56 `stru_3B83C`"),
            (0x16, 6, 3, "ASM Drips.asm loc_3392C `stru_33F06`"),
            (0x17, 8, 1, "ASM Hand.asm loc_32AA2 `stru_32B78`"),
        ] {
            let visuals = labelled_enemy_visuals(required_frames - 1, 3);
            let object = dead_enemy_object(object_type, u16::MAX);
            assert_eq!(
                selected_enemy_tile_base(&trace, &visuals, &object),
                0x0100,
                "{asm_label} should not read death frame {death_frame} without {required_frames} extracted frames"
            );
        }
    }

    #[test]
    fn zero_based_enemy_death_guards_prevent_late_frame_overreads() {
        let trace = NativeTraceFrame::default();

        for (object_type, frame_count, wait_timer, asm_label) in [
            (0x0F, 8, 1, "ASM UFO.asm loc_35174/loc_35180 `stru_34CA0`"),
            (0x0C, 3, 1, "ASM Dragon_Flying.asm loc_3492C `stru_345E4`"),
            (
                0x19,
                3,
                1,
                "ASM Fireball.asm Enemy19_Fireball_Kill `stru_345E4`",
            ),
        ] {
            let visuals = labelled_enemy_visuals(frame_count, 3);
            let object = dead_enemy_object(object_type, wait_timer);
            assert_eq!(
                selected_enemy_tile_base(&trace, &visuals, &object),
                0x0100,
                "{asm_label} should fall back instead of reading a late death frame from a short sheet"
            );
        }
    }

    #[test]
    fn boss_head_death_selector_uses_loc_39ebe_when_sheet_is_available() {
        let trace = NativeTraceFrame::default();
        let visuals = labelled_enemy_visuals(5, 3);
        let object = dead_enemy_object(0x22, BOSS_HEAD_DEATH_ANIMATION_FRAMES_LOC_39EBE);

        assert_eq!(
            selected_enemy_tile_base(&trace, &visuals, &object),
            0x0104,
            "ASM loc_39EBE starts boss-head death on the first C8620 sweep frame"
        );
    }

    #[test]
    fn generic_enemy_fallback_animation_uses_layout_offset_and_stationary_hold() {
        let visuals = labelled_enemy_visuals(7, 4);
        let trace = NativeTraceFrame {
            frame_index: 11,
            ..NativeTraceFrame::default()
        };
        let mut object = live_enemy_object(0x02);
        object.layout_index = Some(2);

        // Generic fallback path: ASM uses per-object animation counters; native
        // approximates this with a layout offset so repeated enemies desync.
        assert_eq!(selected_enemy_tile_base(&trace, &visuals, &object), 0x0104);

        object.layout_index = None;
        assert_eq!(selected_enemy_tile_base(&trace, &visuals, &object), 0x0102);

        object.layout_index = Some(2);
        object.stationary = true;
        assert_eq!(selected_enemy_tile_base(&trace, &visuals, &object), 0x0100);
    }

    #[test]
    fn enemy_death_waited_sequences_select_asm_frames() {
        let trace = NativeTraceFrame::default();

        for (object_type, frame_count, cases, asm_label) in [
            (
                0x04,
                15,
                &[
                    (48, 12),
                    (26, 13),
                    (22, 12),
                    (18, 13),
                    (15, 12),
                    (12, 13),
                    (10, 12),
                    (8, 13),
                    (7, 12),
                    (6, 13),
                    (5, 12),
                    (4, 13),
                    (3, 14),
                ][..],
                "ASM Armadillo.asm loc_3C886 `stru_3C69A`",
            ),
            (
                0x05,
                7,
                &[
                    (63, 2),
                    (54, 3),
                    (45, 4),
                    (42, 3),
                    (39, 4),
                    (30, 5),
                    (27, 4),
                    (24, 5),
                    (15, 6),
                    (12, 5),
                    (9, 6),
                ],
                "ASM Tar_Monster.asm loc_323E6 `stru_32700`",
            ),
            (
                0x08,
                11,
                &[
                    (156, 6),
                    (147, 7),
                    (143, 6),
                    (140, 8),
                    (123, 9),
                    (119, 8),
                    (115, 9),
                    (112, 8),
                    (109, 9),
                    (107, 8),
                    (105, 9),
                    (103, 8),
                    (101, 9),
                    (99, 10),
                ],
                "ASM Orca.asm loc_3D268/loc_3D4BC `stru_3D11E`",
            ),
            (
                0x0D,
                10,
                &[
                    (167, 6),
                    (160, 7),
                    (154, 6),
                    (151, 8),
                    (129, 9),
                    (124, 8),
                    (120, 9),
                    (116, 8),
                    (104, 9),
                    (99, 8),
                ],
                "ASM Dragon.asm loc_3CEFC `stru_3CCEC`",
            ),
        ] {
            let visuals = labelled_enemy_visuals(frame_count, 3);
            for &(remaining, frame_index) in cases {
                let object = dead_enemy_object(object_type, remaining);
                assert_eq!(
                    selected_enemy_tile_base(&trace, &visuals, &object),
                    0x0100 + frame_index,
                    "{asm_label} should select frame {frame_index} when wait_timer={remaining}"
                );
            }
        }
    }

    #[test]
    fn select_player_frame_prefers_asm_specific_pose_branches() {
        let visuals = labelled_player_visuals();

        // ASM loc_ABF0: pull-up hibernate advances every three frames and
        // takes precedence over ordinary motion.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    ledge_snap_cooldown: 12,
                    kid_x_velocity: Some(0x1000),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0900
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    ledge_snap_cooldown: 6,
                    kid_x_velocity: Some(0x1000),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0901
        );

        // ASM sub_B270 loc_B2A0: Cyclone flight uses phase >> 8, clamped to
        // the loaded spin frames.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    cyclone_flying: true,
                    walk_visual_accumulator: 0x0200,
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0100
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Cyclone.helmet_id(),
                    cyclone_flying: true,
                    walk_visual_accumulator: 0x0200,
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0302
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Cyclone.helmet_id(),
                    cyclone_flying: true,
                    walk_visual_accumulator: 0x0500,
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0302
        );

        // ASM loc_AB0A/loc_AB4A: the landing transfer list points at
        // `unk_BB996`, Cyclone walk 1, while grounded motion continues.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Cyclone.helmet_id(),
                    cyclone_grounded_sprite_source: Some(CycloneGroundedSpriteSource::Walk1),
                    kid_on_ground: Some(true),
                    kid_x_velocity: Some(0x1800),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0200
        );

        // ASM sub_942A loc_95BA: the retained landing mapping survives while
        // grounded motion continues, but the no-input stationary branch
        // writes the standing mapping.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Cyclone.helmet_id(),
                    cyclone_grounded_sprite_source: Some(CycloneGroundedSpriteSource::Walk4),
                    kid_on_ground: Some(true),
                    kid_x_velocity: Some(0x1800),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0202
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Cyclone.helmet_id(),
                    cyclone_grounded_sprite_source: Some(CycloneGroundedSpriteSource::Walk4),
                    kid_on_ground: Some(true),
                    kid_x_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0100
        );

        // ASM sub_B270 loc_B350/loc_B380: airborne down slash wins over the
        // normal sword timer and indexes by counter >> 5.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(false),
                    red_stealth_airborne_slash_counter: 0x40,
                    red_stealth_sword_timer: 9,
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0B02
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(false),
                    red_stealth_airborne_slash_counter: 0xE0,
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0B03
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(true),
                    red_stealth_sword_timer: 6,
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0A02
        );
        let mut short_slash_visuals = labelled_player_visuals();
        short_slash_visuals.slash_ground.truncate(2);
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(false),
                    red_stealth_airborne_slash_counter: 0x40,
                    red_stealth_sword_timer: 9,
                    ..NativeTraceFrame::default()
                },
                &short_slash_visuals
            ),
            0x0B02
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(true),
                    red_stealth_sword_timer: 1,
                    ..NativeTraceFrame::default()
                },
                &short_slash_visuals
            ),
            0x0A01
        );

        // ASM sub_B270 loc_B334/loc_B350 -> off_B3B8: the dynamic jump
        // threshold selects four mapping-table entries, not a rise/fall pair.
        for (state, expected) in [
            (
                wildside_core::ModeLevelAirborneVisualMappingPathSubB270::AscendingState0,
                0x0800,
            ),
            (
                wildside_core::ModeLevelAirborneVisualMappingPathSubB270::AscendingState1,
                0x0801,
            ),
            (
                wildside_core::ModeLevelAirborneVisualMappingPathSubB270::DescendingState2,
                0x0802,
            ),
            (
                wildside_core::ModeLevelAirborneVisualMappingPathSubB270::DescendingState3,
                0x0803,
            ),
        ] {
            assert_eq!(
                selected_tile_base(
                    &NativeTraceFrame {
                        kid_on_ground: Some(false),
                        airborne_visual_mapping_sub_b270: state,
                        ..NativeTraceFrame::default()
                    },
                    &visuals,
                ),
                expected,
                "ASM off_B3B8 mapping for {state:?}",
            );
        }

        // ASM loc_A276 hibernate attach falls through to idle; subsequent
        // sub_A3BC climb frames bucket d3 >> 2 through off_A420.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::IronKnight.helmet_id(),
                    iron_knight_climbing: true,
                    iron_knight_climb_pose_accum: 0,
                    kid_y_velocity: Some(-0x4000),
                    kid_on_ground: Some(false),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0100
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::IronKnight.helmet_id(),
                    iron_knight_climbing: true,
                    iron_knight_climb_pose_accum: 0,
                    kid_y_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0500
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::IronKnight.helmet_id(),
                    iron_knight_climbing: true,
                    iron_knight_climb_pose_accum: 0x40,
                    kid_y_velocity: Some(-0x4000),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0501
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::IronKnight.helmet_id(),
                    iron_knight_climbing: true,
                    iron_knight_climb_pose_accum: 0x80,
                    kid_y_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0502
        );
        let mut short_climb_visuals = labelled_player_visuals();
        short_climb_visuals.climb.truncate(2);
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::IronKnight.helmet_id(),
                    iron_knight_climbing: true,
                    iron_knight_climb_pose_accum: 0x80,
                    kid_y_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &short_climb_visuals
            ),
            0x0501
        );
    }

    #[test]
    fn select_player_frame_uses_loc_959c_skid_only_for_backward_motion_as_the_kid() {
        let visuals = labelled_player_visuals();

        // ASM sub_942A loc_9580/loc_9596/loc_959C: only The Kid uses the
        // fixed skid mapping when motion opposes facing. Both signs converge
        // on the same mapping; forward motion and helmet forms keep walking.
        for (x_velocity, facing) in [(0x1000, Direction::Left), (-0x1000, Direction::Right)] {
            assert_eq!(
                selected_tile_base(
                    &NativeTraceFrame {
                        kid_on_ground: Some(true),
                        kid_x_velocity: Some(x_velocity),
                        kid_facing: Some(facing),
                        ..NativeTraceFrame::default()
                    },
                    &visuals
                ),
                0x0250
            );
        }
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(true),
                    kid_x_velocity: Some(0x1000),
                    kid_facing: Some(Direction::Right),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0200
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Cyclone.helmet_id(),
                    kid_on_ground: Some(true),
                    kid_x_velocity: Some(0x1000),
                    kid_facing: Some(Direction::Left),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0200
        );

        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(true),
                    kid_x_velocity: Some(0),
                    kid_facing: Some(Direction::Right),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0100
        );
    }

    #[test]
    fn select_player_frame_uses_accumulator_for_ground_slope_and_walk_poses() {
        let visuals = labelled_player_visuals();

        // ASM loc_9564/off_9688: Berzerker charge pose survives zero velocity.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Berzerker.helmet_id(),
                    berzerker_charge_pose: true,
                    walk_visual_accumulator: 0x0100,
                    kid_x_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0401
        );

        // ASM loc_A13A/off_A190: slope direction canonicalisation chooses
        // opposite tables for reverse/uphill-facing cases, with phase >> 8.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_slope: 1,
                    kid_facing: Some(Direction::Right),
                    walk_visual_accumulator: 0x0100,
                    kid_x_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0701
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Berzerker.helmet_id(),
                    kid_on_slope: -1,
                    kid_facing: Some(Direction::Right),
                    walk_visual_accumulator: 0x0100,
                    kid_x_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0601
        );
        let mut fallback_slope_visuals = labelled_player_visuals();
        fallback_slope_visuals.slope_walk_left.clear();
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_slope: 1,
                    kid_facing: Some(Direction::Right),
                    walk_visual_accumulator: 0x0100,
                    kid_x_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &fallback_slope_visuals
            ),
            0x0201
        );
        fallback_slope_visuals.slope_walk.clear();
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_slope: 1,
                    kid_facing: Some(Direction::Left),
                    walk_visual_accumulator: 0x0200,
                    kid_x_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &fallback_slope_visuals
            ),
            0x0202
        );

        // ASM loc_9580/loc_9600: Berzerker latched walk pose wins even when
        // sampled velocity is zero; ordinary walk still uses the same phase.
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    current_helmet_id: PlayerForm::Berzerker.helmet_id(),
                    berzerker_walk_pose: true,
                    walk_visual_accumulator: 0x0200,
                    kid_x_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0202
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    walk_visual_accumulator: 0x0200,
                    kid_x_velocity: Some(0x1000),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0202
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(false),
                    kid_y_velocity: Some(-1),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0800
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(false),
                    kid_y_velocity: Some(0),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0801
        );
        assert_eq!(
            selected_tile_base(
                &NativeTraceFrame {
                    kid_on_ground: Some(false),
                    kid_y_velocity: Some(1),
                    ..NativeTraceFrame::default()
                },
                &visuals
            ),
            0x0801
        );
    }

    #[test]
    fn select_player_transform_frame_follows_step_durations() {
        let mut visuals = labelled_player_visuals();
        visuals.transform_steps = vec![
            wildside_render::PlayerTransformStep {
                duration_frames: 2,
                frame: Some(labelled_frame(0x0C00)),
            },
            wildside_render::PlayerTransformStep {
                duration_frames: 3,
                frame: None,
            },
            wildside_render::PlayerTransformStep {
                duration_frames: 4,
                frame: Some(labelled_frame(0x0C01)),
            },
        ];

        assert!(
            select_player_transform_frame(
                &NativeTraceFrame {
                    form_change_flash_frames: 0,
                    ..NativeTraceFrame::default()
                },
                &visuals
            )
            .is_none()
        );
        assert_eq!(
            select_player_transform_frame(
                &NativeTraceFrame {
                    form_change_flash_frames: 9,
                    ..NativeTraceFrame::default()
                },
                &visuals
            )
            .map(|frame| frame.tile_base_index),
            Some(0x0C00)
        );
        assert_eq!(
            select_player_transform_frame(
                &NativeTraceFrame {
                    form_change_flash_frames: 7,
                    ..NativeTraceFrame::default()
                },
                &visuals
            )
            .map(|frame| frame.tile_base_index),
            None
        );
        assert_eq!(
            select_player_transform_frame(
                &NativeTraceFrame {
                    form_change_flash_frames: 4,
                    ..NativeTraceFrame::default()
                },
                &visuals
            )
            .map(|frame| frame.tile_base_index),
            Some(0x0C01)
        );
        assert_eq!(
            select_player_transform_frame(
                &NativeTraceFrame {
                    form_change_flash_frames: 1,
                    ..NativeTraceFrame::default()
                },
                &visuals
            )
            .map(|frame| frame.tile_base_index),
            Some(0x0C01)
        );
        assert_eq!(
            select_player_transform_frame(
                &NativeTraceFrame {
                    form_change_flash_frames: 1,
                    ..NativeTraceFrame::default()
                },
                &labelled_player_visuals()
            )
            .map(|frame| frame.tile_base_index),
            None
        );
    }

    /// ASM Kid_Transform loc_7D0A / stru_7FF8 (kid.asm): a synthesised
    /// Kid→IronKnight trace resolves art positionally from the phase table at
    /// every countdown value — no dependence on the legacy duration sum (the
    /// steps here carry deliberately wrong 1-frame durations). HelmetDefault
    /// descriptors (12/14) and the loc_7E90 cleanup frame (countdown 1) render
    /// the form default/idle art on the form palette; HelmetArt passes render
    /// the descriptor's step art on the sub_8106 transform palette.
    #[test]
    fn select_player_transform_render_follows_iron_knight_phase_table_loc_7d0a() {
        let transform_colour = wildside_render::MdColour {
            red: 0xEE,
            green: 0x00,
            blue: 0x00,
            raw_word: 0x000E,
        };
        let mut iron_knight = labelled_player_visuals();
        iron_knight.transform_steps = (0u16..16)
            .map(|descriptor_index| wildside_render::PlayerTransformStep {
                duration_frames: 1,
                frame: if descriptor_index == 12 || descriptor_index == 14 {
                    None
                } else {
                    Some(labelled_frame(0x0C00 + descriptor_index))
                },
            })
            .collect();
        iron_knight.transform_palette = Some(wildside_render::Palette {
            colours: vec![transform_colour],
        });
        let mut forms = std::collections::HashMap::new();
        forms.insert(0u16, labelled_player_visuals());
        forms.insert(6u16, iron_knight);
        let player_visuals = wildside_render::PlayerVisuals { forms };
        let form_visuals = &player_visuals.forms[&6];

        // stru_7FF8 pass counts restated: descriptor per displayed pass.
        let run_lengths: [(u16, u8); 16] = [
            (0, 7),
            (1, 6),
            (2, 5),
            (3, 5),
            (4, 4),
            (5, 4),
            (6, 4),
            (7, 4),
            (8, 5),
            (9, 5),
            (10, 5),
            (11, 5),
            (12, 5),
            (13, 5),
            (14, 9),
            (15, 3),
        ];
        let mut expected_descriptors: Vec<u16> = Vec::new();
        for (descriptor_index, pass_count) in run_lengths {
            expected_descriptors.extend(std::iter::repeat_n(
                descriptor_index,
                usize::from(pass_count),
            ));
        }
        assert_eq!(expected_descriptors.len(), 81);

        for (offset, descriptor_index) in expected_descriptors.iter().copied().enumerate() {
            let countdown = 82 - u8::try_from(offset).unwrap();
            let trace = NativeTraceFrame {
                form_change_flash_frames: countdown,
                current_helmet_id: 6,
                transform_source_helmet_id_loc_7ca6: Some(0),
                ..NativeTraceFrame::default()
            };
            let (frame, palette) =
                select_player_transform_render(&trace, &player_visuals, form_visuals)
                    .unwrap_or_else(|| panic!("no transform render at countdown {countdown}"));
            if descriptor_index == 12 || descriptor_index == 14 {
                assert_eq!(frame.tile_base_index, 0x0100, "countdown {countdown}");
                assert!(palette.colours.is_empty(), "countdown {countdown}");
            } else {
                assert_eq!(
                    frame.tile_base_index,
                    0x0C00 + descriptor_index,
                    "countdown {countdown}"
                );
                assert_eq!(
                    palette.colours,
                    vec![transform_colour],
                    "countdown {countdown}"
                );
            }
        }

        // Countdown 1 is the loc_7E90 cleanup frame: form default art/palette.
        let cleanup_trace = NativeTraceFrame {
            form_change_flash_frames: 1,
            current_helmet_id: 6,
            transform_source_helmet_id_loc_7ca6: Some(0),
            ..NativeTraceFrame::default()
        };
        let (frame, palette) =
            select_player_transform_render(&cleanup_trace, &player_visuals, form_visuals).unwrap();
        assert_eq!(frame.tile_base_index, 0x0100);
        assert!(palette.colours.is_empty());

        // Without the stamped source helmet the legacy duration walk decides,
        // proving the ported path never engages on unstamped traces.
        let unstamped_trace = NativeTraceFrame {
            form_change_flash_frames: 82,
            current_helmet_id: 6,
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            select_player_transform_render(&unstamped_trace, &player_visuals, form_visuals)
                .map(|(frame, _)| frame.tile_base_index),
            select_player_transform_frame(&unstamped_trace, form_visuals)
                .map(|frame| frame.tile_base_index),
        );
    }

    /// ASM Kid_Transform loc_7D0A / stru_7F30 (kid.asm): Cyclone descriptor
    /// 0 enters selector 0 and therefore renders Kid walk-1 on Kid's palette;
    /// descriptor 7 enters selector 8 and restores Cyclone's default art.
    #[test]
    fn select_player_transform_render_follows_cyclone_selector_zero_loc_7d80() {
        let kid_colour = wildside_render::MdColour {
            red: 0x00,
            green: 0xEE,
            blue: 0x00,
            raw_word: 0x00E0,
        };
        let mut kid = labelled_player_visuals();
        kid.palette = wildside_render::Palette {
            colours: vec![kid_colour],
        };
        let mut cyclone = labelled_player_visuals();
        cyclone.palette = marked_palette(0x0500);
        cyclone.transform_palette = Some(marked_palette(0x0600));
        cyclone.transform_steps = (0u16..10)
            .map(|descriptor_index| wildside_render::PlayerTransformStep {
                duration_frames: 1,
                frame: Some(labelled_frame(0x0C00 + descriptor_index)),
            })
            .collect();
        let mut forms = std::collections::HashMap::new();
        forms.insert(0u16, kid);
        forms.insert(2u16, cyclone);
        let player_visuals = wildside_render::PlayerVisuals { forms };
        let cyclone_visuals = &player_visuals.forms[&2];

        let kid_trace = NativeTraceFrame {
            form_change_flash_frames: 73,
            current_helmet_id: 2,
            transform_source_helmet_id_loc_7ca6: Some(0),
            ..NativeTraceFrame::default()
        };
        let (frame, palette) =
            select_player_transform_render(&kid_trace, &player_visuals, cyclone_visuals)
                .expect("Cyclone selector-0 render");
        assert_eq!(frame.tile_base_index, 0x0200);
        assert_eq!(palette.colours, vec![kid_colour]);

        let default_trace = NativeTraceFrame {
            form_change_flash_frames: 29,
            ..kid_trace
        };
        let (frame, palette) =
            select_player_transform_render(&default_trace, &player_visuals, cyclone_visuals)
                .expect("Cyclone selector-8 render");
        assert_eq!(frame.tile_base_index, 0x0100);
        assert_eq!(
            palette.colours, cyclone_visuals.palette.colours,
            "selector 8 runs loc_7D5A's form palette path"
        );
    }

    #[test]
    fn transform_palette_kid_predicate_matches_loc_7cd0_old_form_frames() {
        let mut kid_visuals = labelled_player_visuals();
        kid_visuals.death = Some(labelled_frame(0x0D00));

        assert!(kid_transform_frame_uses_kid_palette_loc_7cd0(
            &kid_visuals.idle,
            &kid_visuals
        ));
        assert!(kid_transform_frame_uses_kid_palette_loc_7cd0(
            &kid_visuals.walk[1],
            &kid_visuals
        ));
        assert!(kid_transform_frame_uses_kid_palette_loc_7cd0(
            &kid_visuals.airborne[0],
            &kid_visuals
        ));
        assert!(kid_transform_frame_uses_kid_palette_loc_7cd0(
            &kid_visuals.airborne[2],
            &kid_visuals
        ));
        assert!(kid_transform_frame_uses_kid_palette_loc_7cd0(
            kid_visuals.death.as_ref().expect("death frame"),
            &kid_visuals
        ));
        assert!(kid_transform_frame_uses_kid_palette_loc_7cd0(
            &kid_visuals.pull_up[1],
            &kid_visuals
        ));
        assert!(!kid_transform_frame_uses_kid_palette_loc_7cd0(
            &labelled_frame(0x0E00),
            &kid_visuals
        ));
    }

    #[test]
    fn player_and_enemy_damage_palettes_follow_asm_flash_scope() {
        let palette = marked_palette(0x0600);

        assert_eq!(
            player_palette_for_damage_phase(&palette, false)
                .colours
                .iter()
                .map(|colour| colour.raw_word)
                .collect::<Vec<_>>(),
            palette
                .colours
                .iter()
                .map(|colour| colour.raw_word)
                .collect::<Vec<_>>(),
            "ASM sub_80BC only whitens the player palette during the flash phase"
        );
        let player_flash = player_palette_for_damage_phase(&palette, true);
        assert!(
            player_flash.colours[..12]
                .iter()
                .all(|colour| colour.raw_word == 0x0FFF),
            "ASM sub_80BC writes twelve visible player colours"
        );
        assert_eq!(
            player_flash.colours[12].raw_word, 0x060C,
            "player flash must not overwrite palette slots beyond the twelve-colour ASM copy"
        );

        assert_eq!(
            enemy_palette_for_damage_flash(&palette, false)
                .colours
                .iter()
                .map(|colour| colour.raw_word)
                .collect::<Vec<_>>(),
            palette
                .colours
                .iter()
                .map(|colour| colour.raw_word)
                .collect::<Vec<_>>(),
            "enemy non-flash frames keep their family palette"
        );
        assert!(
            enemy_palette_for_damage_flash(&palette, true)
                .colours
                .iter()
                .all(|colour| colour.raw_word == 0x0FFF),
            "native enemy damage flash whitens every stored visible colour"
        );
    }

    #[test]
    fn walking_dragon_hurt_animation_keeps_normal_palette_loc_3cebe() {
        assert!(enemy_damage_palette_white_phase(0x0C, true));
        assert!(!enemy_damage_palette_white_phase(0x0D, true));
        assert!(!enemy_damage_palette_white_phase(0x0D, false));
    }

    #[test]
    fn heady_metal_palette_word_only_replaces_slot_15_sub_379e8() {
        let palette = marked_palette(0x0700);
        let mut object = live_enemy_object(0x20);
        object.debug_fields.insert(
            "heady_metal_palette_word_5e_sub_379e8".to_owned(),
            "0x0ACE".to_owned(),
        );

        let non_heady = enemy_palette_for_heady_metal_sub_379e8(&palette, &object, 0x1F);
        assert_eq!(
            non_heady
                .colours
                .iter()
                .map(|colour| colour.raw_word)
                .collect::<Vec<_>>(),
            palette
                .colours
                .iter()
                .map(|colour| colour.raw_word)
                .collect::<Vec<_>>(),
            "ASM sub_379E8 palette pulse belongs to Heady Metal object type $20 only"
        );

        let heady = enemy_palette_for_heady_metal_sub_379e8(&palette, &object, 0x20);
        assert_eq!(heady.colours[14].raw_word, 0x070E);
        assert_eq!(
            heady.colours[15].raw_word, 0x0ACE,
            "ASM sub_379E8 writes Palette_Buffer+$5E, palette slot 15"
        );
    }

    #[test]
    fn push_player_sprite_emits_selected_frame_palette_and_priority() {
        let mut visuals = minimal_level_visuals_for_platforms();
        let kid_visuals = visuals.player.forms.get_mut(&0).expect("Kid visuals");
        kid_visuals.palette = marked_palette(0x0800);
        kid_visuals.death = Some(marked_frame(0x0D00, 5, 2, 18, 9));

        let object = native_object_at(0x0120_0000, 0x0090_0000);
        let trace = NativeTraceFrame {
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0x0080_0000,
            kid_facing: Some(Direction::Right),
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();
        push_player_sprite_with_priority_override(
            &mut commands,
            &object,
            &trace,
            &visuals,
            false,
            Some(true),
        );
        let (x, y, palette, tiles) = only_sprite_command(&commands);
        assert_eq!((*x, *y), (24, 0));
        assert_eq!(palette.colours[0].raw_word, 0x0800);
        assert_eq!(
            tiles
                .iter()
                .map(|tile| tile.tile.tile_index)
                .collect::<Vec<_>>(),
            vec![0x0100, 0x0101, 0x0102, 0x0103]
        );
        assert!(
            tiles.iter().all(|tile| tile.tile.priority),
            "ASM CharacterCollision_TouchFlagpole loc_C070 may promote the current player sprite"
        );

        commands.clear();
        let resident_art = marked_tile_sheet(20, 0x0A);
        push_player_sprite_with_priority_and_resident_art(
            &mut commands,
            &object,
            &trace,
            &visuals,
            false,
            None,
            Some(&resident_art),
        );
        let [DrawCommand::SatSprite { tile_sheet, .. }] = commands.as_slice() else {
            panic!("player emits one SAT sprite");
        };
        assert_eq!(
            tile_sheet.as_ref(),
            &resident_art,
            "ASM loc_AD4 resident VRAM art overrides pixels without changing the selected SAT mapping",
        );

        commands.clear();
        let death_trace = NativeTraceFrame {
            death_windup_elapsed: 1,
            kid_facing: Some(Direction::Left),
            ..trace
        };
        push_player_sprite_with_priority_override(
            &mut commands,
            &object,
            &death_trace,
            &visuals,
            true,
            None,
        );
        let (x, y, palette, tiles) = only_sprite_command(&commands);
        assert_eq!(
            (*x, *y),
            (21, 0),
            "ASM Death loc_B7FA renders Kid death art even while the trace still carries the pre-death form"
        );
        assert!(
            palette.colours[..12]
                .iter()
                .all(|colour| colour.raw_word == 0x0FFF)
        );
        assert_eq!(
            tiles
                .iter()
                .map(|tile| tile.tile.tile_index)
                .collect::<Vec<_>>(),
            vec![0x0D00, 0x0D01, 0x0D02, 0x0D03, 0x0D04, 0x0D05]
        );
        assert!(
            tiles
                .iter()
                .all(|tile| tile.tile.flip_x && !tile.tile.priority),
            "death tumble uses facing flip but no flagpole priority override"
        );
    }

    #[test]
    fn player_screen_and_boss_eye_marker_use_md_sprite_coordinates() {
        assert_eq!(
            player_screen_position(
                &native_object_at(0x0123_8000, 0x0045_4000),
                &NativeTraceFrame {
                    camera_x_pos: 0x0020_0000,
                    camera_y_pos: 0x0010_0000,
                    ..NativeTraceFrame::default()
                }
            ),
            (0x0103, 0x0035)
        );

        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(0), (-4, -4));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(1), (0, -5));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(2), (4, -4));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(3), (-5, 0));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(4), (5, 0));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(5), (-4, 4));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(6), (0, 5));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(7), (4, 4));
        assert_eq!(boss_eye_attack_highlight_offset_sub_37abc(15), (4, 4));

        let mut commands = Vec::new();
        push_boss_eye_attack_marker_sub_37abc(&mut commands, 100, 80, 2);
        assert_eq!(
            commands,
            vec![
                DrawCommand::Rect {
                    x: 92,
                    y: 72,
                    width: 16,
                    height: 16,
                    colour: [120, 200, 255],
                },
                DrawCommand::Rect {
                    x: 102,
                    y: 74,
                    width: 4,
                    height: 4,
                    colour: [255, 255, 255],
                },
            ]
        );

        commands.clear();
        push_boss_eye_attack_marker_sub_37abc(&mut commands, 100, 80, 5);
        assert_eq!(
            commands[1],
            DrawCommand::Rect {
                x: 94,
                y: 82,
                width: 4,
                height: 4,
                colour: [255, 255, 255],
            },
            "ASM sub_37ABC lower-left body-piece marker keeps subtraction on both highlight axes"
        );
    }

    #[test]
    fn heady_metal_renderer_marks_traced_sub_37abc_body_piece_positions() {
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x20 },
            x_pos: 0x0200_0000,
            y_pos: 0x0200_0000,
            ..live_enemy_object(0x20)
        };
        let trace = NativeTraceFrame {
            camera_x_pos: 0x0100_0000,
            camera_y_pos: 0x0080_0000,
            heady_metal_body_pieces_sub_37abc: vec![
                wildside_core::HeadyMetalBodyPieceTraceSub37ABC {
                    slot_offset: 0x000C,
                    x_pos: 0x0120,
                    y_pos: 0x0090,
                    sprite_mapping_index: 7,
                },
                wildside_core::HeadyMetalBodyPieceTraceSub37ABC {
                    slot_offset: 0x0010,
                    x_pos: 0x00F0,
                    y_pos: 0x00C0,
                    sprite_mapping_index: 3,
                },
            ],
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_heady_metal_sub_37abc_body_piece_markers(&mut commands, &object, &trace);

        assert_eq!(
            commands,
            vec![
                DrawCommand::Rect {
                    x: 24,
                    y: 8,
                    width: 16,
                    height: 16,
                    colour: [120, 200, 255],
                },
                DrawCommand::Rect {
                    x: 34,
                    y: 18,
                    width: 4,
                    height: 4,
                    colour: [255, 255, 255],
                },
                DrawCommand::Rect {
                    x: -24,
                    y: 56,
                    width: 16,
                    height: 16,
                    colour: [120, 200, 255],
                },
                DrawCommand::Rect {
                    x: -23,
                    y: 62,
                    width: 4,
                    height: 4,
                    colour: [255, 255, 255],
                },
            ],
            "ASM sub_37ABC loc_37AC6 refreshes only live body-piece workers and renders them at camera-relative word coordinates"
        );
    }

    #[test]
    fn heady_metal_renderer_projects_init_body_offsets_when_trace_slots_absent() {
        let mut level_visuals = minimal_level_visuals_for_platforms();
        level_visuals
            .enemies
            .families
            .insert(0x20, labelled_enemy_visuals(1, 1));
        let object = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x20 },
            x_pos: 0x0064_0000,
            y_pos: 0x00A0_0000,
            ..live_enemy_object(0x20)
        };
        let trace = NativeTraceFrame {
            camera_x_pos: 0x0014_0000,
            camera_y_pos: 0x0030_0000,
            kid_x_pos: Some(0x0064_0000),
            kid_y_pos: Some(0x00A0_0000),
            ..NativeTraceFrame::default()
        };
        let mut commands = Vec::new();

        push_enemy_sprite(&mut commands, &object, &trace, &level_visuals, 0x20);

        assert_eq!(
            commands.len(),
            25,
            "ASM Enemy20_HeadyMetal_Init loc_39266 seeds twelve sub_37ABC body workers in addition to the parent sprite"
        );
        assert_eq!(
            commands[1],
            DrawCommand::Rect {
                x: 60,
                y: 18,
                width: 16,
                height: 16,
                colour: [120, 200, 255],
            },
            "ASM loc_39266 first body worker is anchor+(-$C,-$56), then sub_37ABC renders camera-relative"
        );
        assert_eq!(
            commands[2],
            DrawCommand::Rect {
                x: 62,
                y: 28,
                width: 4,
                height: 4,
                colour: [255, 255, 255],
            },
            "ASM sub_37A22 selects the below-right mapping for Kid below/right of the first body worker"
        );
        assert_eq!(
            commands[23],
            DrawCommand::Rect {
                x: 84,
                y: 98,
                width: 16,
                height: 16,
                colour: [120, 200, 255],
            },
            "ASM loc_39266 twelfth body worker is anchor+(+$C,-6)"
        );
        assert_eq!(
            commands[24],
            DrawCommand::Rect {
                x: 85,
                y: 104,
                width: 4,
                height: 4,
                colour: [255, 255, 255],
            },
            "ASM sub_37A22 selects the below-left mapping for Kid below/left of the twelfth body worker"
        );
    }

    #[test]
    fn player_sprite_priority_stays_low_except_flagpole_override_loc_c070() {
        let frame = synthetic_player_frame();

        let ordinary_tiles = build_player_sprite_tiles_with_flip(&frame, false, None);
        assert_eq!(ordinary_tiles.len(), 4);
        assert!(
            ordinary_tiles.iter().all(|tile| !tile.tile.priority),
            "ASM player sprite descriptors do not encode a global high-priority bit; normal Kid priority stays low"
        );

        let flagpole_tiles = build_player_sprite_tiles_with_flip(&frame, false, Some(true));
        assert!(
            flagpole_tiles.iter().all(|tile| tile.tile.priority),
            "ASM CharacterCollision_TouchFlagpole loc_C070 is the narrow first-frame priority override"
        );
    }
}
