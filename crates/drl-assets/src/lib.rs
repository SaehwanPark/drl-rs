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

  /// Registered source layers in deterministic legacy registration order.
  #[must_use]
  pub const fn layers(self) -> &'static [SpriteLayer] {
    match self {
      Self::Dguy => PLAYER_LAYERS,
      Self::Enemies | Self::EnemiesBig => ACTOR_LAYERS,
      Self::GunsAndPickups => ITEM_LAYERS,
      Self::Levels => LEVEL_LAYERS,
      Self::DoorsAndDecorations => DOOR_LAYERS,
      Self::Fx => FX_LAYERS,
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

  /// Resolves a registered layer to the imported source metadata a future
  /// texture uploader will need. This remains a pure lookup; it does not read
  /// or decode the referenced file.
  #[must_use]
  pub const fn texture_source(self, layer: SpriteLayer) -> AtlasTextureSource {
    let (width, height) = self.dimensions();
    AtlasTextureSource {
      path: self.layer_path(layer),
      width,
      height,
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

/// The renderer-neutral input role of one registered sprite layer.
///
/// The legacy sprite shader samples these inputs independently: the base
/// image supplies normal color, the mask supplies optional colorization, the
/// shadow image supplies the outline mask, and the emissive image supplies an
/// emission mask. Naming the roles here keeps a future compositor from
/// guessing based on file names while leaving blend equations to the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerRole {
  BaseColor,
  ColorizationMask,
  OutlineMask,
  EmissiveMask,
}

impl SpriteLayer {
  /// Returns the shader input role represented by this source layer.
  #[must_use]
  pub const fn role(self) -> LayerRole {
    match self {
      Self::Base => LayerRole::BaseColor,
      Self::Mask => LayerRole::ColorizationMask,
      Self::Shadow => LayerRole::OutlineMask,
      Self::Emissive => LayerRole::EmissiveMask,
    }
  }
}

/// Imported image metadata for one atlas compositing layer.
///
/// Paths are relative to the license-cleared graphics bundle. A frontend owns
/// loading, decoding, GPU upload, and texture-origin policy at its boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtlasTextureSource {
  pub path: &'static str,
  pub width: u32,
  pub height: u32,
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
    self.x <= atlas_width
      && self.y <= atlas_height
      && self.width <= atlas_width - self.x
      && self.height <= atlas_height - self.y
  }

  /// Converts this image-space rectangle to normalized top-left-origin UVs.
  ///
  /// The conversion is intentionally renderer-neutral. A backend using a
  /// bottom-left texture origin can invert the V coordinates at its boundary.
  #[must_use]
  pub fn uv_rect(self, atlas_width: u32, atlas_height: u32) -> Option<SpriteUv> {
    if atlas_width == 0 || atlas_height == 0 || !self.is_within(atlas_width, atlas_height) {
      return None;
    }
    let atlas_width = atlas_width as f32;
    let atlas_height = atlas_height as f32;
    Some(SpriteUv {
      u_min: self.x as f32 / atlas_width,
      v_min: self.y as f32 / atlas_height,
      u_max: self.x.saturating_add(self.width) as f32 / atlas_width,
      v_max: self.y.saturating_add(self.height) as f32 / atlas_height,
    })
  }
}

/// Normalized top-left-origin UV rectangle for a sprite cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteUv {
  pub u_min: f32,
  pub v_min: f32,
  pub u_max: f32,
  pub v_max: f32,
}

/// Stable semantic lookup entry used by scene construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteDescriptor {
  pub atlas: AtlasId,
  pub rect: SpriteRect,
  pub layers: &'static [SpriteLayer],
  /// Optional source-backed frame metadata; presentation owns timing.
  pub animation: Option<SpriteAnimation>,
}

/// Renderer-neutral frame metadata extracted from a legacy sprite descriptor.
///
/// The browser or another presentation backend chooses how to turn progress
/// into a frame. This value records only the pinned content facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteAnimation {
  pub frame_count: u16,
  pub frame_time_ms: u32,
}

