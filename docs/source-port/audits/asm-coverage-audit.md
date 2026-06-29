# ASM-Reference Coverage Audit — 2026-06-28

Scope: does the ASM-reference coverage infrastructure genuinely achieve "100%
coverage that is wired in", where are the gaps, and how can the methodology be
made more accurate, better enforced, and tidier.

Method: 6 parallel investigators (coverage algorithm, enforcement wiring,
dead/stale citations, uncovered-label triage, debug-field registry, docs
consistency), each adversarially re-checked by an independent skeptic, plus
direct orchestrator verification of the headline numbers and the live
`audit-debug-fields` state. All numbers reproduced against the prebuilt binary
and by mirroring the parser with `grep -a` (kid.asm contains non-text bytes, so
plain grep silently treats it as binary — see Reproducibility).

---

## Verdict

**Is coverage 100%?** No — and "coverage" does not mean what the question
implies.

- `wildside asm-coverage` reports **sub_ 279/293 (95.2%)** and **loc_
  3798/3824 (99.3%)**, never 100%.
- The metric counts **citation presence**, not fidelity. The tool says so
  itself: `asm_coverage.rs:771-773` prints *"label citation tracks
  recovered/discussed ASM, not player-visible fidelity"*. A single one-line
  `// ASM loc_XXXX` comment in a scanned crate marks a label "covered" with zero
  verification — the metric is author-attestable, not earned.

**Is it wired in?** No. The tool, its `--check-required` regression gate, and
`audit-debug-fields` are all **dormant** — invoked by no `mise` task, no `hk`
pre-commit step, and no CI (there is no CI: `.github` does not exist). Worse,
**`mise run audit-debug-fields` currently fails (exit 1) on `main`** — it is red
and nothing notices, because it is not in the gate.

