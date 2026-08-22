#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
expected_revision=17d9be1204751899b2d69d8d3a2dde247bd0cc5c
manifest=${DRL_CAPTURE_MANIFEST:-"$repo_root/_workspace/reference-captures/manifest.txt"}
required_keys='status legacy_repository legacy_revision executable executable_sha256 capture_host reason frontend configuration viewport dpr scenario actions capture_tool capture_tool_version media_hashes scenes media_root'
required_scenes='lighting fog targeting ranged knockback low-health inventory hud transition'
placeholder_fields='viewport dpr scenario actions capture_tool capture_tool_version media_hashes'

if [ ! -f "$manifest" ]; then
  printf '%s\n' "Reference capture preflight: NOT_RUN (manifest missing: $manifest)"
  exit 0
fi

errors=0
error() {
  printf 'Reference capture preflight: %s\n' "$1" >&2
  errors=1
}

field() {
  awk -F= -v key="$1" '$1 == key { sub(/^[^=]*=/, ""); print; exit }' "$manifest"
}

for key in $required_keys; do
  count=$(grep -c "^${key}=" "$manifest" || true)
  if [ "$count" -ne 1 ]; then
    error "expected exactly one ${key}= entry (found $count)"
  elif [ -z "$(field "$key")" ]; then
    error "${key} must be non-empty"
  fi
done

status=$(field status)
legacy_repository=$(field legacy_repository)
legacy_revision=$(field legacy_revision)
executable=$(field executable)
executable_sha256=$(field executable_sha256)
capture_host=$(field capture_host)
reason=$(field reason)
scenes=$(field scenes)

if [ "$legacy_revision" != "$expected_revision" ]; then
  error "legacy_revision must be $expected_revision"
fi

case "$status" in
  NOT_RUN|READY_FOR_CONTROLLED_CAPTURE|PASS|INCONCLUSIVE|FAIL) ;;
  *) error "unsupported status: $status" ;;
esac

if [ -z "$reason" ]; then
  error 'reason must be non-empty'
fi

for scene in $required_scenes; do
  case ",$scenes," in
    *,"$scene",*) ;;
    *) error "required fidelity scene is missing: $scene" ;;
  esac
done

placeholder_found=0
for key in $placeholder_fields; do
  value=$(field "$key")
  case "$value" in
    *record-before-running*|*record-after-capture*) placeholder_found=1 ;;
  esac
done

case "$status" in
  READY_FOR_CONTROLLED_CAPTURE|PASS)
    if [ "$placeholder_found" -ne 0 ]; then
      error "$status manifests cannot retain capture placeholders"
    fi
    if [ "$capture_host" != 'Linux-x86_64' ]; then
      error "$status manifests require capture_host=Linux-x86_64"
    fi
    if [ ! -x "$executable" ]; then
      error "executable is not available/executable: $executable"
    fi
    ;;
  NOT_RUN|INCONCLUSIVE)
    ;;
  FAIL)
    error 'manifest status is FAIL'
    ;;
esac

if [ -d "$legacy_repository/.git" ]; then
  actual_revision=$(git -C "$legacy_repository" rev-parse HEAD 2>/dev/null || true)
  if [ "$actual_revision" != "$expected_revision" ]; then
    error 'legacy checkout HEAD does not match pinned revision'
  fi
fi

if [ -f "$executable" ] && [ "$executable_sha256" != 'unavailable' ]; then
  if command -v sha256sum >/dev/null 2>&1; then
    actual_hash=$(sha256sum "$executable" | awk '{print $1}')
  else
    actual_hash=$(shasum -a 256 "$executable" | awk '{print $1}')
  fi
  if [ "$actual_hash" != "$executable_sha256" ]; then
    error 'executable_sha256 does not match the executable'
  fi
fi

if [ "$errors" -ne 0 ]; then
  exit 1
fi

printf 'Reference capture preflight: %s (%s)\n' "$status" "$manifest"
