# Progress

## Current Step

- Active step: Step 7
- Status: completed

## Step Status

1. Step 1: Lock the generated-asset contract for decoded Level 1 visuals - completed
2. Step 2: Add `kid-render` and load Level 1 visual value objects - completed
3. Step 3: Extend the software renderer with indexed tile and sprite primitives - completed
4. Step 4: Replace the legacy terrain seam with manifest-backed Level 1 visuals - completed
5. Step 5: Convert background, foreground/block families, and HUD to asset-backed scene commands - completed
6. Step 6: Replace player placeholders with ROM-art sprites and preserve selection rules - completed
7. Step 7: Cover the remaining visible Level 1 families and harden failure/reporting paths - completed

## Active Wave

- No active runtime wave remains. `kc-level1-graphics` is complete.

## Notes

- Steps 1 to 3 are complete and no longer active in the runtime queue.
- Step 4 is complete and its reviewed runtime wave is closed.
- Step 5 is complete and its reviewed runtime wave is closed.
- Step 6 is complete and its reviewed runtime wave is closed.
- Step 7 landed and closed the final visible-family and failure-hardening slice.

## TDD Evidence

- RED: `cargo test -p kid-data export_split_bundle_writes_decoded_level1_entries` failed first with `Saxman input offset 16504 is past payload length 32` after switching the block fixture to a real-style Level 1 block blob, then failed again with `Enigma packet length must be between 1 and 16 bits` after switching the background fixture to a real-style Level 1 background layout. `cargo test -p kid-data export_split_bundle_keeps_existing_bundle_when_regeneration_fails` also failed before the atomic export repair because the initial bundle generation used the same bad block assumption.
- GREEN: `kid-data` now exports Level 1 foreground via Saxman decode, but exports Level 1 block and background outputs from the extracted raw layout files, bumps the decoded Level 1 contract version, and stages bundle regeneration in a sibling temporary directory before swapping it into place.
- REFACTOR: kept the export path small by extracting staging/swap helpers and a shared raw-level-asset exporter, then reran formatting and the targeted verification set.

## Verification

- `cargo test -p kid-data --lib`
- `cargo test -p kidcli --bin kidcli prepare_generated_assets_rebuilds_when_decoded_version_missing`
- `cargo check -p kid-data -p kidcli`
- `cargo run -p kidcli -- extract-assets --rom /home/connor/git/kc/kc.md`

## Outcome

- Step 1 now matches the real Level 1 split outputs: foreground is decoded from Saxman, while block and background are preserved as the extracted raw layouts under `decoded/level/00/` with previews and manifest metadata.
- Bundle regeneration is atomic, so a failed rebuild no longer deletes the previous generated bundle or leaves `asset-manifest.json` missing.
- Real CLI verification against `/home/connor/git/kc/kc.md` succeeded and rebuilt `/home/connor/git/kc/.cache/generated/28b904000b2863b6760531807760b571f1a5fc1d`.

## Step 2 Evidence

- RED: `cargo test -p kid-render --lib` failed first because `load_level1_visuals` and the renderer-facing types did not exist, and `cargo test -p kid-data --lib export_split_bundle_writes_decoded_level1_entries` failed because the new `ingame_assets` manifest field was unimplemented in existing fixtures.
- GREEN: added the `kid-render` crate with immutable tile, palette, level, and HUD value objects; implemented `load_level1_visuals()` on top of the generated bundle contract; and extended `kid-data` bundle export to copy and catalogue `ingame/artcomp` assets so block and HUD art load through the manifest instead of ad hoc paths.
- REFACTOR: split the loader into `bundle`, `tiles`, `palette`, `level`, and `hud` modules, added contract-aware file reads with contextual errors, and reformatted the touched Rust sources with `cargo fmt`.

## Step 2 Verification

- `cargo test -p kid-render --lib`
- `cargo test -p kid-data --lib`
- `cargo check -p kid-data -p kid-render -p kid-core -p kidcli`

## Step 2 Outcome