impl SpriteDescriptor {
  /// Returns one vertically adjacent frame rectangle when it is in range.
  #[must_use]
  pub const fn frame_rect(self, frame_index: u16) -> Option<SpriteRect> {
    let frame_count = match self.animation {
      Some(animation) => animation.frame_count,
      None => 1,
    };
    if frame_count == 0 || frame_index >= frame_count {
      return None;
    }
    let offset = (frame_index as u32).saturating_mul(self.rect.height);
    let rect = SpriteRect::new(
      self.rect.x,
      self.rect.y.saturating_add(offset),
      self.rect.width,
      self.rect.height,
    );
    let (atlas_width, atlas_height) = self.atlas.dimensions();
    if rect.is_within(atlas_width, atlas_height) {
      Some(rect)
    } else {
      None
    }
  }
}

const LEVEL_LAYERS: &[SpriteLayer] = &[SpriteLayer::Base, SpriteLayer::Mask, SpriteLayer::Emissive];
const DOOR_LAYERS: &[SpriteLayer] = &[
  SpriteLayer::Base,
  SpriteLayer::Mask,
  SpriteLayer::Shadow,
  SpriteLayer::Emissive,
];
const ITEM_LAYERS: &[SpriteLayer] = DOOR_LAYERS;
const ACTOR_LAYERS: &[SpriteLayer] = &[
  SpriteLayer::Base,
  SpriteLayer::Shadow,
  SpriteLayer::Emissive,
];
const PLAYER_LAYERS: &[SpriteLayer] = &[
  SpriteLayer::Base,
  SpriteLayer::Mask,
  SpriteLayer::Shadow,
  SpriteLayer::Emissive,
];
const FX_LAYERS: &[SpriteLayer] = LEVEL_LAYERS;
const STATIC_ANIMATION: Option<SpriteAnimation> = None;
const TWO_FRAME_ANIMATION: Option<SpriteAnimation> = Some(SpriteAnimation {
  frame_count: 2,
  frame_time_ms: 500,
});
#[cfg(test)]
const ALL_ATLASES: &[AtlasId] = &[
  AtlasId::Dguy,
  AtlasId::Enemies,
  AtlasId::EnemiesBig,
  AtlasId::GunsAndPickups,
  AtlasId::Levels,
  AtlasId::DoorsAndDecorations,
  AtlasId::Fx,
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
      layers: LEVEL_LAYERS,
      animation: STATIC_ANIMATION,
    },
    TileKind::Wall => SpriteDescriptor {
      atlas: AtlasId::Levels,
      rect: legacy_slot(15 * SPRITE_COLUMNS + 1),
      layers: LEVEL_LAYERS,
      animation: STATIC_ANIMATION,
    },
    TileKind::DoorClosed => SpriteDescriptor {
      atlas: AtlasId::DoorsAndDecorations,
      rect: legacy_slot(1),
      layers: DOOR_LAYERS,
      animation: STATIC_ANIMATION,
    },
    TileKind::DoorOpen => SpriteDescriptor {
      atlas: AtlasId::DoorsAndDecorations,
      rect: legacy_slot(3 * SPRITE_COLUMNS + 1),
      layers: DOOR_LAYERS,
      animation: STATIC_ANIMATION,
    },
    TileKind::StairsDown => SpriteDescriptor {
      atlas: AtlasId::DoorsAndDecorations,
      rect: legacy_slot(7 * SPRITE_COLUMNS + 1),
      layers: DOOR_LAYERS,
      animation: STATIC_ANIMATION,
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
      layers: PLAYER_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    },
    Some(MonsterKind::FormerHuman) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(1),
      layers: ACTOR_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    },
    Some(MonsterKind::FormerSergeant) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(2),
      layers: ACTOR_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    },
    Some(MonsterKind::Imp) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(5),
      layers: ACTOR_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    },
    Some(MonsterKind::Demon) => SpriteDescriptor {
      atlas: AtlasId::Enemies,
      rect: legacy_slot(6),
      layers: ACTOR_LAYERS,
      animation: TWO_FRAME_ANIMATION,
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
      layers: FX_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::CombatKnife => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(2),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Chainsaw => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(3),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Pistol => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(4),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Shotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(5),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::CombatShotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(6),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::DoubleShotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(7),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Blaster => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(4),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::LaserRifle => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::MissileLauncher => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(9),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::NuclearPlasmaRifle => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::NuclearBfg9000 => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(11),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Bfg10k => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(13),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::MegaBuster => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::GrammatonBeretta => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(4),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::FragShotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(6),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::RevenantsLauncher => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(9),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Railgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AcidSpitter => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::CombatPistol => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(4),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AssaultShotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(6),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::PlasmaShotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(5),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Jackhammer => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(6),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::SuperShotgun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(7),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::TristarBlaster => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(7),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::ButchersCleaver => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(15),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Mjollnir => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(15),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::SubtleKnife => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(2),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Trigun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(4),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AntiFreakJackal => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(4),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Minigun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(8),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Chaingun => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(8),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::RocketLauncher => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(9),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::PlasmaRifle => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Bfg9000 => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(11),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::GreenArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::BlueArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::RedArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::OnyxArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::PhaseshiftArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::GothicArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::MaleksArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::CyberneticArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Necroarmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::MedicalPowerarmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::LavaArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::ShieldedArmor => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 1),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::Ammo9mm => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 7),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AmmoShells => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 9),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AmmoRockets => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 11),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AmmoCells => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 13),
      layers: ITEM_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    },
    ItemArchetype::AmmoPackRockets => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 12),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AmmoPackCells => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 14),
      layers: ITEM_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    },
    ItemArchetype::AmmoPack9mm => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 8),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::AmmoPackShells => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::SmallMedPack => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(3 * SPRITE_COLUMNS + 9),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::LargeMedPack => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(3 * SPRITE_COLUMNS + 10),
      layers: ITEM_LAYERS,
      animation: STATIC_ANIMATION,
    },
    ItemArchetype::PhaseDevice => SpriteDescriptor {
      atlas: AtlasId::GunsAndPickups,
      rect: legacy_slot(SPRITE_COLUMNS + 15),
      layers: ITEM_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    },
  }
}

