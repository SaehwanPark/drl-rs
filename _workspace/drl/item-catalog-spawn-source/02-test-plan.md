predecessor: 01-evidence.md
run_id: item-catalog-spawn-source-01c45a3
status: PASS

# Test plan

1. Run focused `drl-protocol` replay/item tests for stable-name round trips,
   loose-ammo count requirements, inverse spawn reconstruction, and catalog
   order.
2. Run `cargo test -p drl-protocol --quiet` and the full workspace suite.
3. Run `sh scripts/check-repository.sh`, `sh scripts/check-version.sh`, and
   `git diff --check`.
4. Ask the determinism/content reviewer to inspect catalog identity, replay
   boundaries, and evidence claims before handoff.

Acceptance requires unchanged public variant names and deterministic order,
with no gameplay-semantics version change.
