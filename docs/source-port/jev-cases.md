# Jev bug cases

`mise run jev-audit` joins `jev-cases.json` to the live backlog. `--json`
prints the joined records. Frozen lanes are excluded. Backlog status and
priority remain authoritative; the catalogue never promotes human validation.

Readiness describes the next experiment, not a passing result:

- `ready_replay`: inputs exist or can be reconstructed. Current execution and
  oracle comparison are still required.
- `needs_route`: no natural input stream covers the reported symptom.
- `needs_comparison`: a route or probe exists but its reference, alignment or
  coverage does not yet establish the claim.
- `inactive_path`: the historical defect is absent from, or unproven on, the
  current runtime path. A failing helper test cannot establish live coverage.
- `human_device`: real output-device evidence is required.

Evidence references identify repository files. Archive `#issue-N` references
mean search for `#N`; they are lookup keys, not generated Markdown anchors.
Cache paths in route descriptions are leads and may be stale or absent.
Each case names its exact scope and a next action. Broad issue summaries may
need several cases before the whole issue is covered.

The private catalogue is reversible and independent of recording formats.
A shared runner can consume the existing recording/oracle pipeline without
adding another player or changing the public CLI. A standalone browser bot
would duplicate controller and replay handling. Manually running each case
avoids new machinery but cannot consistently enforce evidence or cost limits.

The pilot covers one shared #210/#254 Start-spam case, the #327 same-level
teleporter, and #370 natural Dragon-breath death followed by ordinary reload.
Cross-level teleports and the continue branch require separate evidence.
An attempt counts only when the encounter occurs, the current output agrees
with the relevant oracle evidence, and a verified negative control fails the
same check. Synthetic controls are labelled; they do not reproduce a historical
build. Inconclusive comparisons and blocked routes never count as proven cases.
