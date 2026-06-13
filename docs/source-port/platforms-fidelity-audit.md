<!-- Generated 2026-05-31 by the `platforms-helmets-weather-fidelity-audit` workflow (1 read-only Explore scout enumerated the platform units; one read-only Explore auditor per unit compared the live native path to kid.asm; every high/medium gap adversarially verified by an independent refuter — defaulting to refuted — before inclusion; low gaps included without a refute pass). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native `file:line` searched. Re-run the workflow to refresh. -->
<!-- Staleness-checked 2026-06-10: of 40 gaps, 5 fixed, 3 partially fixed, 32 still open — verdicts and evidence in fidelity-audit-staleness-2026-06-10.md. 2026-06-13 follow-up: crawl-on-platform sub_8ED0 width, type-0 platform height, map-order tail lookup, invented platform preload padding, scripted loop/fall-through targets, scripted-position wrapping, stored-platform sub_8ED0 support, lift initial y-velocity, lift ceiling crush, SoftPlatform frame-start launch velocity sampling, SoftPlatform settled wait helper wiring, TrapPlatform pp=4/8 dispatch, and TrapPlatform wait/move loops are fixed, leaving 18 still open. Still-open entries below remain authoritative; fixed entries are superseded. -->

# Platform Fidelity Audit

Adversarially-verified audit of the Kid Chameleon source-port's **platform** subsystem (`platforms.rs`, with `enemies.rs` platform load/activation, `blocks.rs` lift blocks, `types.rs` field decode), comparing the live path to the `kid.asm` platform routines.

The recurring failure mode is the same one the enemy/movement/level-loading audits found crate-wide: **ASM-faithful pure helpers exist and are unit-tested, but carry `#[cfg_attr(not(test), allow(dead_code))]` (or are `#[cfg(test)]`-gated) and are never called from the live path.** Platforms are an acute case — almost the entire `Platforms_CheckCollision` escape/carry algorithm, the slot allocator, the sprite-tiling compiler, and the Trap-platform coroutines are modelled helper-for-helper yet dead, while the runtime uses a separate approximation.

Findings that are **not** simple modelled-but-unwired gaps and deserve human verification before acting:

- **Architectural — the ROM's 17-slot free-list object model is replaced by a monotonic high-water index plus a per-frame filter.** `Initialize_Platforms`/`Allocate_PlatformSlot`/`Deallocate_PlatformSlot` (a linked free-list of 17 `$22`-byte slots at `$FFFFEDBA`, `kid.asm:4166-4251`) have no native counterpart: `PlatformLoaderState` is a single `next_index:usize` (`types.rs:2461`) that is only ever incremented, and the live set is recomputed every frame via `take(next_index).filter(platform_is_active)` (`enemies.rs:40310-40341`). There is **no 17-slot cap and no `Number_Platforms`**, so a window holding >17 platforms over-spawns where the ROM silently drops the overflow (`loc_6E2`).

- **Current-code spot check, 2026-06-13.** The original "two genuine value bugs to verify" are no longer live: placement x/y now route through `platform_layout_coord_fixed_loc_2762` (`types.rs:2263,2277`; `platforms.rs:509-518`), and spawned lift platforms now seed `y_velocity: -0x0001_0000` with ASM-cited regressions (`blocks.rs:3709`; `tests/runtime_tests.rs:11099-11102`; `tests/level_tests.rs:19956-19958`). Treat those rows as superseded by `docs/source-port/fidelity-audit-staleness-2026-06-10.md`.

- **Current-code spot check, 2026-06-13.** The original "`sub_8ED0` support check hardcodes Kid half-width" claim is also superseded: the live carry caller now passes `current_form.half_width()` and `current_form` (`platforms.rs:711-718`), with `platform_carry_uses_current_form_half_width_loc_8ed0` covering the behaviour.

- **Current-code spot check, 2026-06-13.** TrapPlatformDown and TrapPlatformUp are no longer entirely inert: `pp=4`/`pp=8` now dispatch through native `PlatformSpecialState` variants and the live special-platform step runs their wait/move loops. The remaining trap-platform gap is lifecycle deletion: native still lacks the ROM `$12(a3)` deactivation flag and `Deallocate_PlatformSlot`/`Delete_CurrentObject` tail.

