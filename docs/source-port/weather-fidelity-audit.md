<!-- Generated 2026-05-31 by the `platforms-helmets-weather-fidelity-audit` workflow (1 read-only Explore scout enumerated the weather/environment units; one read-only Explore auditor per unit compared the live native path to kid.asm; every high/medium gap adversarially verified by an independent refuter — defaulting to refuted — before inclusion; low gaps included without a refute pass). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native `file:line` searched. Re-run the workflow to refresh. -->
<!-- Staleness-checked 2026-06-10: of 33 gaps, 10 fixed, 7 partially fixed, 16 still open — verdicts and evidence in fidelity-audit-staleness-2026-06-10.md. Still-open entries below remain authoritative; fixed entries are superseded. 2026-06-15 follow-up: live lava geyser rise/hold/collapse now uses the ASM row cadence, and level entry now spawns/ticks Island shore plus lava/water rotator object state; script-walk, palette initialisation, art/DMA upload, and per-row renderer gaps remain below. -->

# Weather / Environment Fidelity Audit

Adversarially-verified audit of the Kid Chameleon source-port's **weather / environment-effects** subsystem (`weather.rs`, with `level.rs` palette-rotate objects, `lib.rs` per-frame step, `boot.rs`), comparing the live path to the `kid.asm` `Init_SpecialEffects` / `sub_6034` / `sub_61CA` / `Obj_Lava_Geyser` / `Murderwall` routines. It goes deeper than the level-loading audit's "only weather 1 is handled" finding, into the per-frame correctness of what *is* wired and the storm script-machine fidelity.

The recurring failure mode is the same one the other audits found crate-wide: **ASM-faithful pure helpers exist and are unit-tested, but carry `#[cfg_attr(not(test), allow(dead_code))]` (or `#[allow(dead_code)]`) and are never called from the live path.** Weather is one of the most complete examples: the entire storm script-machine, the 5-slot hail worker, and the lava-geyser state machine are modelled helper-for-helper and tested, yet the runtime runs hand-rolled approximations instead.

Findings worth foregrounding:

- **`Init_SpecialEffects` dispatches only weather 1 (Lava).** The ROM indexes 5 init routines by `Level_Special_Effects-1` (1=Lava, 2=Storm, 3=Storm+Hail, 4=Unknown, 5=Nothing, `kid.asm:27461-27476`); native `init_weather` (`weather.rs:2414`) runs only `if weather_type == 1`. Storm/Storm+Hail get no art decompress, no palette load, no VRAM pattern upload, no state-machine seed; weather 4 gets nothing. Confirms and extends the level-loading audit.

- **The per-frame storm machine (`sub_6034 → sub_6048`) is dead.** It drives the 20s storm palette fade, the 25s Mountain thunder SFX/flash, and the 35s snow/hail gates (`kid.asm:6032-6184`). The faithful `step_level_special_effects_sub_6048` (`weather.rs:1061`) and dispatcher `dispatch_level_special_effects_sub_6034` (`174`) are test-only; the live `step_weather` (`weather.rs:2354`) hand-rolls **only** hail for weather 3 and **drops weather 2 entirely** (`weather.rs:2356`), so plain-Storm levels get no palette swap and no thunder.

- **Storm hail is a single generic projectile, not the 5-slot worker.** The faithful `step_weather_worker_sub_61CA` (`weather.rs:726`) — with the collision-map spawn gate, ice/lava splash slots, and the Kid `$28` push-timer — is dead. The live path pushes one `EnemyProjectile::Hail` with an invented PRNG (`frame_index*7+13 % 320`), wrong cadence (fixed 30 vs random 22–37), and wrong inactive period (600 vs ROM 1200).

- **The lava-geyser eruption no longer uses the old 4px pixel ramp.** Fixed candidate 2026-06-15: live `step_lava_geysers` now maps encoded height to 16px sprite rows, uses `$48=2` rise cadence, `$50=$1E` counted through 6-frame top cycles, and one-row collapse from `Obj_Lava_Geyser` (`kid.asm:27908-28035`). Remaining lava-geyser gaps are the script-walk intro/disable sentinels, palette animator, art upload, and renderer window details listed below.

