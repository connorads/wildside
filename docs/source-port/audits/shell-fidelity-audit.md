<!-- Generated 2026-06-10 by the fidelity-audit-wave2 workflow (1 read-only Explore scout enumerated the units; one read-only Explore auditor per unit compared the live native path to the disassembly; every high/medium gap adversarially verified by an independent refuter, defaulting to refuted, before inclusion; low gaps included without a refute pass). This is a reviewable research artefact, not a gate. Each gap cites the owning ASM label and the native file:line searched. Re-run the workflow to refresh. -->

# Game Shell & HUD Fidelity Audit

> First committed 2026-06-11; last substantive update 2026-06-20.

2026-06-19 status: backlog #313 is blocked/exhausted. The remaining audit row
is the visual pause-render VDP save/restore cluster: `loc_695C` saves five
plane-A rows into `Decompression_Buffer` and `loc_6B68` restores them on
unpause. Current native-player rendering is stateless and rebuilds draw
commands from world/trace data every frame; it already covers the visible pause
menu tilemap, live cursor, final-RGB darken pass, and sprite-hide rule, but it
does not own a persistent VDP nametable that tile save/restore could mutate.
Reopen only with a mutable VDP/plane-A renderer or a pixel-oracle pause route.

Per-unit audit of the Kid Chameleon source-port game shell: pause,
death/game-over/continue, the level clock, the HUD, the title/attract/demo loop,
and the 2P state bank. The recurring failure pattern across the codebase is that
ASM-faithful pure helpers exist and are unit-tested but carry
`#[cfg_attr(not(test), allow(dead_code))]` and are never wired into the live
frame step, so the runtime falls back on an invented substitute. The live pause
decision path is now a fixed candidate: Restart-Round / Give-Up cursor movement,
life cost, level restart, and last-life forced game-over are wired. The
remaining pause gap is visual/VDP pause-render wiring.

## Coverage summary

| Unit | Coverage | Confirmed gaps |
|---|---|---|
| pause-menu-restart-giveup | stub | 1 |
| pause-entry-allow-pausing-gate | full | 2 |
| pause-render-darken-tilesave | full | 1 |
| pause-exit-lastlife-death | stub | 2 |
| death-gameover-continue-lifecycle | partial | 4 |
| 2p-death-handoff-and-state-bank | partial | 1 |
| level-timer-decrement-and-freezes | full | 0 |
| hud-timer-and-lives-pip-display | partial | 1 |
| level-intro-namecard-clock-start | partial | 1 |
| title-attract-rotation-demo-handoff | full | 0 |
| demo-input-stream-no-end-guard | full | 0 |
| hud-score-diamond-update-timing | full | 1 |

Totals: 12 units audited, 5 full (3 with no gaps), 5 partial, 2 stub. 15
confirmed gaps (4 high, 7 medium, 4 low) after merging the two duplicate pause
last-life-game-over entries across `pause-menu-restart-giveup` and
`pause-exit-lastlife-death` into one, and including two options-menu
gameplay-effect gaps found post-audit by the completeness critic (each confirmed
by an independent refuter). 0 of the high/medium claims were refuted and dropped.

## Confirmed gaps

### High severity

**RESOLVED candidate: Pause-menu Restart-Round / Give-Up decision path is wired live** (`Game_Paused_Loop` / `loc_6B0E` / `Game_Paused_ChkStart` / `loc_6B62` / `loc_6B68` / `loc_6C0A` / `loc_6C3E`)
The ROM pause loop (`Game_Paused_Loop`, kid.asm:6829) reads the joypad each paused frame and runs an interactive two-option menu via `Pause_Option`: Up/Down (`loc_6B0E`) moves the cursor between Continue (option 0) and Restart-Round / Give-Up (option 1). On Start (`Game_Paused_ChkStart`, kid.asm:6859), option 0 continues, while option 1 spends one life and either restarts the current level or enters the last-life `loc_6C0A` game-over path. Native now runs the same decision path in `apply_pause_menu_frame_loc_6ae0`: fresh Up/Down edges update `pause_menu_option`, Continue clears `Game_Paused`, Restart Round calls `boot_to_level`, decrements lives, clears `Extra_hitpoint_slots` and helmet state, and restores The Kid's base HP, while last-life Give Up calls the `loc_6C0A` helper and enters `DeathWindup`. Kind: fixed-candidate. Verified by `live_pause_menu_restart_round_spends_life_and_restarts_loc_6b46`, `live_pause_menu_give_up_last_life_enters_death_windup_loc_6c0a`, and the pure `loc_6B0E`/`Game_Paused_ChkStart`/`loc_6C0A` tests. Human live validation is still tracked in backlog #293 because the oracle paused-loop lane is not yet comparable. `crates/wildside-core/src/lib.rs`, `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/tests.rs`, `crates/wildside-core/src/tests/boot_tests.rs`.

