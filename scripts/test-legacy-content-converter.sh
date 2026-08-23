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
else
  printf '%s\n' 'Pinned legacy probe: NOT_RUN (legacy checkout unavailable).'
fi

printf '%s\n' 'Legacy content converter contract: PASS (scalar fields, provenance, and migration gaps).'
