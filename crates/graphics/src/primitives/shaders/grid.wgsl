struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) tile_position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    // The fragment shader sees this as the position of the provoking vertex,
    // which is set to the vertex at the right angle of every triangle
    @location(1) @interpolate(flat) vertex_position: vec2<f32>,
}

struct Viewport {
    proj: mat4x4<f32>,
}

struct Display {
    viewport_size_in_pixels: vec2<f32>,
    pixels_per_point: f32,
    inner_thickness_in_points: f32,
}

@group(0) @binding(0)
var<uniform> viewport: Viewport;
@group(0) @binding(1)
var<uniform> display: Display;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    out.position = (viewport.proj * vec4<f32>((vertex.position + instance.tile_position) * 32., 0., 1.)).xy;
    out.vertex_position = out.position;
    out.clip_position = vec4<f32>(out.position, 0., 1.);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if display.viewport_size_in_pixels.x == 0. || display.viewport_size_in_pixels.y == 0. {
        discard;
    }

    var color: f32;
    var alpha: f32;

    let diff = abs(input.position - input.vertex_position) * (display.viewport_size_in_pixels / 2.);

    let adjusted_outer_thickness = 1.001 * display.pixels_per_point;
    let adjusted_inner_thickness = display.inner_thickness_in_points * adjusted_outer_thickness;

    if diff.x < adjusted_outer_thickness + adjusted_inner_thickness || diff.y < adjusted_outer_thickness + adjusted_inner_thickness {
        if diff.x < adjusted_inner_thickness || diff.y < adjusted_inner_thickness {
            color = 0.1;
        } else {
            color = 0.7;
        }
        alpha = 0.25;
    } else {
        color = 0.;
        alpha = 0.;
    }

    return vec4<f32>(color, color, color, alpha);
}
