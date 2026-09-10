use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;

use super::runner::Runner;
use super::{
    Archive, CANDIDATES, Choices, Coverage, Evidence, FRAMES, LAST_PREFIX, PREFIX, Verdict,
    extend_prefix, verdict,
};
use crate::scenarios::{DynamicCheckpoint, DynamicScenario};

pub(super) fn seed_scenario() -> DynamicScenario {
    serde_json::from_str(include_str!(
        "../../../../tests/data/dynamic-scenarios/exploration-level36-neutral.json"
    ))
    .expect("checked-in neutral seed")
}

fn candidate(script: Vec<u8>, name: &str) -> DynamicScenario {
    let mut scenario = seed_scenario();
    scenario.slug = format!("exploration-{name}");
    scenario.script = script;
    scenario.checkpoints = (0..FRAMES)
        .map(|frame_index| DynamicCheckpoint {
            label: format!("frame_{frame_index}"),
            frame_index,
        })
        .collect();
    scenario
}

struct Search {
    archive: Archive,
    baseline_actions: Choices,
    guided_actions: Choices,
    selection: Choices,
}

#[derive(Serialize)]
struct Witness {
    discovery: usize,
    signature: String,
    source: String,
    scenario: DynamicScenario,
}

#[derive(Serialize)]
struct Report<'a> {
    verdict: Verdict,
    completed_pairs: &'a [usize; 3],
    paired_coverage: &'a [(Coverage, Coverage)],
    invalid_candidates: usize,
    error: Option<&'a str>,
    unique_divergences: usize,
    pair_order: [&'static str; 2],
}

fn remember(
    witnesses: &mut BTreeMap<String, Witness>,
    evidence: &Evidence,
    scenario: &DynamicScenario,
    name: &str,
) {
    if let Some(signature) = &evidence.signature {
        let discovery = witnesses.len();
        witnesses
            .entry(signature.clone())
            .or_insert_with(|| Witness {
                discovery,
                signature: signature.clone(),
                source: name.to_owned(),
                scenario: scenario.clone(),
            });
    }
}

fn add_coverage(
    coverage: &mut Coverage,
    archive: Option<&mut Archive>,
    evidence: &Evidence,
    scenario: &DynamicScenario,
    source: &str,
    neutral: &Coverage,
) {
    if !evidence.accepted() {
        return;
    }
    coverage.admit(&evidence.keys, neutral);
    if let Some(archive) = archive {
        for length in PREFIX..=LAST_PREFIX {
            archive.insert(
                evidence.keys[length - 1].clone(),
                &scenario.script[..length],
                source,
            );
        }
    }
}

