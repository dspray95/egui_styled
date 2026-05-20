use egui::Color32;

/// Shorthand for Color32::from_rgb
pub const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Shorthand for Color32::from_rgba_premultiplied
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_premultiplied(r, g, b, a)
}
