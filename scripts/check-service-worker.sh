#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

worker=web/service-worker.js
test -s "$worker"
grep -F 'CACHE_NAMESPACE = "drl-rust-m10-"' "$worker" >/dev/null
grep -F 'CACHE_VERSION = "v1"' "$worker" >/dev/null
grep -F '/* __PRECACHE_URLS__ */ []' "$worker" >/dev/null
grep -F 'request.method === "GET"' "$worker" >/dev/null
grep -F 'self.location.origin' "$worker" >/dev/null
grep -F 'cacheSuccessfulResponse' "$worker" >/dev/null
grep -F 'assets/legacy' scripts/build-web.sh >/dev/null
grep -F 'service-worker.js' scripts/build-web.sh >/dev/null
grep -F 'release-manifest.json' scripts/build-web.sh scripts/check-release-manifest.sh >/dev/null
grep -F 'service-worker.js' scripts/build-web.sh web/bootstrap.js >/dev/null
grep -F 'manifest.webmanifest' web/index.html scripts/build-web.sh >/dev/null
python3 -m json.tool web/manifest.webmanifest >/dev/null

printf '%s\n' 'Static service-worker and manifest contracts passed.'
