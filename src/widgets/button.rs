use egui::{Color32, FontId, Id, Image, Response, RichText, Shape, Stroke, Ui, WidgetText};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{
        BackdropSlot, SharedStyle, paint_shadows, paint_widget_gradient_underlay,
        paint_widget_overlays, render_scoped,
    },
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
///
/// [`selected`](Self::selected) gives toggle and tab styling; see
/// [`show_selectable`](Self::show_selectable) for the `selectable_value` pattern.
pub struct StyledButton {
    text: WidgetText,
    image: Option<Image<'static>>,
    font: Option<FontId>,
    id_override: Option<Id>,
    selected: bool,
    selected_bg: Option<Color32>,
    selected_text_color: Option<Color32>,
    style: SharedStyle,
}

impl StyledButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            image: None,
            font: None,
            id_override: None,
            selected: false,
            selected_bg: None,
            selected_text_color: None,
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
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_override = Some(Id::unique(id));
        self
    }

    /// Mark the button selected, like egui's `selectable_label`. A selected button holds
    /// its selected colours through hover and press; the focus border still applies.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Background while selected. Defaults to `accent`, then the theme's selection fill.
    pub fn selected_bg(mut self, color: Color32) -> Self {
        self.selected_bg = Some(color);
        self
    }

    /// Text colour while selected. Defaults to `text_color`, then the theme's selection text.
    pub fn selected_text_color(mut self, color: Color32) -> Self {
        self.selected_text_color = Some(color);
        self
    }

    /// Show as one option of a mutually exclusive choice, like egui's `selectable_value`:
    /// selected when `*current == value`, and a click sets `*current = value`.
    pub fn show_selectable<V: PartialEq>(self, ui: &mut Ui, current: &mut V, value: V) -> Response {
        let mut response = self.selected(*current == value).show(ui);
        if response.clicked() && *current != value {
            *current = value;
            response.mark_changed();
        }
        response
    }

    /// Selected colours replace the base and hover/press variants, so every state
    /// resolves to them. Returns the selected (bg, text) pair.
    fn apply_selected(&mut self, visuals: &egui::Visuals) -> (Color32, Color32) {
        let s = &mut self.style;
        let bg = self
            .selected_bg
            .or(s.accent)
            .unwrap_or(visuals.selection.bg_fill);
        let text = self
            .selected_text_color
            .or(s.text_color)
            .unwrap_or(visuals.selection.stroke.color);
        s.bg = Some(bg);
        s.hover_bg = None;
        s.active_bg = None;
        s.text_color = Some(text);
        s.hover_text_color = None;
        (bg, text)
    }

    pub fn show(mut self, ui: &mut Ui) -> Response {
        let selected = self.selected.then(|| self.apply_selected(ui.visuals()));
        let visible = self.style.visible != Some(false);

        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));

        let pseudo = PseudoState::load(ui, id);
        let per = self.style.resolve_per_state_button(ui.visuals());

        let response = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);
            let backdrop = BackdropSlot::reserve(ui, &self.style);
            let gradient_idx = ui.painter().add(Shape::Noop);

            let sz = self
                .style
                .resolve_size(ui.available_width(), ui.available_height());
            sz.apply_to_ui(ui);

            let response = ui
                .scope(|ui| {
                    SharedStyle::apply_to_visuals(&per, pseudo, ui.visuals_mut());
                    // egui paints a selected button from `selection`, for every state.
                    if let Some((bg, text)) = selected {
                        let vis = ui.visuals_mut();
                        vis.selection.bg_fill = bg;
                        vis.selection.stroke =
                            Stroke::new(vis.widgets.inactive.fg_stroke.width, text);
                    }

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
                    if selected.is_some() {
                        btn = btn.selected(true);
                    }
                    let min_w = sz.definite_w.or(sz.min_w).unwrap_or(0.0);
                    let min_h = sz.definite_h.or(sz.min_h).unwrap_or(0.0);
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

            let resolved = SharedStyle::for_response(&per, &response);
            backdrop.fill(ui, response.rect, per.corner_radius);
            paint_widget_gradient_underlay(
                ui,
                gradient_idx,
                response.rect,
                per.corner_radius,
                resolved,
            );
            paint_widget_overlays(ui, response.rect, resolved);
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

        response
    }
}

impl_style_builders!(StyledButton);
crate::impl_styled_widget!(StyledButton);

