# Source-Port Completion Audit

> First committed 2026-05-18; last substantive update 2026-06-20.

Last audited: 2026-06-10 (criteria reworked in the autonomy demolition; phase
remains reopened per the 2026-05-25 conclusion below).

## Objective

Finish the Kid Chameleon source port for the current phase: single-player
boot-to-credits native play that is stable, reviewable, and converges on
oracle-backed accuracy.

This audit exists because green proxy signals are not sufficient on their own.
Gates and TAS replay closure are evidence, not the finish line. Only human
playtests close player-visible issues.

## Success Criteria

The source port is complete for this phase only when all of these hold:

| Criterion | Evidence required |
|---|---|
| No open P0/P1 backlog rows | `docs/source-port/backlog.md` table has no `[ ]` or `[~]` rows at P0/P1 |
| Gates green | `mise run verify-slice-fast`, `mise run verify-1p-playable`, `mise run release-gate`, and `mise run verify-tas-full` all pass |
| No rows awaiting human validation | No `[?]` or `[b]` rows remain. Each has been closed (`[x]`) or rejected (`[-]`) by a human playtest. The `[?]`-then-playtest cycle replaces the retired HTML review queue |

## Latest Audit Commands

```bash
mise run completion-audit          # project-status --fail-on-human-blockers
cargo run -q -p wildside-cli -- project-status
rg -n '^\| [0-9]+ \| \[( |~|\?|b)\]' docs/source-port/backlog.md
```

## Conclusion

The 2026-05-24 completion call is revoked by 2026-05-25 playtest feedback. The
audit over-weighted proxy gates and missed player-visible failures in intro,
audio, HUD/prize, wall-grab animation, enemy rendering, enemy death, Drips,
Berzerker side-block, and death/continue transitions. These are tracked as
backlog #160-#170. Treat the source port as reopened until those rows have
frame/audio-backed evidence, not only narrow unit or TAS-state proof.
