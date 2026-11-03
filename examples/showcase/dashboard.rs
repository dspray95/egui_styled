//! A dev-tool dashboard in the Linear / Vercel bento style, or neubrutalist via the palette.

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, UiBuilder, vec2};
use egui_styled::prelude::*;

use crate::code::highlight;
use crate::fonts::{Weight, font};
use crate::theme::{
    Mode, Palette, card, eyebrow, ghost_button, input, primary_button, slider, text,
};

pub struct State {
    email: String,
    password: String,
    remember: bool,
    toggles: [bool; 3],
    concurrency: u32,
    region: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            email: "ada@lovelace.dev".into(),
            password: "hunter22".into(),
            remember: true,
            toggles: [true, false, true],
            concurrency: 8,
            region: 0,
        }
    }
}

const REGIONS: [&str; 3] = [
    "eu-west · London",
    "us-east · Virginia",
    "ap-south · Mumbai",
];
const GUTTER: f32 = 16.0;

pub fn show(ui: &mut Ui, state: &mut State) {
    let p = ui.ctx().design_data::<Palette>();
    let area = ui.max_rect().shrink2(vec2(24.0, 12.0));
    let col = (area.width() - GUTTER * 11.0) / 12.0;
    // Columns `c0..c1` of the 12-column grid, between `y0` and `y1`.
    let cell = |c0: usize, c1: usize, y0: f32, y1: f32| {
        let x0 = area.left() + c0 as f32 * (col + GUTTER);
        let x1 = area.left() + c1 as f32 * (col + GUTTER) - GUTTER;
        Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, y1))
    };
    let row_a = (area.top(), area.top() + 336.0);
    let row_b = (row_a.1 + GUTTER, row_a.1 + GUTTER + 138.0);
    let row_c = (row_b.1 + GUTTER, area.bottom());

    put(ui, cell(0, 8, row_a.0, row_a.1), |ui| hero(ui, &p));
    put(ui, cell(8, 12, row_a.0, row_a.1), |ui| {
        sign_in(ui, &p, state)
    });
    let stats = [
        ("Active users", "12.4k", "▲ 8.2%", true),
        ("Frame time", "1.8 ms", "▼ 0.3 ms", true),
        ("Error rate", "0.04%", "▲ 0.01%", false),
        ("Uptime", "99.98%", "30 days", true),
    ];
    for (i, stat) in stats.into_iter().enumerate() {
        put(ui, cell(i * 3, i * 3 + 3, row_b.0, row_b.1), |ui| {
            stat_tile(ui, &p, i, stat)
        });
    }
    put(ui, cell(0, 7, row_c.0, row_c.1), |ui| code_tile(ui, &p));
    put(ui, cell(7, 12, row_c.0, row_c.1), |ui| {
        settings(ui, &p, state)
    });
}

fn put(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    ui.scope_builder(UiBuilder::new().max_rect(rect), add);
}

fn tile(p: &Palette) -> StyledFrame {
    Styled::frame().apply(card(p)).full_width().full_height()
}

fn hero(ui: &mut Ui, p: &Palette) {
    let frame = match p.mode {
        Mode::Dark => Styled::frame()
            .bg(rgb(12, 10, 24))
            .bg_gradient_stops_h([
                (0.0, rgb(76, 29, 149)),
                (0.45, rgb(30, 58, 138)),
                (0.8, rgb(15, 118, 110)),
                (1.0, rgb(12, 40, 48)),
            ])
            .border(1.0, rgb(255, 255, 255).with_alpha(30))
            .inner_glow(40.0, rgb(255, 255, 255).with_alpha(10))
            .corner_radius(p.radius),
        Mode::Brutal => Styled::frame()
            .bg(p.highlights[0])
            .border(p.border_width, p.border)
            .shadow_filled(p.brutal_shadow(), Color32::BLACK)
            .corner_radius(p.radius),
    };
    frame
        .padding(32.0)
        .full_width()
        .full_height()
        .show(ui, |ui| {
            Styled::column().gap(14.0).show(ui, |ui| {
                badge(ui, p);
                let headline = match p.mode {
                    Mode::Dark => Color32::WHITE,
                    Mode::Brutal => p.text,
                };
                text(
                    "Elegant egui interfaces,\nwithout the boilerplate.",
                    Weight::Bold,
                    40.0,
                    headline,
                )
                .show(ui);
                text(
                    "Per-widget hover, focus and active states, design tokens and\n\
                     composable styles. A few builder calls per widget.",
                    Weight::Regular,
                    16.0,
                    match p.mode {
                        Mode::Dark => rgb(255, 255, 255).with_alpha(190),
                        Mode::Brutal => p.text_muted,
                    },
                )
                .show(ui);
                Styled::row().gap(12.0).show(ui, |ui| {
                    Styled::button("Get started  →")
                        .apply(primary_button(p))
                        .show(ui);
                    Styled::button("Read the docs")
                        .apply(ghost_button(p))
                        .show(ui);
                });
            });
        });
}

