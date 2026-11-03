//! Backdrop blur on containers (`--features backdrop-blur`, wgpu backend).
//!
//! The glass card is a draggable foreground area; the nav bar shares the scene's layer.
//! Set `EGUI_STYLED_SCREENSHOT=out.ppm` to save frame 60 and exit.
//! Set `EGUI_STYLED_BENCH=N` to time N glass panels (vsync off) and exit;
//! add `EGUI_STYLED_BENCH_NOBLUR=1` to draw the same panels without blur,
//! or `EGUI_STYLED_STATIC=1` to freeze the scene.

use egui::{Color32, Id, Order, Pos2, Rect, Stroke, Ui, UiBuilder, vec2};
use egui_styled::prelude::*;

struct App {
    checker: egui::TextureHandle,
    email: String,
    search: String,
    frame: u64,
    screenshot: Option<String>,
    bench: Option<Bench>,
}

struct Bench {
    panels: usize,
    blur: bool,
    /// Summed `cpu_usage` (seconds) over the measured frames.
    cpu: f32,
}

impl Bench {
    const WARMUP: u64 = 60;
    const FRAMES: u64 = 300;

    fn from_env() -> Option<Self> {
        let panels = std::env::var("EGUI_STYLED_BENCH").ok()?.parse().ok()?;
        Some(Self {
            panels,
            blur: std::env::var("EGUI_STYLED_BENCH_NOBLUR").is_err(),
            cpu: 0.0,
        })
    }
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("backdrop blur needs the wgpu backend");
        egui_styled::backdrop::install(&cc.egui_ctx, render_state);

