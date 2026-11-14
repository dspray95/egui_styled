use egui::{
    CollapsingHeader, CollapsingResponse, IdSalt, Response, RichText, Shape, Ui, WidgetText,
};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{
        BackdropSlot, SharedStyle, paint_shadows, paint_widget_gradient_underlay,
        paint_widget_overlays, render_scoped,
    },
};

type IconPainter = Box<dyn FnOnce(&mut Ui, f32, &Response)>;

/// A titled section that folds away, wrapping `egui::CollapsingHeader`.
///
/// Style builders apply to the header only; the body keeps the caller's style. The header
/// draws a background only when a bg, border or gradient is set. Open state persists by
/// title unless [`id`](Self::id) is set.
///
/// Construct via [`Styled::collapsing`](crate::Styled::collapsing).
pub struct StyledCollapsing {
    text: WidgetText,
    id_salt: Option<IdSalt>,
    default_open: bool,
    indent: bool,
    icon: Option<IconPainter>,
    style: SharedStyle,
}

impl StyledCollapsing {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            id_salt: None,
            default_open: false,
            indent: true,
            icon: None,
            style: SharedStyle::default(),
        }
    }

    /// Key the open state by this instead of the title.
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Indent the body under the header, like egui. On by default.
    pub fn indent(mut self, indent: bool) -> Self {
        self.indent = indent;
        self
    }

    /// Replace the triangle. Gets openness (0 closed, 1 open) and the icon slot's response;
    /// `ui.style().interact(response).fg_stroke.color` is the header text colour.
    pub fn icon(mut self, icon: impl FnOnce(&mut Ui, f32, &Response) + 'static) -> Self {
        self.icon = Some(Box::new(icon));
        self
    }

    fn framed(&self) -> bool {
        let s = &self.style;
        s.bg.is_some()
            || s.hover_bg.is_some()
            || s.active_bg.is_some()
            || s.focus_bg.is_some()
            || s.border.is_some()
            || s.hover_border.is_some()
            || s.focus_border.is_some()
            || s.bg_gradient.is_some()
            || s.hover_bg_gradient.is_some()
            || s.active_bg_gradient.is_some()
            || s.border_gradient.is_some()
            || s.hover_border_gradient.is_some()
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R> {
        let visible = self.style.visible != Some(false);
        let framed = self.framed();
        let body_style = ui.style().clone();
        let per = self.style.resolve_per_state(ui.visuals());

        let text: WidgetText = match (&self.style.font_id, self.style.font_size) {
            (Some(font), _) => RichText::new(self.text.text()).font(font.clone()).into(),
            (None, Some(size)) => match self.text {
                WidgetText::RichText(rt) => (*rt).clone().size(size).into(),
                other => RichText::new(other.text()).size(size).into(),
            },
            (None, None) => self.text,
        };
        let mut header = CollapsingHeader::new(text)
            .default_open(self.default_open)
            .show_background(framed);
        if let Some(id) = self.id_salt {
            header = header.id_salt(id);
        }
        if let Some(icon) = self.icon {
            header = header.icon(icon);
        }
        let indent = self.indent;

        render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);
            let backdrop = BackdropSlot::reserve(ui, &self.style);
            let gradient_idx = ui.painter().add(Shape::Noop);

            let resp = ui
                .scope(|ui| {
                    SharedStyle::apply_to_visuals(&per, PseudoState::default(), ui.visuals_mut());
                    // Full-width header when framed.
                    ui.visuals_mut().collapsing_header_frame = framed;
                    // egui pads symmetrically; asymmetric collapses to the max per axis.
                    if per.padding != egui::Margin::ZERO {
                        let m = per.padding;
                        ui.spacing_mut().button_padding =
                            egui::vec2(m.left.max(m.right) as f32, m.top.max(m.bottom) as f32);
                    }
                    let body = move |ui: &mut Ui| {
                        ui.set_style(body_style);
                        body(ui)
                    };
                    let mut wrapper = egui::Frame::new();
                    if per.margin != egui::Margin::ZERO {
                        wrapper = wrapper.outer_margin(per.margin);
                    }
                    wrapper
                        .show(ui, |ui| {
                            if indent {
                                header.show(ui, body)
                            } else {
                                header.show_unindented(ui, body)
                            }
                        })
                        .inner
                })
                .inner;

            let rect = resp.header_response.rect;
            if framed {
                let resolved = SharedStyle::for_response(&per, &resp.header_response);
                backdrop.fill(ui, rect, per.corner_radius);
                paint_widget_gradient_underlay(ui, gradient_idx, rect, per.corner_radius, resolved);
                paint_widget_overlays(ui, rect, resolved);
            }
            paint_shadows(ui, shadow_idx, rect, per.corner_radius, &self.style.shadows);
            if let Some(icon) = per.cursor_icon
                && resp.header_response.hovered()
            {
                ui.ctx().set_cursor_icon(icon);
            }
            resp
        })
    }
}

