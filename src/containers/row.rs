use egui::{Align, InnerResponse, Layout, Ui};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

pub struct StyledRow {
    gap: Option<f32>,
    align: Option<Align>,
    style: SharedStyle,
}

impl Default for StyledRow {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledRow {
    pub fn new() -> Self {
        Self {
            gap: None,
            align: None,
            style: SharedStyle::default(),
        }
    }

    /// Horizontal spacing between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Cross-axis (vertical) alignment of children inside the row.
    /// Equivalent to `Layout::left_to_right(align)`.
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let gap = self.gap;
        let align = self.align;
        let render = move |ui: &mut Ui| {
            let layout = Layout::left_to_right(align.unwrap_or(Align::Center));
            ui.with_layout(layout, |ui| {
                if let Some(gap) = gap {
                    ui.spacing_mut().item_spacing.x = gap;
                }
                body(ui)
            })
        };

        if self.style.has_frame_styles() {
            let ir = StyledFrame {
                style: self.style,
                align: None,
            }
            .show(ui, |ui| render(ui).inner);
            InnerResponse::new(ir.inner, ir.response)
        } else {
            render(ui)
        }
    }
}

impl_style_builders!(StyledRow);
