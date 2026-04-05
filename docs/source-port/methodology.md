# Source-Port Methodology

Detailed reference for reverse-engineering discipline, testing strategy, and fidelity principles.
`CLAUDE.md` has the operational summary; this document explains the *why* and *how* in depth.

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

bsnes (byuu): "Accuracy is not 'does it play Zelda?' — playing games is a side effect of accuracy." Fix the engine, never the game. Accurate reimplementation naturally enables compatibility; the reverse is not true.

### Bugs are features

Original game bugs must be faithfully reproduced:
- **Julius** (Caesar III): 100% bidirectional savegame compatibility — any logic deviation breaks saves. Augustus (fork) proved this by breaking compatibility when it added gameplay changes.
- **OpenMW** (Morrowind): intentional bug preservation for mod compatibility. Attack angle uses flawed dot product math in the original; OpenMW reproduces the flaw.
- **Chocolate Doom**: demo sync requires frame-perfect reproduction of original behaviour including bugs. The Lost Soul bouncing bug fix in Ultimate Doom broke all earlier Doom II demos.
- **devilutionX** (Diablo): "Don't fix bugs found during RE — reproduce them exactly to validate correctness."

Mark preserved bugs with `// ORIGINAL_QUIRK: <description>` so they're visible and intentional.

### Progressive fidelity

Get it working first, get it accurate second — but track the gap. Approximate implementations are acceptable stepping stones when marked `// TODO(fidelity): <description>` and tracked in the backlog. Never ship a guess as if it were verified.

### Code as documentation

Implementation should be readable as a specification of original behaviour. bsnes: speed hacks and convenience features "destroy code's ability to serve as verifiable hardware documentation." Avoid clever abstractions that obscure what the original hardware/software actually did.

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

This mirrors byuu's investigation loop (identify → capture → hypothesise → probe → verify → fix → sweep):

1. **Observe**: capture frames or run scenario showing the problem
2. **Classify**: which root cause category? (Use taxonomy table)
3. **Probe oracle**: run exact same scenario against jgenesis, capture state checkpoints at same frames
4. **Read disassembly**: follow source precedence — `kid.asm` → `variables.asm` → `doc/formats.txt`
5. **Cross-reference**: native logic vs ASM vs oracle data. All three must agree:
   - ASM + oracle agree, native differs → **native bug**
   - ASM + native agree, oracle differs → **oracle limitation** (rare but possible; 8 warp-broken levels are examples)
   - All three disagree → **deeper investigation needed** (hardware docs, web research)
6. **Write probe scenario**: targeted `DynamicScenario` exercising exactly this behaviour (like bsnes test ROMs — test ROMs must exercise specific undocumented behaviours with observable outcomes)
7. **Fix**: implement smallest correct change
8. **Verify**: probe scenario passes + full sweep passes + visual before/after comparison

### Trace comparison techniques

When investigating divergences between native and oracle:

| Technique | When to use | How |
|-----------|------------|-----|
| Field-level state diff | Any divergence | Instead of flat "MISMATCH", report per-field delta with tolerance. E.g. "player.x: +4px (within tolerance), player.vx: +2 (ERROR)" |
| Framewise CRC | Narrowing divergence origin | CRC32 entire game state each frame; first mismatch = exact divergence frame |
| Subframe checksumming | Divergence isolated to one frame | Checksum after each update phase (player → enemies → collision) to narrow to subsystem |
| Frame-level bisection | Large frame range to search | Binary search: checkpoint at midpoint, recurse on divergent half. O(log N) vs O(N) |
| DTW trace alignment | Oracle and native differ in frame count | Dynamic Time Warping aligns traces despite timing drift (Sakoe-Chiba band ±2 frames) |

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

### Architecture layers → test types

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
- `release-gate --target 1p-accurate` answers "do boot/title, gameplay compare, frame-oracle, audio-oracle, and state-sweep all stay green together?"

On 2026-04-05 those lanes were intentionally not equivalent:

