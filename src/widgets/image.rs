use egui::{Color32, Id, Image, Response, Shape, Ui, Vec2};

use crate::{
    impl_style_builders,
    state::PseudoState,
    style::shared_style::{SharedStyle, paint_shadows, render_scoped},
};

/// An inline image widget (icon, portrait, thumbnail) with themed styling.
///
/// Carries a [`SharedStyle`] so it picks up `.corner_radius()`, `.border()`,
/// `.shadow()`, `.visible()`, etc. like every other styled widget. Hover state
/// is tracked so `.hover_*` builders and a distinct `hover_tint` work.
///
/// Texture loading is the app's responsibility. Install egui image loaders via
/// `egui_extras::install_image_loaders` or register a native texture yourself
/// with `ctx.load_texture`. egui_styled only presents, never loads.
///
/// # Example
/// ```ignore
/// Styled::image(egui::include_image!("assets/icon.png"))
///     .corner_radius(8.0)
///     .border(1.0, theme.border_color)
///     .max_size(egui::Vec2::splat(64.0))
///     .show(ui);
/// ```
pub struct StyledImage {
    image: Image<'static>,
    tint: Option<Color32>,
    hover_tint: Option<Color32>,
    fit: ImageFitSpec,
    id_override: Option<Id>,
    style: SharedStyle,
}

/// Sizing / fit specification for [`StyledImage`].
///
/// Mirrors the subset of `egui::Image` fit builders that are meaningful for an
/// inline widget. `None` (the default) defers to egui's own default behaviour:
/// fill available width while maintaining aspect ratio.
#[derive(Clone, Copy, Debug, Default)]
enum ImageFitSpec {
    #[default]
    Default,
    ExactSize(Vec2),
    MaxSize(Vec2),
    FitToFraction(Vec2),
    FitToOriginal(f32),
}

impl StyledImage {
    /// Create a new `StyledImage` from any value that can become an
    /// `egui::Image<'static>`: a finished `Image`, an `ImageSource`, or the
    /// result of `egui::include_image!(...)`.
    pub fn new(image: impl Into<Image<'static>>) -> Self {
        Self {
            image: image.into(),
            tint: None,
            hover_tint: None,
            fit: ImageFitSpec::Default,
            id_override: None,
            style: SharedStyle::default(),
        }
    }

