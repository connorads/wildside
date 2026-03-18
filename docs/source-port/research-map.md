# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (`jgenesis`) is the safety net and primary testing tool.

## Current frontier

All 10 original milestones are done. The project is now in **playtest-driven iteration** — see `backlog.md` for the canonical bug/feature tracker and `CLAUDE.md` for the iteration protocol.

## Done

### Boot and title
- Done: typed boot/title state machine (`BootState` enum)
- Done: mode flow, title timing, 6-press start handling
- Source: `kid.asm`, oracle checkpoints

### Level load
- Done: header parsing, asset pointers, `LevelBundle` loading from extracted assets
- Done: typed asset manifest with level headers, enemy layouts, platform layouts
- Source: `doc/formats.txt`, `level/header/00.bin`, generated asset manifest

### Player physics and camera
- Done: real input-driven physics from disassembly speed tables, jump formula, gravity, camera follow
- Done: `physics.rs` module with pure functions for horizontal velocity, jump, gravity, camera
- Done: speed tables for 3 terrain types x 3 form variants
- Source: `kid.asm` speed tables (`unk_9C0A`), `sub_A432` (jump), `kid.asm:14060` (gravity), `sub_71E4` (camera)

### Gameplay lifecycle
- Done: pause/resume, death windup/hiding/respawn settle
- Done: damage invulnerability (300 frames)
- Source: oracle gameplay checkpoints

### Level 1 visuals
- Done: manifest-backed Level 1 visual loading and asset-backed native scene output
- Done: ROM-backed HUD, player, background, terrain, visible platforms, and visible Level 1 families on the default render path
- Source: generated bundle contract, `kid-render`, `kidcli::native_player`

### Object activation and tracing
- Done: enemy roster built from camera window, platform preload + activation window
- Done: native trace surface now records stable enemy layout identity and motion/state fields
- Source: `doc/ram_layout.txt`, enemy layouts

## Traversal closure

### Default runtime terrain authority
- Done: `NativeRuntime::new` now reloads generated `Level_Layout` terrain on construction
- Done: the default runtime path no longer depends on a separate explicit generated-terrain mode for Level 1 gameplay
- Source: `kid-core`, generated Level 1 bundle, `doc/formats.txt`

### Traversal parity
- Done: `compare-native-gameplay --scenario progress-two-transitions` passes on the default path
- Done: pause/resume and death/respawn stay green on the same default path
- Done: the long traversal script now takes the first opening Dragon hit on the oracle frame (`749`) under walk-only input as well as the existing run-held regression
- Done: `diagnose-native-gameplay-drift --scenario progress-two-transitions --from-frame 719` now reports no live drift after the skip choreography and Level 2 transition-arc fixes
- Source: `.cache/native-gameplay-oracle-runs/progress_two_transitions/`, gameplay comparison harness

### Enemy fidelity after traversal
- `Diamond` remains stationary and collectible
- `MiniHoppingSkull` uses a hopper loop in the native runtime
- Done: Pacer archetype for 6 Tier 1 families (Goat, Armadillo, Tank, Crab, Scorpion, Driller)
- Done: family-specific HP initialisation from ASM init routines
- Done: Cloud (horizontal flight + wall reversal), Sphere (gravity bounce), BigHoppingSkull (parameterised hopper)
- Done: Fireball (gravity bounce with level-dependent gravity), EmoRock/Archer (Inert stubs)
- Done: All 28 enemy families wired — Tier 3-4 mapped to closest archetypes (Pacer/Flyer/Hopper/Bouncer/Inert)
- Source: `code/enemy/*.asm`, oracle per-object traces

## Remaining work

### Visual fidelity (3 TODO(fidelity) markers)
- **#033** [~] Transform animation visual sprites — needs ROM tile extraction for spinning animation frames. `Kid_Transform` (kid.asm:8369-8607) has per-helmet sprite+palette cycling tables. Frame counts/durations already extracted; blocked on art pipeline for transform tile sheets.
- Fire Demon death trail visual sprites — rendered as generic projectile, needs real fire fragment tiles from ROM.
- Floating item ROM sprites — some items use content-type-specific pixel-art icons, others placeholder rectangles (scene.rs). Needs ROM VRAM tile extraction.

