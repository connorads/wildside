# ASM-Coverage Remediation Plan — 2026-06-28

Companion to [`asm-coverage-audit.md`](./asm-coverage-audit.md). The audit found the
coverage infrastructure dormant, gameable, and self-disclaimed. This plan records the
**decisions** taken (via a design grill) about what to do, why, and in what order, plus what
was deliberately *not* done.

This doc is the ADR-equivalent for the work (the repo has no formal ADR dir; decisions live
in `docs/source-port/` + commit messages). Backlog rows #364–#368 reference it.

## The reframe (why this is not the audit's backlog verbatim)

The audit's headline question was "is coverage 100%?". The decision that reshaped the work
came from a sharper question: **does a citation mean the routine is actually used in the
shipped game, or did we just write a test against an assembly routine we never wired in?**

Fresh investigation (2026-06-28, line-aware, tokenizer-validated against `blocks.rs`) found
the audit *undersold* its own finding A5 ("test/impl misclassification", filed Low/cosmetic):

- **289 of 4077 covered labels (7.1%) are cited only in `#[cfg(test)]` context** — the
  audit's file-level "7" missed them because the dominant pattern is inline
  `#[cfg(test)]` modules *inside* production `.rs` files.
- **~224 `pub(crate)` functions are reference-ports of ROM routines gated `#[cfg(test)]`**
  (whole families: `animated_solid_block_*`, `allocate_object_slot_*`, `decompress_*`).
  They read like production implementations and the metric counts them as `Implemented`, but
  they never compile into the game.
- **19 of 293 subroutines have zero production citation**: 14 are GEMS (legitimately
  emulated by `wildside-md` — fine), leaving **5 genuine orphans**: `sub_96E8`, `sub_14582`,
  `sub_144DA`, `sub_1FA2`, `sub_3F366`.
- Confirmed orphan example: `juggernaut_wall_scan_sub_96E8` (`terrain.rs:307`) is a clean,
  fully-tested `#[cfg(test)]` port with **no production caller** and **no differential
  comparison** to a shipped path — it tests itself. The game never runs it.

So "covered" today conflates three different things. The plan makes them distinct.

## Decisions

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| 1 | What the metric means | Three buckets — **reachable** / **test-only debt** / **uncovered** — with explicit explained exceptions. Not one citation %. | A single % hides the 289 test-only ports as if they were shipped behaviour. |
| 2 | "Reachable" line | Cited in **non-`cfg(test)` code** in the scanned crates. | `-D warnings`/`dead_code` already guarantees non-test code is actually called, so this cheap proxy ≈ true call-graph reachability for internal `pub(crate)` ports, without call-graph tooling. The only escape is crate-`pub` exports (few). |
| 3 | Test-only ports | Tracked as **fidelity debt**, not counted as coverage. | They are real (clean ports + tests) but not shipped; deleting them loses fidelity work, counting them lies. |
| 4 | Rollout | **Ratchet**: grandfather the ~289 in a thin baseline, fail the gate on anything *new*, burn down opportunistically. **Triage the 5 orphan subroutines now.** | Matches the project's existing ratchet campaign + `dead-code-ratchet`; immediate honesty on worst offenders without a 224-fn slog. |
| 5 | Tag vocabulary | Inline markers — `// TODO(fidelity): wire into <path>` (WIP, pending integration) and a new `// DEAD_ROM: <why>` (genuinely not in this game) — plus GEMS as an "emulated" bucket. Thin baseline file for the gated set. | Rationale next to code (greppable, reviewable in diff); avoids a second source of truth that drifts (the debug-field-registry already shows drift: 85 registered vs 71 emitted). |
| 6 | Enforcement | Cheap high-value subset into `verify-slice-fast` deps: (a) test-only-port ratchet, (b) `audit-debug-fields` (after the E1 red fix), (c) `asm-coverage --check-required`. Reverse-citation lint = one-off, not a gate. | Answers the audit's "dormant / wired into nothing" without over-investing teeth in a proxy the tool itself disclaims. |
| ⚠ | Detector robustness | The line-aware test-context detector **must be tokenizer-grade** (handle `//`, `/* */`, char/string/raw-string literals). | A throwaway brace-counter broke twice during the grill (a 51.8% phantom; an early `#[cfg(test)]` helper in `blocks.rs` swallowing ~5000 lines of production code). A flaky detector churns the ratchet baseline and resurrects exactly the distrust the audit documents. |

### Honest headline shape (target)

