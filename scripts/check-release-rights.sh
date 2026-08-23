#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
rights_doc=${RELEASE_RIGHTS_DOC:-$repo_root/docs/release-rights.md}
bundle=${RELEASE_DIST:-$repo_root/dist}
graphics_dir=${DRL_GRAPHICS_DIR:-$repo_root/assets/legacy/drl/graphics}

fail() {
  printf '%s\n' "Release rights check: FAIL ($1)" >&2
  exit 1
}

test -s "$rights_doc" || fail "rights inventory is missing"

required_lines='category: project-authored
status: INCLUDED
license: MIT
category: bundled-legacy-graphics
status: INCLUDED
license: CC BY-SA 4.0
source_revision: 17d9be1204751899b2d69d8d3a2dde247bd0cc5c
license_path: assets/legacy/drl/graphics/LICENSE
manifest_path: assets/legacy/drl/graphics/MANIFEST.txt
checksum_path: assets/legacy/drl/graphics/SHA256SUMS
category: legacy-code
status: EXCLUDED
category: legacy-audio-music-fonts
status: EXCLUDED
category: captures-and-media
status: NOT_RUN
category: third-party-dependencies
status: NOTICE-ONLY'

printf '%s\n' "$required_lines" | while IFS= read -r line; do
  grep -Fqx "$line" "$rights_doc" || fail "inventory field is missing: $line"
done

DRL_GRAPHICS_DIR="$graphics_dir" sh "$repo_root/scripts/check-assets.sh" >/dev/null || fail "graphics provenance check failed"

if [ ! -d "$bundle" ]; then
  printf '%s\n' 'Release rights check: PASS (source inventory and graphics provenance; bundle NOT_RUN)'
  exit 0
fi

python3 - "$bundle" "$graphics_dir" <<'PY'
import hashlib
import json
import pathlib
import sys

bundle = pathlib.Path(sys.argv[1])
source_graphics = pathlib.Path(sys.argv[2])
rights_path = pathlib.PurePosixPath("assets/legacy/drl/graphics/LICENSE")
required_graphics = [
    rights_path,
    pathlib.PurePosixPath("assets/legacy/drl/graphics/MANIFEST.txt"),
    pathlib.PurePosixPath("assets/legacy/drl/graphics/SHA256SUMS"),
]
for relative in required_graphics:
    if not (bundle / relative).is_file():
        raise SystemExit(f"bundle is missing declared graphics evidence: {relative}")

source_files = {
    path.relative_to(source_graphics).as_posix(): path
    for path in source_graphics.rglob("*")
    if path.is_file()
}
bundle_graphics = bundle / "assets/legacy/drl/graphics"
bundle_files = {}
for path in bundle_graphics.rglob("*"):
    if path.is_symlink():
        raise SystemExit(f"bundle contains a symlink: {path.relative_to(bundle)}")
    if path.is_file():
        bundle_files[path.relative_to(bundle_graphics).as_posix()] = path
    elif not path.is_dir():
        raise SystemExit(f"bundle contains a non-regular graphics entry: {path.relative_to(bundle)}")
if set(bundle_files) != set(source_files):
    raise SystemExit("bundle graphics file set does not match the pinned source")
for name, source_path in source_files.items():
    source_digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
    bundle_digest = hashlib.sha256(bundle_files[name].read_bytes()).hexdigest()
    if source_digest != bundle_digest:
        raise SystemExit(f"bundle graphics checksum mismatch: {name}")

manifest_path = bundle / "release-manifest.json"
if not manifest_path.is_file():
    raise SystemExit("bundle is missing release-manifest.json")
try:
    manifest = json.loads(manifest_path.read_text())
except (OSError, json.JSONDecodeError) as error:
    raise SystemExit("bundle release-manifest.json is not valid JSON") from error
if manifest.get("rights") != [rights_path.as_posix()]:
    raise SystemExit("bundle release manifest rights declaration is not exact")

for path in bundle.rglob("*"):
    if path.is_symlink():
        raise SystemExit(f"bundle contains a symlink: {path.relative_to(bundle)}")
    if not path.is_file():
        continue
    relative = path.relative_to(bundle).as_posix()
    parts = set(path.relative_to(bundle).parts)
    suffix = path.suffix.lower()
    if "legacy-drlhq" in parts or suffix in {
        ".aac", ".flac", ".lua", ".m4a", ".mid", ".midi", ".mod", ".mp3",
        ".ogg", ".otf", ".pas", ".s3m", ".ttf", ".wad", ".wav", ".woff",
        ".woff2", ".xm",
    } or {"audio", "captures", "font", "fonts", "music", "sound"} & parts:
        raise SystemExit(f"bundle contains excluded rights material: {relative}")

print(f"Release rights check: PASS (source inventory and bundle boundary; {len(list(bundle.rglob('*')))} bundle entries inspected)")
PY
