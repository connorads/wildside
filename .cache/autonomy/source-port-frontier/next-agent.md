---
slug: release-gate-md-runtime-level-entry-slot2-rng-blocker
cycle: 45
updated_at: 2026-03-31T06:10:00Z
---
# What Changed
- Re-read `Load_TitleCard`, `loc_1AB26`, `loc_1ABF4`, `loc_1AC70`, `loc_1AD48`, `Execute_Objects`, and `Mode_Standard` before touching native code again.
- Confirmed from oracle state dumps that frames 500..503 stay in `game_mode=8` with `kid_x_pos=0`, `kid_y_pos=0`, no active enemies, `number_gfx_objects=0`, and flat `rng_offset=65481`, then first expose slot 2 on frame 504 at `0x00940000,0xFFF00000` with 13 active enemies.
- Tried the obvious lazy-materialisation idea locally, then backed it out after verification: hiding the shell until frame 504 and deferring the worker RNG no longer leaked the early 7-byte offset, but frame 504 still revealed the wrong first worker set and `compare-native-oracle` moved the divergence to the later level-entry state.

# Current Model
- The old “delay worker birth” theory is still incomplete, but there is now a narrower result:
  delaying `loc_1AC70` RNG consumption is necessary, not sufficient.
- `Load_TitleCard -> loc_1AB26 -> loc_1ABF4 / loc_1AD48` only proves that object allocation and the Start-poll object exist before frame 504; it does not explain which hidden object(s) have executed by the time the first visible shell appears.
- When native hides everything through frame 503 and then materialises the whole shell on frame 504, it reveals slot 2 at `0xFFE00000,0x00A80000` with 14 active enemies and later diverges at level-entry@520. That means the first visible worker order / visibility set is still wrong even after the early RNG leak is removed.

# Strongest Evidence
- ASM:
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:823-930` `loc_6E2`, `MainGameLoop`, `Mode_Standard`, `Execute_Objects`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:1096-1230` `Init_RNG`, `Get_RandomNumber_byte/word/long`
  - `.cache/upstream/kid-chameleon-disasm/kid.asm:28588-29064` `Load_TitleCard`, `loc_1AB26`, `loc_1ABF4`, `loc_1AC70`, `loc_1AD48`, `loc_1AE2E`
- Oracle:
  - `.cache/oracle-probe/oracle/manifest.json`
  - Frame 500..503: `kid_x_pos=0`, `kid_y_pos=0`, `active_enemies=[]`, `number_gfx_objects=0`, `rng_offset=65481`
  - Frame 504: `kid_x_pos=0x00940000`, `kid_y_pos=0xFFF00000`, `active_enemies.len()=13`, `number_gfx_objects=13`
- Native baseline:
  - `cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 458 --frame-end 466`
    shows the current runtime still exposes all 14 title workers immediately on frame 460.
- Failed local experiment (backed out):
  - Hidden shell through frame 503, first reveal on frame 504
  - Frame 504 still wrong: slot 2 at `0xFFE00000,0x00A80000`, 14 active enemies
  - `cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
    then diverged at level_entry@520 (`native 000E0000,00880000; oracle 007F0000,00020000`)

# Exact Next Moves
- Do one investigation only: identify which `loc_1AC70` / `loc_1AD48` objects are actually visible on oracle frame 504 and why native’s “all 14 workers at once” model is one object too many.
- Read the object-slot / gfx-slot execution order around `Allocate_ObjectSlot`, `Load_GfxObjectSlot`, and the `Hibernate_Object_1Frame` resume path; work out whether one worker is still hibernating or off-screen on the first visible frame.
- Correlate oracle frame-504 active-enemy order with `loc_1AB26` script order and `loc_1AD48`’s deferred suffix worker so the next patch changes worker ordering/visibility, not just RNG timing.
- Do not retry the backed-out lazy-materialisation patch unchanged; it fixes one symptom and still lands on the wrong frame-504 shell.

# Verification To Re-run
- cargo run -q -p kidcli -- oracle-probe --scenario .cache/oracle-probe/boot_title_start_level1_dense.json --frame-start 499 --frame-end 504
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 458 --frame-end 466
- cargo run -q -p kidcli -- probe-native --scenario boot-title-start-level1 --frame-start 500 --frame-end 504
- cargo test -q -p kid-core boot_title_start_stays_in_mode_8_without_second_start_press -- --nocapture
- cargo test -q -p kid-core boot_title_start_mode_8_keeps_advancing_rng_before_gameplay_entry -- --nocapture
- cargo run -q -p kidcli -- compare-native-oracle --scenario boot-title-start-level1

# Dead Ends / Do Not Retry
- Do not retry the pure “delay worker birth to frame 504” patch by itself.
- Do not retry the backed-out “delay worker birth + defer `loc_1AC70` RNG” patch unchanged; it still exposes the wrong frame-504 shell and later diverges at level-entry@520.
- Do not patch compare masking, title-ready, or camera state while the frame-504 worker order / visibility mismatch is unresolved.
