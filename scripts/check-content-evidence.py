#!/usr/bin/env python3
"""Validate pinned legacy-content evidence bundles against a reviewed crosswalk."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

SHA256 = re.compile(r"^[0-9a-f]{64}$")
RUST_LEVEL_BLOCK = re.compile(
  r"^  SpecialLevelDefinition \{\n(.*?)^  \},$",
  re.MULTILINE | re.DOTALL,
)
KINDS = ("being", "item", "cell", "level")


def parse_args() -> argparse.Namespace:
  parser = argparse.ArgumentParser(description=__doc__)
  parser.add_argument("--config", required=True, type=Path)
  parser.add_argument(
    "--bundle",
    action="append",
    required=True,
    metavar="KIND=PATH",
    help="evidence bundle to validate; repeat once for each configured kind",
  )
  parser.add_argument(
    "--rust-catalog",
    type=Path,
    help="Rust special-level definition source to synchronize with the level bundle",
  )
  return parser.parse_args()


def load_json(path: Path) -> object:
  try:
    return json.loads(path.read_text(encoding="utf-8"))
  except (OSError, json.JSONDecodeError) as error:
    raise SystemExit(f"unable to read JSON {path}: {error}") from error


def bundle_paths(values: list[str]) -> dict[str, Path]:
  paths: dict[str, Path] = {}
  for value in values:
    kind, separator, raw_path = value.partition("=")
    if not separator or kind not in KINDS or not raw_path:
      raise SystemExit(f"invalid --bundle {value!r}; expected KIND=PATH")
    if kind in paths:
      raise SystemExit(f"duplicate evidence bundle kind: {kind}")
    paths[kind] = Path(raw_path)
  missing = [kind for kind in KINDS if kind not in paths]
  if missing:
    raise SystemExit(f"missing evidence bundles: {', '.join(missing)}")
  return paths


def require(condition: bool, message: str) -> None:
  if not condition:
    raise SystemExit(f"content evidence coverage failed: {message}")


def decode_rust_string(encoded: str, field: str) -> str:
  try:
    return json.loads(f'"{encoded}"')
  except json.JSONDecodeError as error:
    raise SystemExit(f"invalid Rust string in special-level field {field}: {error}") from error


def rust_required_string_field(block: str, field: str) -> str:
  pattern = re.compile(
    rf'^\s+{field}:\s*"((?:\\.|[^"\\])*)"\s*,',
    re.MULTILINE,
  )
  match = pattern.search(block)
  require(match is not None, f"Rust special-level field {field} is malformed")
  return decode_rust_string(match.group(1), field)


def rust_optional_string_field(block: str, field: str) -> str | None:
  pattern = re.compile(
    rf'^\s+{field}:\s*(None|Some\(\s*"((?:\\.|[^"\\])*)"\s*,?\s*\))\s*,',
    re.MULTILINE,
  )
  match = pattern.search(block)
  require(match is not None, f"Rust special-level field {field} is malformed")
  if match.group(1) == "None":
    return None
  return decode_rust_string(match.group(2), field)


def rust_optional_int_field(block: str, field: str) -> int | None:
  pattern = re.compile(
    rf"^\s+{field}:\s*(None|Some\(\s*(\d+)\s*,?\s*\))\s*,",
    re.MULTILINE,
  )
  match = pattern.search(block)
  require(match is not None, f"Rust special-level field {field} is malformed")
  return None if match.group(1) == "None" else int(match.group(2))


def require_rust_field(block: str, field: str) -> None:
  require(
    re.search(rf"^\s+{field}:\s+", block, re.MULTILINE) is not None,
    f"Rust special-level field {field} is missing",
  )


def strip_rust_comments(source: str) -> str:
  output: list[str] = []
  state = "normal"
  block_comment_depth = 0
  index = 0
  while index < len(source):
    char = source[index]
    next_char = source[index + 1] if index + 1 < len(source) else ""
    if state == "normal":
      if char == "/" and next_char == "/":
        output.extend((" ", " "))
        index += 2
        state = "line-comment"
        continue
      if char == "/" and next_char == "*":
        output.extend((" ", " "))
        index += 2
        state = "block-comment"
        block_comment_depth = 1
        continue
      output.append(char)
      if char == '"':
        state = "string"
      index += 1
      continue
    if state == "line-comment":
      if char == "\n":
        output.append(char)
        state = "normal"
      else:
        output.append(" ")
      index += 1
      continue
    if state == "block-comment":
      if char == "/" and next_char == "*":
        output.extend((" ", " "))
        index += 2
        block_comment_depth += 1
        continue
      if char == "*" and next_char == "/":
        output.extend((" ", " "))
        index += 2
        block_comment_depth -= 1
        if block_comment_depth == 0:
          state = "normal"
      else:
        output.append("\n" if char == "\n" else " ")
        index += 1
      continue
    output.append(char)
    if char == "\\" and index + 1 < len(source):
      output.append(source[index + 1])
      index += 2
      continue
    if char == '"':
      state = "normal"
    index += 1
  require(block_comment_depth == 0, "Rust catalog has an unterminated block comment")
  return "".join(output)


def parse_rust_catalog(path: Path) -> list[dict[str, object]]:
  try:
    source = path.read_text(encoding="utf-8")
  except OSError as error:
    raise SystemExit(f"unable to read Rust special-level catalog {path}: {error}") from error
  records: list[dict[str, object]] = []
  source = strip_rust_comments(source)
  for block in RUST_LEVEL_BLOCK.findall(source):
    for field in ("id", "name", "legacy_depth", "entry", "welcome"):
      require_rust_field(block, field)
    identifier = rust_required_string_field(block, "id")
    name = rust_required_string_field(block, "name")
    records.append(
      {
        "id": identifier,
        "name": name,
        "level": rust_optional_int_field(block, "legacy_depth"),
        "entry": rust_optional_string_field(block, "entry"),
        "welcome": rust_optional_string_field(block, "welcome"),
      }
    )
  return records


def validate_rust_catalog(path: Path, expected_ids: object, level_records: list[dict[str, object]]) -> None:
  require(isinstance(expected_ids, list), "level crosswalk lacks a Rust catalog ID list")
  rust_records = parse_rust_catalog(path)
  rust_ids = [record["id"] for record in rust_records]
  require(rust_ids == expected_ids, "Rust special-level IDs differ from the reviewed level catalog")
  legacy_by_id = {record["id"]: record for record in level_records}
  for rust_record in rust_records:
    identifier = rust_record["id"]
    legacy_fields = legacy_by_id[identifier].get("fields", {})
    require(isinstance(legacy_fields, dict), f"level {identifier} scalar fields are malformed")
    for rust_field, legacy_field in (("name", "name"), ("level", "level"), ("entry", "entry"), ("welcome", "welcome")):
      require(
        rust_record[rust_field] == legacy_fields.get(legacy_field),
        f"level {identifier} field {legacy_field} differs from the Rust catalog",
      )


def validate_bundle(kind: str, path: Path, expected: dict[str, object], revision: str) -> int:
  payload = load_json(path)
  require(isinstance(payload, dict), f"{kind} bundle is not an object")
  require(payload.get("schema_version") == 1, f"{kind} schema version is not 1")
  require(payload.get("record_kind") == kind, f"{kind} record kind mismatch")

  sources = payload.get("sources")
  require(isinstance(sources, list) and sources, f"{kind} has no source provenance")
  require(all(isinstance(source, dict) for source in sources), f"{kind} source provenance is malformed")
  expected_sources = expected.get("sources")
  require(
    [source.get("path") for source in sources] == expected_sources,
    f"{kind} source paths changed",
  )
  for index, source in enumerate(sources):
    require(isinstance(source, dict), f"{kind} source {index} is not an object")
    require(source.get("revision") == revision, f"{kind} source {index} revision mismatch")
    digest = source.get("sha256")
    require(
      isinstance(digest, str) and SHA256.fullmatch(digest) is not None,
      f"{kind} source {index} lacks a SHA-256",
    )
  expected_digests = expected.get("source_sha256")
  require(
    isinstance(expected_digests, list) and len(expected_digests) == len(sources),
    f"{kind} source digest crosswalk is missing or mis-sized",
  )
  for index, source in enumerate(sources):
    require(
      source.get("sha256") == expected_digests[index],
      f"{kind} source {index} digest differs from the reviewed crosswalk",
    )

  records = payload.get("records")
  require(isinstance(records, list), f"{kind} records are not a list")
  for index, record in enumerate(records):
    require(isinstance(record, dict), f"{kind} record {index} is not an object")
    line = record.get("line")
    require(
      isinstance(line, int) and not isinstance(line, bool) and line > 0,
      f"{kind} record {index} has an invalid source line",
    )
    fields = record.get("fields")
    require(isinstance(fields, dict), f"{kind} record {index} fields are not an object")
    for field, value in fields.items():
      require(isinstance(field, str) and field, f"{kind} record {index} has an invalid field name")
      require(
        isinstance(value, (str, int, bool)) and not isinstance(value, (dict, list, tuple)),
        f"{kind} record {index} field {field} is not a scalar",
      )
    gaps = record.get("migration_gaps")
    require(isinstance(gaps, list), f"{kind} record {index} migration gaps are not a list")
    for gap_index, gap in enumerate(gaps):
      require(isinstance(gap, dict), f"{kind} record {index} gap {gap_index} is not an object")
      gap_field = gap.get("field")
      gap_line = gap.get("line")
      require(
        isinstance(gap_field, str) and gap_field,
        f"{kind} record {index} gap {gap_index} has an invalid field",
      )
      require(
        isinstance(gap_line, int) and not isinstance(gap_line, bool) and gap_line > 0,
        f"{kind} record {index} gap {gap_index} has an invalid source line",
      )
  require(len(records) == expected.get("record_count"), f"{kind} record count changed")
  ids = [record.get("id") for record in records]
  require(
    all(isinstance(record_id, str) and record_id for record_id in ids),
    f"{kind} contains an invalid record ID",
  )
  require(ids == sorted(ids), f"{kind} records are not sorted")
  require(len(ids) == len(set(ids)), f"{kind} contains duplicate record IDs")

  required_ids = expected.get("required_ids", [])
  missing = sorted(set(required_ids) - set(ids))
  require(not missing, f"{kind} is missing required IDs: {', '.join(missing)}")
  expected_ids = expected.get("record_ids")
  if expected_ids is not None:
    require(ids == expected_ids, f"{kind} record IDs do not match the reviewed catalog")
  for index, record in enumerate(records):
    source_index = record.get("source_index")
    require(
      isinstance(source_index, int)
      and not isinstance(source_index, bool)
      and 0 <= source_index < len(sources),
      f"{kind} record {index} has an invalid source index",
    )
  return len(records)


def main() -> int:
  args = parse_args()
  config = load_json(args.config)
  require(isinstance(config, dict), "crosswalk is not an object")
  require(config.get("schema_version") == 1, "crosswalk schema version is not 1")
  revision = config.get("revision")
  require(isinstance(revision, str) and revision, "crosswalk revision is missing")
  bundles = config.get("bundles")
  require(isinstance(bundles, dict), "crosswalk bundles are missing")
  missing_kinds = [kind for kind in KINDS if kind not in bundles]
  require(not missing_kinds, f"crosswalk is missing bundles: {', '.join(missing_kinds)}")
  require(all(isinstance(bundles[kind], dict) for kind in KINDS), "crosswalk bundle entries are malformed")
  paths = bundle_paths(args.bundle)
  counts = {
    kind: validate_bundle(kind, paths[kind], bundles[kind], revision)
    for kind in KINDS
  }
  if args.rust_catalog is not None:
    level_payload = load_json(paths["level"])
    require(isinstance(level_payload, dict), "level bundle is not an object")
    level_records = level_payload.get("records")
    require(isinstance(level_records, list), "level bundle records are not a list")
    validate_rust_catalog(args.rust_catalog, bundles["level"].get("record_ids"), level_records)
  print(
    "Content evidence coverage: PASS "
    f"(being={counts['being']}, item={counts['item']}, "
    f"cell={counts['cell']}, level={counts['level']}; pinned {revision}"
    + ("; Rust catalog synchronized" if args.rust_catalog is not None else "")
    + ")"
  )
  return 0


if __name__ == "__main__":
  raise SystemExit(main())
