# Source-Port Methodology

Reference for reverse-engineering discipline, testing strategy, and fidelity principles. `CLAUDE.md` has the operational summary; this document explains the why and how in depth.

## Philosophy

### Evidence ladder, not binary completion

Use these status labels consistently when describing verification depth:

- `stubbed`: implementation exists only as scaffolding or placeholder behaviour
- `boots`: flow reaches the intended screen or runtime state without proving fidelity
- `replay-matched`: deterministic replay/TAS path matches the oracle on its recorded route
- `branch-matched`: replay-derived checkpoints also survive short perturbed continuations
- `mechanic-swept`: isolated mechanic surfaces or focused scenario matrices match across the tested parameter space
- `human-play-stable`: repeated native play no longer finds fresh player-facing breakage in that area

`replay-matched` is necessary and valuable, but it is not equivalent to `human-play-stable`.

### Accuracy is the goal; compatibility is the outcome

bsnes (byuu): "Accuracy is not 'does it play Zelda?' playing games is a side effect of accuracy." Fix the engine, never the game. Accurate reimplementation enables compatibility; the reverse is not true.

### Bugs are features

Original game bugs must be faithfully reproduced:

- **Julius** (Caesar III): 100% bidirectional savegame compatibility, so any logic deviation breaks saves. Augustus (fork) proved this by breaking compatibility when it added gameplay changes.
- **OpenMW** (Morrowind): intentional bug preservation for mod compatibility. Attack angle uses flawed dot product math in the original; OpenMW reproduces the flaw.
- **Chocolate Doom**: demo sync requires frame-perfect reproduction of original behaviour including bugs. The Lost Soul bouncing bug fix in Ultimate Doom broke all earlier Doom II demos.
- **devilutionX** (Diablo): "Don't fix bugs found during RE, reproduce them exactly to validate correctness."

Mark preserved bugs with `// ORIGINAL_QUIRK: <description>` so they are visible and intentional.

### Progressive fidelity

Get it working first, get it accurate second, and track the gap. Approximate implementations are acceptable stepping stones when marked `// TODO(fidelity): <description>` and tracked in the backlog. Never ship a guess as if it were verified.

### Code as documentation

Implementation should be readable as a specification of original behaviour. bsnes: speed hacks and convenience features "destroy code's ability to serve as verifiable hardware documentation." Avoid clever abstractions that obscure what the original hardware/software did.

### Conservative design

bsnes: "Better to not allow valid behaviour than to allow invalid behaviour." Conservative design prevents broken hacks or ROM translations from depending on emulator bugs. When uncertain, preserve original behaviour verbatim rather than guessing at a "better" interpretation.

---

## Investigation Playbook

### Root cause taxonomy

| Category | Symptoms | Primary tools | ASM reference |
|----------|----------|--------------|---------------|
| Timing | Wrong frame for state change, desync drift | `compare-native-oracle`, frame-by-frame stepping | `kid.asm` main loop, interrupt handlers |
| Collision | Stuck in walls, falling through floors, wrong stomp | `capture-frames` + layer overlay, position trace | collision routines in `kid.asm` |
| State machine | Wrong lifecycle transition, stuck states | `probe-native` with state checkpoint logging | player/enemy state handlers |
| Rendering | Wrong tiles, palette, sprite position | `diagnose-frame --save-layers`, pixel diff | VDP register writes |
| Audio | Wrong BGM, missing SFX, timing | `verify-audio-driver`, `verify-audio-output` | `doc/sound.txt`, GEMS engine |

### Structured investigation workflow

This mirrors byuu's investigation loop (identify, capture, hypothesise, probe, verify, fix, sweep):

1. **Observe**: capture frames or run scenario showing the problem
2. **Classify**: which root cause category? (Use taxonomy table)
3. **Probe oracle**: run the same scenario against jgenesis, capture state checkpoints at the same frames
4. **Read disassembly**: follow source precedence, `kid.asm` then `variables.asm` then `doc/formats.txt`
5. **Cross-reference**: native logic vs ASM vs oracle data. All three must agree:
   - ASM + oracle agree, native differs: **native bug**
   - ASM + native agree, oracle differs: **oracle limitation** (rare but possible; 8 warp-broken levels are examples)
   - All three disagree: **deeper investigation needed** (hardware docs, web research)
6. **Write probe scenario**: targeted `DynamicScenario` exercising exactly this behaviour (like bsnes test ROMs, which exercise specific undocumented behaviours with observable outcomes)
7. **Fix**: implement the smallest correct change
8. **Verify**: probe scenario passes, full sweep passes, visual before/after comparison

### Iteration workflow: agent-proposed vs human-closed

The autonomy loop's gameplay-bug investigation lane must never close a backlog item on its own. It owns the `[~]` to `[?]` transition only.

Status transitions:

- `[ ]` open. Selector picks from here by priority.
- `[~]` agent investigating. Set on entry to the lane. Evidence accumulates under the backlog row.
- `[?]` agent proposes a fix complete. Evidence block lists repro scenario path, cited ASM labels, oracle-parity result, and the commit hash of the proposed fix. Human must validate before promoting to `[x]`.
- `[x]` closed by a human.
- `[b]` blocked: needs human repro or external artefact.
- `stalled:agent-blocked` auto-applied when the per-issue attempt counter exceeds the cap. Selector skips until a human re-inspects.

Agent lane guardrails:

- Never writes `[x]`. A reviewer converts `[?]` to `[x]` once the fix is validated.
- Every investigation commit must either touch `crates/wildside-core/src/**` (fix + regression test) or the backlog lanes: a status transition / lane move in `docs/source-port/backlog.md` and the evidence block under `#NNN` in `docs/source-port/backlog-archive.md`. Pure cache-local churn does not count as progress.
- On stall, write the dead end (rejected hypotheses included) under the issue's `#NNN` in `backlog-archive.md` and leave status at `[~]`. No code commit means write the dead end and stop, never commit churn.

### Trace comparison techniques

When investigating divergences between native and oracle:

| Technique | When to use | How |
|-----------|------------|-----|
| Field-level state diff | Any divergence | Instead of flat "MISMATCH", report per-field delta with tolerance. E.g. "player.x: +4px (within tolerance), player.vx: +2 (ERROR)" |
| Framewise CRC | Narrowing divergence origin | CRC32 entire game state each frame; first mismatch = exact divergence frame |
| Subframe checksumming | Divergence isolated to one frame | Checksum after each update phase (player, enemies, collision) to narrow to subsystem |
| Frame-level bisection | Large frame range to search | Binary search: checkpoint at midpoint, recurse on divergent half. O(log N) vs O(N) |
| DTW trace alignment | Oracle and native differ in frame count | Dynamic Time Warping aligns traces despite timing drift (Sakoe-Chiba band +-2 frames) |

