//! Rejected command invariants for the deterministic simulation kernel.

use drl_core::{Game, Tile};
use drl_protocol::{Command, CommandError, Position};

fn assert_rejected_command_is_atomic(
  game: &mut Game,
  command: Command,
  expected_error: CommandError,
) {
  let before = game.clone();
  let error = game.step(command).expect_err("command should be rejected");

  assert_eq!(error, expected_error);
  assert_eq!(
    game, &before,
    "rejected commands must not mutate Game state"
  );
}

#[test]
fn ranged_attack_out_of_range_preserves_ammo_and_rng() {
  let mut game = Game::new(1, 20, 20, Position::new(2, 2)).unwrap();
  let monster_position = Position::new(12, 2);
  game
    .world_mut()
    .spawn_monster(monster_position, "Demon", 100, 0, (2, 4))
    .unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(monster_position),
    CommandError::TargetOutOfRange(monster_position),
  );
}

#[test]
fn ranged_attack_blocked_by_wall_preserves_ammo_and_rng() {
  let mut game = Game::new(2, 20, 20, Position::new(2, 2)).unwrap();
  let monster_position = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(monster_position, "Demon", 100, 0, (2, 4))
    .unwrap();
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(3, 2), Tile::Wall);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(monster_position),
    CommandError::LineOfSightBlocked(monster_position),
  );
}
