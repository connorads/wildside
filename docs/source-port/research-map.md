# Research Map

What the source port still needs to learn, where to source that knowledge, and what the Rust target should look like.

## Development philosophy

Progressive fidelity: get it playable, then get it accurate. Every system starts approximate and is refined with oracle comparison. Native-playability evidence is the primary programme-health signal. The oracle (`jgenesis`) is the parity oracle and safety net.

## Current frontier

The live TAS frontier is closed:

- `kc_any_percent_2024` is authoritative-clean through frame `4916`
- `any-percent-published` is player/camera CRC-clean through frame `37759`
- TAS-frontier history is tracked in the backlog and the `verify-tas-frontier` gate (the per-frame ledger was retired); `mise run verify-tas-full` re-checks the published route

That is not whole-game completion or whole-state/pixel equality. The 2026-07-13
full replay compares all 37,759 requested player/camera CRC frames and all 37,759
typed exported-state and enemy-roster frames with zero differences. The CRC is
the named `player_camera_progression_v1` projection: game mode, level, camera and
Kid positions, helmet, score, diamonds, Diamond Power, and current player. It
explicitly excludes pixels, enemy/render/audio state, object counts, hit points,
lives, Murder Wall X, velocity, and `Two_player_flag`; the typed comparator checks
those exported gameplay fields that both engines model, including
`Two_player_flag` after #404.

`verify-tas-full` fails on any projection mismatch, incomplete CRC, RNG, or RGB24
stream, projection-contract change, or typed-state/enemy/pixel baseline change. Typed RNG
comparison starts after `Init_RNG` at frame 16 and checks the offset plus all 55
ring bytes on every one of the remaining 37,743 frames. Every compared frame
matches exactly. This is complete per-frame RNG and per-frame exported-state
evidence for this route, not whole-state, audio, every-input, or whole-game proof.
The `source_port_visible_rgb24_v1` pass renders one bounded-memory native frame
for every oracle PNG: 21,336/37,759 frames match exactly and 16,423 differ, with
134,964,248 changed pixel instances. The first mismatch is frame 39 (2,643
pixels); frame 16274 is the worst mismatch at 59,062 pixels.
#406's engine-level corrections model manual `loc_119D8` and timeout `loc_C0A`
demo returns through `Pal_FadeOut -> loc_6E2`, the timeout-restart
`Load_IntroSequence6` clocks, and the rotating title transformation art,
palette, and object-worker variants selected by `sub_1C278`. The
`sub_1B43A -> loc_1B45A` tail also preserves its two blackout VBlanks,
intermediate title-plane rebuild, palette restore, and later title-art upload.
The timeout-restart IntroSequence3 Start return also preserves its resident VDP
through `sub_1C246 -> loc_1C264 -> loc_6E2`, `Pal_FadeOut`, the cleared-VRAM
blackout, and the fresh `Load_TitleScreen` worker passes. The natural
IntroSequence3 timeout likewise preserves the old VDP through `loc_1B96A ->
loc_6E2`, then fades the rebuilt IntroSequence4 palette through `sub_1C204`.
Demo `Load_InGame` also exposes `sub_73D0`'s complete palette fade through the
persistent DemoPlay wrapper. The cold IntroSequence1 Start return additionally
stages `Load_TitleScreen`'s base upload and fresh title-worker passes after the
blocking `loc_6E2` blackout. Later IntroSequence1, IntroSequence2,
IntroSequence4, and IntroSequence5 returns reuse the same resident-fade,
blackout, and fresh-loader model at their verified lineage clocks. Together
these corrections make 8,212 additional
route frames exact, suppress
the pause renderer while synchronous `Pal_FadeOut` owns the display, and
remove 467,045,995 changed pixel instances.
#406 owns this measured visual frontier; the four settled-warp source-port
visual-gate checkpoints remain exact.

The latest confidence correction promotes the full published-route state gate
from every 25th frame to every frame. The checked-in baseline enforces zero
typed differences, so fixes must preserve real per-frame equality rather than a
sampled-green claim. Player/camera CRC and complete RNG remain exact per frame.

### Frontier history

The chronological notes below explain how the former sampled and interval-1
debts were removed. Their intermediate counts are superseded by the current
zero-difference result above.

The latest correction removes a synthetic seven-frame Demo 1 GfxObject dip.
Raw WRAM keeps `Number_GfxObjects=7` and the same three walking-Dragon parent
and Gfx slots linked throughout frames 7478..7495. Native now keeps the
controller trace at 8/7 before and during the pause instead of inferring 8/4
for frames 7486..7492. The first remaining row moves to frame 10438.

The latest behaviour correction follows the timeout-restart IntroSequence3
return through `Load_TitleScreen -> Obj_TitleTextKid loc_1B5E4 -> sub_1B652`.
Raw WRAM shows that lineage rejoining `loc_E12` one object pass early, advancing
the initial title GfxObjects and both popup allocations together. Published
frames 6230..6379 are now exact, removing six fields and moving the first
remaining row to frame 7486.

The latest behaviour correction shifts the complete later IntroSequence5 worker lifecycle
with its one-pass-early `loc_1BA28 -> loc_1BCD0/loc_1C7A0` allocation. Native
had shifted the visible pose and later particle wake but not the initial Object
rows or IntroText5's `loc_1C7B6` deletion. Exact frames 27600..28066 now match
all 467 per-frame state, CRC, and RNG checkpoints. The three fixed rows were
outside the 25-frame sampled gate, directly motivating the interval-1 gate.

The previous correction removes a synthetic `$00060000` Kid y-velocity sample
from the later IntroSequence6 Demo 2 hand-off. ASM `Obj_TitleMenu loc_1BB02 ->
Load_DemoPlay` changes mode, selects the recording, allocates the controller,
and enters `Load_InGame`; it does not write Kid's velocity. Exact frames
29380..29450 are 70/70 clean. The full sampled baseline now has zero classified
differences.

The earlier correction keeps `Character_Movement` inert for IntroSequence3/4/5
presentation rosters. Their `Load_IntroSequence*` paths allocate Objects and
GfxObjects but never enter Kid's `loc_75D4` movement routine. Exact frames
27640..27870 improve 203/230 -> 229/230; the remaining frame-27857 Object-count
edge is separate. The full sampled baseline remains authoritative-clean and
classified limitations fall 3 -> 1.

The previous correction removes a synthetic Demo 1 pause trace from later attract
lineages. ASM `loc_74C8` sets `Game_Paused` only after `loc_7420` sets
`Allow_Pausing`; the four later published-route samples keep both latches clear
and continue their gameplay Object passes. Exact 70-frame windows around frames
13725, 22375, 25100, and 33300 are 280/280 clean. The full sampled baseline
remains authoritative-clean and classified limitations fall 7 -> 3.

