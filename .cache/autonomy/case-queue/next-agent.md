---
slug: death_respawn_level1_case_00000_00200
cycle: 60
updated_at: 2026-04-01T03:13:39Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `60` selected `death_respawn_level1_case_00000_00200` again and kept it evidence-first.
- Re-verified only the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails on frame `172` at `player.x_pos`.
- Refreshed the reduced artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the ROM path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `LoadGameModeData`, `Load_InGame`, and `loc_7526`.
- Reconfirmed from native trace code that `crates/kid-core/src/trace.rs` still suppresses the hidden pose before `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`, then hardcodes Level 1 `player.x_pos = 0x0020_0000`, which matches the map header seed rather than the oracle preload.
- Left gameplay code untouched.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `60`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656` (`0x0128_0000`)

## Why This Case

- The autonomous loop selected the same case again, so the session stayed on this case only.
- The task package is still evidence-only, so the correct scope stayed: re-verify, tighten the explanation, and avoid a symptom patch.
- The first divergence is still on the title-start `Load_InGame` bootstrap path, before visible gameplay entities or events appear in the reduced semantic shell.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `60`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `.cache/cases/death-respawn-test/oracle-semantic.json` vs `.cache/cases/death-respawn-test/native-semantic.json`
  - frames `168..171`: both sides still show `player=(0,0)`, `camera=(0,0)`, `entities=[]`, `events=[]`
  - frame `172`: oracle jumps to `player=(0x0128_0000,0x0100_0000)` while native stays at `player=(0,0)`
  - implication: the first authoritative mismatch is a lone hidden preload pose, not a broader shell or camera divergence
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` sets `Game_Mode=$0008` then jumps to `loc_6E2`
  - `loc_6E2` clears VRAM/object slots and dispatches through `LoadGameModeData`
  - `Load_InGame` writes `PlayerStart_X_pos/Y_pos` from the level header at `loc_11BE4`
  - `loc_7526` later allocates Kid's gfx object and copies `PlayerStart_X_pos/Y_pos` into `x_pos(a3)` / `y_pos(a3)`, then subtracts `1` from `y_pos(a3)`
  - implication: the ROM path read this session explains header-seeded ownership for `x=$0020`, `y=$0100`, and the later `y-1` gfx slot adjustment, but it still does not prove any ROM writer for the oracle's lone `x=$0128` preload coordinate while the semantic shell remains otherwise empty
- `crates/kid-core/src/trace.rs`
  - `early_start_load_ingame_player_state()` still returns `None` before `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`
  - once visible, Level 1 still hardcodes `player.x_pos = 0x0020_0000`, which matches the header seed, not the oracle preload pose at frame `172`

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or other case IDs.
2. Map reduced frame `172` onto the exact hidden `BootState::StartingLevel` timeline, then identify which native trace producer should expose the first non-zero preload pose.
3. Trace the ROM path after `Load_InGame` and around the first Kid object/gfx-object handoff to determine whether the oracle's `x=$0128` comes from a later writer than `PlayerStart_X_pos`.
4. If a later session patches this, do not tune only `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`; the `x` source must be proved from ASM or a tighter oracle correlation first.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any evidence or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not treat this reduced case as evidence about the later death/respawn bootstrap; frame `172` is still on the opening title-start lineage.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` from the symptom alone. The hidden-pose gate is real, but it does not explain the wrong `x` source.
- Do not assume the Level 1 header seed solves the case; the ROM header start is `0x0020_0000`, while the oracle preload is `0x0128_0000`.
- Do not assume `loc_7526` alone proves the preload `x`; it only copies `PlayerStart_*` and applies the `y-1` adjustment.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
