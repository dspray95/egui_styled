#[macro_export]
macro_rules! impl_style_builders {
    ([$($gen:tt)*], $ty:ty) => {
        $crate::__impl_style_builders_body!([<$($gen)*>] $ty);
    };
    ($ty:ty) => {
        $crate::__impl_style_builders_body!([] $ty);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_style_builders_body {
    ([$($gen:tt)*] $ty:ty) => {
        impl $($gen)* $ty {
            // --- Background colors ---

            /// Set the base background fill colour.
            pub fn bg(mut self, color: egui::Color32) -> Self {
                self.style.bg = Some(color);
                self
            }

            /// Background fill while the pointer is hovering. Falls back to
            /// [`bg`](Self::bg) when unset.
            pub fn hover_bg(mut self, color: egui::Color32) -> Self {
                self.style.hover_bg = Some(color);
                self
            }

            /// Background fill while the widget is pressed/active. Falls back to
            /// [`hover_bg`](Self::hover_bg) / [`bg`](Self::bg) when unset.
            pub fn active_bg(mut self, color: egui::Color32) -> Self {
                self.style.active_bg = Some(color);
                self
            }

            /// Background fill while the widget has keyboard focus. Falls back to
            /// [`bg`](Self::bg) when unset.
            pub fn focus_bg(mut self, color: egui::Color32) -> Self {
                self.style.focus_bg = Some(color);
                self
            }

            /// Set the accent colour — maps to `selection.bg_fill` in egui
            /// (slider trailing fill, text-selection highlight, focus ring).
            pub fn accent(mut self, color: egui::Color32) -> Self {
                self.style.accent = Some(color);
                self
            }

            /// Accent colour while hovering. Falls back to [`accent`](Self::accent)
            /// when unset.
            pub fn hover_accent(mut self, color: egui::Color32) -> Self {
                self.style.hover_accent = Some(color);
                self
            }

            // --- Text properties ---

            /// Set the text/foreground colour for the widget's label.
            pub fn text_color(mut self, color: egui::Color32) -> Self {
                self.style.text_color = Some(color);
                self
            }

            /// Text/foreground colour while the pointer is hovering. Falls back
            /// to [`text_color`](Self::text_color) when unset.
            pub fn hover_text_color(mut self, color: egui::Color32) -> Self {
                self.style.hover_text_color = Some(color);
                self
            }

            /// Set the label font size in points. On widgets with a dedicated
            /// `font` builder, an explicit `font` takes precedence.
            pub fn font_size(mut self, size: f32) -> Self {
                self.style.font_size = Some(size);
                self
            }

            // --- Border / Stroke ---

            /// Set the base border stroke (`width` in points, `color`).
            pub fn border(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.border = Some(egui::Stroke::new(width, color));
                self
            }

            /// Border stroke while hovering. Falls back to [`border`](Self::border)
            /// when unset.
            pub fn hover_border(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.hover_border = Some(egui::Stroke::new(width, color));
                self
            }

            /// Border stroke while focused. Falls back to [`border`](Self::border)
            /// when unset.
            pub fn focus_border(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.focus_border = Some(egui::Stroke::new(width, color));
                self
            }

            // Per-side borders
            //
            // CSS-style individual edges. Each unset side falls back to the
            // uniform `border` for the same state. When any side is set, the
            // border is painted as straight per-edge line segments (no
            // corner-radius rounding of partial borders).

            /// Set the base top border (`width` in points, `color`).
            pub fn border_top(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.border_sides.top = Some(egui::Stroke::new(width, color));
                self
            }

            /// Set the base right border (`width` in points, `color`).
            pub fn border_right(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.border_sides.right = Some(egui::Stroke::new(width, color));
                self
            }

            /// Set the base bottom border (`width` in points, `color`).
            pub fn border_bottom(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.border_sides.bottom = Some(egui::Stroke::new(width, color));
                self
            }

            /// Set the base left border (`width` in points, `color`).
            pub fn border_left(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.border_sides.left = Some(egui::Stroke::new(width, color));
                self
            }

            /// Set the base left **and** right borders in one call.
            pub fn border_x(mut self, width: f32, color: egui::Color32) -> Self {
                let s = egui::Stroke::new(width, color);
                self.style.border_sides.left = Some(s);
                self.style.border_sides.right = Some(s);
                self
            }

            /// Set the base top **and** bottom borders in one call.
            pub fn border_y(mut self, width: f32, color: egui::Color32) -> Self {
                let s = egui::Stroke::new(width, color);
                self.style.border_sides.top = Some(s);
                self.style.border_sides.bottom = Some(s);
                self
            }

            /// Top border while hovering. Falls back to [`border_top`](Self::border_top).
            pub fn hover_border_top(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.hover_border_sides.top = Some(egui::Stroke::new(width, color));
                self
            }

            /// Right border while hovering. Falls back to [`border_right`](Self::border_right).
            pub fn hover_border_right(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.hover_border_sides.right = Some(egui::Stroke::new(width, color));
                self
            }

            /// Bottom border while hovering. Falls back to [`border_bottom`](Self::border_bottom).
            pub fn hover_border_bottom(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.hover_border_sides.bottom = Some(egui::Stroke::new(width, color));
                self
            }

            /// Left border while hovering. Falls back to [`border_left`](Self::border_left).
            pub fn hover_border_left(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.hover_border_sides.left = Some(egui::Stroke::new(width, color));
                self
            }

            /// Left and right borders while hovering.
            pub fn hover_border_x(mut self, width: f32, color: egui::Color32) -> Self {
                let s = egui::Stroke::new(width, color);
                self.style.hover_border_sides.left = Some(s);
                self.style.hover_border_sides.right = Some(s);
                self
            }

            /// Top and bottom borders while hovering.
            pub fn hover_border_y(mut self, width: f32, color: egui::Color32) -> Self {
                let s = egui::Stroke::new(width, color);
                self.style.hover_border_sides.top = Some(s);
                self.style.hover_border_sides.bottom = Some(s);
                self
            }

            /// Top border while focused. Falls back to [`border_top`](Self::border_top).
            pub fn focus_border_top(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.focus_border_sides.top = Some(egui::Stroke::new(width, color));
                self
            }

            /// Right border while focused. Falls back to [`border_right`](Self::border_right).
            pub fn focus_border_right(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.focus_border_sides.right = Some(egui::Stroke::new(width, color));
                self
            }

            /// Bottom border while focused. Falls back to [`border_bottom`](Self::border_bottom).
            pub fn focus_border_bottom(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.focus_border_sides.bottom = Some(egui::Stroke::new(width, color));
                self
            }

            /// Left border while focused. Falls back to [`border_left`](Self::border_left).
            pub fn focus_border_left(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.focus_border_sides.left = Some(egui::Stroke::new(width, color));
                self
            }

            /// Left and right borders while focused.
            pub fn focus_border_x(mut self, width: f32, color: egui::Color32) -> Self {
                let s = egui::Stroke::new(width, color);
                self.style.focus_border_sides.left = Some(s);
                self.style.focus_border_sides.right = Some(s);
                self
            }

            /// Top and bottom borders while focused.
            pub fn focus_border_y(mut self, width: f32, color: egui::Color32) -> Self {
                let s = egui::Stroke::new(width, color);
                self.style.focus_border_sides.top = Some(s);
                self.style.focus_border_sides.bottom = Some(s);
                self
            }

            // --- Geometry ---

            /// Set the corner radius. Accepts an `f32` (uniform) or any
            /// `Into<egui::CornerRadius>` for per-corner control.
            pub fn corner_radius(mut self, corner_radius: impl Into<egui::CornerRadius>) -> Self {
                self.style.corner_radius = Some(corner_radius.into());
                self
            }

            /// Set inner padding (space between the border and the content).
            /// Accepts any `Into<egui::Margin>`.
            pub fn padding(mut self, padding: impl Into<egui::Margin>) -> Self {
                self.style.padding = Some(padding.into());
                self
            }

            /// Set the top outer margin in points (space outside the border).
            pub fn margin_top(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.top = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            /// Set the bottom outer margin in points (space outside the border).
            pub fn margin_bottom(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.bottom = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            /// Set the left outer margin in points (space outside the border).
            pub fn margin_left(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.left = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            /// Set the right outer margin in points (space outside the border).
            pub fn margin_right(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.right = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            // --- Sizing ---

            /// Stretch the widget to fill the available width of its parent.
            pub fn full_width(mut self) -> Self {
                self.style.full_width = true;
                self
            }

            /// Stretch the widget to fill the available height of its parent.
            pub fn full_height(mut self) -> Self {
                self.style.full_height = true;
                self
            }

            /// Set a minimum width in points.
            pub fn min_width(mut self, width: f32) -> Self {
                self.style.min_width = Some(width);
                self
            }

            /// Set a maximum width in points.
            pub fn max_width(mut self, width: f32) -> Self {
                self.style.max_width = Some(width);
                self
            }

            /// Set a minimum height in points.
            pub fn min_height(mut self, height: f32) -> Self {
                self.style.min_height = Some(height);
                self
            }

            /// Set a maximum height in points.
            pub fn max_height(mut self, height: f32) -> Self {
                self.style.max_height = Some(height);
                self
            }

            /// Set width as a percentage (0–100) of the parent's available width.
            /// Resolves to a definite size at render time, superseding `full_width`.
            /// Composes with `min_width`/`max_width` as clamps after resolution.
            pub fn width_pct(mut self, pct: f32) -> Self {
                self.style.width_pct = Some(pct);
                self
            }

            /// Set height as a percentage (0–100) of the parent's available height.
            /// Resolves to a definite size at render time, superseding `full_height`.
            /// Composes with `min_height`/`max_height` as clamps after resolution.
            pub fn height_pct(mut self, pct: f32) -> Self {
                self.style.height_pct = Some(pct);
                self
            }

            /// Derive height from width (width ÷ height, e.g. `16.0/9.0`).
            /// Requires a definite width (`width_pct` or `full_width`); no-op otherwise.
            /// Overridden by an explicit `height_pct` or `full_height`.
            pub fn aspect_ratio(mut self, ratio: f32) -> Self {
                self.style.aspect_ratio = Some(ratio);
                self
            }

            // --- Interaction ---

            /// Set the cursor icon shown while hovering the widget.
            pub fn cursor(mut self, icon: egui::CursorIcon) -> Self {
                self.style.cursor_icon = Some(icon);
                self
            }

            /// Show or hide the widget. When `false` the widget still occupies
            /// its layout space but is not painted or interactive.
            pub fn visible(mut self, visible: bool) -> Self {
                self.style.visible = Some(visible);
                self
            }

            // --- Decorations ---

            // --- Background image ---

            /// Set a background texture drawn on top of `bg` fill, clipped to the
            /// same rounded rect. Accepts `egui::Image`, `egui::ImageSource`, or
            /// an `include_image!(...)` macro result.
            ///
            /// Texture loading is the app's responsibility — install loaders via
            /// `egui_extras::install_image_loaders` or register textures with
            /// `ctx.load_texture`. egui_styled only paints, never loads.
            pub fn background_image(mut self, image: impl Into<egui::Image<'static>>) -> Self {
                self.style.background_image = Some(image.into());
                self
            }

            /// Override the fill mode for `background_image`.
            /// Default: [`BackgroundImageFit::Stretch`](crate::BackgroundImageFit::Stretch) (maps full texture over the rect).
            pub fn background_image_fit(
                mut self,
                fit: $crate::style::shared_style::BackgroundImageFit,
            ) -> Self {
                self.style.background_image_fit = fit;
                self
            }

            /// Multiply the background image colour by `tint` (default: `WHITE` = no tint).
            pub fn background_image_tint(mut self, tint: egui::Color32) -> Self {
                self.style.background_image_tint = Some(tint);
                self
            }

            /// Fade the background image in over `seconds` the first time its
            /// texture finishes loading, instead of snapping in. Default: no fade.
            pub fn background_image_fade_in(mut self, seconds: f32) -> Self {
                self.style.background_image_fade_in = Some(seconds);
                self
            }

            /// Reveal the whole area — background image *and* body content — in
            /// together over `seconds`, the first time the image's texture
            /// finishes loading. The `bg` backdrop stays opaque, so the area
            /// reveals as a unit instead of content showing over a bare backdrop
            /// while the art is still decoding.
            pub fn reveal_with_background_image(mut self, seconds: f32) -> Self {
                self.style.background_image_fade_in = Some(seconds);
                self.style.background_image_fade_content = true;
                self
            }

            /// Paint an offset stroke rect behind the widget.
            /// Multiple calls append; each shadow uses the widget's `corner_radius`.
            pub fn shadow(mut self, offset: egui::Vec2, width: f32, color: egui::Color32) -> Self {
                self.style.shadows.push($crate::style::shared_style::Shadow {
                    offset,
                    stroke: egui::Stroke::new(width, color),
                    fill: None,
                });
                self
            }

            /// Paint a filled (+ optionally stroked) offset rect behind the widget.
            /// Use for conventional drop shadows.
            pub fn shadow_filled(mut self, offset: egui::Vec2, color: egui::Color32) -> Self {
                self.style.shadows.push($crate::style::shared_style::Shadow {
                    offset,
                    stroke: egui::Stroke::NONE,
                    fill: Some(color),
                });
                self
            }
        }

        impl $($gen)* $crate::apply::Apply for $ty {}
    };
}

#[cfg(test)]
mod tests {
    /// Compile-time coverage: every SharedStyle field must have a builder here.
    /// Add a call below whenever you add a field to SharedStyle. This test fails
    /// to COMPILE (not at runtime) if a builder is missing.
    #[test]
    fn all_shared_builders_compile() {
        use crate::Styled;
        let _ = Styled::frame()
            .bg(egui::Color32::RED)
            .hover_bg(egui::Color32::RED)
            .active_bg(egui::Color32::RED)
            .focus_bg(egui::Color32::RED)
            .accent(egui::Color32::RED)
            .hover_accent(egui::Color32::RED)
            .text_color(egui::Color32::RED)
            .hover_text_color(egui::Color32::RED)
            .font_size(14.0)
            .border(1.0, egui::Color32::RED)
            .hover_border(1.0, egui::Color32::RED)
            .focus_border(1.0, egui::Color32::RED)
            .border_top(1.0, egui::Color32::RED)
            .border_right(1.0, egui::Color32::RED)
            .border_bottom(1.0, egui::Color32::RED)
            .border_left(1.0, egui::Color32::RED)
            .corner_radius(4.0)
            .padding(8.0)
            .margin_top(4.0)
            .margin_bottom(4.0)
            .margin_left(4.0)
            .margin_right(4.0)
            .full_width()
            .full_height()
            .min_width(10.0)
            .max_width(100.0)
            .min_height(10.0)
            .max_height(100.0)
            .width_pct(50.0)
            .height_pct(50.0)
            .aspect_ratio(1.0)
            .cursor(egui::CursorIcon::Default)
            .visible(true)
            .shadow(egui::Vec2::ZERO, 1.0, egui::Color32::BLACK)
            .shadow_filled(egui::Vec2::ZERO, egui::Color32::BLACK);
        // font_id: set via widget-specific .font() builder — intentional omission.
        // background_image_fade_content: set via .reveal_with_background_image() — intentional alias.
    }
}
