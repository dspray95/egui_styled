use egui::{Align, FontId, Id, Response, Stroke, TextEdit, Ui};

use crate::{impl_style_builders, state::PseudoState, style::shared_style::SharedStyle};

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
    /// conditional rendering — see [`StyledButton::id`] for the rationale.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    // Widget specific builder functions

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

    /// Cap the number of characters the field will accept.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    /// Set the font used to render the text (family + size).
    /// Overrides `font_size` from `SharedStyle` when set.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    /// Explicit width for the field. Wins over [`SharedStyle::full_width`].
    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    /// Horizontal alignment of the text within the field.
    pub fn horizontal_align(mut self, align: Align) -> Self {
        self.horizontal_align = Some(align);
        self
    }

    // show
    pub fn show(self, ui: &mut Ui) -> Response {
        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));
        let psuedo = PseudoState::load(ui, id);

        let visuals = ui.visuals().clone();
        let widget_vis = if psuedo.focused {
            &visuals.widgets.active
        } else if psuedo.hovered {
            &visuals.widgets.hovered
        } else {
            &visuals.widgets.inactive
        };

        let resolved = self.style.resolve(psuedo, widget_vis);

        let response = ui
            .scope(|ui| {
                let vis = ui.visuals_mut();

                for widget_state in [
                    &mut vis.widgets.inactive,
                    &mut vis.widgets.hovered,
                    &mut vis.widgets.active,
                ] {
                    widget_state.bg_fill = resolved.bg;
                    widget_state.bg_stroke = resolved.border;
                    widget_state.corner_radius = resolved.corner_radius;
                }

                // TextEdit uses selection.stroke/extreme_bg_color for the field bg in some themes
                vis.extreme_bg_color = resolved.bg;

                if let Some(c) = self.style.text_color {
                    for widget_state in [
                        &mut vis.widgets.inactive,
                        &mut vis.widgets.hovered,
                        &mut vis.widgets.active,
                    ] {
                        widget_state.fg_stroke = Stroke::new(1.0, c);
                    }
                }

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
                // Explicit desired_width wins over full_width.
                if let Some(width) = self.desired_width {
                    text_edit = text_edit.desired_width(width);
                } else if self.style.full_width {
                    text_edit = text_edit.desired_width(ui.available_width());
                }

                let mut wrapper = egui::Frame::new();
                if let Some(m) = self.style.margin {
                    wrapper = wrapper.outer_margin(m);
                }
                wrapper.show(ui, |ui| ui.add(text_edit)).inner
            })
            .inner;

        // Store this frame's state for next frame
        PseudoState::from_response(&response).store(ui, id);

        response
    }
}

impl_style_builders!(StyledTextEdit<'_>);
