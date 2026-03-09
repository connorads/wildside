# AS Macro Assembler -- Pseudo Instructions Reference

## Table of Contents

1. [Definition Instructions](#1-definition-instructions)
2. [Code Modification Instructions](#2-code-modification-instructions)
3. [Data Definition Instructions](#3-data-definition-instructions)
4. [Macro Instructions](#4-macro-instructions)
5. [Structures](#5-structures)
6. [Conditional Assembly](#6-conditional-assembly)
7. [Listing Control](#7-listing-control)
8. [Sections (Local Symbols)](#8-sections-local-symbols)
9. [Miscellaneous](#9-miscellaneous)

---

## 1. Definition Instructions

### EQU / SET

```asm
<name>  EQU   <expression>          ; permanent constant
<name>  =     <expression>          ; alias for EQU
<name>  SET   <expression>          ; reassignable variable
<name>  :=    <expression>          ; alias for SET
<name>  EVAL  <expression>          ; alias for SET (when SET is a machine instruction)
```

Typeless (no segment assignment). Optional segment parameter:
`sym EQU value,CODE` assigns to CODE segment.

### SFR / SFRB

```asm
<name>  SFR   <address>             ; DATA segment register
<name>  SFRB  <address>             ; + generates .0-.7 bit symbols (MCS-51)
```

SFRB checks bit-addressability (20h-3Fh, 80h/88h/90h.../F8h).

### XSFR / YSFR (DSP56xxx)

```asm
<name>  XSFR  <address>             ; X-space register
<name>  YSFR  <address>             ; Y-space register
```

### LABEL

```asm
<name>  LABEL  $                    ; code-segment label (global, escapes macro locality)
```

### BIT

```asm
<name>  BIT  <address>.<bitpos>     ; MCS-51, XA, 80C166, AVR, etc.
```

Encoding varies by target. MCS-51: simple integer in BDATA segment.
80C251: 2D encoding. 80C166: 16-bit positions. ST9: supports inversion (`!`).

### DBIT (TMS370)

```asm
<name>  DBIT  <bitpos>,<address>
```

### DEFBIT (S12Z, Super8)

```asm
<name>  DEFBIT[.size]  <address>,<position>
```

Size: `.b` (8-bit, default), `.w`, `.l`. Defined bit "knows" its size.

### DEFBITFIELD (S12Z)

```asm
<name>  DEFBITFIELD[.size]  <address>,<width>:<position>
```

### PORT

```asm
<name>  PORT  <address>             ; I/O address (8080/8086/Z80/AVR/etc.)
```

### REG / NAMEREG

```asm
<name>  REG   <register>            ; register symbol
        NAMEREG <register>,<name>   ; KCPSM alternate form
```

### LIV / RIV (8X30x)

```asm
<name>  LIV  <addr>,<start>,<len>   ; left IV bus object
<name>  RIV  <addr>,<start>,<len>   ; right IV bus object
```

### CHARSET

```asm
CHARSET                              ; reset to 1:1
CHARSET "<filename>"                 ; load 256 bytes from file
CHARSET <start>,<target>             ; modify single entry
CHARSET <start>,<end>,<target>       ; modify range
CHARSET <start>,"<string>"           ; modify from string
```

### CODEPAGE

```asm
CODEPAGE <name>[,<initial-source>]   ; switch/create character set
```

### ENUM / NEXTENUM / ENUMCONF

```asm
ENUM     sym1[=val],sym2,sym3,...    ; define enumeration (starts at 0)
NEXTENUM sym4,sym5,...               ; continue sequence
ENUMCONF <increment>[,<segment>]     ; configure step and segment
```

### PUSHV / POPV

```asm
PUSHV  <stackname>,<sym1>,<sym2>     ; save symbol values
POPV   <stackname>,<sym2>,<sym1>     ; restore (reverse order)
```

Named LIFO stacks. Cleared at start of each pass.

---

## 2. Code Modification Instructions

### ORG / RORG

```asm
ORG   <address>                      ; set absolute program counter
RORG  <offset>                       ; relative adjustment to PC
```

### CPU

```asm
CPU   <processorname>                ; select target processor
```

Changes available instruction set. `MOMCPU`/`MOMCPUNAME` reflect current selection.

### SEGMENT

```asm
SEGMENT  <name>                      ; switch address space
```

Available segments: `CODE`, `DATA`, `IDATA`, `XDATA`, `YDATA`, `BITDATA`, `IO`, `REG`, `ROMDATA`, `EEDATA`.

### PHASE / DEPHASE

```asm
PHASE    <execution-address>         ; code executes at different address
...
DEPHASE                              ; revert
```

May be nested. Labels and PC references use execution address.

### SAVE / RESTORE

```asm
SAVE                                 ; push: CPU, segment, listing, macexp, codepage
...
RESTORE                              ; pop and restore state
```

Must be balanced.

### ASSUME

```asm
ASSUME  <register>:<value>[,...]     ; inform AS about register settings
```

Processor-specific. No code generated. Key examples:
- **8086:** `ASSUME CS:CODE, DS:DATA, ES:NOTHING`
- **80C166:** `ASSUME DPP0:0,DPP1:1,DPP2:2,DPP3:3`
- **6809/65CE02:** `ASSUME B:<value>` (direct page)
- **65816:** `ASSUME DT:0,PG:0,DPR:0,X:1,M:1`
- **320C3x/4x:** `ASSUME DP:<value>`

### EXPECT / ENDEXPECT

```asm
EXPECT   <error-number>[,...]        ; expect specific errors
...
ENDEXPECT                            ; verify they occurred
```

### Mode Switches

| Instruction | Purpose | Targets |
|-------------|---------|---------|
| `SUPMODE ON\|OFF` | Supervisor mode | 680x0, TLCS-900, SH7000, i960, 29K, XA, PowerPC, M*Core, TMS9900 |
| `FPU ON\|OFF` | FPU instructions | 680x0, 80x86, i960 |
| `PMMU ON\|OFF` | MMU instructions | 680x0 |
| `FULLPMMU ON\|OFF` | Full 68851 MMU set | 680x0 |
| `PADDING ON\|OFF` | Auto word-align padding | 680x0, 68xx, M*Core, XA, H8, SH, MSP430, AVR, ST7 |
| `PACKING ON\|OFF` | Pack bytes in DATA words | AVR |
| `MAXMODE ON\|OFF` | Max/min mode | TLCS-900, H8 |
| `SRCMODE ON\|OFF` | Source/binary mode | MCS-251 |
| `BIGENDIAN ON\|OFF` | Byte order for data | MCS-51/251, PowerPC |
| `WRAPMODE ON\|OFF` | PC wraps at ROM size | AVR |
| `EXTMODE ON\|OFF` | Extended addressing | Z380 |
| `LWORDMODE ON\|OFF` | Long word mode | Z380 |
| `BRANCHEXT ON\|OFF` | Auto extend branches | XA |
| `Z80SYNTAX OFF\|ON\|EXCLUSIVE` | Allow Z80 mnemonics | 8080/8085 |
| `RELAXED ON\|OFF` | Mixed number syntax | All |

### Other

- `EMULATED <instr>,...` (29K) -- mark instructions as emulated
- `CKPT <register|NOTHING>` (TI990/12) -- default checkpoint register

---

## 3. Data Definition Instructions

### Motorola Style

| Instruction | Size | Targets |
|-------------|------|---------|
| `DC.B` | Byte | 680x0, 68xx, H8, SH, XA, ST7, M*Core |
| `DC.W` | Word (16-bit, default) | Same |
| `DC.L` | Long (32-bit) | Same |
| `DC.Q` | Quad (64-bit) | 680x0 |
| `DC.S` | Single float (32-bit) | 680x0 |
| `DC.D` | Double float (64-bit) | 680x0 |
| `DC.X` | Extended float (80-96 bit) | 680x0 |
| `DC.P` | Packed decimal float | 680x0 |
| `DS.B` / `DS.W` / `DS.L` | Reserve bytes/words/longs | Same |

Repeat count: `DC.B [10]0` (10 zero bytes). `?` = reserve without init.
`DS.W 0` aligns to word boundary.

### Intel Style

| Instruction | Size |
|-------------|------|
| `DN` | Nibble (4-bit) |
| `DB` / `DEFB` | Byte (8-bit) |
| `DW` / `DEFW` | Word (16-bit) |
| `DD` | Double word (32-bit) |
| `DQ` | Quad word (64-bit) |
| `DT` | Ten bytes (80-bit) |
| `DS` | Reserve bytes |

`?` = reserve. `DUP` = repeat: `DB 3 DUP (1,2)` -> `1,2,1,2,1,2`.
Cannot mix constants and `?` in one instruction. Max 1024 bytes per line.

### 6502/68xx Style

| Instruction | Equivalent |
|-------------|------------|
| `BYT` / `FCB` | `DC.B` / `DB` |
| `ADR` / `FDB` | `DC.W` / `DW` |
| `FCC` | String (double quotes) |
| `DFS` / `RMB` | `DS.B` / `DS` |

### Other Families

| Instruction | Targets | Notes |
|-------------|---------|-------|
| `DATA` | PIC, AVR, TI, H8/500, 4004 | Char packing varies by target |
| `BYTE` | ST6, TI, MSP | 8-bit values |
| `WORD` | ST6, i960, TI | 16 or 32-bit depending on target |
| `LONG` | TI 320C2x/5x | 32-bit LoWord-HiWord |
| `RES` | PIC, AVR, TI, MELPS | Reserve (words in code, bytes/nibbles in data) |
| `BSS` | TI, MSP | Reserve |
| `ZERO` | PIC | Zero-fill words |
| `FB` / `FW` | COP4/8 | Fill byte/word |
| `ASCII` / `ASCIZ` | ST6 | String / null-terminated string |
| `STRING` / `RSTRING` | TI 320C2x/5x | Packed bytes (upper/lower first) |
| `SPACE` | i960 | Reserve |

### Floating-Point Data

| Instruction | Format | Targets |
|-------------|--------|---------|
| `SINGLE` | 32-bit TI format | 320C3x/4x |
| `DOUBLE` | 64-bit IEEE | 320C6x |
| `EXTENDED` | 2-word TI format | 320C3x/4x |
| `FLOAT` | 32-bit IEEE | 320C2x/5x |
| `EFLOAT` | 16+16 bit | 320C2x/5x |
| `BFLOAT` | 32+16 bit | 320C2x/5x |
| `TFLOAT` | 64+32 bit | 320C2x/5x |

### Fixed-Point Data (TI 320C2x/5x)

`Qxx` = 16-bit, `LQxx` = 32-bit. `xx` = fractional bits (00-99).
`Q15 0.5` -> `4000h`. `LQ20 CONSTPI` -> two words.

### ALIGN

```asm
ALIGN  <boundary>[,<fill>]          ; align PC to multiple of boundary
```

### LTORG (SH7x00)

Flushes pending literal pool constants into code at current position.

### BINCLUDE

```asm
BINCLUDE  <file>[,<offset>[,<length>]]  ; embed binary file data
```

---

## 4. Macro Instructions

### MACRO / ENDM

```asm
<name>  MACRO  [param1[=default],param2,...,{controls}]
        <body>
        ENDM
```

**Control parameters** (in braces):
- `EXPAND` / `NOEXPAND` -- listing of expanded code
- `EXPIF` / `NOEXPIF` -- listing of conditionals
- `EXPMACRO` / `NOEXPMACRO` -- listing of macro defs
- `EXPREST` / `NOEXPREST` -- listing of other lines
- `PUBLIC[:section]` -- assign macro to parent section
- `GLOBAL[:section]` -- create prefixed copy
- `EXPORT` / `NOEXPORT` -- write to `-M` export file
- `INTLABEL` / `NOINTLABEL` -- use calling line's label
- `GLOBALSYMBOLS` / `NOGLOBALSYMBOLS` -- label visibility

**Implicit parameters:**
- `ATTRIBUTE` -- attribute from call (e.g. `.L`)
- `ALLARGS` -- all arguments as comma-separated string
- `ARGCOUNT` -- number of arguments passed
- `__LABEL__` -- label from calling line (requires `INTLABEL`)

**Argument passing:**
- Positional: by order, missing = default or empty
- Keyword: `macro_name para=val1,parb=val2`
- Keyword args must not follow positional

**Underscore concatenation:** `part1_part2` in body expands parameters:
```asm
concat  MACRO   part1,part2
        call    part1_part2
        ENDM
        concat  module,function   ; -> call module_function
```

**Labels:** local by default. Use `LABEL $` for global. `GLOBALSYMBOLS` makes all global.

**Override instructions:** prefix with `!` for original: `!srl op`

**Recursion:** legal, max depth set by `MAXNEST` (default 256).

### IRP

```asm
IRP     <sym>,<val1>,<val2>,...[,{GLOBALSYMBOLS}]
<body>
ENDM
```

Iterates body over value list.

### IRPC

```asm
IRPC    <sym>,"<string>"
<body>
ENDM
```

Iterates body over each character.

### REPT

```asm
REPT    <count>[,{GLOBALSYMBOLS}]
<body>
ENDM
```

Repeats body N times. 0 or negative = no expansion.

### WHILE

```asm
WHILE   <boolean-expr>
<body that modifies condition>
ENDM
```

Repeats until expression is false.

### EXITM

Terminates current macro/REPT/IRP/WHILE expansion.

### SHIFT

Discards first macro parameter, shifts rest down. For variable-length arg lists.

### MAXNEST

```asm
MAXNEST  <value>                    ; max recursive depth (default 256, 0=unlimited)
```

### FUNCTION

```asm
<name>  FUNCTION  <arg1>[,<arg2>,...],<expression>
```

User-defined function for formula expressions. Last item is the expression.

```asm
isdigit FUNCTION ch,(ch>='0')&&(ch<='9')
double  FUNCTION x,x+x
```

---

## 5. Structures

### STRUCT / ENDSTRUCT

```asm
<name>  STRUCT  [NOEXTNAMES] [DOTS]
<elem>  DS.x    <size>
        ALIGN   <n>
<name>  ENDSTRUCT
```

Creates field offset symbols (`<struct>_<field>`) and `<struct>_LEN`.
Only memory-reserving instructions allowed inside. Options:
- `NOEXTNAMES` -- no struct name prefix
- `DOTS` -- use `.` separator instead of `_`

Global setting: `DOTTEDSTRUCTS ON|OFF`

### Usage

```asm
<label>  <StructName>               ; instantiate, creates <label>_<field> symbols
```

### UNION / ENDUNION

All elements at offset 0. Size = max element size.

### Nested Structures

Instantiate or define structs within other structs. Unnamed inner structs
contribute fields directly to the outer struct.

---

## 6. Conditional Assembly

### IF / ELSEIF / ENDIF

```asm
IF      <expr>                      ; non-zero = true
  ...
ELSEIF  <expr>                      ; or ELSE
  ...
ENDIF
```

First true branch assembled; rest skipped. ELSEIF without argument = default (must be last).

### IF Variants

| Instruction | True when... |
|-------------|-------------|
| `IFDEF <sym>` | Symbol defined |
| `IFNDEF <sym>` | Symbol not defined |
| `IFUSED <sym>` | Symbol referenced |
| `IFNUSED <sym>` | Symbol not referenced |
| `IFEXIST <file>` | File exists |
| `IFNEXIST <file>` | File doesn't exist |
| `IFB <args>` | All arguments empty |
| `IFNB <args>` | Any argument non-empty |

### SWITCH / CASE / ENDCASE

```asm
SWITCH  <expr>                      ; or SELECT if SWITCH is a machine instr
CASE    <val1>[,<val2>,...]
  ...
CASE    <val3>
  ...
ELSECASE                            ; default (warning if missing and no match)
  ...
ENDCASE
```

---

## 7. Listing Control

### PAGE / PAGESIZE

```asm
PAGE  <lines>[,<width>]             ; set page dimensions (default: 60, unlimited)
```

Use `PAGESIZE` if `PAGE` is a machine instruction.

### NEWPAGE

```asm
NEWPAGE [<depth>]                   ; force page break (hierarchical numbering)
```

### MACEXP_DFT / MACEXP_OVR

```asm
MACEXP_DFT  ON|OFF|IF|NOIF|MACRO|NOMACRO|REST|NOREST
MACEXP_OVR  ON|OFF|IF|NOIF|MACRO|NOMACRO|REST|NOREST
MACEXP_OVR                          ; disable all overrides
```

### LISTING

```asm
LISTING  ON|OFF|NOSKIPPED|PURECODE
```

### TITLE

```asm
TITLE  <string>                     ; add line to page header
```

### PRTINIT / PRTEXIT

```asm
PRTINIT  <string>                   ; printer init sequence
PRTEXIT  <string>                   ; printer exit sequence
```

### RADIX / OUTRADIX

```asm
RADIX    <base>                     ; default number base (2-36, default 10)
OUTRADIX <base>                     ; output base for \{...} (2-36, default 16)
```

---

## 8. Sections (Local Symbols)

### SECTION / ENDSECTION

```asm
SECTION  <name>
  ; symbols are local
ENDSECTION [<name>]
```

Nestable. Symbol lookup: local first, then parent sections upward.
Cross-section reference: `sym[SectionName]` or `sym[]` for global.
Special: `PARENT`, `PARENT0`-`PARENT9` for relative references.

### PUBLIC

```asm
PUBLIC  <name>                      ; make globally accessible
PUBLIC  <name>:<section>            ; assign to parent section
PUBLIC  <name>:PARENT               ; export one level up
```

Must appear before symbol definition.

### GLOBAL

```asm
GLOBAL  <name>                      ; create prefixed copy globally
GLOBAL  <name>:<section>            ; create prefixed copy in section
```

Keeps symbol local but creates `Section_Name_Symbol` in target scope.

### FORWARD

```asm
FORWARD  <name>                     ; declare explicitly local (prevents global lookup)
```

Must appear before definition and first use.

### SHARED

```asm
SHARED  <sym1>[,<sym2>,...]  [; comment]
```

Writes symbols to share file (requires `-c`/`-p`/`-a`).

---

## 9. Miscellaneous

### INCLUDE

```asm
INCLUDE  <filename>                 ; insert file inline (.INC default extension)
```

### BINCLUDE

```asm
BINCLUDE  <file>[,<offset>[,<length>]]  ; embed binary data
```

### MESSAGE / WARNING / ERROR / FATAL

```asm
MESSAGE  <string>                   ; print to console (no error)
WARNING  <string>                   ; possible error, assembly continues
ERROR    <string>                   ; true error, no code file
FATAL    <string>                   ; immediate termination
```

### READ

```asm
READ  [<prompt>],<symbol>           ; interactive input during assembly
```

### END

```asm
END  [<entry-point>]                ; end of program (optional entry address)
```
