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
- If tracked git changes already exist, treat that diff as the first recovery task: verify it, commit it if it passes, or deliberately discard only that unit before switching lanes.
- Dirty tracked trees now surface a compile preflight and dirty-tree fingerprint in the generated prompt; treat repeated red fingerprints as recovery, not as licence to retry the lane unchanged.
- If the proactive package switches to `loc-coverage-promotion`, promote test-cited `loc_` labels to implementation coverage instead of treating them as fresh uncovered work.
- If the live package reports `recovery_mode`, complete that recovery step before resuming routine delegated-lane work.
- Repeated stalled fingerprints are a cue to change tact, not to replay the same prompt again.
- Treat `.cache/autonomy/source-port-frontier/next-task.json` and `.cache/autonomy/source-port-frontier/next-agent.md` as the primary context for the run.