**RESOLVED candidate: level timer seed now includes Init_SpriteAttr_HUD's `addq #1` pre-decrement second** (`Init_Timer_and_Bonus_Flags` / `Init_SpriteAttr_HUD` / `Decrease_Time_Left` / `loc_C0CC`)
During level load the ASM seeds the timer in two routines. `Init_Timer_and_Bonus_Flags` (kid.asm:6551-6564) sets `Time_Minutes=3` and clears both seconds digits and `Time_Frames` (display 3:00); then `Init_SpriteAttr_HUD` (sub_14244, kid.asm:27087-27101, called at kid.asm:24124 during title-card graphics setup before gameplay is interactive) does `addq.w #1,(Time_Seconds_low_digit)` and `move.w #1,(Time_SubSeconds)`, so the seeded display at gameplay start is 3:01 (181 displayed seconds). Native now models that with `LEVEL_TIMER_START_SECONDS=181` and `sub_frames=1`, while the separate `LEVEL_TIMER_COMPLETION_BASE_SECONDS=180` keeps `loc_C0CC`'s `addi #$B4` completion-time formula on the ROM base. Focused regressions cover the 3:01 seed, the first-tick 3:01 to 3:00 normalisation, the later 60-frame cadence, and the full countdown expiry frame. Human playtest should still validate the visible HUD/timer-death cadence. Kind: fixed-candidate. `crates/wildside-core/src/types.rs`, `crates/wildside-core/src/constants.rs`, `crates/wildside-core/src/combat.rs`, `crates/wildside-core/src/trace.rs`.

### Medium severity

**RESOLVED candidate: pause entry now consults Allow_Pausing through the level fade-in window** (`loc_74C8` / `loc_74B0` / `Teleport` / `sub_73D0` / `loc_7420`)
ASM `loc_74C8` (kid.asm:7631-7637) gates pause entry on a two-stage check: A-not-held AND `tst.b (Allow_Pausing).w; beq.w loc_7452`. `Allow_Pausing` ($FFFFFB56, variables.asm:122) is BSS-zero at boot, set (`$FF`) only when the per-level fade-in worker `sub_73D0` finishes (`loc_7420`, kid.asm:7561), and cleared (`$00`) on every `Teleport` (kid.asm:14773), the shared tail for death (lose_life), telepad warp, level-skip-cheat warp, and level-clear advance. Native now carries live `allow_pausing` state, clears it across level-entry/death/warp/skip transitions, sets it after the seventeenth InGame fade refresh, and blocks `handle_start_press` while it is clear. Regression coverage asserts Start is ignored before `Allow_Pausing` and only pauses after the `sub_73D0` completion window. Kind: fixed-candidate. `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/lib.rs`, `crates/wildside-core/src/level.rs`, `crates/wildside-core/src/tests/combat_tests.rs`.

**RESOLVED candidate: CONTINUE? screen consumes a continue only on Start, matching `Ctrl_Held` bit 7** (`loc_D350` / `loc_D3AA` / `Use_Continue` / `Continue_Screen`)
In both ROM continue-countdown wait loops the only input that grabs a continue is Start: `loc_D350` (kid.asm:16530-16531) and `loc_D3AA` (kid.asm:16557-16558) each do `tst.b (Ctrl_Held).w; bmi.s Use_Continue`. `bmi` tests bit 7 of the SACBRLDU button byte (variables.asm:30), i.e. Start only. Native gates the live `ContinueCountdown` use path on `input.start` and has a regression asserting A/B/C alone leave the countdown running. Kind: fixed-candidate. `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/tests/movement_tests.rs`.

