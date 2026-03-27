---
slug: any-percent-published-frame-24921-level-15-gameplay-divergence
cycle: 70
updated_at: 2026-03-27T04:03:48Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the later demo-2 timeout-return title-pose miss by carrying the “started from the already-restored loc_1B5CE title pose” lineage bit through `begin_demo_play()` and reusing it in both demo-return entry points in [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs) and [`lib.rs`](/home/connor/git/kc/crates/kid-core/src/lib.rs).
- Added [`published_tas_demo_2_timeout_restore_reuses_title_pose_on_frames_23955_and_23956`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the last hidden title frame at 23954 plus the oracle-backed standard title pose on 23955..23956.
- Re-ran `cargo test -q -p kid-core published_tas_demo_2 -- --nocapture`, `cargo test -q -p kid-core published_tas_intro_sequence_6_timeout_restore_reenters_demo_play_on_frame_23248 -- --nocapture`, `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23948 --frame-end 23964 --include-advisory`, `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, and `mise run verify-slice-fast`; the first authoritative divergence now moves to frame 24921.

## Current Model

- The earlier intro-sequence-6 fixes only tagged demo 1 with the “later title already restored the pose” state. Demo 2, which starts from that repaired title shell on frame 23248, was still dropping back to the generic 150-frame hidden rebuild when the published TAS hit Start on frame 23805.
- Carrying that lineage bit into later demos makes the frame-23955/23956 title return match oracle without disturbing the frame-23248 demo re-entry window.
- The next blocker is no longer in title/demo choreography; `verify-tas --frame-start 23250 --bisect` now places the first CRC mismatch at frame 24921 in live level-0x15 gameplay.

## Strongest Evidence

- ASM / script
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`
    - `loc_1B5CE` writes the standard title pose (`x_pos=$2E`, `y_pos=$8C`) before the later title shell hands back into demo playback.
  - `.cache/tas-imports/any-percent-published/scenario.json`
    - Frame 23805 carries a real Start press during the later demo 2, so the frame-23955 window is again on the generic demo-title return path.
- Native
  - [`boot.rs`](/home/connor/git/kc/crates/kid-core/src/boot.rs)
    - `begin_demo_play()` now preserves whether the current demo started from a title shell that already had the loc_1B5CE pose restored, and both demo-return entry points reuse that bit before `enter_title_screen()` clears the title-state flags.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `published_tas_demo_2_timeout_restore_reuses_title_pose_on_frames_23955_and_23956` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23948 --frame-end 23964 --include-advisory`
    - Frames 23948..23963 now all match.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
    - First CRC divergence is now frame 24921.

## Exact Next Moves

1. Run `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 24918 --frame-end 24928 --include-advisory` to capture the first divergent fields around frame 24921.
2. Read the relevant level-0x15 gameplay ASM before coding; the later title/demo shell is now authoritative-clean through frame 24920.
3. Add the next focused gameplay regression only after the frame-24921 behaviour is proven against ASM plus oracle.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_demo_2 -- --nocapture`
- `cargo test -q -p kid-core published_tas_intro_sequence_6_timeout_restore_reenters_demo_play_on_frame_23248 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23948 --frame-end 23964 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old cold-boot demo-1 wrap. `demo_1_bootstrap_keeps_camera_hidden_through_frame_2537` still expects `0xFF000000` on the cold-boot path.
- Do not collapse the new lineage bit back onto `demo_uses_intro_sequence_6_hidden_camera_wrap`; frame 23955 proved that later demo 2 inherits the early title restore without sharing demo 1's hidden camera wrap.
- Do not keep the frontier parked in title mode. The rebisect moved the first real mismatch into level-0x15 gameplay on frame 24921.
