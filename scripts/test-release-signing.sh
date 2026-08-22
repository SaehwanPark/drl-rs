#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-release-signing.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

printf '%s\n' '{"schema_version":1,"project_version":"test"}' > "$temp_dir/release-manifest.json"
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$temp_dir/private.pem" 2>/dev/null
RELEASE_DIST="$temp_dir" RELEASE_SIGNING_KEY="$temp_dir/private.pem" \
  "$repo_root/scripts/sign-release-manifest.sh" >/dev/null
openssl dgst -sha256 -verify "$temp_dir/release-manifest.pub" \
  -signature "$temp_dir/release-manifest.sig" "$temp_dir/release-manifest.json" >/dev/null

printf '%s\n' '{"schema_version":1,"project_version":"mutated"}' > "$temp_dir/release-manifest.json"
if openssl dgst -sha256 -verify "$temp_dir/release-manifest.pub" \
  -signature "$temp_dir/release-manifest.sig" "$temp_dir/release-manifest.json" >/dev/null 2>&1; then
  printf '%s\n' 'expected signature verification failure after manifest mutation' >&2
  exit 1
fi

printf '%s\n' 'Release signing tests: PASS (sign, verify, mutation rejection).'
