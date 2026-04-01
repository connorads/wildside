---
slug: death_respawn_level1_case_00000_00200
cycle: 62
updated_at: 2026-04-01T04:03:31Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `62` selected `death_respawn_level1_case_00000_00200` again and kept it stalled/evidence-first.
- Re-verified only the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails on frame `172` at `player.x_pos`.
- Refreshed the reduced artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM bootstrap path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_InGame`, `loc_11BE4`, `loc_11F48`, and `loc_7526`.
- Reconfirmed in `crates/kid-core/src/trace.rs` that the hidden mode-`$0008` player trace still returns `None` until `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`, then hardcodes Level 1 `player.x_pos = 0x0020_0000`.
- Left gameplay code untouched because the selected case is still evidence-only and the ROM proof for oracle `x=$0128_0000` is still missing.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `62`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656` (`0x0128_0000`)

## Why This Case

- The case-queue loop selected the same case again, so the session stayed on this case only.
- The task package still marks it evidence-only, so the correct scope was to re-verify, tighten the explanation, and avoid a speculative patch.
- The first authoritative divergence still lands on the hidden title-start `Load_InGame` bootstrap, before visible gameplay or entity drift appears in the reduced shell.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `62`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `.cache/cases/death-respawn-test/oracle-semantic.json` vs `.cache/cases/death-respawn-test/native-semantic.json`
  - frames `168..171`: both sides still show `player=(0,0)`, `camera=(0,0)`, `entities=[]`, `events=[]`
  - frame `172`: oracle jumps to `player=(0x0128_0000,0x0100_0000)` while native stays at `player=(0,0)`
  - implication: the first authoritative mismatch is still a lone hidden preload pose
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008` then jumps to `loc_6E2`
  - `loc_6E2` clears VRAM/object slots and dispatches through `LoadGameModeData`
  - `Load_InGame` at `loc_11BE4` seeds `PlayerStart_X_pos/Y_pos` from the map header
  - `Load_InGame` at `loc_11F48` derives camera preload from those same start coordinates
  - `loc_7526` allocates Kid's gfx object, copies `PlayerStart_X_pos/Y_pos` into `x_pos(a3)` / `y_pos(a3)`, then subtracts `1` from `y_pos(a3)`
  - implication: the ROM path re-read this session still proves the Level 1 header seed `x=$0020`, `y=$0100`, but still does not explain the oracle-only hidden `x=$0128`
- `crates/kid-core/src/trace.rs`
  - `early_start_load_ingame_player_state()` still returns `None` before `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`
  - once visible, Level 1 still hardcodes `player.x_pos = 0x0020_0000`
  - implication: native is still exposing the header seed late, while oracle exposes a different hidden preload pose one frame earlier in the reduced case

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or other case IDs.
2. Map reduced frame `172` onto the exact `BootState::StartingLevel` elapsed-frame timeline so the hidden trace producer is identified precisely.
3. Trace the ROM path after `loc_7526` and the first collision/position settle writes to find whether a later writer moves Kid from header `x=$0020` to oracle `x=$0128` before the reduced shell becomes visible.
4. If a later session patches this, do not tune only `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`; prove the `x` source from ASM or tighter oracle correlation first.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not pass the raw case ID to `kidcli case verify`; the CLI expects the case selector (`death-respawn-test`) from the task package.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` from the symptom alone; visibility timing does not explain the wrong hidden `x` source.
- Do not assume the Level 1 header seed solves the case; the ROM header start is `0x0020_0000`, while the oracle preload is `0x0128_0000`.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
