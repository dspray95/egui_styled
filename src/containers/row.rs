use egui::{Align, InnerResponse, Layout, Ui};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

pub struct StyledRow {
    gap: Option<f32>,
    align: Option<Align>,
    justify: Option<Align>,
    wrap: bool,
    style: SharedStyle,
}

impl Default for StyledRow {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledRow {
    pub fn new() -> Self {
        Self {
            gap: None,
            align: None,
            justify: None,
            wrap: false,
            style: SharedStyle::default(),
        }
    }

    /// Horizontal spacing between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Cross-axis (vertical) alignment of children inside the row.
    /// Equivalent to `Layout::left_to_right(align)`.
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    /// Main-axis (horizontal) distribution of children. `Align::Min` packs
    /// to the left, `Align::Center` packs centered, `Align::Max` packs to
    /// the right. Does **not** implement flexbox's `space-between` /
    /// `space-around` - see the README's "Layout" section for why.
    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = Some(justify);
        self
    }

    /// Wrap children onto a new line when they run out of horizontal room
    /// (CSS `flex-wrap: wrap`). When set, `gap` applies to both axes so the
    /// vertical spacing between wrapped lines matches the horizontal gap.
    ///
    /// Requires a bounded width to know where to break — a panel, a
    /// `full_width` row, or any sized parent. In an unbounded/infinite-width
    /// parent wrapping won't trigger.
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let gap = self.gap;
        let align = self.align;
        let justify = self.justify;
        let wrap = self.wrap;
        let render = move |ui: &mut Ui| {
            let mut layout = Layout::left_to_right(align.unwrap_or(Align::Center));
            if wrap {
                layout = layout.with_main_wrap(true);
            }
            if let Some(j) = justify {
                layout = layout.with_main_align(j);
            }
            ui.with_layout(layout, |ui| {
                if let Some(gap) = gap {
                    ui.spacing_mut().item_spacing.x = gap;
                    if wrap {
                        ui.spacing_mut().item_spacing.y = gap;
                    }
                }
                body(ui)
            })
        };

        if self.style.has_frame_styles() {
            let ir = StyledFrame {
                style: self.style,
                align: None,
                justify: None,
                gap: None,
                fill_size: None,
            }
            .show(ui, |ui| render(ui).inner);
            InnerResponse::new(ir.inner, ir.response)
        } else {
            render(ui)
        }
    }
}

impl_style_builders!(StyledRow);

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    fn screen(w: f32, h: f32) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w, h),
            )),
            ..Default::default()
        }
    }

    // Measure the span from the first to the last widget added in the row body.
    fn measure_wrap_height(ctx: &egui::Context, screen_w: f32, do_wrap: bool, gap: f32) -> f32 {
        let mut first_top = f32::MAX;
        let mut last_bottom = f32::MIN;
        let _ = ctx.run_ui(screen(screen_w, 400.0), |ui| {
            let mut row = StyledRow::new().full_width().gap(gap);
            if do_wrap {
                row = row.wrap();
            }
            row.show(ui, |ui| {
                for label in ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"] {
                    let rect = ui.label(label).rect;
                    first_top = first_top.min(rect.top());
                    last_bottom = last_bottom.max(rect.bottom());
                }
            });
        });
        (last_bottom - first_top).max(0.0)
    }

    #[test]
    fn wrap_breaks_into_multiple_lines() {
        let ctx = egui::Context::default();
        // 80px wide — not enough for 5 labels on one line.
        let h = measure_wrap_height(&ctx, 80.0, true, 4.0);
        assert!(h > 25.0, "wrapped row should be taller than one line, got {h}");
    }

    #[test]
    fn no_wrap_stays_single_line() {
        let ctx = egui::Context::default();
        let h = measure_wrap_height(&ctx, 80.0, false, 4.0);
        assert!(h < 25.0, "non-wrapped row should be one line tall, got {h}");
    }

    #[test]
    fn wrap_gap_applies_to_both_axes() {
        let ctx = egui::Context::default();
        let h_large = measure_wrap_height(&ctx, 80.0, true, 20.0);
        let h_zero = measure_wrap_height(&ctx, 80.0, true, 0.0);
        assert!(
            h_large > h_zero,
            "gap(20) wrapped row ({h_large}) should be taller than gap(0) ({h_zero})"
        );
    }
}
