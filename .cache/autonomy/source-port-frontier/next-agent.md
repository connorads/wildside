---
slug: any-percent-published-tail-clean-through-end
cycle: 75
updated_at: 2026-03-27T08:44:53Z
---

# Source-Port Frontier Handoff

## What Changed

- Refreshed the frontier handoff after the published any-percent tail was proven authoritative-clean through frame 37759. The previous note still pointed at the already-fixed frame-29402 `mode=$28` demo hand-off.
- Synced the TAS metadata with the live frontier state: [`docs/source-port/tas-roadmap.md`](/home/connor/git/kc/docs/source-port/tas-roadmap.md) now marks `any-percent-published` verified through frame 37759 and closes milestones M7/M8.
- Re-ran the existing tail evidence from [`docs/source-port/frontier-state.yaml`](/home/connor/git/kc/docs/source-port/frontier-state.yaml): `published_tas_later_timeout_restart_intro_sequence_1_restarts_demo_play_on_frame_34916`, `published_tas_later_demo_2_transform_ignores_start_abort_on_frame_35148`, `verify-tas --frame-start 35140 --frame-end 36000 --checkpoint-interval 1 --bisect`, the 36000..37000 and 37000..37759 chunk checks, and `mise run verify-slice-fast`.

## Current Model

- There is no open authoritative successor frontier in the published any-percent recording. [`frontier-state.yaml`](/home/connor/git/kc/docs/source-port/frontier-state.yaml) is correct: the later intro/title/demo repairs keep the run clean through the last recorded frame, 37759.
- The stale autonomy artefacts were the remaining problem, not gameplay parity. Future TAS work should start from a new regression or a different incomplete tracker item, not from frame 29402 / 16228 / 7969.
- If a new TAS regression appears, restart from the compact frontier state and re-bisect from frame 23250; do not revive the older roadmap note or this note's predecessors.

## Strongest Evidence

- [`frontier-state.yaml`](/home/connor/git/kc/docs/source-port/frontier-state.yaml)
  - Active frontier slug `any-percent-published-tail-clean-through-end`, status `complete`, clean through frame 37759, blocker `None`.
- Tail verification
  - `cargo test -q -p kid-core published_tas_later_timeout_restart_intro_sequence_1_restarts_demo_play_on_frame_34916 -- --nocapture`
  - `cargo test -q -p kid-core published_tas_later_demo_2_transform_ignores_start_abort_on_frame_35148 -- --nocapture`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 35140 --frame-end 36000 --checkpoint-interval 1 --bisect`
  - Chunked CRC checks for frames 36000..37000 and 37000..37759
  - `mise run verify-slice-fast`
- Metadata alignment
  - [`tas-roadmap.md`](/home/connor/git/kc/docs/source-port/tas-roadmap.md) now agrees with the compact frontier state instead of advertising the old frame-7969 blocker as live.

## Exact Next Moves

1. Do not resume the published any-percent tail from an old stale frame. Start from [`docs/source-port/frontier-state.yaml`](/home/connor/git/kc/docs/source-port/frontier-state.yaml); it is already complete through frame 37759.
2. If a new TAS regression is introduced, rerun `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect` and open a fresh frontier from the first new authoritative mismatch.
3. If no new TAS regression exists, take the next non-tail fidelity item from the live tracker instead of reopening this frontier. Backlog `#037` and `#062` are the remaining obvious candidates.

## Verification To Re-run

- `mise run frontier-status`
- `cargo test -q -p kid-core published_tas_later_timeout_restart_intro_sequence_1_restarts_demo_play_on_frame_34916 -- --nocapture`
- `cargo test -q -p kid-core published_tas_later_demo_2_transform_ignores_start_abort_on_frame_35148 -- --nocapture`
- `mise run verify-slice-fast`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`

## Dead Ends / Do Not Retry

- Do not reopen frame 29402, frame 16228, or frame 7969 as if they were still active. Those notes are historical artefacts now.
- Do not use [`tas-roadmap.md`](/home/connor/git/kc/docs/source-port/tas-roadmap.md) as the live frontier source without first checking [`frontier-state.yaml`](/home/connor/git/kc/docs/source-port/frontier-state.yaml).
