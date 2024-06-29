#import luminol::gamma as Gamma
#import luminol::translation as Trans  // 🏳️‍⚧️
#import luminol::hue as Hue  // 🏳️‍⚧️

struct InstanceInput {
    @location(0) tile_id: u32,
    @builtin(instance_index) index: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    // todo: look into using multiple textures?
}

struct Autotiles {
    frame_counts: array<vec4<u32>, 2>,
    animation_index: u32,
    max_frame_count: u32,
}

@group(0) @binding(0)
var atlas: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;

struct Display {
    opacity: f32,
    hue: f32,
    map_size: vec2<u32>,
}

@group(0) @binding(2)
var<uniform> viewport: Trans::Viewport;
@group(0) @binding(3)
var<uniform> transform: Trans::Transform;
@group(0) @binding(4)
var<uniform> autotiles: Autotiles;
@group(0) @binding(5)
var<uniform> display: Display;

const VERTEX_POSITIONS = array<vec2f, 6>(
    vec2f(0.0, 0.0),
    vec2f(32.0, 0.0),
    vec2f(0.0, 32.0),

    vec2f(32.0, 0.0),
    vec2f(0.0, 32.0),
    vec2f(32.0, 32.0),
);
const TEX_COORDS = array<vec2f, 6>(
    // slightly smaller than 32x32 to reduce bleeding from adjacent pixels in the atlas
    vec2f(0.01, 0.01),
    vec2f(31.99, 0.01),
    vec2f(0.01, 31.99),

    vec2f(31.99, 0.01),
    vec2f(0.01, 31.99),
    vec2f(31.99, 31.99),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    if instance.tile_id < #AUTOTILE_ID_AMOUNT {
        return out;
    }

    let layer_instance_index = instance.index % (display.map_size.x * display.map_size.y);
    let tile_position = vec2<f32>(
        f32(layer_instance_index % display.map_size.x), 
        f32(layer_instance_index / display.map_size.x)
    );

    var vertex_positions = VERTEX_POSITIONS;
    let vertex_position = vertex_positions[vertex_index] + (tile_position * 32.0);
    let normalized_pos = Trans::translate_vertex(vertex_position, viewport, transform);

    out.clip_position = vec4<f32>(normalized_pos, 0.0, 1.0); // we don't set the z because we have no z buffer

    let is_autotile = instance.tile_id < #TOTAL_AUTOTILE_ID_AMOUNT;

    let max_tiles_under_autotiles = autotiles.max_frame_count * #ROWS_UNDER_AUTOTILES_TIMES_COLUMNS;
    let is_under_autotiles = !is_autotile && instance.tile_id - #TOTAL_AUTOTILE_ID_AMOUNT < max_tiles_under_autotiles;

    var atlas_tile_position: vec2<f32>;
    if is_autotile {
        atlas_tile_position = vec2<f32>(
            // If the tile is an autotile
            f32((instance.tile_id - #AUTOTILE_ID_AMOUNT) % #AUTOTILE_FRAME_COLS * #TILE_SIZE),
            f32((instance.tile_id - #AUTOTILE_ID_AMOUNT) / #AUTOTILE_FRAME_COLS * #TILE_SIZE)
        );
    } else {
        if is_under_autotiles {
            atlas_tile_position = vec2<f32>(
            // If the tile is not an autotile but is located underneath the autotiles in the atlas
                f32((instance.tile_id % #TILESET_COLUMNS + (instance.tile_id - #TOTAL_AUTOTILE_ID_AMOUNT) / #ROWS_UNDER_AUTOTILES_TIMES_COLUMNS * #TILESET_COLUMNS) * #TILE_SIZE),
                f32(((instance.tile_id - #TOTAL_AUTOTILE_ID_AMOUNT) / #TILESET_COLUMNS % #ROWS_UNDER_AUTOTILES + #TOTAL_AUTOTILE_ROWS) * #TILE_SIZE)
            );
        } else {
            atlas_tile_position = vec2<f32>(
            // If the tile is not an autotile and is not located underneath the autotiles in the atlas
                f32((instance.tile_id % #TILESET_COLUMNS + ((instance.tile_id - #TOTAL_AUTOTILE_ID_AMOUNT - max_tiles_under_autotiles) / (#MAX_SIZE / #TILE_SIZE * #TILESET_COLUMNS) + autotiles.max_frame_count) * #TILESET_COLUMNS) * #TILE_SIZE),
                f32((instance.tile_id - #TOTAL_AUTOTILE_ID_AMOUNT - max_tiles_under_autotiles) / #TILESET_COLUMNS % (#MAX_SIZE / #TILE_SIZE) * #TILE_SIZE)
            );
        }
    }

    if is_autotile {
        let autotile_type = instance.tile_id / #AUTOTILE_ID_AMOUNT - 1;
        let frame_count = autotiles.frame_counts[autotile_type / 4][autotile_type % 4];

        let frame = autotiles.animation_index % frame_count;
        atlas_tile_position.x += f32(frame * #AUTOTILE_FRAME_WIDTH);
    }

    let tex_size = vec2<f32>(textureDimensions(atlas));
    var vertex_tex_coords = TEX_COORDS;
    let vertex_tex_coord = vertex_tex_coords[vertex_index] /  tex_size;

    out.tex_coords = vertex_tex_coord + (atlas_tile_position / tex_size);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(atlas, atlas_sampler, input.tex_coords);

    color.a *= display.opacity;

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
