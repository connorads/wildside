# AS Macro Assembler -- Symbols, Expressions & Built-in Functions

## 1. Symbol Conventions

### Naming Rules

- May contain letters, digits, underscores (`_`), and dots (`.`)
- Must not start with a digit
- Dots reserved for MCS-51 bit notation; prefer underscores
- Max length: 255 characters, all significant
- Case-insensitive by default; `-U` enables case-sensitivity

### Label Syntax

- Trailing colon optional when label starts in column 1
- Colon required when label is not in column 1
- When processor supports colon-separated attributes, add space after label colon

### Segment Assignment

Every symbol carries a segment marker (CODE, DATA, IDATA, XDATA, etc.) preventing
accidental misuse across address spaces. Labels get automatic assignment; EQU/SET
symbols are typeless. Query with `SYMTYPE(sym)`.

### Symbol Composition (Dynamic Names)

Enclose string symbol names in braces to compose symbol names dynamically:

```asm
cnt     set     cnt+1
temp    equ     "\{CNT}"
        jnz     skip{temp}
skip{temp}: nop
```

### Query Functions

- `SYMTYPE(sym)` -- segment type as integer (-1 if undefined)
- `DEFINED(sym)` -- tests existence

## 2. Temporary Symbols

### Named (`$$`)

Symbols starting with `$$` use an internal counter that increments at each
non-temporary label definition:

```asm
$$loop: nop
        dbra    d0,$$loop   ; this $$loop
split:                       ; counter increments
$$loop: nop
        dbra    d0,$$loop   ; new $$loop
```

### Nameless (`+`, `-`, `/`)

- `-` as label = backward reference (`--` = 2nd most recent, etc.)
- `+` as label = forward reference (`++` = 2nd next, etc.)
- `/` as label = bidirectional (backward or forward)

```asm
-       ldx     #00
-       dex
        bne     -           ; backward to 'dex'
        beq     +           ; forward skip
        jsr     SomeRtn
+       bne     --          ; backward to 'ldx'
```

### Composed (`.` prefix)

Symbols beginning with `.` prepend the last non-dotted label:

```asm
proc1:
.loop   moveq   #20,d0      ; actual: proc1.loop
        dbra    d0,.loop
proc2:
.loop   moveq   #10,d1      ; actual: proc2.loop
```

Full qualified references (`proc2.loop`) remain globally accessible.

## 3. Integer Constants

### Motorola Mode (68K, 6502, PIC, Hitachi, Rockwell, Thomson)

| Base | Notation | Example |
|------|----------|---------|
| Decimal | Direct | `255` |
| Hex | `$` prefix | `$FF` |
| Binary | `%` prefix | `%11111111` |
| Octal | `@` prefix | `@377` |

### Intel Mode (8051, Z80, 80x86, TMS, NEC, Siemens, Toshiba, Philips, Fujitsu)

| Base | Notation | Example |
|------|----------|---------|
| Decimal | Direct | `255` |
| Hex | `H` suffix | `0FFH` |
| Binary | `B` suffix | `11111111B` |
| Octal | `O`/`Q` suffix | `377O` |

Hex constants starting with A-F must be prepended with `0` (e.g. `0FFH` not `FFH`).

### C Mode (PowerPC, AVR, AMD 29K, National, Atmel)

| Base | Notation | Example |
|------|----------|---------|
| Decimal | Direct | `255` |
| Hex | `0x` prefix | `0xFF` |
| Binary | `0b` prefix | `0b11111111` |
| Octal | `0` prefix | `0377` |

### RELAXED Mode (cross-platform)

Apostrophe-delimited with prefix, alongside native notation:

| Base | Notation | Example |
|------|----------|---------|
| Hex | `x'...'` / `h'...'` | `x'FF'` |
| Octal | `o'...'` | `o'377'` |
| Binary | `b'...'` | `b'1010'` |

### RADIX Instruction

`RADIX <base>` sets default base for constants without markers (default: 10, range 2-36).
Higher values consume letters needed for system designators.

### Calculation Resolution

- Integers: 32 or 64 bits (platform dependent)
- Floats: up to 80 bits (~10^308 max)
- Strings: 255 characters
- Overflow checked only on final results

## 4. Floating-Point Constants

Format: `[-]<integer>[.decimals][E[-]exponent]`

AS tries integer interpretation first. Force float with: `2.0` instead of `2`.

## 5. String Constants

Enclosed in single (`'`) or double (`"`) quotes.

### Escape Sequences

| Escape | Character | Escape | Character |
|--------|-----------|--------|-----------|
| `\\` | Backslash | `\n` | Linefeed |
| `\b` | Backspace | `\r` | Carriage return |
| `\a` | Bell | `\t` | Tab |
| `\e` | Escape | `\'`/`\H` | Apostrophe |
| `\"`/`\I` | Quote | `\DDD` | 3 decimal digits |
| `\xHH` | 2 hex digits | `\0OOO` | Up to 3 octal digits |

### Expression Interpolation

```asm
message "root of 81 : \{sqrt(81)}"   ; "root of 81 : 9"
```

Integer results default to hex output (changeable via `OUTRADIX`).

### Multi-character Constants

Single quotes concatenate: `'AB'` = `$4142`, `'ABCD'` = `$41424344`

