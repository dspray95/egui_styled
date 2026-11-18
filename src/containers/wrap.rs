use egui::{Align, Layout, Rect, Response, Sense, Ui, UiBuilder, Vec2};

use crate::{
    containers::{frame::StyledFrame, row_item},
    measure,
    style::shared_style::SharedStyle,
};

/// A single child of a [`StyledWrappingRow`], rendered once into the row's `Ui`.
type ItemFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// A horizontal row whose children flow onto new lines when they run out of
/// width (CSS `flex-wrap: wrap`).
///
/// Unlike egui's native `with_main_wrap`, this measures each item's natural
/// width (see [`crate::measure`]), then lays the items out line by line.
/// That makes it work with **scope-isolated styled widgets** (`Styled::button`,
/// etc.), which egui's own wrapping skips because each one reserves its rect at
/// the cursor without consulting the wrap layout.
///
/// Built from [`StyledRow::wrap`](crate::StyledRow::wrap); add children with
/// [`item`](Self::item) and render with [`show`](Self::show).
///
/// ```ignore
/// Styled::row().full_width().gap(6.0)
///     .wrap()
///     .item(|ui| { Styled::button("egui").show(ui); })
///     .item(|ui| { Styled::button("rust").show(ui); })
///     .show(ui);
/// ```
pub struct StyledWrappingRow<'a> {
    pub(crate) gap: f32,
    pub(crate) align: Option<Align>,
    pub(crate) style: SharedStyle,
    pub(crate) items: Vec<ItemFn<'a>>,
    pub(crate) id: Option<egui::Id>,
}

impl<'a> StyledWrappingRow<'a> {
    pub fn item(mut self, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.items.push(Box::new(f));
        self
    }

    /// Stable id for measurement state; set one if siblings come and go.
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id = Some(egui::Id::unique(id));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let gap = self.gap;
        let cross = self.align.unwrap_or(Align::Center);
        let mut items = self.items;
        let id_override = self.id;
        let n = items.len();

        let mut render = move |ui: &mut Ui| {
            if n == 0 {
                return ui.response();
            }
            let avail = ui.available_width();
            // Restored inside each item so the row's own gap doesn't leak into it.
            let spacing = ui.spacing().item_spacing;
            let line_h = ui.spacing().interact_size.y;
            let id = id_override
                .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()))
                .with("__wrap_widths");

            let cached = measure::cached::<Vec<f32>>(ui.ctx(), id);
            // One scope on both paths, so siblings' positional ids match across passes.
            let (resp, widths) = ui
                .scope(|ui| match cached {
                    Some(widths) if widths.len() == n => {
                        let lines = assign_lines(&widths, gap, avail);
                        let mut it = items.drain(..);
                        let mut drawn = Vec::with_capacity(n);
                        let resp = ui
                            .vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = gap;
                                for line in lines {
                                    let initial = egui::vec2(ui.available_width(), line_h);
                                    ui.allocate_ui_with_layout(
                                        initial,
                                        Layout::left_to_right(cross),
                                        |ui| {
                                            ui.spacing_mut().item_spacing.x = gap;
                                            for item in it.by_ref().take(line) {
                                                let k = drawn.len();
                                                let r = row_item(ui, id.with(k), spacing, item);
                                                drawn.push(r.rect.width());
                                            }
                                        },
                                    );
                                }
                            })
                            .response;
                        (resp, drawn)
                    }
                    // Unmeasured or item count changed: measure off-layout, reserve the wrapped size.
                    _ => {
                        // One row high: a centred cross-axis stretches items to the full height.
                        let row = Rect::from_min_size(
                            ui.available_rect_before_wrap().min,
                            Vec2::new(avail, line_h),
                        );
                        let mut child = ui.new_child(
                            UiBuilder::new()
                                .invisible()
                                .max_rect(row)
                                .layout(Layout::left_to_right(cross)),
                        );
                        child.spacing_mut().item_spacing.x = 0.0;
                        let sizes: Vec<Vec2> = items
                            .drain(..)
                            .enumerate()
                            .map(|(k, item)| {
                                row_item(&mut child, id.with(k), spacing, item).rect.size()
                            })
                            .collect();
                        let size = wrapped_size(&sizes, gap, avail, line_h);
                        let (_, resp) = ui.allocate_exact_size(size, Sense::hover());
                        (resp, sizes.iter().map(|s| s.x).collect())
                    }
                })
                .inner;
            measure::record(ui.ctx(), id, "wrap", widths, |a, b| widths_differ(a, b));
            resp
        };

        if self.style.has_frame_styles() {
            StyledFrame {
                style: self.style,
                align: None,
                justify: None,
                gap: None,
                fill_size: None,
                id: self.id.map(|id| id.with("__frame")),
            }
            .show(ui, |ui| render(ui))
            .inner
        } else {
            render(ui)
        }
    }
}

crate::impl_styled_widget!(['a], StyledWrappingRow<'a>);

/// Greedy line packing: returns the number of items on each line such that the
/// summed widths plus `gap` between them stay within `avail`. An item wider than
/// `avail` gets its own line.
fn assign_lines(widths: &[f32], gap: f32, avail: f32) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut count = 0usize;
    let mut line_w = 0.0f32;
    for &w in widths {
        let projected = if count == 0 { w } else { line_w + gap + w };
        if count > 0 && projected > avail {
            lines.push(count);
            count = 0;
            line_w = 0.0;
        }
        line_w = if count == 0 { w } else { line_w + gap + w };
        count += 1;
    }
    if count > 0 {
        lines.push(count);
    }
    lines
}

