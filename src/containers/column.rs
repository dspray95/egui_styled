use egui::{InnerResponse, Ui};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

pub struct StyledColumn {
    gap: Option<f32>,
    style: SharedStyle,
}

impl StyledColumn {
    pub fn new() -> Self {
        Self {
            gap: None,
            style: SharedStyle::default(),
        }
    }

    // Builder fn
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    // Render fn
    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> InnerResponse<()> {
        let gap = self.gap;
        let vertical = move |ui: &mut Ui| {
            ui.vertical(|ui| {
                if let Some(gap) = gap {
                    ui.spacing_mut().item_spacing.y = gap;
                }
                add_contents(ui);
            })
            .response
        };

        if self.style.has_frame_styles() {
            StyledFrame { style: self.style }.show(ui, |ui| {
                vertical(ui);
            })
        } else {
            let response = vertical(ui);
            InnerResponse::new((), response)
        }
    }
}

impl_style_builders!(StyledColumn);
