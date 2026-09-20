# Single-player completion campaign

## Outcome and scope

Deliver a reproducible single-player release candidate across every ROM-confirmed
playable level, branch, form and boss, with one final human playtest package.
The user authorises evidence-backed automated closure of objective bugs.
Subjective and real output-device checks remain human checks and are collected
in that final package. Preparing the package does not claim human validation.

Frozen timing debt #406, #409 and #416 remains disclosed and ratcheted. Two-player
completion is outside this campaign. No new divergence inherits a frozen exemption
merely because it affects the same subsystem. No push, merge or deployment is
authorised. Verified coherent changes are committed on the current branch.

## Execution and selection

Use the native Codex Goal with this document as its durable specification, the
backlog lanes as the issue tracker, and NOW as the resumption point. This is a
reversible process choice: another executor can resume the same evidence and queue.
A manual session loop needs repeated user prompts. A custom agent daemon adds
deployment, recovery and permission machinery without improving game fidelity.
The native Goal retains the existing tools and verification gates without those
obligations. It still requires an available execution environment.

1. Read NOW, run `mise run project-status` and `mise run jev-audit`.
2. Complete coverage prerequisites, then select the highest-severity actionable
   objective case. The awaiting-human lane is eligible for investigation; move a
   selected objective row to `[~]` in the open queue, preserving its old evidence
   in the archive. A readiness label is not a passing result.
3. Reproduce using ordinary recorded inputs; trace ASM and emulator state; inspect
   images; implement the smallest engine fix with a permanent regression.
4. Run the focused checks and `mise run verify-slice-fast`. Run
   `mise run verify-tas-full` after gameplay changes, sequentially with the fast
   gate because they share cache state. Engine fixes also need the corrected full
   sweep. Record existing failures separately from regressions; never weaken a
   gate to call a fix green.
5. Commit the verified unit with evidence, lane changes and an updated NOW.
   Continue to the next item without asking the user to choose it.

After two consecutive investigations without a useful implementation or new
discriminating result, record the rejected hypotheses, the missing evidence and
the next experiment. Park that case and continue another useful item. A parked
case resumes only with new evidence or an unblocked dependency. Do not commit
repeated probe/document churn. After three to five fixes, run focused mutation
testing and a native playtest to discover gaps beyond the existing cases.

Stop only at the finish criteria, user intervention, or a blocker that prevents
all remaining useful work. A milestone, empty ready queue, failed navigation or
exhausted Jev allowance is not completion. Keep the handoff usable after an
execution interruption.

## Milestones and commit boundaries

| Milestone | Atomic units | Verification |
|---|---|---|
| Truthful coverage | Sweep enumeration fix; separate asset-alias fixes if proven; ROM level and progression inventory | Failing enumeration regression becomes green; all non-sentinel MapOrder entries have a disposition; corrected sweep records failures and unverified entries |
| Complete game loop | One reproduced title/start, death/respawn, continue, game-over or ending defect per unit | Natural input replay, ASM-cited regression, emulator state and relevant pixels, full TAS |
| Every playable level | Reusable natural routes and each engine defect they expose | Entry, traversal and exit for every confirmed playable level; flag, teleporter, bonus-return and boss progression edges covered |
| Mechanics and presentation | Each form, enemy, boss, platform, terrain, collectible or audio defect with its own regression | Source-derived behaviour, comparable emulator evidence and inspected captures; device checks retained |
| Backlog closure | One objective report or inseparable symptom cluster per unit | Evidence contract below, including newly discovered defects; no bulk closure from narrow gates |
| Release validation | Reproducible build, final coverage report and playtest package | Fresh journeys, fast gate, playable/accurate release gates, full TAS, corrected sweep and disclosed residual debt |

The inventory distinguishes logical levels, shared resource headers, leftovers
and sentinels. ASM `Load_InGame` indexes `MapOrder_Index` directly by byte; hex
digits A-F are ordinary indices. The standard `0-104` range must not be reduced
to 69 by a BCD filter. Logical `$41` is named playable Crypt in `constants.asm`;
its shared-header assets resolve through the complete ROM header. Remaining
Crypt comparison differences stay visible; the level has no blanket exemption.

Idle warp checkpoints are diagnostic coverage, not proof of natural entry or
completion. A route can reuse a verified input prefix from boot or a legitimate
earlier entry; injected position, health, form or lives cannot prove progression.
Attract-demo level changes do not prove player exits. Record game mode and the
actual exit condition; a recording's title or distinct-level count is insufficient.
The existing whole-game index supplies discovery targets, not verified counts.

## Objective closure evidence

An agent may mark an objective issue `[x]` only when the archive records:

- The precise claim and a deterministic route that actually encounters it.
- The baseline failure, or a labelled negative control which fails the same
  assertion. A synthetic control does not claim to reproduce a historical build.
- The owning ASM labels and comparable emulator evidence for the behaviour.
- A permanent regression, the verified fix revision and the commands/results.
- Inspected before/after captures for visual claims, with all relevant pixel
  differences accounted for and unrelated differences still visible.

A broad issue needs evidence for every claimed branch. A passing unit test,
absent reproduction, fully masked comparison or inaccessible route cannot close
it. `[b]` can return to investigation when the missing evidence is obtained.
`[-]` needs evidence that the report is an original quirk, duplicate or outside
the agreed scope; a difficult bug is not an exclusion. Frozen rows stay frozen.
Catalogue readiness and generated reports never mutate backlog status themselves.

## Jev spending and evidence

The allowance is **$5 additional** for this entire campaign, not per issue or
session. Preserve the historical locked ledger and its charged reservations.
Before any campaign paid request, persist an idempotent campaign allocation and
attempt epoch; repeated startup must never replenish it. Account for failures
and interrupted requests conservatively. Until that allocation is implemented
and tested, retain the existing stricter cap and make no new paid requests.

Jev discovers routes and attempts reproductions. Deterministic replay, ASM,
emulator comparisons and visual inspection establish correctness. Reuse verified
routes before buying exploration. Stop stalled attempts early. After the paid
cap, continue offline verification and game fixes. The cap is for Jev API usage,
not Codex usage. Never put API keys in tracked files or reports.

## Finish criteria

- All confirmed playable levels and progression branches have natural route
  evidence; every form and boss and each enemy/mechanic family has live coverage.
- Boot-to-credits and death, continue, game-over, pause and restart flows pass.
- Every in-scope objective report has a supported resolution. No unresolved
  objective defect is disguised as a human check or a frozen timing exception.
- Required gates pass, coverage has no unexplained omissions, and all retained
  timing debt is within its documented ratchets.
- A reproducible build and evidence report identify the tested revision and
  exact commands. One final playtest package contains only genuine subjective
  and device checks, with concrete routes and expected outcomes.

The final package is a release candidate for human review. It does not claim
`human-play-stable` until the user performs that review.
