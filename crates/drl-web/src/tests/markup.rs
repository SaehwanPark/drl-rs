//! Accessible inventory and minimap markup contracts.

use super::*;

#[test]
fn inventory_markup_qualifies_actions_and_escapes_names() {
  let markup = inventory_markup(&[test_item("Pistol <&\"'")]);
  assert!(markup.contains("id=\"inventory-item-7\""));
  assert!(markup.contains("role=\"group\""));
  assert!(markup.contains("aria-label=\"Inventory item: Pistol &lt;&amp;&quot;&#39;\""));
  assert!(markup.contains("aria-label=\"Equip Pistol &lt;&amp;&quot;&#39;\""));
  assert!(markup.contains("aria-label=\"Use Pistol &lt;&amp;&quot;&#39;\""));
  assert!(markup.contains("aria-label=\"Drop Pistol &lt;&amp;&quot;&#39;\""));
  assert!(!markup.contains("Pistol <&\"'"));
}

#[test]
fn minimap_markup_renders_only_projected_cells_and_markers() {
  let markup = minimap_markup(&MinimapState {
    map_width: 4,
    map_height: 2,
    cells: vec![
      drl_render::MinimapCell {
        position: Position::new(0, 0),
        tile_kind: TileKind::Wall,
        is_visible: true,
        marker: None,
      },
      drl_render::MinimapCell {
        position: Position::new(1, 0),
        tile_kind: TileKind::Floor,
        is_visible: true,
        marker: Some(MinimapMarker::Player),
      },
      drl_render::MinimapCell {
        position: Position::new(2, 0),
        tile_kind: TileKind::Floor,
        is_visible: true,
        marker: Some(MinimapMarker::VisibleActor),
      },
      drl_render::MinimapCell {
        position: Position::new(3, 1),
        tile_kind: TileKind::StairsDown,
        is_visible: false,
        marker: None,
      },
    ],
  });

  assert_eq!(markup, "#@a \n   >");
  assert!(!markup.contains("?"));
}

#[test]
fn minimap_markup_bounds_dom_work_for_malformed_dimensions() {
  assert_eq!(
    minimap_markup(&MinimapState {
      map_width: 65,
      map_height: 65,
      cells: Vec::new(),
    }),
    "Minimap unavailable."
  );
}
