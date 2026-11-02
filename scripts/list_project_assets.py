#!/usr/bin/env python3

"""List unique asset paths referenced by AAW project files."""

import argparse
import json
import sys
from pathlib import Path
from typing import Any


PROJECT_SUFFIXES = {".json", ".lmtproj"}
LEGACY_ASSET_PREFIX = "samples:/"


def normalize_asset_path(path: str) -> str:
    if path.startswith(LEGACY_ASSET_PREFIX):
        return path[len(LEGACY_ASSET_PREFIX) :]
    return path


def current_project_assets(project: dict[str, Any]) -> set[str]:
    assets: set[str] = set()
    instruments = project.get("instrument_graph", {}).get("instruments", {})

    if not isinstance(instruments, dict):
        return assets

    for instrument in instruments.values():
        if not isinstance(instrument, dict):
            continue

        kind = instrument.get("kind")
        if not isinstance(kind, dict) or kind.get("type") not in {"Fluid", "Sfizz", "Clap"}:
            continue

        data = kind.get("data")
        if isinstance(data, dict) and isinstance(data.get("file_path"), str):
            assets.add(normalize_asset_path(data["file_path"]))

    return assets


def legacy_project_assets(project: dict[str, Any]) -> set[str]:
    assets: set[str] = set()
    instruments = project.get("instruments", [])

    if not isinstance(instruments, list):
        return assets

    for instrument in instruments:
        if isinstance(instrument, dict) and isinstance(instrument.get("filePath"), str):
            assets.add(normalize_asset_path(instrument["filePath"]))

    return assets


def load_assets(path: Path) -> set[str]:
    try:
        with path.open(encoding="utf-8") as project_file:
            project = json.load(project_file)
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(f"failed to read {path}: {error}") from error

    if not isinstance(project, dict):
        return set()

    return current_project_assets(project) | legacy_project_assets(project)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Recursively list unique assets referenced by AAW project files."
    )
    parser.add_argument("directory", type=Path, help="directory containing project files")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    directory: Path = args.directory

    if not directory.is_dir():
        print(f"error: not a directory: {directory}", file=sys.stderr)
        return 2

    assets: set[str] = set()
    project_files = sorted(
        path
        for path in directory.rglob("*")
        if path.is_file() and path.suffix.lower() in PROJECT_SUFFIXES
    )

    try:
        for project_file in project_files:
            assets.update(load_assets(project_file))
    except RuntimeError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    for asset in sorted(assets, key=str.casefold):
        print(asset)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
