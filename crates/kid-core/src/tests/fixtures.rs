use super::*;

#[allow(dead_code)] // used by ignored loses_life_and_respawns_after_opening_hazard
pub(super) fn expected_level0_respawn_pose(runtime: &NativeRuntime) -> (u32, u32, u32, u32) {
    let slice = runtime.level_respawn_slice_with_floor_snap(LevelId(0));
    (
        slice.camera.x_pos,
        slice.camera.y_pos,
        slice.player.x_pos,
        slice.player.y_pos,
    )
}
pub(super) fn collision_size_entry_sub_21f8(
    left_offset_px: i16,
    width_px: i16,
    top_offset_px: i16,
    height_px: i16,
) -> CollisionSizeEntrySub21F8 {
    CollisionSizeEntrySub21F8 {
        left_offset_px,
        width_px,
        top_offset_px,
        height_px,
    }
}

pub(super) fn support_overlap_candidate_sub_9832(
    x_anchor_px: i16,
    y_anchor_px: i16,
    mirrored: bool,
    active: bool,
    collision_sizes: Vec<CollisionSizeEntrySub21F8>,
) -> SupportOverlapCandidateSub9832 {
    SupportOverlapCandidateSub9832 {
        x_anchor_px,
        y_anchor_px,
        mirrored,
        active,
        collision_sizes,
    }
}
pub(super) fn manifest() -> AssetManifest {
    AssetManifest {
        generated_bundle_version: kid_data::GENERATED_BUNDLE_VERSION,
        rom_sha1: String::from("abc123"),
        upstream_commit: String::from("deadbeef"),
        source_root: PathBuf::from("/tmp/source"),
        output_root: PathBuf::from("/tmp/output"),
        map_order: Vec::new(),
        levels: vec![LevelRecord {
            level_id: 0,
            title_text_id: Some(0x4A),
            display_name: Some(String::from("Blue Lake Woods 1")),
            title_card: None,
            resource_paths: vec![String::from("level/header/00.bin")],
            resources: LevelResources::default(),
            header_metadata: Some(LevelHeader {
                width_screens: 6,
                height_screens: 4,
                height_flag_bits: 0,
                level_theme: 8,
                murder_wall_flags: 0,
                background_theme: 8,
                weather_flags: 1,
                player_start_x: 0x0750,
                player_start_y: 0x00E0,
                flag_x: 0x0750,
                flag_y: 0x0300,
            }),
            enemy_layout_metadata: Some(EnemyLayout {
                axis_uses_y_position: false,
                unknown_prefix: 0,
                enemy_type_slots: vec![0xFFFF, 0x0001, 0x001C],
                unknown_word: 0x7D00,
                header_byte: 0x80,
                object_count: 22,
                objects: vec![
                    kid_data::EnemyLayoutObject {
                        object_type: 0x01,
                        flags: 0,
                        hit_points_plus_one: 1,
                        x: 0x0120,
                        y: 0x00E0,
                    },
                    kid_data::EnemyLayoutObject {
                        object_type: 0x1C,
                        flags: 0,
                        hit_points_plus_one: 1,
                        x: 0x02A0,
                        y: 0x0100,
                    },
                    kid_data::EnemyLayoutObject {
                        object_type: 0x1C,
                        flags: 0,
                        hit_points_plus_one: 1,
                        x: 0x0900,
                        y: 0x0100,
                    },
                ],
                segments: Vec::new(),
            }),
            bgscroll_metadata: Some(BgscrollProfile {
                layer_speeds: vec![0x10, 0x10, 0x40, 0x60],
                has_terminator: true,
            }),
            platform_metadata: Some(PlatformLayout {
                placements: vec![
                    PlatformPlacement {
                        x: 0x0100,
                        y: 0x00E0,
                        buffer_left: 0,
                        buffer_right: 0,
                        buffer_top: 0,
                        buffer_bottom: 0,
                        kind: 0x00,
                        subtype: 0x00,
                        h_variant: 0,
                        v_variant: 0,
                        script: PlatformScriptRef {
                            raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                            relative_offset: None,
                        },
                    },
                    PlatformPlacement {
                        x: 0x0900,
                        y: 0x0100,
                        buffer_left: 0,
                        buffer_right: 0,
                        buffer_top: 0,
                        buffer_bottom: 0,
                        kind: 0x01,
                        subtype: 0x02,
                        h_variant: 0,
                        v_variant: 0,
                        script: PlatformScriptRef {
                            raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                            relative_offset: None,
                        },
                    },
                ],
            }),
        }],
        constants: Vec::new(),
        ram_variables: Vec::new(),
        enemy_sources: Vec::new(),
        theme_assets: Vec::new(),
        scene_assets: Vec::new(),
        animation_assets: Vec::new(),
        sound_assets: Vec::new(),
        ingame_assets: Vec::new(),
        decoded_levels: vec![],
    }
}
pub(super) fn start_level(runtime: &mut NativeRuntime) {
    if runtime.block_grid().is_none() {
        runtime.set_block_grid(grid_with_floor_at_row(17));
    }
    runtime.enter_level_playtest(LevelId(0));
}
pub(super) fn advance_past_level_entry_damage_grace(runtime: &mut NativeRuntime) {
    let mut saw_spawn_invulnerability = runtime.progression.invulnerable_frames > 0;
    for _ in 0..128 {
        runtime.step_frame(FrameInput::default());
        saw_spawn_invulnerability |= runtime.progression.invulnerable_frames > 0;
        if !saw_spawn_invulnerability || runtime.progression.invulnerable_frames == 0 {
            return;
        }
    }
    panic!("did not reach a stable post-entry gameplay state within the expected frame budget");
}