**RESOLVED candidate: Use_Continue clears extra_hitpoint_slots** (`Use_Continue`)
ASM `Use_Continue` (kid.asm:16563-16575) resets the run: `Number_Lives=3`, `Current_Helmet=0`, `Number_Diamonds=0`, `Number_Hitpoints=2`, and `clr.w (Extra_hitpoint_slots).w` (kid.asm:16572), then branches directly to `Teleport`, skipping the `lose_life` body, so this is its own clear. Native now clears `progression.extra_hitpoint_slots` in the live `ContinueCountdown` Start path before respawn reset, and `use_continue_clears_extra_hitpoint_slots_loc_d3c4` covers a previously earned Diamond-Power extra-HP slot. Kind: fixed-candidate. `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/tests/movement_tests.rs`.

**RESOLVED candidate: continue countdown includes the `loc_D3AA` second Start window** (`loc_D350` / `loc_D3AA` / `Continue_Screen` / `loc_D3F0`)
The ROM continue screen has two countdown loops that both poll Start for `Use_Continue`: `loc_D350` runs `$123`(291) frames, then (after kicking the text/number objects into the falling fade at `loc_D372`/`loc_D386`) `loc_D3AA` runs another `$EF`(239) frames, each `tst.b (Ctrl_Held); bmi.s Use_Continue`; only after `loc_D3AA` expires does `loc_D3F0` return to title (kid.asm:16526-16560). Native now includes the `$1C` initial hold, `$123` first window, and `$EF` second window in `CONTINUE_COUNTDOWN_FRAMES`, preserving the Start poll throughout the lifecycle. Regressions assert the old timeout boundary remains in CONTINUE? and Start during the second window spends the continue. Kind: fixed-candidate. `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/constants.rs`, `crates/wildside-core/src/tests/movement_tests.rs`.

**RESOLVED candidate: per-player state bank preserves Extra_hitpoint_slots** (`sub_8A4` / `loc_8B4` / `loc_8BA` / `sub_8C2` / `loc_8D2` / `loc_8D8`)
ASM `sub_8A4` (load) and `sub_8C2` (save) copy an 11-word per-player bank (`moveq #$A,d0`, kid.asm:973) between the working region at `Number_Lives` ($FC3E) and `Player_X_Lives` ($FC54 / $FC6A). Word index 6 of that bank is `Extra_hitpoint_slots` ($FC4A working, $FC60/$FC76 per-player). Native `PlayerSlot` now includes `extra_hitpoint_slots`, `save_player_slot`/`load_player_slot` copy it, and `loc_b8ba_two_player_switch_from_p2_wraps_back_to_p1_slot` covers the live 2P handoff by saving P2's value and restoring P1's value. Kind: fixed-candidate. `crates/wildside-core/src/types.rs`, `crates/wildside-core/src/level.rs`, `crates/wildside-core/src/tests/helmet_tests.rs`, `crates/wildside-core/src/tests/level_tests.rs`.

**RESOLVED candidate: options control-scheme remap applies before live input latching** (`ReadJoypad` / `Permute_ABCButtons` / `sub_CCE` / `loc_CE8` / `loc_BB6`)
The ROM calls `Permute_ABCButtons` inside `ReadJoypad` immediately after each `Joypad_ReadFromHardware` and before storing the result (kid.asm:1361-1368), so the remap rewrites the bytes downstream gameplay consumes. Native now mirrors that boundary in `readjoypad_input_with_options_loc_bb6`: demo-mode input returns raw, otherwise `with_abc_buttons_permuted(options_controls_scheme)` runs before `loc_C4E`-style held/pressed latching. Kind: fixed-candidate. Verified by `readjoypad_options_scheme_2_swaps_a_b_before_latch_loc_bb6` plus the per-scheme `Permute_ABCButtons` helper tests. `crates/wildside-core/src/lib.rs`, `crates/wildside-core/src/types.rs`, `crates/wildside-core/src/tests.rs`, `crates/wildside-core/src/tests/movement_tests.rs`.

