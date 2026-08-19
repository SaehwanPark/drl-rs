#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

failures=0

report_failure() {
  printf '%s\n' "$1" >&2
  failures=$((failures + 1))
}

require_file() {
  if [ ! -f "$1" ]; then
    report_failure "Missing required harness file: $1"
  fi
}

require_heading() {
  file=$1
  heading=$2

  if ! awk -v expected="$heading" '
    $0 == expected {
      found = 1
    }
    END {
      exit found ? 0 : 1
    }
  ' "$file"; then
    report_failure "Missing required heading '$heading' in $file"
  fi
}

require_text() {
  file=$1
  text=$2

  if ! awk -v expected="$text" '
    index($0, expected) {
      found = 1
    }
    END {
      exit found ? 0 : 1
    }
  ' "$file"; then
    report_failure "Missing required text '$text' in $file"
  fi
}

frontmatter_value() {
  file=$1
  key=$2

  awk -v key="$key" '
    NR == 1 {
      if ($0 != "---") {
        exit 2
      }
      in_frontmatter = 1
      next
    }
    in_frontmatter && $0 == "---" {
      exit
    }
    in_frontmatter && index($0, key ": ") == 1 {
      print substr($0, length(key) + 3)
      found = 1
      exit
    }
    END {
      if (!found) {
        exit 1
      }
    }
  ' "$file"
}

for skill in \
  .agents/skills/drl-milestone-delivery/SKILL.md \
  .agents/skills/drl-legacy-archaeology/SKILL.md \
  .agents/skills/drl-test-play/SKILL.md \
  .agents/skills/drl-determinism-review/SKILL.md
do
  if [ ! -f "$skill" ]; then
    report_failure "Missing required harness skill: $skill"
    continue
  fi

  skill_dir=${skill%/SKILL.md}
  expected_name=${skill_dir##*/}

  if ! name=$(frontmatter_value "$skill" name); then
    report_failure "Missing valid frontmatter name in $skill"
    continue
  fi

  if [ "$name" != "$expected_name" ]; then
    report_failure "Skill name '$name' does not match directory '$expected_name'"
  fi

  if ! description=$(frontmatter_value "$skill" description); then
    report_failure "Missing valid frontmatter description in $skill"
  elif [ -z "$description" ]; then
    report_failure "Empty frontmatter description in $skill"
  fi

  if ! awk '
    NR == 1 && $0 == "---" {
      opened = 1
      next
    }
    opened && $0 == "---" {
      closed = 1
      exit
    }
    END {
      exit closed ? 0 : 1
    }
  ' "$skill"; then
    report_failure "Unclosed YAML frontmatter in $skill"
  fi

  require_heading "$skill" "## When to Use"
  require_heading "$skill" "## Required Inputs"
  require_heading "$skill" "## Outputs"
  require_heading "$skill" "## Validation"
done

for path in \
  AGENTS.md \
  SPEC.md \
  ARCHITECTURE.md \
  CHANGELOG.md \
  docs/DRL-Rust_Project_Roadmap.md \
  docs/harness/drl-delivery/team-spec.md \
  docs/harness/drl-delivery/validation-scenarios.md \
  .agents/skills/drl-milestone-delivery/SKILL.md \
  .agents/skills/drl-legacy-archaeology/SKILL.md \
  .agents/skills/drl-test-play/SKILL.md \
  .agents/skills/drl-test-play/references/test-play-modes.md \
  .agents/skills/drl-determinism-review/SKILL.md
do
  require_file "$path"
done

team_spec=docs/harness/drl-delivery/team-spec.md
milestone_skill=.agents/skills/drl-milestone-delivery/SKILL.md

for artifact in \
  00-scope.md \
  01-evidence.md \
  02-test-plan.md \
  03-review.md \
  04-verification.md \
  final-handoff.md
do
  require_text "$team_spec" "$artifact"
  require_text "$milestone_skill" "$artifact"
done

for status in PASS FAIL INCONCLUSIVE NOT_RUN; do
  require_text "$team_spec" "$status"
  require_text ".agents/skills/drl-test-play/SKILL.md" "$status"
done

for field in "run identifier" "predecessor artifact" "revision lineage"; do
  require_text "$team_spec" "$field"
  require_text "$milestone_skill" "$field"
done

require_text ".agents/skills/drl-legacy-archaeology/SKILL.md" "01-evidence.md"
require_text ".agents/skills/drl-test-play/SKILL.md" "02-test-plan.md"
require_text ".agents/skills/drl-test-play/SKILL.md" "04-verification.md"
require_text ".agents/skills/drl-determinism-review/SKILL.md" "03-review.md"

for skill in \
  .agents/skills/drl-legacy-archaeology/SKILL.md \
  .agents/skills/drl-test-play/SKILL.md \
  .agents/skills/drl-determinism-review/SKILL.md
do
  require_text "$skill" "run identifier"
  require_text "$skill" "predecessor artifact"
done

if [ "$failures" -ne 0 ]; then
  printf '%s\n' "Agent harness validation failed with $failures issue(s)." >&2
  exit 1
fi

printf '%s\n' "Agent harness checks passed."
