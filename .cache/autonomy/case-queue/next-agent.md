---
slug: death_respawn_level1_case_00000_00200
cycle: 65
updated_at: 2026-04-01T03:27:55Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `65` selected `death_respawn_level1_case_00000_00200` again and left the case queue `stalled`.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails on frame `172` at `player.x_pos`.
- Re-ran `cargo run -p kidcli -- case summarise --case death-respawn-test` so the reduced artefacts stay aligned with the latest verification pass.
- Re-read the native hidden-bootstrap path in `crates/kid-core/src/types.rs`, `crates/kid-core/src/level.rs`, `crates/kid-core/src/boot.rs`, `crates/kid-core/src/trace.rs`, and `crates/kid-core/src/movement.rs`.
- Re-read the ROM bootstrap path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Title_InputLoop`, `loc_6E2`, `Load_InGame`, `loc_11BE4`, `loc_11F48`, and `loc_7526`.
- Left gameplay code untouched. The task package is still evidence-only, and the selected case bundle still does not capture the hidden `StartingLevel.elapsed_frames` timing needed to map case frame `172` onto the exact native bootstrap frame.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `65`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656` (`0x0128_0000`)

## Why This Case

- The case-queue loop selected the same case again, so the session stayed on this case only.
- The task package still marks it evidence-only, so the correct scope was re-verification plus tighter evidence, not a speculative gameplay patch.
- The reduced case still isolates one authoritative hidden-pose mismatch before camera motion, entities, or events diverge.
- The freshest evidence now shows the remaining blocker more precisely: the semantic bundle captures the mismatch itself, but not the internal bootstrap timing needed to explain which hidden `Load_InGame` frame should expose oracle `x=$0128`.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `65`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `.cache/cases/death-respawn-test/verify.json`
  - `frame_count = 173`
  - `matching_fields` still only include `world.game_mode`, `camera`, `player.motion`, `entities`, and `events`
  - `likely_subsystems` remain `player physics` and `collision`
- `.cache/cases/death-respawn-test/native-semantic.json` vs `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `168..171`: both sides still show `game_mode=8`, `camera=(0,0)`, `entities=[]`, `events=[]`, and `player.x_pos=0`
  - frame `172`: oracle alone jumps to `player=(0x0128_0000,0x0100_0000)` while camera and events still stay at zero
  - oracle also keeps `collision_type=0` across the hidden window, while native leaves collision type unset because the hidden player state is still absent
- `crates/kid-core/src/trace.rs`
  - `early_start_load_ingame_elapsed_frames()` only exposes hidden bootstrap timing while `BootState::StartingLevel { uses_load_ingame_bootstrap: true, .. }` is active
  - `early_start_load_ingame_player_state()` returns `None` until `EARLY_START_LOAD_INGAME_VISIBLE_FRAME`
  - once visible, Level 1 still hardcodes `player.x_pos = 0x0020_0000`, so the current native fallback cannot explain oracle frame `172` `x=0x0128_0000`
- `crates/kid-core/src/boot.rs`
  - `BootState::StartingLevel.elapsed_frames` increments once per step and only then hands off to `InGame`
  - implication: case frame `172` definitely lands inside a hidden bootstrap timeline, but the reduced case artefacts do not preserve the matching `elapsed_frames` value
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `Title_InputLoop` writes `Game_Mode=$0008` and jumps to `loc_6E2`
  - `Load_InGame` `loc_11BE4` seeds `PlayerStart_X_pos/Y_pos` from the map header
  - `loc_11F48` derives camera preload from those same start coordinates
  - `loc_7526` copies `PlayerStart_X_pos/Y_pos` into Kid's gfx object and only subtracts `1` from `y_pos`
  - implication: the ROM path re-read this session still proves the header seed (`x=$0020`, `y=$0100`) and still does not explain the hidden oracle-only `x=$0128`
- Negative evidence from the reduced bundle
  - both semantic files keep `world.boot_phase = null` on every frame in this case window
  - no case artefact records `StartingLevel.elapsed_frames`, so the current reduced case cannot by itself map frame `172` onto the exact hidden bootstrap subframe that needs explanation

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows or other case IDs.
2. Tighten evidence before any gameplay patch: capture or derive a reduced artefact that preserves the hidden `StartingLevel.elapsed_frames` value for case frame `172`.
3. Once that timing exists, trace the ROM path after `loc_7526` for the first hidden write or settle step that can move Kid from header seed `x=$0020` to oracle `x=$0128` while camera still stays at zero.
4. Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` from the symptom alone; the current native visible fallback would only expose the wrong header-seeded `x=$0020`.
5. Re-run `cargo run -p kidcli -- case verify --case death-respawn-test` after any tighter evidence capture or gameplay change and stop if the first bad frame moves earlier.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`

## Dead Ends / Do Not Retry

- Do not pass the raw case ID to `kidcli case verify`; the CLI expects the selector `death-respawn-test`.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
- Do not patch `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` from the visible symptom alone; that only surfaces native `x=$0020`, not oracle `x=$0128`.
- Do not assume the Level 1 header seed solves the case; the ROM path re-read this session still only proves `PlayerStart_X_pos = 0x0020`.
- Do not claim frame `172` already maps to a known `StartingLevel.elapsed_frames` value; the current reduced case bundle does not record that timing.
