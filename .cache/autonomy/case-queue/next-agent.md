---
slug: death_respawn_level1_case_00000_00200
cycle: 51
updated_at: 2026-04-01T03:40:00Z
---

## What Changed

- Re-ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle `51` selected `death_respawn_level1_case_00000_00200` again and kept it `evidence-only`.
- Re-verified the case with `cargo run -p kidcli -- case verify --case death-respawn-test`; it still fails first on frame `172` at `player.x_pos` (`native=0`, `oracle=19398656`).
- Refreshed the reduced-case artefacts with `cargo run -p kidcli -- case summarise --case death-respawn-test`.
- Re-read the relevant ROM paths in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `Obj_TitleMenu loc_1BB52`, `Title_InputLoop`, `Load_TitleCard`, `loc_1AB26`, `loc_1AC70`, `loc_1AD2A`, `loc_1AE2E`, `loc_6E2`, and `Load_InGame`.
- Tightened the hypothesis for frame `172`: the oracle pose `x=$0128`, `y=$0100` matches `loc_1AC70 -> loc_1ACFE`'s seeded off-screen title-card worker start pose pattern better than the later `Load_InGame` Level 1 settle path.
- Tried a local proof patch that reused the normal level-entry title-worker model for this early title-start lineage, then reverted it. That patch exposed the wrong pose (`x=$0160`, `y=$00A4`) and regressed later bootstrap expectations, so it is not the right implementation path.

## Selected Case

- Case ID: `death_respawn_level1_case_00000_00200`
- Selector: `death-respawn-test`
- Cycle: `51`
- Current result: `FAIL`
- Current first failure: frame `172`, field `player.x_pos`, native `0`, oracle `19398656`

## Why This Case

- The autonomous selector chose the same case again, so the session stayed on this case only.
- The task package still marks it `evidence-only`, so the job remained: re-verify, read the ROM, and tighten the case rather than patch blindly.
- The durable surprise this session is narrower and stronger than before: frame `172` is probably not a hidden `Load_InGame` player pose. It looks like a transient title-card worker seed from `Load_TitleCard`.

## Strongest Evidence

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
  - cycle `51`
  - status `stalled`
  - selected case still `death_respawn_level1_case_00000_00200`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
  - still `FAIL`
  - first bad frame/field unchanged: frame `172`, `player.x_pos`
- `.cache/cases/death-respawn-test/oracle-semantic.json`
  - frames `166..171`: `player=(0,0)`, `game_mode=8`, `camera=(0,0)`
  - frame `172`: `player=(0x0128_0000,0x0100_0000)`, `game_mode=8`, `camera=(0,0)`
  - implication: the pose appears as a one-step hand-off inside mode `8`, not as the later settled Level 1 spawn already modelled in native trace helpers
- `.cache/upstream/kid-chameleon-disasm/kid.asm`
  - `loc_1AC70 -> loc_1ACFE` seeds title-card letter workers from one of four off-screen start poses
  - selector `3` uses `y=$0100` and a random `x=((rand_byte + $3E) & $FFFC)`
  - implication: oracle's `y=$0100` on frame `172` is structurally aligned with this worker path
- Rejected local patch attempt
  - preserving the generic title-worker roster on this lineage produced `frame 172 = (x=$0160,y=$00A4)` instead of oracle's `(x=$0128,y=$0100)`
  - it also broke later title/bootstrap expectations, including `death_respawn_level_1_keeps_bootstrap_hidden_on_frame_200` and `death_respawn_level_1_matches_load_ingame_handoff_through_frame_300`
  - implication: do not reuse the existing level-entry title-card timing/RNG model here without tracing the exact early-title-start RNG and worker lifecycle first

## Exact Next Moves

1. Stay on `death_respawn_level1_case_00000_00200` only; do not widen into later respawn windows.
2. Trace the early title-start path from `Title_InputLoop` into `Load_TitleCard` and confirm exactly when `loc_1AB26` / `loc_1AC70` first become trace-visible on this lineage.
3. Inspect the RNG consumption between the frame-`137` Start press and oracle frame `172`; the naive reuse patch proves the current title-worker RNG sequence is not the one the oracle is following here.
4. If the ROM confirms this is a special trace-only worker carry, implement a narrow trace-path fix instead of reusing full `starting_level_title_workers`.
5. Re-run the selected case and only patch gameplay/trace once the frame-`172` pose and timing both match the ROM path.

## Verification To Re-run

- `cargo run -p kidcli -- autonomous-loop --goal case-queue`
- `cargo run -p kidcli -- case verify --case death-respawn-test`
- `cargo run -p kidcli -- case summarise --case death-respawn-test`
- `cargo test -p kid-core death_respawn_level_1_exposes_title_menu_cursor_on_frame_136 -- --nocapture`
- `cargo test -p kid-core death_respawn_level_1_clears_title_cursor_pose_before_title_shell_counts -- --nocapture`

## Dead Ends / Do Not Retry

- Do not treat frame `172` as the later Level 1 `Load_InGame` settled spawn without more proof; the oracle pose shape fits `loc_1AC70` better than the current `Load_InGame` helper.
- Do not naively preserve `starting_level_title_workers` on the early title-start lineage. The attempted patch produced the wrong seeded pose (`$0160,$00A4`) and broke later bootstrap expectations.
- Do not widen into `pause_resume_level1_case_00000_00200` or `death_respawn_level1_case_00300_00500`; this session is only about `death_respawn_level1_case_00000_00200`.
- Do not re-open the old “player physics/collision” hypothesis from `verify.json` until the title/bootstrap worker evidence is either confirmed or falsified from the ROM.
