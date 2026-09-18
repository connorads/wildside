# Agent playtesting and measured self-improvement

Status: proposed visual-player and learning design. The separate
[Jev pilot](jev-playtesting.md) implements a private structured-state controller;
its [measured result](jev-playtest-pilot-result.md) does not establish successful
autonomous bug reproduction or tester improvement.

## Recommendation

Give an agent a controller and screenshots of the running browser game. Let it
play bounded episodes, inspect its failures, and propose revisions to its own
playbook and harness. Compare each revision with the previous version on
encounters that were unavailable during revision. Keep the game and examiner
fixed while measuring the tester.

The choice of what the agent can observe, change and grade is expensive to
reverse once learned procedures depend on it. A private, disposable control
adapter keeps the first experiment cheap to replace. The interface sketches
below do not establish a public API or stored-data compatibility commitment.

The capability being designed is an agent that chooses actions from the
consequences of its previous actions, produces reproducible playtest evidence,
and improves its ability to do that. Completing a prewritten input script is
replay evidence; interactive decisions must occur during the episode.

## Alternatives

Four alternatives differ in control mechanism, observation privileges and
where improvement is enforced. Each gets its strongest case before selection.

| Alternative | Mechanism and caller interface | Failure behaviour | Cost and fit |
| --- | --- | --- | --- |
| Existing automation, fixed instructions | Ask the current agent to play a short encounter using browser controls and screenshots. Use existing recording facilities where possible. No learning platform. | Wall-clock input timing, screenshot timing and recording failure may be confused with poor play. | Least new code. Establishes a baseline and may make a custom bridge unnecessary. Does not establish improvement by itself. |
| Visual player with procedural learning | A small `start / advance / finish` bridge supplies screenshots and bounded controller actions. Revise evidence-backed playbook entries between episodes. | Sprite recognition and missed events can cause failure. Frozen simulation makes model latency independent of gameplay time. | Fits the existing browser frame step and framebuffer. Needs recording integration and an external comparison to establish improvement. |
| Instrumented planner with motor programmes | Agent reads typed position, velocity, terrain and enemies, then issues goals such as `JumpTo` or `Approach`. Deterministic feedback controllers execute per frame. Revise those programmes and their selection rules. | Controller bugs can resemble game bugs. Privileged observations can bypass missing sprites or misleading terrain art. | Fits native trace data but introduces an observation schema and another behavioural layer. Strong candidate if reaching encounters becomes the measured bottleneck. |
| Evolving playtester population | Generate versions of instructions, memory and harness code. An external examiner promotes candidates from an archive after matched trials. A live observation/action bridge still provides play. | Strong rejection and rollback if the examiner is sound. Repeated selection can overfit evaluation cases; infrastructure can consume the entire experiment. | Broadest self-improvement scope. Highest initial ownership cost for isolation, selection and evaluation. |

Comparison by attribute, without a combined score:

| Attribute | Existing automation | Visual player | Instrumented planner | Evolving population |
| --- | --- | --- | --- | --- |
| Caller simplicity | Simple request, uncertain timing burden | Small explicit action interface | Simple goals, hidden controller complexity | Simple episode request after substantial setup |
| Cost of change | Lowest until workarounds accumulate | Small replaceable adapter | Coupled to engine observations and motor contracts | Most runner and evaluation machinery |
| Failure diagnosis | Weak without synchronised evidence | Bounded actions and raw frames expose mistakes | Must distinguish controller, planner and game | Explicit failed-candidate records and rollback |
| Existing repo fit | Uses browser and recording tools | Uses frame stepping, rendering and replay | Uses typed trace and terrain | Reuses evidence concepts; selection runner is new |
| Visual defect discovery | Direct visual exposure | Direct visual exposure | Needs a separate visual check | Depends on the chosen observation boundary |

The recommendation is the visual player, starting with an existing-tools
baseline and borrowing the incumbent-versus-candidate comparison from the
evolutionary design. Existing automation alone leaves improvement unmeasured.
The instrumented planner adds control-layer obligations and weakens dependence
on displayed pixels. A population runner adds more infrastructure than the
first comparison needs. None is demonstrated infeasible; they lose on the
initial experiment's cost and purpose. Save states, motor programmes and an
agent archive remain later options that must earn their maintenance cost.

