# ROM level inventory

The inventory records all 106 non-sentinel logical entries in `MapOrder_Index`.
The standard sweep range `0-104` covers 105 entries; the additional `$69` entry
is a Crypt leftover. `$6A/$6B` contain `$FF` and are sentinels.

There are 103 named playable candidates and three labelled leftovers. A candidate
is a route-discovery obligation, not a claim of reachability or completed play.
ASM `constants.asm` labels `$38/$47` unplayable; `doc/level_list.txt` also labels
`$69` leftover. `$41` is explicitly the playable Crypt and must not inherit that
exclusion. Its shared-header assets resolve through the shared ROM header.

Sources are the pinned disassembly's `level/maporder.asm`, `level/level_files.txt`,
`doc/level_list.txt`, `constants.asm` and `Load_InGame` in `kid.asm`.
Disassembly revision: `a13456716aec684fa8af7addbbdb5069a7415d69`.
The header column identifies the extracted shared-header resource, not a second
logical level. Platform and scroll resources can differ even when headers match.

BCD filtering omits every index ending in A-F, totalling 36 named candidates.
A green 69-entry sweep therefore cannot establish coverage of the standard range.
Idle warp checkpoints and attract-demo level changes do not establish completion.
Natural route evidence belongs in a progression ledger with an observed entry,
traversal and exit, game mode, recording frame interval and emulator comparison.

## Inspection identity

`inspect-level` reads interactive records through the runtime's logical-level
loader. RAM captures use logical IDs; ROM foreground and BLOCK resources use
`MapOrder_Index`. A missing logical capture must never fall back to a capture
whose filename happens to equal the mapped resource ID.

Stairway to Oblivion, logical `$3A`, maps to resource `$03`. It has no logical
RAM capture. Its six `loc_1269C` teleporter records come from mapped ROM data:

| Source cell | Destination resource | Destination cell |
|---|---|---|
| 32,55 | `$03` | 7,8 |
| 39,64 | `$03` | 27,55 |
| 65,67 | `$03` | 27,55 |
| 87,67 | `$1C` | 2,50 |
| 68,27 | `$75` | 1,21 |
| 70,4 | `$0D` | 1,20 |

Resolve destination resources through the ROM's first-match `loc_BCD6`
search before naming logical progression edges. These records are discovery
targets, not traversal evidence. The isolated inspector regression rejects
logical `$03`'s records when `$3A` lacks either its layout or its block list,
and accepts `$3A`'s own capture when present. The live command reports
38 prizes, six ghost groups, six teleporters and eight hidden records.
Inspection of all 106 non-sentinel logical entries returns block lists with
163 teleporter records, including 65 same-level destinations. Every destination
resource resolves through `MapOrder_Index`. These totals include the three
leftovers and do not count natural exits or completed levels.

## Expanded sweep evidence

State run `1789928876` tests all 105 entries requested by `--levels 0-104`.
It contains 92 comparable passes, 10 failures and three fully masked/unverified
entries (`$38/$41/$47`). The report's `passed=95` includes those three unverified
entries. Every failing ID ends in a hex digit A-F. The 28 authoritative
differences remain release work; the expanded sweep is not a passing release gate.

Commands:

```sh
cargo test -q -p wildside-cli parse_level_range_
cargo run -q -p wildside-cli -- sweep --levels 10 --tier gameplay
cargo run -q -p wildside-cli -- sweep --levels 0-104 --tier state
cargo run -q -p wildside-cli -- sweep-census --run 1789928876
mise run verify-disasm-fidelity
```

The single-level command tests Windy City `$0A` and passes. Regression tests
require letter-digit indices, all 105 standard entries and the final non-sentinel
entry. The disassembly assembles byte-identical to the 1,048,576-byte ROM.
The `sweep` command reports failures in its output/JSON
without a nonzero process exit; inspect the report counts, not just exit status.