The previous correction audits the published route's Dragon walking breath counts
against raw linked lists and `Dragon.asm`. `loc_3CF90` allocates a linked child;
accepted body contact sets parent `$5B` at `loc_3CFDA`, which the child observes
at `loc_3D082` before `loc_3D09E` deletes it. Native children now retain parent
identity and finish from that signal, while trace counts expose allocation and
deletion in Object-pass order. Later roster children have already run at trace
time; the first entry is the next pass and its zero-lifetime child remains
linked. The full sampled baseline keeps zero authoritative differences and 7
limitations. Exact frames 7800..8050 are 250/250 clean.

The latest correction maps the later IntroSequence2 Start return through raw
linked-list WRAM. `loc_1B9D0 -> loc_1BA02 -> sub_1C246 -> loc_1C264` carries
two Objects and no GfxObject through 32 title traces, then follows the generic
`Load_TitleScreen loc_1B258` rebuild schedule. After the title pose returns,
`Obj_TitleTextKid loc_1B5E4 -> sub_1B652` exposes 4/3 for ten frames before the
second popup makes 4/4. Published frames 36750..37560 match all 810 state
checkpoints. The full sampled baseline falls 3 -> 0 authoritative differences
with 15 oracle limitations; full-route CRC and complete RNG remain exact.

The previous correction maps the early-restore IntroSequence1 Start return
through raw linked-list WRAM. `sub_1B8F4 -> sub_1C246 -> loc_1C264` carries
the final controller/text pair at 2/1 through 32 title traces. Its two-frame
early restore advances fresh allocations but not `loc_6E2`'s carried-list
deletion. Published frames 34180..34930 match all 750 state checkpoints. The
full sampled baseline falls 6 -> 3 authoritative differences with 15 oracle
limitations, first at `frame_36825/number_gfx_objects`.

The previous correction maps the later IntroSequence4 Start return through raw
Object/GfxObject WRAM. `sub_1B988 -> sub_1C246 -> loc_1C264` reaches
`Load_TitleScreen` with two carried Objects and no GfxObject. `loc_6E2` clears
both lists on frame 32039; `loc_1B258` exposes the first three fresh Objects on
frame 32062, one pass earlier than the generic title-return schedule. The two
`Obj_TitleTextKid loc_1B5E4 -> sub_1B652` popups then join on frames 32156 and
32166. Published frames 31980..32720 match all 740 state checkpoints. The full
sampled baseline falls 7 -> 6 authoritative differences with 15 oracle
limitations, first at `frame_34250/number_objects`; full-route CRC and complete
RNG remain exact.

The previous correction follows the later IntroSequence6 menu cursor through raw
linked-list WRAM. The published frame-28762 Start edge takes `Obj_TitleMenu
loc_1BAEE -> loc_1BB52`; cursor GfxObject `$FFFF2578`, owned by `$FFFF0DA0`,
joins two object passes later and survives the terminal `loc_1BB02 -> loc_6E2`
demo hand-off sample. Native now separates that allocation from the 15-frame
visual menu activation delay. Published frames 28750..29404 match all 655 state
checkpoints. The full sampled baseline falls 35 -> 7 authoritative differences
with 15 oracle limitations, first at `frame_32025/number_gfx_objects`;
full-route CRC and complete RNG remain exact.

The previous correction maps the complete later IntroSequence6 revisit roster
from raw oracle WRAM. After carrying 4/5 through local elapsed 21, this lineage
delays the post-clear `loc_1BAB0`/`loc_1B37C`/`sub_1B43A` allocation and every
later `loc_1B5CE`, `loc_1B6AC`, `loc_1BB10`, `sub_1B652`, and `loc_1C7A0` edge
by three frames relative to the earlier carried-wrap shell. Published frames
28060..28415 match all 355 state checkpoints. The full sampled baseline falls
52 -> 35 authoritative differences with 15 oracle limitations, first at
`frame_28775/number_gfx_objects`; full-route CRC and complete RNG remain exact.

The previous correction preserves the manual Demo 1 title-return snapshot from
the terminal top-level trace. `loc_119D8` writes `Game_Mode=$2C` and jumps to
`loc_6E2` without deleting its linked Object/Gfx pair, so the oracle carries
9/8 for 32 title frames before the hidden rebuild clears both lists. Native no
longer reconstructs 8/7 from the visible Dragon roster. Published frames
22545..22590 now match every compared state checkpoint. The full sampled
baseline falls 56 -> 52 authoritative differences with 15 oracle limitations,
first at `frame_28075/number_objects`; full-route CRC and complete RNG remain
exact.

The previous correction separates the carried-wrap IntroSequence6 Demo 1
controller count phase from cold Demo 1. Raw Object lists and the native enemy
roster show that `Mode_Level loc_80C -> Manage_EnemyLoading loc_368A8/loc_3691E`
already links both later Dragons on the correct passes. The hidden `loc_119D8`
controller-side sample alone remains one Object pass behind on this lineage.
Published frames 22274..22296 now reproduce both Object-then-Gfx transitions.
The full sampled baseline falls 57 -> 56 authoritative differences with 15
oracle limitations, first at the frame-22552 DemoPlay-to-title transition;
full-route CRC and complete RNG remain exact.

The previous correction models the carried-wrap IntroSequence6 linked lists from
raw oracle WRAM. `loc_1BE74` deletes the two short IntroSequence5 particles on
frame 20890, leaving 4/5 until `Load_IntroSequence6` clears both lists. The new
shell then follows `sub_1B43A`, `loc_1B5CE`, `loc_1B6AC`,
`GfxObjects_Collision_BottomBoundary4`, `loc_1BB10`, `sub_1B652`, and the
two-frame `loc_1C7A0` Sega-text Object through its exact Object/GfxObject
boundaries. Frames 20885..21240 match all 355 per-frame state, CRC, and complete
RNG checkpoints. The full sampled baseline falls 63 -> 57 authoritative
differences with 15 oracle limitations, first at `frame_22275/number_objects`;
all 37,759 CRC frames and 37,743 complete RNG states remain exact.

