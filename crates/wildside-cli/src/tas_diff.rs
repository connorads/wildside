use anyhow::{Context, Result};
use std::path::Path;

use crate::oracle::OracleState;
use crate::scenarios::{
    DynamicCheckpoint, DynamicScenario, NativeDynamicRunOptions, run_native_dynamic_scenario,
};
use crate::sweep::{
    DivergenceCategory, FrontierKind, StateDiff, compare_sweep_states_for_frontier,
    extract_checkpoint_states,
};
use crate::tas::{first_crc_mismatch_from_frame, reclassify_crc_clean_lion_projectile_count_diffs};

/// Run native and oracle side-by-side, print per-frame divergences.
#[allow(clippy::too_many_arguments)]
#[allow(
    clippy::fn_params_excessive_bools,
    reason = "game engine API with multiple independent toggles"
)]
pub fn run_tas_diff(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &wildside_data::AssetManifest,
    scenario_path: &Path,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
    only_divergent: bool,
    include_advisory: bool,
    include_physics_trace: bool,
    show_object_roster_on_match: bool,
    rng_trace: bool,
    drips_trace: bool,
    player_trace: bool,
    visual: bool,
    frontier_kind: FrontierKind,
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
        "tas-diff: {} (frames {start}..{end}, frontier {:?})",
        scenario.slug, frontier_kind
    );

    // Run native (with physics trace if requested)
    println!("  running native ({end} frames)...");
    let native_trace = run_native_dynamic_scenario(
        manifest.clone(),
        &scenario,
        NativeDynamicRunOptions {
            // Honour the scenario's `settle_oracle_warp` flag so diagnostics
            // replay the same recording/title-card bootstrap as the oracle.
            settled_warp: false,
            trace_physics: include_physics_trace,
        },
    )?;

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

    if rng_trace {
        println!(
            "frame native_rng native_byte native_delta oracle_rng oracle_byte oracle_delta ring_diff"
        );
        let mut previous_native_rng = None;
        let mut previous_oracle_rng = None;
        for frame_idx in start..end {
            let label = format!("frame_{frame_idx}");
            let native_rng_offset = native_trace
                .get(frame_idx)
                .and_then(|frame| frame.rng_offset);
            let native_rng_last_byte = native_trace
                .get(frame_idx)
                .and_then(|frame| frame.rng_last_byte);
            let oracle_rng_offset =
                checkpoint_state(&oracle.states, &label).and_then(|state| state.rng_offset);
            let oracle_rng_last_byte =
                checkpoint_state(&oracle.states, &label).and_then(|state| state.rng_last_byte);
            let ring_diff = format_rng_ring_difference(
                native_trace
                    .get(frame_idx)
                    .and_then(|frame| frame.rng_ring.as_ref()),
                checkpoint_state(&oracle.states, &label).and_then(|state| state.rng_ring.as_ref()),
            );
            let native_delta = format_rng_delta(previous_native_rng, native_rng_offset);
            let oracle_delta = format_rng_delta(previous_oracle_rng, oracle_rng_offset);
            let native_rng =
                native_rng_offset.map_or_else(|| "-".to_owned(), |offset| format!("{offset:#06x}"));
            let native_byte =
                native_rng_last_byte.map_or_else(|| "-".to_owned(), |byte| format!("{byte:#04x}"));
            let oracle_rng =
                oracle_rng_offset.map_or_else(|| "-".to_owned(), |offset| format!("{offset:#06x}"));
            let oracle_byte =
                oracle_rng_last_byte.map_or_else(|| "-".to_owned(), |byte| format!("{byte:#04x}"));
            println!(
                "{frame_idx:>5} {native_rng:>10} {native_byte:>11} {native_delta:>12} {oracle_rng:>10} {oracle_byte:>11} {oracle_delta:>12} {ring_diff}"
            );
            previous_native_rng = native_rng_offset;
            previous_oracle_rng = oracle_rng_offset;
        }
        return Ok(());
    }

    if player_trace {
        println!(
            "frame input native_x native_y native_xv native_yv native_move native_ground native_facing native_rng oracle_x oracle_y oracle_xv oracle_yv oracle_move oracle_ground oracle_dir oracle_rng"
        );
        for frame_idx in start..end {
            let label = format!("frame_{frame_idx}");
            let native_frame = native_trace.get(frame_idx);
            let oracle_state = checkpoint_state(&oracle.states, &label);
            let input = scenario
                .script
                .get(frame_idx)
                .map_or_else(|| "-".to_owned(), |mask| format!("{mask:#04x}"));
            println!(
                "{frame_idx:>5} {input:>5} {:>8} {:>8} {:>9} {:>9} {:>11} {:>13} {:>13} {:>10} {:>8} {:>8} {:>9} {:>9} {:>11} {:>13} {:>10} {:>10}",
                format_opt_u32(native_frame.and_then(|frame| frame.kid_x_pos)),
                format_opt_u32(native_frame.and_then(|frame| frame.kid_y_pos)),
                format_opt_i32(native_frame.and_then(|frame| frame.kid_x_velocity)),
                format_opt_i32(native_frame.and_then(|frame| frame.kid_y_velocity)),
                format_opt_u16(native_frame.and_then(|frame| frame.character_movement)),
                format_opt_bool(native_frame.and_then(|frame| frame.kid_on_ground)),
                native_frame
                    .and_then(|frame| frame.kid_facing)
                    .map_or_else(|| "-".to_owned(), |facing| format!("{facing:?}")),
                format_opt_u16_hex(native_frame.and_then(|frame| frame.rng_offset)),
                oracle_state.map_or_else(|| "-".to_owned(), |state| state.kid_x_pos.to_string()),
                oracle_state.map_or_else(|| "-".to_owned(), |state| state.kid_y_pos.to_string()),
                format_opt_i32(oracle_state.and_then(|state| state.kid_x_vel)),
                format_opt_i32(oracle_state.and_then(|state| state.kid_y_vel)),
                format_opt_u16(oracle_state.and_then(|state| state.character_movement)),
                oracle_state
                    .and_then(|state| state.gameplay_debug_fields.get("kid_on_ground"))
                    .cloned()
                    .unwrap_or_else(|| "-".to_owned()),
                oracle_state
                    .and_then(|state| state.kid_x_direction)
                    .map_or_else(|| "-".to_owned(), |direction| format!("{direction:#04x}")),
                format_opt_u16_hex(oracle_state.and_then(|state| state.rng_offset)),
            );
        }
        return Ok(());
    }

    if drips_trace {
        println!(
            "frame native_rng oracle_rng native_kid_x oracle_kid_x native_lifecycle native_reseed oracle_slots"
        );
        for frame_idx in start..end {
            let label = format!("frame_{frame_idx}");
            let native_frame = native_trace.get(frame_idx);
            let oracle_state = checkpoint_state(&oracle.states, &label);
            let native_rng = native_frame
                .and_then(|frame| frame.rng_offset)
                .map_or_else(|| "-".to_owned(), |offset| format!("{offset:#06x}"));
            let oracle_rng = oracle_state
                .and_then(|state| state.rng_offset)
                .map_or_else(|| "-".to_owned(), |offset| format!("{offset:#06x}"));
            let native_kid_x = native_frame
                .and_then(|frame| frame.kid_x_pos)
                .map_or_else(|| "-".to_owned(), |x_pos| format!("{x_pos:#010x}"));
            let oracle_kid_x = oracle_state.map_or_else(
                || "-".to_owned(),
                |state| format!("{:#010x}", state.kid_x_pos),
            );
            println!(
                "{frame_idx:>5} {native_rng:>10} {oracle_rng:>10} {native_kid_x:>12} {oracle_kid_x:>12} {} {} {}",
                native_gameplay_field(native_frame, "drips_lifecycle_probe"),
                native_gameplay_field(native_frame, "drips_splash_reseed_probe"),
                oracle_gameplay_field(oracle_state, "oracle_drips_object_slots_loc_33488")
            );
        }
        return Ok(());
    }

    // Build per-frame diff index
    let mut diffs =
        compare_sweep_states_for_frontier(&native_states, &oracle.states, frontier_kind);
    let crc_clean = matches!(frontier_kind, FrontierKind::Gameplay)
        && !oracle.frame_crcs.is_empty()
        && first_crc_mismatch_from_frame(
            &native_trace
                .iter()
                .map(wildside_core::NativeTraceFrame::authoritative_crc32)
                .collect::<Vec<_>>(),
            &oracle.frame_crcs,
            start,
            end,
        )
        .is_none();
    reclassify_crc_clean_lion_projectile_count_diffs(
        &mut diffs,
        &native_states,
        &oracle.states,
        crc_clean,
    );
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
                if show_object_roster_on_match {
                    let oracle_state = checkpoint_state(&oracle.states, &label);
                    if native_object_debug_relevant(native_trace.get(frame_idx), oracle_state) {
                        println!("  --- object roster ---");
                        println!(
                            "  native: {}",
                            native_object_debug(native_trace.get(frame_idx))
                        );
                        println!("  oracle: {}", oracle_object_debug(oracle_state));
                    }
                    if let Some(gameplay_debug) = native_gameplay_debug(native_trace.get(frame_idx))
                    {
                        println!("  native gameplay: {gameplay_debug}");
                    }
                    if let Some(gameplay_debug) = oracle_gameplay_debug(oracle_state) {
                        println!("  oracle gameplay: {gameplay_debug}");
                    }
                }
            }
            continue;
        }

        diverge_count += 1;
        let diffs = frame_diffs.unwrap();
        let authoritative_count = diffs
            .iter()
            .filter(|diff| diff.category == DivergenceCategory::Authoritative)
            .count();
        let non_authoritative_count = diffs.len() - authoritative_count;

        if authoritative_count > 0 {
            println!(
                "frame {frame_idx}: \x1b[31mDIVERGE\x1b[0m ({authoritative_count} authoritative, {non_authoritative_count} non-authoritative)"
            );
        } else {
            println!(
                "frame {frame_idx}: \x1b[33mNON-AUTH\x1b[0m ({non_authoritative_count} non-authoritative)"
            );
        }

        for diff in diffs {
            if diff.category != DivergenceCategory::Authoritative && !include_advisory {
                continue;
            }
            let tag = diff_category_tag(diff);
            println!(
                "  {:<20} native={:<14} oracle={:<14}{tag}",
                diff.field, diff.native_value, diff.oracle_value
            );
        }

        // Show physics trace if available and requested
        if include_physics_trace
            && let Some(native_frame) = native_trace.get(frame_idx)
            && let Some(ref pt) = native_frame.physics_trace
        {
            println!("  --- physics trace ---");
            println!(
                "  pre_h:  x={:#010x} xv={:#010x}",
                pt.pre_horizontal.0, pt.pre_horizontal.1 as u32
            );
            println!(
                "  post_h: x={:#010x} xv={:#010x}  step={}",
                pt.post_horizontal.0, pt.post_horizontal.1 as u32, pt.horizontal_step_applied
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
                println!("  jump:   yv={:#010x}", pt.jump_y_vel.unwrap_or(0) as u32);
            }
            if pt.floor_hit {
                println!(
                    "  floor:  y_px={} landing={:?}",
                    pt.floor_y_px.unwrap_or(-1),
                    pt.landing_path
                );
            }
            if pt.wall_hit {
                println!("  wall:   snap_x={:#010x}", pt.wall_snap_x.unwrap_or(0));
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

        let oracle_state = checkpoint_state(&oracle.states, &label);
        if should_print_object_roster(
            authoritative_count > 0,
            show_object_roster_on_match,
            native_trace.get(frame_idx),
            oracle_state,
        ) {
            println!("  --- object roster ---");
            println!(
                "  native: {}",
                native_object_debug(native_trace.get(frame_idx))
            );
            println!("  oracle: {}", oracle_object_debug(oracle_state));
        }
        if let Some(gameplay_debug) = native_gameplay_debug(native_trace.get(frame_idx)) {
            println!("  native gameplay: {gameplay_debug}");
        }
        if let Some(gameplay_debug) = oracle_gameplay_debug(oracle_state) {
            println!("  oracle gameplay: {gameplay_debug}");
        }
    }

    println!();
    println!("summary: {match_count} match, {diverge_count} diverge (frames {start}..{end})");

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

        println!(
            "generating visual diffs for {} frames...",
            divergent_frames.len()
        );

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
        let native_rendered = crate::native_player::render_gameplay_frame_evidence_for_level(
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
                .with_context(|| {
                    format!("failed to open oracle PNG: {}", oracle_png_path.display())
                })?
                .to_rgb8();
            let oracle_rgb = oracle_img.as_raw();

            let (cw, ch, composite) =
                crate::diff::generate_diff_composite(native_rgb, oracle_rgb, *w as u32, *h as u32);
            let out_path = visual_dir.join(format!("{label}.png"));
            image::save_buffer(&out_path, &composite, cw, ch, image::ColorType::Rgb8)
                .with_context(|| format!("failed to write {}", out_path.display()))?;
            saved += 1;
        }
        println!("saved {saved} visual diffs to {}", visual_dir.display());
    }

    Ok(())
}

