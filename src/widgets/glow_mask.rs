//! Text glow: the galley's coverage, Gaussian-blurred on the CPU and cached as a texture.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use egui::emath::GuiRounding as _;
use egui::epaint::Galley;
use egui::{
    Color32, ColorImage, Context, Id, Pos2, Rect, TextureHandle, TextureOptions, pos2, vec2,
};

use super::text_effects::Glow;

/// Gaussian σ per point of glow radius; puts the visible edge near `radius`.
const SIGMA_PER_RADIUS: f32 = 0.4;
/// Smallest σ, in texels, blurred after downsampling. Lower shows facets under linear upscale.
const MIN_TEXEL_SIGMA: f32 = 1.5;
const EVICT_AFTER_FRAMES: u64 = 10;

/// Paints `glow` under the galley at `pos`. One textured quad; the mask is cached by content.
pub(crate) fn paint(ui: &egui::Ui, pos: Pos2, galley: &Galley, glow: Glow) {
    let intensity = glow.intensity.clamp(0.0, 1.0);
    let radius_q = (glow.radius * 2.0).round();
    if intensity <= 0.0 || radius_q < 1.0 {
        return;
    }
    let ctx = ui.ctx();
    let ppp = galley.pixels_per_point;
    let key = (job_hash(galley), ppp.to_bits(), radius_q as u32);
    let frame = ctx.cumulative_frame_nr();

    let cached = with_cache(ctx, |c| {
        c.sweep(frame);
        c.entries.get_mut(&key).map(|e| {
            e.used = frame;
            e.clone()
        })
    });
    let entry = cached.unwrap_or_else(|| {
        let e = build(ctx, galley, radius_q * 0.5, frame);
        with_cache(ctx, |c| c.entries.insert(key, e.clone()));
        e
    });
    let Some(texture) = &entry.texture else {
        return;
    };

    let min = pos.round_to_pixels(ppp) + vec2(entry.origin[0] as f32, entry.origin[1] as f32) / ppp;
    let size = vec2(entry.size[0] as f32, entry.size[1] as f32) / ppp;
    ui.painter().image(
        texture.id(),
        Rect::from_min_size(min, size),
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        glow.color.gamma_multiply(intensity),
    );
}

#[derive(Clone)]
struct Entry {
    /// `None` when there is nothing to draw, so empty or oversized text isn't rebuilt each frame.
    texture: Option<TextureHandle>,
    /// Mask top-left relative to the galley, in pixels.
    origin: [i32; 2],
    size: [usize; 2],
    used: u64,
}

type Key = (u64, u32, u32);

#[derive(Clone, Default)]
struct GlowCache {
    entries: HashMap<Key, Entry>,
    swept: u64,
    /// egui only lends the atlas by clone, so keep one and refresh it when it goes stale.
    atlas: Option<Arc<ColorImage>>,
    atlas_fill: f32,
}

impl GlowCache {
    fn sweep(&mut self, frame: u64) {
        if self.swept != frame {
            self.swept = frame;
            self.entries
                .retain(|_, e| frame.saturating_sub(e.used) <= EVICT_AFTER_FRAMES);
        }
    }
}

fn with_cache<R>(ctx: &Context, f: impl FnOnce(&mut GlowCache) -> R) -> R {
    ctx.data_mut(|d| f(d.get_temp_mut_or_default(Id::unique("egui_styled::glow_cache"))))
}

fn job_hash(galley: &Galley) -> u64 {
    let mut h = DefaultHasher::new();
    galley.job.hash(&mut h);
    h.finish()
}

