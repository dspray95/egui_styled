//! Stress test for `Styled::stack()` inside an auto-sizing centered area,
//! mirroring a game-over / scoreboard layout. Run with:
//! `cargo run --example stack`

use egui::{Align, Align2, Color32, Vec2};
use egui_styled::prelude::*;

fn ui(ctx: &egui::Context) {
    Styled::area()
        .id("stack_stress")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .bg(rgb(20, 20, 28))
        .padding(24.0)
        .corner_radius(8.0)
        .show(ctx, |ui| {
            Styled::column()
                .gap(8.0)
                .align(Align::Center)
                .show(ui, |ui| {
                    ui.label("scoreboard above");

                    // Offset chromatic stack.
                    Styled::stack()
                        .layer_offset(Vec2::new(-2.0, 0.0), |ui| {
                            Styled::label("[ENTER]")
                                .text_color(rgb(0, 220, 255))
                                .extend()
                                .show(ui);
                        })
                        .layer_offset(Vec2::new(2.0, 0.0), |ui| {
                            Styled::label("[ENTER]")
                                .text_color(rgb(255, 0, 200))
                                .extend()
                                .show(ui);
                        })
                        .layer(|ui| {
                            Styled::label("[ENTER]")
                                .text_color(Color32::WHITE)
                                .extend()
                                .show(ui);
                        })
                        .show(ui);

                    // Aligned overlay over a background layer.
                    Styled::stack()
                        .layer(|ui| {
                            Styled::label("BACKGROUND TEXT LAYER")
                                .text_color(rgb(60, 60, 80))
                                .extend()
                                .show(ui);
                        })
                        .layer_aligned(Align2::CENTER_CENTER, |ui| {
                            Styled::label("CENTERED")
                                .text_color(Color32::WHITE)
                                .extend()
                                .show(ui);
                        })
                        .show(ui);

                    ui.label("scoreboard below");
                });
        });
    ctx.request_repaint();
}

fn main() -> eframe::Result<()> {
    eframe::run_ui_native(
        "egui_styled - stack stress",
        eframe::NativeOptions::default(),
        |ctx, _| ui(ctx),
    )
}
