use egui::{Image, Response, Stroke, Ui, WidgetText};

use crate::{impl_style_builders, state::PseudoState, style::shared_style::SharedStyle};

pub struct StyledButton {
    text: WidgetText,
    image: Option<Image<'static>>,
    style: SharedStyle,
}

impl StyledButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            image: None,
            style: SharedStyle::default(),
        }
    }

    // Widget specific builder
    pub fn image(mut self, img: Image<'static>) -> Self {
        self.image = Some(img);
        self
    }

    /// Renders the button with the given style
    pub fn show(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(ui.next_auto_id());

        let psuedo = PseudoState::load(ui, id);

        let visuals = ui.visuals().clone();
        let widget_vis = if psuedo.active {
            &*&visuals.widgets.active
        } else if psuedo.hovered {
            &visuals.widgets.hovered
        } else {
            &visuals.widgets.inactive
        };
        let resolved = self.style.resolve(psuedo, widget_vis);

        if self.style.full_width {
            ui.set_min_width(ui.available_width());
        }

        // Override the widget visuals for all states in scope
        let response = ui
            .scope(|ui| {
                let vis = ui.visuals_mut();
                // Default state
                vis.widgets.inactive.bg_fill = resolved.bg;
                vis.widgets.inactive.bg_stroke = resolved.border;
                vis.widgets.inactive.corner_radius = resolved.corner_radius;
                // Hovered
                vis.widgets.hovered.bg_fill = resolved.bg;
                vis.widgets.hovered.bg_stroke = resolved.border;
                vis.widgets.hovered.corner_radius = resolved.corner_radius;
                // Active
                vis.widgets.active.bg_fill = resolved.bg;
                vis.widgets.active.bg_stroke = resolved.border;
                vis.widgets.active.corner_radius = resolved.corner_radius;

                if let Some(color) = self.style.text_color {
                    vis.widgets.inactive.fg_stroke = Stroke::new(1.0, color);
                    vis.widgets.hovered.fg_stroke = Stroke::new(1.0, color);
                    vis.widgets.active.fg_stroke = Stroke::new(1.0, color);
                }

                // Build the actual egui widget
                let mut btn = match self.image {
                    Some(img) => egui::Button::opt_image_and_text(Some(img), Some(self.text)),
                    None => egui::Button::new(self.text),
                };
                if self.style.full_width {
                    btn = btn.min_size(egui::Vec2 {
                        x: ui.available_width(),
                        y: 0.0,
                    })
                }

                let mut wrapper = egui::Frame::new();
                if let Some(m) = self.style.margin {
                    wrapper = wrapper.outer_margin(m);
                }
                wrapper.show(ui, |ui| ui.add(btn)).inner
            })
            .inner;

        // Store this frame's state for next frame
        PseudoState::from_response(&response).store(ui, id);

        // Handle cursor override
        if let Some(icon) = resolved.cursor_icon {
            if response.hovered() {
                ui.ctx().set_cursor_icon(icon);
            }
        }

        response
    }
}

impl_style_builders!(StyledButton);
