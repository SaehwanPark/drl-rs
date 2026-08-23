#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-content-converter.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

python3 scripts/convert-legacy-content.py \
  --kind being \
  --input scripts/fixtures/legacy-content-sample.lua \
  --output "$temp_dir/beings.json" >/dev/null

python3 - "$temp_dir/beings.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["schema_version"] == 1
assert payload["record_kind"] == "being"
assert payload["source"]["revision"] == "unbound-input"
assert [record["id"] for record in payload["records"]] == ["former", "imp"]
imp = payload["records"][1]
assert imp["fields"]["hp"] == 12
assert imp["fields"]["corpse"] is True
assert imp["fields"]["desc"] == "literal } brace"
assert [gap["field"] for gap in imp["migration_gaps"]] == ["OnCreate", "flags"]
PY

python3 scripts/convert-legacy-content.py \
  --kind item \
  --input scripts/fixtures/legacy-content-sample.lua \
  --output "$temp_dir/items.json" >/dev/null
python3 - "$temp_dir/items.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["record_kind"] == "item"
assert [record["id"] for record in payload["records"]] == ["garmor"]
assert payload["records"][0]["fields"]["armor"] == 1
assert payload["records"][0]["migration_gaps"][0]["field"] == "resist"
PY

python3 scripts/convert-legacy-content.py \
  --kind cell \
  --input scripts/fixtures/legacy-content-cell-sample.lua \
  --output "$temp_dir/cells.json" >/dev/null
python3 scripts/convert-legacy-content.py \
  --kind cell \
  --input scripts/fixtures/legacy-content-cell-sample.lua \
  --output "$temp_dir/cells-repeat.json" >/dev/null
cmp "$temp_dir/cells.json" "$temp_dir/cells-repeat.json"
python3 - "$temp_dir/cells.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["record_kind"] == "cell"
assert [record["id"] for record in payload["records"]] == ["floor", "wall"]
floor = payload["records"][0]
assert floor["fields"]["ascii"] == "\u00fa"
assert floor["fields"]["bloodto"] == "bloodpool"
wall = payload["records"][1]
assert [gap["field"] for gap in wall["migration_gaps"]] == ["OnAct", "flags"]
PY
if python3 scripts/convert-legacy-content.py \
  --kind cell \
  --input scripts/fixtures/legacy-content-bad-escape.lua \
  --output "$temp_dir/bad.json" >/dev/null 2>&1; then
  printf '%s\n' 'unsupported Lua escapes must be rejected' >&2
  exit 1
fi

legacy_repo=${DRL_LEGACY_REPO:-../doom-the-roughlike-original}
if [ -d "$legacy_repo/.git" ]; then
  python3 scripts/convert-legacy-content.py \
    --kind item \
    --legacy-repo "$legacy_repo" \
    --revision 17d9be1204751899b2d69d8d3a2dde247bd0cc5c \
    --output "$temp_dir/pinned-items.json" >/dev/null
  python3 - "$temp_dir/pinned-items.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["record_kind"] == "item"
assert payload["source"]["path"] == "bin/data/drl/items/items.lua"
assert payload["source"]["revision"] == "17d9be1204751899b2d69d8d3a2dde247bd0cc5c"
assert payload["records"]
assert payload["records"][0]["id"] == "ammo"
assert "barmor" in {record["id"] for record in payload["records"]}
PY
  python3 scripts/convert-legacy-content.py \
    --kind cell \
    --legacy-repo "$legacy_repo" \
    --revision 17d9be1204751899b2d69d8d3a2dde247bd0cc5c \
    --output "$temp_dir/pinned-cells.json" >/dev/null
  python3 - "$temp_dir/pinned-cells.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["record_kind"] == "cell"
assert payload["source"]["path"] == "bin/data/drl/cells.lua"
assert payload["source"]["revision"] == "17d9be1204751899b2d69d8d3a2dde247bd0cc5c"
assert payload["records"]
assert payload["records"][0]["id"] == "acid"
PY
else
  printf '%s\n' 'Pinned legacy probe: NOT_RUN (legacy checkout unavailable).'
fi

printf '%s\n' 'Legacy content converter contract: PASS (being/item/cell scalars, provenance, and migration gaps).'
