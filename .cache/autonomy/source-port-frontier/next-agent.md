---
slug: release-gate-boot-title-360-game-mode
cycle: 1
updated_at: 2026-03-31T01:34:00Z
---
# What Changed
- Verification hardening advanced: `crates/kidcli/src/sweep.rs` now treats `oracle_limitation` as non-blocking for state-tier pass/fail accounting, so state sweeps no longer auto-escalate into frame rendering just because the oracle paused.
- Added focused tests proving oracle-limit-only summaries are non-blocking while authoritative summaries still fail.
- `cargo run --release -p kidcli -- sweep --levels 0-104 --tier state` now ends `69 passed, 0 failed, 0 errors` while still printing per-level `ORACLE LIMITATION` detail.

# Current Model
- The published any-percent TAS tail remains closed through frame 37759.
- The active broader-work blocker is now `mise run release-gate` for `1p-accurate`: `boot_title_start_level1` fails at `title@360` because native reports `game_mode=0x000C` while oracle reports `0x002C`.
- Relevant ROM routines for the next cycle:
  - `kid.asm:30389` `loc_1BAEA` holds the title idle loop for `$280` frames before `loc_1BB02` sets `Game_Mode=$10`.
  - `kid.asm:31119` `sub_1C246` consumes Start and forces `Game_Mode=$2C`.

# Strongest Evidence
- Fresh sweep evidence: `/home/connor/git/kc/.cache/sweep-runs/1774920396/report.json` reports `69 passed, 0 failed, 0 errors` after the accounting fix.
- Fresh release-gate evidence: `/home/connor/git/kc/.cache/release-gate/1p-accurate/1774920510/report.json` fails with `state title@360 diverged in game mode (native 000C, oracle 002C)`.
- ASM evidence:
  - `loc_1BAEA` clears `Ctrl_Pressed`, hibernates a frame, checks `Ctrl_1_Pressed` bit 7, and falls through to `loc_1BB02` only after the `$280` countdown expires.
  - `sub_1C246` also clears `Ctrl_1_Pressed` bit 7, then forces `Game_Mode=$2C` and stops music when Start is consumed.

# Exact Next Moves
- Diagnose the `title@360` release-gate mismatch by reading `loc_1BAEA`, `loc_1BB02`, and `sub_1C246` in `kid.asm`, then rerunning the boot-title oracle comparison to determine why native stays in `0x000C` instead of taking the ROM-backed `0x002C` Start-return path.
- If ASM + oracle make the cause obvious, patch the title/start hand-off directly in native code and add a regression test for the `title@360` mode transition.
- Re-run `mise run release-gate` after the fix; only return to broader verification hardening once `boot_title_start_level1` is green again.

# Verification To Re-run
- cargo test -q -p kidcli oracle_limitation -- --nocapture
- cargo test -q -p kidcli authoritative_summary_is_blocking -- --nocapture
- cargo run --release -q -p kidcli -- sweep --levels 0-104 --tier state
- mise run release-gate

# Dead Ends / Do Not Retry
- Do not treat `oracle_limitation` as a blocking state-tier failure again; that was the accounting bug fixed in cycle 1.
- Do not reopen the closed published any-percent TAS tail unless a fresh authoritative mismatch appears there.
