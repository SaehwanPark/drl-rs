//! Session transaction, rollback, and presentation-only decal contracts.

use super::*;

#[test]
fn rejected_commands_do_not_advance_the_session() {
  let mut session = BrowserSession::new().expect("fixed session");
  let before = session.observation();
  let error = session.submit(Command::Descend).unwrap_err();
  assert!(!error.is_empty());
  assert_eq!(session.observation(), before);
}

#[test]
fn rejected_command_preserves_core_state_without_outer_checkpoint() {
  let mut session = BrowserSession::new().expect("fixed session");
  session
    .submit(Command::Move(Direction::East))
    .expect("legal command");
  let game_before = session.game.clone();
  let commands_before = session.commands.clone();
  let replay_before = session.replay_log();

  let error = session.submit(Command::Descend).unwrap_err();

  assert_eq!(session.game, game_before);
  assert_eq!(session.commands, commands_before);
  assert_eq!(session.replay_log(), replay_before);
  assert_eq!(session.last_error(), Some(error.as_str()));
}

#[test]
fn late_rejection_preserves_session_without_outer_checkpoint() {
  let mut session = BrowserSession::new().expect("fixed session");
  let target_position = Position::new(5, 8);
  let target_id = session
    .game
    .world_mut()
    .spawn_monster(target_position, "Dropper", 1, 0, (1, 1))
    .expect("spawn target");
  session
    .game
    .world_mut()
    .get_actor_mut(target_id)
    .expect("target actor")
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
  session
    .game
    .world_mut()
    .map_mut()
    .set_tile(target_position, Tile::Wall);
  let game_before = session.game.clone();
  let commands_before = session.commands.clone();
  let replay_before = session.replay_log();

  let error = session
    .submit(Command::AttackMelee(Direction::East))
    .expect_err("blocked death drop must reject");

  assert_eq!(
    error,
    drl_protocol::CommandError::BlockedByTerrain(target_position).to_string()
  );
  assert_eq!(session.game, game_before);
  assert_eq!(session.commands, commands_before);
  assert_eq!(session.replay_log(), replay_before);
  assert_eq!(session.last_error(), Some(error.as_str()));
}

#[test]
fn browser_decal_requests_are_presentation_only() {
  let mut session = BrowserSession::new().expect("fixed session");
  let before = session.observation();
  session
    .try_insert_particle_decal(drl_render::ParticleDecalInsertion {
      placement: drl_render::ParticleDecalPlacement {
        cell: [1, 1],
        pixel: [32, 32],
      },
      sprite_id: 100_001,
    })
    .expect("retain presentation request");

  assert_eq!(session.observation(), before);
  assert_eq!(session.particle_decal_store().len(), 1);
  assert!(session.particle_decal_sprites().is_empty());
}
