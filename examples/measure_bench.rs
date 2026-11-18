//! Headless CPU benchmark for measured containers: UI passes plus tessellation per frame.
//! `cargo run --release --example measure_bench [frames] [rounds]`. Configs are interleaved
//! across rounds and the median round is reported, to wash out frequency and order effects.
//! On glibc, pin the allocator or heap trimming dominates and varies with run history:
//! `MALLOC_MMAP_THRESHOLD_=4194304 MALLOC_TRIM_THRESHOLD_=67108864`.

use std::time::Instant;

use egui::{Align, Align2, Color32, Context, RawInput, Rect, Ui, pos2, vec2};
use egui_styled::Styled;

#[derive(Clone, Copy, PartialEq)]
enum Scenario {
    /// Nothing changes after first show.
    Static,
    /// Every 20 frames some content changes width and a popup toggles.
    Discrete,
    /// One label in each measured row changes width every frame.
    Animated,
}

const SCENARIOS: [(Scenario, &str); 3] = [
    (Scenario::Static, "static"),
    (Scenario::Discrete, "discrete"),
    (Scenario::Animated, "animated"),
];

fn varying_text(scenario: Scenario, frame: usize) -> String {
    let n = match scenario {
        Scenario::Static => 4,
        Scenario::Discrete => 4 + (frame / 20) % 2 * 6,
        Scenario::Animated => 4 + frame % 12,
    };
    "x".repeat(n)
}

fn tag(ui: &mut Ui, text: &str) {
    Styled::button(text).corner_radius(10.0).show(ui);
}

/// A busy screen: 20 distributed rows, 10 wrap rows of 8, 6 stacks, justify, areas.
fn screen(ui: &mut Ui, scenario: Scenario, frame: usize) {
    let varying = varying_text(scenario, frame);
    ui.vertical_centered(|ui| {
        for i in 0..20 {
            Styled::row()
                .full_width()
                .space_between()
                .item(|ui| {
                    tag(ui, &format!("left {i}"));
                })
                .item(|ui| {
                    tag(ui, if i % 4 == 0 { &varying } else { "middle" });
                })
                .item(|ui| {
                    tag(ui, "right");
                })
                .show(ui);
        }
        for i in 0..10 {
            let mut row = Styled::row().full_width().gap(4.0).wrap();
            for j in 0..8 {
                let text = if i % 3 == 0 && j == 0 {
                    varying.clone()
                } else {
                    format!("tag {j}")
                };
                row = row.item(move |ui| tag(ui, &text));
            }
            row.show(ui);
        }
        for i in 0..6 {
            Styled::stack()
                .layer(|ui| {
                    Styled::frame()
                        .bg(Color32::from_gray(40))
                        .padding(8.0)
                        .show(ui, |ui| {
                            ui.label(format!("stack {i} background"));
                        });
                })
                .layer_aligned(Align2::CENTER_CENTER, |ui| {
                    ui.label("overlay");
                })
                .show(ui);
            Styled::frame().padding(4.0).show(ui, |ui| {
                ui.label(if i == 0 { varying.as_str() } else { "shrink" });
            });
        }
        Styled::column()
            .full_width()
            .min_height(60.0)
            .full_height()
            .justify(Align::Center)
            .show(ui, |ui| {
                ui.label("justified");
            });
    });
    for i in 0..3 {
        Styled::area()
            .id(("toast", i))
            .fixed_pos_centered(pos2(200.0 + 300.0 * i as f32, 700.0))
            .bg(Color32::DARK_RED)
            .padding(6.0)
            .show(ui.ctx(), |ui| {
                ui.label(format!("toast {i}"));
            });
    }
    if scenario == Scenario::Discrete && (frame / 40) % 2 == 1 {
        Styled::area()
            .id("popup")
            .fixed_pos_centered(pos2(640.0, 400.0))
            .bg(Color32::DARK_BLUE)
            .padding(12.0)
            .show(ui.ctx(), |ui| {
                ui.label("popup");
            });
    }
}

struct Result {
    mean: f64,
    p99: f64,
    rerun_frames: usize,
    /// Mean of frames that ran a second pass.
    rerun_mean: f64,
    /// Mean of single-pass frames.
    single_mean: f64,
}

fn run(scenario: Scenario, multipass: bool, frames: usize) -> Result {
    let ctx = Context::default();
    configure(&ctx, multipass);
    let screen_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0));
    let warmup = 60;
    let mut times = Vec::with_capacity(frames);
    let mut rerun_times = Vec::new();
    for frame in 0..warmup + frames {
        let raw = RawInput {
            screen_rect: Some(screen_rect),
            time: Some(frame as f64 / 60.0),
            ..Default::default()
        };
        let start = Instant::now();
        let mut out = ctx.run_ui(raw, |ui| screen(ui, scenario, frame));
        let shapes = std::mem::take(&mut out.shapes);
        let _ = ctx.tessellate(shapes, out.pixels_per_point);
        let elapsed = start.elapsed().as_secs_f64() * 1e6;
        out.textures_delta.clear();
        if frame >= warmup {
            times.push(elapsed);
            if out.platform_output.num_completed_passes > 1 {
                rerun_times.push(elapsed);
            }
        }
    }
    let total: f64 = times.iter().sum();
    let rerun_total: f64 = rerun_times.iter().sum();
    let single_count = (times.len() - rerun_times.len()).max(1);
    times.sort_by(f64::total_cmp);
    Result {
        mean: total / times.len() as f64,
        p99: times[times.len() * 99 / 100],
        rerun_frames: rerun_times.len(),
        rerun_mean: rerun_total / rerun_times.len().max(1) as f64,
        single_mean: (total - rerun_total) / single_count as f64,
    }
}

fn configure(ctx: &Context, multipass: bool) {
    Styled::set_multipass(ctx, multipass);
}

fn main() {
    let mut args = std::env::args().skip(1).map(|a| a.parse().ok());
    let frames = args.next().flatten().unwrap_or(2000);
    let rounds = args.next().flatten().unwrap_or(5);
    println!("{frames} frames x {rounds} rounds, µs per frame (UI passes + tessellation)\n");
    println!(
        "{:<10} {:<10} {:>8} {:>8} {:>8} {:>10} {:>10}",
        "scenario", "multipass", "mean", "p99", "reruns", "1-pass", "rerun"
    );
    let configs: Vec<(Scenario, &str, bool)> = SCENARIOS
        .iter()
        .flat_map(|&(s, name)| [(s, name, true), (s, name, false)])
        .collect();
    let mut results: Vec<Vec<Result>> = configs.iter().map(|_| Vec::new()).collect();
    for _ in 0..rounds {
        for (i, &(scenario, _, multipass)) in configs.iter().enumerate() {
            results[i].push(run(scenario, multipass, frames));
        }
    }
    for ((_, name, multipass), mut rs) in configs.into_iter().zip(results) {
        rs.sort_by(|a, b| a.mean.total_cmp(&b.mean));
        let r = &rs[rs.len() / 2];
        let rerun = if r.rerun_frames > 0 {
            format!("{:.0}", r.rerun_mean)
        } else {
            "-".to_owned()
        };
        println!(
            "{:<10} {:<10} {:>8.0} {:>8.0} {:>8} {:>10.0} {:>10}",
            name,
            if multipass { "on" } else { "off" },
            r.mean,
            r.p99,
            r.rerun_frames,
            r.single_mean,
            rerun
        );
    }
}
