use egui::{Color32, CornerRadius, FontFamily, FontId};

pub mod theme_ext;

/// Design tokens for a styled UI. Construct one per app — there are no
/// built-in presets; the values are aesthetic choices that belong to your
/// product, not this library. See `examples/` for full themes you can copy.
#[derive(Clone, Debug)]
pub struct StyledTheme {
    // Semantic colors
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_surface: Color32,
    pub bg_elevated: Color32,

    pub fg_primary: Color32,
    pub fg_secondary: Color32,
    pub fg_muted: Color32,
    /// Text drawn on top of saturated/accent surfaces (filled buttons,
    /// status chips, etc). Usually white in dark themes and very-light in
    /// light themes; rarely matches `fg_primary`.
    pub fg_on_accent: Color32,

    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_active: Color32,

    pub error: Color32,
    pub warning: Color32,
    pub success: Color32,

    pub border: Color32,
    pub border_focus: Color32,

    // Geometry tokens
    pub rounding_sm: CornerRadius,
    pub rounding_md: CornerRadius,
    pub rounding_lg: CornerRadius,
    pub rounding_full: CornerRadius,

    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,

    // Typography — sizes
    pub font_size_sm: f32,
    pub font_size_md: f32,
    pub font_size_lg: f32,
    pub font_size_xl: f32,

    // Typography — families. Compose with sizes via [`StyledTheme::font_display`]
    // / `font_body` / `font_mono`. Register the underlying fonts with egui at
    // startup before assigning a [`FontFamily::Name`].
    pub font_family_display: FontFamily,
    pub font_family_body: FontFamily,
    pub font_family_mono: FontFamily,
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
    /// A neutral, deliberately bland fallback. Backgrounds are mid-grays,
    /// foregrounds are near-white/black, accent is egui's default blue.
    /// This exists so `ctx.styled_theme()` is non-panicking before `set_styled_theme`
    /// is called — it is not meant to be used as a finished aesthetic.
    fn default() -> Self {
        Self {
            bg_primary: Color32::from_gray(30),
            bg_secondary: Color32::from_gray(40),
            bg_surface: Color32::from_gray(50),
            bg_elevated: Color32::from_gray(60),

            fg_primary: Color32::from_gray(230),
            fg_secondary: Color32::from_gray(180),
            fg_muted: Color32::from_gray(130),
            fg_on_accent: Color32::WHITE,

            accent: Color32::from_rgb(70, 120, 200),
            accent_hover: Color32::from_rgb(90, 140, 220),
            accent_active: Color32::from_rgb(55, 100, 180),

            error: Color32::from_rgb(200, 80, 80),
            warning: Color32::from_rgb(200, 160, 60),
            success: Color32::from_rgb(80, 180, 110),

            border: Color32::from_gray(80),
            border_focus: Color32::from_rgb(120, 160, 240),

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
        }
    }
}
