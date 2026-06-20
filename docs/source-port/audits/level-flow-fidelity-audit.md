<!-- Generated 2026-06-10 by the fidelity-audit-wave2 workflow: one read-only Explore scout enumerated the units, one read-only Explore auditor per unit compared the live native path to the disassembly, and an independent refuter (defaulting to refuted) adversarially verified every high/medium gap before inclusion. Low gaps were included without a refute pass. This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native file:line searched. Re-run the workflow to refresh. -->

# Level-Flow and Timer Fidelity Audit

2026-06-19 status: the backlog candidate table below has no open `[ ]` rows. The remaining
`[?]` entries are fixed candidates awaiting human validation, not new coding
targets. Current `crates/wildside-core` code contains the named regressions for
death extra-HP reset, final-level `$33` routing, teleporter MapOrder reverse
scan, 181-second timer/clock normalisation, Continue/GameOver routing, FC36
NoHit/NoPrize reset, respawn invulnerability source, pit/timer ordering, and the
stale `sub_B43A` WarpCheat row. Backlog #312 is a fixed candidate; continue with
#313 for the next actionable fidelity row.

Per-unit audit of the Kid Chameleon source-port level-flow, timer, death and
end-of-level subsystems. The dominant failure pattern is "faithful-but-untranslated
normalisation": the ROM keeps several state variables in a deliberately
pre-compensated form, and the native port previously dropped the compensating
step or substituted a structurally simpler model. Examples of the ROM's
pre-compensation:

- the `addq #1,(Time_Seconds_low_digit)` + `SubSeconds=1` timer normalisation at
  level start and at each clock collect,
- the `clr.w (Extra_hitpoint_slots)` that `lose_life` performs before reseeding HP,
- the MapOrder reverse/forward index translation.

The timer normalisation, death extra-HP clear, final-level `$33` ending check,
and teleporter MapOrder reverse scan are now fixed-candidates with regressions.

## Coverage summary

| Unit | Coverage | Confirmed gaps |
|---|---|---|
| level-timer-countdown-semantics | partial | 2 |
| timer-reinit-flag-fc36-carry | partial | 1 |
| death-cause-routing-and-windup-entry | partial | 3 |
| time-out-death-consequence | partial | 0 |
| death-respawn-lifecycle-state-machine | partial | 2 |
| respawn-state-persistence-reset | full | 1 |
| flagpole-touch-level-complete-sequence | partial | 0 |
| level-complete-to-next-level-exit-routing | partial | 1 |
| in-level-warp-teleporter-routing | partial | 1 |
| murder-wall-level-start-arm | partial | 0 |
| lives-decrement-gameover-continue-branch | partial | 3 |
| final-boss-ending-trigger-and-bonus-entry | partial | 2 |

Totals: 12 units audited, 1 full, 11 partial, 0 stub. 17 confirmed gaps (4 high,
6 medium, 7 low), after merging the duplicated `Extra_hitpoint_slots`-on-death
finding across three units into one entry. 3 high/medium claims were refuted and
dropped before inclusion (the transform-freeze timer-drain claim, the flagpole
`Number_blank_top_rows` Y-offset claim, and the murder-wall non-zero-camera
initial-X claim), each because the native already honours the behaviour or the
divergent branch is unreachable in shipped data.

## Confirmed gaps

### High severity

**RESOLVED candidate: death respawn clears `Extra_hitpoint_slots` before restoring base HP** (`lose_life` / `Death` / `Init_Timer_and_Bonus_Flags`; reported by units respawn-state-persistence-reset, death-respawn-lifecycle-state-machine and death-cause-routing-and-windup-entry)
The ROM's `lose_life` makes `clr.w (Extra_hitpoint_slots).w` (kid.asm:14764) its very first action, before `move.w #2,(Number_Hitpoints).w` (kid.asm:14768), so every same-level death respawn drops the player back to the base TheKid HP cap of 2; the `Death` routine also zeroes the field on entry (kid.asm:14719). Native now clears `progression.extra_hitpoint_slots` in `enter_death_hidden_state` before reseeding `number_hitpoints = PlayerForm::TheKid.max_hitpoints()`, matching the same death/respawn boundary used by the live `lose_life` model. The regression `loc_b7ea_death_strips_helmet_and_resets_to_the_kid` seeds stale DP extra-HP slots before death and proves the post-windup hidden state has form=TheKid, base HP=2, and slots=0. Kind: fixed-candidate. `crates/wildside-core/src/combat.rs`, `crates/wildside-core/src/tests/level_tests.rs`.

