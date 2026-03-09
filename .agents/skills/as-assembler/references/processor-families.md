# AS Macro Assembler V1.42 -- Complete Processor Family Reference

Extracted from the official manual at `https://ubuntourist.codeberg.page/Altair-8800/_static/as_EN.html`.

AS supports **60+ processor families** spanning 4-bit microcontrollers to 32-bit RISC/DSP processors. This document catalogues every family with CPU names, address spaces, special instructions, and gotchas.

---

## Table of Contents

- [8-bit Processors](#8-bit-processors)
- [16-bit Processors](#16-bit-processors)
- [32-bit Processors](#32-bit-processors)
- [4-bit Microcontrollers](#4-bit-microcontrollers)
- [Digital Signal Processors (DSP)](#digital-signal-processors-dsp)
- [Miscellaneous / Specialised](#miscellaneous--specialised)

---

## 8-bit Processors

### Intel 8008

**CPU names:** `8008`, `8008NEW`

- **Address spaces:** CODE 16K, DATA 8
- **Notes:** Intel redefined mnemonics around 1975; `8008NEW` reflects the new instruction set. Simultaneous support of both sets is not possible due to mnemonic conflicts.

### Intel 8080 / 8085

**CPU names:** `8080`, `8085`, `8085UNDOC`

- **Address spaces:** CODE 64K, IO 256
- **Pseudo-ops:** `Z80SYNTAX <ON|OFF|EXCLUSIVE>` enables writing 8080/8085 instructions in Z80-style operand notation.
- **Z80 syntax conflicts (non-exclusive mode):**
  - `CP` -- Intel: "Call on Positive"; Zilog: "Compare". With a single numeric argument, assembler generates the jump. Write `CP A,12h` to force compare.
  - `JP` -- Intel: "Jump on Positive"; Zilog: general jump. Single-argument form generates conditional jump. Use `JMP addr` for unconditional.
- **8085 extras:** `RIM` and `SIM` (interrupt mask / I/O pins). In Z80 style: `LD A,IM` / `LD IM,A`.
- **8085UNDOC instructions:** `DSUB` (HL -= BC), `ARHL` (arithmetic shift right HL), `RDEL` (rotate left DE), `LDHI d8` (DE = HL + d8), `LDSI d8` (DE = SP + d8), `RSTV` (restart on overflow to 40h), `SHLX` (store HL via DE), `LHLX` (load HL via DE), `JNX5`/`JX5` (jump on flag X5 clear/set). All have Z80-syntax equivalents. Use at own risk -- Z80 uses these opcodes differently.

### Intel 8086 .. V35

**CPU names:** `8086`, `80186`, `V30`, `V35`

- **Address spaces:** CODE 64K, DATA 64K, XDATA 64K, IO 64K
- **Small model only:** One code segment (max 64K), one data segment. Only COM files for DOS. Must `ORG 100h` for DOS programs.
- **Addressing syntax differs from MASM:** Expression without brackets is always immediate. Use `[vari]` for memory access. See Table 4.5 in manual for MASM vs AS differences.
- **ASSUME required:** Informs assembler about segment register values for automatic prefix insertion.
- **8087 coprocessor:** `WAIT` auto-inserted before every FPU instruction. Prefix `FN` (e.g. `FNINIT`) to suppress the `WAIT`.
- **Far jumps:** `CALLF` / `JMPF` with segment:offset or linear address.
- **Gotcha:** The 8-bit versions (8088, V20) are instruction-compatible; select `8086` for an 8088 system.

### MCS-48 Family (Intel)

**CPU names:** `8021`, `8022`, `8401`, `8411`, `8421`, `8461`, `8039`, `80C39`, `MSM80C39`, `8048`, `80C48`, `MSM80C48`, `8041`, `8042`, `80C382`

- **Address spaces:** CODE 1K-8K (varies by chip), IDATA 256, XDATA 256 (on variants with MOVX)
- **Banking:** 4K address space split into 2 banks of 2K. `SEL MB0`/`SEL MB1` switches banks.
- **Auto bank switching:** `JMP` and `CALL` automatically insert `SEL MBx` if target is in a different bank. Explicit `SEL MBx` can confuse this automatism.
- **Gotcha:** Jump instructions may grow from 2 to 3 bytes due to bank-switch insertion.
- **Chip differences:** 8021/8022 have strongly reduced instruction set. 8022 adds A/D converter commands. 8039/80C39 lack BUS (port 0) instructions. 8041/8042 add bus interface instructions but omit others. 80C382 (Siemens telephone variant) supports HALT and indirect DJNZ/DEC.

### MCS-51 Family (Intel)

**CPU names:** `87C750`, `8051`, `8052`, `80C320`, `80C501`, `80C502`, `80C504`, `80515`, `80517`, `80C390`

- **Address spaces:** CODE 64K (16M for 80C390), DATA 256 (initial 80h), IDATA 256, XDATA 64K (16M for 80C390), BITDATA 256, IO (absent)
- **Include files:** `STDDEF51.INC`, `80C50X.INC` -- must be included AFTER `CPU` statement.
- **Generic branch:** `JMP` auto-selects `SJMP`/`AJMP`/`LJMP`; `CALL` auto-selects `ACALL`/`LCALL`.
- **USING macro:** Sets `AR0..AR7` to correct absolute addresses for the active register bank. Updates `RegUsage` bitmap.
- **80C504 workaround:** Mask flaw with `AJMP`/`ACALL` at second-last address of a 2K page; AS auto-uses long instructions.
- **80C390:** Dallas Semiconductor's "small solution" -- extends to 16M address space with minimal ISA changes.

### MCS-251 (Intel)

**CPU names:** `80C251`

- **Address spaces:** CODE 16M, IO 512
- **Major changes from 8051:**
  - No separate BIT address space; bits addressed as `address.bitposition` (e.g. `PSW.7`).
  - `SFRB` replaced by `SFR`.
  - Internal RAM, XDATA, CODE unified into single 16M CODE space. Internal RAM at address 0, internal ROM at FF0000h.
  - SFRs moved to separate IO segment with same addresses.
- **Registers:** `R8..R15`, `WR0..WR30`, `DR0..DR28`, `DR56`, `DR60`, `DPX`, `SPX`.
- **Intel address syntax:** `XX:YYYY` (bank:offset) or linear addresses. `S:` prefix required for I/O addresses.
- **Generic branches:** `JMP` uses shortest variant including 24-bit. `CALL` will NOT use `ECALL` (pushes extra byte) -- use explicitly if needed.
- **PUSH ambiguity:** `PUSH` always pushes a byte; use `PUSHW` for word.
- **Gotcha:** Operate in source mode for extended ISA; otherwise all instructions become 1 byte longer.

### MCS-96/196/296 (Intel)

**CPU names:** `8096`, `80196`, `80196N`, `80296`

- **Address spaces:** CODE 64K (16M for 80196N), DATA (varies)
- **Progression:** 80196 adds instructions and RAM windowing. 80196N extends to 16M with new instructions. 80296 adds DSP instructions and second window register but removes PTS (loses 2 instructions).

### Motorola 6800 / 6801 / 6301 / 6811

**CPU names:** `6800`, `6801`, `6301`, `6811`

- **Address spaces:** CODE 64K
- **HC11K4 banking:** Address space extended to 64K + 2x512K. AS treats as one space $000000-$10FFFF. Use `ASSUME` for bank register mapping. `PRWINS` pseudo-instruction prints current window mapping.
- **BSET/BCLR/BRSET/BRCLR:** Arguments may be separated by spaces or commas (both accepted).
- **Progression:** 6801 adds a few instructions; 6301 a few more; 6811 adds second index register and many more instructions.

### Motorola 6805 / 68HC(S)08

**CPU names:** `6805`, `68HC08`, `68HCS08` (and variants)

- **Address spaces:** CODE 8K-64K (varies)
- **Notes:** Partially source-compatible with other 68xx but different binary format. Reduced instruction set compared to 6809.

### Motorola 6809 / 6309

**CPU names:** `6809`, `6309`

- **Address spaces:** CODE 64K
- **6309:** CMOS version officially compatible with 6809, but unofficially offers more registers and many new instructions.

### Motorola 68HC12 / 68HC12X / XGATE

**CPU names:** `68HC12`, `68HC12X`, `XGATE`

- **Address spaces:** CODE 64K
- **12X:** Additional instructions and new addressing modes for existing instructions.

### Freescale S12Z (MagniV)

**CPU names:** Many S912ZVxxx variants (e.g. `S912ZVC19F0MKH`, `S912ZVH128F2CLQ`, etc.)

- **Address spaces:** CODE 16M
- **Notes:** All variants share same CPU core and instruction set; differ only in on-chip peripherals and memory sizes.

### Motorola 68HC16

**CPU names:** `68HC16`

- **Address spaces:** CODE 1M

### Freescale 68RS08

**CPU names:** `68RS08`

- **Address spaces:** CODE 16K

### Motorola 6804

**CPU names:** `6804`

- **Address spaces:** CODE 4K, DATA 256
- **Notes:** Internally a bit-mirrored ST62 core! SGS-Thomson recycled the processor core with different peripherals and Motorola-style syntax. Operand fields are NOT mirrored (unlike opcodes), which is why AS supports it as a distinct target.

### Rockwell 6502 Family

**CPU names:** `6502`, `65SC02`, `65C02`, `W65C02S`, `65CE02`, `65C19`, `HUC6280`, `6502UNDOC`

- **Address spaces:** CODE 64K (2M for HUC6280)
- **65C02:** Additional instructions and new instruction/addressing mode combinations.
- **W65C02S:** Adds two opcodes for fine-grained CPU stop control.
- **65SC02:** Lacks 65C02 bit manipulation instructions.
- **65CE02:** 16-bit branch displacements, Z register, 16-bit stack pointer, programmable base page.
- **65C19:** NOT binary upward compatible! Some addressing modes replaced. Adds DSP-oriented instructions.
- **HUC6280:** 2M address space via banking. Special video processor communication instructions.
- **6502UNDOC:** Enables undocumented opcodes (SLO, ANC, RLA, SRE, ASR, RRA, ARR, SAX, ANE, SHA, SHS, SHY, SHX, LAX, LXA, LAE, DCP, SBX, ISB, JAM/KIL/CRS). Use at own risk -- do not work on CMOS successors.

### CMD 65816 / Mitsubishi MELPS-7700

**CPU names:** `65816`, `MELPS7700`

- **Address spaces:** CODE 16M
- **Banking:** 256 banks of 64K. `ASSUME` required for DT (data bank), PG (program bank), DPR (direct page), M flag (8/16-bit accumulator), X flag (8/16-bit index).
- **Address length override:** Prefix `<` (1 byte), `>` (2 bytes), `>>` (3 bytes) before address.
- **Differences between 65816 and MELPS-7700:**
  - 65816 lacks B accumulator, multiply/divide, SEB, CLB, BBC, BBS, CLM, SEM, PSH, PUL, LDM.
  - 65816 has TSB, TRB, BIT, CLD, SED, XBA, XCE, STZ instead.
  - Several instructions have same function but different names (e.g. REP/CLP, TCS/TAS, PHB/PHT).
  - `PHB`, `PLB`, `TSB` have totally different encoding and meaning on each processor!
- **PSH/PUL:** Accept register name lists as arguments (e.g. `PSH A,B,X,Y`), not just bitmasks.
- **MVN/MVP block transfer:** Specify full source/dest addresses; AS extracts upper 8 bits. Caveat: MELPS-7700 transfers `n` bytes; 65816 transfers `n+1`.
- **PER instruction:** With `#` prefix, operand inserted as-is (Mitsubishi compatible). Without `#`, AS calculates PC-relative displacement.

### Mitsubishi MELPS-740

**CPU names:** `MELPS740`

- **Address spaces:** CODE 64K
- **Special page addressing:** Use `ASSUME` for configuration.
- **Accumulator replacement:** Setting bit 5 of status register (via `SET`) replaces accumulator with memory cell addressed by X for all load/store/arithmetic. No clean syntax support -- use `SET...instructions...CLT`.
- **Gotcha:** Not all MELPS-740 processors implement all instructions. AS does not differentiate among family members.

### Zilog Z80 / Z180 / Z380

**CPU names:** `Z80`, `Z80UNDOC`, `Z180`, `Z380`

- **Address spaces:** CODE 64K/512K/4G, IO 256/256/4G (Z80/Z180/Z380)
- **Z80UNDOC instructions:** Access IX/IY byte halves (IXL, IXU, IYL, IYU) for INC, DEC, LD, arithmetic. `SLIA` (shift left, enter 1 into bit 0). Hidden combined shift+register copy: `SLIA R,(XY+d)`. Combined SET/RES: `SET/RES R,n,(XY+d)`. Hidden I/O: `IN (C)` / `TSTI`, `OUT (C),0`. No guarantee across mask revisions.
- **Z380:** Designed as Z80 grandchild. `EXTMODE` / `LWORDMODE` control 32-bit extensions. `DDIR W` / `DDIR LW` override operand size per instruction. AS auto-merges DDIR prefixes. Do NOT manually use other DDIR variants.
- **Gotcha (Z380):** 32-bit variants currently only available for load/store, not arithmetic.

### Zilog Z8 / Super8 / eZ8

**CPU names:** Z8: `Z8601`, `Z8603`, `Z86C03`, `Z86E03`, `Z86C06`, `Z86E06`, `Z86C08`, `Z86C21`, `Z86E21`, `Z86C30`, `Z86C31`, `Z86C32`, `Z86C40`; Super8: `Z88C00`, `Z88C01`; eZ8: `eZ8`, `Z8F0113`...`Z8F6482` (many variants)

- **Address spaces:** Z8: CODE 64K, DATA 256. eZ8: CODE 64K, DATA 256, XDATA 64K.
- **Work registers:** R0..R15 mapped via RP register to internal address space. `ASSUME RP:value` tells AS the current mapping.
- **Addressing prefixes:** `!` forces 4-bit (work register) addressing. `>` forces 8-bit addressing. `>>` forces 12-bit (eZ8 only).
- **eZ8:** Internal address space has 12 bits. Lower 4 bits of RP provide additional address bits.
- **Super8:** Two register pointers (RP0, RP1) allow mapping upper and lower halves of work registers independently. `ASSUME RP:value` sets RP0=value, RP1=value+8.
- **Differences:** Z8 variants differ only in memory/peripherals. Super8 and eZ8 each have vastly extended (but different) instruction sets and are NOT fully source-compatible.

### Toshiba TLCS-90

**CPU names:** `90C141`

- **Address spaces:** CODE 64K
- **Notes:** Extended Z80; syntax is a subset of TLCS-900 hints. Shifts/increments/decrements by one only, count must not be written. Parentheses required for memory operands of LDA/JP/CALL (orthogonality). Address space principally 1M but AS limits to 64K (upper range only reachable via indirect addressing).

### Toshiba TLCS-870

**CPU names:** `87C00`, `87C20`, `87C40`, `87C70`

- **Address spaces:** CODE 64K
- **Bit addressing:** Uses dot syntax (`A.7`) similar to 8051. AS separates address and bit position by finding the last dot in the expression.
- **Gotcha:** Dots in symbol names create ambiguity. `LD CF,A.7` (accumulator bit 7) vs `LD C,A.7` (constant named "A.7"). Avoid dots in symbol names.
- **Include file:** `STDDEF87.INC` provides correct register definitions per processor type.

### Toshiba TLCS-870/C

Variant of TLCS-870 with separate header byte in code files.

### SGS-Thomson ST6

**CPU names:** `ST6200`, `ST6201`, `ST6203`, `ST6208`, `ST6209`, `ST6210`, `ST6215`, `ST6218`, `ST6220`, `ST6225`, `ST6228`, `ST6230`, `ST6232`, `ST6235`, `ST6240`, `ST6242`, `ST6245`, `ST6246`, `ST6252`, `ST6253`, `ST6255`, `ST6260`, `ST6262`, `ST6263`, `ST6265`, `ST6280`, `ST6285`

- **Address spaces:** CODE 4K, DATA 256
- **ROM paging:** Code ROM mappable pagewise into data area. Use `ASSUME` to configure.
- **Hidden macros:** `CLR A` = `SUB A,A`; `SLA A` = `ADD A,A`; `CLR addr` = `LDI addr,0`; `NOP` = `JRZ PC+1`.
- **Missing instructions:** Has AND but no OR or XOR! `STDDEF62.INC` provides helper macros.
- **AST6 compatibility:** Most pseudo-ops identical or have direct equivalents (see Table 4.8 in manual).

### SGS-Thomson ST7 / STM8

**CPU names:** `ST7` plus many ST72xxx variants; `STM8` plus many STM8Sxxx, STM8Lxxx, STM8AFxxx, STM8ALxxx, STM8TLxxx variants

- **Address spaces:** ST7: CODE 64K. STM8: CODE 16M.
- **`.w` postfix:** Defined by ST for memory indirect operands to mark 16-bit addresses. AS additionally allows it for absolute addresses and displacements to force 16-bit encoding.
- **STM8:** Extends address space to 16M, adds new instructions. Not binary upward-compatible despite similar machine codes.

### SGS-Thomson ST9

**CPU names:** `ST9020`, `ST9030`, `ST9040`, `ST9050`

- **Address spaces:** CODE 64K, DATA 64K, REG 256
- **Bit addressing:** Only within current working register set (except BTSET). Format: `rn.[!]b` where `!` optionally complements source. Symbolic bits store register number, bit position, and complement flag.
- **Gotcha:** Symbolic bit names must NOT contain dots. Bit definitions only work after setting working register bank to peripheral register addresses.
- **Include file:** `REGST9.INC` defines on-chip registers. Defines as general registers (R...) only, not working registers (r...).

### NEC uPD78(C)1x

**CPU names:** `7810`, `78C10`

- **Address spaces:** CODE 64K
- **Branch instructions:** `JR` (short, -32..+31), `JRE` (medium, -256..+255), `J` (pseudo-instruction, auto-selects shortest).
- **Alternative register notation:** `BC`/`(B)`/`(BC)` all accepted for `B`; similarly for DE, HL, with auto-increment/decrement variants (`DE+`, `(DE)+`, `HL-`, etc.) and indexed forms (`HL+A`, `A+HL`, etc.).
- **Gotcha:** NMOS version (7810) has no stop mode; ZCM register omitted. NMOS and CMOS versions have different reset values for some registers.

### NEC 75K0

**CPU names:** `75402`, `75004`, `75006`, `75008`, `75268`, `75304`, `75306`, `75308`, `75312`, `75316`, `75328`, `75104`, `75106`, `75108`, `75112`, `75116`, `75206`, `75208`, `75212`, `75216`, `75512`, `75516`

- **Address spaces:** CODE 16K, DATA 4K
- **Pseudo bit operands:** `BIT` instruction creates combined address+bit symbols. `SFR` for register addresses. AS distinguishes explicit (`addr.bit`) and symbolic bit accesses by the missing dot.
- **Short/long bit format:** Short format covers FBxH-FFxH direct, indirect via @L or @H+d4. Long format covers full address space with correct MBS/MBE banking.
- **Gotcha:** Dots forbidden in symbol names to avoid parser ambiguity.

### NEC uPD75xx (Instruction Set B)

**CPU names:** `7566` (instruction set B), `7508` (instruction set A, "full")

- **Address spaces:** 7566: CODE 1K, DATA 64. 7508: CODE 4K, DATA 256, IO 16.
- **Gotcha:** Instruction sets A and B are NOT 100% binary compatible.

### NEC 78K0

**CPU names:** `78070`

- **Address spaces:** CODE 64K
- **Address prefixes:** `!` forces absolute long, `$` forces PC-relative. Without prefix, AS auto-selects shortest variant.
- **Gotcha with RELAXED mode:** Motorola-syntax hex constants with leading `$` may be misunderstood as relative addressing; use parentheses.

### NEC 78K2 / 78K3 / 78K4

**CPU names:** `78214` (78K2), `78310` (78K3), `784026` (78K4)

- **Address spaces:** 78K2: CODE 1M. 78K3: CODE 64K. 78K4: CODE 16M (program code limited to 1M).
- **Auto address selection:** Automatic short/long selection for RAM and SFR areas. Only `BR` requires explicit relative addressing selection.
- **Same RELAXED/hex gotcha as 78K0.**

### National Semiconductor COP8

**CPU names:** `COP87L84`

- **Address spaces:** CODE 8K, DATA 256
- **Notes:** Uses C syntax for constants by default (IBM mainframe syntax not supported). Only one family member currently supported.

### National Semiconductor SC/MP

**CPU names:** `SC/MP`

- **Address spaces:** CODE 64K
- **Notes:** C syntax for constants by default.

### National Semiconductor INS807x

**CPU names:** `8070`

- **Address spaces:** CODE 64K
- **Notes:** Represents entire 807x family (8070, 8072, 8073) which shares identical CPU core.

### Fairchild F8

**CPU names:** `MK3870`, `MK3872`, `MK3873`, `MK3874`, `MK3875`, `MK3876`, `MK38C70`

- **Address spaces:** CODE 4K (initial 800h/C00h), DATA 64, IO 256
- **Notes:** CMOS version (MK38C70) adds HET and HAL instructions. Variants differ by built-in memory (2-4K ROM, optional 64 bytes executable RAM) and peripherals.

### Fairchild ACE

**CPU names:** `ACE1101`, `ACE1202`

- **Address spaces:** CODE 4K

### Signetics 2650

**CPU names:** `2650`

- **Address spaces:** CODE 8K

### Philips XA

**CPU names:** `XAG1`, `XAG2`, `XAG3`

- **Address spaces:** CODE 16M, DATA 16M, BITDATA 2K (initial 400h)
- **Bit addressing:** One-dimensional bit addresses. Bits 256-511 refer to memory cells 20h-3fh in current data segment (ambiguous depending on state). `BIT` instruction defines individual bits. For RAM bits, bank address is stored and AS checks `ASSUME DS:` value.
- **Auto-alignment:** AS inserts NOPs before branch targets to align to even addresses.
- **No separate BIT segment** due to address ambiguity.

### Parallax SX20/28

**CPU names:** `SX20`, `SX28`

- **Address spaces:** CODE 2K, DATA 256
- **ASSUME:** Inform assembler about STATUS register PA bits and FSR upper bits. Warnings issued for inaccessible addresses.
- **SX20:** Smaller housing, lacks port C.

### Atmel AVR

**CPU names:** Hundreds of variants including `AT90S1200`, `AT90S2313`, `AT90S4414`, `AT90S8515`, all ATtiny variants (`ATTINY4`..`ATTINY4313`), all ATmega variants (`ATMEGA8`..`ATMEGA2561`), AT90USB variants, AT43USB355

- **Address spaces:** CODE 128K (in words, or bytes with `codesegsize=0`), DATA 32K, IO 64, EEDATA up to 8K (varies by chip)
- **Hex format:** Intel format by default (not C syntax). Use `RELAXED ON` for C-style.
- **Code segment organisation:** Default is 16-bit words (Atmel standard). Optional `CPU atmega8:codesegsize=0` switches to byte organisation (useful for LPM; other assemblers use this). PADDING enabled by default in byte mode.
- **Object files:** Can generate `.obj` files for WAVRSIM simulator. File names truncated to ~20 chars.
- **WRAPMODE:** Addresses that exceed the physical address space wrap around (used for JMP/CALL on smaller devices).
- **Missing hardware instructions:** `MUL`, `JMP`, `CALL` defined by architecture but not implemented in all chips. ATmega adds these; ATtiny lacks multiplication.

### Intersil CDP 1802 / 1804 / 1805 / 1806

**CPU names:** `1802`, `1804`, `1805`, `1806`, `1804A`, `1805A`, `1806A`

- **Address spaces:** CODE 64K, IO 8
- **Progression:** 1804/1805/1806 add slightly enhanced instruction set, on-chip RAM, timer. The A variants add `DSAV`, `DBNZ`, and BCD arithmetic instructions.

---

## 16-bit Processors

### Motorola 68000 / ColdFire

**CPU names:** `68008`, `68000`, `68010`, `68012`, `68020`, `68030`, `68040`, `68332`, `68340`, `68360`, `MCF5202`, `MCF5204`, `MCF5206`, `MCF5208`, `MCF52274`, `MCF52277`, `MCF5307`, `MCF5329`, `MCF5373`, `MCF5407`, `MCF5470`..`MCF5475`, `MCF51QM`

- **Address spaces:** CODE 4G
- **Pseudo-ops:** `SUPMODE`, `FPU`, `PMMU`, `FULLPMMU` control access to privileged/FPU/MMU instruction subsets.
- **68030 MMU:** Reduced subset of 68851. Use `FULLPMMU ON` if external 68851 is present.
- **Progression:** Each step adds instructions/addressing modes. 68030 removes `CALLM`/`RTM`. 683xx has reduced 68020 core with different peripherals. ColdFire is downwardly binary compatible RISC variant. 68040 adds on-chip MMU/cache control registers.
- **Default processor** is 68008.

### Motorola M*Core

**CPU names:** `MCORE`

- **Address spaces:** CODE 4G
- **Pseudo-ops:** `SUPMODE` for privileged instructions.

### Freescale XGATE

**CPU names:** `XGATE`

- **Address spaces:** CODE 64K

### Motorola 68HC16

**CPU names:** `68HC16`

- **Address spaces:** CODE 1M

### Siemens 80C166 / 80C167

**CPU names:** `80C166`, `80C167`, `80C165`, `80C163`

- **Address spaces:** 80C166: CODE 256K. 80C167/165/163: CODE 16M.
- **DPP registers:** `ASSUME` required for data paging. Only affects direct data addressing, NOT indirect/indexed.
- **Generic branches:** `JMP` and `CALL` auto-select shortest variant. `JMPS`/`CALLS` accept linear addresses (AS splits segment:offset automatically).
- **Pipeline hazard:** Modifying CP, SP, or paging registers -- value not available for next instruction. AS warns on direct access violations.
- **BIT instruction:** Stores 12-bit word (address in bits 4..11, bit position in LSBs). Incrementing/decrementing works for `BIT`-defined symbols but NOT for explicit specifications crossing word boundaries.
- **Extended SFR area (80C167+):** `EXTR #n` toggles to alternate SFR space for next n instructions (0<n<5). AS tracks this and enforces correct access.
- **Prefix instructions:** `EXTS`/`EXTSR` (64K bank), `EXTP`/`EXTPR` (16K page) -- AS checks prefix vs address consistency when both are constant.

### Panafacom MN1610 / MN1613

**CPU names:** `MN1610`, `MN1613`, `MN1610ALT`, `MN1613ALT`

- **Address spaces:** MN1610: CODE 64K, IO 64K. MN1613: CODE 256K, IO 64K.
- **Two code generators:** Original (by Haruo Asano) via `MN1610`/`MN1613`. Alternative (by AS author) via `MN1610ALT`/`MN1613ALT`. Use ALT for MN1613 extended address space (256 KWords) or floating point format experiments.

### TI TMS9900 Family

**CPU names:** `TI990/4`, `TI990/10`, `TI990/12`, `TMS9900`, `TMS9940`, `TMS9995`, `TMS99105`, `TMS99110`

- **Address spaces:** CODE 64K
- **Hex format:** Intel syntax (not TI's proprietary `>` prefix).
- **Register notation:** Simple integers 0-15 or `R0`..`R15` / `WR0`..`WR15` (both accepted). Register aliases via `EQU`.
- **`@` for absolute addressing:** Cannot be omitted (unlike some other targets) because bare numbers mean register addresses.
- **Progression:** TMS99xx are single-chip implementations of TI990 minicomputers. TI990/12 has the largest instruction set. `SUPMODE` controls privileged mode.

### Mitsubishi M16

**CPU names:** `M16`

- **Address spaces:** CODE 4G
- **Operand sizes:** Both operands may have different sizes. Size appended to operands (`.b`, `.w`, `.l`) or to instruction (as default). `w` = 32 bits (not 16, unlike 68000!).
- **Chained addressing modes:** AS auto-assigns address components. Override displacement lengths with `:4`, `:16`, `:32`.

### Mitsubishi M16C

**CPU names:** `M16C`

- **Address spaces:** CODE 1M

### Fujitsu F2MC8L

**CPU names:** `MB89190`

- **Address spaces:** CODE 64K

### Fujitsu F2MC16L

**CPU names:** `MB9500`

- **Address spaces:** CODE 16M
- **ASSUME:** Required for DTB, DPR, ADB, SSB, USB bank registers if using >64K. AS checks direct memory accesses against current bank register values.
- **Segment prefixes:** AS tracks segment prefix instructions and toggles bank register assumptions for the next machine instruction.

### MSP430 / MSP430X

**CPU names:** `MSP430`, `MSP430X`

- **Address spaces:** CODE 64K (MSP430), 1M (MSP430X)
- **Instruction emulation:** Many "missing" RISC instructions emulated via other instructions (e.g. `RLC @R6+` assembled as `ADDC @R6+,-2(R6)`). Previously in macros in `REGMSP.INC`, now built into assembler core.
- **MSP430X:** Extends address space to 1 MiB, adds instruction set extensions including repeat prefixes.

### MIL STD 1750

**CPU names:** `1750`

- **Address spaces:** CODE 64K
- **Notes:** Standard specification, therefore only one variant.

---

## 32-bit Processors

### PowerPC

**CPU names:** `PPC403`, `MPPC403`, `MPC505`, `MPC601`, `MPC821`, `RS6000`

- **Address spaces:** CODE 4G
- **Notes:** PPC403 lacks FPU (floating point disabled), adds microcontroller-specific instructions. PPC403GC adds MMU instructions. MPC505 differs from 601 mainly in peripheral registers. RS6000 knows additional instructions and uses IBM-style mnemonics (370 mainframe heritage).
- **Incomplete support:** Intel-style mnemonics for data storage used temporarily. Uncommon RS/6000 machine instructions from [70] may be missing.

### Intel i960

**CPU names:** `80960`

- **Address spaces:** CODE 4G
- **Pseudo-ops:** `SUPMODE`, `FPU` for privileged/FPU instruction control.

### AMD 29K

**CPU names:** `AM29245`, `AM29243`, `AM29240`, `AM29000`

- **Address spaces:** CODE 4G
- **Progression:** AM29245 has most emulated instructions (no hardware multiplier). AM29243/AM29240 lack only FPU. AM29000 is "generic" with all instructions in hardware.
- **ASSUME RBP:** AS uses RBP register info to detect privileged register accesses in user mode. Limited to direct accesses; NOT checked for local registers (>127) since those are SP-relative.

### Hitachi H16

**CPU names:** `HD641016`

- **Address spaces:** CODE 16M
- **CISC:** Complex addressing modes, extremely variable instruction length. AS auto-selects shortest format. Override with `:q`, `:g` format suffixes.
- **Optimisations:**
  - `MOV R0,<ea>` optimised to `MOVF <ea>` (disable with explicit format).
  - `SUB` lacks Q format but may be replaced by `ADD:Q` with negated argument (-127..+128).
- **Accumulator bit:** Second operand of two-operand instructions can be omitted if destination is R0. No override available.
- **Operand sizes:** `.b`, `.w`, `.l` or `:8`, `:16`, `:32`. Default for displacements/addresses is shortest form, exploiting that uppermost 8 bits are not output ($FFFF80 = single byte $80).

### Hitachi SH7000 / SH7600 / SH7700

**CPU names:** `SH7000`, `SH7600`, `SH7700`

- **Address spaces:** CODE 4G
- **Hex format:** Motorola syntax (not Hitachi's own format).
- **Literals and LTORG:** Constants that cannot be loaded immediately (outside -128..+127) are collected and dumped at `LTORG` statements. Labels generated as `LITERAL_s_xxxx_n`.
  - Must place `LTORG` before ending a section.
  - PC-relative addressing only allows positive offsets; previously placed literals cannot be referenced by following code.
  - Displacement limited to 512/1024 bytes.
  - No auto-`LTORG` at program end -- unplaced literals produce errors.
  - Literal directly after instruction is unreachable (PC = instruction + 4).
  - Delayed branches: PC-relative uses branch target + 2, not current position. After JMP/JSR/BRAF/BSRF, PC-relative effectively disabled.
- **Operand size semantics:** `.b`/`.w` = shortest instruction putting value in lower 8/16 bits (upper bits don't care). `.l` or no size = full 32-bit value required.
- **PC-relative addressing:** Write as absolute address (`mov.l Var,r8`); AS calculates displacement. Explicit `@(Var,PC)` syntax inserts operand as-is (no conversion).
- **Progression:** SH7600 adds delayed conditional/relative/indirect jumps, 32-bit multiply, multiply-add. SH7700 (SH3) adds second register bank, better shifts, cache control.

### Hitachi H8/300 / H8/300H

**CPU names:** `HD6413308` (H8/300), `HD6413309` (H8/300H)

- **Address spaces:** H8/300: CODE 64K. H8/300H: CODE 16M.
- **Hex format:** Motorola syntax (not Hitachi's format with unbalanced single quotes).
- **H8/300H:** Larger address space, 32-bit registers, additional instructions/addressing modes. Binary upward compatible.

### Hitachi H8/500

**CPU names:** `HD6475328`, `HD6475348`, `HD6475368`, `HD6475388`

- **Address spaces:** Min mode: CODE 64K. Max mode: CODE 16M.
- **Notes:** All share same CPU core; types differ only for correct register subset in `REG53X.INC`.

### Symbios Logic SYM53C8xx

**CPU names:** `SYM53C810`, `SYM53C860`, `SYM53C815`, `SYM53C825`, `SYM53C875`, `SYM53C895`

- **Address spaces:** CODE 4G
- **Notes:** SCSI processors. Simpler members lack some instruction variants and differ in internal registers.

### XMOS XS1 (XCore)

**CPU names:** `XS1`

- **Address spaces:** CODE 4G

---

## 4-bit Microcontrollers

### Intel 4004 / 4040

**CPU names:** `4004`, `4040`

- **Address spaces:** CODE 4K, DATA 256
- **Register pairs:** Written as `RnRm` where n is even (0-E), m = n+1.
- **4040:** Adds about a dozen additional machine instructions.

### Hitachi HMCS400

**CPU names:** `HD614023`, `HD614043`, `HD614081`

- **Address spaces:** CODE 2K/4K/8K, DATA 160/256/512, IO 16
- **Meta instructions:** AS provides a modern alternative notation (Zilog Z80-style) alongside original Hitachi syntax:
  - `LD src,dest` replaces LAI, LBI, LMID, LAB, LBA, LAY, LAM, etc.
  - `XCH src,dest` replaces XMRA, XSPX, XMA, XMB, etc.
  - `ADD`/`ADC`/`SUB`/`SBC`/`OR`/`AND`/`EOR` replace corresponding specific instructions.
  - `CP cond,src,dest` replaces INEM, ANEM, BNEM, YNEI, etc.
  - `BSET`/`BCLR`/`BTST` replace SEC/SEM/REC/REM/TC/TM, etc.
- **Operand types:** A, B, X, Y, W, SPX, SPY (registers), M (memory via X/Y/W), M+/M- (with auto-inc/dec), #val (immediate), addr10 (direct), MRn (memory register 0..15), NE/LE/CA (conditions), bitpos 0..3.

### OKI OLMS-40

**CPU names:** `MSM5840`, `MSM5842`, `MSM58421`, `MSM58422`, `MSM5847`

- **Address spaces:** CODE 768-2K, DATA 32-128 (varies by chip)
- **Meta instructions (like HMCS400):**
  - `LD dest,src` replaces LAI, LLI, LHI, L, LAL, LLA, LAW, LAX, LAY, LAZ, etc.
  - `DEC`/`INC` replace DCA/DCL/DCM.../INA/INL/INM...
  - `BSET`/`BCLR`/`BTST` replace SPB/SMB/SC / RPB/RMB/RC / TAB/TMB/TC.
- **Operand types:** A, W, X, Y, Z, DPL, DPH (registers), T/TL/TH (timer), (DP)/M (memory), #val, PP (port pointer), C (carry), bitpos 0..3.

### OKI OLMS-50

**CPU names:** `MSM5054`, `MSM5055`, `MSM5056`, `MSM6051`, `MSM6052`

- **Address spaces:** CODE 1K-2.5K, DATA 62-119 (varies by chip)
- **Banking:** Data memory up to 128 nibbles but most instructions only hold 4 address bits. P register provides upper bits. Use `ASSUME P:<value>` after `PAGE` instructions.
- **Name conflicts:** `PAGE` and `SWITCH` are machine instructions on these controllers. Use `PAGESIZE` for listing page size and `SELECT` for SWITCH/CASE constructs.

### Mitsubishi MELPS-4500

**CPU names:** `MELPS4500`

- **Address spaces:** CODE 8K, DATA 416
- **Page organisation:** Program memory in 128-word pages. "Long" branches use separate page:offset arguments in standard Mitsubishi syntax. AS also accepts linear addresses (e.g. `BL $1234` instead of `BL $24,$34`).

### Toshiba TLCS-47

**CPU names:** `47C00`, `470C00`, `470AC00`

- **Address spaces:** CODE 64K, DATA 1K, IO 16
- **Data segment:** Reserves nibbles, not bytes (4-bit processor).
- **ASSUME:** Required for data bank register.
- **Extended instruction set:** Partially in `STDDEF47.INC` (macros), partially built-in:
  - `B` -- auto-selects optimal jump variant (BSS, BS, or BSL).
  - `LD HL,#imm` -- immediate load to HL.
  - `ROLC`/`RORC` with shift count > 1.
- **Variants:** Differ in on-chip RAM/ROM size; some bank-switching instructions added or suppressed accordingly.

### National Semiconductor COP4

**CPU names:** `COP410`, `COP420`, `COP440`, `COP444`

- **Address spaces:** CODE 512
- **Progression:** COP42x adds instructions and extends operand ranges.

### TI TMS1000

**CPU names:** `TMS1000`, `TMS1100`, `TMS1200`, `TMS1300`

- **Address spaces:** TMS1000/1200: CODE 1K, DATA 64. TMS1100/1300: CODE 2K, DATA 128.
- **Gotchas:**
  - Instruction set partially defined via ROM mask; AS only knows default codings. Custom instruction sets require macros + `DB`.
  - Branches/calls contain only lower 6 bits of target. Upper bits from page/chapter registers -- AS cannot check correctness.
  - `CALLL`/`BL` pseudo-instructions combine LDP + CALL/BR for same-chapter jumps (convenient but inefficient).
- **TMS1100/1300:** Significantly different default instruction set from TMS1000/1200.

---

## Digital Signal Processors (DSP)

### Motorola DSP56000 / DSP56300

**CPU names:** `56000`, `56002`, `56300`

- **Address spaces:** DSP56000: CODE 64K, XDATA 64K, YDATA 64K. DSP56300: CODE 16M, XDATA 16M, YDATA 16M.
- **Parallel data transfers:** Separated by SPACES (Motorola's design decision). Using spaces for readability in non-parallel code will cause misparse. Use TABS to separate parallel operations; commas separate parts within each operation.
- **Gotcha:** `MOVEC`, `MOVEM`, `ANDI`, `ORI` cannot be replaced by generic `MOVE`/`AND`/`OR` mnemonics (unlike what [65] states). AS does not support this.
- **56002:** Only adds accumulator increment/decrement instructions.
- **56300:** Almost a new processor -- all address spaces enlarged to 16M, instruction count nearly doubled.

### TI TMS32010 / TMS32015

**CPU names:** `32010`, `32015`

- **Address spaces:** CODE 4K, DATA 144 (32010) or 256 (32015), IO 8
- **Hex format:** Intel notation (not TI's `>` prefix which conflicts with AS operators).
- **Banking:** Data space split into two banks of 128 words. AS treats as linear 256 words, auto-clears bit 7 on direct access (except `SST` which writes upper bank only).
- **Gotcha:** `SUBC` internally takes multiple clocks but control unit continues. Next instruction must NOT access the accumulator. AS does not check this.

### TI TMS320C2x

**CPU names:** `320C25`, `320C26`, `320C28`

- **Address spaces:** CODE 64K, DATA 64K, IO 16
- **Untyped labels:** BSS, STRING, RSTRING, BYTE, WORD, LONG, FLOAT, DOUBLE, EFLOAT, BFLOAT, TFLOAT force prefixed labels to be untyped (no segment assignment) to avoid "symbol out of wrong segment" warnings when constants in code space are accessed as data.
- **Variants:** Only differ slightly in peripherals and configuration instructions.

### TI TMS320C20x / C5x / C54x

**CPU names:** `320C203` (C20x family), `320C50`, `320C51`, `320C53` (C5x), `320C541` (C54x)

- **Address spaces:** CODE 64K, DATA 64K, IO 64K
- **C20x:** Subset of C5x instruction set.
- **C54x:** Separate family represented by 320C541.

### TI TMS320C3x / C4x

**CPU names:** `320C30`, `320C31`, `320C40`, `320C44`

- **Address spaces:** C3x: CODE 16M. C40: CODE 4G. C44: CODE 32M.
- **Parallel instructions:** Split across two source lines. AS generates code for first instruction, replaces with parallel encoding when second line parsed. Shown in listing as address not advancing + `R` marker.
- **`||` placement:** Must be in first column (like a label) or prepended to second mnemonic. Other positions confuse the line parser.
- **C31:** Reduced peripherals, same instruction set as C30.
- **C4x:** Source upward compatible but some machine codes differ. C44 is stripped-down C40.

### TI TMS320C6x

**CPU names:** (TMS320C6x family -- specific CPU names not detailed in manual sections)

- **Address spaces:** Listed under separate header in code file format.
- **Notes:** Supported but no dedicated Section 4 processor-specific hints in the manual.

### NEC uPD7720 / uPD7725

**CPU names:** `7720`, `7725`

- **Address spaces:** 7720: CODE 512, DATA 128, ROMDATA 512. 7725: CODE 2K, DATA 256, ROMDATA 1024.
- **ROMDATA segment:** Constant ROM mapped to ROMDATA segment.
- **Gotcha:** NOT binary compatible! Bit positions changed in instruction word between 7720 and 7725. Instruction length changed from 23 to 24 bits. Different code format header IDs.

### NEC uPD77230

**CPU names:** `77230`

- **Address spaces:** CODE 8K, XDATA 512, YDATA 512, ROMDATA 1K

### Toshiba TC9331

**CPU names:** `TC9331`

- **Address spaces:** CODE 320
- **Differences from Toshiba's ASM31T assembler:**
  - No C-like multi-line comments (`/* ... */`).
  - No comma in parenthesised comments.
  - No dashes in symbol/label names.
  - `END` statement optional (required by ASM31T).
- **Gotcha:** AS currently cannot detect conflicting functional unit usage within a single instruction.

---

## Miscellaneous / Specialised

### Toshiba TLCS-900 / TLCS-900L

**CPU names:** `96C141` (TLCS-900), `93C141` (TLCS-900L)

- **Address spaces:** CODE 16M
- **Operating modes:** Minimum mode (Z80/TLCS-90 compatible, 16-bit registers, 8 banks, 64K code) vs Maximum mode (full capabilities, 32-bit registers, 4 banks, 16M code). Set via `MAXMODE ON/OFF` (default OFF).
- **Register usage:** Min mode: WA, BC, DE, HL (16-bit). Max mode: XWA, XBC, XDE, XHL (32-bit). XIX..XIZ, XSP always 32-bit.
- **Address/displacement lengths:** Auto-selects shortest. Override with `:8`, `:16`, `:24` suffix. Zero displacement eliminated (e.g. `(XIX+0)` becomes `(XIX)`).
- **Previous bank register:** Single quote (`'`) conflicts with AS character constants. Use inverse quote (`` ` ``) instead: `ld wa`,wa`.
- **TLCS-900L differences:**
  - `MAX`/`NORMAL` instructions not allowed (L version starts in max mode, has no normal mode).
  - No normal stack pointer XNSP/NSP; has INTNEST instead.
  - Warnings for privileged instructions suppressed.
- **TAS900 compatibility differences:** See manual for detailed comparison of symbol conventions, syntax extensions (parentheses for LDA/JP/CALL, EX operand swapping, optional shift counts/increment values), macro processor differences, output format (relocatable vs absolute), and pseudo-instruction equivalence table.

### Toshiba TLCS-9000

**CPU names:** `97C241`

- **Address spaces:** CODE 16M
- **"Paper design":** Originally implemented before the processor was available. Toshiba reportedly did sell chips, but documentation was preliminary and unclear. Errors possible.
- **Displacement prefixes:** Long displacements require prefix instructions (auto-inserted by AS). Force prefix with leading `>`. Example: `ld:g.b (>0h),0` forces prefix even for zero displacement.

### Xilinx KCPSM (PicoBlaze)

**CPU names:** `KCPSM` (original), `KCPSM3` (v3)

- **Address spaces:** KCPSM: CODE 256, DATA 256. KCPSM3: CODE 256, DATA 64, IO 256.
- **Logic cores** for Xilinx FPGAs, not standalone chips.
- **Gotcha:** KCPSM3 is NOT binary upward-compatible. Has larger address space and additional instructions.

### Lattice Mico8

**CPU names:** `MICO8_05`, `MICO8_V3`, `MICO8_V31`

- **Address spaces:** CODE 4096, DATA 256, IO 256
- **Multiple versions:** Lattice changed machine instructions more than once. `MICO8_05` = 2005 manual; `V3` = version 3.0; `V31` = version 3.1. Separate targets needed for continued support of older projects.

### National Semiconductor SC144xx

**CPU names:** `SC14400`, `SC14401`, `SC14402`, `SC14404`, `SC14405`, `SC14420`, `SC14421`, `SC14422`, `SC14424`

- **DECT controllers.** Differentiated by instruction count (different B field formats) and architecture optimisations over time.
- **IAR assembler compatibility:** Detailed equivalence table in manual. Identical: ALIGN, END, ENDM, EXITM, MACRO, ORG, RADIX, SET, REPT. Many others have AS equivalents (see Table in Section 4.52).
- **SC144xx-specific pseudo-ops:** `DC`, `DC8`, `DW16`, `DS`, `DS8`, `DS16`, `DW`.
- **No direct equivalent** for CASEON, CASEOFF, LOCAL, LSTPAG, #undef, REPTI.

### TI TMS7000 Family

**CPU names:** `TMS70C00`, `TMS70C20`, `TMS70C40`, `TMS70CT20`, `TMS70CT40`, `TMS70C02`, `TMS70C42`, `TMS70C82`, `TMS70C08`, `TMS70C48`

- **Address spaces:** CODE 64K
- **Hex format:** Intel syntax (not TI's proprietary `<` / `?` prefixes).
- **Syntax relaxations (vs original TI syntax):**
  - `#` accepted for immediate (instead of `%`).
  - Port-addressed instructions (ANDP, MOVP, etc.) accept generic mnemonic without trailing P.
  - `@` prefix for absolute addressing may be omitted.
  - `CMPA` = `CMP A`; `LDA`/`STA` = `MOV` with A; `MOVW` = `MOVD`; `RETS`/`RETI` = `RTS`/`RTI`.
  - `TSTA`/`TSTB` = `TST A`/`TST B`; `XCHB B` = `TSTB`.
- **Gotcha:** These relaxed variants are only allowed for TMS70Cxx, NOT for TMS370 successors.

### TI TMS370xxx

**CPU names:** `370C010`, `370C020`, `370C030`, `370C040`, `370C050`

- **Address spaces:** CODE 64K
- **Pseudo bit manipulation:** `DBIT` creates combined address+bit symbols. Pseudo-instructions `SBIT0`, `SBIT1`, `CMPBIT`, `JBIT0`, `JBIT1` translate to OR, AND, XOR, BTJZ, BTJO with appropriate bitmask.
- **Bit symbol format:** Lower half = address, upper half = bit position. Constructable manually but `DBIT` provides EQU-style syntax.
- **Gotcha:** `DBIT` accepts arbitrary addresses but pseudo-instructions only work with addresses 0..255 or 1000h..10FFh (no absolute addressing mode for other ranges).

### Signetics 8X300 / 8X305

**CPU names:** `8X300`, `8X305`

- **Address spaces:** CODE 8K
- **LIV / RIV instructions:** Define symbolic bit group objects with 3 parameters: peripheral address (0..255), first bit number (0..7), group length (1..8 bits).
- **Gotcha:** Bit groups must NOT span multiple memory addresses. AS does not validate this -- produces strange runtime results.
- **Length handling:** AS auto-assigns length from symbolic objects. Explicit third operand checked against symbolic length. `MOVE` with two symbolic operands of different lengths produces error (single length field).
- **8X305 extras:** Additional registers and new operations (e.g. direct 8-bit writes to peripheral addresses).
- **Pseudo instructions (from MCCAP):** `NOP` = `MOVE AUX,AUX`; `HALT` = `JMP *`; `XML ii` = `XMIT ii,R12` (8X305 only); `XMR ii` = `XMIT ii,R13` (8X305 only); `SEL <busobj>` = `XMIT <adr>,IVL/IVR`.

### Kenbak-1

Listed in code file format headers as a supported target. Minimal documentation in manual.

---

## Address Space / Segment Summary

AS supports the following segment types (not all available for every target):

| Segment | Description |
|---------|-------------|
| `CODE` | Program code (supported by ALL targets) |
| `DATA` | Data memory |
| `IDATA` | Internal/indirect data (MCS-48, MCS-51) |
| `XDATA` | External data (MCS-51, DSP56xxx, XA, uPD77230) |
| `YDATA` | Y data memory (DSP56xxx, uPD77230) |
| `BITDATA` | Bit-addressable data (MCS-51, XA) |
| `IO` | I/O port space (8080, MCS-251, 8086, various) |
| `REG` | Register space (ST9, MN161x) |
| `ROMDATA` | ROM data (uPD7720/7725/77230) |
| `EEDATA` | EEPROM data (AVR, PIC16C8x) |

---

## Universal Pseudo-ops / Features

These apply across most or all targets:

- **`CPU <name>`** -- set target processor (resets to CODE segment)
- **`ORG <addr>`** -- set program counter
- **`SEGMENT <name>`** -- switch address space segment
- **`ASSUME <reg>:<value>`** -- inform assembler of register contents for banking/paging
- **`MAXMODE ON/OFF`** -- set min/max operating mode (TLCS-900, etc.)
- **`SUPMODE ON/OFF`** -- enable/disable privileged instructions
- **`FPU ON/OFF`** -- enable/disable FPU instructions
- **`PMMU ON/OFF`** -- enable/disable MMU instructions
- **`Z80SYNTAX ON/OFF/EXCLUSIVE`** -- enable Z80-style mnemonics for 8080/8085
- **`EXTMODE ON/OFF`** / **`LWORDMODE ON/OFF`** -- Z380 extended/longword modes
- **`WRAPMODE ON/OFF`** -- address wrapping for small-memory targets (AVR)
- **`RELAXED ON/OFF`** -- accept C-style and alternative constant formats
- **`PADDING ON/OFF`** -- automatic padding for alignment
- **`BRANCHEXT ON/OFF`** -- automatic branch extension
- **`LTORG`** -- dump collected literals (SH7xxx)
- **`PRWINS`** -- print current bank window mapping (6811 K4)
- **`EMULATED`** -- mark emulated instructions
