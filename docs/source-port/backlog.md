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

Status: `[ ]` open, `[~]` in progress, `[f]` frozen lane (standing decision; never selectable, do not extend), `[?]` agent proposes fix complete — **human must validate before closing**, `[x]` done, `[-]` won't fix / not a bug, `[b]` blocked / needs human repro
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
- **Current work (2026-08-23):** no ready implementation or open playability/visual risk after #459's analysis-only NO-GO; use the proactive-verification fallback. The live Lion `loc_3E356` post-charge resume decision changed after the family's last module census. Three of its four condition mutations survived because the tests covered only both-false and both-true native representations. The ASM decision is now an explicit pure predicate with all four grounded/airborne and ordinary/ResumingCharge combinations; the diff census catches all seven selected predicate and caller mutations. The accidentally widened initial census also exposed 24 unrelated projectile-field survivors, but every one belongs to blocked boss umbrella #301 and is not selectable. This Lion path feeds published-route enemy state. Continue with a different live gameplay or sweep-path module; do not extend this Lion resume slice, the death-shell or wall-stop slices, the prize-wake slice, the Cyclone landing slice, the loader/status/murder-wall slices, the platform-loader branch, the closed `loc_16F0` collision slice, Tar burst-child lifecycle, enemy-frame dispatch, Rock Tank or walking Dragon selectors, UFO, Archer, Goat, Driller, Crab, Scorpion, Emo Rock, Orca, Diamond, Fire Demon, Hand, Big Hopping Skull, Tornado, Fireball, Spinning Twins, Cloud, Robot, Armadillo, blocked boss umbrella #301, or frozen timing feeders.
- **Current #409 boundary (FROZEN, ADR 0002; measurement corrected by ADR 0003; #459 NO-GO):** the recurrence's proven boundary is published frame 1657 / 736 scheduled frames, ratchet downward-only; do NOT extend or promote the recurrence. The corrected interval replay reports **241/241 exact CPU spans** and discriminates at ±1 68000 cycle, but #459 proves the bounded `$7C8` entry interval is not a frame-local seed: two life-icon frames retain multiple H40 commit slots, and `WaitForVint` entry inversion lacks the poll boundary, refresh phase, resume path, and ordered Z80 steals. The freeze, cost case, and 241-frame/2,391-px ratchets stand. See ADR 0003 + archive #459.
- **#409 instrument hardening (2026-08-16):** `raster-span-replay` is now a Rust subcommand sharing the engine divider (byte-identical to the retired Python on the fresh census); the probe recipe is checked in (`mise run hud-timing-probe`; oracle-probe `--no-frame-captures`, full span ~67s); every `scripts/test_*.py` runs in `verify-slice-fast` and `scripts/**/*.py` edits trigger the hk gate.
- **Pixel-gate region ratchet (2026-08-17):** `tas-pixel-baseline` now ratchets a per-region partition (hud_diamond 170/1,651 px + hud_life_icon 71/740 px) and a bbox-carrying digest, so debt relocating between regions or within a frame at constant totals reads as Regression. All three TAS ratchets refuse `--update` on a Regression without `--force`. Frozen rows carry `[f]` and never jam the selector; a commit-msg hook enforces subject/blank/body.
- **Current Diamond witness:** oracle frame 1730 starts the `$DD40` DMA at H40 20:380; adjacent frames emit no Diamond train. The existing mixed-page renderer is correct but lacks this persistent entry cursor; see archive #409.
- **Current frame-1730 bound:** the complete floating-prize roster exposes two untyped `loc_103B2` row-advance callbacks, so the old partial-roster H40 15:284 lower bound is withdrawn. The exact scheduler fails closed until that recurring prize path is traced.
- **Source-port visual ratchet:** `1p-accurate` renders Level 0 frames 0, 30, 60, and 120 through the source-port renderer against jgenesis with exact state, no masking, MAE 0.000, and zero changed pixels. The gate now requires exact pixels.
- **Visual confidence boundary:** #375 is resolved for those four settled-warp checkpoints only. This is not cold-boot or whole-game pixel parity. The older `frame_oracle:*` steps compare wildside-md ROM VDP output with jgenesis, not source-port pixels.
- **Full published-route evidence:** all 37,759 requested player/camera CRC frames match oracle. The report names this narrow projection `player_camera_progression_v1` and declares that it proves neither pixels nor whole state. The complete RNG stream also matches on all 37,743 post-`Init_RNG` frames, including the offset and all 55 ring bytes on every frame. The gameplay and enemy-frontier passes compare every one of the 37,759 frames with zero authoritative, advisory, accumulation, RNG, or enemy-roster differences; the full checkpoint pass carries 24 evidence-backed oracle limitations (the paired `number_objects`/`number_gfx_objects` native=8/oracle=7 frames where native correctly retains the collected-diamond worker per `loc_10396`). #404 adds direct typed `Current_player` and `Two_player_flag` parity; the latter is absent from the CRC. The `source_port_visible_rgb24_v1` pass also renders and compares every route frame: 37,518 are exact and 241 diverge, totalling 2,391 changed pixel instances (the enforced `tas-pixel-baseline` total). This is exact CRC, RNG, exported gameplay state, and enemy-roster evidence plus complete measured visual debt on one route, not whole-RAM, audio, every-input, or whole-game equality.
- **Whole-level sweep boundary:** report `1786881192` is 66/69 pass with 12 blocking differences. The hidden-shell collision hand-off reproduces the 0x30 UFO hit and 0x50 Tornado hit. Focused report `1786887279` proves 0x24 and 0x40 now pass after retaining status words for layout entries beyond index 29. The only remaining authoritative failure is boss level 0x33, owned by blocked umbrella #301. Still two idle checkpoints per sweepable level, not full-route or every-input proof.
- **Sweep-blocker census (#418):** every one of the 106 blocking diffs in historical run `1783914264` is attributed. New `sweep-census` subcommand (mise `sweep-census`; artefact `.cache/audit/sweep-census.json`) mechanically joins blocking diffs onto per-level enemy-family rosters; a 22-agent read-only attribution pass (PNGs + ASM + native code per failing level) produced clusters #419-#433: five P1 enemy clusters (Goat 16, Driller 19, Scorpion 10, UFO 7, Ninja 5 blockers), four remaining P2 single-level enemy items (MiniHoppingSkull, SpinningTwins, Hand, Tornado), one Kid-physics item (0x52), murder-wall 0x21, level-decode 0x42, and leftover-level 0x41 classification (12 blockers comparing garbage). 0x33 HeadyMetal (4 blockers) is owned by blocked #301, not re-rowed. #428's TarMonster attribution was stale after #403; fresh Level 0x64 state report `1784577126` passes. #429 now applies `loc_75C0` spawn-overlap correction to debug title-card entry; fresh Level 0x29 state report `1784577500` passes. No ready item remains; `project-status` selects the open risk lane.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **GEMS audio (frozen + superseded, #416):** the `gems_frame_signature_v1` per-frame frontier is FROZEN at frames 0..922 (`coverage.modelled_frames=923`; decision + rationale in archive #416). `tas-gems-baseline` is now a regression witness only: `mismatch_count` ratchets down, an earlier first mismatch blocks, a later one is never rewarded. Do NOT advance `modelled_frames` or take frame 923 as work. Full counts stay 36,836 mismatches / 885 limitations / 38 advisory (disjoint) - measured timing debt per methodology "Command-parity supersession". Audio authority is the command-parity gate `gems_command_parity_v1` (#435, closed): all 437 native commands match all 437 oracle commands over the published route, with 391 exact and 46 reconciled inside the established one-frame sampling skew; mismatch count is zero. #439 owns exact SFX call sites and #440 owns the final schedule alignment. Replay probe #436 is GO. Bounded replay #437 and discriminator #441 establish that the residual frame-signature debt is bounded true Z80 execution drift, not oracle sampling error: at frame 871 the captured sample clock makes divider phases equal while the native PC remains one instruction ahead. This does not represent command-content debt and remains governed by the frozen-witness decision.
- **Gate hardening (#417 closed):** the `failing-test-ratchet` hk step now pins an empty `wildside-cli` failure set. All 53 grandfathered failures have verdicts and the complete CLI suite is green; any future failure blocks the commit.
- **#406 is metered now, not banned (ADR 0001).** The persistent timing frontier remains frame 995 and the per-frame march is the only known production path beyond it. Frame 1706 no longer needs a mid-route seed for rendering: a source-proven earliest-commit lower bound resolves its Diamond word without claiming the actual clock. Two escape routes remain structurally closed: GEMS checkpoint seeding carries no 68k PC/beam/phase, and r68k's cycle table disagrees with jgenesis on `MULU`. Three gates keep the treadmill bounded: `backlog-lint` rejects a Verify that merely asks for Summary's next frame; `oracle-injection-ratchet` meters clock teleports (frontier 995, 108 below the rolling window, downward-only); and the frame-995 timing test runs in `verify-slice-fast`. **Do not type frame 996 as a bare index step** - an index-advancing commit must also retire an override, injection, or unattributed constant.
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
| 406 | [f] | P1 | rendering | FROZEN with #409 per ADR 0002: the lane stops accruing sessions; the march stays metered as ADR 0001 left it (frontier 995, injection ratchet). Residue authority moves to census-scoped reconstruction + raster-span-replay. | Do not extend the march; ratchets stay green; take #442-#450 instead. See ADR 0002. |
| 409 | [f] | P1 | rendering | FROZEN per ADR 0002: the recurrence is a downward-only witness (scheduled-frames ratchet 736 / boundary 1657; tas-pixel-baseline). ADR 0003 corrected the spans; #459 found the bounded entry phase non-invertible and closed NO-GO. | Do not extend or promote the lane; keep the 2,391-pixel and scheduled-frame ratchets green. Evidence: archive #459. |
| 416 | [f] | P2 | audio | Frontier FROZEN at frames 0..922 (decision in archive #416): `gems_frame_signature_v1` is a regression witness only; command parity (#435) supersedes it as audio authority. | Never advance `modelled_frames`; #434-#438 closed, remaining audio debt tracked via #439-#441; keep verify-tas-full green. |
