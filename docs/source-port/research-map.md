# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. The oracle (`jgenesis`) is the safety net and primary testing tool.

## Current frontier

All 10 original milestones are done. The project is now in **playtest-driven iteration** — see `backlog.md` for the canonical bug/feature tracker and `CLAUDE.md` for the iteration protocol.

## Done

### Boot and title
- Done: typed boot/title state machine (`BootState` enum)
- Done: mode flow, title timing, oracle-backed title readiness and intro-skip handling
- Done: TAS-backed early boot slice parity across 3 imported recordings (first 250 frames, 0 authoritative diffs)
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

### Visual fidelity — sprite extraction (3 TODO(fidelity) markers)
- **#033** [x] Transform animation visual sprites — original `Kid_Transform` art and timing are now wired for all forms. Juggernaut now composes the split head/base sprites into native player frames.
- Fire Demon death trail visual sprites — rendered as generic projectile, needs real fire fragment tiles from ROM.
- Floating item ROM sprites — some items use content-type-specific pixel-art icons, others placeholder rectangles (scene.rs). Needs ROM VRAM tile extraction.

### Visual fidelity — rendering accuracy (not yet investigated)
- **Palette cycling per theme**: some themes animate palette entries (water shimmer done). Lava, neon, and other themes may have CRAM cycling in the VBlank handler. Trace `kid.asm` VBlank for CRAM writes per theme ID. Extract cycle tables (which entries, rotation speed, colour sequence).
- **Sprite priority/layering**: VDP sprite priority bit determines front/behind-BG rendering. Some sprites intentionally render behind background (hiding behind pillars). Search ASM for priority bit usage, compare frame captures on levels with sprite-background overlap.
- **HUD positioning accuracy**: compare HUD element pixel positions against oracle — lives icon, HP bar segments, timer digits, score digits, diamond count, diamond icon. Capture frame, overlay oracle, diff.
- **Background scroll accuracy per theme**: 11 themes each have background scroll configurations (parallax rates, layer ordering). Verify `background_scroll_x_px()` rates are correct per theme by comparing background-only layer at several camera positions against oracle.
- **Block destruction particle effects**: verify particle spawn positions, velocities, colours, and lifetime match ASM. Break blocks in multiple themes, capture debris frames, compare to oracle.
- **Enemy death animations**: verify enemy death visual effects (explosion, fade, particle) match ASM per family. Stomp enemies, capture death frame, compare to oracle.
- **Invulnerability flicker accuracy**: verify flicker pattern (which frames visible/invisible) matches ASM. Verify Diamond Power overlay colour. Take damage, capture flicker frames, compare to oracle frame sequence.

### Gameplay fidelity (2 open backlog items + active TAS frontier)
- **#037** [~] Floor-snap settling diffs on 4 levels — residuals are now small but real. Current focused sweeps show: 0x52 (`camera_y` +2px, `kid_y` +2.25px), 0x29 (`camera_x`/`kid_x` +5px), and 0x24/0x51 (`camera_y` +1px). `0x02` is now clean apart from advisory lives. Root cause remains the init-time spawn-support shortcut versus the original frame-step settling.
- **#062** [~] HeadyMetal death wobble visual — long-lived ASM-timed death effect now renders boss wobble; still needs oracle-backed visual verification.
- **TAS frontier** [~] `kc_any_percent_2024` frame 3400 — the old frame-3126 lower-dragon dead-end is now closed: that turnaround was not a missing patrol-to-alert write, it was a spike shoulder on the shared grounded walker path. DragonWalking now treats spike tiles as blocking side walls there, and the 3124..3140 verify window is CRC-clean with 0 authoritative diffs. The live blocker is back where the wider roster traces said it was: the later frame-3399/3400 walking-dragon warm-state and stomp setup still drift even though the nearby frame-3384 hold tile and frame-3399 zero-velocity probes are green.

### Game flow verification (not yet investigated)
- **2-player alternating mode**: `PlayerSlot` framework exists but not verified against oracle. Need to verify: P1 dies → P2 takes over with independent level/lives/form, game over when both players exhausted, continue screen in 2P context. Write DynamicScenario probes for 2P.
- **Password system**: the original game has a password system for level progression. Not yet investigated whether this is implemented or needed. Trace ASM for password generation/validation. Check whether `BootState` has a password entry screen.

### Enemy behaviour gaps (14 items — see `phase4-enemy-findings.md` for full details)
High impact: Archer arrow lifetime + re-aiming, Dragon Flying fire velocity, Fireball gravity off-by-one, Tornado acceleration scaling.
Medium impact: Crab proximity freeze, Cloud post-bounce idle removal, Dragon Flying wall-reversal, Tornado Cyclone interaction, Sphere/Fireball slope deflection.
Low impact: Driller block drilling, speed caps at high HP, probe half-width per-family, Archer dx==0 aiming, Cloud lightning bolt attack.

### Oracle sweep hardening (not yet implemented)
- **Advisory field promotion**: `number_objects`, `number_hitpoints`, `number_lives`, `murder_wall_x` still masked in sweep. Unmask one at a time, run sweep, fix divergences. `number_objects` may require architectural changes if native object activation/deactivation timing differs from oracle.
- **Framewise checksumming**: methodology.md describes CRC32 per-frame game state checksumming for pinpointing divergences (player + enemies + camera + blocks). Not yet implemented. Add `--checksum` flag to sweep. First mismatch = exact divergence frame.
- **Divergence classification output**: methodology.md describes a taxonomy (authoritative/advisory/accumulation/RNG/oracle limitation) and JSON tracking format. Sweep currently doesn't output structured divergence categories. Add per-level category logging to `.cache/metrics/sweep-history.jsonl`.
- **Full-frame pixel comparison sweep**: run frame capture on all 105 levels at key frames (0, 30, 60, 120, 300), compare pixel-for-pixel against oracle. Extend `kidcli` with `sweep --tier frames`. Each pixel difference classified as rendering, palette, timing, sprite priority, or position error.

### Testing infrastructure (not yet implemented)
- **Input replay recording**: add `--record` flag to `play-native` capturing per-frame `FrameInput`. Write `DynamicScenario` JSON output — live play sessions become regression tests. Chocolate Doom-style demo regression.
- **Mutation testing baseline**: `cargo mutants` exists but no recorded baseline. Run `cargo mutants -p kid-core -- --lib`, target >=80% kill rate on `physics.rs` and `terrain.rs`. Record in `.cache/metrics/mutation-score.txt`.
- **Fuzz testing for parsers**: set up `cargo fuzz` targets for level block grid parser (untrusted binary format), enemy spawn logic (layout data with unexpected IDs), audio bank loading (complex binary format). Run each for a few hours, fix crashes.
- **Stateful property testing**: implement `proptest-state-machine` test for `PlayerLifecycle`. Model lifecycle as states with input-driven transitions. Generate random input sequences, assert invariants hold.

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
- Per-helmet transform timing + transform sprites — done (#033, correct frame counts and original art tables)
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
