//! Application executable entry point for DRL-Rust.

fn main() {
  println!(
    "DRL-Rust ({}, protocol {}) scaffold initialized.",
    drl_core::engine_name(),
    drl_protocol::protocol_version()
  );
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_app_initialization() {
    assert_eq!(drl_core::engine_name(), "drl-core");
    assert_eq!(drl_protocol::protocol_version(), "0.1.0");
  }
}
