---
name: as-assembler
description: >
  Reference skill for the AS (Macro Assembler AS) V1.42 multi-target cross assembler
  supporting 60+ processor families. Use this skill whenever working with AS assembler
  source files, writing assembly for any of its supported targets (68K, Z80, 8051, PIC,
  AVR, ARM, DSP56K, TMS320, H8, 6502, MSP430, PowerPC, and many more), debugging AS
  output, using AS utility programs (P2HEX, P2BIN, BIND, PLIST), or converting between
  code file formats. Also use when the user mentions .asm files targeting AS, asks about
  cross-assembler pseudo instructions, macro definitions, conditional assembly, structure
  definitions, or multi-pass assembly behaviour.
---

# AS Macro Assembler V1.42 Reference

AS is a portable macro cross-assembler for a wide range of microprocessors and
microcontrollers. It supports 60+ processor families from a single tool, using a
unified pseudo-instruction set with processor-specific extensions.

**Authors:** Alfred Arnold, Stefan Hilse, Stephan Kanthak, Oliver Sellke, Vittorio De Tomasi

**Manual:** https://ubuntourist.codeberg.page/Altair-8800/_static/as_EN.html

## Quick Start

```bash
# Basic assembly
as source.asm

# With listing and include path
as -L -i /path/to/includes source.asm

# Convert to Intel HEX
p2hex source

# Convert to raw binary
p2bin source
```

## Invocation

```
as [options] <source file(s)>
```

Key options:

| Option | Purpose |
|--------|---------|
| `-L` | Write listing to `.LST` file |
| `-l` | Send listing to console |
| `-cpu <name>` | Set target processor |
| `-i <paths>` | Semicolon-separated include search paths |
| `-D <SYM[=val]>` | Define symbols before assembly |
| `-U` | Case-sensitive mode |
| `-c` / `-p` / `-a` | Share file format (C / Pascal / ASM) |
| `-g [MAP\|NOICE\|Atmel]` | Generate debug info |
| `-w` | Suppress warnings |
| `-relaxed` | Allow mixed number syntaxes |
| `-alias <new>=<old>` | Define processor alias |
| `-x` | Increase error detail (repeatable) |
| `-n` | Include error numbers in messages |
| `-q` | Quiet mode |

Environment: `ASCMD` holds persistent default options. `@keyfile` loads options from a file.

For the complete option reference, read `references/command-line.md`.

## Input File Format

```
[label[:]] <mnemonic>[.attr] [param[,param...]] [;comment]
```

- Labels in column 1 don't need a colon; labels elsewhere require one
- Max line length: 255 characters
- Semicolons start comments; in column 1 = whole-line comment
- Up to 20 parameters per instruction, comma-separated

## Number Systems

Three syntax modes determined by processor family:

| Mode | Hex | Binary | Octal | Families |
|------|-----|--------|-------|----------|
| **Motorola** | `$FF` | `%1010` | `@377` | 68K, 6502, PIC, Hitachi |
| **Intel** | `0FFH` | `1010B` | `377O` | 8051, Z80, 80x86, TMS, NEC |
| **C** | `0xFF` | `0b1010` | `0377` | PowerPC, AVR, AMD29K |

`RELAXED ON` enables cross-syntax: `x'FF'`, `h'FF'`, `b'1010'`, `o'377'`

## Key Pseudo Instructions

### Definitions

| Instruction | Purpose |
|-------------|---------|
| `EQU` / `=` | Define permanent constant |
| `SET` / `:=` / `EVAL` | Define reassignable variable |
| `SFR` / `SFRB` | Special function register (SFRB adds bit symbols) |
| `BIT` | Name a single bit |
| `PORT` | I/O address symbol |
| `REG` | Register symbol |
| `LABEL` | Code-segment label (escapes macro locality) |
| `CHARSET` | Modify character translation table |
| `ENUM` / `NEXTENUM` | Enumeration constants |

### Code Layout

| Instruction | Purpose |
|-------------|---------|
| `ORG` | Set program counter |
| `CPU` | Select target processor |
| `SEGMENT` | Switch address space (CODE, DATA, IDATA, XDATA, etc.) |
| `PHASE` / `DEPHASE` | Set execution address different from load address |
| `ASSUME` | Inform AS about register settings for addressing |
| `SAVE` / `RESTORE` | Push/pop assembler state |
| `ALIGN` | Align to boundary |
| `PADDING ON\|OFF` | Auto-pad to word alignment |
| `END [addr]` | End of program with optional entry point |

### Data

| Instruction | Purpose |
|-------------|---------|
| `DC[.size]` | Motorola-style constants |
| `DS[.size]` | Motorola-style reservation |
| `DB`/`DW`/`DD`/`DQ`/`DT` | Intel-style constants (`?` = reserve, `DUP` = repeat) |
| `DATA` | PIC/AVR/TI style constants |
| `BYT`/`FCB`/`ADR`/`FDB` | 6502/68xx style |
| `BINCLUDE` | Embed binary file data |

### Macros

