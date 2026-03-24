---
slug: any-percent-published-frame-10986-title-briefly-stays-in-0x10-oracle-window
cycle: 51
updated_at: 2026-03-24T02:51:34Z
---

# Source-Port Frontier Handoff

## What Changed

- Re-read `loc_1BA28`, `loc_1BA5C`, `sub_1C204`, and `sub_1C246` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Added a timeout-restart override for intro sequence 5 so native no longer reuses the shared 80-frame intro timer; it now models `sub_1C204`'s fade delay plus `loc_1BA5C`'s long DBF loop.
- Added title-handoff modelling for the Start-driven `sub_1C246` exit from timeout-restart intro sequence 5: title mode keeps `loc_1BCD0`'s slot-2 pose through frame 10317, blanks Kid on 10318, and restores the standard title pose on 10438.
- Added focused regressions for the old frame-10029 false frontier and the later 10288/10290/10318/10438 intro-sequence-5 title handoff.
- Cleared the frame-10029 timeout-restart intro-sequence-5 blocker; `verify-tas --frame-start 9531 --bisect` now moves the first CRC divergence to frame 10986.

## Current Model

- The old frame-10029 break was genuine native timing debt. The timeout-restart copy of intro sequence 5 does not naturally end after 80 frames; `loc_1BA28` first runs the shared fade helper `sub_1C204`, and only then does `loc_1BA5C` start its `move.w #$1BC,d0` / `Hibernate_Object_1Frame` / `dbf` loop.
- In the published TAS, that natural timeout still does not fire. The script presses Start on frame 10288 (`mask=0x80`), `sub_1C246` handles that during the long intro-sequence-5 loop, and title only becomes visible two frames later on 10290.
- Oracle slot-2 WRAM proves the direct title return keeps `loc_1BCD0`'s slot-2 pose (`x=0`, `y=$E0`) visible through title frame 10317, clears it on 10318, then restores the standard title pose on 10438.
- The next live blocker is now later and smaller: on frames 10986..10987 oracle briefly reports `game_mode=0x0010` while native stays in title mode `0x002C`. Frames 10982..10985 match, frames 10986..10987 diverge only on `game_mode`, and frames 10988..10989 match again.

## Strongest Evidence

- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `loc_1BA28`: allocates the intro-sequence-5 workers, stores `#loc_1BA5C,$44(a5)`, then calls `sub_1C204`.
  - `sub_1C204`: runs the fade-helper loop (`Hibernate_Object_1Frame`, `sub_1C246`, `addi.w #$10,($FFFFF876).w`) before dispatching `$44(a5)`.
  - `loc_1BA5C`: starts `move.w #$1BC,d0`, loops on `Hibernate_Object_1Frame` + `sub_1C246` + `dbf`, and only its natural path eventually leaves intro sequence 5.
  - `sub_1C246`: on Start, writes `move.w #$2C,(Game_Mode).w` immediately without rebuilding the title gfx slots first.
- Native/oracle diagnostics
  - `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
    - all 10 focused timeout-restart regressions pass
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10025 --frame-end 10033 --include-advisory`
    - frames `10025..10032`: advisory-only (`number_objects`) after the intro-sequence-5 timing fix
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 0 --frame-end 10520 --watches 0xFFFBCA:u16,0xFF24FA:u32,0xFF24FE:u32 | awk -F, 'NR==1 || ($1+0)>=10284 && ($1+0)<=10325 {print}'`
    - frame `10288`: `Game_Mode=0x0024`, slot 2 still `x=0`, `y=0x00E00000`
    - frame `10290`: `Game_Mode=0x002C`, slot 2 still `x=0`, `y=0x00E00000`
    - frame `10318`: `Game_Mode=0x002C`, slot 2 clears back to zero
  - `cargo run -q -p kidcli -- oracle-probe --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 0 --frame-end 10620 --watches 0xFF24FA:u32,0xFF24FE:u32 | awk -F, 'NR==1 || ($1+0)>=10430 && ($1+0)<=10510 {print}'`
    - frame `10438`: slot 2 first restores the standard title pose `x=0x002E0000`, `y=0x008C0000`
  - `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
    - `RESULT: first CRC divergence at frame 10986`
  - `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10982 --frame-end 10990 --include-advisory`
    - frames `10982..10985`: `MATCH`
    - frames `10986..10987`: `game_mode native=0x002c oracle=0x0010`
    - frames `10988..10989`: `MATCH`
  - `mise run verify-slice-fast`
    - passes

## Exact Next Moves

1. Read the title-shell ASM that can briefly write `Game_Mode=$10` and then return to the normal title state by frame 10988. Start from the title-side code reached after `sub_1C246` rather than reopening intro-sequence-5 timing.
2. Add a focused regression for the new transient window: frame 10985 should still match title, frames 10986..10987 should expose the oracle's `game_mode=0x0010`, and frame 10988 should return to the matched title state.
3. Fix the native title-state machine so the brief `0x0010` excursion is reproduced, then re-run the frame-9531 bisect.

## Verification To Re-run

- `cargo test -q -p kid-core published_tas_second_timeout_restart_ -- --nocapture`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10025 --frame-end 10033 --include-advisory`
- `cargo run -q -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 10982 --frame-end 10990 --include-advisory`
- `cargo run -q -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 9531 --bisect`
- `mise run verify-slice-fast`

## Dead Ends / Do Not Retry

- Do not reopen the old shared-80-frame intro-sequence-5 timer theory. `loc_1BA28` + `sub_1C204` + `loc_1BA5C` already prove the timeout-restart copy is much longer.
- Do not clear the intro-sequence-5 carry latch on the Start-press frame itself. Oracle keeps `loc_1BCD0`'s slot-2 pose visible through the 2-frame deferred `sub_1C246` title handoff.
- Do not guess from the title pixels alone. Use the fixed slot-2 WRAM words (`$FF24FA/$FF24FE`) and `Game_Mode` when checking the 10288..10438 title rebuild window.