- The workspace now has a dedicated `kid-render` crate between bundle data and the future scene/raster steps.
- Level 1 visual loading now yields stable value objects for foreground/background/block tile sheets, palettes, block mappings, decoded layout bytes, and HUD digit/icon metadata.
- Missing required bundle assets now fail with family-specific errors, and the generated bundle contract was bumped so existing caches rebuild to include the new `ingame/artcomp` catalogue.

## Step 2 Review Repair

- Review feedback identified a fidelity gap: `BlockVisualMapping::resolve()` collapsed every `Some(block_type)` case into one generic block-skin lookup, so `kid-render` lost the gameplay distinction between visible Level 1 block families.
- RED: strengthened `load_block_visual_mapping_from_skin_and_block_type` to require renderer-facing results for terrain plus `Smash`, `Prize`, `Ice`, `Metal`, `Rubber`, and generic blocks, backed by a synthetic `theme/block_mappings.bin` shaped like the real 81-quad asset. `cargo test -p kid-render --lib` then failed because the crate exposed only raw `TileQuad` results and no typed block resolution.
- GREEN: `kid-render` now validates the 81-quad block-mapping payload, splits it into one generic quad plus five 16-skin block-family tables, and returns `ResolvedBlockVisual { kind, quad }` so callers retain `block_type` semantics as data instead of special-casing later.
- REFACTOR: added a small `BlockVisualKind` enum, kept unknown block types on an explicit generic-block fallback, and wrapped block-mapping construction in loader context so malformed bundle data fails clearly.

## Step 2 Re-Verification

- `cargo fmt`
- `cargo test -p kid-render --lib`
- `cargo test -p kid-data --lib`
- `cargo check -p kid-data -p kid-render -p kid-core -p kidcli`
- `cargo run -p kidcli -- extract-assets --rom /home/connor/git/kc/kc.md`

## Step 2 Repair Outcome

- `kid-render` now preserves gameplay-facing block family distinctions in its immutable Level 1 value objects instead of flattening all block cells into one undifferentiated mapping.
- The strengthened loader test covers the visible block families the repo already distinguishes, and the real `extract-assets` smoke run still succeeds against `/home/connor/git/kc/kc.md`.
- Remaining caveat for critic: the five family groups are inferred from the real 81-quad `theme/block_mappings.bin` shape (`1 + 5 x 16`) and the repo’s existing visible block-type set, so review should confirm that this ordering matches the intended upstream family order.

## Step 3 Evidence

- RED: `cargo test -p kidcli --bin kidcli render_` failed first because `DrawCommand` did not yet expose `IndexedTile` or `Sprite`, and `SpriteTile` did not exist in `native_player.rs`.
- GREEN: extended `kidcli`'s software renderer with `DrawCommand::IndexedTile`, `DrawCommand::Sprite`, a `SpriteTile` mapping primitive, palette-aware indexed tile rasterisation, transparency-on-index-zero, flip handling, viewport clipping, and two-pass sprite priority drawing.
- REFACTOR: kept the raster path SDL-independent, factored shared helpers (`draw_indexed_tile`, `draw_sprite`, `set_pixel`, `resolve_palette_colour`), wired `kid-render` tile/palette value objects into `kidcli`, and added narrow `dead_code` allowances because the production scene builder still emits only debug rectangles until Step 4.

## Step 3 Verification

- `cargo fmt`
- `cargo test -p kidcli --bin kidcli`
- `cargo check -p kid-data -p kid-render -p kid-core -p kidcli`

## Step 3 Outcome

- The native RGB24 rasteriser now has indexed 8x8 tile and mapped-sprite primitives ready for asset-backed scene construction.
- Renderer tests now cover the exact acceptance cases for palette application, zero-index transparency, flips, clipping, and sprite mapping priority.
- Current caveat for critic: the new draw-command variants are only constructed in tests for now; Step 4 still needs to replace scene-builder rectangles with these primitives on the default Level 1 path.

## Step 3 Review Repair

