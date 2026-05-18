# Source-Port Completion Audit

Last audited: 2026-05-18

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
| Native 1P core path works | `cargo run -q -p kidcli -- project-status` reports `1p-playable (passed)` with no top open playability risks | 2026-05-18 audit: passed, no top open playability risks | Pass |
| Oracle-backed accuracy stays green | `project-status` reports `1p-accurate: passed`, backed by release-gate/sweep evidence | 2026-05-18 audit: passed | Pass |
| Published TAS frontier is closed | `project-status` reports `any-percent-published [complete]` through its end frame | 2026-05-18 audit: clean through frame 37759 | Pass |
| Source-port frontier has no agent-actionable task | `cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier` reports no ready implementation task, and `cargo run -q -p kidcli -- autonomous-loop --goal case-queue` has no reduced authoritative case | 2026-05-18 audit: source-port frontier reports verification lanes exhausted; case-queue is blocked with no reduced authoritative case after the `issue-143-level27-ik-smash-probe{,-2,-3,-4}` settled recordings were classified as oracle-bootstrap limitations and the `issue-143-published-route-probe` Sega-screen hitpoint false positive was fixed | Pass |
| Canonical backlog has no unresolved phase blockers | `docs/source-port/backlog.md` has no open `[?]`, `[~]`, or `[b]` P1 rows blocking 1P playability or visual parity | No unresolved P1 phase blockers remain after #143 and #146 validation | Pass |
| Human-only validation has been resolved | Saved recording/PNG evidence has been inspected by a human or deliberately deferred out of scope | No human-only validation rows remain | Pass |
| Human-only repro blockers have been resolved | Blocked rows have a human repro bundle or an explicit out-of-scope decision | #146 now has deterministic Level 2 death evidence proving recovery; no `[b]` repro blocker remains | Pass |

## Open Blockers

These are the current blockers to a completion claim:

| Backlog row | Type | Required next evidence |
|---|---|---|
| _none_ | | |

`human-validation-checklist.md` currently has no open rows.

## Latest Audit Commands

```bash
mise run completion-audit
cargo run -q -p kidcli -- project-status
cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier
cargo run -q -p kidcli -- autonomous-loop --goal case-queue
rg -n '^\| [0-9]+ \| \[( |~|\?|b)\]' docs/source-port/backlog.md
```

## Conclusion

The port is complete for the current source-port objective as of this audit:
the native/playability and oracle-backed gates pass, the published TAS frontier
is replay-matched, the autonomous queues report no ready implementation task,
and the canonical backlog has no unresolved phase blockers.
