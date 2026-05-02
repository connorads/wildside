# Ralph Source-Port Frontier Loop

> **Deprecated**: this loop is for TAS frontier work. If TAS work reopens, this loop applies.
> For post-TAS gameplay work, use `ralph-gameplay-loop.md` instead.

Use this as the fixed prompt/context for repeated Ralph cycles on the source-port operator entrypoint. When the TAS frontier is live, this loop packages that frontier. When the TAS frontier is closed, this same loop now auto-delegates to the next repo-local lane instead of handing off vague broader-work prose.

## Bootstrap (follow exactly — do not improvise)

**Step 1.** Run `cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier` to regenerate the live package for this run. Do this every cycle, even if the package files already exist.

**Step 2.** Read `.cache/autonomy/source-port-frontier/next-task.json` for the live delegated lane, commands, freshness metadata, and allowed source paths.

**Step 3.** Read `.cache/autonomy/source-port-frontier/next-agent.md` — this is your primary narrative context. It contains the current model, dead ends, and exact next moves.

**Step 4.** Pick ONE item from "Exact Next Moves" in the handoff note. That is your task for this session.

**Step 5 (self-check).** Read the "Recent Commits" block in the handoff note (or run `git log --oneline -5 --stat`). If the last 3+ commits touched only `docs/source-port/methodology.md` and/or `docs/source-port/research-map.md`, the loop is spinning — **do not produce another commit of the same shape**. Pivot to `autonomous-loop --goal proactive-verification`, or record the stall in `docs/source-port/frontier-state.yaml` before touching those prose docs again.

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
- Treat the freshly generated `next-task.json` as the source of truth. If it delegates to `case-queue` or `sweep-parity`, do not override it with an ad hoc backlog hunt.
- Check "Dead Ends / Do Not Retry" before starting — do not repeat rejected approaches.
- Follow the investigation hierarchy above: ASM → oracle → fix → probe (last resort).
- Every frontier commit must cite the ASM label it implements. If you can't name the routine, you haven't done the investigation yet. No exceptions.
- Use the commands and source paths from `next-task.json` — do not rediscover them.
- Sanity-check `generated_at`, `cycle`, `progress_fingerprint`, `delegated_goal`, and `delegated_target` in `next-task.json` before starting work. If they do not describe the current lane, regenerate the package instead of working stale instructions.
- If the note says the TAS frontier is closed, follow the delegated lane in the task package. Do not switch to a different loop unless the delegated task explicitly tells you to.
- A successful cycle must do one of these:
  - land a gameplay/tooling code commit
  - land a probe or instrumentation commit that narrows the causal model
  - move the earliest authoritative divergence forward with a verified regression/probe update
- Refreshing `next-agent.md` alone does not count as progress and must not be committed on its own.
- If the cycle cannot earn one of the outcomes above, stop as blocked instead of creating a handoff-only commit.
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

The note should be concise but complete enough that the next agent can resume from the active frontier without rediscovering the same model. Keep it cache-local or fold it into the next substantive commit; do not land a handoff-only commit.

## What a good session looks like

1. Read next-agent.md (2 min)
2. Read the named ASM routine in kid.asm (5 min)
3. Run tas-diff to see oracle behaviour at the frontier frame (2 min)
4. Either implement the fix or land the probe/instrumentation that disambiguates the next move (10 min)
5. Update next-agent.md alongside the substantive work (1 min)

A session that only adds debug fields without reading ASM is a wasted session.

## Stuck detection

If a session is blocked:
1. Write/update `next-agent.md` with what was attempted and why it stalled
2. Do not create a handoff-only `WIP` commit
3. The next session must use a different approach or a narrower case

Never run the same investigation twice — check "Dead Ends / Do Not Retry" first.

## Auto-recovery and state-aware operator hints

When the autonomy cascade can't dispatch new work because a regenerable
artefact is missing (state sweep or release-gate evidence), the runner
auto-recovers instead of waiting on the operator. Bookkeeping at
`.cache/autonomy/source-port-frontier/auto_recovery_attempts.json` caps
recovery at **one attempt per fingerprint, ever** — a genuine failure still
surfaces on the second stall, and a manual reset (delete the entry or the
whole file) re-arms it. Dirty trees skip the auto-run.

Operator unblock hints emitted by `emit_circuit_breaker_message` are now
state-aware: they only suggest commands whose preconditions hold (e.g.
`asm-coverage --update` is suppressed when coverage is already 100%, and
the regeneration command for a missing artefact is rendered as the top
hint). The `missing_artefact` typed signal on `TaskPackage` is what drives
both auto-recovery and the rendered hint.
