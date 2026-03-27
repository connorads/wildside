---
slug: any-percent-published-frame-27603-mode-24-kid-y-blank
cycle: 71
updated_at: 2026-03-27T04:19:54Z
---

# Source-Port Frontier Handoff

## What Changed

- Fixed the frame-24921 level-0x15 gameplay blocker by stopping [`apply_wall_collision()`](/home/connor/git/kc/crates/kid-core/src/movement.rs) from promoting `probe_wall_top_edge()` into a fresh airborne wall hit.
- Added [`airborne_top_edge_contact_does_not_start_wall_cling_without_direct_wall_hit`](/home/connor/git/kc/crates/kid-core/src/tests.rs), locking the `sub_8F26` rule that a top-edge-only overhang contact may retain an existing sticky cling but must not create a new one.
- Re-ran `cargo test -q -p kid-core airborne_top_edge_contact_does_not_start_wall_cling_without_direct_wall_hit -- --nocapture`, `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 24918 --frame-end 24928 --checkpoint-interval 1`, `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --include-advisory`, and `mise run verify-slice-fast`; the first CRC divergence now moves to frame 27603.

## Current Model

- The old level-0x15 miss was not a late `sub_ADB8` / `sub_AE6E` release bug. Native was entering wall-cling too early because [`apply_wall_collision()`](/home/connor/git/kc/crates/kid-core/src/movement.rs) treated a top-edge-only overhang probe as if ASM `sub_8F26` had reported a direct wall hit.
- ASM [`sub_8F26`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm) only sets `$FFFFFA72` after the direct `sub_922C` side-wall probe succeeds. Top-edge retention belongs to the later jump helpers once cling is already active.
- With that false attach removed, the next blocker is frame 27603: native blanks `kid_y_pos` to `0x00000000` for one frame in `mode=0x0024`, `level=0x00`, while oracle keeps `kid_y_pos=0x00E00000`.

## Strongest Evidence

- ASM / script
  - [`kid.asm`](/home/connor/git/kc/.cache/upstream/kid-chameleon-disasm/kid.asm)
    - `sub_8F26` writes `$FFFFFA72` only after the direct wall collision path; there is no top-edge fallback in that routine.
- Native
  - [`movement.rs`](/home/connor/git/kc/crates/kid-core/src/movement.rs)
    - `apply_wall_collision()` now uses `probe_wall_top_edge()` only for sticky retention paths, not fresh airborne latches.
  - [`tests.rs`](/home/connor/git/kc/crates/kid-core/src/tests.rs)
    - `airborne_top_edge_contact_does_not_start_wall_cling_without_direct_wall_hit` passes.
- Oracle / parity
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 24918 --frame-end 24928 --checkpoint-interval 1`
    - CRC-clean across the repaired gameplay window.
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
    - First CRC divergence is now frame 27603.
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --include-advisory`
    - Frames 27598..27601 match, frame 27602 is advisory-only, and frame 27603 diverges solely on `kid_y_pos`.

## Exact Next Moves

1. Read the `mode=0x0024` ASM path that owns frame 27603 before touching native code.
2. Explain why oracle preserves `kid_y_pos=0x00E00000` for one frame while native blanks Kid to zero.
3. Add a focused regression for the one-frame `kid_y_pos` settle only after that write path is proven.

## Verification To Re-run

- `cargo test -q -p kid-core airborne_top_edge_contact_does_not_start_wall_cling_without_direct_wall_hit -- --nocapture`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 24918 --frame-end 24928 --checkpoint-interval 1`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 27598 --frame-end 27608 --include-advisory`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not retune `sub_ADB8` / `sub_AE6E` first. Frame 24921 was caused by a false `sub_8F26`-stage attach before those helpers ran.
- Do not revive the removed generic top-edge fallback in [`apply_wall_collision()`](/home/connor/git/kc/crates/kid-core/src/movement.rs). Existing sticky-retention paths already cover the legitimate overhang cases.
- Do not park the frontier in level-0x15 movement anymore. The rebisect moved the first real mismatch to frame 27603 in the shared `mode=0x0024` hand-off.
