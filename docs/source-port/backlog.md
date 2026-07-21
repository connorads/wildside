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
- **Sweep-blocker census (#418):** every one of the 106 blocking diffs in historical run `1783914264` is attributed. New `sweep-census` subcommand (mise `sweep-census`; artefact `.cache/audit/sweep-census.json`) mechanically joins blocking diffs onto per-level enemy-family rosters; a 22-agent read-only attribution pass (PNGs + ASM + native code per failing level) produced clusters #419-#433: five P1 enemy clusters (Goat 16, Driller 19, Scorpion 10, UFO 7, Ninja 5 blockers), four remaining P2 single-level enemy items (MiniHoppingSkull, SpinningTwins, Hand, Tornado), one Kid-physics item (0x52), murder-wall 0x21, level-decode 0x42, and leftover-level 0x41 classification (12 blockers comparing garbage). 0x33 HeadyMetal (4 blockers) is owned by blocked #301, not re-rowed. #428's TarMonster attribution was stale after #403; fresh Level 0x64 state report `1784577126` passes. #429 now applies `loc_75C0` spawn-overlap correction to debug title-card entry; fresh Level 0x29 state report `1784577500` passes. No ready item remains; `project-status` selects the open risk lane.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **GEMS route parity pass:** `gems_frame_signature_v1` compares a per-frame GEMS/Z80 signature (8 KiB audio-RAM CRC, bank address, Z80 PC, wait ledgers, machine flags) native-vs-oracle across all 37,759 route frames, reported in `report.json`, ratcheted by `tas-gems-baseline` in `verify-tas-full`, and bisectable with `--frontier-kind gems`. Frames 0..15 now match every strict field: after the `EntryPoint` bootstrap, a resumable source cursor replays the first `StopMusic -> sub_E13DA` BUSREQ/reset edges, all 5,764 driver writes, and 2,428 tail clears without skipping either intervening `FrameRendered` sample. `coverage.modelled_frames=16`; the first mismatch is frame 16's running driver after its first VBlank/IRQ. Sampling remains pinned to oracle `FrameRendered` at scanline 232; divider phases are advisory because the oracle re-samples them at a 68000 instruction boundary. Signatures after frame 15 remain measured audio-timing debt, not claimed parity.
- **Gate hardening (#417):** the `failing-test-ratchet` hk step pins the `wildside-cli` failing-test set downward-only via `docs/source-port/failing-test-baseline.yaml` (currently 53: 52 `native_player` + 1 `inspect`). A fresh failure outside the set blocks the commit; a fix `--update`-shrinks it. Deferred triage — test-vs-production verdict per cluster; do NOT mass-fix, take clusters as loop items.
- **Current work:** #406's source-ordered Kid -> wobble -> flash SAT makes frames 1368/1683 exact; the full RGB baseline remains 26,382 exact / 11,377 divergent / 3,620,978 changed pixels, with first mismatch frame 1697 in the life icon. #409 already carries the exact `sub_44DC` FIFO/post-HUD ledger to `loc_892`; frame 1709's four GEMS steals at offsets 43/96/151/192 remain the visual frontier. The route-wide GEMS frontier is now frame 16: frames 14-15 defend the progressive first `StopMusic` copy, and the next pass must trace the newly running driver's first VBlank/IRQ without losing those exact samples. The later title-fill 55-cycle residue is upstream state from the documented frame-139 jgenesis X/Y-flag limitation, not a local fill delay.

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
| 416 | [ ] | P2 | audio | `gems_frame_signature_v1` measures all 37,759 route frames divergent from frame 0: the route audio replayer models neither the boot cold-start (driver pre-loaded vs the real first-`StopMusic` copy) nor any route-wide 68000↔Z80 interaction. | Advance `first_mismatch_frame`/`mismatch_count` via `tas-gems-baseline`; start with the cold-start model; verify-tas-full green. |
| 417 | [ ] | P2 | testing | Ungated `wildside-cli` suite rot — 53 grandfathered failing tests (52 `native_player`, 1 `inspect`); the new `failing-test-ratchet` gates fresh regressions, triage each cluster test-vs-production. | Shrink `failing-test-baseline.yaml` per fix; ratchet + verify-slice-fast green. |
| 419 | [ ] | P1 | enemy | Goat contact/lifecycle divergence fails sweep levels 0x38/0x60/0x63 (16 blockers): oracle Kid takes settle-time Goat damage native misses (0x60 cascades to level transition + life loss); 0x38/0x63 hold object/gfx count deficits. | Probe 0x60 settle contact vs oracle; all three levels pass state sweep. See archive #419. |
| 420 | [ ] | P1 | enemy | Driller worker/child undercount + missed Kid contact fail 0x55/0x68 (19 blockers, largest cluster): at 0x55 oracle Kid takes Driller contact and a Micromax transform native never has. | Probe 0x55 settle vs oracle; both levels pass state sweep. See archive #420. |
| 421 | [ ] | P1 | enemy | Scorpion settle contact + missing coil/tail objects fail 0x61/0x62 (10 blockers): oracle Kid is stung on the spawn row while native Kid is untouched. | Probe 0x61 settle vs oracle; both levels pass state sweep. See archive #421. |
| 422 | [ ] | P1 | enemy | UFO beam-worker spawn/timing parity fails 0x30/0x48 (7 blockers): oracle beam hits Kid off the bridge (0x30) and owns beam children native under-spawns (0x48, archive #290). | Probe UFO beam timing at 0x30 vs oracle; both levels pass state sweep. See archive #422. |
| 423 | [ ] | P1 | enemy | Ninja lifecycle/contact divergence fails 0x56 (5 blockers): native Kid dies and leaves the level on idle while oracle Kid survives one hit; native holds ~5 extra Ninja-lineage objects. | Probe 0x56 idle vs oracle; level passes state sweep. See archive #423. |
| 424 | [ ] | P2 | enemy | MiniHoppingSkull dormant proximity controllers (`loc_3E9D0` Object-only slots) are uncounted natively, failing 0x44 (+5 number_objects, gfx equal) and 0x65 (+1). | Model the dormant wait; both levels pass state sweep. See archive #424. |
| 425 | [ ] | P2 | enemy | Leading off-screen SpinningTwins never allocated natively fails 0x47 (+1 object/+2 gfx — the Enemy14 dual-gfx init fingerprint). | Probe 0x47 settle roster vs oracle; level passes state sweep. See archive #425. |
| 426 | [ ] | P2 | enemy | Hand floor-scan deletion parity fails 0x59 (+1/+1): native keeps one Hand alive that oracle deletes via the `loc_32AFE` ground scan. | Probe 0x59 settle roster vs oracle; level passes state sweep. See archive #426. |
| 427 | [ ] | P2 | enemy | Tornado RNG/pattern flight drift fails 0x50: oracle Kid is grazed for exactly 1 HP during settle, native Kid untouched; positions and counts match. | Probe 0x50 settle HP vs oracle; level passes state sweep. See archive #427. |
| 430 | [ ] | P2 | kid-physics | 0x52 vertical settle out of phase: kid_y_vel is sign-flipped at settle (±1.75px/f) and kid_y never reconciles (native pinned, oracle bobs); empty roster, 7 blockers. | Probe 0x52 settle bob vs oracle. See archive #430. |
| 431 | [ ] | P2 | weather | 0x21 murder wall settles ~24px behind oracle and never pushes camera_x off 0 during settle. | Probe murder-wall settle at 0x21 vs oracle. See archive #431. |
| 432 | [ ] | P2 | level | 0x42 city high-skin collision decode leaves Kid one tile (16px) high on the lamppost platform — the residual already named in #330. | Probe 0x42 settle terrain row vs oracle. See archive #432. |
| 433 | [ ] | P2 | verification | Leftover level 0x41 (no foreground; native skips load, oracle plays garbage RAM to InGame) contributes 12 authoritative blockers that compare nothing real. | Classify 0x41 in the sweep with cited evidence (oracle_limitation or load decision). See archive #433. |
