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

## 2026-06-30 status update

This audit is still useful as historical context, but several findings below
were fixed after it was written.

- Current `asm-coverage` reports **3854 reachable / 242 test-only debt / 21
  uncovered**, with all 21 uncovered labels classified as **emulated GEMS** and
  **0 ordinary uncovered** labels.
- Legacy citation presence is now **sub_ 291/293 (99.3%)** and **loc_
  3805/3824 (99.5%)**.
- The scanner now reads `crates/wildside-core/src`, `crates/wildside-data/src`,
  and `crates/wildside-md/src`. `wildside-cli`, `wildside-frame`, and
  `wildside-render` remain outside the metric unless they are added explicitly.
- `verify-slice-fast` now runs the ASM coverage required-label gate, the
  test-only debt ratchet, `audit-debug-fields`, and `backlog-lint`.
- The only remaining uncovered `sub_` labels are `sub_E133C` and `sub_E155A`.
  `sub_E133C` is a GEMS open/close wrapper (`jmp loc_E1576`, which opens and
  closes the Z80 bus without a command). `sub_E155A` queues `FF 0C`; it is not
  `stop_all_sequences`, which queues `FF 16 FF`. Keep both uncited unless native
  code models those exact wrappers.

---

## Verdict

**Is coverage 100%?** No — and "coverage" does not mean what the question
implies.

- As of 2026-06-30, `wildside asm-coverage` reports **sub_ 291/293 (99.3%)**
  and **loc_ 3805/3824 (99.5%)**, not 100%.
- The metric counts **citation presence**, not fidelity. The tool says so
  itself: `asm_coverage.rs:771-773` prints *"label citation tracks
  recovered/discussed ASM, not player-visible fidelity"*. A single one-line
  `// ASM loc_XXXX` comment in a scanned crate marks a label "covered" with zero
  verification — the metric is author-attestable, not earned.

**Is it wired in?** Locally, yes for the source-port slice gate:
`verify-slice-fast` now runs `asm-coverage --check-required`,
`asm-coverage --check-no-new-test-only`, `audit-debug-fields`, and
`backlog-lint`. This audit does not re-check remote CI wiring.

