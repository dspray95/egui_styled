use std::cell::RefCell;

use egui::{
    Align2, Atom, AtomExt as _, Color32, ComboBox, CornerRadius, FontId, Id, IdSalt, InnerResponse,
    NumExt as _, Popup, PopupCloseBehavior, Rect, Response, RichText, Role, ScrollArea, Sense,
    Shape, Stroke, Style, TextStyle, TextWrapMode, Ui, Vec2, WidgetAtom, WidgetInfo, WidgetText,
    class::{Classes, HasClasses as _},
    containers::menu::menu_style,
    style::WidgetVisuals,
    vec2,
    widget_style::ButtonStyle,
};

use crate::{
    StyledButton, StyledFrame, impl_style_builders,
    state::PseudoState,
    style::shared_style::{
        BackdropSlot, SharedStyle, paint_shadows, paint_widget_gradient_underlay,
        paint_widget_overlays, render_scoped,
    },
};

/// Paints the trigger's dropdown icon: `(ui, rect, visuals, is_open)`, like egui's `ComboBox::icon`.
pub type IconPainter = Box<dyn FnOnce(&Ui, Rect, &WidgetVisuals, bool)>;

/// A styled combo box (dropdown). Shared builders style the trigger; [`popup`](Self::popup)
/// and the `item_*` / `selected_*` builders style the dropdown. Unset parts keep egui's look.
pub struct StyledComboBox {
    id_salt: IdSalt,
    selected_text: WidgetText,
    width: Option<f32>,
    font: Option<FontId>,
    id_override: Option<Id>,
    parts: ComboParts,
    style: SharedStyle,
}

#[derive(Default)]
struct ComboParts {
    max_height: Option<f32>,
    wrap_mode: Option<TextWrapMode>,
    close_behavior: Option<PopupCloseBehavior>,
    icon: Option<IconPainter>,
    icon_color: Option<Color32>,
    open_bg: Option<Color32>,
    open_border: Option<Stroke>,
    popup: Option<StyledFrame>,
    items: ItemStyle,
}

#[derive(Clone, Copy, Default)]
struct ItemStyle {
    text_color: Option<Color32>,
    hover_text_color: Option<Color32>,
    hover_bg: Option<Color32>,
    selected_bg: Option<Color32>,
    selected_text_color: Option<Color32>,
    padding: Option<Vec2>,
    gap: Option<f32>,
    corner_radius: Option<CornerRadius>,
}

impl ItemStyle {
    fn apply(self, style: &mut Style) {
        menu_style(style);
        if let Some(p) = self.padding {
            style.spacing.button_padding = p;
        }
        if let Some(g) = self.gap {
            style.spacing.item_spacing.y = g;
        }
        let vis = &mut style.visuals;
        let w = &mut vis.widgets;
        if let Some(c) = self.text_color {
            for v in [
                &mut w.noninteractive,
                &mut w.inactive,
                &mut w.hovered,
                &mut w.active,
            ] {
                v.fg_stroke.color = c;
            }
        }
        if let Some(c) = self.hover_text_color {
            for v in [&mut w.hovered, &mut w.active] {
                v.fg_stroke.color = c;
            }
        }
        if let Some(bg) = self.hover_bg {
            for v in [&mut w.hovered, &mut w.active] {
                v.bg_fill = bg;
                v.weak_bg_fill = bg;
            }
        }
        if let Some(r) = self.corner_radius {
            for v in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                v.corner_radius = r;
            }
        }
        if let Some(bg) = self.selected_bg {
            vis.selection.bg_fill = bg;
        }
        if let Some(c) = self.selected_text_color.or(self.text_color) {
            vis.selection.stroke.color = c;
        }
    }
}

impl StyledComboBox {
    pub fn new(
        id_source: impl std::hash::Hash + std::fmt::Debug,
        selected_text: impl Into<WidgetText>,
    ) -> Self {
        Self {
            id_salt: IdSalt::new(id_source),
            selected_text: selected_text.into(),
            width: None,
            font: None,
            id_override: None,
            parts: ComboParts::default(),
            style: SharedStyle::default(),
        }
    }

