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

            pub fn bg(mut self, color: egui::Color32) -> Self {
                self.style.bg = Some(color);
                self
            }

            pub fn hover_bg(mut self, color: egui::Color32) -> Self {
                self.style.hover_bg = Some(color);
                self
            }

            pub fn active_bg(mut self, color: egui::Color32) -> Self {
                self.style.active_bg = Some(color);
                self
            }

            pub fn focus_bg(mut self, color: egui::Color32) -> Self {
                self.style.focus_bg = Some(color);
                self
            }

            // --- Text properties ---

            pub fn text_color(mut self, color: egui::Color32) -> Self {
                self.style.text_color = Some(color);
                self
            }

            pub fn font_size(mut self, size: f32) -> Self {
                self.style.font_size = Some(size);
                self
            }

            // --- Border / Stroke ---

            pub fn border(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.border = Some(egui::Stroke::new(width, color));
                self
            }

            pub fn hover_border(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.hover_border = Some(egui::Stroke::new(width, color));
                self
            }

            pub fn focus_border(mut self, width: f32, color: egui::Color32) -> Self {
                self.style.focus_border = Some(egui::Stroke::new(width, color));
                self
            }

            // --- Geometry ---

            pub fn corner_radius(mut self, corner_radius: impl Into<egui::CornerRadius>) -> Self {
                self.style.corner_radius = Some(corner_radius.into());
                self
            }

            pub fn padding(mut self, padding: impl Into<egui::Margin>) -> Self {
                self.style.padding = Some(padding.into());
                self
            }

            pub fn margin_top(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.top = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            pub fn margin_bottom(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.bottom = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            pub fn margin_left(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.left = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            pub fn margin_right(mut self, val: f32) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.right = val.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                self.style.margin = Some(m);
                self
            }

            // --- Sizing ---

            pub fn full_width(mut self) -> Self {
                self.style.full_width = true;
                self
            }

            pub fn min_width(mut self, width: f32) -> Self {
                self.style.min_width = Some(width);
                self
            }

            pub fn max_width(mut self, width: f32) -> Self {
                self.style.max_width = Some(width);
                self
            }

            pub fn min_height(mut self, height: f32) -> Self {
                self.style.min_height = Some(height);
                self
            }

            pub fn max_height(mut self, height: f32) -> Self {
                self.style.max_height = Some(height);
                self
            }

            // --- Interaction ---

            pub fn cursor(mut self, icon: egui::CursorIcon) -> Self {
                self.style.cursor_icon = Some(icon);
                self
            }

            // --- Decorations ---

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
