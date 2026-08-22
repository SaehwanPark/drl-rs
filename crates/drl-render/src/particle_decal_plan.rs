use drl_assets::SpriteDescriptor;
use drl_protocol::Position;

use super::{LayerDraw, LightingBand, ParticleDecalStore, PixelRect, PixelViewport};

/// Caller-owned mapping from a retained decal sprite identifier to atlas data.
///
/// The store deliberately retains only the identifier. Resolving that
/// identifier is a presentation concern, so a frontend can select a content
/// table without adding asset knowledge to the simulation or storage boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleDecalSprite {
  pub sprite_id: u32,
  pub descriptor: SpriteDescriptor,
  /// Caller-resolved visibility/light band for this decal source.
  pub lighting: LightingBand,
}

/// Builds the renderer-neutral layer plan for retained particle decals.
///
/// Requests are read exactly once in store order. Duplicate requests remain
/// duplicate draw groups. A request is omitted when its one-based cell cannot
/// be represented as a map position, falls outside the supplied viewport, or
/// has no caller-provided sprite descriptor. The store and simulation state
/// are never mutated.
#[must_use]
pub fn particle_decal_draw_plan(
  store: &ParticleDecalStore,
  viewport: PixelViewport,
  sprites: &[ParticleDecalSprite],
) -> Vec<LayerDraw> {
  let mut plan = Vec::new();

  for (insertion_index, insertion) in store.entries().iter().enumerate() {
    let Some(position) = legacy_cell_position(insertion.placement.cell) else {
      continue;
    };
    let Some(destination) =
      particle_decal_destination(insertion.placement.pixel, position, viewport)
    else {
      continue;
    };
    let Some(sprite) = sprites
      .iter()
      .find(|sprite| sprite.sprite_id == insertion.sprite_id)
    else {
      continue;
    };
    let Some(uv) = sprite.descriptor.frame_rect(0).and_then(|frame| {
      frame.uv_rect(
        sprite.descriptor.atlas.dimensions().0,
        sprite.descriptor.atlas.dimensions().1,
      )
    }) else {
      continue;
    };

    let Ok(sprite_index) = u32::try_from(insertion_index) else {
      continue;
    };
    plan.extend(
      sprite
        .descriptor
        .layers
        .iter()
        .copied()
        .map(|layer| LayerDraw {
          sprite_index,
          atlas: sprite.descriptor.atlas,
          layer,
          role: layer.role(),
          source: sprite.descriptor.atlas.texture_source(layer),
          lighting: sprite.lighting,
          colorization_tint: [0, 0, 0, 0],
          animation: sprite.descriptor.animation,
          destination,
          uv,
        }),
    );
  }

  plan
}

fn legacy_cell_position(cell: [i32; 2]) -> Option<Position> {
  Some(Position::new(
    cell[0].checked_sub(1)?,
    cell[1].checked_sub(1)?,
  ))
}

fn particle_decal_destination(
  pixel: [i32; 2],
  cell: Position,
  viewport: PixelViewport,
) -> Option<PixelRect> {
  viewport.tile_rect(cell)?;
  let origin_x = u32::try_from(pixel[0].checked_sub(32)?).ok()?;
  let origin_y = u32::try_from(pixel[1].checked_sub(32)?).ok()?;
  let x = viewport
    .offset_x
    .checked_add(origin_x.checked_mul(viewport.tile_size)? / 32)?;
  let y = viewport
    .offset_y
    .checked_add(origin_y.checked_mul(viewport.tile_size)? / 32)?;
  Some(PixelRect {
    x,
    y,
    width: viewport.tile_size,
    height: viewport.tile_size,
  })
}

#[cfg(test)]
mod tests {
  use super::drl_render_test_support::insertion;
  use super::*;
  use drl_assets::{AtlasId, SpriteDescriptor, SpriteLayer};

  const SPRITE: ParticleDecalSprite = ParticleDecalSprite {
    sprite_id: 7,
    descriptor: SpriteDescriptor {
      atlas: AtlasId::Fx,
      rect: drl_assets::SpriteRect::new(0, 0, 32, 32),
      layers: &[SpriteLayer::Base, SpriteLayer::Mask, SpriteLayer::Emissive],
      animation: None,
    },
    lighting: LightingBand::Visible,
  };

  #[test]
  fn plan_preserves_order_and_duplicates_without_mutating_store() {
    let first = insertion(7, [1, 1], [32, 32]);
    let second = insertion(7, [2, 1], [64, 32]);
    let mut store = ParticleDecalStore::new(3);
    store.try_insert(first).unwrap();
    store.try_insert(first).unwrap();
    store.try_insert(second).unwrap();
    let before = store.clone();

    let plan = particle_decal_draw_plan(&store, PixelViewport::fit(2, 1, 64, 32), &[SPRITE]);

    assert_eq!(store, before);
    assert_eq!(plan.len(), 9);
    let groups = plan
      .chunks(3)
      .map(|group| (group[0].sprite_index, group[0].destination))
      .collect::<Vec<_>>();
    assert_eq!(
      groups[0],
      (
        0,
        PixelViewport::fit(2, 1, 64, 32)
          .tile_rect(Position::new(0, 0))
          .unwrap()
      )
    );
    assert_eq!(groups[1].1, groups[0].1);
    assert_eq!(groups[2].0, 2);
  }

  #[test]
  fn plan_omits_outside_and_unknown_requests() {
    let mut store = ParticleDecalStore::new(3);
    store.try_insert(insertion(7, [0, 1], [0, 32])).unwrap();
    store.try_insert(insertion(7, [3, 1], [96, 32])).unwrap();
    store.try_insert(insertion(9, [1, 1], [32, 32])).unwrap();

    let plan = particle_decal_draw_plan(&store, PixelViewport::fit(2, 1, 64, 32), &[SPRITE]);

    assert!(plan.is_empty());
  }

  #[test]
  fn plan_uses_stored_pixel_offset_for_subcell_destination() {
    let mut store = ParticleDecalStore::new(1);
    store.try_insert(insertion(7, [1, 1], [56, 32])).unwrap();

    let plan = particle_decal_draw_plan(&store, PixelViewport::fit(2, 1, 64, 32), &[SPRITE]);

    assert_eq!(plan[0].destination.x, 24);
    assert_eq!(plan[0].destination.y, 0);
  }

  #[test]
  fn plan_forwards_caller_resolved_lighting_band() {
    let mut store = ParticleDecalStore::new(1);
    store.try_insert(insertion(7, [1, 1], [32, 32])).unwrap();
    let explored = ParticleDecalSprite {
      lighting: LightingBand::Explored,
      ..SPRITE
    };

    let plan = particle_decal_draw_plan(&store, PixelViewport::fit(1, 1, 32, 32), &[explored]);

    assert!(
      plan
        .iter()
        .all(|draw| draw.lighting == LightingBand::Explored)
    );
  }
}

#[cfg(test)]
mod drl_render_test_support {
  use super::super::{ParticleDecalInsertion, ParticleDecalPlacement};

  pub(super) fn insertion(
    sprite_id: u32,
    cell: [i32; 2],
    pixel: [i32; 2],
  ) -> ParticleDecalInsertion {
    ParticleDecalInsertion {
      placement: ParticleDecalPlacement { cell, pixel },
      sprite_id,
    }
  }
}
