use criterion::{Criterion, criterion_group, criterion_main};
use kid_core::{FrameInput, NativeRuntime};
use kid_data::{
    AssetManifest, BgscrollProfile, EnemyLayout, EnemyLayoutObject, LevelHeader, LevelRecord,
    LevelResources, PlatformLayout,
};
use std::path::PathBuf;

fn test_manifest() -> AssetManifest {
    AssetManifest {
        generated_bundle_version: kid_data::GENERATED_BUNDLE_VERSION,
        rom_sha1: String::from("bench"),
        upstream_commit: String::from("bench"),
        source_root: PathBuf::from("/tmp/bench"),
        output_root: PathBuf::from("/tmp/bench"),
        map_order: Vec::new(),
        levels: vec![LevelRecord {
            level_id: 0,
            title_text_id: Some(0x4A),
            display_name: Some(String::from("Blue Lake Woods 1")),
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
                object_count: 3,
                objects: vec![
                    EnemyLayoutObject {
                        object_type: 0x01,
                        flags: 0,
                        hit_points_plus_one: 1,
                        x: 0x0120,
                        y: 0x00E0,
                    },
                    EnemyLayoutObject {
                        object_type: 0x1C,
                        flags: 0,
                        hit_points_plus_one: 1,
                        x: 0x02A0,
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
                placements: Vec::new(),
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

/// Fast-forward a runtime to InGame state (past boot/title/intro).
fn runtime_in_game() -> NativeRuntime {
    let mut runtime = NativeRuntime::new(test_manifest());
    // Skip boot sequence: ~300 frames to get past sega/intro/title
    for _ in 0..300 {
        runtime.step_frame(FrameInput::default());
    }
    // Press start to begin
    runtime.step_frame(FrameInput {
        start: true,
        ..Default::default()
    });
    // Settle into gameplay
    for _ in 0..60 {
        runtime.step_frame(FrameInput::default());
    }
    runtime
}

fn bench_frame_step_idle(c: &mut Criterion) {
    let mut runtime = runtime_in_game();
    c.bench_function("frame_step_idle", |b| {
        b.iter(|| {
            runtime.step_frame(FrameInput::default());
        });
    });
}

fn bench_frame_step_walking(c: &mut Criterion) {
    let mut runtime = runtime_in_game();
    let input = FrameInput {
        right: true,
        ..Default::default()
    };
    c.bench_function("frame_step_walking", |b| {
        b.iter(|| {
            runtime.step_frame(input);
        });
    });
}

fn bench_level_load(c: &mut Criterion) {
    c.bench_function("level_load", |b| {
        b.iter(|| {
            NativeRuntime::new(test_manifest());
        });
    });
}

fn bench_boot_to_gameplay(c: &mut Criterion) {
    c.bench_function("boot_to_gameplay_360frames", |b| {
        b.iter(|| {
            let mut runtime = NativeRuntime::new(test_manifest());
            for _ in 0..360 {
                runtime.step_frame(FrameInput::default());
            }
        });
    });
}

criterion_group!(
    benches,
    bench_frame_step_idle,
    bench_frame_step_walking,
    bench_level_load,
    bench_boot_to_gameplay,
);
criterion_main!(benches);