**Are the ~40 uncovered labels real gaps?** No genuine gameplay porting gaps. 33
are the deliberately-emulated GEMS sound driver (handled by `wildside-md`, not
hand-ported); 7 are citation-granularity gaps inside already-ported, cited
routines. See [Uncovered-label triage](#d-uncovered-label-triage).

The good news: the **denominator is currently complete and correct** (4117
labels = 293 sub_ + 3824 loc_, reproduced exactly), the output is untracked
`.cache` state (deliberately, #38), and the tool already models a richer
`Mentioned/Implemented/Tested/OracleBacked` ladder than the headline exposes.
The problem is not dishonesty — the tool disclaims itself accurately — it is
that the headline is a weak signal, the only real teeth (the 9-label
`REQUIRED_LABELS` gate) are never pulled, and the docs around it overclaim.

---

## How the metric actually works

`run_asm_coverage` (`crates/wildside-cli/src/asm_coverage.rs:677`):

1. **Denominator** — `extract_all_asm_labels` (lines 124-140) reads `kid.asm` +
   `code/**/*.asm`, takes each line that (trim-left) starts `sub_`/`loc_`, up to
   the first `:`, where the suffix is all hex. → 4117 labels.
2. **Numerator** — `extract_native_citations` (lines 163-175) scans **only**
   `crates/wildside-core/src` and `crates/wildside-data/src` for `sub_`/`loc_`
   tokens; `build_coverage_map` (256-284) marks a label covered iff its exact
   token string appears in that set.
3. The richer per-label state (`coverage_state`, `implementation_cited`,
   `test_cited`, `oracle_backed`, `citation_breakdown`) is overlaid only in the
   untracked `.cache/asm-coverage.yaml`, not in the headline.

This is a one-directional, exact-string, two-crate scan. Every limitation below
follows from those three properties.

---

## Findings

Severity: **High** = misleads or hides regressions / red on main; **Medium** =
material accuracy gap; **Low** = latent or cosmetic; **Info** = confirmed
characterisation.

### A. The headline number

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| A1 | Info | Headline is **95.2% sub_ / 99.3% loc_**, not 100%; it is a citation-presence signal, self-disclaimed as not fidelity. | `asm_coverage.rs:256-284, 771-773`; reproduced via binary + grep parser-mirror (4077/4117 in-scope). |
| A2 | Medium | **Gameable**: one `// ASM loc_XXXX` comment in core/data instantly marks a label covered, no verification. The authors of the comments determine the %. | `build_coverage_map` 256-262 (presence test only). |
| A3 | Medium | **Scanner scope is core+data only** → labels documented elsewhere read as false-negative *uncovered*. Unscanned crates carry citations: cli ~514, frame ~349, render ~131, md ~15 unique tokens. 6 real sub_ (GEMS) are cited only in `wildside-md` and counted uncovered; counting them lifts sub_ to **285/293 (97.3%)**. | `asm_coverage.rs:165-168`; `sub_E13DA/E143A/E149C/E14AC/E151C/E153E` cited in `wildside-md/src/lib.rs` (e.g. :4956-4957). |
| A4 | Low | **Case-sensitive** match vs all-uppercase disasm labels; a lowercase-only citation (e.g. `loc_1002e`) would not credit `loc_1002E`. Net-zero false negatives today (every label also has an uppercase citation), but latent. | `asm_coverage.rs:204-207, 257`; 2412 tokens cited in non-canonical case. |
| A5 | Low | **Inline `#[cfg(test)]` misclassified as implementation** — `classify_citations` buckets by path only (`/tests.rs` or `/tests/`), and `cited_in` is per-file, so inline test modules (the dominant pattern; 45 files) read as `Implemented` not `Tested`. Affects the YAML state ladder, not the headline. | `asm_coverage.rs:494-511, 207`; e.g. `loc_1AB6A` test-only in `movement.rs`. |
| A6 | Medium | **77% of labels collapse into one `kid_main` bucket** — `classify_subsystem` only separates `code/enemy/*`; VDP/DMA/sound/level/gameflow all live in `kid.asm` and cannot be reported per-system. | `asm_coverage.rs:219-246`; `code/` has only `enemy/`. |
| A7 | Low | **Denominator is complete today but fragile** — only `kid.asm` + `code/**` are read; `level/`, `ingame/`, `scenes/`, `constants/variables/macros.asm` define **zero** `sub_`/`loc_` labels now, so nothing is missed — but a future labelled routine under those dirs would silently drop out and inflate the %. | `asm_coverage.rs:124-140`; those dirs grep to 0 label defs. |

### B. Enforcement / wiring

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| B1 | High | **Nothing runs `asm-coverage` or `--check-required` automatically** — zero hits in `.mise.toml`, `hk.pkl`, `.hk-hooks/`, `scripts/`. `verify-slice-fast` deps are `[check-hooks, lint, verify-tas-frontier, backlog-lint]` only. No CI exists (`.github` absent). | `.mise.toml:59-76`; `hk.pkl:14-78`; `grep` across config. |
| B2 | High | **The 9-label `REQUIRED_LABELS` regression gate is dormant.** It is fully built and passing (`required-labels gate: 9 labels covered`, exit 0) but no task/hook/CI passes `--check-required`, so deleting a load-bearing slope/attack citation fails nothing. | `asm_coverage.rs:592-633, 812-838`; `main.rs:1188-1192, 2701-2716`. |
| B3 | High | **The only tracked tests are synthetic** — the 5 `asm_coverage` unit tests build fabricated `CoverageMap`s via `make_label`; none call `extract_native_citations`/`run_asm_coverage` against the real repo. And `verify-slice-fast` only runs `wildside-cli gameplay_regression_`, so they are not even executed by the gate. | `asm_coverage.rs:843-975`; `.mise.toml:70-75`. |
| B4 | Medium | **No tracked coverage artefact** — `.cache/asm-coverage.yaml` is gitignored, so nothing in version control can detect a coverage regression beyond the 9 hand-picked labels. (This was a deliberate move away from a 2.2MB tracked dump, #38 — the fix is a small ratchet, not re-tracking the dump.) | `.gitignore`; `git ls-files`. |
| B5 | Medium | **The `--check-required` doc-comment is misleading** — it claims "Use in CI/pre-commit to prevent regressions" but no CI or hook uses it, and it is documented nowhere else. | `main.rs:1188-1191`. |

### C. Citation integrity (the reverse-direction check the tool omits)

The metric never asks "does each `// ASM` citation resolve to a real label?", so
stale/typo'd citations are invisible. A naive reverse-diff *looks* alarming
(~297 label-shaped "dead" tokens) but adversarial triage collapses it:

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| C1 | Medium | The reverse direction is **structurally unguarded**; ~297 label-shaped citations point at addresses with no `sub_`/`loc_` label. | `asm_coverage.rs:256-262`; case-normalised reverse-diff. |
| C2 | Info | **Almost all of those are measurement artefact, not bugs.** The disassembler only emits `loc_`/`sub_` at branch targets, so any mid-routine offset or range endpoint (`loc_X..loc_Y`) is a real ROM byte with no label. All sampled dead tokens fall inside the labelled ROM range `0x454..0xE15E8`. The tool also ignores **3821 *named* labels** in `kid.asm` (e.g. `EraseBlockFromLevelLayout`, `Load_InGame`) that are the real anchors for ~132/134 sampled dead-in-comment citations. | Bracketing checks; `grep -aoE '^[A-Za-z_]\w*:'` over kid.asm = 3821 named. |
| C3 | Low | **Exactly 2 genuine stale citations** (independently confirmed): `loc_2510C` at `combat.rs:179` and `loc_23629` at `native_player/mod.rs:2086`. Both are absent from the disasm and isolated by an order of magnitude (min-label-gap 30350 / 23467 vs next-largest 254). `loc_23629` is literally the kid.asm *line number* misused as an address. Low harm — each carries a correct named anchor + `(kid.asm:line)`. | `grep -ac` in kid.asm = 0/0; gap-cliff ranking. |
| C4 | Info | A useful reverse lint must therefore **accept named-anchor / in-range addresses** and flag only out-of-range or isolated tokens — a naive "must be a `loc_`/`sub_` label" rule emits ~134 false positives to catch 2 real bugs. The gap-cliff heuristic isolates exactly the 2. | Triage tally: (a) genuine 2, (b) address-not-labelled 132, (c)/(d) 0. |

### D. Uncovered-label triage

40 genuinely-uncovered labels (14 sub_ + 26 loc_); **0 are gameplay porting
gaps.**

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| D1 | Info | **33 are the GEMS sound-driver M68k↔Z80 interface** (`sub_E133C..loc_E15E8`, kid.asm ~49508-49910): Z80 bus arbitration, GEMS driver upload to `$A00000`, mailbox writers, `StopMusic`/`PlaySound`. Audio is the one deliberately-emulated subsystem — `wildside-md::GemsAudioEngine` runs the real `GEMS_Sounddriver.bin` on an embedded Z80 + ymfm YM2612, so these are *not* hand-portable. 6 of the 14 sub_ are already cited in `wildside-md` (hidden by the scope bug, A3); the other 8 + 19 loc_ are GEMS plumbing primitives. | kid.asm:49508-49910; `doc/sound.txt:1`; `project-taxonomy.md:19,54`; `audits/audio-triggers-fidelity-audit.md:124`. |
| D2 | Low | **5 non-GEMS loc_ are citation-granularity gaps** inside already-ported, cited routines: `loc_8370/837C` (chunks of `sub_A4EE`), `loc_8456/8460/8470` (in `sub_8446`). The parser credits only exact label-string matches, so a parent being cited does not credit its internal branch targets. The former `loc_1F56` scan loop and `loc_349C0` Dragon_Flying orphan are now cited inline. | `--json`; parents cited in combat/movement/platforms; `combat.rs:14022`; `dragon.rs` delete-tail note. |

### E. Debug-field registry (reference-accuracy adjacent)

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| E1 | High | **`audit-debug-fields` fails on main (exit 1)** — two live keys `fire_demon_palette_phase_loc_3a4c2` and `fire_demon_palette_source_offset_loc_3a4d6` are emitted but unregistered. Directly confirmed by the orchestrator. | `enemies.rs:2551,2555`; absent from registry; `python3 scripts/audit_debug_fields.py` → exit 1. |
| E2 | High | **The audit is wired into nothing** — standalone `.mise.toml` task only, not in `verify-slice-fast` deps nor the `hk` pre-commit. So E1's red state landed unblocked, contradicting CLAUDE.md ("unregistered keys are errors"). | `.mise.toml:260`; `hk.pkl:14-78`. |
| E3 | High | **Scanner blind spot** — `discover_emitted_keys` globs `crates/**/*.rs` + `scripts/**/*.py`, missing the `include_str!`'d oracle template `oracle_helper_main.rs.txt`, which emits unregistered keys (`pause_menu_option`, `options_speed_button_fast`) that are invisible to the audit. The "no unregistered keys" guarantee is hollow for oracle-side instrumentation. | `audit_debug_fields.py:14-17`; `oracle.rs:16`; `oracle_helper_main.rs.txt:1206-1227`. |
| E4 | Medium | **Candidate staleness ratchet is defeated** — all 3 candidate fields evade `fail_after`: one has an invalid git hash (`c61bd34cd`, not a real object), one is `pending`, one is permanently exempted. A `None` distance only warns, never errors. | `audit_debug_fields.py:43-58,113-128`; registry candidate blocks; `git cat-file -t c61bd34cd` fails. |
| E5 | Low | `candidate_promotions` is `[]` (auto-maturity machinery dead); 41/85 fields have `first_added_commit: pending`; no registered-minus-emitted check, so dead registry entries (85 reg vs 71 emitted) accumulate. Note: the "audit" mutates the tracked registry when `candidate_promotions` is non-empty — a surprising write side-effect. | `audit_debug_fields.py:71-90,105-107,140-143`; registry. |

### F. Documentation consistency

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| F1 | High | **Stale crate paths in the ASM→Rust map** — `m68k-kid-asm/SKILL.md:591-598` points at `crates/kid-core/...` and `crates/kidcli/src/native_player/sprites.rs`, neither of which exists (real: `wildside-core`; `native_player/` has only `mod.rs`; sprites live at `wildside-frame/src/sprites.rs`). The skill CLAUDE.md routes readers to sends them to non-existent files. | `SKILL.md:591-598`; `ls crates/`. |
| F2 | Medium | **`research-map.md:127-131` equates coverage with fixed behaviour** — heading *"Enemy behaviour gaps (via `wildside asm-coverage`)"* + *"Tracked systematically via `asm-coverage --uncovered --subsystem enemy`"* with items struck as `(FIXED)`. The tool cannot verify a behaviour is fixed; this is the exact citation-as-fidelity conflation the tool warns against. | `research-map.md:127-131`. |
| F3 | Medium | **The "citation != fidelity" caveat appears in no doc** — it lives only in code (`asm_coverage.rs:771-773`). Every doc that lists the command (CLAUDE.md:125-127, 218-220; loop.md:57; research-map.md:128) repeats it without the limitation. (loop.md:57-59 is the one place that correctly downgrades it to "a discovery aid, not the lead selector".) | as cited. |
| F4 | Low | `--check-required` and the unwired/untracked reality are documented nowhere; CLAUDE.md duplicates the asm-coverage command block (125-127 and 218-220), both already lagging the real flag set (`--json`, `--check-required` missing). | CLAUDE.md; `main.rs:1175-1193`. |
| F5 | Low | Stale helper refs in `audits/audio-triggers-fidelity-audit.md:97` (`sound_driver_write_command_sub_E1482` etc., `wildside-core/src/lib.rs:1019-1057`) — those symbols no longer exist anywhere; the cited line ranges now hold unrelated code. GEMS plumbing moved to `wildside-md::GemsAudioEngine`. | grep returns 0; ranges repurposed. |
| F6 | Low | Gate failure message says restore citations "in wildside-core or wildside-cli" but the scanner never reads `wildside-cli` (A3); a maintainer following it would add an invisible citation. | `asm_coverage.rs:834` vs `165-168`. |

---

## Proposed backlog (prioritised, de-duplicated)

These are proposals for human review — **not yet applied** (this audit changed
no code/registry/tracker). Effort: S/M/L.

### P1 — red on main, or actively misleading

| Title | Effort | Refs |
|-------|--------|------|
| Register (or de-instrument) the two `fire_demon_palette_*` debug keys to turn `audit-debug-fields` green | S | E1 |
| Close the `audit-debug-fields` scanner blind spot (`*.rs.txt` / oracle template) and register the now-visible keys | M | E3 |
| Fix stale crate paths in `m68k-kid-asm/SKILL.md` (`kid-core`→`wildside-core`; sprites → `wildside-frame/src/sprites.rs`) | S | F1 |
| State "citation != fidelity" once canonically and reference it wherever coverage is mentioned; rewrite `research-map.md:127-131` to stop equating coverage with fixed behaviour | S | F2, F3 |

### P2 — make the metric accurate and enforced

| Title | Effort | Refs |
|-------|--------|------|
| Wire `audit-debug-fields` into the gate (verify-slice-fast deps and/or `hk` pre-commit) — after the P1 key fix turns it green | S | E2 |
| Widen the citation scanner beyond core+data to cli/frame/render/md (+tests) — removes 6 false-negative GEMS labels (95.2→97.3% sub_) and is a prerequisite for the reverse lint | S | A3 |
| Add a reverse "citation resolves to a real ROM address" lint using the named-anchor / in-range / gap-cliff heuristic (NOT naive label-must-exist) | M | C1, C3, C4 |
| Wire `asm-coverage --check-required` into the gate, backed by a real (non-synthetic) integration test asserting the 9 REQUIRED_LABELS resolve against the actual disasm + citations | M | B2, B3 |
| Make candidate-staleness un-bypassable: re-derive bad/`pending` hashes; treat `candidate` + `None` distance as an error, not a warning | M | E4 |
| Document `--check-required` + that the tool/gate are unwired and the yaml is untracked; de-duplicate the CLAUDE.md command block | S | B5, F4 |

### P3 — tidy, hardening, content fixes

| Title | Effort | Refs |
|-------|--------|------|
| Handle the emulated GEMS region honestly (scan `wildside-md` or add an out-of-scope bucket) so the headline isn't permanently capped by intentionally-emulated audio | S | A3, D1 |
| Add a downward-only coverage ratchet (tracked baseline like `dead-code-ratchet`) so any drop in cited-label count is a red gate | M | B4 |
| Track *named* disasm labels (3821 in kid.asm) as first-class citeable anchors | L | C2 |
| Derive a separate "verified fidelity" metric from `oracle_backed`/`parity_evidence` alongside the citation-presence number | M | A2 |
| Case-insensitive matching or a canonical-uppercase-hex citation lint | S | A4 |
| Per-region test classification for inline `#[cfg(test)]` citations | M | A5 |
| Per-system breakdown of `kid.asm` (VDP/DMA/sound/level) via an address-range map | L | A6 |
| Fix the 2 genuine stale citations `loc_2510C` (combat.rs:179) and `loc_23629` (native_player/mod.rs:2086) | S | C3 |
| Reconcile the gate failure message ("wildside-cli" → core/data) and refresh stale `audio-triggers-fidelity-audit.md:97` helper refs | S | F5, F6 |
| Decide `candidate_promotions` usage (populate or remove, incl. the audit's write side-effect) and add a registered-minus-emitted report | S | E5 |
| Add `loop.md` + `whole-game-index.md` to the `methodology.md` cross-reference map; note `audit-debug-fields` red state at CLAUDE.md:205 | S | docs map drift |

---

## Reproducibility / method notes

- **Run the tool**: the prebuilt `target/debug/wildside` has no `LC_RPATH` for
  `libSDL2`; agents invoked it as
  `DYLD_FALLBACK_LIBRARY_PATH=$PWD/target/debug ./target/debug/wildside asm-coverage [--uncovered|--json|--check-required]`,
  or simply `cargo run -q -p wildside-cli -- asm-coverage` (slower).
- **kid.asm is binary to grep** — always `grep -a` on
  `.cache/upstream/kid-chameleon-disasm/*`; plain grep silently returns nothing.
- **Parser-mirror** (matches the tool's 4117 exactly):
  ```sh
  DIS=.cache/upstream/kid-chameleon-disasm
  extract(){ grep -aoE '^[[:space:]]*(sub_|loc_)[0-9A-Fa-f]+:' "$1" | sed -E 's/^[[:space:]]*//; s/:$//'; }
  { extract "$DIS/kid.asm"; for f in "$DIS"/code/enemy/*.asm; do extract "$f"; done; } | sort -u   # 4117
  ```
- **Case-normalise with `tr 'a-z' 'A-Z'`**, never `tr a-f A-F` (the latter
  corrupts the `b`/`c` in `sub_`/`loc_` prefixes — both sides must normalise
  identically or the diff is meaningless).
- The dead-citation denominator is methodology-dependent: ~134-138 distinct
  label-shaped dead tokens appear in `// ASM` comments depending on the
  extraction; the **2 genuine stale citations** and the **gap-cliff** that
  isolates them reproduced robustly.

Audit run: workflow `wf_d30766bc-bd9` (13 agents). One investigator (docs)
returned a stub on the first pass and was re-run as a focused follow-up; the
`audit-debug-fields` red state and the featured docs findings were
orchestrator-verified directly.
