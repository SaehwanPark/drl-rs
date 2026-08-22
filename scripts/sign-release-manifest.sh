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
test -s "$manifest" || {
  printf '%s\n' "Release signature: FAIL (manifest is missing: $manifest)." >&2
  exit 1
}

openssl dgst -sha256 -sign "$signing_key" -out "$signature" "$manifest"
openssl pkey -in "$signing_key" -pubout -out "$public_key" 2>/dev/null
printf 'Release signature: PASS (%s)\n' "$signature"
