# Gameplay Loop

## Bootstrap
1. Run `kidcli asm-coverage --uncovered` to see what's not covered
2. Read `docs/source-port/asm-coverage.yaml` — pick the highest-priority uncovered routine
3. If no priority annotations exist, pick the first uncovered sub_ in the highest-gap subsystem

## Work
4. Read the ASM routine (50-100 lines around the label in kid.asm or code/*.asm)
5. Compare to native implementation (or confirm it's missing)
6. Write a test citing the ASM label — see it fail
7. Implement — see it pass
8. `mise run verify-slice-fast` as regression gate

## Commit
9. Update coverage: `kidcli asm-coverage --update`
10. Commit (MUST touch crates/kid-core/src/ or crates/kid-data/src/)
11. Commit message MUST cite ASM label

## Rules
- Every commit modifies game source code. No exceptions.
- No handoff notes. The coverage map + git log is the state.
- Sweep is a gate, not the work. Never "harden" sweep.
- If stuck after reading ASM + oracle, mark routine `blocked` with reason, move to next.
