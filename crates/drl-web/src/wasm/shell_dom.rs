//! DOM status, diagnostic, and accessibility projections for the browser shell.

use super::*;

pub(crate) fn update_dom(document: &web_sys::Document, observation: &PlayerObservation) {
  if let Some(hp) = document.get_element_by_id("game-hp") {
    let value = observation.player_hp.map_or_else(
      || "HP: —".to_string(),
      |hp| format!("HP: {}/{}", hp.current, hp.max),
    );
    hp.set_text_content(Some(&value));
  }
  if let Some(turn) = document.get_element_by_id("game-turn") {
    turn.set_text_content(Some(&format!("Turn: {}", observation.turn.count)));
  }
  if let Some(weapon) = document.get_element_by_id("game-weapon") {
    let value = observation.equipped_weapon.as_ref().map_or_else(
      || "Weapon: —".to_string(),
      |item| format!("Weapon: {}", item.name),
    );
    weapon.set_text_content(Some(&value));
  }
  if let Some(targets) = document.get_element_by_id("target-indicator") {
    let count = observation
      .visible_actors
      .iter()
      .filter(|actor| !actor.is_player)
      .count();
    let value = if count == 0 {
      "Targets: none visible".to_string()
    } else {
      format!("Targets: {count} visible (F selects nearest)")
    };
    targets.set_text_content(Some(&value));
  }
  if let Some(minimap) = document.get_element_by_id("minimap") {
    let state = MinimapState::from_observation(observation);
    minimap.set_text_content(Some(&minimap_markup(&state)));
  }
  if let Some(inventory) = document.get_element_by_id("inventory") {
    inventory.set_inner_html(&inventory_markup(&observation.inventory));
  }
}

pub(crate) fn update_target_status(document: &web_sys::Document, message: &str) {
  if let Some(targets) = document.get_element_by_id("target-indicator") {
    targets.set_text_content(Some(message));
  }
}

pub(crate) fn set_status(document: &web_sys::Document, message: &str) {
  if let Some(status) = document.get_element_by_id("game-status") {
    status.set_text_content(Some(message));
  }
}

pub(crate) fn clear_persistence_diagnostic(document: &web_sys::Document) {
  let Some(panel) = document.get_element_by_id("game-diagnostics") else {
    return;
  };
  let persistence_diagnostic_active = document
    .get_element_by_id("diagnostics-title")
    .and_then(|node| node.text_content())
    .as_deref()
    == Some("Saved session incompatible");
  if !persistence_diagnostic_active {
    return;
  }
  let _ = panel.set_attribute("hidden", "");
  let _ = panel.remove_attribute("data-diagnostic-source");
  if let Some(title_node) = document.get_element_by_id("diagnostics-title") {
    title_node.set_text_content(Some("Browser support diagnostic"));
  }
  if let Some(detail_node) = document.get_element_by_id("diagnostics-detail") {
    detail_node.set_text_content(Some(""));
  }
  if let Some(action_node) = document.get_element_by_id("diagnostics-action") {
    action_node.set_text_content(Some(""));
  }
}

pub(crate) fn set_diagnostic(
  document: &web_sys::Document,
  title: &str,
  detail: &str,
  action: &str,
) {
  let persistence_diagnostic_active = document
    .get_element_by_id("diagnostics-title")
    .and_then(|node| node.text_content())
    .as_deref()
    == Some("Saved session incompatible");
  if persistence_diagnostic_active && title != "Saved session incompatible" {
    return;
  }
  if let Some(panel) = document.get_element_by_id("game-diagnostics") {
    let _ = panel.remove_attribute("hidden");
    let source = if title == "Saved session incompatible" {
      "persistence"
    } else {
      "general"
    };
    let _ = panel.set_attribute("data-diagnostic-source", source);
  }
  if let Some(title_node) = document.get_element_by_id("diagnostics-title") {
    title_node.set_text_content(Some(title));
  }
  if let Some(detail_node) = document.get_element_by_id("diagnostics-detail") {
    detail_node.set_text_content(Some(detail));
  }
  if let Some(action_node) = document.get_element_by_id("diagnostics-action") {
    action_node.set_text_content(Some(action));
  }
  if let Some(panel) = document
    .get_element_by_id("game-diagnostics")
    .and_then(|panel| panel.dyn_into::<HtmlElement>().ok())
  {
    let _ = panel.focus();
  }
}
