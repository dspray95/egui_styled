use egui::{Align, Color32, InnerResponse, Layout, Shape, Ui};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{
        SharedStyle, background_image_shape, bgimg_fade_alpha, justify_body_vertically,
        paint_shadows, paint_side_borders,
    },
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

        // A frame is non-interactive, so resolve its border against the base
        // (inactive) state. `Some` only when per-side overrides are set — that's
        // the signal to paint edges manually and skip egui's uniform stroke.
        let border_sides = {
            let resolved = self
                .style
                .resolve(PseudoState::default(), &ui.visuals().widgets.inactive);
            resolved
                .has_border_overrides
                .then_some(resolved.border_sides)
        };

        let has_bg_image = self.style.background_image.is_some();
        let fade_id = has_bg_image.then(|| {
            ui.make_persistent_id(ui.next_auto_id())
                .with("__bgimg_fade_start")
        });

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
            // When per-side overrides are set we paint the border ourselves
            // after the frame renders; otherwise delegate the uniform stroke
            // to egui so rounded corners are handled for us.
            if border_sides.is_none()
                && let Some(b) = self.style.border
            {
                frame = frame.stroke(b);
            }
        }

        let full_width = self.style.full_width;
        let full_height = self.style.full_height;
        let min_width = self.style.min_width;
        let max_width = self.style.max_width;
        let min_height = self.style.min_height;
        let max_height = self.style.max_height;
        let style_for_pct = self.style.clone();
        let align = self.align;
        let justify = self.justify;
        let gap = self.gap;
        let fill_size = self.fill_size;

        // Stable id for caching the measured content height used by vertical
        // justify. Only allocated when justify is set (same lazy pattern as
        // fade_id — avoids burning an auto-id slot when not needed).
        let vjustify_id = justify.map(|_| {
            ui.make_persistent_id(ui.next_auto_id())
                .with("__vjustify_content_h")
        });

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
            // Pin to exactly `fill_size` (both min and max). `set_min_size` alone
            // only floors the area, so on a window *shrink* it stays at its
            // previous larger width and content centers on the stale midpoint.
            if let Some(size) = fill_size {
                ui.set_min_size(size);
                ui.set_max_size(size);
            }
            // Capture available before any sizing mutations for pct resolution.
            let avail_w = ui.available_width();
            let avail_h = ui.available_height();
            let pct_w = style_for_pct.resolved_width_pct(avail_w);
            let pct_h = style_for_pct.resolved_height_pct(avail_h);
            // Compute effective definite width for aspect ratio derivation.
            let effective_w = pct_w.or_else(|| {
                if full_width {
                    Some(max_width.map_or(avail_w, |m| avail_w.min(m)))
                } else {
                    None
                }
            });
            let aspect_h = effective_w.and_then(|w| style_for_pct.resolved_aspect_height(w));

            if let Some(w) = pct_w {
                // Definite width: pin both min and max; supersedes full_width.
                ui.set_min_width(w);
                ui.set_max_width(w);
            } else {
                // Apply max constraints first so full_width reads the capped
                // available size, and min constraints last so an explicit minimum
                // always wins over full_width.
                if let Some(w) = max_width {
                    ui.set_max_width(w);
                }
                if full_width {
                    ui.set_min_width(ui.available_width());
                }
                if let Some(w) = min_width {
                    ui.set_min_width(w);
                }
            }

            // Capture fill_height for vertical justify *before* set_min_height
            // changes the available size. Precedence: fill_size > height_pct > aspect > full_height.
            let fill_height_val: Option<f32> = fill_size
                .map(|s| s.y)
                .or(pct_h)
                .or(aspect_h)
                .or(if full_height { Some(avail_h) } else { None });

            if let Some(h) = pct_h {
                // Definite height: pin both min and max; supersedes full_height.
                ui.set_min_height(h);
                ui.set_max_height(h);
            } else if let Some(h) = aspect_h {
                ui.set_min_height(h);
                ui.set_max_height(h);
            } else {
                if let Some(h) = max_height {
                    ui.set_max_height(h);
                }
                if full_height {
                    ui.set_min_height(ui.available_height());
                }
                if let Some(h) = min_height {
                    ui.set_min_height(h);
                }
            }
            if let Some(g) = gap {
                ui.spacing_mut().item_spacing = egui::Vec2::splat(g);
            }
            // Vertical justify via top spacer when the frame has a determinate
            // height and justify is Center or Max. egui's `with_main_align` is a
            // no-op for top-down layouts, so we use the measured-spacer approach
            // instead. Horizontal alignment is applied as the cross-axis layout
            // inside the same body closure.
            let justify_factor = justify.map(|j| j.to_factor()).unwrap_or(0.0);
            match (fill_height_val, vjustify_id, justify_factor > 0.0) {
                (Some(fill_h), Some(vid), true) => {
                    justify_body_vertically(ui, fill_h, justify_factor, vid, |ui| {
                        if let Some(a) = align {
                            ui.with_layout(Layout::top_down(a), body).inner
                        } else {
                            body(ui)
                        }
                    })
                }
                _ => {
                    if align.is_some() || justify.is_some() {
                        let mut layout = Layout::top_down(align.unwrap_or(Align::Min));
                        if let Some(j) = justify {
                            layout = layout.with_main_align(j);
                        }
                        ui.with_layout(layout, body).inner
                    } else {
                        body(ui)
                    }
                }
            }
        });

        // Paint the background image into the slot now that we know the rect.
        if let (Some(slot), Some(image)) = (bgimg_slot, self.style.background_image) {
            let rect = response.response.rect;
            let tint = self.style.background_image_tint.unwrap_or(Color32::WHITE);
            let fit = self.style.background_image_fit;
            let fade = self
                .style
                .background_image_fade_in
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
                border_sides,
                fade,
            ) {
                ui.painter().set(slot, shape);
            }
            // If None (still loading) the Shape::Noop stays, image appears next frame.
        }

        // Per-side borders for the non-bg-image path (the bg-image path paints
        // them inside `background_image_shape`). Drawn on top of the frame.
        if !has_bg_image && let Some(sides) = border_sides {
            paint_side_borders(ui.painter(), response.response.rect, sides);
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
        let raw = egui::RawInput {
            time: Some(time),
            ..Default::default()
        };
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

        assert!(
            alpha0 < alpha1,
            "alpha should increase: {alpha0} < {alpha1}"
        );
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
        let raw = egui::RawInput {
            time: Some(time),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            let mut frame = StyledFrame::new().background_image(img.clone());
            frame = if reveal {
                frame.reveal_with_background_image(secs)
            } else {
                frame.background_image_fade_in(secs)
            };
            frame.show(ui, |ui| {
                let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(10.0, 10.0));
                ui.painter()
                    .rect_filled(rect, 0.0, Color32::from_rgb(40, 50, 60));
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

        assert!(
            a1 > 0 && a1 < 255,
            "content mid-fade alpha should be partial: {a1}"
        );
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
                Shape::Vec(inner) if has_solid_rect(inner, color) => return true,
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

    /// Return the bounding rect of the first non-transparent, non-textured filled rect.
    fn first_solid_rect(shapes: &[Shape]) -> Option<egui::Rect> {
        for shape in shapes {
            match shape {
                Shape::Vec(inner) => {
                    if let Some(r) = first_solid_rect(inner) {
                        return Some(r);
                    }
                }
                Shape::Rect(rs) if rs.brush.is_none() && rs.fill.a() > 0 => {
                    return Some(rs.rect);
                }
                _ => {}
            }
        }
        None
    }

    #[test]
    fn max_width_constrains_frame() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 400.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            // full_width makes the frame try to expand to available (400px), but
            // max_width caps it at 100px. max is applied first so full_width reads
            // the capped available_width.
            StyledFrame::new()
                .bg(Color32::RED)
                .max_width(100.0)
                .full_width()
                .show(ui, |_ui| {});
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
        let rect = first_solid_rect(&shapes).expect("bg fill rect present");
        assert!(
            rect.width() <= 100.0 + 1.0,
            "frame width {} should be <= max_width 100",
            rect.width()
        );
    }

    #[test]
    fn min_height_expands_frame() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 400.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new()
                .bg(Color32::RED)
                .min_height(80.0)
                .show(ui, |_ui| {});
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
        let rect = first_solid_rect(&shapes).expect("bg fill rect present");
        assert!(
            rect.height() >= 80.0 - 1.0,
            "frame height {} should be >= min_height 80",
            rect.height()
        );
    }

    #[test]
    fn full_height_fills_parent() {
        let ctx = egui::Context::default();
        let parent_height = 200.0;
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, parent_height),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new()
                .bg(Color32::RED)
                .full_height()
                .show(ui, |_ui| {});
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
        let rect = first_solid_rect(&shapes).expect("bg fill rect present");
        assert!(
            rect.height() >= parent_height - 1.0,
            "frame height {} should fill parent height {}",
            rect.height(),
            parent_height
        );
    }

    #[test]
    fn no_size_constraints_leaves_frame_natural() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 400.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new().bg(Color32::RED).show(ui, |ui| {
                // Some content so the frame has non-zero natural size.
                ui.set_min_width(50.0);
                ui.set_min_height(20.0);
            });
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
        let rect = first_solid_rect(&shapes).expect("bg fill rect present");
        // Without any size constraints the frame should be exactly content-sized.
        assert!(
            rect.width() >= 50.0 - 1.0 && rect.height() >= 20.0 - 1.0,
            "unconstrained frame should match content size, got {:?}",
            rect
        );
    }

    /// Run a full_height + justify frame at a fixed screen size and return
    /// (content_visible, content_center_y, screen_center_y).
    fn run_full_height_justify(
        ctx: &egui::Context,
        justify: egui::Align,
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
            StyledFrame::new()
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
    fn full_height_justify_center_hides_on_first_frame() {
        let ctx = egui::Context::default();
        let (visible, _, _) = run_full_height_justify(&ctx, egui::Align::Center, 300.0);
        assert!(
            !visible,
            "content should be invisible on the first frame while height is measured"
        );
    }

    #[test]
    fn full_height_justify_center_centers_on_second_frame() {
        let ctx = egui::Context::default();
        // Frame 1: measurement frame (invisible).
        run_full_height_justify(&ctx, egui::Align::Center, 300.0);
        // Frame 2: content renders centered.
        let (visible, cy, screen_cy) = run_full_height_justify(&ctx, egui::Align::Center, 300.0);
        assert!(visible, "content should be visible on the second frame");
        assert!(
            (cy - screen_cy).abs() < 2.0,
            "content center y={cy} should be near screen center y={screen_cy}"
        );
    }

    #[test]
    fn full_height_justify_max_bottom_aligns_on_second_frame() {
        let ctx = egui::Context::default();
        let screen_h = 300.0;
        run_full_height_justify(&ctx, egui::Align::Max, screen_h);
        let (visible, cy, _) = run_full_height_justify(&ctx, egui::Align::Max, screen_h);
        assert!(visible);
        // Bottom-aligned: content center should be near the bottom quarter.
        assert!(
            cy > screen_h * 0.6,
            "bottom-aligned content center y={cy} should be in the lower portion of {screen_h}"
        );
    }

    #[test]
    fn no_full_height_justify_stays_top_aligned() {
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
            StyledFrame::new()
                .bg(Color32::RED)
                // No full_height — no determinate height, so justify is a no-op.
                .justify(egui::Align::Center)
                .show(ui, |ui| {
                    let resp = ui.label("hello");
                    content_center_y = resp.rect.center().y;
                });
        });
        // Content should sit near the top, not centered in the 300px screen.
        assert!(
            content_center_y < 50.0,
            "without full_height, content should be top-aligned, got center_y={content_center_y}"
        );
    }

    #[test]
    fn width_pct_50_is_half_screen() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 400.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new().bg(Color32::RED).width_pct(50.0).show(ui, |_ui| {});
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
        let rect = first_solid_rect(&shapes).expect("bg fill rect present");
        assert!(
            (rect.width() - 200.0).abs() < 2.0,
            "width_pct(50) on 400px screen should be ~200px, got {}",
            rect.width()
        );
    }

    #[test]
    fn width_pct_clamped_by_max_width() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 400.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new()
                .bg(Color32::RED)
                .width_pct(50.0)
                .max_width(120.0)
                .show(ui, |_ui| {});
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
        let rect = first_solid_rect(&shapes).expect("bg fill rect present");
        assert!(
            rect.width() <= 121.0,
            "width_pct(50).max_width(120) on 400px should cap at 120px, got {}",
            rect.width()
        );
    }

    #[test]
    fn height_pct_50_is_half_screen() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(raw, |ui| {
            StyledFrame::new().bg(Color32::RED).height_pct(50.0).show(ui, |_ui| {});
        });
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
        let rect = first_solid_rect(&shapes).expect("bg fill rect present");
        assert!(
            (rect.height() - 150.0).abs() < 2.0,
            "height_pct(50) on 300px screen should be ~150px, got {}",
            rect.height()
        );
    }
}
