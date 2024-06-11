#import luminol::gamma as Gamma

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) tile_id: u32,
    @builtin(instance_index) index: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    // todo: look into using multiple textures?
}

struct Viewport {
    proj: mat4x4<f32>,
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
    map_size: vec2<u32>,
}

@group(0) @binding(2)
var<uniform> viewport: Viewport;
@group(0) @binding(3)
var<uniform> autotiles: Autotiles;
@group(0) @binding(4)
var<uniform> display: Display;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    if instance.tile_id < #AUTOTILE_ID_AMOUNT {
        return out;
    }

    let layer_instance_index = instance.index % (display.map_size.x * display.map_size.y);
    let tile_position = vec2<f32>(
        f32(layer_instance_index % display.map_size.x), 
        f32(layer_instance_index / display.map_size.x)
    );

    let position = viewport.proj * vec4<f32>(vertex.position.xy + (tile_position * 32.), 0.0, 1.0);
    out.clip_position = vec4<f32>(position.xy, 0.0, 1.0); // we don't set the z because we have no z buffer

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
// we get an error about non constant indexing without this.
// not sure why
        let frame_count = autotiles.frame_counts[autotile_type / 4][autotile_type % 4];

        let frame = autotiles.animation_index % frame_count;
        atlas_tile_position.x += f32(frame * #AUTOTILE_FRAME_WIDTH);
    }
    let tex_size = vec2<f32>(textureDimensions(atlas));
    out.tex_coords = vertex.tex_coords + (atlas_tile_position / tex_size);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(atlas, atlas_sampler, input.tex_coords);

    color.a *= display.opacity;

    if color.a <= 0.0 {
        discard;
    }

    return Gamma::from_linear_rgba(color);
}
