<!-- Generated 2026-06-10 by the fidelity-audit-wave2 workflow (1 read-only Explore scout enumerated the units; one read-only Explore auditor per unit compared the live native path to the disassembly; every high/medium gap adversarially verified by an independent refuter - defaulting to refuted - before inclusion; low gaps included without a refute pass). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native file:line searched. Re-run the workflow to refresh. -->

# Camera & Scroll Fidelity Audit

Adversarially-verified per-unit audit of the Kid Chameleon source-port camera and scroll subsystem. The core X/Y follow mathematics (`sub_71E4` target clamp, ±1 edge nudges, `(target-camera+4)>>3` smooth step clamped to ±8, the post-velocity defensive re-clamp, and the page-snap level seed) are faithful and reachable on the live path. The remaining open camera-audit gap is the respawn path's invented +2px carry. The old `.min(30)` cap that made the down-look pan inert has been removed, so remaining look-offset mismatches are small but no longer hidden by that cap.

## Coverage summary

| Unit | Coverage | Confirmed gaps |
|---|---|---|
| horizontal-follow-window | full | 0 |
| vertical-follow-and-final-clamp | partial | 0 |
| look-down-accumulator | full | 0 |
| look-offset-ceiling-pullbacks | full | 0 |
| follow-call-site-coverage | full | 0 |
| murder-wall-left-bound-advance | full | 0 |
| level-bound-camera-clamping | full | 0 |
| warp-teleport-camera-entry | partial | 0 |
| death-respawn-camera-convergence | full | 1 |
| bg-parallax-and-demo-camera | full | 0 |

Totals: 10 units audited, 8 full, 2 partial, 0 stub. 1 confirmed gap (0 high, 0 medium, 1 low). 2 high/medium claims were refuted and dropped (the high `.min(30)` look-down cap, refuted as a duplicate of an already-tracked, already-medium `terrain-fidelity-audit.md` gap with over-stated severity; the cross-level teleporter dest-discard, refuted as an already-tracked recombination with no reachability evidence).

## Confirmed gaps

### High severity

None.

### Medium severity

None.

### Low severity

**Look-down increment not gated on `KidGrabbedByHand` (native increments the accumulator while a Hand grabs Kid)** (`loc_7772` / `loc_8C12` / `loc_8CB6`)
Fixed 2026-06-13: the merged native look-offset branch now keeps Down-held as a no-decay case, but only applies the `addq.w #1` equivalent while `kid_grabbed_by_hand` is false. Regression: `loc_8c70_grabbed_kid_down_held_freezes_look_offset`.

**`loc_7334` `Camera_max_X_pos` right-bound wall clamp fixed** (`loc_7334` / `loc_7316` / `loc_7346`)
Fixed 2026-06-13: the forward MurderWall path now clamps `MurderWall_X_pos.w` to `Camera_max_X_pos` (`level width - $140`) immediately after the accelerated `add.l d7,(MurderWall_X_pos).w` and before the `loc_7346` camera-gap push. The clamp preserves the fixed-point fractional low word, matching the ROM's word write. Regression: `loc_7334_forward_wall_clamps_to_camera_max_x_before_push`.

**`loc_726C` reference (`$FFFFFA2E`) post-collision store fixed** (`loc_726C` / `loc_75D4` / `loc_8218` / `loc_8BF0` / `sub_71E4`)
Fixed 2026-06-13: native now updates `camera_last_kid_y` after the movement/collision step, matching the ROM's `Character_CheckCollision; move.w y_pos(a3),($FFFFFA2E)` order at `loc_75D4`, `loc_8218`, and `loc_8BF0`. Regression: `loc_75d4_camera_last_kid_y_stores_post_collision_y`.

**`loc_728C` look-down accumulator floor-clamp decrement is already modelled** (`loc_728C` / `loc_72A8`)
Fixed before this audit refresh: native applies `camera_y_bottom_look_adjust_loc_728c` in the same pre-camera `sub_71E4` adjustment block as `loc_726C` and `loc_727E`, preserving the current frame's shifted target while decrementing `$FFFFFB58` for future frames when the offset-extended target saturates at `height-$E0`. Regression: `loc_728c_bottom_bound_decrements_look_offset_before_down_increment`.