- `cargo run --release -p kidcli -- sweep --levels 0-104 --tier state` passed all 69 BCD levels again with only advisory/oracle-limitation classifications (`.cache/sweep-runs/1775372033/report.json`), and the refreshed autonomy handoff now cites that report directly while preserving the same `advisory=191`, `oracle_limitation=668`, `failed=0`, `errors=0` classification shape. Each `field_summary` entry now also carries its own `divergence_summary`, concrete BCD `level_ids`, the unique checkpoint labels where that field diverged, and a `checkpoint_counts` breakdown showing how many diffs landed at each checkpoint. Both the sweep report and the append-only history entry now persist a parallel `release_gate_focus` section that captures the current release-gate scenario/checkpoint, whether the failing field matched the state sweep histogram, whether that histogram matched the same checkpoint family, the exact native/oracle value pair reported by the failing gate, which BCD levels carried that exact pair, whether the release-gate scenario's own level is among them, and whether that exact pair appeared at the same exact checkpoint label or only elsewhere. A field like `game_mode` can therefore be read as "oracle-limited/advisory in sweep", "blocking in sweep", "same field class but different concrete miss", or "same value pair but different exact level/checkpoint/family" without rereading every per-level diff or replaying CLI stdout. The CLI still labels levels carrying both categories as `NON-BLOCKING` so operators can see mixed debt directly in the sweep output, and it echoes the latest failing release-gate fields against that same histogram so the delegated lane stays visible during a raw sweep run.
- `mise run release-gate` now fails at `pause_resume_level1`, `level_settle@719`, where `game_mode` diverges (`native 0x0008`, `oracle 0x000C`); the latest persisted gate artefact is `.cache/release-gate/1p-accurate/latest.json`, now at timestamp `1775372233`, and the matching gate report is `.cache/release-gate/1p-accurate/1775372233/report.json`.
- Automation in this lane should report the live failing artefact detail, not static capability prose. The autonomy release-gate signal should prefer the latest persisted `release_gate_focus.summary`, falling back to the flatter state-sweep `field_summary` only when the richer correlation is absent, so the delegated sweep-parity lane keeps the exact pair/checkpoint evidence attached to the failing gate step instead of restating only the broad `game_mode` histogram.
- The operational refresh point is the release-gate write itself: after `mise run release-gate` updates `.cache/release-gate/<target>/latest.json`, it should immediately regenerate `.cache/autonomy/source-port-frontier/next-task.json` and `.cache/autonomy/source-port-frontier/next-agent.md` from the latest persisted gate + sweep artefacts. Run the delegated trio serially in that order; a newer sweep report paired with an older `latest.json` means the lane is only half-refreshed, not that the autonomy code picked the wrong evidence.
- Treat the newest persisted state-tier sweep report in `.cache/sweep-runs/<timestamp>/report.json` as equivalent evidence to the mirrored `sweep-history.jsonl` entry when refreshing that handoff. If the report exists but the history append has not landed yet, the autonomy layer should still consume the fresher report so the release-gate correlation does not freeze on an older timestamp.
- When the closed-frontier loop regenerates `.cache/autonomy/source-port-frontier/next-task.json` and `.cache/autonomy/source-port-frontier/next-agent.md`, the sweep-parity task should embed both the failing release-gate step detail and the matching state-sweep correlation as first-class evidence items. `refresh_evidence` is not complete unless the durable handoff quotes the same persisted gate/focus pair, plus the gate/sweep timestamps and report paths, that operators would otherwise have to reconstruct by hand from `latest.json` plus `.cache/metrics/sweep-history.jsonl`.
- When a stalled sweep-parity loop emits recovery metadata, the durable handoff should repeat that metadata in prose, not just in the JSON package. Include the recovery mode, recovery step, repeated-stall count when present, and the pivot reason so the next operator can see why the lane narrowed to evidence refresh without reopening the package diff.
- When multiple release-gate failures share the same field name, correlate the latest state-sweep evidence by the full release-gate focus key first: `target`, `step_name`, `checkpoint`, `field`, and the exact native/oracle value pair when available. Only fall back to field-only correlation for older sweep-history entries that do not yet persist the richer metadata. Otherwise automation can attach a true statement about the wrong failing step, which is worse than surfacing no correlation at all.
- The sweep summary must carry the release-gate scenario/checkpoint alongside that field histogram (`pause_resume_level1 @ level_settle@719` in the current run), and both the report JSON and `.cache/metrics/sweep-history.jsonl` should persist the same correlation via `release_gate_focus`. That correlation now also records the gate scenario's own BCD level, whether the matching histogram includes it, whether the exact failing value pair reappears on that same level, whether the sweep histogram ever saw the same exact checkpoint label, whether it at least saw the same checkpoint family, and the concrete checkpoint distribution that did appear. The latest run keeps `scenario_level_matched_in_state_sweep=true`, `checkpoint_matched_in_state_sweep=false`, and `checkpoint_family_matched_in_state_sweep=true`, with `matched_checkpoints=[idle_tail=12, level_settle=1]`; the exact-pair flag is also true, but `exact_value_pair_levels=11` and `exact_value_pair_scenario_level_matched_in_state_sweep=false`, so that pair only reappears on the other 11 levels (`0x09`, `0x32`, `0x33`, `0x45`, `0x48`, `0x50`, `0x54`, `0x57`, `0x61`, `0x62`, `0x63`) and only at `idle_tail`. The important constraint is therefore narrower: the idle state sweep can share the `game_mode` field name, and even the `0x0008 -> 0x000c` value pair, without covering the concrete `level_settle@719` gameplay-compare miss on the same level, checkpoint, or checkpoint family. This is still evidence about field class and checkpoint-family coverage only; it does not prove the release-gate failure is already explained by known sweep debt.
- Use the sweep `level_ids` and checkpoint labels as the next lookup before changing code: if the failing release-gate field already maps to the same known BCD coverage and checkpoint family, harden the verification story first; only reopen gameplay work when the field or category escapes that known sweep envelope.

