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
cp web/index.html web/bootstrap.js "$dist/"
cp -R assets/legacy/drl "$dist/assets"
printf '%s\n' "Web bundle written to $dist (ignored by git)."
