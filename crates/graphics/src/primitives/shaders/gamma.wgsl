#define_import_path luminol::gamma

// 0-1 sRGB gamma  from  0-1 linear
fn from_linear_rgb(rgb: vec3<f32>) -> vec3<f32> {
    let cutoff = rgb < vec3<f32>(0.0031308);
    let lower = rgb * vec3<f32>(12.92);
    let higher = vec3<f32>(1.055) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(higher, lower, cutoff);
}

// 0-1 sRGBA gamma  from  0-1 linear
fn from_linear_rgba(linear_rgba: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(from_linear_rgb(linear_rgba.rgb), linear_rgba.a);
}