use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

#[derive(Default)]
struct AppState {
    use_parchment: bool,
    username: String,
}

fn theme_demo(ui: &mut egui::Ui, state: &mut AppState) {
    let t = ui.ctx().styled_theme();
    let p = ui.ctx().design_data::<WebPalette>();

    Styled::frame()
        .bg(p.bg_surface)
        .corner_radius(t.rounding_lg)
        .padding(t.spacing_lg)
        .border(1.0, p.border)
        .show(ui, |ui| {
            Styled::column().gap(t.spacing_md).show(ui, |ui| {
                Styled::row().gap(t.spacing_md).show(ui, |ui| {
                    if Styled::button(if state.use_parchment {
                        "Switch to midnight"
                    } else {
                        "Switch to parchment"
                    })
                    .bg(p.accent)
                    .hover_bg(p.accent_hover)
                    .active_bg(p.accent_active)
                    .text_color(Color32::WHITE)
                    .corner_radius(t.rounding_md)
                    .show(ui)
                    .clicked()
                    {
                        state.use_parchment = !state.use_parchment;
                        let (geo, pal) = if state.use_parchment {
                            (parchment_geometry(), parchment_palette())
                        } else {
                            (midnight_geometry(), midnight_palette())
                        };
                        ui.ctx().set_styled_theme(geo);
                        ui.ctx().set_design_data(pal);
                    }
                });

                Styled::text_edit(&mut state.username)
                    .hint("Username")
                    .full_width()
                    .bg(p.bg_secondary)
                    .corner_radius(t.rounding_md)
                    .border(1.0, p.border)
                    .focus_border(1.0, p.border_focus)
                    .show(ui);

                Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                    swatch(ui, "primary", p.bg_primary, p.fg_primary, &t, &p);
                    swatch(ui, "secondary", p.bg_secondary, p.fg_primary, &t, &p);
                    swatch(ui, "surface", p.bg_surface, p.fg_primary, &t, &p);
                    swatch(ui, "elevated", p.bg_elevated, p.fg_primary, &t, &p);
                });

                Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                    swatch(ui, "accent", p.accent, p.fg_on_accent, &t, &p);
                    swatch(ui, "error", p.error, p.fg_on_accent, &t, &p);
                    swatch(ui, "warning", p.warning, p.fg_on_accent, &t, &p);
                    swatch(ui, "success", p.success, p.fg_on_accent, &t, &p);
                });
            });
        });
}

fn swatch(
    ui: &mut egui::Ui,
    name: &str,
    bg: Color32,
    fg: Color32,
    t: &StyledTheme,
    p: &WebPalette,
) {
    Styled::frame()
        .bg(bg)
        .border(1.0, p.border)
        .corner_radius(t.rounding_sm)
        .padding(t.spacing_sm)
        .show(ui, |ui| {
            Styled::label(name).text_color(fg).show(ui);
        });
}

fn midnight_geometry() -> StyledTheme {
    StyledTheme::default()
}

fn midnight_palette() -> WebPalette {
    WebPalette {
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
    }
}

fn parchment_geometry() -> StyledTheme {
    use egui::CornerRadius;
    StyledTheme {
        rounding_sm: CornerRadius::same(1),
        rounding_md: CornerRadius::same(2),
        rounding_lg: CornerRadius::same(3),
        ..Default::default()
    }
}

fn parchment_palette() -> WebPalette {
    WebPalette {
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
                ctx.set_styled_theme(midnight_geometry());
                ctx.set_design_data(midnight_palette());
                initialized = true;
            }
            CentralPanel::default().show_inside(ctx, |ui| theme_demo(ui, &mut state));
        },
    )
}
