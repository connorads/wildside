use super::*;
use scenarios::{
    idle_level_scenario, prize_block_scenario, stomp_enemy_scenario, walk_right_scenario,
};

pub(crate) fn capture_frames(
    manifest: &AssetManifest,
    level_id: u8,
    frame_indices: &[usize],
    output_dir: &Path,
    filter: native_player::LayerFilter,
) -> Result<()> {
    use native_player::{
        NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, build_scene_filtered, load_session_assets,
        render_scene,
    };

    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let max_frame = frame_indices.iter().copied().max().unwrap_or(0);
    let scenario = idle_level_scenario(level_id, Some(max_frame + 10));

    let native_trace = kid_core::run_script_with_warp(
        manifest.clone(),
        &scenario.script,
        scenario.frame_count,
        scenario.warp_to_level,
    );

    let (level_visuals, block_grid) = load_session_assets(manifest, level_id)?;
    let w = NATIVE_FRAME_WIDTH as usize;
    let h = NATIVE_FRAME_HEIGHT as usize;

    for &frame_idx in frame_indices {
        let Some(frame) = native_trace.get(frame_idx) else {
            println!(
                "  frame {frame_idx}: not in trace (len={})",
                native_trace.len()
            );
            continue;
        };
        let scene = build_scene_filtered(
            frame,
            false,
            Some(&level_visuals),
            Some(&block_grid),
            filter,
        );
        let mut rgb = vec![0u8; w * h * 3];
        render_scene(&scene, &mut rgb);

        let filename = format!("level_{level_id:02x}_frame_{frame_idx:05}.png");
        let path = output_dir.join(&filename);
        image::save_buffer_with_format(
            &path,
            &rgb,
            w as u32,
            h as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        println!("  saved {}", path.display());
    }

    Ok(())
}

/// Layer names used for per-layer diagnosis.
const LAYER_NAMES: &[&str] = &[
    "background",
    "terrain",
    "block_overrides",
    "platforms",
    "enemies",
    "player",
    "effects",
    "hud",
];

/// Per-layer diagnosis: render each layer solo, count non-background pixels.
pub(crate) fn diagnose_frame(
    manifest: &AssetManifest,
    level_id: u8,
    frame_idx: usize,
    save_layers: bool,
    output_dir: &Path,
) -> Result<()> {
    use native_player::{
        LayerFilter, NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, build_scene_filtered,
        load_session_assets, render_scene,
    };

    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let scenario = idle_level_scenario(level_id, Some(frame_idx + 10));
    let native_trace = kid_core::run_script_with_warp(
        manifest.clone(),
        &scenario.script,
        scenario.frame_count,
        scenario.warp_to_level,
    );

    let frame = native_trace.get(frame_idx).ok_or_else(|| {
        anyhow!(
            "frame {frame_idx} not in trace (len={})",
            native_trace.len()
        )
    })?;

    let (level_visuals, block_grid) = load_session_assets(manifest, level_id)?;
    let w = NATIVE_FRAME_WIDTH as usize;
    let h = NATIVE_FRAME_HEIGHT as usize;
    let pixel_count = w * h;

    // Render full scene
    let full_scene = build_scene_filtered(
        frame,
        false,
        Some(&level_visuals),
        Some(&block_grid),
        LayerFilter::default(),
    );
    let mut full_rgb = vec![0u8; pixel_count * 3];
    render_scene(&full_scene, &mut full_rgb);
    let full_non_bg = count_non_background_pixels(&full_rgb, pixel_count);

    if save_layers {
        let full_path = output_dir.join(format!(
            "level_{level_id:02x}_frame_{frame_idx:05}_full.png"
        ));
        image::save_buffer_with_format(
            &full_path,
            &full_rgb,
            w as u32,
            h as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )?;
    }

    println!("Level {level_id:#04x}  frame {frame_idx}  diagnosis:");
    println!("{:<20} {:>8}  {:>10}", "LAYER", "PIXELS", "% OF FRAME");
    println!("{:-<44}", "");

    for &layer_name in LAYER_NAMES {
        let filter = LayerFilter::solo(layer_name);
        let scene = build_scene_filtered(
            frame,
            false,
            Some(&level_visuals),
            Some(&block_grid),
            filter,
        );
        let mut layer_rgb = vec![0u8; pixel_count * 3];
        render_scene(&scene, &mut layer_rgb);
        let layer_pixels = count_non_background_pixels(&layer_rgb, pixel_count);
        let pct = if pixel_count > 0 {
            layer_pixels as f64 / pixel_count as f64 * 100.0
        } else {
            0.0
        };
        println!("{layer_name:<20} {layer_pixels:>8}  {pct:>9.1}%");

        if save_layers {
            let path = output_dir.join(format!(
                "level_{level_id:02x}_frame_{frame_idx:05}_{layer_name}.png"
            ));
            image::save_buffer_with_format(
                &path,
                &layer_rgb,
                w as u32,
                h as u32,
                image::ColorType::Rgb8,
                image::ImageFormat::Png,
            )?;
        }
    }

    println!("{:-<44}", "");
    println!(
        "{:<20} {:>8}  {:>9.1}%",
        "FULL",
        full_non_bg,
        full_non_bg as f64 / pixel_count as f64 * 100.0
    );

    match &level_visuals.background {
        kid_render::BackgroundVisuals::Layered(background) => {
            let mut palette_line_histogram = std::collections::BTreeMap::new();
            for row in &background.rows {
                for tile in &row.tiles {
                    *palette_line_histogram
                        .entry(tile.palette_line)
                        .or_insert(0usize) += 1;
                }
            }
            let selector_preview: Vec<String> = background
                .rows
                .iter()
                .take(8)
                .map(|row| format!("{:#04x}", row.selector))
                .collect();
            println!();
            println!("Layered background:");
            println!(
                "  effect_start_row_selector={:?}",
                background.effect_start_row_selector
            );
            println!("  first_selectors=[{}]", selector_preview.join(", "));
            println!("  palette_lines={palette_line_histogram:?}");
        }
        kid_render::BackgroundVisuals::Chunked(background) => {
            println!();
            println!(
                "Chunked background: {} placements, {} chunks",
                background.placements.len(),
                background.chunks.len()
            );
        }
    }

    // State summary
    println!("\nState at frame {frame_idx}:");
    println!(
        "  game_mode={:?}  level={:#04x}  camera=({:#010x}, {:#010x})",
        frame.game_mode, frame.current_level.0, frame.camera_x_pos, frame.camera_y_pos
    );
    if let (Some(kx), Some(ky)) = (frame.kid_x_pos, frame.kid_y_pos) {
        println!(
            "  kid_pos=({}, {})  on_ground={}  hp={}  lives={}",
            kx >> 16,
            ky >> 16,
            frame.kid_on_ground.unwrap_or(false),
            frame.number_hitpoints,
            frame.number_lives,
        );
    }
    println!(
        "  objects={}  gfx_objects={}",
        frame.number_objects, frame.number_gfx_objects
    );

    Ok(())
}

/// Capture golden state snapshots for all specified levels.
/// Runs each level with idle, walk_right, and prize_block scenarios,
/// storing full NativeTraceFrame at every checkpoint as JSON.
pub(crate) fn golden_capture(
    repo_root: &Path,
    manifest: &AssetManifest,
    levels: &[u8],
) -> Result<()> {
    let golden_dir = repo_root.join(".cache/golden");
    fs::create_dir_all(&golden_dir)
        .with_context(|| format!("failed to create {}", golden_dir.display()))?;

    let source_hash = kid_core_source_hash(repo_root).unwrap_or_else(|_| "unknown".into());
    let mut captured = 0usize;

    for &level_id in levels {
        let level_dir = golden_dir.join(format!("{level_id:02x}"));
        fs::create_dir_all(&level_dir)?;

        let scenarios = golden_scenarios_for_level(level_id);
        for scenario in &scenarios {
            let trace = kid_core::run_script_with_warp(
                manifest.clone(),
                &scenario.script,
                scenario.frame_count,
                scenario.warp_to_level,
            );

            let checkpoint_frames: Vec<NativeTraceFrame> = scenario
                .checkpoints
                .iter()
                .filter_map(|cp| trace.get(cp.frame_index).cloned())
                .collect();

            let snapshot = GoldenSnapshot {
                level_id,
                scenario_slug: scenario.slug.clone(),
                kid_core_hash: source_hash.clone(),
                frames: checkpoint_frames,
            };

            let path = level_dir.join(format!("{}.json", scenario.slug));
            fs::write(&path, serde_json::to_vec_pretty(&snapshot)?)
                .with_context(|| format!("failed to write {}", path.display()))?;
            captured += 1;
        }
        println!(
            "golden capture {level_id:#04x}: {n} scenarios",
            n = scenarios.len()
        );
    }

    println!(
        "\ngolden capture complete: {captured} snapshots across {} levels",
        levels.len()
    );
    Ok(())
}

/// Diff current native state against golden snapshots for all specified levels.
/// Returns a report with per-level results.
pub(crate) fn golden_diff(
    repo_root: &Path,
    manifest: &AssetManifest,
    levels: &[u8],
) -> Result<SweepReport> {
    let golden_dir = repo_root.join(".cache/golden");
    if !golden_dir.exists() {
        bail!("no golden snapshots found — run --golden-capture first");
    }

    let timestamp = chrono_timestamp();
    let mut results = Vec::with_capacity(levels.len());
    let mut total_diffs = 0usize;

    for &level_id in levels {
        let level_dir = golden_dir.join(format!("{level_id:02x}"));
        if !level_dir.exists() {
            println!("golden diff {level_id:#04x}: no golden data, skipping");
            continue;
        }

        let scenarios = golden_scenarios_for_level(level_id);
        let mut level_diffs = Vec::new();

        for scenario in &scenarios {
            let golden_path = level_dir.join(format!("{}.json", scenario.slug));
            if !golden_path.exists() {
                continue;
            }

            let golden_data = fs::read(&golden_path)
                .with_context(|| format!("failed to read {}", golden_path.display()))?;
            let golden: GoldenSnapshot = serde_json::from_slice(&golden_data)
                .with_context(|| format!("failed to parse {}", golden_path.display()))?;

            let trace = kid_core::run_script_with_warp(
                manifest.clone(),
                &scenario.script,
                scenario.frame_count,
                scenario.warp_to_level,
            );

            let current_frames: Vec<NativeTraceFrame> = scenario
                .checkpoints
                .iter()
                .filter_map(|cp| trace.get(cp.frame_index).cloned())
                .collect();

            let diffs = diff_golden_frames(
                &scenario.slug,
                &golden.frames,
                &current_frames,
                &scenario.checkpoints,
            );
            level_diffs.extend(diffs);
        }

        let passed = level_diffs.is_empty();
        if passed {
            println!("golden diff {level_id:#04x}: PASS");
        } else {
            total_diffs += level_diffs.len();
            println!(
                "golden diff {level_id:#04x}: FAIL ({} field changes)",
                level_diffs.len()
            );
            for d in &level_diffs {
                println!(
                    "  {}/{}: {} -> {}",
                    d.checkpoint, d.field, d.old_value, d.new_value
                );
            }
        }

        results.push(LevelSweepResult {
            level_id,
            slug: format!("golden_{level_id:02x}"),
            tier: SweepTier::State,
            passed,
            state_diffs: level_diffs
                .iter()
                .map(|d| StateDiff {
                    checkpoint: d.checkpoint.clone(),
                    field: d.field.clone(),
                    native_value: d.new_value.clone(),
                    oracle_value: d.old_value.clone(),
                    category: DivergenceCategory::Authoritative,
                })
                .collect(),
            mae: None,
            pixel_mismatches: None,
            diff_images: Vec::new(),
            error: None,
            divergence_summary: if passed {
                DivergenceSummary::default()
            } else {
                divergence_summary_with(DivergenceCategory::Authoritative)
            },
        });
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let divergence_summary = summarise_results(&results);
    let field_summary = summarise_field_diffs(&results);
    println!(
        "\ngolden diff complete: {passed} passed, {failed} failed, {total_diffs} total field changes"
    );

    Ok(SweepReport {
        timestamp,
        kid_core_hash: kid_core_source_hash(repo_root).ok(),
        levels: results,
        passed,
        failed,
        errors: 0,
        divergence_summary,
        field_summary,
        release_gate_focus: Vec::new(),
    })
}

/// Build the standard set of scenarios for golden snapshot capture/diff.
pub(super) fn golden_scenarios_for_level(level_id: u8) -> Vec<DynamicScenario> {
    vec![
        idle_level_scenario(level_id, None),
        walk_right_scenario(level_id, 300),
        prize_block_scenario(level_id),
        stomp_enemy_scenario(level_id, 360),
    ]
}

/// Diff two sets of golden frames field-by-field, reporting any changes.
pub(super) fn diff_golden_frames(
    scenario_slug: &str,
    old_frames: &[NativeTraceFrame],
    new_frames: &[NativeTraceFrame],
    checkpoints: &[DynamicCheckpoint],
) -> Vec<GoldenDiff> {
    let mut diffs = Vec::new();

    for (i, (old, new)) in old_frames.iter().zip(new_frames.iter()).enumerate() {
        let cp_label = checkpoints.get(i).map_or_else(
            || format!("{scenario_slug}/checkpoint_{i}"),
            |cp| format!("{}/{}", scenario_slug, cp.label),
        );

        macro_rules! check_field {
            ($field:ident) => {
                if old.$field != new.$field {
                    diffs.push(GoldenDiff {
                        checkpoint: cp_label.clone(),
                        field: stringify!($field).to_string(),
                        old_value: format!("{:?}", old.$field),
                        new_value: format!("{:?}", new.$field),
                    });
                }
            };
        }

        check_field!(game_mode);
        check_field!(game_mode_value);
        check_field!(current_level);
        check_field!(player_1_level_id);
        check_field!(number_lives);
        check_field!(number_continues);
        check_field!(number_hitpoints);
        check_field!(current_helmet_id);
        check_field!(paused);
        check_field!(kid_x_pos);
        check_field!(kid_y_pos);
        check_field!(kid_x_velocity);
        check_field!(kid_y_velocity);
        check_field!(kid_facing);
        check_field!(kid_on_ground);
        check_field!(camera_x_pos);
        check_field!(camera_y_pos);
        check_field!(number_objects);
        check_field!(number_gfx_objects);
        check_field!(level_complete);
        check_field!(game_over);
        check_field!(game_complete);
        check_field!(invulnerable_frames);
        check_field!(diamond_power_active);
        check_field!(diamond_count);
        check_field!(score);
        check_field!(level_timer_seconds);
        check_field!(murder_wall_x);
        check_field!(active_enemy_object_count);
        check_field!(active_platform_object_count);
        check_field!(active_projectiles);
        check_field!(stomped_this_frame);
        check_field!(blocks_hit_this_frame);
        check_field!(two_player);
        check_field!(options_cursor);
    }

    diffs
}

/// Count pixels that differ from the background (first pixel's colour).
fn count_non_background_pixels(rgb: &[u8], pixel_count: usize) -> usize {
    if rgb.len() < 3 {
        return 0;
    }
    let bg = [rgb[0], rgb[1], rgb[2]];
    (0..pixel_count)
        .filter(|&i| {
            let offset = i * 3;
            rgb[offset] != bg[0] || rgb[offset + 1] != bg[1] || rgb[offset + 2] != bg[2]
        })
        .count()
}