## Coverage summary

| Unit | Coverage | Confirmed gaps |
|---|---|---|
| Slot free-list init & alloc/dealloc | stub | 2 |
| Layout address indexing (`MapPtfmLayout_Index`) | partial | 2 |
| Platform-loading manager (scroll-window load/unload) | partial | 5 |
| Slot→ActivePlatform field decode & sprite-list build | partial | 3 |
| Scripted platform per-frame movement (`Move_Platforms`) | partial | 3 |
| Kid riding & carry collision (`Platforms_CheckCollision`) | stub | 6 |
| Platform support / standing-state checks (`sub_8ED0` family) | partial | 4 |
| SoftPlatform special coroutine | partial | 2 |
| TrapPlatform special coroutines (Up/Down) | stub | 4 |
| Lift / elevator blocks (`sub_FCA6`/`FD00`) | partial | 5 |
| Activation window & preload thresholds | stub | 4 |

Totals: 11 units audited, 0 full, 8 partial, 3 stub. **40 confirmed gaps** (16 high, 12 medium, 12 low).

## Confirmed gaps

### High severity

**Slot pool — free-list replaced by monotonic `next_index`; no allocation model at runtime** (`Initialize_Platforms/sub_231C`, `Allocate_PlatformSlot/sub_238E`, `Deallocate_PlatformSlot/sub_23B4`)
The ROM builds a 17-entry pool of `$22`-byte slots at `$FFFFEDBA` as a linked free-list, pops the head on alloc (incrementing `Number_Platforms`), and splices slots back on dealloc, with all active-platform processing iterating the `Addr_FirstPlatformSlot` linked list (`kid.asm:4166-4251,4270…17115`). Native models none of this: `PlatformLoaderState` (`types.rs:2461-2463`) is a single monotonic `next_index`, and the runtime (`enemies.rs:40310-40341`) recomputes `active_platforms` each frame via `take(next_index).filter(platform_is_active)` (`platforms.rs:597-627`). No slot pool, no free-list pop/push, no deallocation, no `Initialize_Platforms`. The faithful `allocate_platform_slot_sub_238E` (`platforms.rs:400-424`) and `deallocate_platform_slot_sub_23B4` (`440-486`) are `#[cfg(test)]`-only (`block_tests.rs:13354…13441`).

**Platform loader — range check uses camera-based enemy window + relative buffers instead of Kid position + absolute (byte×32) edges** (`Manage_PlatformLoading/sub_2744, loc_2762`, `kid.asm:4684-4709`)
The ROM loads Kid's X/Y from `Addr_GfxObject_Kid` and range-checks each placement using the buffer **bytes as absolute world edges** scaled by `asl #5` (×32): `bufL×32 > KidX` out-left, etc. Native `platform_is_active` (`platforms.rs:631-649`) instead compares `placement.x/.y` against the **camera** `ActivationWindow` (`enemies.rs:8543-8551`) and treats `buffer_left/right/top/bottom` as small deltas around that window — wrong reference point (camera vs Kid), wrong buffer semantics (relative delta vs absolute ×32 edge), and the ×32 scale is never applied. Load/unload timing diverges.

**Platform loader — placement x/y decode omits `asl #4` (×16), so loaded position is 16× too small** (`Manage_PlatformLoading loc_286E`, `kid.asm:4717-4723`)
The ROM multiplies the layout x/y words by 16 before storing (`asl.w #4,d7`; the `ptfm` words are 16px block units). Native `from_placement` (`types.rs:2085,2099`) and `active_platforms` (`platforms.rs:608-609`) store `x_pos = placement.x << 16` / `y_pos = placement.y << 16` with **no ×16** (`placement.x` is the raw `ptfm` word; `lib.rs:1304`/test `lib.rs:5482` confirm `ptfm 24 → placement.x==24`). Downstream collision reads `x_pos>>16` as pixels (`platforms.rs:811-812`), so platforms land 16× closer to origin. **Highest-impact claim — verify against a running level.**

