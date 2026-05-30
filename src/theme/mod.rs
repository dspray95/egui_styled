use egui::{CornerRadius, FontFamily, FontId, Vec2};

pub mod theme_ext;
pub mod web_palette;

/// Geometry and typography design tokens.
///
/// Holds the universally-useful scales - corner radii, spacing,
/// font sizes, font families - that apply to most apps regardless of domain.
/// Color choices are *not* in here; they belong to your app, not the library.
///
/// For a starter color vocabulary, see [`WebPalette`](web_palette::WebPalette).
/// For domain-specific colors (game HUD, IDE syntax, etc.), define your own
/// struct and store it via [`DesignSlots::set_design_data`](theme_ext::DesignSlots::set_design_data).
#[derive(Clone, Debug)]
pub struct StyledTheme {
    // Geometry
    pub rounding_sm: CornerRadius,
    pub rounding_md: CornerRadius,
    pub rounding_lg: CornerRadius,
    pub rounding_full: CornerRadius,

    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,

    // Typography - sizes
    pub font_size_sm: f32,
    pub font_size_md: f32,
    pub font_size_lg: f32,
    pub font_size_xl: f32,

    // Typography - families. Compose with sizes via
    // [`StyledTheme::font_display`] / `font_body` / `font_mono`.
    pub font_family_display: FontFamily,
    pub font_family_body: FontFamily,
    pub font_family_mono: FontFamily,

    // Text effect scales (pixel offsets / radii at logical pixel density).
    //
    // `shadow_*` are directional offset vectors for [`StyledLabel::text_shadow`].
    // The default ramp is a gentle downward drop shadow (Tailwind-style).
    //
    // `glow_*` are radius values for [`StyledLabel::glow`].
    // Pass a theme token directly, or pass a per-frame animated `f32` — the
    // methods accept raw values, not enum tokens.
    pub shadow_sm: Vec2,
    pub shadow_md: Vec2,
    pub shadow_lg: Vec2,

    pub glow_sm: f32,
    pub glow_md: f32,
    pub glow_lg: f32,
}

impl StyledTheme {
    /// Build a [`FontId`] from the display family at `size`.
    pub fn font_display(&self, size: f32) -> FontId {
        FontId::new(size, self.font_family_display.clone())
    }

    /// Build a [`FontId`] from the body family at `size`.
    pub fn font_body(&self, size: f32) -> FontId {
        FontId::new(size, self.font_family_body.clone())
    }

    /// Build a [`FontId`] from the monospace family at `size`.
    pub fn font_mono(&self, size: f32) -> FontId {
        FontId::new(size, self.font_family_mono.clone())
    }
}

impl Default for StyledTheme {
    /// Sensible geometry/typography defaults. Spacing/radii on a `*2` scale,
    /// font sizes 12 / 14 / 18 / 24. Font families default to egui's
    /// proportional + monospace.
    fn default() -> Self {
        Self {
            rounding_sm: CornerRadius::same(2),
            rounding_md: CornerRadius::same(4),
            rounding_lg: CornerRadius::same(8),
            rounding_full: CornerRadius::same(u8::MAX),

            spacing_xs: 2.0,
            spacing_sm: 4.0,
            spacing_md: 8.0,
            spacing_lg: 16.0,
            spacing_xl: 32.0,

            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 18.0,
            font_size_xl: 24.0,

            font_family_display: FontFamily::Proportional,
            font_family_body: FontFamily::Proportional,
            font_family_mono: FontFamily::Monospace,

            shadow_sm: Vec2::new(0.0, 1.0),
            shadow_md: Vec2::new(0.0, 2.0),
            shadow_lg: Vec2::new(0.0, 4.0),

            glow_sm: 4.0,
            glow_md: 8.0,
            glow_lg: 16.0,
        }
    }
}
