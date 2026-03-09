# AS Macro Assembler -- File Formats & Utility Programs

## Table of Contents

1. [Code File Format](#1-code-file-format)
2. [Debug File Formats](#2-debug-file-formats)
3. [Share Files](#3-share-files)
4. [Utility Programs](#4-utility-programs)
5. [Error Messages](#5-error-messages)

---

## 1. Code File Format

### Structure

Code files use **little-endian** byte order. Every file begins with magic word `$1489`
(bytes `$89 $14`), followed by an arbitrary number of records.

### Record Types

| Header | Type | Description |
|--------|------|-------------|
| `$00` | Creator | Final record. String identifying the creating program |
| `$01`-`$7f` | Legacy code | Header = processor family. Implicit CODE segment |
| `$80` | Entry point | 4-byte program entry address (from `END` with address) |
| `$81` | Multi-target | General record with explicit Header, Segment, Granularity |

### Multi-Target Record (`$81`) Structure

```
$81 | Header(1) | Segment(1) | Gran(1) | StartAdr(4) | Length(2) | Data(Length)
```

- **Header byte:** processor family identifier
- **Segment byte:** address space (see table below)
- **Gran byte:** granularity (smallest addressable unit in bytes)
- **StartAdr:** 32-bit, always present
- **Length:** 16-bit, always in bytes

### Processor Family Header Bytes

| ID | Family | ID | Family |
|----|--------|----|--------|
| `$01` | 680x0/6833x | `$02` | ATARI_VECTOR |
| `$03` | M*Core | `$05` | PowerPC |
| `$09` | DSP56xxx | `$11` | 65xx/MELPS-740 |
| `$19` | 65816/MELPS-7700 | `$21` | MCS-48 |
| `$29` | 29xxx | `$2a` | i960 |
| `$31` | MCS-51 | `$3b` | AVR |
| `$3c` | XA | `$41` | 8080/8085 |
| `$42` | 8086..V35 | `$4a` | MSP430 |
| `$4c` | 80C166/167 | `$51` | Z80/180/380 |
| `$52` | TLCS-900 | `$53` | TLCS-90 |
| `$54` | TLCS-870 | `$55` | TLCS-47 |
| `$61` | 6800/6301/6811 | `$62` | 6805/HC08 |
| `$63` | 6809 | `$66` | 68HC12 |
| `$68` | H8/300(H) | `$69` | H8/500 |
| `$6c` | SH7000 | `$70` | PIC16C8x |
| `$71` | PIC16C5x | `$72` | PIC17C4x |
| `$74` | TMS3201x | `$75` | TMS320C2x |
| `$76` | TMS320C3x/C4x | `$77` | TMS320C20x/C5x |
| `$78` | ST6 | `$79` | Z8 |

(This is a selection; 100+ families are supported. See manual for full table.)

### Segment Codings

| Number | Segment | Number | Segment |
|--------|---------|--------|---------|
| `$00` | (undefined) | `$01` | CODE |
| `$02` | DATA | `$03` | IDATA |
| `$04` | XDATA | `$05` | YDATA |
| `$06` | BDATA | `$07` | IO |
| `$08` | REG | `$09` | ROMDATA |

---

## 2. Debug File Formats

Generated via `-g` switch:

| Format | Switch | Extension | Description |
|--------|--------|-----------|-------------|
| MAP | `-g MAP` | `.MAP` | AS-specific format |
| NoICE | `-g NOICE` | `.NOI` | NoICE-compatible command file |
| Atmel | `-g Atmel` | `.OBJ` | AVR tools object format |

### MAP File Structure

Three sections:

**Section 1: Source-to-Address Mapping**
```
Segment <segment name>
File <file name>
<line>:<address>
```

**Section 2: Symbol Table**
```
Symbols in Segment <segment name>
<name> <type> <value> <size> <used> <constant>
```
Fields: name, type (Int/Float/String), value, size (-1 if unknown),
used (0/1), constant (0=const, 1=variable).

**Section 3: Section Definitions**
```
Info for Section nn ssss pp
<address or range>
```

---

## 3. Share Files

Export symbols via `SHARED` pseudo instruction. Format selected by switch:

| Switch | Format | Extension |
|--------|--------|-----------|
| `-c` | C (`#define`) | `.H` |
| `-p` | Pascal (`CONST`) | `.INC` |
| `-a` | Assembler | `.INC` |

Without any of these switches, `SHARED` is ignored (with warning).

---

## 4. Utility Programs

### Common Conventions

- Read from stdin, write to stdout, errors to stderr
- Hex addresses: append `h`, prepend `$` or `0x`
- Environment: `<NAME>CMD` for default params (e.g. `BINDCMD`, `P2HEXCMD`)
- Default extension: `.P`

### Return Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Command line error |
| 2 | I/O error |
| 3 | File format error |

### PLIST

Lists all records in a code file.

```
PLIST <file>
```

Output: code type, start address, length, end address per record. Plus creator string and total size.

### BIND

Merges multiple code files into one.

```
BIND <source(s)> <target> [options]
```

| Option | Description |
|--------|-------------|
| `-f <Header[,...]>` | Filter: copy only specified processor family records |

### P2HEX

Converts code files to hex formats.

```
P2HEX <source(s)> <target> [options]
```

Minimal: `P2HEX <name>` (reads `<name>.p`, writes `<name>.hex`).

#### Target Formats (`-F`)

| Format | Description |
|--------|-------------|
| `Intel` | Intel Hex (Intellec-8) -- default for most |
| `Intel16` | 16-bit Intel Hex (MCS-86, 20-bit addresses) |
| `Intel32` | 32-bit Intel Hex |
| `Moto` | Motorola S-Records -- default for 68K/Hitachi/TLCS-900 |
| `MOS` | MOS Hex -- default for 65xx |
| `Tek` | Tektronix Hex |
| `DSK` | Texas Instruments DSK -- default for TI 16-bit DSPs |
| `Atmel` | Atmel AVR Generic -- default for AVR |
| `Mico8` | Lattice Mico8 prom_init |
| `C` | C arrays for `#include` |

#### Key Options

| Option | Description |
|--------|-------------|
| `-f <Header[,...]>` | Record filter (from BIND) |
| `-F <format>` | Select format |
| `-M <1\|2\|3>` | Min S-Record type (Motorola) |
| `-m <0..3>` | PIC Intel Hex variant (0=INHX8M, 1=INHX16M, 2=INHX8L, 3=INHX8H) |
| `-r <start>-<end>` | Address window (`$` for min/max) |
| `-a` | Relative addresses (start at 0) |
| `-R <value>` | Offset all addresses |
| `-e <address>` | Set entry address |
| `-l <count>` | Bytes per line (2-254, default 16) |
| `-i <0\|1\|2>` | Intel Hex end-line variant |
| `-k` | Delete source files after conversion |
| `-segment <name>` | Select segment (default: CODE) |
| `-d <start>-<end>` | DSK data range |
| `-cformat <spec>` | C format descriptor |
| `+5` | Suppress S5 record |
| `(<offset>)` | Shift source file contents by offset |

#### C Format Descriptor (`-cformat`)

| Letter | Element |
|--------|---------|
| `d`/`D` | Pointer to data (lower/uppercase hex) |
| `s`/`S` | Start address (unsigned/unsigned long) |
| `l`/`L` | Length (unsigned/unsigned long) |
| `e`/`E` | End address (unsigned/unsigned long) |

### P2BIN

Converts code files to raw binary. All P2HEX options except `-a` and `-i`.

```
P2BIN <source(s)> <target> [options]
```

#### Additional Options

| Option | Description |
|--------|-------------|
| `-l <byte>` | Fill value for unused areas (default `$FF`) |
| `-s` | Calculate 32-bit checksum, store complement in last byte |
| `-m <mode>` | Bus split: `ALL`, `EVEN`, `ODD`, `BYTE0`-`BYTE3`, `WORD0`/`WORD1` |
| `-S <spec>` | Prepend entry address (`1`-`4` bytes, optional `L`/`B` endian) |

---

## 5. Error Messages

### Categories

| Category | Behaviour |
|----------|-----------|
| **Warning** | Possible issue. Code still generated |
| **Error** | Definite error. No code file produced |
| **Fatal** | Immediate termination |

Show error numbers with `-n`. Increase detail with `-x`.

### Common Warnings (5-360)

| Code | Message |
|------|---------|
| 10 | short addressing possible |
| 20 | short jump possible |
| 30 | no sharefile created, SHARED ignored |
| 50 | privileged instruction |
| 70 | symbol out of wrong segment |
| 80 | symbol value change forces additional pass |
| 90 | overlapping memory usage |
| 100 | no CASE condition matched |
| 130 | obsolete instruction |
| 170 | unknown symbol forces additional pass |
| 180 | address not properly aligned |
| 250 | instruction crosses page boundary |
| 260 | range overflow |
| 320 | macro argument redefined |

### Common Errors (1000-2200)

| Code | Message |
|------|---------|
| 1000 | symbol double defined |
| 1010 | symbol undefined |
| 1020 | invalid symbol name |
| 1090 | invalid format |
| 1110 | wrong number of operands |
| 1120 | addressing mode must be immediate |
| 1130 | invalid operand size |
| 1200 | unknown instruction |
| 1300 | brackets mismatch |
| 1310 | division by 0 |
| 1320 | range overflow |
| 1330 | distance too big |
| 1340 | short addressing not allowed |
| 1350 | addressing mode not allowed here |
| 1370 | jump distance too big |
| 1430 | invalid CPU type |
| 1440 | invalid control register |
| 1445 | invalid register |
| 1450 | RESTORE without SAVE |
| 1460 | missing RESTORE |
| 1470 | missing ENDIF/ENDCASE |
| 1480 | invalid IF-structure |
| 1485 | missing ENDSECTION |
| 1490 | wrong number of function arguments |
| 1495 | unresolved literals (missing LTORG) |
| 1500 | instruction not allowed on [CPU] |
| 1550-1556 | STRUCT errors |
| 1800-1815 | macro errors |
| 1820 | expression must be evaluatable in first pass |
| 1860 | unknown function |
| 1900-1925 | address/overflow errors |
| 1940 | code in structure definition |
| 2010 | invalid escape sequence |
| 2030 | constants cannot be redefined as variables |
| 2035 | variables cannot be redefined as constants |

### Fatal Errors (10001-10007)

| Code | Message |
|------|---------|
| 10001 | error opening file |
| 10002 | error writing listing |
| 10003 | file read error |
| 10004 | file write error |
| 10006 | heap overflow |
| 10007 | stack overflow (try `-A` switch) |

### Processor Aliases

Defined via `-alias <new>=<old>`. The alias inherits the full instruction set;
only `MOMCPU`/`MOMCPUNAME` differ. Aliases are user-defined (no built-in table).
Place in a key file loaded via `ASCMD` for automatic availability.