**Sprite emission/tiling is dead; live renderer draws one fixed frame (no width/height tiling)** (`Make_SpritesFromPlatforms / loc_244C + off_24AC`, `kid.asm:4325-4453`)
The ROM walks the active list, culls each against the 320×224 window using the `$1A/$1C` size words, and dispatches through `off_24AC` into 5 strip routines that emit `(w+1)×(h+1)` 16px cells via `loc_2548`. Faithful ports of all of these exist (`platform_sprite_dispatch_loc_244C` `platforms.rs:997`, `emit_platform_sprite_loc_2548` `1058`, the strip helpers `1094-1222`) but every one is `allow(dead_code)`, test-only. The live renderer `push_platform_tiles` (`crates/kidcli/src/native_player/sprites.rs:1439-1458`) draws a single fixed art frame per platform with no tiling, no culling, no variant dispatch; the snapshot drops `h_variant/v_variant/width_px/height_px` (`platforms.rs:603-607`, `types.rs:1488`). Multi-cell platforms render as one tile.

**`Platforms_CheckCollision` — entire escape-vector/flag algorithm modelled but unwired; live path is a snap-tolerance approximation** (`sub_2574, loc_257E…loc_2672`)
The ROM computes exact AABB overlap of Kid's hitbox vs every platform, derives both horizontal and both vertical escapes, takes absolute values, and pushes Kid out along the **smaller axis** (`kid.asm:4465-4578`). A complete faithful decomposition exists at `platforms.rs:96-382` — all `allow(dead_code)`, called only from `block_tests.rs:13112-13341`. The live `apply_platform_collision_with_ceiling_probe` (`platforms.rs:689`) instead uses `GROUND_SNAP_TOLERANCE_FP` distance checks and a single-axis top-only landing (`platforms.rs:725,823-848`), with no horizontal escape, no axis-magnitude comparison, no min-magnitude resolution. Kid cannot be pushed sideways out of a platform and the wrong axis can resolve.

**`Platforms_CheckCollision` — horizontal carry (loc_2686/26B6/26C0) omitted; Kid never pushed/carried sideways** (`loc_2686, loc_26B6, loc_26A2, loc_26C0`)
The ROM adds the horizontal push into Kid's hitbox and carries the platform's `$A(a2)` x-velocity into `$26(a0)`, clamped by push direction, setting wall-contact flags (`kid.asm:4583-4611`). Faithful helpers (`platforms.rs:231,258,311,328`) are dead (`block_tests.rs:13223-13266`). The live path applies horizontal velocity unconditionally and additively (`platforms.rs:780-781`) with no sign-clamp, no escape push, only in the on-ground branch — Kid is never pushed out horizontally when airborne.

**`Platforms_CheckCollision` — top-landing `$4000` carry bias and `y_vel`/`Addr_PlatformStandingOn` writes (loc_26D4/26FE) bypassed** (`loc_26D4, loc_26FE`)
On a top landing the ROM adds the vertical push into Kid, clears `y_vel`, writes `a2` into `Addr_PlatformStandingOn`, sets `FAA4`; on a bottom hit it loads the platform's `$E(a2)` y-velocity, adds `#$4000` downward bias, writes `$2A(a0)` (`kid.asm:4614-4634`). Faithful `platform_landing_loc_26D4` (`platforms.rs:344`) and `bottom_platform_carry_loc_26FE` (`370`, including `+0x4000`) are dead. The live path snaps to `plat_y-1` with floor-style sub-pixel logic (`platforms.rs:834-847`) and adds raw y-velocity additively (`782`) with no `$4000` bias and no bottom-face carry.

**`Platforms_CheckCollision` — FAA2–FAA5 contact flags, FA96 jump-platform ref, and `loc_271A` crush escalation never set** (`loc_2686, loc_26A2/26B6/26C0, loc_26D4/26FE, loc_271A`)
The ROM tracks four directional contact flags (`$FFFFFAA2-5`), writes `a2` into `$FFFFFA96` when `Character_Movement==MoveID_Jump`, and escalates to `loc_271A` (`$38(a0)=#4`, crush) when opposite-axis flags both fire (`kid.asm:4579-4637`). None of these cells exist in native state — they appear only as comments in dead helpers. The live path returns only `None`/`CrushDeath` (`platforms.rs:684-687`) and its crush is a ceiling-probe (`786-798`), not the FAA flag-collision squash. FA96 jump-platform riding and dual-axis crush are absent.

