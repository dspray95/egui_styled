use egui::{InnerResponse, Rect, Sense, UiBuilder, Vec2};

use crate::{containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle};

type LayerFn<'a> = (Vec2, Box<dyn FnOnce(&mut egui::Ui) + 'a>);

/// An overlay container that renders all children at a shared origin.
///
/// Unlike `row` and `column`, children are stacked on top of each other rather
/// than laid out in sequence. The container allocates the union of all child
/// rects so the parent flow advances correctly.
///
/// Each layer can be given an optional pixel offset from the shared origin,
/// which is useful for chromatic-aberration and similar effects.
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
    layers: Vec<LayerFn<'a>>,
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
            style: SharedStyle::default(),
        }
    }

    /// Add a layer at the shared origin (zero offset).
    pub fn layer(mut self, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.layers.push((Vec2::ZERO, Box::new(f)));
        self
    }

    /// Add a layer shifted by `offset` pixels from the shared origin.
    pub fn layer_offset(mut self, offset: Vec2, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.layers.push((offset, Box::new(f)));
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> InnerResponse<()> {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let render = move |ui: &mut egui::Ui| {
            let origin = ui.next_widget_position();
            let available = ui.available_size();
            let mut union: Option<Rect> = None;

            for (offset, layer_fn) in self.layers {
                let layer_origin = origin + offset;
                let max_rect = Rect::from_min_size(layer_origin, available);
                let mut child = ui.new_child(UiBuilder::new().max_rect(max_rect));
                layer_fn(&mut child);
                let child_rect = child.min_rect();
                // Shift the child rect back to unoffset coordinates for union
                // calculation so the allocation covers the natural size.
                let natural_rect = child_rect.translate(-offset);
                union = Some(match union {
                    None => natural_rect,
                    Some(u) => u.union(natural_rect),
                });
            }

            let alloc_rect = union.unwrap_or(Rect::from_min_size(origin, Vec2::ZERO));
            ui.allocate_rect(alloc_rect, Sense::hover());
        };

        if self.style.has_frame_styles() {
            let ir = StyledFrame {
                style: self.style,
                align: None,
                justify: None,
            }
            .show(ui, render);
            InnerResponse::new((), ir.response)
        } else {
            ui.scope(render)
        }
    }
}

impl_style_builders!(StyledStack<'_>);
