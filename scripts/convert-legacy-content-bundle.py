#!/usr/bin/env python3
"""Combine deterministic evidence records from multiple pinned Lua sources.

This is a build-time evidence tool, not a Lua interpreter. It reuses the
single-source converter so nested behavior and unsupported values remain gaps.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path

CONVERTER_PATH = Path(__file__).with_name("convert-legacy-content.py")
DEFAULT_SOURCES = {
    "being": ["bin/data/drl/beings.lua"],
    "item": [
        "bin/data/drl/items/items.lua",
        "bin/data/drl/items/eitems.lua",
        "bin/data/drl/items/uitems.lua",
    ],
    "cell": ["bin/data/drl/cells.lua"],
}


def load_converter():
    spec = importlib.util.spec_from_file_location("legacy_content_converter", CONVERTER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load converter: {CONVERTER_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=("being", "item", "cell"), required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--input", action="append", type=Path, help="unbound fixture/input file")
    parser.add_argument("--source", action="append", help="pinned Lua path (repeatable)")
    parser.add_argument("--legacy-repo", default=None)
    parser.add_argument("--revision", default=None)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    converter = load_converter()
    inputs = [path.resolve() for path in args.input or []]
    if inputs and args.source:
        raise SystemExit("--input cannot be combined with --source")
    sources = args.source or DEFAULT_SOURCES[args.kind]
    if inputs:
        sources = [str(path) for path in inputs]
    legacy_repo = args.legacy_repo or str(converter.DEFAULT_LEGACY_REPO)
    revision = args.revision or converter.PINNED_REVISION
    records: list[dict[str, object]] = []
    provenance: list[dict[str, str]] = []
    seen: set[object] = set()
    for source_index, source in enumerate(sources):
        source_args = argparse.Namespace(
            input=Path(source) if inputs else None,
            legacy_repo=legacy_repo,
            revision=revision,
            source=source,
        )
        raw, source_provenance = converter.read_source(source_args)
        provenance.append(source_provenance)
        source_records = converter.extract_records(raw, args.kind)
        for record in source_records:
            record_id = record["id"]
            if record_id in seen:
                raise SystemExit(f"duplicate {args.kind} record id across sources: {record_id}")
            seen.add(record_id)
            record["source_index"] = source_index
            records.append(record)
    records.sort(key=lambda record: record["id"])
    payload = {
        "schema_version": 1,
        "record_kind": args.kind,
        "sources": provenance,
        "records": records,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Legacy {args.kind} bundle conversion: PASS ({len(records)} records; gaps preserved)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