Sources: bsnes cycle-addressable trace logs, AoE deterministic lockstep desync detection, DTW (Sakoe & Chiba 1978).

### Formula reverse-engineering

When reimplementing a game formula (physics, damage, spawn rates), use the OpenMW empirical approach:

1. Set up controlled test conditions in oracle (known inputs)
2. Capture output values at checkpoints
3. Read disassembly for the formula's ASM implementation
4. Derive Rust equivalent from ASM
5. Compare Rust output against oracle across the full input range
6. Iterate until outputs match for all test cases

---

## Testing Taxonomy

### Architecture layers to test types

| Layer | What to test | How | Volume | Examples |
|-------|-------------|-----|--------|----------|
| Pure core | Physics, speed tables, slope formulas, FP16.16 arithmetic | Unit tests, property-based (proptest) | Most tests | `timing_regression_*`, slope formula tests |
| Enemy logic | AI state machines, activation, movement patterns | Unit tests + probe scenarios | Many tests | `test_enemy_*` family tests |
| Oracle comparison | Full engine state at checkpoints | `compare-native-oracle` scenarios | Per-feature | `DynamicScenario` with targeted inputs |
| Sweep | All levels idle state | `sweep --tier state` | 69 BCD levels | Pre-merge gate |
| Visual | Pixel-perfect frame output | `capture-frames` + byte comparison | Key frames | Before/after fix verification |
| Gameplay | Multi-frame player journeys | `verify-gameplay` scenarios | Key paths | pause/resume, death/respawn, transitions |

### Verification lane split

Once the active TAS frontier is closed, keep the verification lanes separate:

- `sweep --tier state` answers "do the 69 BCD levels still avoid blocking authoritative diffs under the idle oracle harness?"
- `release-gate --target 1p-playable` answers "do native gameplay smoke, focused gameplay regressions, and the 69-level native gameplay sweep stay green together?"
- `release-gate --target 1p-accurate` answers "do boot/title, gameplay compare, the bounded source-port visual ratchet, ROM-emulator VDP checks, audio-oracle, and state-sweep all stay green together?"

The visual steps have different meanings. `source_port_frame_oracle:level_00_early`
renders the source-port scene at four settled-warp Level 0 checkpoints and requires
exact state, no masked checkpoints, complete diff images, MAE 0, and zero changed
pixels. It is bounded pixel-exact evidence, not authoritative cold-boot or
whole-game evidence. The legacy `frame_oracle:*` steps render the ROM
through wildside-md and compare that VDP output with jgenesis; they test emulator
shell/capture behaviour, not source-port pixels.

2026-07-04 refresh: `env TMPDIR=/tmp mise run verify-1p-playable` passes the
full `1p-playable` lane. `.cache/release-gate/1p-playable/latest.json` points
at timestamp `1783122908`, with report
`.cache/release-gate/1p-playable/1783122908/report.json`; native gameplay
smoke, gameplay compare/verify, focused regressions, and `gameplay_sweep:0-104`
passed, with gameplay sweep totals of 69 passed, 0 failed, and 0 errors.

2026-07-04 refresh: `env TMPDIR=/tmp mise run release-gate` passes the full
`1p-accurate` lane. `.cache/release-gate/1p-accurate/latest.json` points at
timestamp `1783122304`, with report
`.cache/release-gate/1p-accurate/1783122304/report.json`; the embedded state
sweep report is `.cache/sweep-runs/1783122357/report.json` (69/69 passed,
0 authoritative/advisory/accumulation/rng, 1037 oracle limitations,
`unverified=69`).

2026-07-10 refresh: `env TMPDIR=/tmp mise run release-gate` passes at timestamp
`1783701644`. Its real source-port visual step compares four unmasked early Level 0
checkpoints with exact state; worst visual drift is MAE `1.677` and `2,293` changed
pixels at frame 30. The embedded state sweep is
`.cache/sweep-runs/1783701669/report.json`: 69/69 passed but `verified=0/69`, with
all 1,011 diffs classified as oracle limitations.

2026-07-10 later refresh: `env TMPDIR=/tmp mise run release-gate` passes at
timestamp `1783717381`. Its four unmasked Level 0 source-port checkpoints have
exact state, MAE `0.000`, and zero changed pixels under the exact visual
baseline. The embedded state sweep is
`.cache/sweep-runs/1783717406/report.json`: 69/69 passed but `verified=0/69`,
with all 1,011 diffs classified as oracle limitations.

On 2026-04-22 those lanes were green but still intentionally separate (dated record; bullets below reference the orchestrator retired in the 2026-06-10 autonomy demolition, tag `pre-autonomy-demolition`):

