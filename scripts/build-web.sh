#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
dist="$PWD/dist"
rm -rf "$dist"
mkdir -p "$dist"

if ! command -v wasm-pack >/dev/null 2>&1; then
  printf '%s\n' 'wasm-pack 0.15.0 is required to build the web bundle.' >&2
  exit 1
fi

wasm_pack_version=$(wasm-pack --version | awk '{print $2}')
test "$wasm_pack_version" = "0.15.0" || {
  printf '%s\n' "expected wasm-pack 0.15.0, found $wasm_pack_version" >&2
  exit 1
}

wasm-pack build crates/drl-web --target web --release --out-dir "$dist/pkg"
cp web/index.html web/bootstrap.js web/manifest.webmanifest "$dist/"
mkdir -p "$dist/assets/legacy"
cp -R assets/legacy/drl "$dist/assets/legacy/"
python3 - "$dist" web/service-worker.js <<'PY'
import json
import pathlib
import sys

dist = pathlib.Path(sys.argv[1])
template = pathlib.Path(sys.argv[2]).read_text()
files = ["./", "./service-worker.js"]
files.extend(
    f"./{path.relative_to(dist).as_posix()}"
    for path in sorted(dist.rglob("*"))
    if path.is_file()
)
marker = "/* __PRECACHE_URLS__ */ []"
if marker not in template:
    raise SystemExit("service-worker precache marker is missing")
(dist / "service-worker.js").write_text(
    template.replace(marker, json.dumps(files, separators=(",", ":")))
)
PY
printf '%s\n' "Web bundle written to $dist (ignored by git)."
