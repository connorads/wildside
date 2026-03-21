# TAS Verification Parity Roadmap

Progressive milestones for verifying the native engine against TAS recordings via oracle comparison.

## Recordings

| Recording | Frames | Duration | Status |
|-----------|--------|----------|--------|
| `kid_chameleon_short_demo` | 800 | ~13s | 0 diffs (verified) |
| `kc_any_percent_2024` | 4916 | ~82s | clean through frame 1831; next authoritative frontier is frame 1832 |
| `any-percent-published` | 37759 | ~10.5min | untested since boot timing fix |

## Milestones

| # | Target | Frames | Phase | Status |
|---|--------|--------|-------|--------|
| M1 | First 1000 frames | 0-1000 | boot -> title -> demo play (0x1B) | [x] |
| M2 | Through title to level entry | 0-2000 | demo end -> title -> Start -> Level 1 | [ ] |
| M3 | First level traversal | 0-3000 | Level 1 gameplay: enemies, platforms, blocks | [ ] |
| M4 | First level transition | 0-4000 | Level 1 complete -> transition -> Level 2 | [ ] |
| M5 | Full kc_any_percent_2024 | 0-4916 | multiple levels, warps, pause | [ ] |
| M6 | any-percent-published first level | 0-3000 | different TAS, different route | [ ] |
| M7 | any-percent-published half run | 0-19000 | ~10 levels, bosses, forms | [ ] |
| M8 | Full any-percent-published | 0-37759 | boot to credits | [ ] |

## Frontier log

