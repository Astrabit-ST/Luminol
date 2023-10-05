@group(1) @binding(0)
var<uniform> viewport: Viewport;
@group(2) @binding(0)
var<storage, read> autotiles: Autotiles;
@group(3) @binding(0)
var<uniform> opacity: array<vec4<f32>, 1>;
