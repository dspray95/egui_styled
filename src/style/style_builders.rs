#[macro_export]
macro_rules! impl_style_builders {
    ($ty:ty) => {
        impl $ty {
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

            pub fn margin_top(mut self, val: i8) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.top = val;
                self.style.margin = Some(m);
                self
            }

            pub fn margin_bottom(mut self, val: i8) -> Self {
                let mut m = self.style.margin.unwrap_or_default();
                m.bottom = val;
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

            // --- Interaction ---

            pub fn cursor(mut self, icon: egui::CursorIcon) -> Self {
                self.style.cursor_icon = Some(icon);
                self
            }

            // --- Composition ---

            /// Allows applying a reusable style function to this builder.
            pub fn apply(self, f: impl FnOnce(Self) -> Self) -> Self {
                f(self)
            }
        }
    };
}
