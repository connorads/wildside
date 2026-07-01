# Playtest Backlog

Canonical tracker for bugs, missing features, and verification gaps. See `CLAUDE.md`
for iteration protocol, `docs/source-port/methodology.md` for the investigation
playbook, and `docs/source-port/roadmap.md` for the programme-level workstreams.

Important: a green TAS frontier or a green `1p-accurate` release artefact does **not**
mean the game is nearly finished. Native-play breakage, wider verification gaps, and
documented deviations belong here too.

Progress language uses the evidence ladder from `methodology.md`:
`stubbed`, `boots`, `replay-matched`, `branch-matched`, `mechanic-swept`, `human-play-stable`.
A closed TAS frontier proves `replay-matched` on that route, not `human-play-stable`.

Status: `[ ]` open, `[~]` in progress, `[?]` agent proposes fix complete — **human must validate before closing**, `[x]` done, `[-]` won't fix / not a bug, `[b]` blocked / needs human repro
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

`[?]` items carry an evidence block recording the investigation outcome: repro
scenario path, ASM labels cited, oracle-parity result, and the proposed-fix
commit hash. Only a human may promote `[?]` → `[x]`; the agent lane never
auto-closes. If investigation stalls, write the dead end (rejected hypotheses
included) under the issue's `#NNN` in `backlog-archive.md` and leave status at
`[~]`. The stop rule comes from `loop.md`: no code commit ⇒ write the dead end
and stop — never commit churn.

## NOW

- **Select work with `mise run project-status`** (priority-ordered; take its top ready/open item, do not eyeball file order).
- **2026-07-01 loop handoff:** no ready/open agent rows remain; only `[?]`/`[b]`
  human validation/repro blockers are listed by `mise run project-status`.
- **Fallback verification:** `crates/wildside-core/src/vdp.rs`
  mutation-hardening pass started from
  `OUTPUT=.cache/mutants/vdp-current FILE=crates/wildside-core/src/vdp.rs
  mise run mutants-file`, which reported `800 mutants tested in 48m: 98
  missed, 286 caught, 408 unviable, 8 timeouts`.
- **VDP follow-up evidence:** added direct regressions for
  `DecompressToVRAM loc_14300/loc_14314` byte placement across non-zero VRAM
  bases, rollover chunks, and odd tails, plus the exact first-chunk boundary in
  `decompress_chunk_spans`. Focused rerun `/tmp/mutants-vdp-decompress-2`
  reports `37 mutants tested in 4m: 1 missed, 30 caught, 1 unviable, 5
  timeouts`. The missed mutant is equivalent in this wrapper: an oversized DBF
  count on resumed chunks is capped by the actual buffered words, so the same
  bytes are written. The timeouts are malformed chunk-span progress mutations.
- **Current clean VDP shards:** fresh focused reruns found no missed mutants in
  special art remapping (`/tmp/mutants-vdp-special-decompress-1`: `13 mutants
  tested in 2m: 13 caught`), IntroSequence5 particles
  (`/tmp/mutants-vdp-intro5-particles-1`: `36 mutants tested in 3m: 36 caught`),
  the Enigma decode wrapper (`/tmp/mutants-vdp-enigma-decode-1`: `3 mutants
  tested in 61s: 3 caught`), CRAM word extraction (`/tmp/mutants-vdp-cram-words-1`:
  `6 mutants tested in 73s: 6 caught`), IntroSequence5 fade timing
  (`/tmp/mutants-vdp-intro5-fade-1`: `10 mutants tested in 81s: 9 caught, 1
  unviable`), and IntroText script replay (`/tmp/mutants-vdp-intro-text-script-1`:
  `36 mutants tested in 3m: 32 caught, 1 unviable, 3 timeouts`). The text-script
  timeouts are malformed parser-progress mutations.
- **VDP intro handoff hardening:** focused shard
  `/tmp/mutants-vdp-intro-handoff-1` first reported `59 mutants tested in 3m: 6
  missed, 50 caught, 3 unviable` across cold-boot/IntroSequence2 handoff
  timing. Cleaned the unreachable post-dispatch segment-1 overlay, made the
  palette fade windows explicit half-open ranges, rewrote the `loc_1BA02` reveal
  clamp without the equivalent `min(frames - 1) + 1` form, and added a
  `loc_12EDE` cold-boot Sega handoff regression. Rerun
  `/tmp/mutants-vdp-intro-handoff-2` reports `50 mutants tested in 3m: 1 missed,
  46 caught, 3 unviable`; the remaining `handoff + elapsed` -> `handoff -
  elapsed` survivor is equivalent for this slice because the Sega VDP state is
  already stable by the handoff window.
- **VDP title-screen sprite hardening:** focused shard
  `/tmp/mutants-vdp-title-screen-1` reported `42 mutants tested in 3m: 6
  missed, 34 caught, 2 unviable` across direct title-return sprite timing, the
  KID title sprite, and menu cursor rows. Added ASM-cited regressions for
  `loc_1BA60`/`loc_1BB10` first-frame suppression, `loc_1B5E4` KID title sprite
  presence, and `loc_1BB52` cursor rows 0/1. Rerun
  `/tmp/mutants-vdp-title-screen-2` reports `42 mutants tested in 3m: 40 caught,
  2 unviable`. Checked `/tmp/mutants-vdp-intro3-sprite-frame-1` (`17 mutants
  tested in 2m: 15 caught, 2 unviable`) and `/tmp/mutants-vdp-sprite-table-1`
  (`41 mutants tested in 3m: 2 missed, 39 caught`); the sprite-table misses are
  equivalent for valid hardware attributes because palette, priority, and
  x-flip bits are disjoint in the game domain.
  `cargo fmt --check` and
  `cargo test -q -p wildside-core vdp::tests -- --nocapture` are green;
  `mise run verify-slice-fast` passes.
- **Next selector output:** run `mise run project-status` after commit; if no ready/open risks remain,
  use the proactive-verification fallback in `loop.md`.

NOW is the current-state handoff: rewrite it at the end of every loop session, never
append. Old handoffs go to [`session-log.md`](./session-log.md).

For newly opened items, capture these facts in the investigation notes:

- evidence source: playtest, recording slug, oracle, or frame capture
- native repro status: reproduced / not yet reproduced
- oracle repro status: authoritative / advisory / unknown / oracle limitation
- minimum gate: `fast-slice`, `1p-playable`, `1p-accurate`, or `visual-parity`

> **This file is the open queue only** (`[ ]`/`[~]` rows); keep Verify cells short.
> `[?]`/`[b]` rows live in [`backlog-awaiting-human.md`](./backlog-awaiting-human.md);
> resolved rows and all per-issue evidence live in
> [`backlog-archive.md`](./backlog-archive.md) — grep it by `#NNN` (e.g.
> `grep -n '#238' backlog-archive.md`) for an issue's full history. Select work with
> `mise run project-status` (priority-ordered), not by file order. When a row leaves the
> open lane, move it to its lane and append closing evidence to the archive under `#NNN`.

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
