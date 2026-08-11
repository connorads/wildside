# ADR 0002: Freeze the persistent-raster lane; supersede it with scoped measurement

Status: accepted
Date: 2026-08-01
Scope: #406 (persistent CPU/VDP/GEMS timing), #409 (raster recurrence), the
241-frame HUD pixel residue

## Context

The whole remaining visual debt on the published route is two HUD icon boxes:
HUD Diamond (170 frames / 1,651 px, route span 1730..35,929) and the life icon
(71 frames / 740 px, span 3066..33,862) - 2,391 px total, 8.9×10⁻⁷ of route
pixels, already measured, clustered, and ratcheted by `tas-pixel-baseline`.

The lane that pursues them is the persistent raster recurrence (#409): a
cycle-continuous clock from boot whose scheduled region is extended session by
session. Measured velocity: the #406 per-frame march ran at ~1 frame/session;
the #409 recurrence averaged 67 frames/session over 11 sessions — but 82% of
that came from three sessions inside one contiguous, deliberately repetitive
Cyclone-flight segment on one level. The divergent frames run to 35,929, so
closing the cluster via the clock means ~34,000 further frames across 105
levels, every enemy family and transform — ~512 sessions at the measured mean,
with shape novelty per frame rising, not falling.

Two facts sharpen the cost/value call ADR 0001 deferred:

- **Eleven recurrence sessions bought zero pixels.** Every handoff in that run
  repeats "Full pixels remain 37,518 exact / 241 divergent / 2,457 changed".
- **The payoff surface is one verification projection.** The recurrence is
  consumed only by the `verify-tas` pixel pass (`tas_pixels.rs`);
  `native_player` and `wildside-web` step the runtime with `|_| None`
  (`lib.rs:3680`), and `seed_mode_level_raster_sub_e12` still has no non-test
  caller. The shipped game never receives a seed either way.

The repo has a completed precedent for this exact failure shape: #416 froze
the GEMS per-frame signature march at `modelled_frames=923` with the debt kept
counted, visible, and ratcheted downward-only, and moved authority to a
narrower causally-authoritative gate (`gems_command_parity_v1`). #437 then
closed the "is the engine defective or just phase-sampled?" question with a
bounded-span replay under oracle-captured stimuli — explicitly a measurement,
not a CI gate.

## Decision

Apply the #416 precedent to the visual lane.

1. **Freeze the recurrence frontier.** Do not take "extend the recurrence one
   more segment" as loop work. The recurrence becomes a downward-only
   regression witness: `REQUIRED_SCHEDULED_FRAMES_*` in `tas_gems.rs` ratchets
   the scheduled-frame count (a lower count blocks; a higher one is a
   deliberate, reviewed update), and `tas-pixel-baseline` continues to ratchet
   pixels. Nothing is reclassified or masked: the residue stays scoped as
   **HUD-icon DMA-entry-phase debt, 2,457 px, ratcheted**.
2. **Authority for the residue moves to frame-local measurement:**
   - (a) *Census-scoped frame-local exact reconstruction.* The route-wide
     census proved 137 of the 170 Diamond frames have no Z80 wait in either
     interval (Mode entry → `sub_E12` → `sub_44DC`), refresh phase is
     frame-local by hardware reset (`loc_30104`), and the VINT-anchored beam
     seed exists in production. Reconstruct exactly on frames the census
     proves wait-free; return `None` everywhere else. First step: extend
     `scripts/hud_timing_census.py` (which hard-codes the `hud_diamond`
     cluster) to the 71 life-icon frames.
   - (b) *A `raster-span-replay` measurement tool per the #437 precedent:*
     anchor at the per-frame hardware reset, replay the oracle's captured
     stimuli over the bounded span, and report measured debt. Deliberately
     NOT a CI gate, positioned exactly as `gems-span-replay`.

## Alternatives considered

- **Continue the march/recurrence to frame 35,929.** ~512 sessions at the
  measured mean for one verification projection; rejected on the #416 cost
  argument (the GEMS analogue was "~18 min/frame … ≈ a year of agent time").
- **Frame-local reconstruction applied globally.** Already rejected twice
  (stateless prefix as renderer fallback worsened the life icon 71→99;
  a VINT-seeded `wildside-frame` timeline "would silently assume zero
  asynchronous waits"). Both rejections are *globally* scoped; the census
  makes the wait-free subset provable per frame, which is what (a) scopes to.
- **Seeding shortcuts** (mid-route GEMS checkpoint, r68k cycle table): closed
  in ADR 0001 and unchanged by this decision.

## Consequences

- #406 stays metered exactly as ADR 0001 left it (frontier 995, injection
  ratchet); #409's recurrence keeps its current proven boundary and its
  ratchet, and stops accruing sessions.
- The oracle-is-truth principle is preserved the same way #416 preserved it:
  every divergent frame stays an authoritative divergence in the counts; what
  changes is scope and the tool that owns the residue.
- If (a)+(b) close the wait-free subset and the replay bounds the remainder to
  phase-sampling residue, the cluster resolves without the clock. If they
  instead prove a native timing defect, that defect lands as an ordinary
  engine-level fix with a regression test — the freeze does not protect it.
- Revisit if a production consumer for the persistent seed appears (e.g. a
  raster-accurate renderer path in the playable build), which would change the
  payoff surface this decision is priced against.

## Scoped-measurement outcome

The 2026-08-11 bounded spike completed both decision actions without promoting
a new implementation lane. The census now covers both clusters by default:
204/241 frames have zero Z80 wait in both measured intervals (Diamond 137/170;
life icon 67/71). `raster-span-replay` selected the three declared witnesses:
first wait-free Diamond frame 1730, first asynchronous-wait Diamond frame 1736,
and worst life-icon frame 7698 (15 changed pixels).

The result is NO-GO for frame-local exact reconstruction under this ADR's
constraints. Replaying the cumulative 68000 span from `$7C8` does not recover
the observed `sub_E12` H40 cursor on any of the 241 frames. The two wait-free
witnesses still differ by 6 mclk (1730) and 880 mclk (7698), so zero Z80 waits
does not prove that scheduler/VDP phase is absent. Frame 1730 also reaches the
already-recorded untyped `loc_103B2` row-advance callbacks, while frame 1736
adds 1,122 asynchronous Z80-wait cycles before `sub_44DC`. Closing those gaps
would require the persistent clock, new source-typed ownership work, or an
oracle-derived correction. The first remains frozen and the other two violate
the spike's promotion rules. The 241 frames and 2,391 pixels therefore remain
authoritative, unmasked, downward-ratcheted debt.
