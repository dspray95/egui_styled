use egui::{ComboBox, Id, InnerResponse, Stroke, Ui, WidgetText};

use crate::{impl_style_builders, style::shared_style::SharedStyle};

pub struct StyledComboBox {
    id_source: Id,
    selected_text: WidgetText,
    width: Option<f32>,
    style: SharedStyle,
}

impl StyledComboBox {
    pub fn new(id_source: impl std::hash::Hash, selected_text: impl Into<WidgetText>) -> Self {
        Self {
            id_source: Id::new(id_source),
            selected_text: selected_text.into(),
            width: None,
            style: SharedStyle::default(),
        }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        menu_contents: impl FnOnce(&mut Ui),
    ) -> InnerResponse<Option<()>> {
        let visuals = ui.visuals().clone();
        let widget_vis = &visuals.widgets.inactive;
        let resolved = self.style.resolve(Default::default(), widget_vis);

        ui.scope(|ui| {
            let vis = ui.visuals_mut();
            for ws in [
                &mut vis.widgets.inactive,
                &mut vis.widgets.hovered,
                &mut vis.widgets.active,
                &mut vis.widgets.open,
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
                    &mut vis.widgets.open,
                ] {
                    ws.fg_stroke = Stroke::new(1.0, color);
                }
            }

            let mut cb = ComboBox::from_id_salt(self.id_source).selected_text(self.selected_text);
            if let Some(w) = self.width {
                cb = cb.width(w);
            }
            cb.show_ui(ui, menu_contents)
        })
        .inner
    }
}

impl_style_builders!(StyledComboBox);
