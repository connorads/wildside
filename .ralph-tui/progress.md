# Ralph Progress Log

This file tracks progress across iterations. Agents update this file
after each iteration and it's included in prompts for context.

## Codebase Patterns (Study These First)

- **Player death lifecycle**: `Alive` → `DeathWindup` (159 frames) → `Dying` (80 frames, player hidden) → `Respawning` (401 frames) → `Alive`. `begin_death_windup()` starts the chain; `enter_death_hidden_state()` decrements lives and resets form/HP.
- **Respawn position**: `OpeningSliceState::respawn(bounds)` places player at `OPENING_ENTRY_KID_X_POS` / `OPENING_GROUNDED_KID_Y_POS` (level start).
- **Instant death vs damage**: `apply_damage()` respects hitpoints/form; for instant death (pit), set `number_hitpoints = 0` and call `begin_death_windup()` directly.
- **Physics flow in `OpeningSliceState::step()`**: horizontal motion → vertical motion → wall collision → ceiling collision → floor collision → camera follow. Damage/death checks happen in `refresh_opening_slice()` AFTER `step()`.
- **Flat floor fallback**: Without a `block_grid`, `apply_floor_collision` uses a fixed `terrain.floor_y_pos`. Tests needing pit/gap behaviour must set a block grid with empty cells.
- **Fixed-point 16.16**: All positions are `u32` with upper 16 bits = pixels. `>> 16` to get pixel value, `<< 16` to convert pixels to FP.
- **Opening animation**: First 260 frames (`OPENING_SCROLL_START_FRAME`) after level start run entry animation, not player-controlled physics. Tests that need physics must advance past this.
- **Flag collision / level completion**: `Alive` → `LevelComplete` (120 frames, input frozen) → `advance_level()` → `Respawning`. `player_touches_flag()` uses a 16x16 hit region around the flag position from `LevelHeader`. The `level_complete` bool on `NativeTraceFrame` signals renderers to display "LEVEL COMPLETE".
- **Enemy contact damage**: AABB overlap between player hitbox (half_width=5, height=31 above feet) and enemy hitbox (16x16 centred on position). `should_apply_contact_damage()` checks all levels. Invulnerability (300 frames) prevents rapid re-hits. `apply_damage()` handles HP decrement/death.
- **Boot resets HP**: `refresh_opening_slice` resets `number_hitpoints` every frame when not in `StartingLevel`/`InGame` boot state. Tests that set HP must do so AFTER `start_level()` + opening animation advance, not before.

---

## 2026-03-09 - US-001
- Implemented pit death trigger: player dies instantly when y position exceeds level height in pixels
- Added `fell_below_level()` check in `refresh_opening_slice()` after physics step
- Respawn at level start position already existed via `OpeningSliceState::respawn()`
- Files changed: `crates/kid-core/src/lib.rs`
- **Learnings:**
  - `apply_floor_collision` flat-floor fallback snaps player back to floor surface, so pit death only works with a block grid that has gaps
  - Tests that manipulate player position must account for `slice.step()` re-running physics — need a block grid with no floor, not just position injection
  - `start_level()` helper advances ~370 frames; additional ~260 frames needed before physics are active
---

## 2026-03-09 - US-002
- Implemented spike death trigger: player dies instantly when contacting spike collision cells
- Added `on_spikes` field to `PlayerState`, set during `apply_floor_collision` when floor probe returns a `CollisionKind::Spikes` cell
- Added `Spikes` to `is_solid_floor()` in terrain so spikes act as landable surfaces
- Spike death check in `refresh_opening_slice()` after pit death check — sets HP to 0 and calls `begin_death_windup()`
- Two tests: full death/respawn lifecycle and HP-ignoring instant kill
- Files changed: `crates/kid-core/src/lib.rs`, `crates/kid-core/src/terrain.rs`
- **Learnings:**
  - Spikes must be in `is_solid_floor()` so `probe_floor` detects them as ground — otherwise the player falls through
  - Tests with hazard floors at spawn point conflict with `start_level()` which runs ~430 frames of gameplay — need solid floor at spawn and hazard offset to the side
  - `on_spikes` flag on `PlayerState` follows the same pattern as `on_ground` — set during collision, checked during the refresh step
---

## 2026-03-09 - US-004
- Implemented enemy-player contact damage with proper AABB hitbox overlap
- Generalised `should_apply_level_one_contact_damage()` → `should_apply_contact_damage()` — removed Level 0 restriction so contact damage works on all levels
- Replaced Manhattan distance check with true AABB overlap: player hitbox (half_width=5, height=31 above feet) vs enemy hitbox (16x16 centred on position)
- Removed `LEVEL_ONE_ENEMY_CONTACT_X_FP`/`LEVEL_ONE_ENEMY_CONTACT_Y_FP` constants, added `ENEMY_HALF_SIZE` constant (8px)
- Invulnerability frames (300) already prevented rapid re-hits — no change needed
- `apply_damage()` already handled HP decrement and death trigger — no change needed
- Enemies remain alive after contact (no kill logic exists)
- Four tests: contact deals damage, invulnerability prevents rapid re-hit, 1 HP contact triggers death, enemy survives after damaging player
- Files changed: `crates/kid-core/src/lib.rs`
- **Learnings:**
  - `refresh_opening_slice` resets HP/form/lives every frame when boot state is not `StartingLevel`/`InGame`. Tests that inject HP must do so AFTER `start_level()` + opening animation frames, not before
  - The existing level-1 contact damage used a wide 50px×6px centre-distance threshold; proper AABB with player/enemy hitboxes is more accurate and general
  - AABB math uses `i64` to avoid `u32` underflow when subtracting fixed-point offsets from positions near zero
---

## 2026-03-09 - US-003
- Implemented flag collision and level completion trigger
- Added `LevelComplete` variant to `PlayerLifecycle` with 120-frame pause
- Added `player_touches_flag()` — compares player position to `LevelHeader.flag_x/flag_y` using 16x16 hit region (8px half-size in fixed-point)
- Flag check runs in `refresh_opening_slice()` after spike/pit death checks, before damage checks
- On flag touch: lifecycle transitions to `LevelComplete`, freezing input; after countdown, `advance_level()` loads the next level
- Added `level_complete` bool to `NativeTraceFrame` for renderer display
- Three tests: flag touch triggers completion, input frozen then level advances, flag miss does not trigger
- Files changed: `crates/kid-core/src/lib.rs`, `crates/kidcli/src/native_player.rs`
- **Learnings:**
  - Entry animation (260 frames after level start) overrides player position — tests that inject player position must advance past it first
  - Floor grid row must align with the flag's y position for floor collision to keep the player at the right height (flag_y=768px → floor at row 48)
  - `PlayerLifecycle` additions need updates in: the lifecycle match in `refresh_opening_slice`, the roster refresh match, and `NativeTraceFrame` construction
---