pub(super) fn manifest_with_levels(count: u8) -> AssetManifest {
    let mut expanded = manifest();
    let base = expanded.levels[0].clone();
    expanded.levels = (0..count)
        .map(|level_id| {
            let mut level = base.clone();
            level.level_id = level_id;
            level.display_name = Some(format!("Level {level_id}"));
            if let Some(header) = level.header_metadata.as_mut() {
                header.player_start_x = header.player_start_x.saturating_add(u16::from(level_id));
                header.flag_x = header.flag_x.saturating_add(u16::from(level_id));
            }
            level
        })
        .collect();
    expanded.map_order = (0..count).collect();
    expanded
}

pub(super) fn unique_temp_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("kc-{label}-{}-{nanos}", process::id()))
}

/// Helper: build a block grid with solid floor at a given row everywhere.
pub(super) fn grid_with_floor_at_row(floor_row: u16) -> terrain::BlockGrid {
    let width_cells = 6u16 * 20;
    let height_cells = 4u16 * 14;
    let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
    for col in 0..width_cells {
        let offset = (usize::from(floor_row) * usize::from(width_cells) + usize::from(col)) * 2;
        data[offset] = 0x60; // collision=6 (Solid)
    }
    terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
}

// --- US-004: Enemy-player contact damage ---
// Tests use MiniHoppingSkull (not Diamond, which is now collected on touch).

/// Manifest with a static skull near spawn for contact damage tests.
pub(super) fn manifest_with_static_skull() -> AssetManifest {
    let mut m = manifest();
    if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
        // Replace first object with a skull at the Diamond's old position.
        // Keep enemy_type_slots so slot resolution still works.
        enemy.objects[0] = kid_data::EnemyLayoutObject {
            object_type: 0x1C, // MiniHoppingSkull
            flags: 0,
            hit_points_plus_one: 1,
            x: 0x0120,
            y: 0x00E0,
        };
    }
    m
}
// -----------------------------------------------------------------------
// MiniHoppingSkull hopper behaviour
// -----------------------------------------------------------------------

/// Build a grid with a floor platform from col start..end at the given row.
pub(super) fn grid_with_platform(
    floor_row: u16,
    floor_start_col: u16,
    floor_end_col: u16,
) -> terrain::BlockGrid {
    let width_cells = 6u16 * 20; // 120 cells
    let height_cells = 4u16 * 14; // 56 cells
    let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
    let set_solid = |data: &mut Vec<u8>, col: u16, row: u16| {
        let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
        data[offset] = 0x60; // collision=6 (Solid)
    };
    // Floor platform
    for col in floor_start_col..floor_end_col {
        set_solid(&mut data, col, floor_row);
    }
    terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
}

/// Build a grid with a floor platform and a wall blocking movement.
pub(super) fn grid_with_wall(
    floor_row: u16,
    floor_start_col: u16,
    floor_end_col: u16,
    wall_col: u16,
) -> terrain::BlockGrid {
    let width_cells = 6u16 * 20;
    let height_cells = 4u16 * 14;
    let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
    let set_solid = |data: &mut Vec<u8>, col: u16, row: u16| {
        let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
        data[offset] = 0x60;
    };
    for col in floor_start_col..floor_end_col {
        set_solid(&mut data, col, floor_row);
    }
    // Wall: solid cells above the floor at wall_col
    for row in floor_row.saturating_sub(3)..floor_row {
        set_solid(&mut data, wall_col, row);
    }
    terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
}

