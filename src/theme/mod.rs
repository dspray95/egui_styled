use egui::{Color32, CornerRadius};

use crate::rgb;

pub mod theme_ext;

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

    // Typography
    pub font_size_sm: f32,
    pub font_size_md: f32,
    pub font_size_lg: f32,
    pub font_size_xl: f32,
}

impl Default for StyledTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl StyledTheme {
    pub fn dark() -> Self {
        Self {
            bg_primary: rgb(15, 15, 15),
            bg_secondary: rgb(20, 20, 20),
            bg_surface: rgb(30, 30, 30),
            bg_elevated: rgb(40, 40, 40),

            fg_primary: Color32::from_gray(240),
            fg_secondary: Color32::from_gray(180),
            fg_muted: Color32::from_gray(120),

            accent: rgb(60, 60, 255),
            accent_hover: rgb(80, 80, 255),
            accent_active: rgb(40, 40, 200),

            error: rgb(255, 80, 80),
            warning: rgb(255, 180, 60),
            success: rgb(80, 200, 120),

            border: rgb(60, 60, 60),
            border_focus: rgb(100, 100, 255),

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
        }
    }

    pub fn light() -> Self {
        Self {
            bg_primary: rgb(250, 250, 250),
            bg_secondary: rgb(240, 240, 240),
            bg_surface: rgb(255, 255, 255),
            bg_elevated: rgb(230, 230, 235),

            fg_primary: Color32::from_gray(20),
            fg_secondary: Color32::from_gray(70),
            fg_muted: Color32::from_gray(130),

            accent: rgb(40, 90, 220),
            accent_hover: rgb(60, 110, 240),
            accent_active: rgb(25, 70, 190),

            error: rgb(200, 40, 40),
            warning: rgb(190, 130, 20),
            success: rgb(40, 150, 80),

            border: rgb(210, 210, 215),
            border_focus: rgb(60, 110, 240),

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
        }
    }
}
