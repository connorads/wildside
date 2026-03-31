---
slug: any-percent-published-tail-clean-through-end
updated_at: 1774918028
status: complete
---
# What Changed
- The TAS frontier is closed through frame 37759; broader source-port work remains open.

# Current Model
- Recording: any-percent-published
- Area: Published any-percent TAS tail clean through frame 37759 after the later intro-sequence-1 hand-off and demo-2 transform-abort repairs
- Authoritative clean through: frame 37759
- Current blocker: None. `published_tas_later_timeout_restart_intro_sequence_1_restarts_demo_play_on_frame_34916` and `published_tas_later_demo_2_transform_ignores_start_abort_on_frame_35148` both pass, `verify-tas --frame-start 35140 --frame-end 36000 --checkpoint-interval 1 --bisect` is CRC-clean, the later tail chunks 36000..37000 and 37000..37759 are also CRC-clean, and `mise run verify-slice-fast` passes.

# Strongest Evidence
- Hypothesis: No active authoritative frontier remains in the published any-percent tail: Frame 34916 needed the later intro-sequence-1 title idle seeded two frames ahead of the shared `Obj_TitleMenu loc_1BAEA -> loc_1BB02` hand-off, and frame 35148 needed demo Start-abort edges suppressed while Kid_Transform still hibernated the gameplay/object loop. With both fixes in place, chunked CRC checks keep the published tail clean through frame 37759.

# Exact Next Moves
- Do not continue TAS-window work unless a new authoritative frontier is opened.
- Choose the next task from broader source-port remaining work: 1P playability, verification hardening, or another explicit backlog risk area.
- Update the next-agent note again only after that broader task has fresh evidence.

# Verification To Re-run
- cargo test -q -p kid-core published_tas_later_timeout_restart_intro_sequence_5_keeps_camera_handoff_on_frame_32005 -- --nocapture
- cargo test -q -p kid-core published_tas_later_timeout_restart_title_clears_intro_sequence_4_camera_on_frame_32039 -- --nocapture
- cargo test -q -p kid-core published_tas_later_timeout_restart_title_restores_pose_on_frame_32156 -- --nocapture
- cargo test -q -p kid-core published_tas_later_timeout_restart_title_resumes_demo_play_on_frame_32704 -- --nocapture
- cargo test -q -p kid-core published_tas_later_timeout_restart_demo_return_reuses_title_pose_on_frames_34368_and_34369 -- --nocapture
- cargo test -q -p kid-core published_tas_later_timeout_restart_intro_sequence_1_restarts_demo_play_on_frame_34916 -- --nocapture
- cargo test -q -p kid-core published_tas_later_demo_2_transform_ignores_start_abort_on_frame_35148 -- --nocapture
- cargo test -q -p kid-core published_tas_later_intro_sequence_5_revisit_exposes_loc_1bcd0_slot_2_on_frame_27603 -- --nocapture
- cargo test -q -p kid-core published_tas_later_intro_sequence_5_revisit_enters_intro_sequence_6_on_frame_28066 -- --nocapture
- cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098 -- --nocapture
- cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_uses_delayed_seeded_camera_row_on_frame_28101 -- --nocapture
- cargo test -q -p kid-core published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317 -- --nocapture
- cargo test -q -p kid-core published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926 -- --nocapture
- cargo run -q -p kidcli -- verify-tas --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 23250 --bisect
- cargo run -q -p kidcli -- tas-diff --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 32034 --frame-end 32050 --include-advisory
- cargo run -q -p kidcli -- tas-diff --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 32150 --frame-end 32166 --include-advisory
- cargo run -q -p kidcli -- tas-diff --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 32696 --frame-end 32712 --include-advisory
- cargo run -q -p kidcli -- tas-diff --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 34360 --frame-end 34376 --include-advisory
- cargo run -q -p kidcli -- tas-diff --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 34908 --frame-end 34924 --include-advisory
- cargo run -q -p kidcli -- tas-diff --scenario tests/data/tas/any-percent-published/scenario.json --frame-start 35140 --frame-end 35160 --include-advisory
- mise run verify-slice-fast

# Dead Ends / Do Not Retry
- Re-open TAS continuation only if a new authoritative mismatch exists.