**RESOLVED candidate: options Fast-Action Speed inversion applies in live gameplay input** (`loc_749C` / `loc_74B0`)
Every player-update frame the ROM tests `Options_Suboption_Speed` ($FFFFFDC9; Normal=0, Fast=-1 per variables.asm:205) and, when Fast Action is selected, executes `not.b (Ctrl_A_Held).w` (kid.asm:7619-7621, demo-mode skipped at 7617-7618), inverting the held run latch so the run modifier becomes hold-to-walk / tap-to-run. Native now consumes `options_speed_button_fast` in `gameplay_input`, after left/right conflict resolution and before movement sees the A latch; demo play skips `gameplay_input`, matching the ROM's demo-mode gate. Kind: fixed-candidate. Verified by `gameplay_input_fast_action_inverts_a_latch_loc_749c` and the `loc_749c_*` helper tests. `crates/wildside-core/src/lib.rs`, `crates/wildside-core/src/types.rs`, `crates/wildside-core/src/tests.rs`, `crates/wildside-core/src/tests/combat_tests.rs`.

### Low severity

**Pause-render cluster still has VDP/tile gaps after palette/menu wiring** (`sub_6874` / `loc_687C` / `loc_6894` / `loc_68AC` / `loc_690C` / `loc_693A` / `loc_695C` / `loc_69AE` / `loc_6B68` / `loc_6BA4` / `Pause_DarkenPalette`)
ASM `sub_6874`/`loc_687C` runs every paused frame: darken the palette, upload it, hide sprites whose tile index falls in `$676..$68E`, DMA the pause-menu art to VRAM `$CEC0`, and save the five-row plane-A tile region under the menu into `Decompression_Buffer` (restored at `loc_6B68` on unpause) with plane-edge wrap handling. Native has ASM-faithful, unit-tested pure helpers for every step (`pause_menu_gate_loc_687C`, `hide_pause_menu_overwritten_sprites_sub_6874`, `pause_menu_art_total_bytes_loc_690C`, `pause_menu_tile_save_plan_loc_695C`, `pause_menu_tile_restore_plan_loc_6B68`, `pause_darken_palette_loc_6C70`). The native-player renderer now removes the invented centred "PAUSED / Press Start to resume" panel, draws the ROM `MapUnc_PauseMenu_Restart` / `MapUnc_PauseMenu_GiveUp` 16x5 tile block at `loc_69D2`'s screen offsets from the loaded 25 `ArtUnc_PauseMenu` tiles, places the live `Pause_Option` cursor row from `Game_Paused_Loop loc_6B0E`, applies an ASM-cited final-RGB pause darken pass matching `Pause_DarkenPalette`'s per-channel halving after native has resolved CRAM colours, and filters paused sprite commands through `sub_6874`/`loc_68AC`'s masked `$676..=$68E` first-pattern-tile hide rule. Remaining gaps: VDP palette upload/state capture and tile save/restore-on-unpause are still not wired into production rendering. All consequences here are visual/VDP only with no motion/damage/lifecycle effect, hence low; the interactive menu-selection path is the separate high-severity gap above. Kind: partial-helper-wired. Verified by `paused_scene_hides_sprites_using_pause_menu_vram_tiles_sub_6874`, `load_pause_menu_visuals_returns_loc_690c_vram_sheet`, `paused_scene_draws_pause_menu_tilemap_loc_69d2`, `paused_scene_draws_give_up_tilemap_on_last_life_loc_69d2`, `pause_menu_uses_rom_positioned_restart_text_loc_69d2`, and `paused_ingame_scene_requests_palette_darken_loc_6c70`. `crates/wildside-core/src/boot.rs:1107`, `crates/wildside-core/src/boot.rs:1126`, `crates/wildside-core/src/boot.rs:1158`, `crates/wildside-core/src/boot.rs:1335`, `crates/wildside-core/src/boot.rs:1538`, `crates/wildside-core/src/boot.rs:1901`, `crates/wildside-render/src/bundle.rs`, `crates/wildside-cli/src/native_player/scene.rs`, `crates/wildside-cli/src/native_player/render.rs`.

