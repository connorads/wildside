---
slug: death_respawn_level1_case_00000_00200
cycle: 38
updated_at: 2026-04-01T01:30:31Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 38 selected `death_respawn_level1_case_00000_00200` (`death-respawn-derived-check`).
- Re-verified the selected case first. It still fails on frame 136 at `player.x_pos`.
- Re-ran the live gameplay window for `death-respawn-level1` frames `0..200`; it fails with the same first bad frame and field, so this is not another stale case bundle.
- Read the relevant ROM paths before considering any gameplay change:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`: `lose_life`, `Teleport`, `loc_6E2`, `Load_InGame`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Load_TitleCard`, `loc_1AB26`, `loc_1AD48`
- Derived and verified a tighter repro window at frames `130..141` (`death_respawn_level1_case_00130_00141`). It preserves the same failure signature at frame 136 while cutting the evidence window from 200 frames to 12.
- Proved one concrete native trace bug while investigating: `crates/kid-core/src/trace.rs` gates `early_start_load_ingame_object_counts()` behind a `BootState::InGame | DemoPlay` check, so `StartingLevel` traces can never expose those object counts. I did not patch it because the selected case is actually failing in title mode (`Game_Mode=$002C`), not in the later `StartingLevel` bootstrap branch.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-derived-check`
- Autonomy gate at selection time: `evidence-only`
- Current verify result: `FAIL`
- First failure: frame `136`, field `player.x_pos`, native `0`, oracle `2621440`

## Why This Case

- The loop selected this exact case for cycle 38, so the session stayed on `death-respawn-derived-check` only.
- The immediate question was whether the old `0..200` failure was another stale gameplay-case artefact after cycle 37's canonicalisation fix.
- Re-verification and direct gameplay-window comparison show the failure is real, but a full gameplay fix would still be guesswork without first identifying the exact title-mode carry path that owns frame 136.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-derived-check` still fails on frame 136 at `player.x_pos`.
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 0 --frame-end 200 --checkpoint-interval 1` fails on `frame_00136@136` with `kid_x_pos native 00000000, oracle 00280000`.
- Oracle checkpoints for the selected case show:
  - frame 135: `game_mode=44`, `kid_x_pos=0`, `number_objects=4`, `number_gfx_objects=2`
  - frame 136: `game_mode=44`, `kid_x_pos=0x0028_0000`, `kid_y_pos=0x0030_0000`, `number_objects=4`, `number_gfx_objects=3`
- Native checkpoints for the same frames still show `game_mode=44` but keep `kid_x_pos=0` and `(number_objects, number_gfx_objects)=(0,0)`.
- The tighter derived case reproduces the same failure:
  - `cargo run -p kidcli -- case derive --gameplay-scenario death-respawn-level1 --frame-start 130 --frame-end 141 --out /tmp/kc-case-130-141`
  - `cargo run -p kidcli -- case verify --case /tmp/kc-case-130-141`
  - result: `death_respawn_level1_case_00130_00141` still fails first on frame 136 at `player.x_pos`
- ASM read-outs that matter:
  - `lose_life -> Teleport -> move.w #8,(Game_Mode).w -> jmp (j_loc_6E2).w` is real, but it is the wrong branch for frame 136 because the oracle trace is still in title mode (`Game_Mode=$002C`) there.
  - `Load_TitleCard` allocates `loc_1AB26` and `loc_1AE2E`, so the remaining work is to trace which title-mode object/pose should surface as `(x=$28,y=$30)` on frame 136.

## Exact Next Moves

1. Trace the title-mode frame-136 owner, not the later gameplay bootstrap. Start from the `Game_Mode=$002C` / title-screen paths in native trace code and cross-reference against `Load_TitleCard`, `loc_1AB26`, `loc_1AE2E`, and any title-pose carry helpers.
2. Capture the native runtime branch at frame 136 directly: identify which `BootState` variant is active and why native still emits `kid_x_pos=0` and `(number_objects, number_gfx_objects)=(0,0)` while oracle shows `(4,3)` and `(x=$28,y=$30)`.
3. Once the owning title branch is proven, add a focused regression around frame 136 before patching. The 12-frame case `130..141` is the right reproduction window.
4. If a patch lands, re-run both:
   - `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
   - `cargo run -p kidcli -- case verify --case /tmp/kc-case-130-141`

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-derived-check`
- `cargo run -p kidcli -- case summarise --case death-respawn-derived-check`
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 0 --frame-end 200 --checkpoint-interval 1`
- `cargo run -p kidcli -- case derive --gameplay-scenario death-respawn-level1 --frame-start 130 --frame-end 141 --out /tmp/kc-case-130-141`
- `cargo run -p kidcli -- case verify --case /tmp/kc-case-130-141`

## Dead Ends / Do Not Retry

- Do not treat this as another stale case bundle. The direct `compare-native-gameplay-window` run fails on the live scenario too.
- Do not patch `lose_life` / `Teleport` / `Load_InGame` from this case alone. Frame 136 is still `Game_Mode=$002C`, so those gameplay paths are not the immediate owner of the failure.
- Do not land the tempting one-line `EARLY_START_LOAD_INGAME_VISIBLE_FRAME` tweak without first proving the active branch. The oracle pose `(x=$28,y=$30)` does not match the native early `Load_InGame` ramp values.
- Do not rely on `cargo run -p kidcli -- case reduce --case death-respawn-derived-check` for this window right now. It spent minutes in brute-force minimisation without returning useful output; deriving the focused `130..141` case is the practical alternative.
