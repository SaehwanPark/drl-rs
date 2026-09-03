//! Keyboard and DOM control mapping into semantic `Command` values. Mapping is
//! pure and never advances the simulation.

use drl_protocol::{Command, Direction, ItemId, PlayerObservation, Position};

use crate::session::{BrowserSession, chainfire_ammo_cost};

impl BrowserSession {
  /// Maps keyboard names to semantic commands without advancing the game.
  #[must_use]
  pub fn command_for_key(key: &str, observation: &PlayerObservation) -> Option<Command> {
    let direction = match key {
      "ArrowUp" | "w" | "W" | "8" => Some(Direction::North),
      "ArrowRight" | "d" | "D" | "6" => Some(Direction::East),
      "ArrowDown" | "s" | "S" | "2" => Some(Direction::South),
      "ArrowLeft" | "a" | "A" | "4" => Some(Direction::West),
      "7" => Some(Direction::NorthWest),
      "9" => Some(Direction::NorthEast),
      "1" => Some(Direction::SouthWest),
      "3" => Some(Direction::SouthEast),
      _ => None,
    };
    if let Some(direction) = direction {
      return Some(Command::Move(direction));
    }
    match key {
      "." | "5" | "Space" => Some(Command::Wait),
      "g" | "G" => Some(Command::Pickup),
      "r" | "R" => Some(Command::Reload),
      ">" => Some(Command::Descend),
      "f" | "F" => observation
        .visible_actors
        .iter()
        .find(|actor| !actor.is_player)
        .map(|actor| Command::AttackRanged(actor.position)),
      "c" | "C" => observation
        .equipped_weapon
        .as_ref()
        .filter(|weapon| {
          chainfire_ammo_cost(weapon.archetype, weapon.chainfire_level)
            .is_some_and(|ammo_cost| weapon.clip.is_some_and(|(loaded, _)| loaded >= ammo_cost))
        })
        .and_then(|_| {
          observation
            .visible_actors
            .iter()
            .find(|actor| !actor.is_player)
            .map(|actor| Command::AttackRangedChainfire(actor.position))
        }),
      _ => None,
    }
  }

  /// Creates an explicit ranged target command for a DOM/canvas click.
  #[must_use]
  pub const fn target_command(position: Position, confirmed: bool) -> Option<Command> {
    if confirmed {
      Some(Command::AttackRanged(position))
    } else {
      None
    }
  }

  /// Maps an inventory action from a semantic item id.
  #[must_use]
  pub const fn inventory_command(action: InventoryAction, item_id: ItemId) -> Command {
    match action {
      InventoryAction::Equip => Command::Equip(item_id),
      InventoryAction::Use => Command::Use(item_id),
      InventoryAction::Drop => Command::Drop(item_id),
    }
  }
}

/// DOM inventory action supported by the first slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAction {
  Equip,
  Use,
  Drop,
}