**`sub_8ED0` — live carry/support check hardcodes Kid half-width (5), ignoring current form** (`sub_8ED0 / loc_8EFA`, `kid.asm:10282-10306`; half-width `sub_B41C+unk_B408`)
The ROM loads the support half-width from `unk_B408[Current_Helmet]` (Kid 5, generic 7, Skycutter 10, Juggernaut 23, Micromax 4). The live `apply_platform_collision_with_ceiling_probe` calls `platform_support_check_sub_8ED0` with `PLAYER_HALF_WIDTH` (=5) and `PlayerForm::TheKid` hardcoded (`platforms.rs:714-719`) even though `current_form` is in scope. Every non-Kid form is supported at the wrong x positions. The sibling `find_standing_platform_index_loc_8ED0` (`platforms.rs:868`) passes `current_form.half_width()` correctly — only this carry caller is wrong.

**TrapPlatformDown — entire coroutine unwired; 40 level instances become inert platforms** (`TrapPlatformDown_Init`, `loc_41EE`, `off_40D2 index 1`, dispatch `loc_2876`)
Fixed 2026-06-13 for the wait and falling move loops: `ActivePlatform::from_placement` now maps `pp=4` to `PlatformSpecialState::TrapDown`, `trigger_special_platform_standing_loc_40D2` seeds velocity from Kid y velocity >> 2, and `step_special_platform_loc_40D2` runs the fall acceleration including the original `$40000`→`$20000` clamp bug. Regressions: `special_platform_dispatch_loc_2876_uses_off_40d2_indices`, `trap_platform_down_loc_4182_launches_from_kid_velocity_and_falls`, and `trap_platform_down_loc_4182_preserves_original_velocity_clamp_bug`. Remaining lifecycle deletion is tracked by the `$12(a3)` deactivation row below.

**TrapPlatformUp — entire coroutine unwired; 42 level instances become inert platforms** (`TrapPlatformUp_Init/sub_420A`, `loc_4230`, `loc_4270`, `off_40D2 index 2`)
Fixed 2026-06-13 for the wait and rising move loops: `ActivePlatform::from_placement` now maps `pp=8` to `PlatformSpecialState::TrapUp`, `trigger_special_platform_standing_loc_40D2` enters the rising phase when Kid stands on it, and `step_special_platform_loc_40D2` calls the existing `trap_platform_up_move_step_loc_4230` helper. Regression: `trap_platform_up_loc_420a_enters_rise_loop_when_kid_stands_on_it`. Remaining lifecycle deletion is tracked by the `$12(a3)` deactivation row below.

**Special-platform dispatch keys off `raw=="0"` instead of the `off_40D2` index (pp/4)** (`loc_2876 / off_40D2`, `loc_2836`, ptfm macro `(t<<4)|s`)
Fixed 2026-06-13 for shipped TrapDown/TrapUp entries: `from_placement` now dispatches `raw=="0"` to Soft, `raw=="4"` to TrapDown, and `raw=="8"` to TrapUp, with the live loop stepping those variants. Regression: `special_platform_dispatch_loc_2876_uses_off_40d2_indices`. Chain (`pp=12`, 5 instances) remains unimplemented.

**Lift platform — initial y-velocity invented as 0 (ROM is −65536 / −1px/frame up)** (`lift spawn tail`, `kid.asm:20994-20999`; `sub_FD00 kid.asm:21064`)
Fixed 2026-06-13: spawned lift platforms now seed `y_velocity: -0x0001_0000`, matching `move.l #$FFFF0000,$E(a3)` before `sub_FCA6` applies its `$708`-up/`$7D0`-down acceleration. Regressions: `process_block_effects_lift_spawns_platform` and the level test at `tests/level_tests.rs:19956`.

