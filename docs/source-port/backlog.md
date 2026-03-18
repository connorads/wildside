# Playtest Backlog

Canonical tracker for bugs and missing features. See `CLAUDE.md` for iteration protocol.

Status: `[ ]` open, `[~]` in progress, `[x]` done
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 001 | [ ] | P0 | player | Player gets stuck after first damage | Take hit, confirm movement resumes |
| 002 | [ ] | P0 | enemy | Enemies don't die when stomped | Stomp enemy on level 0x00, confirm death |
| 003 | [ ] | P1 | audio | No music or sound effects playing | Load level, confirm BGM and SFX |
| 004 | [ ] | P1 | render | Colour palette wrong (floor/tree colours reversed) | Capture frame, compare to oracle |
| 005 | [ ] | P1 | render | No parallax background | Scroll right, confirm background layers move |
| 006 | [ ] | P1 | enemy | Dragon moonwalking (sprite direction vs movement) | Watch dragon, confirm facing matches walk dir |
| 007 | [ ] | P1 | camera | Screen shakes/moves weirdly when hit | Take damage, confirm camera stays stable |
| 008 | [ ] | P1 | render | Unwanted dark overlay effect when hit | Take damage, confirm no full-screen tint |
| 009 | [ ] | P2 | menu | No menu — spawns directly into game | Launch, confirm title screen appears |
| 010 | [ ] | P2 | enemy | No enemy attack patterns (fire breath, arrows) | Reach archer/dragon, confirm projectiles |
| 011 | [ ] | P2 | block | Non-P bricks look half-broken (wrong sprite state) | Capture frame, compare brick tiles to oracle |
| 012 | [ ] | P2 | block | No particle effects when blocks break | Hit block, confirm debris particles |
| 013 | [ ] | P2 | block | P blocks don't convert to regular blocks after hit | Hit P block, confirm visual change |
| 014 | [ ] | P2 | block | Diamonds/items don't visually emerge from blocks | Hit prize block, confirm item sprite appears |
| 015 | [ ] | P2 | player | Can't wall-grab (A+A to grab block edges) | Jump at wall edge, confirm grab |
| 016 | [ ] | P2 | level | No level intro screen showing level name | Enter level, confirm name card |
| 017 | [ ] | P2 | form | Masks/helmets not visible when they appear | Hit helmet block, confirm sprite |
| 018 | [ ] | P2 | form | No form-change animation when equipping helmet | Collect helmet, confirm transform anim |
| 019 | [ ] | P2 | form | Player doesn't transform at all | Collect helmet, confirm form changes |
| 020 | [ ] | P3 | render | No water/shimmer background effect | Load water level, confirm shimmer |
