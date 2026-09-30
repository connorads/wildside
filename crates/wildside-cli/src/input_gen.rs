//! Controller scripts: deterministic random generation for exploration and
//! parity fuzzing, and reduction of a failing script.

use anyhow::Result;

/// SplitMix64, with rejection sampling so each bounded choice is uniform.
pub(crate) struct Choices(pub(crate) u64);

impl Choices {
    pub(crate) fn index(&mut self, count: usize) -> usize {
        let bound = u64::try_from(count).expect("choice count fits u64");
        assert!(bound > 0);
        loop {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.0;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^= value >> 31;
            if value >= bound.wrapping_neg() % bound {
                return usize::try_from(value % bound).expect("choice fits usize");
            }
        }
    }
}

/// How a script is extended: runs of one held mask, each run's length drawn
/// from `run_lengths`. After the first run, the mask changes with probability
/// `switch_percent`/100; at 100 no switch draw is made.
pub(crate) struct InputProfile {
    pub(crate) actions: &'static [u8],
    pub(crate) run_lengths: &'static [usize],
    pub(crate) switch_percent: usize,
}

/// Appends random runs to `prefix` until the script holds `frames` masks.
pub(crate) fn extend_prefix(
    prefix: &[u8],
    frames: usize,
    profile: &InputProfile,
    choices: &mut Choices,
) -> Vec<u8> {
    let mut script = prefix.to_vec();
    let mut mask = None;
    while script.len() < frames {
        let length = profile.run_lengths[choices.index(profile.run_lengths.len())]
            .min(frames - script.len());
        let switch = mask.is_none()
            || profile.switch_percent >= 100
            || choices.index(100) < profile.switch_percent;
        if switch {
            mask = Some(profile.actions[choices.index(profile.actions.len())]);
        }
        script.extend(std::iter::repeat_n(mask.expect("chosen mask"), length));
    }
    script
}

/// Shrinks `script` while `keep` still accepts it. First truncates to
/// `truncate_to` frames, then zeroes spans from the end back towards the
/// start, halving the span width once a full pass changes nothing.
pub(crate) fn reduce_script(
    script: &[u8],
    truncate_to: usize,
    mut keep: impl FnMut(&[u8]) -> Result<bool>,
) -> Result<Vec<u8>> {
    let mut working = script.to_vec();
    if working.len() > truncate_to && keep(&working[..truncate_to])? {
        working.truncate(truncate_to);
    }
    let mut span = (working.len() / 2).max(1);
    while span > 0 {
        let mut changed = false;
        let mut index = working.len();
        while index > 0 {
            let start = index.saturating_sub(span);
            let mut candidate = working.clone();
            let mut zeroed_any_input = false;
            for slot in candidate.iter_mut().take(index).skip(start) {
                if *slot != 0 {
                    *slot = 0;
                    zeroed_any_input = true;
                }
            }
            if zeroed_any_input && keep(&candidate)? {
                working = candidate;
                changed = true;
                index = working.len();
                continue;
            }
            if start == 0 {
                break;
            }
            index = start;
        }
        if !changed {
            span /= 2;
        }
    }
    Ok(working)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: InputProfile = InputProfile {
        actions: &[0, 1, 2, 4, 8],
        run_lengths: &[4, 5, 6],
        switch_percent: 30,
    };

    #[test]
    fn scripts_are_reproducible_from_the_seed_and_keep_the_prefix() {
        let prefix = [8; 10];
        let first = extend_prefix(&prefix, 200, &PROFILE, &mut Choices(7));
        assert_eq!(
            first,
            extend_prefix(&prefix, 200, &PROFILE, &mut Choices(7))
        );
        assert_ne!(
            first,
            extend_prefix(&prefix, 200, &PROFILE, &mut Choices(8))
        );
        assert_eq!(first.len(), 200);
        assert_eq!(&first[..10], &prefix);
        assert!(first.iter().all(|mask| PROFILE.actions.contains(mask)));
    }

    #[test]
    fn reduction_truncates_then_zeroes_every_input_the_predicate_ignores() {
        let script = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut calls = 0;
        // "Fails" while frame 3 still holds a nonzero input and frame 6 holds 7.
        let reduced = reduce_script(&script, 7, |candidate| {
            calls += 1;
            Ok(candidate.len() >= 7 && candidate[3] != 0 && candidate[6] == 7)
        })
        .unwrap();
        assert_eq!(reduced, [0, 0, 0, 4, 0, 0, 7]);
        assert!(calls < 40, "{calls} predicate calls");

        let kept = reduce_script(&script, 4, |candidate| Ok(candidate == script)).unwrap();
        assert_eq!(kept, script);
    }

    #[test]
    fn a_partial_switch_rate_holds_masks_across_runs() {
        let script = extend_prefix(&[], 5000, &PROFILE, &mut Choices(1));
        let changes = script.windows(2).filter(|pair| pair[0] != pair[1]).count();
        // ~1000 runs at 30% switch and 4/5 of draws picking a new mask.
        assert!((150..350).contains(&changes), "{changes} mask changes");
    }
}