- **Palette animation is only partially live.** Fixed candidate 2026-06-15: level entry now spawns and ticks Island shore `loc_1202A` plus `RotatePalette_Lava_Water`, and the lava/water worker mutates the runtime `Palette_Buffer+$E..+$14` on the ROM cadence. Remaining gaps: the level-load theme palette words feeding that buffer are still not populated live, `Animation_Geyser` remains unspawned/unstepped, and Island shore still lacks the one-shot shore-art DMA/renderer consumption. This compounds the level-loading audit's palette-object row.

Murderwall is the best-wired unit: its init and per-frame motion are faithful and live; only the 16.16 initial-X fraction and a graphics palette copy diverge.

## Coverage summary

| Unit | Coverage | Confirmed gaps |
|---|---|---|
| Special-effect dispatch & per-weather init (`sub_6034` table + `Init_SpecialEffects`) | partial | 3 |
| Storm script machine — per-frame palette/thunder/snow (`sub_6048`) | stub | 5 |
| Storm rain/lightning/hail spawn+step worker (`sub_61CA`) | stub | 4 |
| Lava geyser spawn & position tables (`Init_SpecialEffect_Lava` + `LavaGeyserPositions`) | partial | 3 |
| Lava geyser eruption cycle & collision (`Obj_Lava_Geyser`) | partial | 5 |
| Lava small-fireball variant (`loc_1499C` / `sub_14AB0`) | partial | 1 |
| Lava/water RotatePalette + geyser palette animation (`loc_1223C` / `Animation_Geyser`) | stub | 3 |
| Island shore animated-tile + palette cycle (`loc_1202A` / `loc_12098..12188`) | stub | 3 |
| `Init_SpecialEffect_Unknown` (weather 4) plane fill | stub | 0 |
| Storm art/palette upload init (`Init_SpecialEffect_Storm`) | stub | 4 |
| Murderwall init & per-frame motion (`Murderwall` / `loc_7316` / `loc_736E`) | partial | 1 |
| Murderwall collision & warning-sprite render (`sub_5E02`) | partial | 1 |

Totals: 12 units audited, 0 full, 6 partial, 6 stub. **33 confirmed gaps** (15 high, 10 medium, 8 low). `Init_SpecialEffect_Unknown` (weather 4) is recorded `stub`/0 gaps — it has no live implementation, but its high/medium claims did not survive adversarial refutation.

## Confirmed gaps

### High severity

**`init_weather` — `Init_SpecialEffects` only dispatches weather 1 (Lava); Storm/Storm+Hail/Unknown init omitted** (`Init_SpecialEffects / Init_SpecialEffect_Index`, `Init_SpecialEffect_Storm`, `Init_SpecialEffect_Unknown`)
The ROM indexes `Init_SpecialEffect_Index` by `(Level_Special_Effects-1)` into 5 routines (`kid.asm:27461-27476`). Native `init_weather` (`weather.rs:2414`) replicates only the Lava branch (`if weather_type == 1`). Weather 2/3 skip `Init_SpecialEffect_Storm` entirely — no rain/hail art decompress to VRAM `$7000`, no storm palette to `Palette_Buffer+$20`, no tile remap, no saved-palette pointer to `$FFFFFADE`, and none of the storm state seeds (`$FFFFFB3C=1`, `$FB3D=5`, `$FAD6=4`, `$FADA=$100`, `$FADC=1`, `$FB52=1`, `$FB54=0`). Weather 4 skips `Init_SpecialEffect_Unknown`. Storm-weather levels never initialise their visual/state setup. Faithful unwired helpers (`select_storm_art_loc_14CCE` `weather.rs:198`, `upload_storm_art_loc_14CD8` `218`, `select_saved_storm_palette_loc_14D82` `243`) are `allow(dead_code)`, test-only.

