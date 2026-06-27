//! Example: declarative text effects — shadow, outline, glow, and scale.
//!
//! This example shows all four [`StyledLabel`] appearance primitives introduced
//! in egui_styled 0.4. Effects are *static* — they express how text looks on a
//! given frame. Animation (the scale punch curve, glow intensity over time) is
//! intentionally consumer-side: see the animated section below where the values
//! come from `ui.input(|i| i.time)`.
//!
//! Run with: `cargo run --example text_effects`

use egui::{Align2, CentralPanel, Color32, FontFamily, vec2};
use egui_styled::prelude::*;

fn text_effects_ui(ui: &mut egui::Ui, theme: &StyledTheme) {
    let t = ui.input(|i| i.time) as f32;
    let bg = Color32::from_rgb(8, 4, 16);

    egui::Frame::new().fill(bg).show(ui, |ui| {
        Styled::column().gap(theme.spacing_xl).show(ui, |ui| {
            heading(ui, theme, "TEXT SHADOW");
            shadow_examples(ui, theme, t);

            heading(ui, theme, "OUTLINE");
            outline_examples(ui, theme);

            heading(ui, theme, "GLOW");
            glow_examples(ui, theme, t);

            heading(ui, theme, "SCALE");
            scale_examples(ui, theme, t);

            heading(ui, theme, "COMPOSING EFFECTS");
            composed_examples(ui, theme, t);
        });
    });

    ui.ctx().request_repaint();
}

fn heading(ui: &mut egui::Ui, theme: &StyledTheme, text: &str) {
    Styled::label(text)
        .font(theme.font_display(theme.font_size_sm))
        .text_color(Color32::from_gray(100))
        .show(ui);
}