Treat that shape as a release-gate verification blocker, not as evidence to reopen TAS work or to claim the sweep lane regressed.

### TAS (Tool-Assisted Speedrun) fidelity testing

TAS recordings are deterministic per-frame button sequences from real full-game playthroughs. They exercise code paths that synthetic scenarios don't: level transitions, helmet pickups, boss fights, warps, and full boot-to-completion.

**Workflow:**

1. Obtain a BK2 file (BizHawk TAS format) from [TASVideos.org](https://tasvideos.org/UserFiles/Game/180)
2. Import: `mise run import-tas path/to/recording.bk2`
3. Quick test (first 1000 frames): `cargo run -p kidcli -- verify-tas --scenario tests/data/tas/<slug>/scenario.json --frame-end 1000`
4. Full run: `mise run verify-tas tests/data/tas/<slug>/scenario.json`

**How it works:**

- BK2 is a ZIP containing `Header.txt` (system metadata) and `Input Log.txt` (per-frame button state)
- `import-tas` parses the BK2 and converts to `DynamicScenario` JSON with `warp_to_level: None` (power-on boot)
- Canonical imported TAS scenarios live under `tests/data/tas/<slug>/scenario.json` so clones on other machines can replay and inspect them without rebuilding local cache state
- `verify-tas` runs the scenario through both native and oracle from frame 0
- Level transitions are auto-detected from the native trace and injected as additional checkpoints
- The oracle scenario JSON includes transition checkpoints, giving diagnostic data at level boundaries
- `--frame-start`/`--frame-end` slicing enables iterative debugging of specific sections
- `verify-tas` now reports boot diagnostics for the first authoritative mismatch (phase, elapsed, title readiness, menu slot when available)
- Trimmed 250-frame boot-slice fixtures live in `crates/kidcli/tests/fixtures/tas/` so early boot regressions stay reproducible without cache-local BK2 imports

**Available recordings (Kid Chameleon):**

| Recording | Format | Frames | Use case |
|-----------|--------|--------|----------|
| Any% by CaseyTAS (10:30) — TASVideos #6944 | BK2 | ~37k | Warps, transitions, bosses |
| KC Any% TAS 2024 — TASVideos user files | BK2 | ~4,900 | Quick smoke test (warp code) |

### Semantic case bundles

Long deterministic replays stay as the regression lane. Discovery work should prefer
small reduced case bundles derived from replay windows or native recordings.

Phase-1 bundle workflow:

1. `kidcli case derive` creates a per-frame semantic bundle for a chosen window
2. `kidcli case branch` generates short seeded continuations with a human-like action grammar
3. `kidcli case verify` compares native vs oracle semantic state for one bundle
4. `kidcli case reduce` shrinks a failing continuation while preserving the first semantic failure
5. `kidcli case summarise` refreshes the agent-facing markdown summary

Each case bundle should answer:

- what checkpoint it starts from
- what the first bad semantic frame/field is
- which fields still match
- which subsystem is the likeliest owner

Autonomous execution should consume these bundles through the repo-native source-port dispatcher rather than by repeating stale closed-frontier handoffs:

- `cargo run -p kidcli -- autonomous-loop --goal source-port-frontier` remains the operator entrypoint
- regenerate that source-port package at the start of every loop run; do not reuse an old `next-task.json`
- while TAS work is active it packages the live frontier
- once TAS is closed it auto-delegates to `case-queue` first
- recording-backed cases outrank generic replay slices
- if no failing reduced case is ready, uncovered recordings should become the next gameplay derivation task before any sweep-only lane
- only `autonomy-executable` cases may patch code without a human re-triage
- only when neither a reduced case nor a recording-derived gameplay task exists should the loop pivot to sweep/verification hardening
- use `mise run source-port-rl-run` for `rl`/Codex operator runs so the launcher regenerates the live package and pipes the generated prompt directly to `cxys`; use `mise run source-port-rl-prompt` only to inspect the generated package by hand
- if tracked git changes already exist when the launcher starts, the loop should treat them as the highest-priority recovery task instead of rejecting the run; verify/commit or intentionally discard that coherent unit before switching lanes
- if the same delegated fingerprint stalls repeatedly, emit an explicit recovery step (`finish_dirty_work`, `reverify_current_lane`, `pivot_to_next_lane`, or `refresh_evidence`) rather than replaying the same prompt; repeated sweep stalls should pivot back towards gameplay derivation when recordings still exist

### Decision tree

| Situation | Test type | Rationale |
|-----------|-----------|-----------|
| New engine behaviour | Probe scenario + oracle comparison | Like bsnes test ROMs: exercise specific behaviour, compare to ground truth |
| Fixed bug | Timing/gameplay regression test (`verify-slice-fast` gate) | Like Chocolate Doom statcheck: every fix becomes a permanent regression test |
| Pure function, large input space | Property-based test (proptest) | Speed tables, slope formulas, gravity curves — verify properties hold across range |
| Stateful system, complex transitions | Stateful property test (proptest-state-machine) | Model lifecycle as states, generate random input sequences, auto-shrink failures |
| Complex state hard to assert field-by-field | Snapshot test (insta) | NativeTraceFrame at checkpoint — detect unexpected state changes between commits |
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

Testing infrastructure improvements are tracked in `backlog.md`: #028 (insta snapshots — done), #029 (proptest — done), #030 (local-first regression gate — done; CI deferred), #031 (perf baselines — open).

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
2. Run on pure core: `cargo mutants -p kid-core -- --lib`
3. Review survivors: each surviving mutant is a candidate for a new test
4. Target: ≥80% kill rate on `physics.rs` and `terrain.rs` (pure functions)

### Cadence

| When | Scope | Action |
|------|-------|--------|
| After major fix batch (3-5 fixes) | `kid-core` pure modules | Run, triage survivors, file test gaps as backlog items |
| Quarterly | Full `kid-core` | Run, record kill rate in `.cache/metrics/mutation-score.txt` |

### Scope constraints

- Only run on pure-core crates (`kid-core`). Adapter crates (`kidcli`, `kid-render`) have too much I/O coupling for mutation testing to be productive.
- Exclude `// ORIGINAL_QUIRK` lines from mutation — these are intentional "bugs".

Sources: Trail of Bits ("Use mutation testing to find the bugs your tests don't catch", 2025), `cargo-mutants` (sourcefrog).

---

## Divergence Classification

Beyond binary pass/fail, classify each sweep divergence to track fidelity trends and avoid chasing the wrong signals. Inspired by AccuOracle's four-layer filtering (NDSS '25).

