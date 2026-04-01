---
slug: death_respawn_level1_case_00720_00900
cycle: 24
updated_at: 2026-04-01T00:18:09Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 24 selected `death_respawn_level1_case_00720_00900`.
- Re-verified the selected case with `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`.
- Read the ROM death / respawn shell in `.cache/upstream/kid-chameleon-disasm/kid.asm:14714` before any gameplay edit; `Death -> lose_life -> Teleport` still writes `Game_Mode = #8`, so the case remains centred on a native/oracle mode-order mismatch, not an unexplained gameplay guess.
- Tightened the case verifier in [`case.rs`](/home/connor/git/kc/crates/kidcli/src/case.rs) so `matching_fields` are derived from the actual semantic comparisons instead of a hard-coded list.
- Added a regression test covering a `world.game_mode` mismatch, then refreshed the selected case summary with `cargo run -p kidcli -- case summarise --case death-respawn-level1-720-900`.

## Selected Case

- Case ID: `death_respawn_level1_case_00720_00900`
- Selector: `death-respawn-level1-720-900`
- Autonomy gate: `evidence-only`
- Verify result: `FAIL`
- First live failure: frame `720`, `world.game_mode`, native `8`, oracle `12`

## Why This Case

- The loop spec says to take the selected case only from `.cache/autonomy/case-queue/next-task.json`.
- This case was already narrowed to a 180-frame replay-matched window, so the useful work here was to re-verify it and tighten the evidence before considering any wider gameplay patch.
- The stale verifier output was masking the real shape of the mismatch by claiming several categories matched when they did not.

## Strongest Evidence

- `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900` still returns `case death_respawn_level1_case_00720_00900: FAIL`.
- `.cache/cases/death-respawn-level1-720-900/verify.json` now records `world.game_mode` as the first failure at frame `720` with native `8` and oracle `12`.
- The refreshed `.cache/cases/death-respawn-level1-720-900/summary.md` now reports `matching_fields: entities, events`; camera and player categories no longer appear as false positives.
- ASM `.cache/upstream/kid-chameleon-disasm/kid.asm:14772` (`Teleport`) confirms the death / respawn shell explicitly writes `move.w #8,(Game_Mode).w` before jumping into `loc_6E2`.

## Exact Next Moves

1. Use the tightened case bundle to probe which native path advances from respawn mode `8` to in-game mode `12` too early around frame `720`.
2. Read the ROM routine reached from `loc_6E2` and the native equivalent before changing gameplay code.
3. Keep the work scoped to `death_respawn_level1_case_00720_00900`; if the next evidence shows an earlier inherited divergence, re-queue rather than widening ad hoc.

## Verification To Re-run

- `cargo test -p kidcli compare_semantic_states`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-720-900`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-720-900`
- `sed -n '14714,14830p' .cache/upstream/kid-chameleon-disasm/kid.asm`

## Dead Ends / Do Not Retry

- Do not trust old `matching_fields` output from before this verifier fix; it was hard-coded and could contradict `first_failure`.
- Do not patch random respawn timing from the case summary alone; the next gameplay step still needs the post-`Teleport` ASM path.
- Do not widen to other death-respawn windows in this session; this handoff is only for `death_respawn_level1_case_00720_00900`.
