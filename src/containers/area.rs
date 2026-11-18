use std::panic::Location;

use egui::{Align2, Context, Id, InnerResponse, Order, Pos2, Vec2};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, measure, style::shared_style::SharedStyle,
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
/// Use [`fixed_pos_centered`](Self::fixed_pos_centered).
pub struct StyledArea {
    id: Option<egui::Id>,
    caller: &'static Location<'static>,
    anchor: Option<(Align2, Vec2)>,
    fixed_pos: Option<Pos2>,
    fixed_pos_centered: Option<Pos2>,
    default_pos: Option<Pos2>,
    pivot: Option<Align2>,
    order: Option<Order>,
    interactable: bool,
    movable: bool,
    fill_screen: bool,
    align: Option<egui::Align>,
    justify: Option<egui::Align>,
    gap: Option<f32>,
    style: SharedStyle,
}

impl Default for StyledArea {
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl StyledArea {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: None,
            caller: Location::caller(),
            anchor: None,
            fixed_pos: None,
            fixed_pos_centered: None,
            default_pos: None,
            pivot: None,
            order: None,
            interactable: true,
            movable: false,
            fill_screen: false,
            align: None,
            justify: None,
            gap: None,
            style: SharedStyle::default(),
        }
    }

    /// Stable id for this area. Defaults to the construction site plus its order among
    /// same-site areas this pass; set one if that order can change between frames.
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id = Some(egui::Id::unique(id));
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

    /// Place this area so its *center* lands at `pos`. Size changes are measured
    /// (see [`crate::measure`]). Wins over `fixed_pos` / `anchor` / `fill_screen`.
    pub fn fixed_pos_centered(mut self, pos: Pos2) -> Self {
        self.fixed_pos_centered = Some(pos);
        self
    }

    /// Initial position only; egui remembers it after that, so `movable(true)` works.
    /// Lowest placement priority. Set an explicit [`id`](Self::id) so the position persists.
    pub fn default_pos(mut self, pos: Pos2) -> Self {
        self.default_pos = Some(pos);
        self
    }

    /// Which point of the area `fixed_pos` / `default_pos` refers to, e.g. `Align2::RIGHT_TOP`.
    /// Ignored by `anchor` (its align is the pivot) and `fixed_pos_centered`.
    pub fn pivot(mut self, pivot: Align2) -> Self {
        self.pivot = Some(pivot);
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

    /// Drag to move. Only takes effect with [`default_pos`](Self::default_pos) or no
    /// placement; the other placements re-pin the area every frame.
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

    /// Spacing between children, forwarded to the inner styled frame.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn show<R>(self, ctx: &Context, body: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
        let id = self.id.unwrap_or_else(|| auto_id(ctx, self.caller));
        let mut area = egui::Area::new(id)
            .interactable(self.interactable)
            .movable(self.movable);
        if let Some(order) = self.order {
            area = area.order(order);
        }

        if let Some(pivot) = self.pivot
            && self.fixed_pos_centered.is_none()
        {
            area = area.pivot(pivot);
        }

        // Priority: fixed_pos_centered > fill_screen > fixed_pos > anchor > default_pos.
        if let Some(center) = self.fixed_pos_centered {
            area = area.pivot(Align2::CENTER_CENTER).fixed_pos(center);
        } else if self.fill_screen {
            area = area.fixed_pos(ctx.content_rect().min);
        } else if let Some(pos) = self.fixed_pos {
            area = area.fixed_pos(pos);
        } else if let Some((align, offset)) = self.anchor {
            area = area.anchor(align, offset);
        } else if let Some(pos) = self.default_pos {
            area = area.default_pos(pos);
        }

        let visible = self.style.visible;
        let screen_size = ctx.content_rect().size();
        // Vertical justify (Center/Max) is handled inside StyledFrame::show via
        // the shared justify_body_vertically helper — fill_size gives it a
        // determinate height to space against, and justify passes the factor.
        let frame = StyledFrame {
            style: self.style,
            align: self.align,
            justify: self.justify,
            gap: self.gap,
            fill_size: self.fill_screen.then_some(screen_size),
            id: None,
        };
        // egui places a pivoted area from last pass's size, so a size change needs a rerun.
        let pivoted =
            self.fixed_pos_centered.is_some() || self.pivot.is_some_and(|p| p != Align2::LEFT_TOP);
        // egui hides an area on its first pass to size it.
        let first_show = egui::AreaState::load(ctx, id).is_none();

        let response = area.show(ctx, |ui| {
            if visible == Some(false) {
                ui.set_invisible();
            }
            frame.show(ui, body).inner
        });

        if first_show {
            measure::miss(ctx, id, "area");
        }
        if pivoted {
            let size = response.response.rect.size();
            measure::record(ctx, id.with("__area_size"), "area", size, |a, b| {
                (*a - *b).length() > 0.5
            });
        }

        response
    }
}

