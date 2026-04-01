---
slug: kid_chameleon_short_demo_boot_slice_case_00000_00005
cycle: 21
updated_at: 2026-04-01T00:08:03Z
---

## What Changed

- Ran `cargo run -p kidcli -- autonomous-loop --goal case-queue`; cycle 21 selected `kid_chameleon_short_demo_boot_slice_case_00000_00005`.
- Re-ran `cargo run -p kidcli -- case verify --case kid_chameleon_short_demo_boot_slice_case_00000_00005` first; the stored case failed immediately at frame 0 on `player.x_pos`.
- Read the ROM boot path in `.cache/upstream/kid-chameleon-disasm/kid.asm` before patching. `Load_SegaScreen` only drives the Sega logo object and leaves Kid state in cleared WRAM.
- Traced the failure to scenario-path semantic export, not gameplay simulation: native preserved `kid_x_pos = None`, `kid_y_pos = None`, and `character_movement = None` on the hidden Sega-screen boot window while oracle semantics published the cleared WRAM zeroes.
- Patched `crates/kidcli/src/case.rs` to zero-fill those Sega-screen semantic player fields for scenario-path verification and added a unit regression.

## Selected Case

- Case ID: `kid_chameleon_short_demo_boot_slice_case_00000_00005`
- Selector: `kid_chameleon_short_demo_boot_slice_case_00000_00005`
- Autonomy gate: `patch-allowed`
- Current verify result: `PASS`

## Why This Case

- The cycle-21 autonomy task package selected this exact case, so it was the only allowed scope.
- The first failing field was present on frame 0, which made it a good fit for a focused semantic/boot-shell fix instead of wider gameplay work.
- ASM plus oracle agreed the Sega-screen shell should expose cleared Kid WRAM, so the mismatch lived in native case export.

## Strongest Evidence

- `cargo test -p kidcli 'case::tests::semantic_from_native_zero_fills_hidden_sega_screen_player_fields' -- --exact` passes.
- `cargo run -p kidcli -- case verify --case kid_chameleon_short_demo_boot_slice_case_00000_00005` now returns `PASS`.
- `.cache/cases/kid_chameleon_short_demo_boot_slice_case_00000_00005/verify.json` now reports `"passed": true` and `"first_failure": null`.
- `.cache/cases/kid_chameleon_short_demo_boot_slice_case_00000_00005/summary.md` now reports `passed: true`.

## Exact Next Moves

1. Re-run `cargo run -p kidcli -- autonomous-loop --goal case-queue`.
2. Take the newly selected case only.
3. Re-verify it before patching, and if gameplay behaviour is involved, read the relevant ASM first.

## Verification To Re-run

- `cargo test -p kidcli 'case::tests::semantic_from_native_zero_fills_hidden_sega_screen_player_fields' -- --exact`
- `cargo run -p kidcli -- case verify --case kid_chameleon_short_demo_boot_slice_case_00000_00005`
- `cargo run -p kidcli -- case summarise --case kid_chameleon_short_demo_boot_slice_case_00000_00005`

## Dead Ends / Do Not Retry

- Do not widen this case into demo bootstrap gameplay physics; the selected 0..5 frame window never leaves the Sega-screen shell.
- Do not treat `player.x_pos = 0` on this case as evidence that the native runtime must instantiate Kid during `Load_SegaScreen`; the fix belongs in semantic normalisation for scenario-path verification.
- Do not debug `player.motion` separately if this case regresses again before checking the same semantic mapper; the hidden Sega-screen window needs both zeroed coordinates and `movement_0000`.
