//! A music player in the Apple Music style: the album art, blurred, is the whole backdrop.

use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, UiBuilder, pos2, vec2};
use egui_styled::prelude::*;

use crate::art::{Track, tracks};
use crate::fonts::{Weight, font};
use crate::theme::text;

pub struct State {
    tracks: [Track; 6],
    current: usize,
    playing: bool,
    position: f32,
    hovered: Option<usize>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tracks: tracks(),
            current: 0,
            playing: false,
            position: 84.0,
            hovered: None,
        }
    }
}

const WHITE_MUTED: Color32 = Color32::from_rgba_premultiplied(190, 190, 200, 200);

/// Translucent panel. No blur of its own: the backdrop under it is already blurred.
fn panel() -> StyledFrame {
    Styled::frame()
        .bg(rgb(255, 255, 255).with_alpha(14))
        .border(1.0, rgb(255, 255, 255).with_alpha(36))
        .inner_glow(18.0, rgb(255, 255, 255).with_alpha(8))
        .corner_radius(22.0)
        .padding(22.0)
        .full_width()
        .full_height()
}

pub fn show(ui: &mut Ui, state: &mut State) {
    if state.playing {
        let track_len = state.tracks[state.current].seconds as f32;
        state.position = (state.position + ui.input(|i| i.stable_dt)).min(track_len);
        ui.ctx().request_repaint();
    }
    let area = ui.max_rect();
    let cover = state.tracks[state.current].cover.clone();

    // Album art, then one big blur over it, then the UI.
    put(ui, area, |ui| {
        Styled::frame()
            .background_image(cover)
            .background_image_fit(BackgroundImageFit::Cover)
            .full_width()
            .full_height()
            .show(ui, |_| {});
    });
    put(ui, area, |ui| {
        Styled::frame()
            .backdrop_blur(48.0)
            .bg(rgb(0, 0, 0).with_alpha(70))
            .bg_gradient_v(rgb(0, 0, 0).with_alpha(20), rgb(0, 0, 0).with_alpha(160))
            .full_width()
            .full_height()
            .show(ui, |_| {});
    });

    let inner = area.shrink(24.0);
    let side = Rect::from_min_size(inner.min, vec2(250.0, inner.height()));
    let queue = Rect::from_min_max(pos2(inner.right() - 380.0, inner.top()), inner.max);
    let stage = Rect::from_min_max(
        pos2(side.right() + 24.0, inner.top()),
        pos2(queue.left() - 24.0, inner.bottom()),
    );
    put(ui, side, sidebar);
    put(ui, stage, |ui| now_playing(ui, state));
    put(ui, queue, |ui| up_next(ui, state));
}

fn put(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    ui.scope_builder(UiBuilder::new().max_rect(rect), add);
}

fn sidebar(ui: &mut Ui) {
    panel().show(ui, |ui| {
        Styled::column().gap(6.0).show(ui, |ui| {
            text("Harmony", Weight::Bold, 22.0, Color32::WHITE).show(ui);
            ui.add_space(14.0);
            for (i, item) in ["Listen Now", "Browse", "Radio", "Library"]
                .iter()
                .enumerate()
            {
                nav_item(ui, item, i == 0);
            }
            ui.add_space(18.0);
            text("PLAYLISTS", Weight::Semibold, 11.0, WHITE_MUTED).show(ui);
            ui.add_space(4.0);
            for item in ["Late Night Drive", "Focus Flow", "Weekend Hike", "Deep Sea"] {
                nav_item(ui, item, false);
            }
        });
    });
}

fn nav_item(ui: &mut Ui, label: &str, active: bool) {
    let b = Styled::button(label)
        .font(font(Weight::Medium, 14.0))
        .full_width()
        .corner_radius(10.0)
        .padding(egui::Margin::symmetric(12, 9))
        .hover_bg(rgb(255, 255, 255).with_alpha(18));
    if active {
        b.bg(rgb(255, 255, 255).with_alpha(26))
            .text_color(Color32::WHITE)
    } else {
        b.bg(Color32::TRANSPARENT)
            .text_color(WHITE_MUTED)
            .hover_text_color(Color32::WHITE)
    }
    .show(ui);
}

