use egui::Color32;

/// Shorthand for Color32::from_rgb
pub const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Shorthand for Color32::from_rgba_premultiplied
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_premultiplied(r, g, b, a)
}

/// Convenience methods on [`Color32`].
///
/// All operations work in sRGB - not perceptually uniform. Good enough for
/// hover / active / disabled variants of an existing color. For serious
/// color work (palettes derived from a single brand color, accessibility
/// contrast checks, gamut mapping), reach for the
/// [`palette`](https://crates.io/crates/palette) crate.
pub trait ColorExt {
    /// Return a copy of this color with its alpha replaced.
    fn with_alpha(self, alpha: u8) -> Color32;

    /// Linear blend toward `other` by factor `t` (0.0 = self, 1.0 = other).
    /// `t` is clamped to `[0.0, 1.0]`. Blends RGB and alpha.
    fn lerp(self, other: Color32, t: f32) -> Color32;

    /// Blend toward white by `factor` (0.0 = no change, 1.0 = white).
    /// Useful for `hover_bg` derived from a base color.
    fn lighten(self, factor: f32) -> Color32;

    /// Blend toward black by `factor` (0.0 = no change, 1.0 = black).
    /// Useful for `active_bg` derived from a base color.
    fn darken(self, factor: f32) -> Color32;
}

impl ColorExt for Color32 {
    fn with_alpha(self, alpha: u8) -> Color32 {
        let [r, g, b, _] = self.to_array();
        Color32::from_rgba_unmultiplied(r, g, b, alpha)
    }

    fn lerp(self, other: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        let [r1, g1, b1, a1] = self.to_array();
        let [r2, g2, b2, a2] = other.to_array();
        let mix = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * t).round() as u8 };
        Color32::from_rgba_unmultiplied(mix(r1, r2), mix(g1, g2), mix(b1, b2), mix(a1, a2))
    }

    fn lighten(self, factor: f32) -> Color32 {
        self.lerp(Color32::WHITE, factor)
    }

    fn darken(self, factor: f32) -> Color32 {
        self.lerp(Color32::BLACK, factor)
    }
}
