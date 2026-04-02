# Source-Port RL Prompt

Use this file as the checked-in operator prompt for source-port `rl` runs.

## Canonical launcher

Run:

```bash
mise run source-port-rl-run
```

That command starts a single `rl`/Codex run by piping the generated live prompt to `cxys`.

If you only need to inspect the live delegated package without starting Codex, run:

```bash
mise run source-port-rl-prompt
```

That command regenerates the live source-port package and prints the prompt for the current delegated lane.

## Rules

- Do not hand-write a fresh “find the next task” prompt from shell history.
- Do not embed `` `mise run source-port-rl-prompt` `` inside a double-quoted `cxys` prompt. The shell will expand it before `rl` starts and corrupt the command stream.
- Do not start from `backlog.md`, `frontier-state.yaml`, or `tas-roadmap.md`.
- Always regenerate the source-port package first, then follow the emitted delegated lane.
- If the package delegates to `case-queue` or `sweep-parity`, stay there until the live package changes.
- Treat `.cache/autonomy/source-port-frontier/next-task.json` and `.cache/autonomy/source-port-frontier/next-agent.md` as the primary context for the run.