/// Manifest with a single MiniHoppingSkull close to the player spawn
/// so it activates immediately without needing to scroll the camera.
/// Build a block grid for block interaction tests:
///
///   col: 0  1  2  3  4  5
/// row 2: .  .  P  M  X  .    (P=prize, M=metal, X=smash at row 2 = y 32px)
/// row 3: S  S  S  S  S  S    (solid floor at row 3 = y 48px)
///
/// Grid is 6 cols × 5 rows (96×80 px). Player spawns on the floor at
/// row 3 and jumps upward into blocks at row 2.
pub(super) fn grid_with_blocks() -> terrain::BlockGrid {
    let w = 6u16;
    let h = 5u16;
    let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
    let set_cell = |data: &mut Vec<u8>, col: usize, row: usize, hi: u8| {
        let offset = (row * usize::from(w) + col) * 2;
        data[offset] = hi;
        data[offset + 1] = 0x01; // arbitrary skin
    };
    // Row 2: prize at col 2, metal at col 3, smash at col 4
    set_cell(&mut data, 2, 2, 0xE1); // has_block + collision=6(Solid) + block_type=1(Prize)
    set_cell(&mut data, 3, 2, 0xE5); // has_block + collision=6(Solid) + block_type=5(Metal)
    set_cell(&mut data, 4, 2, 0xE0); // has_block + collision=6(Solid) + block_type=0(Smash)
    // Row 3: solid floor
    for col in 0..6 {
        set_cell(&mut data, col, 3, 0x60); // collision=6 (Solid)
    }
    terrain::BlockGrid::from_level_layout(&data, w, h)
}

// --- US-011: Stomp-kill enemies ---

/// Set up a stomp scenario around the first active test enemy.
/// Build a small grid (width_cells x height_cells) where specific cells are
/// solid and everything else is empty. Each entry in `solid_cells` is (col, row).
pub(super) fn grid_with_solid_cells(
    width_cells: u16,
    height_cells: u16,
    solid_cells: &[(u16, u16)],
) -> terrain::BlockGrid {
    let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
    for &(col, row) in solid_cells {
        let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
        data[offset] = 0x60; // collision=6 (Solid)
    }
    terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
}

pub(super) fn make_pacer(family: EnemyFamily, hp: u8, x_px: u32, y_px: u32) -> ActiveEnemy {
    ActiveEnemy {
        segment_index: 0,
        layout_index: 0,
        enemy_level: 0,
        family,
        x_pos: x_px << 16,
        y_pos: y_px << 16,
        x_velocity: 0,
        y_velocity: 0,
        target_x_velocity: 0,
        target_y_velocity: 0,
        x_acceleration: 0,
        y_acceleration: 0,
        facing: Direction::Left,
        on_ground: true,
        hit_points: hp,
        grounded_frames: 0,
        damage_flash_frames: 0,
        palette_variant: 0,
        wait_timer: 0,
        dragon_phase: DragonFlyingState::None,
        dragon_phase_frames: 0,
        dragon_script_variant: 0,
        dragon_script_cmd: 0,
        dragon_anchor_y: 0,
        dragon_descent_index: 0,
        dragon_descent_frames: 0,
        ninja_state: NinjaState::Stealth,
        hand_state: HandState::Walking,
        lion_state: LionState::Walking,
        robot_state: RobotState::Walking,
        orca_state: OrcaState::Walking,
        wall_pinned: false,
        drips_attached: false,
        drips_surface_table_index: 0,
        tar_monster_state: TarMonsterState::default(),
        spinning_twins_phase: SpinningTwinsPhase::default(),
        spinning_twins_orbit_index: 0,
        ufo_beam_segments: 0,
        ufo_beam_interval: 0,
        boss_head_index: enemies::BOSS_HEAD_NONE,
        scorpion_tracking_lock: false,
        flight_double_acceleration: false,
        flight_double_terminal_speed: false,
        walk_speed_override: 0,
        crab_frozen: false,
        crab_berzerker_knockback: false,
        shared_walker_state: spawn_shared_walker_state(family),
        shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(family),
        shared_walker_turnaround_counter: 0,
        shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(family),
        shared_walker_debug_branch: None,
        shared_walker_dx_to_kid_px: None,
        shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
    }
}

// --- Platform collision ---

