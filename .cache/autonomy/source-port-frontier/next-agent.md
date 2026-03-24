---
slug: any-percent-published-frame-9355-intro-sequence-4-clears-second-roster-pose-before-oracle
cycle: 42
updated_at: 2026-03-24T00:36:22Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `Load_IntroSequence3`, `sub_1B93E` / `loc_1B96A`, and `sub_1C246` / `loc_1C250` in `kid.asm`.
- Replaced the broad no-Start regression with `published_tas_second_timeout_restart_intro_sequence_3_visit_matches_oracle_frame_9312`, which now pins frames 9311, 9312, 9354, and the frame-9355 `Game_Mode` hand-off.
- Retuned `DEMO_AUTO_TIMEOUT_INTRO_THIRD_SEGMENT_FRAMES` from `256` to the oracle-backed `299`, so native keeps the second timeout-restart intro-sequence-3 visit alive through frame 9354 and only flips `Game_Mode` to intro sequence 4 on frame 9355.
- Re-ran the focused intro regressions, `tas-diff 9308..9395`, `verify-tas --frame-start 6975 --frame-end 9530 --bisect`, and `mise run verify-slice-fast`.

## Current Model

- The old frame-9312 timing bug is fixed: native now stays in intro sequence 3 with the second roster pose through frame 9354.
- The new first authoritative mismatch is frame 9355, not on `Game_Mode`, but on pose carry: native enters `game_mode=0x0020` and clears Kid immediately, while oracle is also in `game_mode=0x0020` but still shows the second `Load_IntroSequence3` roster pose at `0x00700000,0x00CC0000`.
- Oracle keeps that carried intro-sequence-3 object roster alive under intro sequence 4 through frame 9381; frames 9382..9391 are authoritative-clean apart from advisory `number_objects`.
- The next causal bug after that is frame 9392 (`camera_y_pos=0x00000000` native vs `0x00040000` oracle), but do not chase it until the frame-9355 pose-carry mismatch is fixed.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `sub_1B93E` / `loc_1B96A` are still the natural timeout path; they poll Start via `sub_1C246` and then write `Game_Mode=$20`.
  - `sub_1C246` / `loc_1C250` remain the only direct-title path here, gated by `bclr #7,(Ctrl_1_Pressed).w`.
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9395 --include-advisory`
  - frames `9308..9354`: advisory-only (`number_objects`)
  - frames `9355..9381`: `Game_Mode` matches intro sequence 4, but native clears Kid while oracle keeps the second roster pose and `lp=0x01B96A`
  - frames `9382..9391`: authoritative-clean again
  - frame `9392`: next authoritative diff is `camera_y_pos`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
  - first CRC divergence moved from frame `9312` to frame `9355`

## Exact Next Moves

1. Read the intro-sequence-4 setup/entry ASM around `Load_IntroSequence4` and `sub_1B988`, then trace how the old `loc_1B96A` roster can remain visible after `Game_Mode` has already changed to `$20`.
2. Compare that ROM behaviour against native `current_player_state()` / intro-sequence boot handling; patch the early intro-sequence-4 pose carry so native keeps the second roster pose alive through the oracle-backed `9355..9381` window.
3. Replace the current frame-9355 `Game_Mode` assertion with an oracle-backed intro-sequence-4 pose-carry regression once the exact carry window is proven.

## Verification To Re-run

- `cargo test -p kid-core published_tas_only_presses_start_on_the_first_timeout_restart_intro_sequence_3_visit -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_uses_the_second_intro_sequence_3_roster_pose_on_frame_9097 -- --nocapture`
- `cargo test -p kid-core published_tas_second_timeout_restart_intro_sequence_3_visit_matches_oracle_frame_9312 -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9308 --frame-end 9395 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reintroduce the old `segment == 2 && intro_sequence_3_frames_override.is_some()` natural-title shortcut; ASM still proves that path is Start-driven only.
- Do not revert the intro-sequence-3 timer back towards `256`; the timer retune is what moved the CRC frontier from frame 9312 to frame 9355.
- Do not assume the frame-9355 bug is still an intro-sequence-3 timer bug. `Game_Mode` now matches; the remaining blocker is the carried roster pose under intro sequence 4.
- Do not chase the frame-9392 camera mismatch yet. It is downstream of the frame-9355 pose-carry bug.
