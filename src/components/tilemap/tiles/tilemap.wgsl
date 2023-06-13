struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) tile_postion: vec3<f32>,
    @location(3) tile_id: i32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    // todo: look into using multiple textures?
}

struct Viewport {
    proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> viewport: Viewport;

struct Autotiles {
    animation_index: u32,
    max_frame_count: u32,
    frame_counts: array<u32, 7>
}
@group(2) @binding(0)
var<storage, read> autotiles: Autotiles;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    var position = viewport.proj * vec4<f32>(vertex.position.xy + instance.tile_postion.xy, 0.0, 1.0);
    out.clip_position = vec4<f32>(position.xy, instance.tile_postion.z, 1.0);

    return out;
}

@group(0) @binding(0)
var atlas: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;


@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(atlas, atlas_sampler, input.tex_coords);

    if color.a <= 0.0 {
        discard;
    }

    return color;
}