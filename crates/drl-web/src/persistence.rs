//! Versioned fixed-session save tokens for the browser boundary.

use drl_protocol::{
  CURRENT_FIXED_CONTENT_ID, CURRENT_GAMEPLAY_SEMANTICS_VERSION,
  CURRENT_GENERATOR_SEMANTICS_VERSION, CURRENT_RNG_SAMPLING_SEMANTICS_VERSION, CURRENT_RULESET_ID,
  Command, Direction, EquipmentSlot, ItemId, Position,
};

const SNAPSHOT_PREFIX: &str = "DRL-RUST-BROWSER-SAVE/";
const SNAPSHOT_V1: &str = "1";
const SNAPSHOT_V2: &str = "2";
const SNAPSHOT_V3: &str = "3";
const SNAPSHOT_MAX_BYTES: usize = 16 * 1024;
const SNAPSHOT_MAX_COMMANDS: usize = 4096;
#[cfg(any(target_arch = "wasm32", test))]
const QUARANTINE_PREFIX: &str = "DRL-RUST-BROWSER-REJECTED/1:";

/// Errors returned when a browser-session snapshot cannot be decoded or replayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
  /// The token uses a snapshot version this build does not understand.
  UnsupportedVersion(String),
  /// The token targets a different fixed-session content profile.
  UnsupportedContent(String),
  /// The token was written without the semantic identities required for safe restore.
  UnboundSemantics(String),
  /// The token targets a different gameplay semantics version.
  UnsupportedGameplaySemantics { found: u32, expected: u32 },
  /// The token targets a different bounded-RNG sampling semantics version.
  UnsupportedRngSamplingSemantics { found: u32, expected: u32 },
  /// The token targets a different procedural-generator semantics version.
  UnsupportedGeneratorSemantics { found: u32, expected: u32 },
  /// The token targets a different ruleset/content policy.
  UnsupportedRuleset { found: String, expected: String },
  /// The token has an invalid prefix, command, number, or delimiter.
  Malformed,
  /// The token exceeds the bounded browser-session save policy.
  TooLarge,
  /// A decoded command is no longer legal for the fixed session.
  CommandRejected(String),
  /// The fixed session could not be initialized for replay.
  Initialization(String),
}

impl std::fmt::Display for SnapshotError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UnsupportedVersion(version) => {
        write!(formatter, "unsupported snapshot version {version}")
      }
      Self::UnsupportedContent(found) => {
        write!(
          formatter,
          "unsupported snapshot content {found}; expected {CURRENT_FIXED_CONTENT_ID}"
        )
      }
      Self::UnboundSemantics(version) => {
        write!(
          formatter,
          "snapshot version {version} has no semantic identity; save it again with a compatible build"
        )
      }
      Self::UnsupportedGameplaySemantics { found, expected } => {
        write!(
          formatter,
          "unsupported snapshot gameplay semantics {found}; expected {expected}"
        )
      }
      Self::UnsupportedRngSamplingSemantics { found, expected } => {
        write!(
          formatter,
          "unsupported snapshot RNG sampling semantics {found}; expected {expected}"
        )
      }
      Self::UnsupportedGeneratorSemantics { found, expected } => {
        write!(
          formatter,
          "unsupported snapshot generator semantics {found}; expected {expected}"
        )
      }
      Self::UnsupportedRuleset { found, expected } => {
        write!(
          formatter,
          "unsupported snapshot ruleset {found}; expected {expected}"
        )
      }
      Self::Malformed => write!(formatter, "malformed browser-session snapshot"),
      Self::TooLarge => write!(formatter, "browser-session snapshot is too large"),
      Self::CommandRejected(error) => write!(formatter, "snapshot command rejected: {error}"),
      Self::Initialization(error) => write!(formatter, "snapshot initialization failed: {error}"),
    }
  }
}

