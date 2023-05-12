// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) frame: u32,
}

struct Viewport {
    // Projection matrix
    proj: mat4x4<f32>,
    // Pan
    // pan: vec2<f32>,
    scale: f32,
}

struct Layers {
    enabled_layers: array<bool>,
}

struct Autotiles {
    frame_counts: array<u32, 7>,
    autotile_region_width: u32,
    ani_frame: u32,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@group(1) @binding(0) // 1.
var<uniform> viewport: Viewport;
@group(1) @binding(1)
var<storage, read> autotiles: Autotiles;

const AUTOTILE_WIDTH = 96.;
const AUTOTILE_HEIGHT = 128.;
const TOTAL_AUTOTILE_HEIGHT = 896.;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    let dimensions = vec2<f32>(textureDimensions(t_diffuse));
    var pix_tex_coords = vec2<f32>(textureDimensions(t_diffuse)) * model.tex_coords;
    if pix_tex_coords.y < TOTAL_AUTOTILE_HEIGHT && pix_tex_coords.x < f32(autotiles.autotile_region_width) {
        let autotile_id = u32(pix_tex_coords.y / AUTOTILE_HEIGHT);
        let frame_count = autotiles.frame_counts[autotile_id];
        let frame = autotiles.ani_frame % frame_count;

        out.frame = frame;
        out.tex_coords.x += (f32(frame) * AUTOTILE_WIDTH) / dimensions.x;
    }

    // var model_position = (model.position.xy + viewport.pan) / (viewport.scale / 100.);
    var position = viewport.proj * vec4<f32>(model.position.xy * (viewport.scale / 100.), 0.0, 1.0);

    out.clip_position = vec4<f32>(position.xy, model.position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    //  var sample = vec4<f32>(in.tex_coords.xy, f32(in.frame) / 10., tex_sample.w);

    if tex_sample.a <= 0. {
        discard;
    }
    return tex_sample;
}