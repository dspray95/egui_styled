use egui::{Align, FontId, Id, Response, Shape, TextEdit, Ui};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{SharedStyle, paint_shadows},
};

pub struct StyledTextEdit<'a> {
    text: &'a mut String,
    hint: Option<String>,
    multiline: bool,
    password: bool,
    char_limit: Option<usize>,
    font: Option<FontId>,
    desired_width: Option<f32>,
    horizontal_align: Option<Align>,
    id_override: Option<Id>,
    style: SharedStyle,
}

impl<'a> StyledTextEdit<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint: None,
            multiline: false,
            password: false,
            char_limit: None,
            font: None,
            desired_width: None,
            horizontal_align: None,
            id_override: None,
            style: SharedStyle::default(),
        }
    }

    /// Override the auto-generated widget id. Pins pseudo-state across
    /// conditional rendering - see [`crate::StyledButton::id`] for the rationale.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    pub fn hint(mut self, h: impl Into<String>) -> Self {
        self.hint = Some(h.into());
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn horizontal_align(mut self, align: Align) -> Self {
        self.horizontal_align = Some(align);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));
        let pseudo = PseudoState::load(ui, id);

        let per = self.style.resolve_per_state(ui.visuals());
        let shadow_idx = ui.painter().add(Shape::Noop);

        let response = ui
            .scope(|ui| {
                SharedStyle::apply_to_visuals(&per, ui.visuals_mut());

                let mut text_edit = if self.multiline {
                    TextEdit::multiline(self.text)
                } else {
                    TextEdit::singleline(self.text)
                };
                if let Some(h) = &self.hint {
                    text_edit = text_edit.hint_text(h.as_str());
                }
                if self.password {
                    text_edit = text_edit.password(true);
                }
                if let Some(limit) = self.char_limit {
                    text_edit = text_edit.char_limit(limit);
                }
                if let Some(font) = self.font.clone() {
                    text_edit = text_edit.font(font);
                } else if let Some(size) = self.style.font_size {
                    text_edit = text_edit.font(egui::FontId::proportional(size));
                }
                if let Some(align) = self.horizontal_align {
                    text_edit = text_edit.horizontal_align(align);
                }
                if let Some(width) = self.desired_width {
                    text_edit = text_edit.desired_width(width);
                } else if self.style.full_width {
                    text_edit = text_edit.desired_width(ui.available_width());
                }

                // Pass a fully-built custom Frame so egui skips its own visuals
                // path (which re-derives stroke from selection/widget visuals and
                // expands the inner margin by `expansion - stroke.width`).
                let resolved = if pseudo.focused {
                    &per.focused
                } else if pseudo.hovered {
                    &per.hovered
                } else {
                    &per.inactive
                };
                let padding = per.padding;
                let custom_frame = egui::Frame::new()
                    .fill(resolved.bg)
                    .stroke(resolved.border)
                    .corner_radius(per.corner_radius)
                    .inner_margin(padding);
                text_edit = text_edit.frame(custom_frame).margin(padding);

                let mut wrapper = egui::Frame::new();
                if per.margin != egui::Margin::ZERO {
                    wrapper = wrapper.outer_margin(per.margin);
                }
                wrapper.show(ui, |ui| ui.add(text_edit)).inner
            })
            .inner;

        paint_shadows(
            ui,
            shadow_idx,
            response.rect,
            per.corner_radius,
            &self.style.shadows,
        );

        PseudoState::from_response(&response).store(ui, id);

        if let Some(icon) = per.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        response
    }
}

impl_style_builders!(StyledTextEdit<'_>);
