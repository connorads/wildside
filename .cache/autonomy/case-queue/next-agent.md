---
slug: death_respawn_level1_case_00600_00750
cycle: 36
updated_at: 2026-04-01T01:14:20Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 36 selected `death_respawn_level1_case_00600_00750` from `death-respawn-level1-600-750`.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`; it still fails immediately at frame `600` on `player.x_pos`.
- Refreshed `cargo run -p kidcli -- case summarise --case death-respawn-level1-600-750` so the on-disk summary matches the current verify result.
- Read the relevant ROM path before considering any gameplay patch: `loc_B80C`, `lose_life`, `Teleport`, `loc_6E2`, `loc_74C8`, and `Load_InGame` in `.cache/upstream/kid-chameleon-disasm/kid.asm`.
- Checked surrounding reduced cases: `death_respawn_level1_case_00700_00800` and `death_respawn_level1_case_00720_00900` both still pass, so the selected `600..750` failure does not point at a later death/respawn gameplay drift.
- Tried `cargo run -p kidcli -- case reduce --case death-respawn-level1-600-750`; after more than a minute it produced no output or artefacts, so I killed the stray reducer and treated that as evidence.
- Did not patch gameplay code. The selected case remains `evidence-only`.

## Selected Case

- Case ID: `death_respawn_level1_case_00600_00750`
- Selector: `death-respawn-level1-600-750`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First bad frame/field after re-verify: frame `600`, `player.x_pos`

## Why This Case

- The loop selected this exact case from `.cache/autonomy/case-queue/next-task.json`, so the session stayed on `death-respawn-level1-600-750` only.
- Re-verification shows the miss happens on the checkpoint frame itself, before the selected window has a chance to prove any later gameplay divergence.
- The neighbouring reduced cases that start later (`700..800`, `720..900`) already pass, so the likely problem is the `frame_00600` seed or checkpoint reconstruction rather than new death/respawn gameplay logic.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750` still reports `death_respawn_level1_case_00600_00750: FAIL`.
- `.cache/cases/death-respawn-level1-600-750/verify.json` records the first failure at frame `600` on `player.x_pos` with native `4096000` and oracle `2097152`.
- `.cache/cases/death-respawn-level1-600-750/native-semantic.json` starts the case in an impossible shell state relative to oracle: frame `600` has native `hp=0`, `x=4096000`, `y=25755648`, `y_velocity=499712`, while oracle has `hp=2`, `x=2097152`, `y=17760256`, `y_velocity=0`.
- The same selected case does not converge onto the later passing seeds: inside `600..750`, frame `700` still has native `x=6848512`, oracle `x=13172736`, while the separate reduced case `.cache/cases/death-respawn-level1-gameplay/summary.md` (`700..800`) passes.
- ASM says the fatal-hit path is `loc_B80C -> lose_life -> Teleport -> move.w #8,(Game_Mode).w -> jmp (j_loc_6E2).w`, with `loc_6E2` rebuilding the shell before `Load_InGame` restores in-level state. That matches the evidence pointing at checkpoint/bootstrap state, not a justified gameplay patch.

## Exact Next Moves

1. Investigate the `frame_00600` checkpoint builder or case-seeding path for `death-respawn-level1-600-750`; compare how the selected case materialises player/life state against the already-passing `700..800` seed.
2. Fix or instrument `kidcli case reduce --case death-respawn-level1-600-750` so this exact selector can be tightened instead of hanging silently.
3. Once reduction works, replace the broad `600..750` parent with the tightest child that still fails on the checkpoint frame; if the tightest failing child still starts in the same impossible shell state, keep the work in the case/checkpoint pipeline rather than gameplay.
4. Only consider a gameplay patch after a tighter child proves a live divergence after the checkpoint seed is trustworthy; if that happens, re-read `lose_life`, `Teleport`, `loc_6E2`, and `Load_InGame` first.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-600-750`
- `cargo run -p kidcli -- case reduce --case death-respawn-level1-600-750`

## Dead Ends / Do Not Retry

- Do not patch gameplay code from this handoff alone; the selected case is still `evidence-only`, and the ROM read does not justify a new gameplay change yet.
- Do not widen back to another death/respawn or pause/resume case in the same session; cycle 36 selected `death-respawn-level1-600-750`.
- Do not treat the passing `700..800` / `720..900` cases as contradictory noise; they are the key evidence that the selected `600..750` case is seeded badly rather than exposing a later live drift.
- Do not leave `kidcli case reduce --case death-respawn-level1-600-750` running indefinitely; the bounded run here produced no output and needed manual cleanup.