fn shadow_examples(ui: &mut egui::Ui, theme: &StyledTheme, _t: f32) {
    let cyan = Color32::from_rgb(0, 220, 255);
    let magenta = Color32::from_rgb(255, 0, 200);
    let gold = Color32::from_rgb(255, 215, 0);

    Styled::column().gap(theme.spacing_sm).show(ui, |ui| {
        // Drop shadow: use a dim version of the text color so it reads on dark bg.
        // Black shadows are invisible on dark backgrounds — color them instead.
        Styled::label("Drop shadow  (theme.shadow_md, dim gold)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(gold)
            .text_shadow(theme.shadow_md, Color32::from_rgb(80, 60, 0))
            .show(ui);

        // Hard offset — use a contrasting dim color so the offset is obvious
        Styled::label("Hard shadow  (shadow_lg, dim magenta)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::WHITE)
            .text_shadow(theme.shadow_lg, Color32::from_rgb(100, 0, 80))
            .show(ui);

        // Chromatic aberration — two opposite shadows
        Styled::label("Chromatic aberration  (two shadows)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::WHITE)
            .text_shadow(vec2(-2.0, 0.0), cyan)
            .text_shadow(vec2(2.0, 0.0), magenta)
            .show(ui);

        // Neon-style colored shadow — offset copy slightly behind the main text
        Styled::label("Neon drop  (colored shadow)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(cyan)
            .text_shadow(theme.shadow_md, Color32::from_rgb(0, 80, 120))
            .show(ui);
    });
}

fn outline_examples(ui: &mut egui::Ui, theme: &StyledTheme) {
    Styled::column().gap(theme.spacing_sm).show(ui, |ui| {
        Styled::label("Thin outline  (1.0px)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::WHITE)
            .outline(1.0, Color32::BLACK)
            .show(ui);

        Styled::label("Bold outline  (2.5px)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::from_rgb(255, 215, 0))
            .outline(2.5, Color32::from_rgb(120, 80, 0))
            .show(ui);

        Styled::label("Colored outline")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::from_rgb(8, 4, 16))
            .outline(2.0, Color32::from_rgb(0, 220, 255))
            .show(ui);
    });
}

fn glow_examples(ui: &mut egui::Ui, theme: &StyledTheme, t: f32) {
    // Animate intensity with a slow sine wave to show the consumer-driven model.
    // In a real UI you would tie this to a game event (e.g. a score pulse).
    let intensity_anim = (t * 1.5).sin() * 0.4 + 0.6; // 0.2 .. 1.0

    Styled::column().gap(theme.spacing_sm).show(ui, |ui| {
        Styled::label("Static glow  (intensity = 0.6)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::WHITE)
            .glow(Color32::from_rgb(0, 220, 255), theme.glow_md, 0.6)
            .show(ui);

        Styled::label("Animated glow  (intensity from sin wave)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::WHITE)
            .glow(
                Color32::from_rgb(0, 220, 255),
                theme.glow_lg,
                intensity_anim,
            )
            .show(ui);

        // Warm glow — different color
        Styled::label("Warm glow")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::from_rgb(255, 215, 0))
            .glow(Color32::from_rgb(255, 140, 0), theme.glow_md, 0.7)
            .show(ui);
    });
}

fn scale_examples(ui: &mut egui::Ui, theme: &StyledTheme, t: f32) {
    // Animate a scale punch — ease-out-back style using abs(sin).
    // The curve stays consumer-side; the label just takes the current factor.
    let scale_anim = 1.0 + (t * 1.2).sin().abs() * 0.3; // 1.0 .. 1.3

    let resting = egui::vec2(280.0, 40.0);

    Styled::column().gap(theme.spacing_sm).show(ui, |ui| {
        // Static scale — demonstrates layout stays stable
        Styled::label("Static scale  (1.3×)")
            .font(theme.font_display(theme.font_size_lg))
            .text_color(Color32::WHITE)
            .scale(1.3, Align2::LEFT_CENTER)
            .extend()
            .show(ui);

        // Animated scale inside layer_fixed so the column doesn't jump
        Styled::stack()
            .layer_fixed(resting, Align2::CENTER_CENTER, |ui| {
                Styled::label("Animated scale  (center pivot)")
                    .font(theme.font_display(theme.font_size_lg))
                    .text_color(Color32::from_rgb(0, 220, 255))
                    .scale(scale_anim, Align2::CENTER_CENTER)
                    .extend()
                    .show(ui);
            })
            .show(ui);
    });
}

fn composed_examples(ui: &mut egui::Ui, theme: &StyledTheme, t: f32) {
    let cyan = Color32::from_rgb(0, 220, 255);
    let magenta = Color32::from_rgb(255, 0, 200);
    let intensity = (t * 2.0).sin() * 0.3 + 0.5;

    Styled::column().gap(theme.spacing_sm).show(ui, |ui| {
        // Glow + chromatic shadow
        Styled::label("GAME OVER")
            .font(theme.font_display(theme.font_size_xl))
            .text_color(Color32::WHITE)
            .text_shadow(vec2(-1.5, 0.0), cyan)
            .text_shadow(vec2(1.5, 0.0), magenta)
            .glow(cyan, theme.glow_lg, intensity)
            .show(ui);

        // Outline + drop shadow
        Styled::label("HIGH SCORE")
            .font(theme.font_display(theme.font_size_xl))
            .text_color(Color32::from_rgb(255, 215, 0))
            .outline(2.0, Color32::from_rgb(120, 80, 0))
            .text_shadow(theme.shadow_md, Color32::from_black_alpha(180))
            .show(ui);
    });
}

fn main() -> eframe::Result<()> {
    let mut initialized = false;
    eframe::run_ui_native(
        "egui_styled — text effects",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(StyledTheme {
                    font_family_display: FontFamily::Proportional,
                    font_size_xl: 36.0,
                    ..Default::default()
                });
                initialized = true;
            }
            let theme = ctx.styled_theme();
            CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    text_effects_ui(ui, &theme);
                });
            });
        },
    )
}
