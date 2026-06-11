use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

fn layout_demo(ui: &mut egui::Ui) {
    let fg = Color32::from_rgb(220, 220, 230);
    let dim = Color32::from_rgb(140, 140, 160);
    let card_bg = rgb(40, 42, 58);
    let highlight = rgb(70, 100, 160);

    Styled::column().gap(20.0).padding(20.0).show(ui, |ui| {
        // ── Edge-pinning (space_between) ───────────────────────────────────
        // For [Left] ........ [Right] use `space_between()`: it measures the
        // content and sizes the gap so nothing overflows. A bare spacer
        // (`Styled::spacer()`) consumes ALL remaining space, so trailing
        // content would be pushed past the right edge.
        Styled::label("Edge-pinning")
            .font_size(16.0)
            .text_color(fg)
            .bold()
            .show(ui);
        Styled::label("space_between - [Left] ........ [Right]")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        Styled::row()
            .full_width()
            .bg(card_bg)
            .corner_radius(6.0)
            .padding(10.0)
            .space_between()
            .item(|ui| {
                Styled::label("Left").text_color(fg).show(ui);
            })
            .item(|ui| {
                Styled::label("Right").text_color(fg).show(ui);
            })
            .show(ui);

        Styled::label("Toolbar: title pinned left, actions grouped right")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);
        Styled::row()
            .full_width()
            .bg(rgb(28, 30, 45))
            .corner_radius(6.0)
            .padding(10.0)
            .space_between()
            .item(|ui| {
                Styled::label("My App").text_color(fg).bold().show(ui);
            })
            .item(|ui| {
                // The right-hand group is itself a row of buttons.
                Styled::row().gap(8.0).show(ui, |ui| {
                    for label in ["File", "Edit", "View"] {
                        Styled::button(label)
                            .bg(Color32::TRANSPARENT)
                            .hover_bg(highlight)
                            .text_color(fg)
                            .corner_radius(4.0)
                            .show(ui);
                    }
                });
            })
            .show(ui);

        ui.separator();

        // ── Percentage sizing ─────────────────────────────────────────────
        Styled::label("Percentage sizing")
            .font_size(16.0)
            .text_color(fg)
            .bold()
            .show(ui);

        Styled::label("width_pct(50) - each card is half the available width")
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

        Styled::label("width_pct(50).max_width(180) - responsive card, capped at 180px")
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

        Styled::label("width_pct on widgets - full-width input, 50% button")
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
        Styled::label("wrap() - tag cloud / chip bar, resize the window to see reflow")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        let mut tag_cloud = Styled::row()
            .full_width()
            .gap(6.0)
            .bg(card_bg)
            .corner_radius(6.0)
            .padding(10.0)
            .wrap();
        for tag in [
            "egui",
            "rust",
            "ui",
            "layout",
            "flex",
            "wrap",
            "responsive",
            "widgets",
            "buttons",
            "labels",
            "frames",
            "rows",
            "columns",
        ] {
            tag_cloud = tag_cloud.item(move |ui| {
                Styled::button(tag)
                    .bg(rgb(55, 65, 95))
                    .hover_bg(highlight)
                    .text_color(fg)
                    .corner_radius(12.0)
                    .padding(egui::Margin {
                        left: 10,
                        right: 10,
                        top: 4,
                        bottom: 4,
                    })
                    .show(ui);
            });
        }
        tag_cloud.show(ui);

        ui.separator();

        // ── Distribution ──────────────────────────────────────────────────
        Styled::label("Distribution")
            .font_size(16.0)
            .text_color(fg)
            .bold()
            .show(ui);
        Styled::label("space_between - ends pinned, gaps grow to fill")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        Styled::row()
            .full_width()
            .bg(card_bg)
            .corner_radius(6.0)
            .padding(10.0)
            .space_between()
            .item(|ui| {
                Styled::button("Home")
                    .bg(highlight)
                    .text_color(egui::Color32::WHITE)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .item(|ui| {
                Styled::button("About")
                    .bg(rgb(50, 55, 75))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .item(|ui| {
                Styled::button("Blog")
                    .bg(rgb(50, 55, 75))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .item(|ui| {
                Styled::button("Contact")
                    .bg(rgb(50, 55, 75))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .show(ui);

        Styled::label("space_around - equal margin each side of each item")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        Styled::row()
            .full_width()
            .bg(card_bg)
            .corner_radius(6.0)
            .padding(10.0)
            .space_around()
            .item(|ui| {
                Styled::label("Stat A").text_color(fg).bold().show(ui);
            })
            .item(|ui| {
                Styled::label("Stat B").text_color(fg).bold().show(ui);
            })
            .item(|ui| {
                Styled::label("Stat C").text_color(fg).bold().show(ui);
            })
            .show(ui);

        Styled::label("space_evenly - equal space everywhere (before, between, after)")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        Styled::row()
            .full_width()
            .bg(card_bg)
            .corner_radius(6.0)
            .padding(10.0)
            .space_evenly()
            .item(|ui| {
                Styled::button("A")
                    .bg(rgb(80, 50, 110))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .item(|ui| {
                Styled::button("B")
                    .bg(rgb(80, 50, 110))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .item(|ui| {
                Styled::button("C")
                    .bg(rgb(80, 50, 110))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .item(|ui| {
                Styled::button("D")
                    .bg(rgb(80, 50, 110))
                    .text_color(fg)
                    .corner_radius(4.0)
                    .show(ui);
            })
            .show(ui);

        ui.separator();

        // ── Aspect ratio ──────────────────────────────────────────────────
        Styled::label("Aspect ratio")
            .font_size(16.0)
            .text_color(fg)
            .bold()
            .show(ui);
        Styled::label("width_pct(60).aspect_ratio(16.0/9.0) — resize to see height track width")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        Styled::frame()
            .width_pct(60.0)
            .aspect_ratio(16.0 / 9.0)
            .bg(card_bg)
            .corner_radius(6.0)
            .show(ui, |ui| {
                Styled::label("16 : 9").text_color(dim).show(ui);
            });

        Styled::label("width_pct(30).aspect_ratio(1.0) — square frame")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        Styled::frame()
            .width_pct(30.0)
            .aspect_ratio(1.0)
            .bg(card_bg)
            .corner_radius(6.0)
            .show(ui, |ui| {
                Styled::label("1 : 1").text_color(dim).show(ui);
            });

        ui.separator();

        // ── Combining alignment + percentage ──────────────────────────────
        Styled::label("Combining alignment + percentage")
            .font_size(16.0)
            .text_color(fg)
            .bold()
            .show(ui);
        Styled::label("Search bar at 60% width, centered via column align(Center)")
            .font_size(12.0)
            .text_color(dim)
            .show(ui);

        let mut search = String::from("Search…");
        Styled::frame()
            .full_width()
            .bg(card_bg)
            .corner_radius(6.0)
            .padding(10.0)
            .show(ui, |ui| {
                Styled::column()
                    .full_width()
                    .align(egui::Align::Center)
                    .show(ui, |ui| {
                        Styled::text_edit(&mut search)
                            .width_pct(60.0)
                            .bg(rgb(28, 30, 45))
                            .corner_radius(4.0)
                            .border(1.0, rgb(60, 65, 90))
                            .show(ui);
                    });
            });
    });
}

fn main() -> eframe::Result<()> {
    eframe::run_ui_native(
        "egui_styled layout",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 700.0]),
            ..Default::default()
        },
        |ctx, _| {
            CentralPanel::default().show_inside(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, layout_demo);
            });
        },
    )
}
