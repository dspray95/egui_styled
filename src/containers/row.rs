use egui::{Align, InnerResponse, Layout, Ui};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

pub struct StyledRow {
    gap: Option<f32>,
    align: Option<Align>,
    justify: Option<Align>,
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
            justify: None,
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

    /// Main-axis (horizontal) distribution of children. `Align::Min` packs
    /// to the left, `Align::Center` packs centered, `Align::Max` packs to
    /// the right. Does **not** implement flexbox's `space-between` /
    /// `space-around` — see the README's "Layout" section for why.
    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = Some(justify);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let gap = self.gap;
        let align = self.align;
        let justify = self.justify;
        let render = move |ui: &mut Ui| {
            let mut layout = Layout::left_to_right(align.unwrap_or(Align::Center));
            if let Some(j) = justify {
                layout = layout.with_main_align(j);
            }
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
                justify: None,
            }
            .show(ui, |ui| render(ui).inner);
            InnerResponse::new(ir.inner, ir.response)
        } else {
            render(ui)
        }
    }
}

impl_style_builders!(StyledRow);
