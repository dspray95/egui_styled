use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

#[derive(Default)]
struct AppState {
    username: String,
    email: String,
}

fn primary_button(
    t: &StyledTheme,
    p: &WebPalette,
) -> impl Fn(StyledButton) -> StyledButton + 'static {
    let p = p.clone();
    let t = t.clone();
    move |b| {
        b.bg(p.accent)
            .hover_bg(p.accent_hover)
            .active_bg(p.accent_active)
            .text_color(p.fg_on_accent)
            .corner_radius(t.rounding_md)
    }
}

fn secondary_button(
    t: &StyledTheme,
    p: &WebPalette,
) -> impl Fn(StyledButton) -> StyledButton + 'static {
    let p = p.clone();
    let t = t.clone();
    move |b| {
        b.bg(Color32::TRANSPARENT)
            .hover_bg(p.bg_elevated)
            .text_color(p.fg_muted)
            .corner_radius(t.rounding_md)
            .border(1.0, p.border)
    }
}

fn input(
    t: &StyledTheme,
    p: &WebPalette,
) -> impl for<'a> Fn(StyledTextEdit<'a>) -> StyledTextEdit<'a> + 'static {
    let p = p.clone();
    let t = t.clone();
    move |s| {
        s.full_width()
            .bg(p.bg_secondary)
            .corner_radius(t.rounding_md)
            .border(1.0, p.border)
            .focus_border(1.0, p.border_focus)
    }
}

fn card(t: &StyledTheme, p: &WebPalette) -> impl Fn(StyledFrame) -> StyledFrame + 'static {
    let p = p.clone();
    let t = t.clone();
    move |f| {
        f.bg(p.bg_surface)
            .corner_radius(t.rounding_lg)
            .padding(t.spacing_lg)
            .border(1.0, p.border)
    }
}

fn composable_styles_example(ui: &mut egui::Ui, state: &mut AppState) {
    let t = ui.ctx().styled_theme();
    let p = ui.ctx().design_data::<WebPalette>();

    Styled::frame().apply(card(&t, &p)).show(ui, |ui| {
        Styled::column().gap(t.spacing_md).show(ui, |ui| {
            Styled::label("Sign in")
                .font_size(t.font_size_lg)
                .text_color(p.accent)
                .bold()
                .show(ui);

            Styled::text_edit(&mut state.username)
                .hint("Username")
                .apply(input(&t, &p))
                .show(ui);

            Styled::text_edit(&mut state.email)
                .hint("Email")
                .apply(input(&t, &p))
                .show(ui);

            Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                Styled::button("Cancel")
                    .apply(secondary_button(&t, &p))
                    .show(ui);
                Styled::button("Save")
                    .apply(primary_button(&t, &p))
                    .show(ui);
            });
        });
    });
}

/// Slate geometry - chunkier radii than default.
fn slate_geometry() -> StyledTheme {
    use egui::CornerRadius;
    StyledTheme {
        rounding_sm: CornerRadius::same(3),
        rounding_md: CornerRadius::same(6),
        rounding_lg: CornerRadius::same(12),
        ..Default::default()
    }
}

/// Slate background with a hot-pink accent - modern dashboard feel.
fn slate_pink_palette() -> WebPalette {
    WebPalette {
        bg_primary: Color32::from_rgb(18, 20, 28),
        bg_secondary: Color32::from_rgb(26, 28, 38),
        bg_surface: Color32::from_rgb(34, 36, 48),
        bg_elevated: Color32::from_rgb(46, 48, 62),

        fg_primary: Color32::from_rgb(230, 230, 240),
        fg_secondary: Color32::from_rgb(170, 175, 195),
        fg_muted: Color32::from_rgb(110, 115, 135),
        fg_on_accent: Color32::WHITE,

        accent: Color32::from_rgb(240, 70, 160),
        accent_hover: Color32::from_rgb(255, 100, 180),
        accent_active: Color32::from_rgb(210, 50, 135),

        error: Color32::from_rgb(230, 90, 100),
        warning: Color32::from_rgb(230, 175, 80),
        success: Color32::from_rgb(95, 200, 140),

        border: Color32::from_rgb(60, 62, 78),
        border_focus: Color32::from_rgb(255, 100, 180),
    }
}

fn main() -> eframe::Result<()> {
    let mut state = AppState::default();
    let mut initialized = false;
    eframe::run_ui_native(
        "egui_styled composable styles",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(slate_geometry());
                ctx.set_design_data(slate_pink_palette());
                initialized = true;
            }
            CentralPanel::default()
                .show(ctx, |ui| composable_styles_example(ui, &mut state));
        },
    )
}