/// Call site plus its nth use this pass, so loops and shared helpers get distinct ids.
fn auto_id(ctx: &Context, caller: &Location<'_>) -> Id {
    let site = Id::unique(("styled_area", caller.file(), caller.line(), caller.column()));
    // +1 so the zeroed default never matches pass 0.
    let pass = ctx.cumulative_pass_nr() + 1;
    let nth = ctx.data_mut(|d| {
        let (seen_pass, count) = d.get_temp_mut_or_default::<(u64, u32)>(site);
        if *seen_pass == pass {
            *count += 1;
        } else {
            (*seen_pass, *count) = (pass, 0);
        }
        *count
    });
    site.with(nth)
}

impl_style_builders!(StyledArea);

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, pos2, vec2};

    #[test]
    fn auto_ids_are_distinct_and_stable() {
        let ctx = Context::default();
        let run = || {
            let mut ids = vec![];
            let _ = crate::test_util::run_ui(&ctx, egui::RawInput::default(), |ui| {
                // A first-show frame runs two passes; keep the last.
                ids.clear();
                for _ in 0..2 {
                    ids.push(StyledArea::new().show(ui.ctx(), |_| {}).response.layer_id);
                }
                ids.push(
                    crate::Styled::area()
                        .show(ui.ctx(), |_| {})
                        .response
                        .layer_id,
                );
            });
            ids
        };
        let first = run();
        assert!(first[0] != first[1] && first[1] != first[2], "{first:?}");
        assert_eq!(first, run(), "same ids next frame");
    }

    #[test]
    fn fixed_pos_centered_is_centred_on_first_frame_and_after_resizing() {
        let ctx = Context::default();
        let center = pos2(300.0, 200.0);
        let run = |text: &str| {
            let mut out = (Rect::NOTHING, false);
            let _ = crate::test_util::run_ui(&ctx, egui::RawInput::default(), |ui| {
                let r = StyledArea::new()
                    .id("centred")
                    .fixed_pos_centered(center)
                    .show(ui.ctx(), |ui| {
                        out.1 = ui.is_visible();
                        ui.label(text);
                    });
                out.0 = r.response.rect;
            });
            out
        };
        for text in ["short", "short", "a much longer line of text"] {
            let (rect, visible) = run(text);
            assert!(visible, "{text}: hidden");
            assert!((rect.center() - center).length() < 1.0, "{text}: {rect:?}");
        }
    }

    #[test]
    fn default_pos_area_is_draggable() {
        let ctx = Context::default();
        let run = |events: Vec<egui::Event>| {
            let raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))),
                events,
                ..Default::default()
            };
            let mut rect = Rect::NOTHING;
            let _ = crate::test_util::run_ui(&ctx, raw, |ui| {
                rect = StyledArea::new()
                    .id("drag_test")
                    .default_pos(pos2(790.0, 10.0))
                    .pivot(Align2::RIGHT_TOP)
                    .movable(true)
                    .show(ui.ctx(), |ui| {
                        ui.allocate_exact_size(vec2(100.0, 50.0), egui::Sense::hover());
                    })
                    .response
                    .rect;
            });
            rect
        };
        run(vec![]);
        let start = run(vec![]);
        assert!(
            (start.right_top() - pos2(790.0, 10.0)).length() < 1.0,
            "{start:?}"
        );

        let press = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let (a, b) = (start.center(), start.center() - vec2(200.0, -100.0));
        run(vec![egui::Event::PointerMoved(a), press(a, true)]);
        run(vec![egui::Event::PointerMoved(b)]);
        run(vec![press(b, false)]);
        let end = run(vec![]);
        assert!(
            (end.center() - b).length() < 1.0,
            "dragged to {b:?}, got {end:?}"
        );
    }

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
        let _ = crate::test_util::run_ui(&ctx, input, |ctx| {
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

    #[test]
    #[allow(deprecated)]
    fn fill_screen_with_justify_center_vertically_centers_content() {
        let ctx = Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))),
            ..Default::default()
        };

        let run_frame = |ctx: &Context, raw: egui::RawInput| -> (bool, f32, f32) {
            let mut content_visible = true;
            let mut widget_center_y = 0.0f32;
            let mut screen_center_y = 0.0f32;
            let _ = crate::test_util::run_ui(ctx, raw, |ctx| {
                screen_center_y = ctx.content_rect().center().y;
                StyledArea::new()
                    .id("vcenter_test")
                    .fill_screen()
                    .align(egui::Align::Center)
                    .justify(egui::Align::Center)
                    .show(ctx, |ui| {
                        content_visible = ui.is_visible();
                        let resp = ui.label("centered");
                        widget_center_y = resp.rect.center().y;
                    });
            });
            (content_visible, widget_center_y, screen_center_y)
        };

        // Right on the first frame: the measuring pass is discarded.
        let (visible, widget_center_y, screen_center_y) = run_frame(&ctx, input);
        assert!(visible);
        assert!(
            (widget_center_y - screen_center_y).abs() < 2.0,
            "widget center y={widget_center_y} should be near screen center y={screen_center_y}"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn fill_screen_recenters_content_after_resize() {
        let ctx = Context::default();

        let run_at = |ctx: &Context, w: f32, h: f32| -> (f32, f32, f32, f32) {
            let raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))),
                ..Default::default()
            };
            let mut center = (0.0f32, 0.0f32);
            let mut screen_center = (0.0f32, 0.0f32);
            let _ = crate::test_util::run_ui(ctx, raw, |ctx| {
                let sc = ctx.content_rect().center();
                screen_center = (sc.x, sc.y);
                StyledArea::new()
                    .id("resize_test")
                    .fill_screen()
                    .align(egui::Align::Center)
                    .justify(egui::Align::Center)
                    .show(ctx, |ui| {
                        let resp = ui.label("centered");
                        center = (resp.rect.center().x, resp.rect.center().y);
                    });
            });
            (center.0, center.1, screen_center.0, screen_center.1)
        };

        // Warm up at the initial size (justify needs a frame to measure height).
        run_at(&ctx, 800.0, 600.0);
        run_at(&ctx, 800.0, 600.0);

        // Now "resize" larger on the same context (memory persists, as on a real
        // window resize) and run two frames to let the height-measure settle.
        run_at(&ctx, 1600.0, 1000.0);
        let (cx, cy, scx, scy) = run_at(&ctx, 1600.0, 1000.0);

        assert!(
            (cx - scx).abs() < 2.0,
            "after growing, content center x={cx} should track screen center x={scx}"
        );
        assert!(
            (cy - scy).abs() < 2.0,
            "after growing, content center y={cy} should track screen center y={scy}"
        );

        // ...then SHRINK back down. This is the reported failure: the content
        // stays centered on the old (wider) midpoint, i.e. right of the new center.
        run_at(&ctx, 800.0, 600.0);
        let (sx, sy, sscx, sscy) = run_at(&ctx, 800.0, 600.0);

        assert!(
            (sx - sscx).abs() < 2.0,
            "after shrinking, content center x={sx} should track screen center x={sscx}"
        );
        assert!(
            (sy - sscy).abs() < 2.0,
            "after shrinking, content center y={sy} should track screen center y={sscy}"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn full_width_child_in_non_fill_screen_area_does_not_balloon() {
        use crate::Styled;
        let ctx = Context::default();

        let mut widths = vec![];
        for _ in 0..2 {
            let raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))),
                ..Default::default()
            };
            let _ = crate::test_util::run_ui(&ctx, raw, |ctx| {
                StyledArea::new().id("balloon_test").show(ctx, |ui| {
                    let resp = Styled::button("test").full_width().show(ui);
                    widths.push(resp.rect.width());
                });
            });
        }

        for &w in &widths {
            assert!(
                w.is_finite() && w < 800.0,
                "full_width in a non-fill_screen area should not balloon: width={w}"
            );
        }
    }
}
