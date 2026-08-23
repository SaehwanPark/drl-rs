#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
dist_dir=${RELEASE_DIST:-"$repo_root/dist"}
signing_key=${RELEASE_SIGNING_KEY:-}
manifest="$dist_dir/release-manifest.json"
signature="$dist_dir/release-manifest.sig"
public_key="$dist_dir/release-manifest.pub"

if [ -z "$signing_key" ]; then
  printf '%s\n' 'Release signature: NOT_RUN (RELEASE_SIGNING_KEY is not configured).'
  exit 0
fi
command -v openssl >/dev/null 2>&1 || {
  printf '%s\n' 'Release signature: FAIL (openssl is required when signing is configured).' >&2
  exit 1
}
test -s "$signing_key" || {
  printf '%s\n' "Release signature: FAIL (private key is missing: $signing_key)." >&2
  exit 1
}
python3 - "$signing_key" "$dist_dir" <<'PY'
import pathlib
import stat
import sys

key_argument = pathlib.Path(sys.argv[1])
dist = pathlib.Path(sys.argv[2]).resolve()
if key_argument.is_symlink():
    raise SystemExit("Release signature: FAIL (private key must not be a symlink).")
try:
    key = key_argument.resolve(strict=True)
except FileNotFoundError as error:
    raise SystemExit("Release signature: FAIL (private key path is invalid).") from error
if key == dist or dist in key.parents:
    raise SystemExit("Release signature: FAIL (private key must be outside the release directory).")
key_stat = key.stat()
if not stat.S_ISREG(key_stat.st_mode):
    raise SystemExit("Release signature: FAIL (private key must be a regular file).")
if stat.S_IMODE(key_stat.st_mode) & 0o077:
    raise SystemExit("Release signature: FAIL (private key must not be group/world-readable).")
PY
test -s "$manifest" || {
  printf '%s\n' "Release signature: FAIL (manifest is missing: $manifest)." >&2
  exit 1
}

openssl dgst -sha256 -sign "$signing_key" -out "$signature" "$manifest"
openssl pkey -in "$signing_key" -pubout -out "$public_key" 2>/dev/null
printf 'Release signature: PASS (%s)\n' "$signature"
