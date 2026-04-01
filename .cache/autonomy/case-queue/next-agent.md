---
slug: death_respawn_level1_case_00600_00750
cycle: 37
updated_at: 2026-04-01T01:21:38Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 37 re-selected `death_respawn_level1_case_00600_00750`.
- Re-verified the selected case before changing code. It still reported `FAIL`, but the canonical gameplay window for the same range (`600..749` and `600..750`) passed.
- Read the ROM death/respawn path in `.cache/upstream/kid-chameleon-disasm/kid.asm`: `loc_B80C`, `lose_life`, `Teleport`, `loc_6E2`, `Load_InGame`. The ASM still says this range should already be in the normal in-level shell, so there was no basis for a gameplay patch.
- Proved the selected cache artefact was stale: a freshly derived case bundle for the same gameplay window verified `PASS`, while the cached bundle verified `FAIL`.
- Patched `crates/kidcli/src/case.rs` so gameplay-backed case bundles are canonicalised from source metadata during `case verify` and `case reduce` instead of trusting stale persisted scenario blobs.
- Re-ran `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`; the selected case now passes and its summary was refreshed.

## Selected Case

- Case ID: `death_respawn_level1_case_00600_00750`
- Selector: `death-respawn-level1-600-750`
- Autonomy gate at selection time: `evidence-only`
- Current verify result: `PASS`

## Why This Case

- The loop selected this exact case for cycle 37, so the session stayed on `death-respawn-level1-600-750` only.
- The first task was to decide whether the immediate frame-600 failure was a real gameplay drift or a stale case artefact.
- The canonical gameplay comparison and the fresh derive proved the failure was in the cached case payload, not in live gameplay.

## Strongest Evidence

- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 600 --frame-end 749 --checkpoint-interval 1` passes.
- `cargo run -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 600 --frame-end 750 --checkpoint-interval 1` passes.
- `cargo run -p kidcli -- case derive --gameplay-scenario death-respawn-level1 --frame-start 600 --frame-end 750 --out /tmp/kc-case-600-750` followed by `cargo run -p kidcli -- case verify --case /tmp/kc-case-600-750` passes.
- After the fix, `.cache/cases/death-respawn-level1-600-750/verify.json` now records `"passed": true` with matching fields `world.game_mode`, `camera`, `player.position`, `player.motion`, `entities`, and `events`.
- ASM still points at the standard respawn shell: `lose_life -> Teleport -> move.w #8,(Game_Mode).w -> jmp (j_loc_6E2).w`, then `Load_InGame` restores `PlayerStart_*`. Nothing in that path justified a new gameplay patch for `600..750`.

## Exact Next Moves

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue` again to select the next open case now that `death-respawn-level1-600-750` is no longer failing.
2. Re-verify the newly selected case before touching code.
3. If the next selected case is gameplay-backed, trust `case verify`/`case reduce` again; they now rebuild canonical gameplay windows instead of replaying stale persisted scenario payloads.

## Verification To Re-run

- `cargo test -p kidcli canonicalise_gameplay_case_manifest_repairs_stale_window_payload`
- `cargo test -p kidcli gameplay_case_verification_scenario_matches_canonical_window_builder`
- `cargo run -p kidcli -- case verify --case death-respawn-level1-600-750`
- `cargo run -p kidcli -- case summarise --case death-respawn-level1-600-750`

## Dead Ends / Do Not Retry

- Do not patch death/respawn gameplay for this case. The selected range now passes once the stale gameplay case manifest is canonicalised.
- Do not assume a cached gameplay case bundle is still trustworthy if it predates gameplay-window builder fixes; re-verify or freshly derive it first.
- Do not use `cargo test -- --exact` for these `kidcli` unit names in this crate layout; substring selection is the reliable path here.
