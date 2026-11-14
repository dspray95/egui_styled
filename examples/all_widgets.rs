use egui::{Align2, CentralPanel, Color32, Margin, pos2};
use egui_styled::prelude::*;

#[derive(Default)]
struct AppState {
    tab: Tab,
    username: String,
    bio: String,
    volume: f32,
    frequency: f32,
    notifications: bool,
    dark_mode: bool,
    quality: Quality,
}

#[derive(Default, PartialEq, Copy, Clone)]
enum Tab {
    #[default]
    Profile,
    Audio,
    Advanced,
}

#[derive(Default, PartialEq, Copy, Clone)]
enum Quality {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

impl Quality {
    fn label(self) -> &'static str {
        match self {
            Quality::Low => "Low",
            Quality::Medium => "Medium",
            Quality::High => "High",
            Quality::Ultra => "Ultra",
        }
    }
}

fn all_widgets(ui: &mut egui::Ui, state: &mut AppState) {
    let t = ui.ctx().styled_theme();
    let p = ui.ctx().design_data::<WebPalette>();

    const CARD_WIDTH: f32 = 440.0;
    const CARD_HEIGHT_ESTIMATE: f32 = 420.0;
    ui.vertical_centered(|ui| {
        // Roughly centre on Y, but always keep a top margin on short windows.
        let top = ((ui.available_height() - CARD_HEIGHT_ESTIMATE) / 2.0).max(32.0);
        ui.add_space(top);
        Styled::frame()
            .bg(p.bg_surface)
            .corner_radius(t.rounding_lg)
            .padding(28.0)
            .border(1.0, p.border)
            .inner_glow(16.0, Color32::from_white_alpha(8))
            .min_width(CARD_WIDTH)
            .max_width(CARD_WIDTH)
            .show(ui, |ui| {
                Styled::column().gap(22.0).full_width().show(ui, |ui| {
                    Styled::column().gap(2.0).show(ui, |ui| {
                        Styled::label("SETTINGS")
                            .font(font("w600", 11.0))
                            .text_color(p.fg_muted)
                            .show(ui);

                        Styled::label("Account")
                            .font(font("w700", 22.0))
                            .text_color(p.fg_primary)
                            .show(ui);
                    });

                    // --- Tab bar: flush text tabs, no container fill. The selected tab is
                    // marked by faint side rules plus a hint of fill - accent stays unique to Save.
                    Styled::row().gap(0.0).full_width().show(ui, |ui| {
                        for (tab, label) in [
                            (Tab::Profile, "Profile"),
                            (Tab::Audio, "Audio"),
                            (Tab::Advanced, "Advanced"),
                        ] {
                            let selected = state.tab == tab;
                            let mut button = Styled::button(label)
                                .bg(Color32::TRANSPARENT)
                                .hover_bg(Color32::from_white_alpha(8))
                                .hover_border_x(1.0, p.border)
                                // Unset sides fall back to egui's default stroke, so zero
                                // top/bottom explicitly to keep the rules to the sides only.
                                .hover_border_y(0.0, Color32::TRANSPARENT)
                                .text_color(if selected { p.fg_primary } else { p.fg_muted })
                                .font(font("w500", 13.0))
                                .corner_radius(0.0)
                                .padding(Margin::symmetric(18, 9));
                            if selected {
                                button = button
                                    .bg(Color32::from_white_alpha(10))
                                    .hover_bg(Color32::from_white_alpha(14))
                                    .border_x(1.0, p.border)
                                    .border_y(0.0, Color32::TRANSPARENT);
                            }
                            if button.show(ui).clicked() {
                                state.tab = tab;
                            }
                        }
                    });

                    // Fixed-height content area so switching tabs doesn't resize the card.
                    Styled::column()
                        .gap(14.0)
                        .full_width()
                        .min_height(190.0)
                        .show(ui, |ui| match state.tab {
                            Tab::Profile => profile_tab(ui, state, &t, &p),
                            Tab::Audio => audio_tab(ui, state, &t, &p),
                            Tab::Advanced => advanced_tab(ui, state, &t, &p),
                        });

                    // --- Button row, pinned bottom-right ---
                    // Right-to-left layout: add Save first so it ends up rightmost.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Styled::button("Save")
                            .bg(p.accent)
                            .hover_bg(p.accent_hover)
                            .active_bg(p.accent_active)
                            .text_color(p.fg_on_accent)
                            .font(font("w600", 14.0))
                            .corner_radius(t.rounding_md)
                            .padding(Margin::symmetric(18, 10))
                            .inner_glow(10.0, Color32::from_white_alpha(28))
                            .show(ui)
                            .clicked()
                        {
                            println!("saved: {:?}", state.quality.label());
                        }

                        ui.add_space(t.spacing_sm);

                        Styled::button("Cancel")
                            .bg(Color32::from_white_alpha(8))
                            .hover_bg(Color32::from_white_alpha(16))
                            .text_color(p.fg_secondary)
                            .font(font("w500", 14.0))
                            .corner_radius(t.rounding_md)
                            .border(1.0, p.border)
                            .padding(Margin::symmetric(18, 10))
                            .show(ui);
                    });
                });
            });
    });

    // --- Draggable floating tips panel (StyledArea) ---
    let ctx = ui.ctx().clone();
    let screen = ctx.content_rect();
    Styled::area()
        .id("tips_panel")
        .default_pos(pos2(screen.right() - 24.0, screen.top() + 24.0))
        .pivot(Align2::RIGHT_TOP)
        .movable(true)
        .show(&ctx, |ui| {
            // Generous padding gives a grab margin around the (selectable) text.
            Styled::frame()
                .bg(p.bg_elevated)
                .corner_radius(t.rounding_md)
                .padding(t.spacing_lg)
                .border(1.0, p.border)
                .inner_glow(12.0, Color32::from_white_alpha(10))
                .show(ui, |ui| {
                    Styled::label("Drag me around")
                        .font_size(t.font_size_sm)
                        .text_color(p.fg_secondary)
                        .show(ui);
                });
        });
}

