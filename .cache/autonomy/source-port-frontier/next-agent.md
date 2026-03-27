---
slug: any-percent-published-tail-clean-through-end
cycle: 77
updated_at: 2026-03-27T11:05:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-checked the supposed live `kc_any_percent_2024` frame-3414 pause frontier before touching code.
- Confirmed the gameplay fix is already landed: [`demo_1_matches_the_oracle_pause_snapshot_on_frame_3414`](/home/connor/git/kc/crates/kid-core/src/tests.rs) passes, and `verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3400 --frame-end 3500 --bisect` is CRC-clean.
- Refreshed the stale TAS-frontier metadata in [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md), [`.cache/autonomy/source-port-frontier/next-task.json`](/home/connor/git/kc/.cache/autonomy/source-port-frontier/next-task.json), and this handoff note so the next session does not reopen the already-solved frame-3414 pause snapshot.

## Current Model

- No TAS frontier is currently open.
- `kc_any_percent_2024` is authoritative-clean through frame `4916`, and `any-percent-published` is authoritative-clean through frame `37759`.
- The remaining frontier work is outside TAS parity unless a fresh `verify-tas --bisect` run proves a new authoritative mismatch.

## Strongest Evidence

- [`crates/kid-core/src/tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
  - `demo_1_matches_the_oracle_pause_snapshot_on_frame_3414`
- Focused verification
  - `cargo test -q -p kid-core demo_1_matches_the_oracle_pause_snapshot_on_frame_3414 -- --nocapture`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3400 --frame-end 3500 --bisect`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3412 --frame-end 3416 --include-advisory --show-object-roster-on-match`
- Tracker/docs state
  - [`docs/source-port/frontier-state.yaml`](/home/connor/git/kc/docs/source-port/frontier-state.yaml)
  - [`docs/source-port/tas-roadmap.md`](/home/connor/git/kc/docs/source-port/tas-roadmap.md)
  - [`docs/source-port/research-map.md`](/home/connor/git/kc/docs/source-port/research-map.md)

## Exact Next Moves

1. Do not reopen `kc_any_percent_2024` frame `3414`; that pause snapshot is already fixed.
2. If TAS parity is revisited, start from `docs/source-port/frontier-state.yaml` and only open a new frontier after a fresh `verify-tas --bisect` identifies an authoritative mismatch.
3. Otherwise continue with the non-TAS open work, currently backlog `#062` (HeadyMetal death wobble visual verification).

## Verification To Re-run

- `cargo test -q -p kid-core demo_1_matches_the_oracle_pause_snapshot_on_frame_3414 -- --nocapture`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3400 --frame-end 3500 --bisect`
- `mise run frontier-status`

## Dead Ends / Do Not Retry

- Do not reopen `kc_any_percent_2024` frame `3414`; the test and focused bisect are already green.
- Do not reopen the published any-percent tail; that frontier is complete through frame `37759`.
- Do not treat advisory-only `tas-diff` output around frames `3412..3416` as proof of an authoritative gameplay break when the CRC bisect is clean.
