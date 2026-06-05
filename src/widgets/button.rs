use egui::{FontId, Id, Image, Response, RichText, Shape, Ui, WidgetText};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{SharedStyle, paint_shadows, render_scoped},
};

/// A clickable button with per-state (hover / focus / active) styling.
///
/// Wraps egui's button with the [`SharedStyle`] box builders (`bg`, `border`,
/// `corner_radius`, `padding`, …) plus per-state color variants (`hover_bg`,
/// `active_bg`, `focus_bg`, …) that resolve through egui's active `Visuals`
/// for anything left unset. An optional leading [`image`](Self::image) and an
/// explicit [`font`](Self::font) round out the label.
///
/// Construct via [`Styled::button`](crate::Styled::button) and call
/// [`show`](Self::show) to add it to a `Ui`. Set [`id`](Self::id) when the
/// button is conditionally rendered so its pseudo-state stays stable.
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

    pub fn show(self, ui: &mut Ui) -> Response {
        let visible = self.style.visible != Some(false);

        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));

        let pseudo = PseudoState::load(ui, id);
        let per = self.style.resolve_per_state(ui.visuals());

        let response = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);

            if self.style.full_width {
                ui.set_min_width(ui.available_width());
            }

            let response = ui
                .scope(|ui| {
                    SharedStyle::apply_to_visuals(&per, ui.visuals_mut());

                    // Wire `padding` through to egui's `button_padding`. egui's
                    // `Button` only supports symmetric padding (a single `Vec2`),
                    // so an asymmetric `Margin` collapses to `max(left, right)` /
                    // `max(top, bottom)`. For true asymmetric padding wrap the
                    // button in a `Styled::frame`.
                    if per.padding != egui::Margin::ZERO {
                        let m = per.padding;
                        let x = m.left.max(m.right) as f32;
                        let y = m.top.max(m.bottom) as f32;
                        ui.spacing_mut().button_padding = egui::Vec2::new(x, y);
                    }

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
                    if per.margin != egui::Margin::ZERO {
                        wrapper = wrapper.outer_margin(per.margin);
                    }
                    wrapper.show(ui, |ui| ui.add(btn)).inner
                })
                .inner;

            SharedStyle::paint_widget_side_borders(ui, &response, &per);
            paint_shadows(
                ui,
                shadow_idx,
                response.rect,
                per.corner_radius,
                &self.style.shadows,
            );
            response
        });

        PseudoState::from_response(&response).store(ui, id);

        if let Some(icon) = per.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        // Suppress unused-variable warning for pseudo (used for state loading only).
        let _ = pseudo;

        response
    }
}

impl_style_builders!(StyledButton);