fn profile_tab(ui: &mut egui::Ui, state: &mut AppState, t: &StyledTheme, p: &WebPalette) {
    Styled::text_edit(&mut state.username)
        .hint("Username")
        .full_width()
        .bg(p.bg_secondary)
        .corner_radius(t.rounding_md)
        .padding(Margin::symmetric(10, 8))
        .border(1.0, p.border)
        .focus_border(1.0, p.border_focus)
        .show(ui);

    Styled::text_edit(&mut state.bio)
        .hint("Short bio")
        .multiline()
        .full_width()
        .bg(p.bg_secondary)
        .corner_radius(t.rounding_md)
        .padding(Margin::symmetric(10, 8))
        .border(1.0, p.border)
        .focus_border(1.0, p.border_focus)
        .show(ui);
}

fn slider_style<'a>(s: StyledSlider<'a, f32>, p: &WebPalette) -> StyledSlider<'a, f32> {
    s.fill(p.accent)
        .rail(Color32::from_white_alpha(22))
        .rail_thickness(5.0)
        .handle(Color32::WHITE)
        .handle_radius(7.0)
        .text_color(p.fg_secondary)
        .value_bg(Color32::from_white_alpha(10))
        .corner_radius(6.0)
}

fn audio_tab(ui: &mut egui::Ui, state: &mut AppState, _t: &StyledTheme, p: &WebPalette) {
    Styled::slider(&mut state.volume, 0.0..=1.0)
        .text("Volume")
        .step(0.01)
        .apply(|s| slider_style(s, p))
        .show(ui);

    // Logarithmic rail with a unit suffix in the value box.
    Styled::slider(&mut state.frequency, 20.0..=20_000.0)
        .text("Frequency")
        .logarithmic(true)
        .suffix(" Hz")
        .apply(|s| slider_style(s, p))
        .show(ui);
}

fn advanced_tab(ui: &mut egui::Ui, state: &mut AppState, t: &StyledTheme, p: &WebPalette) {
    // Collapsing section: unframed header, just the title text and the caret.
    Styled::collapsing("Preferences")
        .default_open(true)
        .text_color(p.fg_secondary)
        .show(ui, |ui| {
            Styled::column().gap(t.spacing_md).show(ui, |ui| {
                Styled::checkbox(&mut state.notifications, "Enable notifications")
                    .text_color(p.fg_secondary)
                    .show(ui);

                Styled::checkbox(&mut state.dark_mode, "Use dark mode")
                    .text_color(p.fg_secondary)
                    .show(ui);

                Styled::row().gap(t.spacing_sm).show(ui, |ui| {
                    Styled::label("Quality").text_color(p.fg_secondary).show(ui);

                    Styled::combo_box("quality_select", state.quality.label())
                        .width(120.0)
                        .bg(p.bg_secondary)
                        .corner_radius(t.rounding_md)
                        .border(1.0, p.border)
                        .show(ui, |ui| {
                            for q in [Quality::Low, Quality::Medium, Quality::High, Quality::Ultra]
                            {
                                ui.selectable_value(&mut state.quality, q, q.label());
                            }
                        });
                });
            });
        });
}

