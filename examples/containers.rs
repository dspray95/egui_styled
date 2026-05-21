use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

fn containers_example(ui: &mut egui::Ui) {
    ui.heading("Containers Demo");
    ui.add_space(12.0);

    Styled::column()
        .gap(8.0)
        .bg(rgb(28, 28, 32))
        .corner_radius(8.0)
        .padding(16.0)
        .border(1.0, rgb(60, 60, 70))
        .show(ui, |ui| {
            ui.label("Outer column: gap=8, padded, bordered.");

            Styled::row().gap(12.0).show(ui, |ui| {
                for label in ["One", "Two", "Three"] {
                    Styled::button(label)
                        .bg(rgb(50, 50, 80))
                        .hover_bg(rgb(70, 70, 110))
                        .text_color(Color32::WHITE)
                        .corner_radius(4.0)
                        .show(ui);
                }
            });

            Styled::row()
                .gap(8.0)
                .bg(rgb(40, 40, 50))
                .corner_radius(6.0)
                .padding(10.0)
                .show(ui, |ui| {
                    ui.label("Styled row with its own bg+padding:");
                    Styled::button("Action")
                        .bg(rgb(80, 140, 90))
                        .text_color(Color32::WHITE)
                        .corner_radius(4.0)
                        .show(ui);
                });

            Styled::column().gap(4.0).show(ui, |ui| {
                for i in 1..=3 {
                    ui.label(format!("Nested column item {}", i));
                }
            });
        });
}

fn main() -> eframe::Result<()> {
    eframe::run_simple_native(
        "egui_styled containers",
        eframe::NativeOptions::default(),
        |ctx, _| {
            CentralPanel::default().show(ctx, |ui| containers_example(ui));
        },
    )
}
