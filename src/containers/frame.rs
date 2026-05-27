use egui::{Align, InnerResponse, Layout, Shape, Ui};

use crate::{impl_style_builders, style::shared_style::{SharedStyle, paint_shadows}};

pub struct StyledFrame {
    pub style: SharedStyle,
    pub align: Option<Align>,
    pub justify: Option<Align>,
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
            justify: None,
        }
    }

    /// Cross-axis (horizontal) alignment of the frame's children.
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    /// Main-axis (vertical) distribution of the frame's children. Treated as
    /// top-down inside the frame; see [`crate::StyledColumn::justify`] for details.
    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = Some(justify);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let shadow_idx = ui.painter().add(Shape::Noop);
        let corner_radius = self.style.corner_radius.unwrap_or_default();
        let shadows = self.style.shadows.clone();

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
        let justify = self.justify;

        let response = frame.show(ui, |ui| {
            if full_width {
                ui.set_min_width(ui.available_width());
            }
            if align.is_some() || justify.is_some() {
                let mut layout = Layout::top_down(align.unwrap_or(Align::Min));
                if let Some(j) = justify {
                    layout = layout.with_main_align(j);
                }
                ui.with_layout(layout, body).inner
            } else {
                body(ui)
            }
        });

        paint_shadows(ui, shadow_idx, response.response.rect, corner_radius, &shadows);

        response
    }
}

impl_style_builders!(StyledFrame);