/// Register Inter 400/500/600/700, each as its own named family (egui has no
/// synthetic bold), and point the default proportional text at Inter 400.
fn install_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Arc;

    let mut defs = FontDefinitions::default();
    let fallback = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let with_fallback = |key: &str| {
        std::iter::once(key.to_owned())
            .chain(fallback.iter().cloned())
            .collect::<Vec<_>>()
    };

    for (name, bytes) in [
        (
            "w400",
            include_bytes!("showcase/assets/fonts/inter/inter-400.ttf").as_slice(),
        ),
        (
            "w500",
            include_bytes!("showcase/assets/fonts/inter/inter-500.ttf").as_slice(),
        ),
        (
            "w600",
            include_bytes!("showcase/assets/fonts/inter/inter-600.ttf").as_slice(),
        ),
        (
            "w700",
            include_bytes!("showcase/assets/fonts/inter/inter-700.ttf").as_slice(),
        ),
    ] {
        defs.font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
        defs.families
            .insert(FontFamily::Name(name.into()), with_fallback(name));
    }
    defs.families
        .insert(FontFamily::Proportional, with_fallback("w400"));
    ctx.set_fonts(defs);
}

/// A `FontId` in one of the registered Inter weights.
fn font(weight: &str, size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(weight.into()))
}

/// Generous radii and the Inter weights, like the dark showcase.
fn dark_geometry() -> StyledTheme {
    use egui::{CornerRadius, FontFamily};
    StyledTheme {
        rounding_sm: CornerRadius::same(8),
        rounding_md: CornerRadius::same(10),
        rounding_lg: CornerRadius::same(14),
        font_family_display: FontFamily::Name("w600".into()),
        font_family_bold: Some(FontFamily::Name("w700".into())),
        ..Default::default()
    }
}

/// Linear / Vercel: near-black surfaces, hairline white borders, one violet accent.
fn dark_palette() -> WebPalette {
    WebPalette {
        bg_primary: Color32::from_rgb(9, 9, 11),
        bg_secondary: Color32::from_rgb(17, 17, 21),
        bg_surface: Color32::from_rgb(17, 17, 21),
        bg_elevated: Color32::from_rgb(28, 28, 34),

        fg_primary: Color32::from_rgb(244, 244, 245),
        fg_secondary: Color32::from_rgb(212, 212, 216),
        fg_muted: Color32::from_rgb(161, 161, 170),
        fg_on_accent: Color32::WHITE,

        accent: Color32::from_rgb(124, 58, 237),
        accent_hover: Color32::from_rgb(139, 92, 246),
        accent_active: Color32::from_rgb(109, 40, 217),

        error: Color32::from_rgb(248, 113, 113),
        warning: Color32::from_rgb(234, 179, 8),
        success: Color32::from_rgb(52, 211, 153),

        // Hairline borders: white at low alpha, not a solid gray line.
        border: Color32::from_white_alpha(22),
        border_focus: Color32::from_rgb(139, 92, 246),
    }
}

fn main() -> eframe::Result<()> {
    let mut state = AppState {
        volume: 0.5,
        frequency: 440.0,
        ..Default::default()
    };
    let mut initialized = false;
    eframe::run_ui_native(
        "egui_styled all widgets",
        eframe::NativeOptions::default(),
        move |ctx, _| {
            if !initialized {
                install_fonts(ctx);
                ctx.set_styled_theme(dark_geometry());
                ctx.set_design_data(dark_palette());
                initialized = true;
            }
            let panel = egui::Frame::new().fill(dark_palette().bg_primary);
            CentralPanel::default()
                .frame(panel)
                .show(ctx, |ui| all_widgets(ui, &mut state));
        },
    )
}
