//! Design tokens and reusable style functions. Scenes read the palette with
//! `ctx.design_data::<Palette>()` and compose these with `.apply(...)`.

use egui::{Color32, vec2};
use egui_styled::prelude::*;

use crate::fonts::{Weight, font};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    #[default]
    Dark,
    Brutal,
}

#[derive(Clone)]
pub struct Palette {
    pub mode: Mode,
    pub bg: Color32,
    pub surface: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub on_accent: Color32,
    pub success: Color32,
    pub danger: Color32,
    /// Tile fills for the brutal theme; tints for the dark one.
    pub highlights: [Color32; 4],
    pub radius: f32,
    pub border_width: f32,
}

impl Default for Palette {
    fn default() -> Self {
        Self::dark()
    }
}

impl Palette {
    pub fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Dark => Self::dark(),
            Mode::Brutal => Self::brutal(),
        }
    }

    /// Linear / Vercel: near-black, hairline borders, one violet accent.
    fn dark() -> Self {
        Self {
            mode: Mode::Dark,
            bg: rgb(9, 9, 11),
            surface: rgb(17, 17, 21),
            border: rgb(255, 255, 255).with_alpha(22),
            text: rgb(244, 244, 245),
            text_muted: rgb(161, 161, 170),
            accent: rgb(124, 58, 237),
            accent_hover: rgb(139, 92, 246),
            on_accent: Color32::WHITE,
            success: rgb(52, 211, 153),
            danger: rgb(248, 113, 113),
            highlights: [
                rgb(124, 58, 237),
                rgb(37, 99, 235),
                rgb(20, 184, 166),
                rgb(236, 72, 153),
            ],
            radius: 14.0,
            border_width: 1.0,
        }
    }

    /// Gumroad / Figma marketing: cream, black outlines, hard offset shadows.
    fn brutal() -> Self {
        Self {
            mode: Mode::Brutal,
            bg: rgb(255, 244, 224),
            surface: Color32::WHITE,
            border: Color32::BLACK,
            text: rgb(17, 17, 17),
            text_muted: rgb(70, 70, 70),
            accent: rgb(255, 92, 168),
            accent_hover: rgb(255, 122, 186),
            on_accent: Color32::BLACK,
            success: rgb(0, 160, 100),
            danger: rgb(220, 38, 38),
            highlights: [
                rgb(255, 210, 63),
                rgb(144, 216, 255),
                rgb(161, 239, 180),
                rgb(255, 170, 205),
            ],
            radius: 8.0,
            border_width: 2.5,
        }
    }

    pub fn brutal_shadow(&self) -> egui::Vec2 {
        vec2(5.0, 5.0)
    }
}

/// A raised surface: hairline border and soft glow (dark), or outline and hard shadow (brutal).
pub fn card(p: &Palette) -> impl Fn(StyledFrame) -> StyledFrame + 'static {
    let p = p.clone();
    move |f| {
        let f = f
            .bg(p.surface)
            .border(p.border_width, p.border)
            .corner_radius(p.radius)
            .padding(20.0);
        match p.mode {
            Mode::Dark => f.inner_glow(14.0, rgb(255, 255, 255).with_alpha(6)),
            Mode::Brutal => f.shadow_filled(p.brutal_shadow(), Color32::BLACK),
        }
    }
}

pub fn primary_button(p: &Palette) -> impl Fn(StyledButton) -> StyledButton + 'static {
    let p = p.clone();
    move |b| {
        let b = b
            .bg(p.accent)
            .hover_bg(p.accent_hover)
            .active_bg(p.accent.darken(0.1))
            .text_color(p.on_accent)
            .font(font(Weight::Semibold, 14.0))
            .corner_radius(p.radius * 0.7)
            .padding(egui::Margin::symmetric(18, 11));
        match p.mode {
            Mode::Dark => b.inner_glow(10.0, rgb(255, 255, 255).with_alpha(28)),
            Mode::Brutal => b
                .border(p.border_width, p.border)
                .shadow_filled(vec2(3.0, 3.0), Color32::BLACK),
        }
    }
}

pub fn ghost_button(p: &Palette) -> impl Fn(StyledButton) -> StyledButton + 'static {
    let p = p.clone();
    move |b| {
        b.bg(match p.mode {
            Mode::Dark => rgb(255, 255, 255).with_alpha(8),
            Mode::Brutal => p.surface,
        })
        .hover_bg(match p.mode {
            Mode::Dark => rgb(255, 255, 255).with_alpha(16),
            Mode::Brutal => p.highlights[0],
        })
        .border(p.border_width, p.border)
        .text_color(p.text)
        .font(font(Weight::Medium, 14.0))
        .corner_radius(p.radius * 0.7)
        .padding(egui::Margin::symmetric(18, 11))
    }
}

pub fn input(p: &Palette) -> impl Fn(StyledTextEdit<'_>) -> StyledTextEdit<'_> + 'static {
    let p = p.clone();
    move |t| {
        t.bg(match p.mode {
            Mode::Dark => rgb(0, 0, 0).with_alpha(90),
            Mode::Brutal => p.surface,
        })
        .border(p.border_width, p.border)
        .focus_border(p.border_width.max(1.5), p.accent)
        .text_color(p.text)
        .font(font(Weight::Regular, 14.0))
        .corner_radius(p.radius * 0.7)
        .padding(egui::Margin::symmetric(12, 10))
        .full_width()
    }
}

pub fn slider<'a, T: egui::emath::Numeric>(
    p: &Palette,
) -> impl Fn(StyledSlider<'a, T>) -> StyledSlider<'a, T> + 'static {
    let p = p.clone();
    move |s| {
        let s = s
            .width(130.0)
            .fill(p.accent)
            .text_color(p.text)
            .font_size(13.0);
        match p.mode {
            Mode::Dark => s
                .rail(rgb(255, 255, 255).with_alpha(22))
                .rail_thickness(5.0)
                .handle(Color32::WHITE)
                .handle_radius(7.0)
                .value_bg(rgb(255, 255, 255).with_alpha(10))
                .corner_radius(6.0),
            Mode::Brutal => s
                .rail(rgb(0, 0, 0).with_alpha(30))
                .rail_thickness(7.0)
                .handle(p.surface)
                .hover_handle(p.highlights[0])
                .handle_border(2.5, p.border)
                .handle_radius(9.0)
                .value_bg(p.surface)
                .border(1.5, p.border)
                .corner_radius(4.0),
        }
    }
}

/// A label in the bundled font at `weight`.
pub fn text(s: impl Into<String>, weight: Weight, size: f32, color: Color32) -> StyledLabel {
    Styled::label(s.into())
        .font(font(weight, size))
        .text_color(color)
}

/// Small uppercase section label.
pub fn eyebrow(s: &str, p: &Palette) -> StyledLabel {
    text(s.to_uppercase(), Weight::Semibold, 11.0, p.text_muted)
}