The previous correction extends the exact state frontier through mature Demo 1.
`Mode_Level loc_80C` runs `Manage_EnemyLoading` after the complete
`Execute_Objects` pass, so Kid's current camera drives the inclusive
`loc_368A8` load window. `loc_3691E` links a Dragon Object on that pass and
`Enemy0D_Dragon_Init` links its GfxObject on the following pass. The Island
`loc_11FB0 -> loc_12098` controller remains Object-only. Native now preserves
that ordering, derives mature counts from the live Dragon roster, and copies
walking-Dragon layout Y through `sub_36FF4` without the unsupported +1px
offset. Published frames 13747..14055 match all 309 per-frame state and CRC
checkpoints. The full sampled baseline falls 238 -> 72 authoritative
differences with 15 oracle limitations; first authoritative sampled debt moves
to title frame 16650. Full-route CRC and complete RNG remain exact.

The latest correction closes the expanded-state fast frontier. Raw oracle WRAM
linked lists show demo-1 startup frame 71 with two system GfxObjects, frame 72
with Kid's two slots, and frame 73 with the first Dragon slot. The owning path
is `Manage_EnemyLoading loc_3691E -> Enemy0D_Dragon_Init ->
Load_GfxObjectSlot`: each Dragon Object is linked one pass before its GfxObject.
The same ordering makes the third Dragon advance 8/6 -> 8/7 on startup frame
530, and `loc_119D8` preserves 8/7 through the following 32-frame title
snapshot. All 3,400 per-frame gameplay checkpoints now match, down from 839
differences before GfxObject comparison was corrected and 99 after the title
fix. The full sampled baseline falls 477 -> 450 authoritative differences; its
10 oracle limitations include four fields newly exposed after the dominant
count mismatch cleared. CRC and complete RNG state remain exact.

The next correction models the cross-mode IntroSequence1 roster carry. Raw
oracle WRAM retains the `sub_1B8F4 -> loc_1B920` controller and its linked
`sub_1C54A` GfxObject through published frame 5534 even though `Game_Mode`
already identifies IntroSequence2. `loc_6E2` clears the inherited 1/1 roster
on frame 5535; `Load_IntroSequence2 -> loc_1B9D0` then exposes its controller
at 1/0 and `loc_1C7A0` text worker at 2/0. The full sampled baseline falls
450 -> 442 authoritative differences, first at demo-play
`frame_7950/number_gfx_objects`; CRC and complete RNG remain exact.

The previous correction classifies walking-Dragon breath children from their real
linked lifecycle. `loc_3CF90` allocates Object_RAM, `loc_3D02A` immediately
calls `j_Load_GfxObjectSlot`, `loc_3D082` keeps both linked, and `loc_3D09E`
deletes the pair. Native previously counted these projectiles as Object-only.
The full sampled baseline falls 442 -> 439 authoritative differences and 10 ->
7 oracle limitations; the first authoritative difference moves to IntroSequence3
`frame_9100/number_gfx_objects`. Exact oracle probes also expose two remaining
one-frame Dragon scheduler edges at frames 7954 and 8006, retained as #290 debt
rather than masked. CRC and complete RNG remain exact.

The latest correction derives IntroSequence3's counts from the selected
`unk_1AF78` roster. `Load_IntroSequence3 -> loc_1AEBA` stops at the first `$FF`,
giving 8/6/6/7 direct GfxObjects; `sub_1B93E -> sub_1C54A`, `sub_1C572`, and
`loc_1C58C` add three linked GfxObjects on the next pass. The second timeout
restart therefore exposes 3/6 at published frame 9097 and 4/9 at 9098. Across
the IntroSequence4 hand-off, the Kid pose clears at elapsed 27 while the 3/9
roster remains counted through elapsed 30 and clears at 31. Frames 9080..9395
are exact at all 315 per-frame CRC, RNG, and state checkpoints. Full sampled
debt falls 439 -> 392 authoritative differences with 7 oracle limitations;
the first authoritative difference moves to `frame_9700/number_objects`.

The next correction models the finite `loc_1C7A0` text-Object lifetime from
the same script data used by the VDP replay. The worker is counted for its
allocation trace, initial hibernate, every `$FE` pause, and every per-glyph
header delay; `$FD` line breaks consume no frame and `$FF` deletes immediately.
IntroText4 therefore remains live for 298 traces. Oracle WRAM frame 9689 links
slot `$FFFF0E14` at `loc_1C7B2` ahead of `sub_1B988`; frame 9690 puts that slot
on the free list and drops `Number_Objects` from 2 to 1. The exact per-frame
frontier advances to IntroSequence5 frame 9949. Full sampled debt falls 392 ->
352 authoritative differences with 7 oracle limitations, first at
`frame_9950/number_objects`; CRC and complete RNG remain exact.

The next correction models IntroSequence5's complete early linked-list and
title-return schedule. `Load_IntroSequence5` allocates `loc_1BA28` and
`loc_1BCD0`; the following object pass adds IntroText5's `loc_1C7A0`, the
`loc_1BC36` tile worker, and five direct GfxObjects. IntroText5 deletes at
`loc_1C7B6` on frame 10261. `sub_1C246 -> loc_1C264` carries the remaining
3/5 lists into title through frame 10321, `loc_6E2` clears both lists on 10322,
and `Load_TitleScreen -> Obj_TitleMenu` rebuilds 3/0, 5/1, 5/2, then 4/2 on
frames 10344..10347. Frames 9949..10360 are exact at every compared state,
CRC, and RNG checkpoint. Full sampled debt falls 352 -> 238 authoritative
differences with 9 oracle limitations, first at `frame_13750/number_objects`;
CRC and complete RNG remain exact.

The latest correction separates IntroSequence2's title-return scheduler from
IntroSequence4. ASM `loc_1B9D0 -> loc_1BA02` performs its two `sub_1C1A2`
updates before `sub_1C246 -> loc_1C264 -> loc_6E2 -> Load_TitleScreen`.
Oracle keeps the complete ring fixed through published frame 36877 at local
title elapsed 56, then resumes the enclosing `Mode_Standard -> Execute_Objects
loc_E12` pass at elapsed 57. Regression
`later_intro_sequence_2_title_return_keeps_final_load_title_screen_stall`
failed on frame 36877 before the fix and pins every following draw through
frame 36929. Per-frame RNG debt falls 882 -> 0 and sampled RNG field debt falls
74 -> 0. All 37,743 post-`Init_RNG` frames now match the complete RNG state.