```asm
name    MACRO   param1,param2[=default]
        ; body using param1, param2
        ENDM
```

- `IRP sym,val1,val2,...` -- iterate over values
- `IRPC sym,"string"` -- iterate over characters
- `REPT count` -- repeat block
- `WHILE expr` -- conditional repeat
- `EXITM` -- early exit
- `SHIFT` -- consume first parameter
- `FUNCTION arg1,arg2,expr` -- user-defined function

### Structures

```asm
MyStruct  STRUCT
field1    DB    ?
field2    DW    ?
MyStruct  ENDSTRUCT
; Creates MyStruct_field1, MyStruct_field2, MyStruct_LEN

inst1     MyStruct          ; instantiate
```

Supports `UNION`/`ENDUNION` for overlapping fields, nesting, and nameless inner structs.

### Conditional Assembly

```asm
IF expr            IFDEF sym          SWITCH expr
  ...              IFNDEF sym         CASE val1[,val2]
ELSEIF expr        IFUSED sym           ...
  ...              IFNUSED sym        CASE val3
ENDIF              IFEXIST "file"       ...
                   IFB / IFNB         ELSECASE
                   ENDIF                ...
                                      ENDCASE
```

### Sections (Local Symbols)

```asm
SECTION MyModule
  ; symbols here are local
  PUBLIC exported_sym        ; make accessible outside
  GLOBAL aliased_sym         ; create prefixed copy outside
  FORWARD local_sym          ; declare explicitly local
ENDSECTION
```

Cross-section reference: `section[symbol]`

## Predefined Symbols

| Symbol | Type | Meaning |
|--------|------|---------|
| `MOMCPU` | Int | Current target CPU numeric ID |
| `MOMCPUNAME` | String | Current target CPU name |
| `MOMFILE` | String | Current source filename |
| `MOMLINE` | Int | Current line number |
| `MOMPASS` | Int | Current pass number |
| `MOMSECTION` | String | Current section name |
| `VERSION` | Int | AS version (BCD) |
| `*`, `$`, `PC` | Int | Current program counter |
| `TRUE` / `FALSE` | Int | 1 / 0 |
| `CONSTPI` | Float | Pi |
| `DATE` / `TIME` | String | Assembly start date/time |

## Expressions

Operators by precedence (highest first): `~` (NOT), `~~` (logical NOT), `<<`/`>>`,
`><` (bit mirror), `&`, `|`, `!` (XOR), `^` (power), `*`/`/`/`#` (mod),
`+`/`-`, `&&`, `||`, `!!`, comparisons (`=`/`<>`/`<`/`>`/`<=`/`>=`).

Key built-in functions: `STRLEN`, `SUBSTR`, `STRSTR`, `UPSTRING`, `LOWSTRING`,
`CHARFROMSTR`, `BITCNT`, `FIRSTBIT`, `LASTBIT`, `BITPOS`, `ABS`, `SGN`,
`SQRT`, `SIN`, `COS`, `TAN`, `LOG`, `LN`, `LD`, `EXP`, `DEFINED`, `SYMTYPE`,
`EXPRTYPE`, `VAL`.

For the complete operator/function reference, read `references/expressions-and-symbols.md`.

## Temporary Symbols

- **Named (`$$`):** `$$loop` -- counter resets at each non-temporary label
- **Nameless:** `-` (backward), `+` (forward), `/` (both); stack with `--`, `++`
- **Composed (`.`):** `.loop` prepends the last non-dotted label (e.g. `proc1.loop`)

## Multi-Pass Assembly

AS uses iterative multi-pass assembly to resolve forward references and generate
optimal code. Unknown symbols get the current PC as a placeholder; passes repeat
until all values stabilise. Complex programs may need 12+ passes.

Best practice: minimise forward references; place `EQU` definitions before use.

## Utility Programs

| Tool | Purpose | Default Extension |
|------|---------|-------------------|
| `PLIST file` | List code file records | `.P` |
| `BIND src... dest` | Merge code files | `.P` |
| `P2HEX src [dest]` | Convert to hex format | `.P` -> `.HEX` |
| `P2BIN src [dest]` | Convert to raw binary | `.P` -> `.BIN` |

P2HEX formats: `-F Intel` (default), `-F Intel16`, `-F Intel32`, `-F Moto` (S-Records),
`-F MOS`, `-F Tek`, `-F DSK`, `-F Atmel`, `-F C`.

For complete utility options and the code file format, read `references/file-formats-and-tools.md`.

## Reference Files

Read these for detailed information on specific topics:

| File | Contents |
|------|----------|
| `references/command-line.md` | All CLI options, env vars, listing format, predefined symbols |
| `references/expressions-and-symbols.md` | Symbol rules, temporaries, operators, all built-in functions, register symbols, forward references |
| `references/pseudo-instructions.md` | Complete pseudo instruction reference by category |
| `references/processor-families.md` | Processor-specific hints for all 60+ supported families |
| `references/file-formats-and-tools.md` | Code file format, debug formats, share files, PLIST/BIND/P2HEX/P2BIN, error codes |
