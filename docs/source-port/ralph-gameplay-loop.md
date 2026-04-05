# Gameplay Loop

## Bootstrap
1. Read `docs/source-port/backlog.md` and pick the highest-priority open `1p-playable` or `visual-parity` issue that does not yet have canonical evidence.
2. Check `.cache/recordings/` and `.cache/cases/` for an existing recording slug or reduced case tied to that issue.
3. If no evidence exists, capture it first with `play-native --record <slug>` and, for visual issues, `capture-frames`.
4. Only when no open evidence-driven issue is ready should you run `kidcli asm-coverage --uncovered` and use `docs/source-port/asm-coverage.yaml` as the fallback queue.

## Work
5. Turn the chosen issue into a reproducible artefact: recording slug, reduced case, oracle compare window, or named frame-capture checkpoint.
6. Read the ASM routine that owns the confirmed behaviour before patching.
7. Write the narrowest failing regression that proves the bug.
8. Implement the fix and re-run the minimum gate named by the backlog item.
9. Always finish with `mise run verify-slice-fast`; add `mise run verify-1p-playable` or frame/oracle checks when the issue requires them.

## Commit
10. Update `docs/source-port/backlog.md` with the new evidence handle, gate, and next verification commands.
11. If ASM-backed code changed, refresh citations with `kidcli asm-coverage --update`.
12. Commit only after the chosen gate is green and the backlog entry explains the new evidence.

## Rules
- Backlog truth beats raw ASM coverage. Do not skip an open player-visible issue just because another subsystem has more uncited labels.
- TAS closure is replay evidence, not completion.
- Visual issues are not done without frame-backed evidence.
- If a report cannot be reproduced, record that explicitly in the backlog instead of leaving it vague.
- Use ASM coverage as a discovery aid and fallback queue, not the lead selector.