**`step_weather` — `sub_6034` per-frame worker-table dispatch never wired; live step omits storm palette fade / thunder / Mountain flash** (`sub_6034`, `dword_6028`, `sub_6048`)
The ROM main loop calls `sub_6034` every gameplay frame (`kid.asm:917`); it indexes `dword_6028 = [0, sub_6048, sub_6048]` by `(Level_Special_Effects-1)`, so weather 2 and 3 both run `sub_6048` (the 20s storm palette darkening, the Mountain thunderstorm SFX/flash, and the snow/ice-ball cadence). Native has no analogous dispatcher; the live `step_weather` (`weather.rs:2354`) handles only `weather_type==3` and only spawns hail after 35s — no palette fade, no per-frame thunder, nothing for weather 2. The faithful `step_level_special_effects_sub_6048` (`weather.rs:1061`), `dispatch_level_special_effects_sub_6034` (`174`), and `step_weather_worker_sub_61CA` (`726`) are all `allow(dead_code)`, test-only.

**`step_weather` — entire `sub_6048` storm-palette/thunder/snow machine unwired (faithful helper dead, test-only)** (`sub_6048`, `loc_60EE`, `loc_613E`, `loc_6172`)
`sub_6048` (`kid.asm:6050-6184`) performs the 20s storm-theme palette swap (Mountain vs non-Mountain branches, fade `-4`), the 25s Mountain thunderstorm (`sfx_Thunderstorm` + `Pal_7B86C`/`Pal_7B85C` flashes, timer reseed `$F7-(rand&$F5)`), and the 35s/36s snow/ice-ball gates. The live `step_weather` (`weather.rs:2354`) hand-rolls only the hail toggle and does none of the palette/SFX/fade work. The fully faithful `step_level_special_effects_sub_6048` (`weather.rs:1061`, `allow(dead_code)`) is called only from tests; its outputs (`palette_ops`, `play_thunder_sound`, `call_snow_worker`) are consumed by no live code.

**`step_weather` — drops weather type 2 (`WeatherID_Storm`) entirely; gates only on `weather_type==3`** (`sub_6034`, `dword_6028`)
`dword_6028` maps both weather 2 and 3 to `sub_6048` (`kid.asm:6025-6027`), so the storm-palette swap and Mountain thunderstorm run for weather 2 as well; only the snow/hail block (`loc_6172`) is exclusive to weather 3. Native `step_weather` returns immediately unless `weather_type==3` (`weather.rs:2356`), so weather-2 levels get no storm palette swap and no thunderstorm at all.

**Storm hail — faithful 5-slot worker `step_weather_worker_sub_61CA` is dead/test-only; live path is a single generic projectile** (`sub_61CA, loc_61E2, loc_626C, loc_6318, loc_63C4, loc_6462`)
The ROM `sub_61CA` (`kid.asm:6190`) runs a 5-slot falling-weather engine at `$FFFFFAE2`: each slot advances `+4px Y / -1px X`, runs an animation countdown, spawns one fresh slot per cooldown (`$25 - rand&$F`) at a random X above the camera with a collision-map gate, sets the Kid `$28` push timer on hitbox overlap, splashes into 10 impact slots on terrain contact, and emits splash + falling sprites. Native ports all of this in the dead `step_weather_worker_sub_61CA` (`weather.rs:726`, `allow(dead_code)`, test-only). The live `step_weather` (`weather.rs:2354`) pushes one `EnemyProjectile::Hail` every 30 frames with a frame-index pseudo-random X and a 600-frame toggle — wrong particle count, cadence, jitter, and randomness source.

