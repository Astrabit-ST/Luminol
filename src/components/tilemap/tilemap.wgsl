// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Uniform {
    pan: vec2<f32>,
    scale: f32,
}
@group(1) @binding(0) // 1.
var<uniform> uniform_data: Uniform;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    var position = model.position;
    position *= (uniform_data.scale / 300.);
    position.x -= 1.;
    position.y -= 1.;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if sample.a <= 0. {
        discard;
    }
    return sample;
}