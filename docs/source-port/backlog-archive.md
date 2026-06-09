# Playtest Backlog — Evidence Archive

Append-only cold storage for backlog evidence. The live index is
[`backlog.md`](./backlog.md); this file holds the investigation prose that used
to be interleaved with the table: follow-ups, notes, rejected hypotheses,
investigation write-ups, and dated playtest logs.

Keyed by issue id — grep `#NNN` (e.g. `grep -n '#238' backlog-archive.md`) to
recover an issue's full history. Order is the original `backlog.md` document
order at split time; nothing here is re-attributed or deduplicated.

---

Live playtest note 2026-06-01: latest human playthrough contradicts several
previous `[x]` rows. Treat those as reopened until a deterministic native
recording or live `play-native` verification proves the human path is stable.
Unit tests, generated-asset renderer tests, and TAS slices are useful evidence
but do not by themselves prove `human-play-stable`.

Live playtest note 2026-06-03: fresh multi-person human playthrough of the
pushed build. New issues filed as #239-#246. Reopened as contradicted: #238
(now broader than a single transition — *every* level after the Highwater Pass
section drops the player into a falling-death, and skip/throw-level keeps
failing until the level index wraps back to the start, then it recovers — so
the spawn/ground state is not being re-established on post-Highwater loads),
#204 (Samurai/Red Stealth still cannot slide down slopes), #145 and #147
(double-tank render and wall-grab "zoom to top" both re-confirmed live).
Generally positive: intro now plays and is "pretty good" aside from the
bottom-left stray Kid sprite already tracked in #210. Audio remains "a bit
fucked in general" (#160/#216/#003). Treat the reopened rows as
`not human-play-stable` until a deterministic native recording proves the
human path.