**Lift platform — ride/carry bypasses the `sub_8ED0` platform framework (bespoke ±[−2..4] foot snap)** (`j_Allocate_PlatformSlot install`, `loc_7886/loc_8DA0/sub_8ED0`)
Partially fixed 2026-06-13: lift mount/carry now uses `current_form.half_width()` for the horizontal support span, matching `sub_8ED0`'s `Current_Helmet` width table. Regression: `lift_platform_support_uses_current_form_width_sub_8ed0`. Remaining gap: native still uses a separate `LiftPlatform` type with the bespoke `(-2..=4)` foot-distance mount window and additive carry, rather than the full platform-slot overlap/snap/copy-y-velocity machinery (`loc_8DA0`).

**Platform activation — camera-window basis instead of the ROM's Kid-centred per-platform buffered edges** (`loc_2762`, `kid.asm:4684-4709`)
The ROM tests Kid's X/Y against each platform's own buffered edges (bytes `4-7(a4)` scaled `asl #5` = ×32) as the absolute world-space activation rectangle — no camera window, no fixed padding. Native `platform_is_active` (`platforms.rs:631-650`) builds an inclusive range from a camera-derived `ActivationWindow` and subtracts/adds the **raw** buffer bytes (no ×32). Double mismatch: range basis (camera vs Kid) and the omitted ×32 scaling (a buffer byte of 40 contributes 40px instead of 1280px). The faithful per-record helpers (`platforms.rs:515,524`) are dead.

**Platform preload — invented 64px X padding + camera basis replaces the ROM's per-record window** (`loc_2762 / loc_286E`, `kid.asm:4684-4709,4786`)
Partially fixed 2026-06-13: `advance_to_window` no longer adds the invented 64px `PLATFORM_PRELOAD_X_PADDING`, and the focused regressions now exercise the exact `loc_2762` `bgt` boundary on byte×32 left-buffer edges. Remaining mismatch: the live path still uses the `advance_to_window` high-water sweep plus `take(next_index).filter(...)` instead of the ROM's one-12-byte-record cursor with explicit alloc/dealloc.

### Medium severity

**Slot pool — 17-slot capacity / `Number_Platforms` cap counter not modelled** (`Initialize_Platforms/sub_231C`, `Allocate_PlatformSlot/sub_238E loc_23AE`)
The ROM pool holds exactly 17 slots; the 18th alloc on an empty free-list jumps `loc_23AE → loc_6E2` (alloc fails), and `Number_Platforms` (`$FFFFFA92`) is maintained by alloc/dealloc (`kid.asm:4168-4249`). Native has no capacity limit: `advance_to_window`/`active_platforms` (`platforms.rs:578-627`) can mark unbounded placements active. A window with >17 in-range platforms over-spawns vs the ROM, which drops the overflow. The only modelling is the `#[cfg(test)]` `sub_238E` increment, never wired.

**Platform loader — type-0 platform Y-size `subq.w #2,$1C(a3)` adjustment not modelled** (`Manage_PlatformLoading loc_2822`, `kid.asm:4744-4746`)
Fixed 2026-06-13: `ActivePlatform::from_placement` now applies the decoded type-zero `subq.w #2,$1C(a3)` height quirk before storing `height_px`. Regression: `active_platform_from_placement_loc_2822_keeps_collision_shape_without_modelling_art_words`.

**Platform loader — architecture mismatch: ROM one-record-per-frame cursor with explicit alloc/dealloc vs native full-rescan; dedicated helpers dead** (`Manage_PlatformLoading`, `sub_2728`, `loc_2850/286A/286E`, `loc_2752`)
The ROM processes one 12-byte record per call: range-check, `sub_2728` dedup-by-offset, alloc + stamp offset, or dealloc on out-of-range; advance `PlatformLoader_Offset` by `$C` and wrap at the `$FFFF` terminator (`kid.asm:4671-4801`). Native carries only `next_index` (`types.rs:2461`) — a monotonic high-water index that never wraps — and re-filters all placements every frame (`enemies.rs:40310-40340`), no per-frame budget, no offset cursor, no dedup, no explicit dealloc. Faithful `find_active_platform_by_layout_offset_sub_2728` (`platforms.rs:524`), `resolve_duplicate_platform_load_loc_286A` (`498`), `advance_platform_loader_offset_loc_286E` (`515`), `find_active_platform_miss_loc_2740` (`490`) are all dead.

