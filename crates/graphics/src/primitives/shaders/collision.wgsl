#import luminol::translation as Trans // 🏳️‍⚧️

struct InstanceInput {
    @location(0) tile_position: vec2<f32>,
    @location(1) passage: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> viewport: Trans::Viewport;
@group(0) @binding(1)
var<uniform> transform: Trans::Transform;

const VERTEX_POSITIONS = array<vec2f, 12>(
    vec2f(16., 16.),
    vec2f(0., 32.),
    vec2f(32., 32.),

    vec2f(16., 16.),
    vec2f(0., 0.),
    vec2f(0., 32.),

    vec2f(16., 16.),
    vec2f(32., 32.),
    vec2f(32., 0.),

    vec2f(16., 16.),
    vec2f(32., 0.),
    vec2f(0., 0.),
);

const VERTEX_DIRECTIONS = array<u32, 4>(
    1,
    2,
    4,
    8,
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    var vertex_directions = VERTEX_DIRECTIONS;
    let vertex_direction = vertex_directions[vertex_index / 3];

    if (instance.passage & vertex_direction) == 0u {
        return out;
    }

    var vertex_positions = VERTEX_POSITIONS;
    let vertex_position = vertex_positions[vertex_index] + (instance.tile_position * 32.);
    let normalized_pos = Trans::translate_vertex(vertex_position, viewport, transform);

    out.clip_position = vec4<f32>(normalized_pos, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1., 0., 0., 0.4);
}
