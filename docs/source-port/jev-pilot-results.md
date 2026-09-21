# Jev pilot evidence

The pilot has no fully proven case. Human backlog statuses remain authoritative.
The run reports preserve full state and pixel differences, including differences
outside each narrow symptom. Paid route collection is accounted separately below.

The #370 comparison covers every input frame, including the whole prefix and
tail. Native recovery cannot shorten the oracle evidence window. The source
revision, patch and runner hash are captured before execution. The negative
control reconstructs that revision and patch in an isolated checkout, so a
later commit or temporary hook stash cannot replace its source evidence.

| Case | Encounter and control | Remaining blocker |
| --- | --- | --- |
| #210/#254 | Ordinary rapid Start stays on the title card through frame 3299. Restoring the ordinary-runtime guard defect fails the same check after 2718 identical prefix frames. | Full comparison differs in RNG from frame 2700, intermediate Kid state and title lettering. |
| #370 | Natural breath fatal hit at 302. Native resets at 409 and resumes at 562; oracle resumes at 571 and lands at 583. Removing the projectile clear retains the stale child and fails the same check after 303 identical prefix frames. This control is synthetic. | Pixels match through 409. State still differs in player clearing at 396, camera clearing at 400 and loader RNG at 409. Title-loader pixels differ from 410. |
| #327 | Six neutral starting frames replay exactly and match oracle state and pixels after the HUD-clock reset. | No natural teleporter route is proven. Paid discovery requires this clean preflight. |

Reproduce #210/#254 with `mise run jev-case -- --issue 210` and #370 with
`mise run jev-case -- --issue 370`. Both return non-zero for an inconclusive
comparison. #327 supports `mise run jev-case -- --issue 327 --script <input-file>`
for offline replay. Its starting-state preflight passes, but the three-attempt
paid discovery allowance is exhausted.

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
The oracle RNG ring stays frozen through frame 412. The death-loop fix preserves
that ring through 408; the collapsed native loader advances it at 409.
The HUD timer decrements at frame 361 in both implementations.

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
checkpoints except 377, where that capture predates the fade fix. RNG first
differs at sampled frame 409, when the collapsed native loader starts.
All 107 frames from 302 through 408 match the existing full-window oracle
RNG offset, last byte and ring. The full reload case remains inconclusive.

The fade regression retains the outgoing gameplay scene and HUD through
`Pal_FadeOut`. The life counter changes in game state without rebuilding the
resident HUD. Palette DMA precedes interpolation, and the captured display
precedes the next DMA. `.cache/jev-cases/death-fade-final/` replays exactly and
matches oracle state and every pixel at 0, 302, 376 and every frame 377-395.

The full replay in `.cache/jev-cases/1789861434948961000/` captures all 620
frames. Native replay is exact; 417 frames match every oracle pixel, including
all frames 0-409. The synthetic cleanup control fails the same symptom check after a
303-frame identical prefix. The verdict remains `comparison inconclusive`.
Its source revision is `63bb97edc`, with the uncommitted renderer patch and runner hash
pinned before execution. The fast gate and all 37,759 TAS frames against existing
baselines pass for the fade and breath fixes. The 0-104 state sweep reports
69 passes, zero failures or errors and three unverified levels.

The complete oracle timeline in `.cache/jev-cases/370-research-timeline/` places
the Start handoff at 502, title-slot clearing at 529, object-count clearing at
533, the first gameplay object pass at 571 and landing at 583. The old native
frame 562 is not an oracle recovery witness.

Phased reload implementation requires more than connecting existing helpers.
VRAM/RAM clear timing visitors exist, but `TitleCardData` retains decoded art
and maps without their compressed streams. `Load_TitleCard` and `loc_1AB26`
lack complete caller/layout timing visitors. The final `Pal_FadeOut` wait
provides a timing origin; its poll and refresh uncertainty must be bounded
before deriving sampled loader boundaries. Observed frame numbers are regression
witnesses, not replacement production delays. Reusing the whole boot loader
would also import cold/debug entry assumptions and reset gameplay too early.

