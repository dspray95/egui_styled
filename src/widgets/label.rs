use egui::{
    Align2, Color32, FontId, Label, Response, RichText, Shape, Stroke, TextWrapMode, Ui, Vec2,
    WidgetText,
};

use crate::{
    impl_style_builders,
    style::shared_style::{SharedStyle, paint_shadows, render_scoped},
};

use super::text_effects::{Glow, GlowQuality, TextEffects, TextShadow, paint_text_effects};

/// A styled label.
///
/// Labels are not interactive so pseudo-state fields (`hover_bg`, `active_bg`,
/// `focus_bg`, `hover_border`, `focus_border`, `hover_text_color`) are
/// accepted by the builder but have no effect. Also unsupported: `max_width/height`.
/// Use `text_color`, `bg`, `border`, `corner_radius`, `padding`, `margin`,
/// `font_size`, `font`, `bold`, `italics`, `wrap_mode`, `extend`, `truncate`,
/// `wrap`, `min_height`, `visible`, `shadows`, and `cursor` instead.
///
/// # Text effects
///
/// [`text_shadow`](StyledLabel::text_shadow), [`outline`](StyledLabel::outline),
/// [`glow`](StyledLabel::glow), and [`scale`](StyledLabel::scale) paint
/// *glyph-shaped* decorations — they stamp the laid-out galley at offsets and
/// colors rather than painting offset rectangles like the box-shadow `.shadow()`.
/// These methods are label-only and compose freely:
///
/// ```ignore
/// // Chromatic aberration via two opposite-offset shadows
/// Styled::label("[ENTER]")
///     .text_shadow(vec2(-2.0, 0.0), cyan)
///     .text_shadow(vec2( 2.0, 0.0), magenta)
///     .text_color(Color32::WHITE)
///     .extend()
///     .show(ui);
///
/// // Soft glow, intensity driven per-frame
/// Styled::label("SCORE")
///     .glow(Color32::from_rgb(0, 220, 255), theme.glow_md, intensity)
///     .text_color(Color32::WHITE)
///     .show(ui);
///
/// // Scale-punch without layout shift — pair with `Styled::stack().layer_fixed()`
/// Styled::label("000123456")
///     .scale(factor, Align2::CENTER_CENTER)
///     .text_color(Color32::WHITE)
///     .show(ui);
/// ```
pub struct StyledLabel {
    text: WidgetText,
    bold: bool,
    italics: bool,
    wrap_mode: Option<TextWrapMode>,
    font: Option<FontId>,
    style: SharedStyle,
    effects: TextEffects,
}

impl StyledLabel {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            bold: false,
            italics: false,
            wrap_mode: None,
            font: None,
            style: SharedStyle::default(),
            effects: TextEffects::default(),
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italics(mut self) -> Self {
        self.italics = true;
        self
    }

    /// Set the text wrap mode explicitly.
    pub fn wrap_mode(mut self, mode: TextWrapMode) -> Self {
        self.wrap_mode = Some(mode);
        self
    }

    /// Lay out at natural width — never shrink, never truncate. Useful in tight
    /// horizontal layouts where truncation would otherwise kick in.
    pub fn extend(self) -> Self {
        self.wrap_mode(TextWrapMode::Extend)
    }

    /// Truncate text with an ellipsis when it overflows.
    pub fn truncate(self) -> Self {
        self.wrap_mode(TextWrapMode::Truncate)
    }

    /// Wrap text onto multiple lines.
    pub fn wrap(self) -> Self {
        self.wrap_mode(TextWrapMode::Wrap)
    }

    /// Set the font (family + size) used to render the label.
    /// Overrides [`SharedStyle::font_size`] when both are set.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    // ── Text effects ────────────────────────────────────────────────────────
    // These methods set glyph-shaped decorations (galley stamps) that are
    // label-only and cannot be expressed through the shared style macro because
    // they have no meaning on non-text widgets.

    /// Paint the glyph run at `offset` pixels in `color`, underneath the main
    /// text. Multiple calls append; shadows paint in call order from bottom up.
    ///
    /// Compose two opposite-offset shadows for chromatic aberration:
    /// ```ignore
    /// .text_shadow(vec2(-2.0, 0.0), cyan)
    /// .text_shadow(vec2( 2.0, 0.0), magenta)
    /// ```
    ///
    /// Use a theme offset token or an animated `Vec2` — the method accepts a
    /// raw value, not an enum token:
    /// ```ignore
    /// .text_shadow(theme.shadow_md, Color32::BLACK.linear_multiply(0.5))
    /// ```
    pub fn text_shadow(mut self, offset: Vec2, color: Color32) -> Self {
        self.effects.shadows.push(TextShadow { offset, color });
        self
    }

    /// Paint a ring of glyph-stamp copies around the text to create a faux
    /// stroke outline. Uses 8 compass directions at `width` pixels distance.
    ///
    /// ```ignore
    /// Styled::label("GAME OVER")
    ///     .outline(1.5, Color32::BLACK)
    ///     .text_color(Color32::WHITE)
    /// ```
    pub fn outline(mut self, width: f32, color: Color32) -> Self {
        self.effects.outline = Some((width, color));
        self
    }

