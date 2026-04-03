# Research Map

This map records what the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

**Progressive fidelity**: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. Native-playability evidence is the primary programme-health signal; the oracle (`jgenesis`) is the primary parity oracle and safety net.

## Current frontier

The live TAS frontier is currently closed:

- `kc_any_percent_2024` is authoritative-clean through frame `4916`
- `any-percent-published` is authoritative-clean through frame `37759`
- `frontier-state.yaml` is the compact source of truth for reopening TAS work

That is not the same thing as whole-game completion. The active programme now shifts
from "close the next TAS window" to "make 1P actually work in native play while
continuing to burn down fidelity debt". `cargo run -p kidcli -- autonomous-loop --goal source-port-frontier` remains the operator entrypoint: while TAS work exists it packages the live frontier, and once TAS is closed it auto-delegates to the next repo-local lane. The default post-TAS lane is `case-queue`, which ranks reduced semantic case bundles, promotes uncovered recordings into case-derivation work, and only falls back to `sweep-parity`/verification hardening when no gameplay-producing lane remains. `kidcli project-status` remains the top-level status surface for reading those signals together.
For `rl`/Codex runs, use `mise run source-port-rl-run` rather than shell-history prompts. That launcher regenerates the live delegated package, records a cache-local run ledger, and pipes the generated prompt straight to `cxys`, so Codex consumes the real task package instead of re-reading the checked-in operator note. If tracked git changes already exist, the launcher keeps the run alive by turning that diff into the first recovery task instead of pretending the tree is clean. Use `mise run source-port-rl-prompt` only when you need to inspect the freshly generated package without launching Codex.

Current post-TAS verification split as of 2026-04-03:

- `cargo run --release -p kidcli -- sweep --levels 0-104 --tier state` passed all 69 BCD levels again; the fresh report is `.cache/sweep-runs/1775225897/report.json`, and the matching `.cache/metrics/sweep-history.jsonl` entry still records `advisory=320` and `oracle_limitation=648` with zero blocking diffs. The CLI still reports levels carrying both categories as `NON-BLOCKING`, each `field_summary` entry still carries its own `divergence_summary`, the concrete BCD `level_ids` where that field appeared, the unique checkpoint labels where that field diverged, and a `checkpoint_counts` breakdown showing how many diffs landed at each checkpoint. Both the per-run report and the append-only history entry now persist `release_gate_focus`, so the current gate/sweep correlation survives in the artefact that automation reads rather than only in stdout or the standalone report.
- `mise run release-gate` still fails the `1p-accurate` lane at `boot_title_start_level1`: `title@360` diverges in `game_mode` (`native 0x000C`, `oracle 0x002C`), and the fresh gate report is `.cache/release-gate/1p-accurate/1775226019/report.json`. Release-gate hardening is still a boot/title verification problem rather than a sweep-parity problem.
- `cargo run -p kidcli -- autonomous-loop --goal sweep-parity` still delegates this lane to release-gate hardening rather than gameplay work; after the fresh sweep it should continue correlating the failing release-gate field against the latest sweep `field_summary`, with the release-gate scenario/checkpoint attached as `boot_title_start_level1 @ title@360`. Treat that line, and the persisted `release_gate_focus` report entry, as field-only correlation rather than proof that the gate failure is already explained by sweep debt. The current `game_mode` histogram is still non-blocking in sweep (`oracle_limitations=15` across 15 levels, including `0x00`), and `release_gate_focus` now also records whether the matching histogram explicitly includes the gate scenario's own level (`scenario_level_id=0x00`, `scenario_level_matched_in_state_sweep=true`) plus the checkpoint distribution actually seen in sweep (`matched_checkpoints=[idle_tail=14, level_settle=1]` in the latest run). That is stronger evidence for the verification story, but it still points to a boot/title verification problem rather than a new gameplay frontier. If the same delegated sweep fingerprint stalls repeatedly, the loop should switch into recovery mode and narrow the next run to evidence refresh instead of replaying the same broad sweep-parity prompt.
- The fresh `release_gate_focus` evidence is now more specific: the latest state sweep still includes level `0x00` in the non-blocking `game_mode` histogram, but it does not include the release-gate checkpoint label `title@360` (`checkpoint_matched_in_state_sweep=false`); instead the histogram only saw `idle_tail` and `level_settle`. That keeps the delegated lane honest: this is shared field-name evidence, not proof that the boot/title gate failure is covered by idle state sweep parity.

## Canonical status language

Use the evidence ladder below when describing progress:

- `stubbed`
- `boots`
- `replay-matched`
- `branch-matched`
- `mechanic-swept`
- `human-play-stable`

Current TAS milestones prove `replay-matched` on their covered routes. They do not, by
themselves, prove `human-play-stable`.

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

### Gameplay fidelity (TAS recordings verified, native-play gaps still open)
- **#037** [x] Floor-snap settling diffs — done. The final `0x24` / `0x51` 1px `camera_y` tail now uses the oracle-backed hidden init camera carry, while `0x29` and `0x52` stay locked down by their focused regression tests.
- **#062** [~] HeadyMetal death wobble visual — long-lived ASM-timed death effect now renders boss wobble from the ROM's staged `sub_37708` quiver magnitudes, and native-player tests lock the body/eye sprite offsets at each timer boundary; full oracle frame verification still remains.
- **TAS recordings** [x] `kc_any_percent_2024` and `any-percent-published` are both authoritative-clean through their recorded endpoints (`4916` and `37759` frames respectively). Only reopen a TAS frontier when a fresh `verify-tas --bisect` run proves a new authoritative mismatch.
- **Native playability** [ ] recurring player-facing failures still need canonical repros and regression coverage for the remaining live gaps rather than the already-closed death/respawn, slope, bounce-block, and post-bounce Cloud issues.

