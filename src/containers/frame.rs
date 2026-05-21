use egui::{InnerResponse, Ui};

use crate::{impl_style_builders, style::shared_style::SharedStyle};

pub struct StyledFrame {
    pub style: SharedStyle,
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
        }
    }

    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> InnerResponse<()> {
        // Build the egui::Frame from our resolved style
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

        frame.show(ui, |ui| {
            if self.style.full_width {
                ui.set_min_width(ui.available_width());
            }
            add_contents(ui);
        })
    }
}

impl_style_builders!(StyledFrame);