impl std::error::Error for SnapshotError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SnapshotFormat {
  V1,
  V2,
  V3,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DecodedSnapshot {
  pub(crate) commands: Vec<Command>,
  pub(crate) format: SnapshotFormat,
}

fn direction_code(direction: Direction) -> Option<char> {
  Some(match direction {
    Direction::None => '0',
    Direction::North => 'n',
    Direction::NorthEast => 'e',
    Direction::East => 'r',
    Direction::SouthEast => 'd',
    Direction::South => 's',
    Direction::SouthWest => 'q',
    Direction::West => 'l',
    Direction::NorthWest => 'z',
  })
}

fn parse_direction(code: &str) -> Result<Direction, SnapshotError> {
  match code {
    "0" => Ok(Direction::None),
    "n" => Ok(Direction::North),
    "e" => Ok(Direction::NorthEast),
    "r" => Ok(Direction::East),
    "d" => Ok(Direction::SouthEast),
    "s" => Ok(Direction::South),
    "q" => Ok(Direction::SouthWest),
    "l" => Ok(Direction::West),
    "z" => Ok(Direction::NorthWest),
    _ => Err(SnapshotError::Malformed),
  }
}

fn parse_item_id(value: &str) -> Result<ItemId, SnapshotError> {
  let id = value.parse::<u64>().map_err(|_| SnapshotError::Malformed)?;
  (id > 0)
    .then_some(ItemId::new(id))
    .ok_or(SnapshotError::Malformed)
}

fn parse_position(value: &str) -> Result<Position, SnapshotError> {
  let mut parts = value.split(',');
  let x = parts
    .next()
    .ok_or(SnapshotError::Malformed)?
    .parse::<i32>()
    .map_err(|_| SnapshotError::Malformed)?;
  let y = parts
    .next()
    .ok_or(SnapshotError::Malformed)?
    .parse::<i32>()
    .map_err(|_| SnapshotError::Malformed)?;
  if parts.next().is_some() {
    return Err(SnapshotError::Malformed);
  }
  Ok(Position::new(x, y))
}

fn encode_command(command: Command) -> Result<String, SnapshotError> {
  Ok(match command {
    Command::Move(direction) => format!(
      "m{}",
      direction_code(direction).ok_or(SnapshotError::Malformed)?
    ),
    Command::AttackMelee(direction) => format!(
      "a{}",
      direction_code(direction).ok_or(SnapshotError::Malformed)?
    ),
    Command::AttackRanged(position) => format!("r{},{}", position.x, position.y),
    Command::AttackRangedAimed(position) => format!("t{},{}", position.x, position.y),
    Command::AttackRangedChainfire(position) => format!("h{},{}", position.x, position.y),
    Command::Wait => "w".to_string(),
    Command::Pickup => "p".to_string(),
    Command::Drop(id) => format!("d{}", id.as_u64()),
    Command::Equip(id) => format!("e{}", id.as_u64()),
    Command::Unequip(slot) => format!(
      "u{}",
      if slot == EquipmentSlot::Weapon {
        "w"
      } else {
        "a"
      }
    ),
    Command::Use(id) => format!("c{}", id.as_u64()),
    Command::Invoke(id) => format!("v{}", id.as_u64()),
    Command::AltReload { item_id, confirmed } => {
      format!("b{}:{}", item_id.as_u64(), u8::from(confirmed))
    }
    Command::Reload => "l".to_string(),
    Command::Descend => "x".to_string(),
  })
}

fn decode_command(token: &str) -> Result<Command, SnapshotError> {
  let mut characters = token.chars();
  let opcode = characters.next().ok_or(SnapshotError::Malformed)?;
  if !opcode.is_ascii() {
    return Err(SnapshotError::Malformed);
  }
  let rest = characters.as_str();
  match opcode {
    'm' => Ok(Command::Move(parse_direction(rest)?)),
    'a' => Ok(Command::AttackMelee(parse_direction(rest)?)),
    'r' => Ok(Command::AttackRanged(parse_position(rest)?)),
    't' => Ok(Command::AttackRangedAimed(parse_position(rest)?)),
    'h' => Ok(Command::AttackRangedChainfire(parse_position(rest)?)),
    'w' if rest.is_empty() => Ok(Command::Wait),
    'p' if rest.is_empty() => Ok(Command::Pickup),
    'd' => Ok(Command::Drop(parse_item_id(rest)?)),
    'e' => Ok(Command::Equip(parse_item_id(rest)?)),
    'u' if rest == "w" => Ok(Command::Unequip(EquipmentSlot::Weapon)),
    'u' if rest == "a" => Ok(Command::Unequip(EquipmentSlot::Armor)),
    'c' => Ok(Command::Use(parse_item_id(rest)?)),
    'v' => Ok(Command::Invoke(parse_item_id(rest)?)),
    'b' => {
      let (item_id, confirmed) = rest.split_once(':').ok_or(SnapshotError::Malformed)?;
      let confirmed = match confirmed {
        "0" => false,
        "1" => true,
        _ => return Err(SnapshotError::Malformed),
      };
      Ok(Command::AltReload {
        item_id: parse_item_id(item_id)?,
        confirmed,
      })
    }
    'l' if rest.is_empty() => Ok(Command::Reload),
    'x' if rest.is_empty() => Ok(Command::Descend),
    _ => Err(SnapshotError::Malformed),
  }
}

pub(crate) fn encode_snapshot(commands: &[Command]) -> Result<String, SnapshotError> {
  if commands.len() > SNAPSHOT_MAX_COMMANDS {
    return Err(SnapshotError::TooLarge);
  }
  let payload = encode_payload(commands)?;
  let mut token = format!(
    "{SNAPSHOT_PREFIX}{SNAPSHOT_V3}:{CURRENT_FIXED_CONTENT_ID}:{CURRENT_GAMEPLAY_SEMANTICS_VERSION}:{CURRENT_RNG_SAMPLING_SEMANTICS_VERSION}:{CURRENT_GENERATOR_SEMANTICS_VERSION}:{CURRENT_RULESET_ID}:{}:",
    commands.len()
  );
  token.push_str(&payload);
  if token.len() > SNAPSHOT_MAX_BYTES {
    return Err(SnapshotError::TooLarge);
  }
  Ok(token)
}

fn encode_payload(commands: &[Command]) -> Result<String, SnapshotError> {
  let mut payload = String::new();
  for (index, command) in commands.iter().copied().enumerate() {
    if index > 0 {
      payload.push(';');
    }
    payload.push_str(&encode_command(command)?);
  }
  Ok(payload)
}

fn decode_payload(payload: &str) -> Result<Vec<Command>, SnapshotError> {
  if payload.is_empty() {
    return Ok(Vec::new());
  }
  let commands = payload.split(';');
  if commands.clone().count() > SNAPSHOT_MAX_COMMANDS {
    return Err(SnapshotError::TooLarge);
  }
  commands.map(decode_command).collect()
}

fn parse_canonical_u32(value: &str) -> Result<u32, SnapshotError> {
  if value.is_empty()
    || (value.len() > 1 && value.starts_with('0'))
    || !value.bytes().all(|byte| byte.is_ascii_digit())
  {
    return Err(SnapshotError::Malformed);
  }
  value.parse::<u32>().map_err(|_| SnapshotError::Malformed)
}

fn parse_canonical_count(value: &str) -> Result<usize, SnapshotError> {
  if value.is_empty()
    || (value.len() > 1 && value.starts_with('0'))
    || !value.bytes().all(|byte| byte.is_ascii_digit())
  {
    return Err(SnapshotError::Malformed);
  }
  let count = value
    .parse::<usize>()
    .map_err(|_| SnapshotError::TooLarge)?;
  if count > SNAPSHOT_MAX_COMMANDS {
    return Err(SnapshotError::TooLarge);
  }
  Ok(count)
}

fn require_content(content: &str) -> Result<(), SnapshotError> {
  if content == CURRENT_FIXED_CONTENT_ID {
    Ok(())
  } else {
    Err(SnapshotError::UnsupportedContent(content.to_string()))
  }
}

fn decode_v1(versioned: &str) -> Result<DecodedSnapshot, SnapshotError> {
  let mut parts = versioned.splitn(2, ':');
  let Some(content) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  require_content(content)?;
  let Some(payload) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  Ok(DecodedSnapshot {
    commands: decode_payload(payload)?,
    format: SnapshotFormat::V1,
  })
}

fn decode_v2(versioned: &str) -> Result<DecodedSnapshot, SnapshotError> {
  let mut parts = versioned.splitn(3, ':');
  let Some(content) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  require_content(content)?;
  let Some(count_text) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  let count = parse_canonical_count(count_text)?;
  let Some(payload) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  let commands = decode_payload(payload)?;
  if commands.len() != count {
    return Err(SnapshotError::Malformed);
  }
  Ok(DecodedSnapshot {
    commands,
    format: SnapshotFormat::V2,
  })
}

fn decode_v3(versioned: &str) -> Result<DecodedSnapshot, SnapshotError> {
  let mut parts = versioned.splitn(7, ':');
  let content = parts.next().ok_or(SnapshotError::Malformed)?;
  require_content(content)?;

  let gameplay = parse_canonical_u32(parts.next().ok_or(SnapshotError::Malformed)?)?;
  if gameplay != CURRENT_GAMEPLAY_SEMANTICS_VERSION {
    return Err(SnapshotError::UnsupportedGameplaySemantics {
      found: gameplay,
      expected: CURRENT_GAMEPLAY_SEMANTICS_VERSION,
    });
  }

  let rng_sampling = parse_canonical_u32(parts.next().ok_or(SnapshotError::Malformed)?)?;
  if rng_sampling != CURRENT_RNG_SAMPLING_SEMANTICS_VERSION {
    return Err(SnapshotError::UnsupportedRngSamplingSemantics {
      found: rng_sampling,
      expected: CURRENT_RNG_SAMPLING_SEMANTICS_VERSION,
    });
  }

  let generator = parse_canonical_u32(parts.next().ok_or(SnapshotError::Malformed)?)?;
  if generator != CURRENT_GENERATOR_SEMANTICS_VERSION {
    return Err(SnapshotError::UnsupportedGeneratorSemantics {
      found: generator,
      expected: CURRENT_GENERATOR_SEMANTICS_VERSION,
    });
  }

  let ruleset = parts.next().ok_or(SnapshotError::Malformed)?;
  if ruleset != CURRENT_RULESET_ID {
    return Err(SnapshotError::UnsupportedRuleset {
      found: ruleset.to_string(),
      expected: CURRENT_RULESET_ID.to_string(),
    });
  }

  let count = parse_canonical_count(parts.next().ok_or(SnapshotError::Malformed)?)?;
  let payload = parts.next().ok_or(SnapshotError::Malformed)?;
  let commands = decode_payload(payload)?;
  if commands.len() != count {
    return Err(SnapshotError::Malformed);
  }
  Ok(DecodedSnapshot {
    commands,
    format: SnapshotFormat::V3,
  })
}

pub(crate) fn decode_snapshot_with_format(token: &str) -> Result<DecodedSnapshot, SnapshotError> {
  if token.len() > SNAPSHOT_MAX_BYTES {
    return Err(SnapshotError::TooLarge);
  }
  let Some(versioned) = token.strip_prefix(SNAPSHOT_PREFIX) else {
    return Err(SnapshotError::Malformed);
  };
  let mut parts = versioned.splitn(2, ':');
  let Some(version) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  let Some(remainder) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  match version {
    SNAPSHOT_V1 => {
      decode_v1(remainder)?;
      Err(SnapshotError::UnboundSemantics(version.to_string()))
    }
    SNAPSHOT_V2 => {
      decode_v2(remainder)?;
      Err(SnapshotError::UnboundSemantics(version.to_string()))
    }
    SNAPSHOT_V3 => decode_v3(remainder),
    _ => Err(SnapshotError::UnsupportedVersion(version.to_string())),
  }
}

/// Builds one bounded diagnostic record for a rejected browser save.
///
/// The record is never accepted by [`decode_snapshot_with_format`]. Keeping the original
/// value when it fits gives a future explicit migration a chance to inspect
/// it, while oversized values are represented by their size rather than
/// allowing localStorage recovery data to grow without bound.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn encode_quarantine_record(token: &str, error: &SnapshotError) -> String {
  let error_text: String = error.to_string().chars().take(256).collect();
  let header = format!(
    "{QUARANTINE_PREFIX}bytes={};error={error_text}\n",
    token.len()
  );
  if header.len().saturating_add(token.len()) <= SNAPSHOT_MAX_BYTES {
    return format!("{header}{token}");
  }
  format!("{header}<token omitted: exceeds {SNAPSHOT_MAX_BYTES} bytes>")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn quarantine_record_preserves_small_rejected_tokens() {
    let record = encode_quarantine_record("not-a-snapshot", &SnapshotError::Malformed);
    assert!(record.starts_with("DRL-RUST-BROWSER-REJECTED/1:bytes=14;error="));
    assert!(record.ends_with("\nnot-a-snapshot"));
    assert!(record.len() <= SNAPSHOT_MAX_BYTES);
  }

  #[test]
  fn quarantine_record_bounds_oversized_tokens() {
    let token = "x".repeat(SNAPSHOT_MAX_BYTES * 2);
    let record = encode_quarantine_record(&token, &SnapshotError::TooLarge);
    assert!(record.contains("token omitted"));
    assert!(record.len() <= SNAPSHOT_MAX_BYTES);
  }

  #[test]
  fn snapshot_roundtrips_trigun_confirmation_command() {
    let commands = [Command::AltReload {
      item_id: ItemId::new(43),
      confirmed: true,
    }];
    let token = encode_snapshot(&commands).unwrap();
    let decoded = decode_snapshot_with_format(&token).unwrap();
    assert_eq!(decoded.commands, commands);
  }
}
