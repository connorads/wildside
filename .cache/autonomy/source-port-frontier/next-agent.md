---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 17
updated_at: 2026-03-22T23:09:30Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read the live ASM jump loop at `loc_A73A -> loc_A74E` and narrowed the on-path upward-velocity sources instead of looking for another carry-state latch.
- Checked the published script around frames `7966..7970`; every mask is still `8` (`right` held only), so there is no fresh B/C edge in the frontier window.
- Cross-checked the native stomp path and confirmed native still resolves stomp bounce after motion via `process_stomps -> pending_stomp_bounce`, not inside the same pre-physics jump loop.

## Current Model

- The active frontier is still frame `7969` in `any-percent-published`, clean through `7968`.
- In the ROM path `loc_A73A -> Character_CheckCollision -> loc_A74E`, the only plausible generic way to reach negative `y_vel` in this exact right-held/no-B/no-C window is a live collision consumer before `sub_B168`, specifically `Jump_On_Enemy`.
- `Jump_On_Enemy` restores the saved anchor from `$FFFFFA2C/$FFFFFA2E` and writes `y_vel = #$FFFC0000` while Kid is still in `MoveID_Jump`, so the bounce is visible in the same loop iteration before gravity finishes.
- Native does not mirror that timing today: it discovers stomps in `resolve_frame_hazards()` after motion, stores `pending_stomp_bounce`, and only applies the bounce at the next frame start. That timing gap is now the strongest candidate for why oracle has already lifted off by frame `7969` while native is still grounded.
- Inference from the existing probes plus the script read: this is no longer best explained as a missing jump edge, wall-cling release, or generic carry-state handoff. The next useful probe should target same-frame collision/stomp state around `Character_CheckCollision`, not another broad movement bag.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm:13055`
  Result: the live jump loop calls `sub_7ACC`, then `Character_CheckCollision`, then `sub_B084`, then `sub_B168`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:14607`
  Result: `Jump_On_Enemy` requires `MoveID_Jump`, restores the saved anchor, and sets `y_vel` to `#$FFFC0000` before returning to the loop.
- `python - <<'PY' ...`
  Result: `scenario.json` frame masks `7966..7970` are all `8`, matching the existing grounded-release probe’s `held=right` and ruling out a fresh B/C edge in the frontier window.
- `crates/kid-core/src/combat.rs:28`
  Result: `apply_pending_stomp_bounce()` replays the bounce from stored anchors at the next frame start.
- `crates/kid-core/src/combat.rs:209`
  Result: `process_stomps()` queues that bounce into `pending_stomp_bounce` instead of applying it immediately.
- `crates/kid-core/src/combat.rs:963`
  Result: native only runs `process_stomps()` from `resolve_frame_hazards()` after the movement step, so same-frame ROM-style `Character_CheckCollision` bounce timing is not currently represented there.

## Exact Next Moves

1. Instrument the native equivalent of the ROM’s `Character_CheckCollision -> Jump_On_Enemy` gate around frames `7968..7969`: expose whether a stomp-eligible enemy contact exists in the same frame before native reaches post-motion `process_stomps()`.
2. If that probe fires, compare the anchor/velocity it would need with the ROM bounce (`saved x/y anchor`, `y_vel = -0x40000`) and then move the stomp resolution to the correct same-frame point with a tight regression around frames `7968..7969`.
3. Only if the new probe stays empty should the next session return to secondary movement-state paths such as wall-cling or helmet-specific live transitions.

## Verification To Re-run

- `cargo test -p kid-core trace_frame_reports_kid_collision_handoff_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_collision_handoff_probe -- --nocapture`
- `cargo test -p kid-core trace_frame_reports_kid_grounded_release_probe -- --nocapture`
- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_grounded_release_probe -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7968 --frame-end 7969 --include-advisory --include-physics-trace --show-object-roster-on-match`
- `mise run audit-debug-fields`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retry terrain-write theories for cell `58,28`; the support tile still matches the generated layout and native still sees solid support there.
- Do not retry generic post-motion support/platform/lift handoff theories for this window; the carry-state probes are still empty.
- Do not retry native goal-touch timing; the goal probe still shows `lifecycle=alive` and `touch=0`.
- Do not retry broad special-motion latch theories; the frontier script is right-held only through frames `7966..7970`, and the special-motion probe is still idle in the live window.
- Do not add another generic carry-state probe unless the next stomp/collision probe points back to one.
