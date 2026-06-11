use egui::{CentralPanel, Color32};
use egui_styled::prelude::*;

// Gold palette for the "NEW GAME" fantasy button demo.
const GOLD_BRIGHT: Color32 = Color32::from_rgb(255, 215, 80);
const GOLD_MID: Color32 = Color32::from_rgb(212, 160, 40);
const GOLD_DIM: Color32 = Color32::from_rgb(130, 90, 20);
const DARK_BG: Color32 = Color32::from_rgb(22, 17, 8);
const DARK_BG_TOP: Color32 = Color32::from_rgb(38, 28, 12);

fn gradients_example(ui: &mut egui::Ui) {
    ui.heading("egui_styled — Gradients & Inner Glow");
    ui.add_space(24.0);

    // ── Fantasy "NEW GAME" button ─────────────────────────────────────────────
    ui.label("Fantasy menu button (bg_gradient + inner_glow + border_gradient):");
    ui.add_space(12.0);
    Styled::button("NEW GAME")
        // Dark parchment background with a subtle vertical gradient.
        .bg(DARK_BG)
        .bg_gradient_v(DARK_BG_TOP, DARK_BG)
        // Gold inner glow — bright at the rect edge, fading inward.
        .inner_glow(10.0, Color32::from_rgba_premultiplied(212, 175, 55, 180))
        // Gold gradient border — brighter at top, darker at bottom.
        .border_gradient(2.0, GOLD_BRIGHT, GOLD_DIM)
        // Hover: amplify the glow and brighten the gradient.
        .hover_bg_gradient_v(Color32::from_rgb(50, 38, 15), DARK_BG)
        .hover_inner_glow(16.0, Color32::from_rgba_premultiplied(255, 215, 80, 220))
        .hover_border_gradient(2.0, Color32::from_rgb(255, 230, 120), GOLD_MID)
        // Active: dim slightly.
        .active_bg_gradient_v(DARK_BG, Color32::from_rgb(15, 11, 4))
        .active_inner_glow(8.0, Color32::from_rgba_premultiplied(180, 140, 40, 160))
        .text_color(GOLD_BRIGHT)
        .hover_text_color(Color32::from_rgb(255, 235, 140))
        .corner_radius(4.0)
        .padding(egui::Margin::symmetric(32, 12))
        .font_size(18.0)
        .show(ui);

    ui.add_space(32.0);

    // ── Vertical gradient frame (card) ────────────────────────────────────────
    ui.label("Vertical gradient frame (.bg_gradient_v):");
    ui.add_space(12.0);
    Styled::frame()
        .bg_gradient_v(
            Color32::from_rgb(30, 60, 120),
            Color32::from_rgb(10, 20, 50),
        )
        .corner_radius(8.0)
        .padding(16.0)
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "Top → bottom gradient");
            ui.colored_label(Color32::from_gray(180), "Layered over a transparent bg");
        });

    ui.add_space(24.0);

    // ── Horizontal gradient ────────────────────────────────────────────────────
    ui.label("Horizontal gradient frame (.bg_gradient_h):");
    ui.add_space(12.0);
    Styled::frame()
        .bg_gradient_h(
            Color32::from_rgb(180, 40, 80),
            Color32::from_rgb(40, 80, 180),
        )
        .corner_radius(8.0)
        .padding(16.0)
        .full_width()
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "Left → right gradient");
        });

    ui.add_space(24.0);

    // ── Translucent gradient over solid bg ────────────────────────────────────
    ui.label("Translucent gradient over solid bg (layering demo):");
    ui.add_space(12.0);
    Styled::frame()
        .bg(Color32::from_rgb(40, 40, 40))
        .bg_gradient_v(
            Color32::from_rgba_premultiplied(255, 255, 255, 40), // faint highlight at top
            Color32::TRANSPARENT,
        )
        .corner_radius(8.0)
        .padding(16.0)
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "Solid bg + translucent gradient on top");
        });

    ui.add_space(24.0);

    // ── 4-corner gradient ─────────────────────────────────────────────────────
    ui.label("4-corner gradient (.bg_gradient):");
    ui.add_space(12.0);
    Styled::frame()
        .bg_gradient(
            Color32::from_rgb(200, 50, 50),  // top-left: red
            Color32::from_rgb(50, 200, 50),  // top-right: green
            Color32::from_rgb(50, 50, 200),  // bottom-left: blue
            Color32::from_rgb(200, 200, 50), // bottom-right: yellow
        )
        .corner_radius(8.0)
        .padding(16.0)
        .full_width()
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "Red / Green / Blue / Yellow corners");
        });

    ui.add_space(24.0);

    // ── Inner glow on a frame ─────────────────────────────────────────────────
    ui.label("Inner glow frame (10 px gold):");
    ui.add_space(12.0);
    Styled::frame()
        .bg(Color32::from_rgb(20, 20, 20))
        .inner_glow(10.0, Color32::from_rgba_premultiplied(212, 175, 55, 200))
        .border(1.0, GOLD_DIM)
        .corner_radius(6.0)
        .padding(20.0)
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "Inward glow — brightest at the edge");
        });

    ui.add_space(24.0);

    // ── Border gradient on a label ────────────────────────────────────────────
    ui.label("Border gradient on a label (top bright → bottom dim):");
    ui.add_space(12.0);
    Styled::label("Gradient border label")
        .bg(Color32::from_rgb(15, 15, 15))
        .text_color(Color32::WHITE)
        .border_gradient(
            2.0,
            Color32::from_rgb(200, 200, 200),
            Color32::from_rgb(60, 60, 60),
        )
        .corner_radius(4.0)
        .padding(egui::Margin::symmetric(16, 8))
        .show(ui);

    ui.add_space(24.0);

    // ── Per-state demo ────────────────────────────────────────────────────────
    ui.label("Per-state gradient button (hover to see gradient change):");
    ui.add_space(12.0);
    Styled::button("HOVER ME")
        .bg(Color32::from_rgb(40, 40, 40))
        .bg_gradient_v(Color32::from_rgb(60, 60, 60), Color32::from_rgb(30, 30, 30))
        .hover_bg_gradient_v(
            Color32::from_rgb(80, 120, 200),
            Color32::from_rgb(40, 60, 120),
        )
        .active_bg_gradient_v(
            Color32::from_rgb(40, 70, 160),
            Color32::from_rgb(20, 40, 90),
        )
        .inner_glow(0.0, Color32::TRANSPARENT)
        .hover_inner_glow(8.0, Color32::from_rgba_premultiplied(100, 150, 255, 150))
        .corner_radius(4.0)
        .padding(egui::Margin::symmetric(24, 10))
        .text_color(Color32::WHITE)
        .show(ui);

    ui.add_space(24.0);

    // ── N-stop rainbow gradient ────────────────────────────────────────────────
    ui.label("Rainbow (.bg_gradient_stops_h, N stops):");
    ui.add_space(12.0);
    let rainbow = [
        (0.0, Color32::from_rgb(228, 3, 3)),    // red
        (0.17, Color32::from_rgb(255, 140, 0)), // orange
        (0.33, Color32::from_rgb(255, 237, 0)), // yellow
        (0.5, Color32::from_rgb(0, 128, 38)),   // green
        (0.67, Color32::from_rgb(0, 77, 255)),  // blue
        (0.83, Color32::from_rgb(117, 7, 135)), // indigo
        (1.0, Color32::from_rgb(180, 40, 200)), // violet
    ];
    Styled::frame()
        .bg_gradient_stops_h(rainbow)
        .corner_radius(8.0)
        .padding(20.0)
        .full_width()
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "Seven-stop horizontal rainbow");
        });

    ui.add_space(24.0);

    // ── Per-side glow ──────────────────────────────────────────────────────────
    ui.label("Inner glow on top + bottom only (.inner_glow_y):");
    ui.add_space(12.0);
    Styled::frame()
        .bg(Color32::from_rgb(18, 18, 22))
        .inner_glow_y(12.0, Color32::from_rgba_premultiplied(120, 180, 255, 200))
        .corner_radius(6.0)
        .padding(20.0)
        .full_width()
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "Glow bands on the top and bottom edges");
        });

    ui.add_space(16.0);
    ui.label("Inner glow on the left edge only (.inner_glow_left):");
    ui.add_space(12.0);
    Styled::frame()
        .bg(Color32::from_rgb(18, 18, 22))
        .inner_glow_left(14.0, Color32::from_rgba_premultiplied(255, 140, 60, 220))
        .corner_radius(6.0)
        .padding(20.0)
        .full_width()
        .show(ui, |ui| {
            ui.colored_label(Color32::WHITE, "A single-side accent glow");
        });
}

fn main() -> eframe::Result<()> {
    eframe::run_ui_native(
        "egui_styled gradients",
        eframe::NativeOptions::default(),
        |ctx, _| {
            CentralPanel::default().show_inside(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, gradients_example);
            });
        },
    )
}