fn build(ctx: &Context, galley: &Galley, radius: f32, frame: u64) -> Entry {
    let sigma = radius * SIGMA_PER_RADIUS * galley.pixels_per_point;
    let k = ((sigma / MIN_TEXEL_SIGMA).floor() as usize).max(1);
    let boxes = box_radii(sigma / k as f32);
    let pad = (boxes.iter().sum::<usize>() + 1) * k;
    let empty = Entry {
        texture: None,
        origin: [0; 2],
        size: [0; 2],
        used: frame,
    };

    let (fill, atlas_size) = ctx.fonts(|f| (f.font_atlas_fill_ratio(), f.font_image_size()));
    let cached = with_cache(ctx, |c| {
        // A drop in fill ratio means the atlas was rebuilt and glyphs moved.
        c.atlas
            .clone()
            .filter(|a| a.size == atlas_size && fill >= c.atlas_fill)
    });
    let mut mask = cached.and_then(|a| build_mask(galley, &a, pad, k, true));
    if mask.is_none() {
        let fresh = Arc::new(ctx.fonts(|f| f.image()));
        mask = build_mask(galley, &fresh, pad, k, false);
        with_cache(ctx, |c| {
            c.atlas = Some(fresh);
            c.atlas_fill = fill;
        });
    }
    let Some(mask) = mask else {
        return empty;
    };
    let mut mask = downsample(mask, k);
    let max_side = ctx.input(|i| i.max_texture_side);
    if mask.w > max_side || mask.h > max_side {
        return empty;
    }

    blur(&mut mask, boxes);
    let peak = mask.data.iter().copied().fold(0.0, f32::max);
    if peak <= 0.0 {
        return empty;
    }
    let scale = 255.0 / peak;
    let pixels = mask
        .data
        .iter()
        .map(|&a| Color32::from_white_alpha((a * scale + 0.5) as u8))
        .collect();
    let image = ColorImage::new([mask.w, mask.h], pixels);
    Entry {
        texture: Some(ctx.load_texture("egui_styled::glow", image, TextureOptions::LINEAR)),
        origin: mask.origin,
        size: [mask.w * k, mask.h * k],
        used: frame,
    }
}

struct Mask {
    w: usize,
    h: usize,
    data: Vec<f32>,
    origin: [i32; 2],
}

struct Quad {
    dst: [i32; 2],
    src: [usize; 2],
    size: [usize; 2],
    /// Italic top-edge shift in pixels; the bottom edge is unshifted.
    shear: f32,
}

/// Copies glyph coverage out of `atlas`, padded by `pad` and sized to a multiple of `align`.
/// `None` for no glyphs, or when `strict` and a glyph is blank or out of bounds (stale atlas).
fn build_mask(
    galley: &Galley,
    atlas: &ColorImage,
    pad: usize,
    align: usize,
    strict: bool,
) -> Option<Mask> {
    let ppp = galley.pixels_per_point;
    let px = |v: f32| (v * ppp).round() as i32;
    let mut quads = Vec::new();
    for row in &galley.rows {
        let v = &row.row.visuals;
        // Glyph quads as epaint tessellates them: lt, rt, lb, rb; italics shift the top edge.
        for q in v.mesh.vertices[v.glyph_vertex_range.clone()].chunks_exact(4) {
            let (lt, lb, rb) = (q[0], q[2], q[3]);
            quads.push(Quad {
                dst: [px(row.pos.x + lb.pos.x), px(row.pos.y + lt.pos.y)],
                src: [lt.uv.x as usize, lt.uv.y as usize],
                size: [(rb.uv.x - lt.uv.x) as usize, (rb.uv.y - lt.uv.y) as usize],
                shear: (lt.pos.x - lb.pos.x) * ppp,
            });
        }
    }
    if quads.is_empty() {
        return None;
    }

    let mut lo = [i32::MAX; 2];
    let mut hi = [i32::MIN; 2];
    for q in &quads {
        lo[0] = lo[0].min(q.dst[0]);
        lo[1] = lo[1].min(q.dst[1]);
        hi[0] = hi[0].max(q.dst[0] + q.size[0] as i32 + q.shear.ceil() as i32);
        hi[1] = hi[1].max(q.dst[1] + q.size[1] as i32);
    }
    let pad_i = pad as i32;
    let origin = [lo[0] - pad_i, lo[1] - pad_i];
    let w = ((hi[0] - lo[0]) as usize + 2 * pad).next_multiple_of(align);
    let h = ((hi[1] - lo[1]) as usize + 2 * pad).next_multiple_of(align);
    let mut data = vec![0.0; w * h];

    let [aw, ah] = atlas.size;
    for q in &quads {
        if q.src[0] + q.size[0] > aw || q.src[1] + q.size[1] > ah {
            if strict {
                return None;
            }
            continue;
        }
        let mut any = false;
        for y in 0..q.size[1] {
            let shift = (q.shear * (1.0 - (y as f32 + 0.5) / q.size[1] as f32)).round() as i32;
            let dy = (q.dst[1] - origin[1]) as usize + y;
            let dx0 = (q.dst[0] - origin[0] + shift) as usize;
            let src_row = (q.src[1] + y) * aw + q.src[0];
            for x in 0..q.size[0] {
                let a = atlas.pixels[src_row + x].a();
                any |= a != 0;
                let d = &mut data[dy * w + dx0 + x];
                *d = f32::max(*d, a as f32 / 255.0);
            }
        }
        if strict && !any {
            return None;
        }
    }
    Some(Mask { w, h, data, origin })
}

