#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
legacy_repo=${DRL_LEGACY_REPO:-"$repo_root/../doom-the-roughlike-original"}
revision=${DRL_LEGACY_REVISION:-17d9be1204751899b2d69d8d3a2dde247bd0cc5c5}
legacy_binary=${DRL_LEGACY_BINARY:-"$legacy_repo/bin/drl"}
capture_root="$repo_root/_workspace/reference-captures"
manifest="$capture_root/manifest.txt"
configuration=${DRL_CAPTURE_CONFIGURATION:-"HQ graphics; fixed resolution"}
frontend=${DRL_CAPTURE_FRONTEND:-"legacy executable"}
viewport=${DRL_CAPTURE_VIEWPORT:-"record-before-running"}
dpr=${DRL_CAPTURE_DPR:-"record-before-running"}
scenario=${DRL_CAPTURE_SCENARIO:-"fixed M4-compatible setup"}
actions=${DRL_CAPTURE_ACTIONS:-"record-before-running"}
capture_tool=${DRL_CAPTURE_TOOL:-"record-before-running"}
capture_tool_version=${DRL_CAPTURE_TOOL_VERSION:-"record-before-running"}
media_hashes=${DRL_CAPTURE_MEDIA_HASHES:-"record-after-capture"}

mkdir -p "$capture_root"
platform=$(uname -s)-$(uname -m)
status=NOT_RUN
reason='legacy executable is unavailable or not executable'
binary_hash=unavailable
if test -x "$legacy_binary"; then
  if command -v sha256sum >/dev/null 2>&1; then
    binary_hash=$(sha256sum "$legacy_binary" | awk '{print $1}')
  else
    binary_hash=$(shasum -a 256 "$legacy_binary" | awk '{print $1}')
  fi
  reason='legacy executable requires Linux x86-64 capture host'
  if [ "$platform" = "Linux-x86_64" ]; then
    status=READY_FOR_CONTROLLED_CAPTURE
    reason='capture command must be supplied by the controlled environment'
  fi
fi
{
  printf '%s\n' "status=$status"
  printf '%s\n' "legacy_repository=$legacy_repo"
  printf '%s\n' "legacy_revision=$revision"
  printf '%s\n' "executable=$legacy_binary"
  printf '%s\n' "executable_sha256=$binary_hash"
  printf '%s\n' "capture_host=$platform"
  printf '%s\n' "reason=$reason"
  printf '%s\n' "frontend=$frontend"
  printf '%s\n' "configuration=$configuration"
  printf '%s\n' "viewport=$viewport"
  printf '%s\n' "dpr=$dpr"
  printf '%s\n' "scenario=$scenario"
  printf '%s\n' "actions=$actions"
  printf '%s\n' "capture_tool=$capture_tool"
  printf '%s\n' "capture_tool_version=$capture_tool_version"
  printf '%s\n' "media_hashes=$media_hashes"
  printf '%s\n' 'scenes=lighting,fog,targeting,ranged,knockback,low-health,inventory,hud,transition'
  printf '%s\n' "media_root=$capture_root/media"
} > "$manifest"
printf '%s\n' "Capture manifest prepared at $manifest; run only in a controlled Linux x86-64 environment."
