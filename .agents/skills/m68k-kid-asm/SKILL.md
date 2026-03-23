---
name: m68k-kid-asm
description: >
  Reference skill for reading Motorola 68000 assembly in the Kid Chameleon disassembly.
  Use this skill when working with 68k ASM, reading the Kid Chameleon disassembly
  (kid.asm, code/enemy/*.asm), translating ASM routines to native Rust, understanding
  M68000 instructions, addressing modes, condition codes, or Mega Drive hardware.
  Also use when debugging oracle divergences that require tracing ASM routines, or
  when encountering unfamiliar 68k opcodes, register conventions, or object field offsets.
  Complements the as-assembler skill (which covers the AS assembler tool) with 68k
  instruction semantics and Kid Chameleon-specific ROM conventions.
---

# M68000 + Kid Chameleon ASM Reading Guide

Read 68k assembly from `.cache/upstream/kid-chameleon-disasm/` with confidence. This skill covers the M68000 instruction set, Mega Drive hardware, and Kid Chameleon's specific ROM conventions.

## 1. M68k Instruction Quick Reference

### Registers

- **D0-D7**: 32-bit data registers (operands for arithmetic, logic, shifts)
- **A0-A6**: 32-bit address registers (pointers; word ops sign-extend to 32 bits)
- **A7 (SP)**: Stack pointer
- **PC**: Program counter
- **SR**: Status register (system byte + CCR)
- **CCR**: Condition code register (lower byte of SR): X N Z V C

### Data Movement

| Instruction | Sizes | Description | CCR (XNZVC) |
|-------------|-------|-------------|--------------|
| MOVE | B/W/L | Copy source to destination | -**00 |
| MOVEA | W/L | Copy to address register (no flags) | ----- |
| MOVEQ | L | Load signed 8-bit immediate to Dn (fast) | -**00 |
| MOVEM | W/L | Move multiple registers to/from memory | ----- |
| LEA | L | Load effective address into An | ----- |
| PEA | L | Push effective address onto stack | ----- |
| EXG | L | Exchange any two registers | ----- |
| SWAP | W | Swap upper/lower words of Dn | -**00 |
| CLR | B/W/L | Clear destination to zero | -0100 |

### Arithmetic

| Instruction | Sizes | Description | CCR (XNZVC) |
|-------------|-------|-------------|--------------|
| ADD | B/W/L | Add source + destination | ***** |
| ADDA | W/L | Add to address register (no flags) | ----- |
| ADDI | B/W/L | Add immediate | ***** |
| ADDQ | B/W/L | Add quick (1-8 immediate) | ***** |
| ADDX | B/W/L | Add with extend (for multi-precision) | *****  |
| SUB | B/W/L | Subtract source from destination | ***** |
| SUBA | W/L | Subtract from address register (no flags) | ----- |
| SUBI | B/W/L | Subtract immediate | ***** |
| SUBQ | B/W/L | Subtract quick (1-8 immediate) | ***** |
| SUBX | B/W/L | Subtract with extend | ***** |
| MULU | W | Unsigned multiply (16x16 -> 32) | -**00 |
| MULS | W | Signed multiply (16x16 -> 32) | -**00 |
| DIVU | W | Unsigned divide (32/16 -> 16r:16q) | -**0* |
| DIVS | W | Signed divide (32/16 -> 16r:16q) | -**0* |
| NEG | B/W/L | Negate (0 - destination) | ***** |
| NEGX | B/W/L | Negate with extend | ***** |
| EXT | W/L | Sign-extend byte->word or word->long | -**00 |

### Logic

| Instruction | Sizes | Description | CCR (XNZVC) |
|-------------|-------|-------------|--------------|
| AND | B/W/L | Bitwise AND | -**00 |
| ANDI | B/W/L | AND immediate | -**00 |
| OR | B/W/L | Bitwise OR | -**00 |
| ORI | B/W/L | OR immediate | -**00 |
| EOR | B/W/L | Bitwise exclusive OR | -**00 |
| EORI | B/W/L | XOR immediate | -**00 |
| NOT | B/W/L | Bitwise complement | -**00 |

### Shift and Rotate

| Instruction | Sizes | Description | CCR (XNZVC) |
|-------------|-------|-------------|--------------|
| LSL | B/W/L | Logical shift left (zero fill) | ***0* |
| LSR | B/W/L | Logical shift right (zero fill) | ***0* |
| ASL | B/W/L | Arithmetic shift left (= LSL) | ***?* |
| ASR | B/W/L | Arithmetic shift right (sign fill) | ***0* |
| ROL | B/W/L | Rotate left | -**0* |
| ROR | B/W/L | Rotate right | -**0* |
| ROXL | B/W/L | Rotate left through extend | ***0* |
| ROXR | B/W/L | Rotate right through extend | ***0* |

### Bit Manipulation

| Instruction | Sizes | Description | CCR (XNZVC) |
|-------------|-------|-------------|--------------|
| BTST | B/L | Test bit (sets Z if bit is 0) | --*-- |
| BSET | B/L | Test and set bit | --*-- |
| BCLR | B/L | Test and clear bit | --*-- |
| BCHG | B/L | Test and change (toggle) bit | --*-- |

### Compare and Test

| Instruction | Sizes | Description | CCR (XNZVC) |
|-------------|-------|-------------|--------------|
| CMP | B/W/L | Compare (destination - source, flags only) | -**** |
| CMPA | W/L | Compare to address register | -**** |
| CMPI | B/W/L | Compare immediate | -**** |
| CMPM | B/W/L | Compare memory (An)+,(An)+ | -**** |
| TST | B/W/L | Test against zero (sets N, Z) | -**00 |
| TAS | B | Test and set bit 7 (atomic) | -**00 |

### Program Control

| Instruction | Description |
|-------------|-------------|
| BRA | Branch always (PC-relative) |
| BSR | Branch to subroutine (pushes return address) |
| Bcc | Branch on condition (14 conditions, see section 3) |
| DBcc | Decrement Dn and branch if condition false AND Dn != -1 |
| Scc | Set byte to $FF if condition true, $00 if false |
| JMP | Jump to effective address (absolute) |
| JSR | Jump to subroutine (pushes return address) |
| RTS | Return from subroutine (pulls PC from stack) |
| RTE | Return from exception |
| NOP | No operation |

### BCD and System

| Instruction | Sizes | Description |
|-------------|-------|-------------|
| ABCD | B | Add BCD with extend |
| SBCD | B | Subtract BCD with extend |
| NBCD | B | Negate BCD |
| MOVEP | W/L | Move peripheral data (alternating bytes between Dn and memory) |
| CHK | W | Check Dn against bound; trap if out of range |
| TRAPV | - | Trap if overflow (V flag set) |
| LINK | - | Allocate stack frame (An, #displacement) |
| UNLK | - | Deallocate stack frame |
| TRAP | - | Software interrupt (#0-15) |
| STOP | - | Stop and load SR |
| RESET | - | Assert reset line |
| ILLEGAL | - | Force illegal instruction trap (opcode $4AFC) |

**CCR key:** `*` = affected, `-` = unchanged, `0` = cleared, `1` = set, `?` = undefined

## 2. Addressing Modes

| # | Mode | Syntax | Example | Notes |
|---|------|--------|---------|-------|
| 0 | Data register direct | `Dn` | `move.l d0,d1` | Fastest |
| 1 | Address register direct | `An` | `movea.l a3,a4` | Word ops sign-extend |
| 2 | Address register indirect | `(An)` | `move.w (a3),d0` | Load from pointer |
| 3 | Post-increment | `(An)+` | `move.w (a0)+,d0` | Read then advance |
| 4 | Pre-decrement | `-(An)` | `move.l d0,-(sp)` | Push onto stack |
| 5 | Displacement | `d16(An)` | `move.w x_pos(a3),d0` | **Object field access** |
| 6 | Indexed | `d8(An,Xn)` | `move.w (a1,d0.w),d1` | Jump table dispatch |
| 7 | Absolute short | `addr.w` | `move.w ($F800).w,d0` | RAM (sign-extended, $FF8000+) |
| 8 | Absolute long | `addr.l` | `jsr $C00000` | Full 24-bit address |
| 9 | PC displacement | `d16(PC)` | `lea off_346EE(pc),a4` | ROM data tables |
| 10 | PC indexed | `d8(PC,Xn)` | `jmp off_XXX(pc,d0.w)` | Jump table dispatch |
| 11 | Immediate | `#imm` | `moveq #0,d0` | Constants |

**Key insight:** Mode 5 (`d16(An)`) is how Kid's disassembly accesses object fields. When you see `move.w x_pos(a3),d0`, that's loading the 16-bit word at offset `x_pos` ($1A) from the GfxObject pointer in `a3`.

## 3. Condition Codes & Branch Conditions

### CCR Flags

| Flag | Name | Set when |
|------|------|----------|
| X | Extend | Same as C, but only set by arithmetic (not shifts/logic) |
| N | Negative | MSB of result is 1 |
| Z | Zero | Result is zero |
| V | Overflow | Signed overflow occurred |
| C | Carry | Unsigned overflow or borrow |

### Branch Conditions

After `CMP.x src,dst` (computes `dst - src` internally):

| Mnemonic | Condition | Flags | Unsigned meaning | Signed meaning |
|----------|-----------|-------|------------------|----------------|
| BEQ | Equal | Z | dst == src | dst == src |
| BNE | Not equal | !Z | dst != src | dst != src |
| BCC (BHS) | Carry clear | !C | dst >= src | - |
| BCS (BLO) | Carry set | C | dst < src | - |
| BHI | High | !C & !Z | dst > src | - |
| BLS | Low or same | C \| Z | dst <= src | - |
| BGE | Greater or equal | N==V | - | dst >= src |
| BGT | Greater than | N==V & !Z | - | dst > src |
| BLE | Less or equal | N!=V \| Z | - | dst <= src |
| BLT | Less than | N!=V | - | dst < src |
| BPL | Plus | !N | - | Result positive |
| BMI | Minus | N | - | Result negative |
| BVC | Overflow clear | !V | - | No overflow |
| BVS | Overflow set | V | - | Overflow |

After `TST.x operand` (computes `operand - 0`):
- `BEQ` = operand is zero
- `BNE` = operand is non-zero
- `BMI` = operand is negative (MSB set)
- `BPL` = operand is positive or zero

**DBcc pattern:** `dbf d0,loop` = decrement d0, branch if d0 != -1. `DBF` (false) always loops — it's the standard counted loop.

## 4. Fixed-Point Arithmetic (FP16.16)

Kid Chameleon stores positions and velocities as 32-bit fixed-point with 16 integer bits and 16 fractional bits.

| Hex value | Decimal | Meaning |
|-----------|---------|---------|
| $00010000 | 65536 | 1.0 pixel |
| $00008000 | 32768 | 0.5 pixel |
| $00000100 | 256 | ~0.004 pixel |
| $FFFF0000 | -65536 | -1.0 pixel |
| $FFFF8000 | -32768 | -0.5 pixel |

### Common ASM patterns

```asm
; Extract integer part (pixel position)
move.l  x_pos(a3),d0
swap    d0              ; d0.w now has integer part

; Add subpixel velocity
addi.l  #$8000,y_vel(a3)  ; add 0.5 px/frame to Y velocity

; Multiply by power of 2
lsl.l   #4,d0           ; multiply by 16

; Divide by power of 2
asr.l   #4,d0           ; signed divide by 16 (arithmetic shift preserves sign)
lsr.l   #4,d0           ; unsigned divide by 16
```

**Position** is stored as unsigned u32 (no negative positions in level space).
**Velocity** is stored as signed i32 (can move in either direction).

## 5. Kid Chameleon Register Conventions

These conventions apply throughout the Kid Chameleon disassembly:

| Register | Usage | Notes |
|----------|-------|-------|
| a3 | Current GfxObject pointer | Always. Field access via `offset(a3)` |
| a4 | Kid's GfxObject pointer | For enemy tracking, distance calculations |
| a5 | Current Object pointer | Parent structure ($74 bytes) containing the GfxObject |
| a6 | VDP data port ($C00000) | During rendering/DMA routines |
| a7 (sp) | Stack pointer | Standard |
| d0-d6 | Scratch/working | Caller-save; freely modified |
| d7 | Working register | Commonly used for distance deltas, loop counters |

**Object ↔ GfxObject relationship:** Each Object (a5, $74 bytes) owns one or more GfxObjects (a3, $4C bytes). The Object holds the AI coroutine; the GfxObject holds position, velocity, animation, and collision state. Object stores registers d0-d3, a0-a3 at offsets $16-$36 during hibernation and restores them next frame.

## 6. Kid Chameleon Object Layout

### GfxObject Fields ($4C bytes per object, RAM at $FFFF2448)

| Offset | Name | Size | Purpose |
|--------|------|------|---------|
| $00 | next_ptr | long | Pointer to next GfxObject in linked list |
| $04 | - | long | Internal slot reference |
| $0C | object_ptr | long | Pointer back to parent Object (a5) |
| $10 | priority | byte | Sprite priority: 0=low, 1=high, 2=conditional |
| $11 | palette_line | byte | Which palette row (0-3) |
| $12 | is_uncompressed | byte | Flag: art is uncompressed (DMA) |
| $13 | is_visible | byte | Flag: whether to render |
| $14 | is_moving | byte | Flag: whether to apply velocity |
| $15 | is_animating | byte | Flag: whether to advance animation |
| **$16** | **x_direction** | **byte** | **0 = default facing, $FF = flipped** |
| $17 | - | byte | Flag |
| $18 | anim_finished | byte | Flag: animation complete |
| **$1A** | **x_pos** | **long** | **FP16.16 X position** |
| **$1E** | **y_pos** | **long** | **FP16.16 Y position** |
| $22 | sprite_offset | word | Offset from data index to current sprite |
| $24 | vram_tile | word | VRAM tile address for art |
| **$26** | **x_vel** | **long** | **FP16.16 X velocity (signed)** |
| **$2A** | **y_vel** | **long** | **FP16.16 Y velocity (signed)** |
| $2E | anim_ptr | long | Pointer to current animation frame |
| $32 | anim_timer | word | Animation countdown timer |
| **$38** | **collision_type** | **word** | **Collision event ID (see below)** |
| $3A | object_meta | word | Bits 0-7: object ID, bit F: berzerker recoil flag |
| $3C | has_level_collision | byte | Flag: execute level collision |
| $3D | has_kid_collision | byte | Flag: execute kid collision |
| $3E | vram_slot | word | Enemy type VRAM slot |
| **$40** | **enemy_level** | **word** | **Hit point base (HP tier)** |
| $42 | - | long | Y position relative to camera / shared walker state |
| **$44** | **current_hp** | **word** | **Current hit points** |

### Collision Event IDs (offset $38)

| Value | Event |
|-------|-------|
| $00 | No collision |
| $04 | Hit right wall |
| $08 | Hit left wall |
| $0C | Hit floor |
| $10 | Hit ceiling |
| $14 | On up-slope / |
| $18 | On down-slope \ |
| $1C | Hurt (bullet, sword, axe) |
| $20 | Kid is to the right |
| $24 | Kid is to the left |
| $28 | Kid is below |
| $2C | Kid is above (stomped) |

### x_direction Convention (CRITICAL)

**`x_direction` = $00 means default sprite orientation.**
**`x_direction` = $FF means flipped (mirrored).**

For enemies, VRAM art is stored facing **LEFT**. Therefore:
- `x_direction = 0` → enemy faces **LEFT** (default, unflipped)
- `x_direction = $FF` → enemy faces **RIGHT** (flipped)

For the player (Kid), VRAM art is stored facing **RIGHT**. Therefore:
- `x_direction = 0` → Kid faces **RIGHT** (default, unflipped)
- `x_direction = $FF` → Kid faces **LEFT** (flipped)

The ASM uses `st x_direction(a3)` to set $FF (flipped) and `sf x_direction(a3)` to set $00 (default).

### Key Global RAM Addresses

| Address | Name | Size | Purpose |
|---------|------|------|---------|
| $FFF806 | V_Int_counter | word | Frame count since game start |
| $FFF80C | Ctrl_1 | byte | Raw controller: SACBRLDU |
| $FFF80D | Ctrl_1_Held | byte | Buttons currently held |
| $FFF80E | Ctrl_1_Pressed | byte | Buttons newly pressed this frame |
| $FFF81C | Camera_X_pos | long | FP16.16 camera X |
| $FFF820 | Camera_Y_pos | long | FP16.16 camera Y |
| $FFF83E | Addr_NextFreeObjectSlot | long | Next free Object slot |
| $FFF842 | Addr_FirstObjectSlot | long | Head of Object linked list |
| $FFF84A | Number_Objects | word | Object count |
| $FFF85E | Addr_GfxObject_Kid | long | Pointer to Kid's GfxObject |
| $FFF89A | Level_width_pixels | word | Level width in pixels |
| $FFF8A0 | Level_height_pixels | word | Level height in pixels |
| $FFF8AA | Foreground_theme | word | Theme (also determines music) |
| $FFF8F2 | Addr_EnemyLayoutHeader | long | Enemy layout pointer |
| $FFFA06 | Number_of_Enemy | word | Enemy count (decremented on kill) |
| $FFFA2B | KidIsInvulnerable | byte | Invulnerability flag |
| $FFFC3E | Lives | word | Number of lives |
| $FFFC40 | Hit_points | word | Kid's current HP |
| $FFFC42 | Diamonds | word | Diamond count |
| $FFFC44 | Current_LevelID | word | Current level |
| $FFFC46 | Costume | word | Current helmet (0-9) |
| $FFFC4C | Score | long | Player score |

## 7. Reading Patterns & Common Idioms

### State Machine via Jump Table

Most enemies use a `routine` field as an index into a jump table:

```asm
; Dispatch based on routine index
        moveq   #0,d0
        move.b  routine(a5),d0       ; load state index
        move.w  off_XXXXX(pc,d0.w),d0  ; load relative offset from table
        jmp     off_XXXXX(pc,d0.w)     ; jump to handler

off_XXXXX:
        dc.w    State_Init - off_XXXXX
        dc.w    State_Idle - off_XXXXX
        dc.w    State_Attack - off_XXXXX
        dc.w    State_Death - off_XXXXX
```

### Player Distance Check

```asm
; Compare enemy X position to Kid's X position
        move.l  (Addr_GfxObject_Kid).w,a4  ; get Kid's GfxObject
        move.w  x_pos(a4),d7              ; Kid's X (integer part)
        sub.w   x_pos(a3),d7              ; d7 = kid_x - enemy_x
        bmi.s   .kid_is_left              ; negative => kid is to the left
        cmpi.w  #$20,d7                   ; within 32 pixels?
        blt.s   .kid_is_close
```

### Face Toward Player

```asm
; Set enemy facing toward Kid
        move.w  x_pos(a4),d4             ; Kid X
        cmp.w   x_pos(a3),d4            ; compare to enemy X
        blt.s   .face_left               ; Kid is left of enemy
        st      x_direction(a3)          ; face RIGHT ($FF = flipped)
        bra.s   .done
.face_left:
        sf      x_direction(a3)          ; face LEFT ($00 = default)
.done:
```

### Counted Loop (DBF)

```asm
        moveq   #7,d0                    ; loop 8 times (7 down to -1)
.loop:  move.w  (a0)+,(a1)+              ; copy word
        dbf     d0,.loop                 ; decrement d0, loop if >= 0
```

### Hibernation (Coroutine Yield)

Objects yield execution via hibernation, resuming next frame:

```asm
        jsr     (Hibernate_Object_1Frame).w  ; pause 1 frame, resume here next frame
        ; execution continues here next frame
```

### Collision Response Pattern

```asm
        move.w  collision_type(a3),d6
        beq.s   .no_collision            ; $00 = no event
        cmpi.w  #$1C,d6
        beq.s   .was_hurt               ; $1C = hurt by projectile
        cmpi.w  #$2C,d6
        beq.s   .was_stomped            ; $2C = kid above (stomped)
```

## 8. Pitfall Ledger (Hard-Won Lessons)

### Direction Inversion (commit 7a2746d)

`x_direction=0` means **default sprite orientation** (LEFT for enemies since VRAM art faces left). `$FF` means **flipped** (RIGHT for enemies). A previous implementation inverted this, treating 0=Right and $FF=Left. Always verify direction conventions against the ASM — trace `st`/`sf` usage on `x_direction` to confirm.

### _move / _tst / _cmp / _clr Macros

In the disassembly, `_move.w`, `_tst.b`, `_cmp.w`, `_clr.w` etc. are **NOT** different instructions. They are AS assembler macros (defined in `macros.asm`) that work around a zero-offset optimization in AS to produce byte-identical ROM output. Read them exactly as `move.w`, `tst.b`, `cmp.w`, `clr.w`.

### st and sf Instructions

`Scc` (Set on condition) instructions:
- `st dst` = Set True: writes **$FF** to destination (condition = True, always)
- `sf dst` = Set False: writes **$00** to destination (condition = False, always)

These are used extensively for boolean flags like `x_direction`. Don't confuse `st` with "store" — it means "set true" and writes $FF.

### moveq Sign Extension

`moveq #N,Dn` sign-extends the 8-bit immediate to 32 bits. `moveq #$FF,d0` sets d0 to `$FFFFFFFF` (-1), not `$000000FF`. This is used intentionally for setting flags to $FF.

### ADDX/SUBX Zero Flag

`ADDX` and `SUBX` only **clear** Z, never set it. Z starts set and is cleared if any partial result is non-zero — this enables multi-precision zero detection across chained operations. Don't assume Z=1 means the result was zero without checking the initial state.

### Word Operations on Address Registers

`move.w` to an address register (`movea.w`) **sign-extends** to 32 bits. `movea.w #$8000,a0` sets a0 to `$FFFF8000`, not `$00008000`. This is how the disassembly accesses RAM in the $FF0000-$FFFFFF range with absolute short addressing.

### bsr.s / bsr.w vs jsr

`bsr` (branch to subroutine) is PC-relative and used for nearby calls. `jsr` uses an effective address for far calls or computed targets. Both push the return address. Functionally identical — just different encoding.

## 9. Mega Drive Hardware Reference

### Memory Map

| Address Range | Size | Description |
|---------------|------|-------------|
| $000000-$3FFFFF | 4 MB | Cartridge ROM |
| $400000-$7FFFFF | 4 MB | Reserved / extra ROM / SRAM |
| $A00000-$A0FFFF | 64 KB | Z80 address space (sound CPU RAM) |
| $A10000-$A1001F | 32 B | I/O registers (controller ports) |
| $A11100 | 2 B | Z80 bus request |
| $A11200 | 2 B | Z80 reset |
| $C00000-$C00003 | 4 B | VDP data port (read/write) |
| $C00004-$C00007 | 4 B | VDP control port (read/write) |
| $C00008-$C0000F | 8 B | VDP HV counter, PSG |
| $C00011 | 1 B | PSG (SN76489) input |
| $FF0000-$FFFFFF | 64 KB | 68k work RAM |

### VDP Registers (Active Registers 0-23)

| Reg | Bits | Purpose |
|-----|------|---------|
| $00 | Mode Set 1 | H-interrupt enable, HV counter latch, others |
| $01 | Mode Set 2 | Display enable, V-interrupt enable, DMA enable, V30 mode |
| $02 | Plane A name table | Base address / $400 |
| $03 | Window name table | Base address / $400 |
| $04 | Plane B name table | Base address / $2000 |
| $05 | Sprite table | Base address / $200 |
| $07 | Background colour | Palette line + colour index |
| $0A | H-interrupt counter | Lines between H-interrupts |
| $0B | Mode Set 3 | H-scroll mode, V-scroll mode, ext interrupt |
| $0C | Mode Set 4 | H40/H32 mode, interlace, shadow/highlight |
| $0D | H-scroll data | Base address / $400 |
| $0F | Auto-increment | Bytes to add to VRAM address after each access |
| $10 | Plane size | Width and height of scroll planes |
| $11 | Window H position | Window left/right boundary |
| $12 | Window V position | Window top/bottom boundary |
| $13 | DMA length low | DMA transfer length (low byte) |
| $14 | DMA length high | DMA transfer length (high byte) |
| $15 | DMA source low | Source address bits 1-8 |
| $16 | DMA source mid | Source address bits 9-16 |
| $17 | DMA source high | Source bits 17-22 + DMA type (bits 6-7) |

### Sprite Attribute Table

80 sprites max (H40 mode), 8 bytes per entry at the address set in VDP register $05:

| Offset | Size | Content |
|--------|------|---------|
| +0 | word | Y position (screen + 128; 0 = off-screen) |
| +2 | byte | Size: bits 2-3 = H size, bits 0-1 = V size (0=8px, 1=16px, 2=24px, 3=32px) |
| +3 | byte | Link: index of next sprite (0 = end of list) |
| +4 | word | Pattern: priority (bit 15), palette (bits 13-14), V-flip (bit 12), H-flip (bit 11), tile index (bits 0-10) |
| +6 | word | X position (screen + 128; 0 = off-screen) |

Sprites form a linked list via the Link field. The VDP traverses starting from sprite 0.

Kid's sprite table is at `$FFFF0000` (Sprite_Table), DMA'd to VRAM each frame. First 10 slots ($00-$48) are reserved for HUD elements (ankh, lives, timer, diamonds). Dynamic sprites (Kid, enemies, objects) start from slot $50 onwards.

### DMA Modes

| Mode | Trigger | Description |
|------|---------|-------------|
| 68k to VRAM/CRAM/VSRAM | Reg $17 bit 7=1, bit 6=0 | Copy from 68k memory to VDP memory. Used for sprite table, palette, tile art |
| VRAM Fill | Reg $17 bit 7=1, bit 6=1 | Fill VRAM range with a value. Used for clearing |
| VRAM Copy | Reg $17 bits 7-6=11 | Copy within VRAM. Rarely used in Kid |

The `dma68kToVDP` macro in `macros.asm` sets up registers $13-$17 then writes the command word to trigger the transfer. 68k→VDP DMA **halts the 68k** during transfer (~205 bytes/scanline during VBlank, ~18 bytes/scanline during active display). VRAM Fill and VRAM Copy do **not** halt the 68k.

### Palette (CRAM)

4 palette lines x 16 colours = 64 colours total. Each colour is a word: `0000BBB0GGG0RRR0` (3 bits per channel, 9-bit colour). Palette buffer at `$FFFF4E58` (Palette_Buffer, $80 bytes) is DMA'd to CRAM each frame.

### Controller Input

Read from I/O port `$A10003` (port 1). The game processes this into:

| Address | Name | Format |
|---------|------|--------|
| $FFF80C | Ctrl_1 | Raw buttons: SACBRLDU |
| $FFF80D | Ctrl_1_Held | Buttons currently held down |
| $FFF80E | Ctrl_1_Pressed | Buttons newly pressed this frame |

Button bits: S=Start (7), A (6), C (5), B (4), R=Right (3), L=Left (2), D=Down (1), U=Up (0).

### Z80 Bus Protocol

Before accessing Z80 RAM ($A00000):
```asm
        move.w  #$100,(Z80_Bus_Request).l  ; request bus
.wait:  btst    #0,(Z80_Bus_Request).l     ; wait until granted
        bne.s   .wait
        ; ... access Z80 RAM ...
        move.w  #0,(Z80_Bus_Request).l     ; release bus
```

Kid uses the GEMS sound driver in Z80 RAM for music and SFX.

## 10. Source Navigation Guide

### Disassembly Structure (`.cache/upstream/kid-chameleon-disasm/`)

| Path | Content | Use for |
|------|---------|---------|
| `kid.asm` | Main file: includes all others, player physics, core engine | Player movement, collision, game loop |
| `code/enemy/*.asm` | One file per enemy family | Enemy AI, state machines, tracking |
| `variables.asm` | All RAM addresses + object field offsets | Field offset lookup |
| `constants.asm` | Game constants (IDs, enums, button masks) | Enemy IDs, music IDs, collision types |
| `macros.asm` | Data structure macros + VDP helpers | Understanding data layouts, DMA |
| `doc/ram_layout.txt` | Comprehensive RAM map with object formats | Detailed object field descriptions |
| `doc/formats.txt` | Level, enemy layout, compression formats | Understanding data structures |
| `doc/vram_layout.txt` | Video RAM tile organisation | Sprite art layout |
| `doc/sound.txt` | GEMS sound driver notes | Music/SFX communication |
| `level/` | Level data files | Level layouts, headers |
| `ingame/` | In-game screen data | HUD, menus |
| `scenes/` | Cutscene/intro data | Non-gameplay sequences |
| `sound/` | Sound data | Music and SFX banks |
| `theme/` | Theme art and mappings | Visual tileset data |

### Label Naming Conventions

| Pattern | Meaning | Example |
|---------|---------|---------|
| `loc_XXXXX` | Code location (auto-named) | `loc_3CD70` — a branch target |
| `sub_XXXXX` | Subroutine (auto-named) | `sub_36FF4` — a called function |
| `off_XXXXX` | Offset table (jump table / data) | `off_346EE` — enemy script table |
| `word_XXXXX` | Word data | `word_282E` — data table |
| `byte_XXXXX` | Byte data | `byte_B408` — per-form widths |
| `stru_XXXXX` | Structure / animation data | `stru_8BB4` — animation frames |
| `j_Name` | Jump thunk to named routine | `j_Load_GfxObjectSlot` |
| `Name` | Named routine (manually labelled) | `Kid_Transform`, `Hibernate_Object_1Frame` |

### Cross-Reference: ASM → Rust

| ASM routine/area | Rust file | Notes |
|------------------|-----------|-------|
| Player physics loop | `crates/kid-core/src/movement.rs` | ASM order preserved |
| Enemy AI families | `crates/kid-core/src/enemies.rs` | State enums map to ASM jump tables |
| Collision resolution | `crates/kid-core/src/movement.rs` | Floor, wall, slope, ceiling |
| Combat/damage | `crates/kid-core/src/combat.rs` | Death, invulnerability, knockback |
| Block interactions | `crates/kid-core/src/blocks.rs` | Prize, ghost, evanescent, shooter |
| Boot/title/menus | `crates/kid-core/src/boot.rs` | Game mode state machine |
| Physics tables | `crates/kid-core/src/physics.rs` | Speed tables, gravity constants |
| Sprite rendering | `crates/kidcli/src/native_player/sprites.rs` | Direction flip logic |
