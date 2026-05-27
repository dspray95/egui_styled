use egui::{ComboBox, Id, InnerResponse, RichText, Shape, Ui, WidgetText};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{SharedStyle, paint_shadows},
};

pub struct StyledComboBox {
    id_source: Id,
    selected_text: WidgetText,
    width: Option<f32>,
    id_override: Option<Id>,
    style: SharedStyle,
}

impl StyledComboBox {
    pub fn new(id_source: impl std::hash::Hash, selected_text: impl Into<WidgetText>) -> Self {
        Self {
            id_source: Id::new(id_source),
            selected_text: selected_text.into(),
            width: None,
            id_override: None,
            style: SharedStyle::default(),
        }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// Override the auto-generated widget id. Pins pseudo-state across
    /// conditional rendering - see [`crate::StyledButton::id`] for the rationale.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        menu_contents: impl FnOnce(&mut Ui),
    ) -> InnerResponse<Option<()>> {
        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(self.id_source));
        let _pseudo = PseudoState::load(ui, id);

        let per = self.style.resolve_per_state(ui.visuals());
        let shadow_idx = ui.painter().add(Shape::Noop);

        let result = ui
            .scope(|ui| {
                SharedStyle::apply_to_visuals(&per, ui.visuals_mut());

                // Apply font to selected_text if requested.
                let selected_text: WidgetText =
                    if let Some(size) = self.style.font_size {
                        let rt = match self.selected_text {
                            WidgetText::RichText(rt) => (*rt).clone().size(size),
                            other => RichText::new(other.text().to_string()).size(size),
                        };
                        rt.into()
                    } else {
                        self.selected_text
                    };

                let mut cb =
                    ComboBox::from_id_salt(self.id_source).selected_text(selected_text);
                if let Some(w) = self.width {
                    cb = cb.width(w);
                }

                let mut wrapper = egui::Frame::new();
                if per.margin != egui::Margin::ZERO {
                    wrapper = wrapper.outer_margin(per.margin);
                }
                if per.padding != egui::Margin::ZERO {
                    wrapper = wrapper.inner_margin(per.padding);
                }
                wrapper.show(ui, |ui| cb.show_ui(ui, menu_contents)).inner
            })
            .inner;

        // Use the outer response rect for shadows and pseudo-state.
        let response = &result.response;
        paint_shadows(
            ui,
            shadow_idx,
            response.rect,
            per.corner_radius,
            &self.style.shadows,
        );

        PseudoState::from_response(response).store(ui, id);

        if let Some(icon) = per.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        result
    }
}

impl_style_builders!(StyledComboBox);
