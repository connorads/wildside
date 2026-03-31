# Ralph Source-Port Case-Queue Loop

Use this loop when the live TAS frontier is closed and the next autonomous task should
come from reduced semantic gameplay cases.

## Bootstrap

1. Run `cargo run -p kidcli -- autonomous-loop --goal case-queue`
2. Read `.cache/autonomy/case-queue/next-task.json`
3. Read `.cache/autonomy/case-queue/next-agent.md` if it exists
4. Work only the selected case from the task package

## Working Rules

- Re-verify the selected case before changing code:
  `cargo run -p kidcli -- case verify --case <selector>`
- If the task package marks the case `patch-allowed`, you may patch and locally verify.
- If the task package marks the case `evidence-only`, do not widen the work; tighten the case instead.
- Read the ASM before changing gameplay code. The case queue narrows the problem, but the ROM is still the source of truth.
- Keep the task focused on one case ID. Do not mix unrelated cases into one session.

## Exit Requirement

Before finishing, refresh `.cache/autonomy/case-queue/next-agent.md` with frontmatter:

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
