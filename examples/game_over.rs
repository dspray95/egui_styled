//! Example: a complete arcade game-over screen.
//!
//! The other examples each exercise one slice of the API (a widget, a
//! container, the theme system in isolation). This one shows what a real
//! screen looks like when you compose them - `Styled::area` for the
//! full-screen backdrop and the modal, `Styled::column` with `gap` + `align`
//! for the vertical layout, every styled widget for content, and a custom
//! color struct stored alongside `StyledTheme` via `DesignSlots`.
//!
//! Two phases drive the screen:
//!
//! - `EnteringInitials` - text edit with focus-border styling + SUBMIT button.
//! - `Showing` - leaderboard with the just-placed row highlighted, plus a
//!   "press to play again" prompt.
//!
//! Run with: `cargo run --example game_over`

use egui::{Align, Align2, Color32, FontFamily, Margin, Order, Vec2};
use egui_styled::prelude::*;

const FINAL_SCORE: i32 = 18_350;
const MAX_INITIALS: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    EnteringInitials,
    Showing,
}

struct AppState {
    phase: Phase,
    initials_buffer: String,
    entries: Vec<(String, i32)>,
    submitted_index: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            phase: Phase::EnteringInitials,
            initials_buffer: String::new(),
            entries: vec![
                ("AAA".into(), 25_000),
                ("BBB".into(), 21_400),
                ("CCC".into(), 15_700),
                ("DDD".into(), 12_100),
                ("EEE".into(), 9_300),
            ],
            submitted_index: None,
        }
    }
}

/// App-defined color vocabulary. Names describe roles in this UI rather than generic
/// web slots
#[derive(Clone, Debug, PartialEq)]
struct ArcadeColors {
    background: Color32,
    panel_surface: Color32,
    panel_elevated: Color32,
    text: Color32,
    text_muted: Color32,
    hud_cyan: Color32,
    hud_cyan_bright: Color32,
    input_border: Color32,
    input_magenta: Color32,
    highlight_gold: Color32,
}

impl Default for ArcadeColors {
    fn default() -> Self {
        Self {
            background: Color32::from_rgb(8, 4, 16),
            panel_surface: Color32::from_rgb(16, 10, 28),
            panel_elevated: Color32::from_rgb(36, 24, 56),
            text: Color32::WHITE,
            text_muted: Color32::from_gray(120),
            hud_cyan: Color32::from_rgb(0, 220, 255),
            hud_cyan_bright: Color32::from_rgb(120, 240, 255),
            input_border: Color32::from_rgb(80, 30, 110),
            input_magenta: Color32::from_rgb(255, 0, 200),
            highlight_gold: Color32::from_rgb(255, 215, 0),
        }
    }
}

fn arcade_theme() -> StyledTheme {
    // Override only the deltas from `StyledTheme::default()`.
    StyledTheme {
        font_size_sm: 13.0,
        font_size_md: 16.0,
        font_size_xl: 40.0,
        font_family_display: FontFamily::Proportional,
        ..Default::default()
    }
}

fn game_over(ctx: &egui::Context, state: &mut AppState) {
    // `design::<T>()` fetches both `StyledTheme` and your custom typed
    // slot in one call. Equivalent to `(ctx.styled_theme(), ctx.design_data::<T>())`
    // but reads as a single binding.
    let (theme, colors) = ctx.design::<ArcadeColors>();

    // Full-screen backdrop. In a real game this would use
    // `colors.background.with_alpha(180)` so the gameplay behind shows
    // through dimmed - `with_alpha` is the one-chain replacement for the
    // byte-deconstruction dance you'd otherwise need.
    Styled::area()
        .id("game_over_backdrop")
        .order(Order::Background)
        .fill_screen()
        .bg(colors.background.with_alpha(255))
        .show(ctx, |_| {});

    // Modal panel - `Styled::area` operates on `&Context` because top-level
    // positioned things don't have a parent `Ui`. Compose box-style builders
    // (bg, border, padding, corner_radius) directly on the area.
    Styled::area()
        .id("game_over_panel")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .bg(colors.panel_surface)
        .border(1.0, colors.input_border)
        .padding(Margin::same(24))
        .corner_radius(theme.rounding_md)
        .show(ctx, |ui| {
            // One column carries the whole panel's vertical rhythm and
            // horizontal alignment. Child widgets inherit centering - no
            // `ui.vertical_centered` wrappers needed.
            Styled::column()
                .gap(theme.spacing_lg)
                .align(Align::Center)
                .show(ui, |ui| {
                    score_block(ui, &theme, &colors);

                    if state.phase == Phase::EnteringInitials {
                        initials_block(ui, &theme, &colors, state);
                    }

                    leaderboard(ui, &theme, &colors, state);
                    play_again_hint(ui, &theme, &colors);
                });
        });
}

fn score_block(ui: &mut egui::Ui, theme: &StyledTheme, colors: &ArcadeColors) {
    Styled::label("YOUR SCORE")
        .font(theme.font_display(theme.font_size_md))
        .text_color(colors.hud_cyan)
        .show(ui);
    Styled::label(format!("{:09}", FINAL_SCORE))
        .font(theme.font_display(theme.font_size_xl))
        .text_color(colors.text)
        .show(ui);
}

