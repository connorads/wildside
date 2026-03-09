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

