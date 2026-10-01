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
   and `mise run jev-audit`. The status command reads every live lane and the
   checked-in parity measurements, and prints:
   - **level ledger**: N/103 levels exited on native with the route confirmed on
     the ROM, and the divergence blocking each unconfirmed native exit;
   - **fuzz**: the random-route census, idle-divergent levels, and every cluster
     no open row (or its archive section) names;
   - **ready to implement now**: open rows with a root cause identified; take the top one;
   - **top open playability / visual risks**: otherwise take the top of these;
   - **human validation/repro blockers**: select objective cases under the campaign
     evidence rules; leave subjective/device checks for the final playtest.
     Each prints `Target` plus the short human `Check`. Move a selected objective
     row to the open lane before starting work, preserving its history.

   **Intake is the measurements.** An untracked fuzz cluster or ledger blocker
   becomes a backlog row before other selection: shrink its repro
   (`wildside fuzz shrink <cluster>`), open a row that names the cluster id, and
   record the repro path under its `#NNN` in the archive. Then select by
   `project-status` order. Do not eyeball the table for "the top row": order in
   the file is not priority, `project-status` is. When nothing is ready, open or
   untracked, re-run the measurements (see **Parity measurements**); the loop
   stops when they report no open cluster and no ledger blocker. The NOW
   "Current work" bullet names the next
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
5. Reproduce with the shrunk repro or ordinary recorded inputs. Write the narrowest
   failing regression, then fix it. Replay the same inputs and assertion after the
   change. A fix's evidence is the repro, an oracle match and a
   `gameplay_regression_*` test that loads the repro; only that prefix runs in the gate.
6. Compare the saved route with the emulator and inspect the relevant captures.
   Verify: `mise run verify-slice-fast` (always) + `mise run verify-tas-full` if the
   change is gameplay-affecting. The gate must be green at every commit. After a
   fix, re-run the parity measurements with `--write`: the fixed cluster must
   disappear and no earlier cluster may appear.
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

## Parity measurements

Two commands turn the oracle into the loop's intake. Both write checked-in yaml
that `project-status` reads; neither runs in a gate.

- `mise run fuzz` (`wildside fuzz run --levels 0x00-0x05 --seeds 8 --frames 600`):
  random routes from each idle-clean level start, replayed on native and the ROM.
  Each route's first blocking divergence is clustered by signature. `--write`
  updates `fuzz-summary.yaml` and `tests/data/fuzz-repros/<cluster>.json`.
  Keep the census arguments fixed between runs so cluster counts compare.
- `mise run ledger` (`wildside ledger run --write`): a deterministic Go-Explore
  search per level for an exit (flag, teleport or game end), then the route's ROM
  replay. A level is `confirmed` when no blocking divergence precedes the exit and
  both sides leave on the same frame; otherwise `blocked_by` names the cluster.
  `--write` updates `level-ledger.yaml` and `tests/data/ledger-routes/`.
  `mise run verify-ledger` replays every confirmed route; run it periodically.

The headline is N/103 levels confirmed. A fix that removes a cluster usually
unblocks ledger levels too, so re-run both after each fix. The fuzz census
takes under a minute. A full ledger pass takes about an hour, so after a fix
re-run only the levels its cluster blocked (`--levels 0x00,0x04`); `--write`
merges them into the ledger.

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
the clock teleports), the commit must move it. If the session can only add index N+1,
say so in the handoff and take a different item; a march that pays down nothing is
indistinguishable from a stalled one, and only the ratchet can tell them apart.

`mise run backlog-lint` enforces the shape (NOW size, lean open-queue Verify cells,
awaiting-human `Target`/`Check`, concrete non-vague human checks, each `#NNN` in exactly
one lane, rows in the lane their status implies, and a `Verify` that is not just the
`Summary`'s index plus one); it runs inside `verify-slice-fast`, so drift fails the gate.