    /// Paint a soft radial glow around the text using concentric rings of
    /// low-alpha galley stamps. `intensity` is 0.0..=1.0 and is supplied
    /// per-frame — animate it on the consumer side, pass the current value here.
    ///
    /// `radius` controls the outer extent in logical pixels. Use a theme glow
    /// token (`theme.glow_md`) or a raw `f32`:
    /// ```ignore
    /// Styled::label("SCORE")
    ///     .glow(Color32::from_rgb(0, 220, 255), theme.glow_md, 0.8)
    /// ```
    ///
    /// See [`glow_quality`](StyledLabel::glow_quality) to tune the stamp count
    /// if glow is used on many labels per frame.
    pub fn glow(mut self, color: Color32, radius: f32, intensity: f32) -> Self {
        self.effects.glow = Some(Glow {
            color,
            radius,
            intensity,
        });
        self
    }

    /// Override the glow stamp density used to render [`glow`](StyledLabel::glow).
    ///
    /// `samples` is a base density (stamp count at an 8px radius); the real
    /// count scales with radius² automatically, and brightness is independent
    /// of this value. The default (64) blends smoothly at typical font sizes.
    /// Drop to 32 for dense UIs with many glowing labels, or raise it for
    /// extra-smooth hero text:
    /// ```ignore
    /// .glow_quality(32)   // cheaper, slightly grainier
    /// ```
    pub fn glow_quality(mut self, samples: u32) -> Self {
        self.effects.glow_quality = GlowQuality { samples };
        self
    }

    /// Scale the painted glyphs about `pivot` without shifting the allocated
    /// layout footprint. Siblings are unaffected.
    ///
    /// `factor` is a plain per-frame value — keep the animation curve on the
    /// consumer side and pass the current scale here. Pair with
    /// `Styled::stack().layer_fixed(resting_size, align, ...)` when the scaled
    /// content would otherwise overflow the label's natural bounds:
    /// ```ignore
    /// let resting = vec2(120.0, 40.0);
    /// Styled::stack()
    ///     .layer_fixed(resting, Align2::CENTER_CENTER, |ui| {
    ///         Styled::label("000123456")
    ///             .scale(factor, Align2::CENTER_CENTER)
    ///             .show(ui);
    ///     })
    ///     .show(ui);
    /// ```
    pub fn scale(mut self, factor: f32, pivot: Align2) -> Self {
        self.effects.scale = Some((factor, pivot));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let visible = self.style.visible != Some(false);

        let mut rich = match self.text {
            WidgetText::RichText(rt) => (*rt).clone(),
            other => RichText::new(other.text().to_string()),
        };
        if let Some(color) = self.style.text_color {
            rich = rich.color(color);
        }
        if let Some(font) = self.font.clone() {
            rich = rich.font(font);
        } else if let Some(size) = self.style.font_size {
            rich = rich.size(size);
        }
        if self.bold {
            rich = rich.strong();
        }
        if self.italics {
            rich = rich.italics();
        }

        // Resolve static style (labels have no interaction state).
        let visuals = ui.visuals().clone();
        let wv = &visuals.widgets.inactive;
        let bg = self.style.bg.unwrap_or(egui::Color32::TRANSPARENT);
        let border = self.style.border.unwrap_or(Stroke::NONE);
        let corner_radius = self.style.corner_radius.unwrap_or(wv.corner_radius);
        let padding = self.style.padding.unwrap_or_default();
        let margin = self.style.margin.unwrap_or_default();

        let has_effects = !self.effects.is_empty();
        let effects = self.effects;
        let wrap_mode = self.wrap_mode;
        let style_shadows = self.style.shadows.clone();

        let response = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);

            let response = egui::Frame::new()
                .fill(bg)
                .stroke(border)
                .corner_radius(corner_radius)
                .inner_margin(padding)
                .outer_margin(margin)
                .show(ui, |ui| {
                    if self.style.full_width {
                        ui.set_min_width(ui.available_width());
                    }
                    if let Some(min_w) = self.style.min_width {
                        ui.set_min_width(min_w);
                    }
                    if let Some(min_h) = self.style.min_height {
                        ui.set_min_height(min_h);
                    }

                    if has_effects {
                        // Effects path: layout without painting so we can stamp
                        // the galley at multiple positions/colors.
                        let mut label = Label::new(rich);
                        if let Some(mode) = wrap_mode {
                            label = label.wrap_mode(mode);
                        }
                        let (pos, galley, response) = label.layout_in_ui(ui);
                        let fallback = ui.visuals().widgets.inactive.text_color();
                        paint_text_effects(ui, pos, galley, fallback, &effects, response.rect);
                        response
                    } else {
                        // Fast path: let egui handle everything (widget_info,
                        // tooltip-when-elided, selection, screen-reader).
                        let mut label = Label::new(rich);
                        if let Some(mode) = wrap_mode {
                            label = label.wrap_mode(mode);
                        }
                        ui.add(label)
                    }
                })
                .inner;

            paint_shadows(ui, shadow_idx, response.rect, corner_radius, &style_shadows);
            response
        });

        if let Some(icon) = self.style.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        response
    }
}

impl_style_builders!(StyledLabel);
