# Source-Port Completion Audit

## Objective

Deliver the single-player release candidate defined by
[the completion campaign](../completion-campaign.md), across every confirmed
playable level, branch, form and boss. Frozen timing debt stays disclosed and
ratcheted. Verified objective bugs may close automatically. Subjective and real
output-device checks remain for one final human playtest package.

Gates and TAS replays are evidence, not the finish line. The package does not
claim human validation or `human-play-stable` status before the user plays it.

## Success criteria

| Criterion | Evidence required |
|---|---|
| Complete coverage | ROM-derived inventory and natural entry/traversal/exit evidence for every playable level and progression branch; live form, boss, enemy and mechanic coverage |
| Objective issues resolved | Every in-scope objective report across all backlog lanes has a verified resolution under the campaign evidence contract; no unresolved objective bug is relabelled as subjective |
| Gates green | `mise run verify-slice-fast`, `mise run verify-1p-playable`, `mise run release-gate`, `mise run verify-tas-full` and the corrected full sweep pass; failures and unverified coverage remain visible |
| Frozen debt bounded | #406/#409/#416 remain within their documented ratchets; unrelated defects are not exempt |
| Review package ready | Reproducible build, tested revision, evidence report and concrete routes for remaining subjective/device checks |

## Audit commands

```bash
mise run project-status
mise run jev-audit
mise run completion-audit
```

`completion-audit` runs `project-status --fail-on-human-blockers`. It is a strict
unresolved-lane check, not the whole campaign verdict. It can remain red when
only genuine final human checks remain. Inspect each remaining row against the
criteria above; never treat a broad failure as permission to ignore objective bugs.

## Conclusion

The 2026-05-24 completion call is revoked by 2026-05-25 playtest feedback. The
audit over-weighted proxy gates and missed player-visible failures in intro,
audio, HUD/prize, wall-grab animation, enemy rendering, enemy death, Drips,
Berzerker side-block, and death/continue transitions. These are tracked as
backlog #160-#170. Treat the source port as reopened until those rows have
frame/audio-backed evidence, not only narrow unit or TAS-state proof.
