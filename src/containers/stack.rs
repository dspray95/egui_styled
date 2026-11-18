use egui::{Align, Align2, Id, Layout, Rect, Response, Sense, UiBuilder, Vec2};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, measure, style::shared_style::SharedStyle,
};

type LayerFn<'a> = Box<dyn FnOnce(&mut egui::Ui) + 'a>;

struct Layer<'a> {
    offset: Vec2,
    align: Option<Align2>,
    fixed_size: Option<Vec2>,
    render: LayerFn<'a>,
}

/// An overlay container that renders all children at a shared origin.
///
/// Unlike `row` and `column`, children are stacked on top of each other rather
/// than laid out in sequence. The container allocates the union of all child
/// rects so the parent flow advances correctly.
///
/// **Z-order:** layers paint in call order - the first layer is the bottom of
/// the stack, the last is on top.
///
/// Each layer can be given a pixel offset (useful for chromatic-aberration and
/// similar effects) or aligned within the stack. Aligned layers are positioned
/// within the union of all preceding layers, so the common "background first,
/// overlay centered on it" pattern works as expected.
///
/// For layers whose visual content may be **larger than their resting footprint**
/// (scale-punch, bounce, pop), use [`layer_fixed`](StyledStack::layer_fixed) to
/// declare an explicit size that is contributed to the union while the content
/// renders freely into the overflow.
///
/// ```ignore
/// Styled::stack()
///     .layer_offset(vec2(-2.0, 0.0), |ui| {
///         Styled::label("[ENTER]").text_color(cyan).extend().show(ui);
///     })
///     .layer_offset(vec2(2.0, 0.0), |ui| {
///         Styled::label("[ENTER]").text_color(magenta).extend().show(ui);
///     })
///     .layer(|ui| {
///         Styled::label("[ENTER]").text_color(white).extend().show(ui);
///     })
///     .show(ui);
/// ```
pub struct StyledStack<'a> {
    layers: Vec<Layer<'a>>,
    sense: Option<Sense>,
    style: SharedStyle,
    id: Option<Id>,
}

