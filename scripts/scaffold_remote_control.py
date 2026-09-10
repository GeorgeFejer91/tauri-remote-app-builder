#!/usr/bin/env python3
"""Copy the integrated Tauri/CLI/remote starter without overwriting files."""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path, PureWindowsPath


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "starter"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Copy the Rust authority, typed CLI, browser companion, contracts, "
            "and notices into a new project subdirectory."
        )
    )
    parser.add_argument("target", type=Path, help="Existing project directory")
    parser.add_argument(
        "--destination-name",
        default="remote-control-starter",
        help="New child directory name (default: remote-control-starter)",
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="Print the planned destination without writing"
    )
    return parser.parse_args(argv)


def safe_child(target: Path, name: str) -> Path:
    windows_name = PureWindowsPath(name)
    if (
        not name
        or name in {".", ".."}
        or "/" in name
        or "\\" in name
        or Path(name).is_absolute()
        or windows_name.is_absolute()
        or bool(windows_name.drive)
        or Path(name).name != name
    ):
        raise ValueError("destination name must be one plain directory name")
    root = target.resolve(strict=True)
    if not root.is_dir():
        raise ValueError("target must be an existing directory")
    destination = root / name
    if destination.resolve(strict=False).parent != root:
        raise ValueError("destination must remain directly inside the target directory")
    if destination.exists() or destination.is_symlink():
        raise FileExistsError(f"refusing to overwrite existing destination: {destination}")
    return destination


def scaffold(target: Path, name: str, dry_run: bool = False) -> Path:
    if not SOURCE.is_dir():
        raise FileNotFoundError(f"starter source is missing: {SOURCE}")
    destination = safe_child(target, name)
    if not dry_run:
        shutil.copytree(
            SOURCE,
            destination,
            ignore=shutil.ignore_patterns("target", "node_modules", "__pycache__", "*.pyc"),
        )
    return destination


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        destination = scaffold(args.target, args.destination_name, args.dry_run)
    except (FileNotFoundError, FileExistsError, OSError, ValueError) as error:
        print(f"scaffold failed: {error}", file=sys.stderr)
        return 2
    action = "Would create" if args.dry_run else "Created"
    print(f"{action} {destination}")
    if not args.dry_run:
        print(
            "Next: replace the example product operations and profile, generate the complete "
            "action/query/subscription/transfer registry for the Tauri UI, local CLI, remote "
            "CLI, and browser companion, run the included parity validator, "
            "connect them to one Rust authority, select reviewed authentication and transport "
            "implementations, and run the application verification gate."
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
