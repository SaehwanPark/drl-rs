//! Renderer-neutral compositing for optional sprite outline masks.

/// Composites an outline mask behind a base sprite using straight-alpha color.
///
/// The mask is intentionally treated as a background layer: opaque base pixels
/// remain unchanged, while transparent or partially transparent base pixels can
/// reveal the optional outline. Lighting is applied to the resolved RGB value;
/// callers still own texture sampling and GPU blending.
#[must_use]
pub fn outline_mask_composite(
  base_rgba: [f32; 4],
  outline_rgba: [f32; 4],
  lighting: f32,
) -> [f32; 4] {
  let base_alpha = base_rgba[3].clamp(0.0, 1.0);
  let outline_alpha = outline_rgba[3].clamp(0.0, 1.0) * (1.0 - base_alpha);
  let output_alpha = (base_alpha + outline_alpha).clamp(0.0, 1.0);
  if output_alpha <= f32::EPSILON {
    return [0.0; 4];
  }

  let base_weight = base_alpha / output_alpha;
  let outline_weight = outline_alpha / output_alpha;
  [
    (base_rgba[0] * base_weight + outline_rgba[0] * outline_weight) * lighting,
    (base_rgba[1] * base_weight + outline_rgba[1] * outline_weight) * lighting,
    (base_rgba[2] * base_weight + outline_rgba[2] * outline_weight) * lighting,
    output_alpha,
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn transparent_base_reveals_lit_outline() {
    assert_eq!(
      outline_mask_composite([0.0, 0.0, 0.0, 0.0], [1.0, 0.25, 0.0, 0.75], 0.5),
      [0.5, 0.125, 0.0, 0.75]
    );
  }

  #[test]
  fn opaque_base_masks_outline_without_changing_base() {
    assert_eq!(
      outline_mask_composite([0.2, 0.4, 0.6, 1.0], [1.0, 0.0, 0.0, 1.0], 0.75),
      [0.15, 0.3, 0.45000002, 1.0]
    );
  }

  #[test]
  fn partial_base_uses_straight_alpha_background_weight() {
    let result = outline_mask_composite([0.8, 0.4, 0.0, 0.5], [0.0, 0.0, 1.0, 0.5], 1.0);
    assert!((result[0] - 0.53333336).abs() < f32::EPSILON);
    assert!((result[1] - 0.26666668).abs() < f32::EPSILON);
    assert!((result[2] - 0.33333334).abs() < f32::EPSILON);
    assert!((result[3] - 0.75).abs() < f32::EPSILON);
  }

  #[test]
  fn empty_layers_return_transparent_black() {
    assert_eq!(
      outline_mask_composite([0.3, 0.2, 0.1, 0.0], [1.0, 1.0, 1.0, 0.0], 1.0),
      [0.0; 4]
    );
  }
}
