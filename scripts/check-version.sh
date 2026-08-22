#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

base=${1:-${DRL_VERSION_BASE:-}}

python3 - "$base" <<'PY'
import pathlib
import re
import subprocess
import sys
import tomllib

root = pathlib.Path.cwd()
base = sys.argv[1]
version_path = root / "VERSION"
version = version_path.read_text().strip()
match = re.fullmatch(r"([0-9]+)\.([0-9]+)\.([0-9]+)", version)
if match is None:
    raise SystemExit("VERSION must contain exactly x.y.z with non-negative integers")
current = tuple(int(part) for part in match.groups())

root_manifest = tomllib.loads((root / "Cargo.toml").read_text())
workspace_version = root_manifest["workspace"]["package"]["version"]
if workspace_version != version:
    raise SystemExit("Cargo workspace package version does not match VERSION")

for manifest_path in sorted(root.glob("crates/*/Cargo.toml")):
    manifest = tomllib.loads(manifest_path.read_text())
    package = manifest.get("package", {})
    package_version = package.get("version")
    if package_version is not None and package_version != version and package_version != {
        "workspace": True
    }:
        raise SystemExit(f"package version does not match VERSION: {manifest_path}")

for name, dependency in root_manifest["workspace"]["dependencies"].items():
    if dependency.get("path") and dependency.get("version") != version:
        raise SystemExit(f"workspace dependency version does not match VERSION: {name}")

def parse(raw):
    parsed = re.fullmatch(r"([0-9]+)\.([0-9]+)\.([0-9]+)", raw.strip())
    if parsed is None:
        raise SystemExit(f"invalid version in comparison base: {raw!r}")
    return tuple(int(part) for part in parsed.groups())

def is_code_path(path):
    if path in {"VERSION", "Cargo.toml", "Cargo.lock"}:
        return False
    if path.startswith(("docs/", ".agents/", ".github/")):
        return False
    if path.endswith((".md", ".toml", ".lock", ".json", ".webmanifest", ".yml", ".yaml")):
        return False
    return pathlib.PurePosixPath(path).suffix in {
        ".css",
        ".html",
        ".js",
        ".py",
        ".rs",
        ".sh",
        ".wgsl",
    }

if base:
    try:
        try:
            base_raw = subprocess.check_output(
                ["git", "show", f"{base}:VERSION"], text=True, stderr=subprocess.DEVNULL
            )
        except subprocess.CalledProcessError:
            base_manifest = subprocess.check_output(
                ["git", "show", f"{base}:Cargo.toml"], text=True
            )
            base_raw = tomllib.loads(base_manifest)["workspace"]["package"]["version"]
        base_version = parse(base_raw)
        changed_paths = subprocess.check_output(
            ["git", "diff", "--name-only", base], text=True
        ).splitlines()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"unable to inspect version comparison base {base!r}: {error}")

    code_changed = any(is_code_path(path) for path in changed_paths)
    if code_changed:
        if current[0] > base_version[0]:
            expected = (base_version[0] + 1, 0, 0)
        elif current[0] == base_version[0] and current[1] > base_version[1]:
            expected = (base_version[0], base_version[1] + 1, 0)
        elif current[:2] == base_version[:2] and current[2] > base_version[2]:
            expected = (base_version[0], base_version[1], base_version[2] + 1)
        else:
            raise SystemExit(
                "code changes require exactly one x, y, or z increment with lower digits reset"
            )
        if current != expected:
            raise SystemExit(
                f"code changes require version {expected[0]}.{expected[1]}.{expected[2]}, "
                f"found {version}"
            )
    elif current != base_version:
        raise SystemExit("document-only or setting-only changes must not bump VERSION")

print(f"Version contract: PASS ({version})")
PY
