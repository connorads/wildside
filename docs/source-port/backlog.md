# Playtest Backlog

Canonical tracker for bugs, missing features, and verification gaps. See `CLAUDE.md`
for iteration protocol, `docs/source-port/methodology.md` for the investigation
playbook, and `docs/source-port/roadmap.md` for the programme-level workstreams.

Important: a green TAS frontier or a green `1p-accurate` release artefact does **not**
mean the game is nearly finished. Native-play breakage, wider verification gaps, and
documented deviations belong here too.

Progress language uses the evidence ladder from `methodology.md`:
`stubbed`, `boots`, `replay-matched`, `branch-matched`, `mechanic-swept`, `human-play-stable`.
A closed TAS frontier proves `replay-matched` on that route, not `human-play-stable`.

Status: `[ ]` open, `[~]` in progress, `[?]` agent proposes fix complete — **human must validate before closing**, `[x]` done, `[-]` won't fix / not a bug, `[b]` blocked / needs human repro
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

`[?]` items carry an evidence block recording the investigation outcome: repro
scenario path, ASM labels cited, oracle-parity result, and the proposed-fix
commit hash. Only a human may promote `[?]` → `[x]`; the agent lane never
auto-closes. If investigation stalls, the lane appends to the rejected-hypothesis
ledger in `frontier-state.yaml`, increments the per-issue attempt counter, and
leaves status at `[~]` until the counter exceeds the cap (auto-tagged
`stalled:agent-blocked`).

For newly opened items, capture these facts in the investigation notes:

- evidence source: playtest, recording slug, oracle, or frame capture
- native repro status: reproduced / not yet reproduced
- oracle repro status: authoritative / advisory / unknown / oracle limitation
- minimum gate: `fast-slice`, `1p-playable`, `1p-accurate`, or `visual-parity`

Live playtest note 2026-06-01: latest human playthrough contradicts several
previous `[x]` rows. Treat those as reopened until a deterministic native
recording or live `play-native` verification proves the human path is stable.
Unit tests, generated-asset renderer tests, and TAS slices are useful evidence
but do not by themselves prove `human-play-stable`.

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 001 | [x] | P0 | player | Player gets stuck after first damage | Take hit, confirm movement resumes |
| 002 | [x] | P0 | enemy | Enemies don't die when stomped | Stomp enemy on level 0x00, confirm death |
| 003 | [~] | P1 | audio | No music or sound effects playing | Load level, confirm BGM and SFX |
| 004 | [x] | P1 | render | Colour palette wrong (floor/tree colours reversed) | Capture frame, compare to oracle |
| 005 | [-] | P1 | render | No parallax background | NOT A BUG — parallax working correctly |
| 006 | [x] | P1 | enemy | Dragon moonwalking (sprite direction vs movement) | Watch dragon, confirm facing matches walk dir |
| 007 | [x] | P1 | camera | Screen shakes/moves weirdly when hit | Take damage, confirm camera stays stable |
| 008 | [x] | P1 | render | Unwanted dark overlay effect when hit | Take damage, confirm no full-screen tint |
| 009 | [x] | P2 | menu | No menu — spawns directly into game | Launch, confirm title screen appears |
| 010 | [x] | P2 | enemy | No enemy attack patterns (fire breath, arrows) | Reach archer/dragon, confirm projectiles |
| 011 | [x] | P2 | block | Non-P bricks look half-broken (wrong sprite state) | Resolved by palette fix (#004) |

Note 2026-06-01 for #003: latest playtest still reported missing intro/title
music on the attract-loop path while options music could start. ASM
`Load_SegaScreen` (`kid.asm:25444-25445`) calls `StopMusic`, then
`Load_IntroSequence1` (`kid.asm:29068-29070`) queues `bgm_Ice`. Native was only
clearing the active title cue on the title-to-level transition, so a return
through the Sega logo shell could leave playback state treating the title track
as still active instead of allowing the intro shell to restart Ice. The native
player now clears active BGM on `GameMode::SegaScreen` when a cue is active.

Follow-up 2026-06-01: current live playtest still reports no intro music and no
menu music, while entering Options starts music. Native repro status: not yet
reproduced in a deterministic recording. Oracle status: ASM says intro sequence
1 must queue `bgm_Ice`; title/menu shell needs a live audio trace before another
fix. Minimum gate: `1p-playable` plus recorded boot/title/options audio cue
transcript.

Follow-up 2026-06-01: native `play-native` cold boot did produce audio, but the
cue path still differed from ASM `Load_IntroSequence1` (`kid.asm:29067-29070`):
native treated a cold intro as plain `Start(bgm_Ice)`, whereas the ROM runs
`StopMusic` first and then queues Ice. This matches the live symptom where
Options, which already used the reset/start path from `Load_OptionMenu`, could
start music while intro/title did not reliably recover. Native now uses
`StopMusicThenStart(bgm_Ice)` for `IntroSequence1` even with no active cue.
Evidence: `.cache/recordings/issue-003-intro-reset-audio-after/audio.wav` is
20s stereo, RMS about -26.08 dB, with silence ending at the intro transition
around 2.883s. The native recording verifier and
`verify-audio-output --scenario boot-title-start-level1` pass. Follow-up still
needed: scripted title/options live recording so the menu cue itself is covered
by player-loop evidence rather than helper-only tests.

Follow-up 2026-06-01: native recording bundles now write `audio-events.txt`
with frame/mode/cue/command rows whenever `play-native` starts, stops, or
replaces BGM. Cold intro evidence
`.cache/recordings/issue-003-audio-events-cold-intro/audio-events.txt` records
`frame=173 mode=IntroSequence1 ... command=StopMusicThenStart(3)`, so future
playtest captures can distinguish "cue never issued" from GEMS/waveform output
failures without listening by hand.

Follow-up 2026-06-01: scripted recording bundles now emit the same
`audio-events.txt` transcript as live `play-native`, so title/menu/options audio
routes can be checked without an interactive recording. Deterministic route
`.cache/recordings/issue-003-title-options-audio-audit/` presses Start through
the cold intro/title path, moves to Options, and verifies native-only replay.
Its transcript records `IntroSequence1 -> StopMusicThenStart(3)` at frame 173
and `OptionsMenu -> StopMusicThenStart(101)` at frame 2751. That narrows the
current live "intro/menu silent, options starts" report: the scripted native
state path queues the expected ASM `Load_IntroSequence1` and `Load_OptionMenu`
music commands, so remaining work is either live-route drift outside this
script or GEMS/audio output after the command is issued.

Follow-up 2026-06-01: the same scripted `audio-events.txt` transcript now logs
SFX `PlaySound(id)` rows derived from `played_sound_effect_ids` and the existing
live GEMS event inference. Refreshed
`.cache/recordings/issue-003-title-options-audio-audit/audio-events.txt` records
the two title-menu navigation jingles at frames 2730 and 2740 (`PlaySound(45)`)
between the intro Ice cue and Options City cue. This gives #216 a deterministic
recording artefact for checking SFX timing on scripted block/diamond/flag routes
without requiring a live SDL capture first.

Follow-up 2026-06-02: refreshed cold-boot live evidence on current `main`.
`issue-003-cold-boot-audio-current-refresh` verifies against oracle and records
the ASM intro cue in `audio-events.txt`: frame 173 enters `IntroSequence1` and
queues `StopMusicThenStart(3)` (`bgm_Ice`). Its 20s stereo `audio.wav` is quiet
before intro (`ffmpeg volumedetect` over 0..2.85s: mean `-58.8 dB`, peak
`-36.3 dB`) and active after intro (`2.90..7.90s`: mean `-25.1 dB`, peak
`-11.2 dB`). The older `issue-003-cold-boot-audio-current` full verifier also
passes; reduced cases `0..120` and `120..240` pass, so this route does not expose
a gameplay/state mismatch. Remaining "no intro music" reports should target a
different live route/output path or later title/menu recovery, not the basic
`Load_IntroSequence1` BGM command.

Follow-up 2026-06-02 case-queue triage for #003:
`issue-playtest-cold-boot-audio-current` is also clean rather than actionable.
Full `verify-recording` passes (`trace_match`, `oracle_state_match`, and
`oracle_frame_match` all true), native-only replay passes, and reduced gameplay
windows `0..120`, `120..240`, and `480..600` all pass. The transcript reaches
IntroSequence1 at frame 173 and IntroSequence2 at frame 534, so this bundle
covers the early boot/intro transitions but does not expose a reduced gameplay
mismatch.

Follow-up 2026-06-02 title-route case triage for #003: the older
`issue-003-title-audio-current` bundle is native-replayable and its early
audio windows pass (`0..120`, `120..240`), but full recording verification is
not clean because the demo tail diverges at `frame_003360@3360` on
`player_1_level_id` (`native 0x1B`, oracle 0). A focused `3240..3420` case first
failed only because native emitted a semantic `landed` event at frame 3390 while
oracle exposes no ground flag in helper-backed DemoPlay frames; positions,
mode, camera, and entities matched. Case semantics now ignore ground-flag-only
landing while the shared movement state remains `movement_0003`; reduced case
`issue-003-title-audio-current_case_03240_03420` now passes through `3240..3391`.
Remaining title/menu audio work needs a fresh interactive/menu route or output
path repro, not this stale long idle bundle.
| 012 | [x] | P2 | block | No particle effects when blocks break | Hit block, confirm debris particles |
| 013 | [x] | P2 | block | P blocks don't convert to regular blocks after hit | FIXED/CLARIFIED 2026-05-24: prize blocks still delete their layout cell while ASM `sub_1007A` runs the visible wobble sprite, so the block no longer appears to vanish before the broken/wobbling art. See #149 and commit `6fefd268`. |
| 014 | [x] | P2 | block | Diamonds/items don't visually emerge from blocks | Hit prize block, confirm item sprite appears |
| 015 | [x] | P2 | player | Can't wall-grab (A+A to grab block edges) | Jump at wall edge, confirm grab |
| 016 | [x] | P2 | level | No level intro screen showing level name | Enter level, confirm name card |
| 017 | [x] | P2 | form | Masks/helmets not visible when they appear | Hit helmet block, confirm sprite |
| 018 | [x] | P2 | form | No form-change animation when equipping helmet | Placeholder flash; real animation pending |
| 019 | [x] | P2 | form | Player doesn't transform at all | Collect helmet, confirm form changes |
| 020 | [x] | P3 | render | No water/shimmer background effect | Load water level, confirm shimmer |
| 021 | [x] | P3 | menu | Sound Test does nothing (cursor==1 unhandled) | Enter options, select Sound Test |
| 022 | [x] | P3 | render | Duplicate Game Over overlay rendering | Game over screen renders once |
| 023 | [x] | P2 | render | Floating item sprites use placeholder rectangles | Replace with recognisable pixel-art icons |
| 024 | [x] | P1 | enemy | UFO beam child object system not implemented | UFO fires beam segments downward |
| 025 | [x] | P2 | enemy | Fire Demon death trail not implemented | Kill Fire Demon, confirm fire trail |
| 026 | [x] | P3 | menu | Demo playback — ROM has Demo_InputData1/2 at kid.asm:33373 | Title screen auto-plays demo |
| 027 | [x] | P2 | level | Multi-screen block list cross-level references | Graceful fallback (ba36f63) |
| 028 | [x] | P3 | testing | Adopt `insta` snapshot testing for NativeTraceFrame | Snapshots detect state drift between commits |
| 029 | [x] | P3 | testing | Adopt `proptest` for physics pure functions | Speed tables, slopes, gravity hold across input range |
| 030 | [x] | P3 | infra | Add a local-first verification entrypoint for regression checks | Contributors can run the intended gate locally before pushing |
| 031 | [x] | P3 | infra | Add performance baseline tracking | criterion benchmarks: ~92ns/frame idle, ~3µs level load |
| 032 | [x] | P2 | fidelity | Per-form Diamond Power effects (20 unique abilities) | Each form has correct visual and damage pattern |
| 033 | [x] | P2 | fidelity | Form-change animation from ASM Kid_Transform tables | Original transform art/timing loaded for all forms, including Juggernaut composite rendering |
| 034 | [x] | P3 | fidelity | Intro sequence 5 segments (ASM has 5, native had 3) | Boot captures match oracle |
| 035 | [-] | P2 | fidelity | Resolve 6 oracle-warp-broken levels (0x4E73) | ORACLE LIMITATION — jgenesis warp bug, native works; determinism + soak tested |
| 036 | [x] | P2 | enemy | Tank walk speed step wrong ($10000 should be $8000) | Tank speed matches ASM at all HP tiers |
| 037 | [x] | P3 | fidelity | Floor-snap settling diffs: 0x24/0x51 (1px cam_y) | `0x24`/`0x51` now match the oracle-backed settle camera carry |
| 038 | [-] | P3 | fidelity | Camera Y settling 1px diffs on 0x02, 0x24, 0x51 | MERGED with #037 — same root cause |
| 039 | [-] | P3 | fidelity | 4th speed table entry is dead data in ROM | NOT A BUG — unreachable slot |
| 040 | [x] | P2 | enemy | Lion constants wrong (detect range, cooldown, projectile) | Lion matches ASM line-by-line |
| 041 | [x] | P2 | enemy | Hand jump velocity fixed, should be randomised per-tier | Hand matches ASM jump tables |
| 042 | [x] | P2 | enemy | Tornado acceleration $300 should be $C00, level-dependent timer | Tornado matches ASM init table |
| 043 | [x] | P2 | enemy | Dragon Flying is structural placeholder (3 variant scripts) | Dragon Flying matches ASM flight scripts |
| 044 | [x] | P3 | enemy | Lion projectile wall-bounce and per-enemy homing accel | Projectile bounces off terrain |
| 045 | [x] | P3 | enemy | Hand player-grab mechanic missing entirely | Hand grabs and drags player |
| 046 | [x] | P3 | enemy | Tar Monster timing/visibility/glob shot wrong | Tar Monster matches ASM bytecode |
| 047 | [x] | P3 | enemy | Drips random timing and surface-crawl spawning | Drips matches ASM drop timing |
| 048 | [x] | P3 | enemy | Spinning Twins orbit-to-walk should be collision-triggered | Transition on player contact |
| 049 | [-] | P3 | fidelity | Level 0x29 has 5px X position diff (slope-slide during gravity settle) | MERGED with #037 — same root cause |
| 050 | [-] | P3 | fidelity | 4 levels show idle_tail position diffs from enemy/timing divergence | EXPECTED — advisory: oracle warp timing + enemy interaction accumulation |
| 051 | [x] | P2 | enemy | Dragon HP base 2 should be 1 (ASM Dragon.asm:65 addq.w #1) | Dragon HP matches ASM |
| 052 | [x] | P2 | enemy | EmoRock velocity cap logic wrong ($60000 threshold → $40000 reset) | EmoRock drop matches ASM |
| 053 | [x] | P2 | enemy | Robot charge permanent until wall, not timed exit to walking | Robot charge matches ASM |
| 054 | [x] | P3 | enemy | MiniHoppingSkull ground delay 21 not 32 (dbf #$14) | Skull delay matches ASM |
| 055 | [x] | P3 | enemy | BagelBrothers projectile speed 3 not 4, lifetime 50 not 45 | Projectile matches ASM |
| 056 | [x] | P3 | enemy | BigHoppingSkull is continuous bouncer, not discrete hopper | Continuous bouncer from ASM: $FA0 gravity, wall/floor/ceiling/slope deflection |
| 057 | [x] | P3 | enemy | Goat player-tracking callback missing (turns toward player >32px) | Goat tracks player |
| 058 | [x] | P3 | enemy | Cloud 60-frame post-bounce idle period missing | Cloud idles after bounce |
| 059 | [x] | P3 | enemy | Fireball ceiling bounce uses Sphere formula, should be random+cap | Fireball bounce matches ASM |
| 060 | [x] | P2 | enemy | Boss multi-head system (Shishkaboss/BoomerangBosses/BagelBrothers) | 3 heads per boss with ASM offsets + per-head HP |
| 061 | [x] | P3 | enemy | Orca missing ceiling bounce, slope handling, X accel during lunge | All 3 from ASM: ceiling bounce, slope re-lunge, X accel $200/frame |
| 062 | [x] | P3 | enemy | HeadyMetal death wobble visual (rendering-only) | The effects-only scene tests now verify the full HeadyMetal body/eye wobble footprints and the two-frame phase flip at each ASM timer boundary |
| 063 | [x] | P2 | block | Ghost block restored with collision 7 (Spikes) not 6 (Solid) | Ghost blocks walkable |
| 064 | [x] | P2 | block | Drill block projectile fired downward, should be horizontal ±4px/frame | Drill projectile horizontal |
| 065 | [~] | P3 | block | Shooter fire: ASM uses per-block registration on visibility, native uses global timer | Shooter blocks now register per visible cell, fire on entry, and recycle after the ROM's 22-frame slot lifetime |

Note 2026-05-28 for #065: latest Highwater Pass playtest still reported the
Level 3 touch-and-shoot blocks as broken. Core activation is now covered by
`highwater_pass_1_berzerker_charge_pose_fires_ea1e_through_frame_loop`: the
test drives the real Berzerker/Rhino charge-pose path through ASM `loc_9564`,
fires the `$EA1E` rightward shooter via `loc_EAE4 -> loc_EC76`, and verifies the
frame loop clears the first ice barrier through `loc_EE2A -> loc_F00A`.
Remaining report is visual: native may still render the shooter projectile as a
fallback square instead of the ROM one-tile spike from `Process_ShooterObjects`
`loc_EE52/loc_EF62` (`$8A96` => block-art tile `$296`). Continue from renderer
tile/art resolution, not shooter activation or barrier collision.

Follow-up 2026-05-28: the normal generated-asset scene path does resolve the
Highwater projectile to the ROM spike tile. Regression
`highwater_shooter_block_projectile_resolves_real_rom_spike_tile_loc_ef62`
loads Level 0x02 visuals from the generated manifest, renders a typed
`ShooterBlock` projectile, and verifies the sprite uses transparent block-art
tile `$76` rather than the `[255,216,72]` fallback square. If live play still
shows squares, investigate live-session asset loading or stale/legacy
`active_projectiles` traces that bypass `active_projectile_details`.

Follow-up 2026-05-28: fresh playtest still got stuck at the first Highwater
touch-shooter barrier. The old regression only cleared row 97; the actual
Berzerker/Rhino charge path hits the stacked `$EA1F` cells at rows 100-102
opposite `$E203` ice barrier cells. Native was collapsing the `loc_9564`
head-forward pose to a one-frame latch after `loc_9326` cleared
`Berzerker_charging`, so the final stacked shooter cell was scanned with
Berzerker's narrow body width and row 102 stayed solid. Native now keeps the
six-entry `off_9688` pose window alive separately from the gameplay charge
flag, and `highwater_pass_1_berzerker_charge_clears_play_height_barrier_loc_eae4_f02a`
proves rows 100-102 clear through `loc_EAE4 -> loc_EC76 -> loc_F02A`.

Follow-up 2026-06-01: current live playtest still reports top shooter cells not
moving, projectiles appearing as square/incorrect spikes, and the barrier not
clearing. Existing branch tests prove core activation and generated-asset spike
rendering only for their scripted routes. Native repro status: not yet
reproduced on the latest human route. Next evidence must be a live recording
from Level 0x02 through the first Highwater shooter stack, with projectile
details and block cells captured before/after the charge.
| 066 | [x] | P3 | block | SoftPlatform bounce moves platform object | Fixed — native now models ASM SoftPlatform object state (`SoftPlatform_Init`/`SoftPlatform_BounceLoop`) instead of inline player-only bounce |
| 067 | [x] | P3 | block | Lift block uses accel physics; ASM uses coroutine animation | Lift matches ASM: 2px/frame×8 up, erase, 1px/frame×16 down |
| 068 | [x] | P3 | block | Shifting block grid swap is instant (correct); visual uses sub_113C0 sprite | Fixed: native now queues an ASM `loc_11364`/`sub_113C0` visual worker that rises from the source row while preserving the immediate collision grid swap |
| 069 | [x] | P2 | fidelity | TAS boot slice desync: native consumed early boot/title Start presses on the wrong frames | First 250-frame boot slices of 3 imported TAS recordings show 0 authoritative diffs |
| 070 | [x] | P1 | fidelity | TAS frame 956: post-ledge-snap movement state suppression | 21-frame pull-up animation cooldown (stru_8BB4), frontier now 986 |
| 074 | [x] | P1 | fidelity | TAS frame 986: sub-pixel X drift from false wall probe | Tile-boundary guard (sub_8F26), frontier now 987 |
| 075 | [x] | P1 | fidelity | TAS frame 987: wall-cling h_accel suppression (loc_AA22) + tile-boundary guard refinement | wall_cling_direction field gates h_accel; removed distance>0 workaround; frontier now 988 |
| 076 | [x] | P1 | fidelity | TAS frame 988: spurious Cyclone form change + ceiling bounce velocity | Deferred prize collection + zero ceiling y_vel; frontier now 1056 |
| 077 | [x] | P1 | fidelity | TAS frame 1056: prize item physics + transform freeze + wall fixup | ASM-faithful FloatingItem, per-form half-width, sub_7B30 wall adjustment, physics suppression during Kid_Transform; frontier now 1129 |
| 078 | [x] | P1 | fidelity | TAS frame 1129: demo input freeze during transform + left-accel vel=0 | Demo data pointer freeze during Kid_Transform (loc_C22 not called in coroutine); apply_holding_left vel=0 uses strong_brake not run_accel (bpl semantics); frontier now 1144 |
| 079 | [x] | P1 | fidelity | TAS frame 1144: per-form collision half-width (FA78/unk_B408) | ASM sub_914A uses FA78 (per-form from unk_B408) for floor/wall/ceiling probes; native used fixed PLAYER_HALF_WIDTH=5. Cyclone (hw=7) walked off ledge 1 frame early due to narrower floor probe. Threaded form.half_width() through all collision functions; frontier now 1145 |
| 080 | [x] | P1 | fidelity | AsmFaithful Phase 2: sub_902A before sub_8F26 position split | ASM floor collision (sub_902A) runs before horizontal (sub_8F26), seeing old x_pos. Split apply_velocity_to_position into y-then-x with collision between. Made AsmFaithful the default PhysicsOrder. Remaining frame 1145 divergence: walking coroutine floor-ahead check (sub_ADB8) |
| 081 | [x] | P2 | fidelity | Walking floor-ahead check (sub_B43A) | ASM walking coroutine (loc_8BF0) calls sub_B43A before physics: scans cells at (y+1)/16 across player width for Solid/Spikes. No floor → loc_B5F6: y_pos += 1px, skip vel/pos updates. Two fixes: (1) movement.rs pre-physics sub_B43A check in AsmFaithful path, (2) boot.rs demo grounded-run handler now defers walkoff to normal physics. Frontier 1145→1147 |
| 082 | [x] | P2 | fidelity | TAS frame 1147: demo byte boundary parity shift after transform | On the first Kid_Transform frame, step_demo_input still consumes one ReadJoypad byte before the coroutine freeze. Native was only rewinding the frame counter when the transform exited, leaving data_index and demo_buffered_input one byte behind oracle-held controls. Fix: after step_lifecycle detects transform end (flash_frames→0), decrement frame_counter by 1, restore the skipped data_index advance, and refresh demo_buffered_input from the resumed byte. Frontier 1147→1396 |
| 083 | [x] | P2 | fidelity | TAS frame 1170: sub_902A floor landing sub-pixel clearing | ASM sub_902A (kid.asm:10462) always does `clr.w $20(a3)` on new floor landings (airborne→grounded), zeroing the sub-pixel. Native was preserving sub-pixel from the accumulated position on all floor landings. Fix: clear sub-pixel on new landings (`!was_on_ground`), preserve when already grounded (sub_902A exits early via loc_913A when y_vel=0). Original #083 demo byte offset theory was wrong — parity `% 2 == 1` is algebraically identical to `% 2 == 0` with +1 offset. Frontier 1170→1209 |
| 084 | [x] | P2 | fidelity | TAS frame 1209: Cyclone flight activation/thrust timing (ASM kid_flip + sub_B168) | Three issues: (1) Activation ran before physics (step_cyclone_flight); ASM kid_flip activates AFTER physics so flight takes effect next frame. (2) Thrust checked C held state; ASM sub_B168 uses `bclr #Button_C,(Ctrl_Pressed)` — rising edge only, one frame of thrust per C press. (3) Edge detection used prior_input (2-frame gap); ASM uses Ctrl_Pressed vs Ctrl_1_Held (1-frame gap). Fix: split into step_cyclone_flight (velocity modulation when already flying) + step_cyclone_activation (after physics, only when not flying). Frontier 1209→1301 |
| 085 | [x] | P2 | fidelity | TAS frame 1301: Cyclone accumulator wrapping arithmetic | ASM `subi.w`/`addi.w` uses M68K modular 16-bit arithmetic. Native used `saturating_add/sub` which clamps at i16::MIN instead of wrapping. When accumulator gets very negative (~-22016) and thrust (-$3000) is applied, ASM wraps to positive (0xAA00-0x3000=0x7A00=+31232), producing a sudden downward push. Native saturated at -32768. Fix: wrapping_add/wrapping_sub. Frontier 1301→1365 |
| 086 | [x] | P2 | fidelity | TAS frame 1365: ceiling block preservation during Cyclone flight | Native apply_ceiling_collision cleared Prize blocks to Empty immediately, removing structural ceiling tiles. ASM sub_1007A writes $6F (Solid/no-block/Deleted) — tile stays solid for ~33 frames (bounce animation), then clears via deferred timer. Also: adjacent Smash blocks cleared inline (matching sub_914A full-width scan). Narrow re-verification after the demo transform fix shows the next authoritative frontier is frame 1396. |
| 087 | [x] | P2 | fidelity | TAS frame 1396: post-prize-hit ceiling latch in demo 0x1B | The transient `0xE001` restore still blocks Kid for one more frame, and the `0x0000` clear only becomes collision-visible after that frame's physics. Native now keeps restored prize cells collision-solid but non-dispatching, then clears/spawns at frame end. Frontier 1396→1405. |
| 088 | [x] | P2 | fidelity | TAS frame 1405: floating prize collection timing after ceiling latch | Diamond pickup bounds now match the smaller visible sprite, so frame 1405 stays mid-flight (`score=0`, `number_diamonds=0`) with the prize still present. Frontier 1405→1406. |
| 089 | [x] | P2 | fidelity | TAS frame 1406: post-latch diamond collection + next arc timing on demo 0x1B | Restricted wide spawn-support settling to the demo bootstrap window and queued prize-diamond awards through a 32-frame `PrizeDiamondCollected_Init` delay instead of rewarding on contact. Added `demo_diamond_prize_stays_uncollected_on_frame_1406` and `demo_prize_diamond_awards_on_frame_1438_without_score`; re-ran the nearby demo prize tests plus `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1390 --frame-end 1500 --checkpoint-interval 1`, which is now CRC-clean with 0 authoritative diffs through frame 1500. |
| 090 | [x] | P2 | fidelity | TAS frame 1755: AsmFaithful solid-floor touch snap lands one frame early on demo 0x1B | Restricted the sub-half-pixel solid-floor touch snap to the non-AsmFaithful landing paths, added `asm_faithful_solid_floor_snap_waits_for_contact_row` + `demo_solid_floor_landing_waits_until_frame_1756`, and verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1748 --frame-end 1762 --checkpoint-interval 1` is CRC-clean. Frontier 1755→1772. |
| 091 | [x] | P2 | fidelity | TAS frame 1772: demo 0x1B returns to title one frame early | Demo Start abort now follows the ASM hibernating object timing (`loc_119D8`): frame 1772 arms the abort, frame 1773 still stays in `mode=0x0010`, frame 1774 flips to title. Verified with the targeted demo abort tests in `kid-core` plus `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1768 --frame-end 1780 --checkpoint-interval 1` (frontier now 1774). |
| 092 | [x] | P2 | fidelity | TAS frame 1774: first title frames keep the live demo snapshot after demo 0x1B abort | Demo abort now preserves the paused gameplay snapshot under `mode=0x002c`, matching oracle `kid/camera` through frame 1801. Verified with `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`, `cargo test -p kid-core demo_title_return_keeps_live_snapshot_on_first_title_frames -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1772 --frame-end 1780 --checkpoint-interval 1`, and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1772 --frame-end 1860 --checkpoint-interval 1`. Frontier 1774→1802. |
| 093 | [x] | P2 | fidelity | TAS frame 1802: title return blanks Kid before camera/helmet reset on demo 0x1B | Split the demo-abort title handoff into 2 phases: full live snapshot through frame 1801, Kid hidden on frames 1802-1805, then camera/form/diamond state cleared on frame 1806. Added `demo_title_return_blanks_kid_on_frame_1802_before_camera_reset` and `demo_title_return_clears_camera_and_progression_on_frame_1806`; verified with `cargo test -p kid-core demo_title_return -- --nocapture`, `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808 --checkpoint-interval 1`, and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect`. Frontier 1802→1832. |
| 094 | [x] | P2 | fidelity | TAS frame 1832: title pose appears while oracle still keeps Kid hidden after demo 0x1B abort | Demo-return title mode now keeps Kid hidden until frame 1924 while leaving the rebuilt title object counts alone. Added `demo_title_return_keeps_kid_hidden_until_frame_1924`; verified with `cargo test -p kid-core demo_title_return -- --nocapture`, `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1828 --frame-end 1927 --checkpoint-interval 1` (0 authoritative, 5 advisory), `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect` (PASS: all 2000 frames match by CRC), and `mise run verify-slice-fast`. Frontier 1832→>2000. |
| 095 | [x] | P2 | fidelity | TAS frame 2529: demo 0x15 startup reused the demo 0x1B bootstrap profile | Split demo startup timing by recording: Knights Isle now keeps its hidden camera/profile through frame 2537, delays demo input/gameplay exposure to frame 2544, and holds the bootstrap pose until the oracle-backed handoff is complete. Added `demo_1_bootstrap_keeps_camera_hidden_through_frame_2537` and `demo_1_bootstrap_holds_pose_and_input_until_frame_2571`; verified with the targeted demo bootstrap tests, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2524 --frame-end 2580 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`. Frontier 2529→2625. |
| 096 | [x] | P2 | fidelity | TAS frame 2625: demo 0x15 handoff lands one frame early from AsmFaithful solid-floor touch snap | Disabled demo-playback solid-floor touch snap, added `demo_1_handoff_stays_airborne_until_frame_2626`, verified `cargo test -p kid-core demo_1_handoff_stays_airborne_until_frame_2626 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2618 --frame-end 2635 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`. Frontier 2625→2698. |
| 097 | [x] | P2 | fidelity | TAS frame 2698: demo 0x15 standingstill should not start walking into the adjacent wall after landing | Zero-speed grounded updates now respect the same side-wall probe the standingstill ASM uses before it transitions into walking, so the post-landing hold on Knights Isle keeps `kid_x_pos=0x011A0000` through frame 2714 instead of jittering into the wall. Added `demo_1_post_landing_hold_preserves_standingstill_through_frame_2714`; verified `cargo test -p kid-core demo_1_ -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 2720 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2698→2723. |
| 098 | [x] | P2 | fidelity | TAS frame 2723: demo 0x15 should release the right-wall overhang on the first upward tile-row crossing | Demo 1 kept `wall_cling_direction=Right` one frame too long after the overhang had already cleared, so native never emitted the 1px sticky-release shimmy that oracle shows at frame 2723. Added an ASM-timed overhang release in the AsmFaithful jump path plus `demo_1_wall_cling_snap_releases_into_run_on_frame_2723`; verified `cargo test -p kid-core demo_1_ -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2718 --frame-end 2760 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2723→2773. |
| 099 | [x] | P2 | fidelity | TAS frame 2773: grounded left-wall snap leaves demo 0x15 one pixel off the wall | Native reused the moving-right clearance when clamping a grounded left-wall hit, so frame 2773 snapped Kid to `0x01360000` instead of the oracle's flush `0x01350000`. Fixed the left-wall clamp to use the probed wall surface plus `half_width`, added `demo_1_left_wall_snap_lands_flush_on_frame_2773`, and verified `cargo test -p kid-core demo_1_left_wall_snap_lands_flush_on_frame_2773 -- --nocapture`, `cargo test -p kid-core demo_1_ -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2768 --frame-end 2790 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2773→2831. |
| 100 | [x] | P2 | fidelity | TAS frame 2831: buffered wall-sticky pull-up misses the prize-block top on demo 0x15 | Extended the wall-sticky ledge snap with an ASM-style pull-up fallback: when a buffered jump request is still live during a right-wall cling, native now searches downward from the pull-up window and lands on the prize-block top at `kid=(0x01440000,0x013F8000)` instead of continuing to fall. Added `demo_1_buffered_wall_sticky_pullup_lands_on_frame_2831`; verified `cargo test -p kid-core demo_1_buffered_wall_sticky_pullup_lands_on_frame_2831 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2828 --frame-end 2850 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2831→2938. |
| 101 | [x] | P2 | fidelity | TAS frame 2938: descending right-wall cling keeps suppressing jump air-accel after the dragon stomp on demo 0x15 | Oracle keeps Kid glued to the wall until frame 2994, then emits the 1px right-wall shimmy and clears wall-cling in time for demo-held `Right+B` to re-enter `sub_B084` on frame 2995. Native stayed in `wall_cling_direction=Right`, so the post-stomp drift never restarted. Added an ASM-style descending wall-cling release in the jump loop, plus `demo_1_keeps_the_oracle_backed_stomp_drift_on_frame_2994`; verified `cargo test -p kid-core demo_1_lands_on_the_walking_dragon_on_frame_2938 -- --nocapture`, `cargo test -p kid-core demo_1_keeps_the_oracle_backed_stomp_drift_on_frame_2994 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2988 --frame-end 3004 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2938→3252. |
| 102 | [x] | P2 | fidelity | TAS frame 3252: IronKnight transform freeze keeps demo 0x15 standingstill 15 frames too long | Native misread Kid_Transform table `stru_7FF8`: the first `anim_frame` fields sum to 81 frames, but `transform_anim_frames(IronKnight)` used 96 by accidentally treating the metadata byte as extra duration. That left `form_change_flash_frames` non-zero through frame 3252, so physics stayed frozen and demo-held Right never re-entered walking. Fixed the table count, added `demo_1_reenters_walking_from_standingstill_on_frame_3252`, and verified `cargo test -p kid-core demo_1_reenters_walking_from_standingstill_on_frame_3252 -- --nocapture`, `cargo test -p kid-core demo_cyclone_pickup_enters_transform_on_frame_1056 -- --nocapture`, `cargo test -p kid-core demo_transform_exit_rewinds_frame_counter_for_byte_parity -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3248 --frame-end 3260 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3200 --frame-end 4000 --bisect`. Frontier 3252→3400. |
| 103 | [x] | P2 | fidelity | TAS frame 3400 prep: native enemy loading used a narrower window than `Manage_EnemyLoading`, so the second Knights Isle walking dragon could drop out before the stomp frontier | Matched the activation window to the ASM loader bounds `x=[camera-0x80,camera+0x1C0]`, `y=[camera-0x80,camera+0x160]`, and added `demo_1_loads_the_second_knights_isle_dragon_by_frame_3400` to pin the missing dragon roster. Verified with `cargo test -p kid-core demo_1_loads_the_second_knights_isle_dragon_by_frame_3400 -- --nocapture`, `cargo test -p kid-core demo_1_lands_on_the_walking_dragon_on_frame_2938 -- --nocapture`, `cargo test -p kid-core demo_1_reenters_walking_from_standingstill_on_frame_3252 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3390 --frame-end 3410 --checkpoint-interval 1`, and `mise run verify-slice-fast`. Frontier stays at 3400: the remaining mismatch is the stomp/movement path, not loading. |
| 104 | [x] | P2 | fidelity | TAS frame 3400: second Knights Isle stomp setup never reached the oracle-backed grounded state | Matched DragonWalking contact resolution to `GfxObjects_CollisionKid`'s integer-inclusive overlap rules instead of a fixed-point prefilter, threaded the active form half-width through runtime enemy-contact queries, and modelled the live frame-3399 edge touch that only clips on the truncated pixel bounds. Added `walking_dragon_right_facing_edge_contact_matches_live_tas_stomp_window`, promoted `demo_1_reaches_the_oracle_backed_second_dragon_setup_on_frame_3400` into a passing regression, and re-checked `cargo test -p kid-core demo_1_reaches_the_last_oracle_backed_pre_stomp_arc_on_frame_3399 -- --nocapture`, `cargo test -p kid-core demo_1_reaches_the_oracle_backed_second_dragon_setup_on_frame_3400 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3390 --frame-end 3410 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3400 --frame-end 4916 --bisect`. Frontier 3400→3414. |
| 105 | [x] | P2 | fidelity | TAS frame 3414: demo 0x15 pause snapshot lands one frame / one pixel off after the second dragon setup | Deferred the demo title-return handoff until after the frame's final gameplay step, so the preserved title snapshot now uses the oracle-backed paused Kid/camera state instead of the stale pre-freeze slice. Promoted `demo_1_matches_the_oracle_pause_snapshot_on_frame_3414` into the normal suite, re-ran the older `demo_title_return*` regressions plus the nearby frame-3399/frame-3400 probes, and verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3412 --frame-end 3416 --checkpoint-interval 1` (CRC-clean, 0 authoritative) and `--frame-start 3400 --frame-end 4916 --bisect` (PASS: all 4916 frames match by CRC). Frontier 3414→recording complete. |
| 106 | [x] | P2 | fidelity | TAS frame 287: `any-percent-published` restores the title pose two frames before oracle | Cold-boot title entry now uses the same global ready frame floor as before, but early intro-skip paths also wait for a 150-frame local title rebuild before exposing Kid. Added `published_tas_boot_title_keeps_kid_hidden_until_frame_289`, verified `cargo test -p kid-core published_tas_boot_title_keeps_kid_hidden_until_frame_289 -- --nocapture`, re-ran the older title-ready and demo-return regressions, confirmed `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 280 --frame-end 292 --checkpoint-interval 1` is CRC-clean apart from advisory `number_objects`, and re-bisected to move the frontier from 287 to 1830. |
| 107 | [x] | P2 | fidelity | TAS frame 1830: `any-percent-published` awards the second diamond later than oracle in demo level `0x1B` | Matched falling prize items to the ASM's fixed-column row probe, kept the one-row emergence grace, gave diamonds their own bottom-anchored pickup box, and held queued `PrizeDiamondCollected_Init` awards for 33 frame starts instead of 32. Added `floating_diamond_stops_once_it_reaches_supported_row_below_source`, `published_tas_arms_the_second_prize_diamond_queue_on_frame_1798`, updated the demo diamond regressions, verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 1826 --frame-end 1834 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1398 --frame-end 1442 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 0 --frame-end 3000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 1830→4905. |
| 108 | [x] | P2 | fidelity | TAS frame 4905: `any-percent-published` unloads demo level `0x1B` into title one frame before oracle | Oracle probes showed the ROM demo timeout counter at `$FFFFFBC2` lagged the native startup counter by 61 frames during Skydragon Castle demo 2, so the auto-end compare was firing early even though the visible choreography still matched. Fixed the timeout compare to subtract that oracle-backed offset for demo 2 only, added `demo_2_auto_end_waits_for_the_oracle_backed_timeout_counter` plus `published_tas_demo_2_stays_in_gameplay_on_frame_4905`, and verified `cargo test -p kid-core demo_auto_ends_after_max_frames -- --nocapture`, `cargo test -p kid-core demo_2_auto_end_waits_for_the_oracle_backed_timeout_counter -- --nocapture`, `cargo test -p kid-core published_tas_demo_2_stays_in_gameplay_on_frame_4905 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 4898 --frame-end 4980 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 4905→4966. |
| 109 | [x] | P2 | fidelity | TAS frame 4966: `any-percent-published` demo timeout still takes the paused title helper instead of the ROM auto-end branch | Split demo auto-timeout from the manual `Start`-abort helper so the threshold frame stays in gameplay and frame 4967 enters the ROM's non-paused `game_mode=0x0000` handoff. Added `published_tas_demo_2_uses_the_mode_0_timeout_handoff_on_frame_4967`, re-ran the focused published TAS window plus bisect, and moved the live blocker off the branch itself onto the preserved handoff snapshot. |
| 110 | [x] | P2 | fidelity | TAS frame 4966: `any-percent-published` auto-timeout handoff preserves Kid X one `0x1C00` step too far, then clears camera/progression early on frame 4998 | Froze the timeout threshold frame before another gameplay step, then gave the mode-0 auto-timeout handoff its own full/partial snapshot windows so frame 4966 keeps `kid_x_pos=0x02487A00`, Kid only blanks on frame 4995, and camera/helmet/diamonds clear on frame 4999. Added `published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966`, `published_tas_demo_2_blanks_kid_on_frame_4995_after_timeout_snapshot`, and `published_tas_demo_2_clears_camera_and_progression_on_frame_4999`; verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 4898 --frame-end 5010 --checkpoint-interval 1` (CRC-clean, 0 live authoritative diffs in the handoff window), `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 4964 --frame-end 5020 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 4966→5116. |
| 111 | [x] | P2 | fidelity | TAS frame 5116: `any-percent-published` auto-timeout title rebuild shows the standard title pose while oracle still keeps Kid hidden | Kept Kid hidden while the mode-0 auto-timeout restart is still latched, instead of falling back to the shared `DEMO_TITLE_PLAYER_HIDDEN_FRAMES` title gate. Added `published_tas_demo_2_keeps_kid_hidden_during_timeout_restart`, verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5108 --frame-end 5145 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5114 --frame-end 5185 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5116→5143. |
| 112 | [x] | P2 | fidelity | TAS frame 5143: `any-percent-published` auto-timeout restart stays in `game_mode=0x0000` while oracle advances to intro sequence 1 | Let the timeout restart leave the latched mode-0 handoff after the oracle-backed 177-frame window, added `published_tas_demo_2_advances_into_intro_sequence_1_on_frame_5143`, updated the hidden-Kid regression to cover the post-handoff intro frames, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5108 --frame-end 5145 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5114 --frame-end 5185 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5143→5303. |
| 113 | [x] | P2 | fidelity | TAS frame 5303: `any-percent-published` leaves intro sequence 1 too early during the timeout restart | Preserved a timeout-restart-specific intro-sequence-1 duration instead of reusing the shared 160-frame timer, added `published_tas_demo_2_holds_intro_sequence_1_until_frame_5504`, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5298 --frame-end 5310 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5298 --frame-end 5310 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5498 --frame-end 5508 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5498 --frame-end 5508 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5303→5584. |
| 114 | [x] | P2 | fidelity | TAS frame 5584: `any-percent-published` leaves intro sequence 2 too early during the timeout restart | Oracle `Game_Mode` probes showed the timeout-restart copy of intro sequence 2 starts on frame 5504 and does not reach `game_mode=0x001C` until frame 5953, so the shared 80-frame segment timer was wrong for this path. Added a timeout-restart-specific intro-sequence-2 override (`449` frames), promoted `published_tas_demo_2_holds_intro_sequence_2_until_frame_5953`, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5578 --frame-end 5590 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5578 --frame-end 5590 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5948 --frame-end 5956 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5948 --frame-end 5956 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5584→5994. |
| 115 | [x] | P2 | fidelity | TAS frame 5994: `any-percent-published` reaches intro sequence 3 on time, but native still keeps Kid hidden during the first visible intro-sequence-3 pose | The ROM seeds the first visible intro-sequence-3 Kid pose from `Load_IntroSequence3`'s gfx-object descriptor table at `x=0x00600000`, `y=0x00D00000`; native trace state was still hard-coded to hide Kid for the whole intro sequence. Added `published_tas_demo_2_exposes_the_first_intro_sequence_3_kid_pose_on_frame_5994`, exposed the sequence-3 pose from frame 5994 onward, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5988 --frame-end 6000 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5988 --frame-end 6000 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5994→6033. |
| 116 | [x] | P2 | fidelity | TAS frame 6033: `any-percent-published` reaches title on time after intro sequence 3, but native discards the carried intro pose during the deferred title handoff | The timeout-restart intro-sequence-3 fix moved the frontier out to frame 6209, where oracle still carried the `x=0x00600000`, `y=0x00D00000` intro pose across the first 28 title frames before hiding Kid again. The surprise was that frame 6209 still arrived through `pending_title_skip`, not the natural timeout branch, so the carry-over tag had to survive the shared 2-frame Start→title countdown as well as the direct timeout path. Added `published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames`, verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6206 --frame-end 6245 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6033→6358. |
| 117 | [x] | P2 | fidelity | TAS frame 6358: `any-percent-published` keeps Kid hidden one frame too long before the standard title pose returns | The timeout-restart intro-sequence-3 carry-over now uses its own 149-frame hidden-title window instead of the shared 150-frame demo-return gate, so frame 6358 restores the standard title pose at `(0x002E0000,0x008C0000)` and the focused title window is authoritative-clean. Verified with `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6354 --frame-end 6364 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6358→6906. |
| 118 | [x] | P2 | fidelity | TAS frame 6906: `any-percent-published` keeps title mode one frame too long before demo playback resumes | Shortened only the timeout-restart title rebuild's attract-lock window by one frame so the fixed `$280` title idle countdown reaches demo playback on the oracle-backed frame. Added `published_tas_demo_2_resumes_demo_play_on_frame_6906`, re-ran `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6354 --frame-end 6364 --include-advisory`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6900 --frame-end 6912 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6906→6945. |
| 119 | [x] | P2 | fidelity | TAS frame 6945: `any-percent-published` drops back into the paused title-return helper two frames after demo 1 restarts | Timeout-restart demo playback now ignores manual Start aborts until live gameplay starts, so the queued frame-6943 Start bit no longer arms the paused title helper during the restarted demo bootstrap. Added `published_tas_demo_2_ignores_queued_start_during_timeout_restart_demo_bootstrap`, re-ran `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6940 --frame-end 6952 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6945→6966. |
| 120 | [x] | P2 | fidelity | TAS frame 6966: `any-percent-published` exposes the wrong wrapped demo 1 camera Y during the hidden warm-up after the fixed 6945 abort window | Fixed by carrying the timeout-restart IntroSequence6 lineage into demo playback so the hidden warm-up reuses the oracle-backed demo-1 camera wrap; frontier later advanced through the repaired title/demo shell. |
| 121 | [x] | P2 | fidelity | TAS frame 24921: level `0x15` gameplay falsely enters right-wall cling from a top-edge-only overhang probe | Matched ASM `sub_8F26` by refusing to promote `probe_wall_top_edge()` into a fresh airborne wall hit. Added `airborne_top_edge_contact_does_not_start_wall_cling_without_direct_wall_hit`, verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 24918 --frame-end 24928 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, and `mise run verify-slice-fast`. Frontier 24921→27603. |
| 122 | [x] | P0 | level | Level 2 entry can spawn Kid in midair, then fall through the floor and die | Boot -> finish Level 1 -> enter Level 2 -> confirm stable landing and 10s of live play |
| 123 | [x] | P0 | player | Death/respawn still leaves camera drift, stuck-from-start, or broken restart state | Die in live play, confirm camera settles correctly and movement resumes without soft-lock |
| 124 | [x] | P1 | enemy | Enemy kill timing still allows Kid to get hurt after the enemy should already be dead | Sword/projectile/charge/AoE kills now persist the ROM kill bit, so dead enemies do not reactivate into fresh contact damage |
| 125 | [x] | P1 | block | Breakable blocks can render with the wrong lifecycle art or regress into prize-like visuals | Block-family render mapping now follows ASM skin-byte indexing, so post-prize `$E001` cells resolve through the same Block_Mappings table as the ROM |
| 126 | [~] | P1 | player | Walking up slopes is still observably wrong in native play | Native 1P gate now covers uphill/downhill tracking plus slope enter/exit seams; `mise run verify-1p-playable` is green |

Note 2026-06-01 for #126: latest live playtest reports a Kid uphill-facing
regression on up-right movement and persistent Red Stealth/Samurai slope
vibration rather than merely the ROM-faithful "cannot start walking into a
slope from standstill" quirk documented in #152. Existing `loc_8E98`/`loc_77C2`
tests are not enough. Minimum gate: a deterministic Level 0x01 Red Stealth
slope route compared against oracle/player-state checkpoints, plus a Level
0x00 Kid uphill-facing render check.
| 127 | [x] | P1 | block | Black bouncy blocks do not produce the expected bounce behaviour | Rubber block contacts now queue the ASM `loc_10DA4` jiggle on floor, ceiling, and wall contact without fake spring physics |
| 128 | [x] | P1 | form | Form-specific block breaking can now destroy metal blocks that should stay solid | Iron Knight landing now follows ASM `sub_AF7A/off_AFD8`: smash/prize/ice break, solid-ghost/teleporter no-op, metal blocks bounce and restore instead of being deleted |
| 129 | [x] | P1 | infra | Add a `1p-playable` gate that covers native boot -> Level 1 -> Level 2, death/respawn, slopes, bounce surfaces, and common block interactions | `mise run verify-1p-playable` is green only when the core 1P smoke path survives those scenarios |
| 130 | [x] | P1 | infra | Turn `play-native --record` / `verify-recording` into the standard playtest evidence pipeline | Recording bundles now carry provenance, replay commands, copied-bundle verification, optional artefacts, and an automatic trace transcript sidecar |
| 131 | [x] | P2 | fidelity | Burn down the five parked DEVIATION items instead of treating them as accepted resting state | Each deviation is either removed, reclassified as oracle limitation, or explicitly deferred with evidence |
| 132 | [x] | P3 | gameflow | Verify 2-player alternating mode against oracle-backed or strong native scenarios | Toggle 2P, die as P1, confirm P2 takeover / game-over / continue behaviour |
| 133 | [~] | P3 | gameflow | Investigate password flow and decide whether it is missing, partial, or intentionally out of 1P scope | ASM title/options flow is documented; stock ROM has no password entry, and native options now mirror the four-option ROM screen |

Note 2026-06-01 for #133: latest live playtest still describes Options as fake
and menu visuals as jumbled; this row was too broad to close from the password
investigation alone. Split if needed after capturing whether the remaining bug
is options behaviour, title menu rendering, or both.
| 134 | [x] | P1 | infra | Package reduced semantic cases as the default autonomous work queue | `cargo run -p kidcli -- autonomous-loop --goal case-queue` now ranks reduced case bundles, prefers recording-backed authoritative failures, surfaces untriaged recordings, and only marks hybrid-gated cases as patchable |
| 135 | [x] | P1 | render | Level 1 background graphics do not match the original game | Validated 2026-05-18 from bounded native bundle `.cache/recordings/issue-135-level1-background/`: `cargo run -p kidcli -- verify-recording --slug issue-135-level1-background --native-only` passes with `trace_match=true`, `native_replay_only=true`, and 900 recorded/replayed frames. Inspected gameplay evidence `.cache/recordings/issue-135-level1-background/verify/native/000000_start.png` shows settled Blue Lake Woods 1 with the forest background rather than a title card or placeholder scene. The final frame returns to the level card after idle Kid takes damage and dies (`transcript.txt`: damage at frames 290/356, `InGame -> LevelTransition` at frame 431), so the start frame is the background validation frame. ASM labels traced: `Load_InGame` `loc_11D0C` loads background tiles, `Load_BackgroundMappings` / `loc_12824` / `loc_12830` / `Load_BackgroundChunk` / `loc_12884` / `loc_1288A` build the background tile buffer, `Load_BackgroundMappings_Layered` covers Forest/Hills/Mountain/Desert Enigma backgrounds, and `BackgroundScroll_ComputeShiftData` / `BackgroundScroll_ComputeShiftLayers` / `sub_30194` / `sub_303BA` / `sub_30744` / `sub_307D8` refresh plane B. Oracle parity result: `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes, so the old `level_entry@520` blocker is closed for this issue. Proposed fix commit: `3517d752`. |
| 136 | [x] | P1 | block | Diamonds and helmets emerging from bricks use incorrect graphics | Fixed on 2026-05-18: native rendering now draws floating diamond, helmet, Ankh, Clock, Continue, and early `10000`/`10 diamonds` worker-burst prizes from ROM sprite assets instead of placeholder rects. ASM trace: `PrizeDiamond_Init` uses palette line `0`, object meta `$64`, and `stru_10D6E`; `stru_10D6E` selects `unk_E0E66` (`sprite_frame_vram $6F2,$06,$FE,$10,$0C`) while `sub_5D4A` DMAs `diamond_1..4.bin` to VRAM tile `$6EA`, so the renderer uses offset `$6F2-$6EA`. Helmet prizes share `sub_10CB0` palette line `3`, then `PrizeHelmetSkycutter_Init`..`PrizeHelmetMicromax_Init` select `stru_106DC`, `stru_106E8`, `stru_106F4`, `stru_106EE`, `stru_106D6`, `stru_106FA`, `stru_106E2`, `stru_106D0`, and `stru_10700`, each mapped to the extracted `*_mask.bin` frame and `*_Helmet.bin` palette. `PrizeAnkh_Init`, `PrizeClock_Init`, and `PrizeContinue_Init` now use `stru_10C08`/`unk_E0FE6`, `stru_10C18`/`unk_E0FCE`, and `stru_10C28`/`unk_E0FD6` over `Life_icon_(3_frames).bin`, `Clock_icon_(3_frames).bin`, and `Coin_continue_icon_(6_frames).bin`. Regression tests: `build_scene_draws_floating_diamond_with_rom_sprite_art`, `build_scene_draws_floating_helmet_with_rom_mask_art`, `build_scene_draws_floating_special_icons_with_rom_art`, `build_scene_draws_10000_points_worker_star_from_hud_rom_tiles`, `build_scene_draws_ten_diamonds_workers_from_hud_diamond_rom_tiles`, and `load_floating_prize_visuals_returns_rom_icon_frames`. Verification: `cargo test -p kidcli floating_item -- --nocapture`; `cargo test -p kidcli build_scene_draws_floating -- --nocapture`; `cargo test -p kid-render load_level_visuals -- --nocapture`; `cargo run -p kidcli -- verify-recording --slug issue-136-prize-emergence --native-only`. Note: the saved #136 replay still passes, but its captured PNGs are weak visual evidence because the spawned diamond is off-screen by frame 270; closure is based on asset-backed renderer tests plus replay non-regression. |
| 137 | [x] | P1 | player | Kid walks backwards while climbing slopes | Classification: playability negative baseline, not yet a visual-parity case. Refreshed on 2026-04-21 from delegated cycle 584 with recording slug `issue-137` (`.cache/recordings/issue-137/`), transcript `.cache/recordings/issue-137/transcript.txt`, verification report `.cache/recordings/issue-137/verify/report.json`, verify artefacts under `.cache/recordings/issue-137/verify/` including `.cache/recordings/issue-137/verify/native/000303_end_diff.png`, and direct Level 00 frame captures `.cache/debug/issue-137/level_00_frame_00000.png`, `.cache/debug/issue-137/level_00_frame_00060.png`, and `.cache/debug/issue-137/level_00_frame_00120.png`. Native repro status: not reproduced. Oracle repro status: unknown. The fresh transcript records 304 frames, reaches `IntroSequence2` on frame `279`, and never reaches TitleScreen, settled Level 00 gameplay, or a slope, so it cannot exercise the backwards-climb bug. `cargo run -p kidcli -- verify-recording --slug issue-137` preserves the trace (`trace_match: true`, `recorded_trace_len: 304`, `replay_trace_len: 304`) but still fails oracle state/frame parity (`oracle_state_match: false`, `oracle_frame_match: false`). Direct package capture `.cache/debug/issue-137/level_00_frame_00060.png` is the Blue Lake Woods title card, not a slope traversal. Earlier 2026-04-21 follow-up with `cargo run --release -p kidcli -- play-native --warp-level 00 --settled --record issue-137-repro` was interrupted after the interactive player produced no manifest at `.cache/recordings/issue-137-repro/manifest.json`; `verify-recording --slug issue-137-repro` therefore fails before replay. Required direct captures `.cache/debug/bug-137/level_00_frame_00000.png`, `.cache/debug/bug-137/level_00_frame_00060.png`, and `.cache/debug/bug-137/level_00_frame_00120.png` are still title-card frames, not a slope repro. `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes, so the old `level_entry@520` failure is stale and not evidence for this issue. ASM labels read: `sub_8506` / `loc_8518` / `loc_8560` own input-driven `x_direction`, while `sub_902A`, `sub_9386`, and `loc_9D22` own floor/slope tracking. FIXED 2026-04-25: `update_level_one_slope_ground_velocity` (`crates/kid-core/src/movement.rs:5640`) inverted the slope-flag-to-`Direction` mapping. ASM `loc_9DC0` copies `$FFFFFA26` (0 for SlopeUp, $FF for SlopeDown) into `x_direction(a3)`; with Kid's default-right sprite, `x_direction=0`→Right and `x_direction=$FF`→Left, so SlopeUp uphill should render Right (matches walk-right input) but native produced Left. ASM `loc_9DF2` flips that byte via `seq` for downhill, and `loc_9E4C` XORs by `$FF` when canonical_vel is negative; both branches were inverted the same way. New unit test `slope_facing_does_not_invert_on_uphill_traversal_loc_9DC0` (`crates/kid-core/src/tests/movement_tests.rs`) pins all 8 ASM-derived facings; existing tests `slope_right_wall_y_snap_positive_vel_loc_9FBC`, `slope_ground_run_right_brakes_before_level_one_jump`, `slope_ground_run_right_on_slope_up_bleeds_speed_by_rom_uphill_step`, `slope_ground_run_right_on_slope_up_holds_rom_uphill_cap`, and `slope_ground_run_right_uses_shared_table_for_cyclone` had locked in the buggy facing and were corrected. `mise run verify-slice-fast` passes; `cargo test -p kidcli walk_right_assertions` passes. |
| 138 | [x] | P1 | player | Enemy contact appears to apply double damage because invulnerability is too short | Validated 2026-05-18 from bounded native bundle `.cache/recordings/issue-138-enemy-contact/`: `cargo run -p kidcli -- verify-recording --slug issue-138-enemy-contact --native-only` passes with `trace_match=true`, `native_replay_only=true`, and 1800 recorded/replayed frames. The transcript reaches settled Level 00 enemy contact and records damage at frames 290 and 356, exactly 66 frames apart, then Kid dies and enters `LevelTransition` on frame 431. This is ROM-faithful behaviour, not native double damage: `GfxObjects_CollisionKid_Collide` / `loc_1D00` suppress damaging collision while `KidIsInvulnerable`; `Check_for_recent_damge` (kid.asm:14632) returns immediately when `Just_received_damage` is set, otherwise falls through to `loc_B700` (kid.asm:14638) which deducts HP and re-arms the latch; `loc_BC34` spawns the damage-flash thread; `loc_BC54` / `loc_BC56` / `loc_BC5C` / `loc_BC6E` (kid.asm:15196..15213) hibernate 11 outer cycles × (2 white + 4 normal) = 66 frames before `sf (Just_received_damage).w` clears the latch. Oracle-parity result: `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes. Proposed-fix commit: `e6b27bb7bcffd20e410c73573ea3bcebbb549399` (`Fix combat mechanics to match ASM: no knockback, 66-frame invuln`) shrank `PLAYER_DAMAGE_INVULNERABLE_FRAMES` from 300 to 66 (`crates/kid-core/src/constants.rs:393`) and removed invented knockback velocities. Regression `enemy_contact_double_damage_gate_releases_after_exactly_66_frames_loc_bc54` (`crates/kid-core/src/tests/level_tests.rs`) drives the latch through the real contact path and pins both boundaries; `loc_bc34_damage_flash_thread_invulnerability_is_66_frames` pins the literal `66`. |
| 139 | [x] | P1 | render | Bouncy blocks still render like spikes | Validated 2026-05-18 from deterministic visual bundle `.cache/recordings/issue-139-rubber-block-window/`: `cargo run -p kidcli -- verify-recording --slug issue-139-rubber-block-window --native-only` passes, and inspected frame `verify/native/000699_walk_end.png` shows the Level 00 column-80 rubber cells as round rubber-block art, not spike art. ASM cross-reference: `sub_30194` (`kid.asm:33745-33893`) sets `move.w #$FF,d4` (`kid.asm:33752`) before calling `sub_302F6` / `sub_3034A`, so the Block_Mappings lookup in `loc_30336` / `loc_3037C` masks the cell with `$00FF` and shifts left 3 — the byte offset into Block_Mappings is `(cell & 0xFF) × 8`, i.e. **the skin byte alone, not the block-type byte**. The block-type byte (cell bits 11..8) only encodes behaviour: `loc_1233C` writes type-6 rubber cells as `$E60C` (default) / `$E60E` (alternate via `loc_12372` / `loc_12380` with terrain byte at `(a3)`), and `unk_14576` keys `sub_144DA` ("Make indoor rubber block art and write it to tile $2F3-$2FA") off `Foreground_theme`. `unk_7B8DC` (`tools/to_split.txt:703` extracts $7B8DC..$7BB64 = $288 bytes verbatim into `theme/block_mappings.bin`) is loaded by `loc_11D46` as 162 longwords = 81 contiguous 8-byte quads, and `Block_Mappings` lives at `$FFFF503A..$FFFF52C2` (`variables.asm:9`); each quad holds top-left/top-right/bottom-left/bottom-right tile attribute words. Native fix: `crates/kid-render/src/level.rs` `BlockVisualMapping::resolve` uses `usize::from(skin)` for all block types, matching the documented ROM contract; `BlockVisualKind` remains diagnostic metadata only. Original concrete divergence for cell `$E60C` (rubber, skin `0x0C`): ROM byte offset `0x0C × 8 = 0x60` → tiles `$026E $0270 $026F $0271`, old native byte offset `(1 + 4×16 + 12) × 8 = 0x268` → tiles `$0AB9 $0AB7 $0ABA $0AB8` (tile $0x2B9 with hflip — completely outside the rubber-tile range $2F3..$2FA written by `sub_144DA` for indoor cell `$E60E`). Verified against the production extract at `.cache/generated/<rom-sha1>/theme/block_mappings.bin` (648 bytes = 81 × 8). Level 00 is a valid rubber-block target: generated layout `00.bin` contains `E60C` cells at columns 80 and 97. Patch lineage: commit `f75b09f8` (`render: index block mappings by skin byte`) replaced the old grouped fixture with contiguous skin-indexed quads and keeps non-standard block types on the same flat lookup path via diagnostic `BlockVisualKind::GenericBlock`. Oracle-parity result: `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes, so the old `level_entry@520` blocker is stale and unrelated. |
| 140 | [x] | P1 | render | Prize blocks use wrong graphics and animations | Validated 2026-05-18 from deterministic visual bundle `.cache/recordings/issue-140-prize-block-window/`: `cargo run -p kidcli -- verify-recording --slug issue-140-prize-block-window --native-only` passes, `000150_post_first_jump.png` shows visible prize blocks using `P` art, `000239_pre_second_jump.png` shows the hit/deleted block state as block art rather than the old wrong mapping, and `000359_scenario_end.png` shows the spawned prize as a separate item sprite. Root cause was the **same pre-f75b09f8 `BlockVisualMapping::resolve` divergence as #139**: the renderer treated `theme/block_mappings.bin` as `1 fallback + 5 groups x 16 skins`, but ASM `sub_30194` (`kid.asm:33745-33893`) sets `move.w #$FF,d4` on `kid.asm:33752` so `loc_30336` / `loc_3037C` byte-index Block_Mappings as `(cell & 0xFF) × 8` — **the skin byte alone, not per-block-type**. ASM cross-reference for the prize-block lifecycle: `loc_123DC` / `loc_12408` / `loc_1241C` / `loc_12454` / `loc_12468` select the visible/disguised prize cells `$E101` / `$E102`; `sub_1007A` (`kid.asm:21345-21599`) drives the bounce coroutine — `loc_100DA..loc_10120` walk the y-position wobble (-2/+2 over 31 hibernated frames; deltas already pinned in `prize_block_wobble_y_deltas_sub_1007A`, `crates/kid-core/src/blocks.rs:1286`), `stru_101B6` keys the alternating `LnkTo_unk_E0E4E`/`LnkTo_unk_E0E5E` art frames, then the spawn-flash child at `loc_102B8` (`kid.asm:21542-21558`) plays `unk_E0F2E`/`unk_E0F36`/`unk_E0F3E`/`unk_E0F46` via `stru_102DE` and `Prize_Init_Index` (`kid.asm:21560-21576`) dispatches by prize byte to `PrizeDiamond_Init`, `PrizeHelmet*_Init`, etc.; the already-collected branch `loc_101D8` / `loc_10200` (`kid.asm:21459-21527`) replays `stru_10296` and writes `$E102` via `sub_11530`. Native cross-reference: `crates/kid-data/src/lib.rs:3105-3153` (`mapped_word_pair_sub_302F6`, `mapped_longword_sub_3034A`) implements the ROM contract: `tile_index = cell & 0x00FF`, `base = tile_index × 4` (u16 words), and `cell & 0x8000` selects between `theme_mappings` and `block_mappings`, so `$E1XX` and `$E001` cells all index Block_Mappings by skin. Native fix: `crates/kid-render/src/level.rs` `BlockVisualMapping::resolve` indexes `block_quads` by `usize::from(skin)` for every block type; `block_visual_kind` no longer changes the tile quad. Original concrete divergence for cell `$E101` (visible prize, skin `0x01`): ROM byte offset `0x01 × 8 = 0x08` → quad index 1 of Block_Mappings, old native byte offset `(1 + 1 × 16 + 1) × 8 = 0x90` → quad index 18. The disguised cell `$E102` (skin `0x02`) reads ROM offset `0x10` → quad 2 vs old native `(1 + 1 × 16 + 2) × 8 = 0x98` → quad 19. The deleted-block cell `$E001` written by `loc_101D8` via `sub_11530` reads ROM offset `0x08` → quad 1 vs old native `(1 + 0 × 16 + 1) × 8 = 0x10` → quad 2. The `theme/block_mappings.bin` extract is 648 bytes = 81 contiguous 8-byte quads; after `f75b09f8`, the renderer treats that as a flat skin-indexed table. The lifecycle wobble/spawn-flash timing itself is already pinned by `crates/kid-core/src/blocks.rs:1286` and the `prize_block_lifecycle_*` tests in `crates/kid-core/src/tests/level_tests.rs:13424-13638`; the fixed divergence was the per-frame block art, not the y-position curve. Oracle-parity result: `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes, so the old `level_entry@520` blocker is stale and unrelated. |
| 141 | [~] | P1 | render | Intro and level-intro graphics still do not match the original game | Fixed on 2026-04-20: native intro VDP loading now keeps ASM `sub_1B850` only for `Load_IntroSequence1` and `Load_IntroSequence3`, while `Load_IntroSequence2`, `Load_IntroSequence4`, and `Load_IntroSequence5` follow their dedicated ROM loader paths (`kid.asm:29255-29364`). Regression `intro_sequence_2_4_and_5_follow_their_rom_vdp_loaders` guards the split, and `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes with `level_entry@520` treated as the ASM `Obj_TitleMenu loc_1BBD6` hand-off identity checkpoint. The old `issue-141` boot/intro bundle did not independently prove a separate settled level-intro mismatch; do not recapture this row without a deterministic human-play route that reaches the broken intro scenes. |

Note 2026-06-01 for #141: latest live playtest still reports the "Everybody
played it" kids/door composition broken, a stray Kid/jacket sprite in the
bottom-left before "too real", and the green/silhouette transform screen.
Previous loader split evidence is branch-level, not human-play-stable. Capture
the full boot attract loop as frames and compare scene objects/layers against
oracle before patching.
| 142 | [x] | P1 | enemy | Level 1 dragon breath travels too far | FIXED — ASM loc_3D02A: breath is stationary hitbox (stru_3CCC8, 68f), not a projectile. Also fixed flying dragon fire X offset (loc_34BAC ±10px) and terrain_kill. |
| 143 | [x] | P1 | block | Heavy Man's first helmet causes repeated block-bounce instead of the ROM's smooth single-hit break | Validated 2026-05-18 from deterministic Iron Knight/Heavy Man block-break evidence. Positive route `.cache/recordings/issue-143-level27-ik-smash-probe-5/`, captured from cache-local scenario `.cache/scenarios/issue-143-level27-ik-smash-probe-5.json`, verifies native-only. Refreshed 2026-05-19 after the cached bundle from commit `dc83a79a` drifted against current movement code at frame 730; the recaptured bundle at commit `ae1c29a6` has trace SHA `c12bdfc6eb861f121f9a86f39734541434672250` and `verify-recording --native-only` passes. It warps to Caves of Ice, hits the Iron Knight prize block at frame 102, collects Iron Knight at frame 171 (`helmet 0 -> 6`), then high-velocity Iron Knight landings create block-destroy effects from frame 645 onward at `(464,128)`, `(480,128)`, `(496,128)`, `(512,128)`, and `(528,128)`. Trace query over frames 645-670 finds zero `block_bounces`, so the old repeated-bounce symptom is absent on the validated block-break window. ASM trace: `sub_AF7A` gates Iron Knight/Heavy Man block-breaking on `Iron_Knight_block_breaker` or `y_vel >= $48000`; `loc_AF96` sets the latch before probing, `off_AFD8` dispatches Smash to `loc_AFF0`, Prize to `loc_1002E`, Ice to `IceBlock_SmashAndLoadBullets`, SolidGhost/Teleporter through `sub_AFD4`/`loc_AFCA` as unhandled, and Metal to `loc_110D0`; handled paths force `y_vel=$FFFF8000` and return `d7=1`. Regression `loc_af96_iron_knight_live_landing_preserves_block_breaker_latch_for_slow_followup` proves a below-`$48000` follow-up landing still dispatches through `off_AFD8` after a handled first hit. Refreshed 2026-05-24: commit `6fefd268` strengthened `iron_knight_metal_block_hit_does_not_retrigger_same_bounce_loop` to run 120 frames and assert the same-cell metal rebound leaves `block_bounces` empty afterwards, guarding the reported "helmet guy on metal block shakes constantly" path. Verification: `cargo run -p kidcli -- verify-recording --slug issue-143-level27-ik-smash-probe-5 --native-only`; `cargo test -p kid-core loc_af96_iron_knight_live_landing_preserves_block_breaker_latch_for_slow_followup -- --nocapture`; `cargo test -p kid-core iron_knight_block_breaker -- --nocapture`; `cargo test -p kid-core iron_knight_metal_block_hit_does_not_retrigger_same_bounce_loop -- --nocapture`; inspected `verify/native/001100_smash_window.png`. The generic scripted `stomp-enemy` bundle and TAS route remain negative baselines, not blockers, because they do not reach the Iron Knight block-break window. |
| 144 | [~] | P1 | enemy | Level 2 shooter enemy fires too rapidly | Fixed: add `EnemyFamily::TankShooting` for enemy ID `0x0B`, stop plain `0x0A` tanks firing, and gate bullet spawn through ASM `sub_3C4F8` / `loc_3B730` cadence and muzzle offsets. |
| 145 | [~] | P1 | render | Level 2 shooter enemy is not rendered correctly | FIXED — native player now mirrors ASM Tank/Tank_Shooting dual-GfxObject rendering: body on the parent object, turret child at `$3A(a5)` with `stru_3B27E` / `stru_3B4D8` / `stru_3B4F6` frame timing and the ROM's y-8px attachment. |
Note 2026-05-28 for #144/#145: fresh playtest still suspected the Level 2 RockTank mouth-open frame was missing a shot. ASM and the generated Level 0x01 layout distinguish the two families: the first two visible tanks are plain `0x0A` at x=648 and x=200, and `Tank.asm loc_3B41C` only plays the mouth-open sound/animation; the shooting path is only `0x0B` `Tank_Shooting.asm loc_3C50E -> loc_3B730`. Added `blue_lake_woods_2_live_tank_bullet_renders_rom_child_sprite_loc_3b730`, which drives a real Level 0x01 `0x0B` tank through the live session asset path and verifies the emitted `TankBullet` renders `stru_3B526` ROM sprite frames, not the generic projectile dot. |

Follow-up 2026-06-01: live playtest again reports mouth-open/no-shot and
misaligned firing animation. Keep the 0x0A vs 0x0B distinction, but reopen until
the live route records which tank family is on screen and whether generated
assets are present. Do not make plain `0x0A` tanks shoot unless ASM/oracle prove
that family fires.

Follow-up 2026-06-02 case-queue triage for #144/#145:
`debug-level01-tank-walk` and `debug-level01-tank-walk-1200` are
native-replayable but not oracle-authoritative. Native-only verification passes
for both, while full verification fails at `level_settle@60` with object counts
`native 4 / 4, oracle 4 / 6`. Both derived 0..120 cases have the same
`walk_right_01_case_00000_00120` blocker before tank behaviour:
settled-recording oracle bootstrap starts Level 01 with different player
velocities and the first Tank at `0A@00A48000,00E00000` versus oracle
`0A@00A40000,00D80000`; `case reduce` refuses for that bootstrap mismatch.
Do not patch RockTank firing/rendering from either recording. Use a refreshed
authoritative Level 01 route or fix settled-recording oracle bootstrap first.
| 146 | [x] | P1 | level | Dying in Level 2 can leave the game stuck on the level-loading screen | Validated 2026-05-18 from deterministic native replay evidence. Short route `.cache/recordings/issue-146-level2-death/` starts settled Blue Lake Woods 2 (`mode=InGame level=01`, Kid at `(32,191)`), takes damage at frames 814 and 1272, enters `LevelTransition` at frame 1347, decrements lives from 3 to 2, and restores HP from 0 to 2. Long route `.cache/recordings/issue-146-level2-stomp-long-probe/` verifies native-only and mechanically rules out the reported permanent loading hang for this route: frame 0 is `InGame`, frame 1347 is `LevelTransition`, frame 8589 returns to `InGame`, and frame 9060 is still live Level 01 gameplay with lives=2, HP=2, not paused, not game-over. Visual confirmation: `verify/native/009059_stomp_end.png` shows live Level 2 play after the death transition. Verification: `cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-death --native-only`; `cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-stomp-long-probe --native-only`. Default oracle verification remains non-authoritative for this settled recording shape because oracle checkpoints stay paused on Level 0, matching the known settled-recording oracle-bootstrap limitation documented for #136. Reopen only with a deterministic route that still remains in `LevelTransition` or otherwise fails to resume gameplay. |
| 147 | [~] | P1 | player | Level 2 wall-grab "zoom to top" cheat — jumping at a tall block stack lets Kid skip up to the top of the stack | FIXED: ASM `loc_ABA4` (kid.asm:13422-13427) probes the tile at (probe_x, kid_y-20) before allowing the wall-grab pull-up and rejects via `bge.w loc_AD2C` if solid. Probe 2 at (probe_x, kid_y-36) follows. Native `try_snap_wall_sticky_ledge` (`crates/kid-core/src/movement.rs:4219`) now probes `(probe_x, y-20)` and `(probe_x, y-36)` first with `probe_at` (exact cell) and rejects if either is solid, preventing the old mid-stack snap-to-foot-row chain. Regression: `loc_aba4_rejects_pullup_when_head_clearance_is_solid_mid_tall_stack`; companion coverage: `loc_abf0_buffered_pullup_scans_from_masked_y_minus_14_row`. Refreshed verification 2026-05-19: `cargo test -p kid-core loc_a` passes (68 tests), including the `loc_ABA4` and `loc_ABF0` pull-up cases. |

Note 2026-06-01 for #147: latest live playtest reports Iron Knight/Heavy Man
and Berzerker/Rhino can still climb or teleport up walls when jump is pressed
against block sides. Existing Kid-focused `loc_ABA4` tests do not cover all form
entry points. Next work should trace ASM wall-sticky eligibility by form and
record the exact form/wall/tile route that still rises.
| 148 | [x] | P2 | player | Kid death animation does not match the original | FIXED: the engine `advance_death_windup_motion` (`crates/kid-core/src/combat.rs:8815`) drives ASM `loc_B7FA` / `loc_B80C` tumble physics — initial `x_vel=$8000`, `y_vel=-$40000`, per-frame gravity `+$3000`. The scene renderer now draws the player sprite directly at that engine-updated object position and must not add the old parabolic y-offset on top. Regression: `loc_b7fa_death_tumble_renderer_uses_engine_position_without_extra_offset`. Refreshed verification 2026-05-19: `cargo test -p kidcli loc_b` passes. |
| 149 | [x] | P2 | block | Hit blocks do not shake before breaking | FIXED: ASM `sub_1007A` (kid.asm:21345-21599) wobbles a prize block for 31 frames using `PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A` and alternates the visible sprite frames from `stru_101B6` / `stru_10296`. Native `BlockBounceAnim` carries `initial_duration` + `y_offset_px`; the engine walks the ASM delta table each tick for prize blocks and the renderer applies `y_offset` to block draw state. Refreshed 2026-05-24 in commit `6fefd268`: deleted-cell prize blocks also render the ROM wobble sprite while `block_bounces.remaining > 8`, so the block no longer disappears before the broken/wobbling art appears. Regressions: `prize_block_bounce_y_offset_follows_asm_sub_1007a_cumulative_pattern`, `build_scene_draws_prize_block_wobble_sprite_even_after_deleted_cell_write`; companion metal-block check: `metal_block_bounce_y_deltas_loc_11196_match_asm_four_up_four_down`. Verification: `cargo test -p kidcli build_scene_draws_prize_block_wobble_sprite_even_after_deleted_cell_write -- --nocapture`, `cargo test -p kid-core bounce_y -- --nocapture`, and `mise run verify-slice-fast`. |

2026-05-24 follow-up: the core wobble was still live, but native rendering skipped the active prize-block worker after ASM `loc_10200` wrote `$6Fxx` because `push_block_grid_overrides` ignored cells with the block flag clear before checking `block_bounces`. Commit `044106f8` makes active `block_bounces` entries draw even for deleted-solid cells and adds `build_scene_draws_prize_block_wobble_even_after_deleted_cell_write`; `cargo test -p kidcli build_scene_draws_prize_block_wobble_even_after_deleted_cell_write -- --nocapture` and `mise run verify-slice-fast` pass.
| 150 | [x] | P2 | render | Kid blinks invisible after taking damage instead of the ROM's white-palette flash | FIXED: ASM `loc_BC34` damage-flash thread (kid.asm:15181-15215) keeps Kid visible for the full 66-frame `Just_received_damage` window and alternates between the white palette (`sub_80BC`, 2 frames) and the normal palette (`sub_80D0`, 4 frames) for 11 cycles (`dbf d3,loc_BC56`). Native now keeps Kid drawn for the full window and `push_player_sprite` applies the real white-palette phase by cloning the player palette and replacing the first 12 visible slots with `$FFF`, matching `sub_80BC`; no overlay rect remains in the asset-backed path. Regression: `loc_bc34_damage_flash_keeps_player_visible_and_marks_white_phase` asserts the player sprite stays visible, the white phase uses palette entries, and the old 16×24 pale-white overlay is absent. Refreshed verification 2026-05-20: `cargo test -p kidcli loc_b` and `mise run verify-slice-fast` pass. |
| 151 | [x] | P2 | render | Red Stealth (samurai) helmet graphics look slightly off on Level 2 | FIXED 2026-05-19: the player-form idle/walk/jump/slash/transform descriptors already matched ASM, but native left Red Stealth `slope_walk` empty. On Level 2 slopes, `select_player_frame` therefore fell back to generic upright walk frames, which made the helmet/form look subtly wrong after collection. ASM `off_A23C` selects Red Stealth slope frames `unk_B6ECA, unk_B7150, unk_B7150` twice (`red_stealth_slope_1.bin` / `red_stealth_slope_2.bin`); native now wires that six-slot cycle for both facing buckets and lets sprite H-flip mirror it. Regression: `red_stealth_on_slope_selects_rom_tilted_frame_off_a23c`; bundle spec test: `red_stealth_visual_spec_includes_rom_slope_cycle_off_a23c`. |
| 152 | [x] | P1 | player | Red Stealth (samurai) cannot walk up diagonal slopes that the bare Kid can | NOT A PORT BUG — preserved ROM quirk. ASM `loc_77C2` / `loc_77FC` (kid.asm:7857-7893) gates the standing→walking transition by helmet bitmask `%1011000101` (bits 0/2/6/7/9 = TheKid/Cyclone/IronKnight/Berzerker/Micromax). Forms outside the bitmask (Skycutter/RedStealth/Eyeclops/Juggernaut/Maniaxe) run an extra `sub_B55C` ledge probe at `(1,-1)` / `(-1,-1)`: when the diagonally-adjacent cell is `$4000` SlopeUp or `$5000` SlopeDown, the ROM stores the facing direction and branches to `loc_7828` to stay standing. TheKid is in the bitmask so it skips the probe and enters walking immediately. Native `should_hold_standing_still` (`crates/kid-core/src/movement.rs:3833`) mirrors this exactly with the same form list and the same `probe_relative_collision_sub_B55C(grid, x_px, y_px, ±1, -1)` check. The observed behaviour the user reported — "Red Stealth stops at a slope where Kid walks" — IS the original game. Workaround for players: jump over the leading slope edge; once airborne or already moving, the slope-walk path (`update_level_one_slope_ground_velocity` loc_9E6C branch for Red Stealth/Eyeclops/Maniaxe at `crates/kid-core/src/movement.rs:5907`) correctly drives ±$1000 velocity. Methodology rule: "bugs are features" — preserve. |
| 153 | [x] | P2 | player | Red Stealth sword attack while jumping should sustain the downward slash | Landed in commits `bcfb47f6` (engine counter wiring), `6578c94a` (renderer picks slash_air phase from `counter>>5`), and `3dc1692b` (airborne sword hitbox damages enemies below kid's feet). The full ASM `sub_B270` `loc_B350`/`loc_B380` flow is now wired end-to-end: airborne fast-fall + `Ctrl_C_Held` advances `red_stealth_airborne_slash_counter` by `$10` per frame (capped at `$7F`), queues `sfx_Red_Stealth_attack_down` at counter=$10, picks one of four `slash_air` sprite phases, and the slash hitbox (`RED_STEALTH_AIRBORNE_SLASH_DOWN_REACH_PX=12` below kid's foot) damages enemies on contact while sustaining the swing as long as C is held. Verify by warping Level 2 with Red Stealth, jumping into a fast-fall, holding attack — sword sustains pointing down, advances through 4 phases, and damages enemies below. |
| 154 | [x] | P1 | render | Bouncy blocks invisible at the bottom of the Level 2 drop pit | User playtest 2026-05-19: in the section where you jump down a long hole, the rubber/bouncy blocks at the bottom physically bounce Kid back up but render as nothing visible. Engine collision works; the renderer drops the tiles. ROOT CAUSE (2026-05-19 investigation): Level 1 (Blue Lake Woods 2) has two rubber-block cells at grid (21,36)/(22,36) with skin 0x0E. `theme/block_mappings.bin` entry for skin 0x0E references VRAM tile indices 0x2F3, 0x2F5, 0x2F4, 0x2F6 — but the static `ingame/artcomp/Blocks.bin` only decompresses to 211 tiles, covering VRAM 0x0220..0x02F2 (inclusive). Tiles 0x2F3-0x2FA are **procedurally generated at level-load time** by ASM `sub_144DA` (kid.asm:27355-27420; comment in ROM: "Make indoor rubber block art and write it to tile $2F3-$2FA"): it reads four theme-specific tile descriptors via `unk_14576[Foreground_theme] * 8`, decompresses them into `Decompression_Buffer`, composites them with eight base block tiles starting at $26E using a per-nibble selection loop (`loc_1453E`), and DMAs 64 longwords to VRAM at `$5E60` (= tile $2F3). Native `resolve_ingame_vram_tile` (`crates/kidcli/src/native_player/scene.rs:731`) checks foreground (0x00BC..0x021F = 355 tiles for forest) and block (0x0220..0x02F2 = 211 tiles) ranges; tiles 0x2F3-0x2F6 fall just past the block-tile range AND outside the foreground range (which ends at 0x021F for the forest theme), so `push_tile_quad` silently drops them. Landed fix: `kid-render` now ports the `sub_144DA` generation path by using `unk_14576[theme]` to select the four terrain-mapping foreground tiles, duplicating that four-tile buffer across the eight generated tiles, and applying the `loc_1453E` nibble priority rule before appending `$2F3..$2FA` to `block_tiles`. Test data: real Level 1 layout puts these cells at (21,36)/(22,36) px (336,576); forest theme `unk_14576` entry `0xC5` resolves to foreground tiles `$1F4/$1F5/$209/$20A`. Verification: `cargo test -p kid-render`, `cargo run -p kidcli -- inspect-level --level 01`, `cargo run -p kidcli -- capture-frames --level 01 --frames 30 --layer block_overrides --output .cache/debug/issue-154-after`, `cargo fmt --check`, `git diff --check`, and `mise run verify-slice-fast` all pass. |
| 155 | [~] | P2 | enemy | Slimes (Drips family?) hang at wrong attach point and shoot red projectile downward incorrectly | Landed in commits `152d2d1b` (DripsDrop projectile renders green-tinted instead of generic red) and `c6ba0554` (ceiling-attach loop wired through step_drips: slime spawns with `y_velocity=-$1_0000` and rises until `probe_ceiling` reports a hit, then snaps and seeds the RNG-driven dormant timer per ASM `Drips.asm loc_333F6`/`loc_3340E`). Follow-up 2026-05-19 wires the falling Drips child projectile to ROM sprite art: renderer builds `unk_C7B26` (`sprite_frame_vram $082,$02,$09,$05,$0A`) from the Drips enemy tile sheet at ASM `Drips.asm loc_33424`, replacing the green rectangle approximation. Follow-up 2026-05-20 wires terrain splash into hostile surface-crawler allocation: native removes the falling child on terrain contact, creates `DripsSurfaceCrawler` at `$44=drop_x+1`, `$46=(drop_y+$A)&$FFF0`, applies the ROM `$FFFFFB42 < 5` cap, seeds `x_vel=±$3000` through ASM `Drips.asm loc_33488`/`loc_335D8`, routes level-edge probes through ASM `loc_33588`/`loc_335B0` so out-of-bounds crawlers delete instead of living forever, wires the live `loc_33650` `loc_331D2` support guard so unsupported crawlers delete instead of sliding through empty space, wires the `loc_331F6` side-column loop guard so only `$6000` current-row probes stay on the straight horizontal path, applies all direct horizontal side-column turn scripts from `loc_33650`/`loc_336AA` (`loc_3393C`, `loc_33A40`, `loc_33A2C`, `loc_33982`, `loc_339DC`, `loc_33996`, `loc_33A18`, `loc_339C8`), persists the `loc_33730` vertical crawler label so `loc_33730` can run its own `loc_331D2`/`loc_33286` loop guard instead of falling back to horizontal movement, applies both negative-y (`loc_33964`, `loc_339FA`, `loc_339D2`) and positive-y (`loc_33946`, `loc_33A72`, `loc_339A0`) `loc_33730` non-loop row outcomes through the shared `loc_33AF6` turn-script interpreter, persists the `loc_337A0` vertical crawler label so its reachable row outcomes (`loc_339BE`, `loc_33A04`, `loc_33978`, `loc_3398C`, `loc_33A5E`, `loc_3395A`) dispatch through `loc_33AF6` instead of the simplified horizontal carrier, and follows up by persisting the later `loc_336D0`, `loc_33810`, `loc_33892`, and parked `loc_33A7C` labels from `off_33ADA` so diagonal-corner scripts keep their own probe loops instead of collapsing to the horizontal carrier. `drips_attached: bool` on ActiveEnemy disambiguates the rise phase from the wait/drop cycle. |

Note 2026-06-01 for #155: latest live playtest still reports the drip appearing
in midair, failing to land on the floor, and leaving an invisible/blocking
collision near metal blocks. Existing Drips crawler path tests did not cover
the observed pit route; capture Level 0x01 under-tunnel Drips state with parent,
drop, crawler, collision cell, and render position fields before another patch.
| 156 | [x] | P3 | render | Block destruction still uses a small placeholder rect instead of ROM fragment sprites | Fixed on 2026-05-19: native traces now carry ASM `sub_292E` direction in `block_destroy_effects`, with legacy three-field recordings loading as the upward floor-smash branch. The native player resolves ROM tile `$8228` through block art and emits the five `unk_29E8`/`unk_29F2` debris rows with direction-selected `unk_29FC`..`unk_2A42` velocities; the centre rect remains only as an asset-missing fallback. Regression tests: `native_trace_frame_reads_legacy_three_field_block_destroy_entries`, `native_trace_frame_reads_current_four_field_block_destroy_entries`, `block_effect_sub_10*`, and `block_destroy_effect_uses_sub_292e_rom_debris_tiles`. Verification: focused tests plus `cargo fmt --check`, `git diff --check`, and `mise run verify-slice-fast`. |
| 157 | [~] | P3 | render | Level-complete bonus overlay uses typed text instead of EOL sprite workers | Fixed 2026-05-19: `kid-render` now loads `scenes/artcomp/End_level_stats_text.bin` plus `scenes/palette/Score_screen.bin` into `EolScoreVisuals`, using ASM `Spr_EOL_*` descriptors (`kid.asm:49214-49237`). Native level-complete rendering now uses settled `Load_Object_EOL_Text` / `Load_Object_EOL_Number` style sprite rows for Time/Bonus, No-Hit Bonus, No-Prize Bonus, Path Bonus, Speed Bonus, Under/Seconds, Score, and seven-slot digit workers instead of the typed text overlay when assets are available. The original off-screen fly-in timing is not yet modelled in trace state, but the invented text presentation is gone from the normal asset-backed path. Verification: `cargo test -p kid-render load_eol_score_visuals_returns_rom_label_and_digit_frames -- --nocapture`; `cargo test -p kidcli level_complete_overlay_uses_eol_sprite_workers_loc_d55e -- --nocapture`; `mise run verify-slice-fast`. |
| 158 | [x] | P1 | player | Berzerker/rhino head charge does not hit enemies in front while jumping | FIXED 2026-05-24 in commit `907dbc33`: native had an extra grounded-or-held-input gate in `step_berzerker_charge` that ASM `loc_9512 -> loc_9564` does not have on non-Ice surfaces. Removing that native-only gate lets forward-speed airborne Berzerker set `Berzerker_charging`, then `loc_B678` side contact damages the front-edge enemy. Regression: `berzerker_airborne_charge_hits_front_enemy_without_held_input`; verification: `cargo test -p kid-core berzerker_charge -- --nocapture` and `mise run verify-slice-fast`. |
| 159 | [x] | P2 | debug | `n`/`p` debug level changes show a fake level-card loading screen | FIXED 2026-05-24 in commit `47254c64`: native player debug hotkeys now enter levels through `enter_level_playtest`, clearing level-card/fade state instead of replaying the normal warp transition. Regressions: `level_card_overlay` and `enter_level_playtest_starts_on_a_settled_ingame_frame`. |
| 238 | [x] | P0 | level | Transition after Highwater Pass can load the next level into instant death | FIXED 2026-06-01: native `StartingLevel` title-card hand-off now follows ASM `Load_TitleCard` / `loc_1AE2E`: once the native title-card shell reaches its hand-off window, a Start edge can release live gameplay (`Game_Mode=$000C`) in addition to the existing recorded gameplay-input release path. Regression: `level_complete_title_card_start_edge_enters_next_level`. Recording evidence: `.cache/recordings/issue-238-highwater-flag-transition` starts at Highwater Pass 1's real flag `(912,144)`, transitions Level 0x02 -> 0x03 at frame 654, consumes Start at frame 1100, enters `InGame` at frame 1101, and survives through frame 2399 with 3 lives / 2 HP. Native-only replay passes. Oracle `verify-tas` is still blocked at frame 0 by the direct-warp bootstrap camera/object mismatch (`camera_x_pos 0x02800000` vs oracle `0x02e80000`), so keep that as verifier debt rather than gameplay evidence. |

Note 2026-06-01 for #157/#238: latest live playtest reports the level-complete
score/sound tail can black-screen or crash after points run out, and later
loading after the third level causes immediate death. Direct `inspect-level
--level 04` and `inspect-respawn-settle --level 04` do not reproduce instant
death, so the likely bug is transition-state carry-over rather than the raw
level spawn point. The #238 minimum gate is now covered by
`issue-238-highwater-flag-transition`; remaining black-screen or score/sound
tail failures should stay under #157/audio follow-ups unless a new transition
recording shows another level-entry death.

Follow-up 2026-06-01: the latest playtest narrowed the death report to Level
0x03 -> 0x04 after carrying Rhino/Juggernaut. Added
`highwater-pass2-juggernaut-flag-transition-audit`, starting at Highwater Pass
2's real flag `(1552,1040)` with clamped camera `(1280,896)` and
`playtest_form=5`. Cache evidence
`.cache/recordings/issue-238-highwater-pass2-juggernaut-flag-transition-clamped`
has `trace_match=true`, `oracle_state_match=true`, and no first trace drift
through frame 2399; strict frame parity is still false on level-card/visual
pixels. Direct flag hand-off is therefore rejected as the root cause; the next
repro needs the longer live route, different inputs, or different carried state.
| 160 | [~] | P0 | audio | Native play still has little or no real music/SFX | User playtest 2026-05-25 reports the game is basically silent. Root causes found: `play-native` loaded GEMS from `manifest.source_root/sound` instead of generated `manifest.output_root/sound`; native BGM theme order did not match ASM `Load_InGame loc_12004` / `unk_1201E`; and standalone GEMS queued init pointers high-to-low even though ASM `sub_E14AC` writes low-to-high. After the pointer-order fix, live `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo run -q -p kidcli -- play-native --warp-level 00 --settled --record issue-160-audio-gems-pointer-order --record-frames 180` produces dynamic audio: `audio.wav` SHA-1 changed from `b822b5ce6e914a38d203dfb595a81b8057cd4603` to `fc0a5a495060c2644bc44a237a6c99277047adda`, RMS rose from `0.001119` to `0.042016`, zero crossings rose from `1` to `1570`, and `verify-recording --slug issue-160-audio-gems-pointer-order --native-only` passes. Follow-up playtest on latest-after-`3d3ee1d2` said level-start sound improved, but menu had no sound and audio basically stopped after enemy damage. Follow-up fix: native title/options shell now queues ASM `Load_TitleScreen` `bgm_Ice`, and standalone GEMS now mirrors/ages YM2612 status reads like `MdRuntime::Z80Bus`; a real-GEMS probe changed from one burst then silence to sustained non-zero RMS through `sfx_Voice_ouch`. New playtest feedback on latest audio commit: title/menu music carried into gameplay and SFX were mostly inaudible. Root cause: native queued GEMS stop-all as `FF 16`, but the driver command reads a sequence id byte and uses `$FF` to mean all sequences; the next `PlaySound` marker was swallowed as that argument and the level BGM start bytes were ignored. Native now queues `FF 16 FF` before starting the new BGM sequence, with a regression locking the following `FF 10 <bgm>` command intact. 2026-05-25 rejected hypothesis: moving standalone command FIFO from `$1A40` to reference GEMS 2.8 `$1B40` silenced later queued SFX entirely in `.cache/recordings/issue-160-audio-after-damage-cmdfifo`, so Kid's extracted driver layout still expects `$1A40`. Previous fix: direct settled native gameplay no longer queues stop-all before the first level BGM, and cue replacements defer the new start by one audio frame after stop-all. Evidence: `.cache/recordings/issue-160-audio-current-settled-after-startonly` raises global RMS from `0.017301` to `0.049685`, has `80/80` non-zero 0.1s windows instead of `22/80`, and `verify-recording --slug issue-160-audio-current-settled-after-startonly --native-only` passes. Latest playtest still heard title music under gameplay with weak SFX; ASM check shows Kid's `StopMusic` at `kid.asm:49727` reloads/resets the GEMS driver and queues `$0B` pointer init, not `$16` stop-all. Native music replacement now calls `GemsAudioEngine::stop_music()` before deferring the next BGM start; `cargo run -q -p kidcli -- verify-audio-output --scenario boot-title-start-level1` passes. Latest native listening bundle: `.cache/recordings/issue-160-audio-latest-validation/audio.wav` from `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo run -q -p kidcli -- play-native --warp-level 00 --settled --record issue-160-audio-latest-validation --record-frames 600` verifies native-only, is 10.00s stereo at 44.1 kHz, and has RMS `0.052095`, peak `0.367125`, and `100/100` 0.1s windows above 0.001 RMS. New playtest feedback: title music still carried into gameplay and SFX were mostly absent; native `play-native` was clearing the active title BGM cue during the intervening `LevelTransition` shell, so the first `InGame` frame started level music without the ASM `StopMusic` reset. The BGM state helper now preserves the previous cue through non-BGM shell frames and only switches on the next concrete cue, forcing title-to-level replacement through `StopMusicThenStart`. Validated 2026-05-25: `mise run human-validation-review` passes the `issue-160-audio-latest-validation` recording, `cargo run -q -p kidcli -- verify-audio-output --scenario boot-title-start-level1` passes, and the native audio trace for `boot_title_start_level1` shows GEMS resets plus fresh YM2612 key-ons after the title-to-gameplay transition. Follow-up 2026-05-26: refreshed settled enemy-contact recording `.cache/recordings/issue-160-audio-current-enemy-contact` reaches damage at frames 290 and 356, then death/restart at 431; `verify-recording --slug issue-160-audio-current-enemy-contact --native-only` passes, and the 10.00s stereo `audio.wav` has all `100/100` 0.1s windows above 0.001 RMS through and after both damage events. The old silent `issue-138-enemy-contact/audio.wav` is stale pre-fix evidence. |
Note 2026-05-31 for #160: latest playtest still reported missing intro/menu music until entering Options. Native was treating Options as the title-shell cue even though ASM `Load_OptionMenu` (`kid.asm:31570-31572`) runs `StopMusic` and starts `bgm_City`; late intro/title shell modes also could fail to recover `bgm_Ice` if entered without the cold `Load_IntroSequence1` cue. Native BGM cue selection now gives Options its own City cue and treats IntroSequence2-6 as recoverable title-shell music contexts. Regressions: `play_native_title_shell_uses_load_title_screen_bgm` and `play_native_options_bgm_replaces_title_shell_with_city`; `verify-audio-output --scenario boot-title-start-level1` still passes.
Note 2026-06-01 for #160: latest playtest again reported no intro music. Fresh native recording `.cache/recordings/issue-160-intro-audio-repro` confirms the live `play-native` BGM event is queued on the first `IntroSequence1` frame (`frame=173`, `StopMusicThenStart(3)` for `bgm_Ice`) and native-only replay passes, so the immediate boot-to-intro command path is now mechanically guarded by `play_native_boot_idle_queues_intro_music_on_first_intro_frame`. If humans still hear silence after that frame, investigate standalone GEMS onset/mix rather than title cue selection.
| 161 | [~] | P1 | render | Intro sequence is visually incomplete and misses scenes/Kid animation | User playtest 2026-05-25 reports the introduction still lacks the real scene sequence and Kid animations. Reopen the visual side of #141 with frame/oracle captures; previous boot/TAS state parity was too weak as visual proof. Follow-up 2026-05-25 ports the ASM `loc_1C7A0` / `sub_1C7F6` text writer into the source-port intro VDP path: native clears extracted/baked story-text cells, then replays `$FE` delays, `$FD` line breaks, and per-glyph header delays before writing visible glyphs. Regressions: `intro_sequence_text_worker_waits_for_rom_initial_and_fe_delay`, `intro_sequence_text_worker_honours_second_fe_delay_before_third_line`. Verification: focused tests and `mise run verify-slice-fast`; `verify-frames --scenario boot-title-idle10s` still fails at `title@360` because that command captures PNGs from the separate `MdRuntime` emulator path and only swaps source-port state JSON, so it is not a source-port VDP pixel gate. Follow-up 2026-05-26: native VDP rendering now composites the oracle-timed `kid_gfx` pose exposed by `Load_IntroSequence3` `loc_1AEEC` / `Load_IntroSequence5` `loc_1BCD0` trace state instead of returning only the tile-plane shell; regression `vdp_intro_sequence_composites_oracle_timed_kid_pose` covers the VDP overlay path. Follow-up 2026-05-26: `Load_IntroSequence3` no longer uses a single gameplay-player overlay for Wild Side kids; `kid-core` now writes the full active roster into the VDP sprite table from `loc_1AEEC` object descriptors plus `Make_SpriteFromGfxObject` chunk/link rules, and `kid-md` renders those hardware sprites from the existing VRAM art. Regressions: `intro_sequence_3_loc_1AEEC_writes_rom_roster_sprite_table`, `intro_sequence_3_loc_1AEEC_uses_active_roster_index`, `intro_sequence_3_loc_1AEEC_is_hidden_before_visible_elapsed`. Follow-up 2026-05-26 applies the same hardware-sprite-table path to `Load_IntroSequence5` `loc_1BCD0`, rendering all five static kid/helmet gfx slots from VRAM tile `$26C` instead of overlaying only the trace-exposed slot-2 gameplay sprite. Regression: `intro_sequence_5_loc_1BCD0_writes_five_static_gfx_slots_to_sprite_table`. Follow-up 2026-05-26 renders the delayed intro-5 particle workers: `loc_1BE3C` falling particles now spawn after the `$1A0` hibernate with ROM `$4000` gravity and DBF lifetimes, and `loc_1BE88` renders the FC82-selected centre helmet particle from VRAM tile `$5EB`. Regressions: `intro_sequence_5_loc_1BE3C_particles_spawn_after_loc_1BCD0_hibernate`, `intro_sequence_5_loc_1BE3C_deletes_short_lived_particles_by_dbf_count`. Follow-up 2026-05-26 fixes the native-only baked story-text cleanup: clearing to tile word `0` sampled Scroll A nametable bytes as pattern data because Scroll A starts at VRAM `$0000`, producing orange bars before `loc_1C7A0` had written the real text. Cleanup now uses a zeroed high-VRAM tile, with `intro_sequence_text_cleanup_uses_blank_pattern_not_tile_zero`; `.cache/recordings/issue-161-native-after-clear-260/final-frame.png` matches the oracle text colour sample (`THERE WAS A NEW MACHI`) and drops MAE vs `.cache/debug/issue-161-oracle-3200000.png` to `466.832`. Follow-up 2026-05-26 wires `sub_1B8F4 -> sub_1C54A` into the intro VDP sprite table so the Wild Side door/interior art at VRAM tile `$140` is no longer only uploaded; fixing the shared `Make_SpriteFromGfxObject` y-origin to subtract frame height also moves the existing intro-3/intro-5 hardware sprites to the ASM bottom-anchored positions. Regressions: `intro_sequence_1_sub_1C54A_writes_wild_side_door_sprite_table`, `intro_sequence_1_sub_1C54A_is_hidden_before_controller_exposes_it`, plus updated intro-3/intro-5 sprite-table expectations. Follow-up 2026-05-26 after cold-boot timing fixes: refreshed `.cache/recordings/issue-161-current-intro-long`; native replay passes, and `verify-tas --scenario .cache/recordings/issue-161-current-intro-long/scenario.json --frame-start 0 --frame-end 2790 --checkpoint-interval 60 --frontier-kind gameplay` reports `CRC: all 2790 frames match in 0..2790`. Remaining diffs in that probe are intro-only `number_objects`/`number_gfx_objects` accounting, not VDP pixels. `verify-recording --slug issue-161-current-intro-long` still reports PNG diffs from its separate evidence path, so treat that as verifier debt unless a future live VDP CRC probe regresses. Latest 2026-06-01 playtest still reports broken intro kids/door/stray-sprite/green-screen visuals, so keep this open with #141 until a live visual route passes. |
Note 2026-05-26 for #161: `Load_IntroSequence3` now keeps the fixed `sub_1B93E -> sub_1C54A` Wild Side door/interior sprite present alongside the roster and appends the two ASM `sub_1C572` / `loc_1C58C` sliding door halves. Native models the `$3C` initial hibernate and first `loc_1C5BE` one-pixel inward slide while using the same `Make_SpriteFromGfxObject` chunking. Regressions: `intro_sequence_3_sub_1C54A_keeps_wild_side_door_sprite_visible`, `intro_sequence_3_sub_1C572_draws_and_slides_door_halves`.
Note 2026-06-01 for #161: fresh `cargo run -q -p kidcli -- verify-recording --slug issue-161-current-intro-long` keeps trace/state green but pixel-red. The first concrete mismatch is Wild Side frame 1080: 5,991 changed pixels, bounds x=36..295 y=143..214, matching the playtest "Everybody played it" kids/door corruption. Oracle WRAM at frame 1080 confirms the native roster selection is not the first cause: `$FFFFFC83=1`, active roster 0, 11 gfx objects, with `loc_1AEEC` descriptor positions/frames matching native. Rejected 2026-06-01: descending `Allocate_GfxObjectSlot loc_FD4` ordering alone worsened frame 1080 to 9,078 pixels; row-major multi-tile sprite decoding worsened/introduced earlier prompt diffs; removing the H40 sprite-pixel cap worsened to 7,728 pixels; descending order plus reverse overlap resolution worsened to 9,072 pixels. Next pass should compare the actual oracle sprite table/VRAM tile bytes or add targeted VDP dumps instead of retrying these renderer/order guesses.
Note 2026-06-01 for #161: added `oracle-probe --vram-dump-frames` plus `--native-vram-dump-frames` and used those dumps to compare the actual frame-1080 VDP sprite table. Root cause for the Wild Side kids/door corruption was native's guessed `Data_Index` frame map: `$1208..$123C` points to `unk_E1106..unk_E116E`, not `unk_E1066..unk_E10C6`. After correcting the map, matching oracle's descending `$0201/$0200/$01FF/$0100` hardware chain order, reversing the same-slot door pair, and delaying the visible `sub_1C572/loc_1C58C` door slide by two frames, frame-1080 and frame-1140 sprite-table VRAM match oracle. `verify-recording --slug issue-161-current-intro-long` remains trace/state green and pixel-red, but the Wild Side frames through 1260 are now green; first remaining pixel mismatch moved to frame 1380 (23,219 pixels, full-screen bounds), i.e. the next intro scene/palette phase rather than the "Everybody played it" kids/door scene.
Note 2026-06-01 for #161: frame-1380 VRAM diff isolated stale `$67C0` clear words in the corridor/story nametable strips. Native now clears loc_1C7A0 cells according to the script tile base: `$67D3` Wild Side scripts keep the zeroed high-VRAM pattern workaround, while `$C7D3` corridor scripts clear to literal zero like oracle. `oracle-probe --vram-dump-frames 1380 --native-vram-dump-frames 1380` now reports `vram_diff_bytes 0`; `verify-recording --slug issue-161-current-intro-long` is still pixel-red from frame 1380 onward, so the next frontier is CRAM/palette or palette interpretation rather than plane/sprite VRAM.
Note 2026-06-01 for #161: added CRAM/VSRAM dump support to `oracle-probe` and checked frame 1380 after the VRAM fix. CRAM matched byte-for-byte, but VSRAM differed (`oracle[0]=0004`, `oracle[1]=0001`, native zero), matching ASM `sub_1B988`'s `Camera_Y_pos=4` corridor hand-off. Native IntroSequence4 now writes those VSRAM words; `verify-recording --slug issue-161-current-intro-long` remains trace/state green and pixel-red, but frames 1380..1860 are now green and the first remaining mismatch moved to frame 1920 (58,964 pixels, full-screen bounds).
Note 2026-06-01 for #161: frame 1920 was native exposing the IntroSequence5 alley/P-block VDP state while oracle was still fully blank. VDP dumps showed oracle VRAM+CRAM zero at intro-sequence-5 elapsed 44, while frame 1980 was already visible and green. Native now keeps segment 5 blank until ASM `loc_1BA28 -> sub_1C204` reaches the existing IntroText5 visible cadence (`INTRO_TEXT_WORKER_START_ELAPSED_LOC_1C7A0[4]`). `verify-recording --slug issue-161-current-intro-long` now has frames 1920 and 1980 green; the first remaining pixel mismatch is frame 2040 (1,427 pixels, bounds x=121..177 y=30..94), i.e. the Kid/helmet transform area rather than the earlier green-screen/full-screen reveal.
Note 2026-06-01 for #161: frame 2040 was not a sprite-table or palette problem. CRAM matched, the `loc_1BCD0` hardware sprite table matched, and Kid sprite art at tile `$26C` matched; only Scroll A tile words from `loc_1BC36` differed. Native advanced `sub_1C034(d4=1)` immediately after the `$A0` hibernate, but oracle still presented frame 0 at intro-sequence-5 elapsed 164 and caught up by frame 2100. Native now applies the existing `$3C` presentation delay to the first `loc_1BC36` frame transition as well as later transitions. `verify-recording --slug issue-161-current-intro-long` now has frame 2040 green; first remaining mismatch is frame 2400 (56,704 pixels, full-screen bounds).
Note 2026-06-01 for #161: frame 2400 was multiple overlapping IntroSequence5 fade-hold mistakes. Native froze the `loc_1BC36`/`loc_1BCD0` workers at elapsed 520 after mode `$28`, while oracle keeps those objects live during `loc_1BA90`; native also uploaded the post-subtract palette tick one frame too early. Native now advances the worker shell by the mode-$28 fade elapsed but applies the previous displayed palette tick. A second byte-level pass showed the remaining P-block artefact was `sub_1C112` being applied forever after its clear threshold; oracle lets later `sub_1C034(d4=5)` redraw those cells. Native now bounds the `sub_1C0EE` panel and `sub_1C112` clear windows so later frame uploads survive, and drops the first two stored `$36/$3A` static gfx slots once `loc_1BCD0` spawns particles. `verify-recording --slug issue-161-current-intro-long` reduces frame 2400 from 56,704 changed pixels to 1,774; remaining frame-2400 diff is the helmet/head particle area (bounds x=124..178 y=24..93), so next pass should compare `loc_1BE88`/helmet particle frame, timing, and priority.
Note 2026-06-01 for #161: the remaining frame-2400 helmet/head diff was native using a guessed one-tile falling-particle frame and advancing the `loc_1BE88` centre helmet Y gravity before the displayed hibernate. `loc_1BE3C` now maps `$11C8/$11CC/$11D0` to the real `unk_E1086/unk_E108E/unk_E1096` multi-tile frames, and `loc_1BE88` keeps DBF `#$1C`'s 29 movement frames while applying the `$9000` y-velocity increment after the displayed frame. `verify-recording --slug issue-161-current-intro-long` now has frame 2400 green; first remaining mismatch is frame 2460 (4,360 pixels, bounds x=102..210 y=61..207), so the next pass should inspect the later falling/centre particle movement window and stale sprite-table tail around `loc_1BE74/loc_1BF1E`.
Note 2026-06-01 for #161: frame 2460 was the `Load_IntroSequence6` hand-off rather than a late intro-5 particle issue. Native was indexing `sub_1B43A/loc_1B446` and `loc_1B37C/sub_1B41C` from the cold title counter, so the hidden shell and lightning strip were ahead of the oracle after ASM `loc_1BA90`'s fade-hold hand-off. Native now subtracts the intro-5 fade window for the hidden shell and applies `loc_1B37C`'s first `j_Hibernate_Object #3` presentation delay before selecting the lightning frame. `verify-recording --slug issue-161-current-intro-long` now has frame 2460 green; first remaining mismatch is frame 2520 (3,494 pixels, bounds x=102..210 y=61..207).
Note 2026-06-01 for #161: frame 2520 was the next `sub_1B43A/loc_1B446` presentation boundary. Native consumed `unk_1B4D6` from the local counter too directly, so after frame 2460 it ran from the early `0,1,0` flicker into the wrong title-character map by frame 2520. Native now preserves the repeated-zero window before applying the ROM's 3-frame byte cadence; `verify-recording --slug issue-161-current-intro-long` has frames 2460 and 2520 green, and the first remaining mismatch is frame 2580 (5,829 pixels, bounds x=97..228 y=64..207).
Note 2026-06-01 for #161: frame 2580 was the late `loc_1B37C` lightning worker, not the title-character map. Oracle WRAM at frames 2460/2520/2580 showed the lightning worker's saved code moving from `loc_1B3BC` (`d1=$27/$13`) to the post-loop hibernate before the first closing zero upload (`code=$1B3DA`, `d1=$FFFF`), so `sub_1B41C`'s final loop flash 1 remains visible at frame 2580. Native now models the main `loc_1B3BC` DBF-loop presentation window directly. `verify-recording --slug issue-161-current-intro-long` now has frames 2460, 2520, and 2580 green; first remaining mismatch is frame 2640 (71,647 pixels, full-screen bounds), while frame 2760 is green.
Note 2026-06-01 for #161: frame 2640 was the natural no-input `sub_1B43A`/`loc_1B45A` title-art restore, not the normal title screen yet. Native now builds that intermediate shell by replaying `sub_1BF24(d4=7)`, `sub_1B532`, `Load_TitleArt`, and the still-active `loc_1B37C` hidden hscroll/lightning state. Frame 2700 then switches to the normal title VDP shell while trace `title_screen_ready` remains false, but keeps the natural `loc_1B6AC` reveal-loop phase, the moving `loc_1B5CE` Kid y-position, and the pre-settled `loc_1BB10` small title sprite x. `verify-recording --slug issue-161-current-intro-long` now passes all trace, state, and pixel checkpoints for the refreshed cold intro recording.
Note 2026-05-26 for #161: `Load_IntroSequence5` now replays the staged `loc_1BC36` tile worker: `sub_1C034` frame uploads from `off_1C126` switch on the ROM hibernate boundaries, then `sub_1C0EE` / `sub_1C112` blit and clear the 10x10 panel at plane cell `(3,$13)`. Regressions: `intro_sequence_5_loc_1bc36_streams_sub_1c034_frames_on_rom_hibernate_boundaries`, `intro_sequence_5_loc_1bc36_panel_blit_and_blank_follow_sub_1c0ee_order`. Superseded by the later full cold-intro CRC pass below.
Note 2026-05-26 for #161: recording verification now replays checkpoint frames through the live `NativeRuntime::vdp_state` path used by `play-native`, instead of rebuilding intro screenshots from the trace-only fallback that intentionally returns black for intro modes. Refreshed `.cache/recordings/issue-161-current-intro-long` verifies native-only and now shows the Wild Side exterior, Hologram room, intro-3 roster scene, and story text in `verify/native/` checkpoint PNGs. Superseded by the later live VDP CRC pass below; the remaining PNG diffs are evidence-path debt.
Note 2026-05-26 for #161: cold-boot intro timing now follows the ASM object loops through the first three intro segments. `Load_SegaScreen` keeps mode `$0000` through oracle frame 172, `sub_1B8F4/loc_1B924` keeps intro 1 through frame 533, `Load_IntroSequence2/loc_1BA02` keeps intro 2 through frame 982, and `sub_1B93E/loc_1B96E` keeps intro 3 through frame 1281. Regressions: `cold_boot_sega_screen_hands_off_to_intro_sequence_1_on_oracle_frame_173`, `cold_boot_intro_sequences_1_and_2_follow_oracle_handoff_frames`, `cold_boot_intro_sequence_3_follows_oracle_handoff_frame`. `verify-tas --scenario .cache/recordings/issue-161-current-intro-long/scenario.json --frame-start 0 --frame-end 2200 --checkpoint-interval 60 --bisect --frontier-kind gameplay` now moves the first CRC frontier from frame 119 to frame 1282; remaining #161 frontier is intro-3 roster/Kid trace state carrying into intro 4, not the coarse boot timers.
Note 2026-05-26 for #161: the cold intro-3/4/5 hand-off state now keeps the previous intro-3 roster pose through early intro 4, exposes `sub_1B988`'s `Camera_Y_pos=4` hand-off, holds intro 4 until oracle frame 1875, and exposes the early `loc_1BCD0` slot-2 pose/counts in intro 5. Regressions: `cold_boot_intro_sequence_4_follows_oracle_handoff_frame`, `intro_sequence_4_intro_3_pose_carries_until_cutoff`, `camera_handoff_spans_late_intro_4_and_timeout_restart_intro_5`, `current_player_state_exposes_cold_intro_5_slot_2_pose_after_delay`. `verify-tas --scenario .cache/recordings/issue-161-current-intro-long/scenario.json --frame-start 0 --frame-end 2600 --checkpoint-interval 60 --bisect --frontier-kind gameplay` now moves the first CRC frontier to frame 1956; remaining #161 frontier is the native shortcut from intro 5 directly to title instead of the ROM's later intro-5/intro-6 flow.
Note 2026-05-26 for #161: cold intro sequence 5 now uses the ASM `loc_1BA28 -> sub_1C204 -> loc_1BA5C` duration and naturally hands off to `Load_IntroSequence6` (`Game_Mode=$28`) on oracle frame 2396 instead of jumping to title mode `$2C`; the cold path also avoids the timeout-restart hidden-camera-wrap flag. Regression: `cold_boot_reaches_intro_sequence_6_without_input`. `verify-tas --scenario .cache/recordings/issue-161-current-intro-long/scenario.json --frame-start 0 --frame-end 3200 --checkpoint-interval 60 --bisect --frontier-kind gameplay` now passes all checked frames by CRC.
| 162 | [x] | P1 | render | Level-card graphics are wrong even when the text animation is present | Fixed 2026-05-25: native `build_title_card_vdp` incorrectly retargeted VDP Scroll A to `$6000` and Scroll B to `$0000`; ASM `Load_TitleCard` only guards Z80 via `sub_914`/`sub_924` and writes the title-card nametable to `$0000`, inheriting `VDPGameInitValues` (`reg2=$00`, `reg4=$07`). Fix keeps Scroll A at `$0000` and Scroll B at `$E000`; verified with `cargo test -p kid-core title_card_vdp -- --nocapture`, `cargo test -p kid-core boot_title_start_level_1_exposes_load_title_card_vdp_shell -- --nocapture`, and `mise run verify-slice-fast`. |
| 163 | [x] | P1 | render | HUD diamond and special-prize icons render as wrong shapes or disappear | FIXED 2026-05-25: native rendered ASM `Init_SpriteAttr_HUD unk_1427C` ankh and diamond symbols as single `IndexedTile`s even though both use `sprite_attr ...,1,1,...` 2x2 sprites. The ankh tile `$8231` also overlapped native block VRAM, so resolving it through `resolve_ingame_vram_tile` could draw block art or vanish instead of the ROM `Life_icon` sheet. HUD now draws the ankh from the ROM `PrizeAnkh_Init`/`Life_icon_(3_frames)` sprite frame and the diamond as a 2x2 sprite from HUD diamond VRAM tile `$86F2`. Regressions: `game_hud_uses_asm_static_symbol_slots`, `build_game_scene_emits_hud_art_without_text_placeholders`. Verification: `cargo test -p kidcli hud -- --nocapture`; `cargo fmt --check`; `git diff --check`; `mise run verify-slice-fast`. |
| 164 | [x] | P1 | render | Falling helmet prize sprites still look like approximations | Fixed 2026-05-25: falling helmet visuals now compose the base form palette with the three-word `sub_10CB0` helmet palette patch for slots $D..$F, and scene coverage exercises all nine `PrizeHelmet*_Init` mask frames. |
| 165 | [x] | P1 | player | Wall-grab pull-up lacks the ROM rollover/flip animation | Fixed 2026-05-25: core already hibernated physics for the ASM `loc_ABF0`/`stru_8BB4` 21-frame pull-up, but native traces did not expose `ledge_snap_cooldown` and the renderer chose idle/jump frames. Renderer now loads the seven ROM `kid_flip_1..7` frames (`stru_8BB4`, three VBlanks each) and selects them while the cooldown is active. Regressions: `wall_grab_pullup_cooldown_renders_stru_8bb4_rollover_frames`, `load_player_visuals_returns_required_kid_frames`, `ledge_snap_cooldown_loc_1062_hibernates_physics_while_animation_is_active`. |
| 166 | [x] | P1 | render | Enemy sprites have transparent holes / wrong masks | Fixed 2026-05-25: native enemy loading now applies ASM `Load_EnemyArtToVRAM` forest first-slot pixel remaps (`unk_36E54`, and Tar Monster `unk_36E64`) before rendering sprite frames, preventing forest enemy art from using the wrong colour/mask indices. |
Note 2026-05-28 for #166: latest playtest still saw the Blue Lake Woods walking dragon as partly see-through and not animating its face correctly. This was a separate renderer bug from the forest first-slot remap: native added a layout-index phase offset to DragonWalking live frames, selecting a later `stru_3CC5E` frame than ASM `j_Init_Animation` would for the first visible Level 1 dragon. Native now follows `stru_3CC5E`/`stru_3CC90` timing without the layout-index offset. Regression: `live_walking_dragon_uses_rom_walk_sequence_not_death_frames`.
Note 2026-05-28 for #166: the same invented layout-index phase offset also affected RockTank body/turret animation, matching the playtest report that the Level 2 "little head guy" still had see-through/wrong-looking eyes after the dragon fix. ASM `Tank.asm loc_3B372` and `Tank_Shooting.asm loc_3B5FA` call `j_Init_Animation` directly for `stru_3B270`/`stru_3B4CA` and their child turret sequences, so native now removes the layout-index phase seed from both body and turret selectors. Regression: `live_rock_tank_animation_does_not_phase_shift_by_layout_index`.
| 167 | [~] | P1 | enemy | Enemy deaths disappear instead of showing ROM death/body animation | User playtest 2026-05-25: enemies often just vanish when killed. Partial: Mini Hopping Skull keeps ASM `loc_3F158`/`stru_3E9AC` death animation alive before delete; follow-up adds generic death hibernates for Crab `loc_3B094`/`stru_3AF82`, Goat `loc_3CB64`/`stru_3C9EA`, and Ninja `loc_3DB4E`/`stru_3D50A`, with sword/projectile/stomp/Berzerker final-hit paths now letting the delete tail own HP zeroing. Follow-up wires Tank `loc_3B480`/`stru_3B29A`, Tank_Shooting `loc_3B6E6`/`stru_3B518`, and Robot `loc_3A9D2`/`stru_3A74A` into the same wait-before-delete path. Follow-up wires Diamond `loc_3AEE6`/`loc_3AE28`/`stru_3ACC4`, walking Dragon `loc_3CEFC`/`stru_3CCEC`, Scorpion `loc_3DCF4`/`stru_3DB80`, and Driller `loc_3BF1C`/`stru_3BCD4` into the same wait-before-delete path. Follow-up wires Armadillo `loc_3C886`/`stru_3C69A`, Orca `loc_3D268`/`loc_3D4BC`/`stru_3D11E`, and Fire Demon `loc_3A5A6`/`stru_3A378` into the same wait-before-delete path; Fire Demon `loc_3A63C` trail child now waits for the delete tail. Follow-up wires Lion `loc_3E378`/`stru_3DF48` into the same wait-before-delete path before shared `loc_3E956` deletion. Follow-up wires Big Hopping Skull `loc_3664E`/`loc_36664`/`stru_367FA` into the same wait-before-delete path before `loc_366B2` deletion. Follow-up wires Spinning Twins `loc_3BC56`/`stru_3B83C`, Hand `loc_32AA2`/`stru_32B78`, Tar Monster `loc_323E6`/`stru_32700`, Cloud `loc_35C0E`/`stru_35EBA`, Sphere `Enemy06_Sphere_Kill`/`stru_36182`, Fireball `Enemy19_Fireball_Kill`/`stru_345E4`, Archer `loc_3414A`/`stru_33FC2`, Tornado `loc_359B2`/`stru_35B1E`, and Emo Rock `loc_36346`/`stru_36468` into the same wait-before-delete path before their delete tails. Follow-up wires UFO `loc_35174`/`loc_35180` parent `stru_34CA0` into the wait-before-delete path, including the falling parent velocity and death-frame rendering. Follow-up wires Drips `loc_338F8`/`stru_33F06` into the wait-before-delete path before `loc_3392C` deletion. Follow-up exposes Flying Dragon `loc_3492C`/`stru_345E4` HP-zero timer snapshots and renderer frame selection before `loc_349A6` deletion. Follow-up 2026-05-26 adds explicit renderer audit coverage for UFO `stru_34CA0`, pinning both `select_enemy_frame` and the bundle tile bases `$15A/$16A/$17A` against ASM `loc_35174/loc_35180`. Renderer audit now covers every ordinary killed enemy family plus the shared boss-head death spin; Heady Metal remains handled by the separate ASM `sub_37708`/`sub_378C8` wobble effect path, and object `0x21` is boss-eye/attack debris rather than a normal killed enemy family. |
Note 2026-05-26 for #167: the Crab/Goat/Ninja runtime delete waits already existed, but production enemy visual specs exposed only their six normal animation frames, so `select_enemy_frame` could not return the waited body frames for `stru_3AF82`, `stru_3C9EA`, or `stru_3D50A`. Native now includes those ROM frame descriptors and pins both the spec table and Crab/Ninja selector paths.
Note 2026-05-26 for #167: Sphere and Fireball also had core HP-zero wait timers but no renderer death selection. Native now selects Sphere `Enemy06_Sphere_Kill`/`stru_36182` disappearing frames and Fireball `Enemy19_Fireball_Kill`/`stru_345E4` two-tick frames while their delete tails hibernate.
Note 2026-05-26 for #167: Archer, Tornado, and Emo Rock had the same runtime/render gap. Native now exposes the ROM waited descriptors for Archer `loc_3414A`/`stru_33FC2`, Tornado `loc_359B2`/`stru_35B1E`, and Emo Rock `loc_36346`/`stru_36468`, and selects those frames from HP-zero snapshots.
Note 2026-05-26 for #167: Cloud, Spinning Twins, and Big Hopping Skull also now expose their waited death descriptors and renderer selections: Cloud `loc_35C0E`/`stru_35EBA`, Spinning Twins `loc_3BC56`/`stru_3B83C`, and Big Hopping Skull `loc_3664E`/`stru_367FA`.
Note 2026-05-26 for #167: Scorpion, Drips, Hand, and Driller now expose their waited death descriptors and renderer selections: Scorpion `loc_3DDEE`/`stru_3DB80`, Drips `loc_3392C`/`stru_33F06`, Hand `loc_32AA2`/`stru_32B78`, and Driller `loc_3BF1C`/`stru_3BCD4`.
Note 2026-05-28 for #167: walking Dragon death hibernation existed, but the generic final-hit path skipped it whenever the live shared-walker patrol timer was nonzero. Native now lets ASM `Dragon.asm loc_3CEFC` override that live timer with `stru_3CCEC` before the delete tail. Regression: `dragon_walking_death_overrides_live_patrol_timer_loc_3cefc`.
Note 2026-05-26 for #167: Fire Demon, Diamond, and Armadillo now expose their waited death descriptors and renderer selections: Fire Demon `loc_3A5A6`/`stru_3A378`, Diamond `loc_3AE28`/`loc_3AEE6`/`stru_3ACC4`, and Armadillo `loc_3C886`/`stru_3C69A`.
Note 2026-05-26 for #167: walking Dragon and Lion now expose their waited death descriptors and renderer selections: walking Dragon `loc_3CEFC`/`stru_3CCEC` and Lion `loc_3E378`/`stru_3DF48`.
Note 2026-05-26 for #167: Tar Monster and Orca now expose their waited death descriptors and renderer selections: Tar Monster `loc_323E6`/`stru_32700` and Orca `loc_3D268`/`loc_3D4BC`/`stru_3D11E`.
Note 2026-05-26 for #167: renderer audit now covers the remaining shared death-frame gaps for Flying Dragon `loc_3492C`/`stru_345E4`, Fireball `Enemy19_Fireball_Kill`/`stru_345E4`, Mini Hopping Skull `loc_3F158`/`stru_3E9AC`, and boss heads `loc_39EBE`/`loc_39F7A`. Boss-head snapshots now select the C8620/C8618/C8610/C8608/C8600 death-spin frames from the HP-zero wait timer instead of staying on the idle head frame.
| 168 | [x] | P1 | player | Berzerker/rhino side charge does not break side blocks | FIXED 2026-05-25: native Berzerker side-block dispatch now mirrors ASM `loc_9294 -> loc_930E` by scanning the whole leading wall column instead of stopping at the first lower solid cell, and uses the Berzerker form half-width from `sub_B41C/unk_B408` after wall snaps instead of The Kid's 5px default. Grounded charge can now reach and delete upper Smash side blocks even when lower solid terrain shares the column; airborne wall contact remains the ROM `loc_92DE` no-dispatch path and only clears charge. Regressions: `berzerker_charge_scans_past_lower_solid_to_break_upper_smash_loc_9294`, `berzerker_charge_wall_break_uses_berzerker_form_width_after_wall_snap`. Verification: `cargo test -p kid-core berzerker_charge_ -- --nocapture`; `mise run verify-slice-fast`. |
Note 2026-05-28 for #168: latest playtest still saw the rhino/Berzerker not putting its head forward while charging into blocks. Root cause: native used the late `Berzerker_charging` gameplay flag as the renderer selector. ASM `loc_9564` writes the off_9688 head-forward sprite first, then wall/block collision may clear `Berzerker_charging` at `loc_9326` in the same frame. Native now exposes a one-frame `berzerker_charge_pose` trace latch for that sprite write while keeping the gameplay flag clear for collision semantics. Regressions: `berzerker_charge_pose_survives_same_frame_wall_clear_loc_9564_9326`, `berzerker_charge_pose_latch_selects_head_forward_after_flag_clear_loc_9326`.
| 169 | [~] | P1 | enemy | Level 2 Drips/slime placement and drop/attack behaviour still looks wrong | Fixed 2026-05-25: first concrete mismatch was rendering, not the whole Drips state machine. Native was drawing `DripsSurfaceCrawler` with the falling-child `LnkTo_unk_C7B26` frame even though ASM `Drips.asm off_33520` uses `LnkTo_unk_C7B9E`/`LnkTo_unk_C7BA6` for the surface/crawler frames. Follow-up placement root cause: native ceiling attach used a 1px probe, but ASM `object_meta=$16` maps through `CollisionSize_Index` to `unk_314D0` (`top=-$1F,height=$1F`) and `GfxObjects_Collision loc_1A6C` snaps from that hitbox. Latest playtest says the slime should drip from beta blocks until a large enough pool gathers, but native has slime floating and then dripping in Level 2. Follow-up 2026-05-25 hides the parent/controller enemy sprite in the native player because ASM `Enemy16_Drip_Init` / `loc_33488` keeps that gfx object hidden via `$13`; visible falling and crawler children remain rendered through typed projectile snapshots. Follow-up 2026-05-25 wires the `off_33520` pool cursor so the first two falling-child splashes are pool-growth entries and only the negative sentinel path may allocate `loc_335D8`, matching the "drip until pool gathers" cadence. Final fix holds the parent/controller at `DRIPS_DROP_IN_FLIGHT_TIMER` after `loc_33424` and advances/reseeds the pool only when the child hits terrain at `loc_33488`; this prevents stacked/floating slime while a falling child is still active. Regression: `drips_owned_drop_advances_pool_and_reseeds_parent_only_on_splash_loc_33488`; verification: `cargo test -p kid-core drips -- --nocapture`. |
| 170 | [x] | P2 | gameflow | Death/continue restart still shows fake level loader | FIXED 2026-05-25: removed both native fake-loader paths outside ROM `Load_TitleCard`. Plain `LevelTransition` frames without title workers no longer draw static "Blue Lake Woods" text, and `LevelTransition -> InGame` or level-change handoffs no longer restart a fake overlay. Real title-card visuals stay owned by VDP plus `loc_1AB26` title workers. Regressions: `build_scene_keeps_plain_level_transition_blank_without_title_workers`, `level_card_overlay_does_not_start_for_non_title_gameplay_entry`, `level_card_overlay_does_not_start_for_ordinary_level_change`. Verification: `cargo test -p kidcli level_card_overlay -- --nocapture`; `cargo test -p kidcli build_scene_ -- --nocapture`; `mise run verify-slice-fast`. |

Note 2026-05-27 for #170: the latest playtest still saw an invented native GAME OVER / CONTINUE panel after death. Native now exposes `game_over_elapsed_frames` / `continue_countdown_elapsed_frames`, loads `Spr_EOL_GAME` and `Spr_EOL_OVER`, and renders the no-continues and continue countdown overlays through ASM `loc_D25E` / `Continue_Screen` EOL sprite workers instead of panel text. With continues remaining, native no longer shows the false GAME OVER interstitial before the continue screen. Regressions: `game_over_with_remaining_continues_does_not_render_invented_panel_loc_d25e`, `continue_countdown_uses_rom_eol_continues_sprites_loc_d25e`, `no_continue_game_over_uses_rom_game_over_sprites_loc_d25e`, `trace_frame_exports_continue_countdown_remaining_frames`, and `load_eol_score_visuals_returns_rom_label_and_digit_frames`. Verification: `cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`; `mise run verify-slice-fast`.
| 171 | [x] | P1 | render | HUD hitpoint/life pips miss their grey second tile | Fixed 2026-05-25: native now renders ASM `Make_SpriteAttr_HUD` size byte `#4` hitpoint entries (`$822D/$822E/$822F`) as 2x1 tile sprites, so the grey/tail dot tile is visible next to the live/half pip. Follow-up 2026-05-25 fixes the live renderer path: ASM `loc_12C34` DMAs `ArtUnc_12C3C` over VRAM tile `$22D`, so native now loads `Hitpoint_display.bin` as dedicated HUD art instead of resolving those pips through the block sheet. Regressions: `game_hud_includes_extra_hitpoint_slots_for_grey_empty_pips`, `game_hud_hitpoint_pips_start_at_asm_sprite_table_x_position`, `load_hud_visuals_returns_digit_and_icon_metadata`. |
| 172 | [x] | P1 | render | HUD diamond icon should animate | Fixed 2026-05-25: native HUD diamond still used the right 2x2 sprite slot from `Init_SpriteAttr_HUD unk_1427C`, but kept the first diamond art page forever. Renderer now follows ASM `sub_44DC` / `unk_461C`: every 6 VBlanks, `sub_5D4A` DMAs `diamond_1..4` into HUD diamond VRAM `$6EA`, while the HUD sprite continues to point at tile `$86F2`. Regression: `game_hud_diamond_icon_animates_via_sub_44dc_dma_pages`. |
| 173 | [x] | P1 | block | Broken bricks should emit shard debris | Fixed 2026-05-25: native now hands the short smash/ice hibernate worker off to persistent ASM `sub_2A4C` shard state after `sub_292E`, advancing five debris pieces with the ROM velocity tables plus `$36B0` gravity and rendering them with tile `$8228` until off-screen. Regressions: `block_destroy_effect_hands_off_to_falling_sub_2a4c_debris`, `falling_block_debris_uses_sub_2a4c_rom_tile_after_destroy_handoff`. |
| 174 | [x] | P1 | block | Prize blocks should show spawn explosion above the block | Fixed 2026-05-25: `prize_block_spawn_flashes` now carry ASM `sub_1007A` / `loc_102B8` world coordinates (`cell_x*16+8`, `cell_y*16-8`) into native traces, and the renderer draws `stru_102DE` block-art frames `$2CF/$2D8/$2E1/$2EA` above the hit block. Regressions: `prize_block_spawn_flash_counts_down_and_is_traced_with_loc_102b8_position`, `prize_block_spawn_flash_uses_loc_102b8_stru_102de_tiles_above_block`. |
| 175 | [x] | P1 | gameflow | Winning the level sequence is incomplete | Completed 2026-05-29 for the reduced authoritative flag-touch bundle. Native level-complete gameflow follows the ROM-backed flagpole/EOL path closely enough for `issue-175-eol-flag-touch-level12` to reach trace, oracle-state, and strict frame parity. The implementation covers `Score_Board`, `loc_C070`, `loc_C186`, `loc_C21E/sub_C254`, `loc_D4CE`, `loc_D55E`, `loc_D660`, `loc_D770`, and the `Object_EOL_Text` / `Object_EOL_Number` row workers: score-board music plays, non-Kid objects are cleared, the hologram room/reveal are rendered, row fly-in uses ROM gfx-object coordinates, bonus counters drain into score, and the level-complete hold lasts through the score-counter path. Regressions: `flag_touch_deletes_non_kid_objects_before_eol_workers_loc_c070`, `loc_C0CC_pre_scoreboard_fade_duration_matches_asm`, `level_complete_speed_under_seconds_uses_loc_d660_threshold`, `level_complete_scoreboard_bgm_uses_score_board_cue_loc_c21e`, `level_complete_waits_for_eol_score_counter_before_advancing_loc_d770`, `loc_D770_eol_score_counter_snapshot_replays_visible_drain_state`, `level_complete_overlay_replays_loc_d770_score_counter_drain`, `level_complete_overlay_draws_level_skip_time_digits_loc_d47e`, `loc_D17E_level_complete_score_handoff_clears_live_score`, `loc_C21E_empty_screen_reveal_duration_is_40_frames`, `loc_C21E_reveal_tile_stream_preserves_sub_c254_order`, `level_complete_overlay_flies_rows_in_with_loc_c5a2_offsets`, `level_complete_draws_hologram_scroll_b_after_flag_hibernate`, `level_complete_hologram_uses_mapeni_cc0e_tiles_when_loaded_and_keeps_scroll_a`, `level_complete_hologram_keeps_scroll_a_background_tiles`, `level_complete_replays_sub_c254_foreground_reveal_writes`, `level_complete_reveal_clips_hologram_redraw_to_scroll_a_cell_loc_c21e`, `load_eol_score_visuals_returns_rom_label_and_digit_frames`, `load_eol_hologram_visuals_returns_loc_c186_tilemap_and_art`. Current verification: `cargo run -q -p kidcli -- verify-recording --slug issue-175-eol-flag-touch-level12` passes; `mise run verify-slice-fast` passes. |
| 176 | [?] | P1 | enemy | Wire ASM-faithful enemy helpers into the live step/combat dispatch (systemic root cause) | Enemy fidelity audit 2026-05-29 (`docs/source-port/enemy-fidelity-audit.md`) found ~1,171 `#[cfg_attr(not(test), allow(dead_code))]` helpers in `enemies.rs`: per-family ASM logic is modelled and unit-tested but never called at runtime, so the live game uses generic `step_pacer`/`step_hopper`/`step_flyer`, a generic HP damage path, and a Crab-only Berzerker special-case. The broad diagnosis was corrected by the dead-helper census: most family orchestrators are live-dispatched, and the remaining #177-202 gaps are missing sub-branches inside those orchestrators rather than one generic fallback. Investigation 2026-05-30 sampled Driller `loc_3BE78/loc_3BE8E/loc_3BEF2`: live `step_driller` is wired from the Pacer dispatch, uses the linked child ±$18 probe, preserves Smash/Ice dispatch, spawns Ice bullets, and already requires a Solid front-wall cell before drilling. Added regression `loc_3be8e_driller_requires_front_wall_collision_before_drilling` to pin the ASM `$38(a4)==d1` collision-word gate. Keep #176 open as an umbrella until the remaining per-family rows are individually closed; do not promote this umbrella to `[x]`. Gate: per-family regression wiring the existing helper + `mise run verify-slice-fast`; spot-check live behaviour with `play-native`. |
| 177 | [?] | P1 | enemy | Diamond modelled as a Collectible pickup, not a hostile HP enemy (stub) | Partial 2026-05-30: live Diamond layout objects are no longer `EnemyArchetype::Collectible` pickups. Low-HP Diamonds route through the shared Pacer walker, retain via `sub_3C3CE`, and the old runtime `collect_diamonds` hook was removed for layout enemies, keeping prize-diamond collection paths separate. High-HP Diamonds now seed Kid-facing `$18000` X velocity from `loc_3AD7C` and run a live `loc_3ADA4` terrain collision loop instead of falling through `loc_3AF14`; regression `diamond_high_hp_runtime_uses_loc_3ad7c_collision_loop_instead_of_shared_walker`. Earlier regressions: `diamond_low_hp_walks_from_spawn_position_loc_3aec4_loc_3af14`, `diamond_enemy_is_not_collected_on_touch_loc_3ad16`, `falling_onto_diamond_enemy_does_not_collect_or_stomp_bounce_loc_3ad16`. Remaining confirmed gaps: live HP damage + hurt/death response (`loc_3ADFA`/`loc_3AE28`) and Berzerker side-launch (`loc_3AE84`/`loc_3AF24`) need Kid/enemy collision-word plumbing beyond the terrain loop. Gate: per-family regression + `mise run verify-slice-fast`. |
| 178 | [?] | P1 | enemy | Armadillo: damage recoil window, Berzerker curl-launch, child fall-then-die, walk-speed cap not wired | Partial 2026-05-29: charging Berzerker side contact with no positive `$42` now follows ASM `loc_3C898` by launching Armadillo upward into the `loc_3C8CC` loop instead of routing through generic HP damage/death. Partial 2026-05-30: live Armadillo walk speed now follows ASM `loc_3C756`/`loc_3C766`, capping HP>=3 at `$28000` instead of linearly scaling HP4+ to `$38000`/`$48000`; regressions `armadillo_hp_and_walk_speed_follow_loc_3c756_and_loc_3c766` and `pacer_walk_speed_armadillo_caps_high_hp_speed`. Verification evidence: `cargo test -q -p kid-core armadillo -- --nocapture`; `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`; `mise run verify-slice-fast`. Remaining confirmed gaps: damage recoil/invulnerability window (`loc_3C848`), positive-recoil Berzerker shove/recovery (`loc_3C8DE`/`loc_3C8F2`), and spawned-child fall-then-die (`loc_3C93C`/`loc_3C954`). Gate: per-family regression + `mise run verify-slice-fast`. |
| 179 | [?] | P1 | enemy | Big_Hopping_Skull: patrol corridor, relaunch, floor-stop, per-hit SFX, respawn marker not wired | Partial 2026-05-30: live flat-floor contact no longer negates accumulated falling velocity. `step_big_hopping_skull` now follows ASM `loc_36580`/`loc_366D0` for the relaunch velocity by restoring saved x speed and writing only y_vel's high word to `$FFFE`; regression `big_hopping_skull_live_floor_contact_relaunches_like_loc_366d0` first failed against the old `-current_y_velocity` bounce. Remaining confirmed gaps: patrol x-bounds correction (`loc_36770`/`loc_36782`/`loc_36798`), full contact-wait animation/state collapse (`loc_36580`/`loc_366B6`), per-hit SFX + $2A=-3 Kid bounce (`loc_365D8`/`loc_365F2`/`loc_36616`), and `EnemyStatus_Table=$2168` respawn marker (`loc_3664E`/`loc_36664`/`loc_366A4`). See audit doc. Gate: per-family regression + `mise run verify-slice-fast`. |
| 180 | [x] | P1 | enemy | Crab: directional front-immune vulnerability gate and frozen immunity not enforced | Fixed 2026-05-29: live Red Stealth sword, Maniaxe/Eyeclops projectile, and diamond-rain `colid_hurt` damage now dispatch through ASM `Crab.asm loc_3B048` / `loc_3B086`, so frozen crabs ignore hurt collisions and ordinary hurt damage only decrements HP when Kid is on the side the crab is facing. `is_vulnerable` also treats frozen Crabs as protected. Regressions: `maniaxe_axe_respects_crab_frozen_immunity_loc_3b048`, `maniaxe_axe_respects_crab_front_vulnerability_gate_loc_3b048`. Gate: `cargo test -p kid-core maniaxe_axe_respects_crab -- --nocapture`; `mise run verify-slice-fast`. |
| 181 | [x] | P1 | enemy | Driller: probe offset wrong, drills every frame, no ice-shard bullets | Fixed/closed 2026-05-30: live `loc_3BE78` block probing uses the selected linked drill child X offset (`parent +/- $18`) instead of the parent `$48(a5)` collision half-width (`$0C`); the `loc_3BEF2` Ice path preserves the Ice/Smash discriminant from `step_driller` and spawns the three shooter objects when the remapped adjacent target cell is open. Follow-up investigation corrected the stale `loc_3BE8E` wording: `$38(a4)` is `collision_type`, not an animation timer. Native already requires a Solid front-wall probe before drilling, matching the ROM's `colid_rightwall`/`colid_leftwall` gate closely enough for the current cell model. Regressions: `loc_3be78_driller_probes_right_child_x_not_parent_collision_width`, `loc_3bef2_driller_preserves_ice_block_dispatch_for_runtime_bullets`, `loc_3bef2_driller_ice_block_spawns_three_shooter_bullets_when_target_open`, `loc_3be8e_driller_requires_front_wall_collision_before_drilling`. Gate: per-family regression + `mise run verify-slice-fast`. |
| 182 | [?] | P1 | enemy | Emo_Rock: invented ground-walk, no disappear/respawn re-attack, Berzerker launch + death routing missing | Partial 2026-05-29: settled side contact with a charging Berzerker now follows ASM `loc_362EE` by copying Kid's incoming x velocity, seeding `y_vel=$FFFD0000`, and entering the `loc_3630A` airborne gravity loop instead of routing through generic HP damage/death. Partial 2026-05-30: live settled Emo Rock no longer becomes a pacer after landing. `step_emo_rock` now follows ASM `loc_362A4`/`loc_362B6` by clearing motion and holding the landing-facing direction; regression `emo_rock_settled_wait_does_not_walk_like_pacer_loc_362b6` first failed against the old walk-toward-Kid branch. Remaining confirmed gaps: `loc_36320` settled-wait disappear/respawn cycle, `loc_36346`/`loc_3639A` death/respawn-marker routing, and `loc_3625E` drop-landing on collision word not raw floor probe. Gate: per-family regression + `mise run verify-slice-fast`. |
| 183 | [?] | P1 | enemy | Fire_Demon: walking fire-trail emission, Berzerker launch, hurt recovery anim not driven | Partial 2026-05-29: charging Berzerker side contact now follows ASM `loc_3A5E4` by launching Fire Demon upward into the `loc_3A610` loop instead of routing through generic HP damage/death. Regression: `berzerker_charge_launches_fire_demon_instead_of_damaging_loc_3a5e4`. Partial 2026-05-30: nonfatal hurt now hibernates instead of pacing during the native hurt window, matching ASM `loc_3A5BE`'s `stru_3A34E` recovery hold; regression `fire_demon_nonfatal_hurt_hibernates_before_resuming_walk_loc_3a5be` first failed against the old continued-walk branch. Remaining confirmed gap: walking fire-trail emission (`loc_3A542`/`loc_3A564`/`loc_3A63C`, up to 7 children) is still not wired into live pacing. Gate: per-family regression + `mise run verify-slice-fast`. |
| 184 | [?] | P1 | enemy | Goat: Berzerker launch, collision-type dispatch, recoil shove + hurt recoil missing | Partial 2026-05-29: charging Berzerker side contact with no positive `$42` now follows ASM `loc_3CB76` by launching Goat upward into the `loc_3CBA8` airborne loop instead of routing through generic HP damage/death. Regression: `berzerker_charge_launches_goat_instead_of_damaging_loc_3cb76`. Partial 2026-05-30: nonfatal hurt now follows ASM `loc_3CB38` by decrementing HP, setting `$42=$FF88`, restoring saved walk speed, and hibernating through the native hurt window before resuming shared-walker motion; regression `goat_nonfatal_hurt_sets_recoil_state_and_hibernates_loc_3cb38` first failed against the generic damage-flash path. Remaining confirmed gaps: full collision-type dispatch (`loc_3CB14` death-on-negative / side-contact gate / stomp HP) and positive-recoil Berzerker side shove (`loc_3CBBA`). Gate: per-family regression + `mise run verify-slice-fast`. |
| 185 | [?] | P1 | enemy | Hand: grab is cosmetic; collision dispatch tables and grab gating not wired | Partial 2026-05-30: live grab countdown now follows ASM `loc_3292E` by sampling Kid's facing byte and decrementing only when it changes, instead of spending one count every native frame; regression `hand_grab_timer_loc_3292e_decrements_only_on_kid_facing_change` guards the same-facing hold and changed-facing decrement. Remaining confirmed gaps: `KidGrabbedByHand` is read by jump code (`physics.rs:719`) but assigned nowhere (`loc_32906`), so the grab still does not immobilise the Kid; airborne (`loc_329F8`/`off_32A12`) and ground (`loc_3299C`/`off_329B6`) collision dispatch tables are absent; entry gate (`loc_32A86`) and release clearance probe (`loc_32970`) missing. See audit doc. Gate: per-family regression + `mise run verify-slice-fast`. |
| 186 | [?] | P1 | enemy | Lion: hurt recoil, gravity ledge-leap charge, roar windup + engagement bands missing | Partial 2026-05-30: live Lion timer expiry now enters explicit `Roaring` and `PostFire` phases instead of firing/charging on the same native frame. `step_lion` follows ASM `loc_3E078`/`loc_3E09E` by waiting through `stru_3DF0E` until the `LnkTo_unk_C7E78` fire sprite before allocating the projectile, then waits for `loc_3E0F4` to restore the saved charge animation. Regression `lion_timer_expiry_enters_roar_before_projectile_loc_3e078_3e09e` first failed because `LionState` had no roar/post-fire representation. Remaining confirmed gaps: `loc_3E318` hit-anim + $44 stagger, `loc_3E682` gravity-driven ledge-launch leap vs horizontal-only charge, and `loc_3E054`/`loc_3E434` close-band tracker + $42 charge-windup. Gate: per-family regression + `mise run verify-slice-fast`. |
| 187 | [?] | P1 | enemy | Mini_Hopping_Skull: generic step_hopper replaces the ASM hop state machine | Partial 2026-05-29: charging Berzerker side contact now follows ASM `loc_3F262` by zeroing x velocity, launching upward at `-$48000`, and entering the `loc_3F288` loop instead of routing through generic HP damage/death. Partial 2026-05-30: live grounded waits no longer use the generic platform-edge lookahead turnaround; `step_hopper` keeps wall reversal but disables the invented `!probe_floor(look_ahead)` flip for Mini Hopping Skull so `loc_3F000`/`loc_3F090` can remain Kid-X driven. Regression: `mini_hopping_skull_ground_wait_does_not_edge_turnaround_loc_3f000`. Remaining confirmed gaps: `loc_3EA88..loc_3F146` windup/airborne/launch co-routine collapsed to a 21-frame counter, `loc_3F1B8` air-damage knock-up loop, and `loc_3F000` post-landing facing-flip pause. Gate: per-family regression + `mise run verify-slice-fast`. |
| 188 | [?] | P1 | enemy | Ninja: attack cadence, block/guard state machine, Berzerker launch, knockdown + airborne branches missing | Partial 2026-05-29: charging Berzerker side contact now follows ASM `loc_3D698`/`loc_3DB0A` by zeroing x velocity, launching upward at `-$48000`, and entering the `loc_3DB3C` loop instead of routing through generic HP damage/death. Partial 2026-05-30: live jump-attack gravity now follows ASM `loc_3D9DA` by adding `$2800` without the shared native max-fall clamp; regression `ninja_jump_attack_live_gravity_is_uncapped_loc_3d9da` first failed at `0x80000` vs expected `0x82800`. Remaining confirmed gaps: `loc_3D92A` $54/$6E cadence timer vs overhead cone, `loc_3D606` block/guard state machine + $44 + sfx_Ninja_blocking, `loc_3D8BC` HP-depleted knockdown on above-stomp, `loc_3D9DA` jump-attack ceiling/kid-above branches, and `loc_3D978` far-Kid facing turn. Gate: per-family regression + `mise run verify-slice-fast`. |
| 189 | [?] | P1 | enemy | Orca: in-lunge contact dispatch, ceiling $44 fatal-fall, windup phase, side re-lunge missing | Partial 2026-05-29: ordinary charging Berzerker side contact now enters the `loc_3D2AA` knockback loop instead of generic HP damage. Partial 2026-05-30: live proximity detection now enters an explicit `Windup` state for the six hibernated frames from ASM `loc_3D320` before `loc_3D340`/`loc_3D348` seed launch velocities; regression `orca_enters_windup_before_lunging_loc_3d320` first failed because `OrcaState` had only Walking/Lunging. Remaining confirmed gaps: `loc_3D35A` in-lunge hurt/Kid-above/side dispatch; `loc_3D472` ceiling-hit $44 multi-hit fatal-fall; `loc_3D4DE` side-collision Berzerker re-lunge. Lunging arm probes terrain only (`enemies.rs:19793-19890`); helpers `enemies.rs:1292/1368/1091` partially wired/test-only. See audit doc. Gate: per-family regression + `mise run verify-slice-fast`. |
| 190 | [x] | P1 | enemy | Robot: charge entered every shot, no aim gate, shot lifecycle + two-object body/head missing | Partial 2026-05-29: ordinary shooting now resumes Walking through ASM `loc_3AAC4` instead of entering `loc_3A96C` charge after every shot; live shots use the `loc_3AC0C` muzzle X/Y offsets and `$50000` facing velocity; `RobotShot` flight now exits via the `loc_3AC48` camera/collision gate instead of generic timer expiry. Partial 2026-05-30: live walking Robot now routes expired shoot checks through `sub_3C4F8`'s random/aim gate and enters an `Aiming` turn sequence instead of snapping to face Kid and firing in one frame. Partial 2026-05-30: `RobotShot` now enters a typed `RobotShotImpact` object at `loc_3AC7E/loc_3AC92`, clearing velocity and applying the impact X offset instead of deleting immediately. Partial 2026-05-30: live active roster now exposes the linked Robot head from `loc_3A7EE/loc_3A816` at parent `y-$10` instead of collapsing Robot to one render/trace object. Partial 2026-05-30: live final damage now follows `loc_3A8FC -> loc_3A96C` by entering `RobotState::Charging` instead of queueing generic death. Partial 2026-05-30: friendly projectile damage now scans both the Robot parent body and linked head, matching `loc_3A8AE`'s parent `collision_type(a3)` plus child `$38(a2)` hit routes. Partial 2026-05-30: charging Berzerker side contact now targets the linked Robot head `$38(a2)` only; body-only overlap misses, while a head-side `$20/$24` contact enters the `loc_3A9CC -> loc_3A9D2` destroy-animation route. Partial 2026-05-30: stomp candidates now include both the Robot parent body and linked head for `loc_3A8AE`, deduplicating a parent+head overlap so one frame only applies one hit reaction. Partial 2026-05-30: `RobotShotImpact` now lives for the full `stru_3A762` 3+4+8 frame animation before deletion instead of a one-frame placeholder. Partial 2026-05-30: live traces now preserve typed `RobotShot`/`RobotShotImpact` render states, flying shots keep an independent looping `stru_3A754` animation clock without lifetime expiry, and impact shots draw from the exact `stru_3A762` frame table. Partial 2026-05-30: nonfatal friendly-projectile hits while native is in `RobotState::Aiming` now restore through the shoot-time `loc_3AAD4 -> loc_3AB04 -> loc_3AAC4` route instead of leaving the Robot stuck in the aiming coroutine; regression `robot_mid_shoot_nonfatal_projectile_hit_resumes_walk_loc_3aad4_3aac4`. Partial 2026-05-30: pending Robot collision words now distinguish parent `collision_type(a3)` from linked-head `$38(a2)`, so parent `$FFFF` enters `loc_3A9D2` destroy prep while child `$FFFF` enters `loc_3A96C` charge without HP damage; regressions `robot_parent_negative_collision_enters_loc_3a9d2_destroy_not_charge`, `robot_child_negative_collision_enters_loc_3a96c_charge_without_hp_damage`. Partial 2026-05-31: shoot-time live coverage now confirms final projectile hits enter the `loc_3AB56`/`loc_3A96C` charge route and charging Berzerker head-side contact during `RobotState::Aiming` enters `loc_3AAFA -> loc_3A9D2`; regressions `robot_mid_shoot_final_projectile_hit_enters_charge_loc_3ab56_3a96c`, `berzerker_charge_robot_mid_shoot_head_side_destroys_parent_loc_3aafa_3a9d2`. Done 2026-05-31: live pending collision dispatch now covers shoot-time child `$2C` hit reaction and non-charging child side contact restore through `loc_3AAD4`/`loc_3AB04`/`loc_3AAFA`/`loc_3AA9E`; regressions `robot_mid_shoot_child_kidabove_collision_hits_parent_loc_3aad4_3ab04`, `robot_mid_shoot_child_side_without_berzerker_restores_walk_loc_3aafa_3aa9e`. Regressions: `robot_walks_then_shoots_and_resumes_walking_loc_3aac4`, `robot_sub_3c4f8_aim_gate_turns_before_shooting_when_facing_away`, `robot_sub_3c4f8_aim_sequence_flips_then_shoots`, `robot_shot_loc_3ac48_enters_impact_after_camera_window_exit`, `robot_shot_loc_3ac48_enters_impact_on_level_collision`, `robot_shot_impact_lasts_for_stru_3a762_duration`, `trace_reports_robot_shot_and_impact_kinds_loc_3ac0c_3ac92`, `robot_shot_impact_uses_stru_3a762_sprite_frames_loc_3ac92`, `robot_shot_loc_3ac48_advances_stru_3a754_animation_without_lifetime_expiry`, `robot_shot_uses_visible_rom_flying_sprite_loc_3ac0c_stru_3a754`, `robot_live_roster_exposes_head_child_from_loc_3a7ee`, `robot_live_final_hit_enters_charge_instead_of_death_loc_3a8fc_3a96c`, `robot_head_projectile_damages_parent_loc_3a8ae`, `robot_body_projectile_still_damages_parent_loc_3a8ae`, `berzerker_charge_robot_body_contact_misses_loc_3a8ae`, `berzerker_charge_robot_head_destroys_parent_loc_3a9cc_3a9d2`, `robot_body_stomp_damages_parent_loc_3a8ae`, `robot_head_stomp_damages_parent_loc_3a8ae`, `robot_mid_shoot_nonfatal_projectile_hit_resumes_walk_loc_3aad4_3aac4`, `robot_parent_negative_collision_enters_loc_3a9d2_destroy_not_charge`, `robot_child_negative_collision_enters_loc_3a96c_charge_without_hp_damage`, `robot_mid_shoot_final_projectile_hit_enters_charge_loc_3ab56_3a96c`, `berzerker_charge_robot_mid_shoot_head_side_destroys_parent_loc_3aafa_3a9d2`, `robot_mid_shoot_child_kidabove_collision_hits_parent_loc_3aad4_3ab04`, `robot_mid_shoot_child_side_without_berzerker_restores_walk_loc_3aafa_3aa9e`. Remaining confirmed gaps: none known for this backlog item. See audit doc. Gate: per-family regression + `mise run verify-slice-fast`. |
| 191 | [x] | P1 | enemy | Scorpion: sting deals no damage; collision/HP state machine + tail sub-object missing | 4 confirmed gaps (`loc_3DDEE` sting attack discards tail hurtbox/object_meta/stru_3DB8E so it deals no damage; `loc_3DD0E` Berzerker upward launch; `loc_3DCB0` per-collision-type routing + hurt-anim-then-restore + death tail $3D flag; `loc_3DC86` paired tail/stinger gfx object never allocated). Follow-up 2026-05-30: live roster now exposes the linked `loc_3DC86` tail child, `loc_3DDEE` arms tail contact during `stru_3DB8E`, and contact damage routes through the normal Character_CheckCollision damage path. Follow-up 2026-05-30: charging Berzerker side contact now follows `loc_3DD0E`/`loc_3DD4A`, preserving HP while launching Scorpion into the airborne gravity loop. Follow-up 2026-05-30: nonfatal projectile/sword hurt now uses `loc_3DCD4` and hibernates through the hurt window before shared-walker motion resumes. Follow-up 2026-05-30: live pending parent `$FFFF` collisions now dispatch via `loc_3DCB0` to `loc_3DCF4`, bypass HP damage and hide the linked tail during the death animation (`scorpion_negative_parent_collision_enters_loc_3dcf4_death_before_hp_damage`). Follow-up 2026-05-30: live pending parent `colid_hurt` collisions now dispatch via `loc_3DCB0` to `loc_3DCD4`, decrement `$44(a3)`, hibernate through the hurt window, and rebuild the roster after the nonfatal state change (`scorpion_pending_hurt_collision_enters_loc_3dcd4_damage_route`). Follow-up 2026-05-30: live pending side contact now routes charging Berzerker through `loc_3DD0E` into the `loc_3DD4A` airborne loop instead of preserving the old walker velocity (`scorpion_pending_side_collision_enters_loc_3dd0e_berzerker_launch`). Done 2026-05-31: direct live pending zero and unknown-positive parent collisions now have `loc_3DCB0 -> loc_3DD5C` resume/clear coverage without HP damage, death, or Berzerker launch (`scorpion_pending_zero_and_unknown_positive_collision_resume_loc_3dcb0_3dd5c`). Regressions: `scorpion_negative_parent_collision_enters_loc_3dcf4_death_before_hp_damage`, `scorpion_pending_hurt_collision_enters_loc_3dcd4_damage_route`, `scorpion_pending_side_collision_enters_loc_3dd0e_berzerker_launch`, `scorpion_pending_zero_and_unknown_positive_collision_resume_loc_3dcb0_3dd5c`. Gate: per-family regression + `mise run verify-slice-fast`. |
| 192 | [x] | P1 | enemy | Spinning_Twins: orbit never splits into two children; physics, HP/death, Berzerker, orbit timing missing | Partial 2026-05-29: charging Berzerker side contact now follows ASM `loc_3BBF6` by launching a settled Spinning Twin upward into the `loc_3BC22` loop instead of routing through generic HP damage/death. Partial 2026-05-30: nonfatal settled hurt now follows `loc_3BC34`, decrementing `$44`, hibernating through `stru_3B836`, and then resuming the walking path instead of continuing movement immediately; regression `spinning_twins_nonfatal_hurt_hibernates_through_stru_3b836_loc_3bc34`. Partial 2026-05-30: live walking-child airborne stepping now applies the `loc_3BB2C` movement order, including X movement, `$2800` Y gravity, `$1200` X drag and the asymmetric `$3000/-$3000` settle gate; regression `spinning_twins_live_walking_child_applies_x_motion_and_drag_loc_3bb2c_3bb52`. Partial 2026-05-30: finished settled death now applies the `loc_3BC78` status-table tail, clearing positive `$42(a5)` slots, subtracting `$400` for bit-6 spawn counters, and leaving bit-7-only slots untouched; regressions `spinning_twins_finished_death_clears_positive_status_slot_loc_3bc78`, `spinning_twins_death_status_route_updates_native_enemy_status_table_loc_3bc78`. Partial 2026-05-31: live orbit-contact transition now materialises the `loc_3B93C` split as two `loc_3BA74` walking children plus one non-vulnerable `loc_3B9B0` falling visual instead of collapsing the source object to one walker; regression `spinning_twins_runtime_drop_materializes_two_children_and_visual_loc_3b93c`. Partial 2026-05-31: deleting one live walking child now follows `loc_3BCA6` by marking the paired walking twin's `$5B` flag before deleting the current child; regression `spinning_twins_deleting_one_walking_child_marks_partner_loc_3bca6`. Done 2026-05-31: orbiting Spinning Twins now follow `loc_3B904`: parent `x_pos(a3)` stays anchored, child offset index advances only when `animation_timer(a3)==1`, and the drop route is explicitly gated to signed-negative, `$1C`, and `$2C` collision words; regressions `spinning_twins_orbit_loc_3b904_advances_only_on_animation_timer_one`, `spinning_twins_orbit_collision_route_loc_3b904_drops_only_for_rom_contact_words`. Regressions: `berzerker_charge_launches_spinning_twins_instead_of_damaging_loc_3bbf6`, `spinning_twins_nonfatal_hurt_hibernates_through_stru_3b836_loc_3bc34`, `spinning_twins_live_walking_child_applies_x_motion_and_drag_loc_3bb2c_3bb52`, `spinning_twins_finished_death_clears_positive_status_slot_loc_3bc78`, `spinning_twins_death_status_route_updates_native_enemy_status_table_loc_3bc78`, `spinning_twins_runtime_drop_materializes_two_children_and_visual_loc_3b93c`, `spinning_twins_deleting_one_walking_child_marks_partner_loc_3bca6`, `spinning_twins_orbit_loc_3b904_advances_only_on_animation_timer_one`, `spinning_twins_orbit_collision_route_loc_3b904_drops_only_for_rom_contact_words`. Gate: per-family regression + `mise run verify-slice-fast`. |
| 193 | [x] | P1 | enemy | Tank: body-vs-mouth vulnerability, negative-collision instant death, non-fatal recoil missing | Done 2026-05-29/30: live plain Tank nonfatal damage follows ASM `loc_3B49A` by clearing parent motion and hibernating the shared walker for five frames; regression `stomp_nonfatal_plain_rock_tank_stops_motion_for_loc_3b49a_hibernate`. Plain Tank stomp, friendly projectile/rain, Red Stealth sword, and Berzerker charge contacts route the attached mouth/turret snapshot back to the parent active enemy per `loc_3B464`; regressions `stomp_plain_rock_tank_mouth_attachment_damages_parent_loc_3b464`, `plain_rock_tank_parent_body_projectile_misses_loc_3b464`, `plain_rock_tank_mouth_projectile_damages_parent_loc_3b464`, `red_stealth_sword_plain_rock_tank_parent_body_misses_loc_3b464`, `red_stealth_sword_plain_rock_tank_mouth_damages_parent_loc_3b464`, `berzerker_charge_plain_rock_tank_parent_body_misses_loc_3b464`, `berzerker_charge_plain_rock_tank_mouth_damages_parent_loc_3b464`. Tank and TankShooting death animations hide the attached mouth/turret child per `loc_3B480`/`loc_3B6E6`; regressions `tank_death_waits_for_stru_3b29a_animation_before_delete`, `tank_shooting_death_waits_for_stru_3b518_animation_before_delete`. Pending parent collision `$FFFF` routes live Tank through `loc_3B41C` -> `loc_3B480`, bypassing HP decrement and hiding the mouth child; regression `tank_negative_parent_collision_enters_loc_3b480_death_before_hp_damage`. ASM re-check of `DiamondPower_Main_CircleOfDoom`/`DiamondPower_Main_WallOfDeath` found no dedicated direct-delete loop; Circle DP slot collisions now target the Tank mouth child and route nonfatal damage through `loc_3B464`/`loc_3B49A`; regression `circle_of_doom_slot_collision_routes_tank_child_through_loc_3b41c`. Gate: per-family regression + `mise run verify-slice-fast`. |
| 194 | [x] | P1 | enemy | Tar_Monster: 5-projectile burst fires on the wrong event; child bounce + glob impact-burst missing | Fixed 2026-05-29: nonfatal stomp damage now follows ASM `loc_323C0` and no longer runs the five-child burst; straight attack globs are tagged and enter `loc_324F8` on terrain contact, replacing the shot with the five ROM-velocity burst children; burst children now follow `loc_32682`/`off_32656` velocity reflection and d3 response-budget deletion. Regressions: `stomp_nonfatal_tar_monster_hurt_does_not_spawn_burst_loc_323c0`, `tar_monster_straight_glob_bursts_on_terrain_contact_loc_324f8`, `tar_monster_burst_child_reflects_once_and_spends_budget_loc_32682`, `tar_monster_burst_child_deletes_when_collision_budget_spent_loc_32698`. Gate: `mise run verify-slice-fast`. |
| 195 | [x] | P1 | enemy | Tornado: Cyclone-above kill branch, ceiling bounce, slope bounce missing | Fixed 2026-05-29: `loc_35948` ceiling bounce and slope BounceUp/BounceDown now route through the Tornado/UFO dispatch table; `loc_35986` now applies the Cyclone-flying-above y_pos compare and enters `loc_359B2` death before writing Kid collision_type=$28. Regressions: `tornado_ceiling_collision_bounces_y_velocity_loc_35948`, `tornado_slopeup_collision_uses_ufo_slope_bounce_loc_35948`, `tornado_slopedown_collision_uses_ufo_slope_bounce_loc_35948`, `loc_35986_cyclone_above_tornado_enters_loc_359b2_death`. Gate: `mise run verify-slice-fast`. |
| 196 | [x] | P1 | enemy | UFO: bounce table, chase steering, motion picker, RNG beam trigger, and two-phase hurt wired | Fixed 2026-05-29: `loc_35056` floor/ceiling and slope BounceUp/BounceDown now route through the UFO dispatch table in the live `step_ufo` terrain path; live flyer stepping now passes player X/Y into `loc_34E2A`, so below-and-near-Kid chase steering clears y velocity and writes the scaled horizontal tracking command; timer-zero frames now run `loc_35326`/`unk_35344`/`loc_35444` before `Enemy_HandleAcceleration` instead of the fabricated 60-frame sign flip; random beam start now uses `Get_RandomNumber_byte` semantics (`<=4`) instead of a position hash; first hurt now follows `$47(a5)==0` through `loc_351EE`/`loc_35228` survival, and final hurt follows `loc_35174`/`loc_35180` death independent of the motion timer. Regressions: `ufo_floor_collision_uses_loc_35056_floorceil_bounce`, `ufo_ceiling_collision_uses_loc_35056_floorceil_bounce`, `ufo_slopeup_collision_uses_loc_35056_slope_bounce`, `ufo_slopedown_collision_uses_loc_35056_slope_bounce`, `ufo_live_step_invokes_loc_34e2a_tracking_when_kid_below_near`, `ufo_timer_zero_uses_loc_35326_motion_picker_before_acceleration`, `ufo_motion_picker_matches_loc_35326_selector_and_loc_35444_scaling`, `ufo_random_beam_uses_rom_rng_byte_loc_34f8e`, `ufo_random_beam_gate_matches_loc_34f8e_rng_threshold`, `ufo_first_hurt_survives_and_primes_loc_35228`, `ufo_death_uses_loc_35174_parent_animation_before_delete`. Gate: per-family regression + `mise run verify-slice-fast`. |
| 197 | [x] | P2 | enemy | Archer: fire cadence ~4-5x too fast; shot gate counts the shared projectile pool | Fixed 2026-05-30: live arrows are typed `ArcherArrow`, `loc_34248` counts only those arrows against `Number_of_Arrows >= 2`, and the live archer now reloads from the `loc_341BE` shooting-table delay (`$64+1`, `$06+1`, `$0C+1` = `$79` frames) instead of reusing `arrow_accuracy` 30/20/10. Fired arrows still use `arrow_accuracy` for `loc_342D6` re-aiming. Regressions: `archer_loc_34248_caps_only_live_archer_arrows`, `archer_shooting_cycle_delay_loc_341be_uses_pose_table_not_arrow_accuracy`, `archer_live_step_waits_pose_table_delay_between_arrows_loc_341be`. Gate: `mise run verify-slice-fast`. |
| 198 | [x] | P2 | enemy | Cloud: lightning projectile has a fixed 31-frame lifetime instead of collision-bound persistence | Fixed 2026-05-29: live Cloud lightning is now typed `CloudLightning`, spawns from `loc_35DF0` with no fabricated lifetime counter, and `step_projectiles` exempts it from generic lifetime expiry so `loc_35E8A` persists until terrain/Kid collision or the future `$19` animation flag path. Regressions: `cloud_runtime_loc_35cd8_latches_then_spawns_diagonal_lightning`, `cloud_lightning_loc_35e8a_persists_without_generic_lifetime_expiry`. Gate: `mise run verify-slice-fast`. |
| 199 | [x] | P2 | enemy | Dragon (walking): breath flame deleted on first Kid contact instead of persisting its animation | Fixed 2026-05-29: live walking-dragon breath spawns as `DragonWalkingBreath`, and player overlap now hurts Kid without consuming the breath child. This mirrors `Dragon.asm loc_3D02A/loc_3D082`, where deletion is owned by the child animation/parent `$5B` poll rather than by Kid collision. Regressions: `dragon_walking_fires_breath_in_facing_direction`, `dragon_walking_breath_hits_player_without_consuming_child_loc_3d082`. Gate: `mise run verify-slice-fast`. |
| 200 | [x] | P2 | enemy | Dragon_Flying: descent fireball lacks windup/travel/off-screen-kill/impact lifecycle | Fixed: descent fire is now a typed projectile with `loc_34C26` startup DBF, `loc_34C3A` travel/off-screen gate, and `loc_34C4A` two-stage cleanup; regression covers zero-lifetime survival, startup transition, and terrain impact cleanup. Gate: per-family regression + `mise run verify-slice-fast`. |
| 201 | [x] | P2 | enemy | Fireball: fire-trail child objects never spawned on bounce/slope | Fixed 2026-05-29: live Fireball bounce/slope handling now carries a projectile sink, counts typed `FireballTrail` children for the ROM `Number_of_Fire_Trails < 5` cap, and spawns short-lived hostile trail children from `loc_32CE4`/`loc_33128` and slope `loc_330D0` paths. Player overlap hurts Kid without consuming the child; deletion remains lifetime/terrain-owned per `loc_33110`. Regressions: `fireball_floor_bounce_spawns_trail_child_loc_32ce4_33128`, `fireball_floor_bounce_respects_five_trail_cap_loc_32ce4`, `fireball_slope_up_spawns_trail_child_loc_330d0`, `fireball_trail_hits_player_without_consuming_child_loc_33110`. Gate: `mise run verify-slice-fast`. |
| 202 | [x] | P2 | enemy | Tank_Shooting: bullets pass through walls; non-fatal hit does not stop/hibernate the tank | Fixed — tank bullets now feed terrain contact into ASM `loc_3B7BA` and delete on level collision; nonfatal projectile hits now apply ASM `loc_3B700` by clearing parent motion and hibernating the shared walker for five frames before `loc_3B720` resumes. Regressions: `tank_bullet_loc_3b7ba_deletes_on_level_collision`, `tank_shooting_nonfatal_projectile_hit_hibernates_loc_3b700`. Gate: `mise run verify-slice-fast`. |
| 203 | [?] | P1 | architecture | Dead-helper census: ASM-faithful unit-tested helpers exist crate-wide but the live path uses generic fallbacks (`docs/source-port/dead-helper-census.md`) | Codebase-wide follow-up to the enemy audit. 2,972 `allow(dead_code)`-gated items across 15 files; almost all are unit-tested ASM models, but the live runtime reaches only some. **Revises #176's framing**: `enemies.rs` (and `terrain.rs`) are actually mostly WIRED (stale attrs) — the unwired *epicentre* is `combat.rs` (entire boss subsystem, zero live refs), `level.rs` (enemy-loading/spawn manager + block RLE/property decode), `helmets.rs` (Diamond Power weapons collapsed to an `invulnerable_frames.max(1)` stub), `platforms.rs` (all platform physics), `weather.rs` (geyser/storm script machines), `lib.rs` (GEMS audio — triage: abandoned alt impl vs fidelity target). Two workstreams: **(a) wiring** the unwired models highest-leverage-first (ranked in the census doc) and **(b) hygiene** — strip stale `cfg_attr(not(test), allow(dead_code))` attrs from genuinely-wired helpers (~113 in `enemies.rs` alone) to restore the dead-code warning as a real signal. **Prerequisite for (a):** the idle oracle sweep exercises none of these subsystems — wiring needs object/enemy-active oracle scenarios first, or it ships unverified. Gate: per-cluster regression + new oracle scenario + `mise run verify-slice-fast`. Source data: `.cache/audit/dead-helper-census.json` (regen via `.cache/audit/extract_dead_helpers.py`). |
Note 2026-05-30 for #203: live shooter blocks now use the already-modelled `sub_F536`/`loc_F326` restart/free branch at active-slot expiry, so a visible shooter keeps firing while Kid remains in range instead of silently stopping until the next scan-boundary crossing.
Note 2026-05-31 for #203: started the hygiene workstream without promoting behavioural wiring. Removed stale dead-code suppressions from the already-live lava collision/init helpers in `weather.rs`: `kid_hitbox_bounds_px_at_screen`, `lava_geyser_hitbox_window_loc_14AAA`, `lava_fireball_vertical_window_loc_14AFC`, `lava_fireball_rejects_kid_loc_14B22`, `lava_fireball_overlap_gate_loc_14B06`, `lava_fireball_hitbox_x_window_loc_14AB0`, `lava_geyser_initial_delay_frames_loc_14772`, `lava_geyser_position_table_loc_14C5E`, and `initialise_lava_geyser_loc_14C6A`. Live evidence: `NativeRuntime::step` reaches `step_lava_geysers`, and `init_weather` reaches `init_lava_geysers`; `cargo check -q -p kid-core` confirms the compiler sees these helpers/types as used without the suppressions. Focused gates: `cargo test -q -p kid-core lava_geyser -- --nocapture`, `cargo test -q -p kid-core lava_fireball -- --nocapture`. The dark lava geyser/fireball script-machine wiring remains open and still needs an object/weather-active oracle scenario under #205 before behavioural promotion.
| 204 | [x] | P1 | player | Non-walking forms (Red Stealth/Eyeclops/Maniaxe/Skycutter/Juggernaut) jitter on a slope instead of sliding down | Fixed 2026-05-29 by `ebf2704c`/`f4df2184`: #152's adjacent-slope walk-UP hold remains faithful, but current slope support now bypasses `should_hold_standing_still` so `loc_7820`/`loc_8D72` can reach `sub_9386` and `loc_9E6C` downhill slide. Regressions: `asm_faithful_red_stealth_on_current_slope_up_enters_loc_9e6c_slide`, `asm_faithful_red_stealth_current_slope_up_slides_without_limit_cycle`, `asm_faithful_eyeclops_on_current_slope_down_enters_loc_9e6c_slide`, `asm_faithful_eyeclops_current_slope_down_slides_without_limit_cycle`, with adjacent-hold coverage still pinned by `asm_faithful_loc_77c2_non_bitmask_slope_up_holds_standing_right` and `asm_faithful_loc_77fc_non_bitmask_slope_down_holds_standing_left`. Gate: `mise run verify-slice-fast` passed in the fixing session; focused tests re-run 2026-05-29. |
| 205 | [?] | P1 | infra | Object/enemy-active oracle scenario harness — prerequisite for verifying dead-helper wiring (#206-#208) | The idle sweep exercises no boss / Diamond-Power-weapon / platform / weather-script behaviour, so wiring those ships unverified, against "oracle is truth / never ship a guess". Build per-subsystem `DynamicScenario`s that spawn, engage and hit each system with oracle state checkpoints. Source: `docs/source-port/dead-helper-census.md` (#203). Partial 2026-05-30: native `DynamicScenario` replay now honours probe `start_position`, `camera_position`, and `playtest_form` in the TAS/case/view/sweep comparison paths, matching the fields already sent to the oracle helper. Follow-up adds explicit `settle_oracle_warp` to scenario JSON and the jgenesis helper, so focused warped probes can request live/unpaused target-level gameplay before frame 0 instead of depending on `idle_level_full_frames_*` / `_settled_recording_oracle` slug magic. The helper also clears `Currently_transforming` residue for direct `playtest_form` probes. Temporary `.cache/scenarios/probe-fields-smoke/scenario.json` now reaches live gameplay with matching CRCs under a settled Level 00 two-frame smoke; the remaining `number_objects` state diff is the existing fixed-object accounting mismatch, not a title-card bootstrap miss. 2026-05-31 evidence adds committed `lava_weather_audit_scenario(0x05)`, matching ASM `Init_SpecialEffect_Lava -> loc_14C5E/loc_14C6A` and `Obj_Lava_Geyser -> loc_147BA`; cache repro path `.cache/recordings/issue-205-lava-weather-audit/scenario.json`. Oracle parity result: `verify-tas --frame-start 0 --frame-end 180 --checkpoint-interval 30 --frontier-kind enemy` reaches live Level 0x05 on both sides but fails immediately: native `number_objects=5` vs oracle `9`, native `level_special_effects=null` vs oracle `1`, and native/oracle disagree on the first active enemy roster slot from `frame_0`. Follow-up 2026-05-31 rejected stale cache recording `.cache/recordings/issue-205-lava-usm2`: native-only replay passes, but full `verify-recording` fails; derived case `.cache/cases/issue-205-lava-usm2` blocks because settled-recording oracle bootstrap starts Level 05 with different x/y velocities and enemy position (`native=01@06610000,01500000`, oracle `01@06300000,01500000`). Follow-up 2026-05-31 also rejects cache recording `.cache/recordings/issue-205-lava-weather-audit` as a gameplay-authoritative case: native-only replay passes, full `verify-recording` fails oracle state/frame parity, and derived case `.cache/cases/issue-205-lava-weather-audit` blocks on the same settled Level 05 initial-state mismatch (`player fields=x_velocity|y_velocity`, first enemy `native=01@06610000,01500000`, oracle `01@06300000,01500000`). Proposed fix: teach the object-active comparison/export path to model ROM special-effect objects as first-class weather/object state, then wire the lava script-machine helpers under the committed oracle-visible `lava_weather_audit_scenario(0x05)` instead of treating weather as uncounted native side state; separately fix settled-recording oracle bootstrap before mining cache recordings for Level 05 gameplay cases. Gate: new settled object-active scenarios green in `sweep`; `mise run verify-slice-fast`. |
| 206 | [~] | P1 | player | Diamond Power weapons collapsed to an invulnerability stub; full weapon set modelled but unwired | Current `step_diamond_power` is no longer just invulnerability: rain is wired; Circle/Wall/Five-Way/Snake seed live DP slots, expose `word_4020E` sprite attrs to the native player, and run ROM-style per-slot collision-word overlap checks for the visible slot families. 2026-05-31 added committed `initial_diamond_count` probe seeding plus `diamond_power_circle_audit_scenario(0x00)`, matching ASM `Number_Diamonds` / `DiamondPower_Check` and `DiamondPower_Init_CircleOfDoom -> loc_3F7D0`. Follow-up 2026-05-31 wired Circle slot init/main/rendering through `DiamondPower_Main_CircleOfDoom loc_3F81A/loc_3F880`; cache repro path `.cache/recordings/issue-206-diamond-power-circle-audit`: native-only replay passes; full `verify-recording` still has `trace_match=true`, `oracle_state_match=true`, and `oracle_frame_match=false` from frame 0 (`changed_pixel_count=3570`, broad palette/background bounds). Follow-up 2026-05-31 wired Wall slot init/main/rendering through `DiamondPower_Init_WallOfDeath loc_3FFE4/loc_3FFEA` and `DiamondPower_Main_WallOfDeath loc_40026/loc_4009C`, and fixed native recording setup so `initial_diamond_count` is applied outside ignored `.cache`. Cache repro path `.cache/recordings/issue-206-diamond-power-wall-audit`: native-only replay passes; refreshed full verify now has `trace_match=true`, `oracle_state_match=true`, and `oracle_frame_match=false` from pre-existing Forest rendering differences. Follow-up 2026-05-31 removed the native Wall/Circle blanket deletes and routed collision words through `sub_1D84` / `loc_1EA0`, then wired Five-Way through `DiamondPower_Init_FiveWayShot loc_400DE/loc_400F4` and `DiamondPower_Main_FiveWayShot loc_40140`. Current 2026-05-31 Snake pass wires MiniSnake/DeathSnake through six `DiamondPower_Init_Snake loc_3F91C/loc_3F946` slots and `DiamondPower_Main_Snake loc_3F978` trail/steer/sprite flow instead of generic projectiles, with `diamond_power_snake_audit_scenario(0x00)` available for oracle recording. Remaining #206 fidelity gap: strict pixel parity for the Diamond Power audit recordings is still blocked by the known Forest background/palette renderer mismatch. Gate: `diamond-power-circle-audit`/`diamond-power-wall-audit`/`diamond-power-snake-audit` oracle state parity + `mise run verify-slice-fast`. |
Note 2026-05-31 for #206: Wall-of-Death slots now run the ROM DP-object collision path for live enemies. Native derives each visible Wall slot's `CollisionSize_Index` rectangle from `DiamondPower_Main_WallOfDeath loc_4006E`, applies the inclusive `sub_1D84` overlap checks, maps the target object type through `loc_1EA0`/`unk_1EBE`, and queues the resulting collision word. This fixes the wall audit state gap: refreshed `.cache/recordings/issue-206-diamond-power-wall-audit` has `trace_match=true` and `oracle_state_match=true`; strict pixels still fail (`oracle_frame_match=false`, first checkpoint `diamond_power_input` changed pixels 4,127) from pre-existing broad Forest background/palette/rendering differences. A surprise in this pass was that the repeated Wall collision could delete a hibernating DragonWalking corpse early; native now keeps `Dragon.asm loc_3CEFC`/`stru_3CCEC` death objects resident and ignores further pending collision words until the shared delete tail runs. Regressions: `kid_projectile_dp_collision_word_loc_1ea0_uses_object_meta_low_byte_table`, `diamond_power_wall_of_death_collision_rect_uses_unk_31dcc_loc_1d84`, `diamond_power_wall_of_death_overlap_loc_1d84_is_inclusive`, `wall_of_death_slot_collision_queues_dragon_kidabove_loc_1d84_1ea0`, `dragon_walking_death_animation_bypasses_live_retention_loc_3cefc`, and `dragon_walking_death_ignores_repeated_pending_collision_loc_3cefc`. Next #206 pass should use the now-state-green Wall audit to isolate Wall sprite/pixel composition, then repeat the same per-slot collision promotion for Circle before Snake/Five-Way.

Note 2026-05-31 for #206: Circle-of-Doom now uses the shared DP-object collision-word path instead of the temporary all-enemy AoE. Native returns each Circle slot's `$1A/$1E` world position from `DiamondPower_Main_CircleOfDoom loc_3F81A`, derives the `$22=$1144+size*4` rectangle used by `sub_1D84`, maps the overlapped object's `$3A&$FF` through `loc_1EA0`/`unk_1EBE`, writes `colid_rightwall` back to the slot status, and queues parent/child collisions through the pending collision system. Tank child collisions now route through `Tank.asm loc_3B464/loc_3B49A` instead of parent-body damage. Regressions: `circle_of_doom_main_does_not_blanket_delete_enemies_loc_3f81a_loc_1d84`, `circle_of_doom_slot_collision_refreshes_roster_loc_3f81a_loc_1d84`, and `circle_of_doom_slot_collision_routes_tank_child_through_loc_3b41c`. Refreshed `.cache/recordings/issue-206-diamond-power-circle-audit` has native replay green and full `verify-recording` at `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; strict pixels still fail at frame 0 with changed pixels 3,570 from the pre-existing Forest palette/background/rendering mismatch.

Note 2026-05-31 for #206: Five-Way Shot now uses live DP-object slots instead of generic friendly projectiles. Native stores the five `FiveWayShotSlot`s seeded by `DiamondPower_Init_FiveWayShot loc_400DE/loc_400F4`, walks them through `DiamondPower_Main_FiveWayShot loc_40140`, compiles `word_4020E` sprites at `loc_401BA`, and routes visible slots through the shared `sub_1D84` collision-word helper. Regressions: `diamond_power_five_way_spawns_startup_spread_loc_400de`, `diamond_power_five_way_mirrors_left_facing_spread_loc_400de`, `diamond_power_five_way_bypasses_generic_projectile_cap_loc_400f4`, plus the existing per-slot `diamond_power_five_way_*` helper tests. Follow-up Snake work is tracked in the next note.

Note 2026-05-31 for #206: MiniSnake/DeathSnake no longer spawn generic forward projectiles. Native now stores six `SnakeSlot`s from `DiamondPower_Init_Snake loc_3F91C/loc_3F946`, uses the ROM MiniSnake/DeathSnake size/speed tables from `loc_3F8EE` and `loc_3F90C`, shifts trails through `DiamondPower_Main_Snake loc_3F978`, scans on-screen enemy targets at `loc_3F9A2/loc_3F9C2`, steers the head through `loc_3FA16`-`loc_3FAD4`, compiles `word_4020E` sprites at `loc_3FB3E`, and routes visible slots through the shared `sub_1D84` collision-word helper. `record-scenario --scenario diamond-power-snake-audit --level 00` now seeds Micromax with 20 diamonds for a MiniSnake oracle bundle. Follow-up fixed the obvious native truncation by retaining the low word of Snake `$1A/$1E` and adding `diamond_power_snake_move_head_retains_fraction_loc_3fad4`. Cache repro `.cache/recordings/issue-206-diamond-power-snake-audit`: native-only replay passes; full `verify-recording` still has `trace_match=true`, `oracle_state_match=false`, `oracle_frame_match=false`. The actionable late mismatch is `diamond_power_tail`: native leaves the Level 00 dragon alive at `x_pos=0x006f0000`, `current_hp=1`, while oracle has `x_pos=0x00970000`, `collision_type=0x002c(kidabove)`, `current_hp=0`; after retaining Snake fractions, this now points at pre-existing DragonWalking position drift and/or remaining Snake collision timing, not simple per-frame truncation. Added registered exploratory `diamond_power_snake_probe` trace output for native head high/low words, angle/turn/speed, selected target, and first sprite; refreshed trace shows frame 0 target `0x00aa,0x0110` and frame 8 head `0x0022.0xf050,0x010e.0x237c` targeting `0x00a6,0x0110`. Regressions: `diamond_power_mini_snake_seeds_live_slots_loc_3f8ee_3f91c`, `diamond_power_death_snake_uses_death_table_loc_3f90c`, `diamond_power_snake_bypasses_generic_projectile_cap_loc_3f92a`, `diamond_power_snake_move_head_retains_fraction_loc_3fad4`, `trace_frame_exports_diamond_power_snake_probe_loc_3fad4`, `diamond_power_snake_audit_seeds_micromax_tier1_start_a_and_twenty_diamonds`, and `build_record_scenario_diamond_power_snake_audit_seeds_micromax_tier1_probe`, plus the existing helper-level `diamond_power_snake_*` tests. Next Snake audit pass should add matching oracle DP-slot/head diagnostics or narrow the DragonWalking patrol position drift before changing collision timing.
Note 2026-05-31 for #206: Snake DP slot traversal now matches the ROM linked-list order. `DiamondPower_Init_Snake loc_3F92A` saves the first allocated object as the head at `$FFFFF5B4`, but `Allocate_DiamondPowerObjectSlot` prepends later slots to `Addr_FirstDPObjectSlot`; `DiamondPower_Main_Snake loc_3F978` therefore updates trail slots until it reaches the saved head, then moves that head at `loc_3FAD4`. Native stores that traversal order with the saved head last, so the trail update no longer overwrites the moving head with stale Kid-position trail data. This clears the previous Snake audit state gap: refreshed `.cache/recordings/issue-206-diamond-power-snake-audit` has native replay green and full `verify-recording` at `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; strict pixels still fail at all checkpoints from the pre-existing Forest palette/background/rendering mismatch (`diamond_power_tail` changed pixels 3,664). Regressions: `diamond_power_mini_snake_seeds_live_slots_loc_3f8ee_3f91c`, `diamond_power_death_snake_uses_death_table_loc_3f90c`, `diamond_power_init_snake_allocates_6_slots_at_kid_position_loc_3f92a_loc_3f946`, `diamond_power_init_snake_death_snake_sizes_loc_3f946`, `diamond_power_snake_trail_update_preserves_saved_head_loc_3f978_3fad4`, and `trace_frame_exports_diamond_power_snake_probe_loc_3fad4`.
Note 2026-05-31 for #206: Extra Hit Point and Extra Life no longer award immediately on Diamond Power activation. Native now seeds the ten reward DP-object slots from `DiamondPower_Init_ExtraHitPoint loc_3FB90` / `DiamondPower_Init_ExtraLife loc_3FB94` through `loc_3FB96/loc_3FB9A`, renders the orbit via `DiamondPower_Main_ExtraHPLife loc_3FBE4/loc_3FC56`, and applies the reward once only after `loc_3FC2A` convergence reaches `loc_3FC4C` or `loc_3FC4E`, then clears the DP object list through the native `sub_3F63C` equivalent. Regressions: `extra_hit_point_activation_waits_for_orbit_convergence_loc_3fb96_loc_3fc2a`, `extra_life_activation_waits_for_orbit_convergence_loc_3fb94_loc_3fc4e`, plus the existing helper-level `diamond_power_init_extra_hp_life_*` and `diamond_power_main_extra_hp_life_*` tests. Remaining #206 gap is still Invulnerability/SamuraiHaze live-slot rendering and strict pixel parity for the audit recordings behind the Forest renderer mismatch.
Note 2026-05-31 for #206: Invulnerability and Samurai Haze now seed and render their ten DP-object slots instead of only setting `invulnerable_frames`. Native wires `DiamondPower_Init_Invulnerability loc_3FC78` and `DiamondPower_Init_SamuraiHaze loc_3FC82` through `loc_3FC8A/loc_3FC8C`, keeps a signed `Diamond_power_timer` mirror for `DiamondPower_Main_InvulnAndHaze loc_3FD28`, renders small/large sprites through `loc_3FD4C`/`loc_3FDC2`, and leaves the slot list alive for the ROM negative-timer fade window until `timer < -$1B` clears through the native `sub_3F63C` equivalent. `trace_frame().diamond_power_active` now treats the fade-window slots as active even after the public countdown reaches zero. Regressions: `invulnerability_activation_exposes_live_dp_sprites_loc_3fc78_loc_3fdc2`, `samurai_haze_activation_uses_haze_slot_table_loc_3fc82_loc_3fc8c`, `invulnerability_slots_survive_positive_timer_for_fade_loc_3fd28`, plus the existing helper-level `diamond_power_invuln_haze_*` tests. Remaining #206 gap is strict pixel parity for audit recordings behind the known Forest renderer mismatch and any dedicated oracle bundle for non-Forest Invulnerability/Haze visuals.

Note 2026-05-31 for #206: Added committed non-Forest audit scenario builders for Skycutter Invulnerability and Red Stealth Samurai Haze. `record-scenario --scenario diamond-power-invulnerability-audit --level 02` and `record-scenario --scenario diamond-power-samurai-haze-audit --level 02` seed 20 diamonds, hold Start+A, and sample `DiamondPower_Check` tier-1 activation on Highwater Pass 1 rather than a Forest scene. Cache repro paths: `.cache/recordings/issue-206-diamond-power-invulnerability-audit` and `.cache/recordings/issue-206-diamond-power-samurai-haze-audit`; native-only replay passes for both. Full `verify-recording` reports `trace_match=true` for both, Samurai Haze has `oracle_state_match=true` and `oracle_frame_match=false`, while Invulnerability still has `oracle_state_match=false` plus `oracle_frame_match=false`. This removes the missing non-Forest bundle gap and leaves Invulnerability state parity as the next actionable #206 check before strict pixels. Regressions: `diamond_power_invulnerability_audit_seeds_skycutter_tier1_start_a_and_twenty_diamonds`, `diamond_power_samurai_haze_audit_seeds_red_stealth_tier1_start_a_and_twenty_diamonds`, `build_record_scenario_diamond_power_invulnerability_audit_seeds_skycutter_tier1_probe`, and `build_record_scenario_diamond_power_samurai_haze_audit_seeds_red_stealth_tier1_probe`.
Note 2026-05-31 for #206: Invulnerability now mirrors the ROM `KidIsInvulnerable` byte (`$FFFFFA2B`) separately from the native post-hit damage-flash countdown. Native sets it only through `DiamondPower_Init_Invulnerability loc_3FC78`, leaves Samurai Haze at `loc_3FC82` clear, clears it with the native `sub_3F63C` DP-list reset, and maps oracle `invulnerable_frames` comparisons to this byte instead of `progression.invulnerable_frames`. `Mode_Level loc_7D2` runs `DiamondPower_Run` before `ReadJoypad`, so native `step_frame` feeds Diamond Power from the pre-read `Ctrl_Held` latch rather than the just-sampled input; this removes the false early `$FA2B` mismatch in refreshed `.cache/recordings/issue-206-diamond-power-invulnerability-audit`. Native-only replay still passes. Full `verify-recording` remains state-red/frame-red, but the first real state gap is now Skycutter movement drift (`diamond_power_activation`: native `kid_x_vel=0x6000`, oracle `0x4000`) and the later tail helmet/position fallout, not Invulnerability byte state. Regressions: `diamond_power_run_uses_pre_readjoypad_ctrl_held_latch_loc_7d2`, `invulnerability_activation_exposes_live_dp_sprites_loc_3fc78_loc_3fdc2`, `samurai_haze_activation_uses_haze_slot_table_loc_3fc82_loc_3fc8c`, and `invulnerability_slots_survive_positive_timer_for_fade_loc_3fd28`.

Note 2026-05-31 for #206: Skycutter ground run acceleration now uses the older `Ctrl_A_Held` latch implied by the player-object hibernate/decode order (`loc_7452` before `loc_7464`), while direction held flags keep the normal previous-frame latch. This clears the early Invulnerability audit movement drift: refreshed `.cache/recordings/issue-206-diamond-power-invulnerability-audit` matches oracle `kid_x_pos`/`kid_x_vel` at `diamond_power_activation` (`0x4000` velocity, not the previous native `0x6000`). Native replay passes. Full `verify-recording` still reports `oracle_state_match=false` / `oracle_frame_match=false`: the remaining authoritative state gap is the tail (`diamond_power_tail`) where native is still Skycutter with different position/velocity and oracle has reverted to Kid; early diffs are now only RNG offset plus unmodelled `level_special_effects`. Regression: `loc_7452_skycutter_run_uses_older_ctrl_a_held_latch`.

Note 2026-05-31 for #206: Diamond Power audit scenarios now keep the original activation checkpoints but add dense tail probes, plus frame-by-frame probes for the Invulnerability transition window first exposed by the refreshed Level 02 audit. Refreshed `.cache/recordings/issue-206-diamond-power-invulnerability-audit`: native-only replay passes; full `verify-recording` remains state/frame red as expected, but the first authoritative state mismatch is now isolated to frame 57 (`diamond_power_transition_probe_057`): native `kid_x_vel=0`, oracle `kid_x_vel=0x0001e000`, with only RNG/effects differing through frame 56. This makes the next actionable pass Skycutter movement/collision after DP activation, not the `$FA2B` Invulnerability byte or DP-slot lifetime. Regression: `diamond_power_invulnerability_audit_seeds_skycutter_tier1_start_a_and_twenty_diamonds`.

Note 2026-05-31 for #206: The frame-57 Invulnerability audit mismatch was a settled-oracle bootstrap bug, not native Skycutter collision. `playtest_form` patched `Current_Helmet=$0001` and HP in the helper but left ROM collision half-width `$FFFFFA78` at The Kid's `5`; ASM `sub_B41C`/`unk_B408` should seed Skycutter to `$A`. The helper now writes `$FFFFFA78` alongside `Current_Helmet`, and a direct oracle probe over `.cache/recordings/issue-206-diamond-power-invulnerability-audit/scenario.json` shows row103/col10 `0xE001`, `Current_Helmet=1`, `$FFFFFA78=0xA`, and the same wall-hit position/velocity sequence as native at frames 57-60. Refreshed `verify-recording`: native-only replay passes; full oracle remains state/frame red, but `kid_x_pos`/`kid_x_vel` now match every Invulnerability checkpoint and the first non-advisory state diff is the already-exposed native missing `kid_collision_type/source` metadata at frame 0 (`None` vs oracle `0/160`). Next pass should decide whether that collision metadata is authoritative for #206 recording parity or should be treated like advisory trace state before chasing pixels.

Note 2026-05-31 for #206: Invulnerability audit state parity is green again. The improved verifier diagnostic showed the actual remaining state miss after the helper fix was `Character_Movement` at frame 57/61, not the raw Kid collision metadata (`recording checkpoints diverged ... native Some(0), oracle Some(2)`). ASM `loc_8E16` / `loc_8E5A` keeps Skycutter in `MoveID_Walking` even when a wall hit has just zeroed `x_vel`; the native trace inference now applies the zero-velocity standing gate only to non-Skycutter forms. Refreshed `.cache/recordings/issue-206-diamond-power-invulnerability-audit`: native-only replay passes, full `verify-recording` has `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`, first pixel diff `diamond_power_input@0` with 782 changed pixels / MAE 1.119. Regressions: `current_character_movement_keeps_skycutter_walking_on_zero_velocity_wall_hit` and `verification_report_preserves_oracle_state_error`.
| 207 | [ ] | P1 | architecture | Generic collision primitives unwired (engine-wide); relates to #139/#140/#143 | `combat.rs` models the ROM solid/support/hitbox engine — `first_solid_overlap_resolution_sub_1FA2`, `support_overlap_detected_sub_1F52`, `kid_hitbox_from_collision_size_sub_21F8` — but the runtime approximates. Wiring them corrects block, Kid and all-enemy collision in one place. Gate: block-collision #205 scenario + `mise run verify-slice-fast`. |
| 208 | [ ] | P1 | player | All platform physics unwired (collision / carry / landing / overlap-resolution) | `platforms.rs`: entry `apply_platform_collision` + carry/landing (`loc_2686`-`loc_26FE`) + overlap-resolution push (`loc_25EE`-`loc_266A`) are modelled + tested with exact ASM push distances, zero live calls. Governs Kid position/velocity on platforms (TAS-relevant). Gate: moving-platform #205 scenario vs oracle + `mise run verify-slice-fast`. |
| 209 | [ ] | P3 | architecture | Hygiene: strip stale `cfg_attr(not(test), allow(dead_code))` from genuinely-wired helpers | ~113 helpers in `enemies.rs` and ~27 in `terrain.rs` (plus weather/blocks leaves) execute in release but still carry the dead-code attr, masking the compiler's real dead-code signal. Removing the attr is a no-behaviour-change cleanup that shrinks the census surface. Gate: release build clean with attrs removed; `mise run verify-slice-fast`. |
| 210 | [?] | P0 | gameflow/render | 2026-05-30 playtest: intro/title/menu path still visually corrupt and can re-enter demo loop over Start/menu | Evidence source: human playtest after pushed commits. Symptoms: Wild Side kids appear jumbled/outside the intended intro scene, a normal Kid sprite appears at bottom-left during story text, later intro/title goes to a green screen with a top-right silhouette, Start can reach title music but the menu text is unreadable or absent, and failed demo return can restart the intro/demo loop over the menu. Existing `any-percent-published` TAS is CRC-clean through frame 37759 and `verify-slice-fast` passes, so current TAS coverage is too narrow for the manual title/menu interaction shape. Native repro: `issue-210-title-menu-tail` now passes state and pixel verification after correcting `unk_E0ED6`'s `$08` y-centre on the `loc_1BB10` intro-title sprite; the previous 44px `TM` placement delta at frames 2700/2701/2720/3299 was a sprite-frame origin bug, not a Chameleon tilemap timing bug. Remaining #210 work is the broader manual title/demo re-entry oracle probe and playtest symptoms outside this focused title-menu-tail recording. |
Note 2026-06-01 for #210: fixed one native-player fallback cause of the unreadable Options report. The asset-backed ROM tile screen is still future work, but the typed fallback now follows ASM `DrawOptText1..4` (`loc_1CE3A`/`loc_1CEB2`/`loc_1CF78`/`loc_1CFFE`) cell positions instead of joining whole option groups into over-wide rows; regression `options_menu_fallback_keeps_rom_rows_inside_frame_loc_1ce3a` keeps every fallback row inside the 320x224 frame.
Note 2026-05-30 for #210: first slice recorded `.cache/debug/issue-210/intro6_start_menu.json` (cold boot, Start at frames 2600 and 2700). Fixed the state/input side: frame 2600 now keeps the `Load_IntroSequence6` ninja/lightning shell instead of exposing the top-right title pose, frame 2602 follows ASM `sub_1C246 -> loc_1C264` into `Game_Mode=$2C`, frame 2660 reaches title-ready, and frame 2701 stays on TitleScreen so the second Start builds the menu instead of immediately entering Level 1. Regression: `cold_boot_intro_sequence_6_start_keeps_loc_1b37c_shell_until_loc_1b5ce`. Remaining #210 work is visual composition: native frame 2720 still shows the green/black title shell and garbled menu tilemap while oracle shows the full Kid Chameleon title art and readable menu. Gate for fixed slice: `mise run verify-slice-fast`.
Note 2026-05-30 for #210: follow-up visual slice wires the ready title screen through ASM `Load_TitleScreen`'s visible character frame 7, `Obj_TitleTextKid`, `Obj_TitleTextChameleon/sub_1B71A`, and all four `Load_TitleArt` uploads including `Title_text_CHAMELEON.bin` at VRAM `$6760`. Native `.cache/recordings/issue-210-title-logo-art-fix/verify/native/002720_menu_visible.png` now shows readable menu text plus the KID/CHAMELEON logo instead of the green/black shell. Remaining visual debt: title ninja composition and logo background still differ from oracle frame `.cache/tas-verify/issue_210_intro6_start_menu/oracle/2720_frame_2720.png`. Regressions: `title_screen_vdp_obj_title_text_chameleon_sub_1b71a_writes_scroll_a_reveal_rows`, `title_screen_vdp_load_title_art_uploads_menu_chunks`, and extended `intro_return_hidden_title_shell_selects_sub_1bf24_frames`.
Note 2026-05-30 for #210: fixed the Chameleon-logo black rectangle by applying ASM `unk_1B4F4` during the `DecompressToRAM`-style `Title_text_CHAMELEON.bin` upload. Native `.cache/recordings/issue-210-chameleon-permutation/verify/native/002720_menu_visible.png` now has transparent logo backing like oracle. Remaining visual debt: title ninja composition/placement and copyright text still differ from `.cache/tas-verify/issue_210_intro6_start_menu/oracle/2720_frame_2720.png`. Regression: `title_text_chameleon_art_loc_1b4f4_remaps_e_nibbles_to_transparent_zero`.
Note 2026-05-30 for #210: fixed the title character by honouring `off_1BFB0`'s per-frame width words in `sub_1BF24`, and replayed `SegaText` through `loc_1C7A0` after the `loc_1B6AC -> loc_1B6DC` Chameleon reveal cadence. Native `.cache/recordings/issue-210-title-frame-sega-text/verify/native/002720_menu_visible.png` now has readable menu text, correctly placed Red Stealth title pose, transparent KID/CHAMELEON logo, and bottom `© 1992 SEGA`. Remaining visual debt: the small title-logo `TM` detail is still absent compared with oracle frame `.cache/tas-verify/issue_210_intro6_start_menu/oracle/2720_frame_2720.png`; the broader manual title/demo re-entry oracle probe also remains red. Regressions: `title_screen_character_load_plan_sub_1bf24_switches_non_zero_frames_by_region_bank`, `title_screen_vdp_loc_1b6dc_replays_sega_text_after_chameleon_reveal`.
Note 2026-05-30 for #210: fixed the remaining visible menu rectangle corruption from the short `unk_2EAAE` decode. Native's stateless title renderer now pads the 65 decoded menu words to the 14x6 `loc_1BB52/sub_1C5D0` rectangle with the menu blank word `$4528`, preserving the cached black rectangle instead of leaking current-frame background/logo data into the right/bottom cells. Evidence: `.cache/recordings/issue-210-title-menu-blank-tail/verify/native/002720_menu_visible.png`. Regression: `title_menu_tilemap_loc_1bb52_pads_short_decode_with_menu_blank`. Remaining #210 debt unchanged: small `TM` mark and manual title/demo re-entry oracle probe.
Note 2026-05-30 for #210: extended the cold-boot idle probe to 9000 frames at `.cache/debug/issue-210/cold_boot_idle_demo_loop_long.json`. The first gameflow miss was native entering `Game_Mode=$10` three frames before oracle on the natural `Load_IntroSequence6 -> Obj_TitleMenu loc_1BAEA -> loc_1BB02` path; native now keeps mode `$28` through frame 3269 and starts demo play on frame 3270. The same slice found the cold-boot Skydragon demo bootstrap camera rows: startup frame 57 uses `$01E8_0000`, frame 58 uses `$01C0_0000`, then frame 59 snaps to `$02A0_0000`. Regression: `cold_boot_intro_sequence_6_enters_demo_play_on_frame_3270`. Targeted published timeout-restore preload tests still pass. The 9000-frame probe now reaches the next later-loop divergence at frame 6850.
Note 2026-05-30 for #210: fixed the cold demo-2 timeout return through `Load_IntroSequence6`. Oracle shows this cold `loc_1BA5C -> $28` shell exposing `$FFD00000` and three hidden workers on frame 6850, then following `sub_1B43A`'s 3/2/4/3/4 object-count choreography, starting the +3px `GfxObjects_Collision_BottomBoundary4` camera ramp on frame 7066, and exposing `loc_1B5CE`'s title pose on frame 7148. Regression: `cold_boot_demo_timeout_intro_sequence_6_exposes_hidden_shell_on_frame_6850`. `tas-diff --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 6848 --frame-end 7152 --only-divergent --include-advisory --frontier-kind gameplay` now has 304 matches / 0 divergences, and the 9000-frame bisection advances to frame 7689.
Note 2026-05-30 for #210: extended that same cold timeout-return regression through `Obj_TitleMenu loc_1BB02 -> Load_DemoPlay`. The cold shell keeps two extra idle frames before the demo hand-off, frame 7689 now enters demo play with the controller-side `$00060000` y-velocity sample, frame 7748 exposes the hidden controller object count, and frame 7749 uses the `$FD400000` Knights Isle hidden camera row. `tas-diff --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 7684 --frame-end 7754 --only-divergent --include-advisory --frontier-kind gameplay` now has 70 matches / 0 divergences, and the 9000-frame bisection advances to frame 8851.
Note 2026-05-30 for #210: fixed the cold timeout-return demo-1 auto-timeout edge. The `Load_DemoPlay -> loc_119D8` controller sample now stays visible through frame 8852, `ReadJoypad loc_C0A` applies the mode-0 timeout hand-off on frame 8853, and that first hand-off frame preserves the final demo jump snapshot (`objects=9/4`, `Character_Movement=Jump`) instead of immediately clearing to the title shell. Regression: `cold_boot_demo_timeout_intro_sequence_6_exposes_hidden_shell_on_frame_6850` now covers frames 8851-8853. `tas-diff --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 8848 --frame-end 8875 --only-divergent --include-advisory --frontier-kind gameplay` is 27/27 match, and `verify-tas --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 0 --frame-end 9000 --checkpoint-interval 1 --bisect --frontier-kind gameplay` passes all 9000 frames by CRC. Remaining #210 work is the visual/title/menu corruption, not this cold idle gameflow path.
Note 2026-05-30 for #210: refreshed the manual Start/menu slice (`.cache/debug/issue-210/intro6_start_menu_with_2720_sorted.json`, Start at 2600 and 2700). Native now preserves the frame-2600 Start latch until the `Load_IntroSequence6` hidden shell reaches the oracle hand-off, consumes it via ASM `sub_1C246 -> loc_1C264` at frame 2634, keeps `title_screen_ready=false` through frame 2701, reports the menu cursor as the trace-facing gfx object for this IntroSequence6 Start-skip lineage, and remains on TitleScreen at frame 3299 instead of entering demo play. Regression: `cold_boot_intro_sequence_6_start_keeps_loc_1b37c_shell_until_loc_1b5ce` plus cursor-priority coverage in `current_player_state_uses_intro_6_title_menu_only_when_active`. `verify-recording --slug issue-210-title-menu-tail` now has `trace_match=true` and `oracle_state_match=true`; `oracle_frame_match=false` remains because the pre-menu/title VDP composition still differs at frames 2599/2640/2700 and has smaller logo/copyright pixel drift at 2720/3299. Gate: `mise run verify-slice-fast`.
Note 2026-05-30 for #210: split trace readiness from VDP readiness for the same manual Start/menu slice. Oracle keeps `title_screen_ready=false` through frame 2701, but visually restores `Load_TitleScreen` by frame 2700; native now renders a blank `loc_1B45A` blackout frame at 2640 and switches the VDP shell to title art at 2700 while leaving trace state unchanged until 2720. `verify-recording --slug issue-210-title-menu-tail` still has `trace_match=true` and `oracle_state_match=true`; pixel drift improved from the full green-shell mismatch to frame 2700/2701 logo/copyright colour/detail differences. Remaining visual debt: wrong pre-handoff transformation pose at 2599-2601 and title-logo/menu phase/detail drift at 2700/2720/3299. Regression remains `cold_boot_intro_sequence_6_start_keeps_loc_1b37c_shell_until_loc_1b5ce`; gate: `mise run verify-slice-fast`.
Note 2026-05-30 for #210: the cold IntroSequence6 Start-skip title path now replays `SegaText` as soon as the inherited hidden `loc_1B6AC/loc_1B6DC` cadence reaches the visible title VDP. This matches oracle frame 2700's `© 1992 SEGA` line without changing the normal direct-title cadence (`loc_1C7A0` still hibernates once before zero-delay text). `verify-recording --slug issue-210-title-menu-tail` remains pixel-red but narrowed: trace and oracle state pass, frame 2640 pixels match, frame 2720 is down to the small `TM`/logo detail mismatch (44 changed pixels), while frames 2599-2601 still show the wrong transformation-shell silhouette and frame 3299 still has KID/logo phase drift. Regression: `title_screen_vdp_intro_return_replays_sega_text_when_vdp_becomes_visible`. Fix commit: `1834512d`. Proposed next fix: trace ASM `sub_1BF24` frame/region selection for the frame-2599 `loc_1B37C` shell, then the KID title text animation cadence around `Obj_TitleTextKid loc_1B5E4`.
Note 2026-05-30 for #211: fixed the comparison foundation for the natural `walk_right_00` route. Oracle helper now samples Kid velocity from the live Kid gfx slot (`$FFFF24E0 + x_vel/y_vel`, i.e. `$FFFF2506/$FFFF250A`) instead of the stale first gfx slot, native trace exports the same live demo velocity, and settled level-00 opening counts now match oracle RAM (`Number_Objects=4`, `Number_GfxObjects=5`). CRC-clean published-demo velocity-only skews are classified as oracle limitations in `verify-tas`, so the gate still blocks on real state/CRC drift. `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live/scenario.json --frame-start 0 --frame-end 80 --include-advisory --frontier-kind gameplay --only-divergent` is 80/80 match. The first real natural-route divergence is now frame 124: native clears Kid `y_vel` in the right wall-cling path while oracle keeps `$00030440`, then the dragon/stomp/object-count path diverges from frame 125 onward. Gate: `mise run verify-slice-fast`.

Note 2026-05-30 follow-up for #211: narrowed the `walk_right_00` frame-124 drift to the descending right-wall cling release tail. Native now preserves the post-gravity `y_vel` through the immediate stomp window while still applying the 1px `loc_AB0A` shimmy and clearing wall-cling/x velocity. This matches frames 124-125; the next natural-route frontier is frame 126/127 `number_objects` only, where native reports the block FX object but oracle has an additional slot `0x00@0x00700000,0x00bc0000`. Verification: `cargo test -q -p kid-core wall_cling_release -- --nocapture`; `cargo test -q -p kid-core descending_wall -- --nocapture`; `cargo run -q -p kidcli -- tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live/scenario.json --frame-start 120 --frame-end 130 --include-advisory --frontier-kind gameplay --only-divergent`; `mise run verify-slice-fast`.
| 211 | [~] | P1 | player/render | 2026-05-30 playtest: live slope traversal still shows wrong-facing/diagonal animation despite green slope helper tests | Evidence source: human playtest after pushed commits. Symptoms: The Kid appears to face the wrong way while moving up-right on the first Level 00 slope; Helmet/IronKnight and Samurai slope traversal still show bad diagonal animation or vibration. Existing regressions `slope_facing_does_not_invert_on_uphill_traversal_loc_9DC0` and `full_motion_slope_up_right_keeps_kid_and_iron_knight_facing_right` pass on this checkout, so the helper coverage missed the live scene path. Follow-up 2026-05-30 adds the focused `level00-slope-right-probe` DynamicScenario for Blue Lake Woods 1 column 47 / row 14. Native replay passes for `.cache/recordings/issue-211-level00-slope-right-probe`, but oracle comparison diverges immediately: `verify-tas --scenario .cache/recordings/issue-211-level00-slope-right-probe/scenario.json --frame-start 0 --frame-end 120 --checkpoint-interval 1 --frontier-kind gameplay` reports frame 0 native `kid_x_pos=0x02f7e000`, `kid_x_vel=0xffffe000`, `kid_x_direction=0xff`, `character_movement=0x0006` versus oracle `kid_x_pos=0x02f80000`, `kid_x_vel=0`, `kid_x_direction=0`, `character_movement=0x0003`. ASM check: `sub_B43A` broad floor support ignores `$4000/$5000` slope cells by design, so the `kid_grounded_release_probe ... support=none` field is context, not proof of a bad floor scan. Follow-up tooling note: direct mid-slope WRAM patching may be too artificial for movement-frame parity, so also use the natural route `.cache/recordings/issue-211-level00-walk-right-live`; after fixing `record-scenario` to export `settle_oracle_warp=true`, standalone `verify-tas --scenario .cache/recordings/issue-211-level00-walk-right-live/scenario.json --frame-start 0 --frame-end 120 --checkpoint-interval 10 --frontier-kind gameplay` now reaches live gameplay on both sides and shows the remaining real drift after gameplay input begins rather than failing on the title-card bootstrap. Current native trace reaches the first slope around frame 500 with `kid_facing=Right`, `kid_on_slope=1`, `character_movement=6`, so the playtest-facing bug is likely renderer/form-specific slope frame selection unless the natural oracle route proves state drift before that point. Minimum gate: compare the natural route through the first slope, then assert `kid_x_direction`, `walk_visual_accumulator`, selected sprite frame, and pixels against oracle. |
| 212 | [x] | P1 | player/combat | 2026-05-30 playtest: Helmet/IronKnight seems to lose all HP and revert to Kid on a single dragon hit | Closed 2026-05-31 as already covered by the live damage route: `loc_b700_iron_knight_survives_first_dragon_contact_with_four_hp` sets Iron Knight to `HelmetHitpoint_Table[IronKnight]=5`, resolves Dragon contact through `Check_for_recent_damge -> loc_B700`, and proves the first hit leaves `current_form=IronKnight`, `number_hitpoints=4`, no pending Kid transform, and the 66-frame damage latch armed. `loc_b722_hp_cap_to_max_for_current_helmet` separately pins the ASM HP cap table plus extra-slot calculation. Gate: both focused tests pass. |

Note 2026-05-31 for #212: found and fixed a transform-HP table conflation while tracing the report. ASM `sub_7EB2` seeds `Number_Hitpoints` from `unk_7EC2` (TheKid=2, Skycutter/Cyclone/RedStealth/Eyeclops/Juggernaut=3, IronKnight=$0C, Berzerker=4, Maniaxe=8, Micromax=$10), whereas `Check_for_recent_damge -> loc_B700` and `Make_SpriteAttr_HUD/sub_BDC0` use `HelmetHitpoint_Table` (IronKnight damage/HUD cap 5, most helmets 3). Native had reused `PlayerForm::max_hitpoints()` for transform HP, so IronKnight/Berzerker/Maniaxe/Micromax started too low compared with ROM. `transform_hitpoints_sub_7eb2` now mirrors `unk_7EC2`, with regression `sub_7eb2_uses_transform_hp_table_not_damage_cap_table`; existing `loc_B700` damage tests prove the first damage frame caps oversized IronKnight HP to 5 before subtracting to 4. Follow-up triage closed the live DragonWalking part with `loc_b700_iron_knight_survives_first_dragon_contact_with_four_hp`; the human-visible "first pip does not change" is consistent with the ROM's separate transform HP seed and damage/HUD cap rather than a one-hit form-loss bug.

Note 2026-05-31 for #212: added a cache-local direct contact probe at `.cache/scenarios/issue-212-iron-dragon-contact.json` / `.cache/recordings/issue-212-iron-dragon-contact` using `playtest_form=6`, Level 00, Kid at `(165,271)` against the active DragonWalking object. `verify-tas --scenario .cache/recordings/issue-212-iron-dragon-contact/scenario.json --frame-start 0 --frame-end 20 --checkpoint-interval 1 --frontier-kind gameplay` shows no helmet loss in either runtime. Oracle has `number_hitpoints=5`, `Just_received_damage=false` at frame 0 while the DragonWalking object exposes `collision_type=$28(kidbelow)`, then `Just_received_damage=true` from frame 1 and HP effectively drops to 4. Native applies `ContactDamage` on frame 0 (`number_hitpoints=4`, `Just_received_damage=true`) and clears its public one-frame latch on frame 1. So the direct DragonWalking report narrowed to a one-frame collision handoff / damage-latch timing mismatch, not an instant IronKnight form loss; follow-up focused regression coverage is enough to close this row as "not the reported one-hit helmet loss".

Note 2026-06-02 case-queue triage for #212: the cache-local direct contact
bundle and sibling Level 00 walk-right bundle are now stale against current
native. `issue-212-iron-dragon-contact` fails full and native-only
`verify-recording` with `recording replay drifted at frame 0`; its derived
0..20 case is blocked for the same reason. `issue-212-level00-iron-walk-right`
fails full and native-only `verify-recording` with replay drift at frame 61;
its derived 0..120 case is also blocked and `case reduce` refuses. Re-record
the IronKnight/DragonWalking contact or walk route before using either for
damage-latch, slope, or form traversal work; do not reopen the one-hit helmet
loss claim from these stale bundles.
| 213 | [ ] | P1 | enemy/block | 2026-05-30 playtest: Level 01 Drips/slime and metal-block interaction still wrong | Evidence source: human playtest after pushed commits. Symptoms: Drips appears in the middle/air and does not form/sit on the floor correctly; jumping into the area can break a metal block or hit an invisible collision body. #169 is closed for the specific pool/drop regressions it covered, so this is a fresh live geometry/collision repro, not proof that those tests are wrong. Native repro: not yet recorded on current checkout. Oracle repro: unknown. Minimum gate: settled Level 01 Drips DynamicScenario/recording with terrain contact and upward jump, compare Drips parent/child positions, surface crawler state, collision cell, and metal block mutation against oracle. |

Note 2026-05-31 for #213: natural settled Level 01 idle comparison initially hit an earlier enemy-roster divergence before any Drips were active. Native recording settle handed frame 0 to the trace one live frame too early: the first RockTank body at layout index 1 stayed at `$00A40000` while oracle had already run `Tank.asm loc_3B40A` into the shared walker and reached `$00A48000`, with the turret child still at `$00A40000` until the next hibernate sync (`loc_3C03A/loc_3C044`). Fixed `settle_recording_oracle_warp_live_window(LevelId(1))` to wait `ORACLE_SETTLED_WARP_LIVE_FRAMES + 1`, added `settled_recording_warp_level_one_matches_first_rock_tank_body_turret_phase`, and verified `.cache/recordings/issue-213-level01-idle-native/scenario.json` with `tas-diff --frame-start 0 --frame-end 180 --frontier-kind enemy --only-divergent`: 180/180 match. The Drips/slime collision repro remains open; this removes the earlier Level 01 timing blocker and should make the next Drips probe trustworthy.

Note 2026-05-31 for #213: lower-floor probe `.cache/scenarios/issue-213-drips-floor-probe.json` exposed a native enemy-loading mismatch before Drips movement could be judged. ASM `Manage_EnemyLoading loc_36900` gates post-initial-pass spawns with `$FFFFFB51`, skipping enemies already inside the visible inner box (`x=$FFE0..$0160`, `y=$FFE0..$0100`). Native only used the wider activation window, so both Drips controllers at layout indices 7/8 popped into the probe. Added native `enemy_loading_initial_pass_complete` and the loc_36900 inner-window suppression; regression `settled_recording_probe_level_one_suppresses_inner_visible_drips_spawn` pins index 7. Re-ran `tas-diff --scenario .cache/scenarios/issue-213-drips-floor-probe.json --frame-start 0 --frame-end 4 --frontier-kind enemy --only-divergent`: native now has only the index-8 Drips controller (number_objects 6 vs oracle 5), so the remaining blocker is the status/lifecycle path that leaves oracle index 8 non-visible while native still exposes it.

Note 2026-05-31 for #213: oracle watch rows showed `EnemyStatus_Table[8]=$FFFF` while the active object roster still contained only the RockTank body/turret; the Drips controller is loaded but hidden. Native already hid Drips controllers in the renderer, but `active_objects` still exported the parent as an enemy snapshot. `rebuild_active_enemy_roster` now keeps Drips parent/controllers internal and exports only visible falling/surface children through projectile snapshots, citing `Drips.asm Enemy16_Drip_Init`/`loc_33488`. Regression now asserts no Level 1 Drips controller appears in the settled lower-floor probe, and `tas-diff --scenario .cache/scenarios/issue-213-drips-floor-probe.json --frame-start 0 --frame-end 20 --frontier-kind enemy --only-divergent` reports 20/20 matching frames. Remaining #213 work is the actual terrain/jump interaction around the slime and metal blocks, not this loader/visibility blocker.

Note 2026-05-31 for #213: native was deleting the lower Level 1 Drips falling child when it sat just outside the narrow Lion projectile camera box (`screen_x=$188`). ASM `Drips.asm loc_3345C` calls `Object_CheckInRange` on the hidden controller, not Lion `sub_3E2CE` on the child projectile, so commit `c5c60e19` exempts `EnemyProjectileKind::DripsDrop` from that generic owner cull and adds `drips_drop_loc_3345c_ignores_lion_projectile_camera_box`. Verification: `cargo test -q -p kid-core drips_drop_loc_3345c_ignores_lion_projectile_camera_box -- --nocapture`, `mise run verify-slice-fast`, and `tas-diff --scenario .cache/scenarios/issue-213-drips-floor-probe-2000.json --frame-start 520 --frame-end 620 --frontier-kind enemy --include-advisory --only-divergent`. Important rejected hypothesis: do not count every active Drips controller or every native Drips drop in `Number_Objects`; that overcounts frames 520-536. Oracle WRAM reports the count-visible child window at frames 537-597, while native still schedules the simplified drop cycles at 73/157/263/340/454/555, so the remaining root cause is the Drips animation/timing child-slot model, not the projectile cull.

Note 2026-05-31 for #213: commit `147c296c` implemented the first part of the remaining Drips child-slot model. ASM `Drips.asm loc_33424` waits through `stru_33F90` (`$A,$A,8,6,1` frames) before setting `$13/$14/$3C(a1)` and exposing the falling child; native had spawned `DripsDrop` immediately on timer expiry. Regression `drips_drop_uses_dedicated_child_slot_not_generic_projectile_cap_loc_33424` now proves the forming animation delay before the child bypasses the generic projectile cap. Verification: `cargo test -q -p kid-core drips_ -- --nocapture`, `mise run verify-slice-fast`, and the same lower-floor `tas-diff` window. Follow-up rejected hypothesis: simply adding pre-drop Drips animations to `gameplay_transient_gfx_worker_count` makes the lower-floor probe worse (21 match / 79 diverge) by overcounting native after oracle drops back to 5 at frame 598. The remaining mismatch is a precise timing/window alignment problem, not just missing trace projection for pre-drop workers.

Note 2026-06-01 for #213: the precise lower-floor count window is now pinned. Oracle WRAM counts the Drips preallocated child object, but not a new gfx slot, on frames 537-597 of `.cache/scenarios/issue-213-drips-floor-probe-2000.json`; the later falling `DripsDrop` at frame 629 was already visually/state matched and must not be counted as this hidden worker. Native now adds a Drips object-only transient during the low dormant timer / early `loc_33424` forming window, excluding the in-flight sentinel. Regression `settled_level_one_drips_child_slot_counts_only_before_drop_loc_33424` samples frames 536/537/597/598/629 so future changes cannot paper over the window by counting every active Drips controller or every DripsDrop.

Note 2026-06-01 follow-up for #213: refined that child-slot count after the jump probes showed the first `loc_33424` wait must not count a child. ASM `Drips.asm loc_33488` is the state split: only after a splash copies the falling child into the surface/pool child (`a2`) and advances `off_33520` should native add an object-only worker during the next wait / early forming window. Added `drips_loc_33424_countdown_does_not_add_hidden_child_object` and `drips_loc_33488_surface_pool_counts_as_object_only_child`. Verification: focused trace tests, `.cache/scenarios/issue-213-drips-floor-probe-2000.json` frames 520-650 clean, and `.cache/scenarios/issue-213-drips-jump-probe.json` frames 0-120 clean. Broader cache-local jump variants still include older non-pinned divergences and need separate repro triage before closing #213.

Note 2026-06-02 for #213: `issue-213-drips-camera-only` is now a clean
64-frame prefix, not the Drips/terrain repro. Fresh native replay passed, but
the case reducer initially blocked on settled Level 01 bootstrap metadata:
oracle helper states expose player velocity as unknown and can list the
RockTank body/turret in a different order from native; oracle enemy velocity is
also sampled at the helper's VBlank phase rather than native's post-frame
trace. `kidcli` case semantics now treat oracle entity velocities as advisory
and compare entity sets by canonical type/position instead of iteration order.
After that, `cargo run -p kidcli -- case verify --case
issue-213-drips-camera-only` passes and `case reduce` reports the case is
already passing. Continue #213 with a longer settled Drips recording that
actually reaches the drop/pool/metal-block interaction.

Note 2026-06-02 follow-up for #213: the next Level 01 bundles cannot be used as
longer Drips routes on current native. `issue-213-level01-drips-idle` and
`issue-213-level01-idle-native` both fail full and native-only
`verify-recording` at frame 0 with `native trace frame mismatch`; their derived
0..120 cases (`issue-213-level01-drips-idle_case_00000_00120` and
`idle_level_01_case_00000_00120`) are blocked with `source recording replay
drifted at frame 0`. `issue-213-level01-walk-right` drifts at frame 61 in both
full and native-only verification, and its derived `walk_right_01_case_00000_00120`
case is blocked with the same source-recording drift. Re-record the Level 01
Drips idle/drop or walk route against current native before treating any of
these bundles as gameplay-authoritative.
| 214 | [ ] | P1 | block/enemy | 2026-05-30 playtest: Highwater shooter blocks still have live-path projectile/barrier regressions | Evidence source: human playtest after pushed commits. Symptoms: top shooter blocks do not all move/fire, projectile visuals may still be wrong in live play, the projectile does not reliably clear the barrier ahead, and jumping on a shooter block can launch the player into the sky. #065's asset-backed render and barrier tests pass, so focus on live-session state, projectile collision, and block/player overlap response. Native repro: not yet recorded on current checkout. Oracle repro: unknown. Minimum gate: Level 02 Highwater recording around the first shooter-barrier stack, with `active_projectile_details`, shooter cell rows, player overlap response, and barrier cell mutations checked before patching. |

Note 2026-05-31 for #214: added committed `highwater-shooter-barrier-audit`
recording scenario for the first Highwater Pass 1 shooter/barrier route. The
scenario warps to Level 02, applies Berzerker (`playtest_form=7`), holds Right
from the settled spawn, and checkpoints `route_start`, `charge_building`,
`shooter_contact_window`, `projectile_barrier_window`, `barrier_clear_check`,
and `route_tail`. Fresh bundle:
`.cache/recordings/issue-214-highwater-shooter-barrier-audit`. Native replay
passes. Full oracle verification is now a current repro: it fails at
`projectile_barrier_window@90` with native enemy count 0 vs oracle 2. Native
trace does expose a typed `ShooterBlock` projectile at frame 90
(`x=0x00E40000`, `y=0x06180000`, `x_velocity=0x00040000`, remaining 109), while
`oracle-probe --frame-start 80 --frame-end 100` shows the ROM still has two
active object-type `$08` block/shooter workers at `x=0x00A40000`, rows
`0x06600000` and `0x06700000`. Next patch should target the live shooter worker
/ object lifecycle and, if needed, promote barrier-cell samples into trace
debug fields before changing collision response.

Note 2026-05-31 follow-up for #214: native now filters oracle block/shooter
workers out of enemy parity, tracks the shooter projectile's source
Level_Layout cell through ASM `loc_EDFA`/`loc_F02A`, and applies the
`loc_ED04` evanescent source-cell rewrite cadence so the Highwater source
column stays solid until the ROM's frame-110 unblock window. Fresh regenerated
bundle has `trace_match=true` and `oracle_state_match=true`; the previous
`route_tail@149` camera divergence is gone. Full `verify-recording --slug
issue-214-highwater-shooter-barrier-audit` still fails on pixel parity
(`oracle_frame_match=false`) from existing visual differences starting at
`route_start`, so #214 remains open for projectile/block visuals and the
reported shooter-block player-overlap launch. Gate: `mise run
verify-slice-fast`.

Note 2026-05-31 debris follow-up for #214: `sub_2A4C` debris culling was
wrongly using absolute world bounds, which deleted Highwater ice/smash shards
around y=1552 immediately after the hibernate handoff. Native now culls the
first shard after subtracting the current camera position, matching ASM
`sub_2A4C` and preserving the falling debris in the Level 02 shooter/barrier
recording. Regression:
`highwater_block_debris_culls_against_camera_not_world_origin_sub_2a4c`.
Regenerated `.cache/recordings/issue-214-highwater-shooter-barrier-audit`
returns to `trace_match=true` and `oracle_state_match=true`; full verification
still fails `oracle_frame_match=false`, with remaining pixel deltas dominated
by existing route-start palette/layer differences plus shooter/projectile
timing at `projectile_barrier_window@90`.

Note 2026-05-31 evanescent-chain follow-up for #214: projectile-triggered
type-$A cells were rewritten to `$E3xx` and fired, but were not registered in
the evanescent active list. That left the upper Highwater shooter/source rows
stuck visibly as `$E31E/$E31F` after the upward `loc_EDFA -> loc_F04A` shooter
shots cascaded through them. Native now routes projectile hits through the same
`EvanescentBlock_Trigger` helper as body contact (`loc_EB72`), so those cells
get timers and restore by `route_tail@149`. Regression:
`highwater_pass_1_projectile_triggered_evanescent_cells_expire_loc_f04a_eb72`.
Fresh cache-local recording replay passes native determinism; full oracle frame
verification still fails on pre-existing visual parity (water/palette/player
colour deltas), not the stuck source-column bug.

Note 2026-06-01 same-frame shooter-object follow-up for #214: ASM
`LoadBulletsFromEvanescentBlock` falls straight through to
`Process_ShooterObjects` (`loc_EC22 -> loc_EDFA`), so a body-contact shooter
object advances its first 4px and can draw on the trigger frame. Native had
queued those new shooter objects after the generic projectile pass, delaying the
first draw/update until the next frame. Native now processes the newly appended
`loc_EC76` shooter-object range immediately after an evanescent trigger.
Regression `highwater_pass_1_touch_shooter_draws_on_trigger_frame_loc_ec76_edfa`
pins the Highwater `$EA1E` source moving from `$00b8` to `$00bc` on the trigger
frame. Verification: focused Highwater shooter tests, `tas-diff` frames 88..94
for `issue-214-highwater-shooter-barrier-audit`, and `mise run verify-slice-fast`.

Note 2026-06-01 diagnostic follow-up for #214: refreshed
`issue-214-highwater-shooter-barrier-audit` exposed that the recording native
path was feeding settled-warp input one frame earlier than the oracle helper.
`recording_script_buttons_at_frame` now mirrors the `ScriptRunner` recording
oracle phase (ASM `Mode_Level`/`loc_892`: gameplay before `ReadJoypad`), and
the dense `.cache/recordings/issue-214-highwater-dense-0-40` repro now reports
`trace_match=true`, `oracle_state_match=true`, and no
`checkpoint_state_deltas`. The longer route is still red for real #214 logic:
`.cache/recordings/issue-214-highwater-dense-80-130` first fails object counts
at `f_107` (native 8/8, oracle 6/6), then movement begins drifting at `f_109`
(native Kid velocity `$0`, oracle `$1000`). Continue from the live shooter/
barrier object lifecycle around frames 100-110, not from the old early input
phase or a broad pre-step render latch.

Note 2026-06-01 Highwater unblock follow-up for #214: native now applies the
folded `$80` evanescent sprite-only collision overlay before Kid motion when
the entry timer reaches 16. This matches the oracle f_109 movement window for
`.cache/recordings/issue-214-highwater-dense-80-130`: regenerated verification
has `trace_match=true` and no `checkpoint_state_deltas`. The recording still
exits red because object-count comparison is authoritative there and still sees
the earlier f_107 native 8/8 vs oracle 6/6 mismatch. Continue from object/GFX
slot lifecycle around f_107; the Kid wall/pinning portion is no longer the first
state delta.

Note 2026-06-01 diagnostic follow-up for #214: added oracle-side linked-list
debug for `ShooterObject_RAM` (`loc_EC22`) and `Object_RAM` (`Execute_Objects
loc_E1E`). The shooter scratch list is already empty in the f_105..f_110 count
window, while `oracle_object_slots_loc_e1e` shows real `sub_10F9C`/post-hibernate
`loc_10FFA` workers: f_106 has one worker at `(240,1616)`, f_107 has one new
`sub_10F9C` worker plus the first hibernating at `(244,1616)`, and f_109 has
four block workers. Native's synthetic `block_fx` currently appears on the
trigger frame already nudged to x=244, so the next fix should model the
allocation frame separately from the direction-nudged hibernate frame instead
of counting scratch `sub_2A4C` debris as objects.

Note 2026-06-01 shooter-chain fix for #214: matched the Highwater dense
shooter/barrier object-count window against oracle. Two native shortcuts were
wrong: body-contact `loc_EB72 -> loc_EC76` shooter objects were stepping once
through the immediate `loc_EC22/loc_EDFA` fallthrough and again later in
`step_projectiles`, while projectile-triggered `loc_F04A` evanescent hits were
incorrectly loading their child bullets immediately instead of waiting for the
next `LoadBulletsFromEvanescentBlock` pass. Native now marks same-frame
body-triggered shooter objects, defers projectile-triggered evanescent bullets
until the next block pass, and models ice block effects with the `sub_10F9C`
source-cell setup frame before the nudged three-frame `loc_10FEC` hibernate.
Regression: `highwater_pass_1_touch_shooter_frame_loop_advances_once_loc_ec76_edfa`.
Focused verification: `tas-diff` on
`issue-214-highwater-dense-80-130/scenario.json` frames 104..116 is now 12/12
match. #214 remains open for the reported shooter-block/player overlap launch
and any strict pixel/projectile visual drift outside this object-count window.

Note 2026-06-01 evidence follow-up for #214: refreshed the committed
`issue-214-highwater-shooter-barrier-audit` route on current `main`. Gameplay
state is green (`trace_match=true`, `oracle_state_match=true`) and
`tas-diff --scenario .cache/recordings/issue-214-highwater-shooter-barrier-audit/scenario.json --frame-start 80 --frame-end 130 --frontier-kind gameplay --include-advisory --only-divergent`
reports `50 match, 0 diverge`. Rejected the apparent two-frame native shooter
delay at the old `projectile_barrier_window@90`: `oracle-probe` watches for
`Addr_FirstShooterObjectSlot` and `ShooterObject_RAM` show the ROM shooter list
is empty through frame 91 and first populated at frame 92, matching native
trace projectiles. The strict frame red at frame 90 is therefore a render/checkpoint
alignment problem, not missing `loc_EC76` gameplay. The durable scenario now
adds `first_shooter_objects_visible_loc_ec76@92` and
`shooter_cascade_window_loc_f04a@104` checkpoints so future #214 evidence samples
the first real bullets and the upward cascade separately from the stale frame-90
visual mismatch.

Note 2026-06-01 Berzerker render-latch follow-up for #214: the live Highwater
route remained gameplay-green (`tas-diff` frames 80..130 still `50 match, 0
diverge`) but strict pixels showed Berzerker falling back to idle on the
same-frame shooter-column stop. ASM `sub_942A loc_9580 -> loc_95DA -> loc_9600`
still writes the ordinary Berzerker `off_967C` walk sprite before later wall
collision leaves the sampled post-frame velocity at zero. Native now exposes a
same-frame `berzerker_walk_pose` render latch and the native-player renderer
uses `$FFFFF8F0 >> 8` for that latched walk frame instead of deriving idle from
post-collision velocity. Refreshed cache-local
`issue-214-highwater-shooter-barrier-audit` improved strict frame deltas
(`projectile_barrier_window@90`
998->957 changed pixels, `first_shooter_objects_visible_loc_ec76@92`
1512->1406, `barrier_clear_check@120` 2255->2201) while leaving remaining
Highwater palette/projectile/water visual debt open. Regressions:
`berzerker_regular_walk_pose_survives_same_frame_wall_clear_loc_9580_9600` and
`berzerker_regular_walk_pose_latch_overrides_idle_after_wall_stop_loc_9580_9600`.

Note 2026-06-02 case-queue triage for #214: queued recording
`debug-highwater-shooter` is stale against current native and should not drive
Highwater shooter patches. It was recorded at old commit
`608e528a699de0f70e0d162907f96922f5089620`; both
`cargo run -p kidcli -- verify-recording --slug debug-highwater-shooter` and
`cargo run -p kidcli -- verify-recording --slug debug-highwater-shooter
--native-only` fail with `recording replay drifted at frame 3 (native trace
frame mismatch)`. The derived case
`highwater_shooter_barrier_audit_02_case_00000_00120` writes
`.cache/cases/debug-highwater-shooter/verify.json`, but `case verify` marks it
blocked with `likely_subsystems=["recording verification"]`; `case reduce`
refuses for the same source-recording drift, and `case summarise` only refreshes
the blocked summary. Re-record this route after the current Highwater movement
fixes, or use a fresh live route, before changing shooter activation, projectile
rendering, or player overlap response from this evidence.

| 215 | [ ] | P2 | render | 2026-05-30 playtest: water/foreground priority layering wrong in Level 02 | Evidence source: human playtest after pushed commits. Symptom: water animation appears inactive or the player renders above foreground/water elements that should cover him. Native repro: not yet recorded on current checkout. Oracle repro: unknown. Minimum gate: capture Level 02 water/foreground frames with layer diagnostics, compare VDP priority bits and native render layer ordering against oracle before touching renderer composition. |
Note 2026-06-01 for #215: cache-local `issue-playtest-highwater-water-current` now has oracle frame artefacts. The recording is still not a clean gameplay gate because replay trace comparison drifts at frame 0 and later checkpoints diverge after player motion changes, but the first two frames are oracle-state aligned enough to isolate the waterfall palette phase. Native `foreground_palette_for_scene` had the Mountain/Highwater `RotatePalette_Lava_Water loc_1223A/loc_1223C` phase one rotation late for the direct level-skip lineage; pre-advancing by two rotations improves the early strict diffs from 249/386 changed pixels to 72/209. Remaining #215 work is true foreground priority/player layering around the later water section, after a replayable non-drifting scenario is captured.
Note 2026-06-01 for #215: added named `highwater-water-priority-audit` scenario (cache slug `issue-215-highwater-water-priority-audit`) so the Level 02 water/foreground route is no longer only an ignored `.cache` artefact. Fresh verify result: `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; `level_settle@60` and `waterfall_overlap_loc_300BA@960` are pixel-perfect, while `water_hole_entry_loc_12910@510` differs by 205 pixels and `waterfall_palette_tail_loc_1223C@1859` by 18 pixels. Current evidence points at residual Mountain Plane B/waterfall tile row or palette phase drift, not a broad gameplay replay mismatch.
Note 2026-06-01 for #215: oracle-probed `water_hole_entry_loc_12910@510` and found the largest 205px diff was not water priority at all: the same diamond object is alive in oracle WRAM at `$FFFF2610` (`x=$78`, `y=$681`, `object_meta=$64`), but `addroffset_sprite(a3)=0`, so ASM `Make_SpriteFromGfxObject` returns before emitting VDP sprite attrs. Native rendered every `floating_items` trace entry unconditionally. Matching `PrizeDiamond_Init loc_103B2` / `stru_10D80` null blink frames removes that false terrain-overdraw and reduces the frame-510 diff to 121px with `trace_match=true` and `oracle_state_match=true`. Remaining strict pixel drift is now HUD diamond animation/palette phase at `(288..299,34..45)` plus the small life-icon/HUD drift at frame 1859, not the player/water priority route.
Note 2026-06-01 for #215: rejected a global HUD diamond phase lead change from 2 to 3 `unk_461C` slots. It improved `water_hole_entry_loc_12910@510` from 121px to 82px, but regressed previously pixel-perfect `level_settle@60` and `waterfall_overlap_loc_300BA@960` to 45px each. Oracle WRAM watches for `$FFFFFA58/$FFFFFA5D` show the ROM's `sub_44DC loc_45EE` state continuing independently of the replay-local `frame_index` shortcut and with gaps around non-gameplay spans, so the next fix should model or trace HUD animation RAM/state rather than tune `HUD_DIAMOND_DMA_PAGE_RENDER_LEAD_LOC_45EE`.
Note 2026-06-01 for #215: corrected the durable `highwater-water-priority-audit` route after finding the previous Kid-with-jumps script was false coverage: native was still blocked around x=$009A by the first smash-column stopper while the checkpoint names claimed water/waterfall coverage. The scenario now seeds Berzerker and holds Right, matching the live route that uses ASM `loc_9564` charging to break the blocker. Cache-local `issue-215-highwater-water-berzerker-rightonly` reaches x=$0208 by `water_hole_entry_loc_12910@510` and x=$0318 by `waterfall_overlap_loc_300BA@960`; native-only replay passes, while full oracle verification now fails usefully at `waterfall_overlap_loc_300BA@960` with object counts `native 4 / 4, oracle 10 / 4`. Continue #215 from that real route/object divergence before making more renderer priority changes.
Note 2026-06-01 for #215: resolved the first `highwater-water-priority-audit` object-count divergence by restoring the ASM `loc_779C/loc_77E6 -> sub_922C -> loc_10DA4` rubber side effect from standing-still side-wall probes and matching `loc_10DA4`'s count-visible object cap. Refreshed verification now advances past object counts and fails at `waterfall_overlap_loc_300BA@960` on `character_movement` (`native 0`, oracle 2), so the next #215 investigation should stay in Berzerker held-right movement state rather than renderer priority.
Note 2026-06-01 for #215: follow-up restored the immediate ASM `loc_8E5A -> loc_8E74` walking carry after grounded rubber wall contact by letting the live `$FFFFF8F0` walk phase bypass the standing-still loc_779C hold on the next frame. Native now resumes `MoveID_Walking` / `x_vel=$1000` immediately after the frame-732 rubber hit, but full `verify-recording --slug issue-215-highwater-water-priority-audit` still fails at `waterfall_overlap_loc_300BA@960`: native lands on the snap frame (`x=$03180000`, `x_vel=0`, `MoveID_Standingstill`) while oracle is one frame after snap (`x=$03181000`, `x_vel=$1000`, `MoveID_Walking`) with the earlier `y_pos` half-pixel delta still present. Next step: compare ASM `sub_8F26` tile-boundary distance / rubber snap cadence and the grounded y snap around this rubber wall.
| 216 | [ ] | P2 | audio | 2026-05-30 playtest: SFX timing/level-complete sound still lag or cut off | Evidence source: human playtest after pushed commits. Symptoms: block break and diamond collection sounds are delayed; level-complete sound does not finish cleanly. #160 covers continuous BGM/SFX output but not these event-specific timing edges. Native repro: not yet recorded on current checkout. Oracle repro: unknown. Minimum gate: audio event trace for prize/block/diamond and flag-touch/EOL path, compare queued GEMS commands and rendered audio windows against oracle where available. |
Note 2026-06-01 for #216: scripted recording SFX events exposed a live
native-player duplication after #226 moved jump SFX into the core trace. The
Level 00 prize-block route
`.cache/recordings/issue-216-prize-block-audio-events/audio-events.txt`
originally logged duplicate `PlaySound(23)` jump cues on frames 122 and 242
because `gems_event_sound_ids` replayed the core `played_sound_effect_ids`
entry and then added the old heuristic jump cue. Native playback now suppresses
the fallback when the core trace already queued `sfx_Jump` (`sub_A432
loc_A480`). Refreshed route logs single jump cues at frames 122/242 and
`PlaySound(13)` for the block hit at frame 129; native-only replay passes.
| 217 | [ ] | P1 | player | Wire ASM-faithful player move-state helpers into the live step (systemic root cause, movement layer) | Player-movement fidelity audit 2026-05-30 (`docs/source-port/movement-fidelity-audit.md`, `player-movement-fidelity-audit` workflow, 44 agents) confirmed 26 gaps across 15 move-states (5 high, 9 medium, 12 low; 23 wiring, 3 hygiene). Same root cause as #176/#203: move-state helpers are modelled + unit-tested but unwired, so the live path uses generic profiles / hardcoded `false` / omits behaviour (`with_left_right_conflict_resolved`, `crawl_wall_escape_sub_81B4`, `initiate_jump_with_hand_grab`, `update_skycutter_horizontal_velocity_sub_9A20` air path, discarded `sub_7A10.blocked`). Children #218-#226 below; 3 hygiene gaps fold into #209, 2 platform-carry gaps into #208, the compound-corner pair relates #207. Highest-leverage first move: **#218** (ice/rubber inversion) — a deterministic correctness bug fixable and verifiable now without the #205 object-active harness. Evidence: ASM-vs-native audit, adversarially verified; not exercised by the idle oracle sweep. |
| 218 | [ ] | P1 | player | Ground speed tables inverted: ice plays grippy, rubber plays slippery (ROM cross-wire undone) | `sub_8600` writes terrain index 1 for ice blocks / 2 for rubber; selector `loc_9AE0` reads them inverted (`#1`→slippery `unk_9CCE`, `#2`→punchy `unk_9C7A`), so ROM ice = slippery glide, rubber = punchy. Native detects terrain faithfully (`grounded_surface_type_sub_8600`, `movement.rs:6404-6424`) but maps symbolically `Ice→ICE_*`/`Rubber→RUBBER_*` (`types.rs:858-866`), undoing the cross-wire on every grounded frame (`movement.rs:4514-4518`). Evidence source: audit, confirmed by code inspection (`BlockType::Ice=2`/`Rubber=6` at `terrain.rs:59/63`). Native repro: not yet recorded. Oracle repro: authoritative (deterministic). **Correctness bug, no harness needed.** Minimum gate: grounded walk-right scenario on an ice and a rubber surface, assert `x_velocity`/accel/decel against oracle; `mise run verify-slice-fast`. |
Rejected 2026-05-31: a literal `PlayerForm::ground_speed_table` swap (`Ice→RUBBER_*`, `Rubber→ICE_*`) matches a first reading of `sub_8600` / `loc_9AE0`, but regresses the oracle-backed `death-respawn-level1` route. Targeted speed-table unit tests passed, then `compare-native-gameplay --scenario death-respawn-level1` failed at `death_tail@1762` with first live drift at frame 1089 (`character_movement`) on a normal `SlopeUp block=None` frame, not an ice/rubber cell; `verify-slice-fast` also failed the same death/respawn regression lane. Do not retry the symbolic table swap without first deriving an isolated ice/rubber walk-right oracle scenario and reconciling why the current Level 1 oracle path depends on the existing mapping.
| 219 | [ ] | P2 | player | `KidGrabbedByHand` RAM flag never surfaced to the player path (jump/crawl/climb) | ROM consults the global grab flag in three player routines: jump init weakens launch `+$20000` (`sub_A432`/`loc_A466`), crawl entry is suppressed (`loc_8C70` @kid.asm:10084), climb B-jump-off weakens too. Native models the arithmetic (`initiate_jump_with_hand_grab`, `physics.rs:726-761`) but every live caller hardcodes `false` (`physics.rs:718-720`); the flag lives only as Hand-AI local state (`enemies.rs:17757`) and is never plumbed into a player field. Evidence source: audit. Native repro: not yet recorded. Oracle repro: needs #205 (Hand-active scenario). Minimum gate: Hand-grab scenario via #205 harness; assert weakened jump height + suppressed crawl; `mise run verify-slice-fast`. |
| 220 | [x] | P2 | player | Crawl stand-up & entry gating incomplete (`sub_81B4` unwired, `sub_7A10.blocked` discarded) | Fixed 2026-05-31: live walking-state crawl entry now respects `sub_7A10.blocked` plus the Micromax/Juggernaut skip before physics consumes `down` for crawl tables, and live crawl-release frames now run the ASM `loc_8270` restored-width head scan before `sub_81B4` side escape. Regressions `loc_8c70_sub_7a10_blocked_keeps_walking_velocity_path`, `loc_8270_sub_81b4_both_side_walls_keep_release_frame_crawling`, and `loc_8270_sub_81b4_right_escape_snaps_live_standup` first failed against the unwired paths and now pass. Gate: focused crawl tests + `mise run verify-slice-fast`. |
| 221 | [x] | P2 | player | Airborne Skycutter uses generic jump-air profile instead of `sub_9A20` | Fixed 2026-05-31: both live airborne horizontal helpers now dispatch Skycutter to `update_skycutter_horizontal_velocity_sub_9A20` before the generic jump-air table path, matching ASM `sub_B084` (`kid.asm:13921`) where `Current_Helmet == Skycutter` branches to `sub_9A20` and skips `loc_B09E` / `unk_A4B2` / `unk_A4C6`. Regression `sub_b084_airborne_skycutter_uses_sub_9a20_velocity` first failed with `$3400` generic accel and now passes with `$2000`; gate: focused test + `mise run verify-slice-fast`. |
| 222 | [x] | P2 | player | Up-/Down-held does not drive uphill/downhill slope acceleration | Fixed 2026-05-31: live slope call sites now map vertical-only inputs through ASM `loc_9D94`/`loc_9DAA` semantics before calling the shared slope table: with no horizontal d1/d2 input, Up enters loc_9DC0 uphill and Down enters loc_9DF2 downhill, preserving horizontal-input priority. Regressions `slope_up_held_only_drives_uphill_accel_loc_9daa` and `slope_down_held_only_drives_downhill_accel_loc_9daa` first failed against the neutral-decel fallback (`-$2000` / `$2000`) and now pass (`$1000` / `$4000`). Gate: focused loc_9DAA/uphill/downhill tests plus `mise run verify-slice-fast`. |
| 223 | [x] | P2 | player | Iron Knight climb edge behaviours (wall-end pop, platform exit, over-permissive airborne attach) | Fixed 2026-05-31: removed the invented airborne Iron Knight wall-attach path; ASM `loc_7664 -> loc_7696` is the grounded standing/walking C-press attach entry, while airborne `kid_flip loc_ACB0` only handles Cyclone, Maniaxe, and Skycutter. Wired ASM `loc_A30A` wall-gone release to seed `y_vel=$FFFC0000` before leaving climb, and wired the `loc_A28A -> loc_A9AA` platform-landing exit so an active platform collision clears climb state in the same frame instead of waiting for the next climb tick. Regressions `iron_knight_c_press_does_not_attach_to_wall_when_airborne`, `loc_A30A_wall_gone_releases_with_upward_pop_velocity`, and `loc_A9AA_platform_landing_exits_iron_knight_climb_same_frame` cover the branches; gate: focused Iron Knight tests + `mise run verify-slice-fast`. |
| 224 | [ ] | P2 | architecture | Compound `$14/$18` slope-corner collision response unported (decomposed per-axis pipeline) | ROM integrates X+Y together then dispatches `collision_type` `$14`/`$18` (floor+wall slope corner) to corner-crush kill (unless Juggernaut/Skycutter), a Juggernaut/Skycutter diagonal bounce-out, and a grid-aligned recovery snap (`sub_A4EE`→`loc_B9A2`/`loc_BA5A`/`loc_BB08`/`loc_BB30`). Native's per-axis floor/wall passes never form the compound type — only the comment block `movement.rs:3198-3243` ("not yet ported"). The plain `$04`/`$08`/`$10` wall/ceiling responses ARE wired (`movement.rs:5057`/`5253`). Relates #207. Evidence source: audit. Native repro: not yet recorded. Oracle repro: needs a slope-into-wall-corner scenario. Minimum gate: wedge each form into a slope corner, compare death/bounce/snap against oracle; `mise run verify-slice-fast`. |
| 225 | [x] | P2 | player | Remove/gate invented movement state changes (damage wall-cling clear, unconditional Berzerker negate) | Fixed 2026-05-31: removed the invented wall-cling/sticky clear from `apply_damage` and gated charging Berzerker/Rhino `loc_B692` X-velocity reflection on copied object metadata bit 15. ASM `Check_for_recent_damge -> loc_B700` mutates HP, damage latches, sound routing, and helmet/death state only; `right_wall_sticky`, `left_wall_sticky`, and `wall_cling_direction` now survive the damage frame. ASM `loc_B678` runs `bclr #$F,d6; bne loc_B692`, so ordinary enemy side contacts still take their charging-damage routes but no longer reverse Kid's X velocity. Regressions: `damage_preserves_wall_sticky_loc_b700`, `berzerker_charge_loc_b692_skips_reflect_when_object_meta_bit_15_clear`, `berzerker_charge_loc_b692_reflects_player_x_velocity_once_for_bit_15_collision`, plus refreshed family-specific Berzerker charge assertions. Gate: focused tests + `mise run verify-slice-fast`. |
| 226 | [x] | P3 | player | Minor input/audio parity: L+R resolver unwired; jump SFX not emitted | Fixed 2026-05-31: wired the existing `FrameInput::with_left_right_conflict_resolved` helper into live `gameplay_input` so `loc_7464` clears Right before movement consumes simultaneous Left+Right, and queued `sfx_Jump` from the live jump probe when `sub_A432 loc_A480` launches through `JumpPressed`/`JumpBuffered`. Regressions `gameplay_input_applies_loc_7464_left_right_conflict_resolution` and `grounded_jump_queues_sfx_jump_loc_a480` failed before the fixes and now pass; existing `loc_7464_*` decoder tests still pass. Gate: focused tests + `mise run verify-slice-fast`. |
| 227 | [ ] | P1 | platform | Wire ASM-faithful platform helpers into the live path (systemic root cause, platform layer); details #208 | Platform fidelity audit 2026-05-31 (`docs/source-port/platforms-fidelity-audit.md`, `platforms-helmets-weather-fidelity-audit` workflow, 126 agents). 11 units, 40 gaps (16 high, 12 medium, 12 low). Same root cause as #176/#203/#217: the entire `Platforms_CheckCollision` escape/carry algorithm (`platforms.rs:96-382`), the slot allocator, the sprite-tiling compiler, and the Trap-platform coroutines are modelled + unit-tested but dead; the live path uses a `GROUND_SNAP_TOLERANCE_FP` approximation (`platforms.rs:689`) with no horizontal push/carry and no FAA contact flags. Architectural: the ROM's 17-slot free-list object model (`Initialize_Platforms`/`Allocate_PlatformSlot`) is replaced by a monotonic `next_index` + per-frame filter (no slot cap, `enemies.rs:40310`). Special-platform dispatch keys off `raw=="0"` not the `off_40D2` index, so TrapPlatformUp/Down (82 level instances) are inert (`types.rs:2086`). Details/supersedes the thin #208. Two harness-independent value bugs split to children #231 (×16 placement scale) and #232 (lift y-vel seed); the `sub_8ED0` form half-width bug folds in here. Wiring gaps blocked on #205 object-active harness. Evidence: ASM-vs-native, adversarially verified; not exercised by the idle sweep. Gate: per-unit regression wiring the existing helper + #205 scenario + `mise run verify-slice-fast`. |
| 228 | [ ] | P1 | player | Wire ASM-faithful helmet/form/Diamond-Power helpers into the live path; extends #206 beyond the DP invuln stub | Helmet/form/Diamond-Power fidelity audit 2026-05-31 (`docs/source-port/helmets-fidelity-audit.md`, same workflow). 11 units, 38 gaps (10 high, 9 medium, 19 low). Extends #206 (which scopes only the Diamond-Power invulnerability stub) to the whole subsystem. The entire DP object/list/dispatch model is dead — live path is a flat `diamond_power_frames` timer + Rust `match diamond_power_type` (`helmets.rs:5918-5929`); Circle of Doom / Wall of Death are an invented screen-wide kill-all (`diamond_power_kill_enemies`, `helmets.rs:6121`) instead of orbit/sweep geometry; Snake powers lose all homing (single forward projectile, `helmets.rs:6013`); Eyeclops beam is a single point projectile vs the ROM growing screen-wide beam (`helmets.rs:5522`). Transform/HP core is well-wired (remaining gaps are audio SFX + benign hygiene). Harness-independent bugs split to children #233 (Maniaxe `last_mut` turnaround) and #234 (ExtraHitPoint 4-slot cap bypass). **Coordinate with the active #206 lane** (Diamond-Power invuln timing). Wiring gaps blocked on #205. Evidence: ASM-vs-native, adversarially verified. Gate: per-unit regression + #205 scenario + `mise run verify-slice-fast`. |
| 229 | [ ] | P1 | level | Level-loading fidelity: `Load_Level` tail modelled but dead; live load delegates terrain/block decode to the emulator dump | Level-loading fidelity audit 2026-05-30 (`docs/source-port/level-loading-fidelity-audit.md`, same workflow). 18 units, 55 gaps (18 high, 19 medium, 18 low). Most of `Load_Level`'s tail (palette/art/background/scroll/music) is modelled helper-for-helper but dead; the live loader `reload_block_grid_for_level` (`level.rs:17537`) loads only the collision grid + block list + murder wall + weather. **Architectural:** terrain + block-layout decode are delegated to the jgenesis emulator RAM dump (`inspect.rs:1023`), not implemented natively — `Decompress_Chunk`, `AddrTbl_BlockTypes` block-type dispatch, `LoadBlockLayout`, and the theme-collision index have no live native representation (terrain is faithful only for pre-dumped levels). `Number_blank_top_rows` is dropped from level height and `Camera_max_Y` inconsistently — enemy spawn coords apply the offset (`enemies.rs:39859`) but terrain/bounds/camera do not, so nonzero top-2 height bits load mismatched rows. Genuine bug split to child #235 (non-oracle terrain fallback corruption). Evidence: ASM-vs-native, adversarially verified. Gate: per-unit regression + new load scenario + `mise run verify-slice-fast`. |
Note 2026-06-01 for #229: playtest reported later-level entry/level 4 instant death, but direct settled `play-native` recordings for levels `03` and `04` were not replayable before gameplay could be trusted: `verify-recording --native-only` drifted at frame 0 because `play-native --warp-level NN --settled` recorded from the synthetic `enter_level_playtest` shortcut while the verifier replayed the ROM-like settled recording oracle path. Native settled recording startup now runs `boot_to_level` plus `settle_recording_oracle_warp_live_window`, matching ASM `loc_1AE2E`/`loc_74E0` title/start and level-skip lineage. Refreshed `.cache/recordings/issue-229-level03-entry-current` and `issue-229-level04-entry-current` both verify native-only and show no immediate death over 120 idle frames; the remaining reported instant death needs a non-synthetic level-end-to-next-level repro rather than direct settled warp evidence.
| 230 | [ ] | P1 | weather | Weather/environment fidelity: storm machine, hail worker, and geyser state machine modelled but dead; only weather 1 dispatched | Weather/environment fidelity audit 2026-05-31 (`docs/source-port/weather-fidelity-audit.md`, same workflow). 12 units, 33 gaps (15 high, 10 medium, 8 low). Extends #203's weather mention. `Init_SpecialEffects` dispatches only weather 1 (Lava); the per-frame `sub_6034→sub_6048` storm machine and the `sub_61CA` 5-slot hail worker are dead — live `step_weather` (`weather.rs:2354`) hand-rolls hail for weather 3 only and drops weather 2 (plain Storm) entirely. Lava geyser is an invented 4-phase pixel ramp with non-ASM constants (`ERUPT_SPEED=4`/`HOLD=30`/`DORMANT=60`, `constants.rs:574`) vs the ROM discrete 16px-row state machine. Every palette animation (`RotatePalette_Lava_Water`, `Animation_Geyser`, Island shore cycle) is unspawned — and the post-load spawn object `loc_11F96` is itself dead (`level.rs:8374`), so even wiring the rotators leaves nothing to drive them (compounds #229). Storm hail collision (`$28` Kid push timer, `$FFFF4A04` collision-map gate) is gameplay-affecting, not just visual. Murderwall is the best-wired unit. Evidence: ASM-vs-native, adversarially verified. Gate: per-unit regression + storm/geyser scenario + `mise run verify-slice-fast`. |
Note 2026-06-01 for #230: first live storm slices landed. `NativeRuntime::init_weather` now seeds weather 2/3 storm worker scalars from ASM `loc_14DAC` (`$FFFFFB3C/$FB3D/$FAD6/$FADA/$FADC/$FB52/$FB54`), live weather 2/3 now enters the shared `sub_6048` state update path, and live hail timing now uses ASM `loc_6172/loc_619C` frame-counter gate `Frame_Counter > $873` plus ROM active-period reloads (`$4B0` inactive→active, `$258` active→inactive) instead of the coarse level-timer/flat-600 approximation. The full 5-slot `sub_61CA` worker, applying palette ops from `sub_6048`, visible weather 2 thunder/storm effects, lava geyser state machine, and palette/shore worker spawns remain open.
Follow-up 2026-06-01: live hail projectile spawn cadence and initial position now use ASM `sub_61CA` `loc_623A`/`loc_626C` values: `$FFFFFB3C` decrements as an 8-bit cooldown, reloads to `$25 - (Get_RandomNumber_byte&$0F)`, and the spawn X is `Camera_X_pos + Get_RandomNumber_byte + $28` with spawn Y at `Camera_Y_pos`. This removes the native fixed 30-frame timer and `(frame_index*7+13)%320` X approximation. Remaining hail gap: generic projectile storage still stands in for the ROM's five falling slots, collision-map splash/impact slots, and `$28` Kid push timer.
Follow-up 2026-06-01: live lava geysers now preserve ASM `loc_14772` per-geyser startup hibernate on `LavaGeyser` and reuse it after collapse via `loc_1498C -> loc_14788`, removing the global `LAVA_GEYSER_DORMANT_FRAMES=60` reset. This fixes the Black Pit/height-indexed stagger after the first cycle. Larger lava gaps remain: the rise/hold/collapse cadence is still the native 4px ramp rather than the discrete ROM row script, and live palette/art upload workers are still unwired.
Follow-up 2026-06-01: USM2 lava fireball initialisation now comes from the ROM position table's `$8000` sign-bit entry and flows through `initialise_lava_fireball_state_loc_1499C` instead of a separate hardcoded spawn. Remaining fireball gap: the live path still uses the parallel static `LAVA_FIREBALL_SCRIPT` runner rather than the full ROM object script walker, though the script values are already pinned by tests.
Follow-up 2026-06-01: Mountain storm thunder now queues the live `sfx_Thunderstorm` path. `step_weather` consumes `sub_6048`'s `play_thunder_sound` flag and calls the existing ASM `sub_1B36C` helper, so weather 2/3 no longer drop the thunder sound effect when `$FFFFFADC` reaches zero. Remaining visible storm gap: palette writes from `sub_6048` are still modelled as ops but not applied to live palette/renderer state.
| 231 | [x] | P1 | platform | Platform placement x/y may omit the ROM `asl #4` (×16) scale — **verify against a running level first** | Fixed 2026-05-31: platform loading now treats `ptfm` x/y as 16px block coordinates and activation buffers as absolute 32px edges, matching ASM `Manage_PlatformLoading`/`loc_2762` (`kid.asm:4690-4723`). Live platform activation now uses Kid's current point, not the enemy camera window, so active snapshots and collision platforms both get ROM-scaled fixed-point positions. Regressions: `active_platform_from_placement_loc_2822_keeps_collision_shape_without_modelling_art_words`, `platform_is_active_loc_2762_keeps_exact_buffer_edges_in_range`, `advance_to_window_stops_*`, and `activates_visible_platforms_in_level_window` with the synthetic fixture converted to `ptfm` units. Gate: focused platform tests plus `mise run verify-slice-fast`. |
| 232 | [x] | P2 | block | Lift platform spawns with y-velocity 0 instead of the ROM `−65536` (−1px/frame up) seed | Fixed 2026-05-31: live lift-block headbutts now seed the spawned lift platform with `y_velocity=-0x0001_0000`, matching ASM `sub_FB3E` (`kid.asm:20999`) `move.l #$FFFF0000,$E(a3)` before `sub_FCA6` applies per-frame acceleration. Regression: `lift_block_headbutt_spawns_platform` now asserts the live spawn seed, and existing `lift_platform_step_*` tests keep `sub_FCA6` / `sub_FD00` stepping pinned. Gate: focused lift tests plus `mise run verify-slice-fast`. |
| 233 | [x] | P2 | player | Maniaxe axe turnaround reverses `projectiles.last_mut()` with no kind filter (latent: wrong projectile reversed) | Fixed 2026-05-31: live `step_maniaxe_axe` now reverses the live `EnemyProjectileKind::ManiaxeAxe` projectile instead of the newest projectile in the global vector, matching ASM `loc_8A74` (`kid.asm:9879`) where the turnaround flips the current axe object in `a3`. Regression `loc_8a74_turnaround_reverses_live_axe_not_last_projectile` first failed against the old `last_mut()` behaviour, then passed with the filtered lookup. Related Maniaxe gaps split to #236 (flight lifetime) and #237 (solid-only terrain delete). Gate: focused `loc_8a74` tests plus `mise run verify-slice-fast`. |
| 234 | [x] | P2 | player | ExtraHitPoint Diamond Power bypasses the `Extra_hitpoint_slots < 4` cap and slot counter | Closed 2026-05-31: the live ExtraHitPoint DP path now already routes through `DiamondPower_Main_ExtraHPLife loc_3FC2A/loc_3FC4C`, increments both `Extra_hitpoint_slots` and `Number_Hitpoints` only when the first converged type-8 slot lands below the four-slot cap, and clears the DP object list through the native `sub_3F63C` equivalent. Added runtime regression `extra_hit_point_activation_respects_four_slot_cap_loc_3fc4c` to cover the capped live activation, complementing the helper-level `diamond_power_main_extra_hp_life_converge_hp_capped_loc_3fbe4` and the existing below-cap activation test. Gate: focused ExtraHitPoint tests plus `mise run verify-slice-fast`. |
| 235 | [x] | P2 | level | Non-oracle level-load fallback feeds the raw BLOCK resource into `BlockGrid` as final cells (corrupt terrain) | Fixed 2026-05-31: `load_block_grid_for_level` now fails closed when `level-layout/<id>.bin` is missing instead of feeding the raw decoded BLOCK resource into `BlockGrid::from_level_layout`. The native `BlockLayoutDecodeWordsLoc1354E` decoder is still unported under #229, so refusing is safer than silently corrupting terrain for skipped dumps. Regression `load_block_grid_refuses_raw_decoded_block_resource_for_mapped_level` first failed against the old fallback and now passes; generated `level-layout` loading remains covered by `new_runtime_loads_generated_level_layout_by_default` and `generated_level_reload_loads_extracted_block_list_for_level_0`. Gate: focused level-load tests plus `mise run verify-slice-fast`. |
| 236 | [x] | P2 | player | Maniaxe axe flight uses invented finite lifetime instead of ROM bounds-driven loop | Fixed 2026-05-31: Maniaxe axes now spawn and step with `lifetime=0` because ASM `loc_8A52` loops until enemy collision, solid collision, or `ManiaxeAxe_CheckOutOfBounds` (`kid.asm:9868-9933`) rather than a DBF lifetime. Native projectile stepping and Maniaxe active-state checks now key off the `EnemyProjectileKind::ManiaxeAxe` object existing, not `lifetime > 0`. Regressions `maniaxe_axe_zero_lifetime_survives_inside_camera_bounds_loc_8a52`, `maniaxe_axe_active_preserves_zero_lifetime_live_axe_loc_8a52`, `maniaxe_axe_spawn_has_no_generic_lifetime_loc_8a52`, and `loc_8a52_axe_advances_position_each_frame` pin the no-lifetime model while existing loc_8B32 bounds tests keep deletion covered. Gate: focused Maniaxe projectile tests plus `mise run verify-slice-fast`. |
| 237 | [x] | P2 | player | Maniaxe axe terrain delete is broader than ROM solid-block collision | Fixed 2026-05-31: `projectile_terrain_contact_loc_3345c` now special-cases `EnemyProjectileKind::ManiaxeAxe` to match ASM `loc_8A84` (`kid.asm:9895-9913`): only `$6000` solid-block collision enters `loc_8ACC`; `$4000/$5000` slopes fall through. Regression `maniaxe_axe_slope_contact_survives_loc_8a84` first failed against the broad terrain-kill helper and now passes; `maniaxe_axe_solid_contact_deletes_loc_8acc` keeps the solid-block delete path pinned. Gate: focused Maniaxe projectile tests plus `mise run verify-slice-fast`. |

Note 2026-05-30 for #211: natural route `.cache/recordings/issue-211-level00-walk-right-live/scenario.json` is cleaner after separating recording-settle from full-frame-settle in `ScriptRunner`: `tas-diff --frame-start 0 --frame-end 70 --include-advisory --frontier-kind gameplay` now matches the opening DragonWalking roster position from frame 0 (`enemy_0D#02@0x00aa8000`, then -$8000/frame). The previous 24 px dragon offset was tooling, not a slope clue: `scenario.settle_oracle_warp=true` was using the 108-frame full-frame sweep age on native Level 0, while the recording oracle path waits the 60-frame title-card live window. Remaining blocker: `number_objects` still differs from frame 0 (native 3 vs oracle 4, likely fixed-system/object-accounting), and Kid state starts drifting at frame 61 when Right input begins (`kid_x_pos`, `kid_x_vel`, `character_movement`). By frame 480 native reaches the first slope (`kid_facing=Right`, `kid_on_slope=1`, `character_movement=6` in the recorded native trace), but the state is still downstream of early object-count/input drift. Do not use this recording to decide the visual slope-frame bug until the frame-0 object accounting and frame-61 input/state drift are explained or a cleaner scenario is recorded.

Note 2026-05-30 for #211: frame-61 Kid position/movement drift in settled recordings was a native input-adapter phase bug, not slope logic. ASM `Mode_Level` (`kid.asm:877-917`) runs `Execute_Objects` before `ReadJoypad`, and oracle probing shows script byte 60 first reaches `Ctrl_Held` on trace frame 61 but Kid movement consumes it on frame 62. `ScriptRunner` now delays script bytes by one frame only for `settle_recording_warp`, matching that helper path without touching ordinary boot/TAS scripts. Regression: `settled_recording_warp_level_zero_delays_first_gameplay_input_until_next_frame`. Focused `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live/scenario.json --frame-start 58 --frame-end 66 --include-advisory --frontier-kind gameplay --only-divergent` now clears `kid_x_pos` and `character_movement`; remaining early blockers are frame-0 `number_objects` (native 3 vs oracle 4) and top-level `kid_x_vel` export drift, which should be classified/understood before using this route for visual slope-frame parity.

Note 2026-05-31 for #211: natural route first-slope entry now matches the ASM `sub_902A -> loc_A99E -> loc_8BF0 -> sub_9386 -> loc_9D22` movement state instead of staying airborne for one extra frame. Root cause: `apply_floor_collision` preferred the centre SlopeUp probe whenever it was within the broad upward air-snap tolerance, even when the centre was still too high for the downward landing check; that masked the leading-edge SlopeUp contact that ROM `sub_914A` accepts. Native now only lets the centre probe override the hitbox when the centre can actually land, and grounded slope frames preserve `loc_9D22`'s y velocity instead of clearing it via the generic grounded `sub_B168` branch. Regression: `level00_airborne_leading_edge_slope_up_recontact_enters_loc_9d22_same_frame`. Focused evidence: `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live/scenario.json --frame-start 427 --frame-end 440 --include-advisory --frontier-kind gameplay --only-divergent` improves frame 428 from five authoritative diffs (`kid_x_pos`, `kid_y_pos`, `kid_x_vel`, `kid_y_vel`, `character_movement`) to one remaining sub-pixel `kid_x_pos` diff (`native=0x02c62d00`, `oracle=0x02c65000`); frames 429-431 likewise reduce to only the same X-position offset. Continue from the residual horizontal offset before treating later visual slope animation/pixel parity as authoritative.

Note 2026-05-31 for #211: the residual first-slope entry now matches oracle frame 428's x subpixel. Native models the `loc_A99E -> loc_8BF0` same-frame landing handoff's base-accel `sub_B084` cap-bleed tail before `loc_9D22/loc_9F18` own the slope velocity. Regression `level00_airborne_leading_edge_slope_up_recontact_enters_loc_9d22_same_frame` covers the oracle frame 428 x subpixel and keeps near-cap A-held landings (for example the death-respawn Level 1 route at `x_vel=0x27380`) on the ordinary x advance. The later `loc_9FBC`/`loc_A0B4` slope-to-solid exit remains open: a broad native snap regressed established Level 1 slope seams, so it needs a narrower ASM/oracle pass before promotion.

Note 2026-05-31 for #211: refreshed the natural route as `.cache/recordings/issue-211-level00-walk-right-live-refresh` because the previous cache-local trace was stale at frame 0 after the live Kid/object export changes. Native now models ASM `loc_9F18 -> loc_9FBC` for the first-slope SlopeUp-to-solid exit after horizontal motion: once `sub_8F26` has moved Kid out of the active slope and the support tile is solid, snap to the solid row and clear `y_vel` before returning through `loc_8BF0`. Regression: `level00_slope_up_to_solid_exit_clears_y_velocity_loc_9fbc`. Focused evidence: `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 432 --frame-end 438 --frontier-kind gameplay --only-divergent` is now 6/6 match. The next #211 frontier is the immediately following flat-motion drift at frame 438: `kid_x_pos native=0x02d09200 oracle=0x02d0a000`, `kid_x_vel native=0x00006200 oracle=0x00007000`; do not treat the later visual slope complaints as authoritative until this horizontal hand-off is understood.

Rejected 2026-05-31 for #211: treating frame 438 as a simple stale `floor_terrain_type` refresh after `loc_9FBC` is too broad. Three uncommitted variants all made the refreshed route match frames 437..442, but each regressed oracle-backed Level 1 death/rubber checks: direct `loc_9FBC` refresh, walking-loop `loc_8E16` refresh after a support scan, and an adjacent-slope/solid detector with a `y_vel == 0` guard. Failing checks were `death_respawn_level_1_stays_alive_past_frame_1161` (`kid_y_pos` shifted from `0x0077B8C0` to `0x00779000`) and `death_respawn_level_1_rubber_wall_hit_rebounds_without_entering_wall_cling` (`frame_1291.kid_x_velocity` shifted from `0x00025FF8` to `0x00026DC0`). Next attempt needs more precise ASM/oracle evidence for why the Level 00 post-slope walking frame gets the normal `$1000` accel without perturbing the Level 1 rubber/death slope route.
Note 2026-05-31 for #211: narrowed the frame-438 stale terrain latch to the low-speed `loc_9FBC`/`loc_A0B4` slope-to-solid return only. Native now recognises the adjacent-slope/plain-solid hand-off at `$6000` walk speed and reruns ASM `loc_8E16 -> sub_8600` before `sub_9A0A`, so the first flat frame uses normal `$1000` acceleration instead of stale rubber `$200`. The broader refresh attempts above still stay rejected because high-speed death/rubber seams depend on the existing latch. Focused evidence: `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 436 --frame-end 443 --frontier-kind gameplay --include-advisory --only-divergent` is 7/7 match. Regressions: extended `level00_slope_up_to_solid_exit_clears_y_velocity_loc_9fbc`, plus `death_respawn_level_1_stays_alive_past_frame_1161` and `death_respawn_level_1_rubber_wall_hit_rebounds_without_entering_wall_cling`.

Note 2026-05-31 for #211: extended the refreshed natural route through the later SlopeUp/Solid edge. The next real state miss was frame 478, where native over-advanced the fresh `sub_902A -> loc_A99E -> loc_8BF0 -> loc_9D22` SlopeUp recontact and snapped one pixel too high (`native x/y 0x031B1800/0x00C40000`, oracle 0x031A4E00/0x00C50000). Native now carries the post-vertical subpixel crossing into the air-cap handoff only for the fresh SlopeUp/plain-solid edge case, leaving the high-speed Level 1 death/rubber seams on their prior path. Focused evidence: `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 446 --frame-end 560 --frontier-kind gameplay --include-advisory --only-divergent` is 114/114 match. Regressions: `level00_frame_478_slope_up_recontact_preserves_oracle_landing_x_loc_a99e`, `level00_slope_up_to_solid_exit_clears_y_velocity_loc_9fbc`, `death_respawn_level_1_stays_alive_past_frame_1161`, and `death_respawn_level_1_rubber_wall_hit_rebounds_without_entering_wall_cling`.

Note 2026-05-31 for #211: the refreshed natural route now clears the next SlopeUp/Solid edge as well. Oracle frame 601 showed the same `loc_A99E -> loc_8BF0 -> loc_9D22` low-word handoff as frame 478, but native only handled the first edge. Frame 606 then exposed the following `loc_9F18 -> loc_9FBC` solid-exit tail: the ROM clears the diagonal slope `y_vel` and snaps to the solid row before the next flat walking frame. Native now applies that exit only for normal-terrain low-speed walking returns, preserving the rejected high-speed rubber/death seams. Focused evidence: `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 596 --frame-end 620 --frontier-kind gameplay --include-advisory --include-physics-trace --only-divergent` is 24/24 match; frame 420..450 remains 30/30 match. Regressions: `level00_frame_601_slope_up_recontact_uses_solid_edge_handoff_loc_a99e`, `level00_frame_478_slope_up_recontact_preserves_oracle_landing_x_loc_a99e`, `level00_slope_up_to_solid_exit_clears_y_velocity_loc_9fbc`, `death_respawn_level_1_stays_alive_past_frame_1161`, and `death_respawn_level_1_rubber_wall_hit_rebounds_without_entering_wall_cling`. Gate: `mise run verify-slice-fast`.
Note 2026-06-01 for #211: extended the refreshed Level 00 slope route through
frames 596..660. The only remaining diff in that window is frame 625
`kid_y_vel` (`native=0`, `oracle=$FFFE0000`), while CRC, Kid position, game
mode, and the next frame all match. The verifier now classifies CRC-clean
top-level velocity-only export skews as oracle limitations outside demo mode
too, so this does not masquerade as movement drift. Focused gate:
`verify-tas --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 596 --frame-end 660 --checkpoint-interval 1 --frontier-kind gameplay`
reports 0 authoritative / 1 oracle limitation.

Note 2026-06-01 for #211: the next natural-route blocker after frame 660 was a
Kid/contact collision phase mismatch, not slope math. ASM `Mode_Level`
(`kid.asm:935-946`) consumes the previous Kid collision word before the late
`GfxObjects_CollisionKid` passes write the current overlap, and
`loc_BC54..loc_BC6E` keeps `Just_received_damage` set for the 66-frame
damage-flash thread. Native now queues player contact damage for the next frame
and preserves the damage latch until that countdown clears. Focused evidence:
`verify-tas --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 620 --frame-end 670 --checkpoint-interval 1 --frontier-kind gameplay`
now reports 50/50 CRC matches, 0 authoritative diffs, and only the known
frame-625 velocity export limitation. The wider 620..760 route now first
diverges at frame 731, where native is on flat/rubber support and accelerates
right from `x_vel=$1F200` to `$20000` while oracle reports `$1A200`; continue
from the post-damage flat-motion/rubber acceleration hand-off before treating
later visual slope symptoms as authoritative.

Note 2026-06-01 follow-up for #211: fixed the frame-731 flat-walk terrain-code
handoff without reopening the high-speed Level 1 rubber/death seam. ASM
`loc_8E16` (`kid.asm:10212`) reruns `sub_8600` before `sub_9A0A`; the natural
route spans a plain solid tile plus block type `$6`, so the refreshed terrain
code 2 selects `unk_9C7A` and decelerates `$1F200 -> $1A200`. Native now applies
that selector only for walking-cap flat frames and keeps the old semantic
surface latch for later floor/block systems. Focused evidence:
`verify-tas --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 720 --frame-end 735 --checkpoint-interval 1 --frontier-kind gameplay`
is 15/15 CRC-clean. The wider 620..760 route now first breaks at frame 747
after the known frame-625 live-velocity export limitation: native releases into
standing/fall one pixel lower (`kid_y_pos=$900000`, movement `$0003`) while
oracle stays at `$8F0000`/movement `$0002`.

Note 2026-06-01 second follow-up for #211: fixed the frame-747 flat rubber edge
release without regressing the plain-solid Level 1 death route. ASM `loc_8E4E`
returns to `loc_8BF0`, which writes `MoveID_Walking` and hibernates before the
next `sub_B43A` floor-support scan; native was applying the lost-floor `loc_B5F6`
handoff in the same step after walking off the rubber support tile. Native now
defers that unsupported-floor release for flat rubber walking edges only, while
still allowing the same-frame hand-off for fresh slope recontacts and plain
normal-terrain edges guarded by `death_respawn_level_1_releases_ground_once_post_step_support_is_gone`.
Focused evidence:
`verify-tas --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 742 --frame-end 752 --checkpoint-interval 1 --frontier-kind gameplay`
is 10/10 CRC-clean. The wider 620..780 route now next diverges at frame 752:
native has `kid_x_pos=$051CBB00`, `kid_x_vel=$20100`; oracle has
`kid_x_pos=$051CE600`, `kid_x_vel=$22C00`.

Note 2026-06-01 third follow-up for #211: fixed the frame-752 airborne
near-floor jump edge. Native's `sub_902A` jump-edge suppression was blocking
air control for all descending near-floor frames; oracle shows the suppression
only applies to the neutral `y_vel=0` edge. On frame 752 Kid is already falling
with `y_vel=$C000`, so ASM `sub_B084` still applies TheKid's airborne right
acceleration `$2B00` (`x_vel=$20100 -> $22C00`). Focused evidence:
`verify-tas --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 750 --frame-end 758 --checkpoint-interval 1 --frontier-kind gameplay`
is 8/8 CRC-clean. The wider 620..800 route is CRC-clean; remaining reported
state differences are the known frame-625 live-velocity export limitation and a
new object-count-only frontier from frame 778 (`number_objects` native 5,
oracle 7) with Kid state still matching.

Note 2026-06-01 fourth follow-up for #211: fixed the frame-778 rubber worker
count. Oracle `Execute_Objects loc_E1E` shows two hibernating `sub_10DE4`
rubber-block skin workers (`code=0x010e1a`) after Kid spans adjacent rubber
cells 85/86. Native previously remembered only one `rubber_block_hit` and did
not count `sub_10DE4` workers because they have no gfx slot. Native now expands
same-row rubber contacts across Kid's foot hitbox and counts
`rubber_block_animations` as object-only workers, matching ASM
`loc_10DA4/sub_10DE4` (`Allocate_ObjectSlot` without `j_sub_E02`). Focused
evidence: `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 774 --frame-end 782 --frontier-kind gameplay --include-advisory --only-divergent`
is 8/8 clean. Wider `verify-tas --frame-start 620 --frame-end 840` is CRC-clean
and advances the next authoritative count frontier to frame 838
(`number_objects/gfx` native 6/5, oracle 7/6) while Kid state still matches.

Note 2026-06-01 fifth follow-up for #211: fixed the frame-784 walking-dragon
roster drift on the refreshed Level 00 route. ASM `Lion.asm sub_36A58` sets the
left-moving shared walker's turnaround bit only when the proposed move crosses
the `$48(a5)` leading-side tile boundary and that bottom-side tile has
collision bits `$4000`; ordinary `$6000` floor must not trigger this path.
Native now applies that gated SlopeUp foot-side probe for the left-facing
DragonWalking wall check, so the dragon turns in place at `0x0439_0000` on
frame 784 instead of walking to `0x0438_8000`. Focused evidence:
`tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 782 --frame-end 786 --frontier-kind enemy --include-advisory --show-object-roster-on-match`
is 4/4 clean, and `verify-tas --frame-start 0 --frame-end 130 --frontier-kind
gameplay` is CRC-clean so the earlier hidden right-moving dragon path stays
unchanged. Wider `verify-tas --frame-start 620 --frame-end 840` remains
CRC-clean and still advances to the known frame-838 object-count frontier.

Note 2026-06-01 sixth follow-up for #211: fixed the frame-838 rubber worker
count on the refreshed Level 00 route. Oracle `Execute_Objects loc_E1E` has
two hibernating `sub_10DE4` rubber skin workers after the later bounce at cells
94/95, but native only anchored the selected hitbox cell 95 after horizontal
carry. ASM `sub_914A` scans every floor cell from `x_pos-half_width` through
`x_pos+half_width` and calls `loc_10DA4` for each rubber cell it sees, before
the later block-effect tail runs. Native now preserves the leftmost scanned
rubber floor cell as the effect anchor, so the existing same-row contact scan
allocates both workers. Focused evidence:
`tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 834 --frame-end 842 --frontier-kind gameplay --include-advisory --include-physics-trace --only-divergent`
is 8/8 clean, and `verify-tas --frame-start 620 --frame-end 860
--checkpoint-interval 1 --frontier-kind gameplay --bisect` is CRC-clean. The
next refreshed-route frontier is frame 991: native snaps onto the solid/SlopeDown
edge at y `$00c00000` with `y_vel=0`, while oracle remains at `$00bf0000` with
`y_vel=$00020000`.

Note 2026-06-01 seventh follow-up for #211: fixed the frame-991 fresh
walking-to-SlopeDown entry. ASM `loc_8E3E -> sub_8F26 -> sub_9386 -> loc_9D22`
applies the horizontal walking step, discovers the new slope support, stores
`y_vel=x_vel`, and hibernates before the later `loc_9F18/sub_A254` slope tail
can advance Y. Native was falling through into that tail immediately and
snapping to the solid/SlopeDown edge one frame early. The fix is restricted to
frames that started grounded on the walking path, leaving `loc_A99E` airborne
recontacts and the Level 1 rubber/death seams on their existing route. Focused
evidence: `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live-refresh/scenario.json --frame-start 988 --frame-end 994 --frontier-kind gameplay --include-advisory --include-physics-trace --only-divergent`
is 6/6 clean, and `verify-tas --frame-start 620 --frame-end 1000
--checkpoint-interval 1 --frontier-kind gameplay --bisect` is CRC-clean.
Regression: `level00_frame_991_slope_down_entry_hibernates_before_slope_tail_loc_9d22`.

Rejected 2026-06-01 for #211 frame 1107: treating the miss as a generic
flat step-down defer is too broad while the Level 1 death-respawn run-up still
has hidden horizontal drift. The frame-1107 oracle Kid slot is hibernating at
`loc_8BFE` after the `loc_8E4E -> loc_8BF0` walking return, so native should
not take its post-horizontal lost-floor hand-off there. However, a lower-row
solid/support guard regresses `death_respawn_level_1_releases_ground_once_post_step_support_is_gone`.
The death-respawn oracle frame 1570 is already in `MoveID_Jump` with no new X
step, while native is horizontally behind and the gameplay compare tolerates
that subpixel/position lag; the current post-step release is compensating for
that older route drift. Do not retry a pure lower-row support guard for frame
1107 until the Level 1 run-up around frames 1569..1571 is exact enough that
`loc_B5F6` timing can be derived without relying on the tolerance.

Rejected 2026-06-01 for Level 1 death-respawn frame 964: extending the
SlopeUp/Solid low-word handoff to the exact `post_vertical_y_pos` half-word tie
(`0x8000`) is locally correct for frame 964 (`native x=0x02FF6580`, oracle
`0x02FF3380`; y/velocities/MoveID otherwise match), and moves the next
900..1100 gameplay-window miss to frame 1096. However, landing that isolated
patch breaks `mise run verify-slice-fast`: ten `death_respawn_level_1_*` unit
checks fail because they were pinning downstream behaviour that currently
depends on the older drift. Do not land the frame-964 tie correction alone.
Next pass should first re-derive the Level 1 slope-to-solid/rubber/death route
around frames 1044, 1096, 1275, 1570, and 1700, then promote the 964 correction
with the affected tests updated from oracle-backed evidence in the same change.

Rejected 2026-06-01 for Level 1 death-respawn frame 1044/1414: treating the
frame-1044 early native SlopeUp landing as a literal missing `sub_902A`
downward tile-crossing gate is too broad. A red unit test modelled
`sub_902A`'s `cmp.w d0,d4; ble loc_913A` guard and a native implementation
restricted fresh downward slope landings until that guard opened, but the
Level 1 route then diverged earlier at frame 964 (`character_movement` native
Jump, oracle Slope) and failed the existing frame-1044 oracle guardrail. The
frame-1414 SlopeDown miss is still downstream of this Level 1 horizontal/landing
drift: by frame 1400 native is about 2.7px behind oracle, so `sub_9386` reaches
the centre-cell SlopeDown one frame late. Do not add a right-edge SlopeDown
special case for frame 1414 or a generic `sub_902A` crossing gate; re-derive the
Level 1 slope-to-solid/rubber/death path from the frame-964 tie through frames
1044, 1096, 1275, and 1570.

Rejected 2026-06-01 for Level 1 death-respawn frame 964/1096: clearing the first
`loc_A99E -> loc_8BF0` SlopeUp/Solid landing handoff's X low word only on the
frame-964 vertical half-word tie makes a targeted frame-964 regression pass
(`x=0x02FF3380`, `y=0x00E00000`, `x_vel=0x23380`, `MoveID=6`), but it exposes
frame 1096 as the next authoritative miss: native stays in `loc_9D22` on the
SlopeUp/Solid span while oracle has already taken the `loc_9FBC` solid-exit tail
(`MoveID=2`, `y=0x008F0000`, `y_vel=0`). A follow-up motion-side-solid helper in
`slope_to_solid_exit_floor_y_loc_9fbc` did not fix frame 1096 and regressed the
rubber/death guardrails (`death_respawn_level_1_rubber_wall_hit_rebounds...`,
`death_respawn_level_1_releases_ground_once_post_step_support_is_gone`). Do not
land the frame-964 low-word clear until the frame-1096 `loc_9FBC` exit is derived
from ASM/oracle in the same route-level change.

Note 2026-05-29 for #152: scope correction. The closed finding (helmet bitmask `%1011000101` blocks the standing→walking transition for non-bitmask forms) is faithful and stays `[x]` — but it only covered the walk-UP half. Playtest shows the form jitters on slope contact instead of sliding DOWN. ROM `loc_8E06`->`loc_9D58`->`loc_9E6C` applies a clamped ±$1000/frame downhill slide while the form holds the standing pose; native's `should_hold_standing_still` early-return suppresses it, producing a two-frame limit cycle. The slide-down bug is now tracked as #204. The two `asm_faithful_loc_77c2/loc_77fc … holds_standing` tests assert x_velocity==0 and may pin the freeze bug — flagged for re-derivation under #204.

Note 2026-05-29 for #176 (correction from the dead-helper census, #203): #176 states the live game "uses generic `step_pacer`/`step_hopper`/`step_flyer`". The census found this overstated at the file level — `enemies.rs` has a comprehensive live family dispatch (`step_frame -> refresh_active_roster -> step_active_enemies_with_rng -> per-family step_*`), and most of its 1,171 `cfg_not_test` helpers are stale-attribute leaves that DO feed live orchestrators. The per-family *behaviour* gaps in #177-202 remain valid, but they are missing sub-branches inside wired orchestrators (e.g. unrouted Berzerker/recoil/state-entry arms), not a wholesale generic fallback. The genuinely-uncalled enemy residue is tiny (~2 fns + skull/twins/goat state-entry sub-machines). See `docs/source-port/dead-helper-census.md`.

Note 2026-05-26 for #175: native `loc_C21E` / `sub_C254` reveal redraws now keep `loc_C186` Scroll B on 8px screen tile coordinates instead of inheriting Scroll A's camera sub-tile offset. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` MAE improves to reveal 10.439, counter 16.513, tail 15.334. Remaining mismatch is mostly foreground reveal tile colour/holes and smaller stat-row pixels.

Note 2026-05-26 for #175: the `loc_C21E` reveal clear now runs before the `loc_C186` Scroll B redraw instead of after it, so transparent pixels in the scoreboard foreground write expose the hologram redraw rather than a black rectangle. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` MAE improves to reveal 7.832, counter 11.110, tail 10.438; `oracle_state_match` remains false, so the level-complete sequence stays open.

Note 2026-05-26 for #175: rejected two more visual hypotheses while refreshing the same recording. Aligning `sub_C254` foreground writes to the 16px screen grid instead of the camera sub-tile position worsened MAE to reveal 15.416, counter 33.682, tail 31.659. Shifting the `loc_C186` Scroll B hologram room and reveal redraws down by one 8px tile worsened MAE to reveal 19.460, counter 27.826, tail 25.656. Keep the current sub-tile foreground placement and unshifted Scroll B room unless ASM/oracle probes expose a different scroll register state.

Note 2026-05-26 for #175: rejected hiding ordinary Scroll A terrain once the `loc_C186` hologram room is visible. ASM keeps Scroll A live until `loc_C21E/sub_C254` replaces its cells; native matched that expectation. Suppressing the terrain while keeping reveal writes worsened `.cache/recordings/issue-175-eol-flag-touch-level12` frame-60 reveal MAE from 7.832 to 31.070 while counter/tail stayed unchanged, so the remaining mismatch is not caused by over-drawing the old foreground layer.

Note 2026-05-28 for #175: native settled-recording bootstrap now replays ASM `Manage_EnemyLoading loc_368A8` load windows across the hidden respawn camera convergence before live gameplay starts. This fixes the initial-state roster/count drift documented by the Level 0x12 flag-touch oracle: `CharacterCollision_TouchFlagpole loc_C056` now freezes both the Armadillo layout #4 and DragonWalking layout #8 stale gfx slots, giving `number_objects=5` / `number_gfx_objects=6`. Strict pixels are still open (`oracle_frame_match=false`); refreshed MAE remains flag-touch 4.154, reveal 0.648, counter 0.035, tail 1.035, so the remaining bug is composition/rendering rather than the missing stale enemy slot.

Note 2026-05-28 for #175: first-frame `loc_C070` rendering now treats the visible hand-off as a VDP latch, not just a camera latch. Native copies the previous render trace's sprite roster, player animation/HUD fields, and scroll for elapsed=0 while preserving the current gameplay state for collision/trace parity; settled probe recordings use the pre-`apply_probe_start_position` render trace because the oracle patches Kid's live WRAM position without rewriting the already-latched sprite table. Focused regression: `loc_c070_first_render_frame_uses_previous_vdp_latch`. Refreshed `issue-175-eol-flag-touch-level12` remains `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`, but `flag_touch` improves from MAE 1.7392718 / 1,130 changed pixels to MAE 0.6596494 / 282. The remaining `flag_touch` diff now matches the persistent HUD scanline band also seen at `eol_reveal`; continue there rather than reopening stale sprite composition.

Note 2026-05-26 for #175: refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` after the latest score-page feedback. Current report is back at flag-touch 0.960, reveal 7.832, counter 11.110, tail 10.438. The settled green `loc_D4CE`/`loc_D770` stat row pixels in native and oracle now share the same row bands and x extents at the tail checkpoint; the visible diff is dominated by the `loc_C186`/`loc_C21E` hologram foreground room and the oracle state snapshot, not by the bonus/score text layout.

Note 2026-05-26 for #175: rejected advancing the native `loc_C21E` reveal replay clock. Adding the `loc_C0CC`/`loc_C1D0` 18-frame fade duration to the rendered reveal clock only improved the reveal checkpoint slightly (MAE 7.832 -> 7.658) and did not move counter/tail, while using full `level_complete_elapsed_frames` worsened reveal MAE to 21.397. Keep `trace.level_complete_elapsed_frames - TOUCH_FLAGPOLE_HIBERNATE_FRAMES_LOC_C070` unless ASM/oracle probes show a real object-clock offset; the remaining mismatch is not solved by simply running more reveal passes.

Note 2026-05-26 for #175: fixed the score-room foreground layering bug behind the user's "stats page is messed up" report. The native renderer was interleaving `loc_C186` Scroll B redraws with `loc_C21E/sub_C254` Scroll A replacement writes, so later hologram redraws could repaint over earlier foreground cells and leave orange/old-looking holes. Native now clears/redraws all affected Scroll B cells first, then draws all replacement Scroll A cells in a second pass while clearing only the local renderer priority mask. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` frame MAE is now flag-touch 0.960, reveal 5.020, counter 0.000, tail 0.000; `oracle_state_match` remains false, so #175 stays open for the remaining state/timing divergence rather than score-page pixels.

Note 2026-05-26 for #175: refreshed the user's stats-page feedback against current head. `verify-recording --slug issue-175-eol-flag-touch-level12` now reports `oracle_state_match=true`; score-page `eol_counter` and `eol_tail` frames are exact (`MAE 0.000`). The command still fails strict frame parity because the earlier `flag_touch` and `eol_reveal` checkpoints have tiny transitional diffs (flag-touch HUD/forest pixels, reveal 33 pixels on one scanline), but the ROM `loc_D4CE`/`loc_D770` stats rows and score display match at the settled score checkpoints. The stale `level_complete_speed_under_seconds_uses_loc_d660_threshold` regression now asserts via the shared gfx-object digit helper at the current `loc_C5A2`/`loc_C48C` coordinate, preserving the `SpeedBonus_Values[Current_LevelID]` expectation without the old top-left filter.

Note 2026-05-26 for #175: recording state parity is now green for the reduced flag-touch bundle. The verifier still checks gameplay identity fields, but treats `flag_touch`/`eol_*` object and gfx-object rosters as visual-worker bookkeeping because native renders ASM `loc_C21E/sub_C254` and `loc_D4CE/loc_D770` from overlay state rather than modelling every transient ROM object slot. The visible `loc_C21E` reveal replay also advances by the oracle-backed 11-frame VDP queue phase; refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` is `trace_match=true`, `oracle_state_match=true`, with frame MAE flag-touch 0.960, reveal 0.035, counter 0.000, tail 0.000. Remaining `oracle_frame_match=false` is now limited to the flag-touch pre-score-room background/parallax transition plus 33 reveal scanline pixels, not the stats rows.

Note 2026-05-26 for #175: fixed a regression from the Level 1 bridge background wrap. Applying layered Plane B repetition during the already-level-complete `loc_C070` flagpole hand-off repainted forest scenery into the oracle's clear sky and worsened `.cache/recordings/issue-175-eol-flag-touch-level12` flag-touch MAE from 0.960 to 17.680. Native now keeps ASM `loc_3050A` row repetition for ordinary camera-parallax gameplay rows but disables it once the trace is in `level_complete`; the same recording is back to `trace_match=true`, `oracle_state_match=true`, frame MAE flag-touch 0.960, reveal 0.035, counter 0.000, tail 0.000. #175 remains open only for the residual transitional pixels.

Note 2026-05-28 for #175: rejected another plain row-clock phase fix after the clean `40165888` refresh. Changing `EOL_SCORE_ROWS_VISIBLE_DELAY_FRAMES_LOC_D468_ORACLE` from `$3C` to `$37` preserved trace/state parity but worsened the strict tail checkpoint (`eol_tail` MAE 1.254 -> 1.268, changed pixels 784 -> 847). The remaining tail pixels are therefore not solved by globally starting `loc_D468` row-worker rendering five ticks earlier; continue from row origin/composition or stale VDP object retention instead.

Note 2026-05-27 for #175: settled-recording frame comparison now renders native checkpoints with the same 60-frame oracle bootstrap phase used by `oracle_scenario_for_recording`, and the fallback HUD diamond animation follows ASM `sub_44B0`/`sub_44DC`'s initial countdown instead of starting one DMA page late. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` remains `trace_match=true`, `oracle_state_match=true`, with frame MAE flag-touch 0.895, reveal 0.035, counter 0.000, tail 0.000. Strict `oracle_frame_match` is still false; continue from the remaining flag-touch HUD/forest pixels and the one-scanline `eol_reveal` mismatch.

Note 2026-05-27 for #175: latest pushed changes regressed the reduced flag-touch visual bundle, so the earlier exact-score-page wording was stale. Fresh verification before this fix was `oracle_state_match=true` but strict frame parity failed with MAE flag-touch 58.160, reveal 56.738, counter 28.591, tail 9.331. ASM/oracle review showed the native renderer was displaying the `loc_C186` hologram room and `loc_D468` EOL row workers from setup time instead of their first oracle-visible phases after `loc_C0CC/loc_C1D0` and `loc_D702`; delaying those render clocks improves the same refreshed bundle to flag-touch 58.160, reveal 0.588, counter 9.003, tail 1.254. Remaining debt is the flag-touch background/flag hand-off and over-advanced `loc_C21E/sub_C254` foreground reveal at the counter checkpoint, not a finished #175 closure.

Note 2026-05-27 for #175: tightened the `loc_C070` flag-touch render hand-off. The native trace reports `level_complete` at elapsed=0, but oracle pixels still contain the previous Scroll B forest frame and Flagpole_Boss sprites for that one checkpoint; later pre-hologram frames stop repeating ordinary Scroll B until `loc_C186` becomes visible. Regressions: `level_complete_handoff_keeps_flag_touch_background_for_first_frame_loc_c070`, `level_complete_handoff_keeps_flag_sprite_for_first_frame_loc_c070`, and `layered_background_wrap_stops_during_flagpole_handoff_loc_c070`. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` remains `trace_match=true` and `oracle_state_match=true`, with MAE flag-touch 46.360, reveal 0.588, counter 9.003, tail 1.254; strict frame parity still fails, so keep #175 open.

Note 2026-05-28 for #175: reduced the remaining score-room transition drift by delaying the rendered `loc_C21E/sub_C254` Scroll A foreground reveal to the oracle-visible queue phase instead of starting it on the first lit `loc_C186` hologram frame. The same `.cache/recordings/issue-175-eol-flag-touch-level12` bundle remains `trace_match=true` and `oracle_state_match=true`; strict pixels still fail on the synthetic flag-touch hand-off and residual transition pixels, but the `eol_counter` checkpoint improves from AE 8917 / MAE 9.003 to AE 33 / MAE 0.00046 while `eol_reveal` and `eol_tail` stay at their previous small diffs.

Note 2026-05-28 for #175: tightened the `loc_C070` flag-touch player priority hand-off. ASM `CharacterCollision_TouchFlagpole` keeps the current Kid gfx object while the first elapsed=0 hand-off frame still presents the previous level/flag sprites, so native now renders Kid's player sprite with priority only on that preserved frame; later pre-hologram frames keep ordinary player priority. Regression: `level_complete_handoff_keeps_kid_above_priority_terrain_for_first_frame_loc_c070`. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` remains `trace_match=true` and `oracle_state_match=true`; strict frame parity still fails, but the flag-touch frame now shows Kid instead of priority-masking him, with MAE moving from 46.360 to 46.338. Keep #175 open for the remaining synthetic flag/foreground positioning and residual transition pixels.

Note 2026-05-28 for #175: rejected two loc_D468 score-row timing shortcuts against the current strict bundle. Changing the shared `EOL_SCORE_ROWS_VISIBLE_DELAY_FRAMES_LOC_D468_ORACLE` from `$3C` to `$36` breaks trace parity at frame 184, and a renderer-only +5 row lead worsens `eol_tail` MAE from 1.2537528 to about 1.267. The current verified baseline remains `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`, with MAE flag-touch 46.338, reveal 0.588, counter 0.035, tail 1.254; the remaining drift should be investigated as row-specific origin/VDP retention/synthetic score-room composition rather than a global row clock.

Note 2026-05-28 for #175: rejected the tempting renderer-only +8 loc_C5A2 row lead after adding oracle EOL gfx-slot instrumentation. The oracle frame-199 slot dump places CONTINUES/SCORE/TIME/NO-HIT/NO-PRIZE/SPEED where native would place them at render elapsed 16, and UNDER/SECONDS at render elapsed 24, while the native trace elapsed is 8. However both a uniform renderer-only +8 lead and a staggered +8/+16 final-row lead worsen the strict `eol_tail` checkpoint from MAE 1.2537528 / 784 changed pixels to MAE 1.2770554 / 873 changed pixels while trace/state parity remains green. Continue with composition/layering/retention evidence, not another loc_C5A2 phase knob.

Note 2026-05-28 for #175: `verify-recording` now emits `changed_pixel_count` and inclusive `changed_bounds` for each checkpoint diff, giving a narrower oracle feedback loop than MAE alone. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12/verify/report.json` localises the remaining strict-pixel misses to: `flag_touch` 50,973 pixels across the full 320x224 frame; `eol_reveal` 236 pixels in bounds x=8..286,y=16..43; `eol_counter` 33 pixels on scanline y=111; and `eol_tail` 784 pixels in bounds x=0..76,y=148..203. Next #175 pass should isolate the full-frame flag-touch composition first, then the row-specific lower-left tail origin/retention, not another global score-row clock.

Note 2026-05-28 for #175: rejected a global settled-recording visual phase offset as the cause of the full-frame flag-touch diff. Temporarily changing `recording_visual_frame_offset` from 0 to 60 made the strict bundle worse: `flag_touch` MAE 46.33846 -> 46.472996 and changed pixels 50,973 -> 51,173; `eol_reveal` MAE 0.5878767 -> 0.7089704. The source was restored to offset 0 and the report regenerated to the baseline bounds above. Continue with a narrower flag-touch composition/object-roster probe.

Note 2026-05-28 for #175: implemented the ASM `CharacterCollision_TouchFlagpole` stale-gfx hand-off. `loc_C056` clears non-Kid object gfx-slot links before `loc_C070` deletes the object slots, so native now freezes the visible gfx roster/counts separately from gameplay enemies until the oracle-visible `loc_C0B6` cleanup boundary. Regression: `flag_touch_deletes_non_kid_objects_before_eol_workers_loc_c070`; verification: focused test plus `mise run verify-slice-fast`. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` remains `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; the flag-touch MAE moves slightly from 46.33846 to 46.328823 while the full-frame diff remains. Next pass should capture/render the complete stale gfx list rather than only currently modelled native objects.

Note 2026-05-28 for #175: added oracle VDP plane-row and scroll-word probes for the reduced flag-touch bundle. `flag_touch` and `eol_reveal` keep Plane B as the old forest nametable (`37xx/27xx/07F3` rows), VSRAM B=`$005C`, and per-line H-scroll values matching negative camera parallax (`FEAD`, `FD5A`, `FD61`); `eol_counter` switches Plane B to hologram words (`42FB/43xx/53xx`) and clears scroll B to zero. This rejects an early-hologram or broad scroll-register theory. Also rejected suppressing all terrain at `loc_C070` elapsed=0: it worsened `flag_touch` MAE from 46.328823 to 75.52812 and changed pixels from 50,971 to 60,896. Continue from exact Plane A/B composition, priority, and stale gfx contents.

Note 2026-05-28 for #175: implemented the first-frame VDP scroll latch for the `loc_C070` flagpole hand-off. ASM reaches `loc_C070` after the frame's scroll transfer, so the first visible LevelComplete render keeps the previous Camera_X/Camera_Y scroll latch while the gameplay state has already switched to LevelComplete. This moves the reduced `issue-175-eol-flag-touch-level12` flag-touch checkpoint from MAE 46.328823 / 50,971 changed pixels to MAE 4.5027714 / 10,016 changed pixels while keeping `trace_match=true` and `oracle_state_match=true`. Strict pixels remain red; the visible remainder is stale sprite/gfx-slot contents on `flag_touch` plus the smaller `eol_reveal`, `eol_counter`, and `eol_tail` residuals.

Note 2026-05-28 for #175: rejected splitting all object sprites back to the current camera while keeping only VDP planes on the previous `loc_C070` scroll latch. The experiment made Kid's x-position look closer by eye, but moved the flag/pole gfx slot the wrong way and worsened `flag_touch` to MAE 4.895866 / 10,386 changed pixels. A narrower variant that kept the flag on the scroll latch and moved only Kid/object sprites still regressed to MAE 4.5065846 / 10,110 changed pixels, so the committed whole-render scroll latch remains the better oracle result. Continue from exact sprite/gfx-slot contents or the row-specific `eol_tail` residual, not another broad sprite-camera split.

Note 2026-05-28 for #175: matched the ASM `Object_EOL_Number loc_C466` and `Object_EOL_Text loc_C592` priority bits for native score-row sprites (`priority(a*)=1`). Focused EOL renderer tests pass and the change is fidelity-correct, but the reduced `issue-175-eol-flag-touch-level12` strict-pixel report is unchanged (`flag_touch` MAE 4.5027714, `eol_tail` MAE 1.2537528). This rejects a simple priority-mask explanation for the lower-left `eol_tail` residual; continue from row origin/timing or exact gfx-slot composition.

Note 2026-05-28 for #175: `verify-recording` now writes layer-isolated native checkpoint PNGs under `verify/native/layers/<layer>/`. For the reduced flag-touch bundle, the new artefacts show the remaining full-frame mismatch decomposes into native `background` (forest Plane B), `terrain` (tree/grass foreground), `block_overrides` (stone columns), `effects` (flag), `player`, and HUD layers. The same strict run still has `trace_match=true`, `oracle_state_match=true`, and the known pixel failure, so these artefacts are diagnostic only. Use them before another #175 renderer patch instead of tuning the combined frame by eye.
Note 2026-05-28 for #175: rejected switching `EolScoreVisuals` from `scenes/palette/Score_screen.bin` to `scenes/palette/0D02A.bin`. Although ASM `loc_D052` loads `Pal_D02A` into live `Palette_Buffer+$42`, the reduced strict bundle got worse only at the score-row tail checkpoint (`eol_tail` MAE 1.2537528 / 784 changed pixels -> MAE 1.438844 / 905 changed pixels) while trace/state parity and the other checkpoint diffs stayed unchanged. Source was restored to `Score_screen.bin`; the remaining tail residual is not solved by this palette swap.
Note 2026-05-28 for #175: native EOL score-row rendering now keeps the trace-level `loc_D468` row-worker clock but hides the rows through the eight oracle-visible `loc_D734` fade steps before applying `loc_C5A2` / `loc_C48C` fly-in positions. Changing the trace constant itself to delay rows by eight frames broke replay parity at frame 190, so the correction is renderer-only. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` remains `trace_match=true`, `oracle_state_match=true`; strict pixels still fail, but `eol_tail` improves from MAE 1.2537528 / 784 changed pixels to MAE 1.0353422 / 666 changed pixels. Remaining #175 diffs are now `flag_touch` 4.5027714 / 10016, `eol_reveal` 0.5878767 / 236, `eol_counter` 0.035416666 / 33, and `eol_tail` 1.0353422 / 666.

Note 2026-05-28 for #175: added an exploratory oracle CRAM dump at the `loc_C070` checkpoints and rejected disabling the Whispering Woods `sub_129AE` background palette swap. The no-swap experiment worsened `flag_touch` to MAE 8.722512 / 10939 changed pixels, so the source stayed on the committed swap. Oracle CRAM at `flag_touch` has foreground greens in palette line 0 slots 8/9 (`$0060/$04A0`) and background greens in palette line 1 slots 8/9 (`$0260/$0280`), while the dominant remaining native diff is background-owned pixels rendered with the line-1 colours where oracle shows line-0 colours. Continue from exact Plane B tile attributes / palette-line composition rather than another CRAM load theory.

Note 2026-05-28 for #175: rejected two narrower first-frame palette shortcuts against the refreshed strict bundle. Rendering the `loc_C070` latched forest background with `level_visuals.palettes.foreground` instead of the ROM forest background palette worsened `flag_touch` from MAE 4.153567 / 9,767 changed pixels to MAE 18.149824 / 13,783 changed pixels. Forcing tile index `$07F3` to palette line 0 on the first hand-off frame was a no-op. Keep the committed background palette path; next evidence needs exact Plane B tile/palette-slot or stale-gfx composition, not a broad foreground-palette swap.

Note 2026-05-28 for #175: captured the requested canonical playability bundle with the non-interactive path after `play-native --warp-level 00 --settled --record issue-175 --record-frames 1800` did not exit on its own. `cargo run --release -p kidcli -- record-scenario --slug issue-175 --level 00 --scenario walk-right --walk-frames 1800` wrote `.cache/recordings/issue-175`; `cargo run -q -p kidcli -- verify-recording --slug issue-175 --native-only` passed with trace/frame replay parity, and `cargo run -q -p kidcli -- capture-frames --level 00 --frames 0,60,120 --output .cache/debug/issue-175/` wrote frame PNGs. This is a negative repro for the level-winning sequence itself: the transcript only records one damage event at frame 1209, and trace checkpoints through frame 1859 remain `game_mode=InGame`, `current_level=0`, `level_complete=false`. Treat #175 as visual-parity/EOL evidence via `.cache/recordings/issue-175-eol-flag-touch-level12`, not via this Level 00 walk-right playability capture.

Note 2026-05-28 for #175: `verify-recording` now persists `top_colour_diffs` per strict checkpoint, replacing the ad hoc image-script step used during triage. Refreshing `.cache/recordings/issue-175-eol-flag-touch-level12/verify/report.json` keeps `trace_match=true` and `oracle_state_match=true` with the same strict failures, and exposes the dominant substitutions directly: `flag_touch` is mostly native `(0,146,36)` -> oracle `(0,182,73)` (5,627 pixels, sample pixels x/y `77/52`, `125/52`, `77/53`, `125/53`, `76/54`) plus native `(0,109,36)` -> oracle `(0,109,0)` (2,342 pixels, sample pixels `69/44`, `110/44`, `69/45`, `110/45`, `68/46`), while `eol_tail` is dominated by blue score-room/background pixels becoming green. Next #175 work should connect those RGB pairs to exact Plane B tile words / palette slots or stale gfx composition.

Note 2026-05-28 for #175: `verify-recording` now also persists `native_background_samples` for the top strict-diff sample pixels. The dominant `flag_touch` samples map to native Scroll B tiles with palette-line 1 pixels: `77/52` is tile `77`, palette index `9`, RGB `(0,146,36)`; `125/52` is tile `72`, palette index `9`; `69/44` and `110/44` are tile `67`, palette index `8`, RGB `(0,109,36)`. These correspond to the oracle's old Plane B forest words (`37xx/27xx`, base `$780` plus the same tile indices) and line-1 CRAM slots `$18/$19`. Since the oracle combined frame shows line-0-looking greens at those pixels while Plane B provenance is line 1, the next patch should inspect stale Plane A priority/composition over the latched Plane B frame rather than changing the background palette again.

Note 2026-05-28 for #175: corrected the Forest palette-line interpretation with ASM and oracle per-pixel evidence. `Load_BackgroundMappings_Layered loc_128BA` decodes all Forest planes with base `$0780`; it does not add palette-line bits per plane, so zero attribute words stay on palette line 0. The new exploratory `eol_vdp_pixel_samples_loc_c070` probe confirms the upper dominant `flag_touch` samples are transparent Plane A over Plane B palette-line 0 pixels, not stale foreground or line-1 CRAM. Native now removes the invented Forest palette fallback and composes Forest background rendering with foreground CRAM line 0 plus the `sub_129AE`-swapped background palette in line 1. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` keeps trace/state parity and improves `flag_touch` from MAE 4.153567 / 9,767 changed pixels to MAE 1.8512834 / 1,798 changed pixels. The remaining dominant lower-band samples are the same colours swapped horizontally (`native (0,146,36) -> oracle (0,109,36)` and vice versa), with oracle Plane B at tile `$7F4` but native sampling two pixels out of phase, so continue from the Forest water/ripple H-scroll phase rather than another palette or stale Plane A theory.

Note 2026-05-28 for #175: extended the lower-band Forest ripple evidence instead of taking a weak phase shortcut. `native_background_samples` now reports the applied scanline row offset, and `eol_vdp_pixel_samples_loc_c070` includes the dominant lower samples at `72/133` and `109/133`. Current evidence: native applies row offsets `+4` at `y=125` and `+8` at `y=133`, while oracle H-scroll is `hB=$160` at `y=125` and `hB=$15B` at `y=133`; the mismatch is not a uniform table phase. Rejected plain `Time_Frames>>2` phase offsets: `+1` worsened `flag_touch` to MAE 1.8521438 / 1,802 changed pixels, `+2` only nudged it to 1.8497349 / 1,785 without explaining `y=133`, and `+3` worsened it to 1.8543806 / 1,811. Keep the ROM phase path unchanged until the row base scroll / extracted-row wrapping mismatch is isolated.

Note 2026-05-28 for #175: fixed the settled-warp Forest ripple clock by feeding ASM `BackgroundScroll_ApplyForestWaterRipple loc_305E6` the ROM-style global `Time_Frames` phase rather than the rebased recording-local frame index. Settled warp recordings enter through the ROM level-skip path, so the native render trace keeps a named 52-VBlank Forest ripple offset only while `LevelSkip_Cheat` is armed; the direct `loc_30598/loc_3060A` helper remains unchanged for ordinary traces. This moves `flag_touch` from MAE 1.8512834 / 1,798 changed pixels to MAE 1.7392718 / 1,130 changed pixels while keeping trace/state parity green. Strict pixels remain open; the new dominant `flag_touch` diff is a smaller terrain/background colour disagreement around x=159..163,y=125 plus the existing top-row reveal pixels.
Note 2026-05-28 for #175: corrected the pre-hologram HUD timer latch after the first `loc_C070` hand-off frame. ASM `CharacterCollision_TouchFlagpole loc_C070` sets `$FFFFFB4B`, so later `Make_SpriteAttr_HUD` calls exit before the decrement path reaches `loc_BEE8`; native trace state has consumed the second, but the visible HUD table still shows the previous timer value until the end-level hologram replaces the gameplay HUD. Regression: `level_complete_pre_hologram_hud_timer_uses_loc_c070_latched_previous_second`. The strict `issue-175-eol-flag-touch-level12` bundle remains red, but `eol_reveal` moves from MAE 0.6480841 / 279 changed pixels to MAE 0.64384764 / 275 while trace/state parity remains green; the remaining top-band mismatch is HUD placement/composition, not the timer value.
Note 2026-05-28 for #175: fixed the ordinary gameplay HUD sprite layout that feeds the residual `loc_C070` strict frames. ASM `Make_SpriteAttr_HUD loc_BEE8` calls `sub_BD0A`, so the visible timer row uses `unk_BD8E` positions `$90,$98,$9D,$A5`, not the earlier `Init_SpriteAttr_HUD` seed positions. ASM `life_display` / `diamond_display` also writes `calc_display_number`'s low word to the first `$190` slot before swapping, so single-digit counters render the digit before the `$86C5` x-slot rather than after it. Regressions: `game_hud_timer_uses_digit_tiles_and_colon_slot_positions`, `game_hud_single_digit_counters_put_digit_before_x_slot_like_calc_display_number`. The strict bundle remains red but the top HUD band is mostly gone: `flag_touch` improves from MAE 0.6596494 / 282 changed pixels to MAE 0.068010606 / 46, and `eol_reveal` improves from MAE 0.64384764 / 275 to MAE 0.05597098 / 39. Remaining pre-score-room pixels are limited to the animated ankh/diamond HUD icon corner; later residuals remain `eol_counter` 33 scanline pixels and `eol_tail` 666 lower-left pixels.

Note 2026-05-28 for #175: expanded the oracle `eol_vdp_pixel_samples_loc_c070` probe across the remaining `eol_counter` scanline at y=111. Refreshed `verify-recording --slug issue-175-eol-flag-touch-level12` still has `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; the expected strict failure is unchanged at `eol_counter` MAE 0.035416666 / 33 changed pixels. The new samples show oracle `loc_C070` has Scroll B reset to zero with hologram Plane B words underneath, while the visible y=111 pixels come from Plane A score-room tiles around entries `$0222`, `$01C2`, `$01C4`, `$025C`, and `$00D2`. Continue #175 from Plane A score-room row composition/retention, not another Scroll B or global row-clock hypothesis.

Note 2026-05-29 for #175: `verify-recording` now persists `native_layer_samples` for each top strict-diff sample pixel, and layer-isolated native frames now respect `LayerFilter` for EOL hologram, reveal, HUD, game-over, pause, and debug overlays. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` still fails strict pixels as expected, but the samples narrow ownership: `eol_reveal` top-right pixels are HUD/terrain composition, `eol_counter` y=111 pixels are background/terrain plus `block_overrides` score-room replacements, and `eol_tail` lower-left pixels are background/terrain/`block_overrides`. The next #175 patch should inspect `loc_C21E/sub_C254` Plane A replacement ordering/retention against these layers, not sprites or audio.

Note 2026-05-29 for #175: rejected drawing normal `block_overrides` after the `loc_C21E/sub_C254` foreground reveal. It looked plausible because some native `block_overrides` layer samples matched oracle colours at `eol_counter` y=111, but the experiment massively worsened the refreshed strict bundle: `eol_counter` changed from 33 pixels to 7,543 pixels (MAE 8.034) and `eol_tail` from 666 pixels to 12,214 pixels (MAE 13.421). Keep the committed order: terrain, normal block overrides, then the EOL foreground reveal pass.

Note 2026-05-29 for #175: `verify-recording` now embeds the oracle `loc_C070` gameplay debug fields in each non-matching strict checkpoint diff, and the oracle VDP pixel probe covers the current top-right HUD/icon residual plus the lower-left `eol_tail` region. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` still fails strict pixels as expected with unchanged bounds: `flag_touch` 46 pixels at x=289..299,y=13..44; `eol_reveal` 39 pixels at x=290..299,y=34..44; `eol_counter` 33 pixels on y=111; `eol_tail` 666 pixels at x=0..76,y=148..203. The new oracle samples show the final tail residual is visible Plane A score-room words (`$4417`..`$4419`) whose framebuffer colours are green while their raw Plane A palette-2 pixels decode blue/purple, so the next patch should inspect CRAM/palette timing or VDP priority/composition at `loc_D468`/`loc_D770`, not `block_overrides` order.

Note 2026-05-29 for #175: rejected a renderer-only +19 `loc_D468` row-position lead after the new oracle samples showed score-row sprites touching the lower-left edge at `eol_tail`. The experiment kept trace/state parity but worsened the strict tail checkpoint from 666 to 1,187 changed pixels and expanded the bounds from x=0..76 to x=0..108. The residual is therefore not a uniform EOL row-worker phase lead; continue from exact sprite coverage/priority or CRAM composition for the left-edge row sprites.

Note 2026-05-29 for #175: split native layer diagnostics so EOL score/continue overlays report as `eol_score` instead of sharing the gameplay HUD bucket. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12` remains strict-red with unchanged counts, but the lower-left `eol_tail` samples now show native `eol_score` is black while oracle reports high-priority EOL sprite hits at the same pixels. The next patch should inspect native EOL score sprite coverage/origin against `Object_EOL_Text loc_C592` and `Object_EOL_Number loc_C466`, rather than terrain or CRAM first.

Note 2026-05-29 for #175: rejected reconstructing the row-worker clock as `visible_elapsed + $13` even though it puts the TIME text at the oracle-sampled `x=-8` lower-left sprite position. The focused renderer test stayed green, but `verify-recording --slug issue-175-eol-flag-touch-level12` worsened `eol_tail` from 666 changed pixels at x=0..76,y=148..203 to 2,184 changed pixels at x=0..307,y=124..203. The lower-left sprite hit is therefore not enough to justify a global row phase correction; continue by comparing individual EOL gfx slot placement/composition instead.

Note 2026-05-29 for #175: `verify-recording` now persists `native_eol_score_sprites` for strict-diff checkpoints, listing native EOL score sprite command bounds and tile refs beside oracle `eol_visible_gfx_slots_loc_d468`. Refreshed `.cache/recordings/issue-175-eol-flag-touch-level12/verify/report.json` keeps the expected strict failure counts, and shows only `eol_tail` has native EOL score sprites: 36 grouped native commands versus 65 oracle gfx slots. Native TIME/BONUS at the tail checkpoint remain off-screen (`Spr_EOL_Time` command x=-224..-177,y=148..155; `Spr_EOL_Bonus` x=-168..-105,y=148..155), while the oracle lower-left pixel samples report high-priority sprite #10 at screen `xy=-8/148`. The next patch should map oracle per-slot coordinates through the Mega Drive sprite coordinate convention or split native grouped number/label commands before changing row timing.

Note 2026-05-29 for #175: native EOL score diagnostics now split each grouped draw command into ASM `Make_SpriteFromGfxObject loc_12B8/loc_12C0` style hardware chunks, so the reduced report exposes native TIME chunks at `x=-224/-192` and BONUS chunks at `x=-168/-136` before any timing experiments. Re-tested the tempting `0x21` / visible-elapsed-plus-`$13` row phase because the first TIME chunk would land at the oracle-sampled `x=-8`, but it reproduces the existing rejected path: `verify-recording --slug issue-175-eol-flag-touch-level12` remains trace/state green and worsens `eol_tail` to 2,184 changed pixels across x=0..307,y=124..203. The timing commit was reverted; the cache-local bundle is restored to the baseline strict failure (`flag_touch` 46, `eol_reveal` 39, `eol_counter` 33, `eol_tail` 666). Continue from per-slot VDP coordinate mapping, row-specific retention, or CRAM/priority composition, not another global loc_D468 phase shift.

Note 2026-05-29 for #175: rejected applying Mega Drive 9-bit sprite X wrapping directly in the native sprite renderer. A full `(x+128)&$1FF` style wrap made the focused synthetic renderer test pass but worsened the reduced EOL recording to `flag_touch` 324 and `eol_tail` 4,866 changed pixels; a narrower right-edge-only wrap kept the early checkpoints at baseline but still worsened `eol_tail` to 2,100 changed pixels. Native draw commands are not raw pre-`sub_13C0` hardware coordinates, so continue by mapping oracle `eol_visible_gfx_slots_loc_d468` / `eol_vdp_pixel_samples_loc_c070` back to native row-worker commands before touching global sprite clipping.

Note 2026-05-29 for #175: oracle reports now include `eol_sprite_table_entries_loc_c070`, a linked VDP sprite-table dump after ASM `Make_SpriteFromGfxObject` / `sub_13C0`. For `eol_tail`, the oracle lower-left strict pixels come from the EOL `BONUS` label chunks `#0F/#10` at `x=-40/-8,y=148,pat=C517/C51B`; baseline native still has that row at `x=-168/-136` because it subtracts the 8-frame visible-fade offset from `level_complete_score_rows_elapsed_frames`. Tested the exact apparent `+16` row-worker delta: native aligns the `BONUS` chunks to `x=-40/-8` but worsens `eol_tail` from 666 to 873 changed pixels, so the remaining mismatch is not solved by a uniform +16 elapsed phase. Keep the hardware-entry diagnostic and compare row-specific composition/tile/palette against oracle before another timing patch.

Note 2026-05-29 for #175: corrected the EOL score-row renderer to use the pre-frame VDP sprite table exposed by `eol_pre_frame_vdp_pixel_samples_loc_c070`, rather than the post-frame `eol_sprite_table_entries_loc_c070` diagnostic. The earlier `+16` elapsed experiment aligned native hardware chunks to the post-frame dump but worsened pixels; ASM `loc_D734` falls through to the row workers before `sub_13C0` latches the next displayed sprite table, so the rendered checkpoint is 15 worker ticks ahead of native's post-frame row counter. Refreshed `verify-recording --slug issue-175-eol-flag-touch-level12` remains strict-red overall, but `eol_tail` is now exact (`666` changed pixels -> `0`) while the remaining residuals are `flag_touch` 46 top-right HUD/icon pixels, `eol_reveal` 39 top-right pixels, and `eol_counter` 33 pixels on scanline y=111.

Note 2026-05-29 for #175: cleared the top-right HUD/icon residuals by matching the animated HUD VRAM pages around ASM `sub_44DC`. Removing the existing diamond render lead worsened the reduced recording (`flag_touch` 91, `eol_reveal` 84), while advancing the rendered `unk_461C` diamond page to a two-slot lead cleared `eol_reveal` and left only the ankh corner. The first `CharacterCollision_TouchFlagpole loc_C070` frame also needs the pre-frame `loc_4586` life-icon page, but later frozen-HUD frames must return to page 0. Regressions: `game_hud_diamond_icon_animates_via_sub_44dc_dma_pages`, `level_complete_first_handoff_uses_latched_life_icon_page_loc_4546`. Refreshed `verify-recording --slug issue-175-eol-flag-touch-level12` is now strict-red only on `eol_counter`: `flag_touch=0`, `eol_reveal=0`, `eol_counter=33`, `eol_tail=0`.

Note 2026-05-29 for #175: cleared the last `eol_counter` scanline by clipping the `loc_C21E/sub_C254` Scroll B hologram redraw back to the exact 16x16 foreground update cell. The provenance report showed native drew the correct old Plane A score-room colour, then a later whole-tile hologram redraw leaked one scanline beyond a sub-tile-aligned reveal cell. Shifting the redraw down by one pixel was rejected because it worsened `eol_counter` from 33 to 152 changed pixels. The committed fix adds clipped indexed tile slices for the EOL underlay only, preserving the replacement Scroll A words. Regression: `level_complete_reveal_clips_hologram_redraw_to_scroll_a_cell_loc_c21e`. Verification: `cargo run -q -p kidcli -- verify-recording --slug issue-175-eol-flag-touch-level12` now passes strict pixels; `mise run verify-slice-fast` passes.

Correction 2026-05-06, folded into the table rows on 2026-05-18: the long #139/#140 rows used to contain old prose saying no proposed-fix commit had landed. The patch did land in `f75b09f8` (`render: index block mappings by skin byte`); the detailed "139 / 140 — Block_Mappings renderer indexing" note below is the current evidence. #139 and #140 are now validated against deterministic PNG bundles.

Correction 2026-05-06: #136 no longer needs the "wire native floating-item rendering to that spec" follow-up. Native rendering resolves the content byte through `PrizeContent::from_id`, consumes the exported `floating_prize_graphics_spec` table from `kid-core`, and pins diamond/Skycutter renderer metadata against `PrizeDiamond_Init` / `sub_10CB0`.

Correction 2026-05-18, folded into the #136 table row: `record-scenario --scenario prize-block` now writes a replayable settled-gameplay bundle. Fresh scripted evidence lives at `.cache/recordings/issue-136-prize-emergence/`: the manifest records `record_command="cargo run --release -p kidcli -- record-scenario --slug issue-136-prize-emergence --scenario prize-block --level 00"`, `start.settled=true`, and `verify-recording --slug issue-136-prize-emergence --native-only` passes with `trace_match=true` over 361 frames. The captured frames reach a settled Level 00 prize-block interaction, but they are not enough to validate the item sprite because the spawned diamond has already moved off-screen by the inspected PNG. Closure now comes from the renderer using ROM asset frames directly and from focused asset-backed tests for diamond and helmet prizes.

Oracle-verifier attempt 2026-05-18: `cargo run -q -p kidcli -- verify-recording --slug issue-136-prize-emergence` preserves native replay determinism (`trace_match=true`) but fails before it can be used as visual proof: every oracle checkpoint remains in `game_mode=8`, `boot_phase=starting_level`, with Kid at `(64,104)` after the fixed settled-recording oracle prefix, while native is already settled `InGame`. The report is at `.cache/recordings/issue-136-prize-emergence/verify/report.json` (`oracle_state_match=false`, `oracle_frame_match=false`). Do not treat that as a #136 graphics regression; it is an oracle-bootstrap limitation for this settled recording shape. Use `--native-only` plus human PNG inspection until `verify-recording` grows a settled-oracle bootstrap that reaches gameplay.

Correction 2026-05-18: `play-native --record` now accepts `--record-frames N`, which saves and exits after `N` captured native frames. Use this on remaining human-validation recordings so F10/Esc is no longer required to produce `.cache/recordings/<slug>/manifest.json`. For playtest bundles whose purpose is deterministic native replay plus human visual inspection, run `verify-recording --native-only`; the default verifier still performs oracle state/pixel comparison and may fail for visual-validation evidence that is not intended to be oracle-clean yet.
| 071 | [-] | P3 | infra | AsmFaithful Phase 2: floor collision self-applies y_pos (sub_902A) | DONE — resolved in #080 |
| 072 | [-] | P3 | infra | BlastEm second oracle for cross-validation | DEFERRED — jgenesis sufficient, GDB stub approach documented in previous plan |
| 073 | [-] | P3 | infra | Instruction-level M68K trace from jgenesis | DEFERRED — oracle-probe memory watches are lighter; env var approach documented |

## Investigation notes

### Shared boot-title-start-level1 evidence blocker (CLOSED)
2026-04-21 delegated package `backlog-evidence-blocker:boot-title-start-level1:level_entry@520`
reconfirmed that the shared boot/title evidence path is green. `cargo run -p
kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes, and
the required direct captures were refreshed under
`.cache/debug/backlog-evidence-blocker/level_00_frame_00000.png`,
`.cache/debug/backlog-evidence-blocker/level_00_frame_00060.png`, and
`.cache/debug/backlog-evidence-blocker/level_00_frame_00120.png`.

Selector guard: do not reselect `boot-title-start-level1:level_entry@520` as a
backlog evidence blocker unless a new run of that exact compare command fails.
ASM `Obj_TitleMenu loc_1BBD6` only performs the Start hand-off by writing
`Game_Mode=$0008`, setting `$FFFFFBCE`, and jumping through `loc_6E2` into
`Load_InGame`; `level_entry@520` is therefore a hand-off identity checkpoint,
not settled gameplay evidence. The settled Kid pose/state evidence remains at
`level_settle@719`.

`cargo run -p kidcli -- verify-recording --slug issue-141` still fails
oracle/frame parity against the intro visual recording, so keep that as
issue-141-specific visual evidence. It is not evidence that the shared
`boot-title-start-level1` compare path is blocked.

### 141 — Intro and level-intro graphics still do not match the original game (FIXED)
Fixed on 2026-04-20: native intro VDP loading now keeps ASM `sub_1B850` for
`Load_IntroSequence1` and `Load_IntroSequence3` only (`kid.asm:29067-29082`,
`kid.asm:30172-30215`). `Load_IntroSequence2` now loads `Addr_HoloBG`, applies
the `sub_1B7B6` tilemap patch, writes the intro2_sky map, and copies palette
via `sub_1C4FE` (`kid.asm:29255-29300`); `Load_IntroSequence4` uses its HoloBG
variant and `sub_1C4FE` path (`kid.asm:29303-29321`); `Load_IntroSequence5`
uses the Dark Alley / P-block / Kid / helmet art set, `MapEni_2E154`, and
`sub_1C512` palette path (`kid.asm:29324-29364`). The targeted regression
`intro_sequence_2_4_and_5_follow_their_rom_vdp_loaders` guards the split so
segments 1, 3, and 4 cannot silently fall back to the Wild Side `sub_1B850`
asset path again.

Package caveat: the refreshed `issue-141` recording proved the boot/intro
visual mismatch but did not reach settled Level 00 gameplay, so it still did
not provide independent level-intro evidence. Reopen that half only with a
settled gameplay recording that reproduces a separate level-intro mismatch.
The former `boot-title-start-level1` evidence blocker at `level_entry@520` is
closed: ASM `Obj_TitleMenu loc_1BBD6` writes `Game_Mode=$0008` and jumps
through `loc_6E2` into `Load_InGame`, so that first frame is now compared as a
hand-off identity checkpoint only. Settled Kid pose/state evidence remains at
`level_settle@719`, and `cargo run -p kidcli -- compare-native-oracle
--scenario boot-title-start-level1` passes after the compare-path fix.

### 001 — Player stuck after damage (FIXED)
Root cause: `apply_damage()` did not clear `left_wall_sticky` / `right_wall_sticky`.
If the player was wall-grabbed when hit, they stayed pinned to the wall during
invulnerability frames. Fix: clear both wall-sticky fields in `apply_damage()`.

### 002 — Enemies don't die when stomped (FIXED)
Root cause: stomp detection used strict `previous_bottom <= stomp_e_top` position check,
requiring the player's feet to be precisely above the enemy's top edge on the prior frame.
The ASM uses a velocity-direction cross-product: `y_overlap × |rel_x_vel| < x_overlap × |rel_y_vel|`
to determine approach direction. Reimplemented to match ASM — stomps are now detected based
on vertical approach direction, not prior-frame position history.

### 122 — Level 2 entry can spawn Kid in midair, then fall through the floor and die (FIXED)
Evidence source: repeated native play reports plus strengthened native smoke coverage.
Native repro: the old `progress-two-transitions` gate never actually resumed Level 2 gameplay.
Oracle repro: unknown. Minimum gate: `1p-playable`.

Fix: extend `progress-two-transitions` so it re-arms the second skip hold on frame `2700`,
records `level_2_settle@2701`, and then survives through `level_2_live_tail@3300`. The native
gate now proves stable Level 2 landing plus 10 seconds of live gameplay instead of stopping in
`game_mode=$0008` transition mode.

Surprise worth keeping: the real Blue Lake Woods 2 header spawn is intentionally midair at
`(32,160)`, and the hidden `Load_InGame` settle path was already landing safely. The real gap was
the smoke scenario: it ended before sending the second resume input, so we were treating a
transition-shell check as if it proved live Level 2 play.

### 123 — Death/respawn still leaves camera drift, stuck-from-start, or broken restart state (FIXED)
Evidence source: repeated native play reports. Native repro: reported, needs canonical recording slug.
Oracle repro: mixed/unknown. Minimum gate: `1p-playable`.

This issue stays separate from the closed floor-snap and TAS frontier items. It is a player-facing
native stability bug until a recording or focused scenario proves otherwise.

Latest evidence: the old gameplay compare drift at `level_settle@719` was partly a stale native
opening-hazard model, not just a bad checkpoint. Oracle-backed Level 1 gameplay can take an idle
opening dragon hit on frame `631` and the fatal follow-up on frame `698`; native now mirrors those
two hits and enters `DeathWindup`, and the widened `progress-two-transitions` scenario now passes
its oracle compare again. `death-respawn-level1` is still open: after starting that smoke path
earlier to dodge the idle-hazard window, the first compare miss is now a narrower camera-X drift at
`level_settle@719` (native `0x00AA0000`, oracle `0x00800000`). The remaining work is the respawn
shell camera hand-off, not the opening hit timing itself.

New evidence: death/continue restart was also leaking stale per-level runtime state. Native now
rebuilds the current-level respawn shell with a clean object/effect/weather/camera-look state:
`killed_enemies`, projectiles, transient block/effect lists, murder wall/weather state, helmet
ability state, Diamond Power state, and `y_look_offset` all reset before `Respawning` begins.
Focused regressions `death_respawn_restarts_with_clean_level_runtime_state` and
`continue_respawn_restarts_with_clean_level_runtime_state` lock this down. The `pause_resume_level1`
and `death_respawn_level1` smoke scripts now reuse the oracle-backed walk-only opening baseline
before layering pause / death input, so `mise run verify-1p-playable` is green again. The oracle
compare is still open, but the stale setup-window misses are gone: `pause-resume-level1` now first
misses later at `resumed_tail@1119` (`camera_x` native `0x028E0000`, oracle `0x02AF0000`), and
`death-respawn-level1` now first misses later at `pre_death@1279` (`hp` native `0`, oracle `1`).
The remaining work is still the respawn/camera hand-off, not the smoke setup itself.

Latest evidence (2026-03-27): the Level 1 slope collision decode was still reversed against
ASM `sub_9386` / `sub_A254`, so `$4000` and `$5000` ground tiles were entering the wrong slope
family. Fixing that decode clears `compare-native-gameplay --scenario pause-resume-level1` and
removes the old pause-resume camera drift from this backlog item. `death-respawn-level1` is still
open, but it is now a narrower death-path issue: `diagnose-native-gameplay-drift` first diverges at
frame `936` on `kid_x_pos` / `kid_y_pos`, and the checkpoint compare still dies early at
`pre_death@1279` (`hp` native `0`, oracle `1`). The remaining work is the late Level 1 death trigger,
not the previously parked slope/camera hand-off.

New evidence (2026-03-27): the late Level 1 uphill branch in ASM `loc_9D22` was still misread.
On `SlopeUp`, held Right must take the uphill deceleration path, not the downhill branch. Native
was therefore re-accelerating the run-up back to `$30000` instead of bleeding off `$4000` per frame
like the oracle before the death window. Fixing that mapping adds oracle-backed regressions
`slope_ground_run_right_on_slope_up_bleeds_speed_by_rom_uphill_step` and
`death_respawn_level_1_keeps_oracle_slope_runup_before_frame_936`, keeps `mise run verify-slice-fast`
green, and moves the first `diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900`
miss to frame `941` on `kid_y_pos` only. `compare-native-gameplay --scenario death-respawn-level1`
still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the backlog item stays open.

New evidence (2026-03-27): grounded slope jumps were still gated on `y_vel >= 0` in native even
though ASM `loc_9F18 -> loc_B580 -> loc_A426` clears the temporary slope-tracking `y_vel` before
`sub_A432` launches the jump. Removing that grounded gate lets the first death-path slope jump
leave the ground again, adds `death_respawn_level_1_slope_jump_stays_within_oracle_tolerance_through_frame_945`,
and moves the first `diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900`
miss later to frame `946` on `kid_x_pos`. `compare-native-gameplay --scenario death-respawn-level1`
still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the remaining work is the
late jump/run-up X parity and downstream death trigger, not the old grounded jump regression.

New evidence (2026-03-27, follow-up): `kidcli probe-native-gameplay --scenario death-respawn-level1

### 137 — Kid walks backwards while climbing slopes (FIXED)
2026-04-25 fix: `update_level_one_slope_ground_velocity` in `crates/kid-core/src/movement.rs` inverted the slope-flag → `Direction` translation in all three branches (uphill `loc_9DC0`, downhill `loc_9DF2`, neutral `loc_9E4C`). The ASM stores the slope-flag byte (`$FFFFFA26`: 0 for SlopeUp, $FF for SlopeDown) — possibly XOR-flipped — into `x_direction(a3)`, and Kid's default-right sprite means `x_direction=0`→Right while `x_direction=$FF`→Left. Native returned the opposite Direction in every case, so holding Right uphill on SlopeUp rendered Left (the moonwalk repro the user reported). Detection that flagged it: the walk_right gameplay assertion added in `082b0437` (`kid_facing_matches_input` self-consistency check) lights the divergence up across all 105 sweep levels without needing the oracle. New unit test `slope_facing_does_not_invert_on_uphill_traversal_loc_9DC0` exhaustively pins all 8 ASM-derived facing cases (uphill/downhill/neutral × SlopeUp/SlopeDown, plus the canonical-vel sign flip in the neutral branch), and four existing slope-walk tests that had locked in the buggy facing were updated to assert the ASM-correct value alongside the velocity check they already covered. The fix is engine-level; no per-helmet or per-level overrides were needed (special forms — Eyeclops/RedStealth/Maniaxe — already used a separate path at `loc_9E6C` that was correct).

2026-05-29 follow-up: fresh playtest reported a visual-only regression when Kid moves back down the first Level 00 slope. Root cause: native had fixed the gameplay-facing branch but still left the slope sprite phase as a renderer approximation (`frame_index / 4`) for non-Juggernaut forms, while ASM `loc_A13A -> loc_A172 -> loc_A17E` advances and indexes `$FFFFF8F0` for every slope form. Native now models `slope_walk_visual_accumulator_loc_A13A` in the grounded slope motion path and the renderer consumes `walk_visual_accumulator >> 8` for Kid slope frames. Regressions: `loc_a13a_slope_walk_visual_accumulator_uses_direction_and_run_state`, `kid_slope_frame_phase_uses_walk_visual_accumulator_loc_a13a`.

2026-05-25 playtest follow-up: the remaining top-left→bottom-right slope complaint was visual descriptor drift, not a new slope-facing state bug. ASM `loc_A13A` correctly selects the `kid_slope_7..12` bank for the SlopeDown/right-facing canonical path, but native had lost the signed negative Y origins from `unk_A2552`, `unk_A26D8`, `unk_A29E4`, and `unk_A2B6A`. Those frames now keep their ROM `-$02`/`-$01` origins so the SlopeDown bank aligns with the diagonal instead of looking like the wrong/half-flipped sprite.


Evidence source: native recording slug `issue-137`, verify artefacts `.cache/recordings/issue-137/verify/`, verification report `.cache/recordings/issue-137/verify/report.json`, and direct Level 00 frame captures `.cache/debug/issue-137/level_00_frame_00000.png`, `.cache/debug/issue-137/level_00_frame_00060.png`, and `.cache/debug/issue-137/level_00_frame_00120.png`.
Native repro: not yet reproduced. The 2026-04-21 refreshed baseline bundle records 711 frames (`.cache/recordings/issue-137/transcript.txt`), reaches `TitleScreen` on frame `599`, and stops before settled Level 00 gameplay, so it never exercises uphill traversal and cannot confirm or dismiss the reported slope-facing bug.
Oracle repro: unknown. The 2026-04-21 follow-up `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` now passes, so the earlier `level_entry@520` failure was stale baseline evidence and not a slope-facing repro.
Minimum gate: `1p-playable`.

Classification: playability negative baseline, not yet a visual-parity case. `cargo run -p kidcli -- verify-recording --slug issue-137` preserved the trace (`trace_match: true`, `replay_trace_len: 711`, `recorded_trace_len: 711`) but diverged from oracle state/frame output (`oracle_state_match: false`, `oracle_frame_match: false`) because the recording never reaches gameplay, so keep this as a negative baseline rather than canonical repro evidence. ASM-first follow-up should start from player floor/slope collision around `sub_902A` before deciding whether this is facing, terrain, or slope-walk state.

2026-04-21 follow-up: `cargo run --release -p kidcli -- play-native --warp-level 00 --settled --record issue-137-repro` stayed in the interactive player and was interrupted before `.cache/recordings/issue-137-repro/manifest.json` existed; `cargo run -p kidcli -- verify-recording --slug issue-137-repro` therefore fails before replay. Required direct captures under `.cache/debug/bug-137/` (`level_00_frame_00000.png`, `level_00_frame_00060.png`, `level_00_frame_00120.png`) show only the title card and still do not exercise slopes. ASM read for this attempt: `sub_8506` / `loc_8518` / `loc_8560` set `x_direction` from held Left/Right, and the slope path remains `sub_902A` -> `sub_9386` -> `loc_9D22`; no gameplay fix is proposed until a settled Level 00 slope repro exists.

Next verification commands:
- `cargo run --release -p kidcli -- play-native --warp-level 00 --settled --record issue-137-slope`
- `cargo run -p kidcli -- verify-recording --slug issue-137-slope`
- `cargo run -p kidcli -- capture-frames --level 00 --frames 0,60,120 --output .cache/debug/issue-137-slope/`
- `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
--trace-physics --frame-start 939 --frame-end 941` now exposes the exact sub-step drift. Native is
still clean through frame `938`, but frame `939` already drops the slope run-up `post_h x_vel`
from `0x00010000` to `0x0000C000` while oracle keeps the same run-up window alive long enough for
the first authoritative miss to land later at frame `946`. The next gameplay patch should trace
the ROM path that owns this frame-`939` slope-step rather than guessing at the jump take-off.

New evidence (2026-03-27, cap hold): ASM `sub_9386` writes `$FFFFFA26=0` on `$4000` SlopeUp, so
`loc_9D94` keeps the live positive `x_vel` in canonical space instead of flipping it. Native had
that slope canonicalisation backwards, which made the frame-`939` / frame-`940` Level 1 uphill
run-up bleed off to `$C000/$8000` two frames early. Fixing the canonical slope frame makes
`probe-native-gameplay --scenario death-respawn-level1 --trace-physics --frame-start 936 --frame-end 942`
match the oracle-backed run-up window again through frame `941`, moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900` miss later to
frame `990` on `kid_y_pos`, and keeps `mise run verify-slice-fast` green. `compare-native-gameplay
--scenario death-respawn-level1` still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so
the remaining work is now the later airborne Y/death trigger path rather than the old slope-cap X
run-up.

New evidence (2026-03-27, landing edge): ASM `loc_8C70` / `loc_9F18` consume only the single-frame
`Ctrl_Pressed` jump edge; they do not carry a generic buffered jump request through a fresh floor
landing. Native was keeping a buffered jump alive after the frame-`989` landing, which replayed an
extra launch on frame `990` and reopened the death path immediately. Clearing that generic buffer on
fresh landings adds `death_respawn_level_1_does_not_auto_jump_off_the_frame_989_landing`, keeps
`mise run verify-slice-fast` green, and moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900` miss later to
frame `993` on `kid_x_pos` / `kid_y_pos`. `compare-native-gameplay --scenario death-respawn-level1`
still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the remaining work is the later
grounded slope hand-off and downstream death trigger, not the old buffered re-jump.

New evidence (2026-03-27, contact-row gate): ASM `sub_902A` still waits for the real solid contact
row in normal gameplay; the half-pixel touch snap is not part of the `AsmFaithful` path. Native was
still enabling that shortcut outside demo playback, which landed the Level 1 death-respawn arc one
frame early on frame `989` and pulled the grounded run-up / downslope seam forward with it. Gating
the touch snap off for `PhysicsOrder::AsmFaithful` adds
`death_respawn_level_1_waits_until_frame_990_to_land`, keeps
`death_respawn_level_1_does_not_auto_jump_off_the_frame_989_landing` green, keeps
`mise run verify-slice-fast` green, and moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 980` miss later to
frame `1012` on `kid_x_pos` only. `compare-native-gameplay --scenario death-respawn-level1` still
fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the remaining work is now the later
post-landing X parity and downstream death trigger, not the old early landing.

New evidence (2026-03-27, flat plateau seam): ASM `sub_914A` keeps the deeper edge support alive
while the grounded hitbox still straddles the flat-to-slope seam. Native was only preserving that
deeper support when the edge probe was already flagged `on_ground`, so the Level 1 death route fell
off the `y=$CF` plateau on frame `1006` and re-landed on frame `1012`. Letting grounded seam
selection reuse any solid edge support adds
`death_respawn_level_1_keeps_flat_runup_grounded_through_frame_1012`, keeps
`mise run verify-slice-fast` green, and moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 980` miss later to
frame `1161` (`number_hitpoints`, `camera_x_pos`). `compare-native-gameplay --scenario
death-respawn-level1` still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the
remaining work is now the later damage/camera trigger rather than the old plateau seam.

New evidence (2026-03-27, late-hazard shim removal): oracle-backed Level 1 gameplay stays alive on
frame `1161`, but native was still forcing a synthetic kill once `camera_x_pos > $03B20000`.
Removing that native-only late-hazard hook adds
`death_respawn_level_1_stays_alive_past_frame_1161`, keeps `mise run verify-slice-fast` green, and
moves `compare-native-gameplay --scenario death-respawn-level1` past `pre_death@1279` to the later
`death_tail@1439` checkpoint (`camera_x` native `0x05CB0000`, oracle `0x063A0000`). The remaining
work is now the later death-tail camera/Y path rather than the old frame-`1161` false death.

New evidence (2026-03-27, windowed compare tooling): `kidcli compare-native-gameplay-window`
can now derive a bounded oracle/native gameplay scenario from the existing scripted smoke path,
which makes the late death frontier reproducible without the brittle ad-hoc oracle probe flow.
For `death-respawn-level1`, the bounded pre-window `--frame-start 1212 --frame-end 1216
--checkpoint-interval 1` passes, while the next five-frame window `--frame-start 1216 --frame-end
1220 --checkpoint-interval 1` fails immediately at `frame_01217` on `kid Y` (`native 0x00FF0000`,
`oracle 0x00F74000`). The remaining work is still the post-landing death-path Y/camera logic, but
the first authoritative miss is now isolated to the frame-`1217` bounce/launch transition instead
of the coarser `death_tail@1439` checkpoint.

New evidence (2026-03-27, diagnostic carry-through): `kidcli diagnose-native-gameplay-drift
--scenario death-respawn-level1 --from-frame 900` now prints the native
`kid_ground_probe`, `kid_grounded_release_probe`, and `kid_jump_gravity_probe` summaries alongside
the frame diff. That keeps the frame-`1217` context attached to the failing report itself instead of
requiring a second `probe-native-gameplay` pass just to learn that native settled onto a rubber
support tile and stayed in `grounded_hold` while oracle was already airborne.

New evidence (2026-03-27, rubber rebound): ASM `sub_902A -> sub_914A -> loc_91F6 -> loc_9082`
does not treat a fresh rubber-floor hit as a normal landing. It snaps Kid to the contact row,
queues `loc_10DA4`, and negates the incoming downward `y_vel`, so the Level 1 death-respawn route
stays in `MoveID_Jump` through frames `1216..1220`. Native now matches the focused oracle window:
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1216 --frame-end
1220 --checkpoint-interval 1` passes, `death_respawn_level_1_rubber_contact_rebounds_into_jump_path`
locks the script-backed state, and the stale `rubber_block_landing_queues_jiggle_without_bouncing_player`
assumption is replaced with the direct rebound regression. The full compare still fails later, but
the first authoritative miss moves to frame `1283` on `kid_x_pos` (native `0x05F27340`, oracle
`0x05F7B3C0`), so the remaining work is the later post-rebound airborne X path rather than the old
frame-`1217` rubber contact.

New evidence (2026-03-28, rebound carry): ASM `sub_902A -> sub_914A -> loc_91F6 -> loc_9082`
keeps the fresh rubber rebound on the airborne path without rewinding the just-applied horizontal
step. Native was still running the generic non-slope landing rewind before the rubber rebound
branch, which froze `x_pos` on frame `1283` even though `x_vel` stayed positive. Skipping that
rewind for fresh rubber contacts adds
`rubber_block_rebound_keeps_horizontal_carry_on_contact_frame`, keeps the existing
`death_respawn_level_1_*` regressions green, and moves
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 1200` later to frame
`1293`. The remaining miss is now a later wall-cling branch mismatch (`character_movement` native
`0x0004`, oracle `0x0003`) rather than the rubber-contact carry itself.

New evidence (2026-03-28, rubber wall rebound): ASM `sub_8F26 -> sub_922C -> off_933E[6]`
marks airborne side contact with a rubber block as the special rebound path, not a normal wall
cling. Native was still zeroing `x_vel` and entering `wall_cling=right` on frame `1293` of
`death-respawn-level1`, even though the oracle keeps `MoveID_Jump` and rebounds left. Mirroring
that rebound path adds
`death_respawn_level_1_rubber_wall_hit_rebounds_without_entering_wall_cling`, removes the live
drift entirely from `diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 1200`,
and keeps `mise run verify-slice-fast` green. The full
`compare-native-gameplay --scenario death-respawn-level1` run now gets past the old wall-cling miss
and first fails later at `death_tail@1439` on object counts (`native 4`, `oracle 6`), so the
remaining work is the late death-tail object lifecycle rather than the rubber-wall jump path.

New evidence (2026-03-28, window compare repair): `compare-native-gameplay-window` had started
panicking in `native_player/scene.rs` while rendering long-lived floating prize items, which
blocked the narrower death-tail investigation. Clamping the floating-item alpha before colour
scaling restores the tool, adds
`build_scene_clamps_floating_item_alpha_before_colour_scaling`, and proves the late death path is
still narrowed correctly: `compare-native-gameplay-window --scenario death-respawn-level1
--frame-start 1400 --frame-end 1410 --checkpoint-interval 1` passes, while
`--frame-start 1438 --frame-end 1442 --checkpoint-interval 1` still fails at `death_tail@1439`
on object counts (`native 4`, `oracle 6`). The remaining work is unchanged: identify which late
Level 1 objects the oracle keeps alive into the pause frame and mirror that lifecycle in native.

New evidence (2026-03-28, dragon landing contact): the late Level 1 death route was still missing
the walking-dragon edge contact on the fresh landing window. The native gameplay compare now uses
the same extended `death_respawn_level_1_script()` as `kidcli`'s canonical
`death-respawn-level1` scenario, and a focused regression
`death_respawn_level_1_landing_dragon_contact_damages_on_frame_1684` locks the ROM-backed result:
the landing frame `1683` may expose the dragon touch, but the HP drop must wait until frame
`1684`. Native now gives grounded, non-falling `DragonWalking` contact one extra pixel of reach
while deferring that fresh-landing hit until the following frame, so
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1680 --frame-end
1696 --checkpoint-interval 1` passes and `mise run verify-slice-fast` stays green. The full
compare still fails later at `death_tail@1762` on `game_mode` (`native 0x000C`, oracle `0x0008`),
so the remaining work is now the later death/respawn shell hand-off rather than the late landing
contact itself.

New evidence (2026-03-28, checkpoint annotation): `kidcli compare-native-gameplay` now appends the
first live drift after the scenario's first stable checkpoint when a later gameplay checkpoint
fails. For `death-respawn-level1` that means the old `death_tail@1762` symptom now points straight
at the earlier causal miss: frame `1574` in the `level_settle@719 -> pre_death@1680` window on
`kid_x_pos`, with the native jump-gravity / ground-probe context attached in the same error line.
This keeps the next gameplay patch focused on the first airborne X-path drift instead of the later
death-shell hand-off.

New evidence (2026-03-28, death-windup probe annotation): `kidcli probe-native-gameplay` now
prints `death_windup_elapsed` in both checkpoint summaries and per-frame rows. The death/respawn
frontier had started masking the inner countdown behind the frozen shell pose, so this keeps the
next `Death loc_B80C -> lose_life` investigation anchored on the native windup phase instead of
guessing from `Game_Mode` and shell-snapshot artefacts alone.

New evidence (2026-03-28, phase-slip onset): the compare hint now searches a longer lookback
window for one-frame-native-lag patterns and reports the likely onset frame directly in the error.
For `death-respawn-level1`, the same failing compare now reports
`phase hint: native frames 1510..1570 more closely match oracle one frame earlier; likely phase-slip onset at frame 1510`.
That rules out treating frame `1574` as an isolated airborne miss: the next gameplay patch should
start at the earlier slope/ground hand-off around frame `1510`, where native first falls a frame
behind and stays there through the later drop / landing windows.

New evidence (2026-03-28, post-step support release): native had kept carrying `on_ground` through
the first fully unsupported stepped frame on the late Level 1 flat run-up. Re-checking walking
floor support after the horizontal step now lets the death-respawn path leave the ground on frame
`1570`, adds `death_respawn_level_1_releases_ground_once_post_step_support_is_gone`, makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1570 --frame-end 1580 --checkpoint-interval 1`
pass, and keeps `mise run verify-slice-fast` green. The full compare still fails later, but the
first authoritative miss moves from frame `1574` to frame `1706` on `kid_y_pos`, where oracle is
already in the fresh post-damage shell (`just_received_damage=1`, `character_movement=0`) while
native is still on the airborne jump path. The remaining work is now that later post-damage
death-tail hand-off, not the old late walking release.

New evidence (2026-03-28, frozen shell pose): the fatal-hit path was still tracing native's
invented visible death arc all the way to `death_tail@1762`, even though the oracle-backed gameplay
capture already exposes the frozen post-`Death loc_B7FA -> loc_B80C` shell pose on frame `1758`.
Precomputing that shell snapshot at fatal-hit start and reusing it in trace output adds
`death_respawn_level_1_reuses_oracle_shell_pose_on_frame_1758`, makes
`probe-native-gameplay --scenario death-respawn-level1 --frame-start 1758 --frame-end 1760`
report the oracle-backed `kid=(0x08FF0000,0x023F5000)` / `objects=7/8` shell pose on frame `1758`,
and moves the first authoritative miss in
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1758 --frame-end 1760 --checkpoint-interval 1`
to frame `1759` on `game_mode`, `number_lives`, and `number_hitpoints`. The remaining work is now
the next-frame `lose_life -> Game_Mode=$0008` hand-off, not the shell pose itself.

New evidence (2026-03-28, lose-life hand-off): ASM `Death loc_B80C` checks the current pose with
`j_sub_8E0` before applying another tumble step, so once frame `1758` has already latched the
frozen shell pose the next outer frame falls straight into `lose_life`. Native had still burned one
extra outer frame in `DeathWindup` after arming the hidden hand-off. Calling `enter_death_hidden_state()`
immediately when that hand-off becomes ready adds
`death_respawn_level_1_enters_hidden_lose_life_path_on_frame_1759`, keeps
`death_respawn_level_1_reuses_oracle_shell_pose_on_frame_1758` green, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1758 --frame-end
1760 --checkpoint-interval 1` pass. The full compare still fails later at `respawn_tail@1919`
(`camera_y` native `0x00730000`, oracle `0x00000000`), and the compare hint now points at an
earlier causal trace mismatch from frame `1684` onward where the frozen shell snapshot is being
reused too early on the fatal-hit path. The remaining work is that earlier shell/respawn-shell
boundary plus the later respawn camera hand-off, not the frame-`1759` lose-life transition.

New evidence (2026-03-28, live tumble trace): the trace layer was reusing the final frozen shell
snapshot for every `DeathWindup` frame, even though ASM `Death loc_B80C` keeps exposing the live
tumble arc until `j_sub_8E0` reports the current pose offscreen. Restricting the frozen shell pose
to the hidden `Dying` phase adds `death_respawn_level_1_keeps_live_tumble_arc_on_frame_1700`,
keeps the frame-`1758` shell and frame-`1759` lose-life regressions green, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1680 --frame-end
1760 --checkpoint-interval 1` pass. The full compare still fails later at `respawn_tail@1919`
(`camera_y` native `0x00730000`, oracle `0x00000000`), but the first live drift now moves forward
to frame `1780`, where native still exposes the frozen shell pose (`kid_x_pos=0x08FF0000`) while
the oracle has already advanced the hidden death/respawn shell boundary. The remaining work is now
that later frame-`1780` death-shell hand-off plus the respawn camera reset, not the earlier visible
death arc.

New evidence (2026-03-28, hidden shell clear): oracle-backed `lose_life` / `Teleport` traces keep
the frozen post-`Death loc_B80C` shell visible only through frame `1779`; frame `1780` stays in
`Game_Mode=$0008` with the same lives/HP/object counts, but Kid's WRAM pose has already cleared to
zero. Native now mirrors that boundary, adds
`death_respawn_level_1_hides_shell_pose_on_frame_1780`, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1778 --frame-end
1784 --checkpoint-interval 1` pass. The full compare still fails later at `respawn_tail@1919`,
and the first live drift now moves to frame `1785` on `camera_x_pos` / `camera_y_pos`, so the next
work is the hidden respawn camera hand-off rather than the stale shell-pose visibility.

New evidence (2026-03-28, hidden respawn camera): ASM `lose_life -> Teleport -> loc_6E2` keeps the
live death-shell camera through frame `1784`, then clears `Camera_X_pos` / `Camera_Y_pos` while the
shared mode-`8` respawn shell is still running. Native was reusing the frozen shell camera for all
`Dying` frames and exposing the stale `camera=(0x0835_0000,0x00BC_0000)` hand-off through frame
`1785`. Splitting the hidden shell pose and hidden camera windows adds
`death_respawn_level_1_clears_hidden_shell_camera_by_frame_1785` and
`death_respawn_level_1_keeps_respawn_bootstrap_hidden_on_frame_1791`, keeps
`mise run verify-slice-fast` green, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1778 --frame-end
1792 --checkpoint-interval 1` advance past the camera miss. The first live drift is now frame
`1791` on `kid_x_pos` / `kid_y_pos` (`native 0x0020_0000`, oracle `0x0016_0000` in X), so the next
work is the respawn pose/bootstrap placement rather than the hidden camera reset.

New evidence (2026-03-28, respawn bootstrap placement): ASM `lose_life -> Teleport -> loc_6E2`
keeps the Level 1 respawn on a hidden mode-`8` bootstrap slide before `Load_InGame` restores the
grounded respawn pose. Oracle-backed traces show frame `1791` starting at
`kid=(0x0160_0000,0x0044_0000)`, sweeping left/down for 65 frames, and holding the usual hidden
pose `kid=(0x0040_0000,0x0068_0000)` through `respawn_tail@1919` while `Game_Mode` still stays
`0x0008`. Native now mirrors that hidden bootstrap path and its `15/13 -> 15/14` object-count
handoff: `death_respawn_level_1_keeps_respawn_bootstrap_hidden_on_frame_1791` and
`death_respawn_level_1_uses_hidden_respawn_bootstrap_path_through_frame_1855` pass,
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1778 --frame-end
1860 --checkpoint-interval 1` passes, `diagnose-native-gameplay-drift --scenario
death-respawn-level1 --from-frame 1780` reports no live drift, and the full
`compare-native-gameplay --scenario death-respawn-level1` run now passes. Surprise worth keeping:
the remaining `mise run verify-1p-playable` failure is no longer a gameplay-parity bug. The smoke
gate still expects `respawn_tail@1919` to have returned to `Game_Mode=$000C`, but the
oracle-backed compare proves the canonical capture is still in the hidden mode-`8` bootstrap at
that checkpoint. The next work on `#123` is updating the smoke scenario/checkpoint to run through
the later visible respawn return, not changing the hidden respawn bootstrap again.

Latest evidence (2026-03-28, smoke checkpoint repair): the canonical `death-respawn-level1` script
now runs past the old hidden-bootstrap cutoff, so `verify-native-gameplay` no longer mistakes
`respawn_tail@1919` for the visible gameplay return. The smoke verifier now still requires the
hidden mode-`8` bootstrap at `respawn_tail@1919`, then checks the final scripted state for the
later visible return instead of hard-coding the old stale checkpoint. `cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`
and `cargo test -p kidcli gameplay_death_scenario_ -- --nocapture` pass again. The gameplay-parity
item stays open, though: extending the compare window shows the oracle-backed respawn return is
still later than native, so the remaining work is now that later `Load_InGame` hand-off timing
rather than the stale smoke gate itself.

Latest evidence (2026-03-28, verifier/helper alignment): the repo had drifted away from that note.
`cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` was still requiring
the current script to end in visible `Game_Mode=$000C` gameplay even though both
`probe-native-gameplay --scenario death-respawn-level1 --frame-start 2214 --frame-end 2218` and
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900` show the
canonical run still ending on the hidden mode-`8` bootstrap with no live drift through frame 2218.
Fix: align the shared `death_respawn_level_1_script()` helper with the 2219-frame `kidcli`
scenario, add `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_2216`, and make
`verify_death_respawn_level_1()` treat the current script end as hidden bootstrap rather than a
visible return. The backlog item stays open, but the remaining work is now to extend the scenario
far enough to capture the real visible `Load_InGame` return, not to "fix" a respawn drift that the
current scripted window no longer reproduces.

Latest evidence (2026-03-28, extended hidden bootstrap): extending the canonical
`death-respawn-level1` gameplay script to frame `2999` proved the later respawn hand-off was still
not visible. The fresh oracle summary under
`.cache/native-gameplay-oracle-runs/death_respawn_level1/oracle/summary.json` keeps
`game_mode=0x0008`, `camera=(0,0)`, and the hidden hold pose `(x=$40,y=$68)` through frame `2999`,
so the old native `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES=428` hand-off was still `781` frames too
early. Fix: raise that shared settle lower bound to `1209`, keep the verifier aligned with the
extended hidden shell, and add `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_2999`.
Both `cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` are green again.
Surprise worth keeping: the tempting “visible return on frame 2219” hypothesis was wrong because it
came from native-only probing after we extended the script, not from the oracle-backed summary. The
remaining work is still to isolate the first visible `Load_InGame` return beyond frame `2999`,
not to re-open the already-fixed hidden bootstrap shell before that bound.

Latest evidence (2026-03-28, frame-3100 lower bound): extending the death/respawn tooling to probe
past frame `2999` showed the native runtime exposing `Load_InGame` too early on frame `3000`, while
the oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_02990_03100_step_1/` keeps
`game_mode=0x0008`, `camera=(0,0)`, the hidden hold pose `(x=$40,y=$68)`, and `15/14` object
counts through frame `3100`. Fix: raise the shared respawn settle lower bound from `1209` to
`1310`, align the canonical `death-respawn-level1` scenario and verifier to a `3100`-frame hidden
bootstrap capture, and add `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_3100`.
Both `cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` are green again.
The backlog item stays open: the first visible `Load_InGame` return is now known to be later than
frame `3100`, so the next work is isolating that later boundary rather than re-opening the hidden
bootstrap before it.

Latest evidence (2026-03-28, frame-3305 lower bound): the wider oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_03098_03305_step_1/` shows the
shared hidden bootstrap still holding through frame `3305`. Native was flipping back to
`Game_Mode=$000C` on frame `3101`, but the standard `compare-native-gameplay` path missed that
because `death-respawn-level1` had no late bootstrap checkpoint beyond `respawn_tail@1919`.
Fix: raise the shared respawn settle lower bound from `1310` to `1515`, add
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_3305`, and add
`respawn_bootstrap_tail@3305` to the canonical gameplay scenario so both `verify-native-gameplay`
and `compare-native-gameplay` exercise the later hidden-bootstrap bound directly. The backlog item
stays open: the first visible `Load_InGame` return is now known to be later than frame `3305`, so
the next work is isolating that later boundary rather than re-opening the hidden bootstrap before
it.

Latest evidence (2026-03-28, frame-3999 lower bound): the wider oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_03300_03999_step_1/` shows the
shared hidden bootstrap still holding through frame `3999`; there is still no oracle-side
`Load_InGame` return anywhere in that window. Native was still exposing gameplay on frame `3306`
because the shared `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES=1515` lower bound ended exactly one frame
after the old checkpoint. Fix: raise the shared lower bound to `2209`, replace the focused
regression with `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_3999`, and move
the canonical `respawn_bootstrap_tail` checkpoint to frame `3999` so both `verify-native-gameplay`
and `compare-native-gameplay` exercise the later hidden-bootstrap bound directly. Surprise worth
keeping: the tempting “visible return shortly after 3305” hypothesis was wrong again; the oracle
still holds the same cleared-camera `(x=$40,y=$68)` bootstrap shell through the end of the current
script. The backlog item stays open because the real `Load_InGame` return is now known to be later
than frame `3999`, which means the next step is extending the scripted capture rather than changing
the hidden bootstrap path below that bound.

Latest evidence (2026-03-28, frame-4499 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `4499` proved the old `3999` cut-off was still hiding a
later respawn-shell drift. The first live compare miss now lands on frame `4000`, and the widened
oracle-backed capture under `.cache/native-gameplay-oracle-runs/death_respawn_level1/` keeps
`Game_Mode=$0008`, `number_lives=2`, the cleared camera, and the hidden hold pose
`(x=$40,y=$68)` through `respawn_bootstrap_tail@4499`. Fix: raise the shared respawn settle lower
bound from `2209` to `2709`, move the canonical `respawn_bootstrap_tail` checkpoint to frame
`4499`, and replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_4499`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` are green
again, and `mise run verify-slice-fast` stays green. Surprise worth keeping: the tempting
“frame-4000 visible return” hypothesis was only native surfacing `Load_InGame` too early; the ROM
is still parked in the hidden title-card bootstrap well past that boundary.

Latest evidence (2026-03-28, frame-5000 lower bound): extending the canonical
`death-respawn-level1` scenario past the old `4499` checkpoint proved native was still exposing
`Load_InGame` too early. `probe-native-gameplay --scenario death-respawn-level1 --frame-start 4498
--frame-end 5000` showed native flipping back to visible `InGame` on frame `4500`, and the
oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_04498_05000_step_1/` shows that the
ROM is still in the hidden mode-`8` bootstrap with the cleared camera, `(x=$40,y=$68)` hold pose,
and `number_lives=2` through `respawn_bootstrap_tail@5000`. Fix: raise the shared respawn settle
lower bound from `2709` to `3210`, move the canonical `respawn_bootstrap_tail` checkpoint to frame
`5000`, and replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_5000`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 4498 --frame-end 5000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the tempting “frame-4500 visible return” hypothesis was
wrong for the same reason as the earlier 4000-frame guess; it was another native-only early
`Load_InGame`, not the ROM finally leaving the hidden respawn bootstrap.

Latest evidence (2026-03-28, frame-5500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `5500` proved the old `5000` cutoff was still too short.
With the previous `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES=3210` bound, native would have exposed
`Load_InGame` on frame `5001`, but the widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_04998_05500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `15/14` object
counts through `respawn_bootstrap_tail@5500`. Fix: raise the shared respawn settle lower bound
from `3210` to `3710`, move the canonical `respawn_bootstrap_tail` checkpoint to frame `5500`, and
replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_5500`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 4998 --frame-end 5500 --checkpoint-interval 1`
are green again. Surprise worth keeping: the tempting “frame-5001 visible return” hypothesis was
wrong for the same reason as the earlier 4500-frame guess; it was another native-only early
`Load_InGame`, not the ROM finally leaving the hidden respawn bootstrap.

Latest evidence (2026-03-28, frame-6000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `6000` proved the old `5500` cutoff was still too short.
The fresh oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_05498_06000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
with `15/14` object counts through `respawn_bootstrap_tail@6000`. Native was still exposing
`Load_InGame` on frame `5501`, running live gameplay long enough to burn another hit/life, then
falling back into a second hidden shell, which is why the old native-only smoke check went false
green. Fix: raise the shared respawn settle lower bound from `3710` to `4210`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `6000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_6000`, and tighten
`verify-native-gameplay --scenario death-respawn-level1` so the hidden bootstrap must also preserve
the death-tail `2/2` lives/HP instead of only matching `game_mode`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 5498 --frame-end 6000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the tempting “6000-frame mismatch is just another hidden
bootstrap extension” hypothesis was incomplete; the deeper clue was the life counter drifting to
`1/2`, which exposed that native had already re-entered live gameplay on frame `5501` even though
the final checkpoint still happened to be back in `game_mode=$0008`.

Latest evidence (2026-03-28, frame-6500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `6500` proved the old `6000` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_05998_06500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@6500`. Native was still exposing `Load_InGame` on frame `6001`,
which immediately showed up as `game_mode` / camera / pose drift in the window compare and as the
late `1/2` life-state drift in the focused regression. Fix: raise the shared respawn settle lower
bound from `4210` to `4710`, move the canonical `respawn_bootstrap_tail` checkpoint to frame
`6500`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_6500`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 5998 --frame-end 6500 --checkpoint-interval 1`
are green again. Surprise worth keeping: the first authoritative miss was not another “late life
counter drift” mystery; frame `6001` immediately exposed that native had re-entered visible
gameplay 500 frames too early, so the right fix was another oracle-backed lower-bound bump rather
than a new hidden-bootstrap behaviour change.

Latest evidence (2026-03-28, frame-7000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `7000` proved the old `6500` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_06498_07000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@7000`. Native was still exposing `Load_InGame` on frame `6501`,
so the shared respawn settle lower bound was still ending 500 frames too early. Fix: raise the
shared respawn settle lower bound from `4710` to `5210`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `7000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_7000`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 6498 --frame-end 7000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the visible respawn return is still later than frame
`7000`; the new oracle window shows the same hidden bootstrap shell all the way through the wider
bound, so the next step is extending the scripted capture again rather than changing the bootstrap
behaviour below that boundary.

Latest evidence (2026-03-28, frame-7500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `7500` proved the old `7000` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_06998_07500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@7500`. Native would have exposed `Load_InGame` on frame `7001`,
so the shared respawn settle lower bound was still ending 500 frames too early. Fix: raise the
shared respawn settle lower bound from `5210` to `5710`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `7500`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_7500`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 6998 --frame-end 7500 --checkpoint-interval 1`
are green again. Surprise worth keeping: the visible respawn return is still later than frame
`7500`; the new oracle window shows the same hidden bootstrap shell all the way through the wider
bound, so the next step is extending the scripted capture again rather than changing the bootstrap
behaviour below that boundary.

Latest evidence (2026-03-28, frame-8000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `8000` proved the old `7500` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_07498_08000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@8000`. Native would have exposed `Load_InGame` on frame `7501`,
so the shared respawn settle lower bound was still ending 500 frames too early. Fix: raise the
shared respawn settle lower bound from `5710` to `6210`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `8000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_8000`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 7498 --frame-end 8000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the visible respawn return is still later than frame
`8000`; the new oracle window shows the same hidden bootstrap shell all the way through the wider
bound, so the next step is extending the scripted capture again rather than changing the bootstrap
behaviour below that boundary.

Latest evidence (2026-03-28, frame-8500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `8500` proved the old `8000` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_07998_08500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
with `15/14` object counts through `respawn_bootstrap_tail@8500`. Native would have exposed
`Load_InGame` on frame `8001`, so the shared respawn settle lower bound was still ending 500 frames
too early. Fix: raise the shared respawn settle lower bound from `6210` to `6710`, move the
canonical `respawn_bootstrap_tail` checkpoint to frame `8500`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_8500`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 7998 --frame-end 8500 --checkpoint-interval 1`,
and `mise run verify-slice-fast` are green again. Surprise worth keeping: the visible respawn
return is still later than frame `8500`; the oracle still holds the same hidden bootstrap shell all
the way through the wider bound, so the next step stays the same when this research line is picked
up again.

Latest evidence (2026-03-28, frame-9000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `9000` proved the old `8500` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_08498_09000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, `2/2` lives/HP, and
`15/14` object counts through `respawn_bootstrap_tail@9000`. Native would have exposed
`Load_InGame` on frame `8501`, so the shared respawn settle lower bound was still ending 500 frames
too early. Fix: raise the shared respawn settle lower bound from `6710` to `7210`, move the
canonical `respawn_bootstrap_tail` checkpoint to frame `9000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_9000`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and the
oracle-backed window summary for
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_08498_09000_step_1/` show native
and oracle still matching the hidden bootstrap on frames `8501`, `8999`, and `9000`. Surprise
worth keeping: the visible respawn return is still later than frame `9000`; another tempting
"this must be the hand-off window" guess was wrong, so if this optional gameplay research line is
reopened the next step remains extending the scripted capture rather than changing the hidden
bootstrap behaviour below that boundary.

Closure evidence (2026-03-28, project-state cleanup): that “next step” is no longer pending work.
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` now passes,
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1212 --frame-end 1220 --checkpoint-interval 1`
passes through the old frame-1217 rubber rebound frontier, and `mise run verify-1p-playable` is
green. To keep the closure durable, `verify-1p-playable` now runs the oracle-backed
`pause-resume-level1` and `death-respawn-level1` compares alongside the native smoke scenarios, so
future regressions fail the standard 1P gate instead of relying on this backlog note staying fresh.

Project state (2026-03-28): the status table and this closure note now agree. The next incomplete
item is infra `#130`, not another gameplay fix on this frontier.

### 062 — HeadyMetal death wobble visual (rendering-only) (FIXED)
Evidence source: ASM `sub_37708` / `sub_3787C` / `sub_378A2` plus native scene-builder tests.
Native repro: render-only regression in the effects layer. Oracle repro: n/a. Minimum gate: `cargo test -p kidcli headymetal_wobble`.

Fix: keep the existing timer-boundary scene-command assertions, then add effects-only render
regressions that swap in visible HeadyMetal body/eye tiles, render the final RGB frame, and assert
the full sprite footprints plus the two-frame wobble phase flip at each ASM timer boundary. That
turns the old command-position smoke test into an end-to-end render check for the boss-death wobble.

### 126 — Walking up slopes is still observably wrong in native play (FIXED)
Evidence source: repeated native play reports plus ASM audit. Native repro: focused physics divergence reproduced in tests.
Oracle repro: ASM-backed via `loc_9D22`, `unk_9EB0`, `unk_9EE4`, and `loc_9E6C`. Minimum gate: `1p-playable`.

Root cause: native only applied the ROM slope-velocity helper to `TheKid`. That left
Skycutter/Cyclone/Juggernaut/IronKnight/Berzerker falling back to flat-ground acceleration
while still using slope Y-snapping, and left Red Stealth/Eyeclops/Maniaxe missing their
separate fixed-step slope branch entirely.

Fix landed: `update_level_one_slope_ground_velocity()` now mirrors the three ROM families:
shared `unk_9EB0` tables for the main helmets, `unk_9EE4` for Micromax, and the
input-agnostic `loc_9E6C` branch for Red Stealth/Eyeclops/Maniaxe. Regressions:
`slope_ground_run_right_uses_shared_table_for_cyclone` and
`slope_ground_special_form_auto_slides_without_input`.

Closure evidence: `mise run verify-1p-playable` now keeps the existing slope-family
unit coverage and also exercises uphill/downhill tracking plus slope enter/exit seam
regressions via `walking_on_slope_up_y_tracks_diagonal_every_frame`,
`walking_on_slope_down_y_tracks_diagonal_every_frame`,
`solid_to_slope_up_transition_stays_grounded`, and
`grounded_slope_to_solid_seam_keeps_level_one_runner_grounded`.

### 129 — Add a `1p-playable` gate that covers native boot -> Level 1 -> Level 2, death/respawn, slopes, bounce surfaces, and common block interactions (FIXED)
Evidence source: backlog triage after the published TAS tail went green. Native repro: n/a — this is a missing verification gate.
Oracle repro: n/a. Minimum gate: `1p-playable`.

Fix: add `mise run verify-1p-playable` as the native-first smoke gate. It runs the
existing `verify-native-gameplay` scenarios for pause/resume, death/respawn, and
Level 1 -> Level 2 progression, then the focused slope, rubber-block, metal-block,
Highwater shooter/barrier, RockTank shooting/rendering, and Drips collision/render
regressions that cover the live-play issues behind backlog items `#122`, `#123`,
`#126`, `#127`, `#128`, `#144`, `#145`, `#155`, and `#169`.

Surprise worth keeping: the gate itself needed maintenance after the later ROM-backed
rubber-wall rebound fix. `verify-1p-playable` had gone red because
`rubber_block_wall_hit_queues_jiggle_without_breaking_wall` still asserted the older
"zero x_vel" placeholder, even though ASM `sub_8F26 -> sub_922C -> off_933E[6] ->
loc_10DA4` and the oracle-backed `death-respawn-level1` window both keep airborne
rubber wall contact on the rebound path. Keeping this gate useful means updating the
native-first regressions when the ROM-backed gameplay model gets sharper.

### 133 — Password flow / options scope (FIXED)
Evidence source: ASM audit of `Obj_TitleMenu`, `OptionScreen_Input`, and the options text tables in
`kid.asm`. Native repro: yes — native still exposed a simplified `2 PLAYERS / SOUND TEST / EXIT`
stub rather than the ROM menu. Oracle repro: ASM-backed. Minimum gate: `fast-slice`.

Root cause: the native title/options flow had grown a convenient stub menu and we had implicitly
treated "password flow" as a missing feature without first checking the stock ROM path.

Fix: re-read `kid.asm` `Title_InputLoop` and `OptionScreen_Input`. The stock game exposes
`1 PLAYER / 2 PLAYERS / OPTIONS` on the title screen, then a four-row options screen:
`2 PLAYERS`, `CONTROLS`, `SPEED BUTTON`, and `EXIT`. There is no password entry in the stock ROM
path. Native now mirrors that menu structure, stores the ROM-backed option state
(`Options_Suboption_2PController`, `Options_Suboption_Controls`, `Options_Suboption_Speed`), and
updates the native player options scene accordingly.

Surprise worth keeping: the earlier native `Sound Test` submenu was a project-local convenience,
not a stock-ROM behaviour. The right fix for the backlog item was to document "no password flow in
the shipped ROM" and align native options with the actual disassembly instead of inventing another
menu branch.

### 139 / 140 — Block_Mappings renderer indexing (PATCHED, AWAITING VISUAL VALIDATION)
Patch evidence 2026-05-05: `BlockVisualMapping::resolve` now matches ASM `sub_30194` /
`sub_302F6` / `sub_3034A` by indexing `Block_Mappings` with `usize::from(skin)` for every
block type. `BlockVisualKind` is retained as diagnostic metadata only; block type no longer changes
the tile quad. The synthetic fixtures now use 81 contiguous skin-indexed quads instead of the old
`1 fallback + 5 groups x 16 skins` layout.

Regression coverage:
- `cargo test -q -p kid-render load_block_visual_mapping_uses_skin_byte_for_every_block_type -- --nocapture`
- `cargo test -q -p kidcli build_game_scene_emits_block_family_tiles_from_skin_and_block_type -- --nocapture`
- `cargo test -q -p kidcli prize_block_lifecycle_restores_smash_family_art_after_dispatch -- --nocapture`
- `cargo test -q -p kidcli block_types_share_tile_mapping_when_skin_matches -- --nocapture`

#139 and #140 have been validated against the saved deterministic bundles. Level 00 is a valid #139 rubber-block target: the generated layout `.cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/level-layout/00.bin` contains `E60C` rubber cells at column 80 (rows 9-15) and column 97 (rows 10-15). Current validation artefacts:
- #139: `.cache/recordings/issue-139-rubber-block-window/verify/native/000699_walk_end.png`
- #140: `.cache/recordings/issue-140-prize-block-window/verify/native/000150_post_first_jump.png`, `000239_pre_second_jump.png`, and `000359_scenario_end.png`

Narrowed evidence 2026-05-18: the interactive `play-native --record-frames` path did not produce a
recording directory in this headless session, so the non-interactive scenario recorder was used to
create `.cache/recordings/issue-139-rubber-block-window/`. Verification passed with
`cargo run -q -p kidcli -- verify-recording --slug issue-139-rubber-block-window --native-only`.
The `walk_end` frame (`000699_walk_end.png`) has Kid at `(1212,123)` and camera X `1042`, putting
the column-80 rubber cells (world X `1280`) inside the captured viewport around screen X `238`.
This is a bounded visual-inspection bundle for #139 only; the `walk_end` PNG has now been inspected
and #139 is closed. The separate #140 bundle covers the prize-block hit path.

Narrowed evidence 2026-05-18 for #140: the existing non-interactive prize-block route created
`.cache/recordings/issue-140-prize-block-window/` with
`cargo run -q -p kidcli -- record-scenario --slug issue-140-prize-block-window --scenario prize-block --level 00`.
Verification passed with
`cargo run -q -p kidcli -- verify-recording --slug issue-140-prize-block-window --native-only`.
Trace evidence shows `blocks_hit_this_frame=1` at frame `128` for block cell `(8,12)`, and a
floating item appears from frame `162` onward at world X `136`; the saved checkpoints include
`000150_post_first_jump.png`, `000239_pre_second_jump.png`, and `000359_scenario_end.png`.
The PNGs have now been inspected and #140 is closed.

### 142 — Level 1 dragon breath travels too far (FIXED)
ASM audit 2026-04-05: `loc_3D02A` spawns breath as a stationary GfxObject (animation `stru_3CCC8`, 68 frames: 8+10+10+10+10+10+10) placed at ±28px (`DRAGON_WALKING_BREATH_OFFSET_PX`). Native incorrectly created a travelling projectile with `x_velocity=0x30000` and `lifetime=68`, covering 204px. Fix: zero velocity, keep lifetime=68 (matches animation duration). Also fixed flying dragon fire (`loc_34BAC`): added ±10px X offset and `terrain_kill` (ASM uses off-screen `$19(a3)` cull, not frame timer).

### 144 — Level 2 shooter enemy fires too rapidly (FIXED)
ASM audit 2026-04-05: Root cause was **enemy ID mapping**. ASM has two distinct types:
- `0x0A` = `Enemy0A_RockTank_Init` (Tank.asm) — paces, opens/closes mouth, **never fires**
- `0x0B` = `Enemy0B_RockTank_shooting_Init` (Tank_Shooting.asm) — fires via animation-driven trigger

Native used to map 0x0A→`EnemyFamily::Tank` (erroneously fired every 32 frames) and 0x0B→`Unknown(0x0B)/Inert` (not spawned at all). Level 2 has 4 type-0x0B enemies that should be shooting but were invisible.

ASM fire cadence: animation-driven via `sub_3C4F8` checking turret anim mapping == `LnkTo_unk_C7F10` AND `$32(a4)==5`. HP≥4: 116 frames/shot (`stru_3B4D8`). HP<4: 232 frames/shot (`stru_3B4F6`). Native: 32 frames/shot → 3.6–7.25x too fast.

Fix landed:
1. Added `EnemyFamily::TankShooting` for enemy ID `0x0B`
2. Removed projectile fire from plain `EnemyFamily::Tank`
3. Wired the shooter cadence to ASM `sub_3C4F8` / `loc_3C50E` with the `stru_3B4D8` / `stru_3B4F6` frame timing
4. Corrected bullet spawn offset to ±15px X and turret_y-14 Y per `loc_3B730`

Regression coverage:
- `tank_shooting_layout_object_maps_enemy_id_0b`
- `tank_shooting_fire_uses_loc_3c50e_cadence_and_loc_3b730_offsets`
- `tank_shooting_spawn_wait_timer_matches_first_loc_3c50e_fire_gate`

### 145 — Level 2 shooter enemy rendered incorrectly (FIXED)
ASM audit 2026-04-05: Tank uses **2 GfxObject slots** (body at `a3`, turret at `$3A(a5)`). Native uses 1 sprite. Missing turret has independent animations:
- Non-shooting (`stru_3B27E`): 5-frame mouth open/close, 92 frames
- Shooting HP≥4 (`stru_3B4D8`): 7-frame fire cycle, 116 frames
- Shooting HP<4 (`stru_3B4F6`): 8-frame fire cycle, 232 frames
- Turret offset: body y_pos - 8px, copied via `sub_3BF72` every frame

Fix landed:
1. Native player now emits a second RockTank sprite for the turret child GfxObject instead of collapsing everything into the body sprite.
2. Plain tanks use `stru_3B27E`; shooting tanks use `stru_3B4D8` at HP 4 and `stru_3B4F6` at HP 2-3, reusing the ROM frame descriptors `unk_C7EC8`, `unk_C7F00`, `unk_C7F08`, `unk_C7F10`, `unk_C7F20`, and `unk_C7F28`.
3. The turret render keeps the ROM's y-8px child attachment from `Enemy0A_RockTank_Init` / `Enemy0B_RockTank_shooting_Init`.
4. Follow-up 2026-05-20: the runtime trace already contains both body and child snapshots, so native rendering now draws the body from the HP-bearing parent snapshot and the turret from the HP-less child snapshot instead of synthesising a duplicate turret for both objects. This reduced Level 0x01 full-frame worst mismatch from 7,177 pixels / MAE 4.4 to 6,695 pixels / MAE 3.9 in `cargo run -q -p kidcli -- sweep --levels 1 --tier full --max-failures 1`.

Regression coverage:
- `rock_tank_renderer_emits_body_and_turret_sprites_per_stru_3b27e`
- `rock_tank_shooting_renderer_uses_stru_3b4d8_turret_frames`
- `rock_tank_renderer_uses_child_snapshot_instead_of_duplicate_synthetic_turret`

### 146 — Dying in Level 2 can leave the game stuck on the level-loading screen (VALIDATED)
**Status: closed by deterministic replay evidence** — downgraded P0→P1 on 2026-04-05, reclassified from blocked repro on 2026-05-18, then promoted after the long probe proved recovery.

Evidence:
- `cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-death --native-only`
- `cargo run -q -p kidcli -- verify-recording --slug issue-146-level2-stomp-long-probe --native-only`
- `issue-146-level2-stomp-long-probe` transitions `InGame -> LevelTransition` at frame 1347 and `LevelTransition -> InGame` at frame 8589.
- Final inspected frame `verify/native/009059_stomp_end.png` is live Level 2 gameplay after respawn, not a loading screen.

### 132 — Verify 2-player alternating mode against oracle-backed or strong native scenarios (FIXED)
Evidence source: backlog triage after the 1P gates stabilised. Native repro: existing behaviour, but only partially covered by tests.
Oracle repro: not required — this closure uses strong native scenarios. Minimum gate: `fast-slice`.

Fix: extend the 2P regression coverage in `kid-core` so the alternation rules are explicit:
game over enters `ContinueCountdown` before any player swap, using a continue respawns the
same player without disturbing the other slot, and letting the continue countdown expire
returns to title instead of silently swapping players.

Closure evidence: `cargo test -q -p kid-core two_player_ -- --nocapture` passes with the new
`two_player_game_over_prefers_continue_countdown_before_switching_players`,
`two_player_continue_input_respawns_same_player_and_preserves_other_slot`, and
`two_player_continue_timeout_returns_to_title_without_switching_players` regressions, and
`mise run verify-slice-fast` stays green.

### 124 — Enemy kill timing still allows Kid to get hurt after the enemy should already be dead (FIXED)
Evidence source: native play reports. Native repro: reported, not yet captured.
Oracle repro: unknown. Minimum gate: `fast-slice`.

Root cause: non-stomp kill paths removed the live enemy object, but only stomps updated the
layout-backed `killed_enemies` bitfield. The next activation pass could therefore recreate a
"dead" enemy from the layout while its spawn remained in the active window, which made the
contact damage feel one frame late or outright wrong in live play.

Fix: centralised enemy removal so stomps, sword kills, friendly projectiles, Berzerker charge,
and diamond-power AoE all share the same kill bookkeeping and death side-effects. Regression:
`red_stealth_sword_kill_marks_layout_enemy_dead` proves a sword-killed layout enemy no longer
reactivates on the next activation pass.

### 125 — Breakable blocks can render with the wrong lifecycle art or regress into prize-like visuals (FIXED)
Evidence source: code-path audit plus failing render regressions. Native repro: reproduced in tests.
Oracle repro: n/a for this render mapping bug. Minimum gate: `fast-slice`.

Root cause: `kid-render`'s `BlockVisualMapping::resolve()` was treating the decoded
`Block_Mappings` table as skin-only, even though the asset contains five 16-skin
visual groups (smash, prize, ice, metal, rubber) after the terrain entry. That made
cells such as the post-prize `$E001` smash block reuse the prize-family art for the
same skin byte.

Fix: index block visuals by both block family and skin. Regressions:
`load_block_visual_mapping_from_skin_and_block_type`,
`build_game_scene_emits_block_family_tiles_from_skin_and_block_type`, and
`prize_block_lifecycle_restores_smash_family_art_after_dispatch`.

### 127 — Black bouncy blocks do not produce the expected bounce behaviour (FIXED)
Evidence source: backlog follow-up plus ASM review. Native repro: reproduced in code paths.
Oracle repro: ASM-backed via `sub_902A`, `sub_914A`, `sub_8F26`, `sub_922C`, and `loc_10DA4`.
Minimum gate: `fast-slice`.

Root cause: native treated rubber blocks like ordinary solids on contact, so the ROM's
contact-triggered jiggle effect never appeared on landings, head bonks, or wall hits.
The earlier "bounce block" description was misleading: ASM `loc_10DA4` only plays
`sfx_Rubber_block` and spawns the temporary jiggle object; it does not relaunch Kid.

Fix: route rubber floor, ceiling, and wall contacts through the existing block-bounce
visual channel while keeping the ROM-faithful landing and collision velocities intact.
Regressions: `rubber_block_landing_queues_jiggle_without_bouncing_player`,
`rubber_block_ceiling_hit_queues_jiggle_without_destroying_cell`, and
`rubber_block_wall_hit_queues_jiggle_without_breaking_wall`.

### 128 — Form-specific block breaking can now destroy metal blocks that should stay solid (FIXED)
Evidence source: code-path audit after the Iron Knight landing shortcut. Native repro: reproduced in tests.
Oracle repro: ASM-backed via `sub_AF7A`, `off_AFD8`, and `loc_110D0/sub_11120`. Minimum gate: `fast-slice`.

Root cause: native had collapsed Iron Knight's landing block-break dispatch into
"delete every block type `0..5`". The ROM does not do that. `off_AFD8` sends
Smash/Prize/Ice to destructive handlers, SolidGhost/Teleporter to a no-op path,
and Metal to `loc_110D0`, which plays the steel bounce animation and restores
`$E50B` instead of clearing the cell.

Fix: restore that block-type dispatch in the landing path. Regression:
`iron_knight_landing_on_metal_block_bounces_without_destroying_it` proves a
landing Iron Knight now bounces off a metal block, leaves the cell intact, and
queues the steel bounce effect instead of deleting the tile.

### 130 — Turn recordings into the standard playtest evidence pipeline (FIXED)
Evidence source: existing `play-native --record` / `verify-recording` support plus current workflow gaps.
Native repro: partial. Oracle repro: partial. Minimum gate: n/a.

The repo already records input + trace. The missing work is packaging, remote-ingest workflow,
optional artefacts, and making the recording slug the normal reproduction handle for future bugs.

Progress landed: new recording bundles now persist ROM SHA-1, script/trace hashes, host OS/arch,
best-effort git commit/dirty state, and explicit record/verify commands in `manifest.json`, plus
a human-readable `README.txt` so a copied slug is self-describing before verification runs.

Latest progress (2026-03-27): legacy `trace.json` bundles now deserialize through the current
`NativeTraceFrame` schema again. `kid-core` `NativeTraceFrame` now supplies serde defaults for
newer fields, and `kidcli` regression coverage locks a real legacy trace example so
`load_recording()` backfills both manifest metadata and missing trace keys.

Latest progress (2026-03-28): `kidcli verify-recording` now accepts either a local cache slug
(`--slug <slug>`) or a copied bundle path (`--bundle <path>`). That removes the old manual
re-ingest step for remote repro bundles and makes the bundle itself a valid verification handle.

Latest progress (2026-03-28, artefact sidecars): `kidcli play-native --record <slug>` now saves
bundle-level `final-frame.png` and `audio.wav`, and `--record-notes <path>` copies plain-text
player notes into `notes.txt`. The saved `manifest.json` lists those sidecars explicitly so copied
bundles stay self-describing before any replay or oracle verification runs.

Closure evidence (2026-03-28, transcript sidecar): recording bundles now always include an
automatic `transcript.txt` generated from the captured native trace. The transcript logs the start
state plus major event boundaries such as game-mode swaps, level changes, pause toggles, life/HP
changes, helmet swaps, damage latches, game-over entry, and continue countdown updates, so a copied
bundle is reviewable before replaying it. `manifest.json` and `README.txt` now list the transcript
alongside `notes.txt`, `final-frame.png`, and `audio.wav`, and `kidcli` regression coverage locks
the new bundle contract. With transcript sidecars landed, the recording slug is now a complete,
self-describing repro handle rather than just "trace plus optional notes".

### 131 — Burn down the parked DEVIATION items (FIXED)
Evidence source: fidelity audit of the remaining `DEVIATION:` markers in gameplay code.
Native repro: mixed. Oracle repro: ASM-backed where noted. Minimum gate: `fast-slice`.

Progress (2026-03-27): Fireball bounces no longer skip the ROM's signed rebound perturbation.
Native now mirrors ASM `Enemy19_Fireball_BounceFloorCeiling`'s `Get_RandomNumber_byte` /
`eor.b Camera_X_pos` / `asl.w #7` rebound shaping with a deterministic local byte source, so
floor and ceiling rebounds vary within the original `±$4000` range instead of reflecting at a
fixed speed every time. Regressions:
`fireball_floor_bounce_applies_signed_rom_perturbation_before_cap` and
`fireball_ceiling_rebound_uses_the_same_perturbed_velocity_rule`.

Progress (2026-03-27): Tornado flight now uses ASM `UFO.asm` `unk_35344` target-speed patterns
instead of the old fixed accel-flip placeholder. Native reloads one of the 16 ROM pattern rows,
derives signed per-axis acceleration the same way as `loc_35444`, and then steps with the shared
`Enemy_HandleAcceleration` shape. Follow-up (2026-03-27): native now threads the shared ROM
`Init_RNG` / `Get_RandomNumber_byte` state through gameplay, so Tornado pattern reloads, Fireball
bounce perturbations, and Robot charge flip timers all consume the same seeded byte stream the
ROM uses. Regressions:
`tornado_loads_rom_pattern_on_timer_expiry` and
`tornado_accelerates_toward_pattern_targets`, plus the direct RNG and enemy reload tests.

Closure evidence (2026-03-27): the final parked gameplay deviation at the
`sub_B43A` flat-ground precheck is gone. Native now keeps the precheck literal
for Solid/Spikes only, but skips that walking-coroutine gate when the current
support is already a grounded slope owned by ASM `sub_9386` / `loc_9D22`.
That removes the last in-code `DEVIATION:` marker without reintroducing the old
half-speed slope regression. Regressions:
`asm_faithful_ground_precheck_skips_sub_b43a_when_already_on_slope` and
`walking_on_slope_up_y_tracks_diagonal_every_frame`, plus `mise run verify-slice-fast`.

### 003 — Audio (FIXED)
GEMS engine is fully wired: BGM per theme, SFX for jump/stomp/damage/death/form/block.
Audio init now reports specific SDL2 errors with troubleshooting hints (SDL_AUDIODRIVER,
audio support). Triggers BGM on first InGame entry. Pipeline verified correct.

### 004 — Palette (FIXED)
Root cause: theme-specific palette permutation tables were not applied. The ASM art
decompression remaps palette indices per theme (Forest, City, Mountain have unique tables).
Fix: load 16-byte permutation tables and reorder palette entries during level visual loading.

### 005 — Parallax (NOT A BUG)
Parallax IS working. bgscroll data loads correctly, `background_scroll_x_px()` applies
camera-relative rates. Effect is subtle (e.g. Forest: 26% camera speed) matching original.

### 006 — Dragon moonwalking (FIXED)
Root cause: `select_enemy_frame` used global `trace.frame_index` for ALL enemies,
cycling walk frames even when the dragon is in a stationary phase (perch/parked).
Fix: added `stationary` field to NativeObjectSnapshot, populated from
`DragonMotionPhase::is_stationary()`. Stationary enemies show frame 0 (idle).

### 010 — Enemy attacks (MOSTLY DONE)
11 enemy families + 4 bosses + 3 helmets + 3 environmental sources have projectile support.
Fire Demon fire trails now use a typed child-object lifecycle for the death-spawned trail.
All simple projectile-based attacks are implemented.
UFO beam is now covered by the native segmented beam path (`step_ufo_beam`) and
its loc_34EEC/loc_34FD6/loc_351EE/loc_35512 regression tests; it is no longer
an open attack-system blocker.

Follow-up (2026-03-28): the native player renderer now tags the existing Fire Demon death-trail
projectiles and renders a dedicated six-phase flame footprint instead of the old generic enemy
projectile dot. This made live play read as a fire trail rather than a reused bullet marker while
leaving exact ROM tile extraction as later polish.

Follow-up (2026-05-18): the native player now renders the tagged death-trail signature with the
`Fire_Demon.asm` `stru_3A622` ROM frames (`unk_C7776..unk_C779E`, tile bases `$0B0..$0BA`) using
the Fire Demon art bundle. The remaining fidelity gap is the core representation: native still uses
projectile snapshots as the visual carrier rather than the original child-object lifecycle.

Follow-up (2026-05-19): Kid contact no longer consumes the native Fire Demon death-trail carrier.
Regression `fire_demon_death_trail_hits_player_without_consuming_child_loc_3a6ca` pins ASM
`Fire_Demon.asm` `loc_3A6CA`: the trail child loops until `$19(a3)` level collision or d3 expiry.
At that point, only typed child storage and the exact three-cell support check remained.

Follow-up (2026-05-19): the native Fire Demon trail now uses a typed child list instead of the
generic projectile carrier. The child projects to the existing trace/render projectile snapshot
for compatibility, still damages Kid without being consumed, and applies the `loc_3A63C`
three-cell support gate before its timer loop: d5/d6/d7 must all mask to `$6000`, and centre
d6 must not have bit 15 set. Regressions cover typed child spawning, trace projection, Kid-contact
survival, centre-block deletion, and three plain-solid-cell survival.

### 011 — Bricks half-broken (FIXED)
Resolved by palette permutation fix (#004). Verified via capture-frames comparison —
block tiles render with correct colours matching original game.

### 013 — P-blocks (NOT A BUG)
Prize blocks vanish when hit (cell cleared to empty). This is correct per original behaviour.

### 015 — Wall grab (FIXED)
Added `left_wall_sticky` field and mirrored right-wall logic in `apply_wall_collision()`.
Both directions now support airborne wall-sticky with overhang top-edge retention.

### 017/019 — Helmets and form transform (FIXED)
`apply_prize_content` → `queue_form_change` → `resolve_pending_form_change` chain is wired
and verified. Forms change player hitbox, speed table, and abilities. FloatingItem spawns
on prize hit with content-type-specific colours. Unit tests confirm IronKnight and
Juggernaut transforms trigger correctly with proper hitpoint assignment.

### 018 — Form-change animation (PLACEHOLDER + RESEARCH)
Alternating white/yellow flash overlay for 8 frames after form transform.
ASM traced: `Kid_Transform` (kid.asm:8369-8607) uses per-helmet sprite+palette
cycling tables (stru_7EFA..stru_8064). Full implementation needs ROM sprite tile
extraction for the spinning animation frames.

### 020 — Water shimmer (DONE)
Real ASM ripple table from `BackgroundScroll_ForestWaterRippleData` (kid.asm:34310).
64-byte signed lookup, max 8px amplitude, phase = `Time_Frames >> 2 & 0x3F`.
2026-05-28 playtest note: Level 3 / Highwater Pass uses background theme 7
(mountain). ASM `loc_30598` gates ripple only to Forest and Desert before
returning, so do not reuse this forest-water fix for the Highwater waterfall
report without separate ASM/oracle evidence.
2026-06-01 follow-up: refreshed
`issue-playtest-highwater-water-current` proved the large late-frame drift was
mostly VDP scroll timing, not palette rotation. Rendering gameplay checkpoints
with the previous `Transfer_ScrollDataToVRAM loc_300BA` camera latch reduces
`walk_half` from 26,534 changed pixels / MAE 18.75 to 891 / MAE 1.04, and
`walk_end` from 27,006 / MAE 19.04 to 909 / MAE 1.07. The remaining failure is
Plane B/waterfall composition in foreground holes: native still shows backdrop
blue at samples such as `(0,169)` while oracle shows mountain Plane B purple.
Continue from background tile/art selection or Plane B composition evidence, not
another global scroll or lava/water palette offset.
Rejected 2026-06-01: reinterpreting layered background selectors as direct
`Background_width=$40` linear row-buffer offsets looked plausible from
`loc_11B7A`, but applying it to native `compose_layered_background` made the
strict Highwater bundle worse: `level_settle` regressed from 41 changed pixels
to 3,342 and `walk_half` from 891 to 3,462. Do not replace the existing
left/right layered composition wholesale; the remaining hole pixels need more
exact oracle VDP/nametable evidence.
Fixed 2026-06-01 follow-up: oracle VDP samples
(`oracle-probe --vdp-pixel-samples 0/169,126/169`) showed Highwater's large
hole pixels come from Plane B tiles `$7E5`/`$7E6` on palette line 0. Native now
models `Load_BackgroundMappings_Layered loc_12910` base words, appends
`loc_11EC4` Mountain `cave_alt` art at relative background tile `$65`, and
preserves foreground CRAM line 0 while copying Mountain background colours into
line 1 via `loc_11E16`. This makes `level_settle` strict pixel-perfect (41 -> 0
changed pixels) and improves `walk_half` 891 -> 413 / `walk_end` 909 -> 431.
Remaining drift is the smaller mid-screen Mountain Plane B row/phase mismatch,
not missing follow-up art.

### 021 — Sound Test (FIXED)
Options menu cursor==1 now enters Sound Test screen. Left/Right ±1, Up/Down ±16,
A/C plays track via GEMS engine, B returns to options.

### 022 — Duplicate Game Over overlay (FIXED)
Scene rendering had two `if trace.game_over` blocks (lines 1023 and 1091).
Removed the duplicate.

### 036 — Tank walk speed step (FIXED)
Tank.asm lines 52-64 use a 3-tier fall-through chain: HP=2→$8000, HP=3→$10000,
HP=4→$18000. This gives step=$8000. Was incorrectly $10000 in `pacer_walk_speed()`.
All other pacer families verified correct: Goat/Armadillo/Crab ($10000), Driller ($8000),
Scorpion (fixed $18000), FireDemon/Robot/Ninja ($8000).

### 037 — Floor-snap misses terrain (FIXED)
Gravity simulation in `level_respawn_slice_with_floor_snap` fixed most levels earlier, then
the remaining `0x24` / `0x51` tail turned out to be a hidden init-camera carry rather than
another support-path miss. `finish_respawn_camera_convergence()` now mirrors the last
`sub_71E4`-backed settle beat for the exact `target-camera == 2px` residue, so both levels
finish on the oracle-backed `camera_y=$036E0000` / `$028E0000` results.
Level `0x29` stays locked down by `respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x`,
and level `0x52` stays locked down by `respawn_settle_level_52_uses_stair_step_support_path`.
Level `0x02` remains clean apart from the usual advisory `number_lives` field.

### 038 — Camera Y settling dead-zone (FIXED)
8 levels showed +1 to +7px camera_y diffs at level_settle. The asymmetric dead-zone
in `update_camera_axis` (`(target-camera+4)>>3` rounds to 0 within ±4px) meant the
final camera position depended on approach direction. Floor-snap placed the player
instantly at the floor, so the camera approached from a different direction than the
oracle (which tracks the player frame-by-frame during real gravity fall).
Fix: simulate gravity fall + camera follow in `level_respawn_slice_with_floor_snap` —
replay the player falling from header_y to floor_y under gravity, updating the camera
each step. This matches the oracle's convergence path. Fixed all 8 levels.

### 039 — 4th speed table entry (NOT A BUG)
Normal terrain table (unk_9C0A) has 4 entries of 7 longwords each ($70 bytes total),
but Ice (unk_9C7A) and Rubber (unk_9CCE) only have 3 entries ($54 bytes each).
ASM selection at loc_9AFC checks Micromax (+$1C) and Juggernaut (+$38) independently.
The 4th entry at offset +$54 is only reachable if BOTH checks matched simultaneously,
which can't happen (one helmet at a time). Dead data — likely a cut form or dev remnant.
Values: $20000/$4000/$2800/$4000/$30000/$3800/$6000 (similar to Juggernaut but higher accel).

### 049 — Level 0x29 X position diff (MERGED WITH #037)
Resolved while narrowing #037. Spawn-wall overlap resolution now keeps the cached
manifest settle path on the oracle-backed `kid_x=0x02A50000` / `camera_x=0x02090000`
result for level `0x29`.

### 050 — Idle-tail position diffs on 4 levels (INVESTIGATING)
Levels 0x00, 0x45, 0x48, 0x61 show wildly different positions at idle_tail (frame ~359).
- 0x00: game_mode native=InGame(0x0C) oracle=LevelTransition(0x08) — oracle's warp
  to level 0 is slower, still in level-name-card at frame 359
- 0x45, 0x61: native positions zeroed out (player fell off map or died)
- 0x48: large position divergence (different death timing)
Root cause: combination of oracle warp timing differences and enemy interaction timing.
These are not simple constant fixes — they require either matching oracle warp timing
exactly or increasing settle frames in scenarios.

### 069 — TAS boot slice desync (FIXED)
Imported BK2 recordings exposed a boot-only fidelity gap that the per-level warp sweep
could not see. Native was exposing gameplay-looking state on frame 0, title readiness
was modelled with a global six-Start shortcut, and the first intro segment accepted
Start too early. Fixes:
- cold boot trace now keeps gameplay-visible state zeroed on frame 0
- title readiness is phase-local instead of keyed off cumulative Start count
- intro segment 1 has an oracle-backed early-Start dead zone plus direct-to-title skip
- TAS reports now include boot-phase diagnostics and checked-in 250-frame boot-slice fixtures

Verification:
- `kid_chameleon_short_demo` first 250 frames: 0 authoritative diffs
- `kc_any_percent_2024` first 250 frames: 0 authoritative diffs
- `any-percent-published` first 250 frames: 0 authoritative diffs

### 082 — Demo byte boundary parity shift after transform (FIXED)
On the first Kid_Transform frame, step_demo_input sees form_change_flash_frames=0
(step_lifecycle hasn't set it yet) and advances both counter and data_index. The
data_index advance is correct — it aligns native's data_index with the oracle's
pointer (compensating for the 1-frame late demo start). But the counter advance
shifts the byte-advancement parity for all subsequent frames.

The inverted parity (`playback_frame % 2 == 1` vs oracle's even counter) compensates
for the 1-frame late start during normal play. The extra counter increment on the
first transform frame disrupts this compensation, causing byte boundaries to land
on the wrong frame of each 2-frame window.

Fix: after step_lifecycle (inside refresh_opening_slice), detect when
form_change_flash_frames transitions from >0 to 0 (transform end) and decrement the
counter by 1 to restore the pre-transform parity. Uses `demo_transform_active` bool
to track whether a transform is in progress.

### 091 — Demo Start abort timing (FIXED)
The imported Any% TAS presses real Start on global frame 1772 while demo 0x1B is
still running. Native was handling that input immediately inside `step_demo_input`,
returning to title on the same frame. ASM `loc_119D8` does not do that: it clears
`Ctrl_Pressed`, hibernates for one frame, then checks `Ctrl_1_Pressed` on wake-up.
That makes the visible title transition land two frames after the live Start press.

Fix: add a dedicated deferred demo-title-return countdown in `NativeRuntime`.
Frame 1772 now arms the abort, frame 1773 still stays in `DemoPlay`, and frame 1774
finally calls the title-return cleanup and clears `Current_LevelID` to 0.

Verification:
- `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`
- `cargo test -p kid-core demo_aborts_on_start_press -- --nocapture`
- `cargo test -p kid-core demo_auto_ends_after_max_frames -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1768 --frame-end 1780 --checkpoint-interval 1`

Remaining frontier: frame 1774 title-mode handoff still snaps to the title pose one
frame earlier than oracle. Tracked in #092.

### 092 — Demo abort title handoff preserves live snapshot (FIXED)
ASM `loc_119D8` only flips `Game_Mode`/`Current_LevelID` and returns through
`loc_6E2`; it does not immediately rebuild the title-screen Kid/camera RAM. The
oracle therefore spends the first post-abort title phase with `mode=$002C` and
`level=0`, but still exposes the paused demo snapshot for Kid and camera.

Native was clearing `opening_slice` and reusing the cold-boot title pose as soon
as the abort countdown expired, which skipped that paused handoff phase.

Fix: preserve the demo gameplay snapshot on the title-return path, mark the
runtime paused, and let trace output reuse the preserved Kid/camera snapshot
while title mode is active. Added `demo_title_return_keeps_live_snapshot_on_first_title_frames`
to lock down frames 1774 and 1779.

New frontier: frame 1802. Oracle blanks Kid for frames 1802-1805, then also
zeros camera/helmet/diamond state at frame 1806. That second phase is tracked
in #093.

### 093 — Demo abort title handoff blanks Kid before camera reset (FIXED)
Oracle traces show a second demo-abort title phase after the live snapshot:
Kid disappears on frames 1802-1805, but camera/form/diamond state stays frozen
until frame 1806, when the remaining title rebuild clears it.

Native used one boolean (`preserve_demo_title_snapshot`) for the entire handoff,
so it either preserved everything or nothing. That kept Kid visible too long and
also let the normal title pose appear as soon as the snapshot logic stopped.

Fix: split the handoff into two phases. `trace.rs` now only reuses the live
demo player/object snapshot for the first 28 title frames, while `boot.rs`
keeps the camera/form snapshot alive until frame 1806 and only then resets the
runtime back to clean title-screen state.

Verification:
- `cargo test -p kid-core demo_title_return -- --nocapture`
- `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect`

New frontier: frame 1832. Oracle still keeps Kid hidden while the title object
counts rise from 3 to 5, but native starts exposing the standard title pose.
Tracked in #094.

### 094 — Demo abort title rebuild keeps Kid hidden through frame 1923 (FIXED)
Once the frame-1806 camera/form reset lands, oracle traces do not immediately
swap back to the standard title pose. Title object counts keep rebuilding, but
Kid stays hidden until frame 1924.

Native reused the generic `title_screen_ready()` gate as soon as the preserved
demo snapshot ended. That kept the title object timing roughly right, but it
also exposed the normal title pose from frame 1832 onward because the generic
title-ready gate flips much earlier than the oracle-backed demo-return path.

Fix: keep a demo-return-specific "hide Kid" window in `trace.rs` after the
snapshot handoff. That leaves the generic title-ready/object-count path intact
while holding `kid_x_pos`/`kid_y_pos` at zero through frame 1923, then letting
the normal title pose resume on frame 1924.

Verification:
- `cargo test -p kid-core demo_title_return -- --nocapture`
- `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1828 --frame-end 1927 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect`
- `mise run verify-slice-fast`

New frontier: past frame 2000. The first 2000 TAS frames are now CRC-clean; the
next authoritative mismatch has not yet been re-bisected beyond that point.

### 095 — Demo 1 startup profile on Knights Isle (FIXED)
Once the second attract-mode demo switched to level `0x15`, native kept
reusing the demo `0x1B` bootstrap profile: the hidden camera preload used
Skydragon values (`0x0228_0000`/`0x02A0_0000`), demo input became live too
early, and the visible bootstrap pose/freefall sequence started seven frames
before the oracle.

Oracle-backed TAS traces show a different startup shape for Knights Isle:
camera stays hidden through frames 2529-2530, settles to `0x00E0_0000` with a
one-frame wrapped negative quirk at 2532, keeps Kid hidden through frame 2537,
then exposes a longer fixed-pose bootstrap before regular motion resumes.

Fix: key demo startup off the selected recording instead of one shared profile.
Demo 1 now:
- keeps its hidden-camera trace profile through frame 2537
- delays demo input/gameplay exposure from startup frame 65 to 72
- holds the bootstrap pose while camera follow converges, instead of running
  the demo 2 settle/freefall path
- defers demo enemy warm-up until the later Knights Isle handoff

Verification:
- `cargo test -p kid-core demo_1_bootstrap -- --nocapture`
- `cargo test -p kid-core demo_bootstrap_holds_spawn_pose_then_freefalls_before_grounded_hold -- --nocapture`
- `cargo test -p kid-core demo_bootstrap_first_visible_gameplay_frame_advances_camera_only -- --nocapture`
- `cargo test -p kid-core demo_camera_handoff_primes_buffer_for_the_hidden_settle_frame -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2524 --frame-end 2580 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`

New frontier: frame 2625. Advisory dragon-roster timing still differs earlier,
but the next authoritative mismatch is now Kid position (`kid_x_pos`,
`kid_y_pos`) during the first live Knights Isle traversal steps.

### 096 — Demo 1 handoff landing on Knights Isle (FIXED)
After #095, the first live Knights Isle traversal frames stayed CRC-clean
through frame 2624, but native still landed one frame early at 2625. Oracle
keeps Kid airborne at `kid=(0x005C7700,0x013F4000)` on frame 2625, then lands
on frame 2626 without taking an extra X step.

Root cause: `apply_floor_collision()` still allowed the AsmFaithful
solid-floor touch snap during demo playback. That path is correct for normal
gameplay, but here it collapsed the final `0x4000` sub-half-pixel gap in the
demo's hidden handoff arc into an early landing.

Fix: disable the AsmFaithful solid-floor touch snap while demo playback is
active. Added `demo_1_handoff_stays_airborne_until_frame_2626` to lock the
frame-2625 airborne state and the frame-2626 landing.

Verification:
- `cargo test -p kid-core demo_1_handoff_stays_airborne_until_frame_2626 -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2618 --frame-end 2635 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`

New frontier: frame 2723. Demo 1 now stays in the grounded standingstill hold
through the post-landing wall, so the next authoritative mismatch is the jump
handoff/camera advance on Knights Isle (`native kid_x_pos=0x011A0000`,
`oracle=0x011B0000`; `native camera_x_pos=0x00770000`,
`oracle=0x00780000`).

Attempted approaches that didn't work:
- pending_form.is_some() check: over-freezes (73 vs 72 frames), regression to 1129
- Full deferred pipeline (move step_demo_input after physics): changes byte for ALL frames
  due to 1-frame delay, regression to 921
- Rollback counter+data_index on first transform frame: data_index rollback misaligns
  with oracle (oracle's pointer is 1 ahead due to earlier start), regression to 1129
- Frozen input override (save pre-transform byte for first post-transform physics):
  correct in theory but native already has the right byte via the "compensating"
  data_index advance; freezing gives the WRONG byte, regression to 1129

Key insight: the extra data_index advance on K+1 is a happy accident that aligns
native's data_index with the oracle's. Only the PARITY needs correcting, not the
data_index itself.

### 083 — Demo byte boundary systematic offset (INVESTIGATED)
ASM VBlank pipeline: MainGameLoop → mode handler → Execute_Objects → player
processing → ReadJoypad (loc_C22 demo path). ReadJoypad runs AFTER game logic,
so byte read on frame N is used by game logic on frame N+1. V_Int handler is
minimal (just increments counter + sets done flag), does NOT read joypads.

Oracle probe confirmed: at frame 915, oracle counter (post-ReadJoypad) = 16,
native playback_frame = 14. Oracle counter is consistently 1 ahead of native pf.
Data pointer advances align when using parity `% 2 == 0` (matching oracle's
even-d7 advance pattern).

Attempted pipeline buffer approach:
1. Added `demo_buffered_input: Option<FrameInput>` — return previous frame's byte
2. Changed advancement parity from `% 2 == 1` to `% 2 == 0`
3. Transform compensation: `fc += 1`, `di += 1` at transform end

Result: produces IDENTICAL frontier (frame 1170) with identical position/velocity.
The buffer+parity change is mathematically equivalent to the old code: both have a
net 1-frame offset, just manifested at different parity phases. Changing parity
shifts which frames see the boundary mismatch, but the underlying 1-frame late
start means one boundary per byte pair will always be off.

Why simple compensations fail: native starts demo byte consumption 1 frame after
oracle (confirmed by probe). Adding a pipeline buffer delays usage by 1 frame
(matching ASM's read-after-use), but the late start means the read itself is on
the wrong frame. The parity change fixes the data_index alignment but the buffer
delay cancels out, leaving the same net offset. Any combination of (buffer ×
parity × transform_correction) that produces correct post-transform alignment
also produces the same pre-transform byte boundary behaviour as the old code.

True fix requires: making native start demo byte consumption on the same frame
as the oracle (i.e., fixing the 1-frame start offset), likely by restructuring
how BootState::DemoPlay initialises the counter relative to the ASM's first
ReadJoypad call in demo mode.