### Categories

| Category | Definition | Example | Action |
|----------|-----------|---------|--------|
| Authoritative | Position, velocity, lifecycle, collision state differ | Player x off by 12px at frame 400 | Must fix — real bug |
| Advisory | Score, lives, diamonds, object count, murder wall x | Score 1200 vs 1400 | Masked in sweep; tracked separately |
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

Each sweep run appends to `.cache/metrics/sweep-history.jsonl` for trend analysis. As of the 2026-04-03 sweep-parity hardening pass, both the per-run report and the history entry also persist a compact `field_summary` histogram plus `release_gate_focus`; each entry includes per-field category totals in `divergence_summary`, concrete BCD `level_ids`, checkpoint labels, and per-checkpoint diff counts, so operators can tell whether a recurring field belongs to advisory/oracle-limitation debt or to a blocking class without rereading every per-level record. `release_gate_focus` records the current failed release-gate step, checkpoint, failed field, the failing native/oracle value pair, whether the field matched the latest sweep histogram, and the matched histogram metadata when available, including whether the exact pair matched the precise checkpoint label or only some other checkpoint. The CLI also distinguishes mixed advisory+oracle-limitation levels as `NON-BLOCKING` instead of collapsing them into a pure oracle-limitation label, the autonomy sweep-parity lane reuses that histogram to annotate release-gate failures automatically, and `kidcli sweep` now prints the same release-gate-focused field summary inline with the failing scenario/checkpoint context so the output stays honest about the difference between shared field coverage and the exact concrete miss.

