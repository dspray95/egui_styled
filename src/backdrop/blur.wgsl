
// Dual Kawase (Bjørge, SIGGRAPH 2015). `half_texel` is for the source level.
struct Blur { half_texel: vec2<f32>, offset: f32, _pad: f32 };
@group(0) @binding(2) var<uniform> u: Blur;

fn tap(uv: vec2<f32>) -> vec4<f32> { return textureSample(src, samp, uv); }

@fragment fn fs_down(in: VsOut) -> @location(0) vec4<f32> {
    let h = u.half_texel * u.offset;
    var s = tap(in.uv) * 4.0;
    s += tap(in.uv - h);
    s += tap(in.uv + h);
    s += tap(in.uv + vec2<f32>(h.x, -h.y));
    s += tap(in.uv - vec2<f32>(h.x, -h.y));
    return s / 8.0;
}

@fragment fn fs_up(in: VsOut) -> @location(0) vec4<f32> {
    let h = u.half_texel * u.offset;
    var s = tap(in.uv + vec2<f32>(-h.x * 2.0, 0.0));
    s += tap(in.uv + vec2<f32>(-h.x, h.y)) * 2.0;
    s += tap(in.uv + vec2<f32>(0.0, h.y * 2.0));
    s += tap(in.uv + vec2<f32>(h.x, h.y)) * 2.0;
    s += tap(in.uv + vec2<f32>(h.x * 2.0, 0.0));
    s += tap(in.uv + vec2<f32>(h.x, -h.y)) * 2.0;
    s += tap(in.uv + vec2<f32>(0.0, -h.y * 2.0));
    s += tap(in.uv + vec2<f32>(-h.x, -h.y)) * 2.0;
    return s / 12.0;
}