- Review feedback identified two concrete fidelity misses in the new raster path: indexed tiles ignored `palette_line`, and mixed tile/sprite layering still followed raw command order so a later low-priority tile could overwrite an earlier priority sprite pixel.
- RED: strengthened `render_indexed_tiles_applies_palette_and_transparency` to draw from palette line 1 and strengthened `render_sprite_command_honours_mapping_priority_and_transparency` to place a low-priority indexed tile after a sprite command. `cargo test -p kidcli --bin kidcli render_` then failed with the expected wrong line-0 colour lookup and sprite-overwritten-by-tile pixel.
- GREEN: `resolve_palette_colour()` now offsets into the selected 15-colour palette line, and `render_scene()` now rasterises in two passes so non-priority commands draw before priority indexed tiles and sprite subtiles while preserving scene order within each pass.
- REFACTOR: simplified `draw_sprite()` to render only the requested priority tier per pass and kept the renderer helpers SDL-independent so the existing native-player test style still covers the full raster path.

## Step 3 Re-Verification

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p kidcli --bin kidcli`
- `cargo check -p kid-data -p kid-render -p kid-core -p kidcli`

## Step 3 Repair Outcome

- Indexed tiles now honour `TileRef.palette_line` instead of always sampling line 0.
- Priority sprite pixels now survive later low-priority tile commands, while transparent sprite pixels still reveal underlying tiles.
- The repaired renderer still passes the full `kidcli` unit test suite and workspace check set.

## Step 3 Second Review Repair

- Review feedback identified one remaining fidelity gap: the global low/high-priority passes let earlier high-priority indexed pixels overwrite later legacy `Rect`/`Text` commands, regressing the pre-Step-3 scene-command ordering contract.
- RED: added `render_priority_tiles_preserve_scene_order_for_legacy_commands`, a mixed-command regression with `[high-priority IndexedTile, later Rect, later low-priority IndexedTile]`. `cargo test -p kidcli --bin kidcli render_` failed first with the earlier high-priority tile colour `[200, 10, 10]` still visible where the later rect should have won.
- GREEN: `render_scene()` now rasterises strictly in scene order again while carrying a per-pixel high-priority mask for indexed tile and sprite pixels only. Legacy commands clear that mask when they redraw a pixel, so later low-priority tiles no longer get blocked by stale priority state, while indexed/sprite priority still protects high-priority pixels from later low-priority indexed draws.
- REFACTOR: threaded the priority mask through the existing SDL-independent helpers (`fill_rect`, `draw_text`, `draw_indexed_tile`, `draw_sprite`, `set_pixel`) instead of keeping separate pass-specific code paths, so the ordering rules now live in one place.

## Step 3 Second Re-Verification

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p kidcli --bin kidcli render_`
- `cargo test -p kidcli --bin kidcli`
- `cargo check -p kid-data -p kid-render -p kid-core -p kidcli`

## Step 3 Second Repair Outcome

- High-priority indexed tile and sprite pixels still survive later low-priority indexed draws.
- Legacy `Rect`/`Text` commands once again compose by scene-command order even when they overlap high-priority indexed content.
- Current caveat for critic remains unchanged: the new draw-command variants are still only constructed in tests until Step 4 rewires the production scene builder onto them, so the strongest available verification for this step is the renderer test harness rather than a live native-player frame.

## Step 4 Evidence

- RED: `cargo test -p kidcli --bin kidcli build_scene_uses_loaded_level1_visuals_for_ingame` failed first with the expected seam errors: `build_scene()` still accepted only `(trace, hud_visible, block_grid)`, `try_load_block_grid()` still mutated `NativeRuntime` from the legacy `level-layout/{id}` dump path, and the session bootstrap helpers named by the new tests did not exist.
- GREEN: added `NativeSessionAssets`, `load_native_session_assets_with()`, and `initialise_runtime()` so `run_native_player()` loads `Level1Visuals` once per session, reuses them across resets, and reinstalls the manifest-backed `BlockGrid` into each fresh `NativeRuntime`. `build_scene()` now accepts optional `Level1Visuals` for non-gameplay modes and requires them for `GameMode::InGame`, while `try_load_block_grid()` now reads `decoded_level1.block` bytes plus `LevelHeader` dimensions instead of `level-layout/{id}.bin` + `.json` sidecars.
- REFACTOR: kept the scene seam pure by threading loaded values into `build_game_scene()` rather than pulling file I/O into scene construction, centralised the header-to-cell conversion in `level_layout_dimensions()`, and collapsed the gameplay test updates behind one `build_ingame_scene()` helper so existing coverage kept its previous intent.

