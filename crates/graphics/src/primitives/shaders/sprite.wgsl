#import luminol::gamma as Gamma
#import luminol::hue as Hue
#import luminol::translation as Trans  // 🏳️‍⚧️

// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Graphic {
    hue: f32,
    opacity: f32,
    opacity_multiplier: f32,
    _padding: u32,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(0) @binding(2)
var<uniform> viewport: Trans::Viewport;
@group(0) @binding(3)
var<uniform> transform: Trans::Transform;
@group(0) @binding(4)
var<uniform> graphic: Graphic;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    out.clip_position = vec4<f32>(Trans::translate_vertex(model.position, viewport, transform), 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    tex_sample.a *= graphic.opacity * graphic.opacity_multiplier;
    if tex_sample.a <= 0.001 {
        discard;
    }

    if graphic.hue > 0.0 {
        var hsv = Hue::rgb_to_hsv(tex_sample.rgb);

        hsv.x += graphic.hue;
        tex_sample = vec4<f32>(Hue::hsv_to_rgb(hsv), tex_sample.a);
    }

    return Gamma::from_linear_rgba(tex_sample);
}