The latest correction models IntroSequence6's post-loader object schedule.
`Load_IntroSequence6` allocates `loc_1BAB0`, `loc_1B37C`, and `sub_1B43A`
before returning to `Mode_Standard`, so `Execute_Objects loc_E12` resumes one
outer draw per displayed frame from published frame 20927. The
`sub_1B43A -> loc_1B45A` tail crosses VBlank once after the palette blackout,
draws once more, then remains in `sub_1B532`/`Load_TitleArt` across the staged
title upload before ordinary passes resume. Regression
`intro_sequence_6_workers_preserve_loc_e12_rng_around_loc_1b45a_upload_stalls`
failed at frame 20927 before the fix and pins the complete ring through both
upload pauses. Per-frame RNG debt falls 16,832 -> 15,146 and sampled RNG field
debt falls 1,398 -> 1,256. The next frontier is frame 22607: native `$FFCB`,
oracle `$FFCF`, first ring difference index 3 `$00/$51`.

The latest correction models `loc_1BA5C`'s mode-write ordering rather than
treating `Game_Mode=$28` as immediate entry to `Load_IntroSequence6`. The ROM
first executes nine `loc_1BA90` hibernation resumes while subtracting `$20`
from its `$100` fade word through `$00`; every resume remains in the enclosing
`Mode_Standard -> Execute_Objects loc_E12` pass. Only the negative result takes
`loc_1BAAC -> loc_6E2` into the loader. Regression
`intro_sequence_6_mode_switch_keeps_nine_loc_1ba90_outer_passes` failed on
frame 20895 before the fix and pins nine complete-ring draws followed by the
frame-20904 loader freeze. Per-frame RNG debt falls 16,864 -> 16,832. The
sampled RNG field count rises 1,264 -> 1,398 because the corrected stream
changes how many ring fields differ at later 25-frame samples; the complete
per-frame comparator is stronger evidence and shows 32 fewer mismatching
frames. The next frontier is IntroSequence6 frame 20927: native `$FFF4`, oracle
`$FFF8`, first ring difference index 44 `$5C/$B7`.

The latest correction scopes `loc_1BC36`'s combined
`sub_1C034(d4=4) -> sub_1C0EE` upload stall to its typed IntroSequence5
lineage. The Start-skip visit retains its oracle-pinned local elapsed 385
presentation stall; the ordinary timeout-restart visit stalls at elapsed 386.
Regression
`ordinary_intro_sequence_5_panel_upload_stalls_after_frame_20758_loc_1bc36`
failed on the missing frame-20758 draw before the fix and pins the complete
ring through the frame-20759 freeze and resumed cadence. Per-frame RNG debt
falls 16,865 -> 16,864; sampled RNG debt remains 1,264 because the corrected
frame lies between sampled checkpoints. The next frontier is IntroSequence6
frame 20895: native `$FFD0`, oracle `$FFD4`, first ring difference index 8
`$27/$F0`.

The latest correction preserves the later Start-skip IntroSequence5 rebuild's
carried hidden-delay frame through `sub_1C246 -> loc_1C264 -> Load_TitleScreen`.
The durable title-local lineage selects local elapsed 56 while the plain
timeout-restart path retains its oracle-pinned elapsed 55 wake. Regression
`start_skip_lineage_title_rebuild_wakes_loc_e12_after_frame_16705` failed at
`$FFCE/$FFCA` before the fix and pins the complete ring frozen on frame 16705,
then live through `Obj_TitleMenu`'s outer `loc_E12` passes from frame 16706.
Per-frame RNG debt falls 21,052 -> 16,865; sampled RNG diffs fall 1,740 ->
1,264. The next frontier at frame 20758 is native `$FFD6` versus oracle
`$FFDA`, with first ring difference index 14 `$50/$79`; both streams rejoin on
frame 20759.

The latest correction models `loc_1BC36`'s unique combined
`sub_1C034(d4=4) -> sub_1C0EE` decompression and VRAM upload as one object pass
that spans two displayed frames. Regression
`intro_sequence_5_panel_upload_stretches_loc_e12_pass_across_frame_16556`
failed on the native-only draw before the fix and pins the complete ring after
the stretched pass. Per-frame RNG debt falls 21,202 -> 21,052; sampled RNG
diffs fall 1,755 -> 1,740. The next frontier at frame 16705 is native `$FFCE`
versus oracle `$FFCA`, with first ring difference index 2 `$16/$D0`.

The latest correction uses the existing Start-skip title-return lineage to wake
IntroSequence5's outer `Mode_Standard -> Execute_Objects loc_E12` pass on the
same frame that oracle exposes `loc_1BCD0`. Regression
`start_skip_lineage_intro_sequence_5_wakes_outer_rng_on_loc_1bcd0_frame` pins
the complete ring from frame 16228. Per-frame RNG debt falls 21,381 -> 21,202;
sampled RNG diffs fall 1,770 -> 1,755. The next frontier at frame 16556 is a
native-only draw: `$FFD7` versus `$FFD3`, with first ring difference index 11
`$9D/$DA`.