### Gameplay fidelity (2 open items)
- **#037** [~] Floor-snap settling diffs on 5 levels — root cause documented: `probe_floor_hitbox` only checks ±half_width edges, misses terrain beyond hitbox width. ASM runs full `Hibernate_Object_1Frame`. Fix requires running real frame steps during init or widening probe. Affected: 0x52 (+14px Y), 0x29 (+5px X), 0x02/0x24/0x51 (1px cam_y).
- **#062** [ ] HeadyMetal death wobble visual — purely cosmetic rendering effect during boss death sequence.

### Known deviations (6 DEVIATION annotations, intentional)
- Tornado, Fireball (×2), Robot: RNG-dependent behaviours. ASM uses deterministic global RNG, native uses independent RNG per enemy. Behaviour is statistically equivalent but not frame-identical.
- Shooter timer: ASM registers per-block on visibility, native uses global timer.
- Mushroom bounce: ASM bounces via SoftPlatform objects, native bounces player inline.
- Shifting block: ASM animates sprite with shared lift coroutine, native swaps grid instantly.

## Recently completed

- Scorpion tracking callback with $38000 speed escalation (HP≥5, within 170px) — Phase 4C
- Dragon Walking dual breath interval ($12C initial, $1F4 reset) — Phase 4C
- MiniHoppingSkull HP-scaled hop velocity (+$4000 X, -$10000 Y at HP>1) — Phase 4C
- Workspace clippy clean for Rust 1.94 (all warnings resolved or allowed)
- Boss multi-head system (#060) — 3 heads per boss with ASM position offsets + per-head HP
- BigHoppingSkull continuous bouncer rework (#056) — ASM $FA0 gravity, wall/floor/ceiling/slope deflection
- Orca X-accel, slope re-lunge, wall-pin (#061) — all 3 from ASM
- Lift block ASM coroutine timing (#067) — 2px/frame×8 up, erase, 1px/frame×16 down
- Shooter mechanism (#065) — documented as DEVIATION (ASM per-block registration vs native global timer)
- Mushroom bounce (#066) — documented as DEVIATION (ASM SoftPlatform objects vs native inline bounce)
- Shifting block (#068) — documented as DEVIATION (visual sprite animation only, grid swap correct)

## Previously deferred (now done)

- Platform collision — done (static + moving, riding logic, movement scripts)
- Player forms — done (speed tables, form-specific abilities, per-form Diamond Power)
- Audio and GEMS — done (GemsAudioEngine: Z80+YM2612+PSG from ROM)
- Tier 3-4 enemies + bosses — done (all 28 families, 26 with real sprite art)
- Y look-ahead camera offset — done (`$FFFFFB58`, +1/frame down+grounded, -8/frame decay, applied at >=30px)
- Block interactions — done at engine level (prize blocks, evanescent, ghost, smash, terrain types)
- Multi-level progression — done (105 levels loadable, full progression, ending sequence, continue, score system)
- Menus — done (boot/title, pause/resume, game over, continue, score board)
- Diamond Power two-tier cost — done (20 form/tier combos mapped to 8 unique power types)
- Demo playback — done (#026, extracted from kid.asm:33373-33436, 2 demos alternate on title idle)
- Per-helmet transform timing — done (#033 partial, correct frame counts from ASM tables)
- Perf baseline tracking — done (#031, criterion benchmarks: ~92ns/frame idle, ~3µs level load)
- Per-enemy constant verification — done for all 28 families against ASM (Phase 4)
- Block constant verification — done for all 13 block types against ASM (Phase 3)
- Ghost block collision fix — type 6 (Solid) not 7 (Spikes) (#063)
- Drill block direction fix — horizontal ±4px/frame not downward (#064)
- Murder wall constants — ASM-verified (acceleration, max speeds, offset)

## Validation methodology

See `docs/source-port/methodology.md` for the full validation methodology: investigation playbook, testing taxonomy (property-based, metamorphic, mutation, fuzzing), divergence classification, framewise checksumming, fidelity metrics, and regression discipline. `CLAUDE.md` has the operational summary.

## Known oracle limitations

- 9 oracle warp-broken levels (game_mode=0x4E73): 0x09, 0x32, 0x33, 0x50, 0x57, 0x5A, 0x5D, 0x5E, 0x63 — jgenesis emulator bug, native works fine (6 in BCD sweep)
- 4 levels with idle_tail advisory diffs (0x00, 0x45, 0x48, 0x61) — oracle warp timing and enemy interaction accumulation
