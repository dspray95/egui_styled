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
        let fade_id = has_bg_image
            .then(|| ui.make_persistent_id(ui.next_auto_id()).with("__bgimg_fade_start"));

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
                // Pin to exactly `size` (both min and max). `set_min_size` alone
                // only floors the area, so on a window *shrink* it stays at its
                // previous larger width and content centers on the stale midpoint.
                ui.set_min_size(size);
                ui.set_max_size(size);
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
            let fade = self.style.background_image_fade_in
                .zip(fade_id)
                .map(|(d, id)| (id, d));
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
                fade,
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

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Color32, epaint::Shape};

    fn load_ready_texture(ctx: &egui::Context) -> egui::Image<'static> {
        let handle = ctx.load_texture(
            "fade_test",
            egui::ColorImage::new([4, 4], vec![Color32::WHITE; 16]),
            Default::default(),
        );
        let sized = egui::load::SizedTexture::from_handle(&handle);
        egui::Image::from_texture(sized)
    }

    /// Walk a shape tree and return the fill alpha of the first textured rect.
    fn textured_rect_alpha(shapes: &[Shape]) -> Option<u8> {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => {
                    if let Some(a) = textured_rect_alpha(inner) {
                        return Some(a);
                    }
                }
                Shape::Rect(rs) if rs.brush.is_some() => {
                    return Some(rs.fill.a());
                }
                _ => {}
            }
        }
        None
    }

    fn run_frame_at(
        ctx: &egui::Context,
        img: egui::Image<'static>,
        fade_secs: Option<f32>,
        time: f64,
    ) -> Vec<Shape> {
        let mut raw = egui::RawInput::default();
        raw.time = Some(time);
        let output = ctx.run_ui(raw, |ui| {
            let mut frame = StyledFrame::new().background_image(img.clone());
            if let Some(d) = fade_secs {
                frame = frame.background_image_fade_in(d);
            }
            frame.show(ui, |_ui| {});
        });
        output.shapes.into_iter().map(|cs| cs.shape).collect()
    }

    #[test]
    fn fade_in_alpha_increases_and_reaches_full() {
        let ctx = egui::Context::default();
        let img = load_ready_texture(&ctx);

        // Frame at t=0: start is stamped, alpha = 0/0.5 = 0 → WHITE multiplied = transparent
        let shapes0 = run_frame_at(&ctx, img.clone(), Some(0.5), 0.0);
        let alpha0 = textured_rect_alpha(&shapes0).expect("textured rect present");

        // Frame at t=0.25: alpha = 0.25/0.5 = 0.5 → 128
        let shapes1 = run_frame_at(&ctx, img.clone(), Some(0.5), 0.25);
        let alpha1 = textured_rect_alpha(&shapes1).expect("textured rect present");

        // Frame at t=0.6: alpha >= 1.0 → full
        let shapes2 = run_frame_at(&ctx, img.clone(), Some(0.5), 0.6);
        let alpha2 = textured_rect_alpha(&shapes2).expect("textured rect present");

        assert!(alpha0 < alpha1, "alpha should increase: {alpha0} < {alpha1}");
        assert_eq!(alpha2, 255, "alpha should be full at >= duration");
    }

    #[test]
    fn no_fade_paints_full_opacity_on_first_ready_frame() {
        let ctx = egui::Context::default();
        let img = load_ready_texture(&ctx);

        let shapes = run_frame_at(&ctx, img, None, 0.0);
        let alpha = textured_rect_alpha(&shapes).expect("textured rect present");
        assert_eq!(alpha, 255, "no-fade path must paint at full opacity");
    }
}
