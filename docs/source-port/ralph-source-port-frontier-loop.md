# Ralph Source-Port Frontier Loop

Use this as the fixed prompt/context for repeated Ralph cycles on the live source-port frontier.

## Bootstrap (follow exactly — do not improvise)

**Step 1.** Read `.cache/autonomy/source-port-frontier/next-agent.md` — this is your primary context. It contains the current model, dead ends, and exact next moves. If this file is missing, run `cargo run -p kidcli -- autonomous-loop --goal source-port-frontier` to generate it, then read it.

**Step 2.** Read `.cache/autonomy/source-port-frontier/next-task.json` for commands and source paths.

**Step 3.** Pick ONE item from "Exact Next Moves" in the handoff note. That is your task for this session.

**STOP.** Do not read `docs/source-port/frontier-state.yaml`, `docs/source-port/backlog.md`, `docs/source-port/tas-roadmap.md`, or any other broad context document. The handoff note already distils them. Reading them wastes your context window on information you already have.

## Working Rules

- Your session has ONE task — the item you picked from "Exact Next Moves".
- Check "Dead Ends / Do Not Retry" before starting — do not repeat rejected approaches.
- Complete one coherent unit of work only: probe, instrumentation, or gameplay patch.
- Use the commands and source paths from `next-task.json` — do not rediscover them.
- You MUST commit before exiting, even if findings are inconclusive. No-commit sessions are failures.
- If the frontier changes, update `docs/source-port/frontier-state.yaml` in the same commit.

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

## Session Types

Choose the right session type based on frontier maturity. Do NOT conflate investigation with fixing.

### Probe session (investigation mode)

Use when the frontier has no proven causal path yet (hypothesis is still "something in gameplay does X").

**Prompt template:**

```
You have 1 task: [paste one item from next-agent.md "Exact Next Moves"].

Context: read .cache/autonomy/source-port-frontier/next-agent.md (do NOT re-read frontier-state.yaml, backlog.md, or tas-roadmap.md).

Rules:
- Add instrumentation, traces, or oracle probes that narrow the cause.
- Run the new instrumentation and record findings.
- Commit the result even if inconclusive — findings go in the commit message.
- Update next-agent.md with what you learned before exiting.
- Do NOT attempt a gameplay fix. Do NOT investigate alternative hypotheses beyond the one assigned.
```

Expected output: 1 commit (instrumentation + findings), updated next-agent.md.

### Fix session (execution mode)

Use only when a probe session has narrowed the cause to a specific code path and the next-agent.md "Exact Next Moves" says "implement fix for X".

**Prompt template:**

```
You have 1 task: implement the fix described in .cache/autonomy/source-port-frontier/next-agent.md "Exact Next Moves".

Context: read .cache/autonomy/source-port-frontier/next-agent.md.

Rules:
- Write the gameplay fix + a regression test around the divergent frame window.
- Run `mise run verify-slice-fast` before committing.
- Update frontier-state.yaml and next-agent.md in the same commit.
- If the fix doesn't work on first attempt, commit what you have and update next-agent.md with findings.
```

Expected output: 1-2 commits (fix + verification), updated frontier docs.

### Stuck detection

If a session produces no commit:
1. Stage all changes and commit as `WIP: <frontier-slug> probe N`
2. Write/update next-agent.md with what was attempted and why it stalled
3. The next session must use a different probe angle from "Exact Next Moves"

Never run the same investigation twice — check "Dead Ends / Do Not Retry" first.
