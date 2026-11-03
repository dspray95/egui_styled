use std::ops::RangeInclusive;

use egui::{
    Color32, DragValue, Id, Rect, Response, Shape, Slider, Stroke, Ui, Visuals, emath::Numeric,
    pos2,
};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{
        BackdropSlot, SharedStyle, paint_shadows, paint_widget_gradient_underlay,
        paint_widget_overlays, render_scoped,
    },
};

/// A styled slider. egui's `Slider` still handles input and accessibility; this paints the
/// rail, trailing fill and handle itself, since egui draws the rail and an idle handle from
/// the same visuals colour.
///
/// Without the part builders it looks like before: `bg` colours the rail and value box,
/// hover/active colours drive the handle, and `accent` is the trailing fill.
pub struct StyledSlider<'a, T: Numeric> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    text: Option<String>,
    step: Option<f64>,
    id_override: Option<Id>,
    parts: SliderParts,
    style: SharedStyle,
}

#[derive(Clone)]
struct SliderParts {
    width: Option<f32>,
    rail: Option<Color32>,
    rail_thickness: Option<f32>,
    fill: Option<Color32>,
    handle: Option<Color32>,
    hover_handle: Option<Color32>,
    handle_border: Option<Stroke>,
    handle_radius: Option<f32>,
    show_value: bool,
    value_bg: Option<Color32>,
}

impl Default for SliderParts {
    fn default() -> Self {
        Self {
            width: None,
            rail: None,
            rail_thickness: None,
            fill: None,
            handle: None,
            hover_handle: None,
            handle_border: None,
            handle_radius: None,
            show_value: true,
            value_bg: None,
        }
    }
}

impl<'a, T: Numeric> StyledSlider<'a, T> {
    pub fn new(value: &'a mut T, range: RangeInclusive<T>) -> Self {
        Self {
            value,
            range,
            text: None,
            step: None,
            id_override: None,
            parts: SliderParts::default(),
            style: SharedStyle::default(),
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Override the auto-generated widget id. Pins pseudo-state across
    /// conditional rendering - see [`crate::StyledButton::id`].
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_override = Some(Id::unique(id));
        self
    }

    /// Rail length in points. Defaults to egui's `slider_width`.
    pub fn width(mut self, width: f32) -> Self {
        self.parts.width = Some(width);
        self
    }

    /// Rail colour. Defaults to `bg`, then the theme's widget fill.
    pub fn rail(mut self, color: Color32) -> Self {
        self.parts.rail = Some(color);
        self
    }

    /// Rail thickness in points. Defaults to egui's `slider_rail_height`.
    pub fn rail_thickness(mut self, thickness: f32) -> Self {
        self.parts.rail_thickness = Some(thickness);
        self
    }

    /// Fill from the rail's start to the handle. Defaults to `accent`, if set.
    pub fn fill(mut self, color: Color32) -> Self {
        self.parts.fill = Some(color);
        self
    }

    /// Handle colour. Defaults to the widget fill for the current state.
    pub fn handle(mut self, color: Color32) -> Self {
        self.parts.handle = Some(color);
        self
    }

    /// Handle colour while hovered or dragged. Falls back to [`handle`](Self::handle).
    pub fn hover_handle(mut self, color: Color32) -> Self {
        self.parts.hover_handle = Some(color);
        self
    }

    /// Handle outline. Defaults to the widget stroke for the current state.
    pub fn handle_border(mut self, width: f32, color: Color32) -> Self {
        self.parts.handle_border = Some(Stroke::new(width, color));
        self
    }

    /// Handle radius in points. Defaults to egui's, which scales with the row height.
    pub fn handle_radius(mut self, radius: f32) -> Self {
        self.parts.handle_radius = Some(radius);
        self
    }

    /// Show the editable value box beside the rail. On by default.
    pub fn show_value(mut self, show: bool) -> Self {
        self.parts.show_value = show;
        self
    }

    /// Value box background. Defaults to `bg`, then the theme's widget fill.
    pub fn value_bg(mut self, color: Color32) -> Self {
        self.parts.value_bg = Some(color);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let visible = self.style.visible != Some(false);

        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));
        let pseudo = PseudoState::load(ui, id);

        let per = self.style.resolve_per_state(ui.visuals());
        let has_accent = self.style.accent.is_some() || self.style.hover_accent.is_some();
        let (value, range, parts) = (self.value, self.range, self.parts);