**RESOLVED candidate: final-level ending trigger uses hard-coded `Final_LevelID=$33`** (`loc_D7F4` / `End_Credits` / `Final_LevelID` / `Boss4_LevelID`; reported by units level-complete-to-next-level-exit-routing and final-boss-ending-trigger-and-bonus-entry)
The ROM EOL routine routes the ending with an exact compare: `cmpi.w #Final_LevelID,(Current_LevelID).w; beq.w End_Credits` else `addq.w #1,(Current_LevelID).w; bra.w Teleport` (kid.asm:16912-16917), where `Final_LevelID = $33` (kid.asm:30) `== Boss4_LevelID = $33` (kid.asm:47). `Current_LevelID` is the logical MapOrder index, so native now mirrors `loc_D7F4` by treating `LevelId(0x33)` as final regardless of later MapOrder tail entries. Regressions `final_level_id_33_sets_game_complete_on_advance_loc_d7f4` and `map_order_tail_after_final_level_does_not_end_game_loc_d7f4` cover the final-level ending and the non-final tail case. Kind: fixed-candidate. `crates/wildside-core/src/level.rs`, `crates/wildside-core/src/tests/combat_tests.rs`.

**RESOLVED candidate: teleporter destination uses the `loc_BCD6` MapOrder reverse scan** (`loc_BCB2` / `loc_BCD6` / `loc_BCEA` / `Get_PlatformLayoutAddress`)
The ASM telepad record's high byte at `+6` is a map number, not a logical index. `loc_BCB2` extracts it (`move.w (a4),d7; asr.w #8,d7`), then `loc_BCD6` scans `LnkTo_MapOrder_Index` byte-by-byte so that `d6` becomes the logical index whose MapOrder entry equals that map number; `loc_BCEA` compares `d6` (the index) to `Current_LevelID` and stores `d6` back. Native now routes teleporter destinations through `teleporter_dest_level_loc_bcd6`, stores the logical MapOrder index in `pending_level_skip`, and preserves same-level inline repositioning when the scan resolves to the current logical level. Regressions `teleporter_cross_level_sets_pending_skip_loc_bcd6` and `live_teleporter_pending_skip_uses_map_order_index_loc_bcd6` cover the non-identity MapOrder case. Kind: fixed-candidate. `crates/wildside-core/src/level.rs`, `crates/wildside-core/src/combat.rs`, `crates/wildside-core/src/blocks.rs`, `crates/wildside-core/src/tests/combat_tests.rs`, `crates/wildside-core/src/tests/projectile_tests.rs`.

### Medium severity

**RESOLVED candidate: level timer starts with the ROM's 181s pre-decrement budget** (`Init_Timer_and_Bonus_Flags` / `Init_SpriteAttr_HUD` / `loc_BE82`)
On level entry the ROM sets minutes=3/seconds=0 in `Init_Timer_and_Bonus_Flags` (kid.asm:6555-6561), then `Init_SpriteAttr_HUD` does `addq.w #1,(Time_Seconds_low_digit)` (kid.asm:27099) and `move.w #1,(Time_SubSeconds)` (27100). The true start state is therefore 3:01 with `SubSeconds=1`; the first gameplay frame drains `SubSeconds 1->0`, reloads `$3C`(60) and decrements 3:01->3:00 (kid.asm:15482-15485). Native now seeds `LevelTimer::new()` at 181 seconds with `sub_frames=1`, preserving the same first-frame normalisation and one-second-longer death budget. The EOL completion-time path deliberately uses a separate `LEVEL_TIMER_COMPLETION_BASE_SECONDS=180` because `loc_C0CC` subtracts remaining time from literal `$B4` and adds `Clocks_collected*$B4`. Kind: fixed-candidate. Native locations: types.rs, constants.rs, combat.rs, trace.rs.

