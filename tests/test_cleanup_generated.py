import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent

sys.path.insert(0, str(REPO_ROOT / "scripts"))

import cleanup_generated


class CleanupGeneratedTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp_dir.cleanup)
        self.repo_root = Path(self.temp_dir.name)
        self.cache_root = self.repo_root / ".cache"
        self.target_root = self.repo_root / "target"
        self.cache_root.mkdir()
        self.target_root.mkdir()

        self.original_repo_root = cleanup_generated.REPO_ROOT
        self.original_cache_root = cleanup_generated.CACHE_ROOT
        self.original_target_root = cleanup_generated.TARGET_ROOT
        cleanup_generated.REPO_ROOT = self.repo_root
        cleanup_generated.CACHE_ROOT = self.cache_root
        cleanup_generated.TARGET_ROOT = self.target_root

    def tearDown(self) -> None:
        cleanup_generated.REPO_ROOT = self.original_repo_root
        cleanup_generated.CACHE_ROOT = self.original_cache_root
        cleanup_generated.TARGET_ROOT = self.original_target_root

    def write_file(self, path: Path, contents: str = "x") -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents, encoding="utf-8")

    def make_run_dir(self, root: Path, name: str) -> Path:
        path = root / name
        path.mkdir(parents=True, exist_ok=True)
        self.write_file(path / "report.json", name)
        return path

    def test_prune_run_dir_keeps_latest_symlink_target_and_newest_dirs(self) -> None:
        sweep_root = self.cache_root / "sweep-runs"
        sweep_root.mkdir()
        kept_by_latest = self.make_run_dir(sweep_root, "1001")
        self.make_run_dir(sweep_root, "1002")
        self.make_run_dir(sweep_root, "1003")
        self.make_run_dir(sweep_root, "1004")
        (sweep_root / "latest").symlink_to(kept_by_latest.name)

        removals = cleanup_generated.prune_run_dir(sweep_root, keep=2)
        removed = {item.path.name for item in removals}

        self.assertEqual(removed, {"1002"})

    def test_balanced_plan_preserves_expensive_caches(self) -> None:
        self.write_file(self.target_root / "debug" / "bin", "bin")
        self.write_file(self.cache_root / "debug" / "capture.bin", "capture")
        self.write_file(self.cache_root / "oracles" / "helper" / "bin", "oracle")
        self.write_file(self.cache_root / "oracle-cache" / "index", "cache")
        self.write_file(self.cache_root / "generated" / "manifest.json", "{}")

        items = cleanup_generated.plan_cache_cleanup("balanced")
        paths = {item.path.relative_to(self.repo_root) for item in items}

        self.assertIn(Path(".cache/debug"), paths)
        self.assertNotIn(Path(".cache/oracles"), paths)
        self.assertNotIn(Path(".cache/oracle-cache"), paths)
        self.assertNotIn(Path(".cache/generated"), paths)

    def test_aggressive_plan_removes_expensive_caches(self) -> None:
        self.write_file(self.cache_root / "tooling" / "bin", "tooling")
        self.write_file(self.cache_root / "oracles" / "helper" / "bin", "oracle")
        self.write_file(self.cache_root / "oracle-cache" / "index", "cache")
        self.write_file(self.cache_root / "generated" / "manifest.json", "{}")

        items = cleanup_generated.plan_cache_cleanup("aggressive")
        paths = {item.path.relative_to(self.repo_root) for item in items}

        self.assertIn(Path(".cache/tooling"), paths)
        self.assertIn(Path(".cache/oracles"), paths)
        self.assertIn(Path(".cache/oracle-cache"), paths)
        self.assertIn(Path(".cache/generated"), paths)

    def test_remove_path_deletes_directory_tree(self) -> None:
        self.write_file(self.target_root / "debug" / "deps" / "kid", "kid")

        cleanup_generated.remove_path(self.target_root)

        self.assertFalse(self.target_root.exists())


if __name__ == "__main__":
    unittest.main()