fn now_playing(ui: &mut Ui, state: &mut State) {
    let track = &state.tracks[state.current];
    let (title, artist, length) = (track.title, track.artist, track.seconds as f32);
    let cover = track.cover.clone();
    Styled::column()
        .gap(14.0)
        .align(egui::Align::Center)
        .full_width()
        .show(ui, |ui| {
            ui.add_space(30.0);
            Styled::image(cover)
                .size(vec2(380.0, 380.0))
                .corner_radius(20.0)
                .border(1.0, rgb(255, 255, 255).with_alpha(40))
                .show(ui);
            ui.add_space(8.0);
            text(title, Weight::Bold, 30.0, Color32::WHITE).show(ui);
            text(artist, Weight::Medium, 17.0, WHITE_MUTED).show(ui);
            ui.add_space(6.0);
            scrubber(ui, &mut state.position, length);
            // A fixed-size slot, so the centred column can centre the controls.
            ui.allocate_ui(vec2(48.0 + 68.0 + 48.0 + 2.0 * 22.0, 68.0), |ui| {
                transport(ui, state)
            });
        });
}

/// A custom progress bar: painter shapes plus one drag-to-seek interaction.
fn scrubber(ui: &mut Ui, position: &mut f32, length: f32) {
    let width = 420.0;
    // The knob only shows while you hover or drag, like Apple Music's scrubber.
    Styled::slider(position, 0.0..=length)
        .show_value(false)
        .width(width)
        .rail(rgb(255, 255, 255).with_alpha(50))
        .rail_thickness(5.0)
        .fill(Color32::WHITE)
        .handle(Color32::TRANSPARENT)
        .hover_handle(Color32::WHITE)
        .handle_radius(7.0)
        .show(ui);

    ui.allocate_ui(vec2(width, 18.0), |ui| {
        Styled::row()
            .full_width()
            .space_between()
            .item(|ui| {
                text(clock(*position), Weight::Mono, 12.0, WHITE_MUTED).show(ui);
            })
            .item(|ui| {
                text(
                    format!("-{}", clock(length - *position)),
                    Weight::Mono,
                    12.0,
                    WHITE_MUTED,
                )
                .show(ui);
            })
            .show(ui);
    });
}

