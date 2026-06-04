use egui::{Align, InnerResponse, Layout, Ui};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

pub struct StyledColumn {
    gap: Option<f32>,
    align: Option<Align>,
    justify: Option<Align>,
    style: SharedStyle,
}

impl Default for StyledColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledColumn {
    pub fn new() -> Self {
        Self {
            gap: None,
            align: None,
            justify: None,
            style: SharedStyle::default(),
        }
    }

    /// Vertical spacing between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Cross-axis (horizontal) alignment of children inside the column.
    /// Equivalent to `Layout::top_down(align)`.
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    /// Main-axis (vertical) distribution of children. `Align::Min` packs to
    /// the top, `Align::Center` packs centered, `Align::Max` packs to the
    /// bottom. Does **not** implement flexbox's `space-between` / `space-around`
    /// - see the README's "Layout" section for why.
    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = Some(justify);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        // A column is a top-down layout, which is exactly what `StyledFrame`
        // builds. Delegate align/justify/gap to the frame so the frame's
        // vertical-justify spacer (`justify_body_vertically`) drives
        // `justify(Center/Max)` on a `full_height` column — `with_main_align`
        // alone is a no-op on the main axis of a top-down layout.
        if self.style.has_frame_styles() {
            let ir = StyledFrame {
                style: self.style,
                align: self.align,
                justify: self.justify,
                gap: self.gap,
                fill_size: None,
            }
            .show(ui, body);
            InnerResponse::new(ir.inner, ir.response)
        } else {
            // Plain column with no box styling: lay out directly. (No
            // determinate height here, so `justify` is best-effort via
            // `with_main_align`, matching prior behavior.)
            let gap = self.gap;
            let mut layout = Layout::top_down(self.align.unwrap_or(Align::Min));
            if let Some(j) = self.justify {
                layout = layout.with_main_align(j);
            }
            ui.with_layout(layout, |ui| {
                if let Some(gap) = gap {
                    ui.spacing_mut().item_spacing.y = gap;
                }
                body(ui)
            })
        }
    }
}

impl_style_builders!(StyledColumn);

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    /// Run a full_height + justify column at a fixed screen size and return
    /// (content_visible, content_center_y, screen_center_y).
    fn run_full_height_justify(
        ctx: &egui::Context,
        justify: Align,
        screen_h: f32,
    ) -> (bool, f32, f32) {
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, screen_h),
            )),
            ..Default::default()
        };
        let mut content_visible = true;
        let mut content_center_y = 0.0f32;
        let screen_center_y = screen_h / 2.0;
        let _ = ctx.run_ui(raw, |ui| {
            StyledColumn::new()
                .bg(Color32::RED)
                .full_height()
                .full_width()
                .justify(justify)
                .show(ui, |ui| {
                    content_visible = ui.is_visible();
                    let resp = ui.label("hello");
                    content_center_y = resp.rect.center().y;
                });
        });
        (content_visible, content_center_y, screen_center_y)
    }

    #[test]
    fn column_full_height_justify_center_centers_on_second_frame() {
        let ctx = egui::Context::default();
        // Frame 1 measures invisibly; frame 2 renders centered.
        run_full_height_justify(&ctx, Align::Center, 300.0);
        let (visible, cy, screen_cy) = run_full_height_justify(&ctx, Align::Center, 300.0);
        assert!(visible, "content should be visible on the second frame");
        assert!(
            (cy - screen_cy).abs() < 2.0,
            "column content center y={cy} should be near screen center y={screen_cy}"
        );
    }

    #[test]
    fn column_full_height_justify_max_bottom_aligns_on_second_frame() {
        let ctx = egui::Context::default();
        let screen_h = 300.0;
        run_full_height_justify(&ctx, Align::Max, screen_h);
        let (visible, cy, _) = run_full_height_justify(&ctx, Align::Max, screen_h);
        assert!(visible);
        assert!(
            cy > screen_h * 0.6,
            "bottom-aligned column content center y={cy} should be in the lower portion of {screen_h}"
        );
    }

    #[test]
    fn column_no_full_height_stays_top_aligned() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            ..Default::default()
        };
        let mut content_center_y = 0.0f32;
        let _ = ctx.run_ui(raw, |ui| {
            StyledColumn::new()
                .bg(Color32::RED)
                // No full_height — no determinate height, so justify is a no-op.
                .justify(Align::Center)
                .show(ui, |ui| {
                    let resp = ui.label("hello");
                    content_center_y = resp.rect.center().y;
                });
        });
        assert!(
            content_center_y < 50.0,
            "without full_height, column content should be top-aligned, got center_y={content_center_y}"
        );
    }
}
