use egui::{Align, InnerResponse, Layout, Ui};

use crate::{
    containers::distribute::DistributedRow,
    containers::frame::StyledFrame,
    containers::wrap::WrappingRow,
    impl_style_builders,
    style::shared_style::{Distribution, SharedStyle},
};

pub struct StyledRow {
    gap: Option<f32>,
    align: Option<Align>,
    justify: Option<Align>,
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
    /// (CSS `flex-wrap: wrap`). Transitions to [`WrappingRow`] — add children
    /// with `.item()` and render with `.show(ui)`.
    ///
    /// This is item-based (rather than a `show(ui, body)` closure) because it
    /// measures each child's natural width to place it, which is what lets
    /// scope-isolated styled widgets wrap — see [`WrappingRow`]. `gap` applies
    /// to both axes, so the vertical spacing between wrapped lines matches the
    /// horizontal gap. Requires a bounded width (`full_width` or a sized parent)
    /// to know where to break.
    pub fn wrap<'a>(self) -> WrappingRow<'a> {
        WrappingRow {
            gap: self.gap.unwrap_or(0.0),
            align: self.align,
            style: self.style,
            items: Vec::new(),
        }
    }

    /// Distribute children evenly with no leading/trailing space and equal gaps
    /// between items (CSS `justify-content: space-between`). Transitions to
    /// [`DistributedRow`] — call `.item()` to add children, then `.show(ui)`.
    pub fn space_between<'a>(self) -> DistributedRow<'a> {
        DistributedRow {
            mode: Distribution::SpaceBetween,
            min_gap: self.gap.unwrap_or(0.0),
            align: self.align,
            style: self.style,
            items: Vec::new(),
        }
    }

    /// Distribute children with equal space around each item (CSS
    /// `justify-content: space-around`). Transitions to [`DistributedRow`].
    pub fn space_around<'a>(self) -> DistributedRow<'a> {
        DistributedRow {
            mode: Distribution::SpaceAround,
            min_gap: self.gap.unwrap_or(0.0),
            align: self.align,
            style: self.style,
            items: Vec::new(),
        }
    }

    /// Distribute children with equal space between, before, and after every
    /// item (CSS `justify-content: space-evenly`). Transitions to [`DistributedRow`].
    pub fn space_evenly<'a>(self) -> DistributedRow<'a> {
        DistributedRow {
            mode: Distribution::SpaceEvenly,
            min_gap: self.gap.unwrap_or(0.0),
            align: self.align,
            style: self.style,
            items: Vec::new(),
        }
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let gap = self.gap;
        let align = self.align;
        let justify = self.justify;
        let render = move |ui: &mut Ui| {
            let mut layout = Layout::left_to_right(align.unwrap_or(Align::Center));
            if let Some(j) = justify {
                layout = layout.with_main_align(j);
            }
            // Seed a one-row height instead of the full available height.
            // `with_layout` would pass `available_size_before_wrap()` (the entire
            // remaining height), and a `left_to_right(Center)` layout then centers
            // content across that whole span — ballooning the row to fill its
            // parent vertically. `ui.horizontal` avoids this by seeding the height
            // with `interact_size.y`; we replicate that so custom align / justify
            // still work without ballooning.
            let initial_size = egui::vec2(
                ui.available_size_before_wrap().x,
                ui.spacing().interact_size.y,
            );
            ui.allocate_ui_with_layout(initial_size, layout, |ui| {
                if let Some(gap) = gap {
                    ui.spacing_mut().item_spacing.x = gap;
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
crate::impl_styled_container!(StyledRow);

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
    fn plain_row_stays_single_line() {
        // A non-wrapping row keeps all children on one line (one-line height).
        let ctx = egui::Context::default();
        let mut first_top = f32::MAX;
        let mut last_bottom = f32::MIN;
        let _ = ctx.run_ui(screen(80.0, 400.0), |ui| {
            StyledRow::new().full_width().gap(4.0).show(ui, |ui| {
                for label in ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"] {
                    let rect = ui.label(label).rect;
                    first_top = first_top.min(rect.top());
                    last_bottom = last_bottom.max(rect.bottom());
                }
            });
        });
        let h = (last_bottom - first_top).max(0.0);
        assert!(h < 25.0, "non-wrapped row should be one line tall, got {h}");
    }
}
