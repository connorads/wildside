# Jev bug cases

`mise run jev-audit` joins `jev-cases.json` to the live backlog. `--json`
prints the joined records. Frozen lanes are excluded. Backlog status and
priority remain authoritative; the catalogue never promotes human validation.

Readiness describes the next experiment, not a passing result:

- `ready_replay`: inputs exist or can be reconstructed. Current execution and
  oracle comparison are still required.
- `needs_route`: no natural input stream covers the reported symptom.
- `needs_comparison`: a route or probe exists but its reference, alignment or
  coverage does not yet establish the claim.
- `inactive_path`: the historical defect is absent from, or unproven on, the
  current runtime path. A failing helper test cannot establish live coverage.
- `human_device`: real output-device evidence is required.

Evidence references identify repository files. Archive `#issue-N` references
mean search for `#N`; they are lookup keys, not generated Markdown anchors.
Cache paths in route descriptions are leads and may be stale or absent.
Each case names its exact scope and a next action. Broad issue summaries may
need several cases before the whole issue is covered.

The private catalogue is reversible and independent of recording formats.
A shared runner can consume the existing recording/oracle pipeline without
adding another player or changing the public CLI. A standalone browser bot
would duplicate controller and replay handling. Manually running each case
avoids new machinery but cannot consistently enforce evidence or cost limits.

The pilot covers one shared #210/#254 Start-spam case, the #327 same-level
teleporter, and #370 natural Dragon-breath death followed by ordinary reload.
Cross-level teleports and the continue branch require separate evidence.
An attempt counts only when the encounter occurs, the current output agrees
with the relevant oracle evidence, and a verified negative control fails the
same check. Synthetic controls are labelled; they do not reproduce a historical
build. Inconclusive comparisons and blocked routes never count as proven cases.

Run `mise run jev-case -- --issue 210` or `--issue 370` for offline controls.
`--script path.json` accepts a private array of ordinary controller bytes.
It cannot set position, form, health or lives. Standard recording bundles and
an HTML frame viewer are written beneath `.cache/jev-cases/<run>/`.
The recorder continues through death and title cards. Oracle errors and all
pixel differences remain visible. Non-proven outcomes exit one.
Ordinary-runtime tests use constructor defaults. Calling
`set_recorded_title_card_handoff_delay(false)` selects the separate direct
capture path; it does not restore ordinary runtime behaviour.

`mise run jev-case -- --issue 327 --live` explicitly permits paid discovery.
It first compares the settled start with the oracle. A red comparison prevents
paid requests. `TYPESAFE_API_KEY` stays in the environment; offline subprocesses
do not inherit it. Successful routes retain controller bytes and exact native
replay evidence. Interrupted calls keep their spend reservation.

The locked `.cache/jev-cases/budget.json` ledger starts at the prior cumulative
upper bound of $0.024954048. Preserve this file across runs. It enforces the
original $1 total, three attempts per case, 600 decisions per attempt and a
20-minute collection deadline shared by a case's attempts. Each decision
applies six frames, with at most 3,600 frames per route. Pricing and request
limits follow the pinned [Jev model](https://docs.typesafe.ai/models) and
[HTTP API](https://docs.typesafe.ai/api). No bulk runner is provided.

Negative controls run in detached temporary worktrees with separate Cargo
output. #370 deliberately removes the projectile clear at the ordinary
respawn boundary. This is synthetic, not a historical defect reconstruction.
The control must preserve the route prefix and fail the same encounter check.
The main checkout remains unchanged. A sensitive control does not override
a red oracle comparison.

Pilot evidence is recorded in [jev-pilot-results.md](jev-pilot-results.md).

Failed teleporter routes retain the full input journal and native trace.
Their visual report includes frame zero and the final 60 frames. A reached
encounter retains its whole event window, including its 30-frame prefix and
tail. The controller sees pause state and named held buttons. A is `0x40`,
B is `0x10`, and C is `0x20`; running actions use A. Decision journals include
the exact question and action rubrics as well as state and response.

Isolated controls copy an idle Cargo target directory. `CARGO_TARGET_DIR` is
honoured. Workspace build-script wrappers are rebuilt in the copied target
because build-wrap embeds their original paths. Sandboxing remains enabled.
Do not clone a target directory while another Cargo process is replacing its
object files.