pub(super) fn make_test_platform(x: u16, y: u16, h_variant: u8, v_variant: u8) -> ActivePlatform {
    ActivePlatform {
        layout_index: None,
        kind: 0,
        subtype: 0,
        x_pos: u32::from(x) << 16,
        y_pos: u32::from(y) << 16,
        width_px: u16::from(h_variant) * 16 + 15,
        height_px: u16::from(v_variant) * 16 + 15,
        x_velocity: 0,
        y_velocity: 0,
        script: None,
        special: None,
    }
}

pub(super) fn make_airborne_slice(x: u16, y: u16) -> OpeningSliceState {
    OpeningSliceState {
        player: PlayerState {
            x_pos: u32::from(x) << 16,
            y_pos: u32::from(y) << 16,
            x_velocity: 0,
            y_velocity: 0x1_0000, // falling
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        },
        camera: CameraState { x_pos: 0, y_pos: 0 },
        terrain: TerrainState {
            floor_y_pos: 0,
            floor_terrain_type: TerrainType::Normal,
        },
        level_bounds: LevelBounds {
            width_pixels: 320,
            height_pixels: 224,
        },
        objects: ObjectSystemState {
            platform_loader: PlatformLoaderState::default(),
        },
        walk_visual_accumulator: 0,
        prize_block_hit: None,
        lift_block_hit: None,
        drill_projectile_hit: None,
        metal_block_hit: None,
        rubber_block_hit: None,
        smash_block_hit: None,
        shooter_block_hit: None,
        teleporter_contact: None,
        telepad_timer: 0x5A,
        ceiling_prize_clear: None,
        pre_gravity_y_vel: 0,
        physics_trace: None,
        wall_cling_release_probe: None,
        kid_flip_follow_on_stages: None,
        kid_jump_gravity_probe: None,
    }
}

// --- Evanescent blocks ---

/// Build a grid with an evanescent block at (col, row) and solid floor
/// one row below. The skin byte controls the initial timer delay.
pub(super) fn grid_with_evanescent(col: u16, row: u16, skin: u8) -> terrain::BlockGrid {
    let w = 6u16;
    let h = (row + 3).max(6);
    let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
    let set = |data: &mut Vec<u8>, c: u16, r: u16, hi: u8, lo: u8| {
        let off = (usize::from(r) * usize::from(w) + usize::from(c)) * 2;
        data[off] = hi;
        data[off + 1] = lo;
    };
    // Evanescent block: has_block=1, collision=6(Solid), block_type=0xA
    // hi byte: 1_110_1010 = 0xEA
    set(&mut data, col, row, 0xEA, skin);
    // Solid floor one row below the evanescent block
    for c in 0..w {
        set(&mut data, c, row + 1, 0x60, 0);
    }
    terrain::BlockGrid::from_level_layout(&data, w, h)
}

pub(super) fn runtime_with_evanescent_grid(grid: terrain::BlockGrid) -> NativeRuntime {
    let manifest = manifest();
    let mut runtime = NativeRuntime::new(manifest);
    runtime.set_block_grid(grid);
    runtime
}

pub(super) fn grid_with_shooter_blocks(blocks: &[(u16, u16, u8)]) -> terrain::BlockGrid {
    let w = 40u16;
    let h = 8u16;
    let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
    let set = |data: &mut Vec<u8>, c: u16, r: u16, hi: u8, lo: u8| {
        let off = (usize::from(r) * usize::from(w) + usize::from(c)) * 2;
        data[off] = hi;
        data[off + 1] = lo;
    };
    for &(col, row, skin) in blocks {
        // Shooter block: has_block=1, collision=6 (Solid), block_type=8.
        set(&mut data, col, row, 0xE8, skin);
    }
    terrain::BlockGrid::from_level_layout(&data, w, h)
}

pub(super) fn shooter_test_slice(camera_x_px: u32, camera_y_px: u32) -> OpeningSliceState {
    OpeningSliceState {
        player: PlayerState {
            x_pos: 0,
            y_pos: 0,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        },
        camera: CameraState {
            x_pos: camera_x_px << 16,
            y_pos: camera_y_px << 16,
        },
        terrain: TerrainState {
            floor_y_pos: 0,
            floor_terrain_type: TerrainType::Normal,
        },
        level_bounds: LevelBounds {
            width_pixels: 640,
            height_pixels: 128,
        },
        objects: ObjectSystemState {
            platform_loader: PlatformLoaderState::default(),
        },
        walk_visual_accumulator: 0,
        prize_block_hit: None,
        lift_block_hit: None,
        drill_projectile_hit: None,
        metal_block_hit: None,
        rubber_block_hit: None,
        smash_block_hit: None,
        shooter_block_hit: None,
        teleporter_contact: None,
        telepad_timer: 0x5A,
        ceiling_prize_clear: None,
        pre_gravity_y_vel: 0,
        physics_trace: None,
        wall_cling_release_probe: None,
        kid_flip_follow_on_stages: None,
        kid_jump_gravity_probe: None,
    }
}

