#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-content-bundle.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

python3 scripts/convert-legacy-content-bundle.py \
  --kind item \
  --input scripts/fixtures/legacy-content-sample.lua \
  --input scripts/fixtures/legacy-content-item-extension.lua \
  --output "$temp_dir/items.json" >/dev/null
python3 - "$temp_dir/items.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["schema_version"] == 1
assert payload["record_kind"] == "item"
assert len(payload["sources"]) == 2
assert all(source["revision"] == "unbound-input" for source in payload["sources"])
assert [record["id"] for record in payload["records"]] == ["bfg9000", "chainsaw", "garmor"]
assert payload["records"][0]["source_index"] == 1
assert payload["records"][1]["migration_gaps"][0]["field"] == "OnUse"
assert payload["records"][2]["source_index"] == 0
assert payload["records"][2]["migration_gaps"][0]["field"] == "resist"
PY

legacy_repo=${DRL_LEGACY_REPO:-../doom-the-roughlike-original}
if [ -d "$legacy_repo/.git" ]; then
  python3 scripts/convert-legacy-content-bundle.py \
    --kind item \
    --legacy-repo "$legacy_repo" \
    --revision 17d9be1204751899b2d69d8d3a2dde247bd0cc5c \
    --output "$temp_dir/pinned-items.json" >/dev/null
  python3 - "$temp_dir/pinned-items.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert len(payload["sources"]) == 3
assert [source["path"] for source in payload["sources"]] == [
    "bin/data/drl/items/items.lua",
    "bin/data/drl/items/eitems.lua",
    "bin/data/drl/items/uitems.lua",
]
assert payload["sources"][0]["revision"] == "17d9be1204751899b2d69d8d3a2dde247bd0cc5c"
assert len(payload["records"]) == 126
assert payload["records"][0]["id"] == "aarmor"
assert payload["records"][-1]["id"] == "utristar"
PY
else
  printf '%s\n' 'Pinned item-family probe: NOT_RUN (legacy checkout unavailable).'
fi

printf '%s\n' 'Legacy item-family bundle contract: PASS (multi-source provenance, ordering, and gaps).'
