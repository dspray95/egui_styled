use crate::state::PseudoState;

use egui::{Color32, CornerRadius, CursorIcon, FontId, Margin, Stroke, style::WidgetVisuals};

/// Properties common to most styled widgets
/// All fields are Option, None means it'll inherit egui defaults
#[derive(Clone, Default, Debug)]
pub struct SharedStyle {
    // Background
    pub bg: Option<Color32>,
    pub hover_bg: Option<Color32>,
    pub active_bg: Option<Color32>,
    pub focus_bg: Option<Color32>,

    // Text
    pub text_color: Option<Color32>,
    pub hover_text_color: Option<Color32>,
    pub font_size: Option<f32>,
    pub font_id: Option<FontId>,

    // Border
    pub border: Option<Stroke>,
    pub hover_border: Option<Stroke>,
    pub focus_border: Option<Stroke>,

    // Geometry
    pub corner_radius: Option<CornerRadius>,
    pub padding: Option<Margin>,
    pub margin: Option<Margin>,

    // Sizing
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub full_width: bool,

    // Cursor
    pub cursor_icon: Option<CursorIcon>,
}

/// Concrete style values for the current frame after
/// resolving pseudo-state and falling back to egui defaults
pub struct ResolvedStyle {
    pub bg: Color32,
    pub text_color: Color32,
    pub border: Stroke,
    pub corner_radius: CornerRadius,
    pub padding: Margin,
    pub margin: Margin,
    pub cursor_icon: Option<CursorIcon>,
}

impl SharedStyle {
    /// Resolve against current pseudo-state and egui's active visuals
    pub fn resolve(&self, state: PseudoState, default: &WidgetVisuals) -> ResolvedStyle {
        let bg = match state {
            _ if state.active && self.active_bg.is_some() => self.active_bg.unwrap(),
            _ if state.hovered && self.hover_bg.is_some() => self.hover_bg.unwrap(),
            _ if state.focused && self.focus_bg.is_some() => self.focus_bg.unwrap(),
            _ => self.bg.unwrap_or(default.bg_fill),
        };

        let border = match state {
            _ if state.focused && self.focus_border.is_some() => self.focus_border.unwrap(),
            _ if state.hovered && self.hover_border.is_some() => self.hover_border.unwrap(),
            _ => self.border.unwrap_or(default.bg_stroke),
        };

        let text_color = match state {
            _ if state.hovered && self.hover_text_color.is_some() => self.hover_text_color.unwrap(),
            _ => self.text_color.unwrap_or(default.text_color()),
        };

        ResolvedStyle {
            bg,
            text_color,
            border,
            corner_radius: self.corner_radius.unwrap_or(default.corner_radius),
            padding: self.padding.unwrap_or_default(),
            margin: self.margin.unwrap_or_default(),
            cursor_icon: self.cursor_icon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Visuals;

    fn default_visuals() -> WidgetVisuals {
        Visuals::default().widgets.inactive
    }

    fn style_with_all_bgs() -> SharedStyle {
        SharedStyle {
            bg: Some(Color32::RED),
            hover_bg: Some(Color32::GREEN),
            active_bg: Some(Color32::BLUE),
            focus_bg: Some(Color32::YELLOW),
            ..Default::default()
        }
    }

    #[test]
    fn bg_active_beats_hovered_and_focused() {
        let style = style_with_all_bgs();
        let state = PseudoState {
            hovered: true,
            active: true,
            focused: true,
        };
        let resolved = style.resolve(state, &default_visuals());
        assert_eq!(resolved.bg, Color32::BLUE);
    }

    #[test]
    fn bg_hovered_beats_focused() {
        let style = style_with_all_bgs();
        let state = PseudoState {
            hovered: true,
            active: false,
            focused: true,
        };
        let resolved = style.resolve(state, &default_visuals());
        assert_eq!(resolved.bg, Color32::GREEN);
    }

    #[test]
    fn bg_focused_used_when_only_focused() {
        let style = style_with_all_bgs();
        let state = PseudoState {
            hovered: false,
            active: false,
            focused: true,
        };
        let resolved = style.resolve(state, &default_visuals());
        assert_eq!(resolved.bg, Color32::YELLOW);
    }

    #[test]
    fn bg_base_used_when_no_pseudo_state() {
        let style = style_with_all_bgs();
        let resolved = style.resolve(PseudoState::default(), &default_visuals());
        assert_eq!(resolved.bg, Color32::RED);
    }

    #[test]
    fn falls_back_to_default_visuals_when_unset() {
        let style = SharedStyle::default();
        let visuals = default_visuals();
        let resolved = style.resolve(PseudoState::default(), &visuals);
        assert_eq!(resolved.bg, visuals.bg_fill);
        assert_eq!(resolved.border, visuals.bg_stroke);
        assert_eq!(resolved.corner_radius, visuals.corner_radius);
        assert_eq!(resolved.text_color, visuals.text_color());
    }

    #[test]
    fn hover_text_color_only_applies_when_hovered() {
        let style = SharedStyle {
            text_color: Some(Color32::RED),
            hover_text_color: Some(Color32::GREEN),
            ..Default::default()
        };
        let visuals = default_visuals();

        let active = PseudoState {
            hovered: false,
            active: true,
            focused: false,
        };
        assert_eq!(style.resolve(active, &visuals).text_color, Color32::RED);

        let focused = PseudoState {
            hovered: false,
            active: false,
            focused: true,
        };
        assert_eq!(style.resolve(focused, &visuals).text_color, Color32::RED);

        let hovered = PseudoState {
            hovered: true,
            active: false,
            focused: false,
        };
        assert_eq!(style.resolve(hovered, &visuals).text_color, Color32::GREEN);
    }

    #[test]
    fn border_focused_beats_hovered() {
        let hover = Stroke::new(1.0, Color32::GREEN);
        let focus = Stroke::new(2.0, Color32::YELLOW);
        let style = SharedStyle {
            hover_border: Some(hover),
            focus_border: Some(focus),
            ..Default::default()
        };
        let state = PseudoState {
            hovered: true,
            active: false,
            focused: true,
        };
        let resolved = style.resolve(state, &default_visuals());
        assert_eq!(resolved.border, focus);
    }
}
