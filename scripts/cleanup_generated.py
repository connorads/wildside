#!/usr/bin/env python3

from __future__ import annotations

import argparse
import os
import shutil
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
CACHE_ROOT = REPO_ROOT / ".cache"
TARGET_ROOT = REPO_ROOT / "target"

BALANCED_REMOVE_DIRS = (
    "debug",
    "diagnose",
    "captures",
    "cases",
    "tas-diff",
    "tas-verify",
    "view-tas",
    "oracle-probe",
    "sweep-compare",
    "gameplay-runs",
    "native-gameplay-runs",
    "native-gameplay-oracle-runs",
    "oracle-runs",
    "native-oracle-runs",
)

AGGRESSIVE_EXTRA_DIRS = (
    "oracles",
    "oracle-cache",
    "generated",
    "tooling",
)

RUN_DIR_RETENTION = {
    "sweep-runs": 10,
    "release-gate": 5,
}

GENERIC_RUN_DIR_RETENTION = 5


@dataclass(frozen=True)
class CleanupItem:
    path: Path
    reason: str


def repo_rel(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def path_size_bytes(path: Path) -> int:
    if not path.exists() and not path.is_symlink():
        return 0
    if path.is_symlink():
        return 0
    if path.is_file():
        return path.stat().st_size

    total = 0
    for root, dirs, files in os.walk(path):
        root_path = Path(root)
        for name in files:
            file_path = root_path / name
            if file_path.is_symlink():
                continue
            try:
                total += file_path.stat().st_size
            except FileNotFoundError:
                continue
        for name in dirs:
            dir_path = root_path / name
            if dir_path.is_symlink():
                continue
    return total


def format_bytes(num_bytes: int) -> str:
    if num_bytes < 1024:
        return f"{num_bytes} B"
    value = float(num_bytes)
    for unit in ("KiB", "MiB", "GiB", "TiB"):
        value /= 1024.0
        if value < 1024.0:
            return f"{value:.1f} {unit}"
    return f"{value:.1f} PiB"


def resolve_latest_target(run_root: Path) -> Path | None:
    latest = run_root / "latest"
    if not latest.exists() and not latest.is_symlink():
        return None
    try:
        return latest.resolve(strict=True)
    except FileNotFoundError:
        return None


def timestamp_dirs(run_root: Path) -> list[Path]:
    if not run_root.is_dir():
        return []
    return sorted(
        (
            child
            for child in run_root.iterdir()
            if child.is_dir() and child.name.isdigit()
        ),
        key=lambda path: int(path.name),
        reverse=True,
    )


def prune_run_dir(run_root: Path, keep: int) -> list[CleanupItem]:
    latest_target = resolve_latest_target(run_root)
    candidates = timestamp_dirs(run_root)
    preserved = {path.resolve() for path in candidates[:keep]}
    if latest_target is not None:
        preserved.add(latest_target)

    items: list[CleanupItem] = []
    for path in candidates:
        resolved = path.resolve()
        if resolved in preserved:
            continue
        items.append(
            CleanupItem(
                path=path,
                reason=f"old {repo_rel(run_root)} run outside retention={keep}",
            )
        )
    return items


def discover_generic_run_dirs(cache_root: Path) -> list[Path]:
    roots: list[Path] = []
    if not cache_root.is_dir():
        return roots
    for child in cache_root.iterdir():
        if not child.is_dir():
            continue
        if child.name in RUN_DIR_RETENTION:
            continue
        if not child.name.endswith("-runs"):
            continue
        if child.name.startswith("native-") or child.name.startswith("oracle-"):
            continue
        roots.append(child)
    return sorted(roots)


def plan_cache_cleanup(mode: str) -> list[CleanupItem]:
    items: list[CleanupItem] = []

    for name in BALANCED_REMOVE_DIRS:
        path = CACHE_ROOT / name
        if path.exists():
            items.append(CleanupItem(path=path, reason=f"disposable cache dir ({mode})"))

    for name, keep in RUN_DIR_RETENTION.items():
        items.extend(prune_run_dir(CACHE_ROOT / name, keep))

    for run_root in discover_generic_run_dirs(CACHE_ROOT):
        items.extend(prune_run_dir(run_root, GENERIC_RUN_DIR_RETENTION))

    if mode == "aggressive":
        for name in AGGRESSIVE_EXTRA_DIRS:
            path = CACHE_ROOT / name
            if path.exists():
                items.append(
                    CleanupItem(
                        path=path,
                        reason="expensive cache dir removed in aggressive mode",
                    )
                )

    unique: dict[Path, CleanupItem] = {}
    for item in items:
        unique[item.path] = item
    return sorted(unique.values(), key=lambda item: str(item.path))


def remove_path(path: Path) -> None:
    if path.is_symlink() or path.is_file():
        path.unlink(missing_ok=True)
        return
    if path.is_dir():
        shutil.rmtree(path)


def print_plan(items: list[CleanupItem], heading: str) -> int:
    total_bytes = 0
    print(heading)
    if not items:
        print("  nothing to remove")
        return 0
    for item in items:
        size_bytes = path_size_bytes(item.path)
        total_bytes += size_bytes
        print(f"  - {repo_rel(item.path)} ({format_bytes(size_bytes)}): {item.reason}")
    print(f"  total reclaimable: {format_bytes(total_bytes)}")
    return total_bytes


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prune generated repo artefacts with conservative defaults."
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the cleanup plan without deleting anything",
    )
    parser.add_argument(
        "--balanced",
        action="store_true",
        help="remove build artefacts and disposable caches while preserving expensive toolchains",
    )
    parser.add_argument(
        "--aggressive",
        action="store_true",
        help="also remove expensive caches such as oracle toolchains and generated assets",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="leave target/ untouched",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.balanced and args.aggressive:
        raise SystemExit("choose either --balanced or --aggressive, not both")
    mode = "aggressive" if args.aggressive else "balanced"

    build_items: list[CleanupItem] = []
    if not args.skip_build and TARGET_ROOT.exists():
        build_items.append(CleanupItem(path=TARGET_ROOT, reason="cargo build artefacts"))
    cache_items = plan_cache_cleanup(mode)

    print(f"Cleanup mode: {mode}")
    print_plan(build_items, "Build artefacts")
    print_plan(cache_items, "Cache artefacts")

    if args.dry_run:
        print("Dry run only; no files deleted.")
        return 0

    for item in build_items + cache_items:
        remove_path(item.path)

    print("Cleanup complete.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
