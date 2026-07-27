# ADR 0001: The frame-1706 raster seed is only reachable by the persistent march

Status: accepted
Date: 2026-07-27
Scope: #406 (persistent CPU/VDP/GEMS timing), #409 (frame 1709)

## Context

Frame 1706's HUD Diamond raster needs a seed of `{beam, refresh_phase}` at
`sub_E12`. That seed is cycle-continuous from boot: `vdp.rs` states it can come
only from "a persistent CPU/VDP scheduler or a bounded hardware checkpoint".

The port currently reaches it by typing one TAS frame per session. The frontier
is frame 995 against a target of 1706 — 711 frames — on a 37,759-frame route,
and `tas_gems.rs` grew from 0 to 21k+ lines in eight days. The payoff so far is
2 of 20 focused pixels over frames 1690..1710, against 106 open sweep blockers
and three awaiting-human P0s.

Three alternatives to the march were investigated. All three are structurally
closed. They are recorded here because each is plausible enough to be proposed
again.

## Decision

Continue treating the march as the only known production path to the seed, and
make its cost measurable rather than banning or endorsing it. Re-take the
cost/value decision once the rate is visible.

## Closed alternatives

### 1. Seed the scheduler from a mid-route GEMS checkpoint

`GemsFrameSignature` is Z80/GEMS-only. It carries no 68k PC, no beam, no
refresh phase, and no pixel — none of the quantities the seed is made of. It is
structurally blind to what it would have to supply.

The failure mode is worse than "does not work": its guards would *pass*. A probe
built on it would print plausible numbers and mislead, because nothing in the
signature can contradict a wrong beam or phase.

Independently, this is a dead end already recorded four times in
`backlog-archive.md` (`:3583`, `:3856`, `:3899`, `:4161`).

### 2. Derive the cycle costs from `r68k-emu`

The port's cycle constants are jgenesis-derived. `r68k` carries Musashi's
timing table, which disagrees with jgenesis on instructions Kid executes
constantly — `MULU` is a fixed 54 in Musashi versus `38 + 2*popcount` in
jgenesis. Mixing the two tables would make every derived constant depend on
which source last touched it.

Adopting r68k also means building a general 68k timing model inside the port,
which `backlog-archive.md:773-774` rejects explicitly.

Deriving constants by reading the disassembly against the jgenesis table remains
correct and productive — see the `loc_EB4C` per-cell derivation under #406,
which corrected a fitted 48 to an ASM-read 30.

### 3. Close the six attributed frame-1706 gaps instead of marching

Two independent reasons.

**Stale.** The attribution (`backlog-archive.md:7124-7133`) was written
2026-07-26 18:31 and superseded 48 minutes later by `534f80f97`.
`Execute_Objects +498` and the `+224` suffix were re-attributed to source work,
and `post-object +18` resolved as a boundary convention.

**Circular.** Closing them buys zero pixels.
`seed_mode_level_raster_sub_e12` has no production callers, so `lib.rs` emits no
`HudRasterUploadTimingSub44DC` (`backlog-archive.md:7165-7167`, `:7178-7180`).
The gaps are gaps in a path nothing takes.

## Consequences

The march is neither banned nor blessed. Three holes that made it free are now
closed:

- `backlog-lint` rejects an open row whose `Verify` is one index past its
  `Summary` — a row that cannot close.
- `scripts/oracle_injection_ratchet.py` counts oracle clock teleports below a
  rolling window under the frontier. The window keeps the working set legal
  while advancing the frontier rolls older frames into the counted set, so the
  march pays down what it leaves behind.
- The only test covering the typed span, `gameplay_regression_persistent_clock_…`,
  now runs under `verify-slice-fast`; previously its name matched no filter the
  gate used, and there is no CI.

Revisit this ADR when the ratchet has a few sessions of data. If retiring an
injection per typed frame proves tractable, the march is paying for itself; if
each frame requires a new injection, the rate is the argument.
