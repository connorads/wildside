# Ralph Source-Port Frontier Loop

Use this as the fixed prompt/context for repeated Ralph cycles on the live source-port frontier.

## Bootstrap

Run these commands first:

```bash
mise run frontier-status
cargo run -p kidcli -- autonomous-loop --goal source-port-frontier
```

Then read:

- `.cache/autonomy/source-port-frontier/next-task.json`
- `.cache/autonomy/source-port-frontier/next-agent.md` only if `frontier-status` reports it as `current`

## Working Rules

- Treat `docs/source-port/frontier-state.yaml` as the canonical truth for the live frontier.
- Treat `.cache/autonomy/source-port-frontier/next-task.json` as the machine package for the current loop cycle.
- Complete one coherent unit of work only: probe, instrumentation, or gameplay patch.
- Prefer the commands and source paths named in `next-task.json` over rediscovering from broader docs.
- If the frontier changes, update the tracked frontier docs in the same unit of work.

## Exit Requirement

Before finishing the cycle, replace `.cache/autonomy/source-port-frontier/next-agent.md` with a fresh handoff note for the next agent.

Required frontmatter:

```yaml
---
slug: <active frontier slug>
cycle: <autonomy cycle number or session-local cycle label>
updated_at: <UTC timestamp>
---
```

Required sections:

1. `What Changed`
2. `Current Model`
3. `Strongest Evidence`
4. `Exact Next Moves`
5. `Verification To Re-run`
6. `Dead Ends / Do Not Retry`

The note should be concise but complete enough that the next agent can resume from the active frontier without rediscovering the same model.
