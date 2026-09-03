//! Atlas manifest, asset URL, and textured-pipeline contracts.

use super::*;

#[test]
fn texture_source_urls_are_same_origin_and_dimensions_are_checked() {
  let source = drl_assets::AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Base);
  assert_eq!(
    texture_source_url(source).expect("manifest path"),
    "assets/legacy/drl/graphics/enemies.png"
  );
  assert_eq!(
    browser_asset_url("dguy.png").expect("safe path"),
    "assets/legacy/drl/graphics/dguy.png"
  );
  for path in [
    "/dguy.png",
    "../dguy.png",
    "foo/../bar.png",
    "dguy.png?x=1",
    "dguy.png#x",
    r"..\dguy.png",
  ] {
    assert!(browser_asset_url(path).is_err(), "{path}");
  }
  assert!(validate_texture_source_dimensions(source, 512, 192).is_ok());
  let error = validate_texture_source_dimensions(source, 256, 192).unwrap_err();
  assert_eq!(error.path, "enemies.png");
  assert_eq!(error.expected, (512, 192));
  assert_eq!(error.actual, (256, 192));
  assert!(error.to_string().contains("expected 512x192"));
}

#[test]
fn texture_source_manifest_is_stable_and_deduplicated() {
  let sources = texture_source_manifest();
  assert_eq!(sources.len(), 24);
  assert_eq!(sources.first().expect("base source").path, "dguy.png");
  assert_eq!(sources.last().expect("last source").path, "fx_emissive.png");
  assert!(sources.windows(2).all(|window| window[0] != window[1]));
  assert_eq!(
    sources
      .iter()
      .filter(|source| source.path == "levels.png")
      .count(),
    1
  );
}

#[test]
fn base_texture_uvs_preserve_top_left_orientation() {
  let uv = SpriteUv {
    u_min: 0.1,
    v_min: 0.2,
    u_max: 0.3,
    v_max: 0.4,
  };
  assert_eq!(
    base_texture_uvs(uv),
    [
      [0.1, 0.4],
      [0.3, 0.4],
      [0.3, 0.2],
      [0.1, 0.4],
      [0.3, 0.2],
      [0.1, 0.2],
    ]
  );
}

#[test]
fn base_texture_lighting_factor_matches_fair_bands() {
  assert_eq!(base_texture_lighting_factor(LightingBand::Visible), 1.0);
  assert_eq!(base_texture_lighting_factor(LightingBand::Explored), 0.45);
}

#[test]
fn emissive_role_raises_but_never_reduces_fair_light() {
  assert_eq!(emissive_lighting_floor(0.45, 0.8), 0.8);
  assert_eq!(emissive_lighting_floor(1.0, 0.8), 1.0);
}

#[test]
fn emissive_role_pairing_uses_registered_atlas_source() {
  let base = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Base);
  let emissive = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Emissive);
  assert_eq!(base.path, "enemies.png");
  assert_eq!(emissive.path, "enemies_emissive.png");
  assert_eq!((base.width, base.height), (emissive.width, emissive.height));
}

#[test]
fn outline_role_registration_preserves_optional_atlas_boundary() {
  assert!(
    AtlasId::Enemies
      .layers()
      .contains(&drl_assets::SpriteLayer::Shadow)
  );
  assert!(
    !AtlasId::Levels
      .layers()
      .contains(&drl_assets::SpriteLayer::Shadow)
  );
  let source = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Shadow);
  assert_eq!(source.path, "enemies_shadow.png");
  assert_eq!((source.width, source.height), (512, 192));
}

#[test]
fn textured_alpha_cutoff_matches_legacy_boundary() {
  assert!(!retains_textured_fragment(0.0));
  assert!(!retains_textured_fragment(0.099));
  assert!(retains_textured_fragment(0.1));
  assert!(retains_textured_fragment(1.0));
}

#[test]
fn textured_shader_contract_keeps_verified_compositing_terms() {
  assert!(BASE_TEXTURE_SHADER.contains("textureSample(base_texture"));
  assert!(BASE_TEXTURE_SHADER.contains("textureSample(emissive_texture"));
  assert!(BASE_TEXTURE_SHADER.contains("textureSample(mask_texture"));
  assert!(BASE_TEXTURE_SHADER.contains("outline_texture: texture_2d<f32>"));
  assert!(BASE_TEXTURE_SHADER.contains("textureSample(outline_texture"));
  assert!(BASE_TEXTURE_SHADER.contains("mask.rgb * input.colorization.rgb"));
  assert!(BASE_TEXTURE_SHADER.contains("output.colorization = input.colorization"));
  assert!(BASE_TEXTURE_SHADER.contains("max(input.lighting.rgb"));
  assert!(BASE_TEXTURE_SHADER.contains("outline.a * (1.0 - sampled.a)"));
  assert!(BASE_TEXTURE_SHADER.contains("colorized * sampled.a + outline.rgb * outline_alpha"));
  assert!(BASE_TEXTURE_SHADER.contains("output_rgb * lighting, output_alpha"));
  assert!(BASE_TEXTURE_SHADER.contains("if (output.a < 0.1)"));
  assert!(BASE_TEXTURE_SHADER.contains("return output;"));
}

#[test]
fn base_texture_ndc_rect_preserves_destination_orientation() {
  let rect = PixelRect {
    x: 10,
    y: 20,
    width: 30,
    height: 40,
  };
  let [left, bottom, right, top] = base_texture_ndc_rect(rect, 100, 100);
  assert!((left + 0.8).abs() < f32::EPSILON);
  assert!((bottom + 0.2).abs() < f32::EPSILON);
  assert!((right + 0.2).abs() < f32::EPSILON);
  assert!((top - 0.6).abs() < f32::EPSILON);
}
