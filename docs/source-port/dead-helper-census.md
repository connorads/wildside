# Dead-Helper Census

This file is a historical pointer for the old codebase-wide dead-helper census.
The original 2026-05-29 narrative was generated from `#[cfg_attr(not(test), allow(dead_code))]`
and `#[allow(dead_code)]` helper scans in `wildside-core`; it is no longer a
current source of counts, line numbers, or wiring status.

Regenerate the raw census when needed:

```bash
python3 scripts/dead_helper_census.py
```

Generated output is written to:

```text
.cache/audit/dead-helper-census.json
```

Use durable routing instead of the old prose:

- `mise run project-status` and `docs/source-port/backlog.md` for current open work.
- `docs/source-port/audits/` for maintained subsystem audits.
- `docs/source-port/backlog-archive.md` for dated issue evidence and rejected hypotheses
  (grep by `#NNN`).

Do not cite old census counts or `dead-helper-census.md:<line>` references. They were
snapshots of generated research and are stale after later live-wiring, attr cleanup, and
audit updates.