#[cfg(test)]
mod tests {
    use crate::Styled;
    use egui::{Color32, Event, PointerButton, Pos2, Rect, Shape, vec2};

    const SEL: Color32 = Color32::from_rgb(200, 10, 10);
    const HOVER: Color32 = Color32::from_rgb(10, 200, 10);
    const SEL_TEXT: Color32 = Color32::from_rgb(1, 2, 3);

    fn raw(events: Vec<Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 200.0))),
            events,
            ..Default::default()
        }
    }

    fn rect_fills(shapes: &[Shape], out: &mut Vec<Color32>) {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => rect_fills(inner, out),
                Shape::Rect(r) => out.push(r.fill),
                _ => {}
            }
        }
    }

    fn text_colors(shapes: &[Shape]) -> Vec<Color32> {
        shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.fallback_color),
                _ => None,
            })
            .collect()
    }

    fn frame(
        ctx: &egui::Context,
        events: Vec<Event>,
        selected: bool,
    ) -> (Rect, Vec<Color32>, Vec<Color32>) {
        let mut rect = Rect::NOTHING;
        let out = crate::test_util::run_ui(ctx, raw(events), |ui| {
            rect = Styled::button("Tab")
                .hover_bg(HOVER)
                .selected(selected)
                .selected_bg(SEL)
                .selected_text_color(SEL_TEXT)
                .show(ui)
                .rect;
        });
        let shapes: Vec<Shape> = out.shapes.into_iter().map(|c| c.shape).collect();
        let mut fills = Vec::new();
        rect_fills(&shapes, &mut fills);
        (rect, fills, text_colors(&shapes))
    }

    #[test]
    fn selected_paints_selected_colors() {
        let ctx = egui::Context::default();
        let (_, fills, texts) = frame(&ctx, vec![], true);
        assert!(fills.contains(&SEL), "{fills:?}");
        assert!(texts.contains(&SEL_TEXT), "{texts:?}");

        let ctx = egui::Context::default();
        let (_, fills, texts) = frame(&ctx, vec![], false);
        assert!(!fills.contains(&SEL), "{fills:?}");
        assert!(!texts.contains(&SEL_TEXT), "{texts:?}");
    }

    #[test]
    fn selected_holds_through_hover() {
        let ctx = egui::Context::default();
        let (rect, _, _) = frame(&ctx, vec![], true);
        let over = vec![Event::PointerMoved(rect.center())];
        frame(&ctx, over.clone(), true);
        let (_, fills, _) = frame(&ctx, over, true);
        assert!(fills.contains(&SEL), "{fills:?}");
        assert!(!fills.contains(&HOVER), "{fills:?}");
    }

    #[test]
    fn show_selectable_sets_value_on_click() {
        #[derive(PartialEq, Debug, Clone, Copy)]
        enum Tab {
            A,
            B,
        }
        let ctx = egui::Context::default();
        let mut current = Tab::A;
        let mut changed = false;
        let mut run = |current: &mut Tab, events| {
            let mut b_rect = Rect::NOTHING;
            let _ = crate::test_util::run_ui(&ctx, raw(events), |ui| {
                ui.horizontal(|ui| {
                    Styled::button("A").show_selectable(ui, current, Tab::A);
                    let r = Styled::button("B").show_selectable(ui, current, Tab::B);
                    changed |= r.changed();
                    b_rect = r.rect;
                });
            });
            b_rect
        };
        let b = run(&mut current, vec![]).center();
        let press = |pressed| Event::PointerButton {
            pos: b,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        run(&mut current, vec![Event::PointerMoved(b), press(true)]);
        run(&mut current, vec![press(false)]);
        assert_eq!(current, Tab::B);
        assert!(changed);
    }

    #[test]
    fn unset_bg_paints_weak_bg_fill_like_egui() {
        const WEAK: Color32 = Color32::from_rgb(7, 8, 9);
        const STRONG: Color32 = Color32::from_rgb(90, 90, 90);
        let ctx = egui::Context::default();
        let out = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| {
            let w = &mut ui.visuals_mut().widgets.inactive;
            w.weak_bg_fill = WEAK;
            w.bg_fill = STRONG;
            Styled::button("x").show(ui);
        });
        let shapes: Vec<Shape> = out.shapes.into_iter().map(|c| c.shape).collect();
        let mut fills = Vec::new();
        rect_fills(&shapes, &mut fills);
        assert!(
            fills.contains(&WEAK) && !fills.contains(&STRONG),
            "{fills:?}"
        );
    }
}
