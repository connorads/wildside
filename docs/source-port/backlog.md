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
- **Fallback verification:** after the render-background hardening, continued
  the rotation through clean/no-code-change shards: render `level.rs` (`41
  mutants tested in 50s: 36 caught, 5 unviable`), render `player.rs` (zero
  candidates), render `palette.rs` (`35 mutants tested in 47s: 33 caught, 2
  unviable`), render `tiles.rs` (`38 mutants tested in 48s: 34 caught, 4
  unviable`), render `hud.rs` (zero candidates), data `level_layout.rs` (`30
  mutants tested in 29s: 30 caught`), ROM `lib.rs` (`26 mutants tested in 11s:
  24 caught, 2 unviable`), and MD audio/register shard (`154 mutants tested in
  51s: 149 caught, 5 unviable`).
- **Current fix:** MD bus/VDP mutation shard
  `/tmp/mutants-md-bus-vdp-1` found `312 mutants tested in 2m: 3 missed, 305
  caught, 4 unviable`. Added a regression proving `AddressBus::copy_from`
  replays traced YM2612 ports through `port & 3`, and changed long-read
  assembly from an equivalent `|` to `+` to avoid an untestable disjoint-bit
  survivor. Final `/tmp/mutants-md-bus-vdp-2` reports `312 mutants tested in
  2m: 308 caught, 4 unviable`.
- **Current fix:** MD GEMS mutation shard followed clean controller
  (`/tmp/mutants-md-controller-1`: `78 mutants tested in 30s: 78 caught`) and
  Z80 bus (`/tmp/mutants-md-z80bus-1`: `61 mutants tested in 26s: 61 caught`)
  shards. Initial `/tmp/mutants-md-gems-1` found `91 mutants tested in 50s: 2
  missed, 86 caught, 2 unviable, 1 timeouts`. Matched GEMS bank-register
  composition to `Z80Bus`'s additive form and rewrote the frame-cycle loop to
  consume `remaining` directly with `saturating_sub`. Final
  `/tmp/mutants-md-gems-3` reports `86 mutants tested in 32s: 84 caught, 2
  unviable`.
- **Current fix:** MD VDP diagnostic render shard followed clean audio mixer
  coverage (`/tmp/mutants-md-audio-mixer-1`: `157 mutants tested in 53s: 157
  caught`). Initial `/tmp/mutants-md-vdp-render-1` found `433 mutants tested in
  2m: 20 missed, 399 caught, 14 unviable`. Added VDP sample diagnostics for
  scrolled plane provenance, exclusive window edges, low-priority window
  samples, and flipped tile source coordinates. Final
  `/tmp/mutants-md-vdp-render-2` reports `433 mutants tested in 2m: 419 caught,
  14 unviable`.
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
