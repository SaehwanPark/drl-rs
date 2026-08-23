#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-release-signing.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

mkdir "$temp_dir/valid-dist"
printf '%s\n' '{"schema_version":1,"project_version":"test"}' > "$temp_dir/valid-dist/release-manifest.json"
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$temp_dir/private.pem" 2>/dev/null
RELEASE_DIST="$temp_dir/valid-dist" RELEASE_SIGNING_KEY="$temp_dir/private.pem" \
  "$repo_root/scripts/sign-release-manifest.sh" >/dev/null
openssl dgst -sha256 -verify "$temp_dir/valid-dist/release-manifest.pub" \
  -signature "$temp_dir/valid-dist/release-manifest.sig" "$temp_dir/valid-dist/release-manifest.json" >/dev/null

mkdir "$temp_dir/release-dist"
cp "$temp_dir/valid-dist/release-manifest.json" "$temp_dir/release-dist/"
cp "$temp_dir/private.pem" "$temp_dir/release-dist/"
if RELEASE_DIST="$temp_dir/release-dist" RELEASE_SIGNING_KEY="$temp_dir/release-dist/private.pem" \
  "$repo_root/scripts/sign-release-manifest.sh" >/dev/null 2>&1; then
  printf '%s\n' 'expected signing to reject a private key inside the release directory' >&2
  exit 1
fi

ln -s "$temp_dir/private.pem" "$temp_dir/private-link.pem"
if RELEASE_DIST="$temp_dir/valid-dist" RELEASE_SIGNING_KEY="$temp_dir/private-link.pem" \
  "$repo_root/scripts/sign-release-manifest.sh" >/dev/null 2>&1; then
  printf '%s\n' 'expected signing to reject a symlinked private key' >&2
  exit 1
fi

chmod 0644 "$temp_dir/private.pem"
if RELEASE_DIST="$temp_dir/valid-dist" RELEASE_SIGNING_KEY="$temp_dir/private.pem" \
  "$repo_root/scripts/sign-release-manifest.sh" >/dev/null 2>&1; then
  printf '%s\n' 'expected signing to reject a group/world-readable private key' >&2
  exit 1
fi

printf '%s\n' '{"schema_version":1,"project_version":"mutated"}' > "$temp_dir/valid-dist/release-manifest.json"
if openssl dgst -sha256 -verify "$temp_dir/valid-dist/release-manifest.pub" \
  -signature "$temp_dir/valid-dist/release-manifest.sig" "$temp_dir/valid-dist/release-manifest.json" >/dev/null 2>&1; then
  printf '%s\n' 'expected signature verification failure after manifest mutation' >&2
  exit 1
fi

printf '%s\n' 'Release signing tests: PASS (sign, verify, mutation rejection).'
