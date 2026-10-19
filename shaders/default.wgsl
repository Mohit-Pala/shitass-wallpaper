struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    _pad: f32,
    mouse: vec2<f32>,
    _pad2: vec2<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // fullscreen triangle
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = frag.xy / u.resolution;
    let m = u.mouse / u.resolution;
    let d = distance(uv, m);
    let wave = 0.5 + 0.5 * sin(u.time + uv.xyx * 6.0 + vec3<f32>(0.0, 2.0, 4.0));
    let glow = smoothstep(0.25, 0.0, d);
    return vec4<f32>(wave * 0.6 + vec3<f32>(glow), 1.0);
}