**RESOLVED candidate: clock-bonus applies the `loc_FF92` `+1`-second carry before `loc_FFDE` adds three minutes** (`loc_FFDE` / `loc_FFFC` / `loc_10002`)
The ROM clock-collect tail, immediately before the `+3`-minute add, does `addq.w #1,(Time_Seconds_low_digit)` with full BCD carry into seconds-high/minutes (kid.asm:21271-21279), then `loc_FFDE` adds 3 to `Time_Minutes` capped at 10:00 (21282-21293), then `loc_FFFC` sets `Time_SubSeconds=1` (21296). Native `add_clock_bonus` now adds `LEVEL_TIMER_CLOCK_COLLECT_SECONDS=181` and caps to 10:00, while the EOL speed-bonus elapsed formula still multiplies `Clocks_collected` by the ROM's flat `$B4`. Regressions cover the clean +3:00 after the compensating tick and the 6:59->10:00 cap boundary. Kind: fixed-candidate. Native locations: types.rs, constants.rs, combat.rs, trace.rs.

**RESOLVED candidate: Continue (CONTINUE?) screen accepts Start only and preserves the second Start window** (`loc_D350` / `loc_D3AA` / `Use_Continue` / `Continue_Screen`)
On the CONTINUE? screen the ROM polls `Ctrl_Held` and branches to `Use_Continue` only when the Start bit is held: `tst.b (Ctrl_Held).w; bmi.s Use_Continue` (kid.asm:16528-16532, identically at 16555-16559); `Button_Start=7` so `tst.b`+`bmi` tests bit 7 alone. Native gates the live `ContinueCountdown` use path on `input.start`, has a regression asserting A/B/C alone leave the countdown running, and includes the post-fade `$EF` `loc_D3AA` Start window before title return. Reachable in normal play via GameOver -> ContinueCountdown when `number_continues>0`. Kind: fixed-candidate.

**RESOLVED candidate: GameOver arm no longer injects a 2P player-switch the ASM zero-lives path never performs** (`loc_D052` / `loc_D17E` / `loc_D25E` / `loc_D2CE` / `loc_D3F0` / `Use_Continue`)
When the current player's `Number_Lives` reaches 0 the ROM always takes `beq.w loc_D052` (game-over teardown), even in two-player: `loc_D17E` merely allocates the P1/P2 indicator text objects then falls into the same `loc_D25E` continue/title decision, reaching `loc_D3F0` (`clr.w (Game_Mode)` -> title) when no continues remain (kid.asm:16325-16601). The surviving-player handoff (`loc_B894`/`loc_B8D0`/`not.b Current_player`) lives only on the non-zero-lives `Teleport` path. Native now keeps `try_switch_player()` only in the non-zero-lives Dying branch and returns to title from the no-continues GameOver arm even when the other 2P slot has lives. Regressions `loc_d25e_p1_game_over_does_not_force_p2_active` and `two_player_game_over_returns_to_title_even_when_other_has_lives` cover the corrected zero-lives path, while `loc_b8ba_two_player_both_alive_swaps_current_player` preserves the real non-zero-lives turn switch. Kind: fixed-candidate. `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/tests/helmet_tests.rs`.

**RESOLVED candidate: FC36-set respawn/continue/cross-level-teleport paths clear both NoHit and NoPrize flags** (`Init_Timer_and_Bonus_Flags` / `lose_life` / `Use_Continue` / `loc_BCEA`)
When `$FFFFFC36` is set, `Init_Timer_and_Bonus_Flags` reloads the timer and clears both bonus flags: `sf (NoHit_Bonus_Flag); sf (NoPrize_Bonus_Flag)` (kid.asm:6559-6560). Native now clears both `level_damage_taken` and `level_prize_collected` in `reset_current_level_runtime_state_for_respawn` and `maybe_begin_pending_level_skip_resume`, so death/continue respawns and cross-level teleporter resumes restore the NoHit/NoPrize bonus flags together. Regressions `fc36_respawn_reset_clears_nohit_and_noprize_flags` and `fc36_level_skip_resume_clears_nohit_and_noprize_flags` cover the two live reset paths. Kind: fixed-candidate. `crates/wildside-core/src/level.rs`, `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/tests/level_tests.rs`.

