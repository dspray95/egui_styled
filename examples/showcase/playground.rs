//! Tweak a card live and copy the exact builder code that renders it.

use egui::{Color32, Rect, Ui, UiBuilder, Vec2, pos2, vec2};
use egui_styled::prelude::*;

use crate::art::sunset;
use crate::code::highlight;
use crate::fonts::Weight;
use crate::theme::{Palette, card, eyebrow, ghost_button, primary_button, text};

#[derive(Clone, PartialEq)]
struct Look {
    bg: Color32,
    gradient: Option<Color32>,
    radius: f32,
    padding: f32,
    border_width: f32,
    border: Color32,
    shadow: Option<(Vec2, Color32)>,
    glow: f32,
    glow_color: Color32,
    blur: f32,
    text: Color32,
}

const PRESETS: [&str; 4] = ["Glass", "Neubrutal", "Soft", "Neon"];

fn preset(name: &str) -> Look {
    match name {
        "Neubrutal" => Look {
            bg: rgb(255, 210, 63),
            gradient: None,
            radius: 8.0,
            padding: 26.0,
            border_width: 3.0,
            border: Color32::BLACK,
            shadow: Some((vec2(8.0, 8.0), Color32::BLACK)),
            glow: 0.0,
            glow_color: Color32::WHITE,
            blur: 0.0,
            text: rgb(17, 17, 17),
        },
        "Soft" => Look {
            bg: rgb(252, 251, 255),
            gradient: Some(rgb(232, 226, 255)),
            radius: 28.0,
            padding: 30.0,
            border_width: 1.0,
            border: rgb(255, 255, 255),
            shadow: Some((vec2(0.0, 10.0), rgb(0, 0, 0).with_alpha(40))),
            glow: 0.0,
            glow_color: Color32::WHITE,
            blur: 0.0,
            text: rgb(35, 30, 60),
        },
        "Neon" => Look {
            bg: rgb(8, 10, 24).with_alpha(190),
            gradient: None,
            radius: 16.0,
            padding: 28.0,
            border_width: 2.0,
            border: rgb(0, 255, 200),
            shadow: None,
            glow: 18.0,
            glow_color: rgb(0, 255, 200).with_alpha(110),
            blur: 10.0,
            text: rgb(200, 255, 245),
        },
        _ => Look {
            bg: rgb(255, 255, 255).with_alpha(30),
            gradient: None,
            radius: 24.0,
            padding: 28.0,
            border_width: 1.0,
            border: rgb(255, 255, 255).with_alpha(90),
            shadow: None,
            glow: 14.0,
            glow_color: rgb(255, 255, 255).with_alpha(30),
            blur: 24.0,
            text: Color32::WHITE,
        },
    }
}

pub struct State {
    look: Look,
}

