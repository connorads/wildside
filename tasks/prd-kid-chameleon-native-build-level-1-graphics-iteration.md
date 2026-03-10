# Kid Chameleon Native Build: Level 1 Graphics Iteration

## Overview
Build the next iteration of the native Kid Chameleon port with a primary focus on getting Level 1 rendered natively using ROM-extracted assets and modern native rendering infrastructure. The target outcome is a playable build where the full Level 1 is traversable and visually recognisable as the original opening level, while keeping architecture clean enough to support later gameplay and presentation work.

This iteration should bias toward faithful original layout and asset usage, but allow a modernised rendering pipeline under the hood. Asset extraction must happen via tooling, with ROM-derived payloads kept out of source control.

## Problem
The project already has some gameplay progress, but it does not yet deliver a convincing, playable native version of Level 1 that looks like Kid Chameleon. The next useful milestone is not more isolated logic work; it is a native-rendered build that makes the first level feel real and testable as a game space.

## Goal
Ship an internal playable prototype where:
- the whole of Level 1 is traversable start to exit
- native rendering shows the level using ROM-extracted visual assets
- the result visibly resembles the original first level
- developers can validate progress with reasonable oracle/native comparison workflows

## Non-Goals
- Full audio support
- Exact end-to-end reproduction of all Level 1 behaviours
- Full enemy and hazard parity
- Shipping ROM-derived raw assets in git
- Broad multi-level support unless needed as a direct enabler

## Product Scope

### Must ship
- Entire Level 1 traversable from start to exit
- Native rendering for level tiles/backgrounds
- Native rendering for the player
- Asset extraction/build tooling that pulls needed graphics from the ROM
- Tracked generated metadata/manifests where useful, but not raw extracted assets
- Oracle/native comparison support for a few named Level 1 scenarios/checkpoints
- Playable build that feels recognisably like the original Level 1

### Stretch
- Core platforms, collisions, pickups, and major hazards are solid enough to support a more authentic playthrough
- Rendering for enemies, pickups, and HUD
- Additional presentation features from the original look, where cheap and low-risk

### Deferred
- Audio, including minimal SFX
- Deeper gameplay edge cases
- Full presentation polish beyond what is needed to make Level 1 readable and playable

## User
Primary user is the developer working on the native port, using this milestone to:
- see real Kid Chameleon graphics running natively
- play through Level 1 in a meaningful way
- compare native output against the original with less manual guesswork
- create a base for later gameplay completion and presentation work

## Requirements

### Functional requirements
1. The game can load and traverse the full Level 1 in the native build.
2. The renderer can display Level 1 tiles/backgrounds using assets extracted from the ROM.
3. The renderer can display the player using ROM-extracted assets.
4. The build/tooling pipeline can extract required assets on demand from the ROM.
5. Extracted raw assets remain untracked in source control.
6. Generated metadata/manifests may be tracked if they improve determinism, debugging, or build ergonomics.
7. The game supports enough movement/camera/game-state integration to complete a start-to-exit traversal.
8. Oracle/native comparison tooling supports a few explicit Level 1 scenarios to help self-check development progress.

### Quality requirements
1. Rendering architecture should support later expansion to enemies, pickups, HUD, and more presentation features without major rewrites.
2. Asset tooling should be repeatable and developer-friendly.
3. The build should clearly separate tracked tooling/metadata from untracked ROM-derived output.
4. Comparison workflows should be “reasonable”: useful during development without becoming a heavy gate on every change.

## Experience principles
- Faithful where visible: preserve the original level’s recognisable composition and feel.
- Modern underneath: use native-friendly rendering and asset pipeline choices if they improve maintainability.
- Playable over exhaustive: prioritise a coherent playthrough over perfect subsystem parity.
- Tooling is part of the product: extraction and oracle checks are required to make iteration practical.

## Acceptance Criteria

### Release bar
- A developer can build or run tooling to extract the required Level 1 visual assets from the ROM.
- The native build displays Level 1 backgrounds/tiles and the player using extracted assets.
- A developer can traverse from Level 1 start to exit in the native build.
- The build visually resembles the original first level strongly enough to function as a real internal milestone, not a placeholder tech demo.
- ROM-derived raw assets are not committed to source control.
- A small set of oracle/native comparison scenarios exists and is usable during development.

### Scenario examples
- Boot/title/start-level flow reaches Level 1 correctly
- Level 1 opening area renders in the right structure/layout
- Mid-level checkpoint or landmark area renders and scrolls correctly
- End-of-level traversal completes without blocking issues

## Suggested milestones

### Milestone 1: Asset and rendering foundation
- ROM extraction flow for required Level 1 graphics
- Asset manifests/metadata shape finalised
- Native renderer can draw static level visuals

### Milestone 2: Playable visual slice
- Player rendered in-world
- Camera and traversal integrated
- Opening section feels recognisable as Kid Chameleon

### Milestone 3: Full Level 1 traversal
- Entire level traversable start to exit
- Rough edges acceptable where explicitly deferred
- Oracle scenarios added for key checkpoints

### Milestone 4: Stretch pass
- Improve gameplay fidelity for core platforms/collisions/pickups/hazards
- Add enemies, pickups, HUD rendering if scope allows

## Risks
- Rendering parity may expand into a large hidden scope if “full presentation” is treated as mandatory rather than aspirational.
- Level-wide traversal may expose gameplay gaps that drag the iteration away from the graphics milestone.
- Asset extraction and representation choices may create rework if not designed for later systems.
- Oracle coverage can become too shallow to be useful or too broad to be maintainable.

## Open decisions for implementation
- Exact set of tracked metadata/manifests
- Which Level 1 checkpoints become named oracle scenarios
- Whether HUD/enemy rendering is worth taking in this iteration once traversal is stable
- How much visual polish to include before it stops paying back

## Success measure
This iteration is successful if the project moves from “some native gameplay work exists” to “the full first level is visibly there, playable end-to-end, and grounded by a practical extraction and comparison workflow”.