#import luminol::gamma as Gamma
#import luminol::translation as Trans  // 🏳️‍⚧️
#import luminol::hue as Hue  // 🏳️‍⚧️

struct InstanceInput {
    @location(0) cell_id: u32,
    @builtin(instance_index) index: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var atlas: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;

struct Display {
    cells_width: u32,
    hue: f32,
    _padding: vec2<u32>,
}

@group(0) @binding(2)
var<uniform> viewport: Trans::Viewport;
@group(0) @binding(3)
var<uniform> transform: Trans::Transform;
@group(0) @binding(4)
var<uniform> display: Display;

const VERTEX_POSITIONS = array<vec2f, 6>(
    vec2f(0.0, 0.0),
    vec2f(192.0, 0.0),
    vec2f(0.0, 192.0),

    vec2f(192.0, 0.0),
    vec2f(0.0, 192.0),
    vec2f(192.0, 192.0),
);
const TEX_COORDS = array<vec2f, 6>(
    // slightly smaller than 192x192 to reduce bleeding from adjacent pixels in the atlas
    vec2f(0.01, 0.01),
    vec2f(191.99, 0.01),
    vec2f(0.01, 191.99),

    vec2f(191.99, 0.01),
    vec2f(0.01, 191.99),
    vec2f(191.99, 191.99),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let tile_position = vec2<f32>(
        f32(instance.index % display.cells_width),
        f32(instance.index / display.cells_width)
    );

    var vertex_positions = VERTEX_POSITIONS;
    let vertex_position = vertex_positions[vertex_index] + (tile_position * 192.0);
    let normalized_pos = Trans::translate_vertex(vertex_position, viewport, transform);

    out.clip_position = vec4<f32>(normalized_pos, 0.0, 1.0); // we don't set the z because we have no z buffer

    let atlas_tile_position = vec2<f32>(
        f32((instance.cell_id % #ANIMATION_COLUMNS + (instance.cell_id / #MAX_CELLS) * #ANIMATION_COLUMNS) * #CELL_SIZE),
        f32(instance.cell_id / #ANIMATION_COLUMNS % #MAX_ROWS * #CELL_SIZE)
    );

    let tex_size = vec2<f32>(textureDimensions(atlas));
    var vertex_tex_coords = TEX_COORDS;
    let vertex_tex_coord = vertex_tex_coords[vertex_index] /  tex_size;

    out.tex_coords = vertex_tex_coord + (atlas_tile_position / tex_size);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(atlas, atlas_sampler, input.tex_coords);

    if color.a <= 0.001 {
        discard;
    }

    if display.hue > 0.0 {
        var hsv = Hue::rgb_to_hsv(color.rgb);

        hsv.x += display.hue;
        color = vec4<f32>(Hue::hsv_to_rgb(hsv), color.a);
    }

    return Gamma::from_linear_rgba(color);
}
