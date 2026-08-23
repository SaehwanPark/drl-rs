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
RECORD_START = re.compile(r'^\s*register_(being|item|cell|level)\s+"([^"]+)"\s*$')
SCALAR = re.compile(
    r'^\s*(?P<field>[A-Za-z_]\w*)\s*=\s*'
    r'(?:(?P<double>"(?:\\.|[^"])*")|(?P<single>\'(?:\\.|[^\'])*\')|'
    r'(?P<int>-?\d+)|(?P<bool>true|false))\s*[,;]?\s*(?:--.*)?$'
)
FIELD = re.compile(r'^\s*([A-Za-z_]\w*)\s*=')


def decode_lua_string(token: str) -> str:
    content = token[1:-1]
    decoded: list[str] = []
    index = 0
    escapes = {"n": "\n", "r": "\r", "t": "\t", "\\": "\\", '"': '"', "'": "'"}
    while index < len(content):
        char = content[index]
        if char != "\\":
            decoded.append(char)
            index += 1
            continue
        index += 1
        if index >= len(content):
            raise ValueError("unterminated Lua string escape")
        escaped = content[index]
        if escaped.isdigit():
            end = index
            while end < len(content) and end < index + 3 and content[end].isdigit():
                end += 1
            value = int(content[index:end])
            if value > 255:
                raise ValueError(f"Lua byte escape is out of range: {value}")
            decoded.append(chr(value))
            index = end
        elif escaped in escapes:
            decoded.append(escapes[escaped])
            index += 1
        else:
            raise ValueError(f"unsupported Lua string escape: \\{escaped}")
    return "".join(decoded)


def scalar_value(match: re.Match[str]) -> object:
    if match.group("double") is not None:
        return decode_lua_string(match.group("double"))
    if match.group("single") is not None:
        return decode_lua_string(match.group("single"))
    if match.group("int") is not None:
        return int(match.group("int"))
    return match.group("bool") == "true"


class LuaLexState:
    def __init__(self) -> None:
        self.quote: str | None = None
        self.escaped = False
        self.long_end: str | None = None
        self.long_comment_end: str | None = None


def record_candidate(line: str, state: LuaLexState) -> str:
    """Hide long strings/comments before checking record declarations."""
    if state.long_end is not None:
        end = line.find(state.long_end)
        if end < 0:
            return ""
        line = line[end + len(state.long_end) :]
        state.long_end = None
    if state.long_comment_end is not None:
        end = line.find(state.long_comment_end)
        if end < 0:
            return ""
        line = line[end + len(state.long_comment_end) :]
        state.long_comment_end = None
    stripped = line.lstrip()
    if stripped.startswith("--"):
        opener = re.match(r"--\[(=*)\[", stripped)
        if opener:
            state.long_comment_end = "]" + opener.group(1) + "]"
            content = stripped[opener.end() :]
            end = content.find(state.long_comment_end)
            if end < 0:
                return ""
            line = content[end + len(state.long_comment_end) :]
            state.long_comment_end = None
            return line
        return ""
    return line


def brace_delta(line: str, state: LuaLexState) -> int:
    """Count braces outside Lua quoted, long-bracket, and comment text."""
    delta = 0
    index = 0
    while index < len(line):
        if state.long_end is not None:
            end = line.find(state.long_end, index)
            if end < 0:
                return delta
            index = end + len(state.long_end)
            state.long_end = None
            continue
        if state.long_comment_end is not None:
            end = line.find(state.long_comment_end, index)
            if end < 0:
                return delta
            index = end + len(state.long_comment_end)
            state.long_comment_end = None
            continue
        char = line[index]
        if state.quote is not None:
            if state.escaped:
                state.escaped = False
            elif char == "\\":
                state.escaped = True
            elif char == state.quote:
                state.quote = None
        elif char in ('"', "'"):
            state.quote = char
        elif char == "[":
            opener = re.match(r"\[(=*)\[", line[index:])
            if opener:
                state.long_end = "]" + opener.group(1) + "]"
                index += len(opener.group(0))
                continue
        elif char == "-" and index + 1 < len(line) and line[index + 1] == "-":
            opener = re.match(r"--\[(=*)\[", line[index:])
            if opener:
                state.long_comment_end = "]" + opener.group(1) + "]"
                index += len(opener.group(0))
                continue
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
    lex_state = LuaLexState()
    for line_number, line in enumerate(source.splitlines(), start=1):
        candidate = record_candidate(line, lex_state)
        start = RECORD_START.match(candidate)
        if start:
            kind, record_id = start.groups()
            if kind != expected_kind:
                continue
            if active is not None:
                raise ValueError(f"unterminated record before line {line_number}")
            active = {
                "id": record_id,
                "fields": {},
                "migration_gaps": [],
                "line": line_number,
            }
            depth = 0
            lex_state = LuaLexState()
            continue
        if active is None:
            continue
        stripped = candidate.strip()
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
        depth += brace_delta(candidate, lex_state)
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
    parser.add_argument("--kind", choices=("being", "item", "cell", "level"), required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--input", type=Path, help="unbound fixture/input file for tests")
    parser.add_argument("--legacy-repo", default=str(DEFAULT_LEGACY_REPO))
    parser.add_argument("--revision", default=PINNED_REVISION)
    parser.add_argument("--source", help="legacy Lua path (defaults by record kind)")
    args = parser.parse_args()
    default_sources = {
        "being": "bin/data/drl/beings.lua",
        "item": "bin/data/drl/items/items.lua",
        "cell": "bin/data/drl/cells.lua",
        "level": "bin/data/drl/levels/armory.lua",
    }
    if args.source is None:
        args.source = default_sources[args.kind]
    if args.input and args.source != default_sources[args.kind]:
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
