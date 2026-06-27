use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

// Demonstrates that `.visible(false)` hides only the widget it's called on,
// while still reserving its layout space. Toggle the checkbox to confirm the
// trailing widgets do NOT move and do NOT disappear when the middle widget is
// hidden.
struct App {
    hide_middle: bool,
    text: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            hide_middle: true,
            text: "editable".to_string(),
        }
    }
}

fn row_label(text: &str, visible: bool) -> StyledLabel {
    Styled::label(text)
        .bg(rgb(40, 40, 60))
        .text_color(Color32::WHITE)
        .corner_radius(4.0)
        .padding(8.0)
        .margin_top(6.0)
        .full_width()
        .visible(visible)
}

fn draw(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("egui_styled - visibility containment");
    ui.add_space(8.0);

    Styled::checkbox(&mut app.hide_middle, "Hide the middle widget")
        .text_color(Color32::WHITE)
        .show(ui);

    ui.add_space(12.0);

    // Three labels: the middle one toggles visibility. First and third must
    // always render, and the third must not shift position.
    ui.label("Labels:");
    row_label("First (always visible)", true).show(ui);
    row_label("Middle (toggles)", !app.hide_middle).show(ui);
    row_label("Third (always visible)", true).show(ui);

    ui.add_space(16.0);

    // Same check with a text_edit / button sandwiched between visible widgets.
    ui.label("Mixed widgets:");
    Styled::button("Top button")
        .bg(rgb(60, 60, 120))
        .text_color(Color32::WHITE)
        .full_width()
        .margin_top(6.0)
        .show(ui);

    Styled::text_edit(&mut app.text)
        .full_width()
        .margin_top(6.0)
        .visible(!app.hide_middle)
        .show(ui);

    Styled::button("Bottom button")
        .bg(rgb(60, 120, 60))
        .text_color(Color32::WHITE)
        .full_width()
        .margin_top(6.0)
        .show(ui);
}

fn main() -> eframe::Result<()> {
    let mut app = App::default();
    eframe::run_ui_native(
        "egui_styled visibility",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            CentralPanel::default().show(ctx, |ui| draw(&mut app, ui));
        },
    )
}
