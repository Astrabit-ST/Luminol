struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) tile_position: vec3<f32>,
    @location(3) tile_id: u32,
    @location(4) layer: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) layer: u32,
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

@group(0) @binding(2)
var<uniform> viewport: Viewport;
@group(0) @binding(3)
var<uniform> autotiles: Autotiles;
@group(0) @binding(4)
var<uniform> opacity: array<vec4<f32>, 1>;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    out.layer = instance.layer;

    if instance.tile_id < #AUTOTILE_ID_AMOUNT {
        return out;
    }

    let position = viewport.proj * vec4<f32>(vertex.position.xy + (instance.tile_position.xy * f32(#TILE_SIZE)), 0.0, 1.0);
    out.clip_position = vec4<f32>(position.xy, instance.tile_position.z, 1.0);

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

// 0-1 sRGB gamma  from  0-1 linear
fn gamma_from_linear_rgb(rgb: vec3<f32>) -> vec3<f32> {
    let cutoff = rgb < vec3<f32>(0.0031308);
    let lower = rgb * vec3<f32>(12.92);
    let higher = vec3<f32>(1.055) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(higher, lower, cutoff);
}

// 0-1 sRGBA gamma  from  0-1 linear
fn gamma_from_linear_rgba(linear_rgba: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(gamma_from_linear_rgb(linear_rgba.rgb), linear_rgba.a);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(atlas, atlas_sampler, input.tex_coords);

    let layer_opacity = opacity[input.layer / 4u][input.layer % 4u];
    color.a *= layer_opacity;

    if color.a <= 0.0 {
        discard;
    }

    return gamma_from_linear_rgba(color);
}
