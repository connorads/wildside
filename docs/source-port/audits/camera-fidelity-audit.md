<!-- Generated 2026-06-10 by the fidelity-audit-wave2 workflow. A read-only Explore scout enumerated the units; one read-only Explore auditor per unit compared the live native path to the disassembly. Every high/medium gap was adversarially verified by an independent refuter (defaulting to refuted) before inclusion; low gaps were included without a refute pass. This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native file:line searched. Re-run the workflow to refresh. -->

# Camera & Scroll Fidelity Audit

> First committed 2026-06-11; last substantive update 2026-06-13.

Per-unit audit of the Kid Chameleon source-port camera and scroll subsystem. The core X/Y follow mathematics are faithful and reachable on the live path: `sub_71E4` target clamp, +/-1 edge nudges, `(target-camera+4)>>3` smooth step clamped to +/-8, the post-velocity defensive re-clamp, and the page-snap level seed. The old `.min(30)` cap that made the down-look pan inert has been removed, so remaining look-offset mismatches are small and no longer hidden by that cap.

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
| death-respawn-camera-convergence | full | 0 |
| bg-parallax-and-demo-camera | full | 0 |

Totals: 10 units audited, 8 full, 2 partial, 0 stub. 0 confirmed gaps. 2 high/medium claims were refuted and dropped: the high `.min(30)` look-down cap (refuted as a duplicate of an already-tracked, already-medium `terrain-fidelity-audit.md` gap with over-stated severity), and the cross-level teleporter dest-discard (refuted as an already-tracked recombination with no reachability evidence).

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

**Respawn camera +2px Y carry is now limited to oracle-backed levels** (`sub_71E4` / `loc_71EC` / `loc_726C` / `loc_727E` / `loc_728C` / `loc_72A8` / `loc_72C8` / `loc_72E2` / `Load_InGame (loc_11F48 camera seed)` / `lose_life` / `Teleport` / `loc_B8DE` / `loc_6E2`)
Fixed 2026-06-13: `finish_respawn_camera_convergence` now requires the respawn level to be `0x24` or `0x51` before applying the archived #037 +2px carry. Ordinary levels that settle with the same 2px residue no longer receive an unjustified +1px camera-Y nudge, while the two oracle-backed respawn regressions continue to match their cached camera targets. Regression: `respawn_camera_convergence_is_limited_to_oracle_backed_levels`.

## Already tracked

- **Resolved `.min(30)` cap on `$FFFFFB58` disabled the entire look-down camera pan** (`loc_728C` / `loc_7772` / `loc_8C70` / `loc_8218`): `terrain-fidelity-audit.md` lines 82-83, backlog row at line 99. Native now grows the accumulator past `$1E`; regression: `loc_8c12_refresh_opening_slice_increments_look_offset_past_threshold`.
- **Resolved `loc_72A8`/`loc_728C` `subi.w #1,($FFFFFB58)` overshoot brake** (`loc_72A8` / `loc_728C`): `terrain-fidelity-audit.md` lines 76-77, backlog row at line 106; the camera-audit duplicate above is now marked fixed.
- **Same-level teleporter does not re-run the camera snap and smooth-follows instead of hard-snapping** (`loc_6E2` / `loc_11F48` / `loc_BCEA`): `blocks-fidelity-audit.md` lines 69-70 and 110-111, cross-referenced to `level-loading-fidelity-audit.md` lines 205-206.
- **FC29 header-PlayerStart skip helper unwired generically** (`loc_11BD2`): `level-loading-fidelity-audit.md` line 117.
- **Dead `initialise_camera_bootstrap_loc_11F48` helper** (`loc_11F48`): `level-loading-fidelity-audit.md` line 191 (hygiene-only; inline math identical).
- **Warp SFX / `$EE` flash omission**: `level-loading-fidelity-audit.md` lines 205-206.

## Backlog candidates

