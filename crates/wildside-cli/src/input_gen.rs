//! Deterministic random controller scripts for exploration and parity fuzzing.

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
    fn a_partial_switch_rate_holds_masks_across_runs() {
        let script = extend_prefix(&[], 5000, &PROFILE, &mut Choices(1));
        let changes = script.windows(2).filter(|pair| pair[0] != pair[1]).count();
        // ~1000 runs at 30% switch and 4/5 of draws picking a new mask.
        assert!((150..350).contains(&changes), "{changes} mask changes");
    }
}
