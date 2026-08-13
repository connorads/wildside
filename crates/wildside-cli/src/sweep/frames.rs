use super::*;
use scenarios::{
    DynamicScenario, idle_level_scenario, prize_block_scenario, stomp_enemy_scenario,
    walk_right_scenario,
};
use wildside_core::{FrameInput, LevelId, NativeRuntime, PlayerForm};

#[cfg(test)]
fn capture_frame_block_grid<'a>(
    runtime: &'a NativeRuntime,
    loaded_block_grid: &'a wildside_core::terrain::BlockGrid,
) -> &'a wildside_core::terrain::BlockGrid {
    runtime.block_grid().unwrap_or(loaded_block_grid)
}

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

    let native_trace = wildside_core::run_script_with_warp(
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

/// Render the NATIVE engine's frames at specific indices while replaying a
/// recording's `DynamicScenario` (per-frame button masks + warp/settle/probe
/// state), saving each as a PNG.
///
/// This complements `capture_frames` (which is idle-only) and `render-frame`
/// (which renders the ORACLE ROM via the emulator). It is the native side of a
/// native-vs-oracle visual comparison on a scripted route — e.g. the #211
/// slope-walk render probe, where the gameplay-state facing already matches the
/// oracle and the remaining complaint is renderer sprite-frame selection.
pub(crate) fn capture_scenario_frames(
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
    frame_indices: &[usize],
    output_dir: &Path,
    filter: native_player::LayerFilter,
    force_scene: bool,
) -> Result<()> {
    use native_player::{
        NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, build_scene_filtered, load_session_assets,
        render_scene, render_trace_with_vdp_latched_state_and_resident, render_vdp_trace_overlay,
        vdp_snapshot_from_core,
    };

    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    // The warp target drives which level's visuals/block grid to load; default
    // to level 0 (the boot level) when the scenario does not warp.
    let mut current_visual_level = scenario.warp_to_level.unwrap_or(0);

    let vdp_full_frame = !force_scene
        && filter.background
        && filter.terrain
        && filter.block_overrides
        && filter.platforms
        && filter.enemies
        && filter.player
        && filter.effects
        && filter.eol_score
        && filter.hud;
    let (mut level_visuals, mut block_grid) = load_session_assets(manifest, current_visual_level)?;
    let w = NATIVE_FRAME_WIDTH as usize;
    let h = NATIVE_FRAME_HEIGHT as usize;

    let mut runtime = NativeRuntime::with_generated_level_layout(manifest.clone());
    runtime.set_recorded_title_card_handoff_delay(true);
    if let Some(level) = scenario.warp_to_level {
        let target_level = LevelId(level);
        runtime.boot_to_level(target_level);
        if scenario.settle_oracle_warp {
            runtime.settle_recording_oracle_warp_live_window(target_level);
        }
    }
    if let Some(position) = scenario.start_position {
        runtime.set_player_probe_position(position.x, position.y);
    }
    if let Some(position) = scenario.camera_position {
        runtime.set_camera_probe_position(position.x, position.y);
    }
    if let Some(helmet_id) = scenario.playtest_form {
        let form = PlayerForm::from_helmet_id(helmet_id)
            .ok_or_else(|| anyhow!("invalid playtest_form helmet id {helmet_id}"))?;
        runtime.set_playtest_form(form);
    }
    if let Some(count) = scenario.initial_diamond_count {
        runtime.set_probe_diamond_count(count);
    }
    if let Some(count) = scenario.initial_lives {
        runtime.set_probe_lives_count(count);
    }

    let mut sorted_frames = frame_indices.to_vec();
    sorted_frames.sort_unstable();
    sorted_frames.dedup();
    let mut requested = sorted_frames.iter().peekable();
    let max_frame = sorted_frames.last().copied().unwrap_or(0);
    let delay_recording_input = scenario.settle_oracle_warp && scenario.warp_to_level.is_some();

    for frame_idx in 0..=max_frame {
        let input_index = if delay_recording_input {
            frame_idx.checked_sub(1)
        } else {
            Some(frame_idx)
        };
        let buttons = input_index
            .and_then(|index| scenario.script.get(index))
            .copied()
            .unwrap_or(0);
        let previous_trace = runtime.trace_frame();
        let trace = runtime.step_frame(FrameInput::from_mask(buttons));

        if requested.peek().is_none_or(|&&wanted| wanted != frame_idx) {
            continue;
        }
        requested.next();

        let render_level = runtime.source_port_render_level().0;
        if render_level != current_visual_level {
            current_visual_level = render_level;
            let loaded = load_session_assets(manifest, render_level)?;
            level_visuals = loaded.0;
            block_grid = loaded.1;
        }
        let render_trace = render_trace_with_vdp_latched_state_and_resident(
            &trace,
            Some(&previous_trace),
            runtime.source_port_render_trace_snapshot(),
        );
        let render_block_grid = runtime
            .source_port_render_block_grid()
            .unwrap_or(&block_grid);

        let mut rgb = vec![0u8; w * h * 3];
        if vdp_full_frame {
            if let Some(vdp) = runtime.vdp_state() {
                let snapshot = vdp_snapshot_from_core(vdp);
                wildside_md::render_vdp_into(&snapshot, &mut rgb);
                render_vdp_trace_overlay(&render_trace, &level_visuals, &mut rgb);
            } else {
                let scene = build_scene_filtered(
                    &render_trace,
                    false,
                    Some(&level_visuals),
                    Some(render_block_grid),
                    filter,
                );
                render_scene(&scene, &mut rgb);
            }
        } else {
            let scene = build_scene_filtered(
                &render_trace,
                false,
                Some(&level_visuals),
                Some(render_block_grid),
                filter,
            );
            render_scene(&scene, &mut rgb);
        }

        let filename = format!("{}_frame_{frame_idx:05}.png", scenario.slug);
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

/// Dump an enemy family's sprite tile sheet as a PNG grid (16 tiles per row,
/// row-major so a tile's grid index equals its `tile_base_index`). Renders with
/// the family's level-resolved palette (incl. forest borrow), so it shows the
/// art exactly as the live renderer sees it. Pixel index 0 is drawn dark
/// magenta to mark tile bounds/transparency. Used to locate the correct
/// `tile_base_index` for sub-sprites like the RockTank turret (#145).
pub(crate) fn dump_enemy_sheet(
    manifest: &AssetManifest,
    level_id: u8,
    enemy_type: u8,
    output: &Path,
    assembled_frame: Option<(u16, u16, u16)>,
) -> Result<()> {
    use native_player::{load_session_assets, render_frame_on_blank_canvas};

    let (level_visuals, _block_grid) = load_session_assets(manifest, level_id)?;
    let family = level_visuals
        .enemies
        .families
        .get(&enemy_type)
        .ok_or_else(|| {
            anyhow!("enemy 0x{enemy_type:02X} not present in level 0x{level_id:02X} visuals")
        })?;
    let first_frame = family
        .frames
        .first()
        .ok_or_else(|| anyhow!("enemy 0x{enemy_type:02X} has no frames"))?;
    let sheet = &first_frame.tile_sheet;
    let palette = &family.palette;

    // Assembled-frame mode: render one sub-sprite frame the way the live
    // renderer assembles it (column-major MD tile order), on a magenta canvas
    // so transparent pixels are obvious. Used to inspect e.g. the RockTank
    // turret geometry (#145).
    if let Some((tile_base, width_px, height_px)) = assembled_frame {
        let frame = wildside_render::SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px,
            height_px,
            tile_base_index: tile_base,
            tile_sheet: sheet.clone(),
        };
        // Render at (16,16) on the blank 320x224 canvas, then crop a generous
        // box around it (rounded up to whole tiles + margin) and upscale.
        let crop_w = u32::from(width_px.div_ceil(8) * 8 + 16);
        let crop_h = u32::from(height_px.div_ceil(8) * 8 + 16);
        let rgb = render_frame_on_blank_canvas(&frame, palette, 16, 16);
        let full = image::RgbImage::from_raw(320, 224, rgb)
            .ok_or_else(|| anyhow!("failed to build canvas image"))?;
        let cropped =
            image::imageops::crop_imm(&full, 8, 8, crop_w.min(312), crop_h.min(216)).to_image();
        let scaled = image::imageops::resize(
            &cropped,
            cropped.width() * 8,
            cropped.height() * 8,
            image::imageops::FilterType::Nearest,
        );
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).ok();
        }
        scaled
            .save(output)
            .with_context(|| format!("failed to write {}", output.display()))?;
        println!(
            "rendered assembled frame base=0x{tile_base:02X} {width_px}x{height_px} to {}",
            output.display()
        );
        return Ok(());
    }

    let cols = 16usize;
    let scale = 6usize;
    let gap = 2usize;
    let cell = 8 * scale + gap;
    let rows = sheet.tiles.len().div_ceil(cols).max(1);
    let image_width = cols * cell;
    let image_height = rows * cell;
    let mut rgb = vec![48u8; image_width * image_height * 3];

    for (idx, tile) in sheet.tiles.iter().enumerate() {
        let cell_x = (idx % cols) * cell;
        let cell_y = (idx / cols) * cell;
        for tile_y in 0..8 {
            for tile_x in 0..8 {
                let pixel_index = usize::from(tile.pixels[tile_y * 8 + tile_x]);
                let colour = if pixel_index == 0 {
                    (40u8, 0u8, 40u8)
                } else {
                    palette
                        .colours
                        .get(pixel_index)
                        .map_or((255, 0, 255), |c| (c.red, c.green, c.blue))
                };
                for scale_y in 0..scale {
                    for scale_x in 0..scale {
                        let dest_x = cell_x + tile_x * scale + scale_x;
                        let dest_y = cell_y + tile_y * scale + scale_y;
                        let offset = (dest_y * image_width + dest_x) * 3;
                        rgb[offset] = colour.0;
                        rgb[offset + 1] = colour.1;
                        rgb[offset + 2] = colour.2;
                    }
                }
            }
        }
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).ok();
    }
    image::save_buffer_with_format(
        output,
        &rgb,
        image_width as u32,
        image_height as u32,
        image::ColorType::Rgb8,
        image::ImageFormat::Png,
    )
    .with_context(|| format!("failed to write {}", output.display()))?;
    println!(
        "dumped {} tiles ({cols} cols x {rows} rows; index = tile_base_index) to {}",
        sheet.tiles.len(),
        output.display()
    );
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
    let native_trace = wildside_core::run_script_with_warp(
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
        wildside_render::BackgroundVisuals::Layered(background) => {
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
        wildside_render::BackgroundVisuals::Chunked(background) => {
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
            let trace = wildside_core::run_script_with_warp(
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

            let trace = wildside_core::run_script_with_warp(
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
            checksum_mismatch: None,
            // Golden diff is native-vs-prior-native, not an oracle compare, so
            // no checkpoint is oracle-masked and nothing is UNVERIFIED here.
            total_checkpoints: 0,
            masked_checkpoints: 0,
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
        // Golden diff is native-vs-prior-native, not an oracle compare, so no
        // level is oracle-masked / UNVERIFIED.
        unverified: 0,
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
    let mut pixels = rgb.chunks_exact(3).take(pixel_count);
    let Some(bg) = pixels.next() else {
        return 0;
    };
    pixels.filter(|pixel| *pixel != bg).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha1::{Digest, Sha1};
    use std::path::PathBuf;
    use wildside_core::terrain::{BlockGrid, Cell};
    use wildside_data::AssetManifest;

    fn cached_generated_manifest() -> Option<AssetManifest> {
        let repo_root = if let Ok(override_root) = std::env::var("KID_REPO_ROOT")
            && !override_root.is_empty()
        {
            PathBuf::from(override_root)
        } else {
            let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            crate_dir
                .parent()
                .and_then(std::path::Path::parent)
                .expect("wildside-cli crate should live under <repo>/crates/wildside-cli")
                .to_path_buf()
        };
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        if !manifest_path.is_file() {
            eprintln!(
                "skipping cache-backed web parity test: generated asset manifest missing at {}",
                manifest_path.display()
            );
            return None;
        }
        Some(wildside_data::load_manifest(&manifest_path).expect("load cached generated manifest"))
    }

    /// Web shell parity (#19): stepping the web session over the packed bundle
    /// blob it would fetch in the browser (#21) produces a framebuffer
    /// pixel-identical to the native `capture-frames` pipeline for the same
    /// scenario frame — the shell's canonical pixel oracle.
    #[test]
    fn gameplay_regression_web_session_matches_capture_frames_pixels() {
        use native_player::{
            LayerFilter, NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, build_scene_filtered,
            load_session_assets, render_scene,
        };

        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        use native_player::render_trace_with_vdp_latched_state_and_resident;

        // Mirror the capture_scenario_frames pipeline exactly — the canonical
        // pixel oracle renders `render_trace_with_vdp_latched_state_and_resident`
        // (previous trace + resident snapshot) against the RUNTIME's live
        // block grid. Compare a spread of frame indices, including the early
        // fade/latch window and a spawn-adjacent index, so parity holds
        // regardless of enemy spawn timing (#419's loc_3691E delay) instead
        // of pinning one frame's output.
        const FRAME_INDICES: [usize; 5] = [0, 1, 2, 50, 100];
        let last_frame = *FRAME_INDICES.iter().max().expect("non-empty");
        let scenario = idle_level_scenario(0, Some(last_frame + 10));
        let (level_visuals, block_grid) =
            load_session_assets(&manifest, 0).expect("session assets");

        let mut runtime =
            wildside_core::NativeRuntime::with_generated_level_layout(manifest.clone());
        runtime.set_recorded_title_card_handoff_delay(true);
        runtime.boot_to_level(wildside_core::LevelId(0));

        let blob =
            wildside_data::pack_bundle_blob(&manifest.output_root).expect("pack bundle blob");
        let mut session =
            wildside_web::WebSession::from_bundle_blob(&blob, Some(0)).expect("web session");

        let mut frame = runtime.trace_frame();
        for frame_idx in 0..=last_frame {
            let buttons = scenario.script.get(frame_idx).copied().unwrap_or(0);
            let previous_trace = frame.clone();
            frame = runtime.step_frame(wildside_core::FrameInput::from_mask(buttons));
            session.step(buttons);
            if !FRAME_INDICES.contains(&frame_idx) {
                continue;
            }
            let render_trace = render_trace_with_vdp_latched_state_and_resident(
                &frame,
                Some(&previous_trace),
                runtime.source_port_render_trace_snapshot(),
            );
            let render_block_grid = runtime
                .source_port_render_block_grid()
                .unwrap_or(&block_grid);
            let scene = build_scene_filtered(
                &render_trace,
                false,
                Some(&level_visuals),
                Some(render_block_grid),
                LayerFilter::default(),
            );
            let mut want = vec![0u8; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
            render_scene(&scene, &mut want);
            assert!(
                session.rgb_frame() == want.as_slice(),
                "web session frame {frame_idx} diverged from the capture-frames pixel pipeline"
            );
        }
    }

    /// Shell-screen parity (#19): an unwarped web session renders the VDP
    /// attract screens (here: the Sega logo, ROM-visible from elapsed frame
    /// 34 per Load_SegaScreen loc_12E64) pixel-identical to the native
    /// `capture-frames` VDP pipeline — render_vdp_into + the title-card
    /// worker overlay, straight off the runtime's VDP state.
    #[test]
    fn shell_regression_web_session_matches_capture_frames_vdp_pixels() {
        use native_player::{LayerFilter, NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH};

        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        const FRAME_IDX: usize = 90;
        let scenario = DynamicScenario {
            slug: String::from("attract_boot_probe"),
            script: vec![0u8; FRAME_IDX + 2],
            frame_count: FRAME_IDX + 1,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        };
        let output_dir = tempfile::tempdir().expect("tempdir");
        capture_scenario_frames(
            &manifest,
            &scenario,
            &[FRAME_IDX],
            output_dir.path(),
            LayerFilter::default(),
            false,
        )
        .expect("capture scenario frames");
        let png_path = output_dir
            .path()
            .join(format!("{}_frame_{FRAME_IDX:05}.png", scenario.slug));
        let want = image::open(&png_path).expect("read captured png").to_rgb8();

        let blob =
            wildside_data::pack_bundle_blob(&manifest.output_root).expect("pack bundle blob");
        let mut session =
            wildside_web::WebSession::from_bundle_blob(&blob, None).expect("web session");
        for frame_idx in 0..=FRAME_IDX {
            session.step(scenario.script.get(frame_idx).copied().unwrap_or(0));
        }
        assert!(
            session.rgb_frame() == want.as_raw().as_slice(),
            "web session VDP frame {FRAME_IDX} diverged from the capture-frames pixel pipeline"
        );
        assert!(
            count_non_background_pixels(
                session.rgb_frame(),
                (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT) as usize
            ) > 0,
            "Sega logo frame {FRAME_IDX} rendered as a blank screen"
        );
    }

    /// Web audio parity (#20): a web session with audio enabled emits the
    /// exact f32 stream `render_native_audio_samples_for_trace` (the
    /// verify-audio-output path) generates for the same traces. Both drive the
    /// shared NativeAudioDirector, so this pins the WebSession wiring — rate
    /// 44.1kHz means every frame pulls exactly AUDIO_SAMPLES_PER_FRAME.
    #[test]
    fn gameplay_regression_web_session_audio_matches_native_render() {
        use native_player::{
            AUDIO_SAMPLE_RATE, load_gems_audio_engine, render_native_audio_samples_for_trace,
        };

        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let fs_source = wildside_data::FsAssetSource::new(manifest.output_root.clone());
        let Ok(mut gems) = load_gems_audio_engine(&fs_source) else {
            eprintln!("skipping web audio parity test: cached bundle has no GEMS sound banks");
            return;
        };

        const FRAMES: usize = 240;
        let blob =
            wildside_data::pack_bundle_blob(&manifest.output_root).expect("pack bundle blob");
        let mut session =
            wildside_web::WebSession::from_bundle_blob(&blob, Some(0)).expect("web session");
        session
            .enable_audio(AUDIO_SAMPLE_RATE)
            .expect("enable audio");
        let mut traces = Vec::with_capacity(FRAMES);
        let mut web_samples = Vec::new();
        for _ in 0..FRAMES {
            session.step(0);
            traces.push(session.trace().clone());
            web_samples.extend_from_slice(session.audio_samples());
        }

        let want = render_native_audio_samples_for_trace(&manifest, &traces, None, &mut gems);
        assert_eq!(web_samples.len(), want.len(), "sample count diverged");
        assert!(
            web_samples == want,
            "web session audio diverged from the native GEMS render for the same traces"
        );
    }

    fn fixture_manifest() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: wildside_data::GENERATED_BUNDLE_VERSION,
            rom_sha1: String::from("fixture-rom"),
            upstream_commit: String::from("fixture-upstream"),
            source_root: PathBuf::from("/tmp/source"),
            output_root: PathBuf::from("/tmp/output"),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

    #[test]
    fn capture_frame_block_grid_prefers_live_runtime_mutations() {
        let fallback_grid = BlockGrid::from_level_layout(&[0xE1, 0x01], 1, 1);
        let mut runtime = NativeRuntime::new(fixture_manifest());

        assert_eq!(
            capture_frame_block_grid(&runtime, &fallback_grid).cell_at(0, 0),
            Cell::from_be_bytes(0xE1, 0x01),
            "capture-scenario-frames should fall back to loaded Level_Layout before the runtime has a grid",
        );

        let live_grid = BlockGrid::from_level_layout(&[0xE0, 0x01], 1, 1);
        runtime.set_block_grid(live_grid);

        assert_eq!(
            capture_frame_block_grid(&runtime, &fallback_grid).cell_at(0, 0),
            Cell::from_be_bytes(0xE0, 0x01),
            "ASM sub_1007A writes `$E001` before Prize_Init; capture-scenario-frames must render the live runtime grid so struck prize blocks do not appear as stale `$E101` cells",
        );
    }

    #[test]
    fn capture_scenario_frame_uses_published_demo_return_render_latch() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate_dir
            .parent()
            .and_then(Path::parent)
            .expect("wildside-cli crate should live under <repo>/crates/wildside-cli");
        let scenario: DynamicScenario = serde_json::from_str(
            &fs::read_to_string(
                repo_root.join("tests/data/tas/any-percent-published/scenario.json"),
            )
            .expect("published TAS scenario should be readable"),
        )
        .expect("published TAS scenario should parse");
        let output_dir = tempfile::tempdir().expect("tempdir");

        capture_scenario_frames(
            &manifest,
            &scenario,
            &[1837],
            output_dir.path(),
            native_player::LayerFilter::default(),
            false,
        )
        .expect("capture published demo-return frame");

        let rgb = image::open(
            output_dir
                .path()
                .join("any-percent-published_frame_01837.png"),
        )
        .expect("read captured frame")
        .to_rgb8()
        .into_raw();
        // Native render-latch snapshot re-derived 2026-07-20 (#417): frame 1837
        // is inside the any-percent-published --compare-pixels range (0..37759)
        // that feeds tas-pixel-baseline, so a green verify-tas-full confirms the
        // current native render is the gate-accepted pixel state, not a regression.
        assert_eq!(
            format!("{:x}", Sha1::digest(rgb)),
            "98998a32c67449c801fb0c0cde1dc420c2e81da3",
            "ASM Mode_Level transfers the terminal sprite table before ReadJoypad loc_C0A resets Current_LevelID and loc_6E2 fades the resident demo-1 raster",
        );
    }

    #[test]
    fn golden_scenarios_cover_standard_level_routes() {
        let scenarios = golden_scenarios_for_level(0x2A);
        let summary = scenarios
            .iter()
            .map(|scenario| {
                (
                    scenario.slug.as_str(),
                    scenario.warp_to_level,
                    scenario.frame_count,
                    scenario.checkpoints.len(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            summary,
            vec![
                ("idle_level_2a", Some(0x2A), 360, 2),
                ("walk_right_2a", Some(0x2A), 360, 4),
                ("prize_block_2a", Some(0x2A), 360, 6),
                ("stomp_enemy_2a", Some(0x2A), 420, 4),
            ]
        );
    }

    #[test]
    fn diff_golden_frames_reports_labelled_field_changes() {
        let old_frames = vec![
            NativeTraceFrame {
                number_lives: 3,
                kid_x_pos: Some(0x0010_0000),
                ..NativeTraceFrame::default()
            },
            NativeTraceFrame {
                paused: false,
                ..NativeTraceFrame::default()
            },
        ];
        let new_frames = vec![
            NativeTraceFrame {
                number_lives: 4,
                kid_x_pos: Some(0x0011_0000),
                ..NativeTraceFrame::default()
            },
            NativeTraceFrame {
                paused: true,
                ..NativeTraceFrame::default()
            },
        ];
        let checkpoints = vec![DynamicCheckpoint {
            label: "level_settle".into(),
            frame_index: 60,
        }];

        let diffs = diff_golden_frames("golden_route", &old_frames, &new_frames, &checkpoints);
        let diff_summary = diffs
            .iter()
            .map(|diff| {
                (
                    diff.checkpoint.as_str(),
                    diff.field.as_str(),
                    diff.old_value.as_str(),
                    diff.new_value.as_str(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            diff_summary,
            vec![
                ("golden_route/level_settle", "number_lives", "3", "4"),
                (
                    "golden_route/level_settle",
                    "kid_x_pos",
                    "Some(1048576)",
                    "Some(1114112)",
                ),
                ("golden_route/checkpoint_1", "paused", "false", "true"),
            ]
        );
    }

    #[test]
    fn diff_golden_frames_only_compares_paired_frames() {
        let old_frames = vec![NativeTraceFrame {
            score: 100,
            ..NativeTraceFrame::default()
        }];
        let new_frames = vec![
            NativeTraceFrame {
                score: 100,
                ..NativeTraceFrame::default()
            },
            NativeTraceFrame {
                score: 999,
                ..NativeTraceFrame::default()
            },
        ];

        assert!(
            diff_golden_frames("golden_route", &old_frames, &new_frames, &[]).is_empty(),
            "extra frames without a paired old/new counterpart are outside the golden diff contract"
        );
    }

    #[test]
    fn count_non_background_pixels_counts_rgb_channel_changes() {
        let rgb = [
            10, 20, 30, // background
            10, 20, 30, // same
            11, 20, 30, // red differs
            10, 21, 30, // green differs
            10, 20, 31, // blue differs
            99, 88, // incomplete trailing bytes are ignored
        ];

        assert_eq!(count_non_background_pixels(&rgb, 5), 3);
        assert_eq!(count_non_background_pixels(&rgb, 3), 1);
        assert_eq!(count_non_background_pixels(&rgb, 0), 0);
        assert_eq!(count_non_background_pixels(&[10, 20], 1), 0);
    }
}
