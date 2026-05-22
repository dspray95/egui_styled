use egui::{Align, InnerResponse, Layout, Ui};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

pub struct StyledColumn {
    gap: Option<f32>,
    align: Option<Align>,
    style: SharedStyle,
}

impl Default for StyledColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledColumn {
    pub fn new() -> Self {
        Self {
            gap: None,
            align: None,
            style: SharedStyle::default(),
        }
    }

    /// Vertical spacing between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Cross-axis (horizontal) alignment of children inside the column.
    /// Equivalent to `Layout::top_down(align)`.
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let gap = self.gap;
        let align = self.align;
        let render = move |ui: &mut Ui| {
            let layout = Layout::top_down(align.unwrap_or(Align::Min));
            ui.with_layout(layout, |ui| {
                if let Some(gap) = gap {
                    ui.spacing_mut().item_spacing.y = gap;
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

impl_style_builders!(StyledColumn);
