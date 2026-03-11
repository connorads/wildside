---
status: completed
created: 2026-03-10
started: 2026-03-10
completed: 2026-03-11
---
# Task: Cover remaining visible Level 1 families and harden failures

## Description
Finish the scoped Level 1 slice by converting the remaining visible families to ROM-backed commands and locking in the failure and verification paths. This is the completion gate for removing default-path placeholders from the validated traversal.

## Background
After static layers and the player are asset-backed, the remaining work is visible enemies, moving platforms, collectibles, and the runtime/reporting hardening that proves the default path no longer silently falls back. The plan intentionally combines these into one final slice because the strict completion rule is behavioural, not crate-local.

## Reference Documentation
**Required:**
- Design: .agents/scratchpad/implementation/kc-level1-graphics/design.md

**Additional References:**
- .agents/scratchpad/implementation/kc-level1-graphics/context.md (codebase patterns)
- .agents/scratchpad/implementation/kc-level1-graphics/plan.md (overall strategy)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. The remaining visible Level 1 enemy families, moving platforms, and collectibles must render from ROM-backed commands on the default path.
2. Startup must fail clearly when a required Level 1 visual asset is missing instead of silently drawing placeholders.
3. Gameplay comparison must retain semantic passes while writing frame evidence or a review artefact for manual inspection.
4. Verification and missing-asset handling must stay at the runtime/CLI boundary, not in low-level raster code.

## Dependencies
- Task 06: Replace player placeholders with ROM-art sprites

## Implementation Approach
1. Write failing tests for remaining-family scene emission, missing-asset startup failure, and gameplay evidence output.
2. Implement the smallest atlas, scene-builder, and CLI/runtime changes needed to satisfy the completion gate.
3. Refactor shared helpers only after the full visible-family and failure-path tests are green.

## Acceptance Criteria

1. **Remaining Visible Families Use Asset-Backed Commands**
   - Given representative Level 1 scenes that include visible enemies, moving platforms, and collectibles
   - When the native scene builder emits commands
   - Then those visible families render through indexed-tile or sprite commands rather than rectangles or text placeholders

2. **Missing Required Assets Fail Explicitly**
   - Given one required decoded Level 1 asset is removed from the generated bundle
   - When the default native path starts
   - Then startup fails with a contextual error instead of silently using placeholders

3. **Gameplay Comparison Writes Review Evidence**
   - Given the pause/resume Level 1 comparison scenario
   - When `compare-native-gameplay` runs
   - Then semantic comparison still passes and frame evidence or an equivalent review report is written for inspection

4. **Unit Tests Pass**
   - Given the implementation is complete
   - When running the task's focused tests
   - Then `build_game_scene_emits_enemy_sprite_commands_for_visible_level1_families`, `build_game_scene_emits_platform_and_collectible_art_commands`, the final form of `native_scene_builder_uses_asset_backed_commands_for_visible_level1_families`, `play_native_fails_fast_when_required_level1_visuals_are_missing`, and `compare_native_gameplay_retains_semantic_passes_while_writing_frame_evidence` all pass

## Metadata
- **Complexity**: High
- **Labels**: level1-graphics, enemies, platforms, verification, failure-handling, tdd
- **Required Skills**: Rust, sprite mapping, CLI integration, verification harnesses
