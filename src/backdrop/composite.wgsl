
// All in framebuffer pixels. `radii` is nw, ne, se, sw; `region` is the blurred area (x, y, w, h).
struct Composite { rect: vec4<f32>, radii: vec4<f32>, region: vec4<f32>, opacity: f32 };
@group(0) @binding(2) var<uniform> c: Composite;

// `pos` is in framebuffer pixels even though the viewport is the callback rect.
@fragment fn fs_composite(in: VsOut) -> @location(0) vec4<f32> {
    let p = in.pos.xy;
    let color = textureSample(src, samp, (p - c.region.xy) / c.region.zw);

    let half_size = (c.rect.zw - c.rect.xy) * 0.5;
    let d = p - (c.rect.xy + c.rect.zw) * 0.5;
    let side = select(c.radii.xw, c.radii.yz, d.x > 0.0);
    let r = min(select(side.x, side.y, d.y > 0.0), min(half_size.x, half_size.y));
    let q = abs(d) - half_size + vec2<f32>(r);
    let dist = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
    let a = clamp(0.5 - dist, 0.0, 1.0) * c.opacity;
    return vec4<f32>(color.rgb * a, a);
}