/// Size of `sizes` packed by [`assign_lines`]; lines are at least `line_h` tall.
fn wrapped_size(sizes: &[Vec2], gap: f32, avail: f32, line_h: f32) -> Vec2 {
    let widths: Vec<f32> = sizes.iter().map(|s| s.x).collect();
    let mut total = Vec2::ZERO;
    let mut start = 0;
    for (i, count) in assign_lines(&widths, gap, avail).into_iter().enumerate() {
        let line = &sizes[start..start + count];
        start += count;
        let w = line.iter().map(|s| s.x).sum::<f32>() + gap * (count - 1) as f32;
        let h = line.iter().map(|s| s.y).fold(line_h, f32::max);
        total.x = total.x.max(w);
        total.y += h + if i > 0 { gap } else { 0.0 };
    }
    total
}

fn widths_differ(a: &[f32], b: &[f32]) -> bool {
    a.len() != b.len() || a.iter().zip(b).any(|(x, y)| (x - y).abs() > 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_lines_single_line_when_room() {
        // three 50-wide items + gap 10 = 170 <= 400 → one line.
        let lines = assign_lines(&[50.0, 50.0, 50.0], 10.0, 400.0);
        assert_eq!(lines, vec![3]);
    }

    #[test]
    fn assign_lines_breaks_when_full() {
        // 50,50,50 with gap 10 into width 130: line1 = 50+10+50=110 (fits),
        // adding third → 110+10+50=170 > 130 → break. line1=2, line2=1.
        let lines = assign_lines(&[50.0, 50.0, 50.0], 10.0, 130.0);
        assert_eq!(lines, vec![2, 1]);
    }

    #[test]
    fn assign_lines_oversized_item_gets_own_line() {
        // second item (500) is wider than avail (200) → its own line.
        let lines = assign_lines(&[50.0, 500.0, 50.0], 8.0, 200.0);
        assert_eq!(lines, vec![1, 1, 1]);
    }

    fn screen(w: f32, h: f32) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w, h),
            )),
            ..Default::default()
        }
    }

    /// A styled-widget wrap row must break onto multiple lines AND keep every
    /// item inside the container (the bug `with_main_wrap` + scope could not fix).
    #[test]
    fn styled_pills_wrap_and_fit() {
        use crate::Styled;
        use std::cell::RefCell;
        use std::rc::Rc;

        let ctx = egui::Context::default();
        let vw = 360.0f32;
        let tags = [
            "egui",
            "rust",
            "ui",
            "layout",
            "flex",
            "wrap",
            "responsive",
            "widgets",
            "buttons",
            "labels",
            "frames",
            "rows",
            "columns",
        ];

        let run = |ctx: &egui::Context| -> (f32, f32) {
            let bottom = Rc::new(RefCell::new(f32::MIN));
            let top = Rc::new(RefCell::new(f32::MAX));
            let right = Rc::new(RefCell::new(0.0f32));
            {
                let b = Rc::clone(&bottom);
                let t = Rc::clone(&top);
                let r = Rc::clone(&right);
                let _ = crate::test_util::run_ui(ctx, screen(vw, 600.0), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        Styled::column().padding(20.0).show(ui, |ui| {
                            let mut row = Styled::row().full_width().gap(6.0).wrap();
                            for tag in tags {
                                let b = Rc::clone(&b);
                                let t = Rc::clone(&t);
                                let r = Rc::clone(&r);
                                row = row.item(move |ui| {
                                    let rect =
                                        Styled::button(tag).corner_radius(12.0).show(ui).rect;
                                    {
                                        let mut bb = b.borrow_mut();
                                        *bb = bb.max(rect.bottom());
                                    }
                                    {
                                        let mut tt = t.borrow_mut();
                                        *tt = tt.min(rect.top());
                                    }
                                    {
                                        let mut rr = r.borrow_mut();
                                        *rr = rr.max(rect.right());
                                    }
                                });
                            }
                            row.show(ui);
                        });
                    });
                });
            }
            let span = *bottom.borrow() - *top.borrow();
            (span, *right.borrow())
        };

        // Frame 1 measures, frame 2 lays out.
        let _ = run(&ctx);
        let (span, right) = run(&ctx);
        assert!(
            span > 40.0,
            "styled pills should wrap onto multiple lines (vertical span {span} too short)"
        );
        assert!(
            right <= vw,
            "wrapped pills right={right} must stay within the viewport {vw}"
        );
    }

    #[test]
    fn remeasures_when_an_item_grows() {
        // Two items fit on one 300pt line; then the first grows past half the width,
        // so the second must move to a new line instead of overflowing.
        use crate::Styled;
        let ctx = egui::Context::default();
        let mut second = egui::Rect::NOTHING;
        for first in [
            "A",
            "A",
            "A rather long label that is now a good deal wider",
            "A rather long label that is now a good deal wider",
            "A rather long label that is now a good deal wider",
        ] {
            let _ = crate::test_util::run_ui(&ctx, screen(300.0, 200.0), |ui| {
                Styled::row()
                    .gap(8.0)
                    .wrap()
                    .item(|ui| {
                        ui.add(egui::Label::new(first).extend());
                    })
                    .item(|ui| {
                        second = ui.add(egui::Label::new("Second item here").extend()).rect;
                    })
                    .show(ui);
            });
        }
        assert!(second.right() <= 300.0, "second item overflows: {second:?}");
    }
}
