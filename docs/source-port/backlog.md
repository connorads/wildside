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
- **Full published-route evidence:** all 37,759 requested player/camera CRC frames match oracle. The complete RNG stream also matches on all 37,743 post-`Init_RNG` frames, including the offset and all 55 ring bytes on every frame. The strengthened 1,512-checkpoint state gate ratchets #290's 72 authoritative export diffs and 15 oracle limitations, beginning at `frame_16650/number_objects`. The earlier 212/2 baseline omitted `Number_GfxObjects` and reused ambiguous labels for repeated transitions. This is exact player/camera CRC and RNG evidence on one route, not whole-state, enemy, render, audio, or whole-game equality.
- **Whole-level sweep boundary:** current report `1783779907` reaches real gameplay on all 69 BCD levels: 43 pass and 26 fail with 96 blocking diffs, 3 advisory diffs, and 45 oracle limitations. The older 45/24 report predates the debug-title scheduler commit; a detached-HEAD audit reproduces the current Level 0x01/0x68 object-count blockers. The old all-masked green sweep was invalid.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **Latest proactive correction:** #389 now uses UFO sprite `$0DE8`'s exact `unk_31CE0` 28x24 collision box, post-object surface snap, one-frame `collision_type(a3)` hand-off, and loc_35104/loc_3512A bounce signs shared with Tornado. Level 0x48 keeps two lifecycle blockers; idle-tail changed pixels improve 2,779 -> 2,159 and worst MAE 3.5 -> 2.8, while settle pixels rise 1,845 -> 2,439. Its 360-frame route remains 0/360 whole-roster exact because beam/Drips child representation still differs.
- **Latest RNG confidence correction:** IntroSequence2's `loc_1B9D0 -> loc_1BA02 -> sub_1C246 -> loc_1C264` title return remains in `Load_TitleScreen` through local elapsed 56 and resumes `Mode_Standard -> Execute_Objects loc_E12` at elapsed 57. Native now carries a distinct `LaterIntroSequence2` lineage instead of sharing IntroSequence4's elapsed-56 schedule. The complete published-route RNG stream is exact on all 37,743 compared frames; the remaining fidelity frontier includes #290's 72 authoritative sampled-state differences under the expanded comparator.
- **Latest proactive correction:** raw WRAM linked lists show demo-1 startup frame 71 has two system GfxObjects, frame 72 links Kid's two slots, and `Manage_EnemyLoading loc_3691E -> Enemy0D_Dragon_Init` links Dragon GfxObjects one object pass after each Dragon Object. Native now models the frame-73 first Dragon edge and the frame-530 third Dragon edge, preserving 8/7 through the 32-frame `loc_119D8` title snapshot. Published frames 0..3399 are exact at every compared gameplay checkpoint; the prior fast baseline falls 99 -> 0. The full sampled baseline falls 477 -> 450 authoritative differences plus 10 exposed oracle limitations, first at `frame_5525/number_gfx_objects`.
- **Latest proactive correction:** mature Demo 1 now follows `Mode_Level loc_80C -> Manage_EnemyLoading loc_368A8/loc_3691E` after Kid's camera update, retains the Island `loc_12098` Object-only controller, counts live Dragons, and exposes each Dragon Object one pass before `Enemy0D_Dragon_Init` links its GfxObject. The removed native-only +1px Dragon spawn offset contradicted `Enemy0D_Dragon_Init -> sub_36FF4`. Published frames 13747..14055 are exact at every one of 309 state/CRC checkpoints. Full sampled debt falls 238 -> 72 authoritative differences with 15 oracle limitations; the first sampled diff moves to `frame_16650/number_objects`. CRC and complete RNG remain exact across the full route.
- **Full-frame sweep boundary:** #383 removes the native-only synthetic settle override. Level 0 report `1783751667` passes with zero blocking diffs and two known object-export limitations; idle-tail render remains non-blocking at MAE 1.2.
- **Current work:** no agent-ready row remains. Run `mise run project-status`, then use `loop.md`'s proactive-verification fallback to select the next concrete fidelity slice.

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
