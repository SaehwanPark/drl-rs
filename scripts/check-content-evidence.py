#!/usr/bin/env python3
"""Validate pinned legacy-content evidence bundles against a reviewed crosswalk."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

SHA256 = re.compile(r"^[0-9a-f]{64}$")
RUST_LEVEL_ID = re.compile(r'^\s+id:\s+"([^"]+)",\s*$', re.MULTILINE)
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


def validate_rust_catalog(path: Path, expected_ids: object) -> None:
  require(isinstance(expected_ids, list), "level crosswalk lacks a Rust catalog ID list")
  try:
    source = path.read_text(encoding="utf-8")
  except OSError as error:
    raise SystemExit(f"unable to read Rust special-level catalog {path}: {error}") from error
  ids = RUST_LEVEL_ID.findall(source)
  require(ids == expected_ids, "Rust special-level IDs differ from the reviewed level catalog")


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

  records = payload.get("records")
  require(isinstance(records, list), f"{kind} records are not a list")
  require(len(records) == expected.get("record_count"), f"{kind} record count changed")
  ids = [record.get("id") for record in records if isinstance(record, dict)]
  require(len(ids) == len(records), f"{kind} contains a non-object record")
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
    validate_rust_catalog(args.rust_catalog, bundles["level"].get("record_ids"))
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
