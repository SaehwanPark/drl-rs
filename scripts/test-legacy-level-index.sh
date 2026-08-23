#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-level-index.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

python3 scripts/convert-legacy-level-index.py \
  --input scripts/fixtures/legacy-content-level-sample.lua \
  --output "$temp_dir/levels.json" >/dev/null
python3 - "$temp_dir/levels.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["record_kind"] == "level"
assert len(payload["sources"]) == 1
assert payload["sources"][0]["revision"] == "unbound-input"
assert [record["id"] for record in payload["records"]] == ["alpha", "beta"]
assert payload["records"][0]["fields"]["level"] == 3
assert [gap["field"] for gap in payload["records"][0]["migration_gaps"]] == ["Create", "map"]
assert payload["records"][1]["fields"]["level"] == 9
assert payload["records"][1]["migration_gaps"][0]["field"] == "canGenerate"
PY
if python3 scripts/convert-legacy-level-index.py \
  --input scripts/fixtures/legacy-content-level-sample.lua \
  --input scripts/fixtures/legacy-content-level-sample.lua \
  --output "$temp_dir/duplicate.json" >/dev/null 2>&1; then
  printf '%s\n' 'duplicate level IDs must be rejected' >&2
  exit 1
fi

legacy_repo=${DRL_LEGACY_REPO:-../doom-the-roughlike-original}
if [ -d "$legacy_repo/.git" ]; then
  python3 scripts/convert-legacy-level-index.py \
    --legacy-repo "$legacy_repo" \
    --revision 17d9be1204751899b2d69d8d3a2dde247bd0cc5c \
    --output "$temp_dir/pinned-levels.json" >/dev/null
  python3 - "$temp_dir/pinned-levels.json" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["record_kind"] == "level"
assert len(payload["sources"]) == 24
assert len(payload["records"]) == 26
assert payload["records"][0]["id"] == "abyssal_plains"
assert payload["records"][-1]["id"] == "unholy_cathedral"
assert all(source["revision"] == "17d9be1204751899b2d69d8d3a2dde247bd0cc5c" for source in payload["sources"])
PY
else
  printf '%s\n' 'Pinned level probe: NOT_RUN (legacy checkout unavailable).'
fi

printf '%s\n' 'Legacy level index contract: PASS (metadata, long strings, provenance, and gaps).'
