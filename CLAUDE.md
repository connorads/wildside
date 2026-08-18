# Source-Port Reference Guide

This repo keeps source-port research material in cache-local checkouts and repo docs. Treat this file as the canonical guide for finding the right reference fast. `AGENTS.md` should point here as a symlink. `.cache/` is always generated local state; if something needs to be durable or reviewable, move it into docs, code/tests, or explicit fixtures instead of tracking it in `.cache/`.

## Loop sessions

If your prompt mentions a loop session (or you're asked to "take the next item"),
follow `docs/source-port/loop.md`; it is the whole loop. The backlog is sharded by
reader: `docs/source-port/backlog.md` holds the NOW handoff and the open queue
(`[ ]`/`[~]` rows, plus never-selectable frozen `[f]` lanes);
`backlog-awaiting-human.md` holds the `[?]`/`[b]` rows you skip;
`backlog-archive.md` holds resolved rows and per-issue history (grep by `#NNN`);
`session-log.md` is the past-handoff log. Select the next item with
`mise run project-status` (it is priority-ordered across the live lanes; file order
is not). Do NOT add more debug fields unless ASM + oracle leave genuine ambiguity;
the answer is in the disassembly.

## Source precedence

Use sources in this order:

1. Local Kid truth
- `.cache/upstream/kid-chameleon-disasm`
- `.cache/generated/<rom-sha1>/asset-manifest.json`
- `.cache/native-oracle-runs/<scenario>/`
- `wildside inspect-level`, `inspect-enemy`, `probe-native`, `compare-native-oracle`

2. Kid bundled docs
- `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/formats.txt`
- `.cache/upstream/kid-chameleon-disasm/doc/sound.txt`
- `.cache/upstream/kid-chameleon-disasm/variables.asm`
- `.cache/upstream/kid-chameleon-disasm/constants.asm`
- `.cache/upstream/kid-chameleon-disasm/kid.asm`

3. Cached research repos
- `.cache/references/m68k-instructions-documentation`
- `.cache/references/gems`
- `.cache/references/gemsplay`

4. Local skills and repo docs
- `docs/source-port/backlog.md` (NOW block + queue) and `backlog-archive.md` (grep `#NNN`)
- `.agents/skills/m68k-kid-asm/SKILL.md`
- `.agents/skills/as-assembler/SKILL.md`
- `docs/source-port/methodology.md`
- `docs/source-port/reference-index.md`
- `docs/source-port/research-map.md`
- `docs/source-port/level-1-dossier.md`
- `docs/source-port/whole-game-index.md`

5. Web/manual fallback
- Sega Retro Mega Drive hardware docs
- official M68000 manual

## Fidelity principles

- **Bugs are features**: original game bugs must be faithfully reproduced: collision edge cases, sprite glitches, off-by-ones. Mark with `// ORIGINAL_QUIRK: <description>`. (Julius, OpenMW, Chocolate Doom, devilutionX all preserve original bugs for compatibility)
- **Oracle is truth**: if native differs from oracle, native is wrong until proven otherwise. Exceptions: evidence-backed `oracle_limitation` classifications only. ("Accuracy is not 'does it play Zelda?'; playing games is a side effect of accuracy", byuu)
- **Never guess**: when behaviour is ambiguous, trace the disassembly or compare oracle frames. Write a probe scenario before writing any fix. (bsnes: 80 hours to reverse-engineer one Speedy Gonzales bus conflict rather than guessing)
- **Engine-level only**: no level-specific overrides or game-specific hacks. If a fix works for level 1 but breaks level 23, the architecture needs rethinking. (bsnes: "whack-a-mole emulation" anti-pattern)
- **Progressive fidelity**: approximate implementations acceptable as stepping stones when marked `// TODO(fidelity): <description>` and tracked. Never ship a guess as if it were verified.
- **Code as documentation**: implementation should read as a specification of original behaviour. Avoid speed hacks or clever abstractions that obscure what the original did. (bsnes: speed hacks "destroy code's ability to serve as verifiable hardware documentation")

## ASM-first implementation rule

The 68k disassembly at `.cache/upstream/kid-chameleon-disasm/` is the **ultimate source of truth** for all gameplay behaviour.

That standing is itself gated, not assumed: `mise run verify-disasm-fidelity` assembles the pinned checkout with the AS toolchain and byte-compares the result against the reference ROM. Run it after any change to the checkout, the split step, or the ROM.

- **Read the ASM before writing native code.** Find the routine in `kid.asm` or `code/*.asm`, read 50-100 lines around the label, and understand what the ROM does before implementing anything.
- **Cite the ASM label in every code comment.** Example: `// ASM loc_3CD70: cmp.w x_pos(a4),d4; blt.s skip; st x_direction(a3)`. This prevents future misreadings and makes the implementation auditable.
- **Name symbols descriptively; keep the address in the doc-comment, not the identifier.** For NEW symbols use a descriptive identifier and record the ASM address (`loc_XXXX`/`sub_XXXX` + `.asm` file) in a one-line doc-comment only — e.g. `/// ASM sub_37ABC (kid.asm): Heady-Metal body-piece offsets`. Do not bake the address into the name, and never use the double-address `_SUB_xxxx_..._LOC_yyyy` form. This matches mature decomps (zeldaret/OoT, pret). No mass-rename of the existing address-bearing identifiers (churn + traceability risk); strip an address suffix only opportunistically when you are already editing that symbol, moving the address into the doc-comment.
- **Understand register/flag conventions before interpreting.** In particular: `x_direction=0` means default sprite orientation (LEFT for enemies whose VRAM art faces left), `x_direction=$FF` means flipped. Don't assume 0=Right.
- **Cross-reference ASM with oracle.** If the ASM and oracle disagree, investigate further. If they agree, implement with confidence.
- **Never guess from observed behaviour alone.** Oracle observation tells you WHAT happens but not WHY. The ASM tells you WHY, which is what you need to implement correctly.
- **Tests must cite the ASM routine they validate.** The test name or comment should reference the label so future readers know the expected value comes from the ROM, not a guess.

## Cache layout

- `.cache/` is untracked generated state only. Do not commit files from it.
- `.cache/upstream/`
  - Kid-specific upstream source repos
- `.cache/generated/`
  - extracted typed asset bundles
- `.cache/native-oracle-runs/`
  - native vs oracle checkpoint artefacts
- `.cache/oracles/`
  - emulator/oracle repos and helpers
- `.cache/references/`
  - generic research repos used for source-port work
- `.cache/tooling/`
  - built tools such as ASL

## Which source to use

- Unknown 68k opcode, addressing mode, condition code, or register convention
  - use `.agents/skills/m68k-kid-asm/SKILL.md` first (sections 1-3)
- Kid Chameleon object layout, direction convention, or collision IDs
  - use `.agents/skills/m68k-kid-asm/SKILL.md` (sections 5-7)
- Mega Drive hardware (VDP, DMA, sprites, memory map)
  - use `.agents/skills/m68k-kid-asm/SKILL.md` (section 9)
- Unknown AS directive, macro, or pseudo-op
  - use `.agents/skills/as-assembler/SKILL.md` and its `references/` docs
- Unknown Kid RAM variable or object-state field
  - use `variables.asm` and `doc/ram_layout.txt`
- Unknown level/header/enemy/platform format
  - use `doc/formats.txt`, `level/*`, and `wildside inspect-level`
- Unknown enemy family logic
  - use `code/enemy/*.asm`, enemy layouts, and oracle checkpoints
- Unknown GEMS or audio bank behaviour
  - use `doc/sound.txt`, `.cache/references/gems`, and `.cache/references/gemsplay`
- Unknown Mega Drive hardware effect
  - use Kid asm first, then Sega Retro hardware docs
- Frame-level divergence debugging (TAS or sweep)
  - follow `docs/source-port/debug-playbook.md`; probe before reading ASM

## First commands

```bash
mise run setup                  # wire the pre-commit hook (clippy/fmt/deny/tests) — once per clone/worktree
mise run bootstrap-upstream
mise run bootstrap-references
mise run oracle-setup
mise run extract-assets
mise run verify-disasm-fidelity  # gate: the disassembly must assemble byte-identical to the ROM
cargo run -p wildside-cli -- inspect-level --level 00
cargo run -p wildside-cli -- inspect-enemy --enemy 01
cargo run -p wildside-cli -- compare-native-oracle --scenario boot-title-start-level1
cargo run -p wildside-cli -- asm-coverage                    # coverage summary
cargo run -p wildside-cli -- asm-coverage --uncovered        # uncovered labels
cargo run -p wildside-cli -- asm-coverage --json             # machine-readable summary
cargo run -p wildside-cli -- asm-coverage --check-required   # fast-gate required-label citations
cargo run -p wildside-cli -- asm-coverage --check-no-new-test-only  # ratchet new test-only debt
cargo run -p wildside-cli -- asm-coverage --update           # regenerate .cache/asm-coverage.yaml (untracked)
```

> **Citation ≠ fidelity ≠ reachability.** `asm-coverage` is a discovery aid: a label is
> "covered" the moment any Rust file cites `loc_XXXX`/`sub_XXXX`, with zero verification. A
> citation that lives only in `#[cfg(test)]` code is test-only debt, not shipped behaviour.
> Fidelity is proven by the oracle/TAS gates and checked-in regression tests — never by the
> coverage number. `verify-slice-fast` runs `--check-required` and
> `--check-no-new-test-only`, but those are reachability/regression guards, not fidelity proof.
> Use coverage to find undiscussed ASM, not to claim behaviour is implemented or correct.

## Level 1 reading order

1. `docs/source-port/level-1-dossier.md`
2. `.cache/upstream/kid-chameleon-disasm/doc/formats.txt`
3. `.cache/upstream/kid-chameleon-disasm/doc/ram_layout.txt`
4. `.cache/upstream/kid-chameleon-disasm/kid.asm`
5. `.cache/upstream/kid-chameleon-disasm/code/enemy/*.asm` for the first visible enemy families
6. `.cache/native-oracle-runs/boot_title_start_level1/`

## Iteration workflow

Canonical bug/feature backlog, sharded by reader (one row per issue):
- `docs/source-port/backlog.md`: NOW handoff + the **open queue** (`[ ]`/`[~]`; frozen
  `[f]` lanes stay here but are never selectable). Keep
  Verify cells short here; table shape is status, priority, system, summary, verify.
- `docs/source-port/backlog-awaiting-human.md`: `[?]`/`[b]` rows. The agent skips them;
  only a human promotes `[?]`→`[x]` or unblocks `[b]`. Its table adds `Target` and
  `Check` so tools and playtesters can see the validator surface directly; `Check`
  must name the concrete symptom and validation action, not "retest the reported route".
- `docs/source-port/backlog-archive.md`: resolved (`[x]`/`[-]`) rows and per-issue
  evidence/history (follow-ups, rejected hypotheses, dated logs); grep it by `#NNN`
  (e.g. `grep -n '#238' docs/source-port/backlog-archive.md`) rather than loading the tracker.
- `docs/source-port/session-log.md`: append-only log of past session handoffs.

Select the next item with `mise run project-status`, which reads the live lanes and
prints a priority-ordered ready/open/blocker view. Session handoff: the NOW block at the
top of `backlog.md` (rewrite it each session; never let it accrete: old handoffs go to
`session-log.md`).

### Principles (bsnes methodology)
- Fix behaviours at engine level, never level-specific overrides
- Every fix must pass full sweep; if it fixes level 1 but breaks level 23, the fix is wrong
- Every bug fixed should become a permanent regression test (scenario or snapshot)
- No advisory field masking: `ADVISORY_FIELDS` (sweep/mod.rs) is empty; every field diff is either a bug or an evidence-backed `oracle_limitation` (e.g. oracle stuck on title/pause), never a silently masked "expected divergence"
- Probe before coding: write a scenario that exercises the behaviour, run against oracle, understand ground truth before touching native code
- Cross-reference sources: check disassembly AND oracle; if they agree, proceed. If they disagree, investigate further
- Preserve original quirks explicitly: mark with `// ORIGINAL_QUIRK:`. If behaviour looks wrong but oracle confirms it, the original game has that behaviour
- Oracle-limitation vs authoritative: `oracle_limitation` covers checkpoints where the oracle never reached a comparable state (title, pause, wrong warp level) and needs cited evidence in the matcher. Authoritative divergences (position, velocity, state, lifecycle, counts, score) are always bugs
- Reject dead ends once: record them in `backlog-archive.md` under the issue's `#NNN` so the next session does not retry them from memory or roadmap prose

### Session protocol
1. Follow `docs/source-port/loop.md`: read the NOW block in `backlog.md`, then run
   `mise run project-status` and take the top item it lists. It is priority-ordered
   across the live lanes; do not eyeball the table for "the top row". Never pick a human
   validation/repro blocker (the `[?]`/`[b]` awaiting-human lane).
   a. project-status lists "ready to implement now" (open rows whose evidence says "root cause identified") first; take those before generic open playability/visual risks.
   b. If project-status shows no ready item and no open risks, use the proactive-verification fallback in `loop.md` (mutation-test one file, audit one enemy family vs ASM, extend ASM coverage)
2. Mark `[~]` when starting work
3. Investigate (structured):
   a. Reproduce: capture frames or run scenario showing the bug
   b. Classify: timing, collision, state machine, rendering, or audio?
   c. Trace oracle: run same scenario against oracle, capture state checkpoints
   d. Read disassembly: find the relevant ASM routine (use source precedence)
   e. Cross-reference: compare native logic to both ASM and oracle data
   f. Write a probe scenario if one doesn't exist for this behaviour
4. After 2 consecutive probe/docs-only commits on the same issue, the next session must add instrumentation or narrow the scenario before another gameplay patch. **No code commit ⇒ write the dead end under the issue's `#NNN` in `backlog-archive.md` and stop; never commit churn.**
5. Fix: implement in focused commit(s)
6. Verify visually: capture frames, Read PNGs, compare before/after
7. Verify mechanically: `mise run verify-slice-fast` must pass (+ `mise run verify-tas-full` after gameplay-affecting commits; do not run the two concurrently, they share `.cache/tas-verify`). The gate runs `cargo clippy --workspace --all-targets -- -D warnings` (the `lint` dep): **a clippy warning is a red gate, same status as a failing test**, and clippy must be green on `main` (the wired pre-commit hook + the gate enforce it). **Gate must be green on main at all times**; never commit tests with unverified values. If values aren't confirmed, mark `#[ignore]` with a comment explaining what needs re-derivation. A red gate blocks all loop commits.
8. Close out (same commit as the fix): set the row's status and, if it leaves the open
    lane, MOVE the whole row to its lane: `[?]`/`[b]` → `backlog-awaiting-human.md`,
    `[x]`/`[-]` → the resolved index in `backlog-archive.md`. For awaiting-human rows,
    set `Target` (`0xNN`, `boot`, `demo`, `demo:0xNN`, `audio-device`,
    `recording:<slug>`, or `unknown`) and one short overlay-safe `Check` sentence with
    no links, shell commands, regression dump, or vague "reported route" wording.
    Append closing evidence to `backlog-archive.md` under its `#NNN` (keep the
    open-queue Verify cell short).
    Rewrite the NOW handoff and prepend a session note to `session-log.md` (newest first);
    `mise run backlog-lint` checks the result.
9. After 3-5 fixes: run `cargo mutants -p wildside-core -- --lib` to check for test gaps
10. After 3-5 fixes: playtest with `mise run play-native`, triage new bugs

### Frontier diagnostics defaults
- For TAS frontier work, start with `cargo run -p wildside-cli -- verify-tas --scenario <scenario> --bisect --frontier-kind enemy`
- Use `--frontier-kind gameplay` only when Kid/camera/global behaviour is the known first causal divergence
- Follow with `cargo run -p wildside-cli -- tas-diff --scenario <scenario> --include-advisory --frontier-kind enemy` so hidden enemy drift is visible before the visible gameplay break

### Debug field promotion workflow
- Register every emitted `debug_fields` or `gameplay_debug_fields` key in `docs/source-port/debug-field-registry.yaml`
- New keys start as `exploratory` unless they already drive pass/fail parity or regression checks
- If a debug key is used to choose, reject, or verify a fix in repeated investigation work, mark it `candidate` and add it to `candidate_promotions` in `docs/source-port/debug-field-registry.yaml`
- Run `mise run audit-debug-fields` before landing instrumentation changes; unregistered keys are errors, stale `candidate` keys need promotion or an explicit exemption note
- Promote a key from debug bag to typed state when it is used in parity logic, checked-in regression assertions, or multiple frontiers; keep the debug bag only for raw or temporary correlation data

## Visual debugging

The Read tool supports viewing PNG/JPEG images directly. Use it to inspect captured frames,
diff composites, and per-layer diagnostics.

### Commands
- `cargo run -p wildside-cli -- capture-frames --level XX --frames N,N,N --output .cache/debug/`
- `cargo run -p wildside-cli -- diagnose-frame --level XX --frame N --output .cache/debug/ --save-layers`
- `cargo run -p wildside-cli -- sweep --levels 0-104 --tier state` (full oracle comparison)
- `cargo run -p wildside-cli -- sweep-census [--run ID]` (classify a sweep run's blocking diffs by enemy family; writes `.cache/audit/sweep-census.json`; `mise run sweep-census`)
- `cargo run -p wildside-cli -- diagnose-level XX` (deep single-level analysis)
- See "First commands" for `asm-coverage` discovery, JSON, ratchet, and update commands.

### Verification loop
1. CAPTURE BEFORE: `capture-frames` to `.cache/debug/before/`
2. Read the PNGs to understand current state
3. Fix the code
4. CAPTURE AFTER: same command to `.cache/debug/after/`
5. Read the PNGs to confirm the fix
6. Run `mise run verify-slice-fast` for mechanical regression check

## Validation methodology

### Disassembly fidelity (is the source of truth actually the ROM?)
- `mise run verify-disasm-fidelity` (`scripts/verify_disasm_fidelity.py`) assembles `.cache/upstream/kid-chameleon-disasm` with the AS toolchain and byte-compares `kid_built.bin` against the reference ROM. Any differing byte fails the task
- On a mismatch it re-assembles with `-g map` and attributes every differing region to its nearest preceding ASM label and source line, so offsets point at source rather than at raw addresses
- No known-mismatch allowlist exists. The pinned upstream commit assembles bit-perfect, so a diff means a real regression: stale `split.py` chunks, a locally patched checkout, or a non-canonical ROM
- ~2s warm, ~12s cold (clones and builds AS into `.cache/tooling`). On-demand rather than a `verify-slice-fast` dep, because it needs the ROM plus a bootstrapped checkout

### Oracle comparison (correctness)
- jgenesis emulator as ground truth for game state
- `DynamicScenario`: per-frame u8 button masks, JSON-serialisable, identical execution paths in native and oracle
- Sweep: 105 levels x idle scenario, state-level checkpoint comparison
- Position tolerance: 6px. No field-level advisory masking (`ADVISORY_FIELDS` is empty); non-comparable oracle checkpoints classify as `oracle_limitation` with cited evidence.

### 68000 core conformance (the layer under the oracle)
- `mise run bootstrap-cpu-vectors` then `mise run verify-cpu-vectors` replays the SingleStepTests 680x0 + m68000 vector sets (1.3M single-instruction vectors) through the pinned `r68k-emu` core; `mise run verify-cpu-vectors-ratchet` is the downward-only gate (baseline `docs/source-port/cpu-vector-baseline.yaml`, provenance-pinned)
- Measured result, defects, and the limits of the measurement: `docs/source-port/cpu-core-conformance.md`. Headline: ~98% arithmetic-exact, but BCD/`DIV` results, exception stack frames, and `MUL`/`DIV` cycle counts are wrong, and r68k panics rather than raising some address errors
- Not in `verify-slice-fast`: 600 MB of vectors and minutes per run, so it is a periodic gate like `sweep`
- A green arithmetic verdict removes the CPU core as a suspect for state divergence; it proves nothing about Kid Chameleon behaviour

### Pixel-perfect frame comparison (visual correctness)
- 320x224 fixed-palette pixel art; exact pixel matching is appropriate (no perceptual metrics needed)
- Use `image` crate for byte-by-byte RGBA comparison
- `dify` CLI for ad-hoc investigation: `cargo install dify; dify native.png oracle.png -o diff.png`

### Input replay (regression)
- `DynamicScenario` already supports per-frame button recording + checkpoint assertions
- Both native and oracle use identical `from_mask(u8)` input paths
- Future: `record-inputs` command to capture live play sessions as replay scripts

### Snapshot testing (regression)
- `insta` crate: `assert_yaml_snapshot!` on NativeTraceFrame structs
- No redactions: snapshots pin every field, including object counts and score; `cargo insta review` TUI for accepting intentional changes
- Complements oracle (correctness) with native-vs-prior-native (regression)

## Test strategy

| Situation | Test type | Rationale |
|-----------|-----------|-----------|
| New engine behaviour | Probe scenario + oracle comparison | Like bsnes test ROMs: exercise specific behaviour, compare to ground truth |
| Fixed bug | Timing/gameplay regression test (`verify-slice-fast` gate) | Every fix becomes a permanent regression test |
| Pure function, large input space | Property-based test (proptest) | Speed tables, slope formulas, gravity; verify properties hold across range |
| Complex state | Snapshot test (insta) | NativeTraceFrame at checkpoint; detect unexpected state changes between commits |
| Visual correctness | Frame capture + pixel comparison | Any pixel mismatch against oracle = regression |
| Determinism | Replay same scenario twice, diff output | Deterministic engine is prerequisite for all other testing |
| Engine-wide invariant | Metamorphic test | Determinism, bounds, symmetry; no per-case oracle needed |
| Stateful system | Stateful property test (proptest-state-machine) | Model lifecycle as states, auto-shrink failing transition sequences |
| Test suite confidence | Mutation testing (`cargo-mutants`) | Surviving mutants = test gaps; target ≥80% kill rate |
| Operator/sign correctness | Targeted unit tests with exact values | Proptests catch bounds violations but miss `+`→`-` or `<`→`>` mutations |

### Mutation testing

Run per-file with demo/TAS/snapshot tests skipped (they have pre-existing failures):
```bash
cargo mutants -p wildside-core -f <file>.rs -j 4 -- --lib -- --skip demo_ --skip published_tas --skip snapshot
```

**Never use `--in-place`**; it silently corrupts source files with `/* ~ changed by cargo-mutants ~ */` markers. Always use `--output /tmp/mutants-<name>`.

**TMPDIR in agent shells**: Codex/Claude Code launch without launchd's per-user
`TMPDIR`. On macOS an *unset* `TMPDIR` is harmless — Rust's `env::temp_dir()`
resolves `/var/folders/.../T/` — so `cargo test` and the pre-commit gate need no
`TMPDIR` export. Two task boundaries need a default: `cargo-mutants` runs
isolated `build.rs` builds, and the wasm build can rerun a wrapped dependency
`build.rs`; both fail when the wrapper receives an empty path. `mutants-file`
and `web-build` default `TMPDIR` to `/tmp` themselves. Do not add a blanket
per-commit export or change ordinary Cargo commands.

**Lessons learned:**
- Proptest properties (bounded, converges) are too loose to catch sign/direction mutations; pair them with exact-value unit tests
- Left-facing code paths are systematically undertested; the ASM is NOT perfectly mirrored (different brake constants), so symmetry doesn't mean redundancy
- Binary parser byte-offset mutations need test data with multiple entries per section and distinct field values

## Code conventions for fidelity

- `// ORIGINAL_QUIRK: <desc>`: faithfully reproduced original bug (verified against oracle/disassembly)
- `// DEVIATION: <desc>`: known intentional difference from original (e.g. modern input handling)
- `// TODO(fidelity): <desc>`: approximate implementation needing refinement
- `// UNVERIFIED: <desc>`: behaviour not yet compared to oracle/disassembly

See `docs/source-port/methodology.md` for full investigation playbook, testing taxonomy (metamorphic, mutation, fuzzing, Design by Contract), divergence classification, framewise checksumming, fidelity metrics, and cross-referencing protocol.
