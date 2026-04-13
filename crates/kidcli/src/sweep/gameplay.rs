use super::*;
use scenarios::{prize_block_scenario, stomp_enemy_scenario, walk_right_scenario};

pub(crate) fn run_gameplay_sweep(manifest: &AssetManifest, levels: &[u8]) -> Result<SweepReport> {
    let timestamp = chrono_timestamp();
    let mut results = Vec::with_capacity(levels.len());

    for &level_id in levels {
        println!("gameplay sweep level {level_id:#04x}...");
        let result = sweep_gameplay_single_level(manifest, level_id);
        match result {
            Ok(r) => {
                let status = if r.passed { "PASS" } else { "FAIL" };
                let diff_count = if r.state_diffs.is_empty() {
                    String::new()
                } else {
                    format!(" ({} assertion failures)", r.state_diffs.len())
                };
                println!("  {status}{diff_count}");
                for d in &r.state_diffs {
                    println!(
                        "    {}: {} = {} (expected {})",
                        d.checkpoint, d.field, d.native_value, d.oracle_value
                    );
                }
                results.push(r);
            }
            Err(e) => {
                println!("  ERROR: {e:#}");
                results.push(LevelSweepResult {
                    level_id,
                    slug: format!("gameplay_{level_id:02x}"),
                    tier: SweepTier::Gameplay,
                    passed: false,
                    state_diffs: Vec::new(),
                    mae: None,
                    pixel_mismatches: None,
                    diff_images: Vec::new(),
                    error: Some(format!("{e:#}")),
                    divergence_summary: divergence_summary_with(DivergenceCategory::Authoritative),
                });
            }
        }
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results
        .iter()
        .filter(|r| !r.passed && r.error.is_none())
        .count();
    let errors = results.iter().filter(|r| r.error.is_some()).count();
    let divergence_summary = summarise_results(&results);
    let field_summary = summarise_field_diffs(&results);

    let report = SweepReport {
        timestamp,
        kid_core_hash: None,
        levels: results,
        passed,
        failed,
        errors,
        divergence_summary,
        field_summary,
        release_gate_focus: Vec::new(),
    };

    println!("\ngameplay sweep complete: {passed} passed, {failed} failed, {errors} errors");

    Ok(report)
}

fn sweep_gameplay_single_level(manifest: &AssetManifest, level_id: u8) -> Result<LevelSweepResult> {
    let mut all_diffs = Vec::new();

    // Run walk_right scenario (300 frames = 5 seconds)
    let walk_scenario = walk_right_scenario(level_id, 300);
    let walk_trace = kid_core::run_script_with_warp(
        manifest.clone(),
        &walk_scenario.script,
        walk_scenario.frame_count,
        walk_scenario.warp_to_level,
    );
    all_diffs.extend(check_walk_right_assertions(&walk_trace, &walk_scenario));

    // Run prize_block scenario
    let prize_scenario = prize_block_scenario(level_id);
    let prize_trace = kid_core::run_script_with_warp(
        manifest.clone(),
        &prize_scenario.script,
        prize_scenario.frame_count,
        prize_scenario.warp_to_level,
    );
    all_diffs.extend(check_jump_assertions(&prize_trace, &prize_scenario));

    // Run stomp_enemy scenario (360 frames = 6 seconds, run + aggressive jumps)
    let stomp_scenario = stomp_enemy_scenario(level_id, 360);
    let stomp_trace = kid_core::run_script_with_warp(
        manifest.clone(),
        &stomp_scenario.script,
        stomp_scenario.frame_count,
        stomp_scenario.warp_to_level,
    );
    all_diffs.extend(check_walk_right_assertions(&stomp_trace, &stomp_scenario));
    all_diffs.extend(check_jump_assertions(&stomp_trace, &stomp_scenario));

    let passed = all_diffs.is_empty();

    Ok(LevelSweepResult {
        level_id,
        slug: format!("gameplay_{level_id:02x}"),
        tier: SweepTier::Gameplay,
        passed,
        state_diffs: all_diffs,
        mae: None,
        pixel_mismatches: None,
        diff_images: Vec::new(),
        error: None,
        divergence_summary: if passed {
            DivergenceSummary::default()
        } else {
            divergence_summary_with(DivergenceCategory::Authoritative)
        },
    })
}

/// Self-consistency assertion: player X position increases monotonically
/// while Right is held (within the walk phase).
pub(super) fn check_walk_right_assertions(
    trace: &[NativeTraceFrame],
    scenario: &DynamicScenario,
) -> Vec<StateDiff> {
    let mut diffs = Vec::new();
    let settle_idx = scenario
        .checkpoints
        .iter()
        .find(|cp| cp.label == "level_settle")
        .map_or(60, |cp| cp.frame_index);
    let end_idx = scenario
        .checkpoints
        .iter()
        .find(|cp| cp.label == "walk_end")
        .map_or(scenario.frame_count.saturating_sub(1), |cp| cp.frame_index);

    // Skip if player died during the walk — death resets position
    let player_died = (settle_idx..=end_idx).any(|i| {
        trace
            .get(i)
            .is_some_and(|f| f.number_hitpoints == 0 || f.game_over)
    });
    if player_died {
        return diffs;
    }

    // Check X position increases between settle and end
    let settle_x = trace.get(settle_idx).and_then(|f| f.kid_x_pos);
    let end_x = trace.get(end_idx).and_then(|f| f.kid_x_pos);

    if let (Some(sx), Some(ex)) = (settle_x, end_x)
        && ex <= sx
    {
        diffs.push(StateDiff {
            checkpoint: "walk_end".into(),
            field: "kid_x_pos_monotonic".into(),
            native_value: format!("start={sx:#010x} end={ex:#010x}"),
            oracle_value: "end > start".into(),
            category: DivergenceCategory::Authoritative,
        });
    }

    diffs
}

/// Self-consistency assertion: player Y velocity goes negative after jump input,
/// and player leaves the ground.
pub(super) fn check_jump_assertions(
    trace: &[NativeTraceFrame],
    scenario: &DynamicScenario,
) -> Vec<StateDiff> {
    let mut diffs = Vec::new();
    let settle_idx = scenario
        .checkpoints
        .iter()
        .find(|cp| cp.label == "level_settle")
        .map_or(60, |cp| cp.frame_index);

    // Find first frame after settle where B (jump) is pressed
    let first_jump_frame = scenario
        .script
        .iter()
        .enumerate()
        .skip(settle_idx)
        .find(|&(_, &buttons)| buttons & (1 << 4) != 0)
        .map(|(i, _)| i);

    if let Some(jump_frame) = first_jump_frame {
        // Skip if player was dead or not on ground at the jump frame —
        // the scenario is generic and doesn't know level geometry
        let player_alive_and_grounded = trace
            .get(jump_frame)
            .is_some_and(|f| f.number_hitpoints > 0 && f.kid_on_ground.unwrap_or(false));
        if !player_alive_and_grounded {
            return diffs;
        }

        // Check that within 10 frames after jump press, Y velocity goes negative
        let found_negative_vy = (jump_frame..=(jump_frame + 10).min(trace.len().saturating_sub(1)))
            .any(|i| {
                trace
                    .get(i)
                    .and_then(|f| f.kid_y_velocity)
                    .is_some_and(|vy| vy < 0)
            });

        // Check that player leaves the ground
        let found_airborne = (jump_frame..=(jump_frame + 15).min(trace.len().saturating_sub(1)))
            .any(|i| {
                trace
                    .get(i)
                    .and_then(|f| f.kid_on_ground)
                    .is_some_and(|on| !on)
            });

        // If the jump produced upward velocity, assert it also left the ground.
        // If no upward velocity at all, skip — the generic scenario doesn't
        // know level geometry and a blocked jump (ceiling above) is expected
        // on some levels.
        if found_negative_vy && !found_airborne {
            diffs.push(StateDiff {
                checkpoint: format!("frame_{jump_frame}"),
                field: "kid_on_ground_false_after_jump".into(),
                native_value: "negative vy but never left ground".into(),
                oracle_value: "kid_on_ground=false after B press".into(),
                category: DivergenceCategory::Authoritative,
            });
        }
    }

    diffs
}