The breath animation regression follows `Init_Animation -> GfxObject_Animate
-> sub_1194`. The spawn pass loads the first descriptor's full eight-frame
timer even though native projectile lifetime has already decremented. Oracle
frames 241/248 retain sprite `$0550` with timers 8/1; frame 249 installs
`$0554` with timer 10. Descriptor selection excludes that initialisation pass.
The prior-SAT rendering latch and gameplay lifetime retain their source order.

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

## Campaign discovery - 2026-09-20

The authorised campaign has a separate fixed allowance in the same ledger.
Its first BLW2 attempt, `.cache/jev-cases/1789943268861172000/`, used 100
requests and $0.010448214. The lifetime charged upper bound is $0.186974676.
The original case window and attempt count remain intact after its deadline
was expired under the ledger lock to stop stalled collection.

The six-frame preflight passed the existing state/pixel comparator. Jev then
moved Kid from X32 to X58 and repeatedly chose release. The 600-frame journal
replays exactly but returns `not exercised`: no teleporter encounter occurred.
The global goal mixes approaching the pad, waiting and the destination. A
single current objective is a candidate controller experiment, not a measured
improvement. Do not repeat paid collection with the same stalled observation.

That attempt exposed #496. The Tank turns left at native frame 570 and oracle
frame 571; the latter frame differs by 494 pixels. Exact RAM comparison finds
a half-pixel body lead already at frame zero, hidden by the normal position
tolerance. A 600-frame idle control reproduces it without Jev. See the
[permanent regression and ASM evidence](backlog-archive.md#496). Paid discovery
finds a concrete defect here even though its requested route remains unproven.

## Controller coordinate units - 2026-09-21

The case adapter supplies Kid and terrain positions in pixels, but serialises
actor snapshots as raw 16.16 integers. Saved decision 72 places Kid at X58
and the Tank at raw X13,139,968, meaning X200.5. The private service converts
enemy/projectile positions and velocities to signed pixels before dispatch.
The decision journal stores the exact converted state sent to Jev. Native
trace values, existing journal files and budget accounting remain unchanged.

The regression fails before conversion and passes afterwards. It covers
fractional positions, wrapped negative positions, negative velocities, absent
velocity values, unchanged source data and the journal/request match. All 21
case-service tests and `verify-slice-fast` pass without an API call. The saved observation also
converts to the measured Tank position without changing its source snapshot.
No improved model policy is claimed from these offline checks.

The stalled collection contains 96 release decisions and four running jumps.
After Kid lands at (58,191), 88 consecutive decisions still release all controls.
An offline control replays the prefix through 71, then runs right for 12 frames:
Kid reaches X64.015625 with two HP and exact native replay. A longer continuation
reaches the lower shaft. Its all-frame comparison first differs visually at
186 and in authoritative state at 190. Investigate that slope/ledge exit under
#300 before treating the continuation as emulator-matched route evidence.
The case window, attempt count and $0.186974676 lifetime charge are unchanged.
A separate progress/stall guard remains required before further paid discovery.

## Collection progress guard - 2026-09-21

The BLW2 live collector stops after 24 six-frame Jev decisions without
improving Kid's best forward X position. It writes a stop reason and retains
the append-only input journal for exact replay. An offline regression covers
the saved release-stall pattern and a moving route with a temporary detour.
This bounds the known paid stall; no new Jev route or model-policy improvement
has been measured.

## Case-specific route collection - 2026-09-21

The private collector takes an explicit settled start, goal and optional
forward-progress limit. #327 retains its 24-decision X guard. #502 has a
separate level 0x2C Drips goal and checks the trace's `number_objects` field
for a 21-Object candidate. A candidate is discovery evidence only: the
`Object_CheckInRange` 20/21 call and native/oracle outcome still need proof.

An offline six-frame #502 run replays exactly but cannot pass the paid
preflight. At level 0x2C, frame 0 RNG differs (native FFCB, oracle FFF2)
and 39 pixels differ. Level 0x48 also fails at frame 0 (native four enemy
Objects, oracle ten; 19 pixels differ). No paid #502 request was sent.
