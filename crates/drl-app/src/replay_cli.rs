//! Native replay-file verification command.

use drl_core::ReplayEngine;
use drl_mcp::json::JsonValue;
use std::fs;
use std::io::{self, Read};

const REPLAY_USAGE: &str = "usage: drl-rs replay verify [path|-]";

/// Runs the replay subcommand using process stdin for the `-` source.
pub(crate) fn run_replay_command(args: &[String]) -> Result<String, String> {
  let mut stdin = io::stdin().lock();
  run_replay_command_with_reader(args, &mut stdin)
}

/// Runs the replay subcommand with an injectable stdin reader for deterministic tests.
fn run_replay_command_with_reader<R: Read>(
  args: &[String],
  stdin: &mut R,
) -> Result<String, String> {
  let path = parse_verify_args(args)?;
  let input = if path == "-" {
    let mut input = String::new();
    stdin
      .read_to_string(&mut input)
      .map_err(|error| format!("replay verification failed: unable to read stdin: {error}"))?;
    input
  } else {
    fs::read_to_string(path).map_err(|error| {
      format!(
        "replay verification failed: unable to read {:?}: {error}",
        path
      )
    })?
  };

  verify_replay_json(&input)
}

fn parse_verify_args(args: &[String]) -> Result<&str, String> {
  match args {
    [command, path] if command == "verify" && !path.is_empty() => Ok(path),
    _ => Err(REPLAY_USAGE.to_string()),
  }
}

fn verify_replay_json(input: &str) -> Result<String, String> {
  let value = JsonValue::parse(input)
    .map_err(|error| format!("replay verification failed: malformed JSON: {error}"))?;
  let replay = drl_mcp::replay_json::from_json_value(&value)
    .map_err(|error| format!("replay verification failed: invalid replay: {error}"))?;
  let deterministic = ReplayEngine::verify_determinism(&replay)
    .map_err(|error| format!("replay verification failed: replay execution error: {error}"))?;
  if deterministic {
    Ok("replay verification passed: deterministic\n".to_string())
  } else {
    Err("replay verification failed: nondeterministic result".to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_mcp::replay_json::to_json_value;
  use drl_protocol::{Position, ReplayLog};
  use std::io::Cursor;
  use std::path::PathBuf;
  use std::time::{SystemTime, UNIX_EPOCH};

  fn valid_replay_json() -> String {
    let replay = ReplayLog::new(123, 10, 10, Position::new(1, 1));
    to_json_value(&replay).to_compact_string()
  }

  fn args(path: &str) -> Vec<String> {
    vec!["verify".to_string(), path.to_string()]
  }

  fn temporary_path(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("clock before Unix epoch")
      .as_nanos();
    std::env::temp_dir().join(format!("drl-rs-replay-cli-{label}-{nonce}.json"))
  }

  #[test]
  fn verifies_valid_replay_from_file() {
    let path = temporary_path("valid");
    fs::write(&path, valid_replay_json()).expect("write replay fixture");

    let result =
      run_replay_command_with_reader(&args(path.to_str().unwrap()), &mut Cursor::new([]))
        .expect("valid replay should verify");

    assert_eq!(result, "replay verification passed: deterministic\n");
    fs::remove_file(path).expect("remove replay fixture");
  }

  #[test]
  fn stdin_and_file_sources_have_identical_output() {
    let input = valid_replay_json();
    let path = temporary_path("parity");
    fs::write(&path, &input).expect("write replay fixture");

    let file_output =
      run_replay_command_with_reader(&args(path.to_str().unwrap()), &mut Cursor::new([]))
        .expect("file replay should verify");
    let stdin_output = run_replay_command_with_reader(&args("-"), &mut Cursor::new(input))
      .expect("stdin replay should verify");

    assert_eq!(file_output, stdin_output);
    fs::remove_file(path).expect("remove replay fixture");
  }

  #[test]
  fn rejects_malformed_json_with_stable_diagnostic() {
    let error = run_replay_command_with_reader(&args("-"), &mut Cursor::new("{"))
      .expect_err("malformed JSON must fail");

    assert!(error.starts_with("replay verification failed: malformed JSON:"));
  }

  #[test]
  fn rejects_unsafe_dimensions_and_oversized_commands() {
    let mut dimensions = JsonValue::parse(&valid_replay_json()).expect("valid JSON");
    dimensions
      .as_object_mut()
      .expect("replay object")
      .insert("width".to_string(), JsonValue::from(1_u32));
    let error = verify_replay_json(&dimensions.to_compact_string()).expect_err("unsafe width");
    assert!(error.contains("replay dimensions must be within 3..=512"));

    let mut oversized = JsonValue::parse(&valid_replay_json()).expect("valid JSON");
    oversized.as_object_mut().expect("replay object").insert(
      "commands".to_string(),
      JsonValue::Array((0..100_001).map(|_| JsonValue::Null).collect()),
    );
    let error = verify_replay_json(&oversized.to_compact_string())
      .expect_err("oversized command array must fail");
    assert!(error.contains("commands exceeds the bounded length 100000"));
  }

  #[test]
  fn rejects_incompatible_metadata_before_execution() {
    let mut replay = JsonValue::parse(&valid_replay_json()).expect("valid JSON");
    replay
      .as_object_mut()
      .expect("replay object")
      .get_mut("metadata")
      .expect("metadata")
      .as_object_mut()
      .expect("metadata object")
      .insert(
        "ruleset_id".to_string(),
        JsonValue::from("unsupported-ruleset"),
      );
    let error =
      verify_replay_json(&replay.to_compact_string()).expect_err("incompatible ruleset must fail");
    assert!(error.contains("unsupported replay ruleset"));
  }

  #[test]
  fn repeated_valid_verification_is_byte_identical() {
    let input = valid_replay_json();
    let first = verify_replay_json(&input).expect("first verification");
    let second = verify_replay_json(&input).expect("second verification");
    assert_eq!(first, second);
  }
}
