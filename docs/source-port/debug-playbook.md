# TAS/Parity Debugging Playbook

Anti-loop protocol for debugging frame-level divergences between native and oracle.

Bootstrap first: read the NOW block at the top of `docs/source-port/backlog.md`
and grep `backlog-archive.md` for the issue's `#NNN`. Rejected hypotheses and
prior probe windows are recorded there, so you do not restart from stale prose.

## Step 1: Find the divergence frame

```bash
cargo run --release -p wildside-cli -- verify-tas \
  --scenario tests/data/tas/kc_any_percent_2024/scenario.json \
  --frame-end 1000 --bisect --frontier-kind enemy
```

`--bisect` uses per-frame state comparison to find the first divergent frame. Use
`--frontier-kind enemy` when hidden enemy drift may arrive before Kid/camera drift;
reserve `gameplay` for frontiers where gameplay/global state is already known to be first.

## Step 2: Probe relevant RAM addresses

```bash
cargo run --release -p wildside-cli -- oracle-probe \
  --scenario tests/data/tas/kc_any_percent_2024/scenario.json \
  --frame-start 950 --frame-end 960 \
  --watches 0xFF24FE:u32,0xFF2502:i32,0xFFA056:u16
```

Outputs CSV to stdout; pipe to file or view directly. Watch addresses come from
`variables.asm` or `doc/ram_layout.txt`.

## Step 3: Dump and diff WRAM to discover which addresses matter

```bash
# Dump full 64KB WRAM at consecutive frames
cargo run --release -p wildside-cli -- oracle-probe \
  --scenario tests/data/tas/kc_any_percent_2024/scenario.json \
  --frame-start 954 --frame-end 956 \
  --wram-dump-frames 954,955,956

# Diff two frames to see what changed
cargo run --release -p wildside-cli -- ram-diff \
  .cache/oracle-probe/oracle/wram_frame_954.bin \
  .cache/oracle-probe/oracle/wram_frame_955.bin --annotate
```

## Step 4: Read ASM only for the specific routine

Once steps 1-3 have identified the diverging address and frame, read the
ASM routine that touches it. Use source precedence from CLAUDE.md:

1. `variables.asm` to identify the field
2. `kid.asm` to find which routine writes it
3. `code/*.asm` for the routine body

## Step 5: Compare physics orders if the divergence is position/velocity

```bash
cargo run --release -p wildside-cli -- sweep-compare --levels 0-68 --tier state
```

Shows which levels improve or regress if physics sub-step order is swapped.

## Anti-patterns

- **Skipping loop bootstrap**: run `mise run project-status` and follow `docs/source-port/loop.md` before reading broad backlog/roadmap prose
- **Manual frame counter tweaking**: use `--bisect` instead of guessing
- **Hypothesis without data**: always probe before reading ASM
- **Unregistered debug payloads**: every new `debug_fields` / `gameplay_debug_fields` key must be added to `docs/source-port/debug-field-registry.yaml`
- **Letting useful debug fields stay ad hoc**: if a field drives more than one session, mark it as a `candidate`, add it to `candidate_promotions`, and run `mise run audit-debug-fields`
- **>30 min without running a probe**: stop and build measurement tooling
- **Fear of breaking sweep**: use `sweep-compare` to preview impact
- **Circular ASM reading**: if you have read the same routine 3 times without
  new data, you need a new probe, not another re-read
- **Probe-only churn**: after 2 consecutive probe/docs-only commits on the same frontier, add instrumentation or narrow the scenario before another gameplay patch
