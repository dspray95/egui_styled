use egui::{Label, Response, RichText, Ui, WidgetText};

use crate::{impl_style_builders, style::shared_style::SharedStyle};

pub struct StyledLabel {
    text: WidgetText,
    bold: bool,
    italics: bool,
    wrap: Option<bool>,
    style: SharedStyle,
}

impl StyledLabel {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            bold: false,
            italics: false,
            wrap: None,
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

    pub fn show(self, ui: &mut Ui) -> Response {
        let mut rich = match self.text {
            WidgetText::RichText(rt) => (*rt).clone(),
            other => RichText::new(other.text().to_string()),
        };
        if let Some(color) = self.style.text_color {
            rich = rich.color(color);
        }
        if let Some(size) = self.style.font_size {
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
        wrapper.show(ui, |ui| {
            if self.style.full_width {
                ui.set_min_width(ui.available_width());
            }
            ui.add(label)
        })
        .inner
    }
}

impl_style_builders!(StyledLabel);
