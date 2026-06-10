use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

fn layout_demo(ui: &mut egui::Ui) {
    let fg = Color32::from_rgb(220, 220, 230);
    let dim = Color32::from_rgb(140, 140, 160);
    let card_bg = rgb(40, 42, 58);
    let highlight = rgb(70, 100, 160);

    Styled::column()
        .gap(20.0)
        .padding(20.0)
        .show(ui, |ui| {
            // ── Spacer ────────────────────────────────────────────────────────
            Styled::label("Spacer")
                .font_size(16.0)
                .text_color(fg)
                .bold()
                .show(ui);
            Styled::label("[Left] ──spacer── [Right]")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);

            Styled::row()
                .full_width()
                .bg(card_bg)
                .corner_radius(6.0)
                .padding(10.0)
                .show(ui, |ui| {
                    Styled::label("Left").text_color(fg).show(ui);
                    Styled::spacer().show(ui);
                    Styled::label("Right").text_color(fg).show(ui);
                });

            Styled::label("[spacer] [Right-aligned]")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);
            Styled::row()
                .full_width()
                .bg(card_bg)
                .corner_radius(6.0)
                .padding(10.0)
                .show(ui, |ui| {
                    Styled::spacer().show(ui);
                    Styled::label("Right-aligned").text_color(fg).show(ui);
                });

            Styled::label("Toolbar: title + spacer + action buttons")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);
            Styled::row()
                .full_width()
                .gap(8.0)
                .bg(rgb(28, 30, 45))
                .corner_radius(6.0)
                .padding(10.0)
                .show(ui, |ui| {
                    Styled::label("My App").text_color(fg).bold().show(ui);
                    Styled::spacer().show(ui);
                    for label in ["File", "Edit", "View"] {
                        Styled::button(label)
                            .bg(Color32::TRANSPARENT)
                            .hover_bg(highlight)
                            .text_color(fg)
                            .corner_radius(4.0)
                            .show(ui);
                    }
                });

            ui.separator();

            // ── Percentage sizing ─────────────────────────────────────────────
            Styled::label("Percentage sizing")
                .font_size(16.0)
                .text_color(fg)
                .bold()
                .show(ui);

            Styled::label("width_pct(50) — each card is half the available width")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);
            Styled::row().gap(8.0).show(ui, |ui| {
                for label in ["Card A", "Card B"] {
                    Styled::frame()
                        .width_pct(50.0)
                        .bg(card_bg)
                        .corner_radius(6.0)
                        .padding(12.0)
                        .show(ui, |ui| {
                            Styled::label(label).text_color(fg).show(ui);
                        });
                }
            });

            Styled::label("width_pct(50).max_width(180) — responsive card, capped at 180px")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);
            Styled::row().gap(8.0).show(ui, |ui| {
                for label in ["Card A", "Card B"] {
                    Styled::frame()
                        .width_pct(50.0)
                        .max_width(180.0)
                        .bg(card_bg)
                        .corner_radius(6.0)
                        .padding(12.0)
                        .show(ui, |ui| {
                            Styled::label(label).text_color(fg).show(ui);
                        });
                }
            });

            Styled::label("width_pct on widgets — full-width input, 50% button")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);

            let mut dummy = String::from("Type here…");
            Styled::text_edit(&mut dummy)
                .width_pct(100.0)
                .bg(rgb(28, 30, 45))
                .corner_radius(4.0)
                .border(1.0, rgb(60, 65, 90))
                .show(ui);

            Styled::row().gap(8.0).show(ui, |ui| {
                Styled::button("Half-width")
                    .width_pct(50.0)
                    .bg(highlight)
                    .hover_bg(rgb(90, 120, 190))
                    .text_color(Color32::WHITE)
                    .corner_radius(4.0)
                    .show(ui);
                Styled::button("Natural")
                    .bg(card_bg)
                    .hover_bg(rgb(55, 58, 80))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            });

            ui.separator();

            // ── Wrapping rows ─────────────────────────────────────────────────
            Styled::label("Wrapping rows")
                .font_size(16.0)
                .text_color(fg)
                .bold()
                .show(ui);
            Styled::label("wrap() — tag cloud / chip bar, resize the window to see reflow")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);

            Styled::row()
                .full_width()
                .wrap()
                .gap(6.0)
                .bg(card_bg)
                .corner_radius(6.0)
                .padding(10.0)
                .show(ui, |ui| {
                    for tag in [
                        "egui", "rust", "ui", "layout", "flex", "wrap", "responsive",
                        "widgets", "buttons", "labels", "frames", "rows", "columns",
                    ] {
                        Styled::button(tag)
                            .bg(rgb(55, 65, 95))
                            .hover_bg(highlight)
                            .text_color(fg)
                            .corner_radius(12.0)
                            .padding(egui::Margin { left: 10, right: 10, top: 4, bottom: 4 })
                            .show(ui);
                    }
                });

            ui.separator();

            // ── Combining spacer + percentage ─────────────────────────────────
            Styled::label("Combining spacer + percentage")
                .font_size(16.0)
                .text_color(fg)
                .bold()
                .show(ui);
            Styled::label("Search bar at 60% width, centered via spacers")
                .font_size(12.0)
                .text_color(dim)
                .show(ui);

            let mut search = String::from("Search…");
            Styled::row()
                .full_width()
                .gap(8.0)
                .bg(card_bg)
                .corner_radius(6.0)
                .padding(10.0)
                .show(ui, |ui| {
                    Styled::spacer().show(ui);
                    Styled::text_edit(&mut search)
                        .width_pct(60.0)
                        .bg(rgb(28, 30, 45))
                        .corner_radius(4.0)
                        .border(1.0, rgb(60, 65, 90))
                        .show(ui);
                    Styled::spacer().show(ui);
                });
        });
}

fn main() -> eframe::Result<()> {
    eframe::run_ui_native(
        "egui_styled layout",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([640.0, 700.0]),
            ..Default::default()
        },
        |ctx, _| {
            CentralPanel::default().show_inside(ctx, layout_demo);
        },
    )
}