The latest correction models demo 2's automatic `ReadJoypad loc_C0A ->
loc_6E2` timeout as an explicit local scheduler. The terminal Mode_Level frame
retains the previous trace-facing player snapshot but still performs
`Execute_Objects loc_E12`'s leading RNG draw. `Load_SegaScreen` then hibernates
inside `loc_12E64/loc_12E6C`, reaching outer object passes only on local
post-step frames 1 and 37 before timeout-restart IntroSequence1 wakes.
Regression
`demo_2_auto_timeout_preserves_terminal_and_mode_0_rng_passes_loc_c0a_12e64`
pins the frozen player/camera snapshot and complete RNG ring through frame 5200.
Per-frame RNG debt falls 32,791 -> 32,213; sampled RNG diffs fall 2,699 ->
2,653. The next frontier at frame 5546 is native `$FFE6` versus oracle `$FFEA`,
with first ring difference index 30 `$D6/$71`.

The timeout-restart IntroSequence2 follow-up traces `loc_1B9D0 -> sub_1C204`
as an RNG-idle hidden loader/fade. Its render worker appears before local
pre-step elapsed 41, where `Mode_Standard -> Execute_Objects loc_E12` resumes
one longword draw per frame through `loc_1BA02`'s mode-switch frame. Regression
`timeout_restart_intro_sequence_2_runs_live_object_passes_loc_1b9d0_1ba02`
pins the complete ring across all 408 live passes and the following loader
freeze. Per-frame RNG debt falls 32,213 -> 31,764; sampled RNG diffs fall 2,653
-> 2,607. The next frontier at frame 5995 is native `$FFD4` versus oracle
`$FFD8`, with first ring difference index 12 `$34/$37`.

The timeout-restart IntroSequence3 follow-up traces `sub_1B93E -> sub_1C204`
through the live `loc_1B96E` loop and `sub_1C246 -> loc_1C264` Start hand-off.
It uses an explicit title-return lineage: the outer `Execute_Objects loc_E12`
pass wakes at local pre-step elapsed 41, remains live on both deferred hand-off
frames, freezes through the hidden title rebuild on published frames
6210..6264, and resumes at 6265. Regression
`timeout_restart_intro_sequence_3_preserves_live_and_title_rng_phases_sub_1b93e_1c264`
pins the complete ring across all three phases. Per-frame RNG debt falls 31,764
-> 29,688; sampled RNG diffs fall 2,607 -> 2,442. The next frontier at frame
8070 is native `$FFD0` versus oracle `$FFCC`, with first ring difference index
4 `$69/$F0`.

The demo-1 timeout follow-up traces the published frame-8069 transition through
`Mode_Level -> Execute_Objects loc_E12 -> ReadJoypad loc_C0A -> loc_6E2`.
Because the ROM calls ReadJoypad after Execute_Objects, that transition frame
retains the terminal mode-level draw before `Load_SegaScreen loc_12E64` enters
its loader hibernation; native had replayed the transition correctly, then added
an extra outer draw on frame 8070.
`DemoAutoTimeoutMode0Entry` now makes fresh versus already-consumed entry
explicit. Regression
`demo_1_timeout_replay_does_not_duplicate_mode_0_rng_pass_loc_c0a_12e64` pins the
complete ring through the second mode-0 pass at frame 8106. Per-frame RNG debt
falls 29,688 -> 28,367; sampled RNG diffs fall 2,442 -> 2,342. The next frontier
at frame 9392 is native `$FFEF` versus oracle `$FFF3`, with first ring difference
index 39 `$B1/$96`.

The visual side has a separate confidence ceiling. Since 2026-07-10,
`source_port_frame_oracle:level_00_early` renders the actual source-port scene at
four settled-warp checkpoints and fails unless state is exact, no checkpoint is
masked, all diff images exist, MAE is <=1.750, and changed pixels are <=2,293.
The fresh gate measured worst MAE `1.677` / `2,293` pixels at frame 30, so this
is a reviewable regression ratchet, not pixel-perfect or authoritative cold-boot
proof. #375 tracks reducing it towards zero. The older `frame_oracle:*` steps
render the original ROM through wildside-md and compare that VDP output with
jgenesis; they exercise emulator capture, not source-port pixels. Their RGB
comparator now rejects empty evidence and has focused mutation coverage, but a
passing step must not be quoted as native renderer fidelity.

As of the 2026-05-18 completion audit, the 1P source-port phase has no open phase blockers: `project-status` reports `1p-playable` and `1p-accurate` green, the published TAS frontier is replay-matched through frame 37759, the canonical backlog has no open phase rows, and the human-validation checklist is empty. Future work should come from fresh reports, post-phase fidelity debt, or broader verification lanes, not from reopening stale TAS/frontier prose. Loop sessions follow `docs/source-port/loop.md`: `backlog.md` is the open queue, the NOW block is the handoff, `mise run project-status` selects the next item in priority order (`[?]`/`[b]` rows live in `backlog-awaiting-human.md`), and the proactive-verification fallback (mutation testing, enemy audit, ASM coverage) covers the no-actionable-item case. `wildside-cli project-status` remains the top-level status surface, with 1P-playable and visual risks ahead of TAS completion prose.

2026-07-04 refresh: `env TMPDIR=/tmp mise run release-gate` passes the full
`1p-accurate` lane. `.cache/release-gate/1p-accurate/latest.json` points at
timestamp `1783122304`, with report
`.cache/release-gate/1p-accurate/1783122304/report.json`; the embedded state
sweep report is `.cache/sweep-runs/1783122357/report.json` (69/69 passed,
0 authoritative/advisory/accumulation/rng, 1037 oracle limitations,
`unverified=69`).

Historical post-TAS verification split as of 2026-04-22. Commands referencing the retired orchestrator are kept as a dated record; the orchestrator was removed in the 2026-06-10 autonomy demolition, tag `pre-autonomy-demolition`:

- `cargo run -p wildside-cli -- autonomous-loop --goal sweep-parity` completed the delegated lane again (`cycle=127` locally). The old `stalled_fingerprint` / `refresh_evidence` package remains superseded by fresh gate/sweep evidence; the regenerated source-port package is now cycle `600` with fingerprint `gate-1776819385:sweep-1776819420`.
- `cargo run --release -p wildside-cli -- sweep --levels 0-104 --tier state` passed all 69 BCD levels again with zero failed/error levels. The latest direct sweep report is `.cache/sweep-runs/1776819377/report.json` (`advisory=178`, `oracle_limitation=662`), and the release-gate-embedded sweep then refreshed `.cache/sweep-runs/1776819420/report.json` with the same zero-blocking classification. The CLI still reports mixed advisory/oracle-limitation levels as `NON-BLOCKING`, and the persisted state-sweep artefact remains the canonical source for field histograms, level IDs, checkpoint counts, and any `release_gate_focus` entries when a gate is red.
- `mise run release-gate` now passes the full `1p-accurate` lane. `.cache/release-gate/1p-accurate/latest.json` points at timestamp `1776819385`, and the matching report is `.cache/release-gate/1p-accurate/1776819385/report.json`; all gameplay-compare, frame-oracle, audio-oracle, and `state_sweep:0-104` steps passed. The previous `gameplay_compare:pause_resume_level1` / `level_settle@719` `game_mode` blocker is stale evidence, not the current release-gate state.
- The sweep-parity handoff must handle both outcomes. If the gate is red, `next-agent.md` should carry the live `release-gate-failure` detail plus the matching persisted `release_gate_focus.summary` from the newest state-sweep evidence. If the gate is green, it should instead cite the passing release-gate timestamp and the latest state-sweep report, so operators do not keep chasing an obsolete failure.
- `mise run release-gate` also refreshes `.cache/autonomy/source-port-frontier/next-task.json` and `.cache/autonomy/source-port-frontier/next-agent.md` after it writes `latest.json`, so run the delegated sweep-parity commands serially in order (`autonomous-loop`, then `sweep`, then `release-gate`). Do not fan those three commands out in parallel: they all touch the same Cargo target directory, and the handoff is only complete once the release-gate write lands after the refreshed sweep artefact.
- The handoff refresh should treat `.cache/sweep-runs/<timestamp>/report.json` as a first-class persisted state-sweep artefact, not just `sweep-history.jsonl`. If history append lags but the fresh sweep report already contains the newest state-sweep summary, the next-agent note must cite that newer report timestamp/path rather than reusing an older history entry.
- The sweep-parity progress fingerprint includes the latest release-gate timestamp and the latest persisted state-sweep timestamp. If the lane repeats after a green refresh, check whether those two artefact timestamps actually advanced before treating it as new evidence; stale timestamps intentionally still count as the same stalled lane.
- Automation still keys red-gate correlation by concrete release-gate focus metadata first (`target`, `step_name`, `checkpoint`, `field`, and exact native/oracle pair when present), then falls back to field-only matching for older sweep-history entries that predate the richer schema. That prevents a later `game_mode` failure in one gate target from borrowing the wrong summary from another target just because both surfaced the same field name.

## Canonical status language

Use this evidence ladder when describing progress:

- `stubbed`
- `boots`
- `replay-matched`
- `branch-matched`
- `mechanic-swept`
- `human-play-stable`

Current TAS milestones prove `replay-matched` on their covered routes. They do not by themselves prove `human-play-stable`.

## Done

### Boot and title
- Typed boot/title state machine (`BootState` enum)
- Mode flow, title timing, oracle-backed title readiness and intro-skip handling
- TAS-backed early boot slice parity across 3 imported recordings (first 250 frames, 0 authoritative diffs)
- Source: `kid.asm`, oracle checkpoints

### Level load
- Header parsing, asset pointers, `LevelBundle` loading from extracted assets
- Typed asset manifest with level headers, enemy layouts, platform layouts
- Source: `doc/formats.txt`, `level/header/00.bin`, generated asset manifest

### Player physics and camera
- Real input-driven physics from disassembly speed tables, jump formula, gravity, camera follow
- `physics.rs` module with pure functions for horizontal velocity, jump, gravity, camera
- Speed tables for 3 terrain types x 3 form variants
- Source: `kid.asm` speed tables (`unk_9C0A`), `sub_A432` (jump), `kid.asm:14060` (gravity), `sub_71E4` (camera)

### Gameplay lifecycle
- Pause/resume, death windup/hiding/respawn settle
- Damage invulnerability (300 frames)
- Source: oracle gameplay checkpoints

### Level 1 visuals
- Manifest-backed Level 1 visual loading and asset-backed native scene output
- ROM-backed HUD, player, background, terrain, visible platforms, and visible Level 1 families on the default render path
- Source: generated bundle contract, `wildside-render`, `wildside_cli::native_player`

### Object activation and tracing
- Enemy roster built from camera window, platform preload + activation window
- Native trace surface records stable enemy layout identity and motion/state fields
- Source: `doc/ram_layout.txt`, enemy layouts

## Traversal closure

### Default runtime terrain authority
- `NativeRuntime::new` reloads generated `Level_Layout` terrain on construction
- The default runtime path no longer depends on a separate explicit generated-terrain mode for Level 1 gameplay
- Source: `wildside-core`, generated Level 1 bundle, `doc/formats.txt`

### Traversal parity
- `compare-native-gameplay --scenario progress-two-transitions` passes on the default path
- Pause/resume and death/respawn stay green on the same default path
- The long traversal script takes the first opening Dragon hit on the oracle frame (`749`) under walk-only input as well as the existing run-held regression
- `diagnose-native-gameplay-drift --scenario progress-two-transitions --from-frame 719` reports no live drift after the skip choreography and Level 2 transition-arc fixes
- Source: `.cache/native-gameplay-oracle-runs/progress_two_transitions/`, gameplay comparison harness

### Enemy fidelity after traversal
- `Diamond` remains stationary and collectible
- `MiniHoppingSkull` uses a hopper loop in the native runtime
- Pacer archetype for 6 Tier 1 families (Goat, Armadillo, Tank, Crab, Scorpion, Driller)
- Family-specific HP initialisation from ASM init routines
- Cloud (horizontal flight + wall reversal), Sphere (gravity bounce), BigHoppingSkull (parameterised hopper)
- Fireball (gravity bounce with level-dependent gravity), EmoRock/Archer (Inert stubs)
- All 28 enemy families wired. Tier 3-4 mapped to closest archetypes (Pacer/Flyer/Hopper/Bouncer/Inert)
- Source: `code/enemy/*.asm`, oracle per-object traces

## Remaining work

### Visual fidelity, sprite extraction
- **#033** [x] Transform animation visual sprites. Original `Kid_Transform` art and timing are wired for all forms. Juggernaut composes the split head/base sprites into native player frames.
- Fire Demon death trail visual sprites. Live play renders the `Fire_Demon.asm` `stru_3A622` ROM frames for the death-trail signature (#025). The core still carries this with native projectile snapshots rather than the original child-object lifecycle.
- Floating item ROM sprites. Diamond, helmets, Ankh, Clock, Continue, and the `10000`/`10 diamonds` worker bursts use ROM sprite assets. Ten Diamonds workers now keep their ROM display flag through the outward and frozen phases instead of disappearing when `loc_10984` clears movement; the monotonic `$0320 - remaining` age also prevents burst replay every 256 frames. Remaining visual debt is the final per-worker `loc_10988` HUD flight, which needs camera-relative state at each `$20` transition, plus exact late 10000-point star motion.

### Visual fidelity, rendering accuracy (not yet investigated)
- Palette cycling per theme: some themes animate palette entries (water shimmer done). Lava, neon, and other themes may have CRAM cycling in the VBlank handler. Trace `kid.asm` VBlank for CRAM writes per theme ID. Extract cycle tables (which entries, rotation speed, colour sequence).
- Sprite priority/layering: the VDP sprite priority bit determines front/behind-BG rendering. Some sprites intentionally render behind background (hiding behind pillars). Search ASM for priority bit usage, compare frame captures on levels with sprite-background overlap.
- HUD positioning accuracy: compare HUD element pixel positions against oracle (lives icon, HP bar segments, timer digits, score digits, diamond count, diamond icon). Capture frame, overlay oracle, diff.
- Background scroll accuracy per theme: 11 themes each have background scroll configurations (parallax rates, layer ordering). Verify `background_scroll_x_px()` rates are correct per theme by comparing the background-only layer at several camera positions against oracle.
- Block destruction particle effects: verify particle spawn positions, velocities, colours, and lifetime match ASM. Break blocks in multiple themes, capture debris frames, compare to oracle.
- Enemy death animations: verify enemy death visual effects (explosion, fade, particle) match ASM per family. Stomp enemies, capture death frame, compare to oracle.
- Invulnerability flicker accuracy: verify the flicker pattern (which frames visible/invisible) matches ASM. Verify Diamond Power overlay colour. Take damage, capture flicker frames, compare to the oracle frame sequence.

### Gameplay fidelity (current 1P phase complete; reopen only with fresh evidence)
- **#037** [x] Floor-snap settling diffs. Done. The `0x24`/`0x50`/`0x51` 1px `camera_y` tail is closed generically: `loc_75D4` runs one leading `sub_71E4` tick against the fresh spawn pose before the first hibernate, mirrored by the settle sim and the loc_74E0 live entry (the level-gated "+2px carry" override is deleted; live-oracle rest is `grounded target - 3`). `0x29` and `0x52` stay locked down by their focused regression tests.
- **#062** [x] HeadyMetal death wobble visual. Covered by effects-only render regressions that use visible body/eye tiles and assert the full sprite footprints plus the two-frame wobble phase flip at each ASM timer boundary.
- **TAS recordings** [x] `kc_any_percent_2024` and `any-percent-published` are both authoritative-clean through their recorded endpoints (`4916` and `37759` frames). Reopen a TAS frontier only when a fresh `verify-tas --bisect` run proves a new authoritative mismatch.
- **Native playability** [x] recurring player-facing failures live in `backlog.md` with canonical evidence, and the 2026-05-18 audit closed the remaining human-validation and scripted Level 2 death/loading decisions. Reopen this area only with a fresh recording, backlog row, or failing gate.

### Game flow verification
- **#132** [x] 2-player alternating mode. Strong native scenarios cover game-over-before-swap, continue respawn for the same player, preservation of the other slot, and continue timeout returning to title.
- **#133** [x] Password/options scope. ASM audit found no stock-ROM password entry path; native options now mirror the ROM four-row options menu.

### Enemy behaviour gaps (discovery aid: `wildside asm-coverage`)
`wildside-cli asm-coverage --uncovered --subsystem enemy` is a **discovery aid only**: it
surfaces ASM labels that no Rust file cites yet. It does **not** prove behaviour is
implemented, reachable in the shipped game, or verified.

> **Citation ≠ fidelity ≠ reachability.** A `// ASM loc_XXXX` comment marks a label
> "covered" with zero verification, and a citation that lives only in `#[cfg(test)]` code is
> test-only debt, not shipped behaviour. Fidelity is proven by the oracle/TAS gates and
> checked-in regression tests, never by a coverage count. (The tool self-discloses this:
> `asm_coverage.rs` — "label citation tracks recovered/discussed ASM, not player-visible
> fidelity".)

The items below were closed by actual fixes + regression tests (not by the coverage
number): Archer arrow lifetime + re-aiming, Tornado Cyclone-flying interaction, Crab
proximity freeze, Dragon Flying wall-reversal, Sphere/Fireball slope deflection, and Dragon
Flying collision-recovery during Descent / death status-table route.

### Oracle sweep hardening
- Advisory field promotion: `number_objects`, `number_hitpoints`, `number_lives`, `murder_wall_x` are still masked in sweep. Unmask one at a time, run sweep, fix divergences. `number_objects` may require architectural changes if native object activation/deactivation timing differs from oracle.
- Framewise checksumming: `wildside-cli sweep --checksum` records the first per-level authoritative CRC mismatch in `.cache/sweep-runs/<timestamp>/report.json`, using the same native/oracle CRC contract as TAS verification. Remaining hardening: promote this from optional report evidence into release-gate policy once the acceptable oracle-limitation windows are classified.
- Divergence classification output: sweep history persists per-run `divergence_summary` totals, a cross-level `field_summary` histogram with per-field category counts, BCD `level_ids`, unique checkpoint labels, per-checkpoint diff counts, and `release_gate_focus`, plus per-level category/field summaries in `.cache/metrics/sweep-history.jsonl`; the sweep report mirrors the same `field_summary` and `release_gate_focus`, while the flatter `.cache/metrics/sweep-telemetry.jsonl` stream remains available for append-only metrics ingestion. `wildside-cli sweep` prints a focused field summary inline that correlates a failing release-gate field against the latest sweep classes (this correlation logic outlived the retired orchestrator). That summary includes the failing release-gate scenario/checkpoint, whether the matching histogram includes that scenario's own BCD level and checkpoint label, which checkpoints actually carried the field, whether the exact gate value pair ever appeared in sweep at all, and whether that exact pair matched the precise checkpoint label versus only some other checkpoint. That lets operators distinguish "same field name" from "same concrete divergence at the same checkpoint".
- Full-frame pixel comparison sweep: `wildside-cli sweep --tier full` uses the visual key-frame idle scenario (frames 0, 30, 60, 120, 300, plus idle tail) instead of the compact state-sweep checkpoints. The native full-frame bootstrap compensates for the ROM level-skip path's live title-card tail, so Level 0x01's first RockTank parent and turret child positions line up with oracle frame 0 (`0x00A40000/0x00A48000`, facing right). The first live smoke, `cargo run -q -p wildside-cli -- sweep --levels 1 --tier full --max-failures 1`, passes under the current MAE threshold after dropping the generated all-zero `PlatformScript_Nothing` sentinel from the active native roster and aligning the settled opening `number_objects` value at 4. `tas-diff --scenario <idle_level_full_frames_01.json> --frame-start 0 --frame-end 3 --frontier-kind enemy` is clean after normalising oracle-only roster metadata, native-only debug hints, RockTank child `None`/`0` HP, and the existing +/-1px/frame enemy velocity capture tolerance. A later frame-300 RockTank roster drift was authoritative, not visual-only: native ran the rightward grounded walker scan after storing the predicted X position, while ASM `loc_3C04E -> loc_3C0A6/loc_3C0B4` runs `sub_36972` before `move.l d2,x_pos(a3)`, so a tile-boundary edge turn consumes the frame without advancing. `tas-diff --frame-start 292 --frame-end 302 --frontier-kind enemy` is clean after moving that scan ahead of native movement. A renderer follow-up stopped drawing duplicate RockTank body/turret sprites when both parent and child snapshots are already present, reducing the committed Level 0x01 full-frame sweep to worst 6,695 mismatched pixels / MAE 3.9. A later renderer fix made `BackgroundScroll_ApplyForestWaterRipple` scanline-based instead of shifting whole 8px tiles, matching ASM `loc_30598`/`loc_3060A`; the same Level 0x01 full-frame sweep improved from worst 6,710 mismatched pixels / MAE 4.0 to 6,324 mismatched pixels / MAE 2.8. Disabling the forest foreground palette permutation as a hypothesis check was rejected because it worsened the sweep to roughly 41.7k mismatched pixels / MAE 44.1. The remaining full-frame Level 0x01 work is visual: roughly 6.0k-6.3k pixel mismatches still sit below the current MAE threshold, concentrated in foreground/background art-colour detail and ordinary HUD pixels, but block pixel parity needs a stricter release gate.

### Testing infrastructure
- Semantic case bundles: `wildside-cli case derive|branch|verify|reduce|summarise` provide the phase-1 discovery lane. Use them to turn replay slices or native recordings into reduced semantic failure bundles before another broad gameplay patch.
- Case queue (the autonomous queue runner was retired with the orchestrator; reduced case bundles in `.cache/cases` remain first-class evidence). As of 2026-05-18, the queue was blocked rather than patch-ready: `issue-143-knights-isle-probe` and `issue-146-level2-walk-right-probe` both reduce to settled-recording oracle-bootstrap limitations, not authoritative native gameplay failures.
- Native replay recording: `play-native --record <slug>` and `verify-recording --slug <slug>` already exist for input + trace capture. Bundles include ROM/build provenance, explicit replay commands, legacy `trace.json` schema backfill, copied-bundle verification via `--bundle <path>`, bundled `final-frame.png` + `audio.wav`, optional `notes.txt` via `--record-notes <path>`, and an automatic `transcript.txt` sidecar distilled from the native trace so copied bundles stay reviewable before replay. `play-native --playtest` layers the interactive validation lane on the same bundle format: segmented recordings under `.cache/playtest/<session>/segments/`, verdicts/bookmarks/sync marks in `session.json`, optional `--mic` narration, and `playtest-ingest` for the draft report (`docs/source-port/playtesting.md`).
- Mutation testing baseline: `cargo mutants` exists but no recorded baseline. Run `cargo mutants -p wildside-core -- --lib`, target >=80% kill rate on `physics.rs` and `terrain.rs`. Record in `.cache/metrics/mutation-score.txt`.
- Fuzz testing for parsers: set up `cargo fuzz` targets for the level block grid parser (untrusted binary format), enemy spawn logic (layout data with unexpected IDs), and audio bank loading (complex binary format). Run each for a few hours, fix crashes.
- Stateful property testing: implement a `proptest-state-machine` test for `PlayerLifecycle`. Model the lifecycle as states with input-driven transitions. Generate random input sequences, assert invariants hold.
- 1P playable gate: `release-gate --target 1p-playable` is the native-playability headline. It covers the focused native smoke scenarios for Level 1/2 progression, death/respawn, slopes, bounce surfaces, common block interactions, and the all-level native gameplay self-consistency sweep over the 69 currently runnable levels. The 2026-07-04 refresh via `env TMPDIR=/tmp mise run verify-1p-playable` passes report `1783122908`, including gameplay sweep totals of 69 passed, 0 failed, and 0 errors. The current `1p-accurate` gate remains supporting oracle evidence, not the sole "game works" signal.

### Known deviations
The old five parked `DEVIATION:` items were burned down by backlog #131: each was removed, reclassified, or explicitly deferred with evidence. Treat any new `DEVIATION:` marker as fresh fidelity debt that needs a backlog row or an explicit out-of-scope decision.

## Recently completed

- Scorpion tracking callback with $38000 speed escalation (HP≥5, within 170px), Phase 4C
- Dragon Walking dual breath interval ($12C initial, $1F4 reset), Phase 4C
- MiniHoppingSkull HP-scaled hop velocity (+$4000 X, -$10000 Y at HP>1), Phase 4C
- Workspace clippy clean for Rust 1.94 (all warnings resolved or allowed)
- Boss multi-head system (#060): 3 heads per boss with ASM position offsets + per-head HP
- BigHoppingSkull continuous bouncer rework (#056): ASM $FA0 gravity, wall/floor/ceiling/slope deflection
- Orca X-accel, slope re-lunge, wall-pin (#061): all 3 from ASM
- Lift block ASM coroutine timing (#067): 2px/frame×8 up, erase, 1px/frame×16 down
- Shooter mechanism (#065): resolved. Visible shooter cells register independently, fire on entry, and recycle after the ROM's 22-frame slot lifetime
- SoftPlatform bounce (#066): fixed. Native moves the platform object per ASM `SoftPlatform_Init`/`SoftPlatform_BounceLoop`
- Shifting block (#068): fixed. Native preserves the immediate collision grid swap and emits the ASM `loc_11364`/`sub_113C0` visual rise worker through the block-bounce trace path

## Previously deferred (now done)

- Platform collision: static + moving, riding logic, movement scripts
- Player forms: speed tables, form-specific abilities, per-form Diamond Power
- Audio and GEMS: GemsAudioEngine: Z80+YM2612+PSG from ROM
- Tier 3-4 enemies + bosses: all 28 families, 26 with real sprite art
- Y look-ahead camera offset: `$FFFFFB58`, +1/frame down+grounded, -8/frame decay, applied at >=30px
- Block interactions: done at engine level (prize blocks, evanescent, ghost, smash, terrain types)
- Multi-level progression: 105 levels loadable, full progression, ending sequence, continue, score system
- Menus: boot/title, pause/resume, game over, continue, score board
- Diamond Power two-tier cost: 20 form/tier combos mapped to 8 unique power types
- Demo playback (#026): extracted from kid.asm:33373-33436, 2 demos alternate on title idle
- Per-helmet transform timing + transform sprites (#033): correct frame counts and original art tables
- Perf baseline tracking (#031): criterion benchmarks, ~92ns/frame idle, ~3µs level load
- Per-enemy constant verification: done for all 28 families against ASM (Phase 4)
- Block constant verification: done for all 13 block types against ASM (Phase 3)
- Ghost block collision fix: type 6 (Solid) not 7 (Spikes) (#063)
- Drill block direction fix: horizontal ±4px/frame not downward (#064)
- Murder wall constants: ASM-verified (acceleration, max speeds, offset)

## Validation methodology

See `docs/source-port/methodology.md` for the full validation methodology: investigation playbook, testing taxonomy (property-based, metamorphic, mutation, fuzzing), divergence classification, framewise checksumming, fidelity metrics, and regression discipline. `CLAUDE.md` has the operational summary.

## Known oracle limitations

- 9 oracle warp-broken levels (game_mode=0x4E73): 0x09, 0x32, 0x33, 0x50, 0x57, 0x5A, 0x5D, 0x5E, 0x63. jgenesis emulator bug, native works fine (6 in BCD sweep)
- 4 levels with idle_tail advisory diffs (0x00, 0x45, 0x48, 0x61): oracle warp timing and enemy interaction accumulation
