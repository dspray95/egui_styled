use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

#[derive(Default)]
struct AppState {
    use_parchment: bool,
    username: String,
}

fn theme_demo(ui: &mut egui::Ui, state: &mut AppState) {
    let theme = ui.ctx().styled_theme();

    Styled::frame()
        .bg(theme.bg_surface)
        .corner_radius(theme.rounding_lg)
        .padding(theme.spacing_lg)
        .border(1.0, theme.border)
        .show(ui, |ui| {
            Styled::column().gap(theme.spacing_md).show(ui, |ui| {
                Styled::row().gap(theme.spacing_md).show(ui, |ui| {
                    if Styled::button(if state.use_parchment {
                        "Switch to midnight"
                    } else {
                        "Switch to parchment"
                    })
                    .bg(theme.accent)
                    .hover_bg(theme.accent_hover)
                    .active_bg(theme.accent_active)
                    .text_color(Color32::WHITE)
                    .corner_radius(theme.rounding_md)
                    .show(ui)
                    .clicked()
                    {
                        state.use_parchment = !state.use_parchment;
                        let next = if state.use_parchment {
                            parchment()
                        } else {
                            midnight()
                        };
                        ui.ctx().set_styled_theme(next);
                    }
                });

                Styled::text_edit(&mut state.username)
                    .hint("Username")
                    .full_width()
                    .bg(theme.bg_secondary)
                    .corner_radius(theme.rounding_md)
                    .border(1.0, theme.border)
                    .focus_border(1.0, theme.border_focus)
                    .show(ui);

                Styled::row().gap(theme.spacing_sm).show(ui, |ui| {
                    swatch(ui, "primary", theme.bg_primary, theme.fg_primary, &theme);
                    swatch(
                        ui,
                        "secondary",
                        theme.bg_secondary,
                        theme.fg_primary,
                        &theme,
                    );
                    swatch(ui, "surface", theme.bg_surface, theme.fg_primary, &theme);
                    swatch(ui, "elevated", theme.bg_elevated, theme.fg_primary, &theme);
                });

                Styled::row().gap(theme.spacing_sm).show(ui, |ui| {
                    swatch(ui, "accent", theme.accent, theme.fg_on_accent, &theme);
                    swatch(ui, "error", theme.error, theme.fg_on_accent, &theme);
                    swatch(ui, "warning", theme.warning, theme.fg_on_accent, &theme);
                    swatch(ui, "success", theme.success, theme.fg_on_accent, &theme);
                });
            });
        });
}

fn swatch(ui: &mut egui::Ui, name: &str, bg: Color32, fg: Color32, theme: &StyledTheme) {
    Styled::frame()
        .bg(bg)
        .border(1.0, theme.border)
        .corner_radius(theme.rounding_sm)
        .padding(theme.spacing_sm)
        .show(ui, |ui| {
            Styled::label(name).text_color(fg).show(ui);
        });
}

/// Deep navy with a cyan accent — high contrast, code-editor feel.
fn midnight() -> StyledTheme {
    use egui::CornerRadius;
    StyledTheme {
        bg_primary: Color32::from_rgb(10, 14, 24),
        bg_secondary: Color32::from_rgb(16, 22, 36),
        bg_surface: Color32::from_rgb(22, 30, 48),
        bg_elevated: Color32::from_rgb(32, 42, 64),

        fg_primary: Color32::from_rgb(220, 235, 250),
        fg_secondary: Color32::from_rgb(160, 180, 210),
        fg_muted: Color32::from_rgb(100, 120, 150),
        fg_on_accent: Color32::from_rgb(10, 14, 24),

        accent: Color32::from_rgb(80, 200, 220),
        accent_hover: Color32::from_rgb(110, 220, 235),
        accent_active: Color32::from_rgb(60, 175, 200),

        error: Color32::from_rgb(240, 90, 110),
        warning: Color32::from_rgb(245, 200, 90),
        success: Color32::from_rgb(110, 220, 150),

        border: Color32::from_rgb(48, 62, 88),
        border_focus: Color32::from_rgb(110, 220, 235),

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

/// Cream paper with warm browns and a rust accent — bookish, low-glare.
fn parchment() -> StyledTheme {
    use egui::CornerRadius;
    StyledTheme {
        bg_primary: Color32::from_rgb(245, 238, 220),
        bg_secondary: Color32::from_rgb(238, 228, 205),
        bg_surface: Color32::from_rgb(250, 245, 230),
        bg_elevated: Color32::from_rgb(232, 220, 195),

        fg_primary: Color32::from_rgb(60, 40, 20),
        fg_secondary: Color32::from_rgb(105, 80, 55),
        fg_muted: Color32::from_rgb(150, 130, 100),
        fg_on_accent: Color32::from_rgb(250, 245, 230),

        accent: Color32::from_rgb(170, 70, 40),
        accent_hover: Color32::from_rgb(195, 90, 55),
        accent_active: Color32::from_rgb(145, 55, 30),

        error: Color32::from_rgb(180, 50, 50),
        warning: Color32::from_rgb(180, 130, 30),
        success: Color32::from_rgb(90, 140, 70),

        border: Color32::from_rgb(200, 180, 145),
        border_focus: Color32::from_rgb(195, 90, 55),

        rounding_sm: CornerRadius::same(1),
        rounding_md: CornerRadius::same(2),
        rounding_lg: CornerRadius::same(3),
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

fn main() -> eframe::Result<()> {
    let mut state = AppState::default();
    let mut initialized = false;
    eframe::run_ui_native(
        "egui_styled theme demo",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(midnight());
                initialized = true;
            }
            CentralPanel::default().show_inside(ctx, |ui| theme_demo(ui, &mut state));
        },
    )
}
