use egui::{Align, Color32, InnerResponse, Layout, Shape, Ui};

use crate::{
    impl_style_builders,
    style::shared_style::{SharedStyle, background_image_shape, paint_shadows},
};

/// A styled box that lives inside the current layout.
///
/// The in-layout counterpart to [`StyledArea`](crate::StyledArea): use
/// `StyledFrame` for a filled / bordered / padded panel inside the current
/// `Ui` tree, and [`StyledArea`](crate::StyledArea) for floating,
/// screen-anchored placement. Applies the [`SharedStyle`] box builders
/// (`bg`, `border`, `padding`, `corner_radius`, `margin`, `background_image`,
/// shadows) and optional [`align`](Self::align) / [`justify`](Self::justify) /
/// [`gap`](Self::gap) for its children.
///
/// Construct via [`Styled::frame`](crate::Styled::frame).
pub struct StyledFrame {
    pub style: SharedStyle,
    pub align: Option<Align>,
    pub justify: Option<Align>,
    pub gap: Option<f32>,
    pub fill_size: Option<egui::Vec2>,
}

impl Default for StyledFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledFrame {
    pub fn new() -> Self {
        Self {
            style: SharedStyle::default(),
            align: None,
            justify: None,
            gap: None,
            fill_size: None,
        }
    }

    /// Cross-axis (horizontal) alignment of the frame's children.
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    /// Main-axis (vertical) distribution of the frame's children. Treated as
    /// top-down inside the frame; see [`crate::StyledColumn::justify`] for details.
    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = Some(justify);
        self
    }

    /// Spacing between children, applied to both axes (`item_spacing.x` and `.y`).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        if self.style.visible == Some(false) {
            ui.set_invisible();
        }

        let shadow_idx = ui.painter().add(Shape::Noop);
        let corner_radius = self.style.corner_radius.unwrap_or_default();
        let shadows = self.style.shadows.clone();

        let has_bg_image = self.style.background_image.is_some();

        let mut frame = egui::Frame::default();
        // When a background image is set we paint fill + texture + border ourselves
        // so the texture sits between the fill and the border. When there is no
        // background image, all three delegate to egui::Frame as before.
        if has_bg_image {
            if let Some(r) = self.style.corner_radius {
                frame = frame.corner_radius(r);
            }
            if let Some(p) = self.style.padding {
                frame = frame.inner_margin(p);
            }
            if let Some(m) = self.style.margin {
                frame = frame.outer_margin(m);
            }
            // No .fill / .stroke — we'll draw those inside the bgimg shape.
        } else {
            if let Some(bg) = self.style.bg {
                frame = frame.fill(bg);
            }
            if let Some(r) = self.style.corner_radius {
                frame = frame.corner_radius(r);
            }
            if let Some(p) = self.style.padding {
                frame = frame.inner_margin(p);
            }
            if let Some(m) = self.style.margin {
                frame = frame.outer_margin(m);
            }
            if let Some(b) = self.style.border {
                frame = frame.stroke(b);
            }
        }

        let full_width = self.style.full_width;
        let align = self.align;
        let justify = self.justify;
        let gap = self.gap;
        let fill_size = self.fill_size;

        // Reserve a slot for the background image before children so it paints
        // behind them on the same layer. `bgimg_slot` is `None` when there is
        // no background image and the slot is never set.
        let bgimg_slot = has_bg_image.then(|| ui.painter().add(Shape::Noop));

        let response = frame.show(ui, |ui| {
            // Expand to the fill size (e.g. full screen) before building the
            // layout, so cross-axis (horizontal) centering measures against the
            // full width. Main-axis (vertical) centering is handled by the
            // caller via a spacer — egui's top-down layout always pins the main
            // axis to the top regardless of `with_main_align`.
            if let Some(size) = fill_size {
                ui.set_min_size(size);
            }
            if full_width {
                ui.set_min_width(ui.available_width());
            }
            if let Some(g) = gap {
                ui.spacing_mut().item_spacing = egui::Vec2::splat(g);
            }
            if align.is_some() || justify.is_some() {
                let mut layout = Layout::top_down(align.unwrap_or(Align::Min));
                if let Some(j) = justify {
                    layout = layout.with_main_align(j);
                }
                ui.with_layout(layout, body).inner
            } else {
                body(ui)
            }
        });

        // Paint the background image into the slot now that we know the rect.
        if let (Some(slot), Some(image)) = (bgimg_slot, self.style.background_image) {
            let rect = response.response.rect;
            let tint = self.style.background_image_tint.unwrap_or(Color32::WHITE);
            let fit = self.style.background_image_fit;
            // For Cover we need intrinsic size; load_for_size is called inside
            // background_image_shape and covers the Pending (not-yet-loaded) case.
            if let Some(shape) = background_image_shape(
                ui,
                rect,
                corner_radius,
                &image,
                fit,
                tint,
                self.style.bg,
                self.style.border,
            ) {
                ui.painter().set(slot, shape);
            }
            // If None (still loading) the Shape::Noop stays, image appears next frame.
        }

        paint_shadows(
            ui,
            shadow_idx,
            response.response.rect,
            corner_radius,
            &shadows,
        );

        response
    }
}

impl_style_builders!(StyledFrame);
