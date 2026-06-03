use egui::{Align2, Context, InnerResponse, Order, Pos2, Vec2};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

/// Top-level positioned container - modal panels, backdrops, toasts.
///
/// Unlike [`StyledFrame`] / [`crate::StyledRow`] / [`crate::StyledColumn`], `show` takes
/// `&Context` (not `&mut Ui`) because the underlying [`egui::Area`] is a
/// floating layer that doesn't live inside the current `Ui` tree. Use this
/// when you need anchored or screen-relative placement; use `StyledFrame`
/// when you just want a styled box inside a normal layout.
///
/// All [`SharedStyle`] box builders (`bg`, `border`, `padding`,
/// `corner_radius`, `margin`) apply to the inner frame.
///
/// # Centering on a screen-space point
///
/// For diegetic UI (HP bars, damage numbers, world-anchored labels) you usually
/// want to place the area's *center* at a projected world-space position, not
/// its top-left.
///
/// - [`fixed_pos_centered`](Self::fixed_pos_centered) (recommended) — uses
///   last-frame's measured size to compute the offset. Stable for content
///   whose size doesn't change frame-to-frame.
/// - If you know the area's size up-front, prefer
///   `fixed_pos(center - size / 2.0)` — no lag, no caching.
pub struct StyledArea {
    id: Option<egui::Id>,
    anchor: Option<(Align2, Vec2)>,
    fixed_pos: Option<Pos2>,
    fixed_pos_centered: Option<Pos2>,
    order: Option<Order>,
    interactable: bool,
    movable: bool,
    fill_screen: bool,
    align: Option<egui::Align>,
    justify: Option<egui::Align>,
    style: SharedStyle,
}

impl Default for StyledArea {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledArea {
    pub fn new() -> Self {
        Self {
            id: None,
            anchor: None,
            fixed_pos: None,
            fixed_pos_centered: None,
            order: None,
            interactable: true,
            movable: false,
            fill_screen: false,
            align: None,
            justify: None,
            style: SharedStyle::default(),
        }
    }

    /// Stable id for this area. Defaults to one derived from call-site
    /// auto-id; set explicitly if you have multiple areas of the same shape.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = Some(egui::Id::new(id));
        self
    }

    /// Anchor to a fixed screen position (e.g., `Align2::CENTER_CENTER`).
    pub fn anchor(mut self, align: Align2, offset: Vec2) -> Self {
        self.anchor = Some((align, offset));
        self
    }

    /// Place at a fixed screen-space position. Wins over `anchor` if both set.
    pub fn fixed_pos(mut self, pos: Pos2) -> Self {
        self.fixed_pos = Some(pos);
        self
    }

    /// Place this area so its *center* lands at the given screen-space
    /// position. Uses last-frame's measured size, cached in [`egui::Memory`]
    /// under this area's id.
    ///
    /// **Trade-offs:**
    /// - On the first frame an area with this id appears, there's no cached
    ///   size yet — the area is placed at `pos` (top-left) and snaps to
    ///   centered on the next frame. Visible pop unless the content is also
    ///   fading in.
    /// - Works cleanly for stable-size content (HP bars, name plates).
    ///   For content that resizes every frame (animating counters, growing
    ///   text), centering will lag the size change by one frame.
    /// - Set an explicit [`id`](Self::id) when calling this — auto-ids
    ///   can shift under conditional rendering and lose the cached size.
    ///
    /// Wins over `fixed_pos` / `anchor` / `fill_screen` if multiple are set.
    pub fn fixed_pos_centered(mut self, pos: Pos2) -> Self {
        self.fixed_pos_centered = Some(pos);
        self
    }

    /// Set the egui rendering order (Background / Middle / Foreground / Tooltip / etc).
    pub fn order(mut self, order: Order) -> Self {
        self.order = Some(order);
        self
    }

    pub fn interactable(mut self, interactable: bool) -> Self {
        self.interactable = interactable;
        self
    }

    pub fn movable(mut self, movable: bool) -> Self {
        self.movable = movable;
        self
    }

    /// Stretch the area to cover the full screen. Useful for backdrops,
    /// dim layers, and full-bleed overlays. Overrides anchor/fixed_pos.
    pub fn fill_screen(mut self) -> Self {
        self.fill_screen = true;
        self
    }

    /// Cross-axis alignment forwarded to the inner styled frame.
    pub fn align(mut self, align: egui::Align) -> Self {
        self.align = Some(align);
        self
    }

    /// Main-axis distribution forwarded to the inner styled frame.
    pub fn justify(mut self, justify: egui::Align) -> Self {
        self.justify = Some(justify);
        self
    }

    pub fn show<R>(self, ctx: &Context, body: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
        let id = self.id.unwrap_or_else(|| egui::Id::new("styled_area"));
        let mut area = egui::Area::new(id)
            .interactable(self.interactable)
            .movable(self.movable);
        if let Some(order) = self.order {
            area = area.order(order);
        }

        // Priority: fixed_pos_centered > fill_screen > fixed_pos > anchor.
        // `fixed_pos_centered` uses last-frame's measured size; on first
        // appearance the area pops in at `pos` (top-left) and snaps centered
        // on the next frame.
        let size_cache_id = id.with("__centered_size");
        if let Some(center) = self.fixed_pos_centered {
            let last_size = ctx.memory(|mem| {
                mem.data
                    .get_temp::<Vec2>(size_cache_id)
                    .unwrap_or(Vec2::ZERO)
            });
            area = area.fixed_pos(center - last_size / 2.0);
        } else if self.fill_screen {
            area = area.fixed_pos(ctx.content_rect().min);
        } else if let Some(pos) = self.fixed_pos {
            area = area.fixed_pos(pos);
        } else if let Some((align, offset)) = self.anchor {
            area = area.anchor(align, offset);
        }

        let visible = self.style.visible;
        let frame = StyledFrame {
            style: self.style,
            align: self.align,
            justify: self.justify,
        };
        let fill_screen = self.fill_screen;
        let screen_size = ctx.content_rect().size();
        let needs_size_cache = self.fixed_pos_centered.is_some();

        let response = area.show(ctx, |ui| {
            if visible == Some(false) {
                ui.set_invisible();
            }
            frame
                .show(ui, |ui| {
                    if fill_screen {
                        ui.set_min_size(screen_size);
                    }
                    body(ui)
                })
                .inner
        });

        // Capture this frame's measured size so next frame can center.
        if needs_size_cache {
            let new_size = response.response.rect.size();
            ctx.memory_mut(|mem| mem.data.insert_temp(size_cache_id, new_size));
        }

        response
    }
}

impl_style_builders!(StyledArea);

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, pos2, vec2};

    #[test]
    #[allow(deprecated)]
    fn fill_screen_area_fills_content_rect_with_tiny_body() {
        let ctx = Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))),
            ..Default::default()
        };

        let mut measured = Vec2::ZERO;
        let mut expected = Vec2::ZERO;
        let _ = ctx.run(input, |ctx| {
            let resp = StyledArea::new().fill_screen().show(ctx, |ui| {
                ui.label("x");
            });
            measured = resp.response.rect.size();
            expected = ctx.content_rect().size();
        });

        assert!(
            (measured - expected).length() < 1.0,
            "fill_screen area measured {measured:?} but content_rect is {expected:?}"
        );
    }
}
