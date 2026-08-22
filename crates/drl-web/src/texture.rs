//! WebGPU texture resources for the imported legacy layer sources.
//!
//! This module owns browser-only GPU objects. It deliberately accepts the
//! renderer-neutral `AtlasTextureSource` manifest and does not know about
//! simulation state or sprite blend equations.

use drl_assets::AtlasTextureSource;
use wasm_bindgen::JsValue;

/// One uploaded source and its view. The texture is retained so the view stays
/// valid for the renderer's lifetime.
struct UploadedTexture {
  source: AtlasTextureSource,
  _texture: wgpu::Texture,
  view: wgpu::TextureView,
}

/// Deterministic cache of decoded legacy sources uploaded to WebGPU.
pub(crate) struct GpuTextureCache {
  entries: Vec<UploadedTexture>,
}

impl GpuTextureCache {
  /// Decodes and uploads each unique source in manifest order.
  pub(crate) async fn load<I>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    sources: I,
  ) -> Result<Self, JsValue>
  where
    I: IntoIterator<Item = AtlasTextureSource>,
  {
    let mut entries: Vec<UploadedTexture> = Vec::new();
    for source in sources {
      if entries.iter().any(|entry| entry.source == source) {
        continue;
      }
      let image = crate::wasm::load_texture_source(source).await?;
      let size = wgpu::Extent3d {
        width: source.width,
        height: source.height,
        depth_or_array_layers: 1,
      };
      let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(source.path),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      });
      queue.copy_external_image_to_texture(
        &wgpu::CopyExternalImageSourceInfo {
          source: wgpu::ExternalImageSource::HTMLImageElement(image),
          origin: wgpu::Origin2d::ZERO,
          flip_y: false,
        },
        wgpu::CopyExternalImageDestInfo {
          texture: &texture,
          mip_level: 0,
          origin: wgpu::Origin3d::ZERO,
          aspect: wgpu::TextureAspect::All,
          color_space: wgpu::PredefinedColorSpace::Srgb,
          premultiplied_alpha: false,
        },
        size,
      );
      let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
      entries.push(UploadedTexture {
        source,
        _texture: texture,
        view,
      });
    }
    Ok(Self { entries })
  }

  /// Returns a view for one imported source, if it was loaded.
  pub(crate) fn view(&self, source: AtlasTextureSource) -> Option<&wgpu::TextureView> {
    self
      .entries
      .iter()
      .find(|entry| entry.source == source)
      .map(|entry| &entry.view)
  }

  /// Number of unique sources retained by the cache.
  pub(crate) const fn len(&self) -> usize {
    self.entries.len()
  }
}