pub(super) fn run_comparison(runner: &mut impl Runner) -> Result<()> {
    let control = runner.evaluate(&candidate(vec![0; FRAMES], "control"), "control")?;
    if !control.accepted() {
        runner.save(
            "summary",
            &serde_json::json!({
                "verdict": "Inconclusive", "reason": "neutral control is not fully comparable",
                "control": control.description(), "completed_pairs": [0, 0, 0],
            }),
        )?;
        if let Some(signature) = &control.signature {
            let repeated = runner.evaluate(
                &candidate(vec![0; FRAMES], "control"),
                "control-reproduction",
            );
            let record = match repeated {
                Ok(evidence) => serde_json::json!({
                    "source": "control", "signature": signature,
                    "reproduced": evidence.signature.as_ref() == Some(signature),
                    "result": evidence.description(),
                }),
                Err(error) => serde_json::json!({
                    "source": "control", "signature": signature,
                    "reproduced": false, "error": format!("{error:#}"),
                }),
            };
            runner.save("reproductions", &vec![record])?;
        }
        return Ok(());
    }
    let neutral = Coverage {
        situations: control.keys.iter().cloned().collect(),
        behaviours: control
            .keys
            .iter()
            .map(|key| key.behaviour.clone())
            .collect(),
        accepted: 1,
    };
    runner.save("neutral-coverage", &neutral)?;
    let mut searches = (0..3)
        .map(|seed| {
            let mut archive = Archive::default();
            archive.insert(control.keys[PREFIX - 1].clone(), &[0; PREFIX], "control");
            Search {
                archive,
                baseline_actions: Choices(seed),
                guided_actions: Choices(seed),
                selection: Choices(seed ^ 0xa5a5_a5a5_a5a5_a5a5),
            }
        })
        .collect::<Vec<_>>();
    let mut pairs = (0..3)
        .map(|_| (Coverage::default(), Coverage::default()))
        .collect::<Vec<_>>();
    let mut completed = [0; 3];
    let mut invalid = 0;
    let mut error = None;
    let mut witnesses = BTreeMap::new();
    'rounds: for round in 0..CANDIDATES {
        for seed in 0..3 {
            if !runner.exploration_time_left() {
                break 'rounds;
            }
            let search = &mut searches[seed];
            let parent = search.archive.select(&mut search.selection);
            let names = [
                format!("seed-{seed}-round-{round:02}-baseline"),
                format!("seed-{seed}-round-{round:02}-guided"),
            ];
            let scenarios = [
                candidate(
                    extend_prefix(&[0; PREFIX], &mut search.baseline_actions),
                    &names[0],
                ),
                candidate(
                    extend_prefix(&parent.script, &mut search.guided_actions),
                    &names[1],
                ),
            ];
            runner.save(&format!("{}-parent", names[1]), &parent)?;
            let order = if (round + seed) % 2 == 0 {
                [0, 1]
            } else {
                [1, 0]
            };
            let mut outcomes = [None, None];
            for index in order {
                match runner.evaluate(&scenarios[index], &names[index]) {
                    Ok(evidence) => {
                        remember(&mut witnesses, &evidence, &scenarios[index], &names[index]);
                        outcomes[index] = Some(evidence);
                    }
                    Err(failure) => {
                        error = Some(format!("{}: {failure:#}", names[index]));
                        break 'rounds;
                    }
                }
            }
            // Admit neither side until both full executions have completed.
            let [Some(baseline), Some(guided)] = outcomes else {
                unreachable!("complete pair")
            };
            invalid +=
                usize::from(baseline.invalid.is_some()) + usize::from(guided.invalid.is_some());
            add_coverage(
                &mut pairs[seed].0,
                None,
                &baseline,
                &scenarios[0],
                &names[0],
                &neutral,
            );
            add_coverage(
                &mut pairs[seed].1,
                Some(&mut search.archive),
                &guided,
                &scenarios[1],
                &names[1],
                &neutral,
            );
            completed[seed] += 1;
            save_report(
                runner,
                &completed,
                &pairs,
                invalid,
                error.as_deref(),
                witnesses.len(),
            )?;
            runner.save("witnesses", &witnesses.values().collect::<Vec<_>>())?;
        }
    }
    save_report(
        runner,
        &completed,
        &pairs,
        invalid,
        error.as_deref(),
        witnesses.len(),
    )?;
    runner.save("witnesses", &witnesses.values().collect::<Vec<_>>())?;
    // BTreeMap lookup deduplicates signatures; reproduction follows discovery order.
    let mut ordered = witnesses.values().collect::<Vec<_>>();
    ordered.sort_by_key(|witness| witness.discovery);
    let mut reproductions = Vec::new();
    for (index, witness) in ordered.into_iter().take(3).enumerate() {
        let result = runner.evaluate(&witness.scenario, &format!("reproduction-{index}"));
        let record = match result {
            Ok(evidence) => serde_json::json!({ "source": witness.source,
                "signature": witness.signature, "reproduced": evidence.signature.as_ref() == Some(&witness.signature),
                "result": evidence.description() }),
            Err(failure) => serde_json::json!({ "source": witness.source,
                "signature": witness.signature, "reproduced": false, "error": format!("{failure:#}") }),
        };
        reproductions.push(record);
        runner.save("reproductions", &reproductions)?;
    }
    Ok(())
}

fn save_report(
    runner: &impl Runner,
    completed: &[usize; 3],
    pairs: &[(Coverage, Coverage)],
    invalid: usize,
    error: Option<&str>,
    witnesses: usize,
) -> Result<()> {
    let complete =
        completed.iter().all(|count| *count == CANDIDATES) && error.is_none() && invalid == 0;
    runner.save(
        "summary",
        &Report {
            verdict: verdict(pairs, complete),
            completed_pairs: completed,
            paired_coverage: pairs,
            invalid_candidates: invalid,
            error,
            unique_divergences: witnesses,
            pair_order: ["baseline", "guided"],
        },
    )
}
