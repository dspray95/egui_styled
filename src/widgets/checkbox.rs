use egui::{Checkbox, Id, Response, Stroke, Ui, WidgetText};

use crate::{impl_style_builders, state::PseudoState, style::shared_style::SharedStyle};

pub struct StyledCheckbox<'a> {
    checked: &'a mut bool,
    label: WidgetText,
    id_override: Option<Id>,
    style: SharedStyle,
}

impl<'a> StyledCheckbox<'a> {
    pub fn new(checked: &'a mut bool, label: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            label: label.into(),
            id_override: None,
            style: SharedStyle::default(),
        }
    }

    /// Override the auto-generated widget id. Pins pseudo-state across
    /// conditional rendering - see [`crate::StyledButton::id`].
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));
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

                let mut wrapper = egui::Frame::new();
                if let Some(m) = self.style.margin {
                    wrapper = wrapper.outer_margin(m);
                }
                wrapper
                    .show(ui, |ui| ui.add(Checkbox::new(self.checked, self.label)))
                    .inner
            })
            .inner;

        PseudoState::from_response(&response).store(ui, id);
        response
    }
}

impl_style_builders!(StyledCheckbox<'_>);