impl<'a> Default for StyledStack<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StyledStack<'a> {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            sense: None,
            style: SharedStyle::default(),
            id: None,
        }
    }

    /// Stable id for the stack's measured size; set one if siblings come and go.
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id = Some(Id::unique(id));
        self
    }

    /// Add a layer at the shared origin (zero offset, top-left anchored).
    pub fn layer(mut self, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.layers.push(Layer {
            offset: Vec2::ZERO,
            align: None,
            fixed_size: None,
            render: Box::new(f),
        });
        self
    }

    /// Add a layer shifted by `offset` pixels from the shared origin.
    pub fn layer_offset(mut self, offset: Vec2, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.layers.push(Layer {
            offset,
            align: None,
            fixed_size: None,
            render: Box::new(f),
        });
        self
    }

    /// Add a layer aligned within the union of all preceding layers.
    ///
    /// For example, render a background as the first layer, then
    /// `.layer_aligned(Align2::CENTER_CENTER, ...)` to center content over it.
    pub fn layer_aligned(mut self, align: Align2, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.layers.push(Layer {
            offset: Vec2::ZERO,
            align: Some(align),
            fixed_size: None,
            render: Box::new(f),
        });
        self
    }

    /// Add a layer with an explicit layout footprint that may overflow visually.
    ///
    /// Only `size` is contributed to the stack's allocated bounds — the layer's
    /// actual rendered content is positioned within that box via `align` and is
    /// free to overflow it without pushing siblings. This is the right choice for
    /// scale-punch, bounce, or pop animations on a fixed-layout element, where
    /// the content briefly renders larger than its resting size.
    ///
    /// `align` controls how content is placed inside the declared box:
    /// - `Align2::CENTER_CENTER` — content centered, overflow is symmetric (scale punch)
    /// - `Align2::LEFT_TOP` — content anchored top-left, overflow goes down/right
    /// - Corner aligns — badge-style overflow in one corner
    ///
    /// **Note:** overflowing content is not clipped by the stack and can draw over
    /// siblings in the parent flow. This is intentional — it is the point of the API.
    ///
    /// ```ignore
    /// // Score reveal: text briefly renders at 1.4× but the stack size stays stable.
    /// let resting = vec2(120.0, 40.0);
    /// let scale = /* animated 1.0..=1.4 */;
    /// Styled::stack()
    ///     .layer_fixed(resting, Align2::CENTER_CENTER, |ui| {
    ///         Styled::label("9999")
    ///             .font_size(base_size * scale)
    ///             .extend()
    ///             .show(ui);
    ///     })
    ///     .show(ui);
    /// ```
    pub fn layer_fixed(
        mut self,
        size: Vec2,
        align: Align2,
        f: impl FnOnce(&mut egui::Ui) + 'a,
    ) -> Self {
        self.layers.push(Layer {
            offset: Vec2::ZERO,
            align: Some(align),
            fixed_size: Some(size),
            render: Box::new(f),
        });
        self
    }

    /// Make the whole stack respond to the given sense (e.g. `Sense::click()`),
    /// so the returned `Response` reports clicks/drags on the stack as a whole.
    /// Defaults to `Sense::hover()`.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = Some(sense);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> Response {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let sense = self.sense.unwrap_or(Sense::hover());
        let layers = self.layers;

        let id_override = self.id;

        let render = move |ui: &mut egui::Ui| {
            let id = id_override
                .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()))
                .with("__stack_bounds");
            let n = layers.len();
            // A changed layer count is a miss: sizes are matched by index.
            let cached = measure::cached::<Measured>(ui.ctx(), id).filter(|m| m.sizes.len() == n);
            let content = ui.ctx().content_rect();
            let available = content.size();

            // Measured: allocate first, so layers render in place and win hit tests.
            // Unmeasured: render invisibly at a provisional origin.
            let (origin, allocated) = match &cached {
                Some(m) => {
                    let (rect, response) = ui.allocate_exact_size(m.bounds.size(), sense);
                    (rect.min - m.bounds.min.to_vec2(), Some(response))
                }
                None => (content.min, None),
            };

            // Aligned content is placed from its measured size. The max rect keeps the frame's
            // size, so text wraps as before and growth isn't clamped to last frame's size.
            let place = |i: usize, frame: Rect, align: Align2| {
                let size = cached.as_ref().map_or(frame.size(), |m| m.sizes[i]);
                Rect::from_min_size(align.align_size_within_rect(size, frame).min, frame.size())
            };

            let mut union: Option<Rect> = None;
            let mut sizes = Vec::with_capacity(n);
            for (i, layer) in layers.into_iter().enumerate() {
                let Layer {
                    offset,
                    align,
                    fixed_size,
                    render,
                } = layer;

                let max_rect = match (fixed_size, align) {
                    // Fixed footprint: content aligned within the declared box, may overflow.
                    (Some(size), Some(a)) => place(i, Rect::from_min_size(origin, size), a),
                    // Align within the frame established by preceding layers.
                    (None, Some(a)) => {
                        let frame = union.unwrap_or(Rect::from_min_size(origin, Vec2::ZERO));
                        place(i, frame.translate(offset), a)
                    }
                    // Natural size at the origin.
                    (None, None) => Rect::from_min_size(origin + offset, available),
                    (Some(_), None) => unreachable!("fixed_size is always paired with align"),
                };

                // Explicit id: allocating first on a hit would otherwise shift the child's id.
                // Explicit layout: an inherited centred cross-axis would stretch the child.
                let mut builder = UiBuilder::new()
                    .scope_id(id.with(i))
                    .max_rect(max_rect)
                    .layout(Layout::top_down(Align::Min));
                if allocated.is_none() {
                    builder = builder.invisible();
                }
                let mut child = ui.new_child(builder);
                render(&mut child);
                sizes.push(child.min_rect().size());

                // Fixed layers contribute their declared size, not what they drew.
                let contribution = match fixed_size {
                    Some(size) => Rect::from_min_size(origin, size),
                    None => child.min_rect().translate(-offset),
                };
                union = Some(union.map_or(contribution, |u| u.union(contribution)));
            }

            let union = union.unwrap_or(Rect::from_min_size(origin, Vec2::ZERO));
            let response =
                allocated.unwrap_or_else(|| ui.allocate_exact_size(union.size(), sense).1);
            let bounds = union.translate(-origin.to_vec2());
            measure::record(
                ui.ctx(),
                id,
                "stack",
                Measured { bounds, sizes },
                Measured::differs,
            );
            response
        };

        if self.style.has_frame_styles() {
            StyledFrame {
                style: self.style,
                align: None,
                justify: None,
                gap: None,
                fill_size: None,
                id: id_override.map(|id| id.with("__frame")),
            }
            .show(ui, render)
            .inner
        } else {
            ui.scope(render).inner
        }
    }
}

/// What a stack measures: the layer union relative to the layer origin (offsets can make
/// `min` negative) and each layer's content size.
#[derive(Clone, Debug)]
struct Measured {
    bounds: Rect,
    sizes: Vec<Vec2>,
}

