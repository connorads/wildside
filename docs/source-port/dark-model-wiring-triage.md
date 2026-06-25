# Dark-Model Wiring Triage

This file is a historical pointer for the old dark-model wiring triage. The original
generated narrative compared test-only ASM helper models with live runtime fallbacks, but
it is not current loop input and must not be used as a fresh source of backlog rows.

Current disposition:

- Suspected wiring gaps must be verified through the live backlog, maintained audits, and
  oracle/probe evidence before implementation.
- Redundant model helpers should not be wired blindly. If a live path is already
  oracle-clean, either leave the helper as documentation or delete it in a focused cleanup.
- Use `mise run project-status` for the next actionable item, and grep
  `docs/source-port/backlog-archive.md` by issue number for dated evidence.

Corrections since the generated triage:

- Diamond Power shared object list/pool is live-wired by later #316 work. The shared
  `sub_3F586` / `sub_3F596` / `sub_3F5BC` / `sub_3F5E8` / `sub_3F63C` list, raw payload RAM,
  and `DiamondPower_CompileSprites sub_3F798` sprite compiler are covered in
  `backlog-archive.md` under `#316`. Do not file a new DP-pool backlog row from this
  historical triage.
- Boss state-machine work is tracked through `docs/source-port/audits/bosses-fidelity-audit.md`
  and the object-active oracle harness lineage (`#205` / boss audit rows, including #301-style
  object-active evidence). Do not duplicate rows from the old `combat-bosses` triage item.
