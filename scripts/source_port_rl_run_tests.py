"""Unit tests for the operator-unblock renderer in source_port_rl_run.

The renderer reads repo state at call time and chooses hints accordingly;
these tests pin its branches so future changes don't silently regress to
the hardcoded list it replaced.
"""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from source_port_rl_run import (
    MISSING_ARTEFACT_REGEN_HINT,
    asm_coverage_has_uncovered_sub_labels,
    backlog_has_unpromoted_open_item,
    derive_operator_unblocks,
)


class AsmCoverageHeuristic(unittest.TestCase):
    def test_full_coverage_returns_false(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "asm-coverage.yaml"
            path.write_text(
                "summary:\n  sub_coverage_pct: 100.0\n",
                encoding="utf-8",
            )
            self.assertFalse(asm_coverage_has_uncovered_sub_labels(path))

    def test_partial_coverage_returns_true(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "asm-coverage.yaml"
            path.write_text(
                "summary:\n  sub_coverage_pct: 87.5\n",
                encoding="utf-8",
            )
            self.assertTrue(asm_coverage_has_uncovered_sub_labels(path))

    def test_missing_file_returns_false(self) -> None:
        self.assertFalse(
            asm_coverage_has_uncovered_sub_labels(Path("/no/such/path"))
        )


class BacklogHeuristic(unittest.TestCase):
    def test_open_item_without_marker_returns_true(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "backlog.md"
            path.write_text(
                "| 100 | [ ] | P0 | render | Some bug | Verify |\n",
                encoding="utf-8",
            )
            self.assertTrue(backlog_has_unpromoted_open_item(path))

    def test_open_item_with_marker_returns_false(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "backlog.md"
            path.write_text(
                "| 100 | [ ] | P0 | render | Some bug. Root cause identified: foo | Verify |\n",
                encoding="utf-8",
            )
            self.assertFalse(backlog_has_unpromoted_open_item(path))

    def test_only_closed_items_returns_false(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "backlog.md"
            path.write_text(
                "| 001 | [x] | P0 | render | Already done | Verified |\n",
                encoding="utf-8",
            )
            self.assertFalse(backlog_has_unpromoted_open_item(path))


class DeriveOperatorUnblocks(unittest.TestCase):
    def _make_repo(self, tmp: str, *, coverage: str, backlog: str) -> Path:
        repo = Path(tmp)
        (repo / "docs/source-port").mkdir(parents=True, exist_ok=True)
        (repo / "docs/source-port/asm-coverage.yaml").write_text(coverage, encoding="utf-8")
        (repo / "docs/source-port/backlog.md").write_text(backlog, encoding="utf-8")
        return repo

    def test_missing_state_sweep_renders_regen_command(self) -> None:
        with TemporaryDirectory() as tmp:
            repo = self._make_repo(
                tmp,
                coverage="summary:\n  sub_coverage_pct: 100.0\n",
                backlog="| 001 | [x] | P0 | render | Done | Verified |\n",
            )
            hints = derive_operator_unblocks(
                {"missing_artefact": "state_sweep"},
                repo_root=repo,
            )
            self.assertEqual(hints[0], MISSING_ARTEFACT_REGEN_HINT["state_sweep"])
            # Fallback rerun is always last.
            self.assertIn("Rerun `rl`", hints[-1])

    def test_clean_state_only_emits_fallback(self) -> None:
        with TemporaryDirectory() as tmp:
            repo = self._make_repo(
                tmp,
                coverage="summary:\n  sub_coverage_pct: 100.0\n",
                backlog="| 001 | [x] | P0 | render | Done | Verified |\n",
            )
            hints = derive_operator_unblocks({}, repo_root=repo)
            # No missing artefact, no all-exhausted, full coverage, no
            # unpromoted open backlog → only the rerun fallback.
            self.assertEqual(len(hints), 1)
            self.assertIn("Rerun `rl`", hints[0])

    def test_all_exhausted_target_triggers_pv_broaden_hint(self) -> None:
        with TemporaryDirectory() as tmp:
            repo = self._make_repo(
                tmp,
                coverage="summary:\n  sub_coverage_pct: 100.0\n",
                backlog="| 001 | [x] | P0 | render | Done | Verified |\n",
            )
            hints = derive_operator_unblocks(
                {"delegated_target": "all-exhausted"},
                repo_root=repo,
            )
            self.assertTrue(
                any("proactive-verification-config.yaml" in hint for hint in hints),
                f"PV broaden hint missing from {hints}",
            )

    def test_release_gate_uses_dedicated_hint(self) -> None:
        with TemporaryDirectory() as tmp:
            repo = self._make_repo(
                tmp,
                coverage="summary:\n  sub_coverage_pct: 100.0\n",
                backlog="| 001 | [x] | P0 | render | Done | Verified |\n",
            )
            hints = derive_operator_unblocks(
                {"missing_artefact": "release_gate"},
                repo_root=repo,
            )
            self.assertEqual(hints[0], MISSING_ARTEFACT_REGEN_HINT["release_gate"])


if __name__ == "__main__":
    unittest.main()