/// Radii of three box blurs approximating a Gaussian of `sigma` (Kovesi, "boxes for gauss").
fn box_radii(sigma: f32) -> [usize; 3] {
    if sigma < 0.5 {
        return [0; 3];
    }
    let n = 3.0;
    let var = sigma * sigma;
    // Two odd widths bracketing the ideal; the first `m` boxes use the narrower one.
    let ideal = (12.0 * var / n + 1.0).sqrt();
    let mut narrow = ideal.floor() as usize;
    if narrow.is_multiple_of(2) {
        narrow -= 1;
    }
    let wide = narrow + 2;
    let nw = narrow as f32;
    let m = ((12.0 * var - n * nw * nw - 4.0 * n * nw - 3.0 * n) / (-4.0 * nw - 4.0)).round();
    std::array::from_fn(|i| {
        let width = if (i as f32) < m { narrow } else { wide };
        (width - 1) / 2
    })
}

/// Box-averages `k`×`k` blocks. `mask` dimensions must be multiples of `k`; origin is kept.
fn downsample(mask: Mask, k: usize) -> Mask {
    if k == 1 {
        return mask;
    }
    let (w, h) = (mask.w / k, mask.h / k);
    let mut data = vec![0.0; w * h];
    for y in 0..mask.h {
        let src = &mask.data[y * mask.w..(y + 1) * mask.w];
        let dst = &mut data[(y / k) * w..(y / k + 1) * w];
        for (d, block) in dst.iter_mut().zip(src.chunks_exact(k)) {
            *d += block.iter().sum::<f32>();
        }
    }
    let inv = 1.0 / (k * k) as f32;
    data.iter_mut().for_each(|d| *d *= inv);
    Mask {
        w,
        h,
        data,
        origin: mask.origin,
    }
}

/// Blurs columns, then transposes and blurs columns again (i.e. the rows).
/// Column passes read memory in order, which is what makes them fast.
fn blur(mask: &mut Mask, radii: [usize; 3]) {
    let mut tmp = vec![0.0; mask.data.len()];
    let (mut w, mut h) = (mask.w, mask.h);
    for _ in 0..2 {
        for r in radii.into_iter().filter(|&r| r > 0) {
            box_columns(&mask.data, &mut tmp, w, h, r);
            std::mem::swap(&mut mask.data, &mut tmp);
        }
        transpose(&mask.data, &mut tmp, w, h);
        std::mem::swap(&mut mask.data, &mut tmp);
        (w, h) = (h, w);
    }
}

/// `src` is `w` wide and `h` tall; `dst` becomes `h` wide and `w` tall.
fn transpose(src: &[f32], dst: &mut [f32], w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w {
            dst[x * h + y] = src[y * w + x];
        }
    }
}

