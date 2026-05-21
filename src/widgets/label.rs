use egui::{FontId, Label, Response, RichText, Ui, WidgetText};

use crate::{impl_style_builders, style::shared_style::SharedStyle};

/// A styled label.
///
/// Labels don't track pseudo-state — `hover_bg`, `focus_border`, and
/// `active_bg` from [`SharedStyle`] are accepted but unused. Use
/// `text_color`, `font_size`, `font`, `bold`, `italics`, and `wrap` instead.
/// The wrapper exists for API uniformity with the rest of the `Styled::*`
/// namespace, not to add new capability over [`egui::Label`].
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
        // `.font(FontId)` wins over `.font_size(f32)` when both are set —
        // it carries the full family + size, not just size.
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

        let mut wrapper = egui::Frame::new();
        if let Some(m) = self.style.margin {
            wrapper = wrapper.outer_margin(m);
        }
        wrapper
            .show(ui, |ui| {
                if self.style.full_width {
                    ui.set_min_width(ui.available_width());
                }
                ui.add(label)
            })
            .inner
    }
}

impl_style_builders!(StyledLabel);
