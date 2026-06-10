use std::ops::RangeInclusive;

use egui::{Id, Response, Shape, Slider, Ui, emath::Numeric};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{SharedStyle, paint_shadows, render_scoped},
};

pub struct StyledSlider<'a, T: Numeric> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    text: Option<String>,
    step: Option<f64>,
    id_override: Option<Id>,
    style: SharedStyle,
}

impl<'a, T: Numeric> StyledSlider<'a, T> {
    pub fn new(value: &'a mut T, range: RangeInclusive<T>) -> Self {
        Self {
            value,
            range,
            text: None,
            step: None,
            id_override: None,
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

    /// Override the auto-generated widget id. Pins pseudo-state across
    /// conditional rendering - see [`crate::StyledButton::id`].
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let visible = self.style.visible != Some(false);

        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));
        let _pseudo = PseudoState::load(ui, id);

        let per = self.style.resolve_per_state(ui.visuals());

        let response = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);
            let response = ui
                .scope(|ui| {
                    // selection.bg_fill drives the slider trailing fill.
                    SharedStyle::apply_to_visuals(&per, ui.visuals_mut());

                    let mut slider = Slider::new(self.value, self.range);
                    if let Some(t) = self.text {
                        slider = slider.text(t);
                    }
                    if let Some(s) = self.step {
                        slider = slider.step_by(s);
                    }

                    let mut wrapper = egui::Frame::new();
                    if per.margin != egui::Margin::ZERO {
                        wrapper = wrapper.outer_margin(per.margin);
                    }
                    if per.padding != egui::Margin::ZERO {
                        wrapper = wrapper.inner_margin(per.padding);
                    }
                    wrapper
                        .show(ui, |ui| {
                            let avail_w = ui.available_width();
                            let avail_h = ui.available_height();
                            if let Some(w) = self.style.resolved_width_pct(avail_w) {
                                ui.set_min_width(w);
                                ui.set_max_width(w);
                            } else {
                                if self.style.full_width {
                                    ui.set_min_width(avail_w);
                                }
                                if let Some(min_w) = self.style.min_width {
                                    ui.set_min_width(min_w);
                                }
                            }
                            if let Some(h) = self.style.resolved_height_pct(avail_h) {
                                ui.set_min_height(h);
                                ui.set_max_height(h);
                            } else if let Some(min_h) = self.style.min_height {
                                ui.set_min_height(min_h);
                            }
                            ui.add(slider)
                        })
                        .inner
                })
                .inner;

            SharedStyle::paint_widget_side_borders(ui, &response, &per);
            paint_shadows(
                ui,
                shadow_idx,
                response.rect,
                per.corner_radius,
                &self.style.shadows,
            );
            response
        });

        PseudoState::from_response(&response).store(ui, id);

        if let Some(icon) = per.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        response
    }
}

impl_style_builders!(['a, T: Numeric], StyledSlider<'a, T>);
