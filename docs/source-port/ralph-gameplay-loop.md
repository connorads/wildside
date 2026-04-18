# Gameplay Loop

## Bootstrap
1. Read `docs/source-port/backlog.md` and pick the highest-priority open `1p-playable` or `visual-parity` issue that does not yet have canonical evidence.
2. Skip items marked `blocked:needs-human-repro` or `stalled:agent-blocked`.
3. Check `.cache/recordings/` and `.cache/cases/` for an existing recording slug or reduced case tied to that issue.
4. If no evidence exists, capture it first with `play-native --record <slug>` and, for visual issues, `capture-frames`.
5. If no open evidence-driven issue is ready, fall through to the **Proactive verification** section below.

## Work
6. Turn the chosen issue into a reproducible artefact: recording slug, reduced case, oracle compare window, or named frame-capture checkpoint.
7. Read the ASM routine that owns the confirmed behaviour before patching.
8. Write the narrowest failing regression that proves the bug.
9. Implement the fix and re-run the minimum gate named by the backlog item.
10. Always finish with `mise run verify-slice-fast`; add `mise run verify-1p-playable` or frame/oracle checks when the issue requires them.

## Commit
11. Update `docs/source-port/backlog.md` with the new evidence handle, gate, and next verification commands.
12. If ASM-backed code changed, refresh citations with `kidcli asm-coverage --update`.
13. Commit only after the chosen gate is green and the backlog entry explains the new evidence.

## Rules
- Backlog truth beats raw ASM coverage. Do not skip an open player-visible issue just because another subsystem has more uncited labels.
- TAS closure is replay evidence, not completion.
- Visual issues are not done without frame-backed evidence.
- If a report cannot be reproduced, record that explicitly in the backlog instead of leaving it vague.
- Use ASM coverage as a discovery aid and fallback queue, not the lead selector.

## Circuit-breaker
- After 3 cycles on the same target with no code commit (only docs/evidence refreshes), mark it `stalled:agent-blocked` in backlog and auto-rotate to the next item.
- Items marked `blocked:needs-human-repro` require interactive play; skip them entirely in agent loops.
- Items marked `stalled:agent-blocked` can be retried after a human provides new evidence or a different approach is identified.
- Proactive verification (mutation testing, enemy audit, sweep expansion) is always valid fallback work when all top backlog items are blocked or stalled.

---

## Proactive verification

When no backlog item is actionable (all blocked/stalled), or after completing a backlog fix, pick one of these strategies. They discover bugs *before* they're noticed in play.

### Strategy 1: Mutation testing sweep

Find where tests are blind. Every surviving mutant (a sign flip, comparison reversal, or constant change that no test catches) is a potential source-port bug.

**Prerequisites**: `.cargo/mutants.toml` has `gitignore = true` (avoids copying `.cache/`).

**Loop fit**: treat full-file sweeps as manual/post-fix-batch verification,
not a default `rl source-port` task. Large modules such as `enemies.rs`
currently enumerate thousands of mutants and can exceed the loop timeout; broad
baseline tests also depend on generated cache assets that `gitignore = true`
intentionally omits from cargo-mutants' scratch tree. Keep autonomous iterations
on commit-sized ASM coverage/audit tasks unless mutation testing has been split
into a focused shard with a runnable baseline.

**Tier 1** (zero unit tests, highest impact):
```bash
cargo mutants -p kid-core -f enemies.rs -j 4 --output /tmp/mutants-enemies -- --lib -- --skip demo_ --skip published_tas --skip snapshot
cargo mutants -p kid-core -f combat.rs -j 4 --output /tmp/mutants-combat -- --lib -- --skip demo_ --skip published_tas --skip snapshot
cargo mutants -p kid-core -f blocks.rs -j 4 --output /tmp/mutants-blocks -- --lib -- --skip demo_ --skip published_tas --skip snapshot
```

**Tier 2** (some tests, check for gaps):
```bash
cargo mutants -p kid-core -f physics.rs -j 4 --output /tmp/mutants-physics -- --lib -- --skip demo_ --skip published_tas --skip snapshot
cargo mutants -p kid-core -f terrain.rs -j 4 --output /tmp/mutants-terrain -- --lib -- --skip demo_ --skip published_tas --skip snapshot
```

**For each surviving mutant**:
1. Find the corresponding ASM routine
2. If the mutant changes behaviour that should differ from ASM → it's a real bug, fix it
3. If behaviour is correct but untested → write the test (cite ASM label)
4. If dead code → investigate why

**Never use `--in-place`** — it corrupts source files.

### Strategy 2: Enemy family audit

10 of 28 enemy families have zero internal coverage. Audit each against the ASM.

**Priority order** (backlog overlap first, then by gap size):
1. Tank/shooting (#144, #145) — `code/enemy/Tank.asm`, `Tank_Shooting.asm`
2. Mini hopping skull — 92 uncovered locs
3. Drips — 74 uncovered locs
4. Ninja — 48 uncovered locs
5. Fireball, Armadillo, Diamond, Emo Rock, Sphere (11–25 locs each)

**Per-family workflow**:
1. Read `code/enemy/<family>.asm` — map the state machine (init → patrol → detect → attack → cooldown → die)
2. Extract all constants: speeds, HP, frame timings, detection ranges, projectile parameters
3. Read corresponding Rust code in `enemies.rs` — find the matching functions
4. Compare constants line by line. **Watch for direction conventions**: `x_direction=0` = LEFT for enemies whose art faces left, `x_direction=$FF` = flipped
5. Compare state transitions and branch conditions
6. Write targeted unit tests for divergences found (cite ASM label in test name/comment)
7. Fix any bugs, cite ASM label in code comment: `// ASM loc_XXXXX: <instruction>`
8. Run `cargo run -p kidcli -- asm-coverage --update`

**If the audit finds a bug**: update the backlog with root-cause evidence, then fix it in the same session. A root-cause-identified backlog entry (like #144/#145 now have) means the next agent can jump straight to implementation.

### Strategy 3: Enable enemy-state sweep comparison

The oracle sweep compares native vs oracle across 69 levels, but enemy-state comparison may be underused. Check `crates/kidcli/src/sweep/mod.rs` for `FrontierKind::Enemy` support and wire it into the default sweep or add `--frontier-kind enemy` CLI flag.

### Strategy 4: ASM coverage drive

The systematic long game. 92.4% of internal ASM branches (loc_ labels) are unverified.

```bash
cargo run -p kidcli -- asm-coverage --uncovered  # gap list
```

Same per-routine workflow as Strategy 2 but for non-enemy code. Focus on:
- `kid.asm` (kid_main) — player physics, state machine, input handling
- Block/platform code — matches backlog items #136, #139, #140, #143
- Level transition code

### Strategy 5: Property-based testing expansion

Extend `physics.rs`-style proptests to `enemies.rs`, `combat.rs`, `terrain.rs`:
- Speed calculations bounded, HP thresholds non-negative, frame timing counts valid
- Collision overlap symmetric, hitbox dimensions non-negative, damage values bounded

**Important**: proptests catch bounds violations but miss `+`→`-` or `<`→`>` mutations. Always pair with exact-value unit tests from ASM.

---

## Autonomous goal: `--goal proactive-verification`

The strategies above are also available as an autonomous loop goal:

```bash
cargo run -p kidcli -- autonomous-loop --goal proactive-verification --emit-json
```

The goal runs a priority cascade through the enabled strategies in `docs/source-port/proactive-verification-config.yaml`. The `source-port-frontier` goal automatically delegates to `proactive-verification` when all `sub_` labels are covered and no backlog/case-queue work is actionable.
