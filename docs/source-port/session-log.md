# Source-port session log

Append-only history of per-session handoffs, extracted from the `backlog.md`
NOW block (which is the *current-state* handoff and is rewritten each session).
Newest entries first. This is narrative history; per-issue evidence keyed by
`#NNN` lives in [`backlog-archive.md`](./backlog-archive.md).

---

## 2026-07-01 handoff (proactive CLI release-gate mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-cli/src/release_gate.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-cli -f crates/wildside-cli/src/release_gate.rs -j 1
  --gitignore=true --output /tmp/mutants-cli-release-gate-3 --
  release_gate::tests::` reports `30 mutants tested in 2m: 30 caught`, with
  zero missed mutants and zero timeouts.
- Strengthened release-gate tests for target-specific missing-ROM failures,
  release-step pass/fail report entries, cargo-step diagnostics, timestamp
  shape, and the shared sweep-failure predicate.
- No native gameplay behaviour changes.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive CLI RAM-diff mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-cli/src/ram_diff.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-cli -f crates/wildside-cli/src/ram_diff.rs -j 1 --gitignore=true
  --output /tmp/mutants-cli-ram-diff-3 -- ram_diff::tests::` reports `12
  mutants tested in 86s: 12 caught`, with zero missed mutants and zero timeouts.
- Split RAM-diff formatting into a pure helper behind the existing file-reading
  CLI wrapper, then covered changed-byte output, annotation gating, size
  mismatch errors, and wrapper error propagation.
- No native gameplay behaviour changes.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive CLI backlog-lint mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-cli/src/backlog.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-cli -f crates/wildside-cli/src/backlog.rs -j 1 --gitignore=true
  --output /tmp/mutants-cli-backlog-3 -- backlog::tests::` reports `36
  mutants tested in 3m: 35 caught, 1 unviable`, with zero missed mutants and
  zero timeouts.
- Strengthened backlog-lint tests for markdown row filtering, malformed/template
  rows, NOW block termination, and exact NOW/Verify threshold boundaries.
- No native gameplay behaviour changes.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive CLI oracle-cache mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-cli/src/cache.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-cli -f crates/wildside-cli/src/cache.rs -j 1 --gitignore=true
  --output /tmp/mutants-cli-cache-3 -- cache::tests::` reports `9 mutants
  tested in 74s: 7 caught, 2 unviable`, with zero missed mutants and zero
  timeouts.
- Strengthened oracle-cache tests with real temp files for persisted frame/audio
  copying, cached manifest path rewrites, stale cached-frame rejection, and
  idempotent storage when a manifest already points at cached frame files.
- No native gameplay behaviour changes.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive CLI diff diagnostic mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-cli/src/diff.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-cli -f crates/wildside-cli/src/diff.rs -j 1 --gitignore=true
  --output /tmp/mutants-cli-diff-4 -- diff::tests::` reports `91 mutants
  tested in 5m: 91 caught`, with zero missed mutants and zero timeouts.
- Strengthened diagnostic-render tests for composite panel row layout, exact
  glyph pixels, clipping boundaries, and annotation panel placement. Fixed
  `draw_text` so horizontally out-of-bounds glyph pixels are skipped instead of
  wrapping into the next row.
- No native gameplay behaviour changes.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive core gfx-animation mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-core/src/gfx_animation.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-core -f crates/wildside-core/src/gfx_animation.rs -j 1
  --gitignore=true --output /tmp/mutants-core-gfx-animation-proactive-2 --
  --lib -- --skip demo_ --skip published_tas --skip snapshot` reports `18
  mutants tested in 2m: 16 caught, 2 unviable`, with zero missed mutants and
  zero timeouts. The skip filters are the documented core mutation-test filters
  for pre-existing demo/TAS/snapshot exclusions.
- Native gfx animation helper behaviour is unchanged. Existing ASM-cited
  gfx-animation tests catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive data asset-source mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-data/src/asset_source.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-data -f crates/wildside-data/src/asset_source.rs -j 1
  --gitignore=true --output /tmp/mutants-data-asset-source-proactive -- --lib`
  reports `15 mutants tested in 36s: 15 caught`, with zero missed mutants and
  zero timeouts.
- Native/in-memory asset-source behaviour is unchanged. Existing port-adapter
  tests catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive render background mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-render/src/background.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/background.rs -j 1
  --gitignore=true --output /tmp/mutants-render-background-proactive -- --lib`
  reports `69 mutants tested in 2m: 62 caught, 7 unviable`, with zero missed
  mutants and zero timeouts.
- Native render background parsing/composition behaviour is unchanged. Existing
  background tests catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive ROM validation mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-rom/src/lib.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-rom -f crates/wildside-rom/src/lib.rs -j 1 --gitignore=true
  --output /tmp/mutants-rom-lib-proactive -- --lib` reports `26 mutants tested
  in 28s: 24 caught, 2 unviable`, with zero missed mutants and zero timeouts.
- Native ROM validation behaviour is unchanged. Existing ROM header/hash/search
  tests catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive data level-layout mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-data/src/level_layout.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-data -f crates/wildside-data/src/level_layout.rs -j 1
  --gitignore=true --output /tmp/mutants-data-level-layout-proactive -- --lib`
  reports `30 mutants tested in 53s: 30 caught`, with zero missed mutants and
  zero timeouts.
- Native data level-layout behaviour is unchanged. Existing ASM-cited layout
  helper tests catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive render level mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-render/src/level.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/level.rs -j 1 --gitignore=true
  --output /tmp/mutants-render-level-proactive-1 -- --lib` reports `41 mutants
  tested in 72s: 36 caught, 5 unviable`, with zero missed mutants and zero
  timeouts.
- Native/render level visual parsing behaviour is unchanged. Existing level
  tests catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive render tiles mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-render/src/tiles.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/tiles.rs -j 1 --gitignore=true
  --output /tmp/mutants-render-tiles-proactive-1 -- --lib` reports `38 mutants
  tested in 68s: 34 caught, 4 unviable`, with zero missed mutants and zero
  timeouts.
- Native/render tile parsing behaviour is unchanged. Existing tile tests catch
  this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive render palette mutation shard)

- **Fallback verification completed.** With no ready/open rows, ran a focused
  mutation shard on `crates/wildside-render/src/palette.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/palette.rs -j 1
  --gitignore=true --output /tmp/mutants-render-palette-proactive-1 -- --lib`
  reports `35 mutants tested in 67s: 33 caught, 2 unviable`, with zero missed
  mutants and zero timeouts.
- Native/render palette behaviour is unchanged. Existing palette tests catch
  this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive frame renderer mutation hardening)

- **Fallback verification hardened renderer test gaps.** With no ready/open
  rows, ran a focused mutation shard on `crates/wildside-frame/src/render.rs`.
- Evidence: initial `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-frame -f crates/wildside-frame/src/render.rs -j 1 --gitignore=true
  --output /tmp/mutants-frame-render-proactive-1 -- --lib` reported `288
  mutants tested in 5m: 50 missed, 231 caught, 7 unviable`; missed mutants were
  concentrated in public frame composition, panel/text drawing, and
  sprite-offset blits.
- Fix: added focused renderer unit tests for full-frame blank-canvas output,
  scene clear-vs-overlay behaviour, panel border rects, scaled text/newlines,
  and sprite tile offsets.
- Result: final rerun at `/tmp/mutants-frame-render-proactive-3` reports `288
  mutants tested in 5m: 281 caught, 7 unviable`, with zero missed mutants.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive frame HUD equivalent-mutant cleanup)

- **Fallback verification cleaned up an equivalent mutant.** With no
  ready/open rows, ran a focused mutation shard on
  `crates/wildside-frame/src/hud.rs`.
- Evidence: initial `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-frame -f crates/wildside-frame/src/hud.rs -j 1 --gitignore=true
  --output /tmp/mutants-frame-hud-proactive-1 -- --lib` reported `170 mutants
  tested in 3m: 1 missed, 160 caught, 9 unviable`; the missed mutant replaced
  `%` with `+` in `hud_diamond_palette_loc_1223c`.
- Fix: the survivor was equivalent because rotating the four-colour foreground
  palette window by `n + 4` produces the same visible palette as `n`; removed
  the redundant modulo and documented the natural wrap.
- Result: rerun at `/tmp/mutants-frame-hud-proactive-2` reports `168 mutants
  tested in 3m: 159 caught, 9 unviable`, with zero missed mutants.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive render background mutation shard)

- **Fallback verification completed.** With no ready/open rows, checked
  `crates/wildside-frame/src/lib.rs` first; it has no non-test mutation
  surface, so continued to `crates/wildside-render/src/background.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/background.rs -j 1
  --gitignore=true --output /tmp/mutants-render-background-proactive-1 --
  --lib` reports `69 mutants tested in 2m: 62 caught, 7 unviable`, with zero
  missed mutants and zero timeouts.
- Native/render background behaviour is unchanged. Existing background tests
  catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive frame DTO mutation hardening)

- **Fallback verification hardened a test gap.** With no ready/open rows, ran a
  focused mutation shard on `crates/wildside-frame/src/dto.rs`.
- Evidence: initial `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-frame -f crates/wildside-frame/src/dto.rs -j 1 --gitignore=true
  --output /tmp/mutants-frame-dto-proactive-1 -- --lib` reported `2 mutants
  tested in 24s: 2 missed`; both missed mutants replaced
  `enemy_family_colour` with constant `[0; 3]` / `[1; 3]` output.
- Fix: added a DTO regression for the documented single degraded-asset fallback
  colour across representative object types.
- Result: rerun at `/tmp/mutants-frame-dto-proactive-2` reports `2 mutants
  tested in 23s: 2 caught`.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive data asset-source mutation hardening)

- **Fallback verification hardened a test gap.** With no ready/open rows, first
  attempted `crates/wildside-md/src/lib.rs`, but stopped it as too broad for a
  loop shard after it enumerated 1352 mutants and produced no useful progress
  output after the baseline. Switched to
  `crates/wildside-data/src/asset_source.rs`.
- Evidence: initial `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-data -f crates/wildside-data/src/asset_source.rs -j 1
  --gitignore=true --output /tmp/mutants-data-asset-source-proactive-1 --
  --lib` reported `15 mutants tested in 37s: 2 missed, 13 caught`; the missed
  mutants replaced `InMemoryAssetSource::len` with `1` and
  `InMemoryAssetSource::is_empty` with `true`.
- Fix: extended the public in-memory asset-source regression to prove
  non-empty state after insertion, two distinct stored paths, and replacement
  preserving length.
- Result: rerun at `/tmp/mutants-data-asset-source-proactive-2` reports `15
  mutants tested in 35s: 15 caught`.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused shard.

## 2026-07-01 handoff (proactive ROM library mutation shard)

- **Fallback verification completed.** With no ready/open rows, continued the
  direct mutation rotation on `crates/wildside-rom/src/lib.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-rom -f crates/wildside-rom/src/lib.rs -j 1 --gitignore=true
  --output /tmp/mutants-rom-lib-proactive-1 -- --lib` reports `26 mutants
  tested in 29s: 24 caught, 2 unviable`, with zero missed mutants and zero
  timeouts.
- Native/ROM validation behaviour is unchanged. Existing ROM tests already
  catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears,
  continue `loop.md` proactive verification on another focused MD shard.

## 2026-07-01 handoff (proactive data level-layout mutation shard)

- **Fallback verification completed.** With no ready/open rows, finished the
  small render rotation probe on `crates/wildside-render/src/hud.rs`; it has no
  non-test mutants, so continued to `crates/wildside-data/src/level_layout.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/hud.rs -j 1 --gitignore=true
  --output /tmp/mutants-render-hud-proactive-1 -- --lib` reports `Found 0
  mutants to test`; `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-data -f crates/wildside-data/src/level_layout.rs -j 1
  --gitignore=true --output /tmp/mutants-data-level-layout-proactive-1 --
  --lib` reports `30 mutants tested in 53s: 30 caught`, with zero missed
  mutants and zero timeouts.
- Native/data extraction behaviour is unchanged. Existing level-layout tests
  catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused data/ROM/MD shard.

## 2026-07-01 handoff (proactive render tiles mutation shard)

- **Fallback verification completed.** With no ready/open rows, continued the
  direct render mutation rotation on `crates/wildside-render/src/tiles.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/tiles.rs -j 1
  --gitignore=true --output /tmp/mutants-render-tiles-proactive-1 -- --lib`
  reports `38 mutants tested in 68s: 34 caught, 4 unviable`, with zero missed
  mutants and zero timeouts.
- Native/render behaviour is unchanged. Existing render-tiles tests already
  catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused render/data/MD shard.

## 2026-07-01 handoff (proactive render palette mutation shard)

- **Fallback verification completed.** With no ready/open rows, continued the
  direct render mutation rotation. `crates/wildside-render/src/player.rs`
  produced no non-test mutants, so the useful shard was
  `crates/wildside-render/src/palette.rs`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/player.rs -j 1
  --gitignore=true --output /tmp/mutants-render-player-proactive-1 -- --lib`
  reports `Found 0 mutants to test`; `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)"
  cargo mutants -p wildside-render -f crates/wildside-render/src/palette.rs -j
  1 --gitignore=true --output /tmp/mutants-render-palette-proactive-1 --
  --lib` reports `35 mutants tested in 65s: 33 caught, 2 unviable`, with zero
  missed mutants and zero timeouts.
- Native/render behaviour is unchanged. Existing render-palette tests already
  catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused render/data/MD shard.

## 2026-07-01 handoff (proactive render level mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-render/src/level.rs` directly because the local
  `mise run mutants-file` wrapper is scoped to `wildside-core`.
- Evidence: `TMPDIR=/tmp KID_REPO_ROOT="$(pwd)" cargo mutants -p
  wildside-render -f crates/wildside-render/src/level.rs -j 1
  --gitignore=true --output /tmp/mutants-render-level-proactive-1 -- --lib`
  reports `41 mutants tested in 75s: 36 caught, 5 unviable`, with zero missed
  mutants and zero timeouts.
- Native/render behaviour is unchanged. Existing render-level tests already
  catch this shard's viable mutations.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused render/data/MD shard.

## 2026-07-01 handoff (proactive projectile mutation hardening)

- **Fallback verification found gaps.** With no ready/open rows, started the
  single-file mutation wrapper against `projectiles.rs`; stopped it after the
  early survivor report rather than spending the whole large file run.
- Evidence: `/tmp/mutants-projectiles-proactive-1` reported survivors at
  `DRIPS_SURFACE_CRAWLER_KID_CONTACT_HALF_WIDTH_LOC_33550` (`+` -> `*`) and
  `BOSS_DEATH_BURST_ENTRY_ANIMATION_FRAMES_STRU_37BD8` (`+` -> `-`).
- Added ASM-cited regressions for Drips `loc_33550` rejecting the live crawler
  at |dx|=16 while keeping the existing |dx|=15 issue-260 hit, and for
  `stru_37BD8`'s exact 5+5+$A+1 frame sum.
- Verification: both targeted tests pass cleanly, manually injecting each
  original survivor mutation makes its new test fail, and `mise run
  verify-slice-fast` passes.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Dragon Walking enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/dragon_walking.rs`.
- The slice covers the ASM-cited Dragon Walking child init, HP setup and
  animation selection, collision routing, Berzerker knockback, breath
  trigger/object/child poll/recovery paths, shared-walker pacing and terrain
  blocking, live breath projectile spawn, damage/death routes, and spawned-child
  resume dispatch.
- Evidence: `/tmp/mutants-dragon-walking-proactive-2` reports `136 mutants
  tested in 13m: 112 caught, 24 unviable`.
- Verification: the Dragon Walking mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Dragon enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/dragon.rs`.
- The slice covers the ASM-cited Dragon drift/hover DBF tails, face-player
  motion, descent script movement and sentinels, terrain collision dispatch,
  slide/rebound/return-to-anchor recovery paths, descent fire projectile setup,
  fire-breath cone geometry, death sequence, and delete/respawn tails.
- Evidence: `/tmp/mutants-dragon-proactive-1` reports `264 mutants tested in
  26m: 222 caught, 42 unviable`.
- Verification: the Dragon mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive UFO enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/ufo.rs`.
- The slice covers the ASM-cited UFO motion-table/scaling helpers, on-screen
  count/latch gates, tracking command selection, beam start/load/clear timing,
  random and targeted beam routes, collision dispatch/clear, terrain rollback
  and bounce tails, hurt driver/debris spawning, and beam-child init/turnaround
  paths.
- Evidence: `/tmp/mutants-ufo-proactive-1` reports `232 mutants tested in 23m:
  194 caught, 38 unviable`.
- Verification: the UFO mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Tar Monster enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/tar_monster.rs`.
- The slice covers the ASM-cited Tar Monster control-script dispatch, wait and
  appear/sound sequencing, collision gates, Berzerker side-contact routing,
  hurt/death/status updates, straight and retaliation shot spawning, burst child
  setup, projectile metadata, collision dispatch, and loop/delete tails.
- Evidence: `/tmp/mutants-tar-monster-proactive-1` reports `141 mutants tested
  in 13m: 107 caught, 34 unviable`.
- Verification: the Tar Monster mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Mini Hopping Skull enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/mini_hopping_skull.rs`.
- The slice covers the ASM-cited Mini Hopping Skull air-sprite selection,
  collision dispatch, slope/terrain contacts, grounded wait and windup routes,
  scream-worker allocation, Berzerker side-contact gates, damage/death paths,
  landing tails, and relaunch loops.
- Evidence: `/tmp/mutants-mini-hopping-skull-proactive-1` reports `146 mutants
  tested in 13m: 90 caught, 56 unviable`.
- Verification: the Mini Hopping Skull mutation run completed with no survivors;
  no source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Ninja enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/ninja.rs`.
- The slice covers the ASM-cited Ninja init, Kid/projectile threat routing,
  guard and jump-threat poses, stealth pacing reset/resume routes,
  jump-attack launch/airborne/collision paths, wall bounce, landing,
  Kid-above contact, and death/delete routing.
- Evidence: `/tmp/mutants-ninja-proactive-1` reports `189 mutants tested in
  18m: 156 caught, 33 unviable`.
- Verification: the Ninja mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Hand enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/hand.rs`.
- The slice covers the ASM-cited Hand jump seed/components/gravity, ground and
  jump collision dispatch, Kid-contact gates, death/delete/respawn paths,
  terrain scan, grab snap/timer/release, init, patrol/reversal, detection, and
  live-step branches.
- Evidence: `/tmp/mutants-hand-proactive-1` reports `167 mutants tested in
  17m: 145 caught, 22 unviable`.
- Verification: the Hand mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Tank enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/tank.rs`.
- The slice covers the ASM-cited Tank and Tank_Shooting art-slot selection,
  visible init, shared drive sound/refcount, mouth animation sound cadence,
  child-collision damage/death routes, shooting-tank collision/damage/death
  routes, bullet lifecycle, and live-step audio edges.
- Evidence: `/tmp/mutants-tank-proactive-1` reports `28 mutants tested in 3m:
  21 caught, 7 unviable`.
- Verification: the Tank mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Crab enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/crab.rs`.
- The slice covers the ASM-cited Crab init, collision dispatch, Berzerker
  knockback, airborne deceleration/collision/gravity/slope response, damage and
  death routing, proximity freeze/unfreeze gates, and live-step branches.
- Evidence: `/tmp/mutants-crab-proactive-1` reports `87 mutants tested in 9m:
  71 caught, 16 unviable`.
- Verification: the Crab mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Cloud enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/cloud.rs`.
- The slice covers the ASM-cited Cloud command-script dispatch, wait/status
  commands, idle sprite gate, diagonal attack setup, projectile lifecycle,
  death/respawn routing, wall reversal, and live-step branches.
- Evidence: `/tmp/mutants-cloud-proactive-1` reports `134 mutants tested in
  13m: 118 caught, 16 unviable`.
- Verification: the Cloud mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Diamond enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/diamond.rs`.
- The slice covers the ASM-cited Diamond high-HP init/collision/damage paths,
  low-HP death/hurt/shared-walker paths, slope velocity mapping, Berzerker
  side-contact launch/tick routing, and live-step branches.
- Evidence: `/tmp/mutants-diamond-proactive-1` reports `67 mutants tested in
  7m: 57 caught, 10 unviable`.
- Verification: the Diamond mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Robot enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/robot.rs`.
- The slice covers the ASM-cited Robot walk speed, jump sound edge, collision
  dispatch, charge entry/direction, shooting windup/hold/recovery, shooting
  collision and Berzerker side-contact routing, projectile spawn/flight/impact,
  child/head state, and live-step branches.
- Evidence: `/tmp/mutants-robot-proactive-1` reports `105 mutants tested in
  11m: 93 caught, 12 unviable`.
- Verification: the Robot mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Orca enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/orca.rs`.
- The slice covers the ASM-cited Orca init, patrol collision dispatch, damage
  and death routing, lunge decision/windup/launch, lunge collision and landing
  resume paths, Berzerker side-contact/airborne routing, and live-step branches.
- Evidence: `/tmp/mutants-orca-proactive-1` reports `91 mutants tested in 9m:
  74 caught, 17 unviable`.
- Verification: the Orca mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Emo Rock enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/emo_rock.rs`.
- The slice covers the ASM-cited Emo Rock wait/hover target, drop velocity and
  landing setup, settled contact dispatch, Berzerker side-hit/airborne branches,
  disappear worker, death start, respawn/status routing, and live-step branches.
- Evidence: `/tmp/mutants-emo-rock-proactive-1` reports `72 mutants tested in
  7m: 61 caught, 11 unviable`.
- Verification: the Emo Rock mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Big Hopping Skull enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/big_hopping_skull.rs`.
- The slice covers the ASM-cited Big Hopping Skull startup and level-height
  delete gate, contact wait probe, damage entry, hit groan/voice gates,
  hit-animation wait, death/respawn/delete-tail routing, relaunch, bounds
  correction, and live-step branches.
- Evidence: `/tmp/mutants-big-hopping-skull-proactive-1` reports `71 mutants
  tested in 7m: 56 caught, 15 unviable`.
- Verification: the Big Hopping Skull mutation run completed with no survivors;
  no source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Spinning Twins enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/spinning_twins.rs`.
- The slice covers the ASM-cited Spinning Twins orbit/drop transition, walking
  velocity gate, airborne collision dispatch, floor/wall settle paths, settled
  collision routing, Berzerker launch loop, damage/death routing, and falling
  visual-step helper.
- Evidence: `/tmp/mutants-spinning-twins-proactive-1` reports `126 mutants
  tested in 12m: 105 caught, 21 unviable`.
- Verification: the Spinning Twins mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Archer enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/archer.rs`.
- The slice covers the ASM-cited Archer shot-selection tables, opening-angle and
  held-arrow setup, pose-frame positioning, arrow delete accounting,
  death/respawn routing, and live-step branches.
- Evidence: `/tmp/mutants-archer-proactive-2` reports `135 mutants tested in
  14m: 124 caught, 11 unviable`.
- Verification: the Archer mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Fire Demon enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/fire_demon.rs`.
- The slice covers the ASM-cited Fire Demon HP-based init animation, palette
  pulse worker, fire-trail wait/delete loop, collision dispatch, hurt/death
  routing, and Berzerker side-contact launch paths.
- Evidence: `/tmp/mutants-fire-demon-proactive-2` reports `48 mutants tested in
  5m: 30 caught, 18 unviable`.
- Verification: the Fire Demon mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Driller enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/driller.rs`.
- The slice covers the ASM-cited Driller init tail, Smash/Prize/Ice block
  drilling dispatch, child-space drill probe, collision dispatch, and live-step
  grid mutation paths.
- Evidence: `/tmp/mutants-driller-proactive-2` reports `24 mutants tested in
  3m: 19 caught, 5 unviable`.
- Verification: the Driller mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Goat enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/goat.rs`.
- The slice covers the ASM-cited Goat HP-based init animation, collision
  dispatch, death sequence, Berzerker side-contact launch/shove paths, and
  collision-clear resume routing.
- Evidence: `/tmp/mutants-goat-proactive-2` reports `23 mutants tested in 3m:
  15 caught, 8 unviable`.
- Verification: the Goat mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Scorpion enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/scorpion.rs`.
- The slice covers the ASM-cited Scorpion tracking, close-range sting/tail
  arming, linked tail sync, collision dispatch, Berzerker side-contact, and
  recovery/resume branches.
- Evidence: `/tmp/mutants-scorpion-proactive-1` reports `57 mutants tested in
  6m: 46 caught, 11 unviable`.
- Verification: the Scorpion mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Armadillo enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/armadillo.rs`.
- The slice covers the ASM-cited Armadillo damage routing, Berzerker side-contact
  launch/recovery paths, status-table update helpers, and spawned-child
  death/fall branches.
- Evidence: `/tmp/mutants-armadillo-proactive-1` reports `46 mutants tested in
  5m: 34 caught, 12 unviable`.
- Verification: the Armadillo mutation run completed with no survivors; no
  source regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Tornado enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, ran the
  single-file mutation wrapper against `enemies/tornado.rs`.
- The slice covers the ASM-cited Tornado collision dispatch, UFO-shared terrain
  bounce application, hit/status-table sequence, and Cyclone side-bounce helper
  paths.
- Evidence: `/tmp/mutants-tornado-proactive-1` reports `70 mutants tested in
  7m: 62 caught, 8 unviable`.
- Verification: the Tornado mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive Fireball enemy mutation check)

- **Fallback verification checked.** With no ready/open rows, sampled
  `enemies/sphere.rs` first; it produced only unviable mutants, so it did not
  expose a useful test blind spot.
- The useful slice was `enemies/fireball.rs`, covering the ASM-cited fireball
  trail lifetime, sprite branch/ratio selection, commit, marker, and delete-tail
  helpers already present in the module tests.
- Evidence: `/tmp/mutants-sphere-proactive-1` reports `3 mutants tested in
  50s: 3 unviable`; `/tmp/mutants-fireball-proactive-1` reports `60 mutants
  tested in 6m: 47 caught, 13 unviable`.
- Verification: the Fireball mutation run completed with no survivors; no source
  regression was warranted.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused core or frame shard.

## 2026-07-01 handoff (proactive frame sprite tiling/player-frame hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-frame` sprite mutation pass into generic sprite tiling and
  `select_player_frame` guard edges.
- This pass adds exact coverage for 96x96 multi-band/multi-chunk sprite mapping
  and ASM-cited player-frame guard paths: loc_ABF0 pull-up saturation, Cyclone
  flight gating, sub_B270 airborne slash fallback, Iron Knight climb clamping,
  loc_A13A slope fallback tables, and the airborne zero-velocity fall pose.
- Evidence: focused rerun `/tmp/mutants-frame-sprites-tiling-player-frame-1`
  reports `124 mutants tested in 47s: 122 caught, 2 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  sprites::tests -- --nocapture`, and the focused mutation rerun completed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused frame/core shard.

## 2026-07-01 handoff (proactive frame sprite push hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-frame` sprite mutation pass into render push/palette helpers.
- Initial focused push/palette sampling `/tmp/mutants-frame-sprites-push-1`
  reported `25 mutants tested in 32s: 8 missed, 12 caught, 5 unviable`.
- This pass adds command-stream coverage for
  `push_player_sprite_with_priority_override`, death windup art, flagpole
  priority override, boss-eye marker placement, player/enemy damage flash
  palettes, and Heady Metal sub_379E8 palette slot writes.
- Evidence: focused rerun `/tmp/mutants-frame-sprites-push-2` reports `25
  mutants tested in 32s: 20 caught, 5 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  sprites::tests -- --nocapture`, and the focused mutation rerun completed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused frame/core shard.

## 2026-07-01 handoff (proactive frame sprite platform hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-frame` sprite mutation pass into platform rendering helpers.
- Initial focused platform sampling `/tmp/mutants-frame-sprites-platforms-1`
  reported `52 mutants tested in 40s: 47 missed, 5 unviable`; the helpers had
  no direct command-emission coverage.
- This pass adds synthetic `NativeLevelVisuals` coverage for
  `push_platform_tiles`, loc_24AC/loc_2548 cell translation, loc_2822 art
  mapping words, fallback art selection, camera-relative culling, tile offsets,
  palettes, and missing debug fields.
- Evidence: focused platform rerun `/tmp/mutants-frame-sprites-platforms-3`
  reports `52 mutants tested in 39s: 47 caught, 5 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  sprites::tests -- --nocapture`, and the focused mutation rerun completed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another `sprites.rs` render push helper
  or a focused frame/core shard.

## 2026-07-01 handoff (proactive frame sprite selector hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-frame` sprite mutation pass into enemy selector families.
- Prior full `sprites.rs` mutation sampling `/tmp/mutants-frame-sprites-5`
  reported `969 mutants tested in 6m: 667 missed, 243 caught, 59 unviable`;
  remaining misses were concentrated in enemy/platform selectors and render
  push integration helpers.
- This pass adds frame-local synthetic enemy visuals and ASM-cited selector
  tests for live guard paths/body phases, `select_enemy_frame` death scripts,
  waited death sequences, and the generic fallback animation path.
- Evidence: focused selector rerun `/tmp/mutants-frame-sprites-selectors-5`
  reports `521 mutants tested in 3m: 56 missed, 435 caught, 30 unviable`.
  Remaining misses are mostly boundary-equivalent live guards and forced-true
  sheet-length guards.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  sprites::tests -- --nocapture`, and the focused mutation rerun completed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on `sprites.rs` platform rendering helpers
  or another focused frame/core shard.

## 2026-07-01 handoff (proactive frame sprite helper hardening)

- **Fallback verification hardened.** With no ready/open rows, moved from HUD
  to `wildside-frame` sprite helpers.
- Broad initial `sprites.rs` mutation sampling showed immediate misses in
  loc_39EBE boss-head death animation helpers, generic sprite tile splitting,
  `select_player_frame`, transform palette/step helpers, player screen
  coordinates, and boss-eye marker offsets.
- This pass adds focused pure tests for those helper surfaces: boss-head
  death frame/facing boundaries, tile chunk/band splitting and x-flip offsets,
  selected ASM-cited player-frame branches, transform-step selection,
  loc_7CD0 Kid-palette detection, fixed-point player screen position, and
  sub_37ABC boss-eye marker commands.
- Evidence: full rerun `/tmp/mutants-frame-sprites-5` reports `969 mutants
  tested in 6m: 667 missed, 243 caught, 59 unviable`. Remaining misses are
  dominated by enemy/platform sprite selector families and render-push
  integration helpers, not the first pure helper cluster.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  sprites::tests -- --nocapture`, and the focused mutation rerun completed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on `sprites.rs` enemy/platform selector
  families or another focused frame/core shard.

## 2026-07-01 handoff (proactive frame HUD aggregate hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-frame` HUD mutation pass into full aggregate `push_game_hud`
  command emission.
- Prior HUD command shard `/tmp/mutants-frame-hud-5` reported `150 mutants
  tested in 61s: 116 caught, 25 missed, 9 unviable`.
- This pass adds full HUD command-stream coverage for HP pips, timer digits,
  lives/diamond counters, and animated ankh/diamond sprites. It also adds
  boundary coverage for counter value 10, truncated HP sheets, and 10-colour
  loc_1223C palette rotation.
- Evidence: focused rerun `/tmp/mutants-frame-hud-8` reports `170 mutants
  tested in 67s: 160 caught, 1 missed, 9 unviable`. The remaining `% 4` -> `+
  4` palette mutant is equivalent because rotating four palette slots by an
  extra four iterations gives the same ordering.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  hud::tests -- --nocapture`, and the focused mutation rerun passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused frame/core shard.

## 2026-07-01 handoff (proactive frame HUD command hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-frame` HUD mutation pass from helper-only coverage into command
  emission paths.
- Prior HUD shard `/tmp/mutants-frame-hud-3` reported `150 mutants tested in
  59s: 77 caught, 64 missed, 9 unviable`.
- This pass adds focused command/palette coverage for HUD digit counters,
  wide/narrow hitpoint sprite emission, transform-flash HP visibility,
  loc_1223C diamond palette rotation, and ankh/diamond/fallback HUD icon
  commands.
- Evidence: focused rerun `/tmp/mutants-frame-hud-5` reports `150 mutants
  tested in 61s: 116 caught, 25 missed, 9 unviable`. Remaining misses are mostly
  aggregate `push_game_hud` arithmetic plus equivalent boundary mutants.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  hud::tests -- --nocapture`, and the focused mutation rerun passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, either
  continue with full `push_game_hud` aggregate command tests or move to another
  focused frame/core shard.

## 2026-07-01 handoff (proactive frame HUD helper hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-frame` proactive mutation pass on HUD helper logic.
- Initial HUD shard `/tmp/mutants-frame-hud-1` reported `150 mutants tested in
  60s: 141 missed, 9 unviable`.
- This pass adds focused HUD helper coverage for loc_BE0C hitpoint slot/tail
  pips, loc_45EE diamond page lead, loc_4546/loc_4586 life icon pages, loc_C070
  visible timer seconds, and loc_BD0A timer coordinate/layout decisions.
- Evidence: focused rerun `/tmp/mutants-frame-hud-3` reports `150 mutants
  tested in 59s: 77 caught, 64 missed, 9 unviable`. Remaining misses are
  command-emission/asset-backed HUD rendering paths.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  hud::tests -- --nocapture`, and the focused mutation rerun passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification with HUD command emission or another focused
  frame/core shard.

## 2026-07-01 handoff (proactive frame render primitive hardening)

- **Fallback verification hardened.** With no ready/open rows, switched lanes
  from `wildside-md` to `wildside-frame` render primitives.
- Initial render primitive shard `/tmp/mutants-frame-render-primitives-1`
  reported `244 mutants tested in 82s: 188 missed, 50 caught, 6 unviable`.
- This pass adds focused render primitive/provenance coverage for command
  bounds, text glyph source mapping and cursor advancement, indexed tile
  row/slice/source coordinates, priority clearing, palette resolution, and
  scene-pixel acceptance. It also rewrites equivalent disjoint-bit and compact
  palette guard forms so mutation has an observable target.
- Evidence: focused rerun `/tmp/mutants-frame-render-primitives-5` reports
  `237 mutants tested in 83s: 231 caught, 6 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-frame
  render::tests -- --nocapture`, and the focused mutation rerun passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused render/frame/core shard.

## 2026-07-01 handoff (proactive MD VDP diagnostic render hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-md` helper rotation after the GEMS pass.
- Clean/no-code-change shard: audio mixer `/tmp/mutants-md-audio-mixer-1`
  reported `157 mutants tested in 53s: 157 caught`.
- Initial VDP render helper shard `/tmp/mutants-md-vdp-render-1` reported `433
  mutants tested in 2m: 20 missed, 399 caught, 14 unviable`.
- This pass adds diagnostic VDP sample coverage for scrolled plane provenance,
  exclusive window edges, low-priority window samples, and flipped tile source
  coordinates. The main RGB render paths were already covered; the gap was the
  trace/debug sample surface.
- Evidence: focused rerun `/tmp/mutants-md-vdp-render-2` reports `433 mutants
  tested in 2m: 419 caught, 14 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-md sample_ --
  --nocapture`, `cargo test -q -p wildside-md tile_pixel -- --nocapture`, and
  the focused mutation rerun passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on remaining helper shards or switch lanes
  to another focused mutation/audit slice.

## 2026-07-01 handoff (proactive MD GEMS mutation hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  `wildside-md` helper rotation after the bus/VDP pass.
- Clean/no-code-change shards: controller `/tmp/mutants-md-controller-1`
  reported `78 mutants tested in 30s: 78 caught`; Z80 bus
  `/tmp/mutants-md-z80bus-1` reported `61 mutants tested in 26s: 61 caught`.
- Initial standalone GEMS shard `/tmp/mutants-md-gems-1` reported `91 mutants
  tested in 50s: 2 missed, 86 caught, 2 unviable, 1 timeouts`.
- This pass matches standalone GEMS bank-register composition to `Z80Bus`'s
  additive form and rewrites the frame-cycle loop to consume `remaining`
  directly with `saturating_sub`, removing the unobservable disjoint-bit
  survivor and timeout-only loop mutant.
- Evidence: focused rerun `/tmp/mutants-md-gems-3` reports `86 mutants tested
  in 32s: 84 caught, 2 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-md
  standalone_gems -- --nocapture`, and the focused mutation rerun passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on the remaining `wildside-md` audio mixer
  or VDP render helper shards.

## 2026-07-01 handoff (proactive MD bus/VDP mutation hardening)

- **Fallback verification hardened.** With no ready/open rows, continued the
  mutation rotation through clean/no-code-change shards: render `level.rs`
  (`41 mutants tested in 50s: 36 caught, 5 unviable`), render `player.rs`
  (zero candidates), render `palette.rs` (`35 mutants tested in 47s: 33
  caught, 2 unviable`), render `tiles.rs` (`38 mutants tested in 48s: 34
  caught, 4 unviable`), render `hud.rs` (zero candidates), data
  `level_layout.rs` (`30 mutants tested in 29s: 30 caught`), ROM `lib.rs`
  (`26 mutants tested in 11s: 24 caught, 2 unviable`), and the MD
  audio/register shard (`154 mutants tested in 51s: 149 caught, 5 unviable`).
- Initial MD bus/VDP shard `/tmp/mutants-md-bus-vdp-1` reported `312 mutants
  tested in 2m: 3 missed, 305 caught, 4 unviable`.
- This pass adds a regression proving `AddressBus::copy_from` replays traced
  YM2612 ports through `port & 3` when restoring the shadow chip, and rewrites
  `read_long` composition from equivalent disjoint-bit `|` to `+` to remove the
  untestable survivor.
- Evidence: focused rerun `/tmp/mutants-md-bus-vdp-2` reports `312 mutants
  tested in 2m: 308 caught, 4 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-md
  address_bus -- --nocapture`, `mise run backlog-lint`, and `mise run
  verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification by sharding the remaining `wildside-md`
  helpers (controller/Z80/GEMS/audio/rendering) before switching lanes.

## 2026-07-01 handoff (proactive render background mutation hardening)

- **Fallback verification hardened.** With no ready/open rows, checked the next
  rotation file `crates/wildside-render/src/lib.rs`; it has zero non-test
  mutants, so continued to `crates/wildside-render/src/background.rs`.
- Initial background run `/tmp/mutants-render-background-1` reported `69
  mutants tested in 67s: 1 missed, 61 caught, 7 unviable`.
- This pass adds an ASM-cited `loc_12830` regression for a complete chunk,x,y
  background layout entry followed by an odd stray byte: native should report a
  missing negative terminator, not a truncated placement.
- Evidence: focused rerun `/tmp/mutants-render-background-2` reports `69
  mutants tested in 68s: 62 caught, 7 unviable`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-render
  background -- --nocapture`, `mise run backlog-lint`, and `mise run
  verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on the next rotation file.

## 2026-07-01 handoff (proactive render bundle mutation hardening)

- **Fallback verification hardened.** With no ready/open rows, mutation-tested
  a focused `crates/wildside-render/src/bundle.rs` helper shard covering pause
  rows, EOL score crops, EOL hologram sheet composition, shared VRAM bases, and
  palette permutation lookups.
- Initial shard `/tmp/mutants-render-bundle-helpers-1` reported `51 mutants
  tested in 60s: 13 missed, 35 caught, 3 unviable`.
- This pass adds ASM-cited synthetic regressions for `End_level_stats_text`
  score sprite crop grid bounds and EOL hologram `$2FB` HoloBG / `$409`
  HoloBlocks overlay placement.
- Evidence: focused rerun `/tmp/mutants-render-bundle-helpers-2` reports `51
  mutants tested in 59s: 1 missed, 47 caught, 3 unviable`. The remaining
  survivor is equivalent: `dest < tiles.len()` to `<=` in the HoloBG copy
  cannot observe `dest == len` because `tile_count` is `base + background_len`
  or larger, and the loop's last destination is `base + background_len - 1`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-render eol_ --
  --nocapture`, `mise run backlog-lint`, and `mise run verify-slice-fast`
  passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on the next rotation file.

## 2026-07-01 handoff (proactive trace helper mutation hardening)

- **Fallback verification hardened.** With no ready/open rows, mutation-tested
  a focused helper shard in `crates/wildside-core/src/trace.rs` covering
  floating-diamond horizontal bands, trace-cell formatting, collected helmet
  visibility, and helmet prize mapping.
- Initial shard `/tmp/mutants-trace-helpers-1` reported `56 mutants tested in
  3m: 11 missed, 44 caught, 1 unviable`.
- This pass expands the collected-helmet first-forward-transform regression
  across all nine helmet prize forms and adds a collision-latched collected
  helmet visibility regression, citing `PrizeHelmet* loc_10464..loc_106B4` and
  `Kid_Transform loc_7CD0/loc_7D22`.
- Evidence: focused rerun `/tmp/mutants-trace-helpers-2` reports `56 mutants
  tested in 3m: 2 missed, 53 caught, 1 unviable`. The remaining two misses are
  equivalent `trace_cell_word` `|`→`^` mutations because the block flag,
  collision, and block-type fields are disjoint after masking.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-core
  trace_frame_keeps -- --nocapture`, `mise run backlog-lint`, and `mise run
  verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on the next rotation file.

## 2026-07-01 handoff (proactive VDP title-card mutation hardening)

- **Fallback verification hardened.** With no ready/open rows, continued VDP
  mutation hardening in `crates/wildside-core/src/vdp.rs`, targeting
  `Load_TitleCard` clear/fill and padding behaviour.
- Initial focused shard `/tmp/mutants-vdp-title-card-1` reported `39 mutants
  tested in 3m: 10 missed, 28 caught, 1 unviable`.
- This pass adds ASM-cited assertions for `loc_19B58` first/second/final
  `$0094` tilemap clear words and `loc_19C04` final bottom-padding word.
- Evidence: final focused rerun `/tmp/mutants-vdp-title-card-3` reports `39 mutants
  tested in 3m: 8 missed, 30 caught, 1 unviable`. The remaining eight misses
  are equivalent for the current stateless builder because `loc_19ABE` and
  `loc_19B6A` write zero into a fresh zeroed VDP.
- Additional checks: `/tmp/mutants-vdp-intro-fade-1` reported `52 mutants tested
  in 3m: 1 missed, 48 caught, 3 unviable`; that survivor is equivalent because
  `SEGA_SCREEN_INTRO_HANDOFF_ELAPSED ± elapsed` is already in the stable Sega
  VDP range for the cold-boot `elapsed 0..16` window.
  `/tmp/mutants-vdp-vsram-writer-1` reported `9 mutants tested in 79s: 9
  caught`.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-core
  title_card_vdp -- --nocapture`, `mise run backlog-lint`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP shard or a new file.

## 2026-07-01 handoff (proactive VDP title-screen sprite mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  continued mutation hardening in `crates/wildside-core/src/vdp.rs`, targeting
  title-screen sprite composition and timing.
- Initial focused shard `/tmp/mutants-vdp-title-screen-1` reported `42 mutants
  tested in 3m: 6 missed, 34 caught, 2 unviable`.
- This pass adds ASM-cited regressions for `loc_1BA60`/`loc_1BB10` direct
  title-return first-frame suppression, `loc_1B5E4` KID title sprite presence,
  and `loc_1BB52` cursor rows 0/1.
- Evidence: focused rerun `/tmp/mutants-vdp-title-screen-2` reports `42 mutants
  tested in 3m: 40 caught, 2 unviable`. Additional current checks:
  `/tmp/mutants-vdp-intro3-sprite-frame-1` (`17 mutants tested in 2m: 15
  caught, 2 unviable`) and `/tmp/mutants-vdp-sprite-table-1` (`41 mutants tested
  in 3m: 2 missed, 39 caught`). The sprite-table misses are equivalent for valid
  hardware attributes because palette, priority, and x-flip bits are disjoint in
  the game domain.
- Verification: `cargo fmt --check`,
  `cargo test -q -p wildside-core vdp::tests -- --nocapture`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP survivor family or a
  new file shard.

## 2026-07-01 handoff (proactive VDP intro-handoff mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  continued mutation hardening in `crates/wildside-core/src/vdp.rs`, targeting
  cold-boot and IntroSequence2 handoff timing.
- Initial focused shard `/tmp/mutants-vdp-intro-handoff-1` reported `59 mutants
  tested in 3m: 6 missed, 50 caught, 3 unviable`.
- This pass removes an unreachable post-dispatch segment-1 overlay, expresses
  the intro fade windows as explicit half-open ranges, rewrites the
  `loc_1BA02` sky-reveal loop clamp without the equivalent `min(frames - 1) +
  1` form, and adds a `loc_12EDE` cold-boot Sega handoff regression.
- Evidence: focused rerun `/tmp/mutants-vdp-intro-handoff-2` reports `50
  mutants tested in 3m: 1 missed, 46 caught, 3 unviable`. The remaining
  `handoff + elapsed` -> `handoff - elapsed` survivor is equivalent for this
  slice because the Sega VDP state is already stable by the handoff window.
- Verification: `cargo fmt --check`,
  `cargo test -q -p wildside-core vdp::tests -- --nocapture`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP survivor family or a
  new file shard.

## 2026-07-01 handoff (proactive VDP clean mutation shards)

- **Fallback verification continued.** With no ready/open rows,
  checked current focused mutation coverage in `crates/wildside-core/src/vdp.rs`
  without source changes.
- Clean focused shards: `/tmp/mutants-vdp-special-decompress-1` (`13 mutants
  tested in 2m: 13 caught`), `/tmp/mutants-vdp-intro5-particles-1` (`36 mutants
  tested in 3m: 36 caught`), `/tmp/mutants-vdp-enigma-decode-1` (`3 mutants
  tested in 61s: 3 caught`), `/tmp/mutants-vdp-cram-words-1` (`6 mutants tested
  in 73s: 6 caught`), `/tmp/mutants-vdp-intro5-fade-1` (`10 mutants tested in
  81s: 9 caught, 1 unviable`), and `/tmp/mutants-vdp-intro-text-script-1` (`36
  mutants tested in 3m: 32 caught, 1 unviable, 3 timeouts`).
- The text-script timeouts are malformed cursor-progress mutations; no fresh
  shard had missed mutants.
- Verification: `cargo fmt --check`,
  `cargo test -q -p wildside-core vdp::tests -- --nocapture`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP survivor family or a
  new file shard.

## 2026-07-01 handoff (proactive VDP decompression mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  continued mutation hardening in `crates/wildside-core/src/vdp.rs`, targeting
  `DecompressToVRAM loc_14300/loc_14314` and chunk-span survivors left by the
  wider VDP shard.
- This pass adds ASM-cited regressions proving that
  `write_decompress_to_vram_art_loc_14300_14314` preserves a non-zero VRAM base
  across rollover chunks and odd trailing bytes, and that
  `decompress_chunk_spans` emits no empty tail span at the exact first-buffer
  boundary.
- Evidence: focused rerun `/tmp/mutants-vdp-decompress-2` reports `37 mutants
  tested in 4m: 1 missed, 30 caught, 1 unviable, 5 timeouts`. The missed mutant
  is equivalent in this native wrapper because an oversized DBF count is capped
  by the actual buffered words; the timeout mutants corrupt chunk-span progress.
- Verification: `cargo fmt --check`,
  `cargo test -q -p wildside-core vdp::tests -- --nocapture`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP survivor family or a
  new file shard.

## 2026-07-01 handoff (proactive VDP intro-text mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  continued mutation hardening in `crates/wildside-core/src/vdp.rs`, targeting
  the IntroText `loc_1C7A0` script/cleanup survivor family left by the wider
  VDP shard.
- This pass adds ASM-cited regressions for IntroText2 and IntroText3 script
  table rows, plus `clear_intro_text_script_cells_loc_1c7a0` handling of the
  `$FF` terminator and `$FE,<duration>` delay marker.
- Evidence: focused rerun `/tmp/mutants-vdp-intro-text-2` reports `39 mutants
  tested in 3m: 35 caught, 1 unviable, 3 timeouts`; no mutants were missed.
  The three remaining timeouts are loop-progress mutations that stop the parser
  advancing.
- Verification: `cargo fmt`,
  `cargo test -q -p wildside-core vdp::tests -- --nocapture`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP survivor family or a
  new file shard.

## 2026-07-01 handoff (proactive VDP intro-5 particle mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  continued mutation hardening in `crates/wildside-core/src/vdp.rs`, targeting
  the IntroSequence5 particle worker survivor family left by the wider VDP
  shard.
- This pass adds ASM-cited regressions for `loc_1BE3C/loc_1BE74`: the first
  post-spawn particle frame must apply the signed x/y velocities before display,
  and the short `$28` particles must remain visible when the DBF counter reaches
  zero.
- Evidence: focused rerun `/tmp/mutants-vdp-intro5-particles-1` reports
  `19 mutants tested in 2m: 19 caught`.
- Verification: `cargo fmt --check`,
  `cargo test -q -p wildside-core vdp::tests -- --nocapture`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP survivor family or a
  new file shard.

## 2026-07-01 handoff (proactive VDP intro-boundary mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  continued mutation hardening in `crates/wildside-core/src/vdp.rs`, targeting
  intro sequence cold-boot, handoff, fade-hold, and sky-reveal survivor families
  left by the wider VDP shard.
- This pass adds ASM-cited regressions for cold-boot Sega handoff and fade
  boundaries (`loc_12EDE`, `sub_1B8F4`, `sub_1C204`), IntroSequence2 handoff
  fade boundaries (`loc_1B9D0`, `sub_1C204`), IntroSequence5 `loc_1BA90`
  worker elapsed and fade-to-`$AAA` CRAM state, and the `loc_1BA02`
  sky-reveal terminal clamp.
- Evidence: targeted rerun `/tmp/mutants-vdp-intro-boundaries-2` reports
  `65 mutants tested in 4m: 7 missed, 54 caught, 4 unviable`; follow-up
  `/tmp/mutants-vdp-coldboot-3` reports `13 mutants tested in 88s: 2 missed,
  10 caught, 1 unviable`.
  Remaining misses are equivalent/unreachable: a post-return `segment == 1`
  branch, saturated Sega handoff `+` -> `-`, full-palette `<=` fade clamps, and
  sky-reveal loop-bound mutations that leave the tested terminal upload slice
  unchanged.
- Verification: `cargo fmt --check`,
  `cargo test -q -p wildside-core vdp::tests -- --nocapture`, and
  `mise run verify-slice-fast` passed.
- NEXT: run `mise run project-status`; if no ready/open item appears, continue
  `loop.md` proactive verification on another focused VDP survivor family or a
  new file shard.

## 2026-07-01 handoff (proactive VDP mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  mutation-tested `crates/wildside-core/src/vdp.rs` via
  `OUTPUT=.cache/mutants/vdp-current FILE=crates/wildside-core/src/vdp.rs mise run mutants-file`.
- Initial evidence: full VDP shard reported `800 mutants tested in 48m:
  98 missed, 286 caught, 408 unviable, 8 timeouts`.
- This pass adds direct VDP helper regressions for `decode_enigma_tilemap_loc_134D4`,
  VSRAM big-endian word writes and end clipping, CRAM word decoding, `sub_1C204`
  palette fade-in/out progression, `loc_1BA60` title-return fade windows,
  IntroSequence3 sprite-frame metadata, `Make_SpriteFromGfxObject` normal and
  flipped sprite-entry packing, and VDP register-5 sprite-table masking.
- Follow-up evidence: targeted rerun `/tmp/mutants-vdp-focused-2` reports
  `82 mutants tested in 4m: 2 missed, 79 caught, 1 unviable`.
  The remaining two focused misses are equivalent `|` -> `^` substitutions in
  sprite pattern-attribute packing because palette, priority, and x-flip bits
  are disjoint. The full `vdp.rs` shard still has wider intro/decompression/text
  survivor families for future focused passes.
- Verification: `cargo fmt --check` and
  `cargo test -q -p wildside-core --lib -- --skip demo_ --skip published_tas
  --skip snapshot` passed.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive types mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  mutation-tested `crates/wildside-core/src/types.rs` via
  `OUTPUT=.cache/mutants/types-current FILE=crates/wildside-core/src/types.rs mise run mutants-file`.
- Initial evidence: full types shard reported `227 mutants tested in 25m:
  51 missed, 166 caught, 7 unviable, 3 timeouts`.
- This pass adds type-level regressions for `BlockInteractionState`
  evanescent/shooter reset and recycle helpers, Island shore DMA default
  deserialisation, full helmet-id mapping, controller bit layout, GEMS command
  names, serde skip predicates, authoritative CRC scope, ROM RNG ring offset
  and long-word byte order, level timer expiry boundaries, and the `loc_EC76`
  shooter-block projectile constructor.
- Follow-up evidence: targeted rerun `/tmp/mutants-types-target-1` reports
  `137 mutants tested in 7m: 3 missed, 133 caught, 1 unviable`; after adding
  the missing `seconds > 0 && sub_frames == 0` timer case, follow-up
  `/tmp/mutants-types-followup-1` reports `86 mutants tested in 5m: 2 missed,
  84 caught`.
  The two remaining misses are equivalent `FrameInput` bit-zero mutants:
  replacing `1 << 0` with `1 >> 0` still evaluates to the same mask bit.
- Verification: `cargo fmt --check` and
  `cargo test -q -p wildside-core --lib -- --skip demo_ --skip published_tas
  --skip snapshot` passed.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive physics mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  mutation-tested `crates/wildside-core/src/physics.rs` via
  `OUTPUT=.cache/mutants/physics-current FILE=crates/wildside-core/src/physics.rs mise run mutants-file`.
- Initial evidence: full physics shard reported `293 mutants tested in 30m:
  3 missed, 278 caught, 12 unviable`.
- This pass adds an ASM-cited `loc_728C` regression for the exact `$FFFFFB58 ==
  $1E` threshold when the shifted camera target equals `max_y`; the ROM does
  not take `blt.s loc_72A8` at equality and then decrements the look offset
  because the later `bcs` max-Y check is strict.
- Follow-up evidence: targeted camera/murder-wall rerun
  `/tmp/mutants-physics-camera-target` reports `34 mutants tested in 3m:
  1 missed, 33 caught`.
  The remaining `forward_murder_wall_camera_push_loc_7346` `<` -> `<=` mutant
  is equivalent at the exact permitted gap: `d0 == d1` already means
  `wall_px == camera_px - d1`, so the clamp path and no-adjust path preserve
  identical high/fractional words.
- Verification: `cargo test -q -p wildside-core loc_728c_bottom_look_adjust --
  --nocapture` and `mise run verify-slice-fast` passed.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive platform mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  mutation-tested `crates/wildside-core/src/platforms.rs` via
  `OUTPUT=.cache/mutants/platforms-current FILE=crates/wildside-core/src/platforms.rs mise run mutants-file`.
- Initial evidence: full platform shard reported `536 mutants tested in 57m:
  82 missed, 398 caught, 51 unviable, 5 timeouts`.
- This pass adds ASM-cited tests for platform collision scratch contact,
  Platform_RAM slot bounds, `$C` loader offsets, loaded-mask bits, active slot
  next-link order, state facade resync after `sub_23B4`, render metadata,
  all-zero `PlatformScript_Nothing` sentinel detection, and special selector
  classification.
- Follow-up evidence: targeted helper/model rerun
  `/tmp/mutants-platforms-target-2` reports `96 mutants tested in 5m: 4
  missed, 86 caught, 3 unviable, 3 timeouts`.
  Remaining misses are equivalent/non-observable in current runtime state:
  loaded-mask `<` vs `<=` at bit 64, loaded-mask OR/XOR with unique slot
  offsets, stale model side-array cleanup after the slot is already unlinked,
  and render flag OR/XOR over disjoint nibbles. The three slot-index alias
  mutants time out by corrupting the slot pool walk.
- Verification: `cargo test -q -p wildside-core platform_ -- --nocapture`,
  targeted `PlatformLoaderState::deallocate_layout_offset_sub_23B4` mutation
  rerun `/tmp/mutants-platforms-state-dealloc`, and `mise run verify-slice-fast`
  passed.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive weather mutation hardening)

- **Fallback verification hardened.** With no ready/open rows,
  mutation-tested `crates/wildside-core/src/weather.rs` via
  `OUTPUT=.cache/mutants/weather-current2 FILE=crates/wildside-core/src/weather.rs mise run mutants-file`.
- Initial evidence: full weather shard reported `868 mutants tested in 2h: 37
  missed, 777 caught, 54 unviable`.
- Fix commit `f900a6d98` adds weather-local tests for storm palette masks, lava
  geyser script/render/runtime boundaries, storm scroll camera word extraction,
  ingame palette loading, murder wall word preservation, hail worker RNG gates,
  stale hail projectile replacement, and Mountain thunder reseeding.
- Follow-up evidence: targeted survivor rerun `/tmp/mutants-weather-target-3`
  reports `205 mutants tested in 10m: 8 missed, 192 caught, 5 unviable`.
  Remaining misses are equivalent/non-observable in current runtime state:
  non-overlapping word OR/XOR merges, startup `*1` vs `/1`, zero-row rise reset,
  and lava hold row target values not stored in runtime state.
- Verification: `cargo test -q -p wildside-core weather -- --nocapture` and
  `mise run verify-slice-fast` passed.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive scenario mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/scenario.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/scenario-current3 FILE=crates/wildside-core/src/scenario.rs mise run mutants-file`
  reports `88 mutants tested in 9m: 72 caught, 16 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing coverage already catches scenario
  construction, serialised input masks, checkpoint timing, and replay stepping
  tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive demo mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/demo.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/demo-current3 FILE=crates/wildside-core/src/demo.rs mise run mutants-file`
  reports `79 mutants tested in 7m: 61 caught, 18 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing coverage already catches demo parsing,
  input replay, and frame stepping tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive constants mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/constants.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/constants-current2 FILE=crates/wildside-core/src/constants.rs mise run mutants-file`
  reports `59 mutants tested in 6m: 50 caught, 9 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing coverage already catches constant
  values and masks used by the current scenarios tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive GFX animation mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/gfx_animation.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/gfx-animation-current2 FILE=crates/wildside-core/src/gfx_animation.rs mise run mutants-file`
  reports `18 mutants tested in 2m: 16 caught, 2 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing animation coverage already catches the
  frame-advance and looping/pause paths tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-07-01 handoff (proactive Dragon mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/dragon.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/dragon-current2 FILE=crates/wildside-core/src/enemies/dragon.rs mise run mutants-file`
  reports `264 mutants tested in 25m: 222 caught, 42 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing ASM-cited Dragon coverage already pins
  live breath, turn/direction, contact, hurt/death, projectile, collision
  dispatch, and cleanup paths tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-30 handoff (proactive Lion mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/lion.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/lion-current2 FILE=crates/wildside-core/src/enemies/lion.rs mise run mutants-file`
  reports `251 mutants tested in 24m: 206 caught, 45 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing ASM-cited Lion coverage already pins
  live walk/roar/leap phases, kid-distance gates, floor/ledge contact, damage,
  death, collision dispatch, and cleanup paths tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-30 handoff (proactive Drips mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/drips.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/drips-current2 FILE=crates/wildside-core/src/enemies/drips.rs mise run mutants-file`
  reports `299 mutants tested in 28m: 234 caught, 65 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing ASM-cited Drips coverage already pins
  placement/type selection, live drop/spawn timing, slope/floor contact, death,
  collision dispatch, and projectile cleanup paths tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-30 handoff (proactive Tar Monster mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/tar_monster.rs` via `mise run mutants-file`.
- Evidence: `OUTPUT=.cache/mutants/tar-monster-current2 FILE=crates/wildside-core/src/enemies/tar_monster.rs mise run mutants-file`
  reports `141 mutants tested in 13m: 107 caught, 34 unviable`, with zero missed
  mutants and zero timeouts.
- Native behaviour is unchanged. Existing ASM-cited Tar Monster coverage already
  pins live movement, floor contact, tar drop, hurt/death, collision dispatch, and
  cleanup paths tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-30 handoff (proactive Ninja mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/ninja.rs` via `mise run mutants-file`.
- Evidence: initial `.cache/mutants/ninja-current` reported `219 mutants tested in
  21m: 36 missed, 150 caught, 33 unviable`; commit `48e183992` hardens the ASM-cited
  coverage and the final `.cache/mutants/ninja-clean2` reports `189 mutants tested in
  18m: 156 caught, 33 unviable`, with zero missed mutants and zero timeouts.
- Native behaviour is intentionally aligned more closely with ASM word comparisons:
  live Ninja player/probe deltas now use high-word position helpers, and tests pin
  guard/threat, airborne knockdown, jump-attack terrain, collision dispatch, and attack-cadence
  period-pair boundaries.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-core ninja -- --nocapture`,
  final `mise run mutants-file`, and `mise run verify-slice-fast` all pass.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-30 handoff (proactive Mini Hopping Skull mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/mini_hopping_skull.rs` via `mise run mutants-file`.
- Evidence: `.cache/mutants/mini-hopping-skull-current` reports
  `146 mutants tested in 12m: 90 caught, 56 unviable`, with zero missed mutants
  and zero timeouts.
- Native behaviour is unchanged. Existing ASM-cited Mini Hopping Skull coverage already
  pins the air sprite, collision dispatch, slope contacts, wait/windup branches,
  scream worker allocation, damage/death paths, and live route enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-30 handoff (proactive UFO mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/ufo.rs` via `mise run mutants-file`.
- Evidence: initial `.cache/mutants/ufo-current` reported `236 mutants tested in
  22m: 59 missed, 139 caught, 38 unviable`; commit `544d661aa` hardens the ASM-cited
  coverage and the final `.cache/mutants/ufo-final` reports `232 mutants tested in
  22m: 194 caught, 38 unviable`, with zero missed mutants and zero timeouts.
- Native behaviour is unchanged except for equivalent refactors around `loc_34FFE`
  and `loc_35056`; new tests pin UFO motion/debris tables, tracking and beam gates,
  terrain collision boundaries, wall bounce, and first-hurt debris initialisation.
- Verification: `cargo fmt --check`, `cargo test -q -p wildside-core ufo -- --nocapture`,
  final `mise run mutants-file`, and `mise run verify-slice-fast` all pass.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-30 handoff (proactive Dragon Walking mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/dragon_walking.rs` via `mise run mutants-file`.
- Evidence: `.cache/mutants/dragon-walking-current` reports
  `136 mutants tested in 13m: 112 caught, 24 unviable`, with zero missed mutants
  and zero timeouts.
- Native behaviour is unchanged. Existing ASM-cited Dragon Walking coverage already pins the
  patrol, wall/ledge turn, breath, death/hurt, stomp/knockback, and slope-foot routes tightly enough
  for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-29 handoff (#361 Hand live render fixed)

- **#361 moved `[~]` -> `[x]`.** Live Hand (0x17) sprites now hold their ROM live
  body/attack/grab frames instead of cycling into the death-tail atlas frames.
- Added trace phases for `HandState::Jumping`/`Grabbing` and a `select_live_hand_frame` renderer
  selector for `Hand.asm loc_327F2` (`stru_32B6A`), `loc_328E0` (`stru_32B56`), and `loc_32906`
  (`stru_32B50`); HP-zero `loc_32AA2` deaths still use the existing death table.
- Evidence: `.cache/debug/issue361/hand_level03_probe.json` direct-starts Highwater Pass 2 near the
  first Hand pair; native enemy-layer frame 6 shows full Hand body sprites instead of the old
  free-run death sliver.
- Verification: focused core/CLI `hand` tests, PNG inspection, `cargo fmt --check`,
  `mise run backlog-lint`, and `mise run verify-slice-fast` pass.
- NEXT: run `project-status`; if no open risks remain, follow `loop.md` proactive verification.

## 2026-06-29 handoff (#358 Cloud lightning render fixed)

- **#358 moved `[~]` -> `[x]`.** Cloud lightning now uses typed ROM bolt art instead of the
  generic red enemy projectile dot.
- Added `ActiveProjectileKind::CloudLightning`, mapped internal `EnemyProjectileKind::CloudLightning`
  through trace, and drew the `Cloud.asm loc_35DF0` child frames (`unk_C7866`/`C786E`/`C7876`) from
  the Cloud sheet in scene.rs.
- Evidence: `.cache/debug/issue358/cloud_level18_probe.json` direct-starts level `0x18` under a
  Cloud; native captures in `.cache/debug/issue358/current/` frames 2/5 show the vertical bolt, and
  oracle captures in `.cache/debug/issue358/oracle/` show the same projectile class.
- Verification: focused core/CLI `cloud_lightning` tests, `cargo fmt --check`,
  `mise run backlog-lint`, and `mise run verify-slice-fast` pass.
- NEXT: `project-status` should select #361 first under visual risks.

## 2026-06-29 handoff (#349 post-death title-card name resolved)

- **#349 moved `[~]` -> `[x]`.** Re-ran the real `death_respawn_level1` route on current `main`;
  the post-death title-card VDP path now shows the level name over the two-tree card.
- Evidence: generated current native frames in `.cache/debug/issue349/current/`; frames 1791, 1855,
  and 1919 visibly show `BLUE / LAKE / WOODS / I`. The new
  `death_respawn_vdp_path_overlays_real_title_name_workers_loc_b8de` regression steps the actual
  script through `NativeRuntime::with_generated_level_layout` and asserts the
  `loc_1AB26`/`loc_1AC70` glyph overlay changes VDP pixels.
- Verification: focused real-route #349 native-player test, existing synthetic glyph-worker test,
  PNG inspection, `backlog-lint`, and `mise run verify-slice-fast`.
- NEXT: `project-status` should select #358 first under visual risks, then #361.

## 2026-06-29 handoff (#324 prize-wobble priority fixed candidate)

- **#324 moved `[~]` -> `[?]`.** BLW1 row-1 prize wobble now clears stale foreground priority
  over the active ASM `sub_1007A` copied-block footprint before drawing the low-priority P/prize
  sprite.
- Evidence: canonical route `.cache/recordings/issue324-blw1-row1-left560-current/scenario.json`
  hits `(57,1)` at frame 574. Before, the isolated `block_overrides` layer had correct P art but
  the composed frame was masked by high-priority terrain; after, native frame-574 crop
  `(150,16)..(165,31)` has `0` differing pixels against oracle.
- Verification: focused `prize_block_wobble` tests, route recapture, crop comparison,
  `cargo fmt --check`, and `mise run verify-slice-fast`.
- NEXT: `project-status` should select #349 first under visual risks, then #358 and #361.

## 2026-06-29 handoff (#254 IntroSequence6 Start-skip fixed candidate)

- **#254 moved `[~]` -> `[?]`.** IntroSequence6 Start-skip no longer uses a probed max-frame
  natural-carry window. Native carries the `$28` VDP lineage only while `loc_1BAB0` is before
  `sub_1B43A/loc_1B446` presents its first hidden-shell frame; Start 2431/2432 now takes the normal
  `Load_TitleScreen` rebuild.
- Evidence: Start=2431/2432 native-vs-oracle frames are exact at 2480, 2500, and 2520 after the fix
  (the old path was about 67k changed pixels at 2500/2520). Start=2420/2430 still preserves the
  intended natural hidden-shell carry, and Start=2400 retains only the existing tiny AE 22/8
  title-detail residue.
- Verification: focused cache-backed IntroSequence6 tests, native/oracle captures, ImageMagick AE
  comparisons, `mise run verify-slice-fast`, and `mise run verify-tas-full` (CRC all 37,759 frames
  match; baseline full-report state/object divergences remain).
- NEXT: `project-status` should select #324 first under visual risks, then #349, #358, and #361.

## 2026-06-29 handoff (#210 IntroSequence5 late Start-skip fixed candidate)

- **#210 moved `[~]` -> `[?]`.** Late IntroSequence5 Start-skip now carries the current
  `loc_1BE3C` particle VDP into the `sub_1C246`/`loc_1C264` title return instead of the stale
  P-block frame.
- Added the missing one-frame `loc_1BCD0` hand-off where the first two static slots are gone but
  particles are not visible yet; the final deferred title flip rebuilds the current particle VDP
  before storing `intro_sequence_5_title_return_vdp_snapshot`.
- Evidence: Start=2350 deterministic probe is exact at frames 2349, 2350, 2351, 2370, 2410, 2411,
  and 2450. The old stale-frame window dropped from AE 2742..5367 to a tiny red-particle residue
  only (max AE 135 in frames 2352..2369).
- Verification: focused cache-backed IntroSequence5 tests, `loc_1BE3C` VDP timing test,
  native/oracle captures, and ImageMagick comparisons.
- NEXT: `project-status` should select #254 first under visual risks, then #324, #349, and #358.

## 2026-06-29 handoff (#368 asm-coverage tidy fixed)

- **#368 moved `[~]` -> `[x]`.** Removed stale current-source `loc_2510C` and `loc_23629`
  citations, using `PrizeDiamondCollected_Init` and `Load_InGame` with real kid.asm line refs.
- Renamed the #339 drone-stop helper/tests away from the stale `loc_23629` suffix; updated
  `asm-coverage --check-required` help/failure text to point at non-test core/data citations.
- Refreshed `audio-triggers-fidelity-audit.md` to describe the current `QueuedSoundCommand` /
  `NativeSoundCommand` / `GemsAudioEngine` command-byte path, and de-duplicated CLAUDE.md's
  asm-coverage command block while documenting `--json`, `--check-required`, and
  `--check-no-new-test-only`.
- Verification: stale-label/deleted-helper `rg` checks, focused asm-coverage/runtime/native-player
  tests, `cargo fmt`, `asm-coverage --check-required`, `backlog-lint`, and
  `mise run verify-slice-fast`.
- NEXT: `project-status` shows no ready or playability rows; take #210 first under visual risks,
  then #254, #324, #349, and #358.

## 2026-06-29 handoff (#336 title-card fade fixed)

- **#336 moved `[~]` -> `[x]`.** BLW1 title-card -> live-gameplay handoff now stays black through
  the ROM `Load_InGame` window and fades in with oracle `sub_73D0`/`sub_14C0` palette progress.
- Preserved the verified camera/Kid-Y ramp; the fix is render-only. The trace now exports
  `level_entry_fade_progress_sub_73d0`, and native-player applies a final `PaletteFadeSub73D0`
  scene pass that blends through the core palette fade helper.
- Evidence: dynamic route `.cache/debug/issue336-cold-boot-live/scenario-card-start.json` is
  CRC/checkpoint-clean for frames 880..1041. Native/oracle captures are exact black at 960/970, and
  fade brightness tracks oracle at 974/980/984.
- Verification: focused core fade trace test, frame palette-fade tests, CLI scene-command test,
  `cargo check -q -p wildside-frame -p wildside-cli --tests`, route `verify-tas`, captures, and
  `mise run verify-slice-fast`.
- NEXT: `project-status` shows no ready-to-implement rows; take #368 first under
  playability/infra/docs. The top visual risks remain #210, #254, #324, #349, then #358.

## 2026-06-29 handoff (#367 verify-slice-fast gate wiring fixed)

- **#367 moved `[~]` -> `[x]`.** `verify-slice-fast` now depends on
  `audit-debug-fields`, `asm-coverage --check-required`, and
  `asm-coverage --check-no-new-test-only`.
- The required-label gate reports 9/9 labels covered, the test-only ratchet reports 242 current /
  242 baseline / 0 burned down, and the debug-field audit reports 91 emitted / 91 registered keys
  with 0 pending `first_added_commit` hashes.
- Hardened `audit-debug-fields` to scan `crates/**/*.rs.txt`, accept split-line `.insert(...)`
  debug-field calls, warn on registered-but-unemitted keys, register the newly visible oracle
  helper template fields, and backfill all pending registry hashes.
- Added a real repository integration test for `--check-required`: it parses the local Kid
  disassembly, scans current native citations, and proves all required ASM labels resolve as
  covered.
- Verification: focused `audit-debug-fields`, `asm_coverage`, `asm-coverage-check-required`,
  `asm-coverage-check-no-new-test-only`, `cargo fmt --check`, registry hash audit, and
  `mise run verify-slice-fast`. `verify-tas-full` was not required because this is tooling/gate-only.
- NEXT: `project-status` shows no ready-to-implement rows; take #336 first under
  playability/infra/intro risks, with #368 next there and #210 still the top visual risk.

## 2026-06-29 handoff (#366 asm-coverage test-only ratchet fixed)

- **#366 moved `[~]` -> `[x]`.** asm-coverage now has a checked-in
  `docs/source-port/asm-coverage-test-only-baseline.yaml` and
  `--check-no-new-test-only` fails on any new label that is cited only from test contexts.
- The baseline records the current 242 `test_only_debt` labels and may shrink as labels become
  reachable or disappear; additions require explicit review.
- The five orphan subroutines are triaged inline: `sub_96E8` needs live Juggernaut wall-scan
  wiring; `sub_1FA2` needs a #205-style global post-object collision scenario; `sub_144DA` and
  `sub_14582` are rubber-block art checks still sitting in core test-only references;
  `sub_3F366` is live-backed but missing a production citation for the exact label.
- Added a methodology pitfall noting that `#[cfg(test)] pub(crate)` ASM reference ports tested
  only through their own output are fidelity debt, not shipped-behaviour proof.
- Verification: focused `asm_coverage`, `juggernaut_wall_scan_sub_96e8`, `sub_1fa2`, and
  `mini_hopping_skull_sub_3f366...` tests, `cargo fmt --check`, `asm-coverage
  --check-no-new-test-only`, `asm-coverage --update`, and `mise run verify-slice-fast`.
  `verify-tas-full` was not required because this is tooling/docs-only.
- NEXT: `project-status` shows no ready-to-implement rows; take #367 first under
  playability/infra risks, with #210 still the top visual risk.

## 2026-06-29 handoff (#365 asm-coverage reachability split fixed)

- **#365 moved `[~]` -> `[x]`.** asm-coverage now reports reachable / test-only-debt / uncovered
  labels with the GEMS interface split as emulated.
- Reachable = non-test core/data citation; test-only debt = `cfg(test)`-only; uncovered = no
  production/test citation; `sub_E133C..loc_E15E8` is `emulated_gems`.
- Removed the vestigial `oracle_backed`/`OracleBacked` output; `parity_evidence` remains evidence
  text only.
- Current headline/JSON/YAML: 3835 reachable / 242 test-only debt / 40 uncovered (33 emulated
  GEMS, 7 ordinary). The rough plan's 3788/289 estimate included 47 labels that now have both
  production and test citations; decision 2 classifies those as reachable.
- `--uncovered` tags the 14 current GEMS `sub_` labels as `[emulated GEMS]`.
- Verification: focused `asm_coverage` and `rust_source_context` tests, `cargo fmt --check`,
  `asm-coverage --update`, JSON/YAML absence checks for `oracle_backed`, and
  `mise run verify-slice-fast`. `verify-tas-full` was not required because this is tooling-only.
- NEXT: `project-status` shows no ready-to-implement rows; take #366 first under playability/infra
  risks, with #210 still the top visual risk.

## 2026-06-29 handoff (#364 asm-coverage cfg detector fixed)

- **#364 moved `[~]` -> `[x]`.** asm-coverage now has a line-aware Rust source-context detector
  for inline `#[cfg(test)]` citations.
- Added shared `rust_source_context` tokenisation for comments/literals/raw strings/chars,
  lifetime preservation, cfg `any`/`all`/`not`, and `cfg_attr` false-positive avoidance.
- `asm_coverage` keeps `cited_in` paths unchanged but builds generated YAML breakdowns from
  per-line Production/Test citation sites; the same source file can now appear in both buckets
  when it has production and test citations.
- Regressions pin the `blocks.rs` trap at 267 test-only, 1692/4675 production, and 5698 test, plus
  same-file prod/test bucket split.
- Mutation testing found and fixed a real lifetime-vs-char bug. Targeted run: 51 mutants, 29
  caught, 2 unviable, 6 non-progress timeouts, 14 token-index equivalent/valid-input residuals.
- Verification: `rust_source_context` tests, `asm_coverage` tests, `asm-coverage --update`,
  `cargo fmt --check`, and `mise run verify-slice-fast`. `verify-tas-full` not required
  (tooling-only).
- NEXT: `project-status` shows no ready-to-implement rows; take #365 first under
  playability/infra risks, with #210 still the top visual risk.

## 2026-06-29 handoff (#363 fatal death ouch fixed)

- **#363 moved `[~]` -> `[x]`.** Clean fatal TheKid deaths no longer replay an ouch after the
  death voice in native playback.
- Root cause was playback-side: the core trace correctly queued `sfx_Voice_die` from `Death`
  (kid.asm:14714), but native-player's legacy HP-drop heuristic inferred generic `sfx_Voice_ouch`
  from HP 1 -> 0 because it only suppressed itself for explicit ouch takes. The resulting audio
  transcript was `[0x60, 0x61]`, so the later ouch could mask die.
- Fix: `is_damage_resolution_voice_sound_id` treats either a loc_B73C ouch take or `VoiceDie` as
  authoritative damage audio for both the GEMS command path and no-GEMS fallback tones.
- Regressions:
  `play_native_does_not_infer_ouch_over_fatal_death_voice_issue_363` and
  `fallback_sfx_does_not_add_ouch_over_fatal_death_voice_issue_363`.
- Verification: focused `issue_363`, `native_player::tests::play_native`, `fallback_sfx`, core
  `voice_die`, `cargo fmt --check`, `mise run verify-slice-fast`, and `mise run verify-tas-full`
  (exit 0; full-run CRC exact, baseline full-report divergences still start at
  `transition_15_to_00_post`).
- NEXT: `project-status` shows no ready-to-implement rows; take #364 first under
  playability/infra risks, with #210 still the top visual risk.

## 2026-06-29 handoff (#362 RockTank early drive SFX fixed)

- **#362 moved `[~]` -> `[x]`.** RockTank/TankShooting `$1F` no longer starts from the generic
  off-screen enemy-activation window before the tank body is sprite-visible.
- The fix follows `Make_SpriteFromGfxObject` loc_11EE, `Tank.asm` loc_3B40A,
  `Tank_Shooting.asm` loc_3B690, and `sub_3C352` loc_3C39C: native computes the Tank body `$19(a3)`
  clipped flag from the ROM sprite bounds and stores the prior clipped state as the `$6A(a5)`
  visibility edge before entering the pooled audible drive refcount.
- Regression `tank_live_step_waits_for_make_sprite_visibility_before_drive_sound_issue_362` covers
  the off-screen silent frame and the later clipped-to-visible start edge; #339 stop/count
  regressions remain green.
- Verification: Tank module tests, RockTank stop/count tests, `mise run verify-slice-fast`, and
  `mise run verify-tas-full` (exit 0; full-run CRC exact, baseline full-report divergences still
  start at `transition_15_to_00_post`).
- NEXT: `project-status` shows no ready-to-implement rows; take #363 first under playability risks,
  with #210 still the top visual risk.

## 2026-06-29 handoff (#359 Sphere settled collision fixed)

- **#359 moved `[~]` -> `[x]`.** Cyclone-form sky level Sphere 0x06 now keeps the opening spawn
  visible instead of over-ageing/drifting left during the settled recording window.
- The fix follows `Enemy06_Sphere_Init` / `stru_36174`, `loc_35F70`, and `GfxObjects_Collision
  loc_171C`: live Sphere uses the active ±7 collision rows, consumes the previous pending terrain
  collision before movement, then probes the next row for the following tick. The right-edge scan
  preserves the ROM's `loc_173E`/`loc_19A8` one-pixel active edge.
- Level 0x1B recording settle now uses a 103-frame live window so the first recorded Sphere matches
  the oracle short-age tuple: x/y `0x00100000/0x025E5800`, velocities `-0x10000/0x4800`, pending
  collision 0, HP 0.
- Verification: focused Sphere/combat/block tests, cache-backed level 0x1B regression,
  `mise run verify-slice-fast`, and `mise run verify-tas-full` (exit 0; full-run CRC exact, baseline
  full-report divergences still start at `transition_15_to_00_post`).
- NEXT: `project-status` shows no ready-to-implement rows; take #362 first under playability risks,
  with #210 still the top visual risk.

## 2026-06-29 handoff (#360 record-inputs route targets fixed candidate)

- **#360 moved `[~]` -> `[?]`.** `play-native --record` now persists named route-target metadata
  (`label`, level, point, cell, cell-hitbox, enemy type) in native recording bundles and README.
- Bundles now include `tas_frontier_command`, pointing at their saved `scenario.json` with
  `verify-tas --bisect --frontier-kind ...`, so a natural live route can immediately show whether
  it stays native/oracle comparable or where it first drifts.
- Regression coverage: `play_native_record_route_target_*`,
  `save_recording_writes_route_target_summary`,
  `save_native_recording_bundle_persists_scripted_route_metadata_and_sfx_events`, and README/frontier
  command tests.
- Verification: focused CLI recording/target tests, `cargo fmt --check`, and
  `mise run verify-slice-fast` green. `verify-tas-full` was not required; this is recording
  metadata/CLI infrastructure, not gameplay-state logic.
- NEXT: run `mise run project-status` and take the top open item. Human validation for #360 is to
  record a real USM3 0x06 slope route, then run the generated `verify-recording` and
  `tas_frontier_command`.

## 2026-06-29 handoff (#339 cold Load_InGame stale Tank SFX fixed candidate)

- **#339 moved `[~]` -> `[?]`.** Cold `Load_InGame` audio now treats same-frame Tank SFX
  deltas as pre-reset state: `StopMusicThenStart` suppresses replayed `$1F` TankDriving and
  `$3C` TankMouthOpen when `active_before=None` on an `InGame`/demo level-load frame.
- The fix is scoped to ASM `Load_InGame loc_23629` / `j_StopMusic`; ordinary active-cue BGM
  replacements still preserve same-frame gameplay SFX, so this does not fold in #362's early
  TankDriving START-side bug.
- Regression `play_native_cold_load_ingame_stopmusic_suppresses_stale_tank_sfx_issue339` covers
  the stale drone + mouth squeak transcript, and
  `scripted_recording_audio_events_follow_trace_mode_and_sfx_transitions` guards the legitimate
  options-to-gameplay SFX case.
- Verification: `cargo fmt --check`, focused CLI audio tests, and `mise run verify-slice-fast`
  green. Real audio-device listen still required for clean/death/skip USM 0x04 loads.
- NEXT: `project-status` shows no ready rows; take #360 first under playability risks, with #210
  still the top visual risk. #339 now needs human audio validation, not another headless
  stop-sequence patch.

## 2026-06-29 handoff (#335 Hills title-card murder-wall fixed candidate)

- **#335 moved `[~]` -> `[?]`.** Hills 0x08 title-card wait now hibernates gameplay input and
  `Mode_Level` while `loc_1AE2E` owns the murder-wall title-card shell; Start/`loc_6E2` resumes
  live gameplay without exposing a pre-aged murder-wall death.
- Added synthetic and cache-backed regressions for the exact "leave the card up" failure:
  `loc_1ae2e_hills_title_card_wait_does_not_age_murder_wall`, plus strengthened debug and
  natural 0x07->0x08 handoff tests.
- Gate hygiene: restored generated layout sidecars for `00`, `01`, `02`, `1B`, and `15`; fixed a
  pre-existing red tar-burst unit by letting `Tar_Monster.asm loc_3263C` own the single post-move
  `$1000` gravity add.
- Verification: focused `loc_1AE2E` and tar tests, `mise run verify-slice-fast`, and
  `mise run verify-tas-full` (exit 0; full-run CRC exact, baseline full-report divergences still
  start at `transition_15_to_00_post`).
- NEXT: run `mise run project-status` and take the top ready/open item; #335 now needs human
  validation via a live/debug skip into Hills 0x08, leaving the title card up before Start.

## 2026-06-26 handoff (playtest triage #2 — backlog validation)

- Human playtested pushed HEAD `7c613d63f` and reported 18 observations. A read-only fan-out
  pass (36 agents) cross-checked each against all four backlog shards. Full evidence in the
  archive block "## 2026-06-26 playtest triage". Almost everything was already logged.
- **Reopened #349 `[?]`→`[ ]`:** the post-death respawn card still shows no level-NAME text and
  the 2026-06-21 fix `b55244ad1` is in the build, so the candidate failed validation.
- **Opened #361** (Hand 0x17 render free-run, deferred #243/#333 follow-up), **#362** (RockTank
  `$1F` drive SFX starts too early — START side, distinct from #339's STOP side), **#363** (Kid
  says "ouch" not "die" on death — human-confirmed bug; capture exact death-cause to pin it).
- **Sharper repros on the route-blocked cluster (stay `[b]`):** #249/#270 widened to all Under
  Skull Mountain (USM 0x04 + USM2 0x05, not only USM3 0x06); #353 sharpened to USM3 0x06; #345
  Lion vanish reconfirmed on the #357-included build (caveat: rule out a #357 resume side-effect).
- **Positives:** green-projectile tank now fine (#252/#323 — #323 ready for a human `[?]`→`[x]`);
  #357 death dimension confirmed (Lion takes the stomp). #324/#335/#336/#358/#280 unchanged.
- All route-blocked items remain gated on the #360 record-inputs capture flow.

## 2026-06-26 handoff (#357 Lion stomp fixed candidate)

- **#357 moved `[ ]` -> `[?]`.** Base-Kid Lion stomps now route through
  `Lion.asm loc_3E318` / `loc_3E32C` in `process_stomps`, so a full-HP Lion
  loses one HP, flashes, and resumes instead of taking generic damage/death.
- Live Lion activation now consumes `loc_3DF92`, `loc_3DFD8`, and `loc_3E030`
  helpers for initial HP, facing, speed, and first dispatch. The runtime-fields
  helper remains marked until its random/projectile-count state is represented.
- Dead-code ratchet lowered **1339 -> 1333**.
- Verification: focused `lion_init` and
  `lion_stomp_decrements_one_hp_and_resumes_loc_3e318`, `RUSTFLAGS='-D dead_code'
  cargo check --workspace --lib`, `mise run dead-code-ratchet`,
  `mise run verify-slice-fast`, and `mise run verify-tas-full` (exit 0; CRC all
  37,759 frames match, baseline full-report divergences still start at
  `transition_15_to_00_post`).
- NEXT: human/playtest validation for #357: stomp a full-health Lion as plain
  Kid; it should need about six stomps and must not vanish/die immediately.

## 2026-06-25 handoff (human playtest triage)

- **17-observation voice playtest triaged** against the lanes + ASM + code (fan-out research
  pass). Build = last pushed commit `253e98961` (origin/main), so it CONTAINED the
  2026-06-23 fixed candidates — the ones still broken therefore FAILED validation.
- **4 fixed-candidates reopened `[?]`→`[ ]`:** #210/#254 (intro skip still garbles — next step is
  an engine-level VDP carry on ANY skip frame, no frame constants), #324 (prize-block wrong
  sprite), #335 (Hills 0x08 skip-entry instant death — freeze the murder wall during the entry
  title card), #339 (USM load drone — extend the `$1F` stop to death/respawn/skip teardown).
- **#336 reopened `[-]`→`[ ]` (P3).** Human cross-emulator evidence disputes the sky-float
  NOT-A-BUG; the prior closure proved camera/Kid-Y WRAM only, never the palette fade-in timing.
- **3 new rows:** #357 (Lion one-stomp death as base Kid — likely a live HP-decrement wiring gap),
  #358 (Cloud lightning renders as a red dot not the bolt sprite — 3-file render fix), #359
  (Cyclone-level Sphere missing at spawn).
- **Route-blocked reconfirmed LIVE on HEAD (stay `[b]`):** #271 + #259 (stomp-hurts + floor-sink at
  the L2 teleporter tank and L3 first tank), #249/#270 (slope-into-wall float, now confirmed on
  BOTH left/right slopes — level named Under Skull Mountain 3, 0x06), #345 (Lion vanish).
- **#360 opened (`[ ]` P1 infra):** record-inputs capture flow so a NATURAL playtest becomes an
  oracle-comparable route — the unblock-key for the route-blocked tank/slope/Lion/Hills items.
  Requested by the human as an issue, not to be built inline.
- **Confirmed correct:** Storm-Mtn "head guy" = EmoRock hover-track (no projectile). Positives:
  continues load, level 2 clean, Cyclone spin (#355), Tar/blob render (#243), Hand grab (#185).
- NEXT: `mise run project-status` now lists the 9 reopened/new open rows priority-ordered; #210
  and #335 (P0) lead. Several route-blocked P0/P1s are unblockable only by a human recording at the
  named tanks/slopes — surface that when picking work.

## 2026-06-25 handoff (proactive Tank mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/tank.rs` via `mise run mutants-file`.
- Evidence: `.cache/mutants/tank` reports
  `28 mutants tested in 12m: 21 caught, 7 unviable`, with zero missed mutants.
- Native behaviour is unchanged. Existing ASM-cited Tank and Tank_Shooting
  coverage already pins the visible init/art-slot, drive refcount, mouth SFX,
  HP/death/hurt routing, turret resume, bullet bounds, shot cadence, and child
  snapshot/recoil branches tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Hand mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/hand.rs` via `mise run mutants-file`.
- Evidence: initial run reported `200 mutants tested in 81m: 55 missed,
  124 caught, 21 unviable`. Survivors clustered around ASM `loc_328AA`
  random-byte XOR, `off_32A12` airborne `$0C`/`$18` dispatch arms,
  `loc_3292E` grab timer direction packing, `loc_3284C`/`loc_32886`
  strict detection boundaries, `loc_32840` reversal, and live walking /
  jumping / landing / grab-window step fields.
- Native behaviour is unchanged. Added exact ASM-cited helper and live-step
  regressions; refactored disjoint grab timer bit packing and wall clamp
  comparisons to avoid equivalent operator mutants.
- Verification: focused `hand_` tests pass; final rerun
  `.cache/mutants/hand-final2` reports
  `184 mutants tested in 72m: 163 caught, 21 unviable`, with zero missed
  mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Goat mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/goat.rs` via `mise run mutants-file`.
- Evidence: `.cache/mutants/goat` reports
  `23 mutants tested in 9m: 15 caught, 8 unviable`, with zero missed mutants.
- Native behaviour is unchanged. Existing ASM-cited Goat coverage already pins
  the init, collision dispatch, Berzerker side-contact, hurt/death,
  player-tracking, and contact-box branches tightly enough for this shard.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Orca mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/orca.rs` via `mise run mutants-file`.
- Evidence: initial run reported `87 mutants tested in 34m: 18 missed,
  52 caught, 17 unviable`. Survivors clustered around ASM `loc_3D348`
  lunge Y velocity, live `loc_3D35A` lunge displacement/FP terrain probes,
  `loc_3D3DE` slope relaunch, `loc_3D40C` wall pinning, `loc_3D47A`
  ceiling bounce gating, `loc_3D420` floor snap, and `loc_3D496` fatal-fall
  movement/collision classification.
- Native behaviour is unchanged. Added exact ASM-cited regressions and
  extracted the fatal-fall floor/slope collision-word classification into a
  tested helper. Removed redundant floor-landing slope exclusions after the
  earlier slope branch has already returned.
- Verification: focused `orca` tests pass; final rerun
  `.cache/mutants/orca-final` reports
  `86 mutants tested in 34m: 69 caught, 17 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Crab mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/crab.rs` via `mise run mutants-file`.
- Evidence: initial run reported `87 mutants tested in 34m: 19 missed,
  52 caught, 16 unviable`. All survivors were in `step_crab_freeze`, covering
  the live `loc_3B0BE`/`loc_3B142` freeze callback, X delta extraction,
  frozen-facing preservation, distance thresholds, and close-range Kid velocity
  gates.
- Fixed one ASM mismatch while adding the missing regressions: when `$6E(a5)`
  is set, Crab.asm jumps straight to `loc_3B142`, skipping the unfrozen facing
  and freeze-sprite branch. The frozen close-range branch clears `$6E(a5)` when
  Kid moves toward the crab, not away.
- Verification: focused `crab` tests pass; final rerun
  `.cache/mutants/crab-final` reports
  `85 mutants tested in 33m: 69 caught, 16 unviable`, with zero missed mutants.
  `mise run verify-slice-fast` passes. `mise run verify-tas-full` reports the
  same pre-existing full-recording divergences on current HEAD and `origin/main`
  (`CRC: all 37759 frames match`; first diff `transition_15_to_00_post`), so
  this change does not move the full-TAS frontier.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Diamond mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/diamond.rs` via `mise run mutants-file`.
- Evidence: initial run reported `67 mutants tested in 27m: 15 missed,
  42 caught, 10 unviable`. Survivors clustered around high-HP `loc_3ADA4`
  floor dispatch, live high-HP terrain collision classification, and
  Berzerker launch step wrappers `loc_3AEB2`/`loc_3AF50`.
- Native behaviour is unchanged. Added exact terrain/launch regressions and
  split `FloorLoop` from the unknown loop route so the explicit ASM
  `colid_floor` branch remains testable while both routes stay runtime no-ops.
- Verification: focused `diamond` tests pass; final rerun
  `.cache/mutants/diamond-final` reports
  `67 mutants tested in 27m: 57 caught, 10 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Fireball mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/fireball.rs` via `mise run mutants-file`.
- Evidence: initial run reported `59 mutants tested in 23m: 5 missed,
  42 caught, 12 unviable`. Survivors covered `loc_32F3E` signed branch
  boundaries and `unk_32EDC` ratio-table range edges used by
  `loc_32F78/loc_32FAC/loc_32FC8/loc_32FE8`.
- Native behaviour is unchanged. Replaced an equivalent post-zero signed-X
  comparison with `is_negative()` and added exact ASM-table/zero-Y regressions.
- Verification: focused `fireball` tests pass; final rerun
  `.cache/mutants/fireball-after` reports
  `56 mutants tested in 22m: 44 caught, 12 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Spinning Twins mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/spinning_twins.rs` via `mise run mutants-file`.
- Evidence: initial run reported `143 mutants tested in 56m: 22 missed,
  98 caught, 21 unviable, 2 timeouts`. Survivors clustered around
  `word_3B9A0` orbit offsets, `loc_3BA00` falling-visual signed camera/reverse
  drift, `loc_3B93C` split materialisation, `loc_3B904` live contact edges,
  `loc_3BB2C/loc_3BB42/loc_3BB52` live walking, `loc_3BB6E` floor snap, and
  `loc_3BA5C` falling-visual deletion.
- Native behaviour is unchanged. Added exact edge regressions and changed
  `materialize_spinning_twins_drop_splits` from index inserts to an
  order-preserving rebuild, removing the loop-counter timeout mutation mode.
- Verification: focused `spinning_twins` tests pass; final rerun
  `.cache/mutants/spinning-twins-final` reports
  `128 mutants tested in 49m: 107 caught, 21 unviable`, with zero missed
  mutants and zero timeouts.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Robot mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/robot.rs` via `mise run mutants-file`.
- Evidence: initial run reported `122 mutants tested in 49m: 38 missed,
  73 caught, 11 unviable`. Survivors clustered around `loc_3A762` impact-frame
  arithmetic, `loc_3A8AE` child-zero/default clear routing, `loc_3AC0C` shot
  payload/default fields, and live walking/aiming/charging terrain/cadence
  edges in `step_robot_with_rng`.
- Native behaviour is unchanged. Added focused Robot edge regressions, removed
  equivalent/default mutation targets, and kept wall clamping explicit via
  bounds rather than nested comparisons.
- Verification: focused `robot` tests pass; final rerun
  `.cache/mutants/robot-after` reports
  `104 mutants tested in 42m: 93 caught, 11 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Sphere mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/sphere.rs` via `mise run mutants-file`.
- Evidence: `.cache/mutants/sphere` reported
  `12 mutants tested in 5m: 4 caught, 8 unviable`, with zero missed mutants.
- Native behaviour is unchanged; no source changes were needed.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Armadillo mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/armadillo.rs` via `mise run mutants-file`.
- Evidence: initial run reported `47 mutants tested in 18m: 2 missed,
  33 caught, 12 unviable`. Survivors were both equivalent mutation targets:
  `loc_3C810` set-bit `|` -> `^` after bit 6 was already cleared, and
  `loc_3C824` deleting the explicit zero-collision arm that returned the same
  shared-walker route as the default unmatched-positive arm.
- Native behaviour is unchanged. Kept the `loc_3C810` mirror invariant explicit,
  added an unrelated-bit preservation assertion, expressed the known-clear bit
  set as wrapping addition, and removed the redundant zero match arm while
  leaving zero covered by the dispatch test.
- Verification: focused `armadillo` tests pass; final rerun
  `.cache/mutants/armadillo-final` reports
  `46 mutants tested in 18m: 34 caught, 12 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Emo Rock mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/emo_rock.rs` via `mise run mutants-file`.
- Evidence: initial run reported `73 mutants tested in 29m: 4 missed,
  58 caught, 11 unviable`. Survivors clustered in live-step edge coverage:
  settled zero-wait disappear (`loc_362B6` -> `loc_36320`), partial hover
  Y tracking (`loc_3621A`), integer-pixel floor probe (`loc_36274`), and
  floor snap back to FP16.16 (`loc_362A4`).
- Native behaviour is unchanged. Added focused regressions for the surviving
  wait, hover, floor-probe, and snap edges, using a one-cell floor fixture so
  the floor probe's `x_pos >> 16` conversion is observable.
- Verification: focused `emo_rock` tests pass; final rerun
  `.cache/mutants/emo-rock-final` reports
  `73 mutants tested in 29m: 62 caught, 11 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Scorpion mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/scorpion.rs` via `mise run mutants-file`.
- Evidence: initial run reported `62 mutants tested in 24m: 10 missed,
  41 caught, 11 unviable`. Survivors clustered in exact Scorpion ASM edge
  coverage: `stru_3DB8E` sting duration, `loc_3DDC8` close-range lock routing,
  `loc_3DD7E` equal-X facing preservation, `loc_3DDA8` inclusive `+/-$AA`
  tracking window, and `loc_3DDEE` vertical-miss lock-without-sting.
- Native behaviour is unchanged. Added focused regressions for the surviving
  tracking, range, duration, and lock edges.
- Verification: focused `scorpion` tests pass; final rerun
  `.cache/mutants/scorpion-after` reports
  `62 mutants tested in 25m: 51 caught, 11 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Big Hopping Skull mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/big_hopping_skull.rs` via
  `mise run mutants-file`.
- Evidence: initial run reported `91 mutants tested in 35m: 26 missed,
  50 caught, 15 unviable`. Survivors clustered in exact ASM edge coverage:
  floor-contact wait frame arithmetic (`loc_36580`/`loc_366B6`), high-word
  velocity writes (`loc_365D8`/`loc_36664`), terrain collision dispatch
  (`loc_3674E`), live position advance/wall backup (`loc_36540`/`loc_366EC`),
  floor-contact gating, slope deflections (`loc_36706`/`loc_36720`), and
  ceiling bounce gating (`loc_366E6`).
- Native behaviour is unchanged except for using the existing high-word velocity
  writer and sign predicates to remove equivalent mutation targets. Added
  focused regressions with tiny synthetic grids for the surviving wall, floor,
  slope, ceiling, dispatcher, and timer edges.
- Verification: focused `big_hopping_skull` tests pass; final rerun
  `.cache/mutants/big-hopping-skull-after` reports
  `72 mutants tested in 28m: 57 caught, 15 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-24 handoff (proactive Archer mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/archer.rs` via `mise run mutants-file`.
- Evidence: initial run reported `149 mutants tested in 60m: 31 missed,
  108 caught, 10 unviable`. Survivors clustered in exact Archer ASM edge
  coverage: `byte_3437A` signed velocity bytes, `loc_3414A` Kid bounce sign,
  `loc_340C0`/`loc_34458` angle boundaries, `loc_34308`/`loc_34320` half-ring
  re-aim boundary, vertical shooting-table spawn offsets, defensive
  `arrow_velocity_from_angle` clamp, and `loc_34248` live countdown decrement.
- Native behaviour is unchanged except for removing dead defensive clamps that
  sit behind already-bounded ASM-derived angle/range checks. Added focused
  regressions for the surviving table values, angle quadrants, half-ring
  re-aim boundary, spawn offsets, velocity clamp, death-bounce sign, and live
  countdown path.
- Verification: focused `archer_` tests pass; final rerun
  `.cache/mutants/archer-final` reports
  `133 mutants tested in 54m: 123 caught, 10 unviable`, with zero missed
  mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-23 handoff (proactive Cloud mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/cloud.rs` via `mise run mutants-file`.
- Evidence: initial run reported `122 mutants tested in 48m: 23 missed,
  84 caught, 15 unviable`. Survivors clustered in ASM branch/offset coverage:
  control dispatch offset `$0C` (`loc_35BB2`), diagonal attack strict bands
  (`loc_35CD8`/`loc_35D40`/`loc_35D62`), zero-velocity wall probe direction
  (`loc_35BDE`), projectile capacity (`loc_35DB8`), and lightning spawn fixed
  point/4px/8px offsets (`loc_35DF0`).
- Native behaviour is unchanged except for removing dead latched-path
  Cloud/Kid position extraction: ASM `loc_35DAE`/`loc_35DB8` only reuses
  latched `d1`, `$18(a3)`, and the flipped direction byte. Added focused
  regressions for the surviving branch edges and projectile construction.
- Verification: focused `cloud_` tests pass; rerun `.cache/mutants/cloud-local-tests`
  reports `120 mutants tested in 47m: 105 caught, 15 unviable`, with zero
  missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-23 handoff (proactive Fire Demon mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/sphere.rs` first; it was already clean
  (`12 mutants tested in 5m: 4 caught, 8 unviable`, zero missed).
- Then mutation-tested `crates/wildside-core/src/enemies/fire_demon.rs`.
  Evidence: initial run reported `40 mutants tested in 14m: 2 missed,
  21 caught, 17 unviable`; survivors were delete-arm mutants for raw
  enemy_level `0` in `fire_demon_palette_pulse_seed_loc_3a4b4` and
  collision_type `0` in `fire_demon_collision_dispatch_loc_3a586`.
- ASM `Fire_Demon.asm loc_3A48E`/`loc_3A4B8` keeps enemy_level `0` on
  Data_Index+$634 with no `$C` add. ASM `loc_3A586` has an explicit
  `beq.s loc_3A5D4` no-collision branch before the side/hurt comparisons;
  native now records that source separately from unknown-positive clear/resume
  so the branch edge is covered without changing live side effects.
- Verification: focused `fire_demon_` tests pass; rerun
  `.cache/mutants/fire-demon-after` reports
  `40 mutants tested in 14m: 23 caught, 17 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-23 handoff (proactive Driller mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/driller.rs` via `mise run mutants-file`.
- Evidence: initial run reported `24 mutants tested in 10m: 2 missed, 17 caught,
  5 unviable`; both survivors changed the left-facing
  `cx - DRILLER_DRILL_CHILD_X_OFFSET` probe in
  `driller_drilled_cells_loc_3be78`.
- ASM `Driller.asm loc_3BE78`/`loc_3BE8E` selects the left linked drill child
  `$3A(a5)`, whose init path places x at parent-$18, and seeds `d0=1`.
  Existing coverage only pinned the right child. Added
  `loc_3be78_driller_probes_left_child_x_not_parent_collision_width`.
- Verification: focused `loc_3be78_driller_` tests pass; rerun
  `.cache/mutants/driller-after` reports
  `24 mutants tested in 10m: 19 caught, 5 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-23 handoff (proactive Tornado mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/tornado.rs` via `mise run mutants-file`.
- Evidence: initial run reported `70 mutants tested in 28m: 12 missed, 50 caught,
  8 unviable`. Survivors were all Tornado branch-boundary coverage gaps:
  negative-vs-zero `loc_35056` dispatch, `$28` return-only table slot, equal-X
  Cyclone side derivation, wall-bounce side effects, and floor/ceiling velocity
  gates in live `step_tornado`.
- Native now exposes a test-only-observable `TornadoCollisionRouteLoc35056`
  route in the dispatch result so zero-return and negative-clear stay distinct
  without changing live behaviour. Focused regressions pin the missed terrain and
  Cyclone boundary cases against `Tornado.asm loc_35948`/`loc_35986` and
  `UFO.asm loc_35056`/`loc_35120`/`loc_35146`.
- Verification: `cargo test -q -p wildside-core tornado_ -- --nocapture` passes
  38 tests; rerun `.cache/mutants/tornado-after` reports
  `70 mutants tested in 28m: 62 caught, 8 unviable`, with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-23 handoff (proactive Goat mutation shard)

- **Fallback verification completed.** With no ready/open rows, mutation-tested
  `crates/wildside-core/src/enemies/goat.rs` via `mise run mutants-file`.
- Evidence: initial run found one survivor: deleting the explicit
  `collision_type == 0` arm in `goat_collision_dispatch_loc_3cb14`. ASM
  `Goat.asm loc_3CB14` reaches the shared `loc_3CBF6` tail by distinct zero
  (`beq`) and unknown-positive (`bne`) branches, so native now exposes separate
  no-op route variants for those branches while `combat.rs` keeps both as no-op.
- Verification: focused Goat dispatch/live-collision tests pass; rerun
  `.cache/mutants/goat-after` reports `23 mutants tested in 9m: 15 caught, 8 unviable`,
  with zero missed mutants.
- NEXT: continue with `mise run project-status`; if no ready/open item appears,
  keep using `loop.md` proactive verification rather than selecting `[?]`/`[b]`
  human-blocked rows.

## 2026-06-23 handoff (#356 Skydragon dropout route-needed)

- **#356 moved `[~]` -> `[b]`.** Current Skydragon Castle 0x1B evidence does
  not reproduce a native-only sky-castle dropout. Forced-camera probes were
  rejected because the oracle helper showed stale/striped VDP state and later
  left gameplay, so they are not a valid basis for render changes.
- Evidence: natural `any-percent-published` captures over frames `875..1825`
  every 25 frames keep castle chunks, clouds, tornado scenery, and background
  present in both native and oracle. The settled no-camera Level 0x1B idle probe
  also keeps the background stable through frames `0..840`.
- Verification: `inspect-level --level 1B`, native/oracle `capture-scenario-frames`
  and `capture-oracle-scenario-frames`, contact-sheet inspection, and ImageMagick
  AE checks. No code change; next useful work needs an exact live route/frame.
- NEXT: continue with `mise run project-status`. If no ready/open item remains,
  follow the proactive-verification fallback in `loop.md`; do not patch #356
  without a reproducible current dropout.

## 2026-06-23 handoff (#351 Drips child palette variant)

- **#351 moved `[~]` -> `[?]`.** Drips child projectiles now carry the parent
  enemy-slot palette variant from `step_drips` through the falling child,
  `loc_335D8` surface crawler, and `loc_3392C` finish shell.
- Evidence: Level `$12`/resource `$49` decodes Drip slot `0x0016` (palette
  variant 0) with Drips at x≈2264/2104/2408. Parent enemies already used the
  slot variant; the missing render path was the Drips child projection, which
  previously always used `drips_visuals.palette`.
- Verification: focused Drips producer/crawler/finish propagation tests,
  focused native-player Drips drop render/palette test, `drips_` core/CLI
  slices, `enemy_art_palette`, and `inspect-level --level 12`. `capture-frames`
  cannot jump directly to x≈2264, so final visual confirmation remains human.
- NEXT: continue with `mise run project-status`; the top open row should now be
  #356 unless a higher-priority ready item appears.

## 2026-06-23 handoff (#347 Shifting block rise render)

- **#347 moved `[~]` -> `[?]`.** Native-player now renders typed Shifting
  block-bounce workers with ASM `sub_113C0/loc_113F4` `LnkTo_unk_E0EBE`
  sprite art at VRAM `$276`, instead of dropping the no-block `$6F00`
  reserved-cell worker and snapping straight to `$E710`.
- Evidence: the renderer branch is keyed on `BlockBounceTraceKind::Shifting`
  and applies the traced y offset; prize and HiddenReveal render paths remain
  covered by neighbouring focused tests.
- Verification: focused Shifting renderer test, neighbouring prize/HiddenReveal
  renderer tests, `cargo test -q -p wildside-core shifting_block --
  --nocapture`, `cargo test -q -p wildside-cli shifting_block --
  --nocapture`, and `verify-slice-fast`.
- NEXT: continue with `mise run project-status`; the top open row should now be
  #351 unless a higher-priority ready item appears.

## 2026-06-23 handoff (#323 Maniaxe axe render validation)

- **#323 moved `[~]` -> `[?]`.** Current HEAD still traces Maniaxe axes as
  `ActiveProjectileKind::ManiaxeAxe` and native-player renders the Maniaxe
  `stru_8B86` axe frame bank rather than the green friendly projectile rect.
- Evidence: focused trace, sprite, spin-cadence, and live-throw regressions all
  pass. No code change was warranted; this was a stale open-lane row.
- Verification: `cargo test -q -p wildside-core
  maniaxe_axe_projectile_projects_to_trace_stru_8b86 -- --nocapture`; `cargo
  test -q -p wildside-cli
  maniaxe_axe_projectile_uses_stru_8b86_player_form_sprite -- --nocapture`;
  `cargo test -q -p wildside-cli
  maniaxe_axe_projectile_spins_through_stru_8b86_frames -- --nocapture`; `cargo
  test -q -p wildside-cli live_maniaxe_throw_renders_spinning_stru_8b86_frames
  -- --nocapture`.
- NEXT: continue with `mise run project-status`; the top open row should now be
  #347 unless a higher-priority ready item appears.

## 2026-06-23 handoff (#316 raw DP payload storage resolved)

- **#316 moved `[~]` -> `[x]`.** The last helmets/DP audit debt is closed:
  raw shared DP payload storage now lives in `DiamondPowerObjectRamSub3F596`
  beside the active/free/last list and sprite compiler.
- Evidence: live DP powers write/read ROM slot offsets for Circle, Wall,
  Five-Way, Snake, Rain, Extra HP/Life, and Invulnerability/Haze; payload
  vectors now follow `Addr_FirstDPObjectSlot` traversal order. Circle removal
  reads raw `$38(a0)`, and `sub_3F63C` leaves stale raw slot payload intact.
- Verification: focused DP filters (`circle_of_doom`, `diamond_power_rain`,
  `invulnerability`, `diamond_power_snake`, `diamond_power_object`, Five-Way
  startup), `cargo fmt`, `mise run verify-slice-fast`, and `mise run
  verify-tas-full` (exit 0; CRC all 37,759 frames match; known #290
  checkpoint-state baseline still reported).
- NEXT: continue with `mise run project-status`; the top open row should now be
  #323 unless a higher-priority ready item appears.

## 2026-06-23 handoff (#280 route-needed)

- **#280 moved `[~]` -> `[b]`.** Current BLW2, Hills, and Bloody Swamp `$3E`
  probes all become state-red before they can prove a new render fix.
- Evidence: BLW2 still shows a narrow top-right blue seam at frame 60, but object
  counts diverge at `level_settle@60`; Hills drifts in camera by `idle_tail@239`;
  Bloody Swamp `$3E` drifts in camera at `level_settle@60`.
- Tooling fix: `record-scenario` / `probe-record-scenario` now parse hex level ids
  via the shared parser (`e66265603`), so `--level 3E` reaches the intended level.
- Verification: `cargo fmt --check`; focused parser test; `record-scenario --level 3E`
  succeeds; `verify-recording` captures the expected state-red blocker; `mise run
  verify-slice-fast` passed, and the commit hook reran it.
- NEXT: continue with `mise run project-status`. #280 needs an exact current live
  route/frame or oracle-comparable scenario before more Chunked/background code.

## 2026-06-23 handoff (#324 prize wobble fixed candidate)

- **#324 moved `[~]` -> `[?]`.** Prize-block wobble now treats `$229` as a
  live copied VRAM slot, not a blind `+9` lookup into the generated block sheet;
  shifted tiles are used only when pixel-identical to the copied `$220` quad.
- Evidence: ASM `sub_1007A` / `stru_101B6` alternates `unk_E0E4E`/`unk_E0E5E`
  (`$220/$229`). BLW1 row-12 native/oracle captures around frame 31/36 are clean;
  direct row-1 native probe hits `(57,1)` and is clean, but direct row-1 oracle
  capture is rejected because patched camera/start state leaves invalid VDP columns.
- Verification: new regression
  `prize_block_wobble_reuses_base_copy_when_second_slot_is_neighbour_art_sub_1007a`,
  `KID_ENABLE_CACHE_BACKED_TESTS=1 cargo test -p wildside-cli prize_block -- --nocapture`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and `mise run verify-slice-fast`.
- NEXT: continue with `mise run project-status`; #324 needs human validation on
  the canonical BLW1 forest route and a Highwater prize block before `[x]`.

## 2026-06-23 handoff (#256 Highwater priority route-blocked)

- **#256 moved `[~]` -> `[b]`.** Current Highwater priority route and the best
  `760..1240 B+Right` overlay still stop at `(840,1548)`, 75 px left of the
  reported waterfall-front mask `(915,1548)`.
- Renderer invariant remains green: ordinary Kid/enemy sprites stay low priority
  except the narrow flagpole override, and priority-marked foreground terrain masks
  later low-priority sprite tiles, including the Highwater asset-backed test.
- Verification: regenerated `.cache/recordings/issue-256-highwater-water-priority-current-20260623/`,
  `verify-tas` frames 940..970 CRC-clean, native/oracle frame captures at 816/960/1284,
  focused priority tests, and `compare -metric AE` frame 1284 = 259.
- NEXT: continue with `mise run project-status`; #256 needs a fresh live route/frame
  that actually enters the waterfall/foreground mask before implementation work.

## 2026-06-23 handoff (#254 intro skip fixed candidate)

- **#254 moved `[~]` -> `[?]`.** IntroSequence6 Start-skip no longer replays the
  blue hidden-shell VDP after `loc_1BAB0 -> sub_1C246` has entered `Load_TitleScreen`;
  it carries/fades the resident VDP briefly, blacks the restore window, then exposes
  title VDP at the 58-frame visual point.
- Evidence: `.cache/recordings/issue210-intro6-start2400-control/scenario.json`
  original samples now AE 22/0/0/0/0/0/8/0 at frames 2460, 2520, 2600,
  2640, 2700, 2720, 2800, 3000; Start=2500 blackout exact at 2520/2550.
- Verification: cache-backed `cold_boot_intro_sequence_6_` tests,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `mise run verify-slice-fast`.
- NEXT: run `mise run project-status`; with #254 awaiting human, continue the next
  open visual row from the priority-ordered list.

## 2026-06-23 handoff (#320 camera audit closed)

- **#320 moved `[ ]` -> `[x]`.** Rechecked `docs/source-port/audits/camera-fidelity-audit.md`,
  archive #320, and `backlog-validation-2026-06-14`: the audit reports zero
  confirmed camera gaps and all six backlog-candidate rows are already `[x]`.
- Validation had already flagged #320 as stale on 2026-06-14. No code change is
  warranted unless a new non-`[x]` camera candidate or oracle-visible camera
  divergence appears.
- Verification: tracker/audit review; no code change warranted.

## 2026-06-23 handoff (#319 audio-trigger audit closed)

- **#319 moved `[ ]` -> `[x]`.** Rechecked `docs/source-port/audits/audio-triggers-fidelity-audit.md`,
  archive #319, and `backlog-validation-2026-06-14`: the only concrete #319
  follow-up, boss-eye pop `$48`, was resolved on 2026-06-16, and the current
  audit has an empty backlog-candidates table.
- Remaining audio notes are not #319 slices: broad live-audio/device gaps are
  owned by #160 or specific rows, StopMusic/BGM details are native-player/GEMS
  architecture, and PlaySound/PlaySound2 command-byte nuance has no current
  consumer beyond the real `wildside-md` GEMS path.
- Verification: tracker/audit review; no code change warranted.

## 2026-06-23 handoff (#210 fixed candidate)

- **#210 moved `[~]` -> `[?]`.** Direct IntroSequence5 frame-2100 Start/title
  return now matches oracle pixels for the previously red fade/title samples:
  frames 2100..2120, 2160, 2161, 2250, 2278..2282, 2299..2302, and 2310.
- Final residual root cause: frame 2160's 22px `TM` edge was an extra linked
  `loc_1BB10` sprite at hardware `(x,y)=($018C,$0114)`; frame 2161 also needed
  the direct-lineage `loc_1C7A0` SegaText boundary. Both now follow the
  `Load_TitleScreen` first object pass / next-frame child-worker cadence.
- Regression coverage: cache-backed `cold_intro_sequence_5_visible_start_carries_current_vdp_into_title_return`
  checks the 2160/2161 sprite boundary; VDP unit tests cover direct SegaText and
  Chameleon phase lineage.
- Verification this slice: focused tests, fresh native/oracle captures,
  `compare -metric AE` = 0 for frames 2160, 2161, 2250, and 2300,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `mise run verify-slice-fast`, and `mise run verify-tas-full` (exit 0; CRC
  all 37,759 frames match, with the existing long-route state divergence report
  starting at `transition_15_to_00_post`).

## 2026-06-23 handoff (#210 fade/Chameleon phase partial)

- **#210 remains `[~]`.** Direct IntroSequence5 frame-2100 Start now matches
  the ROM `Pal_FadeOut` window (`loc_6E2 -> Pal_FadeOut loc_28C6`) for frames
  2100..2120, and the steady direct title Chameleon cadence is exact at 2250
  and 2300.
- The direct `$24` title return needs a separate `sub_1B71A` phase from the
  IntroSequence6 Start-skip lineage: `Load_TitleScreen` allocates fresh
  `Obj_TitleTextChameleon`, but the first presented direct title VDP is seven
  Chameleon ticks ahead.
- **Remaining:** frame 2160 is still AE 22, all on the tiny `TM` edge beside
  `CHAMELEON` (x=268..278, y=148..152). Rejected direct positive `d2` reveal
  retunes; they clip the main word and make the frame much worse. Next pass
  should inspect the exact `TM` tilemap/plane write, not retune `d2`.
- Verification: `cargo fmt --all`, focused tests including
  `KID_ENABLE_CACHE_BACKED_TESTS=1`, `git diff --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `mise run verify-slice-fast`, and `mise run verify-tas-full` (exit 0; CRC all
  37,759 frames match; existing long-route state divergences still reported).

## 2026-06-23 handoff (#210 intro Start-skip partial)

- **#210 remains `[~]`.** The engine-level Start-skip path now follows ASM
  `loc_1BA60 -> sub_1C246 -> loc_1C264` for direct IntroSequence5 and early
  IntroSequence6 title returns instead of frame-specific constants.
- Focused probes are green for ignored frame-1930 Start, accepted frame-1933
  Start, accepted frame-2100 visible IntroSequence5 Start, and frame-2400
  IntroSequence6 Start. Direct frame captures are exact for the black
  frame-1933 title return and the main frame-2100 carry/black/title samples.
- **Remaining:** visible IntroSequence5 frame-2100 Start still has residual
  fade/title pixels at frames 2106..2110, 2160, and 2300. Next pass should
  derive the direct title-return CRAM fade/title cadence from ASM/oracle before
  moving #210 to human validation.
- Verification: `cargo fmt`, `cargo test -q -p wildside-core cold_intro_sequence_ -- --nocapture`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `mise run verify-slice-fast`,
  and `mise run verify-tas-full` (CRC all 37,759 frames match; known baseline
  state report remains).

## 2026-06-23 handoff (#272 route-blocked)

- **#272 moved `[ ]` -> `[b]`.** Cyclone cue wiring is already present and native can
  emit transform `$32`, but the only successful pickup route is not oracle-valid.
- Level 0x14 route `issue272-level14-cyclone-r0-205-b186-241-l205-280` queues
  `StartSequence(50)` at the Cyclone pickup, then verifier rejects it: audio gate fails
  at `route_tail@219`, and `verify-tas` first CRC-splits at frame 200 on block contact.
- Level 0x1B and 0x25 visible Cyclone starts were searched with simple fall/jump/right/left
  overlays; both produce no target block bounce.
- **Next:** exact human/current audio route or a new state-aligned natural Cyclone pickup
  route. Do not retune GEMS, cue ids, or Cyclone transform mapping from native-only evidence.

## 2026-06-23 handoff (#259 route-blocked)

- **#259 moved `[~]` -> `[b]`.** The latest BLW2 shooting-tank candidate activates
  0x0B layouts but diverges before the target proof window, so it is route-search
  evidence only.
- ASM still supports the prior hypothesis: RockTank body Y is layout-driven and the
  linked mouth/turret is body_y-8; no current evidence justifies changing Tank Y,
  priority, or renderer occlusion.
- **Next:** a human/current natural route must reach the named BLW2 teleporter/flag
  tanks or first Highwater tank while staying native/oracle comparable.

## 2026-06-23 handoff (#353 route-blocked)

- **#353 moved `[ ]` -> `[b]`.** Current USM 0x04-0x06 evidence does not reproduce
  invisible spike-metal blocks.
- Generated layouts contain zero `$F5` collision-7/block-type-5 cells; every USM
  collision-7 hazard is plain `0x70xx` terrain. Representative native captures for
  USM1, USM2, and USM3 all draw visible spike/lava/skull hazard art.
- Aligned settled spawn native/oracle frame captures for USM2 and USM3 show no
  missing-spike gap. Direct off-screen oracle camera probes are rejected because the ROM
  camera follows after the one-time camera patch.
- **Next:** keep #353 out of the open lane until a human supplies an exact route,
  coordinate, or frame where Kid dies on a visually empty hazard.

## 2026-06-23 handoff (#318 movement audit closed)

- **#318 moved `[~]` -> `[x]`.** The movement-fidelity audit now has no
  standalone open rows after rechecking the stale platform/climb entries and
  the #224 `$14`/`$18` stuck-in-solid recovery reclassification.
- `sub_DB22` is now live-wired on airborne-to-grounded transitions before
  grounded `sub_78E8`, with mounted Skycutter board landings preserved on the
  `$FA6A` board loop/bounce path.
- Focused regressions for `sub_DB22`, Berzerker landing, Skycutter board
  landing, platform `sub_8F26` carry, `loc_A9AA` climb/platform exit, and
  airborne Iron Knight attach pass.
- Verification: `cargo fmt --all --check`, `cargo check -q -p wildside-core`,
  `git diff --check`, `mise run backlog-lint`, `mise run verify-slice-fast`,
  and `mise run verify-tas-full` (CRC all 37,759 frames match; known baseline
  divergences only).
- **Next:** `project-status` should advance to #319 unless #316 remains the
  higher-priority in-progress umbrella.

## 2026-06-23 handoff (#317 blocked: weather VRAM backend)

- **#317 moved `[~]` -> `[b]`.** After the lava `loc_14940` slice, the remaining
  weather audit residue is only the shared typed VRAM/backend cache.
- Concrete weather render paths now have live coverage for Island shore DMA
  frame patching, storm shared art and pattern plane, lava script/full-height
  rows including `$4C/$4A`, and weather-4 Unknown plane fill.
- The remaining work is a cross-effect mutable VDP/VRAM renderer model, not a
  narrow unattended bug slice; reopen with a deliberate architecture plan or a
  concrete pixel/oracle route proving the current per-effect render paths diverge.
- Verification for the block/status move: `mise run backlog-lint` and
  `git diff --check`.
- **Next:** `project-status` should advance to #318.

## 2026-06-23 handoff (#317 lava loc_14940 top alternation)

- **#317 remains `[~]`.** Active lava geyser traces now carry the ASM
  `loc_14874/loc_14940` full-height render state: current `d3` sprite-row
  index and `$4C(a5)` top-template bit.
- Native-player consumes that state for current recordings, so full-height
  rows select `dword_14636` when `$4C(a5)` is non-zero instead of always
  using `unk_14624`.
- Legacy three-field `active_geysers` traces deserialize with sentinel render
  state and replay verification normalises that sentinel only when the compact
  `(x,y,height)` eruption identity still matches.
- Regressions: `loc_14940_exposes_full_height_top_frame_alternation_for_rendering`,
  `native_trace_frame_reads_legacy_three_field_lava_geysers`,
  `build_game_scene_uses_lava_geyser_top_alt_template_dword_14636`, and
  `first_trace_drift_accepts_legacy_frames_missing_lava_geyser_render_state`.
- Verification: `cargo check -q -p wildside-core -p wildside-cli`,
  `cargo test -q -p wildside-core lava_geyser -- --nocapture`,
  `cargo test -q -p wildside-core native_trace_frame_reads_legacy_three_field_lava_geysers -- --nocapture`,
  `cargo test -q -p wildside-cli lava_geyser -- --nocapture`,
  `cargo test -q -p wildside-cli first_trace_drift_accepts_legacy_frames_missing_lava_geyser_render_state -- --nocapture`,
  `cargo fmt --all --check`, `git diff --check`, `mise run backlog-lint`,
  and `mise run verify-slice-fast` (`6285 + 9` tests; TAS frontier green,
  known oracle-limitation-only notes).
- **Next:** keep #317 open for the shared typed VRAM backend.

## 2026-06-23 handoff (#317 lava loc_14874 full-height rows)

- **#317 remains `[~]`.** Full-height lava geyser rows now route through ASM
  `loc_14874/loc_148F8/loc_14914` row planning instead of the old
  smooth-height column approximation.
- The core planner emits `unk_14624`/`dword_14636` top rows, `unk_1465A`
  middle rows, and `unk_14650` bottom rows with the ASM camera windows.
- Native-player consumes that planner for `active_geysers`, so off-window
  full-height rows do not draw and do not fall back to the placeholder column.
- Regressions: `loc_14874_plans_full_height_rows_through_the_asm_camera_windows`,
  `loc_14874_full_height_planner_culls_or_skips_rows_like_the_rom`,
  `build_game_scene_draws_lava_geyser_from_loc_14be8_sprite_tiles`, and
  `build_game_scene_culls_lava_geyser_full_height_rows_loc_14874`.
- Verification: `cargo test -q -p wildside-core lava_geyser -- --nocapture`,
  `cargo test -q -p wildside-cli lava_geyser -- --nocapture`,
  `cargo fmt --all --check`, `cargo check -q -p wildside-core -p wildside-cli`,
  `git diff --check`, `mise run backlog-lint`, and `mise run verify-slice-fast`.
- **Next:** keep #317 open for live `$4C/$4A` top-row alternation state and
  the shared typed VRAM backend.

## 2026-06-23 handoff (#317 lava loc_147BA template rows)

- **#317 remains `[~]`.** Lava geyser repeat cycles now follow ASM
  `loc_1478E -> loc_1479C/unk_146A0` instead of jumping directly from the
  repeat sound gate to `loc_1484E`.
- Trace output now exposes `lava_geyser_script_templates_loc_147ba` for the
  current `unk_146B6`/`unk_146A0` template label reached by `loc_147BA`.
- The steady `unk_146A0` entries now keep each template live for three frames,
  matching the ASM `d2=2` script words before `$FFFF -> loc_1484E`.
- Script rows now use ASM `loc_147D6` camera culling, so off-window script
  rows do not render and do not fall back to the legacy lava column.
- Native-player renders the exact `unk_145CA`, `unk_145D4`, `unk_145DE`,
  `unk_145E8`, and `unk_14602` template rows for those script frames, then
  skips the old conservative height-column fallback for the same geyser/frame.
- Regressions: `loc_147ba_exposes_startup_and_repeat_template_labels`,
  `trace_frame_exports_lava_geyser_script_template_loc_147ba`, and
  `build_game_scene_draws_lava_geyser_script_template_loc_147ba`.
- Verification: `cargo test -q -p wildside-core lava_geyser -- --nocapture`,
  `cargo test -q -p wildside-cli lava_geyser -- --nocapture`,
  `cargo fmt --all --check`, `cargo check -q -p wildside-core -p wildside-cli`,
  `git diff --check`, `mise run backlog-lint`, `mise run verify-slice-fast`,
  and, after the repeat-cadence gameplay fix, `mise run verify-tas-full` (CRC
  all 37,759 frames match; known baseline 212 authoritative divergences + 2
  oracle limitations).
- **Next:** keep #317 open for exact full-height row-window/backend modelling
  and the shared typed VRAM backend.

## 2026-06-23 handoff (#352 HiddenReveal render worker)

- **#352 moved to `[?]` fixed candidate.** Native-player now draws typed
  HiddenReveal block-bounce workers during ASM `sub_DFB0 loc_E096`'s
  temporary `$60xx` no-block phase.
- The renderer resolves the preserved skin byte as block art and applies the
  traced `loc_E2E2/loc_E2F0` y-bob, so hidden jump-up blocks render a reveal
  object instead of snapping at the static cell origin.
- The separate #347 Shifting rise path remains unchanged; its row still needs
  the oracle mid-rise capture called out in archive #347 before rendering a
  no-block Shifting worker.
- Regression: `build_scene_draws_hidden_reveal_worker_over_solid_placeholder_loc_e096`,
  with neighbouring guards
  `non_prize_cleared_block_bounce_keeps_foreground_terrain_visible` and
  `build_scene_draws_prize_block_wobble_sprite_even_after_deleted_cell_write`
  still green.
- Verification: focused CLI tests, `cargo fmt --all --check`, `backlog-lint`,
  and `verify-slice-fast`.
- **Next:** continue with `mise run project-status`; remaining open b748 rows
  are #259/#272 or blocked/human-validation items, not #352.

## 2026-06-23 handoff (#317 storm loc_30110 scroll gate)

- **#317 remains `[~]`.** Native now models ASM
  `Transfer_ScrollDataToVRAM loc_300FE/loc_30110` for storm Scroll B:
  `$FFFFFAD6` counts down after the `$834` storm-intro window, the zero
  frame emits a trace record, resets `$FAD6=4`, and subtracts 8 from
  `$FFFFFAD8`.
- Native-player now draws the `loc_14D32` storm pattern plane only when the
  trace carries the `loc_30110` transfer, using the second HScroll/VSRAM words
  as the Scroll-B offset; the hail `$1813` attribute-bit test still covers H/V
  flip preservation.
- Regressions: `sub_6048_after_storm_intro_preserves_weather_mode_byte_for_loc_30110_countdown`,
  `transfer_scroll_data_storm_loc_30110_counts_down_and_exposes_trace`,
  `build_game_scene_gates_storm_pattern_plane_on_loc_30110_transfer`,
  `build_game_scene_draws_storm_pattern_plane_loc_30110`,
  `build_game_scene_scrolls_storm_pattern_plane_from_loc_30110_offset`, and
  `first_trace_drift_accepts_legacy_frames_missing_storm_scroll_transfer_diagnostics`.
- Verification: `cargo test -q -p wildside-core sub_6048 -- --nocapture`,
  `cargo test -q -p wildside-core transfer_scroll_data_storm_loc_30110 -- --nocapture`,
  `cargo test -q -p wildside-cli storm_pattern -- --nocapture`,
  `cargo test -q -p wildside-cli first_trace_drift_accepts_legacy_frames_missing_storm -- --nocapture`,
  plus final `cargo fmt --all --check`, `cargo check -q -p wildside-cli`,
  `git diff --check`, `mise run backlog-lint`, and `mise run verify-slice-fast`.
- **Next:** keep #317 open for full lava script-walk/template-row output and
  the shared typed VRAM backend.

## 2026-06-23 handoff (#317 storm pattern plane renderer)

- **#317 remains `[~]`.** Native-player now consumes ASM
  `Init_SpecialEffect_Storm loc_14D1E/loc_14D32` pattern-plane uploads
  from `unk_9B65C` / `unk_9B6E0`.
- The renderer draws the init-time Scroll-B pattern using the recorded
  `$0380_0380` addend, `$6000` VDP command shape, and post-header 8x8
  source table; hail word `$1813` keeps its H/V flip attribute bits.
- Regressions: `build_game_scene_draws_storm_pattern_plane_loc_14d32`,
  `build_game_scene_preserves_hail_storm_pattern_attr_bits_loc_14d32`,
  `first_trace_drift_accepts_legacy_frames_missing_storm_pattern_diagnostics`,
  and `init_weather_seeds_storm_machine_from_loc_14dac`.
- Verification: `cargo test -q -p wildside-cli storm_pattern -- --nocapture`,
  `cargo test -q -p wildside-cli first_trace_drift_accepts_legacy_frames_missing_storm_pattern_diagnostics -- --nocapture`,
  `cargo test -q -p wildside-core init_weather_seeds_storm_machine_from_loc_14dac -- --nocapture`,
  plus final `cargo fmt --all --check`, `cargo check -q -p wildside-cli`,
  `git diff --check`, `mise run backlog-lint`, and `mise run verify-slice-fast`.
- **Next:** keep #317 open for exact `Transfer_ScrollDataToVRAM loc_30110`
  storm timing/scroll, full lava script-walk/template-row output, and the
  shared typed VRAM backend.

## 2026-06-23 handoff (#317 storm shared art renderer)

- **#317 remains `[~]`.** Native-player now consumes ASM
  `Init_SpecialEffect_Storm loc_14D8A` shared `ArtComp_9A7D2 -> $76A0`
  uploads for `sub_61CA` storm sprites.
- Tile words in the shared range, e.g. `$A3CD`, resolve against base `$3B5`
  (local tile `$18`) when the trace carries the upload; missing upload/assets
  keep the existing `$380` rain/hail fallback.
- Regressions: `load_level_visuals_loads_storm_weather_art_loc_14cce_14cd8`,
  `build_game_scene_draws_storm_weather_sprites_from_sub_61ca`, and
  `build_game_scene_uses_loc_14d8a_shared_weather_art_for_storm_sprites`.
- Verification: `cargo fmt --all --check`, `cargo check -q -p wildside-cli`,
  `git diff --check`, focused render/native-player tests, and
  `mise run verify-slice-fast`.
- **Next:** keep #317 open for the storm pattern plane renderer, full lava
  script-walk/template-row output, and the shared typed VRAM backend.

## 2026-06-23 handoff (#252 RockTank bullet render)

- **#252 moved to `[?]` fixed candidate.** The current renderer already covered
  the RockTank body/turret doubled/humanoid path; the stale live TankBullet test
  now exercises a public settled Level 1 route and waits for a real
  `ActiveProjectileKind::TankBullet`.
- The renderer regression recentres only the render viewport around that live
  projectile, then samples visible ASM `Tank_Shooting.asm` `stru_3B526` sprite
  pixels from ROM tiles `$038..=$03B`, rejecting both the orange fallback rect
  and friendly green projectile colour.
- Manual frame dumps for `--frame 38,10,05` and `--frame 3A,0F,05` show grey
  RockTank bullet art in `.cache/debug/issue252-tank-bullet-*.png`.
- **Next:** human playtest should walk BLW2/Highwater past shooting RockTanks
  and confirm coherent grey body/turret plus grey bullet art; floor-sink/
  foreground occlusion remains #259.

## 2026-06-23 handoff (#317 lava geyser disable path)

- **#317 remains `[~]`.** Live lava geysers now honour the `unk_146B6`
  startup path through the `$FFFE -> loc_1483A` disable coroutine.
- `step_lava_geysers` emits five startup-script frames, queues the unconditional
  `PlaySound2` geyser sound at `loc_1483A`, hibernates `$3C` frames, then
  resumes through the visible `loc_1478E` repeat sound gate into the existing
  `loc_1484E` rise path; collapse restarts now follow `loc_1498C -> loc_14788`.
- Regressions: `step_lava_geysers_startup_script_hits_fffe_disable_loc_1483a`,
  `step_lava_geysers_disabled_phase_resumes_repeat_loop_loc_1478e`, plus the
  lava-geyser filter.
- **Next:** keep #317 open for full geyser script-walk/template-row output,
  storm pattern/shared-art renderer consumption, and the shared typed VRAM
  backend.

## 2026-06-23 handoff (#317 Unknown foreground plane render)

- **#317 remains `[~]`.** Native-player now consumes weather-4 Unknown
  `Init_SpecialEffect_Unknown` `loc_14E42` foreground-plane fills.
- `wildside-render` loads `scenes/artunc/unknown_1.bin` (`unk_9AA50`) as
  16 raw 4bpp tiles at VRAM tile `$380`, and native-player reconstructs the
  `$8380..$838F` Scroll A tile-word pattern from the ASM loop shape.
- Regressions: `load_special_effect_unknown_visuals_reads_scene_art_loc_14de4`
  and `build_game_scene_draws_unknown_foreground_plane_loc_14e42`.
- **Next:** keep #317 open for storm pattern/shared-art renderer consumption,
  the lava script walker/intro row output, `$FFFE -> loc_1483A` disable
  coroutine, and the shared typed VRAM backend.

## 2026-06-23 handoff (#317 lava geyser sound gate)

- **#317 remains `[~]`.** Lava geyser phase-0 transitions now model the
  first-eruption vs repeat path and the ASM `$19(a3)` sound-suppression flag.
- Live `step_lava_geysers` queues `QueuedSoundEffect::LavaGeyser` (`$1E`) for
  loc_14772 startup and loc_1478E repeat sounds only when the geyser is visible.
- Regressions: `step_lava_geysers_phase_0_queues_first_sound_when_visible_loc_14772`,
  `step_lava_geysers_phase_0_skips_first_sound_when_clipped_loc_14772`,
  `step_lava_geysers_phase_0_queues_repeat_sound_after_startup_path_loc_1478e`,
  and `loc_14772_only_plays_the_starting_lava_geyser_sound_when_the_flag_is_clear`.
- **Next:** keep #317 open for the actual script walker/intro row output,
  `$FFFE -> loc_1483A` disable coroutine, storm/Unknown plane renderer
  consumption, and the shared typed VRAM backend.

## 2026-06-23 handoff (#317 storm shared art trace)

- **#317 remains `[~]`.** Storm init now records ASM `Init_SpecialEffect_Storm`
  `loc_14D8A`: shared `ArtComp_9A7D2` decompression to VRAM word `$76A0`.
- The record is exposed through `NativeTraceFrame::storm_shared_art_upload_loc_14d8a`
  and replay comparison treats the absent field as legacy additive diagnostics.
- Regressions: `loc_14d8a_selects_the_shared_weather_art_and_seeds_the_later_slot_fill_loop`,
  `init_weather_clears_previous_state`, `init_weather_seeds_storm_machine_from_loc_14dac`,
  and `first_trace_drift_accepts_legacy_frames_missing_storm_shared_art_diagnostics`.
- **Next:** keep #317 open for renderer consumption of recorded storm/Unknown plane
  uploads, lava geyser script-walk/sentinel sound timing, and the shared typed VRAM backend.

## 2026-06-23 handoff (#317 Island shore DMA source labels)

- **#317 remains `[~]`.** Island shore `loc_1202A` DMA trace events now carry the resolved
  `off_1220E` art source labels (`unk_9784A`, `unk_97B2C`, `unk_97E0E`, `unk_980F0`,
  mirrored `unk_97E0E`, `unk_97B2C`) instead of only frame indices and offsets.
- This closes the weather-audit source-label subgap while preserving legacy recording
  deserialisation for older `IslandShoreDmaUploadLoc1202A` JSON.
- Regressions: `loc_1220e_resolves_island_shore_frame_source_labels`,
  `begin_level_entry_spawns_island_shore_cycle_loc_11fb0`,
  `step_island_shore_cycle_ticks_loc_12098_12110_12188`, and
  `first_trace_drift_accepts_legacy_frames_missing_island_shore_diagnostics`.
- **Next:** keep #317 open for storm/Unknown plane renderer consumption, lava geyser
  script-walk/sentinel sound timing, and the shared typed VRAM backend.

## 2026-06-23 handoff (#355 Cyclone flight spin render)

- **#355 moved `[ ]`→`[?]`.** Cyclone flight now mirrors ASM `sub_B270 loc_B2A0`:
  core advances `$FFFFF8F0` from post-physics `Cyclone_YAcceleration`, trace exposes
  `Cyclone_flying`, and native-player renders `cyclone_spinning_1..3` while that flag is set.
- The old "grounded continuous spin" hypothesis was rejected by the ASM; grounded Cyclone still
  uses ordinary `loc_9600/off_96D8` walk frames. Cyclone transform audio remains separate #272.
- Regressions: `cyclone_flight_advances_spin_phase_sub_b270_loc_b2a0`,
  `cyclone_flight_uses_spin_frame_sub_b270_loc_b2a0`, and
  `grounded_cyclone_keeps_walk_table_when_not_flying_loc_9600`.
- Verification: focused tests, `cargo fmt --all --check`, `cargo check -q -p wildside-core`,
  `cargo check -q -p wildside-cli`, `cargo check -q -p wildside-render`, `git diff --check`,
  and `mise run verify-slice-fast` (clippy/hooks/backlog, TAS frontier slices, 6269 core tests
  + 9 CLI tests).
- **Next:** `mise run project-status` shows no ready item; top open playability risk is
  #317 and top visual risk is #210. #355 now waits for human natural Cyclone validation.

## 2026-06-23 handoff (#344 walking Dragon hurt render)

- **#344 moved `[ ]`→`[?]`.** Nonlethal walking-Dragon stomps now render ASM
  `Dragon.asm loc_3CEBE` / `stru_3CCE6`: the single `unk_C755E` hurt frame during
  hibernate. Native previously free-ran the normal walk cycle while `damage_flash=true`.
- HP-zero death rendering is unchanged and still uses `loc_3CEFC` / `stru_3CCEC`;
  breath-phase rendering still uses `loc_3CF90` when not hurt.
- Regressions: new `nonlethal_walking_dragon_stomp_uses_hurt_frame_loc_3cebe_stru_3cce6`
  plus existing live walking, breath-phase, and zero-HP death renderer tests.
- Verification: focused renderer tests, `cargo fmt --all --check`,
  `cargo check -q -p wildside-cli`, `git diff --check`, `mise run backlog-lint`,
  and `mise run verify-slice-fast` (clippy/hooks/backlog, TAS frontier slices,
  6268 core tests + 9 CLI tests). Render-only CLI change, so `verify-tas-full`
  was not required.
- **Next:** `mise run project-status` shows no "ready to implement now" items; top open
  playability risk is #355 and top visual risk is #210.

## 2026-06-23 handoff (#257 continue-screen slide)

- **#257 moved `[ ]`→`[x]`.** `Continue_Screen` text/count now use the ASM `$46/$A=$1C`
  worker duration; native had capped the EOL text/number overlay at 8 frames, leaving
  `CONTINUES` half off left and the count off the right.
- Repro: `.cache/debug/issue257/one_life_continue.json`; transcript enters `game_over`
  at frame 1203 and `continue_countdown` at frame 1213. Before native frame 1290 showed
  the off-centre partial label; after capture
  `.cache/debug/issue257/native-after-tight/issue257_one_life_continue_card_frame_01290.png`
  shows centred, legible `CONTINUES` + `3`.
- Regressions: `continue_countdown_holds_before_slide_in_loc_d31e` now asserts final
  text/count x positions; existing continue/no-continue EOL sprite tests still pass.
- Verification: focused continue/no-continue tests, visual before/after capture,
  `cargo fmt --all --check`, `cargo check -q -p wildside-cli`, `git diff --check`,
  `mise run backlog-lint`, and `mise run verify-slice-fast` (clippy/hooks/backlog,
  TAS frontier slices, 6268 core tests + 9 CLI tests). Render-only CLI change, so
  `verify-tas-full` was not required.
- **Next:** `mise run project-status` shows no "ready to implement now" items; top open
  playability risk is #344 and top visual risk is #210.

## 2026-06-23 handoff (#316 DP sprite compiler)

- **Current #316 remains `[~]`, with the `DiamondPower_CompileSprites sub_3F798` row fixed as a code candidate.** Runtime now mirrors the frame-local DP sprite compiler state: `DiamondPower_Run loc_3F6B0` clears the buffer/count, direct DP sprite paths write size/link bytes immediately, and small Invulnerability/Haze sprites buffer through `loc_3FDAE` before `sub_3F798` links them and masks pattern bit 15.
- Circle/Wall/Five-Way/Snake/Extra/large Invuln-Haze sprite snapshots now carry the ROM link byte in `size_word` low byte while preserving pattern bit 15. Buffered small Invuln/Haze snapshots are compiled through the existing `diamond_power_compile_sprites_sub_3F798` helper, so their pattern high bit is cleared like the ROM.
- Regressions: `invulnerability_small_sprites_compile_via_sub_3f798_loc_3fdae`, `diamond_power_compile_sprites_adjusts_link_and_masks_pattern_loc_3f7ae`, updated live DP sprite assertions, and the full `diamond_power` filter.
- Verification: focused compiler/live sprite tests, `cargo test -q -p wildside-core diamond_power -- --nocapture` (184 pass), `cargo check -q -p wildside-core`, `cargo fmt --all --check`, `git diff --check`, `mise run verify-slice-fast` (clippy/backlog/frontier + 6268+9 tests), and `mise run verify-tas-full` (CRC all 37,759 frames match; known baseline divergences only).
- **Next:** continue #316 from the remaining DP architectural debt: exact raw shared payload storage instead of per-power vectors, then rerun `mise run project-status` to confirm priority.

## 2026-06-23 handoff (#316 DP object-list runtime mirror)

- **Current #316 remains `[~]`, with the shared DP object-list row fixed as a code candidate.** Runtime now mirrors ASM `sub_3F586/3F596/3F5BC/3F5E8/3F63C`: ten-slot free list from `$FFFFF2AC`, active/free/last pointers, allocation on activation/spawn, removal on status/cull, and shared flag clears on list reset/empty.
- Existing per-power payload vectors stay in place, but each live payload now has the ROM slot address projection beside it. Circle/Wall/Five-Way/Snake/Rain removals update the shared list through `sub_3F5E8`; Extra HP/Life and Invulnerability/Haze expiry reset through `sub_3F63C`; level reload clears through `sub_3F586`.
- Regressions: `circle_of_doom_activation_seeds_shared_dp_object_list_sub_3f596_3f5bc`, `circle_of_doom_status_removal_updates_shared_dp_object_list_sub_3f5e8`, `invulnerability_expiry_resets_shared_dp_object_list_sub_3f63c`, plus the pure `diamond_power_object_*` tests.
- Verification: `cargo test -q -p wildside-core diamond_power_object -- --nocapture`, `cargo test -q -p wildside-core shared_dp_object_list -- --nocapture`, `cargo test -q -p wildside-core diamond_power -- --nocapture` (184 pass), `cargo check -q -p wildside-core`, `cargo fmt --all --check`, `mise run backlog-lint`, `git diff --check`, `mise run verify-slice-fast`, and `mise run verify-tas-full` (CRC all 37,759 frames match; known baseline divergences only).
- **Next:** continue #316 from the next concrete open audit row: raw shared payload storage / `DiamondPower_CompileSprites sub_3F798` live sprite-table compile wiring.

## 2026-06-23 handoff (#265 real-device-blocked)

- **#265 moved `[~]`→`[b]`; no code patch.** Current HEAD still emits the EOL score-counter command pair in both scripted export and dummy-SDL live playback: `StartSequence(12)` at frame 314 (`loc_D752`/`loc_D770`) and `StopSequence(12)` at frame 558 (`loc_D7F4`), followed by `StopMusic` at the first `LevelTransition` frame.
- Regenerated `.cache/recordings/issue-265-eol-audio-current/` and replay-verified it native-only. The live dummy-SDL bundle `.cache/recordings/issue265-eol-live-sdl-current/` has the same audio-events transcript, no skipped full frames, no queue errors, and `max_backlog_frames=1`.
- WAV checks show score-counter energy in the tally windows and silence after the transition, so another headless StopMusic/suppression tweak would be speculative and risks reintroducing the inverted symptom.
- Verification: `record-scenario`, `verify-recording --native-only`, dummy-SDL `play-native --record-frames 2400`, `ffmpeg` WAV window checks, `mise run backlog-lint`, and `git diff --check`.
- **Next:** run `mise run project-status`; #257 continue-screen render should remain the highest open P0 unless the queue changes.

## 2026-06-23 handoff (#348 fixed candidate)

- **#348 moved `[~]`→`[?]` fixed candidate.** Live/direct title-card entry now follows ASM `loc_1AE2E`: after the one-frame hibernate, the first Start edge releases `Game_Mode=$000C` with no invented 343-frame minimum and no release requirement.
- ScriptRunner/TAS playback enables `recorded_title_card_handoff_delay`, preserving the issue305 recorded timing: Start edge and release frame stay in `$0008`, then playback releases on the following frame. Respawn Start handling now latches the edge for the one-frame `lose_life -> loc_6E2 -> Load_InGame` handoff instead of requiring Start to be held.
- Regressions cover live title-card first-edge entry, metadata/no-worker entry, level-complete title-card handoff, replay release-frame delay, and death/continue respawn Start-edge handling. The Level 0x07 probe with `--input 330:start` now reaches `InGame` at frame 331, and still observes the target Lion on the route, so #345 remains blocked on an exact missing-Lion route.
- Verification: focused `loc_1ae2e`, `respawn_hold`, `continue_restores_lives_and_respawns`; issue305 fast-action/options TAS probes; Level 0x07 Start probe; `cargo fmt --all --check`; `cargo check -q -p wildside-core`; `cargo check -q -p wildside-cli`; `mise run verify-slice-fast`; `mise run verify-tas-full` (CRC all 37,759 frames match; known baseline divergences only).
- Human still needs to playtest title-card Start feel in native play before promoting #348 to `[x]`.
- **Next:** run `mise run project-status`; #257 continue-screen render should remain the highest-priority open P0 unless the queue changes.

## 2026-06-23 handoff (#345 route-blocked)

- **#345 moved `[~]`→`[b]`; no gameplay patch.** The reported Lion absence after death+respawn on Level 0x07 did not reproduce in settled native/oracle probes. Both sides have a Lion at frame 0 of `.cache/recordings/issue345-settled-level07-idle/scenario.json`, and native keeps one active at `level_settle@60`.
- Re-reading `Lion.asm sub_3E8BA` still leaves a real audit smell: ROM cull bands use `(Number_Objects)`, while native `large_pacer_should_remain_active` uses active enemy roster length. That is not enough evidence for a patch because the reproduced divergence is an X offset, not lifecycle absence.
- The natural post-death route reaches `LevelTransition`; Start overlays at 330/420 remain on the card because #348 level-card Start handling is still open. Re-run #345 after #348, or with an exact human route/frame that reaches live respawn and shows a native-only missing Lion.
- Verification: `inspect-level --level 07`, `probe-record-scenario` settled/native probes, `record-scenario --slug issue345-settled-level07-idle`, `probe-record-scenario-file`, `verify-tas` and `tas-diff` over the settled route, `mise run backlog-lint`, and `git diff --check`.
- **Next:** run `mise run project-status`; #345 is no longer in the open queue.

## 2026-06-23 handoff (#339 fixed candidate)

- **#339 moved `[~]`→`[?]` fixed candidate.** The 2026-06-21 outgoing drone-stop fix missed a clean incoming level-load path: native replayed the first USM 0x04 level cue as plain `Start(6)` when `active_bgm_cue=None`, so ASM `Load_InGame loc_23629` `j_StopMusic` never reset GEMS at the incoming boundary.
- `inspect-level --level 04` shows clean USM 0x04 contains Dragon/Drip/EmoRock objects only, no RockTank, so the reported load-time "washing" drone fits stale incoming GEMS state rather than a fresh local `$1F` tank start.
- `native_bgm_state_after_trace` now emits `StopMusicThenStart(bgm)` for cold `Level`/`CoreQueued` cues, preserving the existing `StoppedForTransition` sentinel after explicit transition resets. The clean recording changed from `.cache/recordings/issue339-clean-usm04-audio-current/audio-events.txt` frame 0 `Start(6)` to `.cache/recordings/issue339-clean-usm04-audio-load-ingame-reset/audio-events.txt` frame 0 `StopMusicThenStart(6)`.
- Verification: focused #339 BGM tests, `cargo test -q -p wildside-cli play_native_`, md StopMusic regression, native replay of `issue339-clean-usm04-audio-load-ingame-reset`, `backlog-lint`, and `verify-slice-fast`.
- Human still needs a real-device listen on USM 0x04 clean/death/skip loads; the drone should not carry into the loaded level.
- **Next:** run `mise run project-status`; with #339 back in the human lane, #257 continue-screen render is likely the highest open P0 unless the queue changes.

## 2026-06-23 handoff (#336 oracle-faithful)

- **#336 moved `[~]`→`[-]` NOT-A-BUG / ORIGINAL_QUIRK.** The missing live oracle gate is now met: a dynamic cold-boot route that presses Start on the BLW1 level card (`.cache/debug/issue336-cold-boot-live/scenario-card-start.json`) is CRC/checkpoint clean across frames 880..1040.
- Direct oracle WRAM watches on frames 970..996 confirm the ROM uses the same `loc_1AE2E -> loc_6E2 -> Load_InGame` ramp as native: camera Y `$00D8_0000`→`$0073_0000` while Kid Y `$00FF_0000`→`$010F_0000`. The visible treetop-to-floor presentation is faithful, so changing the ramp would be a deviation.
- The separate "level/title card ignores early Start" feel issue remains #348. Reopen #336 only with a new native-vs-oracle divergence.
- Verification: `verify-tas --scenario .cache/debug/issue336-cold-boot-live/scenario-card-start.json --frame-start 880 --frame-end 1041 --checkpoint-interval 1 --frontier-kind gameplay`, oracle PNG inspection, and `oracle-probe --watches 0xFFF820:u32,0xFFF85E:u32,0xFF24FA:u32,0xFF24FE:u32` over frames 970..996.
- **Next:** run `mise run project-status`; #257 continue-screen render should remain the highest-priority open P0 unless the queue changes.

## 2026-06-23 handoff (#289 fixed candidate)

- **#289 moves to `[?]`; the deterministic Level 0x07 settled recording is now exact.** The remaining 7-pixel frame-71 residual was the top-right ankh/life icon, not foliage/background.
- ASM `sub_44DC loc_4546/loc_4586` stores `AniArt_LifeIcon` speed/frame pairs at offsets 0, 2, 4, and 6. Settled recording compatibility now scopes the current-page exception to `$FFFFFA5E=6 -> tile base 4`; `$FFFFFA5E=4` stays on the frozen HUD page, preserving frame 60.
- Regression coverage adds `settled_level_skip_hud_life_icon_keeps_verified_sub_44dc_phases_loc_4546`, pinning both frame phases.
- Refreshed `issue289-level07-idle-current` is trace/state/frame green: frame 60 exact (`0` / MAE `0.0`) and frame 71 exact (`0` / `0.0`). Visual diff panes are blank at both checkpoints.
- Verification: focused `sub_44DC` HUD tests, green #289 recording, and `verify-slice-fast` green.
- Human still needs to validate the original hell-theme start/layout symptom and O14 Lion-projectile report in live play; do not reopen #289 without a fresh route/frame because the deterministic repro is closed.

## 2026-06-23 handoff (#289 settled HUD diamond latch)

- **#289 stays `[~]`; the Level 0x07 settled-recording HUD diamond strip is gone.** Normal trace-scene HUD rendering still presents the subsequent `sub_44DC loc_45EE` `unk_461C` page, but settled recording compatibility keeps `LevelSkip_Cheat` armed with the synthetic one-second `Level_completion_time` seed while gameplay is active, and that reduced oracle path already samples `$FFFFFA5D` at the visible page.
- Regression coverage adds `settled_level_skip_hud_diamond_icon_uses_current_sub_44dc_offset_loc_45ee` while keeping `game_hud_diamond_icon_prefers_traced_sub_44dc_offset` green for the Level 0x06 normal-path fix.
- Refreshed `issue289-level07-idle-current` remains expected-red but trace/state-green: frame 60 improves from `45` / MAE `0.04169922` to exact `0` / `0.0`; frame 71 improves from `52` / `0.05170666` to `7` / `0.01000744`.
- Remaining residual is only the frame-71 top-right background/foliage edge pixels at `x=289..293,y=13..16`. Next #289 pass should inspect that edge/provenance, not HUD diamond, HUD timer, Island shore age, selected Plane-B rows, global `Time_Frames`, static cloud H-scroll, simple `loc_12110`, or broad HUD phase/position tuning.
- Verification: exact normal/settled HUD diamond tests, expected-red `verify-recording --slug issue289-level07-idle-current`, and visual diff inspection. Full gates recorded in the commit session.

## 2026-06-23 handoff (#289 recording timer phase)

- **#289 stays `[~]`; the Level 0x07 settled-recording HUD timer phase now matches oracle watches.** ASM `Init_SpriteAttr_HUD loc_BDC0` seeds the timer path before the first scripted recording frame; the recording settle shim now starts at 2:59/$08 so frame 0 decrements to 2:59/$07, frame 60 is 2:58/$07, and frame 71 rolls to 2:57/$38.
- Regression coverage extends `issue289_level07_settled_recording_primes_island_shore_vram_loc_1202a` with timer assertions at frames 60 and 71 while keeping `Time_Frames` and the Island shore frames pinned.
- Refreshed `issue289-level07-idle-current` remains expected-red but trace/state-green: frame 60 stays `45` / MAE `0.04169922`; frame 71 improves from `83` / `0.12003813` to `52` / `0.05170666`. Remaining residual is the HUD diamond strip at both checkpoints plus seven frame-71 top-right foliage/edge pixels, not the visible timer.
- Next #289 pass should inspect the HUD diamond latch/current-page split or top-right foliage/edge samples, not the HUD timer, Island shore age, selected Plane-B rows, global `Time_Frames`, static cloud H-scroll, simple `loc_12110`, or broad HUD phase/position tuning.
- Verification: focused cache-backed `cargo test -q -p wildside-core issue289_level07_settled_recording_primes_island_shore_vram_loc_1202a -- --nocapture`, refreshed `record-scenario --slug issue289-level07-idle-current`, and expected-red `verify-recording --slug issue289-level07-idle-current`; final gates recorded in the commit session.

## 2026-06-23 handoff (#289 Island lower-shore settle)

- **#289 stays `[~]`; the Level 0x07 bottom-left Island shore residual is gone.** Oracle VDP samples showed `loc_12188`'s lower shore band had reached `shore_2.bin` / frame `1` while the already fixed primary/mid bands were correct, so the recording settle shim now advances only the lower-band state after the unified `loc_1202A` tail.
- Regression coverage pins `Time_Frames` unchanged at 174/185 while `island_shore_vram_frames_loc_1202a` becomes `(3,2,1)` at frame 60 and `(0,3,1)` at frame 71. The oracle sampler also now covers the bottom-left shore and top-right HUD residual coordinates.
- Refreshed `issue289-level07-idle-current` remains expected-red but trace/state-green: frame 60 improves from `133` / MAE `0.11480655` to `45` / `0.04169922`; frame 71 improves from `171` / `0.19314545` to `83` / `0.12003813`. Remaining residual is the top-right HUD/diamond strip and frame-71 timer/top specks.
- A HUD-diamond phase shim was rejected: it only improved to `39`/`77` pixels and did not close the known top-right HUD residual class. Next #289 pass should inspect HUD/timer sprite or text evidence, not Island shore age, selected Plane-B rows, global `Time_Frames`, static cloud H-scroll, simple `loc_12110`, or broad HUD phase/position tuning.
- Verification: `cargo fmt --all`, focused cache-backed `cargo test -q -p wildside-core issue289_level07_settled_recording_primes_island_shore_vram_loc_1202a -- --nocapture`, refreshed `record-scenario --slug issue289-level07-idle-current`, and expected-red `verify-recording --slug issue289-level07-idle-current`; final gates recorded in the commit session.

## 2026-06-23 handoff (#289 Lion projectile local palette/clock)

- **#289 stays `[~]`; the Level 0x07 Lion projectile residual is gone from the oracle pixel diff.** Re-reading `Lion.asm loc_3E12A` confirmed the ROM sets hardware `palette_line(a3)=2`, but native-player stores the extracted Lion enemy palette as local slots, so the sprite tiles must index local palette line 0. Re-reading `Lion.asm loc_3E180/loc_3E192` and `ingame/anim/enemy/Lion.asm` confirmed `stru_3DF34` keeps ticking through the hibernated projectile worker.
- Native-player now renders the projectile through the packed Lion palette slots, and core ticks the Lion projectile animation clock once per `loc_3E192` hibernated step. Regression coverage pins the rendered local slot colour and the ticking `animation_elapsed_frames`.
- Refreshed `issue289-level07-idle-current` is still expected-red on pixels but is trace/state-green: frame 60 improves from `236` / MAE `0.2491397` to `133` / `0.11480655`; frame 71 improves from `269` / `0.28263578` to `171` / `0.19314545`. Component inspection shows frame 60 now has only the bottom-left water/edge strip (88 px) and right/top background strip (45 px); frame 71 has those plus timer/HUD/top-right specks. No Lion body or projectile component remains.
- Next #289 pass should continue from the residual background/HUD pixels, not Lion projectile palette/phase, Lion body frame selection, gameplay state, selected Plane-B row words, global `Time_Frames`, static cloud H-scroll, Island shore age, or simple `loc_12110` current-phase rendering.
- Verification: refreshed `record-scenario --slug issue289-level07-idle-current`, focused Lion projectile CLI/core tests, expected-red `verify-recording --slug issue289-level07-idle-current`, component diff inspection, `cargo fmt --all --check`, `cargo check -q -p wildside-cli`, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast`.

## 2026-06-22 handoff (#289 Lion resumed-walk phase)

- **#289 stays `[~]`; the Level 0x07 Lion body phase now matches the oracle checkpoints.** After `Lion.asm loc_3E0F4` restores `$62(a5)` and resumes `loc_3E11E`, native exposes a Lion live walk phase through `NativeObjectSnapshot.walk_phase`; native-player uses that phase with the ROM `stru_3DEDA`/`stru_3DEF4` durations instead of the old global-frame approximation.
- The verified HP-6 resumed phase is 29: checkpoint frame 60 displays previous object phase 13 (C7E98 / tile base `$1B4`), and frame 71 displays phase 24 (C7EA0 / `$1D8`). Regressions cover both the core `loc_3E0F4` phase exposure and renderer phase selection.
- Fresh expected-red `issue289-level07-idle-current` remains state-clean/pixel-red but improves frame 60 from `1289` / MAE `1.4096261` to `236` / `0.2491397`; frame 71 stays `269` / `0.28263578`. The remaining residual is small projectile/timer/top-background pixels, not Lion body frame selection.
- Verification: focused Lion core/CLI tests, expected-red `verify-recording --slug issue289-level07-idle-current`, visual diff inspection, and `mise run verify-slice-fast`.

## 2026-06-22 handoff (#335 natural Hills entry fixed candidate)

- **#335 moved `[~]`→`[?]`.** Natural Isle 0x07→Hills 0x08 level-complete title-card handoff now refreshes the murder-wall runtime at `loc_1AE2E -> loc_6E2 -> Load_InGame` for murder-wall levels, instead of carrying an aged hidden title-card wall/death state into live gameplay.
- The refresh preserves the carried `Current_Helmet_ID`, so Red Stealth stays Red Stealth; it does not reuse the death-respawn helmet strip. The old debug `loc_74E0` regression remains green.
- New regression `natural_level_complete_hills_handoff_refreshes_murder_wall_loc_1ae2e_11fc8` covers a generated-assets Red Stealth 0x07 flag completion, waits on the Hills title card, presses Start, and asserts no life loss/death windup/form strip through the first 60 live frames.
- Verification: focused cache-backed #335 tests, `level_complete_title_card_start_edge_enters_next_level`, `mise run verify-slice-fast`, and `mise run verify-tas-full` (CRC all 37,759 frames match; state report remains baseline, first authoritative `transition_15_to_00_post`).
- **Next actionable open P0:** #257 continue-screen render, per the open queue.

## 2026-06-22 handoff (#271 Red Stealth RockTank route audit)

- **#271 moved `[~]`→`[b]`; no gameplay patch.** Fresh Red Stealth BLW2 natural and clean run+jump overlays all match oracle; observed HP-loss cases are grounded/side contact before/without a clean first-frame `$2C` stomp. Highwater natural Red Stealth walk never activates a 0x0B RockTank.
- The existing `loc_1CE4` / FA74 stomp gate remains the ASM-backed explanation for many "stomp hurts" cases. Re-patching Tank HP or stomp registration without a reproducing live route would be churn.
- **Next actionable open P0s:** #257 continue-screen render, then #335 natural 0x07→0x08 Hills instant-death, per `mise run project-status`.
- Verification: old sharp route CRC-clean; BLW2 base `verify-tas` frames 130..190 CRC-clean; BLW2 run+jump150 frames 130..190 CRC-clean; BLW2 run+jump126-long frames 120..200 CRC-clean; Highwater base frames 0..760 CRC-clean. Tracker gates pending in this commit: `mise run backlog-lint`, `git diff --check`.

## 2026-06-22 handoff (playtest triage — big batch)

- **Triaged a large human playtest (~21 observations) against the lanes; docs-only session, no code changed.** Per-issue evidence is in archive block "## 2026-06-22 playtest triage" (grep by `#NNN`). A 21-investigator read-only fan-out did the matching; the synthesis/edits were applied centrally to keep the lanes lint-clean.
- **9 fix-candidates FAILED human validation and were REOPENED `[?]`→`[ ]`** (moved from `backlog-awaiting-human.md` to the open queue): #271 stomp-on-RockTank hurts (P0), #257 continue-screen wall-from-both-sides (P0), #335 Hills instant-death (P0, now on the NATURAL 0x07→0x08 entry, not just the debug skip the loc_74E0 fix covered), #252 tank sprite doubled + green bullets (P1, raised P2→P1), #259 RockTank floor-sink (P1, 5th reconfirm — close `[-]` or prove on a natural L2 route), #352 hidden jump-up block no anim (P1, renderer-only gap scene.rs:5057), #272 Cyclone transform voice (P1), #339 USM "washing" drone (P1), #265 EOL tally tick missing + rumble (P2).
- **2 human-CONFIRMED WORKING and promoted `[?]`→`[x]`** (moved to the archive resolved index): #343 Maniaxe axe now kills dragons, #346 spike→spike-wall chain. Final close is the human's.
- **3 NEW rows opened:** #353 invisible metal-block spikes (P1, kills with no visible hazard), #355 Cyclone spin form no anim (P2 render), #356 Skydragon Castle bg piece dropout (P2).
- **4 open rows refreshed with 2026-06-22 evidence:** #336 (Kid floats in trees), #324 (prize-block wood bleed, canonical forest route now confirmed broken), #280 (blue/green seam-bar + late/half clouds; island/0x07 Chunked-arm subcase), #345 (Lion now ABSENT after death+respawn on 0x07).
- **NOT-A-BUG / working-confirm:** O7 USM "stacked hovering enemy" = EmoRock hover tracking Kid's X (ASM loc_3621A), faithful — only the #333 render approx stays open; O21 USM3 0x06 "~back to normal" = working-confirm of landed #289 USM3 fixes (#289 stays `[~]`). O14 "Lion red squares" is already addressed by the concurrent commit `4a591855d` (#289 Lion-projectile sprite) — verify next playtest, no new row.
- **Process note:** a concurrent loop session was editing the lanes during triage and reverted in-progress edits; once the human stopped it (after it committed `4a591855d`), the triage was applied cleanly. A standalone `playtest-triage-2026-06-22.md` report written during the conflict was removed once folded into the lanes.
- **Verification:** `mise run backlog-lint` and `git diff --check` pass; no code changed.

## 2026-06-22 handoff (#289 Lion projectile sprite)

- **#289 stays `[~]`; Lion projectiles now render the ROM `stru_3DF34` sprite path instead of the generic red hostile-projectile dot.** ASM `Lion.asm loc_3E12A` sets the projectile child to palette line 2, VRAM tile `$2FB`, and animation `stru_3DF34`; `ingame/anim/enemy/Lion.asm` maps that script to C7E20/C7E28/C7E30 for six ticks each.
- Native-player now appends those three projectile mappings to the Lion (`$12`) visual family and renders `ActiveProjectileKind::LionProjectile` through the Lion sprite path with palette line 2. Regression `lion_projectile_uses_stru_3df34_sprite_frames` pins C7E28/tile base `$004` and palette line 2.
- Fresh expected-red `issue289-level07-idle-current` remains state-clean/pixel-red but improves from frame 60 `1297` / MAE `1.4228051` and frame 71 `277` / `0.2984375` to frame 60 `1289` / `1.4096261` and frame 71 `269` / `0.28263578`. Visual diff confirms the projectile is no longer a red dot; the remaining dominant frame-60 residual is the Lion body/sprite nuance plus tiny top/background pixels.
- Next #289 pass should inspect the remaining Lion body sprite/origin/palette nuance and small top/background residual, not Lion projectile rendering, gameplay state, B08/B17/B18/B19/B22/B23/B27/B29/B30 row words, global `Time_Frames`, static cloud H-scroll, Island shore age, or simple `loc_12110` current-phase rendering.
- Verification: `cargo fmt --all --check`, `cargo check -q -p wildside-cli`, focused `cargo test -q -p wildside-cli native_player::tests::lion_projectile_uses_stru_3df34_sprite_frames`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, visual inspection of frame 60/71 diffs, `git diff --check`, and `mise run verify-slice-fast`.

## 2026-06-22 handoff (#289 live Lion walk script)

- **#289 stays `[~]`; live Lion rendering now uses the ROM walk scripts instead of cycling into appended death frames.** ASM `Lion.asm loc_3DFD8/loc_3E0F4` restores `stru_3DEDA` for HP-6 Lions and `stru_3DEF4` for HP-8 Lions before the charge path, while the extracted sheet stores death frames after the six live C7E88..C7EB0 mappings. Native-player now keeps live object type `$12` inside those scripts.
- Fresh expected-red `issue289-level07-idle-current` remains state-clean/pixel-red but improves from frame 60 `1471` / MAE `1.626809` and frame 71 `1424` / `1.5797665` to frame 60 `1297` / `1.4228051` and frame 71 `277` / `0.2984375`. Enemy-state `tas-diff` frames 55..62 matches oracle, including Lion position, HP, facing, `shared_walker_state=10`, `wait_timer=0`, and `lion_state=ResumingCharge`.
- Next #289 pass should inspect the remaining frame-60 Lion sprite/origin/palette nuance plus the small top/background residual, not gameplay state, B08/B17/B18/B19/B22/B23/B27/B29/B30 row words, global `Time_Frames`, static cloud H-scroll, Island shore age, or simple `loc_12110` current-phase rendering.
- Verification: `cargo fmt --all --check`, `cargo check -q -p wildside-cli`, focused `cargo test -p wildside-cli live_lion_uses_rom_walk_script_not_death_frames -- --nocapture`, `cargo test -p wildside-cli walking_dragon_and_lion_death_snapshots_select_rom_frames -- --nocapture`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, `cargo run -q -p wildside-cli -- tas-diff --scenario .cache/recordings/issue289-level07-idle-current/scenario.json --include-advisory --frontier-kind enemy --frame-start 55 --frame-end 62 --show-object-roster-on-match`, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast`.

## 2026-06-22 handoff (#289 Island shore recording-settle tail)

- **#289 stays `[~]`; settled recording warps now prime the Island shore `loc_1202A` DMA object by 16 object-local ticks.** The oracle VDP probe showed the shore tile rows were ahead of native's stored VRAM phase even though `Time_Frames` already matched oracle, so the fix advances only `step_island_shore_cycle_loc_1202a` after the recording settle window. Non-Island levels no-op because the object is absent.
- Fresh expected-red `issue289-level07-idle-current` remains state-clean/pixel-red but improves from frame 60 `2479` / MAE `2.4002976` and frame 71 `2546` / `2.473256` to frame 60 `1471` / `1.626809` and frame 71 `1424` / `1.5797665`. Bracketed trials found 15/16/17 ticks equivalent at the sampled checkpoints; 18 regresses frame 71 to `1720` / `1.738565`.
- Next #289 pass should inspect the remaining lower shore/water row content and left-edge samples after the `loc_1202A` age fix, not global `Time_Frames`, static cloud H-scroll, Plane-B row words in the do-not-retry set, or simple `loc_12110` current-phase rendering.
- Verification: `cargo fmt --all --check`, `cargo check -q -p wildside-cli`, focused cache-backed `cargo test -p wildside-core issue289_level07_settled_recording_primes_island_shore_vram_loc_1202a -- --nocapture`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast`.

## 2026-06-22 handoff (#289 previous VDP H-scroll latch)

- **#289 stays `[~]`; native auto-scroll display rows now use the previous VDP H-scroll table phase.** Oracle VDP pixel samples showed frame 60 static y=10 rendered hB=`FF86` while current WRAM hbuf debug had already advanced to `FF85`; frame 71 `$00` rows showed the same one-frame display latch. Native applies Time_Frames-1 only to `$00`/`$08` display sampling, while camera-derived shore rows and `sub_30744`/`sub_307D8` row-history stay on their existing phases.
- Fresh expected-red `issue289-level07-idle-current` remains state-clean/pixel-red but improves from frame 60 `2856` / MAE `2.682994` and frame 71 `2823` / `2.684696` to frame 60 `2479` / `2.4002976` and frame 71 `2546` / `2.473256`. Visual diff confirms the static cloud/parallax residual is gone; remaining top diffs are the Island shore/water band.
- Next #289 pass should trace shore VRAM art/upload timing around ASM `loc_1202A` and middle band `$F6C0`, not static cloud display H-scroll, selected Plane-B row words, or the already-rejected simple `loc_12110` current-phase substitution.
- Verification: focused `cargo test -q -p wildside-cli chunked_plane_b_display_uses_horiz_scroll_data_loc_3050a --no-default-features -- --nocapture`, `cargo fmt --all --check`, `cargo check -q -p wildside-cli`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, visual inspection of frame-60 diff, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast`.

## 2026-06-22 handoff (#289 oracle VDP tile-row diagnostics)

- **#289 stays `[~]`; no renderer fix landed this pass.** The native sampled-row diagnostic from the prior pass was paired with oracle VDP pixel provenance: `oracle_background_vdp_pixel_samples_loc_c070` now records Plane A/B nametable words, scrolled source coordinates, decoded 8-pixel tile rows, raw row bytes, palette/index, and RGB for the current #289 strict-diff coordinates.
- Fresh expected-red `issue289-level07-idle-current` is unchanged: `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; frame 60 `2856` / MAE `2.682994`, frame 71 `2823` / `2.684696`. The new report proves selected Plane-B row words are no longer the target: frame 60 `(86,100)` native tile `$39` row `[10,10,10,10,11,11,10,10]`, while oracle Plane B tile `$7B9` row is `A,A,B,A,A,A,A,A`; frame 71 `(91,102)` native row `[10,10,10,10,10,10,10,11]`, while oracle row is all `A`; frame 60 `(284,10)` native static tile `$02` row `[11,11,11,11,11,11,11,13]`, while oracle Plane B tile `$782` row is `B,B,B,B,B,B,B,D` and samples index `B`.
- Next #289 pass should investigate Island VRAM art/source timing for static background tiles and shore `$F6C0`, not row-history placement, display scroll, or the already-rejected simple `loc_12110` current-phase substitution.
- Verification: `cargo fmt --all --check`, `cargo check -q -p wildside-cli`, `mise run audit-debug-fields`, focused `cargo test -q -p wildside-cli background_pixel_sampler_reports_last_drawn_tile_provenance_loc_c070 --no-default-features -- --nocapture`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast`.

## 2026-06-22 handoff (#289 sampled tile-row diagnostics)

- **#289 stays `[~]`; no renderer fix landed this pass.** The previous simple `loc_12110` current-phase trial remains rejected, so this session added narrow report instrumentation instead: `native_background_samples[].sample.source_row_pixels` now records the eight palette indices from the sampled tile row selected by native background provenance.
- Fresh expected-red `issue289-level07-idle-current` is unchanged: `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; frame 60 `2856` / MAE `2.682994`, frame 71 `2823` / `2.684696`. The report now shows residual art rows directly, e.g. frame 60 `(86,100)` tile `$39` row `[10,10,10,10,11,11,10,10]` at `source_x=2,source_y=4`, and frame 60 `(284,10)` tile `$02` row `[11,11,11,11,11,11,11,13]` at `source_x=7,source_y=2`.
- Native/oracle row dumps still have zero diffs for B08/B17/B18/B19/B22/B23/B27/B29/B30. Next #289 pass should investigate Island VRAM art/source timing (static background art and shore `$F6C0`) rather than row-history placement or simple middle-shore phase substitution.
- Verification: `cargo fmt --all --check`, `cargo test -q -p wildside-cli background_pixel_sampler_reports_last_drawn_tile_provenance_loc_c070 --no-default-features -- --nocapture`, `cargo check -q -p wildside-cli`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast`.

## 2026-06-21 handoff (#289 loc_3050A display H-scroll / Plane-B underlay)

- **#289 stays `[~]`; native chunked Plane-B display now uses the actual hardware H-scroll phase from ASM `BackgroundScroll_ComputeShiftData loc_30452` as consumed by `BackgroundScroll_ComputeShiftLayers loc_3050A`.** The decisive residual sample was frame 60 top-left: oracle debug had slot `$08` `cur=0x007B` and hbuf `FF85`, while native display sampling used the older 43px upload phase and selected B17 column 5 (`$2791`). Native now separates display H-scroll from the row-upload/preload phase, so row-history content stays intact while screen sampling matches hbuf.
- A second residual check showed row dumps and oracle agreed B17 column 18 was `$2780`, but native still showed direct chunk tile `$09` because the old direct source-chunk pass sat underneath the reconstructed VDP row. Rows managed by `sub_30744/sub_307D8` now skip the direct underlay; transparent `$2780` row-history cells reveal the scene background instead of stale source art.
- Fresh expected-red `issue289-level07-idle-current`: trace/state stay green, pixels remain red but improve from frame 60 `3915` / MAE `4.2537017` and frame 71 `4113` / `4.5239353` to frame 60 `2856` / `2.682994`, frame 71 `2823` / `2.684696`. The old top-left row-17 display-selection mismatch is gone; remaining samples are lower/mid-row colour swaps (notably y=100 tile 57) and right/top cloud residuals.
- Verification: `cargo test -q -p wildside-cli chunked_plane_b --no-default-features -- --nocapture`, `cargo fmt --check --all`, `cargo check -q -p wildside-cli`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, plus pending final loop gates recorded in NOW.

## 2026-06-21 handoff (#289 Load_InGame grid-preload Plane-B state)

- **#289 stays `[~]`; settled Plane-B row reconstruction now preserves the earlier `Load_InGame loc_11F48` camera-grid preload state.** Settle-trace evidence showed B08 cols 4..6 plus B27/B29 far-edge cells first appeared during `title_wait` step 57 with `Camera_X=0x0140`, `Camera_Y=0x02A0`, before the later target-live settled camera (`0x0124`, `0x0223`). ASM `Load_InGame loc_11F48` rounds `PlayerStart_X/Y` to `$140`/`$E0` screen-grid camera values before calling `loc_2FFE8`, and later `sub_307D8` passes leave cells outside their spans intact.
- Native now applies a first `loc_2FFE8` preload pass using the player-start grid camera, then the existing settled-camera pass. The focused regression keeps B29 column 45 as the old far-edge ocean cell from the grid pass. Fresh expected-red `issue289-level07-idle-current` is still pixel-red with unchanged metrics (`3915` / MAE `4.2537017`, `4113` / `4.5239353`), but native row dumps now have zero diffs against oracle for B08/B27/B29/B30 at both checkpoints. Remaining #289 work is no longer those row cells; inspect sampled residuals against adjacent rows/palette/display selection before another renderer change.
- Verification: focused `cargo test -q -p wildside-cli chunked_settled_row_history_seeds_loc_2ffe8_persisted_plane_b_row --no-default-features -- --nocapture`, `cargo fmt --check --all`, `cargo check -q -p wildside-cli`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast` (TAS frontier plus 6257 core tests and 9 CLI gameplay tests green).

---

## 2026-06-21 handoff (#289 Plane-B wrapped row source)

- **#289 stays `[~]`; native `sub_30744` live column writes now use the source row from the camera-visible row window, not the visible VDP row number.** Re-reading ASM `sub_30744` showed it starts at `Camera_Y_pos>>5` and walks 29 source rows while the VDP row wraps with `& $1F`; at Level 0x07 camera chunk 17, Plane-B row 8 is therefore fed by source row 40. Native previously wrote B08 columns 45..55 from source row 8 and overwrote the persisted preload strip with `$2780` zeros.
- Focused regression extended `chunked_settled_row_history_seeds_loc_2ffe8_persisted_plane_b_row` with B08/source-row-40 coverage. Fresh expected-red `issue289-level07-idle-current` keeps the same pixel metrics (`3915` / MAE `4.2537017`, `4113` / `4.5239353`), but native B08 now matches the oracle row words at columns 45..55/56. Remaining #289 work is no longer this B08 source-row issue; continue from the remaining top-row/backdrop/far-edge visual residual with row evidence.
- Verification: `cargo test -q -p wildside-cli chunked_settled_row_history_seeds_loc_2ffe8_persisted_plane_b_row -- --nocapture`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, `cargo fmt --check --all`, `cargo check -q -p wildside-cli`, `cargo run -q -p wildside-cli -- backlog-lint`, `git diff --check`, and `mise run verify-slice-fast`.

---

## 2026-06-21 handoff (#289 Plane-B H-flip table)

- **#289 stays `[~]`; chunked Plane-B row words now mirror ASM `unk_309B8` for high-bit chunk bytes.** Re-reading `sub_30744`/`sub_307D8` showed both routines load words from `unk_309B8`; the table is `$2780..$27FF,$2F80..$2FFF`, so chunk byte bit 7 is the VDP H-flip bit. Native previously treated bit 7 as tile-index bit 7 and emitted `$2792/$2791` where the oracle row dump has `$2F92/$2F91`.
- New red-first tests cover both levels: `background_tile_uploads_sub_30744_maps_chunk_high_bit_to_hflip_loc_309b8` for the shared data helper and the #289 `chunked_settled_row_history_seeds_loc_2ffe8_persisted_plane_b_row` B23 tail. Fresh expected-red `issue289-level07-idle-current` keeps the same pixel metrics (`3915` / MAE `4.2537017`, `4113` / `4.5239353`), but native B23/B27 row words now match oracle flip bits. Remaining visible work is B08/far-edge row-state, not this H-flip table.
- Verification: `cargo test -q -p wildside-data background_tile_uploads_sub_30744 --lib`, `cargo test -q -p wildside-cli chunked_ --no-default-features`, `cargo check -q -p wildside-data`, `cargo check -q -p wildside-cli`, `cargo fmt --check --all`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, backlog lint, and `git diff --check`.

---

## 2026-06-21 handoff (#289 parallax preload camera phase)

- **#289 stays `[~]`; native-player now splits settled `loc_2FFE8` preload Camera_X by bgscroll slot.** `Load_InGame loc_11F48`'s player-start screen-grid Camera_X remains correct for Time_Frames-driven `$00/$08` rows (B18/B19 still keep old `$0780` columns 0-4, `$2780` columns 5-48, and the right-edge cloud tails), but camera-parallax rows match the oracle when they use the settled trace Camera_X. The focused regression now pins both the B18 cloud row and B29 ocean row.
- Fresh expected-red `issue289-level07-idle-current` improves again: frame 60 `4031` / MAE `4.3676014` → `3915` / `4.2537017`, frame 71 `4181` / `4.603767` → `4113` / `4.5239353`; state remains clean and pixels remain red. B29 now matches the oracle ocean strip through the main visible body, while residual row gaps remain at far-edge cells and flip bits (notably B23/B27/B29/B30), plus the broader top-row/backdrop residual. **NEXT:** read/compare `BackgroundScroll_ComputeShiftData` `sub_304B4` and `sub_30744` for the remaining single-cell far-edge writes and H-flip preservation before changing render code again.
- Verification: `cargo fmt --check --all`, `cargo test -q -p wildside-cli chunked_ --no-default-features`, `cargo check -q -p wildside-cli`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, backlog lint, and `git diff --check`. `mise` wrappers remain blocked here by `fnox@1` linux-arm64, so direct cargo/CLI commands were used.

---

## 2026-06-21 handoff (#346 launched-spike-bullet lifetime cap)

- **#346 moved `[ ]`→`[?]` (awaiting human spike-chain playtest).** Launched spike bullets (ShooterObject) were force-deleted at the invented native `lifetime:120` (~480px) before they could reach a distant evanescent wall and re-fire the return-spike chain. ROM truth re-verified by reading `Process_ShooterObjects` (kid.asm:19298) and the per-object loop `loc_EDFA` (kid.asm:19311) end to end: it dispatches by direction, checks the level bound, and increments a state word for the collision cadence — there is NO countdown/DBF and no timer deletion; the object dies only via `ShooterObject_InteractLevelLayout` (collidable cell or level edge). The native dedicated port `step_tracked_shooter_object_loc_EDFA` already bounds the bullet at the edge/cell in all four directions, so it can't leak. Fix: exempted `EnemyProjectileKind::ShooterBlock` from the generic `step_projectiles` lifetime-zero-delete and lifetime-decrement guards (matching the existing `is_lion_projectile`/`is_juggernaut_skull` ASM-no-timer exemptions); `shooter_block_loc_EC76`'s `lifetime:120` is now inert for these objects. Regression `shooter_object_survives_past_invented_lifetime_to_reach_distant_wall_loc_edfa` (red on the old cap). CRC-neutral: `verify-tas-full` byte-identical (first_crc_mismatch null, 212 authoritative = #290) — the published route's shooter bullets already hit a wall within 120f, so removing the cap changes nothing there. `verify-slice-fast` green (6257+9). **UNVERIFIED (human):** a real long-range spike→evanescent-wall chain firing in live play.

## 2026-06-21 handoff (#343 friendly-projectile-vs-large-enemy hitbox)

- **#343 moved `[~]`→`[?]` (awaiting human combat playtest).** Axes/friendly projectiles passed through the walking dragon and other large enemies. A fan-out workflow established three things at high confidence (each independently re-read): (1) ROM truth — the friendly-attack-vs-enemy pass is `sub_1D76`→`sub_1D84` (kid.asm:3537-3673, called at kid.asm:947), a SEPARATE routine from the Kid-body `GfxObjects_CollisionKid`; it builds each enemy box from per-family `CollisionSize_Index[$22>>1]` and writes `colid_hurt`=$1C to the enemy on an inclusive AABB overlap (kid.asm:3650); DragonWalking (0x0D) box `unk_31630` = (-16,16,-32,0). (2) Native — `process_friendly_projectile_hits` (helmets.rs) used an invented fixed `±4×±8` enemy box for ALL families via a centre-distance test, so the 32×32 dragon body sat outside it; the per-family `enemy_contact_bounds` dispatcher (the #308-verified boxes) was already consumed by the Kid-contact and Diamond-Power paths but NOT here. (3) CRC gate — `authoritative_crc32` excludes enemy/projectile state, so the change is only indirectly CRC-relevant (flipped kill → Kid knockback → camera); `verify-tas-full` confirmed CRC-neutral (all 37759 frames match, 212 authoritative = #290 baseline), the green branch that lands directly. Fix: new helper `friendly_projectile_overlaps_enemy_loc_1d84` sources the enemy box from `enemy_contact_bounds(target)` (with the empty-box guard) and tests the inclusive AABB mirroring combat.rs:13343, rewiring both overlap closures (axe/beam + diamond-rain). Projectile box left as the centred approximation (ROM per-frame projectile CollisionSize = deferred #308 P3). Regression `friendly_projectile_hits_walking_dragon_body_collision_size_unk_31630_loc_1d84`; 5 fixed-box-pinning tests re-derived against the flying-dragon (0x0C) box, incl. the Tank-turret test now targeting the turret child's real offset position (the old setup only worked via the fudged box). `verify-slice-fast` green (6256+9). **FOLLOW-UP:** RedStealth sword (helmets.rs:6369) + berzerker charge (6532) still use the fixed box — same #308 vein. **UNVERIFIED (human):** confirm an axe visibly connects with the walking dragon / lion / orca / tank in live play.

## 2026-06-21 handoff (#349 post-death title-card name glyphs render)

- **#349 moved `[~]`→`[?]` (awaiting human post-death playtest).** The post-death level card showed the scene/trees but dropped the animating level NAME. Reproduced with the native-player RENDER assertion the row asked for: built a glyph-worker trace (`Enemy{object_type:0}` + `glyph` debug field) and rendered it in both modes — `build_transition_scene` (LevelTransition) DOES draw the name, but `build_game_scene_filtered` (InGame) does NOT, and the death-respawn card is traced as InGame during the `respawn_live_handoff` hand-off (trace.rs:9177). Two compounding InGame defects: the enemy loop passed the `object_type:0` glyph workers to `push_enemy_sprite` as enemy family 0, and the glyph overlay (`build_title_card_worker_overlay_scene`) was only ever called from the transition scene. Fix: the gameplay scene now `continue`s past glyph workers in the enemy loop and appends the title-card overlay before the warp tint (no-op outside the title-card window). Engine-level (keys on the `glyph` field). Test `death_respawn_title_card_renders_level_name_glyphs_loc_b8de` asserts BOTH paths draw the name. Render-only/CRC-neutral; `verify-slice-fast` (6251+9) green. **UNVERIFIED (human):** a real post-death playtest confirming the name animates over the respawn card.

## 2026-06-21 handoff (#339 Rock Tank drive drone death/skip stop — REOPEN resolved)

- **#339 moved `[ ]`→`[?]` (awaiting human audio listen).** The drive drone ($1F) persisted after a Kid death and re-accumulated on every level-skip. The per-exit refcount stop edge (`stop_rock_tank_drive_sound_on_last_exit_loc_3c46e` in `refresh_active_roster`) is bypassed by the death-respawn / level-skip bulk clears (`reset_current_level_runtime_state_for_respawn` clears `active_enemies`+`floating_items`; the `level_skip_resume_hold` branch clears `active_enemies`) — so the drone's StopSequence never fired across a death or skip. ROM's only off-switch on those paths is `Load_InGame`→`j_StopMusic` (loc_23629), a global GEMS reset at level (re)entry. Mirrored it with a new helper `stop_level_looping_drones_on_load_ingame_loc_23629` (enemies.rs) that queues `StopSequence(TankDriving)`/`StopSequence(ClockPrize)` for any drone still live, called at the top of the respawn reset and in the skip teardown. This also resolves #342's death/level-load SCOPE BOUNDARY (a clock falling at the bulk-clear frame) — one mirror stops both drones uniformly.
- **Fidelity:** native models the drones as discrete Start/StopSequence pairs (the original #339 simplification of the ROM refcount + global j_StopMusic), so per-sequence stops at the teardown are the consistent native expression. The drone faithfully keeps playing through the visible death tumble (tanks stay alive) and silences at the respawn Load_InGame, matching ROM. Tests `loc_23629_*` (helper + skip wiring in runtime_tests.rs, respawn wiring in level_tests.rs). `cargo fmt` clean, core check warning-free, `verify-slice-fast` (6251+9) green, `verify-tas-full` CRC all 37759 frames match (`first_crc_mismatch: None`), 212 authoritative = #290 baseline (audio CRC-neutral). **UNVERIFIED (human, #264 caveat):** whether `StopSequence($1F/$0E)` audibly halts vs restarts the GEMS loop needs a real audio device.

## 2026-06-21 handoff (#350 Hills below-canopy foliage palette fixed)

- **#350 moved `[~]`→`[?]` (awaiting human Hills playtest).** Hills of the Warrior (theme 3, level 0x08) drew the lower foliage trees/bushes blue/purple. Probed the oracle first: a settled `warp_to_level:8` idle render at frame 39 vs `capture-oracle-scenario-frames` proved the top-of-level purple distant mountains are FAITHFUL (present in the oracle), but a 3× crop of the central below-canopy region showed native drawing foliage in light-blue/purple where the oracle draws green/brown. **Corrected the backlog's original diagnosis:** it blamed a missing theme-3 branch in the CHUNKED `background_palette_for_scene`, but Hills routes through the LAYERED background path. The real mechanism: ASM `Load_BackgroundMappings_Layered loc_1295E` (kid.asm:25305) decodes Hill planes 0/1 at EniDec base `$2780` (CRAM line 1) and the lower foliage plane 2 at `$07C0` (CRAM line 0) — identical palette-line usage to Mountain `loc_12910`. The bundle bakes the per-plane palette lines, so plane-2 tiles render at native palette_line 0, but `background_palette_for_scene` had a two-line branch only for Mountain (7) and Forest (9) — so Hills' line-0 foliage indexed the background SKY palette (blue) instead of the foreground CRAM line.
- **Fix:** extended the Mountain branch guard `== 7` → `matches!(.., 7 | 3)` (the CRAM copy loc_11E16 is theme-independent and Hills shares Mountain's two-line layout, so the construction is identical: line 0 = foreground, line 1 = background colours 1..7). Engine-level (theme-keyed, no per-level override). **Verification:** warp idle frame-39 below-y128 native↔oracle diffs 1334→0 (only the separate #280/#289 sky-seam diffs remain). New regression `hills_background_palette_reads_foreground_foliage_plane_loc_1295e` (mirrors the Mountain/Forest palette tests). `cargo fmt` clean, both crates check warning-free, `verify-slice-fast` green (6251 core + 9 CLI + frontier + backlog-lint). Render-only/CRC-neutral (CLI presentation; no wildside-core change), so `verify-tas-full` not required. **UNVERIFIED (human):** real Hills playtest confirming green foliage below the canopy.

## 2026-06-21 handoff (#342 clock-prize loop start/stop ported)

- **#342 moved `[~]`→`[?]` (awaiting human audio listen).** The looping `sfx_Clock_prize` ($0E) leaked forever: native started it at COLLECTION (`apply_prize_content`) with no stop, while ROM `PrizeClock_Init loc_10792` (kid.asm:21996) starts it when the FALLING clock object SPAWNS (only the first active clock, `tst.b $FFFFFB55`) and `loc_107E0` stops it via `j_PlaySound2` ($12 cmdstopseq) when the last clock is deleted. Native already models the falling clock as a `floating_items` entry, so — mirroring #339's RockTank drone — the `$FFFFFB55` refcount is DERIVED from the live clock count (`active_clock_prize_count`), immune to bulk `floating_items.clear()` desync. `finish_deferred_ceiling_clears` (only production Clock spawn) fires `start_clock_prize_sound_on_first_spawn_loc_10792` on the `0→first` edge; `step_visual_effects` fires `stop_clock_prize_sound_on_last_exit_loc_107e0` on the `>0→0` edge (covers both the collision_latched and fall-gate retains). Removed the wrong collection-start AND the redundant native_player timer-jump heuristic (a collection-frame start would re-leak after the core's stop). The per-object pure helpers stay the documented ASM spec (batch retain can't use a single decrement — same as #339). SCOPE: a clock still falling at the exact death/level-load `floating_items.clear()` frame is the #339-REOPENED global-StopMusic class, intentionally not chased here.
- Regressions (runtime_tests.rs): `loc_107e0_last_clock_prize_exit_emits_stopsequence_clock_prize` (live `step_visual_effects` deletion → one StopSequence), `loc_10792_first_clock_prize_spawn_emits_startsequence_clock_prize`, plus the 2nd-clock-holds / last-clock-holds / silent-when-none cases; updated `prize_content_clock_adds_time` and the two native_player timer-jump dedup tests. **UNVERIFIED (human, same as #339)**: whether `StopSequence($0E)` audibly HALTS vs restarts the GEMS loop needs a real audio device.
- Verification: `cargo fmt` clean, both crates `cargo check` warning-free, focused clock tests (10 core + 2 CLI) green, `mise run verify-slice-fast` green (6251 core + 9 CLI gameplay regressions + verify-tas-frontier + backlog-lint), `mise run verify-tas-full` CRC all 37759 frames match (`first_crc_mismatch: None`), 212 authoritative divergences = #290 baseline (sound advisory/CRC-neutral). `mise` wrappers worked here (the prior session's `fnox@1` linux-arm64 block did not recur). #289 left `[~]` and assessed research-blocked (its latest persisted-replay state + dead-ends preserved under archive #289).
- **Also bisected #336 (stays `[ ]`, no code change — oracle gate unmet).** The cold-boot "Kid floats to treetops then drops" is the `LEVEL_ONE_LOAD_INGAME_CAMERA_RAMP_Y` camera pan (216→115px over 26 frames vs Kid world-Y 255→271 over 13), introduced by commit c65df2500 (2026-06-12, "Align title-card Load_InGame live carry") which added the ramp + the `title_card_live_handoff_camera_state_loc_1ae2e` path — recent, matching the human's "recent regression" and correcting the archive's stale "not a recent regression" verdict (that looked at spawn-Y, not the camera ramp). On-screen Kid moves treetops(39px)→floor(156px); the ramp is applied ONLY for level 0 (asymmetry smell). DID NOT edit the ramp: c65df2500 was an oracle-alignment commit so the values may be faithful, and `compare-native-oracle --scenario boot-title-start-level1` (oracle is set up) only checkpoints to the title card (frame 719, both PNGs are the "BLUE LAKE WOODS I" name card with Kid not yet live) — the live float is uncaptured. Per oracle-is-truth/never-guess, the next session must capture an oracle cold-boot run PAST the title card into live gameplay and diff per-frame camera/kid-Y before touching the ramp. Full evidence under archive #336. **NEXT**: run `project-status`; #336's fix is gated on that oracle capture; otherwise prefer ready items over grinding #289.

---

## 2026-06-21 handoff (#289 persisted Plane-B preload replay)

- **#289 stays `[~]`; native-player now seeds settled level-skip chunked Plane-B rows by replaying persisted `loc_2FFE8`/`sub_307D8` preload segments.** The model feeds the typed alias-aware source bytes through `ChunkedPlaneBRowHistoryLoc30744`, uses `Load_InGame loc_11F48`'s player-start X screen grid plus settled trace camera Y, then applies visible `sub_30744` writes from the start of `Time_Frames`.
- B18 now matches the oracle row-history shape: old `$0780` columns 0-4, `$2780` columns 5-48, and cloud tail `2781..2787` at columns 49-55 by frame 60. Fresh expected-red `issue289-level07-idle-current` improves from frame 60 `5080` / MAE `5.6536646` and frame 71 `5212` / `5.8645835` to frame 60 `4031` / `4.3676014` and frame 71 `4181` / `4.603767`, but remains pixel-red. **NEXT**: inspect the remaining persisted Plane-B row/state mismatches on B08/B17/B22/B23/B27/B29 rather than retrying B18 preload placement, display-scroll, or alias-only variants.
- Verification: `cargo fmt --check --all`, `cargo test -q -p wildside-cli chunked_ --no-default-features`, `cargo check -q -p wildside-cli`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, CLI gameplay regressions, TAS frontier, and backlog lint. `cargo test -q -p wildside-core --lib` still fails on unrelated pre-existing chunk_08 snapshot/demo bootstrap expectations. `mise` wrappers remain blocked by `fnox@1` linux-arm64.

---

## 2026-06-21 handoff (#289 oracle Plane-B span diagnostic)

- **#289 stays `[~]`; oracle hidden-settle rows now include compact Plane-B span summaries.** `background_plane_b_spans_loc_307d8` sits beside the full `background_plane_b_rows_loc_307d8` dump and compresses selected rows into non-baseline VDP spans, treating `$0780`/`$0094` as old/blank.
- Regenerated `oracle-probe --capture-settle-trace` for `issue289-level07-idle-current` now shows the row-history shape without custom CSV parsing: B18 appears as `05..2D:2780` when first streamed, later `05..30:2780|31:2781|32:2782|33:2783` by the late target-live samples. This reinforces the current lead: native’s stateless B18/B19 upload starts at column 0, while oracle’s persisted row starts at column 5 and then accumulates `sub_30744` tail writes. **NEXT**: consume/replay persisted row writes through the typed `loc_2FFE8`/`sub_307D8` helpers; do not retry packed timed full-row placement.
- Verification: `cargo fmt --check --all`, `cargo test -q -p wildside-cli oracle_manifest_deserialises_hidden_settle_trace_rows`, and regenerated `cargo run -q -p wildside-cli -- oracle-probe --scenario .cache/recordings/issue289-level07-idle-current/scenario.json --frame-start 0 --frame-end 0 --capture-settle-trace`. `mise` wrappers remain blocked by `fnox@1` linux-arm64.

---

## 2026-06-21 handoff (#289 rejected packed upload placement)

- **#289 stays `[~]`; no code retained from the packed upload-placement trial.** A scoped change used ASM `BackgroundScroll_ComputeShiftData loc_30452` / `sub_307D8`'s packed word for timed `$00`/`$08` full-row upload placement instead of the display-derived `loc_3050A` placement.
- The trial moved real `$08` rows from `upload=d00<-s01` to `upload=d04<-s05`, but it did not improve `issue289-level07-idle-current` (`5080` / MAE `5.6536646`, `5212` / `5.8645835`) and broke `chunked_timed_plane_b_window_uses_preload_time_loc_2ffe8_307d8` by refreshing VDP column 45 too early (`Some((0,1))` instead of the expected old `$0780`). Reverted. **NEXT**: trace/model persisted Plane-B nametable/source history across `Load_InGame`, `loc_2FFE8`, `sub_30744`, and `sub_307D8`; do not retry blanket packed timed-row full-upload placement.
- Verification: `cargo fmt --check --all`, focused failing `cargo test -q -p wildside-cli chunked_timed_plane_b_window_uses_preload_time_loc_2ffe8_307d8` for the rejected trial, and unchanged expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`.

---

## 2026-06-21 handoff (#289 sub_307D8 upload-word diagnostic)

- **#289 stays `[~]`; native Plane-B row provenance now prints the packed `sub_307D8` upload placement.** The report keeps the current display-derived `upload=dXX<-sYY` field, and adds `upload_data=dXX<-sYYwWWWW` from ASM `BackgroundScroll_ComputeShiftData loc_30452` / `sub_307D8`.
- Fresh expected-red #289 report remains unchanged at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`, but now exposes the next lead: real `$08` rows show `upload=d00<-s01` while packed upload data says `upload_data=d04<-s05w0024`, matching the oracle left-edge boundary where columns 0-4 remain old `$0780`. **NEXT**: test a scoped row-history change that uses packed `sub_307D8` upload placement for timed `$00/$08` rows, without reintroducing the rejected broad display-scroll placement trial.
- Verification: `cargo fmt --check --all`, focused `cargo test -q -p wildside-cli chunked_settled_row_history_replays_loc_30452_columns_into_alias_source_loc_30744`, `cargo check -q -p wildside-cli`, and expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`.

---

## 2026-06-21 handoff (#289 loc_2FFE8 preload model)

- **#289 stays `[~]`; data now has a pure `background_preload_row_uploads_loc_2FFE8` model for the ROM preload sweep.** ASM `loc_2FFE8` sweeps `Camera_Y_pos` below zero before restoring the real camera and calling into gameplay; the model replays the second preload loop through shared `background_row_uploads_sub_307D8`, including the raw `$FFFFFAA8 = Camera_Y_pos-$3A0` seed before per-pass row-chunk latching.
- Regressions cover both the direct B18-shaped negative-y `sub_307D8` case (`Camera_Y_pos=-317`, previous chunk `-11`, streamed row 18, selector `$08`, source column 6, VDP word `$690A`) and the full `loc_2FFE8` sweep from `Camera_Y_pos=547`. The expected 41-cell strip maps to `$2781..$27A9`. This is model/test groundwork only; native rendering is unchanged until persisted Plane-B nametable state consumes it. **NEXT**: build the persisted Plane-B/source-history model on this typed helper instead of another renderer-local stateless preload.
- Verification: `cargo fmt --check --all`, backlog lint, focused `cargo test -q -p wildside-data background_row_uploads_sub_307d8_accepts_loc_2ffe8_negative_preload_camera_y`, focused `cargo test -q -p wildside-data background_preload_row_uploads_loc_2ffe8_sweeps_negative_rows_before_restore`, `cargo check -q -p wildside-data`, and `cargo test -q -p wildside-data --lib`.

---

## 2026-06-21 handoff (#289 Plane-B provenance diagnostic)

- **#289 stays `[~]`; recording reports now include native Plane-B row provenance, not just row words.** Each sampled row records the bgscroll selector, `Time_Frames`, synthetic upload destination/source, changed-column source (`direct`/`alias`), each elapsed `loc_30452`/`sub_30744` write, and final VDP words. The expected-red baseline remains frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`.
- The retained diagnostic proves the current native model starts B18/B19 from synthetic `upload=d00<-s01`, then correctly writes the alias-source right-edge tails. A temporary `loc_2FFE8` vertical preload replay using display-scroll placement fixed B18/B19 raw rows exactly, but B08/B17/B22/B23/B27 stayed wrong and pixels worsened to frame 60 `6069` / `6.235133`, frame 71 `6220` / `6.4576964`; reverted. **NEXT**: model persisted Plane-B nametable/source history, not another stateless all-row preload.
- Verification: `cargo fmt --check --all`, `cargo check -q -p wildside-cli`, `cargo test -q -p wildside-cli chunked_`, expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, backlog lint, TAS frontier, wildside-core lib sweep, and wildside-cli gameplay regressions.

---

## 2026-06-21 handoff (#289 rejected persisted preload shortcut)

- **#289 stays `[~]`; no code retained.** A trial `loc_2FFE8`-shaped persisted preload sweep selected the last source row for each Plane-B row, used `sub_307D8`'s extra `Horiz_Scroll_Data >> 3` shift, and replayed current `sub_30744` column writes only for settled `LevelSkip_Cheat` traces.
- The trial worsened `issue289-level07-idle-current` from the current baseline frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835` to frame 60 `6762` / `6.6236467`, frame 71 `6987` / `6.8926945`. It shifted B17/B27/B29/B30 through the wrong row/source slot, so the code was reverted. **NEXT**: trace or type the ROM `BackgroundScroll_Index`/MapOrder mapping used by `sub_30744`/`sub_307D8` before replaying persisted Plane-B nametable state; do not map Plane-B row number directly to bgscroll speed again.
- Verification for the rejected trial: `cargo fmt`, `cargo check -q -p wildside-cli`, `cargo test -q -p wildside-cli chunked_`, and expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`.

---

## 2026-06-21 handoff (#289 native Plane-B row diagnostic)

- **#289 stays `[~]`; recording reports now include native Plane-B row words from `ChunkedPlaneBRowHistoryLoc30744`.** Fresh `issue289-level07-idle-current` remains expected-red and unchanged at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`, but native rows can now be compared directly to `oracle_background_plane_b_rows_loc_307d8`.
- The row comparison rejects another display-scroll patch: native B18/B19 right-edge cloud tails match oracle (`2781..2787`/`2789..278F`, with frame 71 adding `2788`/`2790`), but native also refreshes B18/B19 columns 0-4 to `$2780` where oracle keeps `$0780`, and native B17 incorrectly contains cloud words at columns 17-24 while oracle only has the columns 5-6 tail. **NEXT**: model or trace persisted Plane-B nametable state across target-level `Load_InGame`, `loc_2FFE8`, `sub_30744`, and `sub_307D8`; do not retry raw `loc_30452` display scroll, alias-only rendering, or alias-fed full row uploads.
- Verification: `cargo fmt --check --all`, `cargo check -q -p wildside-cli`, `cargo test -q -p wildside-cli chunked_`, fresh expected-red `cargo run -q -p wildside-cli -- verify-recording --slug issue289-level07-idle-current`, direct `verify-slice-fast` body (TAS frontier, backlog lint, wildside-core lib sweep, wildside-cli gameplay regressions). `mise` wrappers remain blocked by `fnox@1` on linux-arm64 here, so direct cargo/CLI commands were used.

---

## 2026-06-21 handoff (#289 expanded Plane-B row diagnostic)

- **#289 stays `[~]`; oracle Plane-B row diagnostics now include B17, B19, B22, and B23.** Fresh `issue289-level07-idle-current` remains expected-red at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`, but the row dump now shows B17 is mostly `$2780` except columns 5-6 while B18/B19 hold the right-edge cloud tails at columns 49-55.
- Re-tried the raw `loc_30452` display-scroll basis on current code and rejected it again: it moved right-edge samples onto B18/B19 but exposed wrong native B17 cloud content at the left edge, worsening frame 60 to `5336` / MAE `5.8150578`. **NEXT**: compare native row-history words for B17/B18/B19 against the expanded oracle dump before another display-scroll patch.

---

## 2026-06-21 handoff (#289 loc_30452 Plane-B replay)

- **#289 stays `[~]`; native-player elapsed chunked Plane-B row replay now uses `BackgroundScroll_ComputeShiftData loc_30452` words.** The replay no longer derives `sub_30744` changed columns from display `loc_3050A` pixels, and settled level-skip traces can include the 60 hidden target-live frames before scenario frame 0 against the `loc_12824` alias source for one-column writes only.
- A synthetic Level 0x07-shaped regression proves the hidden `$08` row replay writes alias-source columns `49..55` into B18 by `Time_Frames=174`. The real `issue289-level07-idle-current` recording remains pixel-neutral at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`; alias-fed full row uploads were rejected because they worsened to `6359`/`6437` changed pixels. Direct fast-slice verification passed. **NEXT**: inspect live native row words/provenance for the remaining sampled pixels before another render change.

---

## 2026-06-21 handoff (#289 Plane-B row-history primitive)

- **#289 stays `[~]`; native-player chunked Plane-B reconstruction now runs through `ChunkedPlaneBRowHistoryLoc30744`.** This keeps the current direct `loc_12830` source path and produces no intended render change, but separates old `$0780` row state, the `sub_307D8` row window, and visible `sub_30744` one-column writes behind an explicit row-history object.
- Rechecked the rejected horizontal hidden-window replay against ASM `sub_304B4`/`sub_30744`: Level 0x07 B18 is still a `$08` row and the extra 60 hidden recording frames only cross VDP columns 42..45, so it cannot create oracle columns 49..56. Fresh `issue289-level07-idle-current` remains expected-red at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`. **NEXT**: replay target-level `Load_InGame` / `loc_2FFE8` vertical row uploads through the new row-history primitive, then feed the typed `$87B2`/`loc_12824` source buffer; do not retry stateless alias or horizontal timing patches.

---

## 2026-06-21 handoff (#289 typed loc_12848 source buffer)

- **#289 stays `[~]`; native-player now has a typed `ChunkedBackgroundSourceLoc12848` primitive for ASM `Load_BackgroundChunk loc_12848`.** The renderer still constructs rows through the direct `loc_12830` path, so this is model/test groundwork rather than a pixel change.
- A focused regression proves `Load_BackgroundMappings loc_12824` alias offsets work at the source-buffer boundary: `BackgroundLayout_0E` x offset `$13` moves an 8-byte chunk into B18 columns `49..56`, while the direct path keeps the unadjusted clipped columns. The slice gate also caught stale core/bench `LevelRecord` fixtures, now filled with `background_layout_alias: None`. Fresh `issue289-level07-idle-current` remains expected-red at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`. **NEXT**: consume this source buffer from a persisted Plane-B row-history model, not an alias-only renderer patch.

---

## 2026-06-21 handoff (#289 alias metadata reaches render)

- **#289 stays `[~]`; `BackgroundLayoutAlias` now reaches render `LevelVisuals`.** The data bundle and visual loader preserve the MapOrder-resolved shared-background alias from ASM `Load_BackgroundMappings loc_12824`, so the future `$87B2` source-buffer model can consume it without re-parsing or hardcoding.
- This is still boundary/model work only: no native-player renderer output changed. Fresh `issue289-level07-idle-current` remains expected-red at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`; the alias-only renderer patch remains rejected. **NEXT**: use the metadata in the live `$87B2` construction plus persisted Plane-B row-history model, not in a stateless alias-only overlay.

---

## 2026-06-21 handoff (#289 typed background alias metadata)

- **#289 stays `[~]`; native manifest normalisation now preserves ASM `Load_BackgroundMappings loc_12824` shared-background alias metadata.** `LevelRecord.background_layout_alias` records `BackgroundLayout_0E -> BackgroundLayout_0C`, source path `background/0C.bin`, x offset `$13`, and y offset `0` from the copied `level/mapheader_definitions.asm`/`level/background_includes.asm` files.
- This is asset-boundary model work only, not a renderer fix. Fresh `issue289-level07-idle-current` remains expected-red at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`; the standalone alias-only renderer patch remains rejected. **NEXT**: build the live `$87B2` construction plus persisted Plane-B row-history model before applying alias/source bytes to native rendering.

---

## 2026-06-21 handoff (#289 full `$87B2` source-row trace)

- **#289 stays `[~]`; oracle settle trace now dumps full target `$FFFF87B2` source rows for `Load_BackgroundChunk loc_12848`.** `background_source_rows_loc_12848` shows `$8C9E[00..45]` all zero through `title_wait` step 40, `$8FA0` filling at step 41, then `$8C9E` filling at step 42 with columns `49..56 = 01..08` while `Game_Mode=$000C`, `Current_LevelID=$0007`, `Camera_Y=0`, and `Background_width=$46`.
- This confirms the B18 cloud-tail source is written during target-level `Load_InGame`/`Load_BackgroundMappings loc_12824`, matching the `BackgroundLayout_0E` `$8000` x-offset `$13` path as an ingredient. The standalone alias-only renderer patch remains rejected because it worsened #289 without a persisted Plane-B row-history model. **NEXT**: type/model live `$87B2` construction plus persisted Plane-B row state before applying those source bytes to native rendering.

---

## 2026-06-21 handoff (#289 loc_12884 model correction)

- **#289 stays `[~]`; fixed a narrow ASM model blind spot found while reading `Load_BackgroundChunk`.** `background_chunk_row_loop_loc_12884` now uses the separate chunk-height row counter (`d3` from ASM `loc_1285C`) instead of copying `chunk_width` rows by accident; a 2x3 regression covers the old square-chunk-only gap.
- This is model/test-only and does not affect the current native-player render residual. Fresh `issue289-level07-idle-current` remains state-clean/pixel-red at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`. **NEXT**: continue the previous `Load_InGame` / live `$87B2` construction investigation before replaying Plane-B writes; do not treat the helper correction as the runtime fix.

---

## 2026-06-21 handoff (#289 pre-target source-buffer phase)

- **#289 stays `[~]`; oracle settle trace now samples the fixed `$87B2` target source window across the warp/title-card path.** The stale B18 source bytes are absent through `title_wait` step 41, then appear at step 42 as `8C9E[28..38]=00..00,01,02,03,04,05,06,07,08` while `Game_Mode=$000C`, `Current_LevelID=$0007`, and `Background_width=$46`.
- This narrows the owner to target-level `Load_InGame`, not previous-level residue or target-live hidden frames. B29 fills one frame earlier at title_wait step 41. **NEXT**: capture the exact writer/order inside `Load_InGame` or model the ROM's live `$87B2` buffer construction before replaying Plane-B writes; do not retune row pointers, layout parsing, or `$08` scroll.

---

## 2026-06-21 handoff (#289 source-buffer settle trace)

- **#289 stays `[~]`; oracle settle trace now exposes the live `$FFFF87B2` source bytes read by ASM `sub_30744`.** Target-live step 0 already has `B18@8C9E[$31..$33]=01,02,03`, and steps 18/34/50 then stream those bytes through slot `$08` into B18 VDP columns 49/50/51 as `2781/2782/2783`.
- This confirms the cloud tail comes from stale source-buffer state before target-level load: the generated/static Level 0x07 background leaves row 18 columns `$31..$33` zero, and B29 still matches current generated source data. **NEXT**: capture/model the pre-target `$87B2` source-buffer owner, then persist/replay pre-recording Plane-B writes; do not retry row-pointer remaps, layout parsing, or `$08` scroll substitution.

---

## 2026-06-21 handoff (#289 row-pointer settle trace)

- **#289 stays `[~]`; oracle settle trace now also exposes `BackgroundScroll_ComputeShiftData` slots and the `$FFFF4D5C` row-pointer window consumed by ASM `sub_30744`.** The #289 probe CSV shows target-live steps 18/34/50 writing slot `$08` source columns `$31/$32/$33` to B18 VDP columns 49/50/51 as tiles `2781/2782/2783`.
- New evidence rejects the row-pointer-remap theory: at those writes `Camera_Y>>5=$11`, `Background_width=$46`, `$4D5C[$11]=$8C58`, and `$4D5C[$12]=$8C9E`, matching the normal `$FFFF87B2 + width*row` sequence. **NEXT**: dump/model the live `$FFFF87B2` source bytes that feed `sub_30744`, then persist/replay pre-recording Plane-B writes; do not chase another `$08` scroll substitution or special row-pointer mapping.

---

## 2026-06-21 handoff (#289 settle-trace Plane-B history)

- **#289 stays `[~]`; oracle helper instrumentation now exposes selected Plane-B rows during the hidden settled-warp target-live loop.** `oracle-probe --capture-settle-trace` retains `background_plane_b_rows_loc_307d8` for each hidden settle row and prints it in the settle CSV.
- New #289 evidence rejects the pure `loc_2FFE8` preload-row hypothesis: target-live step 0 has B18 as `$0780` + `$2780` fill only, while step 59/frame 0 already has `2781..2783` at columns 49..51. The frame-60 oracle tail `2781..2787` is hidden-settle plus visible `sub_30744` one-column history. **NEXT**: persist/replay pre-recording `sub_30744` row-column writes into native Plane-B row state; do not retry broad `$08` scroll substitutions or another renderer-local lower-sweep guess.

---

## 2026-06-21 handoff (#289 loc_2FFE8 sweep diagnostic)

- **#289 stays `[~]`; no gameplay code kept.** A temporary renderer-module diagnostic simulated the `loc_2FFE8` lower sweep with `sub_307D8` 41-cell uploads and visible-frame `sub_30744` one-cell updates against the real Level 0x07 source buffer. It matches the broad B29 ocean-row structure but starts one source column behind the oracle unless the renderer's existing +1 source bias is applied.
- The same exact sweep still cannot explain oracle row `B18`'s cloud tail: predicted `B18` is old `$0780` plus `$2780` fill, while oracle keeps `2781..2787` at columns 49..55. Refreshed `issue289-level07-idle-current` remains state-clean/pixel-red at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`. **NEXT**: instrument or type the actual Plane-B nametable rows immediately after `loc_2FFE8` preload, before visible-frame `sub_30744`; do not retry a renderer-local approximate sweep without that oracle preload state.

---

## 2026-06-21 handoff (#289 stale Plane-B row-history dead end)

- **#289 stays `[~]`; no gameplay code kept.** A local diagnostic reconstructed the Level 0x07 chunked `$FFFF87B2` source buffer and rejected the simple right-edge overflow theory for row `B18`: ASM `Load_BackgroundChunk loc_12848` clips chunk 0 at x=68 against `Background_width=70`, so current source row 18 contains only `68:01 69:02`, not the `01..07` tail seen in oracle Plane-B row `B18`.
- The remaining oracle `B18` cloud tail (`2781..2787`, frame 71 adding `2788`) is therefore stale nametable history from the `loc_2FFE8` preload/vertical sweep, later partly overwritten by the current fresh `sub_307D8` span. **NEXT**: model exact `loc_2FFE8` persisted Plane-B row state before visible-frame `sub_30744`; do not retry source-buffer horizontal wrap, raw `$08` display/upload scroll, current-time all-row overlay, or row-count changes.

---

## 2026-06-21 handoff (#289 elapsed sub_30744 columns)

- **#289 stays `[~]`; native-player now replays elapsed visible `sub_30744` tile-column writes after the `loc_2FFE8` preload span.** This models the one-cell `loc_307B2` update that runs each frame before conditional `sub_307D8`, so synthetic `$08` rows advance VDP column 45 from the old `$0780` fill to `$2780` once `Time_Frames=114..174` crosses the next tile. Focused regression: `chunked_timed_plane_b_window_uses_preload_time_loc_2ffe8_307d8`.
- Fresh `issue289-level07-idle-current` remains state-clean/pixel-red and metric-neutral at frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`. Rejected narrower raw `$08` display-scroll experiment: using `loc_30452` slot `$08` for display only broke `auto_scroll_rows_prefer_typed_time_frames_loc_3038a`, broke the timed-row regression, and worsened the recording to frame 60 `5327` / `5.94182`, frame 71 `5466` / `6.162584`. **NEXT**: derive the exact row-specific Plane-B state/scroll split for row `B18`; current x=285,y=10 samples native VDP column 41 as empty while oracle clouds sit at columns 49..56.

---

## 2026-06-21 handoff (#289 old Plane-B cells + raw-scroll rejection)

- **#289 stays `[~]`; cells outside the fresh chunked `sub_307D8` row span now preserve the old Plane-B `$0780` tile.** This matches `loc_11F48` filling Plane B with `$0780` and the oracle row dumps where untouched cells remain `$0780` while streamed cells use `unk_309B8` `$2780+id`. Focused regression: `chunked_timed_plane_b_window_uses_preload_time_loc_2ffe8_307d8`.
- Fresh `issue289-level07-idle-current` remains state-clean/pixel-red at the prior residual: frame 60 `5080` / MAE `5.6536646`, frame 71 `5212` / `5.8645835`. Rejected raw `loc_30452` display-scroll substitution: using slot `$08 cur=0x007B` directly worsened to frame 60 `5676` / `6.381301`, frame 71 `5794` / `6.573024`, and still missed the right cloud span because native VDP row col 51 came from stale source col 52/tile 0. **NEXT**: model accumulated `sub_30744` one-column writes after preload; do not retry raw `$00/$08`, raw `loc_30452` display scroll, current-time all-row overlay, or stateless chunk wrapping.

---

## 2026-06-21 handoff (#289 preload-time Plane-B rows)

- **#289 stays `[~]`; chunked timed Plane-B rows now use the `loc_2FFE8` preload phase for their uploaded row window.** The renderer still displays rows with current `BackgroundScroll_ComputeShiftLayers loc_3050A` H-scroll, but `$00`/`$08` row-cell arrangement uses `Time_Frames - frame_index`, matching the level-entry preload window seen in the settled recording. Sparse `$FFFF87B2` cells are also modelled as chunk id `0` (`$2780`) rather than missing cells. Regressions: `chunked_plane_b_overlay_renders_cleared_chunk_zero_loc_12848_307d8` and `chunked_timed_plane_b_window_uses_preload_time_loc_2ffe8_307d8`.
- Fresh `issue289-level07-idle-current` remains state-clean/pixel-red, but residual improves from frame 60 `5137` / MAE `5.711784` and frame 71 `5260` / `5.920638` to frame 60 `5080` / `5.6536646` and frame 71 `5212` / `5.8645835`. **NEXT**: continue from incremental/persisted Plane-B row history after preload; the remaining clouds/ocean residual needs `sub_30744` one-column updates across the live frames, not another current-time all-row overlay. `mise` wrappers remain blocked by missing `fnox@1` linux-arm64 assets; use direct cargo/CLI commands.

---

## 2026-06-21 handoff (#289 shared-background alias rejected)

- **#289 stays `[~]`; no code kept from the shared-background alias trial.** Re-reading `background_includes.asm` found `BackgroundLayout_0E` uses ASM `loc_12824` indirection with x offset `$13` into `BackgroundLayout_0C`. A trial manifest/render patch applied that offset to native chunked Plane-B source composition; focused synthetic coverage passed, but `issue289-level07-idle-current` worsened from frame 60 `5080` / MAE `5.6536646` and frame 71 `5212` / `5.8645835` to `6359` / `5.9081984` and `6437` / `6.035942`. Applying it to direct placement too was still worse than baseline (`6236` / `5.760384`, `6322` / `5.8884997`). The code was reverted and the dead end recorded under #289.
- **NEXT**: continue from exact `Load_InGame` writer/order or a typed live `$87B2`/Plane-B row-history model. The `$8000` alias may be part of that writer path, but it is not a standalone renderer fix. Do not retry `BackgroundLayout_0E` alias-only, header/start layout, simple scroll offsets, raw `$00/$08`, stateless `$FFFF87B2`, or current-time all-row overlay. Verification this pass: focused alias trial test, expected-red recording metrics for both variants, code reverted, backlog/archive/session docs updated.

---

## 2026-06-21 handoff (#289 Plane-B H-scroll mapping)

- **#289 stays `[~]`; the partial chunked Plane-B row-window overlay now renders through hardware H-scroll.** Native keeps `sub_30744`/`sub_307D8` cells in VDP-column order and maps them to screen pixels with `BackgroundScroll_ComputeShiftLayers loc_3050A`. Focused regression `chunked_plane_b_overlay_samples_hscrolled_vdp_column_loc_3050a` covers the frame-60-shaped 36px row-scroll case.
- Fresh `issue289-level07-idle-current` remains state-clean/pixel-red, but residual improves to frame 60 `5137` / MAE `5.711784` and frame 71 `5260` / `5.920638`. **NEXT**: continue from persisted/stale Plane-B row history for upper `$08` rows; the top cloud diffs now show native `sample=None` at x≈285..319 while oracle row dumps contain cloud words in the scrolled VDP row. Do not retry header/start layout, simple scroll offsets, placement wrap, raw `$00/$08`, y=100 H-scroll, row-window count, render `Time_Frames`, stateless `$FFFF87B2`, or all-row overlay changes. `mise` wrappers remain blocked by missing `fnox@1` linux-arm64 assets; use direct cargo/CLI commands.

---

## 2026-06-21 handoff (#289 render Time_Frames aligned)

- **#289 stays `[~]`; native settled-warp rendering now uses the ROM `Time_Frames` phase.** `NativeTraceFrame` carries `time_frames_loc_3038a`, incremented at ASM `loc_3038A`; the level-skip resume path clears it after the visible/hide/preload tail that reaches `Init_Timer_and_Bonus_Flags`, matching the oracle helper instead of native's earlier resume setup. Fresh `issue289-level07-idle-current` remains state-clean/pixel-red, but native bgscroll diagnostics now match oracle time exactly (`174` at frame 60, `185` at frame 71) and improve the residual to frame 60 `6795` / MAE `6.6150064`, frame 71 `6873` / `6.75519`.
- **NEXT**: continue from live Plane-B row/name-table contents after `sub_30744`/`sub_307D8`; row-29/y=100 H-scroll and render `Time_Frames` are no longer leads. Do not retry header/start layout, simple scroll offsets, placement wrap, raw `$00/$08`, stateless `$FFFF87B2`, or all-row overlay changes. Verification: format/check, focused core + CLI `time_frames_loc_3038a` tests, recording tests, fresh red #289 recording report, debug-field audit with #289 exemptions, direct fast-slice body, and backlog lint; `mise` wrappers remain blocked by missing `fnox@1` linux-arm64 assets.

---

## 2026-06-21 handoff (#289 native scroll diagnostic)

- **#289 stays `[~]`; native verification reports now expose chunked bgscroll row diagnostics beside sampled pixels.** This is report-only instrumentation for ASM `BackgroundScroll_ComputeShiftLayers loc_3050A` plus `sub_30744`/`sub_307D8`; it does not mutate trace or renderer output. Fresh `issue289-level07-idle-current` remains state-clean/pixel-red at frame 60 `7103` / MAE `6.9787807` and frame 71 `7227` / `7.1648297`.
- **NEXT**: split settled-warp render `Time_Frames` from recording-local trace frame indices before another row-history patch. The new report shows native sampled render rows using `time_frames_loc_3038a=1693/1704`, while oracle reports ROM `Time_Frames=174/185` for the same recording frames. Do not retry header/start layout, H-scroll phase, raw `$00/$08`, direct/stateless placement wrapping, or all-row Plane-B overlay. Verification so far: `cargo fmt --check`, `cargo check -p wildside-cli`, `python3 scripts/audit_debug_fields.py`, focused `cargo test -p wildside-cli recording::`, and fresh red #289 recording report.

---

## 2026-06-21 handoff (#289 row primitive correction)

- **#289 stays `[~]`; the shared `sub_307D8` row-upload helper now matches ASM `loc_308A6`'s inclusive `d4=#$28`/`dbf` copy count.** It now streams 41 cells, so the full-row fixture wraps one extra source cell and the source/VDP wrap fixture keeps the ninth byte in the final segment. This is a primitive correction for the pending live Plane-B row-history model; native-player's partial renderer already used its own row-window count, so `issue289-level07-idle-current` remains frame 60 `7103` / MAE `6.9787807` and frame 71 `7227` / `7.1648297`.
- **NEXT**: continue from live/stale Plane-B row history for upper rows. Do not retry H-scroll phase, raw `$00/$08` formulas, stateless placement wrapping, all-row row-window overlay, or header/start-layout edits. Verification this session includes focused `wildside-data` row-upload tests, native-player background tests, fresh `verify-recording`, debug-field audit with the explicit #289 scroll-snapshot stale-candidate exemption, direct fast-slice body, TAS frontier, and backlog lint; `mise` wrappers remain blocked by missing `fnox@1` linux-arm64 assets.

---

## 2026-06-21 handoff (#289 partial Plane-B row-window render)

- **#289 stays `[~]`; native-player now uses a partial chunked Plane-B row-window overlay for verified camera-parallax rows.** The renderer reconstructs `$FFFF87B2` via `Load_BackgroundChunk loc_12848`, then overlays rows using the `sub_30744`/`sub_307D8` window (`d4=#$28` DBF stream plus the preceding one-column updater). Applying the model to every chunked row was rejected because it worsened `issue289-level07-idle-current` to frame 60 `8473` / MAE `7.6971307` and frame 71 `8580` / `7.8483305`; the kept camera-parallax gate improves the current route to frame 60 `7103` / `6.9787807` and frame 71 `7227` / `7.1648297`, still state-clean/pixel-red.
- **NEXT**: model live/stale Plane-B row history for upper rows. The top-cloud row still needs persisted VDP contents rather than direct placement or a stateless row window. Do not retry H-scroll phase, raw `$00/$08` formulas, simple placement wrapping, or header/start-layout edits. Verification passed via focused sampler/render tests, fresh `verify-recording`, direct `verify-slice-fast` body, TAS frontier, and backlog lint; `mise` wrappers remain blocked by missing `fnox@1` linux-arm64 assets.

---

## 2026-06-21 handoff (#289 Plane-B row probe)

- **#289 stays `[~]`; instrumentation now exports selected oracle Plane-B nametable rows after `sub_30744`/`sub_307D8`.** Fresh `issue289-level07-idle-current` remains state-clean/pixel-red at frame 60 `7070` / MAE `6.987421` and frame 71 `7341` / `7.302334`, but the row dump proves the residual is live hardware row state: frame-60 `B29` starts with four blank cells then `2799,2795,2796...`, while native samples y=100 from direct logical chunk column 12 as tile `2795`; `B18` also has cloud words at columns 49..56 while native misses the far-right top band.
- **NEXT**: render/reconstruct chunked backgrounds through the ROM's 41-word `sub_307D8` row window from `$FFFF87B2` plus `Horiz_Scroll_Data`. Do not retry scroll offsets or stateless placement wrapping; the scroll probe already shows y=100 H-scroll matches (`FEDCFFDC`), and prior stateless wrap/raw-scroll experiments worsened the recording. Verification passed via the direct `verify-slice-fast` body because the `mise` wrapper is blocked here by missing `fnox@1` linux-arm64 assets.

---

## 2026-06-21 handoff (#289 oracle scroll probe)

- **#289 stays `[~]`; instrumentation now exports oracle `BackgroundScroll_ComputeShiftData`/`ComputeShiftLayers` RAM for the Level 0x07 recording.** Fresh `issue289-level07-idle-current` remains state-clean/pixel-red at frame 60 `7070` / MAE `6.987421` and frame 71 `7341` / `7.302334`, but `oracle_background_scroll_loc_3050a` shows `hbuf[100]=FEDCFFDC` on both frames, matching native's -36 row-29 scroll. The dominant y=100 ocean mismatch is therefore actual Plane-B row/name-table contents, not H-scroll phase.
- **NEXT**: export/reconstruct Plane-B row contents after `sub_30744`/`sub_307D8`. Do not retry simple scroll offsets: forcing the raw oracle `$00/$08` formula (`Time_Frames + Camera_X>>3` / `Time_Frames/2 + Camera_X>>3`) worsened #289 to frame 60 `7347` / `7.378413` and frame 71 `7568` / `7.640611`, then was reverted.

---

## 2026-06-21 handoff (#289 settled auto-scroll timing narrowed)

- **#289 stays `[~]`; the Level 0x07 recording is still state-clean/pixel-red, but the `$00/$08` auto-scroll rows now use the ROM `Time_Frames` basis for settled warps.** ASM `BackgroundScroll_ComputeShiftData loc_30452` derives those rows from `Time_Frames`; this matches the existing `loc_305E6` ripple offset rule. Fresh `issue289-level07-idle-current` improves frame 60 from `7277` changed / MAE `7.258966` to `7070` / `6.987421`, and frame 71 from `7502` / `7.521498` to `7341` / `7.302334`.
- **NEXT**: do not chase whole-frame X offsets; ImageMagick band checks for y=96..112 and y=8..24 are already best at zero roll. The remaining y=100 ocean row is unchanged, so continue from live `Horiz_Scroll_Data` / actual Plane-B row state after `sub_30744`/`sub_307D8`, not another stateless placement or scroll-offset approximation.

---

## 2026-06-21 handoff (#289 chunked buffer dead end)

- **#289 stays `[~]`; no code change kept.** Tested a source-backed but stateless approximation for chunked backgrounds: reconstruct `$FFFF87B2` via `Load_BackgroundChunk loc_12848`, clip to `Background_width/height` from `loc_11B40`, and fold columns through the 64-cell Plane-B row used by `sub_307D8`. It worsened `issue289-level07-idle-current` from frame 60 `7277` changed / MAE `7.258966` to `7723` / `7.803023`, and frame 71 `7502` / `7.521498` to `7929` / `8.047498`, with new top-row cloud errors. The code was reverted and the verifier was rerun to restore the baseline report.
- **NEXT**: continue #289 from live `Horiz_Scroll_Data` phase/VDP row state, not from stateless chunk wrapping. The remaining ocean rows still look like a one-tile/phase mismatch around y=100, while the cloud row needs the actual VDP row contents after the ROM's incremental `sub_30744`/`sub_307D8` uploads. Verification this session: focused background compile test, fresh `verify-recording --slug issue289-level07-idle-current` after revert (state-clean/pixel-red baseline restored), `mise run backlog-lint`, and `git diff --check`.

---

## 2026-06-21 handoff (#289 damage flash narrowed)

- **#289 stays `[~]`; the Level 0x07 state-clean/pixel-red recording now has Kid's damage flash matched.** Native-player keys the white phase from `Just_received_damage` plus the trace-sampled `64 - invulnerable_frames` phase, matching `kid.asm loc_BC56` / `sub_80BC` at frames 60 and 71. Fresh `issue289-level07-idle-current` remains `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`, but improves from frame 60 `7703` changed / MAE `8.285068` to `7277` / `7.258966`, and frame 71 `7928` / `8.547601` to `7502` / `7.521498`.
- **NEXT**: continue #289 from the remaining background/ocean/cloud colour/scroll residual, not player flash or projectile trajectory. A local experiment applying layered-style horizontal wrapping to chunked Plane-B rows worsened the recording to frame 60 `7752` / MAE `7.8057895` and frame 71 `7993` / `8.071968`, with new top-row cloud errors; inspect actual chunked scroll phase, VDP samples, `Horiz_Scroll_Data`, or row byte mapping before another render patch. Verification this session: `cargo fmt`, `cargo test -p wildside-cli loc_bc`, fresh `verify-recording --slug issue289-level07-idle-current` (state-clean/pixel-red, improved), and `mise run verify-slice-fast`. #345 Lion vanish remains separate.

---

## 2026-06-21 handoff (#289 Lion projectile DBF loop narrowed)

- **#289 stays `[~]`, with the Level 0x07 Lion projectile post-wall arc fixed down to a near-pixel residual.** Native now stores the `Lion.asm loc_3E2A8`/`loc_3E2C0` DBF bounce-loop budget, suppresses homing while the loop is active, decrements only empty checks, and preserves the budget across terrain-hit frames. Fresh `issue289-level07-idle-current` remains `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`, but the projectile lead is mostly exhausted: frame 60 native `(427,648)` vs oracle `(427,649)`, frame 71 native `(488,638)` vs oracle `(488,639)`.
- **NEXT**: continue #289 from the remaining pixel-red Level 0x07 recording or broader hell-theme wrong-start/layout repro, not from projectile homing. The current report shows broad background/terrain-colour diffs (`frame60 changed=7703`, `frame71 changed=7928`); inspect `verify/native/*_diff.png`, VDP/background samples, or capture the human route before changing header/start layout. Verification this session: `cargo test -p wildside-core lion_projectile --lib`, `cargo test -p wildside-core loc_3e260 --lib`, `cargo test -p wildside-core projectile --lib`, regenerated + verified `issue289-level07-idle-current` (state-clean/pixel-red), `mise run verify-slice-fast`, and `mise run verify-tas-full` (CRC matched all frames; existing state-report divergences remain baseline). #345 Lion vanish remains separate.

---

## 2026-06-20 handoff (#289 Lion projectile trajectory narrowed)

- **#289 stays `[~]`, with Level 0x07 Lion projectile launch and terrain-bounce behaviour narrowed.** Native now offsets the left-facing projectile from the native carrier so oracle launch frames 19-26 match, moves Lion shots before homing like `Lion.asm loc_3E192`, and uses the full `unk_31944` collision box for `loc_3E260` floor/left-wall terrain snaps. The fresh `issue289-level07-idle-current` recording is still `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`.
- **NEXT**: continue #289 on the residual post-wall arc. The wall snap now reaches `x=357`, but vertical motion remains low: frame 60 native `(415,677)` vs oracle `(427,649)`, frame 71 native `(477,689)` vs oracle `(488,639)`. Re-check `loc_3E260`/`loc_3E2A8` velocity/timing before broader header/start/layout work. Verification this session: `cargo test -p wildside-core loc_3e260 --lib`, `cargo test -p wildside-core lion_projectile --lib`, `cargo test -p wildside-core projectile --lib`, regenerated + verified `issue289-level07-idle-current` (state-clean/pixel-red), `mise run verify-slice-fast`, and `mise run verify-tas-full` (CRC matched all frames; existing state-report divergences remain baseline). #345 Lion vanish remains separate.

---

## 2026-06-20 handoff (#289 Lion projectile collision/lifecycle narrowed)

- **#289 stays `[~]`, with Level 0x07 Lion projectile collision/lifecycle partially fixed.** Native now uses the projectile's ROM sprite collision row (`Lion.asm loc_3E12A` -> `stru_3DF34` -> `CollisionSize_Index[667..669]`/`unk_31944`) instead of the generic hostile-projectile centre window, so frame 28's old `dx=11 dy=-17` candidate is correctly `overlap=0`. Side/below damage no longer deletes the shot; `Lion.asm loc_3E192` only deletes on `colid_kidabove`/`colid_hurt`. Fresh `issue289-level07-idle-current` still reports `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; damage latches at frame 29 and native keeps the projectile alive.
- **NEXT**: continue #289 by tracing the Lion projectile terrain-bounce/trajectory path after damage. The remaining concrete mismatch is visual/trajectory: at frame 60 native projectile is around `(325,754)`, while oracle raw object slot `0xffff0f70` is `(427,649)`; frame diffs remain `7285/7515` changed pixels. Do not close #345 from this; that is still the separate Lion scroll-off/scroll-back vanish probe. Verification this session: `cargo test -p wildside-core projectile --lib`; regenerated + verified `issue289-level07-idle-current` (state-clean/pixel-red); `mise run verify-slice-fast`. `mise run verify-tas-full` exited 0 and CRC-matched all frames, but still reports existing authoritative object-count divergences from around frame 7850.

---

## 2026-06-20 handoff (#289 stale X rejected)

- **#289 stays `[~]`, but the Level 0x07 X lead is rejected.** Generated fresh local recording `issue289-level07-idle-current` and ran `verify-recording --slug issue289-level07-idle-current`: report has `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`. The earlier `native=$02118000` / `oracle=$02058000` Lion X note came from a stale/missing route and is not current state-oracle evidence. Temporary attempts to pin that X in `level07_recording_settle_lion_survives_frame0_offledge_fall_loc_3e7ca` and to alter Lion settle/movement were reverted.
- **NEXT**: continue with `mise run project-status`; if it returns #289 again, inspect `.cache/recordings/issue289-level07-idle-current/verify/` pixel diffs or capture the original live hell-route symptom before touching header/start/layout code. #345 Lion vanish remains separate.

---

## 2026-06-20 handoff (#289 partial Lion survival)

- **#289 stays `[~]`, with one Level 0x07 Lion subcase fixed.** Native no longer routes the lower Lion's settled frame-0 off-ledge charge fall through `Lion.asm loc_3E94E` when the body terrain scan only overlaps the floor row at the feet. The replacement cache-backed regression now asserts the oracle-backed survival state (`y=$02C0_0000`, HP 6, Kid HP 2, no damage) instead of the stale projectile/body-contact timing claims. Fresh `verify-tas` still reports first divergence at frame 0, but now only on the remaining X-position/grounding/export details (`native x=$02118000`, `oracle x=$02058000`); broader #289 hell spawn/layout work remains open. Verification: focused Level 0x07 Lion regression, Level 0x06 Flying Dragon guard, Lion terrain/floor-edge tests, `verify-slice-fast`, and `verify-tas-full` (exit 0; CRC all 37759 frames match, current baseline state-report divergences still begin at `transition_15_to_00_post`).
- **NEXT**: continue with `mise run project-status`; if it returns #289 again, derive the Level 0x07 settle X-position miss from `Lion.asm` plus oracle before touching header/start/layout code. #345 Lion mid-level vanish is not covered by this fix and still needs the scroll-off/scroll-back cull probe.

---

## 2026-06-20 handoff (#272 fixed candidate)

- **#272 DAC voice pitch moved `[~]` -> `[?]`.** Dynamic native-audio verification now writes `audio-event-oracle-windows.json` beside each real jgenesis WAV capture, so the old native-as-oracle audio audit is no longer the only pitch evidence. The reports first exposed the low/slow native DAC path, then verified the fix: standalone GEMS now advances the YM2612 timer/status shadow at the YM input clock instead of raw Z80 cycles, and clears the hidden init-frame audio event epoch before visible playback. Evidence: `issue272-red-stealth-air-attack-direct` reports `$13` at 641 Hz native vs 638 Hz jgenesis; the level route covers `$60`, `$13`, `$46`, `$66` in the same pitch range. Focused GEMS/verifier tests pass; human audio-device validation is still required.
- **NEXT**: continue with `mise run project-status`; #272 is no longer in the open queue. Current audio work is now the looping-SFX leak family (#342/#339), then block render (#324), entry presentation (#336/#348/#254/#210), and combat/render lifecycle (#343/#345/#346/#350).

---

## 2026-06-20 handoff (#264 fixed candidate)

- **#264 live audio slowdown moved `[~]` -> `[?]`.** Live `play-native` now keeps generated GEMS/fallback frames in a small software backlog whenever SDL's two-frame device queue is full, then drains those frames in order before/after later video frames. This removes the old path where GEMS advanced and an already-mixed block/diamond/BGM frame was discarded at the cap. Hard BGM resets still clear SDL plus the backlog. Diagnostics now expose backlog pressure (`backlog_frames`, `backlog_full_frames`, `max_backlog_frames`) and `skipped_full_frames` stays 0 for compatibility. Verification: focused audio regressions, `cargo fmt`, `mise run backlog-lint`, `mise run verify-slice-fast`, and `mise run verify-tas-full` exits 0 with CRC matching all 37759 frames; current baseline state-report divergences still start at `transition_15_to_00_post`.
- **NEXT**: continue with `mise run project-status`; #264 is no longer in the open queue and awaits real audio-device validation. Current thrust remains audio (#272/#342/#339 family), then block render (#324), entry presentation (#336/#348/#254/#210), then combat/render lifecycle (#343/#345/#346/#350).

---

## 2026-06-20 handoff (#335 fixed candidate)

- **#335 Hills 0x08 instant death moved `[~]` -> `[?]`.** The headless generated-assets debug entry path reproduced the human N/A+C skip failure: first visible InGame frame was already `DeathWindup` with a pre-aged Hills murder wall, then life dropped shortly after. Native debug title-card entry suppresses the loc_1AE2E carry frames, so the handoff now collapses the hidden Load_InGame shell back to a fresh settled target spawn and clears the hidden death windup before exposing Mode_Level. Verification: cache-backed core regression `debug_level_entry_hills_resets_murder_wall_before_live_handoff_loc_74e0`, ignored native-player generated-assets skip regression, `cargo fmt`, `mise run verify-slice-fast` green, and `mise run verify-tas-full` exits 0 with the current baseline state-report divergences unchanged (CRC all 37759 frames match).
- **NEXT**: continue with `mise run project-status`; #335 is no longer in the open queue and awaits human playtest validation. Current thrust remains audio (#264/#342/#272), block render (#324), entry presentation (#336/#348/#254/#210), then combat/render lifecycle (#343/#345/#346/#350). #334 remains blocked on a Caves-of-Ice live leak repro.

## 2026-06-20 handoff (#352 fixed candidate)

- **#352 hidden-block reveal/fold animation moved `[~]` -> `[?]`.** Live hidden-block headbutt reveals now queue `sfx_Reveal_Hidden_block` (`$12`) and spawn a `BlockBounceKind::HiddenReveal` worker with the ASM loc_E2E2/loc_E2F0 8-frame y-bob, so revealed cells have the fold/bob instead of instant pop-in. Trace exposes `BlockBounceTraceKind::HiddenReveal` for the renderer path. Verification: focused hidden-block/trace/block-bounce tests, `cargo fmt`, and `mise run verify-slice-fast` pass. `mise run verify-tas-full` keeps published TAS CRC parity across all 37759 frames; the state report still shows the known #290 object-count wording caveat. Human validation: hit hidden blocks on USM 0x04-0x06 or Highwater 0x02 and confirm the reveal animation is visible.
- **NEXT**: continue with `mise run project-status`; #352 is no longer in the open queue. The current thrust remains audio (#264/#342/#272), block render (#324), entry presentation (#336/#348/#254/#210), then combat/render lifecycle (#343/#345/#346/#350).

## 2026-06-20 handoff (compressed from NOW)

- **2026-06-20 playtest triage — 28-symptom human playthrough cross-referenced vs backlog + ASM (29-agent workflow). Docs-only; the user's "issues keep getting missed or denied" complaint is CORRECT and falls into 4 anti-patterns.** (1) **DENIED-by-parking**: P1 audio rows sat at `[?]` across 5+ playtests, each pass tweaking `AUDIO_QUEUE_TARGET_FRAMES` and re-blocking on "needs a real audio device" — the recurring human repro IS that device, so `[?]` is not defensible. (2) **MISSED start-without-stop wiring**: loop-stop helpers (`loc_107E0` clock $0E, `loc_3C46E` tank-drive $1F) ported as unit-tested but `#[cfg(not(test))]` dead code never called at runtime. (3) **PARKED-with-gate-unmet**: #324's own verify gate ("human forest retest") was treated as satisfied; the fresh High Water Pass repro proves it still breaks. (4) **MISSED-by-keyword-deflection**: triagers grep "dragon"/"tank", land on closed projectile rows, declare it solved. Also: several "green" tests assert MODEL state or use a native-as-oracle audit, so they pass while the human-visible RENDER/PITCH defect goes undetected — durable fixes need native-player render assertions + a real jgenesis WAV comparison, not more model-level tests.
  - **REOPENED `[?]`/`[b]`→`[ ]`** (fresh human repro = the validation the row awaited, and it FAILED): #264, #272, #324, #280, #254, #210, #323; #339 (also P2→P1), #338 (also P2→P3); #336 + #335 flipped `[b]`→`[ ]` (the live repro the block waited for has arrived). **STATUS-CORRECTED** #271 `[x]`→`[?]` (closed on synthetic jugg/Dragon probes; its own note says "needs human re-test", and the live Level-2 RockTank stomp route was never captured).
  - **NEW rows #342–#352**: #342 clock-prize SFX leak (start-without-stop); #343 axe/friendly projectiles miss the walking dragon (live hit test uses an invented fixed 8×16 box, not `CollisionSize` — concrete #308 instance); #344 dragon stomp looks like die-then-revive (likely faithful, visual/UX); #345 Lion vanishes (premature off-window cull, `active_enemy_count` vs `Number_Objects`); #346 launched spike culled at 120f, never reaches the return-spike wall; #347 shifting/springy block has no rise animation (renderer drops it); #348 level card ignores Start ~343f vs `loc_1AE2E` first-edge; #349 post-death card missing level-name text; #350 Hills (theme 3) purple bg (no theme-3 palette branch); #351 Whispering Woods Drip wrong colour variant (orange not green); #352 hidden-block reveal/fold animation never spawned (split from the #309 [b] umbrella — #309 stays blocked on its type-8 slice).
  - **AUDIO splits into THREE distinct root causes — do NOT merge them**: (a) BGM stutter / "every other sound slows" = audio generated synchronously inside the video loop with a 2-frame queue cap (#264, native_player/mod.rs:801/39/844) — stop tweaking the cap, **decouple onto an audio thread / ring buffer**; triggers $12/$2D/$15/$56 are all ASM-correct so this is NOT a trigger gap. (b) DAC voices low/deep (#272) — the audio-audit "oracle" is the SAME native engine (`GemsAudioEngine::from_kid_rom`), so its delta-0 is blind to a systematic pitch error; a **real jgenesis WAV peak-freq comparison** is the only thing that resolves it (caveat: $6C/$6D are intentionally slowed in the ROM). (c) Looping-SFX leaks (#339, #342) — the dead-code stop helpers must be wired on death / level-skip / pickup-expiry.
  - **WORK ORDER (this batch)**: P0 first — **#335** (capture a live A+C skip into 0x08, dump frame-0..8 murder_wall_x vs player_x; suspect stale `MurderWall_X_pos`/fraction at weather.rs:3155 or low player_x during the entry-settle; group "other skip targets" with #266). Then AUDIO: **#264** (architecture) → **#339** + **#342** (stop-wiring) → **#272** (jgenesis audit). Then block render: **#324** (extend foreground-skin preservation to Metal/Shifting bounce cells — subsumes the metal-block background-erase symptom — and oracle-diff a forest AND High Water Pass prize hit before re-closing). Then entry-presentation: **#336** (cold-boot BLW1 frames 130..260 native vs oracle ramp diff), **#348** (level-card Start), **#254**/#210 (engine-level carry-last-VDP per `sub_1C246`, not probe-frame constants). Then combat lifecycle: **#343**, **#271** (live route), **#346**, **#345**, **#352**, **#350**. Probe-before-fix on every lifecycle/live-skip item per loop rules. Per-symptom evidence (ASM labels, file:line, why each status was/wasn't defensible) is in `backlog-archive.md` under each `#NNN` (grep `2026-06-20`).
- **Focus**: select with `mise run project-status` (priority-ordered; file order is not priority). **#274 is now status-corrected to fixed candidate**: current HEAD still rejects the cold-idle attract Diamond-prize hypothesis, with `verify-tas` CRC-clean through the first demo Cyclone prize/transform window; only an exact live SDL route/frame should reopen coding. **#289/#280 Plane-B dark-green seam has a fixed candidate**: nonzero moving `BackgroundScroll_ComputeShiftLayers loc_3050A` rows now wrap across themes, not just Hills `$08`; broader hell-route spawn/layout and live visual validation remain open. **#309 is blocked/exhausted**: actual generated Level_Layout grids still have 0 type-8 Shooter cells (type-9=562, type-C=864), and the only remaining block-audit deviations need a synthetic/oracle type-8 probe before changing the native `BlockType::Shooter` convention. **#310 is blocked/exhausted**: the terrain audit's concrete rows are fixed candidates, and the remaining trivia are representation-only, edge-only, or movement-fidelity/oracle-probe scope. **#312 is now status-corrected to fixed candidate**: the level-flow audit has no open rows left, only `[?]`/`[x]` fixed candidates with current regression coverage. **#313 is blocked/exhausted**: the only remaining shell-audit slice is visual pause VDP tile save/restore, but native-player is a stateless scene renderer with no persistent plane-A nametable to restore. **#315 enemy-fidelity audit is now status-corrected to fixed candidate**: every concrete row in `audits/enemy-fidelity-audit.md` has current fixed-candidate evidence/regressions; remaining caveats are representation-only or need fresh human/oracle routes. Continue via `mise run project-status`, not the enemy audit. **#272 is now a fixed candidate for source-port audio**: route `.cache/recordings/issue272-level01-redstealth-r0-25-b24-64/scenario.json` reaches Red Stealth transform `$2E`, and `verify-native-audio-output-file` passes; state verifier still splits around the floating item/object roster at the pickup, so human device validation remains required for the reported slow/low sound. **#338 is now a fixed candidate**: RockTank shot `$41` and creak `$3C/$42` all have native emit sites; the creak cadence follows Tank.asm `stru_3B27E`'s 80-frame linked-mouth loop and #341 audio-audit reports native-vs-ROM oracle duration delta 0 for `$3C`/`$42`. **#341 remains a fixed candidate** and is the standing tool for future SFX audits. **#249/#270 is blocked again**: the natural 0x04 held-Right/no-jump slope route is oracle-clean through the suspected loc_9FBC window, so the 2026-06-19 slope-into-wall claim is not an implementation route. **#334/#335/#336** still need human repro recordings before gameplay coding; #334's headless loader and `inspect-level` diagnostic now show the MapOrder-resolved Caves assets. Everything else from the 2026-06-19 + 2026-06-18 batches is `[?]`/`[-]`/`[b]` awaiting human validation or repro.
- **Current #316**: Five-Way Shot live slot path/negative-timer fade, Snake slot-chain negative fade/collision status/invulnerability stand-in, Tracking Rain target subpixel velocity, shared `Diamond_power_active` projection from live DP slots, `Diamond_power_ID`/`DiamondPower_Main_Index` dispatch, Juggernaut `FiveWayShotReady`, stale Circle/Wall kill-all audit claims, Eyeclops hard-beam render-time diamond spend, Eyeclops state-owned growing beam strip/direction latch/no invented speed-lifetime, Eyeclops `off_E560` short-beam table dispatch, Eyeclops hard-beam companion `loc_E948` object geometry, Maniaxe `loc_8A74` axe `x_vel` preservation, and the stale transform `+1` cleanup-frame audit claim are resolved/rejected based on current slot-vector/runtime/TAS tests. Continue #316 from the next open concrete row in `audits/helmets-fidelity-audit.md`; the broad shared DP-object linked-list row remains architectural debt.
- **NEXT (directed, 2026-06-19 triage — status after the 2026-06-19 implementation sprint)**:
  1. **#331 P1 [?] DONE** RockTank BLW2 self-kill — reverse-in-place loc_3C0D2 narrow-span wall turn. Awaiting human validation.
  2. **#276 P1 [?] DONE (already landed, e3c31ab3e)** — the reopen claim was STALE: the size-0 HUD pip fix was already on HEAD with green tests. No new work; awaiting human visual validation.
  3. **#249/#270 P1 [b] — DEAD END (do not retry the 4055 guard or the 0x04 held-right seed)**: the proposed wall_hit-gated guard at movement.rs:4055 is a NO-OP because `slope_sliding` suppresses the wall snap on slopes (movement.rs:6256, the BLW1 4082/6024 seam, #211), so `wall_hit` is always false on a slope-into-wall. The natural 0x04 held-Right/no-jump route with the periodic jump cleared (`150-153:none`, `150-220:right`) is CRC-clean against oracle frames 138..190, including the slope exit. Keep #249/#270 blocked until a sharper human route or natural BLW2 shaft route reproduces the launch.
  4. **#332 P1 [?] DONE (already landed, 7804966e7)** — same-form guard already at blocks.rs:4533. Stale reopen; awaiting human validation.
  5. **#243 + #333 P2 [?] DONE 2026-06-19** live-enemy render free-run — three `select_live_*` selectors hold Tar/Sphere/EmoRock on body frames (Tar/Sphere faithful; EmoRock a marked TODO(fidelity) approx). Hand 0x17 OUT of scope. Awaiting human capture-frames.
  6. **#338 P2 [?] DONE** RockTank SFX — SHOT `$41` and CREAK `$3C/$42` now ported. Awaiting human audio validation. **#339 [?] DONE** (drive-SFX stop).
  7. **#337 P2 [?] DONE 2026-06-19** Maniaxe stuck-axe — airborne loc_9454 latch tick (single-dispatch else-branch, no double-tick). Awaiting human playtest (no Maniaxe oracle).
  8. **#340 P3 [-] CLOSED** — `sfx_Voice_diamond` ($62) is an orphan ROM asset (zero trigger sites); native faithfully plays only $2D. NOT-A-BUG.
  9. **#334 P1 [b]** Caves-of-Ice level-skip mismap — `inspect-level` now reports MapOrder-resolved resource paths and Diamond/Hand data; still **needs human repro** to pin any live leak.
  10. **#341 P2 [?] DONE** sound-audit tool — native-vs-ROM-backed oracle duration/RMS/peak/frequency deltas now populate; use it for #338 creak cadence.
  11. **BLOCKED on human repro — do NOT auto-queue:** **#335 [b] P0** 0x08 instant-death-on-skip, **#336 [b] P1** first-level float. Need a `play-native` recording first.

- **Last session** (2026-06-19): **workflow-driven fix sprint — landed 4 fixes, 1 dead end, 1 not-a-bug close, corrected 2 stale reopens.** Ran a 7-item parallel prepare+adversarial-verify workflow over the directed queue, then drove the serial green-gated landing. (1) **#338 [~]** SHOT SFX `sfx_Tank_shoots` ($41) ported on the loc_3B730 spawn edge (`6901c1be5`); creak ($3C/$42) deferred (gun-anim phase counter). (2) **#243+#333 [?]** live-enemy render free-run fixed with three `select_live_*` selectors (`c72ba6641`) — Tar/Sphere ASM-faithful; EmoRock a marked TODO(fidelity) approx (the prepare spec's "loc_3621A has no Init_Animation" was a MISREAD — `loc_3621A` installs stru_3647E, drop animates stru_363FE; hover frames aren't extracted, so the helper pins the invariant live≠shatter-tail, not an exact frame). (3) **#340 [-]** closed NOT-A-BUG (`0091a1961`) — $62 is an orphan asset. (4) **#337 [?]** Maniaxe stuck-axe airborne gate clear (`eb6af3962`) — ported sub_942A loc_9454 as a NEW single-dispatch else-branch (NOT the prepare spec's pre-physics `!on_ground` tick, which the adversarial verify correctly flagged would double-tick on a landing frame); verify-tas-full 212 authoritative = baseline (CRC-neutral). (5) **#249/#270 DEAD END** (`0232f9588`, docs-only) — the 2026-06-19 "root cause identified (4055 wall_hit guard)" is provably a NO-OP: on a grounded slope `slope_sliding=true` so the wall snap is suppressed at movement.rs:6256 (BLW1 seam, #211) → `wall_hit` is always false → guard never fires; instrumentation confirmed `probe.hit cell=(4,4) Solid` with the snap suppressed and `wall_hit=false`. Real fix = centre-based loc_9FBC stop, needs a natural oracle route. Reverted the speculative guard rather than commit churn. (6) **#276 + #332**: confirmed both were ALREADY landed (e3c31ab3e / 7804966e7) — the NOW block's "implementation-ready" claims were stale; no new code. Process note: the prepare-the-fix agents were strong on ASM but twice wrong on native-side interactions (#249/#270 wall_hit suppression; #337 double-tick; #243 EmoRock Init_Animation) — every spec was re-derived against native before landing, which is why the adversarial-verify + serial-land split paid off. Each gameplay-affecting commit passed `verify-slice-fast` (6186+9) and #337 also `verify-tas-full` (baseline). **NEXT: #341** (sound-audit tool) then **#338 creak**; #249/#270 needs an oracle route.
- **Last session** (2026-06-19): **ported the #339 RockTank drive-SFX stop (`[ ]`→`[?]`).** The START (PlaySound `sfx_Tank_driving` $1F) was wired in `step_active_enemies_with_rng` via a pooled, recomputed refcount (count of `Tank|TankShooting` with the `shared_walker_patrol_short_next` latch); the STOP was unported, so the drone looped forever. ASM truth (verified): the per-object `$FFFFFA34` refcount decrements at loc_3C46E→loc_3C4A8 and replays `byte_3C46C[1]=sfx_Tank_driving` via `j_PlaySound2` on the 0 edge; **both** exits route there — scroll-off via the loc_3C436 bounds trampoline AND **death via Tank.asm loc_3B480 `bra.w loc_3C46E` after `Hibernate_UntilAnimFinished`** (Tank_Shooting.asm:139 likewise). Native already removes death corpses only when the death animation finishes (`remove_dead_enemy`), matching that post-hibernate timing. Fix mirrors the native pooled START: `refresh_active_roster` snapshots the pooled driving-tank count before the frame's activations/removals and emits `queue_play_sound2(TankDriving)` when it drops to 0 (`stop_rock_tank_drive_sound_on_last_exit_loc_3c46e`). The existing pure helper `deactivation_drive_sound_cleanup_loc_3C494` stays as the per-object ASM spec (its single-decrement can't express native's batch removal). Discovery via a 4-way Explore workflow; the START/STOP were initially mis-located to `activate_enemies_with_rng_inner` — they live in `step_active_enemies_with_rng`/`refresh_active_roster`. Regressions: `loc_3c46e_last_rock_tank_exit_emits_stopsequence_tank_driving_stop` + 3 (pooling 2→1 holds, silent when none drove, count filter). `verify-slice-fast` green (6183+9); full sweep 0-104 + `verify-tas-full` green (sound is advisory/CRC-neutral). Removed now-live `#[allow(dead_code)]` on `queue_play_sound2`/`QueuedSoundCommand::PlaySound2`. **UNVERIFIED**: whether `PlaySound2($1F)` audibly *halts* vs *restarts* the GEMS loop needs a real audio device (#264 headless caveat) — human to confirm the drone stops when the last tank leaves. Per-issue evidence under `#339` in archive. **NEXT: #338** (RockTank creak/shot SFX $3C/$42/$41 unported) or the #243+#333 render-flicker family.
- **Last session** (2026-06-19): **fixed #331 RockTank BLW2 wall-turn self-kill (`[ ]`→`[?]`).** Resolved the REFINED open question first: `collision_type=$FFFF` DOES kill the Tank (loc_3C464 → `jmp (a0)`=Tank.asm loc_3B41C `tst.w collision_type; bmi loc_3B480` death), so the native bit-2→terrain-kill is FAITHFUL — candidate (A) was wrong. Reproduced vs oracle (probe camera on BLW2 #03 at (1848,239)): native self-kills, oracle survives. The bug is positional: native over-advanced the tank to x=1878 on the turn frame (oracle rests at 1877) because the right-edge "extra half-step" (RightBoundaryAfterExtraAdvance, a #260 CLIFF behaviour) fired at a WALL, pulling the col-118 wall tile (a SlopeUp — `$4000` so the body scan kills, but not `Solid` so `probe_wall` ignores it) into the post-turn leftward body scan → bit-2. Fix: reverse in place (loc_3C0D2) when `wall_bit_set && moving_right && crossed_boundary && col_span*8<half_width`; guarded by the wall bit so #260's cliff path is untouched. BLW2 #03 probe now MATCHes 72/72; `verify-slice-fast` green (6179+9); full sweep 0-104 tier state 69 passed/0 failed; `verify-tas-full` CRC all 37759 frames match, 212 authoritative divergences = baseline (CRC-neutral). Regression `loc_3c0d2_rightward_wall_turn_reverses_in_place_not_extra_advance` (red without the fix). Per-issue evidence under `#331` in archive. NOTE: BLW2 start tank #01 gains a transient 2-frame facing phase at its own wall turn — a pre-existing ~1px spawn-walk lag the old fudge masked; turn position now matches oracle, sweep unaffected. **NEXT: #339** (RockTank drive-SFX stop-on-refcount-0).
- **Last session** (2026-06-19): **playtest triage — 11 new rows (#331-#341) + 4 reopens, no code yet.** Triaged a ~30-symptom human playthrough with a 15-cluster ASM+native workflow plus two focused agents (helmet-pip oracle verify; audio subsystem). Headline outcomes: (1) **#276 helmet HUD pips REOPENED — the human was right, it is a real oracle-verified bug**: the ROM draws a 5-HP Iron Knight as 5 dots and greys the trailing one on the first hit; native drops the trailing **size-0 (8x8, VISIBLE)** pip, so it shows 4→4. Prior NOT-A-BUG verdicts (and the `..._hidden_by_sub_bdc0` test) rest on a MD sprite-size-0=invisible misread — invert the test, emit the pip. (Practical gotcha: `kid.asm` has embedded NULs so plain `grep` silently misses; use `-a`. The disasm uses descriptive labels, not `loc_B700`/`sub_BDC0`.) (2) **RockTank de-confused into 3 separate things** so we stop the whack-a-mole: floor-sink = z-priority/foreground-occlusion (#259, NOT a Y bug, keep [?]); stomp-hurts = loc_1CE4 MoveID-jump gate (largely ORIGINAL, #271/#286); **BLW2 self-kill/stuck = NEW #331 regression** (57cf43bb8/410e713c0 body-overlap terrain-kill, only Robot-height-tested). (3) Slope-into-wall teleport root-caused (#249/#270, distinct from wall-cling). (4) Same-form helmet re-pickup freeze (#332). (5) Enemy flicker = render free-run (#243 reopen + #333). (6) The "USM2 tank guy" is the **Maniaxe form** (green dots = #323, not a current build?; "heavy helmet on slope" = a Juggernaut prize block) — NOT a Tank enemy; do NOT open a 0x05 Tank issue. (7) "Axe can't hit blob/dragon" is mostly not-a-bug (friendly-hit path has no invuln gate; pre-flight dragon has no collision = ROM-faithful). (8) Maniaxe can't climb slopes = ORIGINAL_QUIRK (#152, loc_9E6C). (9) Audio: voice triggers/ids ARE wired (ouch/die/bummer correct); "slow" = #264 queue-cap (resampler is rate-correct post-2f723ab8); **RockTank drive-SFX stuck loop (#339) + creak/shot unported (#338) are statically real**; diamond voice $62 unwired (#340); sound-audit tool requested (#341). Reconfirms: #259/#264/#280/#309/#323/#324/#266. Docs-only; per-issue evidence in `backlog-archive.md` (grep "2026-06-19").
- **Last session** (2026-06-18, latest+23): **fixed #330 Mode B2 blank-top-row decoder skip (`[ ]`→`[?]`).**
  `decode_level_layout` now mirrors ASM `loc_11C34` + `sub_140F6`: decode exactly the populated
  foreground height first, apply the block overlay at that height, then shift the finished grid down by
  `Number_blank_top_rows`. Logical `0x29`/resource `0x31` now decodes its 6720-byte foreground, gains two
  zero-filled top rows, and keeps solid terrain below. Regression:
  `decode_level_layout_shifts_blank_top_rows_after_loc_11c34_for_mode_b2`; focused cache-backed check and
  `mise run verify-slice-fast` green (6155). Note: the wider cache-backed
  `decode_level_layout_matches_fixture_collision_cells_loc_11c34` still exposes an unrelated level `0x42`
  high-skin/city collision-table residual; not part of #330.
- **Last session** (2026-06-18, latest+22): **fixed #329 leftover/aliased foreground render + collision (`[ ]`→`[?]`); split Mode B2 to #330.**
  Two mirrored asymmetries made leftover warp targets skip. (A render) `load_level_visuals` parsed the
  borrowed-logical foreground bytes against the mapped *resource* header (ASM `loc_11A50`) without truncating →
  now truncates to the resource grid (fixes `0x47`/`0x38`). (B1 collision) after the human warp-through (Mac, no
  `level-layout` fixtures) surfaced the remaining block-grid skips, `decode_level_layout` got the same ±5 tolerance:
  `resolve_terrain_to_expected_cells_loc_11C34` zero-pads a terrain buffer short by ≤5 to `expected_cells` instead
  of rejecting (fixes `0x2B`/`0x2D`/`0x32`/`0x39`). Smoke (real manifest): `0x47`/`0x38`/`0x32` now LOAD; `0x41`
  still skips (no foreground at all). **`0x29` still skips → #330** (blank-top-rows inflate `expected_cells` by 240,
  not the ≤5 case; needs the bigger `insert_blank_top_rows_sub_140F6` geometry change). Regressions:
  `load_level_visuals_truncates_borrowed_logical_foreground_to_resource_grid_loc_11a50`, `terrain_length_tolerance`
  unit group, cache-gated `decode_level_layout_tolerates_short_leftover_foreground_loc_11c34`;
  `mise run verify-slice-fast` green (6151).
- **Last session** (2026-06-18, latest+21): **fixed #322 render empty-frame hardening (`[b]`→`[?]`).**
  Added regressions for two renderer totality holes from the human crash lead: a present-but-empty flying-dragon
  family now falls back to the existing enemy rectangle instead of panicking on `% frames.len()`, and effect-only
  enemy sprites such as HeadyMetal split pieces skip empty loaded art instead of indexing `frames[0]`. This does
  not prove the original Under Skull Mountain 3 crash root cause; it closes the robust render path and keeps the
  issue in human validation. If the live repro still crashes, capture `RUST_BACKTRACE=1 mise run play-native` on
  Level 0x06 and file the file:line. Verification: focused `cargo test -q -p wildside-cli issue322 -- --nocapture`.
- **Last session** (2026-06-18, latest+20): **fixed #323 Maniaxe axe green-dot render (`[ ]`→`[?]`).**
  ASM `ManiaxeAxe_Init` installs `stru_8B86`, which cycles four form-art frames
  `LnkTo_unk_A589E/A5924/A59AA/A5A30`; native now traces `EnemyProjectileKind::ManiaxeAxe`
  as `ActiveProjectileKind::ManiaxeAxe`, loads `maniaxe_axe_1..4.bin` into the Maniaxe
  `PlayerFormVisuals::axe` bank with ROM `sprite_frame_unc` offsets, and renders typed axes
  as sprites instead of the generic friendly green rectangle. Engine hit delivery remains covered
  by the existing #296 Maniaxe axe tests. Verification: focused trace/render regressions,
  `cargo test -q -p wildside-core maniaxe_axe -- --nocapture`, `cargo test -q -p wildside-cli maniaxe -- --nocapture`,
  `cargo fmt --all`, and `mise run verify-slice-fast` green. Human validation: throw the axe in
  Maniaxe/Iron Knight form and confirm it is visible axe art; if enemy damage still appears wrong
  with the correct sprite, file a separate collision repro.
- **Latest playtest follow-up (2026-06-18): fixed #323 Maniaxe axe spin.**
  The previous render fix selected the right art bank but left the live axe's exported
  `animation_elapsed_frames` stuck at zero, so native stayed on `maniaxe_axe_1`.
  `step_projectiles` now advances the `stru_8B86` animation clock on each `loc_8A52`
  flight tick while preserving the zero-lifetime collision/cull semantics. Verification:
  `loc_8a52_axe_advances_stru_8b86_animation_clock` and
  `maniaxe_axe_projectile_spins_through_stru_8b86_frames`.
- **Last session** (2026-06-18, latest+19): **landed three playtest-triage fixes — #327, #326, #325 (`[ ]`→`[?]`).**
  Worked the directed triage order (skipping `[b]` #322). (1) **#327 P1 teleporter title-card**: ASM loc_BCF6 keeps
  Game_Mode=$C → Load_InGame, so teleporters reload with NO card. Extracted the shared Load_InGame loader
  (`load_level_entry_common`) out of `begin_level_entry` and added `begin_teleport_level_entry_load_ingame_loc_bcf6`
  landing in `BootState::InGame`; FBCC warp colour now stays visible to the InGame entry fade and clears at fade end.
  Cross-level already bypasses via `LevelSkipHold`. Flipped 3 StartingLevel-misread tests. (2) **#326 P3 title-card
  name**: `parse_level_name_comment_id` parsed the hex `AddrTbl_LevelNames` index comments as decimal — fixed to
  `u8::from_str_radix(_,16)`; new wildside-data test pins 0x1B/0x27. (3) **#325 P2 water palette**: theme palette path was
  numeric (`palette_fg/7.bin`) but assets are name-based — added `theme_palette_basename` (ROM-ordered name table)
  for fg+bg paths; replaced the masking numeric fixtures; real-asset Highwater regression confirms the loc_1223C band
  is non-zero. `verify-slice-fast` green (6135+); `verify-tas-full` green (advisory number_objects only). All three
  are `[?]` — human must validate visually (teleport: no card; Skydragon card name; Highwater waterfall cycles).
  Also INVESTIGATED (no code, deliberately not blind-fixed — both need work that would otherwise be a guess):
  **#323** axe sprite source pinned to `stru_8B86`→4 frames at ROM 0x0A589E + a 4-file wiring plan, blocked on
  decoding the mapping pfd offsets; **#324** confirmed the foreground/block two-plane DATA split but the RENDER
  behaviour needs jgenesis forest-level frames before reverting the scene.rs guards (blind revert risks re-opening
  the closed #289). See `#323`/`#324`/`#325`/`#326`/`#327` archive.
- **Last session** (2026-06-18, latest+18): **blocked #322 P0 crash (`[ ]`→`[b]`) — needs human repro. Docs-only.**
  First substantive investigation since the triage filed it. Disproved the archive's proposed "robust render-guard
  fix": it is **dead code**. A present-but-empty enemy family cannot occur in production — `load_enemy_visuals`
  (wildside-render/src/bundle.rs:3489) inserts a family only after `load_compressed_sprite_frames` returns `Ok` (any
  `Err`→`continue`, never inserts), that helper returns one frame per descriptor, and **no** `ENEMY_FAMILIES` spec
  has `frames: &[]` (grep over 34 specs = 0). So `families.get` is absent (→ existing `object_rect` fallback,
  sprites.rs:1512) or ≥1-frame; the `frames[0]`/`% len()` sites (sprites.rs:1494/1502; scene.rs:5602 is also
  boss-gated to 0x20/0x21) are unreachable on an empty Vec ⇒ guarding them = churn. Also re-audited the whole
  shoot→HP0→death→child chain for the 0x0C flying dragon (`EnemyFamily::Dragon`): all bounds-safe —
  `DRAGON_DIVE_SCRIPT[idx]` has a `>=len` check (enemies.rs:15356), friendly-projectile-hit loops index within
  `0..len` and remove in reverse post-loop (helmets.rs:5943/5990), death-child handling uses `retain_mut`+`get_mut`
  (combat.rs:8957/8995/8934), and the 0x0C death-frame arm is `len>=4`-guarded (sprites.rs:1372; its fallthrough is
  `% len()` = divisor-zero, not index-OOB). No safe blind fix exists and the exact `[0]`-on-empty-Vec site cannot be
  pinned statically (this audit + the original 15-agent triage both failed). Pinning needs `RUST_BACKTRACE=1
  mise run play-native` on Level 0x06 (interactive: Maniaxe form, jump, shoot a flying dragon) → needs a display the
  headless loop lacks. Blocked on that human run per the legend's "needs human repro"; once the backtrace gives the
  file:line, the fix is a targeted guard there + a regression — not the speculative render-guard sweep. See `#322` archive.
- **Last session** (2026-06-18, latest+17): **#309 shifting-block rise-table pinning committed**.
  Iteration had started on the #309 blocks umbrella (top open row before the latest+16 triage landed
  mid-session); completed it as proactive verification since #309's only open behavioural slice stays
  oracle-gated (type-8 Shooter). Discovery via `cargo mutants -f blocks.rs --list` (the full mutation
  run is blocked in this sandbox; `--list` is fast/allowed, hand-mutation is the documented reliable
  verifier) cross-referenced against the test files. Found the one block-bounce y-delta table with no
  exact-value guard: `SHIFTING_BLOCK_RISE_Y_DELTAS_LOC_113F4` (`[-2; 8]`, blocks.rs:360). Its metal
  siblings `…LOC_1117A/11196` and the prize-wobble table are all pinned
  (level_tests.rs:19807/19832/19837), but the shifting table feeds only the `BlockBounceKind::Shifting`
  arm of the `block_bounces` accumulation loop (blocks.rs:4342), and the only `shifting_block*` tests
  (runtime_tests.rs:7188/7212) assert `has_block` presence — never the rise `y_offset_px`. ASM-confirmed
  against sub_113C0/loc_113F4 (kid.asm:23332-23339): `move.w #7,d0; subq.w #2,y_pos(a3); dbf d0` = eight
  unconditional 2px-upward steps; the cell-above worker spawns at `y_offset_px = 16` (blocks.rs:3744) and
  the all-negative deltas pull it to exactly 0 as the row swap commits. New
  `shifting_block_rise_y_deltas_match_asm_loc_113f4` (level_tests.rs, beside the metal-table test) pins
  the table `[-2;8]`, the monotonic cumulative descent to -16, and the +16→0 landing. Genuine-gap proof
  by hand-mutation: flipping the first entry `-2 → 2` makes ONLY the new test fail — all 8 pre-existing
  `shifting_block*` tests still pass. `verify-slice-fast` green (6134+9); test-only ⇒ no `verify-tas-full`.
  #309 stays `[ ]` (umbrella — same residual: only the two oracle-gated type-8 Shooter deviations remain;
  the block-bounce y-delta table family prize/metal/shifting now has no mutation gap). See `#309` archive.
- **Last session** (2026-06-18, latest+16): **playtest triage — 7 new rows + 6 reconfirms, no code yet**.
  Triaged a human playtest with a 15-agent parallel workflow (per-symptom ASM+native+git+backlog dedup) plus an
  adversarial crash verify. **Actionable order for the next sessions (all `[ ]` unless noted):**
  1. **#322 P0 CRASH** — shooting a flying dragon on Under Skull Mountain 3 (Level 0x06) panics
     `index out of bounds: the len is 0 but the index is 0`. Root cause UNPINNED: the first agent's site
     (sprites.rs:1502) was REFUTED by adversarial verify (0x0C frames are statically len-9); dragon_script
     `[0]`/`[cmd]` (DRAGON_SCRIPT_* non-empty), boss-eye/HeadyMetal `[0]` (fixed arrays, boss-gated), and
     enemy-family render `frames[0]` (loader never inserts an empty family) are all ruled out. Needs
     `RUST_BACKTRACE=1 mise run play-native` on 0x06 to pin the line; the robust fix (guard the render
     `frames[0]`/`%len()` sites + skip empty-frame family inserts) holds regardless. See #322 archive.
  2. **#327 P1 teleporter shows the title-card loader** — fully ASM-traced + a clear fix: ROM keeps
     Game_Mode=$C (Load_InGame, no card) via FC29 at loc_BCF6; native routes through BootState::StartingLevel.
     BOTH same- and cross-level teleporters should skip the card. Flip the test that encodes the ASM misread
     (combat_tests.rs:2386). Most actionable of the batch.
  3. **#324 P1 block/terrain regression** — fixed candidate refined again: static live `$E7xx`/block cells follow
     ASM `sub_302F6`/`sub_3034A` and draw only Block_Mappings art, and active temporary `$6Fxx` prize-wobble
     cells are owned by ASM `sub_1007A`'s copied `unk_E0E4E`/`unk_E0E5E` object rather than leaking immutable
     foreground skin through transparent pixels. Needs human retest on a forest tree/block.
  4. **#289 P1 Skull Mountain 3 moving platforms** — fixed candidate: `PlatformScript_*-
     PlatformScript_BaseAddress` placement refs now normalise to the ASM `$A(a4)` relative script word, and stale
     absolute generated-cache offsets are repaired at runtime. Retest the Skull Mountain 3 climb.
  5. **#325 P2 water-palette regression** — fixed candidate: theme palette assets are name-based
     (`palette_fg/mountain.bin`), and live level entry now resolves the ROM-ordered theme basename before seeding
     `Palette_Buffer`. Human retest still needs to confirm the Highwater waterfall palette cycles in native play.
  6. **#323 P2 axe green dot** — fixed candidate updated: live Maniaxe/Iron Knight axes now trace as
     `ActiveProjectileKind::ManiaxeAxe`, use `stru_8B86`/`maniaxe_axe_1..4.bin` art, and advance the animation clock.
     Human retest still needs to confirm the thrown axe visibly spins.
  7. **#326 P3 title-card name** — fixed candidate: `parse_level_name_comment_id` now treats AddrTbl_LevelNames
     comments as hex, so rendered glyph cards are keyed correctly from 0x10 up. Human retest still needs to confirm
     Skydragon Castle 1 and later cards visually.
  8. **#328 `[-]`** — Cyclone "straight up / no spin / too powerful" is ORIGINAL_QUIRK (flight is y-only,
     spin is sprite-anim, no altitude cap). One cheap human check: held-C must not keep rising.
  **Reconfirms (no new rows):** #289 (appear-then-snap + hell wrong-start + dark-green sky + death-wall — the
  human evidence #283/#266/#280 were waiting on; murder-wall init verified faithful), #276 (HP "4->4->3"
  RE-CONFIRMED NOT-A-BUG: sub_BDC0 hides the odd 5th pip), #259 (RockTank floor-sink = z-priority not Y; head-stomp
  dispatch faithful per #193; tank "green dots" not reproduced), #264 (audio cluster: queue-cap frame drops on
  bursts; needs a real audio device — covers #265/#272/#287), #239 (walking dragon off-ledge = settled
  ORIGINAL_QUIRK), #309 (blocks "flip to empty" = evanescent, transient-passable by design; needs human to name
  the block). Full evidence per `#NNN` in `backlog-archive.md` (grep `2026-06-18 playtest triage`). Docs-only.
- **Last session** (2026-06-18, latest+15): **#309 evanescent-block helper pinning committed**.
  Recovered the prior (uncommitted) WIP — two exact-value tests for the type-9 evanescent-block
  restore/fire helpers — and verified both from scratch before committing. (1) `evanescent_collision_overlay_enabled_loc_ED52`
  (blocks.rs:2549): ASM-confirmed against loc_ED52 kid.asm:19240-19255 — the second `lsr.w #1,d0; bcs loc_ED7C`
  takes the theme-collision-overlay branch only when original action bit 1 (`$02`) is set; bit-1-clear
  falls through to plain `+$6000` solid. Native folds the hibernate cadence into the #214 Highwater timer
  threshold. The helper had **no direct test caller**, so its `&`/`&&`/`!=`/`>=` operators were unpinned;
  new `evanescent_collision_overlay_enabled_loc_ED52_matches_asm_action_bit1_and_timer` (combat_tests.rs)
  pins the truth table incl. real `$83`/`$80`/`$81` actions + below-threshold timer. (2) `fire_evanescent_block_projectiles_loc_EC76`
  (blocks.rs:2684): ASM-confirmed against loc_EC76 kid.asm:19124-19144 — `move.w #3,d4; btst d4,d3; dbf d4`
  fires bits 3,2,1,0 descending, d4=direction word; velocity is the round-trip inverse of `…loc_EDFA`
  (0=up,1=right,2=down,3=left). New `fire_evanescent_block_projectiles_loc_EC76_direction_table_matches_asm_bit_order`
  pins exact per-bit (xv,yv) tuples + emit order (left/down/right/up). Mutation over both helpers
  (`/tmp/mutants-evanescent-309`): **23/23 caught, 0 missed**; genuine-gap proof by hand-mutation (the
  documented `--re` caveat means the count alone isn't decisive) — `&0x02→&0x01` and the EC76 bit-3
  `-speed→speed` are caught by **only** the two new tests, no non-`demo_`/`snapshot` pre-existing test.
  `verify-slice-fast` green (6133+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella — same
  residual: only the two oracle-gated type-8 Shooter deviations remain, no narrow behavioural slice; the
  evanescent-block restore/fire helper family now has no mutation gap). Next top open table row is still
  **#309** then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+14): **#309 ice-block ABOVE arm + shooter viewport-boundary
  pinning committed**. Recovered the prior (uncommitted) WIP — two exact-value mutation pins for
  `blocks.rs` helpers — and verified both from scratch before committing. (1) ABOVE arm (`d6==2`,
  `loc_10FDC`) of `ice_block_destroy_effect_origin_sub_10F9C` (blocks.rs:2124): ASM-confirmed against
  sub_10F9C kid.asm:22881-22908 (`d6==2 → subq.w #4,y_pos(a3)`; 3→right/1→left/else→down). The three
  other arms had exact-value tests but ABOVE did not, so deleting that match arm fell through to the
  default down nudge and survived. New `ice_block_effect_sub_10f9c_uses_loc_10fdc_upward_offset`
  (runtime_tests.rs) drives spawn + one `step_visual_effects` for ABOVE (80,48→80,44); cargo-mutants
  over the helper now **12/12 caught, 0 missed** (incl. the previously-surviving ABOVE-arm delete).
  (2) Coarse viewport gate `shooter_horizontal_span_rejected_loc_F830` (blocks.rs:2840): ASM-confirmed
  the four-test reject chain against loc_F7E2..loc_F808 kid.asm:20468-20480 (d4=left col, d5=top row,
  d6=right col, d7=bottom row; rejects row>bottom, row<top, start_col>right, or start_col-left<=-8).
  The pre-existing loc_F830 test only exercised start_col>right, so the row vertical edges, the
  inclusive top/bottom rows, and the `<=-8` left-margin boundary were unpinned. New
  `shooter_horizontal_span_rejected_loc_f830_matches_asm_coarse_viewport_boundaries` (combat_tests.rs)
  pins them; `--re` is unreliable in this cargo-mutants setup (documented tooling note — it matched
  `ice_block_bullets` instead), so verified by hand-mutation: the test catches `&&`→`||` (row),
  `>`→`>=` (left margin), and `-`→`+` (subtraction). `verify-slice-fast` green (6131+9); test-only ⇒
  no `verify-tas-full`. #309 stays `[ ]` (umbrella — same residual: only the two oracle-gated type-8
  Shooter deviations remain open, no narrow behavioural slice; the ice-effect-origin and shooter-span
  helpers now have no mutation gap). Next top open table row is still **#309** then **#310** (terrain).
  See `#309` archive.
- **Last session** (2026-06-18, latest+13): **#309 shooter-block support direction-bit pinning committed**.
  Recovered the prior (uncommitted) WIP — an exact-value test for the per-direction neighbour-bit
  dispatch in `shooter_has_support_loc_f106` (blocks.rs:443) — and verified it from scratch before
  committing. ASM-traced all four directions against `sub_F096` loc_F1DE..loc_F248 (kid.asm:19775-19822):
  `d6=(word&$FF)-$2D`, then a fixed `lsr.w #1; bcc` chain assigns bit0=up (`a1-width`), bit1=right
  (`3(a1)`), bit2=down (`a1+width`), bit3=left (`-1(a1)`, `beq`); a set bit whose neighbour skin byte
  is non-zero marks the slot negative. The helper was private with no direct test — every prior
  `grid_with_shooter_blocks` test drives only the live scan with a single direction byte, so the
  bit→neighbour pairing arms had no exact-value pin. New regression
  `shooter_has_support_loc_f106_matches_asm_per_direction_neighbour_bits` pins each enabled direction
  vs its own skinned neighbour, the clear-neighbour negative, and the cross-direction negative (skinned
  neighbour whose bit is NOT set). Made the helper `pub(crate)` for the direct call. Mutation re-run
  (`/tmp/mutants-shooter-support-309`): **37 mutants, all 37 caught, 0 missed** (27 in the helper, all
  caught; the 10 incidentally-matched `ice_block_bullets_loc_11052` were already pinned).
  `verify-slice-fast` green (6129+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella —
  only the two oracle-gated type-8 Shooter deviations remain open; the ice-bullet, metal-push,
  block-debris, and now shooter-support helper families have no mutation gap). Next top open table row
  is still **#309** (no narrow behavioural slice left) then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+12): **#309 block-debris velocity-table pinning committed**.
  Recovered the prior (uncommitted) WIP — an exact-value test for the per-direction `sub_292E`
  shard velocity tables — and verified it from scratch before committing. ASM-verified all four
  tables byte-for-byte against the `dc.b` data in kid.asm (dispatch 4893-4918; tables 4949-5048):
  base `unk_29E8`/`unk_29F2`=`[0,0,8,8,4]`/`[0,8,8,0,4]`; ABOVE(2) x`[-1,-1,1,1,1]` y`[-4,-3,-3,-4,-5]`;
  BELOW(0) x`[-1,-2,1,1,-1]` y`[-2,0,0,-1,-2]`; LEFT(1) x`[-5,-4,-3,-2,-1]` y`[-3]×5`;
  RIGHT(else) x`[5,4,3,2,1]` y`[-3]×5` — all match the native tables. Only the ABOVE direction had
  an exact-shard test before, so deleting the BELOW/LEFT match arm (fall through to RIGHT default)
  survived mutation. New regression `block_debris_velocity_tables_match_asm_sub_292e_all_directions`
  pins all five shards per direction for BELOW/LEFT/RIGHT. Mutation re-run over the helper + swarm
  builder (`/tmp/mutants-debris-309`): **17 mutants, 16 caught, 0 missed, 1 unviable**.
  `verify-slice-fast` green (6128+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella —
  only the two oracle-gated type-8 Shooter deviations remain open; the ice-bullet, metal-push, and
  now block-debris helper families have no mutation gap). Next top open table row is still **#309**
  (no narrow behavioural slice left) then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+11): **#309 ice-bullet directional-table pinning committed**.
  Recovered the prior (uncommitted) WIP — an exact-value test for the per-direction ice-bullet
  offset + velocity tables — and verified it from scratch before committing. ASM-traced all four
  directions against `IceBlock_BulletPosOffset_Index` (kid.asm:22990), confirming the table holds
  *incremental* (x2,y2,x3,y3) deltas applied via `add.w (a1)+,d1/d2` at `loc_11052`, so bullet 3 =
  centre + (t0+t2, t1+t3): up `(4,4)`, right `(-4,4)`, down `(-4,-4)`, left `(4,-4)`. Velocity is the
  exact round-trip inverse of `shooter_object_direction_from_velocity_loc_EDFA`. The three prior
  ice-bullet tests only exercise remapped dirs 0/2 and never assert exact offsets/velocity, so the
  **left arm** (dir 3 third-bullet y-sign + left velocity x-sign) and the exact dir-2/3 tuples survived
  mutation. New regression `ice_block_bullet_offsets_and_velocity_match_asm_iceblock_bulletposoffset_index`
  pins all four directions. Mutation re-run over both helpers + wrapper (`/tmp/mutants-ice-dir-309`):
  **48 mutants, all 48 caught, 0 missed**. `verify-slice-fast` green (6127+9); test-only ⇒ no
  `verify-tas-full`. #309 stays `[ ]` (umbrella — only the two oracle-gated type-8 Shooter deviations
  remain open; the ice-bullet + metal-push helper families now have no mutation gap). Next top open
  table row is still **#309** (no narrow behavioural slice left) then **#310** (terrain). See `#309` archive.
- **Last session** (2026-06-18, latest+10): **blocked #308 (`[ ]`→`[b]`)** — contact-box vein
  provably exhausted. New angle vs the prior flip/transcription passes: a complete systematic
  re-audit of all 27 `*_contact_bounds` helpers (combat.rs:11595-12287) against their cited ASM
  `CollisionSize_Index` rows, re-deriving the `SubBoxLoop` arithmetic (unflipped right=left+width,
  bottom=top+height, kid.asm:3342/3350; flipped mirror `(-(L+W),-L)`, kid.asm:3354-3370) for every
  single-row helper. **Zero actionable narrow slice**: every single-row box (Fireball/Sphere/Drips/
  TarMonster/SpinningTwins/UFO/Tornado/Robot-head/FireDemon/Armadillo/DragonWalking) is arithmetically
  correct, all multi-row families are documented union approximations with mirror-consistent flip arms,
  and coverage is complete — every family has an exact-value box-edge `*_contact_uses_rom_collision_size_row`
  test (all four edges), asymmetric families have exact `*_right_facing_flips_via_loc_1be4` tests, and
  zero/death rows are pinned. No bug, no test gap. Tooling note: `cargo mutants -f combat.rs --re 'contact_bounds'`
  does NOT scope this large file (`--re` ignored, runs whole file) — do not retry. Remaining #308 work is
  uniformly oracle-gated/architectural (per-sprite-frame `CollisionSize` for union families, the invented
  generic 8×16 default + MiniHoppingSkull arms, P3 projectile-CollisionSize CRC-gated wiring, P3 generic
  `colid` side/below delivery) — identical in kind to the basis on which #306/#307 were blocked. Now waits
  on a human to schedule the per-sprite-frame hitbox refactor or supply an oracle route. Docs-only (block).
  Next top open table row is **#309** (blocks umbrella — mutation-clean as of latest+7), then **#310** (terrain).
  See `#308` archive for the full per-helper arithmetic verification.
- **Last session** (2026-06-18, latest+9): **#308 RockTank contact-box flip-arm pin committed**.
  Found the gap the flip-arm pass (73b760cbc) missed: the **RockTank** (object_type 0x0A/0x0B)
  contact box is built **inline** in `enemy_contact_bounds` (combat.rs:11618-11637), not in a
  `*_contact_bounds` helper, so it was skipped — and even its unflipped arm had no exact-value
  test (the one Tank contact test asserts only a `Damage` result, never edges). Verified the row
  byte-for-byte in kid.bin: `unk_31A26`/`unk_31A54` both read `fff3 0019 fff0 0010` =
  `dc.w -$D,$19,-$10,$10`; SubBoxLoop (kid.asm:3335-3374) → unflipped `(-13,12,-16,0)`, flipped
  `(-12,13,-16,0)`. New regression `rock_tank_contact_box_facing_flips_via_loc_1be4` pins both
  arms. `verify-slice-fast` green (6126+9); test-only ⇒ no `verify-tas-full`. #308 stays `[ ]`
  (umbrella — remaining open work is sprite-link-aware per-frame hitboxes + the
  Drips/MiniHoppingSkull/HeadyMetal gaps, neither a narrow unattended slice without an oracle
  route). Next top open table row is **#309** (mutation-clean), then #310 (terrain). See `#308` archive.
- **Last session** (2026-06-18, latest+8): **#308 contact-box flip-arm pinning committed** (73b760cbc).
  Recovered the prior (uncommitted) WIP — 11 right-facing contact-box pins — and verified it from
  scratch before committing. ASM mirror confirmed against `GfxObjects_CollisionKid_SubBoxLoop_Flipped`
  (kid.asm:3354-3370): unflipped offsets `(L, L+W)` mirror to `(-(L+W), -L)` = `(-right, -left)`,
  top/bottom unchanged. Every prior contact-box test used `test_enemy` (defaults `facing=Left`), so the
  flip-asymmetric families' `Direction::Right` arm had **no** exact-value regression — a sign/value slip
  there would survive. New pins (Goat walk/damage, Crab, Armadillo, Lion, Scorpion walk + sting-tail
  child, Robot parent + linked-head child, Tornado live + death-active, DragonWalking damage) each assert
  the exact mirror of the already-ROM-verified (2026-06-13) unflipped row; all 11 pass. `verify-slice-fast`
  green (6125+9); test-only ⇒ no `verify-tas-full`. #308 stays `[ ]` (umbrella — remaining open work is
  sprite-link-aware per-frame hitboxes + the Drips/MiniHoppingSkull/HeadyMetal gaps, neither a narrow
  unattended slice without an oracle route). Next top open table row is **#309** (blocks umbrella, mutation-clean),
  then #310 (terrain). See `#308` archive for the full mirror derivation.
- **Last session** (2026-06-18, latest+7): **#309 metal-push survivor pinning committed**. Took the
  latest+6 handoff and ASM-verified the 4 flagged `metal_block_push` mutation survivors against
  kid.asm:23130-23284. Pinned the **3 genuinely real** ones with exact-value tests (runtime_tests.rs):
  `4067 < -> <=` (scan_right right-edge bound — metal run to the edge selects no block, `<=` panics),
  `4099 || -> &&` (scan_left abort on solid non-metal block — only column-zero was tested before),
  `4242 == -> !=` (step direction-select gates x_pos +4/-4 SIGN — the Berzerker integration test only
  asserts column hand-off, never x_pos). The 4th, `4258 < -> <=`, is an **equivalent mutant** (handoff
  guess was wrong): at `col+1==width` both branches yield the same `cell_at`-clamped empty `(0,0)`, and
  it's short-circuited by the gate's width check anyway. The `4113 | -> ^` pair is **proven equivalent**
  (bit-disjoint `0x80`/`collision<<4`/`0x0F` for all CollisionKind ≤ Spikes=7). Re-run over the 3
  functions (`/tmp/mutants-metal-push-309`): **34 caught, 1 missed** (only the 4258 equivalent).
  `verify-slice-fast` green (6114+9); test-only ⇒ no `verify-tas-full`. #309 stays `[ ]` (umbrella): only
  the two oracle-gated type-8 Shooter deviations remain open (0 shipped cells; prior removal broke 18
  tests — do not retry without a type-8 ceiling-headbutt oracle probe). The metal-push helper family now
  has no non-equivalent mutation gap. Next top open table row is **#308** (combat umbrella, still `[ ]`),
  then #309 (now mutation-clean) and #310 (terrain) — see `#309` archive for the full survivor breakdown.
- **Last session** (2026-06-18, latest+6): **#309 verification slice committed**. The blocks audit's
  confirmed behavioural gaps are all RESOLVED candidates; the only open audit rows are the two type-8
  Shooter deviations (oracle-gated, 0 shipped cells, prior removal broke 18 tests — do not retry). With
  no narrow behavioural slice, ran proactive mutation testing over `blocks.rs` ice-bullet + metal-push
  helpers (`.cache/mutants/metal-push`, 65 mutants, 10 missed). **Closed** the 4 ice-bullet survivors:
  `ice_block_bullets_loc_11052` sets four non-`Default` shooter-tracking fields (`shooter_tracks_layout=true`,
  layout col/row=source, `shooter_state_word=$FFFE`) mirroring the canonical `shooter_block_loc_EC76`
  bullet (ROM models ice bullets as ShooterObject bullets that track their source cell). New regression
  `ice_block_bullets_loc_11052_track_source_layout_cell_like_shooter_object_bullets` pins all three bullets;
  re-run reports 10 caught / 0 missed (`.cache/mutants/ice-bullets-fields`). `verify-slice-fast` green (6111+9);
  test-only. #309 stays `[ ]` (umbrella). **Handoff:** 6 metal-block-push survivors remain (`blocks.rs:4067/4099/4242/4258`
  real; `4113×2 | vs ^` likely equivalent) — next #309 iteration ASM-verifies and pins the 4 real ones; see `#309` archive.
- **Last session** (2026-06-18, latest+5): **#308 contact-box transcription fix committed**. Recovered the
  prior (uncommitted) WIP on the combat umbrella and *verified it from scratch* before committing. Two
  enemy contact boxes were transcribed wrong against the `GfxObjects_CollisionKid_SubBoxLoop` convention
  (right = left + width@`2(a3)`, bottom = top + height@`6(a3)`, confirmed at kid.asm:3342/3350): Sphere
  `unk_31AB0`/`unk_31ABA` (`fff9 000e fff9 000e` in kid.bin = `-$7,$E,-$7,$E`) had bottom clamped to 0,
  halving the 14×14 box centred on the ball's `$1E(a2)` anchor → corrected to +7; Drips parent
  `unk_3189A` (`fffc 000a fffe 0002` = `-$4,$A,-$2,$2`) had right=5/bottom=-1 → corrected to right=6/bottom=0.
  Both rows read byte-for-byte from `kid.bin`. Verification: `verify-slice-fast` green (6110+9); `verify-tas-full`
  reports **CRC all 37759 frames match, exit 0**, with an identical divergence profile to HEAD (212 pre-existing
  authoritative checkpoint divergences exist on baseline too — confirmed by a stash-and-rerun A/B) so the change
  is **CRC-neutral** on the published TAS. #308 stays `[ ]` (umbrella — many per-sprite-frame hitbox gaps remain).
  Next top open item is **#309** (blocks umbrella).
- **Last session** (2026-06-18, latest+4): **blocked #307 (`[ ]`→`[b]`)** per the loop anti-spin rule (2nd
  dead-end note, no genuinely new *outcome*-producing narrow angle). New angle vs the prior three runs:
  mutation-tested the freshly-wired (2026-06-17) **live `Init_SpecialEffects` storm machine** — the
  `weather.rs` helpers reached from live `init_weather` for weather types 2/3, untouched by the earlier
  level.rs island/lava/blank-top-rows runs. `cargo mutants -f weather.rs --re 'loc_14C..|loc_14D..|init_weather'`
  (`.cache/mutants/weather-storm`) → **39 mutants, 29 caught, 10 unviable, 0 survived**. Every viable mutant
  (the `fill_count = loop_count+1` slot count, the `4*8*8*4` written-longword product factors, the rain/hail
  bool→enum branches) is killed by the exact-value tests in `combat_tests.rs:22228..27396`. This is the **4th
  consecutive clean mutation result** on #307's post-load helper families (blank-top-rows-after, island,
  lava-rotate, storm — 0 survivors across all four), so the subsystem has no behavioural test gap left to
  close. The remainder is identical in shape to #306: only architectural gaps with real fidelity payoff remain
  (`EnemyStatus_Table`/`Manage_EnemyLoading` streaming, `off_7B1EC`/`loc_3050A` BG scroll, `loc_11E48..`
  theme-art VRAM) — none a narrow unattended slice, each needing a dedicated session **and an oracle route**.
  #307 now waits on a human to schedule that or supply an oracle route. Docs-only (block). Next top open item is **#308** (combat umbrella).
- **Last session** (2026-06-18, latest+3): **#307 verification-slice scan — dead end, no commit**. Followed the
  prior session's flagged angle (harden tests on the **live** post-load palette/object helpers). Mutation-tested
  both relevant live slices: `.cache/mutants/level-loading-live` (`classify_post_load_object_routine_loc_11F96`,
  `classify_island_background_object_loc_11FB0`, `select_level_start_music_loc_11FD2`/`select_foreground_theme_music_loc_12004`,
  the three Island palette-cycle bands `loc_12098/12110/12188`, live `step_island_shore_cycle_loc_1202a`) →
  **31 mutants, 25 caught, 6 unviable, 0 survived**; and `.cache/mutants/lava-rotate` (live `rotate_palette_lava_water_loc_1223C`
  4-word rotation + `step_rotate_palette_lava_water_loc_1223c` reload branch) → **15 mutants, all 15 caught, 0 survived**.
  No gap — every viable index/`==`/`&`-wrap mutant is killed by the existing exact-value tests. So the live post-load
  palette/object slice is already fully pinned; the only remaining #307 work is the architectural dead-islands
  (theme-art VRAM decompress, BG scroll, block-mapping tilemap, `EnemyStatus_Table`/`Manage_EnemyLoading` streaming),
  none a narrow unattended slice without an oracle route. First dead-end note on #307, left `[ ]` (not blocked — other
  live-helper verification slices remain viable). Docs-only (the `cargo mutants` unmutated baselines reconfirm the
  wildside-core lib suite green on HEAD). Next top open item is **#308** (combat umbrella).
- **Last session** (2026-06-17, latest+2): **#307 verification slice committed**. Recovered the
  prior (uncommitted) test-hardening on the live `sub_140F6` blank-top-rows shift: 4 exact-value
  unit tests in `terrain.rs` pinning `BlockGrid`/`BlockList::with_blank_top_rows_sub_140F6`. Re-verified
  the mutation gap is closed — `cargo mutants` over both methods now reports **8 mutants, 6 caught,
  2 unviable, 0 survived** (matches the archived before=0-caught/after=all-caught run). `verify-slice-fast`
  green (6110+9). Test-only ⇒ `verify-tas-full` not required. #307 stays `[ ]`: remaining gaps are
  rendering/VRAM dead-islands + the `EnemyStatus_Table`/`Manage_EnemyLoading` streaming machine
  (architectural, needs an oracle route) — no narrow unattended gameplay slice. Next top open item is **#308** (combat umbrella).
- **Last session** (2026-06-17, latest+1): **blocked #306 (`[ ]`→`[b]`)** per the loop anti-spin
  rule (2nd dead-end note, no new narrow angle). New angle vs the prior static walk: mutation-tested
  the pure collision-resolution helpers in `platforms.rs` (regex over `overlap_resolution|push_magnitude|
  platform_carry|apply_horizontal_platform_push|platform_landing|…`, output `.cache/mutants/platforms-pure2`)
  → **24 mutants, 11 caught, 13 unviable, 0 survived**. The 13 unviable are `Default::default()` subs on
  no-`Default` return structs; every viable sign/compare/branch mutant is killed by the exact-value tests
  in `block_tests.rs:16016-16216`. So the platform collision math has no test gap; the only remaining #306
  work is the architectural Vec→Platform_RAM linked/free-list refactor — non-behavioural, no oracle-observable
  payoff on swept levels, gate-risk only, a dedicated refactor session not an unattended slice. #306 now waits
  on a human to schedule that. Docs-only (block). Next top open item is **#307** (level-loading umbrella).
- **Last session** (2026-06-17, latest): **blocked #301 (`[~]`/`[ ]`→`[b]`)** per the loop anti-spin
  rule (2nd dead-end note, no new narrow angle). Confirmed the last plausible unpinned pure-function
  slice — the `sub_37AF0` 8-direction compass eye-attack sprite selector — is already faithful
  (`boss_eye_attack_sprite_sub_37332`, combat.rs:2956) AND already comprehensively pinned
  (`boss_eye_attack_sprite_sub_37332_matches_sub_37af0_direction_buckets_and_diamond_override`,
  weather_tests.rs:207). The three remaining audit rows (#136/#137/#142) are all architectural
  first-class object-slot scheduler lifecycle work with **no boss oracle (#205)** to verify — blind
  implementation would breach "never guess / oracle is truth". #301 now waits on #205. Docs-only
  (block). Then investigated **#306** (platforms umbrella) → **first dead-end note, left `[ ]`**: every
  behavioural row in `audits/platforms-fidelity-audit.md` is FIXED/partial; the live airborne collision pass
  already loops/accumulates/restarts across all platforms (`platforms.rs:1596-1635`); the only open
  work is the **non-behavioural** Vec→explicit-Platform_RAM-linked-list representation refactor (no
  oracle-observable payoff on swept levels, gate-risk only) plus the no-op `ext.l` index sign-extension
  (verified no-effect-on-base-ROM, offline asset build). No narrow unattended slice — needs a dedicated
  no-behaviour-change refactor session (sweep-green as the only check). Audit lines 82/111 cites are
  stale (line 82 is now the live `has_contact`). Next top open item is **#307** (level-loading umbrella).
- **Last session** (2026-06-17, later): #301 narrow-slice scan — docs-only, no gameplay patch.
  The two ASM-named narrow gaps in `audits/bosses-fidelity-audit.md` are both unactionable: the
  `sub_370D2` odd-frame `sfx_Plethora_x` timeout variant is **already faithful and tested**
  (corrected the stale audit prose), and the `sub_3A292` `palette_line` 2/3 flash is
  **render-gated, not engine-gated** — the live path already computes `next_palette_line` but
  the renderer loads only one CRAM row per enemy variant, so there is no row-3 boss-art data,
  and faithfully replacing the (deviating) white-flash needs the boss's 4-row palette + a boss
  oracle (#205). The `loc_3724A`/`sub_3728C` tail is object-context-ambiguous without macro
  defs. Recorded all three as the first explicit dead-end note under #301 in the archive.
  #301's narrow verifiable slices are exhausted; remaining work is the large first-class
  per-piece/eye object-slot scheduler lifecycle (architectural, no boss oracle to verify).
  **Next session on #301**: if no genuinely new narrow angle, set `[b]` per the anti-spin rule
  (this was the 1st dead-end note; a 2nd triggers the block).
- **Last session** (2026-06-17): blocked #260 (`[~]`→`[b]`) per the loop anti-spin rule —
  every synthetic route/bootstrap axis from 2026-06-07..17 is exhausted (Kid/block bounce
  near-misses only, Drips child always vertically separated, oracle often dead before the
  window); it now waits on a fresh human-equivalent live recording reaching the actual
  slime/metal-block contact. Then worked #301: audited the accessible HeadyMetal death
  routine `sub_37708` (kid.asm:39503) and found it faithfully implemented; the only real
  gap was test coverage of the `unk_3795E` piece-selection order (pinned at 3/37 phases),
  so added `heady_metal_breakup_step_sub_37708_selects_pieces_in_unk_3795e_order` pinning
  all 37 phases (verified byte-for-byte vs `unk_3795E` at kid.asm:39723; mutation-checked).
  `verify-slice-fast` green. Note for next session: the 0x37000 boss code lives in
  `kid.asm` keyed by `kid.asm:<line>` comments, NOT `loc_37708`-style labels — grep by the
  line number cited in code comments. #301's remaining gaps are the large architectural
  first-class per-piece/eye object-slot scheduler lifecycle, not narrow slices.
- **Next**: top open item is now #309 (P2 blocks umbrella, mutation-clean as of latest+7); #308 is `[b]` (contact-box vein provably exhausted by a full 27-helper arithmetic re-audit — remaining work is the oracle-gated/architectural per-sprite-frame `CollisionSize` hitbox refactor + projectile/colid oracle slices). #307 is `[b]` (behavioural verification slices exhausted — 4 consecutive clean mutation runs; architectural enemy-streaming/BG-scroll/theme-art VRAM remains, needs a dedicated session + oracle route). #306 is `[b]` (architectural linked-slot refactor only — needs a dedicated no-behaviour-change session, mutation-confirmed no collision-math test gap); #301 is `[b]` (waits on #205 boss oracle); #260 is `[b]` (human recording).
- **Earlier — last session** (2026-06-10): #290 TAS frame_7800 object-count divergence proposed fixed.
  Root cause was Dragon.asm loc_3CE90/loc_3CF90 walking-dragon breath child counting,
  not the suspected tank-pair collapse. Frames 7790..7810 match, the full
  published TAS now passes all 37759 frames by CRC, and `verify-slice-fast` is
  green.
- **Audit programme** (2026-06-10/11): ten new fidelity audits landed (blocks/combat/
  terrain/projectiles f307e9c4; shell/scoring/camera/level-flow/bosses/audio 12c81289)
  plus a staleness re-check of the six May audits (b2029fcd: 82 of 258 fixed, 140 still
  open; platforms 5/40 and level-loading 9/55 barely touched). 177 new adversarially
  verified gaps. Promoted #292-#300 as individual P1s, #301 bosses cluster, #302-#304
  P2s, #305 interactive-lane verification, #306-#320 per-doc umbrellas. Per-gap
  evidence lives in the audit docs (each ends with backlog-candidate rows), not the
  archive; the staleness report carries fixed/still-open verdicts with file:line.
- **Latest playtest, 2026-06-14**: latest pushed build still has several human-visible
  failures despite green focused gates. Most urgent: first-level stair/slope jump
  behaviour is still wrong in both directions (holding right into right stairs then
  jumping fires Kid up/right about 45 degrees; holding left into left stairs barely
  moves left after a delay). Treat this as fresh #258/#277 evidence and build a
  human-equivalent oracle route before another narrow helper patch. Other
  reconfirmed items: intro jacket appears before throw-off (#288/#161), level starts
  flicker / Kid briefly appears in the air (#122/#238/#289), block/diamond/helmet SFX
  still lag or deform music (#264/#272), Heavy Man top metal-block ding may be missing
  (#272/#297), helmet-guy hitting Dragon still appears to leave HP at 4 (#276/#271
  recheck), EOL countdown sound continues past zero until BLW2 intro (#265), BLW2
  RockTanks render half through floor near tunnel/teleporter (#259/#252), same-level
  teleporter visibly shows loader instead of only the teleporter effect (#298/#300),
  Samurai transform/attack audio is weak (#272/#287), Highwater Rhino/Berzerker still
  draws in front of water/foreground (#256/#215), Highwater Pass 2 hands flicker and
  diamond enemies are harmless (#185/#177), Level 0x08 death wall now appears but
  overlays the timer and continue screen is off-centre (#266/#303), death-wall level
  still has dark-green/cloud overlay artefacts (#280/#289), and later intro/Cyclone
  prize route still gives a Diamond and blocks progress (#284/#274/#273).
- **Next**: #258/#277 first-level stair/slope jump repro, then #284/#274 Cyclone prize
  if the movement route does not yield an actionable oracle split.
- **#258/#277 repro update (2026-06-14)**: added named probe
  `level00-slope-right-jump-probe` / recording
  `.cache/recordings/issue258-slope-right-jump-probe/scenario.json`. It starts
  at BLW1 `(760,231)`, holds Right after one neutral explicit-start frame, and
  pulses B on frame 13. Native/oracle diverge around the slope jump: at frame 15
  native keeps `kid_x_vel=0x6000` and frame 17 reaches `0xB600`, while oracle
  enters the `loc_AA22` jump/wall loop with `kid_x_vel=0` at frame 15 and only
  `0x2B00` by frame 17. Rejected hypothesis: this is not the slope-walk
  side-clamp guard suppressing `sub_8F26`; the native wall probe reports no hit
  on the split frame. Next investigation should trace the one-pixel earlier
  oracle slope phase / `loc_AA22` entry rather than broadening slope-as-wall
  collision.
- **#258/#277 follow-up (2026-06-14)**: the direct Level 00 probe is not clean
  movement evidence yet. `tas-diff` shows native/oracle already offset before
  the jump, and a 3x3 start-position nudge matrix over `(759..761,230..232)`
  still diverges at frame 0 with native Kid X consistently `$2000` subpixels
  behind the oracle sample. Natural routes are cleaner: Level 00 right-slope
  jump timings `428`, `432`, `437`, and the one-frame B pulse all pass
  `verify-tas` through the jump window; BLW2 stair-jump right-current and
  right late-B `76/84/90` all have no authoritative gameplay divergence; BLW2
  left early-tap also passes. BLW2 left-current diverges first in
  `number_objects` at frame 90, with later camera/object drift from the
  RockTank/ShooterBlock lifecycle, not Kid movement fields. Next movement work
  needs an exact live input recording or a fixed direct-probe bootstrap before
  patching slope/jump physics.
- **#258/#277 follow-up (2026-06-15)**: the direct Level 00 slope-jump bootstrap
  is now trace-clean for its checked window. Native now reports the same
  `Character_Movement=6` slope-walk export as oracle when `loc_9D22` has written
  MoveID 6 and same-frame `loc_9F18` clamps the slope velocity to zero. Focused
  checks on `.cache/recordings/issue258-slope-right-jump-probe/scenario.json`
  report `tas-diff --include-advisory --include-physics-trace` `30 match, 0
  diverge` and `verify-tas --frame-start 0 --frame-end 30 --checkpoint-interval
  1 --frontier-kind gameplay --bisect` passes. This unblocks using the direct
  probe as a bootstrap sanity check, but it still does not reproduce the latest
  human right/left stair launch complaint; next #258/#277 work needs a
  human-equivalent stair route or live input capture before changing jump
  physics.
- **#258/#277 follow-up (2026-06-15, later)**: the cached BLW2 right-stair
  fixture `.cache/recordings/issue277-blw2-stair-jump-right-current/scenario.json`
  is stale at frame 0 (`kid_y_pos` native `$01800000`, oracle `$017F0000`).
  A cache-only one-pixel-up start nudge
  `.cache/debug/issue277-start-nudge/right-y382.json` verifies cleanly through
  frame 180, so the scripted right-stair jump route is not current movement
  evidence. The paired left-current route also diverges at frame 0
  (`number_objects` native 4, oracle 5). The clean Level 00 slope-jump route
  matches frames 10..24 including physics trace, so the live 45-degree launch
  complaint still needs a new human-equivalent input capture.
- **#258/#277 follow-up (2026-06-15, held-jump routes)**: refreshed natural
  Level 00 route `.cache/recordings/issue-258-level00-natural-right-current/`
  and tested held-B+Right overlays around the first slope/stair window. Best
  native route `400..500 B+Right` reaches the slope and later target cleanly,
  but the materialised scenario `.cache/debug/issue258-natural-b400-500/scenario.json`
  passes `verify-tas --frame-start 380 --frame-end 560 --checkpoint-interval 1
  --frontier-kind gameplay --bisect` by CRC, so this held-right/held-jump
  natural route still does not reproduce the human 45-degree launch as an
  oracle divergence. The cached Level 00 left direct probe remains blocked as
  comparator input: original y=303 and nearby y=304/y=305 all first-diverge at
  frame 0 (`kid_y_pos` or `character_movement`), while y=302 jumps native to
  `$01210000` vs oracle `$012E0000`. Do not patch slope/jump physics from these
  direct left artefacts; the missing piece remains a human-equivalent live
  capture or a clean natural left route.
- **#258 fixed candidate (2026-06-15)**: direct right-slope jump route
  `.cache/recordings/issue258-slope-right-jump-probe/scenario.json` now passes
  `verify-tas --frontier-kind gameplay` across all 120 frames. Native had two
  ASM handoff gaps: frame 36 over-advanced X by `$A500` when `sub_902A ->
  loc_A99E -> loc_8BF0 -> loc_9D22` recontacted the first `$405C/$605D`
  SlopeUp/Solid edge at post-vertical fraction `$2A00`; frame 100 stayed in
  diagonal slope-walk instead of taking the `loc_9F18 -> loc_9FBC` flat-support
  exit and clearing `y_vel`. Regressions
  `level00_frame_36_direct_right_slope_jump_recontact_matches_oracle_loc_a99e`
  and `level00_frame_100_direct_right_slope_exit_clears_diagonal_y_vel_loc_9fbc`
  cover both exact oracle frames. This specifically addresses the held-Right
  BLW1 slope/stair launcher route; #277 left-stair/live BLW2 validation remains
  open until a clean human-equivalent route is captured.
- **#277 route repair (2026-06-15)**: repaired the stale BLW2 left-stair
  fixture with cache-local start nudge
  `.cache/debug/issue277-start-nudge/left-x1746-y237.json`; it passes
  `verify-tas --frame-start 0 --frame-end 180 --checkpoint-interval 1
  --frontier-kind gameplay --bisect`. Grounded `LEFT+B` overlay
  `.cache/debug/issue277-start-nudge/left-x1746-y237-b96.json` is also
  oracle-clean through the stair/jump window; native frame 98 takes
  `kid_grounded_release_probe held=left/jump=1` and
  `kid_jump_gravity_probe branch=jump_pressed` with negative x velocity. This
  rejects the repaired BLW2 scripted stair fixture as the current human
  left-input-drop repro. Earlier B88/B90-style pulses are contaminated by
  ShooterBlock damage before a useful grounded jump signal, so do not patch
  movement from those timings.
- **#277 recheck (2026-06-16)**: the repaired `left-x1746-y237-b96` fixture is
  gameplay-clean through the checked stair/jump window again after the shooter
  slot reread. `c80b9390`/`f5adb2090` first narrowed the native-only frame-165/166
  damage to a fresh visible `sub_F096` slot at source cell `(86,19)`, then the
  ROM reread showed `loc_F28C/loc_F410` only compile visible sprite entries;
  moving `ShooterObject_RAM` bullets are allocated separately by `loc_EC76`
  from evanescent/touch-triggered block workers. Native no longer spawns
  `EnemyProjectile::shooter_block_loc_EC76` from visible slots. Verification:
  `verify-tas --frame-start 0 --frame-end 180 --checkpoint-interval 1
  --frontier-kind gameplay --bisect` passes, and focused `tas-diff
  --frame-start 164 --frame-end 169 --frontier-kind gameplay --include-advisory
  --show-object-roster-on-match --include-physics-trace` reports `5 match, 0
  diverge`. This removes the fixture's shooter contamination but still rejects
  it as the current human dropped-left movement proof; keep #277 open for a
  fresh human-equivalent stair route if the visible complaint persists.
- **Then**: #260 live slime/metal-block route (the Drill-skin headbutt blocker is now
  gameplay-CRC clean on `.cache/debug/drips-route-candidates/orig_l480_530_b528.json`
  through frame 570, so do not re-open block type C/`sub_BB54` from that route; still
  need a route that reaches the reported live slime/metal-block contact. Current clean
  near-miss: `.cache/debug/issue260-tail-grid/l524_b530_540.json` passes
  gameplay CRC and enemy checkpoints through frame 560. A 2026-06-14 `480..540 B`
  lift overlay gets Kid's healthy hitbox to row-32 cells `32,32`/`33,32` at
  `dx=0 dy=-1 @490`, improving the old `dx=0 dy=-16` miss, but it is still not
  the slime/metal-block repro: the closest DripsSurfaceCrawler remains
  `dx=+102 dy=+1 @512`, and the active DripsDrop is still far right. Do not
  repeat this late B-lift window unchanged; it is a Kid/block-line near-miss
  only, not a live enemy/block contact. 2026-06-14 follow-up added durable
  route-search scenario `issue260-dlb-tail-b-row32-bounce`: it preserves the
  `dlb_tail_b` p11 overlay (`150..250 Left`, `251..350 B+Right`, `351..430 Right`,
  neutral `431..500`, `501..570 B+Left`) and verifies enemy-frontier clean through
  frames `450..530`; `tas-diff` frames `500..516` has `16 match, 0 diverge`.
  It produces a matched Bounce on Level 01 cell `37,32` at frame 511, with Kid's
  healthy hitbox `dx=0 dy=-1`; DripsDrop is still closest to Kid at `dx=0 dy=-1
  @519`, and the crawler appears later at `dx=+12 dy=-1 @577`. No Destroy occurs,
  so this is a better oracle-comparable row-32 bounce near-miss, not the reported
  slime/block destruction route. Next search should move the live Drips child onto
  the row-32 block line before/at the bounce instead of repeating Kid-only bounce
  tails. 2026-06-16 follow-up rechecked the durable route with the corrected
  baseline overlay (`0-0:right`) and confirmed the same outcome: enemy-frontier
  `verify-tas` passes frames `450..530`, `Bounce cell 37,32 @511` is present,
  and `Destroy` is still absent. A `lower_170_300` early-approach probe improves
  the DripsDrop/cell conjunction score to 40 but loses the row-32 bounce and ends
  with HP 0, so reject that route shape as a closer death path, not block proof.
  2026-06-14 follow-up route batches on the same committed scenario reject
  the obvious early-right acceleration axis: cleared-script `fast_*` routes overshoot
  into `LevelTransition` and lose DripsDrop/DripsSurfaceCrawler sightings entirely,
  while intermediate `mid_*` routes preserve Drips but leave Kid horizontally near
  row 32 while vertically separated (`dy=-33` or worse) and still report no
  Destroy/Bounce at cells `36,32`/`37,32`. Longer jump-hold `lift_*` variants can
  make an early Kid/block bounce (`lift_a`, `Bounce cell 37,32 @260`) but again have
  no live Drips projectile in the scored window. Do not repeat early-right/long-B
  acceleration from this base unchanged; the useful next axis needs a route that
  preserves the Drips child and raises Kid into row 32 without advancing into the
  early transition/bounce-only branch. Tooling now supports
  `probe-record-scenario-file-batch --target-projectile-cell-conjunction KIND:COL,ROW`
  to rank same-frame healthy Kid hitbox + projectile proximity to a cell. Re-scoring
  the intermediate branch shows `mid_g` is the real best same-frame lead
  (`drips-drop:36,32` score 36 at frame 308: Kid `dx=0 dy=-33`, drop
  `dx=0 dy=-3`), while `lift_a` is only a no-Drips early bounce. A tighter
  `mid_g` second-jump timing sweep (`mg_j220..mg_j270`) does not improve the
  vertical miss: best remains score 36 with `kid_dy=-33`, and no Destroy/Bounce
  proof appears. Do not repeat this `mid_g` second-jump timing axis unchanged.
  2026-06-14 `issue-260-early-jump-r360-conjunction` exposed the same
  falling-child contact timing on a shorter route: before the fix native kept
  HP at 2 while oracle took Drips `$28` damage at frame 372, then an over-broad
  first pass damaged frames 368-371. The committed fix gates DripsDrop
  object-collision damage until `loc_3345C` has reached the oracle-visible
  y-velocity window (`$8000+`); `verify-tas` frames `350..390` now passes all
  40 enemy-frontier checkpoints. This is contact-timing evidence only, not the
  row-32 block destruction route.
  2026-06-14 follow-up after materialising
  `.cache/recordings/issue260-dlb-tail-b-row32-bounce/`: targeted
  same-frame conjunction batches around the `431..500` neutral tail and late
  B+Left timing still fail to produce a live Drips/row-32 metal-block event.
  Best healthy candidate from the first pass was `rb471_500`
  (`471..500 Right+B,501..570 Left+B`), but it only scores 64 against
  `DripsDrop:36,32` at frame 640 (Kid `dx=+28 dy=-33`, drop `dx=0 dy=-3`)
  and has no Bounce/Destroy. The tighter follow-up found `rb451_500` with Kid
  and drop horizontally aligned to cell `36,32`, but Kid is still `dy=-33`,
  finishes at HP 0, and has no row-32 Bounce/Destroy. Do not repeat this
  `rb451..500`/late B+Left tail timing axis; it confirms the child can be near
  the cell horizontally, but not vertically or as a block-mutating contact.
  2026-06-15 follow-up: single `probe-record-scenario-file` now accepts
  `--target-projectile-cell` and `--target-projectile-cell-conjunction`, matching
  the batch scorer for deep inspection of one route candidate. On
  `issue260-dlb-tail-b-row32-bounce`, it confirms the base near-miss has Kid's
  healthy hitbox on cell `36,32` at frame 511, but the closest `DripsDrop`
  cell approach is frame 348 (`dx=0 dy=-3`) and the best same-frame conjunction
  is frame 640 with Kid still `dx=+127 dy=-33`; no Bounce/Destroy follows.
  The `pretap_*` pre-contact batch axis is rejected too: best `pretap_holdr`
  regresses to conjunction score 163 at frame 640 with no block event, while
  most variants stay at score 185 around frame 360. Do not repeat this
  pre-contact tap axis; the next route search needs an earlier/lower block
  approach or delayed Drips phase.
  Named route
  `issue260-drips-drop-contact-probe` promotes the `l528/b530/r536` tail and
  reaches exact native DripsDrop overlap at frame 535; oracle diff for frames
  520..545 has only the known non-authoritative `number_objects` limitation. It
  is route evidence, not the human metal-block contact repro. The right-side raw
  `dx=+13 dy=-1` candidates are HP-zero/LevelTransition routes; healthy closest
  remains `(584,543)` `dx=0 dy=-48`, so do not mine them as proof. Do not reuse
  `.cache/debug/issue260-route-probes/floor-explicit-left-drips-close-617.json` as live
  metal-block evidence. Fresh 2026-06-10 trace reconfirms that exact route is already
  Drips-phase shifted before contact because it uses `settle_oracle_warp=true`: native #08
  seeds at frame 13 with wait `$43` while oracle exposes `loc_33424 d0=$5d` at frame 15.
  The no-settle copy starts #07/#08 clean but is not a valid late metal-block proof either
  because the oracle side has lost the comparable Drips slots by frame 590. Next useful #260
  work is a genuinely refreshed route/bootstrap that starts clean and stays oracle-comparable,
  not Tank/Drips hitbox/block patches from these stale route files. Post-floor delayed-jump
  overlays from the current `l524_b530_540` tail (`release 548/556/560`, neutral/right/left
  jump variants) still rank the old frame-540 near-miss as the best healthy `(456,543)`
  approach, and right-biased variants spend HP without improving row-32 contact; do not
  retry that late-jump axis unchanged. Source-generated `issue260-drips-cleaner-prefix-audit`
  / `issue260-drips-b-jump-climb-seed` plus the `140..300:none,170..260:left` overlay no
  longer reproduces the archived `dx=-2 dy=0 @496` row-32 branch on current native; treat that
  archived branch as cache-specific until a current DynamicScenario file is recovered. Clearing
  `300..479` on the clean no-settle scratch route and restoring the late tail also leaves the
  same early healthy approaches (`(456,543)` `dx=-3 dy=-16 @68`, `(584,543)` `dx=0 dy=-48 @127`);
  do not repeat that idle-window axis unchanged. 2026-06-11 follow-up: a current
  `issue260-drips-drop-contact-probe` pre-380 prefix sweep (`160..330` left-window shifts,
  no-overlap left windows, split-left windows, and `260..360` coast/brake/right/B+right variants)
  also failed to improve the late row-32 target. Windowed probes over frames 300..620 still leave
  the best live late route as the already-documented `wait390`/`left400` family (`(584,543)`
  healthy `dx=+1 dy=-18 @444` or `dx=0 dy=-22 @470`, both later LevelTransition), while the
  drop-contact prefix variants either stay far left at the Drips Y row or keep the old
  `dx=0 dy=-48 @128` early pass. Do not repeat this pre-380 left-window axis unchanged.
  A later `450..469 B+Right` overlay on the named drop-contact probe improves healthy
  right target proximity to `(584,543)` `dx=-1 dy=-16 @539`, but it is not metal-block
  evidence: oracle damage at frame 529 comes from a right-side Drips crawler already
  active at `(588,592)` while native layout #08 is still in
  `pre_drop:0x8005` and has not spawned that cycle's child. Treat this as another
  phase-shifted route rejection, not a Drips hitbox or block-break lead. The cleaner
  `470..500 B+Right` variant verifies cleanly through frame 545 while staying alive,
  but still only reaches `(584,543)` at healthy `dx=+1 dy=-20`; do not repeat late
  right-window trimming as metal-block proof without a route-shape change. A fresh
  no-settle 720-frame route rebuild found native-live candidate `freshF`
  (`0..160 Right`, `55..125 B+Right`, `160..240 Left`, `240..320 B+Left`,
  `340..430 Right`, `478..528 Left`, `530..567 B+Right`) reaching `(456,543)`
  healthy `dx=0 dy=-16 @538`, but it is not oracle-comparable. After direct
  probe oracle warps were fixed to target Level 01 instead of stale Level 00,
  and the direct-probe title-card Start latch was drained/phase-aligned, Kid
  advances on the same input phase as native. The route still diverges
  immediately on bootstrap roster shape (`number_objects` native 4 vs oracle 5),
  and the late window first differs on HP at frame 480 (`native=2`,
  `oracle=1`) with oracle already exposing Drips `loc_33488` child/gfx slots.
  Treat `freshF` as no-settle bootstrap evidence only, not route proof or a
  Drips/block patch lead. 2026-06-11 follow-up after `513b36d1`: the
  focused right-side Drips contact route
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493.json`
  now passes enemy-frontier oracle verification for frames 470..559. The fix
  aligned `Drips.asm loc_3340E` initial seed timing with the visible Kid gfx
  slot, preserved the two-frame loc_333F6→loc_3340E coroutine bridge, and made
  live `loc_33550` crawler contact consume the frame before Kid's horizontal
  movement. This clears the stale Drips initial-seed/contact hypothesis, but it
  still is not the reported live slime/metal-block repro; keep #260 `[~]` and
  continue from a refreshed route/bootstrap that reaches metal-block contact.
  2026-06-14 follow-up: `.cache/debug/issue260-brakehop-probe/oracle-frame-scenario.json`
  no longer diverges on the false Diamond pickup. Root cause was loader
  reconciliation rewriting BLW2 live cell `0xCE8A` from `$EC2D` (type-C no-op
  for `off_BB80`) to `$E102` purely from the durable prize Block List. Native
  now preserves non-prize live `Level_Layout` cells and the route verifies with
  no authoritative divergences through frames `0..700`. This is a fixed
  sub-case only: it removes the stale centre-`EC2D` false-prize branch, but it is
  still not the reported live slime/metal-block contact repro. Success criteria
  on these centre cells should not be "find `Destroy`": raw `$EC2D` is a type-C
  Drill/shooter cell, and ROM `off_BB80[$C]` is a headbutt no-op. Prefer probes
  that show the reported red projectile/type-C route or a live Drips child
  occupying the same row-32 cell window while Kid contacts the cell. A brakehop-seed
  tail branch, `.cache/recordings/issue-260-brakehop-b450-l485/scenario.json`
  (`300..430 Right`, `431..449 none`, `450..484 B`, `485..520 Left`), verifies
  enemy-frontier clean for frames `450..510` and `tas-diff` reports 57 matched
  frames. It bounces `cell 37,32 @460` and later gets a `DripsSurfaceCrawler`
  overlap with Kid (`dx=0 dy=+1 @505`), but these are separate events: the
  crawler is down at `cell 36,37`, `block_destroy_effects=[]`, and the row-32
  block only bounces. Treat this as clean separation evidence, not the reported
  slime/metal-block destruction route.
  2026-06-14 route follow-up: a fresh phase-delay batch over
  `.cache/recordings/issue-260-drips-b-jump-climb-seed-current/scenario.json`
  found one stronger raw `EC2D` hitbox pass, but not a usable repro. Candidate
  `400..620 none,430..455 Right,456..515 B+Right,516..550 Left,551..620 Right`
  reaches cell `36,32` at healthy `dx=0 dy=-1 @465`, then dies into
  `LevelTransition`; its closest DripsSurfaceCrawler is still `dx=-78 dy=+1 @563`.
  The best live survivor,
  `430..620 none,470..530 B+Right,531..560 Left,561..620 Right`, verifies
  enemy-frontier clean through frames `430..620` but only reaches
  `dx=0 dy=-2 @493`, with the closest crawler still `dx=-52 dy=+1 @539` and
  DripsDrop `dx=+48 dy=-11 @409`. Do not repeat this phase-delay/late-lift axis
  unchanged; it is another oracle-comparable near-miss, not the slime/metal-block
  contact. A 2026-06-14 refinement on the same seed, clearing `360..620` and
  trying staggered `390..575` right/B/right/left/right phases, was worse than
  the known 1px row-32 miss: best candidate `390..430 B+Right,431..500 Right,
  501..545 B+Right,546..575 Left,576..620 Right` reaches cell `36,32` at only
  `dx=+10 dy=-12 @409`, exits through `LevelTransition`, leaves the closest
  DripsSurfaceCrawler `dx=-148 dy=+1 @494`, and closest DripsDrop
  `dx=+22 dy=+10 @409`. Do not repeat this staggered right/B/right phase axis
  without changing the bootstrap or target Drips cycle. Tooling follow-up added
  `probe-record-scenario-file-batch --target-projectile-cell KIND:COL,ROW`; a
  smoke batch on the current seed ranks DripsDrop vs cell `36,32` and shows the
  drop already reaches `dx=0 dy=-3 @381` independent of the late overlays, so
  the next useful search axis is Kid timing around that early drop/cell window,
  not more post-390 drop positioning. A 2026-06-14 pre-climb route-shape batch
  on the same seed found a stronger live near-contact prefix: clearing
  `390..440`, then `441..480 Right`, `481..510 B+Right`, `511..565 Left`,
  `566..700 none` keeps final mode `InGame` with HP 1 while preserving
  Kid/DripsDrop `dx=-3 dy=-2 @483` and Kid/cell `36,32` hitbox
  `dx=-1 dy=-3 @488`; sibling exits (`pre390_left520`, `pre390_shortjump`)
  have the same closest frames. This is still not reported slime/metal-block
  proof: Kid spends HP at frame 465 before the row-32 pass and no live
  crawler/block mutation is demonstrated. The materialised survivor bundle
  `.cache/recordings/issue-260-pre390-survivor-current/scenario.json`
  verifies enemy-frontier clean for frames `450..520`, so use it as the current
  refreshed comparable prefix; do not treat the `pre390_*` exit trims as a
  Drips/block patch lead without a follow-on route that actually mutates the
  metal block. Follow-up block-event scoring (`--target-block-event`) confirms
  this family only bounces the row-32 metal block: `pre390_l510` and sibling
  exits report `Bounce cell 37,32 @489`, but all tested `p370..p410`
  pre-climb/exit variants report `Destroy cell 37,32 <none>`. Do not mine the
  pre390 exit trims for the reported block break unless a new route shape
  produces an actual destroy event. A broader re-score of the current documented
  route families (`seed_survivor`, `phase_delay`, `pre390`, `earlyB`, `kH`,
  `p11`, `wait390`, `left400`) likewise finds no `Destroy` at cells `36,32` or
  `37,32`; only `earlyB` (`Bounce cell 37,32 @380`) and `pre390` bounce. A first
  bounded Kid-timing batch around that window found
  `300..450 none,330..370 Right,371..420 B+Right,421..450 Left`
  gets Kid's healthy hitbox to `dx=-5 dy=-3 @379` against cell `36,32`, but it
  is still not contact proof: focused probing reports closest Kid/DripsDrop
  separation `dx=-23 dy=-30 @381`, no DripsSurfaceCrawler in frames 330..520,
  and the run exits through `LevelTransition`. Do not repeat this `earlyB`
  timing unchanged; use it only as evidence that Kid/cell and drop/cell are
  near at adjacent frames but not mutually aligned. 2026-06-15 follow-up:
  materialised
  `.cache/recordings/issue-260-pre390-b480-520/scenario.json` from the current
  pre390 survivor with `480..520 B`. It is oracle-comparable for the relevant
  window (`verify-tas --frame-start 470 --frame-end 505 --checkpoint-interval 1
  --frontier-kind enemy --bisect` passes), and it keeps the good DripsDrop
  near-overlap (`dx=-3 dy=-2 @483`) plus a row-32 metal-block bounce
  (`Bounce cell 37,32 @489`). It is still not the reported slime/metal-block
  mutation: `Destroy` is absent for cells `36,32`/`37,32`, the best same-frame
  Kid/drop/cell conjunction remains 36 px off vertically, and the route ends
  with HP 0. Do not repeat the direct `480..520 B`/`B+Right`/Right/Left tail on
  this prefix as block-destruction proof; the next useful axis must move the
  active drop/crawler onto row 32 or delay the block contact without losing HP.
  A follow-up delay sweep found a cleaner survivor lead,
  `.cache/recordings/issue-260-pre390-br508-538/scenario.json`
  (`470..590 none`, `508..538 B+Right`), verified enemy-frontier clean for
  frames `500..530`. It improves the same-frame Kid/drop/cell conjunction to
  Kid `dx=+5 dy=-1`, DripsDrop `dx=+8 dy=-3` at frame 520 and ends InGame with
  HP 1, but it still has no Bounce/Destroy at `37,32`. Post-538 nudges
  (`Right`, `B`, `B+Right`, `Left`, `B+Left`, and right-then-B continuations)
  preserve the same frame-520 lead or regress into `LevelTransition`; none
  creates a block event. Do not repeat post-538 tail nudges on this branch
  unchanged; the next useful axis is a pre-508 approach change that keeps the
  frame-520 alignment but pushes Kid into the block.
  Follow-up pre-508 approach search found
  `.cache/recordings/issue-260-pre390-r490-br508/scenario.json`
  (`470..590 none`, `490..507 Right`, `508..538 B+Right`), verified
  enemy-frontier clean for frames `510..525`. It produces a clean row-32 bounce
  (`Bounce cell 37,32 @518`) with the best same-frame geometry so far: Kid is
  at `dx=0 dy=-1` against the cell while the DripsDrop is at `dx=+8 dy=-3`.
  It still has no `Destroy`, and the projectile is close to the block rather
  than occupying/mutating it. Recovery overlays after frame 519 can keep HP 1
  with neutral input, but `Left`/`Right`/`B+Left`/`B+Right` recoveries either
  keep the same no-destroy bounce or enter `LevelTransition`. Do not repeat
  post-bounce recovery trims on `r490_br508`; the next useful axis must alter
  the Drips cycle or the pre-490 approach so the projectile/crawler itself
  reaches the row-32 block at contact time.
  2026-06-16 broader pre-330 approach sweep on the same route materialised
  `e220_l280_r330` (`220..279 Left`, `280..329 Right`) as
  `.cache/debug/issue260-e220-l280-r330.json`: it remains enemy-frontier clean
  for frames `500..525` and hits `cell 37,32` at `dx=0 dy=-1 @517`, but the
  closest DripsDrop conjunction is still `dx=+8 dy=-11` from that cell while
  Kid contacts it (`manhattan=20 @517`) and no `Destroy` occurs. Treat early
  left/right pre-330 route shaping on this seed as exhausted unless it changes
  the Drips child spawn/phase, not just Kid's climb timing.
  2026-06-16 post-330 jump/right refinement on the same seed used
  `Drips.asm loc_3340E`/`loc_3345C` timing: the right parent is in
  `pre_drop` at frame 330 and the active DripsDrop reaches `cell 36,32` at
  `dx=0 dy=-3 @361`. Best materialised lead
  `.cache/debug/issue260-r330-br354-l394.json`
  (`330..353 Right`, `354..393 B+Right`, `394..433 Left`) is enemy-frontier
  clean for frames `350..375` and improves same-frame Kid/drop/cell geometry to
  Kid `dx=+1 dy=-1`, DripsDrop `dx=0 dy=-3` at frame 364 (`manhattan=5`).
  It still produces no `Bounce`/`Destroy` at `36,32`/`37,32` and later exits
  through `LevelTransition`, so treat this as an oracle-comparable early-overlap
  dead end, not the reported slime/metal-block mutation repro.
  Follow-up pre-490 approach sweep over `430..489` windows did not improve the
  proof. The DripsDrop still reaches row-32 vicinity early (`dx=+8 dy=-3 @455`)
  while the DripsSurfaceCrawler remains far below the block line (`dy=-65`),
  and the tested `450..470`/`460..480`/`470..490` right/left/B/B+Right/B+Left
  variants either lose the bounce or reproduce only no-destroy bounces at
  frames `517`, `523`, or `542`. Do not repeat these short pre-490 window
  overlays on the pre390 survivor; the next useful axis needs a route/bootstrap
  change that alters the Drips child cycle itself, not another local Kid timing
  trim.
  Follow-up tooling added
  batch `--target-projectile` scoring for closest healthy Kid/projectile
  proximity. Re-scoring the same early timing family shows `kH`
  (`300..350 B,351..390 B+Right,391..430 Left`) as the best branch, but it is
  still only `dx=-9 dy=-11 @409` from DripsDrop while the drop/cell best remains
  `dx=0 dy=-3 @381`; this is closer than `earlyB` but still not a contact or
  block-mutation repro. A cleaner-seed early row-32 rebound batch on
  `.cache/recordings/issue260-drips-drop-contact-probe-local/scenario.json`
  found `re_none330_r360_bl400` (`300..450 none`, `330..359 Right`,
  `360..399 B+Right`, `400..430 B+Left`, `431..450 Left`) and materialised it as
  `.cache/recordings/issue-260-early-row32-bounce-re-none330/scenario.json`. It
  bounces `cell 37,32 @379` while a `DripsDrop` is active, but still has no
  `Destroy`; the native drop remains in `cell 36,33`, and oracle comparison is
  already invalid because `verify-tas --frame-start 350 --frame-end 390
  --frontier-kind enemy --bisect` first diverges on HP at frame 374. `tas-diff
  --drips-trace --frame-start 368 --frame-end 380` shows oracle's right Drips
  child in `Drips.asm loc_33460` writing `$28` contact from frame 372 while
  native is pursuing the later Kid/block bounce. Do not repeat this
  pre-330/360-430 cleaner-seed rebound axis as block-destruction proof. A
  tighter local search found native-live `p11`
  (`300..355 B,356..390 Right,391..425 B+Left`) with HP 1, closest Kid/DripsDrop
  `dx=+1 dy=-1 @402`, and Kid/cell hitbox `dx=0 dy=-1 @400`, but it is still not
  valid proof: a materialised `.cache/debug/issue260-p11/scenario.json`
  `verify-tas --frame-start 390 --frame-end 430 --frontier-kind enemy --bisect`
  first diverges at frame 396 on HP (`native=2`, `oracle=1`). Treat `p11` as a
  useful native route-shape lead only; fix or avoid the pre-396 HP divergence
  before using it for slime/metal-block contact evidence. Follow-up `tas-diff
  --drips-trace --frame-start 390 --frame-end 405` shows the split is a Drips
  fall-phase mismatch before contact: oracle writes `colid_kidbelow` (`$28`) on
  the right falling child by frame 394 (`xy=584,564`, `vel=$2E000`) and HP drops
  at frame 395, while native still has the same child at `y=541`, `vel=$1C000`.
  Do not patch from this stale p11 trace as a generic falling-drop damage toggle;
  first align the `loc_3340E`/`loc_3345C` fall timing for layout #08 or choose a
  route that avoids that pre-contact phase drift. A later 2026-06-14
  left-wall-bounce route on
  `.cache/debug/issue260-phase-delay-br470-20260614/oracle-frame-scenario.json`
  did provide clean proof: native spent HP on a visible `DripsDrop` overlap at
  frame 478 while oracle stayed at HP 2, and `Drips.asm loc_3345C` only waits
  for floor collision before `loc_33488`/`loc_335D8` creates splash/crawler
  state. Native now keeps falling drops out of generic player-damage/FA75
  contact, and the materialised route verifies with no authoritative
  divergences through frames `450..520`. 2026-06-14 follow-up isolated the first
  timing split earlier: at p11 frame 174 native layout #08 splash reseeds with
  RNG byte `$2C` and previous-frame Kid X word `$02A0`, producing wait `$00AB`;
  oracle is already in `Drips.asm loc_33424` after the one-frame hibernate with
  `d0=$00A2` at `a0=$33522`, current `(Addr_GfxObject_Kid).x_pos=$02A1FA00`,
  and the last generated RNG ring byte `$0A`. Native/oracle RNG offsets both
  read `$FFF5`, so the split is the RNG ring contents before this reseed, not
  table order, falling-drop damage, or a universal Kid gfx X offset. Rejected
  patch: using the existing initial-seed `drips_kid_gfx_x_pos_loc_3340e` (+16px)
  for splash reseeds worsens p11 (#08 wait `$003D` vs oracle `$0024` at frame
  300); the correct fix must find the pre-174 RNG-ring content drift before
  patching Drips timing. Follow-up typed `--rng-trace` byte columns show this
  ring-content drift already exists at p11 scenario frame 0 (`native_rng=$FFCF`
  byte `$31`, oracle byte `$09`, with matching cursor), so inspect the settled
  Level 1 warp/bootstrap RNG ring state before touching Drips again. Oracle
  settle tracing for the same p11 scenario ends `target_live` step 59 at
  `rng_offset=$FFCB`, `rng_last_byte=$32`, and scenario frame 0 advances to
  `$FFCF/$09`; native's settled-warp ring must be aligned to that hidden state
  before Drips timing can be trusted. 2026-06-14 native now injects that Level 1
  recording-settle ring: `tas-diff --rng-trace --frame-start 0 --frame-end 5`
  matches oracle bytes through frame 4 (`$FFCF/$09`, `$FFD3/$00`, `$FFD7/$7F`,
  `$FFDB/$E8`, `$FFDF/$92`). The p11 HP divergence remains at frame 396, and
  `--drips-trace --frame-start 173 --frame-end 175` still shows a frame-174
  Drips phase split (`native_rng=$FFF4`, `oracle_rng=$FFF5`), so continue from
  Drips.asm `loc_33424`/`loc_3345C` timing rather than the settled RNG ring.
  2026-06-14 follow-up: `GfxObject_Animate` (`kid.asm loc_1158/sub_1194`) makes
  Drips.asm `stru_33F90`'s raw `$A,$A,8,6,1` script effectively 31 frames, not
  35, because entry-load frames consume one tick and the duration-1 final entry
  finishes as soon as the terminator is visible. With that native timing,
  `tas-diff --drips-trace --frame-start 145 --frame-end 178` lines up the #08
  fall/splash reseed (`rng_last=$0A` at frame 174), but `verify-tas
  --frame-start 390 --frame-end 430 --frontier-kind enemy --bisect` still
  first diverges at frame 396 on HP. The remaining p11 split is now the
  projectile/player damage path: oracle writes `colid_kidbelow` (`$28`) on the
  falling Drips child at frames 394-396 while native keeps HP at 2. 2026-06-14
  follow-up fixed that split: Drips.asm `loc_3345C` leaves the falling child
  alive until floor `$0C`, but Kid's `Character_CheckCollision` `off_B640`
  still routes the same `$28` through `Check_for_recent_damge`; native now
  lets `DripsDrop` hurt Kid without consuming the child. `verify-tas` passes
  p11 frames `390..430`, and `tas-diff --drips-trace --frame-start 388
  --frame-end 402` confirms the oracle `$28` window on frames 394-399. A
  refreshed p11 route probe now verifies enemy-frontier clean through frames
  `360..430`; native p11 remains route-shape evidence only, not the human
  slime/metal-block proof. `probe-record-scenario-file --target-cell-hitbox
  36,32 --target-projectile drips-drop` reports Kid within `dx=0 dy=-1` of the
  cell at frame 400 and `dx=+5 dy=-1` from the DripsDrop at frame 395, but the
  run later enters death/`LevelTransition`. Post-contact tail variants can keep
  the route alive (`426..700 Left` ends `InGame` with HP 1), but that survivor
  retreats far left and leaves the nearest DripsSurfaceCrawler `dx=+208 dy=+1`.
  Early handoff batches around `356..425` also failed to improve the proof:
  the best direct drop branches either keep p11's `dx=+5 dy=-1` or trade it for
  `dx=+2 dy=-11`, and all still end in `LevelTransition`. Do not repeat this
  p11 early-handoff/post-contact tail axis unchanged; the next useful #260
  search needs a different route shape that aligns Kid, the live Drips child,
  and the row-32 metal block in the same frame window.
  2026-06-14 route follow-up: a pre-approach timing batch on `.cache/debug/issue260-p11/scenario.json`
  found `250..290 Left,291..330 B+Right` as a stronger native route shape:
  Kid reaches DripsDrop `dx=+1 dy=-1 @395` and cell `37,32` `dx=+1 dy=-1 @400`.
  Adding `426..700 Right` materialises `.cache/recordings/issue-260-slbr-right426/scenario.json`
  and produces `Bounce cell 37,32 @501`, but this is not oracle-comparable:
  `verify-tas --frame-start 390 --frame-end 520 --frontier-kind enemy --bisect`
  first diverges at frame 393 on HP (`native=2`, `oracle=1`). `tas-diff
  --drips-trace --frame-start 388 --frame-end 402` shows oracle's right falling
  Drips child writing `$28` contact at frames 392..399 before the native bounce
  route can prove anything. Treat this as a phase-shifted native near-miss, not
  metal-block proof; do not repeat this `slbr` pre-approach plus right-recovery
  family unchanged. Follow-up route proof: `mb_left_long` on the same p11 base
  (`150..230 Left`, `231..320 B+Right`, `321..390 Right`, `391..425 B+Left`)
  materialises as `.cache/recordings/issue-260-p11-mb-left-long/scenario.json`
  and is oracle-comparable for the block window:
  `verify-tas --frame-start 450 --frame-end 505 --frontier-kind enemy --bisect`
  passes. It produces native/oracle-matched `Bounce cell 37,32 @490`, but no
  `Destroy`, no active projectile in the bounce frame, and Drips layout #08 is
  still `pre_drop:0x800a` with no child; the nearest enemy support/contact probe
  is separated from Kid. Treat this as clean Kid/block-bounce evidence only,
  not the reported slime/block destruction route. Delaying the same shape with
  `426..500 none` materialises
  `.cache/recordings/issue-260-p11-mb-hold-none/scenario.json` and also verifies
  clean for frames `490..520`; it shifts the bounce to `cell 37,32 @511` while a
  falling `DripsDrop` is active, but that drop remains in cell `36,33`, no
  `Destroy` occurs, and the enemy contact/support probes are still separated.
  Do not repeat this delayed p11 bounce axis as a block-destruction lead unless
  a new route moves the active drop/crawler onto the row-32 block itself.
  A
  follow-up over the clean
  `.cache/debug/issue260-tail-grid/l524_b530_540.json` tail found `480..540 B`
  gets Kid's healthy hitbox within 1 px of row-32 cells `32,32`/`33,32` at
  frame 490, but still leaves the DripsSurfaceCrawler `dx=+102 dy=+1 @512`;
  keep searching for a route that brings the live child/projectile to the
  metal-block line instead of repeating Kid-only lift-window trims.
  2026-06-12 current-route approach-offset batch around frames `430..477`
  found no usable centre-`EC2D` contact. The most interesting right-side
  candidate, `approach_br448_465`, reaches `(584,543)` raw `dx=-10 dy=-1`
  by frame 559 but takes native DripsSurfaceCrawler damage at frame 549
  (`kid_projectile_damage_probe=hit:DripsSurfaceCrawler@0x024a2000,0x02500000`)
  and is not oracle-comparable: `verify-tas --frame-start 500 --frame-end 559`
  first diverges at frame 500 on HP (`native=1`, `oracle=0`). Treat this as
  another phase-shifted route rejection, not a Drips hitbox or block-break lead.
  2026-06-12 follow-up: pre-damage clears on the same
  `.cache/debug/issue260-approach-br448-465.json` candidate do not produce a
  usable route. Clearing `524..528`, `525..528`, `520..528`, or `512..528`
  keeps Kid healthy but collapses back to the old `dx=+1 dy=-20 @473` closest
  approach; clearing only `528` or `526..528` preserves the raw late approach
  but still enters death windup by frame 559. Do not repeat this pre-damage
  left-window-clear threshold axis without changing the earlier route shape.
  A post-`513b36d1` hybrid route batch over the named drop-contact probe also
  failed to find a live right-side contact: clearing `300..559`, applying an
  early `380..388/396 B+Right` pulse plus a later `424..504 B+Right` pulse,
  and trying left/right/no recovery ranked only `LevelTransition` candidates.
  The best right-target score was `(584,543)` healthy `dx=+15 dy=-26 @431`,
  worse than the existing `wait390`/`left400` near-misses; do not repeat this
  two-pulse hybrid axis unchanged. A later post-contact climb probe on
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493.json`
  (`540..600 B`, with nearby `520..620` climb variants) improved native raw
  proximity to `(584,543)` at `dx=-2 dy=-1 @569`, but it is not usable evidence:
  closest healthy was only `dx=+1 dy=-20 @485`, and enemy-frontier oracle bisect
  on derived cache scenario
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493-b540-600.json`
  first diverges at frame 560 on HP (`native=0`, `oracle=1`) after native-only
  DripsSurfaceCrawler damage. Do not repeat post-contact B-climb trimming as
  metal-block proof without a route-shape change. A follow-up avoid-then-climb
  family around the same damage window (`548..562`, `548..566`, `552..566`
  right/B+right/left avoidance, then `B`/`B+Left`/`B+Right` recovery through
  620) also ranked worse than the base route: best healthy right-target approach
  stayed `dx=-20 dy=-48 @549`. Do not repeat small avoid-window trims unless
  the pre-548 route shape changes. A pre-548 setup batch found `480..520 B+Left`
  can align native horizontally with `(584,543)` at `dx=0 dy=-48 @528`, but
  derived scenario
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493-bl480-520.json`
  first diverges against oracle at frame 519 on HP (`native=1`, `oracle=0`);
  `tas-diff` shows oracle has already taken DripsSurfaceCrawler damage while
  native has not, so it is another phase-shifted setup rather than metal-block
  proof. The immediate `530..600` B/B+Left/B+Right climb continuations also do
  not improve the healthy score. Do not reuse this `480..520 B+Left` setup seed
  as proof. 2026-06-12 Iron Knight route-shape check: refreshed
  `issue260-drips-iron-knight-climb-probe`, extended it cache-locally to 700
  frames, and probed post-536 wall-climb overlays. Best new left-target
  approach was `500..535 Left` then `536..700 B+Left`, reaching `(456,543)`
  healthy `dx=-1 dy=-23 @552`; `verify-tas --scenario
  .cache/debug/issue260-iron-extended/ik-l500-bl536.json --frame-start 500
  --frame-end 560 --checkpoint-interval 1 --frontier-kind enemy --bisect`
  passed with no authoritative divergences. This is useful oracle-comparable
  route evidence, but still not row-32 slime/metal-block contact; do not repeat
  Iron Knight late B+Left wall-climb trimming unless the pre-500 route shape
  changes. 2026-06-12 follow-up: a small pre-500 replacement batch over
  `.cache/debug/issue260-iron-extended/ik-l500-bl536.json` changed the setup
  before the late Iron Knight wall-climb tail (`300..499` left-return, high-arc,
  brake-drop, and slingback variants). Best candidate was `pre300_high_arc`,
  still final `InGame` with `lives/hp 3/3`, but only reached `(456,543)` at
  healthy `dx=0 dy=-17 @546`; right target stayed at the old `dy=-48` class.
  This does not improve on the existing comparable near-miss and should not be
  mined as Drips/block proof. A fresh post-300 right-run/brake axis over
  `issue-260-drips-drop-contact-probe-current/scenario.json` (`300..400/430/450`
  clears, `400..520` right windows, `500..555 B+Right`, and early left brakes)
  also failed: the best raw right-target proximity was `dx=+3 dy=-8` around
  frame 456, but Kid was already unhealthy or fell into LevelTransition; every
  healthy closest stayed at the old early `dx=0 dy=-48 @128`. Do not retry this
  post-300 right-run/brake family without a different vertical setup. A 2026-06-11
  early reverse/jump sweep over `160..320` also only reproduced the known
  `(456,543)` `dx=0 dy=-16` near-miss and was not oracle-comparable at the late
  window (`number_hitpoints` native 2 vs oracle 1 at frame 470); do not retry
  that reverse-entry family unchanged. On the comparable left-tail seed
  `.cache/debug/issue260-tail-grid/l524_b530_540.json`, a 360..460 middle-setup
  batch (`B`, `B+Left`, `B+Right`, `Left`, `Right` over `360..420`,
  `380..440`, `400..460`) also failed: B-only variants remain the old clean
  `(456,543) dx=0 dy=-16 @540`, left/B+Left worsen vertical approach, and
  right/B+Right variants improve only invalid LevelTransition routes. Do not
  repeat this middle-setup axis unchanged. 2026-06-11 follow-up: three more
  route-shape batches failed to improve the current near-misses. On
  `.cache/debug/issue260-route-probes/drop-contact-br460-492-l493.json`, early
  420..520 brake/coast/B+Left/B+Right overlays over the right-side route still
  leave the best healthy `(584,543)` approach at `dx=+1 dy=-20`, and the best
  left-target approach only `dx=-1 dy=-20`. A follow-up
  `320..390 Left` / `390..460 B+Right` reverse-entry axis on the named
  drop-contact route still produced only the old left-side `dy=-16` near-miss
  and diverged against oracle at frame 470 on diamonds; do not repeat that
  axis unchanged. On
  `.cache/debug/issue260-tail-grid/l524_b530_540.json`, 430..520 B-pulses,
  left/right shaping, and B+direction variants preserve or worsen the existing
  `(456,543) dx=0 dy=-16 @540` clean seed. A final underside-target batch
  scored `(456,527)` directly with 500..550 B/coast/left/right variants; every
  healthy result was worse (`best dx=+2 dy=-32` to underside, `dx=+2 dy=-16` to
  row-32). Do not repeat these 420..550 shaping axes without a different
  bootstrap/route family. 2026-06-11 follow-up after #253 queue correction:
  a cleared 220..460 / 240..420 / 260..430 mid-route replacement batch
  (`pre240_clear_*`, `pre260_*`, `pre300_*`, `pre220_*`) also failed to improve
  the current seed. Every candidate either entered `LevelTransition` or stayed
  below the targets; best healthy right-target result was still `dy=-33..-48`,
  and best left-target result was worse than the existing clean `dy=-16` class.
  Do not retry this broad 220..460 mid-route replacement axis unless the
  bootstrap/route family changes. 2026-06-12 follow-up: early-vertical route
  batches over `.cache/debug/issue260-tail-grid/l524_b530_540.json` also failed
  to improve the seed. Varying the first jump (`55..145`, `70..135`, split
  holds), early left-air setup, and early right recovery (`190..360`) either
  reproduced `(456,543) dx=0 dy=-16 @540`, worsened to `dy=-48`, or made a
  different-but-useless left-air branch at `dx=+79 dy=-1 @548`; a targeted
  `548..620 B+Right` recovery still only found an early `dx=+2 dy=-17 @66`.
  Do not retry this early-jump/left-air/right-recovery family unchanged.
  Another 2026-06-12 alternate-bootstrap batch over the named contact probe
  cleared `0..239`/`0..299`, rebuilt early low/high arcs, left-air recovery,
  slowdown, double-jump, and right-sling shapes, then restored the known
  `478..528 Left`, `530..559 B`, `536..559 Right` tail. Scoring after frame
  300 still failed: best live result was `early_left_air_recover` at
  `(456,543) dx=-19 dy=-16 @539`, while row-aligned variants stayed far left
  (`dx=+78 dy=0 @546`) and right-biased variants transitioned. Do not repeat
  this alternate `0..299` bootstrap rebuild unless the base route changes.
  A narrower 2026-06-12 Iron Knight continuation check from the named
  `issue-260-drips-iron-knight-climb-probe-current` seed also failed: post-536
  B+Left/B+Right holds through 620/660/700/760 and directional recoveries to
  820 ranked no better than `(456,543) dx=-5 dy=-39` and `(584,543)
  dx=+55 dy=-16`, so the direct Iron Knight post-tail route moves away from
  centre `EC2D` contact. Do not repeat this post-536 Iron Knight continuation
  axis without changing the pre-536 route shape. A distinct pre-478 slingback
  family over the named contact probe (`0..160 Right+B`, `161..220 Left`,
  `221..420 Right+B`, `421..477 Left`) also failed after tail refinement:
  best live result was `sling_tail_right_early` at `(456,543) dx=-1 dy=-17`
  and `(584,543) dx=0 dy=-48`, with a focused probe staying `InGame`/HP 1 and
  showing only the earlier floor `DripsSurfaceCrawler` window, not centre-cell
  block contact. Do not repeat this pre-478 slingback family unchanged.
  2026-06-12 follow-up: two fresh route-shape batches also failed to improve
  the centre `EC2D` target. On oracle-comparable
  `.cache/debug/issue260-iron-extended/ik-l500-bl536.json`, first-jump,
  early-left, slingback, reverse/coast, and `260..430` up/down/B variants still
  ranked no better than the existing `dy=-23` left-target class or
  right-target `dx=-2 dy=-20`; every row-aligned right-side improvement stayed
  below the row-32 line. On the current
  `.cache/recordings/issue-260-drips-b-jump-climb-seed-current/scenario.json`,
  pre-360 lift/drop, double-rise, late-switch, and `430..580` lift variants
  left the best live/nonzero-HP candidate at `(456,543) dx=+6 dy=-19 @488`,
  worse than the baseline, while closer right-side passes were HP-zero or
  `LevelTransition`. Do not repeat these first-jump/early-sling/pre-360
  vertical setup probes unless the base route changes. 2026-06-13 follow-up on
  the same current b-jump climb seed: `probe-record-scenario-file-batch` found
  a small improvement by holding `170..260 B+Left`, reaching `(584,543)`
  healthy `dx=-4 dy=-17 @513` (manhattan 21, better than the old 23px miss).
  Focused `probe-record-scenario-file --input 170-260:b+left --target-projectile
  drips-surface-crawler` shows the crawler window is still at y `$0250` and the
  closest Kid/crawler separation is `dx=+13 dy=+21 @509`; Kid reaches HP 0 by
  the tail. A follow-up continuation sweep from that brake shape found
  `500..620 Left`/`B+Left` improves the healthy target score to `(584,543)`
  `dx=+1 dy=-18 @515` with HP 2, but focused probing still leaves crawler
  separation at `dx=-5 dy=+21 @521`; tighter `480..620`/`490..590`/`510..610`
  left/B+Left windows either keep the same manhattan-19 miss or collapse back
  to the old floor pass. Treat this as another route-shape near-miss, not
  metal-block proof, and do not repeat the `170..260 B+Left` plus `500..620
  Left/B+Left` brake axis unchanged. 2026-06-13 follow-up: `wildside-cli` now accepts
  `--target-cell col,row` for single and batch probe scoring, so #260 route
  search can rank actual 16x16 centre-`EC2D` block-cell contact instead of a
  guessed point. On `issue_260_drips_drop_contact_probe_01`, scoped to
  `--frame-start 520`, the current route remains 32px below left cell `28,32`
  and 157px away from right cell `36,32`; a small baseline/late-B/late-BR batch
  all tied at the same 32px left-cell miss. Do not re-run those unchanged late
  variants as metal-block proof. 2026-06-13 follow-up: `wildside-cli` also accepts
  `--target-cell-hitbox col,row`, which scores Kid's ASM `loc_902A`/`unk_B408`
  upper/body bounds against a 16x16 cell. On
  `issue-260-drips-b-jump-climb-seed-current`, the baseline stays alive and
  reaches right cell `36,32` at hitbox `dx=+1 dy=-2` (manhattan 3) at frame
  503; `early_br440_540`, `early_br450_550`, and `neutral430_jump470` improve
  only to manhattan 2 and all end HP 0. Treat the seed as a confirmed
  near-contact route, not metal-block proof, and do not reject it based on
  origin-only cell scoring. A follow-up late-climb reshape that clears
  `430..620`, applies `460..520 B+Right`, exits `521..560 Left`, then restores
  `561..620 Right` stays alive at HP 1, improves the right `EC2D` cell hitbox
  score to `dx=0 dy=-2 @483`, and verifies oracle-clean on the materialised
  cache scenario through frames 430..620. It is the best current comparable
  near-contact route, but focused probing still leaves the DripsSurfaceCrawler
  far away (`dx=-58 dy=+1 @575`), so it is not the reported slime/metal-block
  contact. A fresh `floor_sling` family on
  `issue-260-drips-drop-contact-probe-current` changed the bootstrap (`220..300
  B+Left`, `301..390 B+Right`, `391..462 Left`, `463..512 Right`) and found
  a stronger native-live right-side near-contact: `513..559 B+Right` reaches
  cell `36,32` at Kid-hitbox `dx=0 dy=-1 @522` while staying `InGame`/HP 1.
  This is still not slime/metal-block proof: the closest DripsDrop separation
  is `dx=-5 dy=-36 @529`, and the only DripsSurfaceCrawler frame is at 559
  with `dx=-67 dy=+1`. Do not repeat this `floor_sling` hand-off family
  unchanged unless a new bootstrap changes the Drips/Kid phase.
  2026-06-14 follow-up route rejection:
  `.cache/debug/issue260-prebl-r330-bl390-20260614/oracle/oracle-frame-scenario.json`
  (`240..470 none,250..320 B+Left,321..370 Right,371..430 B+Left,
  470..530 B+Right,531..560 Left,561..620 Right`) verifies enemy-frontier
  clean through frames `300..680`, and advisory `tas-diff` frames `640..680`
  reports 40/40 matches. It is a useful different route shape but still not
  the reported slime/metal-block proof: Kid reaches cell `28,32` at healthy
  `dx=-4 dy=-1 @380`, DripsDrop is only `dx=+8 dy=-11 @401`, and the later
  DripsSurfaceCrawler reaches Kid at `dx=+1 dy=+1 @665` on the lower floor
  while remaining `dy=-65` from row-32 cells. Do not repeat this
  pre-250/B+Left sling plus late B+Right/Left/Right tail as metal-block proof
  without changing the Drips cycle or vertical route.
  Follow-up survivor seed:
  `.cache/debug/issue260-prebl-longr-left-right-tail-20260614/oracle/oracle-frame-scenario.json`
  (`240..470 none,250..320 B+Left,321..390 Right,391..430 B+Left,
  470..530 B+Right,500..530 Left,531..700 Right`) verifies enemy-frontier clean
  through frames `450..700`, remains InGame with HP 1, and improves the live
  close-window evidence: DripsDrop separation is `dx=+1 dy=+2 @475` while Kid
  reaches cell `28,32` at healthy `dx=0 dy=-1 @480`. This is still not the
  reported slime/metal-block proof because the crawler only appears at frame
  700 on the lower floor, `dx=-55 dy=+1` from Kid and still vertically separated
  from row-32 cells. Use this as the current clean alive route seed; the next
  useful search needs to alter the Drips cycle or vertical route so the child
  reaches the row-32 block line, not just keep trimming the post-500 recovery.
  Drips-trace clarification: on this seed `tas-diff --drips-trace` frames
  `468..482` shows #07 falling through cells `28,34`/`28,35`/`28,36`, then
  splashing at frame 477 and reseeding at frame 478; the row-32 score is only
  a Kid/cell proximity metric, not a `$0C` splash on the reported block line.
  2026-06-14 follow-up on the same clean alive seed: a pre-splash/cycle batch
  over frames `360..470` (`B`, `B+Left`, `B+Right`, `Left`, `Right`, and
  split `B` then direction variants) still produced no row-32 block event.
  Baseline and right-biased survivors remain best for Kid/cell at
  `dx=0 dy=-1`, but DripsDrop still only reaches cell `28,32` at `dy=-3 @373`;
  the closest useful survivor crawler remains on the lower floor (`dy=-65`
  from cell `28,32`). Do not repeat this `prebl-longr` `360..470`
  pre-splash axis unchanged.
  A mid-bootstrap batch on the same clean alive seed over `120..260` also
  failed to create metal-block evidence. Candidate `mid_b140_240` was
  materialised as `.cache/recordings/issue-260-prebl-longr-mid-b140-240/scenario.json`
  and verifies clean for the block and crawler windows (`verify-tas` frames
  `450..500` and `620..670`, enemy frontier). It keeps the route alive at HP 1,
  reaches row-32 cells `28,32`/`29,32` at hitbox `dx=0 dy=-1 @480`, and bounces
  `cell 29,32 @480`, but `Destroy` is absent, the closest DripsDrop is still
  `dx=-17 dy=-11 @432`, and the crawler only reaches Kid later on the lower
  floor (`dx=+16 dy=+1 @649`, still vertically separated from row-32 cells).
  Do not repeat this `prebl-longr` `120..260` mid-bootstrap axis unchanged.
  2026-06-14 early-bootstrap check on the same seed (`0..120` / split
  `0..70,71..140` with `None`, `Right`, `B+Right`, `Left`, `B+Left`, `B`)
  also failed to create metal-block evidence. `early_none` and `early_r`
  reproduce a Kid/cell `dx=0 dy=-1` near-miss, but the Drips child is either
  shifted to the right-side column (`DripsDrop cell 28,32 dx=-121`) or remains
  the baseline lower-line splash (`dy=-3`). Split variants can bring a crawler
  close to Kid on the lower floor (`split_l_r` crawler `dx=0 dy=+1 @669`) but
  still leave it `dy=-65` from row-32 cells. Do not repeat this `prebl-longr`
  early-bootstrap cycle axis unchanged.
  A cache-local form-axis check on the same seed also failed to produce a
  metal-block proof. Red Stealth (`playtest_form=3`), Juggernaut (`5`), and
  Iron Knight (`6`) variants all stayed InGame, but none produced row-32
  `Destroy`/`Bounce` on cells `28,32`/`29,32` from a live Drips child.
  Juggernaut changed Kid's vertical route enough to keep healthy row-32 hitbox
  proximity at `dx=0 dy=-1 @405..407`, but the closest DripsDrop was still far
  from Kid/cell (`dx=-34 dy=-11 @178`; cell `28,32` `dx=-121 dy=-3 @332`)
  and the crawler still emerged on the lower floor (`dx=+16 dy=+1 @662` from
  Kid, `dy=-65` from row-32 cells). A bounded Juggernaut `300..430` timing
  batch preserved the same separation and reported no target block events, so
  do not repeat simple form swaps on this seed without a new Drips cycle or
  vertical route.
  2026-06-15 follow-up on the independent
  `issue_260_dlb_tail_b_row32_bounce_01` route: the new named
  `issue260-gr-pulse600` scenario overlays `431..500 Right` plus B pulses at
  `600..610`, `620..630`, and `640..650`. The materialised cache candidate
  verified enemy-frontier clean through frames `580..660`, and advisory
  `tas-diff` frames `636..652` reported 16/16 matches. It improves late
  DripsDrop/row-32 conjunction around frame 650 (`cell 36,32` Kid
  `dx=-7 dy=-1`, drop `dx=0 dy=-9`; `cell 37,32` Kid `dx=0 dy=-1`, drop
  `dx=+8 dy=-9`) and keeps the row-32 bounce at `cell 37,32 @610`, but it is
  still not the reported mutation: no destroy event occurs, and focused traces
  show the projectile separated from Kid/block contact
  (`kid_projectile_damage_probe dx=-19 dy=-23 overlap=0`). Use it as a durable
  oracle-clean near-miss seed; do not treat it as metal-block proof. Follow-up
  same-seed batches reject the obvious late-tail/setup variants: `520..680`
  left/right/B pulses can improve the drop-to-cell Y to `dy=-3`, but they pull
  Kid down/off line (`best pulse_early score=18`, Kid `dy=-14`) and still only
  bounce `cell 37,32 @590`; `431..500` setup/turn variants either reproduce
  the baseline (`late_right`, score 17, bounce `cell 37,32 @610`) or move Kid
  far away/enter `LevelTransition`. Do not retry this gr-pulse setup/tail axis
  without changing the Drips cycle or vertical route. 2026-06-15 follow-up:
  an earlier pre-431 route-shape batch over the same `issue260-gr-pulse600`
  seed also failed. Replacing `300..430` or `330..430` with `none`, `B`,
  `Right`, `B+Right`, `Left`, `B+Left`, and short split direction/jump windows
  either reproduced the baseline exactly (`pre300_b`/`pre300_br`/`pre300_r`:
  cell `37,32` Kid `dx=0 dy=-1 @610`, DripsDrop still `dy=-9 @650`, final
  HP 0) or kept Kid alive only by moving him far left of the target
  (`split_l_br` best alive score 107). This confirms the current near-miss is
  not sensitive to simple pre-431 input replacement; next useful work needs a
  different bootstrap/Drips cycle or new live recording, not more pre-300/pre-330
  trims on `issue260-gr-pulse600`.
  2026-06-16 engine-parity follow-up: after aligning Drips splash hibernate,
  `loc_3340E` Kid-gfx-x reseed timing, Drips crawler RNG order, RockTank
  mouth-child contact damage, and death-tumble movement trace, the materialised
  `issue-260-gr-pulse600-current` route verifies enemy-frontier clean from
  frames `1..700`. It is still only an oracle-clean near-miss: keep #260 open
  until a route proves the reported live slime/metal-block mutation.
  2026-06-15 follow-up on `.cache/recordings/issue-260-pre390-survivor-current/scenario.json`
  rejects the earlier B-window route-shape axis too. Broad `260..360` input
  replacements showed `pre260_b` can put Kid's healthy hitbox within 1 px of
  row-32 cell `36,32`, but it produces no row-32 Bounce/Destroy and leaves the
  closest DripsSurfaceCrawler 65 px below the target row. A tighter B-window
  sweep (`220..320` through `300..400`) repeats the same pattern: best
  `b220_320`/`b220_340`/`b220_360` put Kid at `dx=0 dy=-1 @489`, while the
  DripsDrop conjunction is still separated vertically (`projectile dy=-53`) and
  no row-32 block event occurs; later B windows only reproduce the old cell
  `37,32` bounce with the Drips child far below. Do not repeat simple
  pre-260/pre-300 B-window overlays on this seed; the next useful #260 route
  search still needs a different bootstrap/Drips cycle or a new live recording
  that brings the child onto the row-32 cell before Kid's block contact.
  2026-06-16 follow-up: a bounded mid-bootstrap batch on
  `.cache/recordings/issue-260-gr-pulse600-current/scenario.json` varied
  `120..430` windows and scored same-frame Kid hitbox plus DripsDrop/crawler
  proximity to cells `36,32`/`37,32`. Best `lowarc_b150` only reproduces the
  documented vertical split (Kid `dy=-33`, DripsDrop `dy=-3`, no row-32
  Bounce/Destroy), while bounce-preserving candidates still keep the projectile
  separated from the row-32 cells. Do not retry this `120..430` mid-bootstrap
  replacement axis on the `gr-pulse600` seed unchanged.
  #270 current wall-pulse probes split the evidence:
  Kid's natural BLW2 left-wall route with B held at `285..330` is gameplay CRC-clean through frame
  380, so Kid upward wall movement with B is ROM-faithful; direct Samurai `playtest_form=3` variants
  diverge before the B pulse and need a natural pickup/well route before any movement patch. #285 crawler art);
  2026-06-11 follow-up: cache-local natural Red Stealth route
  `.cache/debug/issue270-wall-launch/red_stealth_left_wall.json` is a better seed than the
  direct `playtest_form=3` probes. Before commit `1c06f5d8`, it blocked before proving the wall launch:
  `verify-tas --frame-start 0 --frame-end 520 --checkpoint-interval 1 --frontier-kind gameplay`
  first diverged at frame 266. `tas-diff` frames 250..272 showed oracle already in the
  Red Stealth wall/jump path (`kid.asm loc_A6F8 -> loc_A74E -> sub_A4EE -> loc_A75A`,
  trace code `0x00A71C`) with leftward x drift (`kid_x_pos 0x00660000`, then
  `kid_x_vel 0xffffc000`), while native stays pinned at `0x00670000`/`x_vel=0` with
  `kid_wall_cling_release_probe branch=sub_ADB8 sub_ADB8=column_shape_blocked`. Fixed candidate
  `1c06f5d8` allows the sampled `loc_AB0A` broad-probe release on either wall; the natural route now
  passes gameplay CRC through frame 520. Treat #270 as proposed fixed pending human validation of
  the original well-hug launch.
  #277 left-jump input drop reconfirmed after the loc_7464 candidate fix; the
  existing natural BLW2 route is not the repro because its frame-207 L+B jump
  stays CRC-clean and honours held-left through loc_7664 -> loc_A426 -> sub_B084;
  a later two-pulse variant (`205..245` and `270..300` Left+B) is also not the
  repro: frame 272 takes the jump edge, the following airborne frames keep
  negative held-left x velocity, and frames 260..310 pass gameplay CRC. A
  2026-06-12 later lower-left edge scan over the same clean fixture (`300..330`,
  `320..350`, `340..370`, `360..390`, `400..430`, `500..540`, and short
  `310`/`330`/`360` Left+B pulses) is also not the repro: frame 302 consumes
  Left+B as `jump_pressed`, then the level-edge clamp zeros x velocity only
  after several held-left airborne frames. Next pass needs a direct
  human-equivalent lower left-wall/stair jump repro. A 2026-06-12 `LEFT+C`
  variant of the original BLW2 route is invalid too: Kid is already HP 0/death
  windup at the wall-hold checkpoint and enters `LevelTransition`, so it never
  exercises the reported wall-jump input path. A 2026-06-11 lower-left-wall
  scratch route (`lower-left-wall-b-current`) is also not the repro: native
  keeps the held-left wall/jump path, while the
  first oracle split is a Drips controller/object-count mismatch at frame 103;
  a 2026-06-13 current-HEAD stair-route audit rejects
  `.cache/recordings/issue277-blw2-stair-jump-right-current/scenario.json`
  because it passes frames 0..180, and rejects the paired left route as an
  input signal because the first authoritative split is native
  ShooterBlock/stomp-bounce handling at frame 88/90, before it can prove a
  dropped-left jump. Fixed candidate `de869f33` closes the mirrored native
  floor-contact gap found during the latest playtest follow-up: ASM
  `sub_902A -> loc_A984 -> loc_A99E` also promotes leftward leading-edge
  SlopeDown recontact in the same frame, while native's row-boundary guard only
  allowed the rightward mirror. Regression
  `level00_airborne_leading_edge_slope_down_left_recontact_enters_loc_9d22_same_frame`
  now lands the leftward SlopeDown case without weakening the existing
  trailing-edge guard. 2026-06-14 rerun keeps that conclusion: the focused
  held-left input and leftward SlopeDown regressions pass; the paired direct
  stair routes split exactly as before. Right route
  `.cache/recordings/issue277-blw2-stair-jump-right-current/scenario.json`
  verifies frames `0..180` with no authoritative divergence, while left route
  `.cache/recordings/issue277-blw2-stair-jump-left-current/scenario.json`
  first diverges at frame 90 on `number_objects` plus camera after native
  ShooterBlock damage; `tas-diff` frames `86..94` shows frames 86..89 as
  non-authoritative object-count oracle limitations before the damage path.
  Keep #277 open for a fresh human-equivalent stair/wall route, not another
  patch from this blocked direct left-stair fixture.
  #291 stale #249 gate assertion re-derived (tail sample is airborne after walking off support).
- **Gates**: verify-slice-fast green. Full published TAS
  `verify-tas --scenario tests/data/tas/any-percent-published/scenario.json
  --frame-start 0 --checkpoint-interval 1 --bisect --frontier-kind gameplay`
  passes all 37759 frames by CRC.
