/// A flexible spacer that consumes all remaining main-axis space in the
/// current layout, pushing following siblings to the far edge.
///
/// Direction-aware: expands horizontally inside a row, vertically inside a
/// column. Most useful inside a **bounded** container — one with a determinate
/// width (`full_width`, `max_width`, or a fixed-size parent) for a row, or a
/// determinate height for a column. In an auto-sizing container the spacer
/// expands to the full parent available space, which will balloon the
/// container's size.
///
/// With a **single** spacer the pattern is unambiguous:
/// - `[A] [spacer] [B]` → A at the left edge, B at the right edge.
/// - `[spacer] [A]` → A at the right (or bottom) edge.
///
/// With **two or more** spacers in the same container, the first consumes all
/// remaining space and the second is a no-op — even N-way distribution is not
/// supported here. Use `justify(Align::Center)` or the upcoming
/// `space-between` modifier for that.
///
/// Construct via [`Styled::spacer`](crate::Styled::spacer).
pub struct StyledSpacer;

impl Default for StyledSpacer {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledSpacer {
    pub fn new() -> Self {
        StyledSpacer
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let space = if ui.layout().is_horizontal() {
            ui.available_width()
        } else {
            ui.available_height()
        };
        if space.is_finite() && space > 0.0 {
            ui.add_space(space);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(w: f32, h: f32) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w, h),
            )),
            ..Default::default()
        }
    }

    #[test]
    fn spacer_pushes_sibling_to_right_edge() {
        let ctx = egui::Context::default();
        let mut a_rect = egui::Rect::NOTHING;
        let mut b_rect = egui::Rect::NOTHING;
        let screen_w = 400.0;

        let _ = ctx.run_ui(screen(screen_w, 400.0), |ui| {
            crate::StyledRow::new().full_width().show(ui, |ui| {
                a_rect = ui.label("A").rect;
                StyledSpacer::new().show(ui);
                b_rect = ui.label("B").rect;
            });
        });

        // B should be pushed to the right edge (within a pixel rounding tolerance).
        assert!(
            b_rect.right() >= screen_w - 2.0,
            "B right={} should be near screen right={screen_w}",
            b_rect.right()
        );
        // A should stay on the left, well before B.
        assert!(
            a_rect.right() < b_rect.left(),
            "A ({}) should be left of B ({})",
            a_rect.right(),
            b_rect.left()
        );
    }

    #[test]
    fn spacer_pushes_sibling_to_bottom_in_column() {
        let ctx = egui::Context::default();
        let screen_h = 300.0;
        let mut a_rect = egui::Rect::NOTHING;
        let mut b_rect = egui::Rect::NOTHING;

        let _ = ctx.run_ui(screen(400.0, screen_h), |ui| {
            crate::StyledColumn::new().full_height().show(ui, |ui| {
                a_rect = ui.label("A").rect;
                StyledSpacer::new().show(ui);
                b_rect = ui.label("B").rect;
            });
        });

        assert!(
            b_rect.bottom() >= screen_h - 2.0,
            "B bottom={} should be near screen bottom={screen_h}",
            b_rect.bottom()
        );
        assert!(
            a_rect.bottom() < b_rect.top(),
            "A ({}) should be above B ({})",
            a_rect.bottom(),
            b_rect.top()
        );
    }

    #[test]
    fn spacer_in_row_zero_space_left_is_noop() {
        // If there is no space left (e.g. the first sibling already consumed
        // all of it), the spacer must not crash or go negative.
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(screen(400.0, 400.0), |ui| {
            crate::StyledRow::new().full_width().show(ui, |ui| {
                // Consume everything first.
                ui.set_min_width(ui.available_width());
                // Spacer with 0 available — should be a no-op.
                StyledSpacer::new().show(ui);
            });
        });
        // Reaching here without panic is the assertion.
    }
}
