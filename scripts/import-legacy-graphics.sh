#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
legacy_repo=${DRL_LEGACY_REPO:-"$repo_root/../doom-the-roughlike-original"}
revision=${DRL_LEGACY_REVISION:-17d9be1204751899b2d69d8d3a2dde247bd0cc5c}
destination="$repo_root/assets/legacy/drl/graphics"

if ! git -C "$legacy_repo" cat-file -e "$revision^{commit}" 2>/dev/null; then
  printf '%s\n' "legacy revision is unavailable: $revision" >&2
  exit 1
fi

mkdir -p "$destination"
# This directory is generated solely by this importer. Clear prior generated
# files so a rerun cannot silently retain an asset absent from the pinned tree.
find "$destination" -maxdepth 1 -type f -delete
git -C "$legacy_repo" ls-tree -r --name-only "$revision" -- bin/data/drl/graphics/ \
  | while IFS= read -r path; do
      file=${path##*/}
      git -C "$legacy_repo" show "$revision:$path" > "$destination/$file"
    done

{
  printf '%s\n' "legacy_revision=$revision"
  printf '%s\n' "source_path=bin/data/drl/graphics"
  printf '%s\n' "license=CC BY-SA 4.0 (see LICENSE)"
  printf '%s\n' "files="
  find "$destination" -maxdepth 1 -type f ! -name MANIFEST.txt ! -name SHA256SUMS \
    -print | LC_ALL=C sort | while IFS= read -r file; do
      printf '%s\n' "${file#"$destination"/}"
    done
} > "$destination/MANIFEST.txt"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$destination" && sha256sum -- *png LICENSE) > "$destination/SHA256SUMS"
else
  (cd "$destination" && shasum -a 256 -- *png LICENSE) > "$destination/SHA256SUMS"
fi

printf '%s\n' "Imported tracked legacy graphics from $revision into $destination"
