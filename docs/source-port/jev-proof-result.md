# Jev historical-defect comparison result

The 2026-09-19 run `1789816286588649000` reports **case did not expose the bug**.
The comparison runner works, but this is not a successful gameplay reproduction
of #372. Run it with `mise run jev-proof`; this measured outcome exits one.

## Measured contrast

The isolated checkout restores the historical prize-before-terrain ordering
from fix `5d83d0aa3`, adapted to the current scene renderer. No gameplay logic or
bootstrap state is changed. The focused #372 scene regression passes in the
fixed build and fails in the isolated build. Its failed pixel assertion shows
terrain colour `[73, 0, 0]` instead of prize colour `[182, 0, 0]`. This verifies
the defect restoration rather than merely assuming a source edit is effective.

Both builds replay the same 437-frame Jev-generated controller stream exactly.
The target worker is active at frames 376-406, and all 91 captures at 346-436
match the emulator pixel for pixel in both builds. The two gameplay traces are
identical; the oracle state comparisons pass. The emulator emitted no enemy
snapshots, so these comparisons do not establish per-enemy state parity.
The target and full-frame broken-versus-fixed pixel differences are both zero.

The discrepancy has a concrete cause. The old defect affects
`build_game_scene_filtered`. Current recording playback chooses the VDP path
in `render_runtime_checkpoint_frame` whenever a VDP state is present. The
historical scene draw order does not control these gameplay images. An explicit
scene-path feasibility capture also differs from the VDP/oracle output at
frames 400 and 406, confirming that the two render surfaces are not
interchangeable evidence.

The original plan allowed up to three new Jev attempts if needed. That search
was omitted because different player inputs cannot make the restored inactive
scene defect affect VDP rendering. No new API calls were made, and no substitute
VDP fault was invented. A subsequent successful bug-and-fix demonstration needs
a defect that is exercised by the active rendering or gameplay path. #372 stays
in the awaiting-human lane.

## Reproduction and verification

The [runner guide](jev-proof.md) describes the command and verdict rules.
[Machine-readable evidence](jev-proof-evidence.json) retains the input, ROM,
runner and defect hashes plus comparison results. Full images, logs, patches,
the isolated checkout location and the self-contained HTML report are under
`.cache/jev-proof/1789816286588649000/`.

The HTML report was opened in a disposable Playwright session. All three
320x224 images loaded; the slider exposed 91 checkpoints and moved from frame
346 to frame 382 with counts matching the JSON. The visible prize in all three
frame-382 images was inspected. The browser and local report server were closed.

Eight Python runner tests pass, including rejected two-green-build evidence,
missing captures, incomplete encounters, state/trace drift, unrelated pixel
differences and HTML escaping. `verify-slice-fast` passes, including 7,406 core
tests; required commit hooks pass. The isolated historical renderer failure is
intentional and remains outside the working checkout.

The first execution (`1789816149281451000`) stopped while cloning build files:
PATH selected GNU `cp`, which does not support macOS's `-c` clone flag. The
runner uses `/bin/cp` on macOS; the subsequent measured run completed. The
initial execution-failure report is retained separately.
