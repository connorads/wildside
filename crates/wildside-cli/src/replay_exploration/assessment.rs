use anyhow::{Context, Result, ensure};
use serde::Serialize;

use super::{FRAMES, SituationKey, situation};
use crate::oracle::OracleState;
use crate::sweep::{
    FrontierKind, StateDiff, SweepStateComparisonOptions,
    compare_sweep_states_for_frontier_with_options,
};
use wildside_core::RomRandomFrameState;

#[derive(Debug, Serialize)]
pub(super) struct Evidence {
    pub(super) keys: Vec<SituationKey>,
    pub(super) differences: Vec<StateDiff>,
    pub(super) crc_mismatch: Option<usize>,
    pub(super) rng_mismatch: Option<usize>,
    pub(super) invalid: Option<String>,
    pub(super) signature: Option<String>,
}

impl Evidence {
    pub(super) fn accepted(&self) -> bool {
        self.invalid.is_none()
            && self.differences.is_empty()
            && self.crc_mismatch.is_none()
            && self.rng_mismatch.is_none()
            && self.keys.len() == FRAMES
    }

    pub(super) fn description(&self) -> String {
        if self.accepted() {
            "comparable".to_owned()
        } else if let Some(reason) = &self.invalid {
            format!("invalid: {reason}")
        } else {
            format!(
                "divergent: {}",
                self.signature.as_deref().unwrap_or("unknown")
            )
        }
    }
}

fn complete_inputs(
    native: &[OracleState],
    oracle: &[OracleState],
    native_crc: &[u32],
    oracle_crc: &[u32],
    rng: &[Option<RomRandomFrameState>],
) -> Result<Vec<SituationKey>> {
    ensure!(
        [
            native.len(),
            oracle.len(),
            native_crc.len(),
            oracle_crc.len(),
            rng.len()
        ]
        .into_iter()
        .all(|length| length == FRAMES),
        "incomplete 420-frame evidence"
    );
    for (index, (left, right)) in native.iter().zip(oracle).enumerate() {
        ensure!(
            left.frame_index == index
                && right.frame_index == index
                && left.label == format!("frame_{index}")
                && left.label == right.label,
            "unaligned checkpoint at {index}"
        );
        situation(left).with_context(|| format!("native frame {index}"))?;
        situation(right).with_context(|| format!("oracle frame {index}"))?;
        for state in [left, right] {
            let offset = state.rng_offset.context("missing RNG offset")?;
            ensure!(
                RomRandomFrameState::offset_is_initialised(offset),
                "uninitialised RNG"
            );
            state.rng_ring.as_ref().context("missing RNG ring")?;
        }
        let stream = rng[index].as_ref().context("missing RNG stream frame")?;
        ensure!(
            Some(stream.offset()) == right.rng_offset
                && Some(stream.ring()) == right.rng_ring.as_ref(),
            "oracle RNG stream/state disagree"
        );
    }
    oracle.iter().map(situation).collect()
}

pub(super) fn assess(
    native: &[OracleState],
    oracle: &[OracleState],
    native_crc: &[u32],
    oracle_crc: &[u32],
    rng: &[Option<RomRandomFrameState>],
) -> Evidence {
    let keys = match complete_inputs(native, oracle, native_crc, oracle_crc, rng) {
        Ok(keys) => keys,
        Err(error) => {
            return Evidence {
                keys: vec![],
                differences: vec![],
                crc_mismatch: None,
                rng_mismatch: None,
                invalid: Some(format!("{error:#}")),
                signature: None,
            };
        }
    };
    let mut differences = Vec::new();
    for kind in [FrontierKind::Gameplay, FrontierKind::Enemy] {
        differences.extend(compare_sweep_states_for_frontier_with_options(
            native,
            oracle,
            kind,
            SweepStateComparisonOptions {
                include_rng: true,
                ..Default::default()
            },
        ));
    }
    let crc_mismatch = native_crc
        .iter()
        .zip(oracle_crc)
        .position(|(left, right)| left != right);
    let rng_mismatch = native.iter().zip(rng).position(|(state, stream)| {
        let stream = stream.as_ref().expect("validated RNG stream");
        state.rng_offset != Some(stream.offset()) || state.rng_ring.as_ref() != Some(stream.ring())
    });
    let first_diff = differences
        .iter()
        .filter_map(|diff| {
            diff.checkpoint
                .strip_prefix("frame_")
                .and_then(|frame| frame.parse::<usize>().ok())
        })
        .min();
    let first = [first_diff, crc_mismatch, rng_mismatch]
        .into_iter()
        .flatten()
        .min();
    let signature = first.map(|frame| {
        let label = format!("frame_{frame}");
        let mut fields = differences
            .iter()
            .filter(|diff| diff.checkpoint == label)
            .map(|diff| format!("{:?}/{}", diff.category, diff.field))
            .collect::<Vec<_>>();
        if crc_mismatch == Some(frame) {
            fields.push("player_camera_crc".to_owned());
        }
        if rng_mismatch == Some(frame) {
            fields.push("rng_stream".to_owned());
        }
        fields.sort();
        fields.dedup();
        format!(
            "level={}:movement={}:{}",
            keys[frame].level,
            keys[frame].behaviour.movement,
            fields.join(",")
        )
    });
    Evidence {
        keys,
        differences,
        crc_mismatch,
        rng_mismatch,
        invalid: None,
        signature,
    }
}
