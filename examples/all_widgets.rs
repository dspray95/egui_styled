use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

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
    let p = ui.ctx().design_data::<WebPalette>();

    Styled::frame()
        .bg(p.bg_surface)
        .corner_radius(t.rounding_lg)
        .padding(t.spacing_lg)
        .border(1.0, p.border)
        .show(ui, |ui| {
            Styled::column().gap(t.spacing_md).show(ui, |ui| {
                Styled::label("Account Settings")
                    .font_size(t.font_size_lg)
                    .text_color(p.fg_primary)
                    .bold()
                    .show(ui);

                Styled::label("Update your profile and preferences below.")
                    .font_size(t.font_size_sm)
                    .text_color(p.fg_muted)
                    .italics()
                    .show(ui);

                // --- Text inputs ---
                Styled::text_edit(&mut state.username)
                    .hint("Username")
                    .full_width()
                    .bg(p.bg_secondary)
                    .corner_radius(t.rounding_md)
                    .border(1.0, p.border)
                    .focus_border(1.0, p.border_focus)
                    .show(ui);

                Styled::text_edit(&mut state.bio)
                    .hint("Short bio")
                    .multiline()
                    .full_width()
                    .bg(p.bg_secondary)
                    .corner_radius(t.rounding_md)
                    .border(1.0, p.border)
                    .focus_border(1.0, p.border_focus)
                    .show(ui);

                // --- Slider ---
                Styled::slider(&mut state.volume, 0.0..=1.0)
                    .text("Volume")
                    .step(0.01)
                    .show(ui);

                // --- Checkboxes ---
                Styled::checkbox(&mut state.notifications, "Enable notifications")
                    .text_color(p.fg_secondary)
                    .show(ui);

                Styled::checkbox(&mut state.dark_mode, "Use dark mode")
                    .text_color(p.fg_secondary)
                    .show(ui);

                // --- Combo box ---
                Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                    Styled::label("Quality").text_color(p.fg_secondary).show(ui);

                    Styled::combo_box("quality_select", state.quality.label())
                        .width(120.0)
                        .bg(p.bg_secondary)
                        .corner_radius(t.rounding_md)
                        .border(1.0, p.border)
                        .show(ui, |ui| {
                            for q in [Quality::Low, Quality::Medium, Quality::High, Quality::Ultra]
                            {
                                ui.selectable_value(&mut state.quality, q, q.label());
                            }
                        });
                });

                // --- Button row ---
                Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                    Styled::button("Cancel")
                        .bg(Color32::TRANSPARENT)
                        .hover_bg(p.bg_elevated)
                        .text_color(p.fg_muted)
                        .corner_radius(t.rounding_md)
                        .border(1.0, p.border)
                        .show(ui);

                    if Styled::button("Save")
                        .bg(p.accent)
                        .hover_bg(p.accent_hover)
                        .active_bg(p.accent_active)
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

/// Warm coffee-shop terminal - tighter radii than default.
fn warm_terminal_geometry() -> StyledTheme {
    use egui::CornerRadius;
    StyledTheme {
        rounding_sm: CornerRadius::same(2),
        rounding_md: CornerRadius::same(3),
        rounding_lg: CornerRadius::same(5),
        ..Default::default()
    }
}

/// Warm coffee-shop terminal - dark browns with an amber accent.
fn warm_terminal_palette() -> WebPalette {
    WebPalette {
        bg_primary: Color32::from_rgb(28, 22, 18),
        bg_secondary: Color32::from_rgb(38, 30, 24),
        bg_surface: Color32::from_rgb(48, 38, 30),
        bg_elevated: Color32::from_rgb(60, 48, 38),

        fg_primary: Color32::from_rgb(240, 225, 200),
        fg_secondary: Color32::from_rgb(200, 180, 150),
        fg_muted: Color32::from_rgb(150, 130, 105),
        fg_on_accent: Color32::from_rgb(28, 22, 18),

        accent: Color32::from_rgb(220, 150, 60),
        accent_hover: Color32::from_rgb(240, 170, 80),
        accent_active: Color32::from_rgb(190, 125, 45),

        error: Color32::from_rgb(220, 90, 70),
        warning: Color32::from_rgb(230, 180, 70),
        success: Color32::from_rgb(150, 200, 90),

        border: Color32::from_rgb(80, 65, 50),
        border_focus: Color32::from_rgb(240, 170, 80),
    }
}

fn main() -> eframe::Result<()> {
    let mut state = AppState {
        volume: 0.5,
        ..Default::default()
    };
    let mut initialized = false;
    eframe::run_ui_native(
        "egui_styled all widgets",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(warm_terminal_geometry());
                ctx.set_design_data(warm_terminal_palette());
                initialized = true;
            }
            CentralPanel::default().show_inside(ctx, |ui| all_widgets(ui, &mut state));
        },
    )
}
