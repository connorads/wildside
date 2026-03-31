---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 44
updated_at: 2026-03-31T05:05:12Z
---
# What Changed
- Re-read `loc_1ABF4`, `loc_1AC70`, `loc_1AD48`, and the surrounding mode-8 shell path before trying a gameplay fix.
- Ran a delayed-materialisation experiment locally, then backed it out after verification showed it was incomplete.
- Confirmed a new blocker: shifting worker visibility later is not sufficient because native still advances RNG during the hidden mode-8 window before any shell worker is visible.

# Current Model
- The level-entry shell bug is deeper than “workers start too early”.
- Oracle stays hidden at frames 500..503 with `rng_offset=65481`, then exposes slot 2 on frame 504.
- Native can be forced to stay hidden through frame 503, but even in that hidden window its RNG is already at `65488` by frame 503, seven bytes ahead of oracle before any visible `loc_1AC70` worker should exist.
- That means the next safe fix is to identify the hidden mode-8 RNG consumer / scheduler path first. Patching `starting_level_title_workers` timing alone will keep producing the wrong slot-2 seed and pose.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28816-28982` `loc_1ABF4`, `loc_1AC70`, `loc_1AD48`
- Oracle:
  - `.cache/oracle-probe/oracle/manifest.json`
  - Frame 500: `game_mode=8`, `rng_offset=65481`, `slot_2=null`
  - Frame 503: `game_mode=8`, `rng_offset=65481`, `slot_2=null`
  - Frame 504: `game_mode=8`, `rng_offset=65519`, slot 2 at `0x00940000,0xFFF00000`
- Native probes:
  - `cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 468 --frame-end 475`
    shows the shell can be kept fully hidden on those early mode-8 frames.
  - `cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 500 --frame-end 506`
    showed the delayed-materialisation experiment still produced the wrong shell on frame 504.
- Failed red step:
  - `cargo test -q -p kid-core boot_title_start_delays_loc_1abf4_worker_birth_until_mode_8_frame_504 -- --nocapture`
  - Even with the shell hidden through frame 503, native reported `rng_offset=65488` at frame 503 instead of oracle’s `65481`.

# Exact Next Moves
- Do one investigation only: find what advances native RNG by 7 bytes during hidden mode-8 frames 500..503.
- Compare oracle and native around frames 499..504 with the hidden shell intact; inspect whether the live gameplay/object loop is still running when only `loc_1AE2E` should be polling Start.
- Read the mode-8 hand-off around `Load_TitleCard` / object execution again and correlate it with native stepping order before reintroducing any worker-lifecycle patch.
- Do not reapply the delayed-materialisation patch until the hidden RNG drift is explained.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 500 --frame-end 530 --wram-dump-frames 500,510,520,530
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 470 --frame-end 500
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 468 --frame-end 506
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not retry the pure “delay worker birth to frame 504” patch by itself. It hides the shell but leaves native RNG seven bytes ahead before slot 2 exists.
- Do not patch `title_screen_ready`, camera, or compare masking while the hidden mode-8 RNG drift is unresolved.
- Do not assume `loc_1AB26` RNG is the first hidden consumer. The next task is to prove what runs between mode-8 entry and the first visible worker.