## Existing foundations and gaps

- [`WildsideGame::step`](../../crates/wildside-web/src/lib.rs) accepts a raw
  controller mask and exposes the rendered framebuffer. The browser exposes
  the game instance as `window.wildside`.
- [`WebSession`](../../crates/wildside-web/src/session.rs) steps the native
  runtime, renders it and retains a typed trace. The WASM wrapper does not
  expose the full Rust trace; private examiner access needs deliberate wiring.
- [`web/main.js`](../../web/main.js) owns a continuous animation loop with an
  audio-aware clock. Calling `step` from automation while that loop also runs
  does not provide a controlled frame count. Manual advancement needs one
  clock owner and a completed render before observation.
- [`DynamicScenario`](../../crates/wildside-cli/src/scenarios.rs) represents
  per-frame input. Desktop [playtest recordings](playtesting.md) already store
  replayable segments, bookmarks and screenshots. Browser episode export into
  this workflow is integration work, not an existing feature.
- The [replay exploration pilot](replay-exploration-pilot.md) submits complete
  scripts. Its neutral control has a documented RNG discrepancy before search.
  It cannot substitute for interactive play or provide a clean parity start
  without investigation.

## Episode boundary

The proposed player sees current rendered pixels, its own previous actions and
frame numbers. It can request additional observations or retain intermediate
frames within its budget. Hidden collision geometry, enemy state, future inputs
and oracle output remain outside this visual player's observation surface.

The examiner can read typed state to verify task outcomes and classify events.
Keep that instrumentation out of the player's screenshots and response payload.
Agent-written crops, visual estimates and summaries derive only from permitted
observations. A later privileged player is a separate measured condition.

Sketch of the private driving interface:

```text
start(start_id) -> episode_id, observation
advance(episode_id, expected_frame, buttons, frames) -> observation, receipt
bookmark(episode_id, frame, suspected_symptom) -> bookmark_id
finish(episode_id) -> raw_episode

observation = frame_number + rendered_image
receipt = action_id + first_frame + last_frame + applied_masks
```

The first bridge limits an action to 1..30 frames. It applies the complete
button mask on every requested frame; zero means release all buttons. Frame
numbers and action identifiers detect stale or duplicated calls. Retrieving an
observation does not advance simulation. No ordinary animation tick runs
between actions. Resets belong to examiner-controlled attempts and consume the
attempt budget.

The trusted recorder owns the actual applied inputs, original frames, start
recipe and build identity. It retains evidence even when the candidate crashes
or declares the attempt finished. An agent's claim of success is a note, not the
examiner's task result. A discrepancy between requested and applied input is a
harness failure, not a failed gameplay skill.

Frame stepping measures visual decision-making with suspended thinking time.
It does not establish real-time input latency, audio-device quality or human
play stability. Those claims need separate execution through the relevant
browser or SDL path.

## What improves

The first revision can change the playbook, observation history selection,
visual crops, action-duration policy or a harness convenience justified by a
recorded failure. Each candidate includes the failure reference, a proposed
mechanism and a prediction that another episode can test. For example, shorter
action batches near a visually detected ledge might prevent repeated overshoot.
That is a hypothesis until the replay and transfer results support it.

The writer can replace or delete contradicted instructions. Limit both context
size and execution cost so improvement does not mean accumulating instructions
or spending more attempts. Preserve the incumbent, candidate diff and rejected
results. Fresh episode processes receive the selected version explicitly.

The immutable experiment boundary owns:

- the engine, assets and canonical start recipes;
- controller semantics, frame accounting and original evidence capture;
- task predicates, budgets and candidate selection rules;
- evaluation cases and oracle comparison policy.

Candidate execution and revision use separate workspaces from the examiner.
The play process gets only the observation/action interface. The revision
process gets the editable tester files and development evidence. Filesystem
and tool access must enforce that separation; a prompt or recorded hash alone
cannot prevent access to held-out cases. Candidates can change their wrapper
around the trusted controller, but cannot bypass its input limits or substitute
their own evidence.