/// Box blur of radius `r` down every column at once; zero beyond the ends.
/// `sums` holds each column's running window total as it slides down.
fn box_columns(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    let row = |y: usize| &src[y * w..(y + 1) * w];
    let scale = 1.0 / (2 * r + 1) as f32;
    let mut sums = vec![0.0f32; w];

    // Window for row 0 covers rows 0..=r.
    for y in 0..=r.min(h - 1) {
        add_row(&mut sums, row(y), 1.0);
    }
    for y in 0..h {
        for (out, sum) in dst[y * w..(y + 1) * w].iter_mut().zip(&sums) {
            *out = sum * scale;
        }
        // Slide the window down: row y+r+1 enters, row y-r leaves.
        if y + r + 1 < h {
            add_row(&mut sums, row(y + r + 1), 1.0);
        }
        if y >= r {
            add_row(&mut sums, row(y - r), -1.0);
        }
    }
}

fn add_row(sums: &mut [f32], row: &[f32], sign: f32) {
    for (sum, v) in sums.iter_mut().zip(row) {
        *sum += sign * v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use egui::epaint::text::{LayoutJob, TextFormat};
    use egui::{FontId, Shape, TextureId};

    fn mask(w: usize, h: usize) -> Mask {
        Mask {
            w,
            h,
            data: vec![0.0; w * h],
            origin: [0; 2],
        }
    }

    #[test]
    fn blur_preserves_mass_and_symmetry() {
        let mut m = mask(41, 41);
        m.data[20 * 41 + 20] = 1.0;
        blur(&mut m, box_radii(3.0));
        let sum: f32 = m.data.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, "sum = {sum}");
        for y in 0..41 {
            for x in 0..41 {
                let a = m.data[y * 41 + x];
                assert!((a - m.data[y * 41 + (40 - x)]).abs() < 1e-6);
                assert!((a - m.data[x * 41 + y]).abs() < 1e-6);
            }
        }
        // Support is the sum of the box radii.
        let reach: usize = box_radii(3.0).iter().sum();
        assert!(m.data[20 * 41 + 20 + reach] > 0.0);
        assert!(m.data[20 * 41 + 20 + reach + 1].abs() < 1e-6);
    }

    #[test]
    fn downsample_averages_blocks() {
        let mut m = mask(4, 2);
        m.data[..4].copy_from_slice(&[1.0, 1.0, 0.0, 1.0]);
        m.origin = [-3, 5];
        let d = downsample(m, 2);
        assert_eq!((d.w, d.h, d.origin), (2, 1, [-3, 5]));
        assert_eq!(d.data, vec![0.5, 0.25]);
    }

    #[test]
    fn box_radii_track_sigma() {
        assert_eq!(box_radii(0.0), [0; 3]);
        for sigma in [1.0f32, 2.5, 6.4, 12.8] {
            // Variance of a box of radius r is ((2r+1)² - 1) / 12.
            let var: f32 = box_radii(sigma)
                .iter()
                .map(|&r| (((2 * r + 1) * (2 * r + 1)) as f32 - 1.0) / 12.0)
                .sum();
            assert!(
                (var.sqrt() - sigma).abs() < 0.5,
                "sigma {sigma}: {}",
                var.sqrt()
            );
        }
    }

    fn galley(ctx: &egui::Context, italics: bool) -> Arc<Galley> {
        let mut job = LayoutJob::default();
        job.append(
            "Hil",
            0.0,
            TextFormat {
                font_id: FontId::proportional(20.0),
                italics,
                ..Default::default()
            },
        );
        ctx.fonts_mut(|f| f.layout_job(job))
    }

    #[test]
    fn mask_covers_glyphs_inside_padding() {
        let ctx = egui::Context::default();
        let _ = crate::test_util::run_ui(&ctx, Default::default(), |_| {
            let g = galley(&ctx, false);
            let m = ctx
                .fonts(|f| build_mask(&g, &f.image(), 4, 1, true))
                .expect("mask");
            assert!(m.data.iter().any(|&a| a > 0.5));
            // Padding is empty before blurring.
            for y in 0..m.h {
                for x in (0..4).chain(m.w - 4..m.w) {
                    assert_eq!(m.data[y * m.w + x], 0.0);
                }
            }
            let bounds = g.mesh_bounds;
            assert!((m.origin[0] + 4) as f32 >= bounds.min.x.floor() - 1.0);
            assert!((m.w - 8) as f32 <= bounds.width().ceil() + 1.0);
        });
    }

    #[test]
    fn italic_mask_is_sheared_wider() {
        let ctx = egui::Context::default();
        let _ = crate::test_util::run_ui(&ctx, Default::default(), |_| {
            let (a, b) = (galley(&ctx, false), galley(&ctx, true));
            let (a, b) = ctx.fonts(|f| {
                let img = f.image();
                (
                    build_mask(&a, &img, 0, 1, true).unwrap(),
                    build_mask(&b, &img, 0, 1, true).unwrap(),
                )
            });
            assert!(b.w > a.w);
        });
    }

    #[test]
    fn stale_atlas_is_rejected() {
        let ctx = egui::Context::default();
        let _ = crate::test_util::run_ui(&ctx, Default::default(), |_| {
            let g = galley(&ctx, false);
            let blank = ColorImage::new(
                ctx.fonts(|f| f.font_image_size()),
                vec![
                    Color32::TRANSPARENT;
                    ctx.fonts(|f| {
                        let [w, h] = f.font_image_size();
                        w * h
                    })
                ],
            );
            assert!(build_mask(&g, &blank, 2, 1, true).is_none());
        });
    }

    fn glow_label(ui: &mut egui::Ui, text: &str, radius: f32) {
        Styled::label(text)
            .glow(Color32::from_rgb(0, 220, 255), radius, 0.8)
            .show(ui);
    }

    fn entries(ctx: &egui::Context) -> Vec<(Key, Option<TextureId>)> {
        with_cache(ctx, |c| {
            c.entries
                .iter()
                .map(|(k, e)| (*k, e.texture.as_ref().map(|t| t.id())))
                .collect()
        })
    }

    #[test]
    fn glow_paints_one_textured_quad() {
        let ctx = egui::Context::default();
        let _ = crate::test_util::run_ui(&ctx, Default::default(), |ui| {
            glow_label(ui, "GLOW", 16.0);
            let (text, textured) = ctx.graphics(|g| {
                let shapes: Vec<_> = g.get(ui.layer_id()).unwrap().all_entries().collect();
                let text = shapes
                    .iter()
                    .filter(|s| matches!(s.shape, Shape::Text(_)))
                    .count();
                let textured = shapes
                    .iter()
                    .filter(|s| s.shape.texture_id() != TextureId::default())
                    .count();
                (text, textured)
            });
            assert_eq!(text, 1);
            assert_eq!(textured, 1);
        });
    }

    #[test]
    fn cache_reuses_then_evicts() {
        let ctx = egui::Context::default();
        let run = |f: &dyn Fn(&mut egui::Ui)| {
            let _ = crate::test_util::run_ui(&ctx, Default::default(), f);
        };
        run(&|ui| glow_label(ui, "A", 8.0));
        let first = entries(&ctx);
        assert_eq!(first.len(), 1);
        assert!(first[0].1.is_some());
        run(&|ui| glow_label(ui, "A", 8.0));
        assert_eq!(entries(&ctx), first);

        run(&|ui| glow_label(ui, "A", 9.0));
        assert_eq!(entries(&ctx).len(), 2);

        for _ in 0..=EVICT_AFTER_FRAMES {
            run(&|_| {});
        }
        run(&|ui| glow_label(ui, "B", 8.0));
        let left = entries(&ctx);
        assert_eq!(left.len(), 1);
        assert_ne!(left[0].0, first[0].0);
    }
}
