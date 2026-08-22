//! Platform-neutral presentation identifiers and atlas metadata.
//!
//! This crate deliberately contains no image decoder, filesystem, GPU, or
//! browser dependency.  A renderer chooses how to load the paths while the
//! simulation remains unaware of presentation assets.

use drl_protocol::{ItemArchetype, MonsterKind, TileKind};

/// The legacy atlas image that owns a sprite entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtlasId {
  Dguy,
  Enemies,
  EnemiesBig,
  GunsAndPickups,
  Levels,
  DoorsAndDecorations,
  Fx,
}

impl AtlasId {
  /// Relative path in the imported, license-cleared graphics bundle.
  #[must_use]
  pub const fn path(self) -> &'static str {
    match self {
      Self::Dguy => "dguy.png",
      Self::Enemies => "enemies.png",
      Self::EnemiesBig => "enemies_big.png",
      Self::GunsAndPickups => "guns_and_pickups.png",
      Self::Levels => "levels.png",
      Self::DoorsAndDecorations => "doors_and_decorations.png",
      Self::Fx => "fx.png",
    }
  }

  /// Pixel dimensions of the imported PNG backing this atlas.
  #[must_use]
  pub const fn dimensions(self) -> (u32, u32) {
    match self {
      Self::Dguy => (512, 64),
      Self::Enemies => (512, 192),
      Self::EnemiesBig => (512, 384),
      Self::GunsAndPickups => (512, 160),
      Self::Levels => (512, 1152),
      Self::DoorsAndDecorations => (512, 288),
      Self::Fx => (512, 64),
    }
  }

  /// Returns the path for a compositing layer of this atlas.
  #[must_use]
  pub const fn layer_path(self, layer: SpriteLayer) -> &'static str {
    match (self, layer) {
      (Self::Dguy, SpriteLayer::Base) => "dguy.png",
      (Self::Dguy, SpriteLayer::Emissive) => "dguy_emissive.png",
      (Self::Dguy, SpriteLayer::Mask) => "dguy_mask.png",
      (Self::Dguy, SpriteLayer::Shadow) => "dguy_shadow.png",
      (Self::Enemies, SpriteLayer::Base) => "enemies.png",
      (Self::Enemies, SpriteLayer::Emissive) => "enemies_emissive.png",
      (Self::Enemies, SpriteLayer::Mask) => "enemies.png",
      (Self::Enemies, SpriteLayer::Shadow) => "enemies_shadow.png",
      (Self::EnemiesBig, SpriteLayer::Base) => "enemies_big.png",
      (Self::EnemiesBig, SpriteLayer::Emissive) => "enemies_big_emissive.png",
      (Self::EnemiesBig, SpriteLayer::Mask) => "enemies_big.png",
      (Self::EnemiesBig, SpriteLayer::Shadow) => "enemies_big_shadow.png",
      (Self::GunsAndPickups, SpriteLayer::Base) => "guns_and_pickups.png",
      (Self::GunsAndPickups, SpriteLayer::Emissive) => "guns_and_pickups_emissive.png",
      (Self::GunsAndPickups, SpriteLayer::Mask) => "guns_and_pickups_mask.png",
      (Self::GunsAndPickups, SpriteLayer::Shadow) => "guns_and_pickups_shadow.png",
      (Self::Levels, SpriteLayer::Base) => "levels.png",
      (Self::Levels, SpriteLayer::Emissive) => "levels_emissive.png",
      (Self::Levels, SpriteLayer::Mask) => "levels_mask.png",
      (Self::Levels, SpriteLayer::Shadow) => "levels.png",
      (Self::DoorsAndDecorations, SpriteLayer::Base) => "doors_and_decorations.png",
      (Self::DoorsAndDecorations, SpriteLayer::Emissive) => "doors_and_decorations_emissive.png",
      (Self::DoorsAndDecorations, SpriteLayer::Mask) => "doors_and_decorations_mask.png",
      (Self::DoorsAndDecorations, SpriteLayer::Shadow) => "doors_and_decorations_shadow.png",
      (Self::Fx, SpriteLayer::Base) => "fx.png",
      (Self::Fx, SpriteLayer::Emissive) => "fx_emissive.png",
      (Self::Fx, SpriteLayer::Mask) => "fx_mask.png",
      (Self::Fx, SpriteLayer::Shadow) => "fx.png",
    }
  }
}

/// A compositing layer supplied by the legacy renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpriteLayer {
  Base,
  Emissive,
  Mask,
  Shadow,
}

/// Pixel rectangle in an atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteRect {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
}