// --- Skycutter flipboard tests ---

pub(super) fn skycutter_runtime() -> NativeRuntime {
    let mut rt = NativeRuntime::new(manifest());
    rt.progression.current_form = PlayerForm::Skycutter;
    rt
}

pub(super) fn airborne_slice() -> OpeningSliceState {
    let bounds = LevelBounds {
        width_pixels: 4096,
        height_pixels: 512,
    };
    let mut slice = OpeningSliceState::new(bounds);
    slice.player.on_ground = false;
    slice.player.y_velocity = 0x1_0000; // falling
    slice.player.x_pos = 0x0080_0000;
    slice.player.y_pos = 0x0100_0000;
    slice
}

// --- Cyclone flight tests ---

pub(super) fn cyclone_runtime() -> NativeRuntime {
    let mut rt = NativeRuntime::new(manifest());
    rt.progression.current_form = PlayerForm::Cyclone;
    rt
}

pub(super) fn iron_knight_runtime_with_wall() -> NativeRuntime {
    let mut rt = NativeRuntime::new(manifest());
    rt.progression.current_form = PlayerForm::IronKnight;
    rt.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();
    // Wall at col 10, floor at row 3, from col 0..20
    rt.block_grid = Some(grid_with_wall(3, 0, 20, 10));
    rt
}

// --- Combat ability test helpers ---

pub(super) fn test_enemy(family: EnemyFamily, x: u32, y: u32, hp: u8) -> ActiveEnemy {
    ActiveEnemy {
        segment_index: 0,
        layout_index: 0,
        enemy_level: 0,
        family,
        x_pos: x,
        y_pos: y,
        x_velocity: 0,
        y_velocity: 0,
        target_x_velocity: 0,
        target_y_velocity: 0,
        x_acceleration: 0,
        y_acceleration: 0,
        facing: Direction::Left,
        on_ground: false,
        hit_points: hp,
        grounded_frames: 0,
        damage_flash_frames: 0,
        palette_variant: 0,
        wait_timer: 0,
        dragon_phase: DragonFlyingState::None,
        dragon_phase_frames: 0,
        dragon_script_variant: 0,
        dragon_script_cmd: 0,
        dragon_anchor_y: 0,
        dragon_descent_index: 0,
        dragon_descent_frames: 0,
        ninja_state: NinjaState::Stealth,
        hand_state: HandState::Walking,
        lion_state: LionState::Walking,
        robot_state: RobotState::Walking,
        orca_state: OrcaState::Walking,
        wall_pinned: false,
        drips_attached: false,
        drips_surface_table_index: 0,
        tar_monster_state: TarMonsterState::default(),
        spinning_twins_phase: SpinningTwinsPhase::default(),
        spinning_twins_orbit_index: 0,
        ufo_beam_segments: 0,
        ufo_beam_interval: 0,
        boss_head_index: enemies::BOSS_HEAD_NONE,
        scorpion_tracking_lock: false,
        flight_double_acceleration: false,
        flight_double_terminal_speed: false,
        walk_speed_override: 0,
        crab_frozen: false,
        crab_berzerker_knockback: false,
        shared_walker_state: spawn_shared_walker_state(family),
        shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(family),
        shared_walker_turnaround_counter: 0,
        shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(family),
        shared_walker_debug_branch: None,
        shared_walker_dx_to_kid_px: None,
        shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
    }
}

/// Create a Dragon Flying enemy with the given script variant, positioned at (x, y).
/// Initialises the script interpreter to the first command of the selected variant.
pub(super) fn combat_slice() -> OpeningSliceState {
    let bounds = LevelBounds {
        width_pixels: 4096,
        height_pixels: 512,
    };
    OpeningSliceState {
        player: {
            let mut p = PlayerState::title_pose();
            p.x_pos = 0x0080_0000;
            p.y_pos = 0x0100_0000;
            p.facing = Direction::Right;
            p
        },
        ..OpeningSliceState::new(bounds)
    }
}

// -------------------------------------------------------------------
// insta snapshot tests — detect NativeTraceFrame state drift (#028)
// -------------------------------------------------------------------

