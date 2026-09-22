# Jev bug cases

`mise run jev-audit` joins `jev-cases.json` to the live backlog. `--json`
prints the joined records. Frozen lanes are excluded. Backlog status and
priority remain authoritative; the catalogue never changes status. Objective
closure follows [the completion campaign](completion-campaign.md); subjective and
device checks remain for the final human playtest.

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

## Commands

- `mise run jev-case -- check --issue 210` and `check --issue 370` run the
  registered offline controls. Other registered cases accept `--scenario path.json`.
- `mise run jev-case -- replay --scenario path.json` compares an ordinary-input
  DynamicScenario on the current build. It does not evaluate a defect assertion.
- `mise run jev-case -- discover --spec path.json` explicitly permits paid
  collection for an unresolved objective issue. The private specification is:

```json
{"issue": 344, "start_level": 0, "goal": "Stomp a full-health walking Dragon once"}
```

`start_level` is a decimal logical level index. Discovery compares the settled
start against the oracle before requesting any paid decisions. Scenarios accept
ordinary boot or settled diagnostic warps and reject injected gameplay state.
A settled warp establishes diagnostic coverage, not natural level entry.

Standard recording bundles and an HTML frame viewer are written beneath
`.cache/jev-cases/<run>/`. Reports separate collection, comparison, encounter and
proof results. An unregistered assertion is `not evaluated`; navigation success
cannot prove a fix. Replay exits zero for a comparable match; check exits zero
only for `proven check`. Inconclusive outcomes exit one. The recorder continues
through death and title cards. All oracle errors and pixel differences remain
visible. `TYPESAFE_API_KEY` stays in the environment; offline subprocesses do not
inherit it. Interrupted calls keep their spend reservation.

Ordinary-runtime tests use constructor defaults. Calling
`set_recorded_title_card_handoff_delay(false)` selects the separate direct
capture path; it does not restore ordinary runtime behaviour.

Preserve the locked `.cache/jev-cases/budget.json` ledger across runs. Missing
accounting blocks paid requests; the controller never recreates the file.
The first campaign attempt records the existing charged upper bound plus the
user-authorised $5 allowance. Historical spend and attempts remain intact.
Restarts and new cases cannot replenish credit. The historical upper bound
is $0.176526462; an allocation at that value has a $5.176526462 total ceiling.
Three attempts per case, 600 decisions per attempt and a 20-minute collection
deadline apply within this campaign. Historical tokens cannot dispatch.
Each decision
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
