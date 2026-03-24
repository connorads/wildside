---
slug: any-percent-published-frame-10029-timeout-restart-native-enters-title-while-oracle-stays-in-intro-sequence-5
cycle: 50
updated_at: 2026-03-24T02:32:06Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `loc_1BA28`, `loc_1BA5C`, `loc_1BCD0`, `loc_1BE3C`, `loc_1BE88`, and `sub_1C204` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Proved with oracle slot probes that the verifier's `kid_x_pos` / `kid_y_pos` are the fixed slot-2 WRAM words at `$FF24FA/$FF24FE`, and that timeout-restart intro sequence 5 keeps them zero through frame 10006 before exposing `loc_1BCD0`'s slot-2 pose (`x=0`, `y=$E0`) on frame 10007.
- Added timeout-restart intro-sequence-5 regression `published_tas_second_timeout_restart_exposes_intro_sequence_5_slot_2_on_frame_10007`.
- Updated native trace modelling so timeout-restart intro sequence 5 exposes the `loc_1BCD0` slot-2 pose from frame 10007 onward.
- Cleared the old frame-10007 authoritative mismatch; the first CRC divergence now moves to frame 10029.

## Current Model

- The frame-10007 blocker was not gameplay at all; it was missing boot-scene trace state. Oracle slot-2 WRAM stays blank through frame 10006, then `loc_1BCD0`'s third gfx allocation becomes visible at frame 10007 while `Game_Mode` remains `$24`.
- Native still exits intro sequence 5 too early. At frame 10029 native has already entered title mode (`game_mode=0x002C`) and snapped to the standard title pose (`kid_x_pos=0x002E0000`, `kid_y_pos=0x008C0000`), while oracle is still in intro sequence 5 (`game_mode=0x0024`) with slot 2 held at `x=0`, `y=0x00E00000`.
- The likely cause is now the intro-sequence-5 duration itself: native still uses the shared 80-frame segment timer, but ASM `loc_1BA5C` runs `move.w #$1BC,d0` followed by a `Hibernate_Object_1Frame` / `sub_1C246` / `dbf` loop after the `sub_1C204` fade dispatch, so this scene cannot legitimately end on local frame 80.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `loc_1BCD0`: third gfx allocation writes `move.w #0,$1A(a1)` and `move.w #$E0,$1E(a1)`.
  - `loc_1BA28`: stores `#loc_1BA5C,$44(a5)` then runs `sub_1C204`.
  - `sub_1C204`: performs the fade helper loop before jumping to `$44(a5)`.
  - `loc_1BA5C`: `move.w #$1BC,d0`, then `Hibernate_Object_1Frame` + `sub_1C246` + `dbf`, and only afterwards writes `move.w #$28,(Game_Mode).w`.
- Native/oracle diagnostics
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
    - all 8 focused timeout-restart regressions pass
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10003 --frame-end 10011 --include-advisory`
    - frames `10003..10005`: `MATCH`
    - frame `10006`: advisory-only (`number_objects`)
    - frames `10007..10010`: advisory-only (`number_objects`) after the slot-2 fix
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9948 --frame-end 10008 --watches 0xFFFBCA:u16,0xFFF85C:u16,0xFF24F3:u8,0xFF24FA:u32,0xFF24FE:u32`
    - frame `10006`: slot 2 inactive / zero, `Game_Mode=0x0024`
    - frame `10007`: slot 2 active (`0xFF`) with `x=0x00000000`, `y=0x00E00000`, `Number_GfxObjects=0x0005`, `Game_Mode=0x0024`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 10029`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10025 --frame-end 10033 --include-advisory`
    - frames `10025..10028`: advisory-only (`number_objects`)
    - frame `10029`: `game_mode native=0x002c oracle=0x0024`, `kid_x_pos native=0x002e0000 oracle=0x00000000`, `kid_y_pos native=0x008c0000 oracle=0x00e00000`

## Exact Next Moves

1. Read `loc_1BA28`, `loc_1BA5C`, and `sub_1C204` together and map the true intro-sequence-5 timeout window, including the fade-helper delay before the `#$1BC` DBF loop starts.
2. Add a focused regression for the new boundary: frame `10028` should still be intro sequence 5 with the `loc_1BCD0` slot-2 pose, and frame `10029` should still remain in `Game_Mode=$24`.
3. Fix the premature intro-sequence-5 -> title transition in native, then re-run the frame-9531 bisect.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10003 --frame-end 10011 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10025 --frame-end 10033 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old camera theory; `Load_IntroSequence5` still does not call `sub_1B222`, and frame 10007 was not another camera bug.
- Do not guess from visible title pixels. The verifier reads fixed slot-2 WRAM directly at `$FF24FA/$FF24FE`; use oracle probes against that slot when checking intro/title trace state.
- Do not keep the shared 80-frame intro-sequence-5 timer without re-reading `loc_1BA5C`; ASM already shows a much longer post-fade DBF countdown before `Game_Mode=$28`.