**RESOLVED candidate: live engine state now mirrors the Allow_Pausing set-on-fade-complete behaviour** (`sub_73D0` / `loc_7420`)
The old pure `FullscreenPaletteFadeSub73D0` helper still exists for direct ASM coverage, but the live runtime now has a typed `allow_pausing` flag and a small frame counter that mirrors the `sub_73D0` `$10`-per-refresh threshold before `loc_7420` sets `Allow_Pausing`. This wires the previously helper-only behaviour into the pause gate. Kind: fixed-candidate. `crates/wildside-core/src/lib.rs`, `crates/wildside-core/src/tests/combat_tests.rs`.

**RESOLVED candidate: no-continue GAME OVER hold includes the `$3C` pre-hold hibernate** (`loc_D25E` / `loc_D2CE`)
On the no-continues GAME OVER path the ROM hibernates `$3C`(60) frames, the GAME/OVER text fall-in, before entering the `$1A3` Start-skippable hold (`loc_D2CE`); see kid.asm:16491-16499. Native now splits those ASM literals as `PLAYER_GAME_OVER_PRE_HIBERNATE_FRAMES` and `PLAYER_GAME_OVER_START_SKIP_HOLD_FRAMES`, then collapses both into `PLAYER_GAME_OVER_HOLD_FRAMES`, so the automatic title return preserves the pre-hibernate delay while keeping the existing held-Start skip. Kind: fixed-candidate. Verified by `loc_d2ce_zero_continue_game_over_hold_includes_pre_hibernate`. `crates/wildside-core/src/constants.rs`, `crates/wildside-core/src/tests/movement_tests.rs`.

**RESOLVED candidate: low-time Kid-tracking timer relocation wired into live HUD** (`End_Decrease_Time_Left` / `sub_BD0A` / `loc_BD40` / `loc_BD38` / `unk_BD8A`)
The live HUD renderer now routes timer placement through the ASM-backed `hud_timer_low_time_check_end_decrease_time_left` and `hud_timer_sprite_layout_sub_BD0A` helpers. For normal time, `sub_BD0A d7=0` keeps the fixed `unk_BD8E` row. When minutes==0 and seconds-tens<=2 (<=0:29), the renderer follows `End_Decrease_Time_Left` -> `sub_BD0A d7=1` -> `loc_BD40`: minutes/colon move to x=1 and the two seconds digits track Kid's screen position with the `unk_BD8A` offsets -7/+1. Level-complete frozen-HUD frames keep the fixed seeded row, matching `loc_BD38`'s timer-frozen early return. Kind: fixed-candidate. Verified by `game_hud_low_time_tracks_kid_with_loc_bd40_layout`. `crates/wildside-core/src/boot.rs`, `crates/wildside-cli/src/native_player/hud.rs`, `crates/wildside-cli/src/native_player/mod.rs`.

**RESOLVED candidate: lives HUD path applies the ROM 99-life display clamp live** (`loc_BF2A` / `life_display`)
The live level-frame path now calls the ASM-backed `hud_lives_clamp_loc_BF2A` helper before emitting each trace frame, so `Number_Lives>=100` is rewritten to 99 at the same HUD boundary as the ROM. Extra-life award sites remain bare increments, matching the original; the cap belongs to `loc_BF2A`, not to the award routines. Kind: fixed-candidate. Verified by `loc_BF2A_clamps_100_lives_to_99` and the live HUD clamp assertion in `score_life_award_uses_100k_quotient_delta_loc_10aae`. `crates/wildside-core/src/lib.rs`, `crates/wildside-core/src/boot.rs`, `crates/wildside-core/src/tests/boot_tests.rs`, `crates/wildside-core/src/tests/level_tests.rs`.

## Already tracked

- **Pause-menu Restart-Round / Give-Up + last-life game-over fixed candidate** (`Game_Paused_Loop` / `loc_6B68` / `loc_6C0A`): backlog `#293` carries the remaining human validation state; `#305` records why an oracle pause-menu lane is blocked for now. The visual pause-render cluster remains separate and open below.

## Backlog candidates