impl Default for State {
    fn default() -> Self {
        Self {
            look: preset("Glass"),
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut State) {
    let p = ui.ctx().design_data::<Palette>();
    let area = ui.max_rect().shrink2(vec2(24.0, 12.0));
    let controls = Rect::from_min_size(area.min, vec2(340.0, area.height()));
    let right = Rect::from_min_max(pos2(controls.right() + 16.0, area.top()), area.max);
    let stage = Rect::from_min_max(
        right.min,
        pos2(right.right(), right.top() + right.height() * 0.6),
    );
    let code = Rect::from_min_max(pos2(right.left(), stage.bottom() + 16.0), right.max);

    put(ui, controls, |ui| controls_panel(ui, &p, &mut state.look));
    put(ui, stage, |ui| stage_panel(ui, &p, &state.look));
    put(ui, code, |ui| code_panel(ui, &p, &state.look));
}

fn put(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    ui.scope_builder(UiBuilder::new().max_rect(rect), add);
}

/// The card under edit, built straight from `look`: the same calls `code_for` prints.
fn styled(look: &Look) -> StyledFrame {
    let mut f = Styled::frame();
    if look.blur > 0.0 {
        f = f.backdrop_blur(look.blur);
    }
    f = f.bg(look.bg);
    if let Some(bottom) = look.gradient {
        f = f.bg_gradient_v(look.bg, bottom);
    }
    f = f.corner_radius(look.radius).padding(look.padding);
    if look.border_width > 0.0 {
        f = f.border(look.border_width, look.border);
    }
    if let Some((offset, color)) = look.shadow {
        f = f.shadow_filled(offset, color);
    }
    if look.glow > 0.0 {
        f = f.inner_glow(look.glow, look.glow_color);
    }
    f
}

fn stage_panel(ui: &mut Ui, p: &Palette, look: &Look) {
    Styled::frame()
        .background_image(sunset())
        .background_image_fit(BackgroundImageFit::Cover)
        .border(p.border_width, p.border)
        .corner_radius(p.radius)
        .align(egui::Align::Center)
        .justify(egui::Align::Center)
        .full_width()
        .full_height()
        .show(ui, |ui| {
            styled(look)
                .min_width(380.0)
                .max_width(380.0)
                .show(ui, |ui| {
                    Styled::column().gap(10.0).show(ui, |ui| {
                        text("Weekend in the Alps", Weight::Bold, 22.0, look.text).show(ui);
                        text(
                            "Three nights, two passes, one very good view.",
                            Weight::Regular,
                            14.0,
                            look.text.gamma_multiply(0.8),
                        )
                        .show(ui);
                        ui.add_space(6.0);
                        Styled::button("Book trip")
                            .font(crate::fonts::font(Weight::Semibold, 14.0))
                            .bg(look.text)
                            .hover_bg(look.text.gamma_multiply(0.85))
                            .text_color(contrast(look.text))
                            .corner_radius((look.radius * 0.5).min(12.0))
                            .padding(egui::Margin::symmetric(16, 9))
                            .show(ui);
                    });
                });
        });
}

/// Black or white, whichever reads on `c`.
fn contrast(c: Color32) -> Color32 {
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    if 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32 > 150.0 {
        rgb(17, 17, 17)
    } else {
        Color32::WHITE
    }
}

fn controls_panel(ui: &mut Ui, p: &Palette, look: &mut Look) {
    Styled::frame()
        .apply(card(p))
        .full_width()
        .full_height()
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                Styled::column().gap(12.0).full_width().show(ui, |ui| {
                    text("Style playground", Weight::Semibold, 18.0, p.text).show(ui);
                    text(
                        "Every control is one builder call.",
                        Weight::Regular,
                        13.0,
                        p.text_muted,
                    )
                    .show(ui);

                    eyebrow("Presets", p).show(ui);
                    Styled::row().gap(8.0).show(ui, |ui| {
                        for name in PRESETS {
                            let b = Styled::button(name).padding(egui::Margin::symmetric(10, 6));
                            let b = if *look == preset(name) {
                                b.apply(primary_button(p))
                                    .padding(egui::Margin::symmetric(10, 6))
                            } else {
                                b.apply(ghost_button(p))
                                    .padding(egui::Margin::symmetric(10, 6))
                            };
                            if b.show(ui).clicked() {
                                *look = preset(name);
                            }
                        }
                    });

                    eyebrow("Fill", p).show(ui);
                    color_row(ui, p, "Background", &mut look.bg);
                    let mut gradient = look.gradient.is_some();
                    toggle(ui, p, "Vertical gradient", &mut gradient);
                    match (gradient, look.gradient) {
                        (true, None) => look.gradient = Some(look.bg.lighten(0.3)),
                        (false, Some(_)) => look.gradient = None,
                        _ => {}
                    }
                    if let Some(bottom) = &mut look.gradient {
                        color_row(ui, p, "Gradient end", bottom);
                    }
                    color_row(ui, p, "Text", &mut look.text);

                    eyebrow("Shape", p).show(ui);
                    slider(ui, p, "Corner radius", &mut look.radius, 0.0..=48.0);
                    slider(ui, p, "Padding", &mut look.padding, 0.0..=48.0);

                    eyebrow("Border", p).show(ui);
                    slider(ui, p, "Width", &mut look.border_width, 0.0..=6.0);
                    color_row(ui, p, "Color", &mut look.border);

                    eyebrow("Shadow", p).show(ui);
                    let mut shadow = look.shadow.is_some();
                    toggle(ui, p, "Hard offset shadow", &mut shadow);
                    match (shadow, look.shadow) {
                        (true, None) => look.shadow = Some((vec2(6.0, 6.0), Color32::BLACK)),
                        (false, Some(_)) => look.shadow = None,
                        _ => {}
                    }
                    if let Some((offset, color)) = &mut look.shadow {
                        slider(ui, p, "Offset x", &mut offset.x, -16.0..=16.0);
                        slider(ui, p, "Offset y", &mut offset.y, -16.0..=16.0);
                        color_row(ui, p, "Shadow color", color);
                    }

                    eyebrow("Effects", p).show(ui);
                    slider(ui, p, "Inner glow", &mut look.glow, 0.0..=32.0);
                    color_row(ui, p, "Glow color", &mut look.glow_color);
                    slider(ui, p, "Backdrop blur", &mut look.blur, 0.0..=48.0);
                });
            });
        });
}

