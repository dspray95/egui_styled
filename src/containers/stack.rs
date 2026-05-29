use egui::{
    Align, Align2, Direction, InnerResponse, Layout, Rect, Response, Sense, Shape, UiBuilder, Vec2,
    emath::TSTransform,
};

use crate::{containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle};

type LayerFn<'a> = Box<dyn FnOnce(&mut egui::Ui) + 'a>;

struct Layer<'a> {
    offset: Vec2,
    align: Option<Align2>,
    render: LayerFn<'a>,
}

/// An overlay container that renders all children at a shared origin.
///
/// Unlike `row` and `column`, children are stacked on top of each other rather
/// than laid out in sequence. The container allocates the union of all child
/// rects so the parent flow advances correctly.
///
/// **Z-order:** layers paint in call order — the first layer is the bottom of
/// the stack, the last is on top.
///
/// Each layer can be given a pixel offset (useful for chromatic-aberration and
/// similar effects) or aligned within the stack. Aligned layers are positioned
/// within the union of all preceding layers, so the common "background first,
/// overlay centered on it" pattern works as expected.
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
        }
    }

    /// Add a layer at the shared origin (zero offset, top-left anchored).
    pub fn layer(mut self, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.layers.push(Layer {
            offset: Vec2::ZERO,
            align: None,
            render: Box::new(f),
        });
        self
    }

    /// Add a layer shifted by `offset` pixels from the shared origin.
    pub fn layer_offset(mut self, offset: Vec2, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.layers.push(Layer {
            offset,
            align: None,
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

    pub fn show(self, ui: &mut egui::Ui) -> InnerResponse<Response> {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let sense = self.sense.unwrap_or(Sense::hover());
        let layers = self.layers;

        let render = move |ui: &mut egui::Ui| {
            // Children are painted *before* the stack is allocated, but in a
            // centered/auto-sizing parent the final position isn't known until
            // allocation (and `next_widget_position` can even be infinite). So
            // paint at a provisional finite origin, allocate the measured size
            // to get the parent-determined position, then translate just this
            // stack's shapes onto it. Bracketing with `Shape::Noop` markers
            // limits the transform to our own shapes (no extra layer needed, so
            // z-order stays correct and multiple stacks don't collide).
            let layer_id = ui.layer_id();
            let start = ui.painter().add(Shape::Noop);

            let content = ui.ctx().content_rect();
            let origin = content.min;
            let available = content.size();
            let mut union: Option<Rect> = None;

            for layer in layers {
                let Layer {
                    offset,
                    align,
                    render,
                } = layer;

                let (max_rect, layout) = match align {
                    // Align within the frame established by preceding layers.
                    Some(a) => {
                        let frame = union.unwrap_or(Rect::from_min_size(origin, Vec2::ZERO));
                        (frame.translate(offset), Some(layout_for(a)))
                    }
                    // Anchor at the shared origin, natural size. The layout must
                    // be set explicitly: without it the child inherits the parent
                    // ui's layout, and a centered cross-axis (e.g. a row aligned
                    // Center) expands a single child's min_rect to the full
                    // available extent, ballooning the stack's allocated size.
                    None => (
                        Rect::from_min_size(origin + offset, available),
                        Some(Layout::top_down(Align::Min)),
                    ),
                };

                let mut builder = UiBuilder::new().max_rect(max_rect);
                if let Some(layout) = layout {
                    builder = builder.layout(layout);
                }
                let mut child = ui.new_child(builder);
                render(&mut child);

                let natural_rect = child.min_rect().translate(-offset);
                union = Some(match union {
                    None => natural_rect,
                    Some(u) => u.union(natural_rect),
                });
            }

            let union = union.unwrap_or(Rect::from_min_size(origin, Vec2::ZERO));
            let (final_rect, response) = ui.allocate_exact_size(union.size(), sense);
            let end = ui.painter().add(Shape::Noop);

            let delta = final_rect.min - union.min;
            if delta != Vec2::ZERO {
                ui.ctx().graphics_mut(|g| {
                    g.entry(layer_id)
                        .transform_range(start, end, TSTransform::from_translation(delta));
                });
            }
            response
        };

        if self.style.has_frame_styles() {
            let ir = StyledFrame {
                style: self.style,
                align: None,
                justify: None,
            }
            .show(ui, render);
            let inner = ir.inner;
            InnerResponse::new(inner, ir.response)
        } else {
            ui.scope(render)
        }
    }
}

/// Map an `Align2` to a `Layout` that places a single child at that position
/// within its `max_rect`. egui's `Layout` cannot center a single item on its
/// main axis, so center/center uses `centered_and_justified` and edge/corner
/// positions pick a packing direction plus cross-axis alignment.
fn layout_for(align: Align2) -> Layout {
    let Align2([ax, ay]) = align;
    match (ax, ay) {
        (Align::Center, Align::Center) => Layout::centered_and_justified(Direction::TopDown),
        // One axis centered, the other on an edge.
        (_, Align::Center) => {
            let dir = if ax == Align::Max {
                Direction::RightToLeft
            } else {
                Direction::LeftToRight
            };
            Layout::from_main_dir_and_cross_align(dir, Align::Center)
        }
        (Align::Center, _) => {
            let dir = if ay == Align::Max {
                Direction::BottomUp
            } else {
                Direction::TopDown
            };
            Layout::from_main_dir_and_cross_align(dir, Align::Center)
        }
        // Corner: vertical packing direction for the y axis, cross align for x.
        (_, _) => {
            let dir = if ay == Align::Max {
                Direction::BottomUp
            } else {
                Direction::TopDown
            };
            Layout::from_main_dir_and_cross_align(dir, ax)
        }
    }
}

impl_style_builders!(StyledStack<'_>);
