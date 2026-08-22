#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
manifest="dist/release-manifest.json"
if [ ! -s "$manifest" ]; then
  printf '%s\n' 'Release manifest check: NOT_RUN (run scripts/build-web.sh first).'
  exit 0
fi

python3 - "$manifest" <<'PY'
import hashlib
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
dist = manifest_path.parent
data = json.loads(manifest_path.read_text())
if data.get("schema_version") != 1:
    raise SystemExit("unsupported release manifest schema")
project_version = pathlib.Path("VERSION").read_text().strip()
if data.get("project_version") != project_version:
    raise SystemExit("release manifest project version does not match VERSION")
if not isinstance(data.get("source_revision"), str) or not data["source_revision"]:
    raise SystemExit("release manifest source revision is missing")
if data.get("generated") != [
    "release-manifest.json",
    "release-manifest.sha256",
    "service-worker.js",
]:
    raise SystemExit("release manifest generated-file declaration is invalid")
if data.get("rights") != ["assets/legacy/drl/graphics/LICENSE"]:
    raise SystemExit("release manifest rights declaration is invalid")

artifacts = data.get("artifacts")
if not isinstance(artifacts, list) or not artifacts:
    raise SystemExit("release manifest has no artifacts")
paths = [entry.get("path") for entry in artifacts]
if paths != sorted(paths) or len(paths) != len(set(paths)):
    raise SystemExit("release manifest artifact order or uniqueness is invalid")
for entry in artifacts:
    path = entry.get("path")
    digest = entry.get("sha256")
    if (
        not isinstance(path, str)
        or not path
        or path.startswith("/")
        or ".." in pathlib.PurePosixPath(path).parts
        or path in data["generated"]
        or not isinstance(digest, str)
        or len(digest) != 64
        or any(character not in "0123456789abcdef" for character in digest)
    ):
        raise SystemExit(f"invalid release manifest artifact entry: {entry!r}")
    artifact = dist / pathlib.PurePosixPath(path)
    if not artifact.is_file():
        raise SystemExit(f"release manifest artifact is missing: {path}")
    actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
    if actual != digest:
        raise SystemExit(f"release manifest hash mismatch: {path}")

rights = dist / pathlib.PurePosixPath(data["rights"][0])
if not rights.is_file():
    raise SystemExit("release manifest rights file is missing")
digest_path = dist / "release-manifest.sha256"
if not digest_path.is_file():
    raise SystemExit("release manifest digest sidecar is missing")
digest_parts = digest_path.read_text().split()
if len(digest_parts) != 2 or digest_parts[1] != "release-manifest.json":
    raise SystemExit("release manifest digest sidecar is malformed")
manifest_digest = digest_parts[0]
if len(manifest_digest) != 64 or any(
    character not in "0123456789abcdef" for character in manifest_digest
):
    raise SystemExit("release manifest digest sidecar has an invalid hash")
if hashlib.sha256(manifest_path.read_bytes()).hexdigest() != manifest_digest:
    raise SystemExit("release manifest digest sidecar does not match manifest")
worker = (dist / "service-worker.js").read_text()
cache_version = f'v1-{data["project_version"]}-{data["source_revision"][:12]}'
cache_literal = json.dumps(cache_version)
if f"const CACHE_VERSION = {cache_literal};" not in worker:
    raise SystemExit("service-worker cache version does not match release manifest")
for path in [entry["path"] for entry in artifacts] + data["generated"]:
    if f'"./{path}"' not in worker:
        raise SystemExit(f"service-worker precache omits: {path}")

print(f"Release manifest check: PASS ({len(artifacts)} hashed artifacts)")
PY
