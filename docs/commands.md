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

## Parity fuzzing

`mise run fuzz` (`wildside fuzz run --levels 0x00-0x05 --seeds 8 --frames 600`)
replays random routes from each level start on native and the ROM. It first runs
an all-idle control per level; a level whose control diverges is recorded as
`start_state` and gets no random routes. Each route's first blocking divergence
is clustered by a level-agnostic signature: blocking fields, native/ROM
`character_movement`, native on-ground and the two inputs before it. The run
writes `.cache/fuzz-runs/<ts>/report.json`, `clusters.md` and one repro scenario
per cluster; `--write` also updates `docs/source-port/fuzz-summary.yaml` and
`tests/data/fuzz-repros/<cluster-id>.json`. Same arguments give identical reports.

`wildside fuzz shrink <cluster-id>` shrinks that cluster's repro in place
(`tests/data/fuzz-repros/`, else the latest run's `repros/`) while it keeps the
same signature: it truncates to the divergence, then zeroes input spans. It uses
the same reducer as `wildside case reduce`, one oracle replay per candidate
(~4 s each).

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