impl_style_builders!(StyledCollapsing);

#[cfg(test)]
mod tests {
    use crate::Styled;
    use egui::{Color32, Event, PointerButton, Pos2, Rect, Shape, vec2};

    const HEADER: Color32 = Color32::from_rgb(200, 10, 10);

    fn raw(events: Vec<Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            events,
            ..Default::default()
        }
    }

    fn rect_fills(shapes: &[Shape], out: &mut Vec<(Rect, Color32)>) {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => rect_fills(inner, out),
                Shape::Rect(r) => out.push((r.rect, r.fill)),
                _ => {}
            }
        }
    }

    struct Frame {
        header: Rect,
        body: Option<Rect>,
        fills: Vec<(Rect, Color32)>,
    }

    fn frame(ctx: &egui::Context, events: Vec<Event>, indent: bool) -> Frame {
        let (mut header, mut body) = (Rect::NOTHING, None);
        let out = crate::test_util::run_ui(ctx, raw(events), |ui| {
            let r = Styled::collapsing("Section")
                .bg(HEADER)
                .indent(indent)
                .show(ui, |ui| ui.button("inner").rect);
            header = r.header_response.rect;
            body = r.body_returned;
        });
        let shapes: Vec<Shape> = out.shapes.into_iter().map(|c| c.shape).collect();
        let mut fills = Vec::new();
        rect_fills(&shapes, &mut fills);
        Frame {
            header,
            body,
            fills,
        }
    }

    fn click(ctx: &egui::Context, at: Pos2, indent: bool) {
        let press = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(ctx, vec![Event::PointerMoved(at), press(true)], indent);
        frame(ctx, vec![press(false)], indent);
    }

    /// Lets the open animation finish.
    fn settle(ctx: &egui::Context, indent: bool) -> Frame {
        for _ in 0..30 {
            let mut input = raw(vec![]);
            input.predicted_dt = 0.1;
            let _ = crate::test_util::run_ui(ctx, input, |ui| {
                Styled::collapsing("Section")
                    .bg(HEADER)
                    .indent(indent)
                    .show(ui, |ui| ui.button("inner"));
            });
        }
        frame(ctx, vec![], indent)
    }

    #[test]
    fn header_styled_body_is_not() {
        let ctx = egui::Context::default();
        let f = frame(&ctx, vec![], true);
        assert!(f.body.is_none(), "closed by default");
        click(&ctx, f.header.center(), true);
        let f = settle(&ctx, true);
        let body = f.body.expect("open after click");
        let header_fill = f.fills.iter().any(|(r, c)| *c == HEADER && *r == f.header);
        assert!(header_fill, "{:?}", f.fills);
        let body_fill = f
            .fills
            .iter()
            .any(|(r, c)| *c == HEADER && r.intersects(body));
        assert!(!body_fill, "body inherited header bg: {:?}", f.fills);
        assert!(
            (f.header.width() - 400.0).abs() < 20.0,
            "framed header is full width"
        );
    }

    #[test]
    fn indent_toggle() {
        let left = |indent| {
            let ctx = egui::Context::default();
            let f = frame(&ctx, vec![], indent);
            click(&ctx, f.header.center(), indent);
            let f = settle(&ctx, indent);
            f.body.expect("open").left() - f.header.left()
        };
        assert!(left(true) > 5.0);
        assert!(left(false).abs() < 1.0);
    }
}