Resolution summary 2026-06-04: the actionable playtest feedback is
comprehensively addressed. FIXED (regression-tested, oracle/byte/visual
verified): #238 (skip floor-snap + native terrain decoder), #229+#247+#246
(native level-terrain decoder fully built byte-exact — removes the
emulator-fixture dependency; the 4 unreachable bonus levels now load+render
with real floors, verified by capturing logical 5A/5D/5E/63), #242 (metal
SFX), #145 (tank turret z), #210 (intro stray sprite), #243 (Tar Monster
projectile art), #245 (death→black-screen regression), #147 (wall-grab —
regression from 708e663c, oracle-verified), #204 (Red Stealth slope-slide
MoveID-6 latch, oracle-verified). NOT A BUG / ORIGINAL_QUIRK (oracle-verified):
#240 (dragon breath facing follows ROM patrol facing), #239 (walking dragon
walks off ledges + respawns, no ROM arena clamp). PROVEN CORRECT: #241 (lives
HUD), #215 (sprite-vs-plane priority pipeline). Two wrong ASM-only hypotheses
(#147 loc_ADAC re-stick, #204 current-row edge-snap) were DISPROVEN by the
oracle before any bad fix shipped. REMAINING (real-device-blocked, not
verifiable here by listening): audio #160/#216/#003 — only the concrete
rubber→brick SFX bug (#242) was state-level fixable. New non-playtest finding
noted: Kid's L0 traversal/combat physics diverges from oracle ~frame 54-60
(separate, #211-adjacent frontier).

Proactive-verification 2026-06-04 (per CLAUDE.md exhausted-backlog protocol): full state-tier oracle sweep `sweep --levels 0-104 --tier state` = **69 passed, 0 failed, 0 errors**; all 846 diffs are known `oracle_limitations` (warp-state advisory gaps), every top diff field (camera_y/kid_y/current_level_id/game_paused/kid_x) flagged oracle-limitation — ZERO authoritative state divergences across all sweepable levels (incl. the now-decoder-loadable bonus levels). Empirically confirms no hidden actionable state bugs remain; the remaining path to a finished game is human validation of the `[?]` queue + real-device audio + the modelled-but-unwired feature long-tail (#206/#217 already largely wired).

Live playtest note 2026-06-07: latest human playthrough reports multiple
remaining human-play-stability failures and regressions. Newly tracked as
#254-#267 below, with older related rows left in place for history. Treat the
affected older "fixed" claims as not human-play-stable until these new rows are
reproduced or disproven on current native. In particular: intro/title visuals,
level-select input gating, death/respawn black screen, Kid/form slope walking,
RockTank/tank floor placement, Level 2 slime/metal-block projectile behaviour,
Highwater grow/shooter blocks, post-Level-4/5/6/7 tileset-placement alignment,
Samurai downward block break, audio slowdown/end-of-level SFX, death-wall drill
rendering, and EOL flag floor-contact activation all need fresh loop attention.

Follow-up 2026-06-08: latest human pass reconfirmed the same queue and added
more exact wording: intro ninja has black-outline artefacts; level select still
allows jump/move input and jump SFX; Level 3 Rhino priority is wrong; death can
black-hold until Start; Kid/forms can stick on slopes; tanks still look in the
floor; Level 2 Drips/slime plus metal-block contact can break a block and emit a
red left projectile; Level 4 growing blocks still shoot spikes; levels 5/6/7
tileset/placement alignment looks wrong; Samurai cannot break below blocks with
jump+sword; music slows on block hits/diamond pickup; EOL countdown SFX cuts off;
death-wall left drills are absent; and the EOL flag should activate on floor
contact. Keep these as the active human loop items even where a row already has
partial fix evidence.

Follow-up 2026-06-08 latest-pushed-build note: latest human pass adds these
specific acceptance failures. Iron Knight/Heavy Man wall-climb animations are
wrong when climbing walls; RockTank/rock enemy still appears in the floor;
Samurai held-right into a slope glitches animation, especially when starting a
little away from the slope; Samurai attack-in-air, helmet pickup voice, and
bouncy sounds may play slow/low; Level 2 well left-wall hugging can launch
Samurai/Kid upward, and the same class appears with Rhino on level 3; jumping on
RockTank or Dragon can hurt Kid/Rhino instead of resolving as the expected stomp;
and later-level helmet prize blocks may not be mapped/extracted correctly. These
are current human-play-stability checks against the latest pushed changes, not
historical closed-state evidence.

Follow-up 2026-06-09: the initial natural BLW2 route for the Level 2 well
left-wall report was rejected as a wall-cling repro because it died before the
wall-hold window after RockTank contact. A sharpened natural route is now
tracked at `tests/data/dynamic-scenarios/issue-270-blw2-natural-left-wall-right180-left-b205245.json`
and passes gameplay oracle CRC verification through frames 0..960. Use it as
the next #249/#270 seed; it proves the simple approach is currently faithful but
does not yet reproduce the upward wall launch.
Follow-up 2026-06-09 lower-corridor probe: reused the authoritative Level 01
lower-floor prefix from `.cache/recordings/issue-260-drips-cleaner-prefix-audit/scenario.json`
to avoid the stale #249 direct-probe bootstrap. Cache-local variants in
`.cache/debug/issue270-scenarios/` replaced the opening 300 frames with plain
`Left` and with `Left+B`. Both pass `verify-tas --frame-start 0 --frame-end 300
--checkpoint-interval 10 --frontier-kind gameplay --bisect` by CRC. The plain
`Left` route does not launch. The `Left+B` route enters normal
`wall_cling=left` jump gravity and also matches the ROM, so it is not evidence
for a native-only no-input wall launch. Keep #249/#270 open for a sharper
natural well/shaft route; do not patch `sub_ADB8`/`sub_AE6E` from this lower
wall family.

Follow-up 2026-06-09 latest plush-build playtest: preserve these exact
human-observed regressions as current acceptance targets before another patch
round. Intro/title: skipping intro can show the ninja/title figure as a black
silhouette before transforming, while the normal unskipped animation looks good;
the old bottom-left flashing artefact is still present. Intro/demo prize
presentation still appears to produce a diamond where the expected staged helmet
should appear, but ASM `Load_IntroSequence5` uses its own VRAM helmet animation
path (`loc_1B100`/`loc_1BCD0`/`loc_1BE88`), so do not "fix" normal prize block
lookup without a gameplay blocklist oracle repro. Level-card/level-select screen
still allows movement/jump input and jump SFX while the title card is visible
despite focused `loc_1AE2E` core tests passing; investigate the live
native-player shell/audio route. Walking/running now looks like moonwalking for
Kid/forms, and Samurai slope running is no longer flickering but still has the
same orientation/animation regression. RockTank floor placement improved, but
its death keeps normal alive animation while dying and does not leave the
expected death/body frame; existing #167 tests pass, so a fresh live kill route
is still required. Red Stealth ground sword can leave the sprite facing/holding
down after attack; verify whether the hitbox is active or only the sprite is
wrong. Drips/slime now drops full slime balls part-way down, then turns into a
blue ball instead of landing as floor slime; hitting the nearby metal block can
destroy it and emit a red left projectile while dripping continues. Level 3
Rhino still renders in front of foreground river/blocks, loses held-left when
jumping from a wall/step unless Left is pressed after take-off, takes damage
from jumping onto RockTank where another helmet did not, cannot wall-jump while
climbing, can break the block below while climbing and end up in the floor, and
side/head block breaks disappear instead of playing the smash animation. One
death path then shows only the waterfall/bridge level screen without incoming
text. Later levels still show multiplying prize/grow blocks shooting, spinning
diamonds shrinking oddly, hell-level floor/collision placement mismatch, and
later prize blocks that probably should contain helmets still yielding other
items. Treat this note as triage context; each fix still needs an ASM/oracle
repro or focused regression test before implementation.

Issue #271 triage 2026-06-08: current generic `stomp-enemy` probes do not yet
prove an expected stomp regression. Level 00 Dragon frames 137/138 reproduce
damage, but the contact is a side-graze baseline matching the existing
`GfxObjects_CollisionKid` `loc_1C76` damage classification. Level 01 RockTank
with the same route misses the Tank collision box entirely; frame 171 is the
closest pass, still separated vertically. Added named audit routes
`issue271-level00-dragon-stomp-damage-audit` and
`issue271-level01-rock-tank-stomp-route-audit` so the next loop can sharpen the
human-shaped approach route before changing shared stomp/contact logic.
Follow-up: the sharpened `issue271-level01-rock-tank-stomp-sharp-audit` route
reproduced a real one-frame-early RockTank side-damage timing mismatch at frames
223/224; native promoted a zero-width edge side contact to pending damage while
oracle waited for the following positive-overlap frame. Native now keeps
edge-only Tank side damage out of the pending queue while preserving edge-only
`kidabove` stomps, Dragon's existing landing-graze latch, and the earlier Level
01 frame-143 positive-overlap Tank damage case. The RockTank route now matches
frames 223..227; Dragon still needs a sharper current human-shaped route before
changing stomp priority.

Follow-up 2026-06-09: cache-local Level 00 Dragon single-jump route grid
(`.cache/debug/issue271-dragon-route-grid/`) did not reproduce a Dragon-specific
native regression. Short/late jumps (`s115..145`) are the same side-contact HP
loss family as the existing `loc_1C76` graze audit, while earlier/longer jumps
(`s100-h14`, `s105-h8`, `s110-*`) avoid or miss the Dragon. Settled oracle
checks for representative variants `s115-h4` (damage) and `s105-h8` (safe/miss)
both pass gameplay CRC through 220 frames, so this route family should not drive
a stomp-priority patch. Keep #271 open for a sharper human Dragon approach.


Follow-up 2026-06-06 for #004:
`issue-title-card-colour-current` is not gameplay-authoritative on current main.
The 90-frame direct LevelTransition warp recording drifts at frame 0 on native
recorded-vs-replayed trace, so the derived
`issue-title-card-colour-current_case_00000_00090` case fails but `case reduce`
blocks with `source recording replay drifted at frame 0`. Current
native-vs-oracle frame comparison still shows a title-card visual mismatch
(`start`: MAE 7.8519764, 13,158 changed pixels, bounds x=33..235/y=16..199;
dominant diffs are native black pixels where oracle has coloured title-card
pixels). Re-record this title-card colour route on current main before reopening
#004 or changing rendering.

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
Follow-up 2026-06-03: selector-picked `issue-003-title-options-audio-audit`
is native-replayable but not oracle-authoritative at the later Options
checkpoint. Fresh full verification fails at `options_entry@2760` with Kid
position native `002E0000,008C0000` versus oracle `00280000,00440000`, while
native-only replay passes and the report keeps `trace_match=true`,
`first_trace_drift=null`, and matching 2860-frame traces. The requested `0..120`
case derives as `issue-003-title-options-audio-audit_case_00000_00120`, passes
with `likely_subsystems=["none"]`, and `case reduce` reports it is already
passing. Continue #003 from the later title/options audio route or a waveform
capture, not from this clean early gameplay prefix.

Follow-up 2026-06-01: the same scripted `audio-events.txt` transcript now logs
SFX `PlaySound(id)` rows derived from `played_sound_effect_ids` and the existing
live GEMS event inference. Refreshed
`.cache/recordings/issue-003-title-options-audio-audit/audio-events.txt` records
the two title-menu navigation jingles at frames 2730 and 2740 (`PlaySound(45)`)
between the intro Ice cue and Options City cue. This gives #216 a deterministic
recording artefact for checking SFX timing on scripted block/diamond/flag routes
without requiring a live SDL capture first.

Follow-up 2026-06-02: refreshed cold-boot live evidence on current `main`.
Follow-up 2026-06-03: selector-picked `issue-003-audio-events-cold-intro` is
current and oracle-clean, but not an actionable gameplay reducer. Fresh
`verify-recording` passes with `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, no first trace drift, and matching 220-frame
recorded/replay traces; enemy comparison is skipped because the oracle captured
no enemy snapshots. The requested `0..120` case derives as
`issue-003-audio-events-cold-intro_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Use later title/menu/options audio routes or waveform captures for #003, not
this clean cold-intro gameplay prefix.
Follow-up 2026-06-03: selector-picked `issue-003-cold-boot-audio-current` is
also current and oracle-clean, but not an actionable gameplay reducer. Fresh
`verify-recording` passes with `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, no first trace drift, and matching 1200-frame
recorded/replay traces; enemy comparison is skipped because the oracle captured
no enemy snapshots. The requested `0..120` case derives as
`issue-003-cold-boot-audio-current_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Use later title/menu/options audio routes or waveform captures for #003, not
this clean cold-boot gameplay prefix.
Follow-up 2026-06-03: selector-picked `issue-003-intro-audio-current` is
current and oracle-clean, but not an actionable gameplay reducer. Fresh
`verify-recording` passes with `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, no first trace drift, and matching 420-frame
recorded/replay traces; enemy comparison is skipped because the oracle captured
no enemy snapshots. The requested `0..120` case derives as
`issue-003-intro-audio-current_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Use later title/menu/options audio routes or waveform captures for #003, not
this clean intro gameplay prefix.
Follow-up 2026-06-03: selector-picked `issue-003-intro-reset-audio-after` is
current and oracle-clean, but not an actionable gameplay reducer. Fresh
`verify-recording` passes with `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, no first trace drift, and matching 1200-frame
recorded/replay traces; enemy comparison is skipped because the oracle captured
no enemy snapshots. The requested `0..120` case derives as
`issue-003-intro-reset-audio-after_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Use later title/menu/options audio routes or waveform captures for #003, not
this clean intro-reset gameplay prefix.
Follow-up 2026-06-03: selector-picked `issue-003-title-audio-current` has a
clean selected prefix but a stale later tail. Fresh full and native-only
`verify-recording` both drift against the cached native trace at frame 3456,
with `trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`,
and matching 3600-frame recorded/replay traces. The requested `0..120` case
derives as `issue-003-title-audio-current_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Re-record this title/menu audio route before using the later tail for #003; do
not patch from the clean early gameplay prefix.
Follow-up 2026-06-03: selector-picked
`issue-003-cold-boot-audio-current-refresh` confirms the same clean cold-boot
shape. Fresh `verify-recording` passes with `trace_match=true`,
`oracle_state_match=true`, `oracle_frame_match=true`, no first trace drift, and
matching 1200-frame recorded/replay traces; enemy comparison is skipped because
the oracle captured no enemy snapshots. The requested `0..120` case derives as
`issue-003-cold-boot-audio-current-refresh_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Use later title/menu/options audio routes or waveform captures for #003, not
this clean cold-boot gameplay prefix.
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

Follow-up 2026-06-06 case-queue triage for #003/#160:
`issue-playtest-title-menu-current` is clean and non-actionable on current main.
Despite the slug, the transcript only reaches IntroSequence2 at frame 534 and
never reaches title/menu/options interaction. Full `verify-recording` passes
with `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
matching 900-frame recorded/replay traces, and no first drift. The requested
`0..120` case derives as
`issue-playtest-title-menu-current_case_00000_00120` and passes with matching
world/camera/player/motion/entities/events; `case reduce` reports it is already
passing. This route exposes no audio or gameplay mismatch. Remaining
title/menu/options audio work needs a route that actually reaches those modes,
or a real-device/output-path repro.

Follow-up 2026-06-06 case-queue triage for #003/#160:
`issue-playtest-title-menu-current-long-fixed` reaches DemoPlay at frame 3270 and
Skydragon Castle 1 at frame 3301, but is stale rather than gameplay-authoritative
on current main. Full `verify-recording` fails because oracle state diverges at
`frame_003360@3360` on `player_1_level_id` (`native 0x1B`, oracle 0), and native
recorded-vs-replayed trace first drifts at frame 3456. A focused `3300..3600`
case derives as
`issue-playtest-title-menu-current-long-fixed_case_03300_03600` and fails, but
`case reduce` blocks with `source recording replay drifted at frame 3456`.
Re-record this long title/demo route on current main before using it for
title/menu/options audio or Skydragon gameplay work.

Follow-up 2026-06-06 case-queue triage for #003/#160:
`issue-playtest-title-start-2800` is also stale on current main. The transcript
reaches IntroSequence6 at frame 2396 but never reaches DemoPlay or title/menu
interaction. Full `verify-recording` reports `oracle_state_match=true` and
`oracle_frame_match=true`, but `trace_match=false` because native
recorded-vs-replayed trace first drifts at frame 2800. A focused `2800..3400`
case derives as `title_start_2800_case_02800_03400` and fails; `case reduce`
blocks with `source recording replay drifted at frame 2800`. Re-record this
title-start route on current main before using it for #003/#160 audio work.

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

Follow-up 2026-06-02: #160 now carries the current duplicate audio frontier
evidence. `play-native --scenario` can replay deterministic scripts through the
live SDL/audio queue, and `issue-160-live-title-options-audio-diagnostics`
verifies native-only with audible intro/title/options waveform windows and no
queue skips/errors. Treat #003 as blocked on the same missing exact
human/real-device repro rather than re-running the same cold-intro and
title/options probes under this older issue number.

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

Note 2026-06-01 for #126: latest live playtest reports a Kid uphill-facing
regression on up-right movement and persistent Red Stealth/Samurai slope
vibration rather than merely the ROM-faithful "cannot start walking into a
slope from standstill" quirk documented in #152. Existing `loc_8E98`/`loc_77C2`
tests are not enough. Minimum gate: a deterministic Level 0x01 Red Stealth
slope route compared against oracle/player-state checkpoints, plus a Level
0x00 Kid uphill-facing render check.

Follow-up 2026-06-02: refreshed slope evidence on current `main`. The stale
`issue-211-level00-slope-right-probe` and
`issue-211-level00-walk-right-live-refresh` bundles drift before replay and are
not usable. New `issue-126-level00-walk-right-current` verifies native-only and
`verify-tas --scenario .cache/recordings/issue-126-level00-walk-right-current/scenario.json --frame-start 0 --frame-end 900 --checkpoint-interval 1 --frontier-kind gameplay`
is CRC-clean for all 900 frames; the reported Kid uphill-facing issue is not
reproduced on the natural Level 00 route. The direct mid-slope probe
`issue-126-level00-slope-right-current` remains non-authoritative from frame 0,
and a 2026-06-02 re-check of the natural route now shows why CRC-only wording is
not enough: the same 0..900 window is still CRC-clean, but `verify-tas` reports
55 authoritative object-count diffs, first at frame 560. Kid position,
velocity, facing, movement, and HP match there; oracle keeps a hibernating dead
DragonWalking slot (`code=0x03cd46`, `gfx_slot_index=4`, HP 0) while native has
already removed it and reports `Number_Objects/Number_GfxObjects=4/4` instead
of `5/5`. Treat the Kid uphill-facing report as unproven on Level 00 state, but
do not call the route green until the DragonWalking death-slot lifetime is
re-derived from ASM `Dragon.asm loc_3CEFC -> loc_3E956` and oracle.
matching the known artificial-start limitation. Derived Red Stealth Level 01
route `issue-126-level01-redstealth-walk-right` verifies native-only, but
oracle comparison diverges before a slope-vibration window at frame 125
(`camera_y_pos`, `kid_y_pos`, and `character_movement` differ by the first
authoritative checkpoint), then takes repeated damage and loses helmet state.
Next #126 work should fix or avoid that earlier Level 01 route divergence before
patching slope animation/movement.

Follow-up 2026-06-09: fixed the natural Level 00 Red Stealth/Samurai slope
landing miss from `issue-211-samurai-natural-slope-current-20260609`. The
actionable window was frames 520..640; the first real gameplay miss was frame
606, where native used `kid_x_pos=0x031f8600,kid_y_pos=0x00c00000` and oracle
used `0x031ef000,0x00c10000` before camera followed from the bad Y. ASM path:
`sub_902A -> loc_A99E -> loc_8BF0 -> loc_9D22 -> loc_9E94/loc_9EA4 ->
loc_9F18`. Native now gives Red Stealth's high-fraction `$4082/$6024`
SlopeUp/Solid seam the ROM row selection and post-`loc_9F18` X composition.
Regression: `level00_frame_606_red_stealth_slopeup_solid_handoff_loc_a99e`.
Oracle check: `tas-diff` frames 605..611 on the derived 520..640 scenario is
6/6 match. This fixes that gameplay landing/camera frontier only; the broader
human report about Samurai slope animation/pixels remains open until a current
visual-frame route proves the render mismatch.

Follow-up 2026-06-02: derived the selector-requested base Level 01 reduction
from native recording `issue-126-level01-walk-right-base`. The source recording
passes native replay. Case `.cache/cases/issue-126-level01-walk-right-base`
reduces `walk_right_01_case_00000_00180` to 125 continuation frames and remains
oracle-divergent: first bad frame 124, field `player.y_pos`, native `12582912`
versus oracle `12517376`; `world.game_mode`, `camera`, and `entities` still
match, so the likely subsystem is player physics/collision. Semantic evidence
shows native starting `jump_started`/`movement_0003` at the same X position while
oracle remains grounded at `movement_0002`. This is before the Red Stealth slope
window, so the next implementation pass should inspect the grounded
release/jump/floor-support ASM around `sub_902A`, `sub_B43A`, and `sub_B55C`
for this route before any sweep-only hardening or visual slope patching.

Fixed 2026-06-02: the reduced frame-124 movement miss was native doing an extra
post-horizontal flat-floor support release. ASM `loc_8E06 -> sub_B43A` scans
support before `sub_8F26`, and the no-wall/no-slope branch `loc_8E4E ->
loc_8BF0` hibernates before any second support scan. Native now defers that
release for the low-row exact-cap flat edge from the reduced case while
preserving the existing slope/recontact and Level 1 death-respawn edge
guardrails. Regression:
`level01_frame_124_flat_walk_edge_defers_lost_floor_until_next_wake_loc_8e4e`;
guardrail:
`death_respawn_level_1_releases_ground_once_post_step_support_is_gone`.
Focused oracle evidence:
`verify-tas --scenario .cache/recordings/issue-126-level01-walk-right-base/scenario.json --frame-start 0 --frame-end 180 --checkpoint-interval 1 --frontier-kind gameplay`
is CRC-clean through all 180 frames; the next strict state miss is frame 138
object/contact state (`number_objects` first, then Kid damage at frame 144),
not the earlier player Y/camera movement drop.

Follow-up 2026-06-03 selector replay for #126: direct mid-slope recording
`issue-126-level00-slope-right-current` is current on native but still blocked
as gameplay evidence. Full `verify-recording` fails immediately because
checkpoint `slope_probe_start@0` reaches oracle with different character
movement (`native Some(6)`, `oracle Some(3)`), while native-only replay passes.
The verifier report has `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, no trace drift, and matching 121-frame
recorded/replay traces. The requested `0..120` case derives as
`level_00_slope_right_probe_case_00000_00120`, but `case reduce` refuses with
`settled recording oracle bootstrap reached gameplay with a different initial
state on Level 00 (player fields=x_pos|x_velocity|y_velocity|movement_state)`.
Fix settled-recording oracle bootstrap, or use the natural Level 00 route,
before treating this direct slope probe as authoritative slope gameplay.

Follow-up 2026-06-03 selector replay for #126 natural route:
`issue-126-level00-walk-right-current` is not a current full-route authority.
Fresh full and native-only `verify-recording` both drift at native replay frame
192, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 961-frame recorded/replay traces. The
requested `0..120` prefix derives as `walk_right_00_case_00000_00120` and
passes with matching `world.game_mode`, `camera`, `player.position`,
`entities`, and `events`; `case reduce` therefore reports "already passing".
Do not patch #126 slope behaviour from this clean prefix, and do not call the
long natural route green until it is re-recorded or the frame-192 replay drift is
explained.

Follow-up 2026-06-03 selector replay for #126 Red Stealth route:
`issue-126-level01-redstealth-walk-right` is stale on current native. Fresh full
and native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1461-frame recorded/replay traces. The requested `0..120` case derives
as `issue_126_level01_redstealth_walk_right_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record the
Level 01 Red Stealth route on current native before using it for slope
vibration or movement evidence.

Note 2026-06-01 for #133: latest live playtest still describes Options as fake
and menu visuals as jumbled; this row was too broad to close from the password
investigation alone. Split if needed after capturing whether the remaining bug
is options behaviour, title menu rendering, or both.

Follow-up 2026-06-03 selector replay for #135: historical background
validation remains closed, but recording `issue-135-level1-background` is stale
on current native and cannot seed a new reduced gameplay case. Fresh full and
native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 900-frame recorded/replay traces. The requested `0..120` case derives
as `issue-135-level1-background_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record a current
Level 1 background route before using this bundle for new renderer work.


Follow-up 2026-06-03 selector replay for #136: historical closure remains based
on the committed asset-backed tests, but recording `issue-136-prize-emergence`
is stale on current native. Fresh full and native-only `verify-recording` both
drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 361-frame
recorded/replay traces. The requested `0..120` case derives as
`prize_block_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record a current
prize-emergence route before using this bundle for new block/prize work.


Follow-up 2026-06-03 selector replay for #137: recording
`issue-137-slope-current` is stale on current native and cannot reopen the fixed
backwards-slope bug. Fresh full and native-only `verify-recording` both drift at
native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 781-frame recorded/replay traces. The
requested `0..120` case derives as `walk_right_00_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record a current slope route before using this bundle for new #137 or #126
slope evidence.


Follow-up 2026-06-03 selector replay for #138: historical closure remains
test-backed, but recording `issue-138-enemy-contact` is stale on current native.
Fresh full and native-only `verify-recording` both drift at native replay frame
0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1800-frame recorded/replay traces. The
requested `0..120` case derives as `issue-138-enemy-contact_case_00000_00120`,
but `case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record a current enemy-contact route before using this bundle for new combat
or invulnerability work.


Follow-up 2026-06-03 selector replay for #139: historical closure remains
renderer-evidence-backed, but recording `issue-139-rubber-block-approach` is
stale on current native. Fresh full and native-only `verify-recording` both
drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 821-frame
recorded/replay traces. The requested `0..120` case derives as
`walk_right_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record a current
rubber-block route before using this bundle for new #139/#127 evidence.

Follow-up 2026-06-03 selector replay for #139 sibling:
`issue-139-rubber-block-walk` is stale on current native too. Fresh full and
native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 2261-frame recorded/replay traces. The requested `0..120` case derives
as `walk_right_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record a current
rubber-block walk route before using this bundle for new #139/#127 evidence.

Follow-up 2026-06-03 selector replay for #139 historical window:
`issue-139-rubber-block-window` is stale on current native as well. Fresh full
and native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 701-frame recorded/replay traces. The requested `0..120` case derives
as `walk_right_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Keep #139 closed from
the renderer evidence above, not this now-stale saved window bundle.


Follow-up 2026-06-03 selector replay for #140: historical closure remains
renderer-evidence-backed, but recording `issue-140-prize-block-window` is stale
on current native. Fresh full and native-only `verify-recording` both drift at
native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 361-frame recorded/replay traces. The
requested `0..120` case derives as `prize_block_00_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Keep #140 closed from renderer evidence, not this now-stale saved window bundle.


Note 2026-06-01 for #141: latest live playtest still reports the "Everybody
played it" kids/door composition broken, a stray Kid/jacket sprite in the
bottom-left before "too real", and the green/silhouette transform screen.
Previous loader split evidence is branch-level, not human-play-stable. Capture
the full boot attract loop as frames and compare scene objects/layers against
oracle before patching.

Note 2026-06-02 for #141: fresh cold-boot visual repros were recorded at
`.cache/recordings/issue-141-current-cold-intro/` (2700 frames) and
`.cache/recordings/issue-141-current-cold-intro-2760/` (2760 frames). Both
pass native trace replay and oracle state comparison; the 2760-frame bundle
also proves the regular `frame_002700` checkpoint is pixel-perfect against the
oracle. The only red frame in that focused bundle is the synthetic `end@2759`
checkpoint (`MAE 1.838737`, 2,387 changed pixels, bounds x=51..262 y=154..192),
which is weaker evidence than a normal frame checkpoint and likely needs a
recording comparator/end-frame alignment fix before it should drive renderer
changes. ASM checked for the current deterministic route:
`Load_IntroSequence2` (`kid.asm:29255-29300`), `Load_IntroSequence4`
(`kid.asm:29303-29321`), `Load_IntroSequence5` (`kid.asm:29324-29364`), and
`Load_IntroSequence6` (`kid.asm:29366-29422`). Proposed next fix is not
another loader guess: first record a deterministic route that includes the live
Start/menu interruption and reported "Everybody played it" corruption, then
compare normal checkpoint frames against oracle. If only `end` samples stay
red, fix the `verify-recording` final-frame sampling contract instead of
changing intro VDP composition.

Follow-up 2026-06-03 selector replay for #141: selector-picked
`issue-141-current-cold-intro` does not provide a reduced gameplay failure in
the requested `0..120` prefix. Native-only replay passes; the refreshed report
shows `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
no trace drift, and matching 2700-frame recorded/replay traces. The requested
case derives as `issue-141-current-cold-intro_case_00000_00120` and passes with
matching `world.game_mode`, `camera`, `player.position`, `player.motion`,
`entities`, and `events`; `case reduce` reports "already passing". Continue
#141 from the existing later intro/end-frame caveat or a fresh Start/menu live
route, not this clean cold-boot prefix.

Follow-up 2026-06-03 selector replay for #141 sibling:
`issue-141-current-cold-intro-2760` also has a clean `0..120` prefix.
Native-only replay passes; the refreshed report shows `trace_match=true`,
`oracle_state_match=true`, `oracle_frame_match=true`, no trace drift, and
matching 2760-frame recorded/replay traces. The requested case derives as
`issue-141-current-cold-intro-2760_case_00000_00120` and passes with matching
`world.game_mode`, `camera`, `player.position`, `player.motion`, `entities`,
and `events`; `case reduce` reports "already passing". Continue #141 from the
known later strict-pixel/end-frame caveat or a fresh Start/menu live route, not
this clean cold-boot prefix.

Follow-up 2026-06-03 selector replay for #143: selector-picked
`issue-143-heavy-man-block-break` is stale on current native and remains a
negative baseline, not block-break proof. Fresh full and native-only
`verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1861-frame recorded/replay traces. The requested `0..120` case derives
as `stomp_enemy_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record a current
Iron Knight/Heavy Man block-break route before using this bundle for new #143
evidence.

Follow-up 2026-06-03 selector replay for #143 positive route:
`issue-143-level27-ik-smash-probe-5` is also stale on current native. Fresh full
and native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1200-frame recorded/replay traces. The requested `0..120` case derives
as `issue_143_level27_ik_smash_probe_5_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Keep #143
closed from the committed block-break tests and re-record this Level 27 route
before using it as fresh block-break evidence.

Follow-up 2026-06-03 selector replay for #143 published route:
`issue-143-published-route-probe` is stale on current native despite the global
`any-percent-published` frontier being clean through frame 37759. Fresh full and
native-only `verify-recording` both drift at native replay frame 9, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 37759-frame recorded/replay traces. The requested `0..120` case derives
as `any-percent-published_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 9. Do not use this saved
published-route probe as #143 evidence until it is re-recorded or reconciled
with the current TAS scenario.

Follow-up 2026-06-03 selector replay for #143 TAS route: `issue-143-tas-route`
is stale on current native. Fresh full and native-only `verify-recording` both
drift at native replay frame 9, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 4916-frame
recorded/replay traces. The requested `0..120` case derives as
`kc_any_percent_2024_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 9. Do not use this saved
TAS route as #143 evidence until it is re-recorded or replaced by the current
authoritative TAS scenario.

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

Follow-up 2026-06-02 focused verification for #144: current `main` still
satisfies the ASM-backed shooting-tank coverage. `cargo test -p kid-core
tank_shooting -- --nocapture` passes 18 tests, including
`tank_shooting_fire_uses_loc_3c50e_cadence_and_loc_3b730_offsets`,
`tank_shooting_spawn_wait_timer_matches_first_loc_3c50e_fire_gate`, and
`blue_lake_woods_2_live_rock_tank_shooting_spawns_tank_bullet_loc_3b730`.
`cargo test -p kid-core
shared_enemy_robot_helper_sub_3c4f8_spawns_loc_3b730_for_the_firing_rock_tank_parent
-- --nocapture` also passes. ASM rechecked: `Enemy0B_RockTank_shooting_Init`
seeds the turret child, selects `stru_3B4D8` only for exact HP 4 and
`stru_3B4F6` otherwise (`Tank_Shooting.asm:loc_3B530..loc_3B690`), then
`loc_3C50E`/`loc_3B730` spawns the bullet from the turret child. Existing
recordings `debug-level01-tank-walk`, `debug-level01-tank-walk-1200`, and
`debug-rocktank-level2-oracle-settle-current` were recorded at older commits
and should not drive a new fix without fresh replay/oracle verification. The
next action is a deterministic current Level 01 route around the first `$0B`
shooter window, checking turret frame, bullet spawn frame, projectile kind, and
oracle frame/state parity. If that route is green, split the remaining playtest
report to render/audio rather than reopening #144 cadence.

Follow-up 2026-06-02 focused verification for #145: current stateless renderer
coverage still passes: `cargo test -p kidcli rock_tank_renderer --
--nocapture`, `cargo test -p kidcli rock_tank_shooting -- --nocapture`, and
`cargo test -p kidcli rock_tank_child_turret -- --nocapture`. These cover the
body/turret split, traced child snapshot usage, `stru_3B4D8` muzzle-open turret
frame, and child damage-flash palette inheritance. The live generated-asset
test `blue_lake_woods_2_live_tank_bullet_renders_rom_child_sprite_loc_3b730`
is still ignored in this harness (`requires generated .cache assets`), so it is
not current proof for the playtest's misaligned firing animation report. ASM
rechecked: `Enemy0B_RockTank_shooting_Init` creates the turret child at
`$3A(a5)`, copies body X/Y, subtracts 8px from child Y, selects `stru_3B4D8` or
`stru_3B4F6`, and `stru_3B526` supplies the two bullet frames. Next action:
record a fresh deterministic Level 01 `$0B` shooting-tank window with generated
assets, then compare turret child frame/position and bullet sprite frames
against oracle PNGs before touching renderer code.

Follow-up 2026-06-04 current-main retest for #145: fresh native capture
`capture-frames --level 01 --frames 300 --output
.cache/debug/issue-145-current-capture` renders the first Level 01 RockTank as a
compact grey tank, not the old two-stacked turret crop saved in
`.cache/debug/issue-145-tank-2026-06-03/native_tight_crop.png`. The assembled
debug frame `dump-enemy-sheet --level 01 --enemy 0A --frame 20,1e,14` also shows
the compact ROM turret shape. `cargo test -q -p kidcli rock_tank_renderer --
--nocapture` passes 3/3. Treat new tank-stack reports as human retest/current
build validation before changing renderer geometry again.

Note 2026-06-01 for #147: latest live playtest reports Iron Knight/Heavy Man
and Berzerker/Rhino can still climb or teleport up walls when jump is pressed
against block sides. Existing Kid-focused `loc_ABA4` tests do not cover all form
entry points. Next work should trace ASM wall-sticky eligibility by form and
record the exact form/wall/tile route that still rises.

Note 2026-06-02 for #147: ASM recheck shows the obvious pull-up path is already
properly gated: `kid_flip -> loc_ABA4` only runs when `Current_Helmet == The_Kid`
(`kid.asm:13408-13422`), while Micromax/Skycutter/Cyclone/Maniaxe branch to
their separate ability paths and other helmets fall through to `loc_AD2C`.
Native mirrors that with `if form != PlayerForm::TheKid { return false; }` in
`try_snap_wall_sticky_ledge`. Focused verification passes:
`cargo test -p kid-core loc_aba4 -- --nocapture` (4 tests, including
`loc_aba4_pullup_is_the_kid_only`) and `cargo test -p kid-core loc_abf0 --
--nocapture` (7 tests). No current recording named for #147 or a wall/ledge
form route exists in `.cache/recordings/`. Proposed next fix target is not the
Kid `loc_ABA4` helper; first record the reported Iron Knight/Heavy Man or
Berzerker/Rhino wall-side jump route, then inspect whether the rise comes from
Iron Knight climb attach/release (`loc_7664`/`loc_A30A`/`loc_A9AA`), Berzerker
wall contact/charge cleanup, or another wall-sticky latch.

Note 2026-06-03 for #147: narrowed the shared root cause to the
floor-collision-during-cling re-stick at `crates/kid-core/src/movement.rs:3497-3512`.
ASM cross-check confirms the *form selection* is faithful: `loc_AD7C`/`loc_AD98`/
`loc_ADAC` (kid.asm:13581-13599) re-enter the cling loop `loc_AA22` with
`collision_type=4` exactly when `Current_Helmet > 2` (Red Stealth=3, Iron
Knight=6, Berzerker=7 — the three reopened forms), which native mirrors with
`form.helmet_id() > 2`. The single-cell overhang release
`try_release_wall_cling_overhang` (`movement.rs:4171`, ASM `loc_AB22`/`sub_AE6E`)
is one-shot and only fires when the wall actually ends, so it is NOT the climb.
The multi-cell "zoom to top" therefore comes from the form RETAINING its jump's
upward velocity each frame while the re-stick keeps `on_ground=false` and re-arms
sticky, so it rises through the stack instead of landing. REMAINING WORK (per the
hard "probe before changing movement code" rule, deferred twice already): add a
non-Kid (Red Stealth) wall-side-jump probe scenario, trace `y_pos`/`y_velocity`
per frame vs oracle to confirm the retained-velocity mechanism and the ROM's
cling-loop vertical handling at `loc_AA22`, THEN gate the re-stick so it does not
keep an ascending form airborne through solid cells. Not fixed this session —
movement physics must not be changed on a hypothesis.

Follow-up 2026-06-04 current-main deterministic route for #147: generated
`.cache/recordings/issue-147-wall-side-jump-current` from the durable
`issue147-wall-side-jump-audit` scenario and verified it. Result:
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; the
remaining checkpoint diffs are pixel-only palette/background/player sprite
colour differences, not movement. This confirms the Red Stealth wall-side jump
route is currently oracle-state-correct. Do not reintroduce any
`helmet_id()>2` wall-cling re-stick patch without a new state-divergent
recording, preferably for the reported Berzerker/Rhino path rather than the
already-green Red Stealth route.

2026-05-24 follow-up: the core wobble was still live, but native rendering skipped the active prize-block worker after ASM `loc_10200` wrote `$6Fxx` because `push_block_grid_overrides` ignored cells with the block flag clear before checking `block_bounces`. Commit `044106f8` makes active `block_bounces` entries draw even for deleted-solid cells and adds `build_scene_draws_prize_block_wobble_even_after_deleted_cell_write`; `cargo test -p kidcli build_scene_draws_prize_block_wobble_even_after_deleted_cell_write -- --nocapture` and `mise run verify-slice-fast` pass.

Note 2026-06-01 for #155: latest live playtest still reports the drip appearing
in midair, failing to land on the floor, and leaving an invisible/blocking
collision near metal blocks. Existing Drips crawler path tests did not cover
the observed pit route; capture Level 0x01 under-tunnel Drips state with parent,
drop, crawler, collision cell, and render position fields before another patch.

Note 2026-06-02 for #155: fresh Drips investigation found earlier Level 01
blockers before any authoritative slime/drop contact. Direct start/camera Drips
warps are rejected evidence because native and oracle bootstrap to different
camera rows; raw Level 01 recordings still hit object/gfx count drift at frame
60. A semantic Level 01 walk route first exposed the playtest-reported
RockTank/Kid first-hit delay at frame 144 (`player.hitpoints` native 2 vs
oracle 1). Native now uses ASM `Tank.asm loc_3B31E` /
`Tank_Shooting.asm loc_3B5A6` `$48=$0B`, `$4A=$20` collision extents and edge
contact for `$0A/$0B`, with regression
`rock_tank_contact_uses_rom_width_at_oracle_frame_143_loc_3b31e`. Refreshed
case `.cache/cases/codex-rocktank-contact-level01-walk-right-edge-0000-0600`
now clears the frame-144 HP drift and first fails later at frame 181
`player.x_pos`; reduce that movement/collision drift before treating the later
Drips/metal-block playtest observations as oracle-authoritative.

Follow-up 2026-06-02 for #155/#211: reduced the frame-181 Level 01 movement
blocker to the same ASM path as the Level 00 slope-edge fixes:
`sub_902A -> loc_A99E -> loc_8BF0 -> loc_9D22 -> loc_9F18`. The real route
hits the `405C/605D` SlopeUp/Solid seam at post-vertical fraction `$0800`;
unlike the Level 00 `4082/6024` seam, the ROM applies only the low-word handoff
and does not borrow the extra high pixel. Native now keys that high-borrow
piece on the actual seam shape. Regressions:
`level01_frame_181_slope_up_recontact_keeps_boundary_handoff_loc_a99e`,
`level00_frame_478_slope_up_recontact_preserves_oracle_landing_x_loc_a99e`,
and `level00_frame_601_slope_up_recontact_uses_solid_edge_handoff_loc_a99e`.
Fresh case `.cache/cases/codex-level01-slope181-walk-right-0000-0600` now
matches `player.position` through frame 181 and first fails at frame 255 on
`events` (`native=[]`, oracle=`["spawned_entity"]`), so continue Drips/Level 01
triage from that later gameplay handoff/state-machine blocker.

Follow-up 2026-06-02 for #155: the frame-255/258 handoff was already fixed in
native, but enemy-frontier verification was still blocked by comparison-layer
noise: raw `Number_Objects` drift, Tank body/turret slot-order differences, and
the same stale zero-HP Tank gfx proxy outside ASM `loc_368A8`'s loader window
that case semantics already ignored. `codex-level01-walk-right-frontier-3600`
now passes `verify-tas --frontier-kind enemy --bisect` through frame 3660. Use
that as the clean prefix for the next Drips/metal-block reduction.

Follow-up 2026-06-02 for #155/#213: the rightward upward-jump Drips probe then
hit two more non-Drips blockers. First, attached Drips parent/controller contact
is now suppressed per `Drips.asm loc_33424`/`loc_33488`, leaving only the
falling child and surface crawler hostile. Second, the later RockTank contact at
frame 269 was valid oracle damage, not a Tank false positive: native needed to
restore Kid's frame-entry pose when fatal pending contact enters Death
`loc_B7FA`, and trace must preserve the pre-death object x-velocity through
Death `loc_B80C` and hidden `Dying` while the visible shell uses local d0/d1
movement. Cache-local scenario
`.cache/scenarios/issue-213-drips-upjump-right-probe-cache.json` now matches
oracle for frames 0..360. Extend that route past frame 360 to reach the reported
upward-jump/metal-block Drips interaction.

Follow-up 2026-06-02 for #155/#213: a cleaner lower-floor jump probe
`.cache/debug/issue213-jump-sweep/issue213-js-x544-s350-rb.json` reaches the
reported upward block-hit window without early HP loss. It exposed another
RockTank blocker before Drips: native originally damaged layout #09 on Kid's
stomp frame, but ASM `Tank.asm loc_3B464` / `Tank_Shooting.asm loc_3B6C0`
consume `$2C` from the linked mouth/turret child on the Tank object's pass.
Native now queues Tank/TankShooting stomp hits through the child attachment path
and keeps the nonfatal hurt hibernate armed for the next enemy step. The route
now matches frames 366..373; first remaining authoritative divergence is frame
374 on RockTank child/recoil roster (`native child y=$0248`, oracle child
y=$024A`). Continue from that Tank recoil/turret pose before promoting the
route as durable Drips evidence.

Follow-up 2026-06-06 for #155/#213: selector triage rechecked
`issue213-sweep-x552-s380-rc`, a settled Level 01 route from Kid `(552,576)` with
a damage latch at frame 388. It is stale on current main: full
`verify-recording` fails with native recorded-vs-replayed trace drift at frame 0
and oracle state divergence at `pre_jump@379` on object counts (`native 7 / 7`,
`oracle 5 / 8`). The full-window `0..800` case derives as
`issue213-sweep-x552-s380-rc_case_00000_00800` and fails, but `case reduce`
blocks with `source recording replay drifted at frame 0`. Re-record this
Level 01 sweep route on current main before using it for #213 damage, Drips, or
RockTank recoil work.

Follow-up 2026-06-07 for #155/#213: narrowed the checked-in
`issue213-drips-delayed-b-drop-audit` route to a healthy Drips prefix instead
of carrying the old death-windup tail. Debug-field probe frames 532..538 showed
the first zero-HP frame is 537, with `kid_gameplay_path_probe post=projectile_damage`
from a hostile `DripsSurfaceCrawler@0x01c90000,0x02400000`, matching the
intended ASM `Drips.asm loc_335D8`/`loc_33650` hostile-crawler lineage rather
than a RockTank blocker. The scenario now stops at frame 536 and checkpoints
`pre_crawler_contact_tail@535`: native summary remains live (`lives/hp 3/1`,
`death_windup 0`) and still has Drips drops active through frame 535.
Verification: focused route unit test, native probe, recording bundle
`.cache/recordings/issue-213-drips-delayed-b-drop-healthy-prefix-535-20260607`,
native-only `verify-recording`, and enemy-frontier `tas-diff --scenario
.cache/recordings/issue-213-drips-delayed-b-drop-healthy-prefix-535-20260607/scenario.json
--frame-start 0 --frame-end 535 --frontier-kind enemy --only-divergent` reports
535 match / 0 diverge. Full `verify-recording` is still too strict for this
lane because early object-count exports diverge at `delayed_jump_press@60`;
use enemy-frontier diff for the next route extension.

Follow-up 2026-06-07 for #238: the oracle-settle trace requested above is already exposed by `oracle-probe --capture-settle-trace`, and fresh direct warped probes no longer reproduce the stale `0x0280` vs `0x02e8` camera blocker. Captures under `.cache/debug/settle-level03/` and `.cache/debug/settle-level04/` used `warp_to_level` + `settle_oracle_warp=true`; oracle hidden `target_live` rows and frame-0 manifest state match native `inspect-respawn-settle` for both normal post-Highwater targets: Level 03 lands at Kid `0x00200000,0x007F0000`, camera `0x00000000,0x00000000`, and Level 04 lands at Kid `0x00200000,0x006F0000`, camera `0x00000000,0x00000000`. Treat the old direct-warp camera note as stale verifier debt, not an engine patch target. The next #238 action is a fresh current natural Level 02 -> 03 flag-transition recording and oracle comparison; do not change settled-warp camera convergence from the stale `0x02e8` evidence alone.

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
Follow-up 2026-06-02: selector-picked
`codex-highwater-pass2-juggernaut-flag-frontier` is not a new #238 gameplay
failure. Fresh `verify-recording` and native-only replay drift later at frame
308 with native trace mismatch, so do not use the full 2400-frame wrapper past
that point. The requested `0..120` prefix derives as
`highwater_pass2_juggernaut_flag_transition_audit_03_case_00000_00120` and
passes with `world.game_mode`, camera, player position, entities, and events all
matching; `case reduce` reports it is already passing. Continue #238 only from
a longer current route or a reduced failing case beyond the stale wrapper.
Follow-up 2026-06-03: selector-picked `codex-level04-transition-current` is
native-current but still not a reduced gameplay failure. Full verification
reports `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=false`, no trace drift, matching 2401-frame traces, and a
pixel checkpoint divergence after enemy comparison is skipped because oracle
captures no enemy snapshots. Native-only replay passes. The requested 0..120
case derives as
`highwater_pass2_juggernaut_flag_transition_audit_03_case_00000_00120`, passes
with `likely_subsystems=["none"]`, and `case reduce` reports it is already
passing. Do not patch Level 03->04 transition, carried Juggernaut state, or
Highwater Pass 2 gameplay from this clean early prefix; use a longer reduced
failing case for the remaining pixel-only checkpoint drift.
Follow-up 2026-06-06 for #238: delegated case-queue route
`issue-238-highwater-flag-transition` is stale on current native. Fresh
native-only verification drifts at frame 299 with matching 2401-frame
recorded/replay lengths and no frame diffs. The recording transcript still shows
the old route reaching Level 02->03 transition at frame 654 and returning to
`InGame` at frame 1101, but the requested `0..120` case is blocked before oracle
comparison because manifest camera x=`$02F00000` disagrees with replay frame 0
camera x=`$02800000`; `case reduce` refuses with the same
recording-verification blocker. Re-record this Highwater flag route on current
native before using it as #238 gameplay evidence.
Follow-up 2026-06-06 for #238: delegated case-queue route
`issue-238-highwater-pass2-flag-transition-probe` is also stale. Fresh
native-only verification drifts at frame 308 with matching 2400-frame
recorded/replay lengths and no frame diffs. The transcript shows the old Level
03->04 transition at frame 657 and return to `InGame` at frame 1101, but the
requested `0..120` case is blocked before oracle comparison because manifest
camera=`$05700000,$03800000` disagrees with replay frame 0
camera=`$05000000,$037E0000`; `case reduce` refuses with the same
recording-verification blocker. Re-record a current Pass 2 flag route before
using this as #238 gameplay evidence.
Follow-up 2026-06-06 for #238: delegated case-queue route
`issue-238-highwater-pass2-juggernaut-flag-transition-audit` is stale too. Fresh
native-only verification drifts at frame 308 with matching 2401-frame
recorded/replay lengths and no frame diffs. The transcript carries Juggernaut
(`helmet=5`, HP=3), reaches Level 03->04 transition at frame 662, and returns to
`InGame` at frame 1101, but the derived `0..120` case is blocked before oracle
comparison because manifest camera y=`$03800000` disagrees with replay frame 0
camera y=`$037E0000`; `case reduce` refuses with the same blocker. Re-record the
carried-Juggernaut Pass 2 flag route on current native before using it as #238
evidence.
Follow-up 2026-06-06 for #238: delegated case-queue route
`issue-238-highwater-pass2-juggernaut-flag-transition-clamped` now has the same
current blocker despite older notes calling it clean. Fresh native-only
verification drifts at frame 308 with matching 2400-frame recorded/replay
lengths and no frame diffs. The derived `0..120` case is blocked before oracle
comparison because manifest camera y=`$03800000` disagrees with replay frame 0
camera y=`$037E0000`; `case reduce` refuses with the same blocker. Treat the
cached clamped route as stale until re-recorded on current native.
Follow-up 2026-06-06 for #238: delegated case-queue route
`issue-238-highwater-pass2-juggernaut-flag-transition-probe` is stale as well.
Fresh native-only verification drifts at frame 308 with matching 2400-frame
recorded/replay lengths and no frame diffs. The transcript carries Juggernaut
and reaches Level 03->04 transition at frame 657, but the derived `0..120` case
is blocked before oracle comparison because manifest camera=`$05700000,$03800000`
disagrees with replay frame 0 camera=`$05000000,$037E0000`; `case reduce`
refuses with the same blocker. The cached #238 Pass 2 flag-transition family is
now exhausted as recording-verification evidence until re-recorded.
Fresh Pass 1 review evidence 2026-06-06: re-recorded
`issue-238-highwater-flag-transition-current-20260606` on current main. It starts
at Highwater Pass 1's real flag, transitions Level 02 -> 03 at frame 648,
returns to `InGame` at frame 1101, and survives to frame 2399 in Highwater Pass
2. Unlike the stale older bundle, fresh verification reports
`trace_match=true`, `oracle_state_match=true`, and `oracle_frame_match=true`.
`mise run human-validation-review` now includes this route for human re-test.
This supersedes only the Pass 1 flag evidence; the Pass 2/Juggernaut route still
needs a fresh current recording before it can close #238.
Fresh Pass 1 recheck 2026-06-07: re-recorded
`issue-238-highwater-flag-transition-current-20260607` after retiring the stale
direct-warp camera lead. Native-only replay is deterministic, full
`verify-recording` reports `trace_match=true`, `oracle_state_match=true`,
`first_trace_drift=null`, and matching 2401-frame traces. The route starts at
the real Level 02 flag, returns to live Level 03 at frame 1101, and survives to
frame 2399; the only full-verification failure is `oracle_frame_match=false`
from broad pixel drift, while early camera deltas (`$0280` vs `$02E8`) are
classified out of the state gate for this settled-recording route. Treat this as
current gameplay-transition evidence, not pixel-parity closure.
Fresh Pass 2/Juggernaut review evidence 2026-06-06: re-recorded
`issue-238-highwater-pass2-juggernaut-flag-transition-current-20260606` on
current main. Native-only replay is deterministic; full verification reports
`trace_match=true`, `oracle_state_match=true`, and no checkpoint state deltas.
The route starts at Highwater Pass 2's real flag as helmet 5, transitions Level
03 -> 04 at frame 656, returns to `InGame` at frame 1101, and survives to frame
2399 in Under Skull Mountain 1. `oracle_frame_match=false` remains from visual
pixel drift at the checkpoints, so treat this as gameplay-transition evidence,
not pixel-parity closure.
Fresh Pass 2/Juggernaut recheck 2026-06-07:
`issue-238-highwater-pass2-juggernaut-flag-transition-current-20260607` is
native-replay deterministic and full `verify-recording` reports
`trace_match=true`, `oracle_state_match=true`, `first_trace_drift=null`, and
matching 2401-frame traces. The route still carries helmet 5 from the real Level
03 flag, returns to live Level 04 at frame 1101, and survives to frame 2399.
`oracle_frame_match=false` remains from broad visual drift; no gameplay-state
transition failure is reproduced.
Follow-up 2026-06-06 for #238/#229: delegated case-queue route
`issue-level03-instant-death-current` is not yet gameplay-authoritative. Fresh
native-only replay passes and full `verify-recording` reports
`trace_match=true`, `oracle_state_match=true`, and matching 240-frame traces,
but `oracle_frame_match=false` from Level 03 sky/background pixel differences.
The derived `0..120` case fails before gameplay comparison with
`likely_subsystems=["recording verification"]`: settled recording oracle
bootstrap reaches Level 03 with different initial state
(`player.x_velocity|y_velocity`, entities native=1 oracle=0). `case reduce`
refuses with that blocker. Fix settled-recording oracle bootstrap before using
this direct Level 03 instant-death route as #238/#229 gameplay evidence.
Follow-up 2026-06-06 for #238/#229: delegated case-queue route
`issue-playtest-level03-idle-load` hits the same direct Level 03 bootstrap
blocker. Fresh native-only replay passes and full `verify-recording` reports
`trace_match=true`, `oracle_state_match=true`, and matching 301-frame traces,
but `oracle_frame_match=false` from Level 03 sky/background pixel differences.
The derived `0..120` case `idle_level_03_case_00000_00120` is blocked with
`likely_subsystems=["recording verification"]`: settled recording oracle
bootstrap reaches Level 03 with different initial state
(`player.x_velocity|y_velocity`, entities native=1 oracle=0), and `case reduce`
refuses. Fix settled-recording oracle bootstrap before using this idle direct
Level 03 route as gameplay evidence.
Follow-up 2026-06-06 for #238/#229: delegated case-queue route
`issue-playtest-level04-idle-load` rejects raw Level 04 spawn instant-death on
current main. Native-only replay passes for the 301-frame direct Level 04 idle
recording, while full verification is blocked only by oracle checkpoint pixel
differences. The derived `0..120` case `idle_level_04_case_00000_00120` passes
with matching `world.game_mode`, camera, player position, entities, and events;
`case reduce` reports it is already passing. Do not patch Level 04 spawn or
terrain from this clean idle prefix; the remaining instant-death report needs a
transition/carry-state route rather than a direct Level 04 load.
Note 2026-05-31 for #160: latest playtest still reported missing intro/menu music until entering Options. Native was treating Options as the title-shell cue even though ASM `Load_OptionMenu` (`kid.asm:31570-31572`) runs `StopMusic` and starts `bgm_City`; late intro/title shell modes also could fail to recover `bgm_Ice` if entered without the cold `Load_IntroSequence1` cue. Native BGM cue selection now gives Options its own City cue and treats IntroSequence2-6 as recoverable title-shell music contexts. Regressions: `play_native_title_shell_uses_load_title_screen_bgm` and `play_native_options_bgm_replaces_title_shell_with_city`; `verify-audio-output --scenario boot-title-start-level1` still passes.
Note 2026-06-01 for #160: latest playtest again reported no intro music. Fresh native recording `.cache/recordings/issue-160-intro-audio-repro` confirms the live `play-native` BGM event is queued on the first `IntroSequence1` frame (`frame=173`, `StopMusicThenStart(3)` for `bgm_Ice`) and native-only replay passes, so the immediate boot-to-intro command path is now mechanically guarded by `play_native_boot_idle_queues_intro_music_on_first_intro_frame`. If humans still hear silence after that frame, investigate standalone GEMS onset/mix rather than title cue selection.
Note 2026-06-02 for #160: refreshed the live intro waveform after the latest
playtest report with `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo run
-q -p kidcli -- play-native --record issue-160-intro-audio-wave-current
--record-frames 260`. The bundle is native-replayable (`verify-recording
--slug issue-160-intro-audio-wave-current --native-only` passes), queues ASM
`Load_IntroSequence1` at frame 173 (`StopMusicThenStart(3)` / `bgm_Ice`), and
its `audio.wav` is not silent after the cue: boot-pre-intro RMS is `0.001159`,
intro onset RMS is `0.058593`, and intro tail RMS is `0.054369`. This rejects
the immediate "intro BGM command/output is silent" hypothesis on current main.
The title/options recording `issue-003-title-options-audio-audit` still only
contains `audio-events.txt`, not a top-level waveform, so the remaining
playtest complaint needs a fresh title/menu/options route that records
`audio.wav` and compares rendered RMS/onset around title/menu frames, not
another patch to the already-guarded frame-173 cue.
Follow-up 2026-06-03: selector-picked `issue-160-intro-audio-repro` remains
current and clean, but the requested prefix is not an actionable audio case.
Fresh `verify-recording --slug issue-160-intro-audio-repro` passes with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
`first_trace_drift=null`, and matching 300-frame recorded/replay traces. The
requested `0..120` case derives as
`issue-160-intro-audio-repro_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Do not patch #160 from this clean early intro prefix; keep using it only as
guard evidence for the frame-173 intro cue.
Follow-up 2026-06-03: selector-picked `issue-160-live-boot-current` is current
and clean, but its requested prefix is only early boot. Fresh
`verify-recording --slug issue-160-live-boot-current` passes with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
`first_trace_drift=null`, and matching 900-frame recorded/replay traces. The
requested `0..120` case derives as `issue-160-live-boot-current_case_00000_00120`,
passes with `likely_subsystems=["none"]`, and `case reduce` reports it is
already passing. Do not patch #160 from this clean early boot prefix; use a
later title/menu/options or real-device route for remaining audio reports.
Follow-up 2026-06-03: selector-picked `issue-160-intro-audio-wave-current`
also remains current and clean, but its requested prefix is not actionable.
Fresh `verify-recording --slug issue-160-intro-audio-wave-current` passes with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
`first_trace_drift=null`, and matching 260-frame recorded/replay traces. The
requested `0..120` case derives as
`issue-160-intro-audio-wave-current_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Use this route as intro waveform guard evidence only; it does not reach
title/menu/options or expose a new #160 audio mismatch.
Follow-up 2026-06-03: selector-picked `issue-160-audio-bgm-loc12004` is stale
before it can answer any `Load_InGame loc_12004` BGM-ordering question. Fresh
full and native-only `verify-recording` both drift at native replay frame 0,
with `trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`,
and matching 180-frame recorded/replay traces. The requested `0..120` case
derives as `issue-160-audio-bgm-loc12004_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record the
`loc_12004` BGM route on current main before using it for #160 BGM sequencing
work.
Follow-up 2026-06-03: selector-picked `issue-160-audio-ym-status-title` is
stale before it can answer any current YM2612 status/title audio question.
Fresh full and native-only `verify-recording` both drift at native replay frame
0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 180-frame recorded/replay traces. The
requested `0..120` case derives as
`issue-160-audio-ym-status-title_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record the YM-status
title route on current main before using it for #160 audio follow-up work.
Follow-up 2026-06-03: selector-picked `issue-160-audio-smoke` is stale before
it can answer any current audio-smoke question. Fresh full and native-only
`verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 180-frame recorded/replay traces. The requested `0..120` case derives
as `issue-160-audio-smoke_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record the smoke
route on current main before using it for #160 audio follow-up work.
Follow-up 2026-06-03: selector-picked `issue-160-audio-latest-validation` is
no longer current enough to use as listening-bundle evidence. Fresh full and
native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 600-frame recorded/replay traces. The requested `0..120` case derives
as `issue-160-audio-latest-validation_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. This supersedes
the older native-only validation bundle evidence; re-record the listening route
on current main before using it for #160 audio follow-up work.
Follow-up 2026-06-03: selector-picked `issue-160-audio-gems-pointer-order` is
stale before it can answer any current GEMS pointer-order question. Fresh full
and native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 180-frame recorded/replay traces. The requested `0..120` case derives
as `issue-160-audio-gems-pointer-order_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record the
GEMS pointer-order route on current main before using it for #160 audio
follow-up work.
Follow-up 2026-06-03: selector-picked
`issue-160-audio-current-settled-after-startonly` is no longer current enough
to use as settled BGM/SFX evidence. Fresh full and native-only
`verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 480-frame recorded/replay traces. The requested `0..120` case derives
as `issue-160-audio-current-settled-after-startonly_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
This supersedes the older native-only replay evidence for this bundle;
re-record the settled route on current main before using it for #160 audio
follow-up work.
Follow-up 2026-06-03: selector-picked
`issue-160-audio-current-enemy-contact` is no longer current enough to use as
enemy-contact SFX evidence. Fresh full and native-only `verify-recording` both
drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 600-frame
recorded/replay traces. The requested `0..120` case derives as
`issue-160-audio-current-enemy-contact_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. This supersedes
the older native-only replay evidence for this bundle; re-record enemy-contact
audio on current main before using it for #160 SFX follow-up work.
Follow-up 2026-06-03: selector-picked `issue-160-audio-bgm-table` is also
stale before it can answer any BGM table-ordering question. Fresh full and
native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 180-frame recorded/replay traces. The requested `0..120` case derives
as `issue-160-audio-bgm-table_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record the BGM-table
route on current main before using it for #160 BGM sequencing work.
Follow-up 2026-06-03: selector-picked
`issue-160-audio-after-damage-cmdfifo` is stale before it can answer any
command-FIFO or post-damage audio question. Fresh full and native-only
`verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 480-frame recorded/replay traces. The requested `0..120` case derives
as `issue-160-audio-after-damage-cmdfifo_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record the
after-damage audio route on current main before using it for #160 command FIFO
or SFX follow-up work.
Follow-up 2026-06-03: selector-picked
`issue-160-audio-after-damage-post-stopall` is also stale before it can answer
any post-stop-all or after-damage SFX question. Fresh full and native-only
`verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 480-frame recorded/replay traces. The requested `0..120` case derives
as `issue-160-audio-after-damage-post-stopall_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record the after-damage post-stop-all route on current main before using it
for #160 command sequencing or SFX follow-up work.
Follow-up 2026-06-02: the longer selector-picked bundle
`issue-160-latest-playtest-audio-audit` is also non-actionable for title/menu
audio. It verifies fully against oracle (`verify-recording --slug
issue-160-latest-playtest-audio-audit` passes), reaches only
`IntroSequence4` by frame 1426, and contains just the frame-173
`StopMusicThenStart(3)` event. The requested reduced 0..120 case derives and
passes (`case issue-160-latest-playtest-audio-audit_case_00000_00120: PASS`);
`case reduce --case issue-160-latest-playtest-audio-audit` reports the case is
already passing. Do not use this recording for the reported missing menu music:
capture a later route that reaches title/menu/options with `audio.wav` present.
Rechecked 2026-06-03: the same bundle still passes full verification with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
`first_trace_drift=null`, and matching 1426-frame recorded/replay traces; its
`0..120` case still passes with `likely_subsystems=["none"]` and reduction
still reports that it is already passing.
Follow-up 2026-06-02: `issue-160-record-frame-cap-smoke` is likewise clean but
too short for #160 audio triage. It is a 60-frame Sega-screen smoke recording
used to confirm `play-native --record-frames` saves; full verification passes,
the derived 0..60 case passes, and reduction reports the case is already
passing. It never reaches the frame-173 intro music cue, title, menu, or
options, so it should not be selected as an audio regression target.
Rechecked 2026-06-03: the bundle still passes full verification with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
`first_trace_drift=null`, and matching 60-frame recorded/replay traces. The
selector-requested `0..120` range cannot be derived because the scenario has
only 60 frames; the bounded `0..60` case
`issue-160-record-frame-cap-smoke_case_00000_00060` passes with
`likely_subsystems=["none"]` and reduction reports it is already passing.
Follow-up 2026-06-02: `record-scenario` / `record-scenario-file` now persist
rendered `audio.wav` by replaying native trace BGM/SFX events through the GEMS
engine, not just `audio-events.txt`. Regenerated
`issue-160-title-options-audio-wave-current` from the previous title/options
scenario and native replay passes. Its waveform is audible through the reported
windows: post-intro RMS `0.058487`, title/menu pre-options RMS `0.049244`, and
options-after-City RMS `0.046508`. `audio-events.txt` queues
`StopMusicThenStart(3)` at frame 173, menu jingle SFX at 2730/2740, and
`StopMusicThenStart(101)` on `OptionsMenu` entry at 2751. Default oracle
verification is not clean for this Options route (`options_entry@2760` Kid
position mismatch), so this is native waveform evidence rather than an
oracle-authoritative gameplay case. The remaining human "silent menu" report
needs either a live-route reproduction that differs from this scripted path or
an SDL playback/mixer investigation, not another title/options command fix.
Rechecked 2026-06-03: the scripted title/options waveform route remains
native-replayable (`verify-recording --slug
issue-160-title-options-audio-wave-current --native-only` passes), but full
oracle verification still diverges at `options_entry@2760` on Kid position
(`native 002E0000,008C0000`; `oracle 00280000,00440000`). The selector
`0..120` prefix derives as `issue-003-title-options-audio-audit_case_00000_00120`,
passes with `likely_subsystems=["none"]`, and reduction reports it is already
passing, so do not patch from the clean early prefix.
Follow-up 2026-06-02: `play-native` recording bundles now include
`audio-diagnostics.txt` so live captures preserve whether GEMS loaded, whether
SDL audio opened, how many generated frames were queued, and whether queue
back-pressure skipped any frames. Fresh dummy-SDL live bundle
`issue-160-live-audio-diagnostics-current` reaches the frame-173
`Load_IntroSequence1` cue, verifies native-only, and records
`gems_loaded=true`, `sdl_queue_present=true`, `generated_frames=260`,
`queued_frames=260`, `skipped_full_frames=0`, and `queue_errors=0`. Its
waveform is still audible after the cue (`boot_pre_intro_rms=0.001187`,
`intro_after_cue_rms=0.056608`, global RMS `0.032759`, peak `0.228230`).
This rejects dummy-SDL queue starvation for the cold intro route; remaining
human "I hear no intro/menu music" reports need a real-device capture with the
new diagnostics or a longer live title/menu/options capture.
Rechecked 2026-06-03: `issue-160-live-audio-diagnostics-current` still passes
full verification with `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, `first_trace_drift=null`, and matching 260-frame
recorded/replay traces. Its `0..120` case also passes with
`likely_subsystems=["none"]`, and reduction still reports that it is already
passing, so it remains cold-intro diagnostics evidence rather than a new patch
target.
Follow-up 2026-06-02: added `play-native --scenario <DynamicScenario>` so a
script can drive the live SDL player/audio queue rather than only the
non-interactive recorder. With no explicit `--record-frames`, recorded scripted
live routes stop at the scenario frame count. Fresh live-SDL title/options
bundle `issue-160-live-title-options-audio-diagnostics` replays
`.cache/recordings/issue-003-title-options-audio-audit/scenario.json`, records
2,860 frames, and verifies native-only. `audio-diagnostics.txt` reports
`gems_loaded=true`, `sdl_queue_present=true`, `queued_frames=2860`,
`skipped_full_frames=0`, `queue_errors=0`, and max pre-queue depth `3568`
bytes. `audio-events.txt` records frame-173 `StopMusicThenStart(3)` and
frame-2751 `StopMusicThenStart(101)`. The rendered waveform is audible in the
reported windows: intro-after-cue RMS `0.058487`, title/menu pre-options RMS
`0.049244`, options-after-City RMS `0.046508`, global RMS `0.052889`. This
rejects the live SDL queue/output path for the deterministic title/options
route; if a human still hears silence, capture their exact route with
`play-native --record ...` and inspect `audio-diagnostics.txt`.
Rechecked 2026-06-03: the route remains native-replayable
(`verify-recording --slug issue-160-live-title-options-audio-diagnostics
--native-only` passes) and the requested `0..120` case passes as
`issue-160-live-title-options-audio-diagnostics_case_00000_00120` with
`likely_subsystems=["none"]`; reduction still reports that prefix is already
passing. Full oracle verification is still not an authoritative gameplay
wrapper for the later Options checkpoint: `options_entry@2760` diverges on Kid
position (`native 002E0000,008C0000`; `oracle 00280000,00440000`).
Follow-up 2026-06-02: selector-picked recording
`codex-boot-title-idle-audio-current` is also non-actionable for reduced
gameplay or audio fixes. Its original bundle is trace-stale at native replay
frame 0, but full verification still reports oracle state and pixel parity
green. Regenerating the same 600-frame idle DynamicScenario as
`codex-boot-title-idle-audio-current-refresh` verifies fully against oracle,
and the requested 0..120 derived case
`codex-boot-title-idle-audio-current-refresh` passes; `case reduce` correctly
reports that there is nothing to reduce. The follow-up selector-picked
`codex-dummy-boot-audio-current` has the same shape: the old bundle is
trace-stale at frame 0, but regenerated
`codex-dummy-boot-audio-current-refresh` verifies fully and its 0..120 case
passes. These routes reach only the early Sega/intro prefix before title/menu
interaction, so they should not drive another #160 patch.
Follow-up 2026-06-02: the original `codex-boot-title-idle-audio-current`
bundle now verifies fully against oracle without regeneration. Fresh
`verify-recording --slug codex-boot-title-idle-audio-current` passes with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
recorded/replay trace len 600, and zero-pixel title/title-tail frame diffs. The
requested 0..120 case derives as `boot_title_idle_10s_case_00000_00120` and
passes with matching world mode, camera, player, entities, and events. This
remains non-actionable: it covers early boot/title idle parity and exposes no
audio or gameplay mismatch to reduce.
Follow-up 2026-06-02: selector-picked
`codex-boot-title-idle-audio-current-refresh` has the same clean shape. Fresh
`verify-recording --slug codex-boot-title-idle-audio-current-refresh` passes
with trace/state/frame green, recorded/replay trace len 600, and zero-pixel
title/title-tail diffs. Its requested 0..120 case also resolves to
`boot_title_idle_10s_case_00000_00120` and passes. Do not use this refreshed
boot/title idle route as an audio/gameplay patch target unless a future
recording adds later title/menu interaction or a failing audio assertion.
Follow-up 2026-06-02: the original `codex-dummy-boot-audio-current` bundle now
also verifies fully without regeneration. Fresh `verify-recording --slug
codex-dummy-boot-audio-current` passes with trace/state/frame green, trace len
240, and zero-pixel diffs at frames 0/60/120/180/239. Its requested 0..120 case
`codex-dummy-boot-audio-current_case_00000_00120` passes with matching world
mode, camera, player, entities, and events. This dummy boot prefix still exposes
no audio or gameplay mismatch to reduce.
Follow-up 2026-06-02: selector-picked `codex-dummy-boot-audio-current-refresh`
has the same clean shape. Fresh verification passes with trace/state/frame
green, trace len 240, and zero-pixel diffs at frames 0/60/120/180/239. Its
requested 0..120 case resolves to
`codex-dummy-boot-audio-current_case_00000_00120` and passes. This refreshed
dummy boot prefix is also non-actionable.
Follow-up 2026-06-06: selector-picked `issue-audio-cold-title-native` is a
current clean cold-title guard, not a new #160 patch target. Fresh native-only
and full `verify-recording --slug issue-audio-cold-title-native` both pass with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
matching 2889-frame traces, and zero-pixel sampled frame diffs. The transcript
walks the cold shell from `SegaScreen` through `IntroSequence1..6` (`bgm_Ice`
title-shell route), but the requested `0..120` case derives and passes with
`likely_subsystems=["none"]`; reduction has no failing behaviour to minimise.
Keep this as cold intro/title parity evidence. Remaining #160 work still needs
a concrete human/real-device route or failing audio assertion, not another
patch from this clean prefix.
Follow-up 2026-06-06: selector-picked `issue-audio-intro-bgm-fixed` is also a
current clean intro BGM guard, not an actionable reduced gameplay case. Fresh
full `verify-recording --slug issue-audio-intro-bgm-fixed` passes with
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=true`,
matching 755-frame traces, and zero-pixel diffs at every sampled checkpoint
from `0` through `754`. The transcript reaches `IntroSequence1` at frame 173
and `IntroSequence2` at frame 534, but never reaches title/menu/options or
gameplay. The requested `0..120` case derives as
`issue-audio-intro-bgm-fixed_case_00000_00120`, passes with matching
`world.game_mode`, camera, player position/motion, entities, and events, and
`case reduce` reports it is already passing. Keep it as intro-BGM parity
evidence only; #160 still needs a failing audio route or real-device capture.
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
Note 2026-06-02 for #161: fresh `cargo run -p kidcli -- verify-recording --slug issue-161-current-intro-long` on current `main` passes trace, oracle-state, and oracle-frame checks. `report.json` records `trace_match=true`, `first_trace_drift=null`, `oracle_state_match=true`, and `oracle_frame_match=true`; the 48 `frame_diffs` entries are zero-diff checkpoints, not pixel failures. ASM labels rechecked for the covered cold-idle route: `Load_IntroSequence2`, `Load_IntroSequence4`, `Load_IntroSequence5`, and `Load_IntroSequence6` plus their visible workers (`loc_1BA28`, `loc_1BCD0`, `loc_1B37C`, `sub_1B43A`). This rejects the old cold-idle hypothesis for the latest playtest corruption; the remaining report needs a deterministic live route that presses Start/options/menu or otherwise reproduces the human interruption path, not another patch to the already-green `issue-161-current-intro-long` bundle.
Note 2026-06-02 selector follow-up for #161: queued
`codex-intro2-corridor-static-probe` is also clean. Fresh `verify-recording`
passes with trace/state/frame green and no first trace drift; recorded/replay
traces are both 2790 frames. Its requested `0..120` case is
`issue-161-current-intro-long_case_00000_00120`, which passes with world mode,
camera, player position/motion, entities, and events matching; `case reduce`
reports it is already passing. Do not patch the cold intro corridor/static path
from this selector item.
Follow-up 2026-06-02: selector-picked `codex-intro2-fade-hold-probe` is clean
too. Fresh `verify-recording` passes with trace/state/frame green and no first
trace drift; recorded/replay traces are both 2790 frames. Its requested `0..120`
case is `issue-161-current-intro-long_case_00000_00120`, which passes with
world mode, camera, player position/motion, entities, and events matching; `case
reduce` reports it is already passing. This fade-hold route is non-actionable
for #161/#210.
Follow-up 2026-06-02: selector-picked `codex-intro2-handoff-probe` is clean
too. Fresh `verify-recording` passes with trace/state/frame green and no first
trace drift; recorded/replay traces are both 2790 frames. Its requested `0..120`
case is `issue-161-current-intro-long_case_00000_00120`, which passes with
world mode, camera, player position/motion, entities, and events matching; `case
reduce` reports it is already passing. This handoff route is non-actionable for
#161/#210.
Note 2026-05-26 for #161: `Load_IntroSequence5` now replays the staged `loc_1BC36` tile worker: `sub_1C034` frame uploads from `off_1C126` switch on the ROM hibernate boundaries, then `sub_1C0EE` / `sub_1C112` blit and clear the 10x10 panel at plane cell `(3,$13)`. Regressions: `intro_sequence_5_loc_1bc36_streams_sub_1c034_frames_on_rom_hibernate_boundaries`, `intro_sequence_5_loc_1bc36_panel_blit_and_blank_follow_sub_1c0ee_order`. Superseded by the later full cold-intro CRC pass below.
Note 2026-05-26 for #161: recording verification now replays checkpoint frames through the live `NativeRuntime::vdp_state` path used by `play-native`, instead of rebuilding intro screenshots from the trace-only fallback that intentionally returns black for intro modes. Refreshed `.cache/recordings/issue-161-current-intro-long` verifies native-only and now shows the Wild Side exterior, Hologram room, intro-3 roster scene, and story text in `verify/native/` checkpoint PNGs. Superseded by the later live VDP CRC pass below; the remaining PNG diffs are evidence-path debt.
Note 2026-05-26 for #161: cold-boot intro timing now follows the ASM object loops through the first three intro segments. `Load_SegaScreen` keeps mode `$0000` through oracle frame 172, `sub_1B8F4/loc_1B924` keeps intro 1 through frame 533, `Load_IntroSequence2/loc_1BA02` keeps intro 2 through frame 982, and `sub_1B93E/loc_1B96E` keeps intro 3 through frame 1281. Regressions: `cold_boot_sega_screen_hands_off_to_intro_sequence_1_on_oracle_frame_173`, `cold_boot_intro_sequences_1_and_2_follow_oracle_handoff_frames`, `cold_boot_intro_sequence_3_follows_oracle_handoff_frame`. `verify-tas --scenario .cache/recordings/issue-161-current-intro-long/scenario.json --frame-start 0 --frame-end 2200 --checkpoint-interval 60 --bisect --frontier-kind gameplay` now moves the first CRC frontier from frame 119 to frame 1282; remaining #161 frontier is intro-3 roster/Kid trace state carrying into intro 4, not the coarse boot timers.
Note 2026-05-26 for #161: the cold intro-3/4/5 hand-off state now keeps the previous intro-3 roster pose through early intro 4, exposes `sub_1B988`'s `Camera_Y_pos=4` hand-off, holds intro 4 until oracle frame 1875, and exposes the early `loc_1BCD0` slot-2 pose/counts in intro 5. Regressions: `cold_boot_intro_sequence_4_follows_oracle_handoff_frame`, `intro_sequence_4_intro_3_pose_carries_until_cutoff`, `camera_handoff_spans_late_intro_4_and_timeout_restart_intro_5`, `current_player_state_exposes_cold_intro_5_slot_2_pose_after_delay`. `verify-tas --scenario .cache/recordings/issue-161-current-intro-long/scenario.json --frame-start 0 --frame-end 2600 --checkpoint-interval 60 --bisect --frontier-kind gameplay` now moves the first CRC frontier to frame 1956; remaining #161 frontier is the native shortcut from intro 5 directly to title instead of the ROM's later intro-5/intro-6 flow.
Note 2026-05-26 for #161: cold intro sequence 5 now uses the ASM `loc_1BA28 -> sub_1C204 -> loc_1BA5C` duration and naturally hands off to `Load_IntroSequence6` (`Game_Mode=$28`) on oracle frame 2396 instead of jumping to title mode `$2C`; the cold path also avoids the timeout-restart hidden-camera-wrap flag. Regression: `cold_boot_reaches_intro_sequence_6_without_input`. `verify-tas --scenario .cache/recordings/issue-161-current-intro-long/scenario.json --frame-start 0 --frame-end 3200 --checkpoint-interval 60 --bisect --frontier-kind gameplay` now passes all checked frames by CRC.
Note 2026-05-28 for #166: latest playtest still saw the Blue Lake Woods walking dragon as partly see-through and not animating its face correctly. This was a separate renderer bug from the forest first-slot remap: native added a layout-index phase offset to DragonWalking live frames, selecting a later `stru_3CC5E` frame than ASM `j_Init_Animation` would for the first visible Level 1 dragon. Native now follows `stru_3CC5E`/`stru_3CC90` timing without the layout-index offset. Regression: `live_walking_dragon_uses_rom_walk_sequence_not_death_frames`.
Note 2026-05-28 for #166: the same invented layout-index phase offset also affected RockTank body/turret animation, matching the playtest report that the Level 2 "little head guy" still had see-through/wrong-looking eyes after the dragon fix. ASM `Tank.asm loc_3B372` and `Tank_Shooting.asm loc_3B5FA` call `j_Init_Animation` directly for `stru_3B270`/`stru_3B4CA` and their child turret sequences, so native now removes the layout-index phase seed from both body and turret selectors. Regression: `live_rock_tank_animation_does_not_phase_shift_by_layout_index`.
Note 2026-06-02 for #167: latest playtest still reports the Level 01 RockTank final hit as visually wrong, but current evidence does not reproduce an actionable Tank death regression. ASM rechecked: `Tank.asm loc_3B41C -> loc_3B480` and `Tank_Shooting.asm loc_3B6E6` hide the linked mouth/turret child with `sf $13(a2)`, restore its kid collision with `st $3D(a2)`, initialise `stru_3B29A`/`stru_3B518`, and `j_Hibernate_UntilAnimFinished` before shared deletion. Native focused checks pass: `cargo test -p kid-core tank_death_waits_for_stru_3b29a_animation_before_delete -- --nocapture`, `cargo test -p kid-core tank_shooting_death_waits_for_stru_3b518_animation_before_delete -- --nocapture`, and `cargo test -p kidcli tank_and_robot_death_snapshots_select_waited_body_frames -- --nocapture`. Candidate cache bundles are not usable: `debug-rocktank-level2` and `tmp-death-animation-level0-stomp-current` both drift at frame 0 under `verify-recording --native-only`, while the Level 01 tank-walk transcripts only show Kid HP damage (`hp 2 -> 1`) and no Tank kill. Next pass needs a fresh deterministic live Tank/RockTank kill route, preferably Level 01 with the final child-hit collision or a direct oracle-authoritative pending-collision scenario; do not patch death rendering from the stale bundles.
Note 2026-06-04 for #167: added a scene-level guard for the #252 RockTank death visibility report. `build_scene_keeps_rock_tank_death_body_visible_loc_3b480` feeds a zero-HP Tank snapshot with the full `stru_3B29A` wait timer through `build_scene` and asserts the first ROM death body frame is emitted, not only selected by `select_enemy_frame` in isolation. This keeps the actual draw-command path covered while a fresh deterministic live Tank kill route is still needed for oracle-authoritative timing/pixel proof.
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
Note 2026-05-28 for #168: latest playtest still saw the rhino/Berzerker not putting its head forward while charging into blocks. Root cause: native used the late `Berzerker_charging` gameplay flag as the renderer selector. ASM `loc_9564` writes the off_9688 head-forward sprite first, then wall/block collision may clear `Berzerker_charging` at `loc_9326` in the same frame. Native now exposes a one-frame `berzerker_charge_pose` trace latch for that sprite write while keeping the gameplay flag clear for collision semantics. Regressions: `berzerker_charge_pose_survives_same_frame_wall_clear_loc_9564_9326`, `berzerker_charge_pose_latch_selects_head_forward_after_flag_clear_loc_9326`.
Note 2026-06-02 for #169: current selector rechecked this after the latest playtest, but #169's old Level 2 evidence is stale and the active live repro has moved to #213. `cargo test -p kid-core drips_ -- --nocapture` passes 109 focused Drips checks, including `loc_333F6` ceiling attach, `loc_33424` child forming, `loc_3345C` floor collision, `loc_33488` pool/splash writeback, `loc_335D8` crawler spawn, and the `loc_33650..loc_33AF6` crawler scripts. The current clean prefix is still only `cargo run -p kidcli -- case verify --case issue-213-drips-camera-only` (`PASS`), not the later terrain/jump interaction. The old #169 bundles `issue-169-drips-level2-walk` and `issue-169-drips-level2-stomp-long` both drift at frame 0 under `verify-recording --native-only`, so do not patch from them. Continue under #213 with a fresh longer settled Level 01 Drips recording that reaches drop/pool/metal-block contact.

Note 2026-05-27 for #170: the latest playtest still saw an invented native GAME OVER / CONTINUE panel after death. Native now exposes `game_over_elapsed_frames` / `continue_countdown_elapsed_frames`, loads `Spr_EOL_GAME` and `Spr_EOL_OVER`, and renders the no-continues and continue countdown overlays through ASM `loc_D25E` / `Continue_Screen` EOL sprite workers instead of panel text. With continues remaining, native no longer shows the false GAME OVER interstitial before the continue screen. Regressions: `game_over_with_remaining_continues_does_not_render_invented_panel_loc_d25e`, `continue_countdown_uses_rom_eol_continues_sprites_loc_d25e`, `no_continue_game_over_uses_rom_game_over_sprites_loc_d25e`, `trace_frame_exports_continue_countdown_remaining_frames`, and `load_eol_score_visuals_returns_rom_label_and_digit_frames`. Verification: `cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`; `mise run verify-slice-fast`.
Note 2026-06-03 selector follow-up for #175: selector-picked
`debug-eol-phase-native` is stale before any current EOL phase evidence. Fresh
full and native-only `verify-recording` both drift against the cached native
trace at frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 201-frame recorded/replay traces. The
requested `0..120` case derives as
`issue-175-eol-flag-touch-level12_settled_recording_oracle_case_00000_00120`,
but `case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this EOL phase route on current native before using it as gameplay
evidence.
Note 2026-06-03 for #190: selector-picked recording `issue-190-level20-robot-walkright` is stale before any current Level 20 Robot gameplay evidence. Fresh full and native-only verification both drift at native replay frame 0, with `trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and matching 761-frame recorded/replay traces. The requested `0..120` case derives as `walk_right_14_case_00000_00120`, but `case verify` reports `likely_subsystems=["recording verification"]`, and `case reduce` refuses because the source recording replay drifted at frame 0. Do not reopen #190 from this saved bundle; re-record the Level 20 Robot route on current native before using it as gameplay evidence.
Note 2026-06-03 for #190: selector-picked variant
`issue-190-level20-robot-walkright-dec32` is also stale before any current
Level 20 Robot gameplay evidence. Fresh full and native-only verification both
drift at native replay frame 61, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 961-frame
recorded/replay traces. The requested `0..120` case derives as
`walk_right_20_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 61. Re-record this
Level 20 Robot route on current native before using it as gameplay evidence.
Note 2026-05-30 for #203: live shooter blocks now use the already-modelled `sub_F536`/`loc_F326` restart/free branch at active-slot expiry, so a visible shooter keeps firing while Kid remains in range instead of silently stopping until the next scan-boundary crossing.
Note 2026-05-31 for #203: started the hygiene workstream without promoting behavioural wiring. Removed stale dead-code suppressions from the already-live lava collision/init helpers in `weather.rs`: `kid_hitbox_bounds_px_at_screen`, `lava_geyser_hitbox_window_loc_14AAA`, `lava_fireball_vertical_window_loc_14AFC`, `lava_fireball_rejects_kid_loc_14B22`, `lava_fireball_overlap_gate_loc_14B06`, `lava_fireball_hitbox_x_window_loc_14AB0`, `lava_geyser_initial_delay_frames_loc_14772`, `lava_geyser_position_table_loc_14C5E`, and `initialise_lava_geyser_loc_14C6A`. Live evidence: `NativeRuntime::step` reaches `step_lava_geysers`, and `init_weather` reaches `init_lava_geysers`; `cargo check -q -p kid-core` confirms the compiler sees these helpers/types as used without the suppressions. Focused gates: `cargo test -q -p kid-core lava_geyser -- --nocapture`, `cargo test -q -p kid-core lava_fireball -- --nocapture`. The dark lava geyser/fireball script-machine wiring remains open and still needs an object/weather-active oracle scenario under #205 before behavioural promotion.
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
Note 2026-06-02 for #206: current selector rechecked the Diamond Power audit cache after the latest playtest raised repeated fidelity concerns and the earlier question about `.cache`-dependent tests. The five cache bundles are no longer native-replayable on current `main`: `cargo run -q -p kidcli -- verify-recording --slug issue-206-diamond-power-circle-audit`, `...wall-audit`, `...snake-audit`, `...invulnerability-audit`, and `...samurai-haze-audit` all fail at frame 0 with `recording replay drifted ... native trace frame mismatch`. Their regenerated reports still show `oracle_state_match=true`, `state_diff_count=0`, and `oracle_frame_match=false`, so the old state-green notes are not enough for a current gate. Keep #206 at `[?]` until the committed scenario builders are re-recorded or verified through a cache-independent path; do not land tests that depend on ignored `.cache/recordings/issue-206-*` contents.
Note 2026-06-03 for #206: selector rechecked
`issue-206-diamond-power-circle-audit`; it is still stale before any current
Diamond Power gameplay evidence. Fresh full and native-only verification both
drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 121-frame
recorded/replay traces. The requested `0..120` case derives as
`diamond_power_circle_audit_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record the Circle
audit from the committed scenario builder before using it as #206 gate evidence.
Note 2026-06-07 for #206: selector re-recorded the Circle audit from the
committed scenario builder as
`.cache/recordings/issue-206-diamond-power-circle-audit-current-20260607`.
`verify-recording --slug issue-206-diamond-power-circle-audit-current-20260607`
now has `trace_match=true`, `oracle_state_match=true`, no first trace/state
drift, recorded/replay trace lengths `121/121`, and `oracle_frame_match=false`
over 22 sampled checkpoints. The first strict-pixel diff is
`diamond_power_input@0` with `changed_pixel_count=3660`, MAE `1.8747722`,
bounds `x=0..319,y=16..156`, and top colour swaps
`[0,109,36]<->[0,146,36]`, matching the broad Forest palette/background
mismatch shape rather than a Circle replay/state failure.
Note 2026-06-03 for #206: selector rechecked
`issue-206-diamond-power-invulnerability-audit`; it is stale in the same way.
Fresh full and native-only verification both drift at native replay frame 0,
with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 121-frame recorded/replay traces. The
requested `0..120` case derives as
`diamond_power_invulnerability_audit_02_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record the
Invulnerability audit from the committed scenario builder before using it as
#206 gate evidence.
Note 2026-06-07 for #206: selector re-recorded the Invulnerability audit from
the committed Level 02 scenario builder as
`.cache/recordings/issue-206-diamond-power-invulnerability-audit-current-20260607`.
`verify-recording --slug issue-206-diamond-power-invulnerability-audit-current-20260607`
now has `trace_match=true`, `oracle_state_match=true`, no first trace/state
drift, recorded/replay trace lengths `121/121`, and `oracle_frame_match=false`
over 22 sampled checkpoints. The verifier warned that oracle captured no enemy
snapshots for this route, so per-enemy comparison was skipped. The first
strict-pixel diff is `diamond_power_input@0` with `changed_pixel_count=603`,
MAE `0.9727167`, bounds `x=31..299,y=13..163`, and top colour swaps including
`[182,182,73]<->[255,255,255]`; this refreshes the Invulnerability replay/state
gate but still leaves a non-Forest strict-pixel gap.
Note 2026-06-03 for #206: selector rechecked
`issue-206-diamond-power-samurai-haze-audit`; it is stale in the same way.
Fresh full and native-only verification both drift at native replay frame 0,
with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 121-frame recorded/replay traces. The
requested `0..120` case derives as
`diamond_power_samurai_haze_audit_02_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record the
Samurai Haze audit from the committed scenario builder before using it as #206
gate evidence.
Note 2026-06-07 for #206: selector re-recorded the Samurai Haze audit from the
committed Level 02 scenario builder as
`.cache/recordings/issue-206-diamond-power-samurai-haze-audit-current-20260607`.
`verify-recording --slug issue-206-diamond-power-samurai-haze-audit-current-20260607`
now has `trace_match=true`, `oracle_state_match=true`, no first trace/state
drift, recorded/replay trace lengths `121/121`, and `oracle_frame_match=false`
over 22 sampled checkpoints. The verifier warned that oracle captured no enemy
snapshots for this route, so per-enemy comparison was skipped. The first
strict-pixel diff is `diamond_power_input@0` with `changed_pixel_count=687`,
MAE `0.8620303`, bounds `x=32..299,y=13..163`, and top colour swaps including
`[0,0,0] -> [36,36,0]` plus `[182,182,73]<->[255,255,255]`; this refreshes the
Samurai Haze replay/state gate but still leaves a non-Forest strict-pixel gap.
Note 2026-06-03 for #206: selector rechecked
`issue-206-diamond-power-snake-audit`; it is stale in the same way. Fresh full
and native-only verification both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 121-frame recorded/replay traces. The requested `0..120` case derives
as `diamond_power_snake_audit_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record the Snake
audit from the committed scenario builder before using it as #206 gate evidence.
Note 2026-06-07 for #206: selector re-recorded the Snake audit from the
committed scenario builder as
`.cache/recordings/issue-206-diamond-power-snake-audit-current-20260607`.
`verify-recording --slug issue-206-diamond-power-snake-audit-current-20260607`
now has `trace_match=true`, `oracle_state_match=true`, no first trace/state
drift, recorded/replay trace lengths `121/121`, and `oracle_frame_match=false`
over 22 sampled checkpoints. The first strict-pixel diff is
`diamond_power_input@0` with `changed_pixel_count=4101`, MAE `2.4353747`,
bounds `x=0..319,y=16..156`, and top colour swaps
`[0,109,36]<->[0,146,36]`, matching the same broad Forest
palette/background mismatch shape as Circle and Wall rather than a Snake
replay/state failure.
Note 2026-06-03 for #206: selector rechecked
`issue-206-diamond-power-wall-audit`; it is stale in the same way. Fresh full
and native-only verification both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 121-frame recorded/replay traces. The requested `0..120` case derives
as `diamond_power_wall_audit_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record the Wall audit
from the committed scenario builder before using it as #206 gate evidence.
Note 2026-06-07 for #206: selector re-recorded the Wall audit from the committed
scenario builder as
`.cache/recordings/issue-206-diamond-power-wall-audit-current-20260607`.
`verify-recording --slug issue-206-diamond-power-wall-audit-current-20260607`
now has `trace_match=true`, `oracle_state_match=true`, recorded/replay trace
lengths `121/121`, and `oracle_frame_match=false` over 22 sampled checkpoints.
The first strict-pixel diff is `diamond_power_input@0` with
`changed_pixel_count=4212`, MAE `2.6393974`, bounds `x=0..319,y=16..156`, and
top colour swaps `[0,109,36]<->[0,146,36]`, i.e. the broad Forest
palette/background mismatch rather than a fresh Wall state/replay failure. This
refreshes all five #206 replay/state gates. #206 remains `[?]` because strict
pixel parity is still red for every audit bundle.
Note 2026-06-02 for #207: rechecked the ASM and live call graph before changing code. `kid.asm:909` calls `sub_21F8` during `Mode_Level`; `kid.asm:940` calls `sub_1F52`; `kid.asm:948` calls `sub_1FA2`. The native definitions in `crates/kid-core/src/combat.rs` still carry `cfg_attr(not(test), allow(dead_code))` and `rg` finds no non-test caller for `kid_hitbox_from_collision_size_sub_21F8`, `support_overlap_detected_sub_1F52`, `first_solid_overlap_action_sub_1FA2`, or `first_solid_overlap_resolution_sub_1FA2`. Focused verification passed 13 helper tests across the three ASM labels. Proposed implementation path: build a #205 block-collision scenario that exercises the post-object `Mode_Level` collision phase, translate native player/platform/enemy collision state into the existing `CollisionRectSub1F52`/`SolidOverlapTileSub1FA2` values, then wire `first_solid_overlap_action_sub_1FA2` as a global post-movement correction instead of replacing the already-TAS-sensitive `apply_wall_collision`/`apply_ceiling_collision`/`apply_floor_collision` paths blindly.
Note 2026-06-02 for #208: rechecked `kid.asm:4465-4642` (`Platforms_CheckCollision`, `loc_2686`, `loc_26A2`, `loc_26B6`, `loc_26C0`, `loc_26D4`, `loc_26FE`) and current native. `apply_platform_collision_with_ceiling_probe` is live from `lib.rs`, but it still differs from the ROM loop: no exact smaller-axis escape, no FAA2-FAA5 flag accumulation, no `Addr_PlatformStandingOn` writeback, no `$4000` bottom-face y-velocity bias, and it returns after the first approximate match. Focused helper checks pass (`cargo test -q -p kid-core loc_26 -- --nocapture`, 13 tests) and the broad live approximation tests pass (`cargo test -q -p kid-core platform_collision -- --nocapture`, 29 tests). The session fixed the harness-independent `sub_8ED0` width bug by replacing the hardcoded `PLAYER_HALF_WIDTH`/`PlayerForm::TheKid` carry support check with the live `current_form`; regression `platform_carry_uses_current_form_half_width_loc_8ed0` first failed, then passed.
Note 2026-06-01 for #210: fixed one native-player fallback cause of the unreadable Options report. The asset-backed ROM tile screen is still future work, but the typed fallback now follows ASM `DrawOptText1..4` (`loc_1CE3A`/`loc_1CEB2`/`loc_1CF78`/`loc_1CFFE`) cell positions instead of joining whole option groups into over-wide rows; regression `options_menu_fallback_keeps_rom_rows_inside_frame_loc_1ce3a` keeps every fallback row inside the 320x224 frame.
Note 2026-05-30 for #210: first slice recorded `.cache/debug/issue-210/intro6_start_menu.json` (cold boot, Start at frames 2600 and 2700). Fixed the state/input side: frame 2600 now keeps the `Load_IntroSequence6` ninja/lightning shell instead of exposing the top-right title pose, frame 2602 follows ASM `sub_1C246 -> loc_1C264` into `Game_Mode=$2C`, frame 2660 reaches title-ready, and frame 2701 stays on TitleScreen so the second Start builds the menu instead of immediately entering Level 1. Regression: `cold_boot_intro_sequence_6_start_keeps_loc_1b37c_shell_until_loc_1b5ce`. Remaining #210 work is visual composition: native frame 2720 still shows the green/black title shell and garbled menu tilemap while oracle shows the full Kid Chameleon title art and readable menu. Gate for fixed slice: `mise run verify-slice-fast`.
Note 2026-05-30 for #210: follow-up visual slice wires the ready title screen through ASM `Load_TitleScreen`'s visible character frame 7, `Obj_TitleTextKid`, `Obj_TitleTextChameleon/sub_1B71A`, and all four `Load_TitleArt` uploads including `Title_text_CHAMELEON.bin` at VRAM `$6760`. Native `.cache/recordings/issue-210-title-logo-art-fix/verify/native/002720_menu_visible.png` now shows readable menu text plus the KID/CHAMELEON logo instead of the green/black shell. Remaining visual debt: title ninja composition and logo background still differ from oracle frame `.cache/tas-verify/issue_210_intro6_start_menu/oracle/2720_frame_2720.png`. Regressions: `title_screen_vdp_obj_title_text_chameleon_sub_1b71a_writes_scroll_a_reveal_rows`, `title_screen_vdp_load_title_art_uploads_menu_chunks`, and extended `intro_return_hidden_title_shell_selects_sub_1bf24_frames`.
Note 2026-05-30 for #210: fixed the Chameleon-logo black rectangle by applying ASM `unk_1B4F4` during the `DecompressToRAM`-style `Title_text_CHAMELEON.bin` upload. Native `.cache/recordings/issue-210-chameleon-permutation/verify/native/002720_menu_visible.png` now has transparent logo backing like oracle. Remaining visual debt: title ninja composition/placement and copyright text still differ from `.cache/tas-verify/issue_210_intro6_start_menu/oracle/2720_frame_2720.png`. Regression: `title_text_chameleon_art_loc_1b4f4_remaps_e_nibbles_to_transparent_zero`.
Note 2026-05-30 for #210: fixed the title character by honouring `off_1BFB0`'s per-frame width words in `sub_1BF24`, and replayed `SegaText` through `loc_1C7A0` after the `loc_1B6AC -> loc_1B6DC` Chameleon reveal cadence. Native `.cache/recordings/issue-210-title-frame-sega-text/verify/native/002720_menu_visible.png` now has readable menu text, correctly placed Red Stealth title pose, transparent KID/CHAMELEON logo, and bottom `© 1992 SEGA`. Remaining visual debt: the small title-logo `TM` detail is still absent compared with oracle frame `.cache/tas-verify/issue_210_intro6_start_menu/oracle/2720_frame_2720.png`; the broader manual title/demo re-entry oracle probe also remains red. Regressions: `title_screen_character_load_plan_sub_1bf24_switches_non_zero_frames_by_region_bank`, `title_screen_vdp_loc_1b6dc_replays_sega_text_after_chameleon_reveal`.
Note 2026-05-30 for #210: fixed the remaining visible menu rectangle corruption from the short `unk_2EAAE` decode. Native's stateless title renderer now pads the 65 decoded menu words to the 14x6 `loc_1BB52/sub_1C5D0` rectangle with the menu blank word `$4528`, preserving the cached black rectangle instead of leaking current-frame background/logo data into the right/bottom cells. Evidence: `.cache/recordings/issue-210-title-menu-blank-tail/verify/native/002720_menu_visible.png`. Regression: `title_menu_tilemap_loc_1bb52_pads_short_decode_with_menu_blank`. Remaining #210 debt unchanged: small `TM` mark and manual title/demo re-entry oracle probe.
Note 2026-05-30 for #210: extended the cold-boot idle probe to 9000 frames at `.cache/debug/issue-210/cold_boot_idle_demo_loop_long.json`. The first gameflow miss was native entering `Game_Mode=$10` three frames before oracle on the natural `Load_IntroSequence6 -> Obj_TitleMenu loc_1BAEA -> loc_1BB02` path; native now keeps mode `$28` through frame 3269 and starts demo play on frame 3270. The same slice found the cold-boot Skydragon demo bootstrap camera rows: startup frame 57 uses `$01E8_0000`, frame 58 uses `$01C0_0000`, then frame 59 snaps to `$02A0_0000`. Regression: `cold_boot_intro_sequence_6_enters_demo_play_on_frame_3270`. Targeted published timeout-restore preload tests still pass. The 9000-frame probe now reaches the next later-loop divergence at frame 6850.
Note 2026-06-03 selector follow-up for #210: selector-picked
`debug-cold-loop-trace` is not an actionable early gameplay reducer. Fresh full
and native-only `verify-recording` both drift against the cached native trace at
frame 3270, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no oracle state/pixel mismatch, and matching
9000-frame recorded/replay traces. The requested `0..120` case derives as
`issue_210_cold_boot_idle_demo_loop_long_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Re-record this cold-loop wrapper on current native before using its later tail;
do not patch #210 from the clean early prefix.
Note 2026-06-03 selector follow-up for #210: selector-picked
`issue-210-cold-idle-2600` is current for native replay and oracle state, but
still not an actionable gameplay reducer. Fresh native-only verification passes;
fresh full `verify-recording` has `trace_match=true`, `oracle_state_match=true`,
`first_trace_drift=null`, and matching 2601-frame traces, but fails strict pixels
with `oracle_frame_match=false`. The first visible miss is `demo_boot@700`
(`changed_pixel_count=37025`, full-frame bounds `0..319,0..223`), while
`sega_tail@172`, `intro_mid@360`, and `title_timeout@640` are pixel-clean. The
requested `0..120` case derives as
`issue_210_cold_boot_idle_demo_loop_case_00000_00120`, passes with
`likely_subsystems=["none"]`, and `case reduce` reports it is already passing.
Do not patch gameplay from this clean early prefix; use this bundle as a visual
parity lead around `demo_boot@700` or capture a fresh manual title/menu route.
Follow-up 2026-06-03: ASM `Load_IntroSequence2 -> loc_1B9D0 -> loc_1B9F4 ->
loc_1BA02` accounts for an extra hidden handoff delay before the sky reveal loop.
Delaying native reveal by 18 frames reduced `demo_boot@700` from a full-screen
miss to a sparse tile/mask residual (`changed_pixel_count=129`, bounds
`5..317,3..87`) and kept `mise run verify-slice-fast` green. Strict
`verify-recording --slug issue-210-cold-idle-2600` is still red; continue from
intro-2 special sky art/mask/plane parity, not gameplay timing.
Follow-up 2026-06-03: `verify-recording` now records native VDP provenance for
sampled pixel diffs. At `demo_boot@700`, every remaining top sampled miss comes
from low-priority plane A sky/castle tiles overwriting the plane B sky/castle or
black pixel underneath: examples include plane A map `$0000` cells `0..5`
entries `$00BC..$00BE` / tiles `188..190`, source `(5,3)`, palette-line 0 colour
2, and lower rows in tiles `200..231` with colour 14/15. A one-frame later
reveal experiment regressed the frame to `2086` changed pixels, so the broad
`loc_1BA02` phase is correct; continue from ASM `sub_1C1A2/sub_1C1D2` nibble
mask or `DecompressToVRAM_Special` art parity for those plane-A nibbles.
Follow-up 2026-06-04: the delegated case-queue window around the visible
`demo_boot@700` miss also fails to produce an actionable gameplay reducer.
`case derive --recording-slug issue-210-cold-idle-2600 --frame-start 640
--frame-end 760` wrote
`issue_210_cold_boot_idle_demo_loop_case_00640_00760`; `case verify` passed with
`matching_fields=["world.game_mode","camera","entities","events"]`, and
`case reduce` reported the case is already passing. The oracle VDP sample label
`AC` was also clarified to `AC000`: it samples the Plane A nametable at `$C000`,
not the active/window candidate plane. Continue #210 from visual/VDP plane-A
boot nametable `$0000` special art/mask parity, not gameplay case reduction.
Follow-up 2026-06-03: rejected two broader #210 intro-2 hypotheses. Changing
`unk_1C336` nibble `0` from `2` to transparent `0` regressed `demo_boot@700` to
`26575` changed pixels, so ASM `DecompressToVRAM_Special`'s `0 -> 2` mapping is
required for the scene as a whole. Applying one fewer `sub_1C1A2` mask call
regressed the same checkpoint to `1077` changed pixels, so the residual is not a
simple half-frame/global mask-count lead. Continue from the specific
plane-A tile bytes/nibbles reported by `native_vdp_samples`.
Note 2026-06-03 selector follow-up for #210: selector-picked
`issue-210-chameleon-address-fix` has the same stale-later-wrapper shape. Fresh
full and native-only `verify-recording` both drift against the cached native
trace at frame 2602, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 3301-frame recorded/replay traces. The
requested `0..120` case derives as `issue_210_intro6_start_menu_case_00000_00120`,
passes with `likely_subsystems=["none"]`, and `case reduce` reports it is
already passing. Re-record this Chameleon-address wrapper on current native
before using its later title/menu tail; do not patch #210 from the clean early
prefix.
Follow-up 2026-06-03: selector-picked `issue-210-chameleon-permutation` has the
same stale-later-wrapper shape. Fresh full and native-only `verify-recording`
both drift against the cached native trace at frame 2602, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 3301-frame
recorded/replay traces. The requested `0..120` case is again
`issue_210_intro6_start_menu_case_00000_00120`, which passes with
`likely_subsystems=["none"]`; `case reduce` reports it is already passing.
Re-record the permutation wrapper on current native before using its later
title/menu tail.
Follow-up 2026-06-04: selector-picked `issue-210-intro6-start-menu` has the
same stale-later-wrapper shape, but drifts slightly earlier than the two
Chameleon-specific wrappers. Fresh full `verify-recording` and native-only
verification both fail at frame 2600 with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 3301-frame recorded/replay traces. The requested `0..120` case derives
as `issue_210_intro6_start_menu_case_00000_00120`, passes with
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this IntroSequence6
Start/menu wrapper on current native before using the later title/menu tail; do
not patch #210 gameplay from the clean prefix.
Follow-up 2026-06-04: selector-picked `issue-210-intro6-start-menu-2720` is a
malformed saved wrapper for the later menu-visible check. Fresh full and
native-only `verify-recording` both fail with `missing native frame
menu_visible@2720`, so the bundle cannot prove the later title/menu visual tail.
The requested `0..120` case still derives as
`issue_210_intro6_start_menu_case_00000_00120`, passes with the same
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this 2720 wrapper on
current native before using the menu-visible checkpoint; do not patch #210 from
the clean prefix or missing-frame artefact.
Follow-up 2026-06-04: selector-picked
`issue-210-intro6-start-menu-2720-sorted` has the stale-later-wrapper shape
rather than the missing-frame shape. Fresh full and native-only
`verify-recording` both fail at frame 2602 with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 3301-frame recorded/replay traces. The requested `0..120` case derives
as `issue_210_intro6_start_menu_case_00000_00120`, passes with
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this sorted 2720
wrapper on current native before using its later title/menu tail.
Follow-up 2026-06-04: selector-picked
`issue-210-intro6-start-menu-after-ready-delay` is also stale against current
native after the clean prefix. Fresh full and native-only `verify-recording`
both fail at frame 2600 with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 3301-frame
recorded/replay traces. The requested `0..120` case derives as
`issue_210_intro6_start_menu_case_00000_00120`, passes with
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this after-ready-delay
wrapper on current native before using its later title/menu tail.
Follow-up 2026-06-04: selector-picked `issue-210-intro6-start-menu-final`
is also stale against current native after the clean prefix. Fresh full and
native-only `verify-recording` both fail at frame 2602 with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 3301-frame recorded/replay traces. The requested `0..120` case derives
as `issue_210_intro6_start_menu_case_00000_00120`, passes with
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this final wrapper on
current native before using its later title/menu tail.
Follow-up 2026-06-04: selector-picked `issue-210-loc1bb10-title-sprite` is
also stale against current native after the clean prefix. Fresh full and
native-only `verify-recording` both fail at frame 2602 with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 3301-frame recorded/replay traces. The requested `0..120` case derives
as `issue_210_intro6_start_menu_case_00000_00120`, passes with
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this `loc1bb10`
title-sprite wrapper on current native before using its later title/menu tail.
Follow-up 2026-06-04: selector-picked `issue-210-title-frame-sega-text` is
stale against current native after the clean prefix. Fresh full and native-only
`verify-recording` both fail at frame 2602 with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 3301-frame recorded/replay traces. The requested `0..120` case derives
as `issue_210_intro6_start_menu_case_00000_00120`, passes with
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this title-frame
wrapper on current native before using the later SegaText/title-frame tail.
Follow-up 2026-06-04: selector-picked `issue-210-title-frame-widths` is stale
against current native after the clean prefix. Fresh full and native-only
`verify-recording` both fail at frame 2602 with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 3301-frame recorded/replay traces. The requested `0..120` case derives
as `issue_210_intro6_start_menu_case_00000_00120`, passes with
`matching_fields=["world.game_mode","camera","player.position","player.motion","entities","events"]`,
and `case reduce` reports it is already passing. Re-record this title-frame
widths wrapper on current native before using its later title/menu tail.
Note 2026-05-30 for #210: fixed the cold demo-2 timeout return through `Load_IntroSequence6`. Oracle shows this cold `loc_1BA5C -> $28` shell exposing `$FFD00000` and three hidden workers on frame 6850, then following `sub_1B43A`'s 3/2/4/3/4 object-count choreography, starting the +3px `GfxObjects_Collision_BottomBoundary4` camera ramp on frame 7066, and exposing `loc_1B5CE`'s title pose on frame 7148. Regression: `cold_boot_demo_timeout_intro_sequence_6_exposes_hidden_shell_on_frame_6850`. `tas-diff --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 6848 --frame-end 7152 --only-divergent --include-advisory --frontier-kind gameplay` now has 304 matches / 0 divergences, and the 9000-frame bisection advances to frame 7689.
Note 2026-05-30 for #210: extended that same cold timeout-return regression through `Obj_TitleMenu loc_1BB02 -> Load_DemoPlay`. The cold shell keeps two extra idle frames before the demo hand-off, frame 7689 now enters demo play with the controller-side `$00060000` y-velocity sample, frame 7748 exposes the hidden controller object count, and frame 7749 uses the `$FD400000` Knights Isle hidden camera row. `tas-diff --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 7684 --frame-end 7754 --only-divergent --include-advisory --frontier-kind gameplay` now has 70 matches / 0 divergences, and the 9000-frame bisection advances to frame 8851.
Note 2026-05-30 for #210: fixed the cold timeout-return demo-1 auto-timeout edge. The `Load_DemoPlay -> loc_119D8` controller sample now stays visible through frame 8852, `ReadJoypad loc_C0A` applies the mode-0 timeout hand-off on frame 8853, and that first hand-off frame preserves the final demo jump snapshot (`objects=9/4`, `Character_Movement=Jump`) instead of immediately clearing to the title shell. Regression: `cold_boot_demo_timeout_intro_sequence_6_exposes_hidden_shell_on_frame_6850` now covers frames 8851-8853. `tas-diff --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 8848 --frame-end 8875 --only-divergent --include-advisory --frontier-kind gameplay` is 27/27 match, and `verify-tas --scenario .cache/debug/issue-210/cold_boot_idle_demo_loop_long.json --frame-start 0 --frame-end 9000 --checkpoint-interval 1 --bisect --frontier-kind gameplay` passes all 9000 frames by CRC. Remaining #210 work is the visual/title/menu corruption, not this cold idle gameflow path.
Note 2026-05-30 for #210: refreshed the manual Start/menu slice (`.cache/debug/issue-210/intro6_start_menu_with_2720_sorted.json`, Start at 2600 and 2700). Native now preserves the frame-2600 Start latch until the `Load_IntroSequence6` hidden shell reaches the oracle hand-off, consumes it via ASM `sub_1C246 -> loc_1C264` at frame 2634, keeps `title_screen_ready=false` through frame 2701, reports the menu cursor as the trace-facing gfx object for this IntroSequence6 Start-skip lineage, and remains on TitleScreen at frame 3299 instead of entering demo play. Regression: `cold_boot_intro_sequence_6_start_keeps_loc_1b37c_shell_until_loc_1b5ce` plus cursor-priority coverage in `current_player_state_uses_intro_6_title_menu_only_when_active`. `verify-recording --slug issue-210-title-menu-tail` now has `trace_match=true` and `oracle_state_match=true`; `oracle_frame_match=false` remains because the pre-menu/title VDP composition still differs at frames 2599/2640/2700 and has smaller logo/copyright pixel drift at 2720/3299. Gate: `mise run verify-slice-fast`.
Note 2026-05-30 for #210: split trace readiness from VDP readiness for the same manual Start/menu slice. Oracle keeps `title_screen_ready=false` through frame 2701, but visually restores `Load_TitleScreen` by frame 2700; native now renders a blank `loc_1B45A` blackout frame at 2640 and switches the VDP shell to title art at 2700 while leaving trace state unchanged until 2720. `verify-recording --slug issue-210-title-menu-tail` still has `trace_match=true` and `oracle_state_match=true`; pixel drift improved from the full green-shell mismatch to frame 2700/2701 logo/copyright colour/detail differences. Remaining visual debt: wrong pre-handoff transformation pose at 2599-2601 and title-logo/menu phase/detail drift at 2700/2720/3299. Regression remains `cold_boot_intro_sequence_6_start_keeps_loc_1b37c_shell_until_loc_1b5ce`; gate: `mise run verify-slice-fast`.
Note 2026-05-30 for #210: the cold IntroSequence6 Start-skip title path now replays `SegaText` as soon as the inherited hidden `loc_1B6AC/loc_1B6DC` cadence reaches the visible title VDP. This matches oracle frame 2700's `© 1992 SEGA` line without changing the normal direct-title cadence (`loc_1C7A0` still hibernates once before zero-delay text). `verify-recording --slug issue-210-title-menu-tail` remains pixel-red but narrowed: trace and oracle state pass, frame 2640 pixels match, frame 2720 is down to the small `TM`/logo detail mismatch (44 changed pixels), while frames 2599-2601 still show the wrong transformation-shell silhouette and frame 3299 still has KID/logo phase drift. Regression: `title_screen_vdp_intro_return_replays_sega_text_when_vdp_becomes_visible`. Fix commit: `1834512d`. Proposed next fix: trace ASM `sub_1BF24` frame/region selection for the frame-2599 `loc_1B37C` shell, then the KID title text animation cadence around `Obj_TitleTextKid loc_1B5E4`.
Rechecked 2026-06-06 for #210: the focused manual Start/menu tail is now
mechanically clean on current main. `cargo run -q -p kidcli -- verify-recording
--slug issue-210-title-menu-tail` passes with `trace_match=true`,
`oracle_state_match=true`, `oracle_frame_match=true`, `first_trace_drift=null`,
and zero changed pixels at checkpoints 2599, 2600, 2601, 2640, 2700, 2701,
2720, and 3299. Added this bundle to `verify-human-validation` and
`.cache/human-validation/index.html` so humans can judge whether the broader
playtest report still reproduces outside this now-green slice; do not reopen
the old frame-2599/2720/3299 pixel debt without a fresh failing route.
Fresh long-route evidence 2026-06-07 for #210/#141: re-recorded
`issue-playtest-title-menu-current-long-20260607` on current main as the
replacement for stale `issue-playtest-title-menu-current-long-fixed`. Native
replay is deterministic and the transcript reaches DemoPlay at frame 3270, then
Skydragon Castle 1 at frame 3301. Follow-up 2026-06-07 fixed native trace export
for the demo `player_1_level_id` saved-slot timing artefact; full
`verify-recording` now reaches `trace_match=true`, `oracle_state_match=true`,
and `first_trace_drift=null`. It still fails `oracle_frame_match` once demo-level
pixels render (oracle checkpoints are black while native draws the level), so the
tracked gate keeps native replay plus
`issue-playtest-title-menu-current-long-20260607_case_03300_03600` semantic case
coverage rather than treating this as pixel-parity closure. `mise run
human-validation-review` links the late demo frames, state report, and case
artefacts for human review.
Note 2026-06-02 selector follow-up for #210: queued
`codex-intro-frame-probe` is not an actionable intro/title mismatch. Fresh
`verify-recording` passes with `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, and no first trace drift; recorded/replay traces are
both 523 frames. The requested `0..120` case derives as
`codex-intro-smoke_case_00000_00120` and passes with world mode, camera, player
position/motion, entities, and events matching. `case reduce` reports the case
is already passing. Do not patch #210/#161 from this clean intro prefix; use a
manual Start/options/menu route or another current failing visual case.
Follow-up 2026-06-02: selector-picked source recording `codex-intro-smoke`
has the same clean result. Fresh `verify-recording` passes with
trace/state/frame green and no first trace drift; recorded/replay traces are
both 523 frames. Its requested `0..120` case is
`codex-intro-smoke_case_00000_00120`, which passes with world mode, camera,
player position/motion, entities, and events matching; `case reduce` reports it
is already passing. This smoke route is non-actionable for #210/#161.
Follow-up 2026-06-03: selector-picked `codex-sega-logo-frame-probe` also has
the same clean shape. Fresh `verify-recording` passes with trace/state/frame
green and no first trace drift; recorded/replay traces are both 523 frames, with
enemy comparison skipped because the oracle captured no enemy snapshots. Its
requested `0..120` case is `codex-intro-smoke_case_00000_00120`, which passes
with `likely_subsystems=["none"]`; `case reduce` reports it is already passing.
This Sega-logo route is non-actionable for #210/#161.
Follow-up 2026-06-03: selector-picked `debug-current-boot-visuals` is also
clean. Fresh `verify-recording` passes with `trace_match=true`,
`oracle_state_match=true`, `oracle_frame_match=true`, and no first trace drift;
recorded/replay traces are both 2200 frames, with enemy comparison skipped
because the oracle captured no enemy snapshots. Its requested `0..120` case is
`debug-current-boot-visuals_case_00000_00120`, which passes with
`likely_subsystems=["none"]`; `case reduce` reports it is already passing. This
boot-visuals route is non-actionable for #210/#161.
Follow-up 2026-06-02: selector-picked `codex-intro-text-frame-probe` also has
the same clean shape. Fresh `verify-recording` passes with trace/state/frame
green and no first trace drift; recorded/replay traces are both 523 frames. Its
requested `0..120` case is `codex-intro-smoke_case_00000_00120`, which passes
with world mode, camera, player position/motion, entities, and events matching;
`case reduce` reports it is already passing. This text-frame route is
non-actionable for #210/#161.
Note 2026-05-30 for #211: fixed the comparison foundation for the natural `walk_right_00` route. Oracle helper now samples Kid velocity from the live Kid gfx slot (`$FFFF24E0 + x_vel/y_vel`, i.e. `$FFFF2506/$FFFF250A`) instead of the stale first gfx slot, native trace exports the same live demo velocity, and settled level-00 opening counts now match oracle RAM (`Number_Objects=4`, `Number_GfxObjects=5`). CRC-clean published-demo velocity-only skews are classified as oracle limitations in `verify-tas`, so the gate still blocks on real state/CRC drift. `tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live/scenario.json --frame-start 0 --frame-end 80 --include-advisory --frontier-kind gameplay --only-divergent` is 80/80 match. The first real natural-route divergence is now frame 124: native clears Kid `y_vel` in the right wall-cling path while oracle keeps `$00030440`, then the dragon/stomp/object-count path diverges from frame 125 onward. Gate: `mise run verify-slice-fast`.

Note 2026-05-30 follow-up for #211: narrowed the `walk_right_00` frame-124 drift to the descending right-wall cling release tail. Native now preserves the post-gravity `y_vel` through the immediate stomp window while still applying the 1px `loc_AB0A` shimmy and clearing wall-cling/x velocity. This matches frames 124-125; the next natural-route frontier is frame 126/127 `number_objects` only, where native reports the block FX object but oracle has an additional slot `0x00@0x00700000,0x00bc0000`. Verification: `cargo test -q -p kid-core wall_cling_release -- --nocapture`; `cargo test -q -p kid-core descending_wall -- --nocapture`; `cargo run -q -p kidcli -- tas-diff --scenario .cache/recordings/issue-211-level00-walk-right-live/scenario.json --frame-start 120 --frame-end 130 --include-advisory --frontier-kind gameplay --only-divergent`; `mise run verify-slice-fast`.
Note 2026-06-03 selector follow-up for #211: selector-picked
`debug-level0-dragon-stomp-probe` is stale before any current Level 0
DragonWalking stomp evidence. Fresh full and native-only `verify-recording`
both drift against the cached native trace at frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 120-frame
recorded/replay traces. The requested `0..120` case derives as
`level0_dragon_stomp_probe_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 0
dragon-stomp route on current native before using it as gameplay evidence.
Note 2026-06-03 selector follow-up for #211: selector-picked
`idle-level0-dragon` is stale before any current Level 0 DragonWalking idle
evidence. Fresh full and native-only `verify-recording` both drift against the
cached native trace at frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 121-frame
recorded/replay traces. The requested `0..120` case derives as
`idle_level0_dragon_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 0
dragon idle route on current native before using it as gameplay evidence.
Note 2026-06-03 selector follow-up for #211: selector-picked
`debug-slope-level1-oracle-settle-current` is stale before any current settled
slope gameplay evidence. Fresh full and native-only `verify-recording` both
drift against the cached native trace at frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 511-frame
recorded/replay traces. The requested `0..120` case derives as
`walk_right_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this settled
slope route on current native before using it as gameplay evidence.

Note 2026-06-06 for #211: added natural walk-right form routes for Red
Stealth/Samurai, Iron Knight, and Juggernaut/Rhino. Recorded
`issue-211-level00-slope-red-stealth-natural-20260606` with
`walk-right-red-stealth --level 00 --walk-frames 700`; captured player-layer
frames 425/428/440/500/560/620/700 to
`.cache/debug/issue211-redstealth-natural-slope-current/` and the tight
428-440 sequence to `.cache/debug/issue211-redstealth-natural-slope-tight/`.
The natural route does show visible frame-to-frame pose/position evolution
through the first slope window, unlike the stable direct mid-slope seed, but
the current native-only contact sheet is not yet an oracle-backed diagnosis of
the reported rapid flicker. Next step: run the same natural form route through
oracle/pixel comparison and inspect selected Red Stealth/Iron Knight slope
frame IDs rather than patching from native-only screenshots.

Follow-up 2026-06-06 for #211: the direct settled Red Stealth slope probe
`issue-211-level00-slope-red-stealth-current-20260606` is native-replayable but
not gameplay-authoritative against oracle. Full `verify-recording` fails at
checkpoint `slope_probe_start@0`: native starts at
`x=$02EF0000,y=$00E70000`, while oracle starts at
`x=$02F80000,y=$00E80000`. Native-only replay passes. The requested 0..120
case derives as `level_00_slope_right_probe_red_stealth_case_00000_00120`, but
`case verify` blocks with settled-recording oracle bootstrap reaching gameplay
from different player/camera/movement fields, and `case reduce` refuses for the
same reason. Do not patch slope gameplay from this direct seeded bundle; use
the natural Red Stealth route or fix settled-recording oracle bootstrap first.

Follow-up 2026-06-06 for #211: the selector-picked natural Red Stealth route
`issue-211-level00-slope-red-stealth-natural-20260606` has a clean requested
0..120 prefix. Native-only replay passes; full `verify-recording` reaches the
late `walk_end@759` checkpoint before diverging only on camera position
(`native=$037E0000,$00000000`, `oracle=$03310000,$00200000`). The requested
case derives as `walk_right_red_stealth_00_case_00000_00120` and passes with
matching `world.game_mode`, `camera`, `player.position`, `entities`, and
`events`; `case reduce` reports it is already passing. Do not patch #211 from
this clean prefix. Continue from a later natural-window case around the first
slope/pixel divergence, or investigate the late camera drift separately.

Note 2026-06-02 for #211: current `main` invalidates the old #211 cache
evidence. `verify-recording --slug issue-211-level00-walk-right-live-refresh
--native-only` drifts at frame 61, and
`verify-recording --slug issue-211-level00-slope-right-probe --native-only`
drifts at frame 0. The replacement Level 00 natural route under #126 keeps Kid
state CRC-clean through frame 900 but now exposes an earlier non-visual blocker:
frame 560 `Number_Objects/Number_GfxObjects` are `4/4` native versus `5/5`
oracle because the ROM still has the dead DragonWalking gfx/object slot
hibernating after `Dragon.asm loc_3CEFC`. Do not use either stale #211 bundle
as a slope-render proof, and do not treat the green
`slope_facing_does_not_invert_on_uphill_traversal_loc_9DC0` /
`full_motion_slope_up_right_keeps_kid_and_iron_knight_facing_right` unit tests
as live visual coverage: native still leaves normal walk/slope sprite-table
writes (`$FFFFF8F0` -> `addroffset_sprite(a3)`) modelled only as trace
accumulator state, not as selected player sprite/pixels.

Follow-up 2026-06-04 selector replay for #211: natural route
`issue-211-level00-walk-right-live` is also stale on current native. Fresh full
and native-only `verify-recording` both fail at frame 0 with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 1861-frame recorded/replay traces. The requested `0..120` case derives
as `walk_right_00_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses because
the source recording replay drifted at frame 0. Re-record a current natural
Level 00 slope route before using this bundle for #211 render/frame-selection
work.

Follow-up 2026-06-04 selector replay for #211: `issue-211-slope-frames` is
malformed/stale before oracle comparison. Fresh full and native-only
`verify-recording` both fail with `missing native frame first_slope@430`, leave
no `verify/report.json`, and only write native state/trace artefacts. The
requested `0..120` case derives as `walk_right_00_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record a current slope-frame route before using this bundle for #211
render/frame-selection work.

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

Note 2026-06-04 selector follow-up for #213: selector-picked recording
`issue-213-drips-camera-only` is stale and too short for the requested `0..120`
case. Fresh full and native-only `verify-recording` both fail at frame 0 with
`recording replay drifted at frame 0 (native trace frame mismatch)`. The cached
report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 64-frame
recorded/replay traces. `case derive --frame-start 0 --frame-end 120` refuses
because `frame_end 120 exceeds scenario frame count 64`; the valid `0..64`
case derives as `issue-213-drips-camera-only_case_00000_00064`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record a longer current Level 01 Drips/slime/metal-block route before using
this camera-only bundle as #213 evidence.

Note 2026-06-07 follow-up for #213: the durable named direct audit
`issue_213_drips_metal_block_audit_01` is confirmed to be RockTank-only
evidence, not a Drips/slime/metal-block repro. Fresh probes with
`probe-record-scenario --scenario issue213-drips-metal-block-audit --level 01`
reported both `DripsDrop: absent` and `DripsSurfaceCrawler: absent`; checkpoints
`direct_probe_start`, `pre_jump`, `jump_peak_window`, `contact_window`, and
`tail` showed only RockTank (`enemy_0A`) roster entries. `--player-motion`
also showed Kid fixed at x `$02380000`, y `$022F0000..$024F0000`, so the B-jump
route preserves a clean lower-floor blocker window but never exercises the
right Drips controller at `(584,543)`. Natural route sweeps around
`0-68:right` and `0-72:right` showed the safe route tops out around x
`$01D7E300`; inputs that cross x `$0200....` fall into `LevelTransition`.
Fresh `dump-level-layout --level 01` terrain evidence explains that boundary:
the route is on row 37 (`y=592..607`) with a gap across rows 33..36 before the
row-32 upper Drips/metal cells. Continue by building a non-death route that
climbs or otherwise crosses this gap before using #213 as Drips/block evidence;
do not patch from the named direct audit's RockTank-only checkpoints.

Follow-up 2026-06-07 route-boundary scan for #213: enhanced
`capture-scenario-frames --input` plus `probe-record-scenario-file
--player-motion` shows the durable floor-probe jump variants are still lower
shelf/fall-boundary probes, not Drips/metal-block contact. High-jump route
`0-120:right 0-100:b 120-160:left` remains live with peak height at frame 34
(`y=$01F8E000`) and max x only at frame 131 (`x=$01D11100`); the matching
farther route `0-128:right 0-100:b 128-170:left` reaches max x at frame 139
(`x=$01E11100`) but then falls into `LevelTransition` at frame 897. Captures in
`.cache/debug/issue213-boundary-capture/` and
`.cache/debug/issue213-boundary-fall-capture/` show Kid peaks near the left wall
and later walks the lower shelf toward the drop. Treat x `$01D1..$01E1` as the
current non-death/death boundary for this route family; it still does not climb
to the row-32 Drips/metal cells.

Follow-up 2026-06-07 route-sweep for #213/#260: refreshed the current healthy
prefix `.cache/recordings/issue-213-drips-delayed-b-drop-healthy-prefix-535-20260607/scenario.json`.
`verify-tas --frame-start 480 --frame-end 545 --checkpoint-interval 1
--frontier-kind enemy` still passes all 56 checkpoints, so the prefix remains
valid evidence. `probe-record-scenario-file --target-projectile
drips-surface-crawler --player-motion` shows the crawler is visible for only one
replay frame (`486`) at `$02490000,$02400000`; the preserved tail at frame 535
keeps Kid alive at `$01C81A00,$024F0000` with hp 1. A no-late-brake extension
holding Right plus a post-tail B jump falls into `LevelTransition` at frame 572
and still only sees that same one-frame crawler, so removing the late-left brake
does not make a useful Drips/metal route. Decoded `dump-level-layout --level 01`
rows 31..38 confirm the relevant geometry: the two Drips controllers are around
cells `(28,33)`/`(36,33)` on upper block rows with metal/touch cells on row 32,
while the current safe route and crawler contact sit on row 36/37 (`$00FA` floor
under `$0249,$0240`). Rechecked after adding `probe-record-scenario --input
...:none` clearing support: `--input 480-530:none --input 536-900:right --input
560-620:b` reaches the same `LevelTransition` at frame 572, with the old
tail-checkpoint already in death windup (`death_windup 38`, hp 0), so this can
now be reproduced without scratch JSON. Next route work should change the
pre-480 approach or use a different climb path; do not retry the simple
no-brake/right-jump extension.

Follow-up 2026-06-08 route-sweep for #213/#260: promoted the cleaner lower-floor
seed to named scenario `issue260-drips-cleaner-prefix-audit`
(`issue_260_drips_cleaner_prefix_audit_01`). Route inputs are Right frames
0..240, B frames 60..120, and Left frames 170..260. The route keeps Kid live
through frame 535 and sees `DripsSurfaceCrawler` at replay frames 376 and 518
instead of only the old one-frame crawler sighting at 486. Verification against
`.cache/recordings/issue-260-drips-cleaner-prefix-audit/scenario.json` passed
both enemy and gameplay frontiers across frames 360..525 with checkpoint
interval 1. This is still route evidence, not a fix: it does not reach the upper
row-32 Drips/metal cells or the reported metal-block mutation. Next work should
extend or climb from this cleaner prefix instead of patching Drips/block
behaviour from the lower-floor crawler sighting alone.

Case-queue follow-up 2026-06-09 for #213/#260: the queued
`issue-260-drips-cleaner-prefix-audit` bundle is stale on current main. Fresh
`verify-recording --slug issue-260-drips-cleaner-prefix-audit` fails with native
trace replay drift at frame 0. The generated 0..120 case in
`.cache/cases/issue-260-drips-cleaner-prefix-audit` fails, but reduction blocks
with `source recording replay drifted at frame 0`; `summary.md` classifies the
likely subsystem as `recording verification`. Re-record the cleaner prefix before
mining it as gameplay-authoritative #260 evidence.

Follow-up 2026-06-08 route-search correction for #213/#260: the current Level
01 layout window has the reported row-32 cells around cols 28/36 (`C2D`) while
the cleaner prefix remains on the lower row-37 shelf. Fresh probes from
`issue260-drips-cleaner-prefix-audit` show the post-tail extensions still do not
reach those cells: `536-620:c` + `536-700:right` stays in `InGame` through frame
700 and reaches x `$02D3`, but HP is already 0 and Kid is below the target row;
earlier C timings (`500-580:c` and `480-560:c` with right held) reach farther
right (`$02E5..$02ED`) but fall into `LevelTransition` at frames 677/661.
Control-mapping note: `FrameInput::from_mask` and `kidcli` input specs map bit
4/`b` to the jump button and bit 5/`c` to run/speed; a later direct comparison
confirmed the old B direct probe jumps while a C-only direct probe does not.
The route blocker is therefore climb/approach plus tank/gap handling, not an
input remap or Drips/block implementation signal.

Follow-up 2026-06-08 visual route check for #213/#260: native captures for the
best C-tail route (`536-620:c`, `536-700:right`) at
`.cache/debug/issue260-c-tail/` show Kid jumping onto the right RockTank around
frames 640/660, then falling through the lower gap by frame 680. Braking or
backing off after the tank (`641-720:left`) and a second C jump (`650-690:c`,
`650-720:left`) still fall into `LevelTransition` around frames 705/709. The
tank can explain the HP-zero/death tail, but it has not produced a usable climb
to the row-32 Drips/metal cells.

Follow-up 2026-06-08 direct input sanity check for #213/#260: a temporary
direct C-only probe from the existing upper-cell seed (`x=568,y=575`, camera
`240,352`, frames 0..80 holding `c`) did not jump, did not spawn a
`DripsSurfaceCrawler`, and settled back at `$0238,$024F` by frame 11. The
existing `issue213-drips-metal-block-audit` B route from the same seed does
jump (`jump_peak_window@380` at `$0238,$0238`) but still does not spawn the
Drips crawler or hit the reported metal-block mutation. Next #260 route work
should use B for jump and search a real approach/climb to row 32, not add a
C-jump scenario.

Follow-up 2026-06-08 B-jump climb seed for #213/#260: promoted the first live
post-cleaner-prefix climb branch to named scenario
`issue260-drips-b-jump-climb-seed`
(`.cache/recordings/issue-260-drips-b-jump-climb-seed/scenario.json`). Inputs
reuse the cleaner prefix, then hold B+Right from frames 480..560 and Right
through frame 700. Native remains live through frame 700 (`kid=$03E19700,$024F`)
and frame captures in `.cache/debug/issue260-b-tail-live/` show Kid crossing the
RockTank/gap area and entering the rising right-hand tunnel. Gameplay oracle
verification passes all frames 360..700 by CRC. Enemy-frontier verification now
first diverges at frame 603 on the RockTank roster: native has layout #01 half a
pixel left (`$00C5/$00C5.8`) and ground/debug-field differences versus oracle
`$00C6`, while Drips/crawler sightings remain only the two earlier frames
376/518. This is a better route seed, not a metal-block repro yet; next work
should either classify/fix the frame-603 RockTank roster divergence or extend
from the CRC-clean gameplay route without assuming enemy-frontier object parity.

Case-queue follow-up 2026-06-09 for #213/#260: the queued
`issue-260-drips-b-jump-climb-seed` recording is not currently suitable for
case reduction. Fresh `verify-recording --slug issue-260-drips-b-jump-climb-seed`
fails with `missing native frame b_jump_climb_start@480`. Deriving the requested
0..120 case to `.cache/cases/issue-260-drips-b-jump-climb-seed` produces
`issue_260_drips_b_jump_climb_seed_01_case_00000_00120`, but `case reduce`
blocks it with `source recording replay drifted at frame 0`; `summary.md`
classifies the likely subsystem as `recording verification`. Re-record or
repair the climb-seed bundle before using it as gameplay-authoritative evidence.

Follow-up 2026-06-08 route-search note for #213/#260: post-climb and earlier
vertical setup overlays still do not reach the reported row-32 Drips/metal-block
interaction. A live negative variant
(`430-1000:none`, `450-660:right`, `450-560:b`, `660-820:left`,
`780-850:b+left`) stays `InGame` through frame 1000, keeps Drips layouts #7/#8
active, and emits `DripsDrop` through frame 913, but native captures in
`.cache/debug/issue260-live-negative-variant/` show Kid landing back on the
bottom floor and walking left; max x is `$03D94700` at frame 671 and final Kid
is `$027B7000,$024F0000`. Short-hop/shelf attempts that clear inputs from frame
420 and jump/right at 420, 430, or 440 all enter `LevelTransition` by frames
587, 599, or 706, while vertical-hop/right-tap/left-wall variants die by
frames 671..677. Do not retry these exact early B-jump shelf timings; next route
work needs a different vertical-position axis or a different approach to the
row-32/row-20 block corridor.

Follow-up 2026-06-08 wall-cling route rejection for #213/#260: clearing the
cleaner prefix from frames 140..300 and reapplying Left on 170..260 reliably
creates a near-hit on the lower wall-cling branch, not the reported slime/metal
interaction. The probe reaches target point `(456,543)` within 2px at frame 496,
but `kid_wall_cling_release_probe` shows the top edge blocked by the Prize block
at cell 29,32 (`x=464`) on frames 504..506, then releases/falls right by frame
507. Post-contact overlays (`500-560` Left, Left+B, B, Right+B, or clear/right
tails through 620) keep Kid live but still miss target `(584,543)` by 48px
vertically; the best DripsDrop closest approach is still only manhattan 40..45px
and no fresh surface-crawler/metal-block mutation appears. Longer right-only
tailing reaches `LevelTransition` at frame 857, while right+B stays live but
does not climb. Do not retry this cell-29/32 Prize-block wall-cling branch
unless the target route changes; #260 still needs a new vertical approach to the
row-32 Drips/metal cells.

Follow-up 2026-06-08 early-jump route sweep for #213/#260: clearing the
post-cleaner tail from frame 360 and trying B+Right windows starting at
360/380/400/420/440/460 narrows the target approach but still does not yield a
usable repro. The 380-start family gets as close as 19px to `(584,543)` at frame
408, the 420-start family gets 30px at frame 449, and the 440/460-start families
get 23px at frames 465/485; all of those runs later enter `LevelTransition`
between frames 587 and 758 or hit death windup after passing the lower enemy
cluster. The 360-start family dies earlier and never improves on the old 48px
vertical miss. Treat these early B+Right timing families as route-search
evidence only: they improve proximity to the Drips point, but still do not
produce a live row-32 metal-block mutation or crawler contact.

Follow-up 2026-06-08 early-jump Drips timing audit for #213/#260: tight
`tas-diff` on
`.cache/recordings/issue-260-drips-early-jump-close-probe/scenario.json` frames
386..392 shows the first authoritative mismatch at frame 388. Native has already
created a `DripsSurfaceCrawlerBurst@0x0244e000,0x023c2000` with
`count_loc_fffffb42=1`, while oracle still has both Drips controllers in the
`Drips.asm loc_3345C`/`loc_33488` coroutine (`code=0x033460` for both at frame
388; the left controller only returns through `loc_33428` on frames 390..391).
The later frame-403 native stomp/bounce on that burst helper is therefore a
downstream symptom, not evidence for changing generic stomp priority or
projectile damage. Rejected hypothesis: "fix the 403 bounce by changing
DripsSurfaceCrawlerBurst collision order". Next implementation work should
target why native's right Drips reaches `loc_335D8`/`loc_338F8` before oracle,
checking the linked-child Y offset and splash/table countdown around
`loc_33424`, `loc_3345C`, `loc_33488`, and `off_33520`.

Follow-up 2026-06-08 Drips reseed probe for #260: native instrumentation
`drips_splash_reseed_probe` shows the first right-Drips reseed at frame 125:
`#08 table:0->1 crawler=0 kid_x=0x023f reseed_rng=0x58 wait=0x0029`. Oracle is
not in the `loc_33488` splash/table path there; `oracle_drips_object_slots`
shows the controller still hibernating at `code=0x001066` after the
`loc_33424` animation setup, with `a0=0x00033520`. By frame 160 native is
counting down the next Drips cycle, while oracle is only in the falling-child
loop `loc_3345C`; by frame 181 native right Drips is already in
`pre_drop:0x8016` and oracle right Drips is still in `loc_33428` with
`d0=0x9c`, `a0=0x00033522`. Revised hypothesis: the crawler/table drift is
downstream of native advancing the first drop/splash cycle while the ROM is
still inside the `loc_33424` animation hibernate/falling-child handoff. Next
work should compare native `step_drips` wait/pre-drop phase boundaries against
ASM `GfxObject_Animate` + `Hibernate_UntilAnimFinished` before changing crawler
RNG or stomp/contact rules.

Follow-up 2026-06-09 Iron Knight route rejection for #213/#260: added the named
scenario `issue260-drips-iron-knight-climb-probe` to test whether seeding Iron
Knight on the cleaner lower-floor Drips prefix opens the row-32 approach. Probe:
`probe-record-scenario --scenario issue260-drips-iron-knight-climb-probe
--level 01 --target-point 584,543 --player-motion --summary-only`. Native stays
`InGame` through frame 536 with healthy Iron Knight (`lives/hp=3/4`), but it
stalls around `kid=$0216FA00,$024F0000`; closest approach to `(584,543)` is
manhattan 49px at frame 128 (`dx=+1, dy=-48`), worse than the base-Kid B-jump
seed's 23px miss. Treat this form seed as rejected route-search evidence, not
an implementation signal.

Follow-up 2026-06-09 Drips timing correction for #213/#260: fresh current-main
oracle checks show the older `loc_33424`/`loc_33488` timing suspicion is stale
for the checked-in early-jump probe. `tas-diff --scenario
.cache/recordings/issue-260-drips-early-jump-close-probe/scenario.json
--frame-start 120 --frame-end 130 --include-advisory --frontier-kind enemy
--only-divergent` reports 10/10 matching frames, and the documented frame
386..392 window now reports 6/6 matching frames. Do not start the next #260
session by patching Drips timing from those older notes; the active blocker is
still finding or recording a route that reaches the row-32 Drips/metal-block
interaction.

Note 2026-06-04 selector follow-up for #213: selector-picked recording
`issue-213-drips-floor-probe-512` is stale before any current Drips floor/slime
comparison. Fresh full and native-only `verify-recording` both fail at frame 0
with `recording replay drifted at frame 0 (native trace frame mismatch)`. The
cached report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 512-frame
recorded/replay traces. The requested `0..120` case derives as
`issue-213-drips-floor-probe-512_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record a current
Level 01 Drips floor/slime route before using this bundle as #213 evidence.

Note 2026-06-04 selector follow-up for #213: selector-picked recording
`issue-213-nondeath-x536` is stale before any current non-death Drips route
comparison. Fresh full and native-only `verify-recording` both fail at frame 0
with `recording replay drifted at frame 0 (native trace frame mismatch)`. The
cached report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 901-frame
recorded/replay traces. The requested `0..120` case derives as
`issue213_non_death_probe_x536_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Treat the cached
`issue-213-` recording family as stale selector input until a fresh current
Level 01 Drips/slime/metal-block route is recorded.

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

Note 2026-06-03 follow-up for #213: selector-picked recording
`full-frame-level01-idle-grounded-scan` is stale before any current Level 01
idle/grounded scan evidence. Fresh full and native-only verification both drift
at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 561-frame
recorded/replay traces. The requested 0..120 case derives as
`idle_level_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 01
idle/grounded scan route on current native before using it as gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`full-frame-level01-idle-scan-branch` is stale before any current Level 01 idle
branch-scan evidence. Fresh full and native-only verification both drift at
native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 561-frame recorded/replay traces. The
requested 0..120 case derives as `idle_level_01_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this Level 01 idle branch-scan route on current native before using it
as gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`full-frame-level01-idle-tank-height` is stale before any current Level 01 idle
Tank-height scan evidence. Fresh full and native-only verification both drift at
native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 561-frame recorded/replay traces. The
requested 0..120 case derives as `idle_level_01_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this Level 01 idle Tank-height route on current native before using it
as gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`full-frame-level01-idle-tank-width` is stale before any current Level 01 idle
Tank-width scan evidence. Fresh full and native-only verification both drift at
native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 561-frame recorded/replay traces. The
requested 0..120 case derives as `idle_level_01_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this Level 01 idle Tank-width route on current native before using it
as gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level01-walk-right-frontier-3600` is also stale before any current
Level 01 walk/Drips gameplay evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 3661-frame
recorded/replay traces. The requested 0..120 case derives as
`walk_right_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 01
walk-right frontier route on current native before using it as Drips gameplay
evidence.

Note 2026-06-02 follow-up for #213: `tmp-issue-213-drips-current` is only
usable for its first 120 frames. Full and native-only `verify-recording` drift
at frame 135, but the selector-requested 0..120 case derives as
`issue-213-drips-floor-probe-2000_case_00000_00120` and passes with matching
mode, camera, player position, entities, and events. `case reduce` reports the
case is already passing. This confirms the latest lower-floor prefix remains
clean, but it still does not cover the later drop/pool/metal-block interaction;
re-record a stable route past frame 135 before closing #213.

Note 2026-06-02 latest for #213: rechecked current `main` after the playtest
complaint and kept the item blocked, not closed. Focused Drips coverage remains
green (`cargo test -q -p kid-core drips_ -- --nocapture`: 109 passed), and the
only cache-backed current case still green is the 64-frame camera prefix
(`cargo run -q -p kidcli -- case verify --case issue-213-drips-camera-only`:
PASS). The longest candidate bundle still cannot reach the reported
terrain/jump/metal-block interaction because `cargo run -q -p kidcli --
verify-recording --slug tmp-issue-213-drips-current --native-only` drifts at
frame 135 with a native trace mismatch. Continue by re-recording a stable
settled Level 01 route past frame 135 before patching Drips or metal-block
collision behaviour from this report.

Note 2026-06-02 follow-up for #213: promoted the settled Level 01 Drips
floor probe out of `.cache` to
`tests/data/dynamic-scenarios/issue-213-drips-floor-probe-2000.json` so the
oracle evidence is durable. On current `main`, `verify-tas --frontier-kind
enemy --bisect` passes frames 0..900 and 900..2000, proving the hidden
controller, `loc_33424` child countdown, `loc_3345C` floor splash, and long
idle/crawler-visible window are not currently diverging in this focused setup.
This does not close #213: the human report is specifically about jumping up
near the lower-floor slime/metal-block area and hitting/breaking unexpected
collision, so the next gate is a longer checked-in DynamicScenario that drives
that upward-jump contact against oracle.

Note 2026-06-02 follow-up for #213: the first failing upward-jump direct probes
were blocked by RockTank child-contact fidelity, not Drips. ASM
`GfxObjects_CollisionKid` indexes collision boxes from `$22` /
`CollisionSize_Index`: Tank body frames `C7ED0/C7ED8/C7EE0` map to zero
collision, while linked mouth/turret frames `C7EC8`/`C7F20` map to
`unk_31A26`/`unk_31A54` (`dc.w -$D,$19,-$10,$10`). Native contact damage now
tests the linked mouth/turret child for `$0A/$0B`, and death-tumble trace export
now reports object-field velocities (`0`) instead of local `Death loc_B80C`
`d0/d1` deltas. Verification: `index8_jump_now` passes enemy-frontier
`verify-tas` through frames 0..420; `index8_jump_right_now` no longer diverges
at the old frame-312 HP latch or frame-314 velocity mismatch and first fails at
frame 409 `kid_x_pos`, a later death-tail lifetime issue. `mise run
verify-slice-fast` passed. #213 remains open for a durable upward-jump /
metal-block Drips route.

Note 2026-06-02 follow-up for #213: a fresh rightward jump probe from the
checked-in lower-floor Drips scenario found the first human-visible invisible
collision bug. Native damaged Kid at frame 129 from internal `enemy_16#08`,
while oracle kept HP unchanged and listed the Drips object as a hidden
`loc_33424`/`loc_33488` controller slot (`code=0x001066`). Native contact now
skips attached Drips parents/controllers; the falling child and `loc_335D8`
surface crawler remain hostile through typed projectiles. Verification:
`cargo test -q -p kid-core attached_drips_parent_does_not_contact_damage_kid_loc_33424 -- --nocapture`,
`cargo test -q -p kid-core drips_ -- --nocapture`, and cache-local
`verify-tas` for `issue-213-drips-upjump-right-probe-cache` frames 0..180 all
pass. The same probe now first diverges at frame 269 on RockTank contact
(`enemy_0A#10`), so #213 is still open for the later metal-block/slime route
and that tank-contact blocker should be handled separately.

Note 2026-06-02 follow-up for #213: fixed the compact direct jump probe's
death-tail trace blocker. ASM `lose_life -> Teleport loc_B8DE -> loc_6E2`
seeds different hidden `loc_1AC70` co-ordinates on the camera-$026F direct
Drips jump route than on the older wrapped-row route: oracle frames 427..428
show `$0130,$0100` then `$012C4000,$00FDA000`, while
`issue-213-drips-upjump-right-probe-cache` still needs the `$88,$FFE0`
bootstrap. Native now preserves a route-specific trace profile across the
respawn reset; cache-local
`.cache/scenarios/codex-issue-213/index8_jump_right_now.json` passes
enemy-frontier `verify-tas` through frames 0..1000. The route still does not
close #213 because it is a death/hidden-respawn reducer, not the reported live
slime/metal-block contact; next route should avoid the death tail and drive the
upward Drips/metal-block interaction directly.

Note 2026-06-02 follow-up for #213: route reduction after the latest playtest
confirmed the stale-cache jump scripts are the wrong shape for the live
slime/metal-block report. The old `x=$0248,y=$024F` / `x=584,y=591` probes
(`index8_wait620_jump_right`, `index8_wait760_jump_right`,
`index8_wait620_jump`) all collapse into the death/hidden-respawn lineage and
first diverge around frame 508 on `kid_x_pos`; discard them as live Drips
repros. The clean durable baseline at `x=344,y=575` passes wait-620 and
wait-760 C-jump variants through frame 1100, but trace rows show Kid remains
far left while Drips drops/crawlers appear near `x=584,y=543`, so it does not
exercise contact. The closer floor height is `x=584,y=575`: idle, immediate
C-jump, and frame-100 C-jump pass, but a frame-370 C-jump re-enters the same
death-tail. An X sweep at `y=575`, wait-370, C-jump passed for
`x=536/552/568/576` through frame 520 and entered death/respawn for
`x=584/592/600/616` before any usable live slime/metal-block mismatch. A
layout dump around the lower-floor probe did not show decoded block-type
`Metal` cells in that area, so do not assume the playtest "metal block" is a
type-5 block until the exact route proves the mutated cell. Next #213 work
needs a non-death route from the correct floor height that reaches live
upward-jump contact, or should move to another repeated blocker rather than
patching Drips from these reducers.

Note 2026-06-02 follow-up for #213: the B-button upward jump route
`.cache/scenarios/codex-issue-213-jump-b/issue213-x568-y575-w340-jump_b_hold-1200.json`
showed a real plain RockTank nonfatal-hit timing mismatch before the later
Drips/slime area. Native reused generic `damage_flash_frames` for
`Tank.asm loc_3B49A`, so the parent resumed and the mouth child reported its
normal y-offset before oracle's hibernated/recoil window. Native now keeps a
separate plain Tank `wait_timer` for the loc_3B49A hibernate/recoil trace
window while TankShooting keeps its existing `damage_flash_frames` path.
Focused Tank tests pass, but the route then exposed a terminal parent/gfx
handoff at frame 453: native reached the delete tail one scheduler frame
before oracle's `Hibernate_UntilAnimFinished loc_1062`, and oracle then kept a
stale displayed GfxObject whose parent object was no longer in the active
`Execute_Objects loc_E1E` list. Native now keeps the Tank `stru_3B29A` death
parent alive for that terminal scheduler return, and enemy/case oracle
normalisation ignores displayed GfxObjects whose parent object has already
been unlinked by `Deallocate_GfxObject loc_10C6`. Verification:
`issue213-x568-y575-w340-jump_b_hold-1200.json` is enemy-frontier clean through
frames `0..560`, `560..700`, and `700..1200`.

Note 2026-06-07 follow-up for #213: selector-picked recording
`issue213-current-existing-x568-w340-b-hold-native` is stale under current
native (`verify-recording` drifts at frame 368), so I re-recorded the same
scenario as `issue213-current-rerecord-x568-w340-b-hold-native-20260607`.
The fresh recording is native-replayable (`trace_match=true`) but full oracle
verification still fails at `pre_jump@339` on object/gfx counts (`native 7/7`,
`oracle 5/8`) and frame output. The reduced `300..360` case initially blocked
on case-tool semantics: the helper dropped a live zero-HP RockTank child when
the same oracle capture contained its non-zero-HP body sibling. Case semantics
now preserve those `Tank.asm loc_3B31E` / shared-walker `loc_3B40A` body-child
pairs, and `case verify --case
issue213-current-rerecord-x568-w340-b-hold-native-0300-0360` passes. The next
reduced window, `issue213-current-rerecord-x568-w340-b-hold-native-0320-0440`,
then isolated a real `Tank.asm loc_3B49A` child recoil phase bug: first bad
frame 363, field `entities`, with player/camera/events matching and only the
layout-9 RockTank child Y one frame ahead in native (`$024A0000`) versus
oracle (`$02480000`); oracle reached the same child Y at frame 364.
Follow-up fixed the exported plain RockTank recoil window to keep child
`$46=8` through the oracle-confirmed pre-recoil frame and expose `$46=6` from
the next frame. The pre-fix 0320..0440 case now correctly blocks as stale at
frame 363, so I re-recorded current native as
`issue213-current-rerecord-x568-w340-b-hold-native-20260607-post-tank-recoil`,
derived `issue213-current-rerecord-x568-w340-b-hold-native-post-tank-recoil-0320-0440`,
and that refreshed case passes. `issue213-current-rerecord-x568-w340-b-hold-native-0300-0360`
still passes. Continue #213 by finding a non-death upward-contact route that
actually reaches the reported Drips/slime/metal-block interaction; do not
patch Drips from the superseded Tank child case.

Note 2026-06-07 follow-up for #213: autonomy selected the stale
`issue213-current-existing-x568-w340-jump-right-b-native` recording; current
native replay drifts at frame 363, so I re-recorded the same scenario as
`issue213-current-rerecord-x568-w340-jump-right-b-native-20260607-post-tank-recoil`
and derived
`issue213-current-rerecord-x568-w340-jump-right-b-native-post-tank-recoil-0320-0440`.
The refreshed reduced case is gameplay-authoritative again and fails at frame
383 on oracle `spawned_entity`: the oracle helper sees a transient Drips
GfxObject for layout 7 at `$0230,$0202` one frame before native's hidden
controller becomes semantically visible at frame 384 from layout-anchor
`$01C8,$021F`. `active_enemy_details` now exposes hidden native enemy
controllers to case semantics without breaking legacy recording replay
comparison, so future Drips reducers can distinguish "not exported" from
"not spawned". This does not fix #213; next pass should trace
`Drips.asm Enemy16_Drip_Init`'s linked GfxObject position/hibernate timing
against the oracle helper before changing loader timing.

Note 2026-06-02 follow-up for #213: retired the delegated case-queue
`walk_right_01_case_00000_00600` entries as stale recording blockers, not
current RockTank/Drips bugs. Fresh verifies for
`codex-rocktank-contact-level01-walk-right-0000-0600` and
`codex-rocktank-contact-level01-walk-right-edge-0000-0600` both block before
oracle comparison because their source recordings drift at native replay frame
0; `codex-level01-walk-right-shared-walker-fall-0000-0600` has the same stale
source-recording blocker. Re-recording the same DynamicScenarios as
`codex-rocktank-contact-level01-walk-right-current-1800` and
`codex-rocktank-contact-level01-walk-right-current-1800-edge`, plus
`codex-level01-walk-right-shared-walker-fall-current-1800`, deriving
`codex-rocktank-contact-level01-walk-right-current-0000-0600` and
`codex-rocktank-contact-level01-walk-right-current-edge-0000-0600`, and
`codex-level01-walk-right-shared-walker-fall-current-0000-0600`, then running
`case verify` passes all three full 0..600 windows. Do not patch RockTank
HP/contact, Level 01 slope state, or shared-walker falling from the stale
cases; continue #213 with a new non-death upward-contact route that actually
reaches the live Drips/slime/block interaction.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level01-walk-right-shared-walker-fall-current-1800` still is not a
new failing gameplay case. Fresh full verification is native-replayable
(`trace_match=true`, no trace drift, 1861 recorded/replay frames) but fails the
recording checkpoint at `level_settle@60` with object counts `native 4 / 4,
oracle 4 / 6`. Native-only replay passes. The requested 0..120 case derives as
`walk_right_01_case_00000_00120`, `case verify` passes with
`likely_subsystems=["none"]`, and `case reduce` reports the case is already
passing. Do not patch shared-walker falling, RockTank, or Drips behaviour from
this selector item; it exposes only the known settled Level 01 oracle bootstrap
gap plus a clean early gameplay prefix.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-rocktank-contact-level01-walk-right-current-1800` is likewise not a new
failing RockTank contact case. Fresh full verification replays natively
(`trace_match=true`, no trace drift, 1861 recorded/replay frames) but fails the
recording checkpoint at `level_settle@60` with object counts `native 4 / 4,
oracle 4 / 6`. Native-only replay passes. The requested 0..120 case derives as
`walk_right_01_case_00000_00120`, `case verify` passes with
`likely_subsystems=["none"]`, and `case reduce` reports the case is already
passing. Do not patch RockTank contact or Level 01 movement from this selector
item; it only repeats the known settled Level 01 oracle bootstrap gap.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-rocktank-contact-level01-walk-right-current-1800-edge` is the same
non-actionable edge variant. Fresh full verification replays natively
(`trace_match=true`, no trace drift, 1861 recorded/replay frames) but fails the
recording checkpoint at `level_settle@60` with object counts `native 4 / 4,
oracle 4 / 6`. Native-only replay passes. The requested 0..120 case derives as
`walk_right_01_case_00000_00120`, `case verify` passes with
`likely_subsystems=["none"]`, and `case reduce` reports the case is already
passing. Do not patch RockTank contact, edge movement, or Level 01 movement from
this selector item; it only repeats the known settled Level 01 oracle bootstrap
gap.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level01-walk-right-shared-walker-fall-pixel-cross-1800` is stale before
any shared-walker or Level 01 Drips gameplay evidence. Fresh full and
native-only verification both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1861-frame recorded/replay traces. The requested 0..120 case derives
as `walk_right_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this
pixel-cross shared-walker route on current native before using it as Level 01
gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level2-drips-crawler-window` is stale before any current Drips/crawler
gameplay evidence. Fresh full and native-only verification both drift at native
replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 231-frame recorded/replay traces. The
requested 0..120 case derives as
`codex-level2-drips-crawler-window_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
Level 2 Drips/crawler window on current native before using it as gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level2-drips-drop-window` is stale before any current Drips/drop gameplay
evidence. Fresh full and native-only verification both drift at native replay
frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 301-frame recorded/replay traces. The
requested 0..120 case derives as
`codex-level2-drips-drop-window_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 2
Drips/drop window on current native before using it as gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level2-drips-near-pool` is stale before any current Drips/pool gameplay
evidence. Fresh full and native-only verification both drift at native replay
frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 241-frame recorded/replay traces. The
requested 0..120 case derives as
`codex-level2-drips-near-pool_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 2
Drips near-pool route on current native before using it as gameplay evidence.

Note 2026-06-03 follow-up for #169: selector-picked recording
`issue-169-drips-probe-near-pool` is stale before any current Level 2
Drips/pool gameplay evidence. Fresh full and native-only verification both
drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 420-frame
recorded/replay traces. The requested `0..120` case derives as
`issue-169-drips-probe-near-pool_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this #169
near-pool Drips route on current native before using it as gameplay evidence.

Note 2026-06-03 follow-up for #169: selector-picked recording
`issue-169-drips-probe-near-resource-pool` is stale before any current Level 2
Drips/resource-pool gameplay evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 540-frame
recorded/replay traces. The requested `0..120` case derives as
`issue-169-drips-probe-near-resource-pool_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
#169 resource-pool Drips route on current native before using it as gameplay
evidence.

Note 2026-06-03 follow-up for #169: selector-picked recording
`issue-169-level2-drips` is stale before any current Level 2 Drips gameplay
evidence. Fresh full and native-only verification both drift at native replay
frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1861-frame recorded/replay traces. The
requested `0..120` case derives as `walk_right_01_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this #169 Level 2 Drips route on current native before using it as
gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level2-walk-right-rocktank` is stale before any current Level 2 RockTank
or walk-right gameplay evidence. Fresh full and native-only verification both
drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 961-frame
recorded/replay traces. The requested 0..120 case derives as
`walk_right_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 2
RockTank walk-right route on current native before using it as gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`debug-rocktank-level2-oracle-settle-current` is stale before any current Level
2 RockTank oracle-settle gameplay evidence. Fresh full and native-only
verification both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1261-frame
recorded/replay traces. The requested 0..120 case derives as
`walk_right_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 2
RockTank oracle-settle route on current native before using it as gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-rocktank-level2-near` is stale before any current Level 2 near-RockTank
gameplay evidence. Fresh full and native-only verification both drift at native
replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 150-frame recorded/replay traces. The
requested 0..120 case derives as `codex-rocktank-level2-near_case_00000_00120`,
but `case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this Level 2 near-RockTank route on current native before using it as
gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-rocktank-level2-near-camera` is stale before any current Level 2
near-RockTank camera gameplay evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 150-frame
recorded/replay traces. The requested 0..120 case derives as
`codex-rocktank-level2-near-camera_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Re-record this Level 2
near-RockTank camera route on current native before using it as gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-level3-walk-right-shooters` is stale before any current Level 3 shooter
gameplay evidence. Fresh full verification warns that oracle captured no enemy
snapshots, then full and native-only verification both drift at native replay
frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1261-frame recorded/replay traces. The
requested 0..120 case derives as `walk_right_02_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this Level 3 shooters walk-right route on current native before using
it as gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-drips-floor-probe-2000-promoted` is not a gameplay-authoritative
case source on current `main`. The old recording drifts at native replay frame
0. Regenerating the same scenario as
`codex-issue-213-drips-floor-probe-2000-promoted-refresh` passes native-only
replay, but full recording verification still hits the known mid-route object
count mismatch (`mid@1000` native `5/5`, oracle `5/9`), and the requested
0..120 case is blocked because settled-recording oracle bootstrap starts Level
01 with different velocity/entity metadata. Continue using the checked-in
DynamicScenario `tests/data/dynamic-scenarios/issue-213-drips-floor-probe-2000.json`
for oracle TAS verification; do not patch Drips from the recording-derived case
until settled-recording bootstrap is fixed or a new non-death live route is
recorded.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-now-trace` exposed verifier debt before gameplay
evidence. The scenario has two labels at frame 0 (`start` and `jump`), and
`verify-recording` previously consumed only one checkpoint per frame, reporting
`missing native frame jump@0`; native/oracle capture already support multiple
same-frame checkpoints. After fixing the recording verifier to consume all
same-frame labels, regenerating as
`codex-issue-213-index8-jump-now-trace-refresh` reaches real comparison:
trace/state match, recording PNG checkpoints are pixel-red from frame 0, and
the requested 0..120 case is blocked by the settled Level 01 oracle bootstrap
(`player fields=x_velocity|y_velocity`, native 2 entities vs oracle 1). Do not
patch Drips/slime logic from this route; fix settled-recording bootstrap or
record a new authoritative non-death upward-contact route first.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-right-after-tank-child` is the same class of
blocked evidence. The old bundle drifts at native replay frame 0, but
regenerating the saved scenario as
`codex-issue-213-index8-jump-right-after-tank-child-refresh` reaches oracle
comparison with clean trace/state and pixel-red recording checkpoints from frame
0. The requested 0..120 case derives but is blocked by settled Level 01 oracle
bootstrap (`player fields=x_velocity|y_velocity`, native 2 entities vs oracle
1), so this route cannot drive Drips/slime or Tank-child patches. Fix
settled-recording bootstrap or record a fresh authoritative route before using
these index8 jump recordings as gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-right-now-trace` matches the sibling
`jump-right-after-tank-child` result. The old bundle drifts at native replay
frame 0; the refreshed bundle reaches oracle with clean trace/state but
pixel-red recording checkpoints from frame 0; the 0..120 derived case is
blocked by the same settled Level 01 oracle-bootstrap mismatch (`x_velocity`,
`y_velocity`, native 2 entities vs oracle 1). Treat these index8 right-jump
recordings as bootstrap/tooling evidence only, not gameplay-authoritative Drips
or Tank-child evidence.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-death-current` is stale at native replay frame 0. A
refresh from its saved `issue213-x568-y575-w340-jump_b_hold-1200` scenario
matches native trace but is not oracle-authoritative: full recording
verification diverges at `pre_jump@339` on object counts (native `7/7`, oracle
`5/8`), and the requested 0..120 case is blocked by the settled Level 01
oracle-bootstrap mismatch (`x_velocity`, `y_velocity`, native 2 entities vs
oracle 1). Do not patch Tank/RockTank death or Drips contact from this recording
until settled-recording bootstrap is fixed or a fresh authoritative final-hit
route is captured.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-hit-after-7d4e2500` is the same saved
`issue213-x568-y575-w340-jump_b_hold-1200` route as the tank-death candidate.
The old bundle drifts at native replay frame 0. The refreshed bundle matches
native trace but diverges from oracle state at `pre_jump@339` on object counts
(native `7/7`, oracle `5/8`), and the 0..120 case is blocked by the settled
Level 01 oracle-bootstrap mismatch. Treat this as duplicate bootstrap/tooling
evidence, not gameplay-authoritative Tank/Drips evidence.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-hit-current` also uses the same saved
`issue213-x568-y575-w340-jump_b_hold-1200` scenario. Direct verification of the
old bundle drifts at native replay frame 0. The scenario-equivalent refreshed
evidence is already covered by
`codex-issue-213-tank-hit-after-7d4e2500-refresh` and
`codex-issue-213-tank-death-current-refresh`: native trace refreshes, oracle
object counts diverge at `pre_jump@339`, and the reduced 0..120 case is blocked
by settled Level 01 bootstrap. Do not re-run or patch from this duplicate slug.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x536-w370-jump-right-native-scan` is a distinct
`issue213-x536-y575-w370-jump_right_c-1200` scan, but it is still blocked by
settled Level 01 bootstrap rather than Drips/Tank behaviour. The old bundle
drifts at native replay frame 0. Regenerating the saved scenario as
`codex-issue-213-x536-w370-jump-right-native-scan-refresh` matches native trace
but diverges from oracle at `pre_jump@369` on object counts (native `7/7`,
oracle `5/8`), and the requested 0..120 case is blocked by the same
`x_velocity`/`y_velocity` and entity-count bootstrap mismatch. Do not patch
gameplay from this scan until the settled-recording oracle bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s320-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x536-s320-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x536-s320-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale bundle; re-record the route on
current native if this x536/s320 timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s320-rb` is the same stale x536/s320 family with right-held
input. Fresh `verify-recording --slug issue213-js-x536-s320-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s320-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s320-rbc` is the same stale x536/s320 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x536-s320-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s320-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s340-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x536-s340-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x536-s340-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x536/s340 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s340-rb` is the same stale x536/s340 family with right-held
input. Fresh `verify-recording --slug issue213-js-x536-s340-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s340-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s340-rbc` is the same stale x536/s340 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x536-s340-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s340-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s350-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x536-s350-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x536-s350-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x536/s350 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s350-rb` is the same stale x536/s350 family with right-held
input. Fresh `verify-recording --slug issue213-js-x536-s350-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s350-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s350-rbc` is the same stale x536/s350 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x536-s350-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s350-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s360-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x536-s360-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x536-s360-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x536/s360 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s360-rb` is the same stale x536/s360 family with right-held
input. Fresh `verify-recording --slug issue213-js-x536-s360-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s360-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s360-rbc` is the same stale x536/s360 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x536-s360-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s360-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s370-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x536-s370-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x536-s370-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x536/s370 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s370-rb` is the same stale x536/s370 family with right-held
input. Fresh `verify-recording --slug issue213-js-x536-s370-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s370-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x536-s370-rbc` is the same stale x536/s370 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x536-s370-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x536-s370-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s320-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x544-s320-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x544-s320-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x544/s320 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s320-rb` is the same stale x544/s320 family with right-held
input. Fresh `verify-recording --slug issue213-js-x544-s320-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s320-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s320-rbc` is the same stale x544/s320 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x544-s320-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s320-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s340-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x544-s340-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x544-s340-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x544/s340 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s340-rb` is the same stale x544/s340 family with right-held
input. Fresh `verify-recording --slug issue213-js-x544-s340-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s340-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s340-rbc` is the same stale x544/s340 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x544-s340-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s340-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s350-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x544-s350-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x544-s350-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x544/s350 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s350-rb` is the same stale x544/s350 family with right-held
input. Fresh `verify-recording --slug issue213-js-x544-s350-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s350-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s350-rbc` is the same stale x544/s350 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x544-s350-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s350-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s360-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x544-s360-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x544-s360-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x544/s360 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s360-rb` is the same stale x544/s360 family with right-held
input. Fresh `verify-recording --slug issue213-js-x544-s360-rb` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s360-rb_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s360-rbc` is the same stale x544/s360 family with right+B+C
input. Fresh `verify-recording --slug issue213-js-x544-s360-rbc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
651). The requested 0..120 case derives as
`issue213-js-x544-s360-rbc_case_00000_00120`, but `case verify` marks it blocked
with `source recording replay drifted at frame 0`; no fields match and likely
subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sibling; re-record on current native if this timing is
still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-js-x544-s370-b` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-js-x544-s370-b` and `--native-only` both drift
at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 651). The requested 0..120
case derives as `issue213-js-x544-s370-b_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale x544/s370 timing; re-record on
current native if it is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s300-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x536-s300-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x536-s300-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x536/s300 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s300-rc` is the same stale x536/s300 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x536-s300-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x536-s300-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s320-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x536-s320-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x536-s320-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x536/s320 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s320-rc` is the same stale x536/s320 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x536-s320-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x536-s320-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s340-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x536-s340-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x536-s340-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x536/s340 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s340-rc` is the same stale x536/s340 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x536-s340-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x536-s340-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s360-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x536-s360-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x536-s360-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x536/s360 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s360-rc` is the same stale x536/s360 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x536-s360-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x536-s360-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s370-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x536-s370-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x536-s370-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x536/s370 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s370-rc` is the same stale x536/s370 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x536-s370-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x536-s370-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s380-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x536-s380-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x536-s380-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x536/s380 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x536-s380-rc` is the same stale x536/s380 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x536-s380-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x536-s380-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s300-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x544-s300-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x544-s300-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x544/s300 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s300-rc` is the same stale x544/s300 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x544-s300-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x544-s300-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s320-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x544-s320-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x544-s320-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x544/s320 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s320-rc` is the same stale x544/s320 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x544-s320-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x544-s320-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s340-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x544-s340-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x544-s340-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x544/s340 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s340-rc` is the same stale x544/s340 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x544-s340-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x544-s340-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s360-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x544-s360-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x544-s360-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x544/s360 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s360-rc` is the same stale x544/s360 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x544-s360-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x544-s360-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s370-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x544-s370-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x544-s370-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x544/s370 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s370-rc` is the same stale x544/s370 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x544-s370-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x544-s370-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s380-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x544-s380-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x544-s380-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x544/s380 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x544-s380-rc` is the same stale x544/s380 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x544-s380-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x544-s380-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s300-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x552-s300-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x552-s300-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x552/s300 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s300-rc` is the same stale x552/s300 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x552-s300-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x552-s300-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s320-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x552-s320-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x552-s320-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x552/s320 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s320-rc` is the same stale x552/s320 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x552-s320-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x552-s320-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s340-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x552-s340-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x552-s340-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x552/s340 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s340-rc` is the same stale x552/s340 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x552-s340-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x552-s340-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s360-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x552-s360-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x552-s360-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x552/s360 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s360-rc` is the same stale x552/s360 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x552-s360-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x552-s360-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s370-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x552-s370-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x552-s370-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x552/s370 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s370-rc` is the same stale x552/s370 sweep family with
right+C input. Fresh `verify-recording --slug issue213-sweep-x552-s370-rc` and
`--native-only` both drift at native replay frame 0 (`trace_match=false`,
`oracle_state_match=false`, `oracle_frame_match=false`, recorded/replay trace len
801). The requested 0..120 case derives as
`issue213-sweep-x552-s370-rc_case_00000_00120`, but `case verify` marks it
blocked with `source recording replay drifted at frame 0`; no fields match and
likely subsystem is recording verification. Do not patch #213 Drips/slime/Tank
behaviour from this stale sweep sibling; re-record on current native if this
timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue213-sweep-x552-s380-c` is stale before any gameplay comparison. Fresh
`verify-recording --slug issue213-sweep-x552-s380-c` and `--native-only` both
drift at native replay frame 0 (`trace_match=false`, `oracle_state_match=false`,
`oracle_frame_match=false`, recorded/replay trace len 801). The requested 0..120
case derives as `issue213-sweep-x552-s380-c_case_00000_00120`, but `case verify`
marks it blocked with `source recording replay drifted at frame 0`; no fields
match and likely subsystem is recording verification. Do not patch #213
Drips/slime/Tank behaviour from this stale sweep route; re-record on current
native if this x552/s380 C timing is still needed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x552-w370-jump-right-native-scan` is the same scan family as
the x536 route, with scenario `issue213-x552-y575-w370-jump_right_c-1200`.
The old bundle drifts at native replay frame 0. Regenerating as
`codex-issue-213-x552-w370-jump-right-native-scan-refresh` matches native trace
but diverges from oracle at `pre_jump@369` on object counts (native `7/7`,
oracle `5/8`), and the reduced 0..120 case is blocked by the same settled Level
01 bootstrap mismatch. Do not patch gameplay from this scan family until the
bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x568-w370-jump-right-native-scan` is another
`issue213-x*-y575-w370-jump_right_c-1200` scan-family duplicate. Direct
verification of the old bundle drifts at native replay frame 0, and the scenario
has the same checkpoints as the x536/x552 scans (`pre_jump@369`,
`contact_window@490`). The refreshed scan-family evidence already shows native
trace replay is current but oracle object counts diverge at `pre_jump@369`, and
the 0..120 case is blocked by settled Level 01 bootstrap. Treat this slug as
duplicate bootstrap evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x576-w370-jump-right-native-scan` is another
`issue213-x*-y575-w370-jump_right_c-1200` scan-family duplicate. Direct
verification of the old bundle drifts at native replay frame 0; the saved
scenario has the same checkpoint shape as x536/x552/x568. Use the refreshed
scan-family evidence instead: oracle object counts diverge at `pre_jump@369`
and the reduced 0..120 case is blocked by settled Level 01 bootstrap.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w300-jump_b_hold-1200-native-scan` is a distinct
w300 timing family. The old bundle drifts at native replay frame 0. Regenerating
as `codex-issue213-x552-y575-w300-jump_b_hold-1200-native-scan-refresh` matches
native trace but diverges from oracle at `pre_jump@299` on object counts (native
`7/7`, oracle `5/8`), and the 0..120 case is blocked by the same settled Level
01 bootstrap mismatch. Do not patch gameplay from this timing family until
settled-recording oracle bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w300-jump_right_b-1200-native-scan` has the same
checkpoint shape as the w300 `jump_b_hold` route (`pre_jump@299`,
`contact_window@420`). Direct verification of the old bundle drifts at native
replay frame 0. Use the refreshed w300 evidence instead: native trace refreshes,
oracle object counts diverge at `pre_jump@299`, and the reduced 0..120 case is
blocked by settled Level 01 bootstrap.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w340-jump_b_hold-1200-native-scan` is a distinct
w340 timing family. The old bundle drifts at native replay frame 0. Regenerating
as `codex-issue213-x552-y575-w340-jump_b_hold-1200-native-scan-refresh` matches
native trace but diverges from oracle at `pre_jump@339` on object counts (native
`7/7`, oracle `5/8`), and the 0..120 case is blocked by settled Level 01
bootstrap. Do not patch gameplay from this timing family until settled-recording
oracle bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w340-jump_right_b-1200-native-scan` has the same
checkpoint shape as the w340 `jump_b_hold` route (`pre_jump@339`,
`contact_window@460`). Direct verification of the old bundle drifts at native
replay frame 0. Use the refreshed w340 evidence instead: native trace refreshes,
oracle object counts diverge at `pre_jump@339`, and the reduced 0..120 case is
blocked by settled Level 01 bootstrap.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w370-jump_b_hold-1200-native-scan` has the same
checkpoint shape as the w370 scan family (`pre_jump@369`,
`contact_window@490`). Direct verification of the old bundle drifts at native
replay frame 0. Use the refreshed w370 scan evidence instead: native trace
refreshes, oracle object counts diverge at `pre_jump@369`, and the reduced
0..120 case is blocked by settled Level 01 bootstrap.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w370-jump_right_b-1200-native-scan` has the same
checkpoint shape as the w370 `jump_b_hold` route (`pre_jump@369`,
`contact_window@490`). Direct verification of the old bundle drifts at native
replay frame 0. Use the refreshed w370 scan evidence instead; this is duplicate
bootstrap evidence, not gameplay-authoritative contact evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w300-jump_b_hold-1200-native-scan` has the same
checkpoint shape as the w300 x552 route (`pre_jump@299`, `contact_window@420`).
Direct verification of the old bundle drifts at native replay frame 0. Use the
refreshed w300 evidence instead: native trace refreshes, oracle object counts
diverge at `pre_jump@299`, and the reduced 0..120 case is blocked by settled
Level 01 bootstrap.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w300-jump_right_b-1200-native-scan` has the same
checkpoint shape as the x568/w300 `jump_b_hold` route. Direct verification of
the old bundle drifts at native replay frame 0. Use the refreshed w300 evidence
instead; this is duplicate bootstrap evidence, not gameplay-authoritative
contact evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w340-jump_b_hold-1200-native-scan` has the same
checkpoint shape as the w340 x552 route (`pre_jump@339`, `contact_window@460`).
Direct verification of the old bundle drifts at native replay frame 0. Use the
refreshed w340 evidence instead: native trace refreshes, oracle object counts
diverge at `pre_jump@339`, and the reduced 0..120 case is blocked by settled
Level 01 bootstrap.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w340-jump_right_b-1200-native-scan` has the same
checkpoint shape as the x568/w340 `jump_b_hold` route. Direct verification of
the old bundle drifts at native replay frame 0. Use the refreshed w340 evidence
instead; this is duplicate bootstrap evidence, not gameplay-authoritative
contact evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w370-jump_b_hold-1200-native-scan` has the same
checkpoint shape as the w370 scan family (`pre_jump@369`,
`contact_window@490`). Direct verification of the old bundle drifts at native
replay frame 0. Use the refreshed w370 scan evidence instead; this is duplicate
bootstrap evidence, not gameplay-authoritative contact evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w370-jump_right_b-1200-native-scan` has the same
checkpoint shape as the x568/w370 `jump_b_hold` route. Direct verification of
the old bundle drifts at native replay frame 0. Use the refreshed w370 scan
evidence instead; this is duplicate bootstrap evidence, not gameplay-authoritative
contact evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x576-y575-w300-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x576-y575-w300-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
w300 jump route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x576-y575-w300-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x576-y575-w300-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x576/w300 right+B route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x576-y575-w340-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x576-y575-w340-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
w340 jump route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x576-y575-w340-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x576-y575-w340-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x576/w340 right+B route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x576-y575-w370-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x576-y575-w370-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
w370 jump route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x576-y575-w370-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x576-y575-w370-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x576/w370 right+B route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x584-y575-w300-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x584-y575-w300-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
w300 jump route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x584-y575-w300-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x584-y575-w300-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x584/w300 right+B route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x584-y575-w340-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x584-y575-w340-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
w340 jump route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x584-y575-w340-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x584-y575-w340-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x584/w340 right+B route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x584-y575-w370-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x584-y575-w370-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
w370 jump route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x584-y575-w370-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x584-y575-w370-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x584/w370 right+B route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-nondeath-x536` is not yet gameplay-authoritative. The old bundle
drifts at native replay frame 0. Regenerating as `issue-213-nondeath-x536-refresh`
matches trace and oracle state, but recording checkpoint PNGs are pixel-red
(`start@0`, `pre_jump@369`, `post_jump@520`, `tail@899`). The requested 0..120
case derives but is blocked by the settled Level 01 bootstrap mismatch
(`x_velocity`, `y_velocity`, native 2 entities vs oracle 1). This route is a
candidate after settled-recording bootstrap is fixed, not current Drips/slime
gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-nondeath-x552` has the same non-death probe shape as x536. The old
bundle drifts at native replay frame 0. Regenerating as
`issue-213-nondeath-x552-refresh` matches trace and oracle state, but recording
checkpoint PNGs are pixel-red and the requested 0..120 case is blocked by the
settled Level 01 bootstrap mismatch (`x_velocity`, `y_velocity`, native 2
entities vs oracle 1). This is a candidate after bootstrap is fixed, not current
gameplay-authoritative Drips/slime evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-nondeath-x568` has the same non-death probe shape as x536/x552. The
old bundle drifts at native replay frame 0. Regenerating as
`issue-213-nondeath-x568-refresh` matches trace and oracle state, but recording
checkpoint PNGs are pixel-red and the requested 0..120 case is blocked by the
settled Level 01 bootstrap mismatch. This is a candidate after bootstrap is
fixed, not current gameplay-authoritative Drips/slime evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-nondeath-x576` has the same non-death probe shape as x536/x552/x568.
The old bundle drifts at native replay frame 0. Regenerating as
`issue-213-nondeath-x576-refresh` matches trace and oracle state, but recording
checkpoint PNGs are pixel-red and the requested 0..120 case is blocked by the
settled Level 01 bootstrap mismatch. This is a candidate after bootstrap is
fixed, not current gameplay-authoritative Drips/slime evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-stationary-x536` is blocked by the same settled Level 01 bootstrap.
The old bundle drifts at native replay frame 0. Regenerating as
`issue-213-stationary-x536-refresh` matches native trace but full oracle state
diverges at `pre_jump@369` on object counts (native `7/7`, oracle `5/8`), and
the requested 0..120 case is blocked by the initial velocity/entity mismatch.
Do not patch Drips/slime from this stationary route until settled-recording
bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-stationary-x552` has the same stationary jump shape as x536. The old
bundle drifts at native replay frame 0. Regenerating as
`issue-213-stationary-x552-refresh` matches native trace but full oracle state
diverges at `pre_jump@369` on object counts (native `7/7`, oracle `5/8`), and
the requested 0..120 case is blocked by the initial velocity/entity mismatch.
Do not patch Drips/slime from this stationary route until settled-recording
bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-stationary-x568` has the same stationary jump shape as x536/x552.
The old bundle drifts at native replay frame 0. Regenerating as
`issue-213-stationary-x568-refresh` matches native trace but full oracle state
diverges at `pre_jump@369` on object counts (native `7/7`, oracle `5/8`), and
the requested 0..120 case is blocked by the initial velocity/entity mismatch.
Do not patch Drips/slime from this stationary route until settled-recording
bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-stationary-x576` has the same stationary jump shape as x536/x552/x568.
The old bundle drifts at native replay frame 0. Regenerating as
`issue-213-stationary-x576-refresh` matches native trace but full oracle state
diverges at `pre_jump@369` on object counts (native `7/7`, oracle `5/8`), and
the requested 0..120 case is blocked by the initial velocity/entity mismatch.
Do not patch Drips/slime from this stationary route until settled-recording
bootstrap is fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`issue-213-stationary-x584` has the same stationary jump shape as
x536/x552/x568/x576. The old bundle drifts at native replay frame 0.
Regenerating as `issue-213-stationary-x584-refresh` matches native trace but
full oracle state diverges at `pre_jump@369` on object counts (native `7/7`,
oracle `5/8`), and the requested 0..120 case is blocked by the initial
velocity/entity mismatch. Do not patch Drips/slime from this stationary route
until settled-recording bootstrap is fixed.

Note 2026-06-02 follow-up for #213/#155: selector-picked recording
`codex-issue-155-drips-level01-idle-512` is stale before any current Drips
evidence. Fresh `verify-recording` and native-only replay both fail at frame 0
with native trace mismatch, while recorded/replay traces are both 573 frames.
The requested 0..120 case derives as `idle_level_01_case_00000_00120`, but
`case verify` marks it blocked with `likely_subsystems=["recording
verification"]`, and `case reduce` refuses because the source recording replay
drifted at frame 0. Re-record this Level 01 Drips idle route on current native
before using it as #155/#213 gameplay evidence.

Note 2026-06-02 follow-up for #213/#155: selector-picked recording
`codex-issue-155-drips-near-first-current` is another stale Level 01 Drips
idle-shaped bundle. Fresh full and native-only verification both drift at
native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 360-frame recorded/replay traces. The
requested 0..120 case derives as `issue-213-level01-drips-idle_case_00000_00120`,
but `case verify` reports `likely_subsystems=["recording verification"]` and
`case reduce` refuses because the source recording replay drifted at frame 0.
Re-record this near-first Drips route on current native before using it as
#155/#213 gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-drips-floor-probe-2000-promoted` is stale as a recording
bundle even though the earlier checked-in
`tests/data/dynamic-scenarios/issue-213-drips-floor-probe-2000.json` remains
separate durable TAS evidence. Fresh full and native-only verification drift at
native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 2000-frame recorded/replay traces. The
requested 0..120 case derives as `issue-213-drips-floor-probe-2000_case_00000_00120`,
but `case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Do not use this promoted recording bundle as new Drips gameplay evidence;
re-record if the floor-probe route needs recording-derived reduction again.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-drips-floor-probe-2000-promoted-refresh` is replay-matched but
still not gameplay-authoritative. Fresh verification has `trace_match=true`
with matching 2000-frame recorded/replay traces, but oracle comparison fails at
`mid@1000` on object counts (native `5/5`, oracle `5/9`) and frame output. The
requested 0..120 case derives as `issue-213-drips-floor-probe-2000_case_00000_00120`,
but `case verify` blocks before gameplay on settled-recording oracle bootstrap:
Level 01 starts with different player velocity fields and entity count
(`native=2`, `oracle=1`). Fix settled-recording oracle bootstrap before treating
this refreshed recording-derived floor probe as #213 Drips evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-now-trace` is stale as the original source bundle.
Fresh full and native-only verification drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1000-frame recorded/replay traces. The requested 0..120 case derives
as `codex-issue-213-index8_jump_now_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Keep using the
newer refresh/bootstrap notes for tooling work; do not treat this original
bundle as #213 Drips gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-now-trace-refresh` still blocks on
settled-recording bootstrap, not Drips gameplay. Fresh verification now has
`trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`, and
matching 1000-frame recorded/replay traces. The requested 0..120 case derives
as `codex-issue-213-index8_jump_now_case_00000_00120`, but `case verify` and
`case reduce` block before gameplay because Level 01 starts with different
player velocity fields and entity count (`native=2`, `oracle=1`). Fix
settled-recording oracle bootstrap before using this refreshed route as #213
Drips evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-right-after-tank-child` is stale as the original
source bundle. Fresh full and native-only verification drift at native replay
frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1000-frame recorded/replay traces. The
requested 0..120 case derives as
`codex-issue-213-index8_jump_right_now_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Do not patch
Drips/slime or Tank-child behaviour from this original stale bundle.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-right-after-tank-child-refresh` still blocks on
settled-recording bootstrap, not Drips/Tank-child gameplay. Fresh verification
has `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`,
and matching 1000-frame recorded/replay traces. The requested 0..120 case
derives as `codex-issue-213-index8_jump_right_now_case_00000_00120`, but
`case verify` and `case reduce` block before gameplay because Level 01 starts
with different player velocity fields and entity count (`native=2`,
`oracle=1`). Fix settled-recording oracle bootstrap before using this refreshed
route as #213 Drips or Tank-child evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-right-now-trace` is stale as the original source
bundle. Fresh full and native-only verification drift at native replay frame 0,
with `trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`,
and matching 1000-frame recorded/replay traces. The requested 0..120 case
derives as `codex-issue-213-index8_jump_right_now_case_00000_00120`, but
`case verify` reports `likely_subsystems=["recording verification"]`, and
`case reduce` refuses because the source recording replay drifted at frame 0.
Do not patch Drips/slime or Tank-child behaviour from this original stale
right-jump bundle.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-index8-jump-right-now-trace-refresh` still blocks on
settled-recording bootstrap, not Drips/Tank-child gameplay. Fresh verification
has `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`,
and matching 1000-frame recorded/replay traces. The requested 0..120 case
derives as `codex-issue-213-index8_jump_right_now_case_00000_00120`, but
`case verify` and `case reduce` block before gameplay because Level 01 starts
with different player velocity fields and entity count (`native=2`,
`oracle=1`). Fix settled-recording oracle bootstrap before using this refreshed
right-jump route as #213 Drips or Tank-child evidence.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-death-current` is not current Tank death evidence. Fresh
full and native-only verification both drift at native replay frame 369, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1200-frame recorded/replay traces. The requested 0..120 case derives
as `issue213-x568-y575-w340-jump_b_hold-1200_case_00000_00120`, but `case verify`
and `case reduce` block before gameplay because Level 01 settled-recording
oracle bootstrap starts with different player velocity fields and entity count
(`native=2`, `oracle=1`). Do not patch Tank/RockTank death or Drips contact from
this source bundle until native replay and settled-recording bootstrap are
fixed.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-death-current-refresh` replays natively but is not
oracle-authoritative. Fresh verification has `trace_match=true`, matching
1200-frame recorded/replay traces, and no trace drift, but oracle comparison
fails at `pre_jump@339` on object counts (native `7/7`, oracle `5/8`) and frame
output. The requested 0..120 case derives as
`issue213-x568-y575-w340-jump_b_hold-1200_case_00000_00120`, but `case verify`
and `case reduce` block before gameplay because Level 01 settled-recording
oracle bootstrap starts with different player velocity fields and entity count
(`native=2`, `oracle=1`). Fix settled-recording oracle bootstrap and the later
object-count divergence before using this refresh as Tank/RockTank death or
Drips contact evidence.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-hit-after-7d4e2500` is not current Tank hit evidence.
Fresh full and native-only verification both drift at native replay frame 363,
with `trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`,
and matching 1200-frame recorded/replay traces. The requested 0..120 case
derives as `issue213-x568-y575-w340-jump_b_hold-1200_case_00000_00120`, but
`case verify` and `case reduce` block before gameplay because Level 01
settled-recording oracle bootstrap starts with different player velocity fields
and entity count (`native=2`, `oracle=1`). Do not patch Tank/RockTank hit/death
or Drips contact from this source bundle until native replay and
settled-recording bootstrap are fixed.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-hit-after-7d4e2500-refresh` replays natively but is not
oracle-authoritative. Fresh verification has `trace_match=true`, matching
1200-frame recorded/replay traces, and no trace drift, but oracle comparison
fails at `pre_jump@339` on object counts (native `7/7`, oracle `5/8`) and frame
output. The requested 0..120 case derives as
`issue213-x568-y575-w340-jump_b_hold-1200_case_00000_00120`, but `case verify`
and `case reduce` block before gameplay because Level 01 settled-recording
oracle bootstrap starts with different player velocity fields and entity count
(`native=2`, `oracle=1`). Fix settled-recording oracle bootstrap and the later
object-count divergence before using this refresh as Tank/RockTank hit/death or
Drips contact evidence.

Note 2026-06-02 follow-up for #213/#167: selector-picked recording
`codex-issue-213-tank-hit-current` is a duplicate stale Tank hit source shape.
Fresh full and native-only verification both drift at native replay frame 363,
with `trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`,
and matching 1200-frame recorded/replay traces. The requested 0..120 case
derives as `issue213-x568-y575-w340-jump_b_hold-1200_case_00000_00120`, but
`case verify` and `case reduce` block before gameplay because Level 01
settled-recording oracle bootstrap starts with different player velocity fields
and entity count (`native=2`, `oracle=1`). Treat this as duplicate stale
Tank/Drips tooling evidence until native replay and settled-recording bootstrap
are fixed.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x536-w370-jump-right-native-scan` is stale before any current
Drips/Tank contact evidence. Fresh full and native-only verification both drift
at native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1200-frame recorded/replay traces. The
requested 0..120 case derives as
`issue213-x536-y575-w370-jump_right_c-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x536/w370 scan route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x536-w370-jump-right-native-scan-refresh` replays natively but
is not oracle-authoritative. Fresh verification has `trace_match=true`,
matching 1200-frame recorded/replay traces, and no trace drift, but oracle
comparison fails at `pre_jump@369` on object counts (native `7/7`, oracle
`5/8`) and frame output. The requested 0..120 case derives as
`issue213-x536-y575-w370-jump_right_c-1200_case_00000_00120`, but `case verify`
and `case reduce` block before gameplay because Level 01 settled-recording
oracle bootstrap starts with different player velocity fields and entity count
(`native=2`, `oracle=1`). Fix settled-recording oracle bootstrap and the later
object-count divergence before using this refresh as Drips/Tank gameplay
evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x552-w370-jump-right-native-scan` is stale before any current
Drips/Tank contact evidence. Fresh full and native-only verification both drift
at native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1200-frame recorded/replay traces. The
requested 0..120 case derives as
`issue213-x552-y575-w370-jump_right_c-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x552/w370 scan route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x552-w370-jump-right-native-scan-refresh` replays natively but
is not oracle-authoritative. Fresh verification has `trace_match=true`,
matching 1200-frame recorded/replay traces, and no trace drift, but oracle
comparison fails at `pre_jump@369` on object counts (native `7/7`, oracle
`5/8`) and frame output. The requested 0..120 case derives as
`issue213-x552-y575-w370-jump_right_c-1200_case_00000_00120`, but `case verify`
and `case reduce` block before gameplay because Level 01 settled-recording
oracle bootstrap starts with different player velocity fields and entity count
(`native=2`, `oracle=1`). Fix settled-recording oracle bootstrap and the later
object-count divergence before using this refresh as Drips/Tank gameplay
evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x568-w370-jump-right-native-scan` is stale before any current
Drips/Tank contact evidence. Fresh full and native-only verification both drift
at native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1200-frame recorded/replay traces. The
requested 0..120 case derives as
`issue213-x568-y575-w370-jump_right_c-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x568/w370 scan route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue-213-x576-w370-jump-right-native-scan` is stale before any current
Drips/Tank contact evidence. Fresh full and native-only verification both drift
at native replay frame 0, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 1200-frame recorded/replay traces. The
requested 0..120 case derives as
`issue213-x576-y575-w370-jump_right_c-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x576/w370 scan route on current native before using it as Drips/Tank gameplay
evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w300-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x552-y575-w300-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x552/y575/w300 scan route on current native before using it as Drips/Tank
gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w300-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x552-y575-w300-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x552/y575/w300 right+B scan route on current native before using it as
Drips/Tank gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w340-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x552-y575-w340-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x552/y575/w340 scan route on current native before using it as Drips/Tank
gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w340-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x552-y575-w340-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x552/y575/w340 right+B scan route on current native before using it as
Drips/Tank gameplay evidence.

Note 2026-06-02 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w370-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x552-y575-w370-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x552/y575/w370 scan route on current native before using it as Drips/Tank
gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x552-y575-w370-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x552-y575-w370-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x552/y575/w370 right+B scan route on current native before using it as
Drips/Tank gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w300-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x568-y575-w300-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x568/y575/w300 scan route on current native before using it as Drips/Tank
gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w300-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x568-y575-w300-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x568/y575/w300 right+B scan route on current native before using it as
Drips/Tank gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w340-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x568-y575-w340-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x568/y575/w340 scan route on current native before using it as Drips/Tank
gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w340-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x568-y575-w340-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x568/y575/w340 right+B scan route on current native before using it as
Drips/Tank gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w370-jump_b_hold-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x568-y575-w370-jump_b_hold-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x568/y575/w370 scan route on current native before using it as Drips/Tank
gameplay evidence.

Note 2026-06-03 follow-up for #213: selector-picked recording
`codex-issue213-x568-y575-w370-jump_right_b-1200-native-scan` is stale before
any current Drips/Tank contact evidence. Fresh full and native-only verification
both drift at native replay frame 0, with `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, and matching 1200-frame
recorded/replay traces. The requested 0..120 case derives as
`issue213-x568-y575-w370-jump_right_b-1200_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 0. Re-record this
x568/y575/w370 right+B scan route on current native before using it as
Drips/Tank gameplay evidence.

Note 2026-06-04 selector follow-up for #214: selector-picked recording
`issue-214-highwater-dense-0-40` is stale on current native before any new
Highwater shooter/barrier conclusion. Fresh full and native-only
`verify-recording` both fail at frame 4 with
`recording replay drifted at frame 4 (native trace frame mismatch)`. The cached
report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 41-frame
recorded/replay traces. The requested `0..40` case derives as
`highwater_shooter_barrier_audit_02_case_00000_00040`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 4. Re-record this
dense Highwater prefix on current native before using it as #214 evidence.

Note 2026-06-04 selector follow-up for #214: selector-picked recording
`issue-214-highwater-dense-80-130` is stale on current native with the same
frame-4 source replay drift as the `0..40` dense slice. Fresh full and
native-only `verify-recording` both fail at frame 4 with
`recording replay drifted at frame 4 (native trace frame mismatch)`. The cached
report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 131-frame
recorded/replay traces. The requested `0..120` case derives as
`highwater_shooter_barrier_audit_02_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 4. Treat the
cached `issue-214-highwater-dense-` recording family as stale selector input
until the dense Highwater slices are re-recorded on current native.

Note 2026-06-04 selector follow-up for #214: selector-picked recording
`issue-214-highwater-jump-overlap-current` is current and green for the
selected gameplay prefix. Native-only replay passes, full `verify-recording`
now reports `trace_match=true`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 421-frame
recorded/replay traces. The requested `0..120` case derives as
`stomp_enemy_02_case_00000_00120` and passes with matching
`world.game_mode`, camera, player position, entities, and events; `case reduce`
correctly reports the case is already passing. Do not patch #214 gameplay from
this clean prefix. The live jump-overlap / launch report still needs a longer
current route that actually reproduces the overlap response.

Note 2026-06-04 selector follow-up for #214: selector-picked recording
`issue-214-highwater-shooter` is stale before any current shooter/barrier
conclusion. Fresh full and native-only `verify-recording` both fail at frame 1
with `recording replay drifted at frame 1 (native trace frame mismatch)`. The
cached report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 151-frame
recorded/replay traces. The requested `0..120` case derives as
`highwater_shooter_barrier_audit_02_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 1. Re-record this
Highwater shooter route on current native before using it as #214 evidence.

Note 2026-06-04 selector follow-up for #214: selector-picked recording
`issue-214-highwater-shooter-barrier-audit` is stale before any current
shooter/barrier conclusion, matching the stale `issue-214-highwater-shooter`
family shape. Fresh full and native-only `verify-recording` both fail at frame
3 with `recording replay drifted at frame 3 (native trace frame mismatch)`. The
cached report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 151-frame
recorded/replay traces. The requested `0..120` case derives as
`highwater_shooter_barrier_audit_02_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 3. Treat cached
`issue-214-highwater-shooter` recording variants as stale selector input until
the shooter/barrier audit route is re-recorded on current native.

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

Note 2026-06-06 current-route refresh for #214: re-recorded the committed
`highwater-shooter-barrier-audit` scenario as cache-local slug
`issue-214-highwater-shooter-barrier-current-20260606` on current native. Fresh
`verify-recording` now reports `trace_match=true` and
`oracle_state_match=true`: the Level 02 Berzerker route spawns shooter
projectiles from frames 92-117, clears the ice barrier cells by frame 117, and
does not reproduce the original "projectile fails to clear barrier" gameplay
drift. Full verification still fails `oracle_frame_match=false`: the largest
current deltas are checkpoint-pixel differences around the shooter/ice destroy
visuals (`projectile_barrier_window@90` through `barrier_clear_check@120`) plus
the pre-existing Berzerker palette/pose differences visible from
`route_start`. Treat #214's live shooter/barrier state path as current-green;
keep the row open for render/debris pixel parity and a longer route that
actually reproduces the reported shooter-block overlap launch.

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

Note 2026-06-02 follow-up for #214: the cache-local
`issue-214-highwater-jump-overlap-current` route no longer reproduces the
reported player launch as gameplay state: native replay is current and
`tas-diff --scenario ... --frame-start 120 --frame-end 421 --frontier-kind
gameplay --include-advisory --only-divergent` reports `300 match, 0 diverge`.
It did expose a visual trace/export bug: `NativeTraceFrame.floating_items`
stored only the low byte of `PrizeDiamond_Init`'s d3 countdown, so the renderer
treated a live `$0182` diamond as the late `$0082` `stru_10D80` blink phase and
hid it. Native trace now exports the full remaining word, the renderer drives
`stru_10D6E` from the object-local `loc_10396` countdown instead of global
`Time_Frames`, and it only applies the `stru_10D80` blink/null-frame gate once
ASM `PrizeDiamond_Init loc_103B2` has counted down to `$00C8`. Regenerated
verification still has pixel deltas (`stomp_quarter` terrain/palette, and
small floating-diamond palette/frame phase differences at `stomp_half` /
`stomp_end`), so #214 remains open for visual parity and a fresh live route if
the launch is still observable.

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

Note 2026-06-02 selector follow-up for #214: queued recording
`codex-highwater-berzerker-shooter` is also stale before any gameplay/oracle
question. Fresh `verify-recording` and `verify-recording --native-only` both
fail at frame 0 with native trace mismatch while recorded/replay traces are
121 frames. The requested `0..120` case derives as
`codex-highwater-berzerker-shooter_case_00000_00120`, but `case verify` fails
with `likely_subsystems=["recording verification"]`; `case reduce` refuses with
`source recording replay drifted at frame 0`, and `case summarise` only refreshes
the blocked summary. Re-record the Berzerker shooter route on current native
before using it for Highwater shooter activation, projectile rendering, or
player overlap fixes.

Follow-up 2026-06-02: selector-picked sibling
`codex-highwater-berzerker-shooter-long` has the same stale shape. Fresh
`verify-recording` and native-only replay fail at frame 0 with native trace
mismatch; recorded/replay traces are both 301 frames. The requested `0..120`
case derives as `codex-highwater-berzerker-shooter-long_case_00000_00120`, but
`case verify` marks it blocked with `likely_subsystems=["recording verification"]`
and `case reduce` refuses on the same source replay drift. Re-record the long
Berzerker shooter route on current native before treating it as #214 gameplay
evidence.

Follow-up 2026-06-02: selector-picked sibling
`codex-highwater-berzerker-shooter-window` is stale too. Fresh
`verify-recording` and native-only replay fail at frame 0 with native trace
mismatch; recorded/replay traces are both 121 frames. The requested `0..120`
case derives as `codex-highwater-berzerker-shooter-window_case_00000_00120`,
but `case verify` marks it blocked with
`likely_subsystems=["recording verification"]` and `case reduce` refuses on the
same source replay drift. Re-record the window route on current native before
treating it as #214 gameplay evidence.

Follow-up 2026-06-02: selector-picked
`codex-highwater-shooter-barrier-frontier` is stale before its barrier evidence
too. Fresh `verify-recording` and native-only replay both fail at frame 3 with
native trace mismatch; recorded/replay traces are both 151 frames. The requested
`0..120` case derives as `highwater_shooter_barrier_audit_02_case_00000_00120`,
but `case verify` marks it blocked with
`likely_subsystems=["recording verification"]` and `case reduce` refuses with
`source recording replay drifted at frame 3`. Re-record this shooter/barrier
route on current native before using it as #214 activation, projectile, barrier,
or player-overlap evidence.

Note 2026-06-02 latest for #214: current committed shooter/barrier audit route
is gameplay-replayable but still not closed. Native replay passes for
`issue-214-highwater-shooter-barrier-audit`, focused shooter logic coverage is
green (`cargo test -q -p kid-core shooter -- --nocapture`: 97 passed;
`cargo test -q -p kidcli highwater_shooter -- --nocapture`: 1 passed, 1
ignored asset-backed test), and oracle gameplay parity for frames 80..130
reports `50 match, 0 diverge`. Full `verify-recording --slug
issue-214-highwater-shooter-barrier-audit` still fails strict oracle pixels, so
keep #214 blocked for Highwater projectile/palette/water visual debt and the
reported shooter-block/player-overlap launch repro; do not reopen shooter
activation/barrier gameplay without a new current divergent route.

Note 2026-06-02 verifier HUD follow-up for #214: fresh cache-local
`issue-065-highwater-shooter-current` evidence confirms the shooter stack is a
visual-parity problem, not a current gameplay activation failure:
`trace_match=true`, `oracle_state_match=true`, and the first shooter projectiles
appear at frame 92 before the barrier clears by frame 120. The strict PNG diff
exposed stale recording-render compatibility instead: settled warp recordings
were adding one synthetic HUD second even though ASM `Make_SpriteAttr_HUD`
`loc_BEE8` should draw from the authoritative live timer trace. Native frame 92
now renders `2:57` like oracle instead of `2:58`; remaining red pixels are
Highwater terrain/palette/player/projectile composition, so #214 stays open for
visual debt and any fresh shooter-block/player-overlap launch repro.

Follow-up 2026-06-03 selector replay for #065/#214: selector-picked
`issue-065-highwater-shooter-current` is no longer authoritative against current
native. Fresh full and native-only `verify-recording` both drift at native replay
frame 3, with `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, and matching 151-frame recorded/replay traces. The
requested `0..120` case derives as
`highwater_shooter_barrier_audit_02_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses because
the source recording replay drifted at frame 3. Re-record this Highwater
shooter/barrier route on current native before using it as gameplay evidence.


Note 2026-06-04 selector follow-up for #215: selector-picked recording
`issue-215-highwater-water-berzerker-probe` is stale before any current
Highwater water/foreground priority conclusion. Fresh full and native-only
`verify-recording` both fail at frame 63 with
`recording replay drifted at frame 63 (native trace frame mismatch)`. The
cached report has `trace_match=false`, `oracle_state_match=true`,
`oracle_frame_match=true`, no frame diffs, and matching 1861-frame
recorded/replay traces. The requested `0..120` case derives as
`highwater_water_priority_audit_02_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 63. Re-record this
Berzerker water-priority route on current native before using it as #215
evidence.

Note 2026-06-04 selector follow-up for #215: selector-picked recording
`issue-215-highwater-water-berzerker-rightonly` is stale before any current
Highwater water/foreground priority conclusion, matching the stale Berzerker
water-priority probe family. Fresh full and native-only `verify-recording` both
fail at frame 3 with `recording replay drifted at frame 3 (native trace frame
mismatch)`. The cached report has `trace_match=false`,
`oracle_state_match=true`, `oracle_frame_match=true`, no frame diffs, and
matching 1861-frame recorded/replay traces. The requested `0..120` case derives
as `highwater_water_priority_audit_02_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
refuses because the source recording replay drifted at frame 3. Treat cached
`issue-215-highwater-water-berzerker-` recordings as stale selector input until
the Berzerker water-priority route is re-recorded on current native.

Note 2026-06-04 selector follow-up for #215: selector-picked recording
`issue-215-highwater-water-current` has a clean selected gameplay prefix but is
not a current long-route proof. Full and native-only `verify-recording` drift at
frame 140 with matching 1861-frame recorded/replay traces and no frame diffs.
The requested `0..120` case derives as
`highwater_water_priority_audit_02_case_00000_00120` and passes with matching
`world.game_mode`, camera, player position, entities, and events; `case reduce`
correctly reports the case is already passing. Do not patch #215 gameplay from
this clean prefix, and do not use the post-frame-140 recording tail until it is
re-recorded on current native.
Note 2026-06-06 selector follow-up for #215: selector-picked recording
`issue-playtest-highwater-water-current` has the same current shape. Fresh full
and native-only `verify-recording` both drift at frame 140 with matching
1861-frame recorded/replay lengths; the full report still has
`oracle_state_match=true` and `oracle_frame_match=false` from the known
Highwater water/palette pixel diffs. The requested `0..120` prefix derives as
`walk_right_02_case_00000_00120` and passes with matching `world.game_mode`,
camera, player position, entities, and events; `case reduce` reports it is
already passing. Do not patch #215 from this clean prefix, and re-record this
playtest Highwater route on current native before using the later tail.

Note 2026-06-01 for #215: cache-local `issue-playtest-highwater-water-current` now has oracle frame artefacts. The recording is still not a clean gameplay gate because replay trace comparison drifts at frame 0 and later checkpoints diverge after player motion changes, but the first two frames are oracle-state aligned enough to isolate the waterfall palette phase. Native `foreground_palette_for_scene` had the Mountain/Highwater `RotatePalette_Lava_Water loc_1223A/loc_1223C` phase one rotation late for the direct level-skip lineage; pre-advancing by two rotations improves the early strict diffs from 249/386 changed pixels to 72/209. Remaining #215 work is true foreground priority/player layering around the later water section, after a replayable non-drifting scenario is captured.
Note 2026-06-01 for #215: added named `highwater-water-priority-audit` scenario (cache slug `issue-215-highwater-water-priority-audit`) so the Level 02 water/foreground route is no longer only an ignored `.cache` artefact. Fresh verify result: `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`; `level_settle@60` and `waterfall_overlap_loc_300BA@960` are pixel-perfect, while `water_hole_entry_loc_12910@510` differs by 205 pixels and `waterfall_palette_tail_loc_1223C@1859` by 18 pixels. Current evidence points at residual Mountain Plane B/waterfall tile row or palette phase drift, not a broad gameplay replay mismatch.
Note 2026-06-01 for #215: oracle-probed `water_hole_entry_loc_12910@510` and found the largest 205px diff was not water priority at all: the same diamond object is alive in oracle WRAM at `$FFFF2610` (`x=$78`, `y=$681`, `object_meta=$64`), but `addroffset_sprite(a3)=0`, so ASM `Make_SpriteFromGfxObject` returns before emitting VDP sprite attrs. Native rendered every `floating_items` trace entry unconditionally. Matching `PrizeDiamond_Init loc_103B2` / `stru_10D80` null blink frames removes that false terrain-overdraw and reduces the frame-510 diff to 121px with `trace_match=true` and `oracle_state_match=true`. Remaining strict pixel drift is now HUD diamond animation/palette phase at `(288..299,34..45)` plus the small life-icon/HUD drift at frame 1859, not the player/water priority route.
Note 2026-06-01 for #215: rejected a global HUD diamond phase lead change from 2 to 3 `unk_461C` slots. It improved `water_hole_entry_loc_12910@510` from 121px to 82px, but regressed previously pixel-perfect `level_settle@60` and `waterfall_overlap_loc_300BA@960` to 45px each. Oracle WRAM watches for `$FFFFFA58/$FFFFFA5D` show the ROM's `sub_44DC loc_45EE` state continuing independently of the replay-local `frame_index` shortcut and with gaps around non-gameplay spans, so the next fix should model or trace HUD animation RAM/state rather than tune `HUD_DIAMOND_DMA_PAGE_RENDER_LEAD_LOC_45EE`.
Note 2026-06-01 for #215: corrected the durable `highwater-water-priority-audit` route after finding the previous Kid-with-jumps script was false coverage: native was still blocked around x=$009A by the first smash-column stopper while the checkpoint names claimed water/waterfall coverage. The scenario now seeds Berzerker and holds Right, matching the live route that uses ASM `loc_9564` charging to break the blocker. Cache-local `issue-215-highwater-water-berzerker-rightonly` reaches x=$0208 by `water_hole_entry_loc_12910@510` and x=$0318 by `waterfall_overlap_loc_300BA@960`; native-only replay passes, while full oracle verification now fails usefully at `waterfall_overlap_loc_300BA@960` with object counts `native 4 / 4, oracle 10 / 4`. Continue #215 from that real route/object divergence before making more renderer priority changes.
Note 2026-06-01 for #215: resolved the first `highwater-water-priority-audit` object-count divergence by restoring the ASM `loc_779C/loc_77E6 -> sub_922C -> loc_10DA4` rubber side effect from standing-still side-wall probes and matching `loc_10DA4`'s count-visible object cap. Refreshed verification now advances past object counts and fails at `waterfall_overlap_loc_300BA@960` on `character_movement` (`native 0`, oracle 2), so the next #215 investigation should stay in Berzerker held-right movement state rather than renderer priority.
Note 2026-06-01 for #215: follow-up restored the immediate ASM `loc_8E5A -> loc_8E74` walking carry after grounded rubber wall contact by letting the live `$FFFFF8F0` walk phase bypass the standing-still loc_779C hold on the next frame. Native now resumes `MoveID_Walking` / `x_vel=$1000` immediately after the frame-732 rubber hit, but full `verify-recording --slug issue-215-highwater-water-priority-audit` still fails at `waterfall_overlap_loc_300BA@960`: native lands on the snap frame (`x=$03180000`, `x_vel=0`, `MoveID_Standingstill`) while oracle is one frame after snap (`x=$03181000`, `x_vel=$1000`, `MoveID_Walking`) with the earlier `y_pos` half-pixel delta still present. Next step: compare ASM `sub_8F26` tile-boundary distance / rubber snap cadence and the grounded y snap around this rubber wall.
Note 2026-06-02 for #215: do not use the cache-local `issue-215-highwater-water-priority-audit` recording wrapper as current evidence. `cargo run -q -p kidcli -- verify-recording --slug issue-215-highwater-water-priority-audit` now fails before the oracle question with `recording replay drifted at frame 235`, but the durable scenario builder behind it is current: `cargo run -q -p kidcli -- verify-tas --scenario .cache/recordings/issue-215-highwater-water-priority-audit/scenario.json --frame-start 0 --frame-end 1860 --checkpoint-interval 10 --frontier-kind gameplay --bisect` passes all 1860 frames by CRC. This is not a closure: `tas-diff` still reports short authoritative trace divergences around the waterfall cadence (`character_movement` and `number_objects`, including frames 953..965 and 1840..1859), so the next #215 pass should investigate whether those typed fields are genuine `sub_8F26` / `loc_10DA4` state debt or should be reclassified for this route before making more renderer priority changes.
Note 2026-06-02 for #215: cleared the false `character_movement` half of the Highwater rubber cadence mismatch. Native trace export now recognises ASM `sub_8F26 -> loc_8F98 -> loc_8E5A -> loc_8E74`: rubber wall contact can zero `x_vel` while keeping the walking coroutine live for Berzerker. Regression `current_character_movement_keeps_rubber_wall_snap_walking_loc_8e74` keeps ordinary non-rubber zero-velocity wall stops on `MoveID_Standingstill`. Rechecking `tas-diff --scenario .cache/recordings/issue-215-highwater-water-priority-audit/scenario.json --frame-start 950 --frame-end 970 --frontier-kind gameplay --include-advisory --include-physics-trace --only-divergent` removes the frame 953/959/965 `character_movement` diffs; the 1840..1860 window likewise now diverges only on `number_objects`. Continue #215 from `loc_10DA4/sub_10DE4` object-count timing.
Note 2026-06-02 for #215: classified the remaining Highwater rubber cadence `number_objects` drift as an oracle sampling limitation, not live gameplay debt. ASM `loc_10DA4` allocates object-only `sub_10DE4` rubber-skin workers and `sub_10DE4` decrements the ROM counter before all helper slots have cleared; oracle probes show `$FFFFFB5A` cycling 4/6 while the slot dump still contains `code=0x010e*` workers. The native route has same-frame rubber wall contact in `kid_flip_branch_probe`, the oracle has hibernating `sub_10DE4` worker slots, and the enemy-frontier check for frames 952..966 is clean. `tas-diff --scenario .cache/recordings/issue-215-highwater-water-priority-audit/scenario.json --frame-start 952 --frame-end 966 --frontier-kind gameplay --include-advisory --only-divergent` now reports six `NON-AUTH` `(oracle limitation)` rows; the 1840..1860 repeat window reports seven `NON-AUTH` rows. The `tas-diff` display now counts by `DivergenceCategory` so oracle limitations no longer print as authoritative.
Note 2026-06-02 refreshed #215 visual evidence: regenerated
`issue-215-highwater-water-priority-audit` from its current scenario instead of
using the stale wrapper. `verify-recording --slug
issue-215-highwater-water-priority-audit` now has `trace_match=true` and
`oracle_state_match=true`, failing only strict pixels. The visible
`waterfall_overlap_loc_300BA@960` frame shows Berzerker in the same position
and the waterfall/background composition is visually aligned; the remaining
diff is small HUD diamond/player-palette noise, with the top-right diamond
phase also recurring at `water_hole_entry_loc_12910@510`. Do not change
water/player priority from this route without a fresh frame that visibly puts
Kid in front of a priority foreground tile; the next actionable renderer work is
to surface/model ASM `sub_44DC` HUD animation RAM rather than tuning the
existing `trace.frame_index` shortcut.
Note 2026-06-02 for #215: native now carries ASM `sub_44B0`/`sub_44DC`
HUD animation state through `NativeRuntime` and exports `$FFFFFA5D` as
`hud_diamond_frame_offset`. The renderer uses that traced `unk_461C` byte offset
for the top-right HUD diamond when present, leaving the old frame-index phase
only as a legacy trace fallback. Regressions:
`trace_exposes_live_hud_diamond_offset_from_sub_44dc`,
`game_hud_diamond_icon_animates_via_sub_44dc_dma_pages`, and
`game_hud_diamond_icon_prefers_traced_sub_44dc_offset`. Focused verification is
green, but the cache wrapper `verify-recording --slug
issue-215-highwater-water-priority-audit` is now stale again and drifts at frame
0 because recorded traces lack the new HUD field; refresh the recording before
using strict pixels to judge the remaining #215 visual debt.
Note 2026-06-02 follow-up for #215: refreshed current wrapper verification now
has `trace_match=true`, `oracle_state_match=true`, and strict pixel failures
only. Visual inspection plus oracle watches at Kid GfxObject
`addroffset_sprite=$FF2502` show the red frames are Berzerker sprite selection,
not water priority: frame 60 expects `$021C` with `$FFFFF8F0=$0228`, frame 510
expects slope sprite `$0280` with `$FFFFF8F0=$01E0`, frame 960 expects `$0234`,
and frame 1859 expects `$0230`. Native now lets ASM `loc_A13A` slope art win
over the ordinary `loc_9600` Berzerker walk latch while keeping charge
`loc_9564` dominant; this improves `water_hole_entry_loc_12910@510` from 658 to
519 changed pixels, with trace/state still green. Remaining #215 visual work
should model/export the exact player `addroffset_sprite`/ASM mapping selection
for rendered player frames rather than changing foreground priority.
Note 2026-06-02 selector follow-up for #215: queued
`codex-highwater-water-priority-frontier` is stale before any current
water-priority evidence. Fresh `verify-recording` and native-only replay both
fail at frame 3 with native trace mismatch; recorded/replay traces are both
1861 frames. The requested `0..120` case derives as
`highwater_water_priority_audit_02_case_00000_00120`, but `case verify` marks
it blocked with `likely_subsystems=["recording verification"]` and `case reduce`
refuses with `source recording replay drifted at frame 3`. Use the refreshed
current `issue-215-highwater-water-priority-audit` route or re-record this
wrapper before treating it as #215 gameplay or render evidence.
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
Note 2026-06-02 for #216: refreshed the EOL/flag half with current
`codex-highwater-flag-transition-frontier` evidence. The route is
trace/oracle/frame green and `audio-events.txt` records Score_Board BGM at
frame 0 plus `PlaySound(12)` when the score counter starts at frame 305; the
native-player transcript path is now covered by
`scripted_recording_audio_events_emit_eol_score_counter_at_loc_d770`, matching
ASM `Score_Board` / `loc_D752` / `loc_D770`. This covers event timing but not
the human report that the sound audibly cuts off; rendered waveform-window
comparison is still needed. The old `issue-216-prize-block-audio-events` route
is also polluted by frame-109 damage (`PlaySound(97)`), so block/diamond delay
still needs a clean no-damage prize-block recording before closure.
Follow-up 2026-06-02: selector-picked refresh shows the cache-local
`codex-highwater-flag-transition-frontier` wrapper is no longer authoritative
for reduced gameplay work. Fresh `verify-recording` and native-only replay both
drift at frame 299 with native trace mismatch, although recorded/replay traces
are both 2401 frames. The requested `0..120` case derives as
`highwater_flag_transition_audit_02_case_00000_00120`, but `case verify` fails
and `case reduce` blocks with settled-recording oracle bootstrap reaching Level
02 gameplay from a different initial state (`player x_velocity|y_velocity`,
`camera x_pos`). Re-record the flag-transition route or fix settled-recording
oracle bootstrap before treating this cache wrapper as current #216 gameplay or
waveform evidence.

Note 2026-06-04 selector follow-up for #216: selector-picked recording
`issue-216-highwater-flag-audio-events` has the same blocker as the older
flag-transition wrapper. Fresh full and native-only `verify-recording` both
drift at frame 299 with `recording replay drifted at frame 299 (native trace
frame mismatch)`, matching 2401-frame recorded/replay traces and no frame
diffs. The requested `0..120` case derives as
`highwater_flag_transition_audit_02_case_00000_00120`, but `case verify`
reports `likely_subsystems=["recording verification"]`, and `case reduce`
blocks because settled-recording oracle bootstrap reaches Level 02 gameplay
with a different initial state (`player x_velocity|y_velocity`, camera
`x_pos`). Re-record this flag/audio route or fix settled-recording oracle
bootstrap before treating it as current #216 gameplay, audio-event, or waveform
evidence.

Note 2026-06-02 for #216: added a committed clean repro builder,
`record-scenario --scenario issue216-prize-audio-audit --level 00`, because
the old generic `prize-block` route walked through an enemy before the audio
window. Refreshed `.cache/recordings/issue-216-prize-audio-audit/` starts at
Kid `(230,271)`, has no transcript events, native replay passes, and
`audio-events.txt` records Level BGM at frame 0, jump `PlaySound(23)` at frame
22, block hit `PlaySound(13)` at frame 31, and diamond prize `PlaySound(45)`
at frame 132. Full oracle verification is trace/state green
(`trace_match=true`, `oracle_state_match=true`) but strict-pixel red from broad
frame-0 renderer drift (`changed_pixel_count=33565`, later checkpoints
3742/5207/4359/4201), so this closes the missing clean audio-event evidence but
not rendered waveform or pixel parity.
Note 2026-06-02 for #216: `verify-recording` reports now include native
`audio_event_windows` whenever a bundle has `audio.wav` and `audio-events.txt`.
This turns event-specific SFX reports into durable waveform evidence instead of
ad-hoc scripts. Regenerated `issue-216-prize-audio-audit` remains
trace/state-green and pixel-red, but its report now shows audible immediate
windows for the queued cues: jump `PlaySound(23)` at frame 22 (`rms=0.0346`,
`peak=0.0951`), block hit `PlaySound(13)` at frame 31 (`rms=0.0365`,
`peak=0.1109`), and diamond prize `PlaySound(45)` at frame 132 (`rms=0.0629`,
`peak=0.1509`). Remaining #216 work is oracle/native waveform alignment and
the human report that level-complete audio cuts off.
Follow-up 2026-06-04: selector-picked `issue-216-prize-audio-audit` is no
longer current enough to reduce into gameplay-authoritative #216 evidence.
Fresh full and native-only verification both fail with `recording replay
drifted at frame 99 (native trace frame mismatch)`, with matching 181-frame
recorded/replay traces, `oracle_state_match=true`, `oracle_frame_match=true`,
and no frame diffs. The cached report still shows audible event windows for
Level BGM frame 0, jump frame 22, block-hit frame 31, and diamond-prize frame
132, but the requested `0..120` case
`issue-216-prize-audio-audit_02_case_00000_00120` fails with
`likely_subsystems=["recording verification"]`, and `case reduce` blocks on the
same frame-99 source replay drift. Re-record this prize-audio route on current
native before using it for #216 gameplay, waveform, or event-timing work.
Follow-up 2026-06-06: re-recorded the clean #216 prize-audio route on current
native as `issue-216-prize-audio-audit-current-20260606`, plus the old generic
`issue-216-prize-block-audio-events-current-20260606` route for comparison. The
old cached generic route is stale: fresh `verify-recording` on
`issue-216-prize-block-audio-events` drifts at frame 162, and its `120..180`
case is blocked by recording verification. The current focused route is the
usable evidence: native-only replay passes, full oracle verification has
`trace_match=true` and `oracle_state_match=true`, but remains strict-pixel red
(`oracle_frame_match=false`). `audio-events.txt` records Level BGM frame 0,
jump `PlaySound(23)` frame 22, block hit `PlaySound(13)` frame 31, and diamond
prize `PlaySound(45)` frame 99; event-window RMS/peak are audible immediately
for the queued cues (`0.0346/0.0951`, `0.0365/0.1109`, `0.0549/0.1609`). The
fresh semantic case
`issue_216_prize_audio_audit_00_case_00000_00120` passes with matching
`world.game_mode`, camera, player position, entities, and events. Do not
reselect the stale `issue-216-prize-audio-audit` cache bundle; remaining #216
work is rendered-pixel/waveform alignment and the level-complete playback
quality report, not block/diamond event timing.
Follow-up 2026-06-04: selector-picked `issue-216-prize-audio-probe` exposed two
real current gameplay misses before returning to the usual #216 renderer/audio
lane. First, ASM `sub_8F26 -> sub_922C` scans the whole leading wall column from
Kid's feet through the adjusted head row; native sampled only three body points,
missed the top-row right wall in the reducer, and advanced to x=`$006C2000`
instead of oracle x=`$006A0000`. Native now uses the existing
`probe_wall_column` path for live wall hits, covered by
`sub_8f26_right_wall_scan_reaches_top_row_loc_922c_issue_216`. Second, the
same refreshed route exposed DragonWalking contact timing: steady grounded
contact must use the exact ASM box and the normal next-object-pass pending
queue, while the #245 landing-graze route keeps its separate one-frame landing
latch and consumes on the following grounded frame. Regressions
`steady_grounded_dragon_contact_defers_to_next_frame_loc_3a6_issue_216` and
`landing_dragon_graze_consumes_after_existing_landing_defer_loc_b700_issue_245`
pin that split. Fresh bundle
`issue-216-prize-audio-probe-columnscan-contact2-current` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red only from
broad renderer drift; audio windows are Level BGM frame 0, jump frame 22, and
hurt `PlaySound(97)` frame 64. The derived 0..120 case
`issue-216-prize-audio-probe-columnscan-contact2-current_case_00000_00120`
passes. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.
Follow-up 2026-06-06: the delegated case-queue target
`issue-216-prize-audio-probe-columnscan-contact-current_case_00000_00120` is
not current gameplay evidence. Fresh `case verify` rewrote it to blocked with
`source recording replay drifted at frame 63`; direct native-only source
verification fails at the same frame. The sibling
`issue-216-prize-audio-probe-columnscan-current_case_00000_00120` is also
blocked, with source replay drift at frame 62. Do not patch combat from either
stale queue entry; keep using the later
`issue-216-prize-audio-probe-columnscan-contact2-current_case_00000_00120`
green result and continue #216 from waveform/rendered-pixel parity or a fresh
level-complete audio route.
Note 2026-06-02 for #216: delegated x140 prize-audio probe
`issue-216-prize-audio-probe-x140-refresh` first exposed a real native-only
gameplay bug: Kid idled at x=`$008C` while the first Level 00 Dragon approached,
and native queued frame-17 contact damage that oracle did not. Fixed in
`aa95d800` by keeping the DragonWalking `+1px` grounded landing-graze allowance
off zero-velocity idle contacts, matching ASM `GfxObjects_CollisionKid` exact
box for this edge. Fresh verification after the fix: strict `verify-recording`
is trace/state green (`trace_match=true`, `oracle_state_match=true`) but still
pixel-red from broad renderer drift; derived 0..120 case
`issue216_prize_audio_probe_x140_case_00000_00120` passes, and `case reduce`
reports it is already passing. Do not reselect this recording as a gameplay
case; remaining #216 work is rendered pixel/waveform alignment and the
level-complete cut-off report.
Note 2026-06-02 for #216: delegated x160 prize-audio probe
`issue-216-prize-audio-probe-x160` initially failed native replay at frame 0
only because older bundles predate the `hud_diamond_frame_offset` trace
diagnostic. Replay verification now treats legacy-missing HUD diamond offsets
as schema drift, not gameplay drift. Fresh verification is trace/state green
(`trace_match=true`, `oracle_state_match=true`) but strict-pixel red from broad
renderer drift; audible windows include Level BGM `Start(6)` at frame 0,
damage `PlaySound(97)` at frame 1, jump `PlaySound(23)` at frame 22, block hit
`PlaySound(13)` at frame 31, and diamond prize `PlaySound(45)` at frame 132.
Derived 0..120 case `probe_case_00000_00120` passes and `case reduce` reports
it is already passing. Do not reselect this recording as a gameplay case;
remaining #216 work is rendered pixel/waveform alignment and the level-complete
cut-off report.
Note 2026-06-02 for #216: delegated x180 prize-audio probe
`issue-216-prize-audio-probe-x180` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift. Audible windows include Level BGM `Start(6)` at frame 0, damage
`PlaySound(97)` at frame 1, jump `PlaySound(23)` at frame 22, block hit
`PlaySound(13)` plus the debris/bounce cue `PlaySound(17)` at frame 31.
Derived 0..120 case `probe_case_00000_00120` passes and `case reduce` reports
it is already passing. Do not reselect this recording as a gameplay case;
remaining #216 work is rendered pixel/waveform alignment and the level-complete
cut-off report.
Note 2026-06-02 for #216: delegated x190-L prize-audio probe
`issue-216-prize-audio-probe-x190-L` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift. Its derived case first failed only on `events`: native case
semantics reported `jump_started` when native-only `on_ground` metadata dropped
during a lethal damage arc after frame-68 `PlaySound(97)`, while movement state
stayed non-jump and native/oracle positions/entities still matched. Case event
derivation now suppresses ground-flag-only jump events for dead players, and the
0..120 `probe_case_00000_00120` passes; `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x190-N prize-audio probe
`issue-216-prize-audio-probe-x190-N` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift. Audible windows include Level BGM `Start(6)` at frame 0, damage
`PlaySound(97)` at frame 1, jump `PlaySound(23)` at frame 22, block hit
`PlaySound(13)` plus debris/bounce `PlaySound(17)` at frame 31. Derived 0..120
case `probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x190-R prize-audio probe
`issue-216-prize-audio-probe-x190-R` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift. Audible windows include Level BGM `Start(6)` at frame 0, damage
`PlaySound(97)` at frame 1, jump `PlaySound(23)` at frame 22, block hit
`PlaySound(13)` plus debris/bounce `PlaySound(17)` at frame 31. Derived 0..120
case `probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x200 prize-audio probe
`issue-216-prize-audio-probe-x200` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift. Audible windows include Level BGM `Start(6)` at frame 0, jump
`PlaySound(23)` at frame 22, block hit `PlaySound(13)` at frame 31, and
`PlaySound(14)` at frame 100. Derived 0..120 case `probe_case_00000_00120`
passes and `case reduce` reports it is already passing. Do not reselect this
recording as a gameplay case; remaining #216 work is rendered pixel/waveform
alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x200-L prize-audio probe
`issue-216-prize-audio-probe-x200-L` exposed a trace-exporter miss after
frame-46 contact damage. Native and oracle positions matched at the level-left
boundary, but native exported `Character_Movement=0` from frame 128 onward
because x_vel was already clamped to zero; oracle kept `MoveID_Walking`.
ASM `loc_8E16 -> sub_8F26 loc_8FF8 -> loc_8E74` shows held-left movement enters
sub_8F26 with non-zero x_vel, then the left-edge clamp clears x_vel while the
negative/NE return preserves the walking continuation. `trace.rs` now keeps
that held-left boundary clamp as walking; focused regression
`current_character_movement_keeps_held_left_boundary_clamp_walking_loc_8ff8`
passes. Fresh full verification is oracle-state green
(`oracle_state_match=true`) and replay-stale only because the cached native
trace predates the exporter fix (`first_trace_drift.frame_index=128`). Do not
reselect this recording as a gameplay case; remaining #216 work is rendered
pixel/waveform alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x200-N prize-audio probe
`issue-216-prize-audio-probe-x200-N` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift. Audible windows include Level BGM `Start(6)` at frame 0, jump
`PlaySound(23)` at frame 22, block hit `PlaySound(13)` at frame 31, and
`PlaySound(14)` at frame 100. Derived 0..120 case `probe_case_00000_00120`
passes and `case reduce` reports it is already passing. Do not reselect this
recording as a gameplay case; remaining #216 work is rendered pixel/waveform
alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x200-R prize-audio probe
`issue-216-prize-audio-probe-x200-R` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift. Audible windows include Level BGM `Start(6)` at frame 0, jump
`PlaySound(23)` at frame 22, and block hit `PlaySound(13)` at frame 31.
Derived 0..120 case `probe_case_00000_00120` passes and `case reduce` reports
it is already passing. Do not reselect this recording as a gameplay case;
remaining #216 work is rendered pixel/waveform alignment and the level-complete
cut-off report.
Note 2026-06-02 for #216: delegated x210-L prize-audio probe
`issue-216-prize-audio-probe-x210-L` exposed a real DragonWalking contact
timing bug. The derived 0..120 case first failed at frame 53: native spent HP
one frame early while oracle kept HP at 2 until frame 54. ASM
`GfxObjects_CollisionKid loc_1C42` uses TheKid's exact 5px hitbox on this
leftward ground pass; the previous DragonWalking `+1px` allowance was too broad
and now applies only to the rightward grounded landing-graze route it was
introduced for. Focused regression
`walking_dragon_leftward_ground_pass_keeps_exact_asm_box_loc_1c42` covers the
x210-L frame-52 edge. Fresh semantic derivation has native HP staying 2 on
frame 53 and dropping on frame 54, and full `verify-recording --slug
issue-216-prize-audio-probe-x210-L` is oracle-state green
(`oracle_state_match=true`). The old recording remains replay-stale because its
cached native trace predates the fix (`first_trace_drift.frame_index=53`), so
do not reselect it as a gameplay case. Remaining #216 work is rendered
pixel/waveform alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x210-N prize-audio probe
`issue-216-prize-audio-probe-x210-N` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179. Audible windows include Level BGM `Start(6)`
at frame 0, jump `PlaySound(23)` at frame 22, and block hit/debris cues
`PlaySound(13)` plus `PlaySound(17)` at frame 31. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.

Note 2026-06-02 for #216: delegated x240-R prize-audio probe
`issue-216-prize-audio-probe-x240-R` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179 (MAE 1.8945638, changed pixels 3837, bounds
x=0..319 y=16..163). Audible windows include Level BGM `Start(6)` at frame 0,
jump `PlaySound(23)` at frame 22, and block hit/debris cues `PlaySound(13)`
plus `PlaySound(17)` at frame 31. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.
Note 2026-06-02 for #216: delegated x210-R prize-audio probe
`issue-216-prize-audio-probe-x210-R` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179. Audible windows include Level BGM `Start(6)`
at frame 0, jump `PlaySound(23)` at frame 22, and block hit/debris cues
`PlaySound(13)` plus `PlaySound(17)` at frame 31. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.

Note 2026-06-02 for #216: delegated x220 prize-audio probe
`issue-216-prize-audio-probe-x220` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179 (MAE 1.9057524, changed pixels 4136, bounds
x=8..310 y=16..163). Audible windows include Level BGM `Start(6)` at frame 0,
jump `PlaySound(23)` at frame 22, and block hit/debris cues `PlaySound(13)`
plus `PlaySound(17)` at frame 31. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.

Note 2026-06-02 for #216: delegated x220-L prize-audio probe
`issue-216-prize-audio-probe-x220-L` is stale beyond the requested clean slice:
fresh `verify-recording` reports current native replay drift at frame 138
(`trace_match=false`, `first_trace_drift.frame_index=138`) while oracle state is
green and strict pixels remain broad renderer drift. Audible windows include
Level BGM `Start(6)` at frame 0, jump `PlaySound(23)` at frame 22, block
hit/debris cues `PlaySound(13)` plus `PlaySound(17)` at frame 31, and hurt
`PlaySound(97)` at frame 60. Derived 0..120 case `probe_case_00000_00120`
passes and `case reduce` reports it is already passing; an exploratory 0..160
case blocks on the stale frame-138 replay drift. Do not patch gameplay from the
later tail; re-record this route if #216 needs the post-120 hurt-SFX window.

Note 2026-06-02 for #216: delegated x220-N prize-audio probe
`issue-216-prize-audio-probe-x220-N` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179 (MAE 1.9057524, changed pixels 4136, bounds
x=8..310 y=16..163). Audible windows include Level BGM `Start(6)` at frame 0,
jump `PlaySound(23)` at frame 22, and block hit/debris cues `PlaySound(13)`
plus `PlaySound(17)` at frame 31. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.

Note 2026-06-02 for #216: delegated x220-R prize-audio probe
`issue-216-prize-audio-probe-x220-R` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179 (MAE 1.9648205, changed pixels 3801, bounds
x=0..319 y=16..163). Audible windows include Level BGM `Start(6)` at frame 0,
jump `PlaySound(23)` at frame 22, and block hit/debris cues `PlaySound(13)`
plus `PlaySound(17)` at frame 31. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.

Note 2026-06-02 for #216: delegated x230-L prize-audio probe
`issue-216-prize-audio-probe-x230-L` is stale against current native at the
hurt-SFX window: fresh `verify-recording` drifts from cached native at frame 66
(`trace_match=false`, `first_trace_drift.frame_index=66`), while oracle state is
otherwise green and strict pixels remain broad renderer drift. Audible windows
include Level BGM `Start(6)` at frame 0, jump `PlaySound(23)` at frame 22,
block hit/debris cues `PlaySound(13)` plus `PlaySound(17)` at frame 31, and
hurt `PlaySound(97)` at frame 66. Derived 0..120 case
`probe_case_00000_00120` is blocked with `source recording replay drifted at
frame 66`; do not patch gameplay/audio timing from this stale bundle. Re-record
this route if #216 needs the frame-66 hurt-SFX case.

Note 2026-06-02 for #216: delegated x230-N prize-audio probe
`issue-216-prize-audio-probe-x230-N` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179 (MAE 1.9206147, changed pixels 4170, bounds
x=0..300 y=16..163). Audible windows include Level BGM `Start(6)` at frame 0,
jump `PlaySound(23)` at frame 22, block hit/debris `PlaySound(13)` at frame 31,
and later `PlaySound(45)` at frame 132. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing; the frame-132 sound is outside the requested reducer window. Do not
reselect this recording as a gameplay case unless deriving a window that covers
the human-visible audio issue.

Note 2026-06-02 for #216: delegated x230-R prize-audio probe
`issue-216-prize-audio-probe-x230-R` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179 (MAE 2.0254836, changed pixels 3881, bounds
x=0..319 y=16..163). Audible windows include Level BGM `Start(6)` at frame 0,
jump `PlaySound(23)` at frame 22, and block hit/debris `PlaySound(13)` at frame
31. Derived 0..120 case `probe_case_00000_00120` passes and `case reduce`
reports it is already passing. Do not reselect this recording as a gameplay
case; remaining #216 work is rendered pixel/waveform alignment and the
level-complete cut-off report.

Note 2026-06-02 for #216: delegated x240-L prize-audio probe
`issue-216-prize-audio-probe-x240-L` is stale against current native at the
hurt-SFX window: fresh `verify-recording` drifts from cached native at frame 73
(`trace_match=false`, `first_trace_drift.frame_index=73`), while oracle state is
otherwise green and strict pixels remain broad renderer drift. Audible windows
include Level BGM `Start(6)` at frame 0, jump `PlaySound(23)` at frame 22,
block hit/debris cues `PlaySound(13)` plus `PlaySound(17)` at frame 31, and
hurt `PlaySound(97)` at frame 73. Derived 0..120 case
`probe_case_00000_00120` is blocked with `source recording replay drifted at
frame 73`; do not patch gameplay/audio timing from this stale bundle. Re-record
this route if #216 needs the frame-73 hurt-SFX case.

Note 2026-06-02 for #216: delegated x240-N prize-audio probe
`issue-216-prize-audio-probe-x240-N` is trace/state green
(`trace_match=true`, `oracle_state_match=true`) and strict-pixel red from broad
renderer drift at tail frame 179 (MAE 3.1885788, changed pixels 5187, bounds
x=0..290 y=16..211). Audible windows include Level BGM `Start(6)` at frame 0,
jump `PlaySound(23)` at frame 22, and block hit/debris cues `PlaySound(13)`
plus `PlaySound(17)` at frame 31. Derived 0..120 case
`probe_case_00000_00120` passes and `case reduce` reports it is already
passing. Do not reselect this recording as a gameplay case; remaining #216 work
is rendered pixel/waveform alignment and the level-complete cut-off report.
Rejected 2026-05-31: a literal `PlayerForm::ground_speed_table` swap (`Ice→RUBBER_*`, `Rubber→ICE_*`) matches a first reading of `sub_8600` / `loc_9AE0`, but regresses the oracle-backed `death-respawn-level1` route. Targeted speed-table unit tests passed, then `compare-native-gameplay --scenario death-respawn-level1` failed at `death_tail@1762` with first live drift at frame 1089 (`character_movement`) on a normal `SlopeUp block=None` frame, not an ice/rubber cell; `verify-slice-fast` also failed the same death/respawn regression lane. Do not retry the symbolic table swap without first deriving an isolated ice/rubber walk-right oracle scenario and reconciling why the current Level 1 oracle path depends on the existing mapping.
Note 2026-06-01 for #229: playtest reported later-level entry/level 4 instant death, but direct settled `play-native` recordings for levels `03` and `04` were not replayable before gameplay could be trusted: `verify-recording --native-only` drifted at frame 0 because `play-native --warp-level NN --settled` recorded from the synthetic `enter_level_playtest` shortcut while the verifier replayed the ROM-like settled recording oracle path. Native settled recording startup now runs `boot_to_level` plus `settle_recording_oracle_warp_live_window`, matching ASM `loc_1AE2E`/`loc_74E0` title/start and level-skip lineage. Refreshed `.cache/recordings/issue-229-level03-entry-current` and `issue-229-level04-entry-current` both verify native-only and show no immediate death over 120 idle frames; the remaining reported instant death needs a non-synthetic level-end-to-next-level repro rather than direct settled warp evidence.
Follow-up 2026-06-06 for #229: selector-picked
`issue-229-level03-entry-current` is still native-stable but not a
gameplay-authoritative reduced case. Fresh `verify-recording --native-only`
passes, and full verification has `trace_match=true`, `oracle_state_match=true`,
`first_trace_drift=null`, and matching 120-frame traces, but strict pixels are
red from frame 0 (`oracle_frame_match=false`, mostly background/sky colour
drift) and the oracle captured no Diamond/MiniHoppingSkull snapshots while
native exported one entity snapshot. The requested `0..120` case derives as
`issue-229-level03-entry-current_case_00000_00120`, but `case verify` marks it
blocked with `likely_subsystems=["recording verification"]`; `case reduce`
refuses because the settled-recording oracle bootstrap reaches Level 03 with a
different initial state (`player x_velocity|y_velocity`, native entities=1 vs
oracle=0). Do not reselect this direct-entry recording as gameplay evidence;
use a non-synthetic level-end-to-next-level repro for the reported instant-death
bug, or fix settled-recording oracle bootstrap before deriving Level 03 entry
cases.
Follow-up 2026-06-06 for #229: selector then advanced to the better
non-synthetic transition route `issue-229-level03-flag-to-level04`. It is also
native-stable (`verify-recording --native-only` passes), and full verification
has `trace_match=true`, `oracle_state_match=true`, `first_trace_drift=null`, and
matching 200-frame traces, but strict pixels are red and checkpoint state deltas
show the oracle starts with camera `x_pos=$05680000` while native starts at
`$05000000` for every checkpoint. The derived
`issue_229_level03_flag_to_level04_case_00000_00120` is blocked with
`likely_subsystems=["recording verification"]`; `case reduce` refuses because
settled-recording oracle bootstrap reaches Level 03 with different player
`x_velocity|y_velocity` and camera `x_pos`. This is the right repro shape for
the reported Level 03→04 hand-off, but it cannot drive gameplay patches until
the settled-recording bootstrap seeds native/oracle from the same camera/motion
state.
Tooling correction 2026-06-06: the case verifier now detects the more precise
root cause for this bundle before running oracle comparison: source recording
probe metadata disagrees with replay frame 0 (`manifest camera=$05700000,$03800000`
but replay camera=`$05000000,$037E0000`). Treat this as stale/dirty recording
metadata, not an authoritative oracle-bootstrap gameplay mismatch.
Follow-up 2026-06-06 for #229: selector then tried dirty-cache route
`issue-229-level03-flag-to-level04-boundary`. Fresh native-only verification
drifts at frame 308 (`recording replay drifted`, matching 1800-frame
recorded/replay traces, no frame diffs), so the tail cannot drive current
gameplay work. The requested `0..120` prefix case passes with matching
`world.game_mode`, camera, player position, entities, and events. Do not patch
#229 from this clean prefix or stale tail; re-record the Level 03→04 boundary
route on current native after fixing/seeding settled bootstrap, or build a
longer non-synthetic route that remains replay-stable through the reported
Level 04 entry death.
Tooling correction 2026-06-06: after adding the same probe-metadata guard, the
boundary prefix is no longer treated as clean semantic evidence: its manifest
camera y=`$03800000` also disagrees with replay frame 0 camera y=`$037E0000`.
The stale-tail conclusion still stands, but the prefix should be treated as
recording-verification blocked until the route is re-recorded cleanly.
Follow-up 2026-06-06 for #229: delegated case-queue route
`issue-229-level03-flag-to-level04-counterend` is also blocked as recording
evidence. Fresh native-only verification drifts at frame 254 (`recording replay
drifted`, matching 1800-frame recorded/replay lengths, no frame diffs). The
requested `0..120` case derives, but `case verify` fails before oracle
comparison because the source manifest camera y=`$03800000` disagrees with
replay frame 0 camera y=`$037E0000`; `case reduce` refuses with the same
recording-verification blocker. Do not patch #229 from `counterend`; re-record a
current Level 03→04 route with matching probe metadata first.
Follow-up 2026-06-06 for #229: delegated case-queue route
`issue-229-level03-flag-to-level04-countermid` has the same blocker. Fresh
native-only verification drifts at frame 254 with matching 1800-frame
recorded/replay lengths and no frame diffs. The derived `0..120` case fails
before oracle comparison because manifest camera y=`$03800000` disagrees with
replay frame 0 camera y=`$037E0000`; `case reduce` refuses with the same
recording-verification blocker. Treat `countermid` as stale recording evidence,
not gameplay-authoritative #229 proof.
Follow-up 2026-06-06 for #229: delegated case-queue route
`issue-229-level03-flag-to-level04-counterstart` also matches the stale counter
recording signature. Fresh native-only verification drifts at frame 254 with
matching 1800-frame recorded/replay lengths and no frame diffs. The derived
`0..120` case is blocked before oracle comparison by the same manifest/replay
camera-y mismatch (`$03800000` vs `$037E0000`), and `case reduce` refuses. The
`counterstart`/`countermid`/`counterend` set should now be treated as exhausted
recording-verification blockers, not #229 gameplay patch inputs.
Follow-up 2026-06-06 for #229: delegated case-queue route
`issue-229-level03-flag-to-level04-dense` is blocked by the same stale recording
signature. Fresh native-only verification drifts at frame 254 with matching
1800-frame recorded/replay lengths and no frame diffs. The derived `0..120`
case is blocked before oracle comparison by manifest/replay camera-y mismatch
(`$03800000` vs `$037E0000`), and `case reduce` refuses. Do not patch #229 from
`dense`; a fresh current route is needed.
Follow-up 2026-06-06 for #229: delegated case-queue route
`issue-229-level03-flag-to-level04-long` is likewise exhausted. Fresh
native-only verification drifts at frame 254 with matching 1800-frame
recorded/replay lengths and no frame diffs. The derived `0..120` case is blocked
before oracle comparison by manifest/replay camera-y mismatch (`$03800000` vs
`$037E0000`), and `case reduce` refuses. The current cached
`level03-flag-to-level04*` recording family is stale for gameplay purposes; get
a fresh current recording before using #229 as a patch target.
Follow-up 2026-06-06 for #229: delegated case-queue route
`issue-229-level04-entry-current` is native-stable and semantically clean, not an
instant-death gameplay repro. Fresh native-only verification passes, full
recording verification has `trace_match=true`, `oracle_state_match=true`, and
`oracle_frame_match=false`, and the derived `0..120` case passes with matching
`world.game_mode`, camera, player position, entities, and events. The remaining
evidence is visual: captured frame 0 shows native rendering the Level 04 cave
background as black while oracle renders dark red cave silhouettes
(`changed_pixel_count=9416`, mostly native RGB `0,0,0` vs oracle `36,0,0`).
Track this as Level 04 palette/background fidelity, not #229 gameplay logic.
Note 2026-06-01 for #230: first live storm slices landed. `NativeRuntime::init_weather` now seeds weather 2/3 storm worker scalars from ASM `loc_14DAC` (`$FFFFFB3C/$FB3D/$FAD6/$FADA/$FADC/$FB52/$FB54`), live weather 2/3 now enters the shared `sub_6048` state update path, and live hail timing now uses ASM `loc_6172/loc_619C` frame-counter gate `Frame_Counter > $873` plus ROM active-period reloads (`$4B0` inactive→active, `$258` active→inactive) instead of the coarse level-timer/flat-600 approximation. The full 5-slot `sub_61CA` worker, applying palette ops from `sub_6048`, visible weather 2 thunder/storm effects, lava geyser state machine, and palette/shore worker spawns remain open.
Follow-up 2026-06-01: live hail projectile spawn cadence and initial position now use ASM `sub_61CA` `loc_623A`/`loc_626C` values: `$FFFFFB3C` decrements as an 8-bit cooldown, reloads to `$25 - (Get_RandomNumber_byte&$0F)`, and the spawn X is `Camera_X_pos + Get_RandomNumber_byte + $28` with spawn Y at `Camera_Y_pos`. This removes the native fixed 30-frame timer and `(frame_index*7+13)%320` X approximation. Remaining hail gap: generic projectile storage still stands in for the ROM's five falling slots, collision-map splash/impact slots, and `$28` Kid push timer.
Follow-up 2026-06-01: live lava geysers now preserve ASM `loc_14772` per-geyser startup hibernate on `LavaGeyser` and reuse it after collapse via `loc_1498C -> loc_14788`, removing the global `LAVA_GEYSER_DORMANT_FRAMES=60` reset. This fixes the Black Pit/height-indexed stagger after the first cycle. Larger lava gaps remain: the rise/hold/collapse cadence is still the native 4px ramp rather than the discrete ROM row script, and live palette/art upload workers are still unwired.
Follow-up 2026-06-01: USM2 lava fireball initialisation now comes from the ROM position table's `$8000` sign-bit entry and flows through `initialise_lava_fireball_state_loc_1499C` instead of a separate hardcoded spawn. Remaining fireball gap: the live path still uses the parallel static `LAVA_FIREBALL_SCRIPT` runner rather than the full ROM object script walker, though the script values are already pinned by tests.
Follow-up 2026-06-01: Mountain storm thunder now queues the live `sfx_Thunderstorm` path. `step_weather` consumes `sub_6048`'s `play_thunder_sound` flag and calls the existing ASM `sub_1B36C` helper, so weather 2/3 no longer drop the thunder sound effect when `$FFFFFADC` reaches zero. Remaining visible storm gap: palette writes from `sub_6048` are still modelled as ops but not applied to live palette/renderer state.
Follow-up 2026-06-06 for #240: delegated case-queue recording
`issue-240-l0-dragon-idle` is stale around the actual damage/fire window. Fresh
native-only verification drifts at frame 234, matching the transcript's first
damage event and `PlaySound(97)`, with matching 700-frame recorded/replay
lengths and no frame diffs. The derived `0..120` prefix passes semantically, but
that window is before the dragon breath interaction. A focused `220..320` case is
blocked because the source recording replay drifts at frame 234, and `case
reduce` refuses. Keep #240 closed from the older oracle-backed settle proof; do
not use this stale idle recording as new gameplay evidence unless re-recorded.
Follow-up 2026-06-06 for #248: settled the interactive respawn capture with `settle_oracle_warp=true`; oracle proof from `issue-248-death-start-resume-settled` shows the first Start-held frame at 560, stays `Game_Mode=$0008` for frames 560/561, and returns to gameplay `Game_Mode=$000C` at frame 562. 2026-06-08 correction: a threshold-only native change matched `Game_Mode` timing but was wrong because it released ordinary level runtime state at frame 562; oracle still exposes the hidden `loc_6E2`/`Load_InGame` bootstrap pose and title-card object counts there. Native now keeps a short live-mode hand-off: trace mode is `$000C` from frame 562, while the respawn bootstrap continues to own Kid/camera/object state until the existing Load_InGame carry window expires. `tas-diff --scenario .cache/recordings/issue-248-death-start-resume-settled.json --frame-start 555 --frame-end 565 --include-advisory --frontier-kind gameplay` is now 10/10 matched. Remaining #248 follow-ups are unchanged: title-card rendering during the hold and deriving the no-input auto-timeout from oracle.
Review evidence 2026-06-06 for #248: `mise run human-validation-review` now
regenerates `.cache/debug/issue248-start-resume-review/` from
`issue-248-death-start-resume-settled.json` and includes the frames in
`.cache/human-validation/index.html`. Native trace probing shows the rendered
black hold through frame 562 (`LevelTransition`) and restored visible gameplay
from frame 563 (`InGame`, Kid alive, lives=2, HP=2); frame 575 remains restored.
This proves the current Start-resume fix is reviewable by humans, but does not
close the two documented residuals: title-card art during the hold and the
no-input auto-timeout timeline still require oracle-derived work.
Playtest recheck 2026-06-07 for #248/#257: human feedback says death still
goes black and only recovers when Start is pressed. That confirms the
Start-held hand-off works, but the user-facing bug remains open: the hold is
still black/confusing and no-input recovery is still wrong. Do not split this
into a separate death-state-machine fix; continue #248 by capturing the
no-input/progressing oracle respawn timeline for `lose_life -> Game_Mode=$0008
-> loc_6E2` and then derive the live timeout/title-card rendering from that
evidence.

Rejected approach 2026-06-09: do not "fix" #248 by making the generated
title-card `StartingLevel` path consume an old latched Start at
`level_settle@719`. Fresh repro:
`cargo run -q -p kidcli -- compare-native-gameplay --scenario pause-resume-level1`
first failed with native `Game_Mode=$0008` vs oracle `$000C` at
`level_settle@719`. A trial patch to let the title-card branch consume
`start_latched || start_pressed` moved native into `InGame`, but only at frame
705; the compare then failed on the known opening-dragon timing (`hp 2` vs
oracle `hp 1`, camera/Kid X still behind). More importantly, that early release
contradicts the full-frame boot-title contract: cache-backed tests
`boot_title_start_stays_hidden_through_frame_520_then_enters_ingame` and
`boot_title_start_level_1_settles_slot_2_and_number_worker_on_frame_719`
intentionally keep the generated title-card shell hidden at frame 719 with the
settled loc_1AC70/loc_1AD48 worker pose. Root issue is a comparison-mode split:
gameplay-state oracle captures expose live gameplay by frame 600, while
full-frame title-card fidelity keeps mode-$0008 visuals through frame 719. Keep
the engine contract intact; the next pass should reconcile the gameplay compare
harness/oracle classification or derive a mode-specific native capture path,
not shorten `EARLY_START_LOAD_INGAME_HANDOFF_FRAMES` blindly.
Note 2026-06-04 for #249: added named Blue Lake Woods 2 pit audits
`issue249-blw2-left-pit-wall-no-jump-audit` and
`issue249-blw2-left-pit-wall-jump-audit`. The block-grid scan pins the reported
rubber-block drop pit's left wall at level 0x01 block col 20 rows 10..35
(x=320, y=160..576), and both scenarios seed Juggernaut/Rhino beside it at
(352,192) with camera (240,80). Builder tests guard the level, coordinates,
helmet, and jump/no-jump masks. Fresh recording
`.cache/recordings/issue-249-blw2-left-pit-wall-jump-audit` is native-replay
deterministic, but full oracle verification still blocks at checkpoint 30 on
settled-probe camera divergence only (`native 192,203`, `oracle 234,442`), so
this is now the concrete repro harness; the next fix is either the settled
camera bootstrap or a comparison mode that can isolate Kid/wall state for this
probe.
Follow-up 2026-06-04: recording verification now keeps ordinary state compares
camera-strict but ignores camera position for explicit `camera_position` probe
recordings so those mid-level visual patches do not hide Kid-state feedback.
Follow-up 2026-06-06: added cache-backed native regression
`issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_juggernaut`, running
the BLW2 no-jump scenario with generated level data. Native stays on level
0x01 as helmet 5, grounded at y=$00A00000 with y_velocity=0, ledge snap
cooldown=0, `jump_buffer=0`, and `wall_cling=none` at scenario frames 30/90/179.
This confirms the current direct BLW2 left-wall probe does not reproduce the
reported no-input launch; keep #249 open pending a natural route or richer probe
that reproduces the live playtest failure.
Re-running the pit jump audit now gets past the camera false-blocker and fails
on the actual probe-state mismatch at frame 30: native has already bounced/moved
from the rubber pit seed (`kid=(348,139)`, moving up-left), while oracle remains
at the seeded WRAM pose (`kid=(352,560)`, zero velocity). Next action: reconcile
probe-start rubber-bounce/live-object semantics for this seed, or choose a
stable grounded wall seed before changing wall-cling gameplay.
Follow-up 2026-06-06: moved both BLW2 pit probes to the top of the same left
wall column (`start=(352,192)`, `camera=(240,80)`) so native no longer hits the
lower rubber/bridge bounce before the wall audit. The refreshed no-jump and
jump recordings are native-replay deterministic. Full oracle verification still
fails early because the helper's direct WRAM `start_position` patch does not run
the same floor-snap/live-state setup as native (oracle remains at the patched
pose while native settles to the wall/slope at `kid=(343,160)`). Do not infer a
wall-cling gameplay bug from this oracle mismatch; the next real oracle pass
needs either a naturally reached route to the BLW2 shaft or a richer probe seed
that patches the ROM's grounded/collision state consistently.
Rechecked 2026-06-06 on current HEAD with fresh recording
`issue-249-blw2-left-pit-wall-no-jump-current-20260606`: native-only replay is
deterministic and the focused regression still shows no no-input launch
(`cargo test -q -p kidcli issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_juggernaut -- --nocapture`,
plus the builder-coordinate test). Full oracle verification is still blocked
before it can judge wall logic: `left_pit_wall_contact_loc_a8de@30` starts from
different poses (`native 01570000,00A00000`; `oracle 01600000,00C00000`). The
derived case `issue_249_blw2_left_pit_wall_no_jump_audit_01_case_00000_00120`
now reaches replay-matched verification after teaching case provenance to use
the recorded effective frame-0 camera rather than the requested pre-step camera.
It still blocks before gameplay because settled oracle bootstrap reaches Level
01 with a different initial state (`player fields=y_pos`, native entities=4,
oracle entities=2). Keep this open for a natural BLW2 shaft route or a settled
direct-probe oracle-bootstrap fix that aligns the post-probe floor/camera settle
and enemy-load phase; do not spend another pass treating this WRAM-patched probe
as oracle-authoritative.
Oracle settle trace follow-up 2026-06-06: added diagnostic
`oracle-probe --capture-settle-trace` rows for the hidden helper settle loop and
ran them against
`issue-249-blw2-left-pit-wall-no-jump-current-20260606`. The hidden Level 01
warp settle itself matches native `inspect-respawn-settle --level 01`: both
reach `kid=00200000,00BF0000`, `camera=00000000,001C0000`, zero velocity, and
`number_objects=4` before the direct probe patch. Therefore the remaining #249
bootstrap blocker is not generic BLW2 warp settle; it is the first scenario
state after applying `start_position`/`camera_position`, where the oracle direct
probe lacks the native post-probe floor/entity-load phase. Next fix should target
settled direct-probe application or use a natural BLW2 shaft route, not wall-cling
gameplay.
Narrowed follow-up 2026-06-06: the direct-probe blocker is not just the
`playtest_form` seed. Oracle watches on the current no-jump recording show
`playtest_form=5` at the natural BLW2 spawn advances from Kid object code
`$75E2` to `$8BFE` by frame 2 and walks left like the base form. A direct
`start_position` without `playtest_form` also moves, though as base Kid in a
jump/fall state. The failing combination is specifically Juggernaut/Rhino
(`Current_Helmet=5`, half-width `$17`) patched to `(352,192)`: the ROM stays at
Kid object code `$75E2`, `Character_Movement=0`, zero velocity, no platform
handoff, and the patched pose for frames 0..8. This matches ASM `loc_75D4` /
`loc_8D72` standing/collision fallback, not the wall-cling path. Next pass
should either reach the shaft naturally or build a richer direct seed that
patches enough grounded/collision context for this exact wider-form pose before
using oracle diffs as gameplay evidence.
Follow-up 2026-06-09: the newer debug bundle
`issue-249-blw2-left-pit-wall-no-jump-current-debug-20260608` is fresh but
does not unblock wall-cling gameplay. `verify-recording` is native-replay
deterministic (`trace_match=true`) yet still fails before wall logic at
`left_pit_wall_contact_loc_a8de@30`: native Kid `01600000,009F0000` versus
oracle `01600000,00C00000`, with camera already divergent too. The requested
0..120 case derives to
`.cache/cases/issue-249-blw2-left-pit-wall-no-jump-current-debug-20260608` and
`case verify` reports FAIL, but `case reduce` refuses it as blocked because
settled Level 01 oracle bootstrap reaches gameplay with a different initial
state (`player fields=y_pos`, entities native=4 oracle=2, first native
`0A@00A48000,00E00000` versus oracle `0A@00A40000,00D80000`). This is another
direct-probe bootstrap failure, not evidence that native wall-cling should
change; continue with a natural BLW2 shaft route or fix settled direct-probe
seeding before using this family as oracle-authoritative.
Note 2026-06-04 for #252: current render recheck after the #145 baseline fix shows the live RockTank body/turret pair is now compact rather than doubled. ASM `Tank.asm loc_3B372` / `Tank_Shooting.asm loc_3B5FA` still creates the linked child at `body_y-8`; native must compensate that top-anchored hardware offset when rendering child snapshots. Added a regression on the actual child-snapshot path (`rock_tank_renderer_uses_child_snapshot_instead_of_duplicate_synthetic_turret`) asserting the turret X equals the body X and the corrected body/turret top delta is 20px, matching the existing synthetic fallback assertion. Fresh native capture from `.cache/tas-verify/issue213-x568-y575-w340-jump_b_hold-1200/bisect/scenario.json` frame 200 (`.cache/debug/issue-252-current/..._frame_00200.png`) shows a single coherent tank above the floor. Remaining #252 user-facing risk is the invisible death report, tracked by #167; do not reopen tank gameplay state or duplicate-object hypotheses without a new visual repro.

Note 2026-06-06 for #252: rechecked the Level 02 shooting-tank art path against ASM `Tank_Shooting.asm loc_3B5FA` and sprite descriptors `unk_C7EC8`/`unk_C7ED0`/`unk_C7F10`. Fresh dumps in `.cache/debug/issue252-current/` show `level02_enemy0b_turret_idle.png`, `level02_enemy0b_body_idle.png`, and `level02_enemy0b_turret_muzzle.png` assemble coherent RockTank parts; the blue/purple pixels in the muzzle frame are the gun barrel, not the earlier suspected humanoid/two-person tile selection. Tightened `rock_tank_shooting_renderer_uses_stru_3b4d8_turret_frames` so the `$0B` muzzle-open frame now also asserts linked child X parity and the corrected 24px body/turret top delta, proving the renderer compensates the ROM's `body_y-8` child storage on the shooting-tank path. The doubled/half-floor render subclaim is therefore covered on current code; keep #252 open only for a fresh live death-visibility repro, otherwise handle it under #167.

Note 2026-06-06 recheck for #252: current HEAD still covers the render-side tank claims. Verified `cargo test -q -p kidcli rock_tank_renderer -- --nocapture` (3 tests), `cargo test -q -p kidcli rock_tank_shooting_renderer_uses_stru_3b4d8_turret_frames -- --nocapture`, and `cargo test -q -p kidcli build_scene_keeps_rock_tank_death_body_visible_loc_3b480 -- --nocapture`. Native sprite assembly keeps the `$0A/$0B` body+turret pair compact and baseline-correct, and the scene path emits `stru_3B29A` / `stru_3B518` death-body frames from ASM `Tank.asm loc_3B480` / `Tank_Shooting.asm loc_3B6E6` when given a death snapshot. Proposed fix path: do not patch Tank gameplay or live turret assembly from stale #213 bundles; #252 stays `[?]` pending either human confirmation that current live tank visuals are fixed, or a fresh deterministic live Tank/RockTank final-hit route if the invisible-death symptom persists. Route that remaining death-visibility work through #167.
Follow-up 2026-06-09 after latest plush playtest: reproduced the remaining skip-intro black-silhouette window with Start at 2450 and 2500 on `.cache/recordings/issue-playtest-title-menu-current-long-20260607/scenario.json`. Before the fix, native exposed a partially initialised title hidden-shell rectangle from 2455..2500 / 2505..2540 (AE about 53k pixels) while oracle either kept `sub_1B43A/loc_1B446`'s blue hidden transformation shell or `loc_1B45A`'s blackout. Native now carries the still-visible IntroSequence6 VDP through early fast-restore handoffs and blanks the later fast-restore handoff until the title VDP is ready. After capture, Start=2500 frames 2520/2540 are AE 0, Start=2450 frame 2460 shows the blue shell instead of the black rectangle and recovers to AE 0 by frame 2520. Regression: `cold_boot_intro_sequence_6_mid_start_keeps_hidden_shell_or_blackout_loc_1b446_1b45a`. Gate: focused cache-backed `cold_boot_intro_sequence_6` tests and `mise run verify-slice-fast` passed.
Follow-up 2026-06-09 after latest plush playtest: fixed the remaining Start hand-off latch leak. `loc_1AE2E` was already suppressing gameplay physics while `StartingLevel` owned the title-card shell, but the raw hand-off frame still wrote B/Right into native `previous_input`; releasing buttons on the first live frame could therefore export title-card gameplay bits into `Ctrl_Held`. Commit `a9a1eb52` now scrubs gameplay buttons from the latch on `StartingLevel` frames while preserving Start for the hand-off. Regression `loc_1ae2e_title_card_start_handoff_does_not_buffer_gameplay_input` covers the old leak, `loc_1ae2e_title_card*` and native-player `title_card_loc_1ae2e` focused tests pass, and `mise run verify-slice-fast` passed.
Note 2026-06-07 for #257/#248: fresh `compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 8950 --frame-end 9000 --checkpoint-interval 10` showed the oracle still in `Game_Mode=$0008` with lives/hp `2/2`, cleared camera, and Kid at `$0040,$0068`, while native had auto-released to live gameplay, taken another death, and re-entered the shell. Fixed the no-Start lower-bound drift by extending `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES` to 10000 and adding `respawn_hold_without_start_covers_oracle_frame_9000_lower_bound_loc_6e2`; the same oracle window now passes. Follow-up 2026-06-08 extended the scenario/checkpoint tail to frame 9500 and `compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 9000 --frame-end 9500 --checkpoint-interval 50` also passes. Follow-up 2026-06-08 extended the same no-input route again to frame 12200: before the fix, `compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 10550 --frame-end 12250 --checkpoint-interval 50` failed at frame 11800 because native had released to `Game_Mode=$000C` at first live drift frame 11792 while oracle remained `Game_Mode=$0008`; extending `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES` to 10500 and adding `respawn_hold_without_start_covers_oracle_12200_route_lower_bound_loc_6e2` makes that window pass through 12250. Follow-up 2026-06-08 extended the no-input lower-bound route to frame 16000: before the fix, `compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 12200 --frame-end 16000 --checkpoint-interval 100` failed at frame 12300 with first live drift frame 12292 (`native Game_Mode=$000C`, oracle `$0008`); extending `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES` to 14210 and adding `respawn_hold_without_start_covers_oracle_16000_route_lower_bound_loc_6e2` makes the same window pass through 16000. Follow-up 2026-06-08 extended the no-input route to frame 22000: with the old 16000 lower bound, `compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 16000 --frame-end 22000 --checkpoint-interval 500` failed at frame 16500 (`lives` native 1 vs oracle 2) with first live drift at frame 16002; extending `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES` to 20210 and adding `respawn_hold_without_start_covers_oracle_22000_route_lower_bound_loc_6e2` makes the same 16000..22000 window pass. Visual follow-up 2026-06-08: native/oracle captures for `.cache/recordings/issue-245-death-respawn-render/scenario.json` proved frame 1800 was pure black in native (1 colour) while oracle shows the Blue Lake Woods title-card shell (16 colours). Native now exposes the `Load_TitleCard` VDP shell during `Respawning`; recapture drops frame-1800 AE from 13158 to 1436 and shows the background/tree card instead of black. Follow-up 2026-06-08: the respawn trace path now exposes the same `Load_TitleCard` `loc_1AB26`/`loc_1AD48` title-letter and deferred level-number workers whenever `lose_life -> Teleport loc_B8DE -> loc_6E2` rebuilds them, even on levels without route-specific Kid-pose bootstrap profiles. Regression `death_respawn_exposes_title_workers_without_level_specific_bootstrap_profile_loc_b8de` covers the worker omission. #257 stays open for deriving the eventual no-input release point beyond frame 22000 and any remaining pixel/audio parity. Audio follow-up 2026-06-08: ASM `lose_life` enters `Teleport`/`loc_6E2`, then `LoadGameModeData` dispatches `Game_Mode=$0008` to `Load_InGame`; `Load_InGame` runs `j_StopMusic` at kid.asm:23629 before `loc_12004` queues the foreground BGM. Native play audio now mirrors that by stopping an active level cue during the death-respawn `LevelTransition` and restarting the level BGM when gameplay returns.
Follow-up 2026-06-09: extended the no-input death-respawn lower bound again. Extending `death-respawn-level1` to frame 26000 first made `compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 22000 --frame-end 26000 --checkpoint-interval 500` fail at frame 22500 (`lives` native 1 vs oracle 2) with first live drift frame 22016 (`game_mode`), proving native still released from `loc_6E2` too early. Native now keeps `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES=24210`, the scenario has `respawn_bootstrap_tail_26000`, regression `respawn_hold_without_start_covers_oracle_26000_route_lower_bound_loc_6e2` covers the no-Start countdown, and the same 22000..26000 oracle window now passes. #257 still needs the eventual no-input release point beyond frame 26000 or an interactive oracle capture that proves the real visible return timing.
Follow-up 2026-06-09 later: attempted to probe the next no-input lower bound by extending `death-respawn-level1` to frame 30000 and running `compare-native-gameplay-window` for 26000..30000, 26000..26500, and 26000..26100. All high-frame windows stalled; the shortest was killed by `timeout 240s` with no mismatch/pass output and only empty cache directories. The scenario extension was reverted because it had no oracle evidence. Next #257 pass should first make the high-frame oracle probe cheaper/resumable (for example by reusing a cached oracle trace or exposing `skip_frames_before` through the gameplay-window path) or capture an interactive Start/resume oracle route; do not extend `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES` beyond 24210 without a completed oracle window.
Tooling follow-up 2026-06-09: added `compare-cached-native-gameplay` so completed windows can be rechecked from `.cache/native-gameplay-oracle-runs/<slug>/{native,oracle}/summary.json` without deleting or recapturing the oracle run. Verified against `death_respawn_level1_window_22000_26000_step_500`, which still passes. This removes the recheck cost for existing evidence, but #257 still needs a new completed oracle capture beyond frame 26000 or an interactive Start/resume route before changing the no-input timer again.
Proposed-fix/evidence follow-up 2026-06-09: commit `91817c74` keeps #257 in `[?]` state and routes release-gate death-respawn coverage through full gameplay verification instead of the static native gameplay runner. The static runner starts inside the synthetic `Game_Mode=$0008` shell and cannot prove full boot/oracle death-respawn scripts directly (`verify-native-gameplay --scenario death-respawn-level1` stayed in `$0008` before live checkpoints; `verify-native-gameplay --scenario death-respawn-start-resume-level1` used the scheduled Start as a level-start handoff rather than a death-respawn resume). Full gameplay verification is the authoritative repro path for these scripts: `verify-gameplay --scenario death-respawn-start-resume-level1` passes with the `resumed_tail` checkpoint moved to frame 1941, and `verify-gameplay --scenario death-respawn-level1` passes through the existing frame-26000 hidden-bootstrap tail. ASM labels covered by the evidence remain `lose_life`, `loc_6E2`, `Load_InGame`, `loc_1AE2E`, and the `loc_1AB26`/`loc_1AD48` title-card workers. This is a proposed verification/routing fix for the known Start-resume path and release-gate blind spot; #257 still needs either a completed no-input oracle window beyond frame 26000 or a human-confirmed interactive visible-return playtest before closure.

Latest #258 follow-up 2026-06-08: aligned the frame-382 and frame-479 Juggernaut/Skycutter armoured `$14` contacts. Native now follows ASM `sub_A4EE loc_A53C -> loc_A556` by bypassing the old-X landing gate for armoured forms, snapping Y from the cached `$FB6C` SlopeUp tile while preserving the post-vertical low word, and keeping `x_vel/y_vel=$FFFE0000` even when the broad native probe sees the adjacent solid landing. Full 0..1020 route now advances to first CRC divergence frame 516; narrow `tas-diff` shows authoritative `kid_y_vel` drift already at frame 515, where native clears rising wall-cling velocity to `0` but oracle keeps `0xFFFB_DD00`. Next gate: derive the `sub_ADB8/sub_AE6E` rising wall-cling release path before closing #258 or moving to another form route.

Latest #258 follow-up 2026-06-08: aligned the frame-515 ascending wall-cling release sampled state. Native now mirrors the existing descending `loc_AB0A` sampled-state quirk for `loc_AB22 -> loc_AB4A`: release the cling latch and shimmy X, but preserve the post-gravity live `y_vel` until the object/export timing can distinguish the ROM's internal clear from the oracle checkpoint. Full 0..1020 route now advances to first CRC divergence frame 586. Narrow `tas-diff` frames 585..587 shows the next blocker starts as `number_objects` at frame 585: native has an extra `floating_item#07@728,181` while oracle has only the two diamonds plus Kid/flag/helper objects; by frame 586 native has also damaged Kid (`number_hitpoints` 3 vs 5) and advanced Kid to `$02F0A900,$00C6A000`, while oracle leaves the Kid slot at `$02F04F00,$00C58000` with hibernate bit clear. Next gate: derive the extra floating-item/damage or post-release hibernate ordering before continuing #258.

Correction 2026-06-08: do not treat the frame-585 Iron Knight mask disappearance as a successful pickup. A trial using Juggernaut form-width collection made native transform at frame 576 (`current_helmet_id=6`, `currently_transforming=true`) while oracle still has Juggernaut (`current_helmet_id=5`) and keeps the mask object alive through frame 584. Oracle removes the mask and its prize-block flash at frame 585 without a form change, so the next gate is the prize object/delete or collision-type timing around `PrizeHelmetIronKnight_Init -> loc_105D6 -> sub_10C38`, not broader player pickup overlap.

Correction 2026-06-08 follow-up: rejected two native helmet-delete shortcuts while investigating the same frame-585 blocker. A direct `GfxObjects_CollisionKid` form-box delete (`PrizeHelmetIronKnight_Init -> loc_105D6`) removes the Iron Knight mask one trace frame too early: native `tas-diff` frames 584..588 reports frame 584 `number_objects` native=6 vs oracle=7, while oracle still lists the mask at `0xffff10cc:code=0x0105da,hib=1@728,181` and the Drips helper at `0xffff0f70`. Adding a "Kid foot reaches helmet lower edge" gate using the observed frame 196->197 route shape still leaves the same frame-584 object-count miss, so the mismatch is not solved by a simple pickup-vs-form-box threshold in `collect_floating_items`. Do not re-land those shortcuts. Continue from object/update ordering: why the native floating-item trace roster has already dropped the mask by frame 584 while oracle deletes it at frame 585, then the remaining frame-586 damage/motion divergence (`number_hitpoints` native=3 vs oracle=5, native advances to `$02F0A900,$00C6A000` while oracle holds `$02F04F00,$00C58000`).

Correction 2026-06-08 follow-up: do not chase flagpole cleanup for the #258 frame-585 Juggernaut route. `CharacterCollision_TouchFlagpole loc_C070` should clear floating prize threads via `Delete_AllButCurrentObject`, and native now does, but rerunning `tas-diff --scenario .cache/recordings/issue-258-level00-slope-juggernaut-natural-20260608/scenario.json --frame-start 582 --frame-end 588 --include-advisory --include-physics-trace --frontier-kind gameplay` still leaves the extra native Iron Knight floating item at frame 585 and damage/motion drift at frame 586. Continue #258 from prize collision/update ordering, not EOL cleanup.

Follow-up 2026-06-08: fixed the frame-585 object-count half of the Juggernaut route by modelling the object-side helmet collision latch separately from lower-body helmet collection. ASM `PrizeHelmetIronKnight_Init -> loc_105D6 -> loc_105EE` deletes the prize object after its own `collision_type(a3)` is latched, while Kid-side `CharacterCollision_TouchSpecialPrize -> sub_7ACC` can still avoid awarding the new form. Native now keeps the mask through frame 584 (`dy=-17`, `latched=1`) and deletes it before frame 585, matching oracle without transforming Juggernaut. Narrow gate `tas-diff --scenario .cache/recordings/issue-258-level00-slope-juggernaut-natural-20260608/scenario.json --frame-start 583 --frame-end 587 --include-advisory --include-physics-trace --show-object-roster-on-match --frontier-kind gameplay` now reports frames 583..585 match. New first blocker: frame 586 authoritative damage/motion drift remains (`number_hitpoints` 3 vs 5; native advances to `$02F0A900,$00C6A000`, oracle holds `$02F04F00,$00C58000`). Enemy-frontier debug shows oracle object `0xffff0da0:code=0x00a71c` hibernates at frame 586 while native continues normal airborne physics, so continue from post-prize collision hibernate/damage ordering.

Follow-up 2026-06-08: fixed the frame-586..599 damage/motion half of the same Juggernaut route. Oracle shows the latched Iron Knight prize collision exposes the touched helmet's `sub_7EB2` HP seed but keeps `Current_Helmet=Juggernaut` and holds Kid at the `loc_A6F8` hibernate pose; modelling that as a full pending-form transform was rejected because it changed `current_helmet_id` to Iron Knight on frame 585. Native now keeps the object-side delete latch, seeds the touched helmet HP, and hibernates the player without changing form; the checked `unk_7EC2` transform HP table was corrected to the ASM bytes (`IronKnight=5`, `Berzerker/Maniaxe/Micromax=3`). Narrow gate `tas-diff --scenario .cache/recordings/issue-258-level00-slope-juggernaut-natural-20260608/scenario.json --frame-start 584 --frame-end 600 --include-advisory --frontier-kind gameplay` reports 16/16 match, and `mise run verify-slice-fast` passes. Next #258 gate: extend the natural Juggernaut route beyond frame 600 / rerun the 0..1020 window to identify the next blocker before closing the live slope-stuck complaint.

Follow-up 2026-06-08: extended the same latched prize hibernate through `Kid_Transform`. Native now uses the old+new ASM transform table lengths, switches `Current_Helmet` at `loc_7CD0` timing, and clears velocities at `loc_7E90` cleanup before falling back into regular movement. Narrow `tas-diff --frame-start 612 --frame-end 625` matched 13/13, and `verify-tas --frame-start 0 --frame-end 1020 --checkpoint-interval 1 --frontier-kind gameplay --bisect` now advances to first CRC divergence frame 712. The remaining frame-700..711 `number_objects` display is advisory/object-roster accounting only; the next authoritative #258 blocker is the frame-712 SlopeUp landing, where native lands earlier at `$03014700,$00DE0000` while oracle keeps the Kid slot at `$0300EF00,$00DF0000` in the armoured/slope path.

Follow-up 2026-06-08: aligned the frame-712 Iron Knight SlopeUp/Solid seam in the same natural Juggernaut route. The route has transformed to Iron Knight by `loc_7CD0`, so the reduced gate must use Iron Knight rather than the earlier Juggernaut bounce path. Native now lets the `loc_A99E -> loc_8BF0 -> loc_9D22` low-word seam correction run without the air-cap handoff tail, matching oracle frame 712 at `$0300EF00,$00DF0000` with `x_vel/y_vel=$0001EF00/$FFFE1100`. Full 0..1020 `verify-tas --frontier-kind gameplay --bisect` now advances to first CRC divergence frame 841. Narrow `tas-diff --frame-start 837 --frame-end 847 --include-advisory --include-physics-trace` shows the next authoritative blocker is another SlopeUp/Solid landing: native and oracle agree on Y/velocity at frame 841, but native lands `0x5300` subpixels too far right (`$03DDAA00` vs `$03DD5700`), then camera/Y drift follows.

Follow-up 2026-06-08: aligned the frame-841..843 Iron Knight compound SlopeUp continuation in the same #258 natural route. Native now models ASM `sub_A4EE -> loc_B9A2`: the non-armoured `$14` compound hit keeps compound X/Y live, clears the X subpixel word before `loc_9D58 -> loc_9F18`, and only takes the solid-exit clear once the slope tile above the support column is gone. Regression tests cover frame 841's single low-word handoff (`$03DD5700`) and frame 843's preserved diagonal velocity (`y=$00BE0000`, `y_vel=$FFFE2900`). Full route gate `verify-tas --scenario .cache/recordings/issue-258-level00-slope-juggernaut-natural-20260608/scenario.json --frame-start 0 --frame-end 1020 --checkpoint-interval 1 --frontier-kind gameplay --bisect` now passes all 1020 frames by CRC. #258 still remains open for the wider live-play slope complaint across Kid/other forms and level routes.

Follow-up 2026-06-09: rechecked the natural Juggernaut route after the release-gate/death-respawn work. `cargo run -q -p kidcli -- verify-tas --scenario .cache/recordings/issue-258-level00-slope-juggernaut-natural-20260608/scenario.json --frame-start 0 --frame-end 1020 --checkpoint-interval 1 --frontier-kind gameplay --bisect` still passes all 1020 frames by CRC. This keeps the issue in `[?]` rather than `[x]`: the documented natural Kid, Red Stealth, Iron Knight, and Juggernaut slope windows are oracle-clean, but the wider human playtest report still needs a fresh live-play confirmation or a new failing form/level route before closure.

Follow-up 2026-06-09: the queued cache recording `issue-258-level00-slope-iron-knight-natural` is stale on current main (`verify-recording` first native replay drift at frame 0), so it should not be mined directly for a reduced case. Re-recording the same `walk-right-iron-knight` route as `issue-258-level00-slope-iron-knight-natural-refresh-20260609` gives a current bundle with `trace_match=true` and `oracle_state_match=true`; only strict checkpoint pixels still differ. The refreshed 0..120 case `walk_right_iron_knight_00_case_00000_00120` passes with matching `world.game_mode`, camera, player position, entities, and events, and `case reduce` reports it is already passing. Continue #258 from the later live-play slope routes, not this clean prefix.

Follow-up 2026-06-09: the queued cache recording `issue-258-level00-slope-juggernaut-natural-20260608` is also stale on current main (`verify-recording` first native replay drift at frame 0). Re-recording the same `walk-right-juggernaut` route as `issue-258-level00-slope-juggernaut-natural-refresh-20260609` gives a native-replayable bundle (`trace_match=true`) but full oracle verification now blocks later at `walk_quarter@300` on enemy count (`native 0`, `oracle 1`). The refreshed 0..120 case `walk_right_juggernaut_00_case_00000_00120` passes with matching `world.game_mode`, camera, player position, entities, and events, and `case reduce` reports it is already passing. Continue #258 from a later failing refreshed-route window or a new live-play slope repro, not the stale selected bundle or this clean prefix.

Follow-up 2026-06-09: narrowed the refreshed Juggernaut route's frame-300 checkpoint blocker. `tas-diff --scenario .cache/recordings/issue-258-level00-slope-juggernaut-natural-refresh-20260609/scenario.json --frame-start 280 --frame-end 320 --include-advisory --include-physics-trace --frontier-kind gameplay --only-divergent` reports 40/40 gameplay matches, so the `walk_quarter@300` failure is a checkpoint enemy-count/export issue rather than slope-stuck gameplay evidence. Derived `.cache/cases/issue-258-juggernaut-natural-refresh-0240-0360`; `case verify --case issue-258-juggernaut-natural-refresh-0240-0360` passes and `case reduce` refuses it as already passing. Do not mine the 240..360 refreshed-route window for a fix; continue from a later authoritative refreshed-route mismatch after frame 360 or from a fresh live-play slope repro.

Follow-up 2026-06-09: the later refreshed Juggernaut route window is also gameplay-clean. A broad `tas-diff --frame-start 360 --frame-end 620` only reported a raw `Character_Movement` export mismatch at frame 432, but the derived `.cache/cases/issue-258-juggernaut-natural-refresh-0420-0440` semantic case shows both native and oracle as `movement_0000` at frame 432 and `case verify` passes. Full route gate `verify-tas --scenario .cache/recordings/issue-258-level00-slope-juggernaut-natural-refresh-20260609/scenario.json --frame-start 360 --frame-end 960 --checkpoint-interval 1 --frontier-kind gameplay --bisect` passes all 960 frames by CRC. Do not reopen #258 from this refreshed Juggernaut route unless a new failing window beyond frame 960 or a live-play repro is captured.

Follow-up 2026-06-09: the direct seeded Juggernaut slope probe is blocked like
the Iron Knight seeded probe. The selected `issue-258-slope-juggernaut-probe`
bundle is stale, and the refreshed
`issue-258-slope-juggernaut-probe-refresh-20260609` bundle is native-replay
matched but not oracle-authoritative: `slope_probe_start@0` has native Kid
`$02EF0000,$00E70000` versus oracle `$02F80000,$00E70000`, with camera drift as
well. The derived 0..120 case blocks because the settled Level 00 oracle
bootstrap starts with different `player fields=x_pos, camera fields=x_pos`; the
oracle checkpoints still expose the requested patched `x=760` while native has
already stepped collision/settle to `x=751`. Do not patch #258 slope gameplay
from this direct seeded probe; use natural route windows or fix the
settled-recording direct-probe bootstrap first.

Case-queue follow-up 2026-06-09: the autonomy-selected original
`issue-258-slope-juggernaut-probe` bundle remains stale on current main. Fresh
`verify-recording --slug issue-258-slope-juggernaut-probe` fails immediately
with native trace replay drift at frame 0. Deriving the requested 0..120 case to
`.cache/cases/issue-258-slope-juggernaut-probe` produces
`level_00_slope_right_probe_juggernaut_case_00000_00120`, but `case reduce`
blocks it with `source recording replay drifted at frame 0`; `summary.md`
classifies the likely subsystem as `recording verification`. Treat this queued
case as explicitly rejected until the recording is re-recorded or the direct
settle bootstrap is fixed.

Follow-up 2026-06-09: the direct seeded `issue-258-slope-iron-knight-probe` bundle is stale and still unsuitable as gameplay evidence after refresh. The selected cache bundle was recorded from dirty commit `a7421f38` and now fails replay at frame 0. Re-recording the same `level00-slope-right-probe-iron-knight` scenario as `issue-258-slope-iron-knight-probe-refresh-20260609` is replay-matched, but oracle verification diverges immediately at `slope_probe_start@0` (`character_movement` native `Some(6)` vs oracle `Some(3)`), and the 0..120 case `level_00_slope_right_probe_iron_knight_case_00000_00120` is blocked because settled Level 00 oracle bootstrap starts with different `x_pos|movement_state`. Do not patch #258 from this direct seed; use natural route windows or fix settled-recording bootstrap first.

Note 2026-06-08 for #259: rechecked stale `.cache/recordings/codex-rocktank-level2-near/scenario.json` after the latest playtest. Native captures still show a RockTank path, but `tas-diff --frontier-kind enemy --include-advisory --frame-start 0 --frame-end 140` diverges from frame 0 because native is already in Level 01 with RockTank objects while oracle remains in Level 00/title-era Dragon state and a different camera. This bundle is not oracle-authoritative; keep it as scratch visual evidence only. The remaining gate is still a fresh current natural Tank/RockTank visual/death route or a fixed settled direct-probe bootstrap before changing Tank Y, turret baseline, or death visibility.

Recheck 2026-06-09: refreshed the direct camera-seeded visual probe `.cache/debug/rocktank-near-camera-scenario.json` into `.cache/debug/issue259-level2-near-current-refresh/`. Native full frames at 0/80/130 still show the tank low behind the grass foreground, while the enemies-only layer shows a coherent above-ground RockTank body/turret sprite. `verify-tas --scenario .cache/debug/rocktank-near-camera-scenario.json --frame-start 0 --frame-end 140 --checkpoint-interval 1 --frontier-kind enemy --bisect` still diverges at frame 0 (`current_level_id` native 1 vs oracle 0), so this remains native visual evidence only, not an oracle-authoritative placement gate. Do not change Tank Y or renderer baseline from this direct probe; next useful gate is a natural/current Level 02 shooting-tank or death route that is enemy-frontier clean.
Correction 2026-06-09 for #260: the 2026-06-08 ceiling-attach note was too broad. `GfxObjects_Collision loc_1A6C` detects the Drips ceiling through object-meta `$16` / `unk_314D0` (`-$1F` top reach), but writes back attached `y_pos` through the live `$22(a2)` sprite collision top after `Enemy16_Drip_Init` has selected `LnkTo_unk_C7B0E-Data_Index`. The cache-local route `.cache/debug/drips-route-candidates/orig_l480_530_b528.json` confirms oracle has both attached parents at `(456,530)` and `(584,530)` under the row-31 ceiling bottom 512px, and native now matches those parent positions by splitting detection height (`0x1F`) from snap offset (`0x12`). The same route still diverges at frame 537 on Kid/camera/wall-cling state, so this narrows the Drips lifecycle but does not yet reproduce or close the slime/metal-block mutation.

Follow-up 2026-06-09 for #260: fixed the `Drips.asm loc_33488 -> loc_3350E -> loc_3340E` RNG ordering bug on the negative `off_33520` sentinel path. Native was consuming the `loc_335D8` crawler random byte before reseeding the parent wait, but the ASM only queues the new child object at `loc_33488`; the parent immediately returns to `loc_3340E` and consumes the next RNG byte before that child object executes its own `loc_335D8` random branch on a later object pass. Regression `drips_sentinel_splash_reseeds_parent_before_new_crawler_rng_loc_33488` covers the old `$24` wait vs the ASM-backed `$30` wait. `tas-diff --scenario .cache/recordings/issue-260-drips-early-jump-close-probe/scenario.json --frame-start 360 --frame-end 410 --include-advisory --frontier-kind enemy --show-object-roster-on-match --include-physics-trace` now reports 49 match / 0 diverge, and `verify-tas --scenario .cache/recordings/issue-260-drips-early-jump-close-probe/scenario.json --frame-start 360 --frame-end 410 --checkpoint-interval 1 --frontier-kind enemy --bisect` passes. Frame 408 still has RockTank side contact, but it is now mirrored by oracle rather than caused by premature native Drips crawler/burst lifecycle. Metal-block contact remains unknown; next work should extend the route from the now-clean early-jump window or continue the pre-300 vertical setup search.

Route-search follow-up 2026-06-09 for #260: from the clean early-jump seed, the useful pre-contact axis is changing frames 380..409 rather than extending only after the frame-409 RockTank contact. Scratch variants `.cache/debug/issue260-route-probes/{left400,wait390}.json` avoid the frame-409 damage (`post_rocktank_avoidance@409` remains `lives/hp 3/2`) and `tas-diff --scenario <scratch> --frame-start 400 --frame-end 530 --only-divergent --include-advisory --frontier-kind enemy` reports 130 match / 0 diverge for both after setting `frame_count` to the extended script length. Tooling gotcha: extending a `DynamicScenario` script without also raising `frame_count` silently truncates oracle comparisons to the old frame 409 tail. The best proximity is still `left400` at target `(584,543)` with `dx_px +0 dy_px -22 manhattan_px 22 @470`; `left500`/`jump520` stay InGame to frame 700 but end with `lives/hp 3/0` and no better target approach, while other recoveries fall into `LevelTransition` around frames 605..611. Treat the 380..409 avoidance as a clean prefix, but do not promote it as a metal-block repro; next search must improve the row-32/33 vertical setup before frame 530 or branch back to the pre-300 route that reached `(456,543)`.

Route-search rejection 2026-06-09 for #260: extending the two clean scratch tails toward the reported contact did not produce a valid metal-block/slime repro. `left400` stays close to target `(584,543)` (`dx_px +0 dy_px -22 @470`) but `probe-record-scenario-file --frame-start 360 --frame-end 701` shows it first leaves `InGame` at frame 606 with only `lives/hp 3/1` at the closest approach, and enemy-frontier `tas-diff --frame-start 400 --frame-end 701` first diverges authoritatively at frame 625 after native has entered death/respawn while oracle remains live. `wait390` is similar: closest target `(584,543)` is `dx_px +1 dy_px -18 @444`, closest checkpoint still has `lives/hp 3/1`, and the route first leaves `InGame` at frame 587. Do not extend `left400`/`wait390` as if they are contact repros; the next useful search must change the pre-380 vertical setup or return to the pre-300 branch that reached `(456,543)`.

Follow-up 2026-06-09 for #263: fixed the remaining native split-state bug. The ROM uses the same `Red_Stealth_sword_swing` byte for `sub_B270` airborne attack-down and `sub_B000` landing-frame floor impact, but native had separate `red_stealth_sword_timer` and `red_stealth_airborne_slash_counter` fields; `sub_B000` only saw the ground timer, so a C-held airborne slash could render/damage but still miss the block below on landing. Native now treats either field as the shared ASM sword byte for the floor-impact gate. Regression `loc_b01e_red_stealth_airborne_slash_landing_breaks_floor_block` covers a held-C airborne slash landing on a smash block and bouncing via `sub_B000`.
Follow-up 2026-06-08: added named BLW2 left-pit direct probes for the missing forms: `issue249-blw2-left-pit-wall-kid-no-jump-audit` and `issue249-blw2-left-pit-wall-red-stealth-no-jump-audit`. Native probe summaries do not reproduce a no-input upward launch on this direct shaft seed: Kid falls/damages through the pit route, while Samurai/Red Stealth stays around the left-wall shelf; both keep `jump_buffer=0`, `ledge_snap_cooldown=0`, and no climb/fly special path at the sampled contact/hold/tail frames. Focused native regression `issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_kid_or_samurai` now guards those invariants. Full oracle gates still fail before the wall-hold window (`verify-tas` first CRC divergence frame 13 for Kid, 10 for Samurai) because the direct `start_position`/`camera_position` seed still has the #249 direct-probe bootstrap mismatch: oracle keeps the patched pose/falling state while native immediately applies generated terrain, wall-cling/floor snap, and extra RockTank object accounting. Next #270/#249 pass should either build a natural BLW2 shaft route or fix the settled direct-probe application; do not patch wall-cling from these non-authoritative direct-probe diffs.
Follow-up 2026-06-09: the refreshed Samurai/Red Stealth debug bundle
`issue-249-blw2-left-pit-wall-red-stealth-no-jump-current-debug-20260608` is
also non-authoritative. `verify-recording` is replay-matched but fails at
`left_pit_wall_contact_loc_a8de@30` on object counts (`native 7 / 7`, oracle
`5 / 8`). The requested 0..120 case derives to
`.cache/cases/issue-249-blw2-left-pit-wall-red-stealth-no-jump-current-debug-20260608`,
`case verify` reports FAIL, and `case reduce` blocks before gameplay because
settled Level 01 oracle bootstrap starts with different entity state
(`entities native=4 oracle=2`, first native `0A@00A48000,00E00000` versus
oracle `0A@00A40000,00D80000`). This confirms the Red Stealth/Samurai direct
probe repeats the settled direct-probe bootstrap gap; the next actionable
#270/#249 step is still a natural BLW2 shaft route or a direct-probe seeding fix,
not a wall-cling movement patch from this bundle.

Recheck 2026-06-09: current tree still proves the mapping/extraction half is fixed. `cargo test -q -p kid-core generated_later_level_block_list_keeps_helmet_prizes_without_section_overread`, `cargo test -q -p kid-core block_list_from_raw_generated_ram_dump_stops_prizes_before_ghost_section`, `cargo test -q -p kidcli generated_later_level_block_lists_include_helmet_prize_entries`, and `cargo test -q -p kidcli block_list_summary_reports_helmet_prize_entries` pass. `inspect-level --level 05` reports 25 prizes including Juggernaut at `(116,10)` and Micromax at `(4,29)`; `inspect-level --level 14` reports Cyclone at `(16,2)` and Eyeclops at `(22,4)`; `inspect-level --level 20` reports many helmet prizes including Berzerker, RedStealth, Skycutter, Juggernaut, Eyeclops, and Maniaxe. Leave row `[?]` until a current live build confirms the reported visible pickup/render path, but do not reopen extractor parsing without new contradictory evidence.

Latest playtest triage 2026-06-08: the player reconfirmed the current human-facing residual list after the latest loop pass. Keep these as active acceptance checks before closing their rows: #254 intro ninja still shows black-outline artefacts somewhere in the live intro/menu route; #255 level select still allowed jump/movement and played jump SFX in the reported build, now cross-check the follow-up latch-clearing fix in both native and emulated level-select paths; #256 third-level Rhino/Berzerker still draws in front of foreground/z-priority bits; #257 death still sits on a black screen and only Start recovers it, with music expected to pause/stop appropriately during the hold; #258 Kid and other forms still get stuck walking left/right/up slopes in live play despite the Iron Knight route gate; #259 tanks still appear stuck in the floor in current play; #260 Level-2 Drips/slime plus upward metal-block contact still breaks the metal block and emits a red pixel/projectile left; #261 fourth-level growing blocks still shoot spikes; #262 levels 5, 6, and 7 still have tileset/character-placement mismatch; #263 Samurai still cannot break blocks below with jump+sword; #264 music still slows when blocks are hit or diamonds collected; #265 EOL countdown sound still does not finish; #266 death-wall level still appears to miss left-side drill rendering; #267 EOL flag should activate from floor contact and needs the natural-route oracle gate, not just the synthetic hitbox regression.

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

### 141 — Intro and level-intro graphics still do not match the original game (EVIDENCE NEEDED)
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

### 144 — Level 2 shooter enemy fires too rapidly (EVIDENCE NEEDED)
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

### 145 — Level 2 shooter enemy rendered incorrectly (EVIDENCE NEEDED)
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

Follow-up 2026-06-03 selector replay: saved recording
`issue-146-level2-death` is stale on current native. Fresh full and native-only
`verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1861-frame recorded/replay traces. The requested `0..120` case derives
as `stomp_enemy_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Keep #146 closed from
the validated long-probe evidence above; re-record Level 2 death before using
this shorter bundle for new respawn/loading work.

Follow-up 2026-06-03 selector replay for the #146 short stomp probe:
`issue-146-level2-stomp-probe` is stale on current native. Fresh full and
native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 1861-frame recorded/replay traces. The requested `0..120` case derives
as `stomp_enemy_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Keep #146 closed from
the validated long-probe evidence above; re-record the short stomp route before
using this saved bundle for new respawn/loading work.

Follow-up 2026-06-03 selector replay for the #146 long probe:
`issue-146-level2-stomp-long-probe` is also stale on current native. Fresh full
and native-only `verify-recording` both drift at native replay frame 0, with
`trace_match=false`, `oracle_state_match=true`, `oracle_frame_match=true`, and
matching 9061-frame recorded/replay traces. The requested `0..120` case derives
as `stomp_enemy_01_case_00000_00120`, but `case verify` reports
`likely_subsystems=["recording verification"]`, and `case reduce` refuses
because the source recording replay drifted at frame 0. Keep #146 closed from
the validated long-probe behaviour above; re-record the long Level 2 death route
before using this saved bundle for new respawn/loading work.

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