### Low severity

**RESOLVED candidate: post-respawn invulnerability is gated on `Just_received_damage`** (`Check_for_recent_damge` / `loc_B72C` / `loc_BC40` / `Crushed_to_Death` / `loc_9142`)
In the ROM, post-damage invulnerability is the latched `Just_received_damage` byte (`$FFFFFA73`): `Check_for_recent_damge` (kid.asm:14633) skips all damage while it is set, and the cosmetic flash thread clears it only after its 66-frame cycle (kid.asm:15214). It is `st`-set only by depleting contact/projectile damage at `loc_B72C` (kid.asm:14657); pit/timer/crush deaths enter `lose_life` without setting it. Native now preserves `just_received_damage` across the dying reset only when a damage hit latched it, and the live respawn hand-off seeds `PLAYER_DAMAGE_INVULNERABLE_FRAMES` only for that latched case. Regressions `respawn_handoff_without_damage_latch_does_not_seed_invulnerability`, `damage_latched_dying_reset_preserves_respawn_invulnerability_source`, and `respawn_handoff_with_damage_latch_seeds_invulnerability` cover both branches. Kind: fixed-candidate. `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/tests/helmet_tests.rs`.

**RESOLVED candidate: death entry clears `Extra_hitpoint_slots`, retaining no DP slots across death/respawn** (`Death` / `lose_life`)
Reported by the death-cause-routing unit as the same root as the high-severity carry above (the `Death` routine zeroes the field at kid.asm:14719 and `lose_life` again at kid.asm:14764). Native now clears `progression.extra_hitpoint_slots` in the live death-hidden transition. Merged into the high-severity entry; retained here only to record the death-cause unit's independent corroboration. Kind: fixed-candidate.

**Resolved: `resolve_frame_hazards` checks level-timer death before pit-fall; the ROM defers timer death one frame, so a coincident fall picks the wrong windup path** (`loc_9142` / `Crushed_to_Death` / `Character_CheckCollision`)
Native now ticks the timer and queues `VoiceNoTime`, then allows the immediate `sub_902A` `loc_9142` pit branch to win before falling back to the deferred timer-death path. This matches the ROM split: `Decrease_Time_Left` writes collision_type=4 and plays no-time (kid.asm:15494-15502), while the pit kill branches to `Death` immediately with `d7=1` (kid.asm:10454/10550). Covered by `loc_9142_pit_death_wins_over_same_frame_timer_expiry`.

**Resolved: `fell_below_level` fires regardless of y-velocity direction; the ROM pit kill is only reachable from the downward floor-probe branch** (`sub_902A` / `loc_9142` / `loc_908C`)
Native `fell_below_level` now returns false when `y_velocity < 0`, matching `sub_902A`: the `bmi loc_908C` upward branch skips the `Level_height_pixels` compare, while only the non-negative y-velocity branch can reach `loc_9142` pit death (kid.asm:10440-10455, 10476-). Covered by `loc_908c_sub_902a_upward_below_level_height_does_not_trigger_pit_death`.

**Resolved: GameOver hold runs the full 419-frame timer before the continue decision; the ASM reaches the continue screen after the short shared fade when continues remain** (`loc_D0C0` / `loc_D15E` / `loc_D25E` / `Continue_Screen` / `loc_D2CE`)
Native now clamps continues-present GameOver to the short pre-continue fade (`loc_D0C0` plus the `loc_D15E` fade loop) before entering `ContinueCountdown`, while the `$3C + $1A3` no-continues title-return hold remains on the zero-continues path. Covered by `game_over_with_continues_reaches_continue_screen_after_short_fade` and the existing GameOver routing tests.