        let response = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);
            let backdrop = BackdropSlot::reserve(ui, &self.style);
            let gradient_idx = ui.painter().add(Shape::Noop);
            let response = ui
                .scope(|ui| {
                    SharedStyle::apply_to_visuals(&per, pseudo, ui.visuals_mut());

                    let mut wrapper = egui::Frame::new();
                    if per.margin != egui::Margin::ZERO {
                        wrapper = wrapper.outer_margin(per.margin);
                    }
                    if per.padding != egui::Margin::ZERO {
                        wrapper = wrapper.inner_margin(per.padding);
                    }
                    wrapper
                        .show(ui, |ui| {
                            self.style
                                .resolve_size(ui.available_width(), ui.available_height())
                                .apply_to_ui(ui);
                            // A bare rail is one fixed-size widget, so a centred parent can
                            // centre it; a row would take the full width.
                            if !parts.show_value && self.text.is_none() {
                                return rail(ui, value, &range, self.step, &parts, has_accent);
                            }
                            ui.horizontal(|ui| {
                                let mut resp =
                                    rail(ui, value, &range, self.step, &parts, has_accent);
                                if parts.show_value {
                                    resp = resp.union(value_box(ui, value, &range, &parts));
                                }
                                if let Some(text) = self.text {
                                    resp = resp.union(ui.label(text));
                                }
                                resp
                            })
                            .inner
                        })
                        .inner
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

/// egui's slider with its painting hidden, then our rail, fill and handle at its geometry.
fn rail<T: Numeric>(
    ui: &mut Ui,
    value: &mut T,
    range: &RangeInclusive<T>,
    step: Option<f64>,
    parts: &SliderParts,
    has_accent: bool,
) -> Response {
    let styled = ui.visuals().clone();
    // egui's Slider always wraps itself in a full-width row; a fixed slot lets centred
    // parents centre it.
    let width = parts.width.unwrap_or(ui.spacing().slider_width);
    let height = ui
        .text_style_height(&egui::TextStyle::Body)
        .max(ui.spacing().interact_size.y);
    let resp = ui
        .allocate_ui(egui::vec2(width, height), |ui| {
            ui.spacing_mut().slider_width = width;
            hide_painting(ui.visuals_mut());
            let mut slider = Slider::new(value, range.clone())
                .show_value(false)
                .trailing_fill(false);
            if let Some(step) = step {
                slider = slider.step_by(step);
            }
            ui.add(slider)
        })
        .inner;

    // Mirrors egui's slider geometry so the handle stays under the pointer.
    let rect = resp.rect;
    let inset = rect.height() / 2.5;
    let half = parts
        .rail_thickness
        .unwrap_or(ui.spacing().slider_rail_height)
        / 2.0;
    let track = Rect::from_min_max(
        pos2(rect.left(), rect.center().y - half),
        pos2(rect.right(), rect.center().y + half),
    );
    let (lo, hi) = (range.start().to_f64(), range.end().to_f64());
    let t = if hi == lo {
        0.0
    } else {
        ((value.to_f64() - lo) / (hi - lo)).clamp(0.0, 1.0) as f32
    };
    let x = rect.left() + inset + t * (rect.width() - 2.0 * inset);

    let painter = ui.painter();
    let rail = parts.rail.unwrap_or(styled.widgets.inactive.bg_fill);
    painter.rect_filled(track, half, rail);
    if parts.fill.is_some() || has_accent {
        let fill = parts.fill.unwrap_or(styled.selection.bg_fill);
        painter.rect_filled(
            Rect::from_min_max(track.min, pos2(x.max(track.left()), track.bottom())),
            half,
            fill,
        );
    }

    let state = styled.widgets.style(&resp);
    let engaged = resp.hovered() || resp.dragged();
    let handle = match (engaged, parts.hover_handle, parts.handle) {
        (true, Some(c), _) | (_, _, Some(c)) => c,
        _ => state.bg_fill,
    };
    painter.circle(
        pos2(x, rect.center().y),
        parts.handle_radius.unwrap_or(inset) + state.expansion,
        handle,
        parts.handle_border.unwrap_or(state.fg_stroke),
    );
    resp
}

fn value_box<T: Numeric>(
    ui: &mut Ui,
    value: &mut T,
    range: &RangeInclusive<T>,
    parts: &SliderParts,
) -> Response {
    let (lo, hi) = (range.start().to_f64(), range.end().to_f64());
    let speed = (hi - lo).abs() / f64::from(parts.width.unwrap_or(ui.spacing().slider_width));
    ui.scope(|ui| {
        if let Some(bg) = parts.value_bg {
            let w = &mut ui.visuals_mut().widgets;
            for v in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                v.bg_fill = bg;
                v.weak_bg_fill = bg;
            }
        }
        ui.add(DragValue::new(value).range(range.clone()).speed(speed))
    })
    .inner
}

/// Input and accessibility only: egui paints nothing we then paint over.
fn hide_painting(visuals: &mut Visuals) {
    let w = &mut visuals.widgets;
    for v in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        v.bg_fill = Color32::TRANSPARENT;
        v.weak_bg_fill = Color32::TRANSPARENT;
        v.bg_stroke = Stroke::NONE;
        v.fg_stroke = Stroke::NONE;
        v.expansion = 0.0;
    }
    visuals.selection.bg_fill = Color32::TRANSPARENT;
}

