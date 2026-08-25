predecessor: 00-scope.md
run_id: exit-gate-evidence-6d9d930
revision: 6d9d930a0d377936cbc94c660ef57d93f6038234
status: PASS

# Evidence inspected

- `ARCHITECTURE.md` protocol/domain boundary: protocol owns stable semantic
  contracts; core owns simulation balance and behavior.
- `crates/drl-protocol/src/item.rs` and `replay.rs`: catalog work contains
  stable identity/spawn shape only; count-sensitive reconstruction remains an
  explicit replay boundary.
- `crates/drl-core/src/item_definition.rs` and behavior modules: gameplay
  definitions and typed transitions remain core-owned.
- PR #243 hosted checks: repository checks PASS and WASM browser checks PASS.
- Local `sh scripts/check-repository.sh`, workspace tests, and version checks
  from the predecessor loop: PASS at version `0.2.143`.

## Boundary

This evidence supports current architecture and verification claims only. It
does not establish legacy runtime or audiovisual parity.
