"""Catalogue failures must not silently remove unresolved issues."""

import json
from pathlib import Path
import tempfile
import unittest

import jev_audit


class AuditTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.docs = self.root / "docs/source-port"
        self.docs.mkdir(parents=True)
        (self.docs / "backlog.md").write_text("| 406 | [f] | P0 | engine | frozen | evidence |\n")
        (self.docs / "backlog-awaiting-human.md").write_text(
            "| 210 | [?] | P0 | render | boot | skip | title | evidence |\n")
        self.entry = dict(issue=210, readiness="ready_replay", active_path="title",
                          route="reconstruct Start inputs", evidence=["docs/source-port/backlog.md"],
                          next_action="Replay against oracle", scope="rapid Start")

    def write(self, entries):
        (self.docs / "jev-cases.json").write_text(json.dumps(entries))

    def test_joins_status_and_excludes_frozen(self):
        self.write([self.entry])
        rows = jev_audit.audit(self.root)
        self.assertEqual([row["issue"] for row in rows], [210])
        self.assertEqual(rows[0]["status"], "[?]")

    def test_missing_duplicate_stale_and_unbacked_entries_fail(self):
        for entries in ([], [self.entry, self.entry], [self.entry | {"issue": 406}],
                        [self.entry | {"evidence": ["absent.rs"]}],
                        [self.entry | {"readiness": "proven"}],
                        [self.entry | {"next_action": ""}]):
            with self.subTest(entries=entries):
                self.write(entries)
                with self.assertRaises(ValueError):
                    jev_audit.audit(self.root)

    def test_repository_catalogue_matches_live_lanes(self):
        self.assertEqual({row["issue"] for row in jev_audit.audit()}, set(jev_audit.live_rows(jev_audit.ROOT)))


if __name__ == "__main__":
    unittest.main()
