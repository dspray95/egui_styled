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

/// Soft glow that follows the glyphs: a Gaussian blur of the text, tinted `color`.
///
/// `intensity` is a plain 0.0..=1.0 per-frame value; keep animation state on the consumer side.
/// Animating `intensity` or `color` is free; animating `radius` re-blurs each frame.
#[derive(Clone, Copy, Debug)]
pub struct Glow {
    pub color: Color32,
    /// Blur radius in logical pixels. Use a theme glow token or an animated value.
    pub radius: f32,
    /// 0.0 = invisible, 1.0 = full-strength glow.
    pub intensity: f32,
}

/// Unused now that glow is a blur.
#[deprecated(since = "0.9.2", note = "does nothing now that glow is a blur")]
#[derive(Clone, Copy, Debug)]
pub struct GlowQuality {
    pub samples: u32,
}

#[allow(deprecated)]
impl Default for GlowQuality {
    fn default() -> Self {
        Self { samples: 64 }
    }
}

/// All text-effect state carried by a [`StyledLabel`](super::label::StyledLabel).
///
/// You do not construct this directly — use the builder methods on `StyledLabel`:
/// `.text_shadow()`, `.outline()`, `.glow()`, `.scale()`.
#[allow(deprecated)]
#[derive(Clone, Default, Debug)]
pub struct TextEffects {
    pub shadows: Vec<TextShadow>,
    pub outline: Option<(f32, Color32)>,
    pub glow: Option<Glow>,
    #[deprecated(since = "0.9.2", note = "does nothing now that glow is a blur")]
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
/// `(pos, galley, response)`. Paints in z-order: glow (bottom), shadows, outline,
/// main glyph (top). When
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
        super::glow_mask::paint(ui, pos, &galley, glow);
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