**Storm hail — terrain collision-map (`$FFFF4A04`) spawn-gate and ice/lava splash emission entirely omitted** (`loc_626C, loc_6318, loc_638E`)
The ROM gates new-slot spawns against the level collision map at `$FFFF4A04` and tests each falling slot's collision word (`& $7000`: 0 keep falling, `$4000`/`$5000` slope splash with `y+$C`, other non-zero splash directly), allocating an impact slot with anim word `$600` on contact (`kid.asm:6275-6379`). The live `step_weather` never consults the collision map; the Hail projectile only deletes off-camera (`projectiles.rs:1915-1926`) — no terrain contact, no ice/lava splash, no impact animation, no splash sprites (`$A3CD/$A3D1`). The collision-map gate and splash live only in the dead worker (`weather.rs:893-994`).

**Storm hail — Kid hitbox `$28` push timer (`move.w #$28,$38(a5)`) not applied by live path** (`loc_62D6`)
The ROM tests each active falling slot against Kid's hitbox and on overlap writes `move.w #$28,$38(a5)` (Kid's push/stun timer) (`kid.asm:6298`). The faithful model is in the dead worker (`weather.rs:858-879`, `kid_collision_timer = Some(0x0028)`). The live path runs no such scan; Hail is a generic projectile relying on generic projectile-vs-Kid collision, which does not write `$28` to `$38`.

**Lava geyser — rise/hold/collapse row driver wired; script/render residue remains** (`Obj_Lava_Geyser loc_14758, loc_1484E, loc_1496C, loc_1498C`)
Fixed candidate 2026-06-15: live `step_lava_geysers` now uses the ROM's discrete sprite-row state machine for the visible column: rise emits one 16px row every `$48=2` frames, full-height hold counts `$50=$1E` through 6-frame top cycles, collapse removes one row per tick, and collapse re-arms the per-geyser startup hibernate. `initialise_lava_geyser_loc_14C6A` now converts `height >> 4` rows into pixel height for rendering/collision instead of treating the row count as pixels. Covered by `lava_geyser_erupts_and_collapses`, `step_lava_geysers_phase_3_collapse_decrements_one_sprite_row`, and init regressions. Remaining: the first-run/steady script walk (`$FFFF`/`$FFFE`), alternating top-row grow/shrink rendering, and per-row camera-window output are still separate gaps.

**Lava geyser — timing constants replaced with ASM row/cycle values** (`loc_147A0 ($50=$1E)`, `loc_1484E ($48=2)`, `loc_1496C ($48=6)`, `loc_14772 ($4E)`)
Fixed candidate 2026-06-15: `LAVA_GEYSER_ERUPT_SPEED=4` and raw `HOLD=30` no longer drive the live path. Native now names the ROM timing domains (`LAVA_GEYSER_ROW_HEIGHT_PX=16`, `LAVA_GEYSER_RISE_FRAMES_PER_ROW=2`, `LAVA_GEYSER_HOLD_CYCLES=30`, `LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE=6`) and collapse uses the helper for `loc_1498C`. The `$3C` disable branch remains part of the still-open script-walk gap, not a live normal-loop dormancy.

**Partial fixed candidate 2026-06-15 — RotatePalette_Lava_Water object is spawned/ticked and mutates core Palette_Buffer; live palette initialisation/render consumption still incomplete** (`RotatePalette_Lava_Water / loc_1223A / loc_1223C`, spawned `loc_11F86/loc_11F96`)
The ROM spawns an object at load (Theme 7 / Cave) that hibernates `d2` frames (5 Mountain / 8 otherwise) then rotates four `Palette_Buffer` words at `$E,$10,$12,$14` downward forever (`kid.asm:24148-24383`). Native `begin_level_entry` now spawns a live rotator state for `loc_11F86`/`loc_11F96`, computes the ASM delay via `loc_1223A`, and ticks the `loc_1223C` countdown/reload cadence in the frame loop, including the four-word runtime `Palette_Buffer+$E..+$14` rotation. Remaining gap: live level entry still does not seed those words from the ROM theme palette assets and the renderer does not yet consume the core buffer for lava/water colours.