impl_style_builders!(['a, T: Numeric], StyledSlider<'a, T>);
crate::impl_styled_widget!(['a, T: Numeric], StyledSlider<'a, T>);

#[cfg(test)]
mod tests {
    use crate::Styled;
    use egui::{Color32, Event, PointerButton, Pos2, Rect, Shape, vec2};

    const RAIL: Color32 = Color32::from_rgb(10, 20, 30);
    const FILL: Color32 = Color32::from_rgb(200, 0, 0);
    const HANDLE: Color32 = Color32::from_rgb(0, 200, 0);

    fn raw(events: Vec<Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 200.0))),
            events,
            ..Default::default()
        }
    }

    fn fills(shapes: &[Shape], out: &mut Vec<Color32>) {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => fills(inner, out),
                Shape::Rect(r) => out.push(r.fill),
                Shape::Circle(c) => out.push(c.fill),
                _ => {}
            }
        }
    }

    fn texts(shapes: &[Shape]) -> Vec<String> {
        shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn parts_get_their_own_colors() {
        let ctx = egui::Context::default();
        let mut v = 5.0_f32;
        let out = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| {
            Styled::slider(&mut v, 0.0..=10.0)
                .rail(RAIL)
                .fill(FILL)
                .handle(HANDLE)
                .show(ui);
        });
        let shapes: Vec<Shape> = out.shapes.into_iter().map(|c| c.shape).collect();
        let mut colors = Vec::new();
        fills(&shapes, &mut colors);
        for c in [RAIL, FILL, HANDLE] {
            assert!(colors.contains(&c), "{c:?} not painted in {colors:?}");
        }
        assert!(
            texts(&shapes).iter().any(|t| t.contains('5')),
            "value box shown"
        );
    }

    #[test]
    fn show_value_false_hides_the_value_box() {
        let ctx = egui::Context::default();
        let mut v = 7u32;
        let out = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| {
            Styled::slider(&mut v, 0..=10).show_value(false).show(ui);
        });
        let shapes: Vec<Shape> = out.shapes.into_iter().map(|c| c.shape).collect();
        assert!(texts(&shapes).is_empty(), "{:?}", texts(&shapes));
    }

    #[test]
    fn dragging_still_moves_the_value() {
        let ctx = egui::Context::default();
        let mut v = 0.0_f32;
        let frame = |v: &mut f32, events| {
            let mut rect = Rect::NOTHING;
            let _ = crate::test_util::run_ui(&ctx, raw(events), |ui| {
                rect = Styled::slider(v, 0.0..=10.0)
                    .show_value(false)
                    .show(ui)
                    .rect;
            });
            rect
        };
        let rect = frame(&mut v, vec![]);
        let end = Pos2::new(rect.right() - 1.0, rect.center().y);
        let press = |pressed| Event::PointerButton {
            pos: end,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(&mut v, vec![Event::PointerMoved(end), press(true)]);
        frame(&mut v, vec![press(false)]);
        assert!(v > 9.5, "click at the rail's end should set ~max, got {v}");
    }

    #[test]
    fn bare_rail_centres_in_a_centred_layout() {
        let ctx = egui::Context::default();
        let mut v = 3.0_f32;
        let mut rect = Rect::NOTHING;
        let _ = crate::test_util::run_ui(&ctx, raw(vec![]), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                rect = Styled::slider(&mut v, 0.0..=10.0)
                    .show_value(false)
                    .width(200.0)
                    .show(ui)
                    .rect;
            });
        });
        assert!(
            (rect.center().x - 200.0).abs() < 1.0,
            "centred in 400: {rect:?}"
        );
        assert!((rect.width() - 200.0).abs() < 1.0, "{rect:?}");
    }
}
