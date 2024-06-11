#define_import_path luminol::hue

fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    let q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    let d = q.x - min(q.w, q.y);

    // Avoid divide - by - zero situations by adding a very tiny delta.
    // Since we always deal with underlying 8 - Bit color values, this
    // should never mask a real value
    let eps = 1.0e-10;

    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + eps)), d / (q.x + eps), q.x);
}

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);

    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}