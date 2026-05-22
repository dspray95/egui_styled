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
pub trait ColorExt {
    /// Return a copy of this color with its alpha replaced.
    ///
    /// `let dim = colors.background.with_alpha(180);`
    fn with_alpha(self, alpha: u8) -> Color32;
}

impl ColorExt for Color32 {
    fn with_alpha(self, alpha: u8) -> Color32 {
        let [r, g, b, _] = self.to_array();
        Color32::from_rgba_unmultiplied(r, g, b, alpha)
    }
}
