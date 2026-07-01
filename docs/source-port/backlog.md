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
- **Fallback verification:** continuing `crates/wildside-core/src/vdp.rs`
  mutation hardening from the current session-log trail. The latest focused
  shard was title-card VDP:
  `/tmp/mutants-vdp-title-card-1` reported `39 mutants tested in 3m: 10
  missed, 28 caught, 1 unviable` around `Load_TitleCard` clear/fill and
  padding loops. Added ASM-cited assertions for `loc_19B58` first/second/final
  `$0094` tilemap clear words and `loc_19C04` final bottom-padding word. Final
  rerun `/tmp/mutants-vdp-title-card-3` reports `39 mutants tested in 3m: 8 missed,
  30 caught, 1 unviable`; the remaining misses are equivalent for the current
  stateless builder because `loc_19ABE` and `loc_19B6A` write zero into a fresh
  zeroed VDP. Checked `/tmp/mutants-vdp-intro-fade-1` (`52 mutants tested in
  3m: 1 missed, 48 caught, 3 unviable`) and
  `/tmp/mutants-vdp-vsram-writer-1` (`9 mutants tested in 79s: 9 caught`); the
  intro-fade survivor is equivalent because `SEGA_SCREEN_INTRO_HANDOFF_ELAPSED
  ± elapsed` is already in the stable Sega VDP range for the cold-boot
  `elapsed 0..16` window.
  `cargo fmt --check`, `cargo test -q -p wildside-core title_card_vdp --
  --nocapture`, `mise run backlog-lint`, and `mise run verify-slice-fast` pass.
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
