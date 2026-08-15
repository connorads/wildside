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
- **Current #409 boundary (FROZEN, ADR 0002):** the recurrence's proven boundary is published frame 1657 / 736 scheduled frames. The scoped measurement is complete and NO-GO: zero Z80 wait does not recover the exact frame-local raster cursor (204/241 zero-wait frames; both wait-free witnesses retain phase residuals). The ratchet is 736, downward-only; do NOT extend or promote the recurrence. See archive #409 + ADR 0002.
- **Current Diamond witness:** oracle frame 1730 starts the `$DD40` DMA at H40 20:380; adjacent frames emit no Diamond train. The existing mixed-page renderer is correct but lacks this persistent entry cursor; see archive #409.
- **Current frame-1730 bound:** the complete floating-prize roster exposes two untyped `loc_103B2` row-advance callbacks, so the old partial-roster H40 15:284 lower bound is withdrawn. The exact scheduler fails closed until that recurring prize path is traced.
- **Source-port visual ratchet:** `1p-accurate` renders Level 0 frames 0, 30, 60, and 120 through the source-port renderer against jgenesis with exact state, no masking, MAE 0.000, and zero changed pixels. The gate now requires exact pixels.
- **Visual confidence boundary:** #375 is resolved for those four settled-warp checkpoints only. This is not cold-boot or whole-game pixel parity. The older `frame_oracle:*` steps compare wildside-md ROM VDP output with jgenesis, not source-port pixels.
- **Full published-route evidence:** all 37,759 requested player/camera CRC frames match oracle. The report names this narrow projection `player_camera_progression_v1` and declares that it proves neither pixels nor whole state. The complete RNG stream also matches on all 37,743 post-`Init_RNG` frames, including the offset and all 55 ring bytes on every frame. The gameplay and enemy-frontier passes compare every one of the 37,759 frames with zero authoritative, advisory, accumulation, RNG, or enemy-roster differences; the full checkpoint pass carries 24 evidence-backed oracle limitations (the paired `number_objects`/`number_gfx_objects` native=8/oracle=7 frames where native correctly retains the collected-diamond worker per `loc_10396`). #404 adds direct typed `Current_player` and `Two_player_flag` parity; the latter is absent from the CRC. The `source_port_visible_rgb24_v1` pass also renders and compares every route frame: 37,518 are exact and 241 diverge, totalling 2,391 changed pixel instances (the enforced `tas-pixel-baseline` total; ADR 0002's 2,457 figure is stale pending the session-1 correction). This is exact CRC, RNG, exported gameplay state, and enemy-roster evidence plus complete measured visual debt on one route, not whole-RAM, audio, every-input, or whole-game equality.
- **Whole-level sweep boundary:** report `1786796452` is 65/69 pass with 14 blocking differences. #456 now has Armadillo and Lion on the Lion.asm cliff-fall walker: all ten family levels pass with zero whole-sweep regression. Orca and walking Dragon remain staged. #455 is closed: all settled-warp phase injections are deleted with zero sweep regression. The settled-warp shell is witness-gated: frame-0 RNG exact on 49/65 comparable levels; surviving enemy divergences are post-frame-0 behaviour. Still two idle checkpoints per sweepable level, not full-route or every-input proof.
- **Sweep-blocker census (#418):** every one of the 106 blocking diffs in historical run `1783914264` is attributed. New `sweep-census` subcommand (mise `sweep-census`; artefact `.cache/audit/sweep-census.json`) mechanically joins blocking diffs onto per-level enemy-family rosters; a 22-agent read-only attribution pass (PNGs + ASM + native code per failing level) produced clusters #419-#433: five P1 enemy clusters (Goat 16, Driller 19, Scorpion 10, UFO 7, Ninja 5 blockers), four remaining P2 single-level enemy items (MiniHoppingSkull, SpinningTwins, Hand, Tornado), one Kid-physics item (0x52), murder-wall 0x21, level-decode 0x42, and leftover-level 0x41 classification (12 blockers comparing garbage). 0x33 HeadyMetal (4 blockers) is owned by blocked #301, not re-rowed. #428's TarMonster attribution was stale after #403; fresh Level 0x64 state report `1784577126` passes. #429 now applies `loc_75C0` spawn-overlap correction to debug title-card entry; fresh Level 0x29 state report `1784577500` passes. No ready item remains; `project-status` selects the open risk lane.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **GEMS audio (frozen + superseded, #416):** the `gems_frame_signature_v1` per-frame frontier is FROZEN at frames 0..922 (`coverage.modelled_frames=923`; decision + rationale in archive #416). `tas-gems-baseline` is now a regression witness only: `mismatch_count` ratchets down, an earlier first mismatch blocks, a later one is never rewarded. Do NOT advance `modelled_frames` or take frame 923 as work. Full counts stay 36,836 mismatches / 885 limitations / 38 advisory (disjoint) - measured timing debt per methodology "Command-parity supersession". Audio authority is the command-parity gate `gems_command_parity_v1` (#435, closed): all 437 native commands match all 437 oracle commands over the published route, with 391 exact and 46 reconciled inside the established one-frame sampling skew; mismatch count is zero. #439 owns exact SFX call sites and #440 owns the final schedule alignment. Replay probe #436 is GO. Bounded replay #437 and discriminator #441 establish that the residual frame-signature debt is bounded true Z80 execution drift, not oracle sampling error: at frame 871 the captured sample clock makes divider phases equal while the native PC remains one instruction ahead. This does not represent command-content debt and remains governed by the frozen-witness decision.
- **Gate hardening (#417 closed):** the `failing-test-ratchet` hk step now pins an empty `wildside-cli` failure set. All 53 grandfathered failures have verdicts and the complete CLI suite is green; any future failure blocks the commit.
- **#406 is metered now, not banned (ADR 0001).** The persistent timing frontier remains frame 995 and the per-frame march is the only known production path beyond it. Frame 1706 no longer needs a mid-route seed for rendering: a source-proven earliest-commit lower bound resolves its Diamond word without claiming the actual clock. Two escape routes remain structurally closed: GEMS checkpoint seeding carries no 68k PC/beam/phase, and r68k's cycle table disagrees with jgenesis on `MULU`. Three gates keep the treadmill bounded: `backlog-lint` rejects a Verify that merely asks for Summary's next frame; `oracle-injection-ratchet` meters clock teleports (frontier 995, 108 below the rolling window, downward-only); and the frame-995 timing test runs in `verify-slice-fast`. **Do not type frame 996 as a bare index step** - an index-advancing commit must also retire an override, injection, or unattributed constant.
- **Current work - 7-session plan (2026-08-12):** execute the sessions below in order. Session 1 hygiene is complete: #452-#454 are closed, `verify-web-fast` is green, `project-status` surfaces #442, the early-route playtest checklist is checked in, ADR 0002 says 2,391 current pixels, and rows #419/#420 reflect landed #449 evidence. The plan no longer overrides the selector.
  1. **Complete - Crypt trio:** #419's 0x38 arm, #425, and #433 are classified as non-comparable leftover warps. Focused run `1786567508` reports 36 oracle limitations, zero blockers, and all three levels UNVERIFIED.
  2. **Complete - #442 shell CONTENT model (sessions 3-6's centre of gravity, landed):** the per-level load-completion boundary replaced elapsed-71/72, the hand tails and family fudges are deleted, the murder wall runs in the hidden live shell, and the frame-0 RNG witness gate (`gameplay_regression_settled_warp_frame0_witnesses_probe_levels`) pins the shell against `oracle-settle-shell.yaml`. Result: 55/69 pass / 74 blocking (run `1786646904`), zero regressions; 49/65 comparable levels frame-0 witness-exact. #442 closed; residual shell injections opened as #455.
  3. **Complete - #419 Goat model (CLOSED, with #457):** the two-walker-library partition, the spawn delay, and now the grounded loc_3E406/loc_3E5BA wall hold (level edge included via sub_36972 loc_369BC; `$42`=$FF88 re-armed, step never committed, no turn — the fraction survives, closing #457's half-pixel) are all landed. 0x60 idle frames 0..359 match the oracle exactly; sweep `1786660540` clears the whole cluster. The other six lineage families stay on the cliff-turn approximation until their companions land (#456). #420 is CLOSED: the retract zero-row box, transform flash-pause + loc_7E90 velocity clear, init-keyed walk speed, and the loc_104AE latched-prize wake schedule collapse the last five 0x55 blockers — both Driller levels pass the state sweep (1786695230, zero regressions).
  4. **Complete - gate debt:** #443, #445 and #417 are closed. ROM is green-gated; md/frame/data/render/web run under a six-name downward-only ratchet; the complete `wildside-cli` suite has an empty failure baseline.

  Deferred this plan: #406/#409 stay frozen with no instrumentation session (the traced-manifest step waits; `hud-timing-census`/`raster-span-replay` stay aspirational); #408/#413 pixel witnesses wait for state-clean levels post-#442; the warp-battery playtest checklist follows the early-route one. Instruments from phase 3 remain available: `oracle-settle-census`, `sweep --include-rng`, `verify-disasm-fidelity` (bit-perfect), `verify-cpu-vectors`. Full pixels: 37,518 exact / 241 divergent / 2,391 changed (ratchet-enforced). Frontier 995 and injection ratchet 108 unchanged.

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
| 406 | [~] | P1 | rendering | FROZEN with #409 per ADR 0002: the lane stops accruing sessions; the march stays metered as ADR 0001 left it (frontier 995, injection ratchet). Residue authority moves to census-scoped reconstruction + raster-span-replay. | Do not extend the march; ratchets stay green; take #442-#450 instead. See ADR 0002. |
| 409 | [~] | P1 | rendering | FROZEN per ADR 0002: the recurrence is a downward-only witness (scheduled-frames ratchet 736 / boundary 1657; tas-pixel-baseline). Scoped measurement completed NO-GO; residue remains HUD-icon DMA-entry-phase debt, 2,391 px, ratcheted. | Do not extend or promote the lane; keep the pixel and scheduled-frame ratchets green. See ADR 0002 + archive #409. |
| 416 | [~] | P2 | audio | Frontier FROZEN at frames 0..922 (decision in archive #416): `gems_frame_signature_v1` is a regression witness only; command parity (#435) supersedes it as audio authority. | Never advance `modelled_frames`; #434-#438 closed, remaining audio debt tracked via #439-#441; keep verify-tas-full green. |
| 456 | [~] | P2 | enemy | Stage the remaining Lion-lineage families onto the measured cliff-fall walker; each needs its companions first. Armadillo, Lion, Scorpion, and Ninja are complete; Orca and walking Dragon remain. | Per family: model companions, flip the gate, family levels pass the sweep with zero regressions. See archive #456. |
