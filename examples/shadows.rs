use egui::{CentralPanel, Color32, vec2};
use egui_styled::prelude::*;

fn shadows_example(ui: &mut egui::Ui) {
    ui.heading("egui_styled — Shadows");
    ui.add_space(24.0);

    // --- Glitch-style double stroke shadow on a button ---
    ui.label("Glitch outline button (.shadow twice):");
    ui.add_space(12.0);
    Styled::button("SUBMIT")
        .bg(Color32::BLACK)
        .hover_bg(rgb(20, 20, 20))
        .text_color(Color32::WHITE)
        .border(1.0, Color32::WHITE)
        .corner_radius(0.0)
        .padding(egui::Margin::symmetric(20, 10))
        .shadow(vec2(4.0, -3.0), 2.0, rgb(255, 0, 200)) // magenta offset
        .shadow(vec2(-3.0, 4.0), 2.0, Color32::WHITE) // white offset
        .show(ui);

    ui.add_space(32.0);

    // --- Conventional drop shadow on a card frame ---
    ui.label("Card with drop shadow (.shadow_filled):");
    ui.add_space(12.0);
    Styled::frame()
        .bg(rgb(40, 40, 40))
        .corner_radius(8.0)
        .padding(16.0)
        .border(1.0, rgb(70, 70, 70))
        .shadow_filled(vec2(4.0, 4.0), Color32::from_black_alpha(120))
        .show(ui, |ui| {
            ui.label("This card has a soft drop shadow.");
            ui.add_space(8.0);
            Styled::button("Action")
                .bg(rgb(60, 60, 255))
                .hover_bg(rgb(90, 90, 255))
                .text_color(Color32::WHITE)
                .corner_radius(4.0)
                .show(ui);
        });

    ui.add_space(32.0);

    // --- Stacked: fill + stroke on the same widget ---
    ui.label("Button with filled shadow and stroke shadow combined:");
    ui.add_space(12.0);
    Styled::button("SAVE")
        .bg(rgb(60, 60, 255))
        .hover_bg(rgb(90, 90, 255))
        .text_color(Color32::WHITE)
        .corner_radius(4.0)
        .padding(egui::Margin::symmetric(24, 10))
        .shadow_filled(vec2(3.0, 3.0), Color32::from_black_alpha(160))
        .shadow(vec2(3.0, 3.0), 1.0, rgb(30, 30, 180))
        .show(ui);
}

fn main() -> eframe::Result<()> {
    eframe::run_ui_native(
        "egui_styled shadows",
        eframe::NativeOptions::default(),
        |ctx, _| {
            CentralPanel::default().show(ctx, shadows_example);
        },
    )
}