fn format_rng_ring_difference(
    native: Option<&wildside_core::RomRandomRing>,
    oracle: Option<&wildside_core::RomRandomRing>,
) -> String {
    match (native, oracle) {
        (Some(native), Some(oracle)) => native.first_difference(oracle).map_or_else(
            || String::from("match"),
            |(index, native_byte, oracle_byte)| {
                format!("byte[{index}]={native_byte:#04x}/{oracle_byte:#04x}")
            },
        ),
        (None, Some(_)) => String::from("native-missing"),
        (Some(_), None) => String::from("oracle-missing"),
        (None, None) => String::from("missing"),
    }
}

fn diff_category_tag(diff: &StateDiff) -> &'static str {
    match diff.category {
        DivergenceCategory::Authoritative => "",
        DivergenceCategory::Advisory => " (advisory)",
        DivergenceCategory::Accumulation => " (accumulation)",
        DivergenceCategory::Rng => " (rng)",
        DivergenceCategory::OracleLimitation => " (oracle limitation)",
    }
}

fn checkpoint_state<'a>(states: &'a [OracleState], label: &str) -> Option<&'a OracleState> {
    states.iter().find(|state| state.label == label)
}

fn format_rng_delta(previous: Option<u16>, current: Option<u16>) -> String {
    match (previous, current) {
        (Some(previous), Some(current)) => {
            let delta = current.wrapping_sub(previous) & 0x003f;
            format!("+{delta:#04x}")
        }
        _ => "-".to_owned(),
    }
}

