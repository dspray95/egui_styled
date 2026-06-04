use egui::{Align, Color32, InnerResponse, Layout, Shape, Ui};

use crate::{
    impl_style_builders,
    style::shared_style::{SharedStyle, background_image_shape, bgimg_fade_alpha, paint_shadows},
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

        // Content reveal: when set, the body fades in together with the
        // background image (same id + duration → in lockstep). `None` unless
        // `reveal_with_background_image` was used and an image is present.
        let reveal = if self.style.background_image_fade_content {
            match (
                self.style.background_image_fade_in,
                fade_id,
                self.style.background_image.clone(),
            ) {
                (Some(duration), Some(id), Some(image)) => Some((duration, id, image)),
                _ => None,
            }
        } else {
            None
        };

        // Reserve a slot for the background image before children so it paints
        // behind them on the same layer. `bgimg_slot` is `None` when there is
        // no background image and the slot is never set.
        let bgimg_slot = has_bg_image.then(|| ui.painter().add(Shape::Noop));

        let response = frame.show(ui, |ui| {
            // Fade the body in lockstep with the background image when a content
            // reveal is requested. Applied before any content is drawn; the bg
            // fill/image are painted on a separate slot, so the backdrop stays
            // opaque while image + content come up together.
            if let Some((duration, id, image)) = &reveal {
                let ready = matches!(
                    image.load_for_size(ui.ctx(), ui.available_size()),
                    Ok(egui::load::TexturePoll::Ready { .. })
                );
                ui.multiply_opacity(bgimg_fade_alpha(ui.ctx(), *id, *duration, ready));
            }
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

    /// Alpha of the body content rect: a non-textured rect with non-zero colour.
    /// Skips the textured background image (`brush.is_some()`) and egui::Frame's
    /// own fully-transparent `(0,0,0,0)` fill rect.
    fn content_rect_alpha(shapes: &[Shape]) -> Option<u8> {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => {
                    if let Some(a) = content_rect_alpha(inner) {
                        return Some(a);
                    }
                }
                Shape::Rect(rs)
                    if rs.brush.is_none()
                        && (rs.fill.r() > 0 || rs.fill.g() > 0 || rs.fill.b() > 0) =>
                {
                    return Some(rs.fill.a());
                }
                _ => {}
            }
        }
        None
    }

    /// Render a frame with an opaque content rect; `reveal` toggles
    /// `reveal_with_background_image` vs image-only `background_image_fade_in`.
    fn run_with_content(
        ctx: &egui::Context,
        img: egui::Image<'static>,
        secs: f32,
        reveal: bool,
        time: f64,
    ) -> Vec<Shape> {
        let mut raw = egui::RawInput::default();
        raw.time = Some(time);
        let output = ctx.run_ui(raw, |ui| {
            let mut frame = StyledFrame::new().background_image(img.clone());
            frame = if reveal {
                frame.reveal_with_background_image(secs)
            } else {
                frame.background_image_fade_in(secs)
            };
            frame.show(ui, |ui| {
                let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(10.0, 10.0));
                ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(40, 50, 60));
            });
        });
        output.shapes.into_iter().map(|cs| cs.shape).collect()
    }

    #[test]
    fn reveal_fades_content_in_with_image() {
        let ctx = egui::Context::default();
        let img = load_ready_texture(&ctx);

        // t=0: opacity 0 → content recorded as Shape::Noop, no solid rect.
        let s0 = run_with_content(&ctx, img.clone(), 0.5, true, 0.0);
        assert!(
            content_rect_alpha(&s0).is_none(),
            "content should be invisible at the fade start"
        );

        // t=0.25: half-way through the fade.
        let s1 = run_with_content(&ctx, img.clone(), 0.5, true, 0.25);
        let a1 = content_rect_alpha(&s1).expect("content present mid-fade");

        // t=0.6: past the duration → full opacity.
        let s2 = run_with_content(&ctx, img.clone(), 0.5, true, 0.6);
        let a2 = content_rect_alpha(&s2).expect("content present after fade");

        assert!(a1 > 0 && a1 < 255, "content mid-fade alpha should be partial: {a1}");
        assert_eq!(a2, 255, "content should be full opacity after the duration");
    }

    #[test]
    fn image_only_fade_does_not_fade_content() {
        let ctx = egui::Context::default();
        let img = load_ready_texture(&ctx);

        // background_image_fade_in (no content reveal): content full opacity at t=0.
        let shapes = run_with_content(&ctx, img, 0.5, false, 0.0);
        assert_eq!(
            content_rect_alpha(&shapes),
            Some(255),
            "image-only fade must leave content at full opacity"
        );
    }

    /// True if `shapes` contains a solid (non-textured) rect filled with `color`.
    fn has_solid_rect(shapes: &[Shape], color: Color32) -> bool {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => {
                    if has_solid_rect(inner, color) {
                        return true;
                    }
                }
                Shape::Rect(rs) if rs.brush.is_none() && rs.fill == color => return true,
                _ => {}
            }
        }
        false
    }

    fn run_frame_with_bg(
        ctx: &egui::Context,
        img: egui::Image<'static>,
        bg: Color32,
    ) -> Vec<Shape> {
        let raw = egui::RawInput::default();
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new()
                .bg(bg)
                .background_image(img.clone())
                .show(ui, |_ui| {});
        });
        output.shapes.into_iter().map(|cs| cs.shape).collect()
    }

    /// A pending image source — uses `from_bytes` with no loader installed, so
    /// `load_for_size` returns `Pending` every frame.
    fn pending_image() -> egui::Image<'static> {
        egui::Image::from_bytes("bytes://pending_test_image", vec![0u8; 4])
    }

    #[test]
    fn bg_set_pending_texture_paints_bg_fill_only() {
        let ctx = egui::Context::default();
        let img = pending_image();
        let bg_color = Color32::from_rgb(200, 100, 50);

        let shapes = run_frame_with_bg(&ctx, img, bg_color);

        assert!(
            has_solid_rect(&shapes, bg_color),
            "bg fill must paint while texture is pending"
        );
        assert!(
            textured_rect_alpha(&shapes).is_none(),
            "no textured rect should be present while pending"
        );
    }

    #[test]
    fn bg_set_ready_texture_paints_both() {
        let ctx = egui::Context::default();
        let img = load_ready_texture(&ctx);
        let bg_color = Color32::from_rgb(200, 100, 50);

        let shapes = run_frame_with_bg(&ctx, img, bg_color);

        assert!(
            has_solid_rect(&shapes, bg_color),
            "bg fill must be present when texture is ready"
        );
        assert!(
            textured_rect_alpha(&shapes).is_some(),
            "textured rect must be present when texture is ready"
        );
    }

    #[test]
    fn no_bg_pending_texture_paints_nothing() {
        let ctx = egui::Context::default();
        let img = pending_image();

        let raw = egui::RawInput::default();
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new()
                .background_image(img.clone())
                .show(ui, |_ui| {});
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();

        // No bg fill and no textured rect — the slot stays Noop.
        let bg_color = Color32::from_rgb(200, 100, 50);
        assert!(
            !has_solid_rect(&shapes, bg_color),
            "no solid rect expected with no bg set"
        );
        assert!(
            textured_rect_alpha(&shapes).is_none(),
            "no textured rect should appear while pending and no bg set"
        );
    }
}
