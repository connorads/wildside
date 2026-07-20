# Command reference

The [README](../README.md) covers the everyday subset. There is no hand-maintained
catalogue of every command — it drifts. Instead, ask the binary and the task runner:

- `wildside --help` — list every subcommand.
- `wildside <cmd> --help` — usage for any one subcommand.
- `mise tasks` — list every `mise run` task.

## Canonical gates

- `mise run verify-slice-fast` — fast regression gate; run before every commit.
- `mise run verify-tas <scenario.json>` and `mise run verify-tas-full` — TAS/oracle
  replay; run after gameplay-affecting commits.

## Divergence triage

`wildside visual-triage --scenario .cache/tas-verify/<slug>/scenario.json` clusters a
pixel-mismatch corpus (from `verify-tas --compare-pixels --pixel-mismatch-report`) by
root cause — divergence class + renderer layer + draw-command kind + palette line —
writing a ranked report and per-cluster diff composites under
`.cache/visual-triage-runs/`. See `wildside visual-triage --help`.

## Dev / oracle setup

The native runtime, asset extraction, and the emulator oracle are wired through mise:
`mise run bootstrap-native` (assets + level grids), `mise run bootstrap-upstream` /
`mise run bootstrap-references` (disassembly + research repos), and
`mise run oracle-setup` (emulator). These matter only for development and the oracle.

## Web build

`mise run web-worker-dev` builds the wasm module and host-neutral picker-mode
package, then serves `web/dist` through Cloudflare Workers Static Assets. No ROM
or derived bundle is copied by default. `mise run verify-web-fast` checks the
Rust/wasm/package boundaries; `mise run verify-web-local` runs the local Worker
and Playwright journeys when a canonical ROM and Playwright Chromium are
available.

`scripts/package_web.py --config <path> --output <dir> [--copy-source <path>]`
packages another tracked config. `--copy-source` is always explicit and validates
the source as the canonical ROM or a `WSBNDL01` bundle before copying it to the
config's relative URL.
