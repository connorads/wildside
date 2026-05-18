# Human Validation Checklist

Use this checklist to close the remaining source-port completion blockers that
cannot be resolved by another automated sweep or TAS frontier pass.

Before changing any backlog row, rerun the matching verifier command and inspect
the listed artefacts. If the human decision is "pass", update
`docs/source-port/backlog.md` with the reviewer, date, and exact artefacts used.
If the decision is "fail", keep the row open and add the observed mismatch or
repro notes.

To verify every currently referenced bundle before review:

```bash
mise run verify-human-validation
```

To build a local HTML review page for the exact PNGs/transcripts in this
checklist:

```bash
mise run human-validation-review
```

Open `.cache/human-validation/index.html` after generation.

## Visual Validation Rows

| Backlog row | Verify command | Inspect | Pass decision |
|---|---|---|---|
| #135 Level 1 background | `cargo run -p kidcli -- verify-recording --slug issue-135-level1-background --native-only` | `.cache/recordings/issue-135-level1-background/verify/native/000000_start.png` | Settled Blue Lake Woods 1 background matches the original game closely enough for the current 1P phase |
| #136 prize emergence | `cargo run -p kidcli -- verify-recording --slug issue-136-prize-emergence --native-only` | `.cache/recordings/issue-136-prize-emergence/verify/native/000150_post_first_jump.png`, `000270_post_second_jump.png`, `000359_scenario_end.png` | Diamond/helmet emergence uses the expected item graphics and does not show the old incorrect block/item art |
| #139 rubber block | `cargo run -p kidcli -- verify-recording --slug issue-139-rubber-block-window --native-only` | `.cache/recordings/issue-139-rubber-block-window/verify/native/000699_walk_end.png` | Visible rubber blocks use rubber-block art, not spike art |
| #140 prize block | `cargo run -p kidcli -- verify-recording --slug issue-140-prize-block-window --native-only` | `.cache/recordings/issue-140-prize-block-window/verify/native/000150_post_first_jump.png`, `000239_pre_second_jump.png`, `000359_scenario_end.png` | Prize block visible/disguised/deleted states use the expected block art during the hit lifecycle |

## Playability Validation Rows

| Backlog row | Verify command | Inspect | Pass decision |
|---|---|---|---|
| #138 enemy contact | `cargo run -p kidcli -- verify-recording --slug issue-138-enemy-contact --native-only` | `.cache/recordings/issue-138-enemy-contact/transcript.txt`, `verify/native/000300_frame_000300.png`, `verify/native/000360_frame_000360.png` | Two damage events 66 frames apart are accepted as ROM-faithful behaviour rather than a native double-damage bug |
| #143 Heavy Man block break | `cargo run -p kidcli -- verify-recording --slug issue-143-heavy-man-block-break --native-only` | `.cache/recordings/issue-143-heavy-man-block-break/transcript.txt`, `verify/native/*.png` | Current bundle cannot pass: it is a negative baseline that does not reach Heavy Man, a helmet pickup, or block break |

For #143, record a new human-controlled route before closure:

```bash
cargo run --release -p kidcli -- play-native --warp-level 00 --settled --record issue-143-heavy-man-block-break --record-frames 1800
cargo run -p kidcli -- verify-recording --slug issue-143-heavy-man-block-break --native-only
```

The replacement bundle must show Heavy Man or Iron Knight block-breaking with
helmet/form state active. A generic Level 00 damage/death recording is not
evidence for this row.

## Blocked Repro Row

| Backlog row | Current command | Current evidence | Required human action |
|---|---|---|---|
| #146 Level 2 death/loading hang | `cargo run -p kidcli -- verify-recording --slug issue-146-level2-death --native-only` | Current bundle is an idle negative baseline with no events | Record a Level 2 death that either reproduces the loading hang or proves the report cannot be reproduced after reasonable attempts |

Recommended repro command shape:

```bash
cargo run --release -p kidcli -- play-native --warp-level 01 --settled --record issue-146-level2-death --record-frames 1800
cargo run -p kidcli -- verify-recording --slug issue-146-level2-death --native-only
```

If a hang is reproduced, keep #146 open and add the first bad frame, transcript
event window, and any relevant screenshots. If repeated human attempts cannot
reproduce it, close or defer the row only with the tested routes documented.
