# Replay exploration pilot

This internal experiment compares archive-guided exploration with random
restarts on Level `0x36`. Its question is whether retaining successful input
prefixes increases oracle-comparable gameplay diversity within equal replay
budgets. The harness is test-only; its output has no compatibility commitment.

## Running and evidence

The ROM, extracted assets and oracle must be available. The neutral adapter
control is an explicit preflight:

```sh
cargo test -p wildside-cli replay_exploration::runner::control -- --ignored --exact --nocapture
cargo test -p wildside-cli replay_exploration::runner::pilot -- --ignored --exact --nocapture
```

Ordinary `replay_exploration::` tests exercise the search, scoring and evidence
admission without the ROM. The two ignored tests require deliberate invocation.

The pilot writes a fresh `.cache/replay-exploration/<run>/` directory. It retains
source/ROM/helper identity, complete input scenarios, native and oracle state,
oracle images, per-candidate comparison and timing, chosen-parent provenance,
coverage sets, incremental summary and divergence reproduction results.
`paired_coverage` stores baseline first, guided second, for seeds 0, 1 and 2.

## Design

- A settled-warp neutral control runs 420 frames from the normal spawn. No
  position, velocity, camera, form, diamond or lives patches apply.
- Both methods run 40 complete 420-frame candidates per seed. Every candidate
  repeats the same bootstrap and preserves the first 60 neutral inputs. Method
  order alternates within pairs; seeds are interleaved across rounds.
- Uniform input choices are neutral, left, right, down, B, left+B, right+B,
  left+C, right+C, left+B+C and right+B+C. Chunk lengths are uniformly selected
  from 1, 4, 8 and 16 frames. SplitMix64 uses rejection sampling; action and
  archive-selection streams are separate. The saved script is replay authority.
- Random restarts generate all 360 remaining frames. Guided exploration starts
  from a uniformly selected least-visited archive key, then generates the tail.
  The archive retains the shortest successful prefix per key; ties retain the
  earlier discovery. Prefix lengths range from 60 through 388 frames.
- Only fully comparable candidates add coverage or archive entries. Every
  checkpoint must align, every required field and RNG frame must exist, and
  player/camera CRC, gameplay, enemy and RNG comparisons must match. Any
  difference, including an oracle limitation, rejects the whole candidate.
- Situation keys contain level, signed 16-pixel position buckets, movement,
  facing, velocity signs, helmet, HP, diamonds and the sorted enemy
  family/HP/facing multiset. Behaviour keys omit level, position, facing and
  velocity, retaining movement, helmet, HP, diamonds and the enemy multiset.
  Both metrics subtract the neutral control's keys. These are diversity
  descriptors, not equivalence classes or branch-coverage claims.

The execution cap is two hours after build/setup, including the neutral control,
bootstrap, native/oracle execution and evidence IO. Search stops starting new
pairs after 105 minutes; the remaining time is reserved for reproductions.
Individual oracle runs have a 120-second timeout, bounded by the total deadline.
Incomplete pairs contribute to neither side. The frame budget counts complete
scenario replays, including repeated prefixes; the identical bootstrap is paid
on every invocation. Elapsed time records the actual cost of both.

## Decision and findings

Continue only when guided exploration has at least 25% more distinct situation
keys in at least two of three completed paired runs, and each winning run has
at least one behaviour key absent from its baseline. A complete result below
that threshold is **Stop**. Missing evidence, interrupted runs or a zero-coverage
baseline produce **Inconclusive**.

The first three distinct divergence signatures are replayed once, in discovery
order. Signatures group the earliest differing fields with level and movement;
they are investigation leads, not a count of root causes. Remaining signatures
are unconfirmed. Gameplay fixes are separate work. Selected native/oracle images
provide manual evidence; pixel parity and frozen HUD/audio timing are outside
the scoring rule.

The design defers save-state support until prefix retention earns further work.
Routine proofs and hidden-state learning require additional execution-context
modelling. Full pixel comparison would mix this gameplay experiment with frozen
rendering debt. Random restarts provide the control for the archive's added
complexity.

The neutral control's RNG parity is an independent prerequisite. Gameplay and
enemy agreement do not establish equality of the RNG offset and ring. A
control-only RNG divergence produces an inconclusive pilot and a repeat of the
same control; it does not justify changing the seed or weakening admission.

## Recorded pilot outcome

The 2026-09-10 pilot is **Inconclusive**. The neutral control fails RNG parity
before search, so all three seeds have zero completed pairs. This provides no
evidence for or against archive-guided exploration.

The [compact evidence record](replay-exploration-pilot-evidence.json) identifies
the committed harness, ROM and oracle helper. Both control executions contain
420 complete checkpoints and RNG frames. Player/camera CRCs and the compared
non-RNG gameplay/enemy fields match. RNG offsets differ at all 420 checkpoints;
the ring differs at 419. Counts deduplicate fields emitted by both frontiers.
At frame 0, the native offset is `0xffe6` and the oracle offset is `0xffe7`;
ring byte 30 is respectively `0xb4` and `0xda`.

The identical control replay reproduces the complete native state array,
oracle state array, oracle CRC/RNG streams and assessment. The two executions
take about 13 seconds together. The root cause is untraced. The checked-in
neutral scenario reproduces the finding without a gameplay change.

Manual inspection of preflight native/oracle images at frames 0, 59 and 419
shows the expected live level and corresponding scene content. This is not an
every-frame pixel comparison. The implementation passes `verify-slice-fast`,
including 12 deterministic harness tests. The two live entrypoints are ignored
in ordinary test runs; the pilot entrypoint successfully records this negative
outcome, while the strict control entrypoint reports the RNG failure.