**Are the 21 uncovered labels real gaps?** No genuine gameplay porting gaps.
They are the deliberately-emulated GEMS sound-driver wrappers that remain
outside native hand-porting. See [Uncovered-label triage](#d-uncovered-label-triage).

The good news: the **denominator is currently complete and correct** (4117
labels = 293 sub_ + 3824 loc_, reproduced exactly), the output is untracked
`.cache` state (deliberately, #38), and the tool already models a richer
`Mentioned/Implemented/Tested/OracleBacked` ladder than the headline exposes.
The problem is not dishonesty — the tool disclaims itself accurately — it is
that the headline is a weak signal, the local gate only protects selected
regressions and test-only debt, and the docs around it can still overclaim.

---

## How the metric actually works

`run_asm_coverage` (`crates/wildside-cli/src/asm_coverage.rs:677`):

1. **Denominator** — `extract_all_asm_labels` (lines 124-140) reads `kid.asm` +
   `code/**/*.asm`, takes each line that (trim-left) starts `sub_`/`loc_`, up to
   the first `:`, where the suffix is all hex. → 4117 labels.
2. **Numerator** — `extract_native_citations` (lines 163-175) scans
   `crates/wildside-core/src`, `crates/wildside-data/src`, and
   `crates/wildside-md/src` for `sub_`/`loc_` tokens; `build_coverage_map`
   (256-284) marks a label covered iff its exact token string appears in that
   set.
3. The richer per-label state (`coverage_state`, `implementation_cited`,
   `test_cited`, `oracle_backed`, `citation_breakdown`) is overlaid only in the
   untracked `.cache/asm-coverage.yaml`, not in the headline.

This is a one-directional, exact-string scan. Every limitation below follows
from those properties.

---

## Findings

Severity: **High** = misleads or hides regressions / red on main; **Medium** =
material accuracy gap; **Low** = latent or cosmetic; **Info** = confirmed
characterisation.

### A. The headline number

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| A1 | Info | Headline is **99.3% sub_ / 99.5% loc_**, not 100%; it is a citation-presence signal, self-disclaimed as not fidelity. | `asm_coverage.rs:256-284, 771-773`; reproduced via binary and `cargo run -q -p wildside-cli -- asm-coverage --uncovered` on 2026-06-30. |
| A2 | Medium | **Gameable**: one `// ASM loc_XXXX` comment in core/data instantly marks a label covered, no verification. The authors of the comments determine the %. | `build_coverage_map` 256-262 (presence test only). |
| A3 | Low | **Scanner scope now includes `wildside-md`**, removing the previous false-negative GEMS citations. `wildside-cli`, `wildside-frame`, and `wildside-render` remain out of scope. | `asm_coverage.rs:165-168`; current sub coverage 291/293. |
| A4 | Low | **Case-sensitive** match vs all-uppercase disasm labels; a lowercase-only citation (e.g. `loc_1002e`) would not credit `loc_1002E`. Net-zero false negatives today (every label also has an uppercase citation), but latent. | `asm_coverage.rs:204-207, 257`; 2412 tokens cited in non-canonical case. |
| A5 | Low | **Inline `#[cfg(test)]` misclassified as implementation** — `classify_citations` buckets by path only (`/tests.rs` or `/tests/`), and `cited_in` is per-file, so inline test modules (the dominant pattern; 45 files) read as `Implemented` not `Tested`. Affects the YAML state ladder, not the headline. | `asm_coverage.rs:494-511, 207`; e.g. `loc_1AB6A` test-only in `movement.rs`. |
| A6 | Medium | **77% of labels collapse into one `kid_main` bucket** — `classify_subsystem` only separates `code/enemy/*`; VDP/DMA/sound/level/gameflow all live in `kid.asm` and cannot be reported per-system. | `asm_coverage.rs:219-246`; `code/` has only `enemy/`. |
| A7 | Low | **Denominator is complete today but fragile** — only `kid.asm` + `code/**` are read; `level/`, `ingame/`, `scenes/`, `constants/variables/macros.asm` define **zero** `sub_`/`loc_` labels now, so nothing is missed — but a future labelled routine under those dirs would silently drop out and inflate the %. | `asm_coverage.rs:124-140`; those dirs grep to 0 label defs. |

### B. Enforcement / wiring

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| B1 | Info | **Resolved for local slice verification** — `verify-slice-fast` runs `asm-coverage --check-required` and `asm-coverage --check-no-new-test-only`. This audit does not re-check remote CI wiring. | `.mise.toml`; 2026-06-30 `verify-slice-fast`. |
| B2 | Info | **Resolved for local slice verification** — the required-label regression gate is no longer dormant in `verify-slice-fast`. | `asm_coverage.rs:592-633, 812-838`; `.mise.toml`. |
| B3 | Medium | The original synthetic-test criticism is stale: there is now a real-repo `asm_coverage::check_required_labels_are_covered_in_real_repo` test, and the CLI gate runs the required/test-only checks. Remaining risk is that the headline still mostly measures citation presence. | `asm_coverage.rs`; `.mise.toml`. |
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

21 genuinely-uncovered labels (2 sub_ + 19 loc_); **0 are gameplay porting
gaps.**

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| D1 | Info | **21 are the remaining GEMS sound-driver M68k↔Z80 interface labels** (`sub_E133C..loc_E15E8`, kid.asm ~49508-49910): Z80 bus arbitration wrappers and mailbox primitives not modelled as native Rust behaviours. `wildside-md::GemsAudioEngine` now cites the modelled driver load/reset/FIFO marker/start/stop/tempo helpers. The remaining `sub_` labels are `sub_E133C` (open/close wrapper with no command) and `sub_E155A` (queues `FF 0C` only). | kid.asm:49508-49910; `doc/sound.txt:1`; `wildside-md/src/lib.rs`; 2026-06-30 `asm-coverage --uncovered`. |
| D2 | Info | **0 non-GEMS loc_ citation-granularity gaps remain.** The parser credits only exact label-string matches, so a parent being cited did not credit its internal branch targets. Former gaps `loc_1F56`, `loc_8370/837C`, `loc_8456/8460/8470`, and `loc_349C0` are now cited inline; the `sub_8446` labels are explicitly marked as visual-only `off_84A2` sprite-table TODO debt rather than completed behaviour. | `--json`; parents cited in combat/movement/platforms; `combat.rs:14022`; `platforms.rs` stored-platform carry notes; `movement.rs` sub_8446 TODO; `dragon.rs` delete-tail note. |

### E. Debug-field registry (reference-accuracy adjacent)

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| E1 | Info | **Resolved** — `audit-debug-fields` is green on 2026-06-30, with only existing candidate-staleness warnings. | `mise run audit-debug-fields`; `verify-slice-fast`. |
| E2 | Info | **Resolved for local slice verification** — `audit-debug-fields` now runs in `verify-slice-fast`. | `.mise.toml`. |
| E3 | High | **Scanner blind spot** — `discover_emitted_keys` globs `crates/**/*.rs` + `scripts/**/*.py`, missing the `include_str!`'d oracle template `oracle_helper_main.rs.txt`, which emits unregistered keys (`pause_menu_option`, `options_speed_button_fast`) that are invisible to the audit. The "no unregistered keys" guarantee is hollow for oracle-side instrumentation. | `audit_debug_fields.py:14-17`; `oracle.rs:16`; `oracle_helper_main.rs.txt:1206-1227`. |
| E4 | Medium | **Candidate staleness ratchet is defeated** — all 3 candidate fields evade `fail_after`: one has an invalid git hash (`c61bd34cd`, not a real object), one is `pending`, one is permanently exempted. A `None` distance only warns, never errors. | `audit_debug_fields.py:43-58,113-128`; registry candidate blocks; `git cat-file -t c61bd34cd` fails. |
| E5 | Low | `candidate_promotions` is `[]` (auto-maturity machinery dead); 41/85 fields have `first_added_commit: pending`; no registered-minus-emitted check, so dead registry entries (85 reg vs 71 emitted) accumulate. Note: the "audit" mutates the tracked registry when `candidate_promotions` is non-empty — a surprising write side-effect. | `audit_debug_fields.py:71-90,105-107,140-143`; registry. |

### F. Documentation consistency

| # | Sev | Finding | Evidence |
|---|-----|---------|----------|
| F1 | Info | **Resolved/stale** — the ASM→Rust map in `m68k-kid-asm/SKILL.md` now points at `wildside-core` and `wildside-frame/src/sprites.rs`. | `.agents/skills/m68k-kid-asm/SKILL.md:591-598`; `ls crates/`. |
| F2 | Info | **Resolved/stale** — `research-map.md` now frames enemy coverage as a discovery aid only and states that listed fixes were closed by fixes and regressions, not by the coverage number. | `research-map.md:127-143`. |
| F3 | Info | **Resolved/stale** — the citation-not-fidelity caveat now appears in `AGENTS.md` and `research-map.md`, and `loop.md` already downgrades coverage to a discovery aid. | `AGENTS.md:133-140`; `research-map.md:127-143`; `loop.md:57-59`. |
| F4 | Low | The source-port docs should describe the current `--check-required` / `--check-no-new-test-only` gate and the untracked yaml explicitly; CLAUDE.md duplicates the asm-coverage command block (125-127 and 218-220), both historically lagged the real flag set. | CLAUDE.md; `main.rs:1175-1193`. |
| F5 | Low | Stale helper refs in `audits/audio-triggers-fidelity-audit.md:97` (`sound_driver_write_command_sub_E1482` etc., `wildside-core/src/lib.rs:1019-1057`) — those symbols no longer exist anywhere; the cited line ranges now hold unrelated code. GEMS plumbing moved to `wildside-md::GemsAudioEngine`. | grep returns 0; ranges repurposed. |
| F6 | Info | **Resolved/stale** — the failure message now refers to scanned non-test implementation citations, and the scanner includes `wildside-md`. | `asm_coverage.rs`; 2026-06-30 gate output. |

---

## Proposed backlog (prioritised, de-duplicated)

These are remaining proposals for human review after the 2026-06-30 coverage and
gate follow-ups. Effort: S/M/L.

### P1 — actively misleading

| Title | Effort | Refs |
|-------|--------|------|
| Close the `audit-debug-fields` scanner blind spot (`*.rs.txt` / oracle template) and register the now-visible keys | M | E3 |

### P2 — make the metric accurate and enforced

| Title | Effort | Refs |
|-------|--------|------|
| Decide whether to widen the citation scanner to cli/frame/render, or keep the production engine metric limited to core/data/md | S | A3 |
| Add a reverse "citation resolves to a real ROM address" lint using the named-anchor / in-range / gap-cliff heuristic (NOT naive label-must-exist) | M | C1, C3, C4 |
| Make candidate-staleness un-bypassable: re-derive bad/`pending` hashes; treat `candidate` + `None` distance as an error, not a warning | M | E4 |
| Document `--check-required`, `--check-no-new-test-only`, and the untracked yaml; de-duplicate the CLAUDE.md command block | S | B5, F4 |

### P3 — tidy, hardening, content fixes

| Title | Effort | Refs |
|-------|--------|------|
| Decide whether the remaining emulated GEMS wrappers should stay as a visible uncovered bucket or move to an explicit out-of-scope bucket | S | D1 |
| Add a downward-only coverage ratchet (tracked baseline like `dead-code-ratchet`) so any drop in cited-label count is a red gate | M | B4 |
| Track *named* disasm labels (3821 in kid.asm) as first-class citeable anchors | L | C2 |
| Derive a separate "verified fidelity" metric from `oracle_backed`/`parity_evidence` alongside the citation-presence number | M | A2 |
| Case-insensitive matching or a canonical-uppercase-hex citation lint | S | A4 |
| Per-region test classification for inline `#[cfg(test)]` citations | M | A5 |
| Per-system breakdown of `kid.asm` (VDP/DMA/sound/level) via an address-range map | L | A6 |
| Fix the 2 genuine stale citations `loc_2510C` (combat.rs:179) and `loc_23629` (native_player/mod.rs:2086) | S | C3 |
| Refresh stale `audio-triggers-fidelity-audit.md:97` helper refs | S | F5 |
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
