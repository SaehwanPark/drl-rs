//! Stable browser asset identities: the registered atlas manifest, subpath-safe
//! same-origin URLs, and decoded-dimension validation.

use drl_assets::{AtlasId, AtlasTextureSource};

/// Static bundle root used by the browser texture loader.
pub const GRAPHICS_ASSET_ROOT: &str = "assets/legacy/drl/graphics/";

pub(crate) const REGISTERED_ATLASES: [AtlasId; 7] = [
  AtlasId::Dguy,
  AtlasId::Enemies,
  AtlasId::EnemiesBig,
  AtlasId::GunsAndPickups,
  AtlasId::Levels,
  AtlasId::DoorsAndDecorations,
  AtlasId::Fx,
];

/// Returns every unique imported layer source in stable atlas registration
/// order. A browser uploader can use this manifest without inspecting scenes.
#[must_use]
pub fn texture_source_manifest() -> Vec<AtlasTextureSource> {
  let mut sources = Vec::new();
  for atlas in REGISTERED_ATLASES {
    for &layer in atlas.layers() {
      let source = atlas.texture_source(layer);
      if !sources.contains(&source) {
        sources.push(source);
      }
    }
  }
  sources
}

/// A rejected browser asset path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureSourcePathError {
  pub path: String,
}

impl std::fmt::Display for TextureSourcePathError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "invalid texture source path: {}", self.path)
  }
}

/// Validates a relative imported-asset basename for subpath-safe loading.
pub fn browser_asset_url(path: &str) -> Result<String, TextureSourcePathError> {
  let valid = !path.is_empty()
    && !path.starts_with('/')
    && !path.contains("..")
    && !path.contains(['\\', '/', '?', '#'])
    && !path.contains("://")
    && path
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
  if !valid {
    return Err(TextureSourcePathError {
      path: path.to_string(),
    });
  }
  Ok(format!("{GRAPHICS_ASSET_ROOT}{path}"))
}

/// Returns the same-origin URL for an imported atlas layer.
pub fn texture_source_url(source: AtlasTextureSource) -> Result<String, TextureSourcePathError> {
  browser_asset_url(source.path)
}

/// A decoded-source dimension mismatch at the browser asset boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureSourceDimensionsError {
  pub path: &'static str,
  pub expected: (u32, u32),
  pub actual: (u32, u32),
}

impl std::fmt::Display for TextureSourceDimensionsError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      formatter,
      "texture {} has dimensions {}x{}, expected {}x{}",
      self.path, self.actual.0, self.actual.1, self.expected.0, self.expected.1
    )
  }
}

/// Validates decoded image dimensions against the pinned asset metadata.
pub fn validate_texture_source_dimensions(
  source: AtlasTextureSource,
  actual_width: u32,
  actual_height: u32,
) -> Result<(), TextureSourceDimensionsError> {
  let expected = (source.width, source.height);
  if expected == (actual_width, actual_height) {
    Ok(())
  } else {
    Err(TextureSourceDimensionsError {
      path: source.path,
      expected,
      actual: (actual_width, actual_height),
    })
  }
}
