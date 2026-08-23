#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
node scripts/test-offline-cache.mjs
