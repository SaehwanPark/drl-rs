//! Process-level regression for bounded MCP input handling.

use std::io::Write;
use std::process::{Command, Stdio};

use drl_mcp::json::JsonValue;
use drl_mcp::protocol::{MAX_MCP_JSON_DEPTH, error_codes};

#[test]
fn deeply_nested_mcp_frame_returns_error_without_aborting_process() {
  let request = format!(
    "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"params\":{}0{}}}\n{{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}}\n",
    "[".repeat(MAX_MCP_JSON_DEPTH + 8),
    "]".repeat(MAX_MCP_JSON_DEPTH + 8)
  );
  let binary = std::env::var("CARGO_BIN_EXE_drl-rs")
    .expect("cargo must provide the shipped drl-rs binary path");
  let mut child = Command::new(binary)
    .arg("--mcp")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("spawn MCP process");
  let mut stdin = child.stdin.take().expect("MCP stdin");
  stdin
    .write_all(request.as_bytes())
    .expect("write bounded test request");
  drop(stdin);
  let output = child
    .wait_with_output()
    .expect("wait for MCP process to exit");

  assert!(
    output.status.success(),
    "MCP process aborted: status={:?}, stderr={}",
    output.status,
    String::from_utf8_lossy(&output.stderr)
  );
  let responses: Vec<_> = String::from_utf8(output.stdout)
    .expect("MCP responses are UTF-8")
    .lines()
    .map(JsonValue::parse)
    .collect::<Result<_, _>>()
    .expect("MCP responses are JSON");
  assert_eq!(responses.len(), 2);
  assert_eq!(
    responses[0]
      .get("error")
      .and_then(|error| error.get("code"))
      .and_then(JsonValue::as_i64),
    Some(error_codes::PARSE_ERROR as i64)
  );
  assert!(responses[1].get("result").is_some());
}
