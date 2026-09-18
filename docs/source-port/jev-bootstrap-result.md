# Recording bootstrap investigation

The level-0 recording bootstrap must reproduce the direct title-card route
before a Jev episode can support an oracle verdict. The original pilot evidence
is retained in [the pilot report](jev-playtest-pilot-result.md).

## Measured cause

The 2026-09-18 investigation reproduced the original mismatch with a fresh
oracle run of `issue-372-forest-prize-natural-control`. Enumerating
`Init_RNG` / `Get_RandomNumber_byte` from the ROM seed located the complete
native buffer at draw 2461 and the oracle buffer at draw 2957. The 496-byte
difference is 124 `Execute_Objects` longword draws. Comparing offsets alone
made this look like a one-byte difference because the buffer has 55 slots.

The oracle reaches draw 2221 after the 720-frame boot, 2713 on title-card
dismissal, and 2953 immediately before scenario frame zero. The loader retains
the same complete buffer through handoff steps 0-68. Step 69 resumes object
execution. Kid is absent on steps 27-68, so the final 60 consecutive live
samples end at step 128.

Native's direct recording route bypassed that title/loading sequence. The
existing bootstrap witness also skipped level zero because that route has no
debug-warp load-completion entry.

RNG parity alone exposed an incomplete first correction. Preserving the old
enemy roster across `Load_InGame` moved the dragon before gameplay and broke
the scripted block hit. Reloading that roster restored reachability, but
executing it on the loader's opening pass advanced it one frame early. These
are lifecycle errors, not reasons to reseed RNG or adjust its offset.

The oracle's saved object slots identify the dragon allocation at step 69,
its `Enemy0D_Dragon_Init` execution at 70, and an animation restart after its
spawn fall at 74. The existing recorded-title animation adjustment applies at
this allocation boundary, before the stable window ages the sprite.

`loc_720` also clears the HUD animation offsets before `sub_44B0` seeds the
countdowns. Retaining the title-card animation state created extra HUD pixel
differences. A fresh frame-zero WRAM probe pins bytes `FA58..FA61` to
`06 21 22 02 01 06 00 00 00 08`; the regression checks those bytes alongside
the loader's complete RNG buffer and player/roster readiness.

The correction uses the existing title-card lifecycle and actual object
passes. A forced RNG buffer copy would hide the missing world-state work;
leaving the bootstrap unchanged would keep the pilot inconclusive.

## Repeated control

The unchanged 173-frame input script reaches the prize worker at frame 112;
the worker remains active through 142. Run `1789701204276` records:

- Exact native replay for all 173 frames.
- Exact RNG offset and complete buffer for all 173 emulator frames.
- Passing oracle state comparison at all 91 requested checkpoints.
- Zero target-crop differences in all 91 captures, including all 31 active
  wobble frames. The crop is the same 16 by 36 world-pixel rectangle at
  `(128, 176)` used in the original pilot.
- Seven full frames with differences, totalling 2,072 pixels. Frames 123,
  129 and 132 differ around the dragon's sprite; frames 154, 159, 163 and
  167 differ in the small sprite region beneath the prize row. Paired images
  were inspected. These differences are retained in the verification report.

The original control had 47 differing captures and 3,738 changed pixels.
The repeated control passes the starting-state prerequisite, but the complete
fidelity verdict remains **inconclusive**. `verify-recording` exits 1 for the
seven pixel differences. This is a scripted control, not successful Jev play,
and #372 remains awaiting human validation. No additional API calls were made.

Reproduce with the existing commands in [the pilot report](jev-playtest-pilot-result.md).
The local bundle is `.cache/jev-playtest/1789701204276/scripted-control`;
`verify/report.json`, `pixel-comparison.json` and `comparison-contact.png`
contain the comparison results. Historical bundles retain their original
traces; fresh recordings use the corrected bootstrap.

## Verification

The correction passes `verify-slice-fast` and `verify-tas-full`. The full
37,759-frame replay retains its existing state, RNG, pixel and audio baselines.
The core gate has 7,406 passing tests and the full CLI suite has no failures.
The cache-backed loader regression also passes with
`KID_ENABLE_CACHE_BACKED_TESTS=1`.

State sweeps `1789700224` and `1789701257` have identical per-level state
verdicts and differences: 69 passes, including three levels with no comparable
oracle checkpoint. The expanded bootstrap witness covers level 0, level 1 and
the existing warped-level controls. Required commit hooks pass.
