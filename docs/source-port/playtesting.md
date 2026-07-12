# Playtest mode

`play-native --playtest` turns the SDL player into an issue-validation harness: the
awaiting-human backlog lane feeds an in-game browser, voice carries only judgement,
and the engine captures the context (level, frame, Kid position, replayable input)
that used to be dictated by hand.

## Session workflow (human, usually on the MacBook Air)

```bash
mise run playtest            # play-native --playtest --mic
mise run playtest-no-mic     # play-native --playtest
```

In-game keys (all playtest-only; plain play-native is unchanged):

| Key | Action |
|-----|--------|
| F1 | Pause and open/close the issue browser |
| Up/Down or W/S | Select an issue (browser open) |
| Left/Right | Toggle filter: current-level+`unknown` vs all (browser open) |
| Enter/Space | Arm the selected issue (warps for `0xNN`, resets for `boot`, otherwise pins the Check) |
| Y / H / G | Verdict for the armed issue: confirmed fixed / still broken / partial |
| B | Drop a bug bookmark (screenshot + position, marks the segment worth saving) |
| N / P / R | Next/previous level / reset — under playtest these split the segment instead of bailing |
| F10 or Esc | Finalise the session and exit |

While the browser is open the game is hard-paused: the runtime does not step, so
paused frames never enter a segment script (deterministic replay is preserved) and
no audio is generated or captured.

## Session contract

Each session lands in `.cache/playtest/pt-YYYYMMDD-HHMMSS/`:

```
session.json          # manifest: segments, verdicts, bookmarks, sync marks, mic info
bookmarks/PT-001.png  # 320x224 pre-overlay screenshots
segments/segNN/       # standard recording bundles (slug <session>-segNN)
mic.wav               # session-long narration (with --mic)
report.md             # written by playtest-ingest
```

- `session.json` is rewritten on every verdict/bookmark/sync mark, so a crash loses
  at most the in-flight event.
- Segments are standard recording bundles: each starts from a state
  `verify-recording` can reconstruct (`{warp_to_level}` or boot), so
  `verify-recording --bundle .cache/playtest/<session>/segments/segNN` replays them.
  Playtest warps deliberately build a fresh runtime (no carried lives/score/form) to
  keep that guarantee; only save-worthy segments (a bookmark or a still-broken/partial
  verdict) are flushed.
- Sync marks (`SegmentStart`, `Pause`/`Resume`, `Interval` every ~1 s) map wall-clock
  time to segment frames. Pause/Resume marks share a frame index, so paused wall time
  never inflates transcript alignment.
- The session directory is the transfer unit: copy it under `.cache/playtest/` on
  another machine (e.g. record on the Air, ingest on the mini).

## Ingest (draft report only)

```bash
# with a MacWhisper (or any) transcript export:
mise run playtest-ingest -- --session pt-20260712-090807 --transcript export.srt

# or with a local whisper CLI printing SRT/VTT/JSON to stdout:
mise run playtest-ingest -- --session pt-20260712-090807 \
  --whisper-cmd "whisper-cli -m models/ggml-base.en.bin -oj -of /dev/stdout {wav}"
```

Accepted transcript formats: SRT, VTT, whisper.cpp JSON (`transcription[].offsets`),
and OpenAI/mlx-whisper JSON (`segments[].start/end` seconds). Utterance midpoints are
interpolated between sync marks into `(segment, frame)` positions and attached to
verdicts/bookmarks within a ±10 s wall-clock window. A truncated `mic.wav` (crash
before the header finalised) is tolerated with a warning.

`playtest-ingest` writes `<session>/report.md` and **never edits the backlog**.

## Report format (for the consuming agent session)

The report is a draft. A follow-up agent session applies it per `loop.md`, keeping
backlog-lint and archive-evidence conventions with the agent:

- **Verdicts**: one section per Y/H/G verdict with the issue id, Target, Check, the
  segment bundle path for `verify-recording --bundle`, attached narration, and a
  suggested lane move (confirmed fixed → archive with closing evidence; still
  broken → back to the open queue with this session as repro; partial → append
  evidence under the issue's `#NNN`).
- **New issues**: one section per bookmark with level, Kid pixel position, segment
  frame, screenshot path, narration, and a ready-to-fill awaiting-human row skeleton.
  The agent assigns real `#NNN` ids, writes an overlay-safe Check, and decides the
  lane.

Verify a cited segment before acting on it:

```bash
cargo run -p wildside-cli -- verify-recording --bundle .cache/playtest/<session>/segments/seg01
```
