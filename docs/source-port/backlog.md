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
- **VDP follow-up evidence:** added direct regressions for the Enigma tilemap
  wrapper, VSRAM word writes, CRAM word decoding, sub_1C204 palette fade
  progress, title-return fade phases, IntroSequence3 frame metadata,
  Make_SpriteFromGfxObject normal/flipped packing, and sprite-table register
  masking. Targeted rerun `/tmp/mutants-vdp-focused-2` reports `82 mutants
  tested in 4m: 2 missed, 79 caught, 1 unviable`. The two remaining focused
  misses are equivalent `|`→`^` pattern-attribute packers over disjoint
  palette/priority/x-flip bits. The wider `vdp.rs` shard still has uncovered
  intro/decompression/text-script survivor families for future focused passes.
  `cargo fmt --check` and
  `cargo test -q -p wildside-core --lib -- --skip demo_ --skip published_tas
  --skip snapshot` are green.
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
