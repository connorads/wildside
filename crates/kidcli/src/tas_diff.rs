use anyhow::{Context, Result};
use std::path::Path;

use crate::oracle::OracleState;
use crate::scenarios::{DynamicCheckpoint, DynamicScenario};
use crate::sweep::{
    ADVISORY_FIELDS, FrontierKind, compare_sweep_states_for_frontier, extract_checkpoint_states,
};

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
    show_object_roster_on_match: bool,
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
    let diffs = compare_sweep_states_for_frontier(&native_states, &oracle.states, frontier_kind);
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
        let auth_count = diffs
            .iter()
            .filter(|d| !ADVISORY_FIELDS.contains(&d.field.as_str()))
            .count();
        let adv_count = diffs.len() - auth_count;

        if auth_count > 0 {
            println!(
                "frame {frame_idx}: \x1b[31mDIVERGE\x1b[0m ({auth_count} authoritative, {adv_count} advisory)"
            );
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
            auth_count > 0,
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

fn checkpoint_state<'a>(states: &'a [OracleState], label: &str) -> Option<&'a OracleState> {
    states.iter().find(|state| state.label == label)
}

fn should_print_object_roster(
    has_authoritative_diff: bool,
    show_on_match: bool,
    native_frame: Option<&kid_core::NativeTraceFrame>,
    oracle_state: Option<&OracleState>,
) -> bool {
    has_authoritative_diff
        || (show_on_match && native_object_debug_relevant(native_frame, oracle_state))
}

fn native_object_debug_relevant(
    native_frame: Option<&kid_core::NativeTraceFrame>,
    oracle_state: Option<&OracleState>,
) -> bool {
    native_frame.is_some_and(|frame| {
        !frame.active_objects.is_empty()
            || !frame.block_bounces.is_empty()
            || !frame.block_destroy_effects.is_empty()
            || !frame.floating_items.is_empty()
    }) || oracle_state.is_some_and(|state| !state.active_enemies.is_empty())
}

fn native_object_debug(native_frame: Option<&kid_core::NativeTraceFrame>) -> String {
    let Some(frame) = native_frame else {
        return "-".to_owned();
    };

    let mut entries = Vec::new();
    for object in &frame.active_objects {
        match object.kind {
            kid_core::NativeObjectKind::Player => {}
            kid_core::NativeObjectKind::Enemy { object_type } => {
                entries.push(format_native_enemy_debug(object_type, object))
            }
            kid_core::NativeObjectKind::Platform { kind, subtype } => entries.push(format!(
                "platform_{kind:02X}:{subtype:02X}@{:#010x},{:#010x}",
                object.x_pos, object.y_pos
            )),
        }
    }
    for &(cell_x, cell_y, remaining) in &frame.block_bounces {
        entries.push(format!(
            "block_bounce@{cell_x:02X},{cell_y:02X} rem={remaining}"
        ));
    }
    for &(x_px, y_px, remaining, content_id) in &frame.floating_items {
        entries.push(format!(
            "floating_item#{content_id:02X}@{x_px:03},{y_px:03} rem={remaining}"
        ));
    }
    for &(x_px, y_px, remaining) in &frame.block_destroy_effects {
        entries.push(format!("block_fx@{x_px:03},{y_px:03} rem={remaining}"));
    }

    summarize_entries(entries)
}

fn native_gameplay_debug(native_frame: Option<&kid_core::NativeTraceFrame>) -> Option<String> {
    let frame = native_frame?;
    if frame.gameplay_debug_fields.is_empty() {
        return None;
    }
    Some(format!("{:?}", frame.gameplay_debug_fields))
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
    if state.gameplay_debug_fields.is_empty() {
        return None;
    }
    Some(format!("{:?}", state.gameplay_debug_fields))
}

fn format_native_enemy_debug(object_type: u8, object: &kid_core::NativeObjectSnapshot) -> String {
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
            let short_next = short_next.map(u8::from).unwrap_or(0);
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
        format!(" debug={:?}", object.debug_fields)
    };
    format!(
        "0x{:02X}@{:#010x},{:#010x} vel={:#010x},{:#010x} face={:?}{layout_index}{layout_ptr}{parent_ptr}{gfx_slot}{on_ground}{debug_fields}",
        object.object_type,
        object.x_pos,
        object.y_pos,
        object.x_velocity as u32,
        object.y_velocity as u32,
        object.facing
    )
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
        format_native_enemy_debug, format_oracle_enemy_debug, oracle_gameplay_debug,
        should_print_object_roster,
    };
    use kid_core::{
        GameMode, LevelId, NativeObjectKind, NativeObjectSnapshot, NativeTraceFrame,
        physics::Direction,
    };

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
                shared_walker_state: Some(0x000a),
                shared_walker_patrol_timer: Some(0x0029),
                shared_walker_patrol_short_next: Some(false),
                walk_phase: Some(0),
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
            layout_index: None,
            layout_ptr: None,
            parent_object_ptr: None,
            gfx_slot_index: None,
            on_ground: None,
            debug_fields: std::collections::BTreeMap::new(),
        });

        assert_eq!(
            debug,
            "0x0D@0x03670000,0x01c00000 vel=0x00000000,0x00000000 face=Right",
        );
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
            invulnerable_frames: None,
            currently_transforming: None,
            rng_offset: None,
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
    }

    #[test]
    fn match_frames_only_print_object_roster_when_requested() {
        let native_frame = NativeTraceFrame {
            frame_index: 0,
            game_mode: GameMode::TitleScreen,
            game_mode_value: GameMode::TitleScreen.as_disassembly_value(),
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
            current_helmet_id: 0,
            paused: false,
            kid_x_pos: None,
            kid_y_pos: None,
            kid_x_velocity: None,
            kid_y_velocity: None,
            kid_facing: None,
            kid_on_ground: Some(false),
            form_change_flash_frames: 0,
            score: 0,
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 1,
            number_gfx_objects: 0,
            level_complete: false,
            game_over: false,
            continue_countdown: None,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            sound_test_track: None,
            two_player: false,
            invulnerable_frames: 0,
            diamond_power_active: false,
            diamond_count: 0,
            level_timer_seconds: 0,
            level_damage_taken: false,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_geysers: Vec::new(),
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
                shared_walker_state: Some(0x000a),
                shared_walker_patrol_timer: Some(0x0029),
                shared_walker_patrol_short_next: Some(false),
                walk_phase: Some(0),
                wait_timer: Some(0x01df),
                debug_fields: std::collections::BTreeMap::new(),
            }],
            block_bounces: Vec::new(),
            floating_items: Vec::new(),
            block_destroy_effects: Vec::new(),
            enemy_death_effects: Vec::new(),
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
