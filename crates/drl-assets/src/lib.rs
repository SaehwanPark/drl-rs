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

/// Returns the descriptor for a currently implemented terrain tile.
#[must_use]
pub const fn tile_sprite(tile: TileKind) -> SpriteDescriptor {
  let rect = SpriteRect::new(0, 0, 32, 32);
  match tile {
    TileKind::Floor | TileKind::Wall | TileKind::DoorClosed | TileKind::DoorOpen => {
      SpriteDescriptor {
        atlas: AtlasId::Levels,
        rect,
        layers: BASE,
      }
    }
    TileKind::StairsDown => SpriteDescriptor {
      atlas: AtlasId::Levels,
      rect,
      layers: LIT,
    },
  }
}

/// Returns the descriptor for a currently implemented actor archetype.
#[must_use]
pub const fn actor_sprite(kind: Option<MonsterKind>) -> SpriteDescriptor {
  let rect = SpriteRect::new(0, 0, 32, 32);
  match kind {
    None => SpriteDescriptor {
      atlas: AtlasId::Dguy,
      rect,
      layers: LIT,
    },
    Some(MonsterKind::Demon) => SpriteDescriptor {
      atlas: AtlasId::EnemiesBig,
      rect,
      layers: LIT,
    },
    Some(_) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect,
      layers: LIT,
    },
  }
}

/// Returns the descriptor for a currently implemented item archetype.
#[must_use]
pub const fn item_sprite(archetype: ItemArchetype) -> SpriteDescriptor {
  let rect = SpriteRect::new(0, 0, 32, 32);
  match archetype {
    ItemArchetype::Unknown => SpriteDescriptor {
      atlas: AtlasId::Fx,
      rect,
      layers: BASE,
    },
    _ => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect,
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
      assert!(tile_sprite(tile).rect.width > 0);
    }
    for kind in [
      None,
      Some(MonsterKind::FormerHuman),
      Some(MonsterKind::FormerSergeant),
      Some(MonsterKind::Imp),
      Some(MonsterKind::Demon),
    ] {
      assert!(actor_sprite(kind).rect.width > 0);
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
}
