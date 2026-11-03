//! The egui_styled showcase: real-world-style scenes built only from styled builders.
//!
//! `cargo run --release --example showcase --features backdrop-blur`
//! `-- --screenshot <dir>` renders every scene to PNG and exits.

mod art;
mod code;
mod dashboard;
mod fonts;
mod player;
mod playground;
mod theme;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use egui::{Color32, Ui};
use egui_styled::prelude::*;

use fonts::{Choice, Weight};
use theme::{Mode, Palette, text};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scene {
    Dashboard,
    Player,
    Playground,
}

impl Scene {
    const ALL: [Scene; 3] = [Scene::Dashboard, Scene::Player, Scene::Playground];

    fn name(self) -> &'static str {
        match self {
            Scene::Dashboard => "Dashboard",
            Scene::Player => "Player",
            Scene::Playground => "Playground",
        }
    }
}

/// Renders a fixed list of (file, scene, mode, font) shots, one at a time.
struct Screenshots {
    dir: PathBuf,
    shots: Vec<(&'static str, Scene, Mode, Choice)>,
    next: usize,
    /// Frame the current shot's scene was switched in on.
    started: u64,
    /// Bumped by the screenshot callback once the file is written.
    saved: Arc<AtomicUsize>,
    requested: bool,
}

impl Screenshots {
    /// Frames to let images decode and fade-ins finish before capturing.
    const SETTLE_FRAMES: u64 = 45;

    fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            shots: vec![
                (
                    "dashboard_dark",
                    Scene::Dashboard,
                    Mode::Dark,
                    Choice::Inter,
                ),
                (
                    "dashboard_brutal",
                    Scene::Dashboard,
                    Mode::Brutal,
                    Choice::Inter,
                ),
                ("player", Scene::Player, Mode::Dark, Choice::Inter),
                ("playground", Scene::Playground, Mode::Dark, Choice::Inter),
                (
                    "dashboard_dark_geist",
                    Scene::Dashboard,
                    Mode::Dark,
                    Choice::Geist,
                ),
                (
                    "dashboard_brutal_space_grotesk",
                    Scene::Dashboard,
                    Mode::Brutal,
                    Choice::SpaceGrotesk,
                ),
                ("player_geist", Scene::Player, Mode::Dark, Choice::Geist),
            ],
            next: 0,
            started: 0,
            saved: Arc::default(),
            requested: false,
        }
    }
}

struct App {
    scene: Scene,
    mode: Mode,
    font: Choice,
    dashboard: dashboard::State,
    player: player::State,
    playground: playground::State,
    frame: u64,
    screenshots: Option<Screenshots>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, screenshots: Option<Screenshots>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("the showcase needs the wgpu backend");
        egui_styled::backdrop::install(&cc.egui_ctx, render_state);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_fonts(fonts::definitions(Choice::Inter));
        cc.egui_ctx.set_design_data(Palette::for_mode(Mode::Dark));
        // `.bold()` picks this family; every bundled choice registers a real 700 cut there.
        cc.egui_ctx.set_styled_theme(StyledTheme {
            font_family_bold: Some(egui::FontFamily::Name("w700".into())),
            ..Default::default()
        });