**Field decode — type-0 height −2 quirk (loc_2746) omitted** (`loc_2762 size-decode / loc_2822`, `kid.asm:4744-4746`)
Fixed 2026-06-13 with the loader row above: decoded type zero now stores `(v+1)*16-3`, while non-zero types keep `(v+1)*16-1`.

**Field decode — art selection bypasses `PlatformArtAddr_Table`; render-flag vs type nibble conflated** (`loc_2822 PlatformArtAddr_Table`, `off_24AC dispatch nibble`)
The ROM uses two selections: `PlatformArtAddr_Table[type]` (4 entries) for the VRAM art word, and `off_24AC` for the tiling variant. The live renderer picks art by `match kind { 0x00 => horizontal, _ => vertical }` (`sprites.rs:1446-1449`) — a 2-way collapse ignoring the 4-entry table and per-flag strip variant, and never stores a `mapping_addr` (`types.rs:2095-2107`). The dead dispatch helper (`platforms.rs:1020`) also decodes with `render_flags & 0x0F`, the wrong mask for the shipped `platforms_newtype=0` build (`bclr #7`).

**`Platforms_CheckCollision` — invented `GROUND_SNAP_TOLERANCE_FP` replaces exact AABB overlap test** (`loc_25A0, loc_25EE`)
The ROM determines contact by exact hitbox-span overlap with strict edge tests, no tolerance (`kid.asm:4484-4503`). The live path gates landing/riding on `GROUND_SNAP_TOLERANCE_FP = 0x0006_0000` (6px, `constants.rs:475`) plus a 2px fallback (`platforms.rs:725-726`). This 6px window (a reused floor-collision invention) changes the geometry of when Kid is on/in a platform — he can snap onto platforms he is not overlapping.

**`Platforms_CheckCollision` — single-pass full-slot loop with epilogue decomposed into multiple recomputed passes** (`loc_257E, loc_2590, loc_2720`)
The ROM runs one routine looping every slot, accumulating contact flags across slots within a call, re-reading the (already mutated) hitbox each iteration (`kid.asm:4465-4642`). Native decomposes into `find_standing_platform_index_loc_8ED0` → per-platform `step_soft_platform` → `apply_platform_collision_with_ceiling_probe` → `find_standing_platform_index` **again** (`lib.rs:3056-3088`), and the collision pass `return`s on the first match (`platforms.rs:800,848`) rather than accumulating. This loses cross-slot flag accumulation and the post-push hitbox feedback between platforms. The faithful epilogue helper (`platforms.rs:82`) is dead.

**`sub_8ED0` — crawl-on-platform 5/7 half-width override is test-only (never wired)** (`sub_8ED0 crawling branch`, `loc_833E`)
Fixed 2026-06-13: the frame loop now preserves the native crawl-path decision and passes it into `find_standing_platform_index_loc_8ED0` and `apply_platform_collision_with_ceiling_probe`; the helper's `crawling=true` branch now runs live for platform support checks. Regression: `find_standing_platform_index_loc_8ed0_uses_crawling_width_override`.

**Walk/crawl platform crush — additive carry instead of ROM snap-to-(platform_y-1)+copy-y_vel; d7==2 ceiling-up path approximated** (`loc_8DA0/loc_8DF6`, `loc_8306/loc_837C`, `sub_902A d7==2`)
The ROM walk/crawl-on-platform paths snap `y_pos` to platform top −1 and copy the platform's `y_vel` before `sub_902A`; `d7==2` (ceiling hit while carried up) triggers Death (`kid.asm:10164-10195,9077-9090`). The native fused path adds platform y-velocity additively (`platforms.rs:780-783`) and models crush as `y_velocity<0 && ceiling-probe hit` gated on a block grid (`786-799`), collapsing the standing and walk/crawl crush variants into one heuristic — crush timing/position can differ at boundaries (acknowledged at `platforms.rs:765-767`).

**TrapPlatform — deactivation flag `$12(a3)` not modelled for special platforms** (`loc_2876 sf $12(a3)`, trap wait-loops, `loc_41EE`, `loc_4270`)
The ROM clears `$12(a3)` on creation and every trap coroutine polls it each frame to deallocate + delete when the platform scrolls out (`kid.asm:4800,5272,5306,5333,5360`). `ActivePlatform` carries no equivalent field and no self-delete mechanism; even the helpers that model the poll (`platforms.rs:1404,1480`) are dead. The lifecycle (`Deallocate_PlatformSlot + Delete_CurrentObject`) is entirely absent.

