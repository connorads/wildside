# ADR 0004: Fidelity floor - observable parity over invisible timing

Status: accepted
Date: 2026-10-01
Scope: supersedes ADR 0001, 0002 and 0003 in scope. Covers #406 (persistent
CPU/VDP/GEMS timing), #409 (raster recurrence), #416 (GEMS frame-signature
frontier) and the loop's intake.

## Context

The loop ran out of machine-checkable targets. The published TAS matches all
37,759 frames on state and RNG, and `project-status` showed nothing ready.
Recent sessions went to timing work no player can see: #406/#409/#416 and #300.
ADRs 0001-0003 froze those lanes but kept them as backlog rows and ratchets, so
they still framed what "remaining work" meant.

A spike measured a better fitness signal. Random controller input breaks parity
fast: 45 of 48 random 600-frame routes on idle-clean levels 0x00-0x05 diverged,
median frame ~90, into a handful of first-divergence clusters. One route proves
one route; the TAS gate says nothing about the rest of the input space. A
Go-Explore explorer also exited 11 of 17 levels (0x00-0x10) on native within two
minutes each, so "levels exited on native with the route confirmed on the ROM"
is measurable per level.

## Decision

Set a fidelity floor. The project owes the ROM:

- **In scope:** gameplay, enemy and RNG state; level exits; bugs a player can
  see or hear.
- **Out of scope, disclosed with numbers:**
  - HUD raster residue: 241 frames / 2,391 px on the published route. The
    `tas-pixel-baseline` ratchet stays as a regression guard.
  - Persistent raster frontier 995 (#406). The oracle-injection ratchet stays.
  - GEMS frame-signature frontier 922 (#416). Command parity (#435) remains the
    audio authority.
  - Loader cycle exactness.

Close #406, #409 and #416 as won't-fix under this ADR. Delete the ratchets
that only defended the out-of-scope frontiers: the #409 scheduled-frame
coverage check and the `tas-gems-baseline` frame-signature comparison.

The loop's intake becomes two measurements: `wildside fuzz run` clusters and
`wildside ledger run` blockers. The headline is N/103 levels exited on native
with the route confirmed on the ROM.

## Alternatives considered

- **Keep the frozen lanes as rows.** They never close, they keep "remaining
  work" pointing at invisible timing, and their ratchets cost gate time without
  guarding anything a player observes.
- **Delete every timing ratchet, pixel baseline included.** The pixel baseline
  and the oracle-injection ratchet are cheap and would catch a real visible
  regression, so they stay.
- **Pursue whole-route pixel exactness.** ADR 0002 priced the raster march at
  ~512 sessions for one verification projection; nothing since changed that.

## Consequences

- Oracle-is-truth still holds for everything in scope. A fuzz cluster or a
  ledger blocker is an authoritative divergence until fixed or explained.
- A visible symptom traced to an out-of-scope mechanism is in scope again: fix
  it at engine level with a regression test.
- The published TAS gates keep running. They prove one route, not the game.
- Revisit if a shipped build gains a consumer for cycle-exact raster or audio
  phase, which would change what a player can observe.