fn format_opt_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn format_opt_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn format_opt_u16(value: Option<u16>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn format_opt_u16_hex(value: Option<u16>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| format!("{value:#06x}"))
}

fn format_opt_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn should_print_object_roster(
    has_authoritative_diff: bool,
    show_on_match: bool,
    native_frame: Option<&wildside_core::NativeTraceFrame>,
    oracle_state: Option<&OracleState>,
) -> bool {
    has_authoritative_diff
        || (show_on_match && native_object_debug_relevant(native_frame, oracle_state))
}

fn native_object_debug_relevant(
    native_frame: Option<&wildside_core::NativeTraceFrame>,
    oracle_state: Option<&OracleState>,
) -> bool {
    native_frame.is_some_and(|frame| {
        !frame.active_objects.is_empty()
            || !frame.block_bounces.is_empty()
            || !frame.block_destroy_effects.is_empty()
            || !frame.floating_items.is_empty()
    }) || oracle_state.is_some_and(|state| !state.active_enemies.is_empty())
}

fn native_object_debug(native_frame: Option<&wildside_core::NativeTraceFrame>) -> String {
    let Some(frame) = native_frame else {
        return "-".to_owned();
    };

    let mut entries = Vec::new();
    for object in &frame.active_objects {
        match object.kind {
            wildside_core::NativeObjectKind::Player => {}
            wildside_core::NativeObjectKind::Enemy { object_type } => {
                entries.push(format_native_enemy_debug(object_type, object));
            }
            wildside_core::NativeObjectKind::Platform { kind, subtype } => entries.push(format!(
                "platform_{kind:02X}:{subtype:02X}@{:#010x},{:#010x}",
                object.x_pos, object.y_pos
            )),
        }
    }
    for &(cell_x, cell_y, remaining, y_offset) in &frame.block_bounces {
        entries.push(format!(
            "block_bounce@{cell_x:02X},{cell_y:02X} rem={remaining} y_off={y_offset}"
        ));
    }
    for &(x_px, y_px, remaining, content_id) in &frame.floating_items {
        entries.push(format!(
            "floating_item#{content_id:02X}@{x_px:03},{y_px:03} rem={remaining}"
        ));
    }
    for &(x_px, y_px, remaining, direction) in &frame.block_destroy_effects {
        entries.push(format!(
            "block_fx@{x_px:03},{y_px:03} rem={remaining} dir={direction}"
        ));
    }

    summarize_entries(entries)
}

fn native_gameplay_debug(native_frame: Option<&wildside_core::NativeTraceFrame>) -> Option<String> {
    let frame = native_frame?;
    if frame.gameplay_debug_fields.is_empty() && frame.rng_offset.is_none() {
        return None;
    }
    let mut fields = frame.gameplay_debug_fields.clone();
    if let Some(rng_offset) = frame.rng_offset {
        fields.insert("rng_offset".into(), format!("{rng_offset:#06x}"));
    }
    Some(format!("{fields:?}"))
}

fn native_gameplay_field(
    native_frame: Option<&wildside_core::NativeTraceFrame>,
    key: &str,
) -> String {
    native_frame
        .and_then(|frame| frame.gameplay_debug_fields.get(key))
        .cloned()
        .unwrap_or_else(|| "-".to_owned())
}

fn oracle_object_debug(oracle_state: Option<&OracleState>) -> String {
    let Some(state) = oracle_state else {
        return "-".to_owned();
    };
    let entries = state
        .active_enemies
        .iter()
        .map(format_oracle_enemy_debug)
        .collect();
    summarize_entries(entries)
}

fn oracle_gameplay_debug(oracle_state: Option<&OracleState>) -> Option<String> {
    let state = oracle_state?;
    if state.gameplay_debug_fields.is_empty() && state.rng_offset.is_none() {
        return None;
    }
    let mut fields = state.gameplay_debug_fields.clone();
    if let Some(rng_offset) = state.rng_offset {
        fields.insert("rng_offset".into(), format!("{rng_offset:#06x}"));
    }
    Some(format_oracle_debug_fields(&fields))
}

fn oracle_gameplay_field(oracle_state: Option<&OracleState>, key: &str) -> String {
    oracle_state
        .and_then(|state| state.gameplay_debug_fields.get(key))
        .cloned()
        .unwrap_or_else(|| "-".to_owned())
}