**Iron Knight wall-climb ATTACH frame camera follow fixed** (`loc_A276` / `loc_A28A` / `sub_71E4`)
Fixed 2026-06-13: the `AttachHibernate` branch now runs the native `sub_71E4` camera-follow equivalent before hibernating, matching the ASM fallthrough from `loc_A276` into `loc_A28A`. Regression: `loc_a28a_attach_hibernate_advances_camera_follow`.

**Respawn camera +2px Y carry is an invented per-frame substitute, gated by residue value not by level or ASM trajectory** (`sub_71E4` / `loc_71EC` / `loc_726C` / `loc_727E` / `loc_728C` / `loc_72A8` / `loc_72C8` / `loc_72E2` / `Load_InGame (loc_11F48 camera seed)` / `lose_life` / `Teleport` / `loc_B8DE` / `loc_6E2`)
`finish_respawn_camera_convergence` (level.rs:18786-18801) fabricates a final camera beat by re-running `update_camera_axis` with `player_y+2` whenever the settle loop leaves the camera exactly 2px above the plain target (`target_y - camera_y == 2`) and grounded, nudging camera +1px. The ROM has no hidden `sub_71E4` settle loop: `Load_InGame` seeds the camera once by page-snap (`divu.w #$140`/`divu.w #$E0`, floor to screen cell) then returns (kid.asm:24132-24145, ending `move.l d0,(Camera_Y_pos).w` then `rts`); `sub_71E4` only runs on real, player-controlled gameplay frames. A genuine extra `sub_71E4` beat at delta=2 produces velocity `(2+4)>>3 = 0` — it does nothing; the entire +1px effect comes from the invented +2 player-Y shift, which has no ASM counterpart (the "carrying a final +2px pre-settle Y target" comment is inferred from oracle traces, not the disassembly). The carry is gated on residue value (`delta==2`), not on level `0x24`/`0x51` (lib check absent at level.rs:18789-18798), so any other level — including cases the ROM itself leaves at delta=2 — gets an unjustified +1px down nudge. Root cause: the native floor-snap-then-gravity-replay approximation reaches a different camera fixed point than the ROM's frame-by-frame fall for these two levels, and the carry is a band-aid over that trajectory divergence. Consequence is purely a 1px camera-Y at the respawn settle frame, observable only to a frame-exact TAS respawn comparison; it currently yields the correct oracle values for `0x24`/`0x51` per backlog #037. No `// DEVIATION` marker, so it is presented as faithful. Kind: deviating. `crates/kid-core/src/level.rs:18782-18802`, `crates/kid-core/src/level.rs:18765`, `crates/kid-core/src/level.rs:17932`, `crates/kid-core/src/physics.rs:894-928`.

## Already tracked

- **Resolved `.min(30)` cap on `$FFFFFB58` disabled the entire look-down camera pan** (`loc_728C` / `loc_7772` / `loc_8C70` / `loc_8218`) — `terrain-fidelity-audit.md` lines 82-83, backlog row at line 99. Native now grows the accumulator past `$1E`; regression: `loc_8c12_refresh_opening_slice_increments_look_offset_past_threshold`.
- **Resolved `loc_72A8`/`loc_728C` `subi.w #1,($FFFFFB58)` overshoot brake** (`loc_72A8` / `loc_728C`) — `terrain-fidelity-audit.md` lines 76-77, backlog row at line 106; the camera-audit duplicate above is now marked fixed.
- **Same-level teleporter does not re-run the camera snap and smooth-follows instead of hard-snapping** (`loc_6E2` / `loc_11F48` / `loc_BCEA`) — `blocks-fidelity-audit.md` lines 69-70 and 110-111, cross-referenced to `level-loading-fidelity-audit.md` lines 205-206.
- **FC29 header-PlayerStart skip helper unwired generically** (`loc_11BD2`) — `level-loading-fidelity-audit.md` line 117.
- **Dead `initialise_camera_bootstrap_loc_11F48` helper** (`loc_11F48`) — `level-loading-fidelity-audit.md` line 191 (hygiene-only; inline math identical).
- **Warp SFX / `$EE` flash omission** — `level-loading-fidelity-audit.md` lines 205-206.

## Backlog candidates

