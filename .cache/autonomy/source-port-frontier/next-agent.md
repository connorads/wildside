---
slug: any-percent-published-frame-8056-demo-1-climb-drift-after-timeout-fix
cycle: 35
updated_at: 2026-03-23T03:26:58Z
---

# Source-Port Frontier Handoff

## What Changed

- Read the live Iron Knight climb path in `kid.asm` instead of reopening timeout logic: grounded attach still enters `loc_7664 -> sub_922C -> loc_A276`, and the active climb loop is `loc_A28A -> Get_RandomNumber_wordC -> sub_B55C -> sub_902A/sub_A3BC`.
- Re-ran `tas-diff` on frames `8054..8058` with physics traces. That confirmed the first authoritative miss is still frame `8056`, with native still taking `motion=iron_knight_climb_resume` while oracle freezes Kid on the previous climb pose for one frame and then leaves the wall with non-zero leftward carry on `8057`.
- Checked the checked-in scenario masks around `8052..8058`. The raw JSON bytes are only `down+a` / idle there, so do not treat the scenario file alone as authoritative for the live held-input interpretation; the demo/input path still needs ROM-oracle confirmation.
- Did not ship a gameplay patch. The remaining ambiguity is branch selection, not constants, and another speculative change would be guesswork.

## Current Model

- Demo 1 timeout parity is still fixed. The live frontier remains the later Iron Knight climb drift at frame `8056`.
- Native is still staying inside the dedicated climb-resume path too long. On frame `8056`, native advances Kid to `0x01855680`, but oracle keeps Kid at `0x0186d680` for that frame.
- On the next oracle frame (`8057`), Kid has already left the pinned wall-climb pose and now carries left/up motion (`kid_x_pos=0x03a7d500`, `x_vel=0xffffd500`, `y_vel=0xfffec000`), while native is still pinned at `x=0x03a80000` with `x_vel=0`.
- The ROM constants are not the problem. `Get_RandomNumber_wordC` uses the same `+$80`, `-$1800`, `min -$800`, `max +$1800`, velocity clamp `[-$18000, +$40000]` model already reflected in native.
- The likely miss is exit control flow from `loc_A28A`, not climb acceleration. The relevant branches are:
  - `loc_A2E2/loc_A2F6`: exit when held direction requests release.
  - `sub_B55C` at `loc_A30A`: single-cell forward/up wall gate for staying on the wall.
  - The follow-on `loc_A6F8`/normal movement path after leaving wall climb.
- Do not assume the raw TAS JSON bytes equal the live gameplay held-input view during this demo window. The scenario dump and the live trace disagree about the exact button story, so use oracle/ROM evidence before patching.

## Strongest Evidence

- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8057 --include-physics-trace --include-advisory`
  - Frames `8054..8055` remain advisory-only.
  - Frame `8056` is the first authoritative miss: native `kid_y_pos=0x01855680`; oracle `kid_y_pos=0x0186d680`.
  - Native frame `8056` still says `motion=iron_knight_climb_resume`, `x_vel=0`, `wall_cling=none`, `iron_knight_climbing=1`.
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8056 --frame-end 8059 --include-physics-trace --include-advisory`
  - Oracle frame `8057`: `0x30(kid_gfx)@0x03a7d500,0x01859680 vel=0xffffd500,0xfffec000`.
  - Native frame `8057`: still `kid_x_pos=0x03a80000`, `x_vel=0`, `motion=iron_knight_climb_resume`.
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `loc_7664`: grounded Iron Knight attach still gates through `sub_922C` and jumps to `loc_A276`.
  - `loc_A276` / `loc_A28A`: live wall-climb loop.
  - `Get_RandomNumber_wordC`: climb accel/velocity update.
  - `sub_B55C`: one-cell forward/up wall gate from `loc_A30A`.
- `node -e 'const s=require("./.cache/tas-imports/any-percent-published/scenario.json").script; ...'`
  - Raw scenario bytes around `8052..8058` are `66/0` (`down+a` / idle), which is useful only as a warning not to trust the JSON slice alone for live held-input interpretation.
- `crates/kid-core/src/lib.rs`
  - Demo live gameplay uses `input` directly once `demo_live_input` is active, so the remaining mismatch is in motion/control flow, not the earlier delayed-input bootstrap path.

## Exact Next Moves

1. Read the ROM path after wall-climb exit, not just the stay-on-wall branch: follow `loc_A2E2`, `loc_A2F6`, `loc_A30A`, and the `loc_A6F8` path they feed so the native exit frame matches oracle's frozen `8056` then left/up `8057`.
2. Use oracle evidence for held input, not the raw scenario JSON slice. Watch the relevant controller bytes (`Ctrl_Left_Held`, `Ctrl_Right_Held`, `Ctrl_Down_Held`, `Ctrl_C_Held`, and if needed `Ctrl_Pressed`) across `8055..8058` so the release branch is chosen from ROM truth.
3. Patch only the Iron Knight climb exit frame. Goal: native must stop reporting `motion=iron_knight_climb_resume` at the wrong point, pick up the same first leftward carry as oracle, then rerun the focused `8054..8059` diffs, the `6975..9530` bisect, and `mise run verify-slice-fast`.

## Verification To Re-run

- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 8054 --frame-end 8059 --include-physics-trace --include-advisory`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6975 --frame-end 9530 --bisect`
- `cargo test -p kid-core 7968 -- --nocapture`
- `cargo test -p kid-core 7969 -- --nocapture`
- `cargo test -p kid-core iron_knight_ -- --nocapture`
- `cargo test -p kid-core published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay -- --nocapture`
- `cargo test -p kid-core published_tas_demo_1_times_out_on_frame_8069 -- --nocapture`
- `cargo test -p kid-core published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966 -- --nocapture`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old frame-8004 timeout/title-return theory. Timeout parity is already fixed through the real `8068 -> 8069` boundary.
- Do not tweak Iron Knight climb constants again. The ROM `Get_RandomNumber_wordC` math already matches the native constants; the miss is branch/control flow.
- Do not assume the raw scenario JSON bytes tell you the live held-input answer for this window. They are useful for spotting inconsistencies, not for choosing the final fix.
- Do not keep the player pinned in `iron_knight_climb_resume` just because `probe_iron_knight_climb_resume_wall` still sees a wall cell. The oracle proves the ROM leaves that state earlier.