impl SpriteRect {
  /// Creates a rectangle in atlas pixels.
  #[must_use]
  pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
    Self {
      x,
      y,
      width,
      height,
    }
  }

  /// Returns whether this rectangle fits inside an atlas of the given size.
  #[must_use]
  pub const fn is_within(self, atlas_width: u32, atlas_height: u32) -> bool {
    self.x.saturating_add(self.width) <= atlas_width
      && self.y.saturating_add(self.height) <= atlas_height
  }
}

/// Stable semantic lookup entry used by scene construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteDescriptor {
  pub atlas: AtlasId,
  pub rect: SpriteRect,
  pub layers: &'static [SpriteLayer],
}

const BASE: &[SpriteLayer] = &[SpriteLayer::Base];
const LIT: &[SpriteLayer] = &[
  SpriteLayer::Base,
  SpriteLayer::Emissive,
  SpriteLayer::Shadow,
];

const SPRITE_CELL_SIZE: u32 = 32;
const SPRITE_COLUMNS: u32 = 16;

/// Converts a legacy one-based sprite id within a sheet to its pixel cell.
///
/// The legacy registration code numbers each 32-pixel cell from one and
/// advances rows in groups of sixteen (`DRL_COLS`). Keeping that conversion
/// here makes the semantic tables explicit without importing the legacy
/// runtime or its identifiers.
const fn legacy_slot(slot: u32) -> SpriteRect {
  let index = slot.saturating_sub(1);
  SpriteRect::new(
    (index % SPRITE_COLUMNS) * SPRITE_CELL_SIZE,
    (index / SPRITE_COLUMNS) * SPRITE_CELL_SIZE,
    SPRITE_CELL_SIZE,
    SPRITE_CELL_SIZE,
  )
}

/// Returns the descriptor for a currently implemented terrain tile.
#[must_use]
pub const fn tile_sprite(tile: TileKind) -> SpriteDescriptor {
  match tile {
    TileKind::Floor => SpriteDescriptor {
      atlas: AtlasId::Levels,
      rect: legacy_slot(1),
      layers: BASE,
    },
    TileKind::Wall => SpriteDescriptor {
      atlas: AtlasId::Levels,
      rect: legacy_slot(15 * SPRITE_COLUMNS + 1),
      layers: BASE,
    },
    TileKind::DoorClosed => SpriteDescriptor {
      atlas: AtlasId::DoorsAndDecorations,
      rect: legacy_slot(1),
      layers: BASE,
    },
    TileKind::DoorOpen => SpriteDescriptor {
      atlas: AtlasId::DoorsAndDecorations,
      rect: legacy_slot(3 * SPRITE_COLUMNS + 1),
      layers: BASE,
    },
    TileKind::StairsDown => SpriteDescriptor {
      atlas: AtlasId::DoorsAndDecorations,
      rect: legacy_slot(7 * SPRITE_COLUMNS + 1),
      layers: LIT,
    },
  }
}

/// Returns the descriptor for a currently implemented actor archetype.
#[must_use]
pub const fn actor_sprite(kind: Option<MonsterKind>) -> SpriteDescriptor {
  match kind {
    None => SpriteDescriptor {
      atlas: AtlasId::Dguy,
      rect: legacy_slot(1),
      layers: LIT,
    },
    Some(MonsterKind::FormerHuman) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(1),
      layers: LIT,
    },
    Some(MonsterKind::FormerSergeant) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(2),
      layers: LIT,
    },
    Some(MonsterKind::Imp) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(5),
      layers: LIT,
    },
    Some(MonsterKind::Demon) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(6),
      layers: LIT,
    },
  }
}

/// Returns the descriptor for a currently implemented item archetype.
#[must_use]
pub const fn item_sprite(archetype: ItemArchetype) -> SpriteDescriptor {
  match archetype {
    ItemArchetype::Unknown => SpriteDescriptor {
      atlas: AtlasId::Fx,
      rect: legacy_slot(1),
      layers: BASE,
    },
    ItemArchetype::CombatKnife => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(2),
      layers: LIT,
    },
    ItemArchetype::Pistol => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(4),
      layers: LIT,
    },
    ItemArchetype::Shotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(5),
      layers: LIT,
    },
    ItemArchetype::GreenArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: LIT,
    },
    ItemArchetype::Ammo9mm => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 7),
      layers: LIT,
    },
    ItemArchetype::AmmoShells => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 9),
      layers: LIT,
    },
    ItemArchetype::SmallMedPack => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(3 * SPRITE_COLUMNS + 9),
      layers: LIT,
    },
    ItemArchetype::LargeMedPack => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(3 * SPRITE_COLUMNS + 10),
      layers: LIT,
    },
    ItemArchetype::PhaseDevice => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 15),
      layers: LIT,
    },
  }
}

/// Returns the relative path to an imported graphics asset.
#[must_use]
pub const fn asset_path(file: &str) -> &str {
  file
}