In data directives: double quotes place characters as separate elements
(`dc.w "ab"` = two words); single quotes concatenate (`dc.w 'ab'` = one word).

## 6. Operators

| Rank | Operator | Function | Int | Float | String |
|------|----------|----------|-----|-------|--------|
| 1 | `~` | Binary NOT | Y | | |
| 2 | `~~` | Logical NOT | Y | | |
| 3 | `<<` / `>>` | Shift left/right | Y | | |
| 4 | `><` | Bit mirror (lowest N bits) | Y | | |
| 5 | `&` | Binary AND | Y | | |
| 6 | `\|` | Binary OR | Y | | |
| 7 | `!` | Binary XOR | Y | | |
| 8 | `^` | Power | Y | Y | |
| 9 | `*` / `/` / `#` | Multiply / Divide / Modulo | Y | Y (not #) | |
| 10 | `+` / `-` | Add / Subtract (+ also concatenates strings) | Y | Y | + only |
| 11 | `&&` | Logical AND | Y | | |
| 12 | `\|\|` | Logical OR | Y | | |
| 13 | `!!` | Logical XOR | Y | | |
| 14 | `=`/`==`/`<>`/`<`/`>`/`<=`/`>=` | Comparison | Y | Y | Y |

Lower rank = higher precedence. Parentheses override.

**Bit mirror (`><`):** mirrors lowest N bits of left operand (N = right operand, 1-32).

## 7. Built-in Functions

### Trigonometric

| Function | Description |
|----------|-------------|
| `SIN(x)` | Sine |
| `COS(x)` | Cosine |
| `TAN(x)` | Tangent |
| `COT(x)` | Cotangent |
| `ASIN(x)` | Inverse sine (abs(x) <= 1) |
| `ACOS(x)` | Inverse cosine (abs(x) <= 1) |
| `ATAN(x)` | Inverse tangent |
| `ACOT(x)` | Inverse cotangent |

### Hyperbolic

`SINH`, `COSH`, `TANH`, `COTH`, `ASINH`, `ACOSH`, `ATANH`, `ACOTH`

### Exponential & Logarithmic

| Function | Description |
|----------|-------------|
| `SQRT(x)` | Square root (x >= 0) |
| `EXP(x)` | e^x |
| `ALOG(x)` | 10^x |
| `ALD(x)` | 2^x |
| `LN(x)` | Natural log |
| `LOG(x)` | Log base 10 |
| `LD(x)` | Log base 2 |

### Numeric

| Function | Description |
|----------|-------------|
| `ABS(x)` | Absolute value (int or float) |
| `SGN(x)` | Sign: -1, 0, or 1 |
| `INT(x)` | Integer part (truncate float) |

### Bit Manipulation

| Function | Description |
|----------|-------------|
| `BITCNT(x)` | Count of 1-bits |
| `FIRSTBIT(x)` | Position of lowest set bit (-1 if none) |
| `LASTBIT(x)` | Position of highest set bit (-1 if none) |
| `BITPOS(x)` | Position of the single set bit (-1 and error if not exactly one) |

### String

| Function | Description |
|----------|-------------|
| `STRLEN(s)` | String length |
| `SUBSTR(s, start, len)` | Substring (0-based, len=0 means to end) |
| `CHARFROMSTR(s, pos)` | Char code at position (0-based, -1 if out of range) |
| `STRSTR(haystack, needle)` | Find first occurrence (0-based, -1 if not found) |
| `UPSTRING(s)` | Convert to uppercase |
| `LOWSTRING(s)` | Convert to lowercase |
| `TOUPPER(c)` | Uppercase char code (integer) |
| `TOLOWER(c)` | Lowercase char code (integer) |

### Type & Evaluation

| Function | Description |
|----------|-------------|
| `VAL(s)` | Evaluate string as expression |
| `EXPRTYPE(x)` | Return type: 0=int, 1=float, 2=string |
| `DEFINED(sym)` | Test symbol existence |
| `SYMTYPE(sym)` | Return segment type (see SYMTYPE table) |

Built-in functions accept any case (`sin`, `SIN`, `Sin` all work).
User-defined functions respect case in `-U` mode.

## 8. Register Symbols

Register symbols assign names to processor registers. They exist for processors
where registers are fixed literals (not memory-mapped): PowerPC, M-Core, XGate,
MCS-48/51, 80C16x, AVR, Z8, KCPSM, MSP430, ST9, M16, H8, SH7x00, i960, XA, 29K, etc.

### Definition

```asm
myreg   REG     r17
myreg2  REG     myreg       ; myreg2 -> r17
```

### Restrictions

- No arithmetic on register values: `myreg+1` is invalid
- Stricter forward reference rules (unknown = assumed integer, may cause mode errors)
- Local to sections; cross-section: `section[symbol]`

## 9. Forward References & Multi-Pass Assembly

### How It Works

1. Unknown symbol encountered: parser substitutes current PC
2. Code generated with assumed values
3. After pass, AS checks if any symbol values changed
4. Additional passes until values stabilise
5. Complex programs may need 12+ passes

### Best Practices

- Place EQU definitions before first use
- Use explicit short addressing where possible (e.g. `ldd <Var`)
- Minimise forward references to reduce passes
- `-Y` suppresses branch errors during stabilisation (not recommended)
- EQU symbols with forward references are not evaluated in pass 1