**Animation_Geyser — geyser palette cycle (`$28..$2E`) omitted from live path (helper unwired)** (`Animation_Geyser / loc_14B2A`, `kid.asm:28206-28219`)
The ROM's geyser animator hibernates 5 frames then rotates four `Palette_Buffer` words at `$28,$2A,$2C,$2E` upward forever. The faithful `rotate_lava_geyser_palette_loc_14B2A` (`weather.rs:1554`) reproduces this but is `allow(dead_code)`, test-only. The live `step_lava_geysers` (`weather.rs:2455`) animates only eruption height and never writes `Palette_Buffer`.

**Partial fixed candidate 2026-06-15 — post-load object spawn (`loc_11F86/loc_11F96/loc_11FB0`) now allocates core object state** (`loc_11F86 / loc_11F96 / loc_11FB0`, `kid.asm:24146-24168`)
The ROM's `Load_InGame` tail allocates the palette-cycle worker objects (Theme 7 / Cave → `RotatePalette_Lava_Water`; Island → `loc_1202A`). Native `begin_level_entry` now calls the classifier and creates either the lava/water rotator or Island shore-cycle state, matching the foreground-priority branch before the Island fallback. Remaining gaps live below the spawn layer: lava/water needs live palette seeding and renderer consumption, Island needs the one-shot DMA/renderer data path, and `Animation_Geyser` is still separate.

**Partial fixed candidate 2026-06-15 — Island shore `loc_1202A` object routine is spawned/stepped, but DMA/rendering remains missing** (`loc_1202A, loc_12098, loc_12110, loc_12188`)
The ROM allocates a slot running `loc_1202A` when `Background_theme==Island`: a one-shot DMA of the `$2E0`-byte shore art to VRAM `$F600`, then a forever loop streaming the next shore frame for three VRAM bands on independent countdowns (`kid.asm:24217-24355`). Native now spawns that state from live level entry when the foreground lava/cave branch does not win, and the frame loop ticks the three band countdowns/phases. Remaining gap: the state is traced but not consumed by a VRAM/renderer path, so shore-art upload and visual tile streaming are still absent.

**Storm init — entire `Init_SpecialEffect_Storm` omitted from live runtime; faithful helpers all dead/test-only** (`Init_SpecialEffect_Storm / loc_14CCE…loc_14DAC`, `kid.asm:28335-28448`)
On entering a storm level the ROM decompresses rain/hail art (by `Background_theme==Ice`) to VRAM `$7000`, copies the rain/hail palette into `Palette_Buffer+$20`, uploads the `$380`-tinted pattern-block table to VRAM `$6000`, saves the storm palette pointer to `$FFFFFADE`, decompresses shared weather art, and clears the 5 falling + 5 impact slots. The live `init_weather` (`weather.rs:2406-2417`) does none of this — no weather-2/3 branch at all; `WeatherState` (`types.rs:638-651`) has no storm-machine state. Faithful helpers (`weather.rs:198,218,259,274,301,356,243,413,458`) are all `allow(dead_code)`, test-only.

**Storm init — `FB3C/FB3D/FAD6/FADA/FADC/FB52/FB54` seed words never written at runtime** (`loc_14DAC tail`, `kid.asm:28439-28447`)
The ROM tail seeds the storm machine state that `sub_6048`/`sub_61CA` consume (spawn cooldown, free-falling slot count, weather-mode byte, fade countdown, thunder timer, FB52/FB54 control words). The faithful `initialise_storm_slots_loc_14DAC` (`weather.rs:458-485`) models all of them into `StormSlotInitLoc14DAC`, but it is dead (test-only). `WeatherState` has none of these fields, and the per-frame consumers (`weather.rs:1061,726`) are also dead, so even if seeded the live loop would not consume them.

### Medium severity

**`step_weather` — invented hail cadence/spawn physics partially diverges from `sub_6048`/`sub_61CA` worker** (`sub_6048 loc_6172`, `sub_61CA`)
The live `step_weather` (`weather.rs:2354-2404`) reimplements hail with its own cadence: ROM gates on `Frame_Counter` (`$834`=35s, `$873`=36s) and toggles via `$FFFFFB52` reload of `$258`; native gates on `level_timer.seconds>145` (`weather.rs:2360`) with a hand-rolled 600-frame toggle (`2369`), and spawn X uses an invented `(frame_index*7+13)%320` (`2386`) vs the ROM `Get_RandomNumber`. The faithful `sub_61CA` model (`weather.rs:726`) is dead. Hail does at least visibly appear for weather 3.

