//! Keyboard/DOM control mapping to semantic commands.

use super::*;

#[test]
fn keyboard_mapping_covers_diagonal_numpad_and_actions() {
  let observation = BrowserSession::new().expect("fixed session").observation();
  assert_eq!(
    BrowserSession::command_for_key("7", &observation),
    Some(Command::Move(Direction::NorthWest))
  );
  assert_eq!(
    BrowserSession::command_for_key("g", &observation),
    Some(Command::Pickup)
  );
  assert_eq!(
    BrowserSession::command_for_key("r", &observation),
    Some(Command::Reload)
  );
}
