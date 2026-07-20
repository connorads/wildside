#!/usr/bin/env python3
"""Build a host-neutral Wildside static directory from explicit inputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path
from urllib.parse import urlparse

CANONICAL_SHA1 = "28b904000b2863b6760531807760b571f1a5fc1d"
SHELL_FILES = (
    "index.html",
    "main.js",
    "asset-worker.js",
    "audio-worklet.js",
    "_headers",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--copy-source", type=Path)
    return parser.parse_args()


def load_config(path: Path) -> dict:
    try:
        config = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"invalid config {path}: {error}") from error
    if config.get("schemaVersion") != 1 or not isinstance(config.get("source"), dict):
        raise SystemExit("config requires schemaVersion 1 and a source object")
    source = config["source"]
    kind = source.get("kind")
    if kind == "user-rom":
        return config
    if kind not in ("rom-url", "bundle-url"):
        raise SystemExit("source.kind must be user-rom, rom-url, or bundle-url")
    if not isinstance(source.get("url"), str) or not source["url"]:
        raise SystemExit("URL sources require a non-empty url")
    if source.get("fallback") not in ("user-rom", "none"):
        raise SystemExit("URL sources require fallback user-rom or none")
    return config


def source_target(config: dict, output: Path) -> Path:
    url = config["source"]["url"]
    parsed = urlparse(url)
    if parsed.scheme or parsed.netloc or parsed.query or parsed.fragment:
        raise SystemExit("--copy-source requires a relative, query-free source URL")
    target = (output / parsed.path).resolve()
    try:
        target.relative_to(output.resolve())
    except ValueError as error:
        raise SystemExit("source URL escapes the package output directory") from error
    return target


def validate_source(kind: str, path: Path) -> None:
    try:
        data = path.read_bytes()
    except OSError as error:
        raise SystemExit(f"could not read --copy-source {path}: {error}") from error
    if kind == "rom-url":
        actual = hashlib.sha1(data).hexdigest()
        if actual != CANONICAL_SHA1:
            raise SystemExit(
                f"unsupported ROM: expected SHA-1 {CANONICAL_SHA1}, got {actual}"
            )
    elif not data.startswith(b"WSBNDL01"):
        raise SystemExit("bundle source is not a WSBNDL01 runtime bundle")


def package(config_path: Path, output: Path, copy_source: Path | None) -> None:
    repo_root = Path(__file__).resolve().parent.parent
    web_root = repo_root / "web"
    config = load_config(config_path)
    kind = config["source"]["kind"]
    if copy_source and kind == "user-rom":
        raise SystemExit("--copy-source is invalid for user-rom mode")
    if kind != "user-rom" and copy_source is None:
        pass

    staging = output.with_name(f".{output.name}.staging")
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir(parents=True)
    for name in SHELL_FILES:
        shutil.copy2(web_root / name, staging / name)
    shutil.copytree(web_root / "pkg", staging / "pkg")
    (staging / "config.json").write_text(json.dumps(config, indent=2) + "\n")

    if copy_source:
        validate_source(kind, copy_source)
        target = source_target(config, staging)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(copy_source, target)

    if output.exists():
        shutil.rmtree(output)
    staging.replace(output)


if __name__ == "__main__":
    arguments = parse_args()
    package(arguments.config, arguments.output, arguments.copy_source)
