use egui::{CentralPanel, Color32};
use egui_styled::Styled;
use egui_styled::theme::StyledTheme;
use egui_styled::theme::theme_ext::ThemeExt;

#[derive(Default)]
struct AppState {
    use_light: bool,
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
                    if Styled::button(if state.use_light { "Switch to dark" } else { "Switch to light" })
                        .bg(theme.accent)
                        .hover_bg(theme.accent_hover)
                        .active_bg(theme.accent_active)
                        .text_color(Color32::WHITE)
                        .corner_radius(theme.rounding_md)
                        .show(ui)
                        .clicked()
                    {
                        state.use_light = !state.use_light;
                        let next = if state.use_light {
                            StyledTheme::light()
                        } else {
                            StyledTheme::dark()
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
                    swatch(ui, "primary", theme.bg_primary, &theme);
                    swatch(ui, "secondary", theme.bg_secondary, &theme);
                    swatch(ui, "surface", theme.bg_surface, &theme);
                    swatch(ui, "elevated", theme.bg_elevated, &theme);
                });

                Styled::row().gap(theme.spacing_sm).show(ui, |ui| {
                    swatch(ui, "accent", theme.accent, &theme);
                    swatch(ui, "error", theme.error, &theme);
                    swatch(ui, "warning", theme.warning, &theme);
                    swatch(ui, "success", theme.success, &theme);
                });
            });
        });
}

fn swatch(ui: &mut egui::Ui, name: &str, color: Color32, theme: &StyledTheme) {
    Styled::frame()
        .bg(color)
        .border(1.0, theme.border)
        .corner_radius(theme.rounding_sm)
        .padding(theme.spacing_sm)
        .show(ui, |ui| {
            ui.label(name);
        });
}

fn main() -> eframe::Result<()> {
    let mut state = AppState::default();
    let mut initialized = false;
    eframe::run_simple_native(
        "egui_styled theme demo",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(StyledTheme::dark());
                initialized = true;
            }
            CentralPanel::default().show(ctx, |ui| theme_demo(ui, &mut state));
        },
    )
}
