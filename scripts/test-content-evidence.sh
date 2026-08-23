#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-content-evidence-test.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

python3 scripts/convert-legacy-content-bundle.py \
  --kind being \
  --input scripts/fixtures/legacy-content-sample.lua \
  --output "$temp_dir/being.json" >/dev/null
python3 scripts/convert-legacy-content-bundle.py \
  --kind item \
  --input scripts/fixtures/legacy-content-sample.lua \
  --input scripts/fixtures/legacy-content-item-extension.lua \
  --output "$temp_dir/item.json" >/dev/null
python3 scripts/convert-legacy-content-bundle.py \
  --kind cell \
  --input scripts/fixtures/legacy-content-cell-sample.lua \
  --output "$temp_dir/cell.json" >/dev/null
python3 scripts/convert-legacy-level-index.py \
  --input scripts/fixtures/legacy-content-level-sample.lua \
  --output "$temp_dir/level.json" >/dev/null

python3 - "$temp_dir" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
bundles = {}
for kind in ("being", "item", "cell", "level"):
  payload = json.loads((root / f"{kind}.json").read_text(encoding="utf-8"))
  ids = [record["id"] for record in payload["records"]]
  entry = {
    "sources": [source["path"] for source in payload["sources"]],
    "record_count": len(ids),
    "required_ids": ids,
  }
  if kind == "level":
    entry["record_ids"] = ids
  bundles[kind] = entry
(root / "config.json").write_text(
  json.dumps(
    {"schema_version": 1, "revision": "unbound-input", "bundles": bundles},
    indent=2,
  )
  + "\n",
  encoding="utf-8",
)
PY

python3 scripts/check-content-evidence.py \
  --config "$temp_dir/config.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null

python3 - "$temp_dir/being.json" "$temp_dir/duplicate.json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["records"][1]["id"] = payload["records"][0]["id"]
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/config.json" \
  --bundle "being=$temp_dir/duplicate.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'duplicate evidence IDs must be rejected' >&2
  exit 1
fi

python3 - "$temp_dir/being.json" "$temp_dir/unsorted.json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["records"][0], payload["records"][1] = payload["records"][1], payload["records"][0]
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/config.json" \
  --bundle "being=$temp_dir/unsorted.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'unsorted evidence IDs must be rejected' >&2
  exit 1
fi

python3 - "$temp_dir/config.json" "$temp_dir/missing.json" <<'PY'
import json
import pathlib
import sys

config = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
config["bundles"]["being"]["required_ids"].append("missing-representative")
pathlib.Path(sys.argv[2]).write_text(json.dumps(config), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/missing.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'missing representative evidence must be rejected' >&2
  exit 1
fi

python3 - "$temp_dir/config.json" "$temp_dir/wrong-revision.json" <<'PY'
import json
import pathlib
import sys

config = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
config["revision"] = "wrong-revision"
pathlib.Path(sys.argv[2]).write_text(json.dumps(config), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/wrong-revision.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'wrong evidence revision must be rejected' >&2
  exit 1
fi

printf '%s\n' 'Content evidence validator contract: PASS (fixture coverage and rejection cases).'
