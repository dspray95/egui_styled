use egui::{FontId, Id, Image, Response, RichText, Shape, Stroke, Ui, WidgetText};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{SharedStyle, paint_shadows},
};

pub struct StyledButton {
    text: WidgetText,
    image: Option<Image<'static>>,
    font: Option<FontId>,
    id_override: Option<Id>,
    style: SharedStyle,
}

impl StyledButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            image: None,
            font: None,
            id_override: None,
            style: SharedStyle::default(),
        }
    }

    // Widget specific builder
    pub fn image(mut self, img: Image<'static>) -> Self {
        self.image = Some(img);
        self
    }

    /// Set the font (family + size) used to render the button label.
    /// Overrides [`SharedStyle::font_size`] when both are set.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    /// Override the auto-generated widget id. Use this to pin pseudo-state
    /// (hover / active / focus) across conditional rendering - without an
    /// explicit id, `ui.next_auto_id()` shifts when a sibling appears or
    /// disappears, misattributing one frame of state.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    /// Renders the button with the given style
    pub fn show(self, ui: &mut Ui) -> Response {
        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));

        let shadow_idx = ui.painter().add(Shape::Noop);

        let psuedo = PseudoState::load(ui, id);

        let visuals = ui.visuals().clone();
        let widget_vis = if psuedo.active {
            &visuals.widgets.active
        } else if psuedo.hovered {
            &visuals.widgets.hovered
        } else {
            &visuals.widgets.inactive
        };
        let resolved = self.style.resolve(psuedo, widget_vis);

        if self.style.full_width {
            ui.set_min_width(ui.available_width());
        }

        // Override the widget visuals for all states in scope
        let response = ui
            .scope(|ui| {
                let vis = ui.visuals_mut();
                // Default state
                vis.widgets.inactive.bg_fill = resolved.bg;
                vis.widgets.inactive.bg_stroke = resolved.border;
                vis.widgets.inactive.corner_radius = resolved.corner_radius;
                // Hovered
                vis.widgets.hovered.bg_fill = resolved.bg;
                vis.widgets.hovered.bg_stroke = resolved.border;
                vis.widgets.hovered.corner_radius = resolved.corner_radius;
                // Active
                vis.widgets.active.bg_fill = resolved.bg;
                vis.widgets.active.bg_stroke = resolved.border;
                vis.widgets.active.corner_radius = resolved.corner_radius;

                if let Some(color) = self.style.text_color {
                    vis.widgets.inactive.fg_stroke = Stroke::new(1.0, color);
                    vis.widgets.hovered.fg_stroke = Stroke::new(1.0, color);
                    vis.widgets.active.fg_stroke = Stroke::new(1.0, color);
                }

                // Wire `padding` through to egui's `button_padding`. egui's
                // `Button` only supports symmetric padding (a single `Vec2`),
                // so an asymmetric `Margin` collapses to `max(left, right)` /
                // `max(top, bottom)`. For true asymmetric padding (e.g. font
                // visual-correction offsets) wrap the button in a
                // `Styled::frame` with matching `bg` and the asymmetric
                // padding on the wrapper.
                if let Some(m) = self.style.padding {
                    let x = m.left.max(m.right) as f32;
                    let y = m.top.max(m.bottom) as f32;
                    ui.spacing_mut().button_padding = egui::Vec2::new(x, y);
                }

                // If a font is set, fold it into the label text via RichText.
                // `.font(FontId)` wins over `.font_size(f32)` when both are set -
                // it carries the full family + size, not just size.
                let text: WidgetText = if let Some(font) = self.font.clone() {
                    let rich = match self.text {
                        WidgetText::RichText(rt) => (*rt).clone().font(font),
                        other => RichText::new(other.text().to_string()).font(font),
                    };
                    rich.into()
                } else if let Some(size) = self.style.font_size {
                    let rich = match self.text {
                        WidgetText::RichText(rt) => (*rt).clone().size(size),
                        other => RichText::new(other.text().to_string()).size(size),
                    };
                    rich.into()
                } else {
                    self.text
                };

                // Build the actual egui widget
                let mut btn = match self.image {
                    Some(img) => egui::Button::opt_image_and_text(Some(img), Some(text)),
                    None => egui::Button::new(text),
                };
                let min_w = if self.style.full_width {
                    ui.available_width()
                } else {
                    self.style.min_width.unwrap_or(0.0)
                };
                let min_h = self.style.min_height.unwrap_or(0.0);
                if min_w > 0.0 || min_h > 0.0 {
                    btn = btn.min_size(egui::Vec2 { x: min_w, y: min_h });
                }

                let mut wrapper = egui::Frame::new();
                if let Some(m) = self.style.margin {
                    wrapper = wrapper.outer_margin(m);
                }
                wrapper.show(ui, |ui| ui.add(btn)).inner
            })
            .inner;

        paint_shadows(ui, shadow_idx, response.rect, resolved.corner_radius, &self.style.shadows);

        // Store this frame's state for next frame
        PseudoState::from_response(&response).store(ui, id);

        // Handle cursor override
        if let Some(icon) = resolved.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        response
    }
}

impl_style_builders!(StyledButton);
