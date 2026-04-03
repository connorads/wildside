# Ralph Source-Port Case-Queue Loop

> **Deprecated**: the post-TAS delegation cascade now prefers `gameplay-backlog`
> (ASM-driven coverage) over case-queue when no actionable cases exist.
> Use `ralph-gameplay-loop.md` for the generative agent loop.

Use this loop when the live TAS frontier is closed and the next autonomous task should
come from reduced semantic gameplay cases.

## Bootstrap

1. Run `cargo run -q -p kidcli -- autonomous-loop --goal case-queue` to regenerate the live package for this run. Do this every cycle, even if the package files already exist.
2. Read `.cache/autonomy/case-queue/next-task.json`
3. Read `.cache/autonomy/case-queue/next-agent.md` if it exists
4. Work only the selected case from the freshly generated task package

## Working Rules

- Re-verify the selected case before changing code:
  `cargo run -p kidcli -- case verify --case <selector>`
- Sanity-check `generated_at`, `cycle`, and `progress_fingerprint` in the task package before doing anything else. If they are stale, regenerate instead of reusing the old package.
- If the task package marks the case `patch-allowed`, you may patch and locally verify.
- If the task package marks the case `evidence-only`, do not widen the work; tighten the case instead.
- Read the ASM before changing gameplay code. The case queue narrows the problem, but the ROM is still the source of truth.
- Keep the task focused on one case ID. Do not mix unrelated cases into one session.
- A successful cycle must do one of these:
  - land a gameplay/tooling code commit
  - land a new probe or instrumentation commit that narrows the causal model
  - reduce or retire the selected case with new authoritative evidence
- Updating `.cache/autonomy/case-queue/next-agent.md` alone does not count as progress and must not be committed on its own.
- After 2 consecutive docs-only or handoff-only commits, the next cycle must add instrumentation, narrow the case further, switch frontier, or stop as blocked.

## Exit Requirement

Before finishing, refresh `.cache/autonomy/case-queue/next-agent.md` with frontmatter so the next cycle can resume quickly:

```yaml
---
slug: <selected case id>
cycle: <autonomy cycle number or session-local label>
updated_at: <UTC timestamp>
---
```

Required sections:

1. `What Changed`
2. `Selected Case`
3. `Why This Case`
4. `Strongest Evidence`
5. `Exact Next Moves`
6. `Verification To Re-run`
7. `Dead Ends / Do Not Retry`

Keep the note cache-local or fold it into the next substantive commit. Do not create a commit whose only meaningful change is refreshing this note.
