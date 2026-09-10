use std::cell::RefCell;
use std::collections::BTreeMap;

use super::runner::Runner;
use super::*;
use crate::oracle::OracleState;
use crate::scenarios::DynamicScenario;

fn state(index: usize) -> OracleState {
    serde_json::from_value(serde_json::json!({
        "label": format!("frame_{index}"), "frame_index": index,
        "game_mode": 12, "current_player": 0, "number_hitpoints": 2,
        "current_helmet_id": 0, "current_level_id": 54, "player_1_level_id": 54,
        "camera_x_pos": 0, "camera_y_pos": 0, "kid_x_pos": 0, "kid_y_pos": 0,
        "number_objects": 1, "number_gfx_objects": 1, "active_enemies": [],
        "kid_x_vel": 0, "kid_y_vel": 0, "character_movement": 0,
        "kid_x_direction": 0, "rng_offset": 65535, "rng_ring": vec![0; 55],
    }))
    .expect("oracle state fixture")
}

fn key(x: i32) -> SituationKey {
    let mut result = situation(&state(0)).expect("complete key");
    result.x_bucket = x;
    result
}

fn evidence(keys: Vec<SituationKey>) -> Evidence {
    Evidence {
        keys,
        differences: vec![],
        crc_mismatch: None,
        rng_mismatch: None,
        invalid: None,
        signature: None,
    }
}

#[test]
fn archive_retains_shortest_prefix_and_selects_least_visited_cells() {
    let mut archive = Archive::default();
    archive.insert(key(0), &[8; 80], "first");
    archive.insert(key(0), &[4; 80], "tie");
    assert_eq!(archive.0[&key(0)].source, "first");
    archive.insert(key(0), &[4; 70], "shorter");
    archive.insert(key(1), &[0; PREFIX], "second");
    let mut choices = Choices(1);
    let first = archive.select(&mut choices);
    let second = archive.select(&mut choices);
    assert_ne!(first.source, second.source);
    assert_eq!(archive.0[&key(0)].script, vec![4; 70]);
    assert!(archive.0.values().all(|entry| entry.selected == 1));
}

#[test]
fn situation_uses_whole_pixel_buckets_and_requires_predictive_fields() {
    let mut sample = state(0);
    sample.kid_x_pos = (31 << 16) | 65535;
    sample.kid_y_pos = u32::MAX;
    let result = situation(&sample).unwrap();
    assert_eq!((result.x_bucket, result.y_bucket), (1, -1));
    sample.character_movement = None;
    assert!(situation(&sample).is_err());
    sample.character_movement = Some(0);
    sample.kid_x_vel = None;
    assert!(situation(&sample).is_err());
}

