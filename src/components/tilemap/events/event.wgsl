// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Viewport {
    // Projection matrix
    proj: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@group(1) @binding(0) // 1.
var<uniform> viewport: Viewport;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    var position = viewport.proj * vec4<f32>(model.position.xy * (viewport.scale / 100.), 0.0, 1.0);

    out.clip_position = vec4<f32>(position.xy, model.position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if tex_sample.a <= 0. {
        discard;
    }
    return tex_sample;
}