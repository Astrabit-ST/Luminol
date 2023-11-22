struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) tile_position: vec3<f32>,
    @location(3) tile_id: i32,
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

#if USE_PUSH_CONSTANTS == true
struct PushConstants {
    viewport: Viewport,
    autotiles: Autotiles,
    opacity: f32,
}
var<push_constant> push_constants: PushConstants;
#else
@group(1) @binding(0)
var<uniform> viewport: Viewport;
@group(2) @binding(0)
var<uniform> autotiles: Autotiles;
@group(3) @binding(0)
var<uniform> opacity: array<vec4<f32>, 1>;
#endif

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    out.layer = instance.layer;

#if USE_PUSH_CONSTANTS == true
    let viewport = push_constants.viewport;
    let autotiles = push_constants.autotiles;
#endif

    if instance.tile_id < 48 {
        return out;
    }

    let position = viewport.proj * vec4<f32>(vertex.position.xy + (instance.tile_position.xy * 32.), 0.0, 1.0);
    out.clip_position = vec4<f32>(position.xy, instance.tile_position.z, 1.0);

    let is_autotile = instance.tile_id < 384;

    // 1712 is the number of non-autotile tiles that can fit under the autotiles without wrapping around
    let max_tiles_under_autotiles = i32(autotiles.max_frame_count) * 1712;
    let is_under_autotiles = !is_autotile && instance.tile_id - 384 < max_tiles_under_autotiles;

    var atlas_tile_position: vec2<f32>;
    if is_autotile {
        atlas_tile_position = vec2<f32>(
            // If the tile is an autotile
            f32((instance.tile_id - 48) % 8 * 32),
            f32((instance.tile_id - 48) / 8 * 32)
        );
    } else {
        if is_under_autotiles {
            atlas_tile_position = vec2<f32>(
            // If the tile is not an autotile but is located underneath the autotiles in the atlas
                f32((instance.tile_id % 8 + (instance.tile_id - 384) / 1712 * 8) * 32),
                f32(((instance.tile_id - 384) / 8 % 214 + 42) * 32)
            );
        } else {
            atlas_tile_position = vec2<f32>(
            // If the tile is not an autotile and is not located underneath the autotiles in the atlas
                f32((instance.tile_id % 8 + ((instance.tile_id - 384 - max_tiles_under_autotiles) / 2048 + i32(autotiles.max_frame_count)) * 8) * 32),
                f32((instance.tile_id - 384 - max_tiles_under_autotiles) / 8 % 256 * 32)
            );
        }
    }

    if is_autotile {
        let autotile_type = instance.tile_id / 48 - 1;
// we get an error about non constant indexing without this.
// not sure why
#if USE_PUSH_CONSTANTS == true
        let frame_count = push_constants.autotiles.frame_counts[autotile_type / 4][autotile_type % 4];
#else
        let frame_count = autotiles.frame_counts[autotile_type / 4][autotile_type % 4];
#endif

        let frame = autotiles.animation_index % frame_count;
        atlas_tile_position.x += f32(frame * 256u);
    }
    let tex_size = vec2<f32>(textureDimensions(atlas));
    out.tex_coords = vertex.tex_coords + (atlas_tile_position / tex_size);

    return out;
}

@group(0) @binding(0)
var atlas: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(atlas, atlas_sampler, input.tex_coords);

#if USE_PUSH_CONSTANTS == true
    let layer_opacity = push_constants.opacity;
#else
    let layer_opacity = opacity[input.layer / 4u][input.layer % 4u];
#endif
    color.a *= layer_opacity;

    if color.a <= 0.0 {
        discard;
    }

    return color;
}