**`step_weather` — wrong inactive hail period (600 vs ROM `0x4B0`=1200 frames)** (`loc_619C`, `loc_6172`)
The ROM seeds period `0x4B0` (1200 frames) on inactive→active and `0x258` (600) on active→inactive (`kid.asm:6161-6164`). Native reseeds `hail_period_timer` to a flat 600 for both transitions (`weather.rs:2369`), so the storm-off gap between bursts is half the ROM duration. The dead helper (`weather.rs:1205`) has the correct `if hail_active { 0x0258 } else { 0x04B0 }`.

**`step_weather` — timing uses `level_timer.seconds` approximation instead of `Frame_Counter` gates (`0x834`/`0x873`)** (`loc_6172`, `kid.asm:6151,6155`)
The ROM gates the snow/hail block on `Frame_Counter` thresholds `$834` (2100 frames = 35.0s) and the stricter `$873` (2163 = 36.05s). Native gates on `level_timer.seconds > 145` (`weather.rs:2360`) — a coarse 1-second check that collapses the two thresholds into one, ignores the 36s gate, and is keyed off the countdown timer (so it diverges with time bonuses) without sub-second precision.

**Storm hail — invented spawn cadence, period toggle, and PRNG with no ASM basis** (`sub_61CA, loc_623A`)
The live `step_weather` invents a 600-frame `hail_active` toggle (`weather.rs:2364-2370`), a fixed 30-frame spawn timer (`2377-2381`), and a deterministic `frame_index*7+13 % 320` X (`2386`). The ROM `sub_61CA` uses `Get_RandomNumber_byte` for cooldown jitter (`$25 - rand&$F`) and spawn X (`Camera_X + rand_byte + $28`), with no active/inactive period concept inside `sub_61CA`. The real cadence lives only in the dead worker (`weather.rs:780-782`).

**Lava geyser — USM2 `$8000` small-fireball table entry dropped; fireball hardcoded instead of `$8000`-flag decode** (`Init_SpecialEffect_Lava / loc_14C6A / Obj_Lava_Geyser loc_1499C`)
`LavaGeyserPositions_USM2` has three entries; the third (`$346/$312/$8000`) carries the sign-bit flag that makes its worker the small-fireball variant via `bmi loc_1499C` (`kid.asm:28226,27802`). Native's `LAVA_GEYSER_POSITIONS_USM2_LOC_14C5E` (`weather.rs:2161`) contains only the first two entries; the fireball is synthesised out-of-band with hardcoded literals (`x:0x346, base_y:0x312`) and a native `LAVA_FIREBALL_SCRIPT` (`weather.rs:2442-2451`). The spawn count matches (3), but the table no longer encodes the third entry and the `$8000`-flag/height decode is not applied. Faithful `initialise_lava_fireball_state_loc_1499C` (`weather.rs:1588`) is dead.

**Lava geyser — palette upload + `Animation_Geyser` palette-rotation animator omitted from live path** (`Init_SpecialEffect_Lava / Animation_Geyser loc_14B2A`)
`Init_SpecialEffect_Lava` copies 4 longwords from `Pal_7B8AC` into `Palette_Buffer+$20` and allocates `Animation_Geyser`, which rotates palette words `$28/$2A/$2C/$2E` forever (`kid.asm:28290-28333`). Native `init_lava_geysers` (`weather.rs:2421`) seeds no geyser palette and steps no animator (`lib.rs:3160-3161`). The faithful `rotate_lava_geyser_palette_loc_14B2A` (`weather.rs:1554`) is dead, and there is no `Palette_Buffer` seed for it to operate on.

