//! Internal, opt-in replay exploration experiment. No supported output contract.

mod assessment;
mod experiment;
mod runner;
mod tests;

use assessment::{Evidence, assess};
use experiment::{run_comparison, seed_scenario};

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::input_gen::{self, Choices, InputProfile};
use crate::oracle::OracleState;

const FRAMES: usize = 420;
const PREFIX: usize = 60;
const LAST_PREFIX: usize = 388;
const CANDIDATES: usize = 40;
const ACTIONS: [u8; 11] = [0, 4, 8, 2, 16, 20, 24, 36, 40, 52, 56];
const CHUNKS: [usize; 4] = [1, 4, 8, 16];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct EnemyKey {
    family: u8,
    hp: u16,
    facing: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct BehaviourKey {
    movement: u16,
    helmet: u16,
    hp: u16,
    diamonds: u16,
    enemies: Vec<EnemyKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct SituationKey {
    level: u16,
    x_bucket: i32,
    y_bucket: i32,
    facing: u8,
    x_velocity_sign: i32,
    y_velocity_sign: i32,
    behaviour: BehaviourKey,
}

fn situation(state: &OracleState) -> Result<SituationKey> {
    let mut enemies = state
        .active_enemies
        .iter()
        .map(|enemy| {
            Ok(EnemyKey {
                family: enemy.object_type,
                hp: enemy.current_hp.context("missing enemy HP")?,
                facing: format!("{:?}", enemy.facing),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    enemies.sort();
    Ok(SituationKey {
        level: state.current_level_id,
        // Oracle positions are signed 16.16; one bucket is 16 whole pixels.
        x_bucket: i32::from_ne_bytes(state.kid_x_pos.to_ne_bytes()) >> 20,
        y_bucket: i32::from_ne_bytes(state.kid_y_pos.to_ne_bytes()) >> 20,
        facing: state.kid_x_direction.context("missing facing")?,
        x_velocity_sign: state.kid_x_vel.context("missing x velocity")?.signum(),
        y_velocity_sign: state.kid_y_vel.context("missing y velocity")?.signum(),
        behaviour: BehaviourKey {
            movement: state.character_movement.context("missing movement")?,
            helmet: state.current_helmet_id,
            hp: state.number_hitpoints,
            diamonds: state.number_diamonds,
            enemies,
        },
    })
}

const PILOT_INPUT: InputProfile = InputProfile {
    actions: &ACTIONS,
    run_lengths: &CHUNKS,
    switch_percent: 100,
};

fn extend_prefix(prefix: &[u8], choices: &mut Choices) -> Vec<u8> {
    assert!((PREFIX..=LAST_PREFIX).contains(&prefix.len()));
    input_gen::extend_prefix(prefix, FRAMES, &PILOT_INPUT, choices)
}

#[derive(Clone, Serialize)]
struct Entry {
    script: Vec<u8>,
    selected: usize,
    source: String,
}

#[derive(Default)]
struct Archive(BTreeMap<SituationKey, Entry>);

impl Archive {
    fn insert(&mut self, key: SituationKey, script: &[u8], source: &str) {
        assert!((PREFIX..=LAST_PREFIX).contains(&script.len()));
        let entry = self.0.entry(key).or_insert_with(|| Entry {
            script: script.to_vec(),
            selected: 0,
            source: source.to_owned(),
        });
        if script.len() < entry.script.len() {
            entry.script = script.to_vec();
            entry.source = source.to_owned();
        }
    }

    fn select(&mut self, choices: &mut Choices) -> Entry {
        let least = self
            .0
            .values()
            .map(|entry| entry.selected)
            .min()
            .expect("seeded archive");
        let count = self
            .0
            .values()
            .filter(|entry| entry.selected == least)
            .count();
        let entry = self
            .0
            .values_mut()
            .filter(|entry| entry.selected == least)
            .nth(choices.index(count))
            .expect("selected archive entry");
        entry.selected += 1;
        entry.clone()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Coverage {
    situations: BTreeSet<SituationKey>,
    behaviours: BTreeSet<BehaviourKey>,
    accepted: usize,
}

impl Coverage {
    fn admit(&mut self, keys: &[SituationKey], neutral: &Self) {
        self.accepted += 1;
        for key in keys.iter().skip(PREFIX) {
            if !neutral.situations.contains(key) {
                self.situations.insert(key.clone());
            }
            if !neutral.behaviours.contains(&key.behaviour) {
                self.behaviours.insert(key.behaviour.clone());
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
enum Verdict {
    Continue,
    Stop,
    Inconclusive,
}

fn verdict(pairs: &[(Coverage, Coverage)], complete: bool) -> Verdict {
    if !complete
        || pairs.len() != 3
        || pairs
            .iter()
            .any(|(baseline, _)| baseline.situations.is_empty())
    {
        return Verdict::Inconclusive;
    }
    let wins = pairs
        .iter()
        .filter(|(baseline, guided)| {
            guided.situations.len() * 4 >= baseline.situations.len() * 5
                && guided
                    .behaviours
                    .difference(&baseline.behaviours)
                    .next()
                    .is_some()
        })
        .count();
    if wins >= 2 {
        Verdict::Continue
    } else {
        Verdict::Stop
    }
}

#[test]
fn improvement_requires_two_complete_pairs_and_non_position_novelty() {
    let make = |x| SituationKey {
        level: 54,
        x_bucket: x,
        y_bucket: 0,
        facing: 0,
        x_velocity_sign: 0,
        y_velocity_sign: 0,
        behaviour: BehaviourKey {
            movement: 0,
            helmet: 0,
            hp: 2,
            diamonds: 0,
            enemies: vec![],
        },
    };
    let pair = || {
        let baseline = Coverage {
            situations: (0..4).map(make).collect(),
            ..Coverage::default()
        };
        let guided = Coverage {
            situations: (0..5).map(make).collect(),
            behaviours: BTreeSet::from([make(0).behaviour]),
            ..Coverage::default()
        };
        (baseline, guided)
    };
    assert_eq!(verdict(&[pair(), pair(), pair()], true), Verdict::Continue);
    assert_eq!(
        verdict(&[pair(), pair(), pair()], false),
        Verdict::Inconclusive
    );
    assert_eq!(verdict(&[pair()], true), Verdict::Inconclusive);
    let mut pairs = vec![pair(), pair(), pair()];
    for (_, guided) in &mut pairs {
        guided.behaviours.clear();
    }
    assert_eq!(verdict(&pairs, true), Verdict::Stop);
    pairs[0].0.situations.clear();
    assert_eq!(verdict(&pairs, true), Verdict::Inconclusive);
}

#[test]
fn prefix_generation_is_reproducible_and_does_not_use_selection_randomness() {
    let prefix = vec![0; PREFIX];
    let expected = extend_prefix(&prefix, &mut Choices(2));
    let mut selection = Choices(2);
    for _ in 0..100 {
        selection.index(11);
    }
    assert_eq!(expected, extend_prefix(&prefix, &mut Choices(2)));
    assert_eq!(expected.len(), FRAMES);
    assert_eq!(&expected[..PREFIX], prefix);
    assert!(expected.iter().all(|mask| ACTIONS.contains(mask)));
    let long = vec![8; LAST_PREFIX];
    assert_eq!(&extend_prefix(&long, &mut Choices(3))[..LAST_PREFIX], long);
}