| # | Status | Pri | System | Summary | Verify |
|---|---|---|---|---|---|
| NEW | [x] | P2 | camera | Fixed 2026-06-13: forward MurderWall clamps `MurderWall_X_pos.w` to `Camera_max_X_pos` before the `loc_7346` camera push | `loc_7334_forward_wall_clamps_to_camera_max_x_before_push` |
| NEW | [x] | P3 | camera | Fixed 2026-06-13: Iron Knight wall-climb attach now runs the `loc_A28A` / `sub_71E4` camera-follow equivalent before hibernating | `loc_a28a_attach_hibernate_advances_camera_follow` |
| NEW | [x] | P3 | camera | Fixed before this audit refresh: `loc_728C` floor-clamp accumulator decrement is modelled by `camera_y_bottom_look_adjust_loc_728c` | `loc_728c_bottom_bound_decrements_look_offset_before_down_increment` |
| NEW | [x] | P3 | camera | Fixed 2026-06-13: look-down increment now honours the `KidGrabbedByHand` gate from `loc_7772`/`loc_8C70` while preserving Down-held no-decay | `loc_8c70_grabbed_kid_down_held_freezes_look_offset` |
| NEW | [x] | P3 | camera | Fixed 2026-06-13: `loc_726C` reference Y now stores post-collision `y_pos` into native `camera_last_kid_y` | `loc_75d4_camera_last_kid_y_stores_post_collision_y` |
| NEW | [x] | P3 | camera | Fixed 2026-06-13: respawn +2px Y carry is gated to archived oracle levels `0x24`/`0x51`, preventing residue-only nudges on ordinary levels | `respawn_camera_convergence_is_limited_to_oracle_backed_levels`; #037 oracle respawn tests |

## Not audited

Scout exclusions (out of scope by design, owned elsewhere):

- **Camera_BG_X/Y parallax `>>2`** (`loc_72E2`): render/weather-domain. The `camera_bg_scroll_loc_72e2` helper is `#[cfg(test)]`-only and faithfully reproduces the unsigned `lsr.l #2`, but the two `$FFFFF824`/`$FFFFF828` longs are written only at `loc_72E2` and read nowhere in the ROM (dead RAM). Live per-layer background scroll is the separate, properly-modelled `BackgroundScroll_ComputeShiftData`/`sub_30194` family in `level.rs`, which derives scroll from `background_scroll_longword`, not `Camera_BG_*`. Not a helper-unwired gap: the unwired helper corresponds to ROM-dead state.
- **Demo / title hidden-camera-Y wrap preload rows** (`demo_1_hidden_camera_y`, `title_screen_hidden_camera_ramp_loc_1B51A`): feed only render-only intro/title VDP scroll and advisory object-roster/scoreboard counts, not the live gameplay camera follow. The frame-60 `$FF00_0000` wrapped-negative `ORIGINAL_QUIRK` is consistent with the `sub_1C246`/`loc_1B51A` down-ramp and not plainly contradicted by the ASM.
- **MurderWall crush damage model and invulnerability gating**: owned by `combat-fidelity-audit.md` (lines 39/42/105/106). MurderWall initial-X fractional divergence and palette-buffer dead helper owned by `weather-fidelity-audit.md` (lines 144/147).

Units with ambiguity or theoretical-only notes (not flagged as gaps):

- **Final post-velocity `.max(0).min(max_pos)` re-clamp on both axes** (physics.rs:925-927): proven a no-op over the reachable state space (target pre-clamped to `[0,max_pos]`; edge nudges only move toward an in-bounds bound; smooth step clamped to +/-8 toward an in-bounds target), and acknowledged in-code as a defensive guard. Not an invented saturation.
- **X-axis target upper-clamp signedness** (`loc_720E` unsigned `bcs.s` vs native signed `.min()`): diverges only if `max_pos < 0` (level width below `$140` / one screen), which is never reachable since width is always >=1 screen. Theoretical-only.
- **Look-offset add-before-vs-after the floor-at-0 clamp** (native adds offset before flooring `kid_y-$A0` to 0; ASM floors first then adds): diverges only if Kid is simultaneously near the level ceiling and holding Down (`offset >= $1E`), which is not normally co-reachable. The ASM is effectively unambiguous and was not flagged. Would warrant an oracle probe only if that co-reachable state is demonstrated.
- **Respawn settle "hidden frames" abstraction**: the ROM `Load_InGame` page-snaps the camera once with no hidden `sub_71E4` settle loop; the native settle construct is a port-specific approximation of where the camera would land after N real gameplay frames. The trajectory divergence underlying the +2px carry (#038) is the root cause and would benefit from an oracle probe of the real frame-by-frame fall for levels `0x24`/`0x51`.