/// A pill announcing the new feature; glass in the dark theme.
fn badge(ui: &mut Ui, p: &Palette) {
    let frame = match p.mode {
        Mode::Dark => Styled::frame()
            .backdrop_blur(12.0)
            .bg(rgb(255, 255, 255).with_alpha(20))
            .border(1.0, rgb(255, 255, 255).with_alpha(45)),
        Mode::Brutal => Styled::frame()
            .bg(p.surface)
            .border(p.border_width, p.border),
    };
    frame
        .corner_radius(99.0)
        .padding(egui::Margin::symmetric(12, 5))
        .show(ui, |ui| {
            Styled::row().gap(8.0).show(ui, |ui| {
                text("NEW", Weight::Bold, 11.0, p.accent).show(ui);
                text(
                    "Backdrop blur for every widget",
                    Weight::Medium,
                    12.0,
                    match p.mode {
                        Mode::Dark => Color32::WHITE,
                        Mode::Brutal => p.text,
                    },
                )
                .show(ui);
            });
        });
}

fn sign_in(ui: &mut Ui, p: &Palette, state: &mut State) {
    tile(p).padding(24.0).show(ui, |ui| {
        Styled::column().gap(12.0).show(ui, |ui| {
            text("Welcome back", Weight::Semibold, 20.0, p.text).show(ui);
            text(
                "Sign in to your workspace",
                Weight::Regular,
                13.0,
                p.text_muted,
            )
            .show(ui);
            ui.add_space(2.0);
            Styled::text_edit(&mut state.email)
                .hint("Email")
                .apply(input(p))
                .show(ui);
            Styled::text_edit(&mut state.password)
                .hint("Password")
                .password()
                .apply(input(p))
                .show(ui);
            Styled::row()
                .full_width()
                .align(egui::Align::Center)
                .space_between()
                .item(|ui| {
                    Styled::checkbox(&mut state.remember, "Remember me")
                        .text_color(p.text_muted)
                        .accent(p.accent)
                        .show(ui);
                })
                .item(|ui| {
                    text("Forgot password?", Weight::Medium, 13.0, p.accent).show(ui);
                })
                .show(ui);
            Styled::button("Sign in")
                .apply(primary_button(p))
                .full_width()
                .show(ui);
        });
    });
}

fn stat_tile(
    ui: &mut Ui,
    p: &Palette,
    i: usize,
    (label, value, delta, good): (&str, &str, &str, bool),
) {
    let frame = match p.mode {
        Mode::Dark => tile(p),
        Mode::Brutal => tile(p).bg(p.highlights[i]),
    };
    frame.padding(18.0).show(ui, |ui| {
        Styled::column().gap(6.0).show(ui, |ui| {
            eyebrow(label, p).show(ui);
            Styled::row()
                .gap(10.0)
                .align(egui::Align::Center)
                .show(ui, |ui| {
                    text(value, Weight::MonoBold, 26.0, p.text).show(ui);
                    let tone = if good { p.success } else { p.danger };
                    let pill = Styled::label(delta)
                        .font(font(Weight::Semibold, 11.0))
                        .corner_radius(99.0)
                        .padding(egui::Margin::symmetric(8, 3));
                    match p.mode {
                        Mode::Dark => pill.bg(tone.with_alpha(30)).text_color(tone),
                        Mode::Brutal => pill.bg(p.surface).text_color(p.text).border(1.5, p.border),
                    }
                    .show(ui);
                });
            sparkline(ui, p, i);
        });
    });
}

/// Deterministic fake data, drawn straight with the painter.
fn sparkline(ui: &mut Ui, p: &Palette, seed: usize) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
    let n = 28;
    let points: Vec<Pos2> = (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            let s = seed as f32;
            let v = 0.5
                + 0.25 * (t * 7.0 + s * 1.3).sin()
                + 0.15 * (t * 17.0 + s).cos()
                + 0.2 * (t - 0.5);
            Pos2::new(
                rect.left() + t * rect.width(),
                rect.bottom() - v.clamp(0.0, 1.0) * rect.height(),
            )
        })
        .collect();
    let color = match p.mode {
        Mode::Dark => p.highlights[seed],
        Mode::Brutal => p.text,
    };
    ui.painter().line(points.clone(), Stroke::new(2.0, color));
    if let Some(&last) = points.last() {
        ui.painter().circle_filled(last, 3.5, color);
    }
}

