//! Same-origin atlas layer loading at the browser boundary.

use super::*;

/// Loads and decodes one same-origin imported atlas layer.
///
/// The returned DOM image is ready for a future WebGPU upload. Dimensions
/// are checked against the pinned manifest before the image crosses the
/// renderer boundary.
pub async fn load_texture_source(source: AtlasTextureSource) -> Result<HtmlImageElement, JsValue> {
  let image = HtmlImageElement::new()?;
  let url = texture_source_url(source).map_err(|error| JsValue::from_str(&error.to_string()))?;
  image.set_src(&url);
  JsFuture::from(image.decode()).await?;
  validate_texture_source_dimensions(source, image.natural_width(), image.natural_height())
    .map_err(|error| JsValue::from_str(&error.to_string()))?;
  // WebGPU's external-image source reports the element's pixel dimensions;
  // pin them to the validated manifest before issuing the copy.
  image.set_width(source.width);
  image.set_height(source.height);
  Ok(image)
}
