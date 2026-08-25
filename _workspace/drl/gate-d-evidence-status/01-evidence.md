predecessor: 00-scope.md
run_id: gate-d-evidence-status-7449214
revision: 744921409168bd50c24df3c96892273f45be2457
status: PASS

# Evidence inspected

- `docs/legacy-behavior/medical-powerarmor.md`:
  pinned callback decomposition and current typed timer transition.
- `docs/legacy-behavior/subtle-knife.md`:
  pinned alternate-invoke costs, target ordering, and typed transition.
- `docs/legacy-behavior/trigun.md`:
  pinned confirmation, resource costs, nuke transition, and typed action.
- `crates/drl-core/src/behavior.rs`, `subtle_knife.rs`, and `trigun.rs`:
  explicit pure transitions with no dynamic callback registry.
- `crates/drl-core/tests/special_items.rs` and unit tests in the three modules:
  deterministic success, rejection, edge, replay, and event-order coverage.
- `docs/DRL-Rust_Project_Roadmap.md` M9 behavior checklist: all three selected
  cases already recorded as behavior-covered with parity boundaries open.

## Boundary

The evidence supports current Rust behavior coverage for the selected cases,
not controlled legacy runtime parity or presentation parity. Those remain
`NOT_RUN`/open.