pub(super) fn cached_any_percent_published_runtime_and_script() -> (NativeRuntime, Vec<u8>) {
    cached_tas_runtime_and_script("any-percent-published")
}

// `cargo mutants` copies the source tree into `/tmp/cargo-mutants-…/` and runs
// tests there. `.cache/generated/` is gitignored so it is excluded from that
// copy, which makes `env!("CARGO_MANIFEST_DIR")` resolve to a tempdir that has
// no manifest. Allow the mutants invocation to point us back at the real repo
// via `KID_REPO_ROOT`.
pub(super) fn repo_root() -> PathBuf {
    if let Ok(override_root) = env::var("KID_REPO_ROOT")
        && !override_root.is_empty()
    {
        return PathBuf::from(override_root)
            .canonicalize()
            .expect("KID_REPO_ROOT canonicalize");
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

pub(super) fn cached_generated_runtime() -> NativeRuntime {
    let manifest_path = repo_root()
        .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
    let manifest = kid_data::load_manifest(&manifest_path).expect("load cached manifest");
    let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
    runtime.set_trace_physics(false);
    runtime
}

pub(super) fn cached_tas_runtime_and_script(scenario_slug: &str) -> (NativeRuntime, Vec<u8>) {
    let scenario_path = repo_root().join(format!("tests/data/tas/{scenario_slug}/scenario.json"));
    let scenario_raw = fs::read_to_string(scenario_path).expect("read TAS scenario");
    let scenario_json: serde_json::Value =
        serde_json::from_str(&scenario_raw).expect("parse TAS scenario");
    let script: Vec<u8> = scenario_json["script"]
        .as_array()
        .expect("script array")
        .iter()
        .map(|value| value.as_u64().expect("script byte") as u8)
        .collect();

    (cached_generated_runtime(), script)
}

pub(super) fn death_respawn_level_1_script() -> Vec<u8> {
    let mut script = vec![0u8; 9_001 + 1];
    for &frame in &BOOT_TO_LEVEL_START_FRAMES {
        script[frame] |= 0x80;
    }
    for slot in &mut script[600..=899] {
        *slot |= 1 << 3;
    }
    for slot in &mut script[900..=1_350] {
        *slot |= (1 << 3) | (1 << 6);
    }
    for frame in (940..=1_350).step_by(24) {
        let jump_end = 1_350.min(frame + 3);
        for slot in &mut script[frame..=jump_end] {
            *slot |= 1 << 4;
        }
    }
    for slot in &mut script[1_351..=3_000] {
        *slot |= 1 << 3;
    }
    for frame in (1_600..=3_000).step_by(28) {
        let jump_end = 3_000.min(frame + 2);
        for slot in &mut script[frame..=jump_end] {
            *slot |= 1 << 4;
        }
    }
    script
}

// ---------------------------------------------------------------------------
// Block subsystem: targeted mutation-killing tests
// ---------------------------------------------------------------------------

/// Helper: build a minimal grid with specific cells set.
pub(super) fn grid_with_cells(w: u16, h: u16, cells: &[(u16, u16, u8, u8)]) -> terrain::BlockGrid {
    let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
    for &(col, row, hi, lo) in cells {
        let off = (usize::from(row) * usize::from(w) + usize::from(col)) * 2;
        data[off] = hi;
        data[off + 1] = lo;
    }
    terrain::BlockGrid::from_level_layout(&data, w, h)
}

pub(super) fn default_bounds() -> LevelBounds {
    LevelBounds {
        width_pixels: 320,
        height_pixels: 256,
    }
}

// --- Projectile subsystem mutation tests ---

pub(super) fn make_projectile(x: u16, y: u16) -> EnemyProjectile {
    EnemyProjectile {
        x_pos: u32::from(x) << 16,
        y_pos: u32::from(y) << 16,
        x_velocity: 0,
        y_velocity: 0,
        lifetime: 100,
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        ..Default::default()
    }
}

/// Build a 10×10 grid (160×160 px) with solid blocks at specified cells.
/// Uses 0xE0 (has_block=1, collision=Solid) so wall_bounce detects them.
pub(super) fn bounce_grid(cells: &[(u16, u16)]) -> terrain::BlockGrid {
    let w = 10u16;
    let h = 10u16;
    let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
    for &(col, row) in cells {
        let off = (usize::from(row) * usize::from(w) + usize::from(col)) * 2;
        data[off] = 0xE0; // has_block=1, collision=Solid(6)
    }
    terrain::BlockGrid::from_level_layout(&data, w, h)
}