### Game flow verification (not yet investigated)
- **2-player alternating mode**: `PlayerSlot` framework exists but not verified against oracle. Need to verify: P1 dies → P2 takes over with independent level/lives/form, game over when both players exhausted, continue screen in 2P context. Write DynamicScenario probes for 2P.
- **Password system**: the original game has a password system for level progression. Not yet investigated whether this is implemented or needed. Trace ASM for password generation/validation. Check whether `BootState` has a password entry screen.

### Enemy behaviour gaps (10 items — see `phase4-enemy-findings.md` for full details)
High impact: ~~Archer arrow lifetime + re-aiming~~ (FIXED), Tornado Cyclone-flying interaction, Crab proximity freeze, Dragon Flying wall-reversal.
Medium impact: Sphere/Fireball slope deflection, Driller block drilling, probe half-width per-family.
Low impact: speed caps at high HP, Cloud lightning bolt attack, Dragon Flying fire spawn X offset.

### Oracle sweep hardening
- **Advisory field promotion**: `number_objects`, `number_hitpoints`, `number_lives`, `murder_wall_x` still masked in sweep. Unmask one at a time, run sweep, fix divergences. `number_objects` may require architectural changes if native object activation/deactivation timing differs from oracle.
- **Framewise checksumming**: methodology.md describes CRC32 per-frame game state checksumming for pinpointing divergences (player + enemies + camera + blocks). Not yet implemented. Add `--checksum` flag to sweep. First mismatch = exact divergence frame.
- **Divergence classification output**: sweep history now persists per-run `divergence_summary` totals, a cross-level `field_summary` histogram with per-field category counts, BCD `level_ids`, unique checkpoint labels, per-checkpoint diff counts, and `release_gate_focus`, plus per-level category/field summaries in `.cache/metrics/sweep-history.jsonl`; the sweep report mirrors the same `field_summary` and `release_gate_focus`, while the flatter `.cache/metrics/sweep-telemetry.jsonl` stream remains available for append-only metrics ingestion. `kidcli autonomous-loop --goal sweep-parity` now consumes that data to correlate a failing release-gate field against the latest sweep classes automatically instead of leaving that comparison as manual report inspection, and `kidcli sweep` now prints the same focused field summary inline. That summary includes the failing release-gate scenario/checkpoint plus whether the matching histogram includes that scenario's own BCD level and checkpoint label, and which checkpoints actually carried the field, so operators can distinguish "same field name" from "same field name on the same covered level/checkpoint".
- **Full-frame pixel comparison sweep**: run frame capture on all 105 levels at key frames (0, 30, 60, 120, 300), compare pixel-for-pixel against oracle. Extend `kidcli` with `sweep --tier frames`. Each pixel difference classified as rendering, palette, timing, sprite priority, or position error.

### Testing infrastructure
- **Semantic case bundles**: `kidcli case derive|branch|verify|reduce|summarise` now provide the phase-1 discovery lane. Use them to turn replay slices or native recordings into reduced semantic failure bundles before another broad gameplay patch.
- **Autonomous case queue**: `kidcli -- autonomous-loop --goal case-queue` now reads reduced case bundles from `.cache/cases`, ranks them deterministically, surfaces untriaged recordings from `.cache/recordings`, and emits one case-centred handoff note for the next agent.
- **Native replay recording**: `play-native --record <slug>` and `verify-recording --slug <slug>` already exist for input + trace capture. Bundles now include ROM/build provenance, explicit replay commands, legacy `trace.json` schema backfill, copied-bundle verification via `--bundle <path>`, bundled `final-frame.png` + `audio.wav`, optional `notes.txt` via `--record-notes <path>`, and an automatic `transcript.txt` sidecar distilled from the native trace so copied bundles stay reviewable before replay.
- **Mutation testing baseline**: `cargo mutants` exists but no recorded baseline. Run `cargo mutants -p kid-core -- --lib`, target >=80% kill rate on `physics.rs` and `terrain.rs`. Record in `.cache/metrics/mutation-score.txt`.
- **Fuzz testing for parsers**: set up `cargo fuzz` targets for level block grid parser (untrusted binary format), enemy spawn logic (layout data with unexpected IDs), audio bank loading (complex binary format). Run each for a few hours, fix crashes.
- **Stateful property testing**: implement `proptest-state-machine` test for `PlayerLifecycle`. Model lifecycle as states with input-driven transitions. Generate random input sequences, assert invariants hold.
- **1P playable gate**: add a separate native-playability smoke gate covering Level 1 completion, Level 2 entry, death/respawn, slopes, bounce surfaces, and common block interactions. The current `1p-accurate` gate remains valuable but too narrow to stand in for "game works".

### Known deviations (5 DEVIATION annotations, intentional)
- Tornado, Fireball (×2), Robot: RNG-dependent behaviours. ASM uses deterministic global RNG, native uses independent RNG per enemy. Behaviour is statistically equivalent but not frame-identical.
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
- Shooter mechanism (#065) — resolved: visible shooter cells now register independently, fire on entry, and recycle after the ROM's 22-frame slot lifetime
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