    /// Multiply the image colour by `tint` (default: `WHITE` = no tint).
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = Some(tint);
        self
    }

    /// Override the tint when the pointer hovers over the image.
    /// Falls back to `tint` (or `WHITE`) when unset.
    pub fn hover_tint(mut self, tint: Color32) -> Self {
        self.hover_tint = Some(tint);
        self
    }

    /// Render the image at exactly this pixel size.
    pub fn size(mut self, size: Vec2) -> Self {
        self.fit = ImageFitSpec::ExactSize(size);
        self
    }

    /// Constrain the image to fit within this bounding box while maintaining
    /// aspect ratio.
    pub fn max_size(mut self, size: Vec2) -> Self {
        self.fit = ImageFitSpec::MaxSize(size);
        self
    }

    /// Scale the image to a fraction of the available space.
    pub fn fit_to_fraction(mut self, fraction: Vec2) -> Self {
        self.fit = ImageFitSpec::FitToFraction(fraction);
        self
    }

    /// Render at the image's original pixel size multiplied by `scale`.
    pub fn fit_to_original(mut self, scale: f32) -> Self {
        self.fit = ImageFitSpec::FitToOriginal(scale);
        self
    }

    /// Override the auto-generated widget id. Use this to pin pseudo-state
    /// (hover / active / focus) across conditional rendering — without an
    /// explicit id, `ui.next_auto_id()` shifts when a sibling appears or
    /// disappears, misattributing one frame of state.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_override = Some(Id::new(id));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let visible = self.style.visible != Some(false);

        let id = self
            .id_override
            .unwrap_or_else(|| ui.make_persistent_id(ui.next_auto_id()));

        let pseudo = PseudoState::load(ui, id);
        let per = self.style.resolve_per_state(ui.visuals());

        let response = render_scoped(ui, visible, |ui| {
            let shadow_idx = ui.painter().add(Shape::Noop);

            let current_tint = if pseudo.hovered {
                self.hover_tint.or(self.tint).unwrap_or(Color32::WHITE)
            } else {
                self.tint.unwrap_or(Color32::WHITE)
            };

            // Apply fit and corner radius to the egui::Image descriptor.
            // corner_radius is applied directly on the Image — egui's tessellator
            // clips the texture to the rounded rect, avoiding squared-off corners.
            let mut img = self.image.corner_radius(per.corner_radius).tint(current_tint);
            match self.fit {
                ImageFitSpec::Default => {}
                ImageFitSpec::ExactSize(s) => img = img.fit_to_exact_size(s),
                ImageFitSpec::MaxSize(s) => img = img.max_size(s),
                ImageFitSpec::FitToFraction(f) => img = img.fit_to_fraction(f),
                ImageFitSpec::FitToOriginal(s) => img = img.fit_to_original_size(s),
            }

            let mut wrapper = egui::Frame::new();
            if per.margin != egui::Margin::ZERO {
                wrapper = wrapper.outer_margin(per.margin);
            }

            let resp = wrapper
                .show(ui, |ui| {
                    if self.style.full_width {
                        ui.set_min_width(ui.available_width());
                    }
                    let resp = ui.add(img);

                    // Paint border on top of the image (Outside so it doesn't
                    // clip into the texture area).
                    if per.inactive.border != egui::Stroke::NONE {
                        let border = if pseudo.hovered {
                            per.hovered.border
                        } else {
                            per.inactive.border
                        };
                        ui.painter().rect_stroke(
                            resp.rect,
                            per.corner_radius,
                            border,
                            egui::StrokeKind::Outside,
                        );
                    }

                    resp
                })
                .inner;

            paint_shadows(ui, shadow_idx, resp.rect, per.corner_radius, &self.style.shadows);

            resp
        });

        PseudoState::from_response(&response).store(ui, id);

        if let Some(icon) = per.cursor_icon
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(icon);
        }

        let _ = pseudo;

        response
    }
}

impl_style_builders!(StyledImage);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tint_is_none() {
        let img = StyledImage::new(egui::Image::from_bytes("bytes://test", vec![]));
        assert!(img.tint.is_none());
        assert!(img.hover_tint.is_none());
    }

    #[test]
    fn tint_builder_sets_field() {
        let img = StyledImage::new(egui::Image::from_bytes("bytes://test", vec![]))
            .tint(Color32::RED);
        assert_eq!(img.tint, Some(Color32::RED));
    }

    #[test]
    fn hover_tint_builder_sets_field() {
        let img = StyledImage::new(egui::Image::from_bytes("bytes://test", vec![]))
            .hover_tint(Color32::BLUE);
        assert_eq!(img.hover_tint, Some(Color32::BLUE));
    }

    #[test]
    fn size_builder_sets_exact_fit() {
        let size = Vec2::new(64.0, 64.0);
        let img = StyledImage::new(egui::Image::from_bytes("bytes://test", vec![]))
            .size(size);
        assert!(matches!(img.fit, ImageFitSpec::ExactSize(s) if s == size));
    }

    #[test]
    fn max_size_builder_sets_max_fit() {
        let size = Vec2::new(128.0, 128.0);
        let img = StyledImage::new(egui::Image::from_bytes("bytes://test", vec![]))
            .max_size(size);
        assert!(matches!(img.fit, ImageFitSpec::MaxSize(s) if s == size));
    }

    #[test]
    fn visible_false_captured_in_style() {
        let img = StyledImage::new(egui::Image::from_bytes("bytes://test", vec![]))
            .visible(false);
        assert_eq!(img.style.visible, Some(false));
    }

    #[test]
    fn id_override_sets_field() {
        let img = StyledImage::new(egui::Image::from_bytes("bytes://test", vec![]))
            .id("my_image");
        assert!(img.id_override.is_some());
    }
}