Once a measured first improvement exists, a later experiment can let the agent
revise the procedure that proposes its next improvement. The same external
examiner then compares that revised procedure under an equal total improvement
budget. That adds recursion without allowing the procedure to redefine success.
The learned artefacts are instructions and code; this proposal does not train
model weights.

## Measurements

Keep three results separate:

1. Playing competence: independently observed task completion, deaths, stalls,
   unfamiliar encounters reached and total cost.
2. Playtesting value: distinct reproducible problems, their evidence and report
   precision. Deduplicate symptoms before treating them as distinct bugs.
3. Harness integrity: applied-input accuracy, frame alignment, deterministic
   reconstruction and retained evidence when an attempt fails.

A completed native episode measures native control competence. An alleged port
bug requires the same inputs and comparable starting conditions in the oracle.
Native/oracle bootstrap disagreement is an explicit unresolved comparison, not
evidence that the player failed or that the port has that reported defect.
Native play can proceed while that comparison is investigated. Existing parity
gates and the exploration pilot's strict admission rule remain separate.

Do not score success by distance travelled alone, number of reports, number of
new instructions or the agent's own confidence. These can increase without
better playtesting. A playtester may deliberately die or revisit a hazard to
test a hypothesis, so fewer deaths is not a universal objective either.

Agent findings are additional evidence. They do not silently count as the human
validation required by the current awaiting-human lane.

## First experiment

1. Run one short encounter through existing automation with fixed instructions.
   Check that actions affect the displayed game and that the saved input stream
   reconstructs the observed episode. If control or recording cannot meet that
   contract, add only the minimal bridge and recording integration above.
2. Pin the engine, model settings, context allowance and examiner. Establish two
   development encounters and four distinct evaluation encounters. Development
   results may guide revision; evaluation encounters are unavailable to the
   reviser before the candidate is frozen.
3. Run development play, retain failures and ask for one candidate revision.
   Do not require a bug discovery as a precondition for learning to play.
4. Run the incumbent and candidate on the four evaluation encounters with
   separate memories, matching starts and alternating order. Use the same
   ceilings of 1,200 simulated frames and 40 action decisions per attempt,
   plus an identical token ceiling selected before execution. Report wall time
   and improvement-generation cost as well as play cost.
5. Treat valid replay and frame accounting as prerequisites. A candidate earns
   a provisional win by completing at least one additional encounter, losing
   none completed by the incumbent, and respecting every budget. Repeat the
   matched comparison with fresh model calls before retaining it. Small
   samples remain pilot evidence, not proof of general capability.
6. Reserve later encounters for transfer. Repeatedly selecting against the
   same evaluation set leaks information even through a promotion bit. Retire
   exposed cases to development rather than calling them perpetually held out.

An inability to deliver and record correct controls stops learning work until
that specific harness fault is resolved. No measured improvement after the
first candidate and its repeat ends this pilot with its negative result; it
does not trigger an unbounded sequence of prompt rewrites. The evidence can
support a different next experiment, such as privileged motor control.

Any implementation plan needs separate, verified units for the minimal bridge
and replay contract, then the candidate comparison. Changes to an exposed
interface or stored recording shape need a compatibility decision before code
alters that surface. This document records the design without making that
decision or starting implementation.

## Research precedents

- [Cradle](https://baai-agents.github.io/Cradle/) uses screenshots and computer
  input for game interaction, with reflection and memory. Its reported timing
  and spatial-perception failures motivate measuring those limits directly.
- [Voyager](https://voyager.minedojo.org/) retains executable Minecraft skills
  and improves programmes using environment feedback. It supports the
  instrumented-controller alternative, not a claim that visual platforming
  will work with the same mechanism.
- [Darwin Godel Machine](https://arxiv.org/abs/2505.22954) modifies agent code
  and selects descendants using coding benchmarks. Applying empirical
  candidate selection to this playtester is a proposed experiment; its coding
  results do not establish game-playing gains.
