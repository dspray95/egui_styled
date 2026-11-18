//! Visual checks for measured containers: size animation, freeze, staggered appearance.
//! `cargo run --example measure_check`

use egui::{Align2, CentralPanel, Color32, Panel, Ui, Vec2};
use egui_styled::prelude::*;

const SHORT_ANIM: f64 = 2.0;
const LONG_ANIM: f64 = 5.0;
const STAGGERED_ROWS: u64 = 24;

#[derive(Default)]
struct App {
    anim_start: Option<f64>,
    long_anim_start: Option<f64>,
    long_anim_done: bool,
    oscillate: bool,
    stagger_start: Option<u64>,
    continuous: bool,
    force_multipass: bool,
}

/// `x` repeated 4..=24 times, following `t` in 0..=1 out and back.
fn animated_text(t: f64) -> String {
    let wave = (t * std::f64::consts::PI).sin();
    "x".repeat(4 + (wave * 20.0).round() as usize)
}

fn tag(ui: &mut Ui, text: &str, color: Color32) {
    Styled::label(text)
        .text_color(Color32::WHITE)
        .bg(color)
        .padding(6.0)
        .corner_radius(6.0)
        .extend()
        .show(ui);
}

fn card(ui: &mut Ui, title: &str, hint: &str, body: impl FnOnce(&mut Ui)) {
    Styled::frame()
        .full_width()
        .bg(rgb(28, 28, 36))
        .padding(12.0)
        .corner_radius(8.0)
        .show(ui, |ui| {
            Styled::label(title).text_color(Color32::WHITE).show(ui);
            Styled::label(hint).text_color(rgb(150, 150, 170)).show(ui);
            ui.add_space(8.0);
            body(ui);
        });
    ui.add_space(12.0);
}

/// Left and right tags with `middle` distributed between them.
fn distributed(ui: &mut Ui, id: &str, middle: &str) {
    Styled::row()
        .id(id)
        .full_width()
        .space_between()
        .item(|ui| tag(ui, "left", rgb(60, 60, 90)))
        .item(|ui| tag(ui, middle, rgb(150, 60, 40)))
        .item(|ui| tag(ui, "right", rgb(60, 60, 90)))
        .show(ui);
}

impl App {
    fn controls(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let mut config = Styled::measure_config(&ctx);
        ui.heading("Measurement");
        ui.checkbox(&mut config.multipass, "multipass");
        if ui
            .checkbox(&mut self.force_multipass, "debug_force_multipass")
            .changed()
        {
            Styled::debug_force_multipass(&ctx, self.force_multipass);
        }
        ui.checkbox(&mut self.continuous, "repaint continuously")
            .on_hover_text("Off: egui only repaints on input or when something asks");
        ui.add(egui::Slider::new(&mut config.max_streak, 0..=10).text("max_streak"));
        ui.add(egui::Slider::new(&mut config.max_changes, 10..=600).text("max_changes"));
        ui.add(egui::Slider::new(&mut config.budget.0, 0..=16).text("budget frames"));
        ui.add(egui::Slider::new(&mut config.budget.1, 1..=64).text("budget window"));
        if ui.button("Reset config").clicked() {
            config = MeasureConfig::default();
        }
        Styled::set_measure_config(&ctx, config);

        ui.separator();
        let stats = Styled::measure_stats(&ctx);
        ui.monospace(format!("frame     {}", ctx.cumulative_frame_nr()));
        ui.monospace(format!("misses    {}", stats.misses));
        ui.monospace(format!("reruns    {}", stats.discards));
        ui.monospace(format!("fallbacks {}", stats.repaints));
        ui.monospace(format!("freezes   {}", stats.freezes));
        if let Some((kind, id)) = stats.last_freeze {
            ui.monospace(format!("last freeze: {kind} {id:?}"));
        }
        ui.label("Fallbacks are frames shown one frame stale or blank.");
    }

