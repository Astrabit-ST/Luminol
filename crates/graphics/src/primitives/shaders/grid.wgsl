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
    // we need this size(16) because in webgl, buffers cannot have a size that is not a multiple of 16.
    @size(16) map_size: vec2<u32>,
}

@group(0) @binding(0)
var<uniform> viewport: Viewport;
@group(0) @binding(1)
var<uniform> display: Display;

// OpenGL and WebGL use the last vertex in each triangle as the provoking vertex, and
// Direct3D, Metal, Vulkan and WebGPU use the first vertex in each triangle
#ifdef LUMINOL_BACKEND_GL
const QUAD_VERTICES: array<vec2f, 6> = array<vec2f, 6>(
    vec2f(1., 0.),
    vec2f(0., 1.),
    vec2f(0., 0.), // Provoking vertex
    
    vec2f(0., 1.),
    vec2f(1., 0.),
    vec2f(1., 1.), // Provoking vertex
);
#else
const QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0., 0.),
    vec2<f32>(1., 0.),
    vec2<f32>(0., 1.), // Provoking vertex
    
    vec2<f32>(1., 1.),
    vec2<f32>(0., 1.),
    vec2<f32>(1., 0.), // Provoking vertex
);
#endif

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;

    var quad_vertices = QUAD_VERTICES;
    let vertex_position = quad_vertices[vertex_index];
    let tile_position = vec2<f32>(
        f32(instance_index % display.map_size.x), 
        f32(instance_index / display.map_size.x)
    );

    out.position = (viewport.proj * vec4<f32>((vertex_position + tile_position) * 32., 0., 1.)).xy;
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
