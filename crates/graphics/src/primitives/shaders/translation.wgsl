#define_import_path luminol::translation

struct Transform {
  position: vec2f,
  scale: vec2f,
}

struct Viewport {
  screen_size: vec2f,
}

fn translate_vertex(position: vec2f, viewport: Viewport, transform: Transform) -> vec2f {
  let position_px = position * transform.scale + transform.position;
  let position_norm = position_px / viewport.screen_size * 2.0 - 1.0; // convert to normalized device coordinates
  return vec2f(position_norm.x, -position_norm.y); // flip y-axis
}