---
slug: death_respawn_level1_case_00130_00141
cycle: 39
updated_at: 2026-04-01T01:45:31Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 39 selected `death_respawn_level1_case_00130_00141` (`death-respawn-derived-check-tight-130-141`).
- Re-verified the selected case before patching; it failed on frame 136 at `player.x_pos`.
- Read the relevant ROM title path first:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Obj_TitleMenu`, `loc_1BAEA`, `loc_1BB52`, `Title_InputLoop`
- Fixed the native early title-start hand-off to match that ASM:
  - title-menu arm timing now accepts the frame-135 Start edge
  - the title cursor only becomes visible on the following frame, matching `loc_1BB52`
  - the second Start edge is now queued one frame before `StartingLevel`, matching `Title_InputLoop`
  - trace output now carries the title-shell counts/cursor into the early mode-8 bootstrap window instead of dropping straight to hidden gameplay
- Added a focused regression test for frames 130, 136, and 138 of `death_respawn_level1`.

## Selected Case

- Case ID: `death_respawn_level1_case_00130_00141`
- Selector: `death-respawn-derived-check-tight-130-141`
- Autonomy gate: `patch-allowed`
- Current verify result: `PASS`

## Why This Case

- Cycle 39 selected this exact 11-frame window, so the work stayed on the chosen case only.
- The failure was narrow enough to trace to one ROM-owned title-menu hand-off instead of widening into later gameplay bootstrap behaviour.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case .cache/cases/death-respawn-derived-check-tight-130-141`
  - result: `case death_respawn_level1_case_00130_00141: PASS`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 141 --checkpoint-interval 1`
  - result: `native gameplay window comparison passed`
- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136 -- --nocapture`
  - result: `ok`
- ASM basis:
  - `Obj_TitleMenu loc_1BAEA` keeps the title shell alive before the first Start edge
  - `Obj_TitleMenu loc_1BB52` spawns the `$28,$30` cursor on the following frame
  - `Title_InputLoop` defers the `Game_Mode=$0008` hand-off by one traced frame

## Exact Next Moves

1. Re-run the larger duplicate window `death-respawn-derived-check` (`0..200`) and confirm the first bad frame moved beyond 141.
2. If the larger window now fails later in the same early mode-8 lineage, use the existing oracle samples around frames 160..174 before patching anything else.
3. Do not widen into the later death/respawn frontier until the duplicate `0..200` case is either green or has a new first bad frame pinned down.

## Verification To Re-run

- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136 -- --nocapture`
- `cargo run -p kidcli -- case verify --case .cache/cases/death-respawn-derived-check-tight-130-141`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 130 --frame-end 141 --checkpoint-interval 1`
- `cargo run -p kidcli -- case summarise --case .cache/cases/death-respawn-derived-check-tight-130-141`

## Dead Ends / Do Not Retry

- Do not go back to `lose_life`, `Teleport`, or later `Load_InGame` gameplay logic for this case. The selected window was still owned by the title-menu hand-off.
- Do not make the title cursor visible on the same frame as the first Start edge. Oracle only exposes `loc_1BB52` on the following frame.
- Do not switch from title menu straight to `StartingLevel` on the same frame as the second Start edge. Oracle keeps frame 137 in `Game_Mode=$002C` and only reaches mode 8 on frame 138.