fn clock(seconds: f32) -> String {
    let s = seconds.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

#[derive(Clone, Copy)]
enum Icon {
    Prev,
    Play,
    Pause,
    Next,
}

fn transport(ui: &mut Ui, state: &mut State) {
    let n = state.tracks.len();
    // Offset the small buttons by hand: a centred row places its first item before it
    // knows the row's height.
    let side = |ui: &mut Ui, icon| {
        ui.vertical(|ui| {
            ui.add_space((68.0 - 48.0) / 2.0);
            icon_button(ui, icon, 48.0, false)
        })
        .inner
    };
    Styled::row().gap(22.0).show(ui, |ui| {
        if side(ui, Icon::Prev).clicked() {
            state.current = (state.current + n - 1) % n;
            state.position = 0.0;
        }
        let icon = if state.playing {
            Icon::Pause
        } else {
            Icon::Play
        };
        if icon_button(ui, icon, 68.0, true).clicked() {
            state.playing = !state.playing;
        }
        if side(ui, Icon::Next).clicked() {
            state.current = (state.current + 1) % n;
            state.position = 0.0;
        }
    });
}

/// A round styled button with a painter-drawn glyph, so it never depends on font coverage.
fn icon_button(ui: &mut Ui, icon: Icon, size: f32, primary: bool) -> egui::Response {
    let b = Styled::button("")
        .min_width(size)
        .min_height(size)
        .corner_radius(size / 2.0);
    let resp = if primary {
        b.bg(Color32::WHITE).hover_bg(rgb(230, 230, 236))
    } else {
        b.bg(rgb(255, 255, 255).with_alpha(18))
            .hover_bg(rgb(255, 255, 255).with_alpha(34))
            .border(1.0, rgb(255, 255, 255).with_alpha(30))
    }
    .show(ui);

    let c = resp.rect.center();
    let s = size * 0.2;
    let color = if primary {
        rgb(20, 20, 24)
    } else {
        Color32::WHITE
    };
    let painter = ui.painter();
    let triangle = |tip: Pos2, dir: f32| {
        Shape::convex_polygon(
            vec![
                pos2(tip.x - dir * 1.6 * s, tip.y - s),
                tip,
                pos2(tip.x - dir * 1.6 * s, tip.y + s),
            ],
            color,
            Stroke::NONE,
        )
    };
    match icon {
        Icon::Play => {
            painter.add(triangle(pos2(c.x + s, c.y), 1.0));
        }
        Icon::Pause => {
            for dx in [-0.55, 0.55] {
                painter.rect_filled(
                    Rect::from_center_size(pos2(c.x + dx * s, c.y), vec2(0.6 * s, 2.0 * s)),
                    2.0,
                    color,
                );
            }
        }
        Icon::Prev | Icon::Next => {
            let dir = if matches!(icon, Icon::Next) {
                1.0
            } else {
                -1.0
            };
            painter.add(triangle(pos2(c.x + dir * 0.6 * s, c.y), dir));
            painter.rect_filled(
                Rect::from_center_size(pos2(c.x + dir * 0.9 * s, c.y), vec2(0.35 * s, 2.0 * s)),
                1.0,
                color,
            );
        }
    }
    resp
}

fn up_next(ui: &mut Ui, state: &mut State) {
    panel().show(ui, |ui| {
        Styled::column().gap(6.0).full_width().show(ui, |ui| {
            text("Up next", Weight::Bold, 20.0, Color32::WHITE).show(ui);
            text(
                "From “Late Night Drive”",
                Weight::Regular,
                13.0,
                WHITE_MUTED,
            )
            .show(ui);
            ui.add_space(10.0);
            let mut hovered = None;
            for i in 0..state.tracks.len() {
                let resp = queue_row(ui, state, i);
                if resp.hovered() {
                    hovered = Some(i);
                }
                if resp.clicked() {
                    state.current = i;
                    state.position = 0.0;
                }
            }
            state.hovered = hovered;
        });
    });
}

fn queue_row(ui: &mut Ui, state: &State, i: usize) -> egui::Response {
    let track = &state.tracks[i];
    let current = i == state.current;
    let bg = match (current, state.hovered == Some(i)) {
        (true, _) => rgb(255, 255, 255).with_alpha(30),
        (false, true) => rgb(255, 255, 255).with_alpha(14),
        _ => Color32::TRANSPARENT,
    };
    let frame = Styled::frame()
        .bg(bg)
        .corner_radius(12.0)
        .padding(8.0)
        .full_width()
        .show(ui, |ui| {
            Styled::row()
                .full_width()
                .align(egui::Align::Center)
                .space_between()
                .item(|ui| {
                    Styled::row()
                        .gap(12.0)
                        .align(egui::Align::Center)
                        .show(ui, |ui| {
                            Styled::image(track.cover.clone())
                                .size(vec2(48.0, 48.0))
                                .corner_radius(8.0)
                                .show(ui);
                            Styled::column().gap(2.0).show(ui, |ui| {
                                text(track.title, Weight::Semibold, 14.0, Color32::WHITE).show(ui);
                                text(track.artist, Weight::Regular, 12.0, WHITE_MUTED).show(ui);
                            });
                        });
                })
                .item(|ui| {
                    let color = if current { Color32::WHITE } else { WHITE_MUTED };
                    text(clock(track.seconds as f32), Weight::Mono, 12.0, color).show(ui);
                })
                .show(ui);
        });
    ui.interact(
        frame.response.rect,
        ui.scope_id().with(("queue", i)),
        Sense::click(),
    )
}