fn initials_block(
    ui: &mut egui::Ui,
    theme: &StyledTheme,
    colors: &ArcadeColors,
    state: &mut AppState,
) {
    let row_font = theme.font_display(theme.font_size_sm);

    Styled::label("ENTER INITIALS")
        .font(theme.font_display(theme.font_size_md))
        .text_color(colors.input_magenta)
        .show(ui);

    // The text edit picks up focus styling via `focus_border`, the magenta
    // glow appears automatically when the field is focused.
    let response = Styled::text_edit(&mut state.initials_buffer)
        .char_limit(MAX_INITIALS)
        .font(row_font.clone())
        .desired_width(120.0)
        .horizontal_align(Align::Center)
        .bg(colors.panel_surface)
        .text_color(colors.text)
        .border(1.0, colors.input_border)
        .focus_border(1.0, colors.input_magenta)
        .corner_radius(theme.rounding_sm)
        .show(ui);
    response.request_focus();

    // Sanitize to uppercase alphanumerics, max MAX_INITIALS chars.
    let cleaned: String = state
        .initials_buffer
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .take(MAX_INITIALS)
        .collect();
    state.initials_buffer = cleaned;

    let submit_clicked = Styled::button("SUBMIT")
        .font(row_font)
        .bg(Color32::TRANSPARENT)
        .hover_bg(colors.panel_elevated)
        .text_color(colors.hud_cyan)
        .border(1.0, colors.hud_cyan)
        .hover_border(1.0, colors.hud_cyan_bright)
        .corner_radius(theme.rounding_sm)
        .min_width(180.0)
        .min_height(36.0)
        .margin_top(theme.spacing_md)
        .show(ui)
        .clicked();

    if submit_clicked && !state.initials_buffer.is_empty() {
        // Pad short input so leaderboard rendering stays aligned.
        while state.initials_buffer.len() < MAX_INITIALS {
            state.initials_buffer.push('_');
        }
        let initials = state.initials_buffer.clone();
        state.entries.push((initials, FINAL_SCORE));
        state
            .entries
            .sort_by_key(|entry| std::cmp::Reverse(entry.1));
        state.entries.truncate(10);
        state.submitted_index = state
            .entries
            .iter()
            .position(|(_, score)| *score == FINAL_SCORE);
        state.phase = Phase::Showing;
        state.initials_buffer.clear();
    }
}

fn leaderboard(ui: &mut egui::Ui, theme: &StyledTheme, colors: &ArcadeColors, state: &AppState) {
    let row_font = theme.font_display(theme.font_size_sm);

    Styled::label("HIGH SCORES")
        .font(theme.font_display(theme.font_size_md))
        .text_color(colors.hud_cyan)
        .margin_bottom(theme.spacing_sm)
        .show(ui);

    for slot in 0..10 {
        let (initials, score) = state
            .entries
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ("---".into(), 0));
        // Highlight the just-placed entry; muted text for empty slots.
        let color = if state.submitted_index == Some(slot) {
            colors.highlight_gold
        } else if state.entries.get(slot).is_some() {
            colors.text
        } else {
            colors.text_muted
        };
        Styled::label(format!("{:>2}.  {}   {:09}", slot + 1, initials, score))
            .font(row_font.clone())
            .text_color(color)
            .show(ui);
    }
}

fn play_again_hint(ui: &mut egui::Ui, theme: &StyledTheme, colors: &ArcadeColors) {
    const BLINK_PERIOD: f64 = 1.0;
    // Chromatic-aberration offset for the [ENTER] glitch, in pixels.
    const GLITCH_OFFSET: f32 = 1.5;
    let now = ui.input(|i| i.time);
    let visible = (now % BLINK_PERIOD) < (BLINK_PERIOD / 2.0);

    let row_font = theme.font_display(theme.font_size_sm);
    let row_height = row_font.size + 4.0;

    // A tight row of `.extend()` labels (no truncation even when space is
    // tight), with `[ENTER]` rendered as three offset layers via `stack()` for
    // a chromatic-aberration glitch. `.min_height` reserves the slot every
    // frame so the leaderboard above doesn't reflow on each blink.
    Styled::row()
        .gap(0.0)
        .align(Align::Center)
        .min_height(row_height)
        .visible(visible)
        .show(ui, |ui| {
            let label = |text: &str, color: Color32| {
                Styled::label(text.to_owned())
                    .font(row_font.clone())
                    .text_color(color)
                    .extend()
            };
            label("PRESS ", colors.text).show(ui);
            Styled::stack()
                .layer_offset(Vec2::new(-GLITCH_OFFSET, 0.0), |ui| {
                    label("[ENTER]", colors.hud_cyan).show(ui);
                })
                .layer_offset(Vec2::new(GLITCH_OFFSET, 0.0), |ui| {
                    label("[ENTER]", colors.input_magenta).show(ui);
                })
                .layer(|ui| {
                    label("[ENTER]", colors.text).show(ui);
                })
                .show(ui);
            label(" TO PLAY AGAIN", colors.text).show(ui);
        });

    ui.ctx().request_repaint();
}

fn main() -> eframe::Result<()> {
    let mut state = AppState::default();
    let mut initialized = false;
    eframe::run_ui_native(
        "egui_styled - game over",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                ctx.set_styled_theme(arcade_theme());
                ctx.set_design_data(ArcadeColors::default());
                initialized = true;
            }
            game_over(ctx, &mut state);
        },
    )
}
