use egui::{Checkbox, Id, Response, RichText, Shape, Ui, WidgetText};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{
        SharedStyle, paint_shadows, paint_widget_gradient_underlay, paint_widget_overlays,
        render_scoped,
    },
};

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
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let visible = self.style.visible != Some(false);

        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));
        let pseudo = PseudoState::load(ui, id);

        let per = self.style.resolve_per_state(ui.visuals());

        // Apply font to label if requested.
        let label: WidgetText = if let Some(size) = self.style.font_size {
            let rt = match self.label {
                WidgetText::RichText(rt) => (*rt).clone().size(size),
                other => RichText::new(other.text().to_string()).size(size),
            };
            rt.into()
        } else {
            self.label
        };

        let response = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);
            let gradient_idx = ui.painter().add(Shape::Noop);
            let response = ui
                .scope(|ui| {
                    SharedStyle::apply_to_visuals(&per, pseudo, ui.visuals_mut());

                    let mut wrapper = egui::Frame::new();
                    if per.margin != egui::Margin::ZERO {
                        wrapper = wrapper.outer_margin(per.margin);
                    }
                    if per.padding != egui::Margin::ZERO {
                        wrapper = wrapper.inner_margin(per.padding);
                    }
                    wrapper
                        .show(ui, |ui| {
                            self.style
                                .resolve_size(ui.available_width(), ui.available_height())
                                .apply_to_ui(ui);
                            ui.add(Checkbox::new(self.checked, label))
                        })
                        .inner
                })
                .inner;

            let resolved = SharedStyle::for_response(&per, &response);
            paint_widget_gradient_underlay(
                ui,
                gradient_idx,
                response.rect,
                per.corner_radius,
                resolved,
            );
            paint_widget_overlays(ui, response.rect, resolved);
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

impl_style_builders!(StyledCheckbox<'_>);
crate::impl_styled_widget!(['a], StyledCheckbox<'a>);
