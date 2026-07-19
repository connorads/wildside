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

Awaiting-human rows live in `backlog-awaiting-human.md` and add `Target` (`0xNN`,
`boot`, `demo`, `demo:0xNN`, `audio-device`, `recording:<slug>`, or `unknown`) plus
a short human-facing `Check` sentence so tools can show exactly what a playtester
must validate. The `Check` names the symptom and action directly; vague fallbacks like
"retest the reported route" belong in neither docs nor `project-status`.

## NOW

- **Select work with `mise run project-status`** (priority-ordered; take its top ready/open item, do not eyeball file order).
- **Source-port visual ratchet:** `1p-accurate` renders Level 0 frames 0, 30, 60, and 120 through the source-port renderer against jgenesis with exact state, no masking, MAE 0.000, and zero changed pixels. The gate now requires exact pixels.
- **Visual confidence boundary:** #375 is resolved for those four settled-warp checkpoints only. This is not cold-boot or whole-game pixel parity. The older `frame_oracle:*` steps compare wildside-md ROM VDP output with jgenesis, not source-port pixels.
- **Full published-route evidence:** all 37,759 requested player/camera CRC frames match oracle. The report names this narrow projection `player_camera_progression_v1` and declares that it proves neither pixels nor whole state. The complete RNG stream also matches on all 37,743 post-`Init_RNG` frames, including the offset and all 55 ring bytes on every frame. The gameplay and enemy-frontier passes compare every one of the 37,759 frames with zero authoritative, advisory, accumulation, RNG, or enemy-roster differences; the full checkpoint pass carries 24 evidence-backed oracle limitations (the paired `number_objects`/`number_gfx_objects` native=8/oracle=7 frames where native correctly retains the collected-diamond worker per `loc_10396`). #404 adds direct typed `Current_player` and `Two_player_flag` parity; the latter is absent from the CRC. The `source_port_visible_rgb24_v1` pass also renders and compares every route frame: 26,382 are exact and 11,377 diverge, totalling 3,620,978 changed pixel instances. This is exact CRC, RNG, exported gameplay state, and enemy-roster evidence plus complete measured visual debt on one route, not whole-RAM, audio, every-input, or whole-game equality.
- **Whole-level sweep boundary:** canonical report `1783914264` is 47/69 pass with 106 blocking differences, 3 advisory differences, and 56 oracle limitations. #402 corrects #401's coarse Tar Monster model with the real `unk_32264` cursor, 28-frame animations, `unk_316AC` glob collision box, and prior-pass collision latch. Levels 0x46 and 0x51 remain authoritative-exact. This is still two idle checkpoints per sweepable level, not full-route or every-input proof.
- **Check tightening:** insta snapshots re-homed to `src/tests/snapshots/` with real (unredacted) values; `asm-coverage --check-no-new-test-only` now fails on stale baseline entries (45 re-banked, 241 -> 196); advisory-masking prose retired from CLAUDE.md/methodology.md.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **Current work:** #406's source-ordered Kid -> wobble -> flash SAT makes frames 1368/1683 exact; the full RGB baseline remains 26,382 exact / 11,377 divergent / 3,620,978 changed pixels, with first mismatch frame 1697 in the life icon. #409 already carries the exact `sub_44DC` FIFO/post-HUD ledger to `loc_892`; frame 1709's four GEMS steals at offsets 43/96/151/192 remain the frontier. Commits through `80893ee58` add the source-derived Load_InGame cursor, selected-PC GEMS checkpoints, timed StopMusic controls, instruction-boundary wait draining, and the exact 45,253-cycle `Initialize_Platforms` hardware timeline. The published route has no GEMS bank wait during that platform call; the coarse seed is already wrong at entry. A backwards selected-PC trace finds StopMusic at frames 14/30/68/139 and Ice PlaySound at 70/171 before `$11A0E` at 868. The last reset was missing natively because IntroSequence1 and TitleScreen shared one inferred cue. Commit `fdc811fe2` now emits the `sub_1C246 -> loc_1C264` title-entry reset at shell elapsed 0, holds the reset gap, and starts Ice only when `Load_TitleScreen -> sub_E151C` is reached at elapsed 32. NEXT: schedule the frame-139 StopMusic (absolute cycle 17,789,000, refresh phase 32) and frame-171 queue command (21,961,598, phase 48) on the persistent engine, then compare `$11A0E` again before replaying `sub_44DC -> loc_892`. Never substitute a fitted halt, route/frame constant, queued-sound count, or frame-batched approximation.

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
| 406 | [~] | P1 | rendering | Source-ordered Kid → wobble → flash SAT makes frames 1368/1683 exact; frontier is #409's frame-1697 life icon. | Continue via #409; keep full TAS green. |
| 407 | [ ] | P1 | rendering | Five renderer commits after 5b6d74d88 flipped ~130 previously-exact route frames divergent (pixels still improved), so tas-pixel-baseline is red. | Attribute flips per commit, fix or re-baseline with evidence; verify-tas-full green. |
| 408 | [ ] | P1 | rendering | Field sprite/effect animation phase derives from global frame_index instead of the ROM's per-object `GfxObject_Animate` animation_timer; largest #406 cluster (est 2,000-3,500 frames: cyclone spin, prize bursts, diamond spin/sparkle). | Current RGB frontier has no live object; await an object-local mismatch. |
| 409 | [~] | P1 | rendering | Persistent master-clocked GEMS state exists; exact route seeding and the `loc_892` bridge remain. | Derive native frame-1709 steals; bridge to `loc_892`; keep full TAS green. |
| 413 | [ ] | P1 | rendering | Static per-frame Plane-B reconstruction leaks wrong tiles at seams: chunked column-upload source column (space planets) and forest water-ripple right-edge seam; ±1px scroll theory disproven. | Reopen only with a current source-column or row-history discrepancy, not the calibrated four-pixel water edge. |
| 414 | [ ] | P1 | intro/render | Intro/title/story-card overlays have independent timing/rendering residues; IntroText2's final period was one frame early, while later title/flame and pose clusters remain. | Keep each cluster source- and oracle-derived; model title VDP display timing before revisiting frame 21765. |
