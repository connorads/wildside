use super::*;

#[allow(dead_code)]
const MURDER_WALL_PATTERN_SUB_5E02: [u16; 4] = [0xA2FB, 0xA31F, 0xA343, 0xA367];

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn copy_murder_wall_palette_loc_140EE(
    source_palette: &[u16; 8],
    palette_buffer: &mut [u16; 0x40],
) {
    // ASM loc_140EE (kid.asm:26915): `moveq #7,d0` then `move.w (a0)+,(a1)+`
    // copies exactly eight palette words from `Pal_1408A` into
    // `Palette_Buffer+$20`, leaving every other palette slot untouched.
    palette_buffer[0x10..0x18].copy_from_slice(source_palette);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MurderWallWarningInputSub5E02 {
    pub(crate) murder_wall_enabled: bool,
    pub(crate) murder_wall_x_px: i16,
    pub(crate) murder_wall_reversed: bool,
    pub(crate) kid_hitbox_left_px: i16,
    pub(crate) kid_hitbox_right_px: i16,
    pub(crate) camera_x_px: i16,
    pub(crate) camera_y_px: u16,
    pub(crate) animation_timer: u8,
    pub(crate) animation_frame: u8,
    pub(crate) starting_number_sprites: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MurderWallWarningSpriteSub5E02 {
    pub(crate) y: u16,
    pub(crate) link: u16,
    pub(crate) tile_word: u16,
    pub(crate) x: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MurderWallWarningRenderSub5E02 {
    pub(crate) push_animation_timer: Option<u16>,
    pub(crate) push_animation_offset: Option<u16>,
    pub(crate) animation_timer: u8,
    pub(crate) animation_frame: u8,
    pub(crate) number_sprites: u8,
    pub(crate) sprites: Vec<MurderWallWarningSpriteSub5E02>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn compile_murder_wall_warning_sprites_sub_5E02(
    input: MurderWallWarningInputSub5E02,
) -> MurderWallWarningRenderSub5E02 {
    // ASM sub_5E02 (kid.asm:5840): when the murder wall is active and within
    // `$30` pixels of the camera, write Kid's push animation words and emit a
    // 2x12 warning-tile strip into the shared sprite table.
    let mut render = MurderWallWarningRenderSub5E02 {
        push_animation_timer: None,
        push_animation_offset: None,
        animation_timer: input.animation_timer.wrapping_sub(1),
        animation_frame: input.animation_frame,
        number_sprites: input.starting_number_sprites,
        sprites: Vec::new(),
    };

    if !input.murder_wall_enabled {
        return render;
    }

    if render.animation_timer == 0 {
        render.animation_timer = 5;
        render.animation_frame = render.animation_frame.wrapping_add(1) & 3;
    }

    if input.murder_wall_reversed {
        let overlap =
            i32::from(input.murder_wall_x_px) + 0x110 - i32::from(input.kid_hitbox_right_px);
        if overlap <= 0 {
            render.push_animation_timer = Some(4);
        }
    } else {
        let overlap =
            i32::from(input.murder_wall_x_px) + 0x30 - i32::from(input.kid_hitbox_left_px);
        if overlap >= 0 {
            render.push_animation_offset = Some(0x30);
            render.push_animation_timer = Some(4);
        }
    }

    let mut relative_wall_x = i32::from(input.camera_x_px) - i32::from(input.murder_wall_x_px);
    if input.murder_wall_reversed {
        relative_wall_x = -relative_wall_x;
    }
    if relative_wall_x >= 0x30 {
        return render;
    }

    let mut x = if input.murder_wall_reversed {
        0x190 + relative_wall_x
    } else {
        0x80 - relative_wall_x
    };
    let base_y = 0x80 - i32::from(input.camera_y_px / 0x30);
    let pattern = MURDER_WALL_PATTERN_SUB_5E02[usize::from(render.animation_frame & 3)];

    for column in 0..2 {
        let tile_offsets: [u16; 2] = if input.murder_wall_reversed {
            if column == 0 {
                [0x0812, 0x081B]
            } else {
                [0x0809, 0x0800]
            }
        } else if column == 0 {
            [0x0000, 0x0009]
        } else {
            [0x0012, 0x001B]
        };

        let mut y = base_y;
        for _ in 0..6 {
            for tile_offset in tile_offsets {
                render.number_sprites = render.number_sprites.wrapping_add(1);
                render.sprites.push(MurderWallWarningSpriteSub5E02 {
                    y: y as u16,
                    link: 0x0A00 + u16::from(render.number_sprites),
                    tile_word: pattern.wrapping_add(tile_offset),
                    x: x as u16,
                });
                y += 0x18;
            }
        }
        x += 0x18;
    }

    render
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LevelSpecialEffectsRoutineSub6034 {
    WeatherSub6048,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn dispatch_level_special_effects_sub_6034(
    level_special_effects: u16,
) -> Option<LevelSpecialEffectsRoutineSub6034> {
    // ASM sub_6034 (kid.asm:6032): `Level_Special_Effects` values 2 and 3
    // both dispatch through the local jump table to sub_6048; 0 and 1 return.
    let dispatch_index = level_special_effects.checked_sub(1)?;
    if dispatch_index == 0 {
        return None;
    }
    if dispatch_index <= 2 {
        return Some(LevelSpecialEffectsRoutineSub6034::WeatherSub6048);
    }
    None
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WeatherPaletteOpSub6048 {
    CopyPaletteBufferToBuffer2,
    CopyPaletteBufferToBuffer3,
    WritePaletteBuffer { start_word: u8, words: Vec<u16> },
    WritePaletteBuffer3 { start_word: u8, words: Vec<u16> },
}

const FALLING_WEATHER_SLOT_COUNT_SUB_61CA: usize = 5;
const WEATHER_IMPACT_SLOT_COUNT_SUB_61CA: usize = 10;
const FALLING_WEATHER_SPRITES_SUB_61CA: [u16; 6] = [0xA3B5, 0xA3B9, 0xA3BD, 0xA3C1, 0xA3C5, 0xA3C9];
const IMPACT_WEATHER_SPRITES_SUB_61CA: [u16; 2] = [0xA3CD, 0xA3D1];

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FallingWeatherSlotSub61CA {
    pub(crate) y_px: i16,
    pub(crate) x_px: i16,
    pub(crate) animation_cooldown: u8,
    pub(crate) animation_frame: u8,
}

impl FallingWeatherSlotSub61CA {
    #[allow(dead_code)]
    pub(crate) const fn inactive(animation_cooldown: u8, animation_frame: u8) -> Self {
        Self {
            y_px: -1,
            x_px: -1,
            animation_cooldown,
            animation_frame,
        }
    }

    fn is_active(self) -> bool {
        self.y_px >= 0
    }

    fn clear_position(&mut self) {
        self.y_px = -1;
        self.x_px = -1;
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WeatherImpactSlotSub61CA {
    pub(crate) y_px: i16,
    pub(crate) x_px: i16,
    pub(crate) animation_cooldown: u8,
    pub(crate) animation_frame: u8,
}

impl WeatherImpactSlotSub61CA {
    #[allow(dead_code)]
    pub(crate) const fn inactive(animation_cooldown: u8, animation_frame: u8) -> Self {
        Self {
            y_px: -1,
            x_px: -1,
            animation_cooldown,
            animation_frame,
        }
    }

    fn is_active(self) -> bool {
        self.y_px >= 0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherCollisionRowsSub61CA {
    pub(crate) rows: Vec<Vec<u16>>,
}

impl WeatherCollisionRowsSub61CA {
    fn word_at(&self, row: usize, col: usize) -> u16 {
        self.rows
            .get(row)
            .and_then(|cells| cells.get(col))
            .copied()
            .unwrap_or(0)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WeatherSpriteSub61CA {
    pub(crate) y: u16,
    pub(crate) link: u16,
    pub(crate) tile_word: u16,
    pub(crate) x: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherWorkerInputSub61CA {
    pub(crate) camera_x_px: i16,
    pub(crate) camera_y_px: i16,
    pub(crate) spawn_cooldown: u8,
    pub(crate) free_falling_slots: u8,
    pub(crate) falling_slots: [FallingWeatherSlotSub61CA; FALLING_WEATHER_SLOT_COUNT_SUB_61CA],
    pub(crate) impact_slots: [WeatherImpactSlotSub61CA; WEATHER_IMPACT_SLOT_COUNT_SUB_61CA],
    pub(crate) kid_hitbox_top_px: i16,
    pub(crate) kid_hitbox_bottom_px: i16,
    pub(crate) kid_hitbox_left_px: i16,
    pub(crate) kid_hitbox_right_px: i16,
    pub(crate) random_byte: u8,
    pub(crate) collision_rows: WeatherCollisionRowsSub61CA,
    pub(crate) starting_number_sprites: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherWorkerOutputSub61CA {
    pub(crate) spawn_cooldown: u8,
    pub(crate) free_falling_slots: u8,
    pub(crate) falling_slots: [FallingWeatherSlotSub61CA; FALLING_WEATHER_SLOT_COUNT_SUB_61CA],
    pub(crate) impact_slots: [WeatherImpactSlotSub61CA; WEATHER_IMPACT_SLOT_COUNT_SUB_61CA],
    pub(crate) kid_collision_timer: Option<u16>,
    pub(crate) number_sprites: u8,
    pub(crate) sprites: Vec<WeatherSpriteSub61CA>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn step_weather_worker_sub_61CA(
    input: WeatherWorkerInputSub61CA,
) -> WeatherWorkerOutputSub61CA {
    // ASM sub_61CA (kid.asm:6190): advance five falling weather slots,
    // respawn one free slot from the top camera row when `$FFFFFB3C` expires,
    // splash on terrain, hurt Kid on overlap, and emit splash/falling sprites.
    let mut output = WeatherWorkerOutputSub61CA {
        spawn_cooldown: input.spawn_cooldown,
        free_falling_slots: input.free_falling_slots,
        falling_slots: input.falling_slots,
        impact_slots: input.impact_slots,
        kid_collision_timer: None,
        number_sprites: input.starting_number_sprites,
        sprites: Vec::new(),
    };

    let delete_bottom_px = input.camera_y_px.wrapping_add(0x00E0);
    let delete_left_px = input.camera_x_px.wrapping_sub(0x0010);

    for slot in &mut output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        slot.y_px = slot.y_px.wrapping_add(4);
        slot.x_px = slot.x_px.wrapping_sub(1);
        slot.animation_cooldown = slot.animation_cooldown.wrapping_sub(1);
        if slot.animation_cooldown == 0 {
            slot.animation_cooldown = 6;
            slot.animation_frame = if slot.animation_frame >= 5 {
                0
            } else {
                slot.animation_frame.wrapping_add(1)
            };
        }

        if delete_bottom_px < slot.y_px || delete_left_px > slot.x_px {
            slot.clear_position();
            output.free_falling_slots = output.free_falling_slots.wrapping_add(1);
        }
    }

    output.spawn_cooldown = output.spawn_cooldown.wrapping_sub(1);
    if output.spawn_cooldown == 0 {
        output.spawn_cooldown = 0x25u8.wrapping_sub(input.random_byte & 0x0F);
        if output.free_falling_slots != 0 {
            for slot in &mut output.falling_slots {
                if slot.is_active() {
                    continue;
                }

                let spawn_x_px = input
                    .camera_x_px
                    .wrapping_add(i16::from(input.random_byte))
                    .wrapping_add(0x0028);
                let spawn_y_px = input.camera_y_px;
                let row_index = usize::from(
                    (u16::from_be_bytes(spawn_y_px.to_be_bytes())).wrapping_add(0x000F) >> 4,
                );
                let col_floor = usize::from(u16::from_be_bytes(spawn_x_px.to_be_bytes()) >> 4);
                let col_ceil = usize::from(
                    u16::from_be_bytes(spawn_x_px.to_be_bytes()).wrapping_add(0x000F) >> 4,
                );
                let first_word = input.collision_rows.word_at(row_index, col_floor);
                let blocked_first_tile = first_word & 0x000F != 0;
                let crosses_tile = col_ceil != col_floor;
                let blocked_second_tile = crosses_tile
                    && (input.collision_rows.word_at(row_index, col_floor + 1) & 0x00FF != 0);

                if blocked_first_tile || blocked_second_tile {
                    break;
                }

                // ASM sub_61CA loc_62C2 only rewrites Y/X; the animation word
                // is intentionally preserved from the reused slot.
                slot.y_px = spawn_y_px;
                slot.x_px = spawn_x_px;
                output.free_falling_slots = output.free_falling_slots.wrapping_sub(1);
                break;
            }
        }
    }

    for slot in &output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        let y_plus_2 = slot.y_px.wrapping_add(2);
        if y_plus_2 < input.kid_hitbox_top_px {
            continue;
        }
        if y_plus_2.wrapping_add(0x000C) > input.kid_hitbox_bottom_px {
            continue;
        }
        let x_plus_2 = slot.x_px.wrapping_add(2);
        if x_plus_2 > input.kid_hitbox_right_px {
            continue;
        }
        if x_plus_2.wrapping_add(0x000C) < input.kid_hitbox_left_px {
            continue;
        }

        output.kid_collision_timer = Some(0x0028);
    }

    for slot in &mut output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        let row_index =
            usize::from(u16::from_be_bytes(slot.y_px.to_be_bytes()).wrapping_add(0x000F) >> 4);
        let col_floor = usize::from(u16::from_be_bytes(slot.x_px.to_be_bytes()) >> 4);
        let collision_word = input.collision_rows.word_at(row_index, col_floor);
        let terrain_kind = collision_word & 0x7000;
        if terrain_kind == 0 {
            continue;
        }

        if let Some(impact_slot) = output
            .impact_slots
            .iter_mut()
            .find(|slot| !slot.is_active())
        {
            impact_slot.y_px = if terrain_kind == 0x4000 || terrain_kind == 0x5000 {
                slot.y_px.wrapping_add(0x000C)
            } else {
                slot.y_px
            };
            impact_slot.x_px = slot.x_px;
            impact_slot.animation_cooldown = 6;
            impact_slot.animation_frame = 0;
        }

        slot.clear_position();
        output.free_falling_slots = output.free_falling_slots.wrapping_add(1);
    }

    for slot in &mut output.impact_slots {
        if !slot.is_active() {
            continue;
        }

        slot.animation_cooldown = slot.animation_cooldown.wrapping_sub(1);
        if slot.animation_cooldown == 0 {
            slot.y_px = slot.y_px.wrapping_sub(4);
            slot.x_px = slot.x_px.wrapping_sub(4);
            slot.animation_cooldown = 6;
            slot.animation_frame = slot.animation_frame.wrapping_add(1);
            if slot.animation_frame >= 2 {
                slot.y_px = -1;
                slot.x_px = -1;
                slot.animation_cooldown = 1;
                slot.animation_frame = 0;
                continue;
            }
        }

        let sprite_y = slot
            .y_px
            .wrapping_sub(input.camera_y_px)
            .wrapping_add(0x0080);
        let sprite_x = slot
            .x_px
            .wrapping_sub(input.camera_x_px)
            .wrapping_add(0x0080);
        let frame_index = usize::from(slot.animation_frame.min(1));
        output.number_sprites = output.number_sprites.wrapping_add(1);
        output.sprites.push(WeatherSpriteSub61CA {
            y: u16::from_be_bytes(sprite_y.to_be_bytes()),
            link: 0x0500u16.wrapping_add(u16::from(output.number_sprites)),
            tile_word: IMPACT_WEATHER_SPRITES_SUB_61CA[frame_index],
            x: u16::from_be_bytes(sprite_x.to_be_bytes()),
        });
    }

    for slot in &output.falling_slots {
        if !slot.is_active() {
            continue;
        }

        let sprite_y = slot
            .y_px
            .wrapping_sub(input.camera_y_px)
            .wrapping_add(0x0080);
        let sprite_x = slot
            .x_px
            .wrapping_sub(input.camera_x_px)
            .wrapping_add(0x0080);
        let frame_index = usize::from(slot.animation_frame.min(5));
        output.number_sprites = output.number_sprites.wrapping_add(1);
        output.sprites.push(WeatherSpriteSub61CA {
            y: u16::from_be_bytes(sprite_y.to_be_bytes()),
            link: 0x0500u16.wrapping_add(u16::from(output.number_sprites)),
            tile_word: FALLING_WEATHER_SPRITES_SUB_61CA[frame_index],
            x: u16::from_be_bytes(sprite_x.to_be_bytes()),
        });
    }

    output
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherSpecialEffectsStepSub6048 {
    pub(crate) frame_counter: u16,
    pub(crate) level_special_effects: u16,
    pub(crate) foreground_theme_is_mountain: bool,
    pub(crate) background_theme_is_mountain: bool,
    pub(crate) fade_countdown: u16,
    pub(crate) fade_progress_latch: u16,
    pub(crate) palette_animation_speed: u8,
    pub(crate) pending_palette_mask_0: i16,
    pub(crate) weather_mode_byte: u8,
    pub(crate) thunder_timer: i16,
    pub(crate) hail_period_timer: u16,
    pub(crate) hail_active: bool,
    pub(crate) snow_spawn_cooldown: u8,
    pub(crate) snow_slot_count: u8,
    pub(crate) play_thunder_sound: bool,
    pub(crate) call_snow_worker: bool,
    pub(crate) palette_ops: Vec<WeatherPaletteOpSub6048>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn step_level_special_effects_sub_6048(
    mut state: WeatherSpecialEffectsStepSub6048,
    saved_storm_palette: [u16; 7],
    mountain_storm_foreground_palette: [u16; 15],
    thunder_flash_palette: [u16; 7],
    thunder_calm_palette: [u16; 7],
    random_byte: u8,
) -> WeatherSpecialEffectsStepSub6048 {
    // ASM sub_6048 (kid.asm:6050): drive storm palette fades, Mountain
    // thunder flashes, and the later snow/hail cadence before setting
    // `($FFFFFAD6).w=4`.
    state.fade_progress_latch = state.fade_countdown;
    state.play_thunder_sound = false;
    state.call_snow_worker = false;
    state.palette_ops.clear();

    if state.frame_counter <= 0x04B0 {
        state.weather_mode_byte = 4;
        return state;
    }

    if state.fade_countdown != 0 {
        state.palette_animation_speed = 0x7F;
        state.pending_palette_mask_0 = 0x0080;
        state
            .palette_ops
            .push(WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer2);
        state
            .palette_ops
            .push(WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer3);

        if state.foreground_theme_is_mountain {
            state.pending_palette_mask_0 = -1;
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0,
                    words: vec![saved_storm_palette[0]],
                });
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0x19,
                    words: saved_storm_palette[1..].to_vec(),
                });
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 1,
                    words: mountain_storm_foreground_palette.to_vec(),
                });
        } else {
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0x19,
                    words: saved_storm_palette.to_vec(),
                });
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0,
                    words: vec![0],
                });
        }

        state.fade_countdown = state.fade_countdown.wrapping_sub(4);
        state.weather_mode_byte = 4;
        return state;
    }

    if state.level_special_effects <= 3
        && state.background_theme_is_mountain
        && state.frame_counter > 0x05DC
    {
        state.thunder_timer = state.thunder_timer.wrapping_sub(1);
        if state.thunder_timer == 0 {
            state.play_thunder_sound = true;
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0,
                    words: vec![thunder_flash_palette[0]],
                });
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0x19,
                    words: thunder_flash_palette[1..].to_vec(),
                });
        } else if state.thunder_timer == -27 {
            state.thunder_timer = 0x00F7 - i16::from(random_byte & 0xF5);
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0,
                    words: vec![thunder_calm_palette[0]],
                });
            state
                .palette_ops
                .push(WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0x19,
                    words: thunder_calm_palette[1..].to_vec(),
                });
        }
    }

    if state.frame_counter > 0x0834
        && state.level_special_effects == 3
        && state.frame_counter > 0x0873
    {
        state.hail_period_timer = state.hail_period_timer.wrapping_sub(1);
        if state.hail_period_timer == 0 {
            state.hail_period_timer = if state.hail_active { 0x0258 } else { 0x04B0 };
            state.hail_active = !state.hail_active;
        }

        if !state.hail_active && state.snow_slot_count != 5 {
            state.snow_spawn_cooldown = 2;
        }
        state.call_snow_worker = true;
    }

    state.weather_mode_byte = 4;
    state
}