impl Measured {
    fn differs(a: &Self, b: &Self) -> bool {
        let moved = |p: egui::Pos2, q: egui::Pos2| (p - q).length() > 0.5;
        moved(a.bounds.min, b.bounds.min)
            || moved(a.bounds.max, b.bounds.max)
            || a.sizes.len() != b.sizes.len()
            || a.sizes
                .iter()
                .zip(&b.sizes)
                .any(|(x, y)| (*x - *y).length() > 0.5)
    }
}

impl_style_builders!(StyledStack<'_>);
crate::impl_styled_widget!(['a], StyledStack<'a>);

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput};

    /// Centred stack with a button over a background; returns the button's rect and click.
    fn run(ctx: &Context, sense: Sense, events: Vec<Event>) -> (Rect, bool) {
        let raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
            events,
            ..Default::default()
        };
        let mut out = (Rect::NOTHING, false);
        let _ = crate::test_util::run_ui(ctx, raw, |ui| {
            ui.vertical_centered(|ui| {
                StyledStack::new()
                    .sense(sense)
                    .layer(|ui| {
                        ui.add_space(60.0);
                        ui.label("a fairly wide background layer");
                    })
                    .layer_aligned(Align2::CENTER_CENTER, |ui| {
                        let r = ui.button("Click");
                        out = (r.rect, r.clicked());
                    })
                    .show(ui);
            });
        });
        out
    }

    fn click_at(pos: Pos2) -> Vec<Event> {
        let button = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        vec![button(true), button(false)]
    }

    fn assert_clickable_where_painted(sense: Sense) {
        let ctx = Context::default();
        let (rect, _) = run(&ctx, sense, vec![]);
        assert!(
            (rect.center().x - 400.0).abs() < 1.0,
            "painted centred: {rect:?}"
        );
        let _ = run(&ctx, sense, vec![Event::PointerMoved(rect.center())]);
        let (_, clicked) = run(&ctx, sense, click_at(rect.center()));
        assert!(clicked, "button at {rect:?} should take the click");
    }

    /// A 300x100 background with `content` in a centred layer; returns the content's rect.
    fn centred_in_layer(ctx: &Context, content: impl Fn(&mut egui::Ui) -> Rect) -> Rect {
        let mut out = Rect::NOTHING;
        let _ = crate::test_util::run_ui(ctx, RawInput::default(), |ui| {
            StyledStack::new()
                .layer(|ui| {
                    ui.allocate_space(egui::vec2(300.0, 100.0));
                })
                .layer_aligned(Align2::CENTER_CENTER, |ui| out = content(ui))
                .show(ui);
        });
        out
    }

    fn assert_centred(rect: Rect, what: &str) {
        let centre = egui::pos2(150.0, 50.0);
        assert!((rect.center() - centre).length() < 1.0, "{what}: {rect:?}");
        assert!(
            rect.width() < 290.0,
            "{what} kept its natural width: {rect:?}"
        );
    }

    #[test]
    fn containers_are_centred_in_a_centred_layer() {
        let row = |ui: &mut egui::Ui| {
            crate::Styled::row()
                .gap(4.0)
                .show(ui, |ui| {
                    ui.label("a");
                    ui.label("b");
                })
                .response
                .rect
        };
        let frame = |ui: &mut egui::Ui| {
            crate::Styled::frame()
                .padding(4.0)
                .show(ui, |ui| {
                    ui.label("framed");
                })
                .response
                .rect
        };
        let wrap = |ui: &mut egui::Ui| {
            crate::Styled::row()
                .gap(4.0)
                .wrap()
                .item(|ui| {
                    ui.label("one");
                })
                .item(|ui| {
                    ui.label("two");
                })
                .show(ui)
                .rect
        };
        for (what, content) in [
            ("row", &row as &dyn Fn(&mut egui::Ui) -> Rect),
            ("frame", &frame),
            ("wrap", &wrap),
        ] {
            let ctx = Context::default();
            assert_centred(centred_in_layer(&ctx, content), what);
        }
    }

    #[test]
    fn growing_content_stays_centred() {
        let ctx = Context::default();
        let label = |text: &'static str| move |ui: &mut egui::Ui| ui.label(text).rect;
        assert_centred(centred_in_layer(&ctx, label("short")), "short");
        let grown = centred_in_layer(&ctx, label("a considerably longer label"));
        assert_centred(grown, "grown");
        assert!(
            grown.width() > 120.0,
            "not clamped to the old width: {grown:?}"
        );
    }

    #[test]
    fn button_in_centred_stack_is_clickable_where_painted() {
        assert_clickable_where_painted(Sense::hover());
    }

    #[test]
    fn clickable_stack_does_not_swallow_child_clicks() {
        assert_clickable_where_painted(Sense::click());
    }
}