---

## Framewise Checksumming & Desync Detection

Borrowed from deterministic lockstep networking (Age of Empires, StarCraft): CRC32 entire game state each frame, compare checksum stream native vs oracle. First mismatch = exact frame where divergence occurs.

### Technique

1. After each frame step, compute CRC32 of serialised game state (player + enemies + camera + blocks)
2. Oracle emits matching checksums at same frames
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
| Float precision | None — FP16.16 fixed-point is bitwise deterministic | Already handled |
| Iteration order | Medium — enemy list processing order | Use stable index-based iteration; never hash-order |
| RNG seed | Low — deterministic seeding per scenario | Verify seed identical in native and oracle |
| Platform differences | Low — Rust integer semantics are well-defined | Monitor with CI across platforms |

---

## Frame-Level Bisection

Binary search within a level to find the exact frame where divergence begins. O(log N) vs O(N) sequential scan.

### Workflow

1. Identify a level + scenario with known divergence at frame F_bad
2. Confirm frames before some F_good match oracle
3. Checkpoint at midpoint M = (F_good + F_bad) / 2
4. Compare state at M against oracle
5. If M matches → divergence is in [M, F_bad]; recurse
6. If M diverges → divergence is in [F_good, M]; recurse
7. Continue until single frame identified

### Output

"Divergence begins at frame 47: `player.y` differs by 3px from oracle. Prior frame (46) matches."

### Combining with git bisect

For regression hunting: `git bisect run` narrows the commit, frame bisection narrows the frame within that commit. Together they pinpoint both *when* and *where* the bug was introduced.

---

## Fidelity Metrics & Tracking

Track continuous fidelity indicators to detect regressions, measure progress, and maintain confidence.

### Metrics

| Metric | Source | Target | Location |
|--------|--------|--------|----------|
| Sweep pass rate | `sweep --tier state` | 63/69 → 69/69 | `.cache/metrics/sweep-history.jsonl` |
| Test count | `cargo test -- --list 2>&1 \| wc -l` | Monotonically increasing | `.cache/metrics/test-count.txt` |
| Mutation kill rate | `cargo mutants -p kid-core` | ≥80% on physics.rs, terrain.rs | `.cache/metrics/mutation-score.txt` |
| Scenario count | `DynamicScenario` definitions | ≥10 per edge-case level | `.cache/metrics/scenario-count.txt` |
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

Update as scenarios are added. Target: ≥2 scenarios per family (idle + one interaction).

### Collection

Run `mise run collect-metrics` after each sweep to update `.cache/metrics/`. CI automation tracked in backlog.

---

## Regression Discipline

Every fix MUST become a permanent regression test. Chocolate Doom's entire Compet-N demo archive functions as a regression suite — thousands of real player demos that would desync if any gameplay logic changed. Our equivalent: probe scenarios + sweep + unit tests.

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
- **Post-fix-batch**: run `cargo mutants -p kid-core -- --lib` to check for test gaps

---

## Cross-Referencing Protocol

### Source resolution

