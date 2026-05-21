use egui::{CentralPanel, Color32};
use egui_styled::{Styled, rgb};

fn basic_example(ui: &mut egui::Ui) {
    ui.heading("egui styled - Basic Demo");

    Styled::frame()
        .bg(rgb(35, 35, 35))
        .corner_radius(8.0)
        .padding(16.0)
        .border(1.0, rgb(60, 60, 60))
        .show(ui, |ui| {
            ui.label("This is inside a Styled::frame()");
            ui.label("Note the custom background, rounding, and border.");

            ui.add_space(8.0);

            let clicked = Styled::button("Primary Action")
                .bg(rgb(60, 60, 255))
                .hover_bg(rgb(90, 90, 255))
                .text_color(Color32::WHITE)
                .corner_radius(4.0)
                .full_width()
                .show(ui)
                .clicked();

            if clicked {
                println!("Primary action clicked!");
            }
        });

    ui.add_space(20.0);

    for i in 1..=3 {
        Styled::button(format!("List Item {}", i))
            .bg(Color32::TRANSPARENT)
            .border(1.0, rgb(100, 100, 100))
            .margin_top(8)
            .show(ui);
    }

    ui.add_space(20.0);

    ui.group(|ui| {
        ui.label("Raw egui Group:");
        if ui.button("Standard egui Button").clicked() {}
    });
}

fn main() -> eframe::Result<()> {
    eframe::run_simple_native(
        "egui_styled basic",
        eframe::NativeOptions::default(),
        |ctx, _| {
            CentralPanel::default().show(ctx, |ui| basic_example(ui));
        },
    )
}
