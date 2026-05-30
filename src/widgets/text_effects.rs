use std::sync::Arc;

use egui::{
    Align2, Color32, Pos2, Shape, Vec2,
    emath::TSTransform,
    epaint::{Galley, TextShape},
};

/// A single glyph-shaped shadow stamp: the galley is repainted at `offset`
/// from its natural position in the given `color`.
///
/// Multiple shadows compose — append with repeated
/// [`StyledLabel::text_shadow`](super::label::StyledLabel::text_shadow) calls.
/// Chromatic-aberration split = two shadows with opposite horizontal offsets
/// and complementary colors:
///
/// ```ignore
/// Styled::label("[ENTER]")
///     .text_shadow(vec2(-2.0, 0.0), cyan)
///     .text_shadow(vec2( 2.0, 0.0), magenta)
///     .text_color(Color32::WHITE)
/// ```
#[derive(Clone, Copy, Debug)]
pub struct TextShadow {
    pub offset: Vec2,
    pub color: Color32,
}

/// Faux glow that follows the glyphs: a sunflower disk of overlapping galley
/// stamps, brightest on the letterforms and fading out to `radius`.
///
/// egui has no blur pass, so glyph-shaped glow is approximated by stamping the
/// text many times at faint, aperiodically-distributed offsets. With enough
/// overlap the copies blend into a continuous halo rather than visible ghosts.
///
/// `intensity` is a plain 0.0..=1.0 scalar supplied per-frame by the consumer.
/// When you animate it (e.g. a sine wave), keep all timing/state on the consumer
/// side — this struct holds only the current-frame appearance value.
#[derive(Clone, Copy, Debug)]
pub struct Glow {
    pub color: Color32,
    /// Blur radius in logical pixels. Use a theme glow token or an animated value.
    pub radius: f32,
    /// 0.0 = invisible, 1.0 = full-strength glow.
    pub intensity: f32,
}

/// Quality knob for [`Glow`] rendering.
///
/// Glow is a sunflower (Vogel) disk of glyph stamps with no grid or spokes, so
/// more stamps just makes the halo smoother. `samples` is a *base density*
/// (the stamp count at an 8px radius); the real count scales with radius² so a
/// large glow stays as smooth as a small one, and brightness is held constant
/// regardless of this value. The default of 64 blends cleanly at typical sizes.
/// This is the priciest primitive when used on many labels at once — drop to 32
/// for dense UIs, or raise it for extra-smooth hero text.
#[derive(Clone, Copy, Debug)]
pub struct GlowQuality {
    /// Base stamp density (count at an 8px radius). Default: 64.
    pub samples: u32,
}

impl Default for GlowQuality {
    fn default() -> Self {
        Self { samples: 64 }
    }
}

/// All text-effect state carried by a [`StyledLabel`](super::label::StyledLabel).
///
/// You do not construct this directly — use the builder methods on `StyledLabel`:
/// `.text_shadow()`, `.outline()`, `.glow()`, `.glow_quality()`, `.scale()`.
#[derive(Clone, Default, Debug)]
pub struct TextEffects {
    pub shadows: Vec<TextShadow>,
    pub outline: Option<(f32, Color32)>,
    pub glow: Option<Glow>,
    pub glow_quality: GlowQuality,
    pub scale: Option<(f32, Align2)>,
}

impl TextEffects {
    pub fn is_empty(&self) -> bool {
        self.shadows.is_empty()
            && self.outline.is_none()
            && self.glow.is_none()
            && self.scale.is_none()
    }
}

/// Paint all text effects for a label.
///
/// Call this after `label.layout_in_ui(ui)` has allocated space and returned
/// `(pos, galley, response)`. All stamps are inserted into the painter in
/// z-order: glow (bottom) → shadows → outline → main glyph (top). When
/// `.scale()` is set, the entire range is transformed via `TSTransform` about
/// the requested pivot, leaving the allocated rect untouched.
pub fn paint_text_effects(
    ui: &mut egui::Ui,
    pos: Pos2,
    galley: Arc<Galley>,
    fallback_color: Color32,
    effects: &TextEffects,
    response_rect: egui::Rect,
) {
    let layer_id = ui.layer_id();
    let scale_start = ui.painter().add(Shape::Noop);

    // --- glow (bottom layer) ---
    if let Some(glow) = effects.glow {
        paint_glow(ui, pos, galley.clone(), glow, effects.glow_quality.samples);
    }

    // --- directional shadows ---
    for s in &effects.shadows {
        stamp(ui, pos + s.offset, galley.clone(), fallback_color, s.color);
    }

    // --- outline ring ---
    if let Some((width, color)) = effects.outline {
        for offset in outline_offsets(width) {
            stamp(ui, pos + offset, galley.clone(), fallback_color, color);
        }
    }

    // --- main glyph (top) ---
    ui.painter()
        .add(Shape::Text(TextShape::new(pos, galley, fallback_color)));

    let scale_end = ui.painter().add(Shape::Noop);

    // --- scale transform (wraps all shapes above) ---
    if let Some((factor, pivot_align)) = effects.scale {
        let pivot = pivot_align.pos_in_rect(&response_rect);
        // Translate pivot to origin → scale → translate back.
        let t = TSTransform::from_translation(pivot.to_vec2())
            * TSTransform::from_scaling(factor)
            * TSTransform::from_translation(-pivot.to_vec2());
        ui.ctx().graphics_mut(|g| {
            g.entry(layer_id).transform_range(scale_start, scale_end, t);
        });
    }
}

