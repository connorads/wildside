# Source-Port Completion Audit

Last audited: 2026-05-25 (current phase reopened)

## Objective

Finish the Kid Chameleon source port for the current phase: single-player
boot-to-credits native play that is stable, reviewable, and converges on
oracle-backed accuracy.

This audit exists because green proxy signals are not sufficient by themselves.
`1p-playable`, `1p-accurate`, TAS replay closure, and native recording
verification are evidence, not the finish line.

## Success Criteria

The source port can be called complete for this phase only when all of these are
true:

| Criterion | Evidence required | Current evidence | Status |
|---|---|---|---|
| Native 1P core path works | `cargo run -q -p kidcli -- project-status` reports `1p-playable (passed)` with no top open playability risks | 2026-05-24 audit: passed, no top open playability risks | Pass |
| Oracle-backed accuracy stays green | `project-status` reports `1p-accurate: passed`, backed by release-gate/sweep evidence | 2026-05-24 audit: passed; `.cache/release-gate/1p-accurate/latest.json` reports `"passed": true` | Pass |
| Published TAS frontier is closed | `project-status` reports `any-percent-published [complete]` through its end frame | 2026-05-24 audit: clean through frame 37759 | Pass |
| Source-port frontier has no agent-actionable task | `cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier` reports no ready implementation task, and `cargo run -q -p kidcli -- autonomous-loop --goal case-queue` has no reduced authoritative case | 2026-05-24 audit: source-port frontier reports verification lanes exhausted; case queue reports no reduced authoritative case; `project-status` reports no ready implementation item | Pass |
| Canonical backlog has no unresolved phase blockers | `docs/source-port/backlog.md` has no open `[?]`, `[~]`, or `[b]` P1 rows blocking 1P playability or visual parity | 2026-05-24 audit: backlog has no open `[ ]`, `[~]`, `[?]`, or `[b]` rows; no top open playability or visual risks remain | Pass |
| Human-only validation has been resolved | Saved recording/PNG evidence has been inspected by a human or deliberately deferred out of scope | No human-only validation rows remain | Pass |
| Human-only repro blockers have been resolved | Blocked rows have a human repro bundle or an explicit out-of-scope decision | #146 now has deterministic Level 2 death evidence proving recovery; no `[b]` repro blocker remains | Pass |

## Open Blockers

None for the current phase. `human-validation-checklist.md` currently has no
open rows.

## Latest Audit Commands

```bash
mise run completion-audit
cargo run -q -p kidcli -- project-status
cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier
cargo run -q -p kidcli -- autonomous-loop --goal case-queue
rg -n '^\| [0-9]+ \| \[( |~|\?|b)\]' docs/source-port/backlog.md
```

## Conclusion

The 2026-05-24 completion call is revoked by 2026-05-25 playtest feedback.
The audit over-weighted proxy gates and missed player-visible intro, audio,
HUD/prize, wall-grab animation, enemy-rendering, enemy-death, Drips, Berzerker
side-block, and death/continue transition failures. These are now tracked as
backlog #160-#170. Treat the source port as reopened until those rows have
frame/audio-backed evidence, not only narrow unit or TAS-state proof.
