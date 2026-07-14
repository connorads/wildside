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
- **Full published-route evidence:** all 37,759 requested player/camera CRC frames match oracle. The report names this narrow projection `player_camera_progression_v1` and declares that it proves neither pixels nor whole state. The complete RNG stream also matches on all 37,743 post-`Init_RNG` frames, including the offset and all 55 ring bytes on every frame. The gameplay and enemy-frontier passes compare every one of the 37,759 frames with zero authoritative, advisory, accumulation, RNG, oracle-limitation, or enemy-roster differences. #404 adds direct typed `Current_player` and `Two_player_flag` parity; the latter is absent from the CRC. The `source_port_visible_rgb24_v1` pass also renders and compares every route frame: 24,067 are exact and 13,692 diverge, totalling 9,054,996 changed pixel instances. This is exact CRC, RNG, exported gameplay state, and enemy-roster evidence plus complete measured visual debt on one route, not whole-RAM, audio, every-input, or whole-game equality.
- **Whole-level sweep boundary:** canonical report `1783914264` is 47/69 pass with 106 blocking differences, 3 advisory differences, and 56 oracle limitations. #402 corrects #401's coarse Tar Monster model with the real `unk_32264` cursor, 28-frame animations, `unk_316AC` glob collision box, and prior-pass collision latch. Levels 0x46 and 0x51 remain authoritative-exact. This is still two idle checkpoints per sweepable level, not full-route or every-input proof.
- **Check tightening:** insta snapshots re-homed to `src/tests/snapshots/` with real (unredacted) values; `asm-coverage --check-no-new-test-only` now fails on stale baseline entries (45 re-banked, 241 -> 196); advisory-masking prose retired from CLAUDE.md/methodology.md.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **Current work:** #406 is the first full-route source-port visual frontier. The software VDP now separates H40's 20-sprite scan cap from its 40-tile fetch budget, including a partially fetched final sprite. FC82=2 also keeps the pre-particle `loc_1BCD0` SAT for its measured final latch, then exposes `loc_1BE30/24/18` in oracle order without rewinding particle motion. Published frames 20847..20849 are pixel-exact; frame 20850 falls to 128 pixels. Frame 1837 is now worst at 4,313 pixels on the demo-1 `0x1B -> 0x00` return. Trace that transition's presented render latch against ASM and oracle; raw `capture-scenario-frames` does not reproduce the verifier's transition latch.

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
| 406 | [~] | P1 | rendering | Full published TAS pixel pass finds 13,692 divergent frames; next visual frontier is frame 1837. | 24,067/37,759 RGB frames exact; state and RNG remain exact. |
