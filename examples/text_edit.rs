use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

#[derive(Default)]
struct App {
    name: String,
    password: String,
    bio: String,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("StyledTextEdit demo");

            Styled::frame()
                .bg(rgb(30, 30, 30))
                .corner_radius(8.0)
                .padding(16.0)
                .show(ui, |ui| {
                    ui.label("Name");
                    Styled::text_edit(&mut self.name)
                        .hint("Jane Doe")
                        .bg(rgb(45, 45, 45))
                        .text_color(Color32::WHITE)
                        .border(1.0, rgb(70, 70, 70))
                        .focus_border(2.0, rgb(80, 140, 255))
                        .corner_radius(4.0)
                        .full_width()
                        .margin_bottom(8)
                        .show(ui);

                    ui.label("Password");
                    Styled::text_edit(&mut self.password)
                        .password()
                        .hint("••••••")
                        .bg(rgb(45, 45, 45))
                        .text_color(Color32::WHITE)
                        .border(1.0, rgb(70, 70, 70))
                        .focus_border(2.0, rgb(80, 140, 255))
                        .corner_radius(4.0)
                        .full_width()
                        .margin_bottom(8)
                        .show(ui);

                    ui.label("Bio");
                    Styled::text_edit(&mut self.bio)
                        .multiline()
                        .hint("Tell us about yourself...")
                        .bg(rgb(45, 45, 45))
                        .text_color(Color32::WHITE)
                        .border(1.0, rgb(70, 70, 70))
                        .focus_border(2.0, rgb(80, 140, 255))
                        .corner_radius(4.0)
                        .full_width()
                        .show(ui);
                });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "egui_styled text_edit",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
