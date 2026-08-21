#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
test -d dist || { printf '%s\n' 'Run scripts/build-web.sh first.' >&2; exit 1; }
exec python3 -m http.server "${DRL_WEB_PORT:-8080}" --directory dist
