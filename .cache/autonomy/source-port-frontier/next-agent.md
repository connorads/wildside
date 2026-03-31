---
slug: release-gate-md-runtime-ctrl-pressed-mode8
cycle: 13
updated_at: 2026-03-31T02:41:00Z
---
# What Changed
- Modelled the first `loc_1AB26` shell pass inside `StartingLevel`: native now expands `title_card.script_bytes` into one letter worker per visible token, keeps descending gfx-slot allocation, and derives slot 2 from that worker set instead of exposing an empty mode-8 shell.
- Preserved the ROM token walk in the fanout loop: `0` advances to the next `(x,y)` destination, `#$84` consumes horizontal space without allocating a worker, and the special glyph widths from `loc_1ABCA..loc_1ABDC` now move the destination cursor before the next token.
- Added regressions for both the plain level-00 shell (`boot_title_start_level_1_expands_loc_1ab26_letter_workers`) and the special-token level-07 shell (`boot_title_start_isle_of_the_lion_lord_preserves_loc_1ab26_spacing_tokens`).

# Current Model
- The shell is no longer empty. `compare-native-oracle` now reaches `level_entry@520` with native kid position `004EC000,00718000` instead of `00000000,00000000`, which confirms trace slot 2 is now bound to a real `loc_1AB26` worker.
- The remaining miss is motion parity, not content parity. The current shell workers use the correct token expansion and target coordinates, but their spawn edges / RNG consumption / per-frame interpolation still do not match `loc_1AC70`.
- `loc_1ABF4 -> loc_1AD48` is still missing entirely, so levels with numeric suffixes remain one worker short even after the letter shell exists.

# Strongest Evidence
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28770-28852` `loc_1AB6A` walks bytes exactly as implemented now: `beq` swaps to the next line destination, `#$84` advances `d2` by `#$10` without allocation, and `loc_1ABCA..loc_1ABDC` apply the non-uniform glyph widths.
- `.cache/upstream/kid-chameleon-disasm/kid.asm:28912-28988` `loc_1AC70` still owns the remaining parity gap: it chooses a random shell edge, samples another random axis coordinate, and moves through the object system rather than the current direct interpolation.
- `cargo test -q -p kid-core boot_title_start_level_1_expands_loc_1ab26_letter_workers -- --nocapture` passes and confirms Blue Lake Woods 1 maps gfx slot 2 to the third-row `o` at `(0x40, 0x68)`.
- `cargo test -q -p kid-core boot_title_start_isle_of_the_lion_lord_preserves_loc_1ab26_spacing_tokens -- --nocapture` passes and confirms the line-break + `#$84` spacing path for "Isle of the Lion Lord".
- `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
  and
  `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`
  both still fail first at `state level_entry@520 diverged in kid position (native 004EC000,00718000; oracle 007F0000,00020000)`.

# Exact Next Moves
- Model `loc_1ABF4 -> loc_1AD48` only: consume the extracted `level_number_destination` / `level_number_glyph_count`, allocate the deferred `#$7B` number workers, and preserve the ROM's `#$10` horizontal spacing.
- Re-run `cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture`, `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`, then `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s`.
- Do not touch `loc_1AC70` motion yet; keep the next cycle scoped to the deferred number worker only.

# Verification To Re-run
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core level_entry_has_no_damage_grace_window -- --nocapture
- cargo test -q -p kidcli compare_native_states_reports_boot_level_entry_mode_8_release_gate -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1-move-right5s

# Dead Ends / Do Not Retry
- Do not collapse `#$84` into a visible glyph. The ROM treats it as spacing only; level 07 proves native needs the same behaviour.
- Do not skip the line-destination table and reconstruct rows from `display_name`; `loc_1AB26` consumes explicit destination pairs from `AddrTbl_LevelNames`.
- Do not start tuning shell motion before the deferred number worker exists. The next missing behaviour in ASM order is `loc_1ABF4 -> loc_1AD48`, not another interpolation tweak.
