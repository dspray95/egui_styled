use egui::{CentralPanel, Color32};
use egui_styled::Styled;
use egui_styled::theme::StyledTheme;
use egui_styled::theme::theme_ext::ThemeExt;

#[derive(Default)]
struct AppState {
    username: String,
    bio: String,
    volume: f32,
    notifications: bool,
    dark_mode: bool,
    quality: Quality,
}

#[derive(Default, PartialEq, Copy, Clone)]
enum Quality {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

impl Quality {
    fn label(self) -> &'static str {
        match self {
            Quality::Low => "Low",
            Quality::Medium => "Medium",
            Quality::High => "High",
            Quality::Ultra => "Ultra",
        }
    }
}

fn all_widgets(ui: &mut egui::Ui, state: &mut AppState) {
    let t = ui.ctx().styled_theme();

    Styled::frame()
        .bg(t.bg_surface)
        .corner_radius(t.rounding_lg)
        .padding(t.spacing_lg)
        .border(1.0, t.border)
        .show(ui, |ui| {
            Styled::column().gap(t.spacing_md).show(ui, |ui| {
                Styled::label("Account Settings")
                    .font_size(t.font_size_lg)
                    .text_color(t.fg_primary)
                    .bold()
                    .show(ui);

                Styled::label("Update your profile and preferences below.")
                    .font_size(t.font_size_sm)
                    .text_color(t.fg_muted)
                    .italics()
                    .show(ui);

                // --- Text inputs ---
                Styled::text_edit(&mut state.username)
                    .hint("Username")
                    .full_width()
                    .bg(t.bg_secondary)
                    .corner_radius(t.rounding_md)
                    .border(1.0, t.border)
                    .focus_border(1.0, t.border_focus)
                    .show(ui);

                Styled::text_edit(&mut state.bio)
                    .hint("Short bio")
                    .multiline()
                    .full_width()
                    .bg(t.bg_secondary)
                    .corner_radius(t.rounding_md)
                    .border(1.0, t.border)
                    .focus_border(1.0, t.border_focus)
                    .show(ui);

                // --- Slider ---
                Styled::slider(&mut state.volume, 0.0..=1.0)
                    .text("Volume")
                    .step(0.01)
                    .show(ui);

                // --- Checkboxes ---
                Styled::checkbox(&mut state.notifications, "Enable notifications")
                    .text_color(t.fg_secondary)
                    .show(ui);

                Styled::checkbox(&mut state.dark_mode, "Use dark mode")
                    .text_color(t.fg_secondary)
                    .show(ui);

                // --- Combo box ---
                Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                    Styled::label("Quality")
                        .text_color(t.fg_secondary)
                        .show(ui);

                    Styled::combo_box("quality_select", state.quality.label())
                        .width(120.0)
                        .bg(t.bg_secondary)
                        .corner_radius(t.rounding_md)
                        .border(1.0, t.border)
                        .show(ui, |ui| {
                            for q in [Quality::Low, Quality::Medium, Quality::High, Quality::Ultra] {
                                ui.selectable_value(&mut state.quality, q, q.label());
                            }
                        });
                });

                // --- Button row ---
                Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                    Styled::button("Cancel")
                        .bg(Color32::TRANSPARENT)
                        .hover_bg(t.bg_elevated)
                        .text_color(t.fg_muted)
                        .corner_radius(t.rounding_md)
                        .border(1.0, t.border)
                        .show(ui);

                    if Styled::button("Save")
                        .bg(t.accent)
                        .hover_bg(t.accent_hover)
                        .active_bg(t.accent_active)
                        .text_color(Color32::WHITE)
                        .corner_radius(t.rounding_md)
                        .show(ui)
                        .clicked()
                    {
                        println!("saved: {:?}", state.quality.label());
                    }
                });
            });
        });
}

fn main() -> eframe::Result<()> {
    let mut state = AppState {
        volume: 0.5,
        ..Default::default()
    };
    let mut initialized = false;
    eframe::run_simple_native(
        "egui_styled all widgets",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(StyledTheme::dark());
                initialized = true;
            }
            CentralPanel::default().show(ctx, |ui| all_widgets(ui, &mut state));
        },
    )
}
