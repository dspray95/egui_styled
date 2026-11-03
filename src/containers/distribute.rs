use egui::{Align, Response, Ui};

use crate::{
    containers::frame::StyledFrame,
    style::shared_style::{Distribution, SharedStyle, distribute_row_horizontally},
};

/// A single child of a [`StyledDistributedRow`], rendered once into the row's `Ui`.
type ItemFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

pub struct StyledDistributedRow<'a> {
    pub(crate) mode: Distribution,
    pub(crate) min_gap: f32,
    pub(crate) align: Option<Align>,
    pub(crate) style: SharedStyle,
    pub(crate) items: Vec<ItemFn<'a>>,
}

impl<'a> StyledDistributedRow<'a> {
    pub fn item(mut self, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.items.push(Box::new(f));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let n = self.items.len();
        let mode = self.mode;
        let min_gap = self.min_gap;
        let cross_align = self.align.unwrap_or(Align::Center);
        let mut items = self.items;

        // render is called inside the (optional) StyledFrame, so avail and id
        // are captured inside the closure to reflect post-padding dimensions.
        let mut render = move |ui: &mut Ui| {
            let avail = ui.available_width();
            // The gap is applied as `item_spacing`; items get the parent's back so it
            // doesn't leak into widgets that space their own parts (e.g. a slider's value box).
            let spacing = ui.spacing().item_spacing;
            // Per-position stable id for the cross-frame W cache. A constant
            // source would collide between sibling distributed rows, so one row
            // would read another's cached content width. `next_auto_id` advances
            // per widget and is deterministic across frames (same pattern as the
            // vertical-justify cache in `StyledFrame`).
            let id = ui
                .make_persistent_id(ui.next_auto_id())
                .with("__distribute_w");
            distribute_row_horizontally(ui, avail, mode, min_gap, n, cross_align, id, |ui| {
                for item in items.drain(..) {
                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing = spacing;
                        item(ui);
                    });
                }
            });
            ui.response()
        };

        if self.style.has_frame_styles() {
            StyledFrame {
                style: self.style,
                align: None,
                justify: None,
                gap: None,
                fill_size: None,
            }
            .show(ui, |ui| render(ui))
            .inner
        } else {
            render(ui)
        }
    }
}

