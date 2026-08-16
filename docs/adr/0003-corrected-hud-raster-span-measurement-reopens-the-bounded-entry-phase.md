# ADR 0003: Corrected HUD raster-span measurement reopens the bounded entry phase

Status: accepted
Date: 2026-08-16
Scope: #409 scoped measurement (ADR 0002 §"Scoped-measurement outcome"), spike row #459
Supersedes: ADR 0002 §"Scoped-measurement outcome" only. The freeze decision,
cost case, ratchets, and recurrence ban in ADR 0002 are NOT reopened.

## Context

ADR 0002 froze the persistent-raster lane and moved the 241-frame HUD residue
to scoped measurement. Its §"Scoped-measurement outcome" recorded NO-GO for
frame-local exact reconstruction: "Replaying the cumulative 68000 span from
`$7C8` does not recover the observed `sub_E12` H40 cursor on any of the 241
frames", with wait-free witness residuals of 6 mclk (frame 1730) and 880 mclk
(frame 7698).

A 2026-08-15 audit found that verdict rests on a units bug in the measurement
instrument, not on the measured system.

## The defect

`scripts/raster_span_replay.py` added the census' H40 **pixel index** (0..419)
to an **mclk** accumulator (3,420 mclk per scanline) as if they were the same
unit. Every replayed beam position was wrong by the divider factor:

- 197 of the accepted artefact's 241 `replayed_update_hud.pixel` values were
  physically impossible (over 419 on a 420-pixel line; maximum 3413).
- Frame 7698's cited "880 mclk residual" was a replayed pixel value of 1205;
  frame 1730's "6 mclk" was equally an artefact of the same addition.
- The script's unit test locked the bug in with an impossible
  `"pixel": 3400` fixture, and no gate ran the test at all: the hk
  `verify-slice-fast` glob did not cover `scripts/**/*.py`.

`hud_timing_census.py` itself was never defective; only the replay misused
its `pixel` field. The census records (wait counts, witnesses, ASM-anchored
VRAM ranges) all stand.

## Corrected result

The instrument was fixed test-first (units pinned by real-capture fixtures,
one-cycle discrimination tests, gate wiring closed) and the whole evidence
chain regenerated from a recorded recipe (`mise run hud-timing-probe`, new in
this change; the 2026-08-11 census' probe command was never recorded). The
fresh census reproduces the 2026-08-11 wait profile exactly (204/241 frames
zero Z80 wait in both intervals; Diamond 137/170, life icon 67/71).

Under corrected interval semantics — a captured pixel index is an 8/9/10-mclk
interval under the H40 divider (`h40_scanline_mclk_to_pixel`,
wildside-core `vdp.rs`), and `cpu_span_exact` is intersection of the advanced
entry interval with the observed interval — the result inverts:

- **241/241 frames replay exactly** (residual 0): the measured cumulative
  68000 cycle span from `$7C8` entry reproduces the observed `sub_E12` cursor
  within the capture's one-pixel quantisation floor on every mismatch frame,
  asynchronous-wait frames included.
- The predicate is sharp, not loose: perturbing the measured span by +1 68000
  cycle drops exact frames to 165/241, by −1 cycle to 110/241, and by +10
  cycles to 0/241. One CPU cycle of error is visible; the 241/241 verdict is
  not an epsilon artefact.

## The bounded-entry-phase finding

The `$7C8` Mode_Level entry beam is not a boot-continuous unknown. On all 241
mismatch frames (fresh census):

- entry scanline is **224 on every frame** — one scanline into vblank;
- entry pixel is within **193..213 on 236/241 frames**, a ≤168-mclk window;
- the five outliers are pixels {215, 217, 235, 239, 251} on frames
  {11885, 13995, 14061, 33571, 33637} (full spread 464 mclk).

This is the signature of a bounded `WaitForVint` poll-exit phase, not the
boot-continuous clock the ADR 0002 freeze was priced against. What varies
frame to frame is a small phase inside one vblank scanline.

## What this reopens

Only the measurement question. The corrected result shows the frame-local
span replay is exact once the entry beam is known; the open question is
whether the entry phase itself is derivable frame-locally from typed,
ASM-cited terms. That question is scoped as spike row **#459**: a GO/NO-GO
analysis (slot-uniqueness propagation through the engine's own commit chain,
and poll-exit decomposition into disassembly-cited terms), with written
analysis and backlog updates as its only deliverable — no engine code, no
renderer change.

## What this does NOT reopen

- **The freeze.** ADR 0002's decision §1 stands: the persistent recurrence
  stays frozen, downward-ratcheted (scheduled-frames 736 / boundary 1657),
  and is not loop work.
- **The cost case.** Eleven recurrence sessions bought zero pixels; nothing
  here resumes the per-frame march.
- **The ratchets.** 241 frames / 2,391 pixels remain authoritative, unmasked,
  downward-ratcheted debt until an engine-level fix lands through the normal
  gates.
- **The promotion rules.** Fitted constants and oracle-derived corrections
  remain forbidden; any future fix must cite ASM and pass the full sweep.

## Consequences

- ADR 0002's §"Scoped-measurement outcome" is superseded by this record; its
  prose is left intact with a pointer here (records are superseded, never
  edited into changelogs of themselves).
- The measurement instrument now lives in gated, tested code: the replay's
  units are pinned by real-capture and ±1-cycle discrimination tests that run
  in `verify-slice-fast`, the probe recipe is a checked-in mise task, and the
  replay tool is ported to Rust against the engine's own divider so the
  constants cannot be re-derived wrongly again.
- Evidence provenance for this record: mismatch report regenerated by
  `verify-tas-full` on 2026-08-16 (241 frames / 2,391 px, matching the
  enforced baseline); full-span trace-only probe ~67s wall with
  `--no-frame-captures`; census and replay regenerated from that probe.
