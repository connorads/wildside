use super::*;
use scenarios::{prize_block_scenario, stomp_enemy_scenario, walk_right_scenario};

pub(crate) fn run_gameplay_sweep(manifest: &AssetManifest, levels: &[u8]) -> SweepReport {
    let timestamp = chrono_timestamp();
    let mut results = Vec::with_capacity(levels.len());

    for &level_id in levels {
        println!("gameplay sweep level {level_id:#04x}...");
        let r = sweep_gameplay_single_level(manifest, level_id);
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

    report
}

fn sweep_gameplay_single_level(manifest: &AssetManifest, level_id: u8) -> LevelSweepResult {
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

    LevelSweepResult {
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
    }
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

    diffs.extend(check_walk_facing_consistency(
        trace, scenario, settle_idx, end_idx,
    ));

    diffs
}

/// Self-consistency assertion: while the script holds Right (and not Left)
/// and Kid is actually moving (character_movement != 0), the rendered
/// facing must be Right. ASM `loc_9E88: sf x_direction` (SlopeDown -> Right)
/// vs `loc_9EA4: st x_direction` (SlopeUp -> Left): bug #137 ("moonwalking
/// up slopes") flips the byte while velocity and position still progress
/// rightward, so position-only checks miss it. This catches the bug across
/// all 105 sweep levels without needing the oracle to be available.
fn check_walk_facing_consistency(
    trace: &[NativeTraceFrame],
    scenario: &DynamicScenario,
    settle_idx: usize,
    end_idx: usize,
) -> Vec<StateDiff> {
    use kid_core::physics::Direction;

    const RIGHT_BIT: u8 = 1 << 3;
    const LEFT_BIT: u8 = 1 << 2;

    let mut first_inversion_frame: Option<usize> = None;
    let mut inversion_count: usize = 0;

    for frame_index in settle_idx..=end_idx {
        let buttons = scenario.script.get(frame_index).copied().unwrap_or(0);
        if buttons & RIGHT_BIT == 0 || buttons & LEFT_BIT != 0 {
            continue;
        }
        let Some(frame) = trace.get(frame_index) else {
            continue;
        };
        // Only check while Kid is actually moving — a paused or knocked-back
        // frame can legitimately keep facing without contradicting the input.
        if frame.character_movement.unwrap_or(0) == 0 {
            continue;
        }
        if matches!(frame.kid_facing, Some(Direction::Left)) {
            inversion_count += 1;
            if first_inversion_frame.is_none() {
                first_inversion_frame = Some(frame_index);
            }
        }
    }

    if let Some(frame_index) = first_inversion_frame {
        return vec![StateDiff {
            checkpoint: format!("frame_{frame_index}"),
            field: "kid_facing_matches_input".into(),
            native_value: format!("kid_facing=Left while Right held ({inversion_count} frames)"),
            oracle_value: "kid_facing=Right (matches held direction)".into(),
            category: DivergenceCategory::Authoritative,
        }];
    }
    Vec::new()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenarios::walk_right_scenario;
    use kid_core::physics::Direction;

    fn moving_frame_facing(frame_index: usize, facing: Direction) -> NativeTraceFrame {
        NativeTraceFrame {
            frame_index,
            kid_x_pos: Some(0x0010_0000 + (frame_index as u32) * 0x0001_0000),
            kid_y_pos: Some(0x0010_0000),
            character_movement: Some(2), // walking
            kid_facing: Some(facing),
            number_hitpoints: 3,
            ..NativeTraceFrame::default()
        }
    }

    #[test]
    fn walk_right_assertions_flag_moonwalk_inversion() {
        // Regression for backlog #137: position is monotonically increasing
        // (not a kid_x_pos_monotonic miss) but kid_facing is Left while the
        // script holds Right. The new facing-consistency check must surface
        // a single authoritative diff so the bug stops being invisible to
        // the gameplay sweep.
        let scenario = walk_right_scenario(0x00, 300);
        let trace: Vec<NativeTraceFrame> = (0..scenario.frame_count)
            .map(|i| {
                let facing = if (100..=130).contains(&i) {
                    Direction::Left // moonwalk window
                } else {
                    Direction::Right
                };
                moving_frame_facing(i, facing)
            })
            .collect();

        let diffs = check_walk_right_assertions(&trace, &scenario);

        // No position diff (monotonic), but exactly one facing diff.
        assert!(
            diffs.iter().all(|d| d.field != "kid_x_pos_monotonic"),
            "position should still be monotonic: {diffs:?}"
        );
        let facing_diffs: Vec<_> = diffs
            .iter()
            .filter(|d| d.field == "kid_facing_matches_input")
            .collect();
        assert_eq!(facing_diffs.len(), 1, "expected exactly one facing diff");
        assert!(
            matches!(facing_diffs[0].category, DivergenceCategory::Authoritative),
            "facing diff must be authoritative"
        );
    }

    #[test]
    fn walk_right_assertions_pass_when_facing_tracks_input() {
        let scenario = walk_right_scenario(0x00, 300);
        let trace: Vec<NativeTraceFrame> = (0..scenario.frame_count)
            .map(|i| moving_frame_facing(i, Direction::Right))
            .collect();

        let diffs = check_walk_right_assertions(&trace, &scenario);
        assert!(
            diffs.iter().all(|d| d.field != "kid_facing_matches_input"),
            "facing in sync with input must produce no facing diff: {diffs:?}"
        );
    }

    #[test]
    fn walk_right_assertions_ignore_facing_when_not_moving() {
        // ORIGINAL_QUIRK: enemy hits / pause windows can keep kid_facing as
        // Left while character_movement reads Standingstill (0). Those frames
        // are not moonwalk evidence and must not produce false positives.
        let scenario = walk_right_scenario(0x00, 300);
        let trace: Vec<NativeTraceFrame> = (0..scenario.frame_count)
            .map(|i| NativeTraceFrame {
                frame_index: i,
                kid_x_pos: Some(0x0010_0000 + (i as u32) * 0x0001_0000),
                kid_y_pos: Some(0x0010_0000),
                character_movement: Some(0), // standingstill — not moving
                kid_facing: Some(Direction::Left),
                number_hitpoints: 3,
                ..NativeTraceFrame::default()
            })
            .collect();

        let diffs = check_walk_right_assertions(&trace, &scenario);
        assert!(
            diffs.iter().all(|d| d.field != "kid_facing_matches_input"),
            "non-moving frames must not produce facing diffs: {diffs:?}"
        );
    }
}