fn format_native_enemy_debug(
    object_type: u8,
    object: &wildside_core::NativeObjectSnapshot,
) -> String {
    let layout = object
        .layout_index
        .map(|index| format!("#{index:02}"))
        .unwrap_or_default();
    let velocity = match (object.x_velocity, object.y_velocity) {
        (Some(x_velocity), Some(y_velocity)) => {
            format!(
                " vel={:#010x},{:#010x}",
                x_velocity as u32, y_velocity as u32
            )
        }
        _ => String::new(),
    };
    let facing = object
        .facing
        .map(|facing| format!(" face={facing:?}"))
        .unwrap_or_default();
    let on_ground = object
        .on_ground
        .map(|on_ground| format!(" ground={}", u8::from(on_ground)))
        .unwrap_or_default();
    let hit_points = object
        .hit_points
        .map(|hit_points| format!(" hp={hit_points}"))
        .unwrap_or_default();
    let stationary = if object.stationary {
        " stationary=1"
    } else {
        ""
    };
    let shared_walker = match (
        object.shared_walker_state,
        object.shared_walker_patrol_timer,
        object.shared_walker_patrol_short_next,
    ) {
        (Some(state), Some(timer), short_next) => {
            let short_next = short_next.map_or(0, u8::from);
            format!(" walker={state:#06x}/{timer:#06x}/short={short_next}")
        }
        _ => String::new(),
    };
    let walk_phase = object
        .walk_phase
        .map(|phase| format!(" phase={phase:#04x}"))
        .unwrap_or_default();
    let wait_timer = object
        .wait_timer
        .map(|wait_timer| format!(" wait={wait_timer:#06x}"))
        .unwrap_or_default();
    let debug_fields = if object.debug_fields.is_empty() {
        String::new()
    } else {
        format!(" debug={:?}", object.debug_fields)
    };
    format!(
        "enemy_{object_type:02X}{layout}@{:#010x},{:#010x}{velocity}{facing}{on_ground}{hit_points}{stationary}{shared_walker}{walk_phase}{wait_timer}{debug_fields}",
        object.x_pos, object.y_pos,
    )
}

fn format_oracle_enemy_debug(object: &crate::oracle::OracleEnemyState) -> String {
    let role = oracle_object_role_label(object)
        .map(|label| format!("({label})"))
        .unwrap_or_default();
    let layout_index = object
        .layout_index
        .map(|index| format!(" li=#{index:02}"))
        .unwrap_or_default();
    let layout_ptr = object
        .layout_ptr
        .map(|ptr| format!(" lp={ptr:#08x}"))
        .unwrap_or_default();
    let parent_ptr = object
        .parent_object_ptr
        .map(|ptr| format!(" pp={ptr:#08x}"))
        .unwrap_or_default();
    let gfx_slot = object
        .gfx_slot_index
        .map(|slot| format!(" slot={slot}"))
        .unwrap_or_default();
    let on_ground = object
        .on_ground
        .map(|on_ground| format!(" ground={}", u8::from(on_ground)))
        .unwrap_or_default();
    let debug_fields = if object.debug_fields.is_empty() {
        String::new()
    } else {
        format!(
            " debug={}",
            format_oracle_debug_fields(&object.debug_fields)
        )
    };
    format!(
        "0x{:02X}{role}@{:#010x},{:#010x} vel={:#010x},{:#010x} face={:?}{layout_index}{layout_ptr}{parent_ptr}{gfx_slot}{on_ground}{debug_fields}",
        object.object_type,
        object.x_pos,
        object.y_pos,
        object.x_velocity as u32,
        object.y_velocity as u32,
        object.facing
    )
}

fn oracle_object_role_label(object: &crate::oracle::OracleEnemyState) -> Option<&'static str> {
    match object.gfx_slot_index {
        Some(2) => Some("kid_gfx"),
        _ => None,
    }
}

