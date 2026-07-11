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
- **Full published-route evidence:** all 37,759 requested player/camera CRC frames match oracle. At 1,512 sampled checkpoints, the gate ratchets #290's 212 authoritative export diffs and 2 oracle limitations. The complete RNG stream starts after `Init_RNG` at frame 16 and compares all 37,743 remaining frames: 16,864 currently differ, first at frame 20895, pinned by digest `b40154c5...`. Frames 16..20894 are complete-ring exact. This is per-frame RNG observation on one route, not whole-state, enemy, render, audio, or whole-game equality.
- **Whole-level sweep boundary:** current report `1783779907` reaches real gameplay on all 69 BCD levels: 43 pass and 26 fail with 96 blocking diffs, 3 advisory diffs, and 45 oracle limitations. The older 45/24 report predates the debug-title scheduler commit; a detached-HEAD audit reproduces the current Level 0x01/0x68 object-count blockers. The old all-masked green sweep was invalid.
- **Current awaiting-human P0s:** #210 rapid title/menu skip, #370 Dragon breath respawn, and #374 named later-level spawn parity.
- **Latest proactive correction:** #389 now uses UFO sprite `$0DE8`'s exact `unk_31CE0` 28x24 collision box, post-object surface snap, one-frame `collision_type(a3)` hand-off, and loc_35104/loc_3512A bounce signs shared with Tornado. Level 0x48 keeps two lifecycle blockers; idle-tail changed pixels improve 2,779 -> 2,159 and worst MAE 3.5 -> 2.8, while settle pixels rise 1,845 -> 2,439. Its 360-frame route remains 0/360 whole-roster exact because beam/Drips child representation still differs.
- **Latest RNG confidence correction:** `loc_1BC36`'s combined `sub_1C034(d4=4) -> sub_1C0EE` upload crosses the presented VBlank at different local elapsed boundaries on the Start-skip and ordinary timeout-restart lineages. Native now draws on frame 20758 and freezes frame 20759 exactly. Frames 16..20894 are complete-ring exact. The next frontier is IntroSequence6 frame 20895, where native freezes while oracle advances (`$FFD0/$FFD4`, ring index 8 `$27/$F0`); eliminating the 16,864-frame pinned debt remains fidelity work.
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
