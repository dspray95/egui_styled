use egui::{Align, Layout, Response, Ui};

use crate::{containers::frame::StyledFrame, style::shared_style::SharedStyle};

/// A single child of a [`StyledWrappingRow`], rendered once into the row's `Ui`.
type ItemFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// A horizontal row whose children flow onto new lines when they run out of
/// width (CSS `flex-wrap: wrap`).
///
/// Unlike egui's native `with_main_wrap`, this measures each item's natural
/// width on a one-frame invisible pass, then lays the items out line by line.
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
}

impl<'a> StyledWrappingRow<'a> {
    pub fn item(mut self, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.items.push(Box::new(f));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let gap = self.gap;
        let cross = self.align.unwrap_or(Align::Center);
        let mut items = self.items;
        let n = items.len();

        let mut render = move |ui: &mut Ui| {
            if n == 0 {
                return ui.response();
            }
            let avail = ui.available_width();
            // Per-position stable id for the cross-frame width cache (same
            // pattern as the distribution / vertical-justify caches).
            let id = ui
                .make_persistent_id(ui.next_auto_id())
                .with("__wrap_widths");
            let cached: Option<Vec<f32>> = ui.ctx().memory(|m| m.data.get_temp::<Vec<f32>>(id));

            match cached {
                Some(widths) if widths.len() == n => {
                    // Greedy line assignment from the measured item widths.
                    let lines = assign_lines(&widths, gap, avail);
                    let mut it = items.drain(..);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = gap;
                        for line in lines {
                            let initial =
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                            ui.allocate_ui_with_layout(
                                initial,
                                Layout::left_to_right(cross),
                                |ui| {
                                    ui.spacing_mut().item_spacing.x = gap;
                                    for _ in 0..line {
                                        if let Some(item) = it.next() {
                                            item(ui);
                                        }
                                    }
                                },
                            );
                        }
                    })
                    .response
                }
                _ => {
                    // Measure pass: render invisibly with zero spacing and record
                    // each item's natural width, then repaint to lay out for real.
                    let mut widths = Vec::with_capacity(n);
                    let resp = ui
                        .scope(|ui| {
                            ui.set_invisible();
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                for item in items.drain(..) {
                                    let r = ui.scope(|ui| item(ui)).response.rect;
                                    widths.push(r.width());
                                }
                            });
                        })
                        .response;
                    ui.ctx().memory_mut(|m| m.data.insert_temp(id, widths));
                    ui.ctx().request_repaint();
                    resp
                }
            }
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
                let _ = ctx.run_ui(screen(vw, 600.0), |ui| {
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
}