**Lift block — ceiling-crush while riding upward (loc_78A8) omitted** (`loc_78A8`, crush `loc_78C0`, `loc_8DF6`)
Fixed 2026-06-13: `step_lift_platforms` now returns `PlatformCollisionOutcome::CrushDeath` when an upward lift carry probes a ceiling hit, and the frame shell routes that outcome to the existing death windup. Regression: `lift_platform_loc_78a8_crushes_rider_into_ceiling`.

**Platform loader — offset/list helpers modelled but never wired into live roster build** (`sub_2728`, `loc_286E`)
The ROM drives loading through a stateful cursor (`PlatformLoader_Offset += $C` per call; `sub_2728` matches `$20(a2)` against the offset to detect already-loaded records). Faithful `advance_platform_loader_offset_loc_286E` (`platforms.rs:515`) and `find_active_platform_by_layout_offset_sub_2728` (`524`) are dead. The live roster build (`enemies.rs:40310-40340`) rebuilds every frame from a camera window + filter, so the incremental one-record cadence and offset-keyed dedup are not reproduced.

### Low severity

**Layout indexing — platform index sign-extension (`ext.l`) not modelled; unsigned add only (no effect on base ROM)** (`Get_PlatformLayoutAddress/sub_2366`, `MapPtfmLayout_Index`)
The ROM sign-extends the 16-bit index (`ext.l d7`) before adding `PlatformLayout_BaseAddress` (`kid.asm:4201-4202`). Native pre-resolves at asset-build time with an unsigned read (`split.py:346`, `kid-data/src/lib.rs:1119-1123`). Verified against `kid.bin`: every `MapPtfmLayout_Index` entry across all 126 levels is ≤ `0x4000`, so no high bit is set and unsigned == sign-extended for the shipping ROM. Latent only for ROM hacks.

**Layout indexing — `map_order` out-of-range fallback differs from ROM unconditional table read** (`Get_PlatformLayoutAddress/sub_2366`)
Fixed 2026-06-13 for live platform metadata: `NativeRuntime::platform_layout` now follows `Get_PlatformLayoutAddress/sub_2366` and refuses to fall back to raw logical-level platform metadata after the `MapOrder_Index` tail. The broader `resource_level_id()` fallback remains for non-platform fixture/resource paths that deliberately support empty map-order manifests. Regression: `platform_layout_loc_2366_does_not_fallback_to_raw_level_after_map_order_tail`.

**Platform loader — `PLATFORM_PRELOAD_X_PADDING` constant 64 is invented; ROM has no separate preload pass** (`Manage_PlatformLoading loc_2762`, `kid.asm:4684-4698`)
Fixed 2026-06-13: the constant is gone and `advance_to_window` stops exactly when `platform_buffer_edge_px_loc_2762(buffer_left) > KidX`, matching the `cmp.w d6,d7; bgt.w loc_2850` branch. Regressions: `advance_to_window_loc_2762_consumes_exact_left_buffer_edge` and `advance_to_window_loc_2762_stops_past_left_buffer_edge`.

**Scripted movement — `$FFFF` sentinel loop-back pointer discarded; native always rewinds to command 0 of the same script** (`loc_243E`, `loc_2422`, `Execute_ScriptedPlatforms/sub_23EA`)
Fixed 2026-06-13: `PlatformScript` now stores a `loop_target_offset`, `parse_platform_scripts` preserves both `$FFFF`/`dc.l PlatformScript_XXXX` targets and no-sentinel label fall-through (`PlatformScript_3F74 → 3F7E`), and `ActivePlatform::step_script` switches to the target script at the terminal command. Regressions: `parse_platform_scripts_extracts_commands` and `platform_step_script_loc_243e_follows_cross_script_loop_pointer`.

