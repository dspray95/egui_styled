use egui::{FontId, Label, Response, RichText, Shape, Stroke, Ui, WidgetText};

use crate::{
    impl_style_builders,
    style::shared_style::{SharedStyle, paint_shadows},
};

/// A styled label.
///
/// Labels are not interactive so pseudo-state fields (`hover_bg`, `active_bg`,
/// `focus_bg`, `hover_border`, `focus_border`, `hover_text_color`) are
/// accepted by the builder but have no effect. Also unsupported: `max_width/height`.
/// Use `text_color`, `bg`, `border`, `corner_radius`, `padding`, `margin`,
/// `font_size`, `font`, `bold`, `italics`, `wrap`, `min_height`, `visible`,
/// `shadows`, and `cursor` instead.
pub struct StyledLabel {
    text: WidgetText,
    bold: bool,
    italics: bool,
    wrap: Option<bool>,
    font: Option<FontId>,
    style: SharedStyle,
}

impl StyledLabel {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            bold: false,
            italics: false,
            wrap: None,
            font: None,
            style: SharedStyle::default(),
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italics(mut self) -> Self {
        self.italics = true;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Set the font (family + size) used to render the label.
    /// Overrides [`SharedStyle::font_size`] when both are set.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let mut rich = match self.text {
            WidgetText::RichText(rt) => (*rt).clone(),
            other => RichText::new(other.text().to_string()),
        };
        if let Some(color) = self.style.text_color {
            rich = rich.color(color);
        }
        if let Some(font) = self.font.clone() {
            rich = rich.font(font);
        } else if let Some(size) = self.style.font_size {
            rich = rich.size(size);
        }
        if self.bold {
            rich = rich.strong();
        }
        if self.italics {
            rich = rich.italics();
        }

        let mut label = Label::new(rich);
        if let Some(wrap) = self.wrap {
            label = if wrap { label.wrap() } else { label.truncate() };
        }

        // Resolve static style (labels have no interaction state).
        let visuals = ui.visuals().clone();
        let wv = &visuals.widgets.inactive;
        let bg = self.style.bg.unwrap_or(egui::Color32::TRANSPARENT);
        let border = self.style.border.unwrap_or(Stroke::NONE);
        let corner_radius = self.style.corner_radius.unwrap_or(wv.corner_radius);
        let padding = self.style.padding.unwrap_or_default();
        let margin = self.style.margin.unwrap_or_default();

        let shadow_idx = ui.painter().add(Shape::Noop);

        let response = egui::Frame::new()
            .fill(bg)
            .stroke(border)
            .corner_radius(corner_radius)
            .inner_margin(padding)
            .outer_margin(margin)
            .show(ui, |ui| {
                if self.style.full_width {
                    ui.set_min_width(ui.available_width());
                }
                if let Some(min_w) = self.style.min_width {
                    ui.set_min_width(min_w);
                }
                if let Some(min_h) = self.style.min_height {
                    ui.set_min_height(min_h);
                }
                if self.style.visible == Some(false) {
                    ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 0.0),
                        egui::Sense::hover(),
                    )
                    .1
                } else {
                    ui.add(label)
                }
            })
            .inner;

        paint_shadows(
            ui,
            shadow_idx,
            response.rect,
            corner_radius,
            &self.style.shadows,
        );

        if let Some(icon) = self.style.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        response
    }
}

impl_style_builders!(StyledLabel);
