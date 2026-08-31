#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

output_file=$(mktemp)
trap 'rm -f "$output_file"' EXIT HUP INT TERM

revision=$(git rev-parse HEAD)
rust_version=$(rustc --version)
DRL_BENCH_REVISION="$revision" \
DRL_BENCH_RUST_VERSION="$rust_version" \
  cargo bench --quiet --locked -p drl-core --bench transaction -- --contract >"$output_file"

python3 - "$output_file" "$revision" "$rust_version" <<'PY'
import json
import pathlib
import sys

records = [
    json.loads(line)
    for line in pathlib.Path(sys.argv[1]).read_text().splitlines()
    if line
]
metadata = [record for record in records if record.get("kind") == "metadata"]
if len(metadata) != 1:
    raise SystemExit("transaction benchmark must emit one metadata record")

metadata = metadata[0]
if metadata.get("schema_version") != 1:
    raise SystemExit("unexpected transaction benchmark schema")
if metadata.get("benchmark") != "drl-core-transaction":
    raise SystemExit("unexpected transaction benchmark name")
if metadata.get("revision") != sys.argv[2]:
    raise SystemExit("transaction benchmark revision does not match HEAD")
if metadata.get("rust_version") != sys.argv[3]:
    raise SystemExit("transaction benchmark toolchain does not match rustc")
if metadata.get("profile") != "bench":
    raise SystemExit("transaction benchmark must use the optimized bench profile")
if metadata.get("ownership") != "core.rollback":
    raise SystemExit("transaction benchmark ownership is not core.rollback")
if metadata.get("seed") != 42:
    raise SystemExit("transaction benchmark seed must remain 42")
if metadata.get("timing_allocator_counting") is not False:
    raise SystemExit("transaction timing must disable allocator counter updates")
if metadata.get("allocation_measurement") != "separate_pass":
    raise SystemExit("transaction allocation measurement must be a separate pass")

expected_cases = {
    "core.accepted.wait",
    "core.accepted.move",
    "core.rejected.blocked_move",
    "core.rejected.out_of_bounds_ranged",
    "core.rejected.late_death_drop",
}
measurements = [record for record in records if "case" in record]
if {record.get("case") for record in measurements} != expected_cases:
    raise SystemExit("transaction benchmark case set changed")

for case in expected_cases:
    rows = [record for record in measurements if record["case"] == case]
    samples = [record for record in rows if record.get("median") is False]
    medians = [record for record in rows if record.get("median") is True]
    if len(samples) != 3 or len(medians) != 1 or {record.get("sample") for record in samples} != {0, 1, 2}:
        raise SystemExit(f"{case} must have three samples and one median record")
    median = medians[0]
    ordered = sorted(samples, key=lambda record: record["elapsed_ns"])
    if median["elapsed_ns"] != ordered[len(ordered) // 2]["elapsed_ns"]:
        raise SystemExit(f"{case} median elapsed time is inconsistent")
    for record in rows:
        if record.get("schema_version") != 1:
            raise SystemExit(f"{case} has an invalid schema")
        if record.get("iterations") != 1000 or record.get("warmup") != 100:
            raise SystemExit(f"{case} did not honor contract bounds")
        if record.get("ownership") != "core.rollback":
            raise SystemExit(f"{case} has an invalid ownership label")
        if record.get("seed") != 42:
            raise SystemExit(f"{case} did not use fixed seed 42")
        if record.get("timing_allocator_counting") is not False:
            raise SystemExit(f"{case} timing unexpectedly counted allocator updates")
        if record.get("allocation_measurement") != "separate_pass":
            raise SystemExit(f"{case} allocation measurement is not separate")
        if record.get("elapsed_ns", 0) <= 0:
            raise SystemExit(f"{case} did no measurable work")
        if record.get("ns_per_operation") != record["elapsed_ns"] // record["iterations"]:
            raise SystemExit(f"{case} has inconsistent ns/op arithmetic")
        if record.get("operations_per_second") != (record["iterations"] * 1_000_000_000) // record["elapsed_ns"]:
            raise SystemExit(f"{case} has inconsistent throughput arithmetic")
        for field in ("allocations", "deallocations", "allocated_bytes", "deallocated_bytes"):
            if record.get(field, -1) < 0:
                raise SystemExit(f"{case} has an invalid {field} counter")

print(f"Transaction benchmark contract: PASS ({len(expected_cases)} fixed cases)")
PY
