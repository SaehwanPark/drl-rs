#!/usr/bin/env python3
"""Index shallow metadata from the pinned legacy special-level Lua files."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import subprocess
from pathlib import Path

CONVERTER_PATH = Path(__file__).with_name("convert-legacy-content.py")
PINNED_REVISION = "17d9be1204751899b2d69d8d3a2dde247bd0cc5c"
DEFAULT_REPO = Path(__file__).resolve().parents[1].parent / "doom-the-roughlike-original"


def load_converter():
    spec = importlib.util.spec_from_file_location("legacy_content_converter", CONVERTER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load converter: {CONVERTER_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--input", action="append", type=Path, help="unbound fixture/input file")
    parser.add_argument("--legacy-repo", default=str(DEFAULT_REPO))
    parser.add_argument("--revision", default=PINNED_REVISION)
    parser.add_argument("--source-prefix", default="bin/data/drl/levels")
    return parser.parse_args()


def pinned_sources(args: argparse.Namespace) -> list[str]:
    try:
        output = subprocess.check_output(
            [
                "git",
                "-C",
                str(Path(args.legacy_repo).resolve()),
                "ls-tree",
                "-r",
                "--name-only",
                args.revision,
                "--",
                args.source_prefix,
            ],
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"pinned level sources are unavailable: {error}") from error
    sources = [line for line in output.splitlines() if line.endswith(".lua")]
    if not sources:
        raise SystemExit("no pinned level Lua sources found")
    return sorted(sources)


def main() -> int:
    args = parse_args()
    converter = load_converter()
    inputs = [path.resolve() for path in args.input or []]
    if inputs:
        sources = [str(path) for path in inputs]
    else:
        sources = pinned_sources(args)
    records: list[dict[str, object]] = []
    provenance: list[dict[str, str]] = []
    seen: set[object] = set()
    for source_index, source in enumerate(sources):
        if inputs:
            raw = Path(source).read_bytes()
            source_provenance = {
                "path": source,
                "revision": "unbound-input",
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
        else:
            raw = subprocess.check_output(
                [
                    "git",
                    "-C",
                    str(Path(args.legacy_repo).resolve()),
                    "show",
                    f"{args.revision}:{source}",
                ]
            )
            source_provenance = {
                "path": source,
                "revision": args.revision,
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
        provenance.append(source_provenance)
        try:
            source_records = converter.extract_records(raw.decode("utf-8"), "level")
        except ValueError as error:
            # A pinned file may contain only commented-out or non-level content;
            # retain its provenance while contributing no active records.
            if str(error) != "no register_level records found":
                raise
            source_records = []
        for record in source_records:
            record_id = record["id"]
            if record_id in seen:
                raise SystemExit(f"duplicate level record id across sources: {record_id}")
            seen.add(record_id)
            record["source_index"] = source_index
            records.append(record)
    records.sort(key=lambda record: record["id"])
    payload = {
        "schema_version": 1,
        "record_kind": "level",
        "sources": provenance,
        "records": records,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Legacy level index: PASS ({len(records)} records; gaps preserved)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
