#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

first=$(cargo run -q -p drl-app -- cohort --seed 12 --episodes 2 --max-turns 20 --bot explorer)
second=$(cargo run -q -p drl-app -- cohort --seed 12 --episodes 2 --max-turns 20 --bot explorer)
test "$first" = "$second"
printf '%s\n' "$first" | grep -F 'cohort.schema_version=1' >/dev/null
printf '%s\n' "$first" | grep -F 'cohort.seed_end=13' >/dev/null
printf '%s\n' "$first" | grep -F 'telemetry.total_damage_dealt=' >/dev/null

matrix_first=$(cargo run -q -p drl-app -- cohort --seed 4 --episodes 1 --max-turns 8 --bot all)
matrix_second=$(cargo run -q -p drl-app -- cohort --seed 4 --episodes 1 --max-turns 8 --bot all)
test "$matrix_first" = "$matrix_second"
printf '%s\n' "$matrix_first" | grep -F 'matrix.schema_version=1' >/dev/null
printf '%s\n' "$matrix_first" | grep -F 'matrix.bots=greedy,random,explorer' >/dev/null
printf '%s\n' "$matrix_first" | grep -F 'matrix.2.cohort.policy=ExplorerBot' >/dev/null

printf '%s\n' 'Cohort study CLI contract: PASS (bounded deterministic report and policy matrix).'