**Lava geyser — script-walk intro animation + `$FFFF`/`$FFFE` sentinels omitted entirely** (`loc_147BA, loc_147D6, loc_1480C, loc_1483A, loc_14788/unk_146B6, loc_1479C/unk_146A0`)
The ROM walks a script pointer (steady `unk_146A0` / first-run `unk_146B6`) before the rise machine, emitting intro template rows with per-row camera culling; the first eruption terminates in `$FFFE → loc_1483A` (plays `sfx_Lava_Geyser`, hibernates), steady eruptions in `$FFFF → loc_1484E` (`kid.asm:27713-27744,27858`). Native has no script walk, no intro animation, no `$FFFE`-vs-`$FFFF` distinction, and no per-cycle sound gating. Faithful helpers (`weather.rs:1861,1890,1975,1751,1742`) are test-only.

**Island shore — one-shot shore tile-art DMA upload at `loc_1202A` is not modelled by any helper** (`loc_1202A`, `kid.asm:24217-24244`)
Before the animation loop the ROM DMAs the `$2E0`-byte shore art from `unk_9784A+2` to VRAM `$F600` (Z80 stop/start), then seeds the band countdowns/phases. Native wrote only the three steady-state band steppers; there is **no** helper for the one-shot upload or the initial seeding, so even if the steppers were wired the first frame of shore art would never upload. The spawn struct (`level.rs:8112`) carries only `object_slot_seed`/`object_routine_addr`.

**Storm init — `$FFFFFADE` saved storm palette pointer never set** (`loc_14D82 / loc_14D8A`, `kid.asm:28419-28423`)
The ROM writes `Pal_7B85C` into `$FFFFFADE` for both rain and storm+hail (used later for restore/cross-fade). `select_saved_storm_palette_loc_14D82` (`weather.rs:243`) correctly resolves both weather 2 and 3 to `RainPalette7B85C` but is dead (test-only). The live runtime stores no saved storm palette, so the fade machinery (`FADA` etc.) has no source palette.

**Storm init — `$380`-tinted pattern-block VRAM upload (`loc_14D32`) absent from runtime** (`loc_14D1E / loc_14D32 / loc_14D40`, `kid.asm:28364-28409`)
The ROM uploads the storm pattern-name block table (rain `unk_9B65C` vs hail `unk_9B6E0`) to plane address `$6000`, adding the `$0380_0380` tile-base offset to every longword (4 outer × 8 group × 8 repeat). Faithful `upload_storm_pattern_blocks_loc_14D32` (`weather.rs:356`), `select_storm_pattern_upload_loc_14D1E` (`301`), `storm_pattern_emit_group_loc_14D40` (`344`) are dead. The renderer never sets up the storm name-table plane; storm visuals rely on the ad-hoc hail-projectile approximation.

### Low severity

**`step_weather` — invented hail spawn cadence and spawn-X (fixed 30 / frame-index hash vs ROM random 22..37 / camera+rand)** (`loc_623A`, `loc_626C`)
The ROM reseeds spawn cadence `$FFFFFB3C` to `$25 - rand&$F` (22–37 frames) and computes spawn X as `Camera_X + (rand&$FF) + $28` (`kid.asm:6235-6256`). Native uses a fixed `hail_spawn_timer=30` (`weather.rs:2381`) and `(frame_index*7+13)%320` (`2386`). The faithful `sub_61CA` model (`weather.rs:726`) is dead. Cosmetic, but the values have no ROM basis.

**Lava geyser — lava art `DecompressToVRAM` upload omitted** (`Init_SpecialEffect_Lava`, `kid.asm:28319-28322`)
`Init_SpecialEffect_Lava` decompresses `ArtComp_983D2_Lava` to VRAM `$5F60`. The native `init_lava_geysers` (`weather.rs:2421`) performs no art/VRAM upload, so geyser sprite tiles are never loaded by this path. Rendering-asset upload — may be architecturally out of scope, but entirely absent.