| # | Status | Pri | System | Summary | Verify |
|---|---|---|---|---|---|
| NEW | [x] | P2 | camera | Fixed 2026-06-13: forward MurderWall clamps `MurderWall_X_pos.w` to `Camera_max_X_pos` before the `loc_7346` camera push | `loc_7334_forward_wall_clamps_to_camera_max_x_before_push` |
| NEW | [x] | P3 | camera | Fixed 2026-06-13: Iron Knight wall-climb attach now runs the `loc_A28A` / `sub_71E4` camera-follow equivalent before hibernating | `loc_a28a_attach_hibernate_advances_camera_follow` |
| NEW | [x] | P3 | camera | Fixed before this audit refresh: `loc_728C` floor-clamp accumulator decrement is modelled by `camera_y_bottom_look_adjust_loc_728c` | `loc_728c_bottom_bound_decrements_look_offset_before_down_increment` |
| NEW | [x] | P3 | camera | Fixed 2026-06-13: look-down increment now honours the `KidGrabbedByHand` gate from `loc_7772`/`loc_8C70` while preserving Down-held no-decay | `loc_8c70_grabbed_kid_down_held_freezes_look_offset` |
| NEW | [x] | P3 | camera | Fixed 2026-06-13: `loc_726C` reference Y now stores post-collision `y_pos` into native `camera_last_kid_y` | `loc_75d4_camera_last_kid_y_stores_post_collision_y` |
| NEW | [ ] | P3 | camera | Respawn +2px Y carry is an invented residue-gated substitute, not gated by level or ASM trajectory; gate on level `0x24`/`0x51` or replace with a faithful frame-by-frame settle | Respawn on a level other than `0x24`/`0x51` that settles at delta=2; assert no unjustified +1px camera-Y nudge, and `0x24`/`0x51` still match oracle (#037) |

## Not audited

Scout exclusions (out of scope by design, owned elsewhere):

- **Camera_BG_X/Y parallax `>>2`** (`loc_72E2`) — render/weather-domain; the `camera_bg_scroll_loc_72e2` helper is `#[cfg(test)]`-only and faithfully reproduces the unsigned `lsr.l #2`, but the two `$FFFFF824`/`$FFFFF828` longs are written only at `loc_72E2` and read nowhere in the ROM (dead RAM). Live per-layer background scroll is the separate, properly-modelled `BackgroundScroll_ComputeShiftData`/`sub_30194` family in `level.rs`, which derives scroll from `background_scroll_longword`, not `Camera_BG_*`. Not a helper-unwired gap — the unwired helper corresponds to ROM-dead state.
- **Demo / title hidden-camera-Y wrap preload rows** (`demo_1_hidden_camera_y`, `title_screen_hidden_camera_ramp_loc_1B51A`) — feed only render-only intro/title VDP scroll and advisory object-roster/scoreboard counts, not the live gameplay camera follow; the frame-60 `$FF00_0000` wrapped-negative `ORIGINAL_QUIRK` is consistent with the `sub_1C246`/`loc_1B51A` down-ramp and not plainly contradicted by the ASM.
- **MurderWall crush damage model and invulnerability gating** — owned by `combat-fidelity-audit.md` (lines 39/42/105/106); MurderWall initial-X fractional divergence and palette-buffer dead helper owned by `weather-fidelity-audit.md` (lines 144/147).

Units with ambiguity or theoretical-only notes (not flagged as gaps):

- **Final post-velocity `.max(0).min(max_pos)` re-clamp on both axes** (physics.rs:925-927) — proven a no-op over the reachable state space (target pre-clamped to `[0,max_pos]`; edge nudges only move toward an in-bounds bound; smooth step clamped to ±8 toward an in-bounds target), and acknowledged in-code as a defensive guard. Not an invented saturation.
- **X-axis target upper-clamp signedness** (`loc_720E` unsigned `bcs.s` vs native signed `.min()`) — diverges only if `max_pos < 0` (level width below `$140` / one screen), which is never reachable since width is always ≥1 screen. Theoretical-only.
- **Look-offset add-before-vs-after the floor-at-0 clamp** (native adds offset before flooring `kid_y−$A0` to 0; ASM floors first then adds) — diverges only if Kid is simultaneously near the level ceiling and holding Down (`offset >= $1E`), which is not normally co-reachable; the ASM is effectively unambiguous and was not flagged. Would warrant an oracle probe only if that co-reachable state is demonstrated.
- **Respawn settle "hidden frames" abstraction** — the ROM `Load_InGame` page-snaps the camera once with no hidden `sub_71E4` settle loop; the native settle construct is a port-specific approximation of where the camera would land after N real gameplay frames. The trajectory divergence underlying the +2px carry (#038) is the root cause and would benefit from an oracle probe of the real frame-by-frame fall for levels `0x24`/`0x51`.
