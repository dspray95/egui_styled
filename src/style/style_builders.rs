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