## Step 4 Verification

- `cargo test -p kidcli --bin kidcli build_scene_uses_loaded_level1_visuals_for_ingame`
- `cargo fmt --all`
- `cargo test -p kidcli --bin kidcli`
- `cargo check -p kidcli`

## Step 4 Outcome

- The native-player default path now bootstraps Level 1 visuals exactly once, passes them into gameplay scene construction, and reuses them after runtime resets instead of reloading them inside the loop.
- Terrain loading no longer depends on ad hoc `level-layout/{id}` sidecars; it now reconstructs `BlockGrid` from the manifest-backed decoded block layout and the existing level header metadata.
- Title/status scenes remain text/debug driven and continue to build without any `Level1Visuals` value.

## Step 5 Evidence

- RED: `cargo test -p kidcli --bin kidcli build_game_scene_` failed first because `Level1Visuals` had no background model, the scene builder still emitted guide/full-frame rects plus coloured terrain/HUD placeholders, and the new Step 5 tests could not compile against the missing `kid_render::{BackgroundVisuals, BackgroundChunk, BackgroundPlacement, BackgroundTile}` seam.
- GREEN: added `kid-render::background` plus manifest-backed chunk loading for `theme/bg_chunks/<theme>_NN.bin`, threaded the parsed background placements/chunks through `Level1Visuals`, and rewrote `kidcli::native_player` to emit indexed-tile commands for cave background chunks, terrain/block-family quads resolved through `BlockVisualMapping::resolve(skin, block_type)`, and HUD hitpoint/digit art instead of placeholder text/rectangles on the default path.
- REFACTOR: centralised VRAM-to-sheet translation in `resolve_ingame_vram_tile()`, kept scene ordering by pushing background -> terrain -> objects -> HUD through the existing draw-command pipeline, and updated legacy `native_player` tests to assert indexed-tile families rather than the removed debug-colour placeholders.

## Step 5 Verification

- `cargo fmt --all`
- `cargo test -p kidcli --bin kidcli build_game_scene_`
- `cargo test -p kidcli --bin kidcli native_scene_builder_uses_asset_backed_commands_for_visible_level1_families`
- `cargo test -p kidcli --bin kidcli`
- `cargo check -p kidcli`
- `cargo test -p kid-render --lib`

## Step 5 Outcome

- The default Level 1 gameplay scene now emits ROM-backed background tiles, ROM-backed terrain/block-family quads, and HUD art digits/hitpoint tiles while leaving the optional debug overlay separate behind `hud_visible`.
- `kid-render` now models chunked background layouts explicitly, so `play-native` no longer needs to guess at raw Level 1 background bytes inside `kidcli`.
- Scene-order compatibility remains intact because all new static-family rendering still flows through `DrawCommand::IndexedTile` and the existing software rasteriser.

## Step 5 Review Repair

- Review feedback identified one real HUD gap on the default path: the scene builder still omitted the diamond counter icon, and `resolve_ingame_vram_tile()` only covered the `0x06BA` digit span so higher HUD refs such as `0x86C4` (`:`) and `0x86F2` (diamond) could never resolve.
- RED: strengthened `build_game_scene_emits_hud_art_without_text_placeholders` to require the diamond HUD art, added `resolve_ingame_vram_tile_maps_hud_symbols_and_diamond_tiles`, and extended `kid-render`'s HUD loader fixture to cover the `numbers,:,x` tiles plus animated diamond tiles. The first targeted rerun failed at `expected the HUD colon icon`, confirming the resolver gap before implementation changed.
- GREEN: extended the generated bundle contract to copy `ingame/artunc/diamond_1.bin`, taught `kid-render` to load `HUD_numbers.bin` as the full `numbers,:,x` sheet plus a separate uncompressed diamond sheet, and updated `kidcli::native_player` to resolve both HUD VRAM regions and render counter art as `digit(s) + x + icon` for lives and diamonds.
- REFACTOR: extracted `push_hud_counter()` and `push_hud_icon()` so the HUD composition rules live in one place, and kept the resolver-driven scene path intact rather than adding HUD-only draw shortcuts.