#[test]
fn admission_requires_all_frames_rng_and_checkpoint_alignment() {
    // ASM Init_RNG / Get_RandomNumber_byte: all 55 ring bytes and the offset
    // are required evidence, including when every player/camera CRC matches.
    let states = (0..FRAMES).map(state).collect::<Vec<_>>();
    let crcs = vec![0; FRAMES];
    let mut rng = states
        .iter()
        .map(|s| {
            wildside_core::RomRandomFrameState::from_initialised_parts(
                s.rng_offset.unwrap(),
                s.rng_ring.clone().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert!(assess(&states, &states, &crcs, &crcs, &rng).accepted());
    assert!(!assess(&states[..FRAMES - 1], &states, &crcs, &crcs, &rng).accepted());
    assert!(!assess(&states, &states, &crcs[..FRAMES - 1], &crcs, &rng).accepted());
    let mut reordered = states.clone();
    reordered.swap(1, 2);
    assert!(!assess(&states, &reordered, &crcs, &crcs, &rng).accepted());
    rng[10] = None;
    assert!(!assess(&states, &states, &crcs, &crcs, &rng).accepted());
}

#[test]
fn advisory_or_oracle_limitation_and_crc_drift_never_add_coverage() {
    let native = (0..FRAMES).map(state).collect::<Vec<_>>();
    let mut oracle = native.clone();
    let crcs = vec![0; FRAMES];
    let rng = native
        .iter()
        .map(|s| {
            wildside_core::RomRandomFrameState::from_initialised_parts(
                s.rng_offset.unwrap(),
                s.rng_ring.clone().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    oracle[0].game_paused = true;
    oracle[0].kid_x_pos = 100 << 16;
    let result = assess(&native, &oracle, &crcs, &crcs, &rng);
    assert!(!result.accepted());
    assert!(!result.differences.is_empty());
    let mut changed = crcs.clone();
    changed[10] = 1;
    let result = assess(&native, &native, &crcs, &changed, &rng);
    assert_eq!(result.crc_mismatch, Some(10));
    assert!(!result.accepted());
}

#[test]
fn neutral_coverage_is_removed_from_both_metrics() {
    let neutral = Coverage {
        situations: BTreeSet::from([key(0)]),
        behaviours: BTreeSet::from([key(0).behaviour]),
        accepted: 1,
    };
    let mut coverage = Coverage::default();
    coverage.admit(&vec![key(1); FRAMES], &neutral);
    assert_eq!(coverage.situations.len(), 1);
    assert!(coverage.behaviours.is_empty());
}

struct FakeRunner {
    calls: Vec<(String, Vec<u8>)>,
    fail_at: Option<usize>,
    divergent_control: bool,
    limit: usize,
    saved: RefCell<BTreeMap<String, serde_json::Value>>,
}

impl FakeRunner {
    fn new(limit: usize) -> Self {
        Self {
            calls: vec![],
            fail_at: None,
            divergent_control: false,
            limit,
            saved: RefCell::default(),
        }
    }
}

impl Runner for FakeRunner {
    fn evaluate(&mut self, scenario: &DynamicScenario, name: &str) -> Result<Evidence> {
        self.calls.push((name.to_owned(), scenario.script.clone()));
        ensure!(
            self.fail_at != Some(self.calls.len()),
            "injected oracle failure"
        );
        let keys = scenario
            .script
            .iter()
            .map(|mask| key(i32::from(*mask)))
            .collect();
        let mut result = evidence(keys);
        if self.divergent_control {
            result.rng_mismatch = Some(0);
            result.signature = Some("rng_stream".to_owned());
        }
        Ok(result)
    }
    fn exploration_time_left(&self) -> bool {
        self.calls.len() < self.limit
    }
    fn save(&self, name: &str, value: &impl Serialize) -> Result<()> {
        self.saved
            .borrow_mut()
            .insert(name.to_owned(), serde_json::to_value(value)?);
        Ok(())
    }
}

#[test]
fn paired_experiment_is_reproducible_and_charges_complete_replays() -> Result<()> {
    let mut first = FakeRunner::new(usize::MAX);
    let mut second = FakeRunner::new(usize::MAX);
    run_comparison(&mut first)?;
    run_comparison(&mut second)?;
    assert_eq!(first.calls, second.calls);
    assert_eq!(first.calls.len(), 1 + 3 * 2 * CANDIDATES);
    assert!(first.calls.iter().all(|(_, script)| script.len() == FRAMES));
    assert!(
        first
            .calls
            .iter()
            .all(|(_, script)| script[..PREFIX] == [0; PREFIX])
    );
    let summary = &first.saved.borrow()["summary"];
    assert_eq!(summary["completed_pairs"], serde_json::json!([40, 40, 40]));
    assert_eq!(summary["verdict"], "Stop");
    Ok(())
}

#[test]
fn failed_second_execution_cannot_admit_a_half_pair() -> Result<()> {
    let mut runner = FakeRunner::new(usize::MAX);
    runner.fail_at = Some(3);
    run_comparison(&mut runner)?;
    let summary = &runner.saved.borrow()["summary"];
    assert_eq!(summary["completed_pairs"], serde_json::json!([0, 0, 0]));
    assert_eq!(summary["paired_coverage"][0][0]["accepted"], 0);
    assert_eq!(summary["verdict"], "Inconclusive");
    Ok(())
}

#[test]
fn deadline_stops_before_starting_another_pair() -> Result<()> {
    let mut runner = FakeRunner::new(3);
    run_comparison(&mut runner)?;
    assert_eq!(runner.calls.len(), 3);
    let summary = &runner.saved.borrow()["summary"];
    assert_eq!(summary["completed_pairs"], serde_json::json!([1, 0, 0]));
    assert_eq!(summary["verdict"], "Inconclusive");
    Ok(())
}

#[test]
fn divergent_control_is_reproduced_without_exploration_or_a_green_verdict() -> Result<()> {
    let mut runner = FakeRunner::new(usize::MAX);
    runner.divergent_control = true;
    run_comparison(&mut runner)?;
    assert_eq!(runner.calls.len(), 2);
    assert_eq!(runner.calls[0].1, runner.calls[1].1);
    let saved = runner.saved.borrow();
    assert_eq!(saved["summary"]["verdict"], "Inconclusive");
    assert_eq!(
        saved["summary"]["completed_pairs"],
        serde_json::json!([0, 0, 0])
    );
    assert_eq!(saved["reproductions"][0]["reproduced"], true);
    Ok(())
}
