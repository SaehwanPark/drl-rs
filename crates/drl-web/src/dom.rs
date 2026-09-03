//! Accessible DOM projections of fair observations. These helpers build
//! escaped HTML/aria text; they never read hidden core state.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use drl_protocol::ItemView;
use drl_render::{MinimapMarker, MinimapState};

/// Escapes user-visible item names before they cross the HTML string boundary.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn escape_html(value: &str) -> String {
  let mut escaped = String::with_capacity(value.len());
  for character in value.chars() {
    match character {
      '&' => escaped.push_str("&amp;"),
      '<' => escaped.push_str("&lt;"),
      '>' => escaped.push_str("&gt;"),
      '"' => escaped.push_str("&quot;"),
      '\'' => escaped.push_str("&#39;"),
      _ => escaped.push(character),
    }
  }
  escaped
}

/// Builds item-qualified inventory controls for the browser's semantic DOM.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn inventory_markup(items: &[ItemView]) -> String {
  use std::fmt::Write;

  let mut controls = String::new();
  for item in items {
    let name = escape_html(&item.name);
    let item_id = item.id.as_u64();
    write!(
      controls,
      "<div id=\"inventory-item-{item_id}\" role=\"group\" aria-label=\"Inventory item: {name}\"><span>{name}</span><button type=\"button\" data-action=\"equip\" data-item-id=\"{item_id}\" aria-label=\"Equip {name}\">Equip</button><button type=\"button\" data-action=\"use\" data-item-id=\"{item_id}\" aria-label=\"Use {name}\">Use</button><button type=\"button\" data-action=\"drop\" data-item-id=\"{item_id}\" aria-label=\"Drop {name}\">Drop</button></div>"
    )
    .expect("writing inventory markup to a String cannot fail");
  }
  controls
}

pub(crate) const MAX_MINIMAP_CELLS: u64 = 4096;

/// Renders the fair minimap projection as a bounded, accessible text grid.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn minimap_markup(state: &MinimapState) -> String {
  let cell_count = u64::from(state.map_width) * u64::from(state.map_height);
  if state.map_width == 0 || state.map_height == 0 || cell_count > MAX_MINIMAP_CELLS {
    return "Minimap unavailable.".to_string();
  }

  let width = state.map_width as usize;
  let height = state.map_height as usize;
  let mut glyphs = vec![' '; cell_count as usize];
  for cell in &state.cells {
    let Some(x) = usize::try_from(cell.position.x).ok() else {
      continue;
    };
    let Some(y) = usize::try_from(cell.position.y).ok() else {
      continue;
    };
    if x >= width || y >= height {
      continue;
    }
    let glyph = match cell.marker {
      Some(MinimapMarker::Player) => '@',
      Some(MinimapMarker::VisibleActor) => 'a',
      None => match cell.tile_kind {
        drl_protocol::TileKind::Floor => '.',
        drl_protocol::TileKind::Wall => '#',
        drl_protocol::TileKind::DoorClosed => '+',
        drl_protocol::TileKind::DoorOpen => '/',
        drl_protocol::TileKind::StairsDown => '>',
        drl_protocol::TileKind::Lava => '=',
        drl_protocol::TileKind::Acid => 'a',
        drl_protocol::TileKind::Water => '~',
        drl_protocol::TileKind::Mud => 'u',
      },
    };
    glyphs[y * width + x] = glyph;
  }

  let mut markup = String::with_capacity((width + 1) * height);
  for row in glyphs.chunks(width) {
    if !markup.is_empty() {
      markup.push('\n');
    }
    for glyph in row {
      markup.push(*glyph);
    }
  }
  markup
}
