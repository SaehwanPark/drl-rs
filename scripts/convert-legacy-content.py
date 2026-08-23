#!/usr/bin/env python3
"""Extract shallow declarative Lua records into provenance-bearing JSON.

This is intentionally not a Lua interpreter. It reads a pinned legacy file,
copies scalar fields, and records nested tables/functions as explicit migration
gaps for later evidence work.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

PINNED_REVISION = "17d9be1204751899b2d69d8d3a2dde247bd0cc5c"
DEFAULT_LEGACY_REPO = Path(__file__).resolve().parents[1].parent / "doom-the-roughlike-original"
RECORD_START = re.compile(r'^\s*register_(being|item)\s+"([^"]+)"\s*$')
SCALAR = re.compile(
    r'^\s*([A-Za-z_]\w*)\s*=\s*(?:"((?:\\.|[^"])*)"|(-?\d+)|(true|false))\s*,?\s*(?:--.*)?$'
)
FIELD = re.compile(r'^\s*([A-Za-z_]\w*)\s*=')


def scalar_value(match: re.Match[str]) -> object:
    if match.group(2) is not None:
        return json.loads(f'"{match.group(2)}"')
    if match.group(3) is not None:
        return int(match.group(3))
    return match.group(4) == "true"


def brace_delta(line: str) -> int:
    """Count structural braces while ignoring Lua strings and comments."""
    delta = 0
    quote: str | None = None
    escaped = False
    index = 0
    while index < len(line):
        char = line[index]
        if quote is not None:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = None
        elif char in ('"', "'"):
            quote = char
        elif char == "-" and index + 1 < len(line) and line[index + 1] == "-":
            break
        elif char == "{":
            delta += 1
        elif char == "}":
            delta -= 1
        index += 1
    return delta


def extract_records(source: str, expected_kind: str) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    active: dict[str, object] | None = None
    depth = 0
    for line_number, line in enumerate(source.splitlines(), start=1):
        start = RECORD_START.match(line)
        if start:
            if active is not None:
                raise ValueError(f"unterminated record before line {line_number}")
            kind, record_id = start.groups()
            if kind != expected_kind:
                continue
            active = {
                "id": record_id,
                "fields": {},
                "migration_gaps": [],
                "line": line_number,
            }
            depth = 0
            continue
        if active is None:
            continue
        stripped = line.strip()
        if depth == 0:
            if stripped.startswith("{"):
                depth = 1
            continue
        if depth == 1 and stripped and not stripped.startswith("--"):
            scalar = SCALAR.match(line)
            if scalar:
                fields = active["fields"]
                assert isinstance(fields, dict)
                fields[scalar.group(1)] = scalar_value(scalar)
            else:
                field = FIELD.match(line)
                if field:
                    gaps = active["migration_gaps"]
                    assert isinstance(gaps, list)
                    gaps.append({"field": field.group(1), "line": line_number})
        depth += brace_delta(line)
        if depth == 0:
            records.append(active)
            active = None
    if active is not None:
        raise ValueError("unterminated final record")
    if not records:
        raise ValueError(f"no register_{expected_kind} records found")
    ids = [record["id"] for record in records]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate record id")
    for record in records:
        fields = record["fields"]
        gaps = record["migration_gaps"]
        assert isinstance(fields, dict) and isinstance(gaps, list)
        record["fields"] = dict(sorted(fields.items()))
        record["migration_gaps"] = sorted(gaps, key=lambda gap: (gap["field"], gap["line"]))
    return sorted(records, key=lambda record: record["id"])


def read_source(args: argparse.Namespace) -> tuple[str, dict[str, str]]:
    if args.input:
        path = Path(args.input).resolve()
        raw = path.read_bytes()
        return raw.decode("utf-8"), {
            "path": str(path),
            "revision": "unbound-input",
            "sha256": hashlib.sha256(raw).hexdigest(),
        }
    repo = Path(args.legacy_repo).resolve()
    try:
        subprocess.run(
            ["git", "-C", str(repo), "cat-file", "-e", f"{args.revision}^{{commit}}"],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
        )
        raw = subprocess.check_output(
            ["git", "-C", str(repo), "show", f"{args.revision}:{args.source}"],
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise ValueError(f"pinned legacy source is unavailable: {error}") from error
    return raw.decode("utf-8"), {
        "path": args.source,
        "revision": args.revision,
        "sha256": hashlib.sha256(raw).hexdigest(),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=("being", "item"), required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--input", type=Path, help="unbound fixture/input file for tests")
    parser.add_argument("--legacy-repo", default=str(DEFAULT_LEGACY_REPO))
    parser.add_argument("--revision", default=PINNED_REVISION)
    parser.add_argument("--source", help="legacy Lua path (defaults by record kind)")
    args = parser.parse_args()
    if args.source is None:
        args.source = "bin/data/drl/beings.lua" if args.kind == "being" else "bin/data/drl/items/items.lua"
    if args.input and args.source != ("bin/data/drl/beings.lua" if args.kind == "being" else "bin/data/drl/items/items.lua"):
        parser.error("--input cannot be combined with --source")
    try:
        source, provenance = read_source(args)
        records = extract_records(source, args.kind)
    except (OSError, UnicodeDecodeError, ValueError) as error:
        parser.error(str(error))
    payload = {
        "schema_version": 1,
        "record_kind": args.kind,
        "source": provenance,
        "records": records,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Legacy {args.kind} conversion: PASS ({len(records)} records; gaps preserved)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