#[derive(Clone, Copy)]
struct LavaFireballSegment {
    frames: u8,
    delta_y: i16,
    sprite_kind: LavaFireballSpriteKind,
}

#[derive(Clone, Copy)]
enum LavaFireballSpriteKind {
    TallRise,
    MidRise,
    MidBody,
    MidFall,
    TallFall,
}

const LAVA_FIREBALL_SCRIPT: [LavaFireballSegment; 11] = [
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -4,
        sprite_kind: LavaFireballSpriteKind::TallRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -3,
        sprite_kind: LavaFireballSpriteKind::TallRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -3,
        sprite_kind: LavaFireballSpriteKind::MidRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -2,
        sprite_kind: LavaFireballSpriteKind::MidRise,
    },
    LavaFireballSegment {
        frames: 0x09,
        delta_y: -1,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x05,
        delta_y: 0,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x05,
        delta_y: 0,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x09,
        delta_y: 1,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: 2,
        sprite_kind: LavaFireballSpriteKind::MidFall,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: 2,
        sprite_kind: LavaFireballSpriteKind::TallFall,
    },
    LavaFireballSegment {
        frames: 0x15,
        delta_y: 4,
        sprite_kind: LavaFireballSpriteKind::TallFall,
    },
];

fn kid_hitbox_bounds_px(slice: &OpeningSliceState) -> (i32, i32, i32, i32) {
    let x_px = (slice.player.x_pos >> 16) as i32;
    let foot_y_px = (slice.player.y_pos >> 16) as i32;
    (
        x_px - PLAYER_HALF_WIDTH,
        x_px + PLAYER_HALF_WIDTH,
        foot_y_px - PLAYER_HEIGHT,
        foot_y_px,
    )
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserHitboxWindowLoc14AAA {
    pub(crate) left_px: i32,
    pub(crate) right_px: i32,
    pub(crate) top_px: i32,
    pub(crate) bottom_px: i32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_hitbox_window_loc_14AAA(
    geyser_x_px: u16,
    geyser_base_y_px: u16,
    geyser_current_height: u16,
) -> Option<LavaGeyserHitboxWindowLoc14AAA> {
    if geyser_current_height == 0 {
        return None;
    }

    let left_px = i32::from(geyser_x_px) + 3;
    let bottom_px = i32::from(geyser_base_y_px);
    // ASM sub_14A58 -> loc_14AAA (kid.asm:28116-28137): each failed compare
    // branches straight to `loc_14AAA`, so damage only applies when Kid stays
    // within the inclusive X window `[x+3, x+3+$1A]` and Y window
    // `[top_sprite_y+$10, base_y]`.
    Some(LavaGeyserHitboxWindowLoc14AAA {
        left_px,
        right_px: left_px + 0x1A,
        top_px: bottom_px - i32::from(geyser_current_height) + 0x10,
        bottom_px,
    })
}

#[allow(non_snake_case)]
fn lava_geyser_hits_kid_sub_14A58(geyser: &LavaGeyser, slice: &OpeningSliceState) -> bool {
    let Some(hitbox_window) =
        lava_geyser_hitbox_window_loc_14AAA(geyser.x_px, geyser.y_px, geyser.current_height)
    else {
        return false;
    };

    let (kid_left, kid_right, kid_top, kid_bottom) = kid_hitbox_bounds_px(slice);

    kid_right >= hitbox_window.left_px
        && kid_left <= hitbox_window.right_px
        && kid_top <= hitbox_window.bottom_px
        && kid_bottom >= hitbox_window.top_px
}

#[allow(non_snake_case)]
fn lava_fireball_hits_kid_sub_14AB0(
    fireball: &LavaFireball,
    slice: &OpeningSliceState,
    sprite_kind: LavaFireballSpriteKind,
) -> bool {
    let (kid_left, kid_right, kid_top, kid_bottom) = kid_hitbox_bounds_px(slice);
    let fireball_left = i32::from(fireball.x_px) + 1;
    let fireball_right = fireball_left + 6;
    let baseline_y = i32::from(fireball.current_y_px);
    // ASM sub_14AB0 uses d1 as the segment baseline and `$54(a5)` as the top
    // sprite row written by sub_14BD8. The table-specific adjustments below
    // mirror the five sprite-layout buckets (`unk_14664` .. `unk_14696`).
    let (fireball_top, fireball_bottom) = match sprite_kind {
        LavaFireballSpriteKind::TallRise => (baseline_y - 23, baseline_y - 8),
        LavaFireballSpriteKind::MidRise => (baseline_y - 18, baseline_y - 1),
        LavaFireballSpriteKind::MidBody => (baseline_y - 7, baseline_y - 1),
        LavaFireballSpriteKind::MidFall => (baseline_y - 12, baseline_y - 1),
        LavaFireballSpriteKind::TallFall => (baseline_y - 23, baseline_y + 6),
    };

    kid_right >= fireball_left
        && kid_left <= fireball_right
        && kid_top <= fireball_bottom
        && kid_bottom >= fireball_top
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_initial_delay_frames_loc_14772(encoded_height_word: u16) -> u16 {
    // ASM Obj_Lava_Geyser -> loc_14772 (kid.asm:27810-27820): the low nibble of
    // the encoded height indexes `word_14710`, whose entries are `0, 10, 20,
    // ... 150`. `loc_14772` then hibernates once per stored frame before the
    // eruption continues into the sound/effect script.
    (encoded_height_word & 0xF) * 10
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballInitStateLoc1499C {
    pub(crate) startup_delay_frames: u16,
    pub(crate) rise_height_steps: u16,
    pub(crate) biased_x_px: u16,
    pub(crate) biased_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_lava_fireball_state_loc_1499C(
    x_px: u16,
    y_px: u16,
    encoded_height_word: u16,
) -> LavaFireballInitStateLoc1499C {
    // ASM loc_1499C (kid.asm:28038): the USM2 fireball clears the sign bit from
    // `$1A(a5)`, uses the low nibble as a `word_14710` index for `$4E(a5)`,
    // shifts the remaining height into `$4A(a5)`, then stores `$44/$46(a5)`
    // with the usual `$80` sprite bias.
    let untagged_height_word = encoded_height_word & 0x7FFF;
    LavaFireballInitStateLoc1499C {
        startup_delay_frames: lava_geyser_initial_delay_frames_loc_14772(untagged_height_word),
        rise_height_steps: untagged_height_word >> 4,
        biased_x_px: x_px.wrapping_add(0x80),
        biased_y_px: y_px.wrapping_add(0x80),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballScriptStartLoc149CC {
    pub(crate) script_name: &'static str,
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) sprite_top_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_lava_fireball_script_loc_149CC(
    biased_x_px: u16,
    biased_y_px: u16,
) -> LavaFireballScriptStartLoc149CC {
    // ASM loc_149CC (kid.asm:28052): the USM2 fireball restarts from
    // `unk_146CC`, un-biases `$44/$46(a5)` back into `d0/d1`, and seeds
    // `$54(a5)` from the same unbiased baseline row before the segment loop at
    // `loc_149E4`.
    let x_px = biased_x_px.wrapping_sub(0x80);
    let y_px = biased_y_px.wrapping_sub(0x80);
    LavaFireballScriptStartLoc149CC {
        script_name: "unk_146CC",
        x_px,
        y_px,
        sprite_top_y_px: y_px,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballSegmentLoc149E4 {
    pub(crate) dbf_counter: u16,
    pub(crate) frames: u16,
    pub(crate) delta_y: i16,
    pub(crate) sprite_script_word: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaFireballScriptStepLoc149E4 {
    CooldownAndRestart,
    EmitSegment(LavaFireballSegmentLoc149E4),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballLoopResumeLoc149FC {
    pub(crate) hibernation_frames: u16,
    pub(crate) collision_refreshes: u16,
    pub(crate) sprite_script_restored: bool,
    pub(crate) next_baseline_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_lava_fireball_script_step_loc_149E4(
    script_words: &[u16],
) -> Option<LavaFireballScriptStepLoc149E4> {
    // ASM loc_149E4 (kid.asm:28060): each fireball script entry starts with
    // `move.w (a0)+,d2`; negative values branch to `loc_14A4C` for the `$3C`
    // frame hibernation. Non-negative entries load `$52(a5)` as the per-frame
    // Y delta, then resolve the next word as a `LoadGameModeData` sprite
    // script pointer before entering the segment loop at `loc_14A0A`.
    let (&dbf_counter, rest) = script_words.split_first()?;
    if (dbf_counter as i16) < 0 {
        return Some(LavaFireballScriptStepLoc149E4::CooldownAndRestart);
    }

    let (&delta_y_word, remaining) = rest.split_first()?;
    let &sprite_script_word = remaining.first()?;
    Some(LavaFireballScriptStepLoc149E4::EmitSegment(
        LavaFireballSegmentLoc149E4 {
            dbf_counter,
            frames: dbf_counter.wrapping_add(1),
            delta_y: delta_y_word as i16,
            sprite_script_word,
        },
    ))
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_loop_resume_loc_149FC(
    baseline_y_px: u16,
    delta_y: i16,
) -> LavaFireballLoopResumeLoc149FC {
    // ASM loc_149FC (kid.asm:28072): each taken fireball-segment back-edge
    // hibernates for one frame, re-runs `sub_14AB0`, restores `a2` from `a1`,
    // then advances `d1` by the signed `$52(a5)` delta before re-entering the
    // shared camera-window path at `loc_14A0A`.
    LavaFireballLoopResumeLoc149FC {
        hibernation_frames: 1,
        collision_refreshes: 1,
        sprite_script_restored: true,
        next_baseline_y_px: baseline_y_px.wrapping_add_signed(delta_y),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaFireballSegmentBackEdgeLoc14A46 {
    pub(crate) next_dbf_counter: u16,
    pub(crate) reenters_segment_loop: bool,
    pub(crate) advances_to_next_script_step: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_fireball_segment_back_edge_loc_14A46(
    dbf_counter: u16,
) -> LavaFireballSegmentBackEdgeLoc14A46 {
    // ASM loc_14A46 (kid.asm:28098): `dbf d2,loc_149FC` decrements the low
    // word once and loops back while the post-decrement value is not `$FFFF`.
    // When the counter reaches `$FFFF`, the segment finishes and control falls
    // through to the next script entry at `loc_149E4`.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    let reenters_segment_loop = next_dbf_counter != u16::MAX;
    LavaFireballSegmentBackEdgeLoc14A46 {
        next_dbf_counter,
        reenters_segment_loop,
        advances_to_next_script_step: !reenters_segment_loop,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_startup_script_loc_14788() -> &'static str {
    // ASM loc_14788 (kid.asm:27825): after the initial hibernation and optional
    // `sfx_Lava_Geyser_starting`, the very first eruption seeds `a0` from
    // `unk_146B6`, not the steady-state geyser script used by later cycles.
    "unk_146B6"
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_repeat_sound_loc_1478E(sound_suppressed: bool) -> Option<&'static str> {
    // ASM loc_1478E (kid.asm:27830): the repeat-cycle path plays
    // `sfx_Lava_Geyser` only when `$19(a3)` is clear; non-zero skips the sound
    // call and falls straight through to the steady-state script at loc_1479C.
    (!sound_suppressed).then_some("sfx_Lava_Geyser")
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_repeat_script_loc_1479C() -> &'static str {
    // ASM loc_1479C (kid.asm:27836): repeat eruptions reload `a0` from
    // `unk_146A0`, the steady-state geyser script that runs after the optional
    // sound gate at loc_1478E.
    "unk_146A0"
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserFullHeightStateLoc147A0 {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) hold_frames: u16,
    pub(crate) sprite_top_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn initialise_lava_geyser_full_height_state_loc_147A0(
    biased_x_px: u16,
    biased_y_px: u16,
) -> LavaGeyserFullHeightStateLoc147A0 {
    // ASM loc_147A0 (kid.asm:27839): `$44/$46(a5)` already include the VDP's
    // `$80` sprite bias, so the worker subtracts `$80` back out before seeding
    // `d0/d1`, stores the fixed `$1E` full-height timer in `$50(a5)`, and
    // initialises `$54(a5)` from that unbiased base Y row.
    let x_px = biased_x_px.saturating_sub(0x80);
    let y_px = biased_y_px.saturating_sub(0x80);
    LavaGeyserFullHeightStateLoc147A0 {
        x_px,
        y_px,
        hold_frames: 0x1E,
        sprite_top_y_px: y_px,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_peak_row_target_loc_14966(sprite_row_target: u16) -> u16 {
    // ASM loc_14966 (kid.asm:28018): when the alternating top-cap frame wraps
    // back to zero, the worker grows `$4A(a5)` by 2 before reloading `d3`.
    sprite_row_target.wrapping_add(2)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserPeakHoldStepLoc1496C {
    pub(crate) sprite_row_index: u16,
    pub(crate) frames_per_sprite: u16,
    pub(crate) next_hold_frames: u16,
    pub(crate) descending: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_peak_hold_step_loc_1496C(
    sprite_row_target: u16,
    hold_frames_remaining: u16,
) -> LavaGeyserPeakHoldStepLoc1496C {
    // ASM loc_1496C (kid.asm:28021): reload `d3` from `$4A(a5)`, use the
    // 6-frame top-cap cadence while holding, decrement `$50(a5)`, and only
    // switch to the 1-frame collapse cadence once the decremented timer hits 0.
    let next_hold_frames = hold_frames_remaining.wrapping_sub(1);
    let descending = next_hold_frames == 0;
    LavaGeyserPeakHoldStepLoc1496C {
        sprite_row_index: sprite_row_target,
        frames_per_sprite: if descending { 1 } else { 6 },
        next_hold_frames,
        descending,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserCollapseStepLoc1498C {
    pub(crate) next_sprite_row_index: u16,
    pub(crate) descending: bool,
    pub(crate) restart_startup_script: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_collapse_step_loc_1498C(
    sprite_row_index: u16,
) -> LavaGeyserCollapseStepLoc1498C {
    // ASM loc_1498C (kid.asm:28031): decrement `d3` once per collapse tick and
    // keep looping through loc_1485E while rows remain; when the decremented
    // count reaches zero, clear `$4D(a5)` and jump back to loc_14788.
    let next_sprite_row_index = sprite_row_index.wrapping_sub(1);
    let restart_startup_script = next_sprite_row_index == 0;
    LavaGeyserCollapseStepLoc1498C {
        next_sprite_row_index,
        descending: !restart_startup_script,
        restart_startup_script,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaGeyserScriptStepLoc147BA {
    HoldAtFullHeight,
    DisableForCooldown,
    EmitSpriteRows {
        sprite_rows_minus_one: u16,
        sprite_script_word: u16,
    },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn decode_lava_geyser_script_step_loc_147BA(
    script_words: &[u16],
) -> Option<LavaGeyserScriptStepLoc147BA> {
    // ASM loc_147BA (kid.asm:27847): each geyser script entry starts with a
    // word loaded into `d2`. `$FFFF` jumps to the dedicated full-height loop,
    // `$FFFE` disables the geyser for `$3C` frames, and every other value keeps
    // the next word in place so `d2` can count sprite rows while `a1/a2` point
    // at the `LoadGameModeData + script_word` sprite template.
    let (&sprite_rows_minus_one, rest) = script_words.split_first()?;
    Some(match sprite_rows_minus_one {
        0xFFFF => LavaGeyserScriptStepLoc147BA::HoldAtFullHeight,
        0xFFFE => LavaGeyserScriptStepLoc147BA::DisableForCooldown,
        _ => LavaGeyserScriptStepLoc147BA::EmitSpriteRows {
            sprite_rows_minus_one,
            sprite_script_word: *rest.first()?,
        },
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserDisableStepLoc1483A {
    pub(crate) sound: &'static str,
    pub(crate) disabled_frames: u16,
    pub(crate) resumes_at_repeat_loop: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_disable_step_loc_1483A() -> LavaGeyserDisableStepLoc1483A {
    // ASM loc_1483A (kid.asm:27900): the `$FFFE` script sentinel falls here,
    // queues `sfx_Lava_Geyser`, hibernates for `$3C` frames, then resumes at
    // the repeat-eruption path (`loc_1478E`) rather than the startup script.
    LavaGeyserDisableStepLoc1483A {
        sound: "sfx_Lava_Geyser",
        disabled_frames: 0x003C,
        resumes_at_repeat_loop: true,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserRiseResetLoc1484E {
    pub(crate) sprite_row_index: u16,
    pub(crate) frames_per_sprite_row: u16,
    pub(crate) sprite_frame_within_row: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_rise_reset_loc_1484E() -> LavaGeyserRiseResetLoc1484E {
    // ASM loc_1484E (kid.asm:27908): the upward-motion path restarts the row
    // cursor in `d3`, seeds `$48(a5)` with a fixed 2-frame cadence per sprite
    // row, and clears `$4C(a5)` before entering the shared `loc_14866` loop.
    LavaGeyserRiseResetLoc1484E {
        sprite_row_index: 0,
        frames_per_sprite_row: 2,
        sprite_frame_within_row: 0,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserPreLoopHibernateLoc1485E {
    pub(crate) hibernation_frames: u16,
    pub(crate) collision_refreshes: u16,
    pub(crate) falls_through_to_shared_loop: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_pre_loop_hibernate_loc_1485E() -> LavaGeyserPreLoopHibernateLoc1485E {
    // ASM loc_1485E (kid.asm:27915): before reloading `$48(a5)` into `d2` at
    // `loc_14866`, the worker hibernates exactly one frame, re-runs
    // `sub_14A58`, then falls through into the shared sprite-row loop.
    LavaGeyserPreLoopHibernateLoc1485E {
        hibernation_frames: 1,
        collision_refreshes: 1,
        falls_through_to_shared_loop: true,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserLoopBackEdgeLoc1486C {
    pub(crate) hibernation_frames: u16,
    pub(crate) collision_refreshes: u16,
    pub(crate) reenters_shared_loop: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_loop_back_edge_loc_1486C() -> LavaGeyserLoopBackEdgeLoc1486C {
    // ASM loc_1486C (kid.asm:27924): each taken `dbf d2,loc_1486C` back-edge
    // hibernates one frame, re-runs `sub_14A58`, then re-enters the shared
    // `loc_14874` sprite-row builder.
    LavaGeyserLoopBackEdgeLoc1486C {
        hibernation_frames: 1,
        collision_refreshes: 1,
        reenters_shared_loop: true,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserCameraWindowInputLoc147D6 {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) camera_x_px: u16,
    pub(crate) camera_y_px: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_is_inside_camera_window_loc_147D6(
    input: LavaGeyserCameraWindowInputLoc147D6,
) -> bool {
    // ASM loc_147D6 (kid.asm:27858): before building one sprite row, the lava
    // geyser worker culls against a padded camera window. X must stay within
    // `[Camera_X_pos - $20, Camera_X_pos + $160]` and Y within
    // `[Camera_Y_pos - $48, Camera_Y_pos + $128]`; anything outside branches to
    // `loc_1482E`, which hibernates one frame without drawing.
    let x_px = i32::from(input.x_px);
    let y_px = i32::from(input.y_px);
    let camera_x_px = i32::from(input.camera_x_px);
    let camera_y_px = i32::from(input.camera_y_px);

    let min_x = camera_x_px - 0x20;
    let max_x = min_x + 0x180;
    let min_y = camera_y_px - 0x48;
    let max_y = min_y + 0x170;

    x_px >= min_x && x_px <= max_x && y_px >= min_y && y_px <= max_y
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserTopSpriteWindowInputLoc14874 {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) sprite_row_index: u16,
    pub(crate) camera_x_px: u16,
    pub(crate) camera_y_px: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaGeyserTopSpriteWindowResultLoc14874 {
    Culled,
    SkipTopSprite { sprite_row_y_px: u16 },
    BuildTopSprite { sprite_row_y_px: u16 },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_top_sprite_window_loc_14874(
    input: LavaGeyserTopSpriteWindowInputLoc14874,
) -> LavaGeyserTopSpriteWindowResultLoc14874 {
    // ASM loc_14874 (kid.asm:27928): the shared geyser row loop first un-biases
    // `$44/$46(a5)` by `$80`, subtracts `$10 + ((d3 + 1) << 4)` from Y for the
    // current row, and culls against X `[Camera_X_pos - $20, Camera_X_pos +
    // $160]`. Rows above `Camera_Y_pos - $28` skip only the top-cap build via
    // `loc_148E6`; rows below `Camera_Y_pos + $108` branch straight to
    // `loc_14940` and emit nothing.
    let x_px = input.x_px.wrapping_sub(0x80);
    let sprite_row_y_px = input
        .y_px
        .wrapping_sub(0x80)
        .wrapping_sub(0x10)
        .wrapping_sub((input.sprite_row_index.wrapping_add(1)) << 4);

    let x_px_i32 = i32::from(x_px);
    let sprite_row_y_px_i32 = i32::from(sprite_row_y_px);
    let min_x = i32::from(input.camera_x_px) - 0x20;
    let max_x = min_x + 0x180;
    if x_px_i32 < min_x || x_px_i32 > max_x {
        return LavaGeyserTopSpriteWindowResultLoc14874::Culled;
    }

    let min_y = i32::from(input.camera_y_px) - 0x28;
    if sprite_row_y_px_i32 < min_y {
        return LavaGeyserTopSpriteWindowResultLoc14874::SkipTopSprite { sprite_row_y_px };
    }

    let max_y = min_y + 0x130;
    if sprite_row_y_px_i32 > max_y {
        return LavaGeyserTopSpriteWindowResultLoc14874::Culled;
    }

    LavaGeyserTopSpriteWindowResultLoc14874::BuildTopSprite { sprite_row_y_px }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_top_sprite_script_loc_148CE(sprite_frame_within_row: u8) -> &'static str {
    // ASM loc_148CE (kid.asm:27959): when `$4C(a5)` is non-zero the geyser uses
    // `dword_14636` for the top sprite-row template; zero keeps `unk_14624`.
    if sprite_frame_within_row == 0 {
        "unk_14624"
    } else {
        "dword_14636"
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_store_top_sprite_y_loc_148D4(
    sprite_screen_y_with_bias: u16,
    camera_y_px: u16,
) -> u16 {
    // ASM loc_148D4 (kid.asm:27962): after `sub_14BD8` emits the selected
    // top-cap script, `d7` holds the built screen Y with the usual `$80` bias.
    // The worker un-biases it and stores world Y in `$54(a5)` via
    // `d7 - $80 + Camera_Y_pos`.
    sprite_screen_y_with_bias
        .wrapping_sub(0x80)
        .wrapping_add(camera_y_px)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LavaGeyserMiddleSpriteRowResultLoc148F8 {
    SkipAboveWindow,
    BuildSpriteRow,
    CulledBelowWindow,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_middle_sprite_row_window_loc_148F8(
    sprite_row_y_px: u16,
    camera_y_px: u16,
) -> LavaGeyserMiddleSpriteRowResultLoc148F8 {
    // ASM loc_148F8 (kid.asm:27977): each middle row checks `d1` against the
    // inclusive vertical band `[Camera_Y_pos - $10, Camera_Y_pos + $F0]`.
    // Rows above the band skip this build and fall through to loc_14914;
    // rows below it branch to loc_14940 and stop the current geyser frame.
    let sprite_row_y_px = i32::from(sprite_row_y_px);
    let min_y = i32::from(camera_y_px) - 0x10;
    if sprite_row_y_px < min_y {
        return LavaGeyserMiddleSpriteRowResultLoc148F8::SkipAboveWindow;
    }

    let max_y = min_y + 0x100;
    if sprite_row_y_px > max_y {
        return LavaGeyserMiddleSpriteRowResultLoc148F8::CulledBelowWindow;
    }

    LavaGeyserMiddleSpriteRowResultLoc148F8::BuildSpriteRow
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LavaGeyserMiddleRowAdvanceResultLoc14914 {
    pub(crate) next_sprite_row_y_px: u16,
    pub(crate) next_middle_row_index: u16,
    pub(crate) continue_middle_rows: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_middle_row_advance_loc_14914(
    sprite_row_y_px: u16,
    middle_row_index: u16,
    middle_row_limit: u16,
) -> LavaGeyserMiddleRowAdvanceResultLoc14914 {
    // ASM loc_14914 (kid.asm:27988): after each middle-row visit, the geyser
    // advances `d1` by `$10`, increments `d5`, and loops back to loc_148F8
    // while the updated index still satisfies `d5 <= d3`.
    let next_sprite_row_y_px = sprite_row_y_px.wrapping_add(0x10);
    let next_middle_row_index = middle_row_index.wrapping_add(1);
    LavaGeyserMiddleRowAdvanceResultLoc14914 {
        next_sprite_row_y_px,
        next_middle_row_index,
        continue_middle_rows: next_middle_row_index <= middle_row_limit,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_inter_row_hibernations_loc_14804(sprite_rows_minus_one: u16) -> u16 {
    // ASM loc_14804 (kid.asm:27876): the `dbf d2,loc_14804` back-edge runs only
    // between sprite rows. With `d2` seeded from the "rows minus one" script
    // word, the worker hibernates one frame and re-runs `sub_14A58` exactly
    // once per additional row after the first.
    sprite_rows_minus_one
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lava_geyser_sprite_top_y_loc_1480C(
    sprite_screen_y_with_bias: u16,
    camera_y_px: u16,
) -> u16 {
    // ASM loc_1480C (kid.asm:27880): after `sub_14BD8` builds the current row,
    // the worker un-biases the returned screen Y by `$80`, adds `Camera_Y_pos`,
    // and stores the result in `$54(a5)` for the following collision checks.
    lava_geyser_store_top_sprite_y_loc_148D4(sprite_screen_y_with_bias, camera_y_px)
}

fn advance_lava_fireball(fireball: &mut LavaFireball) {
    if fireball.cooldown_timer > 0 {
        fireball.cooldown_timer -= 1;
        if fireball.cooldown_timer == 0 {
            fireball.segment_index = 0;
            fireball.segment_frame = 0;
            fireball.current_y_px = fireball.base_y_px;
        }
        return;
    }

    let segment = LAVA_FIREBALL_SCRIPT[usize::from(fireball.segment_index)];
    fireball.current_y_px = fireball.current_y_px.saturating_add_signed(segment.delta_y);
    fireball.segment_frame = fireball.segment_frame.saturating_add(1);

    if fireball.segment_frame >= segment.frames {
        fireball.segment_frame = 0;
        if usize::from(fireball.segment_index) + 1 == LAVA_FIREBALL_SCRIPT.len() {
            fireball.segment_index = 0;
            fireball.cooldown_timer = 0x3C;
            fireball.current_y_px = fireball.base_y_px;
        } else {
            fireball.segment_index += 1;
        }
    }
}

impl NativeRuntime {
    pub(crate) fn step_murder_wall(&mut self) {
        let Some(wall) = self.murder_wall.as_mut() else {
            return;
        };
        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        if wall.reversed {
            wall.x_pos -= i64::from(wall.speed);
        } else {
            wall.x_pos += i64::from(wall.speed);
        }
    }

    /// Check if the murder wall overlaps the player, triggering instant death.
    pub(crate) fn murder_wall_kills_player(&self, slice: &OpeningSliceState) -> bool {
        let Some(wall) = &self.murder_wall else {
            return false;
        };
        let player_x = slice.player.x_pos as i64;
        let wall_x = wall.x_pos;
        if wall.reversed {
            // Wall moves right-to-left: kill if player's right side touches wall
            let wall_left = wall_x;
            player_x + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16) >= wall_left
        } else {
            // Wall moves left-to-right: kill if wall passes player's left side
            let wall_right = wall_x + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16);
            wall_right >= player_x
        }
    }

    pub(crate) fn step_weather(&mut self, slice: &OpeningSliceState) {
        // Hail only for Storm+Hail (weather 3), active after 35s of level time
        if self.weather.weather_type != 3 {
            return;
        }
        // Level timer counts down from 180s. 35s elapsed = 145s remaining.
        if self.level_timer.seconds > 145 {
            return;
        }

        // Toggle hail active/inactive every 600 frames ($258)
        if self.weather.hail_period_timer > 0 {
            self.weather.hail_period_timer -= 1;
        } else {
            self.weather.hail_active = !self.weather.hail_active;
            self.weather.hail_period_timer = 600;
        }

        if !self.weather.hail_active {
            return;
        }

        // Spawn hail every 30 frames during active period
        if self.weather.hail_spawn_timer > 0 {
            self.weather.hail_spawn_timer -= 1;
            return;
        }
        self.weather.hail_spawn_timer = 30;

        // Spawn a hail projectile above the camera, at a random-ish X
        let camera_x = slice.camera.x_pos >> 16;
        // Simple pseudo-random X offset based on frame index
        let x_offset = ((self.frame_index * 7 + 13) % 320) as u32;
        let spawn_x = camera_x.wrapping_add(x_offset);
        let spawn_y = (slice.camera.y_pos >> 16).saturating_sub(16);

        self.projectiles.push(EnemyProjectile {
            x_pos: spawn_x << 16,
            y_pos: spawn_y << 16,
            x_velocity: 0,
            y_velocity: 0x4_0000, // 4 px/frame downward
            lifetime: 120,
            friendly: false,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
    }

    pub(crate) fn init_weather(&mut self, level_id: LevelId) {
        let header = self.level_header(level_id);
        self.weather.weather_type = header.map(|h| h.weather_flags).unwrap_or(0);
        self.weather.hail_active = false;
        self.weather.hail_spawn_timer = 0;
        self.weather.hail_period_timer = 0;
        self.weather.lava_geysers.clear();
        self.weather.lava_fireballs.clear();
        if self.weather.weather_type == 1 {
            self.init_lava_geysers(level_id);
        }
    }

    /// Initialise lava geyser positions for lava weather levels.
    /// Uses hardcoded per-level position tables matching the original ASM data.
    pub(crate) fn init_lava_geysers(&mut self, level_id: LevelId) {
        // Per-level geyser positions: (x_px, y_px, height_px)
        let positions: &[(u16, u16, u16)] = match level_id.0 {
            // Under Skull Mountain 2 (USM2)
            0x06 => &[(0x180, 0x312, 0x60), (0x240, 0x312, 0x63)],
            // Black Pit
            0x45 => &[
                (0x290, 0x170, 0x20),
                (0x3D0, 0x170, 0x20),
                (0x510, 0x170, 0x40),
                (0x650, 0x170, 0x60),
                (0x790, 0x170, 0x80),
                (0x8D0, 0x170, 0xA0),
                (0xA10, 0x170, 0xC0),
                (0xB50, 0x170, 0xE0),
                (0xC90, 0x170, 0x100),
                (0xDD0, 0x170, 0x100),
                (0xF10, 0x170, 0x100),
                (0x1050, 0x170, 0x100),
                (0x1190, 0x170, 0x100),
            ],
            // Elsewhere 29
            0x3E => &[
                (0x090, 0x330, 0x260),
                (0x010, 0x1D0, 0x090),
                (0x110, 0x1D0, 0x090),
            ],
            _ => &[],
        };

        for &(x, y, h) in positions {
            self.weather.lava_geysers.push(LavaGeyser {
                x_px: x,
                y_px: y,
                height_px: h >> 4,
                phase: 0,
                timer: lava_geyser_initial_delay_frames_loc_14772(h),
                current_height: 0,
            });
        }

        if level_id.0 == 0x06 {
            self.weather.lava_fireballs.push(LavaFireball {
                x_px: 0x346,
                base_y_px: 0x312,
                current_y_px: 0x312,
                segment_index: 0,
                segment_frame: 0,
                cooldown_timer: 0,
            });
        }
    }

    /// Step all active lava geysers through their eruption cycle.
    pub(crate) fn step_lava_geysers(&mut self, slice: &OpeningSliceState) {
        if self.weather.weather_type != 1
            || (self.weather.lava_geysers.is_empty() && self.weather.lava_fireballs.is_empty())
        {
            return;
        }

        let mut should_damage = false;

        for geyser in &mut self.weather.lava_geysers {
            match geyser.phase {
                0 => {
                    // Waiting/dormant
                    if geyser.timer > 0 {
                        geyser.timer -= 1;
                    } else {
                        geyser.phase = 1;
                        geyser.current_height = 0;
                    }
                }
                1 => {
                    // Erupting upward
                    geyser.current_height += LAVA_GEYSER_ERUPT_SPEED;
                    if geyser.current_height >= geyser.height_px {
                        geyser.current_height = geyser.height_px;
                        geyser.phase = 2;
                        geyser.timer = LAVA_GEYSER_HOLD_FRAMES;
                    }
                }
                2 => {
                    // Holding at full height
                    if geyser.timer > 0 {
                        geyser.timer -= 1;
                    } else {
                        geyser.phase = 3;
                    }
                }
                3 => {
                    // Collapsing
                    if geyser.current_height > LAVA_GEYSER_ERUPT_SPEED {
                        geyser.current_height -= LAVA_GEYSER_ERUPT_SPEED;
                    } else {
                        geyser.current_height = 0;
                        geyser.phase = 0;
                        geyser.timer = LAVA_GEYSER_DORMANT_FRAMES;
                    }
                }
                _ => {}
            }

            if lava_geyser_hits_kid_sub_14A58(geyser, slice) {
                should_damage = true;
            }
        }

        for fireball in &mut self.weather.lava_fireballs {
            let segment = LAVA_FIREBALL_SCRIPT[usize::from(fireball.segment_index)];
            if fireball.cooldown_timer == 0
                && lava_fireball_hits_kid_sub_14AB0(fireball, slice, segment.sprite_kind)
            {
                should_damage = true;
            }
            advance_lava_fireball(fireball);
        }

        if should_damage && self.progression.invulnerable_frames == 0 {
            self.apply_damage();
        }
    }

    pub(crate) fn init_murder_wall(&mut self, level_id: LevelId, camera_x: u32) {
        let header = self.level_header(level_id);
        let flags = header.map(|h| h.murder_wall_flags).unwrap_or(0);
        if flags & 0b10 != 0 {
            let reversed = flags & 0b01 != 0;
            let max_speed = if level_id.0 == 0x08 {
                MURDER_WALL_SLOW_MAX_SPEED
            } else {
                MURDER_WALL_DEFAULT_MAX_SPEED
            };
            // ASM loc_140BE clears `MurderWall_speed` after the level-specific
            // max-speed branch, so native always starts the wall from rest.
            let initial_x = if camera_x == 0 {
                // ASM loc_140CE runs after `move.w (Camera_X_pos).w,d0` / `beq.s`,
                // so the zero-camera case subtracts `$30` from zero before the
                // shared `loc_140D2` store.
                -(i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16)
            } else {
                // ASM loc_140D2 receives the non-zero-camera `addi.w #$30,d0`
                // result and stores that shared branch value as Murderwall X.
                (i64::from(camera_x) << 16) + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16)
            };
            self.murder_wall = Some(MurderWallState {
                x_pos: initial_x,
                speed: 0,
                max_speed,
                reversed,
            });
        } else {
            self.murder_wall = None;
        }
    }
}
