use anyhow::{Context, Result};
use std::path::Path;

use crate::scenarios::{DynamicCheckpoint, DynamicScenario};
use crate::sweep::{ADVISORY_FIELDS, compare_sweep_states, extract_checkpoint_states};

/// Run native and oracle side-by-side, print per-frame divergences.
#[allow(clippy::too_many_arguments)]
pub fn run_tas_diff(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &kid_data::AssetManifest,
    scenario_path: &Path,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
    only_divergent: bool,
    include_advisory: bool,
    include_physics_trace: bool,
    visual: bool,
) -> Result<()> {
    let scenario_json = std::fs::read_to_string(scenario_path)
        .with_context(|| format!("failed to read {}", scenario_path.display()))?;
    let mut scenario: DynamicScenario =
        serde_json::from_str(&scenario_json).context("invalid scenario JSON")?;

    let start = frame_start.unwrap_or(0);
    let end = frame_end
        .unwrap_or(scenario.frame_count)
        .min(scenario.frame_count);

    // Slice script and regenerate per-frame checkpoints
    scenario.script = scenario.script[..end.min(scenario.script.len())].to_vec();
    scenario.frame_count = end;
    scenario.checkpoints = (start..end)
        .map(|f| DynamicCheckpoint {
            label: format!("frame_{f}"),
            frame_index: f,
        })
        .collect();

    println!(
        "tas-diff: {} (frames {start}..{end})",
        scenario.slug
    );

    // Run native (with physics trace if requested)
    println!("  running native ({end} frames)...");
    let native_trace = if include_physics_trace {
        kid_core::run_script_with_trace(
            manifest.clone(),
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
        )
    } else {
        kid_core::run_script_with_warp(
            manifest.clone(),
            &scenario.script,
            scenario.frame_count,
            scenario.warp_to_level,
        )
    };

    let native_states = extract_checkpoint_states(&native_trace, &scenario.checkpoints);

    // Run oracle
    println!("  running oracle ({end} frames)...");
    let run_dir = repo_root.join(".cache/tas-diff").join(&scenario.slug);
    std::fs::create_dir_all(&run_dir)?;
    let scenario_file = run_dir.join("scenario.json");
    std::fs::write(&scenario_file, serde_json::to_vec_pretty(&scenario)?)?;
    let env = crate::oracle::ensure_oracles(repo_root)?;
    let oracle_dir = run_dir.join("oracle");
    std::fs::create_dir_all(&oracle_dir)?;
    let oracle = crate::oracle::run_jgenesis_capture_dynamic(
        &env,
        rom_path,
        &scenario_file,
        &oracle_dir,
        None,
    )?;

    // Build per-frame diff index
    let diffs = compare_sweep_states(&native_states, &oracle.states);
    let mut diff_by_checkpoint: std::collections::HashMap<&str, Vec<_>> =
        std::collections::HashMap::new();
    for diff in &diffs {
        diff_by_checkpoint
            .entry(diff.checkpoint.as_str())
            .or_default()
            .push(diff);
    }

    // Print side-by-side
    let mut match_count = 0usize;
    let mut diverge_count = 0usize;
    println!();

    for frame_idx in start..end {
        let label = format!("frame_{frame_idx}");
        let frame_diffs = diff_by_checkpoint.get(label.as_str());
        let has_diffs = frame_diffs.is_some_and(|d| !d.is_empty());

        if !has_diffs {
            match_count += 1;
            if !only_divergent {
                println!("frame {frame_idx}: \x1b[32mMATCH\x1b[0m");
            }
            continue;
        }

        diverge_count += 1;
        let diffs = frame_diffs.unwrap();
        let auth_count = diffs
            .iter()
            .filter(|d| !ADVISORY_FIELDS.contains(&d.field.as_str()))
            .count();
        let adv_count = diffs.len() - auth_count;

        if auth_count > 0 {
            println!("frame {frame_idx}: \x1b[31mDIVERGE\x1b[0m ({auth_count} authoritative, {adv_count} advisory)");
        } else {
            println!("frame {frame_idx}: \x1b[33mADVISORY\x1b[0m ({adv_count} advisory)");
        }

        for diff in diffs {
            let is_advisory = ADVISORY_FIELDS.contains(&diff.field.as_str());
            if is_advisory && !include_advisory {
                continue;
            }
            let tag = if is_advisory { " (advisory)" } else { "" };
            println!(
                "  {:<20} native={:<14} oracle={:<14}{tag}",
                diff.field, diff.native_value, diff.oracle_value
            );
        }

        // Show physics trace if available and requested
        if include_physics_trace {
            if let Some(native_frame) = native_trace.get(frame_idx) {
                if let Some(ref pt) = native_frame.physics_trace {
                    println!("  --- physics trace ---");
                    println!(
                        "  pre_h:  x={:#010x} xv={:#010x}",
                        pt.pre_horizontal.0, pt.pre_horizontal.1 as u32
                    );
                    println!(
                        "  post_h: x={:#010x} xv={:#010x}  step={}",
                        pt.post_horizontal.0,
                        pt.post_horizontal.1 as u32,
                        pt.horizontal_step_applied
                    );
                    println!(
                        "  pre_v:  y={:#010x} yv={:#010x}",
                        pt.pre_vertical.0, pt.pre_vertical.1 as u32
                    );
                    println!(
                        "  post_v: y={:#010x} yv={:#010x}  grav={}  half_grav={}",
                        pt.post_vertical.0,
                        pt.post_vertical.1 as u32,
                        pt.gravity_applied,
                        pt.ascending_and_b_held
                    );
                    if pt.jump_initiated {
                        println!(
                            "  jump:   yv={:#010x}",
                            pt.jump_y_vel.unwrap_or(0) as u32
                        );
                    }
                    if pt.floor_hit {
                        println!(
                            "  floor:  y_px={} landing={:?}",
                            pt.floor_y_px.unwrap_or(-1),
                            pt.landing_path
                        );
                    }
                    if pt.wall_hit {
                        println!(
                            "  wall:   snap_x={:#010x}",
                            pt.wall_snap_x.unwrap_or(0)
                        );
                    }
                    println!(
                        "  final:  ({:#010x},{:#010x}) vel=({:#010x},{:#010x}) ground={}→{}",
                        pt.final_pos.0,
                        pt.final_pos.1,
                        pt.final_vel.0 as u32,
                        pt.final_vel.1 as u32,
                        pt.on_ground_before,
                        pt.on_ground_after
                    );
                }
            }
        }
    }

    println!();
    println!(
        "summary: {match_count} match, {diverge_count} diverge (frames {start}..{end})"
    );

    // Visual diff: generate 3-panel composite PNGs at divergent frames
    if visual && diverge_count > 0 {
        let visual_dir = repo_root
            .join(".cache/tas-diff")
            .join(&scenario.slug)
            .join("visual");
        std::fs::create_dir_all(&visual_dir)?;

        // Collect divergent frame indices
        let divergent_frames: Vec<usize> = (start..end)
            .filter(|f| {
                let label = format!("frame_{f}");
                diff_by_checkpoint
                    .get(label.as_str())
                    .is_some_and(|d| !d.is_empty())
            })
            .collect();

        println!("generating visual diffs for {} frames...", divergent_frames.len());

        // Build oracle PNG index: frame_index → png_path
        let oracle_png_index: std::collections::HashMap<usize, &std::path::Path> = oracle
            .frames
            .iter()
            .map(|f| (f.frame_index, f.png_path.as_path()))
            .collect();

        // Render native frames for divergent frames
        let checkpoints: Vec<crate::scenarios::GameplayCheckpoint> = divergent_frames
            .iter()
            .map(|&f| {
                let label: &'static str = Box::leak(format!("frame_{f}").into_boxed_str());
                crate::scenarios::GameplayCheckpoint {
                    label,
                    frame_index: f,
                }
            })
            .collect();

        let warp_level = scenario.warp_to_level.unwrap_or(0);
        let native_rendered =
            crate::native_player::render_gameplay_frame_evidence_for_level(
                manifest,
                &native_trace,
                &checkpoints,
                warp_level,
            )?;

        let mut saved = 0usize;
        for (label, frame_idx, w, h, native_rgb) in &native_rendered {
            let oracle_png_path = match oracle_png_index.get(frame_idx) {
                Some(p) => *p,
                None => continue,
            };
            let oracle_img = image::open(oracle_png_path)
                .with_context(|| format!("failed to open oracle PNG: {}", oracle_png_path.display()))?
                .to_rgb8();
            let oracle_rgb = oracle_img.as_raw();

            let (cw, ch, composite) = crate::diff::generate_diff_composite(
                native_rgb,
                oracle_rgb,
                *w as u32,
                *h as u32,
            );
            let out_path = visual_dir.join(format!("{label}.png"));
            image::save_buffer(
                &out_path,
                &composite,
                cw,
                ch,
                image::ColorType::Rgb8,
            )
            .with_context(|| format!("failed to write {}", out_path.display()))?;
            saved += 1;
        }
        println!("saved {saved} visual diffs to {}", visual_dir.display());
    }

    Ok(())
}
