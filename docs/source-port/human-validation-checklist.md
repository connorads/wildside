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

This gate is intentionally native-only. It proves the saved bundles still replay
deterministically in the native engine, then leaves the remaining visual and
playability calls to a human reviewer; it must not be used as oracle-parity
evidence.

To run the completion audit gate, which must fail until the rows below are
closed or deliberately deferred:

```bash
mise run completion-audit
```

To verify all referenced bundles and build a local HTML review page for the
exact PNGs/transcripts in this checklist:

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
| #143 Heavy Man / Iron Knight block break | `cargo run -p kidcli -- verify-recording --slug issue-143-level27-ik-smash-probe-5 --native-only` | `.cache/recordings/issue-143-level27-ik-smash-probe-5/transcript.txt`, `.cache/recordings/issue-143-level27-ik-smash-probe-5/verify/native/*.png`, trace frames 171 and 655-666 | Scripted route collects Iron Knight, then destroys block cells at `(496,128)`, `(512,128)`, and `(528,128)` via ASM `sub_AF7A` high-velocity landing behaviour |

For #143, a positive deterministic route now exists:

```bash
cargo run -p kidcli -- verify-recording --slug issue-143-level27-ik-smash-probe-5 --native-only
```

The route warps to Caves of Ice, hits the Iron Knight prize block, collects
Iron Knight at frame 171, and destroys blocks from frame 655 onward. Keep this
row open until a human accepts the saved route/PNGs as sufficient playability
evidence, or records a preferred live Heavy Man route.

The TAS-derived route is useful but still insufficient:

```bash
cargo run --release -p kidcli -- record-scenario-file --slug issue-143-tas-route --scenario tests/data/tas/kc_any_percent_2024/scenario.json
cargo run -p kidcli -- verify-recording --slug issue-143-tas-route --native-only
```

That route reaches Knight's Isle Iron Knight (`helmet 0 -> 6` at frame 3170)
and verifies as replayable, but it has no Iron Knight block-break candidate,
block bounce, or block destroy effect before returning to the title screen.

## Playability Validation Row

| Backlog row | Current command | Current evidence | Required human action |
|---|---|---|---|
| #146 Level 2 death/loading hang | `cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-death --native-only`; `cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-stomp-long-probe --native-only` | Short scripted `stomp-enemy` Level 01 route takes damage at frames 814/1272 and enters `LevelTransition` at frame 1347; long 9000-frame probe recovers to `InGame` at frame 8589 | Decide whether the long death/loading interval is acceptable for the current 1P phase, or record a different Level 2 death route that still hangs |

Recommended refresh command:

```bash
cargo run --release -p kidcli -- record-scenario --slug issue-146-level2-death --scenario stomp-enemy --level 01 --walk-frames 1800
cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-death --native-only
cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-stomp-long-probe --native-only
```

If a hang is reproduced on a different route, keep #146 open and add the first
bad frame, transcript event window, and any relevant screenshots. If human review accepts
the long scripted transition as non-blocking, close or defer the row only with the tested routes documented.