| Logical ID | Checkpoint | Authoritative native / oracle differences |
|---|---|---|
| `0x0F` | `level_settle` | number_hitpoints `1` / `2` |
| `0x0F` | `idle_tail` | number_hitpoints `1` / `2` |
| `0x1B` | `idle_tail` | game_mode `0x0008` / `0x000c`; number_objects `0` / `4`; number_lives `2` / `3` |
| `0x1C` | `level_settle` | number_hitpoints `1` / `2` |
| `0x1C` | `idle_tail` | number_hitpoints `1` / `2` |
| `0x1F` | `level_settle` | number_objects `5` / `6`; number_gfx_objects `8` / `10` |
| `0x1F` | `idle_tail` | number_objects `5` / `6`; number_gfx_objects `8` / `10` |
| `0x2C` | `idle_tail` | number_objects `10` / `7`; number_gfx_objects `16` / `13` |
| `0x3E` | `level_settle` | camera_x_pos `0x00080000` / `0x00070000`; murder_wall_x `Some(575232)` / `Some(488448)` |
| `0x3F` | `idle_tail` | game_mode `0x0008` / `0x000c`; number_objects `0` / `11`; number_gfx_objects `14` / `15`; number_hitpoints `2` / `1`; number_lives `2` / `3` |
| `0x4B` | `idle_tail` | number_objects `11` / `10`; number_gfx_objects `20` / `19` |
| `0x4C` | `level_settle` | number_objects `5` / `7`; number_gfx_objects `5` / `7` |
| `0x4C` | `idle_tail` | number_objects `5` / `6`; number_gfx_objects `5` / `6` |
| `0x5C` | `level_settle` | number_objects `5` / `6`; number_gfx_objects `6` / `7` |

## Shared-header verification

ASM `MapHeader_09_1A_41_42_48` owns complete resource09 foreground, block,
background and enemy data. The extracted catalogue omits these paths for
aliases; manifest normalisation inherits them from identical headers before
metadata parsing and decoding. Platform and scroll paths remain independent.
The bundle contract requires decoded entries for every record with layouts,
so a stale sparse bundle rebuilds. The browser package contains the aliases.

Crypt `$41` loads and renders at frames 0, 60 and 120. Full comparison run
`1789938866` exercises six checkpoints each for `$38/$41/$47`, all comparable
with no state differences. Seven mismatched pixels near the HUD lives icon
remain under #433. The Robot's first and third shot lifecycles match separate
framewise probes; parent position, interruptions and natural entry remain
unverified. These checkpoints do not establish whole-level fidelity.

State run `1789938867` covers all 105 standard entries with 95 comparable
passes, the same ten failures and zero fully masked levels. The ten failure
records match run `1789928876` exactly. This two-checkpoint tier does not sample
frame 300 in Crypt, so the fuller comparison remains required.
The sweep classifies oracle context from pause, title mode and requested-level
mismatch rather than a blanket Crypt exclusion.

The measured settle-shell table remains a separate coverage gap: its 69 rows
omit A-F logical indices. Its witnesses validate measured rows only, not the
entire ROM inventory. Re-measure missing entries from the emulator; do not
populate them from native output or fit new per-level delays.

## Entry inventory