## Step 5 Review Repair Verification

- `cargo fmt --all`
- `cargo test -p kid-data --lib`
- `cargo test -p kid-render --lib`
- `cargo test -p kidcli --bin kidcli build_game_scene_emits_hud_art_without_text_placeholders`
- `cargo test -p kidcli --bin kidcli resolve_ingame_vram_tile_maps_hud_symbols_and_diamond_tiles`
- `cargo test -p kidcli --bin kidcli build_game_scene_`
- `cargo test -p kidcli --bin kidcli native_scene_builder_uses_asset_backed_commands_for_visible_level1_families`
- `cargo test -p kidcli --bin kidcli`
- `cargo check -p kidcli -p kid-render -p kid-data`
- `timeout 5 env SDL_VIDEODRIVER=dummy cargo run -p kidcli --bin kidcli -- play-native --rom /home/connor/git/kc/kc.md --scale 1` (reached the native loop before timeout)

## Step 5 Review Repair Outcome

- The default HUD path now consumes real counter art for lives and diamonds, including the animated diamond icon rather than bare digits on the right-hand counter.
- `resolve_ingame_vram_tile()` now covers both HUD symbol tiles and the separate animated-diamond VRAM window, with focused regression coverage for both offsets.
- The generated bundle contract now includes the HUD diamond art asset required by the repaired default path, and the timeout smoke-test lock was cleared after verification so later iterations are not blocked by a stale `.prepare.lock`.

## Step 5 Second Review Repair

- Review feedback identified a remaining HUD acceptance gap: `push_game_hud()` still sized the hitpoint strip with `trace.current_helmet_id.max(2)`, which treats the helmet enum value as capacity and therefore underdraws Skycutter/Cyclone-style forms and overdraws IronKnight/Juggernaut/Berzerker-family forms.
- RED: added `game_hud_uses_player_form_hitpoint_capacity_for_helmet_state` to cover both a 3-HP form (`Skycutter`) and the 5-HP form (`IronKnight`), then reran `cargo test -p kidcli --bin kidcli game_hud_uses_player_form_hitpoint_capacity_for_helmet_state`. It failed first with `left: 2 right: 3`, proving the HUD still derived capacity from the raw helmet id.
- GREEN: changed `push_game_hud()` to derive its capacity from `PlayerForm::from_helmet_id(trace.current_helmet_id).map(PlayerForm::max_hitpoints)` with The Kid's 2-HP form as the fallback for unknown ids, so the art HUD now reflects the runtime form model instead of the enum discriminant.
- REFACTOR: kept the repair small, added `count_hud_hitpoint_tiles()` as a focused test helper for the HUD strip, and left the rest of the asset-backed HUD pipeline unchanged.

## Step 5 Second Review Repair Verification

- `cargo fmt --all`
- `cargo test -p kidcli --bin kidcli build_game_scene_`
- `cargo test -p kidcli --bin kidcli resolve_ingame_vram_tile_maps_hud_symbols_and_diamond_tiles`
- `cargo test -p kidcli --bin kidcli game_hud_uses_player_form_hitpoint_capacity_for_helmet_state`
- `cargo test -p kidcli --bin kidcli native_scene_builder_uses_asset_backed_commands_for_visible_level1_families`
- `cargo test -p kid-render --lib missing_required_asset_reports_contextual_error`
- `timeout 5 env SDL_VIDEODRIVER=dummy cargo run -p kidcli --bin kidcli -- play-native --rom /home/connor/git/kc/kc.md --scale 1` (reached the native loop before timeout and did not leave `.cache/generated/.prepare.lock`)

## Step 5 Second Review Repair Outcome

- The default Level 1 HUD now draws the correct number of hitpoint slots for each helmet form on the validated path instead of guessing from the numeric helmet id.
- Focused coverage now catches both the 3-HP and 5-HP form cases that the previous HUD tests missed.

## Step 6 Evidence