Under decision 2, the summary should read roughly **~3788 reachable / 289 test-only-debt /
40 uncovered** (of the 40, ~33 are emulated GEMS) — not "95.2% / 99.3%".

## Build sequence (backlog rows)

Dependency-ordered; each is an independently-pickable loop item.

1. **#364 — tokenizer-grade test-context detector** (fixes audit A5 properly). Foundation.
   Verify: unit tests incl. the `blocks.rs`-style trap (early `#[cfg(test)]` helper +
   char-literal `'{'`) — the detector must classify `blocks.rs:1692/4675` as production and
   `blocks.rs:5700` as test.
2. **#365 — 3-way classification** (reachable / test-only / uncovered) + GEMS "emulated"
   bucket; surface in the headline, `--json`, and the YAML. Depends on #364.
3. **#366 — orphan triage + ratchet** : tag the 5 orphan subroutines now (`// TODO(fidelity):
   wire into <path>` or `// DEAD_ROM: <why>`); snapshot the 289 as a thin grandfathered
   baseline; add `asm-coverage --check-no-new-test-only`. Depends on #365.
4. **#367 — wire the gate**: add to `verify-slice-fast` deps — the #366 ratchet,
   `audit-debug-fields` (now green, E1 done), and `asm-coverage --check-required` (B2/B3:
   back `--check-required` with a real integration test, not the synthetic `make_label`
   unit tests).
5. **#368 — one-off tidy** (not gated): fix the 2 genuine stale citations `loc_2510C`
   (`combat.rs:179`) + `loc_23629` (`native_player/mod.rs:2086`, a kid.asm line number
   misused as an address) (C3); reconcile the gate-failure message "wildside-cli" →
   core/data (F6); refresh stale `audio-triggers-fidelity-audit.md:97` helper refs (F5);
   de-duplicate the CLAUDE.md asm-coverage block and document `--check-required` (F4).

## Done this session (2026-06-28)

- **E1 (was red on main)** — registered `fire_demon_palette_phase_loc_3a4c2` /
  `fire_demon_palette_source_offset_loc_3a4d6` in `debug-field-registry.yaml` (exploratory,
  same family as the Heady-Metal palette siblings). `audit-debug-fields` now exits 0.
- **F1** — fixed the stale `kid-core`/`kidcli` crate paths in `m68k-kid-asm/SKILL.md`
  (→ `wildside-core`; sprites → `wildside-frame/src/sprites.rs`).
- **F2/F3** — rewrote `research-map.md` "Enemy behaviour gaps" to stop equating coverage
  with fixed behaviour; added the canonical **"citation ≠ fidelity ≠ reachability"** caveat
  to `CLAUDE.md` (and a back-reference at the second command block).

## Deliberately deferred / rejected

- **Reverse-citation lint as a gate** (audit C1/C3/C4) — rejected as a gate: it catches only
  2 genuine stale citations against ~134 false positives unless it uses the named-anchor /
  in-range / gap-cliff heuristic. Do the 2 fixes one-off (#368); don't gate.
- **Widen the citation scanner to cli/frame/render/md** (audit A3/P2) — *not needed* given
  decision 5: GEMS is handled as an "emulated" bucket rather than by scanning `wildside-md`.
  Revisit only if a non-GEMS production port legitimately lives outside core/data.
- **True call-graph reachability** (vs the decision-2 proxy) — rejected as redundant with the
  `dead_code` lint and not worth the tooling fragility.
- **Debug-field-audit hardening** (audit E3 oracle-template blind spot; E4/E5 candidate
  staleness, `candidate_promotions` write side-effect) — out of the "cheap high-value"
  scope. Note: wiring `audit-debug-fields` into the gate (#367) inherits the E3 blind spot
  (the `oracle_helper_main.rs.txt` template's keys are invisible to the scanner). Track
  separately if/when oracle-side instrumentation needs the same guarantee.
- **Per-system `kid.asm` breakdown, named-label tracking, separate "verified fidelity"
  metric, case-insensitive matching** (audit A4/A6/C2/P3) — latent/cosmetic; not scheduled.

## Reproduction (this session's numbers)

```sh
# honest test-only count needs tokenizer-grade cfg(test) detection — a brace counter
# over-counts (char-literal '{') or under-counts. Validated boundary truth:
#   blocks.rs: #[cfg(test)] at line 267 (a gated helper) but real `mod tests` at 5646;
#   1692 & 4675 are PRODUCTION, 5700 is test.
DYLD_FALLBACK_LIBRARY_PATH=$PWD/target/debug ./target/debug/wildside asm-coverage --json
```
