# Source-port loop

The whole loop. The backlog table in `backlog.md` is the queue (order in table =
human playtest priority); the NOW block at its top is the session handoff.

1. Read the NOW block, then take the **top open item** (`[ ]`/`[~]`) in the table.
   Skip `[?]`/`[b]` rows — they wait on a human.
2. Check history before forming a hypothesis: `grep -n '#NNN'
   docs/source-port/backlog-archive.md` and `git log --oneline --grep '#NNN'`.
   Dead ends and rejected hypotheses live there — do not retry them.
3. Read the owning ASM routine in `.cache/upstream/kid-chameleon-disasm/` before
   touching native code; cite the label in code and tests.
4. Write the narrowest failing regression that proves the bug, then fix it.
5. Verify: `mise run verify-slice-fast` (always) + `mise run verify-tas-full` if the
   change is gameplay-affecting. The gate must be green at every commit.
6. Commit with the *why*, dead ends included. Flip the row's status, append closing
   evidence to `backlog-archive.md` under `#NNN`, update the NOW block, take the
   next item.

**No code commit ⇒ write the dead end under the issue's `#NNN` in
`backlog-archive.md` and stop — never commit churn.**

## Fallback: proactive verification

Backlog empty (or every open row `[?]`/`[b]`)? Discover bugs before the next
playtest does:

- **Mutation-test one file**: `mise run mutants-file FILE=<module.rs>` — every
  surviving mutant is a test blind spot. Pair loose proptests with exact-value
  unit tests; left-facing paths are systematically undertested. Keep runs to one
  module (big files like `enemies.rs` enumerate thousands of mutants).
- **Audit one enemy family vs ASM**: pick a family in `code/enemy/*.asm`, compare
  each branch against the native implementation, write probe scenarios for any
  behaviour the sweep doesn't already exercise.
- **Extend ASM coverage**: `cargo run -p wildside-cli -- asm-coverage --uncovered`, pick
  an uncovered `sub_` label, trace and cite it. Coverage is a discovery aid, not
  the lead selector — backlog truth always beats raw coverage.
