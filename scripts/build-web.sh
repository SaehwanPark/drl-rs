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
python3 - "$dist" web/service-worker.js VERSION <<'PY'
import hashlib
import json
import os
import pathlib
import subprocess
import sys

dist = pathlib.Path(sys.argv[1])
template = pathlib.Path(sys.argv[2]).read_text()
project_version = pathlib.Path(sys.argv[3]).read_text().strip()
manifest_path = dist / "release-manifest.json"
generated_files = ["release-manifest.json", "service-worker.js"]
artifact_paths = [
    path
    for path in sorted(dist.rglob("*"))
    if path.is_file() and path.name not in generated_files
]
try:
    source_revision = os.environ.get("DRL_BUILD_REVISION") or subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=dist.parent, text=True
    ).strip()
except (OSError, subprocess.CalledProcessError):
    source_revision = "unknown"
manifest = {
    "schema_version": 1,
    "project_version": project_version,
    "source_revision": source_revision or "unknown",
    "artifacts": [
        {
            "path": path.relative_to(dist).as_posix(),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        }
        for path in artifact_paths
    ],
    "generated": generated_files,
    "rights": ["assets/legacy/drl/graphics/LICENSE"],
}
manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
cache_version = f"v1-{manifest['source_revision'][:12]}"
cache_marker = '/* __CACHE_VERSION__ */ "v1"'
if cache_marker not in template:
    raise SystemExit("service-worker cache version marker is missing")
template = template.replace(cache_marker, json.dumps(cache_version), 1)
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
