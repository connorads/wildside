# Source-port loop

The whole loop. The backlog is sharded by reader:

- `backlog.md`: the NOW handoff and the **open queue** (`[ ]`/`[~]` rows). Your file.
- `backlog-awaiting-human.md`: `[?]`/`[b]` rows. **Skip these**; only a human closes them.
  Rows use `Target` and `Check` so `project-status` can print the validation surface.
- `backlog-archive.md`: resolved rows and per-issue evidence, keyed by `#NNN`.
- `session-log.md`: append-only history of past session handoffs.

1. **Orient and select.** Read the NOW block in `backlog.md`, then run
   `mise run project-status` (or `cargo run -p wildside-cli -- project-status`). It reads
   every live lane and prints, priority-ordered (P0 before P1 before P2):
   - **ready to implement now**: open rows with a root cause identified; take the top one;
   - **top open playability / visual risks**: otherwise take the top of these;
   - **human validation/repro blockers**: the awaiting-human lane; never pick these.
     Each prints `Target` plus the short human `Check`.

   Do not eyeball the table for "the top row": order in the file is not priority,
   `project-status` is. If `ready` and the risk lists are empty, use the
   proactive-verification fallback below. The NOW "Current work" bullet is the next
   action, so it must not steer below `project-status`'s top open priority (e.g. name a
   P2 while a P1 is open); `backlog-lint` enforces this. Other NOW bullets are
   state/context and may mention any priority.
2. **Check history before forming a hypothesis:** `grep -n '#NNN'
   docs/source-port/backlog-archive.md` and `git log --oneline --grep '#NNN'`.
   Dead ends and rejected hypotheses live there; do not retry them.
3. Mark the row `[~]` in `backlog.md`.
4. Read the owning ASM routine in `.cache/upstream/kid-chameleon-disasm/` before
   touching native code; cite the label in code and tests.
5. Write the narrowest failing regression that proves the bug, then fix it.
6. Verify: `mise run verify-slice-fast` (always) + `mise run verify-tas-full` if the
   change is gameplay-affecting. The gate must be green at every commit.
7. Commit with the *why*, dead ends included. **Close out in the same commit:** set the
   row's status, and if it leaves the open lane, MOVE the whole row to its lane:
   `[?]`/`[b]` → `backlog-awaiting-human.md`, `[x]`/`[-]` → the resolved index in
   `backlog-archive.md`. Awaiting-human rows must set `Target` (`0xNN`, `boot`,
   `demo`, `demo:0xNN`, `audio-device`, `recording:<slug>`, or `unknown`) and a short
   overlay-safe `Check` sentence with no links, commands, regression dump, or vague
   "reported route" wording. The `Check` must name the concrete symptom and validation
   action visible to a playtester. Append closing evidence to the archive under `#NNN`.
   Keep the open-queue Verify cells short (evidence belongs in the archive).
8. Update the handoff: prepend a one-paragraph session note to `session-log.md`
   (newest first) and rewrite NOW in `backlog.md` to the current state. NOW is a
   handoff, not a changelog; never let it accrete.

**No code commit ⇒ write the dead end under the issue's `#NNN` in
`backlog-archive.md` and stop; never commit churn.**

`mise run backlog-lint` enforces the shape (NOW size, lean open-queue Verify cells,
awaiting-human `Target`/`Check`, concrete non-vague human checks, each `#NNN` in exactly
one lane, rows in the lane their status implies); it runs inside `verify-slice-fast`, so
drift fails the gate.

## Fallback: proactive verification

`project-status` shows no `ready` and no open risks (every open row is gone or the
queue is empty)? Discover bugs before the next playtest does:

- **Mutation-test one file**: `FILE=<module.rs> mise run mutants-file`; every
  surviving mutant is a test blind spot. Pair loose proptests with exact-value
  unit tests; left-facing paths are systematically undertested. Keep runs to one
  module (big files like `enemies.rs` enumerate thousands of mutants).
- **Audit one enemy family vs ASM**: pick a family in `code/enemy/*.asm`, compare
  each branch against the native implementation, write probe scenarios for any
  behaviour the sweep doesn't already exercise.
- **Extend ASM coverage**: `cargo run -p wildside-cli -- asm-coverage --uncovered`, pick
  an ordinary uncovered `sub_` label, trace and cite it. If only
  `[emulated GEMS out-of-scope]` labels remain, do not cite them just to move the
  metric; use another fallback. Coverage is a discovery aid, not the lead
  selector; backlog truth always beats raw coverage.
