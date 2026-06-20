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

## Dev / oracle setup

The native runtime, asset extraction, and the emulator oracle are wired through mise:
`mise run bootstrap-native` (assets + level grids), `mise run bootstrap-upstream` /
`mise run bootstrap-references` (disassembly + research repos), and
`mise run oracle-setup` (emulator). These matter only for development and the oracle.