**Resolved: `sub_B43A` WarpCheat dest-level override is implemented outside the floor-probe helper** (`sub_B43A` / `loc_B482` / `WarpCheatStart_LevelID` / `WarpCheatDest_LevelID`)
`sub_B43A`'s entry preamble (kid.asm:14352-14364) implements a developer warp cheat: when `Character_Movement == MoveID_Crawling`, the level is `WarpCheatStart_LevelID` (1), the Kid is at exact pixel `x=$09D3,y=$005F`, and Button_B is held, it writes `move.w #WarpCheatDest_LevelID,(Current_LevelID).w` (`$32`), sets `collision_type=colid_kidbelow` and `object_meta=$A0`. Native handles that side-effect in the live walking/crawling shell via `try_apply_warp_cheat_sub_b43a` before continuing ordinary movement, then routes the fabricated `$A0` flag-touch through `begin_flagpole_level_complete_loc_c056`. Covered by `sub_b43a_warp_cheat_requires_button_b_held` and `sub_b43a_warp_cheat_completes_destination_level_32`. Kind: fixed-candidate.

## Already tracked

- **Death respawn never clears `Extra_hitpoint_slots`** (`lose_life`): `docs/source-port/audits/level-loading-fidelity-audit.md:202-203` (cross-ref `combat-fidelity-audit.md:123`). The entry is stale and under-rated: it logs the identical `lose_life` non-clear but rates it "Hygiene until the field is populated" on the now-false "field unused, inert, permanently 0" premise; its own conditional fired with the 2026-05-31 ExtraHitPoint DP fix (`../backlog-archive.md:8793-8794`). No active `../backlog.md` row exists, so the live high-severity gap is effectively untracked under a dismissed inert precursor. Re-rate to high. The escalated entry appears as the first backlog candidate below.

## Backlog candidates

| # | Status | Pri | System | Summary | Verify |
|---|---|---|---|---|---|
| NEW | [?] | P1 | level | Fixed candidate: death respawn clears `Extra_hitpoint_slots` on the ROM `Death`/`lose_life` path before restoring base HP | Collect an ExtraHitPoint diamond power (slots->1), die, respawn, pick up a helmet; assert max HP = base+0 not base+slots (state probe) |
| NEW | [x] | P1 | level | Final-level ending follows ASM `loc_D7F4`'s direct `Current_LevelID == Final_LevelID ($33)` comparison instead of treating the last MapOrder entry as final | `final_level_id_33_sets_game_complete_on_advance_loc_d7f4`; `map_order_tail_after_final_level_does_not_end_game_loc_d7f4` |
| NEW | [x] | P1 | level | Teleporter destination follows ASM `loc_BCD6` by reverse-scanning MapOrder and storing the logical level index, not the raw map byte | `teleporter_cross_level_sets_pending_skip_loc_bcd6`; `live_teleporter_pending_skip_uses_map_order_index_loc_bcd6` |
| NEW | [?] | P2 | level | Fixed candidate: level timer starts with the ROM's 181s `Init_SpriteAttr_HUD` pre-decrement seed while `loc_C0CC` completion-time math keeps the flat `$B4` base | Run a level to time-out; assert timer-death frame matches ROM (1 + 181x60 = 10861 frames), not 10801 |
| NEW | [?] | P2 | level | Fixed candidate: clock-bonus applies the `loc_FF92` +1s carry before the `loc_FFDE` +3min/cap, with `loc_C0CC` clock elapsed kept at flat `$B4` | Collect a clock at 2:00; assert stable 5:00 next frame; collect at 6:59 and assert ROM cap behaviour |
| NEW | [?] | P2 | level | Fixed candidate: CONTINUE? screen consumes a continue only on Start and includes the `loc_D3AA` second Start window / timeout tail | Press A/B/C only, no continue burned; press Start in either window, continue consumed; no input returns at ROM timeout |
| NEW | [x] | P2 | level | GameOver arm no longer injects a 2P player-switch the ROM never does on zero-lives; the faithful non-zero-lives Dying-arm switch remains | `loc_d25e_p1_game_over_does_not_force_p2_active`; `two_player_game_over_returns_to_title_even_when_other_has_lives`; `loc_b8ba_two_player_both_alive_swaps_current_player` |
| NEW | [x] | P2 | level | FC36-set respawn/continue/teleport paths clear the NoHit flag (`level_damage_taken`) alongside `level_prize_collected` | `fc36_respawn_reset_clears_nohit_and_noprize_flags`; `fc36_level_skip_resume_clears_nohit_and_noprize_flags` |
| NEW | [x] | P3 | level | Native grants 66-frame respawn invuln after every death; ROM only after damage-latched deaths; seed invuln from the cause that latched `Just_received_damage` | `respawn_handoff_without_damage_latch_does_not_seed_invulnerability`; `respawn_handoff_with_damage_latch_seeds_invulnerability`; `damage_latched_dying_reset_preserves_respawn_invulnerability_source` |
| NEW | [x] | P3 | level | `resolve_frame_hazards` orders timer-death before pit-fall; ROM defers timer death one frame; reorder so a coincident below-floor frame yields PitFall | `loc_9142_pit_death_wins_over_same_frame_timer_expiry` |
| NEW | [x] | P3 | level | `fell_below_level` ignores y-velocity sign; ROM pit kill only from the downward floor-probe branch; guard the check on `y_vel >= 0` | `loc_908c_sub_902a_upward_below_level_height_does_not_trigger_pit_death` |
| NEW | [x] | P3 | level | GameOver hold runs full 419 frames before the continue decision; ROM reaches the continue screen in ~34; run only the short fade tail when continues remain | `game_over_with_continues_reaches_continue_screen_after_short_fade` |

