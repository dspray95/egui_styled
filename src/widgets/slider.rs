use std::ops::RangeInclusive;

use egui::{Response, Slider, Stroke, Ui, emath::Numeric};

use crate::{impl_style_builders, state::PseudoState, style::shared_style::SharedStyle};

pub struct StyledSlider<'a, T: Numeric> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    text: Option<String>,
    step: Option<f64>,
    style: SharedStyle,
}

impl<'a, T: Numeric> StyledSlider<'a, T> {
    pub fn new(value: &'a mut T, range: RangeInclusive<T>) -> Self {
        Self {
            value,
            range,
            text: None,
            step: None,
            style: SharedStyle::default(),
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(ui.next_auto_id());
        let pseudo = PseudoState::load(ui, id);

        let visuals = ui.visuals().clone();
        let widget_vis = if pseudo.active {
            &visuals.widgets.active
        } else if pseudo.hovered {
            &visuals.widgets.hovered
        } else {
            &visuals.widgets.inactive
        };
        let resolved = self.style.resolve(pseudo, widget_vis);

        let response = ui
            .scope(|ui| {
                let vis = ui.visuals_mut();
                for ws in [
                    &mut vis.widgets.inactive,
                    &mut vis.widgets.hovered,
                    &mut vis.widgets.active,
                ] {
                    ws.bg_fill = resolved.bg;
                    ws.bg_stroke = resolved.border;
                    ws.corner_radius = resolved.corner_radius;
                }

                if let Some(color) = self.style.text_color {
                    for ws in [
                        &mut vis.widgets.inactive,
                        &mut vis.widgets.hovered,
                        &mut vis.widgets.active,
                    ] {
                        ws.fg_stroke = Stroke::new(1.0, color);
                    }
                }

                let mut slider = Slider::new(self.value, self.range);
                if let Some(t) = self.text {
                    slider = slider.text(t);
                }
                if let Some(s) = self.step {
                    slider = slider.step_by(s);
                }

                let mut wrapper = egui::Frame::new();
                if let Some(m) = self.style.margin {
                    wrapper = wrapper.outer_margin(m);
                }
                wrapper.show(ui, |ui| ui.add(slider)).inner
            })
            .inner;

        PseudoState::from_response(&response).store(ui, id);
        response
    }
}

impl_style_builders!(['a, T: Numeric], StyledSlider<'a, T>);