| Situation | Primary source | Secondary source | If they disagree |
|-----------|---------------|-----------------|-----------------|
| Unknown behaviour | Oracle (jgenesis) | Disassembly (`kid.asm`) | Investigate both; oracle is runtime truth, ASM is specification |
| Oracle crash/error | Disassembly | RAM layout docs | Oracle is fallible; trust ASM if oracle fails (8 warp-broken levels) |
| Complex formula | ASM + `variables.asm` | Oracle checkpoint comparison | Derive formula from ASM, validate against oracle output |
| Hardware effect | Kid ASM first | Sega Retro Mega Drive docs | Kid ASM shows what the game actually does; hardware docs show what's possible |
| Undocumented register | M68k instruction docs | Test with oracle | Reference docs are specification; oracle is empirical validation |

### Resolving disagreements

When sources conflict:
1. **Oracle vs ASM**: oracle is runtime truth (what actually happens), ASM is specification (what should happen). If oracle shows behaviour X but ASM seems to say Y, re-read the ASM more carefully — subtle addressing modes, condition flags, or interrupt timing may explain the difference.
2. **Native vs both**: native is wrong. Fix native.
3. **All three disagree**: escalate to deeper investigation — Mega Drive hardware docs, M68k reference manual, or web research. Document the ambiguity.

### Document cross-reference map

| Document | Cites methodology.md | Role |
|----------|---------------------|------|
| `CLAUDE.md` | Yes (line 192) | Operational summary; points here for depth |
| `reference-index.md` | Yes (Methodology section) | Quick source lookup |
| `research-map.md` | Yes (Validation section) | Frontier tracking |
| `backlog.md` | Yes (header) | Bug tracker; follows Investigation Playbook |
| `roadmap.md` | Yes (line 5) | Milestone tracker |
| `acceleration-strategy.md` | Yes (Track 6 section) | Development strategy |

Rule: any doc that describes validation, investigation, or testing should cite this document as the canonical reference rather than restating the methodology.

---

## Common Pitfalls

Lessons learned from bsnes, Chocolate Doom, Julius, OpenMW, devilutionX, and project experience:

| Pitfall | What goes wrong | How to avoid |
|---------|----------------|--------------|
| **Game-specific fixes** | Fix one level, break another; structural issues go undiagnosed | Engine-level fixes only; full sweep after every change (bsnes) |
| **Guessing behaviour** | Subtle incorrectness compounds over time | Probe oracle first; never infer from "what seems right" (bsnes: 80hrs for one Speedy Gonzales bus conflict rather than guessing) |
| **Fixing original bugs** | Breaks compatibility with original data/replays | Preserve bugs with `ORIGINAL_QUIRK` marker (Julius, OpenMW, devilutionX) |
| **Skipping regression tests** | "Quick fix" introduces new bugs discovered later | Every fix → permanent test; no exceptions (Chocolate Doom statcheck) |
| **Optimising before understanding** | Speed hacks obscure correct behaviour | Get it right first, optimise later with benchmarks (bsnes) |
| **Non-deterministic engine** | Replay testing becomes impossible | Fixed RNG, deterministic frame stepping, no system-time dependencies (Chocolate Doom: M_Random seed bug broke cross-session determinism) |
| **Advisory field confusion** | Chasing divergences that are expected | Clear advisory vs authoritative distinction; mask advisory in sweep |
| **Code churn breaking replays** | Refactors silently change execution order | Snapshot tests detect state drift; replay scenarios catch desync (Chocolate Doom) |
| **Whack-a-mole emulation** | Each fix creates new regressions in a cycle | Understand the root cause at hardware/engine level before fixing symptoms (bsnes) |
| **Conflating compatibility with accuracy** | "It works" masks underlying incorrectness | Accuracy naturally enables compatibility; the reverse is not true (bsnes) |
| **Untested test suite** | Tests pass but miss real bugs (false confidence) | Mutation testing quarterly; surviving mutants = test gaps (Trail of Bits) |
| **Unclassified divergences** | Chasing advisory divergences, ignoring accumulation drift | Classify every divergence by category; trend by category over time |
| **Coarse divergence detection** | "Something's wrong in level 23" but no precision | Framewise checksums + frame-level bisection to pinpoint exact frame and field |

---

## References

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
| ProtonDB | Game compat | Per-game fidelity tier ratings (Platinum→Borked) | protondb.com |
| Blargg test ROMs | NES emulation | Isolated component tests with deterministic checksums | nesdev.org |