**Scripted movement — global freeze gate `$FFFFFA64` not modelled in live caller** (`Execute_ScriptedPlatforms/sub_23EA`, `kid.asm:4266-4267`)
The ROM short-circuits the entire scripted-platform pass when `$FFFFFA64 != 0` (a global freeze for some cutscene/boss/transition states). The live caller (`lib.rs:3050-3055`) steps all platforms unconditionally. The writer of `$FFFFFA64` could not be located in the available disasm, so the trigger conditions are uncertain (confidence lowered).

**Scripted movement — position update saturates to `[0,u32::MAX]` instead of ROM 32-bit wrap** (`Execute_ScriptedPlatforms/sub_23EA`, `kid.asm:4282-4285`)
Fixed 2026-06-13: `ActivePlatform::step_script` now uses 32-bit wrapping addition for x/y positions, matching `add.l d7,2(a2)` and `add.l d7,6(a2)`. Regression: `platform_step_script_loc_23f2_wraps_position_like_add_l`.

**`find_standing_platform_index_loc_8ED0` adds vertical-proximity logic not in `sub_8ED0` (re-derives standing platform each frame)** (`sub_8ED0`; `Addr_PlatformStandingOn`)
Fixed 2026-06-13: runtime now stores `active_platform_standing_on` as the native `Addr_PlatformStandingOn` equivalent. Platform collision reports `PlatformCollisionOutcome::StandingOn(index)` on landing/carry, the per-frame roster rebuild remaps that pointer by layout index, and `standing_platform_index_loc_8ED0` only validates the stored platform with the pure horizontal `sub_8ED0` support test. Regression: `standing_platform_index_loc_8ed0_uses_stored_platform_and_crawling_width_override`.

**SoftPlatform — launch velocity sampled post-physics instead of frame-start (one-tick timing nuance)** (`SoftPlatform_WaitLoop / loc_40E2`, `kid.asm:5206-5224`)
Fixed 2026-06-13: `trigger_standing_soft_platform_bounce_loc_40E2` now feeds `slice.pre_gravity_y_vel` into the SoftPlatform launch helper, matching the `move.l $2A(a2),d1` sample before `Hibernate_Object_1Frame`. Regression: `soft_platform_wait_loop_loc_40e2_samples_pre_hibernate_kid_y_velocity`.

**SoftPlatform — `soft_platform_settled_step_loc_4160` is a dead/test-only helper (logic inlined elsewhere)** (`loc_4160`, `kid.asm:5242-5248`)
Fixed 2026-06-13: `ActivePlatform::step_soft_platform_loc_40E2` now calls `soft_platform_settled_step_loc_4160` from the Settled phase, using the caller's active-platform index comparison as the native equivalent of `Addr_PlatformStandingOn == a3`.

**Lift platform — `sfx_Elevator_block` sound never played** (`sub_FCA6`, `kid.asm:21022-21024`)
Superseded by current code: `process_block_effects` now queues `QueuedSoundEffect::ElevatorBlock` before spawning the lift platform (`blocks.rs:3676-3680`), and `process_block_effects_lift_spawns_platform` asserts the sound effect with the ASM label (`tests/runtime_tests.rs:11048-11058`). Keep the broader lift-platform framework gaps above open; only this sound row is stale.

**Lift block — pre-spawn block bob animation (`sub_FB3E`) omitted; bob helpers dead** (`sub_FB3E`, `loc_FB94…loc_FBDA`)
Before spawning the rideable lift, the ROM bobs the block in several hibernating passes (`kid.asm:20905-20964`). Native models all bob passes (`blocks.rs:3150-3213`) but every one is `allow(dead_code)`, test-only; the live spawn (`blocks.rs:3322-3337`) jumps straight to creating the `LiftPlatform` with no bob pre-roll.

**Platform ride Y snap uses shared 6px `GROUND_SNAP_TOLERANCE_FP` rather than hitbox overlap** (`Platforms_CheckCollision loc_257E/loc_2590`, `kid.asm:4471-4478`)
The ROM seeds the scan from Kid's full hitbox and resolves landing by overlap, snapping to `platform_y-1`. Native gates riding on `y_pos.abs_diff(platform.y_pos) ≤ GROUND_SNAP_TOLERANCE_FP` (6px, `constants.rs:475`, shared with terrain). Faithful hitbox-seed helpers (`platforms.rs:543,560`) are dead. (Severity rises if audited under the collision unit.)
