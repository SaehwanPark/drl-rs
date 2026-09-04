//! Snapshot codec, semantic-identity rejection, and quarantine contracts.
//!
//! Named `snapshots` so its paths keep referring to the production `persistence` module.

use super::*;

#[test]
fn snapshot_round_trip_replays_fixed_session_deterministically() {
  let mut session = BrowserSession::new().expect("fixed session");
  for command in [
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Pickup,
  ] {
    session.submit(command).expect("legal command");
  }
  let expected_observation = session.observation();
  let expected_replay = session.replay_log();
  let token = session.snapshot_token().expect("snapshot encoding");
  assert_eq!(
    token,
    "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:2:drl-rs-ruleset-v1:4:mr;mr;mr;p"
  );

  let mut restored = BrowserSession::new().expect("fixed session");
  restored.restore_snapshot(&token).expect("snapshot restore");
  assert_eq!(restored.observation(), expected_observation);
  assert_eq!(restored.replay_log(), expected_replay);
  assert_eq!(restored.snapshot_token().expect("re-encode"), token);
}

#[test]
fn v3_snapshot_round_trips_empty_history() {
  let session = BrowserSession::new().expect("fixed session");
  let token = session.snapshot_token().expect("snapshot encoding");
  assert_eq!(
    token,
    "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:2:drl-rs-ruleset-v1:0:"
  );
  let decoded = persistence::decode_snapshot_with_format(&token).expect("snapshot decoding");
  assert_eq!(decoded.format, persistence::SnapshotFormat::V3);
  assert!(decoded.commands.is_empty());
}

#[test]
fn v1_and_v2_snapshots_are_rejected_as_unbound() {
  let legacy = "DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:mr;mr;mr;p";
  let mut restored = BrowserSession::new().expect("fixed session");
  assert_eq!(
    restored.restore_snapshot(legacy),
    Err(SnapshotError::UnboundSemantics("1".to_string()))
  );
  assert_eq!(
    restored.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:4:mr;mr;mr;p"),
    Err(SnapshotError::UnboundSemantics("2".to_string()))
  );
}

#[test]
fn snapshot_rejects_corruption_and_unknown_versions() {
  let mut session = BrowserSession::new().expect("fixed session");
  assert_eq!(
    session.restore_snapshot("DRL-RUST-BROWSER-SAVE/4:fixed-m4-v1:143:1:2:drl-rs-ruleset-v1:0:"),
    Err(SnapshotError::UnsupportedVersion("4".to_string()))
  );
  assert_eq!(
    session.restore_snapshot("DRL-RUST-BROWSER-SAVE/3:other:143:1:2:drl-rs-ruleset-v1:1:w"),
    Err(SnapshotError::UnsupportedContent("other".to_string()))
  );
  assert_eq!(
    session.restore_snapshot("not-a-snapshot"),
    Err(SnapshotError::Malformed)
  );
  assert_eq!(
    session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:w;;p"),
    Err(SnapshotError::Malformed)
  );
  assert_eq!(
    session.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:2:w"),
    Err(SnapshotError::Malformed)
  );
  assert_eq!(
    session.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:nope:w"),
    Err(SnapshotError::Malformed)
  );
  assert_eq!(
    session.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:1:"),
    Err(SnapshotError::Malformed)
  );
  let oversized = format!("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:{}", "w;".repeat(8193));
  assert_eq!(
    session.restore_snapshot(&oversized),
    Err(SnapshotError::TooLarge)
  );
}

#[test]
fn snapshot_rejects_each_incompatible_identity_before_restore() {
  let cases = [
    (
      "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:127:1:2:drl-rs-ruleset-v1:0:",
      SnapshotError::UnsupportedGameplaySemantics {
        found: 127,
        expected: drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION,
      },
    ),
    (
      "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:0:2:drl-rs-ruleset-v1:0:",
      SnapshotError::UnsupportedRngSamplingSemantics {
        found: 0,
        expected: drl_protocol::CURRENT_RNG_SAMPLING_SEMANTICS_VERSION,
      },
    ),
    (
      "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:1:drl-rs-ruleset-v1:0:",
      SnapshotError::UnsupportedGeneratorSemantics {
        found: 1,
        expected: drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION,
      },
    ),
    (
      "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:2:legacy-ruleset:0:",
      SnapshotError::UnsupportedRuleset {
        found: "legacy-ruleset".to_string(),
        expected: drl_protocol::CURRENT_RULESET_ID.to_string(),
      },
    ),
  ];
  for (token, expected_error) in cases {
    let mut session = BrowserSession::new().expect("fixed session");
    session
      .submit(Command::Move(Direction::East))
      .expect("legal command");
    let before = session.clone();
    assert_eq!(session.restore_snapshot(token), Err(expected_error));
    assert_eq!(session, before);
  }
}

#[test]
fn snapshot_rejects_noncanonical_v3_numbers_and_count_mismatches() {
  let mut session = BrowserSession::new().expect("fixed session");
  for token in [
    "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:0143:1:2:drl-rs-ruleset-v1:0:",
    "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:+143:1:2:drl-rs-ruleset-v1:0:",
    "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:2:drl-rs-ruleset-v1:01:w",
    "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:2:drl-rs-ruleset-v1:2:w",
    "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:2:drl-rs-ruleset-v1:1:é",
  ] {
    assert_eq!(
      session.restore_snapshot(token),
      Err(SnapshotError::Malformed)
    );
  }
}

#[test]
fn rejected_snapshot_keeps_the_active_session_unchanged() {
  let mut session = BrowserSession::new().expect("fixed session");
  session
    .submit(Command::Move(Direction::East))
    .expect("legal command");
  let before = session.clone();

  assert_eq!(
    session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:w;;p"),
    Err(SnapshotError::Malformed)
  );
  assert_eq!(session, before);
}

#[test]
fn late_replay_failure_keeps_the_active_session_unchanged() {
  let mut session = BrowserSession::new().expect("fixed session");
  session
    .submit(Command::Move(Direction::East))
    .expect("legal command");
  let before = session.clone();
  let token = "DRL-RUST-BROWSER-SAVE/3:fixed-m4-v1:143:1:2:drl-rs-ruleset-v1:2:mr;x";

  assert_eq!(
    session.restore_snapshot(token),
    Err(SnapshotError::CommandRejected(
      "no stairs present at current position (5, 8)".to_string(),
    ))
  );
  assert_eq!(session, before);
}

#[test]
fn snapshot_codec_covers_every_command_variant() {
  let commands = [
    Command::Move(Direction::None),
    Command::Move(Direction::NorthWest),
    Command::AttackMelee(Direction::SouthEast),
    Command::AttackRanged(Position::new(-3, 8)),
    Command::AttackRangedAimed(Position::new(-7, 11)),
    Command::AttackRangedChainfire(Position::new(3, 4)),
    Command::Wait,
    Command::Pickup,
    Command::Drop(ItemId::new(4)),
    Command::Equip(ItemId::new(5)),
    Command::Unequip(drl_protocol::EquipmentSlot::Weapon),
    Command::Unequip(drl_protocol::EquipmentSlot::Armor),
    Command::Use(ItemId::new(6)),
    Command::Invoke(ItemId::new(7)),
    Command::AltReload {
      item_id: ItemId::new(8),
      confirmed: true,
    },
    Command::Reload,
    Command::Descend,
  ];
  let token = persistence::encode_snapshot(&commands).expect("codec encoding");
  assert_eq!(
    persistence::decode_snapshot_with_format(&token)
      .expect("codec decoding")
      .commands,
    commands
  );
}
