#!/bin/sh
# Fedora development-host checks.
#
# Scope policy: the macOS and Linux jobs already run the complete repository
# verification. This job adds only the evidence those jobs cannot provide: that a
# clean Fedora userland builds the platform-adjacent crates without extra system
# packages, and that the deterministic kernel contracts hold there. GPU-dependent
# behavior is reported as NOT_RUN instead of being inferred from a container.
set -eu

cd "$(dirname "$0")/.."

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Fedora development host requires %s.\n' "$1" >&2
    exit 1
  fi
}

for tool in cargo git rustc; do
  require_tool "$tool"
done

printf '== development host\n'
sed -n 's/^\(NAME\|VERSION\|ID\)=/\1=/p' /etc/os-release
printf 'arch=%s\n' "$(uname -m)"
printf 'rustc=%s\n' "$(rustc -vV | sed -n 's/^version: //p')"
printf 'cargo=%s\n' "$(cargo -V)"

printf '== platform-adjacent crates build without extra system packages\n'
# drl-render and drl-audio sit next to the future native shell; drl-web is checked
# natively here so a platform-only regression cannot hide behind the WASM target.
cargo check --locked -p drl-render -p drl-audio -p drl-web

printf '== deterministic kernel contracts\n'
cargo test --locked -p drl-core -p drl-protocol

printf '== host capability probe\n'
# A probe reports what the environment offers; it never converts an absent device
# into a pass or a failure for GPU-dependent behavior.
if [ -e /dev/dri ]; then
  printf 'dri=device-present\n'
else
  printf 'dri=absent\n'
fi
if ldconfig -p 2>/dev/null | grep -q 'libvulkan\.so'; then
  printf 'vulkan=library-present\n'
else
  printf 'vulkan=library-absent\n'
fi
printf 'wayland_session=%s\n' "${WAYLAND_DISPLAY:-absent}"
printf 'gpu_and_wayland_acceptance=NOT_RUN\n'

printf 'Fedora development-host checks passed.\n'