- RED: `cargo test -p kid-data --lib export_split_bundle_writes_decoded_level1_entries` failed with `left: 4 right: 9` because the generated bundle still excluded `ingame/artunc_kid` and `ingame/palette_kid`; `cargo test -p kid-render --lib load_player_visuals_returns_required_kid_frames` then failed because `Level1Visuals` had no `player` field; and `cargo test -p kidcli --bin kidcli build_game_scene_emits_player_sprite_tiles_for_idle_walk_jump_and_fall` failed because `NativeTraceFrame` exposed no motion/facing fields and the scene builder still emitted no player sprite command.
- GREEN: extended the bundle contract to stage `ingame/artunc_kid` and `ingame/palette_kid` with a generated-bundle version bump, added renderer-facing `PlayerVisuals`/`SpriteFrame` loading for Kid idle/walk/jump frames plus palette data in `kid-render`, exposed `kid_x_velocity`, `kid_y_velocity`, `kid_facing`, and `kid_on_ground` on `NativeTraceFrame`, and replaced the native-player placeholder rect with sprite-command emission driven by a pure frame selector that preserves the existing invulnerability blink cadence.
- REFACTOR: kept player frame metadata inside `kid-render` rather than `native_player.rs`, added small helpers for sprite-tile composition and screen-space placement, and updated the synthetic `kidcli` fixture visuals to assert frame selection and sprite positioning without depending on ROM pixels.

## Step 6 Verification

- `cargo fmt --all`
- `cargo test -p kid-data --lib`
- `cargo test -p kid-render --lib`
- `cargo test -p kid-core --lib`
- `cargo test -p kidcli --bin kidcli`
- `cargo check -p kidcli -p kid-render -p kid-data -p kid-core`
- `timeout 15 env SDL_VIDEODRIVER=dummy cargo run -p kidcli --bin kidcli -- play-native --rom /home/connor/git/kc/kc.md --scale 1` (reached the native loop before timeout, regenerated the bundle, and left no `.cache/generated/.prepare.lock`)

## Step 6 Outcome

- The default Level 1 gameplay path now renders The Kid with ROM-art sprite commands for idle, walk, rising jump, and falling states instead of a placeholder rectangle.
- Player frame selection is driven from single-frame trace state, so blink cadence stays unchanged while walk/jump/fall art selection remains pure and testable.
- Generated-bundle regeneration now includes the Kid palette/art families required by the new default path, so existing caches rebuild cleanly onto the asset-backed player seam.

## Step 7 Evidence

- RED: `cargo test -p kidcli --bin kidcli` initially failed in the Step 7 wave because `kidcli::native_player` referenced `NativeObjectKind` inconsistently, the mixed scene tests still assumed placeholder fallback rectangles, and the gameplay-evidence/missing-asset assertions were tied to stale path and top-level-error assumptions.
- GREEN: fixed the native scene-builder/runtime seams so visible Level 1 enemy, platform, and collectible families assert asset-backed commands on the default path; kept missing-asset handling strict at startup with contextual errors; and preserved semantic gameplay comparison while writing frame evidence for inspection.
- REFACTOR: added a small test-only native-session asset validation seam, hardened assertions against the full `anyhow` chain, and split death-respawn settle timing from level-advance settle timing so the oracle path stays accurate without changing normal progression recovery.

## Step 7 Verification

- `cargo test -p kidcli --bin kidcli`
- `cargo test -p kid-render --lib`
- `cargo test -p kid-core --lib`
- `cargo check -p kidcli -p kid-render -p kid-data -p kid-core`
- `cargo run -p kidcli --bin kidcli -- compare-native-gameplay --rom /home/connor/git/kc/kc.md --scenario pause-resume-level1`
- `cargo run -p kidcli --bin kidcli -- compare-native-gameplay --rom /home/connor/git/kc/kc.md --scenario death-respawn-level1`

## Step 7 Outcome

- The validated Level 1 traversal now renders the remaining visible enemy, platform, collectible, HUD, background, terrain, and player families from ROM-backed commands on the default path.
- Required visual assets now fail fast with contextual startup errors instead of silently falling back to placeholder art.
- The `kc-level1-graphics` implementation plan is complete and no mirrored runtime task remains open.
