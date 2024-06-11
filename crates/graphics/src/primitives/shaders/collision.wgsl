struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) direction: u32,
}

struct InstanceInput {
    @location(2) tile_position: vec2<f32>,
    @location(3) passage: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct Viewport {
    proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> viewport: Viewport;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    if (instance.passage & vertex.direction) == 0u {
        return out;
    }

    let position = viewport.proj * vec4<f32>(vertex.position.xy + (instance.tile_position.xy * 32.), 0.0, 1.0);
    out.clip_position = vec4<f32>(position.xy, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1., 0., 0., 0.4);
}