        Self {
            scene: Scene::Dashboard,
            mode: Mode::Dark,
            font: Choice::Inter,
            dashboard: dashboard::State::default(),
            player: player::State::default(),
            playground: playground::State::default(),
            frame: 0,
            screenshots,
        }
    }

    fn set_mode(&mut self, ctx: &egui::Context, mode: Mode) {
        self.mode = mode;
        ctx.set_design_data(Palette::for_mode(mode));
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        let p = ui.ctx().design_data::<Palette>();
        // Items can't share `&mut self`, so they edit copies applied below.
        let (mut scene, mut mode, mut font) = (self.scene, self.mode, self.font);
        Styled::row()
            .align(egui::Align::Center)
            .full_width()
            .space_between()
            .item(|ui| {
                Styled::row()
                    .gap(10.0)
                    .align(egui::Align::Center)
                    .show(ui, |ui| {
                        Styled::label("◆")
                            .font(fonts::font(Weight::Bold, 18.0))
                            .text_color(p.accent)
                            .show(ui);
                        Styled::label("egui_styled")
                            .font_size(16.0)
                            .bold()
                            .text_color(p.text)
                            .show(ui);
                        text("showcase", Weight::Regular, 14.0, p.text_muted).show(ui);
                        ui.add_space(24.0);
                        for s in Scene::ALL {
                            if tab(&p, s.name(), scene == s).show(ui).clicked() {
                                scene = s;
                            }
                        }
                    });
            })
            .item(|ui| {
                Styled::row()
                    .gap(10.0)
                    .align(egui::Align::Center)
                    .show(ui, |ui| {
                        for (m, name) in [(Mode::Dark, "Dark"), (Mode::Brutal, "Neubrutal")] {
                            if tab(&p, name, mode == m).show(ui).clicked() {
                                mode = m;
                            }
                        }
                        ui.add_space(8.0);
                        Styled::combo_box("font", font.name())
                            .width(140.0)
                            .bg(p.surface)
                            .border(p.border_width, p.border)
                            .text_color(p.text)
                            .corner_radius(p.radius * 0.6)
                            .show(ui, |ui| {
                                for choice in Choice::ALL {
                                    ui.selectable_value(&mut font, choice, choice.name());
                                }
                            });
                    });
            })
            .show(ui);

        self.scene = scene;
        if mode != self.mode {
            self.set_mode(ui.ctx(), mode);
        }
        if font != self.font {
            self.font = font;
            ui.ctx().set_fonts(fonts::definitions(font));
        }
    }

    fn drive_screenshots(&mut self, ctx: &egui::Context) {
        let Some(shots) = &mut self.screenshots else {
            return;
        };
        // Static scenes stop producing frames once settled; keep the countdown moving.
        ctx.request_repaint();
        if shots.requested && shots.saved.load(Ordering::SeqCst) > shots.next {
            shots.next += 1;
            shots.requested = false;
        }
        let Some(&(name, scene, mode, font)) = shots.shots.get(shots.next) else {
            std::process::exit(0);
        };
        if self.scene != scene || self.mode != mode || self.font != font || shots.started == 0 {
            shots.started = self.frame;
            self.scene = scene;
            self.mode = mode;
            ctx.set_design_data(Palette::for_mode(mode));
            if self.font != font {
                self.font = font;
                ctx.set_fonts(fonts::definitions(font));
            }
            return;
        }
        if shots.requested || self.frame < shots.started + Screenshots::SETTLE_FRAMES {
            return;
        }
        shots.requested = true;
        let path = shots.dir.join(format!("{name}.png"));
        let saved = Arc::clone(&shots.saved);
        let callback = egui::ScreenshotCallback::new(move |image| {
            let [w, h] = image.size;
            let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
            image::save_buffer(&path, &rgba, w as u32, h as u32, image::ColorType::Rgba8)
                .expect("save screenshot");
            println!("wrote {}", path.display());
            saved.fetch_add(1, Ordering::SeqCst);
        });
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
    }
}

/// A segmented-control style tab.
fn tab(p: &Palette, label: &str, selected: bool) -> StyledButton {
    let b = Styled::button(label)
        .font(fonts::font(Weight::Medium, 13.0))
        .corner_radius(p.radius * 0.6)
        .padding(egui::Margin::symmetric(14, 7));
    match (p.mode, selected) {
        (Mode::Dark, true) => b
            .bg(rgb(255, 255, 255).with_alpha(18))
            .border(1.0, rgb(255, 255, 255).with_alpha(30))
            .text_color(p.text),
        (Mode::Dark, false) => b
            .bg(Color32::TRANSPARENT)
            .hover_bg(rgb(255, 255, 255).with_alpha(10))
            .text_color(p.text_muted)
            .hover_text_color(p.text),
        (Mode::Brutal, true) => b
            .bg(p.highlights[0])
            .border(p.border_width, p.border)
            .text_color(p.text),
        (Mode::Brutal, false) => b
            .bg(Color32::TRANSPARENT)
            .hover_bg(p.surface)
            .text_color(p.text),
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.frame += 1;
        self.drive_screenshots(ui.ctx());
        let p = ui.ctx().design_data::<Palette>();

        egui::Panel::top("top_bar")
            .exact_size(60.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::NONE
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(24, 12)),
            )
            .show(ui, |ui| self.top_bar(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(p.bg))
            .show(ui, |ui| match self.scene {
                Scene::Dashboard => dashboard::show(ui, &mut self.dashboard),
                Scene::Player => player::show(ui, &mut self.player),
                Scene::Playground => playground::show(ui, &mut self.playground),
            });
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let screenshots = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(|dir| {
            std::fs::create_dir_all(dir).expect("create screenshot dir");
            Screenshots::new(PathBuf::from(dir))
        });

    eframe::run_native(
        "egui_styled showcase",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1440.0, 900.0])
                .with_resizable(screenshots.is_none()),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc, screenshots)))),
    )
}
