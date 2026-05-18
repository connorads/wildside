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
| Source-port frontier has no agent-actionable task | `cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier` reports no ready implementation task, and `cargo run -q -p kidcli -- autonomous-loop --goal case-queue` has no reduced authoritative case | 2026-05-18 audit: source-port frontier is `status: stalled`; case-queue is blocked with no reduced authoritative case after `issue-143-knights-isle-probe` and `issue-146-level2-walk-right-probe` were classified as settled-recording oracle-bootstrap limitations | Pass |
| Canonical backlog has no unresolved phase blockers | `docs/source-port/backlog.md` has no open `[?]`, `[~]`, or `[b]` P1 rows blocking 1P playability or visual parity | Seven validation rows remain: #135, #136, #138, #139, #140, #143, #146 | Fail |
| Human-only validation has been resolved | Saved recording/PNG evidence has been inspected by a human or deliberately deferred out of scope | #135, #136, #138, #139, #140, #143, and #146 still require human validation | Fail |
| Human-only repro blockers have been resolved | Blocked rows have a human repro bundle or an explicit out-of-scope decision | #146 now has deterministic Level 2 death evidence; no `[b]` repro blocker remains | Pass |

## Open Blockers

These are the current blockers to a completion claim:

| Backlog row | Type | Required next evidence |
|---|---|---|
| #135 | visual validation | Human validates `.cache/recordings/issue-135-level1-background/verify/native/000000_start.png` against the original game |
| #136 | visual validation | Human validates prize emergence PNGs in `.cache/recordings/issue-136-prize-emergence/verify/native/` |
| #138 | playability validation | Human confirms the ROM-faithful 66-frame enemy-contact re-hit window is acceptable in live play |
| #139 | visual validation | Human validates rubber-block PNGs in `.cache/recordings/issue-139-rubber-block-window/verify/native/` |
| #140 | visual validation | Human validates prize-block-hit PNGs in `.cache/recordings/issue-140-prize-block-window/verify/native/` |
| #143 | playability validation | Human accepts scripted `issue-143-level27-ik-smash-probe-5` Iron Knight/block-break artefacts as sufficient, or records a preferred live Heavy Man route |
| #146 | playability validation | Human accepts the scripted Level 2 death transition as non-blocking or records a different route that still hangs |

Use `human-validation-checklist.md` for the exact verifier commands, PNGs, and
pass/fail decisions for these rows.

## Latest Audit Commands

```bash
mise run completion-audit
cargo run -q -p kidcli -- project-status
cargo run -q -p kidcli -- autonomous-loop --goal source-port-frontier
cargo run -q -p kidcli -- autonomous-loop --goal case-queue
rg -n '^\| [0-9]+ \| \[( |~|\?|b)\]' docs/source-port/backlog.md
```

## Conclusion

The port is not complete as of this audit. Automation supports the current
native/playability and oracle-backed status, and the autonomous case queue has
no reduced authoritative case left to patch. The remaining human validation
blockers are explicit phase blockers in the canonical backlog.
