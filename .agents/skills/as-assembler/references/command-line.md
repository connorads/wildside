# AS Macro Assembler -- Command Line & Environment Reference

## Invocation

```
as [options] <source file(s)>
```

Multiple source files may be specified; all options apply to every file. Wildcards permitted.

### Option Syntax

- Prefix with `-` or `/`
- `~` before option letter forces lowercase: `/~I` becomes `/i`
- `#` before option letter forces uppercase: `-#u` becomes `-U`
- `+` before option negates it (override ASCMD defaults): `as +L file.asm`
- `@keyfile` loads options from file (one per line, max 255 chars, switch+arg on same line)

## Complete Option Reference

| Option | Description |
|--------|-------------|
| `-l` | Send listing to console |
| `-L` | Write listing to `.LST` file |
| `-OLIST <file>` | Redirect listing to specific file |
| `-LISTRADIX <n>` | Listing number base (2-36, default hex) |
| `-o <name>` | Custom output code file name |
| `-SHAREOUT <name>` | Custom share file output name |
| `-c` | Export shared symbols in C format (`.H`) |
| `-p` | Export shared symbols in Pascal format (`.INC`) |
| `-a` | Export shared symbols in assembler format (`.INC`) |
| `-g [format]` | Debug info: `MAP` (default), `NOICE`, `ATMEL` |
| `-noicemask <val>` | Bitmask for NoICE segment filtering |
| `-w` | Suppress all warnings |
| `-E [file]` | Redirect errors (special: `!0`-`!4` for std handles) |
| `-q` / `-quiet` | Suppress non-error messages |
| `-h` | Lowercase hex output |
| `-i <paths>` | Semicolon-separated include search paths |
| `-u` | Calculate and display segment usage |
| `-C` | Cross-reference list |
| `-s` | List sections with nesting |
| `-t <mask>` | Suppress listing components (bitmask) |
| `-D <SYM[=val]>` | Define symbols (default value: TRUE) |
| `-A` | Compact symbol storage (saves memory) |
| `-x` | Increase error detail (repeatable: 0/1/2) |
| `-n` | Show error numbers |
| `-U` | Case-sensitive mode |
| `-P` | Write macro-processed output to `.I` file |
| `-M` | Generate `.MAC` file with macro definitions |
| `-G` | Toggle code generation (default: on) |
| `-r [n]` | Warn about additional-pass situations |
| `-relaxed` | Enable RELAXED number syntax mode |
| `-Y` | Suppress out-of-branch messages during multi-pass |
| `-cpu <name>` | Set target processor |
| `-alias <new>=<old>` | Define processor alias |
| `-gnuerrors` | GNU-style error format |
| `-maxerrors [n]` | Terminate after n errors |
| `-Werror` | Treat warnings as errors |

### Listing Mask Bits (`-t`)

Setting a bit **suppresses** that component:

| Bit | Component |
|-----|-----------|
| 0 | Source files and produced code |
| 1 | Symbol table |
| 2 | Macro list |
| 3 | Function list |
| 4 | Line numbering |
| 5 | Register symbol list |
| 7 | Character set table |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (warnings may have occurred) |
| 1 | Help display only |
| 2 | Errors found, no code file produced |
| 3 | Fatal error |
| 4 | Startup/parameter error |
| 255 | Internal initialisation error |

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ASCMD` | Persistent default options (processed before command line) |
| `AS_MSGPATH` | Directory for message files (`.MSG`) |
| `USEXMS` | DOS: set `n` to suppress XMS usage |
| `USEEMS` | DOS: set `n` to suppress EMS usage |
| `USEANSI` | Set `n` to suppress ANSI sequences |
| `ASXSWAP` | DPMI: configure swap file `<size>[,filename]` |
| `COUNTRY` | DOS/OS2: language selection |
| `LANG` | Unix: language selection |

## Input File Format

### Line Structure

```
[label[:]] <mnemonic>[.attr] [param[,param...]] [;comment]
```

- **Max line length:** 255 characters (excess silently discarded)
- **Label field:** Optional. Column 1 = colon optional; elsewhere = colon mandatory
- **Mnemonic:** Cannot start in column 1 without a preceding label
- **Attribute:** Processor-specific size suffix (e.g. `.B`, `.W`, `.L` for 68K)
- **Parameters:** 0-20 per instruction, comma-separated. Commas in brackets/quotes don't split
- **Comments:** Semicolon to end of line; column 1 = whole-line comment
- **DSP56xxx:** `||` in label field = parallel execution with previous instruction

### Include Path Resolution

Search order for `INCLUDE` directives:
1. Current directory
2. `-i` search paths
3. EXE-file directory
4. `AS_MSGPATH`
5. `PATH` directories
6. Compiled `LIBDIR` macro

## Listing Format

### Structure

1. **Source code with produced code** -- main body
2. **Symbol table** -- alphabetical, all symbols
3. **Usage list** -- occupied memory per segment (requires `-u`)
4. **Cross-reference list** -- symbol usage across files (requires `-C`)

### Source Line Format

```
[<n>] <line>/<address> <code> <source>
```

| Field | Meaning |
|-------|---------|
| `[n]` | Include nesting depth (0 = main file, not shown) |
| `line` | Source line number |
| `address` | Code generation address (after `/`) |
| `code` | Hex output bytes/words |
| `source` | Original source line |

### Symbol Table

- Double-column for symbols up to 40 chars, single for longer
- Alphabetically sorted
- `*` prefix = undefined but referenced

### Cross-Reference List

```
symbol <name> (=<value>, <file>/<line>):
  file <file1>:
    <n1>[(m1)] ... <nk>[(mk)]
```

## Predefined Symbols

### Core Symbols

| Symbol | Type | Meaning |
|--------|------|---------|
| `TRUE` | Int | 1 |
| `FALSE` | Int | 0 |
| `CONSTPI` | Float | Pi |
| `VERSION` | Int | BCD assembler version (e.g. `1331h` = v1.33p1) |
| `ARCHITECTURE` | String | Target platform `processor-manufacturer-OS` |
| `DATE` | String | Assembly start date |
| `TIME` | String | Assembly start time |
| `MOMCPU` | Int | Current CPU numeric ID |
| `MOMCPUNAME` | String | Current CPU name (reflects aliases) |
| `MOMFILE` | String | Current source filename |
| `MOMLINE` | Int | Current line number |
| `MOMPASS` | Int | Current pass number |
| `MOMSECTION` | String | Current section name (empty if none) |
| `*`, `$`, `PC` | Int | Current program counter |
| `CASESENSITIVE` | Int | TRUE if `-U` active |

### State Symbols

| Symbol | Related Pseudo-op | Purpose |
|--------|-------------------|---------|
| `PADDING` | `PADDING` | Current padding mode |
| `PACKING` | `PACKING` | Current packing mode |
| `Z80SYNTAX` | `Z80SYNTAX` | Z80 syntax mode (0/1/2) |
| `BRANCHEXT` | `BRANCHEXT` | Branch extension mode |
| `LISTON` / `LISTING` | `LISTING` | Listing state |
| `MACEXP` | `MACEXP_DFT` | Macro expansion state |
| `NESTING` | -- | Section/include nesting depth |
| `HASFPU` | `FPU` | FPU instructions enabled |
| `HASPMMU` | `PMMU` | PMMU instructions enabled |
| `SUPMODE` / `INSUPMODE` | `SUPMODE` | Supervisor mode active |
| `MOMFIRST` | -- | First pass flag |
| `MOMSEGMENT` | `SEGMENT` | Current segment name |
| `WRAPMODE` | `WRAPMODE` | Address wrap mode |

### SYMTYPE Return Values

| Value | Segment |
|-------|---------|
| -1 | Undefined |
| 0 | None (typeless) |
| 1 | CODE |
| 2 | DATA |
| 3 | IDATA |
| 4 | XDATA |
| 5 | YDATA |
| 6 | BITDATA |
| 7 | IO |
| 8 | REG |
| 9 | ROMDATA |
| 10 | EEDATA |
| 128 | Register symbol |

In case-sensitive mode (`-U`), predefined symbols must be referenced in exact uppercase.
