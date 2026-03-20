# TAS/Parity Debugging Playbook

Anti-loop protocol for debugging frame-level divergences between native and oracle.

## Step 1: Find the divergence frame

```bash
cargo run --release -p kidcli -- verify-tas \
  --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json \
  --frame-end 1000 --bisect
```

`--bisect` uses per-frame CRC comparison to binary-search for the first divergent frame.

## Step 2: Probe relevant RAM addresses

```bash
cargo run --release -p kidcli -- oracle-probe \
  --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json \
  --frame-start 950 --frame-end 960 \
  --watches 0xFF24FE:u32,0xFF2502:i32,0xFFA056:u16
```

Outputs CSV to stdout — pipe to file or view directly. Watch addresses come from
`variables.asm` or `doc/ram_layout.txt`.

## Step 3: Dump and diff WRAM if you need to discover which addresses matter

```bash
# Dump full 64KB WRAM at consecutive frames
cargo run --release -p kidcli -- oracle-probe \
  --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json \
  --frame-start 954 --frame-end 956 \
  --wram-dump-frames 954,955,956

# Diff two frames to see what changed
cargo run --release -p kidcli -- ram-diff \
  .cache/oracle-probe/oracle/wram_frame_954.bin \
  .cache/oracle-probe/oracle/wram_frame_955.bin --annotate
```

## Step 4: Read ASM only for the specific routine

Only after steps 1-3 have identified the diverging address and frame, read the
ASM routine that touches it. Use source precedence from CLAUDE.md:

1. `variables.asm` to identify the field
2. `kid.asm` to find which routine writes it
3. `code/*.asm` for the routine body

## Step 5: Compare physics orders if the divergence is position/velocity

```bash
cargo run --release -p kidcli -- sweep-compare --levels 0-68 --tier state
```

Shows which levels improve/regress if physics sub-step order is swapped.

## Anti-patterns

- **Manual frame counter tweaking** — use `--bisect` instead of guessing
- **Hypothesis without data** — always probe before reading ASM
- **>30 min without running a probe** — stop and build measurement tooling
- **Fear of breaking sweep** — use `sweep-compare` to preview impact
- **Circular ASM reading** — if you've read the same routine 3 times without
  new data, you need a new probe, not another re-read
