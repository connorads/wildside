# Jev pilot evidence

The pilot has no fully proven case. Human backlog statuses remain authoritative.
The run reports preserve full state and pixel differences, including differences
outside each narrow symptom. Paid route collection is accounted separately below.

| Case | Encounter and control | Remaining blocker |
| --- | --- | --- |
| #210/#254 | Ordinary rapid Start stays on the title card through frame 3299. Restoring the ordinary-runtime guard defect fails the same check after 2718 identical prefix frames. | Full comparison differs in RNG from frame 2700, intermediate Kid state and title lettering. |
| #370 | Natural breath fatal hit at 302, reset at 409, living gameplay at 562, observed through 592. Removing the projectile clear retains the stale child and fails the same check after 303 identical prefix frames. This control is synthetic. | Death-loop RNG and HUD timing pass targeted oracle checks; fade pixels differ at 377 and loader RNG at 409. |
| #327 | Six neutral starting frames replay exactly and match oracle state and pixels after the HUD-clock reset. | No natural teleporter route is proven. Paid discovery requires this clean preflight. |

Reproduce #210/#254 with `mise run jev-case -- --issue 210` and #370 with
`mise run jev-case -- --issue 370`. Both return non-zero for an inconclusive
comparison. #327 uses `mise run jev-case -- --issue 327 --live`; the failed
preflight prevents an API request.

## Rapid Start evidence

The ordinary constructor path consumes Start during the title-menu fade,
matching `Obj_TitleMenu loc_1BBD6 -> loc_6E2`. The regression checks both
ordinary and recorded replay paths, and verifies that a fresh later Start
still enters gameplay. The cache-backed regression, `verify-slice-fast`
and all 37,759 frames of `verify-tas-full` pass against existing baselines.

Local artefacts from the 2026-09-19 run:

- Fixed case: `.cache/jev-cases/1789833624894910000/`.
- Before: `.cache/jev-cases/preflight/210/`.
- Corrected full reload case: `.cache/jev-cases/1789833218331609000/`.
- Teleporter preflight: `.cache/jev-cases/preflight/327-start/`.

These paths are disposable generated evidence. The checked-in inputs, runner,
regressions and this result record provide the durable reproduction procedure.

## Reload investigation

`Death loc_B80C -> sub_8E0` runs HUD drawing without `ExecuteObjects`.
The oracle keeps the Dragon-breath child frozen after the fatal hit. That is
original behaviour, not evidence that the breath must animate during death.
The oracle RNG ring stays frozen through frame 412. Native advances at 303.
The HUD timer visibly decrements at oracle frame 361 while native stays frozen.

The current 32-frame hidden-death shortcut also combines loader phases that
the oracle separates. In the natural route, fade runs from 377 through 394,
VRAM clearing from 395 through 404, RAM clearing through 408, the first loader
object pass at 413, letters at 417 and the deferred number at 418.
Changing a delay alone does not reproduce this ordering. Probe evidence lives
in `.cache/jev-cases/370-loader-probe/`.

The death-loop regression preserves the fatal frame's entered object-pass draw,
then freezes RNG during `DeathWindup` and `Dying`. `DeathWindup` still ticks
`loc_BE82`; the hidden fade does not. The natural route in
`.cache/jev-cases/death-loop-fix/` replays exactly. Sampled oracle state agrees
at 0, 302, 303, 359, 360, 361, 376, 377 and 408. Pixels also agree at those
checkpoints except 377, where the known fade boundary differs. RNG first
differs at sampled frame 409, when the collapsed native loader starts.
All 107 frames from 302 through 408 match the existing full-window oracle
RNG offset, last byte and ring. The full reload case remains inconclusive.

## Settled-entry HUD evidence

`loc_720` clears the animation offsets and `Load_InGame -> sub_44B0` seeds
all five countdowns. The debug loader applies this reset at its existing
live boundary, before the first HUD pass. It needs no diamond-only adjustment.
The BLW2 regression checks all ten bytes at frames 0 and 5. The level 0x55
recording-handoff diamond witness and the other settled-warp regressions pass.
Main-checkout replay in `.cache/jev-cases/hud-loader-fix/` matches oracle state
and every pixel on all six frames. This proves the starting comparison,
not the teleporter encounter.

The full requested level range 0-104 passes the state sweep for all 69 runnable
levels, with zero failures and zero errors. Three remain unverified because
their oracle checkpoints are not comparable. These verdicts match the prior
sweep. Result: `.cache/sweep-runs/1789835842/report.json`.

## Paid route collection

Three BLW2 collection attempts use 1,170 decisions in total. The cumulative
spend upper bound is $0.176526462, including the prior $0.024954048. The new
collection adds $0.151572414. No attempt reaches the teleporter, and the
three-attempt allowance is exhausted.

| Attempt | Frames | Finding |
| --- | --- | --- |
| 1 | 1,212 | Pauses at frame 104 and stays paused. The observation lacks the pause flag. |
| 2 | 2,208 | Pause state is exposed; best x=614. Running actions still press C rather than A. |
| 3 | 3,600 | Correct A mapping and named held buttons; best x=746. Stalls on the death-reload title card despite recorded state guidance. |

The adapter regression pins A/B/C mappings. Pause state and named held
buttons are explicit observations. Choice rubrics describe when each control
is appropriate, following the [TypeSafe choice API](https://docs.typesafe.ai/api).
Those richer rubrics are added after the collection allowance is exhausted;
their effect on Jev's gameplay is not yet measured. New journals retain the
exact question so prompt changes are reviewable.

Original collection bundles are retained under `.cache/jev-cases/`:
`1789835385159833000`, `1789835681424562000`, and `1789836030152955000`.
Their full-route rendering was stopped after the missing encounters were
established. Offline reviews replay the complete controller scripts and
capture frame zero plus the final second. Inputs are retained in
`.cache/jev-cases/pilot-inputs/attempt-{1,2,3}.json`; reproduce each with
`mise run jev-case -- --issue 327 --script <input-file>`.

These are unsuccessful discovery attempts. They establish neither a
teleporter reproduction nor a fix. The synthetic #370 control is also not
a reconstruction of a historical build. No human validation status changes.

All three offline reviews reproduce their complete collected native traces
exactly and return `not exercised`, each with 61 comparison frames:
`1789836617957192000`, `1789836718430023000`, and `1789836811525261000`.
The current isolated #370 control also passes after rebuilding relocated
build-script wrappers, with a 303-frame identical prefix and the expected
failed cleanup check. Evidence: `.cache/jev-cases/negative-control-rebuild-check-2/`.
The adapter passes all 23 Jev script tests and `verify-slice-fast`.
