# Source-port loop

The whole loop. The backlog is sharded by reader:

- `backlog.md`: the NOW handoff, the **open queue** (`[ ]`/`[~]` rows), and frozen
  `[f]` lanes (standing decisions kept visible; `project-status` lists them under
  "frozen lanes" and never offers them). Your file.
- `backlog-awaiting-human.md`: `[?]`/`[b]` rows. Objective cases are eligible for
  investigation and evidence-backed closure under [the completion campaign](completion-campaign.md).
  Subjective/device checks stay for the final human playtest. Rows use `Target`
  and `Check` so `project-status` can print the validation surface.
- `backlog-archive.md`: resolved rows and per-issue evidence, keyed by `#NNN`.
- `session-log.md`: append-only history of past session handoffs.

1. **Orient and select.** Read the NOW block in `backlog.md`, then run
   `mise run project-status` (or `cargo run -p wildside-cli -- project-status`)
   and `mise run jev-audit`. The status command reads
   every live lane and prints, priority-ordered (P0 before P1 before P2):
   - **ready to implement now**: open rows with a root cause identified; take the top one;
   - **top open playability / visual risks**: otherwise take the top of these;
   - **human validation/repro blockers**: select objective cases under the campaign
     evidence rules; leave subjective/device checks for the final playtest.
     Each prints `Target` plus the short human `Check`. Move a selected objective
     row to the open lane before starting work, preserving its history.

   Do not eyeball the table for "the top row": order in the file is not priority,
   `project-status` is. If `ready` and the risk lists are empty, use the
   campaign coverage prerequisites and objective cases before the
   proactive-verification fallback below. The NOW "Current work" bullet names the next
   implementation item, so it must not steer below `project-status`'s top open
   priority (e.g. name a P2 while a P1 is open); `backlog-lint` enforces this. Other NOW bullets are
   state/context and may mention any priority.
2. **Check history before forming a hypothesis:** `grep -n '#NNN'
   docs/source-port/backlog-archive.md` and `git log --oneline --grep '#NNN'`.
   Dead ends and rejected hypotheses live there; do not retry them.
3. Mark the row `[~]` in `backlog.md`. Never flip an `[f]` row: freezes are standing
   decisions with their own ADRs; only a human unfreezes.
4. Choose the reproduction under **Jev discovery and saved-route validation** below.
   State the symptom, active runtime path and required encounter witness.
   Read the owning ASM routine in `.cache/upstream/kid-chameleon-disasm/` before
   touching native code; cite the label in code and tests.
5. Reproduce with ordinary recorded inputs. Write the narrowest failing regression,
   then fix it. Replay the same inputs and assertion after the change.
6. Compare the saved route with the emulator and inspect the relevant captures.
   Verify: `mise run verify-slice-fast` (always) + `mise run verify-tas-full` if the
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
   Update the case catalogue's route/readiness with the evidence, or remove its
   entry when the issue leaves the live lanes. `jev-audit` checks the join.
8. Update the handoff: prepend a one-paragraph session note to `session-log.md`
   (newest first) and rewrite NOW in `backlog.md` to the current state. NOW is a
   handoff, not a changelog; never let it accrete.

## Jev discovery and saved-route validation

Jev collects ordinary controller inputs. Replays, ASM and emulator comparisons
establish what those inputs demonstrate. Commands and specification fields live
in [the case runner guide](jev-cases.md); closure and spending authority live in
[the completion campaign](completion-campaign.md).

- Reuse a saved route first and compare it on the current build. A cheap scripted
  reproducer comes before paid navigation. Run a registered `check` when one
  exists; a green `replay` alone does not establish a defect fix.
- For `needs_route`, use bounded `discover` when navigation is the missing
  evidence, the target is on the active runtime path and preflight passes.
  Continue from a verified prefix when available. State the mechanical witness
  required for the claim; a navigation target or frame-end count may be only a
  candidate. Settled-warps establish diagnostic coverage, not natural entry.
- For `needs_comparison`, resolve alignment or fidelity offline. For
  `inactive_path`, establish the relevant live path before navigation. Keep
  subjective and physical-device cases for human validation.
- Keep useful failed routes. Investigate their earliest authoritative divergence
  as a separate defect when appropriate. A miss on the requested encounter is
  still a miss, even when that route exposes another bug.
- Validate a fix with the same encounter and assertion on the broken and fixed
  versions, or a labelled sensitive negative control. A control that passes the
  same assertion supplies no discrimination. Preserve the full relevant event
  window; passing selected images does not establish unchecked frames.
- Promote inputs needed for a regression or handoff into checked-in scenarios.
  Archive the tested revision, ROM/input identity, encounter witness, comparison
  window, commands and result. Cache paths are disposable leads.

After three gameplay fixes since the last exploration review, finish the current
fix and its verification, then review exploration before selecting the next
implementation item. Select one bounded exploration of the highest-priority
eligible route gap. Keep NOW's Current work on the top implementation priority;
record exploration in its separate review bullet. This complements the
existing mutation-testing and native-playtest cadence. Record the last review
revision and outcome in NOW; count gameplay fixes across session boundaries.
With no recorded review, review once. If preflight, credentials, case limits or
budget block collection, record that reason and continue useful offline work.
A blocked review establishes no gameplay coverage. Different goals and prefixes
do not renew the owning issue's allowance; do not relabel exhausted cases.

A useful route belongs in a commit with its permanent regression or executable
coverage check. These executable tests count as code. A saved route plus a report
alone does not satisfy this rule.

**No code commit ⇒ write the dead end under the issue's `#NNN` in
`backlog-archive.md`, park that investigation and continue another useful campaign
item; never commit churn.**

**An index-advancing commit is not progress on its own.** A commit whose whole content
is "do the same thing at index N+1" — type the next TAS frame, model the next level, port
the next enemy — must also leave the issue measurably closer to closing: a retired
override, a retired oracle injection, a newly attributed constant, or a gate that now
covers what it did not. Where a ratchet meters the lane (`oracle-injection-ratchet` for
#406's clock teleports), the commit must move it. If the session can only add index N+1,
say so in the handoff and take a different item; a march that pays down nothing is
indistinguishable from a stalled one, and only the ratchet can tell them apart.

`mise run backlog-lint` enforces the shape (NOW size, lean open-queue Verify cells,
awaiting-human `Target`/`Check`, concrete non-vague human checks, each `#NNN` in exactly
one lane, rows in the lane their status implies, and a `Verify` that is not just the
`Summary`'s index plus one); it runs inside `verify-slice-fast`, so drift fails the gate.

## Fallback: proactive verification

`project-status` shows no `ready` and no open risks (every open row is gone or the
queue is empty)? Frozen `[f]` rows are never selectable, so they do not block the
fallback. Discover bugs before the next playtest does — preferring targets whose
hardening could plausibly move a live gate or ratchet:

- **Prefer live gameplay/sweep-path modules over frozen-lane feeders.** Modules
  that exist to feed a frozen lane (`saxman_timing.rs`, `enigma_timing.rs`,
  `intro_timing.rs`, `vdp.rs` raster scheduling, `mode_level_timing.rs`) harden a
  surface nothing ships through; enemy families, physics, collision, and sweep-path
  rendering harden what the sweep and playtests actually exercise. The handoff must
  name which gate or ratchet the session's hardening could plausibly move — "more
  tests in a frozen feeder" names none. Preference, not mechanism: an allowlist
  lint would fight legitimate work when a frozen lane thaws.
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
