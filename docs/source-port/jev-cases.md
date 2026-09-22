# Jev bug cases

`mise run jev-audit` joins `jev-cases.json` to the live backlog. `--json`
prints the joined records. Frozen lanes are excluded. Backlog status and
priority remain authoritative; the catalogue never changes status. Objective
closure follows [the completion campaign](completion-campaign.md); subjective and
device checks remain for the final human playtest. Route selection and exploration
cadence follow [the loop](loop.md); its [instruction evaluation](jev-loop-evaluation.md)
records the decision checks. `verify-slice-fast` includes the catalogue audit.

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

The optional discovery policy fields are:

```json
{
  "issue": 344,
  "start_level": 0,
  "goal": "Approach the walking Dragon without passing it",
  "actions": ["release", "left", "right", "jump", "left_jump", "right_jump"],
  "progress": "cells",
  "max_decisions": 100,
  "target": {"kind": "position", "level": 0, "x": 128, "y": 191, "radius": 16, "dwell_frames": 1},
  "tail_frames": 30,
  "stop_on_death": true
}
```

This target describes a navigation candidate; it does not establish a stomp or
any defect assertion. Omit `target` for bounded exploration. Other registered
candidate targets are `blw2_teleport`, `dragon_breath_death` and
`object_pressure_candidate`, each containing only `kind`. The Object-pressure
candidate means a frame count of at least 21; actual Drips range-call evidence
is still required. Death goals need `stop_on_death: false` to retain their tail.

The default `cells` policy stops after 24 decisions without entering a new
16-pixel cell, keyed by level. It counts leftward and vertical travel; revisiting
only known cells eventually stops. `rightward` explicitly selects the historical
best-X policy. A target's continuous dwell suspends the stall counter for its
bounded duration. Target and death checks run every frame. Target tails use
ordinary release inputs without more paid calls and count against the suffix
frame limit. Reports distinguish a witnessed target from a completed tail.

`actions` defaults to all supported controls and must include `release`.
Supported names are `release`, `left`, `right`, `up`, `down`, `a`, `jump`,
`special`, `start`, `left_jump`, `right_jump`, `run_left`, `run_right`,
`run_left_jump`, `run_right_jump`, `left_special`, `right_special` and
`jump_special`. A/B/C are controller buttons; their default bindings are
run/jump/special. Options can remap them. Observations include helmet ID, pause,
held buttons, pixel positions and velocities. Start is offered at boot/title
or while paused; a saved prefix can enter a pause deliberately.

`start_level` is a decimal logical level index. Discovery compares the settled
start against the oracle before requesting any paid decisions. Omit `start_level`
for ordinary boot. To continue a saved route, replace `start_level` with
`"prefix": "route.json"`, relative to the specification file. The prefix supplies
its own start and must contain no injected state. `max_decisions` limits the new
suffix to 1..600 six-frame decisions; the prefix does not consume that limit.

Preflight compares frame zero, the prefix's existing checkpoints and each frame
of its final 30-frame handoff window. A green result covers those checkpoints.
The report lists their indices. Collection preserves held controls at the
handoff and verifies exact replay of the complete input stream. Source identity
is checked before collection, before each paid request and after collection;
the collector also checks the preflight ROM identity. Changed source or ROM
requires another preflight. Runs retain a resolved specification, prefix and
source hashes alongside their recording bundle. Scenarios accept
ordinary boot or settled diagnostic warps and reject injected gameplay state.
A settled warp establishes diagnostic coverage, not natural level entry.

Standard recording bundles and an HTML frame viewer are written beneath
`.cache/jev-cases/<run>/`. Reports separate collection, comparison, encounter and
proof results. An unregistered assertion is `not evaluated`; navigation success
cannot prove a fix. Replay exits zero for a comparable match; check exits zero
only for `proven check`. Discovery reports `route collected` when the saved
inputs replay comparably, even if its target was not reached. Read its collection
outcome separately. Request failures report `collection interrupted` and exit one. Inconclusive outcomes exit one. The recorder continues
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
Three attempts per issue and 600 decisions per attempt apply within this
campaign. The 20-minute collection window starts at that issue's first attempt;
restarting, changing its goal or choosing another prefix does not renew it. Historical tokens cannot dispatch.
Each decision
applies six frames, with at most 3,600 newly collected frames per attempt.
Saved prefix frames are retained separately from this limit. Pricing and request
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
tail. Decision journals retain the exact converted state, question, action rubrics
and response sent to the controller.

Isolated controls copy an idle Cargo target directory. `CARGO_TARGET_DIR` is
honoured. Workspace build-script wrappers are rebuilt in the copied target
because build-wrap embeds their original paths. Sandboxing remains enabled.
Do not clone a target directory while another Cargo process is replacing its
object files.