    /// Minimum trigger width. Defaults to egui's `combo_width`. The popup is at least as wide.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// Font (family and size) of the selected text. Overrides `font_size`.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    /// Override the auto id. Pins pseudo-state and popup state; see [`crate::StyledButton::id`].
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_override = Some(Id::unique(id));
        self
    }

    /// Popup height before it scrolls. Defaults to egui's `combo_height`.
    pub fn popup_height(mut self, height: f32) -> Self {
        self.parts.max_height = Some(height);
        self
    }

    /// Wrap the selected text when the trigger is too narrow.
    pub fn wrap(mut self) -> Self {
        self.parts.wrap_mode = Some(TextWrapMode::Wrap);
        self
    }

    /// Truncate the selected text with an ellipsis when the trigger is too narrow.
    pub fn truncate(mut self) -> Self {
        self.parts.wrap_mode = Some(TextWrapMode::Truncate);
        self
    }

    /// When the popup closes. Defaults to [`PopupCloseBehavior::CloseOnClick`].
    pub fn close_behavior(mut self, behavior: PopupCloseBehavior) -> Self {
        self.parts.close_behavior = Some(behavior);
        self
    }

    /// Custom dropdown icon painter, replacing the default triangle.
    pub fn icon(mut self, icon: impl FnOnce(&Ui, Rect, &WidgetVisuals, bool) + 'static) -> Self {
        self.parts.icon = Some(Box::new(icon));
        self
    }

    /// Colour of the dropdown icon. Defaults to the trigger's text colour.
    pub fn icon_color(mut self, color: Color32) -> Self {
        self.parts.icon_color = Some(color);
        self
    }

    /// Trigger background while the popup is open. Defaults to `bg`.
    pub fn open_bg(mut self, color: Color32) -> Self {
        self.parts.open_bg = Some(color);
        self
    }

    /// Trigger border while the popup is open. Defaults to `border`.
    pub fn open_border(mut self, width: f32, color: Color32) -> Self {
        self.parts.open_border = Some(Stroke::new(width, color));
        self
    }

    /// Style the dropdown panel. Unset `bg`, `border`, `corner_radius`, `padding` and shadows
    /// fall back to the theme's popup look.
    pub fn popup(mut self, frame: StyledFrame) -> Self {
        self.parts.popup = Some(frame);
        self
    }

    /// Text colour of popup items.
    pub fn item_text_color(mut self, color: Color32) -> Self {
        self.parts.items.text_color = Some(color);
        self
    }

    /// Text colour of a hovered or pressed popup item. Defaults to `item_text_color`.
    pub fn item_hover_text_color(mut self, color: Color32) -> Self {
        self.parts.items.hover_text_color = Some(color);
        self
    }

    /// Background of a hovered or pressed popup item.
    pub fn item_hover_bg(mut self, color: Color32) -> Self {
        self.parts.items.hover_bg = Some(color);
        self
    }

    /// Background of the selected popup item. Defaults to the theme's selection fill.
    pub fn selected_bg(mut self, color: Color32) -> Self {
        self.parts.items.selected_bg = Some(color);
        self
    }

    /// Text colour of the selected popup item. Defaults to `item_text_color`.
    pub fn selected_text_color(mut self, color: Color32) -> Self {
        self.parts.items.selected_text_color = Some(color);
        self
    }

    /// Padding inside each popup item. egui's menu default is `(2, 0)`.
    pub fn item_padding(mut self, x: f32, y: f32) -> Self {
        self.parts.items.padding = Some(vec2(x, y));
        self
    }

    /// Vertical space between popup items. Defaults to egui's `item_spacing.y`.
    pub fn item_gap(mut self, gap: f32) -> Self {
        self.parts.items.gap = Some(gap);
        self
    }

    /// Corner radius of item highlights.
    pub fn item_corner_radius(mut self, radius: impl Into<CornerRadius>) -> Self {
        self.parts.items.corner_radius = Some(radius.into());
        self
    }

    /// Whether this combo's popup is open. `id` is the [`id`](Self::id) override, else
    /// `ui.make_persistent_id(id_source)`.
    pub fn is_open(ctx: &egui::Context, id: Id) -> bool {
        Popup::is_id_open(ctx, popup_id(id))
    }

    /// Like `egui::ComboBox::show_index`. The trigger shows `get(*selected)`.
    pub fn show_index<T: Into<WidgetText>>(
        mut self,
        ui: &mut Ui,
        selected: &mut usize,
        len: usize,
        get: impl Fn(usize) -> T,
    ) -> Response {
        self.selected_text = get(*selected).into();
        let mut changed = false;
        let mut response = self
            .show(ui, |ui| {
                for i in 0..len {
                    if ui.selectable_label(i == *selected, get(i)).clicked() {
                        *selected = i;
                        changed = true;
                    }
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }

    /// A list of `selectable_value`s. The trigger shows the label matching `*current`,
    /// else the text given to `new`.
    pub fn show_values<V: PartialEq, T: Into<WidgetText>>(
        mut self,
        ui: &mut Ui,
        current: &mut V,
        options: impl IntoIterator<Item = (V, T)>,
    ) -> Response {
        let mut options: Vec<(V, WidgetText)> =
            options.into_iter().map(|(v, t)| (v, t.into())).collect();
        if let Some((_, text)) = options.iter().find(|(v, _)| v == current) {
            self.selected_text = text.clone();
        }
        let mut changed = false;
        let mut response = self
            .show(ui, |ui| {
                for (value, text) in options.drain(..) {
                    if ui.selectable_label(*current == value, text).clicked() && *current != value {
                        *current = value;
                        changed = true;
                    }
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }

    /// [`show_values`](Self::show_values) with each item a full-width [`StyledButton`] that
    /// `style` can restyle. Unset item properties inherit the `item_*` / `selected_*` style.
    pub fn show_values_with<V: PartialEq, T: Into<WidgetText>>(
        mut self,
        ui: &mut Ui,
        current: &mut V,
        options: impl IntoIterator<Item = (V, T)>,
        style: impl Fn(&V, StyledButton) -> StyledButton,
    ) -> Response {
        let mut options: Vec<(V, WidgetText)> =
            options.into_iter().map(|(v, t)| (v, t.into())).collect();
        if let Some((_, text)) = options.iter().find(|(v, _)| v == current) {
            self.selected_text = text.clone();
        }
        let mut changed = false;
        let mut response = self
            .show(ui, |ui| {
                for (value, text) in options.drain(..) {
                    // egui buttons paint `weak_bg_fill`; `StyledButton` falls back to `bg_fill`.
                    let w = &ui.visuals().widgets;
                    let item = StyledButton::new(text)
                        .full_width()
                        .bg(w.inactive.weak_bg_fill)
                        .hover_bg(w.hovered.weak_bg_fill)
                        .active_bg(w.active.weak_bg_fill);
                    let item = style(&value, item);
                    changed |= item.show_selectable(ui, current, value).changed();
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }

    /// The body runs in the popup, whose visuals carry the `item_*` / `selected_*` style.
    ///
    /// Not a [`crate::StyledContainer`]: the body only runs while open, as in egui's `show_ui`.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        menu_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        let visible = self.style.visible != Some(false);

        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(self.id_salt));
        let pseudo = PseudoState::load(ui, id);
        let per = self.style.resolve_per_state(ui.visuals());

        let selected_text: WidgetText = match (self.font, self.style.font_size) {
            (Some(font), _) => rich(self.selected_text).font(font).into(),
            (None, Some(size)) => rich(self.selected_text).size(size).into(),
            (None, None) => self.selected_text,
        };
        let parts = self.parts;
        let width = self.width;

        let result = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);
            let backdrop = BackdropSlot::reserve(ui, &self.style);
            let gradient_idx = ui.painter().add(Shape::Noop);
            let result = ui
                .scope(|ui| {
                    SharedStyle::apply_to_visuals(&per, pseudo, ui.visuals_mut());
                    let open = &mut ui.visuals_mut().widgets.open;
                    if let Some(bg) = parts.open_bg {
                        open.bg_fill = bg;
                        open.weak_bg_fill = bg;
                    }
                    if let Some(b) = parts.open_border {
                        open.bg_stroke = b;
                    }
                    // Same symmetric collapse as `StyledButton`.
                    if per.padding != egui::Margin::ZERO {
                        let m = per.padding;
                        ui.spacing_mut().button_padding =
                            vec2(m.left.max(m.right) as f32, m.top.max(m.bottom) as f32);
                    }

                    let sz = self
                        .style
                        .resolve_size(ui.available_width(), ui.available_height());
                    if let Some(w) = sz.max_w {
                        ui.set_max_width(w);
                    }
                    let min_size = vec2(
                        width
                            .or(sz.definite_w)
                            .or(sz.min_w)
                            .unwrap_or(ui.spacing().combo_width),
                        sz.definite_h.or(sz.min_h).unwrap_or(0.0),
                    );

                    let mut wrapper = egui::Frame::new();
                    if per.margin != egui::Margin::ZERO {
                        wrapper = wrapper.outer_margin(per.margin);
                    }
                    wrapper
                        .show(ui, |ui| {
                            combo(ui, id, selected_text, min_size, parts, menu_contents)
                        })
                        .inner
                })
                .inner;

            let resolved = SharedStyle::for_response(&per, &result.response);
            backdrop.fill(ui, result.response.rect, per.corner_radius);
            paint_widget_gradient_underlay(
                ui,
                gradient_idx,
                result.response.rect,
                per.corner_radius,
                resolved,
            );
            paint_widget_overlays(ui, result.response.rect, resolved);
            paint_shadows(
                ui,
                shadow_idx,
                result.response.rect,
                per.corner_radius,
                &self.style.shadows,
            );
            result
        });

        let response = &result.response;
        PseudoState::from_response(response).store(ui, id);

        if let Some(icon) = per.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        result
    }
}

fn rich(text: WidgetText) -> RichText {
    match text {
        WidgetText::RichText(rt) => (*rt).clone(),
        other => RichText::new(other.text().to_string()),
    }
}

fn popup_id(id: Id) -> Id {
    // Matches egui's `ComboBox`, so popup state carries over.
    id.with("popup")
}

/// egui's `combo_box_dyn`, with our own popup frame so the panel can be a `StyledFrame`.
fn combo<R>(
    ui: &mut Ui,
    id: Id,
    selected_text: WidgetText,
    min_size: Vec2,
    parts: ComboParts,
    menu_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<Option<R>> {
    let popup_id = popup_id(id);
    let is_open = Popup::is_id_open(ui.ctx(), popup_id);

    let mut classes = Classes::default();
    classes.add_class_if(ComboBox::CLASS_OPEN, is_open);
    let ButtonStyle { atom_layout } = ui.widget_style::<ButtonStyle>(id, &classes);
    let min_size = min_size.at_least(atom_layout.min_size);

    // Previous-pass state, like `ui.widget_style`.
    let mut visuals = if is_open {
        ui.visuals().widgets.open
    } else if let Some(r) = ui.ctx().read_response(id) {
        *ui.style().interact(&r)
    } else {
        ui.visuals().widgets.inactive
    };
    if let Some(c) = parts.icon_color {
        visuals.fg_stroke.color = c;
    }

    let icon = RefCell::new(parts.icon);
    let icon_atom = Atom::paint(Vec2::splat(ui.spacing().icon_width), move |ui, args| {
        let rect = args.rect.expand(visuals.expansion);
        match icon.borrow_mut().take() {
            Some(paint) => paint(ui, rect, &visuals, is_open),
            None => paint_default_icon(ui, rect, &visuals),
        }
    })
    .atom_grow(true)
    .atom_align(Align2::RIGHT_CENTER);

    let accessible_text = selected_text.text().to_owned();
    let wrap_mode = parts.wrap_mode.unwrap_or_else(|| ui.wrap_mode());
    let response = atom_layout
        .apply(
            WidgetAtom::new((selected_text, icon_atom))
                .id(id)
                .sense(Sense::click())
                .fallback_font(TextStyle::Button)
                .wrap_mode(wrap_mode),
        )
        .min_size(min_size)
        .show(ui)
        .response;
    response.widget_info(|| {
        let mut info = WidgetInfo::new(Role::ComboBox);
        info.enabled = ui.is_enabled();
        info.current_text_value = Some(accessible_text.clone());
        info
    });

    let height = parts
        .max_height
        .unwrap_or_else(|| ui.spacing().combo_height);
    let items = parts.items;
    let panel = parts.popup;
    let mut popup = Popup::menu(&response)
        .id(popup_id)
        .width(response.rect.width())
        .close_behavior(parts.close_behavior.unwrap_or_default())
        .style(move |s: &mut Style| items.apply(s));
    if panel.is_some() {
        popup = popup.frame(egui::Frame::NONE);
    }

    let inner = popup
        .show(|ui| {
            let body = |ui: &mut Ui| {
                ui.set_min_width(ui.available_width());
                ScrollArea::vertical()
                    .max_height(height)
                    .show(ui, |ui| {
                        // A narrow trigger would otherwise wrap labels early.
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        menu_contents(ui)
                    })
                    .inner
            };
            match panel {
                Some(frame) => show_panel(ui, frame, body),
                None => body(ui),
            }
        })
        .map(|r| r.inner);

    if inner.is_some() {
        ui.ctx().accesskit_node_builder(popup_id, |node| {
            node.set_role(egui::accesskit::Role::ListBox);
        });
    }

    InnerResponse { inner, response }
}

/// Panel as a `StyledFrame`; unset parts from the theme's popup frame.
fn show_panel<R>(ui: &mut Ui, mut frame: StyledFrame, body: impl FnOnce(&mut Ui) -> R) -> R {
    let st = ui.style().clone();
    let s = &mut frame.style;
    s.bg.get_or_insert(st.visuals.window_fill);
    s.border.get_or_insert(st.visuals.window_stroke);
    s.padding.get_or_insert(st.spacing.menu_margin);
    let radius = *s.corner_radius.get_or_insert(st.visuals.menu_corner_radius);
    let egui_shadow = s.shadows.is_empty().then_some(st.visuals.popup_shadow);

    let shadow_idx = ui.painter().add(Shape::Noop);
    let resp = frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        body(ui)
    });
    if let Some(shadow) = egui_shadow {
        ui.painter()
            .set(shadow_idx, shadow.as_shape(resp.response.rect, radius));
    }
    resp.inner
}

fn paint_default_icon(ui: &Ui, rect: Rect, visuals: &WidgetVisuals) {
    let rect = Rect::from_center_size(
        rect.center(),
        vec2(rect.width() * 0.7, rect.height() * 0.45),
    );
    ui.painter()
        .add(Shape::rotated_triangle(rect, 0.0, visuals.fg_stroke.color));
}

impl_style_builders!(StyledComboBox);

#[cfg(test)]
mod tests {
    use crate::Styled;
    use egui::{Color32, Event, PointerButton, Pos2, Rect, Shape, vec2};

    const PANEL: Color32 = Color32::from_rgb(10, 20, 30);
    const SELECTED: Color32 = Color32::from_rgb(200, 0, 0);
    const OPEN: Color32 = Color32::from_rgb(0, 200, 0);

    fn raw(events: Vec<Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 400.0))),
            events,
            ..Default::default()
        }
    }

    fn click(pos: Pos2) -> [Vec<Event>; 2] {
        let press = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        [
            vec![Event::PointerMoved(pos), press(true)],
            vec![press(false)],
        ]
    }

    fn fills(shapes: impl IntoIterator<Item = Shape>, out: &mut Vec<Color32>) {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => fills(inner, out),
                Shape::Rect(r) => out.push(r.fill),
                _ => {}
            }
        }
    }

    /// No popup fade-in, so painted colours are exact.
    fn still_ctx() -> egui::Context {
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|s| s.animation_time = 0.0);
        ctx
    }

    /// Runs `frames` passes, clicking the trigger on the first; returns the last pass's fills.
    fn open_and_paint(mut ui_fn: impl FnMut(&mut egui::Ui) -> Rect, frames: usize) -> Vec<Color32> {
        let ctx = still_ctx();
        let mut rect = Rect::NOTHING;
        let _ = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| rect = ui_fn(ui));
        let [down, up] = click(rect.center());
        let _ = crate::test_util::run_ui(&ctx, raw(down), |ui| rect = ui_fn(ui));
        let mut out = crate::test_util::run_ui(&ctx, raw(up), |ui| rect = ui_fn(ui));
        for _ in 0..frames {
            out = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| rect = ui_fn(ui));
        }
        let mut colors = Vec::new();
        fills(out.shapes.into_iter().map(|c| c.shape), &mut colors);
        colors
    }

    #[test]
    fn popup_panel_and_items_are_styled() {
        let mut v = 1usize;
        let colors = open_and_paint(
            |ui| {
                Styled::combo_box("c", "")
                    .popup(Styled::frame().bg(PANEL))
                    .selected_bg(SELECTED)
                    .open_bg(OPEN)
                    .show_index(ui, &mut v, 3, |i| format!("item {i}"))
                    .rect
            },
            2,
        );
        for c in [PANEL, SELECTED, OPEN] {
            assert!(colors.contains(&c), "{c:?} not painted in {colors:?}");
        }
    }

    #[test]
    fn closed_popup_paints_nothing() {
        let ctx = egui::Context::default();
        let out = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| {
            Styled::combo_box("c", "x")
                .popup(Styled::frame().bg(PANEL))
                .show(ui, |_| ());
        });
        let mut colors = Vec::new();
        fills(out.shapes.into_iter().map(|c| c.shape), &mut colors);
        assert!(!colors.contains(&PANEL));
    }

    #[test]
    fn padding_grows_the_trigger() {
        let ctx = egui::Context::default();
        let (mut plain, mut padded) = (Rect::NOTHING, Rect::NOTHING);
        let _ = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| {
            plain = Styled::combo_box("a", "x").show(ui, |_| ()).response.rect;
            padded = Styled::combo_box("b", "x")
                .padding(12.0)
                .show(ui, |_| ())
                .response
                .rect;
        });
        assert!(
            padded.height() > plain.height() + 10.0,
            "{plain:?} vs {padded:?}"
        );
    }

    #[test]
    fn show_index_selects_on_click() {
        let ctx = still_ctx();
        let mut v = 0usize;
        let rect = std::cell::Cell::new(Rect::NOTHING);
        let run = |v: &mut usize, events| {
            crate::test_util::run_ui(&ctx, raw(events), |ui| {
                rect.set(
                    Styled::combo_box("c", "")
                        .width(100.0)
                        .show_index(ui, v, 3, |i| format!("item {i}"))
                        .rect,
                );
            })
        };
        run(&mut v, vec![]);
        let trigger = rect.get();
        for e in click(trigger.center()) {
            run(&mut v, e);
        }
        let item = run(&mut v, vec![])
            .shapes
            .into_iter()
            .find_map(|c| match c.shape {
                Shape::Text(t) if t.galley.text() == "item 2" => {
                    Some(t.visual_bounding_rect().center())
                }
                _ => None,
            })
            .expect("popup open");
        for e in click(item) {
            run(&mut v, e);
        }
        assert_eq!(v, 2);
    }

    #[test]
    fn id_override_keeps_the_popup_open_across_reordering() {
        let ctx = egui::Context::default();
        let mut rect = Rect::NOTHING;
        let mut run = |sibling: bool, events| {
            let _ = crate::test_util::run_ui(&ctx, raw(events), |ui| {
                if sibling {
                    ui.push_id("extra", |ui| ui.label("x"));
                }
                rect = Styled::combo_box("c", "x")
                    .id("pinned")
                    .show(ui, |_| ())
                    .response
                    .rect;
            });
            rect
        };
        let r = run(false, vec![]);
        for e in click(r.center()) {
            run(false, e);
        }
        run(true, vec![]);
        assert!(super::StyledComboBox::is_open(
            &ctx,
            egui::Id::unique("pinned")
        ));
    }

    const ITEM_TEXT: Color32 = Color32::from_rgb(1, 2, 3);
    const HOVER_TEXT: Color32 = Color32::from_rgb(4, 5, 6);
    const RED: Color32 = Color32::from_rgb(250, 0, 0);

    /// Text shapes as (text, rect, colour).
    fn texts(out: &egui::FullOutput) -> Vec<(String, Rect, Color32)> {
        out.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                Shape::Text(t) => {
                    let fmt = t.galley.job.sections.first()?.format.color;
                    let color = t
                        .override_text_color
                        .unwrap_or(if fmt == Color32::PLACEHOLDER {
                            t.fallback_color
                        } else {
                            fmt
                        });
                    Some((t.galley.text().to_owned(), t.visual_bounding_rect(), color))
                }
                _ => None,
            })
            .collect()
    }

    fn text<'a>(texts: &'a [(String, Rect, Color32)], s: &str) -> &'a (String, Rect, Color32) {
        texts
            .iter()
            .find(|t| t.0 == s)
            .unwrap_or_else(|| panic!("{s:?} not in {texts:?}"))
    }

    /// Clicks the trigger open; returns the pass showing the popup. `ui_fn` returns the trigger rect.
    fn open(ctx: &egui::Context, ui_fn: &mut dyn FnMut(&mut egui::Ui) -> Rect) -> egui::FullOutput {
        let mut rect = Rect::NOTHING;
        let _ = crate::test_util::run_ui(ctx, raw(vec![]), |ui| rect = ui_fn(ui));
        for e in click(rect.center()) {
            let _ = crate::test_util::run_ui(ctx, raw(e), |ui| {
                ui_fn(ui);
            });
        }
        crate::test_util::run_ui(ctx, raw(vec![]), |ui| {
            ui_fn(ui);
        })
    }

    fn pass(
        ctx: &egui::Context,
        events: Vec<Event>,
        ui_fn: &mut dyn FnMut(&mut egui::Ui) -> Rect,
    ) -> egui::FullOutput {
        crate::test_util::run_ui(ctx, raw(events), |ui| {
            ui_fn(ui);
        })
    }

    #[test]
    fn hovered_item_uses_hover_text_color() {
        let ctx = still_ctx();
        let mut v = 9; // Not an option, so the trigger shows no item label.
        let mut ui_fn = |ui: &mut egui::Ui| {
            Styled::combo_box("c", "")
                .item_text_color(ITEM_TEXT)
                .item_hover_text_color(HOVER_TEXT)
                .show_values(ui, &mut v, [(0, "a"), (1, "b")])
                .rect
        };
        let shown = texts(&open(&ctx, &mut ui_fn));
        let b = text(&shown, "b").1.center();
        pass(&ctx, vec![Event::PointerMoved(b)], &mut ui_fn);
        let shown = texts(&pass(&ctx, vec![], &mut ui_fn));
        assert_eq!(text(&shown, "b").2, HOVER_TEXT);
        assert_eq!(text(&shown, "a").2, ITEM_TEXT);
    }

    #[test]
    fn show_values_with_styles_one_item() {
        let ctx = still_ctx();
        let mut v = 0;
        let mut ui_fn = |ui: &mut egui::Ui| {
            Styled::combo_box("c", "")
                .item_text_color(ITEM_TEXT)
                .show_values_with(ui, &mut v, [(0, "a"), (1, "b"), (2, "c")], |v, item| {
                    if *v == 2 { item.text_color(RED) } else { item }
                })
                .rect
        };
        let shown = texts(&open(&ctx, &mut ui_fn));
        assert_eq!(text(&shown, "b").2, ITEM_TEXT);
        assert_eq!(text(&shown, "c").2, RED);
    }

    #[test]
    fn show_values_with_items_fill_the_popup_and_left_align() {
        let ctx = still_ctx();
        let mut v = 1;
        let mut ui_fn = |ui: &mut egui::Ui| {
            Styled::combo_box("c", "")
                .width(200.0)
                .popup(Styled::frame().bg(PANEL).padding(4.0))
                .selected_bg(SELECTED)
                .show_values_with(ui, &mut v, [(0, "a"), (1, "b")], |_, item| item)
                .rect
        };
        let out = open(&ctx, &mut ui_fn);
        let rect_of = |color| {
            out.shapes
                .iter()
                .find_map(|c| match &c.shape {
                    Shape::Rect(r) if r.fill == color => Some(r.rect),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{color:?} not painted"))
        };
        let (panel, item) = (rect_of(PANEL), rect_of(SELECTED));
        // Equal insets: the panel's padding plus its theme border.
        let (l, r) = (item.left() - panel.left(), panel.right() - item.right());
        assert!((l - r).abs() < 0.5 && l < 6.0, "{item:?} in {panel:?}");
        let b = text(&texts(&out), "b").1;
        assert!(b.left() - item.left() < 16.0, "{b:?} in {item:?}");
    }

    #[test]
    fn show_values_with_selects_and_marks_changed() {
        let ctx = still_ctx();
        let mut v = 0;
        let changed = std::cell::Cell::new(false);
        let mut ui_fn = |ui: &mut egui::Ui| {
            let r = Styled::combo_box("c", "").show_values_with(
                ui,
                &mut v,
                [(0, "a"), (1, "b")],
                |_, item| item,
            );
            changed.set(changed.get() | r.changed());
            r.rect
        };
        let b = text(&texts(&open(&ctx, &mut ui_fn)), "b").1.center();
        for e in click(b) {
            pass(&ctx, e, &mut ui_fn);
        }
        assert_eq!(v, 1);
        assert!(changed.get());
    }

    #[test]
    fn item_gap_zero_makes_items_touch() {
        let ctx = still_ctx();
        let mut v = 9;
        let (a, b) = (Color32::from_rgb(1, 1, 1), Color32::from_rgb(2, 2, 2));
        let mut ui_fn = |ui: &mut egui::Ui| {
            Styled::combo_box("c", "")
                .item_gap(0.0)
                .show_values_with(ui, &mut v, [(0, "a"), (1, "b")], |v, item| {
                    item.bg(if *v == 0 { a } else { b })
                })
                .rect
        };
        let out = open(&ctx, &mut ui_fn);
        let rect_of = |color| {
            out.shapes
                .iter()
                .find_map(|c| match &c.shape {
                    Shape::Rect(r) if r.fill == color => Some(r.rect),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{color:?} not painted"))
        };
        let (ra, rb) = (rect_of(a), rect_of(b));
        assert!((rb.top() - ra.bottom()).abs() < 0.5, "{ra:?} then {rb:?}");
    }

    #[test]
    fn show_values_with_items_are_unfilled_at_rest() {
        let ctx = still_ctx();
        let grey = ctx.global_style().visuals.widgets.inactive.bg_fill;
        let mut v = 9;
        let mut ui_fn = |ui: &mut egui::Ui| {
            Styled::combo_box("c", "")
                .popup(Styled::frame().bg(PANEL))
                .show_values_with(ui, &mut v, [(0, "a"), (1, "b")], |_, item| item)
                .rect
        };
        let mut colors = Vec::new();
        fills(
            open(&ctx, &mut ui_fn).shapes.into_iter().map(|c| c.shape),
            &mut colors,
        );
        assert!(colors.contains(&PANEL));
        // The trigger's own fill is `bg_fill` too, so one is expected.
        assert_eq!(
            colors.iter().filter(|&&c| c == grey).count(),
            1,
            "{colors:?}"
        );
    }
}
