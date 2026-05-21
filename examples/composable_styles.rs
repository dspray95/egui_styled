use egui::{CentralPanel, Color32};
use egui_styled::containers::frame::StyledFrame;
use egui_styled::theme::StyledTheme;
use egui_styled::theme::theme_ext::ThemeExt;
use egui_styled::widgets::button::StyledButton;
use egui_styled::widgets::text_edit::StyledTextEdit;
use egui_styled::{Apply, Styled};

#[derive(Default)]
struct AppState {
    username: String,
    email: String,
}

fn primary_button(b: StyledButton, t: &StyledTheme) -> StyledButton {
    b.bg(t.accent)
        .hover_bg(t.accent_hover)
        .active_bg(t.accent_active)
        .text_color(Color32::WHITE)
        .corner_radius(t.rounding_md)
}

fn secondary_button(b: StyledButton, t: &StyledTheme) -> StyledButton {
    b.bg(Color32::TRANSPARENT)
        .hover_bg(t.bg_elevated)
        .text_color(t.fg_muted)
        .corner_radius(t.rounding_md)
        .border(1.0, t.border)
}

fn input<'a>(s: StyledTextEdit<'a>, t: &StyledTheme) -> StyledTextEdit<'a> {
    s.full_width()
        .bg(t.bg_secondary)
        .corner_radius(t.rounding_md)
        .border(1.0, t.border)
        .focus_border(1.0, t.border_focus)
}

fn card(f: StyledFrame, t: &StyledTheme) -> StyledFrame {
    f.bg(t.bg_surface)
        .corner_radius(t.rounding_lg)
        .padding(t.spacing_lg)
        .border(1.0, t.border)
}

fn composable_styles_example(ui: &mut egui::Ui, state: &mut AppState) {
    let t = ui.ctx().styled_theme();

    Styled::frame().apply(|f| card(f, &t)).show(ui, |ui| {
        Styled::column().gap(t.spacing_md).show(ui, |ui| {
            Styled::text_edit(&mut state.username)
                .hint("Username")
                .apply(|s| input(s, &t))
                .show(ui);

            Styled::text_edit(&mut state.email)
                .hint("Email")
                .apply(|s| input(s, &t))
                .show(ui);

            Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                Styled::button("Cancel")
                    .apply(|b| secondary_button(b, &t))
                    .show(ui);

                Styled::button("Save")
                    .apply(|b| primary_button(b, &t))
                    .show(ui);
            });
        });
    });
}

fn main() -> eframe::Result<()> {
    let mut state = AppState::default();
    let mut initialized = false;
    eframe::run_simple_native(
        "egui_styled composable styles",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(StyledTheme::dark());
                initialized = true;
            }
            CentralPanel::default().show(ctx, |ui| composable_styles_example(ui, &mut state));
        },
    )
}