- The delegated sweep-parity lane completed (`cycle=127` locally, via the since-retired orchestrator). The previous repeated-fingerprint recovery step `refresh_evidence` remains complete; the regenerated source-port package is cycle `600` with fingerprint `gate-1776819385:sweep-1776819420`.
- `cargo run --release -p wildside-cli -- sweep --levels 0-104 --tier state` passed all 69 BCD levels with only advisory/oracle-limitation classifications. The direct sweep report is `.cache/sweep-runs/1776819377/report.json` (`failed=0`, `errors=0`, `advisory=178`, `oracle_limitation=662`), and the release-gate-embedded sweep refreshed `.cache/sweep-runs/1776819420/report.json` with the same zero-blocking shape. Each `field_summary` entry still carries its own `divergence_summary`, concrete BCD `level_ids`, unique checkpoint labels, and checkpoint counts when diffs are present.
- `mise run release-gate` passed the full `1p-accurate` lane. `.cache/release-gate/1p-accurate/latest.json` now points at timestamp `1776819385`, with report `.cache/release-gate/1p-accurate/1776819385/report.json`; native bootstrap, all gameplay compares, frame-oracle checks, audio-oracle checks, and `state_sweep:0-104` passed.
- The older `pause_resume_level1` / `level_settle@719` `game_mode` failure is historical context only. A red future gate should still carry `release-gate-failure` plus the matching persisted `release_gate_focus.summary`; a green gate should instead carry `release-gate-status` and `state-sweep-status` so the durable handoff quotes the latest passing evidence.
- Treat the newest persisted state-tier sweep report in `.cache/sweep-runs/<timestamp>/report.json` as equivalent evidence to the mirrored `sweep-history.jsonl` entry when refreshing that handoff. If the report exists but the history append has not landed yet, the autonomy layer should still consume the fresher report so the state-sweep status does not freeze on an older timestamp.
- Sweep-parity stall detection keys the lane by both persisted evidence timestamps: the latest `1p-accurate` release-gate timestamp and the latest state-sweep timestamp. A successful refresh should move at least one of those values; otherwise the next source-port package correctly treats the handoff as the same stale sweep lane and can re-enter `refresh_evidence` recovery.
- A missing state-sweep or release-gate artefact populates `task.missing_artefact` on every cycle (not only on `definitively_stalled`), and the runner auto-recovers by running the deterministic regen command, bounded to one attempt per fingerprint via `.cache/autonomy/source-port-frontier/auto_recovery_attempts.json`. After auto-recovery succeeds the runner re-invokes the prompt script so the next iteration sees the refreshed artefact instead of tripping `is_clean_repeat`. Operator unblock hints are state-aware: only commands whose preconditions hold (uncovered ASM coverage, unpromoted backlog items, broaden-PV-config) are rendered, and the missing-artefact regen command becomes the top hint.
- When multiple release-gate failures share the same field name, correlate the latest state-sweep evidence by the full release-gate focus key first: `target`, `step_name`, `checkpoint`, `field`, and the exact native/oracle value pair when available. Only fall back to field-only correlation for older sweep-history entries that do not yet persist the richer metadata. Otherwise automation can attach a true statement about the wrong failing step, which is worse than surfacing no correlation at all.
- Use the sweep `level_ids` and checkpoint labels as the next lookup before changing code when a future red gate appears. If the failing release-gate field already maps to known BCD coverage and checkpoint family, harden the verification story first; only reopen gameplay work when the field or category escapes that known sweep envelope.

Treat sweep-parity evidence as a release-gate verification lane, not as evidence to reopen TAS work or to claim whole-game completion.

