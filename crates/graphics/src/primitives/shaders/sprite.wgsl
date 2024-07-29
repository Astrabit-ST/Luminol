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
    opacity: f32,
    packed_rotation_and_hue: i32,
    flash_alpha: f32,
    packed_flash_color: u32,
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
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    let rotation = (graphic.packed_rotation_and_hue << 16) >> 16;
    var position_after_rotation: vec2<f32>;
    if rotation == 0 {
        position_after_rotation = model.position;
    } else {
        let r = radians(f32(rotation));
        let c = cos(r);
        let s = sin(r);
        position_after_rotation = mat2x2<f32>(c, -s, s, c) * model.position;
    }

    out.clip_position = vec4<f32>(Trans::translate_vertex(position_after_rotation, viewport, transform), 0., 1.);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    tex_sample.a *= graphic.opacity;
    if tex_sample.a <= 0.001 {
        discard;
    }

    tex_sample = Gamma::from_linear_rgba(tex_sample);

    let hue = graphic.packed_rotation_and_hue >> 16;
    if hue != 0 {
        var hsv = Hue::rgb_to_hsv(tex_sample.rgb);

        hsv.x += f32(hue) / 360.;
        tex_sample = vec4<f32>(Hue::hsv_to_rgb(hsv), tex_sample.a);
    }

    if graphic.flash_alpha > 0.001 {
        let flash_color = vec3<f32>(vec3<u32>(
            graphic.packed_flash_color & 0xff,
            (graphic.packed_flash_color >> 8) & 0xff,
            (graphic.packed_flash_color >> 16) & 0xff,
        )) / 255.;
        tex_sample = vec4<f32>(mix(tex_sample.rgb, flash_color, graphic.flash_alpha / 255.), tex_sample.a);
    }

    return tex_sample;
}