/// Returns the relative path to an imported graphics asset.
#[must_use]
pub const fn asset_path(file: &str) -> &str {
  file
}

/// The legacy graphics revision imported by the asset pipeline.
pub const LEGACY_REVISION: &str = "17d9be1204751899b2d69d8d3a2dde247bd0cc5c";

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
      assert_eq!(descriptor.layers, descriptor.atlas.layers());
      assert!(
        descriptor
          .rect
          .uv_rect(
            descriptor.atlas.dimensions().0,
            descriptor.atlas.dimensions().1
          )
          .is_some()
      );
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
      assert_eq!(descriptor.layers, descriptor.atlas.layers());
      assert!(
        descriptor
          .rect
          .uv_rect(
            descriptor.atlas.dimensions().0,
            descriptor.atlas.dimensions().1
          )
          .is_some()
      );
    }
    for archetype in ItemArchetype::ALL.iter().copied() {
      let descriptor = item_sprite(archetype);
      assert!(descriptor.rect.is_within(
        descriptor.atlas.dimensions().0,
        descriptor.atlas.dimensions().1
      ));
      assert_eq!(descriptor.layers, descriptor.atlas.layers());
      let uv = descriptor
        .rect
        .uv_rect(
          descriptor.atlas.dimensions().0,
          descriptor.atlas.dimensions().1,
        )
        .expect("in-bounds current descriptor has UVs");
      assert!((0.0..=1.0).contains(&uv.u_min));
      assert!((0.0..=1.0).contains(&uv.v_min));
      assert!((0.0..=1.0).contains(&uv.u_max));
      assert!((0.0..=1.0).contains(&uv.v_max));
    }
    assert_eq!(
      AtlasId::Dguy.layer_path(SpriteLayer::Emissive),
      "dguy_emissive.png"
    );
  }

  #[test]
  fn registered_layers_resolve_texture_sources() {
    for atlas in ALL_ATLASES {
      for layer in atlas.layers() {
        let source = atlas.texture_source(*layer);
        assert_eq!(source.path, atlas.layer_path(*layer));
        assert_eq!((source.width, source.height), atlas.dimensions());
        assert!(!source.path.is_empty());
      }
    }
  }

  #[test]
  fn current_item_slots_match_legacy_grid() {
    let expected = [
      (ItemArchetype::Unknown, AtlasId::Fx, (0, 0)),
      (ItemArchetype::CombatKnife, AtlasId::GunsAndPickups, (32, 0)),
      (ItemArchetype::Chainsaw, AtlasId::GunsAndPickups, (64, 0)),
      (ItemArchetype::Pistol, AtlasId::GunsAndPickups, (96, 0)),
      (ItemArchetype::Shotgun, AtlasId::GunsAndPickups, (128, 0)),
      (
        ItemArchetype::CombatShotgun,
        AtlasId::GunsAndPickups,
        (160, 0),
      ),
      (
        ItemArchetype::DoubleShotgun,
        AtlasId::GunsAndPickups,
        (192, 0),
      ),
      (ItemArchetype::Blaster, AtlasId::GunsAndPickups, (96, 0)),
      (ItemArchetype::LaserRifle, AtlasId::GunsAndPickups, (288, 0)),
      (
        ItemArchetype::MissileLauncher,
        AtlasId::GunsAndPickups,
        (256, 0),
      ),
      (
        ItemArchetype::NuclearPlasmaRifle,
        AtlasId::GunsAndPickups,
        (288, 0),
      ),
      (
        ItemArchetype::NuclearBfg9000,
        AtlasId::GunsAndPickups,
        (320, 0),
      ),
      (ItemArchetype::Bfg10k, AtlasId::GunsAndPickups, (384, 0)),
      (ItemArchetype::MegaBuster, AtlasId::GunsAndPickups, (288, 0)),
      (
        ItemArchetype::GrammatonBeretta,
        AtlasId::GunsAndPickups,
        (96, 0),
      ),
      (
        ItemArchetype::FragShotgun,
        AtlasId::GunsAndPickups,
        (160, 0),
      ),
      (
        ItemArchetype::RevenantsLauncher,
        AtlasId::GunsAndPickups,
        (256, 0),
      ),
      (ItemArchetype::Railgun, AtlasId::GunsAndPickups, (288, 0)),
      (
        ItemArchetype::AcidSpitter,
        AtlasId::GunsAndPickups,
        (288, 0),
      ),
      (
        ItemArchetype::CombatPistol,
        AtlasId::GunsAndPickups,
        (96, 0),
      ),
      (
        ItemArchetype::AssaultShotgun,
        AtlasId::GunsAndPickups,
        (160, 0),
      ),
      (
        ItemArchetype::PlasmaShotgun,
        AtlasId::GunsAndPickups,
        (128, 0),
      ),
      (ItemArchetype::Jackhammer, AtlasId::GunsAndPickups, (160, 0)),
      (
        ItemArchetype::SuperShotgun,
        AtlasId::GunsAndPickups,
        (192, 0),
      ),
      (
        ItemArchetype::TristarBlaster,
        AtlasId::GunsAndPickups,
        (192, 0),
      ),
      (
        ItemArchetype::ButchersCleaver,
        AtlasId::GunsAndPickups,
        (448, 0),
      ),
      (ItemArchetype::Mjollnir, AtlasId::GunsAndPickups, (448, 0)),
      (ItemArchetype::SubtleKnife, AtlasId::GunsAndPickups, (32, 0)),
      (ItemArchetype::Trigun, AtlasId::GunsAndPickups, (96, 0)),
      (
        ItemArchetype::AntiFreakJackal,
        AtlasId::GunsAndPickups,
        (96, 0),
      ),
      (ItemArchetype::Minigun, AtlasId::GunsAndPickups, (224, 0)),
      (ItemArchetype::Chaingun, AtlasId::GunsAndPickups, (224, 0)),
      (
        ItemArchetype::RocketLauncher,
        AtlasId::GunsAndPickups,
        (256, 0),
      ),
      (
        ItemArchetype::PlasmaRifle,
        AtlasId::GunsAndPickups,
        (288, 0),
      ),
      (ItemArchetype::Bfg9000, AtlasId::GunsAndPickups, (320, 0)),
      (ItemArchetype::GreenArmor, AtlasId::GunsAndPickups, (0, 32)),
      (ItemArchetype::BlueArmor, AtlasId::GunsAndPickups, (0, 32)),
      (ItemArchetype::RedArmor, AtlasId::GunsAndPickups, (0, 32)),
      (ItemArchetype::OnyxArmor, AtlasId::GunsAndPickups, (0, 32)),
      (
        ItemArchetype::PhaseshiftArmor,
        AtlasId::GunsAndPickups,
        (0, 32),
      ),
      (ItemArchetype::GothicArmor, AtlasId::GunsAndPickups, (0, 32)),
      (ItemArchetype::MaleksArmor, AtlasId::GunsAndPickups, (0, 32)),
      (
        ItemArchetype::CyberneticArmor,
        AtlasId::GunsAndPickups,
        (0, 32),
      ),
      (ItemArchetype::Necroarmor, AtlasId::GunsAndPickups, (0, 32)),
      (
        ItemArchetype::MedicalPowerarmor,
        AtlasId::GunsAndPickups,
        (0, 32),
      ),
      (ItemArchetype::LavaArmor, AtlasId::GunsAndPickups, (0, 32)),
      (
        ItemArchetype::ShieldedArmor,
        AtlasId::GunsAndPickups,
        (0, 32),
      ),
      (ItemArchetype::Ammo9mm, AtlasId::GunsAndPickups, (192, 32)),
      (
        ItemArchetype::AmmoShells,
        AtlasId::GunsAndPickups,
        (256, 32),
      ),
      (
        ItemArchetype::AmmoRockets,
        AtlasId::GunsAndPickups,
        (320, 32),
      ),
      (ItemArchetype::AmmoCells, AtlasId::GunsAndPickups, (384, 32)),
      (
        ItemArchetype::AmmoPackRockets,
        AtlasId::GunsAndPickups,
        (352, 32),
      ),
      (
        ItemArchetype::AmmoPackCells,
        AtlasId::GunsAndPickups,
        (416, 32),
      ),
      (
        ItemArchetype::AmmoPack9mm,
        AtlasId::GunsAndPickups,
        (224, 32),
      ),
      (
        ItemArchetype::AmmoPackShells,
        AtlasId::GunsAndPickups,
        (288, 32),
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
  fn evidenced_animation_metadata_is_explicit_and_bounded() {
    for kind in [
      None,
      Some(MonsterKind::FormerHuman),
      Some(MonsterKind::FormerSergeant),
      Some(MonsterKind::Imp),
      Some(MonsterKind::Demon),
    ] {
      let descriptor = actor_sprite(kind);
      assert_eq!(
        descriptor.animation,
        Some(SpriteAnimation {
          frame_count: 2,
          frame_time_ms: 500,
        })
      );
      assert_eq!(descriptor.frame_rect(0), Some(descriptor.rect));
      let second = descriptor.frame_rect(1).expect("second frame row");
      assert_eq!(second.x, descriptor.rect.x);
      assert_eq!(second.y, descriptor.rect.y + descriptor.rect.height);
      assert_eq!(second.width, descriptor.rect.width);
      assert_eq!(second.height, descriptor.rect.height);
      assert!(descriptor.frame_rect(2).is_none());
    }

    let phase = item_sprite(ItemArchetype::PhaseDevice);
    assert_eq!(
      phase.animation.map(|animation| animation.frame_count),
      Some(2)
    );
    assert_eq!(
      phase.animation.map(|animation| animation.frame_time_ms),
      Some(500)
    );
    assert_eq!(phase.frame_rect(0), Some(phase.rect));
    assert!(phase.frame_rect(1).is_some());

    for tile in [
      TileKind::Floor,
      TileKind::Wall,
      TileKind::DoorClosed,
      TileKind::DoorOpen,
      TileKind::StairsDown,
    ] {
      let descriptor = tile_sprite(tile);
      assert_eq!(descriptor.animation, None);
      assert_eq!(descriptor.frame_rect(0), Some(descriptor.rect));
      assert!(descriptor.frame_rect(1).is_none());
    }
    for item in ItemArchetype::ALL.iter().copied() {
      let descriptor = item_sprite(item);
      assert_eq!(descriptor.frame_rect(0), Some(descriptor.rect));
      match descriptor.animation {
        Some(animation) => {
          assert!(animation.frame_count > 1);
          assert!(descriptor.frame_rect(1).is_some());
        }
        None => assert!(descriptor.frame_rect(1).is_none()),
      }
    }
    let cells = item_sprite(ItemArchetype::AmmoCells);
    assert_eq!(
      cells.animation.map(|animation| animation.frame_count),
      Some(2)
    );
    assert!(cells.frame_rect(1).is_some());
    let cell_pack = item_sprite(ItemArchetype::AmmoPackCells);
    assert_eq!(
      cell_pack.animation.map(|animation| animation.frame_count),
      Some(2)
    );
    assert!(cell_pack.frame_rect(1).is_some());
  }

  #[test]
  fn frame_rect_rejects_atlas_overflow() {
    let descriptor = SpriteDescriptor {
      atlas: AtlasId::Dguy,
      rect: SpriteRect::new(0, 32, 32, 32),
      layers: PLAYER_LAYERS,
      animation: TWO_FRAME_ANIMATION,
    };
    assert_eq!(descriptor.frame_rect(0), Some(descriptor.rect));
    assert!(descriptor.frame_rect(1).is_none());
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

  #[test]
  fn atlas_layers_match_registered_source_order() {
    assert_eq!(
      AtlasId::Levels.layers(),
      &[SpriteLayer::Base, SpriteLayer::Mask, SpriteLayer::Emissive]
    );
    assert_eq!(
      AtlasId::DoorsAndDecorations.layers(),
      &[
        SpriteLayer::Base,
        SpriteLayer::Mask,
        SpriteLayer::Shadow,
        SpriteLayer::Emissive,
      ]
    );
    assert_eq!(
      AtlasId::GunsAndPickups.layers(),
      AtlasId::DoorsAndDecorations.layers()
    );
    assert_eq!(
      AtlasId::Enemies.layers(),
      &[
        SpriteLayer::Base,
        SpriteLayer::Shadow,
        SpriteLayer::Emissive
      ]
    );
    assert_eq!(AtlasId::EnemiesBig.layers(), AtlasId::Enemies.layers());
    assert_eq!(
      AtlasId::Dguy.layers(),
      &[
        SpriteLayer::Base,
        SpriteLayer::Mask,
        SpriteLayer::Shadow,
        SpriteLayer::Emissive,
      ]
    );
    assert_eq!(AtlasId::Fx.layers(), AtlasId::Levels.layers());
  }

  #[test]
  fn sprite_layers_have_explicit_shader_input_roles() {
    assert_eq!(SpriteLayer::Base.role(), LayerRole::BaseColor);
    assert_eq!(SpriteLayer::Mask.role(), LayerRole::ColorizationMask);
    assert_eq!(SpriteLayer::Shadow.role(), LayerRole::OutlineMask);
    assert_eq!(SpriteLayer::Emissive.role(), LayerRole::EmissiveMask);
    assert_eq!(
      AtlasId::Enemies
        .layers()
        .iter()
        .map(|layer| layer.role())
        .collect::<Vec<_>>(),
      vec![
        LayerRole::BaseColor,
        LayerRole::OutlineMask,
        LayerRole::EmissiveMask
      ]
    );
  }

  #[test]
  fn sprite_rect_uv_conversion_is_normalized_and_top_left_origin() {
    let uv = SpriteRect::new(32, 64, 32, 32)
      .uv_rect(512, 256)
      .expect("valid atlas rectangle");
    assert!((uv.u_min - 0.0625).abs() < f32::EPSILON);
    assert!((uv.v_min - 0.25).abs() < f32::EPSILON);
    assert!((uv.u_max - 0.125).abs() < f32::EPSILON);
    assert!((uv.v_max - 0.375).abs() < f32::EPSILON);
  }

  #[test]
  fn sprite_rect_uv_conversion_rejects_invalid_atlases_and_rectangles() {
    let rect = SpriteRect::new(0, 0, 32, 32);
    assert!(rect.uv_rect(0, 256).is_none());
    assert!(rect.uv_rect(512, 0).is_none());
    assert!(SpriteRect::new(500, 0, 32, 32).uv_rect(512, 256).is_none());
    assert!(SpriteRect::new(0, 250, 32, 32).uv_rect(512, 256).is_none());
    assert!(
      SpriteRect::new(u32::MAX, 0, 1, 1)
        .uv_rect(u32::MAX, 1)
        .is_none()
    );
    assert!(
      SpriteRect::new(0, u32::MAX, 1, 1)
        .uv_rect(1, u32::MAX)
        .is_none()
    );
  }
}