| Date | Recording | Frontier frame | Blocker | Fix |
|------|-----------|---------------|---------|-----|
| 2026-03-21 | kc_any_percent_2024 | 1832 | Demo-abort title return still exposes the normal title pose while oracle keeps Kid hidden during the later title rebuild | Split the demo-abort title handoff into a full live snapshot (through frame 1801) followed by a camera/form-only snapshot (through frame 1805), then cleared the remaining title-return state on frame 1806. Added `demo_title_return_blanks_kid_on_frame_1802_before_camera_reset` and `demo_title_return_clears_camera_and_progression_on_frame_1806`, re-ran `cargo test -p kid-core demo_title_return -- --nocapture` plus `demo_start_abort_stays_in_demo_until_frame_1774`, verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808 --checkpoint-interval 1` is CRC-clean apart from advisory object counts, then re-bisected to move the frontier from 1802 to 1832. |
| 2026-03-21 | kc_any_percent_2024 | 1802 | Demo-abort title mode was reusing the cold-boot title pose instead of the paused live demo snapshot | Preserved the paused demo gameplay snapshot on the title-return path so `mode=$002C` reuses the live Kid/camera state through frame 1801, matching oracle after the frame-1772 abort fix. Added `demo_title_return_keeps_live_snapshot_on_first_title_frames`, re-ran the targeted abort tests, verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1772 --frame-end 1780 --checkpoint-interval 1` and `--frame-start 1772 --frame-end 1860 --checkpoint-interval 1`, then re-bisected to move the frontier from 1774 to 1802. |
| 2026-03-21 | kc_any_percent_2024 | 1772 | Demo `0x1B` returns to the title screen one frame early after the frame-1755 landing window | Kept sub-half-pixel solid-floor touch snap out of the `AsmFaithful` landing path, so `sub_902A` no longer collapses frame 1755 into an early landing. Added `asm_faithful_solid_floor_snap_waits_for_contact_row` and `demo_solid_floor_landing_waits_until_frame_1756`, re-ran the nearby floor-snap tests, verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1748 --frame-end 1762 --checkpoint-interval 1` is CRC-clean with 0 authoritative diffs, then re-bisected to move the frontier from 1755 to 1772. |
| 2026-03-21 | kc_any_percent_2024 | >1500 | Previous frame-1406 demo 0x1B blocker resolved; next authoritative mismatch not yet triaged past the verified window | Restricted wide spawn-support settling to the demo bootstrap window so the post-latch arc no longer snaps onto support, then queued prize-diamond awards through a 32-frame `PrizeDiamondCollected_Init` delay instead of rewarding on contact. Added `demo_diamond_prize_stays_uncollected_on_frame_1406` and `demo_prize_diamond_awards_on_frame_1438_without_score`, re-ran the nearby demo prize tests, and verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1390 --frame-end 1500 --checkpoint-interval 1` is CRC-clean with 0 authoritative diffs. |
| 2025-03-19 | kc_any_percent_2024 | 955 | camera_y_pos | wall-sticky ledge snap + floor snap commits |
| 2026-03-21 | kc_any_percent_2024 | 1406 | Post-latch diamond collection/motion still advances one frame early on demo level `0x1B` | Shrunk the diamond pickup bounds to the smaller visible sprite instead of the full 16px box. Added `demo_diamond_prize_is_not_collected_on_frame_1405`, re-ran `cargo test -p kid-core demo_diamond_prize_is_not_collected_on_frame_1405 -- --nocapture`, `cargo test -p kid-core demo_cyclone_pickup_enters_transform_on_frame_1056 -- --nocapture`, `cargo test -p kid-core floating_helmet_prize_above_lower_body_does_not_collect -- --nocapture`, `cargo test -p kid-core floating_helmet_prize_entering_lower_body_collects -- --nocapture`, `cargo test -p kid-core prize_block_lifecycle_spawns_item_only_after_bounce_completion -- --nocapture`, `cargo test -p kid-core demo_prize_ceiling_restore_still_latches_on_frame_1396 -- --nocapture`, and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1360 --frame-end 1425 --checkpoint-interval 1` to move the frontier from 1405 to 1406. |
| 2026-03-21 | kc_any_percent_2024 | 1405 | Post-latch floating prize collection awards `score=500` / `number_diamonds=1` one frame before oracle on demo level `0x1B` | Restored prize cells now stay collision-solid through the clear frame's physics step, then clear/spawn at frame end. Re-ran `cargo test -p kid-core demo_transform -- --nocapture`, `cargo test -p kid-core prize_block_lifecycle_spawns_item_only_after_bounce_completion -- --nocapture`, `cargo test -p kid-core demo_prize_ceiling_restore_still_latches_on_frame_1396 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1138 --frame-end 1160 --checkpoint-interval 1`, and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1360 --frame-end 1425 --checkpoint-interval 1` to confirm the transform window stays clean and the frontier advances to frame 1405. |
| 2026-03-21 | kc_any_percent_2024 | 1396 | Transform-exit demo parity fix moved the first remaining authoritative diffs to `camera_y_pos` and `kid_y_pos` at frame 1396 during the post-prize-hit ceiling sequence on demo level `0x1B` | Re-ran `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1138 --frame-end 1160 --checkpoint-interval 1` to confirm the transform window is clean, then `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1360 --frame-end 1425 --checkpoint-interval 1` to capture the new frontier at frame 1396. |
| 2026-03-20 | kc_any_percent_2024 | 955 | kid_y_pos native=0x028f0000 (pixel-aligned) vs oracle=0x028fe6e0 (sub-pixel) | Exact frame 955: native grounded/jumping, oracle sub-pixel Y. Same root cause for any-percent-published at f958. Demo input verified matching (R+B+A from DEMO_INPUT_DATA_2[27]). Single-frame divergence suggests floor collision or gravity difference on level 0x1B terrain. |
| 2026-03-20 | kc_any_percent_2024 | 902 | kid_y_vel: native starts falling, oracle stays grounded | Boot timing fix: deferred sega screen (1f) and intro→title (2f) transitions to match oracle VBlank timing. New blocker: at kid=(0x00200000,0x02de4000) on level 0x1B demo, native drops off ground while oracle stays grounded. Kid idle, no input — pure ground-check logic difference. |
| 2026-03-20 | kc_any_percent_2024 | 955 | kid_y_pos sub-pixel: native=0x028f0000 oracle=0x028fe6e0 | Demo freefall fix: shadow gravity (Δy=frame×0x4000) without y_vel for frames 67-69. Frontier advanced 902→955. Next blocker: ASM physics order (position-first-then-gravity) differs from native (gravity-first-then-position), causing sub-pixel accumulation divergence over jump arcs. Two ASM floor landing paths: loc_AAF8 preserves sub-pixel (move.w), loc_AB22 zeroes it (clr.w $20). |
| 2026-03-20 | kc_any_percent_2024 | 955 | kid_y_pos sub-pixel (CRC-clean) | CRC cleanup: removed velocity + player_1_level_id from authoritative CRC (capture-timing artefacts). Velocity/objects made advisory. Unmasked real frontier from 866→955. Sub-pixel floor landing infra added (pre_gravity_y_vel + conditional preservation in floor/platform/spawn_support). Frame 955 blocker: GravityFirst position update uses post-gravity velocity, producing different sub-pixel than ASM's sub_A4EE (which runs after sub_B168). Sub-pixel preservation is correct but the sub-pixel has already diverged before landing. Root cause: need to match ASM's exact per-instruction order within sub_7ACC→CheckCollision→sub_B168→sub_A4EE. |

## Known issues by milestone

| Issue | Backlog | Expected milestone |
|-------|---------|-------------------|
| Demo camera settling | active | M1 |
| Floor-snap settling (4 levels) | #037 | M2-M4 |
| 14 enemy behaviour gaps | phase4 | M3-M7 |
| 6 RNG DEVIATIONs | research-map | M5-M7 |
| Shooter timer DEVIATION | #065 | M3-M5 |
| Mushroom bounce DEVIATION | #066 | M3-M7 |

## Infrastructure

| Improvement | Milestone | Status |
|-------------|-----------|--------|
| `--bisect` flag for `verify-tas` | M1 | [x] |
| CI TAS gate | M4 | [ ] |
| TAS frontier tracking | M2 | [~] |
| Framewise CRC checksumming | M7 | [ ] |
| Regression snapshots from TAS | M3 | [ ] |

## Verification commands

```bash
# Re-verify current frontier
cargo run --release -p kidcli -- verify-tas \
  --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json \
  --frame-end 1831 --checkpoint-interval 50

# Bisect to find first divergence
cargo run --release -p kidcli -- verify-tas \
  --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json \
  --frame-end 2000 --bisect

# Narrow to exact frame
cargo run --release -p kidcli -- verify-tas \
  --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json \
  --frame-start 1828 --frame-end 1838 --checkpoint-interval 1
```