Not worth a backlog row: the death-cause unit's `Extra_hitpoint_slots` re-report (folded into the P1 row above).

## Not audited

Scout exclusions (out of scope by design, owned elsewhere):

- **HUD/audio timer consequences**: the `sfx_Voice_no_time` time-up cue (kid.asm:15501), the low-time HUD warning blink (`sub_BD0A`, kid.asm:15518-15524), the `$FFFFFB4B` HUD-latch on flagpole/level-complete (backlog #175), and death/voice SFX are audio/visual/shell concerns outside the gameplay-timer and death-flow scope.
- **Warp-mechanic / level-loading concerns** already owned by `blocks-fidelity-audit.md` and `level-loading-fidelity-audit.md`: the 90-frame Telepad countdown gate, same-level full-reload vs inline reposition (no-fade, invented `on_ground=true`), the dual teleporter consumers in `blocks.rs`+`combat.rs`, cross-level `dest_map_id` handling, and `loc_BCD6`'s `$FC36`/`$FC29` shell-domain side-effects (subsumed by the already-reported full-Teleport-transition gap).
- **Combat / scoring / shell domains**: murder-wall crush damage geometry (`sub_5E02`, combat scope, backlog #283), the EOL score tally and 100K-trip seed, two-player game-over screen rendering, `return_to_title` new-game state, and the EndingSequence presentation staging (`../dark-model-wiring-triage.md:146`).
- **`Init_SpriteAttr_HUD` / level-header parsing**: header/flag decode and murder-wall arm parsing are level-loading scope and were only cross-checked for the timer-normalisation and flag-decode facts cited above.

Units with ambiguity needing oracle probes:

- **murder-wall-level-start-arm**: could not extract the exact `PlayerStart_X` for the three murder-wall levels (Hills_of_the_Warrior_1, Forced_Entry, Bloody_Swamp) from raw ASM map data, so the per-level magnitude of the init-X structural divergence is unconfirmed; the refuter established that all three start below screen-width, making the ROM's non-zero-camera branch unreachable, so no gap was raised. Re-probe if a murder-wall level with a later-screen start is ever introduced.
- **death-respawn-lifecycle-state-machine**: the exact set of ASM `Death` entrants that latch `Just_received_damage` was inferred from the two `bra Death` sites read plus `loc_B72C`; other Death entrants (timer-death paths near `loc_D052`) were not exhaustively traced, though pit and murder-wall (the most player-reachable non-damage deaths) are confirmed not to latch it.
- **flagpole-touch-level-complete-sequence**: the disassembly copy does not expand `loc_C2F2`/`loc_C326`/`loc_C428`/`sub_C378`/`loc_D468`/`loc_D770`, so the EOL-coroutine internals cited by native comments could not be line-checked; those are largely scoring/visual-shell and several are covered by backlog #175.
- **Tooling caveat across units**: `kid.asm` contains embedded non-UTF-8 bytes that make plain `grep` silently return empty; `grep -a`/`rg`/`sed` were required for all ASM quotes. Treat any unit audited only via plain `grep` as suspect.
