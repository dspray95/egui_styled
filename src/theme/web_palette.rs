use egui::Color32;

/// A starter color palette with web-style semantic names.
///
/// Opt-in — the library doesn't assume your app wants this vocabulary.
/// Good fit for SaaS / dashboard / docs UIs. For domain-specific apps
/// (games, IDEs, creative tools) define your own struct with the colors
/// that fit your domain, and store it via
/// [`DesignSlots::set_design_data`](crate::theme::theme_ext::DesignSlots::set_design_data).
///
/// Store it on the context once at startup, read it anywhere:
///
/// ```ignore
/// ctx.set_design_data(WebPalette { /* … */ });
/// let p = ctx.design_data::<WebPalette>();
/// Styled::button("Save").bg(p.accent).show(ui);
/// ```
#[derive(Clone, Debug)]
pub struct WebPalette {
    // Backgrounds
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_surface: Color32,
    pub bg_elevated: Color32,

    // Foregrounds
    pub fg_primary: Color32,
    pub fg_secondary: Color32,
    pub fg_muted: Color32,
    /// Text drawn on top of saturated/accent surfaces (filled buttons,
    /// status chips). Usually white in dark themes, very-light in light
    /// themes; rarely matches `fg_primary`.
    pub fg_on_accent: Color32,

    // Accent
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_active: Color32,

    // Status
    pub error: Color32,
    pub warning: Color32,
    pub success: Color32,

    // Borders
    pub border: Color32,
    pub border_focus: Color32,
}

impl Default for WebPalette {
    /// A deliberately bland neutral palette. Backgrounds are mid-grays,
    /// foregrounds are near-white/black, accent is a plain blue. Exists so
    /// `ctx.design_data::<WebPalette>()` doesn't panic before
    /// `set_design_data` is called — not meant as a finished aesthetic.
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
        }
    }
}
