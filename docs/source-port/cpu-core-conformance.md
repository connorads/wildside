# 68000 core conformance (#447)

The 68000 core under every timing claim in this repository is `r68k-emu`, pinned
by git rev in [`crates/wildside-md/Cargo.toml`](../../crates/wildside-md/Cargo.toml).
It now has conformance evidence. This page is that evidence: what is measured,
what r68k gets wrong, and what the measurement cannot say.

## Running it

```bash
mise run bootstrap-cpu-vectors       # ~600 MB, shallow, pinned, skipped if present
mise run verify-cpu-vectors          # full 1.3M-vector run, ~4 min
mise run verify-cpu-vectors-ratchet  # downward-only gate over the same run
```

`--per-file-limit N` keeps the first N vectors of each opcode file, as a prefix of
the published order so a capped run stays reproducible. `--set` restricts to one
published set. `--json` prints the machine-readable report, which is also written
to `.cache/audit/cpu-vectors.json`.

The ratchet is deliberately **not** in `verify-slice-fast` and not an hk step: it
needs a 600 MB download and minutes per run, so it is a periodic gate like
`sweep`. It skips (never shrinks) when the vector checkouts are absent.

## The vector sets

| Set | Vectors | Files | Encoding | Published `pc` |
| --- | --- | --- | --- | --- |
| [`SingleStepTests/680x0`](https://github.com/SingleStepTests/680x0) | 1,000,060 | 124 | gzipped JSON | instruction address |
| [`SingleStepTests/m68000`](https://github.com/SingleStepTests/m68000) | 317,500 | 127 | bespoke binary | MAME `m_au`, instruction + 4 |

Each vector gives full processor and memory state before and after one
instruction, plus the bus transactions and total cycles. The two sets come from
independent lineages — Tom Harte's own core and MAME's microcoded core — which is
what makes disagreement between them informative rather than merely awkward.

## Three tiers, in order of authority

1. **Arithmetic verdict.** Final registers, both stack pointers, PC, SR and every
   published memory byte, over vectors where no exception was dispatched, with
   r68k's absent trace bit forgiven. This is the tier that says whether the core
   computes the right answer.
2. **Exception frames.** The same comparison over vectors that dispatched an
   exception (detected from a published vector-table fetch).
3. **Timing and bus.** Total cycle count, and the ordered data-space access list.

Splitting them is not grading on a curve: one systematic frame defect otherwise
buries every arithmetic result behind 194k failures, and a core can be
state-exact while charging the wrong cycles — which is precisely the debt the
`oracle-injection-ratchet` pays for.

## Measured result

Both vector sets, uncapped, r68k rev `9119048`:

| | 680x0 | m68000 |
| --- | --- | --- |
| Arithmetic verdict | **790,382/805,209 (98.16%)** | **233,935/240,501 (97.27%)** |
| Exception frames | 16,749/194,851 (8.60%) | 10,618/76,999 (13.79%) |
| Cycle count | 827,906/1,000,060 (82.79%) | 250,079/317,500 (78.77%) |
| Data-space bus sequence | 729,228 (72.92%) | 222,758 (70.16%) |
| Whole-state pass (all tiers, unforgiven) | 794,656 (79.46%) | 127,913 (40.29%) |

Arithmetic failures are confined to **10 opcode files in each set**. Everything
else — 114 of 124 and 117 of 127 files — is exact on every vector.

## Findings

### r68k defects (both sets agree)

- **BCD arithmetic is wrong.** `SBCD` fails 3,715/8,065 (46%) and 1,129/2,499
  (45%); `NBCD` 3,404/8,056 (42%) and 1,037/2,500 (41%); `ABCD` 814/8,064 (10%)
  and 279/2,500 (11%). Mostly SR, but 483 `SBCD` and 159 m68000 `SBCD` vectors
  differ in the **destination register**, so this is not only the officially
  undefined N/V bits. Kid uses BCD for score and HUD digits.
- **`DIVS`/`DIVU` diverge on overflow.** `DIVS` fails 1,895/4,985 (38%) and
  1,025/1,518 (68%); `DIVU` 1,247/4,960 (25%) and 687/1,546 (44%). `divu_16`
  (r68k `common.rs`) sets only `V` on overflow and leaves the destination and
  the other flags untouched.
- **`Bcc` misses a small case.** 14/5,865 and 6/1,830 — under 0.4%, but present
  in both sets, so real.
- **Division and multiplication cycle counts are constants.** `mulu!` charges a
  flat 54 cycles; the hardware takes `38 + 2 × popcount(source)`, verified
  against 1,221/1,221 `MULU Dy,Dx` vectors, spanning 40–66 cycles. r68k is
  therefore right for 235 of 1,221 operand values (19%) and off by up to ±14
  cycles. `DIVU` is a flat 140 against a published 10–130 spread; `DIVS` a flat
  158 against 16–152. **This is the quantified form of the known r68k-vs-jgenesis
  `MULU` disagreement** recorded in the #406 NOW block: r68k is not slightly out,
  it models a data-dependent instruction as a constant.
- **Exception stack frames are systematically wrong.** Only 8.6%/13.8% of
  exception-dispatch vectors pass. Two concrete causes, from
  `handle_address_error`: the group-0 special status word is written as
  `0b10000 | fc` where the hardware leaves the instruction register's upper bits
  in place (`0x0015` vs a published `0x4A75` for `IR = 0x4A66`), and the stacked
  PC is r68k's post-fetch `pc` rather than the value the hardware pushes.
- **r68k panics instead of raising an exception.** 1,115 m68000 vectors abort the
  thread: `pop_32`, `push_sp` and `jump_vector` `unwrap()` a
  `read_data_long`/`write_data_long`, so an odd stack pointer or vector address
  aborts the process where the real chip takes an address error. `RTS` with an
  odd SP is enough. The harness catches and counts these; production code does
  not, so this is a live crash path, not only a fidelity gap.
- **No trace support at all.** r68k has no `T` bit (`CPU_SR_MASK` admits it but
  no field stores it) and no trace exception. 116,666 m68000 vectors — roughly
  half that set — fail on the trace bit alone; the 680x0 set barely exercises it
  (12,477).

### The two sets disagree with each other

Detected mechanically: r68k is deterministic and both sets sample the same
encoding space, so one set failing thousands of vectors on an opcode while the
other passes every one means the sets expect different final state. The clean
side exonerates r68k.

| Opcode file | 680x0 | m68000 | Disagreeing set |
| --- | --- | --- | --- |
| `ASR.b` | 1,642/8,065 fail | 0/2,500 | 680x0 |
| `ASR.w` | 1,063/7,141 fail | 0/2,217 | 680x0 |
| `ASR.l` | 1,031/8,065 fail | 0/2,500 | 680x0 |
| `CHK` | 0/676 | 107/197 fail | m68000 |
| `LINK` | 0/8,065 | 326/2,500 fail | m68000 |
| `MOVEfromSR` | 0/5,013 | 740/1,521 fail | m68000 |

The `ASR` case is confirmed by hand. For `ASR` with a shift count at or above the
operand size and a negative operand, both sets agree the result is all sign bits,
but the 680x0 set expects `C` and `X` **clear** while the m68000 set expects both
**set**. r68k sets them, matching m68000 and matching the M68000UM ("set
according to the last bit shifted out"). So those 3,736 apparent r68k defects are
a 680x0-set discrepancy, and `ASR` is exonerated. It is worth reporting upstream.

`STOP` is not in this table only because the 680x0 set has no `STOP` file. r68k
fails all 1,230 non-exception m68000 `STOP` vectors on PC, unreviewed and
uncorroborated.

There are also **79 opcode words where the two sets disagree on cycle count**,
all `ADDQ`/`SUBQ` with an address-register destination (`0x5088`, `0x518A`, …):
680x0 says 6 cycles, m68000 says 8. r68k's `addq_32_an` returns 8, siding with
m68000 and with Table 8-5 of the M68000UM.

## What this does not say

- **Nothing here is about Kid Chameleon.** A green arithmetic verdict does not
  make any timing claim in this port correct; it removes the CPU core as a
  suspect for state divergence in 98% of instruction space, which is a different
  and smaller statement.
- **The bus tier is weak by construction.** r68k has no prefetch queue: its
  `read_imm_u16` caches a longword from `pc & !3` where the real chip holds two
  separately-fetched words. So program-space fetches are structurally
  incomparable and are excluded, longword accesses are split into two word
  transactions to compare at all, cycle *positions* within an instruction are not
  compared (only the total), and the final prefetch queue is not compared. The
  70-73% data-bus figure is a same-order/same-value check on data accesses only.
- **The prefetch queue is reconstructed, not injected.** The harness writes the
  two published prefetch words into memory at the instruction address. That
  reproduces the publishing machine's memory rather than inventing it — on the
  real machine `prefetch[0]` *is* the word at `pc` — and the runner counts
  vectors where a set's own `ram` contradicts its `prefetch` (measured: zero, in
  both sets).
- **The exception partition is a heuristic.** It keys on a published data-space
  read below `0x400`. An operand address that happens to land in the vector table
  would be misfiled. It never changes whether a vector passes.
- **`TAS` timing is known-bad in the m68000 set upstream** (its README says so),
  and `TRAPV` there is flagged as suspect by its own author. Read those rows with
  that in mind.