/// The legacy graphics revision imported by the asset pipeline.
pub const LEGACY_REVISION: &str = "17d9be1204751899b2d69d8d3a2dde247bd0cc5c5";

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_current_semantics_have_descriptors() {
    for tile in [
      TileKind::Floor,
      TileKind::Wall,
      TileKind::DoorClosed,
      TileKind::DoorOpen,
      TileKind::StairsDown,
    ] {
      let descriptor = tile_sprite(tile);
      assert!(descriptor.rect.width > 0);
      assert!(descriptor.rect.is_within(
        descriptor.atlas.dimensions().0,
        descriptor.atlas.dimensions().1
      ));
    }
    for kind in [
      None,
      Some(MonsterKind::FormerHuman),
      Some(MonsterKind::FormerSergeant),
      Some(MonsterKind::Imp),
      Some(MonsterKind::Demon),
    ] {
      let descriptor = actor_sprite(kind);
      assert!(descriptor.rect.width > 0);
      assert!(descriptor.rect.is_within(
        descriptor.atlas.dimensions().0,
        descriptor.atlas.dimensions().1
      ));
    }
    assert_eq!(
      item_sprite(ItemArchetype::Pistol).atlas,
      AtlasId::GunsAndPickups
    );
    assert_eq!(
      AtlasId::Dguy.layer_path(SpriteLayer::Emissive),
      "dguy_emissive.png"
    );
  }

  #[test]
  fn current_item_slots_match_legacy_grid() {
    let expected = [
      (ItemArchetype::Unknown, AtlasId::Fx, (0, 0)),
      (ItemArchetype::CombatKnife, AtlasId::GunsAndPickups, (32, 0)),
      (ItemArchetype::Pistol, AtlasId::GunsAndPickups, (96, 0)),
      (ItemArchetype::Shotgun, AtlasId::GunsAndPickups, (128, 0)),
      (ItemArchetype::GreenArmor, AtlasId::GunsAndPickups, (0, 32)),
      (ItemArchetype::Ammo9mm, AtlasId::GunsAndPickups, (192, 32)),
      (
        ItemArchetype::AmmoShells,
        AtlasId::GunsAndPickups,
        (256, 32),
      ),
      (
        ItemArchetype::SmallMedPack,
        AtlasId::GunsAndPickups,
        (256, 96),
      ),
      (
        ItemArchetype::LargeMedPack,
        AtlasId::GunsAndPickups,
        (288, 96),
      ),
      (
        ItemArchetype::PhaseDevice,
        AtlasId::GunsAndPickups,
        (448, 32),
      ),
    ];
    for (archetype, atlas, (x, y)) in expected {
      let descriptor = item_sprite(archetype);
      assert_eq!(descriptor.atlas, atlas);
      assert_eq!((descriptor.rect.x, descriptor.rect.y), (x, y));
      assert!(descriptor.rect.is_within(
        descriptor.atlas.dimensions().0,
        descriptor.atlas.dimensions().1
      ));
    }
  }

  #[test]
  fn current_actor_and_tile_slots_match_legacy_grid() {
    assert_eq!(tile_sprite(TileKind::Floor).rect, legacy_slot(1));
    assert_eq!(tile_sprite(TileKind::Wall).rect, legacy_slot(241));
    assert_eq!(tile_sprite(TileKind::DoorClosed).rect, legacy_slot(1));
    assert_eq!(tile_sprite(TileKind::DoorOpen).rect, legacy_slot(49));
    assert_eq!(tile_sprite(TileKind::StairsDown).rect, legacy_slot(113));
    assert_eq!(actor_sprite(None).rect, legacy_slot(1));
    assert_eq!(
      actor_sprite(Some(MonsterKind::FormerHuman)).rect,
      legacy_slot(1)
    );
    assert_eq!(
      actor_sprite(Some(MonsterKind::FormerSergeant)).rect,
      legacy_slot(2)
    );
    assert_eq!(actor_sprite(Some(MonsterKind::Imp)).rect, legacy_slot(5));
    assert_eq!(actor_sprite(Some(MonsterKind::Demon)).rect, legacy_slot(6));
  }

  #[test]
  fn atlas_dimensions_match_imported_sheets() {
    assert_eq!(AtlasId::Dguy.dimensions(), (512, 64));
    assert_eq!(AtlasId::Enemies.dimensions(), (512, 192));
    assert_eq!(AtlasId::EnemiesBig.dimensions(), (512, 384));
    assert_eq!(AtlasId::GunsAndPickups.dimensions(), (512, 160));
    assert_eq!(AtlasId::Levels.dimensions(), (512, 1152));
    assert_eq!(AtlasId::DoorsAndDecorations.dimensions(), (512, 288));
    assert_eq!(AtlasId::Fx.dimensions(), (512, 64));
  }
}
