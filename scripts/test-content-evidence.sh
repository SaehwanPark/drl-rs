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
level_records = []
for kind in ("being", "item", "cell", "level"):
  payload = json.loads((root / f"{kind}.json").read_text(encoding="utf-8"))
  if kind == "level":
    fields = payload["records"][0]["fields"]
    fields["name"] = 'Alpha "level"'
    fields["entry"] = 'On @1 he said "hello".\\nSecond line.'
    fields["welcome"] = "Welcome\\path.\nSecond line."
    (root / "level.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
  ids = [record["id"] for record in payload["records"]]
  entry = {
    "sources": [source["path"] for source in payload["sources"]],
    "source_sha256": [source["sha256"] for source in payload["sources"]],
    "record_count": len(ids),
    "required_ids": ids,
    "record_ids": ids,
  }
  if kind == "level":
    level_records = payload["records"]
  bundles[kind] = entry
(root / "config.json").write_text(
  json.dumps(
    {"schema_version": 2, "revision": "unbound-input", "bundles": bundles},
    indent=2,
  )
  + "\n",
  encoding="utf-8",
)
def optional(value, multiline=False):
  if value is None:
    return "None"
  encoded = json.dumps(value)
  if multiline:
    return f"Some(\n      {encoded},\n    )"
  return f"Some({encoded})"


catalog = []
for record in level_records:
  fields = record["fields"]
  catalog.append(
    "\n".join(
      [
        "  SpecialLevelDefinition {",
        f'    id: "{record["id"]}",',
        f'    name: {json.dumps(fields["name"])},',
        f'    legacy_depth: {optional(fields.get("level"))},',
        f'    entry: {optional(fields.get("entry"))},',
        f'    welcome: {optional(fields.get("welcome"), multiline=True)},',
        "  },",
      ]
    )
  )
(root / "catalog.rs").write_text("\n".join(catalog) + "\n", encoding="utf-8")
PY

python3 scripts/check-content-evidence.py \
  --config "$temp_dir/config.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" \
  --rust-catalog "$temp_dir/catalog.rs" >/dev/null

check_catalog_rejected() {
  label=$1
  catalog=$2
  if python3 scripts/check-content-evidence.py --config "$temp_dir/config.json" \
    --bundle "being=$temp_dir/being.json" \
    --bundle "item=$temp_dir/item.json" \
    --bundle "cell=$temp_dir/cell.json" \
    --bundle "level=$temp_dir/level.json" \
    --rust-catalog "$catalog" >/dev/null 2>&1; then
    printf '%s\n' "$label must be rejected" >&2
    exit 1
  fi
}

python3 - "$temp_dir/catalog.rs" "$temp_dir/catalog-comments.rs" <<'PY'
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
needle = '    name: "Alpha \\"level\\"",\n'
shadow = '    /*\n    name: "Comment shadow",\n    */\n'
if needle not in source:
  raise SystemExit("fixture comment-shadow target missing")
source = "const TYPE: &'static str = \"fixture\"; // lifetime must not suppress comment stripping\n" + source
pathlib.Path(sys.argv[2]).write_text(source.replace(needle, shadow + needle, 1), encoding="utf-8")
PY
if ! python3 scripts/check-content-evidence.py --config "$temp_dir/config.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" \
  --rust-catalog "$temp_dir/catalog-comments.rs" >/dev/null; then
  printf '%s\n' 'comment-shadowed Rust fields must be ignored' >&2
  exit 1
fi

python3 - "$temp_dir/catalog.rs" "$temp_dir/catalog-nested-comment-drift.rs" <<'PY'
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
needle = '    name: "Alpha \\"level\\"",\n'
nested = (
  '    /*\n'
  '      /* nested comment */\n'
  '      name: "Alpha \\"level\\"",\n'
  '    */\n'
  '    name: "Wrong live value",\n'
)
if needle not in source:
  raise SystemExit("fixture nested-comment target missing")
pathlib.Path(sys.argv[2]).write_text(source.replace(needle, nested, 1), encoding="utf-8")
PY
check_catalog_rejected 'nested comment-shadowed live scalar drift' "$temp_dir/catalog-nested-comment-drift.rs"

python3 - "$temp_dir/catalog.rs" "$temp_dir/catalog-mismatch.rs" <<'PY'
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
pathlib.Path(sys.argv[2]).write_text(source.replace('id: "alpha",', 'id: "gamma",', 1), encoding="utf-8")
PY
check_catalog_rejected 'Rust special-level ID drift' "$temp_dir/catalog-mismatch.rs"

python3 - "$temp_dir/catalog.rs" "$temp_dir" <<'PY'
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
root = pathlib.Path(sys.argv[2])
mutations = {
  "name": ('name: "Alpha \\"level\\"",', 'name: "Drifted level",'),
  "depth": ('legacy_depth: Some(3),', 'legacy_depth: Some(4),'),
  "entry-gap": ('entry: None,', 'entry: Some("drifted entry"),'),
  "welcome-gap": ('welcome: None,', 'welcome: Some("drifted welcome"),'),
}
for label, (before, after) in mutations.items():
  if before not in source:
    raise SystemExit(f"fixture mutation target missing: {before}")
  (root / f"catalog-{label}.rs").write_text(source.replace(before, after, 1), encoding="utf-8")
PY
check_catalog_rejected 'Rust special-level name drift' "$temp_dir/catalog-name.rs"
check_catalog_rejected 'Rust special-level depth drift' "$temp_dir/catalog-depth.rs"
check_catalog_rejected 'Rust special-level entry gap drift' "$temp_dir/catalog-entry-gap.rs"
check_catalog_rejected 'Rust special-level welcome gap drift' "$temp_dir/catalog-welcome-gap.rs"

python3 - "$temp_dir/config.json" "$temp_dir/wrong-record-catalog.json" <<'PY'
import json
import pathlib
import sys

config = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
config["bundles"]["being"]["record_ids"][0] = "drifted-record"
pathlib.Path(sys.argv[2]).write_text(json.dumps(config), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/wrong-record-catalog.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'wrong evidence record catalog must be rejected' >&2
  exit 1
fi

python3 - "$temp_dir/being.json" "$temp_dir/malformed-fields.json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["records"][0]["fields"]["nested"] = {"not": "scalar"}
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/config.json" \
  --bundle "being=$temp_dir/malformed-fields.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'nested evidence fields must be rejected' >&2
  exit 1
fi

python3 - "$temp_dir/being.json" "$temp_dir/malformed-gaps.json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["records"][0]["migration_gaps"].append({"field": "nested", "line": "unknown"})
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/config.json" \
  --bundle "being=$temp_dir/malformed-gaps.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'malformed migration gaps must be rejected' >&2
  exit 1
fi

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

python3 - "$temp_dir/config.json" "$temp_dir/wrong-digest.json" <<'PY'
import json
import pathlib
import sys

config = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
config["bundles"]["being"]["source_sha256"][0] = "0" * 64
pathlib.Path(sys.argv[2]).write_text(json.dumps(config), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/wrong-digest.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'wrong evidence digest must be rejected' >&2
  exit 1
fi

python3 - "$temp_dir/config.json" "$temp_dir/wrong-schema.json" <<'PY'
import json
import pathlib
import sys

config = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
config["schema_version"] = 1
pathlib.Path(sys.argv[2]).write_text(json.dumps(config), encoding="utf-8")
PY
if python3 scripts/check-content-evidence.py --config "$temp_dir/wrong-schema.json" \
  --bundle "being=$temp_dir/being.json" \
  --bundle "item=$temp_dir/item.json" \
  --bundle "cell=$temp_dir/cell.json" \
  --bundle "level=$temp_dir/level.json" >/dev/null 2>&1; then
  printf '%s\n' 'obsolete evidence crosswalk schema must be rejected' >&2
  exit 1
fi

printf '%s\n' 'Content evidence validator contract: PASS (fixture coverage and rejection cases).'