const SNIPPET: &str = r#"fn primary(p: &Palette) -> impl Fn(StyledButton) -> StyledButton {
    move |b| b
        .bg(p.accent)
        .hover_bg(p.accent_hover)
        .corner_radius(10.0)
        .inner_glow(10.0, WHITE.with_alpha(28))
}

// Styles are values: store them, compose them, tweak per call.
Styled::button("Deploy").apply(primary(&p)).show(ui);
Styled::button("Delete").apply(primary(&p)).bg(p.danger).show(ui);"#;

fn code_tile(ui: &mut Ui, p: &Palette) {
    tile(p).padding(22.0).show(ui, |ui| {
        Styled::column().gap(14.0).show(ui, |ui| {
            Styled::row()
                .full_width()
                .align(egui::Align::Center)
                .space_between()
                .item(|ui| {
                    Styled::column().gap(2.0).show(ui, |ui| {
                        text("Styles are just values", Weight::Semibold, 18.0, p.text).show(ui);
                        text(
                            "This code renders the buttons on the right.",
                            Weight::Regular,
                            13.0,
                            p.text_muted,
                        )
                        .show(ui);
                    });
                })
                .item(|ui| {
                    Styled::row().gap(10.0).show(ui, |ui| {
                        Styled::button("Deploy").apply(primary_button(p)).show(ui);
                        Styled::button("Delete")
                            .apply(primary_button(p))
                            .bg(p.danger)
                            .hover_bg(p.danger.lighten(0.15))
                            .show(ui);
                    });
                })
                .show(ui);
            Styled::frame()
                .bg(rgb(13, 14, 18))
                .border(
                    p.border_width,
                    match p.mode {
                        Mode::Dark => p.border,
                        Mode::Brutal => p.border,
                    },
                )
                .corner_radius(p.radius * 0.7)
                .padding(16.0)
                .full_width()
                .full_height()
                .show(ui, |ui| {
                    Styled::label(highlight(SNIPPET, 13.0)).show(ui);
                });
        });
    });
}

fn settings(ui: &mut Ui, p: &Palette, state: &mut State) {
    tile(p).padding(22.0).show(ui, |ui| {
        Styled::column().gap(14.0).show(ui, |ui| {
            text("Workspace settings", Weight::Semibold, 18.0, p.text).show(ui);
            let rows = [
                ("Preview deployments", "Build every pull request"),
                ("Strict mode", "Fail builds on warnings"),
                ("Weekly digest", "Email a summary on Mondays"),
            ];
            for (on, (title, detail)) in state.toggles.iter_mut().zip(rows) {
                setting_row(
                    ui,
                    |ui| {
                        Styled::column().gap(1.0).show(ui, |ui| {
                            text(title, Weight::Medium, 14.0, p.text).show(ui);
                            text(detail, Weight::Regular, 12.0, p.text_muted).show(ui);
                        });
                    },
                    |ui| {
                        Styled::checkbox(on, "").accent(p.accent).show(ui);
                    },
                );
            }
            let concurrency = &mut state.concurrency;
            setting_row(
                ui,
                |ui| {
                    text("Build concurrency", Weight::Medium, 14.0, p.text).show(ui);
                },
                |ui| {
                    Styled::slider(concurrency, 1..=16)
                        .apply(slider(p))
                        .show(ui);
                },
            );
            let region = &mut state.region;
            setting_row(
                ui,
                |ui| {
                    text("Region", Weight::Medium, 14.0, p.text).show(ui);
                },
                |ui| {
                    Styled::combo_box("region", REGIONS[*region])
                        .width(190.0)
                        .bg(p.surface)
                        .border(p.border_width, p.border)
                        .text_color(p.text)
                        .corner_radius(p.radius * 0.6)
                        .show(ui, |ui| {
                            for (i, r) in REGIONS.iter().enumerate() {
                                ui.selectable_value(region, i, *r);
                            }
                        });
                },
            );
        });
    });
}

/// Label on the left, control pinned to the right edge.
fn setting_row(ui: &mut Ui, left: impl FnOnce(&mut Ui), right: impl FnOnce(&mut Ui)) {
    Styled::row()
        .full_width()
        .align(egui::Align::Center)
        .space_between()
        .item(left)
        .item(right)
        .show(ui);
}
