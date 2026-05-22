use egui::{Align, InnerResponse, Layout, Ui};

use crate::{impl_style_builders, style::shared_style::SharedStyle};

pub struct StyledFrame {
    pub style: SharedStyle,
    pub align: Option<Align>,
}

impl Default for StyledFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledFrame {
    pub fn new() -> Self {
        Self {
            style: SharedStyle::default(),
            align: None,
        }
    }

    /// Cross-axis (horizontal) alignment of the frame's children.
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let mut frame = egui::Frame::default();
        if let Some(bg) = self.style.bg {
            frame = frame.fill(bg);
        }
        if let Some(r) = self.style.corner_radius {
            frame = frame.corner_radius(r);
        }
        if let Some(p) = self.style.padding {
            frame = frame.inner_margin(p);
        }
        if let Some(m) = self.style.margin {
            frame = frame.outer_margin(m);
        }
        if let Some(b) = self.style.border {
            frame = frame.stroke(b);
        }

        let full_width = self.style.full_width;
        let align = self.align;

        frame.show(ui, |ui| {
            if full_width {
                ui.set_min_width(ui.available_width());
            }
            if let Some(a) = align {
                ui.with_layout(Layout::top_down(a), body).inner
            } else {
                body(ui)
            }
        })
    }
}

impl_style_builders!(StyledFrame);