fn labelled(ui: &mut Ui, p: &Palette, label: &str, control: impl FnOnce(&mut Ui)) {
    Styled::row()
        .full_width()
        .align(egui::Align::Center)
        .space_between()
        .item(|ui| {
            text(label, Weight::Medium, 13.0, p.text).show(ui);
        })
        .item(control)
        .show(ui);
}

fn slider(
    ui: &mut Ui,
    p: &Palette,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    labelled(ui, p, label, |ui| {
        Styled::slider(value, range)
            .step(1.0)
            .apply(crate::theme::slider(p))
            .show(ui);
    });
}

fn toggle(ui: &mut Ui, p: &Palette, label: &str, on: &mut bool) {
    labelled(ui, p, label, |ui| {
        Styled::checkbox(on, "").accent(p.accent).show(ui);
    });
}

fn color_row(ui: &mut Ui, p: &Palette, label: &str, color: &mut Color32) {
    labelled(ui, p, label, |ui| {
        ui.color_edit_button_srgba(color);
    });
}

fn code_panel(ui: &mut Ui, p: &Palette, look: &Look) {
    let src = code_for(look);
    Styled::frame()
        .apply(card(p))
        .full_width()
        .full_height()
        .show(ui, |ui| {
            Styled::column().gap(12.0).full_width().show(ui, |ui| {
                Styled::row()
                    .full_width()
                    .align(egui::Align::Center)
                    .space_between()
                    .item(|ui| {
                        text("The code for what you see", Weight::Semibold, 16.0, p.text).show(ui);
                    })
                    .item(|ui| {
                        if Styled::button("Copy")
                            .apply(ghost_button(p))
                            .padding(egui::Margin::symmetric(14, 6))
                            .show(ui)
                            .clicked()
                        {
                            ui.ctx().copy_text(src.clone());
                        }
                    })
                    .show(ui);
                Styled::frame()
                    .bg(rgb(13, 14, 18))
                    .border(p.border_width, p.border)
                    .corner_radius(p.radius * 0.7)
                    .padding(16.0)
                    .full_width()
                    .full_height()
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            Styled::label(highlight(&src, 13.0)).show(ui);
                        });
                    });
            });
        });
}

/// Mirrors `styled`: one line per builder call it made.
fn code_for(look: &Look) -> String {
    let mut lines = vec!["Styled::frame()".to_owned()];
    let mut call = |s: String| lines.push(format!("    .{s}"));
    if look.blur > 0.0 {
        call(format!("backdrop_blur({})", num(look.blur)));
    }
    call(format!("bg({})", color(look.bg)));
    if let Some(bottom) = look.gradient {
        call(format!(
            "bg_gradient_v({}, {})",
            color(look.bg),
            color(bottom)
        ));
    }
    call(format!("corner_radius({})", num(look.radius)));
    call(format!("padding({})", num(look.padding)));
    if look.border_width > 0.0 {
        call(format!(
            "border({}, {})",
            num(look.border_width),
            color(look.border)
        ));
    }
    if let Some((offset, c)) = look.shadow {
        call(format!(
            "shadow_filled(vec2({}, {}), {})",
            num(offset.x),
            num(offset.y),
            color(c)
        ));
    }
    if look.glow > 0.0 {
        call(format!(
            "inner_glow({}, {})",
            num(look.glow),
            color(look.glow_color)
        ));
    }
    lines.push("    .show(ui, |ui| {".to_owned());
    lines.push("        // your content".to_owned());
    lines.push("    });".to_owned());
    lines.join("\n")
}

fn num(v: f32) -> String {
    format!("{v:.1}")
}

fn color(c: Color32) -> String {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    if a == 255 {
        format!("rgb({r}, {g}, {b})")
    } else {
        format!("rgb({r}, {g}, {b}).with_alpha({a})")
    }
}