| # | Status | Pri | System | Summary | Verify |
|---|---|---|---|---|---|
| NEW | [?] | P1 | shell | Fixed candidate: Pause-menu Restart-Round / Give-Up (incl. last-life game-over) now runs `Game_Paused_Loop`/`Game_Paused_ChkStart`/`loc_6C0A` in the live pause step; human live validation remains | `live_pause_menu_restart_round_spends_life_and_restarts_loc_6b46`; `live_pause_menu_give_up_last_life_enters_death_windup_loc_6c0a` |
| NEW | [x] | P1 | level | Level clock seed now includes `Init_SpriteAttr_HUD`'s `addq #1` low-digit pre-decrement second, so fresh gameplay starts from the ROM 3:01 state | `level_timer_starts_with_hud_normalised_extra_second`; `level_timer_first_frame_normalises_to_three_minutes`; `level_timer_expires_after_full_countdown` |
| NEW | [x] | P2 | shell | Pause entry honours `Allow_Pausing` during the level fade-in / telepad / level-skip warp windows the ROM forbids | `loc_74c8_start_before_allow_pausing_is_ignored`; `sub_73d0_allows_pause_after_seventeenth_ingame_refresh` |
| NEW | [x] | P2 | score | CONTINUE? screen spends a continue only on Start; A/B/C continue counting down | `loc_d350_abc_do_not_use_continue` |
| NEW | [x] | P2 | score | `Use_Continue` clears `extra_hitpoint_slots` before reseeding base HP | `use_continue_clears_extra_hitpoint_slots_loc_d3c4` |
| NEW | [x] | P2 | score | Continue countdown includes the `loc_D3AA` second Start window / 239-frame tail before title return | `loc_d3aa_second_start_window_extends_continue_timeout`; `loc_d3aa_start_uses_continue_during_second_window` |
| NEW | [x] | P3 | level | 2P per-player bank preserves `Extra_hitpoint_slots` through `PlayerSlot` save/load | `loc_b8ba_two_player_switch_from_p2_wraps_back_to_p1_slot` |
| NEW | [b] | P3 | shell | Pause-render cluster is exhausted without a mutable VDP plane: native-player now draws the ROM `MapUnc_PauseMenu_Restart` / `MapUnc_PauseMenu_GiveUp` 16x5 tile block from `loc_69D2` using the 25 `ArtUnc_PauseMenu` tiles at VRAM base `$676`, live `Pause_Option` cursor row from `Game_Paused_Loop loc_6B0E`, final-RGB pause darkening from `Pause_DarkenPalette loc_6C70`, and `sub_6874`/`loc_68AC` sprite-hide for first pattern tiles in `$676..=$68E`; remaining VDP palette upload/state capture and tile save/restore need a persistent plane-A model or pixel-oracle route | Pause; assert the palette darkens, the menu overlay draws, cursor movement is visible, covered-range sprites hide, and the background restores cleanly on unpause |
| NEW | [x] | P3 | shell | No-continue GAME OVER hold now includes the `$3C` pre-hibernate before the `$1A3` Start-skippable wait | `loc_d2ce_zero_continue_game_over_hold_includes_pre_hibernate` |
| NEW | [x] | P2 | shell | Options control schemes 1-5 now apply `with_abc_buttons_permuted` (`Permute_ABCButtons`/sub_CCE) before live input edge detection, demo-mode skipped | `readjoypad_options_scheme_2_swaps_a_b_before_latch_loc_bb6` |
| NEW | [x] | P2 | shell | Options Fast-Action now applies `with_speed_option_a_inverted` (`loc_749C` `not.b Ctrl_A_Held`) each live player-update frame, demo-mode skipped | `gameplay_input_fast_action_inverts_a_latch_loc_749c` |
| NEW | [x] | P3 | shell | Low-time (<=0:29) timer now relocates to Kid's head through the live `sub_BD0A` / `loc_BD40` layout | `game_hud_low_time_tracks_kid_with_loc_bd40_layout` |

## Not audited

Scout exclusions and units resolved with no gap:

- **level-timer-decrement-and-freezes**: audited full; no gap. The premise (timer running during transforms / FB4B freezes) does not hold: the live timer is suppressed in all windows the ASM freezes it via lifecycle/transform early-returns (step_lifecycle returns false for Dying/LevelComplete/GameOver, so refresh_opening_slice early-returns; `gameplay_hibernating_for_transform()` early-returns during form-change flash) rather than mirrored flag bytes. The unconditional tick is `combat.rs:10908`; the early-return at lib.rs:2920 prevents over-ticking.
- **title-attract-rotation-demo-handoff**: audited full; no gap. The in-scope idle/demo/title cycle and auto-timeout-vs-manual-Start handoff (mode-clear vs paused-title) are faithfully modelled; per-frame oracle constants were explicitly out of scope.
- **demo-input-stream-no-end-guard**: audited full; no gap. The ORIGINAL_QUIRK (six demo-1 overflow bytes read past `Demo_InputData1` into the following art blob before the timeout return) is faithfully reproduced on a live path; the six overflow bytes `[0x06,0xC4,0xC7,0xFF,0x9A,0x14]` were ROM-verified at offset 0x1E6E3, and the timeout edge consumes indices {498..503} only.
- **2P death-handoff survivor dispatch** (`loc_B894` / `loc_B8D0`): native's Dying handler takes `number_lives==0 -> GameOver` before `try_switch_player`, so in 2P a player exhausting their lives ends the game even if the other survives; explicitly marked not-implemented in code (combat.rs:9893) and recorded as a known limitation in `combat-fidelity-audit.md:124`. Not re-reported.
- **`lose_life` `$FC4A` clear on death**: already filed at `level-loading-fidelity-audit.md:203` (cross-referenced from `combat-fidelity-audit.md:123`); NB its "permanently 0 / inert" premise is now stale because the Diamond-Power ExtraHitPoint live path assigns the field (helmets.rs:4805-4806), so the death-clear gap is no longer inert and arguably deserves a severity bump.
- **OneTimePrize collected-table helpers** (`loc_10856`/`1086A`/`10876`/`10886`/`1089A`): pure, test-only, no live caller and no live storage; fully filed at `blocks-fidelity-audit.md:48-49,118,134`. NB `Player_X_OneTimePrizes` lives at $FC84/$FD26, outside the `sub_8A4`/`sub_8C2` 11-word bank, so the bank correctly does not carry it.
- **HUD redraw-on-change latches and ordering** (`life_display`/`diamond_display` cmp vs `*_prev`): pure sprite-tile-buffer redraw optimisations; native exports computed HUD numbers directly in trace with no sprite tile buffer, so the latch and its ordering have no observable consequence.
- **score=0 at game-over** (`loc_D17E`): ASM copies `Score` into the EOL number object then `clr.l (Score)` post-fade; native sets `progression.score=0` at GameOver entry as a deliberate collapse of the unmodelled EOL number object (boot.rs:7647-7651), and post-game-over score is advisory. Not a gap.

Units with ambiguity needing oracle probes:

- **Pause-entry fade-in window frame bound**: the existence of a nonzero pause-forbidden fade-in window is unambiguous from the ASM ordering (`loc_7452` pause-handler object allocated alongside the `sub_73D0` fade worker before `Allow_Pausing` is ever set), but a numeric frame-exact bound requires tracing `sub_73D0`'s per-frame scheduling.
- **Demo abort suppression during Kid_Transform**: ASM `loc_119D8` keeps polling Start via the `j_sub_8E0` hibernate loops, but native suppresses based on oracle-observed behaviour (Start at frame 35146 does not abort through 35148); cannot be proven wrong from the ASM alone. Likewise the Allow_Pausing interaction on manual demo abort is a calibrated `frozen_counts!=(12,12)` heuristic with a self-documented TODO(fidelity), not an undisclosed divergence.
- **`$FFFFFB4B` timer-frozen flag**: no symbol in `variables.asm` (one byte after `Background_NoScrollFlag` $FFFFFB4A); its meaning was derived from its two `st` setters (flagpole touch, game-over) and is consistent with the native "timer-frozen at level end" lifecycle interpretation, but unconfirmed by a named symbol.