        let checker = egui::ColorImage::new(
            [8, 8],
            (0..64)
                .map(|i| {
                    if (i % 8 + i / 8) % 2 == 0 {
                        Color32::from_rgb(250, 250, 250)
                    } else {
                        Color32::from_rgb(20, 20, 30)
                    }
                })
                .collect(),
        );
        Self {
            checker: cc
                .egui_ctx
                .load_texture("checker", checker, egui::TextureOptions::NEAREST),
            email: String::new(),
            search: String::new(),
            frame: 0,
            screenshot: std::env::var("EGUI_STYLED_SCREENSHOT").ok(),
            bench: Bench::from_env(),
        }
    }

    /// Moving blobs plus hard-edged text, an image and a line, so the blur is obvious.
    fn scene(&self, ui: &mut Ui) {
        let rect = ui.max_rect();
        // `EGUI_STYLED_STATIC=1` freezes the scene, to benchmark reused backdrops.
        let t = if std::env::var("EGUI_STYLED_STATIC").is_ok() {
            0.0
        } else {
            ui.input(|i| i.time) as f32
        };
        let painter = ui.painter();

        let blobs = [
            (Color32::from_rgb(124, 58, 237), 0.0, 220.0),
            (Color32::from_rgb(37, 99, 235), 2.1, 180.0),
            (Color32::from_rgb(20, 184, 166), 4.2, 160.0),
            (Color32::from_rgb(236, 72, 153), 1.0, 120.0),
        ];
        for (color, phase, r) in blobs {
            let c = rect.center()
                + vec2(
                    (t * 0.4 + phase).cos() * rect.width() * 0.3,
                    (t * 0.3 + phase * 1.3).sin() * rect.height() * 0.28,
                );
            painter.circle_filled(c, r, color);
        }

        for i in 0..14 {
            painter.text(
                Pos2::new(rect.left() + 40.0, rect.top() + 40.0 + i as f32 * 52.0),
                egui::Align2::LEFT_TOP,
                "backdrop-filter: blur(20px);",
                egui::FontId::monospace(28.0),
                Color32::from_white_alpha(160),
            );
        }
        let img = Rect::from_min_size(rect.right_bottom() - vec2(320.0, 320.0), vec2(260.0, 260.0));
        painter.image(
            self.checker.id(),
            img,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        painter.line_segment(
            [rect.left_bottom(), rect.right_top()],
            Stroke::new(6.0, Color32::from_rgb(250, 204, 21)),
        );
    }

    fn glass(blur: f32) -> StyledFrame {
        Styled::frame()
            .backdrop_blur(blur)
            .bg(Color32::from_white_alpha(18))
            .border(1.0, Color32::from_white_alpha(48))
            .inner_glow(10.0, Color32::from_white_alpha(10))
    }

    fn nav(ui: &mut Ui) {
        Self::glass(16.0)
            .corner_radius(14.0)
            .padding(14.0)
            .full_width()
            .show(ui, |ui| {
                Styled::row().gap(16.0).show(ui, |ui| {
                    Styled::label("egui_styled")
                        .bold()
                        .font_size(16.0)
                        .text_color(Color32::WHITE)
                        .show(ui);
                    for item in ["Docs", "Examples", "GitHub"] {
                        Styled::label(item)
                            .text_color(Color32::from_white_alpha(170))
                            .show(ui);
                    }
                });
            });
    }

    /// Glass widgets straight on the scene, no glass container behind them.
    fn toolbar(&mut self, ui: &mut Ui) {
        Styled::row().gap(12.0).show(ui, |ui| {
            Styled::button("Glass button")
                .backdrop_blur(12.0)
                .bg(Color32::from_white_alpha(24))
                .hover_bg(Color32::from_white_alpha(40))
                .border(1.0, Color32::from_white_alpha(60))
                .text_color(Color32::WHITE)
                .corner_radius(12.0)
                .padding(14.0)
                .show(ui);
            Styled::text_edit(&mut self.search)
                .hint("Search…")
                .backdrop_blur(12.0)
                .bg(Color32::from_white_alpha(18))
                .border(1.0, Color32::from_white_alpha(48))
                .text_color(Color32::WHITE)
                .corner_radius(12.0)
                .padding(12.0)
                .min_width(260.0)
                .show(ui);
            Styled::label("Glass label")
                .backdrop_blur(12.0)
                .bg(Color32::from_white_alpha(18))
                .text_color(Color32::WHITE)
                .corner_radius(12.0)
                .padding(12.0)
                .show(ui);
        });
    }

    fn card(&mut self, ui: &mut Ui) {
        Self::glass(24.0)
            .corner_radius(20.0)
            .padding(28.0)
            .min_width(360.0)
            .show(ui, |ui| {
                Styled::column().gap(14.0).show(ui, |ui| {
                    Styled::label("Frosted glass")
                        .font_size(26.0)
                        .bold()
                        .text_color(Color32::WHITE)
                        .show(ui);
                    Styled::label("One builder: .backdrop_blur(24.0)")
                        .text_color(Color32::from_white_alpha(180))
                        .show(ui);
                    Styled::text_edit(&mut self.email)
                        .hint("you@example.com")
                        .full_width()
                        .bg(Color32::from_black_alpha(60))
                        .text_color(Color32::WHITE)
                        .border(1.0, Color32::from_white_alpha(40))
                        .focus_border(1.0, Color32::from_rgb(167, 139, 250))
                        .corner_radius(10.0)
                        .padding(10.0)
                        .show(ui);
                    Styled::button("Get started")
                        .full_width()
                        .bg(Color32::from_rgb(124, 58, 237))
                        .hover_bg(Color32::from_rgb(139, 92, 246))
                        .text_color(Color32::WHITE)
                        .corner_radius(10.0)
                        .padding(12.0)
                        .show(ui);
                });
            });
    }

    /// A grid of small glass panels over the scene; returns once timing is done.
    /// `cpu_usage` is app UI + tessellation + render submission, without the vsync wait.
    fn bench(&mut self, ui: &mut Ui, cpu_usage: Option<f32>) {
        let Some(bench) = &mut self.bench else {
            return;
        };
        let (panels, blur) = (bench.panels, bench.blur);
        if self.frame > Bench::WARMUP {
            bench.cpu += cpu_usage.unwrap_or(0.0);
        }
        if self.frame == Bench::WARMUP + Bench::FRAMES {
            let ms = bench.cpu * 1000.0 / Bench::FRAMES as f32;
            println!("panels={panels} blur={blur}: {ms:.3} ms CPU/frame");
            std::process::exit(0);
        }

        let area = ui.max_rect().shrink(24.0);
        let cols = 5;
        let size = vec2(area.width() / cols as f32 - 12.0, 120.0);
        for i in 0..panels {
            let min = area.min
                + vec2(
                    (i % cols) as f32 * (size.x + 12.0),
                    (i / cols) as f32 * (size.y + 12.0),
                );
            ui.scope_builder(
                UiBuilder::new().max_rect(Rect::from_min_size(min, size)),
                |ui| {
                    let frame = if blur {
                        Self::glass(24.0)
                    } else {
                        Styled::frame()
                            .bg(Color32::from_white_alpha(18))
                            .border(1.0, Color32::from_white_alpha(48))
                            .inner_glow(10.0, Color32::from_white_alpha(10))
                    };
                    frame
                        .corner_radius(16.0)
                        .padding(16.0)
                        .full_width()
                        .min_height(size.y)
                        .show(ui, |ui| {
                            Styled::label(format!("Panel {i}"))
                                .text_color(Color32::WHITE)
                                .show(ui);
                        });
                },
            );
        }
    }

    fn handle_screenshot(&mut self, ui: &Ui) {
        let Some(path) = self.screenshot.clone() else {
            return;
        };
        if self.frame != 60 {
            return;
        }
        let callback = egui::ScreenshotCallback::new(move |image| {
            let [w, h] = image.size;
            let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
            for p in &image.pixels {
                out.extend_from_slice(&[p.r(), p.g(), p.b()]);
            }
            std::fs::write(&path, out).expect("write screenshot");
            std::process::exit(0);
        });
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let cpu_usage = frame.info().cpu_usage;
        ui.ctx().request_repaint();
        self.frame += 1;
        self.handle_screenshot(ui);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(9, 9, 11)))
            .show(ui, |ui| {
                self.scene(ui);
                if self.bench.is_some() {
                    self.bench(ui, cpu_usage);
                    return;
                }
                let nav = Rect::from_min_size(
                    ui.max_rect().min + vec2(24.0, 24.0),
                    vec2(ui.max_rect().width() - 48.0, 0.0),
                );
                ui.scope_builder(UiBuilder::new().max_rect(nav), Self::nav);
                let toolbar = Rect::from_min_size(
                    Pos2::new(ui.max_rect().left() + 24.0, ui.max_rect().bottom() - 84.0),
                    vec2(ui.max_rect().width() - 48.0, 60.0),
                );
                ui.scope_builder(UiBuilder::new().max_rect(toolbar), |ui| self.toolbar(ui));
            });

        if self.bench.is_some() {
            return;
        }
        egui::Area::new(Id::unique("glass_card"))
            .order(Order::Foreground)
            .default_pos(Pos2::new(380.0, 200.0))
            .movable(true)
            .show(ui.ctx(), |ui| self.card(ui));
    }
}

/// Uncapped frame rate when benchmarking, so timings measure work rather than the display.
fn wgpu_options() -> eframe::egui_wgpu::WgpuConfiguration {
    let mut options = eframe::egui_wgpu::WgpuConfiguration::default();
    if std::env::var("EGUI_STYLED_BENCH").is_ok() {
        options.surface.present_mode = eframe::wgpu::PresentMode::AutoNoVsync;
    }
    options
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "egui_styled backdrop blur",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: wgpu_options(),
            viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
