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
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0) // 1.
var<uniform> viewport: Viewport;

struct Graphic {
    hue: f32,
    opacity: f32
}

@group(2) @binding(0)
var<uniform> graphic: Graphic;

fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    let q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    let d = q.x - min(q.w, q.y);

    // Avoid divide - by - zero situations by adding a very tiny delta.
	// Since we always deal with underlying 8 - Bit color values, this 
    // should never mask a real value 
    let eps = 1.0e-10;

    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + eps)), d / (q.x + eps), q.x);
}

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);

    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    var position = viewport.proj * vec4<f32>(model.position.xy, 0.0, 1.0);

    out.clip_position = vec4<f32>(position.xy, model.position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    tex_sample.a *= graphic.opacity;
    if tex_sample.a <= 0. {
        discard;
    }

    if graphic.hue > 0.0 {
        var hsv = rgb_to_hsv(tex_sample.rgb);

        hsv.x += graphic.hue;
        tex_sample = vec4<f32>(hsv_to_rgb(hsv), tex_sample.a);
    }

    return tex_sample;
}