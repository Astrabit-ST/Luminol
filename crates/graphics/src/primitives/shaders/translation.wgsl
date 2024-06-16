#define_import_path luminol::translation

struct Transform {
  position: vec2f,
  scale: vec2f,
}

struct Viewport {
  viewport_size: vec2f, // size of the viewport in pixels
  viewport_translation: vec2f, // additional translation in pixels
  viewport_scale: vec2f, // additional scale in pixels
  _pad: vec2u // 16 byte alignment (webgl requires 16 byte alignment for uniform buffers)
}

fn translate_vertex(position: vec2f, viewport: Viewport, transform: Transform) -> vec2f {
  let position_vp = position * transform.scale + transform.position;
  let position_px = position_vp * viewport.viewport_scale + viewport.viewport_translation;
  let position_norm = position_px / viewport.viewport_size * 2.0 - 1.0;
  return vec2f(position_norm.x, -position_norm.y); // flip y-axis
}