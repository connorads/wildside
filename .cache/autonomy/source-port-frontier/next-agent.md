---
slug: any-percent-published-frame-7969-knights-isle-native-stays-grounded-while-oracle-lifts-off
cycle: 26
updated_at: 2026-03-23T00:45:00Z
---

# Source-Port Frontier Handoff

## What Changed

- Broke the probe-only loop: 15 prior sessions added ~2,868 lines of native-side instrumentation without reading the disassembly. The investigation hierarchy now requires reading ASM before writing any probe code.
- All prior probe findings are preserved in the codebase. The Dead Ends list below captures everything ruled out.

## Current Model

- Frame 7969 in `any-percent-published`, level `0x15` (Knights Isle). Clean through 7968.
- Native keeps Kid grounded on solid terrain cell `58,28` with `y_vel=0`.
- Oracle lifts Kid off with `y_velocity=0xFFFFC000` starting frame 7968.
- 15 native-side probes have ruled out: terrain mutations, goal-touch, special-motion latches, stomp collision, wall-cling, Iron Knight attach, kid_flip fan-out, transform/helmet-change, jump buffer, and pre-sub_B168 stomp candidates.
- The handoff note from cycle 25 pointed at `loc_AA22` / `Character_CheckCollision` as the likely pre-`sub_B168` path that arms the oracle's upward velocity. **Nobody has actually read the ASM yet.**

## Strongest Evidence

- Native probes: `kid_collision_handoff_probe` reports all carry-in state as idle/zero on frames 7968-7969.
- Oracle: `tas-diff` shows Kid gfx slot 2 gaining `y_velocity=0xFFFFC000` on frame 7968.
- The gap: we know native doesn't do something oracle does, but we don't know what because nobody read the disassembly.

## Exact Next Moves

1. **Read the ASM around `loc_AA22`** in `.cache/upstream/kid-chameleon-disasm/kid.asm`. Search for `loc_AA22`. Read ~100 lines in each direction. Trace what happens between `Character_CheckCollision` and `sub_B168` on the Jump path — specifically, what writes to `y_vel` ($2A(a3)) or changes `Character_Movement`.

2. **Run oracle diagnostics**: `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`. Look at how oracle's y_velocity changes from 0 to 0xFFFFC000.

3. **Cross-reference**: compare what the ASM says should write y_vel with what native actually does in `crates/kid-core/src/movement.rs` and `crates/kid-core/src/lib.rs`. Find the missing path.

4. **If the cause is clear, implement the fix** + a regression test around frames 7968-7969. Run `mise run verify-slice-fast` before committing.

5. **Do NOT add more debug fields.** The existing 15 probes have exhausted native-side hypothesis rejection. The answer is in the ASM.

## Verification To Re-run

- `cargo test -p kid-core published_tas_frames_7968_7969_report_kid_ -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 7967 --frame-end 7970 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not add more native-side debug probes — 15 sessions of probing found no positive signal.
- Do not retry terrain-write theories for cell 58,28 — support tile matches the generated layout.
- Do not retry goal-touch timing — kid_goal_probe shows lifecycle=alive with touch=0.
- Do not retry special-motion latches — kid_special_motion_probe shows all idle.
- Do not retry same-frame stomp — collision-handoff probe shows stomp_candidate=0.
- Do not retry transform/helmet-change — pending_form=none, form_flash=0.
- Do not retry wall-cling release, Iron Knight attach, sub_B270, floor collision, wall collision, or apply_jump_and_gravity_vel_only in isolation.
- Do not retry kid_flip fan-out or shared follow-on stages — probed and idle.
- Do not retry pre-sub_B168 stomp/collision candidate carry — probed and empty.
