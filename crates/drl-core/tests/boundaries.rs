//! Invariant tests ensuring architectural isolation of `drl-core`.

#[test]
fn test_core_properties() {
  assert_eq!(drl_core::engine_name(), "drl-core");
}

#[test]
fn test_manifest_dependency_boundaries() {
  let core_manifest = include_str!("../Cargo.toml");
  let protocol_manifest = include_str!("../../drl-protocol/Cargo.toml");

  for disallowed in ["drl-render", "drl-audio", "drl-mcp", "drl-app"] {
    assert!(
      !core_manifest.contains(disallowed),
      "drl-core must not depend on disallowed crate: {disallowed}"
    );
    assert!(
      !protocol_manifest.contains(disallowed),
      "drl-protocol must not depend on disallowed crate: {disallowed}"
    );
  }
}