fn format_oracle_debug_fields(fields: &std::collections::BTreeMap<String, String>) -> String {
    let rendered = fields
        .iter()
        .map(|(key, value)| {
            let value = match key.as_str() {
                "collision_source" | "kid_collision_source" => format_collision_source_value(value),
                _ => value.clone(),
            };
            format!("{key:?}: {value:?}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{rendered}}}")
}

fn format_collision_source_value(raw: &str) -> String {
    let Some(source) = parse_hex_u16(raw) else {
        return raw.to_owned();
    };
    let Some(label) = collision_source_label(source) else {
        return raw.to_owned();
    };
    format!("{raw}({label})")
}

fn parse_hex_u16(raw: &str) -> Option<u16> {
    let hex = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"))?;
    u16::from_str_radix(hex, 16).ok()
}

fn collision_source_label(source: u16) -> Option<&'static str> {
    match source & 0x00FF {
        0x00 => Some("fire_demon"),
        0x01 => Some("diamond"),
        0x03 => Some("robot"),
        0x04 => Some("armadillo"),
        0x05 => Some("tar_monster"),
        0x06 => Some("sphere"),
        0x07 => Some("archer"),
        0x08 => Some("orca"),
        0x09 => Some("crab"),
        0x0A => Some("rock_tank"),
        0x0B => Some("rock_tank_shooting"),
        0x0C => Some("dragon_flying"),
        0x0D => Some("dragon"),
        0x0E => Some("cloud"),
        0x0F => Some("ufo"),
        0x10 => Some("goat"),
        0x11 => Some("ninja"),
        0x12 => Some("lion"),
        0x13 => Some("scorpion"),
        0x14 => Some("spinning_twins"),
        0x16 => Some("drip"),
        0x17 => Some("hand"),
        0x18 => Some("tornado"),
        0x19 => Some("fireball"),
        0x1A => Some("driller"),
        0x1B => Some("emo_rock"),
        0x1C => Some("mini_hopping_skull"),
        0x1D => Some("big_hopping_skull"),
        0x1F => Some("invalid_1f"),
        0x20 => Some("heady_metal"),
        0x22 => Some("shiskaboss"),
        0x23 => Some("boomerang_bosses"),
        0x24 => Some("bagel_brothers"),
        0x64 => Some("prizediamond"),
        0x6C => Some("skycutter"),
        0x70 => Some("cyclone"),
        0x74 => Some("redstealth"),
        0x78 => Some("eyeclops"),
        0x7C => Some("juggernaut"),
        0x80 => Some("ironknight"),
        0x84 => Some("berzerker"),
        0x88 => Some("maniaxe"),
        0x8C => Some("micromax"),
        0x90 => Some("ankh"),
        0x94 => Some("clock"),
        0x98 => Some("continue"),
        0x9C => Some("unknown_prize"),
        0xA0 => Some("flagpole"),
        _ => None,
    }
}

fn summarize_entries(entries: Vec<String>) -> String {
    if entries.is_empty() {
        return "-".to_owned();
    }
    const LIMIT: usize = 6;
    if entries.len() <= LIMIT {
        return entries.join("; ");
    }
    let extra = entries.len() - LIMIT;
    let shown = entries.into_iter().take(LIMIT).collect::<Vec<_>>();
    format!("{}; +{extra} more", shown.join("; "))
}

#[cfg(test)]
mod tests {
    use super::{
        checkpoint_state, collision_source_label, diff_category_tag, format_collision_source_value,
        format_native_enemy_debug, format_opt_bool, format_opt_i32, format_opt_u16,
        format_opt_u16_hex, format_opt_u32, format_oracle_enemy_debug, format_rng_delta,
        format_rng_ring_difference, native_gameplay_debug, native_gameplay_field,
        native_object_debug, native_object_debug_relevant, oracle_gameplay_debug,
        oracle_gameplay_field, oracle_object_debug, should_print_object_roster, summarize_entries,
    };
    use crate::sweep::{DivergenceCategory, StateDiff};
    use wildside_core::{
        GameMode, LevelId, NativeObjectKind, NativeObjectSnapshot, NativeTraceFrame, RomRandomRing,
        physics::Direction,
    };

    fn oracle_enemy(object_type: u8) -> crate::oracle::OracleEnemyState {
        crate::oracle::OracleEnemyState {
            object_type,
            x_pos: 0x0367_0000,
            y_pos: 0x01C0_0000,
            x_velocity: 0,
            y_velocity: 0,
            facing: crate::oracle::EnemyFacing::Right,
            collision_type: None,
            collision_source: None,
            layout_index: None,
            layout_ptr: None,
            parent_object_ptr: None,
            gfx_slot_index: None,
            on_ground: None,
            current_hp: None,
            debug_fields: std::collections::BTreeMap::new(),
        }
    }

    fn oracle_state(label: &str) -> crate::oracle::OracleState {
        crate::oracle::OracleState {
            label: label.into(),
            frame_index: 3400,
            game_mode: 0x0010,
            boot_phase: Some("demo_play".into()),
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 5,
            current_helmet_id: 6,
            current_level_id: 0x15,
            player_1_level_id: 0x15,
            camera_x_pos: 0,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            game_paused: false,
            active_enemies: Vec::new(),
            score: 0,
            number_lives: 0,
            number_diamonds: 0,
            diamond_power_active: false,
            murder_wall_x: None,
            kid_x_vel: None,
            kid_y_vel: None,
            character_movement: None,
            just_received_damage: None,
            kid_collision_type: None,
            kid_collision_source: None,
            kid_x_direction: None,
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
            rng_last_byte: None,
            rng_ring: None,
            level_special_effects: None,
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            cpu_registers: None,
        }
    }

    fn native_enemy() -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0367_0000,
            y_pos: 0x01C0_0000,
            layout_index: Some(4),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: Some(0x000a),
            shared_walker_patrol_timer: Some(0x0029),
            shared_walker_patrol_short_next: Some(false),
            walk_phase: Some(0),
            sprite_animation_phase: None,
            wait_timer: Some(0x01df),
            debug_fields: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn tas_diff_small_formatters_have_explicit_missing_and_value_output() {
        assert_eq!(format_rng_delta(Some(0x0000), Some(0x0041)), "+0x01");
        assert_eq!(format_rng_delta(Some(0x003f), Some(0x0000)), "+0x01");
        assert_eq!(format_rng_delta(None, Some(0x0001)), "-");
        assert_eq!(format_rng_delta(Some(0x0001), None), "-");

        assert_eq!(format_opt_u32(Some(123_456)), "123456");
        assert_eq!(format_opt_u32(None), "-");
        assert_eq!(format_opt_i32(Some(-42)), "-42");
        assert_eq!(format_opt_i32(None), "-");
        assert_eq!(format_opt_u16(Some(513)), "513");
        assert_eq!(format_opt_u16(None), "-");
        assert_eq!(format_opt_u16_hex(Some(0x00ab)), "0x00ab");
        assert_eq!(format_opt_u16_hex(None), "-");
        assert_eq!(format_opt_bool(Some(true)), "true");
        assert_eq!(format_opt_bool(Some(false)), "false");
        assert_eq!(format_opt_bool(None), "-");

        let native = RomRandomRing::from_bytes([0x11; RomRandomRing::LENGTH]);
        let mut oracle_bytes = [0x11; RomRandomRing::LENGTH];
        oracle_bytes[23] = 0x22;
        let oracle = RomRandomRing::from_bytes(oracle_bytes);
        assert_eq!(
            format_rng_ring_difference(Some(&native), Some(&oracle)),
            "byte[23]=0x11/0x22"
        );
        assert_eq!(
            format_rng_ring_difference(Some(&native), Some(&native)),
            "match"
        );
    }

    #[test]
    fn checkpoint_state_matches_exact_labels() {
        let states = vec![oracle_state("frame_10"), oracle_state("frame_11")];

        assert_eq!(
            checkpoint_state(&states, "frame_11").map(|state| state.frame_index),
            Some(3400)
        );
        assert!(checkpoint_state(&states, "frame_1").is_none());
    }

    #[test]
    fn diff_category_tags_cover_every_category() {
        let cases = [
            (DivergenceCategory::Authoritative, ""),
            (DivergenceCategory::Advisory, " (advisory)"),
            (DivergenceCategory::Accumulation, " (accumulation)"),
            (DivergenceCategory::Rng, " (rng)"),
            (DivergenceCategory::OracleLimitation, " (oracle limitation)"),
        ];

        for (category, expected) in cases {
            let diff = StateDiff {
                checkpoint: "frame_956".into(),
                field: "number_objects".into(),
                native_value: "10".into(),
                oracle_value: "8".into(),
                category,
            };
            assert_eq!(diff_category_tag(&diff), expected);
        }
    }

    #[test]
    fn object_roster_relevance_tracks_native_and_oracle_debug_payloads() {
        let empty_native = NativeTraceFrame::default();
        let empty_oracle = oracle_state("frame_0");
        assert!(!native_object_debug_relevant(
            Some(&empty_native),
            Some(&empty_oracle)
        ));
        assert!(!should_print_object_roster(
            false,
            true,
            Some(&empty_native),
            Some(&empty_oracle)
        ));
        assert!(should_print_object_roster(
            true,
            false,
            Some(&empty_native),
            Some(&empty_oracle)
        ));

        let native_with_enemy = NativeTraceFrame {
            active_objects: vec![native_enemy()],
            ..NativeTraceFrame::default()
        };
        assert!(native_object_debug_relevant(
            Some(&native_with_enemy),
            Some(&empty_oracle)
        ));
        assert!(should_print_object_roster(
            false,
            true,
            Some(&native_with_enemy),
            Some(&empty_oracle)
        ));

        let native_with_block_bounce = NativeTraceFrame {
            block_bounces: vec![(0x0A, 0x0B, 3, -2)],
            ..NativeTraceFrame::default()
        };
        assert!(native_object_debug_relevant(
            Some(&native_with_block_bounce),
            None
        ));

        let native_with_floating_item = NativeTraceFrame {
            floating_items: vec![(12, 34, 5, 0x64)],
            ..NativeTraceFrame::default()
        };
        assert!(native_object_debug_relevant(
            Some(&native_with_floating_item),
            None
        ));

        let native_with_block_fx = NativeTraceFrame {
            block_destroy_effects: vec![(6, 7, 8, 1)],
            ..NativeTraceFrame::default()
        };
        assert!(native_object_debug_relevant(
            Some(&native_with_block_fx),
            None
        ));

        let mut oracle_with_enemy = oracle_state("frame_0");
        oracle_with_enemy.active_enemies.push(oracle_enemy(0x0D));
        assert!(native_object_debug_relevant(None, Some(&oracle_with_enemy)));
    }

    #[test]
    fn native_and_oracle_object_debug_summarise_entries() {
        assert_eq!(native_object_debug(None), "-");
        assert_eq!(native_object_debug(Some(&NativeTraceFrame::default())), "-");
        let native_frame = NativeTraceFrame {
            active_objects: vec![
                native_enemy(),
                NativeObjectSnapshot {
                    kind: NativeObjectKind::Platform {
                        kind: 0x12,
                        subtype: 0x34,
                    },
                    x_pos: 0x0012_0000,
                    y_pos: 0x0034_0000,
                    ..native_enemy()
                },
            ],
            block_bounces: vec![(0x0A, 0x0B, 3, -2)],
            floating_items: vec![(12, 34, 5, 0x64)],
            block_destroy_effects: vec![(6, 7, 8, 1)],
            ..NativeTraceFrame::default()
        };
        let native_debug = native_object_debug(Some(&native_frame));
        assert!(native_debug.contains("enemy_0D#04@0x03670000,0x01c00000"));
        assert!(native_debug.contains("platform_12:34@0x00120000,0x00340000"));
        assert!(native_debug.contains("block_bounce@0A,0B rem=3 y_off=-2"));
        assert!(native_debug.contains("floating_item#64@012,034 rem=5"));
        assert!(native_debug.contains("block_fx@006,007 rem=8 dir=1"));

        assert_eq!(oracle_object_debug(None), "-");
        assert_eq!(oracle_object_debug(Some(&oracle_state("frame_0"))), "-");
        let mut oracle = oracle_state("frame_0");
        oracle.active_enemies.push(oracle_enemy(0x0D));
        assert_eq!(
            oracle_object_debug(Some(&oracle)),
            "0x0D@0x03670000,0x01c00000 vel=0x00000000,0x00000000 face=Right",
        );
    }

    #[test]
    fn gameplay_debug_helpers_report_rng_and_named_fields() {
        assert_eq!(native_gameplay_debug(None), None);
        assert_eq!(
            native_gameplay_debug(Some(&NativeTraceFrame::default())),
            None
        );
        let native_rng_only = NativeTraceFrame {
            rng_offset: Some(0x0034),
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            native_gameplay_debug(Some(&native_rng_only)),
            Some("{\"rng_offset\": \"0x0034\"}".into())
        );
        let mut native_field_only_fields = std::collections::BTreeMap::new();
        native_field_only_fields.insert("kid_collision_type".into(), "0x0028(kidbelow)".into());
        let native_field_only = NativeTraceFrame {
            gameplay_debug_fields: native_field_only_fields,
            ..NativeTraceFrame::default()
        };
        assert_eq!(
            native_gameplay_debug(Some(&native_field_only)),
            Some("{\"kid_collision_type\": \"0x0028(kidbelow)\"}".into())
        );
        let mut native_fields = std::collections::BTreeMap::new();
        native_fields.insert("kid_collision_type".into(), "0x0028(kidbelow)".into());
        let native_frame = NativeTraceFrame {
            gameplay_debug_fields: native_fields,
            rng_offset: Some(0x0034),
            ..NativeTraceFrame::default()
        };
        let native_debug =
            native_gameplay_debug(Some(&native_frame)).expect("native debug should print");
        assert!(native_debug.contains("kid_collision_type"));
        assert!(native_debug.contains("0x0028(kidbelow)"));
        assert!(native_debug.contains("rng_offset"));
        assert!(native_debug.contains("0x0034"));
        assert_eq!(
            native_gameplay_field(Some(&native_frame), "kid_collision_type"),
            "0x0028(kidbelow)"
        );
        assert_eq!(native_gameplay_field(Some(&native_frame), "missing"), "-");
        assert_eq!(native_gameplay_field(None, "kid_collision_type"), "-");

        assert_eq!(oracle_gameplay_debug(None), None);
        assert_eq!(oracle_gameplay_debug(Some(&oracle_state("frame_0"))), None);
        let mut oracle = oracle_state("frame_0");
        oracle.rng_offset = Some(0x0012);
        oracle
            .gameplay_debug_fields
            .insert("kid_collision_source".into(), "0x000d".into());
        let oracle_debug = oracle_gameplay_debug(Some(&oracle)).expect("oracle debug should print");
        assert!(oracle_debug.contains("\"kid_collision_source\": \"0x000d(dragon)\""));
        assert!(oracle_debug.contains("\"rng_offset\": \"0x0012\""));
        assert_eq!(
            oracle_gameplay_field(Some(&oracle), "kid_collision_source"),
            "0x000d"
        );
        assert_eq!(oracle_gameplay_field(Some(&oracle), "missing"), "-");
        assert_eq!(oracle_gameplay_field(None, "kid_collision_source"), "-");
    }

    #[test]
    fn collision_source_labels_cover_enemy_form_prize_and_flagpole_ids() {
        let cases = [
            (0x00, "fire_demon"),
            (0x01, "diamond"),
            (0x03, "robot"),
            (0x04, "armadillo"),
            (0x05, "tar_monster"),
            (0x06, "sphere"),
            (0x07, "archer"),
            (0x08, "orca"),
            (0x09, "crab"),
            (0x0A, "rock_tank"),
            (0x0B, "rock_tank_shooting"),
            (0x0C, "dragon_flying"),
            (0x0D, "dragon"),
            (0x0E, "cloud"),
            (0x0F, "ufo"),
            (0x10, "goat"),
            (0x11, "ninja"),
            (0x12, "lion"),
            (0x13, "scorpion"),
            (0x14, "spinning_twins"),
            (0x16, "drip"),
            (0x17, "hand"),
            (0x18, "tornado"),
            (0x19, "fireball"),
            (0x1A, "driller"),
            (0x1B, "emo_rock"),
            (0x1C, "mini_hopping_skull"),
            (0x1D, "big_hopping_skull"),
            (0x1F, "invalid_1f"),
            (0x20, "heady_metal"),
            (0x22, "shiskaboss"),
            (0x23, "boomerang_bosses"),
            (0x24, "bagel_brothers"),
            (0x64, "prizediamond"),
            (0x6C, "skycutter"),
            (0x70, "cyclone"),
            (0x74, "redstealth"),
            (0x78, "eyeclops"),
            (0x7C, "juggernaut"),
            (0x80, "ironknight"),
            (0x84, "berzerker"),
            (0x88, "maniaxe"),
            (0x8C, "micromax"),
            (0x90, "ankh"),
            (0x94, "clock"),
            (0x98, "continue"),
            (0x9C, "unknown_prize"),
            (0xA0, "flagpole"),
        ];

        for (source, label) in cases {
            assert_eq!(collision_source_label(source), Some(label));
            assert_eq!(
                collision_source_label(source | 0x1200),
                Some(label),
                "collision source labels are keyed by the low byte"
            );
        }
        assert_eq!(collision_source_label(0x02), None);
        assert_eq!(collision_source_label(0xFF), None);
    }

    #[test]
    fn collision_source_value_formats_known_hex_and_leaves_raw_unknowns() {
        assert_eq!(format_collision_source_value("0X000D"), "0X000D(dragon)");
        assert_eq!(format_collision_source_value("0x12a0"), "0x12a0(flagpole)");
        assert_eq!(format_collision_source_value("13"), "13");
        assert_eq!(format_collision_source_value("0xzzzz"), "0xzzzz");
    }

    #[test]
    fn summarize_entries_limits_long_lists() {
        assert_eq!(summarize_entries(Vec::new()), "-");
        assert_eq!(
            summarize_entries(vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into(),
                "e".into(),
                "f".into(),
            ]),
            "a; b; c; d; e; f"
        );
        assert_eq!(
            summarize_entries(vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into(),
                "e".into(),
                "f".into(),
                "g".into(),
                "h".into(),
            ]),
            "a; b; c; d; e; f; +2 more"
        );
    }

    #[test]
    fn native_enemy_debug_includes_layout_motion_and_ground_state() {
        let debug = format_native_enemy_debug(
            0x0D,
            &NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x0D },
                x_pos: 0x035E_8000,
                y_pos: 0x0270_0000,
                layout_index: Some(4),
                x_velocity: Some(0),
                y_velocity: Some(0),
                facing: Some(Direction::Left),
                on_ground: Some(true),
                hit_points: Some(2),
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: Some(0x000a),
                shared_walker_patrol_timer: Some(0x0029),
                shared_walker_patrol_short_next: Some(false),
                walk_phase: Some(0),
                sprite_animation_phase: None,
                wait_timer: Some(0x01df),
                debug_fields: std::collections::BTreeMap::new(),
            },
        );

        assert_eq!(
            debug,
            "enemy_0D#04@0x035e8000,0x02700000 vel=0x00000000,0x00000000 face=Left ground=1 hp=2 walker=0x000a/0x0029/short=0 phase=0x00 wait=0x01df",
        );
    }

    #[test]
    fn oracle_enemy_debug_includes_velocity_and_facing() {
        let debug = format_oracle_enemy_debug(&crate::oracle::OracleEnemyState {
            object_type: 0x0D,
            x_pos: 0x0367_0000,
            y_pos: 0x01C0_0000,
            x_velocity: 0,
            y_velocity: 0,
            facing: crate::oracle::EnemyFacing::Right,
            collision_type: None,
            collision_source: None,
            layout_index: None,
            layout_ptr: None,
            parent_object_ptr: None,
            gfx_slot_index: None,
            on_ground: None,
            current_hp: None,
            debug_fields: std::collections::BTreeMap::new(),
        });

        assert_eq!(
            debug,
            "0x0D@0x03670000,0x01c00000 vel=0x00000000,0x00000000 face=Right",
        );
    }

    #[test]
    fn oracle_limitation_diffs_get_explicit_display_tag() {
        let diff = StateDiff {
            checkpoint: "frame_956".into(),
            field: "number_objects".into(),
            native_value: "10".into(),
            oracle_value: "8".into(),
            category: DivergenceCategory::OracleLimitation,
        };

        assert_eq!(diff_category_tag(&diff), " (oracle limitation)");
    }

    #[test]
    fn oracle_enemy_debug_labels_kid_gfx_slot_and_decodes_collision_source() {
        let mut debug_fields = std::collections::BTreeMap::new();
        debug_fields.insert("collision_source".into(), "0x00a0".into());
        let debug = format_oracle_enemy_debug(&crate::oracle::OracleEnemyState {
            object_type: 0x30,
            x_pos: 0x03A8_0000,
            y_pos: 0x01BF_0000,
            x_velocity: 0,
            y_velocity: -0x0000_4000,
            facing: crate::oracle::EnemyFacing::Left,
            collision_type: None,
            collision_source: None,
            layout_index: None,
            layout_ptr: Some(0x0000_75FA),
            parent_object_ptr: Some(0xFFFF_0E88),
            gfx_slot_index: Some(2),
            on_ground: None,
            current_hp: None,
            debug_fields,
        });

        assert!(debug.contains("0x30(kid_gfx)@"));
        assert!(debug.contains("\"collision_source\": \"0x00a0(flagpole)\""));
    }

    #[test]
    fn oracle_gameplay_debug_reports_collision_fields() {
        let mut gameplay_debug_fields = std::collections::BTreeMap::new();
        gameplay_debug_fields.insert("kid_collision_source".into(), "0x000d".into());
        gameplay_debug_fields.insert("kid_collision_type".into(), "0x0028(kidbelow)".into());
        let state = crate::oracle::OracleState {
            label: "frame_3400".into(),
            frame_index: 3400,
            game_mode: 0x0010,
            boot_phase: Some("demo_play".into()),
            boot_phase_elapsed: None,
            title_screen_ready: false,
            main_menu_option: None,
            current_player: 0,
            number_hitpoints: 5,
            current_helmet_id: 6,
            current_level_id: 0x15,
            player_1_level_id: 0x15,
            camera_x_pos: 0,
            camera_y_pos: 0,
            kid_x_pos: 0,
            kid_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            game_paused: false,
            active_enemies: Vec::new(),
            score: 0,
            number_lives: 0,
            number_diamonds: 0,
            diamond_power_active: false,
            murder_wall_x: None,
            kid_x_vel: None,
            kid_y_vel: None,
            character_movement: None,
            just_received_damage: None,
            kid_collision_type: None,
            kid_collision_source: None,
            kid_x_direction: None,
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
            rng_last_byte: None,
            rng_ring: None,
            level_special_effects: None,
            gameplay_debug_fields,
            cpu_registers: None,
        };

        let debug = oracle_gameplay_debug(Some(&state))
            .unwrap_or_else(|| panic!("oracle gameplay debug should be printed when present"));
        assert!(
            debug.contains("kid_collision_type"),
            "the gameplay summary should surface the oracle Kid collision classification",
        );
        assert!(
            debug.contains("kid_collision_source"),
            "the gameplay summary should surface the raw oracle collision source word",
        );
        assert!(
            debug.contains("0x000d(dragon)"),
            "the gameplay summary should decode known collision source ids",
        );
    }

    #[test]
    fn collision_source_label_decodes_flagpole() {
        assert_eq!(collision_source_label(0x00A0), Some("flagpole"));
    }

    #[test]
    fn format_collision_source_value_leaves_unknown_ids_raw() {
        assert_eq!(format_collision_source_value("0x1234"), "0x1234");
    }

    #[test]
    fn match_frames_only_print_object_roster_when_requested() {
        let native_frame = NativeTraceFrame {
            frame_index: 0,
            game_mode: GameMode::TitleScreen,
            game_mode_value: GameMode::TitleScreen.as_disassembly_value(),
            time_frames_loc_3038a: 0,
            start_press_count: 0,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: None,
            player_start_x: None,
            player_start_y: None,
            flag_x: None,
            flag_y: None,
            level_enemy_object_count: None,
            active_enemy_object_count: 1,
            active_platform_object_count: 0,
            number_lives: 0,
            number_continues: 0,
            number_hitpoints: 0,
            extra_hitpoint_slots: 0,
            current_helmet_id: 0,
            red_stealth_sword_timer: 0,
            red_stealth_airborne_slash_counter: 0,
            cyclone_flying: false,
            paused: false,
            pause_menu_option: 0,
            kid_x_pos: None,
            kid_y_pos: None,
            kid_x_velocity: None,
            kid_y_velocity: None,
            character_movement: None,
            kid_facing: None,
            kid_on_ground: Some(false),
            walk_visual_accumulator: 0,
            iron_knight_climbing: false,
            iron_knight_climb_pose_accum: 0,
            hud_diamond_frame_offset: None,
            hud_timer_sprite_seconds_loc_bee8: None,
            hud_life_frame_offset: None,
            berzerker_charging: false,
            berzerker_charge_pose: false,
            berzerker_walk_pose: false,
            ledge_snap_cooldown: 0,
            kid_on_slope: 0,
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            score: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 1,
            number_gfx_objects: 0,
            rng_offset: None,
            rng_last_byte: None,
            rng_ring: None,
            level_complete: false,
            level_complete_elapsed_frames: 0,
            level_complete_score_rows_elapsed_frames: 0,
            level_complete_score_counter_elapsed_frames: 0,
            level_complete_score_counter_drained: false,
            level_complete_initial_score: 0,
            level_complete_score_rows_started: 0,
            level_completion_time: 0,
            level_skip_cheat_enabled: 0,
            game_over: false,
            game_over_elapsed_frames: 0,
            continue_countdown: None,
            continue_countdown_elapsed_frames: 0,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            played_sound_effect_ids: Vec::new(),
            played_sound_commands: Vec::new(),
            played_bgm_ids: Vec::new(),
            played_music_speed_updates: Vec::new(),
            teleport_warp_colour: 0,
            level_entry_fade_progress_sub_73d0: None,
            sound_test_track: None,
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            invulnerable_frames: 0,
            kid_is_invulnerable: false,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            level_timer_seconds: 0,
            level_timer_sub_frames: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            diamond_power_sprites: Vec::new(),
            eyeclops_hard_beam_object: None,
            active_geysers: Vec::new(),
            lava_geyser_script_templates_loc_147ba: Vec::new(),
            island_shore_cycle_active: false,
            island_shore_cycle_state_loc_1202a: None,
            island_shore_vram_frames_loc_1202a: None,
            island_shore_dma_uploads_loc_1202a: Vec::new(),
            lava_geyser_art_uploads_loc_14c6a: Vec::new(),
            rotate_palette_lava_water_state_loc_1223c: None,
            lava_water_palette_words_loc_1223c: None,
            lava_geyser_palette_words_loc_14b2a: None,
            storm_pattern_upload_loc_14d32: None,
            storm_scroll_transfer_loc_30110: None,
            storm_shared_art_upload_loc_14d8a: None,
            special_effect_unknown_art_upload_loc_14df6: None,
            special_effect_unknown_plane_fill_loc_14e42: None,
            storm_falling_slots_sub_61ca: Vec::new(),
            storm_impact_slots_sub_61ca: Vec::new(),
            storm_sprites_sub_61ca: Vec::new(),
            gameplay_debug_fields: std::collections::BTreeMap::new(),
            active_objects: vec![NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy { object_type: 0x0D },
                x_pos: 0x0367_0000,
                y_pos: 0x01C0_0000,
                layout_index: Some(4),
                x_velocity: Some(0),
                y_velocity: Some(0),
                facing: Some(Direction::Right),
                on_ground: Some(true),
                hit_points: Some(2),
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                contact_phase: None,
                shared_walker_state: Some(0x000a),
                shared_walker_patrol_timer: Some(0x0029),
                shared_walker_patrol_short_next: Some(false),
                walk_phase: Some(0),
                sprite_animation_phase: None,
                wait_timer: Some(0x01df),
                debug_fields: std::collections::BTreeMap::new(),
            }],
            active_enemy_details: Vec::new(),
            block_bounces: Vec::new(),
            block_bounce_details: Vec::new(),
            floating_items: Vec::new(),
            clock_prize_animation_swaps_loc_10796: Vec::new(),
            continue_prize_animation_swaps_loc_1080a: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_pieces: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            kid_projectile_hit_sparks: Vec::new(),
            enemy_death_effects: Vec::new(),
            heady_metal_death_splits: Vec::new(),
            heady_metal_body_pieces_sub_37abc: Vec::new(),
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            physics_trace: None,
        };

        assert!(
            !should_print_object_roster(false, false, Some(&native_frame), None),
            "match frames should stay compact unless the caller opts into match-roster output",
        );
        assert!(
            should_print_object_roster(false, true, Some(&native_frame), None),
            "match frames should expose object rosters when the caller is tracing clean history",
        );
    }
}