**Self-consistency sweep vs authoritative cold-boot gate (issues #26, #7).** The idle `sweep` is a *self-consistency* check, not the authoritative fidelity gate. Both sides warp into the level — native via the synthetic `boot_to_level` shortcut, the oracle via the ROM level-skip cheat — so their warp-arrival initial conditions (RNG offset, enemy/Murderwall tick age, camera) differ. Any checkpoint where the oracle never reached a comparable gameplay state (paused, on the title card, or on the wrong warp level) is masked to `oracle_limitation`, so a level can read as `passed` while nothing real was compared. Such a level is **UNVERIFIED**, not faithful: read the sweep's `verified`/`unverified` line (and per-level `masked_checkpoints`/`total_checkpoints`), not `passed`, as the coverage signal — a fully-masked level proves only "native did not crash". Warping is legitimate here precisely because the sweep is self-consistency. The authoritative gate is `verify-tas`, which is meant to **cold-boot** (`warp_to_level: None`) and reach the level by replayed input from power-on, exactly like the published TAS. When a scenario warps, `verify-tas` flags it as non-authoritative (an advisory warning, `cold_boot_advisory`): a warp means the run proves self-consistency, not cold-boot fidelity. The flag stays advisory rather than a hard reject because committed frontier regression probes (e.g. `issue305_pause_give_up_last_life_probe`, `warp_to_level: 0`) legitimately auto-boot through this path. By decision (issue #7, resolved path (a)) the gate keeps the warp flag **advisory**: a hard reject is a deferred follow-up, not adopted now, because committed cases legitimately warp and would all need migrating first.

> **Policy — authoritative cold-boot (issue #7, decided).** Any scenario used as authoritative
> fidelity evidence MUST cold-boot (`warp_to_level: None`) and reach the level by replayed input
> from power-on, exactly like the published TAS. Synthetic `enter_level_playtest` warps
> (`crates/wildside-core/src/scenario.rs:109`, builder at `:324`, field at `:283`) are for local
> iteration and the self-consistency sweep only.
>
> **Rationale.** A warp spawns into a level with different RNG / enemy-tick-age / camera than the
> ROM's title-card / level-skip arrival, so the warped run's steady state can never match the
> oracle (e.g. DragonWalking spawn-X off by `0xD0000`/208px, `game_paused` true-vs-false,
> divergent `camera_y`). Comparing such a run against the oracle generates false frontiers that
> burn agent cycles.
>
> **Gate behaviour (decision = advisory, not hard reject).** `cold_boot_advisory`
> (`crates/wildside-cli/src/tas.rs:333`) warns on any `warp_to_level`-set scenario (printed at
> `:358`, never fatal); the sweep reclassifies warp-arrival diffs as oracle-unreliable
> (`crates/wildside-cli/src/sweep/mod.rs:950`, `oracle_unreliable_checkpoints`); and
> `settled_recording_oracle_bootstrap_block_reason` (`crates/wildside-cli/src/case.rs:920`) blocks
> settled-recording bootstrap mismatches as an initial-conditions artefact. It stays advisory
> because committed cases legitimately warp (warped `tests/data/dynamic-scenarios/*` probes such as
> `issue-305-pause-give-up-last-life-probe.json`, `warp_to_level: 0`); a blanket reject would turn
> the green frontier gate red. **Hard reject (path b) is deferred** — promoting it would first
> require migrating those warp fixtures to cold-boot routes. See #26 (sweep self-consistency); the
> decision unblocks #10 (cold-boot fuzzing).

### TAS (Tool-Assisted Speedrun) fidelity testing

TAS recordings are deterministic per-frame button sequences from real full-game playthroughs. They exercise code paths that synthetic scenarios do not: level transitions, helmet pickups, boss fights, warps, and full boot-to-completion.

Workflow:

1. Obtain a BK2 file (BizHawk TAS format) from [TASVideos.org](https://tasvideos.org/UserFiles/Game/180)
2. Import: `mise run import-tas path/to/recording.bk2`
3. Quick test (first 1000 frames): `cargo run -p wildside-cli -- verify-tas --scenario tests/data/tas/<slug>/scenario.json --frame-end 1000`
4. Full run: `mise run verify-tas tests/data/tas/<slug>/scenario.json`

How it works:

- BK2 is a ZIP containing `Header.txt` (system metadata) and `Input Log.txt` (per-frame button state)
- `import-tas` parses the BK2 and converts to `DynamicScenario` JSON with `warp_to_level: None` (power-on boot)
- Canonical imported TAS scenarios live under `tests/data/tas/<slug>/scenario.json` so clones on other machines can replay and inspect them without rebuilding local cache state
- `verify-tas` runs the scenario through both native and oracle from frame 0
- Level transitions are auto-detected from the native trace and injected as additional checkpoints
- The oracle scenario JSON includes transition checkpoints, giving diagnostic data at level boundaries
- `--frame-start`/`--frame-end` slicing enables iterative debugging of specific sections
- `verify-tas` reports boot diagnostics for the first authoritative mismatch (phase, elapsed, title readiness, menu slot when available)
- Trimmed 250-frame boot-slice fixtures live in `crates/wildside-cli/tests/fixtures/tas/` so early boot regressions stay reproducible without cache-local BK2 imports

Available recordings (Kid Chameleon):

| Recording | Format | Frames | Use case |
|-----------|--------|--------|----------|
| Any% by CaseyTAS (10:30), TASVideos #6944 | BK2 | ~37k | Warps, transitions, bosses |
| KC Any% TAS 2024, TASVideos user files | BK2 | ~4,900 | Quick smoke test (warp code) |

### Semantic case bundles

Long deterministic replays stay as the regression lane. Discovery work should prefer small reduced case bundles derived from replay windows or native recordings.

Phase-1 bundle workflow:

1. `wildside case derive` creates a per-frame semantic bundle for a chosen window
2. `wildside case branch` generates short seeded continuations with a human-like action grammar
3. `wildside case verify` compares native vs oracle semantic state for one bundle
4. `wildside case reduce` shrinks a failing continuation while preserving the first semantic failure
5. `wildside case summarise` refreshes the agent-facing markdown summary

Each case bundle should answer:

- what checkpoint it starts from
- what the first bad semantic frame/field is
- which fields still match
- which subsystem is the likeliest owner

Loop sessions consume these bundles through `docs/source-port/loop.md`:

- `backlog.md` is the open queue and the NOW block is the handoff; `mise run project-status` picks the next item in priority order (`[?]`/`[b]` rows live in `backlog-awaiting-human.md`)
- grep `backlog-archive.md` for `#NNN` before forming a hypothesis, since dead ends are recorded there
- recording-backed cases outrank generic replay slices when both exist for an issue
- if tracked git changes already exist when a session starts, treat them as the highest-priority recovery task; verify/commit or intentionally discard that coherent unit before taking a new item
- when no backlog item is actionable, use the proactive-verification fallback in `loop.md` (mutation-test one file, audit one enemy family vs ASM, extend ASM coverage)

### Decision tree

| Situation | Test type | Rationale |
|-----------|-----------|-----------|
| New engine behaviour | Probe scenario + oracle comparison | Like bsnes test ROMs: exercise specific behaviour, compare to ground truth |
| Fixed bug | Timing/gameplay regression test (`verify-slice-fast` gate) | Like Chocolate Doom statcheck: every fix becomes a permanent regression test |
| Pure function, large input space | Property-based test (proptest) | Speed tables, slope formulas, gravity curves: verify properties hold across range |
| Stateful system, complex transitions | Stateful property test (proptest-state-machine) | Model lifecycle as states, generate random input sequences, auto-shrink failures |
| Complex state hard to assert field-by-field | Snapshot test (insta) | NativeTraceFrame at checkpoint: detect unexpected state changes between commits |
| Visual correctness | Frame capture + pixel comparison | Like MAME acid tests: any pixel mismatch against oracle = regression |
| Determinism | Replay same scenario twice, diff output | Like Chocolate Doom: deterministic engine is prerequisite for all other testing |
| Engine-wide invariant | Metamorphic test (see below) | Catches classes of bugs without per-case expected outputs |
| Test suite confidence | Mutation testing (`cargo-mutants`) | Measures what % of injected faults tests catch; surviving mutants = gaps |
| Parser / boundary code | Fuzz testing (`cargo fuzz`) | Coverage-guided input generation finds crashes humans miss |

### Property-based testing guidance

Use `proptest` for pure functions where the input space is large or combinatorial.

| Module | Function/area | Property | Generator |
|--------|--------------|----------|-----------|
| `physics.rs` | `apply_horizontal_physics` | Velocity never exceeds `max_run_speed` for any input sequence | Random `(SpeedTable, Direction, i32)` |
| `physics.rs` | `apply_gravity` | Y velocity bounded by terminal velocity | Random `(i32 initial_vel, u8 frames)` |
| `physics.rs` | Speed table lookup | All 9 tables: `max_walk_speed <= max_run_speed` | Enumerate all tables |
| `physics.rs` | Jump profile | Peak height deterministic for given profile and hold duration | Random `(JumpProfile, u8 hold_frames)` |
| `terrain.rs` | `probe_terrain` | Solid cell always blocks, empty cell never blocks | Random `(BlockGrid, x, y)` |
| `terrain.rs` | Cell encoding | `encode(decode(cell)) == cell` for all valid values | `u16` in `0..=0xFFFF` |
| FP16.16 | Arithmetic | Addition commutative; multiplication by 1.0 is identity | Random `(i32, i32)` pairs |

For **stateful** property testing (`proptest-state-machine` crate): model `PlayerLifecycle` as states with input-driven transitions. Generate random input sequences with precondition guards. Automatic shrinking finds the minimal failing transition sequence.

### Fuzzing guidance

Coverage-guided fuzzing finds crashes in parser and boundary code that hand-written tests miss. The mGBA and VBA-M fuzzing campaigns (AFL++) found 3,662 crashes across emulator subsystems.

| Target | Why | Tool |
|--------|-----|------|
| Level block grid parser | Untrusted binary format from extracted assets | `cargo fuzz` |
| Enemy spawn logic | Layout data can have unexpected object IDs | `cargo fuzz` |
| Collision edge cases | Grid boundaries, zero-size objects | `cargo fuzz` + proptest |
| Audio bank loading | Complex binary format | `cargo fuzz` |

Scope: high-risk subsystems at system boundaries. Pure game logic is better served by property tests and oracle comparison.

### Design by Contract

The `contracts` crate provides `#[requires]`, `#[ensures]`, and `#[invariant]` macros for executable pre/post conditions. Use on critical physics functions as living documentation of invariants:

```rust
#[requires(velocity.abs() <= MAX_SPEED)]
#[ensures(ret.abs() <= MAX_SPEED)]
fn apply_friction(velocity: i32, decel: i32) -> i32 { ... }
```

Feature-flagged: enabled in debug/test builds, disabled in release. Violations panic immediately, making the root cause obvious.

### Planned improvements

Testing infrastructure improvements are tracked in `backlog.md`: #028 (insta snapshots, done), #029 (proptest, done), #030 (local-first regression gate, done; CI deferred), #031 (perf baselines, open).

---

## Metamorphic Relations

Metamorphic testing (Chen et al. 1998; 750+ subsequent papers) defines relations between test inputs/outputs rather than specifying expected outputs per case. This sidesteps the oracle problem for properties where jgenesis comparison is impractical or unavailable.

### Core relations

| ID | Relation | Property | Test strategy |
|----|----------|----------|---------------|
| MR1 | Replay determinism | Same `DynamicScenario` replayed twice produces identical `NativeTraceFrame` sequences | Run scenario, hash all frames, run again, assert identical hash |
| MR2 | Idle convergence | Player standing still on flat ground: velocity converges to zero within N frames | Parameterise across all 9 speed tables; assert velocity reaches 0 |
| MR3 | Speed table bounds | For any `SpeedTable`, `apply_horizontal_physics` never produces velocity exceeding `max_run_speed` | proptest: random input sequences, assert `abs(vel) <= max_run_speed` |
| MR4 | Collision symmetry | Horizontal collision probe left-then-right produces same blocked result as right-then-left for symmetric geometry | Construct symmetric block grid, assert probe results mirror |
| MR5 | Damage monotonicity | Taking damage never increases HP (unless ankh/pickup in same frame) | Inject damage events across random frames, assert HP non-increasing |

### When to use metamorphic tests

- Behaviour has no oracle comparison (demo playback, RNG sequences)
- Property spans many input combinations (physics across all forms/terrains)
- Regression coverage needed without hand-crafted expected values
- Engine invariant that should hold universally (determinism, bounds, symmetry)

---

## Mutation Testing

Mutation testing injects small faults (operator swaps, constant changes, deleted statements) into source code and checks whether existing tests detect them. A surviving mutant = a test gap.

### Workflow

1. Install: `cargo install cargo-mutants`
2. Run on pure core: `cargo mutants -p wildside-core -- --lib`
3. Review survivors: each surviving mutant is a candidate for a new test
4. Target: >=80% kill rate on `physics.rs` and `terrain.rs` (pure functions)

### Cadence

| When | Scope | Action |
|------|-------|--------|
| After major fix batch (3-5 fixes) | `wildside-core` pure modules | Run, triage survivors, file test gaps as backlog items |
| Quarterly | Full `wildside-core` | Run, record kill rate in `.cache/metrics/mutation-score.txt` |

### Scope constraints

- Only run on pure-core crates (`wildside-core`). Adapter crates (`wildside-cli`, `wildside-render`) have too much I/O coupling for mutation testing to be productive.
- Exclude `// ORIGINAL_QUIRK` lines from mutation, since these are intentional "bugs".

Sources: Trail of Bits ("Use mutation testing to find the bugs your tests don't catch", 2025), `cargo-mutants` (sourcefrog).

---

## Divergence Classification

Beyond binary pass/fail, classify each sweep divergence to track fidelity trends and avoid chasing the wrong signals. Inspired by AccuOracle's four-layer filtering (NDSS '25).

### Categories

| Category | Definition | Example | Action |
|----------|-----------|---------|--------|
| Authoritative | Position, velocity, lifecycle, collision state differ | Player x off by 12px at frame 400 | Must fix, real bug |
| Advisory | Retired category: `ADVISORY_FIELDS` (sweep/mod.rs) is empty, no field is masked | Historic: score masked as "expected divergence" | Do not reintroduce; classify each diff as a bug or an evidence-backed oracle limitation |
| Accumulation | Small per-frame error compounds over time | 1px/frame camera drift | Fix root cause (rounding, FP truncation) |
| RNG | Divergence from non-deterministic or seed-dependent path | Enemy spawn timer differs | Verify seed handling; may need RNG synchronisation |
| Oracle limitation | Oracle crashes or behaves incorrectly | 8 warp-broken levels (game_mode=0x4E73) | Document; do not chase |

### Tracking

Sweep output should log per-level divergence counts by category:

```json
{
  "date": "2026-03-18",
  "levels_passed": 63,
  "levels_failed": 0,
  "levels_errored": 6,
  "by_category": {
    "authoritative": 0,
    "advisory": 42,
    "accumulation": 3,
    "rng": 0,
    "oracle_limitation": 6
  }
}
```

Each sweep run appends to `.cache/metrics/sweep-history.jsonl` for trend analysis. As of the 2026-04-03 sweep-parity hardening pass, both the per-run report and the history entry also persist a compact `field_summary` histogram plus `release_gate_focus`; each entry includes per-field category totals in `divergence_summary`, concrete BCD `level_ids`, checkpoint labels, and per-checkpoint diff counts, so operators can tell whether a recurring field belongs to advisory/oracle-limitation debt or to a blocking class without rereading every per-level record. `release_gate_focus` records the current failed release-gate step, checkpoint, failed field, the failing native/oracle value pair, whether the field matched the latest sweep histogram, and the matched histogram metadata when available, including whether the exact pair matched the precise checkpoint label or only some other checkpoint. The CLI also distinguishes mixed advisory+oracle-limitation levels as `NON-BLOCKING` instead of collapsing them into a pure oracle-limitation label, the autonomy sweep-parity lane reuses that histogram to annotate release-gate failures automatically, and `wildside sweep` prints the same release-gate-focused field summary inline with the failing scenario/checkpoint context so the output stays honest about the difference between shared field coverage and the exact concrete miss.

---

## Framewise Checksumming and Desync Detection

Borrowed from deterministic lockstep networking (Age of Empires, StarCraft): CRC32 entire game state each frame, compare checksum stream native vs oracle. First mismatch = exact frame where divergence occurs.

### Technique

1. After each frame step, compute CRC32 of serialised game state (player + enemies + camera + blocks)
2. Oracle emits matching checksums at the same frames
3. Compare streams: first mismatch narrows divergence to a single frame
4. **Subframe checksumming**: if frame-level is too coarse, checksum after each major update phase:
   - After player physics
   - After all enemies update
   - After collision passes
   - This narrows divergence to a specific subsystem

### Desync root-cause taxonomy

From RTS netcode experience, common causes of state divergence:

| Cause | Risk in this project | Mitigation |
|-------|---------------------|------------|
| Float precision | None: FP16.16 fixed-point is bitwise deterministic | Already handled |
| Iteration order | Medium: enemy list processing order | Use stable index-based iteration; never hash-order |
| RNG seed | Low: deterministic seeding per scenario | Verify seed identical in native and oracle |
| Platform differences | Low: Rust integer semantics are well-defined | Monitor with CI across platforms |

---

## GEMS/Z80 Audio-Timing Parity (`gems_frame_signature_v1`)

The audio analogue of the framewise CRC and pixel passes: a per-frame GEMS
driver state signature compared native-vs-oracle across the whole published
TAS route, reported in `report.json` (`gems_frame_comparison`), ratcheted by
`tas-gems-baseline`, and bisectable with `--frontier-kind gems`. It
industrialises what was previously artisanal — hand-derived cycle ledgers per
title-screen transfer in `wildside-frame/src/audio.rs` — and turns unknown
audio-timing debt past the hand-derived frontier into a measured, ratcheted
baseline. The monotonically advancing `first_mismatch_frame` is #409's
progress metric.

### Signature contract

Per frame, both sides emit a `wildside_core::GemsFrameSignature`:

| Field | Comparison | Notes |
|-------|------------|-------|
| `audio_ram_crc32` | strict | CRC32 of the full self-modifying 8 KiB driver RAM; raw RAM never leaves the oracle helper |
| `bank_address` | strict | Z80 bank register as 68000 base address (`bank << 15`); #409 bank steals surface here |
| `z80_pc` | strict + limitation predicate | see the X/Y-flag case below |
| `z80_wait_mclk_cycles` | strict | outstanding 49/50-master-clock banked-access wait |
| `pending_m68k_wait_cycles` | strict | 68000 wait queued at the boundary (Z80 bank steals; oracle-side also VDP stalls) |
| `status_bits` | strict | packed bus-halt / BUSREQ / RESET / VDP-owns-bus / IFF1 / HALTED |
| `z80_mclk_counter` | advisory | Z80 divider phase remainder (mod 15) |
| `ym2612_mclk_counter` | advisory | YM2612 scheduler phase remainder (mod 42) |

Deliberately not proven by this projection (declared in the report):
YM2612 register writes, sample-level audio, and intra-frame event ordering.

### Sampling instant

Pinned against the pinned jgenesis, never fitted:

- The oracle helper samples immediately after `TickEffect::FrameRendered`,
  which fires when the VDP enters scanline 232 —
  `rendered_lines_per_frame() (243) - top_border (11)`, i.e. VBlank start at
  scanline 224 plus the 8-line NTSC bottom border — quantised to the end of
  the 68000 instruction quantum that crossed the line.
- The native extractor drives the trace→command audio replayer
  (`NativeAudioDirector`) with the persistent `GemsMasterClockTimingEngine`
  as the sink and samples 8 scanlines past each VBlank frame boundary, so
  both sides sample the same raster line while the native timeline still
  advances exactly 262 scanlines per frame.
- The oracle's residual sub-instruction quantisation cannot be reproduced
  without a full 68000. That jitter shifts the divider phase remainders on
  every frame regardless of model quality, which is why `z80_mclk_counter`
  and `ym2612_mclk_counter` are captured and reported but advisory. A
  fixed-offset divergence appearing in *strict* fields still means the
  instants differ — fix the instant, never subtract a constant.

### Coverage semantics

The report declares `coverage.modelled_frames`: the edge up to which the
native per-frame GEMS timeline is *defended* as faithful. It is 59: frames
0..58 replay `EntryPoint`, both source-positioned `StopMusic` calls, the
power-on clears, the blank-display Sega-logo upload, and the first outer
`Mode_Standard` DMA/VINT pass plus three local `loc_12E6C` iterations, the
Start exit, all sixteen `Pal_FadeOut loc_28C6` loop uploads, and the
complete `loc_28EA` exit pass, and the first post-fade `loc_700` clear span. A
resumable cursor preserves the two `FrameRendered` samples inside
`sub_E13DA`, materialising each `loc_E1400` driver byte and `loc_E1406` clear
when its M68K write instruction starts, including the active one-cycle Z80-RAM
wait. Frames 16, 19, 20, 22, 25, 28, 30, and 33..58 each have one strict residual from the
documented jgenesis X/Y-flag limitation below. Signatures past the edge are
still emitted by the coarse
continuous route replayer and compared so divergence is measured, not hidden,
but never claimed as modelled. The first unmodelled boundary is frame 59 in
the continuing post-fade `loc_700` clear.
Later debt includes command injection at frame boundaries rather than true
mid-frame queue writes and every 68000↔Z80 interaction (bank steals,
FIFO/DMA bus collisions) outside the hand-derived adapters.

### Oracle limitation: undocumented X/Y flags

jgenesis preserves previous undocumented X/Y flag bits for subtract-family
instructions; real Z80 hardware (and the native core) copy bits 5/3 from the
result. GEMS `CP B` at `$01E2` first exposes this on frame 16: the flag split
shifts IRQ boundaries and changes only `$003E`'s saved-SP byte on frames 16,
19, 20, 22, 25, and 28. Frame 30 catches the same stale flag directly at
stack byte `$1A1C`. Frame 37 catches the carried phase in `CHECKTICK/VBLINT`
at `$076D` and `$1A1C..$1A1F`; frames 38/39 retain only `$003E` plus PC phase.
Frame 40 retains `$003E`, `$076D`, `$1A1E`, and the corresponding PC phase.
Frame 41 returns to only `$003E` plus PC phase.
Frame 42 samples opposite sides of the shifted VBLINT boundary and differs at
`$003E/$003F` plus PC/IFF phase.
Frames 43/44 differ only at `SBPTACC` `$076C` plus PC phase. Frame 45 differs
only at `SBPTACC` `$076C/$076D`, stack byte `$1A1E`, and PC phase.
Frame 46 differs only at `SBPTACC` `$076C/$076D`, stack bytes `$1A1E/$1A1F`,
and PC phase.
Frame 47 differs only at saved-SP `$003E`, `SBPTACC` `$076C/$076D`, stack
bytes `$1A1C..$1A1F`, and PC phase.
Frame 48 differs only at saved-SP `$003E`, `SBPTACC` `$076C`, and PC phase.
Frame 49 differs only at `SBPTACC` `$076C` and PC phase.
Frame 50 differs only at `SBPTACC` `$076C/$076D`, stack byte `$1A1E`, and PC
phase.
Frame 51 differs only at `SBPTACC` `$076C/$076D`, stack bytes `$1A1E/$1A1F`,
and PC phase.
Frame 52 differs only at saved-SP `$003E`, `SBPTACC` `$076C/$076D`, stack
bytes `$1A1C..$1A1F`, and PC phase.
Frame 53 differs only at `SBPTACC` `$076C` and PC phase.
Frame 54 differs only at `SBPTACC` `$076C/$076D`, stack bytes
`$1A1C..$1A1E`, and PC phase.
Frame 55 differs only at saved-SP `$003E`, `SBPTACC` `$076C/$076D`, stack
bytes `$1A1E/$1A1F`, and PC phase.
Frame 56 differs only at `SBPTACC` `$076C`, stack byte `$1A1E`, and PC phase.
Frame 57 differs only at `SBPTACC` `$076C` and PC phase.
Frame 58 differs only at saved-SP `$003E`, `SBPTACC` `$076C`, stack bytes
`$1A1E/$1A1F`, and PC phase.
Later,
`SUB 1` at `$07B3` leaves the oracle at `$02AA` where hardware reaches `$02AE`.
A residual reclassifies as `oracle_limitation` only for those exact frame/CRC
or PC pairs when every other strict field is byte-equal; anything broader
stays authoritative.

### Gates

- `verify-tas --compare-gems` computes and reports the comparison.
- `tas-gems-baseline` ratchets `full_expected_gems_frame_comparison` in
  `tests/tas_frontier.json` (Match / Improvement exit 3 / Regression exit 1;
  ranked metrics: `mismatch_count` lower, `first_mismatch_frame` later).
  Wired into `verify-tas-full`, never `verify-slice-fast`.
- `verify-tas --bisect --frontier-kind gems` localises the first signature
  mismatch from one full run (per-frame data needs no binary search).

---

## Frame-Level Bisection

Binary search within a level to find the exact frame where divergence begins. O(log N) vs O(N) sequential scan.

### Workflow

1. Identify a level + scenario with known divergence at frame F_bad
2. Confirm frames before some F_good match oracle
3. Checkpoint at midpoint M = (F_good + F_bad) / 2
4. Compare state at M against oracle
5. If M matches, divergence is in [M, F_bad]; recurse
6. If M diverges, divergence is in [F_good, M]; recurse
7. Continue until single frame identified

### Output

"Divergence begins at frame 47: `player.y` differs by 3px from oracle. Prior frame (46) matches."

### Combining with git bisect

For regression hunting: `git bisect run` narrows the commit, frame bisection narrows the frame within that commit. Together they pinpoint both when and where the bug was introduced.

---

## Fidelity Metrics and Tracking

Track continuous fidelity indicators to detect regressions, measure progress, and maintain confidence.

### Metrics

| Metric | Source | Target | Location |
|--------|--------|--------|----------|
| Sweep pass rate | `sweep --tier state` | 63/69 to 69/69 | `.cache/metrics/sweep-history.jsonl` |
| Test count | `cargo test -- --list 2>&1 \| wc -l` | Monotonically increasing | `.cache/metrics/test-count.txt` |
| Mutation kill rate | `cargo mutants -p wildside-core` | >=80% on physics.rs, terrain.rs | `.cache/metrics/mutation-score.txt` |
| Scenario count | `DynamicScenario` definitions | >=10 per edge-case level | `.cache/metrics/scenario-count.txt` |
| Divergence by category | Sweep JSON output | Authoritative = 0 | `.cache/metrics/sweep-divergences.json` |

### Per-level fidelity tiers

Inspired by ProtonDB and Dolphin compatibility databases:

| Tier | Definition |
|------|-----------|
| Flawless | Oracle-identical position, state, and frames |
| Accurate | Oracle-identical state, within 6px position tolerance |
| Playable | Completes level, visible divergences within acceptable bounds |
| Issues | Major visual or gameplay divergences |
| Broken | Crashes or unplayable (includes oracle-limitation levels) |

### Per-enemy validation tracking

| Archetype | Families | Oracle-verified scenarios | Status |
|-----------|----------|--------------------------|--------|
| Pacer | Goat, Armadillo, Tank, Crab, Scorpion, Driller | (count per family) | Partial |
| Hopper | MiniHoppingSkull, BigHoppingSkull | (count) | Partial |
| Flyer | Cloud, DragonFlying | (count) | Partial |
| Bouncer | Sphere, Fireball | (count) | Partial |
| Stationary | Diamond, Archer, EmoRock | (count) | Partial |
| Complex | Dragon, Ninja, Orca, Hand, Robot, etc. | (count) | Partial |
| Inert | (stubs) | N/A | N/A |

Update as scenarios are added. Target: >=2 scenarios per family (idle + one interaction).

### Collection

Run `mise run collect-metrics` after each sweep to update `.cache/metrics/`. CI automation tracked in backlog.

---

## Regression Discipline

Every fix MUST become a permanent regression test. Chocolate Doom's entire Compet-N demo archive functions as a regression suite: thousands of real player demos that would desync if any gameplay logic changed. Our equivalent: probe scenarios + sweep + unit tests.

### Test type by fix category

| Fix category | Regression test type |
|-------------|---------------------|
| Physics fix | `timing_regression_*` unit test |
| Enemy behaviour fix | `test_enemy_*` + probe scenario |
| Visual fix | Frame capture golden master |
| State machine fix | Gameplay scenario with state checkpoints |
| Collision fix | Targeted scenario + sweep verification |

### Gates

- **Pre-commit**: `mise run verify-slice-fast` (timing + gameplay regression tests)
- **Pre-merge**: full sweep (`sweep --tier state` on all BCD levels)
- **Post-playtest**: triage new bugs into backlog, write regression tests for each
- **Post-fix-batch**: run `cargo mutants -p wildside-core -- --lib` to check for test gaps

---

## Cross-Referencing Protocol

### Source resolution

| Situation | Primary source | Secondary source | If they disagree |
|-----------|---------------|-----------------|-----------------|
| Unknown behaviour | Oracle (jgenesis) | Disassembly (`kid.asm`) | Investigate both; oracle is runtime truth, ASM is specification |
| Oracle crash/error | Disassembly | RAM layout docs | Oracle is fallible; trust ASM if oracle fails (8 warp-broken levels) |
| Complex formula | ASM + `variables.asm` | Oracle checkpoint comparison | Derive formula from ASM, validate against oracle output |
| Hardware effect | Kid ASM first | Sega Retro Mega Drive docs | Kid ASM shows what the game does; hardware docs show what is possible |
| Undocumented register | M68k instruction docs | Test with oracle | Reference docs are specification; oracle is empirical validation |

### Resolving disagreements

When sources conflict:

1. **Oracle vs ASM**: oracle is runtime truth (what actually happens), ASM is specification (what should happen). If oracle shows behaviour X but ASM seems to say Y, re-read the ASM more carefully; subtle addressing modes, condition flags, or interrupt timing may explain the difference.
2. **Native vs both**: native is wrong. Fix native.
3. **All three disagree**: escalate to deeper investigation, Mega Drive hardware docs, M68k reference manual, or web research. Document the ambiguity.

### Document cross-reference map

| Document | Cites methodology.md | Role |
|----------|---------------------|------|
| `CLAUDE.md` | Yes | Operational summary; points here for depth |
| `reference-index.md` | Yes (Methodology section) | Quick source lookup |
| `research-map.md` | Yes (Validation section) | Frontier tracking |
| `backlog.md` | Yes (header) | Bug tracker; follows Investigation Playbook |
| `acceleration-strategy.md` | Yes (Validation methodology) | Development strategy |

Rule: any doc that describes validation, investigation, or testing should cite this document as the canonical reference rather than restating the methodology.

---

## Common Pitfalls

Lessons learned from bsnes, Chocolate Doom, Julius, OpenMW, devilutionX, and project experience:

| Pitfall | What goes wrong | How to avoid |
|---------|----------------|--------------|
| **Game-specific fixes** | Fix one level, break another; structural issues go undiagnosed | Engine-level fixes only; full sweep after every change (bsnes) |
| **Guessing behaviour** | Subtle incorrectness compounds over time | Probe oracle first; never infer from "what seems right" (bsnes: 80hrs for one Speedy Gonzales bus conflict rather than guessing) |
| **Fixing original bugs** | Breaks compatibility with original data/replays | Preserve bugs with `ORIGINAL_QUIRK` marker (Julius, OpenMW, devilutionX) |
| **Skipping regression tests** | "Quick fix" introduces new bugs discovered later | Every fix becomes a permanent test; no exceptions (Chocolate Doom statcheck) |
| **Optimising before understanding** | Speed hacks obscure correct behaviour | Get it right first, optimise later with benchmarks (bsnes) |
| **Non-deterministic engine** | Replay testing becomes impossible | Fixed RNG, deterministic frame stepping, no system-time dependencies (Chocolate Doom: M_Random seed bug broke cross-session determinism) |
| **Advisory field confusion** | Blanket field masks hide real bugs behind "expected divergence" labels | `ADVISORY_FIELDS` stays empty; every diff is a bug or an evidence-backed `oracle_limitation` matcher (see sweep/mod.rs rationale comment) |
| **Code churn breaking replays** | Refactors silently change execution order | Snapshot tests detect state drift; replay scenarios catch desync (Chocolate Doom) |
| **Whack-a-mole emulation** | Each fix creates new regressions in a cycle | Understand the root cause at hardware/engine level before fixing symptoms (bsnes) |
| **Conflating compatibility with accuracy** | "It works" masks underlying incorrectness | Accuracy naturally enables compatibility; the reverse is not true (bsnes) |
| **Untested test suite** | Tests pass but miss real bugs (false confidence) | Mutation testing quarterly; surviving mutants = test gaps (Trail of Bits) |
| **Unclassified divergences** | Chasing oracle-limitation noise, ignoring accumulation drift | Classify every divergence by category; trend by category over time |
| **Coarse divergence detection** | "Something's wrong in level 23" but no precision | Framewise checksums + frame-level bisection to pinpoint exact frame and field |
| **Dead-coded helper drift** | Helper implements ASM routine and has tests, but no production caller exists; the broken behaviour the helper would fix ships anyway | Grep for `#[cfg_attr(not(test), allow(dead_code))]` whenever a user reports a behaviour gap. Examples found 2026-05-19: `kid_airborne_visual_sub_B270` (Red Stealth airborne slash, #153) and `drips_ceiling_gate_loc_333_f6` (Drips slime ceiling attach, #155) both shipped untested in production. Pattern: helper is faithful to ASM and unit-tested, but no `step_*` or runtime call invokes it. Fix order: state field, runtime call, renderer plumbing, integration test |
| **Tautological reference-port tests** | `#[cfg(test)] pub(crate) fn` ports an ASM routine and the test asserts only that helper's own output; coverage rises but the shipped path never executes the routine | Treat this as test-only fidelity debt, not behaviour proof. Prefer wiring the port into production, or write a differential/integration test that drives the production path and compares against oracle/ASM-derived expectations. Grandfather existing debt with `asm-coverage --check-no-new-test-only`; do not add new labels without review. |

---

## References

For a comparison of this project's approach to peer projects (SM64 decomp, devilutionX, OpenMW, etc.) and an explanation of why we verify by runtime oracle rather than binary matching, see `docs/source-port/project-taxonomy.md`.

Projects and papers studied for methodology:

| Project | Domain | Key technique | Source |
|---------|--------|--------------|--------|
| bsnes/higan | SNES (all) | Cycle-accurate emulation, test ROMs, code as documentation | bsnes.org |
| Chocolate Doom | Doom | Statcheck demo regression (4000+ demos), deterministic replay | chocolate-doom.org |
| Julius | Caesar III | 100% savegame compatibility, bidirectional validation | github.com/bvschaik/julius |
| OpenMW | Morrowind | Bug compatibility tiers, empirical formula reverse-engineering | openmw.org |
| devilutionX | Diablo | Function-by-function binary matching, debug symbol recovery | github.com/diasurgical/devilution |
| SM64 decomp | Super Mario 64 | Matching decompilation (byte-for-byte binary identity) | github.com/n64decomp/sm64 |
| Zelda OoT decomp | Ocarina of Time | Incremental matching with progress tracking, non-matching directory | github.com/zeldaret/oot |
| MAME | Arcade (all) | Test ROM acid tests, pixel-perfect frame regression | mamedev.org |
| AoE lockstep | RTS netcode | Framewise checksumming, desync detection taxonomy | Bettner & Terrano (GDC 2001) |
| Chen et al. 1998 | Testing theory | Metamorphic relations for oracle-free testing | DOI:10.1093/comjnl/41.6.386 |
| AccuOracle | Oracle comparison | Divergence classification, four-layer filtering | NDSS '25 |
| cargo-mutants | Rust testing | Mutation testing for Rust crates | github.com/sourcefrog/cargo-mutants |
| proptest-state-machine | Rust testing | Stateful property testing with auto-shrinking | crates.io/crates/proptest-state-machine |
| rr | Debugging | Deterministic record-replay with reverse execution | rr-project.org |
| ProtonDB | Game compat | Per-game fidelity tier ratings (Platinum to Borked) | protondb.com |
| Blargg test ROMs | NES emulation | Isolated component tests with deterministic checksums | nesdev.org |
