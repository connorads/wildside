# Human Validation Checklist

Use this checklist to close the remaining source-port completion blockers that
cannot be resolved by another automated sweep or TAS frontier pass.

Before changing any backlog row, rerun the matching verifier command and inspect
the listed artefacts. If the human decision is "pass", update
`docs/source-port/backlog.md` with the reviewer, date, and exact artefacts used.
If the decision is "fail", keep the row open and add the observed mismatch or
repro notes.

To verify every currently referenced bundle before review:

```bash
mise run verify-human-validation
```

This gate is intentionally native-only. It proves the saved bundles still replay
deterministically in the native engine, then leaves the remaining visual and
playability calls to a human reviewer; it must not be used as oracle-parity
evidence.

To run the completion audit gate, which must fail until the rows below are
closed or deliberately deferred:

```bash
mise run completion-audit
```

To verify all referenced bundles and build a local HTML review page for the
exact PNGs/transcripts in this checklist:

```bash
mise run human-validation-review
```

Open `.cache/human-validation/index.html` after generation.

## Visual Validation Rows

| Backlog row | Verify command | Inspect | Pass decision |
|---|---|---|---|
| _none_ | | | |

## Playability Validation Rows

| Backlog row | Verify command | Inspect | Pass decision |
|---|---|---|---|
| _none_ | | | |