crate::impl_styled_widget!(['a], StyledDistributedRow<'a>);

#[cfg(test)]
mod tests {
    use crate::containers::row::StyledRow;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn screen(w: f32, h: f32) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w, h),
            )),
            ..Default::default()
        }
    }

    fn run_space_between(ctx: &egui::Context, screen_w: f32) -> Vec<egui::Rect> {
        let out: Rc<RefCell<Vec<egui::Rect>>> = Rc::new(RefCell::new(Vec::new()));
        {
            let out_ref = Rc::clone(&out);
            let _ = crate::test_util::run_ui(ctx, screen(screen_w, 400.0), |ui| {
                let o1 = Rc::clone(&out_ref);
                let o2 = Rc::clone(&out_ref);
                let o3 = Rc::clone(&out_ref);
                out_ref.borrow_mut().clear();
                StyledRow::new()
                    .full_width()
                    .space_between()
                    .item(move |ui| {
                        o1.borrow_mut().push(ui.label("Alpha").rect);
                    })
                    .item(move |ui| {
                        o2.borrow_mut().push(ui.label("Beta-longer").rect);
                    })
                    .item(move |ui| {
                        o3.borrow_mut().push(ui.label("Gamma").rect);
                    })
                    .show(ui);
            });
        } // out_ref dropped here
        Rc::try_unwrap(out).unwrap().into_inner()
    }

    fn run_space_evenly(ctx: &egui::Context, screen_w: f32) -> Vec<egui::Rect> {
        let out: Rc<RefCell<Vec<egui::Rect>>> = Rc::new(RefCell::new(Vec::new()));
        {
            let out_ref = Rc::clone(&out);
            let _ = crate::test_util::run_ui(ctx, screen(screen_w, 400.0), |ui| {
                let o1 = Rc::clone(&out_ref);
                let o2 = Rc::clone(&out_ref);
                let o3 = Rc::clone(&out_ref);
                out_ref.borrow_mut().clear();
                StyledRow::new()
                    .full_width()
                    .space_evenly()
                    .item(move |ui| {
                        o1.borrow_mut().push(ui.label("A").rect);
                    })
                    .item(move |ui| {
                        o2.borrow_mut().push(ui.label("B").rect);
                    })
                    .item(move |ui| {
                        o3.borrow_mut().push(ui.label("C").rect);
                    })
                    .show(ui);
            });
        }
        Rc::try_unwrap(out).unwrap().into_inner()
    }

    #[test]
    fn space_between_pins_ends() {
        let ctx = egui::Context::default();
        let screen_w = 400.0f32;

        // Frame 1: measure pass (items invisible, W cached).
        let _ = run_space_between(&ctx, screen_w);
        // Frame 2: layout applied with distribution.
        let rects = run_space_between(&ctx, screen_w);

        assert_eq!(rects.len(), 3, "all 3 items should render on frame 2");

        let left = rects[0].left();
        let right = rects[2].right();
        assert!(
            left < 10.0,
            "space_between: first item should be near left edge, got {left}"
        );
        assert!(
            right > screen_w - 10.0,
            "space_between: last item right should be near {screen_w}, got {right}"
        );
        let mid = rects[1].center().x;
        assert!(
            (mid - screen_w / 2.0).abs() < 30.0,
            "space_between: middle center ({mid:.1}) should be near screen center ({:.1})",
            screen_w / 2.0
        );
    }

    #[test]
    fn space_evenly_leading_equals_gap() {
        let ctx = egui::Context::default();
        let screen_w = 400.0f32;

        let _ = run_space_evenly(&ctx, screen_w);
        let rects = run_space_evenly(&ctx, screen_w);

        assert_eq!(rects.len(), 3);

        let leading = rects[0].left();
        let gap = rects[1].left() - rects[0].right();
        // space_evenly: leading ≈ gap (both = slack/(n+1)).
        assert!(
            (leading - gap).abs() < 5.0,
            "space_evenly: leading ({leading:.1}) should ≈ gap ({gap:.1})"
        );
    }

    /// Regression: unlike a bare spacer (which pushes trailing content past the
    /// edge), `space_between` measures content and must keep every item *inside*
    /// the container — even a `full_width` row with bg + padding.
    #[test]
    fn space_between_does_not_overflow_container() {
        let ctx = egui::Context::default();
        let screen_w = 1140.0f32;

        let run = |ctx: &egui::Context| -> f32 {
            let right = Rc::new(RefCell::new(0.0f32));
            {
                let r = Rc::clone(&right);
                let _ = crate::test_util::run_ui(ctx, screen(screen_w, 600.0), |ui| {
                    let r1 = Rc::clone(&r);
                    let r2 = Rc::clone(&r);
                    StyledRow::new()
                        .full_width()
                        .bg(egui::Color32::from_rgb(40, 42, 58))
                        .padding(10.0)
                        .space_between()
                        .item(move |ui| {
                            let right = ui.label("Left").rect.right();
                            let mut cell = r1.borrow_mut();
                            *cell = cell.max(right);
                        })
                        .item(move |ui| {
                            let right = ui.label("Right").rect.right();
                            let mut cell = r2.borrow_mut();
                            *cell = cell.max(right);
                        })
                        .show(ui);
                });
            }
            *right.borrow()
        };
        // Frame 1 measures, frame 2 lays out.
        let _ = run(&ctx);
        let max_right = run(&ctx);
        assert!(
            max_right <= screen_w,
            "space_between content right={max_right} must stay within container width {screen_w}"
        );
    }

    /// Regression: two distributed rows on the same page must not share a W
    /// cache. With a constant cache id the second row read the first row's
    /// (smaller) content width, computed too much slack, and overflowed. Each
    /// row needs a per-position id so its own measured width is used.
    #[test]
    fn sibling_distributed_rows_do_not_share_w_cache() {
        let ctx = egui::Context::default();
        let screen_w = 1140.0f32;

        // Row A has narrow content; row B has wide content. If B reads A's W it
        // overshoots far past the edge.
        let run = |ctx: &egui::Context| -> f32 {
            let b_right = Rc::new(RefCell::new(0.0f32));
            {
                let br = Rc::clone(&b_right);
                let _ = crate::test_util::run_ui(ctx, screen(screen_w, 600.0), |ui| {
                    let br1 = Rc::clone(&br);
                    // Row A: tiny content.
                    StyledRow::new()
                        .full_width()
                        .space_between()
                        .item(|ui| {
                            ui.label("A");
                        })
                        .item(|ui| {
                            ui.label("B");
                        })
                        .show(ui);
                    // Row B: wide content (a long label + a button group).
                    StyledRow::new()
                        .full_width()
                        .space_between()
                        .item(|ui| {
                            ui.label("A fairly long title goes here");
                        })
                        .item(move |ui| {
                            let r = ui.button("Some Action Button").rect.right();
                            *br1.borrow_mut() = r;
                        })
                        .show(ui);
                });
            }
            *b_right.borrow()
        };
        let _ = run(&ctx);
        let b_right = run(&ctx);
        assert!(
            b_right <= screen_w + 1.0,
            "second distributed row right={b_right} overflowed {screen_w} — W caches collided"
        );
    }

    #[test]
    fn remeasures_when_content_width_changes() {
        use crate::Styled;
        // Pin "B" to the right edge, then widen "A": the row must re-measure,
        // not keep laying out with the first frame's width.
        let ctx = egui::Context::default();
        let (mut b_right, mut edge) = (0.0, 0.0);
        for left in [
            "A",
            "A",
            "A much wider label",
            "A much wider label",
            "A much wider label",
        ] {
            let _ = crate::test_util::run_ui(&ctx, screen(400.0, 200.0), |ui| {
                edge = ui.max_rect().right();
                Styled::row()
                    .full_width()
                    .space_between()
                    .item(|ui| {
                        ui.label(left);
                    })
                    .item(|ui| {
                        b_right = ui.label("B").rect.right();
                    })
                    .show(ui);
            });
        }
        assert!(
            (b_right - edge).abs() < 1.0,
            "B should stay pinned to the right edge {edge}, got {b_right}"
        );
    }

    #[test]
    fn gap_does_not_leak_into_items() {
        // egui's Slider spaces its value box with `item_spacing`: a leaked gap widens it.
        use crate::Styled;
        let ctx = egui::Context::default();
        let mut v = 8u32;
        let mut widths = Vec::new();
        let mut right = (0.0, 0.0);
        for _ in 0..4 {
            let _ = crate::test_util::run_ui(&ctx, screen(400.0, 200.0), |ui| {
                let edge = ui.max_rect().right();
                Styled::row()
                    .full_width()
                    .space_between()
                    .item(|ui| {
                        ui.label("Build concurrency");
                    })
                    .item(|ui| {
                        let r = Styled::slider(&mut v, 1..=16).show(ui).rect;
                        widths.push(r.width());
                        right = (r.right(), edge);
                    })
                    .show(ui);
            });
        }
        assert!(
            widths.windows(2).all(|w| (w[0] - w[1]).abs() < 0.5),
            "slider width should be stable, got {widths:?}"
        );
        assert!(
            (right.0 - right.1).abs() < 1.0,
            "slider pinned right: {right:?}"
        );
    }
}