| Logical ID | Resource ID | Name | Shared header | Disposition |
|---|---|---|---|---|
| `0x00` | `0x4A` | Blue Lake Woods 1 | `header/4A.bin` | candidate |
| `0x01` | `0x4B` | Blue Lake Woods 2 | `header/4B.bin` | candidate |
| `0x02` | `0x2D` | Highwater Pass 1 | `header/2D.bin` | candidate |
| `0x03` | `0x2E` | Highwater Pass 2 | `header/2E.bin` | candidate |
| `0x04` | `0x01` | Under Skull Mountain 1 | `header/01.bin` | candidate |
| `0x05` | `0x00` | Under Skull Mountain 2 | `header/00.bin` | candidate |
| `0x06` | `0x02` | Under Skull Mountain 3 | `header/02.bin` | candidate |
| `0x07` | `0x0E` | Isle of the Lion Lord | `header/0E.bin` | candidate |
| `0x08` | `0x54` | Hills of the Warrior 1 | `header/54.bin` | candidate |
| `0x09` | `0x56` | Hills of the Warrior 2 | `header/56.bin` | candidate |
| `0x0A` | `0x18` | Windy City | `header/18.bin` | candidate |
| `0x0B` | `0x1D` | Sinister Sewers | `header/1D.bin` | candidate |
| `0x0C` | `0x3D` | The Crystal Crags 1 | `header/3D.bin` | candidate |
| `0x0D` | `0x37` | The Crystal Crags 2 | `header/37.bin` | candidate |
| `0x0E` | `0x2C` | Dragonspike | `header/2C.bin` | candidate |
| `0x0F` | `0x2B` | Stormwalk Mountain | `header/2B.bin` | candidate |
| `0x10` | `0x13` | Shishkaboss | `header/13.bin` | candidate |
| `0x11` | `0x4C` | The Whispering Woods 1 | `header/4C.bin` | candidate |
| `0x12` | `0x49` | The Whispering Woods 2 | `header/49.bin` | candidate |
| `0x13` | `0x14` | Devil's Marsh 1 | `header/14.bin` | candidate |
| `0x14` | `0x16` | Devil's Marsh 2 | `header/16.bin` | candidate |
| `0x15` | `0x0B` | Knight's Isle | `header/0B.bin` | candidate |
| `0x16` | `0x0C` | Whale Grotto | `header/0C.bin` | candidate |
| `0x17` | `0x3F` | Hoverboard Beach | `header/3F.bin` | candidate |
| `0x18` | `0x40` | Pyramids of Peril | `header/40.bin` | candidate |
| `0x19` | `0x34` | Madmaze Mountain | `header/34.bin` | candidate |
| `0x1A` | `0x1C` | The Deadly Skyscrapers | `header/1C.bin` | candidate |
| `0x1B` | `0x27` | Skydragon Castle 1 | `header/27.bin` | candidate |
| `0x1C` | `0x21` | Skydragon Castle 2 | `header/21.bin` | candidate |
| `0x1D` | `0x0D` | Coral Blade Grotto | `header/0D.bin` | candidate |
| `0x1E` | `0x6C` | Boomerang Bosses | `header/6C.bin` | candidate |
| `0x1F` | `0x4F` | Woods of Despair 1 | `header/4F.bin` | candidate |
| `0x20` | `0x4D` | Woods of Despair 2 | `header/4D.bin` | candidate |
| `0x21` | `0x19` | Forced Entry | `header/19.bin` | candidate |
| `0x22` | `0x0A` | The Cliffs of Illusion | `header/0A.bin` | candidate |
| `0x23` | `0x5C` | Lion's Den | `header/5C.bin` | candidate |
| `0x24` | `0x25` | Wind Castles 1 | `header/25.bin` | candidate |
| `0x25` | `0x26` | Wind Castles 2 | `header/26.bin` | candidate |
| `0x26` | `0x35` | Blizzard Mountain | `header/35.bin` | candidate |
| `0x27` | `0x3E` | Caves of Ice | `header/3E.bin` | candidate |
| `0x28` | `0x2F` | The Nightmare Peaks 1 | `header/2F.bin` | candidate |
| `0x29` | `0x31` | The Nightmare Peaks 2 | `header/31.bin` | candidate |
| `0x2A` | `0x74` | Bagel Brothers | `header/74.bin` | candidate |
| `0x2B` | `0x3B` | Diamond Edge | `header/3B.bin` | candidate |
| `0x2C` | `0x5A` | The Hills Have Eyes | `header/5A.bin` | candidate |
| `0x2D` | `0x33` | Secrets in the Rocks | `header/33.bin` | candidate |
| `0x2E` | `0x3A` | Ice God's Vengeance | `header/3A.bin` | candidate |
| `0x2F` | `0x57` | Beneath the Twisted Hills | `header/57.bin` | candidate |
| `0x30` | `0x10` | Alien Isle | `header/10.bin` | candidate |
| `0x31` | `0x59` | The Land Below | `header/59.bin` | candidate |
| `0x32` | `0x38` | The Final Marathon | `header/38.bin` | candidate |
| `0x33` | `0x05` | Plethora | `header/05.bin` | candidate |
| `0x34` | `0x30` | The Pinnacle | `header/30.bin` | candidate |
| `0x35` | `0x55` | Hidden Canyon | `header/55.bin` | candidate |
| `0x36` | `0x32` | The Caged Beasts | `header/32.bin` | candidate |
| `0x37` | `0x0F` | Crab Cove | `header/0F.bin` | candidate |
| `0x38` | `0x48` | The Crypt (leftover) | `header/09.bin` | leftover |
| `0x39` | `0x47` | The Forbidden Tombs | `header/47.bin` | candidate |
| `0x3A` | `0x03` | Stairway to Oblivion | `header/03.bin` | candidate |
| `0x3B` | `0x53` | The Valley of Life | `header/53.bin` | candidate |
| `0x3C` | `0x04` | The Black Pit | `header/04.bin` | candidate |
| `0x3D` | `0x36` | Frosty Doom | `header/36.bin` | candidate |
| `0x3E` | `0x15` | Bloody Swamp | `header/15.bin` | candidate |
| `0x3F` | `0x12` | Scorpion Isle | `header/12.bin` | candidate |
| `0x40` | `0x22` | Towers of Blood | `header/22.bin` | candidate |
| `0x41` | `0x41` | The Crypt (playable version) | `header/09.bin` | candidate |
| `0x42` | `0x17` | Alien Twilight | `header/17.bin` | candidate |
| `0x43` | `0x4E` | Tunnels Beneath the Woods | `header/4E.bin` | candidate |
| `0x44` | `0x58` | Hills of Forever | `header/58.bin` | candidate |
| `0x45` | `0x11` | Monster Island | `header/11.bin` | candidate |
| `0x46` | `0x39` | The Shimmering Caves | `header/39.bin` | candidate |
| `0x47` | `0x42` | The Crypt (leftover) | `header/09.bin` | leftover |
| `0x48` | `0x3C` | Sky Fortress | `header/3C.bin` | candidate |
| `0x49` | `0x61` | Elsewhere 1 | `header/61.bin` | candidate |
| `0x4A` | `0x5E` | Elsewhere 2 | `header/5E.bin` | candidate |
| `0x4B` | `0x5F` | Elsewhere 3 | `header/5F.bin` | candidate |
| `0x4C` | `0x60` | Elsewhere 4 | `header/60.bin` | candidate |
| `0x4D` | `0x5D` | Elsewhere 5 | `header/5D.bin` | candidate |
| `0x4E` | `0x62` | Elsewhere 6 | `header/62.bin` | candidate |
| `0x4F` | `0x64` | Elsewhere 7 | `header/64.bin` | candidate |
| `0x50` | `0x67` | Elsewhere 8 | `header/67.bin` | candidate |
| `0x51` | `0x6E` | Elsewhere 10 | `header/6E.bin` | candidate |
| `0x52` | `0x75` | Elsewhere 9 | `header/75.bin` | candidate |
| `0x53` | `0x66` | Elsewhere 11 | `header/66.bin` | candidate |
| `0x54` | `0x68` | Elsewhere 12 | `header/68.bin` | candidate |
| `0x55` | `0x6D` | Elsewhere 13 | `header/6D.bin` | candidate |
| `0x56` | `0x6F` | Elsewhere 14 | `header/6F.bin` | candidate |
| `0x57` | `0x70` | Elsewhere 32 | `header/70.bin` | candidate |
| `0x58` | `0x77` | Elsewhere 16 | `header/77.bin` | candidate |
| `0x59` | `0x78` | Elsewhere 15 | `header/78.bin` | candidate |
| `0x5A` | `0x63` | Elsewhere 17 | `header/63.bin` | candidate |
| `0x5B` | `0x65` | Elsewhere 19 | `header/65.bin` | candidate |
| `0x5C` | `0x69` | Elsewhere 18 | `header/69.bin` | candidate |
| `0x5D` | `0x6A` | Elsewhere 22 | `header/6A.bin` | candidate |
| `0x5E` | `0x6B` | Elsewhere 20 | `header/6B.bin` | candidate |
| `0x5F` | `0x71` | Elsewhere 21 | `header/71.bin` | candidate |
| `0x60` | `0x08` | Elsewhere 23 | `header/08.bin` | candidate |
| `0x61` | `0x7C` | Elsewhere 24 | `header/7C.bin` | candidate |
| `0x62` | `0x7D` | Elsewhere 25 | `header/7D.bin` | candidate |
| `0x63` | `0x7A` | Elsewhere 26 | `header/7A.bin` | candidate |
| `0x64` | `0x7B` | Elsewhere 27 | `header/7B.bin` | candidate |
| `0x65` | `0x79` | Elsewhere 28 | `header/79.bin` | candidate |
| `0x66` | `0x72` | Elsewhere 29 | `header/72.bin` | candidate |
| `0x67` | `0x73` | Elsewhere 30 | `header/73.bin` | candidate |
| `0x68` | `0x76` | Elsewhere 31 | `header/76.bin` | candidate |
| `0x69` | `0x09` | The Crypt (leftover) | `header/09.bin` | leftover |