// --- helpers ---

fn stamp(
    ui: &mut egui::Ui,
    pos: Pos2,
    galley: Arc<Galley>,
    fallback_color: Color32,
    color: Color32,
) {
    ui.painter().add(Shape::Text(
        TextShape::new(pos, galley, fallback_color).with_override_text_color(color),
    ));
}

/// Paint glow as a sunflower (Vogel) disk of glyph stamps: the galley is
/// repainted at aperiodic offsets spiralling out to `radius`, each faint and
/// weighted by a window that reaches zero at the disk edge. The golden-angle
/// distribution has no grid or spokes, so overlapping low-alpha copies blend
/// into a smooth halo that follows the letterforms instead of showing moiré.
///
/// `samples` is a base density (stamps at an 8px radius); the actual count is
/// scaled with radius² so larger glows stay equally smooth, and per-stamp alpha
/// is normalized so brightness is independent of both radius and `samples`.
fn paint_glow(ui: &mut egui::Ui, pos: Pos2, galley: Arc<Galley>, glow: Glow, samples: u32) {
    if glow.intensity <= 0.0 || glow.radius <= 0.0 {
        return;
    }
    const GOLDEN_ANGLE: f32 = 2.399_963_2; // π · (3 − √5)
    const STRENGTH: f32 = 0.5;
    const REF_RADIUS: f32 = 8.0; // radius at which `samples` is taken verbatim
    const REF_SAMPLES: f32 = 64.0; // base density the alpha is calibrated against
    const MAX_SAMPLES: u32 = 384; // perf ceiling for very large glows

    let base = samples.max(1) as f32;
    // Hold stamp density (and therefore overlap, and therefore brightness)
    // constant as radius grows: count ∝ area ∝ radius².
    let density_scale = (glow.radius / REF_RADIUS).powi(2).clamp(0.25, 6.0);
    let n = ((base * density_scale).round() as u32)
        .clamp(8, MAX_SAMPLES);

    let intensity = glow.intensity.clamp(0.0, 1.0);
    // Constant overlap scales with `base`, so divide it back out to keep the
    // visible brightness tied only to `intensity`, not to the quality knob.
    let alpha_norm = STRENGTH * intensity * (REF_SAMPLES / base);

    for i in 0..n {
        // Uniform-area radius (sqrt) keeps stamp density constant across the disk.
        let frac = (i as f32 + 0.5) / n as f32;
        let r_norm = frac.sqrt();
        let r = glow.radius * r_norm;
        let theta = i as f32 * GOLDEN_ANGLE;
        let off = Vec2::new(theta.cos() * r, theta.sin() * r);

        // Smooth window: peak 1 at center, exactly 0 at the edge. Because edge
        // stamps are ~0 regardless of intensity, animating intensity never makes
        // them pop in/out (which a hard alpha cutoff would).
        let edge = 1.0 - r_norm * r_norm;
        let a = edge * edge * alpha_norm;
        if a <= 0.002 {
            continue;
        }
        let color = Color32::from_rgba_premultiplied(
            (glow.color.r() as f32 * a) as u8,
            (glow.color.g() as f32 * a) as u8,
            (glow.color.b() as f32 * a) as u8,
            (glow.color.a() as f32 * a) as u8,
        );
        stamp(ui, pos + off, galley.clone(), Color32::TRANSPARENT, color);
    }
}

/// 8-compass outline offsets at `width` pixels from origin.
fn outline_offsets(width: f32) -> [Vec2; 8] {
    let d = width;
    let d45 = width * std::f32::consts::FRAC_1_SQRT_2;
    [
        Vec2::new(d, 0.0),
        Vec2::new(-d, 0.0),
        Vec2::new(0.0, d),
        Vec2::new(0.0, -d),
        Vec2::new(d45, d45),
        Vec2::new(-d45, d45),
        Vec2::new(d45, -d45),
        Vec2::new(-d45, -d45),
    ]
}
