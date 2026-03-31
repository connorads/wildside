# Ralph Source-Port Frontier Loop

Use this as the fixed prompt/context for repeated Ralph cycles on the live source-port frontier.
If the frontier is already closed and the work should come from reduced semantic cases
instead, use `docs/source-port/ralph-source-port-case-queue-loop.md`.

## Bootstrap (follow exactly — do not improvise)

**Step 1.** Read `.cache/autonomy/source-port-frontier/next-agent.md` — this is your primary context. It contains the current model, dead ends, and exact next moves. If this file is missing, run `cargo run -p kidcli -- autonomous-loop --goal source-port-frontier` to generate it, then read it.

**Step 2.** Read `.cache/autonomy/source-port-frontier/next-task.json` for commands and source paths.

**Step 3.** Pick ONE item from "Exact Next Moves" in the handoff note. That is your task for this session.

**STOP.** Do not read `docs/source-port/frontier-state.yaml`, `docs/source-port/backlog.md`, `docs/source-port/tas-roadmap.md`, or any other broad context document. The handoff note already distils them. Reading them wastes your context window on information you already have.

## Investigation hierarchy (follow this order)

Before writing ANY code, understand what the ROM does first:

1. **Read the ASM** — the handoff note names ROM routines. Find them in `.cache/upstream/kid-chameleon-disasm/kid.asm` or `code/*.asm`. Read 50-100 lines around the named label. This is the source of truth.
2. **Run oracle diagnostics** — use `tas-diff --include-physics-trace` or `oracle-probe` to see what the ROM actually does at the frontier frame. Compare to what you read in the ASM.
3. **If the cause is clear, fix it** — if ASM + oracle tell you what's missing in native, implement the fix directly. Skip probing.
4. **Only probe native as a last resort** — if ASM and oracle are ambiguous, add ONE targeted probe to disambiguate. Do not add probes to rule out theories the ASM already answers.

**Do not write native-side probes to guess what the ROM does. Read the ROM.**

## Working Rules

- Your session has ONE task — the item you picked from "Exact Next Moves".
- Check "Dead Ends / Do Not Retry" before starting — do not repeat rejected approaches.
- Follow the investigation hierarchy above: ASM → oracle → fix → probe (last resort).
- Every frontier commit must cite the ASM label it implements. If you can't name the routine, you haven't done the investigation yet. No exceptions.
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

## What a good session looks like

1. Read next-agent.md (2 min)
2. Read the named ASM routine in kid.asm (5 min)
3. Run tas-diff to see oracle behaviour at the frontier frame (2 min)
4. Either implement the fix or write a handoff explaining what the ASM says (10 min)
5. Commit + update next-agent.md (1 min)

A session that only adds debug fields without reading ASM is a wasted session.

## Stuck detection

If a session produces no commit:
1. Stage all changes and commit as `WIP: <frontier-slug>`
2. Write/update next-agent.md with what was attempted and why it stalled
3. The next session must use a different approach

Never run the same investigation twice — check "Dead Ends / Do Not Retry" first.