    fn size_animation(&mut self, ui: &mut Ui, now: f64) {
        let t = self.anim_start.map(|start| (now - start) / SHORT_ANIM);
        if t.is_some_and(|t| t >= 1.0) {
            self.anim_start = None;
        }
        let text = animated_text(t.unwrap_or(0.0).min(1.0));
        if self.anim_start.is_some() {
            ui.ctx().request_repaint();
        }

        card(
            ui,
            "1. Size animation",
            "Right stays pinned, the middle tag stays centred, the wrap row reflows cleanly, \
             the boxes stay centred. Past max_streak, changes show one frame late: look for lag.",
            |ui| {
                if ui.button("Animate (2 s)").clicked() {
                    self.anim_start = Some(now);
                }
                distributed(ui, "anim_row", &text);
                ui.add_space(6.0);
                let mut wrap = Styled::row().id("anim_wrap").full_width().gap(6.0).wrap();
                for i in 0..10 {
                    let label = if i == 0 {
                        text.clone()
                    } else {
                        format!("tag {i}")
                    };
                    wrap = wrap.item(move |ui| tag(ui, &label, rgb(40, 90, 70)));
                }
                wrap.show(ui);
                ui.add_space(6.0);
                let centre = ui
                    .horizontal(|ui| {
                        Styled::stack()
                            .id("anim_stack")
                            .layer(|ui| {
                                Styled::frame().bg(rgb(45, 45, 60)).show(ui, |ui| {
                                    ui.allocate_space(Vec2::new(300.0, 60.0));
                                });
                            })
                            .layer_aligned(Align2::CENTER_CENTER, |ui| {
                                tag(ui, &text, rgb(150, 60, 40));
                            })
                            .show(ui);
                        ui.add_space(20.0);
                        ui.allocate_space(Vec2::new(300.0, 60.0)).1.center()
                    })
                    .inner;
                Styled::area()
                    .id("anim_area")
                    .fixed_pos_centered(centre)
                    .show(ui.ctx(), |ui| tag(ui, &text, rgb(120, 40, 120)));
            },
        );
    }

    fn long_animation(&mut self, ui: &mut Ui, now: f64) {
        let t = self.long_anim_start.map(|start| (now - start) / LONG_ANIM);
        if t.is_some_and(|t| t >= 1.0) {
            self.long_anim_start = None;
            self.long_anim_done = true;
        }
        // Grows for the whole run and stops at a new size.
        let len = match t {
            Some(t) => 4 + (t * 20.0) as usize,
            None if self.long_anim_done => 24,
            None => 4,
        };
        if self.long_anim_start.is_some() {
            ui.ctx().request_repaint();
        }
        // Flips every frame; only the measurement system's own repaints drive it.
        let oscillating = if self.oscillate && ui.ctx().cumulative_frame_nr().is_multiple_of(2) {
            "wide oscillating item"
        } else {
            "narrow"
        };

        card(
            ui,
            "2. Long animation and freeze",
            "Run with continuous repaint off. The 5 s run outlasts max_changes: the row must end \
             correctly placed. Oscillate never settles: it should freeze (counter goes up) and \
             stop repainting by itself; moving the mouse then updates it.",
            |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Run 5 s animation").clicked() {
                        self.long_anim_start = Some(now);
                        self.long_anim_done = false;
                    }
                    ui.checkbox(&mut self.oscillate, "Oscillate");
                });
                distributed(ui, "long_row", &"x".repeat(len));
                ui.add_space(6.0);
                distributed(ui, "oscillating_row", oscillating);
            },
        );
    }

    fn staggered(&mut self, ui: &mut Ui) {
        let frame = ui.ctx().cumulative_frame_nr();
        let shown = self
            .stagger_start
            .map_or(0, |start| (frame - start).min(STAGGERED_ROWS));
        if self.stagger_start.is_some() && shown < STAGGERED_ROWS {
            ui.ctx().request_repaint();
        }

        card(
            ui,
            "3. Staggered appearance",
            "One new measured row per frame, more than the rerun budget allows. Rows refused a \
             rerun appear a frame late, but none should appear in the wrong place.",
            |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Show rows").clicked() {
                        self.stagger_start = Some(frame);
                    }
                    if ui.button("Clear").clicked() {
                        self.stagger_start = None;
                    }
                });
                for i in 0..shown {
                    distributed(ui, &format!("stagger_{i}"), &format!("row {i}"));
                }
            },
        );
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

impl App {
    fn show(&mut self, ui: &mut Ui) {
        if self.continuous {
            ui.ctx().request_repaint();
        }
        let now = ui.input(|i| i.time);
        Panel::left("controls")
            .default_size(260.0)
            .show(ui, |ui| self.controls(ui));
        CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.size_animation(ui, now);
                self.long_animation(ui, now);
                self.staggered(ui);
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "egui_styled measurement check",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 800.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::<App>::default())),
    )
}
