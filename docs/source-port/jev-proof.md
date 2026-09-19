# Jev bug-comparison runner

`mise run jev-proof` checks whether the saved Jev-generated prize route can
expose #372's historical render-order defect. It requires the ROM, generated
assets, the oracle toolchain and ImageMagick's `magick`. It makes no API calls.

The runner checks the fixed scene regression, creates a fresh recording from
the checked-in input stream, and compares it with the emulator. A detached
checkout contains a snapshot of tracked working-tree changes plus only the
historical prize-before-terrain ordering defect. The exact patch and revision
are retained. Its focused scene regression must fail at the expected assertion,
then the identical recording is verified using that build. Build directories
are separate; oracle runs are sequential. The main checkout is never patched.

## Rendering-path constraint

The historical fix `5d83d0aa3` concerns `build_game_scene_filtered`. Current
recording playback uses `render_runtime_checkpoint_frame`, which renders the
VDP when available. The scene renderer is a separate fallback and diagnostic
path. Moving the old scene draw call cannot affect VDP gameplay output.

A passing scene test followed by a failing restored scene test proves that the
historical defect was restored. It does not prove the gameplay route exposes
it. Two matching gameplay builds are explicitly **case did not expose the bug**.
Further Jev attempts cannot make an inactive renderer affect the VDP output,
so this case does not spend API budget on target discovery or new play. A future
case needs a defect in the active rendering path before paid collection can
supply useful evidence. No substitute VDP fault is manufactured to force a win.

## Evidence

Each run creates `.cache/jev-proof/<run>/report.html` and `report.json`. The HTML
embeds the chronological broken/fixed/emulator images and provides a frame
slider. It is self-contained. JSON includes exact input and ROM hashes,
encounter timing, replay and state results, full-frame and target pixel counts,
and the original Jev decision transcript. A complete capture covers 30 frames
before the hit through 30 frames after the target worker disappears.

Command exit zero requires **proof passed**: the historical regression is
verified, the encounter and capture are complete, traces match, oracle states
match, fixed pixels match exactly, and broken target pixels differ during the
animation. Other outcomes exit one: **case did not expose the bug**,
**comparison inconclusive**, or **execution failed**. Unrelated fixed pixel
differences are retained and prevent a clean proof. Missing images never pass.

The isolated checkout and logs remain for diagnosis. Its location is recorded
in `provenance.json`; build files live beside it under the system temporary
directory. The runner neither commits nor changes issue status. #372 retains
its human-validation requirement.
