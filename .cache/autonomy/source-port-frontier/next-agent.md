---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 22
updated_at: 2026-03-23T00:01:09Z
---

# Source-Port Frontier Handoff

## What Changed

- Added a focused `kid_flip_branch_probe` at the shared post-`sub_AF10` branch-selection boundary, capturing the wall-scan result, raw `B`/`C` edges, grounded window, facing, helmet id, and the native `kid_flip` fan-out branch label.
- Registered the new exploratory debug key and added both a unit formatter regression and a published-TAS regression for frames `7968..7969`.
- Re-ran the new probe tests, the existing `published_tas_frames_7968_7969_report_kid_` suite, `tas-diff` for frames `7968..7969`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- The new probe proves the raw `C` edge does survive to the shared `kid_flip` fan-out on frame `7968`: the post-`sub_AF10` wall scan is still hot at `cell=59,27`, Kid stays grounded (`1=>1`), and native still chooses `branch=c_pressed_fallthrough`.
- Inference from the new probe plus the earlier attach probe: the live blocker is no longer "native never sees the `C` edge" or "the wall scan is already cold". Native sees both, but no current shared `kid_flip` consumer claims grounded Iron Knight here, so the missing lift-off is more likely in the shared fallthrough follow-on (`loc_AD2C` / `sub_B270` / `loc_AA22`) or an omitted grounded branch adjacent to the fan-out, not in the explicit Iron Knight attach gate.

## Strongest Evidence

- `crates/kid-core/src/lib.rs`
  Result: `note_kid_flip_branch_probe` now records the shared `kid_flip` branch-selection outcome after the post-`sub_AF10` wall scan and after the native post-physics Cyclone activation hook.
- `crates/kid-core/src/trace.rs`
  Result: `kid_flip_branch_probe_debug` formats `wall_scan`, raw button edges, helmet/facing, grounded transition, branch label, and wall contact.
- `crates/kid-core/src/tests.rs`
  Result: `trace_frame_reports_kid_flip_branch_probe` and `published_tas_frames_7968_7969_report_kid_flip_branch_probe` lock the formatter and the frontier findings in place.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
  Result: frame `7968` now reports `kid_flip_branch_probe = wall_scan=1/b_press=0/c_press=1/helmet=0x0006/facing=right/ground=1=>1/branch=c_pressed_fallthrough/wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0`.
- `mise run audit-debug-fields`
  Result: clean after registering `kid_flip_branch_probe`.
- `mise run verify-slice-fast`
  Result: passes.

## Exact Next Moves

1. Inspect the shared fallthrough path immediately after `kid_flip` chooses `c_pressed_fallthrough`, concentrating on the native mirrors of `loc_AD2C`, `sub_B270`, and `loc_AA22` rather than re-reading the explicit Iron Knight attach/block-break branches.
2. Add the next focused probe at that shared post-fallthrough boundary if needed, capturing whether the native follow-on mutates grounded state, velocity, or wall state on frame `7968` before the frame closes.
3. Do not implement a gameplay fix yet unless that follow-on probe exposes a concrete omitted shared consumer for the oracle lift-off.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_flip_branch_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_flip_branch_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run audit-debug-fields`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retry terrain-write theories for cell `58,28`; the support tile still matches the generated layout.
- Do not retry generic post-motion support/platform/lift handoff theories; the existing carry-state probes are still empty.
- Do not retry native goal-touch timing; the goal probe is still `lifecycle=alive` with `touch=0`.
- Do not retry broad special-motion latch theories; the existing special-motion probe is still idle in the live window.
- Do not retry same-frame stomp timing as the primary explanation unless a new movement-state probe later exposes a concrete stomp candidate.
- Do not retry ROM `sub_AF7A` / Iron Knight block-breaker as the primary explanation unless a later probe finds non-zero pre-physics horizontal speed or a block-bearing forward wall in the live window.
- Do not retry wall-cling live-release as the primary explanation; the `kid_wall_cling_release_probe` shows frames `7968..7969` never carry wall-cling state into the `loc_AA9C` mirror.
- Do not retry Iron Knight wall-attach as the primary explanation; the `kid_iron_knight_attach_probe` shows frame `7968` does carry a raw `C` edge, but Kid is still grounded so the candidate stays `0` and no `-0x4000` attach velocity is injected.
- Do not retry "missing raw C-edge at kid_flip" as the primary explanation; the new `kid_flip_branch_probe` shows frame `7968` reaches the shared post-`sub_AF10` fan-out with `c_press=1`, `wall_scan=1`, and still falls through `c_pressed_fallthrough`.
- Do not rely on the broad `cargo test -p kid-core trace_frame_reports_kid_ -- --nocapture` filter as a frontier signal; it still trips the unrelated existing `trace_frame_reports_kid_object_support_candidates` assertion.