**Lava geyser — top/middle/bottom sprite-row culling windows (`loc_14874/loc_148F8`) reconstructed in renderer, faithful core helpers dead** (`loc_14874, loc_148CE, loc_148F8, loc_14914, sub_14BD8/loc_14BE8`)
The ROM emits sprite rows through three templates with per-row camera windows. The chunk builder `sub_14BD8` IS faithfully ported and wired (`lava_geyser_sprite_chunks_loc_14BE8` `weather.rs:1394`, used at `scene.rs:2057`), but the renderer rebuilds the top/middle/bottom layout from a smooth height (`scene.rs:2068-2099`) rather than the worker's discrete rows, and the window helpers (`weather.rs:2016,2090,2055,2067,2122`) are dead. Cosmetic, but the `$54(a5)` value those windows compute also feeds collision (links to the medium-severity hitbox gap).

**Lava geyser — inter-cycle restart uses flat `DORMANT` instead of re-seeded per-geyser startup hibernate** (`loc_1498C, loc_14788, loc_14772, word_14710`)
On collapse the ROM re-runs the `loc_14772` startup hibernate (`d2 = word_14710[height&$F]`) before the next eruption — a per-geyser delay (`kid.asm:28031-28035,27807`). Native wires `lava_geyser_initial_delay_frames_loc_14772` into the phase-0 timer once (`weather.rs:1599`) but resets to the flat `LAVA_GEYSER_DORMANT_FRAMES=60` (`2499`) after collapse, losing the staggered phasing (the point of the height-indexed `$4E` table in Black Pit's 13-geyser cascade) after the first cycle.

**Lava fireball — faithful script-decode/init helpers exist but are dead; live path uses a parallel static table** (`loc_1499C / loc_149CC / loc_149E4`)
The ROM decodes the `$8000`-flagged USM2 fireball (clear sign bit, `word_14710` startup delay, `height>>4`, `$80` bias) and runs the `unk_146CC` triplet script (`kid.asm:28038-28060`). Native provides faithful ports (`weather.rs:1588,1617,1662`) all `allow(dead_code)`; the live runtime hardcodes the spawn (`weather.rs:2443-2450`) and a static `LAVA_FIREBALL_SCRIPT` (`1244-1300`). The static table is faithful to `unk_146CC` and the spawn constants match the third USM2 entry — hygiene/duplication, but the duplicated path carries the off-by-one advance noted under the eruption gaps.

**Island shore — `off_1220E` frame-source link table not modelled (phase index only)** (`off_1220E`, `unk_12226`, `kid.asm:24350-24361`)
Each band uploads `off_1220E[phase]` (one of six frame-art pointers) plus a band offset. The step helpers return `uploaded_frame_index` and a DMA offset/length but do not resolve the six `off_1220E` source addresses, so the actual frame art per phase is unrepresented. `unk_12226` is correctly captured (`COUNTDOWN_TABLE_LOC_121A6` matches the six ROM bytes). Modelling-completeness gap, moot while the subsystem is unwired.

**Murderwall — init writes full 16.16 X but ROM stores only the integer high word** (`loc_140D2`, `loc_140BE`)
ASM init stores the initial X with `move.w d0,(MurderWall_X_pos)` — only the integer-pixel high word; the fraction retains prior RAM contents. Native sets a full i64 `(camera<<16)+(0x30<<16)` (`weather.rs:2541-2546`), zeroing the fraction. Practically negligible (speed cleared, first step adds whole pixels), but a strict divergence. `clr.l MurderWall_speed` is matched.

**Murderwall — palette copy (`loc_140EE`) helper is dead — never wired into runtime palette load** (`Murderwall loc_140EE`, `kid.asm:26915`)
The ROM copies 8 palette words from `Pal_1408A` into `Palette_Buffer+$20` each murder-wall setup. The faithful `copy_murder_wall_palette_loc_140EE` (`weather.rs:8-16`) is `allow(dead_code)` with no live caller; the warning strip is rendered from precomputed `level_visuals.murder_wall` tiles/palette, so the ROM palette-buffer copy is never reproduced.
